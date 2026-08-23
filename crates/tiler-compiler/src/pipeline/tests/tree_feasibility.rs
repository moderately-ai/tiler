use super::support::{reduction_frontier, tree_request, tree_target};
use super::*;

/// A second profile that omits the policy offers no tree and does not inherit
/// `256` or the balanced partition.
///
/// At 8,192 contributors the qualified rule takes 256 participants and
/// `governed_partition` takes 128. Neither width may appear when the policy is
/// absent: the strategy is withheld, not retuned.
#[test]
fn omitting_the_width_policy_offers_no_tree_and_does_not_inherit_a_width() {
    let (_, request) = tree_request(
        Shape::from_dims([1, 8_192]),
        TargetProfile::workgroup_tree_target_without_width_policy_for_test(
            32_768,
            1 << 24,
            Some(crate::target::SynchronizationSupport::Realized),
        ),
    );
    assert_eq!(
        crate::physical::single_workgroup_tree_region(
            &request,
            request.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
        .err(),
        Some(crate::physical::WorkgroupTreeUnavailable::QualifiedWidthPolicyUndeclared)
    );
    let frontier = reduction_frontier(&request);
    assert!(
        frontier.rejections().iter().any(|rejection| matches!(
            rejection,
            crate::frontier::FrontierRejection::StrategyDeclined {
                strategy: "tiler.reduction.single-workgroup-tree",
                cause: crate::frontier::StrategyDeclineCause::TargetPolicyUndeclared { .. },
                ..
            }
        )),
        "the omitted-policy profile still offered or otherwise declined the tree: {:?}",
        frontier.rejections()
    );
    for admitted in frontier.admitted() {
        if let Some(region) = admitted.scheduled() {
            assert_ne!(
                region.region().schedule.threads_per_workgroup,
                256,
                "omitting the policy inherited the private 256 width"
            );
            assert_ne!(
                region.region().schedule.threads_per_workgroup,
                128,
                "omitting the policy substituted the balanced partition"
            );
        }
    }
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
    let (region, members) = crate::physical::single_workgroup_tree_region(
        &request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("a reassociating eight-contributor request admits the tree");
    // The control: the tile the strategy actually emits verifies.
    assert!(
        crate::physical::verify_schedule(
            region.clone(),
            members.clone(),
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        )
        .is_ok()
    );

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
        crate::physical::verify_schedule(
            divergent,
            members,
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        ),
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
    let (region, members) = crate::physical::single_workgroup_tree_region(
        &request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("a reassociating eight-contributor request admits the tree");
    // The control: unperturbed, it binds.
    assert!(
        crate::physical::verify_schedule(
            region.clone(),
            members.clone(),
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        )
        .is_ok()
    );

    // A region ordinal the tree does not own. Two strategies sharing one ordinal
    // would make the program's region correlation ambiguous.
    let mut forged = region.clone();
    forged.index.id = RegionId::new(1);
    assert!(matches!(
        crate::physical::verify_schedule(
            forged,
            members.clone(),
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        ),
        Err(crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            ..
        })
    ));

    // Claiming the prologue's occurrences as well as the reduction's, which
    // would double-cover the graph.
    let forged_members = request.serial_sum().members.all();
    assert!(matches!(
        crate::physical::verify_schedule(
            region.clone(),
            forged_members,
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        ),
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
        crate::physical::verify_schedule(
            forged,
            members,
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        ),
        Err(crate::physical::PhysicalError::Intrinsic { .. })
    ));
}
