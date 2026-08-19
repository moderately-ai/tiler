//! Structured selected-capability rows through identity and the codec.

use super::super::super::error::ArtifactBuildError;
use super::super::super::keys::CapabilityFamilyKey;
use super::super::super::model::SelectedProvider;
use super::super::super::tests::default_artifact;
use super::super::decode::{Cursor, decode, read_providers};
use super::super::encode::encode;
use super::super::error::ArtifactCodecError;
use super::super::view::{ArtifactCodecFailure, decode_artifact};
use super::support::{envelope_of, manifest_offset, reseal, subject};
use std::error::Error as _;
use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::semantic::{IdentityComponent, OpKey};

/// Encodes exactly the selected-provider table that `read_providers` consumes.
fn encoded_provider_rows(rows: &[(&str, &str, &str, u32)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_len(&mut bytes, rows.len());
    for (family, namespace, name, semantic_version) in rows {
        push_slice(&mut bytes, b"tiler-test");
        push_slice(&mut bytes, b"fused-serial-sum");
        bytes.extend_from_slice(&1_u32.to_be_bytes());
        push_slice(&mut bytes, family.as_bytes());
        push_slice(&mut bytes, namespace.as_bytes());
        push_slice(&mut bytes, name.as_bytes());
        bytes.extend_from_slice(&semantic_version.to_be_bytes());
        bytes.extend_from_slice(&1_u32.to_be_bytes());
    }
    bytes
}

#[test]
fn dotted_operation_boundaries_remain_distinct_through_artifact_and_codec_identity() {
    let mut envelope = envelope_of(&default_artifact());
    let provider = envelope.providers[0].provider.clone();
    let selected = |namespace, name| SelectedProvider {
        provider: provider.clone(),
        capability: subject("index-access", namespace, name, 1),
        capability_revision: 1,
    };
    envelope.providers = vec![selected("a.b", "c"), selected("a", "b.c")];
    envelope
        .providers
        .sort_unstable_by_key(SelectedProvider::canonical_key);

    assert_eq!(envelope.providers().len(), 2, "both selected rows survive");
    assert_ne!(
        envelope.providers()[0].canonical_key(),
        envelope.providers()[1].canonical_key(),
        "separate frames retain the namespace/name boundary",
    );

    let identity = envelope
        .canonical_identity()
        .expect("the collision pair has one injective artifact identity");
    let bytes = encode(&envelope).expect("the collision pair packages");
    let decoded = decode(&bytes).expect("the collision pair decodes");
    assert_eq!(decoded.providers().len(), 2, "both rows decode");
    assert!(decoded.providers().iter().any(|selected| {
        selected.capability.operation.namespace() == "a.b"
            && selected.capability.operation.name() == "c"
    }));
    assert!(decoded.providers().iter().any(|selected| {
        selected.capability.operation.namespace() == "a"
            && selected.capability.operation.name() == "b.c"
    }));
    assert_eq!(
        decoded
            .canonical_identity()
            .expect("decoded identity re-derives"),
        identity,
        "the codec retains the structured identity",
    );
}

#[test]
fn every_selected_provider_component_perturbs_artifact_identity_independently() {
    let mut baseline = envelope_of(&default_artifact());
    baseline.providers[0].capability = subject("index-access", "a.b", "c", 1);
    let baseline_identity = baseline.canonical_identity().unwrap();

    let mut family = baseline.clone();
    family.providers[0].capability.family = CapabilityFamilyKey::new("index-access-alt").unwrap();

    let mut boundary = baseline.clone();
    boundary.providers[0].capability.operation = OpKey::new("a", "b.c", 1).unwrap();

    let mut version = baseline.clone();
    version.providers[0].capability.operation = OpKey::new("a.b", "c", 2).unwrap();

    let mut provider = baseline.clone();
    provider.providers[0].provider =
        tiler_ir::semantic::ProviderIdentity::new("tiler-test", "other-provider", 1).unwrap();

    let mut revision = baseline.clone();
    revision.providers[0].capability_revision = 2;

    for (name, perturbed) in [
        ("capability family", family),
        ("operation namespace/name boundary", boundary),
        ("operation semantic version", version),
        ("provider", provider),
        ("capability revision", revision),
    ] {
        assert_ne!(
            perturbed.canonical_identity().unwrap(),
            baseline_identity,
            "{name} must independently enter artifact identity",
        );
    }
}

#[test]
fn uppercase_and_maximum_length_operation_components_package_without_conversion() {
    let namespace = "A".repeat(255);
    let name = "Z".repeat(255);
    let mut envelope = envelope_of(&default_artifact());
    envelope.providers[0].capability = subject("index-access", &namespace, &name, u32::MAX);

    let bytes = encode(&envelope).expect("maximum legal components package");
    let decoded = decode(&bytes).expect("maximum legal components decode");
    let operation = &decoded.providers()[0].capability.operation;
    assert_eq!(operation.namespace(), namespace);
    assert_eq!(operation.name(), name);
    assert_eq!(operation.semantic_version(), u32::MAX);
}

#[test]
fn corrupt_structured_components_preserve_typed_causes_but_classify_publicly_as_malformed() {
    let cases = [
        ("family", "Index-access", "tiler", "op", 1),
        ("namespace", "index-access", "bad/name", "op", 1),
        ("name", "index-access", "tiler", "bad/name", 1),
        ("version", "index-access", "tiler", "op", 0),
    ];

    for (component, family, namespace, name, version) in cases {
        let bytes = encoded_provider_rows(&[(family, namespace, name, version)]);
        let error = read_providers(&mut Cursor::new(&bytes))
            .expect_err("the independently corrupt component is refused");
        assert!(
            error.source().is_some(),
            "{component} retains a typed source"
        );
        match (&error, component) {
            (
                ArtifactCodecError::InvalidGovernedKey {
                    cause:
                        ArtifactBuildError::NoncanonicalKeyByte {
                            kind: super::super::super::error::ArtifactKeyKind::CapabilityFamily,
                            index: 0,
                            value: b'I',
                        },
                },
                "family",
            )
            | (
                ArtifactCodecError::InvalidOperationKey {
                    cause:
                        tiler_ir::semantic::TypeIdentityError::InvalidComponentCharacter {
                            component: IdentityComponent::Namespace,
                            byte_index: 3,
                        },
                },
                "namespace",
            )
            | (
                ArtifactCodecError::InvalidOperationKey {
                    cause:
                        tiler_ir::semantic::TypeIdentityError::InvalidComponentCharacter {
                            component: IdentityComponent::Name,
                            byte_index: 3,
                        },
                },
                "name",
            )
            | (
                ArtifactCodecError::InvalidOperationKey {
                    cause: tiler_ir::semantic::TypeIdentityError::ZeroSemanticVersion,
                },
                "version",
            ) => {}
            _ => panic!("{component} produced the wrong typed cause: {error:?}"),
        }

        let public = ArtifactCodecFailure::from(error);
        assert!(
            matches!(public, ArtifactCodecFailure::Malformed { .. }),
            "{component} must classify as malformed: {public:?}",
        );
        assert!(
            public.source().is_none(),
            "the public classifier truthfully carries no typed source",
        );
    }
}

#[test]
fn public_decode_classifies_each_corrupt_structured_component_as_malformed_without_a_source() {
    const NAMESPACE: &str = "UniqueNamespace";
    const NAME: &str = "UniqueOperation";
    let mut envelope = envelope_of(&default_artifact());
    envelope.providers[0].capability = subject("index-access", NAMESPACE, NAME, 7);
    let baseline = encode(&envelope).expect("the structured baseline encodes");

    for component in ["family", "namespace", "name", "version"] {
        let mut forged = baseline.clone();
        match component {
            "family" => {
                let offset = manifest_offset(&forged, b"index-access");
                forged[offset] = b'I';
            }
            "namespace" => {
                let offset = manifest_offset(&forged, NAMESPACE.as_bytes());
                forged[offset + 6] = b'/';
            }
            "name" => {
                let offset = manifest_offset(&forged, NAME.as_bytes());
                forged[offset + 6] = b'/';
            }
            "version" => {
                let offset = manifest_offset(&forged, NAME.as_bytes()) + NAME.len();
                forged[offset..offset + 4].copy_from_slice(&0_u32.to_be_bytes());
            }
            _ => unreachable!(),
        }
        reseal(&mut forged);

        let failure = decode_artifact(&forged)
            .expect_err("a forged structured component is rejected at the public boundary");
        assert!(
            matches!(failure, ArtifactCodecFailure::Malformed { .. }),
            "{component} classified incorrectly: {failure:?}",
        );
        assert!(
            failure.source().is_none(),
            "{component}'s private typed cause must not be claimed publicly",
        );
    }
}

#[test]
fn a_legacy_flat_capability_row_is_not_reinterpreted() {
    let mut bytes = Vec::new();
    push_len(&mut bytes, 1);
    push_slice(&mut bytes, b"tiler-test");
    push_slice(&mut bytes, b"fused-serial-sum");
    bytes.extend_from_slice(&1_u32.to_be_bytes());
    push_slice(
        &mut bytes,
        b"tiler.capability.index-access.tiler.strict-serial-sum-f32.v1",
    );
    bytes.extend_from_slice(&1_u32.to_be_bytes());

    assert!(
        matches!(
            read_providers(&mut Cursor::new(&bytes)),
            Err(ArtifactCodecError::Truncated { .. })
        ),
        "schema 19 requires the structured fields; no legacy parser exists",
    );
}
