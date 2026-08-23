//! The extrema/maximum-fold subject of `super::reduction`'s own
//! production module, split into its own file purely to keep both
//! under the size bound the split enforces; both map to
//! `schedule::builder::reduction`.

use super::super::{
    BoundsProofKind, LogicalAccess, ReductionTopology, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, ScheduledRegionDiagnostic, TensorRole, partial_reduction_axis,
};
use super::support::{
    SPLIT, bare_sum, cooperative_builder_parts, cooperative_tile_fixture, cooperative_topology,
    extrema_cooperative_builder, extrema_partial_builder, final_pass_builder, float_rows,
    into_extrema_split, maximum_scalar, partial_pass_builder, reassociating_numerical,
    serial_reduction_builder, set_numerical, set_scalar, strict_numerical,
};
use crate::schedule::model::{ContributorOrder, ContributorPartition};
use crate::schedule::numerics::NumericalPermission;
use crate::shape::{Axis, Shape};

/// The extrema fold verifies as a serial pass reading the original input.
#[test]
fn the_extrema_fold_verifies_as_a_serial_pass() {
    let region = serial_reduction_builder(maximum_scalar())
        .build()
        .expect("an extrema serial pass verifies");
    assert!(matches!(
        region.region().index.program,
        RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialMaximum { .. },
            ..
        }
    ));
}

/// The extrema fold does not share identity with the bare serial sum.
///
/// The two regions differ in nothing but their scalar program — same access
/// relation, same contributor order, same numerical realization — so an
/// appended scalar-program tag that had collided with an existing one would
/// make these equal. It is the check behind "the schedule domain did not
/// step": the new tag separates, and every earlier tag keeps its meaning.
///
/// The sum reads an intermediate where the extrema fold reads the first
/// input, so the bare-sum control is built with that one field changed and
/// nothing else.
#[test]
fn the_extrema_fold_has_its_own_canonical_identity() {
    let maximum = serial_reduction_builder(maximum_scalar())
        .build()
        .expect("the extrema pass verifies");
    let mut bare = serial_reduction_builder(ScalarProgram::StrictSerialSum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    });
    bare.accesses[0].tensor = TensorRole::Intermediate;
    bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
    let bare = bare.build().expect("the bare pass verifies");
    assert_ne!(maximum.canonical_identity(), bare.canonical_identity());
}

/// An empty reduced domain is refused, because the family has no identity.
///
/// **This is the one obligation the extrema fold has and no sum does.** A sum
/// commits `+0.0`; `Maximum` has no value it could commit, so the region is
/// refused rather than given a default. The control is the *same shape* under
/// the bare sum, which verifies — so the refusal is about the family and not
/// about the zero extent.
#[test]
fn an_empty_reduced_domain_is_refused_for_the_identity_less_fold() {
    let empty_input = Shape::from_dims([2, 0]);
    let widen = |builder: &mut ScheduledRegionBuilder| {
        let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
        else {
            panic!("the fixture reads a reduction contributor");
        };
        *input_shape = empty_input.clone();
        let BoundsProofKind::ReductionDomain { input_shape, .. } =
            &mut builder.bounds_proofs[0].kind
        else {
            panic!("the fixture proves a reduction domain");
        };
        *input_shape = empty_input.clone();
    };

    let mut maximum = serial_reduction_builder(maximum_scalar());
    widen(&mut maximum);
    assert_eq!(
        maximum.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "an identity-less fold over an empty domain has no value to commit"
    );

    // The control: the identical region under the bare sum verifies, because
    // that family declares `+0.0` for the empty case.
    let mut bare = serial_reduction_builder(ScalarProgram::StrictSerialSum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    });
    bare.accesses[0].tensor = TensorRole::Intermediate;
    bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
    widen(&mut bare);
    assert!(bare.build().is_ok());
}

