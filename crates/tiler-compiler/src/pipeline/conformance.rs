//! The target-neutral optimizer conformance gate.
//!
//! Everything here drives the ordinary `compile()` entry point. Nothing reaches
//! past it into a stage-local constructor, and no fixture is admitted by a
//! `cfg(test)` shortcut: the operation definitions come from a registry provider
//! written entirely against `tiler-ir`'s public surface, exactly as an
//! out-of-crate consumer would supply them.

use std::sync::Arc;

use super::tests::{interpret_fused, reduction_loop};
use super::{
    CompilationProduct, CompileError, ProgramAlternative, ProgramAlternativeKind, compile,
};
use crate::capability::{
    IndexAccessLoweringContext, IndexAccessLoweringProvider, LoweringCapabilityRegistryBuilder,
    LoweringCapabilityRevision, LoweringEmitError, LoweringSignature,
};
use crate::cover::RegionCover;
use crate::explain::{ExplainDisposition, ExplainStage, ProviderRef};
use crate::region::form_region_candidates;
use crate::request::{
    CompilationRequest, CompilerCapabilitySnapshot, RequestError, verify_planned_request,
};
use tiler_ir::index::{
    DomainRole, FrozenScalarRegistry, IndexRealizationLaw, ScalarAttributes, ScalarRegistryBuilder,
    SourcedExtent, add_f32_scalar_op,
};
use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, InputKey, NormativeDefinitionRef, OpKey, OperationArity,
    OperationAttributeSchema, OperationConformance, OperationDefinition, OperationDefinitionFacts,
    OperationEffect, OperationInferenceError, OperationInferencer, OperationSchema, OutputKey,
    ProviderDiagnosticCode, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, RegistryError,
    SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, TypeDefinitionFacts, TypeKey, ValueFact, ValueTypeDefinition,
    ValueTypeDefinitionKey, add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// The shape-inference behaviour one externally registered operation declares.
#[derive(Clone, Copy)]
enum ExternalOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for ExternalOperation {
    fn infer(
        &self,
        request: tiler_ir::semantic::OperationInferenceRequest<'_>,
        outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = operands[0].shape();
                let right = operands[1].shape();
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        ProviderDiagnosticCode::new("external.binary.shape").unwrap(),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = request
                    .attributes()
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        ProviderDiagnosticCode::new("external.sum.axes").unwrap(),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                ProviderDiagnosticCode::new("external.sum.axis-width").unwrap(),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            ProviderDiagnosticCode::new("external.sum.axis-kind").unwrap(),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    operands[0].shape().without_axes(&axes),
                ))
            }
        }
    }
}

/// An out-of-crate semantic provider that defines the whole operation set.
///
/// Its revision is the output-affecting provider revision ADR 0072 keeps
/// separate from graph meaning, so the same graph admitted at two revisions
/// is the exact identity-conformance subject this gate asserts.
struct ExternalSemantics {
    revision: u32,
}

struct LawSubstitutionSemantics {
    law: IndexRealizationLaw,
}

impl SemanticRegistryProvider for LawSubstitutionSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("acme", "law-substitution-semantics", 1).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                NormativeDefinitionRef::new("law substitution binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register(
            registrar,
            multiply_f32_op(),
            2,
            &[],
            ExternalOperation::Binary,
        )?;
        registrar.register_index_realization_law(multiply_f32_op(), 1, self.law.clone())
    }
}

impl SemanticRegistryProvider for ExternalSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("acme", "external-f32-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                NormativeDefinitionRef::new("external binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register(
            registrar,
            constant_f32_op(),
            0,
            &[OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            ExternalOperation::Constant,
        )?;
        register(
            registrar,
            multiply_f32_op(),
            2,
            &[],
            ExternalOperation::Binary,
        )?;
        register(registrar, add_f32_op(), 2, &[], ExternalOperation::Binary)?;
        register(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            &[OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            ExternalOperation::Sum,
        )
    }
}

fn register(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: &[OperationAttributeSchema],
    inferencer: ExternalOperation,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key,
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes.to_vec(),
        )
        .expect("the external operation schema is valid"),
        NormativeDefinitionRef::new("external governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))
}

/// Builds a scale-bias-then-serial-sum program from the external registry.
///
/// Every operation the graph contains is defined by [`ExternalSemantics`];
/// nothing in it comes from `SemanticProgramBuilder::try_standard`.
fn external_program(
    revision: u32,
    shape: Shape,
    axes: &[Axis],
    share_constant: bool,
) -> SemanticProgram {
    external_program_with_bias(revision, shape, axes, share_constant, 1.0_f32.to_bits())
}

/// Builds the same program with an explicit bias constant bit pattern.
///
/// A bias equal to the scale gives two *distinct* constant occurrences with
/// identical content, which is the region content/occurrence separation
/// subject; the default fixture keeps them distinguishable instead.
fn external_program_with_bias(
    revision: u32,
    shape: Shape,
    axes: &[Axis],
    share_constant: bool,
    bias_bits: u32,
) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&ExternalSemantics { revision })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape)
        .unwrap();
    let scale = tiler_ir::semantic::F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = if share_constant {
        scale
    } else {
        tiler_ir::semantic::F32Constant::apply(&mut builder, bias_bits).unwrap()
    };
    let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = tiler_ir::semantic::F32Add::apply(&mut builder, product, bias).unwrap();
    let sum =
        tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, mapped, axes.to_vec()).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// Builds the same graph under the governed semantic authority.
