//! The proof records every family gate discharges, and their populations.
//!
//! One file for the bounds and ownership obligations because they are checked
//! against derived quantities no single gate owns: the boundary positions a
//! region's owning write covers, and the reduction output domain an iteration
//! shape realizes. Deriving each once is what lets one bounds-proof rule serve
//! the serial, partial, and cooperative shapes, and what stops a topology from
//! claiming ownership of positions it never writes.

use crate::schedule::error::ScheduledRegionDiagnostic;
use crate::schedule::model::{
    Access, BoundsProof, BoundsProofKind, LogicalAccess, OwnershipProofKind, ReductionPass,
    ReductionTopology, ScheduledRegion,
};

/// Returns the boundary output positions one region's owning write covers.
///
/// Equal to the work-item count for every topology in which one invocation owns
/// one output. A cooperative tile runs one invocation per (output, participant)
/// pair, so its owned set is `participants` times smaller — and the ownership
/// proof, the write's bounds proof, and the write's linear index all read this
/// value rather than the work-item count, which would otherwise claim ownership
/// of positions the region never writes.
fn owned_output_positions(region: &ScheduledRegion) -> Option<u64> {
    let work_items = region.schedule.work_items;
    // Ownership is a fact of the topology, not of the mere presence of a tile.
    // [`ReductionTopology::CooperativeWorkgroup`] runs one invocation per
    // (output, participant) pair and one committer writes; the operand-sharing
    // sibling owns one position per invocation. Inferring the first from
    // `cooperative_tile` would silently undersize the operand-sharing write.
    match &region.schedule.reduction {
        ReductionTopology::CooperativeWorkgroup { tile, .. } => {
            let participants = tile.coordinates.participants.participants()?;
            if participants == 0 || !work_items.is_multiple_of(participants) {
                return None;
            }
            Some(work_items / participants)
        }
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::LiveContraction { .. }
        | ReductionTopology::CooperativeContraction { .. } => Some(work_items),
    }
}

/// Returns the reduction output shape this region's iteration domain realizes.
///
/// A serial or final pass iterates the reduction's own output; a partial pass
/// iterates it once per partition, so its iteration shape carries one trailing
/// axis the reduction domain does not. A cooperative tile has the same trailing
/// axis, one coordinate per participant, for the same reason. Reading the domain
/// back from the iteration shape is what lets one bounds-proof rule serve all
/// three.
fn reduction_output_shape(region: &ScheduledRegion) -> Option<crate::shape::Shape> {
    let shape = &region.index.iteration_shape;
    let trailing_partitions = match &region.schedule.reduction {
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            coverage,
            ..
        }
        | ReductionTopology::CooperativeWorkgroup { coverage, .. } => {
            coverage.partition().partitions
        }
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::CooperativeContraction { .. }
        | ReductionTopology::LiveContraction { .. }
        | ReductionTopology::MultiPass { .. } => return Some(shape.clone()),
    };
    let kept = shape.rank().checked_sub(1)?;
    let trailing = shape.extents().get(kept)?;
    (trailing.get() == trailing_partitions)
        .then(|| crate::shape::Shape::try_new(shape.extents()[..kept].iter().copied()).ok())
        .flatten()
}

