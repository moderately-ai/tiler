//! Whether this backend realizes the synchronization a routed entry requires.
//!
//! A *derived* requirement, exactly as
//! [`crate::direct_requirement`]'s index arithmetic is: the routed entry's
//! [`ResourceRequirements`](tiler_ir::schedule::ResourceRequirements) record is
//! the single authority, no artifact row carries it, and no producer can restate
//! it. `crates/tiler-artifact/src/program/requirement.rs` admits a backend-feature
//! row only for something the verified program does **not** already state, and
//! `ResourceRequirements::synchronization` is derived by
//! `cooperative_synchronization_requirement` from the visibility edges a
//! cooperative tile carries. A row restating it would be a second,
//! independently editable statement about one KIR fact.
//!
//! # Feasibility, and not a cost input
//!
//! The answer here is a hard feasibility predicate. A subject this backend cannot
//! deliver is not a slower plan; it is a plan whose barrier could never have been
//! written, so the staged handoff it exists to order would not happen and the
//! reader of a staged value would see whatever was in workgroup memory. Nothing
//! in this module ranks, scores, or prices anything, and no caller may treat a
//! refusal as an expensive alternative.
//!
//! # Why this needs no device, unlike its index-arithmetic sibling
//!
//! [`crate::direct_requirement`] takes a normalized *observation* because an
//! Apple GPU family is a property of the bound device. A synchronization
//! realization is not. Metal's barrier builtins and their coupled visibility are
//! fixed by the language, no `MTLDevice` property varies them, and
//! `tiler-compiler`'s own feasibility authority records the same thing where it
//! explains why such a fact can never be deferred to a runtime query: no query
//! vocabulary can ask a device "do you order a workgroup-scoped acquire-release
//! fence over threadgroup memory".
//!
//! So this takes the required subject alone. A caller must not read that as a
//! weaker check — it is a *stronger* one, because there is no observation that
//! could have been missing, and the two evidence classes are kept in separate
//! modules so no reader concludes that changing the device could change this
//! answer.
//!
//! # One authority, not a table beside emission
//!
//! What Metal realizes is decided by `barrier_realization`, the same function
//! `barrier_call` emits from. This module's own work is the *inversion*: the
//! neutral schedule vocabulary is deliberately wider than the kernel spelling
//! vocabulary, so a required subject may name something no `BarrierSpec` can
//! spell at all. That is refused here, by name, before emission's authority is
//! consulted — and it is a different refusal from one emission declines, because
//! the repairs differ: an unspellable subject is a program this backend has no
//! construct for, while a rejected spelling is a construct Metal declines to
//! couple that way.
//!
//! # Draft boundary
//!
//! Every public item here is a reviewed *draft* boundary under ADR 0074 §7 and
//! ADR 0075, prepared under
//! `tickets/check-synchronization-realization-before-the-routing-commit.md`. Its
//! exact surface returns to Tom for acceptance before it is treated as accepted.

use core::fmt;
use std::error::Error;

use tiler_ir::kernel::{AddressSpace, BarrierOrdering, BarrierSpec, ExecutionScope, MemoryScope};
use tiler_ir::schedule::{
    MemoryOrdering, SyncPointId, SynchronizationKind, SynchronizationScope, SynchronizationSubject,
};

use crate::diagnostic::BarrierRejection;
use crate::emit::barrier_realization;

