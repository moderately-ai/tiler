//! Incompetent forgeries: corrupted bytes, truncation, framing, and schema.

use super::super::super::model::{
    ArtifactExecutionPolicy, RoutingPolicy, StageDependencyData, StageDependencyReason,
};
use super::super::super::tests::default_artifact;
use super::super::decode::decode;
use super::super::encode::{
    HEADER_BYTES, MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN, MANIFEST_SCHEMA, encode, identity_digest,
};
use super::super::error::{ArtifactCodecError, TagSubject};
use super::support::{
    MANIFEST_LENGTH_AT, encoded, envelope_of, manifest_occurrences, manifest_offset, reseal,
};
use tiler_digest::{DIGEST_BYTES, DigestAlgorithm};
use tiler_ir::identity::push_slice;

// -------------------------------------------------------------------------
// Byte-level corruption
// -------------------------------------------------------------------------

/// Sweeps single-byte corruptions of one encoded envelope, exhaustively.
///
/// **Every byte, with no sampling anywhere.** The header and the framed section
/// stream carry fields read *before* any digest can speak for them — the magic,
/// the versions, the digest algorithm, the declared lengths and counts, each
/// section identifier and length, and each section's exact bytes — and the
/// manifest interior is covered by one digest over the whole run.
///
/// **The manifest interior used to be sampled at a stride of 61, and the reason
/// is gone rather than overridden.** Sampling was a cost decision: the sweep
/// took ~13 s, and flipping each interior byte exercises the same digest check
/// once per byte rather than a different check each time. Neither half of that
/// still holds. The envelope shrank from 26,126 bytes to 15,030 when ABI
/// expression identity moved to a linear encoding, and a decode fell from
/// 662 µs to 18.7 µs across the codec work of 2026-07-27.
///
/// **Measurement, 2026-07-27:** the fully exhaustive sweep runs in **132 ms**,
/// against ~70 ms for the sampled form it replaces. `reduce-the-codec-corruption-sweep-to-its-distinct-classes`
/// asked which of exhaustive or representative coverage the suite's standard
/// should be, and anticipated that a cheaper decode might dissolve the question.
/// It did: for 62 ms the property under test is "no single-byte corruption of
/// this envelope is accepted" rather than "no sampled single-byte corruption
/// is", and the stronger one needs no argument about which bytes are
/// representative.
///
/// A no-op corruption (`^= 0x00`) fails this test at byte 0, so the sweep can
/// say no rather than passing because every decode errors for some other
/// reason.
#[test]
fn single_byte_corruptions_are_rejected() {
    let bytes = encoded(&default_artifact());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest_end = HEADER_BYTES + manifest_len;
    let swept = (0..HEADER_BYTES)
        .chain(HEADER_BYTES..manifest_end)
        .chain(manifest_end..bytes.len());
    let mut forged = bytes.clone();
    for index in swept {
        forged[index] ^= 0xff;
        assert!(
            decode(&forged).is_err(),
            "flipping byte {index} produced an accepted envelope",
        );
        forged[index] = bytes[index];
    }
}

#[test]
fn every_truncation_is_rejected() {
    let bytes = encoded(&default_artifact());
    for length in 0..bytes.len() {
        assert!(
            decode(&bytes[..length]).is_err(),
            "a {length}-byte prefix decoded",
        );
    }
    // The two boundaries the reader distinguishes: too short to hold a header,
    // and a complete header whose declared total length is not what arrived.
    assert!(matches!(
        decode(&bytes[..4]),
        Err(ArtifactCodecError::Truncated { .. }),
    ));
    assert!(matches!(
        decode(&bytes[..bytes.len() - 1]),
        Err(ArtifactCodecError::TotalLengthMismatch { .. }),
    ));
}

#[test]
fn trailing_bytes_are_rejected() {
    let bytes = encoded(&default_artifact());
    let mut extended = bytes.clone();
    extended.push(0x00);
    assert!(matches!(
        decode(&extended),
        Err(ArtifactCodecError::TotalLengthMismatch { .. }),
    ));
    // Repairing the declared length does not make the extra byte admissible.
    reseal(&mut extended);
    assert_eq!(
        decode(&extended),
        Err(ArtifactCodecError::TrailingBytes { count: 1 }),
    );
}

