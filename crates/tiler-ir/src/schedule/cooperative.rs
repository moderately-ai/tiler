//! Cross-invocation cooperative dataflow for one bounded workgroup tile.
//!
//! A [`CooperativeTile`] states, in target-neutral terms, everything a workgroup
//! reduction needs before a synchronization point can be admitted: which
//! invocations participate, what local coordinate each one occupies, what
//! workgroup-shared storage the tile allocates, which phase writes and which
//! phase reads each allocation, how long an allocation stays live, and which
//! participant commits the region's owning write.
//!
//! # What this is and what it deliberately is not
//!
//! This module represents a *dependency* and names what discharges it. A tile
//! that writes staging in one phase and reads it in a later one carries a
//! [`VisibilityEdge`] for each such pair, and an edge is exactly the obligation
//! a [`SynchronizationPoint`] has to satisfy. The edges are still derived here
//! and never declared; the points are declared and their discharge is derived
//! from where they sit. A tile whose edges are not all discharged is refused by
//! the schedule verifier, and a kernel whose body does not separate a staged
//! write from its staged read by the barrier realizing the discharging point is
//! refused by the structured-kernel verifier — the two together are what keep a
//! representable tile from becoming an executable race.
//!
//! Nothing here claims any *target* can order the two phases. That is one atomic
//! provenance-bearing fact a target profile declares, composed against
//! [`super::synchronization::required_subject`] by a feasibility authority.
//!
//! # Why the relations are stated per participant
//!
//! The participants occupy a stated [`ParticipantSpace`] — per-dimension
//! extents, slowest-varying first — and every staged access is a
//! [`StagedSpan`]: the participant at coordinate `(l_0, .., l_{r-1})` addresses
//! the contiguous run of `count` slots beginning at
//! `offset + sum_d strides[d] * l_d`. That form is what makes disjointness and
//! coverage *decidable by enumeration* rather than by a modular argument: the
//! enumeration ranges over the Cartesian product of the extents, which is the
//! same participant set a linear run walks, re-indexed rather than multiplied.
//!
//! One stride per dimension is strictly more general than naming one coordinate
//! component and subsumes it — selecting component `d` is the stride vector that
//! is zero everywhere but `d`. So the three shapes a bounded tile needs are one
//! construct: one slot per participant on the producing side (rank one,
//! `strides = [1]`, `count = 1`), the whole staged set read by every participant
//! on the consuming side (rank one, `strides = [0]`, `count = participants`),
//! and a blocked operand tile's transposed staged write (rank two,
//! `strides = [1, 16]`, `count = 1`), whose profile no single-term relation over
//! a linear coordinate can express.
//!
//! # Rounds, and why the lifetime is not a field
//!
//! [`CooperativeTile::rounds`] states how many times the whole phase sequence
//! executes. `rounds > 1` is the loop-carried tile every blocked GPU kernel has:
//! the participants fill one fixed allocation, hand it off behind a point, and
//! *rewrite the same slots* on the next round.
//!
//! The allocation's lifetime therefore becomes round-scoped — and that is a
//! consequence of the round structure rather than a choice an allocation makes,
//! which is why no field carries it. Every phase runs on every round, so a phase
//! that writes an allocation writes it on every round; there is no way to state
//! "written once, read on every later round" in this vocabulary, so a
//! `live_from`/`live_through` pair is already a within-round lifetime and a
//! second field naming the scope would be a place to restate what the structure
//! determines. The one-writer-per-slot rule is unchanged for the same reason:
//! it enumerates the phase sequence once, which *is* one round, so it still
//! refuses a genuine double write inside a round and no longer refuses the
//! rewrite between them.
//!
//! What the rewrite does add is an obligation in the other direction. Round
//! `r + 1`'s write must not overtake round `r`'s read, which is an
//! [`AntiDependencyEdge`] — a second derived evidence class beside
//! [`VisibilityEdge`], discharged by exactly one point in exactly the same way,
//! and derived rather than declared for the same reason.
//!
//! # The broader space, and where this profile stops
//!
//! A logarithmic tree is still not statable, and the reason is now the *only*
//! remaining one: a [`StagedSpan`] is addressed by every participant of the tile
//! and is the same span on every round, so a write phase writes
//! `participants * count` slots whatever the round and the active lanes never
//! narrow. A log-depth tree needs two things this profile does not have — a
//! per-access active-participant subset, separate from a phase's
//! `participation`, which is *arrival* and must stay uniform for a point between
//! rounds to be convergent; and a span whose stride and count are functions of
//! the round ordinal, since each level halves them. Both are absent rather than
//! reserved, and neither is the rewrite rule that used to be blamed for the
//! depth: [`workgroup_tree_tile`] is the depth-two tree this profile states, and
//! it is depth two because of the subset and the varying span, not because a
//! slot may not be rewritten.
//!
//! The round ordinal is absent from the staging relation *deliberately*, and the
//! omission is a decision rather than an oversight (ADR 0097 decision 4). A
//! participant dimension indexes concurrent invocations — at one instant
//! different participants hold different values of it — while the round ordinal
//! indexes sequential iterations, at one instant identical across every
//! participant. The occupancy map that decides disjointness and coverage spans
//! the phase sequence once, which is exactly one round, and it is sound
//! *because* a span is the same on every round; a round-dependent span would
//! make per-round coverage a shrinking subset rather than a bijection, which is
//! a different decision procedure and not a wider parameter for this one.
//!
//! A stride *within* one participant's run is likewise absent. [`StagedSpan`]
//! already places participants' first slots by a per-dimension stride vector;
//! what each participant then addresses is [`StagedSpan::count`] contiguous
//! slots, and the blocked tile's transposed write is what keeps its reads
//! contiguous, so nothing needs the per-participant strided form yet.

