//! Executing one emitted entry point on this host's Metal device.
//!
//! Everything here is safe. The two raw-pointer sites the buffer round trip
//! needs live in [`crate::device_buffer`] and are reached through its byte
//! interface, so nothing in this module — and nothing in the conformance logic
//! above it — contains `unsafe`.
//!
//! # What this module decides, and what it deliberately does not
//!
//! It decides nothing numerical. The launch geometry, the argument-table
//! indices, the buffer capacities, and the readback length all arrive as
//! arguments — from [`crate::bf16_vertical`] and [`crate::serial_sum`] out of a
//! kernel and a physical carrier, and from [`crate::envelope`] out of a decoded
//! dispatch record. That separation is what lets a run perturb a derivation and
//! watch the composition fail: a width computed inside this module could not be
//! varied without varying the dispatch with it.
//!
//! It also decides no *comparison*. Every refusal a device merely supplies
//! numbers to — a workgroup against a pipeline's capacity, a reservation against
//! a device's threadgroup memory, a range against a buffer bound, an allocation
//! against the length it was asked for — is classified by
//! [`crate::device_preflight`], which is compiled on every host so those cases
//! run in the gate rather than only on hardware.
//!
//! # The submission contract
//!
//! A command buffer's terminal state is checked *before* the host reads
//! anything, and the accepted state is exactly `Completed`. A failed submission
//! leaves the output buffer holding whatever it held before, and comparing that
//! against the oracle would report a numerical disagreement for what is
//! actually a dispatch failure. There is no retry and no fallback: ADR 0051
//! permits a fallback only before the routing commit, and every submission this
//! module makes runs after it.

use std::fmt;

use metal::{
    Buffer, CommandBufferRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};
use tiler_metal::applicability::{
    AppleGpuFamilyConstant, MetalGpuFamily, MetalHostObservation, observe_highest_gpu_family,
};

use crate::applicability::{ProbedGpuFamily, observe_host_environment, stating_probed_family};
use crate::device_preflight::{PreflightRefusal, local_memory_fits, workgroup_fits};

/// Every Apple `MTLGPUFamily` enumerator `metal` 0.33.0 names, ascending.
///
/// The binding's vocabulary rather than Tiler's, joined to
/// [`MetalGpuFamily`] by Apple's own enumerator value: `MTLGPUFamily` is
/// `#[repr(i64)]` at the numbers `MTLDevice.h` gives, and
/// [`AppleGpuFamilyConstant`] carries the same numbers transcribed from the
/// same header. Hand-written because the binding's enum is `#[non_exhaustive]`,
/// publishes no iteration, and offers no `TryFrom`.
const BINDING_APPLE_FAMILIES: [MTLGPUFamily; 9] = [
    MTLGPUFamily::Apple1,
    MTLGPUFamily::Apple2,
    MTLGPUFamily::Apple3,
    MTLGPUFamily::Apple4,
    MTLGPUFamily::Apple5,
    MTLGPUFamily::Apple6,
    MTLGPUFamily::Apple7,
    MTLGPUFamily::Apple8,
    MTLGPUFamily::Apple9,
];

/// Names one governed Apple enumerator back into the type this binding takes.
///
/// `None` is an enumerator this binding cannot name, which is reachable rather
/// than theoretical — the macOS SDK declares `MTLGPUFamilyApple10 = 1010` and
/// this binding stops at `Apple9`. The caller reports it as an unasked question
/// rather than as a device that answered no.
const fn binding_apple_enumerator(constant: AppleGpuFamilyConstant) -> Option<MTLGPUFamily> {
    let mut index = 0;
    while index < BINDING_APPLE_FAMILIES.len() {
        let candidate = BINDING_APPLE_FAMILIES[index];
        if candidate as isize == constant.value() {
            return Some(candidate);
        }
        index += 1;
    }
    None
}

