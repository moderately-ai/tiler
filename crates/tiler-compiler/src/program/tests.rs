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
use crate::request::{CompilationRequest, verify_planned_request};

/// The two-stage materialized assembly, spelled as a cover of two regions
/// states it.
///
/// **Stated rather than derived, and the trade is deliberate.** The compile path
/// reaches [`CoverAssembly::from_plan`] and nothing else, so the derivation is
/// exercised by every compiled program in `pipeline::tests` and
/// `pipeline::conformance`. These tests are about the *assembler* — what it
/// builds from a description and what it refuses — so they say the description
/// out loud: one value materialized across the boundary between the prologue and
/// the fold, and one named output the fold publishes.
fn materialized_assembly(
    request: &VerifiedTargetRequest,
    scheduled: &[VerifiedScheduledRegion],
) -> CoverAssembly {
    let subject = request.serial_sum();
    CoverAssembly::stated(
        scheduled.to_vec(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
            },
            AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![AssemblyBinding::Internal(0), AssemblyBinding::Internal(1)],
            },
        ],
        Vec::new(),
        vec![(subject.output_key.clone(), 1)],
    )
    .expect("the two-region assembly is well formed")
}

/// The one-stage fused assembly: no materialization, one named output.
fn fused_assembly(
    request: &VerifiedTargetRequest,
    scheduled: &VerifiedScheduledRegion,
) -> CoverAssembly {
    let subject = request.serial_sum();
    CoverAssembly::stated(
        vec![scheduled.clone()],
        vec![(subject.output_shape.clone(), ValueRole::Output)],
        vec![AssemblyStage {
            coverage: subject.members.all(),
            bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
        }],
        Vec::new(),
        vec![(subject.output_key.clone(), 0)],
    )
    .expect("the one-region assembly is well formed")
}

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
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let scheduled = build_scheduled_regions(&request).unwrap();
    (semantic, request, scheduled)
}

#[test]
fn stage_coverage_uses_verified_canonical_receipt_occurrences() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let semantic = builder.build().unwrap();
    let request = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = request.for_target(request.target_profiles()[0]).unwrap();
    let lowering = crate::lowering::resolve_lowering(&semantic, &request).unwrap();
    let member = crate::region::SemanticMemberId(0);
    let occurrence = lowering.occurrence(member).unwrap();
    let covered = covered(&[member], &lowering).unwrap();
    let [record] = covered.as_slice() else {
        panic!("one member covers one occurrence")
    };
    assert_ne!(
        record.occurrence(),
        tiler_ir::program::SemanticOccurrence::new(member.0),
        "the storage member ordinal and the canonical occurrence differ here",
    );
    // The record is the receipt's own, not an occurrence looked up beside an
    // identity: the two halves must not be separately derivable.
    assert_eq!(record, &occurrence.covered_occurrence());
}

