//! The sidecar's canonical encoding, its bounded reader, and its association.
//!
//! The wire form deliberately mirrors the artifact envelope's discipline
//! without sharing its bytes: a fixed-width framing header that bounds
//! everything before an allocation is made, one canonical manifest the header
//! digests, and a stream of length-delimited payloads each of which the
//! manifest describes by exact length and content digest.
//!
//! Three properties are load-bearing.
//!
//! **Every variable-length run carries a fixed-width length before its
//! content**, so no concatenation of fields is ambiguous.
//!
//! **Payload position is structural, not referential.** Payloads are framed in
//! one canonical order — cases by stable key, then inputs in interface order,
//! then expectations in interface order — and a case's descriptors are aligned
//! with that order positionally. There is no payload index a manifest could
//! point at, so the class of forgery in which one payload's descriptor names
//! another payload's bytes does not exist here.
//!
//! **The reader re-proves, never repairs.** A decode ends by re-deriving the
//! canonical identity from the decoded content and re-encoding the whole
//! container; a manifest that is well formed but not canonical is refused
//! rather than normalized on the way in.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::semantic::{BuildError, InputKey, OutputKey};

use crate::program::{
    ArtifactCodecFailure, DIGEST_BYTES, Digest, DigestAlgorithm, VerifiedArtifactProgram,
    decode_artifact, envelope_digest,
};

use super::builder::{
    InterfaceProjectionError, ProofInterfaceError, project_interface, verify_case_payloads,
    verify_cases,
};
use super::model::{
    CanonicalProofSidecarIdentity, ProofCaseData, ProofCaseKey, ProofCaseKeyError, ProofCaseRef,
    ProofNumericalIdentity, ProofReferenceIdentity, ProofSemanticSubject, ProofSidecarData,
    ProofSubjectError, ProofSubjects, VerifiedProofSidecar, case_of, cases_of,
};
use super::{
    MAX_PROOF_CASES, MAX_PROOF_IDENTITY_BYTES, MAX_PROOF_INTERFACE_ENTRIES,
    MAX_PROOF_MANIFEST_BYTES, MAX_PROOF_PAYLOAD_BYTES, MAX_PROOF_SIDECAR_BYTES,
    MAX_PROOF_SUBJECT_BYTES,
};

/// Fixed framing magic of the proof sidecar.
///
/// Distinct from the envelope's `TILERART` in the first differing byte, so a
/// sidecar handed to an artifact reader and an envelope handed to this one are
/// each refused at the magic rather than misparsed.
pub(super) const MAGIC: [u8; 8] = *b"TILERPRF";
/// Exact byte length of the fixed framing header.
pub(super) const HEADER_BYTES: usize = 69;
/// Sidecar framing format version this build writes and reads.
pub(super) const SIDECAR_FORMAT: (u16, u16) = (1, 0);
/// Canonical byte-encoding profile version this build writes and reads.
pub(super) const CANONICAL_ENCODING: (u16, u16) = (1, 0);
/// Sidecar manifest schema version this build writes and reads.
pub(super) const MANIFEST_SCHEMA: (u16, u16) = (1, 0);

/// Versioned domain tag opening the canonical manifest bytes.
pub(super) const MANIFEST_DOMAIN: &[u8] = b"tiler.proof-sidecar.manifest.v1\0";
/// Domain separator of the manifest digest carried in the framing header.
pub(super) const MANIFEST_DIGEST_DOMAIN: &[u8] = b"tiler.proof-sidecar.manifest-digest.v1\0";
/// Domain separator of one framed payload's exact-content digest.
///
/// The pre-image is the separator, the payload's canonical ordinal, and then
/// its exact bytes. Binding the ordinal is what makes the digest a standalone
/// address of *this slot's* content: without it, two slots holding equal bytes
/// would share one address and a swap between them would be invisible.
pub(super) const PAYLOAD_DIGEST_DOMAIN: &[u8] = b"tiler.proof-sidecar.payload-digest.v1\0";
/// Versioned domain tag opening the canonical sidecar identity bytes.
pub(super) const IDENTITY_DOMAIN: &[u8] = b"tiler.proof-sidecar.identity.v1\0";

/// Maximum UTF-8 byte length of one encoded text run.
const MAX_TEXT_BYTES: usize = 4 * 1024;

/// A governed structural bound of the proof sidecar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProofLimitKind {
    /// Total encoded sidecar bytes.
    SidecarBytes,
    /// Canonical manifest bytes.
    ManifestBytes,
    /// Derived canonical identity bytes.
    IdentityBytes,
    /// Proof-case count.
    Cases,
    /// Named interface entries bound per direction.
    InterfaceEntries,
    /// Framed payload count.
    Payloads,
    /// Bytes of one framed payload.
    PayloadBytes,
    /// Bytes of one received provenance subject.
    SubjectBytes,
    /// Byte length of one encoded text run.
    TextBytes,
}

impl fmt::Display for ProofLimitKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// One governed structural bound, exceeded.
///
/// A single record shared by construction and by decoding, so a bound has one
/// name and one reported shape whichever side refused it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProofLimitExceeded {
    /// The bound that was exceeded.
    pub(crate) kind: ProofLimitKind,
    /// Quantity that was attempted.
    pub(crate) attempted: usize,
    /// Governed maximum.
    pub(crate) limit: usize,
}

impl fmt::Display for ProofLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} of {} exceeds the governed limit {}",
            self.kind, self.attempted, self.limit
        )
    }
}

impl Error for ProofLimitExceeded {}

/// Checks one governed bound.
///
/// # Errors
///
/// Returns [`ProofLimitExceeded`] when `attempted` is beyond `limit`.
pub(super) fn proof_limit(
    attempted: usize,
    limit: usize,
    kind: ProofLimitKind,
) -> Result<(), ProofLimitExceeded> {
    if attempted > limit {
        return Err(ProofLimitExceeded {
            kind,
            attempted,
            limit,
        });
    }
    Ok(())
}

/// A collection whose canonical wire order and distinctness are load-bearing.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5c): a
/// consumer that classifies a rejection matches this to decide what to report,
/// and a new ordered subject must break such a match.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProofOrderedSubject {
    /// The proof cases, ordered by stable case key.
    Case,
    /// The bound input keys, in the artifact's interface order.
    Input,
    /// The bound output keys, in the artifact's interface order.
    Output,
}

