//! The public read side of the codec: encode an artifact, decode bytes back.
//!
//! # Why this exists rather than a promoted encoder
//!
//! An out-of-crate assembler must be able to prove that what it packaged
//! survives a round trip; before this module nothing outside `tiler-artifact`
//! could encode an artifact, decode one, or observe an envelope digest, so a
//! real compilation could never meet the codec's own checks.
//!
//! The obvious promotion — making [`super::encode`], [`super::decode`], and
//! `ArtifactEnvelope` public — was rejected on review. The envelope is an
//! internal projection the codec is still changing, and publishing it would
//! commit the boundary to its field layout. This module exposes the
//! *capability* instead: bytes out, a validated view back, and accessors rather
//! than fields.
//!
//! # What a decoded artifact deliberately is not
//!
//! It is **not** a [`VerifiedArtifactProgram`]. A variant's program reaches the
//! envelope as its canonical identity bytes alone (`super::model`'s projection
//! stores `canonical_identity()`), so nothing decoded can rebuild a
//! `VerifiedKernelProgram`, and a decoder that returned one would be claiming a
//! reconstruction the format does not carry.
//!
//! What a reader gets is everything the envelope actually holds: re-derived
//! identity, the governed feature set, payload descriptors, framed section
//! bytes, and each variant's entries with their ABI and launch expressions.
//! Whether that is enough for a runtime to dispatch — or whether the envelope
//! must carry a reconstructable program — is
//! `carry-reconstructable-kernel-programs-in-the-neutral-envelope`'s question,
//! and this module deliberately does not answer it by inventing a shape.

use super::super::model::{
    BackendPayloadDescriptor, CanonicalArtifactProgramIdentity, RoutingPolicy,
    VerifiedArtifactProgram,
};
use super::error::ArtifactCodecError;
use super::model::{ArtifactEnvelope, SectionKind};
use std::error::Error;
use std::fmt;

use super::decode::decode;
use super::encode::encode;

impl VerifiedArtifactProgram {
    /// Encodes this artifact into its canonical envelope bytes.
    ///
    /// The bytes are a function of the artifact's identity rather than of the
    /// order a producer declared things in, so two producers that built the
    /// same artifact emit the same bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCodecFailure`] when the canonical encoding exceeds a
    /// governed envelope bound.
    ///
    /// # Panics
    ///
    /// Panics if the artifact's data no longer projects into an envelope.
    /// [`ArtifactProgramBuilder::build`] already performed that projection to
    /// derive this artifact's identity, and the data is immutable afterward, so
    /// a failure here is a defect in this crate rather than a caller error —
    /// which is why it is not a returned variant a caller would have to handle.
    ///
    /// [`ArtifactProgramBuilder::build`]: super::super::ArtifactProgramBuilder::build
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactCodecFailure> {
        let envelope = ArtifactEnvelope::project(&self.data)
            .expect("a verified artifact projected into an envelope when its identity was derived");
        encode(&envelope).map_err(ArtifactCodecFailure::from)
    }
}

/// Decodes and fully validates one encoded artifact envelope.
///
/// Validation is not optional and not separable: framing, manifest and section
/// digests, schema, canonical order, arena closure, and identity re-derivation
/// all run before this returns. A rejection never yields a partially validated
/// view, so holding a [`DecodedArtifact`] is itself the evidence that the bytes
/// passed every check.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecFailure`] naming the first boundary that
/// rejected.
pub fn decode_artifact(bytes: &[u8]) -> Result<DecodedArtifact, ArtifactCodecFailure> {
    decode(bytes)
        .map(|envelope| DecodedArtifact { envelope })
        .map_err(ArtifactCodecFailure::from)
}

/// A validated read view over one decoded artifact envelope.
///
/// Accessors rather than fields, so this commits the public boundary to what an
/// artifact carries and not to how the codec lays it out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedArtifact {
    envelope: ArtifactEnvelope,
}

impl DecodedArtifact {
    /// Returns the identity re-derived from the decoded content.
    ///
    /// Re-derived, never read from the bytes: [`decode_artifact`] already
    /// proved this equals the identity the encoded manifest carried, so a
    /// forged manifest cannot present a chosen identity.
    ///
    /// # Panics
    ///
    /// Panics if the identity no longer derives. [`decode_artifact`] already
    /// derived it to compare against the encoded manifest, and this view is
    /// immutable, so a failure here is a defect in this crate.
    #[must_use]
    pub fn identity(&self) -> CanonicalArtifactProgramIdentity {
        self.envelope
            .canonical_identity()
            .expect("a decoded envelope derived its identity during validation")
    }

