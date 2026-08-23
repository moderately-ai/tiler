//! Canonical target-neutral lowering of a verified scheduled region.
//!
//! This is the authoritative structured realization of the bounded profile. It
//! makes every fact a backend needs explicit in the IR itself: the guarded
//! iteration predicate, the exact element-offset arithmetic that realizes the
//! scheduled logical access, the typed loads and stores with their schedule
//! witnesses, the named NaN-canonicalization conversion the numerical contract
//! requires, and — for a reduction — a bounded loop carrying the accumulator in
//! the exact scheduled contributor order. No backend has to consult the
//! semantic graph, re-derive an access relation, or infer a reduction order.
//!
//! The lowering constructs its kernel through the same public
//! [`KernelBuilder`] path an external producer uses, so it cannot bypass an
//! insertion-time invariant. [`super::verify`] re-derives this canonical body
//! and requires structural equality, which is what makes a producer-authored
//! kernel a proven refinement rather than a trusted one.

use crate::schedule::{
    Access, BoundsWitnessId, CanonicalScheduledRegionIdentity, ContributorCoverage,
    ExecutionBinding, LogicalAccess, NumericalRealization, OwnershipWitnessId,
    PointwiseBf16Expression, PointwiseBf16Node, PointwiseF32Expression, PointwiseF32Node,
    ReductionPass, ReductionTopology, RegionProgram, ResourceRequirements, ScalarProgram,
    ScheduledRegion, TailPolicy, TensorRole, VerifiedScheduledRegion, contributor_count,
    element_count, gather_index_read_map, live_input_extents, live_source_axis,
};
use crate::shape::Shape;

use super::builder::KernelBuilder;
use super::error::{KernelBuildError, KernelDiagnostic, KernelLoweringError};
use super::handles::{KernelBufferId, KernelStagingId, KernelValueId};
use super::model::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BufferAccess, BufferParameter, Builtin,
    CompareOp, ConvertOp, ExecutionScope, InputExtentParameter, KernelConstant, KernelData,
    KernelType, MemoryScope, PackedExtractOp, SerialLoopSpec, StagingParameter, UnaryOp,
    VerifiedKernel, region_element_type,
};
use super::verify::{access_elements, boundary_accesses, gather_address_reads};

/// Which root index a linearization term extracts its coordinate from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OffsetRoot {
    /// The linear output coordinate carried by the global invocation index.
    Output,
    /// The linear contributor coordinate carried by the loop induction variable.
    Contributor,
}

/// One `stride * ((root / divisor) % modulus)` term of a linearized offset.
///
/// `mirror` replaces the decoded coordinate `c` by `extent − 1 − c` before the
/// stride is applied, which is the one structural map that needs it: a reindex
/// reversing an axis. It carries the axis extent rather than a bare flag because
/// the mirror is stated against that extent and a term that had to look it up
/// elsewhere could be emitted against the wrong one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OffsetTerm {
    root: OffsetRoot,
    divisor: u64,
    modulus: Option<u64>,
    mirror: Option<u64>,
    stride: u64,
}

/// Which coordinate of a blocked cooperative contraction one operand axis reads.
///
/// The coordinates a blocked body already holds as separate values: the
/// participant's output row, its output column, the contracted index its staged
/// tile load names, and — on a batched block — one coordinate per leading output
/// axis. Naming the coordinate rather than a position in a linear space is the
/// whole point of this form — it is what lets the address be a sum of
/// `stride * coordinate` with no decode, where [`OffsetTerm`] would need a divide
/// and a modulo to recover coordinates this body never linearized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BlockedCoordinate {
    /// The output coordinate one leading batch axis names.
    ///
    /// Carries the axis's position in the *output* shape, which on a batched
    /// block is also its ordinal among the batch axes: the participants occupy
    /// the trailing two axes, so every axis before them is a batch axis and the
    /// two numberings coincide. A workgroup covers exactly one coordinate on each
    /// such axis — the block's leading extents are all one, which is the relation
    /// `blocked_batch_prefix` enforces — so this coordinate is the workgroup's
    /// own position on that axis and needs no participant dimension.
    Batch(usize),
    /// The output coordinate the participant's block row names.
    Row,
    /// The output coordinate the participant's block column names.
    Column,
    /// The contracted coordinate the participant's staged tile load names.
    ///
    /// Its value differs between the two operands — participant `(m, n)` fetches
    /// the left tile's column `n` and the right tile's column `m` — so the
    /// emission supplies it rather than this term carrying it.
    Contracted,
}

/// One `stride * coordinate` term of a blocked contraction operand's address.
///
/// The stride is the operand's *own* row-major stride for that axis, so a
/// transposed operand yields the same two terms with the strides exchanged. That
/// exchange is the entire difference between `[M, K]` and `[K, M]`, and reading
/// it from the declared sources is what keeps the emitted address the one the
/// region states rather than the one the first fixture happened to use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct BlockedTerm {
    coordinate: BlockedCoordinate,
    stride: u64,
}

/// How the read access computes its element offset.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ReadAddressing {
    /// One iteration coordinate addresses one linear element position.
    Identity,
    /// One invocation owns a static outer coordinate and loops a live inner
    /// extent: `offset = row * N + col`.
    LiveRowMajor { inner_axis: crate::shape::Axis },
    /// One contraction operand addressed by its static free coordinates and a
    /// live inner contracted index: `offset = free * S + contributor`.
    LiveContracted { free: Vec<OffsetTerm> },
    /// One operand of a blocked cooperative contraction, addressed by the block
    /// coordinates its body already holds.
    ///
    /// Deliberately not [`Self::Linearized`]. That form decodes every coordinate
    /// out of one linear root, and a blocked body has no such root: its row,
    /// column, and contracted coordinates are separate values built from the
    /// workgroup and local indices. Reconstructing a linear coordinate to decode
    /// it again would add a divide and a modulo per operand per round to a
    /// kernel that carries a retained timing.
    BlockedContraction(Vec<BlockedTerm>),
    /// A reduction contributor position linearized over the input shape.
    Linearized(Vec<OffsetTerm>),
    /// A partitioned contributor position of one pass of a split reduction.
    ///
    /// The invocation index carries the output coordinate *and* the partition
    /// ordinal, because the partial pass runs one invocation per
    /// (output, partition) pair. Splitting them here is what keeps the shared
    /// linearization below unchanged: it still receives one linear output
    /// coordinate and one linear contributor coordinate.
    Partitioned {
        /// Row-major terms over the read tensor's own shape.
        terms: Vec<OffsetTerm>,
        /// Partial values per output position.
        partitions: u64,
        /// Contributors each partition combines.
        contributors_per_partition: u64,
    },
    /// The data-dependent read, addressed by direct coordinates and one
    /// coordinate loaded from the region's index operand.
    ///
    /// The one form that reads memory to build an address, which is why it is a
    /// form of its own rather than a term inside [`Self::Linearized`]: every
    /// other term here is arithmetic on a linear root the body already holds,
    /// and this one has to emit a load, a widening conversion, and the scale
    /// before the sum exists at all.
    ///
    /// Boxed because it is far the widest arm and every read of every region
    /// carries a `ReadAddressing`; the indirection costs a pointer chase in the
    /// one family that takes it and keeps the other seven at their old size.
    Gather(Box<GatherAddressing>),
    /// A gather's address-only index operand.
    ///
    /// Fieldless deliberately. This read supplies coordinates rather than a
    /// value, it is never loaded on its own, and how it computes its own offset
    /// is stated once — in the owning [`GatherAddressing::index`] — so a field
    /// here would be a second account of the same addressing that the two could
    /// disagree in. What this form carries is the fact that the read *is* an
    /// address operand, which is what keeps it out of the scalar leaves.
    GatherAddress,
}

/// How one data-dependent read builds the element offset it loads from.
///
/// The direct half and the indirect half, kept apart because they come from
/// different places: `direct` is arithmetic on the invocation index alone, and
/// the gathered coordinate exists only after a load. A single term list could
/// not state that ordering.
#[derive(Clone, Debug, Eq, PartialEq)]
struct GatherAddressing {
    /// Row-major terms for every source axis other than the gathered one.
    direct: Vec<OffsetTerm>,
    /// The source tensor's own row-major stride on the gathered axis.
    ///
    /// Carried rather than recomputed at emission for the reason
    /// [`OffsetTerm::mirror`] carries its extent: the scale is stated against
    /// this source shape, and an emission that looked it up elsewhere could
    /// apply the wrong one.
    gathered_stride: u64,
    /// Region-local position of the address-supplying read.
    index_read: usize,
    /// How that read computes its own element offset.
    ///
    /// Derived from [`gather_index_read_map`] rather than read off the address
    /// access, so the offset this body emits is the relation the schedule
    /// layer's single authority states and not a second account of it.
    index: ReadAddressing,
}

/// The one cooperative tile shape this lowering has a canonical body for.
///
/// Resolved once, before any operation, so every refusal is a named diagnostic
/// rather than a builder error deep inside an emission. The narrowness is
/// deliberate and stated rather than assumed: a tile is *representable* whenever
/// the schedule verifier admits it, and *lowered* only in this shape.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CooperativePlan {
    /// Invocations of the workgroup that cooperate on one output.
    participants: u64,
    /// Times the whole phase sequence executes.
    ///
    /// `1` is the single-pass tile. A larger count is the loop-carried tile,
    /// whose body peels round zero — a fold seeds at its first contributor —
    /// and carries the accumulator through a `1..rounds` loop.
    rounds: u64,
    /// Contributors each participant folds in the producing phase, on one round.
    contributors_per_partition: u64,
    /// Contributors one whole round covers, which is the round ordinal's stride.
    ///
    /// `participants * contributors_per_partition`. Participant `p` of round `r`
    /// owns the contiguous range at index `r * participants + p`, so its first
    /// contributor is `r * contributors_per_round + p * contributors_per_partition`
    /// — the reassociation the schedule verifier proved covers the declared
    /// sequence once, never a permutation of it.
    contributors_per_round: u64,
    /// Participants that perform the owning store; always the first `n`.
    commit_count: u64,
    /// Scheduled staging allocation the partials are staged in.
    staging: crate::schedule::StagingId,
    /// Slots the allocation holds.
    slots: u64,
    /// Phase whose staged write publishes the partials.
    produce_phase: crate::schedule::PhaseId,
    /// Phase whose staged reads consume them.
    consume_phase: crate::schedule::PhaseId,
    /// Slots between consecutive participants' staged writes.
    produce_stride: u64,
    /// First slot participant zero writes.
    produce_offset: u64,
    /// First slot the consuming fold reads.
    consume_offset: u64,
    /// The realization the phase-boundary point requires, in KIR spelling.
    ///
    /// The point discharging the tile's one visibility edge, resolved through the
    /// discharge relation rather than by position, so the barrier the body emits
    /// is the one the schedule proved orders the handoff.
    barrier: BarrierSpec,
    /// The realization the round-boundary point requires, when there is one.
    ///
    /// `None` for a single-round tile, which derives no anti-dependency and
    /// therefore declares no point to discharge one.
    round_barrier: Option<BarrierSpec>,
}

/// Everything the canonical emission needs, resolved before any operation.
///
/// `reads`, `read_elements`, and `addressing` are parallel and complete: a
/// pointwise region declares one buffer per read and a contraction two with
/// *different* coordinate maps, so a plan carrying only the first read's facts
/// would emit a signature narrower than the region it lowers and address the
/// second operand by the first's relation. `contributors` is the fold length one
/// invocation performs, and is zero for the families that fold nothing.
#[derive(Clone, Debug)]
struct CanonicalPlan<'a> {
    scalar: &'a ScalarProgram,
    reads: &'a [Access],
    write: &'a Access,
    numerical: NumericalRealization,
    write_tensor: TensorRole,
    read_elements: Vec<u64>,
    write_elements: u64,
    work_items: u64,
    write_bounds: BoundsWitnessId,
    ownership: OwnershipWitnessId,
    contributors: u64,
    addressing: Vec<ReadAddressing>,
    cooperative: Option<CooperativePlan>,
    contraction: Option<CooperativeContractionPlan>,
    live_extents: Vec<(crate::schedule::AccessOrdinal, crate::shape::Axis)>,
    live_contraction: Option<(crate::schedule::AccessOrdinal, crate::shape::Axis)>,
}

/// The blocked cooperative-contraction shape this lowering emits.
///
/// Square tiles only: `B_m == B_n == T_k`, which is the measured 16×16
/// `contract_tiled` composition. Other representable tiles stay
/// [`KernelDiagnostic::CooperativeLoweringShape`].
///
/// # Rank
///
/// The *block* takes the output's rank and the participant space stays rank two,
/// which is the only arrangement available:
/// [`MAX_COOPERATIVE_PARTICIPANT_RANK`](crate::schedule::MAX_COOPERATIVE_PARTICIPANT_RANK)
/// is three, so a rank-four participant space is unrepresentable rather than
/// merely unimplemented. Every axis before the trailing pair is therefore a batch
/// axis whose block extent is one, and `batch_workgroups` holds the workgroup
/// count on each — which, because that block extent is one, is also the output
/// extent on that axis.
///
/// A rank-two output is the batched output with no batch axes, so `batch_*` are
/// empty and every derivation below reduces to the unbatched one it replaces.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CooperativeContractionPlan {
    block: u64,
    /// Workgroups on each leading batch axis, in output axis order.
    ///
    /// One workgroup per batch coordinate, so this is the output's own extent on
    /// that axis. Empty for a rank-two output.
    batch_workgroups: Vec<u64>,
    /// Element stride of each leading batch axis in the row-major output.
    ///
    /// Parallel to `batch_workgroups`. The suffix product of every later output
    /// extent, so the last batch axis carries `output_m * output_n`.
    batch_output_strides: Vec<u64>,
    workgroups_m: u64,
    workgroups_n: u64,
    output_m: u64,
    output_n: u64,
    rounds: u64,
    predicated: bool,
    left_staging: crate::schedule::StagingId,
    right_staging: crate::schedule::StagingId,
    left_slots: u64,
    right_slots: u64,
    produce_phase: crate::schedule::PhaseId,
    consume_phase: crate::schedule::PhaseId,
    barrier: BarrierSpec,
    round_barrier: Option<BarrierSpec>,
}

