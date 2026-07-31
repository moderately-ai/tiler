//! Decoding artifact bytes into a validated, device-free program record.
//!
//! # Decode, optional device preparation, and the one-way commit
//!
//! [`DecodedProgram::decode`] takes bytes and returns a fully validated read
//! view, or a typed rejection naming the class of failure.
//! [`DecodedProgram::preflight`] is the device-free path and refuses a selected variant with unanswered deferred predicates or unobserved live-device route requirements. [`DecodedProgram::prepare`] instead returns a [`LiveDeviceQualification`], which publishes every live-device requirement of the route; resolving those yields a [`RoutePreparation`] that exposes exact entries for reversible pipeline preparation and binds every property request to the entry that must answer it. Resolving those answers in turn yields the [`Preflight`], whose [`Preflight::commit`] consumes it infallibly.
//!
//! The two device stages are ordered by when their facts become true rather than by convenience: a live-device fact is readable as soon as a device is bound, and a prepared-entry fact only once its pipeline exists. Making the first a distinct stage is also what stops a host from skipping it — a `RoutePreparation` can only have come from a resolved qualification, including for a route whose requirement set is empty.
//!
//! This crate allocates nothing and touches no device — including for the live-device stage, where it publishes the rows and checks the answers a host brings back but never observes a device itself, and never interprets a backend-scoped payload. A host may bind a device while holding [`LiveDeviceQualification`] and create reversible pipeline state while holding [`RoutePreparation`], but no program buffer, encoding, submission, or other irreversible program work belongs before the commit.
//!
//! # One authority per attempt, not merely one use per authority
//!
//! Those three types alone are not enough, and the gap is worth naming because
//! this module claimed the stronger property while only holding the weaker one.
//! A non-`Clone` [`Preflight`] and a consuming `commit` prove that *a given*
//! authority is single-use. They say nothing about how many a caller may mint.
//! While [`DecodedProgram`] was `Clone` and `preflight` took `&self`, a caller
//! could mint one per clone, commit one, and still hold an uncommitted authority
//! for the same attempt — the exact state the commit exists to forbid.
//!
//! So [`DecodedProgram`] is not `Clone` and [`DecodedProgram::preflight`] takes
//! `&mut self`. A committed [`RoutedDispatch`] carries the borrow forward, so
//! the program stays exclusively borrowed for as long as the route lives and a
//! second `preflight` does not compile. Abandoning a `Preflight` instead of
//! committing it releases the borrow, which is precisely the fallback ADR 0051
//! permits and is deliberately still allowed.
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
//! is not the one this process expected, an object this build cannot execute —
//! is a different variant from a damaged file, because the two mean different
//! things to do next and collapsing them would make a version skew look like
//! corruption.
//!
//! # How a route is chosen
//!
//! Routing is driven by what the artifact declares rather than by a search over
//! its tables. Variants are tried in declaration order — which is meaning under
//! [`RoutingPolicy::StablePriority`] — and the first whose applicability guard
//! evaluates true against the caller's facts is selected. That variant's entry
//! names the payload descriptor realizing it, and that descriptor names its own
//! object section. Neither association is inferred from a count.
//!
//! **Two refusals this module documented are retracted**, because both described
//! gaps that `expose-the-dispatch-record-on-a-decoded-artifact` closed. It
//! refused more than one packaged variant on the grounds that "a guard is
//! reachable only through a `VerifiedArtifactProgram` that no decode produces",
//! and more than one payload descriptor on the grounds that "the descriptor-to-
//! section map is not published". `DecodedVariant::applicability_guard` and
//! `DecodedArtifact::payload_object` publish precisely those, so both refusals
//! are gone rather than restated, and `LoadRejection` no longer has a class for
//! either.
//!
//! # What is still refused, and why each reason survives
//!
//! - **A variant whose shared storage cannot be paired.** Multi-entry routing is
//!   supported: every entry is preflighted and returned in the variant's own
//!   execution order. What a route still needs beyond that order is the *storage*
//!   the order exists to sequence — a `Data` edge means the successor reads what
//!   the predecessor wrote, and an `Internal` binding carries no name to match on.
//!   The pairing is therefore derived, by finding each end's sole internal
//!   binding of the required access, and a route whose ends are not determined is
//!   refused here rather than guessed. This is the one place in the stack that
//!   would otherwise fail *open*: a loader allocating per binding hands the
//!   successor a fresh buffer and it reads uninitialised device memory, which is
//!   plausible garbage rather than an error.
//! - **A deferred variant on the device-free path.** [`DecodedProgram::preflight`] has no prepared entry to query and refuses rather than guessing. [`DecodedProgram::prepare`] publishes each exact-entry request for a capable host to answer.
//! - **A route requiring live-device facts, on the device-free path.** The same
//!   argument one phase earlier: `preflight` binds no device, so it can observe
//!   no device requirement and refuses instead of assuming one holds.
//! - **A route requirement no adapter decides.** An unknown owner is refused
//!   here without consulting anything, because the host stated which backend it
//!   is. An unknown key, version, or payload *within* that backend is the
//!   adapter's own `Unrecognized`, and is equally a refusal: a requirement
//!   nothing evaluated has not been met.
//! - **Every guard false.** An artifact whose own guards exclude the bound facts
//!   has nothing applicable to route to, and taking a variant anyway is how a
//!   plan gets executed on a host it was proven not to fit.
//! - **Any execution policy other than a native image.** Device translation is
//!   by definition not device-free.

mod host;
mod route;

pub use host::{ExecutionEnvironment, TargetCompatibility};
pub use route::{
    EntrySlot, LiveDeviceObservation, LiveDeviceQualification, LiveDeviceRequest, Preflight,
    RoutePreparation, RoutedBinding, RoutedDispatch, RoutedEntry, RoutedLaunch, SharedAllocation,
    TargetPropertyRequest,
};

