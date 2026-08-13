//! The preflight stage and the one-way routing commit.
//!
//! # Preparation and commit are distinct types, so the order is not a convention
//!
//! ADR 0051 requires routing to commit one way, before program work, and
//! forbids falling back after it. That is enforced here by construction rather
//! than by documentation. Every obligation that can refuse lives in [`super::DecodedProgram::preflight`] or in the consuming [`LiveDeviceQualification::resolve_live_device_requirements`] and [`RoutePreparation::resolve_target_properties`] path; both routes yield a [`Preflight`] only after every requirement holds. [`Preflight::commit`] consumes that value and is **infallible**.
//!
//! The device path is two consuming stages rather than one because its
//! obligations become answerable at two different moments: a live-device
//! requirement as soon as a device is bound, and a prepared-entry property only
//! once its pipeline exists. Chaining them by type is also what makes the first
//! unskippable — a [`RoutePreparation`] can only have come from a resolved
//! [`LiveDeviceQualification`], including for a route that requires nothing.
//!
//! A caller that wants a fallback takes it by not calling [`Preflight::commit`],
//! which is exactly ADR 0051's "fallback only before program work".
//!
//! # What a committed route names
//!
//! Everything one dispatch needs, all of it read from the artifact's own bytes.
//! A [`RoutedDispatch`] names the carried object, the descriptor identifying it,
//! the backend entry symbol to look up inside it, the evaluated launch geometry,
//! and — per ABI slot — the backend transport it occupies, what it addresses,
//! and how many bytes must be reachable through it.
//!
//! **Four claims this module previously made are retracted.** It stated that a
//! committed route "does not name an entry symbol, a binding-to-buffer
//! correspondence, or an evaluated launch extent, because a decoded envelope
//! publishes none of those", and concluded that "a caller that does not hold the
//! program it compiled cannot dispatch from an artifact alone". All four were
//! true when written and none is true now:
//! [`DecodedEntry::backend_symbol`](tiler_artifact::program::DecodedEntry::backend_symbol),
//! [`DecodedEntry::transport_slots`](tiler_artifact::program::DecodedEntry::transport_slots),
//! [`DecodedBinding::target`](tiler_artifact::program::DecodedBinding::target),
//! and `DecodedExpr::evaluate` publish exactly those facts, and this module
//! routes through them.
//!
//! # Why the two stages publish different things
//!
//! A [`Preflight`] publishes what a caller *judges*: the identity, the
//! descriptor, the geometry, and the bindings. Those decide whether to commit at
//! all — a launch wider than the host's storage is a reason to abandon this
//! route, and abandoning is only permitted while this value is still held.
//!
//! The object bytes and the entry symbol are published only by
//! [`RoutedDispatch`], because they are what a caller *executes*. Reaching them
//! should require having made the decision rather than merely having considered
//! it, which makes "no program work before the commit" a property of the type
//! rather than a rule to remember.

use tiler_artifact::program::{
    ArtifactExecutionPolicy, BackendPayloadDescriptor, CanonicalArtifactProgramIdentity,
    DecodedBinding, DecodedEntry, PreparedEntryTargetRequirement, RouteRequirement,
    TargetPropertyRequirementRelation,
};

use super::LoadRejection;
use std::fmt;

/// The evaluated launch geometry of one routed entry.
///
/// Scalars rather than expressions. The artifact carries formulas over its own
/// interface, and they are evaluated against the facts the host bound during
/// preflight — the only point at which an evaluation failure can still be
/// reported as a refusal instead of arriving after the routing commit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoutedLaunch {
    pub(super) grid_threads: u64,
    pub(super) threads_per_workgroup: u64,
    pub(super) zero_work_skips_dispatch: bool,
}

impl RoutedLaunch {
    /// Returns the total number of threads this launch covers.
    #[must_use]
    pub const fn grid_threads(self) -> u64 {
        self.grid_threads
    }

    /// Returns the number of threads in one workgroup.
    #[must_use]
    pub const fn threads_per_workgroup(self) -> u64 {
        self.threads_per_workgroup
    }

    /// Returns whether a zero-thread launch is skipped rather than encoded.
    ///
    /// Returned rather than assumed. Encoding a zero-thread dispatch against a
    /// backend that refuses one would turn a well-formed empty launch into a
    /// submission failure.
    #[must_use]
    pub const fn zero_work_skips_dispatch(self) -> bool {
        self.zero_work_skips_dispatch
    }
}