/// Lowers one verified scheduled region to its canonical verified kernel.
///
/// # Errors
///
/// Returns [`KernelLoweringError`] when the region is outside the lowered
/// structured-kernel profile, when an operation cannot be inserted, or when the
/// resulting kernel fails whole-kernel verification.
pub fn lower_scheduled_region(
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernel, KernelLoweringError> {
    let plan = plan(scheduled.region()).map_err(KernelLoweringError::Verification)?;
    let mut builder = KernelBuilder::new(scheduled)?;
    emit(&mut builder, &plan, scheduled.requirements())?;
    builder.build().map_err(|error| {
        KernelLoweringError::Verification(
            error
                .diagnostics()
                .first()
                .copied()
                .unwrap_or(KernelDiagnostic::BodyRefinement),
        )
    })
}

/// Derives the canonical structured body of one scheduled region.
///
/// This is the reference the refinement gate compares a producer's kernel
/// against; it deliberately stops before whole-kernel verification so the gate
/// cannot recurse into itself.
pub(super) fn derive_canonical(
    schedule: &ScheduledRegion,
    schedule_identity: &CanonicalScheduledRegionIdentity,
    requirements: ResourceRequirements,
) -> Result<KernelData, KernelDiagnostic> {
    let plan = plan(schedule)?;
    let mut builder =
        KernelBuilder::from_parts(schedule.clone(), schedule_identity.clone(), requirements)
            .map_err(|_| KernelDiagnostic::BodyRefinement)?;
    emit(&mut builder, &plan, requirements).map_err(|error| match error {
        KernelLoweringError::Verification(diagnostic) => diagnostic,
        KernelLoweringError::Construction(_) | KernelLoweringError::UnsupportedRegion { .. } => {
            KernelDiagnostic::BodyRefinement
        }
    })?;
    builder.into_data()
}

fn plan(schedule: &ScheduledRegion) -> Result<CanonicalPlan<'_>, KernelDiagnostic> {
    // The fixed-vector map is refused before any body is derived: this
    // profile's canonical bodies are scalar, and the lane-shaped values and
    // memory operations a packet body needs are a separate accepted boundary.
    // Refusing here — rather than deriving the scalar body the binding does
    // not state — is what keeps the accepted carrier non-executable instead of
    // silently scalarized, and it covers `derive_canonical` and the
    // refinement gate as well as direct lowering.
    if matches!(
        schedule.schedule.binding,
        ExecutionBinding::FixedVectorMap { .. }
    ) {
        return Err(KernelDiagnostic::UnloweredExecutionBinding);
    }
    // The partitioned-copy region is refused before any body is derived, for
    // the reason the vector binding is: this profile's canonical bodies
    // evaluate a scalar program the copy does not carry, and the copy's
    // guarded-store body and bit-preserving evidence are a separate accepted
    // boundary (`lower-the-partitioned-copy-region-through-kernel-ir`).
    // Refusing here covers `derive_canonical` and the refinement gate as well
    // as direct lowering; silence is not an option because the dispatch is a
    // total match.
    let RegionProgram::Numerical { scalar, numerical } = &schedule.index.program else {
        return Err(KernelDiagnostic::UnloweredRegionProgram);
    };
    let (reads, write) = boundary_accesses(schedule)?;
    let read = reads.first().ok_or(KernelDiagnostic::ScheduleAccessCount)?;
    // The contributors *one invocation* combines. For a partial pass that is
    // its own partition's share, not the whole reduction's sequence, which is
    // exactly the difference the split exists to create.
    let contributors = match &schedule.schedule.reduction {
        ReductionTopology::None | ReductionTopology::LiveContraction { .. } => 0,
        ReductionTopology::Serial { axes, .. }
        | ReductionTopology::MultiPass {
            pass: ReductionPass::Final,
            axes,
            ..
        } => contributor_count(axes, &read.map).map_err(|_| KernelDiagnostic::ContributorDomain)?,
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage,
            ..
        } => exact_partition(*coverage)?.contributors_per_partition,
        // The contracted index space, which the topology states because no
        // single operand's map determines it.
        ReductionTopology::Contraction {
            contracted_shape, ..
        } => crate::schedule::element_count(contracted_shape)
            .map_err(|_| KernelDiagnostic::ElementCountOverflow)?,
        // What one participant folds in the producing phase, which the split
        // states directly for the reason a partial pass's does: counting the
        // access's contributors here would count the whole sequence.
        ReductionTopology::CooperativeWorkgroup { coverage, .. } => {
            exact_partition(*coverage)?.contributors_per_partition
        }
        ReductionTopology::CooperativeContraction {
            contracted_tile, ..
        } => crate::schedule::element_count(contracted_tile)
            .map_err(|_| KernelDiagnostic::ElementCountOverflow)?,
    };
    // The strict-affine decode addresses its three role-scoped components by the
    // invocation index directly, so it consults no coordinate map.
    //
    // The live axis is resolved once from the region's unique source marker and
    // threaded into every read's addressing, because the fieldless consumer
    // carries no axis of its own: its stride and loop bound are the checked
    // containing region's, exactly the contextual interpretation the accepted
    // fieldless-marker surface states.
    let live = live_source_axis(schedule);
    // Which reads carry addresses rather than values, from the one derivation
    // `verify_signature` also reads. An address operand is classified before any
    // of them is addressed, because a gather's own form has to name the position
    // of its operand and the operand's own form has to say it is not a leaf.
    let address_reads = gather_address_reads(reads);
    let addressing = if matches!(scalar, ScalarProgram::StrictAffineU4Dequantize { .. }) {
        vec![ReadAddressing::Identity; reads.len()]
    } else {
        reads
            .iter()
            .enumerate()
            .map(|(position, read)| {
                if address_reads.get(position).copied().unwrap_or(false) {
                    Ok(ReadAddressing::GatherAddress)
                } else {
                    addressing(read, reads, &schedule.schedule.reduction, live)
                }
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    // A live loop and a data-dependent read do not compose in this profile. The
    // live body loads every read at one row-major offset it computes itself, so
    // a gather reaching it would be addressed by the loop's coordinate instead
    // of by its own relation and would return a wrong element silently. The
    // schedule verifier already refuses the pair — its addressing-regime rule
    // admits an all-static region or an all-`LiveRowMajor` one and nothing
    // between — so this is the fail-closed backstop for a body derived outside
    // that gate rather than a reachable refusal.
    if addressing
        .iter()
        .any(|addressing| matches!(addressing, ReadAddressing::Gather(_)))
        && addressing
            .iter()
            .any(|addressing| matches!(addressing, ReadAddressing::LiveRowMajor { .. }))
    {
        return Err(KernelDiagnostic::BodyRefinement);
    }
    Ok(CanonicalPlan {
        scalar,
        reads,
        write,
        numerical: *numerical,
        write_tensor: write.tensor,
        read_elements: reads
            .iter()
            .map(|read| access_elements(read, schedule))
            .collect::<Result<Vec<_>, _>>()?,
        write_elements: access_elements(write, schedule)?,
        work_items: schedule.schedule.work_items,
        write_bounds: write.bounds,
        ownership: schedule.schedule.output_owner,
        contributors,
        addressing,
        cooperative: cooperative_plan(schedule)?,
        contraction: cooperative_contraction_plan(schedule)?,
        live_extents: live_input_extents(schedule),
        live_contraction: match &schedule.schedule.reduction {
            ReductionTopology::LiveContraction {
                live_access,
                live_axis,
                ..
            } => Some((*live_access, *live_axis)),
            _ => None,
        },
    })
}

/// Resolves the cooperative shape this lowering emits, or refuses by name.
///
/// Returns `None` for every region that is not cooperative — the canonical
/// absence — and `Err` for a cooperative tile outside the lowered profile. Each
/// narrowing below is a shape the schedule verifier admits and this emission has
/// no body for, so refusing keeps "representable" and "lowered" apart instead of
/// lowering the nearest thing.
fn cooperative_plan(
    schedule: &ScheduledRegion,
) -> Result<Option<CooperativePlan>, KernelDiagnostic> {
    let ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } =
        &schedule.schedule.reduction
    else {
        return Ok(None);
    };
    let partition = exact_partition(*coverage)?;
    let shape = KernelDiagnostic::CooperativeLoweringShape;
    // One allocation and two phases: the bounded profile's tile stages one
    // partial per participant and reads the set back once, however many rounds
    // it runs. The points are resolved below from the edges they discharge
    // rather than destructured positionally, because *which* point orders which
    // obligation is what the body has to get right and the schedule already
    // proved exactly one point answers each.
    let ([staging], [produce, consume]) = (tile.staging.as_slice(), tile.phases.as_slice()) else {
        return Err(shape);
    };
    let ([write], [], [], [read]) = (
        produce.writes.as_slice(),
        produce.reads.as_slice(),
        consume.writes.as_slice(),
        consume.reads.as_slice(),
    ) else {
        return Err(shape);
    };
    if write.staging != staging.id || read.staging != staging.id {
        return Err(shape);
    }
    let participants = tile.coordinates.participants.participants().ok_or(shape)?;
    // A rank-one participant space, so the participant coordinate *is* the
    // linear local index this body reads. A tile arranged in two or three
    // dimensions is a tile this emission has no body for — it would need to read
    // a per-dimension position and reconstruct the address sum — and it is
    // refused here by the destructuring rather than lowered against a coordinate
    // the emitted kernel never computes.
    let ([produce_stride], [consume_stride]) = (write.span.strides(), read.span.strides()) else {
        return Err(shape);
    };
    // One slot per participant on the producing side, and the whole staged set
    // read by the committing participant on the consuming side. Any other span
    // needs an addressing form this emission does not have.
    if write.span.count != 1
        || *produce_stride == 0
        || *consume_stride != 0
        || read.span.count != participants
    {
        return Err(shape);
    }
    // `IndexLessThan` selects a prefix, so a commit range that does not start at
    // participant zero has no governed predicate and its store would be
    // undominated by schedule-derived evidence.
    if tile.commit.first != 0 {
        return Err(shape);
    }
    // The two obligations, each resolved to the one point that discharges it.
    // A tile of this shape derives exactly one visibility edge and — when its
    // phases repeat — exactly one anti-dependency, so anything else is a tile
    // this emission has no body for rather than a defect the schedule missed.
    let edges = tile.visibility_edges();
    let [edge] = edges.as_slice() else {
        return Err(shape);
    };
    let barrier = sole_discharging_barrier(&tile.discharging_points(*edge)).ok_or(shape)?;
    let anti = tile.anti_dependency_edges();
    let round_barrier = match anti.as_slice() {
        [] => None,
        [edge] => {
            Some(sole_discharging_barrier(&tile.anti_discharging_points(*edge)).ok_or(shape)?)
        }
        _ => return Err(shape),
    };
    // A tile whose phases repeat has a rewrite, and a tile whose phases run once
    // has none. The equality is checked rather than assumed so a tile that
    // somehow carried one without the other is refused instead of lowering a
    // round loop with no boundary or a boundary with no round.
    if (tile.rounds > 1) != round_barrier.is_some() {
        return Err(shape);
    }
    let contributors_per_round = participants
        .checked_mul(partition.contributors_per_partition)
        .ok_or(shape)?;
    Ok(Some(CooperativePlan {
        participants,
        rounds: tile.rounds,
        contributors_per_partition: partition.contributors_per_partition,
        contributors_per_round,
        commit_count: tile.commit.count,
        staging: staging.id,
        slots: staging.slots,
        produce_phase: produce.id,
        consume_phase: consume.id,
        produce_stride: *produce_stride,
        produce_offset: write.span.offset,
        consume_offset: read.span.offset,
        barrier,
        round_barrier,
    }))
}

fn cooperative_contraction_plan(
    schedule: &ScheduledRegion,
) -> Result<Option<CooperativeContractionPlan>, KernelDiagnostic> {
    let ReductionTopology::CooperativeContraction {
        tile,
        contracted_shape,
        contracted_tile,
        ..
    } = &schedule.schedule.reduction
    else {
        return Ok(None);
    };
    let shape = KernelDiagnostic::CooperativeLoweringShape;
    let ExecutionBinding::BlockedWorkgroup { block, workgroups } = &schedule.schedule.binding
    else {
        return Err(shape);
    };
    let output_shape = &schedule.index.iteration_shape;
    // The participants occupy the output's trailing two axes whatever its rank,
    // so a rank-two output is the batched output with an empty prefix and reaches
    // this same derivation. The three ranks are required to agree because the
    // blocked bijection is stated per axis, and `contracted_tile` stays rank one
    // because the round loop walks a single contracted coordinate.
    let Some(prefix) = output_shape.rank().checked_sub(2) else {
        return Err(shape);
    };
    if block.rank() != output_shape.rank()
        || workgroups.rank() != output_shape.rank()
        || contracted_shape.rank() != 1
        || contracted_tile.rank() != 1
    {
        return Err(shape);
    }
    // **The load-bearing batch clause.** A leading block extent above one would
    // have one workgroup span several batch coordinates with no participant
    // dimension distinguishing them, so the tile's staged operand rows would hold
    // elements of two different batches. It is the same relation
    // `blocked_batch_prefix` enforces at the schedule layer, restated here because
    // this emission derives each batch coordinate *from the workgroup index* and
    // that derivation is sound only under it.
    if block.extents()[..prefix]
        .iter()
        .any(|extent| extent.get() != 1)
    {
        return Err(shape);
    }
    let block_m = block.extents()[prefix].get();
    let block_n = block.extents()[prefix + 1].get();
    let tile_k = contracted_tile.extents()[0].get();
    if block_m == 0 || block_n == 0 || tile_k == 0 || block_m != block_n || block_n != tile_k {
        return Err(shape);
    }
    // With a leading block extent of one the per-axis quotient is the output
    // extent itself, so the two must agree. Checked rather than assumed: this
    // emission reads the batch coordinate off the *workgroup* index and applies it
    // to an *output* stride, and nothing else here would notice the two spaces
    // disagreeing.
    let batch_workgroups: Vec<u64> = workgroups.extents()[..prefix]
        .iter()
        .map(|extent| extent.get())
        .collect();
    let batch_extents: Vec<u64> = output_shape.extents()[..prefix]
        .iter()
        .map(|extent| extent.get())
        .collect();
    if batch_workgroups != batch_extents {
        return Err(shape);
    }
    // Suffix products over the *whole* output shape, so the last batch axis
    // carries `output_m * output_n` and the store's linear position is the
    // row-major one its `LinearIdentity` write map states.
    let output_extents: Vec<u64> = output_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let output_strides = suffix_products(&output_extents);
    let batch_output_strides = output_strides[..prefix].to_vec();
    if batch_output_strides.contains(&0) {
        return Err(KernelDiagnostic::ElementCountOverflow);
    }
    let ([left, right], [produce, consume]) = (tile.staging.as_slice(), tile.phases.as_slice())
    else {
        return Err(shape);
    };
    let ([left_write, right_write], [], [], [left_read, right_read]) = (
        produce.writes.as_slice(),
        produce.reads.as_slice(),
        consume.writes.as_slice(),
        consume.reads.as_slice(),
    ) else {
        return Err(shape);
    };
    if left_write.staging != left.id
        || right_write.staging != right.id
        || left_read.staging != left.id
        || right_read.staging != right.id
    {
        return Err(shape);
    }
    let edges = tile.visibility_edges();
    if edges.is_empty() {
        return Err(shape);
    }
    let mut barrier = None;
    for edge in &edges {
        let point = sole_discharging_barrier(&tile.discharging_points(*edge)).ok_or(shape)?;
        match &barrier {
            None => barrier = Some(point),
            Some(existing) if existing.point == point.point => {}
            Some(_) => return Err(shape),
        }
    }
    let barrier = barrier.ok_or(shape)?;
    // Resolved by the same rule as the visibility edges above, and for the same
    // reason: a two-allocation tile that repeats carries one anti-dependency per
    // staging, and neither `SynchronizationPoint::discharges_anti` nor the edge
    // itself reads which allocation is rewritten, so one round boundary orders
    // both. Matching on a single edge instead — as this did — refused every
    // multi-round operand tile as an unlowerable shape, which is exactly the
    // body the tiled contraction is: two staged tiles reloaded every round.
    // Two boundaries would be a genuinely different body and are still refused.
    let anti = tile.anti_dependency_edges();
    let mut round_barrier = None;
    for edge in &anti {
        let point = sole_discharging_barrier(&tile.anti_discharging_points(*edge)).ok_or(shape)?;
        match &round_barrier {
            None => round_barrier = Some(point),
            Some(existing) if existing.point == point.point => {}
            Some(_) => return Err(shape),
        }
    }
    if (tile.rounds > 1) != round_barrier.is_some() {
        return Err(shape);
    }
    Ok(Some(CooperativeContractionPlan {
        block: block_m,
        batch_workgroups,
        batch_output_strides,
        workgroups_m: workgroups.extents()[prefix].get(),
        workgroups_n: workgroups.extents()[prefix + 1].get(),
        output_m: output_extents[prefix],
        output_n: output_extents[prefix + 1],
        rounds: tile.rounds,
        predicated: matches!(schedule.schedule.tail, TailPolicy::Predicated),
        left_staging: left.id,
        right_staging: right.id,
        left_slots: left.slots,
        right_slots: right.slots,
        produce_phase: produce.id,
        consume_phase: consume.id,
        barrier,
        round_barrier,
    }))
}

/// Returns the exact split a lowering may fold, or refuses a padded one.
///
/// Identity-padded coverage is representable and intrinsically verified; this
/// profile has no emission that injects the stated identity, so a padded
/// region is refused rather than folded as if every capacity slot were real.
fn exact_partition(
    coverage: ContributorCoverage,
) -> Result<crate::schedule::ContributorPartition, KernelDiagnostic> {
    match coverage {
        ContributorCoverage::Exact(partition) => Ok(partition),
        ContributorCoverage::IdentityPadded { .. } => {
            Err(KernelDiagnostic::PaddedContributorCoverage)
        }
    }
}

/// Resolves the cooperative lowering shape without exposing its private plan.
///
/// This test-only projection lets the kernel module's fixture tests drive the
/// lowering's defensive refusals even where the schedule verifier rejects the
/// malformed subject first.
#[cfg(test)]
pub(super) fn cooperative_plan_shape_check(
    schedule: &ScheduledRegion,
) -> Result<(), KernelDiagnostic> {
    cooperative_plan(schedule).map(|_| ())
}

/// Restates the one point discharging an obligation in the KIR barrier spelling.
///
/// `None` when the obligation has any number of discharging points other than
/// one, which a verified region cannot have — the check is here because this
/// function turns a schedule fact into an emitted operation, and taking the first
/// of an unexpected set would emit a barrier for an obligation the schedule never
/// proved this point orders.
fn sole_discharging_barrier(
    points: &[&crate::schedule::SynchronizationPoint],
) -> Option<BarrierSpec> {
    let [point] = points else {
        return None;
    };
    barrier_spelling(point)
}

/// Restates one schedule synchronization point in the KIR barrier spelling.
///
/// The inverse of `verify::barrier_subject`, and deliberately partial: a subject
/// this vocabulary cannot spell has no canonical barrier, so the region is
/// refused rather than emitted with the nearest available construct. The
/// verifier projects the result back and requires equality, so a mistake here is
/// a build failure of the lowering itself rather than a wrong emission.
fn barrier_spelling(point: &crate::schedule::SynchronizationPoint) -> Option<BarrierSpec> {
    let subject = point.subject;
    if subject.kind != crate::schedule::SynchronizationKind::ControlBarrier {
        return None;
    }
    let execution_scope = match subject.execution_scope {
        crate::schedule::SynchronizationScope::Subgroup => ExecutionScope::Subgroup,
        crate::schedule::SynchronizationScope::Workgroup => ExecutionScope::Workgroup,
        crate::schedule::SynchronizationScope::Device => return None,
    };
    let memory_scope = match subject.visibility_scope {
        crate::schedule::SynchronizationScope::Workgroup => MemoryScope::Workgroup,
        crate::schedule::SynchronizationScope::Device => MemoryScope::Device,
        crate::schedule::SynchronizationScope::Subgroup => return None,
    };
    let ordering = match subject.ordering {
        crate::schedule::MemoryOrdering::AcquireRelease => BarrierOrdering::AcquireRelease,
        crate::schedule::MemoryOrdering::Relaxed
        | crate::schedule::MemoryOrdering::SequentiallyConsistent => return None,
    };
    // Ascending governed order, which is what makes the emitted flag expression
    // independent of how the fence was stated.
    let mut fenced_spaces = Vec::new();
    if subject.fenced_spaces.device {
        fenced_spaces.push(AddressSpace::Device);
    }
    if subject.fenced_spaces.workgroup {
        fenced_spaces.push(AddressSpace::Workgroup);
    }
    Some(BarrierSpec {
        point: point.id,
        execution_scope,
        memory_scope,
        fenced_spaces,
        ordering,
    })
}

/// Resolves how the read access computes its element offset.
///
/// `live` is the containing region's unique source-marker axis, resolved once
/// by the caller. The fieldless consumer is interpreted through it and never
/// through a default: a consumer with no marker to consume is a region the
/// intrinsic verifier refuses, so the refusal here is the fail-closed backstop
/// for a body derived outside that gate rather than a reachable lowering.
///
/// `reads` is the region's whole read run, which only the gather arm consults:
/// a data-dependent read names its address operand by region-local ordinal, so
/// resolving it needs the list the ordinal indexes. Every other arm answers
/// from `read` alone.
fn addressing(
    read: &Access,
    reads: &[Access],
    reduction: &ReductionTopology,
    live: Option<crate::shape::Axis>,
) -> Result<ReadAddressing, KernelDiagnostic> {
    match &read.map {
        LogicalAccess::LinearIdentity => Ok(ReadAddressing::Identity),
        LogicalAccess::LiveRowMajorSource { inner_axis } => Ok(ReadAddressing::LiveRowMajor {
            inner_axis: *inner_axis,
        }),
        LogicalAccess::LiveRowMajor => match live {
            Some(inner_axis) => Ok(ReadAddressing::LiveRowMajor { inner_axis }),
            None => Err(KernelDiagnostic::BodyRefinement),
        },
        LogicalAccess::ScalarBroadcast | LogicalAccess::PackedU4LsbZeroTail { .. } => {
            Err(KernelDiagnostic::BodyRefinement)
        }
        LogicalAccess::ReductionContributor {
            input_shape, axes, ..
        } => {
            let reduced: Vec<usize> = axes
                .iter()
                .map(|axis| usize::try_from(axis.get()).unwrap_or(usize::MAX))
                .collect();
            if reduced.iter().any(|axis| *axis >= input_shape.rank()) {
                return Err(KernelDiagnostic::ContributorDomain);
            }
            let terms = linearize(input_shape, &reduced);
            // Only the partial pass splits its invocation index. A final pass
            // runs one invocation per output, exactly as a serial reduction
            // does, so it uses the unsplit form.
            match reduction {
                ReductionTopology::MultiPass {
                    pass: ReductionPass::Partial,
                    coverage,
                    ..
                } => {
                    let partition = exact_partition(*coverage)?;
                    Ok(ReadAddressing::Partitioned {
                        terms,
                        partitions: partition.partitions,
                        contributors_per_partition: partition.contributors_per_partition,
                    })
                }
                // A cooperative tile splits its invocation index exactly as a
                // partial pass does, but the split is emitted *once* at the top
                // level rather than inside each offset: the committing
                // participant's owning store needs the same output coordinate
                // the loads used, and a value defined inside a guarded block
                // could not reach it. The shared linearization therefore
                // receives the already-split roots, which is why this arm is the
                // unsplit form and not `Partitioned`.
                ReductionTopology::CooperativeWorkgroup { .. }
                | ReductionTopology::None
                | ReductionTopology::Serial { .. }
                | ReductionTopology::Contraction { .. }
                | ReductionTopology::CooperativeContraction { .. }
                | ReductionTopology::LiveContraction { .. }
                | ReductionTopology::MultiPass { .. } => Ok(ReadAddressing::Linearized(terms)),
            }
        }
        LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
            ..
        } => {
            // The blocked topology addresses through named block coordinates
            // rather than through a linear root, so it resolves to its own form
            // before the linearization below — which would otherwise produce
            // terms whose roots this body never computes.
            if matches!(reduction, ReductionTopology::CooperativeContraction { .. }) {
                return Ok(ReadAddressing::BlockedContraction(
                    blocked_contraction_terms(operand_shape, output_shape, sources)?,
                ));
            }
            let terms = linearize_contraction_operand(
                operand_shape,
                output_shape,
                contracted_shape,
                sources,
            )?;
            if matches!(reduction, ReductionTopology::LiveContraction { .. }) {
                Ok(ReadAddressing::LiveContracted { free: terms })
            } else {
                Ok(ReadAddressing::Linearized(terms))
            }
        }
        // Both structural relations linearize identically, because both are
        // written in the same per-operand-axis decode and differ only in the
        // admission rule the *schedule verifier* already discharged. Sharing the
        // emission here is not collapsing the two concepts: a bijection and a
        // replication reach this point already proven, and what they have in
        // common at this layer is exactly the arithmetic.
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        }
        | LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => Ok(ReadAddressing::Linearized(linearize_axis_decodes(
            operand_shape,
            result_shape,
            axes,
        )?)),
        // The parametric carrier is a different coordinate language — sourced
        // mapping extents, not concrete AxisDecode windows. Binding those
        // extents here would specialize the one-artifact relation. Lowering
        // refuses rather than inventing a second language or selecting a
        // concrete neighbour.
        LogicalAccess::ParametricBroadcast { .. } => Err(KernelDiagnostic::BodyRefinement),
        // The data-dependent read. Its address is a sum of two halves: the
        // source coordinates the result domain carries directly, and the one
        // coordinate the loaded U32 supplies on the gathered axis.
        //
        // The relation is checked against its own derivations before an address
        // is built from it, rather than trusted. `gather_result_shape` and
        // `gather_index_read_map` are the schedule layer's single authorities
        // for the result domain and the address relation, and the schedule
        // verifier compared the stated members against both before this region
        // could be verified. Re-deriving here is what keeps that comparison
        // from being the only thing between a self-inconsistent relation and an
        // address computed off the wrong shape — the failure that stays in
        // bounds and returns a wrong element.
        LogicalAccess::GatherSource {
            source_shape,
            result_shape,
            axis,
            index_access,
            index_shape,
        } => {
            let derived = crate::semantic::gather_result_shape(*axis, source_shape, index_shape)
                .map_err(|_| KernelDiagnostic::BodyRefinement)?;
            if derived.1 != *result_shape {
                return Err(KernelDiagnostic::BodyRefinement);
            }
            let index_read = usize::try_from(index_access.get())
                .map_err(|_| KernelDiagnostic::BodyRefinement)?;
            let operand = reads
                .get(index_read)
                .ok_or(KernelDiagnostic::BodyRefinement)?;
            let expected = gather_index_read_map(source_shape, *axis, index_shape)
                .ok_or(KernelDiagnostic::BodyRefinement)?;
            if operand.map != expected {
                return Err(KernelDiagnostic::BodyRefinement);
            }
            let (direct, gathered_stride) = gather_direct_terms(source_shape, result_shape, *axis)?;
            Ok(ReadAddressing::Gather(Box::new(GatherAddressing {
                direct,
                gathered_stride,
                index_read,
                index: gather_address_addressing(&expected)?,
            })))
        }
        // Unreachable through `plan`, which refuses the copy region program
        // before any read is addressed; refused by name so a reachable path
        // added later names the missing carrier rather than a body defect.
        LogicalAccess::PartitionedCopySource => Err(KernelDiagnostic::UnloweredRegionProgram),
    }
}

