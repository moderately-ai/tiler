//! The consumer-selected runtime adapter seam, and the loader-driven route that uses it.
//!
//! # There is no registry, and independent selection is the mechanism
//!
//! [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! decides that a component claims individual responsibilities rather than a
//! whole backend, and that the runtime adapter — row 12 of its responsibility
//! matrix — is *independently selected*. Nothing here registers an adapter,
//! discovers one, or resolves one from a key. A consumer names the adapter it
//! links and hands it to [`route_with_adapter`]; that is the whole mechanism,
//! and giving it a registry is an eliminated alternative rather than a missing
//! feature.
//!
//! What joins the producing backend to the executing adapter is therefore not a
//! Rust object, a `TypeId`, or a registration handle — none of which survives
//! the process boundary an artifact crosses. It is the artifact's own governed
//! identities: the target profile, and the backend family and executable
//! representation the carried payload declares. **The join is bytes**, and
//! producer provenance is never matched against adapter identity, because there
//! is no adapter identity in an artifact to match against.
//!
//! # Every comparison is the loader's; the adapter only reports
//!
//! This is the property most easily eroded by a plausible simplification, so it
//! is stated as a division rather than left implicit. The adapter answers
//! questions and never rules on them:
//!
//! | Subject | Who reports | Who compares |
//! |---|---|---|
//! | Program identity | the caller states an expectation | [`DecodedProgram::prepare`] |
//! | Applicable variant | the artifact's own guards | [`DecodedProgram::prepare`] |
//! | Target profile, variant and payload | [`RuntimeAdapter::bind_execution_context`] | [`ExecutionEnvironment::classify`] |
//! | Backend family and representation | [`RuntimeAdapter::bind_execution_context`] | [`DecodedProgram::prepare`], **as a pair** |
//! | Carried payload's own bytes | — | [`RuntimeAdapter::validate_payload`] (the backend owns this one; see below) |
//! | Live-device requirement | [`RuntimeAdapter::observe_live_device`] | [`LiveDeviceQualification::resolve_live_device_requirements`] |
//! | Prepared-entry property | [`RuntimeAdapter::observe_prepared_entry`] | [`RoutePreparation::resolve_target_properties`] |
//!
//! An adapter that could decide a row's own comparison on its way to an answer
//! would make `Unrecognized`, a wrong-shaped answer, and an unsatisfied
//! requirement one outcome instead of three. The pair comparison and the
//! profile classification stay separate for the same reason: "this host cannot
//! execute these bytes" and "this artifact is for another target" are two
//! refusals with two remedies.
//!
//! # A caller-stated tuple is not discovered device truth
//!
//! [`ExecutionEnvironment`] is what a host *states*, and the device-free
//! [`DecodedProgram::preflight`] and [`DecodedProgram::prepare`] take it as
//! given: a host that states it wrongly gets a wrong answer, and that is the
//! correct division for a loader that binds no device.
//!
//! An adapter has bound one, so the same three identities are no longer a claim
//! — they are an observation, and conflating the two would let a configuration
//! literal drive a route that only a bound device can justify.
//! [`LiveExecutionContext`] is that distinction made structural: it has no
//! public constructor, and the only value of the type in existence is the one
//! [`route_with_adapter`] mints from what an adapter reported when the loader
//! asked it to bind.
//!
//! # Where the payload validation runs, and why it cannot run anywhere else
//!
//! ADR 0090 item 8 is normative on every backend: **a backend validates its own
//! payload from bytes, before the routing commit.** A payload's `code` bytes are
//! opaque to every check the artifact layer performs — the envelope proves
//! framing, digests, schema, canonical order, arena closure, and identity, and
//! none of that says whether the bytes decode into something this backend can
//! execute. So the artifact layer provably cannot discharge it, and
//! [ADR 0051](../../../docs/decisions/0051-make-runtime-routing-commit-one-way.md)
//! forbids selecting another plan after the commit, which leaves an unvalidated
//! payload with nowhere left to fail safely.
//!
//! [`RuntimeAdapter::validate_payload`] is where it runs, and the schedule is
//! fixed by this module rather than left to each adapter: once per routed entry,
//! in execution order, immediately after the loader has routed the entries and
//! published their carried objects, and before the first live-device question,
//! before any preparation, and long before the commit. An adapter that defers it
//! to `dispatch` discovers a malformed payload where nothing may be done about
//! it.
//!
//! # The two device stages are unconditional here
//!
//! ADR 0090 item 9 records that [`DecodedProgram::preflight`] alone is
//! sufficient exactly when the selected variant has zero deferred predicates
//! *and* zero route requirements, and that once a caller enters
//! [`DecodedProgram::prepare`] both device stages are mandatory even when their
//! lists are empty. This route always takes `prepare`.
//!
//! That is not a convenience. An adapter that reached this function has bound a
//! live context, so the device-free path is not the one it is on; and running
//! both stages unconditionally is what stops an adapter writing a device check
//! that only executes when the artifact happens to need it. A route requiring
//! nothing still passes through both, which is a state rather than an absence.