    /// Returns the governed features a reader must implement to use this
    /// artifact.
    #[must_use]
    pub fn features(&self) -> &[String] {
        self.envelope.features()
    }

    /// Returns the routing policy the artifact declares.
    #[must_use]
    pub const fn routing(&self) -> RoutingPolicy {
        self.envelope.routing
    }

    /// Returns the carried backend payload descriptors in canonical order.
    #[must_use]
    pub fn payloads(&self) -> &[BackendPayloadDescriptor] {
        &self.envelope.payloads
    }

    /// Returns one view per framed section, in canonical order.
    #[must_use]
    pub fn sections(&self) -> impl ExactSizeIterator<Item = SectionView<'_>> {
        self.envelope.sections.iter().map(|section| SectionView {
            kind: section.kind,
            bytes: &section.bytes,
        })
    }

    /// Returns the number of packaged plan variants, in routing priority order.
    #[must_use]
    pub fn variant_count(&self) -> usize {
        self.envelope.variants.len()
    }

    /// Re-encodes this decoded artifact.
    ///
    /// A decode followed by this must reproduce the original bytes exactly.
    /// That is the round-trip property worth asserting: it proves the decoder
    /// read every field the encoder wrote, because a field silently dropped on
    /// the way in cannot be written back out.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCodecFailure`] when the canonical encoding exceeds a
    /// governed envelope bound.
    pub fn re_encode(&self) -> Result<Vec<u8>, ArtifactCodecFailure> {
        encode(&self.envelope).map_err(ArtifactCodecFailure::from)
    }
}

/// One framed section of a decoded artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionView<'a> {
    kind: SectionKind,
    bytes: &'a [u8],
}

impl<'a> SectionView<'a> {
    /// Returns what this section carries.
    #[must_use]
    pub const fn purpose(self) -> SectionPurpose {
        match self.kind {
            SectionKind::KernelProgramSubject => SectionPurpose::KernelProgramSubject,
            SectionKind::BackendPayloadMetadata => SectionPurpose::BackendPayloadMetadata,
            SectionKind::BackendPayloadCode => SectionPurpose::BackendPayloadCode,
        }
    }

    /// Returns the section's exact framed bytes.
    #[must_use]
    pub const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

/// What one framed section of an artifact carries.
///
/// A public mirror of the codec's internal section vocabulary, written by an
/// exhaustive match rather than shared by re-export, so the wire vocabulary can
/// gain a purpose without that being a public change by default (ADR 0074
/// convention 3).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SectionPurpose {
    /// The canonical identity of one packaged variant's kernel program.
    ///
    /// The identity alone. The program is not carried.
    KernelProgramSubject,
    /// The canonical compilation subject of one carried backend payload.
    ///
    /// These exact bytes are the payload's identity subject, so a descriptor's
    /// digest is a function of this section.
    BackendPayloadMetadata,
    /// The emitted object bytes of one carried backend payload.
    ///
    /// Carried opaquely under an integrity digest that artifact identity
    /// deliberately excludes, so relinking the same source yields the same
    /// artifact identity and different envelope bytes.
    BackendPayloadCode,
}

/// Why an artifact's bytes were rejected, or could not be produced.
///
/// # Why this is coarser than the codec's own vocabulary
///
/// `super::error`'s rejection enum names the exact boundary that refused, and
/// its variants carry internal subject enums — which section role, which
/// ordered table, which reference class. Publishing it would publish those too,
/// and they are the codec's working vocabulary rather than a contract. This
/// classifies instead: enough for a caller to decide what to *do*, with the
/// exact boundary preserved in [`fmt::Display`] for a person reading a log.
///
/// The classes answer different questions. Bytes that are not a Tiler artifact
/// at all, bytes that are one but were damaged, an artifact from a newer writer
/// this build cannot read, one that is well-formed but breaks an invariant, and
/// one that exceeds a governed bound are five different things to do next, and
/// collapsing them would make a version skew look like corruption.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a new class lands
/// additively.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ArtifactCodecFailure {
    /// The bytes are not a well-formed artifact envelope.
    ///
    /// Framing, magic, domain separation, or a length that does not agree with
    /// what it frames. Usually the wrong bytes entirely rather than bad ones.
    Malformed {
        /// The exact boundary that refused, for diagnosis.
        detail: String,
    },
    /// A digest did not match the content it covers.
    ///
    /// Distinct from [`Self::Malformed`] because the framing was readable: this
    /// is damage or tampering, not a category error.
    IntegrityFailure {
        /// The exact boundary that refused, for diagnosis.
        detail: String,
    },
    /// The artifact declares a schema, encoding, or feature this build does not
    /// implement.
    ///
    /// The artifact is not wrong; this reader is older than its writer. Failing
    /// closed here rather than ignoring the unknown part is what keeps a
    /// forward-incompatible artifact from being partially honoured.
    Unsupported {
        /// The exact schema, encoding, or feature that is not implemented.
        detail: String,
    },
    /// The artifact is well-formed but violates an invariant it must satisfy.
    ///
    /// Canonical order, arena closure, a dangling reference, a type or phase
    /// disagreement, or a re-derived identity that does not match the manifest.
    Invalid {
        /// The exact invariant that was violated.
        detail: String,
    },
    /// The artifact exceeds a governed structural bound.
    Limit {
        /// The exact bound that was exceeded.
        detail: String,
    },
}

