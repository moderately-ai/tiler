//! The partitioned bit-preserving copy family.
//!
//! The one region program that carries no arithmetic, so none of the numerical
//! agreement the other families owe applies to it and its whole obligation is
//! structural. Its partition theorem is re-derived here from checked prefix
//! sums rather than encoded in the program or supplied by a caller, which is
//! why the derivation and the rules that read it stay in one file: no
//! authority to state coverage exists anywhere else.

use crate::schedule::MAX_PARTITIONED_COPY_MEMBERS;
use crate::schedule::error::{PartitionedCopyRule, ScheduledRegionDiagnostic};
use crate::schedule::model::{
    Access, AccessMode, BoundsProofKind, ExecutionBinding, LogicalAccess, PartitionedCopyProgram,
    ReductionTopology, ScheduledRegion, TailPolicy, TensorRole, element_count,
};

use super::diagnostics::partitioned_copy;
use super::proof::verify_proof_records;

/// Verifies a partitioned bit-preserving copy region.
///
/// The rule order is the accepted first-failure precedence: `Topology`, the
/// shared access-count/contract gates, `ReadTensor`, `WriteTensor`, one call to
/// the unchanged [`verify_proof_records`], then `MemberCount`, `AxisRange`,
/// `SourceReference`, `SourceOrder`, `ExtentOverflow`, `CoverageSum`,
/// `SourceShape`, and `UnreferencedSource`.
///
/// The partition theorem — member intervals pairwise disjoint and jointly
/// exhaustive over the axis extent — is re-derived here rather than encoded or
/// caller-supplied: offsets are checked exclusive prefix sums, so the intervals
/// are adjacent by construction and the one representable coverage defect is a
/// wrong extent total. A caller cannot mint proof authority because no proof
/// field exists for a caller to fill; the write references the region's one
/// [`OwnershipProofKind::OneGlobalInvocationPerOutput`], which is literally
/// true of the copy — the iteration domain is the output domain and each
/// invocation performs one guarded store.
///
/// [`verify_proof_records`]: super::proof::verify_proof_records
/// [`OwnershipProofKind::OneGlobalInvocationPerOutput`]: crate::schedule::model::OwnershipProofKind::OneGlobalInvocationPerOutput
pub(super) fn verify_partitioned_copy(
    region: &ScheduledRegion,
    program: &PartitionedCopyProgram,
) -> Result<(), ScheduledRegionDiagnostic> {
    // Topology: the copy admits exactly one schedule shape. The blocked
    // binding and the predicated tail are also refused by the shared gates
    // ahead of this dispatch; the reduction clause and the fixed-vector
    // binding clause are the independently watchable ones.
    if !matches!(region.schedule.reduction, ReductionTopology::None)
        || !matches!(
            region.schedule.binding,
            ExecutionBinding::GlobalLinearInvocation
        )
        || !matches!(region.schedule.tail, TailPolicy::Exact)
    {
        return Err(partitioned_copy(PartitionedCopyRule::Topology));
    }
    // Shared access count: at least one read, then the one owning write.
    let Some((write, reads)) = region.index.accesses.split_last() else {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    };
    if reads.is_empty() {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    }
    // Shared access contract: modes, ownership placement, the write's map, and
    // `output_owner` agreement — exactly what the pointwise contract checks.
    // The read maps are deliberately not checked here: a referenced read's map
    // is `SourceReference`'s clause and an unreferenced read is refused
    // outright, so map totality is owned by the copy rules below.
    if reads
        .iter()
        .any(|read| read.mode != AccessMode::Read || read.ownership.is_some())
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    // Boundary categories, refused by name per side.
    if reads
        .iter()
        .any(|read| read.tensor != TensorRole::Input || read.component_role.is_some())
    {
        return Err(partitioned_copy(PartitionedCopyRule::ReadTensor));
    }
    if write.tensor != TensorRole::Output || write.component_role.is_some() {
        return Err(partitioned_copy(PartitionedCopyRule::WriteTensor));
    }
    // Shared proof gates: one call to the unchanged `verify_proof_records`,
    // whose `LinearRange`/copy-source refinement arm is structural — the
    // fieldless map cannot name the member-derived count from inside
    // `bounds_proof_refines_access`, so exactness is `SourceShape`'s below.
    let read_refs: Vec<&Access> = reads.iter().collect();
    verify_proof_records(region, &read_refs, write)?;
    if program.members.len() < 2 || program.members.len() > MAX_PARTITIONED_COPY_MEMBERS {
        return Err(partitioned_copy(PartitionedCopyRule::MemberCount));
    }
    let Some(axis) = usize::try_from(program.axis.get())
        .ok()
        .filter(|axis| *axis < region.index.iteration_shape.rank())
    else {
        return Err(partitioned_copy(PartitionedCopyRule::AxisRange));
    };
    // Every member names a read — never the write, never a dangling ordinal —
    // and the read it names carries the fieldless copy-source map.
    for member in &program.members {
        let Ok(position) = usize::try_from(member.source.get()) else {
            return Err(partitioned_copy(PartitionedCopyRule::SourceReference));
        };
        let Some(read) = reads.get(position) else {
            return Err(partitioned_copy(PartitionedCopyRule::SourceReference));
        };
        if read.map != LogicalAccess::PartitionedCopySource {
            return Err(partitioned_copy(PartitionedCopyRule::SourceReference));
        }
    }
    // Canonical read order: the sequence of first references of member sources
    // must be exactly the dense ascending run `0, 1, 2, ...` — the rule that
    // gives one meaning one identity. It is a prefix requirement over the
    // reads; a read outside the referenced prefix is `UnreferencedSource`'s
    // below.
    let mut next_first_reference = 0_u32;
    for member in &program.members {
        let source = member.source.get();
        if source == next_first_reference {
            next_first_reference += 1;
        } else if source > next_first_reference {
            return Err(partitioned_copy(PartitionedCopyRule::SourceOrder));
        }
    }
    // Derived quantities under checked arithmetic: prefix sums and per-member
    // source element counts. Overflow is refused before any comparison uses a
    // wrapped value.
    if program.member_offsets().is_none() {
        return Err(partitioned_copy(PartitionedCopyRule::ExtentOverflow));
    }
    let mut member_source_elements = Vec::with_capacity(program.members.len());
    for position in 0..program.members.len() {
        let Some(shape) = program.member_source_shape(&region.index.iteration_shape, position)
        else {
            // Unreachable once the axis gate above passed; refusing under the
            // axis rule keeps the cause named if it ever is reached.
            return Err(partitioned_copy(PartitionedCopyRule::AxisRange));
        };
        let Ok(elements) = element_count(&shape) else {
            return Err(partitioned_copy(PartitionedCopyRule::ExtentOverflow));
        };
        member_source_elements.push(elements);
    }
    // Coverage: the checked extent total must be the axis extent exactly. With
    // derived prefix offsets the intervals are adjacent by construction, so
    // this one equality is the whole partition theorem — no gap or overlap is
    // representable.
    let axis_extent = region.index.iteration_shape.extents()[axis].get();
    let mut extent_sum = 0_u64;
    for member in &program.members {
        let Some(sum) = extent_sum.checked_add(member.extent) else {
            return Err(partitioned_copy(PartitionedCopyRule::ExtentOverflow));
        };
        extent_sum = sum;
    }
    if extent_sum != axis_extent {
        return Err(partitioned_copy(PartitionedCopyRule::CoverageSum));
    }
    // Source shapes: members sharing one read must agree on its extent, and
    // the read's bounds-proof element count must equal the derived source
    // element count — the exactness the structural refinement arm deferred.
    let mut read_extents: Vec<Option<u64>> = vec![None; reads.len()];
    for (member, elements) in program.members.iter().zip(&member_source_elements) {
        let Ok(position) = usize::try_from(member.source.get()) else {
            return Err(partitioned_copy(PartitionedCopyRule::SourceReference));
        };
        match read_extents[position] {
            None => {
                read_extents[position] = Some(member.extent);
                let proof = &region.index.bounds_proofs[position];
                let BoundsProofKind::LinearRange { element_count } = &proof.kind else {
                    return Err(partitioned_copy(PartitionedCopyRule::SourceShape));
                };
                if *element_count != *elements {
                    return Err(partitioned_copy(PartitionedCopyRule::SourceShape));
                }
            }
            Some(extent) => {
                if extent != member.extent {
                    return Err(partitioned_copy(PartitionedCopyRule::SourceShape));
                }
            }
        }
    }
    // Every read is referenced: with `SourceOrder`'s dense-prefix rule already
    // proved, an unreferenced read is exactly one past the referenced prefix.
    if usize::try_from(next_first_reference).ok() != Some(reads.len()) {
        return Err(partitioned_copy(PartitionedCopyRule::UnreferencedSource));
    }
    Ok(())
}