use tiler_artifact::program::{
    AbiFacts, BackendKey, RecordedArtifactProgramIdentity, RepresentationKey, TargetProfileRef,
};

use crate::load::{
    DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceQualification,
    LiveDeviceRequest, LoadRejection, Preflight, RoutePreparation, RoutedDispatch, RoutedEntry,
    TargetPropertyRequest,
};

use std::error::Error;
use std::fmt;

/// The live device and execution context one runtime adapter bound.
///
/// Distinct from [`ExecutionEnvironment`], and the distinction is the whole
/// point of the type. An `ExecutionEnvironment` is a tuple a host *states* about
/// itself and a device-free loader believes. A `LiveExecutionContext` carries
/// the same three governed identities as an adapter's *report* about a context
/// it actually bound.
///
/// # No public constructor, deliberately
///
/// A value of this type cannot be written at a call site. The only one that
/// exists is minted by [`route_with_adapter`] from what
/// [`RuntimeAdapter::bind_execution_context`] returned, so holding one is
/// evidence that a binding call the loader sequenced produced it. That is the
/// enforceable half of "a caller-stated tuple must not masquerade as discovered
/// device truth": what an adapter reports remains the adapter's own claim —
/// trusted, statically linked code under
/// [ADR 0045](../../../docs/decisions/0045-bound-proc-macro-providers-to-host-dependencies.md)'s
/// linkage model — but *when* it is minted is a fact the type system holds.
///
/// # The property is checked by the compiler
///
/// Both examples are compiled by `cargo test` and pin their exact diagnostic, so
/// a change that made either *compile* — or that made it fail for some unrelated
/// reason — fails the gate. Without them the paragraph above would be a claim
/// about the code rather than a fact about it.
///
/// A caller cannot state one, because the field is private (`E0451`):
///
/// ```compile_fail,E0451
/// use tiler_runtime::adapter::LiveExecutionContext;
/// use tiler_runtime::load::ExecutionEnvironment;
///
/// fn state_a_context(stated: ExecutionEnvironment) -> LiveExecutionContext {
///     LiveExecutionContext { observed: stated }
/// }
/// ```
///
/// And a caller cannot promote a stated environment into one, because the only
/// constructor is crate-private (`E0624`):
///
/// ```compile_fail,E0624
/// use tiler_runtime::adapter::LiveExecutionContext;
/// use tiler_runtime::load::ExecutionEnvironment;
///
/// fn promote(stated: ExecutionEnvironment) -> LiveExecutionContext {
///     LiveExecutionContext::from_observation(stated)
/// }
/// ```
///
/// # Why the three identities and nothing else
///
/// These are exactly the subjects the loader compares an artifact against. What
/// else an adapter discovered — a device handle, a queue, a measured
/// floating-point environment — is the adapter's own to keep and never crosses
/// into this crate, which touches no device.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LiveExecutionContext {
    observed: ExecutionEnvironment,
}

impl LiveExecutionContext {
    /// Wraps what an adapter reported about the context it bound.
    ///
    /// Crate-private on purpose: see the type documentation for why no caller
    /// may mint one.
    pub(crate) const fn from_observation(observed: ExecutionEnvironment) -> Self {
        Self { observed }
    }

    /// Returns the observed environment the loader compares against.
    pub(crate) const fn observed(&self) -> &ExecutionEnvironment {
        &self.observed
    }

    /// Returns the target profile the adapter observed, key and exact descriptor.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileRef {
        &self.observed.target_profile
    }

    /// Returns the backend family the adapter observed it can execute.
    #[must_use]
    pub const fn backend(&self) -> &BackendKey {
        &self.observed.backend
    }

    /// Returns the executable representation the adapter observed it can consume.
    #[must_use]
    pub const fn representation(&self) -> &RepresentationKey {
        &self.observed.representation
    }
}

