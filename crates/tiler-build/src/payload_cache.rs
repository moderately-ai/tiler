//! The backend-neutral seam between one pending artifact and the expansion cache.
//!
//! This is the second half of the build-time orchestration boundary
//! [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 opened. [`crate::assemble_plan_artifact`] promoted *assembly*; this
//! module promotes cache-subject composition, miss-only external compilation,
//! and payload correspondence validation. It names no backend, and this crate's
//! Metal path is one caller of it rather than its owner.
//!
//! # One artifact, one payload per delivery position
//!
//! A **delivery position** is the ordered slot a consumer's build target
//! resolves to, and one selection produces one envelope carrying one payload per
//! built family. So a caller declares a *run* of payloads in delivery order,
//! compiles one object for each on a miss, and the composed cache subject names
//! every one of their compilations beside the artifact program's own identity.
//! The subject therefore covers the whole selection: dropping a family changes
//! the subject and the artifact identity together rather than silently reusing
//! an entry that carries more objects than the consumer asked to build.
//!
//! **Delivery order is not the payload table's order.** An artifact's descriptor
//! table is canonically ordered by content, so position `p` of a declaration run
//! and position `p` of `DecodedArtifact::payloads` are unrelated. Every check
//! below therefore resolves a delivery position through the artifact's own
//! entries — [`DecodedEntry::payload`] — which is the only authority that says
//! which object a consumer at position `p` would load. Comparing the tables
//! positionally would have compared a declaration against whichever descriptor
//! sorted there, and would have passed for two payloads swapped between two
//! positions, which is exactly the defect this seam exists to refuse.
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
//! exactly the declared payloads, one per delivery position, each the one the
//! backend declared at that position, and each with the digest the cache key
//! will name. The portfolio and digest halves are structural; "is this the
//! payload I declared" is the backend's.
//!
//! **Point 2, inside the miss closure, after encoding and before publication.**
//! The produced artifact must carry the declared payloads, metadata that
//! corresponds to the compilation just performed at each position, the same
//! descriptors the pending artifact declared, and the exact object bytes the
//! compile step produced for each — and its canonical identity must already have
//! agreed with the pending one. Only the correspondence half is the backend's.
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
//! facade invokes at exactly the points where a decoded payload exists. It takes
//! the delivery position beside the metadata, because with several objects in
//! flight "which compilation was this supposed to be" is the question a backend
//! is being asked. One closure, not a trait, for the reason item 11 gives: a
//! closure parameter already abstracts this edge and a trait would only
//! re-mediate it.
//!
//! # What this seam is bounded to
//!
//! One payload per delivery position, shared by every executable entry, which
//! [`DeliveredPayloadProtocolError::DeliveryRealization`] states rather than
//! assumes. An artifact whose entries are realized by *different* objects at one
//! position is expressible in the artifact model and is deliberately not
//! orchestrated here.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, BackendKey, BackendPayloadDescriptor,
    DecodedArtifact, DecodedVariant, Digest, DigestAlgorithm, PayloadContent, PayloadDigest,
    PayloadMetadata, RepresentationKey, SchemaVersion, VariantRef, VerifiedArtifactProgram,
    decode_artifact,
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

