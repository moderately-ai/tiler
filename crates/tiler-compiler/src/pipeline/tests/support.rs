use super::*;

/// A retained root record the stage chain hangs from.
pub(super) fn test_root(explain: &mut ExplainWriter) -> ExplainRecordId {
    let subject = explain
        .subject(SubjectKind::SemanticProgram, "semantic-program")
        .unwrap();
    explain
        .push_detail(
            RuleRef::builtin("test.root").unwrap(),
            vec![subject],
            check(
                ExplainStage::RequestVerification,
                "test.root",
                EvidenceBasis::CheckedInvariant,
            )
            .unwrap(),
            Vec::new(),
        )
        .unwrap()
}

pub(super) fn semantic(reverse_constants: bool) -> SemanticProgram {
    semantic_case(
        Shape::from_dims([2, 2]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        reverse_constants,
    )
}

pub(super) fn request_with_targets(
    program: &SemanticProgram,
    target_profiles: Vec<TargetProfile>,
    contracts: Vec<StrictF32NumericalContract>,
) -> CompilationRequest<'_> {
    let mut request = CompilationRequest::governed_preferring(
        program,
        NumericalContractPreference::ordered(contracts).expect("the fixture states a contract"),
    );
    request.target_profiles = target_profiles;
    request
}

pub(super) fn outcome_for_key<'a>(
    product: &'a CompilationProduct,
    key: &str,
) -> &'a TargetCompilationOutcome {
    product
        .targets
        .iter()
        .find(|outcome| outcome.target_profile().profile_key().as_str() == key)
        .unwrap_or_else(|| panic!("missing target outcome for {key}"))
}

pub(super) fn semantic_case(
    shape: Shape,
    scale_bits: u32,
    bias_bits: u32,
    reverse_constants: bool,
) -> SemanticProgram {
    semantic_case_with_axis(
        shape,
        scale_bits,
        bias_bits,
        reverse_constants,
        Axis::new(1),
    )
}

pub(super) fn semantic_case_with_axis(
    shape: Shape,
    scale_bits: u32,
    bias_bits: u32,
    reverse_constants: bool,
    reduction_axis: Axis,
) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape)
        .unwrap();
    let (scale, bias) = if reverse_constants {
        let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
        let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
        (scale, bias)
    } else {
        let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
        let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
        (scale, bias)
    };
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [reduction_axis]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

pub(super) fn tensor_add_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let first = F32Constant::apply(&mut builder, 1.0e20_f32.to_bits()).unwrap();
    let second = F32Constant::apply(&mut builder, (-1.0e20_f32).to_bits()).unwrap();
    let left = F32Add::apply(&mut builder, input, first).unwrap();
    let root = F32Add::apply(&mut builder, left, second).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// One typed value produced while interpreting a structured kernel.
#[derive(Clone, Copy, Debug)]
pub(super) enum KirValue {
    Bool(bool),
    Index(u64),
    F32(f32),
    /// A `bf16` value, carried as its exact encoding.
    ///
    /// Bits rather than a host float, because there is no host `bf16` type whose
    /// arithmetic could stand in for the format's: modelling it as an `f32`
    /// would round to twenty-four significand bits where the format has eight,
    /// and would agree with the reference only where the extra precision
    /// happened not to be observable.
    Bf16(u16),
}

impl KirValue {
    fn index(self) -> u64 {
        match self {
            Self::Index(value) => value,
            other => panic!("expected an index-typed value, found {other:?}"),
        }
    }
    fn float(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("expected an f32-typed value, found {other:?}"),
        }
    }
    fn bf16(self) -> u16 {
        match self {
            Self::Bf16(value) => value,
            other => panic!("expected a bf16-typed value, found {other:?}"),
        }
    }
    fn boolean(self) -> bool {
        match self {
            Self::Bool(value) => value,
            other => panic!("expected a predicate value, found {other:?}"),
        }
    }
}

/// The typed boundary payload one interpreted kernel reads and writes.
///
/// A typed pair rather than one byte run, because the machine has to produce
/// typed SSA values from a load and cannot recover a width from raw bytes. The
/// kernel's own buffer parameters state the element type, and a fixture whose
/// payload disagrees with them fails on the value kind rather than by silently
/// reinterpreting the bytes.
#[derive(Clone, Copy, Debug)]
pub(super) enum KirElements<'a> {
    F32(&'a [f32]),
    Bf16(&'a [u16]),
}

impl KirElements<'_> {
    fn len(self) -> usize {
        match self {
            Self::F32(values) => values.len(),
            Self::Bf16(values) => values.len(),
        }
    }
}

/// The written boundary payload one interpreted kernel produces.
#[derive(Clone, Debug)]
pub(super) enum KirOutputs {
    F32(Vec<f32>),
    Bf16(Vec<u16>),
}

/// Whether the machine honours the kernel's BF16 NaN canonicalization.
///
/// **[`Self::Omitted`] exists for one deliberate perturbation and nothing else.**
/// It models a lowering that emitted the BF16 arithmetic without the
/// `CanonicalizeBf16Nan` conversion beside it, so the reference comparison that
/// passes under [`Self::Applied`] can be watched failing — and failing at exactly
/// the element whose result is a NaN, which is what makes the conversion's
/// obligation the reason rather than a coincidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Bf16Canonicalization {
    Applied,
    Omitted,
}