/// Compiles only while this binding can name every family the vocabulary probes.
///
/// The counted half is the literal, and the literal is the point: nothing else
/// in this file states how many families it expects to be able to ask about, so
/// a vocabulary that grew would otherwise reach the runtime refusal below on
/// every host with the tree green. Widening [`MetalGpuFamily`] is a build error
/// here, which is where whoever widens it learns that this crate needs a newer
/// `metal` binding before its applicability observation means anything again.
///
/// The sweep half is why bumping the literal is not the repair. It asks the same
/// question the probe asks at run time, of the same population, and keeps
/// failing until the binding genuinely names the added family — so the two
/// halves fail for different reasons and a build that passes both has actually
/// gained the enumerator rather than been told to expect one more.
///
/// Neither half makes the runtime refusal redundant. An assertion is a claim
/// about this build and can be relaxed in one line; what the probe *answers*
/// when it cannot name an enumerator is the part that must stay fail-closed on
/// its own.
const _: () = {
    assert!(
        MetalGpuFamily::COUNT == 5,
        "this crate expects the governed vocabulary to name five Apple families; `metal` 0.33.0 \
         stops at Apple9, so a widened vocabulary needs a newer binding here before the count is \
         raised",
    );
    let mut index = 0;
    while index < MetalGpuFamily::ALL.len() {
        assert!(
            binding_apple_enumerator(MetalGpuFamily::ALL[index].apple_constant()).is_some(),
            "`metal` 0.33.0 cannot name an Apple enumerator MetalGpuFamily::ALL declares, so this \
             crate would leave the GPU-family predicate unobserved on every host",
        );
        index += 1;
    }
};

/// Asks one device about exactly the families the governed vocabulary names.
///
/// One unnameable enumerator discards the whole walk rather than only its own
/// query, because [`observe_highest_gpu_family`] walks highest first and stops
/// at the first supported family: a family above the one that answered would
/// leave `Highest(lower)` an understatement wearing the shape of a
/// most-specific claim.
pub(crate) fn probe_apple_families(device: &Device) -> ProbedGpuFamily {
    let mut unnameable = None;
    let observed = observe_highest_gpu_family(|constant| {
        if let Some(enumerator) = binding_apple_enumerator(constant) {
            device.supports_family(enumerator)
        } else {
            unnameable = Some(constant);
            // Not an answer: the caller discards this walk entirely. Returning
            // `false` is only how this closure declines to end the walk on a
            // family it never asked about.
            false
        }
    });
    unnameable.map_or(
        ProbedGpuFamily::Answered(observed),
        ProbedGpuFamily::Unnameable,
    )
}

/// The launch this dispatch encodes, in threads.
///
/// Both numbers are the *kernel schedule's own* and arrive from the caller
/// rather than being chosen here, so a launch that disagreed with the schedule
/// would be a defect in the derivation rather than in this encoder.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Launch {
    /// Total threads the grid covers.
    pub(crate) grid_threads: u64,
    /// Threads per workgroup.
    pub(crate) threads_per_workgroup: u64,
}

/// The storage one dispatch binds, sized and strided by its caller.
#[derive(Clone, Debug)]
pub(crate) struct Storage {
    /// The exact bytes to place in the read buffer, already packed.
    pub(crate) operand_bytes: Vec<u8>,
    /// Argument-table index of the read buffer.
    pub(crate) operand_index: u64,
    /// Bytes to allocate for the write buffer.
    pub(crate) result_capacity: usize,
    /// Argument-table index of the write buffer.
    pub(crate) result_index: u64,
}

