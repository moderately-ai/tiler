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
use super::numerics::ArithmeticType;

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
    /// A parallel topology declared a combining width its region does not
    /// compute in.
    ///
    /// Separate from [`Self::NumericalOrAccessRefinement`], which the two
    /// parallel gates otherwise share, and separate for the reason
    /// [`Self::CooperativeTile`] gives for carrying its rule: a strategy that
    /// accumulates at a width the contract does not admit is *a different
    /// computation*, and a producer told only that "the program, the topology,
    /// and the access map disagree" cannot tell that from a wrong axis set or a
    /// wrong contributor order. The two widths are carried because the refusal
    /// is otherwise unactionable — an accumulator is wrong only relative to
    /// something, and that something is the region's own element width rather
    /// than a literal this variant could name.
    ///
    /// A *narrower* declaration is the case
    /// `implement-parallel-reduction-strategies` criterion 3 names. A wider one
    /// is refused by this same rule, because widening the accumulator is
    /// equally a computation the region did not declare — the check is
    /// disagreement with the region's width, not a comparison of widths.
    AccumulationWidth {
        /// The width the topology declared it combines at.
        declared: ArithmeticType,
        /// The width the region's own scalar program computes in.
        required: ArithmeticType,
    },
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
    /// A schedule's synchronization authority violated one rule.
    ///
    /// Separate from [`Self::CooperativeTile`] because the two answer different
    /// questions about one region: the tile rules say whether the dataflow is
    /// well formed, and these say whether anything legally orders it. A producer
    /// whose staging is correct but whose point sits at the wrong boundary needs
    /// to be told the second, not the first.
    Synchronization {
        /// The violated synchronization rule.
        rule: super::synchronization::SynchronizationRule,
    },
    /// A reduction topology's contributor coverage is malformed.
    ///
    /// Separate from [`Self::NumericalOrAccessRefinement`] and from
    /// [`Self::CooperativeTile`] with [`CooperativeTileRule::ContributorSplit`]:
    /// those name program/access disagreement and a tile whose split, participants,
    /// and iteration shape disagree. This names the coverage statement itself —
    /// exact versus identity-padded — so an exact split that misses the real
    /// count and a padded split whose identity is not two-sided-neutral cannot
    /// share one diagnostic.
    ContributorCoverage {
        /// The violated coverage rule.
        rule: ContributorCoverageRule,
    },
    /// A cooperative contraction split violated a topology-specific rule.
    ContractionSplit {
        /// The violated split rule.
        rule: ContractionSplitRule,
    },
    /// The blocked-workgroup execution binding is not a bijection from launched
    /// invocations onto the declared output domain, or it is paired with the
    /// wrong topology.
    ///
    /// Separate from [`Self::CooperativeTile`] and from
    /// [`Self::LaunchCoverage`]: the tile rules police staging dataflow, and
    /// launch coverage is the existing 1-D exact-cover equality. This names the
    /// blocked map itself — overlap, gap, a missing required binding, or a
    /// binding on a topology that does not use one.
    BlockedWorkgroup {
        /// The violated blocked-map rule.
        rule: BlockedWorkgroupRule,
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
    /// The participant space is empty, has a zero extent, or its extent product
    /// overflows.
    ///
    /// A rank above
    /// [`MAX_COOPERATIVE_PARTICIPANT_RANK`](crate::schedule::MAX_COOPERATIVE_PARTICIPANT_RANK)
    /// is deliberately *not* among these: the space's constructor makes it
    /// unrepresentable, so nothing that reaches this rule can carry one.
    LocalCoordinates,
    /// A staged span's stride vector does not have one entry per participant
    /// dimension.
    ///
    /// Separate from [`Self::StagingCapacity`], which says a span leaves the
    /// storage the tile declared, and from [`Self::LocalCoordinates`], which
    /// says the participant space is malformed: this one says a well-formed span
    /// and a well-formed space disagree about how many dimensions there are, and
    /// neither is wrong on its own terms.
    SpanRank,
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
    /// The tile's round count is zero or beyond the governed bound.
    ///
    /// Separate from [`Self::StructuralLimit`] because a round count is not an
    /// enumeration bound: nothing walks the rounds, so an oversized one does not
    /// make verification unbounded. What it does is overflow the contributor
    /// arithmetic a consumer performs against it and name a loop trip count no
    /// launch could finish — and a count of *zero* is not a bound failure at all
    /// but a tile whose phases never run, which would derive staged accesses and
    /// a synchronization requirement for a program that executes nothing.
    RoundStructure,
    /// The split, the participants, and the contributor sequence disagree.
    ContributorSplit,
    /// The contributor domain is empty, so nothing is staged.
    ///
    /// An empty reduction commits its declared identity from one invocation
    /// with no fold and no handoff; a tile there would declare a visibility edge
    /// over values no participant produces.
    EmptyContributorDomain,
    /// The declared arrival order needs a permission the contract withholds.
    ///
    /// Separate from the reassociation the split itself consumes, and checked
    /// separately, because the two permissions are independent (ADR 0011): an
    /// arrival the program does not fix permutes the contributor sequence in
    /// addition to regrouping it, and a strategy admitted on reassociation while
    /// using both would consume a freedom nobody granted.
    ArrivalPermission,
    /// The declared arrival order names a construct this profile does not
    /// realize.
    ///
    /// Distinct from [`Self::ArrivalPermission`], and reached only after it: a
    /// contract may well permit permutation and there still be no
    /// arrival-ordered construct to realize the order, because
    /// [`crate::schedule::SynchronizationKind`] admits only a control barrier.
    /// Collapsing the two would make a permitted-but-unrealizable arrival report
    /// a numerical refusal the caller could not act on.
    UnadmittedArrival,
    /// An operand-sharing tile does not name every participant as a committer.
    ///
    /// The inverse of [`Self::CommitOwnership`]. The one-committer theorem stays
    /// on [`crate::schedule::ReductionTopology::CooperativeWorkgroup`]; this
    /// sibling requires the full participant run because every invocation owns
    /// its own output position.
    OperandTileCommit,
}

