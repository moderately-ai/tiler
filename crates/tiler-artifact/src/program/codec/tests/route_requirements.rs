//! Live-device route requirement rows through the envelope.

use super::super::super::MAX_ROUTE_REQUIREMENTS;
use super::super::super::error::ArtifactBuildError;
use super::super::super::requirement::{
    RouteRequirement, RouteRequirementError, RouteResourceDimension,
};
use super::super::super::tests::{requiring_artifact, route_feature, route_resource};
use super::super::decode::decode;
use super::super::encode::{HEADER_BYTES, encode};
use super::super::error::{ArtifactCodecError, CodecLimitKind, OrderedSubject, TagSubject};
use super::super::model::FEATURE_ROUTE_REQUIREMENTS;
use super::support::{MANIFEST_LENGTH_AT, encoded, envelope_of, reseal};

// -------------------------------------------------------------------------
// Live-device route requirements
// -------------------------------------------------------------------------

/// A required quantity no other fixture field can collide with, for locating a
/// resource row.
const PROBE_QUANTITY: u64 = 0x0000_0000_dead_beef;

/// Returns the absolute offset of the resource row in an encoded manifest.
///
/// Located by its exact wire spelling — kind tag, dimension tag, and the
/// distinctive quantity — rather than by a computed offset, so the search fails
/// loudly if the row's layout changes instead of quietly patching another field.
///
/// **The pattern occurs exactly once, and that is asserted rather than worked
/// around.** It occurred twice until manifest schema `15.0`, the second time
/// inside the identity *preimage* the manifest then carried; the manifest now
/// declares its identity by digest, so the only spelling of the row left in
/// these bytes is the row. The identity still folds the row's canonical bytes —
/// which is the property that makes a declared device precondition part of what
/// the artifact *is* — but through a digest this search cannot see. A count
/// other than one would mean something else in the manifest now collides with
/// the row, or that the manifest carries a second copy of it again.
fn resource_row_offset(bytes: &[u8]) -> usize {
    let mut pattern = vec![0x01, RouteResourceDimension::SubgroupThreads.tag()];
    pattern.extend_from_slice(&PROBE_QUANTITY.to_be_bytes());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let found: Vec<usize> = bytes[HEADER_BYTES..HEADER_BYTES + manifest_len]
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(offset, _)| offset)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "the row is written once and the identity it folds into is carried as a digest",
    );
    HEADER_BYTES + found[0]
}

/// Rows survive an encode/decode round trip, and their presence is declared.
///
/// Three states rather than one: no rows, one row, and several of both kinds.
/// The feature key is derived from content, so an artifact requiring nothing
/// must *not* declare it — otherwise a reader that predates the family would be
/// refused an artifact it can honour.
#[test]
fn route_requirements_round_trip_and_declare_their_feature() {
    let empty = requiring_artifact(&[]);
    let decoded = decode(&encoded(&empty)).expect("the zero-row envelope decodes");
    assert!(decoded.variants[0].route_requirements.is_empty());
    assert!(
        !decoded
            .features()
            .iter()
            .any(|feature| feature == FEATURE_ROUTE_REQUIREMENTS),
        "a route requiring nothing must not demand the feature",
    );

    let rows = [
        route_resource(PROBE_QUANTITY),
        route_feature(
            "tiler.metal.route-requirement.minimum-gpu-family",
            1,
            b"apple9",
        ),
        route_feature("tiler.metal.route-requirement.other", 3, b"payload"),
    ];
    let artifact = requiring_artifact(&rows);
    let bytes = encoded(&artifact);
    let decoded = decode(&bytes).expect("the requiring envelope decodes");
    assert_eq!(decoded.variants[0].route_requirements.len(), rows.len());
    let mut expected: Vec<_> = rows.to_vec();
    expected.sort_by_cached_key(RouteRequirement::canonical_bytes);
    assert_eq!(decoded.variants[0].route_requirements, expected);
    assert!(
        decoded
            .features()
            .iter()
            .any(|feature| feature == FEATURE_ROUTE_REQUIREMENTS),
        "an artifact carrying rows must demand the feature",
    );
    assert_eq!(
        decoded.canonical_identity().expect("identity re-derives"),
        *artifact.canonical_identity(),
    );
    // Re-encoding the decoded view is byte-identical, which is what makes the
    // canonical-form check above a statement about this artifact's one encoding.
    assert_eq!(
        encode(&decoded).expect("the decoded view re-encodes"),
        bytes
    );
}

/// Declaration order is presentation-only, and a row is identity.
///
/// The two halves are the same property from both sides: reordering must not
/// change the bytes, and changing a row must.
#[test]
fn route_requirement_order_is_presentation_and_content_is_identity() {
    let first = route_feature("tiler.metal.route-requirement.a", 1, b"x");
    let second = route_resource(PROBE_QUANTITY);
    let forward = requiring_artifact(&[first.clone(), second.clone()]);
    let reversed = requiring_artifact(&[second, first.clone()]);
    assert_eq!(forward.canonical_identity(), reversed.canonical_identity());
    assert_eq!(encoded(&forward), encoded(&reversed));

    let none = requiring_artifact(&[]);
    assert_ne!(
        forward.canonical_identity(),
        none.canonical_identity(),
        "a declared device precondition is part of what an artifact is",
    );
    let changed = requiring_artifact(&[
        route_feature("tiler.metal.route-requirement.a", 1, b"y"),
        route_resource(PROBE_QUANTITY),
    ]);
    assert_ne!(
        forward.canonical_identity(),
        changed.canonical_identity(),
        "a payload naming a different capability is a different artifact",
    );
}