/// Why one device dispatch could not produce comparable bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DispatchError {
    /// The linked library could not be loaded from the compiled bytes.
    LibraryLoad(String),
    /// The entry-point symbol was not found in the loaded library.
    FunctionLookup(String),
    /// A compute pipeline could not be prepared for that function.
    Pipeline(String),
    /// The declared workgroup exceeds what this prepared pipeline admits.
    ///
    /// Checked before anything is encoded, so a schedule the compiled function
    /// cannot carry is a named refusal rather than a submission failure.
    WorkgroupTooWide {
        /// The entry-point symbol whose pipeline was asked.
        symbol: String,
        /// Threads per workgroup the schedule declares.
        declared: u64,
        /// The maximum this prepared pipeline admits.
        admitted: u64,
    },
    /// The launch covers no threads.
    ///
    /// `dispatchThreads` has no meaning at zero and inventing one thread would
    /// run a body the schedule did not ask for.
    EmptyLaunch,
    /// The command buffer did not reach `Completed`.
    Submission {
        /// The terminal or non-terminal status observed after the wait.
        status: &'static str,
        /// What that status means for a readback.
        detail: &'static str,
    },
}

impl fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LibraryLoad(cause) => write!(formatter, "the metallib did not load: {cause}"),
            Self::FunctionLookup(cause) => {
                write!(formatter, "the entry point was not found: {cause}")
            }
            Self::Pipeline(cause) => write!(formatter, "the pipeline did not prepare: {cause}"),
            Self::WorkgroupTooWide {
                symbol,
                declared,
                admitted,
            } => write!(
                formatter,
                "{symbol} declares {declared} threads per workgroup and the prepared pipeline \
                 admits {admitted}",
            ),
            Self::EmptyLaunch => formatter.write_str("the launch covers no threads"),
            Self::Submission { status, detail } => {
                write!(formatter, "submission ended in {status}: {detail}")
            }
        }
    }
}

impl std::error::Error for DispatchError {}

/// What a command buffer's status permits after the wait.
///
/// Three outcomes and deliberately no fourth: the runtime execution contract's
/// transition table says "never" for every post-commit transition, so there is
/// nothing here that could mean "try another route".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SubmissionOutcome {
    /// The one status that permits a readback.
    Completed,
    /// The device reported a terminal execution error.
    ExecutionError,
    /// The wait returned with the buffer in a non-terminal state.
    NotTerminal(&'static str),
}

/// Classifies one command-buffer status into what it permits.
///
/// Matched exhaustively and wildcard-free, so a status added to the binding is a
/// build error here rather than falling into whichever arm a catch-all named.
/// This is the one place a wrong answer would be read as arithmetic: a readback
/// taken from a buffer whose dispatch failed returns whatever the output held
/// before, which compares against the oracle as a numerical disagreement.
const fn submission_outcome(status: MTLCommandBufferStatus) -> SubmissionOutcome {
    match status {
        MTLCommandBufferStatus::Completed => SubmissionOutcome::Completed,
        MTLCommandBufferStatus::Error => SubmissionOutcome::ExecutionError,
        MTLCommandBufferStatus::NotEnqueued => SubmissionOutcome::NotTerminal("NotEnqueued"),
        MTLCommandBufferStatus::Enqueued => SubmissionOutcome::NotTerminal("Enqueued"),
        MTLCommandBufferStatus::Committed => SubmissionOutcome::NotTerminal("Committed"),
        MTLCommandBufferStatus::Scheduled => SubmissionOutcome::NotTerminal("Scheduled"),
    }
}

/// Prepares a compute pipeline for one named function of one object image.
pub(crate) fn pipeline_for(
    device: &Device,
    object: &[u8],
    symbol: &str,
) -> Result<ComputePipelineState, DispatchError> {
    let library = device
        .new_library_with_data(object)
        .map_err(DispatchError::LibraryLoad)?;
    let function = library
        .get_function(symbol, None)
        .map_err(DispatchError::FunctionLookup)?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    device
        .new_compute_pipeline_state(&descriptor)
        .map_err(DispatchError::Pipeline)
}