/// **Accepted public surface.** Tom accepted this exact spelling on
/// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
/// Dependents may treat this type as accepted vocabulary.
///
/// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
///
/// One live input-extent parameter frozen before [`Preflight::commit`].
///
/// The committed authority owns these bytes. A backend binds exactly the
/// declared transport; it does not re-evaluate the fact.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoutedExtentParameter {
    pub(super) transport: u32,
    pub(super) value: u64,
}

impl RoutedExtentParameter {
    /// Returns the backend transport index this scalar occupies.
    #[must_use]
    pub const fn transport_slot(self) -> u32 {
        self.transport
    }

    /// Returns the frozen unsigned extent value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.value
    }

    /// Returns the canonical parameter bytes the backend binds.
    #[must_use]
    pub fn parameter_bytes(self) -> [u8; 8] {
        self.value.to_le_bytes()
    }
}

/// One ABI binding of a routed entry: where it goes, and what byte range it reaches.
///
/// The two facts the loader *derived* — the backend transport slot and the
/// evaluated range start and extent — are published beside the decoded binding
/// they came from rather than instead of it. A host needing the element type,
/// address space, or access mode reads them through [`Self::binding`]. Naming
/// those types here would give this crate a direct `tiler-ir` edge, and its
/// dependency closure is a decided property under ADR 0081 rather than an
/// accident of ordering.
#[derive(Clone, Copy, Debug)]
pub struct RoutedBinding<'a> {
    pub(super) binding: DecodedBinding<'a>,
    pub(super) transport: u32,
    pub(super) accessible_offset: u64,
    pub(super) accessible_bytes: u64,
}

impl<'a> RoutedBinding<'a> {
    /// Returns the zero-based ABI slot, in the kernel signature's own order.
    #[must_use]
    pub fn slot(self) -> usize {
        self.binding.slot()
    }

    /// Returns the backend transport index this slot occupies.
    ///
    /// Deliberately not the same number as [`Self::slot`]: an artifact orders
    /// its bindings by the kernel signature, and a backend places them wherever
    /// its own argument table says. Collapsing the two would bind the right
    /// storage to the wrong index on any backend whose mapping is not the
    /// identity.
    #[must_use]
    pub const fn transport_slot(self) -> u32 {
        self.transport
    }

    /// Returns the first addressed byte of the bound value.
    ///
    /// A host binds the storage at this byte rather than assuming that every
    /// slot addresses its value whole. Together with [`Self::accessible_bytes`]
    /// this is the exact range the entry may reach.
    #[must_use]
    pub const fn accessible_offset(self) -> u64 {
        self.accessible_offset
    }

    /// Returns the minimum number of bytes reachable through this binding.
    ///
    /// Counted from [`Self::accessible_offset`], not from the start of the
    /// addressed value. Evaluated from the artifact's own accessible-range
    /// expression against the facts the host bound, so a host compares it
    /// against the storage it holds rather than re-deriving an extent the
    /// artifact already derived.
    #[must_use]
    pub const fn accessible_bytes(self) -> u64 {
        self.accessible_bytes
    }

    /// Returns the decoded binding this slot was routed from.
    ///
    /// Carries what the slot addresses — a named program input, a named program
    /// output, or entry-internal storage — with every declared storage fact the
    /// artifact records about it.
    #[must_use]
    pub const fn binding(self) -> DecodedBinding<'a> {
        self.binding
    }
}

/// One entry of a routed variant, with everything its dispatch needs.
///
/// A route carries one of these per stage, in execution order. Every fact here
/// is per *entry* rather than per route, and that is not uniformity for its own
/// sake: nothing requires two entries of one variant to be realized by the same
/// payload, so the object, the symbol, and the descriptor are resolved and
/// checked for each. A loader that validated one entry's payload and executed
/// another's would be routing on a fact it never checked.
#[derive(Clone, Debug)]
pub struct RoutedEntry<'a> {
    pub(super) payload: &'a BackendPayloadDescriptor,
    pub(super) object: &'a [u8],
    pub(super) entry: DecodedEntry<'a>,
    pub(super) symbol: &'a str,
    pub(super) launch: RoutedLaunch,
    pub(super) bindings: Vec<RoutedBinding<'a>>,
    pub(super) extent_parameters: Vec<RoutedExtentParameter>,
}

