//! Contributor coverage arithmetic and a padded split's identity obligation.
//!
//! Every rule here is decided by the declared partition, the round count, and
//! the family's own combiner — never by the topology that declared them, which
//! is why the admissions call into this file instead of restating the
//! arithmetic each way a split can be spelled. The two-sided neutrality of a
//! padding identity is derived rather than asserted, so an identity that is
//! not neutral under the region's combiner cannot be admitted by naming it.

use crate::schedule::error::{ContributorCoverageRule, ScheduledRegionDiagnostic};
use crate::schedule::model::{
    ContributorCoverage, ContributorPartition, ReductionPaddingIdentity, ScalarProgram,
    scalar_arithmetic_type,
};
use crate::schedule::numerics::NumericalRealization;

use super::diagnostics::coverage_rule;

/// Verifies one topology's contributor coverage against the real sequence.
///
/// `rounds` is `1` for a multi-pass split — the partitions are the whole story —
/// and the tile's declared round count for a cooperative one. Exact coverage
/// reuses [`ContributorPartition::covers`] when there is no extra round factor,
/// so that method keeps the meaning every existing consumer already applies.
/// Identity-padded coverage derives the pad count by checked subtraction and
/// requires a canonical suffix: the last unit of the covered sequence still
/// holds a real contributor, and a zero-length pad is exact coverage under
/// another name.
pub(super) fn verify_contributor_coverage(
    coverage: ContributorCoverage,
    contributors: u64,
    rounds: u64,
    program: &ScalarProgram,
    numerical: &NumericalRealization,
) -> Result<(), ScheduledRegionDiagnostic> {
    match coverage {
        ContributorCoverage::Exact(partition) => {
            verify_exact_coverage(partition, contributors, rounds)
        }
        ContributorCoverage::IdentityPadded {
            partition,
            identity,
        } => {
            verify_padded_coverage(partition, contributors, rounds)?;
            verify_padding_identity(identity, program, numerical)
        }
    }
}

fn verify_exact_coverage(
    partition: ContributorPartition,
    contributors: u64,
    rounds: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if rounds == 0 {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    }
    if rounds == 1 {
        if partition.total_contributors().is_none() {
            return Err(coverage_rule(ContributorCoverageRule::Overflow));
        }
        if !partition.covers(contributors) {
            return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
        }
        return Ok(());
    }
    if partition.partitions == 0 {
        return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
    }
    let Some(total) = partition.total_contributors() else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    let Some(covered) = total.checked_mul(rounds) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if covered != contributors {
        return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
    }
    Ok(())
}

fn verify_padded_coverage(
    partition: ContributorPartition,
    contributors: u64,
    rounds: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if rounds == 0 || partition.partitions == 0 {
        return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
    }
    let Some(per_round) = partition.total_contributors() else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    let Some(capacity) = per_round.checked_mul(rounds) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if capacity < contributors {
        return Err(coverage_rule(
            ContributorCoverageRule::CapacityBelowRealCount,
        ));
    }
    if capacity == contributors {
        return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
    }
    // Canonical suffix: only the last unit may be ragged, and a unit with no
    // real contributor is refused. For `rounds == 1` that is `C > 0`.
    let Some(prefix) = per_round.checked_mul(rounds - 1) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if contributors <= prefix {
        return Err(coverage_rule(
            ContributorCoverageRule::NoncanonicalPlacement,
        ));
    }
    Ok(())
}

fn verify_padding_identity(
    identity: ReductionPaddingIdentity,
    program: &ScalarProgram,
    numerical: &NumericalRealization,
) -> Result<(), ScheduledRegionDiagnostic> {
    let required = scalar_arithmetic_type(program);
    if identity.arithmetic_type() != required {
        return Err(coverage_rule(
            ContributorCoverageRule::ArithmeticTypeMismatch,
        ));
    }
    let Some(combiner) = reduction_combiner(program) else {
        return Err(coverage_rule(ContributorCoverageRule::TwoSidedNeutrality));
    };
    if !identity_is_two_sided_neutral(identity, combiner, numerical) {
        return Err(coverage_rule(ContributorCoverageRule::TwoSidedNeutrality));
    }
    Ok(())
}

