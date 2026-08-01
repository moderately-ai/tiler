//! The backend-neutral seam between one pending artifact and the expansion cache.
//!
//! This is the second half of the build-time orchestration boundary
//! [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 opened. [`crate::assemble_plan_artifact`] promoted *assembly*; this
//! module promotes cache-subject composition, miss-only external compilation,
//! and payload correspondence validation. It names no backend, and this crate's
//! Metal path is one caller of it rather than its owner.
//!
//! # Why the shape is a declaration plus one closure, and not the alternatives
//!
//! The obligations here interleave: three times in one call, a structural rule
//! about *the cache* and a payload-specific rule about *this backend* have to
//! run in a fixed order, and the order is observable because it decides which
//! refusal a producer is handed. Naming those points is what makes the shape
//! derivable rather than a matter of taste.
//!
//! **Point 1, before the subject exists.** The pending artifact must carry
//! exactly one payload, that payload must be the one the backend declared, and
//! its digest must be the compilation subject the cache key will name. The
//! portfolio and digest halves are structural; "is this the payload I declared"
//! is the backend's.
//!
//! **Point 2, inside the miss closure, after encoding and before publication.**
//! The produced artifact must carry the declared payload, metadata that
//! corresponds to the compilation just performed, the same descriptor the
//! pending artifact declared, and the exact object bytes the compile step
//! produced — and its canonical identity must already have agreed with the
//! pending one. Only the correspondence half is the backend's.
//!
//! **Point 3, after any resolution.** Every result — a hit, a publication, or an
//! uncached artifact — is re-validated by the same rules, minus the object
//! comparison, because a hit's object is whatever was published rather than
//! anything this call compiled.
//!
//! *A structural facade with one post-decode hook* does not survive point 1:
//! the pending artifact is not decoded, so a post-decode hook cannot express the
//! check at all, and at points 2 and 3 a single hook cannot be placed *between*
//! the facade's own metadata-presence and descriptor-equality steps, which is
//! exactly where the correspondence refusal has to fire. A hook that ran before
//! or after the structural block would reorder the refusals.
//!
//! *A declaration record plus one compile closure* does not survive points 2 and
//! 3. A record can say which payload is expected; it cannot say how a
//! disagreement is *named*. The only comparison a neutral record supports is
//! equality of the declared and carried metadata, which collapses a
//! fact-by-fact diagnostic into one undifferentiated refusal. That loses no
//! accept/reject decision — the descriptor digest is derived from the canonical
//! metadata bytes, so any metadata disagreement already moves the digest and is
//! caught one step later — but a refinement that only ever refines explanation
//! is still the whole reason the finer check exists.
//!
//! *A split into two functions*, composing the subject and then resolving under
//! it, does not survive at all. Both halves need the same operands: the subject
//! is composed from the pending artifact's canonical identity, and every
//! identity-agreement check compares against that same identity. Splitting them
//! produces a first function whose only output is redundant with an input the
//! second still requires, and a signature in which a subject composed from one
//! artifact can be handed to a publication of another — the exact protocol
//! violation the checks below exist to refuse.
//!
//! **What survives is the first shape, refined by the second.** The backend's
//! *descriptor* statement is data, because it is four governed values compared
//! for equality and a neutral comparison names the disagreement as well as any
//! backend could — so it lives in [`DeclaredPayload`]. The backend's
//! *correspondence* statement is behaviour, because only the backend knows which
//! facts its metadata asserts and what to call each one — so it is a closure the
//! facade invokes at exactly the two points where a decoded payload exists.
//! One closure, not a trait, for the reason item 11 gives: a closure parameter
//! already abstracts this edge and a trait would only re-mediate it.
//!
//! # What this seam is bounded to
//!
//! Exactly one payload per artifact, which [`SinglePayloadProtocolError::PayloadPortfolio`]
//! states rather than assumes. Ordered multi-payload orchestration is a broader
//! slice and is deliberately not inferred here.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, BackendKey, BackendPayloadDescriptor,
    DecodedArtifact, Digest, DigestAlgorithm, PayloadContent, PayloadDigest, PayloadMetadata,
    RepresentationKey, SchemaVersion, VerifiedArtifactProgram, decode_artifact,
};
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, Resolution, SubjectFacets, SubjectRefusal,
};

/// Domain the seam compares two object byte runs under.
///
/// Never published and never composed into any identity: it separates one
/// in-process comparison between the object a compile step produced and the
/// object the artifact it was assembled into carries.
const OBJECT_VALIDATION_DOMAIN: &[u8] = b"tiler.build.object-validation.v1\0";