use super::MAX_COOPERATIVE_PARTICIPANT_RANK;
use super::handles::{PhaseId, StagingId, SyncPointId};
use super::synchronization::{
    ConvergenceEvidence, SynchronizationPlacement, SynchronizationPoint, required_subject,
};

/// The order in which one cooperative tile's staged partials reach the
/// participant that combines them.
///
/// Deliberately not [`super::ContributorOrder`], which names the order of the
/// *original* contributor sequence one participant folds before staging
/// anything. This names what happens on the far side of the synchronization
/// point: which staged values the committing participant combines, and in what
/// order.
///
/// # Why it is stated, and why the distinction is a legality one
///
/// A tree over a *fixed* arrival order regroups the declared contributor
/// sequence without moving any contributor across a group boundary, which is
/// exactly reassociation. An arrival order the program does not fix — an atomic
/// accumulation into one location, or a collective whose combine order follows
/// scheduling — additionally reorders the contributors themselves, which is
/// contributor permutation. The two permissions are independent (ADR 0011), and
/// a strategy that checked reassociation and then used both would be admitted
/// for a freedom nobody granted. Carrying the arrival on the topology is what
/// makes that composition *statable*, and therefore refusable, instead of a
/// property a reader has to infer from the emitted body.
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder and the schedule
/// verifier map this totally, so a widened vocabulary must be a build error at
/// each rather than a silently admitted arrival.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContributorArrival {
    /// The staged partials are combined in ascending participant order.
    ///
    /// The only admitted arrival. Participant `p` stages the partial of the
    /// contiguous contributor range its partition owns, and the committing
    /// participant folds slots `0..participants` in ascending order — one
    /// deterministic sequence, fixed by the program rather than by the machine.
    AscendingParticipant,
    /// The partials are combined in whatever order participants arrive.
    ///
    /// Unadmitted, and refused by name rather than left unstatable. It requires
    /// contributor permutation *in addition to* reassociation, and it needs an
    /// arrival-ordered construct — a split-phase barrier or a collective — whose
    /// contract [`super::SynchronizationKind`] does not admit, so no point in
    /// this vocabulary could order it.
    NondeterministicArrival,
    /// Every participant accumulates into one shared location.
    ///
    /// Unadmitted, for both reasons above and one more: the ordering is
    /// established by [`super::SynchronizationKind::Atomic`], whose participant
    /// set and convergence proof are meaningless, so the point this tile
    /// declares could not be the one that orders it.
    AtomicAccumulation,
}

impl ContributorArrival {
    /// Returns whether this arrival consumes contributor permutation.
    ///
    /// An arrival the program fixes consumes reassociation alone; one it does
    /// not fix additionally permutes the contributor sequence. Written as an
    /// exhaustive match so a widened vocabulary must decide this rather than
    /// inherit the deterministic answer.
    #[must_use]
    pub const fn requires_permutation(self) -> bool {
        match self {
            Self::AscendingParticipant => false,
            Self::NondeterministicArrival | Self::AtomicAccumulation => true,
        }
    }

    /// Returns the canonical tag naming this arrival in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::AscendingParticipant => 0x01,
            Self::NondeterministicArrival => 0x02,
            Self::AtomicAccumulation => 0x03,
        }
    }

    /// Returns the stable identifier naming this arrival in an explanation.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::AscendingParticipant => "ascending-participant",
            Self::NondeterministicArrival => "nondeterministic-arrival",
            Self::AtomicAccumulation => "atomic-accumulation",
        }
    }
}

