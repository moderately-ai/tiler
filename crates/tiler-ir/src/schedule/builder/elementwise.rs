//! The families that evaluate one scalar program per output position.
//!
//! No fold happens here, so nothing in this file reads a contributor domain,
//! an accumulation width, or a reassociation permission — which is the seam.
//! What it does own is the access side of a per-position region: the shared
//! N-input pointwise contract and its two widths, the source-bound
//! live-row-major relation, the read maps such a region admits, and the
//! strict-affine `u4` dequantize, whose three component reads are a fixed
//! signature rather than an expression's leaf count.

use crate::schedule::SubnormalMode;
use crate::schedule::error::{
    GatherAddressReadRule, LiveRowMajorSourceRule, ScheduledRegionDiagnostic,
};
use crate::schedule::handles::AccessOrdinal;
use crate::schedule::model::{
    Access, AccessMode, BoundsProofKind, LogicalAccess, ReductionTopology, ScalarProgram,
    ScheduledRegion, TensorRole, broadcast_decodes_are_replicating, gather_index_read_map,
    reindex_decodes_are_bijective,
};
use crate::schedule::numerics::ExceptionalValueAssumption;
use crate::schedule::parametric::parametric_broadcast_read_is_admissible;
use crate::schedule::pointwise::PointwiseF32Expression;
use crate::schedule::pointwise_bf16::PointwiseBf16Expression;
use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};

use super::diagnostics::numerical_program;
use super::proof::verify_proof_records;

/// Verifies an N-input physical `f32` pointwise region.
///
/// The whole obligation is the shared access contract below; this width states
/// nothing of its own. Its canonical arithmetic NaN payload is *not* checked
/// here, and that asymmetry with `bf16` is deliberate rather than an omission:
/// an `f32` region's payload is already compared against the request's own
/// numerical contract by `tiler-compiler`'s subject binding, whereas a 16-bit
/// payload sitting in a 32-bit field has no such comparison anywhere and would
/// otherwise be an unstated reading.
pub(super) fn verify_pointwise_f32(
    region: &ScheduledRegion,
    expression: &PointwiseF32Expression,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    verify_pointwise_region(
        region,
        expression.input_count(),
        expression.is_valid(),
        reads,
        write,
    )
}

/// Verifies an N-input physical `bf16` pointwise region.
///
/// The access contract is the shared one below; what is stated here is the
/// obligation that belongs to this width alone.
///
/// **The region's declared canonical arithmetic NaN payload must be `bf16`'s
/// own, zero-extended.** [`NumericalRealization::canonical_arithmetic_nan_bits`]
/// is a 32-bit field and `bf16`'s canonical arithmetic NaN is the 16-bit
/// [`CANONICAL_BF16_ARITHMETIC_NAN_BITS`](crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
/// so which reading applies would otherwise be an unstated invariant every
/// consumer had to guess — and the one consumer that matters guesses wrongly by
/// default, because a lowering that canonicalized to `0x7fc0_0000` would write a
/// pattern no `bf16` value can hold. Requiring it here makes the reading checked
/// at the boundary that produces the region rather than assumed at the boundary
/// that emits it.
///
/// [`NumericalRealization::canonical_arithmetic_nan_bits`]: crate::schedule::numerics::NumericalRealization::canonical_arithmetic_nan_bits
pub(super) fn verify_pointwise_bf16(
    region: &ScheduledRegion,
    expression: &PointwiseBf16Expression,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if u32::from(crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS)
        != numerical_program(region)?.1.canonical_arithmetic_nan_bits
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_pointwise_region(
        region,
        expression.input_count(),
        expression.is_valid(),
        reads,
        write,
    )
}

