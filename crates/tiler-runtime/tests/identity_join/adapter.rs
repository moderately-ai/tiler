//! The runtime half of the join: an adapter configured by a durable record.
//!
//! # What makes this adapter the consumer half rather than a second producer
//!
//! It is constructed from a [`Sidecar`](crate::sidecar::Sidecar) and a caller's
//! operand storage, and from nothing else. It holds no compiler value, no plan,
//! no emitter, and no handle the producing process created — it could not, since
//! this binary does not link the crates those live in. Everything it knows about
//! the artifact it is about to run, it re-reads from the artifact's own bytes.
//!
//! The environment it reports is the one it was **configured** with, never the
//! one the artifact declares. That distinction is the whole fixture: an adapter
//! that read the host identities out of the envelope it was handed would agree
//! with every artifact it was ever given, and every mismatched-subject case here
//! would pass by construction.
//!
//! # What is deliberately thin
//!
//! [`RuntimeAdapter::observe_live_device`] and
//! [`RuntimeAdapter::observe_prepared_entry`] are unreachable for these
//! artifacts: `assemble_plan_artifact` declares no route requirement, and this
//! backend's profile answers the workgroup bound at declaration time so the plan
//! mints no deferred predicate. Both are implemented rather than left to panic,
//! and the stage log is what proves they never ran. Their comparison behaviour
//! is evidenced by `tests/adapter_route`, which packages artifacts that do carry
//! both, and repeating it here would be a second copy of that suite rather than
//! evidence about the process boundary.

use std::fmt;

use tiler_artifact::program::{RouteRequirement, RouteResourceDimension, TargetProfileRef};
use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler_runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight, RoutedDispatch,
    RoutedEntry, TargetPropertyRequest,
};

use crate::image::{
    ExecutionFault, Placement, ScalarEntry, ScalarPayloadRefusal, addresses_program_input, decode,
};

/// Which stage of the route the adapter was last asked for.
///
/// Recorded so a case can assert that a refused route never reached a later
/// stage. A stage-ordering claim made only from the returned error would be a
/// claim about the error.
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

/// The stages a complete single-entry route runs here.
///
/// Neither observing stage appears: nothing this producer packages defers a
/// prepared-entry property or requires a live-device row, so the loader has
/// nothing to ask about. Both surrounding stages do appear, which is what ADR
/// 0090 item 9 makes unskippable once a caller enters `prepare`.
pub const COMPLETE_ROUTE: [Stage; 5] = [
    Stage::Bind,
    Stage::ValidatePayload,
    Stage::PrepareEntries,
    Stage::PlanDispatch,
    Stage::Dispatch,
];

/// Where a completed dispatch reads its result from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Readback {
    allocation: usize,
    offset: usize,
    elements: usize,
}

/// What one completed dispatch yields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion {
    /// Output element bit patterns, read back from this adapter's own storage.
    pub result_bits: Vec<u32>,
    /// Invocations that ran, summed across entries.
    pub executed: u64,
    /// The governed profile key the route was carried out under.
    pub profile_key: String,
}

/// Why this adapter refused a route before it committed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
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

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

impl std::error::Error for Refusal {}

/// One consumer-selected adapter for the `tiler.test.scalar-host` backend.
///
/// Selected by naming it. Nothing registers it, nothing discovers it, and no
/// artifact carries an identity of it to match against — the only thing that
/// decides whether it may run a given artifact is whether the governed
/// identities it was configured with are the ones the artifact declares.
#[derive(Debug)]
pub struct ScalarHostAdapter {
    environment: ExecutionEnvironment,
    input: Vec<u8>,
    invocation_budget: u64,
    validated: Vec<ScalarEntry>,
    prepared: Vec<ScalarEntry>,
    allocations: Vec<Vec<u8>>,
    placements: Vec<Vec<Placement>>,
    readback: Option<Readback>,
    /// Every stage the loader drove, in order.
    pub stages: Vec<Stage>,
}

impl ScalarHostAdapter {
    /// Builds an adapter that reports `environment` and holds the caller's operands.
    #[must_use]
    pub fn new(environment: ExecutionEnvironment, input_bits: &[u32]) -> Self {
        let mut input = Vec::with_capacity(input_bits.len() * 4);
        for bits in input_bits {
            input.extend_from_slice(&bits.to_le_bytes());
        }
        Self {
            environment,
            input,
            invocation_budget: 64,
            validated: Vec::new(),
            prepared: Vec::new(),
            allocations: Vec::new(),
            placements: Vec::new(),
            readback: None,
            stages: Vec::new(),
        }
    }