impl fmt::Display for ProofOrderedSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

/// What a consumer should do about a rejected sidecar.
///
/// The classes answer different questions, and collapsing them would make a
/// version skew look like corruption. Deliberately **not**
/// `#[non_exhaustive]` (ADR 0074 convention 5c): this is a recognizer whose
/// arms are the behaviours a consumer supports.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProofFailureClass {
    /// The bytes are not a well-formed proof sidecar.
    Malformed,
    /// A digest or a derived identity did not match the content it covers.
    IntegrityFailure,
    /// The sidecar declares a schema or encoding this build does not implement.
    Unsupported,
    /// The sidecar is well formed but violates an invariant it must satisfy.
    Invalid,
    /// The sidecar exceeds a governed structural bound.
    Limit,
}

impl fmt::Display for ProofFailureClass {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Malformed => "malformed",
            Self::IntegrityFailure => "integrity",
            Self::Unsupported => "unsupported",
            Self::Invalid => "invalid",
            Self::Limit => "limit",
        })
    }
}

/// A typed failure of proof-sidecar encoding or decoding.
///
/// Every variant names the boundary that refused. A reader never reinterprets a
/// corrupt or unsupported sidecar as a case that simply does not apply, and
/// never returns a partially validated container.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: the exhaustive map a
/// consumer needs is [`Self::classification`], which this crate owns and keeps
/// total, so no out-of-crate reader has to enumerate these variants.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ProofCodecError {
    /// The encoding ran out of bytes before a field was complete.
    Truncated {
        /// Bytes the field required.
        needed: usize,
        /// Bytes that remained.
        available: usize,
    },
    /// Bytes remained after the last framed payload.
    TrailingBytes {
        /// Count of unconsumed bytes.
        count: usize,
    },
    /// Bytes remained after the last canonical manifest field.
    TrailingManifestBytes {
        /// Count of unconsumed manifest bytes.
        count: usize,
    },
    /// The framing magic is not a proof sidecar's.
    BadMagic,
    /// The manifest does not open with its versioned domain tag.
    BadManifestDomain,
    /// The header's declared total length is not the supplied length.
    TotalLengthMismatch {
        /// Length the header declared.
        declared: u64,
        /// Length actually supplied.
        actual: u64,
    },
    /// The header and the manifest disagree on the framed payload count.
    PayloadCountMismatch {
        /// Count the framing header declared.
        header: usize,
        /// Count the manifest described.
        manifest: usize,
    },
    /// A framed payload's length is not the length its descriptor declared.
    PayloadLengthMismatch {
        /// Canonical ordinal of the payload.
        payload: u32,
        /// Length the descriptor declared.
        declared: u64,
        /// Length actually framed.
        framed: u64,
    },
    /// A payload's declared ordinal is not its canonical position.
    NonCanonicalPayloadId {
        /// Canonical position in the framed stream.
        position: usize,
        /// Ordinal the encoding declared.
        declared: u32,
    },
    /// The manifest digest does not match the manifest bytes.
    ManifestDigestMismatch,
    /// A payload's content digest does not match its framed bytes.
    PayloadDigestMismatch {
        /// Canonical ordinal of the payload.
        payload: u32,
    },
    /// The identity re-derived from the content is not the carried identity.
    SidecarIdentityMismatch,
    /// The manifest is well formed but is not its own canonical spelling.
    NonCanonicalManifest,
    /// The sidecar framing format is not one this build implements.
    UnsupportedSidecarFormat {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The canonical byte-encoding profile is not one this build implements.
    UnsupportedCanonicalEncoding {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The manifest schema is not one this build implements.
    UnsupportedManifestSchema {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The digest algorithm tag is not a governed one this build implements.
    UnsupportedDigestAlgorithm {
        /// Declared algorithm tag.
        tag: u8,
    },
    /// A governed structural bound was exceeded.
    Limit(ProofLimitExceeded),
    /// An encoded text run is not valid UTF-8.
    InvalidText,
    /// An interface key was rejected by the shared-IR key constructor.
    InvalidInterfaceKey {
        /// Typed rejection from that constructor.
        cause: BuildError,
    },
    /// A stable case key was rejected by its validating constructor.
    InvalidCaseKey {
        /// Typed rejection from that constructor.
        cause: ProofCaseKeyError,
    },
    /// A provenance subject was rejected by its validating constructor.
    InvalidSubject {
        /// Typed rejection from that constructor.
        cause: ProofSubjectError,
    },
    /// An ordered collection is not in its canonical order.
    NonCanonicalOrder {
        /// The collection that was out of order.
        subject: ProofOrderedSubject,
        /// Position of the first item that broke the order.
        position: usize,
    },
    /// An ordered collection repeats an item that must be distinct.
    DuplicateItem {
        /// The collection that repeated an item.
        subject: ProofOrderedSubject,
        /// Position of the repeat.
        position: usize,
    },
    /// The sidecar carries no case.
    NoCases,
    /// The decoded cases disagree with the sidecar's own bound interface.
    CasePayloads {
        /// The obligation that failed.
        cause: ProofInterfaceError,
    },
}

impl ProofCodecError {
    /// Classifies this rejection for a consumer deciding what to do next.
    ///
    /// The match is exhaustive with no wildcard arm, so a new boundary is a
    /// build error here and has to be classified deliberately instead of
    /// silently becoming whichever class a wildcard named.
    pub(crate) const fn classification(&self) -> ProofFailureClass {
        match self {
            Self::Truncated { .. }
            | Self::TrailingBytes { .. }
            | Self::TrailingManifestBytes { .. }
            | Self::BadMagic
            | Self::BadManifestDomain
            | Self::TotalLengthMismatch { .. }
            | Self::PayloadCountMismatch { .. }
            | Self::PayloadLengthMismatch { .. }
            | Self::InvalidText
            | Self::InvalidInterfaceKey { .. }
            | Self::InvalidCaseKey { .. }
            | Self::InvalidSubject { .. } => ProofFailureClass::Malformed,

            Self::ManifestDigestMismatch
            | Self::PayloadDigestMismatch { .. }
            | Self::SidecarIdentityMismatch => ProofFailureClass::IntegrityFailure,

            Self::UnsupportedSidecarFormat { .. }
            | Self::UnsupportedCanonicalEncoding { .. }
            | Self::UnsupportedManifestSchema { .. }
            | Self::UnsupportedDigestAlgorithm { .. } => ProofFailureClass::Unsupported,

            Self::Limit(_) => ProofFailureClass::Limit,

            Self::NonCanonicalPayloadId { .. }
            | Self::NonCanonicalManifest
            | Self::NonCanonicalOrder { .. }
            | Self::DuplicateItem { .. }
            | Self::NoCases
            | Self::CasePayloads { .. } => ProofFailureClass::Invalid,
        }
    }
}

impl fmt::Display for ProofCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "proof-sidecar.{}: ", self.classification())?;
        match self {
            Self::Truncated { needed, available } => write!(
                formatter,
                "a field needed {needed} bytes and {available} remained"
            ),
            Self::TrailingBytes { count } => {
                write!(formatter, "{count} bytes followed the last framed payload")
            }
            Self::TrailingManifestBytes { count } => {
                write!(formatter, "{count} bytes followed the last manifest field")
            }
            Self::BadMagic => formatter.write_str("the framing magic is not a proof sidecar's"),
            Self::BadManifestDomain => {
                formatter.write_str("the manifest does not open with its domain tag")
            }
            Self::TotalLengthMismatch { declared, actual } => write!(
                formatter,
                "the header declares {declared} total bytes and {actual} were supplied"
            ),
            Self::PayloadCountMismatch { header, manifest } => write!(
                formatter,
                "the header declares {header} payloads and the manifest describes {manifest}"
            ),
            Self::PayloadLengthMismatch {
                payload,
                declared,
                framed,
            } => write!(
                formatter,
                "payload {payload} is described as {declared} bytes and framed as {framed}"
            ),
            Self::NonCanonicalPayloadId { position, declared } => write!(
                formatter,
                "the payload at position {position} declares ordinal {declared}"
            ),
            Self::ManifestDigestMismatch => {
                formatter.write_str("the manifest digest does not cover the manifest bytes")
            }
            Self::PayloadDigestMismatch { payload } => write!(
                formatter,
                "payload {payload}'s digest does not cover its framed bytes"
            ),
            Self::SidecarIdentityMismatch => formatter
                .write_str("the identity re-derived from the content is not the carried identity"),
            Self::NonCanonicalManifest => {
                formatter.write_str("the manifest is not its own canonical spelling")
            }
            Self::UnsupportedSidecarFormat { major, minor } => {
                write!(formatter, "sidecar framing format {major}.{minor}")
            }
            Self::UnsupportedCanonicalEncoding { major, minor } => {
                write!(formatter, "canonical encoding profile {major}.{minor}")
            }
            Self::UnsupportedManifestSchema { major, minor } => {
                write!(formatter, "manifest schema {major}.{minor}")
            }
            Self::UnsupportedDigestAlgorithm { tag } => {
                write!(formatter, "digest algorithm tag {tag:#04x}")
            }
            Self::Limit(cause) => write!(formatter, "{cause}"),
            Self::InvalidText => formatter.write_str("an encoded text run is not valid UTF-8"),
            Self::InvalidInterfaceKey { cause } => write!(formatter, "interface key: {cause}"),
            Self::InvalidCaseKey { cause } => write!(formatter, "case key: {cause}"),
            Self::InvalidSubject { cause } => write!(formatter, "provenance subject: {cause}"),
            Self::NonCanonicalOrder { subject, position } => write!(
                formatter,
                "{subject} at position {position} breaks the canonical order"
            ),
            Self::DuplicateItem { subject, position } => {
                write!(formatter, "{subject} at position {position} is a repeat")
            }
            Self::NoCases => formatter.write_str("the sidecar carries no case"),
            Self::CasePayloads { cause } => write!(formatter, "{cause}"),
        }
    }
}

