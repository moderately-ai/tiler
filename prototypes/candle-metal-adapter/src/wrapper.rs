//! The Tensor-level wrapper: preflight, the fallback decision, and the custom op.
//!
//! # Where the fallback lives, and why it is only here
//!
//! `docs/integration/candle.md` puts semantic fallback at the Tensor level and
//! nowhere else, and this module is the whole of that level. Before
//! [`TilerPlan::apply`] enters Candle's custom-op path it has decided every
//! question a Tensor can answer — device, dtype, rank, extents, contiguity,
//! aliasing, autograd — and it has decided target availability from the
//! artifact's own declared profile. Nothing has been allocated, no device object
//! exists, and the wrapper still owns the ordinary Candle expression, so
//! declining here is a real choice between two ways of computing the result.
//!
//! Once `apply_op1_no_bwd` is entered, that choice is spent. Everything the
//! adapter refuses afterwards is a typed error rather than a route change, and
//! that is deliberately **stricter** than ADR 0051 requires: the seam permits a
//! fallback for every pre-commit refusal, and this ticket's criterion 2
//! forecloses one from the custom-op boundary onward regardless. Foreclosing
//! there is the only place that rule can be enforced without inspecting which
//! stage refused.
//!
//! The original ground for that strictness — that the adapter allocated output
//! storage during `plan_dispatch`, so a pre-commit refusal could follow an
//! allocation — no longer holds:
//! [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`] moved allocation
//! into `allocate_dispatch`, past the commit, so nothing is acquired before a
//! refusal any more. The custom-op boundary stays the foreclosure point anyway,
//! because Candle's own choice between the two ways of computing the result is
//! what is spent there, and that is unrelated to what the adapter allocated.
//!
//! [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`]: ../../../tickets/reconcile-the-pre-commit-allocation-seam-with-adr-0051.md
//!
//! # The friction this design records
//!
//! It would be better to run the adapter's device-dependent pre-commit stages —
//! payload validation, the live-device rows, pipeline preparation, the
//! prepared-entry properties, and the sizing — *before* `apply_op1_no_bwd`, and
//! to enter Candle's path holding a prepared selection that only commits,
//! allocates, and dispatches. That is exactly what the contract's
//! `PreparedSelection` token describes, and it is not expressible against the
//! seam as accepted:
//!
//! - `route_with_adapter` is the only driver of a [`RuntimeAdapter`], and it runs
//!   stages 1 through 10 in one call. There is no way to stop it after stage 7.
//! - Driving the trait methods by hand instead is impossible, because every one
//!   of them takes a `&LiveExecutionContext` and that type has **no public
//!   constructor** — `route_with_adapter` mints the only value that ever exists.
//!   The property is deliberate and compiler-checked, and its cost here is that
//!   a consumer cannot straddle a foreign framework's callback boundary.
//!
//! So the pre-commit device stages necessarily run inside `metal_fwd`. What that
//! costs is stated rather than hidden: a device-side refusal is reported after
//! Candle's custom-op path was entered rather than before it. What it does not
//! cost is correctness — no fallback is taken at that point either way, and this
//! wrapper foreclosed one before entering.
//!
//! # Autograd is refused, not silently dropped
//!
//! The op is applied through `apply_op1_no_bwd`, which records no backprop node
//! at all, and [`TilerPlan::apply`] refuses a tracked tensor before reaching it.
//! Both halves are needed: the first makes "this result carries no gradient" a
//! structural fact rather than a convention, and the second stops a tracked
//! input from silently losing its graph. `CustomOp1::bwd` is left at its default,
//! which errors.
//!
//! [`RuntimeAdapter`]: tiler_runtime::adapter::RuntimeAdapter
//! [`LiveExecutionContext`]: tiler_runtime::adapter::LiveExecutionContext

use std::cell::RefCell;
use std::fmt;

use candle_core::backend::BackendStorage;
use candle_core::{
    CpuStorage, CustomOp1, DType, Device, Layout, MetalStorage, Shape, Storage, Tensor,
};

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, AvailabilityPhase, BindingTarget, RecordedArtifactProgramIdentity,
};
use tiler_ir::semantic::F32;
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment, LoadRejection};

use crate::adapter::{
    CandleMetalAdapter, DeviceFacts, INPUT_KEY, OUTPUT_KEY, bind_candle_storage,
    bound_accessible_extent, declared_transport_slots, load_library,
    prepare_pipeline_with_reflection,
};
use crate::refusal::{
    Delivered, DeliveredPath, DispatchFailure, FallbackAvailability, Realization, RouteRefusal,
    TensorRefusal, fallback_availability,
};

/// Minimum Metal allocation Candle's `buf_size` supplies for a zero-byte request.
///
/// `0_usize.next_power_of_two()` in candle-core 0.11.0. The helper asks
/// `MetalDevice::new_buffer` for a zero logical element count so that rounding
/// stays inside Candle's allocator rather than a Tiler-private length.
pub(crate) const EMPTY_INPUT_SENTINEL_BYTES: u64 = 1;

/// The one delivery position every artifact here is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and these artifacts are built for a single target, so the sole position
/// is zero. Named rather than written as a bare `0` at each call, because the
/// argument decides *which compiled object* is loaded and a literal there says
/// nothing about why that one.
const SOLE_DELIVERY: usize = 0;

/// The operations a delivered realization claim covers, and only those.
///
/// Named as a constant because `docs/integration/candle.md` makes the scope
/// obligatory wherever a realization is reported: the consumer sees one tensor
/// whose value composes several numerical contracts, and this names the part
/// Tiler states.
const COVERED_OPERATIONS: &str = "the fused reduction this artifact packages, from its declared input to its declared output; \
     not the Candle kernels that produced the input or consume the output";

/// Why one wrapper call did not deliver a result.
#[derive(Debug)]
pub enum WrapperError {
    /// The artifact could not be decoded or did not route.
    Load(LoadRejection),
    /// The Tensor-level preflight declined and no fallback was available.
    Tensor(TensorRefusal),
    /// The route refused or the committed dispatch failed.
    Route(Box<AdapterRouteFailure<RouteRefusal, DispatchFailure>>),
    /// Candle itself refused.
    Candle(String),
}