/// The element one workgroup staging allocation holds.
///
/// A vocabulary of its own rather than a [`super::ArithmeticType`]: an
/// allocation needs a *storage width* to derive its local-memory requirement,
/// and `ArithmeticType` deliberately carries no width because two arithmetic
/// formats can share one and differ in bias, special values, or encoding. This
/// names storage, so the width is part of what it means.
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder and the
/// local-memory derivation both map this totally, so a widened vocabulary must
/// be a build error at each rather than a silently mis-sized allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StagedElement {
    /// An IEEE-754 binary32 slot, four bytes wide.
    F32,
}

impl StagedElement {
    /// Returns the storage width of one slot in bytes.
    #[must_use]
    pub const fn storage_bytes(self) -> u64 {
        match self {
            Self::F32 => 4,
        }
    }

    /// Returns the canonical tag naming this element in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::F32 => 0x01,
        }
    }
}

/// The governed execution key a participant's local coordinate reads.
///
/// A key rather than a target spelling, exactly as
/// [`super::ExecutionBinding`] is: a backend maps a supported source onto its
/// own builtin or rejects it.
///
/// Deliberately not `#[non_exhaustive]`: the identity encoder maps this totally.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LocalCoordinateSource {
    /// The linear index of one invocation within its own workgroup.
    ///
    /// Its relation to a multi-dimensional participant space is the row-major
    /// decomposition against the extents, which is definitionally true: the
    /// linear index *is* the linearization.
    LocalLinearInvocation,
    /// The per-dimension position of one invocation within its own workgroup.
    ///
    /// The relation to [`Self::LocalLinearInvocation`] is defined and is the
    /// row-major decomposition against the participant extents, which is what
    /// separates this source from a subgroup-derived one: two vendor
    /// specifications decline to fix any relation between a subgroup coordinate
    /// and the linear index, so a source naming one may not claim this
    /// decomposition and must be a variant of its own when it lands.
    LocalWorkgroupPosition,
}

impl LocalCoordinateSource {
    /// Returns the canonical tag naming this source in an identity encoding.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::LocalLinearInvocation => 0x01,
            Self::LocalWorkgroupPosition => 0x02,
        }
    }
}

/// A contiguous run of local invocation coordinates.
///
/// Used for both the tile's participant set and each phase's reachable set, so
/// "every participant reaches this phase" is an equality between two values of
/// one type rather than a claim spelled twice in different shapes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParticipantRange {
    /// Inclusive first local coordinate in the run.
    pub first: u64,
    /// Number of local coordinates in the run.
    pub count: u64,
}

impl ParticipantRange {
    /// Returns the exclusive end of the run, or `None` when it overflows `u64`.
    #[must_use]
    pub const fn end(self) -> Option<u64> {
        self.first.checked_add(self.count)
    }

    /// Returns whether `other` lies entirely inside this run.
    #[must_use]
    pub fn contains_range(self, other: Self) -> bool {
        match (self.end(), other.end()) {
            (Some(outer), Some(inner)) => other.first >= self.first && inner <= outer,
            _ => false,
        }
    }
}

/// The shape of one cooperative tile's participant space.
///
/// Extents are stated slowest-varying first, so a participant's linear index is
/// the row-major linearization of its coordinate. The product is the participant
/// count, and it is what the intrinsic verifier compares against the launched
/// workgroup width — a first-class fact rather than a divisor embedded in an
/// address expression, which is the difference between a wrong tile width being
/// refused and one being admitted to emit a silently wrong broadcast.
///
/// # Why the extents are a fixed-rank inline array behind a constructor
///
/// [`MAX_COOPERATIVE_PARTICIPANT_RANK`] is a property of the domain rather than
/// a tuning parameter: a threadgroup is at most three-dimensional on every
/// target this repository names, and a fourth dimension would be a shape no
/// launch could declare. An owned `Vec` would model an unbounded sequence the
/// domain forbids, and would cost `Copy` on this type and on [`StagedSpan`],
/// [`StagedWrite`], [`StagedRead`], and [`LocalCoordinates`] to express a
/// generality that cannot exist.
///
/// The array and the rank are private because they are one fact stated in two
/// places, and only [`Self::new`] can make them agree. It also zeroes the unused
/// tail, which is what makes the derived [`Eq`] and [`Hash`] agree with the
/// canonical identity encoding: the encoding frames the rank and the *used*
/// extents only, so two spaces equal in meaning must not differ in a byte no
/// encoder reads.
///
/// Raising the ceiling later is a one-constant edit plus an identity recompute
/// rather than an API break, precisely because the array size sits behind the
/// constructor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParticipantSpace {
    /// Per-dimension extents, slowest-varying first, zero past `rank`.
    extents: [u64; MAX_COOPERATIVE_PARTICIPANT_RANK],
    /// Dimensions the space has, never above the array's length.
    rank: usize,
}

