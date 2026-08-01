//! The schedule-owned synchronization authority of one cooperative tile.
//!
//! A [`CooperativeTile`](super::CooperativeTile) states a cross-invocation
//! dependency as a set of [`VisibilityEdge`]s; nothing in that module discharges
//! one. A [`SynchronizationPoint`] is the only construct that does. It names, in
//! target-neutral terms, exactly what a realization must deliver — the operation
//! kind, the invocations that must arrive, the invocations across which effects
//! become visible, the address spaces fenced, and the ordering established — and
//! separately where in the tile it sits, which participants reach it, and on what
//! evidence its convergence rests.
//!
//! # Why the point lives inside the tile
//!
//! A point is placed at a *phase boundary*, and phases exist only inside a
//! cooperative tile. Putting the list beside the topology on
//! [`KernelSchedule`](super::KernelSchedule) would make "a synchronization point
//! in a schedule that has no phases" a statement the verifier has to refuse,
//! rather than one the model cannot make — and would break every
//! `KernelSchedule` struct literal in the workspace for a field that is `empty`
//! at all but one of them. The tile is the physical realization of one
//! reduction, and the ordering of its own phases is part of that realization,
//! exactly as `MultiPass` carries the storage contract of its split.
//!
//! The consequence is deliberate and is the ticket's own closing elimination: a
//! barrier in the pointwise, global-linear program *cannot be stated here at
//! all*. The structured-kernel verifier still refuses one intrinsically, so the
//! elimination is checked at the layer where a barrier can be written.
//!
//! # Why the target-facing subject is one value
//!
//! [`SynchronizationSubject`] groups the five dimensions a target must realize
//! *together*. Each of them is separately true of some realization on some
//! machine — a fence over device memory, an acquire-release ordering, a
//! subgroup-wide arrival — so a target that declared them independently would
//! let their conjunction be inferred from facts none of which is about it. The
//! subject is therefore matched as one equality, and a feasibility authority
//! never reads a field of it in isolation.
//!
//! Placement, participants, and convergence are deliberately *not* in the
//! subject. They are properties of the program, proven here; asking a target to
//! attest to them would ask it about the caller's code. What a target attests to
//! is the realization, and that is what the subject names.

use super::cooperative::{ParticipantRange, VisibilityEdge};
use super::handles::{PhaseId, SyncPointId};

/// The class of synchronization construct one point requires.
///
/// The kind is stated rather than assumed, because "a synchronization point" is
/// not a synonym for "a control barrier": an asynchronous copy completes without
/// halting execution, a split-phase barrier separates arrival from waiting, a
/// collective both synchronizes and computes, an atomic orders one location
/// without an execution scope arriving at all, and an inter-dispatch dependency
/// is a queue-level ordering rather than an in-kernel one. Each has a different
/// contract over participants, visibility, and failure, and none of them is
/// admitted.
///
/// **Only [`Self::ControlBarrier`] is admitted, and the rest are refused with a
/// named rule rather than left unstatable.** The distinction matters twice.
/// A schedule that names one is rejected as
/// [`SynchronizationRule::UnadmittedKind`](super::SynchronizationRule::UnadmittedKind)
/// instead of silently lowering as a barrier. And the kind is part of
/// [`SynchronizationSubject`], so a target fact declaring a realization of some
/// *other* kind can never satisfy a control barrier's requirement — which is a
/// composition a single-variant vocabulary could not even express, let alone
/// refuse.
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder and the schedule
/// verifier map this totally, so a widened vocabulary must be a build error at
/// each rather than a silently admitted construct.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationKind {
    /// Every invocation of the execution scope arrives, and the fenced effects
    /// of all of them become visible to all of them.
    ControlBarrier,
    /// A copy issued by one invocation and completed asynchronously.
    ///
    /// Unadmitted: completion is observed through a separate wait construct this
    /// vocabulary does not have, so a point naming this kind would state an
    /// obligation nothing could discharge.
    AsynchronousCopy,
    /// An arrival separated from its wait by unrelated work.
    ///
    /// Unadmitted: two program points, not one, so a single placement cannot say
    /// where it is.
    SplitPhaseBarrier,
    /// A synchronizing operation that also computes over the participants.
    ///
    /// Unadmitted: it carries a combine order and a numerical realization of its
    /// own, which no field here states.
    Collective,
    /// An ordered read-modify-write of one location.
    ///
    /// Unadmitted: no execution scope arrives, so the participant set and the
    /// convergence proof are meaningless for it.
    Atomic,
    /// An ordering between two dispatches rather than inside one.
    ///
    /// Unadmitted: it is a property of a kernel *program*, not of a region's
    /// phases, so a phase boundary cannot place it.
    InterDispatchDependency,
}

