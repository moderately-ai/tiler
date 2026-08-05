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
//! # Several artifact families, one envelope
//!
//! A selection naming several artifact families is one compilation, one plan,
//! one kernel program, and one compiled object per family — so this takes a
//! *run* of prepared payloads in delivery order and the neutral seam resolves
//! each delivery position through the artifact's own entries. The correspondence
//! closure is what makes a wrong-position payload a build error rather than a
//! wrong artifact: position `p`'s decoded metadata is compared against the
//! compilation prepared for position `p`, and two families whose objects were
//! placed the other way round disagree on
//! [`MetalPayloadFact::Target`](crate::MetalPayloadFact::Target) — the AOT
//! triple, which is the one fact that distinguishes them, since the ledger
//! records the artifact family as backend-only and the two share a byte-identical
//! compiler profile descriptor.
//!
//! The refusal vocabulary below is Metal's own and is preserved exactly: the
//! neutral seam's protocol refusals map one-to-one onto it, so a caller reading
//! [`MetalArtifactProtocolError`] sees the same kinds in the same order.

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
    AcceptedArtifact, CompiledPayloads, DeclaredPayload, DeliveredPayloadCacheError,
    DeliveredPayloadProtocolError, accept_or_publish_delivered_payload_artifact,
};

/// A decoded or assembled artifact contradicts the prepared Metal subjects.
///
/// Every position-scoped variant names the **delivery position** it is about —
/// the ordered slot a consumer's build target resolves to — because with one
/// object per artifact family "which one disagreed" is the first thing a
/// producer needs, and a descriptor-table index would name a canonical content
/// slot the producer never chose.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalArtifactProtocolError {
    /// The artifact does not carry exactly the backend compilations in the cache subject.
    PayloadPortfolio {
        /// Number of payloads the cache subject names.
        expected: usize,
        /// Number of payload descriptors the artifact carries.
        actual: usize,
    },
    /// The artifact realizes its entries at a different number of delivery positions.
    DeliveryPositions {
        /// Number of delivery positions prepared.
        expected: usize,
        /// Number the artifact realizes its entries at.
        actual: usize,
    },
    /// Two executable entries name different payloads at one delivery position.
    DeliveryRealization {
        /// The delivery position the entries disagreed about.
        delivery: usize,
    },
    /// One position's payload is not the governed Metal native-image descriptor.
    PayloadDescriptor {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compilation metadata.
    MissingPayloadMetadata {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compiled object.
    MissingPayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The carried metadata contradicts the compilation prepared for that position.
    ///
    /// This is the refusal a wrong-position payload arrives as: two artifact
    /// families share a compiler profile and differ in their AOT triple, so the
    /// object built for the other family disagrees on
    /// [`MetalPayloadFact::Target`](crate::MetalPayloadFact::Target).
    Correspondence {
        /// The delivery position that disagreed.
        delivery: usize,
        /// The exact compilation fact that disagreed.
        mismatch: MetalPayloadMismatch,
    },
    /// One position's payload names a different complete compilation subject.
    PayloadSubject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The compile step produced a different number of objects than were prepared.
    CompiledPortfolio {
        /// Number of compilations prepared.
        expected: usize,
        /// Number the compile step produced.
        actual: usize,
    },
    /// The compiled artifact differs from the pending artifact program used in the key.
    ArtifactIdentity,
    /// One position carries object bytes other than the exact miss-produced object.
    PayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
}

impl fmt::Display for MetalArtifactProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadPortfolio { expected, actual } => write!(
                formatter,
                "Metal cache orchestration requires exactly {expected} payload(s), found {actual}",
            ),
            Self::DeliveryPositions { expected, actual } => write!(
                formatter,
                "{expected} Metal compilation(s) were prepared and the artifact realizes its \
                 entries at {actual} delivery position(s)",
            ),
            Self::DeliveryRealization { delivery } => write!(
                formatter,
                "two executable entries name different Metal payloads at delivery position \
                 {delivery}",
            ),
            Self::PayloadDescriptor { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} is not the governed Metal metallib \
                 native-image descriptor",
            ),
            Self::MissingPayloadMetadata { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries no compilation metadata",
            ),
            Self::MissingPayloadObject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries no compiled object",
            ),
            Self::Correspondence { delivery, mismatch } => {
                write!(formatter, "at delivery position {delivery}: {mismatch}")
            }
            Self::PayloadSubject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} names a different complete \
                 compilation subject",
            ),
            Self::CompiledPortfolio { expected, actual } => write!(
                formatter,
                "{expected} Metal compilation(s) were prepared and the compile step produced \
                 {actual}",
            ),
            Self::ArtifactIdentity => formatter.write_str(
                "compiled artifact identity differs from the pending artifact cache subject",
            ),
            Self::PayloadObject { delivery } => write!(
                formatter,
                "the Metal payload at delivery position {delivery} carries a different compiled \
                 object",
            ),
        }
    }
}