#[test]
fn a_corrupted_manifest_is_rejected_before_it_is_parsed() {
    let mut bytes = encoded(&default_artifact());
    bytes[HEADER_BYTES + MANIFEST_DOMAIN.len()] ^= 0xff;
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::ManifestDigestMismatch)
    );
}

#[test]
fn a_corrupted_section_is_rejected_by_its_own_digest() {
    let mut bytes = encoded(&default_artifact());
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    // The manifest is resealed, so the section descriptor's digest is the only
    // thing standing between the forgery and acceptance.
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionDigestMismatch { section: 0 }),
    );
}

#[test]
fn a_flipped_tag_byte_is_rejected_by_name() {
    let mut bytes = encoded(&default_artifact());
    // The routing tag directly follows the manifest domain and the manifest and
    // component schema versions.
    let routing_at = HEADER_BYTES + MANIFEST_DOMAIN.len() + 4 + 16;
    assert_eq!(bytes[routing_at], RoutingPolicy::StablePriority.tag());
    bytes[routing_at] = 0x7f;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::RoutingPolicy,
            tag: 0x7f,
        }),
    );
}

/// The retired execution-policy tag is refused by name, not reinterpreted.
///
/// `ArtifactExecutionPolicy::RequiresDeviceTranslation` held wire tag `0x02`
/// until `route-or-refuse-the-device-translation-execution-policy` retired it.
/// Removing a variant *narrows* what decodes — the converse of an appended tag
/// — so the property that has to hold is that the withdrawn value refuses
/// rather than resolving to the surviving policy: an artifact declaring a
/// delivery step nothing performs must not silently become one declaring bytes
/// this loader will hand straight to a device.
///
/// `NativeImage` keeps tag `0x01`, so no artifact this vocabulary can still
/// express moves a byte, and `0x02` is never reassigned.
#[test]
fn the_retired_execution_policy_tag_is_refused_by_name() {
    const RETIRED_DEVICE_TRANSLATION_TAG: u8 = 0x02;

    // Tag `0x02` is unassigned at the vocabulary, which is what stops a decoder
    // from resolving it to anything at all.
    assert_eq!(
        ArtifactExecutionPolicy::from_tag(RETIRED_DEVICE_TRANSLATION_TAG),
        None,
    );
    assert_eq!(
        ArtifactExecutionPolicy::from_tag(ArtifactExecutionPolicy::NativeImage.tag()),
        Some(ArtifactExecutionPolicy::NativeImage),
    );

    let artifact = default_artifact();
    let [payload] = artifact.payloads() else {
        panic!("the fixture declares exactly one payload");
    };
    // The encoder writes the policy tag directly after the payload's digest,
    // compatibility key, and compatibility descriptor, so the anchor is built
    // from those fields' own bytes rather than guessed at. It is deliberately
    // *not* asserted unique: the same canonical run recurs in the identity
    // subjects the manifest also carries, and only the first is the payload
    // table the decoder reads a field from. Flipping the wrong run would not
    // produce this refusal, so the assertion below is what proves the position
    // rather than a comment claiming it.
    let mut anchor = Vec::new();
    push_slice(&mut anchor, payload.digest.as_bytes());
    push_slice(&mut anchor, payload.compatibility.key.as_str().as_bytes());
    push_slice(&mut anchor, payload.compatibility.descriptor.as_bytes());

    let mut bytes = encoded(&artifact);
    let first_run = bytes
        .windows(anchor.len())
        .position(|window| window == anchor.as_slice())
        .expect("the encoded manifest carries the payload descriptor");
    let policy_at = first_run + anchor.len();
    assert_eq!(
        bytes[policy_at],
        ArtifactExecutionPolicy::NativeImage.tag(),
        "the located byte must be the execution-policy tag",
    );
    bytes[policy_at] = RETIRED_DEVICE_TRANSLATION_TAG;
    // Resealed, so the manifest digest is not what rejects this.
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::ExecutionPolicy,
            tag: RETIRED_DEVICE_TRANSLATION_TAG,
        }),
    );
}

