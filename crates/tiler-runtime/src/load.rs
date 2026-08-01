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
//! its tables, and it is two decisions taken in a fixed order.
//!
//! **Eligibility first.** A packaged variant this host cannot execute at all is
//! removed from the candidate set before any guard is evaluated. Eligibility is
//! decided from the loading host's stated [`ExecutionEnvironment`] and nothing
//! else: the backend family and executable representation every entry's payload
//! declares, compared *as a pair*, and the target profiles the variant and each
//! of those payloads declare, classified. Those are exactly the comparisons
//! [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 4 leaves to the loader, and none of them needs an ABI fact, an adapter,
//! or a device — the host stated which machine it is.
//!
//! **Priority second.** Among the eligible variants, and only among those,
//! declaration order is meaning under [`RoutingPolicy::StablePriority`], and the
//! first whose applicability guard evaluates true against the caller's facts is
//! selected. That variant's entry names the payload descriptor realizing it, and
//! that descriptor names its own object section. Neither association is inferred
//! from a count.
//!
//! # Why ineligibility filters rather than refuses
//!
//! It refused, and that made a multi-family artifact unroutable. The walk took
//! the first variant whose guard held and *then* compared the host against what
//! that variant's payload declared, so an artifact packaging a Metal plan ahead
//! of a CPU one refused on the Metal payload — on a host that could have run the
//! CPU plan packaged right behind it. The guard had already said "this plan
//! applies"; the host was simply not the one that plan was for, which is not a
//! reason to stop looking.
//!
//! So an ineligible variant is a **non-candidate**: it never reaches its guard,
//! and its exclusion is reported as a filter rather than as the outcome. Three
//! properties are preserved rather than traded away for that.
//!
//! - **Stable priority still holds among the eligible.** Filtering removes
//!   candidates; it never reorders the ones that remain, so the selected variant
//!   is still the producer's highest-ranked plan this host can run.
//! - **An eligible variant whose guard holds is still selected.** Nothing became
//!   more permissive: every comparison that refused before still refuses, and
//!   the entry-by-entry granularity is unchanged because eligibility walks every
//!   entry of a variant rather than the first.
//! - **A guard that cannot be *evaluated* still aborts the walk** — for an
//!   eligible variant. Skipping one would route to a plan the producer ranked
//!   lower because the caller bound too little, which is a real plan
//!   substitution. That argument does not reach an ineligible variant: it is not
//!   a candidate, so its unanswerable guard cannot substitute anything.
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
//! - **No eligible variant at all.** Every packaged variant names a backend,
//!   representation, or target profile this host did not state, so there is
//!   nothing here to run and the refusal names what excluded each one.
//! - **Every eligible guard false.** An artifact whose own guards exclude the
//!   bound facts has nothing applicable to route to, and taking a variant anyway
//!   is how a plan gets executed on a host it was proven not to fit. Distinct
//!   from the reason above: the host *can* execute these plans and the producer's
//!   own guards say none of them applies.
//! - **Any execution policy other than a native image.** Device translation is
//!   by definition not device-free. Deliberately terminal rather than an
//!   eligibility filter: a host states a profile, a backend, and a
//!   representation, and how a payload reaches an executable state is not among
//!   them. A portfolio that offers a translated and a native member declares two
//!   *representations*, which the pair comparison already filters on.

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
    delivery: usize,
}

