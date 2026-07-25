//! Tests for the compiler-owned host, ABI, routing, and artifact layers.
//!
//! The stage DAG, materialized values, views, allocations, lifetimes, typed
//! dependencies, named outputs, and complete semantic coverage are verified by
//! `tiler_ir::program` and tested there; a malformed one cannot be constructed
//! here at all. These tests cover what this module still owns.

use super::*;

use tiler_ir::program::{AllocationOwnership, DependencyReasonView, ValueRole};
use tiler_ir::semantic::{
    F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
    StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

use crate::physical::{
    build_fused_scheduled_region, build_scheduled_regions, lower_structured_kernel,
};
use crate::request::{CompilationRequest, verify_request};

fn fixture() -> (
    SemanticProgram,
    VerifiedTargetRequest,
    Vec<VerifiedScheduledRegion>,
) {
    fixture_with_scale(2.0_f32.to_bits())
}

/// Returns the lowering provenance the request's installed registry resolves.
fn resolved_providers(
    semantic: &SemanticProgram,
    request: &VerifiedTargetRequest,
) -> Vec<crate::request::LoweringProviderIdentity> {
    crate::lowering::resolve_capabilities(semantic, request)
        .expect("the governed registry lowers the fixture")
}

fn fixture_with_scale(
    scale_bits: u32,
) -> (
    SemanticProgram,
    VerifiedTargetRequest,
    Vec<VerifiedScheduledRegion>,
) {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let semantic = builder.build().unwrap();
    let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let scheduled = build_scheduled_regions(&request).unwrap();
    (semantic, request, scheduled)
}

#[test]
fn artifact_construction_rejects_a_cross_program_semantic_request_mix() {
    let (_, request, scheduled) = fixture_with_scale(2.0_f32.to_bits());
    let (different_semantic, _, _) = fixture_with_scale(3.0_f32.to_bits());
    let (semantic, _, _) = fixture_with_scale(2.0_f32.to_bits());
    let program = build_kernel_program(&semantic, &request, &scheduled).unwrap();

    assert_eq!(
        build_artifact_plan(
            &different_semantic,
            &request,
            &scheduled,
            &scheduled
                .iter()
                .map(lower_structured_kernel)
                .collect::<Result<Vec<_>, _>>()
                .unwrap(),
            &program,
            resolved_providers(&semantic, &request),
        ),
        Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        })
    );
    // The shared builder is opened against the request's exact program, so a
    // mismatched semantic program cannot even assemble a core.
    assert_eq!(
        build_kernel_program(&different_semantic, &request, &scheduled),
        Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        })
    );
}

#[test]
fn two_stage_program_has_explicit_temporary_abi_and_routing_commit() {
    let (semantic, request, scheduled) = fixture();
    let program = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    let kernels = [
        lower_structured_kernel(&scheduled[0]).unwrap(),
        lower_structured_kernel(&scheduled[1]).unwrap(),
    ];
    assert_kernels_match_program(&request, &scheduled, &program, &kernels).unwrap();
    verify_semantic_output_type(&semantic).unwrap();
    let artifact = build_artifact_plan(
        &semantic,
        &request,
        &scheduled,
        &kernels,
        &program,
        resolved_providers(&semantic, &request),
    )
    .unwrap();

    let core = program.core();
    let temporary = core.values().nth(1).expect("the cross-stage temporary");
    assert_eq!(temporary.role(), ValueRole::Temporary);
    assert_eq!(temporary.required_bytes(), 24);
    assert_eq!(
        temporary.allocation().ownership(),
        AllocationOwnership::Program
    );
    let dependency = core.dependencies().next().expect("one data dependency");
    assert!(matches!(
        dependency.reason(),
        DependencyReasonView::Data(value) if value == temporary
    ));
    // Every stage's bound implementation is the kernel the compiler lowered.
    assert!(
        core.stages()
            .zip(&kernels)
            .all(|(stage, kernel)| stage.kernel() == kernel)
    );
    // The two stages cover the whole graph exactly once between them.
    let covered: usize = core.stages().map(|stage| stage.coverage().len()).sum();
    assert_eq!(covered, semantic.operation_count());

    assert_eq!(
        program.entries[0].bindings[1].role,
        ComponentRole::Intermediate
    );
    assert_eq!(
        program.entries[1].bindings[0].role,
        ComponentRole::Intermediate
    );
    assert!(!program.routing[1].fallback_permitted);
    assert!(!program.routing[2].fallback_permitted);
    assert_eq!(artifact.entry_regions, [RegionId::new(0), RegionId::new(1)]);
    assert_eq!(
        artifact.numerical_realizations,
        [
            scheduled[0].region().index.numerical,
            scheduled[1].region().index.numerical,
        ]
    );
    assert!(!artifact.semantic_identity.graph().as_bytes().is_empty());
    assert!(
        !artifact
            .semantic_identity
            .reached_definitions()
            .as_bytes()
            .is_empty()
    );
    assert!(
        !artifact
            .semantic_identity
            .admission_provenance()
            .as_bytes()
            .is_empty()
    );
    assert!(
        !artifact
            .semantic_identity
            .registry_snapshot()
            .as_bytes()
            .is_empty()
    );
}