impl ParticipantSpace {
    /// Builds a participant space over `extents`, slowest-varying first.
    ///
    /// Returns `None` when the rank exceeds
    /// [`MAX_COOPERATIVE_PARTICIPANT_RANK`], which is the one condition the
    /// inline array makes unrepresentable rather than merely invalid. A zero
    /// extent, an empty space, and an overflowing extent product are all
    /// representable and are refused by the intrinsic verifier under
    /// [`CooperativeTileRule::LocalCoordinates`](super::CooperativeTileRule::LocalCoordinates)
    /// — a space is a *statement* a schedule makes, and a statement that cannot
    /// be made cannot be refused with an explanation.
    #[must_use]
    pub fn new(extents: &[u64]) -> Option<Self> {
        if extents.len() > MAX_COOPERATIVE_PARTICIPANT_RANK {
            return None;
        }
        let mut stored = [0_u64; MAX_COOPERATIVE_PARTICIPANT_RANK];
        stored[..extents.len()].copy_from_slice(extents);
        Some(Self {
            extents: stored,
            rank: extents.len(),
        })
    }

    /// Returns the per-dimension extents, slowest-varying first.
    ///
    /// The used prefix alone. The array's tail is an implementation detail of
    /// the fixed-rank storage and is never part of what this space means.
    #[must_use]
    pub fn extents(&self) -> &[u64] {
        &self.extents[..self.rank]
    }

    /// Returns the number of dimensions.
    #[must_use]
    pub const fn rank(&self) -> usize {
        self.rank
    }

    /// Returns the number of participants, or `None` when the product overflows.
    ///
    /// The empty space's product is `1`, which is the arithmetic identity and
    /// deliberately not a refusal here: a rank-zero space is a malformed
    /// *statement*, which the intrinsic verifier names, rather than an
    /// arithmetic failure this method could explain.
    #[must_use]
    pub fn participants(&self) -> Option<u64> {
        self.extents()
            .iter()
            .try_fold(1_u64, |total, extent| total.checked_mul(*extent))
    }
}

/// How one cooperative tile derives each participant's local coordinate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocalCoordinates {
    /// Governed execution key the coordinate reads.
    pub source: LocalCoordinateSource,
    /// Shape of the space the tile's participants occupy.
    ///
    /// A space and not also a [`ParticipantRange`], because the run is what the
    /// shape determines: its start is zero and its length is the extent
    /// product, so a range beside this would be a second place to state one
    /// fact and a place for two producers to disagree. A phase's reachable set,
    /// a synchronization point's participant set, and the committing
    /// participant *are* contiguous runs over the linearized space, because
    /// each is a claim about which invocations reach a program point rather
    /// than about the shape they are arranged in — which is why those three
    /// keep [`ParticipantRange`] and this does not.
    pub participants: ParticipantSpace,
}

/// The staging slots one participant addresses in one phase.
///
/// The participant at coordinate `(l_0, .., l_{r-1})` addresses the `count`
/// contiguous slots beginning at `offset + sum_d strides[d] * l_d`. One stride
/// per participant dimension, in the same axis order as the tile's extents: a
/// stride of `1` on the fastest-varying dimension and `0` elsewhere gives each
/// participant of a row its own slot, and a stride of `0` on every dimension has
/// every participant address one shared run.
///
/// A span whose stride count differs from the participant rank is refused by
/// [`CooperativeTileRule::SpanRank`](super::CooperativeTileRule::SpanRank)
/// rather than padded or truncated, because a well-formed span and a well-formed
/// space that disagree about how many dimensions there are are not wrong on
/// their own terms and silently repairing either would reinterpret what the
/// other denotes.
///
/// The strides are private behind [`Self::new`] for the reason
/// [`ParticipantSpace`]'s extents are, and this type keeps `Copy` for the same
/// reason.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StagedSpan {
    /// Slots between the first slots of participants adjacent along each
    /// dimension, in axis order, zero past `rank`.
    strides: [u64; MAX_COOPERATIVE_PARTICIPANT_RANK],
    /// Dimensions the stride vector has, never above the array's length.
    rank: usize,
    /// First slot the participant at the origin addresses.
    pub offset: u64,
    /// Contiguous slots each participant addresses.
    pub count: u64,
}

