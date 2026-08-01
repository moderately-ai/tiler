//! Guarded selection of the artifact one expansion embedded, and the fallback
//! it takes before the routing commit.
//!
//! Everything here is reachable only through [`crate::__private`], on the same
//! footing as [`crate::expansion`]: these are the items generated tokens name,
//! not a surface a consumer writes.
//!
//! # What a delivered region carries into the consumer's binary
//!
//! An expansion that stated a selected artifact family embeds one artifact
//! envelope as a byte-string literal, together with the facts its *producer*
//! declared about it: the artifact's canonical identity, the target profile key
//! and exact descriptor the plan was compiled under, and the backend and
//! representation the payload realizes. [`RouteFacts`] is that record, and
//! `tiler_macros::aot` is the only thing that constructs one.
//!
//! Those facts are read from the produced artifact rather than restated: the
//! expansion takes each of them off the verified artifact program it just
//! assembled, so a frontend-local copy cannot come to disagree with the bytes it
//! ships beside.
//!
//! # This is producer-declared equality, not host-earned eligibility
//!
//! The environment [`select_embedded_route`] hands the loader is the profile the
//! *producer* declared, so [`ExecutionEnvironment::classify`] answers a real
//! question — do these bytes name the profile they were built under, under the
//! same exact descriptor — and does not answer whether this machine is a host
//! that profile applies to. ADR 0086 gates that second question and refuses on
//! every macOS row, and nothing in this crate can ask it: the facade holds no
//! device. `prototypes/serial-sum-run` prints the same distinction in the same
//! words, and the reason it must be said here too is that a reader who mistook
//! one for the other would read a successful decode as a qualified host.
//!
//! # Where the route stops, and why that is the whole of it today
//!
//! [`select_embedded_route`] walks the loader exactly as far as a device-free
//! caller can: it decodes, matches the artifact against the identity the
//! expansion recorded, selects the variant, and then stops at the first question
//! only a device can answer. It never calls `Preflight::commit`, and there is no
//! call to it anywhere in this crate — the fallback ADR 0051 permits is taken by
//! dropping a pre-commit stage, which is what every path below does.
//!
//! That is not a shortcut. Committing means "everything after this is program
//! work", and the program work is a dispatch — which needs the operand storage
//! and the device object that [`crate::value`] deliberately publishes neither of.
//! A route that committed here would have committed to nothing.
//! `route-an-embedded-artifact-through-a-consumer-storage-seam` is the ticket
//! that carries the seam, and until it lands the honest terminal state of a
//! guarded selection in a `tiler`-only consumer is
//! [`RouteOutcome::NoDeviceAuthority`].
//!
//! # Why the facade decodes at all, rather than holding inert bytes
//!
//! Two reasons, and neither is anticipation. A `const` nothing reads is a `const`
//! the linker need not emit, so "the bytes are embedded in the produced binary"
//! would stop being true of an expansion whose bytes no code touched. And a
//! decode is the guard: `tiler_runtime::load::DecodedProgram::decode` re-proves
//! the manifest digest, every section digest, and the artifact's canonical
//! identity, so a consumer that received damaged or foreign bytes learns it here
//! rather than at a dispatch that does not exist yet.

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, AvailabilityPhase, BackendKey, RecordedArtifactProgramIdentity,
    RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment, LoadRejection};

use crate::expansion::{RegionFacts, bind_region, build_result};
use crate::value::{BindError, Tensor, TensorAdapter};

/// What one expansion embedded, and what its producer declared about it.
///
/// Every field is `&'static` because an expansion emits each of them as a
/// literal; nothing here is derived at runtime. The fields are public for the
/// same reason [`RegionFacts`]' are: generated tokens construct this value in
/// the consumer's crate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RouteFacts {
    /// The one artifact envelope this region's selection produced.
    ///
    /// One envelope carries one payload per built family, so this is the whole
    /// selection's bytes and not this consumer target's share of them.
    pub artifact: &'static [u8],
    /// This consumer target's payload position within [`Self::artifact`], or
    /// `None` when the target matched no built family and takes the fallback.
    ///
    /// The `#[cfg]` selector the expansion emits is what decides this, so it is
    /// already resolved by the time this record exists.
    pub payload: Option<usize>,
    /// The canonical artifact identity the expansion recorded for those bytes.
    pub artifact_identity: &'static [u8],
    /// The governed target profile key the plan was compiled under.
    pub target_profile_key: &'static str,
    /// The exact target profile descriptor identity that key was carried with.
    pub target_profile_descriptor: &'static [u8],
    /// The governed backend family the embedded payload realizes.
    pub backend: &'static str,
    /// The governed executable representation the embedded payload is in.
    pub representation: &'static str,
}

