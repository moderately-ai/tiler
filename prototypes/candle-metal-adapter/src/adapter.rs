//! The Candle Metal runtime adapter: `tiler-runtime`'s seam, consumed for real.
//!
//! # What this owns, and what it is forbidden to touch
//!
//! `docs/integration/candle.md` fixes the boundary in both directions. This
//! adapter owns Candle storage, layout, allocation, and the command stream. It
//! owns no compiler optimization and no MSL generation: it never reads the
//! expansion compiler cache, never compiles MSL at run time, and loads a
//! precompiled `metallib` from the bytes the artifact carries — through
//! `newLibraryWithData:error:`, which is delivery rather than compilation.
//!
//! It is a *runtime adapter* in the glossary's exact sense and not a backend. A
//! Metal capability it observes here is a live-device or prepared-pipeline fact
//! and never a compile guarantee.
//!
//! # Every comparison stays with the loader
//!
//! [`RuntimeAdapter`]'s division is the one this file follows without
//! exception: [`CandleMetalAdapter::observe_live_device`] reports what the device says and the
//! loader decides whether the row is met; [`CandleMetalAdapter::observe_prepared_entry`]
//! returns a measurement and the loader holds the threshold and the direction. A
//! row this adapter does not know exactly is
//! [`LiveDeviceObservation::Unrecognized`], which is fail-closed.
//!
//! # The synchronous checked boundary, and why it is not Candle's stream
//!
//! The ordinary contract has the adapter encode into Candle's active command
//! stream and neither commit nor wait. This prototype does not, and the reason
//! is a specific, checkable gap rather than convenience.
//!
//! **Fact — Candle 0.11.0 performs no post-wait terminal check.**
//! `Commands::ensure_completed` (`candle-metal-kernels/src/metal/commands.rs`)
//! inspects the command buffer's status *before* waiting: `NotEnqueued` and
//! `Enqueued` commit and wait, `Committed` and `Scheduled` wait, and both arms
//! return `Ok` without re-reading the status afterwards. A buffer that
//! transitions to `Error` during the wait therefore returns success.
//! `docs/research/runtime/candle-metal-post-wait-error-checking.md` is the
//! standing report of that gap.
//!
//! **Fact — the check is unreachable from outside Candle.** `MetalDevice`'s
//! `commands` field is `pub(crate)`, `Commands` publishes no accessor for the
//! current or in-flight `CommandBuffer`, and `MetalDevice::wait_until_completed`
//! returns `Result<()>` carrying no status. A consumer encoding into Candle's
//! stream has no object to ask.
//!
//! The contract anticipates exactly this and permits it: that method "is not
//! sufficient until the verified gap is fixed **or the adapter supplies an
//! equivalent checked boundary**". This adapter supplies one — its own command
//! queue and command buffer, committed and waited, with the terminal status read
//! after the wait and before anything reads device memory. The cost is stated
//! rather than hidden: this is the synchronous validation path, not the
//! asynchronous launch path, so it forfeits overlap with surrounding Candle
//! work. `adopt-candle-command-stream-once-a-terminal-check-is-reachable`
//! carries the activation trigger.
//!
//! # Derived requirements are discharged before anything is prepared
//!
//! Two obligations reach this adapter on the routed entry's own
//! [`ResourceRequirements`] record rather than as artifact rows, and
//! `crates/tiler-artifact/src/program/requirement.rs` is why: a backend-feature
//! row is admitted only for something "not already derivable from its verified
//! program", and a region's synchronization subject and index arithmetic are
//! both derived from the schedule. A row restating either would be a second,
//! independently editable statement about one KIR fact.
//!
//! Both are decided in [`CandleMetalAdapter::prepare_entries`], over **every**
//! entry, before the first pipeline is built. That is the earliest stage of the
//! [`RuntimeAdapter`] seam at which the whole route arrives at once —
//! [`CandleMetalAdapter::validate_payload`] is called per entry, so a check
//! written there would load entry 0's library before entry 1's subject was
//! looked at. The comparisons are `tiler_metal`'s: no Apple family table and no
//! MSL barrier spelling is written here.
//!
//! The ordering is not a convention. `check_direct_requirements` is the only
//! function that mints a `DirectRequirementsDischarged`, and
//! `CandleMetalAdapter::build_pipelines` takes one by value — so removing the
//! discharge is a compile error rather than the dead-code warning it would
//! otherwise be, which matters here because `prototypes/` is excluded from the
//! workspace style gate and no gate in this repository goes red on a warning.
//!
//! Ordering across the two streams is host-ordered rather than assumed:
//! [`CandleMetalAdapter::plan_dispatch`] brings Candle's own pending work to a terminal state
//! before anything is encoded — a tensor produced by a Candle kernel may still
//! be a promise in an uncommitted command buffer, and a separate command buffer
//! is not ordered against it — and [`CandleMetalAdapter::dispatch`] waits for its own
//! submission before returning, so Candle work that follows reads storage this
//! adapter has finished writing.

use std::sync::Arc;

use candle_core::backend::BackendStorage;
use candle_core::{DType, MetalDevice, MetalStorage, Shape};
use candle_metal_kernels::metal::{
    Buffer, CommandBuffer, ComputePipeline, Fence, Function, Library,
};
use dispatch2::DispatchData;
use objc2_metal::{
    MTLBinding, MTLBindingType, MTLCommandBufferStatus, MTLCommandQueue,
    MTLComputePipelineDescriptor, MTLDevice, MTLGPUFamily, MTLPipelineOption, MTLSize,
};

use tiler_artifact::program::{
    BindingTarget, BufferAccess, RouteRequirement, RouteResourceDimension,
};
use tiler_ir::schedule::ResourceRequirements;
use tiler_metal::applicability::{
    MetalGpuFamily, MetalGpuFamilySupport, try_observe_highest_gpu_family,
};
use tiler_metal::direct_requirement::evaluate_index_arithmetic;
use tiler_metal::synchronization_requirement::evaluate_synchronization;
use tiler_runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler_runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight, RoutedDispatch,
    RoutedEntry, TargetPropertyRequest,
};

use crate::cache::{DeviceScope, PipelineCache, PreparedPipeline};
use crate::refusal::{DispatchFailure, ReflectedBinding, ReflectedBindingClass, RouteRefusal};

/// Governed key of the Metal requirement naming a minimum Apple GPU family.
///
/// Owned by `tiler.metal`, the backend key this host states, so the loader
/// refuses a row owned by anything else before this adapter is asked.
const METAL_MINIMUM_GPU_FAMILY: &str = "tiler.metal.route-requirement.minimum-gpu-family";

/// Governed version of [`METAL_MINIMUM_GPU_FAMILY`]'s meaning.
///
/// Matched exactly. One key at two versions can mean two things, and guessing
/// which is how a route runs on a device it was refused on.
const METAL_MINIMUM_GPU_FAMILY_VERSION: u32 = 1;

/// Interface key of the program input this consumer binds.
pub const INPUT_KEY: &str = "input";
/// Interface key of the program output this consumer reads.
pub const OUTPUT_KEY: &str = "result";

/// What one completed dispatch yields to the wrapper.
///
/// Owned, per [`RuntimeAdapter::Completion`]'s contract. The storage is a real
/// Candle `MetalStorage` over the allocation the route wrote, so the wrapper
/// hands it straight back to Candle as the custom op's result.
#[derive(Debug)]
pub struct CandleCompletion {
    /// The program output, as Candle storage.
    pub storage: MetalStorage,
    /// The output's shape.
    pub shape: Shape,
    /// The governed profile key the route was carried out under.
    pub profile_key: String,
    /// How many entries were encoded, and how many the route declared skippable.
    pub encoded: usize,
    /// The route's declared entries.
    pub entries: usize,
}

/// What this device reports about itself, recorded rather than compared.
///
/// The two limits with an artifact-side counterpart — a pipeline's threadgroup
/// capacity and the per-buffer length bound — are compared in
/// [`CandleMetalAdapter::plan_dispatch`]; the rest is provenance.
#[derive(Clone, Debug)]
pub struct DeviceFacts {
    /// The name the device reports for itself.
    pub name: String,
    /// The largest single allocation this device admits.
    pub max_buffer_length: u64,
    /// The highest named Apple family this device claims.
    pub highest_apple_family: MetalGpuFamilySupport,
}

/// One routed ABI slot resolved to the storage this consumer supplies for it.
#[derive(Clone, Debug)]
struct PlacedSlot {
    /// The backend argument-table index this slot occupies.
    transport: u32,
    /// The first addressed byte within the bound allocation.
    offset: u64,
    /// The access mode the ABI declares for this slot.
    ///
    /// Read from the artifact rather than inferred from what the slot is bound
    /// to. `docs/integration/candle.md` requires exactly this — "resource access
    /// modes come from the ABI so the encoder can declare read-only,
    /// write-only, and read/write resources accurately" — and inferring it would
    /// get the *consuming* half of a shared intermediate wrong: that slot is
    /// backed by an allocation some other entry writes, and is itself a read.
    access: BufferAccess,
    /// The allocation itself, retained through the dispatch that reads it.
    buffer: Arc<Buffer>,
}

/// Which storage one routed slot will be backed by, decided before the commit.
///
/// A decision about storage rather than storage. Only the last two are
/// acquisitions, and ADR 0051 puts those after the routing commit; the first two
/// name allocations that either already exist or are made once for a pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Backing {
    /// The caller's own Candle tensor, which exists before the route does.
    CallerInput,
    /// One end of a pair the route says must share a single allocation.
    Shared(usize),
    /// The program output this consumer hands back.
    Output,
    /// Entry-internal storage this route allocates and discards.
    Internal,
}

/// One routed ABI slot sized against the route, with nothing acquired for it.
#[derive(Clone, Copy, Debug)]
struct SlotPlan {
    /// Zero-based ABI slot, carried so a failure can name it.
    slot: usize,
    /// The backend argument-table index this slot occupies.
    transport: u32,
    /// The first addressed byte within the allocation this slot will take.
    offset: u64,
    /// The access mode the ABI declares for this slot.
    access: BufferAccess,
    /// Bytes the allocation must reach for this slot.
    needed: u64,
    /// Where the bytes will come from.
    backing: Backing,
}

/// One entry sized against the route, with no program storage acquired.
#[derive(Clone, Debug)]
struct EntryPlan {
    pipeline: ComputePipeline,
    slots: Vec<SlotPlan>,
    grid_threads: u64,
    threads_per_workgroup: u64,
    skipped: bool,
}

