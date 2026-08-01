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

use std::collections::BTreeSet;

use crate::schedule::{
    Access, AccessMode, BoundsProofKind, BoundsWitnessId, CanonicalScheduledRegionIdentity,
    CooperativeTile, ExecutionBinding, OwnershipWitnessId, ReductionPass, ReductionTopology,
    ResourceRequirements, ScheduledRegion, StagedElement, contributor_count, cooperative_tile,
    element_count,
};

use super::error::KernelDiagnostic;
use super::model::{
    AddressSpace, BufferAccess, Builtin, CompareOp, KernelConstant, KernelData, KernelType,
    OperationKind,
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
        BoundsProofKind::LinearRange { element_count } => Ok(*element_count),
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
}

#[derive(Clone, Copy, Debug)]
enum EffectKind {
    Load {
        bounds: BoundsWitnessId,
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
    accumulators: Vec<KernelType>,
    block_depth: u32,
}

#[derive(Debug, Default)]
struct Walk {
    effects: Vec<Effect>,
    has_synchronization: bool,
    loops: Vec<LoopSummary>,
    ungoverned_predicate: bool,
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
    verify_cooperative(data, schedule)?;

    let guards = guard_values(data, schedule.schedule.work_items);
    let mut walk = Walk::default();
    visit_block(data, 0, false, 0, 0, &guards, &mut walk);
    if walk.ungoverned_predicate {
        return Err(KernelDiagnostic::PredicateDominance);
    }
    verify_effects(&walk, schedule, reads, write)?;
    verify_reduction(&walk, schedule, reads)?;