/// What a backend declares about the single payload its artifact carries.
///
/// A caller-constructed leaf record with public fields, in the convention this
/// crate's callers already write. Every field is a statement the cache cannot
/// derive and the plan does not make; a fact either party *does* hold — the
/// payload's compatibility profile, the artifact's canonical identity — is
/// absent by design and read from the pending artifact instead.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredPayload<'facts> {
    /// Governed backend family the sole payload must declare.
    pub backend: &'facts BackendKey,
    /// Governed executable representation it must declare.
    pub representation: &'facts RepresentationKey,
    /// Schema version of the backend's own payload metadata.
    pub payload_schema: SchemaVersion,
    /// How the payload reaches an executable state.
    pub execution_policy: ArtifactExecutionPolicy,
    /// Digest of the compilation subject the payload's metadata identifies.
    ///
    /// Derived by the artifact layer from canonical metadata bytes — never
    /// stamped by a producer — so a pending artifact whose descriptor names
    /// another digest is filing under a compilation it did not perform.
    pub digest: &'facts PayloadDigest,
    /// Canonical bytes of the backend compilation the cache subject names.
    ///
    /// Opaque here and wrapped rather than parsed: the composed subject frames
    /// this run beside the artifact program's own canonical identity, and
    /// completeness *within* the run is the producing authority's obligation.
    pub compilation: &'facts [u8],
}

/// One cache resolution whose artifact was accepted against its declaration.
#[derive(Debug)]
pub struct AcceptedArtifact {
    subject: ComposedSubject,
    resolution: Resolution,
}

impl AcceptedArtifact {
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

    /// Returns the validated artifact this resolution carries.
    ///
    /// The three resolution kinds reach their artifact by different fields, and
    /// a consumer writing that match itself would be writing the one place a
    /// future kind could be missed. It is an accessor rather than a field so a
    /// hit and a publication stay distinguishable through
    /// [`Self::resolution`].
    #[must_use]
    pub const fn decoded(&self) -> &DecodedArtifact {
        resolution_artifact(&self.resolution)
    }

    /// Consumes the acceptance proof and returns the underlying cache resolution.
    #[must_use]
    pub fn into_resolution(self) -> Resolution {
        self.resolution
    }
}

/// A pending, produced, or cached artifact contradicts its payload declaration.
///
/// The variants are ordered as they can fire within one validation pass, which
/// is the contract: a producer reading two refusals in sequence is reading them
/// in this order, and a check moved past another changes what it is handed.
///
/// `M` is the backend's own correspondence vocabulary — the naming half this
/// seam delegates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SinglePayloadProtocolError<M> {
    /// The artifact does not carry exactly the one declared payload.
    PayloadPortfolio {
        /// Number of payload descriptors the artifact carries.
        actual: usize,
    },
    /// The sole payload is not the declared backend, representation, schema, and policy.
    PayloadDescriptor,
    /// The carried payload omits its compilation metadata.
    MissingPayloadMetadata,
    /// The backend refused the carried metadata against its own compilation.
    Correspondence(M),
    /// The payload's complete compilation subject differs from the pending declaration.
    PayloadSubject,
    /// The carried payload omits its compiled object.
    ///
    /// Unreachable through today's artifact codec, which frames a payload's
    /// metadata and its object under one presence flag, so
    /// [`Self::MissingPayloadMetadata`] fires first for every artifact that
    /// carries neither. It is retained because it guards an accessor contract
    /// this crate does not own: `DecodedArtifact::payload_object` answers
    /// `Option`, and reading through that with an assumption rather than a
    /// refusal is what would have to be rewritten if the framing ever separated.
    MissingPayloadObject,
    /// The artifact carries object bytes other than the exact miss-produced object.
    PayloadObject,
    /// The artifact's identity differs from the pending artifact used in the key.
    ArtifactIdentity,
}

impl<M: fmt::Display> fmt::Display for SinglePayloadProtocolError<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadPortfolio { actual } => write!(
                formatter,
                "single-payload cache orchestration requires exactly one payload, found {actual}",
            ),
            Self::PayloadDescriptor => {
                formatter.write_str("artifact payload is not the declared payload descriptor")
            }
            Self::MissingPayloadMetadata => {
                formatter.write_str("artifact payload carries no compilation metadata")
            }
            Self::Correspondence(error) => error.fmt(formatter),
            Self::PayloadSubject => formatter
                .write_str("artifact payload names a different complete compilation subject"),
            Self::MissingPayloadObject => {
                formatter.write_str("artifact payload carries no compiled object")
            }
            Self::PayloadObject => {
                formatter.write_str("artifact payload carries a different compiled object")
            }
            Self::ArtifactIdentity => formatter.write_str(
                "produced artifact identity differs from the pending artifact cache subject",
            ),
        }
    }
}

impl<M: Error + 'static> Error for SinglePayloadProtocolError<M> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correspondence(error) => Some(error),
            _ => None,
        }
    }
}