impl Error for ProofCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Limit(cause) => Some(cause),
            Self::InvalidInterfaceKey { cause } => Some(cause),
            Self::InvalidCaseKey { cause } => Some(cause),
            Self::InvalidSubject { cause } => Some(cause),
            Self::CasePayloads { cause } => Some(cause),
            Self::Truncated { .. }
            | Self::TrailingBytes { .. }
            | Self::TrailingManifestBytes { .. }
            | Self::BadMagic
            | Self::BadManifestDomain
            | Self::TotalLengthMismatch { .. }
            | Self::PayloadCountMismatch { .. }
            | Self::PayloadLengthMismatch { .. }
            | Self::NonCanonicalPayloadId { .. }
            | Self::ManifestDigestMismatch
            | Self::PayloadDigestMismatch { .. }
            | Self::SidecarIdentityMismatch
            | Self::NonCanonicalManifest
            | Self::UnsupportedSidecarFormat { .. }
            | Self::UnsupportedCanonicalEncoding { .. }
            | Self::UnsupportedManifestSchema { .. }
            | Self::UnsupportedDigestAlgorithm { .. }
            | Self::InvalidText
            | Self::NonCanonicalOrder { .. }
            | Self::DuplicateItem { .. }
            | Self::NoCases => None,
        }
    }
}

impl From<ProofLimitExceeded> for ProofCodecError {
    fn from(cause: ProofLimitExceeded) -> Self {
        Self::Limit(cause)
    }
}

impl From<ProofInterfaceError> for ProofCodecError {
    fn from(cause: ProofInterfaceError) -> Self {
        Self::CasePayloads { cause }
    }
}

/// Why a validated sidecar is not evidence about the artifact it was offered
/// with.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ProofAssociationError {
    /// The supplied envelope bytes do not digest to the recorded envelope.
    ///
    /// Checked first, because it is the cheapest and it is the one failure that
    /// distinguishes damaged bytes from the wrong artifact entirely.
    EnvelopeDigestMismatch,
    /// The supplied envelope bytes are not a valid artifact envelope.
    EnvelopeRejected {
        /// The artifact codec's own rejection.
        cause: ArtifactCodecFailure,
    },
    /// The artifact offered is not the artifact the sidecar names.
    ArtifactIdentityMismatch,
    /// The sidecar's cases disagree with the offered artifact's interface.
    Interface {
        /// The obligation that failed.
        cause: ProofInterfaceError,
    },
    /// The offered artifact's declared interface could not be projected.
    Interfaceable {
        /// The projection's own rejection.
        cause: ProofLimitExceeded,
    },
}

