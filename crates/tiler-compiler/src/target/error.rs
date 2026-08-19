//! The typed refusal vocabulary of target-profile construction.
//!
//! One variant per rule a draft can break, each naming the axis, row, licence,
//! or phase that refused, so a producer can repair the declaration from the
//! diagnostic alone. The `From<FeasibilityError>` bridge is where the checked
//! boundary's own refusals enter this vocabulary.

use tiler_ir::program::abi::AvailabilityPhase;

use crate::target::ScalarArithmetic;
use crate::target::feasibility::{FeasibilityError, MAX_TARGET_PROFILE_DESCRIPTOR_BYTES};

/// Typed target-profile construction diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetProfileBuildError {
    /// A numerical dimension was paired with another dimension's behaviour space.
    InvalidDimensionBehaviour,
    /// A declared relaxation did not match the subject and dimension.
    InvalidRelaxation,
    /// No semantic authority registered the arithmetic/resolved-type association.
    UnvalidatedScalarArithmetic,
    /// A caller attempted to assert compiler-proved exact emulation.
    UnverifiedExactEmulation,
    /// Structured producer attribution was incomplete or incoherent.
    InvalidProducerClaim,
    /// The same quantitative capability axis was declared twice at one phase.
    DuplicateQuantitativeCapability {
        /// Stable governed axis key.
        axis: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// A quantitative query was declared for an availability phase that cannot
    /// answer that axis's exact requirement.
    InvalidQuantitativeQueryPhase {
        /// Stable governed axis key.
        axis: &'static str,
        /// Earliest phase that can answer this axis correctly.
        required: AvailabilityPhase,
        /// Phase the rejected query declared.
        actual: AvailabilityPhase,
    },
    /// The same quantitative capability axis received two query schemas.
    DuplicateQuantitativeQuery {
        /// Stable governed axis key.
        axis: &'static str,
    },
    /// One axis cannot carry both an available fact and a deferred query.
    ConflictingQuantitativeFactAndQuery {
        /// Stable governed axis key.
        axis: &'static str,
    },
    /// The same synchronization subject was declared twice at one phase.
    ///
    /// The verdict is deliberately not part of that key: a profile declaring one
    /// subject both realized and unrealizable has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateSynchronizationRealization,
    /// A declared synchronization subject fences no memory domain.
    ///
    /// A fence over nothing publishes nothing, so no handoff could consume it and
    /// a realization of one would be a permission for an operation with no effect.
    VacuousSynchronizationSubject,
    /// The same numerical behaviour was declared twice at the same phase.
    DuplicateScalarDeclaration,
    /// A complete measured subnormal table would overlap an existing row.
    ConflictingSubnormalDeclaration {
        /// Exact scalar subject whose table was already partially declared.
        subject: Box<ScalarArithmetic>,
        /// Stable numerical-dimension key of the conflicting row.
        dimension: &'static str,
        /// Availability phase of the conflicting row.
        phase: AvailabilityPhase,
    },
    /// The same exact resolved type received more than one dispatch verdict at
    /// one availability phase.
    DuplicateDispatchability,
    /// The same scalar subject and backend licence received two
    /// evaluation-order verdicts at one availability phase.
    ///
    /// The verdict is deliberately not part of that key, for the reason
    /// [`Self::DuplicateSynchronizationRealization`] excludes it: a profile
    /// declaring one subject both preserved and not preserved has stated a
    /// contradiction, and admitting both rows would leave whichever the sort put
    /// first deciding.
    DuplicateEvaluationOrderPreservation {
        /// Stable governed key of the licence both rows claimed.
        licence: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same measured cost row was declared twice at one availability phase.
    ///
    /// The value is deliberately not part of that key, for the reason
    /// [`Self::DuplicateQuantitativeCapability`] excludes its bound: a profile
    /// stating one machine quantity twice has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateCostRow {
        /// Stable governed key of the row both declarations claimed.
        row: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same workgroup-tree-width policy phase was declared twice.
    ///
    /// The variant is deliberately not part of that key: a profile stating two
    /// policies at one phase has stated a contradiction, and admitting both
    /// would leave whichever the sort put first deciding. One variant exists
    /// today, so the refusal is a restatement; a second variant would still
    /// refuse at the same phase rather than encode a choice.
    DuplicateWorkgroupTreeWidthPolicy {
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same complete elementary-realization row was declared twice.
    ///
    /// Distinct contracts for one operation remain legal. Only an exact
    /// restatement of the verified contract, both evidence records, and the
    /// source is rejected. No row is replaced, merged, or preferred.
    DuplicateElementaryRealization,
    /// The same subgroup subject was declared twice at one phase.
    ///
    /// The verdict is deliberately not part of that key: a profile declaring one
    /// subject both realized and unrealizable has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateSubgroupRealization,
    /// A prepared subgroup-width query was declared at a phase that cannot
    /// answer the exact prepared pipeline's execution width.
    InvalidSubgroupQueryPhase {
        /// The one phase that can answer the property.
        required: AvailabilityPhase,
        /// Phase the rejected query declared.
        actual: AvailabilityPhase,
    },
    /// A second prepared subgroup-width query was declared.
    ///
    /// The contract is profile-level and singular: one prepared pipeline has
    /// one execution width, and two query contracts would let one later
    /// observation stand for two claims with no way to attribute the answer.
    DuplicateSubgroupWidthQuery,
    /// A subgroup subject was declared `Realized` with no prepared
    /// subgroup-width query to confirm its width.
    ///
    /// ADR 0094 decision 7 requires the exact prepared pipeline's width to
    /// confirm the realization before routing commits; a profile that licenses
    /// the schedule without saying how that width is obtained would leave the
    /// gate undischargeable.
    MissingSubgroupWidthQuery,
    /// A prepared subgroup-width query was declared with no `Realized`
    /// subgroup subject to confirm.
    ///
    /// Silence and `Unrealizable` license no schedule whose width could be
    /// confirmed, so the query claims a realization this profile never made.
    OrphanSubgroupWidthQuery,
    /// The canonical descriptor exceeded the artifact identity bound.
    DescriptorTooLong {
        /// Encoded byte length.
        actual: usize,
        /// Maximum admitted encoded byte length.
        max: usize,
    },
    /// The quantitative feasibility profile was malformed.
    MalformedProfile {
        /// Stable refusing rule.
        rule: &'static str,
    },
}

impl std::fmt::Display for TargetProfileBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetProfileBuildError {}

impl From<FeasibilityError> for TargetProfileBuildError {
    fn from(value: FeasibilityError) -> Self {
        match value {
            FeasibilityError::MalformedProfile { rule } => Self::MalformedProfile { rule },
            FeasibilityError::DescriptorTooLong { actual, .. } => Self::DescriptorTooLong {
                actual,
                max: MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
            },
            FeasibilityError::MalformedProposal { .. } => Self::MalformedProfile {
                rule: "unexpected-proposal-validation",
            },
            // Deferred predicates are minted at assessment, not at profile
            // construction, so this arm mirrors the proposal one: reachable
            // only if the feasibility authority's error vocabulary is misused.
            FeasibilityError::MalformedDeferred { .. } => Self::MalformedProfile {
                rule: "unexpected-deferred-validation",
            },
        }
    }
}
