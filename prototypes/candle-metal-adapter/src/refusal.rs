//! Every way this adapter says no, and the numerical realization it says it delivered.
//!
//! # Three vocabularies, because they are answered at three different moments
//!
//! [`TensorRefusal`] is decided from Tensor-visible facts alone, before Candle's
//! custom-op path is entered. It is the **only** place a fallback is expressible:
//! the wrapper still owns the Tensor expression there, nothing has been
//! allocated, and no device object exists.
//!
//! [`RouteRefusal`] is the adapter's own pre-commit vocabulary. Every value of it
//! arrives while `tiler_runtime`'s routing authority is still uncommitted, so
//! ADR 0051 would still permit a fallback — but this consumer has already
//! foreclosed one by entering the custom op, so each of these is a typed error
//! rather than a route change. See [`crate::wrapper`] for why that foreclosure is
//! the stricter and correct reading.
//!
//! [`DispatchFailure`] is post-commit. It reports and is never retried.
//!
//! # The rejection classes this profile refuses by name
//!
//! The contiguous / no-autograd first profile is enforced by
//! [`TensorRefusal`], and the complete list of what it refuses is the set of
//! variants below rather than a comment that can drift from them. Affine-strided
//! layouts are [`TensorRefusal::AffineStridedLayout`] specifically, and are
//! tracked as explicitly beyond this profile by `docs/open-questions.md`
//! Q-RUNTIME-002; a broadcast view is a separate variant because it is not a
//! stride pattern a later affine-stride variant would subsume — it aliases one
//! element into many, which is an aliasing question rather than an indexing one.

use std::fmt;

use candle_core::DType;
use tiler_runtime::load::TargetCompatibility;

/// The numerical realization a delivered result was produced under.
///
/// Carried with the operations it covers, never alone.
/// `docs/integration/candle.md` makes that pairing obligatory: a consumer reads
/// one tensor whose value composes several numerical contracts, and Tiler states
/// one of them, so a realization reported without its scope invites exactly the
/// misattribution the contract's numerical-scope section describes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Realization {
    /// The contract the artifact's kernels were emitted and compiled under.
    ///
    /// `FlushSubnormalsToZeroF32` with a strictly ordered serial reduction, from
    /// a toolchain the artifact's provenance names.
    TilerFlushSubnormalsToZeroF32StrictOrder,
    /// What Candle's own built-in kernels deliver for the same expression.
    ///
    /// Named rather than described as "the fallback", because the point of the
    /// comparison is that it is a *different* realization: a different compiler
    /// build, a different math mode, and — for a reduction — a different
    /// summation order.
    CandleBuiltinF32,
}

impl Realization {
    /// A stable identifier for reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TilerFlushSubnormalsToZeroF32StrictOrder => {
                "tiler.f32.flush-subnormals-to-zero.strict-order"
            }
            Self::CandleBuiltinF32 => "candle.builtin.f32",
        }
    }

    /// Whether this realization fixes the order in which contributions are summed.
    ///
    /// The one property that decides whether the ordinary Candle expression may
    /// stand in for this artifact. Candle's reductions are free to associate,
    /// and a strictly ordered serial sum is a different function on operands
    /// where floating-point addition is not associative — which is every
    /// interesting one.
    pub const fn fixes_reduction_order(self) -> bool {
        match self {
            Self::TilerFlushSubnormalsToZeroF32StrictOrder => true,
            Self::CandleBuiltinF32 => false,
        }
    }
}

impl fmt::Display for Realization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// What a completed wrapper call delivered, and over which operations.
///
/// Returned on the success path so a caller can tell a fast path from a
/// fallback without inspecting timing, which is criterion 2's "the wrapper must
/// be able to report *which* numerical realization it delivered".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delivered {
    /// Which of the two paths produced the result.
    pub path: DeliveredPath,
    /// The realization the produced values were computed under.
    pub realization: Realization,
    /// The operations that realization covers, and only those.
    pub covered_operations: &'static str,
}

impl fmt::Display for Delivered {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} realization {} covering {}",
            self.path.as_str(),
            self.realization,
            self.covered_operations,
        )
    }
}

/// Which of the two paths produced a delivered result.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeliveredPath {
    /// The Tiler artifact ran as a Candle custom op.
    TilerArtifact,
    /// The ordinary Candle expression ran instead.
    CandleExpression,
}