use route::RouteRequirementRefusal;
use tiler_artifact::program::{
    AbiEvaluationError, AbiFacts, AbiValue, ArtifactCodecFailure, ArtifactExecutionPolicy,
    BackendPayloadDescriptor, BindingTarget, BufferAccess, CanonicalArtifactProgramIdentity,
    DecodedArtifact, DecodedEntry, DecodedExpr, DecodedInput, DecodedOutput, DecodedVariant,
    RecordedArtifactProgramIdentity, RouteRequirement, RouteRequirementSubject, RoutingPolicy,
    SectionView, StageDependencyReason, decode_artifact,
};

use std::error::Error;
use std::fmt;

/// One artifact's bytes, decoded and fully validated by the artifact layer.
///
/// Accessors rather than fields, and deliberately no `From`/`Deref` onto
/// [`DecodedArtifact`]: this crate's job is to add host-relative obligations on
/// top of a decode, and handing out the raw view would let a caller skip them
/// while still appearing to have gone through the runtime.
///
/// # Deliberately not `Clone`, and that is half of the routing authority
///
/// It *was* `Clone`, and that made ADR 0051's one-way commit weaker than the
/// doc-tests on [`Preflight::commit`] suggested. Those prove a single
/// [`Preflight`] cannot be committed twice or duplicated. They say nothing about
/// *minting a second one*, and a clonable program reachable through `&self`
/// could mint as many as a caller liked: clone the program, preflight both,
/// commit one, and keep an uncommitted authority for the same attempt — exactly
/// the state the commit exists to make unreachable.
///
/// The other half is that [`Self::preflight`] takes `&mut self`. Together they
/// make the property structural rather than documented: see that method for how
/// the borrow discharges it.
#[derive(Debug, Eq, PartialEq)]
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

    /// Returns the named program inputs in semantic interface order.
    ///
    /// The interface a host binds storage *to*: a routed binding addressing
    /// [`BindingTarget::ProgramInput`] names one of these keys, and the extents
    /// bound from these shapes are the free variables of every launch and
    /// accessible-range formula the artifact carries.
    ///
    /// [`BindingTarget::ProgramInput`]: tiler_artifact::program::BindingTarget::ProgramInput
    #[must_use]
    pub fn inputs(&self) -> impl ExactSizeIterator<Item = DecodedInput<'_>> {
        self.decoded.inputs()
    }

    /// Returns the named program outputs in semantic interface order.
    #[must_use]
    pub fn outputs(&self) -> impl ExactSizeIterator<Item = DecodedOutput<'_>> {
        self.decoded.outputs()
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
    /// The order is chosen so that the first refusal is the most useful one,
    /// and it is this, exactly: program identity; variant selection; the
    /// profile the selected variant was assessed against; that variant's
    /// deferred predicates and entry cardinality; the backend and
    /// representation its payload declares; the profile that payload's own
    /// bytes were built for; how that payload reaches an executable state;
    /// whether the object is carried at all; and last the launch geometry and
    /// the bindings.
    ///
    /// Identity is first because if these are not the bytes of the artifact the
    /// caller expects, no later answer about them is worth reporting. The
    /// geometry and the bindings are last because they are the only obligations
    /// that depend on the caller's facts, so every refusal that is a property of
    /// the artifact alone is reported before any that is a property of what the
    /// caller bound.
    ///
    /// `expected` is the identity of the artifact the caller means to run,
    /// stated as a [`RecordedArtifactProgramIdentity`]. This is the
    /// binding-by-identity path a decoded envelope supports: it proves the
    /// loaded bytes *are* that artifact without reconstructing anything, because
    /// [`Self::decode`] already re-derived the identity from content rather than
    /// reading it from the manifest. Its strength is exactly the strength of
    /// whatever recorded it. An identity re-read from these same bytes is a
    /// tautology, and this method cannot tell the difference, so a caller
    /// restating [`Self::identity`] has checked nothing.
    ///
    /// **An assertion type rather than a
    /// [`CanonicalArtifactProgramIdentity`], and the asymmetry is the point.**
    /// The canonical type has no public constructor, so only code that *built*
    /// or decoded an artifact can hold one; taking it here would make the second
    /// source this method documents — an identity recorded beside cached bytes —
    /// unrepresentable, which is the whole cold-consumer case. Taking bytes
    /// instead named nothing at the call site. The recorded type states which of
    /// the two this argument is, and deliberately does not claim it was derived
    /// from validated content.
    ///
    /// `facts` are the ABI facts the caller has bound — input extents, target
    /// properties. They are taken here rather than after the commit because
    /// evaluating a guard, a launch extent, or an accessible byte range can
    /// *fail*, and ADR 0051 forbids a refusal after the routing commit.
    /// Evaluating them in [`RoutedDispatch`] instead would move a failure past
    /// the point where a fallback is still permitted, which is the one thing the
    /// commit exists to prevent.
    ///
    /// # `&mut self`, because the route authority is consumable
    ///
    /// Taking `&mut self` is not about mutation — nothing here mutates. It is
    /// what makes "one authority per attempt" a fact the compiler enforces.
    ///
    /// The returned [`Preflight`] borrows this program, and [`Preflight::commit`]
    /// passes that borrow into the [`RoutedDispatch`] it returns. So for as long
    /// as a caller holds a committed route, the program stays exclusively
    /// borrowed and a second `preflight` call does not compile. Combined with
    /// [`DecodedProgram`] not being [`Clone`], there is no way to hold a
    /// committed route and an uncommitted authority for the same attempt.
    ///
    /// **Preflighting again after *abandoning* is legal and stays legal.**
    /// Dropping a `Preflight` without committing is how a caller takes the
    /// fallback ADR 0051 permits, and the borrow ends with it, so the next
    /// attempt may preflight again. What is refused is minting a second
    /// authority while one has already been *carried across* the commit.
    ///
    /// # Errors
    ///
    /// Returns the [`LoadRejection`] naming the first obligation that failed.
    /// Nothing has been allocated or committed when it does.
    ///
    /// # Panics
    ///
    /// Panics if the decoded artifact contradicts an invariant
    /// [`decode_artifact`] already proved: an entry naming a payload the
    /// artifact does not declare, an expression whose evaluated value disagrees
    /// with its validated type, or a transport mapping that is not one slot per
    /// binding. Each would be a defect in `tiler-artifact` rather than a caller
    /// error, which is why none is a returned variant a caller would handle.
    pub fn preflight(
        &mut self,
        environment: &ExecutionEnvironment,
        expected: &RecordedArtifactProgramIdentity,
        facts: &AbiFacts,
    ) -> Result<Preflight<'_>, LoadRejection> {
        let (identity, variant) = self.select_route(environment, expected, facts)?;

        // The earlier phase first. Both are unanswerable here, and a host that
        // learns it needs a bound device before it learns it needs a prepared
        // pipeline is told the first thing it is short of rather than the last.
        refuse_route_requirements(variant)?;
        refuse_deferred(variant)?;

        // Every entry, in the order the variant says they run — not the entry
        // table's canonical stage-key order, which is identity's and carries no
        // execution meaning.
        let ordered: Vec<_> = variant.execution_order().collect();
        let candidate = self.route_candidate(identity, variant, &ordered, environment, facts)?;
        Ok(Preflight::from_candidate(candidate))
    }

    /// Routes all artifact-only obligations and publishes what a device host must answer before commit.
    ///
    /// Returns the first of two device stages. The live-device requirements are
    /// published here and the prepared-entry property requests are carried
    /// through, so a host answers each at the phase its facts become readable.
    ///
    /// One check is discharged here rather than passed to an adapter: a
    /// backend-scoped requirement owned by a backend this host did not state is
    /// refused outright. That is decidable from the host's own declaration, and
    /// asking an adapter about another backend's namespace would invite it to
    /// answer.
    ///
    /// # Errors
    ///
    /// Returns the [`LoadRejection`] naming the first obligation that failed. No target property has been acquired and routing has not committed when it does.
    ///
    /// # Panics
    ///
    /// Panics if the decoded artifact contradicts the invariant that every deferred requirement names an entry in its own variant. [`decode_artifact`] proves that invariant before this route is constructed.
    pub fn prepare(
        &mut self,
        environment: &ExecutionEnvironment,
        expected: &RecordedArtifactProgramIdentity,
        facts: &AbiFacts,
    ) -> Result<LiveDeviceQualification<'_>, LoadRejection> {
        let (identity, variant) = self.select_route(environment, expected, facts)?;
        let requirements = route_requirements(variant, environment)?;
        let ordered: Vec<_> = variant.execution_order().collect();
        let mut requests = Vec::with_capacity(variant.deferred_predicates().len());
        for (predicate, deferred) in variant.deferred_predicates().enumerate() {
            let entry = ordered
                .iter()
                .position(|entry| entry.stage_key() == deferred.entry().stage_key())
                .expect("a decode proved every deferred requirement names an entry in its variant");
            requests.push(TargetPropertyRequest {
                variant: variant.routing_rank(),
                predicate,
                entry,
                requirement: deferred.requirement(),
            });
        }
        let candidate = self.route_candidate(identity, variant, &ordered, environment, facts)?;
        Ok(LiveDeviceQualification {
            candidate,
            requirements,
            requests,
        })
    }

    fn select_route<'a>(
        &'a self,
        environment: &ExecutionEnvironment,
        expected: &RecordedArtifactProgramIdentity,
        facts: &AbiFacts,
    ) -> Result<(CanonicalArtifactProgramIdentity, DecodedVariant<'a>), LoadRejection> {
        let identity = self.identity();
        if identity.as_bytes() != expected.as_bytes() {
            return Err(LoadRejection::ProgramMismatch {
                expected: expected.clone(),
                loaded: identity,
            });
        }
        let variant = self.select_variant(facts)?;
        let classification = environment.classify(variant.target_profile());
        if !classification.is_compatible() {
            return Err(LoadRejection::IncompatibleTarget {
                declaration: TargetDeclaration::Variant,
                classification,
            });
        }
        Ok((identity, variant))
    }

    fn route_candidate<'a>(
        &'a self,
        identity: CanonicalArtifactProgramIdentity,
        variant: DecodedVariant<'a>,
        ordered: &[DecodedEntry<'a>],
        environment: &ExecutionEnvironment,
        facts: &AbiFacts,
    ) -> Result<route::RouteCandidate<'a>, LoadRejection> {
        let mut entries = Vec::with_capacity(ordered.len());
        for (position, entry) in ordered.iter().copied().enumerate() {
            entries.push(self.route_entry(position, entry, environment, facts)?);
        }

        // Derived after every entry is routed, because it names slots of routed
        // entries and a refusal here must still arrive before the commit.
        let shared = shared_allocations(variant, ordered)?;

        Ok(route::RouteCandidate {
            identity,
            kernel_program: variant.kernel_program_identity(),
            entries,
            shared,
        })
    }

    /// Discharges every obligation of one entry of a route.
    ///
    /// **Per entry rather than once per route, and the difference is not
    /// cosmetic.** Nothing requires two entries of one variant to be realized by
    /// the same payload: `BackendPayloadDescriptor::compatibility` exists
    /// precisely because the association is many-to-one. Checking the backend,
    /// the representation, the payload's own compatibility contract, and the
    /// execution policy once and then executing a different entry's object would
    /// be routing on a fact that was never checked.
    fn route_entry<'a>(
        &'a self,
        position: usize,
        entry: DecodedEntry<'a>,
        environment: &ExecutionEnvironment,
        facts: &AbiFacts,
    ) -> Result<RoutedEntry<'a>, LoadRejection> {
        let descriptor = entry.payload();
        let payload = self
            .decoded
            .payloads()
            .get(descriptor)
            .expect("a decode proved every entry names a payload the artifact declares");
        if payload.backend != environment.backend
            || payload.representation != environment.representation
        {
            return Err(LoadRejection::UnexecutablePayload {
                declared_backend: payload.backend.as_str().to_owned(),
                declared_representation: payload.representation.as_str().to_owned(),
                host_backend: environment.backend.as_str().to_owned(),
                host_representation: environment.representation.as_str().to_owned(),
            });
        }

        // The payload's own compatibility contract, classified separately from
        // the variant's. `BackendPayloadDescriptor::compatibility` exists
        // because two variants declaring different profiles may realize their
        // entries through one payload, so deriving either from the other is the
        // inference the artifact layer records that field to forbid.
        let classification = environment.classify(&payload.compatibility);
        if !classification.is_compatible() {
            return Err(LoadRejection::IncompatibleTarget {
                declaration: TargetDeclaration::Payload,
                classification,
            });
        }

        // Exhaustive rather than a wildcard: `ArtifactExecutionPolicy` is
        // deliberately not `#[non_exhaustive]` (ADR 0074 convention 5b), so a
        // policy added to the artifact layer is a build failure here instead of
        // being silently treated as directly loadable.
        match payload.execution_policy {
            ArtifactExecutionPolicy::NativeImage => {}
            policy @ ArtifactExecutionPolicy::RequiresDeviceTranslation => {
                return Err(LoadRejection::UndeliverableExecutionPolicy { policy });
            }
        }

        // One condition asked three ways: a descriptor-only payload carries no
        // object, publishes no entry symbol, and publishes no transport mapping.
        // All three are answered from the descriptor position the entry named.
        let (Some(object), Some(symbol), Some(transports)) = (
            self.decoded.payload_object(descriptor),
            entry.backend_symbol(),
            entry.transport_slots(),
        ) else {
            return Err(LoadRejection::ObjectNotCarried);
        };

        Ok(RoutedEntry {
            payload,
            object,
            entry,
            symbol,
            launch: evaluate_launch(position, entry, facts)?,
            bindings: place_bindings(position, entry, transports, facts)?,
        })
    }

    /// Selects the first packaged variant whose applicability guard holds.
    ///
    /// Declaration order *is* the priority order, so the walk stops at the first
    /// guard that evaluates true rather than scoring the survivors. A variant is
    /// never selected for being the only one: cardinality is not a guard, and an
    /// artifact whose every guard is false is refused.
    ///
    /// A guard that cannot be *evaluated* aborts the walk instead of being
    /// skipped, and the distinction is load-bearing. A guard evaluating false
    /// is the producer's own answer that this variant does not apply, so trying
    /// the next one is what the ranking means. A guard that could not be
    /// answered is a fact the caller did not bind, and skipping past it would
    /// silently route to a variant the producer ranked lower — a real plan
    /// substitution caused by an under-bound caller, reported as a successful
    /// route. The rejection names the guard's own variant rank so the caller can
    /// see which formula went unanswered.
    fn select_variant(&self, facts: &AbiFacts) -> Result<DecodedVariant<'_>, LoadRejection> {
        // Exhaustive rather than a wildcard: `RoutingPolicy` is deliberately not
        // `#[non_exhaustive]` (ADR 0074 convention 5b), so a policy added to the
        // artifact layer is a build failure here instead of silently reusing
        // stable-priority selection.
        match self.decoded.routing() {
            RoutingPolicy::StablePriority => {}
        }
        for variant in self.decoded.variants() {
            let subject = AbiSubject::ApplicabilityGuard {
                variant: variant.routing_rank(),
            };
            if boolean(variant.applicability_guard(), subject, facts)? {
                return Ok(variant);
            }
        }
        Err(LoadRejection::NoApplicableVariant {
            packaged: self.decoded.variant_count(),
        })
    }
}

