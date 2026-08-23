use super::super::{
    BoundsProofKind, ContributorCoverage, ContributorCoverageRule, LogicalAccess,
    NumericalRealization, ReductionPaddingIdentity, ReductionTopology, ScheduledRegionBuilder,
    ScheduledRegionDiagnostic,
};
use super::support::{
    NEG_ZERO, PADDED_SPLIT, SPLIT, cooperative_builder, cooperative_builder_with,
    cooperative_rejection, cooperative_tile_fixture, extrema_partial_builder, final_pass_builder,
    partial_pass_builder, reassociating_numerical, set_numerical,
};
use crate::schedule::model::ContributorPartition;
use crate::schedule::numerics::NumericalPermission;
use crate::shape::Shape;

/// A split whose product does not cover the contributor sequence rejects.
///
/// The cases reach two different rules, and both are the right one. A split
/// that changes the *partition count* also changes the partial tensor the
/// region iterates, so its bounds proof stops refining its access first; a
/// split that keeps the count and misstates the per-partition share reaches
/// the coverage check itself. Driving both is what shows neither an
/// over-covering nor an under-covering split can slip through on the other
/// one's silence.
#[test]
fn an_inexact_split_is_rejected() {
    for (partition, expected) in [
        // Six contributors, five covered, and a partial tensor of five.
        (
            ContributorPartition {
                partitions: 5,
                contributors_per_partition: 1,
            },
            ScheduledRegionDiagnostic::BoundsProof,
        ),
        // A split of nothing covers nothing, and stages nothing.
        (
            ContributorPartition {
                partitions: 0,
                contributors_per_partition: 2,
            },
            ScheduledRegionDiagnostic::BoundsProof,
        ),
        // Three partitions, as the region stages, but nine contributors
        // claimed where the access supplies six.
        (
            ContributorPartition {
                partitions: 3,
                contributors_per_partition: 3,
            },
            ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::ExactCoverage,
            },
        ),
        // The same partition count, three covered where the access supplies
        // six.
        (
            ContributorPartition {
                partitions: 3,
                contributors_per_partition: 1,
            },
            ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::ExactCoverage,
            },
        ),
    ] {
        let mut builder = partial_pass_builder(SPLIT);
        let ReductionTopology::MultiPass { coverage, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *coverage = ContributorCoverage::Exact(partition);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [expected],
            "{partition:?} does not cover six contributors exactly once each"
        );
    }
}

const POS_ZERO: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0x0000_0000);

const NEG_INF: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0xff80_0000);

fn set_coverage(builder: &mut ScheduledRegionBuilder, coverage: ContributorCoverage) {
    let ReductionTopology::MultiPass {
        coverage: declared, ..
    } = &mut builder.schedule.as_mut().unwrap().reduction
    else {
        panic!("expected a split topology")
    };
    *declared = coverage;
}

fn padded_partial(identity: ReductionPaddingIdentity) -> ScheduledRegionBuilder {
    let mut builder = partial_pass_builder(PADDED_SPLIT);
    set_coverage(
        &mut builder,
        ContributorCoverage::IdentityPadded {
            partition: PADDED_SPLIT,
            identity,
        },
    );
    builder
}

/// Exact coverage of a previously encodable split keeps the pre-coverage
/// layout: the identity of two Exact regions is byte-identical, and a
/// padded sibling is a strict extension rather than a reinterpretation.
#[test]
fn exact_multi_pass_encodings_remain_byte_identical_and_padding_appends() {
    let exact = partial_pass_builder(SPLIT)
        .build()
        .unwrap()
        .canonical_identity()
        .as_bytes()
        .to_vec();
    let again = partial_pass_builder(SPLIT)
        .build()
        .unwrap()
        .canonical_identity()
        .as_bytes()
        .to_vec();
    assert_eq!(exact, again, "exact coverage is a closed encoding");

    let padded = padded_partial(NEG_ZERO)
        .build()
        .expect("a suffix-padded add split with -0.0 verifies")
        .canonical_identity()
        .as_bytes()
        .to_vec();
    assert_ne!(exact, padded);
    assert!(
        padded.len() > exact.len(),
        "the padded arm appends a local tag and identity; exact writes neither"
    );
}

/// Coverage tag: claiming a pad on an exactly covering split is padded
/// coverage, not exact coverage under another name.
#[test]
fn a_zero_length_pad_is_refused_as_padded_coverage() {
    let mut builder = partial_pass_builder(SPLIT);
    set_coverage(
        &mut builder,
        ContributorCoverage::IdentityPadded {
            partition: SPLIT,
            identity: NEG_ZERO,
        },
    );
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::PaddedCoverage,
        }]
    );
}

/// Partition capacity: a pad whose split is shorter than the real sequence
/// is refused by name.
#[test]
fn a_pad_below_the_real_count_is_refused() {
    let mut builder = partial_pass_builder(SPLIT);
    set_coverage(
        &mut builder,
        ContributorCoverage::IdentityPadded {
            partition: ContributorPartition {
                partitions: 3,
                contributors_per_partition: 1,
            },
            identity: NEG_ZERO,
        },
    );
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::CapacityBelowRealCount,
        }]
    );
}

/// Arithmetic type: a well-formed `bf16` identity on an `f32` fold is a
/// named mismatch, not an unrepresentable one.
#[test]
fn a_padding_identity_of_the_wrong_arithmetic_type_is_refused() {
    assert_eq!(
        padded_partial(ReductionPaddingIdentity::Bf16(0x8000))
            .build()
            .unwrap_err()
            .diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::ArithmeticTypeMismatch,
        }]
    );
}