impl DeliveredPath {
    /// A stable identifier for reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TilerArtifact => "tiler-artifact",
            Self::CandleExpression => "candle-expression",
        }
    }
}

/// Whether the ordinary Candle expression may stand in for a refused route.
///
/// Decided from the *requested* realization rather than from the refusal, and
/// that is the whole point: `docs/integration/candle.md` requires a fallback to
/// match the requested semantics' numerical contract before it may be selected,
/// so a request Candle's kernels cannot realize has no valid fallback however
/// benign the reason for refusing was.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FallbackAvailability {
    /// Candle's own kernels realize the requested contract.
    Available,
    /// They do not, and the wrapper must fail closed naming what is unmet.
    Unavailable {
        /// The realization the caller asked for.
        requested: Realization,
        /// What the ordinary Candle expression would deliver instead.
        candle_delivers: Realization,
    },
}

/// Decides whether the ordinary Candle expression realizes a requested contract.
///
/// One rule, applied rather than special-cased: a request that fixes the
/// reduction order cannot be served by kernels that do not, because floating-point
/// addition is not associative and the two are then different functions of the
/// same operands. Everything else this profile can request is realizable by
/// both.
pub const fn fallback_availability(requested: Realization) -> FallbackAvailability {
    let candle_delivers = Realization::CandleBuiltinF32;
    if requested.fixes_reduction_order() && !candle_delivers.fixes_reduction_order() {
        return FallbackAvailability::Unavailable {
            requested,
            candle_delivers,
        };
    }
    FallbackAvailability::Available
}

/// Why the tensor-level preflight declined to apply the custom op.
///
/// Decided before Candle's custom-op path is entered, from Tensor-visible facts
/// and the artifact's own declared interface. Deliberately **not**
/// `#[non_exhaustive]`: this enum *is* the profile boundary, and a case added to
/// it must stop every reader's build rather than reach a wildcard that treats an
/// unclassified layout as supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TensorRefusal {
    /// The tensor does not live on a Metal device.
    NotAMetalDevice {
        /// How Candle describes where it does live.
        observed: String,
    },
    /// The tensor lives on a *different* Metal device from the one this adapter bound.
    ///
    /// Separate from [`Self::NotAMetalDevice`] because the remedy differs: this
    /// one is a real Metal tensor that would run correctly on its own device,
    /// and binding it here would encode against an allocation this queue cannot
    /// address.
    ForeignMetalDevice {
        /// How Candle identifies the tensor's device.
        tensor: String,
        /// How Candle identifies the device this adapter bound.
        adapter: String,
    },
    /// The element type is outside this profile.
    UnsupportedDtype {
        /// The dtype the tensor carries.
        observed: DType,
        /// The one dtype this profile's artifacts declare.
        supported: DType,
    },
    /// The view is contiguous in Fortran order or under some other permutation.
    ///
    /// The affine-strided case `docs/open-questions.md` Q-RUNTIME-002 tracks as
    /// beyond this profile. Refused by name rather than relayed out to a
    /// contiguous copy, because a silent copy is a different program with
    /// different allocation behaviour than the one the caller asked for.
    AffineStridedLayout {
        /// The view's extents.
        dims: Vec<usize>,
        /// The view's element strides.
        stride: Vec<usize>,
    },
    /// The view aliases one element into many through a zero stride.
    ///
    /// Not a stride pattern a later affine-stride variant subsumes: a broadcast
    /// view is an aliasing fact, and the initial integration is out-of-place and
    /// proves no aliasing.
    BroadcastView {
        /// The view's extents.
        dims: Vec<usize>,
        /// The view's element strides.
        stride: Vec<usize>,
    },
    /// The tensor's rank is not the one the artifact's interface declares.
    UnsupportedRank {
        /// The tensor's rank.
        observed: usize,
        /// The rank the artifact's declared input has.
        required: usize,
    },
    /// One extent disagrees with the artifact's declared input shape.
    ExtentMismatch {
        /// Zero-based axis.
        axis: usize,
        /// What the artifact declares.
        declared: u64,
        /// What the tensor carries.
        observed: usize,
    },
    /// The tensor participates in tracked autograd.
    ///
    /// A fused forward custom operation provides no gradient, and
    /// `docs/integration/candle.md` is explicit that silently breaking autograd
    /// is not acceptable. Refused rather than applied without a backward
    /// formula.
    AutogradTracked,
    /// The artifact's declared interface is not the one this wrapper implements.
    ForeignInterface {
        /// What disagreed.
        detail: String,
    },
    /// The artifact declares a target profile this host does not offer.
    ///
    /// "Target availability", decided before the custom op is entered so a host
    /// that cannot run the artifact at all never reaches Candle's path.
    IncompatibleTargetProfile {
        /// The loader's own classification.
        classification: TargetCompatibility,
    },
    /// A refusal above would have fallen back, and no valid fallback exists.
    ///
    /// The fail-closed rule `docs/integration/candle.md` derives: a strict
    /// numerical contract removes the Candle fallback rather than weakening it,
    /// so this carries both the refusal that would have fallen back and the
    /// realization the fallback could not deliver.
    NoRealizableFallback {
        /// The refusal that would otherwise have selected the fallback.
        refused: Box<TensorRefusal>,
        /// The realization the caller asked for.
        requested: Realization,
        /// What the ordinary Candle expression would have delivered.
        candle_delivers: Realization,
    },
}