///
/// Compilation conformance fixtures use this authority because the governed
/// realization laws are admitted against its exact frozen definitions.
fn governed_program(shape: Shape, axes: &[Axis], share_constant: bool) -> SemanticProgram {
    governed_program_with_bias(shape, axes, share_constant, 1.0_f32.to_bits())
}

fn governed_program_with_bias(
    shape: Shape,
    axes: &[Axis],
    share_constant: bool,
    bias_bits: u32,
) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape)
        .unwrap();
    let scale = tiler_ir::semantic::F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = if share_constant {
        scale
    } else {
        tiler_ir::semantic::F32Constant::apply(&mut builder, bias_bits).unwrap()
    };
    let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = tiler_ir::semantic::F32Add::apply(&mut builder, product, bias).unwrap();
    let sum =
        tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, mapped, axes.to_vec()).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

fn alternative(product: &CompilationProduct, kind: ProgramAlternativeKind) -> &ProgramAlternative {
    product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| alternative.kind == kind)
        .expect("the requested plan shape is retained")
}

/// Asserts a cover assigns every operation to exactly one region.
fn assert_complete_partition(cover: &RegionCover, operation_count: u32) {
    let mut members: Vec<u32> = cover
        .regions()
        .iter()
        .flat_map(|region| region.members().iter().map(|member| member.0))
        .collect();
    members.sort_unstable();
    let distinct = members
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        members.len(),
        distinct.len(),
        "no operation is double covered"
    );
    assert_eq!(
        u32::try_from(members.len()).unwrap(),
        operation_count,
        "no operation is left uncovered"
    );
}

/// A foreign semantic authority cannot borrow governed realization evidence.
#[test]
fn externally_registered_operations_require_their_own_realization_authority() {
    let program = external_program(1, Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    assert!(matches!(
        compile(CompilationRequest::governed(&program)).unwrap_err(),
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "capability",
            rule: "semantic-authority-pairing",
        })
    ));
}

/// The gate's core claim: the governed operation set compiles end to end
/// through the ordinary path, and every implemented layer is present.
#[test]
fn governed_operations_compile_through_the_ordinary_path() {
    let program = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let product = compile(CompilationRequest::governed(&program)).unwrap();
    let target = &product.targets[0];
    assert_eq!(
        target.target_profile.profile_key().as_str(),
        "tiler.prototype-target-neutral-baseline.v1"
    );
    assert_eq!(target.portfolio.alternatives.len(), 2);

    for alternative in &target.portfolio.alternatives {
        // Complete legal cover, one implementation per region, verified KIR,
        // a neutral kernel program, and an artifact construction plan.
        assert_complete_partition(
            alternative.plan.cover(),
            u32::try_from(program.operation_count()).unwrap(),
        );
        assert_eq!(
            alternative.plan.selections().len(),
            alternative.plan.cover().region_count()
        );
        assert_eq!(
            alternative.kernels.len(),
            alternative.scheduled_regions.len()
        );
        assert_eq!(
            alternative.program.stage_count(),
            alternative.scheduled_regions.len()
        );
        assert!(!alternative.artifact_plan.lowering_providers().is_empty());
        // Every retained plan rests on hard-feasibility evidence, never cost.
        assert!(!alternative.plan.guards().is_empty());
        // Every fused region carries a replayable fusion-legality proof.
        let fused_regions = alternative
            .plan
            .cover()
            .regions()
            .iter()
            .filter(|region| region.members().len() > 1)
            .count();
        assert_eq!(alternative.equivalence.legality().len(), fused_regions);
    }
    assert!(
        alternative(&product, ProgramAlternativeKind::Fused)
            .equivalence
            .numerical()
            .is_some(),
        "a whole-program fused plan carries its strict-f32 equivalence proof"
    );
    assert_eq!(
        reduction_loop(&alternative(&product, ProgramAlternativeKind::Fused).kernels[0]),
        Some((1, 2))
    );
    // The verified KIR alone drives the backend-shaped interpreter.
    let values = vec![1.0, -2.0, 3.5, 0.5];
    let fused = interpret_fused(
        &alternative(&product, ProgramAlternativeKind::Fused).kernels[0],
        &values,
    );
    assert_eq!(fused.len(), 2);
}