/// Returns a selected variant's entries in the order it says they run.
///
/// Separate from [`DecodedProgram::select_variant`] because the two answer
/// different questions. Selection asks which variant *applies*; this asks
/// whether the applicable one is something a device-free loader can carry out,
/// and its refusal is a property of the selected variant rather than a reason to
/// try the next one. Falling through to a lower-priority variant here would
/// silently substitute a plan the producer ranked below the one whose guard held.
fn refuse_deferred(variant: DecodedVariant<'_>) -> Result<(), LoadRejection> {
    let rank = variant.routing_rank();
    let deferred = variant.deferred_predicates().len();
    if deferred > 0 {
        return Err(LoadRejection::UnansweredDeferredPredicates {
            variant: rank,
            deferred,
        });
    }

    Ok(())
}

/// Refuses a selected variant whose route requires facts no device-free path can observe.
///
/// Separate from [`refuse_deferred`] because the two name different phases, and
/// a host reading a refusal has to know which one it is short of: a deferred
/// predicate needs a prepared pipeline, and a route requirement needs only a
/// bound device. Falling through to a lower-priority variant is wrong here for
/// the same reason it is wrong there — the guard that held selected *this*
/// variant, and substituting another is a plan change the producer did not rank.
fn refuse_route_requirements(variant: DecodedVariant<'_>) -> Result<(), LoadRejection> {
    let required = variant.route_requirements().len();
    if required > 0 {
        return Err(LoadRejection::UnansweredRouteRequirements {
            variant: variant.routing_rank(),
            required,
        });
    }
    Ok(())
}

