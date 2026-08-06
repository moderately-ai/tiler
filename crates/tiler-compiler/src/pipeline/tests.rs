//! The pipeline's unit and integration tests.
//!
//! Split out of `pipeline.rs` so the module root reads as the compilation
//! story rather than as orchestration followed by sixteen hundred lines of
//! fixtures. `conformance` is deliberately a separate sibling: it drives the
//! public `compile()` entry point only, and mixing it in here would blur the
//! line between a test that may reach a stage-local constructor and one that
//! may not.

use super::*;

/// A retained root record the stage chain hangs from.
fn test_root(explain: &mut ExplainWriter) -> ExplainRecordId {
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
use crate::explain::ExplainDisposition;
use crate::frontier::{PhysicalImplementationProvider, PhysicalProposalKind};
use crate::physical::{InputOrdinal, RegionId, TensorRole};
use crate::request::{
    CompilerCapabilitySnapshot, NumericalContractPreference, StrictF32NumericalContract,
    TargetProfile,
};
use std::collections::BTreeMap;
use tiler_ir::kernel::{BinaryOp, CompareOp, ConvertOp, KernelConstant, OperationView, UnaryOp};
use tiler_ir::program::abi::{AvailabilityPhase, TargetPropertyRequirementRelation};
use tiler_ir::program::{DependencyReasonView, ValueRole};
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionIndex,
    ContractionIndexStructure, F32, F32Add, F32Constant, F32Multiply, F32TensorContraction,
    InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

fn semantic(reverse_constants: bool) -> SemanticProgram {
    semantic_case(
        Shape::from_dims([2, 2]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        reverse_constants,
    )
}

fn request_with_targets(
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

fn outcome_for_key<'a>(product: &'a CompilationProduct, key: &str) -> &'a TargetCompilationOutcome {
    product
        .targets
        .iter()
        .find(|outcome| outcome.target_profile().profile_key().as_str() == key)
        .unwrap_or_else(|| panic!("missing target outcome for {key}"))
}

fn semantic_case(
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

fn semantic_case_with_axis(
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

fn algebraic_add_chain() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let leaves = [1.0e20_f32, -1.0e20, 1.0]
        .map(|value| F32Constant::apply(&mut builder, value.to_bits()).unwrap());
    let left = F32Add::apply(&mut builder, leaves[0], leaves[1]).unwrap();
    let root = F32Add::apply(&mut builder, left, leaves[2]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

fn tensor_add_chain() -> SemanticProgram {
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

/// Builds the serial-sum program with one constant shared by both operands.
///
/// This is the canonical spelling that `NormalizeSemantics` produces from a
/// program that authored the same constant twice.
fn shared_constant_semantic(shape: Shape, constant_bits: u32) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape)
        .unwrap();
    let constant = F32Constant::apply(&mut builder, constant_bits).unwrap();
    let product = F32Multiply::apply(&mut builder, input, constant).unwrap();
    let mapped = F32Add::apply(&mut builder, product, constant).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// One typed value produced while interpreting a structured kernel.
#[derive(Clone, Copy, Debug)]
enum KirValue {
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
enum KirElements<'a> {
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
enum KirOutputs {
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
enum Bf16Canonicalization {
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
struct KirMachine<'a> {
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

pub(super) fn interpret_fused(kernel: &VerifiedKernel, input: &[f32]) -> Vec<f32> {
    interpret_fused_inputs(kernel, &[input])
}

/// Interprets one `f32` kernel over one payload per declared read buffer.
///
/// `inputs` is in the kernel's own buffer declaration order, which the region
/// builder fixes as ascending input ordinal followed by the owning write — so a
/// caller binds its program's declared inputs in the order it declared them.
pub(super) fn interpret_fused_inputs(kernel: &VerifiedKernel, inputs: &[&[f32]]) -> Vec<f32> {
    let payloads: Vec<KirElements<'_>> = inputs.iter().copied().map(KirElements::F32).collect();
    match KirMachine::run(kernel, &payloads, Bf16Canonicalization::Applied) {
        KirOutputs::F32(values) => values,
        KirOutputs::Bf16(_) => panic!("an f32 fixture produced a bf16 boundary"),
    }
}

/// Interprets one BF16 kernel over a BF16 boundary payload.
fn interpret_bf16(
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
const fn bf16_is_nan(bits: u16) -> bool {
    bits & 0x7f80 == 0x7f80 && bits & 0x007f != 0
}

/// The exact value of one `bf16` encoding, as an `f64`.
///
/// Exact, not approximate: `bf16` has eight significand bits and an exponent
/// range inside `f64`'s, so every finite encoding — including every subnormal —
/// is an `f64` value. Infinities and NaNs never reach here; [`bf16_binary`]
/// decides those from the encoding.
fn bf16_exact_value(bits: u16) -> f64 {
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
fn bf16_round(value: f64) -> u16 {
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
fn round_half_to_even(mantissa: u64, shift: u32) -> u64 {
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
fn bf16_binary(lhs: u16, rhs: u16, op: impl Fn(f64, f64) -> f64) -> u16 {
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
fn bf16_exact_value_or_infinity(bits: u16) -> f64 {
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
fn barrier_segments(
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
pub(super) fn reduction_loop(kernel: &VerifiedKernel) -> Option<(u64, u64)> {
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
fn alternative(product: &CompilationProduct, kind: ProgramAlternativeKind) -> &ProgramAlternative {
    product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .filter(|alternative| alternative.kind == kind)
        .min_by(|left, right| left.identity.cmp(&right.identity))
        .unwrap_or_else(|| panic!("a retained {} alternative", kind.name()))
}

/// Returns the kind of the alternative the portfolio selected.
fn selected_kind(product: &CompilationProduct) -> ProgramAlternativeKind {
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
fn rule_counts(trace: &VerifiedExplainTrace) -> BTreeMap<&str, usize> {
    trace
        .records()
        .iter()
        .fold(BTreeMap::new(), |mut counts, record| {
            *counts.entry(record.rule().key().as_str()).or_insert(0) += 1;
            counts
        })
}

/// A reindex reaches a kernel whose result is the reference evaluator's, bit for bit.
///
/// **The ticket's closing condition for the structural vocabulary.** The program
/// is `out = reverse(a)` on a `[2, 3]` operand, reversing axis 1 — the one
/// admitted within-axis coordinate permutation, and the only reindex form whose
/// decode needs the mirror. It exercises the whole vertical the widening
/// opened: the request boundary derives the coordinate map, the schedule
/// verifier discharges its bijectivity, the region's identity encodes it under
/// an appended tag, and the kernel lowering emits `extent - 1 - c` as real
/// offset arithmetic.
///
/// Bit-compared rather than approximately compared, which a reindex makes an
/// exact claim: the family computes nothing, so every output element must be an
/// input element unchanged. A tolerance here would hide the only way this can be
/// wrong — reading the *wrong* element.
#[test]
fn a_reindex_reaches_a_kernel_matching_the_reference_evaluator() {
    // Four elements, which is the governed baseline profile's declared grid
    // axis: a wider domain would decline for a launch reason and stop being
    // evidence about the access relation.
    let shape = Shape::from_dims([2, 2]);
    // Distinct, exactly representable, and deliberately not symmetric: a
    // palindromic row would make a reversal indistinguishable from an identity
    // read, which is exactly the defect this test exists to catch.
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape.clone())
        .unwrap();
    let reversed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(1))
            .expect("an axis reversal is an admitted form"),
        input,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), reversed)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a reindex of a declared input compiles");
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let actual = interpret_fused(&fused.kernels[0], &values);

    let key = InputKey::new("input").unwrap();
    let tensor = Tensor::dense(
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
    .unwrap();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    let expected_bits = match expected[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
            .collect::<Vec<_>>(),
        _ => panic!("expected dense f32 reference output"),
    };
    assert_eq!(
        actual
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        expected_bits,
    );
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught here. Row-major `[2, 2]`
    // reversed on axis 1 is each row read backwards.
    assert_eq!(actual, vec![2.0, 1.0, 8.0, 4.0]);
}

/// Compiles one two-input `f32` program and returns its kernel's result beside
/// the reference evaluator's, both as bits.
///
/// The two payloads are bound to the reference by *key* and to the kernel by
/// buffer declaration order, which are independent routes to the same
/// correspondence: a compiler that ordered its region's reads against the
/// program's declared inputs would disagree here rather than agree by
/// construction.
fn compiled_and_reference_bits(
    semantic: &SemanticProgram,
    bindings: &[(&str, Shape, &[f32]); 2],
) -> (Vec<u32>, Vec<u32>) {
    let product =
        compile(CompilationRequest::governed(semantic)).expect("the structural program compiles");
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let actual = interpret_fused_inputs(&fused.kernels[0], &[bindings[0].2, bindings[1].2]);

    let keys: Vec<InputKey> = bindings
        .iter()
        .map(|(key, ..)| InputKey::new(key).unwrap())
        .collect();
    let tensors: Vec<Tensor> = bindings
        .iter()
        .map(|(_, shape, values)| f32_tensor(shape.clone(), values))
        .collect();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            semantic,
            &[
                InputBinding::new(&keys[0], &tensors[0]),
                InputBinding::new(&keys[1], &tensors[1]),
            ],
        )
        .unwrap();
    (bits_of(&actual), tensor_bits(&expected[0]))
}

/// A widening broadcast reaches a kernel whose result is the reference
/// evaluator's, bit for bit.
///
/// **This is the ticket's user-visible outcome at its smallest honest size.**
/// The program is `out = a * broadcast(w)` with `a` at `[2, 2]` and `w` declared
/// at `[2]` and read at every row — the `[1024]`-against-`[T, 1024]` shape of the
/// normalization weight multiply, which is 113 of the pinned workload's 197
/// broadcast occurrences. Only the extents are smaller: the governed baseline
/// profile declares a four-thread grid axis, so a wider domain would decline for
/// a launch reason and stop being evidence about the access relation.
///
/// **The two reads are at different element counts, and that is the point.** The
/// widened read addresses its *operand's* range — two elements against the
/// region's four — so a region binding it against the domain would address past
/// the weight's end, and a machine holding one payload could not model the
/// program at all.
///
/// Bit-compared rather than approximately compared, for the reason the reindex
/// test states: a broadcast computes nothing, so every weight the multiply reads
/// must be an input element unchanged, and a tolerance would hide the only way
/// this can be wrong — replicating along the wrong axis.
#[test]
fn a_broadcast_reaches_a_kernel_matching_the_reference_evaluator() {
    let domain = Shape::from_dims([2, 2]);
    let weight_shape = Shape::from_dims([2]);
    // Distinct and exactly representable on both sides. The weight's two entries
    // must differ: a uniform weight makes replication along axis 0
    // indistinguishable from replication along axis 1, which is exactly the
    // defect this test exists to catch.
    let activations: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let weights: Vec<f32> = vec![3.0, 5.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let w = builder
        .input::<F32>(InputKey::new("w").unwrap(), weight_shape.clone())
        .unwrap();
    let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
        [
            tiler_ir::shape::Extent::new(2),
            tiler_ir::shape::Extent::new(2),
        ],
        [
            tiler_ir::semantic::BroadcastAxisSource::Replicate,
            tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("one replicated axis over a rank-one operand is an admitted relation");
    let widened = tiler_ir::semantic::F32Broadcast::apply(&mut builder, &mapping, w)
        .expect("the standard registry admits the broadcast family");
    let scaled = F32Multiply::apply(&mut builder, a, widened).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let (actual, expected) = compiled_and_reference_bits(
        &semantic,
        &[("a", domain, &activations), ("w", weight_shape, &weights)],
    );
    assert_eq!(actual, expected);
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught. Row-major `[2, 2]`
    // against a `[2]` weight replicated over axis 0 is `out[i][j] = a[i][j] *
    // w[j]`; replicating over the *other* axis would give `3, 6, 20, 40`, which
    // is why this literal discriminates.
    assert_eq!(actual, bits_of(&[3.0, 10.0, 12.0, 40.0]));
}

/// A reindex feeding a pointwise multiply reaches a kernel matching the
/// reference evaluator, bit for bit.
///
/// **This is Milestone 2's "reindex plus pointwise fusion" at the smallest
/// domain the governed profile launches.** The program is
/// `out = permute(a) * b`, where the permutation is `rearrange('i j -> j i')` —
/// an einops `rearrange` written in the one form the `Reindex` family spells for
/// it, `permute-axes`. Both operands are declared inputs at `[2, 2]`, so one
/// region carries a structural read and a dense read side by side, which is what
/// "fused" means here: no intermediate is materialized between the rearrangement
/// and the arithmetic, and the transpose contributes an access map rather than a
/// copy kernel.
///
/// **The reindex half is deliberately not the reversal.** `reverse-axis` is the
/// one form whose decode mirrors, and its bit comparison is already
/// [`a_reindex_reaches_a_kernel_matching_the_reference_evaluator`]'s. A permute
/// exercises the divide-and-modulo decode instead, over a *second* read the
/// region addresses densely in the same body — the composition neither of those
/// two tests covers on its own.
#[test]
fn a_reindexed_operand_feeding_a_multiply_matches_the_reference_evaluator() {
    let domain = Shape::from_dims([2, 2]);
    // Powers of two, so every product below is exact and any disagreement is a
    // wrong *element* rather than a rounding. Deliberately not symmetric under
    // transposition: `a` transposed is `1, 4, 2, 8`, so a compiler that dropped
    // the permutation entirely would produce a different tensor.
    let left: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let right: Vec<f32> = vec![3.0, 5.0, 7.0, 11.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), domain.clone())
        .unwrap();
    let transposed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("an axis permutation is an admitted form"),
        a,
    )
    .expect("the standard registry admits the reindex family");
    let scaled = F32Multiply::apply(&mut builder, transposed, b).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let (actual, expected) = compiled_and_reference_bits(
        &semantic,
        &[("a", domain.clone(), &left), ("b", domain, &right)],
    );
    assert_eq!(actual, expected);
    // Independently of the oracle: `a` transposed is `1, 4, 2, 8`, multiplied
    // elementwise by `3, 5, 7, 11`. Without the permutation the product would be
    // `3, 10, 28, 88`, which is what makes this literal discriminating.
    assert_eq!(actual, bits_of(&[3.0, 20.0, 14.0, 88.0]));
}

fn assert_fused_matches_reference(shape: Shape, values: Vec<f32>, scale_bits: u32, bias_bits: u32) {
    assert_fused_axis_matches_reference(shape, values, scale_bits, bias_bits, Axis::new(1));
}

fn assert_fused_axis_matches_reference(
    shape: Shape,
    values: Vec<f32>,
    scale_bits: u32,
    bias_bits: u32,
    reduction_axis: Axis,
) {
    let semantic =
        semantic_case_with_axis(shape.clone(), scale_bits, bias_bits, false, reduction_axis);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let actual = interpret_fused(&fused.kernels[0], &values);
    let key = InputKey::new("input").unwrap();
    let tensor = Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .into_iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(
        actual
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        match expected[0].payload() {
            TensorPayloadView::Dense(elements) => elements
                .iter()
                .map(|element| {
                    u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap())
                })
                .collect::<Vec<_>>(),
            _ => panic!("expected dense f32 reference output"),
        }
    );
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "keeps the exact explain snapshot beside the end-to-end product invariants"
)]
fn product_is_deterministic_and_preserves_the_materialized_boundary() {
    let first = semantic(false);
    let second = semantic(true);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );
    let occurrence_count = first.operations().count();
    crate::lowering::reset_refinement_proof_work();
    let first = compile(CompilationRequest::governed(&first)).unwrap();
    assert_eq!(
        crate::lowering::refinement_proof_work(),
        occurrence_count * 2,
        "each occurrence is refined once by planning and once by the independent portfolio verifier"
    );
    crate::lowering::reset_refinement_proof_work();
    let second = compile(CompilationRequest::governed(&second)).unwrap();
    assert_eq!(
        crate::lowering::refinement_proof_work(),
        occurrence_count * 2,
        "two retained alternatives must not multiply verifier proof work"
    );

    assert_eq!(first, second);
    for kind in [
        ProgramAlternativeKind::Materialized,
        ProgramAlternativeKind::Fused,
    ] {
        let forward = alternative(&first, kind);
        let reversed = alternative(&second, kind);
        let coverage = |alternative: &ProgramAlternative| {
            alternative
                .program
                .core()
                .stages()
                .map(|stage| stage.coverage().to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            coverage(forward),
            coverage(reversed),
            "{kind:?} stage coverage changed with authoring order"
        );
    }
    let target = &first.targets[0];
    let rendered = target.explain.render();
    assert!(rendered.starts_with("tiler-explain-v7 request="));
    assert!(rendered.contains("feasibility:threads-per-workgroup:deferred"));
    assert!(rendered.contains("feasibility:buffer-bindings:admitted"));
    assert!(rendered.contains("event=selection:tiler.selection.structural-pareto.v1:selected"));
    assert_eq!(target.portfolio.alternatives.len(), 2);
    assert_eq!(selected_kind(&first), ProgramAlternativeKind::Fused);
    let materialized = alternative(&first, ProgramAlternativeKind::Materialized);
    let fused = alternative(&first, ProgramAlternativeKind::Fused);
    assert_eq!(materialized.program.stage_count(), 2);
    let temporary = materialized
        .program
        .core()
        .values()
        .nth(1)
        .expect("the cross-stage temporary");
    assert_eq!(temporary.role(), ValueRole::Temporary);
    assert!(matches!(
        materialized
            .program
            .core()
            .dependencies()
            .next()
            .expect("one data dependency")
            .reason(),
        DependencyReasonView::Data(value) if value == temporary
    ));
    assert_eq!(
        materialized.kernels[0].buffers().nth(1).unwrap().tensor,
        TensorRole::Intermediate
    );
    assert_eq!(
        materialized.kernels[1].buffers().next().unwrap().tensor,
        TensorRole::Intermediate
    );
    assert_eq!(reduction_loop(&materialized.kernels[1]), Some((1, 2)));
    assert_eq!(fused.program.stage_count(), 1);
    assert_eq!(fused.program.core().values().len(), 2);
    // The exact aggregate structural cost is the sum of the per-region
    // estimates plus the cover's deliberate cross-region materializations.
    assert_eq!(materialized.structural_cost.dispatch_count(), 2);
    assert_eq!(materialized.structural_cost.launched_threads(), 6);
    assert_eq!(materialized.structural_cost.temporary_bytes(), 16);
    assert_eq!(materialized.structural_cost.materialization_count(), 1);
    assert_eq!(fused.structural_cost.dispatch_count(), 1);
    assert_eq!(fused.structural_cost.launched_threads(), 2);
    assert_eq!(fused.structural_cost.temporary_bytes(), 0);
    assert_eq!(fused.structural_cost.materialization_count(), 0);
    assert!(
        fused
            .structural_cost
            .dominates(&materialized.structural_cost)
    );
    // Lowering provenance is the set of providers the installed registry
    // resolved for the recognized occurrences. Both plan shapes cover the
    // same occurrences, so both name the same four governed providers: the
    // alternatives differ in their cover, not in who lowers each operation.
    // Provider and operation are named separately rather than one derived
    // from the other: they coincide by naming convention in the governed
    // registry, and a test that split the provider name would assert the
    // convention instead of the resolution.
    let expected_providers: Vec<_> = [
        ("governed-index-access.add-f32", "add-f32"),
        ("governed-index-access.constant-f32", "constant-f32"),
        ("governed-index-access.multiply-f32", "multiply-f32"),
        (
            "governed-index-access.strict-serial-sum-f32",
            "strict-serial-sum-f32",
        ),
    ]
    .into_iter()
    .map(|(provider, operation)| {
        crate::request::LoweringProviderIdentity::new(
            tiler_ir::semantic::ProviderIdentity::new("tiler", provider, 1).unwrap(),
            // The governed key names the capability family and the
            // operation it lowers, never the provider, which is recorded
            // beside it.
            format!("tiler.capability.index-access.tiler.{operation}.v1"),
            crate::capability::LoweringCapabilityRevision::new(1).unwrap(),
        )
    })
    .collect();
    assert_eq!(
        materialized.artifact_plan.lowering_providers(),
        expected_providers
    );
    assert_eq!(fused.artifact_plan.lowering_providers(), expected_providers);
    assert_eq!(reduction_loop(&fused.kernels[0]), Some((1, 2)));
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "compile.plan.boundary"
            && record.event().disposition() == ExplainDisposition::Admitted
    }));
    // The materialized plan discharges exactly one cross-region handoff; the
    // fused plan materializes nothing across a boundary.
    assert_eq!(materialized.plan.handoffs().len(), 1);
    assert!(fused.plan.handoffs().is_empty());
    // Stable identity binds the semantic origin and request contract as well as
    // the selected physical plan.
    for alternative in &target.portfolio.alternatives {
        assert_eq!(alternative.stable_id, alternative.identity.label());
    }
}

/// One region subject's frontier attribution, read out of the compile path's
/// own trace.
struct RegionAttribution {
    role: String,
    admitted: u64,
    /// Whether the provider declined the serial baseline for this region and
    /// named the region-vocabulary wall it hit.
    declined_baseline: Option<String>,
    /// The record closing this region's frontier enumeration, which is what
    /// every later attribution for this
    /// region cites: its causal chain runs back through each decline the
    /// enumeration recorded to the admitted count that opened it, so following
    /// one cause from a coverage gap reaches the wall that caused it.
    enumeration_tail: ExplainRecordId,
}

/// Reads one attribution per region subject out of a compiled trace.
///
/// Keyed by the region's explain subject, which is what the whole explain half
/// of this work is about: a role-keyed reading would fold fourteen of the
/// governed program's subjects into one entry and could not tell an answered
/// region from an unanswered one.
fn region_attributions(trace: &VerifiedExplainTrace) -> BTreeMap<String, RegionAttribution> {
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

/// **Obligation 4 of the minimum correct physical realization profile, end to
/// end: no region a legal cover placed is answered with silence.**
///
/// Every region subject the compile path enumerates either admits at least one
/// implementation or carries a typed decline naming which region-vocabulary
/// wall it hit. Before the provider read the cover region subject, fourteen of
/// this program's seventeen subjects reached the final `else` of a member-set
/// comparison and returned an empty offer, so complete-plan selection saw an
/// unimplemented region with nothing said about it.
///
/// The implication is what this asserts, and it is a check that can say no:
/// deleting the `Err(wall)` arm of `GovernedPhysicalProvider::propose` restores
/// the empty offer and fourteen subjects fail it at once.
#[test]
fn every_cover_region_receives_a_proposal_or_a_typed_decline() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let attributions = region_attributions(&product.targets[0].explain);

    assert_eq!(
        attributions.len(),
        17,
        "the governed five-operation program covers seventeen distinct region subjects",
    );
    for (key, attribution) in &attributions {
        assert!(
            attribution.admitted > 0 || attribution.declined_baseline.is_some(),
            "region {key} ({}) was answered with silence",
            attribution.role,
        );
    }
    // The three the vocabulary spells are answered with implementations, and
    // every other one with a wall. Asserting both halves is what stops a
    // regression that declined *everything* from passing the implication above.
    let mut answered: Vec<&str> = attributions
        .values()
        .filter(|attribution| attribution.admitted > 0)
        .map(|attribution| attribution.role.as_str())
        .collect();
    answered.sort_unstable();
    assert_eq!(answered, ["pointwise", "reduction", "whole-program"]);
    let walls: BTreeMap<&str, usize> = attributions
        .values()
        .filter_map(|attribution| attribution.declined_baseline.as_deref())
        .fold(BTreeMap::new(), |mut counts, reason| {
            *counts.entry(reason).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        walls,
        BTreeMap::from([
            // Five regions covering the reduction together with part, but not
            // all, of its four-occurrence prologue.
            ("region-partial-fused-program", 5),
            // Nine regions covering a proper part of that prologue.
            ("region-partial-coverage", 9),
        ]),
        "the fourteen declines no longer name the walls they hit",
    );
}

/// **Fourteen region subjects share one role and are fourteen explain
/// subjects.**
///
/// The role vocabulary has four values and the cover space has seventeen
/// regions, so a role-keyed trace could not name thirteen of them at all —
/// `record_frontier` was called on the first sighting of each role, and the
/// rest emitted nothing. Keying on the region's canonical occurrence makes the
/// deduplication correct rather than lossy.
///
/// The check that can say no is the key itself: reverting the subject key to
/// `region:{role}` collapses these fourteen to one and the count fails.
#[test]
fn region_subjects_sharing_a_role_are_distinct_explain_subjects() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let attributions = region_attributions(&product.targets[0].explain);

    let unrecognized: Vec<&String> = attributions
        .iter()
        .filter(|(_, attribution)| attribution.role == "unrecognized")
        .map(|(key, _)| key)
        .collect();
    assert_eq!(unrecognized.len(), 14);
    // `region_attributions` is a map keyed by the subject, so distinctness is
    // structural — but stating it is what makes the count above a claim about
    // fourteen regions rather than about fourteen records.
    let distinct: std::collections::BTreeSet<&&String> = unrecognized.iter().collect();
    assert_eq!(distinct.len(), unrecognized.len());
    // Each one covers a different occurrence set, which is why one record for
    // all of them was lossy: the declines they carry are not interchangeable.
    assert!(
        attributions
            .values()
            .filter(|attribution| attribution.role == "unrecognized")
            .any(|attribution| attribution.declined_baseline.as_deref()
                == Some("region-partial-fused-program"))
    );
}

/// **The per-cover coverage gap reaches a production reader.**
///
/// `PlanRejection::RegionUnimplemented` has always been constructed — the
/// governed program records thirty-eight per compile — and
/// `SelectedPortfolio::rejections()` had no caller outside `selection.rs`'s own
/// test module, so the one authority that states the gap *per cover* was
/// compiled away. This drives the reader that now emits it, and checks each
/// record is caused by the frontier enumeration for its own region rather than
/// by whatever record happened to be last.
///
/// The check that can say no is the emission: removing the `record_coverage_gaps`
/// call leaves the rejections constructed and the trace empty of them, and the
/// count below fails.
#[test]
fn the_per_cover_coverage_gap_reaches_the_trace() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;
    let attributions = region_attributions(trace);

    let gaps: Vec<&crate::explain::ExplainRecord> = trace
        .records()
        .iter()
        .filter(|record| record.rule().key().as_str() == "selection.region-coverage.v1")
        .collect();
    assert_eq!(
        gaps.len(),
        38,
        "the governed program's covers report thirty-eight region coverage gaps",
    );
    for gap in &gaps {
        assert!(matches!(
            gap.event(),
            ExplainEvent::Check { assessment, .. }
                if assessment.predicate().as_str() == "selection.region-implemented"
                    && assessment
                        .reason()
                        .is_some_and(|reason| reason.as_str() == "region-unimplemented")
        ));
        // Two subjects: the region that had no implementation and the cover
        // that placed it. A gap naming only one of them cannot be acted on —
        // the same region is implementable in no cover, and the same cover
        // fails for exactly one region.
        let subjects: Vec<&str> = gap
            .subjects()
            .iter()
            .map(|subject| subject.key().as_str())
            .collect();
        assert_eq!(subjects.len(), 2);
        assert!(subjects[1].starts_with("region-cover:"));
        let attribution = attributions
            .get(subjects[0])
            .expect("a coverage gap names a region whose frontier was enumerated");
        assert_eq!(attribution.admitted, 0);
        assert_eq!(
            gap.causes(),
            [attribution.enumeration_tail],
            "the coverage gap is not caused by its own region's frontier enumeration",
        );
    }
    // Every cover that reported a gap is one that contributed no plan, and the
    // regions named are exactly the ones nothing implemented.
    let named: std::collections::BTreeSet<&str> = gaps
        .iter()
        .map(|gap| gap.subjects()[0].key().as_str())
        .collect();
    assert!(
        named
            .iter()
            .all(|key| attributions[*key].declined_baseline.is_some()),
        "a region was reported unimplemented without a decline explaining why",
    );
}

/// Every draft authority the conformance gate wires must speak the explain
/// vocabulary; a silent authority cannot be audited.
#[test]
fn every_wired_authority_emits_its_typed_explain_records() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;
    // The exhaustive snapshot: every rule the wired compile path emits, and
    // exactly how many records each contributes. A new authority that stays
    // explain-silent, or one that becomes chatty, fails here.
    assert_eq!(
        rule_counts(trace),
        BTreeMap::from([
            ("compile.request.general-boundary", 1),
            ("region.formation.v1", 1),
            ("region.candidate.v1", 17),
            // One resolution and one refinement per recognized occurrence.
            ("capability.index-access-resolution.v1", 5),
            ("kernel.index-region-refinement.v1", 5),
            ("cover.enumeration.v1", 1),
            ("fusion.legality.v1", 12),
            ("fusion.strict-f32-equivalence", 1),
            // Two summary records per region subject: admitted count and
            // rejected count. Typed per-opaque-rejection detail records accompany
            // them when present; this governed compile fixture has no opaque
            // rejection, so its **seventeen** region subjects contribute
            // thirty-four.
            //
            // Seventeen rather than the four this census used to record, and
            // the difference is the explain half of the region-general
            // provider: the frontier record is keyed by each region's canonical
            // occurrence rather than by its role, so the fourteen subjects that
            // share the role `unrecognized` are fourteen records instead of one
            // record and thirteen silences.
            ("frontier.enumeration.v1", 34),
            // Sixteen: the two parallel reduction strategies, plus the serial
            // baseline withheld once for each of the fourteen region subjects
            // this schedule vocabulary cannot spell.
            //
            // The two are the multi-pass split and the single-workgroup tree,
            // both at the reduction subject and both for the same reason — this
            // fixture compiles under the strict contract, and each *is* a
            // reassociation of the declared contributor sequence. No other
            // subject reaches either strategy, so a seventeenth record would
            // mean one was being considered somewhere it does not apply.
            ("frontier.strategy-decline.v1", 16),
            ("selection.complete-plan.v1", 1),
            // One per (cover, unimplemented region) pair: the coverage gap that
            // was constructed and never emitted. Thirty-eight is the count
            // `select_physical_plans` already recorded internally, now reaching
            // a reader with the region named rather than the role.
            ("selection.region-coverage.v1", 38),
            ("compile.region.verified", 3),
            ("compile.plan.boundary", 2),
            ("schedule.plan-regions", 2),
            ("kernel.plan-refinement", 2),
            ("program.plan-verified", 2),
            ("artifact.plan-construction", 2),
            ("target.buffer-bindings", 3),
            ("target.device-memory", 3),
            ("target.grid-axis", 3),
            ("target.index-arithmetic-u64", 3),
            ("target.local-memory-bytes", 3),
            // The four per-dimension honourability records replace the one
            // `target.strict-f32` predicate, which is the whole point of
            // retiring it: three regions each now report which dimension was
            // assessed and by what means, where one boolean reported neither.
            ("target.numerics.contraction", 3),
            ("target.numerics.input-subnormals", 3),
            ("target.numerics.reassociation", 3),
            ("target.numerics.result-subnormals", 3),
            ("target.threads-per-workgroup", 3),
            // Two retained plans, two records each: exact terms share one
            // checked-invariant assessment, while the memory-traffic bound
            // shares one assumption assessment. Grouping by evidence class is
            // the mechanism that moved this census: one assessment has one
            // basis, so mixing the terms would make one class lie.
            ("tiler.cost.analytical.v1", 4),
            // One record per legal cover the partition search's own estimate
            // beat, naming the pruned cover and the cover that beat it. Sixteen
            // covers are enumerated for this program and fifteen of them are
            // dominated by the fused one, which crosses no boundary at all — so
            // the count is a fact about this program's cover space rather than
            // an arbitrary number. It is deliberately a *cost* record: those
            // fifteen covers are legal, and only a refused one is a rejection.
            ("tiler.cost.partition-structural.v1", 15),
            ("tiler.cost.structural.v1", 2),
            ("tiler.selection.structural-pareto.v1", 2),
        ])
    );
    assert!(
        trace.records().iter().all(|record| {
            record.rule().key().as_str() != "target.barriers"
                && !matches!(
                    record.event(),
                    ExplainEvent::Feasibility { predicate, .. }
                        if predicate.as_str() == "barriers"
                )
        }),
        "a zero-synchronization program emitted an invented barrier capability fact"
    );
    // The same absence one layer up. The retired barrier-count axis is gone, and
    // the *realization* record that replaced it must not appear either: a program
    // with no synchronization point derives no requirement, so the authority
    // consults no fact and there is no check to report. A record saying
    // "undeclared" would be the manufactured zero in a new spelling — it would
    // read as a target limitation rather than as a question never asked.
    assert!(
        trace.records().iter().all(|record| {
            !record
                .rule()
                .key()
                .as_str()
                .starts_with("target.synchronization")
                && !matches!(
                    record.event(),
                    ExplainEvent::SynchronizationRealization { .. }
                )
        }),
        "a zero-synchronization program emitted a synchronization-realization record"
    );
    assert!(
        trace.records().iter().all(|record| {
            record.rule().key().as_str() != "target.device-address-bits"
                && !matches!(
                    record.event(),
                    ExplainEvent::Feasibility { predicate, .. }
                        if predicate.as_str() == "device-address-bits"
                )
        }),
        "a program with no address-width requirement emitted an address-width fact"
    );
    let analytical = trace
        .records()
        .iter()
        .filter(|record| record.rule().key().as_str() == ANALYTICAL_MODEL_KEY)
        .collect::<Vec<_>>();
    assert_eq!(
        analytical
            .iter()
            .filter(|record| matches!(
                record.event(),
                ExplainEvent::CostAssessment {
                    basis: EvidenceBasis::CheckedInvariant,
                    terms,
                    disposition: CostDisposition::Reported,
                    ..
                } if terms.len() == 7
            ))
            .count(),
        2,
        "each plan reports six exact components and its exact unknown count"
    );
    assert_eq!(
        analytical
            .iter()
            .filter(|record| matches!(
                record.event(),
                ExplainEvent::CostAssessment {
                    basis: EvidenceBasis::Assumption,
                    terms,
                    disposition: CostDisposition::Reported,
                    ..
                } if terms.len() == 2
            ))
            .count(),
        2,
        "each plan reports both endpoints of its modelled memory bound"
    );
    assert!(
        analytical
            .iter()
            .all(|record| record.event().disposition() == ExplainDisposition::Reported)
    );
    let rendered = trace.render();
    for typed_term in [
        "cost.memory-traffic.bounded.low:bytes=",
        "cost.indexing.exact:operations=",
        "cost.dispatch.exact:count=",
        "cost.threadgroup-memory.exact:bytes=",
    ] {
        assert!(
            rendered.contains(typed_term),
            "missing typed analytical term {typed_term}"
        );
    }
    for (rule, fact_key, expected) in [
        ("normalize.semantics.v1", "rewrite-count", 0),
        ("region.formation.v1", "candidate-count", 17),
        ("region.formation.v1", "operation-count", 5),
        ("cover.enumeration.v1", "cover-count", 16),
        ("selection.complete-plan.v1", "plan-count", 2),
    ] {
        let records = if rule == "normalize.semantics.v1" {
            product.targets[0].selection_explain.records()
        } else {
            trace.records()
        };
        let record = records
            .iter()
            .find(|record| record.rule().key().as_str() == rule)
            .unwrap_or_else(|| panic!("missing typed count emitter {rule}"));
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            panic!("typed count emitter {rule} must be a checked assertion");
        };
        assert!(assessment.predicate().as_str().contains('.'));
        let actual = assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == fact_key)
            .map(|fact| fact.value().clone());
        assert_eq!(
            actual,
            Some(FactValue::Count(expected)),
            "{rule}/{fact_key}"
        );
    }
    // Every recognized occurrence resolved a lowering capability and carries
    // exhaustive finite refinement evidence attributed to the same provider.
    for (rule, stage, basis) in [
        (
            "capability.index-access-resolution.v1",
            ExplainStage::CapabilityResolution,
            EvidenceBasis::CheckedInvariant,
        ),
        (
            "kernel.index-region-refinement.v1",
            ExplainStage::KernelRefinement,
            EvidenceBasis::ExhaustiveFinite,
        ),
    ] {
        let records: Vec<_> = trace
            .records()
            .iter()
            .filter(|record| record.rule().key().as_str() == rule)
            .collect();
        assert_eq!(records.len(), 5, "{rule}");
        for record in records {
            assert_eq!(record.event().disposition(), ExplainDisposition::Admitted);
            assert_eq!(record.event().stage(), stage);
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                panic!("{rule} must be a checked assertion");
            };
            assert_eq!(assessment.basis(), &basis);
            // Attribution is the resolved lowering provider, never the
            // compiler: an out-of-crate provider owns this claim.
            assert_ne!(record.rule().provider(), &ProviderRef::builtin());
        }
    }
    // Fusion legality is attributed to the capability provider that declared
    // the member operations' roles, never to the compiler itself.
    let legality = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "fusion.legality.v1")
        .expect("a fusion-legality record");
    assert_eq!(legality.event().disposition(), ExplainDisposition::Admitted);
    assert!(trace.render().starts_with("tiler-explain-v7 request="));
}

/// Asserts the honourability half of the end-to-end explain conformance.
///
/// The numerical dimensions left the quantitative predicate space when
/// `strict-f32` was retired, so they are counted through their own typed
/// record. Each names the dimension, the behaviour the resolved contract
/// required, the means the profile declares, and the declaring profile — and
/// the admitted trace asserts the *means*, because a proven verdict alone
/// would not distinguish native support from emulation.
fn assert_honoured_dimensions_are_exhaustive(trace: &crate::explain::VerifiedExplainTrace) {
    let mut honoured = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::NumericalHonourability {
            dimension,
            arithmetic,
            required,
            outcome,
            profile,
            resolved_type,
        } = record.event()
        else {
            continue;
        };
        assert_eq!(
            outcome,
            &crate::explain::HonourabilityOutcome::Honoured {
                means: crate::explain::ReasonCode::new("supported-exactly").unwrap(),
            }
        );
        assert_eq!(
            profile.as_str(),
            "tiler.prototype-target-neutral-baseline.v1"
        );
        assert_eq!(*arithmetic, tiler_ir::schedule::ArithmeticType::F32);
        assert_eq!(resolved_type, &tiler_ir::semantic::F32::resolved_type());
        *honoured
            .entry((dimension.as_str(), required.as_str()))
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        honoured,
        BTreeMap::from([
            (("numerics.contraction", "forbidden"), 3),
            (("numerics.input-subnormals", "preserve"), 3),
            (("numerics.reassociation", "forbidden"), 3),
            (("numerics.result-subnormals", "preserve"), 3),
        ])
    );
    assert!(trace.render().contains(
            "honourability:numerics.input-subnormals:tiler::f32@1:preserve:honoured:supported-exactly:profile=tiler.prototype-target-neutral-baseline.v1"
        ));
}