impl StagedSpan {
    /// Builds a staged span with one stride per participant dimension.
    ///
    /// Returns `None` when the stride count exceeds
    /// [`MAX_COOPERATIVE_PARTICIPANT_RANK`]. A `count` of zero and a stride
    /// vector whose rank disagrees with the tile's are both representable and
    /// are refused by the intrinsic verifier, under
    /// [`CooperativeTileRule::StagingCapacity`](super::CooperativeTileRule::StagingCapacity)
    /// and
    /// [`CooperativeTileRule::SpanRank`](super::CooperativeTileRule::SpanRank)
    /// respectively — neither is a property of a span alone.
    #[must_use]
    pub fn new(strides: &[u64], offset: u64, count: u64) -> Option<Self> {
        if strides.len() > MAX_COOPERATIVE_PARTICIPANT_RANK {
            return None;
        }
        let mut stored = [0_u64; MAX_COOPERATIVE_PARTICIPANT_RANK];
        stored[..strides.len()].copy_from_slice(strides);
        Some(Self {
            strides: stored,
            rank: strides.len(),
            offset,
            count,
        })
    }

    /// Returns the per-dimension strides, in the tile's own axis order.
    ///
    /// The used prefix alone, for the reason
    /// [`ParticipantSpace::extents`] returns one.
    #[must_use]
    pub fn strides(&self) -> &[u64] {
        &self.strides[..self.rank]
    }

    /// Returns the number of dimensions the stride vector states.
    #[must_use]
    pub const fn rank(&self) -> usize {
        self.rank
    }
}

/// One workgroup-shared staging allocation of a cooperative tile.
///
/// The lifetime this carries is a *within-round* one, and on a tile with several
/// rounds the allocation is reborn at the start of each. There is deliberately
/// no field naming that scope: every phase runs on every round, so an allocation
/// written by a phase is written on every round whatever a scope field claimed,
/// and the field would be a second place to state what
/// [`CooperativeTile::rounds`] already determines.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorkgroupStaging {
    /// Tile-local ordinal naming this allocation.
    pub id: StagingId,
    /// Element every slot holds.
    pub element: StagedElement,
    /// Addressable slots.
    pub slots: u64,
    /// First phase of a round in which this allocation may be written.
    pub live_from: PhaseId,
    /// Last phase of a round in which this allocation may be read.
    ///
    /// The lifetime is declared rather than inferred from the accesses, so a
    /// phase reading an allocation the tile considers dead is a rejectable
    /// statement instead of a silent extension of the allocation's life.
    pub live_through: PhaseId,
}

/// One phase's write of a staging allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StagedWrite {
    /// Allocation written.
    pub staging: StagingId,
    /// Slots each participant writes.
    pub span: StagedSpan,
}

/// One phase's read of a staging allocation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StagedRead {
    /// Allocation read.
    pub staging: StagingId,
    /// Slots each participant reads.
    pub span: StagedSpan,
}

/// One phase of a cooperative tile.
///
/// A phase is a maximal span of participant-local work between two points at
/// which staged values change hands. `participation` is stated per phase rather
/// than inherited from the tile precisely so a non-uniformly reached phase is
/// *expressible* and therefore rejectable: a synchronization point inside a
/// phase some participants skip is divergent, and a model that could not state
/// the divergence could not refuse it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CooperativePhase {
    /// Tile-local ordinal naming this phase.
    pub id: PhaseId,
    /// Local coordinates that reach this phase.
    pub participation: ParticipantRange,
    /// Staging this phase writes.
    pub writes: Vec<StagedWrite>,
    /// Staging this phase reads.
    pub reads: Vec<StagedRead>,
}

/// One staged value's producer-to-consumer dependency across invocations.
///
/// This is the entire content of "the exact dependency that requires
/// visibility": the values participant `p` wrote to `staging` in `produced_in`
/// are read by other participants in `consumed_in`, and nothing in the tile
/// orders the two. A synchronization authority discharges an edge; this module
/// only derives them.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VisibilityEdge {
    /// Allocation whose values cross invocations.
    pub staging: StagingId,
    /// Phase whose participants wrote the values.
    pub produced_in: PhaseId,
    /// Phase whose participants read them.
    pub consumed_in: PhaseId,
}

