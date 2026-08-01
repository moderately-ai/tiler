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
//! A tile whose staging is rewritten between rounds carries a second derived
//! class, [`AntiDependencyEdge`](super::AntiDependencyEdge), and a point
//! discharges one exactly as it discharges a visibility edge — from where it
//! sits, never from a declared edge set.
//!
//! # Why the point lives inside the tile
//!
//! A point is placed at a boundary between phases, or between rounds, and both
//! exist only inside a cooperative tile. Putting the list beside the topology on
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

use super::cooperative::{AntiDependencyEdge, ParticipantRange, VisibilityEdge};
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
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder and the schedule
/// verifier map this totally, so a widened vocabulary must be a build error at
/// each rather than a silently admitted position.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SynchronizationPlacement {
    /// Between the last effect of `preceding` and the first of `following`.
    PhaseBoundary {
        /// Phase whose effects are released at this point.
        preceding: PhaseId,
        /// Phase whose effects acquire them.
        following: PhaseId,
    },
    /// Between the last phase of one round and the first phase of the next.
    ///
    /// It carries no phase ordinals, and that is what makes it a single
    /// spelling: the phases it separates are the tile's last and first, which
    /// the tile already states, so naming them here would be a second place to
    /// say it and two tiles ordering their rounds identically could differ in
    /// identity. It is not a [`Self::PhaseBoundary`] wrapping around, because the
    /// verifier's `preceding + 1 == following` rule is what makes a phase
    /// boundary a program point at all, and a wrap-around spelling would have to
    /// carve an exception into it.
    ///
    /// A point here exists only on a tile with more than one round: on a
    /// single-round tile no round follows, so it orders nothing and is refused as
    /// [`SynchronizationRule::RedundantPoint`].
    RoundBoundary,
}