/// Binds each of a variant's route requirements to a request, or refuses the route.
///
/// The one refusal available without a device is the owner check: the host
/// stated the backend family it can execute, so a row owned by another backend
/// describes a host this is not.
fn route_requirements<'a>(
    variant: DecodedVariant<'a>,
    environment: &ExecutionEnvironment,
) -> Result<Vec<LiveDeviceRequest<'a>>, LoadRejection> {
    let rank = variant.routing_rank();
    let mut requests = Vec::with_capacity(variant.route_requirements().len());
    for (position, requirement) in variant.route_requirements().iter().enumerate() {
        if let RouteRequirement::BackendFeature(feature) = requirement
            && *feature.owner() != environment.backend
        {
            return Err(LoadRejection::ForeignRouteRequirementOwner {
                variant: rank,
                position,
                owner: feature.owner().as_str().to_owned(),
                host_backend: environment.backend.as_str().to_owned(),
            });
        }
        requests.push(LiveDeviceRequest {
            variant: rank,
            position,
            requirement,
        });
    }
    Ok(requests)
}

/// Pairs the slots that must be backed by one allocation, or refuses the route.
///
/// # What a loader cannot see, and why that is dangerous rather than merely
/// inconvenient
///
/// A binding addressing entry-internal storage carries no name: the artifact
/// layer has no durable name for a program value, so two `Internal` slots are
/// indistinguishable by design. A loader allocating per binding therefore gives
/// the consuming stage a *fresh* buffer, the producing stage's result never
/// reaches it, and the dispatch reads uninitialised device memory. No digest
/// fails and no preflight refuses; the values that come back are plausible
/// garbage. Every other refusal in this stack fails closed, and this is the one
/// that would not.
///
/// # The pairing is read from the program, not guessed
///
/// Each `Data` dependency edge names a producing entry and a consuming one, and
/// the obligation it discharges is that the consumer reads what the producer
/// wrote. The two slots are located by searching each entry for its internal
/// write and its internal read.
///
/// **Searched rather than assumed at a fixed index.** Every verified kernel in
/// today's profile has exactly one read buffer and one write buffer, which makes
/// the producer's slot always 1 and the consumer's always 0 — so a fixed index
/// would work now and silently mis-bind the day that profile widens. Requiring
/// exactly one candidate on each side turns that widening into a refusal here
/// rather than a wrong answer downstream.
fn shared_allocations(
    variant: DecodedVariant<'_>,
    ordered: &[DecodedEntry<'_>],
) -> Result<Vec<SharedAllocation>, LoadRejection> {
    let rank = variant.routing_rank();
    let position = |entry: &DecodedEntry<'_>| {
        ordered
            .iter()
            .position(|candidate| candidate.stage_key() == entry.stage_key())
    };

    let mut shared = Vec::new();
    for edge in variant.stage_dependencies() {
        // Exhaustive rather than a wildcard: a reason added to the artifact
        // layer must be classified here deliberately instead of being treated as
        // a data flow it is not.
        match edge.reason() {
            StageDependencyReason::Data => {}
            // Storage handoff names an allocation the successor may reuse after
            // the predecessor released it. A loader that allocates the two
            // separately is wasteful and correct, so nothing is paired.
            StageDependencyReason::StorageHandoff => continue,
        }

        let predecessor = edge.predecessor();
        let successor = edge.successor();
        let (Some(producer_entry), Some(consumer_entry)) =
            (position(&predecessor), position(&successor))
        else {
            return Err(LoadRejection::UnpairableSharedAllocation {
                variant: rank,
                detail: "a dependency edge names an entry the execution order does not",
            });
        };

        let producer = sole_internal_slot(predecessor, BufferAccess::Write).ok_or(
            LoadRejection::UnpairableSharedAllocation {
                variant: rank,
                detail: "the producing entry does not have exactly one internal write binding",
            },
        )?;
        let consumer = sole_internal_slot(successor, BufferAccess::Read).ok_or(
            LoadRejection::UnpairableSharedAllocation {
                variant: rank,
                detail: "the consuming entry does not have exactly one internal read binding",
            },
        )?;

        shared.push(SharedAllocation {
            producer: EntrySlot {
                entry: producer_entry,
                slot: producer,
            },
            consumer: EntrySlot {
                entry: consumer_entry,
                slot: consumer,
            },
        });
    }
    Ok(shared)
}