impl<'a> RoutedEntry<'a> {
    /// Returns the descriptor of the payload realizing this entry.
    #[must_use]
    pub const fn payload(&self) -> &'a BackendPayloadDescriptor {
        self.payload
    }

    /// Returns the exact emitted object bytes this entry executes from.
    #[must_use]
    pub const fn object(&self) -> &'a [u8] {
        self.object
    }

    /// Returns the backend's own entry-point symbol to look up in that object.
    #[must_use]
    pub const fn entry_symbol(&self) -> &'a str {
        self.symbol
    }

    /// Returns the evaluated launch geometry this entry encodes.
    #[must_use]
    pub const fn launch(&self) -> RoutedLaunch {
        self.launch
    }

    /// Returns this entry's routed ABI bindings in the kernel signature's order.
    #[must_use]
    pub fn bindings(&self) -> &[RoutedBinding<'a>] {
        &self.bindings
    }

    /// **Accepted public surface.** Tom accepted this exact spelling on
    /// 2026-08-13 under [`accept-the-live-extent-operand-public-surface`].
    ///
    /// [`accept-the-live-extent-operand-public-surface`]: ../../../../../tickets/accept-the-live-extent-operand-public-surface.md
    ///
    /// Returns the live input-extent parameters frozen from the same
    /// [`tiler_artifact::program::AbiFacts`] used to evaluate ranges and launch.
    ///
    /// Canonical declaration order, which is the scalar transport order after
    /// the buffer table. The bound *value* is here; it is not part of artifact,
    /// payload, library, or pipeline identity.
    #[must_use]
    pub fn extent_parameters(&self) -> &[RoutedExtentParameter] {
        &self.extent_parameters
    }

    /// Returns the decoded entry this was routed from.
    #[must_use]
    pub const fn entry(&self) -> DecodedEntry<'a> {
        self.entry
    }
}

/// Two ABI slots of two entries that must be backed by **one** allocation.
///
/// # Why a loader cannot work this out for itself
///
/// A binding addressing entry-internal storage carries no name — two `Internal`
/// slots are indistinguishable by design, because the artifact layer has no
/// durable name for a program value. So a loader allocating per binding gives
/// the consumer a *fresh* buffer, the producer's result never reaches it, and
/// the dispatch reads uninitialised device memory. That is a wrong answer rather
/// than a refusal, and it is the one place in this stack that would fail open.
///
/// The pairing is derived from the variant's own typed data dependencies, so it
/// states what the packaged program proved rather than what a loader guessed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedAllocation {
    pub(super) producer: EntrySlot,
    pub(super) consumer: EntrySlot,
}

impl SharedAllocation {
    /// Returns the entry and slot that writes the shared storage.
    #[must_use]
    pub const fn producer(self) -> EntrySlot {
        self.producer
    }

    /// Returns the entry and slot that reads it.
    #[must_use]
    pub const fn consumer(self) -> EntrySlot {
        self.consumer
    }
}

/// One ABI slot of one entry, both indices into the route's own execution order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntrySlot {
    pub(super) entry: usize,
    pub(super) slot: usize,
}

#[derive(Debug)]
pub(super) struct RouteCandidate<'a> {
    pub(super) identity: CanonicalArtifactProgramIdentity,
    pub(super) kernel_program: &'a [u8],
    pub(super) entries: Vec<RoutedEntry<'a>>,
    pub(super) shared: Vec<SharedAllocation>,
}

/// One exact prepared-entry property the host must acquire before routing can commit.
#[derive(Clone, Copy, Debug)]
pub struct TargetPropertyRequest<'a> {
    pub(super) variant: usize,
    pub(super) predicate: usize,
    pub(super) entry: usize,
    pub(super) requirement: &'a PreparedEntryTargetRequirement,
}

impl<'a> TargetPropertyRequest<'a> {
    /// Returns the selected variant's routing rank.
    #[must_use]
    pub const fn variant(self) -> usize {
        self.variant
    }

    /// Returns the predicate's position within the selected variant.
    #[must_use]
    pub const fn predicate(self) -> usize {
        self.predicate
    }

    /// Returns the exact prepared entry's position in execution order.
    #[must_use]
    pub const fn entry(self) -> usize {
        self.entry
    }