#[test]
fn end_to_end_explain_emitter_has_exhaustive_typed_conformance() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;

    let mut target_predicates = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::Feasibility {
            predicate,
            outcome: crate::explain::FeasibilityOutcome::Admitted,
            required,
            available,
        } = record.event()
        else {
            continue;
        };
        let unit_is_exact = match predicate.as_str() {
            "grid-axis" => {
                matches!(
                    (required, available),
                    (Quantity::Threads(_), Quantity::Threads(_))
                )
            }
            "buffer-bindings" => matches!(
                (required, available),
                (Quantity::Bindings(_), Quantity::Bindings(_))
            ),
            "local-memory-bytes" => {
                matches!(
                    (required, available),
                    (Quantity::Bytes(_), Quantity::Bytes(_))
                )
            }
            "index-arithmetic-u64" | "device-memory" => {
                matches!(
                    (required, available),
                    (Quantity::Count(_), Quantity::Count(_))
                )
            }
            "device-address-bits" => {
                matches!(
                    (required, available),
                    (Quantity::Bits(_), Quantity::Bits(_))
                )
            }
            other => panic!("unexpected target predicate {other}"),
        };
        assert!(unit_is_exact);
        *target_predicates
            .entry(predicate.as_str())
            .or_insert(0_usize) += 1;
    }
    assert_eq!(
        target_predicates,
        BTreeMap::from([
            ("buffer-bindings", 3),
            ("device-memory", 3),
            ("grid-axis", 3),
            ("index-arithmetic-u64", 3),
            ("local-memory-bytes", 3),
        ])
    );

    let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let mut deferred_subjects = BTreeMap::new();
    for record in trace.records() {
        let ExplainEvent::DeferredTargetRequirement {
            entry,
            predicate,
            required,
            requirement,
        } = record.event()
        else {
            continue;
        };
        assert_eq!(predicate.as_str(), "threads-per-workgroup");
        assert_eq!(*required, Quantity::Threads(1));
        assert_eq!(requirement.required(), 1);
        assert_eq!(
            requirement.relation(),
            TargetPropertyRequirementRelation::ObservedAtLeastRequired
        );
        let query = requirement.query();
        assert_eq!(
            query.key().as_str(),
            "tiler.target.prepared-entry.max-threads-per-workgroup.v1"
        );
        assert_eq!(
            query.available_at(),
            AvailabilityPhase::PreparedKernelPreflight
        );
        assert_eq!(query.provider().namespace(), "tiler");
        assert_eq!(query.provider().name(), "prepared-entry-properties");
        assert_eq!(query.provider().revision(), 1);
        assert_eq!(record.subjects().len(), 1);
        assert_eq!(
            deferred_subjects.insert(
                (record.subjects()[0].key().as_str().to_owned(), *entry,),
                1_usize,
            ),
            None,
            "each exact alternative/region/entry subject is reported once"
        );
    }
    let expected_deferred_subjects = [materialized, fused]
        .into_iter()
        .flat_map(|alternative| {
            alternative
                .scheduled_regions
                .iter()
                .enumerate()
                .map(move |(entry, scheduled)| {
                    (
                        (
                            format!(
                                "{}/region:{}",
                                alternative.stable_id,
                                scheduled.region().index.id.get()
                            ),
                            u32::try_from(entry).unwrap(),
                        ),
                        1_usize,
                    )
                })
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(deferred_subjects, expected_deferred_subjects);
    assert_eq!(deferred_subjects.len(), 3);

    assert_honoured_dimensions_are_exhaustive(trace);

    let selections = trace
        .records()
        .iter()
        .filter_map(|record| match record.event() {
            ExplainEvent::Selection { outcome, .. } => {
                Some((record.subjects()[0].key().as_str().to_owned(), *outcome))
            }
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        selections.get(&materialized.stable_id),
        Some(&SelectionOutcome::Dominated)
    );
    assert_eq!(
        selections.get(&fused.stable_id),
        Some(&SelectionOutcome::Selected)
    );
}

#[test]
fn normalization_converges_duplicated_and_shared_constants_on_one_portfolio() {
    let shape = Shape::from_dims([2, 2]);
    let bits = 2.0_f32.to_bits();
    let duplicated = semantic_case(shape.clone(), bits, bits, false);
    let shared = shared_constant_semantic(shape, bits);
    assert_eq!(duplicated.operation_count(), 5);
    assert_eq!(shared.operation_count(), 4);
    assert_ne!(
        duplicated.semantic_identity().graph(),
        shared.semantic_identity().graph()
    );

    let from_duplicated = compile(CompilationRequest::governed(&duplicated)).unwrap();
    let from_shared = compile(CompilationRequest::governed(&shared)).unwrap();
    let rendered = from_duplicated.targets[0].compilation_explain.render();
    let request_headers = rendered
        .lines()
        .filter(|line| line.starts_with("tiler-explain-v7 request="))
        .collect::<Vec<_>>();
    assert_eq!(request_headers.len(), 2);
    assert_ne!(
        request_headers[0], request_headers[1],
        "the original selection subject and canonical candidate remain independently sealed"
    );

    // Both spellings normalize to the same canonical program, so every
    // downstream physical decision and receipt is identical.
    assert_eq!(
        from_duplicated.targets[0].portfolio,
        from_shared.targets[0].portfolio
    );

    // The traces differ only in what normalization actually did.
    let rewrite_counts = |product: &CompilationProduct| {
        product.targets[0]
            .selection_explain
            .records()
            .iter()
            .find(|record| record.rule().key().as_str() == "normalize.semantics.v1")
            .and_then(|record| match record.event() {
                ExplainEvent::Check { assessment, .. } => Some(
                    assessment
                        .facts()
                        .iter()
                        .find(|fact| fact.key().as_str() == "rewrite-count")
                        .map(|fact| fact.value().clone())
                        .unwrap(),
                ),
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(rewrite_counts(&from_duplicated), FactValue::Count(1));
    assert_eq!(rewrite_counts(&from_shared), FactValue::Count(0));
    assert!(
        from_duplicated.targets[0]
            .selection_explain
            .records()
            .iter()
            .any(
                |record| record.rule().key().as_str() == "normalize.common-subexpression.v1"
                    && record.event().disposition() == ExplainDisposition::Admitted
            )
    );
    assert!(
        !from_shared.targets[0]
            .selection_explain
            .records()
            .iter()
            .any(|record| record.rule().key().as_str() == "normalize.common-subexpression.v1")
    );
}

/// A shared constant read by two operations is graph fan-out, and a legal
/// cover must materialize it once rather than duplicate its producer.
#[test]
fn shared_constant_fan_out_is_materialized_once_and_never_duplicated() {
    let shared = shared_constant_semantic(Shape::from_dims([2, 2]), 2.0_f32.to_bits());
    let product = compile(CompilationRequest::governed(&shared)).unwrap();
    for alternative in &product.targets[0].portfolio.alternatives {
        assert!(
            alternative.plan.cover().duplication().is_none(),
            "producer duplication is disabled in this profile"
        );
        // Every cross-region value is one materialization edge with one or
        // more consumers, never one edge per consumer.
        let edges = alternative.plan.cover().materializations();
        let distinct = edges
            .iter()
            .map(crate::cover::MaterializationEdge::producer_position)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(edges.len(), distinct.len());
        assert_eq!(
            alternative.plan.handoffs().len(),
            edges.len(),
            "every materialization edge is discharged by exactly one handoff"
        );
    }
}

#[test]
fn valid_but_unsupported_program_has_a_capability_failure() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), input)
        .unwrap();
    let semantic = builder.build().unwrap();
    let error = compile(CompilationRequest::governed(&semantic)).unwrap_err();
    assert_eq!(
        error,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "operation-set",
        })
    );
    assert_eq!(
        error.to_string(),
        "compile.unsupported.strategy.operation-set: no installed capability can compile this valid semantic program"
    );
}