/// One staging allocation's cross-round anti-dependency.
///
/// The second derived evidence class, and the one a loop-carried tile needs that
/// a single-round tile cannot state at all: the values participant `p` read from
/// `staging` in `consumed_in` are still being read when round `r + 1`'s
/// `rewritten_in` overwrites the same slots, and nothing in the tile orders the
/// two. It is an *anti*-dependency rather than a [`VisibilityEdge`] — no value
/// crosses it, only the storage is reused — which is why it is a separate type
/// rather than an edge with the ends swapped: swapping the ends of a visibility
/// edge would claim a value flows backwards, and a reader would have to know
/// which direction each instance meant.
///
/// There is no ordinal comparison between the two phases, and that absence is
/// the content: the rewrite is in the *next* round, so it follows the read in
/// program order however the two phases are ordered within a round. A tile whose
/// read is in phase 1 and whose rewrite is in phase 0 has the edge, and so does
/// one whose read is in phase 1 and whose rewrite is in phase 2. A read in phase
/// 0 is what cannot occur: `StagedProducer` requires every read's writer in a
/// strictly earlier phase, so the first phase never reads at all.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AntiDependencyEdge {
    /// Allocation whose slots are reused.
    pub staging: StagingId,
    /// Phase whose participants read the values a later round overwrites.
    pub consumed_in: PhaseId,
    /// Phase of the following round whose participants overwrite them.
    pub rewritten_in: PhaseId,
}

/// The cross-invocation dataflow of one bounded cooperative workgroup tile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CooperativeTile {
    /// Participants and the local coordinate space they occupy.
    pub coordinates: LocalCoordinates,
    /// Times the whole phase sequence executes.
    ///
    /// `1` is the single-pass tile: the phases run once, no allocation is
    /// rewritten, and the tile derives no [`AntiDependencyEdge`]. A larger count
    /// is the loop-carried tile — the phases are a round body, each round
    /// rewrites the slots the previous one read, and the anti-dependencies that
    /// creates must each be discharged.
    ///
    /// A declared literal rather than a value, and that is what makes a point
    /// inside the round loop convergent: every participant of the workgroup runs
    /// the identical trip count, so they all reach the same dynamic instance of
    /// the point. A round count read from a buffer or derived per invocation
    /// would carry no such proof, which is why this is a `u64` here and not an
    /// index expression.
    pub rounds: u64,
    /// Workgroup-shared allocations, in ascending [`StagingId`] order.
    pub staging: Vec<WorkgroupStaging>,
    /// Phases, in ascending [`PhaseId`] order.
    pub phases: Vec<CooperativePhase>,
    /// Synchronization points, in ascending [`SyncPointId`] order.
    ///
    /// Declared rather than derived, because *where* to order a handoff is a
    /// physical decision with more than one correct answer — a tile with three
    /// phases and two handoffs may separate them with one point or two — and a
    /// derivation would silently pick one and make the alternative unstatable.
    /// What is derived is which edges each point discharges, so the declaration
    /// is checked against the dependency rather than trusted beside it. That
    /// holds for both evidence classes: a point at
    /// [`SynchronizationPlacement::RoundBoundary`]
    /// orders every [`AntiDependencyEdge`] the tile has, and on a tile with
    /// enough phases an ordinary boundary may already fall between a read and
    /// the next round's rewrite.
    pub synchronization: Vec<SynchronizationPoint>,
    /// Participants that perform the region's owning write.
    ///
    /// Carried by the tile rather than left to the ownership proof: the proof
    /// states that exactly one global invocation writes each output position,
    /// and in a tile several invocations reach the commit phase, so which of
    /// them stores is the fact that makes the proof true.
    pub commit: ParticipantRange,
}

impl CooperativeTile {
    /// Returns every cross-invocation visibility dependency this tile requires.
    ///
    /// One edge per (allocation, producing phase, consuming phase) triple with
    /// the producing phase strictly earlier. A tile whose reads are all in the
    /// phase that wrote them needs no edge — and needs no synchronization — so
    /// an empty result is a claim, not a missing derivation.
    #[must_use]
    pub fn visibility_edges(&self) -> Vec<VisibilityEdge> {
        let mut edges = Vec::new();
        for consumer in &self.phases {
            for read in &consumer.reads {
                for producer in &self.phases {
                    if producer.id >= consumer.id {
                        continue;
                    }
                    if producer
                        .writes
                        .iter()
                        .any(|write| write.staging == read.staging)
                    {
                        edges.push(VisibilityEdge {
                            staging: read.staging,
                            produced_in: producer.id,
                            consumed_in: consumer.id,
                        });
                    }
                }
            }
        }
        edges.sort_unstable_by_key(|edge| {
            (
                edge.staging.get(),
                edge.produced_in.get(),
                edge.consumed_in.get(),
            )
        });
        edges.dedup();
        edges
    }