    /// Returns the complete query, threshold, and directional relation.
    #[must_use]
    pub const fn requirement(self) -> &'a PreparedEntryTargetRequirement {
        self.requirement
    }

    /// Returns the owned subject this request asked about.
    pub(super) fn property_subject(self) -> PreparedEntryPropertySubject {
        let query = self.requirement.query();
        let provider = query.provider();
        PreparedEntryPropertySubject {
            key: query.key().as_str().to_owned(),
            provider_namespace: provider.namespace().to_owned(),
            provider_name: provider.name().to_owned(),
            provider_revision: provider.revision(),
            required: self.requirement.required(),
            relation: self.requirement.relation(),
        }
    }
}

/// One additional requirement the selected route places on the live device.
#[derive(Clone, Copy, Debug)]
pub struct LiveDeviceRequest<'a> {
    pub(super) variant: usize,
    pub(super) position: usize,
    pub(super) requirement: &'a RouteRequirement,
}

impl<'a> LiveDeviceRequest<'a> {
    /// Returns the selected variant's routing rank.
    #[must_use]
    pub const fn variant(self) -> usize {
        self.variant
    }

    /// Returns the requirement's position among the variant's own rows.
    #[must_use]
    pub const fn position(self) -> usize {
        self.position
    }

    /// Returns the complete requirement the device must satisfy.
    #[must_use]
    pub const fn requirement(self) -> &'a RouteRequirement {
        self.requirement
    }
}

/// What a host answers about one prepared-entry target property.
///
/// Two answers rather than a number, because "I do not own this property" is
/// not the same as an observed quantity. Collapsing the first into a numeric
/// sentinel would let an unknown key compare equal to a required value;
/// collapsing it into satisfaction would route on a property nothing evaluated.
///
/// The comparison stays in this crate: a host reports what it measured and the
/// loader applies the relation the requirement carries, so an adapter cannot
/// decide a row's own comparison on its way to an answer.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): an answer
/// added here changes what a host must be able to say, and that must stop each
/// host's build rather than reach a wildcard.
///
/// # Public boundary status
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-13
/// under [`accept-the-prepared-entry-observation-surface`]. The type is `pub`
/// so its shape can be reviewed as a whole.
///
/// [`accept-the-prepared-entry-observation-surface`]: ../../../../../tickets/accept-the-prepared-entry-observation-surface.md
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PreparedEntryObservation {
    /// The host measured this property on the named prepared entry.
    ///
    /// A measurement and not a verdict. The loader compares it against the
    /// request's required quantity by the relation the requirement carries.
    Quantity(u64),
    /// No adapter on this host owns or understands the property.
    ///
    /// The fail-closed answer, and the one a host must give for a provider
    /// namespace, name, revision, or property key it does not recognize
    /// exactly. A property nothing can decide is a refusal, never a quantity.
    Unrecognized,
}

/// The prepared-entry property one observation answered or failed to own.
///
/// Distinct from the observation itself: the subject is what was asked, and
/// the observation is what the adapter reported. Owned, because a refusal
/// outlives the artifact borrow that produced it.
///
/// # Public boundary status
///
/// **Accepted public surface.** Tom accepted this exact spelling on 2026-08-13
/// under [`accept-the-prepared-entry-observation-surface`], with
/// [`PreparedEntryObservation`].
///
/// [`accept-the-prepared-entry-observation-surface`]: ../../../../../tickets/accept-the-prepared-entry-observation-surface.md
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PreparedEntryPropertySubject {
    /// Governed property key the request named.
    pub key: String,
    /// Provider namespace the request named.
    pub provider_namespace: String,
    /// Provider name the request named.
    pub provider_name: String,
    /// Nonzero provider revision the request named.
    pub provider_revision: u32,
    /// Required quantity the loader compares, or would have compared.
    pub required: u64,
    /// Directional relation the loader applies to a quantity.
    pub relation: TargetPropertyRequirementRelation,
}

impl fmt::Display for PreparedEntryPropertySubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} from {}::{}@{} required {} {}",
            self.key,
            self.provider_namespace,
            self.provider_name,
            self.provider_revision,
            self.required,
            match self.relation {
                TargetPropertyRequirementRelation::ObservedAtLeastRequired => {
                    "observed-at-least-required"
                }
                TargetPropertyRequirementRelation::ObservedEqualsRequired => {
                    "observed-equals-required"
                }
                TargetPropertyRequirementRelation::RequiredImpliesObserved => {
                    "required-implies-observed"
                }
            },
        )
    }
}

