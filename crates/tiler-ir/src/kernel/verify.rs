//! Whole-kernel structural and schedule-refinement verification.
//!
//! Verification has the two jobs ADR 0048 accepts. It proves ordinary program
//! well-formedness that insertion-time checks cannot decide locally — signature
//! agreement, address spaces, effect ordering, output coverage, barrier
//! obligations, and loop structure — and it proves that the body is a
//! refinement of the exact scheduled region the builder was opened against.
//!
//! The specific rules run before the whole-body refinement check so a rejected
//! kernel names the exact violated obligation instead of a generic mismatch.
//! The final refinement gate re-derives the canonical structured body from the
//! verified scheduled region and requires structural equality. That is a
//! deliberate bounded profile: a semantically equivalent but differently
//! spelled body is rejected as [`KernelDiagnostic::BodyRefinement`] rather than
//! admitted by an unproven equivalence argument.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::schedule::{
    Access, AccessMode, AntiDependencyEdge, BoundsProofKind, BoundsWitnessId,
    CanonicalScheduledRegionIdentity, ContributorCoverage, ContributorPartition, CooperativeTile,
    ExecutionBinding, FencedSpaces, MemoryOrdering, OwnershipWitnessId, PhaseId, ReductionPass,
    ReductionTopology, ResourceRequirements, ScheduledRegion, StagedElement, StagingId,
    SyncPointId, SynchronizationKind, SynchronizationPlacement, SynchronizationPoint,
    SynchronizationScope, SynchronizationSubject, TailPolicy, VisibilityEdge, contributor_count,
    cooperative_tile, element_count, live_input_extents,
};

use super::error::KernelDiagnostic;
use super::model::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BufferAccess, Builtin, CompareOp,
    ExecutionScope, InputExtentParameter, KernelConstant, KernelData, KernelType, MemoryScope,
    OperationKind, region_element_type,
};

/// Returns the number of addressable elements one scheduled access spans.
pub(super) fn access_elements(
    access: &Access,
    schedule: &ScheduledRegion,
) -> Result<u64, KernelDiagnostic> {
    let proof = schedule
        .index
        .bounds_proofs
        .iter()
        .find(|proof| proof.id == access.bounds)
        .ok_or(KernelDiagnostic::BoundsEvidence)?;
    match &proof.kind {
        BoundsProofKind::LinearRange { element_count } => {
            if matches!(
                access.map,
                crate::schedule::LogicalAccess::LiveRowMajor { .. }
            ) || (matches!(
                access.map,
                crate::schedule::LogicalAccess::ContractionOperand { .. }
            ) && matches!(
                schedule.schedule.reduction,
                ReductionTopology::LiveContraction { .. }
            )) {
                Ok(0)
            } else {
                Ok(*element_count)
            }
        }
        BoundsProofKind::ReductionDomain { input_shape, .. } => {
            element_count(input_shape).map_err(|_| KernelDiagnostic::ElementCountOverflow)
        }
    }
}

/// Returns the ordered read and write accesses of a bounded scheduled region.
pub(super) fn boundary_accesses(
    schedule: &ScheduledRegion,
) -> Result<(&[Access], &Access), KernelDiagnostic> {
    let Some((write, reads)) = schedule.index.accesses.split_last() else {
        return Err(KernelDiagnostic::ScheduleAccessCount);
    };
    if reads.is_empty()
        || reads.iter().any(|read| read.mode != AccessMode::Read)
        || write.mode != AccessMode::Write
    {
        return Err(KernelDiagnostic::ScheduleAccessCount);
    }
    Ok((reads, write))
}

/// One memory effect recorded in program order.
#[derive(Clone, Copy, Debug)]
struct Effect {
    kind: EffectKind,
    loop_depth: u32,
    guarded: bool,
    row: bool,
    column: bool,
}

#[derive(Clone, Copy, Debug)]
enum EffectKind {
    Load {
        bounds: BoundsWitnessId,
        ordinary: bool,
    },
    Store {
        bounds: BoundsWitnessId,
        ownership: OwnershipWitnessId,
    },
}

/// The shape of one structured loop, summarized for the reduction contract.
#[derive(Clone, Debug)]
struct LoopSummary {
    start: u64,
    end: u64,
    start_value: Option<u32>,
    end_value: Option<u32>,
    accumulators: Vec<KernelType>,
    block_depth: u32,
    /// Synchronization-relevant events emitted inside this loop's body.
    ///
    /// A half-open range into [`Walk::sync`], which is filled in program order,
    /// so a loop's own events are contiguous. This is what lets the
    /// synchronization rules speak about "the round body" as a region — the
    /// events of one iteration — rather than about a nesting depth, which cannot
    /// tell the round loop apart from a staged fold that happens to sit at the
    /// same depth.
    sync: std::ops::Range<usize>,
}

/// One synchronization-relevant event, recorded in program order.
///
/// Staged accesses and barriers share one ordered list because the property they
/// have to prove is *relative*: a staged read is legal exactly when the barrier
/// realizing the point that discharges its edge sits between it and the write it
/// consumes. Two separate lists could each be well formed and still leave the
/// read ahead of the fence.
#[derive(Clone, Copy, Debug)]
enum SyncEvent {
    /// A store into workgroup staging, tagged with the phase authorizing it.
    StagedWrite { staging: u32, phase: PhaseId },
    /// A load from workgroup staging, tagged with the phase authorizing it.
    StagedRead { staging: u32, phase: PhaseId },
    /// A barrier realizing one schedule synchronization point.
    Barrier {
        point: SyncPointId,
        /// Lexical block depth, `0` at the kernel's top level.
        block_depth: u32,
        /// Enclosing loop nesting, `0` outside every loop.
        ///
        /// Carried beside the block depth rather than folded into it because the
        /// two answer different questions: the block depth counts every
        /// enclosing region and the loop depth counts only the ones every
        /// invocation enters. Their difference is the number of predicates on
        /// the path, which is what decides convergence.
        loop_depth: u32,
    },
}

/// Returns whether a barrier at this nesting is reached by every participant.
///
/// Two facts, and both are needed.
///
/// **No predicate may enclose it.** A barrier inside a predicated region is
/// reached by whichever invocations the predicate admits, and one not reached by
/// every participant is undefined execution on every target. `block_depth`
/// counts every enclosing region and `loop_depth` only the loops, so their
/// equality is exactly "nothing on the path is a predicate" — the two counts
/// were always tracked separately and this is the rule that reads the
/// difference.
///
/// **The loop nesting must be within what the tile's round structure
/// authorizes.** A [`SerialLoopSpec`](super::model::SerialLoopSpec) carries
/// `start` and `end` as `u64` *literals*, not values, so every invocation of a
/// workgroup runs an identical trip count and a barrier in that body is reached
/// by all of them at the same dynamic instance — which is why a loop, unlike a
/// predicate, can enclose one at all. What a loop level still needs is a
/// *reason*: the tile's round loop is the only repetition its schedule declares,
/// so a tile with several rounds authorizes at most one enclosing loop and a
/// single-round tile authorizes none. A barrier inside a contributor fold would
/// otherwise be admitted, and it would synchronize once per contributor for a
/// point the schedule places once between two phases.
///
/// **At most one, not exactly one, and the difference is the peel.** A fold seeds
/// at its first contributor, so a loop-carried body emits round zero ahead of the
/// round loop and realizes the phase boundary there at the top level. Requiring
/// the enclosing loop would refuse that peeled realization; what makes the
/// top-level one earn its place instead is the realization count, which
/// [`verify_synchronization`] checks against the placement — `rounds` for a phase
/// boundary and `rounds - 1` for a round boundary.
///
/// This proves convergence, not that the enclosing loop is the *right* loop.
/// Matching its trip count against the declared round count is
/// [`round_body`]'s obligation, and it is discharged there because it is a fact
/// about a loop rather than about a depth.
pub(super) const fn barrier_is_convergent(block_depth: u32, loop_depth: u32, rounds: u64) -> bool {
    block_depth == loop_depth && loop_depth <= authorized_barrier_loop_depth(rounds)
}

/// Returns the deepest loop nesting a tile of `rounds` rounds authorizes.
const fn authorized_barrier_loop_depth(rounds: u64) -> u32 {
    if rounds > 1 { 1 } else { 0 }
}

#[derive(Debug, Default)]
struct Walk {
    effects: Vec<Effect>,
    has_synchronization: bool,
    /// Barrier specifications in program order, parallel to the `Barrier`
    /// entries of `sync`.
    barriers: Vec<BarrierSpec>,
    /// Staged accesses and barriers in program order.
    sync: Vec<SyncEvent>,
    loops: Vec<LoopSummary>,
    ungoverned_predicate: bool,
    incomplete_staging: bool,
    guarded_loads: Vec<GuardedLoadFact>,
}

