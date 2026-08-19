//! Recorded artifact-identity assertions, and every way one is refused.

use super::super::model::ARTIFACT_DOMAIN_LABEL;
use super::super::{
    MAX_ARTIFACT_IDENTITY_BYTES, RecordedArtifactIdentityError, RecordedArtifactProgramIdentity,
};
use super::default_artifact;

// -------------------------------------------------------------------------
// Recorded identity assertions
// -------------------------------------------------------------------------

/// The cold-consumer round trip: a producer records the identity it derived, a
/// consumer reads those bytes back and states them. The stated assertion carries
/// the same bytes, and is a different type from the derivation it came from.
#[test]
fn recorded_bytes_from_a_derived_identity_state_that_identity() {
    let artifact = default_artifact();
    let derived = artifact.canonical_identity();

    let recorded = RecordedArtifactProgramIdentity::from_bytes(derived.as_bytes()).unwrap();

    assert_eq!(recorded.as_bytes(), derived.as_bytes());
}

#[test]
fn an_empty_recording_is_refused() {
    assert_eq!(
        RecordedArtifactProgramIdentity::from_bytes([]),
        Err(RecordedArtifactIdentityError::Empty),
    );
}

/// Checked before the domain frame is read, so an over-bound recording is
/// refused whatever it leads with.
#[test]
fn a_recording_beyond_the_identity_bound_is_refused() {
    let oversized = vec![0_u8; MAX_ARTIFACT_IDENTITY_BYTES + 1];

    assert_eq!(
        RecordedArtifactProgramIdentity::from_bytes(&oversized),
        Err(RecordedArtifactIdentityError::TooLong {
            bytes: MAX_ARTIFACT_IDENTITY_BYTES + 1,
            limit: MAX_ARTIFACT_IDENTITY_BYTES,
        }),
    );
}

/// The three shapes a wrong recording actually takes: bytes of some other
/// subject, an identity from a superseded artifact domain, and a recording
/// truncated inside the separator itself.
#[test]
fn a_recording_under_a_foreign_domain_is_refused() {
    let foreign: [&[u8]; 3] = [
        b"tiler.kernel.v3\0some other subject",
        b"tiler.artifact-program.v10\0",
        // The label is the separator without its terminator, so this is a
        // recording truncated one byte inside the frame being matched.
        ARTIFACT_DOMAIN_LABEL.as_bytes(),
    ];

    for bytes in foreign {
        assert_eq!(
            RecordedArtifactProgramIdentity::from_bytes(bytes),
            Err(RecordedArtifactIdentityError::ForeignDomain { bytes: bytes.len() }),
            "expected a foreign-domain refusal for {bytes:?}",
        );
    }
}

/// The domain a rejection names is the one the encoder writes, not a second
/// copy of the string that a version bump could leave behind.
#[test]
fn a_foreign_domain_rejection_names_the_current_domain() {
    let rejection = RecordedArtifactProgramIdentity::from_bytes(b"not an artifact identity")
        .expect_err("bytes under no artifact domain are refused");

    let rendered = rejection.to_string();
    assert!(
        rendered.contains(ARTIFACT_DOMAIN_LABEL),
        "{rendered} does not name the {ARTIFACT_DOMAIN_LABEL} domain",
    );
}