/// Two non-isomorphic graph shapes — a rank-2 trailing reduction and a rank-3
/// interior reduction — both compile, and neither borrows the other's plan.
#[test]
fn non_isomorphic_graph_shapes_produce_distinct_verified_plans() {
    let rank_two = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let rank_three = governed_program(Shape::from_dims([1, 2, 2]), &[Axis::new(1)], false);
    assert_ne!(
        rank_two.semantic_identity().graph(),
        rank_three.semantic_identity().graph(),
        "the two fixtures must be non-isomorphic graphs"
    );

    let first = compile(CompilationRequest::governed(&rank_two)).unwrap();
    let second = compile(CompilationRequest::governed(&rank_three)).unwrap();
    for product in [&first, &second] {
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 2);
    }
    // Distinct semantics yield distinct plan identities at every layer.
    let left = alternative(&first, ProgramAlternativeKind::Fused);
    let right = alternative(&second, ProgramAlternativeKind::Fused);
    assert_ne!(left.plan.identity(), right.plan.identity());
    assert_ne!(left.stable_id, right.stable_id);
    assert_ne!(
        left.scheduled_regions[0].canonical_identity().as_bytes(),
        right.scheduled_regions[0].canonical_identity().as_bytes()
    );
    assert_ne!(left.kernels[0], right.kernels[0]);
    assert_ne!(left.artifact_plan, right.artifact_plan);
}

/// Graph fan-out: one constant read by two operations is materialized once.
#[test]
fn shared_producer_fan_out_compiles_without_duplicating_the_producer() {
    let shared = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], true);
    assert_eq!(shared.operation_count(), 4);
    let product = compile(CompilationRequest::governed(&shared)).unwrap();
    for alternative in &product.targets[0].portfolio.alternatives {
        assert_complete_partition(
            alternative.plan.cover(),
            u32::try_from(shared.operation_count()).unwrap(),
        );
        assert!(alternative.plan.cover().duplication().is_none());
    }
}

/// Two ordered named outputs over two distinct domains, in one declaration
/// order.
///
/// `summed = a + b` at `[2, 2]` and `reduced = sum(a * b, axis 1)` at `[2]`,
/// over the same two declared `[2, 2]` inputs.
///
/// Three properties, each load-bearing. The two outputs are **independent** —
/// neither reads the other — so each recognition walk claims its own part of
/// the partition. Each walk reads **both** declared inputs, which is what the
/// elementwise reader requires and what
/// `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs` owns
/// widening. And the two are published at **different domains**, which is what
/// makes the assembler's output attribution observable at all: a cover's regions
/// run in canonical occurrence order, which has nothing to do with declaration
/// order, so an assembler pairing the two lists positionally binds a
/// `[2]`-shaped write to a `[2, 2]`-shaped published value for whichever
/// declaration order disagrees.
fn independent_two_output_program(wide_first: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let wide = tiler_ir::semantic::F32Add::apply(&mut builder, a, b).unwrap();
    let contributor = tiler_ir::semantic::F32Multiply::apply(&mut builder, a, b).unwrap();
    let narrow =
        tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, contributor, [Axis::new(1)])
            .unwrap();
    let wide_key = OutputKey::new("summed").unwrap();
    let narrow_key = OutputKey::new("reduced").unwrap();
    if wide_first {
        builder.output(wide_key, wide).unwrap();
        builder.output(narrow_key, narrow).unwrap();
    } else {
        builder.output(narrow_key, narrow).unwrap();
        builder.output(wide_key, wide).unwrap();
    }
    builder.build().unwrap()
}

/// The gate's multi-output row, discharged by a program that compiles.
///
/// **This assertion was the inverse until `output-arity` was relaxed.** It read
/// the refusal and recorded the multi-output row of
/// `docs/correctness-and-testing.md`'s requirement as a negative test; the row
/// is now positive, and what it reports is a complete legal cover, one
/// implementation selected per region, a verified kernel program, and an
/// artifact construction plan for a program declaring two ordered named
/// outputs.
///
/// **Both declaration orders are compiled, and that is the check that can say
/// no.** The published interface must follow the caller's declaration order,
/// and each key must carry the domain its own producing occurrence computes —
/// which is what an attribution by value proves and an attribution by execution
/// order does not. One of the two orders necessarily disagrees with the cover's
/// canonical region order, so pairing positionally makes exactly one of these
/// two compilations fail.
#[test]
fn ordered_multi_output_programs_compile_through_the_ordinary_path() {
    for wide_first in [true, false] {
        let program = independent_two_output_program(wide_first);
        assert_eq!(program.output_count(), 2);
        assert_eq!(program.operation_count(), 3);

        let product = compile(CompilationRequest::governed(&program)).unwrap();
        assert_eq!(product.targets[0].failure(), None);
        let target = &product.targets[0];
        assert!(!target.portfolio.alternatives.is_empty());

        let declared: Vec<(OutputKey, Shape)> = program
            .outputs()
            .map(|output| {
                (
                    output.key().clone(),
                    program.shape(output.value()).unwrap().clone(),
                )
            })
            .collect();
        for alternative in &target.portfolio.alternatives {
            // A complete legal cover over both outputs' partitions, with one
            // implementation selected per region: this is the complete-plan
            // selection evidence a multi-output program had never reached.
            assert_complete_partition(
                alternative.plan.cover(),
                u32::try_from(program.operation_count()).unwrap(),
            );
            assert_eq!(
                alternative.plan.selections().len(),
                alternative.plan.cover().region_count()
            );
            assert_eq!(
                alternative.kernels.len(),
                alternative.scheduled_regions.len()
            );

            // The published interface, in the caller's declaration order, with
            // each key bound to a value at its own producing occurrence's
            // domain.
            let published: Vec<(OutputKey, Shape)> = alternative
                .program
                .core()
                .outputs()
                .map(|output| (output.key().clone(), output.value().shape().clone()))
                .collect();
            assert_eq!(
                published, declared,
                "the assembled interface does not match the declared one",
            );
            assert!(!alternative.artifact_plan.lowering_providers().is_empty());
            assert!(!alternative.plan.guards().is_empty());
        }
    }
}