/// The workload's own projection shape is refused by the *target*, not by
/// recognition.
///
/// Two claims, and they are different. A program carrying the pinned workload's
/// `[128, 1024] x [3072, 1024]` projection is now recognized, lowered, scheduled,
/// and assembled — the request boundary and the lowering registry both admit it.
/// What refuses is the governed baseline target profile, whose `GridAxisThreads`
/// bound is four: 393,216 output elements is a hard-feasibility refusal naming
/// that axis, and it is the same refusal the four-element pointwise fixtures in
/// this file would get at this size.
///
/// Asserting a recognition refusal here would now be fiction about which check
/// said no, which is exactly what this test guarded against in the other
/// direction before the direct path landed. The compiling case is
/// `tests/contraction_direct_path.rs`, at a shape the baseline admits.
#[test]
fn a_contraction_of_the_workload_shape_is_refused_by_the_target_not_by_recognition() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let activations = builder
        .input::<F32>(
            InputKey::new("activations").unwrap(),
            Shape::from_dims([128, 1024]),
        )
        .unwrap();
    let weights = builder
        .input::<F32>(
            InputKey::new("weights").unwrap(),
            Shape::from_dims([3072, 1024]),
        )
        .unwrap();
    // `td,od->to`, spelled with the frontend's own labels.
    let structure = ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .unwrap();
    let projected =
        F32TensorContraction::apply(&mut builder, &structure, activations, weights).unwrap();
    builder
        .output(OutputKey::new("projected").unwrap(), projected)
        .unwrap();
    let semantic = builder.build().unwrap();
    assert_eq!(semantic.operation_count(), 1);

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("recognition, lowering, and assembly all admit the projection");
    let Some(CompileError::Explained { source, .. }) = product.targets[0].failure() else {
        panic!("the baseline profile launches at most four threads");
    };
    assert_eq!(
        source.as_ref(),
        &CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
            rule: "grid-axis",
            region: tiler_ir::schedule::RegionId::new(0),
            required: 393_216,
            available: 4,
        })),
        "the refusal is the target's launch bound, named as such",
    );
}

/// A pure-BF16 program builds and does not compile.
///
/// The gap this guards is the one registration opens: `builder.build()` now
/// succeeds for BF16, and "the program verifies" is the step most easily
/// mistaken for "the dtype works". Nothing below the semantic layer realizes
/// BF16 — there is no capability row, no lowering capability, and no region
/// vocabulary for it.
///
/// **Which boundary answers moved, and the move is the point.** This used to
/// assert the request-wide `dtype-f32` rule, because the recognizer ran before
/// any target was consulted and so no profile ever got to answer about BF16.
/// Target resolution now precedes recognition, and the refusal here is the
/// governed baseline's own: it declares dispatchability for `tiler::f32@1` and
/// says nothing about `tiler::bf16@1`, so the dtype is `Unknown` to it and the
/// program is rejected *per target* rather than for the whole request. That is
/// a strictly more specific answer — it names the profile that could not take
/// the program — and it is what lets a profile with measured BF16 rows report a
/// numerical verdict instead of being unreachable behind the recognizer.
///
/// `dtype-f32` has not gone away; it is what a BF16 program reaches on a profile
/// that *does* declare the dtype, which
/// `a_dispatchable_bf16_profile_reaches_the_recognizer_dtype_wall` covers.
#[test]
fn a_pure_bf16_program_is_statable_and_refused_at_the_request_boundary() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    // 1.0 and 2.0 in bf16.
    let one = Bf16Constant::apply(&mut builder, 0x3f80).unwrap();
    let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
    let scaled = Bf16Multiply::apply(&mut builder, input, one).unwrap();
    let shifted = Bf16Add::apply(&mut builder, scaled, two).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), shifted)
        .unwrap();
    let bf16_program = builder.build().unwrap();
    assert_eq!(bf16_program.operation_count(), 4);

    let product = compile(CompilationRequest::governed(&bf16_program))
        .expect("a target-local dtype refusal is a batch outcome, not a request error");
    let [target] = product.targets.as_slice() else {
        panic!("the governed request names exactly one target");
    };
    let Some(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
        RequestError::DTypeNotDispatchable {
            target_profile,
            resolved_type,
            disposition,
        },
    ))) = target.failure()
    else {
        panic!("expected the profile's own dtype refusal, got {target:?}");
    };
    assert_eq!(
        resolved_type.as_ref(),
        &tiler_ir::semantic::Bf16::resolved_type(),
        "the refusal names the exact value identity the profile was asked about",
    );
    assert_eq!(
        *disposition,
        crate::request::DTypeDispatchRefusalDisposition::Unknown,
        "silence about bf16 is Unknown, never an inherited f32 verdict",
    );
    assert_eq!(
        target_profile.as_str(),
        target.target_profile().profile_key().as_str(),
    );

    // The neighbour that keeps this about bf16 rather than about a dead
    // request path: the same shape of program in f32 compiles.
    compile(CompilationRequest::governed(&semantic(false)))
        .expect("the governed f32 fixture still compiles");
}

/// The `bf16` encodings this vertical's witnesses are stated in.
mod bf16_bits {
    /// `3.0`, the scale: a multiplier that forces a rounding at every ordinary
    /// operand rather than one that would pass under any implementation.
    pub(super) const THREE: u16 = 0x4040;
    /// `-0.0`, the bias: chosen so a signed zero and a subnormal survive the
    /// whole expression instead of being swallowed by a nonzero addend.
    pub(super) const NEGATIVE_ZERO: u16 = 0x8000;
    pub(super) const LEAST_POSITIVE_SUBNORMAL: u16 = 0x0001;
    pub(super) const LEAST_NEGATIVE_SUBNORMAL: u16 = 0x8001;
    /// A quiet NaN whose payload is *not* the family's canonical one.
    pub(super) const NONCANONICAL_NAN: u16 = 0x7fc1;
    pub(super) const CANONICAL_NAN: u16 = 0x7fc0;
    pub(super) const POSITIVE_INFINITY: u16 = 0x7f80;
    pub(super) const NEGATIVE_INFINITY: u16 = 0xff80;
    pub(super) const MAX_FINITE: u16 = 0x7f7f;
    /// `1 + 2^-7`, whose product with `3.0` lands exactly on a rounding tie.
    pub(super) const ONE_PLUS_ULP: u16 = 0x3f81;
}

/// The boundary payload the BF16 vertical is driven over.
///
/// Every class the region's arithmetic can distinguish, in one tensor: a signed
/// zero, both least subnormals, a non-canonical NaN, both infinities, the
/// overflow boundary from both sides, and a tie the rounding rule has to break.
const BF16_WITNESSES: [u16; 10] = [
    bf16_bits::NEGATIVE_ZERO,
    bf16_bits::LEAST_POSITIVE_SUBNORMAL,
    bf16_bits::LEAST_NEGATIVE_SUBNORMAL,
    bf16_bits::NONCANONICAL_NAN,
    bf16_bits::POSITIVE_INFINITY,
    bf16_bits::NEGATIVE_INFINITY,
    bf16_bits::MAX_FINITE,
    0xff7f,
    bf16_bits::ONE_PLUS_ULP,
    0x3f80,
];

/// The semantic `(x * 3.0) + (-0.0)` program in BF16, for the reference oracle.
fn bf16_semantic_program(key: &InputKey, elements: u64) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(key.clone(), Shape::from_dims([elements]))
        .unwrap();
    let scale = Bf16Constant::apply(&mut builder, bf16_bits::THREE).unwrap();
    let product = Bf16Multiply::apply(&mut builder, input, scale).unwrap();
    let bias = Bf16Constant::apply(&mut builder, bf16_bits::NEGATIVE_ZERO).unwrap();
    let root = Bf16Add::apply(&mut builder, product, bias).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    builder.build().unwrap()
}

/// The same computation as a verified BF16 scheduled region.
///
/// Assembled through `tiler-ir`'s public builders rather than through
/// `compile()`, and that is the measurement boundary this vertical carries: the
/// recognizer refuses every non-`f32` program under `dtype-f32` before a subject
/// is ever normalized, so no BF16 region is reachable from the request boundary
/// at this commit. What is established here is that the schedule, kernel, and
/// physical-carrier vocabularies admit and verify one — not that a caller can
/// ask for it.
fn bf16_scheduled_region(elements: u64) -> tiler_ir::schedule::VerifiedScheduledRegion {
    use tiler_ir::schedule::{
        Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
        ExceptionalValueAssumption, ExecutionBinding, InputOrdinal, KernelSchedule, LaunchPlan,
        LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder, ReductionTopology,
        ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    };

    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(InputOrdinal::FIRST).unwrap();
    let scale = expression.constant(bf16_bits::THREE).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bf16_bits::NEGATIVE_ZERO).unwrap();
    let root = expression.add(product, bias).unwrap();
    let expression = expression.build(root).unwrap();

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([elements]))
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Output),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::PointwiseBf16(expression))
        .unwrap();
    // The accepted `tiler.contract.bf16.v1` strict vector, restated as the
    // region's own realization: preserving subnormals in both dimensions, every
    // numeric-reshaping permission withheld, no exceptional value assumed
    // absent, and the family's canonical arithmetic NaN payload zero-extended
    // into the thirty-two-bit field. `NumericalContract::STRICT_BF16` resolves
    // exactly these dimensions; the region carries them rather than the contract
    // because a schedule preserves a realization, not a caller's request.
    builder
        .numerical(NumericalRealization::new(
            "tiler.test.strict-bf16",
            u32::from(tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
        ))
        .unwrap();
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder.build().unwrap()
}

fn bf16_tensor(elements: &[u16]) -> Tensor {
    Tensor::dense(
        Bf16::resolved_type(),
        Shape::from_dims([u64::try_from(elements.len()).unwrap()]),
        elements
            .iter()
            .map(|encoding| {
                ReferenceElement::from_float_bits(
                    encoding.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("a two-byte payload is a bounded element")
            })
            .collect(),
    )
    .expect("a bounded bf16 tensor")
}

fn bf16_tensor_bits(tensor: &Tensor) -> Vec<u16> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a bf16 result is dense");
    };
    elements
        .iter()
        .map(|element| {
            u16::from_be_bytes(
                <[u8; 2]>::try_from(element.as_bytes()).expect("a bf16 element is two bytes"),
            )
        })
        .collect()
}

