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
    Access, BoundsWitnessId, CanonicalScheduledRegionIdentity, LogicalAccess, NumericalRealization,
    OwnershipWitnessId, PointwiseBf16Expression, PointwiseBf16Node, PointwiseF32Expression,
    PointwiseF32Node, ReductionPass, ReductionTopology, ResourceRequirements, ScalarProgram,
    ScheduledRegion, TensorRole, VerifiedScheduledRegion, contributor_count, element_count,
};
use crate::shape::Shape;

use super::builder::KernelBuilder;
use super::error::{KernelBuildError, KernelDiagnostic, KernelLoweringError};
use super::handles::{KernelBufferId, KernelStagingId, KernelValueId};
use super::model::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BufferAccess, BufferParameter, Builtin,
    CompareOp, ConvertOp, ExecutionScope, KernelConstant, KernelData, KernelType, MemoryScope,
    PackedExtractOp, SerialLoopSpec, StagingParameter, UnaryOp, VerifiedKernel,
    region_element_type,
};
use super::verify::{access_elements, boundary_accesses};

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

/// How the read access computes its element offset.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ReadAddressing {
    /// One iteration coordinate addresses one linear element position.
    Identity,
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
    let (reads, write) = boundary_accesses(schedule)?;
    let read = reads.first().ok_or(KernelDiagnostic::ScheduleAccessCount)?;
    // The contributors *one invocation* combines. For a partial pass that is
    // its own partition's share, not the whole reduction's sequence, which is
    // exactly the difference the split exists to create.
    let contributors = match &schedule.schedule.reduction {
        ReductionTopology::None => 0,
        ReductionTopology::Serial { axes, .. }
        | ReductionTopology::MultiPass {
            pass: ReductionPass::Final,
            axes,
            ..
        } => contributor_count(axes, &read.map).map_err(|_| KernelDiagnostic::ContributorDomain)?,
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            partition,
            ..
        } => partition.contributors_per_partition,
        // The contracted index space, which the topology states because no
        // single operand's map determines it.
        ReductionTopology::Contraction {
            contracted_shape, ..
        } => crate::schedule::element_count(contracted_shape)
            .map_err(|_| KernelDiagnostic::ElementCountOverflow)?,
        // What one participant folds in the producing phase, which the split
        // states directly for the reason a partial pass's does: counting the
        // access's contributors here would count the whole sequence.
        ReductionTopology::CooperativeWorkgroup { partition, .. } => {
            partition.contributors_per_partition
        }
        // Representable, not lowered. The Metal body is a different ticket;
        // refusing here is what keeps this path from emitting a direct
        // contraction over the same work items.
        ReductionTopology::CooperativeContraction { .. } => {
            return Err(KernelDiagnostic::CooperativeLoweringShape);
        }
    };
    // The strict-affine decode addresses its three role-scoped components by the
    // invocation index directly, so it consults no coordinate map.
    let addressing = if matches!(
        &schedule.index.scalar_program,
        ScalarProgram::StrictAffineU4Dequantize { .. }
    ) {
        vec![ReadAddressing::Identity; reads.len()]
    } else {
        reads
            .iter()
            .map(|read| addressing(read, &schedule.schedule.reduction))
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok(CanonicalPlan {
        scalar: &schedule.index.scalar_program,
        reads,
        write,
        numerical: schedule.index.numerical,
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
    let ReductionTopology::CooperativeWorkgroup {
        partition, tile, ..
    } = &schedule.schedule.reduction
    else {
        return Ok(None);
    };
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
fn addressing(
    read: &Access,
    reduction: &ReductionTopology,
) -> Result<ReadAddressing, KernelDiagnostic> {
    match &read.map {
        LogicalAccess::LinearIdentity => Ok(ReadAddressing::Identity),
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
                    partition,
                    ..
                } => Ok(ReadAddressing::Partitioned {
                    terms,
                    partitions: partition.partitions,
                    contributors_per_partition: partition.contributors_per_partition,
                }),
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
                | ReductionTopology::MultiPass { .. } => Ok(ReadAddressing::Linearized(terms)),
            }
        }
        LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
            ..
        } => Ok(ReadAddressing::Linearized(linearize_contraction_operand(
            operand_shape,
            output_shape,
            contracted_shape,
            sources,
        )?)),
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
    }
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
    // One buffer per read, in access order. The component role stays `None`
    // rather than being copied from the access: these families read dense
    // values, and `verify_signature` compares the two, so copying it would make
    // that comparison agree with itself instead of checking anything.
    //
    // The element type is derived from the region's scalar program through the
    // same authority `verify_signature` reads, so a widened dtype cannot declare
    // one type here and be checked against another there.
    let element_type = region_element_type(plan.scalar);
    let mut read_buffers = Vec::with_capacity(plan.reads.len());
    for (read, elements) in plan.reads.iter().zip(&plan.read_elements) {
        read_buffers.push(builder.declare_buffer(BufferParameter {
            tensor: read.tensor,
            component_role: None,
            element_type,
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

    let invocation = builder.builtin(Builtin::GlobalInvocationIndex)?;
    let extent = builder.constant(KernelConstant::Index(plan.work_items))?;
    let active = builder.compare(CompareOp::IndexLessThan, invocation, extent)?;
    builder.predicated(active, |builder| {
        emit_guarded(builder, plan, &read_buffers, write_buffer, invocation)
    })?;
    Ok(())
}

fn emit_guarded(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    read_buffers: &[KernelBufferId],
    write_buffer: KernelBufferId,
    invocation: KernelValueId,
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
            let mut inputs = Vec::with_capacity(read_buffers.len());
            for (position, (buffer, read)) in read_buffers.iter().zip(plan.reads).enumerate() {
                // Through the read's own addressing rather than at the
                // invocation index. Loading at the invocation directly was
                // correct while every pointwise read was `LinearIdentity`, and
                // it is exactly the check that keeps passing for the wrong
                // reason once a second relation exists: a structural read would
                // have addressed its operand densely and returned a plausible
                // tensor that is the wrong one. `Identity` still emits the
                // invocation itself, so every dense region's body is unchanged.
                let addressing =
                    plan.addressing
                        .get(position)
                        .ok_or(KernelBuildError::InvalidHandle {
                            entity: super::error::KernelEntityKind::Buffer,
                        })?;
                let offset = emit_offset(builder, addressing, invocation, None)?;
                inputs.push(builder.load(*buffer, offset, read.bounds)?);
            }
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
        // decide what an `f32`-only node means at `bf16`.
        ScalarProgram::PointwiseBf16(expression) => {
            let mut inputs = Vec::with_capacity(read_buffers.len());
            for (position, (buffer, read)) in read_buffers.iter().zip(plan.reads).enumerate() {
                // Through the read's addressing for the reason the `f32` arm is,
                // and not because a `bf16` region can carry a structural map
                // today: the recognizer builds none. It is the same derivation
                // either way, and an arm that addressed densely "because nothing
                // reaches it yet" is one a later widening silently breaks.
                let addressing =
                    plan.addressing
                        .get(position)
                        .ok_or(KernelBuildError::InvalidHandle {
                            entity: super::error::KernelEntityKind::Buffer,
                        })?;
                let offset = emit_offset(builder, addressing, invocation, None)?;
                inputs.push(builder.load(*buffer, offset, read.bounds)?);
            }
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
) -> Result<(), KernelBuildError> {
    let seed = emit_contraction_product(builder, plan, reads, invocation, None)?;
    let total = if plan.contributors <= 1 {
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
                let product =
                    emit_contraction_product(builder, plan, reads, invocation, Some(induction))?;
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
fn emit_contraction_product(
    builder: &mut KernelBuilder,
    plan: &CanonicalPlan<'_>,
    reads: [(KernelBufferId, BoundsWitnessId); 2],
    invocation: KernelValueId,
    contributor: Option<KernelValueId>,
) -> Result<KernelValueId, KernelBuildError> {
    let mut loaded = [None, None];
    for (position, (buffer, bounds)) in reads.into_iter().enumerate() {
        let addressing = plan
            .addressing
            .get(position)
            .ok_or(KernelBuildError::InvalidHandle {
                entity: super::error::KernelEntityKind::Buffer,
            })?;
        let offset = emit_offset(builder, addressing, invocation, contributor)?;
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
/// [`KernelDiagnostic::UnorderedStagedRewrite`](super::error::KernelDiagnostic::UnorderedStagedRewrite)
/// is what refuses that.
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

/// Emits the scalar body of a pointwise expression over its loaded inputs.
///
/// `inputs` is indexed by the leaf's own ordinal, not by the order the leaves
/// appear: canonicalization orders nodes by root-first discovery, so a leaf's
/// position among the nodes says nothing about which tensor it reads. An ordinal
/// with no loaded value is a region whose reads and expression disagree, which
/// the schedule verifier rejects — this reports it as an invalid handle rather
/// than reading whichever value sits at that index.
fn emit_pointwise(
    builder: &mut KernelBuilder,
    expression: &PointwiseF32Expression,
    inputs: &[KernelValueId],
) -> Result<KernelValueId, KernelBuildError> {
    let mut values = Vec::with_capacity(expression.nodes().len());
    for node in expression.nodes() {
        let value = match node {
            PointwiseF32Node::Input { ordinal } => usize::try_from(ordinal.get())
                .ok()
                .and_then(|ordinal| inputs.get(ordinal).copied())
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
    inputs: &[KernelValueId],
) -> Result<KernelValueId, KernelBuildError> {
    let mut values = Vec::with_capacity(expression.nodes().len());
    for node in expression.nodes() {
        let value = match node {
            PointwiseBf16Node::Input { ordinal } => usize::try_from(ordinal.get())
                .ok()
                .and_then(|ordinal| inputs.get(ordinal).copied())
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
/// The folded value is bound to input ordinal zero, which is the sole leaf the
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
        Some(expression) => emit_pointwise(builder, expression, &[value]),
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