/// Submits one encoded command buffer and reads the result buffer back.
fn submit(
    device: &Device,
    result: &Buffer,
    read_bytes: usize,
    encode: impl FnOnce(&CommandBufferRef),
) -> Result<Vec<u8>, DispatchError> {
    let queue = device.new_command_queue();
    let command_buffer = queue.new_command_buffer();
    encode(command_buffer);
    command_buffer.commit();
    command_buffer.wait_until_completed();

    match submission_outcome(command_buffer.status()) {
        SubmissionOutcome::Completed => Ok(crate::device_buffer::read_bytes(result, read_bytes)),
        SubmissionOutcome::ExecutionError => Err(DispatchError::Submission {
            status: "Error",
            detail: "the device reported an execution error for this command buffer",
        }),
        SubmissionOutcome::NotTerminal(status) => Err(DispatchError::Submission {
            status,
            detail: "the wait returned with the command buffer in a non-terminal state",
        }),
    }
}

/// Runs one entry point over one packed operand run and returns the result bytes.
///
/// # Errors
///
/// Returns the named refusal for a library, function, pipeline, workgroup,
/// launch, or submission failure. Every refusal before the commit is checked
/// before anything is encoded.
pub(crate) fn run_entry_point(
    device: &Device,
    metallib: &[u8],
    symbol: &str,
    storage: &Storage,
    launch: Launch,
) -> Result<Vec<u8>, DispatchError> {
    if launch.grid_threads == 0 {
        return Err(DispatchError::EmptyLaunch);
    }
    let pipeline = pipeline_for(device, metallib, symbol)?;
    let admitted = pipeline.max_total_threads_per_threadgroup();
    if launch.threads_per_workgroup > admitted {
        return Err(DispatchError::WorkgroupTooWide {
            symbol: symbol.to_owned(),
            declared: launch.threads_per_workgroup,
            admitted,
        });
    }

    // `max(1)` because Metal refuses a zero-length allocation, and a run whose
    // corpus is empty is a caller error the comparison reports rather than one
    // this allocation should invent a byte for.
    let operand_capacity = u64::try_from(storage.operand_bytes.len())
        .expect("a packed operand run fits a u64")
        .max(1);
    let result_capacity = u64::try_from(storage.result_capacity)
        .expect("a result capacity fits a u64")
        .max(1);
    let operands = device.new_buffer(operand_capacity, MTLResourceOptions::StorageModeShared);
    let results = device.new_buffer(result_capacity, MTLResourceOptions::StorageModeShared);
    crate::device_buffer::write_bytes(&operands, &storage.operand_bytes);

    let bytes = submit(
        device,
        &results,
        storage.result_capacity,
        |command_buffer| {
            let encoder = command_buffer.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(&pipeline);
            encoder.set_buffer(storage.operand_index, Some(&operands), 0);
            encoder.set_buffer(storage.result_index, Some(&results), 0);
            encoder.dispatch_threads(
                MTLSize::new(launch.grid_threads, 1, 1),
                MTLSize::new(launch.threads_per_workgroup, 1, 1),
            );
            encoder.end_encoding();
        },
    )?;

    // Both buffers are still live here, which is the retention this function
    // owes: `submit` waits for the command buffer's terminal state before it
    // returns, so every buffer the encode bound is held across the whole device
    // lifetime of the work that reads it.
    drop(operands);
    drop(results);
    Ok(bytes)
}

/// What the device reported about itself, recorded rather than checked.
///
/// **No artifact field names a required GPU family, a threadgroup floor, or a
/// buffer-length floor**, so there is nothing here to compare those against.
/// Declaring a requirement the artifact never made would be inventing one, so
/// these are provenance: they say which device produced a measurement, and they
/// are what a future artifact-side family declaration would be checked against.
///
/// The two limits that *do* have an artifact-side counterpart — the pipeline's
/// threadgroup capacity and the per-buffer length bound — are compared by
/// [`crate::device_preflight`] rather than recorded here, because a declared
/// launch and a declared accessible range are things an artifact does state.
#[derive(Clone, Debug)]
pub(crate) struct DeviceFacts {
    /// The name the device reports for itself.
    pub(crate) name: String,
    /// The widest threadgroup this device admits at all.
    pub(crate) max_threads_per_threadgroup: u64,
    /// Threadgroup memory this device admits per threadgroup.
    pub(crate) max_threadgroup_memory_length: u64,
    /// Bytes this device holds in one buffer.
    pub(crate) max_buffer_length: u64,
    /// The working set this device recommends staying within.
    pub(crate) recommended_working_set: u64,
    /// The highest Apple family it claims, or why it was never asked.
    pub(crate) apple_family: ProbedGpuFamily,
}

