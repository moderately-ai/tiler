//! This consumer's Metal device authority, written against `tiler` alone.
//!
//! Two seams, and the split is the facade's own: [`TensorAdapter`] is what any
//! region needs of a consumer's values, and [`DispatchAdapter`] is what a region
//! that *delivers* an artifact additionally needs — the dense byte run of each
//! value, and a factory for the [`RuntimeAdapter`] that will carry the route out.
//!
//! Nothing here names an internal Tiler crate. Every type below is reached
//! through `tiler::value`, `tiler::runtime`, or `tiler::artifact`, which is the
//! property the facade exists to have: an inline-frontend consumer declares one
//! dependency.
//!
//! # Every comparison stays with the loader
//!
//! [`RuntimeAdapter`]'s division is followed without exception.
//! [`MetalExecutor::observe_live_device`] reports what this device says and the
//! loader decides whether the row is met; [`MetalExecutor::observe_prepared_entry`]
//! returns a typed observation and the loader holds the threshold and the
//! direction. A row or property this adapter does not recognize exactly is
//! [`LiveDeviceObservation::Unrecognized`] or
//! [`PreparedEntryObservation::Unrecognized`], which refuses the route.

use std::cell::RefCell;
use std::rc::Rc;

use metal::{
    Buffer, CommandBufferRef, ComputePipelineDescriptor, ComputePipelineState, Device, Library,
    MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};

use tiler::artifact::program::{BindingTarget, RouteRequirement, RouteResourceDimension};
use tiler::runtime::adapter::{LiveExecutionContext, RuntimeAdapter};
use tiler::runtime::load::{
    ExecutionEnvironment, LiveDeviceObservation, LiveDeviceRequest, Preflight,
    PreparedEntryObservation, RoutedDispatch, RoutedEntry, TargetPropertyRequest,
};
use tiler::value::{
    AdapterCapability, DispatchAdapter, RegionRequest, ResultRequest, StorageScalar, TensorAdapter,
    ValueMetadata,
};

use crate::buffer;

/// A deliberate perturbation of this consumer's own behaviour.
///
/// Each value perturbs the *adapter* and never the loader or the artifact: the
/// same device, the same region, and the same operands complete moments earlier
/// in the unperturbed run, which is what makes the failure evidence about the
/// perturbation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Perturbation {
    /// The submission is halted after the routing commit.
    ///
    /// [`RuntimeAdapter::dispatch`] is reached only after `Preflight::commit`,
    /// so everything this arm does is past the one-way commit. It encodes the
    /// route exactly as the sound path does and then does not commit or wait the
    /// command buffer, which leaves it live and non-terminal — the state
    /// `submission_outcome` classifies as [`SubmissionOutcome::NotTerminal`] and
    /// the one the runtime execution contract names when it records that "a
    /// pre-wait non-error status is not evidence of successful completion".
    ///
    /// The terminal `Error` state is deliberately *not* injected, and the
    /// boundary is stated rather than left as apparent coverage: forcing a
    /// command buffer into `Error` means provoking a GPU fault, which risks a
    /// device reset and would not reproduce.
    HaltAfterCommit,
    /// The committed route's entries are encoded back to front.
    ///
    /// The one failure in this stack that fails *open*. Nothing refuses it:
    /// every payload still validates, every pipeline still builds, the plan is
    /// unchanged, both entries reach terminal success, and the answer is wrong —
    /// because a stage that reads what an earlier stage writes ran first.
    ///
    /// It perturbs the encode order alone, which is exactly the ordering
    /// guarantee [`RuntimeAdapter::dispatch`] documents: Metal orders *encoders*
    /// within one command buffer, so the order they are created in is the order
    /// the entries run in. On a one-entry route it is the identity, which is why
    /// only a route the compiler actually split can watch it fail.
    #[allow(
        dead_code,
        reason = "this module is compiled into both binaries of this crate and only the two-entry one can construct this arm; a one-entry route's reversal is the identity, so offering it there would be a check nothing could watch fail"
    )]
    ReverseEncodeOrder,
}

/// Whether a perturbation encodes a committed route's entries back to front.
///
/// Matched exhaustively rather than compared for equality, so a perturbation
/// added later is a build error here instead of silently taking the sound path.
const fn reverses_encode_order(perturbation: Option<Perturbation>) -> bool {
    match perturbation {
        Some(Perturbation::ReverseEncodeOrder) => true,
        Some(Perturbation::HaltAfterCommit) | None => false,
    }
}

/// What this consumer observed while its region was routed.
///
/// Shared through the context rather than returned by the adapter, because
/// `bind_route_and_build` yields the region's value and not the route's own
/// account — the value is what a consumer writing `let d = tiler::tensor! { … }`
/// asked for.
#[derive(Debug, Default)]
pub struct Journal {
    /// Every stage the loader drove, in order.
    pub stages: Vec<&'static str>,
    /// One line per fact worth reading beside the result.
    pub notes: Vec<String>,
    /// The governed profile key the seam published as the producer's.
    pub declared_profile: Option<String>,
    /// Slot pairs the pre-commit plan sized as one allocation shared by two entries.
    ///
    /// `None` until `plan_dispatch` runs, which is what distinguishes "the route
    /// declared none" from "the route never got that far".
    pub shared_allocations: Option<usize>,
    /// What the committed dispatch reported, when one completed.
    ///
    /// Carried structurally as well as in a note because a consumer asserting an
    /// entry *count* must read a number rather than parse a sentence.
    pub completion: Option<Completion>,
}

/// Everything a wrapped value carries: this consumer's device and its journal.
///
/// One `Rc` per value rather than one device per value: `metal::Device` is a
/// retained handle and cloning it is cheap, but the journal has to be *shared*
/// or three operands would keep three different records of one route.
#[derive(Debug)]
pub struct Session {
    device: Device,
    perturbation: Option<Perturbation>,
    journal: RefCell<Journal>,
}