/// The access contract every pointwise region satisfies, at any width.
///
/// Three obligations make an N-input region safe, and they are about different
/// things. **The count**: there must be exactly as many reads as the expression
/// has input leaves, or an expression could read a position no access binds — a
/// load through a buffer the signature never declares. The expression's own
/// verifier already proved its access coordinates are the dense `0..n`, so leaf `i` is
/// served by read `i` and the pairing is exhaustive rather than a sample.
/// **The binding category**: each read must name an input or the sole attributed
/// materialized intermediate. Exact declared-input association is absent here
/// and is projected later from the compiler's checked request subject.
/// **The addressing regime**: every access is static, or every access is
/// `LiveRowMajor` on the same inner axis. Canonical lowering has one body-wide
/// live loop and one offset for every boundary effect, so admitting one live map
/// beside a static or differently live map would execute an access relation the
/// region did not prove.
///
/// Shared by the two width-specific verifiers above rather than written twice:
/// the obligation is about *accesses*, and nothing in it reads an element type.
/// The expression's own validity and input count arrive as already-derived facts
/// so this function never has to classify which vocabulary it is looking at,
/// which is what keeps the dispatch above exhaustive instead of pushing a second
/// match in here.
fn verify_pointwise_region(
    region: &ScheduledRegion,
    input_count: usize,
    expression_is_valid: bool,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    // A region's reads are the expression's scalar leaves followed by one
    // address-only read per gather. Without a gather there is no second run, so
    // the leaf count is the whole rule and this stays the exact equality every
    // pointwise region has always satisfied.
    //
    // With a gather present the equality is deliberately **not** restated as
    // `input_count + gathers`. That would be a second authority over the same
    // fact, and it would make
    // [`GatherAddressReadRule::IndexUnowned`] unreachable: an extra address read
    // would be refused as a wrong count before the bijection could name which
    // read is orphaned. The association gate below owns the accounting for every
    // read past the leaf run — each must be named by exactly one gather — and
    // that bijection *implies* the count rather than assuming it.
    let gathers = reads
        .iter()
        .filter(|read| matches!(read.map, LogicalAccess::GatherSource { .. }))
        .count();
    let miscounted = if gathers == 0 {
        reads.len() != input_count
    } else {
        reads.len() < input_count
    };
    if reads.is_empty() || miscounted {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    }
    // The source-relation gate runs before the broad access-contract and
    // refinement gates, so a marker-count, marker-role, or missing-consumer
    // defect is named under its own dedicated rule rather than collapsing into
    // `AccessContract` or `NumericalOrAccessRefinement` — the accepted
    // precedence the fieldless-marker surface states.
    verify_live_row_major_source(reads, write)?;
    // The gather association gate runs here for the same reason: an ordering,
    // ownership, occurrence, or proof defect between a source and its address
    // read is a cross-access property that would otherwise collapse into one of
    // the broad buckets.
    verify_gather_address_reads(region, input_count, reads)?;
    if reads
        .iter()
        .any(|read| read.mode != AccessMode::Read || read.ownership.is_some())
        || write.mode != AccessMode::Write
        || !matches!(
            write.map,
            LogicalAccess::LinearIdentity | LogicalAccess::LiveRowMajor
        )
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    let read_refs: Vec<&Access> = reads.iter().collect();
    verify_proof_records(region, &read_refs, write)?;
    if !expression_is_valid
        || !matches!(region.schedule.reduction, ReductionTopology::None)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || !reads_bind_boundary_tensors_in_order(reads)
        // Scalar leaves only. The address-only reads that follow them carry the
        // relation `gather_index_read_map` derived and the gather gate above
        // already checked; putting them through the leaf admission as well would
        // refuse a rank-zero index, whose derived `ScalarBroadcast` is
        // deliberately not an admissible *leaf* map.
        || !reads
            .get(..input_count)
            .is_some_and(|leaves| {
                leaves.iter().all(|read| {
                    pointwise_read_map_is_admissible(&read.map, &region.index.iteration_shape)
                })
            })
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies every gather source read's association with its address-only read.
///
/// **Accepted public surface** (2026-08-18, under
/// `decide-the-data-dependent-index-representation-public-surface`): the eight
/// [`GatherAddressReadRule`]s at the accepted first-failure precedence — owner
/// range and order, then mode, then relation, then scalar-leaf use, then
/// sharing, then orphaning, then occurrence binding, then proof mismatch. A
/// region carrying no gather has no obligation here and passes untouched.
///
/// The canonical access order this polices is scalar value-producing reads in
/// pointwise-leaf order, then one address-only U32 read per owning gather in
/// owner-access order, then the write. `input_count` is therefore the boundary
/// between the two runs, and it arrives as an already-derived fact rather than
/// being recovered by classifying maps — a gather *source* is itself a scalar
/// leaf, so the two runs cannot be told apart by relation alone.
///
/// **Only statically proved gathers reach schedule formation.** A gather whose
/// index-value obligation is outstanding carries a validation requirement, which
/// has no spelling in this vocabulary at all, so there is no arm here that could
/// admit one.
///
/// One deliberate refinement of the stated precedence, recorded because it is a
/// deviation a later reader would otherwise have to re-derive: when the relation
/// is itself malformed — a rank-zero source, an out-of-range axis, or overflowing
/// result arithmetic — [`gather_index_read_map`] has no answer to compare the
/// address read against, so [`GatherAddressReadRule::IndexRelation`] is
/// *undecidable* rather than violated. That case is skipped here and reported at
/// its own position as [`GatherAddressReadRule::OccurrenceBinding`], which names
/// the actual defect instead of attributing it to the address read.
fn verify_gather_address_reads(
    region: &ScheduledRegion,
    input_count: usize,
    reads: &[Access],
) -> Result<(), ScheduledRegionDiagnostic> {
    let ordinal = |position: usize| {
        AccessOrdinal::new(u32::try_from(position).expect("verified access count is bounded"))
    };
    let fail = |source: Option<usize>, index: usize, rule: GatherAddressReadRule| {
        ScheduledRegionDiagnostic::GatherAddressRead {
            source_access: source.map(ordinal),
            index_access: ordinal(index),
            rule,
        }
    };
    let mut owned: Vec<Option<usize>> = vec![None; reads.len()];
    for (position, read) in reads.iter().enumerate() {
        // `result_shape` is deliberately not read in this loop: rules 1 to 5
        // are about the *address read*, and the relation's own occurrence
        // agreement is rule 7, which runs after the ownership bijection closes.
        let LogicalAccess::GatherSource {
            source_shape,
            result_shape: _,
            axis,
            index_access,
            index_shape,
        } = &read.map
        else {
            continue;
        };
        let named = usize::try_from(index_access.get()).unwrap_or(usize::MAX);
        // Rule 1: owner range and order. An ordinal past the access list or at
        // or before its own source cannot be this gather's address read.
        if named <= position || named >= reads.len() {
            return Err(fail(
                Some(position),
                named.min(reads.len()),
                GatherAddressReadRule::IndexNotLater,
            ));
        }
        let address = &reads[named];
        // Rule 2: mode. An address read on an intermediate, an output, or a
        // write would address the gather from storage no program input backs.
        if address.tensor != TensorRole::Input
            || address.mode != AccessMode::Read
            || address.ownership.is_some()
        {
            return Err(fail(
                Some(position),
                named,
                GatherAddressReadRule::IndexMode,
            ));
        }
        // Rule 3: relation, when one is derivable. The address map is
        // verifier-derived and never caller-selected, so any other map is a
        // second, contradictory account of the same addressing.
        let derived = gather_index_read_map(source_shape, *axis, index_shape);
        if let Some(derived) = derived.as_ref()
            && address.map != *derived
        {
            return Err(fail(
                Some(position),
                named,
                GatherAddressReadRule::IndexRelation,
            ));
        }
        // Rule 4: scalar-leaf use. An address read supplies coordinates, not
        // values; a position inside the leaf run would expose the loaded U32 as
        // scalar SSA.
        if named < input_count {
            return Err(fail(
                Some(position),
                named,
                GatherAddressReadRule::IndexUsedAsScalarLeaf,
            ));
        }
        // Rule 5: sharing. Two gathers naming one address read would have a
        // single coordinate authority between them.
        if let Some(first) = owned[named] {
            return Err(fail(Some(first), named, GatherAddressReadRule::IndexShared));
        }
        owned[named] = Some(position);
    }
    // Rule 6: orphaning. Every read past the leaf run is an address read and must
    // have exactly one owner. This is the half of the bijection the per-gather
    // loop cannot see, and it is the one rule whose `source_access` is `None`.
    for (position, owner) in owned.iter().enumerate().skip(input_count) {
        if owner.is_none() {
            return Err(fail(None, position, GatherAddressReadRule::IndexUnowned));
        }
    }
    // Rule 7: occurrence binding. The relation must describe its own occurrence
    // — the stated result shape is the one the source shape, axis, and index
    // shape derive, and it is the region's iteration domain.
    for (position, read) in reads.iter().enumerate() {
        let LogicalAccess::GatherSource {
            source_shape,
            result_shape,
            axis,
            index_access,
            index_shape,
        } = &read.map
        else {
            continue;
        };
        let named = usize::try_from(index_access.get()).unwrap_or(usize::MAX);
        let derived_result =
            crate::semantic::gather_result_shape(*axis, source_shape, index_shape).ok();
        if derived_result.is_none_or(|(_, shape)| shape != *result_shape)
            || *result_shape != region.index.iteration_shape
        {
            return Err(fail(
                Some(position),
                named,
                GatherAddressReadRule::OccurrenceBinding,
            ));
        }
    }
    // Rule 8: proof mismatch. The paired bounds proof must restate this exact
    // relation, and the retained static proof's own subject must agree with it.
    // The proof is read through its public accessors rather than re-derived, so
    // this compares two independently produced accounts.
    for (position, read) in reads.iter().enumerate() {
        let LogicalAccess::GatherSource {
            source_shape,
            result_shape,
            axis,
            index_access,
            index_shape,
        } = &read.map
        else {
            continue;
        };
        let named = usize::try_from(index_access.get()).unwrap_or(usize::MAX);
        let mismatch = || fail(Some(position), named, GatherAddressReadRule::ProofMismatch);
        let Some(record) = region
            .index
            .bounds_proofs
            .iter()
            .find(|record| record.id == read.bounds)
        else {
            return Err(mismatch());
        };
        let BoundsProofKind::GatherSource {
            source_shape: proof_source,
            result_shape: proof_result,
            axis: proof_axis,
            index_access: proof_index_access,
            index_shape: proof_index,
            proof,
        } = &record.kind
        else {
            return Err(mismatch());
        };
        if proof_source != source_shape
            || proof_result != result_shape
            || proof_axis != axis
            || proof_index_access != index_access
            || proof_index != index_shape
            || proof.source_shape() != source_shape
            || proof.result_shape() != result_shape
            || proof.index_shape() != index_shape
            || proof.axis() != *axis
        {
            return Err(mismatch());
        }
    }
    Ok(())
}

/// Verifies the source-bound live-row-major relation of one pointwise region.
///
/// **Accepted public surface** (2026-08-18, under
/// `decide-the-source-bound-live-row-major-access-surface`): the fieldless
/// contextual marker's four rules at the accepted first-failure precedence —
/// marker count, then the unique marker's role and mode, then complete
/// live-relation coverage. An all-static region has no source obligation and
/// passes untouched; static reads keep their own admitted coordinate relations
/// under the refinement gate below.
///
/// Once any selected live relation appears, canonical lowering has one
/// body-wide loop bound, stride, and element offset, so every pointwise read
/// and the owning write must state the relation: exactly one
/// [`LogicalAccess::LiveRowMajorSource`] input read declares the region's
/// runtime extent operand and every other access carries the fieldless
/// [`LogicalAccess::LiveRowMajor`] consumer marker. No access becomes
/// authority for a sibling here — a consumer stores no axis to disagree with,
/// and the disagreement states the retired contextual relation could spell are
/// unrepresentable on this surface.
///
/// This replaces the landed
/// `refuse-mixed-pointwise-live-row-major-access-relations-before-lowering`
/// broad refusal: the mixed static/live subjects it closed under
/// `NumericalOrAccessRefinement` now fail here as exact
/// [`LiveRowMajorSourceRule::ConsumerMissingRelation`] coordinates, still
/// before any verified region or canonical identity exists.
fn verify_live_row_major_source(
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let is_live = |map: &LogicalAccess| {
        matches!(
            map,
            LogicalAccess::LiveRowMajorSource { .. } | LogicalAccess::LiveRowMajor
        )
    };
    if !reads.iter().any(|read| is_live(&read.map)) && !is_live(&write.map) {
        return Ok(());
    }
    let source_rule =
        |rule: LiveRowMajorSourceRule| ScheduledRegionDiagnostic::LiveRowMajorSource { rule };
    let ordinal = |position: usize| {
        AccessOrdinal::new(u32::try_from(position).expect("verified access count is bounded"))
    };
    let accesses = || reads.iter().chain(std::iter::once(write));
    // Rule 1: marker count. No marker leaves fieldless consumers with no axis
    // authority; a second marker would be a second runtime extent authority.
    let mut markers = accesses()
        .enumerate()
        .filter(|(_, access)| matches!(access.map, LogicalAccess::LiveRowMajorSource { .. }));
    let Some((first, marker)) = markers.next() else {
        return Err(source_rule(LiveRowMajorSourceRule::Missing));
    };
    if let Some((second, _)) = markers.next() {
        return Err(source_rule(LiveRowMajorSourceRule::Multiple {
            first: ordinal(first),
            second: ordinal(second),
        }));
    }
    // Rule 2: marker role and mode. The unique marker must be an input read —
    // a source on the owning write, an intermediate, or an output declares a
    // runtime input-axis operand no program input backs.
    if !matches!(marker.tensor, TensorRole::Input) || marker.mode != AccessMode::Read {
        return Err(source_rule(LiveRowMajorSourceRule::SourceNotInputRead {
            source: ordinal(first),
        }));
    }
    // Rule 3: complete live-relation coverage over every read and the final
    // write, at the first offending access in access order.
    if let Some((position, _)) = accesses()
        .enumerate()
        .find(|(_, access)| !is_live(&access.map))
    {
        return Err(source_rule(
            LiveRowMajorSourceRule::ConsumerMissingRelation {
                access: ordinal(position),
            },
        ));
    }
    Ok(())
}

/// Returns whether one pointwise region's reads use admissible boundary categories.
///
/// A read's access position is the expression leaf it serves. `TensorRole::Input`
/// is intentionally fieldless; the compiler binds each exact position against
/// its already-verified request subject. Two local rules remain intrinsic:
///
/// - **At most one read binds the materialized intermediate.** A second read
///   leaves nothing to say which materialization
///   edge it binds — which is exactly why the repeated-read admission above
///   cannot extend to it. `CoverAssembly::from_plan` refuses that a layer up
///   under `cover-intermediate-read-attribution`; stating it here is what stops
///   an intrinsically ambiguous region from being built at all, for a producer
///   that never passes through a cover.
/// - **A program output is never read.** Refused by name rather than under a
///   wildcard, so a role added to the vocabulary later is a build error here
///   instead of silently inheriting an admission nobody checked it for.
fn reads_bind_boundary_tensors_in_order(reads: &[Access]) -> bool {
    let mut intermediate_reads = 0_usize;
    for read in reads {
        match read.tensor {
            TensorRole::Input => {}
            TensorRole::Intermediate => {
                intermediate_reads += 1;
                if intermediate_reads > 1 {
                    return false;
                }
            }
            TensorRole::Output => return false,
        }
    }
    true
}

/// Returns whether one read map is admissible on a pointwise region.
///
/// A pointwise region evaluates one scalar program per output position, so a
/// read may address its operand however it likes *provided* the addressing is
/// a total function of the iteration coordinate with a discharged bounds
/// obligation. Five maps satisfy that and no others do:
///
/// - [`LogicalAccess::LinearIdentity`], the dense one-to-one read.
/// - [`LogicalAccess::ReindexBijection`], whose decodes are required to tile the
///   iteration domain exactly, so every operand element is read once.
/// - [`LogicalAccess::BroadcastReplication`], whose decodes are required to name
///   distinct result axes and leave at least one replicated.
/// - [`LogicalAccess::ParametricBroadcast`], the accepted sourced carrier.
///   Structural rank agreement is checked here; the environment proof is
///   [`crate::schedule::parametric::interpret_parametric_broadcast`].
/// - [`LogicalAccess::LiveRowMajorSource`] and the fieldless
///   [`LogicalAccess::LiveRowMajor`] consumer, provided the source gate above
///   already proved exactly one input-read marker and complete live-relation
///   coverage.
///
/// Both structural maps must state the region's own iteration shape as their
/// result shape. That is what stops a region from carrying an access relation
/// derived against some *other* domain — the decodes are divisors of this
/// region's linear coordinate, so a result shape that disagreed with the
/// iteration shape would make every divisor address the wrong window while still
/// satisfying its own internal consistency.
///
/// The remaining maps are refused by name rather than by a wildcard, so a map
/// added to the vocabulary later is a build error here instead of silently
/// inheriting a pointwise admission it was never checked for.
fn pointwise_read_map_is_admissible(
    map: &LogicalAccess,
    iteration_shape: &crate::shape::Shape,
) -> bool {
    match map {
        LogicalAccess::LinearIdentity
        | LogicalAccess::LiveRowMajorSource { .. }
        | LogicalAccess::LiveRowMajor => true,
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => {
            result_shape == iteration_shape
                && reindex_decodes_are_bijective(operand_shape, result_shape, axes)
        }
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            result_shape == iteration_shape
                && broadcast_decodes_are_replicating(operand_shape, result_shape, axes)
        }
        LogicalAccess::ParametricBroadcast { .. } => {
            parametric_broadcast_read_is_admissible(map, iteration_shape.rank())
        }
        // A scalar broadcast reads a rank-zero parameter and belongs to the
        // decode program; a packed carrier belongs to it too; and the two
        // reduction relations address a contributor domain a pointwise region
        // does not have. The partitioned-copy source map is refused by name
        // for the accepted reason: it is a derivation of a copy program a
        // pointwise region does not carry, so admitting it here would let the
        // map leak into arithmetic regions.
        // The gather source is a value-producing leaf like any other read: it
        // addresses one source element per iteration coordinate, with the
        // gathered axis supplied by the loaded U32. What makes that a *total*
        // function of the iteration coordinate with a discharged bounds
        // obligation is the association gate above, which has already proved
        // the paired address read and the retained static proof. Only the
        // domain agreement is restated here, for the reason the two structural
        // relations restate theirs: a relation derived against some other
        // domain would otherwise satisfy its own internal consistency.
        LogicalAccess::GatherSource { result_shape, .. } => result_shape == iteration_shape,
        LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. }
        | LogicalAccess::PartitionedCopySource => false,
    }
}

