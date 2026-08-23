use super::support::{reduction_frontier, semantic_case_with_axis, split_request};
use super::*;

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
    // The single-workgroup tree is withheld before a region exists: the
    // prototype baseline declares no qualified width policy. That is a
    // strategy decline, not a local-memory feasibility refusal — the latter
    // is driven against a profile that *does* declare the policy. The split
    // and the serial alternative stay untouched.
    assert!(
        matches!(
            frontier.rejections(),
            [crate::frontier::FrontierRejection::StrategyDeclined {
                strategy: "tiler.reduction.single-workgroup-tree",
                cause: crate::frontier::StrategyDeclineCause::TargetPolicyUndeclared {
                    policy: "qualified-width-policy-undeclared",
                },
                ..
            }]
        ),
        "the split request's rejections are not the tree's policy decline: {:?}",
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
    // win by pruning. **Preference landed under
    // `activate-measured-reduction-selection-from-a-target-cost-row`**, and this
    // assertion is what its design rests on: because the split is *dominated*
    // rather than merely non-dominated, a measured term confined to breaking ties
    // inside the non-dominated set could not have preferred it at any shape.
    // `the_parallel_reduction_plans_are_structurally_dominated` states the same
    // fact at plan level, which is where selection reads it.
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
