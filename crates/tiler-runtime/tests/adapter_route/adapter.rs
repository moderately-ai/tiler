//! One consumer's runtime adapter for a backend that is not Metal.
//!
//! # What this proves, and what it deliberately does not
//!
//! It is an *out-of-crate* implementor. This file compiles against
//! `tiler-runtime`'s public surface alone, so anything it needs and cannot reach
//! is a hole in the seam rather than a hole in a test. That is the whole reason
//! the fixture lives in `tests/` and not in a `#[cfg(test)]` module beside the
//! trait: a same-crate implementor can reach `pub(crate)` items and would prove
//! nothing about whether a consumer can write one.
//!
//! It is not a claim that this backend is fast, general, or production-shaped.
//! Its execution model is one invocation at a time on the calling thread, its
//! representation carries one kernel shape, and its live context is a process
//! rather than a device. Those are the properties that make it a *different*
//! backend from Metal, which is what the seam has to survive.
//!
//! # Where each responsibility sits
//!
//! Everything the loader could decide has been decided before a method here
//! runs. What is left is exactly the adapter half: binding a live context by
//! measuring this process, validating the carried payload from its own bytes,
//! reporting device facts the loader compares, preparing each entry, sizing
//! storage without acquiring any, and — only after the commit — allocating that
//! storage and running and observing the dispatch to a terminal outcome.
//!
//! # Sizing and allocating are two methods here because they are two stages
//!
//! ADR 0051 places program allocation after the routing commit, so
//! [`RuntimeAdapter::plan_dispatch`] below records a plan and this adapter's
//! `Vec<u8>` storage is created in [`RuntimeAdapter::allocate_dispatch`]. The
//! one comparison that stays pre-commit is against the *caller's* operand
//! storage, which already exists: comparing it costs nothing and refusing on it
//! is still a route another artifact could satisfy.

use std::collections::BTreeMap;
use std::fmt;

use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler_runtime::load::{
    DTypeDispatch, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight,
    RoutedDispatch, RoutedEntry, TargetPropertyRequest,
};

use tiler_artifact::program::{
    ArithmeticType, BackendKey, RepresentationKey, RouteRequirement, RouteResourceDimension,
    TargetProfileRef,
};

use crate::fixture;
use crate::image::{
    ExecutionFault, Placement, ScalarEntry, ScalarPayloadRefusal, addresses_program_input, decode,
};

/// Which stage of the route the adapter was last asked for.
///
/// Recorded so a test can assert the *order* the loader drove the adapter in and,
/// more importantly, that a refused route never reached a later stage. A
/// stage-ordering claim asserted only from the returned error would be a claim
/// about the error rather than about what ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    /// A live device and execution context was bound.
    Bind,
    /// One routed entry's carried payload was validated from its bytes.
    ValidatePayload,
    /// One live-device route requirement was reported.
    ObserveLiveDevice,
    /// The route's entries were prepared.
    PrepareEntries,
    /// One prepared entry's target property was reported.
    ObservePreparedEntry,
    /// The dispatch was sized and its capacity checked, acquiring nothing.
    PlanDispatch,
    /// The committed route's storage was allocated and bound.
    AllocateDispatch,
    /// The committed route was encoded, run, and observed.
    Dispatch,
}

/// One measured target family's dtype-dispatchability row.
///
/// The three answers `decide-per-dtype-dispatchability-as-a-target-capability`
/// established the mechanism produces, named after the families that produced
/// them rather than after the verdicts, because the point of routing three of
/// them through one adapter is that the refusal is the mechanism saying no on a
/// real family's behalf and not a second code path.
///
/// **Measurement boundary.** The two positive and negative rows restate the
/// verdicts the [BF16 spike](../../../../spikes/numerics/bf16-second-dtype/README.md)
/// declares from the retained Apple record — macOS Apple9 dispatches `bfloat`,
/// the iOS Simulator refuses it at pipeline creation — for the fixture's own
/// scalar-host profile. Nothing here measures an Apple device; what is under
/// test is what a loader does with such a row, not whether the row is true.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DispatchFamily {
    /// Declares both `f32` and `bf16` dispatchable, as the macOS row does.
    DispatchesBf16,
    /// Declares `bf16` explicitly unsupported and `f32` dispatchable.
    ///
    /// `f32` is declared beside it deliberately: a family that said nothing at
    /// all would make every refusal below indistinguishable from an unmeasured
    /// one, and it is the accepted neighbour that makes the `bf16` answer
    /// evidence about `bf16`.
    RefusesBf16,
    /// Declares `f32` alone, saying nothing about `bf16`.
    ///
    /// The `Unknown` control: a family nobody measured for this dtype.
    UnmeasuredForBf16,
}