    /// Returns the same adapter reporting another target profile.
    ///
    /// The consumer-side half of the profile subject: an artifact assessed for
    /// one target and a host that is another one must not route, whichever side
    /// moved.
    #[must_use]
    pub fn on_profile(mut self, profile: TargetProfileRef) -> Self {
        self.environment.target_profile = profile;
        self
    }

    /// Returns the same adapter reporting another backend family.
    ///
    /// A host with no adapter for the family the artifact declares, which is the
    /// missing-adapter case rather than a mismatched one.
    #[must_use]
    pub fn for_backend(mut self, backend: tiler_artifact::program::BackendKey) -> Self {
        self.environment.backend = backend;
        self
    }
}

impl RuntimeAdapter for ScalarHostAdapter {
    type Refusal = Refusal;
    type Failure = ExecutionFault;
    type Completion = Completion;

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        self.stages.push(Stage::Bind);
        Ok(self.environment.clone())
    }

    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        self.stages.push(Stage::ValidatePayload);
        // ADR 0090 item 8, and the producing process's own validation counts for
        // nothing here: these bytes arrived from a file, and the artifact layer
        // carried them opaquely.
        let image = decode(entry.object()).map_err(Refusal::Payload)?;
        let validated = image.entry_for(entry).map_err(Refusal::Payload)?;
        self.validated.push(validated.clone());
        Ok(())
    }

    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        self.stages.push(Stage::ObserveLiveDevice);
        // Exhaustive on the kind and the dimension. Nothing this producer
        // packages reaches here, and an unrecognized answer is the honest one for
        // a row this adapter has never been given: reporting anything else would
        // be answering a question it was not asked.
        match request.requirement() {
            RouteRequirement::ResourceFloor(floor) => match floor.dimension() {
                RouteResourceDimension::SubgroupThreads => LiveDeviceObservation::Unrecognized,
            },
            RouteRequirement::BackendFeature(_) => LiveDeviceObservation::Unrecognized,
        }
    }

    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        self.stages.push(Stage::PrepareEntries);
        for (position, entry) in entries.iter().enumerate() {
            let grid_threads = entry.launch().grid_threads();
            if grid_threads > self.invocation_budget {
                return Err(Refusal::LaunchBeyondBudget {
                    entry: position,
                    grid_threads,
                    budget: self.invocation_budget,
                });
            }
        }
        // An interpreter's prepared state is the validated image itself. Saying
        // so is more honest than inventing a pipeline object it does not have.
        self.prepared.clone_from(&self.validated);
        Ok(())
    }

    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> u64 {
        self.stages.push(Stage::ObservePreparedEntry);
        u64::from(self.prepared[request.entry()].rows)
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

        for (position, entry) in preflight.entries().iter().enumerate() {
            let mut slots = Vec::with_capacity(entry.bindings().len());
            for binding in entry.bindings() {
                let bytes = binding.accessible_bytes();
                let offset = binding.accessible_offset();
                // Counted from the start of the allocation rather than the start
                // of the value: a binding may address a window at a nonzero
                // offset, and storage sized to the extent alone would be short by
                // exactly that offset.
                let reach = offset + bytes;
                let allocation = if addresses_program_input(binding.binding().target()) {
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
                let held =
                    u64::try_from(allocations[allocation].len()).expect("a small fixture buffer");
                if held < reach {
                    return Err(Refusal::UndersizedStorage {
                        entry: position,
                        slot: binding.slot(),
                        required: reach,
                        supplied: held,
                    });
                }
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
        let mut executed = 0_u64;
        for (position, entry) in routed.entries().iter().enumerate() {
            let launch = entry.launch();
            if launch.grid_threads() == 0 && launch.zero_work_skips_dispatch() {
                continue;
            }
            let prepared = &self.prepared[position];
            let slots = &self.placements[position];
            let placement = |transport: u32| {
                slots
                    .iter()
                    .zip(entry.bindings())
                    .find(|(_, binding)| binding.transport_slot() == transport)
                    .map(|(placement, _)| *placement)
                    .expect("payload validation proved this transport is one of the entry's")
            };
            let ran = crate::image::execute(
                position,
                prepared,
                placement(prepared.read_transport),
                placement(prepared.write_transport),
                &mut self.allocations,
                launch.grid_threads(),
                None,
            )?;
            // Terminal success, observed rather than assumed. Past the commit,
            // so this is reported and never resolved by routing somewhere else.
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
        } = self.readback.expect("the plan named a readback allocation");
        let result_bits = self.allocations[allocation][offset..]
            .as_chunks::<4>()
            .0
            .iter()
            .take(elements)
            .map(|chunk| u32::from_le_bytes(*chunk))
            .collect();
        Ok(Completion {
            result_bits,
            executed,
            profile_key: context.target_profile().key.as_str().to_owned(),
        })
    }
}