/// A backend-shaped interpreter that reads only the structured kernel IR.
///
/// It resolves nothing from the semantic graph, the request, or the
/// schedule: buffer roles and extents, addressing, predication, reduction
/// order, and NaN canonicalization all come from the kernel itself. That is
/// the property the KIR layer exists to guarantee, so exercising it against
/// the reference evaluator is the end-to-end proof that a backend needs no
/// graph-specific reconstruction.
pub(super) struct KirMachine<'a> {
    kernel: &'a VerifiedKernel,
    /// One boundary payload per declared read buffer, in declaration order.
    ///
    /// A list rather than one payload, because a region reads one tensor per
    /// program input and the families that compute nothing are only worth
    /// compiling beside the operand they address: the workload's occurrence is
    /// `activation * broadcast(weight)`, whose region binds two reads at
    /// *different* element counts. A machine holding one payload cannot model
    /// that at all — the widened read addresses its operand's range, which is
    /// the smaller of the two.
    inputs: Vec<KirElements<'a>>,
    /// Position in [`Self::inputs`] for each declared read buffer.
    ///
    /// Resolved from the kernel's own signature rather than from load order: a
    /// buffer parameter's position *is* its argument-table ordinal, and a body
    /// that happens to load its second operand first would otherwise bind the
    /// payloads the wrong way round and still produce a plausible tensor.
    reads: BTreeMap<tiler_ir::kernel::VerifiedBufferId, usize>,
    output: KirOutputs,
    canonicalization: Bf16Canonicalization,
    values: BTreeMap<tiler_ir::kernel::VerifiedValueId, KirValue>,
    /// The lane's local coordinate within its workgroup.
    ///
    /// Equal to the global invocation index for every single-lane kernel, which
    /// is why the pre-cooperative machine needed no second field. A cooperative
    /// kernel reads both, and giving them one value would make its staged store
    /// address the slot its output coordinate names.
    local: u64,
    /// The workgroup's shared staging, when it declares any.
    staged: Vec<f32>,
}