impl fmt::Display for ArtifactCodecFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (class, detail) = match self {
            Self::Malformed { detail } => ("malformed", detail),
            Self::IntegrityFailure { detail } => ("integrity", detail),
            Self::Unsupported { detail } => ("unsupported", detail),
            Self::Invalid { detail } => ("invalid", detail),
            Self::Limit { detail } => ("limit", detail),
        };
        write!(formatter, "artifact.{class}: {detail}")
    }
}

impl Error for ArtifactCodecFailure {}

impl From<ArtifactCodecError> for ArtifactCodecFailure {
    /// Classifies one internal rejection.
    ///
    /// The match is exhaustive over every variant rather than falling through a
    /// wildcard, so a new codec boundary is a build error here and has to be
    /// classified deliberately instead of silently becoming whichever class the
    /// wildcard named.
    fn from(error: ArtifactCodecError) -> Self {
        let detail = error.to_string();
        match error {
            ArtifactCodecError::Truncated { .. }
            | ArtifactCodecError::TrailingBytes { .. }
            | ArtifactCodecError::TrailingManifestBytes { .. }
            | ArtifactCodecError::BadMagic
            | ArtifactCodecError::BadManifestDomain
            | ArtifactCodecError::BadPayloadMetadataDomain
            | ArtifactCodecError::TotalLengthMismatch { .. }
            | ArtifactCodecError::SectionLengthMismatch { .. }
            | ArtifactCodecError::SectionCountMismatch { .. }
            | ArtifactCodecError::InvalidText
            | ArtifactCodecError::InvalidGovernedKey { .. }
            | ArtifactCodecError::InvalidInterfaceKey { .. }
            | ArtifactCodecError::InvalidProviderIdentity { .. }
            | ArtifactCodecError::InvalidShape { .. }
            | ArtifactCodecError::UnknownTag { .. } => Self::Malformed { detail },

            ArtifactCodecError::ManifestDigestMismatch
            | ArtifactCodecError::SectionDigestMismatch { .. }
            | ArtifactCodecError::PayloadIdentityMismatch { .. }
            | ArtifactCodecError::ArtifactIdentityMismatch => Self::IntegrityFailure { detail },

            ArtifactCodecError::UnsupportedEnvelopeFormat { .. }
            | ArtifactCodecError::UnsupportedCanonicalEncoding { .. }
            | ArtifactCodecError::UnsupportedManifestSchema { .. }
            | ArtifactCodecError::UnsupportedComponentSchema { .. }
            | ArtifactCodecError::UnsupportedDigestAlgorithm { .. }
            | ArtifactCodecError::UnsupportedRequiredFeature { .. }
            | ArtifactCodecError::UnsupportedSectionSchema { .. }
            | ArtifactCodecError::UnsupportedPayloadMetadataSchema { .. } => {
                Self::Unsupported { detail }
            }

            ArtifactCodecError::Limit { .. } => Self::Limit { detail },

            ArtifactCodecError::SectionDispositionMismatch { .. }
            | ArtifactCodecError::SectionPurposeMismatch { .. }
            | ArtifactCodecError::NonCanonicalSectionId { .. }
            | ArtifactCodecError::NonCanonicalOrder { .. }
            | ArtifactCodecError::DuplicateItem { .. }
            | ArtifactCodecError::NonCanonicalManifest
            | ArtifactCodecError::UnreferencedSection { .. }
            | ArtifactCodecError::DeclaredFeatureMismatch
            | ArtifactCodecError::MissingReference { .. }
            | ArtifactCodecError::ExpressionOperandOrder { .. }
            | ArtifactCodecError::ExpressionOperandType { .. }
            | ArtifactCodecError::ExpressionSelectBranchType { .. }
            | ArtifactCodecError::ModelRule { .. }
            | ArtifactCodecError::ModelObligation { .. }
            | ArtifactCodecError::IdentityDerivation { .. } => Self::Invalid { detail },
        }
    }
}