    /// Returns every cross-round anti-dependency this tile requires.
    ///
    /// One edge per (allocation, reading phase, rewriting phase) triple, over
    /// every pair of phases regardless of their order, because the rewrite
    /// happens in the *following* round and therefore follows every read of the
    /// current one. A single-round tile returns nothing — not because the
    /// derivation is missing, but because no round follows the only one.
    #[must_use]
    pub fn anti_dependency_edges(&self) -> Vec<AntiDependencyEdge> {
        let mut edges = Vec::new();
        if self.rounds <= 1 {
            return edges;
        }
        for consumer in &self.phases {
            for read in &consumer.reads {
                for producer in &self.phases {
                    if producer
                        .writes
                        .iter()
                        .any(|write| write.staging == read.staging)
                    {
                        edges.push(AntiDependencyEdge {
                            staging: read.staging,
                            consumed_in: consumer.id,
                            rewritten_in: producer.id,
                        });
                    }
                }
            }
        }
        edges.sort_unstable_by_key(|edge| {
            (
                edge.staging.get(),
                edge.consumed_in.get(),
                edge.rewritten_in.get(),
            )
        });
        edges.dedup();
        edges
    }

    /// Returns the points that discharge `edge`, in declaration order.
    ///
    /// The schedule verifier requires exactly one, so a caller holding a
    /// verified region may take the single element. The plural form is what lets
    /// the verifier tell "no point orders this handoff" apart from "two points
    /// order it", which are different defects with different fixes.
    #[must_use]
    pub fn discharging_points(&self, edge: VisibilityEdge) -> Vec<&SynchronizationPoint> {
        self.synchronization
            .iter()
            .filter(|point| point.discharges(edge))
            .collect()
    }

    /// Returns the points that discharge `edge`, in declaration order.
    ///
    /// The anti-dependency counterpart of [`Self::discharging_points`], and
    /// plural for the same reason: the verifier requires exactly one, and needs
    /// to tell an unordered rewrite apart from a doubly ordered one.
    #[must_use]
    pub fn anti_discharging_points(&self, edge: AntiDependencyEdge) -> Vec<&SynchronizationPoint> {
        self.synchronization
            .iter()
            .filter(|point| point.discharges_anti(edge))
            .collect()
    }

    /// Returns the workgroup memory this tile allocates, in bytes.
    ///
    /// Returns `None` when the product exceeds `u64`, which the schedule
    /// verifier reports rather than saturating: a saturated requirement would be
    /// composed against a target profile as if it were the real one.
    #[must_use]
    pub fn local_memory_bytes(&self) -> Option<u64> {
        self.staging.iter().try_fold(0_u64, |total, staging| {
            staging
                .slots
                .checked_mul(staging.element.storage_bytes())
                .and_then(|bytes| total.checked_add(bytes))
        })
    }

    /// Returns the slots every participant addresses through `span`, in the
    /// space's own row-major participant order.
    ///
    /// The walk ranges over the Cartesian product of the extents, which is the
    /// same participant set a linear run walks — re-indexed, not multiplied — so
    /// the governed participant and staging-slot bounds continue to bound it
    /// exactly.
    ///
    /// `None` when the ranks disagree, when the participant count overflows, or
    /// when any address exceeds `u64`. The caller separates the first from the
    /// rest, because a rank disagreement is
    /// [`CooperativeTileRule::SpanRank`](super::CooperativeTileRule::SpanRank)
    /// while an unrepresentable address is the same capacity refusal an
    /// out-of-range slot receives — both of the latter mean the span leaves the
    /// allocation.
    pub(super) fn addressed_slots(
        participants: ParticipantSpace,
        span: StagedSpan,
    ) -> Option<Vec<u64>> {
        if participants.rank() != span.rank() {
            return None;
        }
        let extents = participants.extents();
        let strides = span.strides();
        let count = participants.participants()?;
        let mut slots = Vec::new();
        // The row-major decomposition of the linear participant index, which is
        // the relation `LocalCoordinateSource` states: the last extent varies
        // fastest, so the quotient chain runs from the end of the vector back.
        for linear in 0..count {
            let mut remaining = linear;
            let mut base = span.offset;
            for (extent, stride) in extents.iter().zip(strides).rev() {
                // A zero extent makes the product zero, so this loop is not
                // reached; the verifier refuses the space before the walk.
                let coordinate = remaining % *extent;
                remaining /= *extent;
                base = stride
                    .checked_mul(coordinate)
                    .and_then(|scaled| base.checked_add(scaled))?;
            }
            for step in 0..span.count {
                slots.push(base.checked_add(step)?);
            }
        }
        Some(slots)
    }
}