/// Identity bits: `+0.0` is the empty-domain result, not the additive pad,
/// when signed zero is observable.
#[test]
fn plus_zero_is_not_a_two_sided_additive_identity_under_strict_signed_zero() {
    assert_eq!(
        padded_partial(POS_ZERO).build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::TwoSidedNeutrality,
        }]
    );
}

/// Signed-zero permission: the same `+0.0` bits are admitted once
/// elimination is permitted, because the two zeros are then observably equal.
#[test]
fn plus_zero_is_neutral_when_signed_zero_elimination_is_permitted() {
    let mut builder = padded_partial(POS_ZERO);
    set_numerical(
        &mut builder,
        NumericalRealization {
            signed_zero: NumericalPermission::Permitted,
            ..reassociating_numerical()
        },
    );
    builder
        .build()
        .expect("+0.0 is observably neutral under signed-zero elimination");
}

/// Family: `-0.0` is the additive pad and `-inf` is the maximum pad; each
/// is refused on the other family.
#[test]
fn padding_identity_is_family_specific() {
    let mut maximum = extrema_partial_builder(PADDED_SPLIT);
    set_coverage(
        &mut maximum,
        ContributorCoverage::IdentityPadded {
            partition: PADDED_SPLIT,
            identity: NEG_ZERO,
        },
    );
    assert_eq!(
        maximum.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::TwoSidedNeutrality,
        }]
    );

    assert_eq!(
        padded_partial(NEG_INF).build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::TwoSidedNeutrality,
        }]
    );

    let mut admitted = extrema_partial_builder(PADDED_SPLIT);
    set_coverage(
        &mut admitted,
        ContributorCoverage::IdentityPadded {
            partition: PADDED_SPLIT,
            identity: NEG_INF,
        },
    );
    admitted
        .build()
        .expect("-inf is the two-sided identity of the NaN-propagating maximum");
}

/// An all-padding sequence has no real prefix and is not a canonical suffix.
#[test]
fn an_all_padding_split_is_refused_as_noncanonical_placement() {
    let mut builder = partial_pass_builder(SPLIT);
    let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
    else {
        panic!("the fixture reads a reduction contributor");
    };
    *input_shape = Shape::from_dims([2, 0]);
    let BoundsProofKind::ReductionDomain {
        input_shape: proof_shape,
        ..
    } = &mut builder.bounds_proofs[0].kind
    else {
        panic!("the fixture proves a reduction domain");
    };
    *proof_shape = Shape::from_dims([2, 0]);
    set_coverage(
        &mut builder,
        ContributorCoverage::IdentityPadded {
            partition: SPLIT,
            identity: NEG_ZERO,
        },
    );
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::NoncanonicalPlacement,
        }]
    );
}

/// Partition capacity overflow is named rather than folded into a coverage miss.
#[test]
fn an_overflowing_padded_capacity_is_refused() {
    let mut builder = partial_pass_builder(SPLIT);
    set_coverage(
        &mut builder,
        ContributorCoverage::IdentityPadded {
            partition: ContributorPartition {
                partitions: 3,
                contributors_per_partition: u64::MAX,
            },
            identity: NEG_ZERO,
        },
    );
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::Overflow,
        }]
    );
}

/// A padded final pass invents partials the tensor does not hold.
#[test]
fn a_padded_final_pass_is_refused() {
    let mut builder = final_pass_builder(SPLIT);
    let ReductionTopology::MultiPass { coverage, .. } =
        &mut builder.schedule.as_mut().unwrap().reduction
    else {
        panic!("expected a split topology")
    };
    *coverage = ContributorCoverage::IdentityPadded {
        partition: SPLIT,
        identity: NEG_ZERO,
    };
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::PaddedCoverage,
        }]
    );
}

/// A split that ignores the round count folds every contributor twice.
///
/// The single-round split covers the sequence once; declared on a two-round
/// tile it would have each participant fold the same range on both rounds,
/// which is a different computation and not the declared reduction. Named
/// as exact-coverage rather than as a tile-shape mismatch: the participants
/// and iteration domain still agree, and the product does not.
#[test]
fn a_split_that_ignores_the_round_count_is_refused() {
    let mut tile = cooperative_tile_fixture();
    tile.rounds = 2;
    assert_eq!(
        cooperative_rejection(cooperative_builder_with(tile, SPLIT)),
        ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::ExactCoverage,
        }
    );
}

/// A split that does not cover the contributor sequence is rejected.
///
/// The partition count is held at the participant count, so this isolates
/// the coverage half: three participants folding three contributors each
/// would combine nine of the six the access declares.
#[test]
fn a_split_that_does_not_cover_the_contributors_is_rejected() {
    let mut builder = cooperative_builder(cooperative_tile_fixture());
    let ReductionTopology::CooperativeWorkgroup { coverage, .. } =
        &mut builder.schedule.as_mut().unwrap().reduction
    else {
        panic!("expected a cooperative topology")
    };
    *coverage = ContributorCoverage::Exact(ContributorPartition {
        partitions: 3,
        contributors_per_partition: 3,
    });
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::ContributorCoverage {
            rule: ContributorCoverageRule::ExactCoverage,
        }
    );
}
