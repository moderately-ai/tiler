//! The sidecar's immutable verified product, its vocabulary, and its identity.
//!
//! Three shapes appear here and they are deliberately different (ADR 0074 §2).
//!
//! A **stable case key** is bounded UTF-8 text a producer chooses and every
//! later run must keep spelling the same way. It is meaning: it is what a
//! failure report names, what a regression is tracked by, and what makes two
//! runs of the same case comparable. It has a validating constructor.
//!
//! A **received provenance subject** is bytes another authority derived — the
//! semantic graph identity, the numerical contract identity, the reference
//! implementation identity. This module compares and encodes them and never
//! re-derives them, because it is not the authority for any of the three.
//!
//! A **derived sidecar identity** is this module's own, produced only by the
//! encoder in [`super::codec`]. It has no constructor at all, so no caller can
//! assemble one naming content no encoder examined.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{InputKey, OutputKey};

use crate::program::DIGEST_BYTES;

use super::{MAX_PROOF_CASE_KEY_BYTES, MAX_PROOF_SUBJECT_BYTES};

/// Failure to construct a stable proof-case key.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies, never one any crate maps totally.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ProofCaseKeyError {
    /// The key was empty.
    ///
    /// An empty key is refused rather than normalized because a case key is
    /// how a failure is reported and how a regression is tracked; an unnamed
    /// case is not trackable.
    Empty,
    /// The key exceeded the governed byte bound.
    TooLong {
        /// Byte length of the rejected key.
        bytes: usize,
        /// Governed maximum.
        limit: usize,
    },
}

impl fmt::Display for ProofCaseKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a proof-case key is empty"),
            Self::TooLong { bytes, limit } => write!(
                formatter,
                "a proof-case key of {bytes} bytes exceeds the governed limit {limit}"
            ),
        }
    }
}

impl Error for ProofCaseKeyError {}

/// Failure to accept a received provenance subject.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub(crate) enum ProofSubjectError {
    /// The subject bytes were empty.
    ///
    /// An absent identity must be absent by construction, not spelled as zero
    /// bytes: a sidecar that recorded an empty subject would state that it
    /// knows which authority produced its expectations while naming none.
    Empty,
    /// The subject exceeded the governed byte bound.
    TooLong {
        /// Byte length of the rejected subject.
        bytes: usize,
        /// Governed maximum.
        limit: usize,
    },
}

impl fmt::Display for ProofSubjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a proof provenance subject is empty"),
            Self::TooLong { bytes, limit } => write!(
                formatter,
                "a proof provenance subject of {bytes} bytes exceeds the governed limit {limit}"
            ),
        }
    }
}

impl Error for ProofSubjectError {}

/// A stable key naming one proof case across producer runs.
///
/// Stability is the whole point. The key is chosen by the producer, encoded
/// into the sidecar's canonical identity, and required to be unique within one
/// sidecar; a consumer reports and tracks a case by it. Nothing derives it, so
/// renaming a case is a deliberate act that changes the sidecar's identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProofCaseKey(String);

impl ProofCaseKey {
    /// Creates a validated stable case key.
    ///
    /// # Errors
    ///
    /// Returns [`ProofCaseKeyError::Empty`] for an empty key, or
    /// [`ProofCaseKeyError::TooLong`] beyond [`MAX_PROOF_CASE_KEY_BYTES`].
    pub(crate) fn new(value: impl AsRef<str>) -> Result<Self, ProofCaseKeyError> {
        Self::from_owned(value.as_ref().to_owned())
    }

    /// Validates and retains an already-owned key without copying it.
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`Self::new`], before retaining the string.
    pub(crate) fn from_owned(value: String) -> Result<Self, ProofCaseKeyError> {
        if value.is_empty() {
            return Err(ProofCaseKeyError::Empty);
        }
        if value.len() > MAX_PROOF_CASE_KEY_BYTES {
            return Err(ProofCaseKeyError::TooLong {
                bytes: value.len(),
                limit: MAX_PROOF_CASE_KEY_BYTES,
            });
        }
        Ok(Self(value))
    }

    /// Returns the exact key text.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProofCaseKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