#[derive(Clone, Copy, Debug)]
struct GuardedLoadFact {
    predicate: u32,
    buffer: u32,
    inactive: u32,
}

#[derive(Clone, Debug, Default)]
struct AxisGuards {
    row: BTreeSet<u32>,
    column: BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug)]
struct Nest(u8);

impl Nest {
    const GUARDED: u8 = 1;
    const ROW: u8 = 2;
    const COLUMN: u8 = 4;
    const PREDICATED: u8 = 8;

    const fn empty() -> Self {
        Self(0)
    }

    const fn with(self, flag: u8) -> Self {
        Self(self.0 | flag)
    }

    const fn has(self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

struct Visit<'a> {
    data: &'a KernelData,
    guards: &'a BTreeSet<u32>,
    axis: &'a AxisGuards,
    walk: &'a mut Walk,
}

/// Verifies one assembled kernel against the scheduled region it refines.
pub(super) fn verify_kernel(
    data: &KernelData,
    schedule: &ScheduledRegion,
    schedule_identity: &CanonicalScheduledRegionIdentity,
    derived: ResourceRequirements,
) -> Result<(), KernelDiagnostic> {
    let (reads, write) = boundary_accesses(schedule)?;
    verify_signature(data, schedule, reads, write, derived)?;
    verify_input_extents(data, schedule, reads)?;
    verify_cooperative(data, schedule)?;

    let axis = axis_guards(data, schedule);
    let mut guards = guard_values(data, schedule);
    guards.extend(axis.row.iter().copied());
    guards.extend(axis.column.iter().copied());
    let mut walk = Walk::default();
    let mut nest = Nest::empty();
    if matches!(schedule.schedule.tail, TailPolicy::Predicated) {
        nest = nest.with(Nest::PREDICATED);
    }
    let mut visit = Visit {
        data,
        guards: &guards,
        axis: &axis,
        walk: &mut walk,
    };
    visit_block(&mut visit, 0, nest, 0, 0);
    if matches!(schedule.schedule.tail, TailPolicy::Predicated)
        && matches!(
            schedule.schedule.reduction,
            ReductionTopology::CooperativeContraction { .. }
        )
        && walk
            .effects
            .iter()
            .any(|effect| matches!(effect.kind, EffectKind::Load { ordinary: true, .. }))
    {
        return Err(KernelDiagnostic::BoundsEvidence);
    }
    if walk.ungoverned_predicate {
        return Err(KernelDiagnostic::PredicateDominance);
    }
    if walk.incomplete_staging {
        return Err(KernelDiagnostic::IncompleteStaging);
    }
    verify_synchronization(&walk, data, schedule)?;
    verify_effects(&walk, schedule, reads, write)?;
    verify_predicated_contraction_roles(&walk, data, schedule, reads)?;
    verify_reduction(&walk, data, schedule, reads)?;

    let canonical = super::lower::derive_canonical(schedule, schedule_identity, derived)?;
    if data != &canonical {
        return Err(KernelDiagnostic::BodyRefinement);
    }
    Ok(())
}

fn verify_input_extents(
    data: &KernelData,
    schedule: &ScheduledRegion,
    reads: &[Access],
) -> Result<(), KernelDiagnostic> {
    let expected: Vec<InputExtentParameter> = live_input_extents(schedule)
        .into_iter()
        .map(|(tensor, axis)| InputExtentParameter { tensor, axis })
        .collect();
    if data.input_extents != expected {
        return Err(KernelDiagnostic::InputExtentContract);
    }
    let mut seen = BTreeSet::new();
    for parameter in &data.input_extents {
        let crate::schedule::TensorRole::Input { ordinal } = parameter.tensor else {
            return Err(KernelDiagnostic::InputExtentContract);
        };
        if !seen.insert((ordinal.get(), parameter.axis.get())) {
            return Err(KernelDiagnostic::InputExtentContract);
        }
        let rank = reads
            .iter()
            .find(|read| read.tensor == parameter.tensor)
            .map_or_else(
                || access_rank_from_iteration(schedule),
                |read| access_rank(read, schedule),
            );
        if u64::from(parameter.axis.get()) >= rank {
            return Err(KernelDiagnostic::InputExtentContract);
        }
    }
    if !data.input_extents.is_empty() {
        let mut used = vec![false; data.input_extents.len()];
        for block in &data.blocks {
            for operation in &block.operations {
                if let OperationKind::InputExtent { parameter } = operation.kind
                    && let Some(slot) = used.get_mut(parameter as usize)
                {
                    *slot = true;
                }
            }
        }
        if used.iter().any(|used| !used) {
            return Err(KernelDiagnostic::UnusedInputExtent);
        }
    }
    Ok(())
}

fn access_rank(access: &Access, schedule: &ScheduledRegion) -> u64 {
    match &access.map {
        crate::schedule::LogicalAccess::LinearIdentity
        | crate::schedule::LogicalAccess::ScalarBroadcast
        | crate::schedule::LogicalAccess::PackedU4LsbZeroTail { .. } => 1,
        crate::schedule::LogicalAccess::ReductionContributor { input_shape, .. } => {
            input_shape.rank() as u64
        }
        crate::schedule::LogicalAccess::ContractionOperand { operand_shape, .. } => {
            let static_rank = operand_shape.rank() as u64;
            if matches!(
                schedule.schedule.reduction,
                ReductionTopology::LiveContraction { .. }
            ) {
                static_rank.saturating_add(1)
            } else {
                static_rank
            }
        }
        crate::schedule::LogicalAccess::ReindexBijection { operand_shape, .. }
        | crate::schedule::LogicalAccess::BroadcastReplication { operand_shape, .. } => {
            operand_shape.rank() as u64
        }
        crate::schedule::LogicalAccess::ParametricBroadcast { operand_shape, .. } => {
            operand_shape.rank() as u64
        }
        crate::schedule::LogicalAccess::LiveRowMajor { inner_axis } => {
            u64::from(inner_axis.get()).saturating_add(1)
        }
    }
}

fn access_rank_from_iteration(schedule: &ScheduledRegion) -> u64 {
    schedule.index.iteration_shape.rank() as u64
}

fn verify_signature(
    data: &KernelData,
    schedule: &ScheduledRegion,
    reads: &[Access],
    write: &Access,
    derived: ResourceRequirements,
) -> Result<(), KernelDiagnostic> {
    if data.buffers.len() != reads.len().saturating_add(1) {
        return Err(KernelDiagnostic::BufferContract);
    }
    let (write_buffer, read_buffers) = data
        .buffers
        .split_last()
        .ok_or(KernelDiagnostic::BufferContract)?;
    // The strict-affine signature is fixed by its three named components; the
    // reduction families read one contributor domain. A pointwise region reads
    // one dense tensor per expression leaf, so its expected signature is as wide
    // as its own access list rather than a constant — and the schedule verifier
    // already proved that width equals the expression's input count.
    //
    // The element type comes from `region_element_type`, the same authority the
    // canonical lowering declares its buffers from, so a widened dtype cannot be
    // declared as one type and checked against another. The *arities* stay
    // written out per family, because an arity read back from the region would
    // agree with whatever the region declared.
    let element_type = region_element_type(&schedule.index.scalar_program);
    let expected_types: Vec<KernelType> = match schedule.index.scalar_program {
        crate::schedule::ScalarProgram::StrictAffineU4Dequantize { .. } => {
            vec![KernelType::U8, KernelType::F32, KernelType::U8]
        }
        crate::schedule::ScalarProgram::PointwiseF32(_)
        | crate::schedule::ScalarProgram::PointwiseBf16(_) => vec![element_type; reads.len()],
        crate::schedule::ScalarProgram::StrictSerialSum { .. }
        | crate::schedule::ScalarProgram::SquaredSerialSum { .. }
        // One read, like every other fold: the epilogue's leaf is the folded
        // value rather than a boundary tensor, so it adds no buffer.
        | crate::schedule::ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | crate::schedule::ScalarProgram::StrictSerialMaximum { .. }
        | crate::schedule::ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            vec![element_type]
        }
        // Two, stated as two rather than as `reads.len()`: the count is the
        // contraction family's own arity, so a region declaring some other
        // number of reads must fail this comparison instead of agreeing with
        // itself.
        crate::schedule::ScalarProgram::StrictTensorContraction { .. } => {
            vec![element_type, element_type]
        }
    };
    let expected_elements = reads
        .iter()
        .map(|read| access_elements(read, schedule))
        .collect::<Result<Vec<_>, _>>()?;
    if read_buffers.len() != expected_types.len()
        || read_buffers
            .iter()
            .zip(reads)
            .zip(expected_types.iter().zip(expected_elements))
            .any(|((buffer, read), (expected_type, expected_elements))| {
                buffer.tensor != read.tensor
                    || buffer.component_role != read.component_role
                    || buffer.access != BufferAccess::Read
                    || buffer.element_type != *expected_type
                    || buffer.element_count != expected_elements
            })
        || write_buffer.tensor != write.tensor
        || write_buffer.component_role != write.component_role
        || write_buffer.access != BufferAccess::Write
        || write_buffer.element_type != element_type
        || write_buffer.element_count != access_elements(write, schedule)?
    {
        return Err(KernelDiagnostic::BufferContract);
    }
    for buffer in &data.buffers {
        let admitted = match buffer.address_space {
            AddressSpace::Device => derived.requires_device_memory,
            // A workgroup allocation is never a buffer *parameter*, whatever the
            // region's local-memory requirement is. A parameter's position is
            // its argument-table ordinal, and workgroup storage is declared
            // inside the entry point rather than bound as an argument — so
            // admitting one here would re-base every later ordinal and change
            // what an existing signature position means. It is declared through
            // [`super::KernelBuilder::declare_staging`] instead, and
            // `verify_cooperative` proves that list against the region's tile.
            AddressSpace::Workgroup | AddressSpace::InvocationPrivate | AddressSpace::Constant => {
                false
            }
        };
        if !admitted {
            return Err(KernelDiagnostic::AddressSpaceContract);
        }
    }
    // The binding fixes the global coordinate; a cooperative tile additionally
    // needs the local one, because its participants are named by their position
    // within the workgroup. The second builtin is required rather than merely
    // permitted, so a tile whose kernel cannot name its participants is refused.
    let mut expected_builtins = match &schedule.schedule.binding {
        ExecutionBinding::GlobalLinearInvocation | ExecutionBinding::BlockedWorkgroup { .. } => {
            vec![Builtin::GlobalInvocationIndex]
        }
    };
    if cooperative_tile(&schedule.schedule.reduction).is_some() {
        expected_builtins.push(Builtin::LocalInvocationIndex);
    }
    if data.admitted_builtins != expected_builtins {
        return Err(KernelDiagnostic::BuiltinContract);
    }
    if data.numerical != schedule.index.numerical {
        return Err(KernelDiagnostic::NumericalRealization);
    }
    if data.requirements != derived {
        return Err(KernelDiagnostic::ResourceRequirements);
    }
    Ok(())
}