impl DecodedProgram {
    /// Decodes and validates one encoded artifact envelope for one delivery position.
    ///
    /// # Why the position is stated here and not at each route
    ///
    /// An artifact may carry one backend object **per delivery position** — the
    /// ordered slot a consumer's build target resolves to — and every payload
    /// lookup below has to resolve through it. Taking it once, at the point the
    /// bytes become a program, is what makes "this program is being loaded as
    /// position `p`" a property of the value rather than an argument three call
    /// sites could disagree about, and it is what lets an out-of-range position
    /// refuse before any route exists.
    ///
    /// It is deliberately **not** part of [`ExecutionEnvironment`]. That record
    /// is what a *device* reports — `RuntimeAdapter::bind_execution_context`
    /// mints one by observing the machine — and the delivery position is a
    /// `#[cfg]` fact of the consumer's own compilation that no adapter can
    /// observe. Putting it there would require an adapter to report a fact it
    /// cannot know.
    ///
    /// There is no default. An artifact carrying several objects has no "the"
    /// payload, and taking the first would hand a consumer the object built for
    /// somebody else's target — which
    /// `docs/research/apple-targets/artifact-compatibility.md` records as
    /// loading and dispatching without error.
    ///
    /// [`ExecutionEnvironment`]: crate::load::ExecutionEnvironment
    /// [`RuntimeAdapter::bind_execution_context`]: crate::adapter::RuntimeAdapter::bind_execution_context
    ///
    /// # Errors
    ///
    /// Returns [`LoadRejection::Artifact`] carrying the codec's own
    /// classification of the first boundary that refused, or
    /// [`LoadRejection::UnknownDeliveryPosition`] when the artifact declares no
    /// payload at the requested position.
    pub fn decode(bytes: &[u8], delivery: usize) -> Result<Self, LoadRejection> {
        let decoded = decode_artifact(bytes).map_err(LoadRejection::Artifact)?;
        let positions = decoded.delivery_positions();
        if delivery >= positions {
            return Err(LoadRejection::UnknownDeliveryPosition {
                requested: delivery,
                positions,
            });
        }
        Ok(Self { decoded, delivery })
    }

    /// Returns the delivery position this program was loaded as.
    #[must_use]
    pub const fn delivery_position(&self) -> usize {
        self.delivery
    }

    /// Returns how many delivery positions the artifact carries a payload for.
    #[must_use]
    pub fn delivery_positions(&self) -> usize {
        self.decoded.delivery_positions()
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
    /// and it is this, exactly: program identity; variant selection, which is
    /// itself host eligibility and then the applicability guards; the selected
    /// variant's deferred predicates and live-device requirements; per entry of
    /// that variant, how its payload reaches an executable state and whether the
    /// object is carried at all; and last the launch geometry and the bindings.
    ///
    /// Identity is first because if these are not the bytes of the artifact the
    /// caller expects, no later answer about them is worth reporting. The
    /// geometry and the bindings are last because they are the only obligations
    /// that depend on the caller's facts, so every refusal that is a property of
    /// the artifact alone is reported before any that is a property of what the
    /// caller bound.
    ///
    /// The backend family, the executable representation, and both declared
    /// target profiles are decided inside selection rather than after it. That
    /// is the ordering change `select-executable-variants-across-registered-backend-families`
    /// made: comparing them after a variant had been chosen let an artifact's
    /// first plan refuse on this host's behalf while a plan it *could* run sat
    /// behind it in the same portfolio.
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
        let candidate = self.route_candidate(identity, variant, &ordered, facts)?;
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
        let candidate = self.route_candidate(identity, variant, &ordered, facts)?;
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
        let variant = self.select_variant(environment, facts)?;
        Ok((identity, variant))
    }