/// What a host answers about one live-device route requirement.
///
/// Three answers rather than a boolean, because "I do not own this row" is not
/// the same as "this device does not satisfy it" and neither is the same as a
/// measurement. Collapsing the first into the second would report an adapter gap
/// as a device limitation; collapsing it into satisfaction would route on a
/// requirement nothing evaluated.
///
/// The comparison for a quantitative row stays in this crate: a host reports
/// what it measured and the loader applies the relation that row's dimension
/// fixes, so an adapter cannot decide a row's own comparison on its way to an
/// answer.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): an answer
/// added here changes what a host must be able to say, and that must stop each
/// host's build rather than reach a wildcard.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LiveDeviceObservation {
    /// The host measured this dimension on the bound device.
    ///
    /// A measurement and not a verdict, and valid only for a
    /// [`RouteRequirement::Resource`] row: the loader compares it against that
    /// row's required quantity by the relation the row's dimension fixes.
    /// Answering it for a backend feature row is refused rather than coerced.
    Quantity(u64),
    /// The owning adapter decided this qualitative row for the bound device.
    ///
    /// Valid only for a backend feature row.
    Feature(bool),
    /// No adapter on this host owns or understands the row.
    ///
    /// The fail-closed answer, and the one a host must give for an owner, key,
    /// version, or payload it does not recognize. A requirement nothing can
    /// decide is a refusal, never a row to skip.
    Unrecognized,
}

/// A routed candidate awaiting the live-device facts its route requires.
///
/// # Why this is a stage rather than a method
///
/// Live-device facts are available before a pipeline exists, so they are decided
/// first — and, more importantly, they are decided *at all*. A host that reached
/// [`RoutePreparation`] has already passed through here, so there is no path on
/// which a route requirement goes unevaluated. A variant with no rows still
/// passes through this stage, which is what stops a host writing a device check
/// that only runs when it happens to be needed.
///
/// Routing cannot commit from here:
///
/// ```compile_fail
/// fn commit_early(qualification: tiler_runtime::load::LiveDeviceQualification<'_>) {
///     let _ = qualification.commit();
/// }
/// ```
#[derive(Debug)]
#[must_use = "a qualification that is neither resolved nor abandoned decides nothing"]
pub struct LiveDeviceQualification<'a> {
    pub(super) candidate: RouteCandidate<'a>,
    pub(super) requirements: Vec<LiveDeviceRequest<'a>>,
    pub(super) requests: Vec<TargetPropertyRequest<'a>>,
}

impl<'a> LiveDeviceQualification<'a> {
    /// Returns the identity of the artifact this qualification would execute.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.candidate.identity
    }

    /// Returns the canonical identity of the kernel program it would run.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.candidate.kernel_program
    }

    /// Returns the entries this route would dispatch, in execution order.
    ///
    /// Published before the device facts are answered so a host can size or
    /// inspect the route while deciding, and *only* inspect it: nothing
    /// irreversible is reachable from here.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.candidate.entries
    }

    /// Returns every live-device requirement of the selected route.
    ///
    /// Empty for a route that requires nothing additional, which is a state
    /// rather than an absence.
    #[must_use]
    pub fn live_device_requirements(
        &self,
    ) -> impl ExactSizeIterator<Item = LiveDeviceRequest<'a>> + '_ {
        self.requirements.iter().copied()
    }

    /// Resolves every live-device requirement exactly once, or refuses the route.
    ///
    /// Each row is passed to `resolve` once, in the artifact's canonical order,
    /// and the answer is checked against the row's own kind before it is
    /// believed.
    ///
    /// # Errors
    ///
    /// Returns [`LoadRejection::UnownedRouteRequirement`] for a row no adapter
    /// claimed, [`LoadRejection::MisansweredRouteRequirement`] for an answer
    /// whose shape disagrees with the row's kind, or
    /// [`LoadRejection::UnsatisfiedRouteRequirement`] for a row the device does
    /// not satisfy — each before anything is prepared or committed.
    pub fn resolve_live_device_requirements(
        self,
        mut resolve: impl FnMut(LiveDeviceRequest<'a>) -> LiveDeviceObservation,
    ) -> Result<RoutePreparation<'a>, LoadRejection> {
        for request in &self.requirements {
            let request = *request;
            let refusal = |kind| LoadRejection::from_request(kind, request);
            // Exhaustive on both axes rather than matching the satisfying cases
            // and defaulting the rest: a kind or an answer added later must stop
            // this build instead of falling into a branch that refuses — or,
            // worse, accepts — for a reason nobody chose.
            let satisfied = match (request.requirement, resolve(request)) {
                (
                    RouteRequirement::Resource(resource),
                    LiveDeviceObservation::Quantity(observed),
                ) => resource.is_satisfied_by(observed),
                (
                    RouteRequirement::BackendFeature(_),
                    LiveDeviceObservation::Feature(supported),
                ) => supported,
                (_, LiveDeviceObservation::Unrecognized) => {
                    return Err(refusal(RouteRequirementRefusal::Unowned));
                }
                (RouteRequirement::Resource(_), LiveDeviceObservation::Feature(_))
                | (RouteRequirement::BackendFeature(_), LiveDeviceObservation::Quantity(_)) => {
                    return Err(refusal(RouteRequirementRefusal::Misanswered));
                }
            };
            if !satisfied {
                return Err(refusal(RouteRequirementRefusal::Unsatisfied));
            }
        }
        Ok(RoutePreparation {
            candidate: self.candidate,
            requests: self.requests,
        })
    }
}