impl fmt::Display for WrapperError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(rejection) => write!(formatter, "{rejection}"),
            Self::Tensor(refusal) => write!(formatter, "{refusal}"),
            Self::Route(failure) => write!(formatter, "{failure}"),
            Self::Candle(detail) => write!(formatter, "candle: {detail}"),
        }
    }
}

impl std::error::Error for WrapperError {}

impl WrapperError {
    /// Returns whether ADR 0051 would still permit a fallback for this outcome.
    ///
    /// Reported rather than acted on. This wrapper takes no fallback after
    /// entering the custom op — see the module note — so the answer is
    /// diagnostic: it separates "this artifact does not fit this call" from "the
    /// submission did not complete", which are different things to report.
    pub fn fallback_would_be_permitted(&self) -> bool {
        match self {
            Self::Load(_) | Self::Tensor(_) | Self::Candle(_) => true,
            Self::Route(failure) => failure.fallback_permitted(),
        }
    }
}

/// What one completed route observed on the way through.
#[derive(Clone, Debug)]
pub struct RouteReport {
    /// What the bound device reported about itself.
    pub facts: DeviceFacts,
    /// The governed profile key the route ran under.
    pub profile_key: String,
    /// The route's declared entries.
    pub entries: usize,
    /// How many of them were encoded; the rest declared their dispatch skippable.
    pub encoded: usize,
    /// Shared allocations the loader paired for this route.
    pub shared_allocations: usize,
}

/// One constructed zero-element input, with the allocation evidence that is not
/// the accessible range.
///
/// This is construction evidence, not a semantic input enum: the caller still
/// applies [`EmptyInputTensor::tensor`] as an ordinary Candle `Tensor`.
#[derive(Debug)]
pub struct EmptyInputTensor {
    /// The untracked contiguous Metal tensor of logical element count zero.
    pub tensor: Tensor,
    /// Bytes Candle allocated for the sentinel. At least [`EMPTY_INPUT_SENTINEL_BYTES`].
    pub allocation_bytes: u64,
    /// Artifact-derived accessible extent. Always zero for an admitted construction.
    pub accessible_bytes: u64,
}

/// One delivered result, with the realization it was produced under.
#[derive(Debug)]
pub struct Applied {
    /// The result tensor.
    pub tensor: Tensor,
    /// Which path produced it and under which numerical realization.
    pub delivered: Delivered,
    /// What the route observed, when the artifact path ran.
    pub report: Option<RouteReport>,
}

/// One real object's argument table beside the declaration it is compared against.
///
/// Both sides are carried because the *agreement* is the measurement worth
/// reporting: it is the evidence that Metal's reflection on this toolchain row
/// enumerates exactly the `[[buffer(N)]]` parameters the emitter declared, which
/// no amount of unit-testing the comparison establishes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArgumentSlotProbe {
    /// The entry symbol whose pipeline was reflected.
    pub symbol: String,
    /// The transport slots the artifact declares, ascending.
    pub declared: Vec<u64>,
    /// The buffer argument indices the compiled object addresses, ascending.
    pub addressed: Vec<u64>,
}

/// One artifact, decoded far enough to preflight a tensor against it.
///
/// Holds the envelope bytes rather than a decoded program, and that is not a
/// convenience. `DecodedProgram` is deliberately not `Clone` and yields exactly
/// one routing authority, so a plan reused across calls has to decode afresh
/// each time; keeping a decoded program here and handing out `&mut` references
/// to it would be the state ADR 0051's one-authority-per-attempt property exists
/// to make unreachable.
#[derive(Debug)]
pub struct TilerPlan {
    bytes: Vec<u8>,
    recorded: RecordedArtifactProgramIdentity,
    environment: ExecutionEnvironment,
    rows: u64,
    columns: u64,
    facts: AbiFacts,
    realization: Realization,
}

impl TilerPlan {
    /// Reads one artifact's interface and proves this host can offer its profile.
    ///
    /// Everything here is device-free and happens once per artifact. The routing
    /// authority this decode mints is dropped rather than carried: it exists to
    /// read the declared interface and to classify the declared profile, and a
    /// plan that held it would be holding an authority for an attempt that has
    /// not happened.
    ///
    /// # Errors
    ///
    /// Returns [`WrapperError::Load`] for bytes that are not this artifact, and
    /// [`WrapperError::Tensor`] carrying
    /// [`TensorRefusal::IncompatibleTargetProfile`],
    /// [`TensorRefusal::ForeignInterface`], or
    /// [`TensorRefusal::ZeroExtentInterface`] for an empty *output* this first
    /// pass does not construct.
    pub fn load(
        bytes: Vec<u8>,
        recorded: RecordedArtifactProgramIdentity,
        environment: ExecutionEnvironment,
        realization: Realization,
    ) -> Result<Self, WrapperError> {
        let decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(WrapperError::Load)?;
        if decoded.identity().as_bytes() != recorded.as_bytes() {
            return Err(WrapperError::Load(LoadRejection::ProgramMismatch {
                expected: recorded,
                loaded: decoded.identity(),
            }));
        }
        let (rows, columns, facts) = bind_interface(&decoded, &environment)?;
        drop(decoded);
        Ok(Self {
            bytes,
            recorded,
            environment,
            rows,
            columns,
            facts,
            realization,
        })
    }

    /// Returns the input shape the artifact declares, as rows by columns.
    ///
    /// The column extent may be zero: that is the empty-domain input
    /// [`Self::empty_input_tensor`] constructs. [`Self::load`] still refuses an
    /// empty output with [`TensorRefusal::ZeroExtentInterface`].
    pub const fn declared_shape(&self) -> (u64, u64) {
        (self.rows, self.columns)
    }