#[test]
fn artifact_construction_rejects_a_cross_program_semantic_request_mix() {
    let (_, request, scheduled) = fixture_with_scale(2.0_f32.to_bits());
    let (different_semantic, _, _) = fixture_with_scale(3.0_f32.to_bits());
    let (semantic, _, _) = fixture_with_scale(2.0_f32.to_bits());
    let assembly = materialized_assembly(&request, &scheduled);
    let program = build_kernel_program(&semantic, &request, &assembly).unwrap();

    assert_eq!(
        build_artifact_plan(
            &different_semantic,
            &request,
            &assembly,
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
        build_kernel_program(&different_semantic, &request, &assembly),
        Err(ProgramError::Structure {
            rule: "semantic-request-binding",
        })
    );
}

#[test]
fn two_stage_program_has_explicit_temporary_abi_and_routing_commit() {
    let (semantic, request, scheduled) = fixture();
    let assembly = materialized_assembly(&request, &scheduled);
    let program = build_kernel_program(&semantic, &request, &assembly).unwrap();
    let kernels = [
        lower_structured_kernel(&scheduled[0]).unwrap(),
        lower_structured_kernel(&scheduled[1]).unwrap(),
    ];
    assert_kernels_match_program(&request, &scheduled, &program, &kernels).unwrap();
    verify_semantic_output_type(&semantic).unwrap();
    let artifact = build_artifact_plan(
        &semantic,
        &request,
        &assembly,
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
    let assembly = materialized_assembly(&request, &scheduled);
    let program = build_kernel_program(&semantic, &request, &assembly).unwrap();
    let kernels: Vec<_> = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<_, _>>()
        .unwrap();
    let artifact = build_artifact_plan(
        &semantic,
        &request,
        &assembly,
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
    let assembly = materialized_assembly(&request, &scheduled);
    let first = build_kernel_program(&semantic, &request, &assembly).unwrap();
    let second = build_kernel_program(&semantic, &request, &assembly).unwrap();
    assert_eq!(
        first.core().canonical_identity().as_bytes(),
        second.core().canonical_identity().as_bytes()
    );

    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_kernel_program(
        &semantic,
        &request,
        &fused_assembly(&request, &fused_region),
    )
    .unwrap();
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
    let valid = build_kernel_program(
        &semantic,
        &request,
        &materialized_assembly(&request, &scheduled),
    )
    .unwrap();

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
    let fused = build_kernel_program(
        &semantic,
        &request,
        &fused_assembly(&request, &fused_region),
    )
    .unwrap();
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

/// **The obligations the assembler constructs are proven by the shared builder,
/// and every one of them can say no.**
///
/// Each perturbation below states one obligation wrongly and watches
/// [`tiler_ir::program::KernelProgramBuilder::build`] — or the compiler layer
/// immediately above it — refuse. The assembly they perturb is the one the
/// production route builds for a two-region cover, so a check that stopped
/// firing here would be a check that stopped firing on the compile path.
#[test]
fn the_assembled_obligations_are_refused_when_stated_wrongly() {
    let (semantic, request, scheduled) = fixture();
    let subject = request.serial_sum();
    let sound = materialized_assembly(&request, &scheduled);
    build_kernel_program(&semantic, &request, &sound).expect("the stated assembly assembles");

    // The fold reads the tensor the prologue never wrote: the temporary is now
    // written by nobody and read by the fold, which is an uninitialized read the
    // whole-program verifier refuses.
    let unwritten = CoverAssembly::stated(
        scheduled.clone(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(1)],
            },
            AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![AssemblyBinding::Internal(0), AssemblyBinding::Internal(2)],
            },
        ],
        Vec::new(),
        vec![(subject.output_key.clone(), 2)],
    );
    assert_eq!(
        unwritten.unwrap_err().rule(),
        "internal-unwritten",
        "a value nothing writes was described rather than refused"
    );

    // The prologue materializes a value the fold never reads. Nothing downstream
    // would refuse this — the whole-program verifier requires a writer for every
    // value and a dependency behind every cross-stage read, and a temporary with
    // no reader violates neither — so the program would allocate and fill a
    // buffer for no consumer.
    let unread = CoverAssembly::stated(
        scheduled.clone(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
            },
            AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(1)],
            },
        ],
        Vec::new(),
        vec![(subject.output_key.clone(), 1)],
    );
    assert_eq!(
        unread.unwrap_err().rule(),
        "materialized-value-unread",
        "a value the cover materializes for nobody was described rather than refused"
    );

    // The fold covers the occurrences the prologue already claims, which is the
    // double coverage `KernelProgramBuilder::build` exists to refuse.
    let double = CoverAssembly::stated(
        scheduled.clone(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
            },
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Internal(0), AssemblyBinding::Internal(1)],
            },
        ],
        Vec::new(),
        vec![(subject.output_key.clone(), 1)],
    )
    .expect("the description itself is well formed");
    assert!(
        build_kernel_program(&semantic, &request, &double).is_err(),
        "a stage claiming occurrences another stage covers was admitted"
    );

    // The program publishes nothing, so the semantic interface the builder was
    // opened against is left uncovered.
    //
    // The diagnostic is `EmptyProgram` rather than `MissingNamedOutput` because
    // this fixture declares exactly one named output, and publishing *fewer*
    // outputs than declared while publishing at least one needs a program
    // declaring two. `MissingNamedOutput` is proven reachable in
    // `tiler_ir::program::tests`; reaching it from a compiler-built assembly
    // needs a described plan that drops one of two attributed outputs, which
    // `CoverAssembly::from_plan` refuses one layer earlier under
    // `cover-named-output-attribution` — so the state it names stays this
    // fixture's neighbour rather than its own row.
    let unpublished = CoverAssembly::stated(
        scheduled.clone(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.output_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
            },
            AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![AssemblyBinding::Internal(0), AssemblyBinding::Internal(1)],
            },
        ],
        Vec::new(),
        Vec::new(),
    )
    .expect("the description itself is well formed");
    assert_eq!(
        build_kernel_program(&semantic, &request, &unpublished),
        Err(ProgramError::CoreVerification(
            tiler_ir::program::KernelProgramDiagnostic::EmptyProgram
        )),
        "a program publishing no declared output was admitted"
    );

    // One allocation sized by the wrong value: the fold's result is described at
    // the contributor extent, so the view it addresses is not the extent its
    // kernel's write buffer declares.
    let mis_sized = CoverAssembly::stated(
        scheduled.clone(),
        vec![
            (subject.input_shape.clone(), ValueRole::Temporary),
            (subject.input_shape.clone(), ValueRole::Output),
        ],
        vec![
            AssemblyStage {
                coverage: subject.members.pointwise().to_vec(),
                bindings: vec![AssemblyBinding::Input(0), AssemblyBinding::Internal(0)],
            },
            AssemblyStage {
                coverage: subject.members.reduction().to_vec(),
                bindings: vec![AssemblyBinding::Internal(0), AssemblyBinding::Internal(1)],
            },
        ],
        Vec::new(),
        vec![(subject.output_key.clone(), 1)],
    )
    .expect("the description itself is well formed");
    assert!(
        matches!(
            build_kernel_program(&semantic, &request, &mis_sized),
            Err(ProgramError::CoreConstruction(_))
        ),
        "a value sized by the wrong extent was admitted"
    );
}