impl CooperativeTileRule {
    /// Returns the stable rule identifier for this cooperative-tile failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ParticipantConvergence => "cooperative-participant-convergence",
            Self::LocalCoordinates => "cooperative-local-coordinates",
            Self::SpanRank => "cooperative-span-rank",
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
            Self::RoundStructure => "cooperative-round-structure",
            Self::ContributorSplit => "cooperative-contributor-split",
            Self::EmptyContributorDomain => "cooperative-empty-contributor-domain",
            Self::ArrivalPermission => "cooperative-arrival-permission",
            Self::UnadmittedArrival => "cooperative-unadmitted-arrival",
            Self::OperandTileCommit => "cooperative-operand-tile-commit",
        }
    }
}

impl fmt::Display for CooperativeTileRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for CooperativeTileRule {}

/// One violated rule of a blocked-workgroup execution binding.
///
/// Each variant is a property the binding *states*, so overlap and gap are
/// separately perturbable: the model can express a map that claims two
/// invocations for one output, or a map that leaves an output unowned, and
/// refusing what cannot be stated would be a rule nothing could ever trip.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum BlockedWorkgroupRule {
    /// The operand-sharing tile requires this binding and did not receive it.
    BindingRequired,
    /// A topology that does not use a blocked map carried one.
    BindingForbidden,
    /// The block, workgroup grid, and output ranks disagree.
    RankMismatch,
    /// Two launched invocations map to the same output coordinate.
    MappingOverlap,
    /// An output coordinate has no launched preimage.
    MappingGap,
    /// The launch width or work-item count disagrees with the block product.
    LaunchGeometry,
    /// The tile's participant space is not the binding's output block.
    ParticipantBlockMismatch,
}

impl BlockedWorkgroupRule {
    /// Returns the stable rule identifier for this blocked-map failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::BindingRequired => "blocked-workgroup-binding-required",
            Self::BindingForbidden => "blocked-workgroup-binding-forbidden",
            Self::RankMismatch => "blocked-workgroup-rank-mismatch",
            Self::MappingOverlap => "blocked-workgroup-mapping-overlap",
            Self::MappingGap => "blocked-workgroup-mapping-gap",
            Self::LaunchGeometry => "blocked-workgroup-launch-geometry",
            Self::ParticipantBlockMismatch => "blocked-workgroup-participant-block-mismatch",
        }
    }
}

impl fmt::Display for BlockedWorkgroupRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for BlockedWorkgroupRule {}

/// One violated rule of a reduction topology's contributor coverage.
///
/// Exact-coverage and padded-coverage failures are named separately so a
/// producer cannot mistake one for the other. Overflow, a capacity below the
/// real count, a noncanonical suffix, an arithmetic-type mismatch, and a
/// failed two-sided-neutrality proof are each independently perturbable.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ContributorCoverageRule {
    /// Exact coverage does not cover the real contributor sequence once each.
    ExactCoverage,
    /// Identity-padded coverage is not a proper padded extension of the real
    /// sequence — typically a zero-length pad, which is exact coverage under
    /// another name, or a padded final pass whose partials are already staged.
    PaddedCoverage,
    /// The partition capacity overflowed `u64`.
    Overflow,
    /// The split's capacity is below the real contributor count.
    CapacityBelowRealCount,
    /// The derived padding is not a canonical suffix of the covered sequence.
    ///
    /// Suffix-only is the only representable placement. This fires when the
    /// derived pad would occupy a whole leading unit — a fully padded
    /// intermediate round, or an all-padding sequence with no real prefix.
    NoncanonicalPlacement,
    /// The identity's arithmetic type is not the region's own arithmetic.
    ArithmeticTypeMismatch,
    /// The identity is not two-sided-neutral under the region's combiner,
    /// arithmetic, rounding, signed-zero contract, NaN behaviour, and
    /// family-specific canonicalization.
    TwoSidedNeutrality,
}

