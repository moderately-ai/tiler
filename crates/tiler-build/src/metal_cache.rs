//! Metal's specialization of the neutral expansion-cache seam.
//!
//! Everything structural — subject composition, miss-only compilation, identity
//! agreement before publication, re-validation of every result — belongs to
//! [`crate::payload_cache`] and is shared with every other backend. What is
//! Metal's and stays here is exactly two statements: the governed
//! `tiler.metal`/`metallib`/`NativeImage` payload descriptor this backend
//! declares, and the fact-level correspondence between a carried payload's
//! metadata and the Apple compilation that was prepared for it. The first is
//! data and travels in a [`DeclaredPayload`]; the second is a closure, because
//! naming *which* compilation fact disagreed is a judgement only this backend
//! can make.
//!
//! The refusal vocabulary below is Metal's own and is preserved exactly: the
//! neutral seam's protocol refusals map one-to-one onto it, so a caller reading
//! [`MetalArtifactProtocolError`] sees the same kinds in the same order it saw
//! before the seam was promoted.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, VerifiedArtifactProgram,
};
use tiler_cache::expansion::{ExpansionCache, SubjectRefusal};

use crate::MetalPayloadMismatch;
use crate::metal_assembly::{
    CompiledMetalPayload, MetalAssemblyError, PAYLOAD_SCHEMA, PreparedMetalPayload,
};
use crate::metal_payload::validate_metal_payload_metadata;
use crate::payload_cache::{
    AcceptedArtifact, DeclaredPayload, SinglePayloadCacheError, SinglePayloadProtocolError,
    accept_or_publish_single_payload_artifact,
};

/// A decoded or assembled artifact contradicts the singular prepared Metal subject.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalArtifactProtocolError {
    /// The artifact does not carry exactly the one backend compilation in the cache subject.
    PayloadPortfolio {
        /// Number of payload descriptors the artifact carries.
        actual: usize,
    },
    /// The sole payload is not the governed Metal native-image descriptor.
    PayloadDescriptor,
    /// The carried payload omits its compilation metadata.
    MissingPayloadMetadata,
    /// The carried payload omits its compiled object.
    MissingPayloadObject,
    /// The carried metadata contradicts the current prepared compilation.
    Correspondence(MetalPayloadMismatch),
    /// The payload's complete compilation subject differs from the pending declaration.
    PayloadSubject,
    /// The compiled artifact differs from the pending artifact program used in the key.
    ArtifactIdentity,
    /// The artifact carries object bytes other than the exact miss-produced object.
    PayloadObject,
}

impl fmt::Display for MetalArtifactProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadPortfolio { actual } => write!(
                formatter,
                "Metal cache orchestration requires exactly one payload, found {actual}",
            ),
            Self::PayloadDescriptor => formatter.write_str(
                "artifact payload is not the governed Metal metallib native-image descriptor",
            ),
            Self::MissingPayloadMetadata => {
                formatter.write_str("artifact Metal payload carries no compilation metadata")
            }
            Self::MissingPayloadObject => {
                formatter.write_str("artifact Metal payload carries no compiled object")
            }
            Self::Correspondence(error) => error.fmt(formatter),
            Self::PayloadSubject => formatter
                .write_str("artifact Metal payload names a different complete compilation subject"),
            Self::ArtifactIdentity => formatter.write_str(
                "compiled artifact identity differs from the pending artifact cache subject",
            ),
            Self::PayloadObject => {
                formatter.write_str("artifact Metal payload carries a different compiled object")
            }
        }
    }
}

impl Error for MetalArtifactProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correspondence(error) => Some(error),
            _ => None,
        }
    }
}

impl From<SinglePayloadProtocolError<MetalPayloadMismatch>> for MetalArtifactProtocolError {
    /// Renames the neutral protocol refusals into Metal's own vocabulary.
    ///
    /// Exhaustive by arm rather than by wildcard, so a refusal added to the
    /// neutral seam is a build error here instead of a Metal diagnostic that
    /// silently loses a case.
    fn from(error: SinglePayloadProtocolError<MetalPayloadMismatch>) -> Self {
        match error {
            SinglePayloadProtocolError::PayloadPortfolio { actual } => {
                Self::PayloadPortfolio { actual }
            }
            SinglePayloadProtocolError::PayloadDescriptor => Self::PayloadDescriptor,
            SinglePayloadProtocolError::MissingPayloadMetadata => Self::MissingPayloadMetadata,
            SinglePayloadProtocolError::Correspondence(mismatch) => Self::Correspondence(mismatch),
            SinglePayloadProtocolError::PayloadSubject => Self::PayloadSubject,
            SinglePayloadProtocolError::MissingPayloadObject => Self::MissingPayloadObject,
            SinglePayloadProtocolError::PayloadObject => Self::PayloadObject,
            SinglePayloadProtocolError::ArtifactIdentity => Self::ArtifactIdentity,
        }
    }
}