pub(super) fn verify_strict_affine_u4_dequantize(
    region: &ScheduledRegion,
    codes: &Access,
    scale: &Access,
    zero_point: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ScalarProgram::StrictAffineU4Dequantize {
        codes_role,
        scale_role,
        zero_point_role,
    } = numerical_program(region)?.0
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = *numerical_program(region)?.1;
    if *codes_role != STRICT_AFFINE_CODES_ROLE
        || *scale_role != STRICT_AFFINE_SCALE_ROLE
        || *zero_point_role != STRICT_AFFINE_ZERO_POINT_ROLE
        || codes.tensor != TensorRole::Input
        || codes.component_role != Some(*codes_role)
        || codes.mode != AccessMode::Read
        || codes.ownership.is_some()
        || codes.map
            != (LogicalAccess::PackedU4LsbZeroTail {
                logical_elements: region.schedule.work_items,
            })
        || scale.tensor != TensorRole::Input
        || scale.component_role != Some(*scale_role)
        || scale.mode != AccessMode::Read
        || scale.ownership.is_some()
        || scale.map != LogicalAccess::ScalarBroadcast
        || zero_point.tensor != TensorRole::Input
        || zero_point.component_role != Some(*zero_point_role)
        || zero_point.mode != AccessMode::Read
        || zero_point.ownership.is_some()
        || zero_point.map != LogicalAccess::ScalarBroadcast
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(region.schedule.reduction, ReductionTopology::None)
        || numerical.input_subnormals != SubnormalMode::Preserve
        || numerical.result_subnormals != SubnormalMode::Preserve
        || numerical.permits_contraction()
        || numerical.permits_reassociation()
        || numerical.permits_permutation()
        || numerical.permits_signed_zero_elimination()
        || numerical.nan_assumptions != ExceptionalValueAssumption::MakeNoAssumption
        || numerical.infinity_assumptions != ExceptionalValueAssumption::MakeNoAssumption
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[codes, scale, zero_point], write)
}