impl Session {
    /// Opens a session on this host's default Metal device.
    ///
    /// Returns `None` when the host has no device, which is a refusal to report
    /// rather than a panic: a run on a machine with no GPU should say so.
    #[must_use]
    pub fn open(perturbation: Option<Perturbation>) -> Option<Self> {
        Some(Self {
            device: Device::system_default()?,
            perturbation,
            journal: RefCell::new(Journal::default()),
        })
    }

    /// Borrows the device this session bound.
    #[must_use]
    pub const fn device(&self) -> &Device {
        &self.device
    }

    /// Borrows the journal for reading.
    #[must_use]
    pub fn journal(&self) -> std::cell::Ref<'_, Journal> {
        self.journal.borrow()
    }

    /// Records one note.
    fn note(&self, note: String) {
        self.journal.borrow_mut().notes.push(note);
    }

    /// Records one stage the loader drove.
    fn stage(&self, stage: &'static str) {
        self.journal.borrow_mut().stages.push(stage);
    }
}

/// The context every wrapped value carries.
pub type Context = Rc<Session>;

/// This consumer's own tensor value. Tiler never learns what is in it.
///
/// Host storage, densely packed, innermost axis fastest — which is exactly what
/// [`AdapterCapability::DenseRowMajorStorage`] claims and what
/// [`DispatchAdapter::storage`] hands over. The device buffers this spike
/// allocates are the *adapter's* and never the consumer's: a value the caller
/// receives holds bytes, not a `MTLBuffer`, so the oracle below compares the
/// bytes a kernel wrote rather than a handle.
#[derive(Clone, Debug, PartialEq)]
pub struct HostTensor {
    scalar: StorageScalar,
    extents: Vec<u64>,
    bytes: Vec<u8>,
}

impl HostTensor {
    /// Builds one dense `f32` vector.
    #[must_use]
    #[allow(
        dead_code,
        reason = "this module is compiled into both binaries of this crate and each region uses the constructor its own declared interface needs: the pointwise region hands over rank-1 operands and the reduction region a rank-2 one, so exactly one of the two is dead in each"
    )]
    pub fn f32s(values: &[f32]) -> Self {
        Self::f32_dense(&[values.len() as u64], values)
    }

    /// Builds one dense `f32` value of the stated extents, innermost axis fastest.
    ///
    /// A region declaring named axes — `f32[rows: 1, cols: 4]` — hands over a
    /// rank-2 operand, and [`HostTensor::f32s`]'s rank-1 shape would be refused
    /// against that declared interface before any route existed. The layout is
    /// the one [`AdapterCapability::DenseRowMajorStorage`] claims, so the extents
    /// are metadata over the same byte run rather than a second representation.
    ///
    /// # Panics
    ///
    /// Panics when the extents do not describe exactly `values.len()` elements.
    /// A consumer that miscounted its own value would otherwise hand the seam a
    /// length that disagrees with the metadata it published for it.
    #[must_use]
    pub fn f32_dense(extents: &[u64], values: &[f32]) -> Self {
        let elements: u64 = extents.iter().product();
        assert_eq!(
            elements,
            values.len() as u64,
            "extents {extents:?} describe {elements} element(s) and {} were supplied",
            values.len(),
        );
        let mut bytes = Vec::with_capacity(values.len() * 4);
        for value in values {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
        Self {
            scalar: StorageScalar::F32,
            extents: extents.to_vec(),
            bytes,
        }
    }

    /// Returns the stored bytes, which is what the oracle compares.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the extents, outermost first.
    #[must_use]
    pub fn extents(&self) -> &[u64] {
        &self.extents
    }

    /// Returns the scalar this value's storage holds.
    #[must_use]
    pub const fn scalar(&self) -> StorageScalar {
        self.scalar
    }

    /// Reads the value back out as `f32`s, for reporting only.
    #[must_use]
    pub fn read(&self) -> Vec<f32> {
        read_f32s(&self.bytes)
    }
}

/// This consumer's own error, carried by Tiler and never replaced.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    /// A result of this many bytes could not be built.
    Allocation(u64),
}

impl core::fmt::Display for HostError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Allocation(bytes) => {
                write!(
                    formatter,
                    "inline-dispatch.host: {bytes} byte(s) is not a result this consumer can build"
                )
            }
        }
    }
}

impl std::error::Error for HostError {}

/// Why this consumer refused a route **before** it committed.
///
/// Every variant here arrives while a fallback is still permitted, which is the
/// half of ADR 0051's split [`RuntimeAdapter::Refusal`] names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteRefusal {
    /// The carried bytes are not a Metal library.
    PayloadNotALibrary {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Metal's own account.
        detail: String,
    },
    /// The library does not publish the symbol the artifact declares.
    EntrySymbolAbsent {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The symbol the artifact declares.
        symbol: String,
        /// Metal's own account.
        detail: String,
    },
    /// The device refused to build a pipeline for the entry.
    PipelineRejected {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The symbol the pipeline was built for.
        symbol: String,
        /// Metal's own account.
        detail: String,
    },
    /// The declared workgroup is larger than the prepared pipeline admits.
    WorkgroupTooLarge {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Threads per workgroup the route declares.
        declared: u64,
        /// Threads per workgroup the pipeline admits.
        capacity: u64,
    },
    /// One binding's accessible range exceeds a single buffer on this device.
    BindingExceedsBufferLimit {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// Bytes the route requires be reachable.
        needed: u64,
        /// The largest single allocation this device admits.
        limit: u64,
    },
    /// A binding's offset plus extent does not fit an addressable range.
    BindingRangeOverflow {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
    },
    /// A binding names a program input this region did not supply.
    UnsuppliedProgramInput {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The interface key the binding names.
        key: String,
    },
    /// A binding names a target this consumer does not place.
    UnboundBindingTarget {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// The target, rendered.
        target: String,
    },
    /// The route declares an empty launch its own artifact does not permit skipping.
    EmptyLaunchNotSkippable {
        /// Position of the entry in the route's execution order.
        entry: usize,
    },
    /// No binding of the route addresses the region's result.
    NoOutputBinding,
}