    fn route_candidate<'a>(
        &'a self,
        identity: CanonicalArtifactProgramIdentity,
        variant: DecodedVariant<'a>,
        ordered: &[DecodedEntry<'a>],
        facts: &AbiFacts,
    ) -> Result<route::RouteCandidate<'a>, LoadRejection> {
        let mut entries = Vec::with_capacity(ordered.len());
        for (position, entry) in ordered.iter().copied().enumerate() {
            entries.push(self.route_entry(position, entry, facts)?);
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

    /// Discharges every obligation of one entry of a selected route.
    ///
    /// **Per entry rather than once per route, and the difference is not
    /// cosmetic.** Nothing requires two entries of one variant to be realized by
    /// the same payload: `BackendPayloadDescriptor::compatibility` exists
    /// precisely because the association is many-to-one. Checking the execution
    /// policy or the carried object once and then executing a different entry's
    /// payload would be routing on a fact that was never checked.
    ///
    /// **Nothing host-relative is left here, and that is not a weakening.** The
    /// backend family, the executable representation, and the payload's own
    /// compatibility contract used to be compared at this point, one entry at a
    /// time. They are now [`Self::variant_eligibility`]'s, which walks *every*
    /// entry of a variant with the same per-entry granularity before the variant
    /// is admitted as a candidate at all — so reaching this function means those
    /// comparisons already held for this entry, and a variant for which they did
    /// not was never selected.
    fn route_entry<'a>(
        &'a self,
        position: usize,
        entry: DecodedEntry<'a>,
        facts: &AbiFacts,
    ) -> Result<RoutedEntry<'a>, LoadRejection> {
        let descriptor = entry
            .payload(self.delivery)
            .expect("a decode proved every entry is realized at this program's delivery position");
        let payload = self
            .decoded
            .payloads()
            .get(descriptor)
            .expect("a decode proved every entry names a payload the artifact declares");

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
            entry.backend_symbol(self.delivery),
            entry.transport_slots(self.delivery),
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

