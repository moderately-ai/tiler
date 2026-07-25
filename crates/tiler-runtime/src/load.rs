//! Decoding artifact bytes into a validated, device-free program record.
//!
//! # The three stages, and why they are three types
//!
//! [`DecodedProgram::decode`] takes bytes and returns a fully validated read
//! view, or a typed rejection naming the class of failure.
//! [`DecodedProgram::preflight`] takes a host's stated
//! [`ExecutionEnvironment`] and the identity of the program the caller expects,
//! discharges every remaining obligation this loader can decide, and returns a
//! [`Preflight`]. [`Preflight::commit`] consumes it and is infallible.
//!
//! Nothing allocates, touches a device, or is irreversible before the commit,
//! and nothing can refuse after it. That is ADR 0051's one-way routing commit
//! expressed as three types rather than as a rule to remember.
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
//!
//! # What this loader refuses rather than approximates
//!
//! Three refusals are the honest shape of what a decoded envelope publishes,
//! not defects to route around:
//!
//! - **More than one packaged variant.** Choosing among variants means
//!   evaluating their applicability guards, and a guard is reachable only
//!   through a `VerifiedArtifactProgram` that no decode produces. A loader that
//!   took the first variant would be treating declaration order as a decided
//!   guard.
//! - **More than one payload descriptor.** A descriptor names its object by
//!   nothing a reader can follow: `BackendPayloadDescriptor::digest` is the
//!   digest of the payload's *compilation subject*, the section table is
//!   content-addressed and deduplicates equal objects, and the descriptor-to-
//!   section map is not published. One descriptor and one object section is the
//!   only cardinality in which the association is derivable.
//! - **Any execution policy other than a native image.** Device translation is
//!   by definition not device-free.
//!
//! Each of the first two is a projection gap owned by
//! `carry-reconstructable-kernel-programs-in-the-neutral-envelope`. Widening
//! this loader past them needs that ticket, not a relaxed check here.

mod host;
mod route;

pub use host::{ExecutionEnvironment, TargetCompatibility};
pub use route::{Preflight, RoutedDispatch};