impl fmt::Display for DeviceFacts {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} ({}), {} thread(s) per threadgroup, {} byte(s) of threadgroup memory, buffers to \
             {} byte(s), working set {} byte(s)",
            self.name,
            self.apple_family,
            self.max_threads_per_threadgroup,
            self.max_threadgroup_memory_length,
            self.max_buffer_length,
            self.recommended_working_set,
        )
    }
}

/// Observes every predicate the first Metal profile's applicability row names.
///
/// The device contributes exactly two of them — the name it reports for itself
/// and the Apple family it claims — and nothing else about the device reaches
/// the policy. In particular the registry ID does not: ADR 0086 excludes it by
/// name, because the retained records report two different values for this same
/// named Apple M4 Max.
///
/// The probed family is returned beside the observation rather than only folded
/// into it, so a caller can report the exact enumerator a refusal is about; a
/// probe hidden in here could only leave the predicate unobserved without saying
/// why.
pub(crate) fn observe_metal_host(device: &Device) -> (MetalHostObservation, ProbedGpuFamily) {
    let probed = probe_apple_families(device);
    let observation = stating_probed_family(
        observe_host_environment().observing_device_name(device.name()),
        probed,
    );
    (observation, probed)
}

/// Reads what this device reports about itself.
pub(crate) fn device_facts(device: &Device) -> DeviceFacts {
    DeviceFacts {
        name: device.name().to_owned(),
        max_threads_per_threadgroup: device.max_threads_per_threadgroup().width,
        max_threadgroup_memory_length: device.max_threadgroup_memory_length(),
        max_buffer_length: device.max_buffer_length(),
        recommended_working_set: device.recommended_max_working_set_size(),
        apple_family: probe_apple_families(device),
    }
}

/// One stage of a multi-stage dispatch, with every device object it needs.
///
/// Resolved before the submission, so the encode allocates nothing, looks
/// nothing up, and has no failure of its own to report. That is the property the
/// staging exists for, and it is what lets a caller that holds a routing
/// authority discharge every device-decidable obligation while abandoning is
/// still permitted.
pub(crate) struct PreparedStage {
    /// The compiled pipeline this stage encodes.
    pub(crate) pipeline: ComputePipelineState,
    /// Buffers in this stage's own binding order, each with the argument-table
    /// index it binds at and the byte offset it is bound from.
    pub(crate) placements: Vec<(u64, Buffer, u64)>,
    /// Threads the stage's launch covers.
    pub(crate) grid_threads: u64,
    /// Threads per workgroup the stage declares.
    pub(crate) threads_per_workgroup: u64,
    /// This stage covers no threads and its declaration says to skip its
    /// dispatch.
    ///
    /// Its buffers are still allocated and still retained: an empty producing
    /// stage shares its intermediate with the consumer that follows, and the
    /// consumer must bind an allocation rather than nothing.
    pub(crate) skipped: bool,
}

