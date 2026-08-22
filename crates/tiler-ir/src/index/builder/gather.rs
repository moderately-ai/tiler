//! The closed authority that discharges one gather's index-bounds obligation.
//!
//! Two things make this module the *only* producer of either record. First, the
//! two identity wrappers have no public constructor and no byte conversion, so
//! holding one is itself evidence that the derivation below ran. Second, the
//! kind precedence is fixed here rather than selected by a caller: no sample,
//! caller assertion, target fact, profile, or reference run can mint a proof,
//! and a valid gather that neither closed argument reaches receives the exact
//! invocation requirement rather than an optimistic proof or an `Unknown`.
//!
//! The obligation is *total*. Every admitted gather leaves here with exactly
//! one of the two records, which is why [`GatherIndexBoundsResolutionData`] has
//! no absent arm — "not proved" and "no obligation" would otherwise be one
//! value.

use super::super::model::{
    CompactedGatherReadAccess, DimensionData, GatherIndexBoundsProof,
    GatherIndexBoundsProofIdentity, GatherIndexBoundsProofKind, GatherIndexBoundsResolutionData,
    GatherIndexBoundsSubject, GatherIndexValidationRequirement,
    GatherIndexValidationRequirementIdentity, IndexExprData, IndexNode, TensorData,
};
use super::{
    CanonicalIndexRegionIdentity, IndexDomainFactSource, VerifiedDimensionId, VerifiedRegionOwner,
    VerifiedTensorAccessId, VerifiedTensorId, bounded_index, push_len, push_slice,
};
use crate::semantic::gather_result_shape;
use crate::shape::{Axis, ExtentSources, Shape};

const GATHER_INDEX_BOUNDS_PROOF_DOMAIN: &[u8] = b"tiler.gather-index-bounds-proof.v1\0";
const GATHER_INDEX_VALIDATION_REQUIREMENT_DOMAIN: &[u8] =
    b"tiler.gather-index-validation-requirement.v1\0";

/// Every exact U32 value is below this bound, in mathematical extent space.
///
/// Compared as `u64` and never narrowed into U32, where the constant itself
/// would wrap to zero and turn the strongest source into the weakest one.
const U32_UNIVERSE: u64 = 1 << 32;

/// Derives the one record that discharges `gather`'s index-bounds obligation.
///
/// The fact source is derived **first**, from the complete proof subject, and
/// deliberately not from whichever short circuit ended up concluding. A proof
/// that stopped at the first zero result extent still records that a declared
/// symbol participated in the subject it reasoned over, because `Program` is
/// the strong claim that the *complete* population was literal — the same rule
/// `access_fact_source` states for a direct access.
///
/// Because this surface rejects every nonliteral boundary and every nonliteral
/// domain extent at construction, `ShapeEnvironment` is reachable here only
/// when a coordinate expression names a declared symbol. That is coordinate
/// evaluation, not sourced boundary support.
#[expect(
    clippy::too_many_arguments,
    reason = "the complete proof subject is the argument list; bundling it into a \
              struct would let a caller assemble a subject the region never held"
)]
pub(super) fn derive_gather_index_bounds(
    region: &CanonicalIndexRegionIdentity,
    access: VerifiedTensorAccessId,
    owner: VerifiedRegionOwner,
    gather: &CompactedGatherReadAccess,
    tensors: &[TensorData],
    dimensions: &[DimensionData],
    expressions: &[IndexExprData],
    sources: Option<&ExtentSources>,
) -> GatherIndexBoundsResolutionData {
    let source_tensor = &tensors[gather.source as usize];
    let index_tensor = &tensors[gather.index as usize];
    let source_shape = source_tensor
        .shape
        .as_static()
        .expect("an admitted gather source boundary is authored wholly literal")
        .clone();
    let index_shape = index_tensor
        .shape
        .as_static()
        .expect("an admitted gather index boundary is authored wholly literal")
        .clone();
    let axis = Axis::new(gather.axis);
    let source_extent = source_shape.extents()[gather.axis as usize];
    // The same authority `gather_read` derived the declared domain against, so
    // the proof subject's result shape and the admitted access's result domain
    // cannot disagree.
    let (_, result_shape) = gather_result_shape(axis, &source_shape, &index_shape)
        .expect("an admitted gather satisfied the rank and axis rules at construction");

    let facts = subject_fact_source(gather, dimensions, expressions, sources);

    let subject = GatherIndexBoundsSubject {
        region: region.clone(),
        access,
        source: VerifiedTensorId::from_verified(owner, gather.source),
        index: VerifiedTensorId::from_verified(owner, gather.index),
        source_type: source_tensor.value_type.clone(),
        index_type: index_tensor.value_type.clone(),
        source_shape,
        index_shape,
        result_shape,
        axis,
        source_extent,
        domain: gather
            .domain
            .iter()
            .copied()
            .map(|dimension| VerifiedDimensionId::from_verified(owner, dimension))
            .collect(),
    };

    // The closed kind precedence. The empty-result argument wins even when the
    // source-axis universe argument also holds, because a domain that visits no
    // point places no obligation on any value at all — reporting the weaker
    // universe claim there would attribute the conclusion to the wrong premise.
    //
    // Inspecting *every* result extent rather than the index shape alone is the
    // repaired rule: a source of `[0, 5]` gathered on axis 1 by an index of
    // `[3]` has result `[0, 3]` and is vacuous even though the index is
    // inhabited.
    let kind = if subject
        .result_shape
        .extents()
        .iter()
        .any(|extent| extent.get() == 0)
    {
        Some(GatherIndexBoundsProofKind::VacuousEmptyResultDomain)
    } else if source_extent.get() >= U32_UNIVERSE {
        Some(GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent)
    } else {
        None
    };

    if let Some(kind) = kind {
        let identity = GatherIndexBoundsProofIdentity(encode_gather_bounds_identity(
            GATHER_INDEX_BOUNDS_PROOF_DOMAIN,
            Some((kind, facts)),
            &subject,
        ));
        GatherIndexBoundsResolutionData::StaticallyProved(GatherIndexBoundsProof {
            identity,
            kind,
            facts,
            subject,
        })
    } else {
        let identity = GatherIndexValidationRequirementIdentity(encode_gather_bounds_identity(
            GATHER_INDEX_VALIDATION_REQUIREMENT_DOMAIN,
            None,
            &subject,
        ));
        GatherIndexBoundsResolutionData::InvocationValidationRequired(
            GatherIndexValidationRequirement { identity, subject },
        )
    }
}