#[test]
fn a_length_prefix_that_disagrees_with_its_payload_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    // The semantic graph subject is the first length-prefixed run after the
    // (empty) feature list, which the routing tag precedes.
    let length_at = HEADER_BYTES + MANIFEST_DOMAIN.len() + 4 + 16 + 1 + 8;
    let declared = u64::from_be_bytes(bytes[length_at..length_at + 8].try_into().unwrap());
    assert!(declared > 0, "the fixture carries a semantic graph subject");
    let beyond_manifest =
        u64::try_from(bytes.len()).expect("the supported envelope length fits u64");
    bytes[length_at..length_at + 8].copy_from_slice(&beyond_manifest.to_be_bytes());
    reseal(&mut bytes);
    assert!(matches!(
        decode(&bytes),
        Err(ArtifactCodecError::Truncated { .. }),
    ));
}

#[test]
fn an_unknown_envelope_format_version_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    bytes[8..10].copy_from_slice(&2_u16.to_be_bytes());
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedEnvelopeFormat { major: 2, minor: 0 }),
    );
    let mut newer_minor = encoded(&default_artifact());
    newer_minor[10..12].copy_from_slice(&9_u16.to_be_bytes());
    reseal(&mut newer_minor);
    assert_eq!(
        decode(&newer_minor),
        Err(ArtifactCodecError::UnsupportedEnvelopeFormat { major: 1, minor: 9 }),
    );
}

#[test]
fn an_unknown_digest_algorithm_is_rejected_rather_than_inferred() {
    let mut bytes = encoded(&default_artifact());
    bytes[16] = 0x02;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedDigestAlgorithm { tag: 0x02 }),
    );
}

#[test]
fn an_unknown_manifest_or_component_schema_is_rejected() {
    let schema_at = HEADER_BYTES + MANIFEST_DOMAIN.len();
    let mut previous_manifest = encoded(&default_artifact());
    previous_manifest[schema_at..schema_at + 2].copy_from_slice(&7_u16.to_be_bytes());
    reseal(&mut previous_manifest);
    assert_eq!(
        decode(&previous_manifest),
        Err(ArtifactCodecError::UnsupportedManifestSchema { major: 7, minor: 0 }),
    );

    let mut manifest = encoded(&default_artifact());
    manifest[schema_at + 2..schema_at + 4].copy_from_slice(&5_u16.to_be_bytes());
    reseal(&mut manifest);
    assert_eq!(
        decode(&manifest),
        Err(ArtifactCodecError::UnsupportedManifestSchema {
            major: MANIFEST_SCHEMA.0,
            minor: 5,
        }),
    );

    let mut component = encoded(&default_artifact());
    component[schema_at + 4..schema_at + 6].copy_from_slice(&3_u16.to_be_bytes());
    reseal(&mut component);
    assert_eq!(
        decode(&component),
        Err(ArtifactCodecError::UnsupportedComponentSchema {
            component: super::super::error::ComponentSchemaKind::Program,
            major: 3,
            minor: 0,
        }),
    );
}

#[test]
fn an_unsupported_required_feature_is_rejected() {
    // The feature list is derived, so an unreadable requirement can only arrive
    // from a producer newer than this reader. The key is a fabricated future one
    // rather than a real reserved key: every key this build knows is now
    // supported, and a test that named one would start passing for the wrong
    // reason the moment that stopped being true — which is exactly what happened
    // to its previous spelling when `multi-stage-program` became readable.
    const FUTURE_FEATURE: &str = "tiler.artifact.feature.from-a-later-producer";
    let mut envelope = envelope_of(&default_artifact());
    envelope.features = vec![FUTURE_FEATURE.to_owned()];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedRequiredFeature {
            feature: FUTURE_FEATURE.to_owned(),
        }),
    );
}

/// An execution order that does not sequence every entry exactly once is refused.
///
/// Both directions of "not a permutation", because they fail for different
/// reasons and an implementation can get one right and the other wrong: an order
/// that omits an entry leaves a stage undispatched, and one that repeats an
/// entry dispatches it twice. Either would run a program the artifact does not
/// describe.
#[test]
fn an_execution_order_that_is_not_a_permutation_is_rejected() {
    for order in [vec![], vec![0_u32, 0]] {
        let mut envelope = envelope_of(&default_artifact());
        envelope.variants[0].execution_order = order;
        let bytes = encode(&envelope).expect("a forged envelope still encodes");
        assert!(
            matches!(
                decode(&bytes),
                Err(ArtifactCodecError::StageOrderNotAPermutation { .. })
            ),
            "an order that is not a permutation must be refused as such",
        );
    }
}