/// Builds the canonical cooperative tile of a single-workgroup tree reduction.
///
/// # The tree, stated level by level
///
/// | Level | Active lanes | Fan-in per lane | Where the result goes |
/// | --- | --- | --- | --- |
/// | 0 | every participant | the partition's contributors | slot `l` of the staging allocation |
/// | 1 | the one committing participant | `participants` | the region's owning write |
///
/// Level 0's fan-in is the region's
/// [`ContributorPartition::contributors_per_partition`](super::ContributorPartition),
/// which this constructor does not see and does not need: the tile states the
/// *staged dataflow*, and the schedule verifier is what requires the partition
/// to have exactly `participants` partitions covering the contributor sequence
/// once each. Level 1's fan-in is therefore the participant count, and the total
/// depth is two.
///
/// **Active lanes narrow between the levels, and the narrowing is stated by
/// [`CooperativeTile::commit`] rather than by a span.** Every phase is reached
/// by every participant — that is the tile's own uniform-convergence rule, and
/// it is what makes the point between the levels convergent — while only the
/// committing participant performs level 1's fold and the owning write.
///
/// # Why the depth is two and not `log2(participants)`
///
/// A logarithmic tree narrows the *writing* lanes at every round and halves the
/// span each level addresses, and this vocabulary can state neither. A
/// [`StagedSpan`] is addressed by every participant of the tile — the slot
/// enumeration runs over the tile's whole participant space, not over a
/// per-access subset — so every write phase writes exactly `participants *
/// count` slots however few lanes are meant to be doing useful work, and the
/// same span is addressed on every round because a span carries no dependence on
/// the round ordinal. Rewriting one slot across rounds is *not* what blocks it:
/// [`CooperativeTile::rounds`] admits exactly that. Stated here rather than left
/// to be rediscovered, because "tree" is otherwise read as "logarithmic".
///
/// # Tail handling
///
/// There is none, and that is the contract: the split must cover the contributor
/// sequence exactly once each, so a contributor count with no exact split of
/// `participants` partitions is *declined by the strategy that chooses the
/// count*, never padded with identity elements or truncated. A masked tail lane
/// would additionally break the emitted body's soundness argument, which rests
/// on every launched invocation reaching the staged store.
///
/// Returns `None` when `participants` is below two — a single participant stages
/// values it reads back itself, which the synchronization authority refuses as
/// the semantically redundant barrier — or above
/// [`MAX_COOPERATIVE_PARTICIPANTS`](super::MAX_COOPERATIVE_PARTICIPANTS), or
/// when the participant run does not fit the tile's ordinal space.
#[must_use]
pub fn workgroup_tree_tile(participants: u64) -> Option<CooperativeTile> {
    if !(2..=super::MAX_COOPERATIVE_PARTICIPANTS).contains(&participants) {
        return None;
    }
    let range = ParticipantRange {
        first: 0,
        count: participants,
    };
    range.end()?;
    // Rank one, so the tile's participant coordinate *is* its linear index and
    // the space states nothing the range did not. The tree is a linear
    // construct; the rank-general relation is what a blocked operand tile needs,
    // and nothing here pretends this fold has a second dimension.
    let space = ParticipantSpace::new(&[participants])?;
    let staging = StagingId::FIRST;
    let produce = PhaseId::FIRST;
    let consume = PhaseId::new(1);
    let tile = CooperativeTile {
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalLinearInvocation,
            participants: space,
        },
        // One pass over the phases. The tree stages each participant's partial
        // once and reads the set back once, so no slot is ever rewritten and the
        // tile carries no anti-dependency.
        rounds: 1,
        staging: vec![WorkgroupStaging {
            id: staging,
            element: StagedElement::F32,
            slots: participants,
            live_from: produce,
            live_through: consume,
        }],
        phases: vec![
            CooperativePhase {
                id: produce,
                participation: range,
                writes: vec![StagedWrite {
                    staging,
                    // One slot per participant: participant `l` writes slot `l`.
                    span: StagedSpan::new(&[1], 0, 1)?,
                }],
                reads: Vec::new(),
            },
            CooperativePhase {
                id: consume,
                participation: range,
                writes: Vec::new(),
                reads: vec![StagedRead {
                    staging,
                    // The whole staged set, which is what makes the combining
                    // level's fan-in the participant count.
                    span: StagedSpan::new(&[0], 0, participants)?,
                }],
            },
        ],
        // Filled in below from the edges this dataflow derives, so the point's
        // subject is the one the handoff requires rather than one restated here.
        synchronization: Vec::new(),
        commit: ParticipantRange { first: 0, count: 1 },
    };
    let subject = required_subject(&tile.visibility_edges())?;
    Some(CooperativeTile {
        synchronization: vec![SynchronizationPoint {
            id: SyncPointId::FIRST,
            subject,
            placement: SynchronizationPlacement::PhaseBoundary {
                preceding: produce,
                following: consume,
            },
            participants: range,
            convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
        }],
        ..tile
    })
}