impl DispatchFamily {
    fn declarations(self) -> BTreeMap<ArithmeticType, DTypeDispatch> {
        let mut declared = BTreeMap::new();
        declared.insert(ArithmeticType::F32, DTypeDispatch::Dispatchable);
        match self {
            Self::DispatchesBf16 => {
                declared.insert(ArithmeticType::Bf16, DTypeDispatch::Dispatchable);
            }
            Self::RefusesBf16 => {
                declared.insert(ArithmeticType::Bf16, DTypeDispatch::Unsupported);
            }
            Self::UnmeasuredForBf16 => {}
        }
        declared
    }
}

/// A deliberate perturbation of the adapter's own behaviour.
///
/// Every value here perturbs the *adapter*, never the loader: the artifact-side
/// perturbations vary the fixture's bytes instead. Keeping the two apart is what
/// lets a failing case name which half produced the refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Perturbation {
    /// No live execution context can be bound.
    NoContext,
    /// The bound context reports another governed profile key.
    ForeignProfileKey,
    /// The bound context reports another exact profile descriptor.
    ForeignProfileDescriptor,
    /// The bound context reports another backend family.
    ForeignBackend,
    /// The bound context reports another executable representation.
    ForeignRepresentation,
    /// Every live-device row is answered `Unrecognized`.
    UnrecognizeLiveDevice,
    /// A qualitative row is answered with a measured quantity.
    MisanswerLiveDevice,
    /// A row this adapter owns is answered as unsupported.
    RefuseLiveDeviceFeature,
    /// One fewer invocation is reported than the deferred predicate requires.
    UnderreportPreparedEntry,
    /// Exactly the threshold is reported, which must be accepted.
    ///
    /// Stated rather than relying on the fixture's row count happening to equal
    /// the threshold: a boundary asserted through a coincidence stops being a
    /// boundary the moment the fixture's extents change.
    ReportPreparedEntryAtThreshold,
    /// No entry can be prepared.
    RefusePreparation,
    /// The caller's input storage is one element short.
    ///
    /// Pre-commit, and it stays there under the split seam: the caller supplied
    /// that storage, so the comparison needs nothing allocated to make it.
    UndersizedInput,
    /// The run halts after one invocation, after the routing commit.
    HaltAfterOneInvocation,
    /// The run halts after one invocation of the **second** entry.
    ///
    /// Distinct from the perturbation above rather than a parameter of it: the
    /// state it produces is a route whose earlier entries reached terminal
    /// success and whose later one did not, and a single-entry route cannot be
    /// in that state at all.
    HaltSecondEntryAfterOneInvocation,
    /// One shared allocation comes back one element shorter than the plan sized it.
    ///
    /// **Post-commit**, because the allocation is. The pre-commit plan sized the
    /// pair correctly and the acquisition did not honour it, which is the
    /// allocator-residue case ADR 0051's placement makes terminal: there is no
    /// second route to take, so the only useful behaviour is to report it.
    UndersizeSharedAllocation,
    /// The committed entries are dispatched back to front.
    ///
    /// The one perturbation here that produces neither a refusal nor a panic.
    /// See [`RuntimeAdapter::dispatch`]'s contract and the shared-allocation
    /// pairing's own documentation: reading a scratch buffer before its producer
    /// wrote it is a wrong answer, which is why the ordering has to be checked
    /// against an oracle rather than trusted.
    ReverseStageOrder,
}