#[test]
fn the_program_identity_is_the_shared_canonical_identity() {
    let (semantic, request, scheduled) = fixture();
    let first = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    let second = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    assert_eq!(
        first.core().canonical_identity().as_bytes(),
        second.core().canonical_identity().as_bytes()
    );

    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_fused_kernel_program(&semantic, &request, &fused_region).unwrap();
    // The two strategies realize one graph with different bound refinements and
    // a different coverage partition, so their identities differ.
    assert_ne!(
        first.core().canonical_identity().as_bytes(),
        fused.core().canonical_identity().as_bytes()
    );
    assert_eq!(
        first.core().semantic_graph_identity(),
        fused.core().semantic_graph_identity()
    );
}

#[test]
fn compiler_layers_reject_abi_and_routing_failures() {
    let (semantic, request, scheduled) = fixture();
    let valid = build_kernel_program(&semantic, &request, &scheduled).unwrap();

    let mut invalid_abi = valid.clone();
    invalid_abi.entries[1].bindings[0].access = AbiAccess::Write;
    assert_eq!(
        verify_kernel_program_layers(&invalid_abi, &request, &scheduled),
        Err(ProgramError::Abi {
            rule: "binding",
            stage: StageId(1),
        })
    );

    let mut wrong_binding_value = valid.clone();
    wrong_binding_value.entries[0].bindings[1].value = MaterializedValueId(2);
    assert_eq!(
        verify_kernel_program_layers(&wrong_binding_value, &request, &scheduled),
        Err(ProgramError::Abi {
            rule: "binding",
            stage: StageId(0),
        })
    );

    let mut extra_binding = valid.clone();
    let duplicate = extra_binding.entries[0].bindings[0];
    extra_binding.entries[0].bindings.push(duplicate);
    assert_eq!(
        verify_kernel_program_layers(&extra_binding, &request, &scheduled),
        Err(ProgramError::Abi {
            rule: "binding-cardinality",
            stage: StageId(0),
        })
    );

    let mut invalid_routing = valid;
    invalid_routing.routing[1].fallback_permitted = true;
    assert_eq!(
        verify_kernel_program_layers(&invalid_routing, &request, &scheduled),
        Err(ProgramError::Routing {
            rule: "fallback-after-commit",
        })
    );
}

#[test]
fn compiler_layers_recheck_the_target_and_the_host_expression_graph() {
    let (semantic, request, scheduled) = fixture();
    let valid = build_kernel_program(&semantic, &request, &scheduled).unwrap();

    let mut wrong_target = valid.clone();
    wrong_target.target_profile_key = "wrong-target";
    assert_eq!(
        verify_kernel_program_layers(&wrong_target, &request, &scheduled),
        Err(ProgramError::Structure {
            rule: "target-profile",
        })
    );

    let mut wrong_bytes = valid.clone();
    wrong_bytes.host_expressions[2] = ExprNode::Root(AbiRoot::UnsignedLiteral(4));
    assert_eq!(
        verify_kernel_program_layers(&wrong_bytes, &request, &scheduled),
        Err(ProgramError::HostExpression {
            rule: "canonical-graph",
            expression: HostExprId(0),
        })
    );

    let mut wrong_launch = valid.clone();
    wrong_launch.host_expressions[5] = ExprNode::Root(AbiRoot::UnsignedLiteral(5));
    assert_eq!(
        verify_kernel_program_layers(&wrong_launch, &request, &scheduled),
        Err(ProgramError::HostExpression {
            rule: "canonical-graph",
            expression: HostExprId(0),
        })
    );

    let mut missing_stage_entry = valid;
    missing_stage_entry.entries.pop();
    assert_eq!(
        verify_kernel_program_layers(&missing_stage_entry, &request, &scheduled),
        Err(ProgramError::Structure {
            rule: "cardinality",
        })
    );
}