#[test]
fn a_fused_program_binds_one_stage_and_no_cross_stage_value() {
    let (semantic, request, scheduled) = fixture();
    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_kernel_program(
        &semantic,
        &request,
        &fused_assembly(&request, &fused_region),
    )
    .unwrap();
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
    let assembly = materialized_assembly(&request, &scheduled);
    let program = build_kernel_program(&semantic, &request, &assembly).unwrap();
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
                &semantic, &request, &assembly, &kernels, &program, providers,
            ),
            Err(ProgramError::Structure {
                rule: "artifact-provider-coverage",
            })
        );
    }

    let plan = build_artifact_plan(
        &semantic,
        &request,
        &assembly,
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
            &assembly,
            &kernels,
            &program,
            resolved.clone(),
        ),
        Err(ProgramError::Structure {
            rule: "artifact-receipt",
        })
    );

    // A program from the other strategy is not the artifact's expected program,
    // and the receipt path finds that by re-deriving through the same route the
    // build path took rather than by matching the schedule count.
    let fused_region = build_fused_scheduled_region(&request).unwrap();
    let fused = build_kernel_program(
        &semantic,
        &request,
        &fused_assembly(&request, &fused_region),
    )
    .unwrap();
    assert_eq!(
        build_artifact_plan(&semantic, &request, &assembly, &kernels, &fused, resolved),
        Err(ProgramError::Structure {
            rule: "artifact-program-refinement",
        })
    );
}