impl<'a> KirMachine<'a> {
    fn run(
        kernel: &'a VerifiedKernel,
        inputs: &[KirElements<'a>],
        canonicalization: Bf16Canonicalization,
    ) -> KirOutputs {
        // Walked once over the declared signature, so the read population is
        // *counted* against the payloads offered rather than consumed until one
        // side runs out: a fixture binding three payloads to a two-read kernel
        // would otherwise leave the third silently unread.
        let mut reads = BTreeMap::new();
        let mut write = None;
        for (id, parameter) in kernel.declared_buffers() {
            match parameter.access {
                tiler_ir::kernel::BufferAccess::Read => {
                    let position = reads.len();
                    let payload = inputs
                        .get(position)
                        .unwrap_or_else(|| panic!("no payload bound for read buffer {position}"));
                    // Against the *parameter's* own count, which is the read's
                    // addressable range and not the region's domain. A widening
                    // broadcast is exactly the case where the two differ, so
                    // comparing against the domain would admit a payload the
                    // kernel can address past the end of.
                    assert_eq!(
                        payload.len(),
                        usize::try_from(parameter.element_count).unwrap(),
                        "the payload bound to read buffer {position} is not its declared length",
                    );
                    reads.insert(id, position);
                }
                tiler_ir::kernel::BufferAccess::Write => {
                    assert!(
                        write.replace(parameter).is_none(),
                        "this machine models one written boundary",
                    );
                }
            }
        }
        assert_eq!(
            reads.len(),
            inputs.len(),
            "the fixture bound a payload the kernel declares no read buffer for",
        );
        let write = write.expect("a write buffer parameter");
        let outputs = usize::try_from(write.element_count).unwrap();
        // Read from the kernel's own staging declaration, so the machine still
        // resolves nothing from the schedule, the request, or the graph: a
        // kernel that stages nothing runs one lane per output exactly as before.
        let slots = kernel
            .staging()
            .next()
            .map_or(1, |staging| staging.element_count.max(1));
        let participants = usize::try_from(slots).unwrap();
        // The written payload's type is the *write buffer's* own, so a kernel
        // whose two boundaries differed in width would still be modelled at each
        // one rather than at whichever the input happened to be.
        let output = match write.element_type {
            tiler_ir::kernel::KernelType::Bf16 => KirOutputs::Bf16(vec![0; outputs]),
            _ => KirOutputs::F32(vec![f32::NAN; outputs]),
        };
        let mut machine = KirMachine {
            kernel,
            inputs: inputs.to_vec(),
            reads,
            output,
            canonicalization,
            values: BTreeMap::new(),
            local: 0,
            staged: vec![f32::NAN; participants],
        };
        for workgroup in 0..outputs {
            // One value map per lane, carried across the barrier: a lane's
            // pre-barrier definitions are exactly what its post-barrier block
            // reads, and clearing them at the barrier would make every
            // cooperative body fail on an undefined value rather than on the
            // property under test.
            let mut lanes = vec![BTreeMap::new(); participants];
            machine.staged.fill(f32::NAN);
            // Barrier semantics, structurally: every lane runs the segment
            // before the barrier, and only then does any lane run the segment
            // after it. The KIR verifier requires every barrier at block depth
            // zero, which is what makes splitting the top-level operation list
            // the faithful model rather than an approximation.
            for segment in barrier_segments(kernel.body()) {
                for (lane, values) in lanes.iter_mut().enumerate() {
                    let lane = u64::try_from(lane).unwrap();
                    machine.values = std::mem::take(values);
                    machine.local = lane;
                    let invocation = u64::try_from(workgroup).unwrap() * slots + lane;
                    for operation in &segment {
                        machine.run_operation(*operation, invocation);
                    }
                    *values = std::mem::take(&mut machine.values);
                }
            }
        }
        machine.output
    }

    fn run_block(&mut self, block: tiler_ir::kernel::BlockRef<'a>, invocation: u64) {
        for operation in block.operations() {
            self.run_operation(operation, invocation);
        }
    }

    fn run_operation(&mut self, operation: tiler_ir::kernel::OperationRef<'a>, invocation: u64) {
        {
            let mut results = operation.results();
            match operation.view() {
                OperationView::Builtin { builtin } => {
                    let value = match builtin {
                        tiler_ir::kernel::Builtin::GlobalInvocationIndex => invocation,
                        tiler_ir::kernel::Builtin::LocalInvocationIndex => self.local,
                        other => panic!("unsupported launch builtin {other:?}"),
                    };
                    self.define(&mut results, KirValue::Index(value));
                }
                OperationView::Constant { value } => {
                    let value = match value {
                        KernelConstant::Bool(flag) => KirValue::Bool(flag),
                        KernelConstant::Index(index) => KirValue::Index(index),
                        KernelConstant::F32Bits(bits) => KirValue::F32(f32::from_bits(bits)),
                        // Carried through unchanged: a constant is not an
                        // arithmetic result, and `tiler::constant-bf16@1`
                        // declares its payload preserved exactly.
                        KernelConstant::Bf16Bits(bits) => KirValue::Bf16(bits),
                        other => panic!("unsupported constant {other:?}"),
                    };
                    self.define(&mut results, value);
                }
                OperationView::Binary { op, lhs, rhs } => {
                    let value = match op {
                        BinaryOp::IndexAdd => {
                            KirValue::Index(self.get(lhs).index() + self.get(rhs).index())
                        }
                        BinaryOp::IndexMultiply => {
                            KirValue::Index(self.get(lhs).index() * self.get(rhs).index())
                        }
                        BinaryOp::IndexDivide => {
                            KirValue::Index(self.get(lhs).index() / self.get(rhs).index())
                        }
                        BinaryOp::IndexModulo => {
                            KirValue::Index(self.get(lhs).index() % self.get(rhs).index())
                        }
                        // Asserted rather than wrapped. The operation's contract
                        // is that its result is proven non-negative — a reindex
                        // mirror subtracts a `% extent` result from `extent - 1`
                        // — so an underflow here is a defect in the map that
                        // produced it, and a machine that wrapped would model a
                        // behaviour the kernel vocabulary does not define.
                        BinaryOp::IndexSubtract => {
                            let (lhs, rhs) = (self.get(lhs).index(), self.get(rhs).index());
                            assert!(
                                lhs >= rhs,
                                "an index subtraction underflowed: {lhs} - {rhs}",
                            );
                            KirValue::Index(lhs - rhs)
                        }
                        BinaryOp::F32Add => {
                            KirValue::F32(self.get(lhs).float() + self.get(rhs).float())
                        }
                        BinaryOp::F32Multiply => {
                            KirValue::F32(self.get(lhs).float() * self.get(rhs).float())
                        }
                        // The operator division, deliberately, and not a
                        // reciprocal followed by a multiply: the two round a
                        // different number of times, so a machine that took the
                        // second spelling would agree with the reference only
                        // where the difference happens not to be observable.
                        BinaryOp::F32Divide => {
                            KirValue::F32(self.get(lhs).float() / self.get(rhs).float())
                        }
                        // The `bf16` arithmetic, computed at the format's own
                        // precision rather than at a host float's. See
                        // [`bf16_binary`] for why an exact `f64` intermediate
                        // followed by one rounding is the format's function and
                        // not an approximation of it.
                        BinaryOp::Bf16Add => KirValue::Bf16(bf16_binary(
                            self.get(lhs).bf16(),
                            self.get(rhs).bf16(),
                            |lhs, rhs| lhs + rhs,
                        )),
                        BinaryOp::Bf16Multiply => KirValue::Bf16(bf16_binary(
                            self.get(lhs).bf16(),
                            self.get(rhs).bf16(),
                            |lhs, rhs| lhs * rhs,
                        )),
                        other => panic!("unsupported binary operation {other:?}"),
                    };
                    self.define(&mut results, value);
                }
                OperationView::Unary { op, source } => {
                    let argument = self.get(source).float();
                    let value = match op {
                        // The *certified* exponential rather than the host
                        // library's. `UnaryOp::F32Exp` names the precise
                        // function, whose admitted result set is the registered
                        // accuracy contract; modelling it with `f32::exp` would
                        // make this machine agree with the reference only where
                        // the host happened to round the same way, which is a
                        // property of the host and not of the kernel.
                        UnaryOp::F32Exp => tiler_reference::certified_exp_f32(argument)
                            .expect("the certified exponential decides every reachable argument"),
                        UnaryOp::F32Rsqrt => tiler_reference::certified_rsqrt_f32(argument)
                            .expect("the certified reciprocal square root decides its arguments"),
                    };
                    self.define(&mut results, KirValue::F32(value));
                }
                OperationView::Compare { op, lhs, rhs } => {
                    let value = match op {
                        CompareOp::IndexLessThan => {
                            KirValue::Bool(self.get(lhs).index() < self.get(rhs).index())
                        }
                        other => panic!("unsupported comparison {other:?}"),
                    };
                    self.define(&mut results, value);
                }
                OperationView::Convert { op, source } => {
                    let value = match op {
                        ConvertOp::CanonicalizeF32Nan => {
                            let value = self.get(source).float();
                            KirValue::F32(if value.is_nan() {
                                f32::from_bits(
                                    self.kernel.numerical().canonical_arithmetic_nan_bits,
                                )
                            } else {
                                value
                            })
                        }
                        // The payload is read from the kernel's own realization
                        // and taken as this region's arithmetic type's pattern
                        // zero-extended, which is the invariant the schedule
                        // verifier requires of a BF16 region — so a realization
                        // carrying an `f32` payload could not have reached here.
                        ConvertOp::CanonicalizeBf16Nan => {
                            let bits = self.get(source).bf16();
                            let canonical = u16::try_from(
                                self.kernel.numerical().canonical_arithmetic_nan_bits,
                            )
                            .expect("a bf16 region declares a sixteen-bit canonical payload");
                            KirValue::Bf16(match (self.canonicalization, bf16_is_nan(bits)) {
                                (Bf16Canonicalization::Applied, true) => canonical,
                                _ => bits,
                            })
                        }
                        other => panic!("unsupported conversion {other:?}"),
                    };
                    self.define(&mut results, value);
                }
                // Addressed through the buffer the load *names*, not through
                // whichever payload the machine holds. That is the whole reason
                // a structural region is interpretable here: the widened read
                // and the dense one differ in nothing a body-shaped model sees
                // except which parameter they carry.
                OperationView::Load { buffer, offset, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    let position = *self
                        .reads
                        .get(&buffer)
                        .expect("a load names a declared read buffer");
                    let value = match self.inputs[position] {
                        KirElements::F32(values) => KirValue::F32(values[offset]),
                        KirElements::Bf16(values) => KirValue::Bf16(values[offset]),
                    };
                    self.define(&mut results, value);
                }
                OperationView::Store { offset, value, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    let value = self.get(value);
                    match &mut self.output {
                        KirOutputs::F32(values) => values[offset] = value.float(),
                        KirOutputs::Bf16(values) => values[offset] = value.bf16(),
                    }
                }
                OperationView::Predicated { predicate, body } => {
                    if self.get(predicate).boolean() {
                        self.run_block(body, invocation);
                    }
                }
                OperationView::SerialLoop(reduction) => {
                    let mut carried: Vec<KirValue> =
                        reduction.initial().map(|value| self.get(value)).collect();
                    let induction = reduction.induction().expect("an induction variable");
                    let parameters: Vec<_> = reduction.accumulators().collect();
                    for step in reduction.start()..reduction.end() {
                        self.values.insert(induction, KirValue::Index(step));
                        for (parameter, value) in parameters.iter().zip(&carried) {
                            self.values.insert(*parameter, *value);
                        }
                        self.run_block(reduction.body(), invocation);
                        carried = reduction.yields().map(|value| self.get(value)).collect();
                    }
                    for (result, value) in results.zip(carried) {
                        self.values.insert(result, value);
                    }
                }
                // Reached only inside a nested block, which the KIR verifier
                // forbids; a top-level barrier is consumed by `barrier_segments`
                // and never executed as an operation.
                OperationView::Barrier { .. } => {
                    panic!("a barrier below block depth zero reached the machine")
                }
                OperationView::StagedStore { offset, value, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    self.staged[offset] = self.get(value).float();
                }
                OperationView::StagedLoad { offset, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    let value = KirValue::F32(self.staged[offset]);
                    self.define(&mut results, value);
                }
                other => panic!("unsupported structured operation {other:?}"),
            }
        }
    }

    fn define(
        &mut self,
        results: &mut impl Iterator<Item = tiler_ir::kernel::VerifiedValueId>,
        value: KirValue,
    ) {
        let result = results.next().expect("one defined result");
        self.values.insert(result, value);
    }

    fn get(&self, id: tiler_ir::kernel::VerifiedValueId) -> KirValue {
        *self
            .values
            .get(&id)
            .expect("a value defined before its use")
    }
}

pub(crate) fn interpret_fused(kernel: &VerifiedKernel, input: &[f32]) -> Vec<f32> {
    interpret_fused_inputs(kernel, &[input])
}

/// Interprets one `f32` kernel over one payload per declared read buffer.
///
/// `inputs` is in the kernel's own buffer declaration order, which the region
/// builder fixes as the recognized read list followed by the owning write. For
/// almost every program that is the declared inputs in declaration order; a
/// program reading one input both densely and through a relation has two entries
/// for it, and binds the same payload twice.
pub(crate) fn interpret_fused_inputs(kernel: &VerifiedKernel, inputs: &[&[f32]]) -> Vec<f32> {
    let payloads: Vec<KirElements<'_>> = inputs.iter().copied().map(KirElements::F32).collect();
    match KirMachine::run(kernel, &payloads, Bf16Canonicalization::Applied) {
        KirOutputs::F32(values) => values,
        KirOutputs::Bf16(_) => panic!("an f32 fixture produced a bf16 boundary"),
    }
}

/// Interprets one BF16 kernel over a BF16 boundary payload.
pub(super) fn interpret_bf16(
    kernel: &VerifiedKernel,
    input: &[u16],
    canonicalization: Bf16Canonicalization,
) -> Vec<u16> {
    match KirMachine::run(kernel, &[KirElements::Bf16(input)], canonicalization) {
        KirOutputs::Bf16(values) => values,
        KirOutputs::F32(_) => panic!("a bf16 fixture produced an f32 boundary"),
    }
}

/// Whether one `bf16` encoding is a NaN.
///
/// Decided from the format's own fields — a saturated exponent over a nonzero
/// significand — because there is no host `bf16` whose `is_nan` could be asked.
pub(super) const fn bf16_is_nan(bits: u16) -> bool {
    bits & 0x7f80 == 0x7f80 && bits & 0x007f != 0
}

/// The exact value of one `bf16` encoding, as an `f64`.
///
/// Exact, not approximate: `bf16` has eight significand bits and an exponent
/// range inside `f64`'s, so every finite encoding — including every subnormal —
/// is an `f64` value. Infinities and NaNs never reach here; [`bf16_binary`]
/// decides those from the encoding.
pub(super) fn bf16_exact_value(bits: u16) -> f64 {
    let sign = if bits & 0x8000 == 0 { 1.0_f64 } else { -1.0 };
    let exponent = i32::from((bits >> 7) & 0xff);
    let fraction = f64::from(bits & 0x007f);
    if exponent == 0 {
        // Subnormal or zero: `fraction` quanta of `2^-133`.
        sign * fraction * 2.0_f64.powi(-133)
    } else {
        sign * (1.0 + fraction / 128.0) * 2.0_f64.powi(exponent - 127)
    }
}

/// Rounds one exact `f64` to `bf16`, round-to-nearest ties-to-even.
///
/// The rounding is performed on the `f64` significand rather than by a host
/// narrowing conversion, so subnormals, the tie rule, and the overflow-to-
/// infinity boundary are the format's own rather than whatever a two-step
/// conversion through `f32` would produce.
pub(super) fn bf16_round(value: f64) -> u16 {
    let sign: u16 = if value.is_sign_negative() { 0x8000 } else { 0 };
    if value.is_infinite() {
        return sign | 0x7f80;
    }
    if value == 0.0 {
        return sign;
    }
    let bits = value.abs().to_bits();
    let biased = i32::try_from((bits >> 52) & 0x7ff).expect("an eleven-bit field");
    assert!(
        biased != 0,
        "no product or sum of two bf16 values is subnormal in f64"
    );
    let exponent = biased - 1023;
    // `|value| == mantissa * 2^(exponent - 52)`, with the implicit bit restored.
    let mantissa = (1_u64 << 52) | (bits & ((1_u64 << 52) - 1));
    // The bf16 grid's quantum at this exponent, floored at the subnormal one.
    let quantum = (exponent - 7).max(-133);
    let shift = u32::try_from(quantum - (exponent - 52))
        .expect("the bf16 quantum is never finer than the f64 one");
    let rounded = round_half_to_even(mantissa, shift);
    if quantum == -133 {
        // A subnormal count, or — when the rounding carried into the eighth bit
        // — the smallest normal, whose encoding is exactly that count.
        return sign | u16::try_from(rounded).expect("a subnormal count is below 2^8");
    }
    // `rounded` is the eight-bit significand in `[2^7, 2^8]`; the upper end is
    // the carry that increments the exponent.
    let (exponent, fraction) = if rounded == 256 {
        (exponent + 1, 0)
    } else {
        (
            exponent,
            u16::try_from(rounded - 128).expect("a seven-bit fraction"),
        )
    };
    let biased = exponent + 127;
    if biased > 254 {
        return sign | 0x7f80;
    }
    sign | (u16::try_from(biased).expect("a biased exponent below 255") << 7) | fraction
}

/// Rounds `mantissa >> shift` half-to-even.
pub(super) fn round_half_to_even(mantissa: u64, shift: u32) -> u64 {
    if shift >= 64 {
        // The mantissa is below `2^53` and the half is at least `2^63`, so every
        // bit is under the rounding boundary and the result is zero.
        return 0;
    }
    let half = 1_u64 << (shift - 1);
    let low = mantissa & (half - 1 + half);
    let truncated = mantissa >> shift;
    if low > half || (low == half && truncated & 1 == 1) {
        truncated + 1
    } else {
        truncated
    }
}

/// One ordered `bf16` binary operation, shaped like hardware rather than like
/// the contract.
///
/// **The arithmetic is exact and rounds once.** `f64` holds the product of two
/// `bf16` values exactly — eight significand bits times eight is sixteen, well
/// inside fifty-three, and the exponent range is `f64`'s with room to spare — so
/// the multiply's only rounding is [`bf16_round`]'s. The sum is exact whenever
/// the operands' exponents differ by at most forty-five and otherwise rounds in
/// `f64` by less than `2^-53` relative, which cannot cross a `bf16` rounding
/// boundary sitting at `2^-9` relative granularity: the result is the larger
/// operand either way. So one rounding, at the format's own precision, in both
/// cases.
///
/// **A NaN result keeps the operand's payload rather than the contract's.** That
/// is what hardware does and it is the whole reason the kernel emits a
/// `CanonicalizeBf16Nan` beside every arithmetic operation; a machine that
/// canonicalized here would satisfy the reference comparison for a reason that
/// has nothing to do with the kernel, and would make the conversion's
/// perturbation unobservable.
pub(super) fn bf16_binary(lhs: u16, rhs: u16, op: impl Fn(f64, f64) -> f64) -> u16 {
    if bf16_is_nan(lhs) {
        return lhs | 0x0040;
    }
    if bf16_is_nan(rhs) {
        return rhs | 0x0040;
    }
    let value = op(
        bf16_exact_value_or_infinity(lhs),
        bf16_exact_value_or_infinity(rhs),
    );
    if value.is_nan() {
        // An invalid operation on non-NaN operands — infinity times zero, or
        // opposite infinities added. Hardware's default quiet NaN, which happens
        // to be the canonical payload, so this case is deliberately *not* the
        // one the canonicalization perturbation is watched on.
        return 0x7fc0;
    }
    bf16_round(value)
}

/// The exact value of one non-NaN `bf16` encoding, infinities included.
pub(super) fn bf16_exact_value_or_infinity(bits: u16) -> f64 {
    if bits & 0x7fff == 0x7f80 {
        if bits & 0x8000 == 0 {
            f64::INFINITY
        } else {
            f64::NEG_INFINITY
        }
    } else {
        bf16_exact_value(bits)
    }
}

/// Splits one kernel body's top-level operations at each barrier.
///
/// The barrier itself is dropped rather than retained: it is not an operation a
/// lane executes, it is the boundary the lanes are advanced across together.
pub(super) fn barrier_segments(
    body: tiler_ir::kernel::BlockRef<'_>,
) -> Vec<Vec<tiler_ir::kernel::OperationRef<'_>>> {
    let mut segments = vec![Vec::new()];
    for operation in body.operations() {
        if matches!(operation.view(), OperationView::Barrier { .. }) {
            segments.push(Vec::new());
        } else {
            segments
                .last_mut()
                .expect("the segment list is never empty")
                .push(operation);
        }
    }
    segments
}