impl SynchronizationKind {
    /// Returns the canonical tag naming this kind in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::ControlBarrier => 0x01,
            Self::AsynchronousCopy => 0x02,
            Self::SplitPhaseBarrier => 0x03,
            Self::Collective => 0x04,
            Self::Atomic => 0x05,
            Self::InterDispatchDependency => 0x06,
        }
    }

    /// Returns the stable identifier naming this kind in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::ControlBarrier => "control-barrier",
            Self::AsynchronousCopy => "asynchronous-copy",
            Self::SplitPhaseBarrier => "split-phase-barrier",
            Self::Collective => "collective",
            Self::Atomic => "atomic",
            Self::InterDispatchDependency => "inter-dispatch-dependency",
        }
    }
}

/// A governed set of invocations, used for arrival and for visibility.
///
/// One vocabulary rather than two, because "which invocations must arrive" and
/// "across which invocations effects become visible" range over the same
/// partition of a launch — and stating them as two values of one type is what
/// makes a realization that arrives workgroup-wide but publishes only
/// subgroup-wide *expressible*, and therefore refusable.
///
/// Deliberately not `#[non_exhaustive]`: totally mapped by the identity encoder
/// and by the structured-kernel verifier's barrier projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationScope {
    /// All invocations of one subgroup.
    Subgroup,
    /// All invocations of one workgroup.
    Workgroup,
    /// All invocations of the dispatch.
    Device,
}

impl SynchronizationScope {
    /// Returns the canonical tag naming this scope in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Subgroup => 0x01,
            Self::Workgroup => 0x02,
            Self::Device => 0x03,
        }
    }

    /// Returns the stable identifier naming this scope in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Subgroup => "subgroup",
            Self::Workgroup => "workgroup",
            Self::Device => "device",
        }
    }
}

/// The memory domains one synchronization point fences.
///
/// A pair of named flags rather than an ordered list of address spaces, because
/// a set has no canonical spelling as a sequence: two schedules that fenced the
/// same domains in different orders would be two identities for one realization.
/// Invocation-private and constant memory have no flag at all — neither is
/// shared between invocations, so fencing one is not a statement about
/// visibility.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FencedSpaces {
    /// Whether workgroup-shared memory effects are fenced.
    pub workgroup: bool,
    /// Whether device memory effects are fenced.
    pub device: bool,
}

impl FencedSpaces {
    /// The empty fence, which orders no memory domain.
    pub const NONE: Self = Self {
        workgroup: false,
        device: false,
    };

    /// Returns whether this fence names no memory domain at all.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.workgroup && !self.device
    }
}

/// The ordering one synchronization point establishes over the effects it fences.
///
/// Deliberately not `#[non_exhaustive]`: totally mapped by the identity encoder
/// and by the structured-kernel verifier's barrier projection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MemoryOrdering {
    /// No happens-before edge is established; only the arrival is ordered.
    Relaxed,
    /// Effects before the point are released and effects after it acquire them.
    AcquireRelease,
    /// A single total order over every participant's fenced effects.
    SequentiallyConsistent,
}

impl MemoryOrdering {
    /// Returns the canonical tag naming this ordering in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Relaxed => 0x01,
            Self::AcquireRelease => 0x02,
            Self::SequentiallyConsistent => 0x03,
        }
    }

    /// Returns the stable identifier naming this ordering in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Relaxed => "relaxed",
            Self::AcquireRelease => "acquire-release",
            Self::SequentiallyConsistent => "sequentially-consistent",
        }
    }
}

/// The complete realization one synchronization point requires of a target.
///
/// **This is the atomic unit a target fact ranges over.** Its five dimensions
/// are matched as one value and never composed: a machine that fences device
/// memory, a machine that arrives subgroup-wide, and a machine that establishes
/// acquire-release ordering are three true statements whose conjunction is not
/// a statement about any of them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SynchronizationSubject {
    /// Class of synchronization construct.
    pub kind: SynchronizationKind,
    /// Invocations that must arrive at the same dynamic instance.
    pub execution_scope: SynchronizationScope,
    /// Invocations across which the fenced effects become visible.
    pub visibility_scope: SynchronizationScope,
    /// Memory domains whose effects the point fences.
    pub fenced_spaces: FencedSpaces,
    /// Ordering established over the fenced effects.
    pub ordering: MemoryOrdering,
}