/// A BF16 kernel's interpreted result agrees bit for bit with the reference.
///
/// **The two sides are independent.** The reference decodes each operand to its
/// exact rational value from the registered descriptor's own fields and rounds
/// once; the machine holds `bf16` encodings, computes in `f64`, and rounds with
/// its own half-to-even implementation. Neither consults the other, and neither
/// consults the semantic graph while executing the kernel: the machine resolves
/// buffer extents, addressing, predication, the constants, the arithmetic width,
/// and the NaN canonicalization from the structured kernel alone.
///
/// **The comparison runs in `tiler-compiler`** because it is the only crate that
/// sees both `tiler-ir`'s lowering and `tiler-reference`'s oracle: `tiler-ir`
/// declares no dependency on the reference and the reference depends on
/// `tiler-ir`, so an in-crate comparison would be against an oracle that is not
/// independent — the weaker claim, recorded as the stronger one.
#[test]
fn a_bf16_kernel_agrees_with_the_reference_oracle_bit_for_bit() {
    let elements = u64::try_from(BF16_WITNESSES.len()).unwrap();
    let scheduled = bf16_scheduled_region(elements);
    let kernel = tiler_ir::kernel::lower_scheduled_region(&scheduled)
        .expect("the bf16 region lowers to a verified kernel");

    // The physical carrier the compiler derives for this region is two bytes
    // wide, which is the other half of this ticket's stated outcome.
    assert_eq!(
        tiler_ir::program::StorageScalar::Bf16.byte_width(),
        2,
        "the bf16 carrier is two bytes at its single width authority"
    );

    let key = InputKey::new("x").unwrap();
    let program = bf16_semantic_program(&key, elements);
    let tensor = bf16_tensor(&BF16_WITNESSES);
    let outputs = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    let expected = bf16_tensor_bits(&outputs[0]);

    let interpreted = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Applied);
    assert_eq!(
        interpreted, expected,
        "the kernel's interpreted result differs from the reference"
    );

    // The witnesses reached the classes they were chosen for, so a fixture that
    // silently stopped exercising them would fail here rather than pass.
    assert_eq!(interpreted[0], bf16_bits::NEGATIVE_ZERO);
    assert_eq!(interpreted[1], 0x0003, "three least positive quanta");
    assert_eq!(interpreted[2], 0x8003, "three least negative quanta");
    assert_eq!(interpreted[3], bf16_bits::CANONICAL_NAN);
    assert_eq!(interpreted[4], bf16_bits::POSITIVE_INFINITY);
    assert_eq!(interpreted[5], bf16_bits::NEGATIVE_INFINITY);
    assert_eq!(
        interpreted[6],
        bf16_bits::POSITIVE_INFINITY,
        "three times the greatest finite value overflows"
    );
    assert_eq!(interpreted[7], bf16_bits::NEGATIVE_INFINITY);
    // `(1 + 2^-7) * 3` is exactly `193.5` quanta of the grid at that exponent,
    // so the tie rule decides it: half-to-even rounds up to `194`, which is
    // `3.03125`. A machine rounding half-away-from-zero agrees here, and one
    // rounding half-to-odd does not — this is the witness that makes the rule
    // observable rather than assumed.
    assert_eq!(interpreted[8], 0x4042);
    assert_eq!(interpreted[9], bf16_bits::THREE);
}

/// Dropping the BF16 canonicalization fails on exactly the NaN element.
///
/// The perturbation the CPU vertical uses, applied at this width: the same
/// kernel is interpreted by a machine modelling a lowering that emitted the BF16
/// arithmetic without its `CanonicalizeBf16Nan` beside it, and the reference
/// comparison is watched failing. What makes it evidence rather than a red test
/// is *where* it fails — the one element whose arithmetic result is a NaN, and
/// no other — so the conversion's obligation is the reason rather than a
/// coincidence.
#[test]
fn deleting_the_bf16_canonicalization_disagrees_at_exactly_the_nan_element() {
    let elements = u64::try_from(BF16_WITNESSES.len()).unwrap();
    let scheduled = bf16_scheduled_region(elements);
    let kernel = tiler_ir::kernel::lower_scheduled_region(&scheduled).unwrap();

    let canonical = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Applied);
    let perturbed = interpret_bf16(&kernel, &BF16_WITNESSES, Bf16Canonicalization::Omitted);

    let disagreements: Vec<usize> = canonical
        .iter()
        .zip(&perturbed)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect();
    assert_eq!(
        disagreements,
        vec![3],
        "only the non-canonical NaN witness depends on the canonicalization"
    );
    assert_eq!(canonical[3], bf16_bits::CANONICAL_NAN);
    assert_eq!(
        perturbed[3],
        bf16_bits::NONCANONICAL_NAN,
        "without the conversion the operand's own payload reaches the boundary"
    );
}

#[test]
fn budget_exhaustion_is_not_reported_as_unsupported() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.budgets.semantic_operations = 4;
    let error = compile(request).unwrap_err();
    assert_eq!(
        error,
        CompileError::BudgetExhausted(RequestError::BudgetExceeded {
            resource: "semantic-operations",
            limit: 4,
            actual: 5,
        })
    );
}

#[test]
fn malformed_request_is_not_reported_as_missing_capability() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.target_profiles.clear();
    assert_eq!(
        compile(request),
        Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
    );
}

#[test]
fn target_outcomes_preserve_caller_order_in_both_directions() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.success.v1");
    let no_contract = TargetProfile::without_numerical_declarations_for_test("test.no-contract.v1");
    for profiles in [
        vec![success.clone(), no_contract.clone()],
        vec![no_contract.clone(), success.clone()],
    ] {
        let expected_keys = profiles
            .iter()
            .map(|profile| profile.profile_key().as_str().to_owned())
            .collect::<Vec<_>>();
        let product = compile(request_with_targets(
            &semantic,
            profiles,
            vec![StrictF32NumericalContract::governed()],
        ))
        .expect("a target-local numerical refusal does not fail the batch");
        assert_eq!(
            product
                .targets
                .iter()
                .map(|outcome| outcome.target_profile().profile_key().as_str())
                .collect::<Vec<_>>(),
            expected_keys.iter().map(String::as_str).collect::<Vec<_>>()
        );
        assert!(
            outcome_for_key(&product, "test.success.v1")
                .compiled()
                .is_some()
        );
        assert!(matches!(
            outcome_for_key(&product, "test.no-contract.v1").failure(),
            Some(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
                RequestError::NoResolvableNumericalContract { .. }
            )))
        ));
    }
}

#[test]
fn target_identity_is_independent_of_batch_order() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.identity.v1");
    let no_contract = TargetProfile::without_numerical_declarations_for_test("test.companion.v1");
    let compile_order = |profiles| {
        compile(request_with_targets(
            &semantic,
            profiles,
            vec![StrictF32NumericalContract::governed()],
        ))
        .unwrap()
    };
    let forward = compile_order(vec![success.clone(), no_contract.clone()]);
    let reverse = compile_order(vec![no_contract, success]);
    assert_eq!(
        outcome_for_key(&forward, "test.identity.v1").compiled(),
        outcome_for_key(&reverse, "test.identity.v1").compiled()
    );
}

#[test]
fn distinct_resolved_contracts_are_compiled_as_two_groups() {
    let semantic = semantic(false);
    let strict = TargetProfile::governed_with_key_for_test("test.strict.v1");
    let flush = TargetProfile::flush_only_for_test("test.flush.v1");
    let (result, group_count) = observe_contract_group_compilations(|| {
        compile(request_with_targets(
            &semantic,
            vec![strict, flush],
            vec![
                StrictF32NumericalContract::governed(),
                StrictF32NumericalContract::governed_flush_to_zero(),
            ],
        ))
    });
    let product = result.unwrap();
    assert_eq!(group_count, 2);
    assert_eq!(
        outcome_for_key(&product, "test.strict.v1")
            .compiled()
            .unwrap()
            .resolved_contract,
        StrictF32NumericalContract::governed()
    );
    assert_eq!(
        outcome_for_key(&product, "test.flush.v1")
            .compiled()
            .unwrap()
            .resolved_contract,
        StrictF32NumericalContract::governed_flush_to_zero()
    );
}

#[test]
fn one_target_failure_does_not_erase_a_companion_in_the_same_group() {
    let semantic = semantic(false);
    let success = TargetProfile::governed_with_key_for_test("test.isolation.success.v1");
    let bounded = TargetProfile::with_grid_axis_limit_for_test("test.isolation.bounded.v1", 1);
    let (result, group_count) = observe_contract_group_compilations(|| {
        compile(request_with_targets(
            &semantic,
            vec![success, bounded],
            vec![StrictF32NumericalContract::governed()],
        ))
    });
    let product = result.expect("target-local feasibility cannot erase its companion");
    assert_eq!(group_count, 1);
    assert!(
        outcome_for_key(&product, "test.isolation.success.v1")
            .compiled()
            .is_some()
    );
    assert!(matches!(
        outcome_for_key(&product, "test.isolation.bounded.v1").failure(),
        Some(CompileError::Explained { source, .. })
            if matches!(
                source.as_ref(),
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                    PhysicalError::Target { .. }
                ))
            )
    ));
}

#[test]
fn empty_and_duplicate_target_sets_are_outer_request_failures() {
    let semantic = semantic(false);
    let mut empty = CompilationRequest::governed(&semantic);
    empty.target_profiles.clear();
    assert_eq!(
        compile(empty),
        Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
    );

    let duplicate = TargetProfile::governed_with_key_for_test("test.duplicate.v1");
    assert_eq!(
        compile(request_with_targets(
            &semantic,
            vec![duplicate.clone(), duplicate],
            vec![StrictF32NumericalContract::governed()],
        )),
        Err(CompileError::InvalidRequest(
            RequestError::DuplicateTargetProfile
        ))
    );
}

#[test]
fn target_group_cardinality_mismatch_is_an_outer_compiler_invariant() {
    let semantic = semantic(false);
    let verified = verify_planned_request(request_with_targets(
        &semantic,
        vec![
            TargetProfile::governed_with_key_for_test("test.group.first.v1"),
            TargetProfile::governed_with_key_for_test("test.group.second.v1"),
        ],
        vec![StrictF32NumericalContract::governed()],
    ))
    .unwrap();
    let group = resolved_target_groups(&verified).remove(0);
    let candidate = verified
        .readmit_candidate(&semantic, &group.target_indexes[..1])
        .unwrap();
    assert_eq!(
        verify_target_group_coordination(&verified, &group, &candidate),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "target-group-cardinality"
            })
        ))
    );
}

#[test]
fn invalid_compiler_output_from_target_compilation_remains_outer() {
    let target = TargetProfile::governed_with_key_for_test("test.outer-invariant.v1");
    let result = target_compilation_outcome(
        &target,
        Err(TargetCompileFailure::Outer(
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "test-target-compiler-invariant",
                },
            )),
        )),
    );
    assert!(matches!(
        result,
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "test-target-compiler-invariant"
            })
        ))
    ));
}

#[test]
fn a_caller_declared_target_profile_reaches_target_feasibility() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.target_profiles[0] = crate::request::TargetProfile::governed_with_grid_axis_limit(1);
    let product = compile(request).expect("the well-formed caller profile is admitted");
    assert!(matches!(
        product.targets[0].failure(),
        Some(CompileError::Explained { source, .. })
            if matches!(
                source.as_ref(),
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                    PhysicalError::Target { .. }
                ))
            )
    ));
}

/// An installed authority that lowers nothing is a deferred capability, and
/// it stops the compilation instead of quietly producing a narrower
/// portfolio: an occurrence nobody can lower has no valid plan at all.
#[test]
fn a_registry_without_capabilities_defers_and_fails_closed() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.capabilities = CompilerCapabilitySnapshot::without_capabilities();
    let error = compile(request).unwrap_err();
    let CompileError::Explained { source, explain } = error else {
        panic!("target compilation failures retain their explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: "missing-capability",
        })
    );
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.event().disposition() == ExplainDisposition::DeferredUnsupported
    }));
    let failure = explain
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .expect("a terminal failure record");
    assert!(matches!(
        failure.event(),
        ExplainEvent::CompilerFailure {
            stage: ExplainStage::CapabilityResolution,
            reason,
        } if reason.as_str() == "lowering-missing-capability"
    ));
}

#[test]
fn region_budget_retains_the_verified_baseline() {
    let semantic = semantic(false);
    // A zero per-seed growth budget leaves only singleton candidates, and the
    // bounded profile implements no singleton region. Every plan therefore
    // depends on a region that was never formed, so compilation fails closed
    // with a typed no-complete-plan error rather than implementing a region
    // region formation never proposed.
    let mut bounded = CompilationRequest::governed(&semantic);
    bounded.budgets.region_candidates_per_seed = 0;
    let product = compile(bounded).expect("a target-local refusal is an ordered outcome");
    let CompileError::Explained { source, explain } = product.targets[0]
        .failure()
        .expect("the bounded target has no complete plan")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        source.as_ref(),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Selection(SelectionError::Structure {
            rule: "no-complete-plan"
        }))
    ));
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "region.formation.v1"
            && record.event().disposition() == ExplainDisposition::BudgetStopped
    }));
    assert_eq!(
        explain
            .records()
            .iter()
            .filter(|record| record.rule().key().as_str() == "region.candidate.v1")
            .count(),
        5
    );
}

/// A cover budget never loses the two covers the enumerator retains
/// unconditionally — the all-singleton and the whole-program cover — and any
/// discovered partition it does lose is reported as a typed budget stop.
///
/// The bounded profile implements no singleton region, so the all-singleton
/// cover yields no plan. Losing the discovered two-region partition therefore
/// costs the materialized alternative, which is exactly what the typed stop
/// makes visible instead of silently narrowing the portfolio.
#[test]
fn cover_budget_stops_are_reported_without_losing_either_extreme() {
    let semantic = semantic(false);
    let mut bounded = CompilationRequest::governed(&semantic);
    bounded.budgets.region_covers = 1;
    let product = compile(bounded).unwrap();
    assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
    assert_eq!(selected_kind(&product), ProgramAlternativeKind::Fused);
    assert!(product.targets[0].explain.records().iter().any(|record| {
        record.rule().key().as_str() == "cover.enumeration.v1"
            && record.event().disposition() == ExplainDisposition::BudgetStopped
    }));
}

#[test]
fn infeasible_baseline_does_not_suppress_a_feasible_fused_plan() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([70_000, 2]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(0),
    );

    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];
    assert_eq!(target.portfolio.alternatives.len(), 1);
    assert_eq!(
        target.portfolio.alternatives[0].kind,
        ProgramAlternativeKind::Fused
    );
    let pointwise = region_subject_key(&target.explain, "pointwise")
        .expect("the pointwise region subject reached the frontier");
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "target.grid-axis"
            && record.subjects()[0].key().as_str() == pointwise
            && record.event().disposition() == ExplainDisposition::RejectedTarget
            && matches!(
                record.event(),
                ExplainEvent::Feasibility {
                    required: Quantity::Threads(140_000),
                    available: Quantity::Threads(4),
                    ..
                }
            )
    }));
    // The cover whose pointwise region the target refused is retained in the
    // terminal ledger as an infeasible alternative rather than disappearing.
    assert!(target.explain.records().iter().any(|record| {
        matches!(
            record.event(),
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Infeasible,
                ..
            }
        )
    }));
}

#[test]
fn the_governed_grid_authority_admits_four_and_refuses_five() {
    let bounded = semantic_case(
        Shape::from_dims([4, 1]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
    );
    let accepted = compile(CompilationRequest::governed(&bounded))
        .expect("the governed four-thread serial sum compiles");
    assert!(accepted.targets[0].compiled().is_some());

    let oversized = semantic_case(
        Shape::from_dims([5, 1]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
    );
    let refused = compile(CompilationRequest::governed(&oversized))
        .expect("a target-local refusal remains an ordered compilation outcome");
    let CompileError::Explained { source, explain } = refused.targets[0]
        .failure()
        .expect("the five-thread target is refused")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        source.as_ref(),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target { .. }))
    ));
    assert!(explain.records().iter().any(|record| {
        matches!(
            record.event(),
            ExplainEvent::Feasibility {
                predicate,
                required: Quantity::Threads(5),
                available: Quantity::Threads(4),
                ..
            } if predicate.as_str() == "grid-axis"
        )
    }));
}

#[test]
fn no_feasible_plan_retains_a_typed_terminal_failure_trace() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([70_000, 70_000]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a target-local refusal is an ordered outcome");
    let CompileError::Explained { source, explain } = product.targets[0]
        .failure()
        .expect("the target has no feasible plan")
    else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        source.as_ref(),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target { .. }))
    ));
    assert_eq!(
        explain
            .records()
            .iter()
            .filter(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .count(),
        1
    );
    let failure = explain
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .unwrap();
    assert!(matches!(
        failure.event(),
        ExplainEvent::CompilerFailure {
            stage: ExplainStage::TargetFeasibility,
            reason,
        } if reason.as_str() == "target-grid-axis"
    ));
    let causal_rejections = failure
        .causes()
        .iter()
        .map(|cause| {
            explain
                .records()
                .iter()
                .find(|record| record.id() == *cause)
                .expect("every failure cause is a retained exact target rejection")
        })
        .collect::<Vec<_>>();
    assert!(!causal_rejections.is_empty());
    assert!(
        causal_rejections
            .iter()
            .all(|record| { record.event().disposition() == ExplainDisposition::RejectedTarget })
    );
    // Every recognized region the target refused is named exactly once, by the
    // region's own explain subject rather than by its role: the roles below
    // resolve to three distinct occurrence labels, and a rejection keyed by role
    // could not have told them apart from the eleven other subjects this
    // program covers.
    let mut subjects = causal_rejections
        .iter()
        .map(|record| record.subjects()[0].key().as_str().to_owned())
        .collect::<Vec<_>>();
    subjects.sort();
    let mut expected = ["pointwise", "reduction", "whole-program"]
        .into_iter()
        .map(|role| {
            region_subject_key(explain, role)
                .unwrap_or_else(|| panic!("the {role} region subject reached the frontier"))
        })
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(subjects, expected);
}