impl core::fmt::Display for RouteRefusal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::PayloadNotALibrary { entry, detail } => write!(
                formatter,
                "metal.payload: entry {entry}'s carried object is not a library: {detail}"
            ),
            Self::EntrySymbolAbsent {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "metal.payload: entry {entry}'s library does not publish `{symbol}`: {detail}"
            ),
            Self::PipelineRejected {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "metal.prepare: this device refused a pipeline for entry {entry}'s `{symbol}`: \
                 {detail}"
            ),
            Self::WorkgroupTooLarge {
                entry,
                declared,
                capacity,
            } => write!(
                formatter,
                "metal.plan: entry {entry} declares {declared} thread(s) per workgroup and its \
                 pipeline admits {capacity}"
            ),
            Self::BindingExceedsBufferLimit {
                entry,
                slot,
                needed,
                limit,
            } => write!(
                formatter,
                "metal.plan: entry {entry} slot {slot} needs {needed} byte(s) and this device \
                 admits {limit} in one buffer"
            ),
            Self::BindingRangeOverflow { entry, slot } => write!(
                formatter,
                "metal.plan: entry {entry} slot {slot}'s offset and extent do not form an \
                 addressable range"
            ),
            Self::UnsuppliedProgramInput { entry, key } => write!(
                formatter,
                "metal.plan: entry {entry} binds program input `{key}`, which this region did not \
                 supply"
            ),
            Self::UnboundBindingTarget {
                entry,
                slot,
                target,
            } => write!(
                formatter,
                "metal.plan: entry {entry} slot {slot} addresses {target}, which this consumer \
                 does not place"
            ),
            Self::EmptyLaunchNotSkippable { entry } => write!(
                formatter,
                "metal.plan: entry {entry} launches no threads and its artifact does not permit \
                 skipping the dispatch"
            ),
            Self::NoOutputBinding => {
                formatter.write_str("metal.plan: no binding of this route addresses the result")
            }
        }
    }
}

impl std::error::Error for RouteRefusal {}

/// Why a **committed** dispatch did not complete.
///
/// Separate from [`RouteRefusal`] because ADR 0051 draws the line between them,
/// and reported rather than retried: everything here is past the one-way commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchFailure {
    /// An allocation came back shorter than the plan sized it for.
    ///
    /// Post-commit because the allocation is: ADR 0051 reserves program storage
    /// to the committed execution authority. A defect report rather than a
    /// routing input — every buffer is requested at the length the pre-commit
    /// plan derived from the route, so a short one means the allocator did not
    /// honour a request it accepted.
    UndersizedStorage {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// Bytes the plan sized the allocation for.
        needed: u64,
        /// Bytes the allocation reported.
        held: u64,
    },
    /// The device reported a terminal execution error for the command buffer.
    ///
    /// `metal` 0.33.0's `CommandBufferRef` publishes no accessor for the
    /// buffer's `NSError`, so the status is named and no claim is made about
    /// *why* the device rejected it. Reading the error would need a second
    /// `unsafe` `msg_send!`, which is a decision under ADR 0079 rather than a
    /// convenience this spike may take.
    ExecutionError,
    /// The command buffer had not reached a terminal state.
    NonTerminalStatus {
        /// The exact status it stopped in.
        status: &'static str,
    },
}

impl core::fmt::Display for DispatchFailure {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UndersizedStorage {
                entry,
                slot,
                needed,
                held,
            } => write!(
                formatter,
                "metal.allocate: entry {entry} slot {slot} was sized for {needed} byte(s) and the \
                 allocation holds {held}, after the route committed"
            ),
            Self::ExecutionError => formatter.write_str(
                "metal.dispatch: the device reported a terminal execution error for this command \
                 buffer",
            ),
            Self::NonTerminalStatus { status } => write!(
                formatter,
                "metal.dispatch: the command buffer is {status}, which is not a terminal state, \
                 so nothing was read back"
            ),
        }
    }
}

impl std::error::Error for DispatchFailure {}

/// A type, not a registration: nothing global learns this adapter exists.
///
/// It travels in `Tensor<Metal>`'s type parameter, which is where the answer to
/// "which adapter reads this value" is fixed and checked by the compiler.
#[derive(Debug)]
pub struct Metal;

impl TensorAdapter for Metal {
    type Value = HostTensor;
    type Context = Context;
    type Error = HostError;

    fn supports(capability: AdapterCapability) -> bool {
        match capability {
            AdapterCapability::DenseRowMajorStorage | AdapterCapability::ResultConstruction => true,
        }
    }

    fn metadata(value: &HostTensor) -> Result<ValueMetadata, HostError> {
        Ok(ValueMetadata::new(
            value.scalar,
            value.extents.iter().copied(),
        ))
    }

    fn build(_: &Context, request: &ResultRequest<'_>) -> Result<HostTensor, HostError> {
        let elements: u64 = request.extents().iter().product();
        let bytes = elements
            .checked_mul(4)
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or(HostError::Allocation(elements))?;
        Ok(HostTensor {
            scalar: request.storage_scalar(),
            extents: request.extents().to_vec(),
            bytes: vec![0; bytes],
        })
    }
}

