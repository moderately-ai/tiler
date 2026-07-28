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
use crate::physical::{RegionId, TensorRole};
use crate::request::CompilerCapabilitySnapshot;
use std::collections::BTreeMap;
use tiler_ir::kernel::{BinaryOp, CompareOp, ConvertOp, KernelConstant, OperationView};
use tiler_ir::program::{DependencyReasonView, ValueRole};
use tiler_ir::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey,
    SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

fn semantic(reverse_constants: bool) -> SemanticProgram {
    semantic_case(
        Shape::from_dims([2, 3]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        reverse_constants,
    )
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
        let mut machine = KirMachine {
            kernel,
            input,
            output: vec![f32::NAN; outputs],
            values: BTreeMap::new(),
        };
        for invocation in 0..u64::try_from(outputs).unwrap() {
            machine.values.clear();
            machine.run_block(kernel.body(), invocation);
        }
        machine.output
    }

    fn run_block(&mut self, block: tiler_ir::kernel::BlockRef<'a>, invocation: u64) {
        for operation in block.operations() {
            let mut results = operation.results();
            match operation.view() {
                OperationView::Builtin { .. } => {
                    self.define(&mut results, KirValue::Index(invocation));
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
                        other => panic!("unsupported binary operation {other:?}"),
                    };
                    self.define(&mut results, value);
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
                OperationView::Barrier { .. } => {}
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
    let mut matching = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .filter(|alternative| alternative.kind == kind);
    let found = matching
        .next()
        .unwrap_or_else(|| panic!("a retained {} alternative", kind.name()));
    assert!(
        matching.next().is_none(),
        "the bounded profile retains exactly one {} alternative",
        kind.name()
    );
    found
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
    let first = compile(CompilationRequest::governed(&first)).unwrap();
    let second = compile(CompilationRequest::governed(&second)).unwrap();

    assert_eq!(first, second);
    let target = &first.targets[0];
    let rendered = target.explain.render();
    assert!(rendered.starts_with("tiler-explain-v2 request="));
    assert!(rendered.contains("feasibility:threads-per-workgroup:admitted"));
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
    assert_eq!(reduction_loop(&materialized.kernels[1]), Some((1, 3)));
    assert_eq!(fused.program.stage_count(), 1);
    assert_eq!(fused.program.core().values().len(), 2);
    // The exact aggregate structural cost is the sum of the per-region
    // estimates plus the cover's deliberate cross-region materializations.
    assert_eq!(materialized.structural_cost.dispatch_count(), 2);
    assert_eq!(materialized.structural_cost.launched_threads(), 8);
    assert_eq!(materialized.structural_cost.temporary_bytes(), 24);
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
    assert_eq!(reduction_loop(&fused.kernels[0]), Some((1, 3)));
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "compile.plan.boundary"
            && record.event().disposition() == ExplainDisposition::Admitted
    }));
    // The materialized plan discharges exactly one cross-region handoff; the
    // fused plan materializes nothing across a boundary.
    assert_eq!(materialized.plan.handoffs().len(), 1);
    assert!(fused.plan.handoffs().is_empty());
    // Both alternatives are the exact selected plans, so their stable
    // identity is the plan's content-derived identity label.
    for alternative in &target.portfolio.alternatives {
        assert_eq!(alternative.stable_id, alternative.plan.identity().label());
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
            ("normalize.semantics.v1", 1),
            ("region.formation.v1", 1),
            ("region.candidate.v1", 17),
            // One resolution and one refinement per recognized occurrence.
            ("capability.index-access-resolution.v1", 5),
            ("kernel.index-region-refinement.v1", 5),
            ("cover.enumeration.v1", 1),
            ("fusion.legality.v1", 12),
            ("fusion.strict-f32-equivalence", 1),
            ("frontier.enumeration.v1", 4),
            ("selection.complete-plan.v1", 1),
            ("compile.region.verified", 3),
            ("compile.plan.boundary", 2),
            ("schedule.plan-regions", 2),
            ("kernel.plan-refinement", 2),
            ("program.plan-verified", 2),
            ("artifact.plan-construction", 2),
            ("target.barriers", 3),
            ("target.buffer-bindings", 3),
            ("target.device-memory", 3),
            ("target.grid-axis", 3),
            ("target.index-bits", 3),
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
            // Two retained plans, each reporting its four modelled analytical
            // components (allocation, dispatch, synchronization, indexing) plus
            // its count of unmodelled ones. The five `Unknown` components are
            // deliberately not emitted as zeros, so this number grows as components become
            // modelled rather than staying at nine from the start.
            ("tiler.cost.analytical.v1", 10),
            ("tiler.cost.structural.v1", 2),
            ("tiler.selection.structural-pareto.v1", 2),
        ])
    );
    for (rule, fact_key, expected) in [
        ("normalize.semantics.v1", "rewrite-count", 0),
        ("region.formation.v1", "candidate-count", 17),
        ("region.formation.v1", "operation-count", 5),
        ("cover.enumeration.v1", "cover-count", 16),
        ("selection.complete-plan.v1", "plan-count", 2),
    ] {
        let record = trace
            .records()
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
    assert!(trace.render().starts_with("tiler-explain-v2 request="));
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
            required,
            outcome,
            profile,
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
            "honourability:numerics.input-subnormals:preserve:honoured:supported-exactly:profile=tiler.prototype-target-neutral-baseline.v1"
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
            "grid-axis" | "threads-per-workgroup" => {
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
            "index-bits" | "device-memory" | "barriers" => {
                matches!(
                    (required, available),
                    (Quantity::Count(_), Quantity::Count(_))
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
            ("barriers", 3),
            ("buffer-bindings", 3),
            ("device-memory", 3),
            ("grid-axis", 3),
            ("index-bits", 3),
            ("local-memory-bytes", 3),
            ("threads-per-workgroup", 3),
        ])
    );

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
    let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
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
    let shape = Shape::from_dims([2, 3]);
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

    // Both spellings normalize to the same canonical program, so every
    // downstream physical decision and receipt is identical.
    assert_eq!(
        from_duplicated.targets[0].portfolio,
        from_shared.targets[0].portfolio
    );

    // The traces differ only in what normalization actually did.
    let rewrite_counts = |product: &CompilationProduct| {
        product.targets[0]
            .explain
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
            .explain
            .records()
            .iter()
            .any(
                |record| record.rule().key().as_str() == "normalize.common-subexpression.v1"
                    && record.event().disposition() == ExplainDisposition::Admitted
            )
    );
    assert!(
        !from_shared.targets[0]
            .explain
            .records()
            .iter()
            .any(|record| record.rule().key().as_str() == "normalize.common-subexpression.v1")
    );
}

