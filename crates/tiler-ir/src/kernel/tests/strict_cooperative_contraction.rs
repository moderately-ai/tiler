use super::super::{
    BinaryOp, BlockRef, OperationRef, OperationView, SerialLoopRef, VerifiedKernel,
    lower_scheduled_region,
};
use super::support::cooperative_contraction_region;
use crate::schedule::{
    NumericalPermission, ReductionTopology, RegionProgram, ScheduledRegionBuilder, TailPolicy,
    VerifiedScheduledRegion,
};

/// The same region under a contract that forbids reassociation.
///
/// Built by re-verifying the fixture's own region rather than by a second
/// literal, so the only differences are the two the strict contract forces: the
/// realization's permission and the topology field that must agree with it.
fn strict_cooperative_contraction_region(
    output_m: u64,
    output_n: u64,
    contracted: u64,
    tail: TailPolicy,
) -> VerifiedScheduledRegion {
    let mut region = cooperative_contraction_region(output_m, output_n, contracted, tail)
        .region()
        .clone();
    let RegionProgram::Numerical { numerical, .. } = &mut region.index.program else {
        panic!("the contraction fixture builds a numerical program")
    };
    numerical.reassociation = NumericalPermission::Forbidden;
    let ReductionTopology::CooperativeContraction {
        permits_reassociation,
        ..
    } = &mut region.schedule.reduction
    else {
        panic!("the contraction fixture builds a cooperative contraction")
    };
    *permits_reassociation = false;
    ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("a strict contract admits the tiled contraction")
}

/// Returns the operations one block performs directly, ignoring nested bodies.
fn direct_operation_views(block: BlockRef<'_>) -> Vec<OperationView<'_>> {
    block.operations().map(OperationRef::view).collect()
}

/// Returns the round loop of a multi-round tiled contraction body.
///
/// The body carries exactly two top-level loops and the second is the round
/// loop: round zero is peeled ahead of it because the fold seeds at its first
/// product, so the peel contributes the first — its own tile fold. Selecting by
/// position rather than by bounds keeps the helper honest about the shape it
/// reads; a body that grew a third top-level loop fails here instead of having
/// one of them silently picked.
fn round_loop_of(kernel: &VerifiedKernel) -> SerialLoopRef<'_> {
    let loops: Vec<SerialLoopRef<'_>> = direct_operation_views(kernel.body())
        .into_iter()
        .filter_map(|view| match view {
            OperationView::SerialLoop(serial) => Some(serial),
            _ => None,
        })
        .collect();
    let [peeled_tile_fold, round_loop] = loops.as_slice() else {
        panic!(
            "a multi-round tiled body is a peeled tile fold then a round loop, found {} \
             top-level loops",
            loops.len()
        )
    };
    assert_eq!(
        (peeled_tile_fold.start(), peeled_tile_fold.end()),
        (1, 16),
        "round zero seeds from its first product, so its fold starts at one"
    );
    *round_loop
}

/// A strict contract admits the tiled contraction rather than refusing it.
///
/// The permission is recorded and cross-checked, never consulted to admit: the
/// schedule tiles the *memory* and leaves the contributor sequence alone, so
/// requiring reassociation here would refuse under every strict contract the one
/// realization the first-contraction record attributes uniquely to
/// `strict_fold+ftz`. The reassociating spelling stays admissible too, which is
/// what makes this a widening of the admitted contracts and not a swap.
#[test]
fn a_strict_contract_admits_the_tiled_contraction() {
    for tail in [TailPolicy::Exact, TailPolicy::Predicated] {
        let strict = strict_cooperative_contraction_region(32, 32, 32, tail);
        let ReductionTopology::CooperativeContraction {
            permits_reassociation,
            ..
        } = &strict.region().schedule.reduction
        else {
            panic!("the strict fixture keeps its topology")
        };
        assert!(!permits_reassociation, "{tail:?}");
        lower_scheduled_region(&strict).expect("the strict tiled contraction lowers");
        let permitted = cooperative_contraction_region(32, 32, 32, tail);
        assert_ne!(
            strict.canonical_identity().as_bytes(),
            permitted.canonical_identity().as_bytes(),
            "the contract's permission is identity-bearing: {tail:?}"
        );
    }
}

/// A repeating operand tile carries two anti-dependencies and still lowers.
///
/// Both staged allocations are rewritten each round, so the tile derives one
/// anti-dependency per allocation and one round boundary discharges both. The
/// lowering used to match a single edge and refused every such tile as an
/// unlowerable shape — which refused exactly the body this schedule is.
#[test]
fn a_multi_round_operand_tile_discharges_both_anti_dependencies() {
    let scheduled = cooperative_contraction_region(32, 32, 64, TailPolicy::Exact);
    let tile = crate::schedule::cooperative_tile(&scheduled.region().schedule.reduction)
        .expect("the region carries an operand tile");
    assert_eq!(tile.rounds, 4, "64 contracted points over a 16-wide tile");
    assert_eq!(
        tile.anti_dependency_edges().len(),
        2,
        "one rewrite obligation per staged allocation"
    );
    for edge in tile.anti_dependency_edges() {
        assert_eq!(
            tile.anti_discharging_points(edge).len(),
            1,
            "one round boundary discharges each"
        );
    }
    lower_scheduled_region(&scheduled).expect("the multi-round operand tile lowers");
}

/// The round loop continues one accumulator instead of adding a subtotal.
///
/// The distinguishing structure, and the reason it is asserted rather than
/// described: `acc + (p0 + … + p15)` and `((acc + p0) + …) + p15` combine the
/// same contributors in the same order and differ only in grouping, so they are
/// different binary32 values and only the second is the declared contributor
/// sequence. Both spellings emit the same *number* of additions, which is why
/// counting them proves nothing; what separates them is where the additions sit.
/// A round loop that combined a subtotal would perform one addition directly in
/// its own body, and its tile loop would start at one having seeded from a fresh
/// product. The carried form performs none, and its tile loop starts at zero.
#[test]
fn the_tiled_contraction_carries_one_accumulator_across_its_rounds() {
    let kernel = lower_scheduled_region(&cooperative_contraction_region(
        32,
        32,
        64,
        TailPolicy::Exact,
    ))
    .expect("the multi-round tiled contraction lowers");
    let round_loop = round_loop_of(&kernel);
    assert_eq!((round_loop.start(), round_loop.end()), (1, 4));
    let body = direct_operation_views(round_loop.body());
    let combines = body
        .iter()
        .filter(|view| {
            matches!(
                view,
                OperationView::Binary {
                    op: BinaryOp::F32Add,
                    ..
                }
            )
        })
        .count();
    assert_eq!(
        combines, 0,
        "the round loop combines {combines} subtotal(s) of its own; a carried \
         accumulator performs every addition inside the tile fold"
    );
    let mut tile_loops = body.iter().filter_map(|view| match view {
        OperationView::SerialLoop(serial) => Some(*serial),
        _ => None,
    });
    let tile_loop = tile_loops.next().expect("the round body folds one tile");
    assert!(tile_loops.next().is_none(), "one tile fold per round");
    assert_eq!(
        (tile_loop.start(), tile_loop.end()),
        (0, 16),
        "every contributor of the round enters the carried accumulator, \
         including the first — a fold starting at one would have seeded from a \
         fresh product and made this round a subtotal"
    );
    assert_eq!(
        tile_loop.accumulators().len(),
        1,
        "the carried accumulator is the loop's only state"
    );
}