impl SynchronizationPlacement {
    /// Returns the canonical tag naming this placement in an identity encoding.
    ///
    /// `0x02` is appended: `0x01` keeps its tag and its two phase ordinals, so a
    /// phase boundary encodes exactly the bytes it always did.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::PhaseBoundary { .. } => 0x01,
            Self::RoundBoundary => 0x02,
        }
    }

    /// Returns the phase whose effects this placement releases, if it names one.
    ///
    /// `None` for [`Self::RoundBoundary`], whose separated phases are the tile's
    /// last and first rather than a pair the placement carries. An `Option`
    /// rather than a defaulted ordinal, because every caller has to decide what
    /// a placement without ordinals means and a default would let one of them
    /// silently compare against phase zero.
    #[must_use]
    pub const fn preceding(self) -> Option<PhaseId> {
        match self {
            Self::PhaseBoundary { preceding, .. } => Some(preceding),
            Self::RoundBoundary => None,
        }
    }

    /// Returns the phase whose effects acquire at this placement, if it names one.
    #[must_use]
    pub const fn following(self) -> Option<PhaseId> {
        match self {
            Self::PhaseBoundary { following, .. } => Some(following),
            Self::RoundBoundary => None,
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
    ///
    /// Sufficient only on a single-round tile. On a tile whose phases repeat,
    /// reaching a point is no longer the same as reaching the *same dynamic
    /// instance* of it, and this class does not carry that second fact — which
    /// is why the verifier requires
    /// [`Self::EveryParticipantExecutesEveryRound`] there and refuses this one.
    EveryParticipantReachesThePoint,
    /// Derived: the above, plus a round count no participant can disagree about.
    ///
    /// The evidence a point inside the round loop needs. A control barrier is
    /// defined only when every participant arrives at the same dynamic instance,
    /// and on a repeating phase sequence that additionally requires an identical
    /// trip count. [`CooperativeTile::rounds`](super::CooperativeTile::rounds) is
    /// a declared literal on the tile rather than a loaded or per-invocation
    /// value, so every participant executes the same number of rounds *by
    /// construction* — the derivation is the one fact the field's type already
    /// establishes, checked here against the tile that carries it.
    ///
    /// A separate class rather than a widening of the one above, because the two
    /// prove different things and a point that named the weaker one on a
    /// repeating tile would be claiming a proof it does not have.
    EveryParticipantExecutesEveryRound,
    /// Asserted by the producer with no derivation.
    ///
    /// Always refused. It exists so that "the caller said so" is a statement the
    /// model can make and the verifier can reject by name, instead of a
    /// possibility the type system silently forecloses and no test can drive.
    CallerAsserted,
}

impl ConvergenceEvidence {
    /// Returns the canonical tag naming this evidence in an identity encoding.
    ///
    /// `0x03` is appended: the two earlier classes keep their tags, so a point
    /// that named one encodes exactly the byte it always did.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::EveryParticipantReachesThePoint => 0x01,
            Self::CallerAsserted => 0x02,
            Self::EveryParticipantExecutesEveryRound => 0x03,
        }
    }

    /// Returns the evidence class a point on a tile of `rounds` rounds requires.
    ///
    /// One definition, read by the verifier and available to a producer building
    /// a tile, so the two cannot disagree about which derivation a round count
    /// demands.
    #[must_use]
    pub const fn required_for_rounds(rounds: u64) -> Self {
        if rounds > 1 {
            Self::EveryParticipantExecutesEveryRound
        } else {
            Self::EveryParticipantReachesThePoint
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
    /// A round boundary discharges none of them. It sits after the last phase of
    /// a round, so it separates one round's effects from the next round's — and
    /// a visibility edge's producer and consumer are both inside *one* round,
    /// with the boundary either wholly before or wholly after the pair.
    ///
    /// Derived rather than declared: an edge set on the point would be a second
    /// place to state what the placement already determines, and the two could
    /// disagree.
    #[must_use]
    pub const fn discharges(&self, edge: VisibilityEdge) -> bool {
        match self.placement {
            SynchronizationPlacement::PhaseBoundary {
                preceding,
                following,
            } => {
                edge.produced_in.get() <= preceding.get()
                    && edge.consumed_in.get() >= following.get()
            }
            SynchronizationPlacement::RoundBoundary => false,
        }
    }

    /// Returns whether this point orders `edge`'s read before the next round's
    /// rewrite.
    ///
    /// A round boundary discharges every anti-dependency the tile has: it is the
    /// one program point every effect of round `r` precedes and every effect of
    /// round `r + 1` follows.
    ///
    /// A phase boundary discharges one when it falls between the two in program
    /// order, which — the rewrite being in the following round — happens in
    /// either of two ways: the boundary is at or after the reading phase, so it
    /// runs after the read in round `r`; or it is at or before the rewriting
    /// phase, so it runs before the rewrite in round `r + 1`. The disjunction is
    /// what makes this different from [`Self::discharges`]'s conjunction, and it
    /// is why a tile with three or more phases may need no round boundary at all.
    #[must_use]
    pub const fn discharges_anti(&self, edge: AntiDependencyEdge) -> bool {
        match self.placement {
            SynchronizationPlacement::PhaseBoundary {
                preceding,
                following,
            } => {
                preceding.get() >= edge.consumed_in.get()
                    || following.get() <= edge.rewritten_in.get()
            }
            SynchronizationPlacement::RoundBoundary => true,
        }
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
/// It is written over the *visibility* edges alone, and a tile's
/// anti-dependencies deliberately add no dimension to it. An anti-dependency
/// needs the same participants to arrive and the same workgroup allocations
/// fenced — it crosses no allocation the visibility edges do not — so composing
/// it in could only reproduce the same subject or, if a later widening made it
/// differ, require a realization no single point could state. A tile that has an
/// anti-dependency always has a visibility edge as well, because a read must
/// have a producer in an earlier phase before the tile is well formed, so this
/// never faces a tile whose only obligation is an anti-dependency.
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
    /// Convergence does not rest on the derivation the tile's rounds require.
    ///
    /// Either the point names a caller's assertion, which no tile admits, or it
    /// names the single-round derivation on a tile whose phases repeat — a
    /// weaker proof than a point inside the round loop needs, and one that would
    /// leave a barrier reached by participants at different rounds.
    ConvergenceEvidence,
    /// A phase the point bounds is not reached by every participant.
    Convergence,
    /// A cross-invocation visibility edge is discharged by no point.
    UndischargedVisibility,
    /// A cross-round anti-dependency is discharged by no point.
    ///
    /// Separate from [`Self::UndischargedVisibility`] because the two name
    /// different defects with different fixes: an undischarged visibility edge
    /// means a reader may see a value that was never published, and an
    /// undischarged anti-dependency means a writer may destroy a value that has
    /// not finished being read. A producer told only "something is unordered"
    /// would not know which end of the handoff to move its point to.
    UndischargedAntiDependency,
    /// A point discharges an edge another point already discharges, or none.
    ///
    /// Both halves are canonicality. A second point over one edge is a schedule
    /// that says the same thing twice and would have two identities for one
    /// realization; a point that discharges nothing is a synchronization the
    /// program does not need, and admitting it would let a schedule consume a
    /// target authority for an operation it has no reason to perform.
    ///
    /// "Discharges nothing" ranges over *both* evidence classes. A point that
    /// orders no visibility edge is not redundant if it discharges an
    /// anti-dependency — that is exactly what a round boundary does, and the
    /// tiled kernels this exists for declare one — so the check that a point
    /// earns its keep asks both questions before refusing it.
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
            Self::UndischargedAntiDependency => "synchronization-undischarged-anti-dependency",
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