impl DispatchAdapter for Metal {
    type Refusal = RouteRefusal;
    type Failure = DispatchFailure;
    type Dispatch<'region> = MetalExecutor<'region>;

    fn storage(value: &HostTensor) -> Result<&[u8], HostError> {
        Ok(&value.bytes)
    }

    fn storage_mut(value: &mut HostTensor) -> Result<&mut [u8], HostError> {
        Ok(&mut value.bytes)
    }

    fn dispatcher<'region>(
        context: &Context,
        request: RegionRequest<'region>,
    ) -> Result<MetalExecutor<'region>, HostError> {
        // Recorded here rather than inside a stage, because this is the moment
        // the seam hands over — a record taken later could not distinguish
        // "nothing was handed over" from "nothing got that far".
        for operand in request.operands() {
            context.note(format!(
                "handover: {} = {:?}",
                operand.key(),
                read_f32s(operand.bytes()),
            ));
        }
        context.note(format!(
            "handover: {} = {} byte(s) to write",
            request.result_key(),
            request.result_len(),
        ));
        context.journal.borrow_mut().declared_profile = Some(
            request
                .declared_environment()
                .target_profile
                .key
                .as_str()
                .to_owned(),
        );
        Ok(MetalExecutor {
            session: Rc::clone(context),
            request,
            validated: Vec::new(),
            prepared: Vec::new(),
            plan: Vec::new(),
            shared_plan: Vec::new(),
            planned: Vec::new(),
            output: None,
        })
    }
}

/// Renders a dense `f32` byte run.
///
/// A trailing partial element is dropped rather than guessed at; every run this
/// spike renders is a whole number of elements, because
/// `BindError::StorageLengthMismatch` refused any that was not before the bytes
/// reached this consumer.
fn read_f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_ne_bytes(*chunk))
        .collect()
}

/// One routed ABI slot resolved to the device storage bound for it.
#[derive(Debug)]
struct PlacedSlot {
    /// The argument-table index this slot occupies.
    transport: u32,
    /// The first addressed byte within the bound allocation.
    offset: u64,
    /// The allocation itself, retained through the dispatch that reads it.
    buffer: Buffer,
}

/// One entry of a route with every device object its dispatch needs.
#[derive(Debug)]
struct PlannedEntry {
    pipeline: ComputePipelineState,
    slots: Vec<PlacedSlot>,
    grid_threads: u64,
    threads_per_workgroup: u64,
    /// This entry covers no threads and the artifact says to skip its dispatch.
    skipped: bool,
}

/// Which storage one routed slot will be backed by, decided before the commit.
///
/// A decision about storage rather than storage. Naming the four cases here is
/// what lets the sizing stage refuse a target this consumer does not place
/// without acquiring an `MTLBuffer` to find out.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Backing {
    /// The caller's own operand, named by the interface key it was handed under.
    ProgramInput(String),
    /// One end of a pair the route says must share a single allocation.
    Shared(usize),
    /// The region's result, which is the one allocation the caller reads back.
    Output,
    /// Entry-internal storage this route allocates and discards.
    Internal,
}

/// One routed ABI slot sized against the route, with nothing acquired for it.
#[derive(Clone, Debug)]
struct SlotPlan {
    /// Zero-based ABI slot, carried so a failure can name it.
    slot: usize,
    /// The argument-table index this slot occupies.
    transport: u32,
    /// The first addressed byte within the allocation it will take.
    offset: u64,
    /// Bytes the allocation must reach for this slot.
    needed: u64,
    /// Where the bytes will come from.
    backing: Backing,
}

/// One entry sized against the route, with no program storage acquired.
#[derive(Debug)]
struct EntryPlan {
    pipeline: ComputePipelineState,
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

/// Governed prepared-entry key this adapter answers from a compiled pipeline.
const METAL_PREPARED_WORKGROUP_KEY: &str =
    "tiler.target.prepared-entry.max-threads-per-workgroup.v1";
const METAL_PREPARED_PROVIDER_NAMESPACE: &str = "tiler";
const METAL_PREPARED_PROVIDER_NAME: &str = "prepared-entry-properties";
const METAL_PREPARED_PROVIDER_REVISION: u32 = 1;

fn observe_metal_prepared_entry(
    request: TargetPropertyRequest<'_>,
    quantity: u64,
) -> PreparedEntryObservation {
    let query = request.requirement().query();
    let provider = query.provider();
    if query.key().as_str() == METAL_PREPARED_WORKGROUP_KEY
        && provider.namespace() == METAL_PREPARED_PROVIDER_NAMESPACE
        && provider.name() == METAL_PREPARED_PROVIDER_NAME
        && provider.revision() == METAL_PREPARED_PROVIDER_REVISION
    {
        PreparedEntryObservation::Quantity(quantity)
    } else {
        PreparedEntryObservation::Unrecognized
    }
}

/// This consumer's device authority for one region invocation.
///
/// It holds the region's storage by borrow for exactly the route's duration,
/// which is why [`DispatchAdapter::dispatcher`] builds one per invocation rather
/// than lending a stored adapter out.
#[derive(Debug)]
pub struct MetalExecutor<'region> {
    session: Context,
    request: RegionRequest<'region>,
    /// Libraries validated from their own bytes, in execution order.
    validated: Vec<Library>,
    /// Entries promoted to prepared pipelines, in execution order.
    prepared: Vec<ComputePipelineState>,
    /// What the pre-commit plan decided, per entry, having acquired nothing.
    plan: Vec<EntryPlan>,
    /// The allocations the pre-commit plan sized for shared slot pairs.
    shared_plan: Vec<SharedPlan>,
    /// Everything the committed dispatch will touch.
    planned: Vec<PlannedEntry>,
    /// The allocation the region's result lands in.
    output: Option<Buffer>,
}