#[test]
fn target_rejections_are_deduplicated_by_region_role_and_axis() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let mut explain = ExplainWriter::new(&request).unwrap();
    let pointwise = PhysicalError::Target {
        rule: "grid-axis",
        region: RegionId::new(0),
        required: 65_536,
        available: 65_535,
    };
    let fused = PhysicalError::Target {
        rule: "threads-per-workgroup",
        region: RegionId::new(1),
        required: 2,
        available: 1,
    };
    let root = test_root(&mut explain);
    let pointwise_cause =
        record_target_rejection(&mut explain, &pointwise, "pointwise", root).unwrap();
    let fused_cause = record_target_rejection(&mut explain, &fused, "whole-program", root).unwrap();
    let mut rejections = TargetRejections::default();
    rejections
        .push(TargetRejection {
            role: "whole-program",
            error: fused.clone(),
            cause: fused_cause,
        })
        .unwrap();
    rejections
        .push(TargetRejection {
            role: "pointwise",
            error: pointwise,
            cause: pointwise_cause,
        })
        .unwrap();
    // The same role and axis observed on another cover adds no second cause.
    rejections
        .push(TargetRejection {
            role: "whole-program",
            error: fused,
            cause: fused_cause,
        })
        .unwrap();
    let failure = rejections.into_failure().unwrap();
    let trace = explain.finish_failure(*failure.context).unwrap();
    let terminal = trace
        .records()
        .iter()
        .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
        .unwrap();
    assert_eq!(terminal.causes().len(), 2);
    let predicates = terminal
        .causes()
        .iter()
        .map(|cause| {
            trace
                .records()
                .iter()
                .find(|record| record.id() == *cause)
                .and_then(|record| match record.event() {
                    ExplainEvent::Feasibility { predicate, .. } => Some(predicate.as_str()),
                    _ => None,
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(predicates, ["grid-axis", "threads-per-workgroup"]);
}

#[test]
fn physical_error_stages_are_attributed_to_their_exact_phase() {
    assert_eq!(
        physical_error_stage(&PhysicalError::Target {
            rule: "grid-axis",
            region: RegionId::new(0),
            required: 2,
            available: 1,
        }),
        ExplainStage::TargetFeasibility
    );
    assert_eq!(
        physical_error_stage(&PhysicalError::Intrinsic {
            rule: "fixture",
            region: RegionId::new(0),
        }),
        ExplainStage::IntrinsicScheduling
    );
    assert_eq!(
        physical_error_stage(&PhysicalError::ShapeProductOverflow {
            region: RegionId::new(0),
        }),
        ExplainStage::IntrinsicScheduling
    );
    assert_eq!(
        physical_error_stage(&PhysicalError::Refinement {
            rule: "fixture",
            region: RegionId::new(0),
        }),
        ExplainStage::KernelRefinement
    );
}

#[test]
fn structural_policy_requires_pareto_dominance_instead_of_guessing_latency() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    // Fusion is strictly better on every structural dimension here, so it
    // dominates; the reverse comparison must not hold.
    assert!(
        fused
            .structural_cost
            .dominates(&materialized.structural_cost)
    );
    assert!(
        !materialized
            .structural_cost
            .dominates(&fused.structural_cost)
    );
    // Dominance is a partial order: a plan never dominates itself.
    assert!(!fused.structural_cost.dominates(&fused.structural_cost));
    // The selection is the first non-dominated plan in canonical order, so
    // it is exactly the plan the portfolio's own Pareto view retains.
    let retained = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .filter(|candidate| {
            !product.targets[0]
                .portfolio
                .alternatives
                .iter()
                .any(|other| other.structural_cost.dominates(&candidate.structural_cost))
        })
        .map(|candidate| candidate.stable_id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        retained,
        [product.targets[0]
            .portfolio
            .selection
            .selected_alternative_id
            .clone()]
    );
}

#[test]
fn structured_fused_body_interpreter_matches_reference_evaluator() {
    assert_fused_matches_reference(
        Shape::from_dims([2, 2]),
        vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE],
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
    );
    assert_fused_matches_reference(
        Shape::from_dims([4, 1]),
        vec![-0.0, f32::from_bits(1), f32::INFINITY, f32::NAN],
        1.0_f32.to_bits(),
        0.0_f32.to_bits(),
    );
    assert_fused_matches_reference(
        Shape::from_dims([2, 0]),
        Vec::new(),
        f32::NAN.to_bits(),
        f32::NEG_INFINITY.to_bits(),
    );
    let contraction_input = 1.000_000_1_f32;
    let contraction_scale = 1.000_000_1_f32;
    let contraction_bias = -1.000_000_2_f32;
    assert_ne!(
        (contraction_input * contraction_scale + contraction_bias).to_bits(),
        contraction_input
            .mul_add(contraction_scale, contraction_bias)
            .to_bits(),
        "the conformance vector must distinguish separate operations from FMA"
    );
    assert_fused_matches_reference(
        Shape::from_dims([1, 2]),
        vec![contraction_input, -1.0],
        contraction_scale.to_bits(),
        contraction_bias.to_bits(),
    );
}

/// A lone contributor's NaN payload must not survive the reduction boundary.
///
/// The strict serial sum canonicalizes at its result boundary "even when the
/// contributor sequence is a singleton" (`docs/numerical-semantics.md`, ADR
/// 0055). A reduced axis of extent one is exactly where that rule is
/// load-bearing rather than redundant: no combine has run, so nothing else
/// has canonicalized the value being written.
///
/// `structured_fused_body_interpreter_matches_reference_evaluator` cannot
/// see this. Its `[4, 1]` vector carries `f32::NAN`, which already *is*
/// `CANONICAL_F32_ARITHMETIC_NAN_BITS`, and it interprets the fused kernel,
/// whose scale/bias prologue canonicalizes the seed regardless. This case
/// interprets the materialized alternative's bare `StrictSerialSum` kernel
/// and supplies the payload directly.
#[test]
fn a_singleton_reduction_canonicalizes_a_lone_non_canonical_nan() {
    let shape = Shape::from_dims([4, 1]);
    let semantic = semantic_case(shape.clone(), 1.0_f32.to_bits(), 0.0_f32.to_bits(), false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
    let reduction = &materialized.kernels[1];
    assert_eq!(
        reduction.buffers().next().unwrap().tensor,
        TensorRole::Intermediate,
        "the second materialized kernel reduces the materialized intermediate"
    );

    // The intermediate is an ordinary runtime buffer whose declared element
    // domain is every binary32 pattern, not only the ones this program's own
    // prologue happens to produce.
    let intermediate = vec![
        f32::from_bits(0x7fc0_1234),
        f32::from_bits(0xffc0_0000),
        -0.0_f32,
        f32::from_bits(1),
    ];
    let actual: Vec<u32> = interpret_fused(reduction, &intermediate)
        .iter()
        .map(|value| value.to_bits())
        .collect();

    let key = InputKey::new("input").unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let bare = builder.input::<F32>(key.clone(), shape.clone()).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, bare, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let bare_sum = builder.build().unwrap();
    let tensor = Tensor::dense(
        F32::resolved_type(),
        shape,
        intermediate
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
    .unwrap();
    let evaluated = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&bare_sum, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    let expected: Vec<u32> = match evaluated[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
            .collect(),
        _ => panic!("expected dense f32 reference output"),
    };
    assert_eq!(
        expected,
        [
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
            (-0.0_f32).to_bits(),
            1,
        ],
        "the boundary rule rewrites both NaN payloads and preserves every other one"
    );
    assert_eq!(
        actual, expected,
        "the compiled kernel must realize that rule"
    );
}

/// The structured addressing must realize a non-trailing reduced axis.
///
/// A leading reduced axis makes the contributor stride differ from one, and
/// a middle reduced axis additionally forces the kept coordinate to be
/// recovered with an explicit index division and remainder. Both are lowered
/// as ordinary index arithmetic, so interpreting the emitted operations must
/// still reproduce the reference evaluator exactly.
#[test]
fn structured_addressing_realizes_non_trailing_reduction_axes() {
    assert_fused_axis_matches_reference(
        Shape::from_dims([3, 2]),
        vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, 0.0],
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        Axis::new(0),
    );
    assert_fused_axis_matches_reference(
        Shape::from_dims([2, 3, 2]),
        (0..12_u8).map(|value| f32::from(value) - 4.0).collect(),
        0.5_f32.to_bits(),
        (-0.25_f32).to_bits(),
        Axis::new(1),
    );
}

#[test]
fn portfolio_selection_and_evidence_are_recomputed_from_exact_contents() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];
    let alternatives = &target.portfolio.alternatives;
    let selected = target.portfolio.selection.selected_alternative_id.clone();
    let portfolio = plan_portfolio(&semantic, &request);

    assert!(
        verify_portfolio(
            &semantic,
            &request,
            &plan_formation(&semantic, &request),
            &portfolio,
            alternatives,
            &selected,
            None
        )
        .is_ok()
    );
    assert!(
        verify_portfolio(
            &semantic,
            &request,
            &plan_formation(&semantic, &request),
            &portfolio,
            &[],
            &selected,
            None
        )
        .is_err()
    );
    let selection = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        alternatives,
        "stale-selection",
        None,
    )
    .unwrap_err();
    assert_eq!(selection.context.stage, ExplainStage::Selection);
    assert_eq!(
        selection.context.reason.as_str(),
        "structure-portfolio-selection"
    );

    let mut forged = alternatives.clone();
    forged[0].stable_id = "forged-plan".to_owned();
    let identity = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(identity.context.stage, ExplainStage::Costing);

    let mut forged_artifact = alternatives.clone();
    forged_artifact[0].artifact_plan = forged_artifact[1].artifact_plan.clone();
    let artifact = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged_artifact,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(artifact.context.stage, ExplainStage::ArtifactPlanning);

    let mut forged_numerics = alternatives.clone();
    forged_numerics[0].equivalence = forged_numerics[1].equivalence.clone();
    let numerical = verify_portfolio(
        &semantic,
        &request,
        &plan_formation(&semantic, &request),
        &portfolio,
        &forged_numerics,
        &selected,
        None,
    )
    .unwrap_err();
    assert_eq!(numerical.context.stage, ExplainStage::NumericalLegality);
    assert_eq!(
        numerical.context.reason.as_str(),
        "structure-portfolio-equivalence"
    );
}

/// The formation a verified target request runs under.
fn plan_formation(
    semantic: &SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
) -> crate::region::RegionFormationOutcome {
    form_region_candidates(semantic, request.budgets(), request.numerical_contract())
        .expect("the fixture forms regions")
}

/// Re-derives the selected portfolio for a verified target request.
fn plan_portfolio(
    semantic: &SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
) -> crate::selection::SelectedPortfolio {
    let mut explain = ExplainWriter::new(request).unwrap();
    let formation =
        form_region_candidates(semantic, request.budgets(), request.numerical_contract()).unwrap();
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

/// A retained opaque plan reaches the lowering boundary and is refused there,
/// rather than having its absent schedule filtered out.
#[test]
fn lowering_refuses_an_opaque_plan_before_program_assembly() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let formation = plan_formation(&semantic, &request);
    let mut explain = ExplainWriter::new(&request).unwrap();
    let root = test_root(&mut explain);
    let complete = enumerate_complete_plans(
        &semantic,
        &request,
        &formation,
        &PhysicalAuthorities::governed(),
        &mut explain,
        root,
        None,
    )
    .expect("the governed compile enumerates its support evidence");
    let opaque = crate::selection::opaque_fused_portfolio_fixture(&semantic);
    let plan = opaque
        .plans()
        .iter()
        .find(|plan| crate::program::CoverAssembly::from_plan(&semantic, plan).is_err())
        .expect("one opaque plan");

    let error = build_alternative(
        &semantic,
        &request,
        plan,
        ProgramAlternativeKind::Fused,
        &complete,
        None,
    )
    .unwrap_err();
    assert_eq!(error.context.stage, ExplainStage::ProgramVerification);
    assert_eq!(
        error.context.reason.as_str(),
        "structure-unlowerable-opaque-body"
    );
}

/// Verification independently re-derives the schedule binding and refuses a
/// receipt whose selected plan contains an opaque body.
#[test]
fn verification_refuses_an_alternative_with_an_opaque_plan() {
    let semantic = semantic(false);
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let formation = plan_formation(&semantic, &request);
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let mut forged = alternative(&compiled, ProgramAlternativeKind::Fused).clone();
    let opaque = crate::selection::opaque_fused_portfolio_fixture(&semantic);
    let plan = opaque
        .plans()
        .iter()
        .find(|plan| crate::program::CoverAssembly::from_plan(&semantic, plan).is_err())
        .expect("one opaque plan")
        .clone();
    forged.structural_cost = plan.cost();
    forged.plan = plan;
    forged.identity = ProgramAlternativeIdentity::new(
        SemanticAlternativeOrigin::Baseline,
        &semantic,
        &request,
        &forged.plan,
    );
    forged.stable_id = forged.identity.label();

    let lowering = resolve_lowering(&semantic, &request).unwrap();
    let error = super::verify::verify_alternative(
        &semantic, &request, &formation, &forged, &lowering, None,
    )
    .unwrap_err();
    assert_eq!(error.context.stage, ExplainStage::ProgramVerification);
    assert_eq!(
        error.context.reason.as_str(),
        "structure-portfolio-schedule-binding"
    );
}

#[test]
fn global_semantic_selection_rejects_a_forged_winner() {
    let semantic = semantic(false);
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let mut portfolio = compiled.targets[0].portfolio.clone();
    let forged = portfolio
        .alternatives
        .iter()
        .find(|alternative| alternative.stable_id != portfolio.selection.selected_alternative_id)
        .expect("the fixture retains a non-selected physical alternative")
        .stable_id
        .clone();
    portfolio.selection.selected_alternative_id = forged;

    let error = verify_global_selection(&portfolio).unwrap_err();
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
            ProgramError::Structure {
                rule: "semantic-portfolio-selection"
            }
        ))
    ));
}

#[test]
fn final_portfolio_verifier_rejects_deletion_owner_and_origin_misbinding() {
    let semantic = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let portfolio = compiled.targets[0].portfolio.clone();
    let expected_identities = portfolio
        .alternatives
        .iter()
        .map(|alternative| alternative.identity.clone())
        .collect();
    let expected = [ExpectedCandidateOwner {
        key: "semantic:baseline".to_owned(),
        origin: SemanticAlternativeOrigin::Baseline,
        semantic: &semantic,
        request: request.clone(),
        alternatives: expected_identities,
    }];
    assert!(verify_global_portfolio(&portfolio, &expected).is_ok());

    let mut deleted = portfolio.clone();
    deleted.alternatives.pop();
    assert!(matches!(
        verify_global_portfolio(&deleted, &expected),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-set"
            })
        ))
    ));

    let mut misowned = portfolio.clone();
    misowned.alternatives[0].owner_key = "semantic:wrong-owner".to_owned();
    assert!(matches!(
        verify_global_portfolio(&misowned, &expected),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-binding"
            })
        ))
    ));

    let wrong_origin = RewriteRuleIdentity::new("test", "wrong-origin", 1).unwrap();
    let wrong_expected = [ExpectedCandidateOwner {
        key: "semantic:baseline".to_owned(),
        origin: SemanticAlternativeOrigin::Rewrite(wrong_origin),
        semantic: &semantic,
        request,
        alternatives: portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.identity.clone())
            .collect(),
    }];
    assert!(matches!(
        verify_global_portfolio(&portfolio, &wrong_expected),
        Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "semantic-portfolio-owner-binding"
            })
        ))
    ));
}

#[test]
fn contract_groups_fall_back_after_infeasibility_and_do_not_plan_later_groups() {
    let stated = StrictF32NumericalContract::named_profile();
    let groups = vec![
        (stated[0].key, vec![("preferred", false)]),
        (stated[1].key, vec![("fallback", true)]),
        (stated[2].key, vec![("later", true)]),
    ];
    let mut evaluated = Vec::new();
    let outcome = evaluate_preferred_groups(
        &stated,
        groups,
        |item| {
            evaluated.push(item.0);
            Ok::<_, ()>(item)
        },
        |item| item.1,
        |_| (),
    )
    .unwrap();

    assert_eq!(outcome.selected_contract, Some(stated[1].key));
    assert_eq!(evaluated, ["preferred", "fallback"]);
    assert_eq!(
        outcome
            .evaluated
            .iter()
            .map(|item| item.0)
            .collect::<Vec<_>>(),
        ["preferred", "fallback"]
    );
    assert_eq!(outcome.pruned, [(("later", true), stated[1].key)]);
}

#[test]
fn contract_group_evaluation_rejects_an_unstated_contract_key() {
    let stated = StrictF32NumericalContract::named_profile();
    let error = evaluate_preferred_groups(
        &stated,
        vec![("test.unstated-contract", vec![("candidate", true)])],
        Ok::<_, CompileError>,
        |item| item.1,
        |_| {
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-unstated-contract",
                },
            ))
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
            ProgramError::Structure {
                rule: "semantic-portfolio-unstated-contract"
            }
        ))
    ));
}

#[test]
fn live_semantic_portfolio_explains_every_governed_rule_decline_stably() {
    let semantic = semantic(false);
    let first = compile(CompilationRequest::governed(&semantic)).unwrap();
    let second = compile(CompilationRequest::governed(&semantic)).unwrap();
    let first = first.targets[0].compilation_explain.render();
    let second = second.targets[0].compilation_explain.render();

    assert_eq!(first, second);
    for rule in [
        "ordered-reassociate-add-f32.v1",
        "ordered-reassociate-multiply-f32.v1",
    ] {
        assert!(
            first.contains(rule),
            "the complete rule identity must remain visible when the rule declines"
        );
    }
    assert!(first.contains("disproved:semantic.no-left-associated-chain"));
}

#[test]
fn relaxed_reassociation_reaches_verified_global_physical_selection() {
    let semantic = tensor_add_chain();
    let product = compile(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let target = &product.targets[0];

    assert!(
        target.portfolio.alternatives.iter().any(|alternative| {
            alternative.owner_key == "semantic:baseline" && alternative.program.stage_count() == 1
        }),
        "the unchanged semantic baseline remains physically available",
    );
    let reassociated = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative
                .owner_key
                .contains("ordered-reassociate-add-f32.v1")
                && alternative.program.stage_count() == 1
        })
        .expect("the accepted reassociation reaches a verified program under its own owner");
    assert_eq!(
        reassociated.scheduled_regions[0].semantic_members(),
        [
            crate::region::SemanticMemberId(0),
            crate::region::SemanticMemberId(1),
            crate::region::SemanticMemberId(2),
            crate::region::SemanticMemberId(3),
        ],
    );
    assert_eq!(reassociated.equivalence.legality().len(), 1);
    let exploration = explore_algebraic_alternatives_owned(
        semantic.clone(),
        crate::request::DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed_relaxed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let rewritten = exploration
        .alternatives()
        .iter()
        .find(|alternative| {
            alternative.rule() == crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap()
        })
        .expect("the relaxed contract admits the add reassociation")
        .candidate();
    let rewritten_request = verify_planned_request(CompilationRequest::governed_under(
        rewritten,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let rewritten_request = rewritten_request
        .for_target(rewritten_request.target_profiles()[0])
        .unwrap();
    let lowering = resolve_lowering(rewritten, &rewritten_request).unwrap();
    assert_eq!(
        lowering
            .occurrences()
            .iter()
            .map(crate::lowering::OccurrenceLowering::member)
            .collect::<Vec<_>>(),
        [
            crate::region::SemanticMemberId(0),
            crate::region::SemanticMemberId(1),
            crate::region::SemanticMemberId(2),
            crate::region::SemanticMemberId(3),
        ],
        "the rewritten program resolves all four semantic occurrences",
    );
    assert!(
        lowering
            .occurrences()
            .iter()
            .all(|occurrence| matches!(occurrence.evidence(), OccurrenceEvidence::Refined(_))),
        "each rewritten occurrence carries checked refinement evidence",
    );
    assert!(
        target.portfolio.alternatives.iter().any(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        }),
        "global selection names one verified flattened alternative",
    );
}

#[test]
fn pointwise_region_roles_require_the_exact_whole_program_subject() {
    let semantic = tensor_add_chain();
    let verified = verify_planned_request(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_relaxed(),
    ))
    .unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let members = [
        crate::region::SemanticMemberId(0),
        crate::region::SemanticMemberId(1),
        crate::region::SemanticMemberId(2),
        crate::region::SemanticMemberId(3),
    ];

    assert_eq!(region_role(&request, &members), "whole-program");
    for member in members {
        assert_eq!(region_role(&request, &[member]), "unrecognized");
    }
}