/// A program publishing an intermediate it also consumes still refuses, by name.
///
/// This is the shape the multi-output row used to be asserted with — publish
/// `scaled` and reduce it into `reduced` — and it is *not* what discharges the
/// row, because it does not compile. Recording which refusal it now reports is
/// what keeps the gate's bound honest: the two outputs' recognition walks share
/// the scaling occurrence, so it refuses at the request boundary under
/// `output-partition-overlap` rather than at any layer below it. One region's
/// owning write would otherwise have to serve both the materialization edge its
/// consumer reads across and the publication, and
/// `tiler_ir::program::ValueRole` is exclusive.
/// `admit-elementwise-epilogues-over-a-materialized-intermediate` owns the copy
/// stage that lifts it.
#[test]
fn a_published_and_consumed_intermediate_refuses_by_name() {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&ExternalSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let scale = tiler_ir::semantic::F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, scale).unwrap();
    let sum = tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, product, [Axis::new(1)])
        .unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), sum)
        .unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), product)
        .unwrap();
    let multi_output = builder.build().unwrap();
    assert_eq!(multi_output.output_count(), 2);

    let error = compile(CompilationRequest::governed(&multi_output)).unwrap_err();
    assert_eq!(
        error,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "output-partition-overlap",
        })
    );
}

/// ADR 0072 identity conformance for a provider-only revision change.
///
/// The same graph admitted by two revisions of the same external provider
/// keeps its graph meaning and its reached definition projection, changes its
/// admission provenance and registry snapshot, and — because neither is
/// structural content — reproduces every structural layer byte for byte.
#[test]
fn provider_only_revision_changes_provenance_and_not_structure() {
    let first = external_program(1, Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let second = external_program(2, Shape::from_dims([2, 2]), &[Axis::new(1)], false);

    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );
    assert_eq!(
        first.semantic_identity().reached_definitions(),
        second.semantic_identity().reached_definitions()
    );
    assert_ne!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance()
    );
    assert_ne!(
        first.semantic_identity().registry_snapshot(),
        second.semantic_identity().registry_snapshot()
    );

    for program in [&first, &second] {
        assert!(matches!(
            compile(CompilationRequest::governed(program)).unwrap_err(),
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "capability",
                rule: "semantic-authority-pairing",
            })
        ));
    }
}

/// Equal region *content* is reused across distinct graph *occurrences*.
///
/// The two pointwise constants of the unshared program are structurally
/// identical singleton regions, so region formation must give them one
/// content identity and two distinct occurrence identities (ADR 0072).
#[test]
fn identical_region_content_keeps_distinct_occurrence_identities() {
    let program = external_program_with_bias(
        1,
        Shape::from_dims([2, 2]),
        &[Axis::new(1)],
        false,
        2.0_f32.to_bits(),
    );
    let formation = form_region_candidates(
        &program,
        crate::request::DeterministicBudgets::governed(),
        crate::request::StrictF32NumericalContract::governed(),
    )
    .unwrap();
    let constants: Vec<_> = formation
        .candidates()
        .iter()
        .filter(|candidate| {
            candidate.members().len() == 1 && candidate.boundary_inputs().is_empty()
        })
        .collect();
    assert_eq!(
        constants.len(),
        2,
        "the fixture has exactly two constant occurrences"
    );
    assert_eq!(
        constants[0].content(),
        constants[1].content(),
        "structurally identical regions share one content identity"
    );
    assert_ne!(
        constants[0].occurrence(),
        constants[1].occurrence(),
        "distinct graph occurrences keep distinct occurrence identities"
    );
}

/// Every enumerated cover a plan rests on is a complete legal partition, and
/// every retained plan implements each of its regions exactly once.
#[test]
fn complete_plan_coverage_is_exact_at_every_retained_plan() {
    for (shape, axes) in [
        (Shape::from_dims([2, 2]), vec![Axis::new(1)]),
        (Shape::from_dims([2, 2]), vec![Axis::new(0)]),
        (Shape::from_dims([1, 2, 2]), vec![Axis::new(1)]),
    ] {
        let program = governed_program(shape, &axes, false);
        let product = compile(CompilationRequest::governed(&program)).unwrap();
        for alternative in &product.targets[0].portfolio.alternatives {
            assert_complete_partition(
                alternative.plan.cover(),
                u32::try_from(program.operation_count()).unwrap(),
            );
            let mut occurrences: Vec<_> = alternative
                .plan
                .selections()
                .iter()
                .map(|selection| selection.occurrence().clone())
                .collect();
            occurrences.sort();
            let distinct = occurrences.len();
            occurrences.dedup();
            assert_eq!(
                occurrences.len(),
                distinct,
                "no region occurrence is implemented twice"
            );
            // Every materialization edge the cover proposes is discharged by
            // exactly one satisfied cross-region handoff.
            assert_eq!(
                alternative.plan.handoffs().len(),
                alternative.plan.cover().materializations().len()
            );
        }
    }
}

