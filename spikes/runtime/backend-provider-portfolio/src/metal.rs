//! Metal payload production and, on eligible hosts, live dispatch.

use std::collections::BTreeMap;
use std::fmt;

use tiler_artifact::program::{
    ArithmeticType, BindingTarget, PayloadContent, RecordedArtifactProgramIdentity,
    TargetProfileRef,
};
use tiler_build::{
    BoundMetalCompileDeclaration, DTypeDispatchability, accept_or_publish_metal_plan,
};
use tiler_cache::expansion::ExpansionCache;
use tiler_compiler::session::PlanAlternative;
use tiler_ir::semantic::SemanticProgram;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::OptimizationLevel;
use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter, route_with_adapter};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest,
    LoadRejection, Preflight, PreparedEntryObservation, RoutedDispatch, RoutedEntry,
    TargetPropertyRequest, VariantIneligibility,
};

use crate::cpu::{SOLE_DELIVERY, bind_facts};

/// Governed Metal backend family.
pub const BACKEND_KEY: &str = "tiler.metal";
/// Governed Metal executable representation.
pub const REPRESENTATION_KEY: &str = "metallib";

/// Why the Metal path could not produce or execute a payload.
#[derive(Clone, Debug)]
pub enum MetalError {
    /// The Apple toolchain is not usable on this host.
    Unavailable(String),
    /// Payload production failed.
    Produce(String),
    /// The loader refused the route.
    Load(String),
    /// The adapter refused before the commit.
    Adapter(String),
    /// The committed dispatch failed.
    Execute(String),
    /// Output bits disagreed with the independent reference.
    Mismatch {
        /// Index of the first disagreeing element.
        index: usize,
        /// Bits Metal produced.
        actual: u32,
        /// Bits the reference requires.
        expected: u32,
    },
}

impl fmt::Display for MetalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(formatter, "metal.unavailable: {message}"),
            Self::Produce(message) => write!(formatter, "metal.produce: {message}"),
            Self::Load(message) => write!(formatter, "metal.load: {message}"),
            Self::Adapter(message) => write!(formatter, "metal.adapter: {message}"),
            Self::Execute(message) => write!(formatter, "metal.execute: {message}"),
            Self::Mismatch {
                index,
                actual,
                expected,
            } => write!(
                formatter,
                "metal.compare: element {index} is 0x{actual:08x} and the reference requires 0x{expected:08x}",
            ),
        }
    }
}

impl std::error::Error for MetalError {}

/// One produced Metal artifact.
pub struct ProducedMetal {
    /// Encoded envelope bytes of the single-family Metal artifact.
    pub bytes: Vec<u8>,
    /// Identity recorded beside those bytes.
    #[allow(
        dead_code,
        reason = "retained so a later probe can compare Metal-only identity"
    )]
    pub expected: RecordedArtifactProgramIdentity,
    /// Carried payload reused when the portfolio is assembled.
    pub content: PayloadContent,
}

