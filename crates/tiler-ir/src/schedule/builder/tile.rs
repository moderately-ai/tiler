//! A cooperative tile's participant space, staging dataflow, and handoff order.
//!
//! The dataflow rules and the synchronization authority stay in one file
//! because the authority is stated against the edges the dataflow derives: it
//! decides whether anything legally discharges each visibility and
//! anti-dependency obligation, and it can only do that over edges that already
//! exist. Nothing here reads what is being reduced — that is
//! [`super::reduction`]'s half — so a tile's dataflow is well formed or not
//! independently of the fold it realizes.

use crate::schedule::cooperative::{
    AntiDependencyEdge, CooperativeTile, ParticipantRange, ParticipantSpace, StagedSpan,
    VisibilityEdge, WorkgroupStaging,
};
use crate::schedule::error::{CooperativeTileRule, ScheduledRegionDiagnostic};
use crate::schedule::handles::{PhaseId, StagingId};
use crate::schedule::synchronization::{
    ConvergenceEvidence, SynchronizationRule, required_subject,
};
use crate::schedule::{
    MAX_COOPERATIVE_PARTICIPANTS, MAX_COOPERATIVE_PHASE_ACCESSES, MAX_COOPERATIVE_PHASES,
    MAX_COOPERATIVE_STAGING_SLOTS, MAX_COOPERATIVE_SYNCHRONIZATION_POINTS,
};

use super::diagnostics::{cooperative, synchronization};

/// Decides a cooperative tile's participant space and its agreement with the
/// launch.
///
/// Two rules, together because they are the two ways a space can be wrong before
/// anything reads the participant count: the space is not a space at all, or it
/// is a well-formed space over a different number of invocations than the
/// workgroup launches.
///
/// A rank above
/// [`MAX_COOPERATIVE_PARTICIPANT_RANK`](crate::schedule::MAX_COOPERATIVE_PARTICIPANT_RANK)
/// is deliberately not decided here: `ParticipantSpace::new` makes it
/// unrepresentable, so a check for it could never fail.
pub(super) fn verify_participant_space(
    space: ParticipantSpace,
    threads_per_workgroup: u32,
) -> Result<(), ScheduledRegionDiagnostic> {
    // An empty space names no participants; a zero extent gives it none; and a
    // product that overflows `u64` is a count no launch could hold. Each is a
    // malformed statement rather than a disagreement with something else, so
    // they share the rule that names the space.
    let Some(participants) = space
        .participants()
        .filter(|_| space.rank() != 0 && space.extents().iter().all(|extent| *extent != 0))
    else {
        return Err(cooperative(CooperativeTileRule::LocalCoordinates));
    };
    // Uniform convergence. Every launched invocation of the workgroup is a
    // participant, so a synchronization point placed in any phase is one they
    // all reach. The extent *product* is what the launch is compared against —
    // the shape the participants are arranged in is a fact the launch geometry
    // does not yet carry, which is why this stays a product equality and why a
    // `[4, 64]` tile and a `[16, 16]` tile are equally admissible against a
    // 256-thread workgroup.
    if participants != u64::from(threads_per_workgroup) {
        return Err(cooperative(CooperativeTileRule::ParticipantConvergence));
    }
    Ok(())
}

/// Verifies one cooperative tile's cross-invocation dataflow.
///
/// Every rule here is decided by enumerating the slots each participant
/// addresses, which is why the governed bounds are checked first: enumeration is
/// what makes disjointness and coverage exact instead of a modular argument, and
/// an unbounded tile would make it unbounded work.
pub(super) fn verify_cooperative_tile(
    tile: &CooperativeTile,
) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = cooperative_participants(tile)?;
    // Exactly one participant performs the region's owning write, which is what
    // makes `OneGlobalInvocationPerOutput` true of a workgroup that runs several
    // invocations over one output position.
    if tile.commit.count != 1 || !participants.contains_range(tile.commit) {
        return Err(cooperative(CooperativeTileRule::CommitOwnership));
    }
    verify_cooperative_tile_dataflow(tile, participants)
}

/// Verifies an operand-sharing tile: every participant commits its own write.
pub(super) fn verify_operand_tile(tile: &CooperativeTile) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = cooperative_participants(tile)?;
    if tile.commit != participants {
        return Err(cooperative(CooperativeTileRule::OperandTileCommit));
    }
    verify_cooperative_tile_dataflow(tile, participants)
}

fn cooperative_participants(
    tile: &CooperativeTile,
) -> Result<ParticipantRange, ScheduledRegionDiagnostic> {
    let space = tile.coordinates.participants;
    let participant_count = space
        .participants()
        .ok_or_else(|| cooperative(CooperativeTileRule::LocalCoordinates))?;
    Ok(ParticipantRange {
        first: 0,
        count: participant_count,
    })
}