#[test]
fn strict_contract_keeps_the_pointwise_baseline_and_declines_reassociation() {
    let semantic = tensor_add_chain();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let target = &product.targets[0];

    assert!(
        target
            .portfolio
            .alternatives
            .iter()
            .all(|alternative| alternative.owner_key == "semantic:baseline"),
    );
    assert!(
        target
            .compilation_explain
            .render()
            .contains("numerical.reassociation-forbidden"),
    );
}

#[test]
fn live_semantic_portfolio_renders_per_rule_disablement() {
    let semantic = semantic(false);
    let add = crate::rewrite::ORDERED_REASSOCIATE_ADD_RULE.unwrap();
    let configuration = AlgebraicRuleConfiguration::all().with(add, false);
    let product = compile_configured(
        CompilationRequest::governed(&semantic),
        configuration,
        &PhysicalAuthorities::governed(),
    )
    .unwrap();
    let rendered = product.targets[0].compilation_explain.render();

    assert!(
        rendered.contains("rewrite.configuration-enabled:disproved:configuration.rule-disabled")
    );
    assert!(rendered.contains("rewrite-provider:identity=tiler.algebraic"));
    assert!(rendered.contains("rewrite-rule:identity=ordered-reassociate-add-f32.v1"));
    assert!(rendered.contains("rewrite-revision:count=1"));
    assert!(
        rendered.contains("ordered-reassociate-multiply-f32.v1"),
        "disabling add must not remove multiply's independent assessment"
    );
}

#[test]
fn top_level_emitter_renders_strict_numerical_decline_and_algebraic_budget_stop() {
    let chain = algebraic_add_chain();
    let strict = crate::normalize::explore_algebraic_alternatives_owned(
        chain.clone(),
        crate::request::DeterministicBudgets::governed(),
        StrictF32NumericalContract::governed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let AlgebraicExplorationParts { assessments, .. } = strict.into_parts();
    let binding = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&binding)).unwrap();
    let target = verified.for_target(verified.target_profiles()[0]).unwrap();
    let mut writer = ExplainWriter::new(&target).unwrap();
    let root = test_root(&mut writer);
    record_algebraic_exploration(&mut writer, root, &assessments, None, &[]).unwrap();
    let alternative = writer
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    writer
        .note_selection(alternative, SelectionOutcome::Selected, None)
        .unwrap();
    let strict = writer
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
        .render();
    assert!(strict.contains("rewrite.semantic-applicable:proven"));
    assert!(
        strict.contains("rewrite.numerically-legal:disproved:numerical.reassociation-forbidden")
    );
    assert!(strict.contains("rewrite-rule:identity=ordered-reassociate-add-f32.v1"));
    assert!(strict.contains("rewrite-revision:count=1"));

    let mut budgets = crate::request::DeterministicBudgets::governed();
    budgets.normalization_rewrites = 0;
    let stopped = crate::normalize::explore_algebraic_alternatives_owned(
        chain,
        budgets,
        StrictF32NumericalContract::governed_relaxed(),
        AlgebraicRuleConfiguration::all(),
    )
    .unwrap();
    let AlgebraicExplorationParts {
        assessments,
        budget_stop,
        ..
    } = stopped.into_parts();
    let mut writer = ExplainWriter::new(&target).unwrap();
    let root = test_root(&mut writer);
    record_algebraic_exploration(&mut writer, root, &assessments, budget_stop, &[]).unwrap();
    let alternative = writer
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    writer
        .note_selection(alternative, SelectionOutcome::Selected, None)
        .unwrap();
    let stopped = writer
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
        .render();
    assert!(stopped.contains("budget-stop:normalization-rewrites:0:1"));
}

#[test]
fn intrinsic_physical_failures_are_invalid_output_not_empty_frontiers() {
    let error = CompileError::from(PhysicalError::Intrinsic {
        rule: "forged",
        region: RegionId::new(0),
    });
    assert!(matches!(
        error,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Intrinsic { .. }
        ))
    ));
}

struct UnregisteredOpaqueProvider {
    identity: tiler_ir::semantic::ProviderIdentity,
    call: crate::call_registry::OpaqueCallIdentity,
    bindings: Vec<(&'static str, TensorRole)>,
}

impl PhysicalImplementationProvider for UnregisteredOpaqueProvider {
    fn provenance(
        &self,
    ) -> Result<
        crate::frontier::PhysicalProviderProvenance,
        crate::frontier::PhysicalProviderProvenanceError,
    > {
        crate::frontier::PhysicalProviderProvenance::new(self.identity.clone())
    }

    fn propose(
        &self,
        context: &crate::frontier::ImplementationContext<'_>,
    ) -> crate::frontier::ProviderOffer {
        crate::frontier::ProviderOffer::proposing(vec![
            crate::frontier::ImplementationProposal::new(
                crate::frontier::ProposalBody::OpaqueCall(Box::new(
                    crate::call_registry::OpaqueCallProposal::new(self.call, self.bindings.clone())
                        .expect("fixture proposal is exactly reportable"),
                )),
                crate::frontier::TargetApplicability::for_targets([context
                    .request()
                    .target_profile()
                    .profile_key()
                    .clone()]),
                crate::frontier::PhysicalCostEstimate::structural(1, 2, 0),
            ),
        ])
    }
}

fn mixed_frontier_trace(
    provider_revision: u32,
    call_revision: u32,
    reverse_providers: bool,
    reverse_bindings: bool,
) -> VerifiedExplainTrace {
    let semantic = semantic(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let subject = FrontierRegionSubject::new(
        "fused",
        request.serial_sum().members.all(),
        crate::physical::RegionWrite::ProgramOutput,
    );
    let governed = GovernedPhysicalProvider;
    let opaque = UnregisteredOpaqueProvider {
        identity: tiler_ir::semantic::ProviderIdentity::new(
            "tiler.test.physical",
            "opaque",
            provider_revision,
        )
        .unwrap(),
        call: crate::call_registry::OpaqueCallIdentity::new("call-owner", "mystery", call_revision)
            .unwrap(),
        bindings: if reverse_bindings {
            vec![
                ("output", TensorRole::Output),
                (
                    "input",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
            ]
        } else {
            vec![
                (
                    "input",
                    TensorRole::Input {
                        ordinal: InputOrdinal::FIRST,
                    },
                ),
                ("output", TensorRole::Output),
            ]
        },
    };
    let providers: Vec<&dyn PhysicalImplementationProvider> = if reverse_providers {
        vec![&opaque, &governed]
    } else {
        vec![&governed, &opaque]
    };
    let frontier = enumerate_frontier(
        &request,
        &subject,
        &providers,
        &crate::call_registry::OpaqueCallRegistry::new(),
    )
    .unwrap();
    assert_eq!(frontier.admitted().len(), 1);
    assert_eq!(frontier.rejections().len(), 1);

    let mut explain = ExplainWriter::new(&request).unwrap();
    let root = test_root(&mut explain);
    let cause = record_frontier(&mut explain, "region:fused", "fused", &frontier, root).unwrap();
    let alternative = explain
        .subject(SubjectKind::Alternative, "alternative:test")
        .unwrap();
    explain
        .note_selection(
            alternative,
            SelectionOutcome::Selected,
            Some(TerminalCause::from_record(cause)),
        )
        .unwrap();
    explain
        .finish_success(&["alternative:test"], "alternative:test")
        .unwrap()
}

#[test]
fn mixed_frontier_records_exact_opaque_call_rejection_detail() {
    let trace = mixed_frontier_trace(7, 3, false, false);
    let rejection = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "opaque-call.registration.v1")
        .expect("one unregistered-call detail");
    assert!(matches!(
        rejection.event(),
        ExplainEvent::Check {
            stage: ExplainStage::CapabilityResolution,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        } if assessment.predicate().as_str() == "opaque-call.registered"
            && assessment.reason().is_some_and(|reason| reason.as_str() == "opaque-call.unregistered")
    ));
    assert_eq!(
        rejection.event().disposition(),
        ExplainDisposition::RejectedIntrinsic
    );
    let subjects = rejection
        .subjects()
        .iter()
        .map(|subject| (subject.kind(), subject.key().as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        subjects,
        [
            (
                SubjectKind::OpaqueCall,
                "call-owner/mystery@3[input=input#0,output=output]",
            ),
            (SubjectKind::Provider, "tiler.test.physical::opaque@7"),
        ]
    );
    assert!(
        trace
            .records()
            .iter()
            .all(|record| !matches!(record.event(), ExplainEvent::CostAssessment { .. })),
        "a local rejection is never cost evidence"
    );
    let rendered = trace.render();
    assert!(rendered.starts_with("tiler-explain-v7 "));
    assert!(rendered.contains("opaque-call:call-owner/mystery@3[input=input#0,output=output]"));
    assert!(rendered.contains("provider:tiler.test.physical::opaque@7"));
    assert!(rendered.contains("admitted-count:count=1"));
    assert!(rendered.contains("rejected-count:count=1"));
}

#[test]
fn opaque_call_trace_identity_is_order_independent_and_identity_sensitive() {
    let forward = mixed_frontier_trace(7, 3, false, false);
    let reversed = mixed_frontier_trace(7, 3, true, false);
    assert_eq!(forward.identity(), reversed.identity());
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(7, 4, false, false).identity()
    );
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(8, 3, false, false).identity()
    );
    assert_ne!(
        forward.identity(),
        mixed_frontier_trace(7, 3, false, true).identity(),
        "ordered named bindings were absent from explain identity"
    );
}

// ---------------------------------------------------------------------------
// Opaque calls on the compile path
// ---------------------------------------------------------------------------

/// A provider offering one opaque call for the whole-program region only.
///
/// It gates on the region subject rather than proposing everywhere, so a
/// compilation using it differs from the governed one by exactly one
/// implementation: the whole-program region is one the governed provider already
/// implements, which makes the call an *alternative* to a checked scheduled body
/// rather than the only implementation of a region nothing else covers.
struct WholeProgramCallProvider {
    call: crate::call_registry::OpaqueCallIdentity,
}

impl WholeProgramCallProvider {
    /// The call's ABI parameters bound to this region's tensor roles.
    ///
    /// Stated by the provider and never inferred: the ABI says a parameter is
    /// read or written and never which tensor it reads, so the claim is the
    /// provider's and the frontier checks it against the declaration.
    fn bindings() -> Vec<(&'static str, TensorRole)> {
        vec![
            (
                "x",
                TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
            ),
            ("y", TensorRole::Output),
        ]
    }
}

impl PhysicalImplementationProvider for WholeProgramCallProvider {
    fn provenance(
        &self,
    ) -> Result<
        crate::frontier::PhysicalProviderProvenance,
        crate::frontier::PhysicalProviderProvenanceError,
    > {
        crate::frontier::PhysicalProviderProvenance::new(
            tiler_ir::semantic::ProviderIdentity::new(
                "tiler.test.physical",
                "whole-program-call",
                1,
            )
            .expect("the fixture provider identity is valid"),
        )
    }

    fn propose(
        &self,
        context: &crate::frontier::ImplementationContext<'_>,
    ) -> crate::frontier::ProviderOffer {
        if context.subject().role() != "whole-program" {
            return crate::frontier::ProviderOffer::default();
        }
        crate::frontier::ProviderOffer::proposing(vec![
            crate::frontier::ImplementationProposal::new(
                crate::frontier::ProposalBody::OpaqueCall(Box::new(
                    crate::call_registry::OpaqueCallProposal::new(self.call, Self::bindings())
                        .expect("fixture proposal is exactly reportable"),
                )),
                crate::frontier::TargetApplicability::for_targets([context
                    .request()
                    .target_profile()
                    .profile_key()
                    .clone()]),
                crate::frontier::PhysicalCostEstimate::structural(1, 2, 0),
            ),
        ])
    }
}

/// The governed authorities plus one opaque-call provider, with or without the
/// declaration that provider's proposal names.
///
/// The two compositions differ in exactly one registration, which is what makes
/// either case evidence about the registry rather than about the provider.
fn opaque_call_authorities<'a>(
    governed: &'a GovernedPhysicalProvider,
    opaque: &'a WholeProgramCallProvider,
    register: bool,
) -> PhysicalAuthorities<'a> {
    let mut calls = crate::call_registry::OpaqueCallRegistry::new();
    if register {
        calls
            .register(
                opaque.call,
                crate::selection::opaque_call_declaration_fixture(
                    crate::effects::Aliasing::Distinct,
                ),
            )
            .expect("the fixture registers one call");
    }
    PhysicalAuthorities::composed(vec![governed, opaque], calls)
}

/// The fixture call identity both compile-path cases name.
fn fixture_call_identity() -> crate::call_registry::OpaqueCallIdentity {
    crate::call_registry::OpaqueCallIdentity::new("test-owner", "whole-program-call", 1)
        .expect("the fixture call identity is valid")
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
fn frontier_record<'trace>(
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
fn region_subject_key(trace: &VerifiedExplainTrace, role: &str) -> Option<String> {
    Some(
        frontier_record(trace, role)?.subjects()[0]
            .key()
            .as_str()
            .to_owned(),
    )
}

/// The implementations the frontier admitted for one region role, as the compile
/// path's own explain trace reports them.
fn admitted_count(trace: &VerifiedExplainTrace, role: &str) -> Option<u64> {
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

/// A registered opaque call reaches the compile path and is admitted there.
///
/// Admission is the property; the compilation's *refusal* is how a caller of
/// `compile` observes it. Lowering an opaque call is not implemented, so a
/// retained plan that selects one is refused by name at program assembly — and
/// that refusal is reachable only through an admitted opaque body in a retained
/// plan. Before this wiring, no registry any caller could populate reached
/// `enumerate_frontier` at all, so every test of the admission path was a test
/// of an authority nothing could drive.
///
/// The control is in the case: the identical compilation with the registration
/// removed compiles, and its whole-program frontier admits one implementation
/// instead of two. The refusal is therefore caused by the registration and not
/// by the provider merely being installed.
#[test]
fn a_registered_opaque_call_is_admitted_through_the_compile_path() {
    let semantic = semantic(false);
    let governed = GovernedPhysicalProvider;
    let opaque = WholeProgramCallProvider {
        call: fixture_call_identity(),
    };

    // Matched rather than unwrapped: the success value is a whole compilation
    // product, and printing it would bury the one fact a reader of a failure
    // here needs — that the call never reached a plan.
    let Err(refusal) = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, true),
    ) else {
        panic!("an admitted opaque call has no lowering; the compilation succeeded");
    };
    let CompileError::Explained { source, explain } = refusal else {
        panic!("a refusal after the trace boundary retains its trace");
    };
    assert!(
        matches!(
            *source,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "unlowerable-opaque-body"
                }
            ))
        ),
        "the registered call reached a retained plan: {source:?}",
    );
    assert_eq!(
        admitted_count(&explain, "whole-program"),
        Some(2),
        "the registered call was admitted beside the governed scheduled body",
    );

    let unregistered = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, false),
    )
    .expect("the same compilation without the registration has no opaque plan");
    assert_eq!(
        admitted_count(&unregistered.targets[0].explain, "whole-program"),
        Some(1),
        "removing the registration removes the admission, not merely the plan",
    );
}

/// A call a proposal names and no registry holds is refused as unregistered.
///
/// The refusal belongs to the provider rather than to the target: nothing about
/// this profile made the call infeasible, so the compilation keeps its governed
/// alternatives and records the exact proposal it could not resolve.
#[test]
fn an_unregistered_opaque_call_named_on_the_compile_path_is_refused_by_name() {
    let semantic = semantic(false);
    let governed = GovernedPhysicalProvider;
    let opaque = WholeProgramCallProvider {
        call: fixture_call_identity(),
    };

    let product = compile_configured(
        CompilationRequest::governed(&semantic),
        AlgebraicRuleConfiguration::all(),
        &opaque_call_authorities(&governed, &opaque, false),
    )
    .expect("an unregistered call is a provider fault, not a target refusal");
    let trace = &product.targets[0].explain;
    let rejection = trace
        .records()
        .iter()
        .find(|record| record.rule().key().as_str() == "opaque-call.registration.v1")
        .expect("one unregistered-call refusal");
    assert!(matches!(
        rejection.event(),
        ExplainEvent::Check {
            stage: ExplainStage::CapabilityResolution,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        } if assessment.predicate().as_str() == "opaque-call.registered"
            && assessment
                .reason()
                .is_some_and(|reason| reason.as_str() == "opaque-call.unregistered")
    ));
    assert_eq!(
        rejection
            .subjects()
            .iter()
            .map(|subject| (subject.kind(), subject.key().as_str()))
            .collect::<Vec<_>>(),
        [
            (
                SubjectKind::OpaqueCall,
                "test-owner/whole-program-call@1[x=input#0,y=output]",
            ),
            (
                SubjectKind::Provider,
                "tiler.test.physical::whole-program-call@1",
            ),
        ],
        "the exact proposal that could not be resolved is retained",
    );
    assert_eq!(
        product.targets[0].portfolio.alternatives.len(),
        compile(CompilationRequest::governed(&semantic))
            .expect("the governed compilation")
            .targets[0]
            .portfolio
            .alternatives
            .len(),
        "an unregistered proposal removes no governed alternative",
    );
}

// ---------------------------------------------------------------------------
// The multi-pass split: enumerated on the frontier, assembled into a program
// ---------------------------------------------------------------------------
//
// **Why these drive the authorities directly, and what now also reaches
// `compile`.** The split consumes reassociation. Under `governed_relaxed` — for
// a long time the only registered contract permitting it — contraction is
// permitted too, and for the recognized serial-sum program, whose members mix
// multiply and add, `derive_fusion_legality` reports `unrealized-contraction`
// for every multi-member candidate, so no legal cover survives and the whole
// compile has no complete plan. That property is unchanged and still pinned by
// `fusion_legality::tests::a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence`.
//
// `admit-a-reassociating-contract-without-contraction` closed the gap from the
// contract side rather than the proof side:
// `StrictF32NumericalContract::governed_reassociating` permits reassociation and
// forbids contraction, so the prologue's mixed region discharges its contraction
// obligation under the contract's own normative guarantee and the materialized
// cover survives. `the_reassociating_contract_reaches_the_split_through_compile`
// below is the end-to-end half; these keep exercising the exact authorities at
// the relaxed contract, where the split is still enumerable and assemblable
// without being reachable.