/// Verifies the workgroup staging a kernel declares against its region's tile.
///
/// The staging declarations must realize the tile exactly — same count, same
/// ordinals in order, same element type, same slot count — so a producer cannot
/// allocate more or differently shaped workgroup storage than the schedule
/// proved well formed. Whether anything *orders* the staged handoff is
/// [`verify_synchronization`]'s question, and it runs after the body walk
/// because the answer is a property of the body's operation order.
fn verify_cooperative(
    data: &KernelData,
    schedule: &ScheduledRegion,
) -> Result<(), KernelDiagnostic> {
    let Some(tile) = cooperative_tile(&schedule.schedule.reduction) else {
        // A region that stages nothing must declare nothing, or the kernel
        // claims workgroup storage its schedule never proved.
        if data.staging.is_empty() {
            return Ok(());
        }
        return Err(KernelDiagnostic::StagingContract);
    };
    if data.staging.len() != tile.staging.len() {
        return Err(KernelDiagnostic::StagingContract);
    }
    for (declared, staged) in data.staging.iter().zip(&tile.staging) {
        let element_type = match staged.element {
            StagedElement::F32 => KernelType::F32,
        };
        if declared.staging != staged.id
            || declared.element_type != element_type
            || declared.address_space != AddressSpace::Workgroup
            || declared.element_count != staged.slots
        {
            return Err(KernelDiagnostic::StagingContract);
        }
    }
    Ok(())
}

/// Projects one barrier's declared spelling onto a schedule subject.
///
/// A *total* mapping, written as exhaustive matches over both vocabularies, so
/// widening either is a build error here rather than a silent disagreement
/// between a schedule obligation and the barrier a backend will emit. The two
/// vocabularies stay separate on purpose: the schedule's is the obligation and
/// the kernel's is the emission spelling, and equal field shapes are not what
/// makes them agree — this projection plus the equality below is.
///
/// Returns `None` when the spelling names something the schedule vocabulary
/// cannot express at all, which is a refusal rather than a nearest match.
fn barrier_subject(spec: &BarrierSpec) -> Option<SynchronizationSubject> {
    // Exhaustive on purpose. `#[non_exhaustive]` has no effect inside the
    // defining crate, so widening either vocabulary is a build error here — the
    // one place that has to decide what the new spelling means — rather than a
    // wildcard silently projecting it onto whichever scope it resembles.
    let execution_scope = match spec.execution_scope {
        ExecutionScope::Subgroup => SynchronizationScope::Subgroup,
        ExecutionScope::Workgroup => SynchronizationScope::Workgroup,
    };
    let visibility_scope = match spec.memory_scope {
        MemoryScope::Workgroup => SynchronizationScope::Workgroup,
        MemoryScope::Device => SynchronizationScope::Device,
    };
    let ordering = match spec.ordering {
        BarrierOrdering::AcquireRelease => MemoryOrdering::AcquireRelease,
    };
    // A repeated or unorderable fenced space is a spelling with two readings, so
    // it is refused rather than deduplicated into one.
    let mut fenced = FencedSpaces::NONE;
    for space in &spec.fenced_spaces {
        let flag = match space {
            AddressSpace::Workgroup => &mut fenced.workgroup,
            AddressSpace::Device => &mut fenced.device,
            AddressSpace::InvocationPrivate | AddressSpace::Constant => return None,
        };
        if std::mem::replace(flag, true) {
            return None;
        }
    }
    Some(SynchronizationSubject {
        // A `Barrier` operation *is* the control-barrier construct; a different
        // kind is a different operation, which is why no field spells it.
        kind: SynchronizationKind::ControlBarrier,
        execution_scope,
        visibility_scope,
        fenced_spaces: fenced,
        ordering,
    })
}

/// Proves the body realizes exactly the schedule's synchronization authority.
///
/// Seven obligations, in the order a failure is most usefully reported.
///
/// 1. A region whose schedule owns no synchronization point contains no barrier
///    and no staged access. This is where the pointwise, global-linear barrier
///    is eliminated: that schedule has no cooperative tile, so it can state no
///    point, so any barrier in its body is unauthorized by construction.
/// 2. Every barrier names a declared point.
/// 3. The loop nesting the barriers sit in is the tile's round structure: none
///    for a single-round tile, and exactly one top-level `1..rounds` loop for a
///    loop-carried one. See [`round_body`].
/// 4. Every barrier sits outside every predicate and no deeper than that loop,
///    which is what makes it convergent. See [`barrier_is_convergent`].
/// 5. Every barrier's declared spelling projects onto its point's subject.
/// 6. Every declared point is realized as many times as its placement requires —
///    once per round for a phase boundary, once per round *transition* for a
///    round boundary. See [`verify_realizations`], which is where a peeled round
///    zero earns its top-level barrier.
/// 7. Every staged access is authorized by the phase it names; every visibility
///    edge's write precedes, and its read follows, the barrier realizing the
///    point that discharges it, *within each region the body emits*; and every
///    anti-dependency's rewrite is separated from the reads it destroys, both
///    around the round body's back edge and across the peel.
fn verify_synchronization(
    walk: &Walk,
    data: &KernelData,
    schedule: &ScheduledRegion,
) -> Result<(), KernelDiagnostic> {
    let Some(tile) = cooperative_tile(&schedule.schedule.reduction) else {
        if walk.has_synchronization {
            return Err(KernelDiagnostic::UnexpectedSynchronization);
        }
        if walk.sync.is_empty() {
            return Ok(());
        }
        return Err(KernelDiagnostic::StagedAccessEvidence);
    };

    // Obligation 2.
    for spec in &walk.barriers {
        resolve_point(tile, spec.point)?;
    }

    // Obligation 3.
    let round = round_body(walk, tile.rounds)?;

    // Obligations 4 and 5.
    for (spec, event) in walk.barriers.iter().zip(
        walk.sync
            .iter()
            .filter(|event| matches!(event, SyncEvent::Barrier { .. })),
    ) {
        if let SyncEvent::Barrier {
            block_depth,
            loop_depth,
            ..
        } = event
            && !barrier_is_convergent(*block_depth, *loop_depth, tile.rounds)
        {
            return Err(KernelDiagnostic::SynchronizationConvergence);
        }
        let point = resolve_point(tile, spec.point)?;
        if barrier_subject(spec) != Some(point.subject) {
            return Err(KernelDiagnostic::SynchronizationContract);
        }
    }

    let edges = tile.visibility_edges();

    // Obligation 6.
    verify_realizations(walk, tile, &edges, round.as_ref())?;

    // Obligation 7a: every staged access names a phase that authorizes it.
    for event in &walk.sync {
        let (staging, phase, writes) = match event {
            SyncEvent::StagedWrite { staging, phase } => (*staging, *phase, true),
            SyncEvent::StagedRead { staging, phase } => (*staging, *phase, false),
            SyncEvent::Barrier { .. } => continue,
        };
        let id = data
            .staging
            .get(usize::try_from(staging).unwrap_or(usize::MAX))
            .ok_or(KernelDiagnostic::StagedAccessEvidence)?
            .staging;
        let authorized = tile
            .phases
            .iter()
            .find(|candidate| candidate.id == phase)
            .is_some_and(|candidate| {
                if writes {
                    candidate.writes.iter().any(|write| write.staging == id)
                } else {
                    candidate.reads.iter().any(|read| read.staging == id)
                }
            });
        if !authorized {
            return Err(KernelDiagnostic::StagedAccessEvidence);
        }
    }

    // Obligation 7b and 7c: the fences actually separate each handoff and each
    // rewrite.
    let regions = regions(walk, round.as_ref());
    for edge in &edges {
        verify_edge_is_ordered(walk, data, tile, *edge, &regions)?;
    }
    for edge in tile.anti_dependency_edges() {
        verify_anti_edge_is_ordered(walk, data, tile, edge, round.as_ref())?;
    }
    Ok(())
}