impl RuntimeAdapter for MetalExecutor<'_> {
    type Refusal = RouteRefusal;
    type Failure = DispatchFailure;
    type Completion = Completion;

    /// Reports the profile, backend, and representation the **producer**
    /// declared.
    ///
    /// This is the labelled diagnostic made structural. Under ADR 0086 no host
    /// earns this tuple, so stating it is a decision to route on
    /// producer-declared equality; an adapter that had *observed* an eligible
    /// device would report what it observed instead, and none can.
    fn bind_execution_context(&mut self) -> Result<ExecutionEnvironment, RouteRefusal> {
        self.session.stage("bind");
        Ok(self.request.declared_environment().clone())
    }

    /// Validates one entry's carried payload from its own bytes.
    ///
    /// ADR 0090 item 8, and the artifact layer performed no part of it: the
    /// envelope proved this object's integrity digest and carried it opaquely.
    /// Only Metal can say whether the bytes decode into a library and whether it
    /// publishes the symbol the artifact declares, so both are asked here, while
    /// a refusal still costs nothing.
    fn validate_payload(
        &mut self,
        _: &LiveExecutionContext,
        entry: &RoutedEntry<'_>,
    ) -> Result<(), RouteRefusal> {
        self.session.stage("validate-payload");
        let position = self.validated.len();
        self.session.note(format!(
            "entry {position}: symbol {:?}, {} object byte(s), {} binding(s), launch {}×{}",
            entry.entry_symbol(),
            entry.object().len(),
            entry.bindings().len(),
            entry.launch().grid_threads(),
            entry.launch().threads_per_workgroup(),
        ));
        let library = self
            .session
            .device
            .new_library_with_data(entry.object())
            .map_err(|detail| RouteRefusal::PayloadNotALibrary {
                entry: position,
                detail,
            })?;
        let symbol = entry.entry_symbol();
        library
            .get_function(symbol, None)
            .map_err(|detail| RouteRefusal::EntrySymbolAbsent {
                entry: position,
                symbol: symbol.to_owned(),
                detail,
            })?;
        // Retained rather than re-derived later. Loading twice would mean two
        // libraries for one object, and the second would be the one that ran.
        self.validated.push(library);
        Ok(())
    }

    /// Reports what the bound device is for one live-device route requirement.
    ///
    /// Reports, and does not decide — and this consumer decides nothing, because
    /// it owns no row. Both arms answer
    /// [`LiveDeviceObservation::Unrecognized`], which is the fail-closed answer:
    /// the loader refuses the route and the region takes its declared result.
    ///
    /// **This is an unsupported case, not an unreached one, and the difference
    /// matters.** The region this spike delivers declares no live-device
    /// requirement at all — `observe-live-device` is absent from the recorded
    /// stage list, which is what a route with zero rows looks like — so nothing
    /// here ran on the transcript in the README. Writing a decision for a row
    /// this run never exercises would put unwatched code in the position of
    /// evidence.
    ///
    /// Each arm is unowned for its own reason, and they are different reasons.
    /// [`RouteResourceDimension::SubgroupThreads`] has no device-scoped answer on
    /// Metal at all: `threadExecutionWidth` lives on `MTLComputePipelineState`,
    /// which is a prepared-kernel fact, so answering it from a family table would
    /// report a documentation constant as a device observation. A backend feature
    /// row — `tiler.metal.route-requirement.minimum-gpu-family` is the one this
    /// backend defines — *is* answerable from `MTLDevice::supportsFamily`, but
    /// its payload vocabulary is `tiler_metal::applicability::MetalGpuFamily`,
    /// and a consumer may not name an internal crate. Spelling the family names
    /// again here would mint a second authority over a governed vocabulary, so
    /// this spike declines and refuses instead. A consumer that needs the row
    /// needs that vocabulary re-exported, which is a public-boundary question
    /// rather than something to work around locally.
    fn observe_live_device(
        &mut self,
        _: &LiveExecutionContext,
        request: LiveDeviceRequest<'_>,
    ) -> LiveDeviceObservation {
        self.session.stage("observe-live-device");
        // Exhaustive on both the kind and the dimension: a row this adapter has
        // never seen must stop this build rather than reach an arm that guesses.
        let unowned = match request.requirement() {
            RouteRequirement::Resource(resource) => match resource.dimension() {
                RouteResourceDimension::SubgroupThreads => {
                    "a subgroup-threads row, which Metal publishes no device-scoped answer for"
                        .to_owned()
                }
            },
            RouteRequirement::BackendFeature(feature) => format!(
                "backend feature `{}` v{}, whose payload vocabulary lives in an internal crate a \
                 consumer may not name",
                feature.key().as_str(),
                feature.version(),
            ),
        };
        self.session
            .note(format!("live-device row: {unowned}; reported Unrecognized"));
        LiveDeviceObservation::Unrecognized
    }

    /// Builds every entry's compute pipeline, before any deferred property is answered.
    ///
    /// Every entry, not the first one: a two-entry route whose *second* pipeline
    /// will not build must be refused here rather than discovered between two
    /// dispatches. Reversible — a pipeline an abandoned route never uses costs
    /// only the build.
    fn prepare_entries(
        &mut self,
        _: &LiveExecutionContext,
        entries: &[RoutedEntry<'_>],
    ) -> Result<(), RouteRefusal> {
        self.session.stage("prepare-entries");
        let mut prepared = Vec::with_capacity(entries.len());
        for (position, entry) in entries.iter().enumerate() {
            let symbol = entry.entry_symbol();
            let function = self.validated[position]
                .get_function(symbol, None)
                .map_err(|detail| RouteRefusal::EntrySymbolAbsent {
                    entry: position,
                    symbol: symbol.to_owned(),
                    detail,
                })?;
            let descriptor = ComputePipelineDescriptor::new();
            descriptor.set_compute_function(Some(&function));
            let pipeline = self
                .session
                .device
                .new_compute_pipeline_state(&descriptor)
                .map_err(|detail| RouteRefusal::PipelineRejected {
                    entry: position,
                    symbol: symbol.to_owned(),
                    detail,
                })?;
            prepared.push(pipeline);
        }
        self.prepared = prepared;
        Ok(())
    }

    /// Reports one exact prepared entry's maximum threadgroup size.
    ///
    /// From *that* entry's pipeline rather than from a device-wide property that
    /// resembles it: `MTLDevice.maxThreadsPerThreadgroup` is a device bound and
    /// `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` is this kernel's,
    /// and the second is what a launch is actually limited by.
    fn observe_prepared_entry(
        &mut self,
        _: &LiveExecutionContext,
        request: TargetPropertyRequest<'_>,
    ) -> PreparedEntryObservation {
        self.session.stage("observe-prepared-entry");
        observe_metal_prepared_entry(
            request,
            self.prepared[request.entry()].max_total_threads_per_threadgroup(),
        )
    }

    /// Sizes what the route will dispatch and checks its capacity, acquiring nothing.
    ///
    /// The last chance to refuse, so every obligation a device can answer
    /// *without acquiring storage* is answered here: the threadgroup capacity
    /// this route's pipelines admit, the single-buffer limit each binding must
    /// fit, whether each binding's offset and extent form an addressable range,
    /// and whether every slot names an operand this region supplied or a target
    /// this consumer places. What it produces is a plan and no `MTLBuffer`.
    ///
    /// **Allocation moved out of this stage under ADR 0051**, which reserves
    /// program storage to the committed execution authority; it lives in
    /// [`RuntimeAdapter::allocate_dispatch`] below. Encoding and submission are
    /// program work too and live in [`RuntimeAdapter::dispatch`].
    fn plan_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        preflight: &Preflight<'_>,
    ) -> Result<(), RouteRefusal> {
        self.session.stage("plan-dispatch");
        let limit = self.session.device.max_buffer_length();
        let routed = preflight.entries();

        // Paired first, because a shared allocation belongs to two entries and
        // neither owns it. A planner that sized per binding would let the
        // consumer be handed a fresh buffer and read uninitialised device memory
        // — a wrong answer rather than a refusal. Empty for a single-entry
        // route, which is a state rather than an absence.
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
            binding_fits(producer.entry(), producer.slot(), needed, limit)?;
            let index = shared_plan.len();
            paired[producer.entry()][producer.slot()] = Some(index);
            paired[consumer.entry()][consumer.slot()] = Some(index);
            shared_plan.push(SharedPlan {
                producer: (producer.entry(), producer.slot()),
                needed,
            });
        }
        self.session.note(format!(
            "plan: {} entry(ies), {} shared allocation(s)",
            routed.len(),
            preflight.shared_allocations().len(),
        ));
        self.session.journal.borrow_mut().shared_allocations =
            Some(preflight.shared_allocations().len());

        let mut binds_result = false;
        let mut plan = Vec::with_capacity(routed.len());
        for (position, entry) in routed.iter().enumerate() {
            let launch = entry.launch();
            if launch.grid_threads() == 0 && !launch.zero_work_skips_dispatch() {
                return Err(RouteRefusal::EmptyLaunchNotSkippable { entry: position });
            }
            let pipeline = self.prepared[position].clone();
            let capacity = pipeline.max_total_threads_per_threadgroup();
            if launch.threads_per_workgroup() > capacity {
                return Err(RouteRefusal::WorkgroupTooLarge {
                    entry: position,
                    declared: launch.threads_per_workgroup(),
                    capacity,
                });
            }

            let mut slots = Vec::with_capacity(entry.bindings().len());
            for binding in entry.bindings() {
                let slot = binding.slot();
                let needed = reach(routed, position, slot)?;
                binding_fits(position, slot, needed, limit)?;

                // An occupied slot is one half of a shared pair, and naming the
                // same plan index is what will make the two entries address one
                // buffer rather than two that merely have the same length.
                let backing = if let Some(index) = paired[position][slot] {
                    Backing::Shared(index)
                } else {
                    backing_for(&self.request, position, slot, binding.binding().target())?
                };
                binds_result |= backing == Backing::Output;
                slots.push(SlotPlan {
                    slot,
                    transport: binding.transport_slot(),
                    offset: binding.accessible_offset(),
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

        // Read off the plan's own targets rather than off whether an allocation
        // happened, which is the form this check must take once nothing is
        // allocated on this side of the commit.
        if !binds_result {
            return Err(RouteRefusal::NoOutputBinding);
        }
        self.plan = plan;
        self.shared_plan = shared_plan;
        Ok(())
    }

    /// Acquires and binds the committed route's device storage.
    ///
    /// Reached only from a [`RoutedDispatch`], which [`Preflight::commit`] is the
    /// only source of, so every `MTLBuffer` here is taken by the committed
    /// execution authority ADR 0051 reserves them to. Nothing is decided: the
    /// plan already fixed which storage each slot takes, how long it must be, and
    /// which storage mode it needs, and this stage acquires exactly that.
    ///
    /// The storage mode still follows from the binding target rather than from a
    /// default. A program input and the region's result are host-visible because
    /// bytes cross the boundary in both directions; entry-internal storage never
    /// leaves the device, so it is private.
    fn allocate_dispatch(
        &mut self,
        _: &LiveExecutionContext,
        _: &RoutedDispatch<'_>,
    ) -> Result<(), DispatchFailure> {
        self.session.stage("allocate-dispatch");
        let device = self.session.device.clone();

        // One allocation per pair, taken first so that both ends can name it.
        let mut shared: Vec<Buffer> = Vec::with_capacity(self.shared_plan.len());
        for pair in &self.shared_plan {
            let (entry, slot) = pair.producer;
            let buffer =
                device.new_buffer(pair.needed.max(1), MTLResourceOptions::StorageModePrivate);
            allocation_holds(entry, slot, pair.needed, &buffer)?;
            shared.push(buffer);
        }

        // Taken out rather than borrowed, so this loop can read the operands the
        // plan it is consuming named.
        let plan = std::mem::take(&mut self.plan);
        let mut output = None;
        let mut planned = Vec::with_capacity(plan.len());
        for (position, entry) in plan.into_iter().enumerate() {
            let mut slots = Vec::with_capacity(entry.slots.len());
            for placed in entry.slots {
                let buffer = match &placed.backing {
                    Backing::Shared(index) => shared[*index].clone(),
                    Backing::ProgramInput(key) => {
                        // Sized from the operand rather than from `needed`, so a
                        // binding addressing a window at a nonzero offset still
                        // has the bytes preceding it to address past.
                        let bytes = self
                            .request
                            .operand(key)
                            .expect("the plan proved this region supplies this operand");
                        let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
                        device.new_buffer_with_data(
                            bytes.as_ptr().cast::<std::ffi::c_void>(),
                            length.max(1),
                            MTLResourceOptions::StorageModeShared,
                        )
                    }
                    Backing::Output => {
                        let buffer = device.new_buffer(
                            placed.needed.max(1),
                            MTLResourceOptions::StorageModeShared,
                        );
                        output = Some(buffer.clone());
                        buffer
                    }
                    Backing::Internal => device
                        .new_buffer(placed.needed.max(1), MTLResourceOptions::StorageModePrivate),
                };
                allocation_holds(position, placed.slot, placed.needed, &buffer)?;
                slots.push(PlacedSlot {
                    transport: placed.transport,
                    offset: placed.offset,
                    buffer,
                });
            }
            planned.push(PlannedEntry {
                pipeline: entry.pipeline,
                slots,
                grid_threads: entry.grid_threads,
                threads_per_workgroup: entry.threads_per_workgroup,
                skipped: entry.skipped,
            });
        }

        self.output =
            Some(output.expect("the pre-commit plan proved one slot binds the region's result"));
        self.planned = planned;
        Ok(())
    }

    /// Encodes, submits, and observes the committed route to terminal success.
    ///
    /// Everything here is past the one-way commit: nothing is looked up, nothing
    /// is allocated, and there is no refusal left to make. What remains is the
    /// encode, the submission, the terminal-status check that must precede any
    /// read of device memory, and the read itself into the region's own result
    /// storage.
    ///
    /// **One encoder per entry, and that is the ordering guarantee.** Commands
    /// within a single compute encoder are not ordered against each other unless
    /// the encoder's dispatch type says so, and a second stage reading what the
    /// first wrote must not overlap it. Metal orders *encoders* within a command
    /// buffer unconditionally, with an implicit barrier between them — so the
    /// order this loop creates them in *is* the order the entries execute in,
    /// which is what [`Perturbation::ReverseEncodeOrder`] perturbs and nothing
    /// else here does.
    fn dispatch(
        &mut self,
        context: &LiveExecutionContext,
        routed: &RoutedDispatch<'_>,
    ) -> Result<Completion, DispatchFailure> {
        self.session.stage("dispatch");
        let queue = self.session.device.new_command_queue();
        let command_buffer = queue.new_command_buffer();
        let mut encoded = 0_usize;
        // The artifact's declared execution order, or its reverse under the
        // perturbation. Positions rather than a reversed iterator so that the
        // sound path's traversal is the one written here and the perturbation is
        // the single statement that moves it.
        let mut order: Vec<usize> = (0..self.planned.len()).collect();
        if reverses_encode_order(self.session.perturbation) {
            order.reverse();
        }
        for position in order {
            let entry = &self.planned[position];
            if entry.skipped {
                continue;
            }
            encode_entry(command_buffer, entry);
            encoded += 1;
        }

        // The halting perturbation, and the whole of it. Everything above is the
        // sound path's own encode; what this arm withholds is the submission,
        // which is program work after the routing commit. The status check below
        // is then reached with a live, never-committed command buffer — a
        // non-terminal state — and refuses the readback. A reordered route is
        // submitted and waited exactly like a sound one, because a reordering
        // that refused would not be the failure it is here to watch.
        if self.session.perturbation != Some(Perturbation::HaltAfterCommit) {
            command_buffer.commit();
            command_buffer.wait_until_completed();
        }

        // The only decision left after the commit, and the only path to a
        // readback. Read *after* the wait: `waitUntilCompleted` returns no
        // success value, so a pre-wait status is not evidence of completion.
        match submission_outcome(command_buffer.status()) {
            SubmissionOutcome::Completed => {}
            SubmissionOutcome::ExecutionError => return Err(DispatchFailure::ExecutionError),
            SubmissionOutcome::NotTerminal(status) => {
                return Err(DispatchFailure::NonTerminalStatus { status });
            }
        }

        // Into the region's own result storage, which is the value the caller
        // receives. Cloned out of `self.output` first because the read borrows
        // the request mutably and the plan immutably.
        let output = self
            .output
            .clone()
            .expect("allocate_dispatch bound the result from the committed route");
        buffer::read_into(&output, self.request.result_mut());

        let completion = Completion {
            profile_key: context.target_profile().key.as_str().to_owned(),
            encoded,
            entries: routed.entries().len(),
        };
        // Recorded here because this is the only place it can be: returning
        // `Ok` from this method is what makes `route_with_adapter` return
        // `Ok(completion)`, which is what makes the facade's outcome
        // `RouteOutcome::Dispatched`. Reaching this line at all is the commit
        // evidence — `route_with_adapter` calls `Preflight::commit()` on the
        // line before it calls this method, and nothing else calls it.
        self.session.note(format!(
            "committed route completed: {}/{} entry(ies) encoded, terminal status Completed, \
             profile {}",
            completion.encoded, completion.entries, completion.profile_key,
        ));
        self.session.journal.borrow_mut().completion = Some(completion.clone());
        Ok(completion)
    }
}

/// What one completed dispatch yields.
///
/// The result itself is not here: it was written into the region's own storage,
/// which is what the caller receives. This is the account of *what ran*.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Completion {
    /// The governed profile key the route was carried out under.
    pub profile_key: String,
    /// How many entries were encoded.
    pub encoded: usize,
    /// How many entries the route declared.
    pub entries: usize,
}

/// Encodes one prepared entry into one compute encoder of its own.
fn encode_entry(command_buffer: &CommandBufferRef, entry: &PlannedEntry) {
    let encoder = command_buffer.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&entry.pipeline);
    for slot in &entry.slots {
        encoder.set_buffer(u64::from(slot.transport), Some(&slot.buffer), slot.offset);
    }
    encoder.dispatch_threads(
        MTLSize::new(entry.grid_threads, 1, 1),
        MTLSize::new(entry.threads_per_workgroup, 1, 1),
    );
    encoder.end_encoding();
}

