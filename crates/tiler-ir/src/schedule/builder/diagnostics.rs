//! Refusal constructors and the fail-closed program view the gates share.
//!
//! Every `const fn` here wraps one rule vocabulary in the diagnostic that
//! carries it, so a gate states its refusal by naming the rule rather than by
//! restating a variant path each time. [`numerical_program`] belongs with them
//! because its only outcome besides the arithmetic view is a refusal: a
//! verifier internal reached outside the dispatch must be told the region
//! declares no arithmetic rather than answer for some.

use crate::schedule::error::{
    BlockedWorkgroupRule, ContributorCoverageRule, CooperativeTileRule, PartitionedCopyRule,
    ScheduleComponent, ScheduledRegionDiagnostic, VectorLaneRule,
};
use crate::schedule::model::{RegionProgram, ScalarProgram, ScheduledRegion};
use crate::schedule::numerics::NumericalRealization;
use crate::schedule::synchronization::SynchronizationRule;

pub(super) const fn incomplete(component: ScheduleComponent) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::IncompleteRegion { component }
}

/// The arithmetic-arm view every numerical-family verifier reads through.
///
/// Fail-closed rather than assumed: every caller runs under the dispatch that
/// already destructured the `Numerical` arm, so the refusal is unreachable
/// today — but a verifier internal reached some other way must refuse the copy
/// arm rather than answer for arithmetic the region does not declare.
pub(super) fn numerical_program(
    region: &ScheduledRegion,
) -> Result<(&ScalarProgram, &NumericalRealization), ScheduledRegionDiagnostic> {
    match &region.index.program {
        RegionProgram::Numerical { scalar, numerical } => Ok((scalar, numerical)),
        RegionProgram::PartitionedCopy(_) => {
            Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)
        }
    }
}

/// Shorthand for the one carried-rule constructor the copy gate uses.
pub(super) const fn partitioned_copy(rule: PartitionedCopyRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::PartitionedCopy { rule }
}

pub(super) const fn coverage_rule(rule: ContributorCoverageRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::ContributorCoverage { rule }
}

pub(super) const fn synchronization(rule: SynchronizationRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::Synchronization { rule }
}

pub(super) const fn cooperative(rule: CooperativeTileRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::CooperativeTile { rule }
}

pub(super) const fn blocked(rule: BlockedWorkgroupRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::BlockedWorkgroup { rule }
}

pub(super) const fn vector_lane(rule: VectorLaneRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::VectorLaneBinding { rule }
}