/// Resolves one barrier's point reference against the region's tile.
fn resolve_point(
    tile: &CooperativeTile,
    point: SyncPointId,
) -> Result<&SynchronizationPoint, KernelDiagnostic> {
    tile.synchronization
        .iter()
        .find(|candidate| candidate.id == point)
        .ok_or(KernelDiagnostic::UnexpectedSynchronization)
}

/// Returns the flat positions of every barrier, paired with the point it names.
fn realizations(walk: &Walk) -> impl Iterator<Item = (usize, SyncPointId)> + '_ {
    walk.sync
        .iter()
        .enumerate()
        .filter_map(|(position, event)| match event {
            SyncEvent::Barrier { point, .. } => Some((position, *point)),
            SyncEvent::StagedWrite { .. } | SyncEvent::StagedRead { .. } => None,
        })
}

/// Returns the events of the tile's round body, proving the loop is that loop.
///
/// A barrier may be enclosed by at most one loop, and on a tile whose phases
/// repeat it *must* be: the round loop is the only repetition the schedule
/// declares. Which loop that is cannot be read off a nesting depth — a staged
/// fold at the kernel's top level has the same depth — so it is identified by
/// containing a barrier, and then held to the round loop's own shape: at the
/// kernel's top level, and running `1..rounds`, which is the range a body peeling
/// round zero emits. This is the trip-count obligation
/// [`barrier_is_convergent`] states and deliberately does not decide.
///
/// `None` is a single-round tile, whose barriers are all at the top level. A
/// multi-round tile with no such loop is refused rather than admitted as a fully
/// unrolled body: the schedule declares a repetition, and a body that spelled it
/// as `rounds` copies would leave every region below with two candidate fences.
fn round_body(walk: &Walk, rounds: u64) -> Result<Option<Range<usize>>, KernelDiagnostic> {
    let barriers: Vec<usize> = realizations(walk).map(|(position, _)| position).collect();
    let enclosing: Vec<&LoopSummary> = walk
        .loops
        .iter()
        .filter(|summary| {
            barriers
                .iter()
                .any(|position| summary.sync.contains(position))
        })
        .collect();
    let [round] = enclosing.as_slice() else {
        return if enclosing.is_empty() && rounds <= 1 {
            Ok(None)
        } else {
            Err(KernelDiagnostic::SynchronizationConvergence)
        };
    };
    if rounds <= 1 || round.block_depth != 0 || round.start != 1 || round.end != rounds {
        return Err(KernelDiagnostic::SynchronizationConvergence);
    }
    Ok(Some(round.sync.clone()))
}

/// Proves each declared point is realized exactly as often as its placement asks.
///
/// A barrier at the kernel's top level runs once; one inside the round body runs
/// once per iteration, and the round loop's `1..rounds` range makes that
/// `rounds - 1`. What the placement requires is derived from what the placement
/// *means*: a phase boundary separates two phases of a round and therefore
/// happens on every one of the `rounds` rounds, while a round boundary separates
/// consecutive rounds and therefore happens at each of the `rounds - 1`
/// transitions between them.
///
/// **This is the rule that makes the peeled round checkable rather than merely
/// plausible.** A fold seeds at its first contributor, so a loop-carried body
/// emits round zero ahead of the loop: the phase boundary is realized once there
/// and `rounds - 1` times inside, summing to `rounds`, and the round boundary is
/// realized only inside. A body that dropped the peel would realize the phase
/// boundary `rounds - 1` times, and one that put the round boundary in the peel
/// would realize it `rounds` times; both are refused here by arithmetic rather
/// than by recognizing a shape.
fn verify_realizations(
    walk: &Walk,
    tile: &CooperativeTile,
    edges: &[VisibilityEdge],
    round: Option<&Range<usize>>,
) -> Result<(), KernelDiagnostic> {
    let transitions = tile.rounds.saturating_sub(1);
    let mut realized: BTreeMap<SyncPointId, u64> = BTreeMap::new();
    for (position, point) in realizations(walk) {
        let count = if round.is_some_and(|body| body.contains(&position)) {
            transitions
        } else {
            1
        };
        let total = realized.entry(point).or_insert(0);
        *total = total.saturating_add(count);
    }
    for point in &tile.synchronization {
        let required = match point.placement {
            SynchronizationPlacement::PhaseBoundary { .. } => tile.rounds,
            SynchronizationPlacement::RoundBoundary => transitions,
        };
        let count = realized.get(&point.id).copied().unwrap_or(0);
        if count == required {
            continue;
        }
        if count == 0 {
            // A point discharges visibility edges or anti-dependencies but never
            // both — the two conditions contradict — so an unrealized point falls
            // into exactly one of these, and each names the race it leaves.
            return Err(if edges.iter().any(|edge| point.discharges(*edge)) {
                KernelDiagnostic::UndischargedVisibility
            } else {
                KernelDiagnostic::UndischargedAntiDependency
            });
        }
        return Err(KernelDiagnostic::SynchronizationRealization);
    }
    Ok(())
}

/// Splits the body's synchronization events into the regions a round spans.
///
/// One region for a single-round body. Three for a loop-carried one — before the
/// round loop, inside it, and after it — because a visibility handoff is a
/// property of *one* round and the peeled round is a separate lexical copy of it.
/// Checking one global fence position across both would refuse the loop's own
/// write for following the peel's fence, which is not a defect.
///
/// The trailing region is empty in the canonical body and is kept anyway: without
/// it a staged access emitted after the loop would belong to no region and be
/// checked by nothing.
fn regions(walk: &Walk, round: Option<&Range<usize>>) -> Vec<Range<usize>> {
    let end = walk.sync.len();
    let mut ranges = Vec::with_capacity(3);
    match round {
        None => ranges.push(0..end),
        Some(body) => {
            ranges.push(0..body.start);
            ranges.push(body.clone());
            ranges.push(body.end..end);
        }
    }
    ranges.retain(|range| !range.is_empty());
    ranges
}

/// Returns the flat positions of one region's staged accesses of one allocation.
fn staged_positions(
    walk: &Walk,
    data: &KernelData,
    region: &Range<usize>,
    staging: StagingId,
    phase: PhaseId,
    writes: bool,
) -> Vec<usize> {
    walk.sync
        .iter()
        .enumerate()
        .filter(|(position, _)| region.contains(position))
        .filter_map(|(position, event)| {
            let (index, event_phase, is_write) = match event {
                SyncEvent::StagedWrite { staging, phase } => (*staging, *phase, true),
                SyncEvent::StagedRead { staging, phase } => (*staging, *phase, false),
                SyncEvent::Barrier { .. } => return None,
            };
            let declared = data
                .staging
                .get(usize::try_from(index).unwrap_or(usize::MAX))
                .map(|parameter| parameter.staging);
            (is_write == writes && event_phase == phase && declared == Some(staging))
                .then_some(position)
        })
        .collect()
}