/// Where a completed dispatch reads its result from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Readback {
    /// Index of the host allocation the output landed in.
    allocation: usize,
    /// First addressed byte of the output value within it.
    offset: usize,
    /// How many `f32` elements to read from there.
    elements: usize,
}

/// Which storage one routed slot will be backed by, decided before the commit.
///
/// A decision about storage rather than storage: the three cases differ in *who*
/// supplies the bytes, and only one of them is an acquisition this adapter has
/// to wait for the commit to make.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backing {
    /// The caller's own operand storage, which exists before the route does.
    CallerInput,
    /// One end of a pair the route says must share a single allocation.
    Shared(usize),
    /// Storage this adapter acquires for itself after the commit.
    Fresh,
}

/// One routed slot sized against the route, with nothing acquired for it.
#[derive(Clone, Copy, Debug)]
struct SlotPlan {
    /// First addressed byte of the bound value within its allocation.
    offset: u64,
    /// Bytes the route requires be reachable from that offset.
    bytes: u64,
    /// The byte the slot must reach, counted from the start of the allocation.
    ///
    /// Not the extent alone: a binding may address a window at a nonzero offset,
    /// and an allocation sized to the extent would be short by exactly it.
    reach: u64,
    /// Where the bytes will come from.
    backing: Backing,
}

/// One pair of slots the route requires be backed by a single allocation.
#[derive(Clone, Copy, Debug)]
struct SharedPlan {
    /// Entry and ABI slot writing the shared storage.
    producer: (usize, usize),
    /// Entry and ABI slot reading it.
    consumer: (usize, usize),
    /// Bytes the allocation must hold to satisfy both ends.
    needed: u64,
}

/// What one completed dispatch yields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarCompletion {
    /// Output element bit patterns, read back from the adapter's own storage.
    pub result_bits: Vec<u32>,
    /// Invocations that ran, summed across entries.
    pub executed: u64,
    /// The governed profile key the route was carried out under.
    pub profile_key: String,
}

/// Where one shared-allocation pair ended up in this adapter's own storage.
///
/// Recorded so a test can assert that the two slots resolved to **one**
/// allocation. The result agreeing with the reference is evidence that the
/// consumer read what the producer wrote; this is the evidence that it did so
/// through a single buffer rather than through two that happened to hold the
/// same bytes, which is the failure a wrong planner actually produces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SharedPlacement {
    /// Entry and ABI slot writing the shared storage.
    pub producer: (usize, usize),
    /// Entry and ABI slot reading it.
    pub consumer: (usize, usize),
    /// Index of the one host allocation backing both.
    pub allocation: usize,
}

/// One consumer-selected adapter for the `tiler.test.scalar-host` backend.
///
/// Selected by naming it. Nothing registers it, nothing discovers it, and the
/// artifact carries no identity of it to match against.
#[derive(Debug)]
pub struct ScalarHostAdapter {
    profile: TargetProfileRef,
    backend: BackendKey,
    representation: RepresentationKey,
    /// Which dtypes this host's family states it can dispatch.
    ///
    /// An adapter field rather than a fixture constant, because it is the one
    /// thing the three measured target families in this suite differ by: the
    /// macOS row declares BF16 dispatchable, the iOS-Simulator row declares it
    /// unsupported, and an unmeasured family declares nothing at all.
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    /// The caller-supplied storage for the one named program input.
    input: Vec<u8>,
    /// The largest grid this single-threaded interpreter admits.
    invocation_budget: u64,
    perturbation: Option<Perturbation>,
    /// Entries this backend validated from their carried bytes, in execution order.
    validated: Vec<ScalarEntry>,
    /// Entries promoted to prepared state, in execution order.
    prepared: Vec<ScalarEntry>,
    /// Per entry, per ABI slot, what the pre-commit plan decided, acquiring nothing.
    plan: Vec<Vec<SlotPlan>>,
    /// The pairs the pre-commit plan sized for one allocation each, in route order.
    shared_plan: Vec<SharedPlan>,
    /// The entry and slot whose allocation the completion reads back from.
    ///
    /// Decided while planning, because it follows from the binding targets, and
    /// resolved to an allocation index only once one exists.
    readback_slot: Option<(usize, usize)>,
    /// Host allocations, indexed by the placements below.
    allocations: Vec<Vec<u8>>,
    /// Per entry, per ABI slot, where the slot's storage lives.
    placements: Vec<Vec<Placement>>,
    /// The pairs this planner backed with one allocation each, in route order.
    ///
    /// Empty for a single-entry route, which is a state rather than an absence.
    shared: Vec<SharedPlacement>,
    /// The allocation, first addressed byte, and element count to read back.
    ///
    /// The offset is carried rather than assumed zero. A binding may address
    /// part of the value it names, and a read-back that started at the
    /// allocation's first byte would return the right count of the wrong
    /// elements the day a plan binds a partial window.
    readback: Option<Readback>,
    /// Every stage the loader drove, in order.
    pub stages: Vec<Stage>,
}