/// The binary combiner a padded split injects into.
///
/// Derived from the scalar program rather than declared beside the identity:
/// the identity is a statement about this combiner, and a second field would be
/// a place for a producer to name the wrong one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionCombiner {
    Add,
    Maximum,
}

fn reduction_combiner(program: &ScalarProgram) -> Option<ReductionCombiner> {
    match program {
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. } => Some(ReductionCombiner::Add),
        ScalarProgram::StrictSerialMaximum { .. } => Some(ReductionCombiner::Maximum),
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}

/// Derives two-sided neutrality of `identity` under the region's combiner.
///
/// For IEEE-754 binary32 addition the only possible identities are the two
/// zeros; the witness set is therefore a case analysis, not a sample. `-0.0`
/// is two-sided-neutral with signed zero observable.
/// `+0.0 + (-0.0)` is `+0.0`, so `+0.0` is admitted only when signed-zero
/// elimination is permitted. For the NaN-propagating maximum family with
/// `-0.0 < +0.0`, `-inf` is the unique two-sided identity once each combine
/// is followed by the family's canonicalization.
fn identity_is_two_sided_neutral(
    identity: ReductionPaddingIdentity,
    combiner: ReductionCombiner,
    numerical: &NumericalRealization,
) -> bool {
    match identity {
        ReductionPaddingIdentity::F32(bits) => {
            f32_identity_is_two_sided_neutral(bits, combiner, numerical)
        }
        ReductionPaddingIdentity::F16(_)
        | ReductionPaddingIdentity::Bf16(_)
        | ReductionPaddingIdentity::F64(_) => false,
    }
}

fn f32_identity_is_two_sided_neutral(
    identity: u32,
    combiner: ReductionCombiner,
    numerical: &NumericalRealization,
) -> bool {
    const WITNESSES: [u32; 9] = [
        0x0000_0000, // +0.0
        0x8000_0000, // -0.0
        0x3f80_0000, // 1.0
        0xbf80_0000, // -1.0
        0x0000_0001, // smallest subnormal
        0x0080_0000, // smallest positive normal
        0x7f80_0000, // +inf
        0xff80_0000, // -inf
        0x7fc0_0001, // a non-canonical quiet NaN
    ];
    let combine = match combiner {
        ReductionCombiner::Add => f32_add_bits,
        ReductionCombiner::Maximum => f32_maximum_bits,
    };
    let canonical = numerical.canonical_arithmetic_nan_bits;
    for operand in WITNESSES {
        let left = canonicalize_f32(combine(identity, operand), canonical);
        let right = canonicalize_f32(combine(operand, identity), canonical);
        let expected = canonicalize_f32(operand, canonical);
        if !f32_observably_equal(left, expected, numerical)
            || !f32_observably_equal(right, expected, numerical)
        {
            return false;
        }
    }
    true
}

fn f32_add_bits(lhs: u32, rhs: u32) -> u32 {
    (f32::from_bits(lhs) + f32::from_bits(rhs)).to_bits()
}

fn f32_maximum_bits(lhs: u32, rhs: u32) -> u32 {
    let left = f32::from_bits(lhs);
    let right = f32::from_bits(rhs);
    if left.is_nan() {
        return lhs;
    }
    if right.is_nan() {
        return rhs;
    }
    match left.partial_cmp(&right) {
        Some(std::cmp::Ordering::Less) => rhs,
        Some(std::cmp::Ordering::Equal) => {
            // IEEE 754-2018 maximum orders `-0.0 < +0.0`; `partial_cmp` does not.
            if left == 0.0 && (lhs ^ rhs) == 0x8000_0000 {
                lhs & 0x7fff_ffff
            } else {
                lhs
            }
        }
        Some(std::cmp::Ordering::Greater) | None => lhs,
    }
}

fn canonicalize_f32(bits: u32, canonical_nan: u32) -> u32 {
    if f32::from_bits(bits).is_nan() {
        canonical_nan
    } else {
        bits
    }
}

fn f32_observably_equal(lhs: u32, rhs: u32, numerical: &NumericalRealization) -> bool {
    if lhs == rhs {
        return true;
    }
    if numerical.permits_signed_zero_elimination() {
        let left = f32::from_bits(lhs);
        let right = f32::from_bits(rhs);
        if left == 0.0 && right == 0.0 {
            return true;
        }
    }
    false
}