/// Resolves how a gather's address-only index operand computes its own offset.
///
/// Deliberately narrower than [`addressing`], and deliberately not a call into
/// it: it admits exactly the three relations [`gather_index_read_map`] derives
/// and refuses every other, so an address operand can never be addressed by a
/// relation the schedule layer's authority does not produce.
///
/// [`LogicalAccess::ScalarBroadcast`] is admitted here and refused there, which
/// is the same asymmetry the schedule verifier keeps: one address read by every
/// invocation is a coordinate, and a rank-zero *value* leaf belongs to the
/// decode program instead. Its offset is element zero, which is the empty term
/// list rather than [`ReadAddressing::Identity`] — the invocation index would
/// address a different element per invocation of a tensor that holds one.
///
/// Written out arm by arm with no wildcard, so a widened [`LogicalAccess`] is a
/// build error here rather than a relation silently inheriting the refusal.
fn gather_address_addressing(map: &LogicalAccess) -> Result<ReadAddressing, KernelDiagnostic> {
    match map {
        LogicalAccess::LinearIdentity => Ok(ReadAddressing::Identity),
        LogicalAccess::ScalarBroadcast => Ok(ReadAddressing::Linearized(Vec::new())),
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => Ok(ReadAddressing::Linearized(linearize_axis_decodes(
            operand_shape,
            result_shape,
            axes,
        )?)),
        LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. }
        | LogicalAccess::ReindexBijection { .. }
        | LogicalAccess::ParametricBroadcast { .. }
        | LogicalAccess::LiveRowMajorSource { .. }
        | LogicalAccess::LiveRowMajor
        | LogicalAccess::PartitionedCopySource
        | LogicalAccess::GatherSource { .. } => Err(KernelDiagnostic::BodyRefinement),
    }
}

/// Builds the direct half of one data-dependent read's address.
///
/// Every source axis other than the gathered one takes its coordinate from the
/// result axis carrying it. The result domain is
/// `source[..axis] ++ index ++ source[axis + 1..]`, so a source axis before the
/// gathered one keeps its position and a source axis after it sits
/// `index_rank - 1` positions further along. The gathered axis receives no
/// result coordinate at all — that is precisely what the loaded U32 supplies —
/// so it contributes no term and its stride is returned instead, for the caller
/// to scale the loaded coordinate by.
///
/// The conventions are [`linearize_axis_decodes`]'s, shared for the reason the
/// two structural relations share theirs: a coordinate that is constantly zero
/// or a stride that is zero contributes nothing and is dropped, and the wrap is
/// omitted exactly where the decode names the leading window of the result's
/// linear coordinate, so the quotient is already below the modulus.
///
/// Each axis's result extent is compared against the source extent it claims to
/// carry. That comparison is what separates this from an arithmetic that merely
/// type-checks: a relation whose result domain does not match the composition
/// would otherwise yield an address that stays inside the source buffer and
/// names a different element.
fn gather_direct_terms(
    source_shape: &Shape,
    result_shape: &Shape,
    axis: crate::shape::Axis,
) -> Result<(Vec<OffsetTerm>, u64), KernelDiagnostic> {
    let source_extents: Vec<u64> = source_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let gathered = usize::try_from(axis.get())
        .ok()
        .filter(|position| *position < source_extents.len())
        .ok_or(KernelDiagnostic::BodyRefinement)?;
    let result_extents: Vec<u64> = result_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    // `result_rank = source_rank - 1 + index_rank`, solved for the index run.
    let index_rank = result_extents
        .len()
        .checked_add(1)
        .and_then(|rank| rank.checked_sub(source_extents.len()))
        .ok_or(KernelDiagnostic::BodyRefinement)?;
    let source_strides = suffix_products(&source_extents);
    let result_strides = suffix_products(&result_extents);
    let result_elements =
        element_count(result_shape).map_err(|_| KernelDiagnostic::ElementCountOverflow)?;

    let mut terms = Vec::with_capacity(source_extents.len());
    for (position, extent) in source_extents.iter().copied().enumerate() {
        if position == gathered {
            continue;
        }
        let carrier = if position < gathered {
            position
        } else {
            position
                .checked_add(index_rank)
                .and_then(|shifted| shifted.checked_sub(1))
                .ok_or(KernelDiagnostic::BodyRefinement)?
        };
        let (Some(divisor), Some(result_extent)) = (
            result_strides.get(carrier).copied(),
            result_extents.get(carrier).copied(),
        ) else {
            return Err(KernelDiagnostic::BodyRefinement);
        };
        if result_extent != extent {
            return Err(KernelDiagnostic::BodyRefinement);
        }
        let stride = source_strides[position];
        if extent == 1 || divisor == 0 || stride == 0 {
            continue;
        }
        let leading = divisor
            .checked_mul(result_extent)
            .is_some_and(|window| window == result_elements);
        terms.push(OffsetTerm {
            root: OffsetRoot::Output,
            divisor,
            modulus: (!leading).then_some(result_extent),
            // No source axis of a gather mirrors: the relation states a
            // reversal of no axis, and the gathered coordinate is a loaded
            // value rather than a decode a mirror could apply to.
            mirror: None,
            stride,
        });
    }
    Ok((terms, source_strides[gathered]))
}

/// Builds the row-major linearization terms of one structural access.
///
/// Each operand axis contributes one term: its coordinate is decoded from the
/// linear *result* index the global invocation carries, then scaled by the
/// operand's own row-major stride. The wrap is omitted exactly where it is
/// provably redundant — when the decode's window is the leading one, so the
/// quotient is already below the modulus — which is the same convention the
/// reduction and contraction linearizations follow.
///
/// A term whose operand extent is one is dropped: that coordinate is constantly
/// zero, so it contributes nothing to the offset. A replicated result axis
/// contributes no term at all, which is precisely what makes the read invariant
/// in it.
fn linearize_axis_decodes(
    operand_shape: &Shape,
    result_shape: &Shape,
    axes: &[crate::schedule::AxisDecode],
) -> Result<Vec<OffsetTerm>, KernelDiagnostic> {
    let operand_extents: Vec<u64> = operand_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    if axes.len() != operand_extents.len() {
        return Err(KernelDiagnostic::ContributorDomain);
    }
    let operand_strides = suffix_products(&operand_extents);
    let result_elements =
        element_count(result_shape).map_err(|_| KernelDiagnostic::ElementCountOverflow)?;

    let mut terms = Vec::with_capacity(axes.len());
    for (axis, decode) in axes.iter().enumerate() {
        let stride = operand_strides[axis];
        if operand_extents[axis] == 1 || decode.divisor == 0 || decode.modulus == 0 || stride == 0 {
            continue;
        }
        // Redundant exactly when the decode names the most significant window of
        // the result's linear coordinate: the quotient is then already below the
        // modulus, so the remainder is the identity.
        let leading = decode
            .divisor
            .checked_mul(decode.modulus)
            .is_some_and(|window| window == result_elements);
        terms.push(OffsetTerm {
            root: OffsetRoot::Output,
            divisor: decode.divisor,
            modulus: (!leading).then_some(decode.modulus),
            mirror: decode.mirrored.then_some(decode.modulus),
            stride,
        });
    }
    Ok(terms)
}

/// Builds the row-major linearization terms of one contraction operand access.
///
/// Each operand axis contributes one term. The axis's coordinate is decoded from
/// whichever linear index the schedule verifier proved it names — the invocation
/// index for an output coordinate, the loop induction variable for a contracted
/// one — using the suffix products of *that* space, and is scaled by the
/// operand's own row-major stride. The leading position of a space needs no
/// wrap, because the linear index is already below the product of every extent
/// in it. A term whose extent is one, or whose divisor, modulus, or stride is
/// zero, is dropped: the coordinate is then constantly zero, or the domain is
/// empty and the guarded block never executes.
fn linearize_contraction_operand(
    operand_shape: &Shape,
    output_shape: &Shape,
    contracted_shape: &Shape,
    sources: &[crate::schedule::ContractionAxisSource],
) -> Result<Vec<OffsetTerm>, KernelDiagnostic> {
    let extents_of =
        |shape: &Shape| -> Vec<u64> { shape.extents().iter().map(|extent| extent.get()).collect() };
    let operand_extents = extents_of(operand_shape);
    let operand_strides = suffix_products(&operand_extents);
    let output_extents = extents_of(output_shape);
    let output_suffix = suffix_products(&output_extents);
    let contracted_extents = extents_of(contracted_shape);
    let contracted_suffix = suffix_products(&contracted_extents);
    if sources.len() != operand_extents.len() {
        return Err(KernelDiagnostic::ContributorDomain);
    }

    let mut terms = Vec::with_capacity(sources.len());
    for (axis, source) in sources.iter().enumerate() {
        let (root, position, sub_extents, sub_suffix) = match source {
            crate::schedule::ContractionAxisSource::Output { position } => (
                OffsetRoot::Output,
                *position,
                &output_extents,
                &output_suffix,
            ),
            crate::schedule::ContractionAxisSource::Contracted { position } => (
                OffsetRoot::Contributor,
                *position,
                &contracted_extents,
                &contracted_suffix,
            ),
        };
        let position =
            usize::try_from(position).map_err(|_| KernelDiagnostic::ContributorDomain)?;
        let (Some(divisor), Some(sub_extent)) =
            (sub_suffix.get(position), sub_extents.get(position))
        else {
            return Err(KernelDiagnostic::ContributorDomain);
        };
        let modulus = (position > 0).then_some(*sub_extent);
        let stride = operand_strides[axis];
        if operand_extents[axis] == 1 || *divisor == 0 || modulus == Some(0) || stride == 0 {
            continue;
        }
        terms.push(OffsetTerm {
            root,
            divisor: *divisor,
            modulus,
            // No contraction operand axis mirrors: the family's index structure
            // names coordinates, never reflections of them.
            mirror: None,
            stride,
        });
    }
    Ok(terms)
}

/// Builds the address terms of one blocked cooperative-contraction operand.
///
/// Each operand axis contributes one `stride * coordinate` term: the coordinate
/// is whichever of the block's the declared source names, and the stride is the
/// operand's own row-major stride for that axis. Nothing is decoded, because the
/// emitted body holds every one of those coordinates already — the row, the
/// column, the contracted index, and one value per leading batch axis.
///
/// **This is the function that makes `[K, M]` a different kernel from `[M, K]`.**
/// The two declare the same coordinates in the opposite axis order, so they
/// differ only in which term carries the non-unit stride — and an emission that
/// hardcoded one of them read the wrong element for the other with no refusal
/// anywhere.
///
/// A term whose operand extent is one, or whose stride is zero, is dropped: the
/// coordinate is then constantly zero wherever the load actually happens, which
/// is the same convention [`linearize_contraction_operand`] follows. It stays
/// sound under a predicated tail, where the row or column coordinate may exceed
/// its extent, because such an invocation's load is guarded off and performs no
/// access.
///
/// # Errors
///
/// Returns [`KernelDiagnostic::CooperativeLoweringShape`] for a declared layout
/// this emission has no body for — an operand axis reading an output position
/// past the participants' trailing pair, or a contracted coordinate other than
/// the single one the round loop walks. Both are refused by *layout*, not by
/// shape: they name a coordinate no value in this body computes. The schedule
/// layer's `verify_blocked_operand_roles` and `cooperative_contraction_plan`
/// already exclude them, so this is the fail-closed backstop for a body derived
/// outside those gates rather than a reachable refusal.
fn blocked_contraction_terms(
    operand_shape: &Shape,
    output_shape: &Shape,
    sources: &[crate::schedule::ContractionAxisSource],
) -> Result<Vec<BlockedTerm>, KernelDiagnostic> {
    let operand_extents: Vec<u64> = operand_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    if sources.len() != operand_extents.len() {
        return Err(KernelDiagnostic::ContributorDomain);
    }
    let operand_strides = suffix_products(&operand_extents);
    // The participants occupy the output's trailing two axes, which is the
    // relation `participant_space_matches_block` couples the block to.
    let Some(row) = output_shape.rank().checked_sub(2) else {
        return Err(KernelDiagnostic::CooperativeLoweringShape);
    };
    let column = row + 1;

    let mut terms = Vec::with_capacity(sources.len());
    for (axis, source) in sources.iter().enumerate() {
        let position = match source {
            crate::schedule::ContractionAxisSource::Output { position }
            | crate::schedule::ContractionAxisSource::Contracted { position } => {
                usize::try_from(*position).map_err(|_| KernelDiagnostic::ContributorDomain)?
            }
        };
        let coordinate = match source {
            crate::schedule::ContractionAxisSource::Output { .. } if position == row => {
                BlockedCoordinate::Row
            }
            crate::schedule::ContractionAxisSource::Output { .. } if position == column => {
                BlockedCoordinate::Column
            }
            // A leading batch coordinate. The body decodes one per axis from the
            // workgroup index, which is sound exactly because the block's leading
            // extents are one — one workgroup then covers one coordinate on each
            // batch axis, so the workgroup's position on that axis *is* the output
            // coordinate the operand reads.
            crate::schedule::ContractionAxisSource::Output { .. } if position < row => {
                BlockedCoordinate::Batch(position)
            }
            // An output coordinate past the trailing pair, which no output axis
            // has. No value in this body names one, so the layout is refused
            // rather than addressed by whichever coordinate sits nearest it.
            //
            // Kept a separate arm from the contracted refusal below, which shares
            // its diagnostic: the two name different missing values — an output
            // position outside the shape, against a second contracted induction
            // the round loop does not have — and merging them would leave one
            // comment standing for both.
            #[expect(
                clippy::match_same_arms,
                reason = "distinct unaddressable coordinates that share one diagnostic"
            )]
            crate::schedule::ContractionAxisSource::Output { .. } => {
                return Err(KernelDiagnostic::CooperativeLoweringShape);
            }
            crate::schedule::ContractionAxisSource::Contracted { .. } if position == 0 => {
                BlockedCoordinate::Contracted
            }
            // The round loop walks one contracted coordinate; a second would need
            // a second induction variable this body does not have.
            crate::schedule::ContractionAxisSource::Contracted { .. } => {
                return Err(KernelDiagnostic::CooperativeLoweringShape);
            }
        };
        let stride = operand_strides[axis];
        if operand_extents[axis] == 1 || stride == 0 {
            continue;
        }
        terms.push(BlockedTerm { coordinate, stride });
    }
    Ok(terms)
}