/// Where in a cooperative tile one synchronization point sits.
///
/// A boundary is named by the two phases it separates rather than by one phase
/// plus a side, so a boundary has exactly one spelling and two schedules that
/// order their handoffs identically cannot differ in identity. The verifier
/// requires the two to be consecutive existing phases: a "boundary" between
/// phase 0 and phase 2 is not a program point, because phase 1 runs between
/// them and its own effects would fall on an undetermined side of the fence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationPlacement {
    /// Between the last effect of `preceding` and the first of `following`.
    PhaseBoundary {
        /// Phase whose effects are released at this point.
        preceding: PhaseId,
        /// Phase whose effects acquire them.
        following: PhaseId,
    },
}

impl SynchronizationPlacement {
    /// Returns the canonical tag naming this placement in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::PhaseBoundary { .. } => 0x01,
        }
    }

    /// Returns the phase whose effects this placement releases.
    #[must_use]
    pub const fn preceding(self) -> PhaseId {
        match self {
            Self::PhaseBoundary { preceding, .. } => preceding,
        }
    }

    /// Returns the phase whose effects acquire at this placement.
    #[must_use]
    pub const fn following(self) -> PhaseId {
        match self {
            Self::PhaseBoundary { following, .. } => following,
        }
    }
}

/// The evidence class a point's convergence claim rests on.
///
/// Convergence — every participant reaching the same dynamic instance of the
/// point — is what makes a control barrier defined rather than undefined
/// behaviour, and it is a property of the *program*, so no target declares it.
/// The evidence class is carried so that the difference between a derived proof
/// and a caller's word is a value the verifier can refuse, rather than a
/// distinction that exists only in a comment.
///
/// Deliberately not `#[non_exhaustive]`: totally mapped by the identity encoder
/// and the verifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConvergenceEvidence {
    /// Derived: every phase the point bounds is reached by every participant.
    ///
    /// The verifier re-derives this from the tile's own phase participation
    /// rather than accepting the declaration, so the class states *which*
    /// derivation was intended and the check states whether it holds.
    EveryParticipantReachesThePoint,
    /// Asserted by the producer with no derivation.
    ///
    /// Always refused. It exists so that "the caller said so" is a statement the
    /// model can make and the verifier can reject by name, instead of a
    /// possibility the type system silently forecloses and no test can drive.
    CallerAsserted,
}

impl ConvergenceEvidence {
    /// Returns the canonical tag naming this evidence in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::EveryParticipantReachesThePoint => 0x01,
            Self::CallerAsserted => 0x02,
        }
    }
}

/// One synchronization point of a cooperative tile.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SynchronizationPoint {
    /// Tile-local ordinal naming this point.
    pub id: SyncPointId,
    /// The complete realization this point requires of a target.
    pub subject: SynchronizationSubject,
    /// Where in the tile the point sits.
    pub placement: SynchronizationPlacement,
    /// Local coordinates that must arrive at this point.
    pub participants: ParticipantRange,
    /// Evidence class backing the point's convergence.
    pub convergence: ConvergenceEvidence,
}

impl SynchronizationPoint {
    /// Returns whether this point orders `edge`'s producer before its consumer.
    ///
    /// A boundary between `preceding` and `following` separates every effect of
    /// phase `preceding` and earlier from every effect of `following` and later,
    /// so it discharges exactly the edges whose production is at or before
    /// `preceding` and whose consumption is at or after `following`.
    ///
    /// Derived rather than declared: an edge set on the point would be a second
    /// place to state what the placement already determines, and the two could
    /// disagree.
    #[must_use]
    pub const fn discharges(&self, edge: VisibilityEdge) -> bool {
        let preceding = self.placement.preceding();
        let following = self.placement.following();
        edge.produced_in.get() <= preceding.get() && edge.consumed_in.get() >= following.get()
    }
}

