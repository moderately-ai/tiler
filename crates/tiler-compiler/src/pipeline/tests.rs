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
    fn boolean(self) -> bool {
        match self {
            Self::Bool(value) => value,
            other => panic!("expected a predicate value, found {other:?}"),
        }
    }
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
    input: &'a [f32],
    output: Vec<f32>,
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
    fn run(kernel: &'a VerifiedKernel, input: &'a [f32]) -> Vec<f32> {
        let mut buffers = kernel.buffers();
        let read = buffers.next().expect("a read buffer parameter");
        let write = buffers.next().expect("a write buffer parameter");
        assert_eq!(read.access, tiler_ir::kernel::BufferAccess::Read);
        assert_eq!(write.access, tiler_ir::kernel::BufferAccess::Write);
        assert_eq!(input.len(), usize::try_from(read.element_count).unwrap());
        let outputs = usize::try_from(write.element_count).unwrap();
        // Read from the kernel's own staging declaration, so the machine still
        // resolves nothing from the schedule, the request, or the graph: a
        // kernel that stages nothing runs one lane per output exactly as before.
        let slots = kernel
            .staging()
            .next()
            .map_or(1, |staging| staging.element_count.max(1));
        let participants = usize::try_from(slots).unwrap();
        let mut machine = KirMachine {
            kernel,
            input,
            output: vec![f32::NAN; outputs],
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
                    let value = self.get(source).float();
                    let value = match op {
                        ConvertOp::CanonicalizeF32Nan => {
                            if value.is_nan() {
                                f32::from_bits(
                                    self.kernel.numerical().canonical_arithmetic_nan_bits,
                                )
                            } else {
                                value
                            }
                        }
                        other => panic!("unsupported conversion {other:?}"),
                    };
                    self.define(&mut results, KirValue::F32(value));
                }
                OperationView::Load { offset, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    let value = KirValue::F32(self.input[offset]);
                    self.define(&mut results, value);
                }
                OperationView::Store { offset, value, .. } => {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    self.output[offset] = self.get(value).float();
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
    KirMachine::run(kernel, input)
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
            // Two summary records remain per region subject: admitted count and
            // rejected count. Typed per-opaque-rejection detail records accompany
            // them when present; this governed compile fixture has no opaque
            // rejection, so its four region subjects still contribute eight.
            ("frontier.enumeration.v1", 8),
            // Exactly two strategies are considered and withheld, both at the
            // reduction subject and both for the same reason: this fixture
            // compiles under the strict contract, and the multi-pass split and
            // the single-workgroup tree each *are* a reassociation of the
            // declared contributor sequence. The other three subjects never
            // reach either strategy, so a third record would mean one was being
            // considered somewhere it does not apply.
            ("frontier.strategy-decline.v1", 2),
            ("selection.complete-plan.v1", 1),
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
/// mistaken for "the dtype works". Nothing below the semantic layer moved —
/// there is no capability row, no lowering capability, and no target profile
/// that can state a BF16 numerical contract — so the request boundary refuses
/// it before any target is consulted.
///
/// The rule that says no is `dtype-f32` rather than `operation-set`: the
/// bounded strategy requires the governed `f32` identity, and it reaches that
/// check before it reaches the operation vocabulary. Asserting `operation-set`
/// here would be fiction about which check refused.
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

    assert_eq!(
        compile(CompilationRequest::governed(&bf16_program)).unwrap_err(),
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "dtype-f32",
        }),
        "a bf16 program is refused for its dtype, not routed to a target"
    );

    // The neighbour that keeps this about bf16 rather than about a dead
    // request path: the same shape of program in f32 compiles.
    compile(CompilationRequest::governed(&semantic(false)))
        .expect("the governed f32 fixture still compiles");
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
    let verified = verify_request(request_with_targets(
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
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "target.grid-axis"
            && record.subjects()[0].key().as_str() == "region:pointwise"
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
    // Every recognized region role the target refused is named exactly once.
    let mut subjects = causal_rejections
        .iter()
        .map(|record| record.subjects()[0].key().as_str())
        .collect::<Vec<_>>();
    subjects.sort_unstable();
    assert_eq!(
        subjects,
        [
            "region:pointwise",
            "region:reduction",
            "region:whole-program"
        ]
    );
}

#[test]
fn target_rejections_are_deduplicated_by_region_role_and_axis() {
    let semantic = semantic(false);
    let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
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
    let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
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
    let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
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
        .find(|plan| plan_region_order(plan).is_none())
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
    let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let formation = plan_formation(&semantic, &request);
    let compiled = compile(CompilationRequest::governed(&semantic)).unwrap();
    let mut forged = alternative(&compiled, ProgramAlternativeKind::Fused).clone();
    let opaque = crate::selection::opaque_fused_portfolio_fixture(&semantic);
    let plan = opaque
        .plans()
        .iter()
        .find(|plan| plan_region_order(plan).is_none())
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
    let verified = verify_request(CompilationRequest::governed(&semantic)).unwrap();
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
    let rewritten_request = verify_request(CompilationRequest::governed_under(
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
    let verified = verify_request(CompilationRequest::governed_under(
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
    let verified = verify_request(CompilationRequest::governed(&binding)).unwrap();
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
    let verified = verify_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let subject = FrontierRegionSubject::new("fused", request.serial_sum().members.all());
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
    let cause = record_frontier(&mut explain, "fused", &frontier, root).unwrap();
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

/// The implementations the frontier admitted for one region role, as the compile
/// path's own explain trace reports them.
fn admitted_count(trace: &VerifiedExplainTrace, role: &str) -> Option<u64> {
    let key = format!("region:{role}");
    trace.records().iter().find_map(|record| {
        let ExplainEvent::Check { assessment, .. } = record.event() else {
            return None;
        };
        if assessment.predicate().as_str() != "frontier.locally-feasible"
            || !record
                .subjects()
                .iter()
                .any(|subject| subject.key().as_str() == key)
        {
            return None;
        }
        assessment
            .facts()
            .iter()
            .find_map(|fact| match (fact.key().as_str(), fact.value()) {
                ("admitted-count", FactValue::Count(count)) => Some(*count),
                _ => None,
            })
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
    let verified = verify_request(CompilationRequest::governed_under(
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
    let verified = verify_request(CompilationRequest::governed(&semantic)).unwrap();
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
    let verified = verify_request(request).expect("the relaxed contract is admitted");
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
        crate::physical::single_workgroup_tree_region(&request).expect("the tree is available");
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
    let program = crate::program::build_kernel_program(&semantic, &request, &scheduled)
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
    let verified = verify_request(request).expect("the strict contract is admitted");
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
    let (region, members) = crate::physical::single_workgroup_tree_region(&request)
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
    let (region, members) = crate::physical::single_workgroup_tree_region(&request)
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
        let (region, members) = crate::physical::single_workgroup_tree_region(&request)
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
            crate::physical::single_workgroup_tree_region(&request).err(),
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
        crate::physical::single_workgroup_tree_region(&prime).err(),
        Some(
            crate::physical::WorkgroupTreeUnavailable::NoAdmissibleParticipantCount {
                contributors: 7,
            }
        )
    );
}

/// Assembles the three verified regions of one request's split program.
fn split_regions(
    request: &crate::request::VerifiedTargetRequest,
) -> Vec<crate::physical::VerifiedScheduledRegion> {
    let (raw, members) = crate::physical::pointwise_region(request);
    let mut regions = vec![
        crate::physical::verify_schedule(raw, members, request).expect("the prologue verifies"),
    ];
    let split = crate::physical::split_reduction_regions(request)
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
    let program = build_split_kernel_program(&semantic, &request, &scheduled)
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
    let program = build_split_kernel_program(&semantic, &request, &scheduled).unwrap();
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
                verify_request(request),
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
    // The budget is `6` rather than the `4` this test first pinned because the
    // recognizer now admits an elementwise prologue over several declared
    // inputs. The *requirement* this one-input program places on it is still
    // four — every declared input, the prologue's temporary, the split's staged
    // partial tensor, and the output — and `verify_program` derives that from
    // the declared arity, which is what the `buffers: 3` refusal above drives.
    let (semantic, request) = split_request(Shape::from_dims([1, 4]));
    let scheduled = split_regions(&request);
    assert!(build_split_kernel_program(&semantic, &request, &scheduled).is_ok());
    assert_eq!(request.budgets().buffers, 6);
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