/// Builds the ordered row-major linearization terms of a contributor access.
///
/// Each input axis contributes one term. A kept axis extracts its coordinate
/// from the linear output index and a reduced axis from the linear contributor
/// index, each using the suffix products of its own sub-shape. A term whose
/// extent is one, or whose divisor, modulus, or stride is zero, is dropped: the
/// coordinate is then constantly zero, or the whole iteration domain is empty
/// and the guarded block never executes.
fn linearize(input_shape: &Shape, reduced: &[usize]) -> Vec<OffsetTerm> {
    let extents: Vec<u64> = input_shape
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    let strides = suffix_products(&extents);
    let kept: Vec<usize> = (0..extents.len())
        .filter(|axis| !reduced.contains(axis))
        .collect();
    let kept_extents: Vec<u64> = kept.iter().map(|axis| extents[*axis]).collect();
    let reduced_extents: Vec<u64> = reduced.iter().map(|axis| extents[*axis]).collect();
    let kept_suffix = suffix_products(&kept_extents);
    let reduced_suffix = suffix_products(&reduced_extents);

    let mut terms = Vec::new();
    for (axis, extent) in extents.iter().copied().enumerate() {
        let (root, position, sub_extents, sub_suffix) =
            if let Some(position) = reduced.iter().position(|reduced| *reduced == axis) {
                (
                    OffsetRoot::Contributor,
                    position,
                    &reduced_extents,
                    &reduced_suffix,
                )
            } else if let Some(position) = kept.iter().position(|kept| *kept == axis) {
                (OffsetRoot::Output, position, &kept_extents, &kept_suffix)
            } else {
                continue;
            };
        let divisor = sub_suffix[position];
        let modulus = (position > 0).then(|| sub_extents[position]);
        let stride = strides[axis];
        if extent == 1 || divisor == 0 || modulus == Some(0) || stride == 0 {
            continue;
        }
        terms.push(OffsetTerm {
            root,
            divisor,
            modulus,
            // No reduction contributor axis mirrors: a contributor family is a
            // sub-shape of its input, read in ascending order.
            mirror: None,
            stride,
        });
    }
    terms
}

/// Returns the product of every later extent, saturating an overflow to zero.
///
/// An overflowing suffix product can only occur when some extent is zero, which
/// makes the whole domain empty; zero then drops the affected term.
fn suffix_products(extents: &[u64]) -> Vec<u64> {
    let mut products = vec![1_u64; extents.len()];
    for index in (0..extents.len()).rev() {
        let next = products.get(index + 1).copied().unwrap_or(1);
        let extent = extents.get(index + 1).copied().unwrap_or(1);
        products[index] = next.checked_mul(extent).unwrap_or(0);
    }
    products
}

fn emit(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    if matches!(plan.scalar, ScalarProgram::StrictAffineU4Dequantize { .. }) {
        return emit_strict_affine_u4_dequantize(builder, plan, requirements);
    }
    if let Some(cooperative) = &plan.cooperative {
        return emit_cooperative(builder, plan, cooperative, requirements);
    }
    if let Some(contraction) = &plan.contraction {
        return emit_cooperative_contraction(builder, plan, contraction, requirements);
    }
    // One buffer per read, in access order. The component role stays `None`
    // rather than being copied from the access: these families read dense
    // values, and `verify_signature` compares the two, so copying it would make
    // that comparison agree with itself instead of checking anything.
    //
    // The element type is derived from the region's scalar program through the
    // same authority `verify_signature` reads, so a widened dtype cannot declare
    // one type here and be checked against another there.
    let element_type = region_element_type(plan.scalar);
    // A gather's address operand is the one read that is not a dense boundary
    // value, so it is the one buffer that is not declared at the region's
    // element type: it carries the exact-width U32 coordinates the body loads.
    // Read from `gather_address_reads`, which is also what `verify_signature`
    // checks these declarations against, so the two cannot drift into declaring
    // one type and verifying another.
    let address_reads = gather_address_reads(plan.reads);
    let mut read_buffers = Vec::with_capacity(plan.reads.len());
    for (position, (read, elements)) in plan.reads.iter().zip(&plan.read_elements).enumerate() {
        read_buffers.push(builder.declare_buffer(BufferParameter {
            tensor: read.tensor,
            component_role: None,
            element_type: if address_reads.get(position).copied().unwrap_or(false) {
                KernelType::U32
            } else {
                element_type
            },
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: *elements,
        })?);
    }
    let write_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write_tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Write,
        element_count: plan.write_elements,
    })?;
    builder.admit_builtin(Builtin::GlobalInvocationIndex)?;
    builder.numerical(plan.numerical)?;
    builder.requirements(requirements)?;

    let live = declare_plan_live_extents(builder, plan)?;
    let invocation = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
    let active = builder.compare(CompareOp::IndexLessThan, invocation, extent)?;
    builder.predicated(active, |builder| {
        if plan
            .addressing
            .iter()
            .any(|addressing| matches!(addressing, ReadAddressing::LiveRowMajor { .. }))
        {
            emit_live_row_major(
                builder,
                plan,
                &read_buffers,
                write_buffer,
                invocation,
                &live,
            )
        } else {
            emit_guarded(
                builder,
                plan,
                &read_buffers,
                write_buffer,
                invocation,
                &live,
            )
        }
    })?;
    Ok(())
}

fn declare_plan_live_extents(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
) -> Result<Vec<(InputExtentParameter, KernelValueId)>, KernelBuildError> {
    let mut declared = Vec::with_capacity(plan.live_extents.len());
    for &(access, axis) in &plan.live_extents {
        let id = builder.declare_input_extent(InputExtentParameter { access, axis })?;
        let value = builder.input_extent(id)?;
        declared.push((InputExtentParameter { access, axis }, value));
    }
    Ok(declared)
}

fn emit_live_row_major(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffers: &[KernelBufferId],
    write_buffer: KernelBufferId,
    row: KernelValueId,
    live: &[(InputExtentParameter, KernelValueId)],
) -> Result<(), KernelBuildError> {
    let columns = live
        .iter()
        .find_map(|(parameter, value)| {
            let position = usize::try_from(parameter.access.get()).ok()?;
            match plan.addressing.get(position) {
                Some(ReadAddressing::LiveRowMajor { inner_axis })
                    if parameter.axis == *inner_axis =>
                {
                    Some(*value)
                }
                _ => None,
            }
        })
        .ok_or(KernelBuildError::UndeclaredInputExtent)?;
    let start = builder.constant(KernelConstant::Index(0))?;
    let seed = builder.constant(KernelConstant::Index(0))?;
    builder.serial_loop_range(start, columns, &[seed], |builder, parameters| {
        let col = parameters.induction();
        let stride = builder.binary(BinaryOp::IndexMultiply, row, columns)?;
        let offset = builder.binary(BinaryOp::IndexAdd, stride, col)?;
        // Every read at the loop's own row-major offset, which is correct
        // exactly because `plan` admits a live region only when every access is
        // live on the same inner axis — and refuses a live region carrying a
        // data-dependent read, whose address is its own rather than this one.
        let mut inputs = Vec::with_capacity(read_buffers.len());
        for (buffer, read) in read_buffers.iter().zip(plan.reads) {
            inputs.push(Some(builder.load(*buffer, offset, read.bounds)?));
        }
        let mapped = match plan.scalar {
            ScalarProgram::PointwiseF32(expression) => {
                emit_pointwise(builder, expression, &inputs)?
            }
            ScalarProgram::PointwiseBf16(expression) => {
                emit_pointwise_bf16(builder, expression, &inputs)?
            }
            _ => {
                return Err(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Value,
                });
            }
        };
        builder.store(
            write_buffer,
            offset,
            mapped,
            plan.write_bounds,
            plan.ownership,
        )?;
        let carried = parameters
            .accumulator(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
        Ok(vec![carried])
    })?;
    Ok(())
}

fn emit_guarded(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffers: &[KernelBufferId],
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
    live: &[(InputExtentParameter, KernelValueId)],
) -> Result<(), KernelBuildError> {
    // The reduction families read exactly one tensor, which the schedule
    // verifier proved before this plan existed. Resolving that here as a typed
    // handle error rather than by indexing keeps a widened region from lowering
    // against whichever buffer happened to be first.
    let sole_read_buffer = || {
        let [buffer] = read_buffers else {
            return Err(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            });
        };
        Ok(*buffer)
    };
    let sole_read = || {
        let [read] = plan.reads else {
            return Err(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            });
        };
        Ok(read.bounds)
    };
    match plan.scalar {
        ScalarProgram::PointwiseF32(expression) => {
            let inputs = emit_pointwise_loads(builder, plan, read_buffers, invocation)?;
            let mapped = emit_pointwise(builder, expression, &inputs)?;
            builder.store(
                write_buffer,
                invocation,
                mapped,
                plan.write_bounds,
                plan.ownership,
            )
        }
        // The same shape at the other width, and deliberately a separate arm
        // rather than a shared one parameterized by an operation table: the two
        // node vocabularies are different sets, so one emitter would have to
        // decide what an `f32`-only node means at `bf16`. The *loads* are shared
        // because they read no element type at all — a load takes the type its
        // buffer declares — so one loader cannot make that decision.
        ScalarProgram::PointwiseBf16(expression) => {
            let inputs = emit_pointwise_loads(builder, plan, read_buffers, invocation)?;
            let mapped = emit_pointwise_bf16(builder, expression, &inputs)?;
            builder.store(
                write_buffer,
                invocation,
                mapped,
                plan.write_bounds,
                plan.ownership,
            )
        }
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            unreachable!("strict-affine lowering uses its role-addressed signature")
        }
        ScalarProgram::StrictSerialSum {
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            SerialFold {
                empty_identity_bits: *empty_identity_bits,
                prologue: ReductionPrologue::None,
                epilogue: None,
            },
        ),
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            SerialFold {
                empty_identity_bits: *empty_identity_bits,
                prologue: ReductionPrologue::ScaleBias {
                    scale_bits: *scale_bits,
                    bias_bits: *bias_bits,
                },
                epilogue: None,
            },
        ),
        ScalarProgram::SquaredSerialSum {
            empty_identity_bits,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            SerialFold {
                empty_identity_bits: *empty_identity_bits,
                prologue: ReductionPrologue::Square,
                epilogue: None,
            },
        ),
        // The same fold, and the epilogue applied to its value before the store.
        ScalarProgram::SquaredSerialSumThenEpilogue {
            empty_identity_bits,
            epilogue,
            ..
        } => emit_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
            SerialFold {
                empty_identity_bits: *empty_identity_bits,
                prologue: ReductionPrologue::Square,
                epilogue: Some(epilogue),
            },
        ),
        ScalarProgram::StrictSerialMaximum { .. } => emit_maximum_reduction(
            builder,
            plan,
            (sole_read_buffer()?, sole_read()?),
            write_buffer,
            invocation,
        ),
        ScalarProgram::StrictTensorContraction { .. } => {
            let ([left, right], [left_read, right_read]) = (read_buffers, plan.reads) else {
                return Err(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Buffer,
                });
            };
            emit_contraction(
                builder,
                plan,
                [(*left, left_read.bounds), (*right, right_read.bounds)],
                write_buffer,
                invocation,
                live,
            )
        }
    }
}

/// Emits the guarded body of one strict tensor contraction.
///
/// One thread folds its own output element in ascending contracted order. The
/// accumulator is seeded at the *first product* rather than at `+0.0`: the two
/// differ observably where every product is `-0.0`, and the registered family
/// declares no seed, so an identity-seeded fold would compute a contraction
/// carrying an explicit `initial` — a different operation.
///
/// **The fold is deliberately three separate structured operations per step**: a
/// multiply, a NaN canonicalization, and an add. The canonicalization is the
/// declared `after-every-combine-and-at-the-result-boundary` rule reaching the
/// product, and it is also what makes a fused multiply-add unformable — the
/// backend sees a call between the two arithmetic operations, not an adjacent
/// pair. That matters because the governed contracts forbid ADR 0015 contraction
/// and the measured Apple row shows `-ffp-contract=off` is no defence against a
/// *fused instruction the source asks for*; here the source cannot ask for one.
///
/// **No result-boundary conversion is emitted, and its absence is derived.** The
/// serial sum needs one when its contributor sequence is a singleton, because
/// its seed is a raw load no combine has canonicalized. A contraction's seed is
/// a *product*, which this emission canonicalizes, so every path out of the fold
/// already carries the canonical payload and a second conversion would be a
/// provable identity in a body the refinement gate compares structurally.
fn emit_contraction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    reads: [(KernelBufferId, BoundsWitnessId); 2],
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
    live: &[(InputExtentParameter, KernelValueId)],
) -> Result<(), KernelBuildError> {
    let seed = emit_contraction_product(builder, plan, reads, invocation, None, live)?;
    let total = if let Some(bound) = live_contraction_bound(plan, live) {
        let start = builder.constant(KernelConstant::Index(1))?;
        let results = builder.serial_loop_range(start, bound, &[seed], |builder, parameters| {
            let induction = parameters.induction();
            let accumulator = parameters
                .accumulator(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            let product =
                emit_contraction_product(builder, plan, reads, invocation, Some(induction), live)?;
            let sum = builder.binary(BinaryOp::F32Add, accumulator, product)?;
            let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
            Ok(vec![sum])
        })?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?
    } else if plan.contributors <= 1 {
        seed
    } else {
        let results = builder.serial_loop(
            SerialLoopSpec {
                start: 1,
                end: plan.contributors,
            },
            &[seed],
            |builder, parameters| {
                let induction = parameters.induction();
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                let product = emit_contraction_product(
                    builder,
                    plan,
                    reads,
                    invocation,
                    Some(induction),
                    live,
                )?;
                let sum = builder.binary(BinaryOp::F32Add, accumulator, product)?;
                let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
                Ok(vec![sum])
            },
        )?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?
    };
    builder.store(
        write_buffer,
        invocation,
        total,
        plan.write_bounds,
        plan.ownership,
    )
}

/// Emits one contracted point's separately rounded, canonicalized product.
fn live_contraction_bound(
    plan: &CanonicalPlan<'_>,
    live: &[(InputExtentParameter, KernelValueId)],
) -> Option<KernelValueId> {
    let (access, axis) = plan.live_contraction?;
    live.iter().find_map(|(parameter, value)| {
        (parameter.access == access && parameter.axis == axis).then_some(*value)
    })
}

fn emit_contraction_product(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    reads: [(KernelBufferId, BoundsWitnessId); 2],
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
    live: &[(InputExtentParameter, KernelValueId)],
) -> Result<KernelValueId, KernelBuildError> {
    let mut loaded = [None, None];
    for (position, (buffer, bounds)) in reads.into_iter().enumerate() {
        let addressing = plan
            .addressing
            .get(position)
            .ok_or(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            })?;
        let offset = match addressing {
            ReadAddressing::LiveContracted { free } => {
                let bound = live_contraction_bound(plan, live)
                    .ok_or(KernelBuildError::UndeclaredInputExtent)?;
                emit_live_contracted_offset(builder, free, invocation, contributor, bound)?
            }
            _ => emit_offset(builder, addressing, invocation, contributor)?,
        };
        loaded[position] = Some(builder.load(buffer, offset, bounds)?);
    }
    let [Some(left), Some(right)] = loaded else {
        return Err(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Value,
        });
    };
    let product = builder.binary(BinaryOp::F32Multiply, left, right)?;
    builder.convert(ConvertOp::CanonicalizeF32Nan, product)
}