/// Returns the one internal binding of an entry with the given access, if it is
/// the only one.
///
/// `None` covers both "no such binding" and "more than one", because both mean
/// the same thing to a caller: the pairing this route needs is not determined,
/// and guessing which slot was meant is how a wrong buffer gets bound.
///
/// Only the "no such binding" half is covered by a test. Every kernel this
/// profile verifies destructures to `[read_buffer, write_buffer]`, so an entry
/// with two internal writes is not constructible through the builder and the
/// uniqueness rejection cannot be reached from a fixture — neutering it to take
/// the first match leaves the suite green. It is retained as the check that
/// makes widening the kernel profile a refusal rather than a silent mis-bind,
/// which is a claim about the code, not a measured one.
fn sole_internal_slot(entry: DecodedEntry<'_>, access: BufferAccess) -> Option<usize> {
    let mut found = None;
    for binding in entry.bindings() {
        if matches!(binding.target(), BindingTarget::Internal) && binding.access() == access {
            if found.is_some() {
                return None;
            }
            found = Some(binding.slot());
        }
    }
    found
}

/// Evaluates one entry's launch geometry and proves its preconditions hold.
fn evaluate_launch(
    position: usize,
    entry: DecodedEntry<'_>,
    facts: &AbiFacts,
) -> Result<RoutedLaunch, LoadRejection> {
    for (index, precondition) in entry.launch_preconditions().enumerate() {
        let subject = AbiSubject::LaunchPrecondition {
            entry: position,
            index,
        };
        if !boolean(precondition, subject, facts)? {
            return Err(LoadRejection::LaunchPrecondition {
                entry: position,
                index,
            });
        }
    }
    Ok(RoutedLaunch {
        grid_threads: unsigned(
            entry.launch_threads(),
            AbiSubject::LaunchThreads { entry: position },
            facts,
        )?,
        threads_per_workgroup: unsigned(
            entry.threads_per_workgroup(),
            AbiSubject::ThreadsPerWorkgroup { entry: position },
            facts,
        )?,
        zero_work_skips_dispatch: entry.zero_work_skips_dispatch(),
    })
}

/// Places each ABI binding on its backend transport and sizes it.
///
/// `transports[slot]` is the mapping the carried payload declares, and a decode
/// proved it covers every slot, so the pairing is read rather than assumed.
fn place_bindings<'a>(
    position: usize,
    entry: DecodedEntry<'a>,
    transports: &[u32],
    facts: &AbiFacts,
) -> Result<Vec<RoutedBinding<'a>>, LoadRejection> {
    let mut placed = Vec::with_capacity(entry.bindings().len());
    for binding in entry.bindings() {
        let slot = binding.slot();
        let offset = unsigned(
            binding.accessible_offset(),
            AbiSubject::AccessibleOffset {
                entry: position,
                slot,
            },
            facts,
        )?;
        placed.push(RoutedBinding {
            binding,
            transport: *transports
                .get(slot)
                .expect("a decode proved one transport slot per ABI binding"),
            accessible_offset: offset,
            accessible_bytes: unsigned(
                binding.accessible_bytes(),
                AbiSubject::AccessibleBytes {
                    entry: position,
                    slot,
                },
                facts,
            )?,
        });
    }
    Ok(placed)
}