/// Returns the one position in `region` realizing `point`, if there is exactly one.
fn sole_fence(walk: &Walk, region: &Range<usize>, point: SyncPointId) -> Option<usize> {
    let fences: Vec<usize> = realizations(walk)
        .filter(|(position, realized)| region.contains(position) && *realized == point)
        .map(|(position, _)| position)
        .collect();
    match fences.as_slice() {
        [fence] => Some(*fence),
        _ => None,
    }
}

/// Proves one visibility edge's write precedes, and its read follows, the fence.
///
/// The schedule already proved exactly one point discharges the edge; this
/// proves the *body* placed that point's barrier between the two effects. Both
/// halves are needed and neither implies the other: a body can carry the right
/// point and the right barrier and still read staged values ahead of it.
///
/// Checked once per region, because a loop-carried body performs the handoff on
/// every round and each lexical copy of the round has to order its own. A region
/// that touches the allocation at all must carry both ends and exactly one fence
/// between them; a region that touches it not at all is skipped, and at least one
/// region must have done the handoff or the edge is unordered by omission.
fn verify_edge_is_ordered(
    walk: &Walk,
    data: &KernelData,
    tile: &CooperativeTile,
    edge: VisibilityEdge,
    regions: &[Range<usize>],
) -> Result<(), KernelDiagnostic> {
    let discharging = tile.discharging_points(edge);
    let [point] = discharging.as_slice() else {
        return Err(KernelDiagnostic::UndischargedVisibility);
    };
    let point = point.id;
    let mut ordered = false;
    for region in regions {
        let writes = staged_positions(walk, data, region, edge.staging, edge.produced_in, true);
        let reads = staged_positions(walk, data, region, edge.staging, edge.consumed_in, false);
        if writes.is_empty() && reads.is_empty() {
            continue;
        }
        let fence =
            sole_fence(walk, region, point).ok_or(KernelDiagnostic::UnorderedStagedHandoff)?;
        if writes.is_empty()
            || reads.is_empty()
            || writes.iter().any(|position| *position > fence)
            || reads.iter().any(|position| *position < fence)
        {
            return Err(KernelDiagnostic::UnorderedStagedHandoff);
        }
        ordered = true;
    }
    if ordered {
        return Ok(());
    }
    Err(KernelDiagnostic::UnorderedStagedHandoff)
}

/// Proves one anti-dependency's rewrite follows the reads it destroys.
///
/// Two obligations, and a body can satisfy either without the other.
///
/// **The back edge, which is cyclic.** Inside the round body the read at position
/// `c` belongs to round `r` and the write at position `w` to round `r + 1`, so a
/// barrier at position `b` separates them exactly when `b > c` — it runs after
/// the read, at the end of round `r` — or `b < w` — it runs before the write, at
/// the start of round `r + 1`. This is the body-level mirror of
/// [`SynchronizationPoint::discharges_anti`], whose disjunction says the same
/// thing over phase ordinals, and it is a disjunction for the same reason: the
/// two ends of the dependency are in different rounds.
///
/// **The peel, which is not.** Round zero is emitted ahead of the loop, so its
/// reads are ordinary predecessors of the loop's first write and need a
/// realization between them in flat program order. Only the `b < w` arm of the
/// cyclic rule also satisfies this, which is why the canonical body puts the
/// round boundary at the head of the loop rather than the tail — and why a body
/// that put it at the tail is refused here rather than by inspection.
fn verify_anti_edge_is_ordered(
    walk: &Walk,
    data: &KernelData,
    tile: &CooperativeTile,
    edge: AntiDependencyEdge,
    round: Option<&Range<usize>>,
) -> Result<(), KernelDiagnostic> {
    let discharging = tile.anti_discharging_points(edge);
    let [point] = discharging.as_slice() else {
        return Err(KernelDiagnostic::UndischargedAntiDependency);
    };
    let point = point.id;
    // An anti-dependency exists only on a tile whose phases repeat, and
    // `round_body` already proved such a tile has a round loop.
    let Some(body) = round else {
        return Err(KernelDiagnostic::UnorderedStagedRewrite);
    };
    let writes = staged_positions(walk, data, body, edge.staging, edge.rewritten_in, true);
    let reads = staged_positions(walk, data, body, edge.staging, edge.consumed_in, false);
    let (Some(first_write), Some(last_read)) =
        (writes.iter().min().copied(), reads.iter().max().copied())
    else {
        return Err(KernelDiagnostic::UnorderedStagedRewrite);
    };
    if !realizations(walk)
        .filter(|(position, realized)| body.contains(position) && *realized == point)
        .any(|(position, _)| position > last_read || position < first_write)
    {
        return Err(KernelDiagnostic::UnorderedStagedRewrite);
    }
    let peel = 0..body.start;
    let peeled_reads = staged_positions(walk, data, &peel, edge.staging, edge.consumed_in, false);
    let Some(last_peeled_read) = peeled_reads.iter().max().copied() else {
        return Err(KernelDiagnostic::UnorderedStagedRewrite);
    };
    if !realizations(walk)
        .filter(|(_, realized)| *realized == point)
        .any(|(position, _)| position > last_peeled_read && position < first_write)
    {
        return Err(KernelDiagnostic::UnorderedStagedRewrite);
    }
    Ok(())
}

/// Collects the values that denote a schedule-derived governed predicate.
///
/// Two forms, and both bounds come from the schedule rather than from the body.
///
/// The **iteration guard** compares an admitted global invocation index against
/// the exact scheduled work-item count; every kernel has one and every memory
/// effect must be dominated by it.
///
/// The **commit guard** compares an admitted *local* invocation index against a
/// cooperative tile's committing participant count, and exists only for a
/// cooperative region. It is what lets several invocations reach the same
/// output's phases while exactly one of them stores, which is the fact
/// `OwnershipProofKind::OneGlobalInvocationPerOutput` rests on for a tile. It is
/// admitted only when the tile's commit range starts at zero, because
/// `IndexLessThan` cannot express "equals `k`" for a nonzero `k`; a tile that
/// commits from some other participant has no governed predicate here and its
/// body is refused rather than approximated.
///
/// Any other predicate leaves an effect undominated by schedule-derived
/// evidence.
fn guard_values(data: &KernelData, schedule: &ScheduledRegion) -> BTreeSet<u32> {
    let mut global = BTreeSet::new();
    let mut local = BTreeSet::new();
    for block in &data.blocks {
        for operation in &block.operations {
            if let OperationKind::Builtin { builtin } = operation.kind {
                let admitted = match builtin {
                    Builtin::GlobalInvocationIndex => &mut global,
                    Builtin::LocalInvocationIndex => &mut local,
                };
                admitted.extend(operation.results.iter().copied());
            }
        }
    }
    let commit = cooperative_tile(&schedule.schedule.reduction)
        .map(|tile| tile.commit)
        .filter(|commit| commit.first == 0)
        .map(|commit| commit.count);
    let mut guards = BTreeSet::new();
    for block in &data.blocks {
        for operation in &block.operations {
            let OperationKind::Compare {
                op: CompareOp::IndexLessThan,
                lhs,
                rhs,
            } = operation.kind
            else {
                continue;
            };
            let bound = data
                .values
                .get(rhs as usize)
                .and_then(|value| value.constant)
                .and_then(KernelConstant::as_index);
            let governed = (global.contains(&lhs) && bound == Some(schedule.schedule.work_items))
                || (local.contains(&lhs) && bound.is_some() && bound == commit);
            if governed {
                guards.extend(operation.results.iter().copied());
            }
        }
    }
    guards
}