    /// Returns whether the artifact's declared input has no elements.
    pub const fn declares_empty_input(&self) -> bool {
        self.rows == 0 || self.columns == 0
    }

    /// Constructs the artifact's declared zero-element input as a real Candle tensor.
    ///
    /// Allocates the minimum Candle-managed sentinel on `device` *before* any
    /// routing attempt, records logical element count zero, and returns an
    /// untracked contiguous `Tensor` the caller then passes through
    /// [`Self::preflight`] and [`Self::apply`]. Never a default for an omitted
    /// nonempty input.
    ///
    /// # Errors
    ///
    /// Returns [`WrapperError::Tensor`] when the declaration is nonempty, the
    /// output is empty, the device or dtype is wrong, the sentinel is shorter
    /// than [`EMPTY_INPUT_SENTINEL_BYTES`], storage and shape disagree, the
    /// tensor is tracked, or any `ProgramInput` binding's artifact-derived
    /// accessible extent is nonzero. Returns [`WrapperError::Load`] when the
    /// artifact cannot be prepared far enough to read that extent.
    pub fn empty_input_tensor(&self, device: &Device) -> Result<EmptyInputTensor, WrapperError> {
        empty_input_declaration(self.rows, self.columns).map_err(WrapperError::Tensor)?;
        let accessible_bytes = self.empty_input_accessible_bytes()?;
        empty_input_window_is_unread(accessible_bytes).map_err(WrapperError::Tensor)?;
        let tensor = allocate_empty_input_tensor(device, self.rows, self.columns)
            .map_err(WrapperError::Tensor)?;
        admit_constructed_empty_input(&tensor, device, self.rows, self.columns)
            .map_err(WrapperError::Tensor)?;
        let allocation_bytes = metal_allocation_bytes(&tensor).map_err(WrapperError::Tensor)?;
        if bound_accessible_extent(accessible_bytes, allocation_bytes) != accessible_bytes {
            return Err(WrapperError::Tensor(
                TensorRefusal::NonzeroVerifiedInputWindow { accessible_bytes },
            ));
        }
        Ok(EmptyInputTensor {
            tensor,
            allocation_bytes,
            accessible_bytes,
        })
    }

    /// Evaluates every `ProgramInput` binding's artifact-derived accessible extent.
    ///
    /// The routing authority this prepare mints is dropped: the helper only
    /// needs the evaluated window so it can refuse a kernel that would read the
    /// sentinel.
    fn empty_input_accessible_bytes(&self) -> Result<u64, WrapperError> {
        let mut program =
            DecodedProgram::decode(&self.bytes, SOLE_DELIVERY).map_err(WrapperError::Load)?;
        let qualification = program
            .prepare(&self.environment, &self.recorded, &self.facts)
            .map_err(WrapperError::Load)?;
        let mut window = 0_u64;
        for entry in qualification.entries() {
            for binding in entry.bindings() {
                if matches!(binding.binding().target(), BindingTarget::ProgramInput(_)) {
                    empty_input_window_is_unread(binding.accessible_bytes())
                        .map_err(WrapperError::Tensor)?;
                    window = binding.accessible_bytes();
                }
            }
        }
        drop(qualification);
        Ok(window)
    }

    /// Returns the realization a delivered result is claimed under.
    pub const fn realization(&self) -> Realization {
        self.realization
    }

    /// Decides every Tensor-visible question, before Candle's custom-op path.
    ///
    /// The complete first-profile boundary: a device this adapter binds, the one
    /// dtype its artifacts declare, a contiguous non-aliasing view, the rank and
    /// extents the artifact declares, and no tracked autograd. Each refusal
    /// names what was unsupported; none copies, relayouts, or approximates.
    ///
    /// # Errors
    ///
    /// Returns the [`TensorRefusal`] naming the first unsupported fact.
    pub fn preflight(&self, input: &Tensor, device: &Device) -> Result<(), TensorRefusal> {
        // Autograd first, because it is the one refusal that is about what the
        // caller will do with the result rather than about what this adapter can
        // bind: a tracked tensor is refused even when every other fact fits.
        if input.track_op() {
            return Err(TensorRefusal::AutogradTracked);
        }

        let Device::Metal(bound) = device else {
            return Err(TensorRefusal::NotAMetalDevice {
                observed: format!("{:?}", device.location()),
            });
        };
        match input.device() {
            Device::Metal(theirs) => {
                if theirs.id() != bound.id() {
                    return Err(TensorRefusal::ForeignMetalDevice {
                        tensor: format!("{:?}", theirs.id()),
                        adapter: format!("{:?}", bound.id()),
                    });
                }
            }
            other => {
                return Err(TensorRefusal::NotAMetalDevice {
                    observed: format!("{:?}", other.location()),
                });
            }
        }

        if input.dtype() != DType::F32 {
            return Err(TensorRefusal::UnsupportedDtype {
                observed: input.dtype(),
                supported: DType::F32,
            });
        }

        let layout = input.layout();
        let dims = layout.dims();
        let stride = layout.stride();
        // Broadcast before affine-stride, because a zero stride is an aliasing
        // fact rather than an indexing one and a later affine-stride variant
        // would not subsume it.
        if dims
            .iter()
            .zip(stride)
            .any(|(extent, step)| *step == 0 && *extent > 1)
        {
            return Err(TensorRefusal::BroadcastView {
                dims: dims.to_vec(),
                stride: stride.to_vec(),
            });
        }
        if !layout.is_contiguous() {
            return Err(TensorRefusal::AffineStridedLayout {
                dims: dims.to_vec(),
                stride: stride.to_vec(),
            });
        }

        if dims.len() != 2 {
            return Err(TensorRefusal::UnsupportedRank {
                observed: dims.len(),
                required: 2,
            });
        }
        for (axis, declared) in [self.rows, self.columns].into_iter().enumerate() {
            let observed = dims[axis];
            if u64::try_from(observed).unwrap_or(u64::MAX) != declared {
                return Err(TensorRefusal::ExtentMismatch {
                    axis,
                    declared,
                    observed,
                });
            }
        }
        Ok(())
    }

