use super::support::{reduction_frontier, tree_request, tree_target};
use super::*;

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
        let (region, members) = crate::physical::single_workgroup_tree_region(
            &request,
            request.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
        .expect("a reassociating request admits the tree at this extent");
        let tiler_ir::schedule::ReductionTopology::CooperativeWorkgroup { coverage, .. } =
            &region.schedule.reduction
        else {
            panic!("the tree region carries a cooperative topology")
        };
        let partition = coverage.partition();
        assert_eq!(partition.partitions, participants, "extent {extent}");
        assert_eq!(
            partition.contributors_per_partition, contributors_per_partition,
            "extent {extent}"
        );
        let verified = crate::physical::verify_schedule(
            region,
            members,
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test(),
        )
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
            crate::physical::single_workgroup_tree_region(
                &request,
                request.sole_output(),
                crate::physical::RegionWrite::ProgramOutput,
            )
            .err(),
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
        crate::physical::single_workgroup_tree_region(
            &prime,
            prime.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
        .err(),
        Some(
            crate::physical::WorkgroupTreeUnavailable::NoAdmissibleParticipantCount {
                contributors: 7,
            }
        )
    );
}

/// The tree's width is the measured cap's choice, and the split's is not.
///
/// **What this pins.** `single_workgroup_tree_region` reads
/// `capped_tree_partition` — the admissible participant count nearest the
/// measured 256 — while `split_reduction_regions` keeps
/// `governed_partition`'s balanced exact split. At 8,192 contributors those are
/// 256 participants folding 32 each against 128 folding 64, so the assertion
/// separates the two rules rather than restating one of them. The count is read
/// from the region's *own* cooperative topology, not from the function under
/// test, so a call site that reverted to the balanced rule fails here even
/// though both rules would still return a legal partition.
///
/// **Watched failing.** Pointing `single_workgroup_tree_region` back at
/// `governed_partition(contributors)` fails the first assertion with "the tree
/// did not take the capped participant count", left 128 against right 256 —
/// the balanced choice arriving where the capped one belongs.
///
/// **The decline set is pinned beside it, because the width rule must not narrow
/// the domain.** A rule that refused where the balanced one admitted would
/// withhold a legal alternative on a cost heuristic. The two rules agree about
/// which extents admit a participant count across every contributor count below
/// 4,096 — 3,530 of them admit one — and both that population and the number
/// of counts at which the two *choices* diverge are asserted, so neither a loop
/// that ran over nothing nor a width rule that quietly became the balanced rule
/// again can look green. The width *window* the rule may choose within is
/// asserted here and its lower edge is the subject of
/// [`the_tree_widens_toward_the_cap_rather_than_truncating_at_it`].
#[test]
fn the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs() {
    const CONTRIBUTORS: u64 = 8_192;
    // Not `tree_target()`: staging is one `f32` slot per participant, so the
    // capped 256 needs 1,024 bytes where that profile declares 256. The wider
    // width costs more workgroup memory than the balanced one and the profile
    // is what decides whether that is affordable — the authoritative Apple9
    // declaration carries 32,768, so the cap is well inside the row the
    // calibration measured against.
    let (_, request) = tree_request(
        Shape::from_dims([1, CONTRIBUTORS]),
        TargetProfile::workgroup_tree_target_for_test(
            1_024,
            1_024,
            Some(crate::target::SynchronizationSupport::Realized),
        ),
    );
    let (region, members) = crate::physical::single_workgroup_tree_region(
        &request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("a reassociating request admits the tree at this extent");
    let tiler_ir::schedule::ReductionTopology::CooperativeWorkgroup { coverage, .. } =
        &region.schedule.reduction
    else {
        panic!("the tree region carries a cooperative topology")
    };
    let partition = coverage.partition();
    assert_eq!(
        partition.partitions, 256,
        "the tree did not take the capped participant count"
    );
    assert_eq!(partition.contributors_per_partition, 32);
    assert!(partition.covers(CONTRIBUTORS));
    assert_eq!(
        region.schedule.threads_per_workgroup, 256,
        "the declared width did not follow the participant count"
    );

    // The multi-pass split is deliberately untouched, so the two strategies now
    // declare different groupings at this count. Asserting the difference is
    // what makes the value above the tree's rule rather than a rule they share.
    let balanced = crate::physical::governed_partition(CONTRIBUTORS)
        .expect("8,192 contributors admit a balanced exact split");
    assert_eq!(balanced.partitions, 128);
    assert_eq!(balanced.contributors_per_partition, 64);
    assert_ne!(partition.partitions, balanced.partitions);

    // The region still verifies at the wider width: the cap chooses among the
    // participant counts the schedule admits and does not reach past them.
    crate::physical::verify_schedule(
        region,
        members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the capped tree region verifies");

    // Same domain, different choice. Counted, and the disagreement counted too,
    // so neither half can pass by being empty.
    let mut admitted = 0_u32;
    let mut differing = 0_u32;
    for contributors in 0..4_096_u64 {
        let capped = crate::physical::capped_tree_partition(contributors);
        let governed = crate::physical::governed_partition(contributors);
        assert_eq!(
            capped.is_some(),
            governed.is_some(),
            "the cap moved the tree's decline set at {contributors} contributors"
        );
        if let Some(capped) = capped {
            admitted += 1;
            assert!(capped.covers(contributors));
            assert!(capped.partitions >= 2);
            assert!(capped.contributors_per_partition >= 2);
            // The window the rule may choose within: never wider than 509, which
            // is what `2 * cap - 2` allows and what keeps the preference inside
            // every width authority downstream.
            assert!(
                capped.partitions <= 2 * crate::physical::MEASURED_TREE_PARTICIPANT_CAP - 2,
                "{contributors} contributors chose a width outside the rule's window"
            );
            if capped != governed.expect("the domains agree") {
                differing += 1;
            }
        }
    }
    assert_eq!(admitted, 3_530, "the admitting population moved");
    assert_eq!(
        differing, 2_350,
        "the population separating the two rules moved"
    );

    assert_eq!(
        crate::physical::take_capped_tree_above_cap_candidate_checks_for_test(),
        0,
        "the below-4,096 domain sweep must not enter the above-cap search"
    );

    // The above-cap branch, which the sweep cannot reach: it needs a contributor
    // count whose smallest divisor above one already exceeds the cap, and the
    // smallest is 257 * 257 — far past the ladder. The branch is what keeps the
    // cap a preference rather than a feasibility test, so leaving it to the
    // unreachable-in-practice argument alone would leave the one piece of this
    // rule that withholds nothing untested.
    let above = crate::physical::capped_tree_partition(257 * 257)
        .expect("a composite count admits a partition however large its smallest factor");
    assert_eq!(
        above.partitions, 257,
        "the above-cap branch did not take the smallest admissible count"
    );
    assert_eq!(above.contributors_per_partition, 257);
    assert!(above.covers(257 * 257));
    assert!(
        above.partitions > crate::physical::MEASURED_TREE_PARTICIPANT_CAP,
        "this case exists to exceed the cap; a count at or below it tests the branch above"
    );
    // Still no wider than the balanced rule would have gone, which is what makes
    // exceeding the cap here safe for every width authority downstream.
    assert!(
        above.partitions
            <= crate::physical::governed_partition(257 * 257)
                .expect("the domains agree")
                .partitions,
        "the above-cap branch chose a wider count than the balanced rule"
    );
    assert_eq!(
        crate::physical::take_capped_tree_above_cap_candidate_checks_for_test(),
        1,
        "257 squared must check its one above-cap divisor candidate"
    );

    // A prime count admits nothing, but this one makes the above-cap search do
    // real work before it reaches that conclusion. Its floor square root is
    // 257, so after the lower scan finds no divisor the fallback checks 257;
    // 65,537 instead has floor square root 256 and would make that loop empty.
    let prime_reaching_above_cap_search = 66_067_u64;
    assert_eq!(
        prime_reaching_above_cap_search.isqrt(),
        crate::physical::MEASURED_TREE_PARTICIPANT_CAP + 1,
        "the prime subject must leave one candidate for the above-cap search"
    );
    assert!(
        (2..=crate::physical::MEASURED_TREE_PARTICIPANT_CAP)
            .all(|candidate| !prime_reaching_above_cap_search.is_multiple_of(candidate)),
        "the prime subject must enter the above-cap search rather than return from the lower scan"
    );
    assert_eq!(crate::physical::capped_tree_partition(65_537), None);
    assert_eq!(
        crate::physical::take_capped_tree_above_cap_candidate_checks_for_test(),
        0,
        "65,537 must leave the above-cap search empty"
    );
    assert_eq!(
        crate::physical::capped_tree_partition(prime_reaching_above_cap_search),
        None,
        "the above-cap search must exhaust its one non-dividing candidate for this prime"
    );
    assert_eq!(
        crate::physical::take_capped_tree_above_cap_candidate_checks_for_test(),
        1,
        "66,067 must check exactly one candidate in the actual above-cap loop"
    );
}

/// The capped tree's extra staging is rejected by feasibility rather than
/// erased by its partition choice.
///
/// At 8,192 contributors, the capped tree has 256 participants and therefore
/// stages 1,024 bytes. A synthetic 512-byte test profile is deliberately below
/// that requirement while meeting the balanced tree's former 512-byte
/// requirement, so this is the refusal band the cap opened. The adjacent
/// 1,024-byte profile verifies the identical region, separating the band from a
/// blanket refusal.
///
/// **Watched failing.** Raising the synthetic row from 512 to 1,024 makes the
/// first verification succeed, so its `expect_err` fails; this confirms the
/// target row is the subject that reaches the typed diagnostic.
#[test]
fn the_capped_tree_refuses_the_local_memory_band_and_admits_its_neighbour() {
    const CONTRIBUTORS: u64 = 8_192;
    let tree_for = |local_memory_bytes| {
        let (_, request) = tree_request(
            Shape::from_dims([1, CONTRIBUTORS]),
            TargetProfile::workgroup_tree_target_for_test(
                local_memory_bytes,
                1_024,
                Some(crate::target::SynchronizationSupport::Realized),
            ),
        );
        let (region, members) = crate::physical::single_workgroup_tree_region(
            &request,
            request.sole_output(),
            crate::physical::RegionWrite::ProgramOutput,
        )
        .expect("the capped tree is constructible before target feasibility");
        (region, members, request)
    };

    let (region, members, request) = tree_for(512);
    assert_eq!(
        crate::physical::verify_schedule(
            region,
            members,
            &request,
            &crate::lowering::ResolvedLowering::unresolved_for_test()
        )
        .expect_err("the 512-byte profile must refuse the capped tree"),
        crate::physical::PhysicalError::Target {
            rule: "local-memory-bytes",
            region: tiler_ir::schedule::RegionId::new(4),
            required: 1_024,
            available: 512,
        }
    );

    let (region, members, request) = tree_for(1_024);
    crate::physical::verify_schedule(
        region,
        members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the same capped tree verifies once 1,024 bytes are available");
}

/// The tree's width rule bounds the *downward* direction, and pays nothing for it
/// where the calibration measured.
///
/// **Why this test exists at all.** `MEASURED_TREE_PARTICIPANT_CAP` is measured
/// in one direction only. Every shape the calibration swept has a power-of-two
/// contributor count, and on a power of two "the largest admissible count not
/// exceeding 256" is identically `min(256, contributors / 2)` — the widest count
/// the cap admits — so no measured cell could tell that formulation apart from
/// "the admissible count nearest 256". Where the divisor lattice is sparse they
/// part company sharply: at 514 (`2 * 257`) the only admissible count at or below
/// the cap is **2**, and the truncating formulation took it.
///
/// **The three properties asserted, and why each is separate.**
///
/// 1. *Agreement where the evidence is.* The seven contributor counts the
///    calibration swept keep the exact widths its leave-one-out selection scored,
///    so the 1.008 held-out regret is preserved rather than re-argued. Asserted
///    as a table so a rule that drifted at one shape cannot hide behind six.
/// 2. *The window.* A width above the cap is taken only when it is nearer to the
///    cap than every admissible width below, which forces it under `2 * 256 - 2`.
///    509 is the arithmetic consequence, not a tuned constant, and it is what
///    keeps the preference inside `MAX_COOPERATIVE_PARTICIPANTS` and inside every
///    workgroup width a profile here declares.
/// 3. *The alternative that would not have property 2.* Taking the wider of this
///    rule and `governed_partition` — the obvious candidate, and identical to
///    this rule on all seven measured counts — chooses 4,099 participants at
///    8,198 contributors, which `workgroup_tree_tile` cannot represent. The tree
///    would be *withdrawn* at a count where it is offered today, so a cost
///    preference would have decided feasibility. Asserted through the region
///    builder rather than the partition function, because the withdrawal is only
///    visible there.
///
/// **Populations are counted so nothing can pass by being empty**: below 4,096
/// there are 3,530 admitting counts and exactly 1,061 at which the chosen width
/// exceeds the cap, and the widest width reached across the whole sweep is
/// asserted exactly rather than bounded. Every one of the 1,061 is a widening
/// against the truncating rule, because a count below 4,096 cannot reach the
/// above-cap fallback branch at all — the smallest count with no divisor at or
/// below the cap is `257 * 257`.
///
/// **Watched failing.** Reverting `capped_tree_partition` to return `below`
/// without the search above the cap fails with "514 contributors truncated at the
/// cap instead of widening toward it", left 2 against right 257.
#[test]
fn the_tree_widens_toward_the_cap_rather_than_truncating_at_it() {
    // The count whose only admissible widths are 2 and an unrepresentable 4,099,
    // which is what separates this rule from the wider-of-the-two candidate.
    const SPARSE: u64 = 8_198;
    let cap = crate::physical::MEASURED_TREE_PARTICIPANT_CAP;
    let width = |contributors: u64| {
        crate::physical::capped_tree_partition(contributors).map(|partition| {
            assert!(partition.covers(contributors));
            partition.partitions
        })
    };

    // 1. The calibration's own seven contributor counts, unmoved. Two shapes of
    //    the seven share 8,192, which is why the count appears twice: the sweep
    //    scored it as two cells and the rule owes both the same width.
    for (contributors, participants) in [
        (16_u64, 8_u64),
        (32, 16),
        (2_048, 256),
        (4_096, 256),
        (8_192, 256),
        (8_192, 256),
        (16_384, 256),
    ] {
        assert_eq!(
            width(contributors),
            Some(participants),
            "the rule moved a width the calibration scored at {contributors} contributors"
        );
    }

    // The count the two formulations part company at, and the first one at which
    // an admissible width sits above the cap while everything below it is 2.
    assert_eq!(
        width(514),
        Some(257),
        "514 contributors truncated at the cap instead of widening toward it"
    );
    assert_eq!(
        crate::physical::capped_tree_partition(514)
            .expect("514 admits a partition")
            .contributors_per_partition,
        2
    );
    // The fallback branch is untouched: nothing at or below the cap divides
    // 257 * 257, and the smallest admissible count is still what it takes.
    assert_eq!(width(257 * 257), Some(257));

    // 2. The window, over a named population, with both the widened set and the
    //    direction of every move counted.
    let mut admitted = 0_u32;
    let mut widened = 0_u32;
    let mut widest = 0_u64;
    for contributors in 0..4_096_u64 {
        let Some(chosen) = width(contributors) else {
            continue;
        };
        admitted += 1;
        widest = widest.max(chosen);
        if chosen > cap {
            widened += 1;
            // Taken only because nothing at or below the cap is nearer to it.
            let below = (2..=cap.min(contributors / 2))
                .rev()
                .find(|candidate| contributors.is_multiple_of(*candidate));
            let below = below.expect("a width above the cap needs one below to be nearer than");
            assert!(
                chosen - cap < cap - below,
                "{contributors} contributors took {chosen} over the nearer {below}"
            );
            assert!(
                chosen <= 2 * cap - 2,
                "{contributors} contributors chose {chosen}, outside the window the tie-break allows"
            );
        }
    }
    assert_eq!(admitted, 3_530, "the admitting population moved");
    assert_eq!(widened, 1_061, "the widened population moved");
    assert_eq!(widest, 509, "the widest reachable width moved");

    // 3. The candidate this rule is *not*, and the reason. Both rules agree at
    //    8,198 that a partition exists; they disagree about which, and only one
    //    of the two answers survives the tile.
    let balanced = crate::physical::governed_partition(SPARSE)
        .expect("8,198 contributors admit a balanced exact split");
    assert_eq!(balanced.partitions, 4_099);
    assert_eq!(
        width(SPARSE),
        Some(2),
        "the rule left the representable width"
    );
    assert!(
        tiler_ir::schedule::workgroup_tree_tile(balanced.partitions).is_none(),
        "8,198 was chosen because the balanced width is unrepresentable; it no longer is"
    );
    assert!(tiler_ir::schedule::workgroup_tree_tile(2).is_some());
    // And the tree really is still offered there, which is the property the
    // wider-of-the-two candidate would have cost.
    let (_, request) = tree_request(
        Shape::from_dims([1, SPARSE]),
        TargetProfile::workgroup_tree_target_for_test(
            1_024,
            1_024,
            Some(crate::target::SynchronizationSupport::Realized),
        ),
    );
    let (region, members) = crate::physical::single_workgroup_tree_region(
        &request,
        request.sole_output(),
        crate::physical::RegionWrite::ProgramOutput,
    )
    .expect("the tree is offered at a count whose only sub-cap width is two");
    assert_eq!(region.schedule.threads_per_workgroup, 2);
    crate::physical::verify_schedule(
        region,
        members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the two-participant tree region verifies");
}