impl fmt::Display for ProofAssociationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvelopeDigestMismatch => formatter
                .write_str("the supplied envelope bytes are not the ones this sidecar names"),
            Self::EnvelopeRejected { cause } => {
                write!(formatter, "the supplied envelope was rejected: {cause}")
            }
            Self::ArtifactIdentityMismatch => {
                formatter.write_str("the supplied artifact is not the one this sidecar names")
            }
            Self::Interface { cause } => write!(formatter, "interface disagreement: {cause}"),
            Self::Interfaceable { cause } => {
                write!(formatter, "the artifact's interface is unbindable: {cause}")
            }
        }
    }
}

impl Error for ProofAssociationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EnvelopeRejected { cause } => Some(cause),
            Self::Interface { cause } => Some(cause),
            Self::Interfaceable { cause } => Some(cause),
            Self::EnvelopeDigestMismatch | Self::ArtifactIdentityMismatch => None,
        }
    }
}

fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("a bounded sidecar table fits u32")
}

fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

/// Derives one payload's content digest over its canonical ordinal and bytes.
fn payload_digest(algorithm: DigestAlgorithm, slot: u32, bytes: &[u8]) -> Digest {
    algorithm.digest_parts(&[PAYLOAD_DIGEST_DOMAIN, &slot.to_be_bytes(), bytes])
}

/// Walks a sidecar's payloads in the one canonical framing order.
///
/// Every producer and every reader of the stream shares this iterator, so the
/// order the encoder writes, the order the manifest describes, and the order
/// the identity folds cannot drift apart.
fn payloads(data: &ProofSidecarData) -> impl Iterator<Item = &[u8]> {
    data.cases
        .iter()
        .flat_map(|case| case.inputs.iter().chain(&case.expected).map(Vec::as_slice))
}

/// Derives the canonical identity of one sidecar's content.
///
/// Payload *digests* are folded rather than payload bytes: the identity then
/// stays bounded by the case and interface counts while still changing whenever
/// any carried byte changes.
///
/// # Errors
///
/// Returns [`ProofLimitExceeded`] when the derived identity exceeds
/// [`MAX_PROOF_IDENTITY_BYTES`].
pub(super) fn derive_identity(
    data: &ProofSidecarData,
) -> Result<CanonicalProofSidecarIdentity, ProofLimitExceeded> {
    let algorithm = DigestAlgorithm::GOVERNED;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(IDENTITY_DOMAIN);
    bytes.extend_from_slice(&MANIFEST_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&MANIFEST_SCHEMA.1.to_be_bytes());
    push_slice(&mut bytes, &data.artifact_identity);
    bytes.extend_from_slice(&data.envelope_digest);
    push_slice(&mut bytes, data.subjects.semantic.as_bytes());
    push_slice(&mut bytes, data.subjects.numerical.as_bytes());
    push_slice(&mut bytes, data.subjects.reference.as_bytes());
    push_len(&mut bytes, data.input_keys.len());
    for key in &data.input_keys {
        push_slice(&mut bytes, key.as_str().as_bytes());
    }
    push_len(&mut bytes, data.output_keys.len());
    for key in &data.output_keys {
        push_slice(&mut bytes, key.as_str().as_bytes());
    }
    push_len(&mut bytes, data.cases.len());
    let mut slot = 0_usize;
    for case in &data.cases {
        push_slice(&mut bytes, case.key.as_str().as_bytes());
        push_len(&mut bytes, case.inputs.len());
        push_len(&mut bytes, case.expected.len());
        for payload in case.inputs.iter().chain(&case.expected) {
            bytes.extend_from_slice(payload_digest(algorithm, ordinal(slot), payload).as_bytes());
            slot += 1;
        }
    }
    proof_limit(
        bytes.len(),
        MAX_PROOF_IDENTITY_BYTES,
        ProofLimitKind::IdentityBytes,
    )?;
    Ok(CanonicalProofSidecarIdentity(bytes))
}

/// Encodes one sidecar into its exact canonical bytes.
///
/// Visible to the module so its tests can seal a deliberately invalid
/// container: a forgery that carried a stale digest would be caught by
/// integrity alone, and the properties worth proving are the ones that survive
/// a correctly re-sealed forgery.
///
/// # Errors
///
/// Returns [`ProofCodecError::Limit`] when the encoding exceeds a governed
/// bound.
pub(super) fn encode(
    data: &ProofSidecarData,
    identity: &CanonicalProofSidecarIdentity,
) -> Result<Vec<u8>, ProofCodecError> {
    let algorithm = DigestAlgorithm::GOVERNED;
    let payload_count = payloads(data).count();
    proof_limit(payload_count, max_payloads(), ProofLimitKind::Payloads)?;
    let manifest = encode_manifest(data, identity, algorithm)?;
    proof_limit(
        manifest.len(),
        MAX_PROOF_MANIFEST_BYTES,
        ProofLimitKind::ManifestBytes,
    )?;

    let mut bytes = Vec::with_capacity(HEADER_BYTES + manifest.len());
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&SIDECAR_FORMAT.0.to_be_bytes());
    bytes.extend_from_slice(&SIDECAR_FORMAT.1.to_be_bytes());
    bytes.extend_from_slice(&CANONICAL_ENCODING.0.to_be_bytes());
    bytes.extend_from_slice(&CANONICAL_ENCODING.1.to_be_bytes());
    bytes.push(algorithm.tag());
    // Derived once the framing is complete; never a producer claim.
    let total_length_at = bytes.len();
    bytes.extend_from_slice(&0_u64.to_be_bytes());
    push_len(&mut bytes, manifest.len());
    bytes.extend_from_slice(&ordinal(payload_count).to_be_bytes());
    bytes.extend_from_slice(
        algorithm
            .digest(MANIFEST_DIGEST_DOMAIN, &manifest)
            .as_bytes(),
    );
    debug_assert_eq!(
        bytes.len(),
        HEADER_BYTES,
        "the framing header is fixed width"
    );
    bytes.extend_from_slice(&manifest);

    for (slot, payload) in payloads(data).enumerate() {
        bytes.extend_from_slice(&ordinal(slot).to_be_bytes());
        push_len(&mut bytes, payload.len());
        bytes.extend_from_slice(payload);
    }

    proof_limit(
        bytes.len(),
        MAX_PROOF_SIDECAR_BYTES,
        ProofLimitKind::SidecarBytes,
    )?;
    let total = u64::try_from(bytes.len()).expect("supported usize fits u64");
    bytes[total_length_at..total_length_at + 8].copy_from_slice(&total.to_be_bytes());
    Ok(bytes)
}