/// Evaluates one expression a decode proved unsigned.
fn unsigned(
    expression: DecodedExpr<'_>,
    subject: AbiSubject,
    facts: &AbiFacts,
) -> Result<u64, LoadRejection> {
    match expression
        .evaluate(facts)
        .map_err(|error| LoadRejection::AbiEvaluation { subject, error })?
    {
        AbiValue::Unsigned(value) => Ok(value),
        AbiValue::Boolean(_) => {
            panic!("a decode proved {subject} has an unsigned type, and it evaluated to a boolean")
        }
    }
}

/// Evaluates one expression a decode proved boolean.
fn boolean(
    expression: DecodedExpr<'_>,
    subject: AbiSubject,
    facts: &AbiFacts,
) -> Result<bool, LoadRejection> {
    match expression
        .evaluate(facts)
        .map_err(|error| LoadRejection::AbiEvaluation { subject, error })?
    {
        AbiValue::Boolean(value) => Ok(value),
        AbiValue::Unsigned(_) => {
            panic!("a decode proved {subject} has a boolean type, and it evaluated to an unsigned")
        }
    }
}

/// Which of an artifact's expressions an evaluation failure was reported for.
///
/// A failure there is about the *facts the caller bound* rather than about the
/// artifact, so a caller has to be able to tell which formula went unanswered:
/// an unbound input extent under a launch count and one under a binding's byte
/// range are the same error class and two different mistakes.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: an expression class added
/// to the dispatch record lands here additively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum AbiSubject {
    /// One plan variant's applicability guard, by routing rank.
    ApplicabilityGuard {
        /// Zero-based routing rank of the variant whose guard was evaluated.
        variant: usize,
    },
    /// One routed entry's total launch thread count.
    LaunchThreads {
        /// Position of the entry in the route's execution order.
        entry: usize,
    },
    /// One routed entry's per-workgroup thread count.
    ThreadsPerWorkgroup {
        /// Position of the entry in the route's execution order.
        entry: usize,
    },
    /// One launch-instance precondition, by declaration position.
    LaunchPrecondition {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based position among that entry's preconditions.
        index: usize,
    },
    /// One binding's minimum accessible byte range, by ABI slot.
    AccessibleBytes {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot of the binding whose range was evaluated.
        slot: usize,
    },
    /// One binding's accessible-range starting offset, by ABI slot.
    AccessibleOffset {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot of the binding whose offset was evaluated.
        slot: usize,
    },
}

impl fmt::Display for AbiSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApplicabilityGuard { variant } => {
                write!(formatter, "variant {variant}'s applicability guard")
            }
            Self::LaunchThreads { entry } => {
                write!(formatter, "entry {entry}'s launch thread count")
            }
            Self::ThreadsPerWorkgroup { entry } => {
                write!(formatter, "entry {entry}'s per-workgroup thread count")
            }
            Self::LaunchPrecondition { entry, index } => {
                write!(formatter, "entry {entry}'s launch precondition {index}")
            }
            Self::AccessibleBytes { entry, slot } => write!(
                formatter,
                "the accessible byte range of entry {entry}'s ABI slot {slot}"
            ),
            Self::AccessibleOffset { entry, slot } => write!(
                formatter,
                "the accessible-range offset of entry {entry}'s ABI slot {slot}"
            ),
        }
    }
}

/// Which declaration of a target profile a compatibility refusal was about.
///
/// Two are classified and they are separate declarations. Reporting only the
/// classification would leave a host unable to tell a plan *assessed* for
/// another profile from an object *compiled* for one, and those are different
/// things to go and fix.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetDeclaration {
    /// The profile the selected plan variant was assessed against.
    Variant,
    /// The profile the carried payload's own bytes were built for.
    Payload,
}

impl fmt::Display for TargetDeclaration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variant => formatter.write_str("the selected variant"),
            Self::Payload => formatter.write_str("the carried payload"),
        }
    }
}