/// Which way one live-device requirement failed to decide in the route's favour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RouteRequirementRefusal {
    /// No adapter on the host owned the row.
    Unowned,
    /// The answer's shape disagreed with the row's kind.
    Misanswered,
    /// The device does not satisfy the row.
    Unsatisfied,
}

/// A fully routed candidate awaiting exact properties from its prepared entries.
///
/// Reached only through [`LiveDeviceQualification`], so every live-device
/// requirement of this route has already been decided.
///
/// Routing cannot commit while any prepared-entry property remains unanswered:
///
/// ```compile_fail
/// fn commit_early(preparation: tiler_runtime::load::RoutePreparation<'_>) {
///     let _ = preparation.commit();
/// }
/// ```
#[derive(Debug)]
#[must_use = "a preparation that is neither resolved nor abandoned decides nothing"]
pub struct RoutePreparation<'a> {
    pub(super) candidate: RouteCandidate<'a>,
    pub(super) requests: Vec<TargetPropertyRequest<'a>>,
}

impl<'a> RoutePreparation<'a> {
    /// Returns the identity of the artifact this preparation would execute.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.candidate.identity
    }

    /// Returns the canonical identity of the kernel program this preparation would run.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.candidate.kernel_program
    }

    /// Returns the entries whose exact executable pipelines must be prepared.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.candidate.entries
    }

    /// Returns every exact-entry property request in predicate order.
    #[must_use]
    pub fn target_property_requests(
        &self,
    ) -> impl ExactSizeIterator<Item = TargetPropertyRequest<'a>> + '_ {
        self.requests.iter().copied()
    }

    /// Resolves every request exactly once and yields a committable route only when all hold.
    ///
    /// Each request is passed to `resolve` once, in predicate order. The
    /// adapter reports an observation; this method holds the comparison, the
    /// threshold, and the direction.
    ///
    /// # Errors
    ///
    /// Returns [`LoadRejection::UnownedPreparedEntryProperty`] for a property no
    /// adapter claimed, or [`LoadRejection::UnsatisfiedDeferredPredicate`] for
    /// the first observed quantity that does not satisfy its retained relation.
    pub fn resolve_target_properties(
        self,
        mut resolve: impl FnMut(TargetPropertyRequest<'a>) -> PreparedEntryObservation,
    ) -> Result<Preflight<'a>, LoadRejection> {
        for request in self.requests {
            // Exhaustive on the observation rather than matching the satisfying
            // case and defaulting the rest: an answer added later must stop this
            // build instead of falling into a branch that refuses — or, worse,
            // accepts — for a reason nobody chose.
            match resolve(request) {
                PreparedEntryObservation::Unrecognized => {
                    return Err(LoadRejection::unowned_prepared_entry(request));
                }
                PreparedEntryObservation::Quantity(observed) => {
                    if !request.requirement.is_satisfied_by(observed) {
                        return Err(LoadRejection::unsatisfied_prepared_entry(request, observed));
                    }
                }
            }
        }
        Ok(Preflight::from_candidate(self.candidate))
    }
}

impl EntrySlot {
    /// Returns the position of the entry in the route's execution order.
    #[must_use]
    pub const fn entry(self) -> usize {
        self.entry
    }

    /// Returns the zero-based ABI slot within that entry.
    #[must_use]
    pub const fn slot(self) -> usize {
        self.slot
    }
}