/// Assembles one Metal plan through `accept_or_publish_metal_plan`.
pub fn assemble(
    cache: &ExpansionCache,
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<ProducedMetal, MetalError> {
    let accepted = accept_or_publish_metal_plan(
        cache,
        &Toolchain::system(),
        semantic,
        plan,
        std::slice::from_ref(declaration),
        OptimizationLevel::Default,
    )
    .map_err(|error| {
        let message = error.to_string();
        if message.contains("xcrun")
            || message.contains("toolchain")
            || message.contains("unavailable")
        {
            MetalError::Unavailable(message)
        } else {
            MetalError::Produce(message)
        }
    })?;
    let artifact = accepted.artifact();
    let bytes = artifact
        .encode()
        .map_err(|error| MetalError::Produce(error.to_string()))?;
    let expected =
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .map_err(|error| MetalError::Produce(error.to_string()))?;
    let decoded = accepted.decoded();
    let metadata = decoded
        .payload_metadata(0)
        .cloned()
        .ok_or_else(|| MetalError::Produce("the Metal payload carries no metadata".into()))?;
    let object = decoded
        .payload_object(0)
        .ok_or_else(|| MetalError::Produce("the Metal payload carries no object".into()))?
        .to_vec();
    Ok(ProducedMetal {
        bytes,
        expected,
        content: PayloadContent {
            metadata,
            code: object,
        },
    })
}

/// Maps the declaration's compile-profile dtype rows onto the host vocabulary.
#[must_use]
pub fn dtype_dispatch(
    declaration: &BoundMetalCompileDeclaration,
) -> BTreeMap<ArithmeticType, DTypeDispatch> {
    declaration
        .dtype_dispatchability_rows()
        .into_iter()
        .map(|(arithmetic, verdict)| {
            let dispatch = match verdict {
                DTypeDispatchability::Dispatchable => DTypeDispatch::Dispatchable,
                DTypeDispatchability::Unsupported => DTypeDispatch::Unsupported,
            };
            (arithmetic, dispatch)
        })
        .collect()
}

/// Returns Metal's governed backend key.
#[must_use]
pub fn backend() -> tiler_artifact::program::BackendKey {
    tiler_artifact::program::BackendKey::new(BACKEND_KEY).expect("a governed backend key")
}

/// Returns Metal's governed representation key.
#[must_use]
pub fn representation() -> tiler_artifact::program::RepresentationKey {
    tiler_artifact::program::RepresentationKey::new(REPRESENTATION_KEY)
        .expect("a governed representation key")
}

/// Reports whether this host can bind a Metal device.
pub fn device_available() -> Result<(), MetalError> {
    #[cfg(target_os = "macos")]
    {
        metal::Device::system_default()
            .map(|_| ())
            .ok_or_else(|| MetalError::Unavailable("no Metal device is present".into()))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(MetalError::Unavailable(
            "Metal execution is macOS-only".into(),
        ))
    }
}

/// Routes one Metal artifact on an eligible host and compares the result.
pub fn route_and_compare(
    bytes: &[u8],
    expected_identity: &RecordedArtifactProgramIdentity,
    environment_profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    reference: &[u32],
) -> Result<Vec<u32>, MetalError> {
    device_available()?;
    #[cfg(target_os = "macos")]
    {
        let mut program = DecodedProgram::decode(bytes, SOLE_DELIVERY)
            .map_err(|rejection| MetalError::Load(rejection.to_string()))?;
        let facts = bind_facts(&program);
        let mut adapter = MetalAdapter::new(environment_profile, dtype_dispatch);
        let bits = route_with_adapter(&mut program, &mut adapter, expected_identity, &facts)
            .map_err(|failure| match failure {
                tiler_runtime::adapter::AdapterRouteFailure::Load(rejection) => {
                    MetalError::Load(rejection.to_string())
                }
                other => MetalError::Adapter(format!("{other:?}")),
            })?;
        if bits.len() != reference.len() {
            return Err(MetalError::Execute(format!(
                "result length {} disagrees with the reference {}",
                bits.len(),
                reference.len()
            )));
        }
        for (index, (actual, expected)) in bits.iter().zip(reference).enumerate() {
            if actual != expected {
                return Err(MetalError::Mismatch {
                    index,
                    actual: *actual,
                    expected: *expected,
                });
            }
        }
        Ok(bits)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (
            bytes,
            expected_identity,
            environment_profile,
            dtype_dispatch,
            reference,
        );
        Err(MetalError::Unavailable(
            "Metal execution is macOS-only".into(),
        ))
    }
}

/// Preflights one artifact under a Metal environment and reports the refusal.
pub fn preflight_refusal(
    bytes: &[u8],
    expected: &RecordedArtifactProgramIdentity,
    environment: &ExecutionEnvironment,
) -> Result<LoadRejection, MetalError> {
    let mut program = DecodedProgram::decode(bytes, SOLE_DELIVERY)
        .map_err(|rejection| MetalError::Load(rejection.to_string()))?;
    let facts = bind_facts(&program);
    match program.preflight(environment, expected, &facts) {
        Err(rejection) => Ok(rejection),
        Ok(_) => Err(MetalError::Load(
            "preflight succeeded where a refusal was required".into(),
        )),
    }
}

/// Returns whether a refusal is a cross-family representation mismatch.
#[must_use]
pub fn is_unsupported_representation(rejection: &LoadRejection) -> bool {
    let LoadRejection::NoEligibleVariant { filtered, .. } = rejection else {
        return false;
    };
    filtered.iter().any(|filtered| {
        matches!(
            filtered.reason,
            VariantIneligibility::UnsupportedRepresentation { .. }
        )
    })
}

/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

#[cfg(target_os = "macos")]
struct MetalAdapter {
    profile: TargetProfileRef,
    dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    device: metal::Device,
    pipelines: Vec<metal::ComputePipelineState>,
    input: metal::Buffer,
    output: metal::Buffer,
    result_count: usize,
}

#[cfg(target_os = "macos")]
impl MetalAdapter {
    fn new(
        profile: TargetProfileRef,
        dtype_dispatch: BTreeMap<ArithmeticType, DTypeDispatch>,
    ) -> Self {
        let device = metal::Device::system_default().expect("device_available already checked");
        let elements = crate::semantic::OPERANDS.len() as u64;
        let input = device.new_buffer(
            elements * F32_BYTES,
            metal::MTLResourceOptions::StorageModeShared,
        );
        let output = device.new_buffer(
            elements * F32_BYTES,
            metal::MTLResourceOptions::StorageModeShared,
        );
        let operands: Vec<f32> = crate::semantic::OPERANDS
            .iter()
            .map(|value| f32::from_bits(*value))
            .collect();
        write_f32(&input, &operands);
        Self {
            profile,
            dtype_dispatch,
            device,
            pipelines: Vec::new(),
            input,
            output,
            result_count: crate::semantic::OPERANDS.len(),
        }
    }
}

#[cfg(target_os = "macos")]
impl RuntimeAdapter for MetalAdapter {
    type Refusal = MetalError;
    type Failure = MetalError;
    type Completion = Vec<u32>;

    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        Ok(ExecutionEnvironment {
            target_profile: self.profile.clone(),
            backend: backend(),
            representation: representation(),
            dtype_dispatch: self.dtype_dispatch.clone(),
        })
    }

    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        let library = self
            .device
            .new_library_with_data(entry.object())
            .map_err(|error| MetalError::Adapter(format!("library load: {error}")))?;
        let function = library
            .get_function(entry.entry_symbol(), None)
            .map_err(|error| MetalError::Adapter(format!("function lookup: {error}")))?;
        let descriptor = metal::ComputePipelineDescriptor::new();
        descriptor.set_compute_function(Some(&function));
        let pipeline = self
            .device
            .new_compute_pipeline_state(&descriptor)
            .map_err(|error| MetalError::Adapter(format!("pipeline: {error}")))?;
        self.pipelines.push(pipeline);
        Ok(())
    }

    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        _request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        LiveDeviceObservation::Unrecognized
    }

    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        if self.pipelines.len() != entries.len() {
            return Err(MetalError::Adapter(
                "prepared pipelines disagree with the route".into(),
            ));
        }
        Ok(())
    }

    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> PreparedEntryObservation {
        let query = request.requirement().query();
        let provider = query.provider();
        if query.key().as_str() != "tiler.target.prepared-entry.max-threads-per-workgroup.v1"
            || provider.namespace() != "tiler"
            || provider.name() != "prepared-entry-properties"
            || provider.revision() != 1
        {
            return PreparedEntryObservation::Unrecognized;
        }
        self.pipelines.get(request.entry()).map_or(
            PreparedEntryObservation::Unrecognized,
            |pipeline| {
                PreparedEntryObservation::Quantity(pipeline.max_total_threads_per_threadgroup())
            },
        )
    }

    fn plan_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        _preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal> {
        Ok(())
    }

    fn allocate_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        _routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure> {
        Ok(())
    }

    fn dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        let queue = self.device.new_command_queue();
        let command_buffer = queue.new_command_buffer();
        for (position, entry) in routed.entries().iter().enumerate() {
            let launch = entry.launch();
            if launch.grid_threads() == 0 && launch.zero_work_skips_dispatch() {
                continue;
            }
            let pipeline = &self.pipelines[position];
            let encoder = command_buffer.new_compute_command_encoder();
            encoder.set_compute_pipeline_state(pipeline);
            for binding in entry.bindings() {
                let buffer = match binding.binding().target() {
                    BindingTarget::ProgramInput(_) => &self.input,
                    BindingTarget::ProgramOutput(_) => &self.output,
                    BindingTarget::Internal => {
                        return Err(MetalError::Execute(
                            "this spike's Metal adapter places no internal storage".into(),
                        ));
                    }
                };
                encoder.set_buffer(
                    u64::from(binding.transport_slot()),
                    Some(buffer),
                    binding.accessible_offset(),
                );
            }
            let width = pipeline
                .thread_execution_width()
                .min(launch.threads_per_workgroup())
                .max(1);
            encoder.dispatch_threads(
                metal::MTLSize::new(launch.grid_threads(), 1, 1),
                metal::MTLSize::new(width, 1, 1),
            );
            encoder.end_encoding();
        }
        command_buffer.commit();
        command_buffer.wait_until_completed();
        match command_buffer.status() {
            metal::MTLCommandBufferStatus::Completed => {
                Ok(read_f32(&self.output, self.result_count)
                    .into_iter()
                    .map(f32::to_bits)
                    .collect())
            }
            metal::MTLCommandBufferStatus::Error => Err(MetalError::Execute(
                "the device reported an execution error for this command buffer".into(),
            )),
            status => Err(MetalError::Execute(format!(
                "the wait returned with command-buffer status {status:?}"
            ))),
        }
    }
}