impl ScalarHostAdapter {
    /// Builds an unperturbed adapter over the caller's input element bits.
    #[must_use]
    pub fn new(input_bits: &[u32]) -> Self {
        let mut input = Vec::with_capacity(input_bits.len() * 4);
        for bits in input_bits {
            input.extend_from_slice(&bits.to_le_bytes());
        }
        Self {
            profile: fixture::profile(),
            backend: fixture::backend(),
            representation: fixture::representation(),
            dtype_dispatch: fixture::dispatches_f32_and_bf16(),
            input,
            invocation_budget: 64,
            perturbation: None,
            validated: Vec::new(),
            prepared: Vec::new(),
            plan: Vec::new(),
            shared_plan: Vec::new(),
            readback_slot: None,
            allocations: Vec::new(),
            placements: Vec::new(),
            shared: Vec::new(),
            readback: None,
            stages: Vec::new(),
        }
    }

    /// Returns the same adapter with one deliberate perturbation applied.
    #[must_use]
    pub fn perturbed(mut self, perturbation: Perturbation) -> Self {
        match perturbation {
            Perturbation::UndersizedInput => {
                self.input.truncate(self.input.len().saturating_sub(4));
            }
            Perturbation::RefusePreparation => self.invocation_budget = 1,
            _ => {}
        }
        self.perturbation = Some(perturbation);
        self
    }

    /// Returns the same adapter reporting one measured target family's dtype row.
    ///
    /// Not a [`Perturbation`]: a perturbation is a deliberate defect, and each of
    /// these is a correct report from a family that was actually measured. The
    /// refusals they produce are the mechanism working, not the adapter lying.
    #[must_use]
    pub fn on_family(mut self, family: DispatchFamily) -> Self {
        self.dtype_dispatch = family.declarations();
        self
    }

    fn perturbed_by(&self, perturbation: Perturbation) -> bool {
        self.perturbation == Some(perturbation)
    }

    /// Returns the shared-allocation pairs this planner backed, in route order.
    #[must_use]
    pub fn shared_placements(&self) -> &[SharedPlacement] {
        &self.shared
    }

    /// Returns where one routed slot's storage was placed.
    ///
    /// # Panics
    ///
    /// Panics when the route never allocated that entry and slot. Allocation
    /// runs after the commit, so a caller asking about a slot of a route that
    /// refused earlier is asking about something that does not exist.
    #[must_use]
    pub fn placement(&self, entry: usize, slot: usize) -> Placement {
        self.placements[entry][slot]
    }

    /// Reads one host allocation back as `f32` bit patterns.
    ///
    /// Callable after the route returns, which is the point: an allocation this
    /// answers for outlived every dispatch that used it. A backend releasing
    /// storage between two entries would have nothing to answer with here.
    ///
    /// # Panics
    ///
    /// Panics when no such allocation was made.
    #[must_use]
    pub fn allocation_bits(&self, allocation: usize) -> Vec<u32> {
        self.allocations[allocation]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect()
    }
}