/// Returns which facts the **complete** proof subject was allowed to read.
///
/// Scans the access domain, both boundary shapes, and every source and index
/// coordinate expression transitively. Boundary and domain extents cannot be
/// symbolic on this surface, so the scan is retained rather than assumed away:
/// it is what keeps this answer correct if a later decision admits sourced
/// gather boundaries, and what makes the coordinate case the only reachable
/// route to `ShapeEnvironment` today.
fn subject_fact_source(
    gather: &CompactedGatherReadAccess,
    dimensions: &[DimensionData],
    expressions: &[IndexExprData],
    sources: Option<&ExtentSources>,
) -> IndexDomainFactSource {
    let _ = sources;
    let domain_reads_environment = gather
        .domain
        .iter()
        .any(|dimension| dimensions[*dimension as usize].extent.symbol().is_some());
    let coordinates_read_environment = gather
        .source_coordinates
        .iter()
        .chain(gather.index_coordinates.iter())
        .any(|coordinate| node_reads_environment(*coordinate, dimensions, expressions));
    if domain_reads_environment || coordinates_read_environment {
        IndexDomainFactSource::ShapeEnvironment
    } else {
        IndexDomainFactSource::Program
    }
}

/// Whether one compacted expression names a declared symbol, transitively.
///
/// Mirrors `IndexRegionBuilder::expression_reads_environment` over compacted
/// records: compaction has already remapped every child ordinal, so the walk
/// follows the same edges without a separate reachability pass.
fn node_reads_environment(
    expression: u32,
    dimensions: &[DimensionData],
    expressions: &[IndexExprData],
) -> bool {
    match &expressions[expression as usize].node {
        IndexNode::Constant(_) => false,
        IndexNode::Dimension(dimension) => {
            dimensions[*dimension as usize].extent.symbol().is_some()
        }
        IndexNode::LinearCombination { terms, .. } => terms.iter().any(|term| {
            term.coefficient.symbol().is_some()
                || node_reads_environment(term.value, dimensions, expressions)
        }),
        IndexNode::FloorDiv { dividend, divisor } | IndexNode::Modulo { dividend, divisor } => {
            divisor.symbol().is_some() || node_reads_environment(*dividend, dimensions, expressions)
        }
    }
}

/// Frames one gather bounds record's canonical bytes.
///
/// `evidence` is `Some` for a proof and `None` for a requirement, which is the
/// whole difference between the two encodings: a requirement writes neither a
/// proof-kind tag nor a fact-source tag, because it is not proof evidence. The
/// fact source appears **once**, on the proof only, and the proof kind is
/// likewise written once here rather than copied into schedule state.
///
/// Every variable component is length-framed and every integer big-endian, so
/// swapping the source and index bindings, dropping the axis, or shortening the
/// domain each changes the bytes.
fn encode_gather_bounds_identity(
    domain: &[u8],
    evidence: Option<(GatherIndexBoundsProofKind, IndexDomainFactSource)>,
    subject: &GatherIndexBoundsSubject,
) -> Vec<u8> {
    let mut out = domain.to_vec();
    if let Some((kind, facts)) = evidence {
        out.push(match kind {
            GatherIndexBoundsProofKind::VacuousEmptyResultDomain => 0x01,
            GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent => 0x02,
        });
        out.push(facts.tag());
    }
    push_slice(&mut out, subject.region.as_bytes());
    out.extend_from_slice(&bounded_index(subject.access.as_usize()).to_be_bytes());
    out.extend_from_slice(&bounded_index(subject.source.as_usize()).to_be_bytes());
    out.extend_from_slice(&bounded_index(subject.index.as_usize()).to_be_bytes());
    push_slice(
        &mut out,
        subject.source_type.canonical_encoding().as_bytes(),
    );
    push_slice(&mut out, subject.index_type.canonical_encoding().as_bytes());
    push_shape(&mut out, &subject.source_shape);
    push_shape(&mut out, &subject.index_shape);
    push_shape(&mut out, &subject.result_shape);
    out.extend_from_slice(&subject.axis.get().to_be_bytes());
    out.extend_from_slice(&subject.source_extent.get().to_be_bytes());
    push_len(&mut out, subject.domain.len());
    for dimension in &subject.domain {
        out.extend_from_slice(&bounded_index(dimension.as_usize()).to_be_bytes());
    }
    out
}

fn push_shape(out: &mut Vec<u8>, shape: &Shape) {
    push_len(out, shape.rank());
    for extent in shape.extents() {
        out.extend_from_slice(&extent.get().to_be_bytes());
    }
}