/// Emits the canonical body of one cooperative workgroup reduction.
///
/// # The shape, and why each part of it is where it is
///
/// ```text
/// %gid = global invocation index        %lid = local invocation index
/// %out = %gid / participants            %par = %gid % participants
/// %act = %gid < work_items
/// if (%act) { fold contributors [%par·k, (%par+1)·k) ; staged_store[%lid] }
/// barrier(point)
/// if (%act) { if (%lid < commit) { fold staged[0..participants] ; store[%out] } }
/// ```
///
/// A tile with several rounds is [`emit_loop_carried_cooperative`]; the two
/// shapes are deliberately distinct rather than one generalized emission, and
/// that function states why.
///
/// **The barrier is at the top level, and that is a correctness rule rather than
/// a layout preference.** A control barrier inside a predicated region is reached
/// by whichever invocations the predicate admits, and one not reached by every
/// participant is undefined execution on every target. Placing it outside both
/// guarded blocks makes convergence structural, so it survives any later change
/// to what those guards test.
///
/// **The staged store is inside the iteration guard, and the launch is what
/// makes that sound.** `TailPolicy::Exact` and the intrinsic verifier's
/// `grid_threads == work_items` rule together mean every launched invocation
/// satisfies the guard, so every slot the consuming phase reads was written. A
/// tail policy that admitted inactive lanes would leave slots unwritten, which
/// is why widening the tail vocabulary must revisit this emission rather than
/// inherit it.
///
/// **The invocation split is emitted once, at the top level.** The committing
/// participant's owning store needs the same output coordinate the producing
/// phase's loads used, and a value defined inside a guarded block cannot cross
/// into the next one.
///
/// **The staged fold seeds at the first slot and canonicalizes after each
/// combine**, exactly as the serial fold does. A partial that is a single
/// uncombined load gets no conversion of its own: it is not a result boundary,
/// and the fold that consumes it canonicalizes every combine — so a conversion
/// there would be a provable identity in a body the refinement gate compares
/// structurally.
fn emit_cooperative(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    cooperative: &CooperativePlan,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    let ([read], [addressing]) = (plan.reads, plan.addressing.as_slice()) else {
        return Err(KernelLoweringError::UnsupportedRegion {
            rule: "cooperative-access-count",
        });
    };
    let fold = reduction_fold(plan.scalar).ok_or(KernelLoweringError::UnsupportedRegion {
        rule: "cooperative-scalar-program",
    })?;
    // Derived from the scalar program for the reason `emit` states. Every
    // cooperative region is a reduction and every reduction is `f32` today, so
    // this resolves to `F32` at present; deriving it keeps the declaration and
    // `verify_signature`'s expectation reading one authority rather than two
    // literals that a later widening would have to move together.
    let element_type = region_element_type(plan.scalar);
    let read_buffer = builder.declare_buffer(BufferParameter {
        tensor: read.tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements.first().copied().unwrap_or(0),
    })?;
    let write_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write_tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Write,
        element_count: plan.write_elements,
    })?;
    builder.admit_builtin(Builtin::GlobalInvocationIndex)?;
    builder.admit_builtin(Builtin::LocalInvocationIndex)?;
    let staging = builder.declare_staging(StagingParameter {
        staging: cooperative.staging,
        element_type: KernelType::F32,
        address_space: AddressSpace::Workgroup,
        element_count: cooperative.slots,
    })?;
    builder.numerical(plan.numerical)?;
    builder.requirements(requirements)?;

    let invocation = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let local = builder.builtin(Builtin::LocalInvocationIndex)?;
    let (output, partition) =
        split_partitioned_invocation(builder, invocation, cooperative.participants)?;
    let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
    let active = builder.compare(CompareOp::IndexLessThan, invocation, extent)?;
    let emission = CooperativeEmission {
        plan: cooperative,
        addressing,
        read: (read_buffer, read.bounds),
        staging,
        split: SplitInvocation { output, partition },
        local,
        active,
        fold,
    };

    if cooperative.round_barrier.is_some() {
        return emit_loop_carried_cooperative(builder, plan, &emission, write_buffer);
    }
    emit_round_production(builder, &emission, None)?;
    builder.barrier(cooperative.barrier.clone())?;
    builder.predicated(active, |builder| {
        let commit = builder.constant(KernelConstant::Index(cooperative.commit_count))?;
        let commits = builder.compare(CompareOp::IndexLessThan, local, commit)?;
        builder.predicated(commits, |builder| {
            let total = emit_staged_fold(builder, cooperative, staging, fold.combiner)?;
            builder.store(
                write_buffer,
                output,
                total,
                plan.write_bounds,
                plan.ownership,
            )
        })
    })?;
    Ok(())
}

fn emit_cooperative_contraction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    contraction: &CooperativeContractionPlan,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    let ([left, right], [left_addr, right_addr]) = (plan.reads, plan.addressing.as_slice()) else {
        return Err(KernelLoweringError::UnsupportedRegion {
            rule: "cooperative-contraction-access-count",
        });
    };
    // The declared layout of each operand, resolved by `plan` from the access
    // map the region states. Refusing anything else here rather than falling back
    // to an assumed `[M, K]` / `[N, K]` pair is the point: those two layouts are
    // one of four the vocabulary expresses, and the other three used to lower to
    // a kernel that read the wrong elements with no refusal anywhere.
    let (
        ReadAddressing::BlockedContraction(left_terms),
        ReadAddressing::BlockedContraction(right_terms),
    ) = (left_addr, right_addr)
    else {
        return Err(KernelLoweringError::UnsupportedRegion {
            rule: "cooperative-contraction-operand-layout",
        });
    };
    let element_type = region_element_type(plan.scalar);
    let left_buffer = builder.declare_buffer(BufferParameter {
        tensor: left.tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements.first().copied().unwrap_or(0),
    })?;
    let right_buffer = builder.declare_buffer(BufferParameter {
        tensor: right.tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements.get(1).copied().unwrap_or(0),
    })?;
    let write_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write_tensor,
        component_role: None,
        element_type,
        address_space: AddressSpace::Device,
        access: BufferAccess::Write,
        element_count: plan.write_elements,
    })?;
    builder.admit_builtin(Builtin::GlobalInvocationIndex)?;
    builder.admit_builtin(Builtin::LocalInvocationIndex)?;
    let left_staging = builder.declare_staging(StagingParameter {
        staging: contraction.left_staging,
        element_type: KernelType::F32,
        address_space: AddressSpace::Workgroup,
        element_count: contraction.left_slots,
    })?;
    let right_staging = builder.declare_staging(StagingParameter {
        staging: contraction.right_staging,
        element_type: KernelType::F32,
        address_space: AddressSpace::Workgroup,
        element_count: contraction.right_slots,
    })?;
    builder.numerical(plan.numerical)?;
    builder.requirements(requirements)?;

    let gid = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let lid = builder.builtin(Builtin::LocalInvocationIndex)?;
    let block = builder.constant(KernelConstant::Index(contraction.block))?;
    let threads = builder.constant(KernelConstant::Index(
        contraction.block.checked_mul(contraction.block).ok_or(
            KernelLoweringError::Verification(KernelDiagnostic::ElementCountOverflow),
        )?,
    ))?;
    let workgroups_n = builder.constant(KernelConstant::Index(contraction.workgroups_n))?;
    let output_m = builder.constant(KernelConstant::Index(contraction.output_m))?;
    let output_n = builder.constant(KernelConstant::Index(contraction.output_n))?;
    // One index constant per *distinct* non-unit operand stride, in first
    // appearance order across the two operands. Two operands that share a stride
    // — which every `[M, K]` / `[N, K]` pair does, both being the contracted
    // extent — therefore share one constant, exactly as the single hardcoded
    // constant this replaces did.
    let mut strides: Vec<(u64, KernelValueId)> = Vec::new();
    for term in left_terms.iter().chain(right_terms.iter()) {
        if term.stride <= 1 || strides.iter().any(|(stride, _)| *stride == term.stride) {
            continue;
        }
        let value = builder.constant(KernelConstant::Index(term.stride))?;
        strides.push((term.stride, value));
    }
    // The workgroup index, decoded axis by axis from the fastest-varying end
    // backwards. Each step divides the running quotient by the axis it has just
    // consumed, so the quotient entering axis `d` is the linearization over axes
    // `0..=d` and one modulo recovers that axis's coordinate. The *leading* axis
    // needs no modulo, because its quotient is already below its extent — the
    // same redundancy the linearizing forms drop.
    //
    // At rank two the chain is one step long and emits exactly `wg % W_n` then
    // `wg / W_n`, in that order, which is the body this generalizes and whose
    // canonical identity it must not disturb.
    let wg = builder.binary(BinaryOp::IndexDivide, gid, threads)?;
    let wg_n = builder.binary(BinaryOp::IndexModulo, wg, workgroups_n)?;
    let mut quotient = builder.binary(BinaryOp::IndexDivide, wg, workgroups_n)?;
    // Every workgroup axis above N, in axis order: the batch axes, then M. Their
    // coordinates come out of `quotient` from the fastest-varying end backwards,
    // each step consuming one extent, so the value entering axis `d` is the
    // linearization over axes `0..=d` and one modulo recovers `d`'s coordinate.
    // The *leading* axis needs no modulo — its quotient is already below its
    // extent — which is the same redundancy the linearizing forms drop.
    //
    // At rank two this list is `[workgroups_m]` alone: the loop takes its
    // leading-axis arm immediately, emits nothing, and `wg_m` is the bare
    // `wg / W_n` the unbatched body has always computed. That is what keeps a
    // rank-two region's emitted body, and so its canonical identity, unchanged.
    let above_extents: Vec<u64> = contraction
        .batch_workgroups
        .iter()
        .copied()
        .chain(std::iter::once(contraction.workgroups_m))
        .collect();
    // Every slot is assigned by the loop below, which covers `0..len` and ends at
    // the leading axis; the seed only gives the vector a length.
    let mut above = vec![quotient; above_extents.len()];
    for axis in (0..above_extents.len()).rev() {
        if axis == 0 {
            above[axis] = quotient;
            break;
        }
        let extent = builder.constant(KernelConstant::Index(above_extents[axis]))?;
        above[axis] = builder.binary(BinaryOp::IndexModulo, quotient, extent)?;
        quotient = builder.binary(BinaryOp::IndexDivide, quotient, extent)?;
    }
    let (batch, wg_m) = above.split_last().map(|(last, rest)| (rest, *last)).ok_or(
        KernelLoweringError::Verification(KernelDiagnostic::CooperativeLoweringShape),
    )?;
    let local_n = builder.binary(BinaryOp::IndexModulo, lid, block)?;
    let local_m = builder.binary(BinaryOp::IndexDivide, lid, block)?;
    let row_base = builder.binary(BinaryOp::IndexMultiply, wg_m, block)?;
    let col_base = builder.binary(BinaryOp::IndexMultiply, wg_n, block)?;
    let row = builder.binary(BinaryOp::IndexAdd, row_base, local_m)?;
    let col = builder.binary(BinaryOp::IndexAdd, col_base, local_n)?;
    let row_active = builder.compare(CompareOp::IndexLessThan, row, output_m)?;
    let column_active = builder.compare(CompareOp::IndexLessThan, col, output_n)?;
    let inactive = builder.constant(KernelConstant::F32Bits(0.0_f32.to_bits()))?;

    let emit_tile =
        |builder: &mut KernelBuilder, k0: KernelValueId| -> Result<(), KernelBuildError> {
            // Participant `(m, n)` fetches the left tile's column `n` and the
            // right tile's column `m`. That is the tile's staging relation rather
            // than either operand's layout, so it stays stated here while the
            // address the coordinate feeds comes from the declared map.
            let left_k = builder.binary(BinaryOp::IndexAdd, k0, local_n)?;
            let right_k = builder.binary(BinaryOp::IndexAdd, k0, local_m)?;
            // Both operands' terms are scaled before either is summed. The two
            // addresses are independent, so the interleaving is free — and fixing
            // it this way is what keeps a region whose operands are already
            // `[M, K]` and `[N, K]` emitting the byte-identical body, and so the
            // identical canonical identity, that it did before.
            let left_scaled =
                scale_blocked_terms(builder, left_terms, &strides, batch, row, col, left_k)?;
            let right_scaled =
                scale_blocked_terms(builder, right_terms, &strides, batch, row, col, right_k)?;
            let left_off = sum_blocked_terms(builder, &left_scaled)?;
            let right_off = sum_blocked_terms(builder, &right_scaled)?;
            let left_val = if contraction.predicated {
                builder.guarded_load(row_active, left_buffer, left_off, left.bounds, inactive)?
            } else {
                builder.load(left_buffer, left_off, left.bounds)?
            };
            let right_val = if contraction.predicated {
                builder.guarded_load(
                    column_active,
                    right_buffer,
                    right_off,
                    right.bounds,
                    inactive,
                )?
            } else {
                builder.load(right_buffer, right_off, right.bounds)?
            };
            let left_slot = builder.binary(BinaryOp::IndexMultiply, local_m, block)?;
            let left_slot = builder.binary(BinaryOp::IndexAdd, left_slot, local_n)?;
            let right_slot = builder.binary(BinaryOp::IndexMultiply, local_n, block)?;
            let right_slot = builder.binary(BinaryOp::IndexAdd, right_slot, local_m)?;
            builder.staged_store(left_staging, left_slot, left_val, contraction.produce_phase)?;
            builder.staged_store(
                right_staging,
                right_slot,
                right_val,
                contraction.produce_phase,
            )?;
            Ok(())
        };

    // One tile's contributors, folded into the *carried* accumulator rather than
    // into a subtotal of their own. `carried` is `None` on the first round and
    // the running accumulator on every later one, which is the whole difference
    // between this schedule and a contiguous-interval split: `acc + (p0 + … +
    // p15)` regroups the declared contributor sequence and consumes
    // reassociation, while `((acc + p0) + …) + p15` is that sequence itself and
    // consumes nothing. The L3 record attributes the measured `tiled` kernel
    // uniquely to `strict_fold+ftz` and records it byte-identical to `direct` at
    // every profile cell, which is only true of the second form — the reference
    // text carries one `accumulator` across its `k0` loop and never restarts it.
    // The regrouped form is a different realization with its own reserved
    // vocabulary (`CooperativeContractionSplit`, schedule tag `0x36`), so
    // emitting it here would give one topology two numerical meanings.
    let fold_tile = |builder: &mut KernelBuilder,
                     carried: Option<KernelValueId>|
     -> Result<KernelValueId, KernelBuildError> {
        let left_base = builder.binary(BinaryOp::IndexMultiply, local_m, block)?;
        let right_base = builder.binary(BinaryOp::IndexMultiply, local_n, block)?;
        // The first round has no accumulator to continue, so its first product
        // *is* the seed and the loop starts at one. `+0.0 + p` is not the same
        // binary32 value as `p` when `p` is negative zero, so seeding from a
        // literal zero would be a different computation rather than a tidier
        // spelling of this one.
        let (seed, start) = match carried {
            None => {
                let zero = builder.constant(KernelConstant::Index(0))?;
                let left_first = builder.binary(BinaryOp::IndexAdd, left_base, zero)?;
                let right_first = builder.binary(BinaryOp::IndexAdd, right_base, zero)?;
                let a0 =
                    builder.staged_load(left_staging, left_first, contraction.consume_phase)?;
                let b0 =
                    builder.staged_load(right_staging, right_first, contraction.consume_phase)?;
                let seed = builder.binary(BinaryOp::F32Multiply, a0, b0)?;
                let seed = builder.convert(ConvertOp::CanonicalizeF32Nan, seed)?;
                (seed, 1)
            }
            Some(accumulator) => (accumulator, 0),
        };
        if start >= contraction.block {
            return Ok(seed);
        }
        let results = builder.serial_loop(
            SerialLoopSpec {
                start,
                end: contraction.block,
            },
            &[seed],
            |builder, parameters| {
                let kk = parameters.induction();
                let acc = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                let left_slot = builder.binary(BinaryOp::IndexAdd, left_base, kk)?;
                let right_slot = builder.binary(BinaryOp::IndexAdd, right_base, kk)?;
                let a = builder.staged_load(left_staging, left_slot, contraction.consume_phase)?;
                let b =
                    builder.staged_load(right_staging, right_slot, contraction.consume_phase)?;
                let product = builder.binary(BinaryOp::F32Multiply, a, b)?;
                let product = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
                let folded = builder.binary(BinaryOp::F32Add, acc, product)?;
                let folded = builder.convert(ConvertOp::CanonicalizeF32Nan, folded)?;
                Ok(vec![folded])
            },
        )?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)
    };

    let k0 = builder.constant(KernelConstant::Index(0))?;
    emit_tile(builder, k0)?;
    builder.barrier(contraction.barrier.clone())?;
    let seed = fold_tile(builder, None)?;
    let total =
        if contraction.rounds > 1 {
            let round_barrier = contraction.round_barrier.clone().ok_or(
                KernelLoweringError::UnsupportedRegion {
                    rule: "cooperative-contraction-round-boundary",
                },
            )?;
            let results = builder.serial_loop(
                SerialLoopSpec {
                    start: 1,
                    end: contraction.rounds,
                },
                &[seed],
                |builder, parameters| {
                    let round = parameters.induction();
                    let acc = parameters
                        .accumulator(0)
                        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                    builder.barrier(round_barrier.clone())?;
                    let stride = builder.constant(KernelConstant::Index(contraction.block))?;
                    let k0 = builder.binary(BinaryOp::IndexMultiply, round, stride)?;
                    emit_tile(builder, k0)?;
                    builder.barrier(contraction.barrier.clone())?;
                    // The accumulator enters the tile fold rather than meeting a
                    // subtotal after it, so this round loop adds no combining
                    // step of its own.
                    let folded = fold_tile(builder, Some(acc))?;
                    Ok(vec![folded])
                },
            )?;
            results
                .get(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?
        } else {
            seed
        };

    // The owning store's linear position in the row-major output. `row * N + col`
    // is the trailing pair's contribution and is emitted first, so a rank-two
    // region's body is exactly the two operations it always was; each batch axis
    // then adds its own `coordinate * stride`. The write map is `LinearIdentity`,
    // so this sum *is* the position that map names.
    let emit_output_offset =
        |builder: &mut KernelBuilder| -> Result<KernelValueId, KernelBuildError> {
            let mut out = builder.binary(BinaryOp::IndexMultiply, row, output_n)?;
            out = builder.binary(BinaryOp::IndexAdd, out, col)?;
            for (coordinate, stride) in batch.iter().zip(&contraction.batch_output_strides) {
                let stride = builder.constant(KernelConstant::Index(*stride))?;
                let scaled = builder.binary(BinaryOp::IndexMultiply, *coordinate, stride)?;
                out = builder.binary(BinaryOp::IndexAdd, out, scaled)?;
            }
            Ok(out)
        };

    if contraction.predicated {
        builder.predicated(row_active, |builder| {
            builder.predicated(column_active, |builder| {
                let out = emit_output_offset(builder)?;
                builder.store(write_buffer, out, total, plan.write_bounds, plan.ownership)
            })
        })?;
    } else {
        let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
        let active = builder.compare(CompareOp::IndexLessThan, gid, extent)?;
        builder.predicated(active, |builder| {
            let out = emit_output_offset(builder)?;
            builder.store(write_buffer, out, total, plan.write_bounds, plan.ownership)
        })?;
    }
    Ok(())
}