/// One pair of slots the route requires be backed by a single allocation.
#[derive(Clone, Copy, Debug)]
struct SharedPlan {
    /// Entry and ABI slot of the producing end, which names the allocation.
    producer: (usize, usize),
    /// Bytes the one allocation must hold to satisfy both ends.
    needed: u64,
}

/// One entry of a route with every device object its dispatch needs.
#[derive(Clone, Debug)]
struct PreparedEntry {
    pipeline: ComputePipeline,
    slots: Vec<PlacedSlot>,
    grid_threads: u64,
    threads_per_workgroup: u64,
    /// This entry covers no threads and the artifact says to skip its dispatch.
    ///
    /// Its buffers are still allocated and still retained: an empty producing
    /// stage shares its intermediate with the consumer that follows, and the
    /// consumer must bind an allocation rather than nothing.
    skipped: bool,
}

/// One route this device proved it can carry out, held across the commit.
///
/// **This value is the retention criterion.** Every `Arc<Buffer>` the encoder
/// binds, every pipeline it sets, and the output the caller receives are owned
/// here until [`CandleMetalAdapter::dispatch`] has observed terminal success.
///
/// Holding the `Arc` rather than a clone of the `Buffer` is load-bearing and not
/// interchangeable. Candle's `Buffer` is a retained Objective-C handle, so
/// cloning it keeps the `MTLBuffer` alive — but Candle's *allocator* recycles a
/// pooled buffer as soon as its `Arc::strong_count` reaches one, and would hand
/// the same live `MTLBuffer` to an unrelated allocation while this route still
/// reads it. An Objective-C retain is therefore not sufficient evidence of
/// exclusive use here; the `Arc` is.
#[derive(Debug)]
struct PlannedRoute {
    entries: Vec<PreparedEntry>,
    /// The allocation the program output lands in.
    output: Arc<Buffer>,
    /// How many `f32` elements it holds.
    output_elements: usize,
}

/// The caller's Candle-owned input, resolved to a buffer and a byte offset.
#[derive(Clone, Debug)]
pub struct BoundInput {
    /// The allocation Candle's tensor is backed by.
    pub buffer: Arc<Buffer>,
    /// The first addressed byte of the logical view within it.
    ///
    /// Composed from Candle's `Layout::start_offset` — an *element* offset into
    /// the allocation — and never assumed zero. The adapter never uses the full
    /// allocation length as the logical tensor length and never binds offset
    /// zero merely because it holds the underlying buffer.
    pub byte_offset: u64,
}

/// One consumer-selected runtime adapter for `tiler.metal` / `metallib` over Candle storage.
#[derive(Debug)]
pub struct CandleMetalAdapter {
    device: MetalDevice,
    /// This adapter's own command queue; see the module note on the checked boundary.
    queue: candle_metal_kernels::metal::CommandQueue,
    /// The environment the producer declared, restated rather than re-derived.
    environment: ExecutionEnvironment,
    /// The identity of the artifact being routed, which scopes every cache entry.
    artifact_identity: Vec<u8>,
    input: BoundInput,
    output_elements: usize,
    facts: DeviceFacts,
    cache: PipelineCache,
    /// Libraries validated from their own bytes, in execution order.
    validated: Vec<Library>,
    /// Entries promoted to prepared pipelines, in execution order.
    prepared: Vec<ComputePipeline>,
    /// What the pre-commit plan decided, per entry, having acquired nothing.
    plan: Vec<EntryPlan>,
    /// The allocations the pre-commit plan sized for shared slot pairs.
    shared_plan: Vec<SharedPlan>,
    /// Everything the committed dispatch will touch.
    planned: Option<PlannedRoute>,
    /// Slot pairs the loader required be backed by one allocation each.
    ///
    /// Recorded rather than recomputed by the caller: `Preflight` is the only
    /// authority that publishes them and it is reachable from `plan_dispatch`
    /// alone, so a caller that wanted the count would have to mint a second
    /// routing authority for a route it will not execute.
    shared_allocations: usize,
}

impl CandleMetalAdapter {
    /// Builds an adapter bound to one Candle Metal device and one caller input.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRefusal::NoExecutionContext`] when the device yields no
    /// command queue.
    pub fn new(
        device: &MetalDevice,
        environment: ExecutionEnvironment,
        artifact_identity: &[u8],
        input: BoundInput,
        output_elements: usize,
    ) -> Result<Self, RouteRefusal> {
        let queue = device.metal_device().new_command_queue().map_err(|cause| {
            RouteRefusal::NoExecutionContext {
                detail: cause.to_string(),
            }
        })?;
        Ok(Self {
            facts: device_facts(device),
            cache: PipelineCache::new(device),
            device: device.clone(),
            queue,
            environment,
            artifact_identity: artifact_identity.to_vec(),
            input,
            output_elements,
            validated: Vec::new(),
            prepared: Vec::new(),
            plan: Vec::new(),
            shared_plan: Vec::new(),
            planned: None,
            shared_allocations: 0,
        })
    }

    /// Returns what the bound device reported about itself.
    pub fn facts(&self) -> &DeviceFacts {
        &self.facts
    }

    /// Returns how many slot pairs the loader required be backed by one allocation.
    pub const fn shared_allocations(&self) -> usize {
        self.shared_allocations
    }

    /// Returns how many libraries and pipelines this adapter's cache holds.
    pub fn cache_occupancy(&self) -> (usize, usize) {
        self.cache.occupancy()
    }

    /// Returns the scope every cache entry of this adapter is minted under.
    pub fn scope(&self) -> DeviceScope {
        DeviceScope::of(&self.device)
    }

    /// Loads one entry's carried object into a Metal library, through the cache.
    fn library_for(
        &mut self,
        position: usize,
        entry: &RoutedEntry<'_>,
    ) -> Result<Library, RouteRefusal> {
        let scope = DeviceScope::of(&self.device);
        let key = self
            .cache
            .library_key(&scope, &self.artifact_identity, position)?;
        if let Some(cached) = self.cache.library(&key) {
            return Ok(cached.clone());
        }
        let library = load_library(&self.device, position, entry.object())?;
        self.cache.insert_library(key, library.clone());
        Ok(library)
    }

    /// Builds one entry's compute pipeline and reads its argument table, through the cache.
    ///
    /// The pipeline is built through [`prepare_pipeline_with_reflection`] rather
    /// than Candle's `new_compute_pipeline_state_with_function`, because that
    /// wrapper discards the reflection out-param and no later call recovers it.
    /// The table it returns is cached beside the pipeline; the *comparison*
    /// against the entry's declaration is not cached, and runs in
    /// [`RuntimeAdapter::prepare_entries`] on hits and misses alike.
    fn pipeline_for(
        &mut self,
        position: usize,
        library: &Library,
        symbol: &str,
    ) -> Result<PreparedPipeline, RouteRefusal> {
        let scope = DeviceScope::of(&self.device);
        let library_key = self
            .cache
            .library_key(&scope, &self.artifact_identity, position)?;
        let key = PipelineCache::pipeline_key(&library_key, symbol);
        if let Some(cached) = self.cache.pipeline(&key) {
            return Ok(cached.clone());
        }
        // Function lookup and pipeline creation stay distinct stages, because a
        // missing declared symbol is an artifact invariant and a pipeline the
        // device refuses is a route fact about this device.
        let function = library.get_function(symbol, None).map_err(|cause| {
            RouteRefusal::EntrySymbolAbsent {
                entry: position,
                symbol: symbol.to_owned(),
                detail: cause.to_string(),
            }
        })?;
        let prepared = prepare_pipeline_with_reflection(&self.device, position, symbol, &function)?;
        self.cache.insert_pipeline(key, prepared.clone());
        Ok(prepared)
    }