use tiler_artifact::program::{
    ArtifactCodecFailure, ArtifactExecutionPolicy, BackendPayloadDescriptor,
    CanonicalArtifactProgramIdentity, DecodedArtifact, RoutingPolicy, SectionPurpose, SectionView,
    decode_artifact,
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

    /// Discharges every obligation this loader can decide, before any commit.
    ///
    /// The order is chosen so that the first refusal is the most useful one.
    /// Identity is checked first: if these are not the bytes of the artifact
    /// the caller expects, no later answer about them is worth reporting.
    /// Routing and payload selection follow, then the host's target profile,
    /// then how the object reaches an executable state, then the object itself.
    ///
    /// `expected` is the caller's own artifact identity — the one it obtained
    /// by building this artifact, or recorded when it cached these bytes. This
    /// is the binding-by-identity path a decoded envelope supports: it proves
    /// the loaded bytes *are* that artifact without reconstructing anything,
    /// because [`Self::decode`] already re-derived the identity from content
    /// rather than reading it from the manifest.
    ///
    /// # Errors
    ///
    /// Returns the [`LoadRejection`] naming the first obligation that failed.
    /// Nothing has been allocated or committed when it does.
    pub fn preflight(
        &self,
        environment: &ExecutionEnvironment,
        expected: &CanonicalArtifactProgramIdentity,
    ) -> Result<Preflight<'_>, LoadRejection> {
        let identity = self.identity();
        if &identity != expected {
            return Err(LoadRejection::ProgramMismatch {
                expected: expected.clone(),
                loaded: identity,
            });
        }

        // Exhaustive rather than a wildcard: `RoutingPolicy` is deliberately not
        // `#[non_exhaustive]` (ADR 0074 convention 5b), so a policy added to the
        // artifact layer is a build failure here instead of silently reusing
        // stable-priority selection.
        match self.decoded.routing() {
            RoutingPolicy::StablePriority => {}
        }
        let packaged = self.decoded.variant_count();
        if packaged != 1 {
            return Err(LoadRejection::UnroutableVariants { packaged });
        }

        let payload = self.select_payload(environment)?;

        let classification = environment.classify(&payload.compatibility);
        if !classification.is_compatible() {
            return Err(LoadRejection::IncompatibleTarget { classification });
        }

        // Also exhaustive, and for the same reason. Device translation is not
        // device-free, so this crate cannot deliver it and says so rather than
        // handing over bytes a host would have to guess how to use.
        match payload.execution_policy {
            ArtifactExecutionPolicy::NativeImage => {}
            policy @ ArtifactExecutionPolicy::RequiresDeviceTranslation => {
                return Err(LoadRejection::UndeliverableExecutionPolicy { policy });
            }
        }

        let object = self.resolve_object()?;
        Ok(Preflight {
            identity,
            payload,
            object,
        })
    }

    /// Selects the one payload descriptor this host can execute.
    fn select_payload(
        &self,
        environment: &ExecutionEnvironment,
    ) -> Result<&BackendPayloadDescriptor, LoadRejection> {
        let mut matching = self.decoded.payloads().iter().filter(|payload| {
            payload.backend == environment.backend
                && payload.representation == environment.representation
        });
        let Some(selected) = matching.next() else {
            return Err(LoadRejection::NoSuchPayload {
                backend: environment.backend.as_str().to_owned(),
                representation: environment.representation.as_str().to_owned(),
            });
        };
        let extra = matching.count();
        if extra > 0 {
            return Err(LoadRejection::AmbiguousPayload {
                backend: environment.backend.as_str().to_owned(),
                representation: environment.representation.as_str().to_owned(),
                matching: extra + 1,
            });
        }
        Ok(selected)
    }

    /// Resolves the object bytes the selected payload carries.
    ///
    /// Sound in exactly one cardinality, and every other is refused rather than
    /// guessed. Each *carried* payload contributes exactly one object section,
    /// and the section table is content-addressed, so with one descriptor and
    /// one object section the association is forced: that section is that
    /// payload's object. With more descriptors it is not — two equal objects
    /// deduplicate into one section and a descriptor-only payload contributes
    /// none, so the same counts have several readings and nothing published
    /// separates them.
    fn resolve_object(&self) -> Result<&[u8], LoadRejection> {
        let mut objects = self
            .decoded
            .sections()
            .filter(|section| section.purpose() == SectionPurpose::BackendPayloadCode);
        let first = objects.next();
        let object_sections = first.iter().count() + objects.count();
        let payloads = self.decoded.payloads().len();
        match (payloads, object_sections, first) {
            (1, 1, Some(object)) => Ok(object.bytes()),
            (1, 0, _) => Err(LoadRejection::ObjectNotCarried),
            _ => Err(LoadRejection::ObjectUnresolvable {
                payloads,
                object_sections,
            }),
        }
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
    /// The bytes are a valid artifact, and not the one the caller expected.
    ///
    /// The whole substance of binding by identity. Both identities are carried
    /// because a caller that logs only "mismatch" cannot tell a stale cache
    /// entry from a mixed-up path.
    ProgramMismatch {
        /// Identity the caller expected these bytes to have.
        expected: CanonicalArtifactProgramIdentity,
        /// Identity re-derived from the bytes that were actually loaded.
        loaded: CanonicalArtifactProgramIdentity,
    },
    /// The artifact packages a variant count this loader cannot route.
    ///
    /// Choosing among variants requires evaluating applicability guards, which
    /// a decoded envelope does not publish. Refusing is the fail-closed form;
    /// taking the first variant would treat declaration order as a decided
    /// guard.
    UnroutableVariants {
        /// How many plan variants the artifact packages.
        packaged: usize,
    },
    /// No packaged payload names the backend and representation this host has.
    NoSuchPayload {
        /// Governed backend family key the host stated.
        backend: String,
        /// Governed executable representation key the host stated.
        representation: String,
    },
    /// More than one payload names them, and nothing decoded chooses.
    AmbiguousPayload {
        /// Governed backend family key the host stated.
        backend: String,
        /// Governed executable representation key the host stated.
        representation: String,
        /// How many descriptors matched.
        matching: usize,
    },
    /// The payload's declared target profile is not this host's.
    ///
    /// Carries the classification rather than a message, so a caller can
    /// distinguish an artifact for another target family from one for this
    /// family under a profile descriptor the host does not offer.
    IncompatibleTarget {
        /// How the declared profile relates to the host's own.
        classification: TargetCompatibility,
    },
    /// The payload needs a delivery step this device-free loader cannot perform.
    UndeliverableExecutionPolicy {
        /// The policy the payload declares.
        policy: ArtifactExecutionPolicy,
    },
    /// The artifact names its payload and does not carry the object bytes.
    ///
    /// A descriptor-only payload is well formed; it just cannot be executed
    /// from this artifact alone.
    ObjectNotCarried,
    /// The carried objects cannot be attributed to their descriptors.
    ///
    /// Not damage and not a missing object: the envelope's public read surface
    /// publishes no descriptor-to-section map, so outside the one-descriptor,
    /// one-object cardinality the association is genuinely unavailable rather
    /// than merely inconvenient.
    ObjectUnresolvable {
        /// How many payload descriptors the artifact declares.
        payloads: usize,
        /// How many object sections the artifact frames.
        object_sections: usize,
    },
}