/// A topology that describes no fold is refused for the extrema family.
///
/// The parallel topologies are admitted (below); these two are not, and the
/// reasons are different in kind. [`ReductionTopology::None`] says the region
/// performs no reduction, which contradicts a scalar program that is one.
/// [`ReductionTopology::Contraction`] folds a *contracted index space* stated
/// by the topology, which a one-tensor reduction access does not have.
/// Neither is a conservative refusal waiting to be widened.
#[test]
fn a_topology_that_describes_no_fold_is_refused_for_the_extrema_family() {
    let mut none = serial_reduction_builder(maximum_scalar());
    none.schedule.as_mut().unwrap().reduction = ReductionTopology::None;
    assert_eq!(
        none.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    let mut contraction = serial_reduction_builder(maximum_scalar());
    contraction.schedule.as_mut().unwrap().reduction = ReductionTopology::Contraction {
        contracted_shape: Shape::from_dims([6]),
        order: ContributorOrder::OriginalAxisLexicographic,
        permits_reassociation: false,
        permits_permutation: false,
    };
    assert_eq!(
        contraction.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    // The control: the unmodified serial fixture verifies, so the refusals
    // above are about the topology rather than about the fixture.
    assert!(serial_reduction_builder(maximum_scalar()).build().is_ok());
}

/// A split that covers no contributor, for the empty-domain fixtures below.
///
/// `partitions` stays nonzero — [`ContributorPartition::covers`] refuses a
/// zero partition count outright — so the empty case is expressed by the
/// per-partition width alone, which is exactly the shape an identity-seeded
/// family is allowed to have and an identity-less one is not.
const EMPTY_SPLIT: ContributorPartition = ContributorPartition {
    partitions: 3,
    contributors_per_partition: 0,
};

/// The final pass of an extrema split: fold the staged maxima into the result.
fn extrema_final_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
    let axes = vec![partial_reduction_axis(&Shape::from_dims([2])).expect("rank one fits u32")];
    let mut builder = final_pass_builder(partition);
    into_extrema_split(&mut builder, axes, TensorRole::Intermediate);
    builder
}

/// Both passes of an extrema split verify under a contract that forbids
/// reassociation, and the same split of a *sum* still refuses.
///
/// **This is the asymmetry the softmax's two passes owe.** The pinned extrema
/// family is associative and commutative on every binary32 input, so a split
/// of it changes no observable value and spends no permission —
/// `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY`. The denominator's sum is the
/// other fact, `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`, and the control here is
/// what keeps the widening from reading as a relaxation of both: the same
/// split shape, the same strict realization, the same fixture — and the sum
/// is refused.
#[test]
fn an_extrema_split_verifies_under_a_strict_contract_and_a_sum_split_does_not() {
    let partial = extrema_partial_builder(SPLIT)
        .build()
        .expect("an extrema partial pass verifies under a strict contract");
    let combine = extrema_final_builder(SPLIT)
        .build()
        .expect("an extrema final pass verifies under a strict contract");
    assert_eq!(partial.region().schedule.work_items, 6);
    assert_eq!(combine.region().schedule.work_items, 2);
    // The split is admitted without spending anything, which is the claim.
    assert_eq!(
        float_rows(&partial.requirements()).reassociation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&partial.requirements()).permutation,
        NumericalPermission::Forbidden
    );

    // The perturbation that fires: the same split of the sum, under the same
    // strict realization, is still refused.
    let mut summed = partial_pass_builder(SPLIT);
    set_numerical(&mut summed, strict_numerical());
    let Some(ReductionTopology::MultiPass {
        permits_reassociation,
        ..
    }) = summed
        .schedule
        .as_mut()
        .map(|schedule| &mut schedule.reduction)
    else {
        panic!("the fixture schedules a multi-pass split")
    };
    *permits_reassociation = false;
    assert_eq!(
        summed.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "splitting an ordered sum still consumes the reassociation permission"
    );
}

/// A cooperative tile over the extrema fold verifies under a strict contract.
///
/// The same claim as the split's, at the topology whose partials never leave
/// the workgroup: the staged fold seeds at the first slot, which is
/// admissible for an identity-less family exactly because the tile's staging
/// coverage and the exact launch prove every slot was written by a
/// participant that folded at least one contributor.
#[test]
fn a_cooperative_extrema_tile_verifies_under_a_strict_contract() {
    let verified = extrema_cooperative_builder()
        .build()
        .expect("an extrema tile verifies under a strict contract");
    assert_eq!(verified.requirements().local_memory_bytes, 12);
    assert_eq!(
        float_rows(&verified.requirements()).reassociation,
        NumericalPermission::Forbidden
    );

    // The control: the fixture's own sum, under the same strict realization
    // and the same tile, is refused.
    let ReductionTopology::CooperativeWorkgroup {
        coverage,
        tile,
        axes,
        order,
        accumulation,
        arrival,
        ..
    } = cooperative_topology(cooperative_tile_fixture())
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    let summed = cooperative_builder_parts(
        SPLIT,
        6,
        ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation: false,
            permits_permutation: false,
            arrival,
        },
        strict_numerical(),
    );
    assert_eq!(
        summed.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "a tile over an ordered sum still consumes the reassociation permission"
    );
}

