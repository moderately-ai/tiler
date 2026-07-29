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
    CompilationProduct, CompileError, CompilerOutputError, ProgramAlternative,
    ProgramAlternativeKind, compile,
};
use crate::capability::{
    IndexAccessLoweringContext, IndexAccessLoweringProvider, LoweringCapabilityRegistryBuilder,
    LoweringCapabilityRevision, LoweringEmitError, LoweringSignature,
};
use crate::cover::RegionCover;
use crate::explain::{
    EvidenceBasis, ExplainDisposition, ExplainEvent, ExplainStage, FactValue, ProviderRef,
};
use crate::region::form_region_candidates;
use crate::request::{
    CompilationRequest, CompilerCapabilitySnapshot, RequestError, verify_request,
};
use tiler_ir::index::{DomainRole, FrozenScalarRegistry, ScalarAttributes};
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
use tiler_ir::shape::{Axis, Shape};

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

/// The gate's core claim: an externally defined operation set compiles end to
/// end through the ordinary path, and every implemented layer is present.
#[test]
fn externally_registered_operations_compile_through_the_ordinary_path() {
    let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
    let product = compile(CompilationRequest::governed(&program)).unwrap();
    let target = &product.targets[0];
    assert_eq!(
        target.target_profile_key,
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
        Some((1, 3))
    );
    // The verified KIR alone drives the backend-shaped interpreter.
    let values = vec![1.0, -2.0, 3.5, 0.5, -0.0, 0.0];
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
    let rank_two = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
    let rank_three = external_program(1, Shape::from_dims([2, 3, 2]), &[Axis::new(1)], false);
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
    let shared = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], true);
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

/// An ordered multi-output program is not silently approximated; the bounded
/// profile rejects it explicitly at the request boundary.
#[test]
fn ordered_multi_output_programs_reject_explicitly() {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&ExternalSemantics { revision: 1 })
        .unwrap();
    let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
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
            rule: "signature",
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
    let first = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
    let second = external_program(2, Shape::from_dims([2, 3]), &[Axis::new(1)], false);

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

    let first = compile(CompilationRequest::governed(&first)).unwrap();
    let second = compile(CompilationRequest::governed(&second)).unwrap();
    for kind in [
        ProgramAlternativeKind::Materialized,
        ProgramAlternativeKind::Fused,
    ] {
        let left = alternative(&first, kind);
        let right = alternative(&second, kind);
        // Pure structural content is identical: index/schedule identity, KIR,
        // the complete-plan receipt, and the plan's aggregate cost.
        assert_eq!(
            left.scheduled_regions[0].canonical_identity().as_bytes(),
            right.scheduled_regions[0].canonical_identity().as_bytes()
        );
        assert_eq!(left.kernels, right.kernels);
        assert_eq!(left.plan.identity(), right.plan.identity());
        assert_ne!(
            left.stable_id, right.stable_id,
            "the complete semantic provenance participates in alternative identity"
        );
        assert_eq!(left.structural_cost, right.structural_cost);
        // Selected-provider provenance is retained and unchanged: a semantic
        // provider revision is not a lowering-provider revision.
        assert_eq!(
            left.artifact_plan.lowering_providers(),
            right.artifact_plan.lowering_providers()
        );
        // The artifact construction plan retains the four-subject semantic
        // identity bundle atomically, so a changed admission subject is
        // visible there rather than being silently discarded.
        assert_ne!(left.artifact_plan, right.artifact_plan);
    }
    // The explain trace is bound to the exact compilation subject, so the two
    // request digests differ while the record sequence does not.
    assert_ne!(
        first.targets[0].explain.render(),
        second.targets[0].explain.render()
    );
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
        Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        false,
        2.0_f32.to_bits(),
    );
    let request = verify_request(CompilationRequest::governed(&program)).unwrap();
    let target = request.for_target(request.target_profiles()[0]).unwrap();
    let formation =
        form_region_candidates(&program, target.budgets(), target.numerical_contract()).unwrap();
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
        (Shape::from_dims([2, 3]), vec![Axis::new(1)]),
        (Shape::from_dims([3, 2]), vec![Axis::new(0)]),
        (Shape::from_dims([2, 3, 2]), vec![Axis::new(1)]),
    ] {
        let program = external_program(1, shape, &axes, false);
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
    );
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
    let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
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
    let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
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
            let modulo = context.modulo(read_coordinates[0], 2)?;
            let quotient = context.floor_div(read_coordinates[0], 2)?;
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
    let program = external_program(1, Shape::from_dims([70_000, 2]), &[Axis::new(0)], false);
    let product = compile(CompilationRequest::governed(&program)).unwrap();
    assert!(!product.targets[0].explain.records().iter().any(|record| {
        record.rule().key().as_str() == "kernel.index-region-refinement.v1"
            && record.event().disposition() != ExplainDisposition::Admitted
    }));
}

