//! Payload objects and subjects as content-addressed sections.

use super::super::super::keys::{BackendKey, PayloadDigest, RepresentationKey};
use super::super::super::model::{ArtifactExecutionPolicy, SchemaVersion};
use super::super::super::tests::offered_physical;
use super::super::super::tests::{
    SCALE_BITS, declare_realization, default_artifact, formulas, fused_program, lowering_provider,
    profile, selection, semantic_program, variant,
};
use super::super::super::{
    ArtifactProgramBuilder, CompilationEnvironment, VerifiedArtifactProgram,
};
use super::super::decode::decode;
use super::super::encode::encode;
use super::super::error::{ArtifactCodecError, OrderedSubject};
use super::super::model::{SectionKind, position};
use super::super::payload::decode_metadata;
use super::support::{carried_artifact, encoded, envelope_of, payload_content, payload_metadata};

/// The descriptor's digest is the identity of the subject, not of the object.
///
/// This is the identity decision stated as a test: changing a compilation input
/// is a different artifact, and changing only the emitted bytes is not.
#[test]
fn payload_identity_follows_the_compilation_subject_and_not_the_object() {
    let baseline = carried_artifact(b"kernel void fused() {}", b"first-link");
    let relinked = carried_artifact(b"kernel void fused() {}", b"second-link");
    let recompiled = carried_artifact(b"kernel void other() {}", b"first-link");

    assert_eq!(
        baseline.canonical_identity(),
        relinked.canonical_identity(),
        "a non-reproducible linker must not change what artifact this is",
    );
    assert_ne!(
        encoded(&baseline),
        encoded(&relinked),
        "the object still travels, so the two encodings differ",
    );
    assert_ne!(
        baseline.canonical_identity(),
        recompiled.canonical_identity(),
        "a different compilation subject is a different artifact",
    );
}

/// Assembles a one-variant artifact delivering two payloads that carry one object.
///
/// The two compilation subjects differ and the two emitted objects are equal,
/// which is the shape the section table's content addressing is *for*: the
/// subjects must occupy two sections because they name two payloads, and the
/// object must occupy one because a section's address is its content.
fn twice_delivered_artifact(code: &[u8]) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let carry = |draft: &mut ArtifactProgramBuilder, source: &[u8]| {
        draft
            .push_carried_payload(
                BackendKey::new("tiler.metal").unwrap(),
                RepresentationKey::new("metallib").unwrap(),
                SchemaVersion::new(1, 0),
                profile(),
                ArtifactExecutionPolicy::NativeImage,
                None,
                payload_content(source, code),
            )
            .unwrap()
    };
    let first = carry(&mut draft, b"kernel void fused() {}");
    let second = carry(&mut draft, b"kernel void other() {}");
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, first, b"fused");
    spec.entries[0].implementation.payloads = vec![first, second];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

/// Two payloads carrying equal objects share exactly one section.
///
/// The property the section table's content addressing states, asserted rather
/// than argued: an envelope that framed the same object twice would grow by the
/// whole of a compiled library per delivery position, and a reader resolving
/// either position would still reach identical bytes — so the duplication would
/// be invisible to every other check in this suite.
#[test]
fn two_payloads_carrying_equal_objects_share_one_section() {
    let artifact = twice_delivered_artifact(b"\x00metallib\xff");
    let envelope = envelope_of(&artifact);
    let carried: Vec<super::super::model::PayloadSections> = envelope
        .payload_content()
        .iter()
        .map(|content| content.expect("both payloads are carried"))
        .collect();
    assert_eq!(carried.len(), 2, "the fixture delivers two payloads");
    assert_eq!(
        carried[0].code, carried[1].code,
        "equal objects are one content address and therefore one section",
    );
    assert_ne!(
        carried[0].metadata, carried[1].metadata,
        "two compilation subjects are two content addresses",
    );
    let of_kind = |kind: SectionKind| {
        envelope
            .sections()
            .iter()
            .filter(|section| section.kind == kind)
            .count()
    };
    assert_eq!(of_kind(SectionKind::BackendPayloadCode), 1);
    assert_eq!(of_kind(SectionKind::BackendPayloadMetadata), 2);
    let bytes = encode(&envelope).expect("the envelope encodes");
    assert_eq!(
        decode(&bytes).expect("a shared-object envelope decodes"),
        envelope,
        "the shared section survives the round trip",
    );
}

#[test]
fn carrying_a_payload_requires_the_governed_feature() {
    let carried = carried_artifact(b"kernel void fused() {}", b"link");
    assert!(
        envelope_of(&carried)
            .features()
            .contains(&"tiler.artifact.feature.embedded-payload-code".to_owned()),
    );
    assert!(
        !envelope_of(&default_artifact())
            .features()
            .contains(&"tiler.artifact.feature.embedded-payload-code".to_owned()),
        "a descriptor-only artifact must not require a reader to implement carrying",
    );
}

/// A code reference pointed at a compilation subject must be refused by name.
///
/// Both sections exist, both digests verify, and the manifest digest is
/// restamped by the encoder, so nothing in framing or integrity can catch this.
/// Only the purpose check can, which is why the reference carries one.
#[test]
fn a_payload_section_reference_of_the_wrong_purpose_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    envelope.payload_content[0] = Some(super::super::model::PayloadSections {
        metadata: sections.metadata,
        code: sections.metadata,
    });
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionPurposeMismatch {
            section: sections.metadata,
            expected: SectionKind::BackendPayloadCode.tag(),
            actual: SectionKind::BackendPayloadMetadata.tag(),
        }),
    );
}

/// A descriptor that claims a subject it does not carry must be refused.
#[test]
fn a_payload_digest_that_is_not_its_carried_subject_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    envelope.payloads[0].digest = PayloadDigest::from_bytes([0x01, 0x02, 0x03]).unwrap();
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::PayloadIdentityMismatch { payload: 0 }),
    );
}

/// A carried subject that does not parse is refused, not carried opaquely.
#[test]
fn a_carried_subject_that_is_not_payload_metadata_is_rejected() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    // At least as long as the versioned domain tag, so the case decides on the
    // tag rather than being pre-empted by a truncation.
    envelope.sections[position(sections.metadata)].bytes = vec![b'x'; 512];
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::BadPayloadMetadataDomain),
    );
}

/// A non-canonical collection inside a carried subject is refused.
#[test]
fn a_non_canonical_carried_subject_collection_is_rejected() {
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    metadata.provenance.components.reverse();
    assert_eq!(
        decode_metadata(&super::super::payload::encode_metadata(&metadata)),
        Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::ProvenanceComponent,
        }),
    );
}

/// Compiler flag order is meaning and must survive a round trip unsorted.
#[test]
fn carried_flag_order_is_retained_rather_than_canonicalized() {
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    metadata.provenance.compile_flags = vec![
        "-O2".to_owned(),
        "-ffast-math".to_owned(),
        "-fno-fast-math".to_owned(),
    ];
    let decoded = decode_metadata(&super::super::payload::encode_metadata(&metadata))
        .expect("declared flag order decodes");
    assert_eq!(
        decoded.provenance.compile_flags,
        [
            "-O2".to_owned(),
            "-ffast-math".to_owned(),
            "-fno-fast-math".to_owned(),
        ]
    );
}
