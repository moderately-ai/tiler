//! Tests for the compiler-owned target, budget, and artifact layers.
//!
//! The stage DAG, materialized values, views, allocations, lifetimes, typed
//! dependencies, named outputs, complete semantic coverage, the ABI expression
//! arena, the applicability guard, the entry ABI, and the routing-commit
//! contract are all verified by `tiler_ir::program` and tested there; a
//! malformed one cannot be constructed here at all. These tests cover what this
//! module still owns.
//!
//! Several tests here used to forge a compiler-side copy of the ABI — a wrong
//! accessible-byte node, a binding naming the wrong value, a routing step
//! permitting fallback after commit — and assert that
//! [`verify_kernel_program_layers`] caught it. Those copies no longer exist:
//! the subjects moved into the opaque verified program, where the equivalent
//! malformations are rejected at construction. Their successors are
//! `tiler_ir::program::tests`'
//! `an_accessible_range_the_declared_view_contradicts_is_rejected`,
//! `a_workgroup_width_the_bound_kernel_contradicts_is_rejected`, and
//! `a_routing_commit_step_that_breaks_the_lifecycle_is_rejected_at_insertion`.
//! What remains forgeable at this layer — the target profile binding, and a
//! program paired with the schedules of another strategy — is tested below.

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
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
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
    assert_eq!(temporary.required_bytes(), 16);
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

    // The cross-stage temporary is written by the first entry and read by the
    // second, which is what makes it an intermediate rather than an interface
    // component. The role now lives on the materialized value itself.
    let stages: Vec<_> = core.stages().collect();
    assert_eq!(
        stages[0]
            .accesses()
            .nth(1)
            .expect("write access")
            .view()
            .value(),
        temporary
    );
    assert_eq!(
        stages[1]
            .accesses()
            .next()
            .expect("read access")
            .view()
            .value(),
        temporary
    );

    // Fallback is admitted only before commit, over the whole lifecycle.
    let routing = core.routing_commit_contract();
    assert_eq!(routing.len(), 3);
    assert!(routing[0].fallback_permitted);
    assert!(!routing[1].fallback_permitted);
    assert!(!routing[2].fallback_permitted);

    // The guard is a declared node of the program's own arena, not a constant
    // this layer assumes.
    assert!(
        usize::try_from(core.applicability_guard()).expect("host usize")
            < core.abi_expressions().len()
    );
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

/// The plan carries the target profile and the feasibility rules as two
/// independent identities, so neither has to be invented from the other.
///
/// The artifact layer's `TargetProfileRef` and `FeasibilityRuleSetRef` are
/// separate references because one profile can be re-assessed under new rules
/// and one rule set applies across profiles. Before the split, the plan carried
/// a single key-and-version pair whose key named the profile and whose version
/// named the rules, so an assembler had no rule-set key to record and would have
/// had to name the rules after the profile.
#[test]
fn the_plan_names_its_target_profile_and_its_feasibility_rules_separately() {
    let (semantic, request, scheduled) = fixture();
    let program = build_kernel_program(&semantic, &request, &scheduled).unwrap();
    let kernels: Vec<_> = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<_, _>>()
        .unwrap();
    let artifact = build_artifact_plan(
        &semantic,
        &request,
        &scheduled,
        &kernels,
        &program,
        resolved_providers(&semantic, &request),
    )
    .unwrap();

    // The profile half: a governed key and the exact descriptor of the profile
    // the plan was assessed against.
    assert_eq!(
        artifact.target_profile.profile_key(),
        request.target_profile().profile_key()
    );
    assert!(!artifact.target_profile_descriptor().is_empty());

    // The rule set half: its own governed key and a nonzero revision, neither
    // recoverable from the profile's.
    let rules = artifact.feasibility_rule_set();
    assert!(!rules.key().is_empty());
    assert_ne!(rules.key(), artifact.target_profile.profile_key().as_str());
    assert!(rules.revision() > 0);
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
fn compiler_layers_recheck_the_target_and_the_planned_launch() {
    let (semantic, request, scheduled) = fixture();
    let valid = build_kernel_program(&semantic, &request, &scheduled).unwrap();

    let mut wrong_target = valid.clone();
    wrong_target.target_profile =
        crate::request::TargetProfile::governed_without_numerical_declarations();
    assert_eq!(
        verify_kernel_program_layers(&wrong_target, &request, &scheduled),
        Err(ProgramError::Structure {
            rule: "target-profile",
        })
    );

    // A program paired with fewer schedules than it has stages: the stage/region
    // correspondence this layer verifies does not exist.
    assert_eq!(
        verify_kernel_program_layers(&valid, &request, &scheduled[..1]),
        Err(ProgramError::Structure {
            rule: "cardinality",
        })
    );

    // The fused program's single stage launches over the *output* extent, so
    // pairing it with the two-stage strategy's pointwise region — which is
    // planned over the input extent — must be caught as a launch disagreement
    // rather than accepted because both are individually verified.
    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_fused_kernel_program(&semantic, &request, &fused_region).unwrap();
    assert_eq!(
        verify_kernel_program_layers(&fused, &request, &scheduled[..1]),
        Err(ProgramError::Abi {
            rule: "launch-expression",
            stage: StageId(0),
        })
    );
}

#[test]
fn host_expression_overflow_is_a_hard_failure() {
    // The program's own arena cannot be forged, so the overflow is exercised on
    // the shared evaluator this layer wraps: a checked multiply that leaves the
    // 64-bit domain must be a typed failure at the exact node, never a wrapped
    // byte count that a later binding would silently accept.
    let overflowing = vec![
        ExprNode::Root(AbiRoot::UnsignedLiteral(u64::MAX)),
        ExprNode::Root(AbiRoot::UnsignedLiteral(2)),
        ExprNode::Binary {
            op: AbiBinaryOp::CheckedMultiply,
            left: 0,
            right: 1,
        },
    ];
    assert_eq!(
        evaluate_expressions(&overflowing),
        Err(ProgramError::HostExpression {
            rule: "overflow",
            expression: HostExprId(2),
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
    forged.applicability_guard = forged.applicability_guard.wrapping_add(1);
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