/// Returns the bounded range of the kernel's single guarded reduction loop.
pub(crate) fn reduction_loop(kernel: &VerifiedKernel) -> Option<(u64, u64)> {
    kernel
        .body()
        .operations()
        .filter_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .flat_map(tiler_ir::kernel::BlockRef::operations)
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some((reduction.start(), reduction.end())),
            _ => None,
        })
}

/// Returns the one retained alternative of the requested plan shape.
pub(super) fn alternative(
    product: &CompilationProduct,
    kind: ProgramAlternativeKind,
) -> &ProgramAlternative {
    product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .filter(|alternative| alternative.kind == kind)
        .min_by(|left, right| left.identity.cmp(&right.identity))
        .unwrap_or_else(|| panic!("a retained {} alternative", kind.name()))
}

/// Returns the one retained alternative dispatching exactly `stages` kernels.
///
/// Selected by stage count rather than by [`ProgramAlternativeKind`], because
/// every plan of an epilogue chain is `Materialized` — the kind is `Fused`
/// exactly when one region covers the program, and a chain has at least two —
/// so the kind cannot name which of them a bit comparison is about. The count
/// can: it is the number of dispatches the assembled program declares, and the
/// chain under test is the one whose dispatches are its recognized regions.
pub(super) fn alternative_dispatching(
    product: &CompilationProduct,
    stages: usize,
) -> &ProgramAlternative {
    let retained: Vec<&ProgramAlternative> = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .filter(|alternative| alternative.program.stage_count() == stages)
        .collect();
    let [alternative] = retained.as_slice() else {
        panic!(
            "expected exactly one retained {stages}-stage alternative, found {}",
            retained.len()
        );
    };
    alternative
}