/// Builds the recognized serial-sum program and its reassociation-permitting
/// verified request.
fn split_request(shape: Shape) -> (SemanticProgram, crate::request::VerifiedTargetRequest) {
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
fn reduction_frontier(
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
    )
    .expect("the governed provider emits well-formed proposals")
}

/// **The ticket's core claim:** the split is retained *beside* the serial
/// reduction, not in place of it.
///
/// Both are admitted for the same subject, with distinct identities, and their
/// boundary contracts are identical — which is what makes the split composable
/// exactly where the serial reduction is, and is why
/// `selection::reconcile_boundaries` needs no widening: the partial tensor is
/// internal to the subprogram and never reaches a cover edge.
#[test]
fn the_frontier_retains_the_split_beside_the_serial_reduction() {
    // Four contributors, which `governed_partition` splits as two partitions of
    // two. Four is also the governed profile's declared grid-axis guarantee, so
    // this is the largest splittable domain the bounded target admits.
    let (_, request) = split_request(Shape::from_dims([1, 4]));
    let frontier = reduction_frontier(&request);

    let kinds: Vec<_> = frontier
        .admitted()
        .iter()
        .map(|admitted| admitted.provenance().kind())
        .collect();
    assert_eq!(frontier.admitted().len(), 2, "{kinds:?}");
    assert!(kinds.contains(&PhysicalProposalKind::ScheduledKernel));
    assert!(kinds.contains(&PhysicalProposalKind::KernelSubprogram));
    assert_ne!(
        frontier.admitted()[0].identity(),
        frontier.admitted()[1].identity(),
        "the two alternatives share one identity, so one shadows the other"
    );
    // The single-workgroup tree is proposed for the same subject and refused by
    // the *target*, not withheld by the strategy: the bounded prototype profile
    // guarantees zero threadgroup memory, so the tree's eight staged bytes are a
    // hard-feasibility rejection naming the exact axis and both quantities. That
    // is the shape a resource refusal must have — never an arbitrary cost — and
    // it leaves the split and the serial alternative untouched.
    assert!(
        matches!(
            frontier.rejections(),
            [crate::frontier::FrontierRejection::Infeasible {
                axis: "local-memory-bytes",
                required: 8,
                available: 0,
                ..
            }]
        ),
        "the split request's rejections are not the tree's single resource refusal: {:?}",
        frontier.rejections()
    );

    let split = frontier
        .admitted()
        .iter()
        .find(|admitted| admitted.provenance().kind() == PhysicalProposalKind::KernelSubprogram)
        .expect("the split alternative");
    let serial = frontier
        .admitted()
        .iter()
        .find(|admitted| admitted.provenance().kind() == PhysicalProposalKind::ScheduledKernel)
        .expect("the serial alternative");
    assert_eq!(split.boundary(), serial.boundary());
    assert_eq!(split.semantic_members(), serial.semantic_members());
    // Two dispatches for one occurrence: the fact the scheduled-kernel body
    // cannot express and the subprogram exists for.
    assert_eq!(split.scheduled_stages().map(<[_]>::len), Some(2));
    assert_eq!(serial.scheduled_stages().map(<[_]>::len), Some(1));
    // The split's cost is worse on every structural dimension, so it can never
    // win by pruning. Preference is `calibrate-and-activate-parallel-reduction-selection`'s.
    assert!(split.cost().dispatch_count() > serial.cost().dispatch_count());
    assert!(split.cost().temporary_bytes() > serial.cost().temporary_bytes());
}

/// A prime contributor extent retains only the serial alternative, explainably.
///
/// The ragged split stays out of scope, so this is the boundary where that
/// exclusion becomes observable: three contributors admit no exact partition
/// whose parts each fold more than one value. The frontier withholds the split
/// and names the extent that admitted none, rather than proposing a ragged tail
/// it cannot lower or leaving the absence unexplained.
#[test]
fn a_prime_contributor_extent_declines_the_split_with_its_extent() {
    let (_, request) = split_request(Shape::from_dims([1, 3]));
    let frontier = reduction_frontier(&request);
    assert_eq!(frontier.admitted().len(), 1);
    assert_eq!(
        frontier.admitted()[0].provenance().kind(),
        PhysicalProposalKind::ScheduledKernel
    );
    assert!(
        frontier.rejections().iter().any(|rejection| matches!(
            rejection,
            crate::frontier::FrontierRejection::StrategyDeclined {
                strategy: "tiler.reduction.multi-pass-split",
                cause: crate::frontier::StrategyDeclineCause::NoAdmissibleShape { extent: 3, .. },
                ..
            }
        )),
        "the prime extent's missing split is unexplained: {:?}",
        frontier.rejections()
    );
}

/// A contract forbidding reassociation withholds the split by naming the
/// dimension it consumes.
///
/// The decline is decided from the contract before any region is built. Building
/// one and letting the schedule verifier refuse it would report a caller's
/// numerical choice as malformed compiler output — a `FrontierError`, which
/// fails the whole enumeration closed rather than retaining the serial plan.
#[test]
fn a_reassociation_forbidding_contract_declines_the_split_by_dimension() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([1, 4]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let frontier = reduction_frontier(&request);
    assert_eq!(frontier.admitted().len(), 1);
    assert!(
        frontier.rejections().iter().any(|rejection| matches!(
            rejection,
            crate::frontier::FrontierRejection::StrategyDeclined {
                cause: crate::frontier::StrategyDeclineCause::NumericalPermissionRefused {
                    dimension: "numerics.reassociation",
                },
                ..
            }
        )),
        "a strict contract withheld the split without naming the permission: {:?}",
        frontier.rejections()
    );
}

// ---------------------------------------------------------------------------
// The single-workgroup tree: enumerated beside serial, and executed
// ---------------------------------------------------------------------------
//
// **Why the positive path uses a widened test profile.** The bounded prototype
// baseline declares `local-memory-bytes` as zero and declares nothing at all
// about synchronization, so it refuses every cooperative region — twice over,
// and both refusals are driven below as required evidence. Raising the
// baseline's own rows would be a capability claim this build has no authority
// for; `TargetProfile::workgroup_tree_target_for_test` says so at length and
// names who owns the real declaration.

/// Builds the reassociating request for one shape against a chosen profile.
fn tree_request(
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
fn tree_target() -> TargetProfile {
    TargetProfile::workgroup_tree_target_for_test(
        256,
        1_024,
        Some(crate::target::SynchronizationSupport::Realized),
    )
}

/// **The ticket's core claim:** the single-workgroup tree is retained *beside*
/// the serial reduction and the multi-pass split, not in place of either.
///
/// All three implement the same occurrences with the same boundary contract, so
/// the planner sees three legal alternatives for one subject and selection is
/// left to decide between them on evidence this slice deliberately does not
/// supply.
#[test]
fn the_frontier_retains_the_workgroup_tree_beside_serial_and_the_split() {
    let (_, request) = tree_request(Shape::from_dims([1, 8]), tree_target());
    let frontier = reduction_frontier(&request);
    assert!(
        frontier.rejections().is_empty(),
        "a profile that realizes the handoff still refused something: {:?}",
        frontier.rejections()
    );
    assert_eq!(frontier.admitted().len(), 3);

    let scheduled: Vec<_> = frontier
        .admitted()
        .iter()
        .filter(|admitted| admitted.provenance().kind() == PhysicalProposalKind::ScheduledKernel)
        .collect();
    assert_eq!(scheduled.len(), 2, "the serial region and the tree");
    let subprograms = frontier
        .admitted()
        .iter()
        .filter(|admitted| admitted.provenance().kind() == PhysicalProposalKind::KernelSubprogram)
        .count();
    assert_eq!(subprograms, 1, "the multi-pass split");
    // Distinct identities, or one alternative shadows another and the portfolio
    // silently holds two.
    let mut identities: Vec<_> = frontier
        .admitted()
        .iter()
        .map(crate::frontier::AdmittedImplementation::identity)
        .collect();
    let total = identities.len();
    identities.sort_unstable();
    identities.dedup();
    assert_eq!(identities.len(), total);
    // The same boundary contract and the same claimed occurrences, which is what
    // makes the tree composable exactly where the serial reduction is.
    for admitted in &scheduled {
        assert_eq!(admitted.boundary(), scheduled[0].boundary());
        assert_eq!(admitted.semantic_members(), scheduled[0].semantic_members());
    }
    // One dispatch each, and the tree launches strictly more threads: under the
    // structural model it can never win by pruning, which is exactly the
    // cost-free legality this slice is limited to.
    let tree = scheduled
        .iter()
        .find(|admitted| admitted.cost().launched_threads() > 1)
        .expect("the tree launches one invocation per participant per output");
    let serial = scheduled
        .iter()
        .find(|admitted| admitted.cost().launched_threads() == 1)
        .expect("the serial reduction launches one invocation per output");
    assert_eq!(tree.cost().dispatch_count(), serial.cost().dispatch_count());
    assert!(tree.cost().launched_threads() > serial.cost().launched_threads());
    assert_eq!(
        tree.cost().temporary_bytes(),
        serial.cost().temporary_bytes()
    );
}

/// A cooperative region's assembled program declares the launch it needs.
///
/// **The regression this pins.** The host ABI used to declare one literal `1`
/// as every stage's workgroup width, and to reuse whichever element count
/// happened to equal a stage's work items as its grid. Both hold for a region
/// that runs one independent invocation per result element, and both are false
/// for a single-workgroup tree: it launches one invocation per participant
/// inside one workgroup, so its work items and its width are the participant
/// count while its output count is one. `verify_stage_abi` and the shared
/// kernel-program builder each prove the declared launch against the schedule,
/// so the effect was the whole compilation failing as invalid compiler output —
/// `ThreadsPerWorkgroupDisagreement { expected: 2, actual: 1 }` on the first
/// tree to reach a kernel program.
///
/// **Watched failing.** Restoring either half — a literal `1` width, or
/// `abi.output_elements` as the grid — makes this test fail on the tree's stage
/// while the serial reduction beside it still passes, which is what distinguishes
/// a launch derived from the schedule from one that agrees by coincidence.
#[test]
fn a_cooperative_region_declares_its_own_launch() {
    let (semantic, request) = tree_request(Shape::from_dims([1, 8]), tree_target());
    let (tree, members) =
        crate::physical::single_workgroup_tree_region(&request, request.sole_output())
            .expect("the tree is available");
    let tree =
        crate::physical::verify_schedule(tree, members, &request).expect("the tree verifies");
    // The tree replaces the reduction of the materialized pair; its prologue is
    // the ordinary pointwise stage, which is what makes the two stages' launches
    // differ in both quantities inside one program.
    let serial = crate::physical::build_scheduled_regions(&request).expect("the serial pair");
    let [pointwise, _] = serial.as_slice() else {
        panic!("the materialized strategy is a pointwise stage and a reduction");
    };
    let scheduled = vec![pointwise.clone(), tree];
    let program = crate::program::build_kernel_program(
        &semantic,
        &request,
        &materialized_assembly(&request, &scheduled),
    )
    .expect("the tree's program assembles");
    let expressions = program.core().abi_expressions();
    let literal = |position: u32| match expressions
        .get(usize::try_from(position).expect("an arena position fits a usize"))
    {
        Some(tiler_ir::program::abi::ExprNode::Root(
            tiler_ir::program::abi::AbiRoot::UnsignedLiteral(value),
        )) => *value,
        other => panic!("a launch quantity is not a declared literal: {other:?}"),
    };
    let stages: Vec<_> = program.core().stages().collect();
    assert_eq!(stages.len(), scheduled.len());
    let cooperative = scheduled
        .iter()
        .filter(|region| region.region().schedule.threads_per_workgroup > 1)
        .count();
    assert_eq!(
        cooperative, 1,
        "exactly one stage must be cooperative, or the check is vacuous",
    );
    for (stage, region) in stages.iter().zip(&scheduled) {
        let schedule = &region.region().schedule;
        let launch = stage.launch();
        assert_eq!(
            (
                literal(launch.grid_threads),
                literal(launch.threads_per_workgroup),
            ),
            (
                schedule.work_items,
                u64::from(schedule.threads_per_workgroup)
            ),
        );
    }
}

/// Every way the tree can fail rejects before admission with its own reason.
///
/// Four causes, four distinct outcomes, and the point of driving them together
/// is that none of them is a cost and none of them is the same answer as another:
/// a withheld permission is decided from the contract before a region exists, a
/// resource refusal names the axis and both quantities, a declared refusal names
/// the profile that refused, and silence names no profile at all.
#[test]
fn each_way_the_tree_can_fail_rejects_before_admission_with_its_own_reason() {
    // The control: the same shape against a realizing profile admits the tree,
    // so every refusal below is earned by the change that produced it.
    let (_, admitting) = tree_request(Shape::from_dims([1, 8]), tree_target());
    assert_eq!(reduction_frontier(&admitting).admitted().len(), 3);

    // A withheld numerical permission: decided from the contract, before any
    // region is built, and naming the dimension the tree consumes.
    let strict = semantic_case_with_axis(
        Shape::from_dims([1, 8]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let mut request = CompilationRequest::governed(&strict);
    request.target_profiles = vec![tree_target()];
    let verified = verify_planned_request(request).expect("the strict contract is admitted");
    let verified = verified.for_target(verified.target_profiles()[0]).unwrap();
    assert!(
        reduction_frontier(&verified)
            .rejections()
            .iter()
            .any(|rejection| matches!(
                rejection,
                crate::frontier::FrontierRejection::StrategyDeclined {
                    strategy: "tiler.reduction.single-workgroup-tree",
                    cause: crate::frontier::StrategyDeclineCause::NumericalPermissionRefused {
                        dimension: "numerics.reassociation",
                    },
                    ..
                }
            )),
        "a strict contract withheld the tree without naming the permission"
    );

    // Insufficient workgroup resources: a hard bound, with the exact axis and
    // both quantities, never an infinite cost.
    let (_, starved) = tree_request(
        Shape::from_dims([1, 8]),
        TargetProfile::workgroup_tree_target_for_test(
            8,
            1_024,
            Some(crate::target::SynchronizationSupport::Realized),
        ),
    );
    assert!(
        matches!(
            reduction_frontier(&starved).rejections(),
            [crate::frontier::FrontierRejection::Infeasible {
                axis: "local-memory-bytes",
                required: 16,
                available: 8,
                ..
            }]
        ),
        "a profile too small for the staging did not refuse it by bound: {:?}",
        reduction_frontier(&starved).rejections()
    );

    // A declared refusal: the profile was asked and said no, so the rejection
    // carries the whole subject and the authority behind it.
    let (_, refused) = tree_request(
        Shape::from_dims([1, 8]),
        TargetProfile::workgroup_tree_target_for_test(
            256,
            1_024,
            Some(crate::target::SynchronizationSupport::Unrealizable),
        ),
    );
    let rejections = reduction_frontier(&refused).rejections().to_vec();
    let [crate::frontier::FrontierRejection::Unsynchronizable { cause, .. }] =
        rejections.as_slice()
    else {
        panic!("a declared refusal did not reject the tree by subject: {rejections:?}")
    };
    assert_eq!(
        cause.subject().kind,
        tiler_ir::schedule::SynchronizationKind::ControlBarrier
    );
    assert_eq!(
        cause.subject().execution_scope,
        tiler_ir::schedule::SynchronizationScope::Workgroup
    );
    assert!(cause.subject().fenced_spaces.workgroup);
    assert!(!cause.subject().fenced_spaces.device);

    // Missing authority: the profile was never asked, so the rejection carries
    // the subject and no profile. Distinguishing this from the refusal above is
    // the whole reason the two rejections are separate variants.
    let (_, unasked) = tree_request(
        Shape::from_dims([1, 8]),
        TargetProfile::workgroup_tree_target_for_test(256, 1_024, None),
    );
    let rejections = reduction_frontier(&unasked).rejections().to_vec();
    let [crate::frontier::FrontierRejection::SynchronizationUndeclared { subject, .. }] =
        rejections.as_slice()
    else {
        panic!("an unasked profile did not reject the tree as undeclared: {rejections:?}")
    };
    assert_eq!(*subject, cause.subject());
}

/// A divergent tile cannot reach the frontier at all.
///
/// The fourth required rejection, and it is a *schedule* refusal rather than a
/// target one: a synchronization point in a phase some participants skip is
/// undefined execution, so the schedule verifier refuses it and no proposal is
/// ever assessed. Driven against the verifier directly, because the strategy
/// constructor cannot emit a divergent tile — which is the point.
#[test]
fn a_divergent_tile_is_refused_by_the_schedule_before_any_target_is_consulted() {
    let (_, request) = tree_request(Shape::from_dims([1, 8]), tree_target());
    let (region, members) =
        crate::physical::single_workgroup_tree_region(&request, request.sole_output())
            .expect("a reassociating eight-contributor request admits the tree");
    // The control: the tile the strategy actually emits verifies.
    assert!(crate::physical::verify_schedule(region.clone(), members.clone(), &request).is_ok());

    let mut divergent = region;
    let tiler_ir::schedule::ReductionTopology::CooperativeWorkgroup { tile, .. } =
        &mut divergent.schedule.reduction
    else {
        panic!("the tree region carries a cooperative topology")
    };
    // One participant skips the consuming phase, which is exactly the divergence
    // the per-phase participation field exists to make statable.
    tile.phases[1].participation = tiler_ir::schedule::ParticipantRange { first: 0, count: 3 };
    assert_eq!(
        crate::physical::verify_schedule(divergent, members, &request),
        Err(crate::physical::PhysicalError::Intrinsic {
            rule: "cooperative-phase-participation",
            region: RegionId::new(4),
        })
    );
}

/// The tree's subject binding refuses a region that does not realize the
/// request.
///
/// The binding is what stops a provider implementing a *different* reduction and
/// having it admitted because the schedule verifier — which sees only the region
/// — cannot notice. Each perturbation changes exactly one fact the binding
/// re-derives from the request, so a rule that stopped re-deriving it would let
/// one of these through.
#[test]
fn the_tree_subject_binding_refuses_a_region_that_does_not_realize_the_request() {
    let (_, request) = tree_request(Shape::from_dims([1, 8]), tree_target());
    let (region, members) =
        crate::physical::single_workgroup_tree_region(&request, request.sole_output())
            .expect("a reassociating eight-contributor request admits the tree");
    // The control: unperturbed, it binds.
    assert!(crate::physical::verify_schedule(region.clone(), members.clone(), &request).is_ok());

    // A region ordinal the tree does not own. Two strategies sharing one ordinal
    // would make the program's region correlation ambiguous.
    let mut forged = region.clone();
    forged.index.id = RegionId::new(1);
    assert!(matches!(
        crate::physical::verify_schedule(forged, members.clone(), &request),
        Err(crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            ..
        })
    ));

    // Claiming the prologue's occurrences as well as the reduction's, which
    // would double-cover the graph.
    let forged_members = request.serial_sum().members.all();
    assert!(matches!(
        crate::physical::verify_schedule(region.clone(), forged_members, &request),
        Err(crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            ..
        })
    ));

    // An iteration shape that is not the output shape carrying this split's
    // participant axis, so the region's invocations no longer stand in
    // one-to-one correspondence with (output, participant) pairs.
    let mut forged = region;
    forged.index.iteration_shape = Shape::from_dims([1, 2]);
    assert!(matches!(
        crate::physical::verify_schedule(forged, members, &request),
        Err(crate::physical::PhysicalError::Intrinsic { .. })
    ));
}

/// An input whose fold depends on *where* the partition boundaries fall.
///
/// After the recognized prologue (`x * 2 + 1`) these are `[2V, 1, -2V, 1, …]`
/// with `V` far above the unit ulp, so a partition that spans the cancelling
/// pair absorbs the ones beside it and a partition that stops between them does
/// not. Cancellation alone is not enough — a strictly alternating input sums to
/// the same value under every balanced split, which would let an agreement be
/// luck rather than evidence.
const REGROUPING_SENSITIVE_INPUT: [f32; 8] = [5.0e19, 0.0, -5.0e19, 0.0, 0.0, 0.0, 0.0, 0.0];

/// The neighbouring split really does compute something else.
///
/// The guard on the test below: it asserts an executed kernel equals its
/// declared order's oracle, and that assertion is only evidence if some *other*
/// order would have disagreed. This pins that, so an input chosen to make the
/// comparison vacuous fails here rather than silently weakening the conformance
/// claim next door.
#[test]
fn the_declared_split_is_what_the_agreement_is_evidence_about() {
    let scaled: Vec<f32> = REGROUPING_SENSITIVE_INPUT
        .iter()
        .map(|value| value * 2.0_f32 + 1.0_f32)
        .collect();
    let tensor = f32_tensor(Shape::from_dims([1, 8]), &scaled);
    let declared = tiler_reference::strict_partitioned_sum(&tensor, &[Axis::new(1)], 4, 2)
        .expect("the declared split is exact");
    let neighbouring = tiler_reference::strict_partitioned_sum(&tensor, &[Axis::new(1)], 2, 4)
        .expect("the neighbouring split is exact");
    assert_ne!(
        tensor_bits(&declared),
        tensor_bits(&neighbouring),
        "the conformance input cannot tell two splits apart"
    );
}

/// The tree's executed result is the reference's, at every extent it admits and
/// at every extent it declines.
///
/// The kernel is *run* rather than inspected: `KirMachine` advances every lane of
/// a workgroup to the barrier before any lane crosses it, so a body that read a
/// staged slot before its writer produced it would read `NaN` and fail here
/// rather than pass by accident.
///
/// The oracle is `strict_partitioned_sum` at the region's *own* declared split —
/// a second exact oracle, not a relaxation of the first. A contract permitting
/// reassociation admits a set of results, so no oracle can answer "the" value for
/// it; what a plan is checked against is the one order it selected.
#[test]
fn the_tree_matches_the_reference_at_its_declared_order_for_every_extent() {
    for (extent, participants, contributors_per_partition) in [(8_u64, 4_u64, 2_u64), (6, 3, 2)] {
        let values = REGROUPING_SENSITIVE_INPUT;
        let extent_usize = usize::try_from(extent).unwrap();
        let (_, request) = tree_request(Shape::from_dims([1, extent]), tree_target());
        let (region, members) =
            crate::physical::single_workgroup_tree_region(&request, request.sole_output())
                .expect("a reassociating request admits the tree at this extent");
        let tiler_ir::schedule::ReductionTopology::CooperativeWorkgroup { partition, .. } =
            &region.schedule.reduction
        else {
            panic!("the tree region carries a cooperative topology")
        };
        assert_eq!(partition.partitions, participants, "extent {extent}");
        assert_eq!(
            partition.contributors_per_partition, contributors_per_partition,
            "extent {extent}"
        );
        let partition = *partition;
        let verified = crate::physical::verify_schedule(region, members, &request)
            .expect("the tree region verifies");
        let kernel = crate::physical::lower_structured_kernel(&verified)
            .expect("the tree region lowers to a verified kernel");

        // The prologue the recognized program applies before the fold, applied
        // here so the reference sees the same contributor values the kernel's
        // reduction reads.
        let scaled: Vec<f32> = values[..extent_usize]
            .iter()
            .map(|value| value * 2.0_f32 + 1.0_f32)
            .collect();
        let actual = interpret_fused(&kernel, &scaled);
        let expected = tiler_reference::strict_partitioned_sum(
            &f32_tensor(Shape::from_dims([1, extent]), &scaled),
            &[Axis::new(1)],
            partition.partitions,
            partition.contributors_per_partition,
        )
        .expect("the declared split is an exact oracle");
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            tensor_bits(&expected),
            "extent {extent} disagreed with its declared order"
        );
    }

    // One element and an empty domain admit no tree at all, and the decline
    // names the extent rather than leaving the absence unexplained. The serial
    // alternative carries both, including the empty domain's `+0.0` identity,
    // which the zero-extent precedent already proves and this does not restate.
    for extent in [1_u64, 0] {
        let (_, request) = tree_request(Shape::from_dims([1, extent]), tree_target());
        assert_eq!(
            crate::physical::single_workgroup_tree_region(&request, request.sole_output()).err(),
            Some(
                crate::physical::WorkgroupTreeUnavailable::NoAdmissibleParticipantCount {
                    contributors: extent,
                }
            ),
            "extent {extent} did not decline by naming its contributor count"
        );
        let frontier = reduction_frontier(&request);
        assert!(
            frontier.rejections().iter().any(|rejection| matches!(
                rejection,
                crate::frontier::FrontierRejection::StrategyDeclined {
                    strategy: "tiler.reduction.single-workgroup-tree",
                    cause: crate::frontier::StrategyDeclineCause::NoAdmissibleShape { .. },
                    ..
                }
            )),
            "extent {extent}'s missing tree is unexplained: {:?}",
            frontier.rejections()
        );
        // The serial alternative is still there, which is what makes the decline
        // a narrowing of the portfolio rather than a compilation failure.
        assert!(
            frontier
                .admitted()
                .iter()
                .any(|admitted| admitted.provenance().kind()
                    == PhysicalProposalKind::ScheduledKernel)
        );
    }

    // A prime extent is the tail case the exact-or-decline policy exists for:
    // seven contributors admit no balanced split, so the tree is withheld rather
    // than padded with identity elements or given a masked lane.
    let (_, prime) = tree_request(Shape::from_dims([1, 7]), tree_target());
    assert_eq!(
        crate::physical::single_workgroup_tree_region(&prime, prime.sole_output()).err(),
        Some(
            crate::physical::WorkgroupTreeUnavailable::NoAdmissibleParticipantCount {
                contributors: 7,
            }
        )
    );
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
fn materialized_assembly(
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
        vec![(subject.output_key.clone(), 1)],
    )
    .expect("the two-region assembly is well formed")
}