/// How far a guarded selection got before the fallback was taken.
///
/// There is no `Committed` variant, and its absence is the point: nothing in
/// this crate calls `Preflight::commit`, so every outcome here is a pre-commit
/// one by construction rather than by discipline.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a caller reports one, and a
/// later outcome — a committed route, once a storage seam exists — must be able
/// to land additively.
#[derive(Debug)]
#[non_exhaustive]
pub enum RouteOutcome {
    /// This consumer target matched no built artifact family.
    ///
    /// The expansion's `#[cfg]` selector resolved to `None`, so there is nothing
    /// to route and the semantic fallback is the region's declared behaviour on
    /// this target rather than a downgrade from anything.
    NoEmbeddedPayload,
    /// The embedded facts do not form a host environment at all.
    ///
    /// A defect in the expansion rather than in the bytes: the producer's own
    /// profile key, descriptor, backend, or representation exceeded an artifact
    /// identity bound on the way into a literal.
    MalformedRouteFacts {
        /// Which field could not be restated.
        detail: &'static str,
    },
    /// The loader refused the embedded artifact before any device question.
    ///
    /// Integrity, identity, target compatibility, and variant applicability all
    /// land here, each keeping the loader's own typed reason.
    Refused(LoadRejection),
    /// The route reached the first question only a device can answer.
    ///
    /// This is the terminal state of a guarded selection in a `tiler`-only
    /// consumer: the artifact is valid, it is the one the expansion recorded, it
    /// declares this producer's profile, and a variant applies — and the facade
    /// holds no device authority to answer what comes next, so the pre-commit
    /// stage is dropped and the fallback runs.
    NoDeviceAuthority {
        /// How many live-device rows the selected variant declares.
        live_device_requirements: usize,
    },
}

impl RouteOutcome {
    /// Reports whether this outcome left the region on its semantic fallback.
    ///
    /// Every outcome does today, which is why this reads as a constant rather
    /// than a query. It is written as a method rather than assumed at the call
    /// site so that the day a committed variant exists, the call site is a
    /// compile error instead of a silent wrong answer.
    #[must_use]
    pub const fn is_fallback(&self) -> bool {
        match self {
            Self::NoEmbeddedPayload
            | Self::MalformedRouteFacts { .. }
            | Self::Refused(_)
            | Self::NoDeviceAuthority { .. } => true,
        }
    }
}

/// Walks the loader as far as a device-free caller can, and stops.
///
/// Pure with respect to the process: it reads the embedded bytes and the
/// embedded facts, touches no filesystem, no environment variable, and no
/// device, and returns what it found. It never commits.
#[must_use]
pub fn select_embedded_route(route: &RouteFacts) -> RouteOutcome {
    if route.payload.is_none() {
        return RouteOutcome::NoEmbeddedPayload;
    }
    let environment = match execution_environment(route) {
        Ok(environment) => environment,
        Err(detail) => return RouteOutcome::MalformedRouteFacts { detail },
    };
    let Ok(expected) = RecordedArtifactProgramIdentity::from_bytes(route.artifact_identity) else {
        return RouteOutcome::MalformedRouteFacts {
            detail: "the recorded artifact identity is not a well-formed identity",
        };
    };
    let mut program = match DecodedProgram::decode(route.artifact) {
        Ok(program) => program,
        Err(rejection) => return RouteOutcome::Refused(rejection),
    };
    let facts = match abi_facts(&program) {
        Ok(facts) => facts,
        Err(detail) => return RouteOutcome::MalformedRouteFacts { detail },
    };
    // `prepare` rather than `preflight`: the shortcut path refuses any variant
    // declaring a device-answerable row, and a Metal plan declares one per
    // prepared entry, so taking it would report an unanswered-requirement
    // refusal for a route that is in fact perfectly well formed.
    let qualification = match program.prepare(&environment, &expected, &facts) {
        Ok(qualification) => qualification,
        Err(rejection) => return RouteOutcome::Refused(rejection),
    };
    let live_device_requirements = qualification.live_device_requirements().len();
    // The qualification is dropped here, unresolved, and that drop *is* the
    // fallback ADR 0051 permits. Answering these rows would mean stating device
    // facts this crate cannot observe; answering them wrongly is the one thing
    // a guard must never do, because every feasibility decision after it would
    // be made against a machine that does not exist.
    drop(qualification);
    RouteOutcome::NoDeviceAuthority {
        live_device_requirements,
    }
}