/// Writes `values` into `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `values` requires.
#[cfg(target_os = "macos")]
#[allow(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; no Metal binding exposes it safely. The write is bounded by an asserted length check against the buffer's own byte length, copies a plain-old-data type with no destructor, and retains no borrow."
)]
fn write_f32(buffer: &metal::Buffer, values: &[f32]) {
    let required = u64::try_from(values.len()).expect("a slice length fits a u64") * F32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the write needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` returns a pointer valid for `buffer.length()` bytes
    // for as long as `buffer` is alive, and `buffer` is borrowed for this call.
    // The assertion above proves the destination spans at least `required`
    // bytes. `f32` is `Copy` with no invalid bit patterns and no destructor, so
    // a byte copy into uninitialized Metal storage is well defined. Source and
    // destination are distinct allocations, so they cannot overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(
            values.as_ptr(),
            buffer.contents().cast::<f32>(),
            values.len(),
        );
    }
}

/// Reads `count` `f32` values out of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `count` requires.
#[cfg(target_os = "macos")]
#[allow(
    unsafe_code,
    reason = "the read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`. Bounded by an asserted length check, reads a plain-old-data type, and copies out rather than retaining a borrow of device memory."
)]
fn read_f32(buffer: &metal::Buffer, count: usize) -> Vec<f32> {
    let required = u64::try_from(count).expect("an element count fits a u64") * F32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the read needs {required}",
        buffer.length(),
    );
    let mut values = vec![0.0_f32; count];
    // SAFETY: as in `write_f32`, with the direction reversed. The source spans
    // at least `required` bytes for the life of `buffer`. Destination is a
    // freshly allocated host `Vec` that cannot overlap the mapping.
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.contents().cast::<f32>(), values.as_mut_ptr(), count);
    }
    values
}