macro_rules! received_subject {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        ///
        /// The bytes are opaque: this module compares and encodes them and
        /// never re-derives them, because it is not the authority for the
        /// subject they name (ADR 0074 convention 2).
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(crate) struct $name(Vec<u8>);

        impl $name {
            /// Wraps subject bytes another authority derived.
            ///
            /// # Errors
            ///
            /// Returns [`ProofSubjectError::Empty`] for empty bytes, or
            /// [`ProofSubjectError::TooLong`] beyond
            /// [`MAX_PROOF_SUBJECT_BYTES`].
            pub(crate) fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, ProofSubjectError> {
                let value = value.as_ref();
                if value.is_empty() {
                    return Err(ProofSubjectError::Empty);
                }
                if value.len() > MAX_PROOF_SUBJECT_BYTES {
                    return Err(ProofSubjectError::TooLong {
                        bytes: value.len(),
                        limit: MAX_PROOF_SUBJECT_BYTES,
                    });
                }
                Ok(Self(value.to_vec()))
            }

            /// Returns the exact subject bytes.
            pub(crate) fn as_bytes(&self) -> &[u8] {
                &self.0
            }
        }
    };
}

received_subject!(
    ProofSemanticSubject,
    "The canonical semantic-graph identity the expected bytes were evaluated over."
);
received_subject!(
    ProofNumericalIdentity,
    "The identity of the numerical contract under which the expected bytes are normative."
);
received_subject!(
    ProofReferenceIdentity,
    "The identity of the reference implementation that produced the expected bytes."
);

/// The three authorities a proof expectation depends on, recorded together.
///
/// They answer three different staleness questions and are therefore kept
/// separately typed rather than folded into one blob. The semantic subject says
/// *which mathematical program* was evaluated; the numerical identity says
/// *under which contract* its result is normative; the reference identity says
/// *which implementation* computed it. A sidecar can be stale against any one
/// of the three while agreeing with the other two.
///
/// The frozen registry snapshot the semantic layer also identifies is
/// deliberately absent, for ADR 0072's reason: a provider that was available
/// and never reached does not change what the program computes, so recording it
/// would let an unused provider invalidate a still-correct expectation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProofSubjects {
    pub(crate) semantic: ProofSemanticSubject,
    pub(crate) numerical: ProofNumericalIdentity,
    pub(crate) reference: ProofReferenceIdentity,
}

/// One proof case, as retained.
///
/// The payload vectors are positionally aligned with the sidecar's bound
/// interface keys, which is why they carry no keys of their own: a case that
/// could name its own keys could name a different set from its siblings, and
/// the sidecar would then have no single interface to associate with an
/// artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProofCaseData {
    pub(crate) key: ProofCaseKey,
    pub(crate) inputs: Vec<Vec<u8>>,
    pub(crate) expected: Vec<Vec<u8>>,
}

/// Everything one sidecar carries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProofSidecarData {
    /// Canonical identity bytes of the associated artifact program.
    ///
    /// Held as bytes rather than as a
    /// [`CanonicalArtifactProgramIdentity`](crate::program::CanonicalArtifactProgramIdentity)
    /// because that type has no constructor outside its own encoder — which is
    /// the property that makes it unforgeable, and which a decoder therefore
    /// cannot and must not work around.
    pub(crate) artifact_identity: Vec<u8>,
    /// Digest over the exact encoded bytes of the associated envelope.
    pub(crate) envelope_digest: [u8; DIGEST_BYTES],
    pub(crate) subjects: ProofSubjects,
    /// The artifact's declared inputs, in the artifact's own interface order.
    pub(crate) input_keys: Vec<InputKey>,
    /// The artifact's declared outputs, in the artifact's own interface order.
    pub(crate) output_keys: Vec<OutputKey>,
    /// The cases, in canonical case-key order.
    pub(crate) cases: Vec<ProofCaseData>,
}

/// Opaque canonical bytes identifying one proof sidecar.
///
/// The identity folds the association, the three provenance subjects, the bound
/// interface keys, every case key, and a content digest of every payload. It
/// deliberately folds payload *digests* rather than payload bytes, so the
/// identity stays bounded by the case count while still changing whenever any
/// carried byte changes.
///
/// There is no constructor: [`super::codec`] derives it and nothing else can
/// (ADR 0074 convention 2).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CanonicalProofSidecarIdentity(pub(super) Vec<u8>);

impl CanonicalProofSidecarIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, verified proof-case sidecar.
///
/// Only [`super::ProofSidecarBuilder::build`] produces one. Equality compares
/// the canonical identity.
#[derive(Clone)]
pub(crate) struct VerifiedProofSidecar {
    pub(super) data: ProofSidecarData,
    pub(super) identity: CanonicalProofSidecarIdentity,
}

impl PartialEq for VerifiedProofSidecar {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for VerifiedProofSidecar {}

impl fmt::Debug for VerifiedProofSidecar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedProofSidecar")
            .field("cases", &self.data.cases.len())
            .field("inputs", &self.data.input_keys.len())
            .field("outputs", &self.data.output_keys.len())
            .field("identity_bytes", &self.identity.0.len())
            .finish()
    }
}