/// Checks one prepared pipeline against the geometry and reservation a stage
/// declares, before anything is encoded.
///
/// The declared workgroup goes against what *this* pipeline admits and the
/// reserved threadgroup memory against what *this device* admits, which are two
/// different capacities from two different authorities. A tree declaring more
/// participants than the compiled function accepts, or reserving more local
/// memory than the device has, is a named refusal here rather than a submission
/// failure later.
///
/// # Errors
///
/// Returns the refusal whichever comparison declined, naming the entry it came
/// from.
pub(crate) fn admit_stage(
    entry: usize,
    symbol: &str,
    threads_per_workgroup: u64,
    pipeline: &ComputePipelineState,
    facts: &DeviceFacts,
) -> Result<(), PreflightRefusal> {
    workgroup_fits(
        entry,
        symbol,
        threads_per_workgroup,
        pipeline.max_total_threads_per_threadgroup(),
    )?;
    local_memory_fits(
        entry,
        symbol,
        pipeline.static_threadgroup_memory_length(),
        facts.max_threadgroup_memory_length,
    )
}

/// Submits every stage in order and reads `read_bytes` back out of `output`.
///
/// **One encoder per stage, and that is the ordering guarantee.** Commands
/// within a single compute encoder are not ordered against each other unless the
/// encoder's dispatch type says so, and a stage reading what an earlier stage
/// wrote must not overlap it. Metal orders *encoders* within a command buffer
/// unconditionally, with an implicit barrier between them, so a separate encoder
/// per stage needs no assumption about dispatch type at all.
///
/// # Errors
///
/// Returns [`DispatchError::EmptyLaunch`] for a stage that covers no threads and
/// is not marked skipped, and [`DispatchError::Submission`] when the command
/// buffer does not reach `Completed`.
pub(crate) fn run_stages(
    device: &Device,
    stages: &[PreparedStage],
    output: &Buffer,
    read_bytes: usize,
) -> Result<Vec<u8>, DispatchError> {
    for stage in stages {
        if stage.grid_threads == 0 && !stage.skipped {
            return Err(DispatchError::EmptyLaunch);
        }
    }
    let bytes = submit(device, output, read_bytes, |command_buffer| {
        for stage in stages {
            // Skipped stages are not encoded at all. Encoding an empty encoder
            // would be harmless and pointless; encoding a zero-thread dispatch
            // is what the guard above already refused.
            if stage.skipped {
                continue;
            }
            let encoder = command_buffer.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(&stage.pipeline);
            for (index, buffer, offset) in &stage.placements {
                encoder.set_buffer(*index, Some(buffer), *offset);
            }
            encoder.dispatch_threads(
                MTLSize::new(stage.grid_threads, 1, 1),
                MTLSize::new(stage.threads_per_workgroup, 1, 1),
            );
            encoder.end_encoding();
        }
    })?;
    // `stages` is still live here, which is the retention this function owes:
    // `submit` waits for the command buffer's terminal state before it returns,
    // so every buffer an encode bound is held across the whole device lifetime
    // of the work that reads it.
    Ok(bytes)
}

/// Observes the terminal-status check refusing a live command buffer that has
/// not reached a terminal state.
///
/// **The contract's own warning case, injected rather than argued.**
/// `waitUntilCompleted` returns no success value, so the runtime execution
/// contract records that "a pre-wait non-error status is not evidence of
/// successful completion". A command buffer that has just been created and never
/// committed is exactly that: alive, valid, and carrying a status that must not
/// admit a readback. Nothing is committed and nothing is encoded, so the probe
/// costs one allocation and reaches no GPU work.
///
/// **The terminal `Error` state is deliberately not injected**, and the boundary
/// is stated rather than left as apparent coverage: forcing a command buffer to
/// fail means provoking a GPU fault, which risks a device reset and would not
/// reproduce. `tests::one_status_permits_a_readback_and_none_permits_a_retry`
/// covers that arm over the complete status vocabulary without hardware.
///
/// # Errors
///
/// Returns the status name when an uncommitted buffer classifies as terminal,
/// which would mean a probe that cannot fail.
pub(crate) fn probe_uncommitted_status(device: &Device) -> Result<&'static str, &'static str> {
    let queue = device.new_command_queue();
    let uncommitted = queue.new_command_buffer();
    match submission_outcome(uncommitted.status()) {
        SubmissionOutcome::NotTerminal(reported) => Ok(reported),
        SubmissionOutcome::Completed => Err("Completed"),
        SubmissionOutcome::ExecutionError => Err("Error"),
    }
}