// Externally registered *lowering* capabilities.
//
// Everything below composes a lowering-capability registry through the
// public `capability` surface, exactly as an out-of-crate consumer would,
// and drives it through the ordinary `compile()` entry point.

/// An out-of-crate index-access lowering for `tiler.multiply-f32`.
///
/// It reads every extent and every broadcast from the occurrence facts, so
/// one registration covers every program shape. Nothing in it touches a
/// crate-internal item.
struct ExternalMultiplyLowering;

impl IndexAccessLoweringProvider for ExternalMultiplyLowering {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let shape = context.occurrence().results()[0].shape().clone();
        let value_type = context.occurrence().results()[0].value_type().clone();
        let inputs = context.occurrence().inputs().to_vec();
        let operands = context.occurrence().operands().to_vec();
        let mut dimensions = Vec::new();
        for extent in shape.extents() {
            dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
        }
        let mut coordinates = Vec::new();
        for dimension in &dimensions {
            coordinates.push(context.dimension_expr(*dimension)?);
        }
        let mut tensors = Vec::new();
        for input in &inputs {
            tensors.push(context.input_tensor(input.value_type().clone(), input.shape().clone())?);
        }
        let mut values = Vec::new();
        for position in &operands {
            let value = if inputs[*position].shape().rank() == 0 {
                context.read(tensors[*position], &[], &[])?
            } else {
                context.read(tensors[*position], &dimensions, &coordinates)?
            };
            values.push(value);
        }
        let product = context.apply(
            tiler_ir::index::multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &values,
        )?;
        let product = product.get(0).expect("multiply yields one result");
        let output = context.output_tensor(value_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, product)
    }
}

struct ExternalAddLowering;