    /// Asks this artifact's first routed entry's library for a named symbol.
    ///
    /// Exists so `crate::proof` can watch the entry-symbol refusal fail against
    /// a real published object, through the same [`load_library`] the route
    /// takes rather than a second loader written to resemble it. The routing
    /// authority it mints is abandoned rather than committed, which is exactly
    /// the fallback ADR 0051 permits.
    ///
    /// # Errors
    ///
    /// Returns [`WrapperError::Load`] when the artifact does not route, and
    /// [`WrapperError::Route`] carrying the payload or symbol refusal.
    pub fn probe_entry_symbol(
        &self,
        device: &candle_core::MetalDevice,
        symbol: &str,
    ) -> Result<(), WrapperError> {
        let mut program =
            DecodedProgram::decode(&self.bytes, SOLE_DELIVERY).map_err(WrapperError::Load)?;
        let qualification = program
            .prepare(&self.environment, &self.recorded, &self.facts)
            .map_err(WrapperError::Load)?;
        let entry = qualification
            .entries()
            .first()
            .ok_or_else(|| WrapperError::Candle("this route has no entries".to_owned()))?;
        let outcome = load_library(device, 0, entry.object()).and_then(|library| {
            library
                .get_function(symbol, None)
                .map(|_| ())
                .map_err(|cause| RouteRefusal::EntrySymbolAbsent {
                    entry: 0,
                    symbol: symbol.to_owned(),
                    detail: cause.to_string(),
                })
        });
        drop(qualification);
        outcome.map_err(|refusal| {
            WrapperError::Route(Box::new(AdapterRouteFailure::Payload { entry: 0, refusal }))
        })
    }

    /// Reads this artifact's first routed entry's declared and addressed argument tables.
    ///
    /// Both sides as the route itself obtains them: the declaration through
    /// [`declared_transport_slots`] over the loader's own routed bindings, and
    /// the addressed table through [`prepare_pipeline_with_reflection`] over a
    /// pipeline built from the real published object — the same two functions
    /// [`CandleMetalAdapter`]'s
    /// [`prepare_entries`](tiler_runtime::adapter::RuntimeAdapter::prepare_entries)
    /// compares.
    ///
    /// Exists so `crate::proof` can perturb one side of a *real* comparison and
    /// watch the refusal arrive. Perturbing the artifact's own bytes is not an
    /// available alternative: the envelope proves an integrity digest over them,
    /// so an edited transport mapping is refused as a damaged envelope long
    /// before any argument table is read — which is a different check passing,
    /// not this one.
    ///
    /// The routing authority it mints is abandoned rather than committed, which
    /// is exactly the fallback ADR 0051 permits.
    ///
    /// # Errors
    ///
    /// Returns [`WrapperError::Load`] when the artifact does not route, and
    /// [`WrapperError::Route`] carrying the payload, symbol, or pipeline refusal.
    pub fn probe_argument_slots(
        &self,
        device: &candle_core::MetalDevice,
    ) -> Result<ArgumentSlotProbe, WrapperError> {
        let mut program =
            DecodedProgram::decode(&self.bytes, SOLE_DELIVERY).map_err(WrapperError::Load)?;
        let qualification = program
            .prepare(&self.environment, &self.recorded, &self.facts)
            .map_err(WrapperError::Load)?;
        let entry = qualification
            .entries()
            .first()
            .ok_or_else(|| WrapperError::Candle("this route has no entries".to_owned()))?;
        let symbol = entry.entry_symbol().to_owned();
        let declared = declared_transport_slots(entry);
        let outcome = load_library(device, 0, entry.object()).and_then(|library| {
            let function = library.get_function(&symbol, None).map_err(|cause| {
                RouteRefusal::EntrySymbolAbsent {
                    entry: 0,
                    symbol: symbol.clone(),
                    detail: cause.to_string(),
                }
            })?;
            prepare_pipeline_with_reflection(device, 0, &symbol, &function)
        });
        drop(qualification);
        let prepared = outcome.map_err(|refusal| {
            WrapperError::Route(Box::new(AdapterRouteFailure::Preparation(refusal)))
        })?;
        Ok(ArgumentSlotProbe {
            symbol,
            declared,
            addressed: prepared.addressed_slots,
        })
    }

    /// Runs the artifact as a Candle custom op, or fails closed.
    ///
    /// The fallback decision is taken here and only here. A refused preflight
    /// selects the ordinary Candle expression **only** when that expression
    /// realizes the requested numerical contract; when it does not — which is
    /// the case for every order-fixing contract — the refusal is reported with
    /// the unmet realization named, rather than the faster, differently rounded
    /// expression being run silently.
    ///
    /// # Errors
    ///
    /// Returns [`WrapperError::Tensor`] for a refused preflight with no
    /// realizable fallback, and [`WrapperError::Route`] for anything the route
    /// or the committed dispatch reported.
    pub fn apply(&self, input: &Tensor, device: &Device) -> Result<Applied, WrapperError> {
        if let Err(refused) = self.preflight(input, device) {
            return match fallback_availability(self.realization) {
                FallbackAvailability::Available => {
                    let tensor = candle_expression(input).map_err(WrapperError::Candle)?;
                    Ok(Applied {
                        tensor,
                        delivered: Delivered {
                            path: DeliveredPath::CandleExpression,
                            realization: Realization::CandleBuiltinF32,
                            covered_operations: COVERED_OPERATIONS,
                        },
                        report: None,
                    })
                }
                FallbackAvailability::Unavailable {
                    requested,
                    candle_delivers,
                } => Err(WrapperError::Tensor(TensorRefusal::NoRealizableFallback {
                    refused: Box::new(refused),
                    requested,
                    candle_delivers,
                })),
            };
        }

        let operation = TilerFusedOp {
            plan: self,
            outcome: RefCell::new(None),
        };
        // `apply_op1_no_bwd` rather than `apply_op1`: it records no backprop node
        // at all, which is what makes "this fused forward op provides no
        // gradient" structural rather than a promise. It also takes `&C`, so
        // this operation may borrow the plan and carry a cell the call writes
        // its report into.
        let applied = input.apply_op1_no_bwd(&operation);
        let outcome = operation.outcome.into_inner();
        match (applied, outcome) {
            (Ok(tensor), Some(Ok(report))) => Ok(Applied {
                tensor,
                delivered: Delivered {
                    path: DeliveredPath::TilerArtifact,
                    realization: self.realization,
                    covered_operations: COVERED_OPERATIONS,
                },
                report: Some(report),
            }),
            // The typed outcome is authoritative when there is one: Candle can
            // only carry a string out of `metal_fwd`, and losing the
            // classification would leave a caller unable to tell a route refusal
            // from a dispatch failure.
            (_, Some(Err(error))) => Err(error),
            (Err(cause), _) => Err(WrapperError::Candle(cause.to_string())),
            (Ok(_), None) => Err(WrapperError::Candle(
                "Candle returned a tensor without entering this operation's Metal path".to_owned(),
            )),
        }
    }
}