impl fmt::Display for TensorRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAMetalDevice { observed } => write!(
                formatter,
                "candle.preflight.device: this adapter binds Metal storage and the tensor lives \
                 on {observed}",
            ),
            Self::ForeignMetalDevice { tensor, adapter } => write!(
                formatter,
                "candle.preflight.foreign-device: the tensor's storage belongs to {tensor} and \
                 this adapter bound {adapter}",
            ),
            Self::UnsupportedDtype {
                observed,
                supported,
            } => write!(
                formatter,
                "candle.preflight.dtype: this profile's artifacts declare {} and the tensor \
                 carries {}",
                supported.as_str(),
                observed.as_str(),
            ),
            Self::AffineStridedLayout { dims, stride } => write!(
                formatter,
                "candle.preflight.affine-strided-layout: dims {dims:?} with strides {stride:?} is \
                 not a contiguous view, and affine-strided support is beyond this first profile \
                 (Q-RUNTIME-002)",
            ),
            Self::BroadcastView { dims, stride } => write!(
                formatter,
                "candle.preflight.broadcast-view: dims {dims:?} with strides {stride:?} aliases \
                 one element into many, and this integration is out-of-place with no alias proof",
            ),
            Self::UnsupportedRank { observed, required } => write!(
                formatter,
                "candle.preflight.rank: the artifact's declared input is rank {required} and the \
                 tensor is rank {observed}",
            ),
            Self::ExtentMismatch {
                axis,
                declared,
                observed,
            } => write!(
                formatter,
                "candle.preflight.extent: the artifact declares {declared} along axis {axis} and \
                 the tensor carries {observed}",
            ),
            Self::AutogradTracked => formatter.write_str(
                "candle.preflight.autograd: the tensor participates in tracked autograd and this \
                 fused forward op carries no backward formula",
            ),
            Self::ForeignInterface { detail } => write!(
                formatter,
                "candle.preflight.interface: the artifact does not declare this wrapper's \
                 interface: {detail}",
            ),
            Self::IncompatibleTargetProfile { classification } => write!(
                formatter,
                "candle.preflight.target: the artifact declares a profile this host does not \
                 offer: {classification:?}",
            ),
            Self::NoRealizableFallback {
                refused,
                requested,
                candle_delivers,
            } => write!(
                formatter,
                "candle.preflight.no-realizable-fallback: {refused}, and the ordinary Candle \
                 expression delivers {candle_delivers} where {requested} was requested, so there \
                 is no fallback to take",
            ),
        }
    }
}

impl std::error::Error for TensorRefusal {}