    /// Selects the highest-priority **eligible** variant whose guard holds.
    ///
    /// One walk, two decisions per variant, in this order: whether this host can
    /// execute the variant at all, and whether the producer's own guard says it
    /// applies. A variant that fails the first is a non-candidate and its guard
    /// is never evaluated; a variant that fails the second is a candidate the
    /// producer excluded. The two are separate outcomes because they send a
    /// caller to different repairs — find a build for this machine, or bind
    /// different facts.
    ///
    /// Declaration order *is* the priority order among the eligible, so the walk
    /// stops at the first guard that evaluates true rather than scoring the
    /// survivors. Filtering removes candidates and never reorders the ones that
    /// remain, so what is selected is still the producer's highest-ranked plan
    /// this host can run. A variant is never selected for being the only one:
    /// neither cardinality nor being the sole eligible member is a guard.
    ///
    /// A guard that cannot be *evaluated* aborts the walk instead of being
    /// skipped, and the distinction is load-bearing. A guard evaluating false
    /// is the producer's own answer that this variant does not apply, so trying
    /// the next one is what the ranking means. A guard that could not be
    /// answered is a fact the caller did not bind, and skipping past it would
    /// silently route to a variant the producer ranked lower — a real plan
    /// substitution caused by an under-bound caller, reported as a successful
    /// route. The rejection names the guard's own variant rank so the caller can
    /// see which formula went unanswered. **An ineligible variant's guard is
    /// never evaluated**, so it cannot abort the walk either: it is not a
    /// candidate, and a plan this host cannot execute substitutes nothing.
    fn select_variant(
        &self,
        environment: &ExecutionEnvironment,
        facts: &AbiFacts,
    ) -> Result<DecodedVariant<'_>, LoadRejection> {
        // Exhaustive rather than a wildcard: `RoutingPolicy` is deliberately not
        // `#[non_exhaustive]` (ADR 0074 convention 5b), so a policy added to the
        // artifact layer is a build failure here instead of silently reusing
        // stable-priority selection.
        match self.decoded.routing() {
            RoutingPolicy::StablePriority => {}
        }
        // Stays empty — and therefore unallocated — for a portfolio this host
        // can execute whole, which is the ordinary case.
        let mut filtered = Vec::new();
        for variant in self.decoded.variants() {
            let rank = variant.routing_rank();
            if let Err(reason) = self.variant_eligibility(variant, environment) {
                filtered.push(FilteredVariant {
                    variant: rank,
                    reason,
                });
                continue;
            }
            let subject = AbiSubject::ApplicabilityGuard { variant: rank };
            if boolean(variant.applicability_guard(), subject, facts)? {
                return Ok(variant);
            }
        }
        let packaged = self.decoded.variant_count();
        if filtered.len() == packaged {
            return Err(LoadRejection::NoEligibleVariant { packaged, filtered });
        }
        Err(LoadRejection::NoApplicableVariant { packaged, filtered })
    }

    /// Decides whether this host can execute one packaged variant at all.
    ///
    /// Every comparison is against the host's own stated
    /// [`ExecutionEnvironment`] and against nothing else. No ABI fact is read,
    /// no adapter is consulted, and no device is bound, which is what makes
    /// eligibility decidable ahead of the guards and therefore usable as a
    /// filter rather than as a refusal.
    ///
    /// **Every entry, not the first.** A variant may legitimately be realized by
    /// several payloads, and one whose entries name two backend families is
    /// executable by neither host — so walking the whole execution order is what
    /// keeps "this variant is eligible" a statement about the variant rather
    /// than about whichever entry happened to be looked at. It is the same
    /// per-entry granularity [`Self::route_entry`] applied after selection
    /// before this filter existed. Positions are reported in execution order, as
    /// everything else this loader reports about an entry is.
    ///
    /// The three subjects stay separate classes rather than one boolean: a
    /// backend and representation pair this host does not execute, a *plan*
    /// assessed for another profile, and an *object* built for one are three
    /// different things to go and fix.
    fn variant_eligibility(
        &self,
        variant: DecodedVariant<'_>,
        environment: &ExecutionEnvironment,
    ) -> Result<(), VariantIneligibility> {
        let classification = environment.classify(variant.target_profile());
        if !classification.is_compatible() {
            return Err(VariantIneligibility::AssessedProfile { classification });
        }
        for (entry, decoded) in variant.execution_order().enumerate() {
            // This program's delivery position, so eligibility is decided about
            // the objects this consumer would actually load rather than about a
            // sibling position's, which may name another backend entirely.
            let descriptor = decoded.payload(self.delivery).expect(
                "a decode proved every entry is realized at this program's delivery position",
            );
            let payload = self
                .decoded
                .payloads()
                .get(descriptor)
                .expect("a decode proved every entry names a payload the artifact declares");
            // As a pair. A backend family this host executes, under a
            // representation it cannot consume, is not a payload it can run, and
            // admitting either half alone would route on the one that matched.
            if payload.backend != environment.backend
                || payload.representation != environment.representation
            {
                return Err(VariantIneligibility::UnsupportedRepresentation {
                    entry,
                    declared_backend: payload.backend.as_str().to_owned(),
                    declared_representation: payload.representation.as_str().to_owned(),
                    host_backend: environment.backend.as_str().to_owned(),
                    host_representation: environment.representation.as_str().to_owned(),
                });
            }
            // The payload's own compatibility contract, classified separately
            // from the variant's. `BackendPayloadDescriptor::compatibility`
            // exists because two variants declaring different profiles may
            // realize their entries through one payload, so deriving either from
            // the other is the inference the artifact layer records that field
            // to forbid.
            let classification = environment.classify(&payload.compatibility);
            if !classification.is_compatible() {
                return Err(VariantIneligibility::PayloadProfile {
                    entry,
                    classification,
                });
            }
        }
        Ok(())
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

/// Why one packaged variant was not a candidate on this host.
///
/// Every class here is decided from the host's own stated
/// [`ExecutionEnvironment`] before any guard is evaluated, so none of them is a
/// statement about the caller's bound facts. They stay separate because the
/// repairs differ: a backend and representation pair this build does not execute
/// sends a reader to look for a different build, a *plan* assessed for another
/// profile sends them to look for a different artifact or profile revision, and
/// an *object* compiled for one says the plan was right and its emitted bytes
/// were not — the distinction
/// [`BackendPayloadDescriptor::compatibility`](tiler_artifact::program::BackendPayloadDescriptor)
/// exists to record.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: this is a classification a
/// caller consumes to decide what to do next, so a later eligibility subject
/// must be able to land additively.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum VariantIneligibility {
    /// The profile this plan variant was assessed against is not this host's.
    ///
    /// A property of the variant rather than of any one entry, which is why this
    /// is the only class here that names no entry.
    AssessedProfile {
        /// How the variant's declared profile relates to the host's own.
        classification: TargetCompatibility,
    },
    /// One entry's payload names a backend family and executable representation
    /// pair this host did not state.
    ///
    /// The pair is compared whole: each half alone failing is enough, and both
    /// report here, because "this host cannot execute these bytes" is one
    /// finding with one remedy.
    UnsupportedRepresentation {
        /// Position of the entry in the variant's own execution order.
        entry: usize,
        /// Governed backend family key that entry's payload declares.
        declared_backend: String,
        /// Governed executable representation key it declares.
        declared_representation: String,
        /// Governed backend family key this host stated.
        host_backend: String,
        /// Governed executable representation key this host stated.
        host_representation: String,
    },
    /// One entry's payload was built for a profile this host does not offer.
    PayloadProfile {
        /// Position of the entry in the variant's own execution order.
        entry: usize,
        /// How that payload's declared profile relates to the host's own.
        classification: TargetCompatibility,
    },
}