/// Evaluates the ordinary Candle expression this artifact fuses.
///
/// Retained even though this program's realization forbids selecting it, for two
/// reasons. It is the fallback machinery the contract requires the wrapper to
/// own, and it is the labelled counter-example that makes the numerical-scope
/// warning concrete: `crate::proof` runs it beside the artifact path and reports
/// where the two disagree, which is the three-independent-axes fact rather than
/// a defect in either.
///
/// # Errors
///
/// Returns Candle's own account of why the expression did not evaluate.
pub fn candle_expression(input: &Tensor) -> Result<Tensor, String> {
    // The same declarative program the artifact packages: sum((x * 1.0) + 0.0)
    // over the reduced axis. Written out rather than shortened to `sum`, so the
    // two paths are compared on the same expression and not on two different
    // ones.
    (|| {
        let scaled = (input * 1.0)?;
        let shifted = (scaled + 0.0)?;
        shifted.sum(1)
    })()
    .map_err(|cause: candle_core::Error| cause.to_string())
}

/// The Candle custom op that carries one Tiler artifact.
struct TilerFusedOp<'a> {
    plan: &'a TilerPlan,
    /// The typed outcome of the route, written by [`Self::metal_fwd`].
    ///
    /// Candle's custom-op boundary carries only a `candle_core::Error` out, so
    /// the classification would be flattened to a string without this. The cell
    /// is written exactly once per call and read once by the caller.
    outcome: RefCell<Option<Result<RouteReport, WrapperError>>>,
}

impl CustomOp1 for TilerFusedOp<'_> {
    fn name(&self) -> &'static str {
        "tiler.fused-artifact"
    }

    /// Refuses rather than falling back to a host evaluation.
    ///
    /// Unreachable through [`TilerPlan::apply`], which refuses a non-Metal
    /// tensor before applying the op. Retained because the trait requires it and
    /// because the honest implementation is a refusal: a CPU evaluation here
    /// would be a *third* numerical realization, delivered under a claim that
    /// names the artifact's.
    fn cpu_fwd(
        &self,
        _storage: &CpuStorage,
        _layout: &Layout,
    ) -> candle_core::Result<(CpuStorage, Shape)> {
        Err(candle_core::Error::Msg(
            "candle.custom-op.cpu: this operation delivers a Metal artifact and has no host \
             evaluation; a CPU path would deliver a realization the artifact's claim does not \
             cover"
                .to_owned(),
        ))
    }

    fn metal_fwd(
        &self,
        storage: &MetalStorage,
        layout: &Layout,
    ) -> candle_core::Result<(MetalStorage, Shape)> {
        match self.route(storage, layout) {
            Ok((completion, report)) => {
                *self.outcome.borrow_mut() = Some(Ok(report));
                Ok(completion)
            }
            Err(error) => {
                // Rendered before the value is stored, because Candle's boundary
                // carries only a string out and the typed value is what the
                // caller reads back from the cell.
                let rendered = error.to_string();
                *self.outcome.borrow_mut() = Some(Err(error));
                Err(candle_core::Error::Msg(rendered))
            }
        }
    }
}

impl TilerFusedOp<'_> {
    /// Routes the artifact through the adapter and returns what it produced.
    #[allow(
        clippy::type_complexity,
        reason = "the pair is returned once, to the one caller above, and naming it would add a type whose only purpose is this signature"
    )]
    fn route(
        &self,
        storage: &MetalStorage,
        layout: &Layout,
    ) -> Result<((MetalStorage, Shape), RouteReport), WrapperError> {
        let device = storage.device().clone();
        let input = bind_candle_storage(storage, layout.start_offset())
            .map_err(|refusal| WrapperError::Route(Box::new(AdapterRouteFailure::Plan(refusal))))?;

        // Decoded afresh for this attempt. The plan holds bytes rather than a
        // program precisely so that each application mints its own single-use
        // routing authority.
        let mut program =
            DecodedProgram::decode(&self.plan.bytes, SOLE_DELIVERY).map_err(WrapperError::Load)?;
        let output_elements = usize::try_from(self.plan.rows).unwrap_or(usize::MAX);
        let mut adapter = CandleMetalAdapter::new(
            &device,
            self.plan.environment.clone(),
            input,
            output_elements,
        )
        .map_err(|refusal| WrapperError::Route(Box::new(AdapterRouteFailure::Context(refusal))))?;

        let completion = route_with_adapter(
            &mut program,
            &mut adapter,
            &self.plan.recorded,
            &self.plan.facts,
        )
        .map_err(|failure| WrapperError::Route(Box::new(failure)))?;

        let report = RouteReport {
            facts: adapter.facts().clone(),
            profile_key: completion.profile_key.clone(),
            entries: completion.entries,
            encoded: completion.encoded,
            shared_allocations: adapter.shared_allocations(),
        };
        Ok(((completion.storage, completion.shape), report))
    }
}