/// The largest framed payload count any admitted sidecar can reach.
///
/// Derived from the case and interface bounds rather than declared, so the
/// framing bound and the structural bounds cannot disagree.
const fn max_payloads() -> usize {
    MAX_PROOF_CASES * (2 * MAX_PROOF_INTERFACE_ENTRIES)
}

fn encode_manifest(
    data: &ProofSidecarData,
    identity: &CanonicalProofSidecarIdentity,
    algorithm: DigestAlgorithm,
) -> Result<Vec<u8>, ProofCodecError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MANIFEST_DOMAIN);
    bytes.extend_from_slice(&MANIFEST_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&MANIFEST_SCHEMA.1.to_be_bytes());
    push_slice(&mut bytes, &data.artifact_identity);
    bytes.extend_from_slice(&data.envelope_digest);
    push_slice(&mut bytes, data.subjects.semantic.as_bytes());
    push_slice(&mut bytes, data.subjects.numerical.as_bytes());
    push_slice(&mut bytes, data.subjects.reference.as_bytes());
    push_len(&mut bytes, data.input_keys.len());
    for key in &data.input_keys {
        push_slice(&mut bytes, key.as_str().as_bytes());
    }
    push_len(&mut bytes, data.output_keys.len());
    for key in &data.output_keys {
        push_slice(&mut bytes, key.as_str().as_bytes());
    }
    push_len(&mut bytes, data.cases.len());
    let mut slot = 0_usize;
    for case in &data.cases {
        push_slice(&mut bytes, case.key.as_str().as_bytes());
        push_len(&mut bytes, case.inputs.len());
        push_len(&mut bytes, case.expected.len());
        for payload in case.inputs.iter().chain(&case.expected) {
            proof_limit(
                payload.len(),
                MAX_PROOF_PAYLOAD_BYTES,
                ProofLimitKind::PayloadBytes,
            )?;
            let id = ordinal(slot);
            bytes.extend_from_slice(&id.to_be_bytes());
            push_len(&mut bytes, payload.len());
            bytes.extend_from_slice(payload_digest(algorithm, id, payload).as_bytes());
            slot += 1;
        }
    }
    push_slice(&mut bytes, identity.as_bytes());
    Ok(bytes)
}

impl VerifiedProofSidecar {
    /// Encodes this sidecar into its exact canonical bytes.
    ///
    /// The bytes are a function of the sidecar's content rather than of the
    /// order a producer admitted cases in, so two producers that assembled the
    /// same evidence emit the same bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProofCodecError::Limit`] when the canonical encoding exceeds a
    /// governed bound.
    pub(crate) fn encode(&self) -> Result<Vec<u8>, ProofCodecError> {
        encode(&self.data, &self.identity)
    }
}

/// Decodes and fully validates one encoded proof sidecar.
///
/// Validation is not optional and not separable: framing, manifest and payload
/// digests, schema, canonical order, case-payload agreement, and identity
/// re-derivation all run before this returns. A rejection never yields a
/// partially validated view, so holding a [`DecodedProofSidecar`] is itself the
/// evidence that the bytes passed every check.
///
/// It is *not* evidence about any artifact. Association is a separate,
/// explicit step; see [`DecodedProofSidecar::bind_to_envelope`].
///
/// # Errors
///
/// Returns the typed [`ProofCodecError`] naming the first boundary that
/// rejected.
pub(crate) fn decode_proof_sidecar(bytes: &[u8]) -> Result<DecodedProofSidecar, ProofCodecError> {
    proof_limit(
        bytes.len(),
        MAX_PROOF_SIDECAR_BYTES,
        ProofLimitKind::SidecarBytes,
    )?;
    let mut cursor = Cursor::new(bytes);
    let header = read_header(&mut cursor, bytes.len())?;

    let manifest = cursor.take(header.manifest_bytes)?;
    if header.algorithm.digest(MANIFEST_DIGEST_DOMAIN, manifest) != header.manifest_digest {
        return Err(ProofCodecError::ManifestDigestMismatch);
    }
    let parsed = parse_manifest(manifest)?;
    if parsed.descriptors.len() != header.payload_count {
        return Err(ProofCodecError::PayloadCountMismatch {
            header: header.payload_count,
            manifest: parsed.descriptors.len(),
        });
    }
    let contents = read_payloads(&mut cursor, &parsed.descriptors, header.algorithm)?;
    if cursor.remaining() != 0 {
        return Err(ProofCodecError::TrailingBytes {
            count: cursor.remaining(),
        });
    }

    let data = assemble(parsed.body, contents);
    verify_case_payloads(&data)?;
    let derived = derive_identity(&data)?;
    if derived.as_bytes() != parsed.identity {
        return Err(ProofCodecError::SidecarIdentityMismatch);
    }
    // The manifest is fully understood, so re-encoding it must reproduce the
    // exact bytes that were read. This is the backstop that makes one sidecar
    // have one byte identity: a well-formed but non-canonical spelling that no
    // named check caught fails here rather than being silently normalized.
    if encode(&data, &derived)? != bytes {
        return Err(ProofCodecError::NonCanonicalManifest);
    }
    Ok(DecodedProofSidecar {
        sidecar: VerifiedProofSidecar {
            data,
            identity: derived,
        },
    })
}

/// A validated read view over one decoded proof sidecar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DecodedProofSidecar {
    sidecar: VerifiedProofSidecar,
}

impl DecodedProofSidecar {
    /// Returns the identity re-derived from the decoded content.
    ///
    /// Re-derived, never read from the bytes: [`decode_proof_sidecar`] already
    /// proved this equals the identity the manifest carried, so a damaged
    /// manifest cannot present a chosen identity.
    pub(crate) const fn identity(&self) -> &CanonicalProofSidecarIdentity {
        self.sidecar.canonical_identity()
    }

    /// Returns the canonical identity bytes of the artifact this sidecar names.
    pub(crate) fn artifact_identity_bytes(&self) -> &[u8] {
        self.sidecar.artifact_identity_bytes()
    }

    /// Returns the digest of the exact envelope bytes this sidecar names.
    pub(crate) const fn envelope_digest(&self) -> &[u8; DIGEST_BYTES] {
        self.sidecar.envelope_digest()
    }

