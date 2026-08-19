//! Payload provenance fields, and how the platform block reaches the wire.

use super::super::super::error::{ArtifactBuildError, ProvenanceField};
use super::super::error::{ArtifactCodecError, TagSubject};
use super::super::payload::{
    PAYLOAD_PLATFORM_UNVERSIONED_TAG, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    PayloadSdkIdentity, decode_metadata,
};
use super::support::payload_metadata;

// -------------------------------------------------------------------------
// Payload provenance: which fields a shape owes, and how the platform block
// reaches the wire
// -------------------------------------------------------------------------

/// The fixture above, restated by a backend whose toolchain has no SDK.
fn unversioned_payload_metadata(source: &[u8]) -> PayloadMetadata {
    let mut metadata = payload_metadata(source);
    metadata.provenance.platform = PayloadPlatform::Unversioned;
    metadata
}

/// A backend with no SDK states that, and its statement survives a round trip.
///
/// The point is the *absence* being carried rather than inferred: a decoder
/// hands back `Unversioned` because the producer said so, not because it found
/// blank fields and guessed.
#[test]
fn a_payload_with_no_sdk_states_that_and_round_trips() {
    let metadata = unversioned_payload_metadata(b"scalar-image");
    let decoded = decode_metadata(&super::super::payload::encode_metadata(&metadata))
        .expect("an unversioned payload decodes");
    assert_eq!(decoded.provenance.platform, PayloadPlatform::Unversioned);
    assert_eq!(decoded, metadata);
}

/// The widening is one appended byte, and the two shapes cannot collide.
///
/// This is the injectivity argument as an assertion rather than as prose. The
/// versioned encoding of a record with the platform positions blank is exactly
/// the unversioned encoding minus its tag, so the *only* thing separating the
/// two classes is that one byte — and the second half of the case is why that
/// suffices: the blank versioned record has no identity at all, so nothing a
/// producer can publish occupies the bytes an unversioned payload would need in
/// order to be mistaken for one.
#[test]
fn the_unversioned_encoding_is_the_versioned_one_plus_one_appended_tag() {
    let mut blank = payload_metadata(b"scalar-image");
    blank.provenance.platform = PayloadPlatform::VersionedSdk {
        deployment_major: 0,
        deployment_minor: 0,
        sdk: PayloadSdkIdentity {
            name: String::new(),
            version: String::new(),
            build: String::new(),
        },
    };
    let blank_bytes = super::super::payload::encode_metadata(&blank);
    let unversioned_bytes =
        super::super::payload::encode_metadata(&unversioned_payload_metadata(b"scalar-image"));

    let mut expected = blank_bytes.clone();
    expected.push(PAYLOAD_PLATFORM_UNVERSIONED_TAG);
    assert_eq!(
        unversioned_bytes, expected,
        "the unversioned shape must be the untagged grammar plus exactly one tag",
    );

    assert_eq!(
        blank.identity(),
        Err(ArtifactBuildError::IncompletePayloadProvenance {
            field: ProvenanceField::DeploymentMinimum,
        }),
        "a versioned payload with blank platform fields must have no identity",
    );
    assert_eq!(
        decode_metadata(&blank_bytes),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::IncompletePayloadProvenance {
                field: ProvenanceField::DeploymentMinimum,
            }),
        }),
    );
}

/// Every field a payload's declared shape owes is refused by name when empty.
///
/// The refusal is raised where the payload's identity is derived, so a subject
/// that does not state what it claims to state cannot be named, cached, or
/// pushed. The last case is the one the Apple-shaped record could not express:
/// the SDK fields are owed *because this payload declared a versioned SDK*, and
/// the case below it proves the same payload owes none of them once it declares
/// it has none.
#[test]
fn an_owed_provenance_field_left_empty_is_refused_by_name() {
    let versioned = |field: ProvenanceField, perturb: fn(&mut PayloadProvenance)| {
        let mut metadata = payload_metadata(b"kernel void fused() {}");
        perturb(&mut metadata.provenance);
        assert_eq!(
            metadata.identity(),
            Err(ArtifactBuildError::IncompletePayloadProvenance { field }),
        );
    };

    versioned(ProvenanceField::Toolchain, |provenance| {
        provenance.toolchain = String::new();
    });
    versioned(ProvenanceField::Target, |provenance| {
        provenance.target = String::new();
    });
    versioned(ProvenanceField::Family, |provenance| {
        provenance.family = String::new();
    });
    versioned(ProvenanceField::Language, |provenance| {
        provenance.language = String::new();
    });
    versioned(ProvenanceField::ToolComponentRole, |provenance| {
        provenance.components[0].role = String::new();
    });
    versioned(ProvenanceField::ToolComponentVersion, |provenance| {
        provenance.components[1].version = String::new();
    });
    versioned(ProvenanceField::DeploymentMinimum, |provenance| {
        set_deployment_major(provenance, 0);
    });
    versioned(ProvenanceField::SdkName, |provenance| {
        sdk_of(provenance).name = String::new();
    });
    versioned(ProvenanceField::SdkVersion, |provenance| {
        sdk_of(provenance).version = String::new();
    });
    versioned(ProvenanceField::SdkBuild, |provenance| {
        sdk_of(provenance).build = String::new();
    });

    // And the whole point of the generalization: a backend that declares no SDK
    // owes none of the four above, and its payload is nameable without one.
    unversioned_payload_metadata(b"scalar-image")
        .identity()
        .expect("a payload that declares no SDK owes no SDK field");
}