/// Reads the shape the artifact declares and proves it is this wrapper's interface.
///
/// The declared shape is *read* rather than asserted equal to a constant this
/// crate holds. What a consumer may take from an artifact is what the artifact
/// says; substituting an expectation here would make the two agree because they
/// were told to.
fn bind_interface(
    decoded: &DecodedProgram,
    environment: &ExecutionEnvironment,
) -> Result<(u64, u64, AbiFacts), WrapperError> {
    let foreign = |detail: String| WrapperError::Tensor(TensorRefusal::ForeignInterface { detail });
    let f32_type = F32::resolved_type().canonical_encoding();

    let inputs: Vec<_> = decoded.inputs().collect();
    let [input] = inputs.as_slice() else {
        return Err(foreign(format!(
            "the artifact declares {} input(s) and this wrapper binds 1",
            inputs.len(),
        )));
    };
    let [rows, columns] = input.shape().extents() else {
        return Err(foreign(format!(
            "the artifact's input is {} and this wrapper binds a rank-2 input",
            input.shape(),
        )));
    };
    if input.key().as_str() != INPUT_KEY || input.resolved_type_encoding() != f32_type.as_bytes() {
        return Err(foreign(format!(
            "the artifact's input is {:?} of logical type {:02x?}, and this wrapper binds \
             {INPUT_KEY:?} of canonical F32",
            input.key().as_str(),
            input.resolved_type_encoding(),
        )));
    }

    let outputs: Vec<_> = decoded.outputs().collect();
    let [output] = outputs.as_slice() else {
        return Err(foreign(format!(
            "the artifact declares {} output(s) and this wrapper reads 1",
            outputs.len(),
        )));
    };
    let published: u64 = output
        .shape()
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product();
    if output.key().as_str() != OUTPUT_KEY
        || output.resolved_type_encoding() != f32_type.as_bytes()
        || published != rows.get()
    {
        return Err(foreign(format!(
            "the artifact's output is {:?} of {} and logical type {:02x?}, and this wrapper reads \
             {} F32 element(s) under {OUTPUT_KEY:?}",
            output.key().as_str(),
            output.shape(),
            output.resolved_type_encoding(),
            rows.get(),
        )));
    }

    // First pass is input-only. An empty published result is named from the
    // output's own extents; an empty *input* is constructed later by
    // `empty_input_tensor` rather than refused here.
    if published == 0 {
        let extents: Vec<u64> = output
            .shape()
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect();
        declared_extents_are_nonzero(output.key().as_str(), &extents)
            .map_err(WrapperError::Tensor)?;
    }

    // Target availability, decided here so a host that cannot offer the declared
    // profile never reaches Candle's custom-op path. This is the wrapper's own,
    // weaker question — "could any packaged payload run here at all" — and not a
    // substitute for the loader's: `DecodedProgram::prepare` classifies the
    // *selected* variant and the payload realizing its every entry, which is the
    // comparison that decides the route. Refusing here on the first incompatible
    // payload would report an artifact as unusable when a later one is this
    // host's own.
    //
    // An artifact carrying no payload at all is not refused here: it has nothing
    // to classify, and the loader's `ObjectNotCarried` is the authority on that.
    let mut first_refusal = None;
    let mut any_compatible = false;
    for payload in decoded.payloads() {
        let observed = environment.classify(&payload.compatibility);
        if observed.is_compatible() {
            any_compatible = true;
            break;
        }
        first_refusal.get_or_insert(observed);
    }
    if !any_compatible && let Some(classification) = first_refusal {
        return Err(WrapperError::Tensor(
            TensorRefusal::IncompatibleTargetProfile { classification },
        ));
    }

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(input.key(), input.shape())
        .map_err(|cause| foreign(format!("the declared input shape does not bind: {cause}")))?;
    Ok((rows.get(), columns.get(), binder.build()))
}

/// Refuses a declared value whose shape has an axis of extent zero.
///
/// Split from the artifact read for the reason [`crate::adapter::binding_fits`]
/// is split from the device call: the decision is then one the repository gate
/// can watch say no, which a comparison written inline against a decoded
/// artifact is not.
///
/// The first empty axis is named rather than all of them. Used for empty
/// *outputs*; empty inputs go through [`empty_input_declaration`] instead.
///
/// # Errors
///
/// Returns [`TensorRefusal::ZeroExtentInterface`] naming the first empty axis.
fn declared_extents_are_nonzero(value: &str, extents: &[u64]) -> Result<(), TensorRefusal> {
    let Some(axis) = extents.iter().position(|extent| *extent == 0) else {
        return Ok(());
    };
    Err(TensorRefusal::ZeroExtentInterface {
        value: value.to_owned(),
        axis,
        extents: extents.to_vec(),
    })
}

/// Admits a zero-element input declaration whose output would still be nonempty.
///
/// # Errors
///
/// Returns [`TensorRefusal::ZeroExtentInterface`] when the row extent is empty
/// (the first pass does not construct empty outputs) and
/// [`TensorRefusal::EmptyConstructionOnNonemptyDeclaration`] when both extents
/// are nonzero.
pub(crate) fn empty_input_declaration(rows: u64, columns: u64) -> Result<(), TensorRefusal> {
    if rows == 0 {
        return Err(TensorRefusal::ZeroExtentInterface {
            value: OUTPUT_KEY.to_owned(),
            axis: 0,
            extents: vec![rows, columns],
        });
    }
    if columns == 0 {
        return Ok(());
    }
    Err(TensorRefusal::EmptyConstructionOnNonemptyDeclaration { rows, columns })
}

/// Refuses a constructed storage whose logical element count is not zero.
///
/// # Errors
///
/// Returns [`TensorRefusal::EmptyInputCount`].
pub(crate) fn empty_input_count_is_zero(count: usize) -> Result<(), TensorRefusal> {
    if count == 0 {
        return Ok(());
    }
    Err(TensorRefusal::EmptyInputCount { observed: count })
}

