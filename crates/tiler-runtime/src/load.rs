//! Decoding artifact bytes into a validated, device-free program record.
//!
//! # The stage this module owns
//!
//! [`DecodedProgram::decode`] is the whole of it: bytes in, a fully validated
//! read view out, or a typed rejection naming the class of failure. Nothing
//! here inspects a host, allocates anything, or commits to executing.
//!
//! The validation is [`tiler_artifact`]'s, not this crate's.
//! [`decode_artifact`] proves framing, manifest and section digests, component
//! schemas, canonical order, expression-arena closure, required-feature
//! support, and — last — that the identity re-derived from the decoded content
//! equals the one the manifest carries. A rejection never yields a partially
//! validated view, so holding a [`DecodedProgram`] *is* the evidence that the
//! bytes passed every one of those checks.
//!
//! # Why the rejection is reclassified rather than passed through
//!
//! [`ArtifactCodecFailure`] already classifies the codec's own boundaries, and
//! this module keeps every one of those distinctions by carrying the value
//! whole in [`LoadRejection::Artifact`]. It does not flatten them into strings
//! and it does not add a class the codec already draws. The reclassification
//! exists so that a *host* failure — an incompatible profile, an artifact that
//! is not the one this process compiled, an object this build cannot resolve —
//! is a different variant from a damaged file, because the two mean different
//! things to do next and collapsing them would make a version skew look like
//! corruption.

use tiler_artifact::program::{
    ArtifactCodecFailure, BackendPayloadDescriptor, CanonicalArtifactProgramIdentity,
    DecodedArtifact, RoutingPolicy, SectionView, decode_artifact,
};

use std::error::Error;
use std::fmt;

/// One artifact's bytes, decoded and fully validated by the artifact layer.
///
/// Accessors rather than fields, and deliberately no `From`/`Deref` onto
/// [`DecodedArtifact`]: this crate's job is to add host-relative obligations on
/// top of a decode, and handing out the raw view would let a caller skip them
/// while still appearing to have gone through the runtime.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedProgram {
    decoded: DecodedArtifact,
}

impl DecodedProgram {
    /// Decodes and validates one encoded artifact envelope.
    ///
    /// # Errors
    ///
    /// Returns [`LoadRejection::Artifact`] carrying the codec's own
    /// classification of the first boundary that refused.
    pub fn decode(bytes: &[u8]) -> Result<Self, LoadRejection> {
        decode_artifact(bytes)
            .map(|decoded| Self { decoded })
            .map_err(LoadRejection::Artifact)
    }

    /// Returns the identity re-derived from this artifact's decoded content.
    ///
    /// Never read from the manifest: [`decode_artifact`] derived it from
    /// content and refused when it disagreed with the manifest's copy, so a
    /// forged envelope cannot present a chosen identity here.
    #[must_use]
    pub fn identity(&self) -> CanonicalArtifactProgramIdentity {
        self.decoded.identity()
    }

    /// Returns the governed features this artifact requires of a reader.
    ///
    /// Informational at this point rather than a gate: the codec already
    /// refused any feature this build cannot supply, so a
    /// [`DecodedProgram`] never carries an unsupported one. It is exposed so a
    /// host can log or report what an artifact needed.
    #[must_use]
    pub fn required_features(&self) -> &[String] {
        self.decoded.features()
    }

    /// Returns the policy by which this artifact's variants are chosen among.
    #[must_use]
    pub fn routing_policy(&self) -> RoutingPolicy {
        self.decoded.routing()
    }

    /// Returns the number of packaged plan variants, in routing priority order.
    #[must_use]
    pub fn variant_count(&self) -> usize {
        self.decoded.variant_count()
    }

    /// Returns the carried backend payload descriptors in canonical order.
    #[must_use]
    pub fn payloads(&self) -> &[BackendPayloadDescriptor] {
        self.decoded.payloads()
    }

    /// Returns every framed section this artifact carries.
    #[must_use]
    pub fn sections(&self) -> impl ExactSizeIterator<Item = SectionView<'_>> {
        self.decoded.sections()
    }

}

/// Why one artifact was not accepted for execution on this host.
///
/// The classes answer different questions, which is the whole reason there is
/// more than one. Bytes the artifact layer refused, an artifact that is not the
/// one this process expected, a host that cannot honour the declared target
/// profile, and a carried object this build cannot resolve are four different
/// things to do next; reporting them as one would make a stale cache entry
/// indistinguishable from a corrupt file.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later obligation lands
/// as a new class rather than by widening an existing one's meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LoadRejection {
    /// The artifact layer refused the bytes, with its own classification.
    ///
    /// Carried whole rather than restated. The codec draws five distinctions —
    /// malformed, integrity, unsupported, invalid, limit — and this crate is
    /// not a better authority on which of them applies.
    Artifact(ArtifactCodecFailure),
}

impl fmt::Display for LoadRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(failure) => write!(formatter, "runtime.artifact: {failure}"),
        }
    }
}

impl Error for LoadRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(failure) => Some(failure),
        }
    }
}

impl From<ArtifactCodecFailure> for LoadRejection {
    fn from(value: ArtifactCodecFailure) -> Self {
        Self::Artifact(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodedProgram, LoadRejection};
    use tiler_artifact::program::ArtifactCodecFailure;

    /// Bytes that are not an artifact at all are refused as malformed.
    ///
    /// The class matters more than the refusal: a host that cannot tell "this
    /// is not a Tiler artifact" from "this artifact is damaged" cannot decide
    /// whether to look for a different file or to re-fetch this one.
    #[test]
    fn foreign_bytes_are_malformed_rather_than_damaged() {
        let rejection = DecodedProgram::decode(b"not a Tiler artifact at all")
            .expect_err("foreign bytes are not an artifact");
        assert!(
            matches!(
                rejection,
                LoadRejection::Artifact(ArtifactCodecFailure::Malformed { .. }),
            ),
            "expected a malformed classification, got {rejection}",
        );
    }

    /// An empty input is refused rather than treated as an empty artifact.
    #[test]
    fn empty_bytes_are_refused() {
        assert!(DecodedProgram::decode(&[]).is_err());
    }

    /// The rejection keeps the codec's own failure reachable as its source.
    ///
    /// Asserted because the alternative — formatting the cause into a string —
    /// is the easy way to write this type and destroys a caller's ability to
    /// match on what actually happened.
    #[test]
    fn a_rejection_preserves_the_codec_failure_it_classifies() {
        let rejection =
            DecodedProgram::decode(b"short").expect_err("five bytes are not an artifact");
        match &rejection {
            LoadRejection::Artifact(failure) => assert!(
                rejection.to_string().contains(&failure.to_string()),
                "the display form must not lose the boundary that refused",
            ),
        }
    }
}