/// **Stage order comes from the cover's materialization edges, never from a
/// region identifier.**
///
/// The identifiers are constants of the schedule vocabulary — every elementwise
/// region carries `RegionId::new(0)` whichever occurrences it covers — so the
/// ordering the retired assembler used returns an arbitrary order the moment a
/// cover places two regions the same builder produced. This asserts the property
/// that actually has to hold, over every legal cover the governed program
/// enumerates: each edge's producer is dispatched before every one of its
/// consumers.
///
/// The population is counted rather than assumed, and the count of covers
/// carrying at least one edge is asserted separately: a run over covers that all
/// happened to have no edges would satisfy the property vacuously, and would be
/// indistinguishable from the check not running.
#[test]
fn the_execution_order_places_every_producer_before_its_consumers() {
    let (semantic, request, _) = fixture();
    let formation = crate::region::form_region_candidates(
        &semantic,
        request.budgets(),
        request.numerical_contract(),
    )
    .expect("the fixture forms regions");
    let enumeration = crate::cover::enumerate_covers(
        &semantic,
        request.budgets(),
        &formation,
        crate::cover::CoverPolicy::governed(request.numerical_contract()),
    )
    .expect("the fixture enumerates covers");

    let mut with_edges = 0_usize;
    let mut reordered = 0_usize;
    for cover in enumeration.covers() {
        let order = crate::program::execution_order(cover).expect("a legal cover is acyclic");
        assert_eq!(order.len(), cover.regions().len());
        let position = |occurrence: &crate::region::RegionOccurrenceIdentity| {
            let region = cover
                .regions()
                .iter()
                .position(|placed| placed.occurrence() == occurrence)
                .expect("an edge names a placed region");
            order
                .iter()
                .position(|placed| *placed == region)
                .expect("every placed region is ordered")
        };
        if !cover.materializations().is_empty() {
            with_edges += 1;
        }
        if order != (0..cover.regions().len()).collect::<Vec<_>>() {
            reordered += 1;
        }
        for edge in cover.materializations() {
            let producer = position(edge.producer());
            for consumer in edge.consumers() {
                assert!(
                    producer < position(consumer),
                    "a consumer is dispatched before the region that materializes its input"
                );
            }
        }
    }
    assert!(
        with_edges > 0,
        "no enumerated cover materializes anything, so the ordering property is vacuous"
    );
    // Stated rather than asserted: whether any of this program's covers needs
    // reordering out of canonical occurrence order depends on identity digests,
    // so a run where none does is a fact about the fixture and not a defect. The
    // property above holds either way, and this records which case was observed.
    assert!(reordered <= enumeration.covers().len());
}

/// The balanced split is exact, and its degenerate inputs are refused.
#[test]
fn the_governed_split_covers_its_contributor_sequence_exactly() {
    for contributors in [4_u64, 6, 8, 9, 12, 16, 36] {
        let partition = crate::physical::governed_partition(contributors)
            .unwrap_or_else(|| panic!("{contributors} admits a balanced split"));
        assert_eq!(partition.total_contributors(), Some(contributors));
        assert!(partition.partitions >= 2);
        assert!(partition.contributors_per_partition >= 2);
        assert!(partition.covers(contributors));
    }
    // Nothing to split, and nothing splittable: a prime extent has no exact
    // split whose partitions each fold more than one contributor, so the
    // proposal is withheld rather than offered as a dispatch that does no work.
    for contributors in [0_u64, 1, 2, 3, 5, 7, 11, 13] {
        assert_eq!(crate::physical::governed_partition(contributors), None);
    }
}

/// A split proposed under a contract that forbids reassociation is refused.
#[test]
fn a_split_under_a_reassociation_forbidding_contract_is_refused() {
    let (_, request, _) = fixture();
    // The `[2, 2]` fixture reduces two contributors, which the balanced rule
    // withholds a split for, so the split under test is stated directly: the
    // subject here is the contract, not the choice of partition.
    let partition = tiler_ir::schedule::ContributorPartition {
        partitions: 2,
        contributors_per_partition: 1,
    };
    let (partial, members) =
        crate::physical::partial_reduction_region(&request, request.sole_output(), partition)
            .expect("a partial pass");
    // The governed strict contract forbids reassociation, so the region the
    // constructor produces carries `permits_reassociation: false` and the
    // schedule verifier refuses it rather than costing it.
    assert_eq!(
        crate::physical::verify_schedule(partial, members, &request),
        Err(crate::physical::PhysicalError::Intrinsic {
            rule: "numerical-or-access-refinement",
            region: crate::physical::RegionId::new(2),
        })
    );
}

