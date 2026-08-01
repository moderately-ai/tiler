//! Typed errors for scheduled-region construction and verification.
//!
//! Two error boundaries mirror the [`crate::index`] discipline: insertion-time
//! [`ScheduleBuildError`] rejects locally malformed builder input, while the
//! consuming [`super::ScheduledRegionBuilder::build`] returns a recoverable
//! [`ScheduledRegionBuildError`] carrying the whole-region
//! [`ScheduledRegionDiagnostic`] set and the intact builder.

use std::error::Error;
use std::fmt;

use super::ScheduledRegionBuilder;

/// A governed structural resource in the scheduled-region profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ScheduleLimitKind {
    /// Logical tensor-access count.
    Accesses,
    /// Bounds-proof witness count.
    BoundsProofs,
    /// Reduction axis count in one schedule.
    ReductionAxes,
}

impl fmt::Display for ScheduleLimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A component whose single-assignment slot was set more than once.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ScheduleComponent {
    /// The iteration shape.
    IterationShape,
    /// The write-ownership proof.
    OwnershipProof,
    /// The scalar program.
    ScalarProgram,
    /// The numerical realization.
    NumericalRealization,
    /// The kernel schedule.
    KernelSchedule,
}

impl fmt::Display for ScheduleComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Failure during one transactional scheduled-region builder insertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScheduleBuildError {
    /// A single-assignment component was set more than once.
    ComponentAlreadySet {
        /// Component whose slot was already populated.
        component: ScheduleComponent,
    },
    /// A governed construction resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: ScheduleLimitKind,
        /// Attempted quantity.
        actual: usize,
        /// Maximum admitted quantity.
        limit: usize,
    },
}

impl fmt::Display for ScheduleBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for ScheduleBuildError {}

/// One deterministic whole-region schedule-verification failure.
///
/// Each variant names an intrinsic legality rule proven by
/// [`super::ScheduledRegionBuilder::build`]. [`ScheduledRegionDiagnostic::rule`]
/// returns the stable rule identifier a consumer can surface in an explanation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScheduledRegionDiagnostic {
    /// A required component was never supplied to the builder.
    IncompleteRegion {
        /// The missing component.
        component: ScheduleComponent,
    },
    /// The launch geometry does not exactly cover the iteration domain.
    LaunchCoverage,
    /// The region does not carry exactly one read and one write access.
    AccessCount,
    /// An access violated the read/write mode, map, or ownership contract.
    AccessContract,
    /// The region does not carry exactly one bounds proof per access.
    BoundsProofCount,
    /// A bounds or ownership proof referenced the wrong access or witness.
    ProofReference,
    /// A bounds proof did not refine the access it is attached to.
    BoundsProof,
    /// The scalar program, reduction topology, and access map disagree.
    NumericalOrAccessRefinement,
    /// The iteration-domain element count overflowed `u64`.
    ShapeProductOverflow,
    /// A cooperative workgroup tile violated one cross-invocation dataflow rule.
    ///
    /// The rule is carried rather than collapsed into this variant, because a
    /// tile can fail in seven structurally different ways and a producer needs
    /// to know which handoff it got wrong (ADR 0048's explainability
    /// obligation). A single opaque diagnostic here would name the tile without
    /// naming the defect.
    CooperativeTile {
        /// The violated cross-invocation dataflow rule.
        rule: CooperativeTileRule,
    },
}

/// One violated rule of a cooperative workgroup tile's dataflow.
///
/// Each variant is a property the tile *states*, so each is separately
/// perturbable: the model can express a nonuniformly reached phase, an
/// overlapping write, or a read of a dead allocation, and refusing what cannot
/// be stated would be a rule nothing could ever trip.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CooperativeTileRule {
    /// The participant count does not equal the launched workgroup width.
    ///
    /// Uniform convergence: a tile whose participants are a proper subset of
    /// its workgroup leaves the remaining invocations outside every phase, and
    /// a synchronization point they never reach is divergent.
    ParticipantConvergence,
    /// The local coordinate space is not the dense run `0..participants`.
    LocalCoordinates,
    /// The phase ordinals are not the dense ascending run `0..phases`.
    PhaseSequence,
    /// A phase is reachable by only some of the tile's participants.
    PhaseParticipation,
    /// A staging allocation's declared lifetime is malformed, or an access
    /// falls outside it.
    StagingLifetime,
    /// A staged access addresses a slot the allocation does not have.
    StagingCapacity,
    /// Two participants write one staging slot.
    StagingConflict,
    /// A staging slot inside the allocation has no writer.
    StagingCoverage,
    /// A phase reads a staging allocation no earlier phase writes.
    StagedProducer,
    /// The tile declares no cross-invocation handoff.
    ///
    /// A tile with no [`crate::schedule::VisibilityEdge`] performs no
    /// cooperation: every staged value is read in the phase that wrote it, so
    /// the region is an ordinary per-invocation reduction wearing a tile's
    /// vocabulary, and admitting it would let a schedule claim cooperation it
    /// does not perform.
    NoVisibilityEdge,
    /// The tile does not name exactly one committing participant.
    CommitOwnership,
    /// The tile's storage, coordinates, or phase count exceeds a governed bound.
    ///
    /// Disjointness and coverage are decided by enumerating addressed slots, so
    /// the bound is what keeps that decision finite.
    StructuralLimit,
    /// The split, the participants, and the contributor sequence disagree.
    ContributorSplit,
    /// The contributor domain is empty, so nothing is staged.
    ///
    /// An empty reduction commits its declared identity from one invocation
    /// with no fold and no handoff; a tile there would declare a visibility edge
    /// over values no participant produces.
    EmptyContributorDomain,
}