    let canonical = super::lower::derive_canonical(schedule, schedule_identity, derived)?;
    if data != &canonical {
        return Err(KernelDiagnostic::BodyRefinement);
    }
    Ok(())
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
    // one dense `f32` tensor per expression leaf, so its expected signature is
    // as wide as its own access list rather than a constant — and the schedule
    // verifier already proved that width equals the expression's input count.
    let expected_types: Vec<KernelType> = match schedule.index.scalar_program {
        crate::schedule::ScalarProgram::StrictAffineU4Dequantize { .. } => {
            vec![KernelType::U8, KernelType::F32, KernelType::U8]
        }
        crate::schedule::ScalarProgram::PointwiseF32(_) => vec![KernelType::F32; reads.len()],
        crate::schedule::ScalarProgram::StrictSerialSum { .. }
        | crate::schedule::ScalarProgram::SquaredSerialSum { .. }
        | crate::schedule::ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            vec![KernelType::F32]
        }
        // Two, stated as two rather than as `reads.len()`: the count is the
        // contraction family's own arity, so a region declaring some other
        // number of reads must fail this comparison instead of agreeing with
        // itself.
        crate::schedule::ScalarProgram::StrictTensorContraction { .. } => {
            vec![KernelType::F32, KernelType::F32]
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
        || write_buffer.element_type != KernelType::F32
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
    let mut expected_builtins = match schedule.schedule.binding {
        ExecutionBinding::GlobalLinearInvocation => vec![Builtin::GlobalInvocationIndex],
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
/// Two jobs. The staging declarations must realize the tile exactly — same
/// count, same ordinals in order, same element type, same slot count — so a
/// producer cannot allocate more or differently shaped workgroup storage than
/// the schedule proved well formed. And a region carrying any cross-invocation
/// visibility dependency is refused outright: the tile states that one
/// participant's staged writes are read by others in a later phase, and nothing
/// orders the two.
///
/// **The refusal is derived, not a placeholder.** The barrier vocabulary is
/// rejected intrinsically by [`verify_effects`], and no schedule owns a
/// synchronization point a barrier could be matched to, so there is no
/// construct this kernel could contain that would discharge the edge. Admitting
/// the kernel would hand a backend a program whose staged reads observe
/// unordered writes. Discharging an edge is the synchronization authority's
/// work; representing one is this ticket's.
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
    if CooperativeTile::visibility_edges(tile).is_empty() {
        return Ok(());
    }
    Err(KernelDiagnostic::UndischargedVisibility)
}

/// Collects the values that denote the scheduled bounds predicate.
///
/// A governed predicate compares an admitted global invocation index against
/// the exact scheduled work-item count. Any other predicate leaves an effect
/// undominated by schedule-derived bounds evidence.
fn guard_values(data: &KernelData, work_items: u64) -> BTreeSet<u32> {
    let mut invocations = BTreeSet::new();
    for block in &data.blocks {
        for operation in &block.operations {
            if let OperationKind::Builtin {
                builtin: Builtin::GlobalInvocationIndex,
            } = operation.kind
            {
                invocations.extend(operation.results.iter().copied());
            }
        }
    }
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
            if invocations.contains(&lhs) && bound == Some(work_items) {
                guards.extend(operation.results.iter().copied());
            }
        }
    }
    guards
}

fn visit_block(
    data: &KernelData,
    block: u32,
    guarded: bool,
    loop_depth: u32,
    block_depth: u32,
    guards: &BTreeSet<u32>,
    walk: &mut Walk,
) {
    let Some(block) = data.blocks.get(block as usize) else {
        return;
    };
    for operation in &block.operations {
        match &operation.kind {
            OperationKind::Load { bounds, .. } => walk.effects.push(Effect {
                kind: EffectKind::Load { bounds: *bounds },
                loop_depth,
                guarded,
            }),
            OperationKind::Store {
                bounds, ownership, ..
            } => walk.effects.push(Effect {
                kind: EffectKind::Store {
                    bounds: *bounds,
                    ownership: *ownership,
                },
                loop_depth,
                guarded,
            }),
            OperationKind::Barrier { .. } => walk.has_synchronization = true,
            OperationKind::Predicated { predicate, body } => {
                if !guards.contains(predicate) {
                    walk.ungoverned_predicate = true;
                }
                visit_block(
                    data,
                    *body,
                    true,
                    loop_depth,
                    block_depth.saturating_add(1),
                    guards,
                    walk,
                );
            }
            OperationKind::SerialLoop {
                start, end, body, ..
            } => {
                let accumulators = data
                    .blocks
                    .get(*body as usize)
                    .map(|inner| {
                        inner
                            .parameters
                            .iter()
                            .skip(1)
                            .filter_map(|index| data.values.get(*index as usize))
                            .map(|value| value.value_type)
                            .collect()
                    })
                    .unwrap_or_default();
                walk.loops.push(LoopSummary {
                    start: *start,
                    end: *end,
                    accumulators,
                    block_depth,
                });
                visit_block(
                    data,
                    *body,
                    guarded,
                    loop_depth.saturating_add(1),
                    block_depth.saturating_add(1),
                    guards,
                    walk,
                );
            }
            OperationKind::Builtin { .. }
            | OperationKind::Constant { .. }
            | OperationKind::Binary { .. }
            | OperationKind::Compare { .. }
            | OperationKind::Convert { .. }
            | OperationKind::Unary { .. }
            | OperationKind::PackedExtract { .. } => {}
        }
    }
}

fn verify_effects(
    walk: &Walk,
    schedule: &ScheduledRegion,
    reads: &[Access],
    write: &Access,
) -> Result<(), KernelDiagnostic> {
    if walk.has_synchronization {
        return Err(KernelDiagnostic::UnexpectedSynchronization);
    }
    if walk.effects.iter().any(|effect| !effect.guarded) {
        return Err(KernelDiagnostic::PredicateDominance);
    }
    let mut stores = 0_usize;
    for effect in &walk.effects {
        match effect.kind {
            EffectKind::Load { bounds } => {
                if !reads.iter().any(|read| bounds == read.bounds) {
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
                if effect.loop_depth != 0 {
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
    schedule: &ScheduledRegion,
    reads: &[Access],
) -> Result<(), KernelDiagnostic> {
    match &schedule.schedule.reduction {
        ReductionTopology::None => {
            if walk.loops.is_empty() {
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
            partition,
            ..
        } => verify_contributor_loop(walk, partition.contributors_per_partition),
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
        // `verify_cooperative` refused this region before the walk reached
        // here, so no loop obligation is stated for it: a cooperative fold's
        // shape is decided together with the synchronization point that
        // separates its phases, and stating one now would be a contract written
        // against a body nothing can produce.
        ReductionTopology::CooperativeWorkgroup { .. } => {
            Err(KernelDiagnostic::UndischargedVisibility)
        }
    }
}

/// Proves the body realizes exactly the scheduled contributor fold.
///
/// Zero contributors commit the reduction identity and exactly one contributor
/// commits the single loaded value; neither admits a bounded loop, whose range
/// would have to be empty.
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