/// One consumer's statically linked runtime adapter for a backend and representation family.
///
/// # What belongs on this side of the seam
///
/// Everything a device is needed for, and nothing a device is not needed for.
/// Device discovery, library and pipeline preparation, storage binding, command
/// encoding, submission, terminal-success observation, and the retention of
/// asynchronous resources through their final device use all live in the
/// implementor. They sit downstream of decoding and validation, which
/// [`tiler_artifact`] and [`crate::load`] have already finished before the first
/// method here is called.
///
/// Nothing in this trait names a device, a queue, a pipeline, a buffer, or any
/// other backend object. An implementor holds those in `Self`; this crate
/// depends on [`tiler_artifact`] alone and stays device-free, which is what
/// makes the loader's half testable on a machine with no accelerator.
///
/// # Two error types, because ADR 0051 draws the line between them
///
/// [`Self::Refusal`] is every pre-commit outcome. A refusal arrives while a
/// fallback is still permitted, nothing irreversible has happened, and the
/// caller may try another artifact.
///
/// [`Self::Failure`] is post-commit and reports rather than retries. Reaching
/// [`Self::allocate_dispatch`] or [`Self::dispatch`] means the route committed,
/// and ADR 0051 forbids selecting another plan afterwards.
/// [`AdapterRouteFailure::fallback_permitted`] is the same split, readable by a
/// caller.
///
/// # Sizing and allocating are two stages, on the two sides of the commit
///
/// ADR 0051 places program allocation after the routing commit: "only the
/// resulting committed execution authority may allocate program resources or
/// encode work", so that "program allocations and partial encodings never
/// precede a fallback decision". [`Self::plan_dispatch`] is therefore the
/// pre-commit half and acquires nothing — it derives each binding's range,
/// compares it against the limits this context declares, and records which
/// storage every slot will take — while [`Self::allocate_dispatch`] is the
/// post-commit half, reached only from a [`RoutedDispatch`], which is the type
/// [`Preflight::commit`] mints.
///
/// The cost of that division is priced rather than hidden: a device that cannot
/// hold the plan is terminal at the allocating stage instead of recoverable.
/// Pre-commit sizing against declared limits catches all but the allocator's own
/// residue, and that residue failing loudly is a defect signal — an allocator
/// returning less than a length it accepted — rather than a routing input.
///
/// # No adapter identity, no version, no capability query
///
/// There is deliberately no `identity()`, `version()`, or `supports()` method.
/// An adapter identity would have to be matched against something an artifact
/// carries, and there is nothing: the artifact records which authority produced
/// a payload as *provenance*, never as a key to match an executor against. A
/// capability vocabulary would duplicate the route-requirement family the
/// artifact layer already governs, and this trait reuses that instead —
/// [`LiveDeviceRequest`] and [`TargetPropertyRequest`] are the only two
/// questions an adapter is asked about what a route needs.
pub trait RuntimeAdapter {
    /// Why this adapter refused a route before it committed.
    ///
    /// A fallback is still permitted for every value of this type.
    type Refusal;

    /// Why a committed dispatch did not complete.
    ///
    /// Reported and never retried on another route: the routing commit is
    /// one-way, and this type names outcomes that are only observable after it.
    type Failure;

    /// What a completed dispatch yields to the caller.
    ///
    /// Owned rather than borrowed from the artifact, so an adapter that returns
    /// read-back results copies them out of its own storage.
    type Completion;

    /// Binds a live device and execution context and reports what it observed.
    ///
    /// Called first, before the artifact is routed, because the identities it
    /// reports are the ones every later comparison is made against. The return
    /// value is a *report*: the loader promotes it into a
    /// [`LiveExecutionContext`] and hands that back to every method below.
    ///
    /// An adapter that cannot bind — no device present, a device that is not the
    /// one its profile describes, a process whose arithmetic it refuses — refuses
    /// here, where no artifact obligation has been decided yet.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Refusal`] when no context could be bound. Nothing has been
    /// routed and a fallback is permitted.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal>;