/// Why cache orchestration could not return an accepted artifact.
///
/// The three type parameters are three different authorities, kept apart
/// because they carry different remedies: `M` names a payload fact the backend
/// compared, `C` an external compilation that failed, and `A` an assembly the
/// caller could not perform. Collapsing `M` into `C` would make a protocol
/// defect that must never become a rebuild indistinguishable from a compiler
/// failure that legitimately can.
#[derive(Debug)]
#[non_exhaustive]
pub enum SinglePayloadCacheError<M, C, A> {
    /// The complete cache subject could not be composed.
    Subject(SubjectRefusal),
    /// The backend's external compilation failed on a cache miss.
    Compile(C),
    /// The caller could not assemble the compiled payload into its artifact.
    Assemble(A),
    /// The caller's verified artifact could not be encoded.
    Encode(ArtifactCodecFailure),
    /// The cache's governed artifact validator rejected the produced envelope.
    CacheArtifact(ArtifactCodecFailure),
    /// The pending, produced, or cached artifact contradicted its declaration.
    Protocol(SinglePayloadProtocolError<M>),
}

impl<M: fmt::Display, C: fmt::Display, A: fmt::Display> fmt::Display
    for SinglePayloadCacheError<M, C, A>
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Subject(error) => write!(formatter, "cache subject was refused: {error}"),
            Self::Compile(error) => write!(formatter, "payload compilation failed: {error}"),
            Self::Assemble(error) => write!(formatter, "artifact assembly failed: {error}"),
            Self::Encode(error) => write!(formatter, "artifact encoding failed: {error}"),
            Self::CacheArtifact(error) => write!(
                formatter,
                "expansion cache refused the generated artifact: {error}"
            ),
            Self::Protocol(error) => error.fmt(formatter),
        }
    }
}

impl<M: Error + 'static, C: Error + 'static, A: Error + 'static> Error
    for SinglePayloadCacheError<M, C, A>
{
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

enum PublicationError<M, C, A> {
    Compile(C),
    Assemble(A),
    Encode(ArtifactCodecFailure),
    Protocol(SinglePayloadProtocolError<M>),
}

/// Resolves one singular declared payload's artifact through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation, and `declared` is what its sole payload must
/// say. `compile` runs only on a cache miss and performs whatever external work
/// the backend's representation requires; `assemble` then carries the compiled
/// content into the corresponding artifact. `correspondence` is the backend's
/// own fact-level check over a decoded payload's metadata, invoked before
/// publication and again on every result.
///
/// # The order of the checks is part of the contract
///
/// Within one validation pass the refusals fire in the declaration order of
/// [`SinglePayloadProtocolError`]. In particular `correspondence` runs *before*
/// the descriptor comparison that subsumes it, so a backend that names its facts
/// keeps that naming; a backend supplying `|_| Ok(())` loses only the finer
/// diagnostic and no accept/reject decision, because the payload digest is
/// derived from the canonical metadata bytes.
///
/// # What runs before publication, and why not afterwards
///
/// The produced envelope is decoded and validated *inside* the miss closure,
/// before `get_or_publish` performs its own governed decode. The cache can
/// validate an envelope but cannot prove a backend's correspondence, and
/// deferring the check to the returned resolution would publish first and
/// diagnose afterwards.
///
/// # Errors
///
/// Returns a typed subject, compilation, assembly, codec, or protocol failure.
/// A protocol failure is hard: it is never translated into a cache miss or an
/// automatic rebuild, because rebuilding under the same contradicted
/// declaration would repeat the defect.
pub fn accept_or_publish_single_payload_artifact<M, C, A>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    declared: &DeclaredPayload<'_>,
    correspondence: impl Fn(&PayloadMetadata) -> Result<(), M>,
    compile: impl FnOnce() -> Result<PayloadContent, C>,
    assemble: impl FnOnce(PayloadContent) -> Result<VerifiedArtifactProgram, A>,
) -> Result<AcceptedArtifact, SinglePayloadCacheError<M, C, A>> {
    let expected_descriptor =
        validate_pending_payload(pending, declared).map_err(SinglePayloadCacheError::Protocol)?;
    let expected_artifact = pending.canonical_identity().clone();
    let compilation = [declared.compilation];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &compilation,
        artifact_program: expected_artifact.as_bytes(),
    })
    .map_err(SinglePayloadCacheError::Subject)?;

    let resolution = cache
        .get_or_publish(&subject, || {
            let content = compile().map_err(PublicationError::Compile)?;
            let expected_object = object_digest(&content.code);
            let artifact = assemble(content).map_err(PublicationError::Assemble)?;
            if artifact.canonical_identity() != &expected_artifact {
                return Err(PublicationError::Protocol(
                    SinglePayloadProtocolError::ArtifactIdentity,
                ));
            }
            let envelope = artifact.encode().map_err(PublicationError::Encode)?;
            let decoded = decode_artifact(&envelope).map_err(PublicationError::Encode)?;
            validate_decoded_payload(
                &decoded,
                declared,
                &expected_descriptor,
                &correspondence,
                Some(&expected_object),
            )
            .map_err(PublicationError::Protocol)?;
            Ok(envelope)
        })
        .map_err(map_publish_failure)?;

    let decoded = resolution_artifact(&resolution);
    validate_decoded_payload(
        decoded,
        declared,
        &expected_descriptor,
        &correspondence,
        None,
    )
    .map_err(SinglePayloadCacheError::Protocol)?;
    if decoded.identity().as_bytes() != expected_artifact.as_bytes() {
        return Err(SinglePayloadCacheError::Protocol(
            SinglePayloadProtocolError::ArtifactIdentity,
        ));
    }
    Ok(AcceptedArtifact {
        subject,
        resolution,
    })
}

