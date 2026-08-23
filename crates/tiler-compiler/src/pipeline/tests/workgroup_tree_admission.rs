use super::support::{
    materialized_assembly, reduction_frontier, semantic_case_with_axis, tree_request, tree_target,
};
use super::*;

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
/// count while its output count is one. `crate::program`'s `verify_entry` and the shared
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
    let (tree, members) = crate::physical::single_workgroup_tree_region(
        &request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("the tree is available");
    let tree = crate::physical::verify_schedule(
        tree,
        members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the tree verifies");
    // The tree replaces the reduction of the materialized pair; its prologue is
    // the ordinary pointwise stage, which is what makes the two stages' launches
    // differ in both quantities inside one program.
    let serial = crate::physical::build_scheduled_regions(
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the serial pair");
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
/// Five causes, five distinct outcomes, and the point of driving them together
/// is that none of them is a cost and none of them is the same answer as another:
/// a withheld permission is decided from the contract before a region exists, a
/// missing width policy withholds the strategy before a region exists, a
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

    // A missing width policy: decided from the profile, before any region is
    // built, and it must not substitute `256` or the balanced partition.
    let (_, undeclared) = tree_request(
        Shape::from_dims([1, 8]),
        TargetProfile::workgroup_tree_target_without_width_policy_for_test(
            256,
            1_024,
            Some(crate::target::SynchronizationSupport::Realized),
        ),
    );
    assert_eq!(
        crate::physical::single_workgroup_tree_region(
            &undeclared,
            undeclared.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
        .err(),
        Some(crate::physical::WorkgroupTreeUnavailable::QualifiedWidthPolicyUndeclared),
        "omitting the policy must not offer a tree"
    );
    assert!(
        reduction_frontier(&undeclared)
            .rejections()
            .iter()
            .any(|rejection| matches!(
                rejection,
                crate::frontier::FrontierRejection::StrategyDeclined {
                    strategy: "tiler.reduction.single-workgroup-tree",
                    cause: crate::frontier::StrategyDeclineCause::TargetPolicyUndeclared {
                        policy: "qualified-width-policy-undeclared",
                    },
                    ..
                }
            )),
        "omitting the policy left the missing tree unexplained: {:?}",
        reduction_frontier(&undeclared).rejections()
    );
    assert!(
        !reduction_frontier(&undeclared)
            .rejections()
            .iter()
            .any(|rejection| matches!(
                rejection,
                crate::frontier::FrontierRejection::StrategyDeclined {
                    cause: crate::frontier::StrategyDeclineCause::NoAdmissibleShape { .. },
                    ..
                }
            )),
        "omitting the policy must not fall back to the balanced decline set: {:?}",
        reduction_frontier(&undeclared).rejections()
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