/// Returns the kind of the alternative the portfolio selected.
pub(super) fn selected_kind(product: &CompilationProduct) -> ProgramAlternativeKind {
    let target = &product.targets[0];
    target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        })
        .expect("the selected identity names a retained alternative")
        .kind
}

/// Counts every retained explain record by its stable rule key.
pub(super) fn rule_counts(trace: &VerifiedExplainTrace) -> BTreeMap<&str, usize> {
    trace
        .records()
        .iter()
        .fold(BTreeMap::new(), |mut counts, record| {
            *counts.entry(record.rule().key().as_str()).or_insert(0) += 1;
            counts
        })
}

/// Every retained alternative's stage count for one program, ascending.
pub(super) fn retained_stage_counts(
    semantic: &SemanticProgram,
    contract: crate::request::StrictF32NumericalContract,
) -> Vec<usize> {
    let request = CompilationRequest::governed_preferring(
        semantic,
        crate::request::NumericalContractPreference::ordered(vec![contract]).unwrap(),
    );
    let product = compile(request).expect("the chain compiles");
    let mut counts: Vec<usize> = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .map(|alternative| alternative.program.stage_count())
        .collect();
    counts.sort_unstable();
    counts
}

/// One region subject's frontier attribution, read out of the compile path's
/// own trace.
pub(super) struct RegionAttribution {
    pub(super) role: String,
    pub(super) admitted: u64,
    /// Whether the provider declined the serial baseline for this region and
    /// named the region-vocabulary wall it hit.
    pub(super) declined_baseline: Option<String>,
    /// The record closing this region's frontier enumeration, which is what
    /// every later attribution for this
    /// region cites: its causal chain runs back through each decline the
    /// enumeration recorded to the admitted count that opened it, so following
    /// one cause from a coverage gap reaches the wall that caused it.
    pub(super) enumeration_tail: ExplainRecordId,
}