impl IndexAccessLoweringProvider for ExternalAddLowering {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let shape = context.occurrence().results()[0].shape().clone();
        let value_type = context.occurrence().results()[0].value_type().clone();
        let inputs = context.occurrence().inputs().to_vec();
        let operands = context.occurrence().operands().to_vec();
        let dimensions = shape
            .extents()
            .iter()
            .map(|extent| context.dimension(DomainRole::Parallel, *extent))
            .collect::<Result<Vec<_>, _>>()?;
        let coordinates = dimensions
            .iter()
            .map(|dimension| context.dimension_expr(*dimension))
            .collect::<Result<Vec<_>, _>>()?;
        let tensors = inputs
            .iter()
            .map(|input| context.input_tensor(input.value_type().clone(), input.shape().clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let values = operands
            .iter()
            .map(|position| context.read(tensors[*position], &dimensions, &coordinates))
            .collect::<Result<Vec<_>, _>>()?;
        let sum = context.apply(add_f32_scalar_op(), ScalarAttributes::empty(), &values)?;
        let output = context.output_tensor(value_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, sum.get(0).expect("add yields one result"))
    }
}

#[test]
fn equal_semantic_snapshots_cannot_substitute_the_programs_law() {
    let build_semantic = |law| {
        let mut builder = SemanticRegistryBuilder::new();
        builder
            .register_provider(&LawSubstitutionSemantics { law })
            .unwrap();
        builder.freeze().unwrap()
    };
    let program_semantic = build_semantic(IndexRealizationLaw::multiply_f32());
    let capability_semantic = build_semantic(IndexRealizationLaw::add_f32());
    assert_eq!(
        program_semantic.snapshot_identity(),
        capability_semantic.snapshot_identity()
    );

    let mut program = SemanticProgramBuilder::try_new(program_semantic.clone()).unwrap();
    let shape = Shape::from_dims([2]);
    let left = program
        .input::<F32>(InputKey::new("left").unwrap(), shape.clone())
        .unwrap();
    let right = program
        .input::<F32>(InputKey::new("right").unwrap(), shape)
        .unwrap();
    let product = tiler_ir::semantic::F32Multiply::apply(&mut program, left, right).unwrap();
    program
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    let program = program.build().unwrap();

    let standard = FrozenScalarRegistry::standard().unwrap();
    let mut scalar_builder = ScalarRegistryBuilder::new(capability_semantic.clone());
    for operation in [
        tiler_ir::index::multiply_f32_scalar_op(),
        add_f32_scalar_op(),
    ] {
        scalar_builder
            .register(
                ProviderIdentity::new("tiler", "standard-scalars", 1).unwrap(),
                standard.definition(&operation).unwrap().clone(),
            )
            .unwrap();
    }
    let scalars = scalar_builder.freeze();
    assert!(matches!(
        tiler_ir::index::FrozenIndexRealizationLawRegistry::from_semantic(
            program_semantic.clone(),
            scalars.clone(),
        ),
        Err(tiler_ir::index::IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch)
    ));
    assert!(matches!(
        tiler_ir::index::IndexRealizationAuthority::admit(
            &program_semantic,
            &scalars,
            multiply_f32_op(),
            tiler_ir::index::IndexRefinementSignature::new(
                [F32::resolved_type(), F32::resolved_type()],
                [F32::resolved_type()],
            )
            .unwrap(),
            &[add_f32_scalar_op()],
        ),
        Err(tiler_ir::index::IndexRefinementVerificationError::ScalarSemanticAuthorityMismatch)
    ));
    assert!(matches!(
        LoweringCapabilityRegistryBuilder::new(program_semantic, scalars.clone()),
        Err(crate::capability::LoweringRegistryError::RefinementAuthority { .. })
    ));
    let mut lowerings =
        LoweringCapabilityRegistryBuilder::new(capability_semantic, scalars.clone()).unwrap();
    lowerings
        .register_index_access(
            external_lowering_provider(),
            multiply_f32_op(),
            LoweringSignature::new(
                [F32::resolved_type(), F32::resolved_type()],
                [F32::resolved_type()],
            )
            .unwrap(),
            &[add_f32_scalar_op()],
            LoweringCapabilityRevision::new(1).unwrap(),
            Arc::new(ExternalAddLowering),
        )
        .unwrap();
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = CompilerCapabilitySnapshot::new(lowerings.freeze(), scalars);
    assert_eq!(
        verify_planned_request(request),
        Err(RequestError::UnsupportedCapability {
            phase: "capability",
            rule: "semantic-authority-pairing",
        })
    );
}

fn external_lowering_provider() -> ProviderIdentity {
    ProviderIdentity::new("acme", "external-multiply-lowering", 3).unwrap()
}

/// Composes a registry from the governed families plus an external one.
///
/// `substitute` replaces the governed `tiler.multiply-f32` capability;
/// otherwise the external capability is registered *beside* it, which is the
/// contended-capability case.
fn registry_with_external_multiply(
    substitute: bool,
    implementation: Arc<dyn IndexAccessLoweringProvider>,
) -> CompilerCapabilitySnapshot {
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let mut builder = LoweringCapabilityRegistryBuilder::new(
        scalars.semantic_authority().clone(),
        scalars.clone(),
    )
    .unwrap();
    for capability in crate::governed::governed_index_access_capabilities().unwrap() {
        if substitute && capability.operation() == &multiply_f32_op() {
            continue;
        }
        capability.register(&mut builder).unwrap();
    }
    builder
        .register_index_access(
            external_lowering_provider(),
            multiply_f32_op(),
            LoweringSignature::new(
                [F32::resolved_type(), F32::resolved_type()],
                [F32::resolved_type()],
            )
            .unwrap(),
            &[tiler_ir::index::multiply_f32_scalar_op()],
            LoweringCapabilityRevision::new(7).unwrap(),
            implementation,
        )
        .unwrap();
    CompilerCapabilitySnapshot::new(builder.freeze(), scalars)
}

/// The lowering half of the gate: an out-of-crate provider lowers a
/// recognized occurrence end to end, and the artifact plan names it.
#[test]
fn an_externally_registered_lowering_provider_drives_the_compile_path() {
    let program = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities =
        registry_with_external_multiply(true, Arc::new(ExternalMultiplyLowering));
    let product = compile(request).unwrap();
    let target = &product.targets[0];
    assert_eq!(target.portfolio.alternatives.len(), 2);

    let external = crate::request::LoweringProviderIdentity::new(
        external_lowering_provider(),
        "tiler.capability.index-access.tiler.multiply-f32.v1".to_owned(),
        LoweringCapabilityRevision::new(7).unwrap(),
    );
    for alternative in &target.portfolio.alternatives {
        assert!(
            alternative
                .artifact_plan
                .lowering_providers()
                .contains(&external),
            "the artifact plan records the external provider that lowered multiply"
        );
    }
    // The external provider's own capability revision is what the resolution
    // record is attributed at, not the governed one.
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.rule().provider()
                == &ProviderRef::registered(&external_lowering_provider()).unwrap()
    }));
}

/// Two providers claiming one occurrence is a contradiction, not a choice.
#[test]
fn contended_lowering_capabilities_fail_closed_with_a_distinct_error() {
    let program = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities =
        registry_with_external_multiply(false, Arc::new(ExternalMultiplyLowering));
    let error = compile(request).unwrap_err();
    let CompileError::Explained { source, explain } = error else {
        panic!("target compilation failures retain their explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: "ambiguous-capability",
        })
    );
    // A contradiction is a disproved check, never a deferred capability: the
    // authority was extended, and its extensions disagree.
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.event().disposition() == ExplainDisposition::RejectedIntrinsic
    }));
    assert!(!explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.event().disposition() == ExplainDisposition::DeferredUnsupported
    }));
}

/// An out-of-crate lowering whose read interval proof cannot settle.
///
/// Each reconstruction round computes `2 * floor(i / 2) + (i mod 2)`. It is
/// exactly `i`, but interval propagation treats the two terms independently and
/// includes the exclusive upper bound. Verification therefore has to enumerate
/// the access domain, at `points × plan_len` evaluated cells, which is exactly
/// the budget
/// `tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS` governs.
struct ConservativeReadMultiplyLowering {
    rounds: usize,
    offset: i128,
}