fn validate_pending_payload<M>(
    pending: &VerifiedArtifactProgram,
    declared: &DeclaredPayload<'_>,
) -> Result<BackendPayloadDescriptor, SinglePayloadProtocolError<M>> {
    let [descriptor] = pending.payloads() else {
        return Err(SinglePayloadProtocolError::PayloadPortfolio {
            actual: pending.payloads().len(),
        });
    };
    validate_descriptor(descriptor, declared)?;
    if descriptor.digest != *declared.digest {
        return Err(SinglePayloadProtocolError::PayloadSubject);
    }
    Ok(descriptor.clone())
}

fn validate_decoded_payload<M>(
    artifact: &DecodedArtifact,
    declared: &DeclaredPayload<'_>,
    expected_descriptor: &BackendPayloadDescriptor,
    correspondence: &impl Fn(&PayloadMetadata) -> Result<(), M>,
    expected_object: Option<&Digest>,
) -> Result<(), SinglePayloadProtocolError<M>> {
    let [descriptor] = artifact.payloads() else {
        return Err(SinglePayloadProtocolError::PayloadPortfolio {
            actual: artifact.payloads().len(),
        });
    };
    validate_descriptor(descriptor, declared)?;
    let metadata = artifact
        .payload_metadata(0)
        .ok_or(SinglePayloadProtocolError::MissingPayloadMetadata)?;
    correspondence(metadata).map_err(SinglePayloadProtocolError::Correspondence)?;
    if descriptor != expected_descriptor {
        return Err(SinglePayloadProtocolError::PayloadSubject);
    }
    let object = artifact
        .payload_object(0)
        .ok_or(SinglePayloadProtocolError::MissingPayloadObject)?;
    if expected_object.is_some_and(|expected| object_digest(object) != *expected) {
        return Err(SinglePayloadProtocolError::PayloadObject);
    }
    Ok(())
}

fn validate_descriptor<M>(
    descriptor: &BackendPayloadDescriptor,
    declared: &DeclaredPayload<'_>,
) -> Result<(), SinglePayloadProtocolError<M>> {
    if &descriptor.backend == declared.backend
        && &descriptor.representation == declared.representation
        && descriptor.payload_schema == declared.payload_schema
        && descriptor.execution_policy == declared.execution_policy
    {
        Ok(())
    } else {
        Err(SinglePayloadProtocolError::PayloadDescriptor)
    }
}

fn object_digest(object: &[u8]) -> Digest {
    DigestAlgorithm::GOVERNED.digest(OBJECT_VALIDATION_DOMAIN, object)
}

/// Returns the validated artifact every resolution kind carries.
const fn resolution_artifact(resolution: &Resolution) -> &DecodedArtifact {
    match resolution {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => entry.artifact(),
        Resolution::Uncached { artifact, .. } => artifact,
    }
}

fn map_publish_failure<M, C, A>(
    failure: PublishFailure<PublicationError<M, C, A>>,
) -> SinglePayloadCacheError<M, C, A> {
    match failure {
        PublishFailure::Build(PublicationError::Compile(error)) => {
            SinglePayloadCacheError::Compile(error)
        }
        PublishFailure::Build(PublicationError::Assemble(error)) => {
            SinglePayloadCacheError::Assemble(error)
        }
        PublishFailure::Build(PublicationError::Encode(error)) => {
            SinglePayloadCacheError::Encode(error)
        }
        PublishFailure::Build(PublicationError::Protocol(error)) => {
            SinglePayloadCacheError::Protocol(error)
        }
        PublishFailure::Artifact(error) => SinglePayloadCacheError::CacheArtifact(error),
    }
}