/// Why cache orchestration could not return an accepted Metal artifact.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalCacheError<E> {
    /// The complete cache subject could not be composed.
    Subject(SubjectRefusal),
    /// The prepared Metal compiler or linker failed on a cache miss.
    Compile(MetalAssemblyError),
    /// The caller could not assemble the compiled payload into its artifact.
    Assemble(E),
    /// The caller's verified artifact could not be encoded.
    Encode(ArtifactCodecFailure),
    /// The cache's governed artifact validator rejected the produced envelope.
    CacheArtifact(ArtifactCodecFailure),
    /// The pending, produced, or cached artifact contradicted the prepared operation.
    Protocol(MetalArtifactProtocolError),
}

impl<E: fmt::Display> fmt::Display for MetalCacheError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subject(error) => write!(formatter, "Metal cache subject was refused: {error}"),
            Self::Compile(error) => error.fmt(formatter),
            Self::Assemble(error) => write!(formatter, "Metal artifact assembly failed: {error}"),
            Self::Encode(error) => write!(formatter, "Metal artifact encoding failed: {error}"),
            Self::CacheArtifact(error) => {
                write!(
                    formatter,
                    "expansion cache refused the generated artifact: {error}"
                )
            }
            Self::Protocol(error) => error.fmt(formatter),
        }
    }
}

impl<E: Error + 'static> Error for MetalCacheError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Subject(error) => Some(error),
            Self::Compile(error) => Some(error),
            Self::Assemble(error) => Some(error),
            Self::Encode(error) | Self::CacheArtifact(error) => Some(error),
            Self::Protocol(error) => Some(error),
        }
    }
}

impl<E> From<SinglePayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>>
    for MetalCacheError<E>
{
    fn from(error: SinglePayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>) -> Self {
        match error {
            SinglePayloadCacheError::Subject(error) => Self::Subject(error),
            SinglePayloadCacheError::Compile(error) => Self::Compile(error),
            SinglePayloadCacheError::Assemble(error) => Self::Assemble(error),
            SinglePayloadCacheError::Encode(error) => Self::Encode(error),
            SinglePayloadCacheError::CacheArtifact(error) => Self::CacheArtifact(error),
            SinglePayloadCacheError::Protocol(error) => Self::Protocol(error.into()),
        }
    }
}

/// Resolves one singular prepared Metal artifact through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation. `assemble` runs only on a cache miss and
/// must assemble the supplied compiled payload into the corresponding carried
/// artifact. The carried identity and payload are checked before publication;
/// every cache result is checked again before this function returns.
///
/// This binds [`accept_or_publish_single_payload_artifact`] to Metal's two
/// statements and nothing else: the prepared payload already holds its governed
/// descriptor keys and its derived compilation digest, and this crate's own
/// `validate_metal_payload_metadata` is the correspondence closure. The
/// compilation facet the cache subject names is the AOT driver's own prepared
/// identity, which has no public constructor, so it cannot be minted from
/// invented toolchain facts.
///
/// # Errors
///
/// Returns a typed subject, compilation, assembly, codec, or protocol failure.
/// A protocol failure is hard: it is never translated into a cache miss or an
/// automatic rebuild.
pub fn accept_or_publish_single_payload_metal_artifact<E>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    prepared: PreparedMetalPayload<'_>,
    assemble: impl FnOnce(CompiledMetalPayload) -> Result<VerifiedArtifactProgram, E>,
) -> Result<AcceptedArtifact, MetalCacheError<E>> {
    let backend = prepared.backend().clone();
    let representation = prepared.representation().clone();
    let digest = prepared.digest().clone();
    let expected_metadata = prepared.metadata().clone();
    // Owned because the declaration outlives the prepared token: the compile
    // closure consumes the token, and the same declaration is compared again
    // after resolution.
    let compilation = prepared.compilation_identity_bytes().to_vec();
    let declared = DeclaredPayload {
        backend: &backend,
        representation: &representation,
        payload_schema: PAYLOAD_SCHEMA,
        execution_policy: ArtifactExecutionPolicy::NativeImage,
        digest: &digest,
        compilation: &compilation,
    };
    let (prepared, compiled_metadata, _digest) = prepared.into_parts();

    accept_or_publish_single_payload_artifact(
        cache,
        pending,
        &declared,
        |actual| validate_metal_payload_metadata(&expected_metadata, actual),
        || {
            CompiledMetalPayload::compile_prepared(prepared, compiled_metadata)
                .map(CompiledMetalPayload::into_content)
        },
        |content| assemble(CompiledMetalPayload::from_content(content)),
    )
    .map_err(MetalCacheError::from)
}