/// Scales each blocked term's coordinate by the operand's own stride.
///
/// A unit stride emits no operation and yields the coordinate itself, which is
/// what keeps the trailing axis of a row-major operand free — and what makes the
/// transposed layout cost exactly the same one multiply and one add as the
/// hardcoded one did, rather than paying for its generality.
///
/// `strides` holds the top-level constant for every non-unit stride the two
/// operands declare. A term whose stride is absent from it is reported as an
/// invalid handle rather than given a freshly emitted constant, because a
/// constant emitted inside the round loop would not be the same value the peeled
/// round used and the refinement gate compares those bodies structurally.
fn scale_blocked_terms(
    builder: &mut KernelBuilder,
    terms: &[BlockedTerm],
    strides: &[(u64, KernelValueId)],
    batch: &[KernelValueId],
    row: KernelValueId,
    column: KernelValueId,
    contracted: KernelValueId,
) -> Result<Vec<KernelValueId>, KernelBuildError> {
    let mut scaled = Vec::with_capacity(terms.len());
    for term in terms {
        let coordinate = match term.coordinate {
            // A term naming a batch axis the decode did not produce is an
            // invalid handle rather than a silently dropped coordinate: dropping
            // it would address batch zero's element for every batch.
            BlockedCoordinate::Batch(axis) => {
                *batch.get(axis).ok_or(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Value,
                })?
            }
            BlockedCoordinate::Row => row,
            BlockedCoordinate::Column => column,
            BlockedCoordinate::Contracted => contracted,
        };
        if term.stride == 1 {
            scaled.push(coordinate);
            continue;
        }
        let stride = strides
            .iter()
            .find_map(|(stride, value)| (*stride == term.stride).then_some(*value))
            .ok_or(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Value,
            })?;
        scaled.push(builder.binary(BinaryOp::IndexMultiply, coordinate, stride)?);
    }
    Ok(scaled)
}

/// Sums the scaled terms of one blocked operand address in axis order.
///
/// An operand every one of whose axes was dropped — every extent one — addresses
/// its single element at zero, which is the one case that emits a constant.
fn sum_blocked_terms(
    builder: &mut KernelBuilder,
    scaled: &[KernelValueId],
) -> Result<KernelValueId, KernelBuildError> {
    let mut total: Option<KernelValueId> = None;
    for value in scaled {
        total = Some(match total {
            None => *value,
            Some(accumulated) => builder.binary(BinaryOp::IndexAdd, accumulated, *value)?,
        });
    }
    match total {
        Some(value) => Ok(value),
        None => builder.constant(KernelConstant::Index(0)),
    }
}

/// Emits the canonical body of a cooperative tile whose phases repeat.
///
/// ```text
/// %gid, %lid, %out, %par, %act                    (as above)
/// if (%act) { fold round 0's range ; staged_store[%lid] }
/// barrier(phase point)
/// %seed = fold staged[0..participants]
/// %total = loop r in 1..rounds carrying %seed {
///     barrier(round point)
///     if (%act) { fold round r's range ; staged_store[%lid] }
///     barrier(phase point)
///     %t = fold staged[0..participants]
///     yield canonicalize(%acc + %t)
/// }
/// if (%act) { if (%lid < commit) { store[%out] = %total } }
/// ```
///
/// **Round zero is peeled because the fold seeds at its first contributor.** A
/// sum seeded at `+0.0` is a different function — `+0.0 + x` is not `x` at
/// `x = -0.0` — and the registered family declares no seed, so the accumulator's
/// initial value has to be round zero's own staged total. That makes the loop
/// `1..rounds` and realizes the phase boundary once ahead of the loop and
/// `rounds - 1` times inside it, which is exactly `rounds` dynamic realizations;
/// the round boundary, which separates *consecutive* rounds, is realized
/// `rounds - 1` times and therefore belongs at the head of the loop body rather
/// than the tail. At the tail it would leave the peeled round's reads unordered
/// against the loop's first rewrite, and
/// [`KernelDiagnostic::UnorderedStagedRewrite`] is what refuses that.
///
/// **The staged fold is outside every predicate, and that is what makes the
/// accumulator expressible at all.** A predicated region produces no values, so a
/// total computed inside one cannot cross the loop's back edge. Staged accesses
/// are deliberately not boundary effects — `verify_effects` requires predicate
/// dominance of loads and stores, not of staging — so the fold needs no guard,
/// and it is sound for the same reason the top-level barrier is: the launch is
/// exact, so every launched invocation satisfies the iteration guard and every
/// slot it reads was written.
///
/// **Every participant therefore folds the staged set, and only one commits.**
/// That is redundant work the single-round shape does not pay, which is why the
/// two emissions are separate: hoisting the single-round fold out of its commit
/// guard would buy nothing and cost `participants - 1` redundant folds per
/// workgroup. Removing the redundancy here needs a predicated region that yields
/// values, which this vocabulary does not have.
fn emit_loop_carried_cooperative(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    emission: &CooperativeEmission<'_>,
    write_buffer: KernelBufferId,
) -> Result<(), KernelLoweringError> {
    let cooperative = emission.plan;
    let round_barrier =
        cooperative
            .round_barrier
            .clone()
            .ok_or(KernelLoweringError::UnsupportedRegion {
                rule: "cooperative-round-boundary",
            })?;
    emit_round_production(builder, emission, None)?;
    builder.barrier(cooperative.barrier.clone())?;
    let combiner = emission.fold.combiner;
    let seed = emit_staged_fold(builder, cooperative, emission.staging, combiner)?;
    let results = builder.serial_loop(
        SerialLoopSpec {
            start: 1,
            end: cooperative.rounds,
        },
        &[seed],
        |builder, parameters| {
            let round = parameters.induction();
            let accumulator = parameters
                .accumulator(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            builder.barrier(round_barrier.clone())?;
            emit_round_production(
                builder,
                emission,
                Some(RoundOrdinal {
                    value: round,
                    contributors_per_round: cooperative.contributors_per_round,
                }),
            )?;
            builder.barrier(cooperative.barrier.clone())?;
            let staged = emit_staged_fold(builder, cooperative, emission.staging, combiner)?;
            let folded = builder.binary(combiner.op(), accumulator, staged)?;
            let folded = builder.convert(ConvertOp::CanonicalizeF32Nan, folded)?;
            Ok(vec![folded])
        },
    )?;
    let total = results
        .get(0)
        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
    builder.predicated(emission.active, |builder| {
        let commit = builder.constant(KernelConstant::Index(cooperative.commit_count))?;
        let commits = builder.compare(CompareOp::IndexLessThan, emission.local, commit)?;
        builder.predicated(commits, |builder| {
            builder.store(
                write_buffer,
                emission.split.output,
                total,
                plan.write_bounds,
                plan.ownership,
            )
        })
    })?;
    Ok(())
}

/// The already-split invocation roots one cooperative fold addresses through.
///
/// A pair rather than two arguments because they are always derived together and
/// always travel together: the output coordinate and the partition ordinal are
/// the two halves of one `IndexDivide`/`IndexModulo` split, emitted once at the
/// kernel's top level so the committing store can reach the same output the
/// producing phase's loads used.
#[derive(Clone, Copy, Debug)]
struct SplitInvocation {
    output: KernelValueId,
    partition: Option<KernelValueId>,
}

/// The round ordinal one cooperative fold addresses through, and its stride.
///
/// `None` at the call sites that have no round to name — the peeled round zero,
/// whose ordinal is constantly zero, and the multi-pass partial, which has no
/// round dimension at all — so every round term vanishes exactly rather than
/// being emitted as a multiplication by a zero the reader has to trust.
#[derive(Clone, Copy, Debug)]
struct RoundOrdinal {
    value: KernelValueId,
    contributors_per_round: u64,
}

/// Everything one cooperative round body is emitted against, resolved once.
///
/// A struct rather than eight arguments threaded through three functions: they
/// are all top-level values or plan fields, they are all needed by both the
/// peeled round and the loop body, and passing them separately made the round
/// ordinal — the one thing that actually differs between the two — the ninth
/// positional argument rather than the visible difference.
#[derive(Clone, Copy, Debug)]
struct CooperativeEmission<'a> {
    plan: &'a CooperativePlan,
    addressing: &'a ReadAddressing,
    read: (KernelBufferId, BoundsWitnessId),
    staging: KernelStagingId,
    split: SplitInvocation,
    local: KernelValueId,
    active: KernelValueId,
    fold: ReductionFold,
}

/// Emits one round's guarded fold and the staged write that publishes it.
fn emit_round_production(
    builder: &mut KernelBuilder,
    emission: &CooperativeEmission<'_>,
    round: Option<RoundOrdinal>,
) -> Result<(), KernelBuildError> {
    builder.predicated(emission.active, |builder| {
        let partial = emit_partition_fold(builder, emission, round)?;
        let slot = emit_staged_slot(builder, emission.plan, emission.local)?;
        builder.staged_store(emission.staging, slot, partial, emission.plan.produce_phase)
    })
}

/// Folds one participant's own contiguous share of one round's contributors.
fn emit_partition_fold(
    builder: &mut KernelBuilder,
    emission: &CooperativeEmission<'_>,
    round: Option<RoundOrdinal>,
) -> Result<KernelValueId, KernelBuildError> {
    let (read_buffer, read_bounds) = emission.read;
    let SplitInvocation { output, partition } = emission.split;
    let addressing = emission.addressing;
    let ReductionFold { prologue, combiner } = emission.fold;
    let contributors = emission.plan.contributors_per_partition;
    let seed_contributor =
        emit_partition_contributor(builder, round, partition, None, contributors)?;
    let first_offset = emit_offset(builder, addressing, output, seed_contributor)?;
    let first = builder.load(read_buffer, first_offset, read_bounds)?;
    let seed = emit_prologue(builder, first, prologue)?;
    if contributors <= 1 {
        return Ok(seed);
    }
    let results = builder.serial_loop(
        SerialLoopSpec {
            start: 1,
            end: contributors,
        },
        &[seed],
        |builder, parameters| {
            let induction = parameters.induction();
            let accumulator = parameters
                .accumulator(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            let contributor = emit_partition_contributor(
                builder,
                round,
                partition,
                Some(induction),
                contributors,
            )?;
            let offset = emit_offset(builder, addressing, output, contributor)?;
            let loaded = builder.load(read_buffer, offset, read_bounds)?;
            let value = emit_prologue(builder, loaded, prologue)?;
            let folded = builder.binary(combiner.op(), accumulator, value)?;
            let folded = builder.convert(ConvertOp::CanonicalizeF32Nan, folded)?;
            Ok(vec![folded])
        },
    )?;
    results
        .get(0)
        .ok_or(KernelBuildError::EmptyLoopAccumulators)
}

/// Emits the staging slot one participant writes.
///
/// `stride * local + offset`, with each operation dropped where it is provably
/// the identity, so the canonical body carries no operation a refinement gate
/// would have to compare against a computed nothing.
fn emit_staged_slot(
    builder: &mut KernelBuilder,
    cooperative: &CooperativePlan,
    local: KernelValueId,
) -> Result<KernelValueId, KernelBuildError> {
    let mut slot = local;
    if cooperative.produce_stride != 1 {
        let stride = builder.constant(KernelConstant::Index(cooperative.produce_stride))?;
        slot = builder.binary(BinaryOp::IndexMultiply, slot, stride)?;
    }
    if cooperative.produce_offset != 0 {
        let offset = builder.constant(KernelConstant::Index(cooperative.produce_offset))?;
        slot = builder.binary(BinaryOp::IndexAdd, slot, offset)?;
    }
    Ok(slot)
}

/// Folds the staged partials in ascending participant order.
///
/// **The seed is the first slot, which is admissible only because every slot was
/// written.** `TailPolicy::Exact` and the intrinsic verifier's
/// `grid_threads == work_items` rule make every launched invocation satisfy the
/// producing guard, and the tile's staging-coverage rule proves the participants'
/// writes are a bijection onto the allocation's slots — so slot zero holds a
/// participant's own partial rather than an uninitialized value. That argument is
/// what admits an identity-less family here at all: a seed of `+0.0` would be
/// wrong for a sum at `-0.0` and unavailable to a maximum, and neither needs one.
fn emit_staged_fold(
    builder: &mut KernelBuilder,
    cooperative: &CooperativePlan,
    staging: KernelStagingId,
    combiner: ReductionCombiner,
) -> Result<KernelValueId, KernelBuildError> {
    let base = builder.constant(KernelConstant::Index(cooperative.consume_offset))?;
    let seed = builder.staged_load(staging, base, cooperative.consume_phase)?;
    if cooperative.participants <= 1 {
        return Ok(seed);
    }
    let results = builder.serial_loop(
        SerialLoopSpec {
            start: 1,
            end: cooperative.participants,
        },
        &[seed],
        |builder, parameters| {
            let induction = parameters.induction();
            let accumulator = parameters
                .accumulator(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            let slot = if cooperative.consume_offset == 0 {
                induction
            } else {
                let offset = builder.constant(KernelConstant::Index(cooperative.consume_offset))?;
                builder.binary(BinaryOp::IndexAdd, induction, offset)?
            };
            let staged = builder.staged_load(staging, slot, cooperative.consume_phase)?;
            let folded = builder.binary(combiner.op(), accumulator, staged)?;
            let folded = builder.convert(ConvertOp::CanonicalizeF32Nan, folded)?;
            Ok(vec![folded])
        },
    )?;
    results
        .get(0)
        .ok_or(KernelBuildError::EmptyLoopAccumulators)
}

/// Returns the per-contributor prologue and combiner one reduction folds with.
///
/// `None` for every program that is not a reduction, which is a refusal rather
/// than an absent fold: a cooperative region whose scalar program is not one of
/// these folds nothing, and the schedule verifier already rejects it.
const fn reduction_fold(program: &ScalarProgram) -> Option<ReductionFold> {
    match program {
        ScalarProgram::StrictSerialSum { .. } => Some(ReductionFold {
            prologue: ReductionPrologue::None,
            combiner: ReductionCombiner::F32Add,
        }),
        ScalarProgram::SquaredSerialSum { .. } => Some(ReductionFold {
            prologue: ReductionPrologue::Square,
            combiner: ReductionCombiner::F32Add,
        }),
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            ..
        } => Some(ReductionFold {
            prologue: ReductionPrologue::ScaleBias {
                scale_bits: *scale_bits,
                bias_bits: *bias_bits,
            },
            combiner: ReductionCombiner::F32Add,
        }),
        // The extrema fold: no prologue — the softmax's subtraction and
        // exponential belong to the pointwise pass that consumes this reduction's
        // result — and the combiner that makes it a maximum rather than a sum.
        // Carrying the combiner is what admits it here at all: a cooperative tile
        // over this family stages one participant's maximum and folds the staged
        // set with the same operation, and reusing the addition would produce a
        // structurally identical body computing a different function.
        ScalarProgram::StrictSerialMaximum { .. } => Some(ReductionFold {
            prologue: ReductionPrologue::None,
            combiner: ReductionCombiner::F32Maximum,
        }),
        // Refused for the reason the schedule verifier refuses the topologies
        // themselves: the epilogue applies to the complete fold, so a tile's
        // per-participant share is not a value it may transform.
        ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}

/// The per-contributor expression and the binary operation one reduction folds
/// with.
///
/// Carried as a pair because a cooperative emission needs both at every level it
/// folds at — the participant's own share, the staged set, and the accumulator
/// across rounds — and resolving them separately let one of the three keep an
/// operation the others had moved off.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReductionFold {
    prologue: ReductionPrologue,
    combiner: ReductionCombiner,
}

/// The binary operation one reduction family combines two partials with.
///
/// A closed enum rather than a [`BinaryOp`], which also spells index arithmetic
/// and multiplication: an emission that took a `BinaryOp` could be handed one no
/// reduction family combines with, and nothing would say no. The exhaustive match
/// in [`ReductionCombiner::op`] is what forces a third family to state its own
/// operation rather than inherit whichever it resembles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionCombiner {
    /// The ordered sum every family but the extrema fold combines with.
    F32Add,
    /// The NaN-propagating `Maximum` with `-0.0 < +0.0`.
    F32Maximum,
}

impl ReductionCombiner {
    /// Returns the structured operation this combiner emits.
    const fn op(self) -> BinaryOp {
        match self {
            Self::F32Add => BinaryOp::F32Add,
            Self::F32Maximum => BinaryOp::F32Maximum,
        }
    }
}

/// What one serial fold computes, as opposed to where it reads and writes.
///
/// Carried as one value because the three travel together at every emission
/// site: a family's per-contributor prologue, the value it commits over an empty
/// domain, and the chain it applies to the folded value are its *arithmetic*,
/// while the buffers, the invocation, and the plan are its placement. Passing
/// them separately made the emitter's argument list one no reader could scan.
#[derive(Clone, Copy, Debug)]
struct SerialFold<'a> {
    /// Empty-reduction identity bit pattern the scalar program declares.
    empty_identity_bits: u32,
    /// Expression applied to each contributor before it is combined.
    prologue: ReductionPrologue,
    /// Chain applied to the folded value before it is committed, if any.
    epilogue: Option<&'a PointwiseF32Expression>,
}

