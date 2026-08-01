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
//! The environment [`dispatch_embedded_route`] publishes to the integration is
//! the profile the *producer* declared, so [`ExecutionEnvironment::classify`]
//! answers a real question — do these bytes name the profile they were built
//! under, under the same exact descriptor — and does not answer whether this
//! machine is a host that profile applies to. ADR 0086 gates that second
//! question and refuses on every macOS row, and nothing in this crate can ask
//! it: the facade holds no device. `prototypes/serial-sum-run` prints the same
//! distinction in the same words, and the reason it must be said here too is
//! that a reader who mistook one for the other would read a successful decode —
//! or now a successful *dispatch* — as a qualified host.
//!
//! An integration's adapter is the one that answers
//! [`RuntimeAdapter::bind_execution_context`](crate::runtime::adapter::RuntimeAdapter::bind_execution_context),
//! so it is the one that decides
//! which of the two questions a route is settled on. An adapter returning
//! [`RegionRequest::declared_environment`] has chosen producer-declared
//! equality, and [`PRODUCER_DECLARED_EQUALITY`] is the label it must report
//! beside the result.
//!
//! # Where the route goes now, and who holds the commit
//!
//! This module used to stop at the first question only a device can answer, and
//! said why: committing "means everything after this is program work, and the
//! program work is a dispatch — which needs the operand storage and the device
//! object that [`crate::value`] deliberately publishes neither of". Both now
//! exist. [`crate::value::DispatchAdapter`] yields the storage and builds the
//! integration's own [`RuntimeAdapter`](crate::runtime::adapter::RuntimeAdapter), so
//! [`dispatch_embedded_route`] hands both to
//! [`route_with_adapter`](crate::runtime::adapter::route_with_adapter) and the
//! route reaches `Preflight::commit`.
//!
//! **The commit is not this crate's to take, and that is deliberate.** Nothing
//! here calls `commit` either — `route_with_adapter` does, after every
//! obligation the loader can decide has been discharged and the adapter has
//! sized, allocated, and bound. Keeping the one-way commit inside the driver
//! that already owns it is what stops this crate becoming a second place where
//! ADR 0051's boundary is drawn, and a second place is a place the two can
//! disagree.
//!
//! What this crate still owns is the *fallback*, and it owns it by not being
//! able to lie about it: [`RouteOutcome::is_fallback`] delegates to
//! [`AdapterRouteFailure::fallback_permitted`], so whether a region falls back is
//! the failure's own classification of which side of the commit it arrived on
//! rather than an answer this module composes.
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

use std::fmt;

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, AvailabilityPhase, BackendKey, RecordedArtifactProgramIdentity,
    RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment, LoadRejection};

use crate::expansion::{RegionFacts, bind_region, build_result};
use crate::value::{BindError, DispatchAdapter, RegionOperand, RegionRequest, Tensor, dense_bytes};

/// The label a consumer reports beside any result a Tiler region dispatched.
///
/// # Why a constant rather than prose in a doc comment
///
/// Because the claim it makes is one a reader will otherwise get wrong, and
/// getting it wrong is expensive: a successful dispatch looks exactly like a
/// qualified host, and it is not one. Publishing the words means an integration
/// prints *these* words rather than a paraphrase that drops the "NOT".
///
/// The wording is `prototypes/serial-sum-run/src/proof.rs`'s, deliberately
/// unchanged — that binary prints the same distinction for the same reason, and
/// `crates/tiler/tests/labelled_diagnostic.rs` reads its source and fails if the
/// two ever stop agreeing. The `{}` is the governed target profile key the route
/// was settled against.
pub const PRODUCER_DECLARED_EQUALITY: &str =
    "DIAGNOSTIC — producer-declared equality against {}, NOT host-earned eligibility";

/// Renders [`PRODUCER_DECLARED_EQUALITY`] for one governed profile key.
#[must_use]
pub fn producer_declared_equality(profile_key: &str) -> String {
    PRODUCER_DECLARED_EQUALITY.replacen("{}", profile_key, 1)
}

/// One region's route outcome, spelled from the adapter that carried it.
///
/// The two parameters always travel together and always come from one
/// [`DispatchAdapter`], so naming them separately at every call site is two
/// chances to pair them wrongly rather than one piece of information.
pub type RegionOutcome<A> =
    RouteOutcome<<A as DispatchAdapter>::Refusal, <A as DispatchAdapter>::Failure>;

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