/// Why this backend does not realize the synchronization an entry requires.
///
/// One variant per repair. An unadmitted kind, an unspellable scope, and an
/// unspellable ordering are three different gaps between the neutral schedule
/// vocabulary and this backend's kernel spelling, and [`Self::Unrealizable`] is
/// the case where the spelling exists and Metal declines it. Collapsing them
/// would tell a reader to go and change the wrong thing.
///
/// Every variant carries the **whole** required subject rather than the single
/// dimension that failed. The subject is atomic — its five dimensions are matched
/// as one value, because each is separately true of some realization and their
/// conjunction is not a statement about any of them — so a refusal naming one
/// dimension would invite a reader to conclude the other four were satisfied.
/// The offending dimension is carried *beside* the subject, not instead of it.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: this is a classification a
/// caller consumes to decide what to do next, and a later gap must land
/// additively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum MetalSynchronizationRefusal {
    /// The subject names an operation kind this backend has no construct for.
    ///
    /// Only [`SynchronizationKind::ControlBarrier`] has a Metal barrier spelling.
    /// The neutral vocabulary names five further kinds — an asynchronous copy, a
    /// split-phase barrier, a collective, an atomic, and an inter-dispatch
    /// dependency — each with a different contract over participants, visibility,
    /// and failure. None is lowered as a barrier, which is the substitution this
    /// refusal exists to make impossible at delivery time as well as at emission.
    UnadmittedKind {
        /// The complete realization the routed entry requires.
        required: SynchronizationSubject,
        /// The kind that has no spelling here.
        kind: SynchronizationKind,
    },
    /// No kernel execution scope spells the arrival this subject requires.
    ///
    /// [`ExecutionScope`] names a subgroup and a workgroup. A subject requiring
    /// every invocation of the *dispatch* to arrive names a construct this
    /// backend's kernel vocabulary cannot even state, so it is refused here
    /// rather than reaching emission's own scope arm.
    UnspellableExecutionScope {
        /// The complete realization the routed entry requires.
        required: SynchronizationSubject,
        /// The arrival scope that has no spelling here.
        scope: SynchronizationScope,
    },
    /// No kernel memory scope spells the publication this subject requires.
    ///
    /// [`MemoryScope`] names workgroup and device visibility. A subject
    /// publishing to a *subgroup* names a visibility the governed memory scopes
    /// cannot express — the same gap emission records when it says a SIMD-group
    /// barrier has no admissible scope — so it is a vocabulary gap rather than a
    /// Metal limitation, and is told apart from one.
    UnspellableVisibilityScope {
        /// The complete realization the routed entry requires.
        required: SynchronizationSubject,
        /// The publication scope that has no spelling here.
        scope: SynchronizationScope,
    },
    /// No kernel barrier ordering spells the ordering this subject requires.
    ///
    /// [`BarrierOrdering`] names acquire-release alone. A subject requiring
    /// relaxed ordering, or a single total order over every participant's fenced
    /// effects, names an ordering this backend's kernel vocabulary cannot state.
    UnspellableOrdering {
        /// The complete realization the routed entry requires.
        required: SynchronizationSubject,
        /// The ordering that has no spelling here.
        ordering: MemoryOrdering,
    },
    /// The subject spells a barrier Metal declines to realize.
    ///
    /// The spelling exists and this backend's own emission authority refused it —
    /// most importantly the visibility coupling, because Metal ties visibility to
    /// the builtin and no in-kernel barrier establishes device-wide visibility.
    /// The cause is carried whole rather than flattened, and it is the *same*
    /// value emission would have produced for the same specification.
    Unrealizable {
        /// The complete realization the routed entry requires.
        required: SynchronizationSubject,
        /// The emission authority's own reason for declining the spelling.
        reason: BarrierRejection,
    },
}

impl MetalSynchronizationRefusal {
    /// Returns the stable rule identifier for this refusal.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::UnadmittedKind { .. } => "metal.synchronization.unadmitted-kind",
            Self::UnspellableExecutionScope { .. } => {
                "metal.synchronization.unspellable-execution-scope"
            }
            Self::UnspellableVisibilityScope { .. } => {
                "metal.synchronization.unspellable-visibility-scope"
            }
            Self::UnspellableOrdering { .. } => "metal.synchronization.unspellable-ordering",
            Self::Unrealizable { .. } => "metal.synchronization.unrealizable",
        }
    }

    /// Returns the complete subject the routed entry requires.
    ///
    /// Always the whole value. A caller reporting a refusal names the realization
    /// that was needed, never the one dimension that happened to fail first.
    #[must_use]
    pub const fn required(&self) -> SynchronizationSubject {
        match self {
            Self::UnadmittedKind { required, .. }
            | Self::UnspellableExecutionScope { required, .. }
            | Self::UnspellableVisibilityScope { required, .. }
            | Self::UnspellableOrdering { required, .. }
            | Self::Unrealizable { required, .. } => *required,
        }
    }
}