/// An alternate multiply-shaped region used to prove exact-canonical law
/// refusal; a lowering installer has no semantic-authority callback to replace.
impl IndexAccessLoweringProvider for ConservativeReadMultiplyLowering {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let shape = context.occurrence().results()[0].shape().clone();
        let value_type = context.occurrence().results()[0].value_type().clone();
        let inputs = context.occurrence().inputs().to_vec();
        let operands = context.occurrence().operands().to_vec();
        let mut dimensions = Vec::new();
        for extent in shape.extents() {
            dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
        }
        let mut coordinates = Vec::new();
        for dimension in &dimensions {
            coordinates.push(context.dimension_expr(*dimension)?);
        }
        let mut read_coordinates = coordinates.clone();
        for _ in 0..self.rounds {
            let two = SourcedExtent::Static(Extent::new(2));
            let modulo = context.modulo(read_coordinates[0], two.clone())?;
            let quotient = context.floor_div(read_coordinates[0], two)?;
            read_coordinates[0] = context.linear_combination(
                0_i128.into(),
                &[(2_i128.into(), quotient), (1_i128.into(), modulo)],
            )?;
        }
        if self.offset != 0 {
            read_coordinates[0] = context
                .linear_combination(self.offset.into(), &[(1_i128.into(), read_coordinates[0])])?;
        }
        let mut tensors = Vec::new();
        for input in &inputs {
            tensors.push(context.input_tensor(input.value_type().clone(), input.shape().clone())?);
        }
        let mut values = Vec::new();
        for position in &operands {
            let value = if inputs[*position].shape().rank() == 0 {
                context.read(tensors[*position], &[], &[])?
            } else {
                context.read(tensors[*position], &dimensions, &read_coordinates)?
            };
            values.push(value);
        }
        let product = context.apply(
            tiler_ir::index::multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &values,
        )?;
        let product = product.get(0).expect("multiply yields one result");
        let output = context.output_tensor(value_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, product)
    }
}

/// The governed lowerings are interval-provable at any recognized size.
///
/// Their writes are coordinate permutations and their reads are bounded by
/// their own dimensions, so verification never enters the exhaustive path and
/// the proof budget is never charged. This is the measured fact that lets
/// refinement be attempted for every occurrence rather than gated on size.
#[test]
fn governed_lowerings_never_charge_the_exhaustive_proof_budget() {
    let program = governed_program(Shape::from_dims([70_000, 2]), &[Axis::new(0)], false);
    let product = compile(CompilationRequest::governed(&program)).unwrap();
    assert!(!product.targets[0].explain.records().iter().any(|record| {
        record.rule().key().as_str() == "kernel.index-region-refinement.v1"
            && record.event().disposition() != ExplainDisposition::Admitted
    }));
}

/// `silu(x)` over one declared input, at the requested shape.
fn activation_program(shape: Shape) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape)
        .unwrap();
    let activated = tiler_ir::semantic::F32Silu::apply(&mut builder, input).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), activated)
        .unwrap();
    builder.build().unwrap()
}

/// Composes the governed registry with one family's capability omitted.
fn registry_without(omitted: &OpKey) -> CompilerCapabilitySnapshot {
    let scalars = FrozenScalarRegistry::standard().unwrap();
    let mut builder = LoweringCapabilityRegistryBuilder::new(
        scalars.semantic_authority().clone(),
        scalars.clone(),
    )
    .unwrap();
    for capability in crate::governed::governed_index_access_capabilities().unwrap() {
        if capability.operation() == omitted {
            continue;
        }
        capability.register(&mut builder).unwrap();
    }
    CompilerCapabilitySnapshot::new(builder.freeze(), scalars)
}