/// Why this adapter refused a route before it committed.
///
/// Deliberately not `#[non_exhaustive]` for the same reason [`TensorRefusal`] is
/// not: these are the exact obligations the adapter discharges, and one added
/// later must be classified rather than absorbed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteRefusal {
    /// No Metal device or command queue could be bound.
    NoExecutionContext {
        /// What the binding reported.
        detail: String,
    },
    /// A carried payload's bytes did not decode into a Metal library.
    ///
    /// The envelope already proved this object's integrity digest, so these are
    /// the bytes the producer published and they are content that will not
    /// execute — a rebuild, not a re-fetch of a damaged file.
    PayloadNotALibrary {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// What the Metal binding reported.
        detail: String,
    },
    /// The library loaded and publishes no function by the declared entry symbol.
    EntrySymbolAbsent {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The symbol the artifact declares.
        symbol: String,
        /// What the Metal binding reported.
        detail: String,
    },
    /// The device refused pipeline state for a function it did publish.
    PipelineRejected {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The symbol the pipeline was built for.
        symbol: String,
        /// What the Metal binding reported.
        detail: String,
    },
    /// A cache lookup named a device scope this cache was not built for.
    ///
    /// Unreachable through this adapter's own code, because a key carries the
    /// scope it was minted under and a lookup from another device therefore
    /// cannot match one. Retained as the assertion that makes that structural
    /// claim checkable rather than only stated.
    ForeignDeviceScope {
        /// The scope the cache holds.
        cache: String,
        /// The scope the lookup named.
        lookup: String,
    },
    /// The declared workgroup is larger than this pipeline admits.
    WorkgroupTooLarge {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// The symbol whose pipeline was queried.
        symbol: String,
        /// What the artifact declares.
        declared: u64,
        /// What the pipeline admits.
        capacity: u64,
    },
    /// A binding must reach more bytes than one buffer holds on this device.
    BindingExceedsBufferLimit {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// Bytes the route requires be reachable.
        needed: u64,
        /// The device's single-buffer limit.
        limit: u64,
    },
    /// A binding's accessible range does not fit the storage bound to it.
    UndersizedStorage {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// Bytes the route requires be reachable.
        needed: u64,
        /// Bytes the bound allocation holds.
        held: u64,
    },
    /// A binding's offset plus its extent does not fit an unsigned range.
    BindingRangeOverflow {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// The evaluated starting offset.
        offset: u64,
        /// The evaluated extent.
        extent: u64,
    },
    /// A routed binding addresses something this consumer supplies no storage for.
    UnboundBindingTarget {
        /// Position of the entry in the route's execution order.
        entry: usize,
        /// Zero-based ABI slot.
        slot: usize,
        /// How the artifact describes what the slot addresses.
        target: String,
    },
    /// An entry covers no threads and the artifact does not say to skip it.
    ///
    /// `dispatchThreads` has no meaning at zero, and inventing one thread would
    /// run a body the plan did not ask for.
    EmptyLaunchNotSkippable {
        /// Position of the entry in the route's execution order.
        entry: usize,
    },
    /// No entry of this route binds the program output.
    NoOutputBinding,
    /// Candle's allocator refused a request this route needs.
    Allocation {
        /// What Candle reported.
        detail: String,
    },
    /// Candle's own pending work could not be brought to a terminal state.
    ///
    /// Flushed before anything is encoded, because the tensor this route reads
    /// may still be a promise in Candle's uncommitted command buffer, and a
    /// separate command buffer is not ordered against it.
    PendingCandleWork {
        /// What Candle reported.
        detail: String,
    },
}

impl fmt::Display for RouteRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoExecutionContext { detail } => write!(
                formatter,
                "candle-metal.context: no live execution context was bound: {detail}",
            ),
            Self::PayloadNotALibrary { entry, detail } => write!(
                formatter,
                "candle-metal.payload: entry {entry}'s carried object did not load as a Metal \
                 library: {detail}",
            ),
            Self::EntrySymbolAbsent {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "candle-metal.payload: entry {entry}'s library publishes no {symbol:?}: {detail}",
            ),
            Self::PipelineRejected {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "candle-metal.prepare: no pipeline state for entry {entry}'s {symbol:?}: {detail}",
            ),
            Self::ForeignDeviceScope { cache, lookup } => write!(
                formatter,
                "candle-metal.cache: this cache is scoped to {cache} and the lookup named \
                 {lookup}",
            ),
            Self::WorkgroupTooLarge {
                entry,
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "candle-metal.plan: entry {entry}'s {symbol:?} admits {capacity} thread(s) per \
                 threadgroup and the artifact declares {declared}",
            ),
            Self::BindingExceedsBufferLimit {
                entry,
                slot,
                needed,
                limit,
            } => write!(
                formatter,
                "candle-metal.plan: entry {entry} slot {slot} must reach {needed} byte(s) and one \
                 buffer holds at most {limit}",
            ),
            Self::UndersizedStorage {
                entry,
                slot,
                needed,
                held,
            } => write!(
                formatter,
                "candle-metal.plan: entry {entry} slot {slot} needs {needed} byte(s) reachable \
                 and the bound storage holds {held}",
            ),
            Self::BindingRangeOverflow {
                entry,
                slot,
                offset,
                extent,
            } => write!(
                formatter,
                "candle-metal.plan: entry {entry} slot {slot} starts at {offset} and reaches \
                 {extent} more byte(s), which is not an addressable range",
            ),
            Self::UnboundBindingTarget {
                entry,
                slot,
                target,
            } => write!(
                formatter,
                "candle-metal.plan: entry {entry} slot {slot} addresses {target}, and this \
                 consumer supplies no storage for it",
            ),
            Self::EmptyLaunchNotSkippable { entry } => write!(
                formatter,
                "candle-metal.plan: entry {entry} covers no threads and the artifact does not \
                 declare its dispatch skippable",
            ),
            Self::NoOutputBinding => formatter
                .write_str("candle-metal.plan: no entry of this route binds the program output"),
            Self::Allocation { detail } => {
                write!(
                    formatter,
                    "candle-metal.plan: Candle's allocator refused: {detail}"
                )
            }
            Self::PendingCandleWork { detail } => write!(
                formatter,
                "candle-metal.plan: Candle's pending work did not reach a terminal state: \
                 {detail}",
            ),
        }
    }
}