/// A split of the identity-less fold is refused over an empty domain.
///
/// The obligation that replaces the empty-domain constant the family has no
/// correct value for, checked where a split could otherwise hide it: a
/// partition covering no contributor has nothing to stage. The control is the
/// *same* split under the bare sum, which verifies because that family
/// commits `+0.0` — so the refusal is about the family and not about the
/// zero extent or the zero-width partition.
#[test]
fn an_empty_split_is_refused_for_the_identity_less_fold() {
    let empty_input = Shape::from_dims([2, 0]);
    let empty = |builder: &mut ScheduledRegionBuilder| {
        let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
        else {
            panic!("the fixture reads a reduction contributor");
        };
        *input_shape = empty_input.clone();
        let BoundsProofKind::ReductionDomain { input_shape, .. } =
            &mut builder.bounds_proofs[0].kind
        else {
            panic!("the fixture proves a reduction domain");
        };
        *input_shape = empty_input.clone();
    };

    let mut maximum = extrema_partial_builder(EMPTY_SPLIT);
    empty(&mut maximum);
    assert_eq!(
        maximum.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "no partition of an identity-less fold may cover nothing"
    );

    let mut bare = partial_pass_builder(EMPTY_SPLIT);
    empty(&mut bare);
    assert!(bare.build().is_ok());
}

/// An extrema partial pass respelled as a sum verifies as *that sum*.
///
/// **This assertion inverted when a bare fold gained its declared input, and
/// the inversion narrows what the intrinsic verifier claims rather than losing
/// a check.** It was previously refused because every sum admitted as a partial
/// pass had to read an intermediate; a bare sum now folds whichever boundary
/// tensor holds its declared contributor domain, so this region is a coherent
/// partial pass of a prologue-less sum — the same accesses, the same split, a
/// different fold. That it was *authored* as an extrema pass is not a fact the
/// region carries: which occurrences a region claims is the compiler's subject
/// binding, and an intrinsic rule guessing intent from the read would have to
/// refuse the legal program this widening exists to admit.
///
/// What still separates the two spellings is identity, asserted here beside
/// the admission so they can never be interchanged downstream.
#[test]
fn an_extrema_partial_pass_respelled_as_a_sum_verifies_as_that_sum() {
    let mut summed = extrema_partial_builder(SPLIT);
    set_numerical(&mut summed, reassociating_numerical());
    let Some(ReductionTopology::MultiPass {
        permits_reassociation,
        ..
    }) = summed
        .schedule
        .as_mut()
        .map(|schedule| &mut schedule.reduction)
    else {
        panic!("the fixture schedules a multi-pass split")
    };
    // Reassociation permitted, because a split of a sum consumes it where a
    // split of the extrema fold does not: without it the region would be
    // refused for the permission and say nothing about the boundary role.
    *permits_reassociation = true;
    set_scalar(&mut summed, bare_sum(vec![Axis::new(1)]));
    let summed = summed
        .build()
        .expect("a bare sum folding the first input is a coherent partial pass");

    // The control: the extrema program over the identical region verifies,
    // and the two are not one region under two names.
    let extrema = extrema_partial_builder(SPLIT)
        .build()
        .expect("the extrema partial pass verifies");
    assert_ne!(summed.canonical_identity(), extrema.canonical_identity());
}

/// A split extrema region shares identity with neither neighbour.
///
/// The concrete form of the step verdict. Admitting the parallel topologies
/// introduced no tag and moved no field: an extrema split encodes under the
/// scalar-program tag `0x28` and the topology tag `0x33`, both already in
/// their existing positions, and the pair was simply unreachable before. So
/// no previously encodable region's bytes moved — which
/// `the_strict_f32_region_has_its_recorded_canonical_identity` pins — while
/// the newly reachable regions still separate from every neighbour they could
/// be confused with: the same fold serially, the same split summed, and the
/// other pass of their own split.
#[test]
fn a_split_extrema_region_has_its_own_canonical_identity() {
    let partial = extrema_partial_builder(SPLIT).build().unwrap();
    let combine = extrema_final_builder(SPLIT).build().unwrap();
    let serial = serial_reduction_builder(maximum_scalar()).build().unwrap();
    let summed = partial_pass_builder(SPLIT).build().unwrap();
    let tile = extrema_cooperative_builder().build().unwrap();
    let identities = [
        partial.canonical_identity(),
        combine.canonical_identity(),
        serial.canonical_identity(),
        summed.canonical_identity(),
        tile.canonical_identity(),
    ];
    for (position, identity) in identities.iter().enumerate() {
        assert!(
            !identities[..position].contains(identity),
            "identity {position} collided with an earlier region"
        );
    }
}

/// The pinned NaN-propagating extrema family, restated for this test.
///
/// `maximum_f32` in `crates/tiler-reference/src/softmax.rs` is the authority
/// and evaluates it for the registered operation; this crate cannot call it,
/// because `tiler-reference` depends on `tiler-ir` and not the other way
/// round. So the schedule-level evidence restates the two rules that make the
/// family what it is — NaN is absorbing, and `-0.0 < +0.0` is a total order —
/// and the control in
/// [`a_split_of_the_extrema_fold_agrees_with_the_serial_fold_bit_for_bit`]
/// fails if this is `maxNum` (Rust's `f32::max`) instead, which is the other
/// ADR 0023 family and the one a careless restatement would land on.
fn maximum_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() || right.is_nan() {
        return f32::NAN;
    }
    #[allow(
        clippy::float_cmp,
        reason = "the extrema family is defined by exact IEEE-754 comparison"
    )]
    let equal = left == right;
    if equal {
        // Equal under IEEE comparison means two identical values or the pair
        // `(-0.0, +0.0)` in some order. The bitwise `and` selects `+0.0` for
        // the second without branching on which side it arrived from, and is
        // the identity for the first.
        return f32::from_bits(left.to_bits() & right.to_bits());
    }
    if left > right { left } else { right }
}