/// Reads one attribution per region subject out of a compiled trace.
///
/// Keyed by the region's explain subject, which is what the whole explain half
/// of this work is about: a role-keyed reading would fold fourteen of the
/// governed program's subjects into one entry and could not tell an answered
/// region from an unanswered one.
pub(super) fn region_attributions(
    trace: &VerifiedExplainTrace,
) -> BTreeMap<String, RegionAttribution> {
    let mut attributions: BTreeMap<String, RegionAttribution> = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            continue;
        };
        if assessment.predicate().as_str() != "frontier.locally-feasible" {
            continue;
        }
        let fact = |key: &str| {
            assessment
                .facts()
                .iter()
                .find(|fact| fact.key().as_str() == key)
        };
        let Some(FactValue::Count(admitted)) = fact("admitted-count").map(ExplainFact::value)
        else {
            panic!("a frontier record states its admitted count");
        };
        let Some(FactValue::Identity(role)) = fact("region-role").map(ExplainFact::value) else {
            panic!("a frontier record states its region role");
        };
        let previous = attributions.insert(
            record.subjects()[0].key().as_str().to_owned(),
            RegionAttribution {
                role: role.as_str().to_owned(),
                admitted: *admitted,
                declined_baseline: None,
                enumeration_tail: record.id(),
            },
        );
        assert!(
            previous.is_none(),
            "one region subject enumerated its frontier twice",
        );
    }
    for record in trace.records() {
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            continue;
        };
        if assessment.predicate().as_str() != "frontier.rejections-recorded" {
            continue;
        }
        attributions
            .get_mut(record.subjects()[0].key().as_str())
            .expect("a rejection count closes a subject whose frontier was enumerated")
            .enumeration_tail = record.id();
    }
    for record in trace.records() {
        if record.rule().key().as_str() != "frontier.strategy-decline.v1" {
            continue;
        }
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            continue;
        };
        let baseline = assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "strategy"
                && matches!(
                    fact.value(),
                    FactValue::Identity(key)
                        if key.as_str() == crate::physical::SERIAL_BASELINE_STRATEGY
                )
        });
        if !baseline {
            continue;
        }
        let key = record.subjects()[0].key().as_str();
        let reason = assessment
            .reason()
            .expect("a decline names the wall it hit")
            .as_str()
            .to_owned();
        attributions
            .get_mut(key)
            .expect("a decline is recorded on a subject whose frontier was enumerated")
            .declined_baseline = Some(reason);
    }
    attributions
}

