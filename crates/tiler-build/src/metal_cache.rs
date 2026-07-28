//! Expansion-cache orchestration for one prepared Metal artifact.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, BackendPayloadDescriptor, DecodedArtifact,
    Digest, DigestAlgorithm, PayloadMetadata, VerifiedArtifactProgram, decode_artifact,
};
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, Resolution, SubjectFacets, SubjectRefusal,
};

use crate::MetalPayloadMismatch;
use crate::metal_assembly::{
    BACKEND, CompiledMetalPayload, MetalAssemblyError, PAYLOAD_SCHEMA, PreparedMetalPayload,
    REPRESENTATION,
};
use crate::metal_payload::validate_metal_payload_metadata;

const OBJECT_VALIDATION_DOMAIN: &[u8] = b"tiler.build.metal-object-validation.v1\0";

/// One cache resolution whose artifact was accepted against the current Metal preparation.
#[derive(Debug)]
pub struct AcceptedMetalArtifact {
    subject: ComposedSubject,
    resolution: Resolution,
}

impl AcceptedMetalArtifact {
    /// Returns the cache resolution, including its validated artifact and report.
    #[must_use]
    pub const fn resolution(&self) -> &Resolution {
        &self.resolution
    }

    /// Returns the exact composed subject this accepted artifact resolved under.
    #[must_use]
    pub const fn cache_subject(&self) -> &ComposedSubject {
        &self.subject
    }

    /// Consumes the acceptance proof and returns the underlying cache resolution.
    #[must_use]
    pub fn into_resolution(self) -> Resolution {
        self.resolution
    }
}

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

enum PublicationError<E> {
    Compile(MetalAssemblyError),
    Assemble(E),
    Encode(ArtifactCodecFailure),
    Protocol(MetalArtifactProtocolError),
}

/// Resolves one singular prepared Metal artifact through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation. `assemble` runs only on a cache miss and
/// must assemble the supplied compiled payload into the corresponding carried
/// artifact. The carried identity and payload are checked before publication;
/// every cache result is checked again before this function returns.
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
) -> Result<AcceptedMetalArtifact, MetalCacheError<E>> {
    let expected_descriptor =
        validate_pending_payload(pending, prepared.digest()).map_err(MetalCacheError::Protocol)?;
    let expected_artifact = pending.canonical_identity().clone();
    let compilation = [prepared.compilation_identity_bytes()];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &compilation,
        artifact_program: expected_artifact.as_bytes(),
    })
    .map_err(MetalCacheError::Subject)?;
    let (prepared, expected_metadata, _expected_digest) = prepared.into_parts();

    let resolution = cache
        .get_or_publish(&subject, || {
            let compiled =
                CompiledMetalPayload::compile_prepared(prepared, expected_metadata.clone())
                    .map_err(PublicationError::Compile)?;
            let expected_object = object_digest(&compiled.content().code);
            let artifact = assemble(compiled).map_err(PublicationError::Assemble)?;
            if artifact.canonical_identity() != &expected_artifact {
                return Err(PublicationError::Protocol(
                    MetalArtifactProtocolError::ArtifactIdentity,
                ));
            }
            let envelope = artifact.encode().map_err(PublicationError::Encode)?;
            // Deliberately before `get_or_publish` performs its governed decode:
            // the cache can validate an envelope but cannot prove this
            // backend-specific correspondence. Deferring this check until the
            // returned resolution would publish first and diagnose afterward.
            let decoded = decode_artifact(&envelope).map_err(PublicationError::Encode)?;
            validate_decoded_payload(
                &decoded,
                &expected_descriptor,
                &expected_metadata,
                Some(&expected_object),
            )
            .map_err(PublicationError::Protocol)?;
            Ok(envelope)
        })
        .map_err(map_publish_failure)?;

    let decoded = resolution_artifact(&resolution);
    validate_decoded_payload(decoded, &expected_descriptor, &expected_metadata, None)
        .map_err(MetalCacheError::Protocol)?;
    if decoded.identity().as_bytes() != expected_artifact.as_bytes() {
        return Err(MetalCacheError::Protocol(
            MetalArtifactProtocolError::ArtifactIdentity,
        ));
    }
    Ok(AcceptedMetalArtifact {
        subject,
        resolution,
    })
}

fn validate_pending_payload(
    pending: &VerifiedArtifactProgram,
    expected_digest: &tiler_artifact::program::PayloadDigest,
) -> Result<BackendPayloadDescriptor, MetalArtifactProtocolError> {
    let [descriptor] = pending.payloads() else {
        return Err(MetalArtifactProtocolError::PayloadPortfolio {
            actual: pending.payloads().len(),
        });
    };
    validate_descriptor(descriptor)?;
    if descriptor.digest != *expected_digest {
        return Err(MetalArtifactProtocolError::PayloadSubject);
    }
    Ok(descriptor.clone())
}

fn validate_decoded_payload(
    artifact: &DecodedArtifact,
    expected_descriptor: &BackendPayloadDescriptor,
    expected_metadata: &PayloadMetadata,
    expected_object: Option<&Digest>,
) -> Result<(), MetalArtifactProtocolError> {
    let [descriptor] = artifact.payloads() else {
        return Err(MetalArtifactProtocolError::PayloadPortfolio {
            actual: artifact.payloads().len(),
        });
    };
    validate_descriptor(descriptor)?;
    let metadata = artifact
        .payload_metadata(0)
        .ok_or(MetalArtifactProtocolError::MissingPayloadMetadata)?;
    validate_metal_payload_metadata(expected_metadata, metadata)
        .map_err(MetalArtifactProtocolError::Correspondence)?;
    if descriptor != expected_descriptor {
        return Err(MetalArtifactProtocolError::PayloadSubject);
    }
    let object = artifact
        .payload_object(0)
        .ok_or(MetalArtifactProtocolError::MissingPayloadObject)?;
    if expected_object.is_some_and(|expected| object_digest(object) != *expected) {
        return Err(MetalArtifactProtocolError::PayloadObject);
    }
    Ok(())
}

fn validate_descriptor(
    descriptor: &BackendPayloadDescriptor,
) -> Result<(), MetalArtifactProtocolError> {
    if descriptor.backend.as_str() == BACKEND
        && descriptor.representation.as_str() == REPRESENTATION
        && descriptor.payload_schema == PAYLOAD_SCHEMA
        && descriptor.execution_policy == ArtifactExecutionPolicy::NativeImage
    {
        Ok(())
    } else {
        Err(MetalArtifactProtocolError::PayloadDescriptor)
    }
}

fn object_digest(object: &[u8]) -> Digest {
    DigestAlgorithm::GOVERNED.digest(OBJECT_VALIDATION_DOMAIN, object)
}

const fn resolution_artifact(resolution: &Resolution) -> &DecodedArtifact {
    match resolution {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => entry.artifact(),
        Resolution::Uncached { artifact, .. } => artifact,
    }
}

fn map_publish_failure<E>(failure: PublishFailure<PublicationError<E>>) -> MetalCacheError<E> {
    match failure {
        PublishFailure::Build(PublicationError::Compile(error)) => MetalCacheError::Compile(error),
        PublishFailure::Build(PublicationError::Assemble(error)) => {
            MetalCacheError::Assemble(error)
        }
        PublishFailure::Build(PublicationError::Encode(error)) => MetalCacheError::Encode(error),
        PublishFailure::Build(PublicationError::Protocol(error)) => {
            MetalCacheError::Protocol(error)
        }
        PublishFailure::Artifact(error) => MetalCacheError::CacheArtifact(error),
    }
}
