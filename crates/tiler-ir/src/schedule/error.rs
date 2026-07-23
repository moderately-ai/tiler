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
}

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
