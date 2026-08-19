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
use crate::schedule::error::{LiveRowMajorSourceRule, ScheduledRegionDiagnostic};
use crate::schedule::handles::AccessOrdinal;
use crate::schedule::model::{
    Access, AccessMode, LogicalAccess, ReductionTopology, ScalarProgram, ScheduledRegion,
    TensorRole, broadcast_decodes_are_replicating, reindex_decodes_are_bijective,
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
    if reads.is_empty() || reads.len() != input_count {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    }
    // The source-relation gate runs before the broad access-contract and
    // refinement gates, so a marker-count, marker-role, or missing-consumer
    // defect is named under its own dedicated rule rather than collapsing into
    // `AccessContract` or `NumericalOrAccessRefinement` — the accepted
    // precedence the fieldless-marker surface states.
    verify_live_row_major_source(reads, write)?;
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
        || !reads
            .iter()
            .all(|read| pointwise_read_map_is_admissible(&read.map, &region.index.iteration_shape))
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
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