    /// Returns the semantic graph the expected bytes were evaluated over.
    pub(crate) const fn semantic_subject(&self) -> &ProofSemanticSubject {
        self.sidecar.semantic_subject()
    }

    /// Returns the numerical contract the expected bytes are normative under.
    pub(crate) const fn numerical_identity(&self) -> &ProofNumericalIdentity {
        self.sidecar.numerical_identity()
    }

    /// Returns the reference implementation that produced the expected bytes.
    pub(crate) const fn reference_identity(&self) -> &ProofReferenceIdentity {
        self.sidecar.reference_identity()
    }

    /// Returns the bound input keys, in the artifact's interface order.
    pub(crate) fn input_keys(&self) -> &[InputKey] {
        self.sidecar.input_keys()
    }

    /// Returns the bound output keys, in the artifact's interface order.
    pub(crate) fn output_keys(&self) -> &[OutputKey] {
        self.sidecar.output_keys()
    }

    /// Returns the proof cases in canonical case-key order.
    pub(crate) fn cases(&self) -> impl ExactSizeIterator<Item = ProofCaseRef<'_>> {
        cases_of(&self.sidecar.data)
    }

    /// Returns the case with this key, or `None`.
    pub(crate) fn case(&self, key: &ProofCaseKey) -> Option<ProofCaseRef<'_>> {
        case_of(&self.sidecar.data, key)
    }

    /// Re-encodes this decoded sidecar.
    ///
    /// A decode followed by this must reproduce the original bytes exactly.
    /// That is the round-trip property worth asserting: it proves the reader
    /// read every field the encoder wrote, because a field silently dropped on
    /// the way in cannot be written back out.
    ///
    /// # Errors
    ///
    /// Returns [`ProofCodecError::Limit`] when the canonical encoding exceeds a
    /// governed bound.
    pub(crate) fn re_encode(&self) -> Result<Vec<u8>, ProofCodecError> {
        self.sidecar.encode()
    }

    /// Proves these cases are about the artifact those exact envelope bytes
    /// encode.
    ///
    /// This is the check a consumer holding only bytes can run. It re-derives
    /// the envelope digest over the bytes supplied, decodes them through the
    /// artifact codec, and compares the re-derived artifact identity with the
    /// one recorded here. Nothing is taken on the producer's word: both values
    /// are computed from the caller's own bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProofAssociationError::EnvelopeDigestMismatch`] when the bytes
    /// are not the ones this sidecar names,
    /// [`ProofAssociationError::EnvelopeRejected`] when they are not a valid
    /// envelope, or [`ProofAssociationError::ArtifactIdentityMismatch`] when
    /// they encode a different artifact.
    pub(crate) fn bind_to_envelope(
        &self,
        envelope_bytes: &[u8],
    ) -> Result<(), ProofAssociationError> {
        if envelope_digest(envelope_bytes) != *self.envelope_digest() {
            return Err(ProofAssociationError::EnvelopeDigestMismatch);
        }
        let artifact = decode_artifact(envelope_bytes)
            .map_err(|cause| ProofAssociationError::EnvelopeRejected { cause })?;
        if artifact.identity().as_bytes() != self.artifact_identity_bytes() {
            return Err(ProofAssociationError::ArtifactIdentityMismatch);
        }
        Ok(())
    }

    /// Proves these cases are about that verified artifact, and re-proves every
    /// structural obligation against its declared interface.
    ///
    /// This is the stronger of the two checks, available to a consumer that
    /// holds the program it compiled. The association it establishes is the
    /// same one [`Self::bind_to_envelope`] establishes — both compare the
    /// artifact's canonical identity, which already folds the ordered named
    /// interface. What it adds is that the obligations are re-proven here
    /// rather than inherited through that comparison, which matters when the
    /// sidecar was written by an older producer than this reader.
    ///
    /// # Errors
    ///
    /// Returns [`ProofAssociationError::ArtifactIdentityMismatch`] for a
    /// different artifact, [`ProofAssociationError::Interfaceable`] when the
    /// artifact's declared shapes cannot be projected on this host, or
    /// [`ProofAssociationError::Interface`] when a case disagrees with the
    /// declared interface.
    pub(crate) fn bind_to_artifact(
        &self,
        artifact: &VerifiedArtifactProgram,
    ) -> Result<(), ProofAssociationError> {
        if artifact.canonical_identity().as_bytes() != self.artifact_identity_bytes() {
            return Err(ProofAssociationError::ArtifactIdentityMismatch);
        }
        let interface = project_interface(artifact).map_err(|cause| match cause {
            InterfaceProjectionError::Limit(cause) => {
                ProofAssociationError::Interfaceable { cause }
            }
            InterfaceProjectionError::Interface(cause) => {
                ProofAssociationError::Interface { cause }
            }
        })?;
        verify_cases(&interface, &self.sidecar.data)
            .map_err(|cause| ProofAssociationError::Interface { cause })
    }
}

/// The fixed framing header, read before anything is allocated for the body.
struct FramingHeader {
    algorithm: DigestAlgorithm,
    manifest_bytes: usize,
    payload_count: usize,
    manifest_digest: Digest,
}

fn read_header(cursor: &mut Cursor<'_>, supplied: usize) -> Result<FramingHeader, ProofCodecError> {
    if cursor.take(MAGIC.len())? != MAGIC {
        return Err(ProofCodecError::BadMagic);
    }
    let format = (cursor.u16()?, cursor.u16()?);
    if format.0 != SIDECAR_FORMAT.0 || format.1 > SIDECAR_FORMAT.1 {
        return Err(ProofCodecError::UnsupportedSidecarFormat {
            major: format.0,
            minor: format.1,
        });
    }
    let encoding = (cursor.u16()?, cursor.u16()?);
    if encoding.0 != CANONICAL_ENCODING.0 || encoding.1 > CANONICAL_ENCODING.1 {
        return Err(ProofCodecError::UnsupportedCanonicalEncoding {
            major: encoding.0,
            minor: encoding.1,
        });
    }
    let algorithm_tag = cursor.u8()?;
    let algorithm = DigestAlgorithm::from_tag(algorithm_tag)
        .ok_or(ProofCodecError::UnsupportedDigestAlgorithm { tag: algorithm_tag })?;
    let declared_total = cursor.u64()?;
    let actual_total = u64::try_from(supplied).expect("supported usize fits u64");
    if declared_total != actual_total {
        return Err(ProofCodecError::TotalLengthMismatch {
            declared: declared_total,
            actual: actual_total,
        });
    }
    let manifest_bytes = cursor.count(MAX_PROOF_MANIFEST_BYTES, ProofLimitKind::ManifestBytes)?;
    let payload_count = position(cursor.u32()?);
    proof_limit(payload_count, max_payloads(), ProofLimitKind::Payloads)?;
    let manifest_digest = Digest::from_wire(cursor.array()?);
    debug_assert_eq!(
        cursor.position, HEADER_BYTES,
        "the framing header is fixed width",
    );
    Ok(FramingHeader {
        algorithm,
        manifest_bytes,
        payload_count,
        manifest_digest,
    })
}