/// The operands at which associativity could fail, and nothing else.
const EXTREMA_CORPUS: [f32; 7] = [
    0.0,
    -0.0,
    1.0,
    -1.0,
    f32::INFINITY,
    f32::NEG_INFINITY,
    f32::NAN,
];

/// Folds one contiguous run left to right, as the emitted serial loop does.
fn fold(values: &[f32], combine: fn(f32, f32) -> f32) -> f32 {
    values
        .iter()
        .copied()
        .reduce(combine)
        .expect("every fold below is over a non-empty run")
}

/// Folds a sequence through the partition boundaries a split declares.
fn fold_split(values: &[f32], width: usize, combine: fn(f32, f32) -> f32) -> f32 {
    let partials: Vec<f32> = values
        .chunks(width)
        .map(|partition| fold(partition, combine))
        .collect();
    fold(&partials, combine)
}

/// The split and the serial fold agree bit for bit on every corpus sequence.
///
/// The legality claim executed at the schedule level, over the split a
/// *verified* region declares rather than one this test invents: the
/// partition width comes back out of the built region's topology, so a change
/// to what the verifier admits changes what this folds. Every assignment of
/// the corpus to the six contributor positions is enumerated, which is
/// exhaustive over the operands the property could fail at.
///
/// Two controls, because the agreement is worth nothing without them. The
/// *same* split boundaries applied to an ordered sum change its bits, so the
/// split shape is one a reassociation difference can travel through; and the
/// family restated here is not `f32::max`, so the agreement is this family's
/// rather than any maximum's.
#[test]
fn a_split_of_the_extrema_fold_agrees_with_the_serial_fold_bit_for_bit() {
    let verified = extrema_partial_builder(SPLIT).build().unwrap();
    let ReductionTopology::MultiPass { coverage, .. } = verified.region().schedule.reduction else {
        panic!("the extrema partial fixture schedules a multi-pass split")
    };
    let partition = coverage.partition();
    let width = usize::try_from(partition.contributors_per_partition)
        .expect("the fixture's partition width fits usize");
    let contributors = usize::try_from(
        partition
            .total_contributors()
            .expect("the fixture's split does not overflow"),
    )
    .expect("the fixture's contributor count fits usize");

    let mut sequence = vec![0.0_f32; contributors];
    let corpus = EXTREMA_CORPUS.len();
    for encoded in 0..corpus.pow(u32::try_from(contributors).expect("six fits u32")) {
        let mut remaining = encoded;
        for slot in &mut sequence {
            *slot = EXTREMA_CORPUS[remaining % corpus];
            remaining /= corpus;
        }
        assert_eq!(
            fold_split(&sequence, width, maximum_f32).to_bits(),
            fold(&sequence, maximum_f32).to_bits(),
            "the split disagrees with the serial fold at {sequence:?}"
        );
    }

    // The first control. `1.0 + 2^-24` rounds back to `1.0` under
    // ties-to-even, so the serial fold absorbs every addend and returns
    // `1.0`; the split adds the small terms to each other first, where they
    // are exact, and the partials then reach the result. The corpus above
    // cannot show this — every one of its values is exact under addition —
    // so the control needs its own sequence rather than a search.
    let half_ulp = f32::EPSILON / 2.0;
    let absorbing = vec![1.0_f32, half_ulp, half_ulp, half_ulp, half_ulp, half_ulp];
    assert_eq!(absorbing.len(), contributors);
    let add = |left: f32, right: f32| left + right;
    assert_eq!(fold(&absorbing, add).to_bits(), 1.0_f32.to_bits());
    assert_ne!(
        fold_split(&absorbing, width, add).to_bits(),
        fold(&absorbing, add).to_bits(),
        "these split boundaries cannot expose a reassociation difference at all"
    );

    // The second control: the family folded above is the NaN-propagating one
    // and not `maxNum`, which returns the number beside a NaN.
    assert!(
        EXTREMA_CORPUS
            .iter()
            .any(|left| EXTREMA_CORPUS
                .iter()
                .any(|right| maximum_f32(*left, *right).to_bits() != left.max(*right).to_bits())),
        "the family folded here is indistinguishable from `maxNum` on this corpus"
    );
}