/// Restates the producer's declared environment from the emitted facts.
fn execution_environment(route: &RouteFacts) -> Result<ExecutionEnvironment, &'static str> {
    Ok(ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new(route.target_profile_key)
                .map_err(|_| "the recorded target profile key is not a governed key")?,
            descriptor: TargetProfileDescriptorDigest::from_bytes(route.target_profile_descriptor)
                .map_err(
                    |_| "the recorded target profile descriptor is not a descriptor identity",
                )?,
        },
        backend: BackendKey::new(route.backend)
            .map_err(|_| "the recorded backend family is not a governed backend key")?,
        representation: RepresentationKey::new(route.representation)
            .map_err(|_| "the recorded representation is not a governed representation key")?,
    })
}

/// Binds the ABI facts a route evaluation needs from the artifact's own
/// declared interface.
///
/// The extents come from the artifact rather than from the values the region
/// was handed, and that is correct rather than convenient: only a region whose
/// every declared extent is literal can be compiled at all — a symbolic one has
/// no semantic program to optimize — so the artifact's declared input shapes
/// *are* the region's, and reading them here keeps this function free of any
/// second derivation that could disagree with the packaged one.
fn abi_facts(program: &DecodedProgram) -> Result<AbiFacts, &'static str> {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_input_shape(input.key(), input.shape())
            .map_err(|_| "the artifact's declared input shapes do not bind")?;
    }
    Ok(binder.build())
}

/// Checks one region's operands, routes its embedded artifact, and constructs
/// its declared result.
///
/// The one item a `tiler::tensor!` expansion calls when its `deliver` statement
/// selected an artifact family; a region delivering `FallbackOnly` embeds
/// nothing and calls [`crate::expansion::bind_and_build`] instead.
///
/// The order is the contract. The region's own obligations — operand count,
/// rank, stored scalar, symbol unification — are checked *first*, so a region
/// whose interface was not honoured refuses with the reason a consumer can act
/// on rather than with whatever the loader made of an artifact it was never
/// going to use. Guarded selection runs second, and its outcome is deliberately
/// not an error: a refused artifact leaves the region on the semantic fallback
/// it always had, which is the behaviour ADR 0053 requires of every target that
/// is *not* a matching selected family, and the behaviour ADR 0051 permits
/// before the commit for every target that is.
///
/// # Errors
///
/// Returns whatever [`bind_region`] or [`build_result`] returns, and
/// [`BindError::MalformedRegionFacts`] for a region declaring no operand.
pub fn bind_route_and_build<A: TensorAdapter>(
    facts: &RegionFacts,
    route: &RouteFacts,
    operands: &[&Tensor<A>],
) -> Result<A::Value, BindError<A::Error>> {
    let bound = bind_region(facts, operands)?;
    let first = operands.first().ok_or(BindError::MalformedRegionFacts {
        detail: "a region declares no operand, so no context exists to construct its result from",
    })?;
    let outcome = select_embedded_route(route);
    debug_assert!(
        outcome.is_fallback(),
        "a guarded selection reached a state this build cannot execute",
    );
    build_result::<A>(facts, &bound, first.context())
}

#[cfg(test)]
mod tests;