/// One artifact that passed every obligation this loader can decide.
///
/// Deliberately neither [`Clone`] nor [`Copy`]. A route that could be duplicated
/// could be committed twice, and "committed once" is the property ADR 0051
/// asks for.
#[derive(Debug)]
#[must_use = "a preflight that is neither committed nor abandoned decides nothing"]
pub struct Preflight<'a> {
    pub(super) identity: CanonicalArtifactProgramIdentity,
    pub(super) kernel_program: &'a [u8],
    pub(super) entries: Vec<RoutedEntry<'a>>,
    pub(super) shared: Vec<SharedAllocation>,
}

impl<'a> Preflight<'a> {
    pub(super) fn from_candidate(candidate: RouteCandidate<'a>) -> Self {
        let RouteCandidate {
            identity,
            kernel_program,
            entries,
            shared,
        } = candidate;
        Self {
            identity,
            kernel_program,
            entries,
            shared,
        }
    }
    /// Returns the identity of the artifact this route would execute.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the canonical identity of the kernel program this route runs.
    ///
    /// The identity alone; the program is not carried and cannot be rebuilt
    /// from an envelope. It is published before the commit because it is the
    /// strongest binding available to a caller that *does* hold the program it
    /// compiled: comparing it proves these bytes package that exact program,
    /// which no artifact identity from a sidecar can establish. A caller that
    /// holds no program ignores it, and has correspondingly less evidence.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.kernel_program
    }

    /// Returns this route's entries **in the order they must be dispatched**.
    ///
    /// The variant's own execution order, not the entry table's canonical
    /// stage-key order. A caller dispatches this sequence front to back.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.entries
    }

    /// Returns the slot pairs that must be backed by one allocation each.
    ///
    /// Empty for a single-entry route. See [`SharedAllocation`] for why a loader
    /// cannot derive these from the bindings alone.
    #[must_use]
    pub fn shared_allocations(&self) -> &[SharedAllocation] {
        &self.shared
    }

    /// Commits to executing this route. One way, and infallible.
    ///
    /// There is no `Result` here on purpose. Every decidable obligation was
    /// discharged by the preflight that produced this value, so a failure at
    /// this point would mean an obligation was checked in the wrong stage.
    /// Consuming `self` is what makes the commit one-way: the caller cannot
    /// afterwards hold this value to fall back to.
    ///
    /// # The one-way property is checked by the compiler
    ///
    /// The three examples below are the evidence that ADR 0051's commit is
    /// structural here rather than a rule a caller is trusted to follow. Each
    /// is compiled by `cargo test`; the two negative ones pin the exact
    /// diagnostic, so a change that made either *compile* — or that made it
    /// fail for some unrelated reason — fails the gate.
    ///
    /// Committing once is the whole of what a caller may do with this value:
    ///
    /// ```
    /// use tiler_runtime::load::{Preflight, RoutedDispatch};
    ///
    /// fn route(preflight: Preflight<'_>) -> RoutedDispatch<'_> {
    ///     preflight.commit()
    /// }
    /// ```
    ///
    /// Committing a second time does not compile, because the first commit
    /// moved the value the second one would need (`E0382`). This is what
    /// "committed once" means:
    ///
    /// ```compile_fail,E0382
    /// use tiler_runtime::load::Preflight;
    ///
    /// fn commit_twice(preflight: Preflight<'_>) {
    ///     let _first = preflight.commit();
    ///     let _second = preflight.commit();
    /// }
    /// ```
    ///
    /// Keeping a spare to fall back to after committing does not compile
    /// either, because [`Preflight`] is deliberately not [`Clone`] (`E0277`).
    /// Without that, a caller could duplicate the route, commit one copy, and
    /// still hold an uncommitted one — which is exactly the state the commit
    /// exists to make unreachable:
    ///
    /// ```compile_fail,E0277
    /// use tiler_runtime::load::Preflight;
    ///
    /// fn duplicate<T: Clone>(value: T) -> (T, T) {
    ///     (value.clone(), value)
    /// }
    ///
    /// fn keep_a_fallback(preflight: Preflight<'_>) {
    ///     let (_spare, _route) = duplicate(preflight);
    /// }
    /// ```
    ///
    /// # Neither can a second authority be minted
    ///
    /// The three examples above all start from a `Preflight` a caller already
    /// holds, so on their own they prove only that *one* authority is
    /// single-use. They were the whole of the evidence until
    /// `make-runtime-routing-commit-authority-one-shot`, and they left the real
    /// hole open: a caller could mint a second authority from the program and
    /// commit that instead.
    ///
    /// Holding a committed route keeps the program exclusively borrowed, so
    /// preflighting it again does not compile (`E0499`):
    ///
    /// ```compile_fail,E0499
    /// use tiler_artifact::program::{AbiFacts, RecordedArtifactProgramIdentity};
    /// use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment};
    ///
    /// fn commit_then_mint_another(
    ///     program: &mut DecodedProgram,
    ///     environment: &ExecutionEnvironment,
    ///     expected: &RecordedArtifactProgramIdentity,
    ///     facts: &AbiFacts,
    /// ) {
    ///     let route = program.preflight(environment, expected, facts).unwrap().commit();
    ///     let _second = program.preflight(environment, expected, facts);
    ///     let _still_held = route;
    /// }
    /// ```
    ///
    /// And the program cannot be duplicated to escape that borrow, because
    /// [`DecodedProgram`] is deliberately not [`Clone`] (`E0277`):
    ///
    /// ```compile_fail,E0277
    /// use tiler_runtime::load::DecodedProgram;
    ///
    /// fn duplicate<T: Clone>(value: T) -> (T, T) {
    ///     (value.clone(), value)
    /// }
    ///
    /// fn two_programs_one_artifact(program: DecodedProgram) {
    ///     let (_spare, _original) = duplicate(program);
    /// }
    /// ```
    ///
    /// [`DecodedProgram`]: super::DecodedProgram
    #[must_use]
    pub fn commit(self) -> RoutedDispatch<'a> {
        let Self {
            identity,
            kernel_program,
            entries,
            shared,
        } = self;
        RoutedDispatch {
            identity,
            kernel_program,
            entries,
            shared,
        }
    }
}