impl fmt::Display for VariantIneligibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssessedProfile { classification } => write!(
                formatter,
                "it was assessed against a profile this host does not offer: {classification:?}",
            ),
            Self::UnsupportedRepresentation {
                entry,
                declared_backend,
                declared_representation,
                host_backend,
                host_representation,
            } => write!(
                formatter,
                "entry {entry} is realized by a {declared_backend}/{declared_representation} \
                 payload and this host states {host_backend}/{host_representation}",
            ),
            Self::PayloadProfile {
                entry,
                classification,
            } => write!(
                formatter,
                "entry {entry}'s payload was built for a profile this host does not offer: \
                 {classification:?}",
            ),
        }
    }
}

/// One packaged variant this host excluded, and why.
///
/// Carried by both selection refusals so a reader can tell what was *filtered*
/// from what *failed*. A refusal reporting only that nothing routed leaves a
/// host unable to distinguish an artifact built for another machine from one
/// whose own guards excluded the facts it bound, and those are opposite repairs.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FilteredVariant {
    /// Zero-based routing rank of the excluded variant.
    pub variant: usize,
    /// The host-relative subject that excluded it.
    pub reason: VariantIneligibility,
}

impl fmt::Display for FilteredVariant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { variant, reason } = self;
        write!(formatter, "variant {variant}: {reason}")
    }
}