impl ContributorCoverageRule {
    /// Returns the stable rule identifier for this coverage failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ExactCoverage => "contributor-exact-coverage",
            Self::PaddedCoverage => "contributor-padded-coverage",
            Self::Overflow => "contributor-coverage-overflow",
            Self::CapacityBelowRealCount => "contributor-capacity-below-real-count",
            Self::NoncanonicalPlacement => "contributor-noncanonical-padding-placement",
            Self::ArithmeticTypeMismatch => "contributor-padding-arithmetic-type",
            Self::TwoSidedNeutrality => "contributor-padding-two-sided-neutrality",
        }
    }
}

impl fmt::Display for ContributorCoverageRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for ContributorCoverageRule {}

/// One violated rule of a cooperative contraction split.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ContractionSplitRule {
    /// The split consumes reassociation and the contract withholds it.
    ReassociationPermission,
    /// Lane-strided membership consumes permutation and the contract withholds it.
    PermutationPermission,
    /// The exact partition does not cover the positive contracted sequence.
    ExactCoverage,
    /// The tile participant set does not equal the partition set.
    ParticipantPartition,
    /// The staged partial arrival is not fixed ascending-participant order.
    UnadmittedArrival,
}

impl ContractionSplitRule {
    /// Returns the stable rule identifier for this refusal.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ReassociationPermission => "contraction-split-reassociation-permission",
            Self::PermutationPermission => "contraction-split-permutation-permission",
            Self::ExactCoverage => "contraction-split-exact-coverage",
            Self::ParticipantPartition => "contraction-split-participant-partition",
            Self::UnadmittedArrival => "contraction-split-unadmitted-arrival",
        }
    }
}

impl fmt::Display for ContractionSplitRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for ContractionSplitRule {}

/// Typed preflight refusal for an exact-divisible cooperative contraction.
///
/// A caller selecting the tiled approach receives one of these when an
/// exact-divisibility equality is absent or false. The function that returns
/// this never substitutes the direct contraction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CooperativeContractionAdmission {
    /// An output extent is not divisible by its block extent.
    OutputBlockNotDivisible {
        /// Output axis, in the output shape's own order.
        axis: usize,
        /// Declared output extent on that axis.
        output: u64,
        /// Requested block extent on that axis.
        block: u64,
    },
    /// A contracted extent is not divisible by its tile width.
    ContractedTileNotDivisible {
        /// Contracted axis, in the contracted shape's own order.
        axis: usize,
        /// Declared contracted extent on that axis.
        contracted: u64,
        /// Requested tile extent on that axis.
        tile: u64,
    },
    /// The output shape and the output block have different ranks.
    OutputBlockRankMismatch {
        /// Rank of the declared output.
        output_rank: usize,
        /// Rank of the requested block.
        block_rank: usize,
    },
    /// The contracted shape and the contracted tile have different ranks.
    ContractedTileRankMismatch {
        /// Rank of the declared contracted space.
        contracted_rank: usize,
        /// Rank of the requested contracted tile.
        tile_rank: usize,
    },
    /// A block extent is zero, so no invocation owns that axis.
    EmptyOutputBlock {
        /// Output axis whose block extent is zero.
        axis: usize,
    },
    /// A contracted tile extent is zero, so no tile covers that axis.
    EmptyContractedTile {
        /// Contracted axis whose tile extent is zero.
        axis: usize,
    },
    /// A workgroup-grid or tile-count product overflowed `u64`.
    ShapeProductOverflow,
}

impl CooperativeContractionAdmission {
    /// Returns the stable preflight-rule identifier for this refusal.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::OutputBlockNotDivisible { .. } => {
                "cooperative-contraction-output-block-not-divisible"
            }
            Self::ContractedTileNotDivisible { .. } => {
                "cooperative-contraction-contracted-tile-not-divisible"
            }
            Self::OutputBlockRankMismatch { .. } => {
                "cooperative-contraction-output-block-rank-mismatch"
            }
            Self::ContractedTileRankMismatch { .. } => {
                "cooperative-contraction-contracted-tile-rank-mismatch"
            }
            Self::EmptyOutputBlock { .. } => "cooperative-contraction-empty-output-block",
            Self::EmptyContractedTile { .. } => "cooperative-contraction-empty-contracted-tile",
            Self::ShapeProductOverflow => "cooperative-contraction-shape-product-overflow",
        }
    }
}

impl fmt::Display for CooperativeContractionAdmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for CooperativeContractionAdmission {}

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
            Self::AccumulationWidth { .. } => "accumulation-width",
            Self::ShapeProductOverflow => "shape-product-overflow",
            Self::CooperativeTile { rule } => rule.rule(),
            Self::Synchronization { rule } => rule.rule(),
            Self::ContributorCoverage { rule } => rule.rule(),
            Self::ContractionSplit { rule } => rule.rule(),
            Self::BlockedWorkgroup { rule } => rule.rule(),
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