impl fmt::Display for MetalSynchronizationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let required = self.required();
        write!(
            formatter,
            "{}: this entry requires a {} synchronization arriving {}-wide, publishing {}-wide, \
             fencing{}{}, ordered {} — and ",
            self.rule(),
            required.kind.key(),
            required.execution_scope.key(),
            required.visibility_scope.key(),
            if required.fenced_spaces.workgroup {
                " workgroup"
            } else {
                ""
            },
            if required.fenced_spaces.device {
                " device"
            } else if required.fenced_spaces.is_empty() {
                " nothing"
            } else {
                ""
            },
            required.ordering.key(),
        )?;
        match self {
            Self::UnadmittedKind { kind, .. } => write!(
                formatter,
                "this backend has no construct for a {} at all",
                kind.key(),
            ),
            Self::UnspellableExecutionScope { scope, .. } => write!(
                formatter,
                "this backend's kernel vocabulary cannot state a {}-wide arrival",
                scope.key(),
            ),
            Self::UnspellableVisibilityScope { scope, .. } => write!(
                formatter,
                "this backend's kernel vocabulary cannot state {}-wide publication",
                scope.key(),
            ),
            Self::UnspellableOrdering { ordering, .. } => write!(
                formatter,
                "this backend's kernel vocabulary cannot state {} ordering",
                ordering.key(),
            ),
            Self::Unrealizable { reason, .. } => {
                write!(formatter, "Metal declines that barrier: {reason}")
            }
        }
    }
}

impl Error for MetalSynchronizationRefusal {}

/// Decides whether this backend realizes one entry's required synchronization.
///
/// `None` is the canonical absence a region with no synchronization point
/// derives, and it is `Ok`: such a region emits no requirement at all, so there
/// is nothing for this backend to realize. The [`Option`] is taken rather than
/// unwrapped by the caller so that reading the absence as permission is this
/// authority's decision and not a call site's.
///
/// Deterministic and pure: nothing here reads a device, a process, an
/// environment variable, or an artifact. See this module's documentation for why
/// no device observation is taken.
///
/// # Errors
///
/// Returns the [`MetalSynchronizationRefusal`] naming the whole required subject
/// and the gap that refused it.
///
/// ```
/// use tiler_ir::schedule::{
///     FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationScope,
///     SynchronizationSubject,
/// };
/// use tiler_metal::synchronization_requirement::{
///     MetalSynchronizationRefusal, evaluate_synchronization,
/// };
///
/// // The subject every cooperative tile in this workspace derives: a workgroup
/// // control barrier fencing workgroup memory under acquire-release ordering.
/// let staged = SynchronizationSubject {
///     kind: SynchronizationKind::ControlBarrier,
///     execution_scope: SynchronizationScope::Workgroup,
///     visibility_scope: SynchronizationScope::Workgroup,
///     fenced_spaces: FencedSpaces { workgroup: true, device: false },
///     ordering: MemoryOrdering::AcquireRelease,
/// };
/// assert_eq!(evaluate_synchronization(Some(staged)), Ok(()));
///
/// // A region that stages nothing requires nothing.
/// assert_eq!(evaluate_synchronization(None), Ok(()));
///
/// // Publishing device-wide is the neighbour Metal cannot deliver: no in-kernel
/// // barrier establishes device-wide visibility. The refusal names the whole
/// // subject, not the one dimension that differs.
/// let device_wide = SynchronizationSubject {
///     visibility_scope: SynchronizationScope::Device,
///     ..staged
/// };
/// let refusal = evaluate_synchronization(Some(device_wide)).unwrap_err();
/// assert_eq!(refusal.required(), device_wide);
/// assert_eq!(refusal.rule(), "metal.synchronization.unrealizable");
///
/// // An atomic is refused before any spelling is attempted: this backend has no
/// // construct for it, which is a different repair from a declined barrier.
/// let atomic = SynchronizationSubject { kind: SynchronizationKind::Atomic, ..staged };
/// assert!(matches!(
///     evaluate_synchronization(Some(atomic)),
///     Err(MetalSynchronizationRefusal::UnadmittedKind { .. }),
/// ));
/// ```
pub fn evaluate_synchronization(
    required: Option<SynchronizationSubject>,
) -> Result<(), MetalSynchronizationRefusal> {
    let Some(required) = required else {
        return Ok(());
    };
    let spec = spell(required)?;
    barrier_realization(&spec)
        .map(|_| ())
        .map_err(|reason| MetalSynchronizationRefusal::Unrealizable { required, reason })
}