    /// Builds every routed entry's pipeline and compares its argument table.
    ///
    /// The body of [`RuntimeAdapter::prepare_entries`] after the derived
    /// requirements are discharged, split out **only** so the ordering is carried
    /// by an argument type. `_discharged` is unread and cannot be otherwise: a
    /// `DirectRequirementsDischarged` holds no data, and what it states is that
    /// `discharge::check_direct_requirements` ran. Its position in this signature
    /// is the enforcement — the trait method's own signature is fixed by
    /// [`RuntimeAdapter`] and cannot carry it.
    ///
    /// # Errors
    ///
    /// Returns the pipeline-stage member of [`RouteRefusal`] the entry produced.
    fn build_pipelines(
        &mut self,
        entries: &[RoutedEntry<'_>],
        _discharged: DirectRequirementsDischarged,
    ) -> Result<(), RouteRefusal> {
        let mut prepared = Vec::with_capacity(entries.len());
        for (position, entry) in entries.iter().enumerate() {
            let library = self.validated[position].clone();
            let symbol = entry.entry_symbol();
            let built = self.pipeline_for(position, &library, symbol)?;
            argument_slots_agree(
                position,
                symbol,
                &declared_transport_slots(entry),
                &built.addressed_slots,
            )?;
            prepared.push(built.pipeline);
        }
        self.prepared = prepared;
        Ok(())
    }

    /// Allocates one buffer of at least `bytes` through Candle's own allocator.
    ///
    /// Through Candle rather than through `MTLDevice` directly, because the
    /// contract says output and temporary storage come from the input device's
    /// allocator — that is what keeps the result a tensor Candle can go on using
    /// and what keeps the allocation inside Candle's residency set.
    ///
    /// Reached only from [`RuntimeAdapter::allocate_dispatch`], which is why it
    /// returns [`DispatchFailure`]: the route has committed by then, and ADR
    /// 0051 leaves no fallback for an allocator that refuses.
    fn allocate(&self, bytes: u64) -> Result<Arc<Buffer>, DispatchFailure> {
        let element_bytes = u64::try_from(DType::F32.size_in_bytes()).unwrap_or(4);
        // Rounded up rather than truncated: a byte range that is not a whole
        // number of elements must still be entirely reachable.
        let elements = bytes.div_ceil(element_bytes).max(1);
        let elements = usize::try_from(elements).map_err(|_| DispatchFailure::Allocation {
            detail: format!("{bytes} byte(s) is not an allocation this host can address"),
        })?;
        self.device
            .new_buffer(elements, DType::F32, "tiler.route")
            .map_err(|cause| DispatchFailure::Allocation {
                detail: cause.to_string(),
            })
    }
}

/// Loads one carried object into a Metal library on a bound Candle device.
///
/// **Delivery, not compilation.** `newLibraryWithData:error:` takes an
/// already-compiled `metallib` image; no MSL is compiled at run time and no
/// expansion compiler cache is read. The bytes are the ones the envelope carries
/// and whose integrity digest the artifact layer already proved, so a refusal
/// here is content that will not execute rather than a damaged file.
///
/// A free function rather than a method, so `crate::proof`'s fail-closed probe
/// drives the exact code path a route takes instead of a second one written to
/// resemble it.
///
/// # Errors
///
/// Returns [`RouteRefusal::PayloadNotALibrary`] with Metal's own account.
pub fn load_library(
    device: &MetalDevice,
    entry: usize,
    object: &[u8],
) -> Result<Library, RouteRefusal> {
    let data = DispatchData::from_bytes(object);
    device
        .metal_device()
        .as_ref()
        .newLibraryWithData_error(&data)
        .map(Library::new)
        .map_err(|cause| RouteRefusal::PayloadNotALibrary {
            entry,
            detail: cause.to_string(),
        })
}

/// Builds one compute pipeline **and** reads the argument table it addresses.
///
/// # Why this exists rather than Candle's own constructor
///
/// **Fact — Candle 0.11.0 discards reflection.**
/// `Device::new_compute_pipeline_state_with_function`
/// (`candle-metal-kernels/src/metal/device.rs`) calls
/// `newComputePipelineStateWithFunction:error:`, the overload that takes no
/// options and no reflection out-param, and `MTLComputePipelineState` publishes
/// no argument table of its own. A consumer that builds through Candle therefore
/// has no object to ask, at any later point, what slots the compiled function
/// addresses — which is why ADR 0090 item 8's third obligation was open.
///
/// # Why a descriptor rather than the function overload
///
/// `objc2-metal` 0.3.2 declares
/// `newComputePipelineStateWithFunction:options:reflection:error:` as an
/// `unsafe fn` — it cannot check that a bare `MTLFunction` is safe to call with
/// the arguments a caller will bind — while
/// `newComputePipelineStateWithDescriptor:options:reflection:error:` is safe.
/// Both produce the same reflection for a descriptor whose only set property is
/// the compute function, so the safe one is taken and this crate keeps the
/// workspace's `unsafe_code = "forbid"`.
///
/// # Two questions from one reflection, in this order
///
/// **Every row's class must be one the artifact ABI can declare**, which today
/// means [`MTLBindingType::Buffer`] and nothing else. A texture, sampler, or
/// `[[threadgroup(N)]]` row has no declared counterpart to disagree with, so it
/// is refused as [`RouteRefusal::UndeclarableBindings`] rather than compared
/// against a transport slot. That is deliberately not a comparison against the
/// entry's declaration: the ABI's vocabulary is a property of the artifact
/// layer, not of one route, and an object that fails it is refused whatever it
/// was declared beside.
///
/// **Then the buffer rows' indices are the addressed argument table**, because a
/// transport slot *is* a `[[buffer(N)]]` index. The other namespaces number from
/// zero independently, so a threadgroup row counted as a slot would compare two
/// numberings against one — which is why they are refused above rather than
/// folded into the table here.
///
/// A row is read whether or not it reports `isUsed`. What the comparison is
/// about is the *argument table the object addresses* — a declared-but-unread
/// parameter still occupies its index, and an entry that declares it is correct
/// to bind it. Filtering by use would refuse an object the compiler merely
/// optimized well, and would let an unbindable texture through whenever the
/// compiler decided the kernel did not need it.
///
/// An object refused here never becomes a [`PreparedPipeline`], so it never
/// enters the cache and no later hit can skip the refusal.
///
/// # Errors
///
/// Returns [`RouteRefusal::PipelineRejected`] when the device refuses the
/// pipeline, [`RouteRefusal::ArgumentTableUnavailable`] when it builds one and
/// returns no reflection, and [`RouteRefusal::UndeclarableBindings`] when the
/// reflection reports a resource class the ABI cannot declare.
pub fn prepare_pipeline_with_reflection(
    device: &MetalDevice,
    entry: usize,
    symbol: &str,
    function: &Function,
) -> Result<PreparedPipeline, RouteRefusal> {
    let descriptor = MTLComputePipelineDescriptor::new();
    descriptor.setComputeFunction(Some(function.as_ref()));

    let mut reflection = None;
    let raw = device
        .metal_device()
        .as_ref()
        .newComputePipelineStateWithDescriptor_options_reflection_error(
            &descriptor,
            MTLPipelineOption::BindingInfo,
            Some(&mut reflection),
        )
        .map_err(|cause| RouteRefusal::PipelineRejected {
            entry,
            symbol: symbol.to_owned(),
            detail: cause.to_string(),
        })?;

    // Asked for and absent is a refusal, not an empty table. An empty argument
    // table is a real answer — a kernel taking no buffers — and treating a
    // missing reflection as one would silently accept every object.
    let reflection = reflection.ok_or_else(|| RouteRefusal::ArgumentTableUnavailable {
        entry,
        symbol: symbol.to_owned(),
    })?;

    let bindings = reflection.bindings();
    let mut reflected = Vec::with_capacity(bindings.count());
    for position in 0..bindings.count() {
        let binding = bindings.objectAtIndex(position);
        reflected.push(ReflectedBinding {
            class: reflected_binding_class(binding.r#type()),
            index: u64::try_from(binding.index()).unwrap_or(u64::MAX),
        });
    }
    // Before the buffer table is derived, so an object addressing an
    // undeclarable resource is refused as one rather than reaching a slot
    // comparison over the half of its table that happens to agree.
    bindings_are_declarable(entry, symbol, &reflected)?;

    let mut addressed_slots: Vec<u64> = reflected
        .iter()
        .filter(|row| row.class.is_declarable())
        .map(|row| row.index)
        .collect();
    // Sorted so the comparison is against the table's content rather than the
    // order the runtime happened to enumerate it in.
    addressed_slots.sort_unstable();

    Ok(PreparedPipeline {
        pipeline: ComputePipeline::new(raw),
        addressed_slots,
    })
}

/// Classifies one reflected binding row's resource class.
///
/// Exhaustive over every class `objc2-metal` 0.3.2 names, written out rather
/// than reduced to "is it a buffer": a reader of the refusal has to be able to
/// tell a texture from a threadgroup allocation, and a boolean cannot say which
/// one arrived.
///
/// The final arm binds the raw code instead of discarding it. `MTLBindingType`
/// is a `#[repr(transparent)]` newtype over `NSInteger` with associated
/// constants rather than a Rust enum — the same shape as
/// [`MTLCommandBufferStatus`], for the reason [`submission_outcome`] records —
/// so a class Apple adds arrives as an unrecognised number rather than as a
/// build error, and it must reach [`ReflectedBindingClass::Unnamed`], which is
/// not declarable and is therefore refused.
///
/// [`MTLCommandBufferStatus`]: objc2_metal::MTLCommandBufferStatus
pub fn reflected_binding_class(kind: MTLBindingType) -> ReflectedBindingClass {
    match kind {
        MTLBindingType::Buffer => ReflectedBindingClass::Buffer,
        MTLBindingType::ThreadgroupMemory => ReflectedBindingClass::ThreadgroupMemory,
        MTLBindingType::Texture => ReflectedBindingClass::Texture,
        MTLBindingType::Sampler => ReflectedBindingClass::Sampler,
        MTLBindingType::ImageblockData => ReflectedBindingClass::ImageblockData,
        MTLBindingType::Imageblock => ReflectedBindingClass::Imageblock,
        MTLBindingType::VisibleFunctionTable => ReflectedBindingClass::VisibleFunctionTable,
        MTLBindingType::PrimitiveAccelerationStructure => {
            ReflectedBindingClass::PrimitiveAccelerationStructure
        }
        MTLBindingType::InstanceAccelerationStructure => {
            ReflectedBindingClass::InstanceAccelerationStructure
        }
        MTLBindingType::IntersectionFunctionTable => {
            ReflectedBindingClass::IntersectionFunctionTable
        }
        MTLBindingType::ObjectPayload => ReflectedBindingClass::ObjectPayload,
        MTLBindingType::Tensor => ReflectedBindingClass::Tensor,
        MTLBindingType(code) => ReflectedBindingClass::Unnamed(code),
    }
}

/// The one step that may mint evidence of a discharged derived requirement.
///
/// **A module, and its only reason is that a witness declared beside its
/// consumer is forgeable.** A unit struct with a private field can still be
/// written by any line of the module that declares it, so a witness sitting in
/// `crate::adapter` beside [`CandleMetalAdapter::prepare_entries`] could be
/// constructed there with the check deleted — the same silent hole a bare
/// convention leaves. [`check_direct_requirements`] is the only function inside
/// this module, so the only way any caller obtains a
/// [`DirectRequirementsDischarged`] is to have run it.
///
/// **One function rather than one per requirement.** Two minting functions would
/// let a caller discharge the cheaper one and hold evidence whose name claims
/// both, which is what would go wrong as the derived record grows.
mod discharge {
    use tiler_ir::schedule::ResourceRequirements;
    use tiler_metal::applicability::MetalGpuFamilySupport;
    use tiler_runtime::load::RoutedEntry;

    use super::derived_requirements_hold;
    use crate::refusal::RouteRefusal;

    /// Evidence that every routed entry's derived requirements were checked
    /// against this backend and this device before anything was prepared.
    ///
    /// Carries no data. What it carries is the ordering:
    /// `super::CandleMetalAdapter::build_pipelines` takes one by value, so
    /// removing the check is a *compile* error at the call site rather than a
    /// dead-code warning — and `prototypes/` is excluded from the workspace
    /// style gate by design, so a warning is not something any gate in this
    /// repository would have gone red on.
    #[must_use]
    pub(super) struct DirectRequirementsDischarged(());

    /// Checks every routed entry's derived requirements before anything is prepared.
    ///
    /// The record is decoded once per entry into an owned slice rather than read
    /// twice from the view: the decision makes two passes over the population —
    /// see [`derived_requirements_hold`] for why the passes are separate — and
    /// `DecodedEntry::resources` decodes on each call.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRefusal::SynchronizationUnrealizable`] or
    /// [`RouteRefusal::IndexArithmeticUnsupported`] naming the first entry this
    /// backend or this device cannot carry, and the typed cause from the owning
    /// comparison.
    pub(super) fn check_direct_requirements(
        observed: MetalGpuFamilySupport,
        entries: &[RoutedEntry<'_>],
    ) -> Result<DirectRequirementsDischarged, RouteRefusal> {
        let required: Vec<ResourceRequirements> = entries
            .iter()
            .map(|entry| entry.entry().resources())
            .collect();
        derived_requirements_hold(&required, observed)?;
        Ok(DirectRequirementsDischarged(()))
    }
}

use discharge::{DirectRequirementsDischarged, check_direct_requirements};

/// Whether this backend and this device carry every entry's derived requirements.
///
/// Split from the route for the same reason [`binding_fits`] is split from the
/// device: the walk contributes the population and this contributes the
/// decision, so the repository gate can watch every refusal fail without a
/// routed artifact and without hardware. Neither comparison is made here —
/// `tiler_metal` owns both, because which Apple family carries an arithmetic and
/// which barrier realizes a synchronization subject are backend vocabulary.
///
/// # The two passes are separate, and the order says why
///
/// Synchronization is checked for **every** entry before index arithmetic is
/// checked for **any**. The subject needs no device at all — Metal's barrier
/// builtins and their coupled visibility are fixed by the language — so a route
/// requiring a realization this backend has no construct for is refused before
/// the Apple-family observation is consulted. Reporting the device-dependent
/// refusal first would send a reader to change hardware for a program no Metal
/// device could run.
///
/// Every entry rather than the first, on both passes, for the reason
/// [`CandleMetalAdapter::prepare_entries`] builds every pipeline: a two-entry
/// route whose *second* entry needs something this host lacks must be refused
/// here rather than discovered between two dispatches.
///
/// # Why the observation is not an [`Option`]
///
/// [`MetalIndexArithmeticRefusal::Unobserved`] exists for an adapter that never
/// asked, and `prototypes/serial-sum-run` can reach it because the `metal` 0.33
/// binding cannot name every enumerator the governed vocabulary lists. This
/// binding can: [`observed_apple_family`] passes each constant straight through
/// `MTLGPUFamily`, which `objc2-metal` models as a newtype over `NSInteger`, so
/// every family is asked about and there is no unasked case to represent. Taking
/// [`MetalGpuFamilySupport`] rather than an `Option` states that at the
/// signature instead of leaving a `Some` at the call site to be read as a
/// choice.
///
/// # Errors
///
/// Returns [`RouteRefusal::SynchronizationUnrealizable`] or
/// [`RouteRefusal::IndexArithmeticUnsupported`] naming the first offending entry.
///
/// [`MetalIndexArithmeticRefusal::Unobserved`]: tiler_metal::direct_requirement::MetalIndexArithmeticRefusal::Unobserved
fn derived_requirements_hold(
    required: &[ResourceRequirements],
    observed: MetalGpuFamilySupport,
) -> Result<(), RouteRefusal> {
    for (entry, requirements) in required.iter().enumerate() {
        evaluate_synchronization(requirements.synchronization)
            .map_err(|cause| RouteRefusal::SynchronizationUnrealizable { entry, cause })?;
    }
    for (entry, requirements) in required.iter().enumerate() {
        evaluate_index_arithmetic(requirements.index_arithmetic, Some(observed))
            .map_err(|cause| RouteRefusal::IndexArithmeticUnsupported { entry, cause })?;
    }
    Ok(())
}

/// Reads what one bound Candle Metal device reports about itself.
fn device_facts(device: &MetalDevice) -> DeviceFacts {
    let raw = device.metal_device().as_ref();
    DeviceFacts {
        name: raw.name().to_string(),
        max_buffer_length: u64::try_from(raw.maxBufferLength()).unwrap_or(u64::MAX),
        highest_apple_family: observed_apple_family(device),
    }
}

/// Reads the highest named Apple GPU family this device reports supporting.
///
/// This adapter supplies the device call and nothing else. Which families are
/// asked about, in what order, and what the answer is called are all
/// `tiler-metal`'s, because they are facts about the governed vocabulary rather
/// than about this binding.
///
/// **It used to pair each family with its Apple constant here, and that was the
/// defect.** A pair table has no arm that can be missing, so a family added to
/// `MetalGpuFamily` compiled cleanly at this site, the device was never asked
/// about it, and `crate::proof` then reported a lower family — or none — as the
/// observation an applicability policy was refused against. Driving the walk
/// from `MetalGpuFamily::ALL` leaves no population here to fall behind.
///
/// `objc2-metal` models `MTLGPUFamily` as a newtype over `NSInteger`, so the
/// enumerator crosses as the raw value with no correspondence written here.
///
/// Public because `crate::proof` asks ADR 0086's separate applicability question
/// from the *same* observation this adapter routes under; two spellings of "what
/// family does this device claim" would let the two answers drift.
pub fn observed_apple_family(device: &MetalDevice) -> MetalGpuFamilySupport {
    let raw = device.metal_device().as_ref();
    try_observe_highest_gpu_family::<core::convert::Infallible>(|family| {
        Ok(raw.supportsFamily(MTLGPUFamily(family.value())))
    })
    .unwrap_or_else(|never| match never {})
}

/// Reads a canonical family payload through the governed vocabulary's own spelling.
///
/// Scanned against `MetalGpuFamily::ALL` rather than against a second table of
/// names written here: one spelling authority, so a family added to that
/// vocabulary cannot be silently unreadable at this boundary.
fn gpu_family_from_payload(payload: &[u8]) -> Option<MetalGpuFamily> {
    MetalGpuFamily::ALL
        .into_iter()
        .find(|family| family.as_str().as_bytes() == payload)
}

/// What a command buffer's status permits after the wait.
///
/// Three outcomes and deliberately no fourth. There is no retry and no fallback
/// variant, because every post-commit transition the runtime execution contract
/// names is "never".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubmissionOutcome {
    /// The one status that permits a readback.
    Completed,
    /// The device reported a terminal execution error.
    ExecutionError,
    /// The wait returned and the buffer had not reached a terminal state.
    NotTerminal(&'static str),
}

/// Classifies one command-buffer status into what it permits.
///
/// **Apple defines exactly two terminal states, `Completed` and `Error`.** A
/// check written as `status != Completed` is correct today and collapses that
/// distinction — it reports a buffer that never left the queue in the same
/// breath as one the GPU rejected, which are different things for a caller to do
/// next. This is the one place a wrong answer would be read as arithmetic: a
/// readback taken from a buffer whose dispatch failed returns whatever the
/// output held before, which compares against a reference as a numerical
/// disagreement.
///
/// **An unnamed status is non-terminal, and it has to be a default rather than a
/// build error here.** `objc2-metal` models `MTLCommandBufferStatus` as a
/// `#[repr(transparent)]` newtype over `NSUInteger` with associated constants
/// rather than as a Rust enum, so a wildcard-free exhaustive match — which
/// `prototypes/serial-sum-run` can write against `metal` 0.33's real enum — is
/// not expressible against this binding. A status Apple adds therefore arrives
/// as an unrecognised number, and the fail-closed classification is the one that
/// refuses the readback.
pub fn submission_outcome(status: MTLCommandBufferStatus) -> SubmissionOutcome {
    match status {
        MTLCommandBufferStatus::Completed => SubmissionOutcome::Completed,
        MTLCommandBufferStatus::Error => SubmissionOutcome::ExecutionError,
        MTLCommandBufferStatus::NotEnqueued => SubmissionOutcome::NotTerminal("NotEnqueued"),
        MTLCommandBufferStatus::Enqueued => SubmissionOutcome::NotTerminal("Enqueued"),
        MTLCommandBufferStatus::Committed => SubmissionOutcome::NotTerminal("Committed"),
        MTLCommandBufferStatus::Scheduled => SubmissionOutcome::NotTerminal("Scheduled"),
        _ => SubmissionOutcome::NotTerminal("an unnamed status"),
    }
}

impl RuntimeAdapter for CandleMetalAdapter {
    type Refusal = RouteRefusal;
    type Failure = DispatchFailure;
    type Completion = CandleCompletion;

    /// Reports the identities this bound context executes under.
    ///
    /// **Producer-declared equality, not host-earned eligibility.** The profile
    /// reported here is the one `tiler-build` declares for this Metal target;
    /// nothing about this host earned the right to offer it, and ADR 0086
    /// refuses that second question on every macOS row currently observable.
    /// `crate::proof` asks it separately and prints the refusal, before any
    /// routing commit.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, Self::Refusal> {
        Ok(self.environment.clone())
    }

    /// Validates one entry's carried payload from its own bytes.
    ///
    /// ADR 0090 item 8, and the artifact layer performed no part of it: the
    /// envelope proved this object's integrity digest and carried it opaquely.
    /// Only Metal can say whether the bytes decode into a library and whether it
    /// publishes the symbol the artifact declares, so both are asked here, while
    /// a refusal still costs nothing.
    ///
    /// **Where the third obligation is discharged, stated rather than implied.**
    /// That the slots the object addresses are the ones the entry declares is
    /// asked in [`Self::prepare_entries`], not here, because the argument table
    /// exists only once the pipeline does. It is still pre-commit, and that
    /// method records why the two stages divide where they do.
    fn validate_payload(
        &mut self,
        _context: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), Self::Refusal> {
        let position = self.validated.len();
        let library = self.library_for(position, entry)?;
        let symbol = entry.entry_symbol();
        library
            .get_function(symbol, None)
            .map_err(|cause| RouteRefusal::EntrySymbolAbsent {
                entry: position,
                symbol: symbol.to_owned(),
                detail: cause.to_string(),
            })?;
        // Retained rather than re-derived later. Loading twice would mean two
        // libraries for one object, and the second would be the one that ran.
        self.validated.push(library);
        Ok(())
    }

    /// Reports what the bound device is for one live-device route requirement.
    ///
    /// Reports, and does not decide. `RouteResourceDimension::SubgroupThreads`
    /// is `Unrecognized` on purpose: Metal publishes no device-scoped execution
    /// width — `threadExecutionWidth` lives on `MTLComputePipelineState`, which
    /// is a prepared-kernel fact — so answering it from a family table would
    /// report a documentation constant as a device observation.
    fn observe_live_device(
        &mut self,
        _context: &LiveExecutionContext,
        request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        match request.requirement() {
            RouteRequirement::Resource(resource) => match resource.dimension() {
                RouteResourceDimension::SubgroupThreads => LiveDeviceObservation::Unrecognized,
            },
            RouteRequirement::BackendFeature(feature) => {
                if feature.key().as_str() != METAL_MINIMUM_GPU_FAMILY
                    || feature.version() != METAL_MINIMUM_GPU_FAMILY_VERSION
                {
                    return LiveDeviceObservation::Unrecognized;
                }
                let Some(required) = gpu_family_from_payload(feature.payload()) else {
                    return LiveDeviceObservation::Unrecognized;
                };
                // Cumulative families: the highest supported one implies every
                // lower one, so the ordering decides support without a second
                // device call. A device naming none of them satisfies no family
                // requirement.
                let supported = match self.facts.highest_apple_family {
                    MetalGpuFamilySupport::Highest(highest) => highest >= required,
                    MetalGpuFamilySupport::NoneNamed => false,
                };
                LiveDeviceObservation::Feature(supported)
            }
        }
    }

    /// Discharges the derived requirements, then builds every entry's compute pipeline.
    ///
    /// Two stages, in the order their evidence exists. The requirements the
    /// verified program itself derived — the synchronization subject and the
    /// index arithmetic on each entry's `ResourceRequirements` — need no pipeline
    /// and are decided first, over every entry, by `derived_requirements_hold`'s
    /// owning comparisons in `tiler_metal`. `Self::build_pipelines` then takes
    /// the resulting witness by value, so a future edit that drops the discharge
    /// does not compile.
    ///
    /// Every entry, not the first one: a two-entry route whose *second* pipeline
    /// will not build must be refused here rather than discovered between two
    /// dispatches. Reversible — a pipeline an abandoned route never uses costs
    /// only the build.
    ///
    /// # ADR 0090 item 8's third obligation is discharged here, and why here
    ///
    /// That the slots the object addresses are the ones the entry declares is
    /// asked at this stage and not in [`Self::validate_payload`], for a reason
    /// that is structural rather than a preference: the argument table is a
    /// property of the *compiled pipeline*, Metal publishes it only as the
    /// reflection out-param of pipeline creation, and no pipeline exists until
    /// this stage. Asking it earlier would mean building every pipeline during
    /// payload validation — moving `PipelineRejected`, which is a route fact
    /// about this device, into the stage that decides artifact invariants.
    ///
    /// The timing requirement is met regardless, and by construction: this
    /// method returns [`Self::Refusal`], `route_with_adapter` runs it before
    /// `resolve_target_properties`, `plan_dispatch`, and `Preflight::commit`, and
    /// nothing has been allocated when it refuses. It is pre-commit in the seam's
    /// own terms.
    ///
    /// The comparison runs per entry and on every route, including one served
    /// entirely from the pipeline cache: the cache stores the reflected table as
    /// a fact and never the verdict, because the declared side comes from *this*
    /// attempt's routed bindings and is not part of the cache key.
    ///
    /// The obligation's other half — that the object addresses no resource class
    /// the artifact ABI can express at all — arrives from
    /// [`prepare_pipeline_with_reflection`] rather than from a second comparison
    /// here, because it has no declared side to vary per attempt: it is a
    /// property of the object, answered where the reflection is read, and an
    /// object that fails it never becomes a cache entry to be hit later.
    fn prepare_entries(
        &mut self,
        _context: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), Self::Refusal> {
        let discharged = check_direct_requirements(self.facts.highest_apple_family, entries)?;
        self.build_pipelines(entries, discharged)
    }

    /// Reports one exact prepared entry's maximum threadgroup size.
    ///
    /// From *that* entry's pipeline rather than from a device-wide property that
    /// resembles it: `MTLDevice.maxThreadsPerThreadgroup` is a device bound and
    /// `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` is this kernel's,
    /// and the second is what a launch is actually limited by.
    fn observe_prepared_entry(
        &mut self,
        _context: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> u64 {
        u64::try_from(self.prepared[request.entry()].max_total_threads_per_threadgroup())
            .unwrap_or(u64::MAX)
    }

    /// Sizes what the route will dispatch and checks its capacity, acquiring nothing.
    ///
    /// The last chance to refuse, so every obligation a device can answer
    /// *without acquiring storage* is answered here: the threadgroup capacity
    /// this route's pipelines admit, the single-buffer limit each binding must
    /// fit, whether each binding's offset and extent form an addressable range,
    /// and whether every slot addresses something this consumer places. What it
    /// produces is a plan — which storage each slot will take and how long it
    /// must be — and no `MTLBuffer`.
    ///
    /// **Allocation is not here, and that is ADR 0051.** Program storage is
    /// acquired only from the committed execution authority, in
    /// [`RuntimeAdapter::allocate_dispatch`].
    ///
    /// Candle's pending work is brought to a terminal state first. The tensor
    /// this route reads may still be a promise in Candle's uncommitted command
    /// buffer, and this adapter's own command buffer is not ordered against it;
    /// flushing here rather than in `dispatch` keeps the failure on the side of
    /// the commit where a fallback would still be permitted, and it acquires
    /// nothing of its own.
    fn plan_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), Self::Refusal> {
        self.device
            .wait_until_completed()
            .map_err(|cause| RouteRefusal::PendingCandleWork {
                detail: cause.to_string(),
            })?;

        let routed = preflight.entries();
        self.shared_allocations = preflight.shared_allocations().len();

        // Paired first, because a shared allocation belongs to two entries and
        // neither owns it. A planner that sized per binding would let the
        // consumer be handed a fresh buffer and read uninitialised device memory
        // — a wrong answer rather than a refusal.
        let mut paired: Vec<Vec<Option<usize>>> = routed
            .iter()
            .map(|entry| vec![None; entry.bindings().len()])
            .collect();
        let mut shared_plan: Vec<SharedPlan> = Vec::new();
        for pair in preflight.shared_allocations() {
            let (producer, consumer) = (pair.producer(), pair.consumer());
            let needed = reach(routed, producer.entry(), producer.slot())?.max(reach(
                routed,
                consumer.entry(),
                consumer.slot(),
            )?);
            self.fits_one_buffer(producer.entry(), producer.slot(), needed)?;
            let index = shared_plan.len();
            paired[producer.entry()][producer.slot()] = Some(index);
            paired[consumer.entry()][consumer.slot()] = Some(index);
            shared_plan.push(SharedPlan {
                producer: (producer.entry(), producer.slot()),
                needed,
            });
        }

        let mut binds_output = false;
        let mut plan = Vec::with_capacity(routed.len());
        for (position, entry) in routed.iter().enumerate() {
            let launch = entry.launch();
            if launch.grid_threads() == 0 && !launch.zero_work_skips_dispatch() {
                return Err(RouteRefusal::EmptyLaunchNotSkippable { entry: position });
            }
            let pipeline = self.prepared[position].clone();
            workgroup_fits(
                position,
                entry.entry_symbol(),
                launch.threads_per_workgroup(),
                u64::try_from(pipeline.max_total_threads_per_threadgroup()).unwrap_or(u64::MAX),
            )?;

            let mut slots = Vec::with_capacity(entry.bindings().len());
            for binding in entry.bindings() {
                let slot = binding.slot();
                let needed = reach(routed, position, slot)?;
                self.fits_one_buffer(position, slot, needed)?;

                // An occupied slot is one half of a shared pair, and naming the
                // same plan index is what will make the two entries address one
                // buffer rather than two that merely have the same length.
                let (backing, offset) = if let Some(index) = paired[position][slot] {
                    (Backing::Shared(index), binding.accessible_offset())
                } else {
                    match binding.binding().target() {
                        BindingTarget::ProgramInput(key) if key.as_str() == INPUT_KEY => {
                            // The caller's own storage, bound at the byte the
                            // Candle view starts at plus the byte the artifact
                            // says this slot addresses. Neither is assumed
                            // zero.
                            let offset = self
                                .input
                                .byte_offset
                                .checked_add(binding.accessible_offset())
                                .ok_or(RouteRefusal::BindingRangeOverflow {
                                    entry: position,
                                    slot,
                                    offset: self.input.byte_offset,
                                    extent: binding.accessible_offset(),
                                })?;
                            (Backing::CallerInput, offset)
                        }
                        BindingTarget::ProgramOutput(keys)
                            if keys.len() == 1 && keys[0].as_str() == OUTPUT_KEY =>
                        {
                            binds_output = true;
                            (Backing::Output, binding.accessible_offset())
                        }
                        BindingTarget::Internal => (Backing::Internal, binding.accessible_offset()),
                        other => {
                            return Err(RouteRefusal::UnboundBindingTarget {
                                entry: position,
                                slot,
                                target: format!("{other:?}"),
                            });
                        }
                    }
                };

                slots.push(SlotPlan {
                    slot,
                    transport: binding.transport_slot(),
                    offset,
                    access: binding.binding().access(),
                    needed,
                    backing,
                });
            }

            plan.push(EntryPlan {
                pipeline,
                slots,
                grid_threads: launch.grid_threads(),
                threads_per_workgroup: launch.threads_per_workgroup(),
                // The pipeline above was still built for a skipped entry, and
                // deliberately: a route is ready only if every object it names
                // loads, and an entry that runs no threads on this input may run
                // some on the next one.
                skipped: launch.grid_threads() == 0,
            });
        }

        // Decided from the plan's own targets rather than from whether an
        // allocation happened, which is the form this check has to take once no
        // allocation happens on this side of the commit.
        if !binds_output {
            return Err(RouteRefusal::NoOutputBinding);
        }
        self.plan = plan;
        self.shared_plan = shared_plan;
        Ok(())
    }

    /// Acquires and binds the committed route's program storage, through Candle.
    ///
    /// Reached only from a [`RoutedDispatch`], so every allocation here is made
    /// by the committed execution authority ADR 0051 reserves them to. Nothing
    /// is decided: the plan already fixed which storage each slot takes and how
    /// long it must be, and this stage acquires exactly that.
    ///
    /// The observed-length assertion is retained and is a defect report rather
    /// than a routing input — every buffer is requested at the plan's own
    /// length, so a short one means the allocator did not honour a request it
    /// accepted.
    fn allocate_dispatch(
        &mut self,
        _context: &LiveExecutionContext,
        _routed: &RoutedDispatch<'_>,
    ) -> Result<(), Self::Failure> {
        // One allocation per pair, taken first so that both ends can name it.
        let mut shared: Vec<Arc<Buffer>> = Vec::with_capacity(self.shared_plan.len());
        for pair in &self.shared_plan {
            let (entry, slot) = pair.producer;
            let buffer = self.allocate(pair.needed)?;
            Self::holds(entry, slot, pair.needed, &buffer)?;
            shared.push(buffer);
        }

        let mut output: Option<Arc<Buffer>> = None;
        let mut entries = Vec::with_capacity(self.plan.len());
        for position in 0..self.plan.len() {
            let mut slots = Vec::with_capacity(self.plan[position].slots.len());
            for index in 0..self.plan[position].slots.len() {
                // Copied out rather than borrowed, so this loop can acquire the
                // storage the plan it is reading decided on.
                let planned = self.plan[position].slots[index];
                let buffer = match planned.backing {
                    Backing::Shared(pair) => Arc::clone(&shared[pair]),
                    Backing::CallerInput => Arc::clone(&self.input.buffer),
                    Backing::Output => {
                        let buffer = self.allocate(planned.needed)?;
                        output = Some(Arc::clone(&buffer));
                        buffer
                    }
                    Backing::Internal => self.allocate(planned.needed)?,
                };
                Self::holds(position, planned.slot, planned.needed, &buffer)?;
                slots.push(PlacedSlot {
                    transport: planned.transport,
                    offset: planned.offset,
                    access: planned.access,
                    buffer,
                });
            }
            entries.push(PreparedEntry {
                pipeline: self.plan[position].pipeline.clone(),
                slots,
                grid_threads: self.plan[position].grid_threads,
                threads_per_workgroup: self.plan[position].threads_per_workgroup,
                skipped: self.plan[position].skipped,
            });
        }

        self.planned = Some(PlannedRoute {
            entries,
            // `plan_dispatch` refused a route with no output binding before the
            // commit, so the plan proves one exists.
            output: output.expect("the pre-commit plan proved one slot binds the program output"),
            output_elements: self.output_elements,
        });
        Ok(())
    }

    /// Encodes, submits, and observes the committed route to terminal success.
    ///
    /// Everything here is past the one-way commit: nothing is looked up, nothing
    /// is allocated, and there is no refusal left to make. What remains is the
    /// encode, the submission, and the terminal-status check that must precede
    /// any read of device memory.
    ///
    /// **One encoder per entry, with an explicit fence between them.** Metal
    /// orders encoders within a command buffer, but Candle allocates every
    /// buffer `HazardTrackingModeUntracked`, which means Metal does not flush
    /// GPU caches at an encoder boundary. A successor reading what a predecessor
    /// wrote therefore needs a fence rather than the ordering alone, and the
    /// fence is created and waited on rather than assumed.
    fn dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Self::Completion, Self::Failure> {
        let planned = self
            .planned
            .as_ref()
            .ok_or(DispatchFailure::EncoderUnavailable { entry: None })?;
        let raw = self
            .queue
            .commandBuffer()
            .ok_or(DispatchFailure::EncoderUnavailable { entry: None })?;
        let command_buffer = CommandBuffer::new(raw);
        command_buffer.set_label("tiler.candle.route");

        let mut previous: Option<Arc<Fence>> = None;
        let mut dispatched = 0_usize;
        for (position, entry) in planned.entries.iter().enumerate() {
            if entry.skipped {
                continue;
            }
            let fence = Arc::new(Fence::new(self.device.metal_device()));
            let encoder = command_buffer.compute_command_encoder(&fence);
            if let Some(previous) = previous.as_ref() {
                encoder.wait_for_fence(previous);
            }
            encoder.set_compute_pipeline_state(&entry.pipeline);
            for slot in &entry.slots {
                let index = usize::try_from(slot.transport).unwrap_or(usize::MAX);
                let offset = usize::try_from(slot.offset).unwrap_or(usize::MAX);
                // Declared through the setter the ABI's access mode names, not
                // through one derived from what the slot is bound to. Candle's
                // encoder uses the distinction for its own hazard tracking, and
                // a shared intermediate is written by one entry and read by the
                // next — so the placement cannot supply the answer even though
                // the allocation is the same.
                //
                // Exhaustive and wildcard-free: `BufferAccess` is a governed
                // vocabulary, and a mode added to it must stop this build rather
                // than reach an arm that guesses which setter is safe.
                match slot.access {
                    BufferAccess::Read => {
                        encoder.set_input_buffer(index, Some(&slot.buffer), offset);
                    }
                    BufferAccess::Write => {
                        encoder.set_output_buffer(index, Some(&slot.buffer), offset);
                    }
                }
            }
            encoder.dispatch_threads(
                MTLSize {
                    width: usize::try_from(entry.grid_threads).unwrap_or(usize::MAX),
                    height: 1,
                    depth: 1,
                },
                MTLSize {
                    width: usize::try_from(entry.threads_per_workgroup).unwrap_or(usize::MAX),
                    height: 1,
                    depth: 1,
                },
            );
            encoder.end_encoding();
            previous = Some(fence);
            dispatched += 1;
            debug_assert!(position < routed.entries().len());
        }

        command_buffer.commit();
        command_buffer.wait_until_completed();

        // The only decision left after the commit, and the only path to a
        // readback. Read *after* the wait, which is the check Candle's own
        // `ensure_completed` does not perform.
        match submission_outcome(command_buffer.status()) {
            SubmissionOutcome::Completed => {}
            SubmissionOutcome::ExecutionError => {
                return Err(DispatchFailure::CommandBufferError {
                    detail: command_buffer.error().map(std::borrow::Cow::into_owned),
                });
            }
            SubmissionOutcome::NotTerminal(status) => {
                return Err(DispatchFailure::NonTerminalStatus { status });
            }
        }

        Ok(CandleCompletion {
            storage: MetalStorage::new(
                Arc::clone(&planned.output),
                self.device.clone(),
                planned.output_elements,
                DType::F32,
            ),
            shape: Shape::from_dims(&[planned.output_elements]),
            profile_key: context.target_profile().key.as_str().to_owned(),
            encoded: dispatched,
            entries: routed.entries().len(),
        })
    }
}

impl CandleMetalAdapter {
    /// Refuses a binding whose accessible range exceeds one buffer on this device.
    fn fits_one_buffer(&self, entry: usize, slot: usize, needed: u64) -> Result<(), RouteRefusal> {
        binding_fits(entry, slot, needed, self.facts.max_buffer_length)
    }

    /// Reports storage that does not reach the byte range the plan sized it for.
    fn holds(
        entry: usize,
        slot: usize,
        needed: u64,
        buffer: &Buffer,
    ) -> Result<(), DispatchFailure> {
        allocation_holds(
            entry,
            slot,
            needed,
            u64::try_from(buffer.length()).unwrap_or(u64::MAX),
        )
    }
}

/// Whether one binding's accessible range fits in a single buffer here.
///
/// Split from the device call so the decision is testable without hardware: the
/// device contributes one number and this contributes the comparison. That
/// split is what lets the repository gate watch the refusal fail, which it could
/// not do for a comparison written inline against a live `MTLDevice`.
///
/// # Errors
///
/// Returns [`RouteRefusal::BindingExceedsBufferLimit`].
pub fn binding_fits(
    entry: usize,
    slot: usize,
    needed: u64,
    limit: u64,
) -> Result<(), RouteRefusal> {
    if needed > limit {
        return Err(RouteRefusal::BindingExceedsBufferLimit {
            entry,
            slot,
            needed,
            limit,
        });
    }
    Ok(())
}

/// Whether an allocation the allocator returned reaches the length it was asked for.
///
/// An assertion against Candle's own report rather than against a number this
/// adapter computed twice: every buffer is requested at the length the route
/// states, so reaching this means the allocator did not honour a request it
/// accepted, or that a caller bound a shorter tensor than the artifact declares.
///
/// Post-commit, because the allocation it inspects is. That is why it yields a
/// [`DispatchFailure`] rather than a refusal — under ADR 0051 there is no second
/// route to take once the storage has been acquired.
///
/// # Errors
///
/// Returns [`DispatchFailure::UndersizedStorage`].
pub fn allocation_holds(
    entry: usize,
    slot: usize,
    needed: u64,
    held: u64,
) -> Result<(), DispatchFailure> {
    if held < needed {
        return Err(DispatchFailure::UndersizedStorage {
            entry,
            slot,
            needed,
            held,
        });
    }
    Ok(())
}

/// Returns the backend transport slots one routed entry declares, ascending.
///
/// [`RoutedBinding::transport_slot`] rather than [`RoutedBinding::slot`], and the
/// distinction is the whole point: the ABI slot is the kernel signature's own
/// ordinal and the transport slot is the argument-table index the backend places
/// it at. Reflection reports indices, so comparing against ABI slots would
/// silently pass on any mapping that is the identity and silently fail on every
/// other one.
///
/// [`RoutedBinding::transport_slot`]: tiler_runtime::load::RoutedBinding::transport_slot
/// [`RoutedBinding::slot`]: tiler_runtime::load::RoutedBinding::slot
#[must_use]
pub fn declared_transport_slots(entry: &RoutedEntry<'_>) -> Vec<u64> {
    let mut declared: Vec<u64> = entry
        .bindings()
        .iter()
        .map(|binding| u64::from(binding.transport_slot()))
        .collect();
    declared.sort_unstable();
    declared
}

/// Whether every row a compiled object's reflection reports is one the ABI can declare.
///
/// The remainder of ADR 0090 item 8's third obligation. Split from the device
/// call for the same reason [`binding_fits`] is: the reflection contributes the
/// rows and this contributes the decision, and a decision written inline against
/// a live pipeline is one the repository gate cannot watch fail.
///
/// # Why the whole table rather than a pre-filtered list
///
/// The filter *is* the decision. Taking the complete classified table means this
/// function answers "which of these can the ABI declare" rather than trusting a
/// caller to have asked that already, and it means a test can hand it a buffer
/// row beside a texture row and watch exactly one of them be named.
///
/// # Why an empty table is admitted
///
/// A kernel taking no arguments at all is a real answer, the same way an empty
/// buffer table is in [`argument_slots_agree`]. Nothing about "no undeclarable
/// resources" needs a lower bound; the entry's own declaration is what decides
/// whether the buffer table is the right one, and that is a separate check.
///
/// # Errors
///
/// Returns [`RouteRefusal::UndeclarableBindings`] naming every offending row's
/// class and index, sorted so the message does not depend on the order the
/// runtime enumerated the reflection in.
pub fn bindings_are_declarable(
    entry: usize,
    symbol: &str,
    reflected: &[ReflectedBinding],
) -> Result<(), RouteRefusal> {
    let mut bindings: Vec<ReflectedBinding> = reflected
        .iter()
        .copied()
        .filter(|row| !row.class.is_declarable())
        .collect();
    if bindings.is_empty() {
        return Ok(());
    }
    bindings.sort_unstable();
    Err(RouteRefusal::UndeclarableBindings {
        entry,
        symbol: symbol.to_owned(),
        bindings,
    })
}

/// Whether the slots one compiled object addresses are the ones its entry declares.
///
/// ADR 0090 item 8's third obligation, and the comparison half of it: the device
/// contributes the reflected table and this contributes the decision, for the
/// same reason [`binding_fits`] is split — a comparison written inline against a
/// live pipeline is one the repository gate cannot watch fail.
///
/// # Why exact sequence equality rather than a set or a count
///
/// Both directions are defects and both must refuse. A slot the object does not
/// address is set and ignored; a slot it addresses that the entry does not
/// declare is never set, and the kernel reads an unbound argument. Neither
/// produces an error at encode time.
///
/// Duplicates are not collapsed, deliberately. Two declared bindings on one
/// transport slot is itself a defect — the second binding overwrites the first
/// and one of the two values never reaches the kernel — and a set comparison
/// would accept it, because the reflected table cannot contain a duplicate index
/// to disagree with. Comparing the sorted sequences makes that case a length
/// disagreement rather than a silent pass.
///
/// # Errors
///
/// Returns [`RouteRefusal::ArgumentSlotsDisagree`] naming both tables.
pub fn argument_slots_agree(
    entry: usize,
    symbol: &str,
    declared: &[u64],
    addressed: &[u64],
) -> Result<(), RouteRefusal> {
    if declared != addressed {
        return Err(RouteRefusal::ArgumentSlotsDisagree {
            entry,
            symbol: symbol.to_owned(),
            declared: declared.to_vec(),
            addressed: addressed.to_vec(),
        });
    }
    Ok(())
}

/// Whether a declared workgroup fits what one prepared pipeline admits.
///
/// # Errors
///
/// Returns [`RouteRefusal::WorkgroupTooLarge`].
pub fn workgroup_fits(
    entry: usize,
    symbol: &str,
    declared: u64,
    capacity: u64,
) -> Result<(), RouteRefusal> {
    if declared > capacity {
        return Err(RouteRefusal::WorkgroupTooLarge {
            entry,
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Returns the last byte one routed binding must be able to reach.
fn reach(entries: &[RoutedEntry<'_>], entry: usize, slot: usize) -> Result<u64, RouteRefusal> {
    let binding = entries[entry].bindings()[slot];
    binding
        .accessible_offset()
        .checked_add(binding.accessible_bytes())
        .ok_or(RouteRefusal::BindingRangeOverflow {
            entry,
            slot,
            offset: binding.accessible_offset(),
            extent: binding.accessible_bytes(),
        })
}

/// Reads the buffer and byte offset one Candle Metal storage and layout name.
///
/// The layout's `start_offset` is in *elements*, so it is converted through the
/// dtype's own width rather than assumed to be bytes. The allocation's length is
/// never used as the logical tensor length, and offset zero is never bound
/// merely because the underlying buffer is at hand.
///
/// # Errors
///
/// Returns [`RouteRefusal::BindingRangeOverflow`] when the element offset does
/// not convert to an addressable byte offset.
pub fn bind_candle_storage(
    storage: &MetalStorage,
    start_offset_elements: usize,
) -> Result<BoundInput, RouteRefusal> {
    let width = u64::try_from(storage.dtype().size_in_bytes()).unwrap_or(0);
    let elements = u64::try_from(start_offset_elements).unwrap_or(u64::MAX);
    let byte_offset = elements
        .checked_mul(width)
        .ok_or(RouteRefusal::BindingRangeOverflow {
            entry: 0,
            slot: 0,
            offset: elements,
            extent: width,
        })?;
    Ok(BoundInput {
        // Candle hands out `&Buffer` rather than the `Arc` its allocator holds,
        // so the handle is retained here. The caller's tensor keeps the `Arc`
        // alive for the whole call, which is what makes this retain sufficient
        // for the *input*; storage this adapter allocates itself is held as an
        // `Arc` instead, for the reason `PlannedRoute` records.
        buffer: Arc::new(storage.buffer().clone()),
        byte_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ReflectedBinding, ReflectedBindingClass, RouteRefusal, SubmissionOutcome, allocation_holds,
        argument_slots_agree, binding_fits, bindings_are_declarable, derived_requirements_hold,
        gpu_family_from_payload, reflected_binding_class, submission_outcome, workgroup_fits,
    };
    use objc2_metal::{MTLBindingType, MTLCommandBufferStatus};
    use tiler_ir::schedule::{
        ExceptionalValueAssumption, FencedSpaces, IndexArithmetic, MemoryOrdering,
        NumericalPermission, ResourceRequirements, SubnormalMode, SynchronizationKind,
        SynchronizationScope, SynchronizationSubject,
    };
    use tiler_metal::applicability::{MetalGpuFamily, MetalGpuFamilySupport};

    /// The subject every cooperative tile in this workspace derives.
    ///
    /// A workgroup control barrier fencing workgroup memory under acquire-release
    /// ordering, which `tiler_metal::synchronization_requirement` realizes.
    const STAGED: SynchronizationSubject = SynchronizationSubject {
        kind: SynchronizationKind::ControlBarrier,
        execution_scope: SynchronizationScope::Workgroup,
        visibility_scope: SynchronizationScope::Workgroup,
        fenced_spaces: FencedSpaces {
            workgroup: true,
            device: false,
        },
        ordering: MemoryOrdering::AcquireRelease,
    };

    /// The nearest neighbour of [`STAGED`] this backend cannot deliver.
    ///
    /// One dimension away: publishing device-wide rather than workgroup-wide. No
    /// in-kernel Metal barrier establishes device-wide visibility, so the
    /// spelling exists and emission declines it — which keeps the refusal about
    /// realizability rather than about an unspellable vocabulary gap.
    const DEVICE_WIDE: SynchronizationSubject = SynchronizationSubject {
        visibility_scope: SynchronizationScope::Device,
        ..STAGED
    };

    /// One entry's derived record, varying only what this decision reads.
    ///
    /// The whole [`ResourceRequirements`] rather than the two fields alone, so
    /// the population these tests drive is the record a routed entry actually
    /// carries and cannot drift from it under a rename.
    const fn requiring(
        synchronization: Option<SynchronizationSubject>,
        index_arithmetic: IndexArithmetic,
    ) -> ResourceRequirements {
        ResourceRequirements {
            buffer_bindings: 2,
            threads_per_workgroup: 1,
            local_memory_bytes: 0,
            requires_device_memory: true,
            index_arithmetic,
            synchronization,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
            permutation: NumericalPermission::Forbidden,
            signed_zero: NumericalPermission::Forbidden,
            nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
        }
    }

    /// Every routed entry's derived requirements are checked, not only the first.
    ///
    /// The admitting cases lead, because a decision that refused everything would
    /// take every route with it — including the one this adapter proves on
    /// hardware — and would pass any test built only from refusals. A region that
    /// stages nothing derives `None` and is one of them: that absence is a fact
    /// some region states, not a check to skip.
    ///
    /// The refusing case puts the unrealizable subject on the **second** entry,
    /// which is the property a first-entry-only walk would fail: such a route
    /// would reach pipeline creation and then dispatch a barrier that orders less
    /// than the schedule proved it needed.
    #[test]
    fn every_entry_s_derived_synchronization_is_checked_and_the_refusal_names_it() {
        let observed = MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9);
        assert!(
            derived_requirements_hold(
                &[
                    requiring(Some(STAGED), IndexArithmetic::CompleteU64),
                    requiring(None, IndexArithmetic::CompleteU64),
                ],
                observed,
            )
            .is_ok(),
        );

        let Err(refusal) = derived_requirements_hold(
            &[
                requiring(Some(STAGED), IndexArithmetic::CompleteU64),
                requiring(Some(DEVICE_WIDE), IndexArithmetic::CompleteU64),
            ],
            observed,
        ) else {
            panic!("no in-kernel Metal barrier publishes device-wide, on any entry of a route");
        };
        assert!(
            matches!(
                &refusal,
                RouteRefusal::SynchronizationUnrealizable { entry: 1, cause }
                    if cause.required() == DEVICE_WIDE,
            ),
            "{refusal} does not name entry 1's own required subject",
        );
    }

    /// An unrealizable subject is refused before any device observation is read.
    ///
    /// Both requirements fail here: the second entry's subject is unrealizable
    /// **and** the observation cannot decide the Apple-family floor for either
    /// entry. The synchronization refusal must still be the one reported, because
    /// no device change repairs it — reporting the device-dependent refusal first
    /// would send a reader to buy hardware for a program no Metal device runs.
    ///
    /// That is why the decision makes two passes rather than one per entry: a
    /// single interleaved pass would report entry 0's index arithmetic before it
    /// ever reached entry 1's subject.
    #[test]
    fn an_unrealizable_synchronization_outranks_an_undecidable_device_observation() {
        let Err(refusal) = derived_requirements_hold(
            &[
                requiring(Some(STAGED), IndexArithmetic::CompleteU64),
                requiring(Some(DEVICE_WIDE), IndexArithmetic::CompleteU64),
            ],
            MetalGpuFamilySupport::NoneNamed,
        ) else {
            panic!("neither requirement is satisfied here");
        };
        assert!(
            matches!(
                refusal,
                RouteRefusal::SynchronizationUnrealizable { entry: 1, .. },
            ),
            "{refusal} is the device-dependent refusal, and no device change repairs the other",
        );
    }

    /// A device that names no family refuses the index arithmetic by entry.
    ///
    /// The `Unknown` disposal rather than an unsupported device:
    /// `MetalGpuFamily` starts at `Apple5` and the sourced floor is `Apple3`, so
    /// a device naming none of them is consistent with both sides of the floor.
    ///
    /// Only entry 0 can be named. [`IndexArithmetic`] has one variant today, so
    /// two entries cannot differ in it and a second-entry case is not
    /// constructible — the per-entry walk is the same one the synchronization
    /// test above exercises, and `tiler_metal::direct_requirement`'s
    /// wildcard-free `minimum_gpu_family` is what stops a widened vocabulary from
    /// reaching this decision unclassified.
    #[test]
    fn an_undecidable_family_refuses_the_index_arithmetic_and_names_the_entry() {
        let required = [requiring(Some(STAGED), IndexArithmetic::CompleteU64); 2];
        assert!(
            derived_requirements_hold(
                &required,
                MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple5),
            )
            .is_ok(),
            "the lowest family this vocabulary reports is above the Apple3 floor",
        );

        let Err(refusal) = derived_requirements_hold(&required, MetalGpuFamilySupport::NoneNamed)
        else {
            panic!("a device naming no Apple family establishes no floor");
        };
        assert!(
            matches!(
                &refusal,
                RouteRefusal::IndexArithmeticUnsupported { entry: 0, cause }
                    if cause.rule() == "metal.index-arithmetic.undecidable-below-vocabulary",
            ),
            "{refusal} must be the Unknown disposal rather than an unsupported-device claim",
        );
    }

    /// Only `Completed` permits a readback, and `Error` is distinguished from it.
    ///
    /// The population is every status the binding names plus one it does not, so
    /// a classification that collapsed "never submitted" into "failed" — or,
    /// worse, an unnamed status into success — is visible here.
    #[test]
    fn only_a_completed_submission_permits_a_readback() {
        assert_eq!(
            submission_outcome(MTLCommandBufferStatus::Completed),
            SubmissionOutcome::Completed,
        );
        assert_eq!(
            submission_outcome(MTLCommandBufferStatus::Error),
            SubmissionOutcome::ExecutionError,
        );
        for status in [
            MTLCommandBufferStatus::NotEnqueued,
            MTLCommandBufferStatus::Enqueued,
            MTLCommandBufferStatus::Committed,
            MTLCommandBufferStatus::Scheduled,
            // A value Apple has not named. It must classify as non-terminal
            // rather than as success, and this is the only way to state that
            // against a newtype the binding does not model as an enum.
            MTLCommandBufferStatus(4242),
        ] {
            assert!(
                matches!(
                    submission_outcome(status),
                    SubmissionOutcome::NotTerminal(_)
                ),
                "{status:?} must not permit a readback",
            );
        }
    }

    /// Each pre-commit comparison accepts its boundary and refuses one past it.
    ///
    /// Both directions for each, because a comparison written the wrong way
    /// round passes any test that only exercises the failing side. The exact
    /// boundary value is stated rather than derived, so a fixture that changed
    /// shape cannot quietly stop testing the edge.
    #[test]
    fn every_pre_commit_comparison_admits_its_boundary_and_refuses_one_past_it() {
        assert!(binding_fits(0, 0, 16, 16).is_ok());
        assert!(matches!(
            binding_fits(1, 2, 17, 16),
            Err(super::RouteRefusal::BindingExceedsBufferLimit {
                entry: 1,
                slot: 2,
                needed: 17,
                limit: 16,
            }),
        ));

        // Post-commit under the split seam, and kept in this population because
        // the comparison's boundary is what is being watched, not its stage.
        assert!(allocation_holds(0, 0, 16, 16).is_ok());
        assert!(matches!(
            allocation_holds(1, 2, 16, 15),
            Err(super::DispatchFailure::UndersizedStorage {
                entry: 1,
                slot: 2,
                needed: 16,
                held: 15,
            }),
        ));

        assert!(workgroup_fits(0, "tiler_kernel", 32, 32).is_ok());
        assert!(matches!(
            workgroup_fits(1, "tiler_kernel", 33, 32),
            Err(super::RouteRefusal::WorkgroupTooLarge {
                entry: 1,
                declared: 33,
                capacity: 32,
                ..
            }),
        ));
    }

    /// An argument table agrees with its declaration, and every way it can differ refuses.
    ///
    /// The agreeing case leads, because a comparison that always refused would
    /// pass any test built only from disagreements — and it would take the whole
    /// adapter with it, since every route runs this check. The disagreeing
    /// population is written out as the four *distinct* ways two tables differ
    /// rather than as one representative: a slot the object addresses and the
    /// entry does not declare, a slot the entry declares and the object does not
    /// address, the same count at a renumbered index, and a duplicated
    /// declaration — which a set comparison would wrongly accept.
    #[test]
    fn an_argument_table_agrees_with_its_declaration_or_names_the_disagreement() {
        assert!(argument_slots_agree(0, "tiler_kernel", &[0, 1], &[0, 1]).is_ok());
        // A kernel taking no buffers is a real answer and not a missing table.
        assert!(argument_slots_agree(0, "tiler_kernel", &[], &[]).is_ok());

        for (declared, addressed) in [
            (vec![0, 1], vec![0, 1, 2]),
            (vec![0, 1, 2], vec![0, 1]),
            (vec![0, 1], vec![0, 2]),
            (vec![0, 1, 1], vec![0, 1]),
        ] {
            // `let ... else` rather than `expect_err`, whose message is a plain
            // `&str`: the tables have to be interpolated for the failure to say
            // which row of the population did not refuse.
            let Err(refusal) = argument_slots_agree(1, "tiler_kernel", &declared, &addressed)
            else {
                panic!("declared {declared:?} and addressed {addressed:?} are different tables");
            };
            assert!(
                matches!(
                    &refusal,
                    super::RouteRefusal::ArgumentSlotsDisagree {
                        entry: 1,
                        declared: named_declared,
                        addressed: named_addressed,
                        ..
                    } if *named_declared == declared && *named_addressed == addressed,
                ),
                "{refusal} does not carry both tables it compared",
            );
        }
    }

    /// Exactly one binding class is declarable, and every other one is named.
    ///
    /// The population is every constant `objc2-metal` 0.3.2 declares on
    /// `MTLBindingType`, plus a value it does not declare. Written out rather
    /// than derived, because the binding models the type as a newtype over
    /// `NSInteger` and offers no iteration over its constants — so nothing but
    /// this list would notice a class that mapped to the wrong variant, and
    /// nothing but the last row would notice one that fell through to a class
    /// this adapter treats as bindable.
    #[test]
    fn only_a_buffer_binding_is_a_class_the_abi_declares() {
        let classified = [
            (MTLBindingType::Buffer, ReflectedBindingClass::Buffer),
            (
                MTLBindingType::ThreadgroupMemory,
                ReflectedBindingClass::ThreadgroupMemory,
            ),
            (MTLBindingType::Texture, ReflectedBindingClass::Texture),
            (MTLBindingType::Sampler, ReflectedBindingClass::Sampler),
            (
                MTLBindingType::ImageblockData,
                ReflectedBindingClass::ImageblockData,
            ),
            (
                MTLBindingType::Imageblock,
                ReflectedBindingClass::Imageblock,
            ),
            (
                MTLBindingType::VisibleFunctionTable,
                ReflectedBindingClass::VisibleFunctionTable,
            ),
            (
                MTLBindingType::PrimitiveAccelerationStructure,
                ReflectedBindingClass::PrimitiveAccelerationStructure,
            ),
            (
                MTLBindingType::InstanceAccelerationStructure,
                ReflectedBindingClass::InstanceAccelerationStructure,
            ),
            (
                MTLBindingType::IntersectionFunctionTable,
                ReflectedBindingClass::IntersectionFunctionTable,
            ),
            (
                MTLBindingType::ObjectPayload,
                ReflectedBindingClass::ObjectPayload,
            ),
            (MTLBindingType::Tensor, ReflectedBindingClass::Tensor),
            // A class Apple has not named. It must classify as unnamed rather
            // than as a buffer, and carry the code so a reader can look it up.
            (MTLBindingType(4242), ReflectedBindingClass::Unnamed(4242)),
        ];
        let mut rendered: Vec<String> = Vec::new();
        for (kind, expected) in classified {
            let observed = reflected_binding_class(kind);
            assert_eq!(observed, expected, "{kind:?} is classified as {observed}");
            assert_eq!(
                observed.is_declarable(),
                expected == ReflectedBindingClass::Buffer,
                "the artifact ABI declares buffer arguments and nothing else, and {observed} \
                 disagrees",
            );
            let text = observed.to_string();
            assert!(
                !rendered.contains(&text),
                "{text:?} is not distinguishable from an earlier class",
            );
            rendered.push(text);
        }
    }

    /// A reflected table is admitted or names every row the ABI cannot declare.
    ///
    /// The admitted cases lead, because a check that refused everything would
    /// take every route with it — including the one this adapter proves on
    /// hardware. The refusing population pairs each undeclarable row with a
    /// buffer row beside it, which is the case the whole ticket is about: the
    /// buffer half agreeing is exactly why the object reached this far.
    #[test]
    fn a_reflected_table_is_declarable_or_names_every_row_that_is_not() {
        let buffer = |index| ReflectedBinding {
            class: ReflectedBindingClass::Buffer,
            index,
        };
        assert!(bindings_are_declarable(0, "tiler_kernel", &[]).is_ok());
        assert!(bindings_are_declarable(0, "tiler_kernel", &[buffer(0), buffer(1)]).is_ok());

        for class in [
            ReflectedBindingClass::Texture,
            ReflectedBindingClass::Sampler,
            ReflectedBindingClass::ThreadgroupMemory,
            ReflectedBindingClass::Unnamed(4242),
        ] {
            let offending = ReflectedBinding { class, index: 0 };
            let Err(refusal) =
                bindings_are_declarable(1, "tiler_kernel", &[buffer(0), offending, buffer(1)])
            else {
                panic!("{class} is not a class the artifact ABI can declare");
            };
            assert!(
                matches!(
                    &refusal,
                    super::RouteRefusal::UndeclarableBindings {
                        entry: 1,
                        bindings,
                        ..
                    } if bindings.as_slice() == [offending],
                ),
                "{refusal} does not name exactly the row the ABI cannot declare",
            );
            let rendered = refusal.to_string();
            assert!(
                rendered.contains(&class.to_string()) && rendered.contains("index 0"),
                "{rendered:?} must name the resource kind and the index it found",
            );
        }
    }

    /// A family payload is read through the governed vocabulary and nothing else.
    #[test]
    fn a_family_payload_is_read_or_refused_by_name() {
        assert_eq!(
            gpu_family_from_payload(MetalGpuFamily::Apple9.as_str().as_bytes()),
            Some(MetalGpuFamily::Apple9),
        );
        assert_eq!(gpu_family_from_payload(b"apple-nine"), None);
        assert_eq!(gpu_family_from_payload(b""), None);
    }
}