/// A dependency edge that orders an entry against itself is refused.
///
/// An entry cannot precede itself, and an edge saying so would make the order
/// unsatisfiable rather than merely wrong — every order violates it. Refused as
/// a malformed edge rather than reported as an ordering violation, so the
/// diagnostic names the defect and not its consequence.
#[test]
fn a_stage_dependency_on_itself_is_rejected() {
    let mut envelope = envelope_of(&default_artifact());
    envelope.variants[0].dependencies = vec![StageDependencyData {
        predecessor: 0,
        successor: 0,
        reason: StageDependencyReason::Data,
    }];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert!(
        matches!(
            decode(&bytes),
            Err(ArtifactCodecError::StageDependencyOnItself { .. })
        ),
        "a self-edge must be refused as a malformed edge",
    );
}

// `StageDependencyOutOfOrder` is deliberately not reached from this module, and
// the reason is structural rather than an omission: it needs two entries whose
// stated order contradicts an edge between them, and every fixture here packages
// a single-stage program, where any edge with distinct endpoints cannot be
// built at all. `prototypes/serial-sum-compile`'s
// `a_multi_stage_variant_round_trips_with_a_recoverable_sequence` exercises the
// satisfied direction against a real two-stage plan; the contradicting direction
// wants a two-stage fixture here and is owed one.

/// The manifest's declared identity is refused when it is not the content's.
///
/// The declaration is a digest of the derived identity under its own domain, so
/// the forgery flips a byte of that digest rather than of an identity preimage —
/// the manifest carries none since schema `15.0`. What the case proves is
/// unchanged and is the reason the run exists: a producer whose two derivations
/// of one artifact disagree is refused, over the identical set of
/// disagreements, and the whole of `validate` and the canonicity backstop pass
/// on the way there.
#[test]
fn a_forged_identity_is_rejected() {
    let artifact = default_artifact();
    let bytes = encoded(&artifact);
    let declared = identity_digest(DigestAlgorithm::GOVERNED, artifact.canonical_identity());
    let at = manifest_offset(&bytes, declared.as_bytes());
    let mut forged = bytes;
    forged[at] ^= 0xff;
    reseal(&mut forged);
    assert_eq!(
        decode(&forged),
        Err(ArtifactCodecError::ArtifactIdentityMismatch),
    );
}

/// The declared identity is a digest under its own domain, not the identity.
///
/// Two claims one assertion cannot make together. The manifest no longer
/// contains the identity preimage anywhere — which is the 49% of the envelope
/// this schema step removed — and what stands in its place is the digest a
/// decoder re-derives and compares, at the manifest's exact end.
#[test]
fn the_manifest_declares_its_identity_by_digest_and_carries_no_preimage() {
    let artifact = default_artifact();
    let bytes = encoded(&artifact);
    let identity = artifact.canonical_identity().as_bytes();
    assert!(
        identity.len() > DIGEST_BYTES,
        "the fixture's identity must be long enough for its absence to mean something",
    );
    assert!(
        manifest_occurrences(&bytes, identity).is_empty(),
        "the manifest must not carry the canonical identity preimage",
    );
    let declared = identity_digest(DigestAlgorithm::GOVERNED, artifact.canonical_identity());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let end = HEADER_BYTES + manifest_len;
    assert_eq!(
        &bytes[end - DIGEST_BYTES..end],
        declared.as_bytes(),
        "the manifest ends with the digest of the identity it declares",
    );
    // A digest under a different domain is a different value, which is what a
    // separate `identity-digest` domain buys: the manifest digest already covers
    // these very bytes, so the two must not be one subject.
    assert_ne!(
        declared.as_bytes(),
        DigestAlgorithm::GOVERNED
            .digest(
                MANIFEST_DIGEST_DOMAIN,
                artifact.canonical_identity().as_bytes()
            )
            .as_bytes(),
    );
}