/// Why this adapter refused a route before it committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScalarRefusal {
    /// This process is not one this adapter will execute on.
    NoExecutionContext,
    /// The carried payload is not one this backend executes.
    Payload(ScalarPayloadRefusal),
    /// One entry's launch exceeds what this single-threaded interpreter admits.
    LaunchBeyondBudget {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Invocations the route launches.
        grid_threads: u64,
        /// Invocations this adapter admits.
        budget: u64,
    },
    /// The caller supplied less storage than the route requires.
    ///
    /// The only storage comparison left on this side of the commit, and it is
    /// here because the caller's bytes exist independently of the route: nothing
    /// is acquired to make it, so it is a refusal another artifact may satisfy.
    UndersizedStorage {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot of the binding.
        slot: usize,
        /// Bytes the route requires be reachable.
        required: u64,
        /// Bytes the caller supplied.
        supplied: u64,
    },
}

impl fmt::Display for ScalarRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoExecutionContext => formatter
                .write_str("scalar-host.context: this process is not one this adapter executes on"),
            Self::Payload(refusal) => write!(formatter, "{refusal}"),
            Self::LaunchBeyondBudget {
                entry,
                grid_threads,
                budget,
            } => write!(
                formatter,
                "scalar-host.prepare: entry {entry} launches {grid_threads} invocation(s) and this \
                 interpreter admits {budget}",
            ),
            Self::UndersizedStorage {
                entry,
                slot,
                required,
                supplied,
            } => write!(
                formatter,
                "scalar-host.plan: entry {entry}'s slot {slot} needs {required} byte(s) reachable \
                 and {supplied} were supplied",
            ),
        }
    }
}

impl std::error::Error for ScalarRefusal {}