fn verify_cooperative_tile_dataflow(
    tile: &CooperativeTile,
    participants: ParticipantRange,
) -> Result<(), ScheduledRegionDiagnostic> {
    let space = tile.coordinates.participants;
    if participants.count > MAX_COOPERATIVE_PARTICIPANTS
        || tile.phases.len() > MAX_COOPERATIVE_PHASES
        || tile
            .phases
            .iter()
            .any(|phase| phase.writes.len() > MAX_COOPERATIVE_PHASE_ACCESSES)
        || tile
            .phases
            .iter()
            .any(|phase| phase.reads.len() > MAX_COOPERATIVE_PHASE_ACCESSES)
        || tile
            .staging
            .iter()
            .try_fold(0_u64, |total, staging| total.checked_add(staging.slots))
            .is_none_or(|slots| slots > MAX_COOPERATIVE_STAGING_SLOTS)
    {
        return Err(cooperative(CooperativeTileRule::StructuralLimit));
    }

    let phase_count = u32::try_from(tile.phases.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;
    if tile.phases.is_empty()
        || tile
            .phases
            .iter()
            .enumerate()
            .any(|(position, phase)| u32::try_from(position) != Ok(phase.id.get()))
    {
        return Err(cooperative(CooperativeTileRule::PhaseSequence));
    }
    // Nonuniform reachability is stated per phase precisely so it can be
    // refused: a synchronization point inside a phase some participants skip is
    // divergent, and this is where a tile that would place one is caught.
    if tile
        .phases
        .iter()
        .any(|phase| phase.participation != participants)
    {
        return Err(cooperative(CooperativeTileRule::PhaseParticipation));
    }

    let staging_count = u32::try_from(tile.staging.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;
    if tile.staging.is_empty()
        || tile
            .staging
            .iter()
            .enumerate()
            .any(|(position, staging)| u32::try_from(position) != Ok(staging.id.get()))
    {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    tile.local_memory_bytes()
        .ok_or_else(|| cooperative(CooperativeTileRule::StructuralLimit))?;
    // A lifetime that ends before it begins, or that names a phase the tile does
    // not have, cannot bound anything.
    if tile.staging.iter().any(|staging| {
        staging.live_from > staging.live_through || staging.live_through.get() >= phase_count
    }) {
        return Err(cooperative(CooperativeTileRule::StagingLifetime));
    }

    // One writer per slot *within one round*, and every in-range slot written on
    // every round. The two are checked together over one occupancy map because
    // they are the two halves of the same statement: the participants' writes
    // are a bijection onto the allocation's slots.
    //
    // The map spans the phase sequence once, which is exactly one round — every
    // phase runs on every round — so this needs no round dimension and gains
    // none. A tile with several rounds rewrites these slots on the next one, and
    // that is the capability rather than the defect; what the map still refuses
    // is two writers reaching one slot inside a single round, where no point
    // could separate them.
    let mut written: Vec<Vec<bool>> = tile
        .staging
        .iter()
        .map(|staging| {
            vec![
                false;
                usize::try_from(staging.slots)
                    .unwrap_or(usize::MAX)
                    .min(usize::try_from(MAX_COOPERATIVE_STAGING_SLOTS).unwrap_or(usize::MAX))
            ]
        })
        .collect();
    for phase in &tile.phases {
        for write in &phase.writes {
            let staging = resolve_staging(tile, write.staging, staging_count)?;
            if phase.id < staging.live_from || phase.id > staging.live_through {
                return Err(cooperative(CooperativeTileRule::StagingLifetime));
            }
            let occupancy = written
                .get_mut(usize::try_from(write.staging.get()).unwrap_or(usize::MAX))
                .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
            for slot in addressed_slots(space, write.span, staging.slots)? {
                let slot = occupancy
                    .get_mut(usize::try_from(slot).unwrap_or(usize::MAX))
                    .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
                // Two writers to one slot inside one round: nothing orders them,
                // because a point sits between phases and both writes would be
                // on the same side of every one that could.
                if std::mem::replace(slot, true) {
                    return Err(cooperative(CooperativeTileRule::StagingConflict));
                }
            }
        }
    }
    if written
        .iter()
        .any(|occupancy| occupancy.iter().any(|written| !written))
    {
        return Err(cooperative(CooperativeTileRule::StagingCoverage));
    }

    for phase in &tile.phases {
        for read in &phase.reads {
            let staging = resolve_staging(tile, read.staging, staging_count)?;
            if phase.id < staging.live_from || phase.id > staging.live_through {
                return Err(cooperative(CooperativeTileRule::StagingLifetime));
            }
            // The result is discarded because coverage has already proved every
            // in-range slot has a writer, so a read's addressed set needs no
            // second occupancy pass — but the call is *not* discardable: it is
            // what refuses a read whose stride vector disagrees with the tile's
            // rank, and a read whose addressed slot leaves the allocation. A
            // read admitted on either would emit a silently wrong broadcast,
            // which is exactly the defect class a widened relation must not
            // inherit.
            addressed_slots(space, read.span, staging.slots)?;
            // Coverage above already proved every in-range slot has a writer, so
            // what remains is the *ordering*: the writer must be in an earlier
            // phase of the same round, or the read observes values its own phase
            // is still producing and no synchronization point could ever
            // separate them. A loop-carried tile does not widen this. Admitting
            // a same- or later-phase writer would leave the read observing the
            // previous round's value, which the first round has not written at
            // all.
            if !tile.phases.iter().any(|producer| {
                producer.id < phase.id
                    && producer
                        .writes
                        .iter()
                        .any(|write| write.staging == read.staging)
            }) {
                return Err(cooperative(CooperativeTileRule::StagedProducer));
            }
        }
    }

    let edges = tile.visibility_edges();
    if edges.is_empty() {
        return Err(cooperative(CooperativeTileRule::NoVisibilityEdge));
    }
    verify_synchronization(tile, participants, &edges, &tile.anti_dependency_edges())
}

/// Verifies the synchronization authority that orders one tile's handoffs.
///
/// Runs after the dataflow rules and reads their results, which is why it takes
/// the derived edges rather than recomputing them: the edges are the obligation,
/// and this decides whether anything legally discharges each one. Every rule is
/// stated against a fact a point *declares*, so each is separately perturbable
/// — the model can express an unadmitted kind, a boundary that is not a program
/// point, a narrowed participant set, a weaker ordering, or a convergence claim
/// with nothing behind it.
///
/// Both derived evidence classes arrive here and are held to the same standard.
/// A visibility edge and an anti-dependency each need exactly one discharging
/// point, and a point earns its place by discharging at least one of either —
/// which is what lets a round boundary, whose whole job is the anti-dependency,
/// be something other than redundant.
///
/// `participants` is the linearized run the caller derived from the tile's
/// participant space, passed rather than re-derived so the two authorities
/// cannot disagree about how many invocations a space holds.
fn verify_synchronization(
    tile: &CooperativeTile,
    participants: ParticipantRange,
    edges: &[VisibilityEdge],
    anti: &[AntiDependencyEdge],
) -> Result<(), ScheduledRegionDiagnostic> {
    // A tile whose participants are one invocation stages values it reads back
    // itself: program order already orders the handoff, so a point there is the
    // semantically redundant barrier this authority exists to eliminate.
    if participants.count < 2 {
        return Err(synchronization(SynchronizationRule::SingleParticipant));
    }
    if tile.synchronization.len() > MAX_COOPERATIVE_SYNCHRONIZATION_POINTS {
        return Err(synchronization(SynchronizationRule::StructuralLimit));
    }
    if tile
        .synchronization
        .iter()
        .enumerate()
        .any(|(position, point)| u32::try_from(position) != Ok(point.id.get()))
    {
        return Err(synchronization(SynchronizationRule::PointSequence));
    }

    // The one realization every point of this tile must state. Derived from the
    // edges rather than read from any point, so the points are checked against
    // the dependency instead of against each other.
    let required = required_subject(edges)
        .ok_or_else(|| synchronization(SynchronizationRule::UndischargedVisibility))?;
    let phase_count = u32::try_from(tile.phases.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;
    // The last phase of a round, which is the far side of a round boundary. The
    // caller already proved the ordinals are the dense run `0..phase_count` and
    // that the sequence is nonempty, so the subtraction is exact.
    let last_phase = PhaseId::new(
        phase_count
            .checked_sub(1)
            .ok_or_else(|| cooperative(CooperativeTileRule::PhaseSequence))?,
    );

    for point in &tile.synchronization {
        // The kind is checked first and separately, because a point naming an
        // unadmitted construct fails for a reason none of the dimension checks
        // below would name: its contract is undefined here, so comparing its
        // scopes against a control barrier's would be comparing the wrong thing.
        if point.subject.kind != required.kind {
            return Err(synchronization(SynchronizationRule::UnadmittedKind));
        }
        // The phases this point separates, which a placement either names or
        // leaves to the tile. A phase boundary must name two consecutive
        // existing phases: a "boundary" spanning a phase is not a program point,
        // because that phase's own effects would fall on an undetermined side of
        // the fence. A round boundary names none — the phases it separates are
        // the sequence's last and first, which the tile already fixed — so there
        // is nothing here for it to get wrong and the check has nothing to say
        // about it.
        let bounded = match (point.placement.preceding(), point.placement.following()) {
            (Some(preceding), Some(following)) => {
                if following.get() >= phase_count
                    || preceding.get().checked_add(1) != Some(following.get())
                {
                    return Err(synchronization(SynchronizationRule::Placement));
                }
                vec![preceding, following]
            }
            (None, None) => vec![last_phase, PhaseId::FIRST],
            // Unreachable while every placement names both ordinals or neither,
            // and refused rather than assumed so a half-named placement added
            // later cannot silently pick one of the two arms above.
            _ => return Err(synchronization(SynchronizationRule::Placement)),
        };
        if point.participants != participants {
            return Err(synchronization(SynchronizationRule::ParticipantSet));
        }
        if point.subject.execution_scope != required.execution_scope {
            return Err(synchronization(SynchronizationRule::ExecutionScope));
        }
        if point.subject.visibility_scope != required.visibility_scope {
            return Err(synchronization(SynchronizationRule::VisibilityScope));
        }
        if point.subject.fenced_spaces != required.fenced_spaces {
            return Err(synchronization(SynchronizationRule::FencedSpaces));
        }
        if point.subject.ordering != required.ordering {
            return Err(synchronization(SynchronizationRule::Ordering));
        }
        // The evidence class, then the derivation it names. A caller's assertion
        // is refused whatever the tile looks like, and so is the single-round
        // derivation on a tile whose phases repeat — reaching a point is not the
        // same as reaching the same *instance* of it once there are several. The
        // derived class is only as good as the re-derivation below, which is why
        // both run.
        if point.convergence != ConvergenceEvidence::required_for_rounds(tile.rounds) {
            return Err(synchronization(SynchronizationRule::ConvergenceEvidence));
        }
        if !phases_are_reached_by(tile, &bounded, participants) {
            return Err(synchronization(SynchronizationRule::Convergence));
        }
        // A point that orders nothing consumes a target authority for an
        // operation the program has no reason to perform. Both classes count: a
        // round boundary discharges no visibility edge by construction, and
        // refusing it for that would make the loop-carried tile unstatable.
        if !edges.iter().any(|edge| point.discharges(*edge))
            && !anti.iter().any(|edge| point.discharges_anti(*edge))
        {
            return Err(synchronization(SynchronizationRule::RedundantPoint));
        }
    }

    // Exactly one point per edge, in each class. Zero leaves a race — a read of
    // values never published, or a rewrite over values still being read; two are
    // two schedules spelling one realization, which would give one program two
    // identities.
    for edge in edges {
        match tile.discharging_points(*edge).len() {
            1 => {}
            0 => return Err(synchronization(SynchronizationRule::UndischargedVisibility)),
            _ => return Err(synchronization(SynchronizationRule::RedundantPoint)),
        }
    }
    for edge in anti {
        match tile.anti_discharging_points(*edge).len() {
            1 => {}
            0 => {
                return Err(synchronization(
                    SynchronizationRule::UndischargedAntiDependency,
                ));
            }
            _ => return Err(synchronization(SynchronizationRule::RedundantPoint)),
        }
    }
    Ok(())
}

/// Returns whether every named phase is reached by exactly `participants`.
///
/// The convergence derivation, written over the tile's own per-phase
/// participation rather than assumed from the tile-wide rule that currently
/// implies it. Keeping it explicit is what lets a phase-participation change
/// break this check rather than silently leaving a point convergent by
/// inheritance.
pub(super) fn phases_are_reached_by(
    tile: &CooperativeTile,
    phases: &[PhaseId],
    participants: ParticipantRange,
) -> bool {
    phases.iter().all(|id| {
        tile.phases
            .iter()
            .find(|phase| phase.id == *id)
            .is_some_and(|phase| phase.participation == participants)
    })
}

/// Resolves one staged access's allocation, refusing an ordinal the tile lacks.
fn resolve_staging(
    tile: &CooperativeTile,
    id: StagingId,
    staging_count: u32,
) -> Result<WorkgroupStaging, ScheduledRegionDiagnostic> {
    if id.get() >= staging_count {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    tile.staging
        .get(usize::try_from(id.get()).unwrap_or(usize::MAX))
        .copied()
        .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))
}

/// Returns every slot the participants address through one span.
///
/// An address that overflows `u64` and one that merely exceeds the allocation
/// are the same refusal, because both mean the span leaves the storage the tile
/// declared. A stride vector whose rank disagrees with the participant space's
/// is a *different* refusal and is decided first: the span and the space are
/// each well formed alone and disagree only with one another, so folding it into
/// the capacity refusal would report a storage fault for a shape disagreement.
fn addressed_slots(
    participants: ParticipantSpace,
    span: StagedSpan,
    slots: u64,
) -> Result<Vec<u64>, ScheduledRegionDiagnostic> {
    if span.rank() != participants.rank() {
        return Err(cooperative(CooperativeTileRule::SpanRank));
    }
    if span.count == 0 {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    let addressed = CooperativeTile::addressed_slots(participants, span)
        .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
    if addressed.iter().any(|slot| *slot >= slots) {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    Ok(addressed)
}