pub(super) fn verify_proof_records(
    region: &ScheduledRegion,
    reads: &[&Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let Some((write_proof, read_proofs)) = region.index.bounds_proofs.split_last() else {
        return Err(ScheduledRegionDiagnostic::BoundsProofCount);
    };
    // A witness id is a *key*, not a label: [`BoundsWitnessId`] is a region-local
    // *reference* to a proof witness, and both resolvers that follow one back
    // take the first record bearing it — the pointwise gate's
    // `gather-address-read-proof-mismatch` rule, and the kernel layer's
    // `access_elements`, which sizes a buffer parameter from whichever record it
    // lands on. Two records sharing an id therefore make the later one
    // unreachable through the only handle anything has on it, while it is still
    // folded into canonical scheduled-region identity: an admitted region
    // carrying a proof that nothing can resolve and no rule ever compared.
    //
    // Distinctness spans the whole list rather than the read-versus-write pair
    // it replaces, because the write proof is resolved by id too. One rule for
    // one invariant: a narrower clause beside this one would suggest that
    // read-versus-read and read-versus-write are separate properties, and would
    // leave the two free to drift apart.
    let mut witness_ids: Vec<_> = region
        .index
        .bounds_proofs
        .iter()
        .map(|proof| proof.id)
        .collect();
    witness_ids.sort_unstable();
    if read_proofs.len() != reads.len()
        || read_proofs.iter().zip(reads).any(|(proof, read)| {
            proof.id != read.bounds
                || proof.tensor != read.tensor
                || proof.component_role != read.component_role
        })
        || write_proof.id != write.bounds
        || write_proof.tensor != write.tensor
        || write_proof.component_role != write.component_role
        || witness_ids.windows(2).any(|pair| pair[0] == pair[1])
        || region.index.ownership_proof.id != region.schedule.output_owner
        || region.index.ownership_proof.tensor != write.tensor
        || owned_output_positions(region).is_none_or(|output_count| {
            region.index.ownership_proof.kind
                != (OwnershipProofKind::OneGlobalInvocationPerOutput { output_count })
        })
    {
        return Err(ScheduledRegionDiagnostic::ProofReference);
    }
    if read_proofs
        .iter()
        .zip(reads)
        .any(|(proof, read)| !bounds_proof_refines_access(proof, &read.map, region))
        || !bounds_proof_refines_access(write_proof, &write.map, region)
    {
        return Err(ScheduledRegionDiagnostic::BoundsProof);
    }
    Ok(())
}

fn bounds_proof_refines_access(
    proof: &BoundsProof,
    access: &LogicalAccess,
    region: &ScheduledRegion,
) -> bool {
    match (&proof.kind, access) {
        // The owned positions rather than the work items: they are the same
        // number for every topology that runs one invocation per output. A
        // one-committer cooperative tile's write covers one position per
        // workgroup; the operand-sharing sibling owns one position per
        // invocation. `owned_output_positions` decides from the topology, not
        // from the mere presence of a tile.
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LinearIdentity) => {
            owned_output_positions(region).is_some_and(|owned| *element_count == owned)
        }
        // Both live relations record the same absence: the buffer is sized by
        // the live inner extent the schedule does not specialize, so the proof
        // is a zero linear range for the source marker and every consumer
        // alike.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::LiveRowMajorSource { .. } | LogicalAccess::LiveRowMajor,
        ) => *element_count == 0,
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::ScalarBroadcast) => {
            *element_count == 1
        }
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::PackedU4LsbZeroTail { logical_elements },
        ) => {
            *logical_elements == region.schedule.work_items
                && *element_count == logical_elements.div_ceil(2)
        }
        (
            BoundsProofKind::ReductionDomain {
                input_shape,
                output_shape,
                axes,
                order,
            },
            LogicalAccess::ReductionContributor {
                input_shape: access_input,
                output_shape: access_output,
                axes: access_axes,
                order: access_order,
            },
        ) => {
            input_shape == access_input
                && output_shape == access_output
                && reduction_output_shape(region).is_some_and(|domain| *output_shape == domain)
                && axes == access_axes
                && order == access_order
                && input_shape.without_axes(axes) == *output_shape
        }
        // A live contraction's operand buffers are sized by the live inner
        // extent, which the schedule does not specialize. The proof records
        // that absence as a zero linear range, the same convention
        // `LiveRowMajor` uses. The static `ContractionOperand` arm below still
        // compares a concrete operand product, so a live region cannot inherit
        // that check and silently bake `S`.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ContractionOperand { .. },
        ) if matches!(
            region.schedule.reduction,
            ReductionTopology::LiveContraction { .. }
        ) =>
        {
            *element_count == 0
        }
        // A contraction operand's proven domain is the contiguous linear range
        // of its own elements, exactly as an identity-mapped access's is. It
        // pairs with `LinearRange` for that reason rather than needing a fourth
        // proof structure: which of those positions the access touches is what
        // the map states, and `verify_contraction` proves every coordinate the
        // map derives is in range by requiring per-axis extent agreement.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ContractionOperand { operand_shape, .. },
        ) => crate::schedule::model::element_count(operand_shape)
            .is_ok_and(|elements| *element_count == elements),
        // Both structural relations prove the same domain a contraction operand
        // does, and for the same reason: the access ranges over its operand's own
        // contiguous element range, and *which* of those positions each iteration
        // coordinate touches is what the map states and what
        // `pointwise_read_map_is_admissible` proved in range. A reindex and a
        // replication differ in how many times a position is touched, which is a
        // fact about the map and not about the domain the proof bounds — so a
        // separate proof structure here would carry no information the map does
        // not already carry.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ReindexBijection { operand_shape, .. }
            | LogicalAccess::BroadcastReplication { operand_shape, .. },
        ) => crate::schedule::model::element_count(operand_shape)
            .is_ok_and(|elements| *element_count == elements),
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ParametricBroadcast { operand_shape, .. },
        ) => operand_shape.as_static().is_some_and(|shape| {
            crate::schedule::model::element_count(shape)
                .is_ok_and(|elements| *element_count == elements)
        }),
        // Structural pairing only: the fieldless copy-source map cannot say
        // which member-derived source element count applies, and this
        // function's signature carries no access ordinal to look one up with.
        // The exact element-count agreement is owned by the
        // `partitioned-copy-source-shape` rule in the copy gate.
        //
        // The gather pair joins it for the same reason at a different layer:
        // the exact agreement between the proof's five relation fields, the
        // relation's own five, and the retained static proof's subject is the
        // `gather-address-read-proof-mismatch` rule in the pointwise gate,
        // which is the only place that can see the whole access list. What
        // this arm contributes is the *crossing* refusal — a gather relation
        // paired with a `LinearRange`, or a gather proof paired with any other
        // relation, falls to the wildcard below and is refused as
        // `BoundsProof` rather than admitted on a domain nobody proved.
        //
        // That delegation is total only because the witness ids are distinct.
        // The rule it delegates to resolves a read's record *by id* while this
        // arm is reached *positionally*, so under a duplicate id the two would
        // disagree about which record they are talking about and the one this
        // arm waved through would be a record that rule never examined. The
        // distinctness clause in `verify_proof_records` above is what closes
        // that gap; this arm may not be read as discharging anything itself.
        (BoundsProofKind::LinearRange { .. }, LogicalAccess::PartitionedCopySource)
        | (BoundsProofKind::GatherSource { .. }, LogicalAccess::GatherSource { .. }) => true,
        _ => false,
    }
}