/// Why one artifact was not accepted for execution on this host.
///
/// The classes answer different questions, which is the whole reason there is
/// more than one. Bytes the artifact layer refused, an artifact that is not the
/// one this process expected, a host that cannot honour a declared target
/// profile, an artifact whose own guards exclude this host, and a carried object
/// this build cannot execute are different things to do next; reporting them as
/// one would make a stale cache entry indistinguishable from a corrupt file.
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
    /// The whole substance of binding by identity. Both sides are carried
    /// because a caller that logs only "mismatch" cannot tell a stale cache
    /// entry from a mixed-up path.
    ///
    /// The two are deliberately different types, and the asymmetry is the point:
    /// the loaded side was *derived* from content this decode validated, and the
    /// expected side is a byte string somebody *recorded*. Spelling both as one
    /// type would suggest the two carry equal evidence.
    ProgramMismatch {
        /// The identity the caller recorded and stated as its expectation.
        expected: RecordedArtifactProgramIdentity,
        /// Identity re-derived from the bytes that were actually loaded.
        loaded: CanonicalArtifactProgramIdentity,
    },
    /// Every packaged variant's applicability guard evaluated false.
    ///
    /// The artifact is well formed and nothing it packages applies to the facts
    /// this host bound. Distinct from an incompatible target profile: what
    /// excluded it is the producer's own guard rather than the host's profile.
    NoApplicableVariant {
        /// How many plan variants the artifact packages.
        packaged: usize,
    },
    /// The selected variant defers feasibility predicates this loader cannot
    /// answer.
    ///
    /// Answering one means querying the provider the predicate names, and this
    /// crate holds no provider registry. Refusing is the fail-closed form;
    /// routing past an open feasibility condition is not.
    UnansweredDeferredPredicates {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// How many predicates it defers.
        deferred: usize,
    },
    /// One exact prepared-entry target property did not satisfy its retained requirement.
    UnsatisfiedDeferredPredicate {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Zero-based position among that variant's deferred predicates.
        predicate: usize,
        /// Position of the queried entry in the route's execution order.
        entry: usize,
    },
    /// The selected variant requires live-device facts this device-free path
    /// cannot observe.
    ///
    /// The route-requirement counterpart of
    /// [`Self::UnansweredDeferredPredicates`], and refused for the same reason:
    /// observing one means binding a device, and
    /// [`DecodedProgram::preflight`] binds none. A capable host reaches the
    /// rows through [`DecodedProgram::prepare`].
    UnansweredRouteRequirements {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// How many live-device requirements it declares.
        required: usize,
    },
    /// A backend-scoped route requirement is owned by a backend this host is not.
    ///
    /// Decided without a device and without consulting any adapter: the host
    /// states the backend family it can execute, so a row owned by another one
    /// can only mean the artifact expects a host this is not. Distinct from
    /// [`Self::UnownedRouteRequirement`], which is this host's own backend
    /// failing to recognize a key, version, or payload within its namespace.
    ForeignRouteRequirementOwner {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Zero-based position among that variant's route requirements.
        position: usize,
        /// Governed backend key that owns the requirement.
        owner: String,
        /// Governed backend key this host stated.
        host_backend: String,
    },
    /// No adapter on this host claimed one live-device route requirement.
    ///
    /// The fail-closed outcome of an unknown owner, key, version, or payload.
    /// A requirement nothing evaluated is not a requirement that holds.
    UnownedRouteRequirement {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Zero-based position among that variant's route requirements.
        position: usize,
        /// The exact subject nothing decided.
        subject: RouteRequirementSubject,
    },
    /// A host's answer disagreed in shape with the requirement's own kind.
    ///
    /// A measured quantity offered for a qualitative row, or a capability
    /// verdict offered for a floor. Refused rather than coerced: either coercion
    /// would invent a comparison the producer did not state.
    MisansweredRouteRequirement {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Zero-based position among that variant's route requirements.
        position: usize,
        /// The subject that was answered in the wrong shape.
        subject: RouteRequirementSubject,
    },
    /// The live device does not satisfy one requirement of the selected route.
    ///
    /// Names the exact unmet requirement, which is the whole point of carrying
    /// the subject: a host that learns only "a requirement failed" cannot tell a
    /// missing GPU capability from a capacity floor.
    UnsatisfiedRouteRequirement {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Zero-based position among that variant's route requirements.
        position: usize,
        /// The exact unmet subject.
        subject: RouteRequirementSubject,
    },
    /// The routed entry's payload is not one this host stated it can execute.
    UnexecutablePayload {
        /// Governed backend family key the packaged payload declares.
        declared_backend: String,
        /// Governed executable representation key the packaged payload declares.
        declared_representation: String,
        /// Governed backend family key this host stated.
        host_backend: String,
        /// Governed executable representation key this host stated.
        host_representation: String,
    },
    /// A declared target profile is not this host's.
    ///
    /// Carries which declaration refused and how it relates to the host's own,
    /// so a caller can distinguish an artifact for another target family from
    /// one for this family under a profile descriptor the host does not offer.
    IncompatibleTarget {
        /// Which of the two declarations was classified.
        declaration: TargetDeclaration,
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
    /// from this artifact alone, and it publishes no entry symbol or transport
    /// mapping either.
    ObjectNotCarried,
    /// An ABI expression could not be evaluated against the caller's facts.
    ///
    /// Always about the facts rather than the artifact: a decode already proved
    /// every operand's type and that no root escapes its availability phase, so
    /// what remains is an extent or property the caller did not bind, or an
    /// arithmetic boundary the bound values crossed.
    AbiEvaluation {
        /// Which of the artifact's expressions was being evaluated.
        subject: AbiSubject,
        /// The artifact layer's own account of why it could not be evaluated.
        error: AbiEvaluationError,
    },
    /// A launch-instance precondition evaluated false.
    ///
    /// The formula was answerable and its answer forbids this launch. Separate
    /// from [`Self::AbiEvaluation`] because an unanswered precondition is a
    /// caller that bound too little, and a false one is a launch the artifact
    /// itself declares invalid.
    LaunchPrecondition {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based position of the precondition that did not hold.
        index: usize,
    },
    /// A data dependency's shared storage could not be paired to two slots.
    ///
    /// An internal binding carries no name, so a loader that allocated per
    /// binding would give the consuming stage a fresh buffer and it would read
    /// uninitialised device memory — a wrong answer rather than a refusal. The
    /// pairing is derived from the variant's own typed dependency edges, and
    /// when it is not determined this refuses instead of guessing which slot was
    /// meant.
    UnpairableSharedAllocation {
        /// Zero-based routing rank of the selected variant.
        variant: usize,
        /// Which part of the pairing was not determined.
        detail: &'static str,
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
            Self::NoApplicableVariant { packaged } => write!(
                formatter,
                "runtime.no-applicable-variant: none of the {packaged} packaged variant(s) has an \
                 applicability guard that holds for the bound facts",
            ),
            Self::UnansweredDeferredPredicates { variant, deferred } => write!(
                formatter,
                "runtime.deferred-predicates: variant {variant} defers {deferred} feasibility \
                predicate(s), and a device-free loader queries no provider",
            ),
            Self::UnsatisfiedDeferredPredicate {
                variant,
                predicate,
                entry,
            } => write!(
                formatter,
                "runtime.unsatisfied-deferred-predicate: variant {variant}'s predicate \
                 {predicate} does not hold for prepared entry {entry}",
            ),
            Self::UnansweredRouteRequirements { variant, required } => write!(
                formatter,
                "runtime.route-requirements: variant {variant} requires {required} live-device \
                 fact(s), and a device-free loader binds no device",
            ),
            Self::ForeignRouteRequirementOwner {
                variant,
                position,
                owner,
                host_backend,
            } => write!(
                formatter,
                "runtime.foreign-route-requirement: variant {variant}'s requirement {position} is \
                 owned by {owner} and this host states {host_backend}",
            ),
            Self::UnownedRouteRequirement {
                variant,
                position,
                subject,
            } => write!(
                formatter,
                "runtime.unowned-route-requirement: no adapter decided variant {variant}'s \
                 requirement {position}, {subject}",
            ),
            Self::MisansweredRouteRequirement {
                variant,
                position,
                subject,
            } => write!(
                formatter,
                "runtime.misanswered-route-requirement: variant {variant}'s requirement \
                 {position}, {subject}, was answered in the wrong shape",
            ),
            Self::UnsatisfiedRouteRequirement {
                variant,
                position,
                subject,
            } => write!(
                formatter,
                "runtime.unsatisfied-route-requirement: this device does not satisfy variant \
                 {variant}'s requirement {position}, {subject}",
            ),
            Self::UnexecutablePayload {
                declared_backend,
                declared_representation,
                host_backend,
                host_representation,
            } => write!(
                formatter,
                "runtime.unexecutable-payload: the routed entry is realized by a \
                 {declared_backend}/{declared_representation} payload and this host states \
                 {host_backend}/{host_representation}",
            ),
            Self::IncompatibleTarget {
                declaration,
                classification,
            } => write!(
                formatter,
                "runtime.incompatible-target: {declaration} declares a profile this host does not \
                 offer: {classification:?}",
            ),
            Self::UndeliverableExecutionPolicy { policy } => write!(
                formatter,
                "runtime.undeliverable: a device-free loader cannot deliver {policy:?}",
            ),
            Self::ObjectNotCarried => formatter.write_str(
                "runtime.object-absent: the artifact names its payload and carries no object",
            ),
            Self::AbiEvaluation { subject, error } => write!(
                formatter,
                "runtime.abi-evaluation: {subject} could not be evaluated: {error}",
            ),
            Self::LaunchPrecondition { entry, index } => write!(
                formatter,
                "runtime.launch-precondition: entry {entry}'s precondition {index} does not hold \
                 for the bound facts",
            ),
            Self::UnpairableSharedAllocation { variant, detail } => write!(
                formatter,
                "runtime.unpairable-shared-allocation: variant {variant} declares a data \
                 dependency whose shared storage is not determined: {detail}",
            ),
        }
    }
}