/// Spells one neutral subject as the kernel barrier that would realize it.
///
/// The inverse of the projection `tiler-ir`'s whole-kernel verifier applies in
/// the other direction, and it is written as exhaustive matches over the
/// *neutral* vocabulary so that widening one is a build error here — the one
/// place that has to decide what a new schedule construct spells as — rather
/// than a wildcard that silently maps it onto whichever spelling it resembles.
///
/// The neutral vocabulary is wider than the kernel one on three axes, and each
/// gap is refused by name rather than rounded to a neighbour. Rounding is the
/// dangerous repair here: a device-wide arrival narrowed to a workgroup one, or
/// a sequentially-consistent ordering weakened to acquire-release, would spell a
/// barrier that emits cleanly and orders less than the schedule proved it needed.
///
/// `pub(crate)` rather than public, for the reason [`crate::direct_requirement`]'s
/// `evaluate_against` is: a caller outside this crate choosing its own spelling
/// would be a second authority over what a subject requires. It is reachable
/// within the crate so a test can assert that an admitted subject spells the
/// barrier whose *emitted text* fences exactly the domains it named — a property
/// no public signature exposes.
pub(crate) fn spell(
    required: SynchronizationSubject,
) -> Result<BarrierSpec, MetalSynchronizationRefusal> {
    match required.kind {
        SynchronizationKind::ControlBarrier => {}
        kind @ (SynchronizationKind::AsynchronousCopy
        | SynchronizationKind::SplitPhaseBarrier
        | SynchronizationKind::Collective
        | SynchronizationKind::Atomic
        | SynchronizationKind::InterDispatchDependency) => {
            return Err(MetalSynchronizationRefusal::UnadmittedKind { required, kind });
        }
    }
    let execution_scope = match required.execution_scope {
        SynchronizationScope::Subgroup => ExecutionScope::Subgroup,
        SynchronizationScope::Workgroup => ExecutionScope::Workgroup,
        scope @ SynchronizationScope::Device => {
            return Err(MetalSynchronizationRefusal::UnspellableExecutionScope { required, scope });
        }
    };
    let memory_scope = match required.visibility_scope {
        SynchronizationScope::Workgroup => MemoryScope::Workgroup,
        SynchronizationScope::Device => MemoryScope::Device,
        scope @ SynchronizationScope::Subgroup => {
            return Err(MetalSynchronizationRefusal::UnspellableVisibilityScope {
                required,
                scope,
            });
        }
    };
    let ordering = match required.ordering {
        MemoryOrdering::AcquireRelease => BarrierOrdering::AcquireRelease,
        ordering @ (MemoryOrdering::Relaxed | MemoryOrdering::SequentiallyConsistent) => {
            return Err(MetalSynchronizationRefusal::UnspellableOrdering { required, ordering });
        }
    };
    // In ascending `AddressSpace` order, which is the order a `BarrierSpec`'s
    // field is declared to be in. The flag set is derived from the two booleans
    // rather than accumulated, so a fence naming neither domain spells an empty
    // list — a barrier that orders no memory — instead of being unrepresentable.
    let mut fenced_spaces = Vec::with_capacity(2);
    if required.fenced_spaces.device {
        fenced_spaces.push(AddressSpace::Device);
    }
    if required.fenced_spaces.workgroup {
        fenced_spaces.push(AddressSpace::Workgroup);
    }
    Ok(BarrierSpec {
        // The point is a verification reference rather than an emission fact:
        // `barrier_realization` reads the scopes, the fences, and the ordering
        // and never the point. There is no point to name here at all — this
        // subject came off a delivered artifact's resource record, which carries
        // one realization for the whole region and no tile-local ordinals — so
        // the first ordinal stands in for a field the decision does not read.
        point: SyncPointId::FIRST,
        execution_scope,
        memory_scope,
        fenced_spaces,
        ordering,
    })
}