/// Builds a two-output program and returns it with its declared value ordinals.
///
/// `product = a * b` and `sum = a + b`: two ordered named outputs over two
/// declared inputs, whose producing occurrences are disjoint. It is the smallest
/// program for which "which region publishes which named output" has more than
/// one answer, which is the whole subject of [`attribute_named_outputs`].
fn two_output_attribution_fixture() -> (SemanticProgram, Vec<SemanticValueId>) {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, a, b).unwrap();
    let sum = F32Add::apply(&mut builder, a, b).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    let program = builder.build().unwrap();
    let ordinals = program
        .outputs()
        .map(|output| {
            crate::region::value_ordinal(&program, output.value())
                .expect("a declared output names a value the program holds")
        })
        .collect();
    (program, ordinals)
}

/// The named-output attribution refuses in every direction it can be wrong in.
///
/// **Each row is driven against a case that must fail, because none of them is
/// reachable through the request boundary today** — `verify_cover` proves each
/// named output is produced by exactly one placed region, and
/// `physical::spell_region` declines a region straddling two outputs' recognized
/// partitions before it can be proposed. Stating the inputs directly is what
/// makes the arms drivable at all, and it is the same trade
/// [`CoverAssembly::stated`] makes for the assembler itself.
///
/// The accepted neighbour comes first and every row differs from it by exactly
/// one fact, so a row that stopped failing would be reporting about the
/// perturbation rather than about the check.
#[test]
fn named_output_attribution_can_say_no_in_every_direction() {
    let (program, [product, sum]) = {
        let (program, ordinals) = two_output_attribution_fixture();
        let [product, sum] = ordinals.as_slice() else {
            panic!("the fixture declares two outputs");
        };
        (program, [*product, *sum])
    };
    let publishes_both = [false, false];

    // The neighbour: one region per declared output, each retaining its own,
    // neither materializing. Declaration order is `product` then `sum`, and the
    // regions are stated in the opposite order — so a correct attribution
    // answers `[1, 0]` and a positional one would answer `[0, 1]`.
    assert_eq!(
        attribute_named_outputs(&program, &[&[sum], &[product]], &publishes_both),
        Ok(vec![1, 0]),
    );

    // No region retains `sum`, so nothing writes what the interface names.
    assert_eq!(
        attribute_named_outputs(&program, &[&[product], &[]], &publishes_both),
        Err(AttributionFailure::Unattributed { output: 1 }),
    );

    // Two regions retain `product`: the cover would have two writers for one
    // destination.
    assert_eq!(
        attribute_named_outputs(&program, &[&[product], &[product, sum]], &publishes_both),
        Err(AttributionFailure::Ambiguous {
            output: 0,
            region: 1
        }),
    );

    // One region retains both, so its one owning write would publish twice.
    assert_eq!(
        attribute_named_outputs(&program, &[&[product, sum], &[]], &publishes_both),
        Err(AttributionFailure::Shared { region: 0 }),
    );

    // The region retaining `sum` also materializes an edge, so its owning write
    // is already spoken for.
    assert_eq!(
        attribute_named_outputs(&program, &[&[sum], &[product]], &[true, false]),
        Err(AttributionFailure::MaterializesAndPublishes { region: 0 }),
    );

    // A third region materializes nothing and publishes nothing, so its owning
    // write has no destination the program's interface names.
    assert_eq!(
        attribute_named_outputs(&program, &[&[product], &[sum], &[]], &[false, false, false]),
        Err(AttributionFailure::Unpublished { region: 2 }),
    );
    // The same three regions with the third materializing instead: the converse
    // check reads the write's destination and not merely the absence of a
    // declared output.
    assert_eq!(
        attribute_named_outputs(&program, &[&[product], &[sum], &[]], &[false, false, true]),
        Ok(vec![0, 1]),
    );
}