fn axis_guards(data: &KernelData, schedule: &ScheduledRegion) -> AxisGuards {
    let Some(geom) = blocked_geometry(schedule) else {
        return AxisGuards::default();
    };
    let mut roles = BTreeMap::new();
    let mut axis = AxisGuards::default();
    for block in &data.blocks {
        for operation in &block.operations {
            match &operation.kind {
                OperationKind::Builtin {
                    builtin: Builtin::GlobalInvocationIndex,
                } => {
                    for result in &operation.results {
                        roles.insert(*result, IndexRole::Gid);
                    }
                }
                OperationKind::Builtin {
                    builtin: Builtin::LocalInvocationIndex,
                } => {
                    for result in &operation.results {
                        roles.insert(*result, IndexRole::Lid);
                    }
                }
                OperationKind::Constant {
                    value: KernelConstant::Index(value),
                } => {
                    for result in &operation.results {
                        roles.insert(*result, IndexRole::Const(*value));
                    }
                }
                OperationKind::Binary { op, lhs, rhs } => {
                    let Some(role) = classify_binary(*op, *lhs, *rhs, &roles, &geom) else {
                        continue;
                    };
                    for result in &operation.results {
                        roles.insert(*result, role);
                    }
                }
                OperationKind::Compare {
                    op: CompareOp::IndexLessThan,
                    lhs,
                    rhs,
                } => {
                    let bound = match roles.get(rhs) {
                        Some(IndexRole::Const(value)) => *value,
                        _ => continue,
                    };
                    let Some(lhs_role) = roles.get(lhs).copied() else {
                        continue;
                    };
                    let lhs_role = normalize_coord(lhs_role, &geom);
                    for result in &operation.results {
                        if lhs_role == IndexRole::Row && bound == geom.output_m {
                            axis.row.insert(*result);
                        }
                        if lhs_role == IndexRole::Col && bound == geom.output_n {
                            axis.column.insert(*result);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    axis
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IndexRole {
    Gid,
    Lid,
    Const(u64),
    WgLinear,
    WgM,
    WgN,
    LocalM,
    LocalN,
    Row,
    Col,
}

struct BlockedGeom {
    block_m: u64,
    block_n: u64,
    workgroups_m: u64,
    workgroups_n: u64,
    output_m: u64,
    output_n: u64,
    threads_per_workgroup: u64,
}

fn blocked_geometry(schedule: &ScheduledRegion) -> Option<BlockedGeom> {
    let ExecutionBinding::BlockedWorkgroup { block, workgroups } = &schedule.schedule.binding
    else {
        return None;
    };
    if block.rank() != 2 || workgroups.rank() != 2 || schedule.index.iteration_shape.rank() != 2 {
        return None;
    }
    let block_m = block.extents()[0].get();
    let block_n = block.extents()[1].get();
    let workgroups_m = workgroups.extents()[0].get();
    let workgroups_n = workgroups.extents()[1].get();
    let output_m = schedule.index.iteration_shape.extents()[0].get();
    let output_n = schedule.index.iteration_shape.extents()[1].get();
    let threads_per_workgroup = block_m.checked_mul(block_n)?;
    Some(BlockedGeom {
        block_m,
        block_n,
        workgroups_m,
        workgroups_n,
        output_m,
        output_n,
        threads_per_workgroup,
    })
}

fn normalize_coord(role: IndexRole, geom: &BlockedGeom) -> IndexRole {
    match role {
        IndexRole::LocalM if geom.workgroups_m == 1 => IndexRole::Row,
        IndexRole::LocalN if geom.workgroups_n == 1 => IndexRole::Col,
        other => other,
    }
}

fn classify_binary(
    op: BinaryOp,
    lhs: u32,
    rhs: u32,
    roles: &BTreeMap<u32, IndexRole>,
    geom: &BlockedGeom,
) -> Option<IndexRole> {
    let left = roles.get(&lhs).copied()?;
    let right = roles.get(&rhs).copied()?;
    match op {
        BinaryOp::IndexDivide => match (left, right) {
            (IndexRole::Gid, IndexRole::Const(divisor))
                if divisor == geom.threads_per_workgroup =>
            {
                Some(IndexRole::WgLinear)
            }
            (IndexRole::WgLinear, IndexRole::Const(divisor)) if divisor == geom.workgroups_n => {
                Some(IndexRole::WgM)
            }
            (IndexRole::Lid, IndexRole::Const(divisor)) if divisor == geom.block_n => {
                Some(IndexRole::LocalM)
            }
            _ => None,
        },
        BinaryOp::IndexModulo => match (left, right) {
            (IndexRole::WgLinear, IndexRole::Const(modulus)) if modulus == geom.workgroups_n => {
                Some(IndexRole::WgN)
            }
            (IndexRole::Lid, IndexRole::Const(modulus)) if modulus == geom.block_n => {
                Some(IndexRole::LocalN)
            }
            _ => None,
        },
        BinaryOp::IndexMultiply => match (left, right) {
            (IndexRole::WgM, IndexRole::Const(factor))
            | (IndexRole::Const(factor), IndexRole::WgM)
                if factor == geom.block_m =>
            {
                Some(IndexRole::WgM)
            }
            (IndexRole::WgN, IndexRole::Const(factor))
            | (IndexRole::Const(factor), IndexRole::WgN)
                if factor == geom.block_n =>
            {
                Some(IndexRole::WgN)
            }
            _ => None,
        },
        BinaryOp::IndexAdd => {
            let pair = (left, right);
            match pair {
                (IndexRole::WgM, IndexRole::LocalM) | (IndexRole::LocalM, IndexRole::WgM) => {
                    Some(IndexRole::Row)
                }
                (IndexRole::WgN, IndexRole::LocalN) | (IndexRole::LocalN, IndexRole::WgN) => {
                    Some(IndexRole::Col)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// After `IndexMultiply` of a workgroup-m term by the block extent the role
/// stays that term so the following add can form `m`.
fn verify_predicated_contraction_roles(
    walk: &Walk,
    data: &KernelData,
    schedule: &ScheduledRegion,
    reads: &[Access],
) -> Result<(), KernelDiagnostic> {
    if schedule.schedule.tail != TailPolicy::Predicated {
        return Ok(());
    }
    if !matches!(
        schedule.schedule.reduction,
        ReductionTopology::CooperativeContraction { .. }
    ) {
        return Ok(());
    }
    let [left, right] = reads else {
        return Err(KernelDiagnostic::ScheduleAccessCount);
    };
    let left_buffer = u32::try_from(
        data.buffers
            .iter()
            .position(|buffer| buffer.tensor == left.tensor)
            .ok_or(KernelDiagnostic::BufferContract)?,
    )
    .map_err(|_| KernelDiagnostic::BufferContract)?;
    let right_buffer = u32::try_from(
        data.buffers
            .iter()
            .position(|buffer| buffer.tensor == right.tensor)
            .ok_or(KernelDiagnostic::BufferContract)?,
    )
    .map_err(|_| KernelDiagnostic::BufferContract)?;
    let axis = axis_guards(data, schedule);
    let plus_zero = data.values.iter().enumerate().find_map(|(index, value)| {
        matches!(value.constant, Some(KernelConstant::F32Bits(0)))
            .then(|| u32::try_from(index).ok())
            .flatten()
    });
    for load in &walk.guarded_loads {
        let inactive_ok = plus_zero == Some(load.inactive)
            || data
                .values
                .get(load.inactive as usize)
                .and_then(|value| value.constant)
                == Some(KernelConstant::F32Bits(0.0_f32.to_bits()));
        if !inactive_ok {
            return Err(KernelDiagnostic::BodyRefinement);
        }
        if load.buffer == left_buffer && !axis.row.contains(&load.predicate) {
            return Err(KernelDiagnostic::LeftLoadGuard);
        }
        if load.buffer == right_buffer && !axis.column.contains(&load.predicate) {
            return Err(KernelDiagnostic::RightLoadGuard);
        }
    }
    let has_left = walk
        .guarded_loads
        .iter()
        .any(|load| load.buffer == left_buffer);
    let has_right = walk
        .guarded_loads
        .iter()
        .any(|load| load.buffer == right_buffer);
    if !has_left {
        return Err(KernelDiagnostic::LeftLoadGuard);
    }
    if !has_right {
        return Err(KernelDiagnostic::RightLoadGuard);
    }
    for effect in &walk.effects {
        if matches!(effect.kind, EffectKind::Store { .. }) && (!effect.row || !effect.column) {
            return Err(KernelDiagnostic::OutputStoreGuard);
        }
    }
    Ok(())
}

fn visit_block(visit: &mut Visit<'_>, block: u32, nest: Nest, loop_depth: u32, block_depth: u32) {
    let Some(block_data) = visit.data.blocks.get(block as usize) else {
        return;
    };
    let operations = block_data.operations.clone();
    for operation in &operations {
        match &operation.kind {
            OperationKind::Load { bounds, .. } => visit.walk.effects.push(Effect {
                kind: EffectKind::Load {
                    bounds: *bounds,
                    ordinary: true,
                },
                loop_depth,
                guarded: nest.has(Nest::GUARDED),
                row: nest.has(Nest::ROW),
                column: nest.has(Nest::COLUMN),
            }),
            OperationKind::GuardedLoad {
                predicate,
                buffer,
                bounds,
                inactive,
                ..
            } => {
                visit.walk.effects.push(Effect {
                    kind: EffectKind::Load {
                        bounds: *bounds,
                        ordinary: false,
                    },
                    loop_depth,
                    guarded: true,
                    row: visit.axis.row.contains(predicate),
                    column: visit.axis.column.contains(predicate),
                });
                visit.walk.guarded_loads.push(GuardedLoadFact {
                    predicate: *predicate,
                    buffer: *buffer,
                    inactive: *inactive,
                });
            }
            OperationKind::Store {
                bounds, ownership, ..
            } => visit.walk.effects.push(Effect {
                kind: EffectKind::Store {
                    bounds: *bounds,
                    ownership: *ownership,
                },
                loop_depth,
                guarded: nest.has(Nest::GUARDED),
                row: nest.has(Nest::ROW),
                column: nest.has(Nest::COLUMN),
            }),
            OperationKind::Barrier { spec } => {
                visit.walk.has_synchronization = true;
                visit.walk.barriers.push(spec.clone());
                visit.walk.sync.push(SyncEvent::Barrier {
                    point: spec.point,
                    block_depth,
                    loop_depth,
                });
            }
            OperationKind::StagedStore { staging, phase, .. } => {
                if nest.has(Nest::PREDICATED)
                    && (nest.has(Nest::GUARDED) || nest.has(Nest::ROW) || nest.has(Nest::COLUMN))
                {
                    visit.walk.incomplete_staging = true;
                }
                visit.walk.sync.push(SyncEvent::StagedWrite {
                    staging: *staging,
                    phase: *phase,
                });
            }
            OperationKind::StagedLoad { staging, phase, .. } => {
                visit.walk.sync.push(SyncEvent::StagedRead {
                    staging: *staging,
                    phase: *phase,
                });
            }
            OperationKind::Predicated { predicate, body } => {
                if !visit.guards.contains(predicate) {
                    visit.walk.ungoverned_predicate = true;
                }
                let mut inner = nest.with(Nest::GUARDED);
                if visit.axis.row.contains(predicate) {
                    inner = inner.with(Nest::ROW);
                }
                if visit.axis.column.contains(predicate) {
                    inner = inner.with(Nest::COLUMN);
                }
                visit_block(
                    visit,
                    *body,
                    inner,
                    loop_depth,
                    block_depth.saturating_add(1),
                );
            }
            OperationKind::SerialLoopRange {
                start, end, body, ..
            } => {
                let accumulators = visit
                    .data
                    .blocks
                    .get(*body as usize)
                    .map(|inner| {
                        inner
                            .parameters
                            .iter()
                            .skip(1)
                            .filter_map(|index| visit.data.values.get(*index as usize))
                            .map(|value| value.value_type)
                            .collect()
                    })
                    .unwrap_or_default();
                let summary = visit.walk.loops.len();
                let first_event = visit.walk.sync.len();
                visit.walk.loops.push(LoopSummary {
                    start: 0,
                    end: 0,
                    start_value: Some(*start),
                    end_value: Some(*end),
                    accumulators,
                    block_depth,
                    sync: first_event..first_event,
                });
                visit_block(
                    visit,
                    *body,
                    nest,
                    loop_depth.saturating_add(1),
                    block_depth.saturating_add(1),
                );
                let last_event = visit.walk.sync.len();
                if let Some(summary) = visit.walk.loops.get_mut(summary) {
                    summary.sync = first_event..last_event;
                }
            }
            OperationKind::SerialLoop {
                start, end, body, ..
            } => {
                let accumulators = visit
                    .data
                    .blocks
                    .get(*body as usize)
                    .map(|inner| {
                        inner
                            .parameters
                            .iter()
                            .skip(1)
                            .filter_map(|index| visit.data.values.get(*index as usize))
                            .map(|value| value.value_type)
                            .collect()
                    })
                    .unwrap_or_default();
                let summary = visit.walk.loops.len();
                let first_event = visit.walk.sync.len();
                visit.walk.loops.push(LoopSummary {
                    start: *start,
                    end: *end,
                    start_value: None,
                    end_value: None,
                    accumulators,
                    block_depth,
                    sync: first_event..first_event,
                });
                visit_block(
                    visit,
                    *body,
                    nest,
                    loop_depth.saturating_add(1),
                    block_depth.saturating_add(1),
                );
                let last_event = visit.walk.sync.len();
                if let Some(summary) = visit.walk.loops.get_mut(summary) {
                    summary.sync = first_event..last_event;
                }
            }
            OperationKind::Builtin { .. }
            | OperationKind::Constant { .. }
            | OperationKind::Binary { .. }
            | OperationKind::Compare { .. }
            | OperationKind::Convert { .. }
            | OperationKind::Unary { .. }
            | OperationKind::PackedExtract { .. }
            | OperationKind::InputExtent { .. } => {}
        }
    }
}

/// Verifies the ordered boundary memory effects of one kernel.
///
/// Staged accesses are deliberately not counted here. This function's subject is
/// the region's *boundary* contract — which tensors are read, which position is
/// owned, and that the owning store commits last — and workgroup staging is
/// neither a boundary tensor nor an owned position. Counting a staged store
/// among the stores would make "exactly one store per invocation" false of every
/// cooperative kernel; [`verify_synchronization`] owns the staged half.
fn verify_effects(
    walk: &Walk,
    schedule: &ScheduledRegion,
    reads: &[Access],
    write: &Access,
) -> Result<(), KernelDiagnostic> {
    let exact_contraction = matches!(schedule.schedule.tail, TailPolicy::Exact)
        && matches!(
            schedule.schedule.reduction,
            ReductionTopology::CooperativeContraction { .. }
        );
    if walk.effects.iter().any(|effect| !effect.guarded) && !exact_contraction {
        return Err(KernelDiagnostic::PredicateDominance);
    }
    let mut stores = 0_usize;
    for effect in &walk.effects {
        match effect.kind {
            EffectKind::Load { bounds, ordinary } => {
                if !reads.iter().any(|read| bounds == read.bounds) {
                    return Err(KernelDiagnostic::BoundsEvidence);
                }
                if ordinary
                    && matches!(schedule.schedule.tail, TailPolicy::Predicated)
                    && matches!(
                        schedule.schedule.reduction,
                        ReductionTopology::CooperativeContraction { .. }
                    )
                {
                    return Err(KernelDiagnostic::BoundsEvidence);
                }
            }
            EffectKind::Store { bounds, ownership } => {
                stores = stores.saturating_add(1);
                if bounds != write.bounds {
                    return Err(KernelDiagnostic::BoundsEvidence);
                }
                if ownership != schedule.schedule.output_owner
                    || ownership != schedule.index.ownership_proof.id
                {
                    return Err(KernelDiagnostic::OwnershipEvidence);
                }
                if effect.loop_depth != 0
                    && !(effect.loop_depth == 1
                        && matches!(
                            write.map,
                            crate::schedule::LogicalAccess::LiveRowMajor { .. }
                        ))
                {
                    return Err(KernelDiagnostic::OutputCoverage);
                }
            }
        }
    }
    if stores != 1 {
        return Err(KernelDiagnostic::OutputCoverage);
    }
    let commits_last = walk
        .effects
        .last()
        .is_some_and(|effect| matches!(effect.kind, EffectKind::Store { .. }));
    if !commits_last {
        return Err(KernelDiagnostic::EffectOrdering);
    }
    Ok(())
}

fn verify_reduction(
    walk: &Walk,
    data: &KernelData,
    schedule: &ScheduledRegion,
    reads: &[Access],
) -> Result<(), KernelDiagnostic> {
    match &schedule.schedule.reduction {
        ReductionTopology::None => {
            let live_row_major = reads.iter().any(|read| {
                matches!(
                    read.map,
                    crate::schedule::LogicalAccess::LiveRowMajor { .. }
                )
            });
            if walk.loops.is_empty() || (live_row_major && walk.loops.len() == 1) {
                Ok(())
            } else {
                Err(KernelDiagnostic::ReductionContract)
            }
        }
        ReductionTopology::Serial { axes, .. }
        | ReductionTopology::MultiPass {
            pass: ReductionPass::Final,
            axes,
            ..
        } => {
            let [read] = reads else {
                return Err(KernelDiagnostic::ReductionContract);
            };
            let contributors = contributor_count(axes, &read.map)
                .map_err(|_| KernelDiagnostic::ContributorDomain)?;
            verify_contributor_loop(walk, contributors)
        }
        // A partial pass combines its own partition's share, which the split
        // states directly. Counting the access's contributors here would count
        // the whole sequence and reject every partition but a trivial one.
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage,
            ..
        } => verify_contributor_loop(walk, exact_partition(*coverage)?.contributors_per_partition),
        // The two-read case. A contraction's contributor count is the size of
        // its contracted index space, which no single read's map determines:
        // each operand names only the contracted coordinates its own tuple
        // mentions. The topology states it, and the schedule verifier already
        // proved both operand maps agree with that statement — so the fold
        // obligation itself is the shared one, including the `start == 1` seed
        // at the first product.
        ReductionTopology::Contraction {
            contracted_shape, ..
        } => {
            if reads.len() != 2 {
                return Err(KernelDiagnostic::ReductionContract);
            }
            let contributors = element_count(contracted_shape)
                .map_err(|_| KernelDiagnostic::ElementCountOverflow)?;
            if contributors == 0 {
                return Err(KernelDiagnostic::ContributorDomain);
            }
            verify_contributor_loop(walk, contributors)
        }
        ReductionTopology::LiveContraction { .. } => {
            if reads.len() != 2 {
                return Err(KernelDiagnostic::ReductionContract);
            }
            verify_live_contributor_loop(walk, data, schedule)
        }
        // A cooperative fold is two folds, and the split is exactly what the
        // partition states: each participant combines its own contiguous
        // contributor range, and the committing participant combines the staged
        // partials in ascending participant order. Both trip counts come from
        // the partition rather than from the access, for the reason a partial
        // pass's does — the access counts the whole sequence.
        // The participant count is the tile's extent product, which is the same
        // number of invocations whatever shape the space arranges them in: the
        // fold this verifies is over staged partials, one per participant, and
        // it is indifferent to the participant coordinate's rank.
        ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } => verify_cooperative_loops(
            walk,
            exact_partition(*coverage)?,
            tile.coordinates
                .participants
                .participants()
                .ok_or(KernelDiagnostic::ElementCountOverflow)?,
            tile.rounds,
        ),
        ReductionTopology::CooperativeContraction {
            contracted_tile, ..
        } => {
            if reads.len() != 2 {
                return Err(KernelDiagnostic::ReductionContract);
            }
            let tile_k = contracted_tile
                .extents()
                .first()
                .map_or(0, |extent| extent.get());
            if tile_k <= 1 {
                return if walk.loops.is_empty() {
                    Ok(())
                } else {
                    Err(KernelDiagnostic::ReductionContract)
                };
            }
            let Some(inner) = walk
                .loops
                .iter()
                .find(|reduction| reduction.start == 1 && reduction.end == tile_k)
            else {
                return Err(KernelDiagnostic::ReductionContract);
            };
            if inner.accumulators != [KernelType::F32] {
                return Err(KernelDiagnostic::ReductionContract);
            }
            Ok(())
        }
    }
}

/// Proves the body realizes every fold a cooperative tile's rounds require.
///
/// A trivial half emits no loop at all, exactly as the serial contract does for
/// a single contributor: a loop over one element would need an empty range.
///
/// **A single-round tile folds twice**, in the two guards its body nests: each
/// participant's contributor share inside the iteration guard, and the staged set
/// inside the commit guard.
///
/// **A loop-carried tile folds five times, and the shape is what the round
/// structure forces.** Round zero is peeled, because a fold seeds at its first
/// contributor, so its contributor fold and its staged fold are emitted ahead of
/// the round loop — and the staged fold is at the kernel's *top level* rather
/// than inside the commit guard, because its result is the round loop's initial
/// accumulator and a predicated region produces no value that could cross the
/// back edge. The round loop then repeats both folds inside itself, one block
/// deeper each. The expected block depths are therefore
/// `1, 0, 0, 2, 1` — and it is the depths, not the trip counts, that distinguish
/// the peel's folds from the loop's.
fn exact_partition(
    coverage: ContributorCoverage,
) -> Result<ContributorPartition, KernelDiagnostic> {
    match coverage {
        ContributorCoverage::Exact(partition) => Ok(partition),
        ContributorCoverage::IdentityPadded { .. } => {
            Err(KernelDiagnostic::PaddedContributorCoverage)
        }
    }
}

fn verify_cooperative_loops(
    walk: &Walk,
    partition: ContributorPartition,
    participants: u64,
    rounds: u64,
) -> Result<(), KernelDiagnostic> {
    let contributors = partition.contributors_per_partition;
    let mut expected = Vec::with_capacity(5);
    if contributors > 1 {
        // Inside the iteration guard, which is block depth one.
        expected.push((contributors, 1_u32));
    }
    if rounds > 1 {
        // The peeled round's staged fold, at the kernel's top level, then the
        // round loop, then the loop body's own two folds one block deeper.
        if participants > 1 {
            expected.push((participants, 0_u32));
        }
        expected.push((rounds, 0_u32));
        if contributors > 1 {
            expected.push((contributors, 2_u32));
        }
        if participants > 1 {
            expected.push((participants, 1_u32));
        }
    } else if participants > 1 {
        // Inside the iteration guard and then the commit guard, which is two.
        expected.push((participants, 2_u32));
    }
    if walk.loops.len() != expected.len() {
        return Err(KernelDiagnostic::ReductionContract);
    }
    for (summary, (end, block_depth)) in walk.loops.iter().zip(expected) {
        if summary.start != 1
            || summary.end != end
            || summary.accumulators != [KernelType::F32]
            || summary.block_depth != block_depth
        {
            return Err(KernelDiagnostic::ReductionContract);
        }
    }
    Ok(())
}

/// Proves the body realizes exactly the scheduled contributor fold.
///
/// A live contraction has no empty result, so a program selecting this verified
/// kernel derives the precondition that its contributor extent is nonzero.
/// Exactly one contributor therefore commits the single loaded value and leaves
/// the `1..S` loop body empty; no admitted zero-contributor invocation executes
/// the seed.
fn verify_live_contributor_loop(
    walk: &Walk,
    data: &KernelData,
    schedule: &ScheduledRegion,
) -> Result<(), KernelDiagnostic> {
    let [reduction] = walk.loops.as_slice() else {
        return Err(KernelDiagnostic::ReductionContract);
    };
    if reduction.start != 0
        || reduction.end != 0
        || reduction.accumulators != [KernelType::F32]
        || reduction.block_depth != 1
    {
        return Err(KernelDiagnostic::ReductionContract);
    }
    let (Some(start), Some(end)) = (reduction.start_value, reduction.end_value) else {
        return Err(KernelDiagnostic::ReductionContract);
    };
    match defining_kind(data, start) {
        Some(OperationKind::Constant {
            value: KernelConstant::Index(1),
        }) => {}
        _ => return Err(KernelDiagnostic::ReductionContract),
    }
    let Some(OperationKind::InputExtent { parameter }) = defining_kind(data, end) else {
        return Err(KernelDiagnostic::ReductionContract);
    };
    let Some(declared) = data.input_extents.get(*parameter as usize) else {
        return Err(KernelDiagnostic::InputExtentContract);
    };
    let expected = live_input_extents(schedule);
    if expected.first().copied() != Some((declared.tensor, declared.axis)) {
        return Err(KernelDiagnostic::InputExtentContract);
    }
    let Some(end_value) = data.values.get(end as usize) else {
        return Err(KernelDiagnostic::InputExtentContract);
    };
    if end_value.block != 0 {
        return Err(KernelDiagnostic::InputExtentContract);
    }
    Ok(())
}

fn defining_kind(data: &KernelData, value: u32) -> Option<&OperationKind> {
    data.blocks.iter().find_map(|block| {
        block.operations.iter().find_map(|operation| {
            operation
                .results
                .contains(&value)
                .then_some(&operation.kind)
        })
    })
}

fn verify_contributor_loop(walk: &Walk, contributors: u64) -> Result<(), KernelDiagnostic> {
    if contributors <= 1 {
        return if walk.loops.is_empty() {
            Ok(())
        } else {
            Err(KernelDiagnostic::ReductionContract)
        };
    }
    let [reduction] = walk.loops.as_slice() else {
        return Err(KernelDiagnostic::ReductionContract);
    };
    if reduction.start != 1
        || reduction.end != contributors
        || reduction.accumulators != [KernelType::F32]
        || reduction.block_depth != 1
    {
        return Err(KernelDiagnostic::ReductionContract);
    }
    Ok(())
}