    /// Validates one routed entry's carried payload from its own bytes.
    ///
    /// **This is the obligation ADR 0090 item 8 places on every backend, and the
    /// module documentation states why nothing else can discharge it.** The
    /// artifact layer proved the object's integrity and carried it opaquely; only
    /// the backend whose representation it is can say whether the bytes decode
    /// into something executable, whether they name the entry symbol the artifact
    /// says they do, and whether the slots they address are the ones the entry
    /// declares.
    ///
    /// Called once per routed entry, in execution order, before the first
    /// live-device question and before any preparation. Every refusal is
    /// therefore an artifact defect reported while a fallback is still permitted.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Refusal`] for a payload this backend will not execute.
    fn validate_payload(
        &mut self,
        context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal>;

    /// Reports what the bound device is for one live-device route requirement.
    ///
    /// **Reports, and does not decide.** The loader compares the answer against
    /// the row and produces the refusal; an adapter that returned a verdict
    /// instead of an observation would be ruling on itself.
    ///
    /// Answer [`LiveDeviceObservation::Unrecognized`] for any key, version, or
    /// payload this adapter does not know exactly. That is fail-closed — the
    /// loader refuses the route — and it is the only correct answer for a row
    /// nothing evaluated. A row owned by another backend never reaches here: the
    /// loader refuses it from the host's own declaration first.
    ///
    /// Answering in the wrong shape — a quantity for a qualitative row, a verdict
    /// for a quantitative one — is refused rather than coerced, because either
    /// coercion would substitute a comparison for the one the row's own kind
    /// fixes.
    fn observe_live_device(
        &mut self,
        context: &LiveExecutionContext,
        request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation;

    /// Prepares every routed entry into whatever executable state this backend needs.
    ///
    /// Runs between the two device stages, because that is when its facts become
    /// true: a live-device fact is readable as soon as a device is bound, and a
    /// prepared-entry fact only once the entry's pipeline exists. A backend with
    /// no preparation step does nothing here and says so by returning `Ok`.
    ///
    /// Reversible. Building a library, a pipeline, or an interpreter image is
    /// state an abandoned route discards, so it is not the program work ADR 0051
    /// puts after the commit.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Refusal`] when an entry could not be prepared — a library
    /// the device rejected, an absent entry symbol, a pipeline that would not
    /// build. Every entry is prepared before the route commits, so a two-entry
    /// route whose *second* entry fails is refused rather than discovered
    /// mid-dispatch.
    fn prepare_entries(
        &mut self,
        context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal>;

    /// Reports one exact prepared entry's target property.
    ///
    /// The request names the entry by its position in the route's execution
    /// order, so the answer comes from *that* entry's prepared state rather than
    /// from a device-wide property that resembles it. The loader holds the
    /// comparison, the threshold, and the direction; this returns the measurement
    /// alone.
    fn observe_prepared_entry(
        &mut self,
        context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> u64;

    /// Sizes what the route will dispatch and checks its capacity, acquiring nothing.
    ///
    /// The last chance to refuse. Called with a [`Preflight`] — every obligation
    /// the loader can decide is already discharged, and the route has not
    /// committed — so an adapter derives each binding's required byte range from
    /// what the route publishes, compares it against the limits this context
    /// declares, resolves which storage every slot will be backed by, and pairs
    /// the slots [`Preflight::shared_allocations`] says must share one
    /// allocation. What it records is a statement about storage, never storage.
    ///
    /// **No program storage is acquired here, and that is ADR 0051 rather than a
    /// style.** A program output, program temporary, validation record, or
    /// private transaction result taken at this stage would precede a fallback
    /// decision the caller is still permitted to make, and abandoning the route
    /// would leave an observable resource effect a retry could duplicate.
    /// Backend-internal library and pipeline state is the reversible kind and
    /// belongs in [`Self::prepare_entries`]; program storage belongs in
    /// [`Self::allocate_dispatch`], after the commit.
    ///
    /// What this stage can still refuse is everything statable without storage:
    /// a byte range whose offset and extent do not form an addressable interval,
    /// a range larger than one allocation this context admits, a launch wider
    /// than the prepared pipeline admits, a binding naming an input the caller
    /// did not supply or a target this consumer does not place, and
    /// caller-supplied storage shorter than the route's published range — the
    /// caller's storage already exists, so comparing against it acquires nothing.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Refusal`] when the route cannot be carried out on this
    /// context. A fallback is still permitted.
    fn plan_dispatch(
        &mut self,
        context: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal>;

    /// Acquires and binds the program storage the **committed** route dispatches.
    ///
    /// Reached only from a [`RoutedDispatch`], and that is the enforcement rather
    /// than the documentation: the only way to obtain one is
    /// [`Preflight::commit`], so an adapter cannot be handed the authority to
    /// allocate program storage before the route has committed.
    ///
    /// An adapter allocates each program output and temporary, honours the
    /// paired [`RoutedDispatch::shared_allocations`] so both ends of a data
    /// dependency address one allocation, fills host-visible inputs, and asserts
    /// that every allocation came back holding the range [`Self::plan_dispatch`]
    /// sized it for.
    ///
    /// **That observed-length assertion is a defect report, not a routing
    /// input.** Every allocation is requested at the length the route states, so
    /// reaching it means an allocator returned less than a request it accepted.
    /// Failing loudly is the signal; refusing recoverably against an allocation
    /// made before the commit is the arrangement this stage replaces.
    ///
    /// Encoding and submission do **not** belong here either. They run in
    /// [`Self::dispatch`].
    ///
    /// # Errors
    ///
    /// Returns [`Self::Failure`]. The route has committed, so an allocation that
    /// fails is reported to the caller and never resolved by routing somewhere
    /// else.
    fn allocate_dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure>;

    /// Encodes, submits, and observes the committed route to terminal success.
    ///
    /// Everything here is past the one-way commit, and the storage it encodes
    /// against was acquired by [`Self::allocate_dispatch`] on the same side of
    /// it. An adapter encodes each entry in the order [`RoutedDispatch::entries`]
    /// publishes, submits, waits for a terminal outcome, reads back what its
    /// completion carries, and keeps every asynchronous resource alive through
    /// its final device use.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Failure`]. There is no fallback after this point: a
    /// failure is reported to the caller, never resolved by routing somewhere
    /// else.
    fn dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure>;
}

/// Routes one decoded artifact through a consumer-selected adapter and dispatches it.
///
/// The whole sequence, in the order the obligations become decidable:
///
/// 1. the adapter binds a live context and reports the identities it observed;
/// 2. the loader compares program identity, selects the variant its guards
///    admit, classifies the variant's declared profile, matches the backend
///    family and representation of every routed entry's payload **as a pair**,
///    classifies that payload's own profile, refuses an execution policy this
///    build cannot deliver, evaluates the launch geometry and the bindings, and
///    derives the storage the entries share;
/// 3. the adapter validates each carried payload from its own bytes;
/// 4. the adapter reports each live-device requirement and the loader compares;
/// 5. the adapter prepares every entry;
/// 6. the adapter reports each prepared-entry property and the loader compares;
/// 7. the adapter sizes the dispatch and checks its capacity, acquiring nothing;
/// 8. the route commits, once and infallibly;
/// 9. the adapter allocates and binds the committed route's program storage;
/// 10. the adapter encodes, submits, and observes terminal success.
///
/// Steps 1 through 7 may refuse and a fallback is permitted throughout. Step 8
/// cannot fail — every decidable obligation was discharged before it — and steps
/// 9 and 10 report rather than retry. That step 9 sits *after* step 8 is ADR
/// 0051's placement of program allocation, and the reason step 7 exists as its
/// own stage: what a device can be asked without acquiring anything is asked
/// while abandoning the route is still free.
///
/// # Errors
///
/// Returns the [`AdapterRouteFailure`] naming the stage that refused or failed.
/// [`AdapterRouteFailure::fallback_permitted`] separates the refusals a caller
/// may still route around from the one failure it may not.
pub fn route_with_adapter<A: RuntimeAdapter>(
    program: &mut DecodedProgram,
    adapter: &mut A,
    expected: &RecordedArtifactProgramIdentity,
    facts: &AbiFacts,
) -> Result<A::Completion, AdapterRouteFailure<A::Refusal, A::Failure>> {
    let context = LiveExecutionContext::from_observation(
        adapter
            .bind_execution_context()
            .map_err(AdapterRouteFailure::Context)?,
    );

    // Every comparison in this call belongs to the loader. The observed
    // environment is an input to it, never a substitute for it.
    let qualification: LiveDeviceQualification<'_> =
        program.prepare(context.observed(), expected, facts)?;

    // ADR 0090 item 8, at the earliest point the object bytes exist and the
    // latest point at which refusing still costs nothing. Per entry, because
    // nothing requires two entries of one variant to be realized by one payload.
    for (position, entry) in qualification.entries().iter().enumerate() {
        adapter
            .validate_payload(&context, entry)
            .map_err(|refusal| AdapterRouteFailure::Payload {
                entry: position,
                refusal,
            })?;
    }

    let preparation: RoutePreparation<'_> =
        qualification.resolve_live_device_requirements(|request| {
            adapter.observe_live_device(&context, request)
        })?;

    adapter
        .prepare_entries(&context, preparation.entries())
        .map_err(AdapterRouteFailure::Preparation)?;

    let preflight: Preflight<'_> = preparation
        .resolve_target_properties(|request| adapter.observe_prepared_entry(&context, request))?;

    adapter
        .plan_dispatch(&context, &preflight)
        .map_err(AdapterRouteFailure::Plan)?;

    // ---- the routing commit, one way ------------------------------------
    // Consuming and infallible. Nothing below may return to a different route,
    // and there is no branch here that could: the only value left is a
    // `RoutedDispatch`, and no variant of it selects a plan.
    let routed = preflight.commit();
    // ADR 0051's "only the resulting committed execution authority may allocate
    // program resources". The authority is the value, so the ordering is carried
    // by the argument type rather than by this line's position.
    adapter
        .allocate_dispatch(&context, &routed)
        .map_err(AdapterRouteFailure::Allocation)?;
    adapter
        .dispatch(&context, &routed)
        .map_err(AdapterRouteFailure::Dispatch)
}

/// Why one artifact did not run to completion through a selected adapter.
///
/// The stage is carried rather than flattened, because "this host cannot execute
/// these bytes", "this payload is malformed", "this device is too small", "the
/// storage the committed route needs could not be acquired", and "the submission
/// did not complete" are five different things to do next, and only the last two
/// foreclose a fallback.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a stage added to the route
/// lands here additively, and no crate outside this one has to match the set
/// completely.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdapterRouteFailure<R, F> {
    /// The loader refused the artifact, with its own classification.
    ///
    /// Carried whole. Every distinction [`LoadRejection`] draws is one this
    /// enum is not a better authority on.
    Load(LoadRejection),
    /// The adapter could not bind a live device and execution context.
    Context(R),
    /// The adapter refused one routed entry's carried payload.
    Payload {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The backend's own account of what it refused.
        refusal: R,
    },
    /// The adapter could not prepare the route's entries.
    Preparation(R),
    /// The adapter could not size what the route dispatches within this context.
    ///
    /// The last refusal before the commit, and the one that costs nothing to
    /// take: nothing was acquired to reach it.
    Plan(R),
    /// The committed route's program storage could not be acquired or bound.
    ///
    /// Past the commit. ADR 0051 places program allocation on this side of the
    /// boundary, which makes a device that cannot hold the plan terminal here
    /// rather than recoverable at [`Self::Plan`].
    Allocation(F),
    /// The committed dispatch did not complete.
    Dispatch(F),
}

impl<R, F> AdapterRouteFailure<R, F> {
    /// Returns whether ADR 0051 still permits this caller to take a fallback.
    ///
    /// True for every refusal reached before the routing commit and false for
    /// [`Self::Allocation`] and [`Self::Dispatch`], the two outcomes reached
    /// after it. The split follows the two associated error types exactly: a
    /// variant carrying `R` is pre-commit and a variant carrying `F` is not.
    ///
    /// Written as an exhaustive match with no wildcard arm (ADR 0074 convention
    /// 3): a stage added to the route must be classified deliberately here, and
    /// a catch-all would classify it as recoverable by default — which is the
    /// answer that turns a post-commit failure into a second route.
    #[must_use]
    pub const fn fallback_permitted(&self) -> bool {
        match self {
            Self::Load(_)
            | Self::Context(_)
            | Self::Payload { .. }
            | Self::Preparation(_)
            | Self::Plan(_) => true,
            Self::Allocation(_) | Self::Dispatch(_) => false,
        }
    }
}

impl<R, F> From<LoadRejection> for AdapterRouteFailure<R, F> {
    fn from(value: LoadRejection) -> Self {
        Self::Load(value)
    }
}

impl<R: fmt::Display, F: fmt::Display> fmt::Display for AdapterRouteFailure<R, F> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(rejection) => write!(formatter, "adapter.load: {rejection}"),
            Self::Context(refusal) => write!(
                formatter,
                "adapter.context: the adapter bound no live execution context: {refusal}",
            ),
            Self::Payload { entry, refusal } => write!(
                formatter,
                "adapter.payload: the backend refused entry {entry}'s carried payload: {refusal}",
            ),
            Self::Preparation(refusal) => write!(
                formatter,
                "adapter.preparation: the adapter prepared no executable entry: {refusal}",
            ),
            Self::Plan(refusal) => write!(
                formatter,
                "adapter.plan: the adapter cannot carry out this route: {refusal}",
            ),
            Self::Allocation(failure) => write!(
                formatter,
                "adapter.allocation: the committed route's storage could not be acquired, and no \
                 fallback follows: {failure}",
            ),
            Self::Dispatch(failure) => write!(
                formatter,
                "adapter.dispatch: the committed route did not complete, and no fallback follows: \
                 {failure}",
            ),
        }
    }
}

impl<R, F> Error for AdapterRouteFailure<R, F>
where
    R: Error + 'static,
    F: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Load(rejection) => Some(rejection),
            Self::Context(refusal)
            | Self::Payload { refusal, .. }
            | Self::Preparation(refusal)
            | Self::Plan(refusal) => Some(refusal),
            Self::Allocation(failure) | Self::Dispatch(failure) => Some(failure),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AdapterRouteFailure;
    use crate::load::{DecodedProgram, LoadRejection};
    use tiler_artifact::program::ArtifactCodecFailure;