/// A payload that declares no SDK may not state a platform field anyway.
///
/// Forged by appending the unversioned tag to a *versioned* encoding, which is
/// the exact byte string a producer would have to write to give one record two
/// spellings. Refusing it is what keeps payload identity injective across the
/// widening, and the field is named so the forgery is diagnosable.
#[test]
fn a_tagged_encoding_that_states_a_platform_field_is_refused_by_name() {
    let mut bytes =
        super::super::payload::encode_metadata(&payload_metadata(b"kernel void fused() {}"));
    bytes.push(PAYLOAD_PLATFORM_UNVERSIONED_TAG);
    assert_eq!(
        decode_metadata(&bytes),
        Err(ArtifactCodecError::PlatformFieldWithoutPlatform {
            field: ProvenanceField::DeploymentMinimum,
        }),
    );

    // The same forgery with only the SDK filled reaches the next field rather
    // than passing, so the check is a conjunction and not one guarded position.
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    set_deployment_major(&mut metadata.provenance, 0);
    set_deployment_minor(&mut metadata.provenance, 0);
    let mut bytes = super::super::payload::encode_metadata(&metadata);
    bytes.push(PAYLOAD_PLATFORM_UNVERSIONED_TAG);
    assert_eq!(
        decode_metadata(&bytes),
        Err(ArtifactCodecError::PlatformFieldWithoutPlatform {
            field: ProvenanceField::SdkName,
        }),
    );
}

/// A platform tag this build does not implement is refused rather than skipped.
#[test]
fn an_unimplemented_platform_tag_is_refused() {
    let mut bytes =
        super::super::payload::encode_metadata(&payload_metadata(b"kernel void fused() {}"));
    bytes.push(0x02);
    assert_eq!(
        decode_metadata(&bytes),
        Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::PayloadPlatform,
            tag: 0x02,
        }),
    );
}

/// Stripping the appended tag leaves a versioned payload that owes what it lacks.
///
/// The truncation is what a reader predating the platform block would see, and
/// the case pins its behaviour: it reads the record as versioned — which is what
/// keeps every older payload's bytes meaning what they meant — and then refuses
/// it, because the platform positions an unversioned payload pins to blank are
/// exactly the fields a versioned payload owes. A truncation is a refusal here
/// rather than a silent reinterpretation.
#[test]
fn an_unversioned_payload_with_its_tag_stripped_is_refused() {
    let bytes =
        super::super::payload::encode_metadata(&unversioned_payload_metadata(b"scalar-image"));
    let truncated = &bytes[..bytes.len() - 1];
    assert_eq!(
        decode_metadata(truncated),
        Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::IncompletePayloadProvenance {
                field: ProvenanceField::DeploymentMinimum,
            }),
        }),
    );
}

/// Reaches the deployment minimum of a fixture known to declare a versioned SDK.
fn set_deployment_major(provenance: &mut PayloadProvenance, value: u16) {
    let PayloadPlatform::VersionedSdk {
        deployment_major, ..
    } = &mut provenance.platform
    else {
        panic!("the fixture declares a versioned SDK");
    };
    *deployment_major = value;
}

/// Reaches the deployment minor of a fixture known to declare a versioned SDK.
fn set_deployment_minor(provenance: &mut PayloadProvenance, value: u16) {
    let PayloadPlatform::VersionedSdk {
        deployment_minor, ..
    } = &mut provenance.platform
    else {
        panic!("the fixture declares a versioned SDK");
    };
    *deployment_minor = value;
}

/// Reaches the SDK identity of a fixture known to declare a versioned SDK.
fn sdk_of(provenance: &mut PayloadProvenance) -> &mut PayloadSdkIdentity {
    let PayloadPlatform::VersionedSdk { sdk, .. } = &mut provenance.platform else {
        panic!("the fixture declares a versioned SDK");
    };
    sdk
}