impl std::error::Error for RouteRefusal {}

/// Why a committed dispatch did not complete.
///
/// Reported and never retried: reaching any of these means the route committed,
/// and ADR 0051 forbids selecting another plan afterwards.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchFailure {
    /// The command buffer could not be created or encoded into.
    EncoderUnavailable {
        /// Position of the entry in the route's execution order, or the route
        /// itself when the command buffer never existed.
        entry: Option<usize>,
    },
    /// The device reported a terminal execution error for the submission.
    ///
    /// Unlike `prototypes/serial-sum-run`, the error's own text is carried:
    /// Candle's `CommandBuffer::error` reads `MTLCommandBuffer.error`'s
    /// localized description, which the `metal` 0.33 binding does not expose.
    CommandBufferError {
        /// Metal's own account, when it supplied one.
        detail: Option<String>,
    },
    /// The wait returned with the command buffer in a non-terminal state.
    ///
    /// Carried separately from an execution error because which non-terminal
    /// state it stopped in is the diagnostic: `NotEnqueued` means nothing was
    /// ever submitted, and `Scheduled` means the work was accepted and had not
    /// finished.
    NonTerminalStatus {
        /// The status name the binding reported.
        status: &'static str,
    },
}

impl fmt::Display for DispatchFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EncoderUnavailable { entry } => match entry {
                Some(entry) => write!(
                    formatter,
                    "candle-metal.dispatch: entry {entry} could not be encoded",
                ),
                None => {
                    formatter.write_str("candle-metal.dispatch: no command buffer could be created")
                }
            },
            Self::CommandBufferError { detail } => match detail {
                Some(detail) => write!(
                    formatter,
                    "candle-metal.dispatch: the device reported an execution error: {detail}",
                ),
                None => formatter.write_str(
                    "candle-metal.dispatch: the device reported an execution error and no detail",
                ),
            },
            Self::NonTerminalStatus { status } => write!(
                formatter,
                "candle-metal.dispatch: the wait returned with the command buffer {status}",
            ),
        }
    }
}

impl std::error::Error for DispatchFailure {}

#[cfg(test)]
mod tests {
    use super::{
        DeliveredPath, FallbackAvailability, Realization, RouteRefusal, TensorRefusal,
        fallback_availability,
    };
    use candle_core::DType;

    /// A strictly ordered request has no Candle fallback, and a free-order one does.
    ///
    /// Both arms are asserted because the rule is a comparison rather than a
    /// constant: a version that always returned `Unavailable` would satisfy this
    /// crate's own program and be wrong for every other one.
    #[test]
    fn only_an_order_fixing_request_loses_its_fallback() {
        assert_eq!(
            fallback_availability(Realization::TilerFlushSubnormalsToZeroF32StrictOrder),
            FallbackAvailability::Unavailable {
                requested: Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
                candle_delivers: Realization::CandleBuiltinF32,
            },
        );
        assert_eq!(
            fallback_availability(Realization::CandleBuiltinF32),
            FallbackAvailability::Available,
        );
    }