/// How far a guarded selection got, and whether the region fell back.
///
/// Generic over the integration's own two error types rather than flattening
/// them, for the reason ADR 0074 convention 1 gives and for one more that is
/// specific to this enum: the *split* between them is what
/// [`Self::is_fallback`] reads. `Refusal` and `Failure` are exactly ADR 0051's
/// two sides of the routing commit, and collapsing them into one type would
/// leave this module composing the answer instead of reporting it.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a caller reports one, and a
/// later outcome must be able to land additively.
#[derive(Debug)]
#[non_exhaustive]
pub enum RouteOutcome<R, F> {
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
    /// The loader refused the embedded artifact before the adapter was built.
    ///
    /// Integrity and identity land here, each keeping the loader's own typed
    /// reason. Everything the loader decides *with* an adapter in hand — target
    /// compatibility, variant applicability, the bindings — arrives as
    /// [`Self::Adapter`] carrying [`AdapterRouteFailure::Load`] instead, because
    /// by then the route is the driver's to report on.
    Refused(LoadRejection),
    /// The route did not run to completion through the integration's adapter.
    ///
    /// Carried whole rather than reclassified, because
    /// [`AdapterRouteFailure`] already names which of the nine stages ended the
    /// route and therefore which side of the commit it ended on — and that is
    /// the fact [`Self::is_fallback`] needs. Restating it here would be a second
    /// authority over ADR 0051's boundary.
    Adapter(AdapterRouteFailure<R, F>),
    /// The route committed and the dispatch ran to terminal success.
    ///
    /// The variant whose absence this module used to document. The region's
    /// declared result now holds what the kernel wrote, the semantic fallback
    /// did **not** run, and [`Self::is_fallback`] is `false` — which is the
    /// whole of what "a committed outcome" means to a caller.
    Dispatched,
}

impl<R, F> RouteOutcome<R, F> {
    /// Reports whether this outcome left the region on its semantic fallback.
    ///
    /// No longer a constant, and no longer this module's own composition. Three
    /// outcomes fall back because nothing was ever committed to; [`Self::Dispatched`]
    /// does not because the kernel produced the result; and [`Self::Adapter`]
    /// delegates to [`AdapterRouteFailure::fallback_permitted`], which is
    /// `tiler-runtime`'s exhaustive classification of which stages precede the
    /// one-way commit.
    ///
    /// Delegating rather than matching the stages again is what keeps the two
    /// from drifting. A stage added to the route is already a build error in
    /// `fallback_permitted`; restating the split here would make it a silent
    /// disagreement instead.
    ///
    /// Written without a wildcard arm (ADR 0074 convention 3) so an outcome
    /// added here must be classified deliberately.
    #[must_use]
    pub const fn is_fallback(&self) -> bool {
        match self {
            Self::NoEmbeddedPayload | Self::MalformedRouteFacts { .. } | Self::Refused(_) => true,
            Self::Adapter(failure) => failure.fallback_permitted(),
            Self::Dispatched => false,
        }
    }
}

/// Routes one embedded artifact through the integration's own device authority.
///
/// Touches no filesystem and no environment variable, and touches a device only
/// through the adapter the integration built — this crate still holds none.
///
/// # The order, and why the storage is read before the artifact
///
/// The payload selector runs first, so a target whose `#[cfg]` resolved to
/// nothing reads no value and decodes no envelope — that is a non-Apple consumer
/// of a `deliver macos;` region on every build, and it should pay for neither.
///
/// **Then the values, and only then the bytes.** That is the same ordering
/// [`bind_route_and_build`] already applies to rank, stored scalar, and extents,
/// and for the same reason: a byte run that disagrees with the extents its own
/// adapter reported is something the consumer can act on, and an artifact
/// refusal reported ahead of it would name the thing the consumer cannot fix.
/// Storage length is a value obligation, so it is checked where the other value
/// obligations are.
///
/// The artifact steps follow — the producer's declared environment, the recorded
/// identity, the decode, the declared interface — and the integration is asked
/// for a device authority last, when there is something for it to carry out.
///
/// # Errors
///
/// Returns [`BindError::StorageLengthMismatch`] for a value whose byte run is
/// not the length its own reported extents describe, and [`BindError::Adapter`]
/// carrying the integration's error when a storage borrow or the adapter
/// construction fails. A *route* that does not complete is not an error: it is a
/// [`RouteOutcome`], because a refused route is a fallback rather than a failure.
pub fn dispatch_embedded_route<A: DispatchAdapter>(
    facts: &RegionFacts,
    route: &RouteFacts,
    operands: &[&Tensor<A>],
    result: &mut A::Value,
) -> Result<RegionOutcome<A>, BindError<A::Error>> {
    let Some(delivery) = route.payload else {
        return Ok(RouteOutcome::NoEmbeddedPayload);
    };

    let Some(first) = operands.first() else {
        return Err(BindError::MalformedRegionFacts {
            detail: "a region declares no operand, so no context exists to dispatch through",
        });
    };
    // The operands are borrowed before the result, because the result's borrow
    // is exclusive and would otherwise foreclose reading the operands at all.
    let mut borrowed = Vec::with_capacity(operands.len());
    for (declared, supplied) in facts.operands.iter().zip(operands) {
        let bytes = A::storage(supplied.value()).map_err(BindError::Adapter)?;
        checked_length::<A>(declared.key, bytes.len(), A::metadata(supplied.value()))?;
        borrowed.push(RegionOperand::new(declared.key, bytes));
    }
    let result_extents = A::metadata(result).map_err(BindError::Adapter)?;
    let result_bytes = A::storage_mut(result).map_err(BindError::Adapter)?;
    checked_length::<A>(facts.result.key, result_bytes.len(), Ok(result_extents))?;

    let environment = match execution_environment(route) {
        Ok(environment) => environment,
        Err(detail) => return Ok(RouteOutcome::MalformedRouteFacts { detail }),
    };
    let Ok(expected) = RecordedArtifactProgramIdentity::from_bytes(route.artifact_identity) else {
        return Ok(RouteOutcome::MalformedRouteFacts {
            detail: "the recorded artifact identity is not a well-formed identity",
        });
    };
    // The `#[cfg]`-resolved position, carried into the decode rather than used
    // to slice the bytes: one envelope carries one payload per built family, so
    // what this consumer target selects is a *position within* the artifact it
    // already holds. A position the artifact does not declare is the loader's
    // own refusal rather than something this crate decides.
    let mut program = match DecodedProgram::decode(route.artifact, delivery) {
        Ok(program) => program,
        Err(rejection) => return Ok(RouteOutcome::Refused(rejection)),
    };
    let abi = match abi_facts(&program) {
        Ok(abi) => abi,
        Err(detail) => return Ok(RouteOutcome::MalformedRouteFacts { detail }),
    };

    let request = RegionRequest::new(borrowed, facts.result.key, result_bytes, environment);
    let mut adapter = A::dispatcher(first.context(), request).map_err(BindError::Adapter)?;

    // `route_with_adapter` owns every remaining comparison, both device stages,
    // and the one-way commit. This crate deliberately does not reimplement any
    // of them: a second driver is a second place ADR 0051's boundary is drawn.
    match route_with_adapter(&mut program, &mut adapter, &expected, &abi) {
        Ok(_completion) => Ok(RouteOutcome::Dispatched),
        Err(failure) => Ok(RouteOutcome::Adapter(failure)),
    }
}