impl VerifiedProofSidecar {
    /// Returns the canonical identity of this sidecar.
    pub(crate) const fn canonical_identity(&self) -> &CanonicalProofSidecarIdentity {
        &self.identity
    }

    /// Returns the canonical identity bytes of the artifact this sidecar names.
    pub(crate) fn artifact_identity_bytes(&self) -> &[u8] {
        &self.data.artifact_identity
    }

    /// Returns the digest of the exact envelope bytes this sidecar names.
    pub(crate) const fn envelope_digest(&self) -> &[u8; DIGEST_BYTES] {
        &self.data.envelope_digest
    }

    /// Returns the semantic graph the expected bytes were evaluated over.
    pub(crate) const fn semantic_subject(&self) -> &ProofSemanticSubject {
        &self.data.subjects.semantic
    }

    /// Returns the numerical contract the expected bytes are normative under.
    pub(crate) const fn numerical_identity(&self) -> &ProofNumericalIdentity {
        &self.data.subjects.numerical
    }

    /// Returns the reference implementation that produced the expected bytes.
    pub(crate) const fn reference_identity(&self) -> &ProofReferenceIdentity {
        &self.data.subjects.reference
    }

    /// Returns the bound input keys, in the artifact's interface order.
    pub(crate) fn input_keys(&self) -> &[InputKey] {
        &self.data.input_keys
    }

    /// Returns the bound output keys, in the artifact's interface order.
    pub(crate) fn output_keys(&self) -> &[OutputKey] {
        &self.data.output_keys
    }

    /// Returns the proof cases in canonical case-key order.
    pub(crate) fn cases(&self) -> impl ExactSizeIterator<Item = ProofCaseRef<'_>> {
        cases_of(&self.data)
    }

    /// Returns the case with this key, or `None`.
    pub(crate) fn case(&self, key: &ProofCaseKey) -> Option<ProofCaseRef<'_>> {
        case_of(&self.data, key)
    }
}

/// A read view over one proof case.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProofCaseRef<'a> {
    data: &'a ProofSidecarData,
    case: &'a ProofCaseData,
}

impl<'a> ProofCaseRef<'a> {
    /// Returns this case's stable key.
    pub(crate) const fn key(self) -> &'a ProofCaseKey {
        &self.case.key
    }

    /// Returns the bit-preserving input payloads, in interface order.
    pub(crate) fn inputs(self) -> impl ExactSizeIterator<Item = ProofPayloadRef<'a, InputKey>> {
        self.data
            .input_keys
            .iter()
            .zip(&self.case.inputs)
            .map(|(key, bytes)| ProofPayloadRef { key, bytes })
    }

    /// Returns the normative expected payloads, in interface order.
    pub(crate) fn expected(self) -> impl ExactSizeIterator<Item = ProofPayloadRef<'a, OutputKey>> {
        self.data
            .output_keys
            .iter()
            .zip(&self.case.expected)
            .map(|(key, bytes)| ProofPayloadRef { key, bytes })
    }
}

/// One named payload of one proof case.
///
/// `Clone` and `Copy` are written rather than derived: a derive would demand
/// `K: Copy`, and the interface keys this view names are heap-backed. The view
/// itself is two references and is unconditionally copyable.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ProofPayloadRef<'a, K> {
    key: &'a K,
    bytes: &'a [u8],
}

impl<K> Clone for ProofPayloadRef<'_, K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for ProofPayloadRef<'_, K> {}

impl<'a, K> ProofPayloadRef<'a, K> {
    /// Returns the interface key this payload belongs to.
    pub(crate) const fn key(self) -> &'a K {
        self.key
    }

    /// Returns the exact payload bytes.
    ///
    /// Bit-preserving: they are carried and compared as bytes and are never
    /// interpreted as numbers here. A canonical NaN, a signed zero, and a
    /// subnormal all survive this container unchanged, which is the only reason
    /// a bitwise readback comparison means anything.
    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.bytes
    }
}

pub(super) fn cases_of(data: &ProofSidecarData) -> impl ExactSizeIterator<Item = ProofCaseRef<'_>> {
    data.cases
        .iter()
        .map(move |case| ProofCaseRef { data, case })
}

pub(super) fn case_of<'a>(
    data: &'a ProofSidecarData,
    key: &ProofCaseKey,
) -> Option<ProofCaseRef<'a>> {
    // Cases are held in canonical key order, so the lookup is a binary search
    // rather than a scan, and the ordering the encoder relies on is the same
    // ordering that makes this correct.
    data.cases
        .binary_search_by(|case| case.key.cmp(key))
        .ok()
        .map(|index| ProofCaseRef {
            data,
            case: &data.cases[index],
        })
}