/// A finite residual read obligation is proved before executable planning.
///
/// The index region itself remains structurally verified and retains the exact
/// verifier-budget `Unknown` reason. The separately governed compiler authority
/// evaluates its whole finite domain, seals the proof, and only then permits
/// cover enumeration.
#[test]
fn a_finite_residual_index_obligation_is_proved_before_plan_construction() {
    let program = external_program(1, Shape::from_dims([65_535, 1]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = registry_with_external_multiply(
        true,
        Arc::new(ConservativeReadMultiplyLowering {
            rounds: 5,
            offset: 0,
        }),
    );
    let product = compile(request).expect("the exact finite authority proves the residual");
    let explain = &product.targets[0].explain;
    let residuals = explain
        .records()
        .iter()
        .filter(|record| {
            record.event().stage() == ExplainStage::SemanticDischarge
                && record.event().disposition() == ExplainDisposition::Admitted
        })
        .collect::<Vec<_>>();
    assert_eq!(residuals.len(), 1);
    let residual = residuals[0];
    assert_eq!(
        residual.rule().key().as_str(),
        "index-domain.semantic-discharge.v1"
    );
    assert_eq!(residual.rule().provider(), &ProviderRef::builtin());
    let ExplainEvent::Check { assessment, .. } = residual.event() else {
        panic!("the residual is a predicate assessment");
    };
    assert_eq!(assessment.basis(), &EvidenceBasis::ExhaustiveFinite);
    let fact = |key| {
        assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == key)
            .map_or_else(
                || panic!("residual assessment carries {key}"),
                crate::explain::ExplainFact::value,
            )
    };
    assert_eq!(fact("obligation-ordinal"), &FactValue::Count(0));
    assert_eq!(
        fact("predicate-kind"),
        &FactValue::Identity(
            crate::explain::SubjectKey::new("index-domain.less-than-extent").unwrap()
        )
    );
    assert_eq!(
        fact("evidence-basis"),
        &FactValue::Identity(crate::explain::SubjectKey::new("exhaustive-finite").unwrap())
    );
    assert_eq!(fact("exhaustive-points"), &FactValue::Count(65_535));
    assert!(matches!(fact("obligation-key"), FactValue::Identity(_)));
    assert_eq!(
        fact("discharge-provider"),
        &FactValue::Identity(
            crate::explain::SubjectKey::new("tiler.compiler.index-domain-discharge").unwrap()
        )
    );
    assert_eq!(
        fact("discharge-rule"),
        &FactValue::Identity(
            crate::explain::SubjectKey::new("tiler.exact-finite-index-domain-enumeration").unwrap()
        )
    );
    let discharge_position = explain
        .records()
        .iter()
        .position(|record| record.event().stage() == ExplainStage::SemanticDischarge)
        .expect("semantic discharge is explained");
    let cover_position = explain
        .records()
        .iter()
        .position(|record| record.event().stage() == ExplainStage::CandidateEnumeration)
        .expect("cover enumeration follows a proof");
    assert!(discharge_position < cover_position);
}

/// An exact counterexample is an invalid lowering, never invalid user input.
#[test]
fn a_disproved_residual_index_obligation_is_invalid_compiler_output() {
    let program = external_program(1, Shape::from_dims([65_535, 1]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = registry_with_external_multiply(
        true,
        Arc::new(ConservativeReadMultiplyLowering {
            rounds: 5,
            offset: 1,
        }),
    );
    let CompileError::Explained { source, explain } = compile(request).unwrap_err() else {
        panic!("target compilation failures retain their explain trace");
    };
    assert!(matches!(
        *source,
        CompileError::InvalidCompilerOutput(CompilerOutputError::Lowering(_))
    ));
    let record = explain
        .records()
        .iter()
        .find(|record| record.event().stage() == ExplainStage::SemanticDischarge)
        .expect("the counterexample is explained at semantic discharge");
    assert_eq!(
        record.event().disposition(),
        ExplainDisposition::RejectedIntrinsic
    );
    let ExplainEvent::Check { assessment, .. } = record.event() else {
        panic!("the disproof is a predicate assessment");
    };
    assert_eq!(
        assessment.reason().map(crate::explain::ReasonCode::as_str),
        Some("logical-index-not-less-than-extent")
    );
    assert!(assessment.facts().iter().any(|fact| {
        fact.key().as_str() == "discharge-provider"
            && fact.value()
                == &FactValue::Identity(
                    crate::explain::SubjectKey::new("tiler.compiler.index-domain-discharge")
                        .unwrap(),
                )
    }));
    assert_eq!(
        assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == "counterexample-point-ordinal")
            .map(crate::explain::ExplainFact::value),
        Some(&FactValue::Count(65_534))
    );
    assert!(
        !explain
            .records()
            .iter()
            .any(|record| record.event().stage() == ExplainStage::CandidateEnumeration),
        "a disproved lowering never reaches cover enumeration"
    );
}

/// A second proof-budget stop remains unsupported without execution permission.
#[test]
fn an_over_discharge_budget_obligation_remains_unknown_before_planning() {
    let program = external_program(1, Shape::from_dims([65_535, 64]), &[Axis::new(1)], false);
    let mut request = CompilationRequest::governed(&program);
    request.capabilities = registry_with_external_multiply(
        true,
        Arc::new(ConservativeReadMultiplyLowering {
            rounds: 5,
            offset: 0,
        }),
    );
    let CompileError::Explained { source, explain } = compile(request).unwrap_err() else {
        panic!("target compilation failures retain their explain trace");
    };
    assert_eq!(
        *source,
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "semantic-discharge",
            rule: "index-domain-discharge-unsupported",
        })
    );
    let record = explain
        .records()
        .iter()
        .find(|record| record.event().stage() == ExplainStage::SemanticDischarge)
        .expect("the resource stop is explained at semantic discharge");
    assert_eq!(
        record.event().disposition(),
        ExplainDisposition::DeferredUnsupported
    );
    let ExplainEvent::Check { assessment, .. } = record.event() else {
        panic!("the unknown result is a predicate assessment");
    };
    assert_eq!(assessment.basis(), &EvidenceBasis::Unknown);
    assert_eq!(
        assessment.reason().map(crate::explain::ReasonCode::as_str),
        Some("index-domain-proof-resource-limit")
    );
    let fact = |key| {
        assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == key)
            .map_or_else(
                || panic!("resource assessment carries {key}"),
                crate::explain::ExplainFact::value,
            )
    };
    assert_eq!(fact("proof-limit"), &FactValue::Count(16 * 1024 * 1024));
    assert_eq!(
        fact("verifier-proof-limit"),
        &FactValue::Count(tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS)
    );
    assert!(
        !explain
            .records()
            .iter()
            .any(|record| record.event().stage() == ExplainStage::CandidateEnumeration),
        "a second resource stop never reaches cover enumeration"
    );
}
