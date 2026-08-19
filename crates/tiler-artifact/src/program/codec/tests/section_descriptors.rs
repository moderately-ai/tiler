//! Section purpose, disposition, schema, and the compatibility contract.

use super::super::super::keys::{
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use super::super::super::model::SchemaVersion;
use super::super::super::tests::{
    SCALE_BITS, declare_realization, default_artifact, formulas, fused_program, lowering_provider,
    payload, profile, selection, semantic_program, variant,
};
use super::super::super::{ArtifactProgramBuilder, CompilationEnvironment};
use super::super::decode::decode;
use super::super::encode::section_digest;
use super::super::error::{ArtifactCodecError, TagSubject};
use super::super::model::{Section, SectionDisposition, SectionKind};
use super::support::{encoded, envelope_of, manifest_offset, reseal};
use tiler_digest::DigestAlgorithm;

// -------------------------------------------------------------------------
// Section descriptor purpose, disposition, and schema
// -------------------------------------------------------------------------

/// A section digest is a *standalone* content address, not only an in-envelope
/// integrity check.
///
/// This is the property the descriptor change exists for. Inside a complete
/// envelope the purpose was already bound one level up, by the manifest
/// descriptor that names it and the manifest digest that covers that
/// descriptor. Lifted out of the envelope — which is exactly what
/// content-addressing a backend code section does — a digest over bytes alone
/// would give two sections of different purposes one address.
#[test]
fn equal_bytes_under_different_purposes_have_different_section_digests() {
    let bytes = b"the same section content".to_vec();
    let metadata = Section {
        kind: SectionKind::BackendPayloadMetadata,
        bytes: bytes.clone(),
    };
    let code = Section {
        kind: SectionKind::BackendPayloadCode,
        bytes,
    };
    assert_ne!(
        section_digest(DigestAlgorithm::GOVERNED, &metadata),
        section_digest(DigestAlgorithm::GOVERNED, &code),
    );
}

/// Locates the fixed-width prefix of the first section descriptor.
///
/// The prefix is `id | purpose | disposition | schema major | schema minor`,
/// which is distinctive enough to find unambiguously; a bare purpose tag is
/// one byte and occurs throughout the manifest.
fn first_section_descriptor(bytes: &[u8]) -> usize {
    let kind = SectionKind::KernelProgramSubject;
    let schema = kind.schema();
    let mut needle = 0_u32.to_be_bytes().to_vec();
    needle.push(kind.tag());
    needle.push(kind.disposition().tag());
    needle.extend_from_slice(&schema.major().to_be_bytes());
    needle.extend_from_slice(&schema.minor().to_be_bytes());
    manifest_offset(bytes, &needle)
}

/// A descriptor may not assert a skip permission its purpose does not have.
#[test]
fn a_section_disposition_that_contradicts_its_purpose_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 5] = SectionDisposition::Optional.tag();
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::SectionDispositionMismatch {
            section: 0,
            declared: SectionDisposition::Optional.tag(),
            expected: SectionDisposition::Required.tag(),
        }),
    );
}

/// A descriptor may not assert a content schema its purpose does not carry.
#[test]
fn a_section_content_schema_that_contradicts_its_purpose_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 6..at + 8].copy_from_slice(&7_u16.to_be_bytes());
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnsupportedSectionSchema {
            section: 0,
            major: 7,
            minor: 0,
        }),
    );
}

/// An unrecognized disposition tag is refused by name rather than defaulted.
#[test]
fn an_unknown_section_disposition_tag_is_rejected() {
    let mut bytes = encoded(&default_artifact());
    let at = first_section_descriptor(&bytes);
    bytes[at + 5] = 0x7f;
    reseal(&mut bytes);
    assert_eq!(
        decode(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::SectionDisposition,
            tag: 0x7f,
        }),
    );
}

/// Every governed section purpose declares a disposition and a schema.
///
/// Exhaustive over the vocabulary, so a purpose added without deciding either
/// fails here rather than inheriting whatever the match arm beside it said.
#[test]
fn every_section_purpose_declares_its_disposition_and_schema() {
    for kind in [
        SectionKind::KernelProgramSubject,
        SectionKind::BackendPayloadMetadata,
        SectionKind::BackendPayloadCode,
    ] {
        assert_eq!(SectionKind::from_tag(kind.tag()), Some(kind));
        assert_eq!(kind.disposition(), SectionDisposition::Required);
        assert_eq!(
            SectionDisposition::from_tag(kind.disposition().tag()),
            Some(kind.disposition()),
        );
        assert_eq!(kind.schema(), SchemaVersion::new(1, 0));
    }
}

/// A payload's compatibility contract is its own, not its variant's.
///
/// Two payloads that agree on backend, representation, schema, digest, and
/// execution policy but were built against different target profiles are two
/// distinct payloads with two distinct canonical keys — so an artifact carrying
/// one object per delivery position encodes each object's own contract rather
/// than one the loader would have to infer from whichever variant it routed to.
///
/// Sharing one object across variants declaring *different* profiles is not the
/// case this serves, and never was: `ArtifactProgramBuilder::check_subject`
/// refuses a second variant declaring a different profile, which
/// `super::super::super::tests::refuses_a_second_variant_declaring_a_different_target_profile`
/// pins.
#[test]
fn the_payload_compatibility_contract_participates_in_its_canonical_key() {
    let baseline = payload(0xa1);
    let mut elsewhere = baseline.clone();
    elsewhere.compatibility = TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.other").unwrap(),
        descriptor: baseline.compatibility.descriptor.clone(),
    };
    assert_ne!(baseline.canonical_key(), elsewhere.canonical_key());

    let mut redescribed = baseline.clone();
    redescribed.compatibility = TargetProfileRef {
        key: baseline.compatibility.key.clone(),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x09, 0x09]).unwrap(),
    };
    assert_ne!(
        baseline.canonical_key(),
        redescribed.canonical_key(),
        "the descriptor digest is part of the contract, not decoration",
    );
}

/// The compatibility contract reaches artifact identity and the envelope bytes.
#[test]
fn a_changed_payload_compatibility_contract_changes_the_artifact() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let build = |compatibility: TargetProfileRef| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let mut descriptor = payload(0xa1);
        descriptor.compatibility = compatibility;
        let id = draft.push_payload(descriptor).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, id, b"fused"))
            .unwrap();
        declare_realization(&mut draft, &program);
        draft.build().unwrap()
    };
    let baseline = build(profile());
    let elsewhere = build(TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.other").unwrap(),
        descriptor: profile().descriptor,
    });
    assert_ne!(
        baseline.canonical_identity(),
        elsewhere.canonical_identity(),
    );
    assert_ne!(encoded(&baseline), encoded(&elsewhere));
    assert_eq!(
        decode(&encoded(&elsewhere)).unwrap(),
        envelope_of(&elsewhere)
    );
}