/// What a backend declares about one payload its artifact carries.
///
/// A caller-constructed leaf record with public fields, in the convention this
/// crate's callers already write. Every field is a statement the cache cannot
/// derive and the plan does not make; a fact either party *does* hold — the
/// payload's compatibility profile, the artifact's canonical identity — is
/// absent by design and read from the pending artifact instead.
///
/// One of these per delivery position. The position is the record's index in the
/// run a caller supplies rather than a field, so a declaration cannot claim a
/// position the run does not put it at.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DeclaredPayload<'facts> {
    /// Governed backend family this position's payload must declare.
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
/// Every position-scoped variant names its **delivery position**, not a
/// descriptor-table index. With one object per consumer build target, "which one
/// disagreed" is the first thing a producer needs, and a descriptor position
/// would name a canonical content slot the producer never chose.
///
/// `M` is the backend's own correspondence vocabulary — the naming half this
/// seam delegates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DeliveredPayloadProtocolError<M> {
    /// The artifact does not carry exactly the declared payloads.
    PayloadPortfolio {
        /// Number of payloads declared, one per delivery position.
        expected: usize,
        /// Number of payload descriptors the artifact carries.
        actual: usize,
    },
    /// The artifact declares a different number of delivery positions.
    ///
    /// Distinct from [`Self::PayloadPortfolio`] because the two are different
    /// subjects: an artifact may carry the right number of *objects* and realize
    /// its entries at the wrong number of positions, which is a consumer facing
    /// no payload for the target it built for.
    DeliveryPositions {
        /// Number of delivery positions declared.
        expected: usize,
        /// Number the artifact realizes its entries at.
        actual: usize,
    },
    /// Two executable entries name different payloads at one delivery position.
    ///
    /// This seam orchestrates one object per position, shared by every entry;
    /// the artifact model admits more, so the bound is stated rather than
    /// assumed.
    ///
    /// Unreachable through [`crate::assemble_plan_artifact`], which hands every
    /// entry the one delivery-ordered run a backend declared, so no artifact
    /// this crate assembles can contradict it. It is retained for the reason
    /// [`Self::MissingPayloadObject`] is: the artifact model is a public
    /// boundary an out-of-crate producer may reach directly, and a seam that
    /// took the first entry's answer as the position's would be reading a
    /// property of whichever entry it looked at.
    DeliveryRealization {
        /// The delivery position the entries disagreed about.
        delivery: usize,
    },
    /// One position's payload is not the declared backend, representation, schema, and policy.
    PayloadDescriptor {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compilation metadata.
    MissingPayloadMetadata {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The backend refused a carried metadata against its own compilation.
    Correspondence {
        /// The delivery position that disagreed.
        delivery: usize,
        /// The backend's own naming of the disagreement.
        cause: M,
    },
    /// One position's payload names a different complete compilation subject.
    PayloadSubject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position's carried payload omits its compiled object.
    ///
    /// Unreachable through today's artifact codec, which frames a payload's
    /// metadata and its object under one presence flag, so
    /// [`Self::MissingPayloadMetadata`] fires first for every artifact that
    /// carries neither. It is retained because it guards an accessor contract
    /// this crate does not own: `DecodedArtifact::payload_object` answers
    /// `Option`, and reading through that with an assumption rather than a
    /// refusal is what would have to be rewritten if the framing ever separated.
    MissingPayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// One position carries object bytes other than the exact miss-produced object.
    PayloadObject {
        /// The delivery position that disagreed.
        delivery: usize,
    },
    /// The compile step produced a different number of objects than declared.
    ///
    /// A defect in the caller rather than in any artifact: it promised one
    /// compilation per delivery position and delivered another count, and
    /// zipping the two would have silently assembled the shorter run.
    CompiledPortfolio {
        /// Number of compilations declared.
        expected: usize,
        /// Number the compile step produced.
        actual: usize,
    },
    /// The artifact's identity differs from the pending artifact used in the key.
    ArtifactIdentity,
}

impl<M: fmt::Display> fmt::Display for DeliveredPayloadProtocolError<M> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadPortfolio { expected, actual } => write!(
                formatter,
                "delivered-payload cache orchestration requires exactly {expected} payload(s), \
                 found {actual}",
            ),
            Self::DeliveryPositions { expected, actual } => write!(
                formatter,
                "{expected} payload(s) were declared and the artifact realizes its entries at \
                 {actual} delivery position(s)",
            ),
            Self::DeliveryRealization { delivery } => write!(
                formatter,
                "two executable entries name different payloads at delivery position {delivery}",
            ),
            Self::PayloadDescriptor { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} is not the declared payload descriptor",
            ),
            Self::MissingPayloadMetadata { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} carries no compilation metadata",
            ),
            Self::Correspondence { delivery, cause } => {
                write!(formatter, "at delivery position {delivery}: {cause}")
            }
            Self::PayloadSubject { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} names a different complete \
                 compilation subject",
            ),
            Self::MissingPayloadObject { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} carries no compiled object",
            ),
            Self::PayloadObject { delivery } => write!(
                formatter,
                "the payload at delivery position {delivery} carries a different compiled object",
            ),
            Self::CompiledPortfolio { expected, actual } => write!(
                formatter,
                "{expected} compilation(s) were declared and the compile step produced {actual}",
            ),
            Self::ArtifactIdentity => formatter.write_str(
                "produced artifact identity differs from the pending artifact cache subject",
            ),
        }
    }
}