/// A shared constant read by two operations is graph fan-out, and a legal
/// cover must materialize it once rather than duplicate its producer.
#[test]
fn shared_constant_fan_out_is_materialized_once_and_never_duplicated() {
    let shared = shared_constant_semantic(Shape::from_dims([2, 3]), 2.0_f32.to_bits());
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
            rule: "signature",
        })
    );
    assert_eq!(
        error.to_string(),
        "compile.unsupported.strategy.signature: no installed capability can compile this valid semantic program"
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
fn forged_same_key_target_facts_are_rejected_at_the_request_boundary() {
    let semantic = semantic(false);
    let mut request = CompilationRequest::governed(&semantic);
    request.target_profiles[0].max_threads_per_grid_axis = 1;
    let error = compile(request).unwrap_err();
    assert_eq!(
        error,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "target",
            rule: "prototype-target-neutral-baseline-v1",
        })
    );
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
    let error = compile(bounded).unwrap_err();
    let CompileError::Explained { source, explain } = error else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        *source,
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
                    available: Quantity::Threads(65_535),
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
fn no_feasible_plan_retains_a_typed_terminal_failure_trace() {
    let semantic = semantic_case_with_axis(
        Shape::from_dims([70_000, 70_000]),
        2.0_f32.to_bits(),
        1.0_f32.to_bits(),
        false,
        Axis::new(1),
    );
    let error = compile(CompilationRequest::governed(&semantic)).unwrap_err();
    let CompileError::Explained { source, explain } = error else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        *source,
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
        Shape::from_dims([2, 3]),
        vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, 0.0],
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
    enumerate_complete_plans(semantic, request, &formation, &mut explain, root, None).map_or_else(
        |_| panic!("the governed request enumerates complete plans"),
        |plans| plans.portfolio,
    )
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