impl fmt::Display for LoadRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(failure) => write!(formatter, "runtime.artifact: {failure}"),
            Self::ProgramMismatch { expected, loaded } => write!(
                formatter,
                "runtime.program-mismatch: expected an artifact of {} identity bytes, loaded one \
                 of {}, and they differ",
                expected.as_bytes().len(),
                loaded.as_bytes().len(),
            ),
            Self::UnroutableVariants { packaged } => write!(
                formatter,
                "runtime.unroutable: {packaged} packaged variants, and a decoded envelope \
                 publishes no applicability guard to choose among them",
            ),
            Self::NoSuchPayload {
                backend,
                representation,
            } => write!(
                formatter,
                "runtime.no-payload: no packaged payload is {backend}/{representation}",
            ),
            Self::AmbiguousPayload {
                backend,
                representation,
                matching,
            } => write!(
                formatter,
                "runtime.ambiguous-payload: {matching} payloads are {backend}/{representation}",
            ),
            Self::IncompatibleTarget { classification } => {
                write!(formatter, "runtime.incompatible-target: {classification:?}")
            }
            Self::UndeliverableExecutionPolicy { policy } => write!(
                formatter,
                "runtime.undeliverable: a device-free loader cannot deliver {policy:?}",
            ),
            Self::ObjectNotCarried => formatter.write_str(
                "runtime.object-absent: the artifact names its payload and carries no object",
            ),
            Self::ObjectUnresolvable {
                payloads,
                object_sections,
            } => write!(
                formatter,
                "runtime.object-unresolvable: {payloads} payload descriptor(s) and \
                 {object_sections} object section(s); a decoded envelope publishes no map \
                 between them",
            ),
        }
    }
}

impl Error for LoadRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(failure) => Some(failure),
            Self::ProgramMismatch { .. }
            | Self::UnroutableVariants { .. }
            | Self::NoSuchPayload { .. }
            | Self::AmbiguousPayload { .. }
            | Self::IncompatibleTarget { .. }
            | Self::UndeliverableExecutionPolicy { .. }
            | Self::ObjectNotCarried
            | Self::ObjectUnresolvable { .. } => None,
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
    use std::error::Error;
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
        let LoadRejection::Artifact(failure) = &rejection else {
            panic!("bytes that are not an artifact are an artifact-layer rejection: {rejection}");
        };
        assert!(
            rejection.to_string().contains(&failure.to_string()),
            "the display form must not lose the boundary that refused",
        );
        assert!(
            rejection.source().is_some(),
            "the classified codec failure must stay reachable as a source",
        );
    }
}