/// The formation a verified target request runs under.
pub(super) fn plan_formation(
    semantic: &SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
) -> crate::region::RegionFormationOutcome {
    crate::region::form_region_candidates(semantic, request.budgets(), request.numerical_contract())
        .expect("the fixture forms regions")
}

/// Re-derives the selected portfolio for a verified target request.
pub(super) fn plan_portfolio(
    semantic: &SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
) -> crate::selection::SelectedPortfolio {
    let mut explain = ExplainWriter::new(request).unwrap();
    let formation = crate::region::form_region_candidates(
        semantic,
        request.budgets(),
        request.numerical_contract(),
    )
    .unwrap();
    let root = test_root(&mut explain);
    enumerate_complete_plans(
        semantic,
        request,
        &formation,
        &PhysicalAuthorities::governed(),
        &mut explain,
        root,
        None,
    )
    .map_or_else(
        |_| panic!("the governed request enumerates complete plans"),
        |plans| plans.portfolio,
    )
}

/// The frontier record the compile path emitted for the region subject carrying
/// `role`, if any.
///
/// The record is keyed by the region's canonical occurrence label rather than by
/// its role, so a test asking for "the pointwise region" asks the trace which
/// subject reported that role instead of reconstructing a digest. It panics when
/// two subjects report the role: the roles these tests name identify exactly one
/// region of the governed program, and a silent first match would let a
/// collapsed role pass as a resolved one.
pub(super) fn frontier_record<'trace>(
    trace: &'trace VerifiedExplainTrace,
    role: &str,
) -> Option<&'trace crate::explain::ExplainRecord> {
    let matching: Vec<_> = trace
        .records()
        .iter()
        .filter(|record| {
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                return false;
            };
            assessment.predicate().as_str() == "frontier.locally-feasible"
                && assessment.facts().iter().any(|fact| {
                    fact.key().as_str() == "region-role"
                        && matches!(fact.value(), FactValue::Identity(key) if key.as_str() == role)
                })
        })
        .collect();
    assert!(
        matching.len() <= 1,
        "{} region subjects reported the role {role}, so it names no single region",
        matching.len(),
    );
    matching.first().copied()
}