#[cfg(test)]
mod tests {
    use tiler_metal::applicability::MetalGpuFamily;

    use super::{
        BINDING_APPLE_FAMILIES, MTLCommandBufferStatus, binding_apple_enumerator,
        submission_outcome,
    };

    /// Every status the binding declares classifies, exactly one reads back, and
    /// none permits a retry.
    ///
    /// The complete population rather than a sample, so this establishes the
    /// classification for every input that exists. The retry half is structural
    /// — `SubmissionOutcome` has no such variant — and is stated because a later
    /// edit adding one would compile.
    #[test]
    fn one_status_permits_a_readback_and_none_permits_a_retry() {
        let population = [
            (MTLCommandBufferStatus::NotEnqueued, "NotEnqueued"),
            (MTLCommandBufferStatus::Enqueued, "Enqueued"),
            (MTLCommandBufferStatus::Committed, "Committed"),
            (MTLCommandBufferStatus::Scheduled, "Scheduled"),
            (MTLCommandBufferStatus::Completed, "Completed"),
            (MTLCommandBufferStatus::Error, "Error"),
        ];
        assert_eq!(population.len(), 6, "the binding declares six statuses");

        let mut readable = 0;
        for (status, name) in population {
            match submission_outcome(status) {
                super::SubmissionOutcome::Completed => {
                    readable += 1;
                    assert_eq!(name, "Completed");
                }
                super::SubmissionOutcome::ExecutionError => assert_eq!(name, "Error"),
                super::SubmissionOutcome::NotTerminal(reported) => {
                    assert_eq!(reported, name);
                    assert!(!matches!(name, "Completed" | "Error"));
                }
            }
        }
        assert_eq!(readable, 1, "exactly one status may be read back from");
    }

    /// The binding names every family the governed vocabulary probes.
    ///
    /// Without this the family walk would silently stop asking about a family
    /// added to [`MetalGpuFamily`], and the measurement boundary would report an
    /// understatement as a most-specific claim.
    #[test]
    fn the_binding_names_every_family_the_governed_vocabulary_probes() {
        assert_eq!(
            BINDING_APPLE_FAMILIES.len(),
            9,
            "`metal` 0.33.0 names nine Apple enumerators",
        );
        let mut named = 0;
        for family in MetalGpuFamily::ALL {
            assert!(
                binding_apple_enumerator(family.apple_constant()).is_some(),
                "this binding cannot name {}, so a device would never be asked about it",
                family.as_str(),
            );
            named += 1;
        }
        assert_eq!(
            named,
            MetalGpuFamily::COUNT,
            "every governed family was walked, not the ones that happened to match",
        );
    }

    /// The unnameable case is reachable rather than theoretical.
    ///
    /// Apple declares `MTLGPUFamilyApple10 = 1010` in the macOS 26.5 SDK and
    /// this binding stops at Apple9, so the moment the governed vocabulary
    /// widens, the probe meets an enumerator it cannot name. Pinned here because
    /// every refusal built on that outcome — `ProbedGpuFamily::Unnameable`, the
    /// unobserved GPU-family predicate, and the `Unrecognized` family row — is
    /// only worth writing while it is true. A binding that gained the enumerator
    /// makes this fail and say so.
    #[test]
    fn this_binding_cannot_name_the_family_apple_declares_above_its_last() {
        assert!(
            !BINDING_APPLE_FAMILIES
                .iter()
                .any(|enumerator| *enumerator as isize == 1010),
            "`metal` 0.33.0 gained MTLGPUFamilyApple10; the vocabulary can now widen to it and \
             this crate's unnameable-enumerator refusal is no longer reachable through it",
        );
    }
}