/// Renders a filtered set as one line, in routing-rank order.
fn render_filtered(filtered: &[FilteredVariant]) -> String {
    filtered
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
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
    /// No packaged variant is one this host can execute at all.
    ///
    /// Every variant was filtered before its guard was evaluated, so this says
    /// nothing about the facts the caller bound: the artifact packages plans for
    /// some other machine, or for a profile revision this host does not offer.
    /// The per-variant reasons are carried because a portfolio can be excluded
    /// for more than one subject at once, and "wrong backend" and "wrong profile
    /// descriptor" are different repairs.
    ///
    /// Distinct from [`Self::NoApplicableVariant`], which is the opposite
    /// finding: the host *can* execute what is packaged and the producer's own
    /// guards exclude these facts.
    NoEligibleVariant {
        /// How many plan variants the artifact packages.
        packaged: usize,
        /// Every one of them, with the subject that excluded it.
        ///
        /// Has exactly `packaged` elements — that is what makes this class
        /// rather than [`Self::NoApplicableVariant`] the one reported.
        filtered: Vec<FilteredVariant>,
    },
    /// Every **eligible** packaged variant's applicability guard evaluated false.
    ///
    /// The artifact is well formed, this host can execute at least one of the
    /// plans it packages, and none of those applies to the facts this host bound.
    /// What excluded them is the producer's own guard rather than anything about
    /// the host, which is what separates this from [`Self::NoEligibleVariant`].
    NoApplicableVariant {
        /// How many plan variants the artifact packages.
        packaged: usize,
        /// The variants that never reached their guard, with the subject that
        /// excluded each.
        ///
        /// Shorter than `packaged`, and empty when the host could execute every
        /// packaged variant. `packaged - filtered.len()` is how many guards were
        /// evaluated and answered false.
        filtered: Vec<FilteredVariant>,
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
    /// The artifact carries no payload at the delivery position the caller stated.
    ///
    /// The bytes are a valid artifact and the caller is asking to be a consumer
    /// target it was not built for. Refusing is the only fail-closed answer: an
    /// artifact carrying two objects has no way to say which of them a third
    /// build target should take, and taking the first would load the object
    /// built for somebody else — which loads and dispatches without error on the
    /// Apple targets `docs/research/apple-targets/artifact-compatibility.md`
    /// measures.
    UnknownDeliveryPosition {
        /// The delivery position the caller stated.
        requested: usize,
        /// How many the artifact declares.
        positions: usize,
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
            Self::NoEligibleVariant { packaged, filtered } => write!(
                formatter,
                "runtime.no-eligible-variant: this host can execute none of the {packaged} \
                 packaged variant(s), and no guard was evaluated: {}",
                render_filtered(filtered),
            ),
            Self::NoApplicableVariant { packaged, filtered } => write!(
                formatter,
                "runtime.no-applicable-variant: none of the {} eligible variant(s) of {packaged} \
                 packaged has an applicability guard that holds for the bound facts{}",
                packaged - filtered.len(),
                if filtered.is_empty() {
                    String::new()
                } else {
                    format!("; this host filtered {}", render_filtered(filtered))
                },
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
            Self::UnknownDeliveryPosition {
                requested,
                positions,
            } => write!(
                formatter,
                "runtime.unknown-delivery-position: this artifact carries a payload for \
                 {positions} delivery position(s) and was asked for position {requested}",
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
            | Self::NoEligibleVariant { .. }
            | Self::NoApplicableVariant { .. }
            | Self::UnansweredDeferredPredicates { .. }
            | Self::UnsatisfiedDeferredPredicate { .. }
            | Self::UnansweredRouteRequirements { .. }
            | Self::ForeignRouteRequirementOwner { .. }
            | Self::UnownedRouteRequirement { .. }
            | Self::MisansweredRouteRequirement { .. }
            | Self::UnsatisfiedRouteRequirement { .. }
            | Self::UnknownDeliveryPosition { .. }
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
    use super::{
        AbiSubject, DecodedProgram, FilteredVariant, LoadRejection, TargetCompatibility,
        VariantIneligibility,
    };
    use std::error::Error;
    use tiler_artifact::program::ArtifactCodecFailure;

    /// Bytes that are not an artifact at all are refused as malformed.
    ///
    /// The class matters more than the refusal: a host that cannot tell "this
    /// is not a Tiler artifact" from "this artifact is damaged" cannot decide
    /// whether to look for a different file or to re-fetch this one.
    #[test]
    fn foreign_bytes_are_malformed_rather_than_damaged() {
        let rejection = DecodedProgram::decode(b"not a Tiler artifact at all", 0)
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
        assert!(DecodedProgram::decode(&[], 0).is_err());
    }

    /// The rejection keeps the codec's own failure reachable as its source.
    ///
    /// Asserted because the alternative — formatting the cause into a string —
    /// is the easy way to write this type and destroys a caller's ability to
    /// match on what actually happened.
    #[test]
    fn a_rejection_preserves_the_codec_failure_it_classifies() {
        let rejection =
            DecodedProgram::decode(b"short", 0).expect_err("five bytes are not an artifact");
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

    /// Every eligibility subject reads distinguishably in a refusal.
    ///
    /// A plan assessed for another profile, an object compiled for one, and a
    /// representation this host cannot consume are three different repairs, and
    /// `TargetCompatibility` alone cannot separate the first two: a descriptor
    /// mismatch carries the key both sides agree on and nothing about which
    /// declaration carried it. The entry position is varied as well, because a
    /// multi-payload variant's filter that named no entry would leave a caller
    /// unable to tell which of its payloads was the foreign one.
    #[test]
    fn every_eligibility_subject_names_a_distinct_exclusion() {
        let rendered: Vec<String> = [
            VariantIneligibility::AssessedProfile {
                classification: TargetCompatibility::DescriptorMismatch {
                    key: "tiler.target.apple-m4".to_owned(),
                },
            },
            VariantIneligibility::PayloadProfile {
                entry: 0,
                classification: TargetCompatibility::DescriptorMismatch {
                    key: "tiler.target.apple-m4".to_owned(),
                },
            },
            VariantIneligibility::PayloadProfile {
                entry: 1,
                classification: TargetCompatibility::DescriptorMismatch {
                    key: "tiler.target.apple-m4".to_owned(),
                },
            },
            VariantIneligibility::UnsupportedRepresentation {
                entry: 0,
                declared_backend: "tiler.metal".to_owned(),
                declared_representation: "metallib".to_owned(),
                host_backend: "tiler.test.scalar-host".to_owned(),
                host_representation: "tiler.test.scalar-host-image-v1".to_owned(),
            },
            VariantIneligibility::UnsupportedRepresentation {
                entry: 1,
                declared_backend: "tiler.metal".to_owned(),
                declared_representation: "metallib".to_owned(),
                host_backend: "tiler.test.scalar-host".to_owned(),
                host_representation: "tiler.test.scalar-host-image-v1".to_owned(),
            },
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier exclusion",
            );
        }
    }

    /// The two selection refusals are told apart by class, not by their payload.
    ///
    /// The distinction is the whole substance of the filter: "this host cannot
    /// run anything here" sends a reader to find another build, and "the
    /// producer's guards excluded your facts" sends them to bind different ones.
    /// A caller that had to count `filtered` against `packaged` to work out
    /// which it was holding would be re-deriving the decision this loader
    /// already made.
    #[test]
    fn a_filtered_portfolio_and_an_unmatched_guard_are_separate_classes() {
        let filtered = vec![FilteredVariant {
            variant: 0,
            reason: VariantIneligibility::UnsupportedRepresentation {
                entry: 0,
                declared_backend: "tiler.metal".to_owned(),
                declared_representation: "metallib".to_owned(),
                host_backend: "tiler.test.scalar-host".to_owned(),
                host_representation: "tiler.test.scalar-host-image-v1".to_owned(),
            },
        }];
        let ineligible = LoadRejection::NoEligibleVariant {
            packaged: 1,
            filtered: filtered.clone(),
        };
        let inapplicable = LoadRejection::NoApplicableVariant {
            packaged: 2,
            filtered,
        };
        assert_ne!(ineligible, inapplicable);
        assert_ne!(ineligible.to_string(), inapplicable.to_string());
        // Both render what was filtered rather than only how much: an excluded
        // variant a reader cannot name is not an explanation.
        for rejection in [&ineligible, &inapplicable] {
            let text = rejection.to_string();
            assert!(
                text.contains("variant 0") && text.contains("tiler.metal"),
                "{text:?} dropped the exclusion it carries",
            );
        }
    }

    /// A guard-only refusal reports no filtered variant at all.
    ///
    /// The state a portfolio this host can execute whole is in, asserted rather
    /// than left implicit: an empty list here is what makes the count in the
    /// message the artifact's own cardinality.
    #[test]
    fn an_unfiltered_portfolio_reports_every_variant_as_eligible() {
        let rejection = LoadRejection::NoApplicableVariant {
            packaged: 3,
            filtered: Vec::new(),
        };
        let text = rejection.to_string();
        assert!(
            text.contains("none of the 3 eligible variant(s) of 3 packaged"),
            "{text:?} must say that nothing was filtered",
        );
        assert!(
            !text.contains("filtered"),
            "{text:?} must not report an exclusion it does not carry",
        );
    }
}