/// Refuses a constructed shape that is not the artifact's declared empty input.
///
/// # Errors
///
/// Returns [`TensorRefusal::UnsupportedRank`] or [`TensorRefusal::ExtentMismatch`].
pub(crate) fn empty_input_shape_matches(
    observed: &[usize],
    rows: u64,
    columns: u64,
) -> Result<(), TensorRefusal> {
    if observed.len() != 2 {
        return Err(TensorRefusal::UnsupportedRank {
            observed: observed.len(),
            required: 2,
        });
    }
    for (axis, declared) in [rows, columns].into_iter().enumerate() {
        let seen = observed[axis];
        if u64::try_from(seen).unwrap_or(u64::MAX) != declared {
            return Err(TensorRefusal::ExtentMismatch {
                axis,
                declared,
                observed: seen,
            });
        }
    }
    Ok(())
}

/// Refuses any dtype other than the F32 this profile constructs.
///
/// # Errors
///
/// Returns [`TensorRefusal::UnsupportedDtype`].
pub(crate) fn empty_input_dtype_is_f32(dtype: DType) -> Result<(), TensorRefusal> {
    if dtype == DType::F32 {
        return Ok(());
    }
    Err(TensorRefusal::UnsupportedDtype {
        observed: dtype,
        supported: DType::F32,
    })
}

/// Refuses a tracked or variable tensor. Construction is untracked.
///
/// # Errors
///
/// Returns [`TensorRefusal::AutogradTracked`].
pub(crate) fn empty_input_is_untracked(tracked: bool) -> Result<(), TensorRefusal> {
    if tracked {
        return Err(TensorRefusal::AutogradTracked);
    }
    Ok(())
}

/// Refuses a nonzero artifact-derived accessible extent.
///
/// The physical allocation is not an argument: a placeholder must never widen
/// the bound range merely because Candle rounded the sentinel up.
///
/// # Errors
///
/// Returns [`TensorRefusal::NonzeroVerifiedInputWindow`].
pub(crate) fn empty_input_window_is_unread(accessible_bytes: u64) -> Result<(), TensorRefusal> {
    if accessible_bytes == 0 {
        return Ok(());
    }
    Err(TensorRefusal::NonzeroVerifiedInputWindow { accessible_bytes })
}

/// Refuses a sentinel shorter than [`EMPTY_INPUT_SENTINEL_BYTES`].
///
/// # Errors
///
/// Returns [`TensorRefusal::EmptyInputSentinel`].
pub(crate) fn sentinel_allocation_holds(held: u64) -> Result<(), TensorRefusal> {
    if held >= EMPTY_INPUT_SENTINEL_BYTES {
        return Ok(());
    }
    Err(TensorRefusal::EmptyInputSentinel {
        held,
        required: EMPTY_INPUT_SENTINEL_BYTES,
        detail: None,
    })
}

/// Allocates a zero-element F32 tensor through Candle's public storage APIs.
fn allocate_empty_input_tensor(
    device: &Device,
    rows: u64,
    columns: u64,
) -> Result<Tensor, TensorRefusal> {
    let Device::Metal(metal) = device else {
        return Err(TensorRefusal::NotAMetalDevice {
            observed: format!("{:?}", device.location()),
        });
    };
    let buffer = metal
        .new_buffer(0, DType::F32, "tiler.empty-input")
        .map_err(|cause| TensorRefusal::EmptyInputSentinel {
            held: 0,
            required: EMPTY_INPUT_SENTINEL_BYTES,
            detail: Some(cause.to_string()),
        })?;
    let held = u64::try_from(buffer.length()).unwrap_or(0);
    sentinel_allocation_holds(held)?;
    empty_input_count_is_zero(0)?;
    empty_input_dtype_is_f32(DType::F32)?;
    let rows_usize = usize::try_from(rows).unwrap_or(usize::MAX);
    let columns_usize = usize::try_from(columns).unwrap_or(usize::MAX);
    empty_input_shape_matches(&[rows_usize, columns_usize], rows, columns)?;
    let shape = Shape::from_dims(&[rows_usize, columns_usize]);
    empty_input_count_is_zero(shape.elem_count())?;
    Ok(Tensor::from((
        Storage::Metal(MetalStorage::new(buffer, metal.clone(), 0, DType::F32)),
        shape,
    )))
}

/// Checks the constructed tensor against the accepted empty-input boundary.
fn admit_constructed_empty_input(
    tensor: &Tensor,
    device: &Device,
    rows: u64,
    columns: u64,
) -> Result<(), TensorRefusal> {
    empty_input_is_untracked(tensor.track_op())?;
    empty_input_count_is_zero(tensor.elem_count())?;
    empty_input_shape_matches(tensor.dims(), rows, columns)?;
    empty_input_dtype_is_f32(tensor.dtype())?;
    match (tensor.device(), device) {
        (Device::Metal(theirs), Device::Metal(bound)) if theirs.id() == bound.id() => {}
        (Device::Metal(theirs), Device::Metal(bound)) => {
            return Err(TensorRefusal::ForeignMetalDevice {
                tensor: format!("{:?}", theirs.id()),
                adapter: format!("{:?}", bound.id()),
            });
        }
        (other, _) => {
            return Err(TensorRefusal::NotAMetalDevice {
                observed: format!("{:?}", other.location()),
            });
        }
    }
    let (storage, layout) = tensor.storage_and_layout();
    if !layout.is_contiguous() || layout.start_offset() != 0 {
        return Err(TensorRefusal::AffineStridedLayout {
            dims: layout.dims().to_vec(),
            stride: layout.stride().to_vec(),
        });
    }
    let Storage::Metal(metal) = &*storage else {
        return Err(TensorRefusal::NotAMetalDevice {
            observed: format!("{:?}", tensor.device().location()),
        });
    };
    empty_input_dtype_is_f32(metal.dtype())?;
    let held = u64::try_from(metal.buffer().length()).unwrap_or(0);
    sentinel_allocation_holds(held)?;
    Ok(())
}