impl<M: Error + 'static> Error for DeliveredPayloadProtocolError<M> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Correspondence { cause, .. } => Some(cause),
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
pub enum DeliveredPayloadCacheError<M, C, A> {
    /// The complete cache subject could not be composed.
    Subject(SubjectRefusal),
    /// The backend's external compilation failed on a cache miss.
    Compile(C),
    /// The caller could not assemble the compiled payloads into its artifact.
    Assemble(A),
    /// The caller's verified artifact could not be encoded.
    Encode(ArtifactCodecFailure),
    /// The cache's governed artifact validator rejected the produced envelope.
    CacheArtifact(ArtifactCodecFailure),
    /// The pending, produced, or cached artifact contradicted its declaration.
    Protocol(DeliveredPayloadProtocolError<M>),
}

impl<M: fmt::Display, C: fmt::Display, A: fmt::Display> fmt::Display
    for DeliveredPayloadCacheError<M, C, A>
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
    for DeliveredPayloadCacheError<M, C, A>
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
    Protocol(DeliveredPayloadProtocolError<M>),
}

/// Resolves one artifact's delivery-ordered payload run through the expansion cache.
///
/// `pending` is the verified descriptor-only artifact whose canonical identity
/// is available before compilation, and `declared` is what its payloads must say
/// at each delivery position, in delivery order. `compile` runs only on a cache
/// miss and performs whatever external work the backend's representation
/// requires, returning one compiled object per declaration in the same order;
/// `assemble` then carries them into the corresponding artifact.
/// `correspondence` is the backend's own fact-level check over one decoded
/// payload's metadata at a named position, invoked before publication and again
/// on every result.
///
/// # The order of the checks is part of the contract
///
/// Within one validation pass the refusals fire in the declaration order of
/// [`DeliveredPayloadProtocolError`], and within that, in delivery order. In
/// particular `correspondence` runs *before* the descriptor comparison that
/// subsumes it, so a backend that names its facts keeps that naming; a backend
/// supplying `|_, _| Ok(())` loses only the finer diagnostic and no accept/reject
/// decision, because the payload digest is derived from the canonical metadata
/// bytes.
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
pub fn accept_or_publish_delivered_payload_artifact<M, C, A>(
    cache: &ExpansionCache,
    pending: &VerifiedArtifactProgram,
    declared: &[DeclaredPayload<'_>],
    correspondence: impl Fn(usize, &PayloadMetadata) -> Result<(), M>,
    compile: impl FnOnce() -> Result<Vec<PayloadContent>, C>,
    assemble: impl FnOnce(Vec<PayloadContent>) -> Result<VerifiedArtifactProgram, A>,
) -> Result<AcceptedArtifact, DeliveredPayloadCacheError<M, C, A>> {
    let expected_descriptors = validate_pending_payloads(pending, declared)
        .map_err(DeliveredPayloadCacheError::Protocol)?;
    let expected_artifact = pending.canonical_identity().clone();
    // Delivery order, so the subject names the whole selection in the order the
    // producer built it: two families swapped are two subjects, not one.
    let compilations: Vec<&[u8]> = declared.iter().map(|payload| payload.compilation).collect();
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &compilations,
        artifact_program: expected_artifact.as_bytes(),
    })
    .map_err(DeliveredPayloadCacheError::Subject)?;

    let resolution = cache
        .get_or_publish(&subject, || {
            let contents = compile().map_err(PublicationError::Compile)?;
            if contents.len() != declared.len() {
                return Err(PublicationError::Protocol(
                    DeliveredPayloadProtocolError::CompiledPortfolio {
                        expected: declared.len(),
                        actual: contents.len(),
                    },
                ));
            }
            let expected_objects: Vec<Digest> = contents
                .iter()
                .map(|content| object_digest(&content.code))
                .collect();
            let artifact = assemble(contents).map_err(PublicationError::Assemble)?;
            if artifact.canonical_identity() != &expected_artifact {
                return Err(PublicationError::Protocol(
                    DeliveredPayloadProtocolError::ArtifactIdentity,
                ));
            }
            let envelope = artifact.encode().map_err(PublicationError::Encode)?;
            let decoded = decode_artifact(&envelope).map_err(PublicationError::Encode)?;
            validate_decoded_payloads(
                &decoded,
                declared,
                &expected_descriptors,
                &correspondence,
                Some(&expected_objects),
            )
            .map_err(PublicationError::Protocol)?;
            Ok(envelope)
        })
        .map_err(map_publish_failure)?;

    let decoded = resolution_artifact(&resolution);
    validate_decoded_payloads(
        decoded,
        declared,
        &expected_descriptors,
        &correspondence,
        None,
    )
    .map_err(DeliveredPayloadCacheError::Protocol)?;
    if decoded.identity().as_bytes() != expected_artifact.as_bytes() {
        return Err(DeliveredPayloadCacheError::Protocol(
            DeliveredPayloadProtocolError::ArtifactIdentity,
        ));
    }
    Ok(AcceptedArtifact {
        subject,
        resolution,
    })
}