/// The activation compiles, and its kernel agrees with the reference bit for bit.
///
/// **This is the closing evidence that the projection is the composition and not
/// a plausible chain.** The boundary states `tiler::silu-f32@1`'s per-point body
/// once, into the physical expression vocabulary; `KirMachine` reads only the
/// structured kernel, resolving nothing from the semantic graph, the request, or
/// the schedule; and the expected values come from `tiler-reference`'s own
/// evaluation of the same semantic program. A reordered composition, a
/// reciprocal-and-multiply spelling, or a divisor built as `e + 1.0` would each
/// have to survive a bit comparison against the pinned reference, and the
/// `-88.73` band below is where the first two stop doing so.
///
/// The corpus is the activation's own boundary corpus rather than a sample: both
/// zeros, both infinities, the one-ULP disagreement point at `-88.0`, the last
/// normal result at `-88.7228`, and the first exactly `-0.0` at `-88.73`. It runs
/// in rows of four because the governed profile declares a four-thread grid axis
/// and a pointwise region launches one invocation per element, so a wider row
/// would be refused for a reason this test does not model.
#[test]
fn the_activation_compiles_and_matches_the_reference_bit_for_bit() {
    let rows: [[f32; 4]; 3] = [
        [0.0, -0.0, 1.0, -1.0],
        [f32::INFINITY, f32::NEG_INFINITY, -88.0, -88.722_8],
        [-88.73, 2.0, -2.0, 1.0e-30],
    ];
    let shape = Shape::from_dims([4]);
    let program = activation_program(shape.clone());
    let product = compile(CompilationRequest::governed(&program)).unwrap();
    // One occurrence covered by one region, so the only retained plan shape is
    // the whole-program fused one.
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let key = InputKey::new("input").unwrap();
    let evaluator = ReferenceEvaluator::standard().unwrap();

    let mut results = Vec::new();
    for values in rows {
        let actual = interpret_fused(&fused.kernels[0], &values);
        let tensor = Tensor::dense(
            F32::resolved_type(),
            shape.clone(),
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
        let expected = evaluator
            .evaluate(&program, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        let TensorPayloadView::Dense(elements) = expected[0].payload() else {
            panic!("the activation's reference result is a dense f32 tensor")
        };
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            elements
                .iter()
                .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
                .collect::<Vec<_>>(),
            "row {values:?}",
        );
        results.push(actual);
    }

    // Three values are asserted against their exact bit patterns as well as
    // against the reference, because each is a point a *wrong* composition can
    // still reproduce elsewhere. `silu(-0.0)` is `-0.0` only if the negation is
    // an exact sign flip; `silu(-inf)` is a NaN only if the result is a division
    // rather than a multiply by a reciprocal; and the `-88.73` band's `-0.0`
    // comes from a finite negative divided by an overflowed exponential, which a
    // reciprocal-and-multiply spelling would reach as `-0.0 * inf` — a NaN.
    assert_eq!(results[0][1].to_bits(), 0x8000_0000);
    assert!(results[1][1].is_nan());
    assert_eq!(results[2][0].to_bits(), 0x8000_0000);
}

/// A recognized family with no installed capability refuses by name.
///
/// The perturbation is one omitted registration and nothing else: the same
/// program, the same request, the same target. Recognition still admits the
/// activation — a program the boundary refused would report `operation-set`
/// instead — and lowering resolution then fails closed against the exact
/// occurrence, which is the disposition an absent capability owes.
#[test]
fn omitting_the_activation_capability_refuses_the_recognized_occurrence() {
    let program = activation_program(Shape::from_dims([4]));
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = registry_without(&tiler_ir::semantic::silu_f32_op());
    let CompileError::Explained { source, explain } = compile(request).unwrap_err() else {
        panic!("a capability refusal retains its explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: "missing-capability",
        })
    );
    // Deferred rather than disproved: the authority was never extended to this
    // occurrence, which is a different finding from two extensions contradicting
    // each other.
    assert!(explain.records().iter().any(|record| {
        record.rule().key().as_str() == "capability.index-access-resolution.v1"
            && record.event().disposition() == ExplainDisposition::DeferredUnsupported
    }));
}

/// A profile that declares no elementary realization refuses the activation.
///
/// **The perturbed profile differs from the governed one in its key alone**, so
/// every capability, numerical, dispatch, and synchronization fact it declares is
/// identical and the only thing it has not declared is that it realizes
/// `tiler::silu-f32@1`'s subordinate exponential. That is what makes the refusal
/// attributable to the accuracy obligation rather than to a weaker profile.
///
/// The pair is completed by the arithmetic control below: the same perturbed
/// profile compiles a program with no elementary family, so the refusal is the
/// contract's and not the profile's.
#[test]
fn a_profile_declaring_no_elementary_realization_refuses_the_activation() {
    let unattested = crate::request::TargetProfile::governed_with_key_for_test("acme.gpu.v1");

    let activation = activation_program(Shape::from_dims([4]));
    let mut request = CompilationRequest::governed(&activation);
    request.target_profiles = vec![unattested.clone()];
    let product = compile(request).unwrap();
    assert_eq!(
        product.targets[0].failure(),
        Some(&CompileError::UnsupportedCapability(
            RequestError::UnrealizedElementaryAccuracy {
                operation: tiler_ir::semantic::silu_f32_op(),
                target_profile: unattested.profile_key().clone(),
                reason: "accuracy.elementary.no-installed-realization",
            }
        )),
    );

    // The control: the same profile, a program whose every operation's result is
    // fixed by IEEE-754 alone, and no accuracy obligation to place on it.
    let arithmetic = governed_program(Shape::from_dims([2, 2]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&arithmetic);
    request.target_profiles = vec![unattested];
    let product = compile(request).unwrap();
    assert!(
        product.targets[0].compiled().is_some(),
        "the perturbed profile refuses only the elementary obligation",
    );

    // And the governed profile, which does declare the realization, compiles the
    // activation — so the refusal above is the missing declaration rather than
    // an obligation nothing can discharge.
    let product = compile(CompilationRequest::governed(&activation)).unwrap();
    assert!(product.targets[0].compiled().is_some());
}

/// An alternate logical realization cannot certify itself as multiply.
#[test]
fn a_lowering_cannot_replace_the_semantic_providers_realization_law() {
    let program = governed_program(Shape::from_dims([65_535, 1]), &[Axis::new(0)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = registry_with_external_multiply(
        true,
        Arc::new(ConservativeReadMultiplyLowering {
            rounds: 5,
            offset: 0,
        }),
    );
    let CompileError::Explained { source, explain } = compile(request).unwrap_err() else {
        panic!("the noncanonical realization is rejected with its explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: "refinement-refused",
        })
    );
    assert!(
        explain.records().iter().any(|record| {
            record.event().stage() == ExplainStage::KernelRefinement
                && record.event().disposition() != ExplainDisposition::Admitted
        }),
        "the semantic-law mismatch is explained before planning"
    );
}