/// The realization a cooperative tile's staged handoff requires.
///
/// Derived from the edges the point discharges rather than declared, which is
/// why this takes the edges rather than reading a field: every edge crosses one
/// [`WorkgroupStaging`](super::WorkgroupStaging) allocation, so the fence names
/// workgroup memory and nothing else, and the arrival and publication scopes are
/// the workgroup whose invocations the tile's participants exhaust. A handoff
/// through device memory would derive a different fence here, which is why the
/// function is written over the edges instead of returning a constant.
///
/// Returns `None` when `edges` is empty: a point that orders no handoff requires
/// no realization, and an empty fence with an arrival scope would be a
/// requirement stated over nothing.
#[must_use]
pub fn required_subject(edges: &[VisibilityEdge]) -> Option<SynchronizationSubject> {
    if edges.is_empty() {
        return None;
    }
    Some(SynchronizationSubject {
        // The only admitted kind, and the only one whose contract — every
        // participant arrives, and all their fenced effects publish to all of
        // them — is what a staged handoff between two phases needs.
        kind: SynchronizationKind::ControlBarrier,
        // The tile's participants are exactly one workgroup's invocations, which
        // the tile's own convergence rule requires; no narrower scope contains
        // them and no wider one is needed.
        execution_scope: SynchronizationScope::Workgroup,
        // The readers are the same set as the writers, so publication has to
        // reach the workgroup and no further. A device-wide publication is a
        // different, stronger realization a target may or may not have.
        visibility_scope: SynchronizationScope::Workgroup,
        fenced_spaces: FencedSpaces {
            // Every edge crosses a workgroup staging allocation.
            workgroup: true,
            // Nothing device-visible crosses this point: the region's boundary
            // loads and its owning store are on one side of it each. Fencing
            // device memory anyway would be a different realization with a
            // different cost, and the schedule is normalized, so the exact
            // derived fence is required rather than a superset admitted.
            device: false,
        },
        // A producer-to-consumer handoff is exactly a release followed by an
        // acquire. `Relaxed` orders no effect and cannot discharge an edge;
        // `SequentiallyConsistent` is a stronger realization that the handoff
        // does not need and that the derivation therefore must not claim.
        ordering: MemoryOrdering::AcquireRelease,
    })
}

/// One violated rule of a schedule's synchronization authority.
///
/// Each variant names a property a point *states*, so each is separately
/// perturbable: the model can express an unadmitted kind, a boundary that is not
/// a program point, a participant set narrower than the tile's, or a convergence
/// claim with no derivation behind it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SynchronizationRule {
    /// The point ordinals are not the dense ascending run `0..points`.
    PointSequence,
    /// The tile declares more synchronization points than the governed bound.
    StructuralLimit,
    /// The point names an operation kind whose contract is not admitted.
    UnadmittedKind,
    /// The placement is not a boundary between two consecutive existing phases.
    Placement,
    /// The point's participants are not the tile's participants.
    ParticipantSet,
    /// The arrival scope is not the one the discharged handoff requires.
    ExecutionScope,
    /// The publication scope is not the one the discharged handoff requires.
    VisibilityScope,
    /// The fenced memory domains are not the ones the handoff requires.
    FencedSpaces,
    /// The ordering is not the one a producer-to-consumer handoff requires.
    Ordering,
    /// Convergence rests on a caller's assertion rather than a derivation.
    ConvergenceEvidence,
    /// A phase the point bounds is not reached by every participant.
    Convergence,
    /// A cross-invocation visibility edge is discharged by no point.
    UndischargedVisibility,
    /// A point discharges an edge another point already discharges, or none.
    ///
    /// Both halves are canonicality. A second point over one edge is a schedule
    /// that says the same thing twice and would have two identities for one
    /// realization; a point that discharges nothing is a synchronization the
    /// program does not need, and admitting it would let a schedule consume a
    /// target authority for an operation it has no reason to perform.
    RedundantPoint,
    /// The tile has fewer than two participants, so nothing crosses invocations.
    ///
    /// A single-participant tile stages values it reads back itself. The handoff
    /// is within one invocation, program order already orders it, and a point
    /// there would be the semantically redundant barrier this authority exists
    /// to eliminate.
    SingleParticipant,
}

impl SynchronizationRule {
    /// Returns the stable rule identifier for this synchronization failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::PointSequence => "synchronization-point-sequence",
            Self::StructuralLimit => "synchronization-structural-limit",
            Self::UnadmittedKind => "synchronization-unadmitted-kind",
            Self::Placement => "synchronization-placement",
            Self::ParticipantSet => "synchronization-participant-set",
            Self::ExecutionScope => "synchronization-execution-scope",
            Self::VisibilityScope => "synchronization-visibility-scope",
            Self::FencedSpaces => "synchronization-fenced-spaces",
            Self::Ordering => "synchronization-ordering",
            Self::ConvergenceEvidence => "synchronization-convergence-evidence",
            Self::Convergence => "synchronization-convergence",
            Self::UndischargedVisibility => "synchronization-undischarged-visibility",
            Self::RedundantPoint => "synchronization-redundant-point",
            Self::SingleParticipant => "synchronization-single-participant",
        }
    }
}

impl std::fmt::Display for SynchronizationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl std::error::Error for SynchronizationRule {}
