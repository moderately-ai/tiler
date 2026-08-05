//! Metal's specialization of the neutral expansion-cache seam.
//!
//! Everything structural — subject composition, miss-only compilation, identity
//! agreement before publication, re-validation of every result — belongs to
//! [`crate::payload_cache`] and is shared with every other backend. What is
//! Metal's and stays here is three statements: the governed
//! `tiler.metal`/`metallib`/`NativeImage` payload descriptor this backend
//! declares, the fact-level correspondence between a carried payload's metadata
//! and the Apple compilation that was prepared for it, and what a succeeding
//! compilation retains beside its entry. The first is data and travels in a
//! [`DeclaredPayload`]; the second is a closure, because naming *which*
//! compilation fact disagreed is a judgement only this backend can make; the
//! third is [`stage_retention`], because only this backend knows that a Metal
//! compilation is two tools and which one wrote what.
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
use tiler_cache::expansion::{DebugRetention, ExpansionCache, RetentionRefusal, SubjectRefusal};
use tiler_metal_aot::diagnostic::CompileStage;
use tiler_metal_aot::record::StageOutputs;

use crate::MetalPayloadMismatch;
use crate::metal_assembly::{
    BACKEND, CompiledMetalPayload, MetalAssemblyError, PAYLOAD_SCHEMA, PreparedMetalPayload,
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
/// # What a miss retains beside the entry
///
/// Each position's `metal` and `metallib` runs are retained under their own
/// labels — see `stage_retention` in this module — so a later hit can be asked what the
/// compiler said about the object it is serving. None of it reaches the payload
/// metadata, the payload digest, the composed subject, or the cache key: all of
/// those are derived before either tool runs, from the prepared compilation and
/// the pending artifact. A build whose compiler warns therefore resolves to the
/// same entry as one whose compiler is silent.
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
            // Each compilation's object and its stage output are separated here:
            // the object is what the artifact carries and every check below
            // compares, while the output is retained beside the published entry
            // and reaches no identity at all.
            let mut contents = Vec::with_capacity(tokens.len());
            let mut outputs = Vec::with_capacity(tokens.len());
            for (prepared, metadata, _digest) in tokens {
                let (content, stage_outputs) =
                    CompiledMetalPayload::compile_prepared_parts(prepared, metadata)?;
                contents.push(content);
                outputs.push(stage_outputs);
            }
            Ok(CompiledPayloads {
                contents,
                retained: stage_retention(&outputs),
            })
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

/// Names one delivery position's stage run.
///
/// The backend key, the delivery position, and the stage's own tool name, in
/// that order: the run a reader wants is "what did `metallib` say about the
/// object my build target loads", and every part of that question is in the
/// label. The position is included because one entry covers the whole selection
/// — several artifact families are several compilations under one key — so a
/// label naming only the stage would be two runs fighting over one name, which
/// [`DebugRetention::retaining`] refuses rather than silently merges.
fn stage_label(delivery: usize, stage: CompileStage) -> String {
    format!("{BACKEND}.{delivery}.{}", stage.tool())
}

/// States what every stage of every position in this selection wrote.
///
/// **Always stated, never discovered.** This backend retains its stage output on
/// every publication rather than consulting an environment variable or a build
/// profile, which is the ADR 0089 root policy the retention module restates: the
/// decision lives with the caller that has one, and this caller's decision is
/// that a Metal compilation's own words belong beside the entry it produced. The
/// cost is bounded by the retention's own limits and is two empty runs for a
/// quiet compilation.
///
/// **A silent stage is retained as an empty run.** Both stages ran, so both are
/// named; dropping the quiet one would leave a reader unable to tell a compiler
/// that warned about nothing from an entry published before any of this existed,
/// which is the state [`DebugRetention::is_empty`] already answers.
///
/// **The text is host-specific, which is the second reason it is not identity.**
/// A `metal` diagnostic names the file it diagnosed, and the driver compiles from
/// a per-process scratch directory, so two hosts compiling byte-identical source
/// under one toolchain retain different bytes for one warning. They resolve to
/// one entry regardless, because the key is a function of the composed subject
/// alone; an implementation that folded this text into a subject would have given
/// them two.
///
/// # Where a truncation stops being visible
///
/// `tiler_metal_aot::diagnostic::ToolOutput` and
/// `tiler_cache::expansion::MAX_RETAINED_RUN_BYTES` bound one run identically, at
/// 16 KiB. A stage that wrote more therefore arrives here already truncated and
/// exactly at the bound, and the retention records its total as the length it was
/// handed — so `RetainedText::is_truncated` reads false on the entry while
/// `ToolOutput::is_truncated` read true at the run that produced it. The fact is
/// not lost where the compilation happened; it is not carried to a later hit,
/// because the retention API takes bytes and derives the total from them.
/// `carry-a-producer-stated-total-into-a-retained-run` owns closing that, and
/// doing it here instead would mean either a second bound or editing the tool's
/// own bytes to describe them.
///
/// # A refusal is not a build failure
///
/// A retention that cannot be stated — a selection wide enough to pass the
/// run-count limit is the reachable case — leaves the compilation entirely
/// correct, so it is recorded as one run saying so rather than returned as an
/// error. Failing a successful compilation over a diagnostic would make a warning
/// a compilation input in the only way that actually matters.
fn stage_retention(outputs: &[StageOutputs]) -> DebugRetention {
    let mut retention = DebugRetention::none();
    for (delivery, stage_outputs) in outputs.iter().enumerate() {
        for stage in CompileStage::ALL {
            match retention.retaining(
                &stage_label(delivery, stage),
                stage_outputs.stage(stage).as_bytes(),
            ) {
                Ok(extended) => retention = extended,
                // All or nothing: a partial run set reads as a selection with
                // fewer positions than it had, and a reader cannot tell which.
                Err(refusal) => return elided_retention(&refusal),
            }
        }
    }
    retention
}

/// Retains one run stating why no stage output is here.
///
/// A positive statement rather than an absent section, because absence already
/// means "published by a build that retained nothing" and a reader that could
/// not tell the two apart would go looking for a compiler that never spoke.
fn elided_retention(refusal: &RetentionRefusal) -> DebugRetention {
    DebugRetention::none()
        .retaining(
            &format!("{BACKEND}.retention-elided"),
            format!("no Metal stage output was retained: {refusal}").as_bytes(),
        )
        // One governed label and one run cannot exceed a bound, so this is the
        // unreachable arm of a total function rather than a case to handle: it
        // resolves to the same "nothing to show" a non-retaining build leaves.
        .unwrap_or_else(|_| DebugRetention::none())
}