/// The split's three-stage assembly: the same two-region cover, with its
/// reduction realized by a partial pass and a combining pass.
///
/// The combining pass covers **no** occurrence — the partial pass already claims
/// the reduction the two of them realize — which whole-program verification
/// admits only because the split contract below is declared.
fn split_assembly(
    request: &crate::request::VerifiedTargetRequest,
    scheduled: &[crate::physical::VerifiedScheduledRegion],
) -> crate::program::CoverAssembly {
    let subject = request.serial_sum();
    let partial = scheduled[1].region().index.iteration_shape.clone();
    let partition = crate::physical::declared_partial_partition(scheduled[1].region())
        .expect("the partial pass declares its split");
    crate::program::CoverAssembly::stated(
        scheduled.to_vec(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (partial, ValueRole::Temporary),
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
            crate::program::AssemblyStage {
                coverage: Vec::new(),
                bindings: vec![
                    crate::program::AssemblyBinding::Internal(1),
                    crate::program::AssemblyBinding::Internal(2),
                ],
            },
        ],
        vec![crate::program::AssemblySplit {
            producer: 1,
            combiner: 2,
            partial: 1,
            result: 2,
            partition,
        }],
        vec![(subject.output_key.clone(), 2)],
    )
    .expect("the split assembly is well formed")
}

fn split_regions(
    request: &crate::request::VerifiedTargetRequest,
) -> Vec<crate::physical::VerifiedScheduledRegion> {
    let (raw, members) = crate::physical::pointwise_region(
        request,
        request.sole_output(),
        crate::physical::RegionWrite::Materialized,
    );
    let mut regions = vec![
        crate::physical::verify_schedule(raw, members, request).expect("the prologue verifies"),
    ];
    let split = crate::physical::split_reduction_regions(request, request.sole_output())
        .expect("a four-contributor relaxed request admits the split");
    assert_eq!(split.partition.partitions, 2);
    assert_eq!(split.partition.contributors_per_partition, 2);
    for (raw, members) in split.stages {
        regions.push(
            crate::physical::verify_schedule(raw, members, request).expect("each pass verifies"),
        );
    }
    regions
}

fn f32_tensor(shape: Shape, values: &[f32]) -> Tensor {
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

fn tensor_bits(tensor: &Tensor) -> Vec<u32> {
    match tensor.payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
            .collect(),
        _ => panic!("expected dense f32 reference output"),
    }
}

fn bits_of(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// The assembled split program reproduces the oracle for its own chosen order.
///
/// **The comparison is `strict_partitioned_sum` and never the serial fold.** A
/// split computes a *different* value from the serial reduction — that is what
/// reassociation means — so comparing against the serial answer could only ever
/// pass under a tolerance, and a tolerance is exactly the check that cannot fail
/// for the reason it exists. The oracle for the order the split actually
/// performs is the partitioned sum, and the comparison is bit for bit.
///
/// The oracle's input is the program's own prologue output rather than a value
/// re-derived here: the prologue is unchanged by the split, and re-implementing
/// `scale * x + bias` in the test would assert the test's arithmetic.
#[test]
fn the_assembled_split_program_matches_the_partitioned_sum_oracle() {
    let shape = Shape::from_dims([1, 4]);
    // The prologue maps these to `2e20, 3, -2e20, 3`, whose serial fold
    // `((2e20 + 3) - 2e20) + 3` is `3` while the split's
    // `(2e20 + 3) + (-2e20 + 3)` is `0`. A fixture without that cancellation
    // would let an implementation that never split pass every assertion below.
    let values: Vec<f32> = vec![1.0e20_f32, 1.0, -1.0e20, 1.0];
    let (semantic, request) = split_request(shape.clone());
    let scheduled = split_regions(&request);
    let program = crate::program::build_kernel_program(
        &semantic,
        &request,
        &split_assembly(&request, &scheduled),
    )
    .expect("the split program verifies");
    assert_eq!(program.stage_count(), 3);

    let kernels: Vec<_> = scheduled
        .iter()
        .map(|region| crate::physical::lower_structured_kernel(region).expect("each pass lowers"))
        .collect();
    let pointwise = interpret_fused(&kernels[0], &values);
    let partials = interpret_fused(&kernels[1], &pointwise);
    let actual = interpret_fused(&kernels[2], &partials);

    let pointwise_tensor = f32_tensor(shape, &pointwise);
    let axes = [Axis::new(1)];
    let expected_partials =
        tiler_reference::strict_partial_sums(&pointwise_tensor, &axes, 2, 2).unwrap();
    let expected = tiler_reference::strict_partitioned_sum(&pointwise_tensor, &axes, 2, 2).unwrap();
    assert_eq!(
        bits_of(&partials),
        tensor_bits(&expected_partials),
        "the partial pass staged values the oracle's partial fold does not produce"
    );
    assert_eq!(
        bits_of(&actual),
        tensor_bits(&expected),
        "the assembled split program does not compute its own declared order"
    );

    // The serial fold of the same prologue output disagrees, which is what makes
    // the exact comparison above discriminating.
    let serial_regions = crate::physical::build_scheduled_regions(&request).unwrap();
    let serial = interpret_fused(
        &crate::physical::lower_structured_kernel(&serial_regions[1]).unwrap(),
        &pointwise,
    );
    assert_ne!(
        bits_of(&serial),
        bits_of(&actual),
        "the fixture no longer distinguishes the two orders, so this test would \
         pass for an implementation that never split"
    );
}

/// The split's three-stage program declares the contract its two passes share.
#[test]
fn the_split_program_declares_its_partial_reduction_and_dispatch_order() {
    let (semantic, request) = split_request(Shape::from_dims([1, 4]));
    let scheduled = split_regions(&request);
    let program = crate::program::build_kernel_program(
        &semantic,
        &request,
        &split_assembly(&request, &scheduled),
    )
    .unwrap();
    let core = program.core();
    assert_eq!(core.stages().len(), 3);
    assert_eq!(core.values().len(), 4);

    let declared: Vec<_> = core.partial_reductions().collect();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].partitions(), 2);
    assert_eq!(declared[0].contributors_per_partition(), 2);
    assert_eq!(declared[0].total_contributors(), Some(4));
    assert_eq!(
        declared[0].producer().kernel(),
        core.stages().nth(1).unwrap().kernel()
    );
    assert_eq!(
        declared[0].combiner().kernel(),
        core.stages().nth(2).unwrap().kernel()
    );
    assert_eq!(declared[0].partial().role(), ValueRole::Temporary);
    assert_eq!(declared[0].result().role(), ValueRole::Output);
    assert_eq!(declared[0].partial().shape(), &Shape::from_dims([1, 2]));

    // The final pass covers no occurrence: the partial pass already claims the
    // reduction the two of them realize. The whole-program verifier admits that
    // only because the split above is declared — without it the stage is one
    // that computes nothing, and `UncoveringStage` rejects it.
    let coverage: Vec<usize> = core.stages().map(|stage| stage.coverage().len()).collect();
    assert_eq!(coverage, vec![4, 1, 0]);

    // Two ordering edges, both justified by data flow rather than declared. The
    // second is the visibility transition a split relies on instead of a
    // barrier: the pass boundary *is* the dispatch boundary.
    assert_eq!(core.dependencies().len(), 2);
    let ordered: Vec<ValueRole> = core
        .dependencies()
        .map(|edge| match edge.reason() {
            DependencyReasonView::Data(value) => value.role(),
            DependencyReasonView::StorageHandoff(_) => panic!("expected a data edge"),
        })
        .collect();
    assert_eq!(ordered, vec![ValueRole::Temporary, ValueRole::Temporary]);
}

/// The widened budgets admit the split program and still refuse a wider one.
///
/// The widening is an upper bound, not a licence: a request whose stated budget
/// is narrower than the shape this profile may assemble is refused at the
/// request boundary, and a program exceeding the budget its request states is
/// refused at assembly. Both directions are driven, because a widening that
/// only ever admitted would be indistinguishable from removing the check.
#[test]
fn the_widened_budgets_admit_the_split_program_and_still_refuse_a_narrower_request() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([1, 4]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    // The pre-widening values. The request boundary refuses them by name,
    // rather than admitting a request whose split it would later fail to build.
    for (resource, narrow) in [("regions", 2_u32), ("buffers", 3)] {
        let mut request = CompilationRequest::governed_under(
            &semantic,
            StrictF32NumericalContract::governed_relaxed(),
        );
        match resource {
            "regions" => request.budgets.regions = narrow,
            _ => request.budgets.buffers = narrow,
        }
        assert!(
            matches!(
                verify_planned_request(request),
                Err(crate::request::RequestError::BudgetExceeded { resource: named, .. })
                    if named == resource
            ),
            "a budget too narrow for the split program was admitted: {resource}"
        );
    }

    // And the program-side check still bites: a request stating exactly the
    // widened buffer budget admits the split, one stating less does not reach
    // it at all, so the value that separates them is the one that moved.
    //
    // The budget is `21` rather than the `4` this test first pinned, then the
    // `6` it pinned next, because it is sized to the largest program shape the
    // profile may be asked to admit and that is now the eighteen-input decoder
    // layer. The *requirement* this one-input program places on it is still
    // four — every declared input, the prologue's temporary, the split's staged
    // partial tensor, and the output — and `verify_program` derives that from
    // the declared arity, which is what the `buffers: 3` refusal above drives.
    // That is the point of the pair: the bound moved and the derived demand did
    // not, so a widening that had removed the check would fail the loop above.
    let (semantic, request) = split_request(Shape::from_dims([1, 4]));
    let scheduled = split_regions(&request);
    assert!(
        crate::program::build_kernel_program(
            &semantic,
            &request,
            &split_assembly(&request, &scheduled)
        )
        .is_ok()
    );
    assert_eq!(request.budgets().buffers, 21);
    assert_eq!(request.budgets().regions, 3);
}

/// **The closing evidence of
/// `admit-a-reassociating-contract-without-contraction`.** The recognized
/// serial-sum program compiles under a reassociation-permitting contract,
/// `compile` retains the three-stage split beside the two-stage serial one, and
/// the selected plan is still the serial one.
///
/// The last clause is the one that matters for the ticket boundary: the split is
/// *enumerated and retained*, and preference stays with
/// `calibrate-and-activate-parallel-reduction-selection`, because the structural
/// cost model prices two dispatches and a staged partial tensor above one
/// dispatch and no temporary. Nothing here calibrates anything.
///
/// The strict compilation at the end is the perturbation: the same program under
/// a contract that forbids reassociation retains no three-stage alternative at
/// all, so the assertion above cannot pass for a build that never split.
#[test]
fn the_reassociating_contract_reaches_the_split_through_compile() {
    /// The stage counts of one compilation's retained alternatives, ascending.
    fn retained_stage_counts(product: &CompilationProduct) -> Vec<usize> {
        let mut counts: Vec<usize> = product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .map(|alternative| alternative.program.stage_count())
            .collect();
        counts.sort_unstable();
        counts
    }

    // Four contributors: the extent `governed_partition` splits two-by-two and
    // the largest the governed target's declared grid-axis guarantee admits.
    let semantic = semantic_case_with_axis(
        Shape::from_dims([1, 4]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let product = compile(CompilationRequest::governed_under(
        &semantic,
        StrictF32NumericalContract::governed_reassociating(),
    ))
    .expect("the reassociating contract compiles the recognized serial sum");
    let target = &product.targets[0];

    // Two stages is the materialized prologue-then-reduce plan; three is the
    // same cover with its reduction realized as a partial and a final pass. The
    // whole-program fused plan is absent, and on a *different* obligation: it
    // contains the reduction, whose permitted reassociation `derive_fusion_legality`
    // does not prove — see
    // `fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`.
    assert_eq!(retained_stage_counts(&product), vec![2, 3]);

    let selected = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        })
        .expect("the selected alternative is one of the retained ones");
    assert_eq!(
        selected.program.stage_count(),
        2,
        "the split was selected; preference belongs to calibration, not to this ticket"
    );

    // Perturbation: forbidding reassociation withholds the split entirely, so
    // the three-stage retention above is a property of the contract rather than
    // of the program.
    let strict = compile(CompilationRequest::governed(&semantic))
        .expect("the strict contract compiles the same program");
    assert_eq!(retained_stage_counts(&strict), vec![1, 2]);
}