impl RuntimeAdapter for ScalarHostAdapter {
    type Refusal = ScalarRefusal;
    type Failure = ExecutionFault;
    type Completion = ScalarCompletion;

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        self.stages.push(Stage::Bind);
        if self.perturbed_by(Perturbation::NoContext) {
            return Err(ScalarRefusal::NoExecutionContext);
        }
        // A real binding for this backend measures the process it will run in.
        // The fixture's live fact is that this build's `f32` arithmetic performs
        // the multiply and add the image declares at all; a host whose
        // arithmetic produced something else would refuse here rather than
        // return numbers that are close and wrong.
        Ok(ExecutionEnvironment {
            target_profile: match self.perturbation {
                Some(Perturbation::ForeignProfileKey) => {
                    fixture::profile_named("tiler.test.other-host", fixture::PROFILE_DESCRIPTOR)
                }
                Some(Perturbation::ForeignProfileDescriptor) => {
                    fixture::profile_named(fixture::PROFILE_KEY, b"scalar-host-descriptor-b")
                }
                _ => self.profile.clone(),
            },
            backend: if self.perturbed_by(Perturbation::ForeignBackend) {
                BackendKey::new("tiler.test.other-backend").expect("a governed backend key")
            } else {
                self.backend.clone()
            },
            representation: if self.perturbed_by(Perturbation::ForeignRepresentation) {
                RepresentationKey::new("tiler.test.scalar-host-image-v2")
                    .expect("a governed representation key")
            } else {
                self.representation.clone()
            },
            dtype_dispatch: self.dtype_dispatch.clone(),
        })
    }

    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        self.stages.push(Stage::ValidatePayload);
        // ADR 0090 item 8, and the artifact layer performed no part of it: the
        // envelope proved this object's integrity digest and carried it opaquely.
        let image = decode(entry.object()).map_err(ScalarRefusal::Payload)?;
        let validated = image.entry_for(entry).map_err(ScalarRefusal::Payload)?;
        // Retained rather than re-derived later. Decoding twice would mean two
        // decisions about one object, and the second would be the one that ran.
        self.validated.push(validated.clone());
        Ok(())
    }

    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        self.stages.push(Stage::ObserveLiveDevice);
        if self.perturbed_by(Perturbation::UnrecognizeLiveDevice) {
            return LiveDeviceObservation::Unrecognized;
        }
        // Exhaustive on both the kind and the dimension: a row this adapter has
        // never seen must stop this build rather than reach an arm that guesses.
        match request.requirement() {
            RouteRequirement::Resource(resource) => match resource.dimension() {
                // A scalar host has no subgroup, and reporting a width from a
                // table would report documentation as an observation.
                RouteResourceDimension::SubgroupThreads => LiveDeviceObservation::Unrecognized,
            },
            RouteRequirement::BackendFeature(feature) => {
                if self.perturbed_by(Perturbation::MisanswerLiveDevice) {
                    return LiveDeviceObservation::Quantity(1);
                }
                // Key and version matched exactly. One key at two versions can
                // mean two things, and guessing which is how a route runs on a
                // host it was refused on.
                if feature.key().as_str() != fixture::HOST_ARITHMETIC_FEATURE
                    || feature.version() != fixture::HOST_ARITHMETIC_VERSION
                    || feature.payload() != fixture::HOST_ARITHMETIC_PAYLOAD
                {
                    return LiveDeviceObservation::Unrecognized;
                }
                // A measurement, not a constant. **Measurement boundary:** the
                // fixture's artifact declares strict `f32` with subnormals
                // preserved, so on a process running with flush-to-zero enabled
                // this reports `false` and the loader refuses the route as
                // `runtime.unsatisfied-route-requirement`. That is the contract
                // working rather than a defect, and it is also why this suite's
                // accepted case is a claim about a host that preserves them —
                // every macOS arm64 host measured in this repository does.
                LiveDeviceObservation::Feature(
                    !self.perturbed_by(Perturbation::RefuseLiveDeviceFeature)
                        && preserves_subnormals(),
                )
            }
        }
    }

    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        self.stages.push(Stage::PrepareEntries);
        // Every entry, not the first one. A two-entry route whose *second* entry
        // could not be prepared must be refused here rather than discovered
        // between two dispatches.
        for (position, entry) in entries.iter().enumerate() {
            let grid_threads = entry.launch().grid_threads();
            if grid_threads > self.invocation_budget {
                return Err(ScalarRefusal::LaunchBeyondBudget {
                    entry: position,
                    grid_threads,
                    budget: self.invocation_budget,
                });
            }
        }
        // The images this backend validated, promoted to prepared entries. A
        // real device backend builds a library and a pipeline here; an
        // interpreter's prepared state is the validated image itself, and saying
        // so is more honest than inventing an object it does not have.
        self.prepared.clone_from(&self.validated);
        Ok(())
    }

    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> u64 {
        self.stages.push(Stage::ObservePreparedEntry);
        // The exact prepared entry the request names, not a host-wide property
        // that resembles it. The comparison, the threshold, and the direction
        // stay with the loader.
        let entry = &self.prepared[request.entry()];
        let invocations = u64::from(entry.rows);
        match self.perturbation {
            Some(Perturbation::UnderreportPreparedEntry) => fixture::PREPARED_PROPERTY_MINIMUM - 1,
            Some(Perturbation::ReportPreparedEntryAtThreshold) => {
                fixture::PREPARED_PROPERTY_MINIMUM
            }
            _ => invocations,
        }
    }

    fn plan_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal> {
        self.stages.push(Stage::PlanDispatch);

        // The byte a slot must reach through, counted from the start of the
        // allocation rather than from the start of the value: a binding may
        // address a window at a nonzero offset, and an allocation sized to the
        // extent alone would be short by exactly that offset.
        let reach = |entry: usize, slot: usize| {
            let binding = preflight.entries()[entry].bindings()[slot];
            binding.accessible_offset() + binding.accessible_bytes()
        };

        // Paired first, because a shared allocation belongs to two entries and
        // neither owns it. A planner that sized per binding would let the
        // consumer be handed a fresh buffer and read uninitialised storage — a
        // wrong answer rather than a refusal. Empty for a single-entry route,
        // which is a state rather than an absence.
        let mut backing: Vec<Vec<Option<usize>>> = preflight
            .entries()
            .iter()
            .map(|entry| vec![None; entry.bindings().len()])
            .collect();
        let mut shared_plan: Vec<SharedPlan> = Vec::new();
        for pair in preflight.shared_allocations() {
            let (producer, consumer) = (pair.producer(), pair.consumer());
            let index = shared_plan.len();
            backing[producer.entry()][producer.slot()] = Some(index);
            backing[consumer.entry()][consumer.slot()] = Some(index);
            shared_plan.push(SharedPlan {
                producer: (producer.entry(), producer.slot()),
                consumer: (consumer.entry(), consumer.slot()),
                needed: reach(producer.entry(), producer.slot())
                    .max(reach(consumer.entry(), consumer.slot())),
            });
        }

        // The caller's operand storage exists already, so this length is a fact
        // about the route's inputs rather than about anything acquired for it.
        let supplied = u64::try_from(self.input.len()).expect("a small fixture buffer");
        let mut plan: Vec<Vec<SlotPlan>> = Vec::with_capacity(preflight.entries().len());
        let mut readback_slot = None;
        for (position, entry) in preflight.entries().iter().enumerate() {
            let mut slots = Vec::with_capacity(entry.bindings().len());
            for binding in entry.bindings() {
                let bytes = binding.accessible_bytes();
                let offset = binding.accessible_offset();
                let reach = offset + bytes;
                let backing = if let Some(index) = backing[position][binding.slot()] {
                    Backing::Shared(index)
                } else if addresses_program_input(binding.binding().target()) {
                    // The one comparison that belongs on this side of the
                    // commit: it is against storage the caller supplied, so a
                    // route this host cannot satisfy is refused while another
                    // artifact may still be tried, and nothing was acquired to
                    // find that out.
                    if supplied < reach {
                        return Err(ScalarRefusal::UndersizedStorage {
                            entry: position,
                            slot: binding.slot(),
                            required: reach,
                            supplied,
                        });
                    }
                    Backing::CallerInput
                } else {
                    readback_slot = Some((position, binding.slot()));
                    Backing::Fresh
                };
                slots.push(SlotPlan {
                    offset,
                    bytes,
                    reach,
                    backing,
                });
            }
            plan.push(slots);
        }

        self.plan = plan;
        self.shared_plan = shared_plan;
        self.readback_slot = readback_slot;
        Ok(())
    }

    fn allocate_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        _routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure> {
        self.stages.push(Stage::AllocateDispatch);
        let mut allocations: Vec<Vec<u8>> = Vec::new();
        let mut recorded: Vec<SharedPlacement> = Vec::new();

        // One allocation per pair, taken first so that both ends can name it.
        let mut shared_index = Vec::with_capacity(self.shared_plan.len());
        for pair in &self.shared_plan {
            let needed = if self.perturbation == Some(Perturbation::UndersizeSharedAllocation) {
                pair.needed.saturating_sub(4)
            } else {
                pair.needed
            };
            allocations.push(vec![
                0_u8;
                usize::try_from(needed).expect("a small fixture range")
            ]);
            let index = allocations.len() - 1;
            shared_index.push(index);
            recorded.push(SharedPlacement {
                producer: pair.producer,
                consumer: pair.consumer,
                allocation: index,
            });
        }

        let mut placements: Vec<Vec<Placement>> = Vec::with_capacity(self.plan.len());
        let mut readback = None;
        for position in 0..self.plan.len() {
            let mut slots = Vec::with_capacity(self.plan[position].len());
            for slot in 0..self.plan[position].len() {
                // Copied out rather than borrowed, so this loop can push to the
                // storage it is deciding about.
                let planned = self.plan[position][slot];
                let allocation = match planned.backing {
                    Backing::Shared(index) => shared_index[index],
                    Backing::CallerInput => {
                        allocations.push(self.input.clone());
                        allocations.len() - 1
                    }
                    Backing::Fresh => {
                        allocations.push(vec![
                            0_u8;
                            usize::try_from(planned.reach)
                                .expect("a small fixture range")
                        ]);
                        allocations.len() - 1
                    }
                };
                // Every allocation against the length the plan sized it for, and
                // this is an assertion rather than a routing input: the route has
                // committed, so an allocator that returned less than it accepted
                // is a defect to report and not a reason to try elsewhere.
                let held =
                    u64::try_from(allocations[allocation].len()).expect("a small fixture buffer");
                if held < planned.reach {
                    return Err(ExecutionFault::UndersizedStorage {
                        entry: position,
                        slot,
                        required: planned.reach,
                        held,
                    });
                }
                if self.readback_slot == Some((position, slot)) {
                    readback = Some(Readback {
                        allocation,
                        offset: usize::try_from(planned.offset).expect("a small fixture range"),
                        elements: usize::try_from(planned.bytes / 4)
                            .expect("a small fixture element count"),
                    });
                }
                slots.push(Placement {
                    allocation,
                    offset: planned.offset,
                    bytes: planned.bytes,
                });
            }
            placements.push(slots);
        }

        self.allocations = allocations;
        self.placements = placements;
        self.shared = recorded;
        self.readback = readback;
        Ok(())
    }

    fn dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        self.stages.push(Stage::Dispatch);
        // The order the committed route published, front to back. Reversing it
        // is a perturbation rather than a choice: the route's entries are
        // ordered by the data dependencies the packaged program proved, so a
        // backend encoding them in any other order dispatches a consumer before
        // its producer.
        let mut order: Vec<usize> = (0..routed.entries().len()).collect();
        if self.perturbed_by(Perturbation::ReverseStageOrder) {
            order.reverse();
        }
        let mut executed = 0_u64;
        for position in order {
            let entry = &routed.entries()[position];
            let halt = if self.perturbed_by(Perturbation::HaltAfterOneInvocation)
                || (self.perturbed_by(Perturbation::HaltSecondEntryAfterOneInvocation)
                    && position == 1)
            {
                Some(1)
            } else {
                None
            };
            let launch = entry.launch();
            if launch.grid_threads() == 0 && launch.zero_work_skips_dispatch() {
                continue;
            }
            let prepared = &self.prepared[position];
            let slots = &self.placements[position];
            let read = slots
                .iter()
                .zip(entry.bindings())
                .find(|(_, binding)| binding.transport_slot() == prepared.read_transport)
                .map(|(placement, _)| *placement)
                .expect("payload validation proved the read transport is one of this entry's");
            let write = slots
                .iter()
                .zip(entry.bindings())
                .find(|(_, binding)| binding.transport_slot() == prepared.write_transport)
                .map(|(placement, _)| *placement)
                .expect("payload validation proved the write transport is one of this entry's");
            let ran = crate::image::execute(
                position,
                prepared,
                read,
                write,
                &mut self.allocations,
                launch.grid_threads(),
                halt,
            )?;
            // Terminal success, observed rather than assumed, and per entry
            // rather than once for the route: a later entry must not be
            // dispatched over an earlier one that did not complete. Past the
            // commit, so this is reported and never resolved by routing
            // somewhere else.
            if ran != launch.grid_threads() {
                return Err(ExecutionFault::Incomplete {
                    entry: position,
                    executed: ran,
                    expected: launch.grid_threads(),
                });
            }
            executed += ran;
        }

        let Readback {
            allocation,
            offset,
            elements,
        } = self
            .readback
            .expect("the allocating stage resolved the plan's readback slot");
        let result_bits = self.allocations[allocation][offset..]
            .as_chunks::<4>()
            .0
            .iter()
            .take(elements)
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        Ok(ScalarCompletion {
            result_bits,
            executed,
            profile_key: context.target_profile().key.as_str().to_owned(),
        })
    }
}

/// Measures whether this process preserves a subnormal operand.
///
/// The arithmetic is performed rather than read from `cfg!`, because a
/// constant-folded probe would measure the compiler's compile-time arithmetic —
/// which always preserves subnormals — rather than the unit this dispatch runs
/// on, and would report a preserving host on a machine running with
/// flush-to-zero enabled.
fn preserves_subnormals() -> bool {
    let operand = std::hint::black_box(f32::from_bits(0x8000_0001));
    (operand * std::hint::black_box(1.0_f32)).to_bits() == 0x8000_0001
}