/// The elementwise expression applied to each contributor before the fold.
///
/// A typed enum rather than an `Option<(u32, u32)>`, because there are now two
/// prologues and they are not two constant choices of one shape: the scale-bias
/// form is affine in the contributor and the squaring form is quadratic, so no
/// pair of constants makes one express the other. The exhaustive match at the
/// emission site is what forces a third prologue to state its own arithmetic
/// rather than borrowing whichever of these two it resembles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionPrologue {
    /// The contributor enters the fold unchanged.
    None,
    /// `scale * x + bias`, two roundings per contributor.
    ScaleBias {
        /// Scale constant bit pattern.
        scale_bits: u32,
        /// Bias constant bit pattern.
        bias_bits: u32,
    },
    /// `x * x`, one rounding per contributor.
    ///
    /// Emitted as a multiplication of the loaded value by *itself* rather than by
    /// a second load of the same address: one load, one product, and no
    /// assumption that two reads of one element agree.
    Square,
}

fn emit_strict_affine_u4_dequantize(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    requirements: ResourceRequirements,
) -> Result<(), KernelLoweringError> {
    let [codes, scale, zero_point] = plan.reads else {
        return Err(KernelLoweringError::UnsupportedRegion {
            rule: "strict-affine-u4-access-count",
        });
    };
    let codes_buffer = builder.declare_buffer(BufferParameter {
        tensor: codes.tensor,
        component_role: codes.component_role,
        element_type: KernelType::U8,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: plan.read_elements.first().copied().unwrap_or(0),
    })?;
    let scale_buffer = builder.declare_buffer(BufferParameter {
        tensor: scale.tensor,
        component_role: scale.component_role,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: 1,
    })?;
    let zero_buffer = builder.declare_buffer(BufferParameter {
        tensor: zero_point.tensor,
        component_role: zero_point.component_role,
        element_type: KernelType::U8,
        address_space: AddressSpace::Device,
        access: BufferAccess::Read,
        element_count: 1,
    })?;
    let output_buffer = builder.declare_buffer(BufferParameter {
        tensor: plan.write.tensor,
        component_role: plan.write.component_role,
        element_type: KernelType::F32,
        address_space: AddressSpace::Device,
        access: BufferAccess::Write,
        element_count: plan.write_elements,
    })?;
    builder.admit_builtin(Builtin::GlobalInvocationIndex)?;
    builder.numerical(plan.numerical)?;
    builder.requirements(requirements)?;

    let invocation = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
    let active = builder.compare(CompareOp::IndexLessThan, invocation, extent)?;
    builder.predicated(active, |builder| {
        let two = builder.constant(KernelConstant::Index(2))?;
        let carrier_index = builder.binary(BinaryOp::IndexDivide, invocation, two)?;
        let carrier = builder.load(codes_buffer, carrier_index, codes.bounds)?;
        let code = builder.packed_extract(PackedExtractOp::U4LsbZeroTail, carrier, invocation)?;
        let zero_index = builder.constant(KernelConstant::Index(0))?;
        let scale_value = builder.load(scale_buffer, zero_index, scale.bounds)?;
        let zero = builder.load(zero_buffer, zero_index, zero_point.bounds)?;
        let code = builder.convert(ConvertOp::U8ToI32, code)?;
        let zero = builder.convert(ConvertOp::U8ToI32, zero)?;
        let difference = builder.binary(BinaryOp::I32Subtract, code, zero)?;
        let difference = builder.convert(ConvertOp::I32ToF32, difference)?;
        let result = builder.binary(BinaryOp::F32Multiply, difference, scale_value)?;
        builder.store(
            output_buffer,
            invocation,
            result,
            plan.write.bounds,
            plan.ownership,
        )
    })?;
    Ok(())
}

/// Loads one boundary value per read, through each read's own addressing.
///
/// Through the read's own relation rather than at the invocation index.
/// Loading at the invocation directly was correct while every pointwise read
/// was `LinearIdentity`, and it is exactly the check that keeps passing for the
/// wrong reason once a second relation exists: a structural read would have
/// addressed its operand densely and returned a plausible tensor that is the
/// wrong one. [`ReadAddressing::Identity`] still emits the invocation itself,
/// so every dense region's body is unchanged.
///
/// **One slot per access position, and `None` at a gather's address operand.**
/// A scalar leaf names its read by region-local ordinal, so the ordinal indexes
/// this list directly; compacting it would shift every later leaf onto its
/// neighbour the moment an address operand appeared before one. The address
/// operand yields no value because it is not one — it is loaded inside the
/// offset of the gather that owns it, at the U32 type its buffer declares, and
/// an expression naming it is a typed refusal here rather than a `u32` reaching
/// `f32` arithmetic.
fn emit_pointwise_loads(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffers: &[KernelBufferId],
    invocation: KernelValueId,
) -> Result<Vec<Option<KernelValueId>>, KernelBuildError> {
    let missing = || KernelBuildError::InvalidHandle {
        entity: super::error::KernelEntityKind::Buffer,
    };
    let mut inputs = Vec::with_capacity(read_buffers.len());
    for (position, (buffer, read)) in read_buffers.iter().zip(plan.reads).enumerate() {
        let addressing = plan.addressing.get(position).ok_or_else(missing)?;
        let value = match addressing {
            ReadAddressing::GatherAddress => None,
            ReadAddressing::Gather(gather) => {
                let operand = read_buffers.get(gather.index_read).ok_or_else(missing)?;
                let bounds = plan
                    .reads
                    .get(gather.index_read)
                    .ok_or_else(missing)?
                    .bounds;
                let offset = emit_gather_offset(builder, gather, *operand, bounds, invocation)?;
                Some(builder.load(*buffer, offset, read.bounds)?)
            }
            ReadAddressing::Identity
            | ReadAddressing::LiveRowMajor { .. }
            | ReadAddressing::LiveContracted { .. }
            | ReadAddressing::BlockedContraction(_)
            | ReadAddressing::Linearized(_)
            | ReadAddressing::Partitioned { .. } => {
                let offset = emit_offset(builder, addressing, invocation, None)?;
                Some(builder.load(*buffer, offset, read.bounds)?)
            }
        };
        inputs.push(value);
    }
    Ok(inputs)
}

/// Emits the element offset of one data-dependent read.
///
/// The address is `direct + coordinate * stride`, where `coordinate` is the U32
/// this invocation loads from the gather's own index operand, widened to the
/// index role by the one named conversion that does it. The load comes first
/// because the indirection *is* the relation: a reader of the emitted body
/// meets the coordinate it depends on before the direct terms that surround it.
///
/// **The scale is emitted whenever the stride is not one, including zero.** A
/// zero stride is the exact row-major contribution of a gathered axis whose
/// source carries a later empty axis, so multiplying by it is arithmetic rather
/// than a special case; skipping it — the shape the `> 1` guard elsewhere would
/// take — would leave the raw loaded coordinate standing in the sum.
///
/// The direct sum is folded in only when there is one. A gather whose source is
/// rank one has no direct coordinate at all, and adding a constant zero to its
/// address would put an operation in the canonical body that a structurally
/// compared producer kernel would have to reproduce exactly.
fn emit_gather_offset(
    builder: &mut KernelBuilder,
    gather: &GatherAddressing,
    operand: KernelBufferId,
    bounds: BoundsWitnessId,
    invocation: KernelValueId,
) -> Result<KernelValueId, KernelBuildError> {
    let coordinate_offset = emit_offset(builder, &gather.index, invocation, None)?;
    let address = builder.load(operand, coordinate_offset, bounds)?;
    let coordinate = builder.convert(ConvertOp::U32ToIndex, address)?;
    let mut total = if gather.gathered_stride == 1 {
        coordinate
    } else {
        let stride = builder.constant(KernelConstant::Index(gather.gathered_stride))?;
        builder.binary(BinaryOp::IndexMultiply, coordinate, stride)?
    };
    if !gather.direct.is_empty() {
        let direct = emit_offset(
            builder,
            &ReadAddressing::Linearized(gather.direct.clone()),
            invocation,
            None,
        )?;
        total = builder.binary(BinaryOp::IndexAdd, total, direct)?;
    }
    Ok(total)
}

/// Emits the scalar body of a pointwise expression over its loaded inputs.
///
/// `inputs` is indexed by the leaf's own ordinal, not by the order the leaves
/// appear: canonicalization orders nodes by root-first discovery, so a leaf's
/// position among the nodes says nothing about which tensor it reads. An ordinal
/// with no loaded value is a region whose reads and expression disagree, which
/// the schedule verifier rejects — this reports it as an invalid handle rather
/// than reading whichever value sits at that index.
///
/// A slot holding `None` is that same disagreement in its one reachable
/// spelling: the ordinal names a gather's address operand, which produces
/// coordinates rather than a value. It is refused on the same rule rather than
/// resolved to a neighbouring leaf.
fn emit_pointwise(
    builder: &mut KernelBuilder,
    expression: &PointwiseF32Expression,
    inputs: &[Option<KernelValueId>],
) -> Result<KernelValueId, KernelBuildError> {
    let mut values = Vec::with_capacity(expression.nodes().len());
    for node in expression.nodes() {
        let value = match node {
            PointwiseF32Node::Input { access } => usize::try_from(access.get())
                .ok()
                .and_then(|position| inputs.get(position).copied().flatten())
                .ok_or(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Buffer,
                })?,
            PointwiseF32Node::Constant { bits } => {
                builder.constant(KernelConstant::F32Bits(*bits))?
            }
            PointwiseF32Node::Add { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Add, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Divide { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Divide, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            // The exponential's result is canonicalized on the same rule every
            // other arithmetic result is: the numerical realization installs one
            // canonical arithmetic NaN payload, and an elementary function that
            // skipped it would deliver a payload the contract does not name.
            PointwiseF32Node::Exp { argument } => {
                let argument = pointwise_value(&values, *argument)?;
                let result = builder.unary(UnaryOp::F32Exp, argument)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Rsqrt { argument } => {
                let argument = pointwise_value(&values, *argument)?;
                let result = builder.unary(UnaryOp::F32Rsqrt, argument)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
            PointwiseF32Node::Multiply { lhs, rhs } => {
                let lhs = pointwise_value(&values, *lhs)?;
                let rhs = pointwise_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::F32Multiply, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeF32Nan, result)?
            }
        };
        values.push(value);
    }
    pointwise_value(&values, expression.root())
}

/// Emits the scalar body of a `bf16` pointwise expression over its loaded inputs.
///
/// The `f32` emission's structure, with two deliberate differences that are the
/// family's rather than this lowering's.
///
/// **Every arithmetic result is canonicalized with
/// [`ConvertOp::CanonicalizeBf16Nan`], never the `f32` conversion.** The two
/// install different-width patterns, and `CanonicalizeF32Nan` is not even
/// spellable over a `bf16` value — its `source_type` is `F32`, so the builder
/// refuses it with a type mismatch. That is what makes "the BF16 canonicalization
/// rather than `CanonicalizeF32Nan`" a property of the vocabulary and not a rule
/// this function has to be trusted to follow.
///
/// **The constant is a [`KernelConstant::Bf16Bits`], carrying the node's own
/// sixteen-bit payload unchanged.** A constant is not an arithmetic result, so it
/// is deliberately *not* canonicalized: `tiler::constant-bf16@1` declares its
/// payload preserved exactly, and the family's canonicalization applies to
/// arithmetic results alone.
fn emit_pointwise_bf16(
    builder: &mut KernelBuilder,
    expression: &PointwiseBf16Expression,
    inputs: &[Option<KernelValueId>],
) -> Result<KernelValueId, KernelBuildError> {
    let mut values = Vec::with_capacity(expression.nodes().len());
    for node in expression.nodes() {
        let value = match node {
            PointwiseBf16Node::Input { access } => usize::try_from(access.get())
                .ok()
                .and_then(|position| inputs.get(position).copied().flatten())
                .ok_or(KernelBuildError::InvalidHandle {
                    entity: super::error::KernelEntityKind::Buffer,
                })?,
            PointwiseBf16Node::Constant { bits } => {
                builder.constant(KernelConstant::Bf16Bits(*bits))?
            }
            PointwiseBf16Node::Add { lhs, rhs } => {
                let lhs = pointwise_bf16_value(&values, *lhs)?;
                let rhs = pointwise_bf16_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::Bf16Add, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeBf16Nan, result)?
            }
            PointwiseBf16Node::Multiply { lhs, rhs } => {
                let lhs = pointwise_bf16_value(&values, *lhs)?;
                let rhs = pointwise_bf16_value(&values, *rhs)?;
                let result = builder.binary(BinaryOp::Bf16Multiply, lhs, rhs)?;
                builder.convert(ConvertOp::CanonicalizeBf16Nan, result)?
            }
        };
        values.push(value);
    }
    pointwise_bf16_value(&values, expression.root())
}

fn pointwise_bf16_value(
    values: &[KernelValueId],
    node: crate::schedule::PointwiseBf16NodeId,
) -> Result<KernelValueId, KernelBuildError> {
    usize::try_from(node.index())
        .ok()
        .and_then(|index| values.get(index).copied())
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Value,
        })
}

fn pointwise_value(
    values: &[KernelValueId],
    node: crate::schedule::PointwiseF32NodeId,
) -> Result<KernelValueId, KernelBuildError> {
    usize::try_from(node.index())
        .ok()
        .and_then(|index| values.get(index).copied())
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Value,
        })
}

fn emit_scale_bias(
    builder: &mut KernelBuilder,
    value: KernelValueId,
    scale_bits: u32,
    bias_bits: u32,
) -> Result<KernelValueId, KernelBuildError> {
    let scale = builder.constant(KernelConstant::F32Bits(scale_bits))?;
    let product = builder.binary(BinaryOp::F32Multiply, value, scale)?;
    let product = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
    let bias = builder.constant(KernelConstant::F32Bits(bias_bits))?;
    let biased = builder.binary(BinaryOp::F32Add, product, bias)?;
    builder.convert(ConvertOp::CanonicalizeF32Nan, biased)
}

/// Emits one contributor's prologue expression, or the contributor unchanged.
///
/// The match is exhaustive over a crate-private enum, so a third prologue is a
/// build error here rather than a silent reuse of whichever arm it resembles.
fn emit_prologue(
    builder: &mut KernelBuilder,
    value: KernelValueId,
    prologue: ReductionPrologue,
) -> Result<KernelValueId, KernelBuildError> {
    match prologue {
        ReductionPrologue::None => Ok(value),
        ReductionPrologue::ScaleBias {
            scale_bits,
            bias_bits,
        } => emit_scale_bias(builder, value, scale_bits, bias_bits),
        // The loaded value multiplied by itself, so the square rests on one read
        // rather than on two reads agreeing. One rounding, which is what the
        // semantic reference states for `q_i = x_i * x_i`.
        ReductionPrologue::Square => {
            let square = builder.binary(BinaryOp::F32Multiply, value, value)?;
            builder.convert(ConvertOp::CanonicalizeF32Nan, square)
        }
    }
}

/// Emits the guarded body of one serial fold, with its per-contributor prologue
/// and — where the family carries one — the epilogue over the folded value.
///
/// **The epilogue is applied on every path out of the fold, including the empty
/// one.** The program is "fold, then epilogue", so what the epilogue transforms
/// is whatever the fold's value is: the identity constant over an empty
/// contributor domain, the seed over a singleton, the accumulator otherwise.
/// Applying it on two of the three paths would make the empty and singleton cases
/// compute different functions from the general one, which no scalar program in
/// this vocabulary states.
///
/// It emits no result-boundary canonicalization of its own: every node
/// [`emit_pointwise`] emits is already followed by one, so the value the store
/// receives carries the canonical payload — and an epilogue that computes
/// *something* is what the schedule verifier requires, so the "no nodes at all"
/// case where that would not hold is unreachable.
fn emit_reduction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read: (KernelBufferId, BoundsWitnessId),
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
    fold: SerialFold<'_>,
) -> Result<(), KernelBuildError> {
    let SerialFold {
        empty_identity_bits,
        prologue,
        epilogue,
    } = fold;
    let (read_buffer, read_bounds) = read;
    let addressing = plan
        .addressing
        .first()
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Buffer,
        })?;
    if plan.contributors == 0 {
        let identity = builder.constant(KernelConstant::F32Bits(empty_identity_bits))?;
        let identity = emit_fold_epilogue(builder, identity, epilogue)?;
        return builder.store(
            write_buffer,
            invocation,
            identity,
            plan.write_bounds,
            plan.ownership,
        );
    }
    let first_offset = emit_offset(builder, addressing, invocation, None)?;
    let first = builder.load(read_buffer, first_offset, read_bounds)?;
    let seed = emit_prologue(builder, first, prologue)?;
    // A single contributor supplies the whole strict-serial value, but the
    // reduction still canonicalizes at its result boundary: ADR 0055 and the
    // numerical contract both require that boundary rule "even when the
    // contributor sequence is a singleton", so an uncombined input payload
    // cannot leak its NaN bits through an arithmetic reduction.
    //
    // The conversion is what realizes the rule here. Emitting a loop would need
    // an empty iteration range, and combining with the reduction identity would
    // change the observable sign of a negative zero, whereas canonicalization
    // rewrites a NaN and leaves every other payload — including `-0.0` — alone.
    //
    // It is emitted exactly where the boundary value is an uncombined input,
    // which is the leak the rule names. The fold already applies the conversion
    // after each combine, and a prologue already applies it to the scaled seed,
    // so those boundaries are canonical without a second one.
    let total = if plan.contributors == 1 {
        match prologue {
            ReductionPrologue::ScaleBias { .. } | ReductionPrologue::Square => seed,
            ReductionPrologue::None => builder.convert(ConvertOp::CanonicalizeF32Nan, seed)?,
        }
    } else {
        let results = builder.serial_loop(
            SerialLoopSpec {
                start: 1,
                end: plan.contributors,
            },
            &[seed],
            |builder, parameters| {
                let induction = parameters.induction();
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                let offset = emit_offset(builder, addressing, invocation, Some(induction))?;
                let loaded = builder.load(read_buffer, offset, read_bounds)?;
                let contributor = emit_prologue(builder, loaded, prologue)?;
                let sum = builder.binary(BinaryOp::F32Add, accumulator, contributor)?;
                let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
                Ok(vec![sum])
            },
        )?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?
    };
    let total = emit_fold_epilogue(builder, total, epilogue)?;
    builder.store(
        write_buffer,
        invocation,
        total,
        plan.write_bounds,
        plan.ownership,
    )
}

