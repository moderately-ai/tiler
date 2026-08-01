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
//! reporting device facts the loader compares, preparing each entry, sizing and
//! allocating storage, and — only after the commit — running and observing the
//! dispatch to a terminal outcome.

use std::fmt;

use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler_runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight, RoutedDispatch,
    RoutedEntry, TargetPropertyRequest,
};

use tiler_artifact::program::{
    BackendKey, RepresentationKey, RouteRequirement, RouteResourceDimension, TargetProfileRef,
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
    /// The dispatch was sized, allocated, and bound.
    PlanDispatch,
    /// The committed route was encoded, run, and observed.
    Dispatch,
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
    UndersizedInput,
    /// The run halts after one invocation, after the routing commit.
    HaltAfterOneInvocation,
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

/// One consumer-selected adapter for the `tiler.test.scalar-host` backend.
///
/// Selected by naming it. Nothing registers it, nothing discovers it, and the
/// artifact carries no identity of it to match against.
#[derive(Debug)]
pub struct ScalarHostAdapter {
    profile: TargetProfileRef,
    backend: BackendKey,
    representation: RepresentationKey,
    /// The caller-supplied storage for the one named program input.
    input: Vec<u8>,
    /// The largest grid this single-threaded interpreter admits.
    invocation_budget: u64,
    perturbation: Option<Perturbation>,
    /// Entries this backend validated from their carried bytes, in execution order.
    validated: Vec<ScalarEntry>,
    /// Entries promoted to prepared state, in execution order.
    prepared: Vec<ScalarEntry>,
    /// Host allocations, indexed by the placements below.
    allocations: Vec<Vec<u8>>,
    /// Per entry, per ABI slot, where the slot's storage lives.
    placements: Vec<Vec<Placement>>,
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
            input,
            invocation_budget: 64,
            perturbation: None,
            validated: Vec::new(),
            prepared: Vec::new(),
            allocations: Vec::new(),
            placements: Vec::new(),
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

    fn perturbed_by(&self, perturbation: Perturbation) -> bool {
        self.perturbation == Some(perturbation)
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
            RouteRequirement::ResourceFloor(floor) => match floor.dimension() {
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
        let mut allocations: Vec<Vec<u8>> = Vec::new();
        let mut placements: Vec<Vec<Placement>> = Vec::new();
        let mut readback = None;

        // Paired first, because a shared allocation belongs to two entries and
        // neither owns it. A planner that allocated per binding would hand the
        // consumer a fresh buffer and it would read uninitialised storage — a
        // wrong answer rather than a refusal. Empty for this single-entry route,
        // which is a state rather than an absence.
        let mut shared: Vec<Option<usize>> = preflight
            .entries()
            .iter()
            .map(|entry| entry.bindings().len())
            .flat_map(|count| std::iter::repeat_n(None, count))
            .collect();
        let slot_index = |entry: usize, slot: usize| {
            preflight.entries()[..entry]
                .iter()
                .map(|routed| routed.bindings().len())
                .sum::<usize>()
                + slot
        };
        for pair in preflight.shared_allocations() {
            let (producer, consumer) = (pair.producer(), pair.consumer());
            let needed = preflight.entries()[producer.entry()].bindings()[producer.slot()]
                .accessible_bytes()
                .max(
                    preflight.entries()[consumer.entry()].bindings()[consumer.slot()]
                        .accessible_bytes(),
                );
            allocations.push(vec![
                0_u8;
                usize::try_from(needed).expect("a small fixture range")
            ]);
            let index = allocations.len() - 1;
            shared[slot_index(producer.entry(), producer.slot())] = Some(index);
            shared[slot_index(consumer.entry(), consumer.slot())] = Some(index);
        }

        for (position, entry) in preflight.entries().iter().enumerate() {
            let mut slots = Vec::with_capacity(entry.bindings().len());
            for binding in entry.bindings() {
                let bytes = binding.accessible_bytes();
                let offset = binding.accessible_offset();
                let reach = offset + bytes;
                let allocation = if let Some(index) = shared[slot_index(position, binding.slot())] {
                    index
                } else if addresses_program_input(binding.binding().target()) {
                    // Caller-supplied storage, checked against the route's own
                    // published range while abandoning is still permitted.
                    let supplied = u64::try_from(self.input.len()).expect("a small fixture buffer");
                    if supplied < reach {
                        return Err(ScalarRefusal::UndersizedStorage {
                            entry: position,
                            slot: binding.slot(),
                            required: reach,
                            supplied,
                        });
                    }
                    allocations.push(self.input.clone());
                    allocations.len() - 1
                } else {
                    allocations.push(vec![
                        0_u8;
                        usize::try_from(reach).expect("a small fixture range")
                    ]);
                    let index = allocations.len() - 1;
                    readback = Some(Readback {
                        allocation: index,
                        offset: usize::try_from(offset).expect("a small fixture range"),
                        elements: usize::try_from(bytes / 4)
                            .expect("a small fixture element count"),
                    });
                    index
                };
                slots.push(Placement {
                    allocation,
                    offset,
                    bytes,
                });
            }
            placements.push(slots);
        }

        self.allocations = allocations;
        self.placements = placements;
        self.readback = readback;
        Ok(())
    }

    fn dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        self.stages.push(Stage::Dispatch);
        let halt = self
            .perturbed_by(Perturbation::HaltAfterOneInvocation)
            .then_some(1);
        let mut executed = 0_u64;
        for (position, entry) in routed.entries().iter().enumerate() {
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
                prepared,
                read,
                write,
                &mut self.allocations,
                launch.grid_threads(),
                halt,
            )?;
            // Terminal success, observed rather than assumed. Past the commit,
            // so this is reported and never resolved by routing somewhere else.
            if ran != launch.grid_threads() {
                return Err(ExecutionFault::Incomplete {
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
        } = self.readback.expect("the plan named a readback allocation");
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
