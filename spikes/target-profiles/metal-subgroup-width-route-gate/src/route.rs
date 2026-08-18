//! Drives one hand-assembled Metal artifact through the ordinary loader route
//! on the real device, answering the prepared subgroup-width requests from the
//! exact prepared pipelines.
//!
//! The stage order is the loader's own: decode, prepare (payload, identity,
//! variant selection, ABI evaluation), live-device rows (none declared here),
//! **prepare every exact pipeline**, observe/compare the prepared-entry
//! properties, plan buffers, one-way commit, allocate nothing further, encode,
//! dispatch. Every refusal this spike demonstrates returns from
//! `resolve_target_properties`, which runs strictly before `Preflight::commit`
//! is reachable — the pre-commit claim is structural in this function's shape,
//! not a log line.

use std::cell::RefCell;
use std::collections::BTreeMap;

use metal::{
    Buffer, CommandQueue, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};
use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArithmeticType, BindingTarget, DeferredPredicateSpec,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LoadRejection, Preflight,
    PreparedEntryObservation, RoutedEntry, TargetPropertyRequest,
};

use crate::device_io::{read_bytes, write_bytes};
use crate::fixture;

/// The governed prepared-entry key the accepted width gate dispatches on.
pub const GOVERNED_SUBGROUP_KEY: &str = "tiler.target.prepared-entry.subgroup-width.v1";
/// Provider namespace answering the governed key.
pub const PROVIDER_NAMESPACE: &str = "tiler";
/// Provider name answering the governed key.
pub const PROVIDER_NAME: &str = "prepared-entry-properties";
/// Provider revision answering the governed key.
pub const PROVIDER_REVISION: u32 = 1;

/// How the demonstration's observer answers a prepared-entry request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserverMode {
    /// The governed dispatch: the exact named entry's own retained pipeline's
    /// `threadExecutionWidth`, and `Unrecognized` for any other ownership.
    Exact,
    /// An adapter predating the subgroup dispatch: every request is
    /// `Unrecognized`, including the governed key.
    PreSubgroupAdapter,
    /// The cross-pipeline perturbation: every governed request is answered
    /// from entry 0's retained pipeline, whichever entry the request names.
    FirstEntry,
}

/// One recorded observation the closure made, for the retained log.
#[derive(Clone, Debug)]
pub struct ObservationRecord {
    /// The entry position the loader's request named.
    pub entry: usize,
    /// The requested property key.
    pub key: String,
    /// What the observer answered.
    pub answer: String,
}

/// What a committed, dispatched route produced.
#[derive(Clone, Debug)]
pub struct CommittedRun {
    /// The program output read back from the device.
    pub output: Vec<f32>,
    /// `threadExecutionWidth` of each prepared entry pipeline, route order.
    pub widths: Vec<u64>,
}

/// One route attempt's complete evidence.
pub struct CaseOutcome {
    /// Every prepared-entry request the loader issued, in order.
    pub observations: Vec<ObservationRecord>,
    /// The committed run, or the pre-commit refusal.
    pub result: Result<CommittedRun, LoadRejection>,
}

/// The demonstration's route environment: the fixture's own declared profile,
/// Metal's governed backend and representation keys, and an F32 dispatch row.
fn environment() -> ExecutionEnvironment {
    ExecutionEnvironment {
        target_profile: fixture::profile(),
        backend: fixture::metal_backend(),
        representation: fixture::metal_representation(),
        dtype_dispatch: BTreeMap::from([(ArithmeticType::F32, DTypeDispatch::Dispatchable)]),
    }
}

/// Binds the ABI facts from the artifact's own declared interface.
fn bind_facts(program: &DecodedProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_input_shape(input.key(), input.shape())
            .expect("the declared interface binds");
    }
    binder.build()
}