#[test]
fn host_expression_overflow_is_a_hard_failure() {
    let (semantic, request, scheduled) = fixture();
    let mut program = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    program.host_expressions[0] = ExprNode::Root(AbiRoot::UnsignedLiteral(u64::MAX));
    assert_eq!(
        evaluate_expressions(&program.host_expressions),
        Err(ProgramError::HostExpression {
            rule: "overflow",
            expression: HostExprId(2),
        })
    );

    let mut malformed = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    malformed.host_expressions[2] = ExprNode::Binary {
        op: AbiBinaryOp::CheckedMultiply,
        left: 99,
        right: 1,
    };
    assert_eq!(
        verify_kernel_program_layers(&malformed, &request, &scheduled),
        Err(ProgramError::HostExpression {
            rule: "canonical-graph",
            expression: HostExprId(0),
        })
    );
}

#[test]
fn builders_are_total_over_short_and_forged_slices() {
    let (semantic, request, scheduled) = fixture();
    assert_eq!(
        build_kernel_program(&semantic, &request, &[]),
        Err(ProgramError::Structure {
            rule: "strategy-cardinality",
        })
    );
    assert_eq!(
        build_kernel_program(&semantic, &request, &scheduled[..1]),
        Err(ProgramError::Structure {
            rule: "strategy-cardinality",
        })
    );

    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_fused_kernel_program(&semantic, &request, &fused_region).unwrap();
    let kernel = lower_structured_kernel(&fused_region).unwrap();
    assert_kernels_match_program(
        &request,
        std::slice::from_ref(&fused_region),
        &fused,
        std::slice::from_ref(&kernel),
    )
    .unwrap();
    assert_eq!(fused.stage_count(), 1);
    assert_eq!(fused.dependency_count(), 0);
    assert_eq!(fused.core().values().len(), 2);

    // A kernel list that does not match the program's bound implementations is
    // rejected even when both are individually verified.
    assert_eq!(
        assert_kernels_match_program(&request, &scheduled, &fused, &[kernel]),
        Err(ProgramError::Structure {
            rule: "kernel-entry-cardinality",
        })
    );
}

#[test]
fn artifact_receipt_rejects_provider_program_and_receipt_mutations() {
    let (semantic, request, scheduled) = fixture();
    let program = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let resolved = resolved_providers(&semantic, &request);
    let first = resolved
        .first()
        .expect("the governed registry resolves at least one provider")
        .clone();

    // A receipt that under-reports, duplicates, or drops resolved provenance is
    // not the provenance the installed registry resolves for this program.
    for providers in [
        Vec::new(),
        vec![first.clone()],
        resolved
            .iter()
            .cloned()
            .chain(std::iter::once(first))
            .collect(),
    ] {
        assert_eq!(
            build_artifact_plan(
                &semantic, &request, &scheduled, &kernels, &program, providers,
            ),
            Err(ProgramError::Structure {
                rule: "artifact-provider-coverage",
            })
        );
    }

    let plan = build_artifact_plan(
        &semantic,
        &request,
        &scheduled,
        &kernels,
        &program,
        resolved.clone(),
    )
    .unwrap();
    let mut forged = plan.clone();
    forged.routing_guard = HostExprId(6);
    assert_eq!(
        verify_artifact_plan(
            &forged,
            &semantic,
            &request,
            &scheduled,
            &kernels,
            &program,
            resolved.clone(),
        ),
        Err(ProgramError::Structure {
            rule: "artifact-receipt",
        })
    );

    // A program from the other strategy is not the artifact's expected program.
    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_fused_kernel_program(&semantic, &request, &fused_region).unwrap();
    assert_eq!(
        build_artifact_plan(&semantic, &request, &scheduled, &kernels, &fused, resolved),
        Err(ProgramError::Structure {
            rule: "artifact-program-refinement",
        })
    );
}