impl CooperativeTileRule {
    /// Returns the stable rule identifier for this cooperative-tile failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ParticipantConvergence => "cooperative-participant-convergence",
            Self::LocalCoordinates => "cooperative-local-coordinates",
            Self::PhaseSequence => "cooperative-phase-sequence",
            Self::PhaseParticipation => "cooperative-phase-participation",
            Self::StagingLifetime => "cooperative-staging-lifetime",
            Self::StagingCapacity => "cooperative-staging-capacity",
            Self::StagingConflict => "cooperative-staging-conflict",
            Self::StagingCoverage => "cooperative-staging-coverage",
            Self::StagedProducer => "cooperative-staged-producer",
            Self::NoVisibilityEdge => "cooperative-no-visibility-edge",
            Self::CommitOwnership => "cooperative-commit-ownership",
            Self::StructuralLimit => "cooperative-structural-limit",
            Self::ContributorSplit => "cooperative-contributor-split",
            Self::EmptyContributorDomain => "cooperative-empty-contributor-domain",
        }
    }
}

impl fmt::Display for CooperativeTileRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for CooperativeTileRule {}

impl ScheduledRegionDiagnostic {
    /// Returns the stable intrinsic-rule identifier for this diagnostic.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::IncompleteRegion { .. } => "incomplete-region",
            Self::LaunchCoverage => "launch-coverage",
            Self::AccessCount => "access-count",
            Self::AccessContract => "access-contract",
            Self::BoundsProofCount => "bounds-proof-count",
            Self::ProofReference => "proof-reference",
            Self::BoundsProof => "bounds-proof",
            Self::NumericalOrAccessRefinement => "numerical-or-access-refinement",
            Self::ShapeProductOverflow => "shape-product-overflow",
            Self::CooperativeTile { rule } => rule.rule(),
        }
    }
}

impl fmt::Display for ScheduledRegionDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for ScheduledRegionDiagnostic {}

/// Recoverable failure from consuming whole-region schedule verification.
///
/// Carries the deterministic diagnostics and returns the intact builder through
/// [`ScheduledRegionBuildError::into_parts`] so a caller can amend and retry.
#[derive(Debug)]
pub struct ScheduledRegionBuildError {
    pub(super) builder: Box<ScheduledRegionBuilder>,
    pub(super) diagnostics: Vec<ScheduledRegionDiagnostic>,
}

impl ScheduledRegionBuildError {
    /// Returns all deterministic diagnostics in stable order.
    #[must_use]
    pub fn diagnostics(&self) -> &[ScheduledRegionDiagnostic] {
        &self.diagnostics
    }

    /// Recovers the intact builder and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (ScheduledRegionBuilder, Vec<ScheduledRegionDiagnostic>) {
        (*self.builder, self.diagnostics)
    }
}

impl fmt::Display for ScheduledRegionBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "scheduled-region verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}
impl Error for ScheduledRegionBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.diagnostics.first().map(|diagnostic| diagnostic as _)
    }
}

/// Failure to count reduction contributors for a logical access.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContributorError {
    /// The access is not a reduction-contributor access.
    NotReductionAccess,
    /// The reduction axes are not a canonical ascending in-range set.
    NonCanonicalAxes,
    /// A reduction axis did not resolve to an input extent.
    AxisOutOfRange,
    /// The contributor product overflowed `u64`.
    Overflow,
}

impl ContributorError {
    /// Returns the stable intrinsic-rule identifier for this error.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::NotReductionAccess => "contributor-access",
            Self::NonCanonicalAxes => "contributor-axes",
            Self::AxisOutOfRange => "contributor-axis",
            Self::Overflow => "contributor-product",
        }
    }
}

impl fmt::Display for ContributorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for ContributorError {}

/// The iteration-domain element product exceeded `u64`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementCountOverflow;

impl fmt::Display for ElementCountOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "iteration-domain element count exceeds u64")
    }
}
impl Error for ElementCountOverflow {}
