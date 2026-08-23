//! The artifact refusal wall in front of the bit-preserving-copy resource arm.
//!
//! The wire and identity grammars carry the ten floating-point rows every
//! existing artifact wrote; the copy's tagged entry row and its schema step
//! are `admit-an-explicit-non-arithmetic-region-and-delivery-state`'s accepted
//! boundary. `push_resources` refuses the copy arm by name, and both of its
//! callers are watched here: the identity encoder, which surfaces the model's
//! own diagnostic, and the wire encoder, which wraps it as
//! [`ArtifactCodecError::ModelObligation`].
//!
//! The subject is projected from a verified artifact and then perturbed at the
//! entry's numerical requirement, for the reason the subgroup carrier fixture
//! states: supported derivation still yields the arithmetic arm, and making a
//! copy region produce an entry is the owning ticket's boundary, not this
//! module's claim.

use super::super::super::error::ArtifactDiagnostic;
use super::super::super::model::push_resources;
use super::super::super::tests::default_artifact;
use super::super::encode::{encode, encode_with_identity, section_digest};
use super::super::error::ArtifactCodecError;
use super::super::model::ArtifactEnvelope;
use super::support::envelope_of;
use tiler_digest::DigestAlgorithm;
use tiler_ir::schedule::RegionNumericalRequirements;

/// The one diagnostic every refusal below must name.
const RULE: &str = "bit-preserving-copy-resources";

/// A verified artifact's envelope with its single entry's numerical
/// requirement moved to the proved-absence arm.
fn envelope_with_copy_resources() -> ArtifactEnvelope {
    let mut envelope = envelope_of(&default_artifact());
    envelope.variants[0].entries[0].resources.numerical =
        RegionNumericalRequirements::BitPreservingCopy;
    envelope
}

/// One record, one field moved: the arithmetic arm encodes its rows and the
/// copy arm refuses, so the refusal is the numerical arm and not a defect the
/// rest of the record carries.
#[test]
fn the_resource_encoder_refuses_the_copy_numerical_arm_by_name() {
    let mut resources = envelope_of(&default_artifact()).variants[0].entries[0].resources;
    let mut arithmetic = Vec::new();
    push_resources(&mut arithmetic, resources).expect("the arithmetic rows encode");
    assert!(!arithmetic.is_empty(), "the arithmetic arm writes its rows");

    resources.numerical = RegionNumericalRequirements::BitPreservingCopy;
    let mut copy = Vec::new();
    let refusal = push_resources(&mut copy, resources).expect_err("the copy arm has no grammar");
    assert_eq!(refusal, ArtifactDiagnostic::BitPreservingCopyResources);
    assert_eq!(refusal.rule(), RULE);
    // Refused before a byte is written, so no half-encoded row can reach a
    // caller that ignores the error.
    assert!(copy.is_empty(), "{copy:?}");
}

/// The identity path surfaces the model's own diagnostic, and the public
/// encoder — which derives the identity before it frames anything — refuses at
/// that derivation rather than reaching the wire encoder at all.
#[test]
fn the_identity_path_refuses_an_entry_carrying_the_copy_numerical_arm() {
    let envelope = envelope_with_copy_resources();
    // The success value is discarded before `expect_err` formats it: a whole
    // artifact identity or envelope in a panic message buries the failure text
    // a perturbation run is read for.
    let refusal = envelope
        .canonical_identity()
        .map(|_| ())
        .expect_err("the copy arm has no identity grammar");
    assert_eq!(refusal, ArtifactDiagnostic::BitPreservingCopyResources);
    assert_eq!(refusal.rule(), RULE);

    let refusal = encode(&envelope)
        .map(|_| ())
        .expect_err("the public encoder derives an identity first");
    assert_eq!(
        refusal,
        ArtifactCodecError::IdentityDerivation {
            cause: ArtifactDiagnostic::BitPreservingCopyResources,
        }
    );
}

/// The wire path reports the model's own cause rather than a second, drifting
/// codec vocabulary.
///
/// Reached through the identity-taking encoder, because the identity-deriving
/// one refuses above before framing begins. The identity is derived from the
/// unperturbed envelope for exactly that reason: the perturbed one has none.
#[test]
fn the_wire_path_wraps_the_copy_refusal_as_a_model_obligation() {
    let arithmetic = envelope_of(&default_artifact());
    let identity = arithmetic
        .canonical_identity()
        .expect("the arithmetic envelope derives an identity");
    let envelope = envelope_with_copy_resources();
    let section_digests: Vec<_> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();
    let refusal = encode_with_identity(&envelope, &identity, &section_digests)
        .map(|_| ())
        .expect_err("the copy arm has no wire grammar");
    assert_eq!(
        refusal,
        ArtifactCodecError::ModelObligation {
            cause: ArtifactDiagnostic::BitPreservingCopyResources,
        }
    );
}