/// A committed route: every entry, in dispatch order, and what each one needs.
///
/// Reaching this type is the boundary ADR 0051 draws. Everything before it may
/// be abandoned for a fallback; everything after it is program work, and a
/// failure there is reported rather than retried on another route.
/// `Clone` here is deliberate and is not the permission [`Preflight`] withholds.
/// Cloning a route that is already committed cannot un-commit it or produce a
/// second choice; it only lets a host hand the committed decision to the code
/// that encodes it.
#[derive(Clone, Debug)]
pub struct RoutedDispatch<'a> {
    identity: CanonicalArtifactProgramIdentity,
    kernel_program: &'a [u8],
    entries: Vec<RoutedEntry<'a>>,
    shared: Vec<SharedAllocation>,
}

impl<'a> RoutedDispatch<'a> {
    /// Returns the identity of the artifact being executed.
    #[must_use]
    pub const fn identity(&self) -> &CanonicalArtifactProgramIdentity {
        &self.identity
    }

    /// Returns the canonical identity of the kernel program being executed.
    ///
    /// The identity alone; the program is not carried. Republished after the
    /// commit so a host can record *what* it ran beside the result, which is the
    /// value a numerical comparison needs to be attributable.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.kernel_program
    }

    /// Returns the committed entries **in the order they must be dispatched**.
    ///
    /// The same sequence [`Preflight::entries`] published, carried across the
    /// commit unchanged. A host encodes them front to back, and each carries its
    /// own object, symbol, launch geometry, and bindings.
    #[must_use]
    pub fn entries(&self) -> &[RoutedEntry<'a>] {
        &self.entries
    }

    /// Returns the slot pairs that must be backed by one allocation each.
    ///
    /// Empty for a single-entry route. See [`SharedAllocation`] for why a loader
    /// that ignored these would read uninitialised storage rather than refuse.
    #[must_use]
    pub fn shared_allocations(&self) -> &[SharedAllocation] {
        &self.shared
    }

    /// Returns how each committed object reaches an executable state.
    ///
    /// Always [`ArtifactExecutionPolicy::NativeImage`], because that is the only
    /// policy the vocabulary defines — no longer because preflight refuses the
    /// alternative. It is still returned rather than assumed, so a host does not
    /// hard-code at its own load site a fact that a second policy would change.
    ///
    /// Per entry rather than per route, because nothing requires two entries to
    /// name one payload and a single answer would be a claim about one of them.
    #[must_use]
    pub fn execution_policies(
        &self,
    ) -> impl ExactSizeIterator<Item = ArtifactExecutionPolicy> + '_ {
        self.entries
            .iter()
            .map(|entry| entry.payload.execution_policy)
    }
}