impl Error for LoadRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Artifact(failure) => Some(failure),
            Self::AbiEvaluation { error, .. } => Some(error),
            Self::ProgramMismatch { .. }
            | Self::NoApplicableVariant { .. }
            | Self::UnansweredDeferredPredicates { .. }
            | Self::UnsatisfiedDeferredPredicate { .. }
            | Self::UnansweredRouteRequirements { .. }
            | Self::ForeignRouteRequirementOwner { .. }
            | Self::UnownedRouteRequirement { .. }
            | Self::MisansweredRouteRequirement { .. }
            | Self::UnsatisfiedRouteRequirement { .. }
            | Self::UnexecutablePayload { .. }
            | Self::IncompatibleTarget { .. }
            | Self::UndeliverableExecutionPolicy { .. }
            | Self::ObjectNotCarried
            | Self::LaunchPrecondition { .. }
            | Self::UnpairableSharedAllocation { .. } => None,
        }
    }
}

impl LoadRejection {
    /// Builds the rejection one refused live-device request produces.
    ///
    /// Minted from the row itself rather than from anything a host supplied, so
    /// the refusal names the exact requirement the artifact declared and cannot
    /// name one it does not.
    fn from_request(refusal: RouteRequirementRefusal, request: LiveDeviceRequest<'_>) -> Self {
        let variant = request.variant();
        let position = request.position();
        let subject = request.requirement().subject();
        match refusal {
            RouteRequirementRefusal::Unowned => Self::UnownedRouteRequirement {
                variant,
                position,
                subject,
            },
            RouteRequirementRefusal::Misanswered => Self::MisansweredRouteRequirement {
                variant,
                position,
                subject,
            },
            RouteRequirementRefusal::Unsatisfied => Self::UnsatisfiedRouteRequirement {
                variant,
                position,
                subject,
            },
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
    use super::{AbiSubject, DecodedProgram, LoadRejection, TargetDeclaration};
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

    /// Each ABI subject names a distinct expression in its display form.
    ///
    /// The subject is the whole value of the evaluation rejection: the error it
    /// carries is one class for an unbound extent under a launch count and
    /// under a byte range, so a subject that read the same for both would leave
    /// a caller unable to tell which formula went unanswered.
    #[test]
    fn every_abi_subject_names_a_distinct_expression() {
        let rendered: Vec<String> = [
            AbiSubject::ApplicabilityGuard { variant: 0 },
            AbiSubject::LaunchThreads { entry: 0 },
            AbiSubject::ThreadsPerWorkgroup { entry: 0 },
            AbiSubject::LaunchPrecondition { entry: 0, index: 0 },
            AbiSubject::AccessibleBytes { entry: 0, slot: 0 },
            AbiSubject::AccessibleOffset { entry: 0, slot: 0 },
            // A second entry must render distinguishably from the first, or a
            // multi-stage route's refusal would not say which stage it is about.
            AbiSubject::LaunchThreads { entry: 1 },
            AbiSubject::AccessibleBytes { entry: 1, slot: 0 },
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier subject",
            );
        }
    }

    /// The two target declarations are distinguishable in a refusal.
    ///
    /// A plan assessed for another profile and an object compiled for one are
    /// different repairs, and `TargetCompatibility` alone cannot separate them:
    /// a descriptor mismatch carries the key both sides agree on and nothing
    /// about which declaration carried it.
    #[test]
    fn a_target_refusal_names_which_declaration_it_is_about() {
        assert_ne!(
            TargetDeclaration::Variant.to_string(),
            TargetDeclaration::Payload.to_string(),
        );
    }
}
