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
//! arguments, derived by [`crate::bf16_vertical`] from the kernel and the
//! physical carrier. That separation is what lets the run perturb a derivation
//! and watch the composition fail: a width computed inside this module could
//! not be varied without varying the dispatch with it.
//!
//! # The submission contract
//!
//! A command buffer's terminal state is checked *before* the host reads
//! anything, and the accepted state is exactly `Completed`. A failed submission
//! leaves the output buffer holding whatever it held before, and comparing that
//! against the oracle would report a numerical disagreement for what is
//! actually a dispatch failure. There is no retry and no fallback: ADR 0051
//! permits a fallback only before the routing commit, and this module runs
//! entirely after it.

use std::fmt;

use metal::{
    Buffer, CommandBufferRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};
use tiler_metal::applicability::{
    AppleGpuFamilyConstant, MetalGpuFamilySupport, observe_highest_gpu_family,
};

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

/// What this binding could learn about the Apple families a device supports.
///
/// Two outcomes rather than a bare [`MetalGpuFamilySupport`], because "the
/// device named no family this vocabulary knows" and "this binding could not
/// ask" are different facts with different repairs, and a measurement boundary
/// that collapsed them would report an unasked question as an answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProbedGpuFamily {
    /// The governed vocabulary's own answer, from a walk this binding completed.
    Answered(MetalGpuFamilySupport),
    /// The vocabulary named an enumerator this binding cannot, so the device was
    /// never asked and there is no answer to report.
    Unnameable(AppleGpuFamilyConstant),
}

impl fmt::Display for ProbedGpuFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Answered(MetalGpuFamilySupport::Highest(family)) => {
                formatter.write_str(family.as_str())
            }
            Self::Answered(MetalGpuFamilySupport::NoneNamed) => {
                formatter.write_str("no named Apple family")
            }
            Self::Unnameable(constant) => write!(
                formatter,
                "unobserved: the governed vocabulary names MTLGPUFamily {constant}, which this \
                 binding cannot name, so this device was never asked",
            ),
        }
    }
}

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
fn pipeline_for(
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
}