/// Refuses a value whose byte run is not the length its own extents describe.
///
/// The metadata is passed in already-fetched rather than re-read, because
/// [`TensorAdapter::metadata`](crate::value::TensorAdapter::metadata) is the
/// adapter's own report and asking twice
/// invites two answers.
fn checked_length<A: DispatchAdapter>(
    key: &'static str,
    actual: usize,
    reported: Result<crate::value::ValueMetadata, A::Error>,
) -> Result<(), BindError<A::Error>> {
    let reported = reported.map_err(BindError::Adapter)?;
    let declared = dense_bytes(reported.storage_scalar(), reported.extents()).ok_or(
        BindError::MalformedRegionFacts {
            detail: "a value reports extents whose dense byte length overflows",
        },
    )?;
    let actual = u64::try_from(actual).unwrap_or(u64::MAX);
    if declared == actual {
        return Ok(());
    }
    Err(BindError::StorageLengthMismatch {
        input: key,
        declared,
        actual,
    })
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
/// # Why a failed dispatch is an error and a refused one is not
///
/// A refusal arrives before the routing commit, so the region is exactly where
/// it would have been without an artifact: it returns the result it declared,
/// and ADR 0053 makes that the correct behaviour for every target that is not a
/// matching selected family. A *failure* arrives after the commit, where ADR
/// 0051 permits no fallback — the result's storage holds whatever a partial
/// dispatch left in it, and returning that as though the semantic fallback had
/// produced it would be returning an incorrect tensor to preserve a fast path.
/// [`BindError::DispatchFailed`] is that refusal.
///
/// # Errors
///
/// Returns whatever [`bind_region`], [`build_result`], or
/// [`dispatch_embedded_route`] returns, [`BindError::DispatchFailed`] for a
/// committed route that did not complete, and
/// [`BindError::MalformedRegionFacts`] for a region declaring no operand.
pub fn bind_route_and_build<A: DispatchAdapter>(
    facts: &RegionFacts,
    route: &RouteFacts,
    operands: &[&Tensor<A>],
) -> Result<A::Value, BindError<A::Error>>
where
    A::Refusal: fmt::Display,
    A::Failure: fmt::Display,
{
    let bound = bind_region(facts, operands)?;
    let first = operands.first().ok_or(BindError::MalformedRegionFacts {
        detail: "a region declares no operand, so no context exists to construct its result from",
    })?;
    let mut result = build_result::<A>(facts, &bound, first.context())?;
    let outcome = dispatch_embedded_route::<A>(facts, route, operands, &mut result)?;
    // The one branch this function may not get wrong. Everything else here
    // returns the declared result; a post-commit failure must not.
    if let RouteOutcome::Adapter(failure) = &outcome
        && !failure.fallback_permitted()
    {
        return Err(BindError::DispatchFailed {
            detail: failure.to_string(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests;