impl Error for MetalArtifactProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correspondence { mismatch, .. } => Some(mismatch),
            _ => None,
        }
    }
}

impl From<DeliveredPayloadProtocolError<MetalPayloadMismatch>> for MetalArtifactProtocolError {
    /// Renames the neutral protocol refusals into Metal's own vocabulary.
    ///
    /// Exhaustive by arm rather than by wildcard, so a refusal added to the
    /// neutral seam is a build error here instead of a Metal diagnostic that
    /// silently loses a case.
    fn from(error: DeliveredPayloadProtocolError<MetalPayloadMismatch>) -> Self {
        match error {
            DeliveredPayloadProtocolError::PayloadPortfolio { expected, actual } => {
                Self::PayloadPortfolio { expected, actual }
            }
            DeliveredPayloadProtocolError::DeliveryPositions { expected, actual } => {
                Self::DeliveryPositions { expected, actual }
            }
            DeliveredPayloadProtocolError::DeliveryRealization { delivery } => {
                Self::DeliveryRealization { delivery }
            }
            DeliveredPayloadProtocolError::PayloadDescriptor { delivery } => {
                Self::PayloadDescriptor { delivery }
            }
            DeliveredPayloadProtocolError::MissingPayloadMetadata { delivery } => {
                Self::MissingPayloadMetadata { delivery }
            }
            DeliveredPayloadProtocolError::Correspondence { delivery, cause } => {
                Self::Correspondence {
                    delivery,
                    mismatch: cause,
                }
            }
            DeliveredPayloadProtocolError::PayloadSubject { delivery } => {
                Self::PayloadSubject { delivery }
            }
            DeliveredPayloadProtocolError::MissingPayloadObject { delivery } => {
                Self::MissingPayloadObject { delivery }
            }
            DeliveredPayloadProtocolError::PayloadObject { delivery } => {
                Self::PayloadObject { delivery }
            }
            DeliveredPayloadProtocolError::CompiledPortfolio { expected, actual } => {
                Self::CompiledPortfolio { expected, actual }
            }
            DeliveredPayloadProtocolError::ArtifactIdentity => Self::ArtifactIdentity,
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

impl<E> From<DeliveredPayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>>
    for MetalCacheError<E>
{
    fn from(
        error: DeliveredPayloadCacheError<MetalPayloadMismatch, MetalAssemblyError, E>,
    ) -> Self {
        match error {
            DeliveredPayloadCacheError::Subject(error) => Self::Subject(error),
            DeliveredPayloadCacheError::Compile(error) => Self::Compile(error),
            DeliveredPayloadCacheError::Assemble(error) => Self::Assemble(error),
            DeliveredPayloadCacheError::Encode(error) => Self::Encode(error),
            DeliveredPayloadCacheError::CacheArtifact(error) => Self::CacheArtifact(error),
            DeliveredPayloadCacheError::Protocol(error) => Self::Protocol(error.into()),
        }
    }
}

/// Resolves one plan's delivery-ordered Metal payload run through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation. `prepared` is one prepared compilation per
/// delivery position, in the order the producer built its artifact families;
/// `assemble` runs only on a cache miss and must assemble the supplied compiled
/// payloads — in that same order — into the corresponding carried artifact. The
/// carried identity and payloads are checked before publication; every cache
/// result is checked again before this function returns.
///
/// This binds [`accept_or_publish_delivered_payload_artifact`] to Metal's two
/// statements and nothing else: each prepared payload already holds its governed
/// descriptor keys and its derived compilation digest, and this crate's own
/// `validate_metal_payload_metadata` is the correspondence closure, applied to
/// the compilation prepared for *that* position. The compilation facet the cache
/// subject names is the AOT driver's own prepared identity, which has no public
/// constructor, so it cannot be minted from invented toolchain facts.
///
/// # Errors
///
/// Returns a typed subject, compilation, assembly, codec, or protocol failure.
/// A protocol failure is hard: it is never translated into a cache miss or an
/// automatic rebuild.
pub fn accept_or_publish_delivered_metal_artifact<E>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    prepared: Vec<PreparedMetalPayload<'_>>,
    assemble: impl FnOnce(Vec<CompiledMetalPayload>) -> Result<VerifiedArtifactProgram, E>,
) -> Result<AcceptedArtifact, MetalCacheError<E>> {
    // Owned before the tokens are consumed: the compile closure consumes each
    // token, and the same declarations are compared again after resolution.
    let backends: Vec<_> = prepared
        .iter()
        .map(|payload| payload.backend().clone())
        .collect();
    let representations: Vec<_> = prepared
        .iter()
        .map(|payload| payload.representation().clone())
        .collect();
    let digests: Vec<_> = prepared
        .iter()
        .map(|payload| payload.digest().clone())
        .collect();
    let expected_metadata: Vec<_> = prepared
        .iter()
        .map(|payload| payload.metadata().clone())
        .collect();
    let compilations: Vec<Vec<u8>> = prepared
        .iter()
        .map(|payload| payload.compilation_identity_bytes().to_vec())
        .collect();
    let declared: Vec<DeclaredPayload<'_>> = (0..prepared.len())
        .map(|delivery| DeclaredPayload {
            backend: &backends[delivery],
            representation: &representations[delivery],
            payload_schema: PAYLOAD_SCHEMA,
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            digest: &digests[delivery],
            compilation: &compilations[delivery],
        })
        .collect();
    let tokens: Vec<_> = prepared
        .into_iter()
        .map(PreparedMetalPayload::into_parts)
        .collect();

    accept_or_publish_delivered_payload_artifact(
        cache,
        pending,
        &declared,
        |delivery, actual| validate_metal_payload_metadata(&expected_metadata[delivery], actual),
        || {
            // Retains nothing, and that is a fact about the driver rather than a
            // decision here. `tiler_metal_aot::driver::Toolchain::run_stage`
            // keeps a stage's captured output only when the stage *fails*, in
            // `DriverError::ToolFailure`, and drops both streams on success — so
            // a succeeding Metal compilation has no diagnostics for this backend
            // to state. `retain-succeeding-metal-stage-tool-output` owns making
            // them reachable; until it lands, a retention here would be an empty
            // section claiming a capability nothing can fill.
            tokens
                .into_iter()
                .map(|(prepared, metadata, _digest)| {
                    CompiledMetalPayload::compile_prepared(prepared, metadata)
                        .map(CompiledMetalPayload::into_content)
                })
                .collect::<Result<Vec<_>, _>>()
                .map(CompiledPayloads::from)
        },
        |contents| {
            assemble(
                contents
                    .into_iter()
                    .map(CompiledMetalPayload::from_content)
                    .collect(),
            )
        },
    )
    .map_err(MetalCacheError::from)
}