fn validate_pending_payloads<M>(
    pending: &VerifiedArtifactProgram,
    declared: &[DeclaredPayload<'_>],
) -> Result<Vec<BackendPayloadDescriptor>, DeliveredPayloadProtocolError<M>> {
    if pending.payloads().len() != declared.len() {
        return Err(DeliveredPayloadProtocolError::PayloadPortfolio {
            expected: declared.len(),
            actual: pending.payloads().len(),
        });
    }
    if pending.delivery_positions() != declared.len() {
        return Err(DeliveredPayloadProtocolError::DeliveryPositions {
            expected: declared.len(),
            actual: pending.delivery_positions(),
        });
    }
    let mut expected = Vec::with_capacity(declared.len());
    for (delivery, payload) in declared.iter().enumerate() {
        // Resolved through the entries rather than read off the descriptor
        // table, because the table is canonically ordered by content and says
        // nothing about which object a consumer at this position would load.
        let descriptor = sole_realization(
            pending.variants().flat_map(VariantRef::entries),
            delivery,
            |entry| entry.payload(delivery),
        )?;
        validate_descriptor(descriptor, payload, delivery)?;
        if descriptor.digest != *payload.digest {
            return Err(DeliveredPayloadProtocolError::PayloadSubject { delivery });
        }
        expected.push(descriptor.clone());
    }
    Ok(expected)
}

fn validate_decoded_payloads<M>(
    artifact: &DecodedArtifact,
    declared: &[DeclaredPayload<'_>],
    expected_descriptors: &[BackendPayloadDescriptor],
    correspondence: &impl Fn(usize, &PayloadMetadata) -> Result<(), M>,
    expected_objects: Option<&[Digest]>,
) -> Result<(), DeliveredPayloadProtocolError<M>> {
    if artifact.payloads().len() != declared.len() {
        return Err(DeliveredPayloadProtocolError::PayloadPortfolio {
            expected: declared.len(),
            actual: artifact.payloads().len(),
        });
    }
    if artifact.delivery_positions() != declared.len() {
        return Err(DeliveredPayloadProtocolError::DeliveryPositions {
            expected: declared.len(),
            actual: artifact.delivery_positions(),
        });
    }
    for (delivery, payload) in declared.iter().enumerate() {
        let position = sole_realization(
            artifact.variants().flat_map(DecodedVariant::entries),
            delivery,
            |entry| entry.payload(delivery),
        )?;
        let descriptor = &artifact.payloads()[position];
        validate_descriptor(descriptor, payload, delivery)?;
        let metadata = artifact
            .payload_metadata(position)
            .ok_or(DeliveredPayloadProtocolError::MissingPayloadMetadata { delivery })?;
        correspondence(delivery, metadata)
            .map_err(|cause| DeliveredPayloadProtocolError::Correspondence { delivery, cause })?;
        if descriptor != &expected_descriptors[delivery] {
            return Err(DeliveredPayloadProtocolError::PayloadSubject { delivery });
        }
        let object = artifact
            .payload_object(position)
            .ok_or(DeliveredPayloadProtocolError::MissingPayloadObject { delivery })?;
        if expected_objects.is_some_and(|expected| object_digest(object) != expected[delivery]) {
            return Err(DeliveredPayloadProtocolError::PayloadObject { delivery });
        }
    }
    Ok(())
}

/// Reads the one thing every entry names at one delivery position.
///
/// Every entry is asked, not the first: the artifact model admits entries
/// realized by different objects at one position, and this seam orchestrates one
/// object per position. Requiring agreement is what makes "position `delivery`'s
/// payload" a fact rather than a property of whichever entry was looked at.
fn sole_realization<T, E, M>(
    entries: impl Iterator<Item = E>,
    delivery: usize,
    resolve: impl Fn(E) -> Option<T>,
) -> Result<T, DeliveredPayloadProtocolError<M>>
where
    T: Copy + Eq,
{
    let mut found: Option<T> = None;
    for entry in entries {
        let resolved = resolve(entry).ok_or(DeliveredPayloadProtocolError::DeliveryPositions {
            expected: delivery.saturating_add(1),
            actual: delivery,
        })?;
        match found {
            Some(held) if held != resolved => {
                return Err(DeliveredPayloadProtocolError::DeliveryRealization { delivery });
            }
            Some(_) => {}
            None => found = Some(resolved),
        }
    }
    found.ok_or(DeliveredPayloadProtocolError::DeliveryPositions {
        expected: delivery.saturating_add(1),
        actual: 0,
    })
}

fn validate_descriptor<M>(
    descriptor: &BackendPayloadDescriptor,
    declared: &DeclaredPayload<'_>,
    delivery: usize,
) -> Result<(), DeliveredPayloadProtocolError<M>> {
    if &descriptor.backend == declared.backend
        && &descriptor.representation == declared.representation
        && descriptor.payload_schema == declared.payload_schema
        && descriptor.execution_policy == declared.execution_policy
    {
        Ok(())
    } else {
        Err(DeliveredPayloadProtocolError::PayloadDescriptor { delivery })
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
) -> DeliveredPayloadCacheError<M, C, A> {
    match failure {
        PublishFailure::Build(PublicationError::Compile(error)) => {
            DeliveredPayloadCacheError::Compile(error)
        }
        PublishFailure::Build(PublicationError::Assemble(error)) => {
            DeliveredPayloadCacheError::Assemble(error)
        }
        PublishFailure::Build(PublicationError::Encode(error)) => {
            DeliveredPayloadCacheError::Encode(error)
        }
        PublishFailure::Build(PublicationError::Protocol(error)) => {
            DeliveredPayloadCacheError::Protocol(error)
        }
        PublishFailure::Artifact(error) => DeliveredPayloadCacheError::CacheArtifact(error),
    }
}