/// Decides which storage one binding target will take, acquiring none of it.
///
/// The three placeable targets are three different obligations, and both of the
/// refusals below are decidable from the target and the region's own handover
/// alone — which is exactly why they belong on the pre-commit side of the seam
/// rather than travelling with the allocation that used to make them.
///
/// # Errors
///
/// Returns [`RouteRefusal::UnsuppliedProgramInput`] for a named input this
/// region did not hand over, and [`RouteRefusal::UnboundBindingTarget`] for a
/// program output under a key that is not this region's result.
fn backing_for(
    request: &RegionRequest<'_>,
    entry: usize,
    slot: usize,
    target: BindingTarget<'_>,
) -> Result<Backing, RouteRefusal> {
    match target {
        // The caller's own operand. Its presence is asked here so that a region
        // that did not hand it over is refused while a fallback is permitted.
        BindingTarget::ProgramInput(key) => {
            if request.operand(key.as_str()).is_none() {
                return Err(RouteRefusal::UnsuppliedProgramInput {
                    entry,
                    key: key.as_str().to_owned(),
                });
            }
            Ok(Backing::ProgramInput(key.as_str().to_owned()))
        }
        // The region's result. Matched against the key the region declared its
        // result under rather than accepted for any output: a plan publishing a
        // value this region did not ask for is something to refuse, not
        // something to read back.
        BindingTarget::ProgramOutput(keys)
            if keys.iter().any(|key| key.as_str() == request.result_key()) =>
        {
            Ok(Backing::Output)
        }
        BindingTarget::Internal => Ok(Backing::Internal),
        other @ BindingTarget::ProgramOutput(_) => Err(RouteRefusal::UnboundBindingTarget {
            entry,
            slot,
            target: format!("{other:?}"),
        }),
    }
}