    /// Every tensor-level refusal renders distinguishably and names its class.
    ///
    /// The population is written out rather than derived, so a variant added
    /// without a rendering is a missing row here as well as a build error in the
    /// `Display` match.
    #[test]
    fn every_tensor_refusal_names_a_distinct_class() {
        let rendered: Vec<String> = [
            TensorRefusal::NotAMetalDevice {
                observed: "Cpu".to_owned(),
            },
            TensorRefusal::ForeignMetalDevice {
                tensor: "DeviceId(2)".to_owned(),
                adapter: "DeviceId(1)".to_owned(),
            },
            TensorRefusal::UnsupportedDtype {
                observed: DType::F16,
                supported: DType::F32,
            },
            TensorRefusal::AffineStridedLayout {
                dims: vec![2, 3],
                stride: vec![1, 2],
            },
            TensorRefusal::BroadcastView {
                dims: vec![2, 3],
                stride: vec![0, 1],
            },
            TensorRefusal::UnsupportedRank {
                observed: 1,
                required: 2,
            },
            TensorRefusal::ExtentMismatch {
                axis: 1,
                declared: 3,
                observed: 4,
            },
            TensorRefusal::AutogradTracked,
            TensorRefusal::ForeignInterface {
                detail: "two inputs".to_owned(),
            },
            TensorRefusal::IncompatibleTargetProfile {
                classification: tiler_runtime::load::TargetCompatibility::DescriptorMismatch {
                    key: "tiler.metal.macos-apple9.msl4-0.f32.v1".to_owned(),
                },
            },
            TensorRefusal::NoRealizableFallback {
                refused: Box::new(TensorRefusal::AutogradTracked),
                requested: Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
                candle_delivers: Realization::CandleBuiltinF32,
            },
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                text.starts_with("candle.preflight."),
                "{text:?} does not name the phase it was decided in",
            );
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier refusal",
            );
        }
    }

    /// Every route refusal renders distinguishably and names its stage.
    #[test]
    fn every_route_refusal_names_a_distinct_stage() {
        let rendered: Vec<String> = [
            RouteRefusal::NoExecutionContext {
                detail: "no device".to_owned(),
            },
            RouteRefusal::PayloadNotALibrary {
                entry: 0,
                detail: "not a metallib".to_owned(),
            },
            RouteRefusal::EntrySymbolAbsent {
                entry: 0,
                symbol: "tiler_kernel".to_owned(),
                detail: "absent".to_owned(),
            },
            RouteRefusal::PipelineRejected {
                entry: 0,
                symbol: "tiler_kernel".to_owned(),
                detail: "refused".to_owned(),
            },
            RouteRefusal::ForeignDeviceScope {
                cache: "a".to_owned(),
                lookup: "b".to_owned(),
            },
            RouteRefusal::WorkgroupTooLarge {
                entry: 0,
                symbol: "tiler_kernel".to_owned(),
                declared: 2,
                capacity: 1,
            },
            RouteRefusal::BindingExceedsBufferLimit {
                entry: 0,
                slot: 0,
                needed: 2,
                limit: 1,
            },
            RouteRefusal::UndersizedStorage {
                entry: 0,
                slot: 0,
                needed: 2,
                held: 1,
            },
            RouteRefusal::BindingRangeOverflow {
                entry: 0,
                slot: 0,
                offset: u64::MAX,
                extent: 1,
            },
            RouteRefusal::UnboundBindingTarget {
                entry: 0,
                slot: 0,
                target: "ProgramInput(\"other\")".to_owned(),
            },
            RouteRefusal::EmptyLaunchNotSkippable { entry: 0 },
            RouteRefusal::NoOutputBinding,
            RouteRefusal::Allocation {
                detail: "out of memory".to_owned(),
            },
            RouteRefusal::PendingCandleWork {
                detail: "command buffer error".to_owned(),
            },
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        for (position, text) in rendered.iter().enumerate() {
            assert!(
                text.starts_with("candle-metal."),
                "{text:?} does not name the backend that refused",
            );
            assert!(
                !rendered[..position].contains(text),
                "{text:?} is not distinguishable from an earlier refusal",
            );
        }
    }

    /// The two delivered paths are distinguishable in a report.
    #[test]
    fn a_delivered_path_is_readable() {
        assert_ne!(
            DeliveredPath::TilerArtifact.as_str(),
            DeliveredPath::CandleExpression.as_str(),
        );
    }
}