/// Builds every exact entry pipeline before any deferred property is answered.
fn prepare_pipelines(device: &Device, entries: &[RoutedEntry<'_>]) -> Vec<ComputePipelineState> {
    entries
        .iter()
        .map(|entry| {
            let library = device
                .new_library_with_data(entry.object())
                .expect("the carried metallib loads");
            let function = library
                .get_function(entry.entry_symbol(), None)
                .expect("the entry symbol resolves");
            let descriptor = ComputePipelineDescriptor::new();
            descriptor.set_compute_function(Some(&function));
            device
                .new_compute_pipeline_state(&descriptor)
                .expect("the pipeline prepares")
        })
        .collect()
}

fn observe(
    request: TargetPropertyRequest<'_>,
    pipelines: &[ComputePipelineState],
    mode: ObserverMode,
    log: &RefCell<Vec<ObservationRecord>>,
) -> PreparedEntryObservation {
    let query = request.requirement().query();
    let provider = query.provider();
    let owned = provider.namespace() == PROVIDER_NAMESPACE
        && provider.name() == PROVIDER_NAME
        && provider.revision() == PROVIDER_REVISION
        && query.key().as_str() == GOVERNED_SUBGROUP_KEY;
    let observation = match (owned, mode) {
        (false, _) | (true, ObserverMode::PreSubgroupAdapter) => {
            PreparedEntryObservation::Unrecognized
        }
        (true, ObserverMode::Exact) => {
            PreparedEntryObservation::Quantity(pipelines[request.entry()].thread_execution_width())
        }
        (true, ObserverMode::FirstEntry) => {
            PreparedEntryObservation::Quantity(pipelines[0].thread_execution_width())
        }
    };
    log.borrow_mut().push(ObservationRecord {
        entry: request.entry(),
        key: query.key().as_str().to_owned(),
        answer: format!("{observation:?}"),
    });
    observation
}

/// The demonstration operands: 1.0 through 6.0 over the fixture's `[2, 3]`.
pub fn operands() -> Vec<f32> {
    (1..=6).map(|value| value as f32).collect()
}

/// The strict reference: `out[row] = sum over 3 columns of (in * 2.0 + 1.0)`.
///
/// Every value is exactly representable, so the comparison below is bit-exact
/// rather than a tolerance.
pub fn expected() -> Vec<f32> {
    let operands = operands();
    (0..2)
        .map(|row| {
            (0..3)
                .map(|column| operands[row * 3 + column] * 2.0 + 1.0)
                .sum()
        })
        .collect()
}

/// Routes one assembled artifact carrying `rows`, end to end.
pub fn attempt(
    device: &Device,
    queue: &CommandQueue,
    metallib: &[u8],
    rows: Vec<DeferredPredicateSpec>,
    mode: ObserverMode,
) -> CaseOutcome {
    let mut spec = fixture::FixtureSpec::metal(fixture::PackagedPlan::Materialized);
    spec.code = metallib.to_vec();
    spec.entries[0].symbol = "route_pointwise_f32".to_owned();
    spec.entries[1].symbol = "route_reduce_f32".to_owned();
    spec.deferred_predicates = rows;
    let built = fixture::assemble(&spec);

    let mut program =
        DecodedProgram::decode(&built.bytes, 0).expect("the assembled artifact decodes");
    let facts = bind_facts(&program);
    let log = RefCell::new(Vec::new());

    let result = (|| {
        let qualification = program.prepare(&environment(), &built.expected, &facts)?;
        // This artifact declares no live-device row; the stage is still passed
        // through rather than skipped, so a variant that did declare one could
        // not reach the prepared stage unchecked.
        let preparation = qualification.resolve_live_device_requirements(|_| {
            tiler_runtime::load::LiveDeviceObservation::Unrecognized
        })?;
        let pipelines = prepare_pipelines(device, preparation.entries());
        let widths: Vec<u64> = pipelines
            .iter()
            .map(|pipeline| pipeline.thread_execution_width())
            .collect();
        // Every demonstrated refusal returns from this call, before
        // `Preflight::commit` below is reachable: the pre-commit claim is the
        // shape of this closure.
        let preflight = preparation
            .resolve_target_properties(|request| observe(request, &pipelines, mode, &log))?;
        let output = commit_and_dispatch(device, queue, preflight, &pipelines);
        Ok(CommittedRun { output, widths })
    })();

    CaseOutcome {
        observations: log.into_inner(),
        result,
    }
}

/// Plans every buffer pre-commit, commits, encodes, and reads the output back.
fn commit_and_dispatch(
    device: &Device,
    queue: &CommandQueue,
    preflight: Preflight<'_>,
    pipelines: &[ComputePipelineState],
) -> Vec<f32> {
    let entries = preflight.entries();
    let operand_bytes: Vec<u8> = operands().iter().flat_map(|v| v.to_le_bytes()).collect();

    // Shared allocations first: one buffer, referenced by both slots.
    let mut storage: Vec<Vec<Option<Buffer>>> = entries
        .iter()
        .map(|entry| vec![None; entry.bindings().len()])
        .collect();
    for shared in preflight.shared_allocations() {
        let (producer, consumer) = (shared.producer(), shared.consumer());
        let needed = |slot: (usize, usize)| {
            let binding = entries[slot.0].bindings()[slot.1];
            binding.accessible_offset() + binding.accessible_bytes()
        };
        let length = needed((producer.entry(), producer.slot()))
            .max(needed((consumer.entry(), consumer.slot())));
        let buffer = device.new_buffer(length.max(1), MTLResourceOptions::StorageModePrivate);
        storage[producer.entry()][producer.slot()] = Some(buffer.clone());
        storage[consumer.entry()][consumer.slot()] = Some(buffer);
    }

    let mut output = None;
    let mut output_bytes = 0_usize;
    let mut stages = Vec::with_capacity(entries.len());
    for (position, entry) in entries.iter().enumerate() {
        let mut placements = Vec::with_capacity(entry.bindings().len());
        for binding in entry.bindings() {
            let needed = (binding.accessible_offset() + binding.accessible_bytes()).max(1);
            let buffer = if let Some(shared) = storage[position][binding.slot()].clone() {
                shared
            } else {
                match binding.binding().target() {
                    BindingTarget::ProgramInput(_) => {
                        let buffer =
                            device.new_buffer(needed, MTLResourceOptions::StorageModeShared);
                        write_bytes(&buffer, &operand_bytes);
                        buffer
                    }
                    BindingTarget::ProgramOutput(_) => {
                        let buffer =
                            device.new_buffer(needed, MTLResourceOptions::StorageModeShared);
                        output = Some(buffer.clone());
                        output_bytes = usize::try_from(needed).expect("a small output");
                        buffer
                    }
                    BindingTarget::Internal => {
                        device.new_buffer(needed, MTLResourceOptions::StorageModePrivate)
                    }
                }
            };
            placements.push((
                u64::from(binding.transport_slot()),
                buffer,
                binding.accessible_offset(),
            ));
        }
        let launch = entry.launch();
        stages.push((
            pipelines[position].clone(),
            placements,
            launch.grid_threads(),
            launch.threads_per_workgroup(),
        ));
    }
    let output = output.expect("the route binds the program output");

    // ---- the routing commit, one way ------------------------------------
    // Everything the encode touches exists; nothing after this may refuse.
    // `commit` consumes the `Preflight`, so a second authority for this
    // attempt does not compile — ADR 0051 expressed structurally.
    let _routed = preflight.commit();
    let command_buffer = queue.new_command_buffer();
    for (pipeline, placements, grid, per_workgroup) in &stages {
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(pipeline);
        for (index, buffer, offset) in placements {
            encoder.set_buffer(*index, Some(buffer), *offset);
        }
        encoder.dispatch_threads(
            MTLSize::new(*grid, 1, 1),
            MTLSize::new(*per_workgroup, 1, 1),
        );
        encoder.end_encoding();
    }
    command_buffer.commit();
    command_buffer.wait_until_completed();
    assert_eq!(
        command_buffer.status(),
        MTLCommandBufferStatus::Completed,
        "the committed dispatch reached its terminal state",
    );

    let bytes = read_bytes(&output, output_bytes);
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}