/// Returns the last byte one routed binding must be able to reach.
fn reach(entries: &[RoutedEntry<'_>], entry: usize, slot: usize) -> Result<u64, RouteRefusal> {
    let binding = entries[entry].bindings()[slot];
    binding
        .accessible_offset()
        .checked_add(binding.accessible_bytes())
        .ok_or(RouteRefusal::BindingRangeOverflow { entry, slot })
}

/// Whether one binding's accessible range fits in a single buffer here.
fn binding_fits(entry: usize, slot: usize, needed: u64, limit: u64) -> Result<(), RouteRefusal> {
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

/// Whether an allocation the device returned reaches the length the plan sized it for.
///
/// Against the buffer's own report rather than against a number computed twice:
/// every allocation is requested at the length the route states, so reaching
/// this means the allocator did not honour a request it accepted.
///
/// Post-commit, because the allocation it inspects is — which is why it yields a
/// [`DispatchFailure`]. There is no second route to take once the storage has
/// been acquired, so this is a defect report rather than a routing input.
fn allocation_holds(
    entry: usize,
    slot: usize,
    needed: u64,
    buffer: &Buffer,
) -> Result<(), DispatchFailure> {
    let held = buffer.length();
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
/// result held before, which compares against the oracle as a numerical
/// disagreement rather than as the dispatch failure it is.
///
/// Matched exhaustively and wildcard-free, which `metal` 0.33.0's real enum
/// admits: a status added to the binding is a build error here rather than
/// falling into whichever arm a catch-all named.
#[must_use]
pub const fn submission_outcome(status: MTLCommandBufferStatus) -> SubmissionOutcome {
    match status {
        MTLCommandBufferStatus::Completed => SubmissionOutcome::Completed,
        MTLCommandBufferStatus::Error => SubmissionOutcome::ExecutionError,
        MTLCommandBufferStatus::NotEnqueued => SubmissionOutcome::NotTerminal("NotEnqueued"),
        MTLCommandBufferStatus::Enqueued => SubmissionOutcome::NotTerminal("Enqueued"),
        MTLCommandBufferStatus::Committed => SubmissionOutcome::NotTerminal("Committed"),
        MTLCommandBufferStatus::Scheduled => SubmissionOutcome::NotTerminal("Scheduled"),
    }
}