/// Applies one fold's epilogue to its value, or returns the value unchanged.
///
/// The folded value is bound to local access zero, which is the sole leaf the
/// schedule verifier admits: this region reads one boundary tensor and the
/// epilogue names none, so the ordinal indexes the fold's own accumulator rather
/// than a loaded buffer. A second leaf finds no value and is reported as an
/// invalid handle rather than reading whichever one sits beside it.
fn emit_fold_epilogue(
    builder: &mut KernelBuilder,
    value: KernelValueId,
    epilogue: Option<&PointwiseF32Expression>,
) -> Result<KernelValueId, KernelBuildError> {
    match epilogue {
        Some(expression) => emit_pointwise(builder, expression, &[Some(value)]),
        None => Ok(value),
    }
}

/// Emits the guarded body of one strict serial `Maximum` reduction.
///
/// The same shape as [`emit_reduction`] with two deliberate differences, and both
/// are the extrema family's rather than this lowering's.
///
/// **There is no empty-domain path.** A sum commits the identity its scalar
/// program declares when the reduced domain is empty; the extrema family declares
/// none — what a maximum over no contributors means is a declaration no
/// registered operation embedding this fold has made — so there is no value to
/// commit and the only correct answer is to refuse. That is a statement about the
/// empty case and not about the family's algebra, which
/// [`ScalarProgram::StrictSerialMaximum`] keeps apart: `-inf` *is* neutral for
/// this combiner, and no emission here pads with it. The schedule verifier
/// refuses such a region before it reaches here, and this restates the refusal
/// where the lowering could still emit — a fold over zero contributors is exactly
/// the empty iteration range [`KernelBuildError::InvalidLoopRange`] names.
///
/// **There is no prologue.** The softmax's subtraction and exponential belong to
/// the pointwise pass that *consumes* this reduction's result, not to the fold
/// itself, so folding them in here would make one region carry two iteration
/// domains — the same reason `SquaredSerialSum` carries no epilogue.
///
/// The result-boundary canonicalization is emitted on the single-contributor path
/// exactly as the bare sum's is, and for the same reason: the boundary value is
/// then an uncombined load whose NaN payload would otherwise leak through an
/// arithmetic reduction. The combining path needs none of its own, because every
/// combine is already followed by one.
fn emit_maximum_reduction(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read: (KernelBufferId, BoundsWitnessId),
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
) -> Result<(), KernelBuildError> {
    let (read_buffer, read_bounds) = read;
    let addressing = plan
        .addressing
        .first()
        .ok_or(KernelBuildError::InvalidHandle {
            entity: super::error::KernelEntityKind::Buffer,
        })?;
    if plan.contributors == 0 {
        return Err(KernelBuildError::InvalidLoopRange { start: 0, end: 0 });
    }
    let first_offset = emit_offset(builder, addressing, invocation, None)?;
    let seed = builder.load(read_buffer, first_offset, read_bounds)?;
    let total = if plan.contributors == 1 {
        builder.convert(ConvertOp::CanonicalizeF32Nan, seed)?
    } else {
        let results = builder.serial_loop(
            SerialLoopSpec {
                start: 1,
                end: plan.contributors,
            },
            &[seed],
            |builder, parameters| {
                let induction = parameters.induction();
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                let offset = emit_offset(builder, addressing, invocation, Some(induction))?;
                let loaded = builder.load(read_buffer, offset, read_bounds)?;
                let combined = builder.binary(BinaryOp::F32Maximum, accumulator, loaded)?;
                let combined = builder.convert(ConvertOp::CanonicalizeF32Nan, combined)?;
                Ok(vec![combined])
            },
        )?;
        results
            .get(0)
            .ok_or(KernelBuildError::EmptyLoopAccumulators)?
    };
    builder.store(
        write_buffer,
        invocation,
        total,
        plan.write_bounds,
        plan.ownership,
    )
}

/// Splits a partial pass's invocation index into its output and partition parts.
///
/// One invocation covers one (output, partition) pair, laid out so the
/// partition ordinal is the innermost coordinate — which is also what makes the
/// partial tensor's linear write index equal to the invocation index. Returns
/// the linear output coordinate and the partition's first contributor ordinal.
///
/// A single partition needs neither operation: the output coordinate is the
/// invocation and the partition ordinal is constantly zero, so emitting the
/// division and remainder would put two provably identity operations into the
/// canonical body a refinement gate compares against.
fn split_partitioned_invocation(
    builder: &mut KernelBuilder,
    invocation: KernelValueId,
    partitions: u64,
) -> Result<(KernelValueId, Option<KernelValueId>), KernelBuildError> {
    if partitions <= 1 {
        return Ok((invocation, None));
    }
    let extent = builder.constant(KernelConstant::Index(partitions))?;
    let output = builder.binary(BinaryOp::IndexDivide, invocation, extent)?;
    let partition = builder.binary(BinaryOp::IndexModulo, invocation, extent)?;
    Ok((output, Some(partition)))
}

/// Emits the contributor ordinal one partitioned load addresses.
///
/// The ordinal is
/// `round * contributors_per_round + partition * contributors_per_partition +
/// within`, which is the contiguous range participant `partition` owns on round
/// `round` in the region's declared contributor order — the range at index
/// `round * participants + partition`. `within` is `None` for the seed load,
/// whose position inside the range is zero, and `round` is `None` wherever the
/// ordinal is constantly zero: the peeled round zero, and the multi-pass partial
/// pass, which has no round dimension.
///
/// `None` comes back only when the whole ordinal is provably zero — round zero of
/// a single partition seeding at its first contributor — so the caller drops
/// every contributor term exactly as the unsplit lowering does.
fn emit_partition_contributor(
    builder: &mut KernelBuilder,
    round: Option<RoundOrdinal>,
    partition: Option<KernelValueId>,
    within: Option<KernelValueId>,
    contributors_per_partition: u64,
) -> Result<Option<KernelValueId>, KernelBuildError> {
    let base = match partition {
        None => None,
        Some(partition) => {
            if contributors_per_partition <= 1 {
                Some(partition)
            } else {
                let stride = builder.constant(KernelConstant::Index(contributors_per_partition))?;
                Some(builder.binary(BinaryOp::IndexMultiply, partition, stride)?)
            }
        }
    };
    let base = match round {
        None => base,
        Some(round) => {
            let scaled = if round.contributors_per_round <= 1 {
                round.value
            } else {
                let stride =
                    builder.constant(KernelConstant::Index(round.contributors_per_round))?;
                builder.binary(BinaryOp::IndexMultiply, round.value, stride)?
            };
            Some(match base {
                None => scaled,
                Some(base) => builder.binary(BinaryOp::IndexAdd, scaled, base)?,
            })
        }
    };
    Ok(match (base, within) {
        (None, within) => within,
        (Some(base), None) => Some(base),
        (Some(base), Some(within)) => Some(builder.binary(BinaryOp::IndexAdd, base, within)?),
    })
}

/// Emits `free * S + contributor` for one live-contraction operand.
///
/// The free terms are the static output coordinates the schedule already
/// linearized. `S` is the named live input-axis operand, never a literal, so
/// neighbouring extents share the kernel and a baked neighbour does not.
fn emit_live_contracted_offset(
    builder: &mut KernelBuilder,
    free: &[OffsetTerm],
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
    live: KernelValueId,
) -> Result<KernelValueId, KernelBuildError> {
    if free.is_empty() {
        return match contributor {
            None => builder.constant(KernelConstant::Index(0)),
            Some(index) => Ok(index),
        };
    }
    let free_offset = emit_offset(
        builder,
        &ReadAddressing::Linearized(free.to_vec()),
        invocation,
        None,
    )?;
    let scaled = builder.binary(BinaryOp::IndexMultiply, free_offset, live)?;
    match contributor {
        None => Ok(scaled),
        Some(index) => builder.binary(BinaryOp::IndexAdd, scaled, index),
    }
}

/// Emits the element offset of one read access.
///
/// `contributor` is `None` for the seed load, whose contributor coordinate is
/// zero; every contributor term then vanishes exactly.
fn emit_offset(
    builder: &mut KernelBuilder,
    addressing: &ReadAddressing,
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
) -> Result<KernelValueId, KernelBuildError> {
    let (terms, output, contributor) = match addressing {
        ReadAddressing::Identity => return Ok(invocation),
        // Three forms with no linear root to decode. The blocked contraction is
        // here for a reason worth stating: routing it through this function would
        // silently emit the divide and modulo it exists to avoid, so the refusal
        // is what keeps that cost out of a kernel with a retained timing.
        //
        // The two gather forms are here for a different reason. A gather's
        // address needs the builder to emit a load before any sum exists, which
        // this function's signature cannot express, so it is built by
        // `emit_gather_offset` instead; and an address operand has no offset of
        // its own at all — the gather that owns it holds one. Reaching either
        // here means a caller addressed a data-dependent read as though it were
        // arithmetic, which is a refusal rather than an approximation.
        ReadAddressing::LiveRowMajor { .. }
        | ReadAddressing::LiveContracted { .. }
        | ReadAddressing::BlockedContraction(_)
        | ReadAddressing::Gather(_)
        | ReadAddressing::GatherAddress => {
            return Err(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Value,
            });
        }
        ReadAddressing::Linearized(terms) => (terms, invocation, contributor),
        ReadAddressing::Partitioned {
            terms,
            partitions,
            contributors_per_partition,
        } => {
            let (output, base) = split_partitioned_invocation(builder, invocation, *partitions)?;
            let contributor = emit_partition_contributor(
                builder,
                None,
                base,
                contributor,
                *contributors_per_partition,
            )?;
            (terms, output, contributor)
        }
    };
    let mut total: Option<KernelValueId> = None;
    for term in terms {
        let root = match term.root {
            OffsetRoot::Output => output,
            OffsetRoot::Contributor => match contributor {
                Some(value) => value,
                None => continue,
            },
        };
        let mut value = root;
        if term.divisor > 1 {
            let divisor = builder.constant(KernelConstant::Index(term.divisor))?;
            value = builder.binary(BinaryOp::IndexDivide, value, divisor)?;
        }
        if let Some(modulus) = term.modulus {
            let modulus = builder.constant(KernelConstant::Index(modulus))?;
            value = builder.binary(BinaryOp::IndexModulo, value, modulus)?;
        }
        // Between the wrap and the stride, which is the only correct place: the
        // mirror is stated on the axis coordinate, so mirroring before the wrap
        // would reflect the wrong quantity and mirroring after the stride would
        // reflect a scaled one.
        if let Some(extent) = term.mirror {
            let last = builder.constant(KernelConstant::Index(extent.saturating_sub(1)))?;
            value = builder.binary(BinaryOp::IndexSubtract, last, value)?;
        }
        if term.stride > 1 {
            let stride = builder.constant(KernelConstant::Index(term.stride))?;
            value = builder.binary(BinaryOp::IndexMultiply, value, stride)?;
        }
        total = Some(match total {
            Some(accumulated) => builder.binary(BinaryOp::IndexAdd, accumulated, value)?,
            None => value,
        });
    }
    match total {
        Some(value) => Ok(value),
        None => builder.constant(KernelConstant::Index(0)),
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockedCoordinate, BlockedTerm, KernelDiagnostic, blocked_contraction_terms};
    use crate::schedule::ContractionAxisSource::{Contracted, Output};
    use crate::shape::Shape;

    /// The output every case below contracts into: `[M, N] = [32, 48]`.
    ///
    /// `M`, `N`, and `K` are deliberately three different values, so a term that
    /// picked up the wrong extent as its stride cannot compare equal to the right
    /// one by coincidence.
    fn output() -> Shape {
        Shape::from_dims([32, 48])
    }

    fn term(coordinate: BlockedCoordinate, stride: u64) -> BlockedTerm {
        BlockedTerm { coordinate, stride }
    }

    /// `[M, K]`: the row coordinate carries the contracted extent as its stride.
    #[test]
    fn a_row_major_left_operand_scales_the_row_by_the_contracted_extent() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([32, 16]),
            &output(),
            &[Output { position: 0 }, Contracted { position: 0 }],
        )
        .expect("the row-major left layout is expressible");
        assert_eq!(
            terms,
            vec![
                term(BlockedCoordinate::Row, 16),
                term(BlockedCoordinate::Contracted, 1),
            ]
        );
    }

    /// `[K, M]`: the same two coordinates, with the strides exchanged.
    ///
    /// This is the layout the emission used to address as if it were `[M, K]`,
    /// and the exchange here is the whole content of that defect.
    #[test]
    fn a_transposed_left_operand_scales_the_contracted_index_by_the_row_extent() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([16, 32]),
            &output(),
            &[Contracted { position: 0 }, Output { position: 0 }],
        )
        .expect("the transposed left layout is expressible");
        assert_eq!(
            terms,
            vec![
                term(BlockedCoordinate::Contracted, 32),
                term(BlockedCoordinate::Row, 1),
            ]
        );
    }

    /// `[K, N]`: the attention value structure's middle-contracted orientation.
    #[test]
    fn a_transposed_right_operand_scales_the_contracted_index_by_the_column_extent() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([16, 48]),
            &output(),
            &[Contracted { position: 0 }, Output { position: 1 }],
        )
        .expect("the transposed right layout is expressible");
        assert_eq!(
            terms,
            vec![
                term(BlockedCoordinate::Contracted, 48),
                term(BlockedCoordinate::Column, 1),
            ]
        );
    }

    /// Each leading batch axis becomes its own coordinate, at its own stride.
    ///
    /// The batch axes are numbered by their *output* position, so an operand
    /// reading two of them yields `Batch(0)` and `Batch(1)` rather than two terms
    /// a later lookup could not tell apart. The four extents are pairwise
    /// distinct and so are the four strides, so a term that took the wrong axis's
    /// stride cannot compare equal to the right one by coincidence.
    ///
    /// This subject was `an_output_coordinate_outside_the_participant_pair_is_refused`
    /// before the batched block was lowered, and it asserted exactly the refusal
    /// this now replaces — the population it named has moved from unaddressable
    /// to addressed, so the assertion had to move with it rather than be relaxed.
    #[test]
    fn each_leading_batch_axis_becomes_its_own_coordinate() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([8, 2, 32, 16]),
            &Shape::from_dims([8, 2, 32, 48]),
            &[
                Output { position: 0 },
                Output { position: 1 },
                Output { position: 2 },
                Contracted { position: 0 },
            ],
        )
        .expect("a batched operand layout is expressible");
        assert_eq!(
            terms,
            vec![
                term(BlockedCoordinate::Batch(0), 2 * 32 * 16),
                term(BlockedCoordinate::Batch(1), 32 * 16),
                term(BlockedCoordinate::Row, 16),
                term(BlockedCoordinate::Contracted, 1),
            ]
        );
    }

    /// An operand may read some batch axes and not others.
    ///
    /// The unread axis contributes no term at all, which is what makes the read
    /// invariant in it — the broadcast a shared key or value tensor needs. The
    /// axis that *is* read keeps its own output position, so the term names
    /// `Batch(0)` even though it is the operand's only batch term.
    #[test]
    fn an_unread_batch_axis_contributes_no_term() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([8, 32, 16]),
            &Shape::from_dims([8, 2, 32, 48]),
            &[
                Output { position: 0 },
                Output { position: 2 },
                Contracted { position: 0 },
            ],
        )
        .expect("an operand reading one batch axis is expressible");
        assert_eq!(
            terms,
            vec![
                term(BlockedCoordinate::Batch(0), 32 * 16),
                term(BlockedCoordinate::Row, 16),
                term(BlockedCoordinate::Contracted, 1),
            ]
        );
    }

    /// An output position past the trailing pair names no axis and is refused.
    ///
    /// **This is what still fails, and the case is reachable.** The arms above it
    /// take `position < row`, `position == row`, and `position == column`; a
    /// rank-four output's column is position three, so position four falls
    /// through to the refusal. The operand shape is well formed and its source
    /// count matches, so this refusal is the position's alone.
    #[test]
    fn an_output_position_past_the_trailing_pair_is_refused() {
        assert_eq!(
            blocked_contraction_terms(
                &Shape::from_dims([8, 32, 16]),
                &Shape::from_dims([8, 2, 32, 48]),
                &[
                    Output { position: 4 },
                    Output { position: 2 },
                    Contracted { position: 0 },
                ],
            ),
            Err(KernelDiagnostic::CooperativeLoweringShape)
        );
    }

    /// A second contracted coordinate is refused: one round loop, one induction.
    #[test]
    fn a_contracted_coordinate_beyond_the_first_is_refused() {
        assert_eq!(
            blocked_contraction_terms(
                &Shape::from_dims([32, 4, 16]),
                &output(),
                &[
                    Output { position: 0 },
                    Contracted { position: 1 },
                    Contracted { position: 0 },
                ],
            ),
            Err(KernelDiagnostic::CooperativeLoweringShape)
        );
    }

    /// An axis whose extent is one contributes no term: its coordinate is zero.
    #[test]
    fn a_unit_extent_axis_contributes_no_term() {
        let terms = blocked_contraction_terms(
            &Shape::from_dims([1, 16]),
            &Shape::from_dims([1, 48]),
            &[Output { position: 0 }, Contracted { position: 0 }],
        )
        .expect("a unit row extent is expressible");
        assert_eq!(terms, vec![term(BlockedCoordinate::Contracted, 1)]);
    }
}