/// An unrecognized row kind or dimension tag is refused with the rejected byte.
///
/// Patched in the encoded manifest and resealed, so what refuses is the tag
/// table rather than the manifest digest a forger would have recomputed. The
/// unpatched neighbour decodes, which is what makes each rejection attributable.
#[test]
fn an_unrecognized_route_requirement_tag_is_rejected() {
    let bytes = encoded(&requiring_artifact(&[route_resource(PROBE_QUANTITY)]));
    decode(&bytes).expect("the unperturbed requiring envelope decodes");
    let at = resource_row_offset(&bytes);

    for (offset, subject) in [
        (at, TagSubject::RouteRequirementKind),
        (at + 1, TagSubject::RouteResourceDimension),
    ] {
        let mut forged = bytes.clone();
        forged[offset] = 0x7f;
        reseal(&mut forged);
        assert_eq!(
            decode(&forged),
            Err(ArtifactCodecError::UnknownTag { subject, tag: 0x7f }),
            "an unrecognized tag must be refused by name with its rejected byte",
        );
    }
}

/// A zero required quantity is refused on decode as the model rule it breaks.
///
/// The constructor refuses it, so an envelope carrying one can only have been
/// written by something other than this builder — which is exactly the case a
/// decoder exists to refuse rather than trust.
#[test]
fn a_zero_required_quantity_is_rejected_on_decode() {
    let bytes = encoded(&requiring_artifact(&[route_resource(PROBE_QUANTITY)]));
    let at = resource_row_offset(&bytes);
    let mut forged = bytes;
    forged[at + 2..at + 10].copy_from_slice(&0_u64.to_be_bytes());
    reseal(&mut forged);
    assert_eq!(
        decode(&forged),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::InvalidRouteRequirement {
                cause: RouteRequirementError::ZeroResourceQuantity {
                    dimension: RouteResourceDimension::SubgroupThreads,
                },
            }),
        }),
    );
}

/// A well-formed but non-canonical row order is refused rather than normalized.
#[test]
fn a_non_canonical_route_requirement_order_is_rejected() {
    let artifact = requiring_artifact(&[
        route_resource(PROBE_QUANTITY),
        route_feature("tiler.metal.route-requirement.a", 1, b"x"),
    ]);
    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].route_requirements.swap(0, 1);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::RouteRequirement,
        }),
    );
}

/// Two rows constraining one subject are refused on decode as well as at construction.
///
/// Re-proven here because this envelope was never built: a forger writes two
/// contradictory rows, the encoder stamps a correct digest and identity for
/// them, and admitting them would let a consumer satisfy the weaker one.
#[test]
fn duplicate_route_requirement_subjects_are_rejected_on_decode() {
    let artifact = requiring_artifact(&[route_feature("tiler.metal.route-requirement.a", 1, b"x")]);
    let mut envelope = envelope_of(&artifact);
    let row = envelope.variants[0].route_requirements[0].clone();
    // A second row on the same subject with a different payload: distinct
    // canonical bytes, so canonical order still holds and only the subject
    // check can refuse it.
    envelope.variants[0].route_requirements.push(route_feature(
        "tiler.metal.route-requirement.a",
        1,
        b"y",
    ));
    envelope.variants[0]
        .route_requirements
        .sort_by_cached_key(RouteRequirement::canonical_bytes);
    assert_eq!(envelope.variants[0].route_requirements.len(), 2);
    assert_eq!(
        row.subject(),
        envelope.variants[0].route_requirements[0].subject()
    );
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert!(
        matches!(
            decode(&bytes),
            Err(ArtifactCodecError::ModelRule { ref cause })
                if matches!(
                    **cause,
                    ArtifactBuildError::DuplicateRouteRequirementSubject { .. },
                )
        ),
        "two rows on one subject must be refused by name",
    );
}

/// More rows than the governed bound admits are refused before allocation.
#[test]
fn too_many_route_requirements_are_rejected() {
    let artifact = requiring_artifact(&[route_resource(PROBE_QUANTITY)]);
    let mut envelope = envelope_of(&artifact);
    envelope.variants[0].route_requirements = (0..=MAX_ROUTE_REQUIREMENTS)
        .map(|index| {
            let index = u8::try_from(index).expect("the governed bound is well under u8::MAX");
            route_feature("tiler.metal.route-requirement.a", 1, &[u8::MAX, index])
        })
        .collect();
    envelope.variants[0]
        .route_requirements
        .sort_by_cached_key(RouteRequirement::canonical_bytes);
    assert!(
        matches!(
            encode(&envelope),
            Err(ArtifactCodecError::Limit {
                resource: CodecLimitKind::RouteRequirements,
                ..
            }),
        ),
        "the encoder refuses to write a variant no reader would admit",
    );
}