    /// Every stage reached before the commit permits a fallback, and the two
    /// stages after it do not.
    ///
    /// Asserted over a written-out population rather than over whatever the enum
    /// happens to hold, so a stage added without being classified is visible as a
    /// missing row here as well as a build error in `fallback_permitted`.
    #[test]
    fn no_post_commit_stage_permits_a_fallback_and_every_pre_commit_one_does() {
        let load: AdapterRouteFailure<&str, &str> = AdapterRouteFailure::Load(
            LoadRejection::Artifact(match DecodedProgram::decode(b"short", 0) {
                Err(LoadRejection::Artifact(failure)) => failure,
                other => panic!("five bytes are not an artifact: {other:?}"),
            }),
        );
        let recoverable: [AdapterRouteFailure<&str, &str>; 5] = [
            load,
            AdapterRouteFailure::Context("no device"),
            AdapterRouteFailure::Payload {
                entry: 0,
                refusal: "truncated image",
            },
            AdapterRouteFailure::Preparation("no pipeline"),
            AdapterRouteFailure::Plan("storage too small"),
        ];
        for failure in &recoverable {
            assert!(
                failure.fallback_permitted(),
                "{failure:?} is reached before the commit and must permit a fallback",
            );
        }
        let committed: [AdapterRouteFailure<&str, &str>; 2] = [
            AdapterRouteFailure::Allocation("the device returned no buffer"),
            AdapterRouteFailure::Dispatch("submission did not complete"),
        ];
        for failure in &committed {
            assert!(
                !failure.fallback_permitted(),
                "{failure:?} is reached after the commit and must foreclose a fallback",
            );
        }
    }

    /// The display form names the stage and keeps the cause readable.
    #[test]
    fn each_stage_is_distinguishable_in_a_report() {
        let rendered: Vec<String> = [
            AdapterRouteFailure::<&str, &str>::Context("cause"),
            AdapterRouteFailure::Payload {
                entry: 0,
                refusal: "cause",
            },
            AdapterRouteFailure::Preparation("cause"),
            AdapterRouteFailure::Plan("cause"),
            AdapterRouteFailure::Allocation("cause"),
            AdapterRouteFailure::Dispatch("cause"),
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                text.contains("cause"),
                "{text:?} dropped the cause it classifies",
            );
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier stage",
            );
        }
    }

    /// A codec failure stays reachable through the whole failure chain.
    #[test]
    fn a_load_rejection_survives_the_adapter_classification() {
        let rejection = DecodedProgram::decode(&[], 0).expect_err("no bytes, no artifact");
        let failure: AdapterRouteFailure<std::io::Error, std::io::Error> = rejection.into();
        assert!(
            matches!(
                failure,
                AdapterRouteFailure::Load(LoadRejection::Artifact(
                    ArtifactCodecFailure::Malformed { .. }
                )),
            ),
            "the codec's own classification must survive: {failure:?}",
        );
    }
}