/// The explain subject key of the region subject carrying `role`.
pub(super) fn region_subject_key(trace: &VerifiedExplainTrace, role: &str) -> Option<String> {
    Some(
        frontier_record(trace, role)?.subjects()[0]
            .key()
            .as_str()
            .to_owned(),
    )
}

/// The implementations the frontier admitted for one region role, as the compile
/// path's own explain trace reports them.
pub(super) fn admitted_count(trace: &VerifiedExplainTrace, role: &str) -> Option<u64> {
    let ExplainEvent::Check { assessment, .. } = frontier_record(trace, role)?.event() else {
        return None;
    };
    assessment
        .facts()
        .iter()
        .find_map(|fact| match (fact.key().as_str(), fact.value()) {
            ("admitted-count", FactValue::Count(count)) => Some(*count),
            _ => None,
        })
}

/// Builds the recognized serial-sum program and its reassociation-permitting
/// verified request.
pub(super) fn split_request(
    shape: Shape,
) -> (SemanticProgram, crate::request::VerifiedTargetRequest) {
    let semantic = semantic_case_with_axis(
        shape,
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let verified = verify_planned_request(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .expect("the relaxed contract is admitted");
    let verified = verified
        .for_target(verified.target_profiles()[0])
        .expect("the governed target resolves the relaxed contract");
    (semantic, verified)
}

/// Enumerates the reduction subject's frontier for one verified request.
pub(super) fn reduction_frontier(
    request: &crate::request::VerifiedTargetRequest,
) -> crate::frontier::ImplementationFrontier {
    let subject = FrontierRegionSubject::new(
        "reduction",
        request.serial_sum().members.reduction().to_vec(),
        crate::physical::RegionWrite::ProgramOutput,
    );
    let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
    enumerate_frontier(
        request,
        &subject,
        &providers,
        &crate::call_registry::OpaqueCallRegistry::new(),
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the governed provider emits well-formed proposals")
}

/// Builds the reassociating request for one shape against a chosen profile.
pub(super) fn tree_request(
    shape: Shape,
    profile: TargetProfile,
) -> (SemanticProgram, crate::request::VerifiedTargetRequest) {
    let semantic = semantic_case_with_axis(
        shape,
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let mut request = CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    );
    request.target_profiles = vec![profile];
    let verified = verify_planned_request(request).expect("the relaxed contract is admitted");
    let verified = verified
        .for_target(verified.target_profiles()[0])
        .expect("the target resolves the relaxed contract");
    (semantic, verified)
}

/// A profile that realizes the tree's exact handoff and stages enough for it.
pub(super) fn tree_target() -> TargetProfile {
    TargetProfile::workgroup_tree_target_for_test(
        256,
        1_024,
        Some(crate::target::SynchronizationSupport::Realized),
    )
}

/// Assembles the three verified regions of one request's split program.
/// The materialized two-stage assembly a two-region cover states: the prologue
/// writes one value across the boundary, and the region after it publishes the
/// program's named output.
///
/// Stated rather than derived from a plan for the reason
/// `program::tests::materialized_assembly` gives: these tests drive the
/// assembler with a description, and `CoverAssembly::from_plan` is exercised by
/// every compiled program in this module and in `conformance`.
pub(super) fn materialized_assembly(
    request: &crate::request::VerifiedTargetRequest,
    scheduled: &[crate::physical::VerifiedScheduledRegion],
) -> crate::program::CoverAssembly {
    let subject = request.serial_sum();
    crate::program::CoverAssembly::stated(
        scheduled.to_vec(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            crate::program::AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![
                    crate::program::AssemblyBinding::Input(0),
                    crate::program::AssemblyBinding::Internal(0),
                ],
            },
            crate::program::AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![
                    crate::program::AssemblyBinding::Internal(0),
                    crate::program::AssemblyBinding::Internal(1),
                ],
            },
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![(subject.output_key.clone(), 1)],
    )
    .expect("the two-region assembly is well formed")
}

pub(crate) fn f32_tensor(shape: Shape, values: &[f32]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap()
}

pub(crate) fn tensor_bits(tensor: &Tensor) -> Vec<u32> {
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
            .collect(),
        _ => panic!("expected dense f32 reference output"),
    }
}

pub(crate) fn bits_of(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}