/// One payload descriptor, held only until its framed bytes are validated.
struct PayloadDescriptor {
    id: u32,
    exact_len: u64,
    digest: Digest,
}

/// Everything the manifest carries except its derived payload descriptors.
struct DecodedBody {
    artifact_identity: Vec<u8>,
    envelope_digest: [u8; DIGEST_BYTES],
    subjects: ProofSubjects,
    input_keys: Vec<InputKey>,
    output_keys: Vec<OutputKey>,
    cases: Vec<CaseRow>,
}

/// One case's stable key and the payload counts its manifest row declares.
struct CaseRow {
    key: ProofCaseKey,
    inputs: usize,
    expected: usize,
}

/// The case table and the payload descriptors its rows account for.
struct CaseTable {
    rows: Vec<CaseRow>,
    descriptors: Vec<PayloadDescriptor>,
}

struct ParsedManifest {
    body: DecodedBody,
    descriptors: Vec<PayloadDescriptor>,
    identity: Vec<u8>,
}

fn parse_manifest(bytes: &[u8]) -> Result<ParsedManifest, ProofCodecError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(MANIFEST_DOMAIN.len())? != MANIFEST_DOMAIN {
        return Err(ProofCodecError::BadManifestDomain);
    }
    let schema = (cursor.u16()?, cursor.u16()?);
    if schema.0 != MANIFEST_SCHEMA.0 || schema.1 > MANIFEST_SCHEMA.1 {
        return Err(ProofCodecError::UnsupportedManifestSchema {
            major: schema.0,
            minor: schema.1,
        });
    }
    let artifact_identity = cursor.slice()?.to_vec();
    let envelope_digest: [u8; DIGEST_BYTES] = cursor.array()?;
    let subjects = ProofSubjects {
        semantic: subject(&mut cursor)?,
        numerical: subject(&mut cursor)?,
        reference: subject(&mut cursor)?,
    };

    let input_keys = read_interface_keys(&mut cursor, InputKey::from_owned)?;
    require_distinct(&input_keys, ProofOrderedSubject::Input)?;
    let output_keys = read_interface_keys(&mut cursor, OutputKey::from_owned)?;
    require_distinct(&output_keys, ProofOrderedSubject::Output)?;

    let CaseTable { rows, descriptors } = read_case_table(&mut cursor)?;
    let identity = cursor.slice()?.to_vec();
    if cursor.remaining() != 0 {
        return Err(ProofCodecError::TrailingManifestBytes {
            count: cursor.remaining(),
        });
    }
    Ok(ParsedManifest {
        body: DecodedBody {
            artifact_identity,
            envelope_digest,
            subjects,
            input_keys,
            output_keys,
            cases: rows,
        },
        descriptors,
        identity,
    })
}

/// Reads one half of the bound named interface.
fn read_interface_keys<K>(
    cursor: &mut Cursor<'_>,
    construct: impl Fn(String) -> Result<K, BuildError>,
) -> Result<Vec<K>, ProofCodecError> {
    let count = cursor.count(
        MAX_PROOF_INTERFACE_ENTRIES,
        ProofLimitKind::InterfaceEntries,
    )?;
    let mut keys = Vec::with_capacity(count);
    for _ in 0..count {
        keys.push(
            construct(cursor.text()?)
                .map_err(|cause| ProofCodecError::InvalidInterfaceKey { cause })?,
        );
    }
    Ok(keys)
}

/// Reads the case table and the payload descriptors its declarations imply.
///
/// The descriptors are derived from the per-case payload counts rather than
/// read as an independent table, so a manifest cannot describe more or fewer
/// payloads than its cases account for.
fn read_case_table(cursor: &mut Cursor<'_>) -> Result<CaseTable, ProofCodecError> {
    let case_count = cursor.count(MAX_PROOF_CASES, ProofLimitKind::Cases)?;
    if case_count == 0 {
        return Err(ProofCodecError::NoCases);
    }
    let mut rows = Vec::with_capacity(case_count);
    let mut descriptors = Vec::new();
    let mut slot = 0_usize;
    for _ in 0..case_count {
        let key = ProofCaseKey::from_owned(cursor.text()?)
            .map_err(|cause| ProofCodecError::InvalidCaseKey { cause })?;
        let inputs = cursor.count(
            MAX_PROOF_INTERFACE_ENTRIES,
            ProofLimitKind::InterfaceEntries,
        )?;
        let expected = cursor.count(
            MAX_PROOF_INTERFACE_ENTRIES,
            ProofLimitKind::InterfaceEntries,
        )?;
        for _ in 0..(inputs + expected) {
            let id = cursor.u32()?;
            if position(id) != slot {
                return Err(ProofCodecError::NonCanonicalPayloadId {
                    position: slot,
                    declared: id,
                });
            }
            let exact_len = cursor.u64()?;
            let digest = Digest::from_wire(cursor.array()?);
            descriptors.push(PayloadDescriptor {
                id,
                exact_len,
                digest,
            });
            slot += 1;
        }
        rows.push(CaseRow {
            key,
            inputs,
            expected,
        });
    }
    // Case keys carry the canonical order of the whole container, so the order
    // is checked here rather than left to the re-encode backstop: a reader that
    // reported "not canonical" for an out-of-order case list would name the
    // wrong cause.
    require_sorted_and_distinct(rows.iter().map(|row| &row.key), ProofOrderedSubject::Case)?;
    Ok(CaseTable { rows, descriptors })
}