/// Reads the Candle allocation length of a constructed Metal tensor.
fn metal_allocation_bytes(tensor: &Tensor) -> Result<u64, TensorRefusal> {
    let (storage, _) = tensor.storage_and_layout();
    let Storage::Metal(metal) = &*storage else {
        return Err(TensorRefusal::NotAMetalDevice {
            observed: format!("{:?}", tensor.device().location()),
        });
    };
    Ok(u64::try_from(metal.buffer().length()).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::{
        EMPTY_INPUT_SENTINEL_BYTES, declared_extents_are_nonzero, empty_input_count_is_zero,
        empty_input_declaration, empty_input_dtype_is_f32, empty_input_is_untracked,
        empty_input_shape_matches, empty_input_window_is_unread, sentinel_allocation_holds,
    };
    use crate::adapter::bound_accessible_extent;
    use crate::refusal::TensorRefusal;
    use candle_core::DType;

    /// A declared shape is admitted, or the first empty axis is named.
    ///
    /// The admitted cases lead, because a check that refused every shape would
    /// take every route with it — including the members `crate::proof` carries
    /// onto hardware — and would still pass a test built only from empty axes.
    /// The refusing population puts the zero at each axis in turn, so a check
    /// written against one position is visible here. This function now names
    /// empty *outputs*; empty inputs are constructed rather than refused.
    #[test]
    fn a_declared_shape_is_admitted_or_names_its_first_empty_axis() {
        assert!(declared_extents_are_nonzero("input", &[1, 3]).is_ok());
        assert!(declared_extents_are_nonzero("input", &[1, 1]).is_ok());
        // No axes is no empty axis. Unreachable from `bind_interface`, which
        // refuses anything but a rank-2 declaration before asking, and stated
        // here so the function's own boundary is not left to that caller.
        assert!(declared_extents_are_nonzero("input", &[]).is_ok());

        for (extents, empty) in [(vec![1, 0], 1_usize), (vec![0, 3], 0), (vec![0, 0], 0)] {
            let Err(refusal) = declared_extents_are_nonzero("input", &extents) else {
                panic!("{extents:?} has an empty axis");
            };
            assert!(
                matches!(
                    &refusal,
                    TensorRefusal::ZeroExtentInterface {
                        value,
                        axis,
                        extents: named,
                    } if value == "input" && *axis == empty && *named == extents,
                ),
                "{refusal} does not name axis {empty} of {extents:?}",
            );
        }
    }

    /// Each empty-input subject is refused independently, with its own wording.
    ///
    /// The admitted neighbour leads so a function that refused everything would
    /// still fail. Count, shape, dtype, device-independent lifetime, accessible
    /// extent, and sentinel length are each perturbed alone.
    #[test]
    fn empty_input_construction_refuses_each_subject_independently() {
        assert!(empty_input_declaration(1, 0).is_ok());
        let nonempty = empty_input_declaration(1, 3).expect_err("1x3 is nonempty");
        assert_eq!(
            nonempty.to_string(),
            "candle.preflight.empty-input.nonempty-declaration: the empty-input helper \
             constructs a zero-element tensor and the artifact declares 1x3",
        );
        let empty_output = empty_input_declaration(0, 3).expect_err("0 rows is an empty output");
        assert_eq!(
            empty_output.to_string(),
            "candle.preflight.zero-extent: the artifact declares \"result\" with extents [0, 3], \
             whose axis 0 is empty, and this first pass constructs empty inputs only",
        );

        assert!(empty_input_count_is_zero(0).is_ok());
        let count = empty_input_count_is_zero(1).expect_err("count 1 is not empty");
        assert_eq!(
            count.to_string(),
            "candle.preflight.empty-input.count: constructed MetalStorage count is 1 and must be 0",
        );

        assert!(empty_input_shape_matches(&[1, 0], 1, 0).is_ok());
        let rank = empty_input_shape_matches(&[0], 1, 0).expect_err("rank 1 is not rank 2");
        assert_eq!(
            rank.to_string(),
            "candle.preflight.rank: the artifact's declared input is rank 2 and the tensor is \
             rank 1",
        );
        let shape = empty_input_shape_matches(&[2, 0], 1, 0).expect_err("2x0 is not 1x0");
        assert_eq!(
            shape.to_string(),
            "candle.preflight.extent: the artifact declares 1 along axis 0 and the tensor \
             carries 2",
        );

        assert!(empty_input_dtype_is_f32(DType::F32).is_ok());
        let dtype = empty_input_dtype_is_f32(DType::F16).expect_err("f16 is not f32");
        assert_eq!(
            dtype.to_string(),
            "candle.preflight.dtype: this profile's artifacts declare f32 and the tensor \
             carries f16",
        );

        assert!(empty_input_is_untracked(false).is_ok());
        let lifetime = empty_input_is_untracked(true).expect_err("tracked is refused");
        assert_eq!(
            lifetime.to_string(),
            "candle.preflight.autograd: the tensor participates in tracked autograd and this \
             fused forward op carries no backward formula",
        );

        assert!(empty_input_window_is_unread(0).is_ok());
        let window = empty_input_window_is_unread(4).expect_err("4 bytes is a read window");
        assert_eq!(
            window.to_string(),
            "candle.preflight.empty-input.window: the artifact-derived accessible extent is 4 \
             byte(s) and an empty input may bind only a zero window",
        );

        assert!(sentinel_allocation_holds(EMPTY_INPUT_SENTINEL_BYTES).is_ok());
        let sentinel =
            sentinel_allocation_holds(0).expect_err("zero bytes is shorter than the sentinel");
        assert_eq!(
            sentinel.to_string(),
            "candle.preflight.empty-input.sentinel: Candle allocated 0 byte(s) and the \
             sentinel requires 1",
        );

        assert_eq!(bound_accessible_extent(0, 1), 0);
        assert_eq!(bound_accessible_extent(0, 16), 0);
        assert_ne!(
            bound_accessible_extent(0, 1),
            1,
            "a one-byte sentinel must not become a one-byte accessible range",
        );
    }
}