fn read_payloads(
    cursor: &mut Cursor<'_>,
    descriptors: &[PayloadDescriptor],
    algorithm: DigestAlgorithm,
) -> Result<Vec<Vec<u8>>, ProofCodecError> {
    let mut contents = Vec::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.iter().enumerate() {
        let declared_id = cursor.u32()?;
        if position(declared_id) != index || descriptor.id != declared_id {
            return Err(ProofCodecError::NonCanonicalPayloadId {
                position: index,
                declared: declared_id,
            });
        }
        let framed = cursor.count(MAX_PROOF_PAYLOAD_BYTES, ProofLimitKind::PayloadBytes)?;
        let framed_len = u64::try_from(framed).expect("supported usize fits u64");
        if framed_len != descriptor.exact_len {
            return Err(ProofCodecError::PayloadLengthMismatch {
                payload: declared_id,
                declared: descriptor.exact_len,
                framed: framed_len,
            });
        }
        let content = cursor.take(framed)?;
        if payload_digest(algorithm, declared_id, content) != descriptor.digest {
            return Err(ProofCodecError::PayloadDigestMismatch {
                payload: declared_id,
            });
        }
        contents.push(content.to_vec());
    }
    Ok(contents)
}

/// Rebuilds the sidecar's data from the parsed manifest and framed payloads.
///
/// The split is positional and total: each case declared how many payloads it
/// owns, the descriptors were counted from those declarations, and the framed
/// stream was read one payload per descriptor, so the drain below consumes the
/// stream exactly.
fn assemble(body: DecodedBody, contents: Vec<Vec<u8>>) -> ProofSidecarData {
    let mut remaining = contents.into_iter();
    let cases = body
        .cases
        .into_iter()
        .map(|row| ProofCaseData {
            key: row.key,
            inputs: remaining.by_ref().take(row.inputs).collect(),
            expected: remaining.by_ref().take(row.expected).collect(),
        })
        .collect();
    ProofSidecarData {
        artifact_identity: body.artifact_identity,
        envelope_digest: body.envelope_digest,
        subjects: body.subjects,
        input_keys: body.input_keys,
        output_keys: body.output_keys,
        cases,
    }
}

fn subject<T>(cursor: &mut Cursor<'_>) -> Result<T, ProofCodecError>
where
    T: FromSubjectBytes,
{
    let bytes = cursor.slice()?;
    proof_limit(
        bytes.len(),
        MAX_PROOF_SUBJECT_BYTES,
        ProofLimitKind::SubjectBytes,
    )?;
    T::from_subject_bytes(bytes).map_err(|cause| ProofCodecError::InvalidSubject { cause })
}

/// Lets one reader construct any of the three received provenance subjects.
///
/// A trait rather than three near-identical readers, and crate-private rather
/// than a public seam: it exists so the three subjects cannot be read by three
/// implementations that drift, not so anything outside this module implements
/// it.
pub(super) trait FromSubjectBytes: Sized {
    fn from_subject_bytes(bytes: &[u8]) -> Result<Self, ProofSubjectError>;
}

macro_rules! from_subject_bytes {
    ($name:ty) => {
        impl FromSubjectBytes for $name {
            fn from_subject_bytes(bytes: &[u8]) -> Result<Self, ProofSubjectError> {
                Self::from_bytes(bytes)
            }
        }
    };
}

from_subject_bytes!(ProofSemanticSubject);
from_subject_bytes!(ProofNumericalIdentity);
from_subject_bytes!(ProofReferenceIdentity);

fn require_distinct<T: Ord>(
    items: &[T],
    subject: ProofOrderedSubject,
) -> Result<(), ProofCodecError> {
    for (position, item) in items.iter().enumerate() {
        if items[..position].contains(item) {
            return Err(ProofCodecError::DuplicateItem { subject, position });
        }
    }
    Ok(())
}

fn require_sorted_and_distinct<'a, T: Ord + 'a>(
    items: impl Iterator<Item = &'a T>,
    subject: ProofOrderedSubject,
) -> Result<(), ProofCodecError> {
    let mut previous: Option<&T> = None;
    for (position, item) in items.enumerate() {
        if let Some(previous) = previous {
            match item.cmp(previous) {
                std::cmp::Ordering::Less => {
                    return Err(ProofCodecError::NonCanonicalOrder { subject, position });
                }
                std::cmp::Ordering::Equal => {
                    return Err(ProofCodecError::DuplicateItem { subject, position });
                }
                std::cmp::Ordering::Greater => {}
            }
        }
        previous = Some(item);
    }
    Ok(())
}

/// A bounded forward reader over one encoded sidecar.
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], ProofCodecError> {
        let available = self.remaining();
        if len > available {
            return Err(ProofCodecError::Truncated {
                needed: len,
                available,
            });
        }
        let taken = &self.bytes[self.position..self.position + len];
        self.position += len;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], ProofCodecError> {
        let taken = self.take(N)?;
        Ok(taken.try_into().expect("the exact width was taken"))
    }

    fn u8(&mut self) -> Result<u8, ProofCodecError> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, ProofCodecError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, ProofCodecError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, ProofCodecError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    /// Reads a declared count and proves it is within its governed bound.
    ///
    /// The bound is checked before anything proportional to the count is
    /// reserved, so a forged count reports the exceeded limit rather than
    /// allocating for content that is not there.
    fn count(&mut self, limit: usize, kind: ProofLimitKind) -> Result<usize, ProofCodecError> {
        let declared = self.u64()?;
        let available = self.remaining();
        let count = usize::try_from(declared).map_err(|_| ProofCodecError::Truncated {
            needed: available.saturating_add(1),
            available,
        })?;
        proof_limit(count, limit, kind)?;
        Ok(count)
    }

    /// Reads one length-prefixed byte run.
    ///
    /// The declared length is bounded by what remains before it is consumed, so
    /// a forged length reports truncation rather than reserving for content
    /// that is not there. The semantic bound on each such run belongs to the
    /// constructor that wraps it.
    fn slice(&mut self) -> Result<&'a [u8], ProofCodecError> {
        let declared = self.u64()?;
        let available = self.remaining();
        let len = usize::try_from(declared).map_err(|_| ProofCodecError::Truncated {
            needed: available.saturating_add(1),
            available,
        })?;
        self.take(len)
    }

    fn text(&mut self) -> Result<String, ProofCodecError> {
        let len = self.count(MAX_TEXT_BYTES, ProofLimitKind::TextBytes)?;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ProofCodecError::InvalidText)
    }
}
