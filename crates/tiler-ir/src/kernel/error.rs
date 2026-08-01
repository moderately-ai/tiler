//! Typed errors for structured-kernel construction and verification.
//!
//! Three error boundaries mirror the [`crate::index`] and [`crate::schedule`]
//! discipline. Insertion-time [`KernelBuildError`] rejects locally malformed
//! builder input; the consuming [`super::KernelBuilder::build`] returns a
//! recoverable [`KernelVerificationError`] carrying the whole-kernel
//! [`KernelDiagnostic`] set and the intact builder; and
//! [`VerifiedKernelHandleError`] rejects a handle resolved against the wrong
//! verified kernel.

use std::error::Error;
use std::fmt;

use super::KernelBuilder;
use super::model::KernelType;

/// A governed structural resource in the structured-kernel profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum KernelLimitKind {
    /// Buffer-parameter count of one kernel signature.
    Buffers,
    /// Workgroup staging-allocation count of one kernel.
    Staging,
    /// Admitted launch-builtin count of one kernel signature.
    AdmittedBuiltins,
    /// SSA value count of one kernel.
    Values,
    /// Operation count of one kernel.
    Operations,
    /// Structured block count of one kernel.
    Blocks,
    /// Lexical nesting depth of structured blocks.
    BlockDepth,
    /// Loop-carried accumulator count of one structured loop.
    LoopAccumulators,
    /// Canonical identity bytes retained for one kernel.
    IdentityBytes,
}

impl fmt::Display for KernelLimitKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Builder-owned or verified entity category used by typed handle errors.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum KernelEntityKind {
    /// A structured SSA value.
    Value,
    /// A kernel buffer parameter.
    Buffer,
    /// A workgroup staging allocation.
    Staging,
}

impl fmt::Display for KernelEntityKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// A component whose single-assignment kernel slot was set more than once.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum KernelComponent {
    /// The preserved numerical realization.
    NumericalRealization,
    /// The declared resource requirements.
    ResourceRequirements,
}

impl fmt::Display for KernelComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Failure to resolve a verified handle against a verified kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerifiedKernelHandleError {
    /// The handle belongs to another verified kernel.
    ForeignKernel {
        /// Category of rejected handle.
        entity: KernelEntityKind,
    },
    /// The handle index does not identify a retained entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: KernelEntityKind,
    },
}

impl fmt::Display for VerifiedKernelHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for VerifiedKernelHandleError {}

/// Failure during one transactional structured-kernel builder insertion.
///
/// Every variant names a locally decidable well-formedness rule. Whole-kernel
/// refinement of the scheduled region is a separate boundary reported as a
/// [`KernelDiagnostic`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelBuildError {
    /// No fresh builder ownership identity remained.
    BuilderIdentityExhausted,
    /// A builder-owned handle came from another builder.
    ForeignHandle {
        /// Category of rejected handle.
        entity: KernelEntityKind,
    },
    /// A builder-owned handle did not identify a live entity.
    InvalidHandle {
        /// Category of rejected handle.
        entity: KernelEntityKind,
    },
    /// A value defined in a closed nested block was used from an outer block.
    ValueOutOfScope,
    /// An operand, result, or stored value had the wrong structured type.
    TypeMismatch {
        /// Required type.
        expected: KernelType,
        /// Supplied type.
        actual: KernelType,
    },
    /// A load targeted a write-only buffer or a store targeted a read-only one.
    BufferAccessViolation,
    /// A builtin was read without being admitted by the kernel signature.
    UndeclaredBuiltin,
    /// A builtin was admitted more than once.
    DuplicateAdmittedBuiltin,
    /// A single-assignment component was set more than once.
    ComponentAlreadySet {
        /// Component whose slot was already populated.
        component: KernelComponent,
    },
    /// A divisor or modulus operand was not a compile-time constant.
    NonConstantDivisor,
    /// A divisor or modulus constant was zero.
    NonPositiveDivisor,
    /// A structured loop declared an empty or descending iteration range.
    InvalidLoopRange {
        /// Inclusive first induction value.
        start: u64,
        /// Exclusive last induction value.
        end: u64,
    },
    /// A structured loop carried no accumulator state.
    EmptyLoopAccumulators,
    /// A structured loop yielded a different number of values than it carries.
    LoopYieldArity {
        /// Accumulator arity.
        expected: usize,
        /// Yielded arity.
        actual: usize,
    },
    /// One yielded loop value had the wrong accumulator type.
    LoopYieldTypeMismatch {
        /// Ordered accumulator position.
        position: usize,
        /// Accumulator type.
        expected: KernelType,
        /// Yielded type.
        actual: KernelType,
    },
    /// A governed construction resource exceeded its limit.
    StructuralLimit {
        /// Governed resource.
        resource: KernelLimitKind,
        /// Attempted quantity.
        actual: usize,
        /// Maximum admitted quantity.
        limit: usize,
    },
}

impl fmt::Display for KernelBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for KernelBuildError {}

/// One deterministic whole-kernel verification failure.
///
/// Each variant names an intrinsic or schedule-refinement rule proven by
/// [`super::KernelBuilder::build`]. [`KernelDiagnostic::rule`] returns the
/// stable rule identifier a consumer can surface in an explanation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelDiagnostic {
    /// A required kernel component was never supplied to the builder.
    IncompleteKernel {
        /// The missing component.
        component: KernelComponent,
    },
    /// The scheduled region does not carry exactly one read and one write access.
    ScheduleAccessCount,
    /// The declared buffers do not realize the scheduled region's accesses.
    BufferContract,
    /// A declared buffer address space contradicts the derived requirements.
    AddressSpaceContract,
    /// The admitted builtins do not realize the scheduled execution binding.
    BuiltinContract,
    /// The declared numerical realization differs from the scheduled region's.
    NumericalRealization,
    /// The declared resource requirements differ from the derived requirements.
    ResourceRequirements,
    /// An effect is not dominated by the scheduled bounds predicate.
    PredicateDominance,
    /// A load or store carried the wrong schedule bounds witness.
    BoundsEvidence,
    /// A store carried the wrong schedule write-ownership witness.
    OwnershipEvidence,
    /// The kernel does not commit exactly one owning store per invocation.
    OutputCoverage,
    /// The ordered effect sequence does not end with the owning store.
    EffectOrdering,
    /// The kernel contains synchronization that no schedule has authorized.
    ///
    /// Either the region's schedule owns no synchronization point at all — the
    /// pointwise and global-linear case, where a barrier is redundant under an
    /// exact launch and divergent under any other — or the barrier names a point
    /// ordinal the region's cooperative tile does not declare.
    UnexpectedSynchronization,
    /// A barrier's declared spelling is not the schedule point's subject.
    ///
    /// The point states the obligation and the barrier declares how it is spelled
    /// for a backend; this refuses a kernel whose declaration would emit
    /// something other than what the schedule requires and what a target fact was
    /// matched against.
    SynchronizationContract,
    /// A barrier is not reached by every invocation that must arrive at it.
    ///
    /// A control barrier inside a predicated region is reached by a dynamic
    /// subset of the participants, which is undefined execution on every target
    /// rather than an unsupported one. The rule is structural — no predicate may
    /// enclose a barrier — because a predicate that is *provably* uniform today
    /// would stop being one the moment the launch geometry admitted a tail.
    ///
    /// A loop is different, and the same diagnostic covers the loop rule for the
    /// reason the two are one question: a bounded loop's trip count is a literal,
    /// so every invocation reaches the same dynamic instance, and what a loop
    /// level still needs is a *reason*. The only repetition a tile declares is
    /// its round loop, so this also refuses a barrier enclosed by more than one
    /// loop, by a loop that is not at the kernel's top level, or by a loop whose
    /// range is not the round loop's `1..rounds`.
    SynchronizationConvergence,
    /// A declared point is realized a different number of times than its
    /// placement requires.
    ///
    /// Reaching a barrier is not the same as reaching it as often as the tile's
    /// rounds demand. A phase boundary happens once per round and a round
    /// boundary once per *transition* between consecutive rounds, so a tile of
    /// `rounds` rounds requires `rounds` realizations of the first and
    /// `rounds - 1` of the second. A barrier at the kernel's top level realizes
    /// its point once; one inside the round loop realizes it once per iteration.
    ///
    /// This is what makes the peeled round checkable. A fold seeds at its first
    /// contributor, so round zero is emitted ahead of the loop and the phase
    /// boundary is realized once there and `rounds - 1` times inside — and a body
    /// that dropped the peel, or that put the round boundary in it, has the wrong
    /// count rather than a wrong-looking shape.
    SynchronizationRealization,
    /// A staged load or store names a phase and allocation its tile does not
    /// declare, or addresses a slot outside the declared span.
    StagedAccessEvidence,
    /// A staged read is not separated from its producing write by the barrier
    /// that realizes the point discharging their visibility edge.
    ///
    /// The schedule proves a point *exists* for every edge; this proves the body
    /// actually places the barrier between the two effects. Without it a kernel
    /// could carry a correct point, a correct barrier, and still read staged
    /// values before the fence that publishes them.
    UnorderedStagedHandoff,
    /// A staged rewrite is not separated from the reads it destroys by the
    /// barrier realizing the point that discharges their anti-dependency.
    ///
    /// Separate from [`Self::UnorderedStagedHandoff`] for the reason the schedule
    /// layer keeps the two evidence classes apart: an unordered handoff means a
    /// reader observes a value that was never published, and an unordered rewrite
    /// means a writer destroys a value that has not finished being read. The
    /// fixes are opposite — one moves the fence earlier, the other later.
    ///
    /// The condition is *cyclic*, because the rewrite is in the following round:
    /// a barrier at position `b` of the round body separates round `r`'s read at
    /// `c` from round `r + 1`'s write at `w` exactly when `b > c` or `b < w`.
    /// Only the second of those also orders the peeled round's reads against the
    /// loop's first write, which is the extra obligation a peel creates.
    UnorderedStagedRewrite,
    /// The declared staging allocations do not realize the region's tile.
    StagingContract,
    /// The region's cooperative dataflow carries a visibility dependency that
    /// no schedule has authorized a synchronization point for.
    ///
    /// A cooperative tile states that values one participant writes to
    /// workgroup storage are read by others in a later phase, and the kernel
    /// body contains no barrier realizing the point that orders them.
    ///
    /// The schedule proves that a point exists for every edge before a kernel is
    /// opened, so reaching this means the *body* omitted the barrier — a
    /// well-authorized region lowered into a race. Admitting it would deliver
    /// staged reads that observe unordered writes.
    UndischargedVisibility,
    /// The region's cooperative tile carries a cross-round anti-dependency whose
    /// discharging point the body never realizes.
    ///
    /// The anti-dependency counterpart of [`Self::UndischargedVisibility`], and
    /// separate for the same reason the schedule layer separates them: a rewrite
    /// that overtakes an unfinished read destroys a value rather than reading an
    /// unpublished one. A point discharges at most one of the two classes — the
    /// conditions contradict — so a point realized nowhere falls into exactly one
    /// of these two diagnostics.
    UndischargedAntiDependency,
    /// The region's cooperative tile is outside the lowered dataflow profile.
    ///
    /// The tile is well formed and its synchronization authority is complete —
    /// the schedule verifier proved both — but its shape is one this lowering
    /// has no canonical body for: more than one staging allocation, more than
    /// two phases, a staged span other than the one-slot-per-participant write
    /// and whole-set read the bounded profile uses, or a committing participant
    /// other than the first, which `IndexLessThan` cannot select. Refusing is
    /// what keeps "representable" and "lowered" different claims.
    CooperativeLoweringShape,
    /// The structured loops do not realize the scheduled reduction topology.
    ReductionContract,
    /// The reduction contributor domain is malformed.
    ContributorDomain,
    /// The iteration-domain or access element count overflowed `u64`.
    ElementCountOverflow,
    /// The kernel body is not the canonical refinement of its scheduled region.
    BodyRefinement,
    /// The fully encoded canonical identity exceeded its bound.
    IdentityLimit {
        /// Encoded byte count.
        bytes: usize,
        /// Maximum byte count.
        limit: usize,
    },
}

impl KernelDiagnostic {
    /// Returns the stable verification-rule identifier for this diagnostic.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::IncompleteKernel { .. } => "incomplete-kernel",
            Self::ScheduleAccessCount => "schedule-access-count",
            Self::BufferContract => "buffer-contract",
            Self::AddressSpaceContract => "address-space-contract",
            Self::BuiltinContract => "builtin-contract",
            Self::NumericalRealization => "numerical-realization",
            Self::ResourceRequirements => "resource-requirements",
            Self::PredicateDominance => "predicate-dominance",
            Self::BoundsEvidence => "bounds-evidence",
            Self::OwnershipEvidence => "ownership-evidence",
            Self::OutputCoverage => "output-coverage",
            Self::EffectOrdering => "effect-ordering",
            Self::UnexpectedSynchronization => "unexpected-synchronization",
            Self::SynchronizationContract => "synchronization-contract",
            Self::SynchronizationConvergence => "synchronization-convergence",
            Self::SynchronizationRealization => "synchronization-realization",
            Self::StagedAccessEvidence => "staged-access-evidence",
            Self::UnorderedStagedHandoff => "unordered-staged-handoff",
            Self::UnorderedStagedRewrite => "unordered-staged-rewrite",
            Self::StagingContract => "staging-contract",
            Self::UndischargedVisibility => "undischarged-visibility",
            Self::UndischargedAntiDependency => "undischarged-anti-dependency",
            Self::CooperativeLoweringShape => "cooperative-lowering-shape",
            Self::ReductionContract => "reduction-contract",
            Self::ContributorDomain => "contributor-domain",
            Self::ElementCountOverflow => "element-count-overflow",
            Self::BodyRefinement => "body-refinement",
            Self::IdentityLimit { .. } => "identity-limit",
        }
    }
}

impl fmt::Display for KernelDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for KernelDiagnostic {}

/// Recoverable failure from consuming whole-kernel verification.
///
/// Carries the deterministic diagnostics and returns the intact builder through
/// [`KernelVerificationError::into_parts`] so a caller can amend and retry.
#[derive(Debug)]
pub struct KernelVerificationError {
    pub(super) builder: Box<KernelBuilder>,
    pub(super) diagnostics: Vec<KernelDiagnostic>,
}

impl KernelVerificationError {
    /// Returns all deterministic diagnostics in stable order.
    #[must_use]
    pub fn diagnostics(&self) -> &[KernelDiagnostic] {
        &self.diagnostics
    }

    /// Recovers the intact builder and its diagnostics.
    #[must_use]
    pub fn into_parts(self) -> (KernelBuilder, Vec<KernelDiagnostic>) {
        (*self.builder, self.diagnostics)
    }
}

impl fmt::Display for KernelVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "structured-kernel verification failed with {} diagnostic(s)",
            self.diagnostics.len()
        )
    }
}
impl Error for KernelVerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.diagnostics.first().map(|diagnostic| diagnostic as _)
    }
}

/// Failure to lower one verified scheduled region to its canonical kernel.
///
/// The canonical lowering constructs its kernel through the same public
/// [`super::KernelBuilder`] path an external producer uses, so it can fail
/// either while inserting an operation or during whole-kernel verification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KernelLoweringError {
    /// The canonical lowering could not insert a well-formed operation.
    Construction(KernelBuildError),
    /// The canonical lowering produced a kernel the verifier rejected.
    Verification(KernelDiagnostic),
    /// The scheduled region is outside the lowered structured-kernel profile.
    UnsupportedRegion {
        /// Stable identifier of the unsupported structure.
        rule: &'static str,
    },
}

impl KernelLoweringError {
    /// Returns the stable rule identifier for this lowering failure.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::Construction(_) => "kernel-construction",
            Self::Verification(diagnostic) => diagnostic.rule(),
            Self::UnsupportedRegion { rule } => rule,
        }
    }
}

impl fmt::Display for KernelLoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rule())
    }
}
impl Error for KernelLoweringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Construction(source) => Some(source),
            Self::Verification(source) => Some(source),
            Self::UnsupportedRegion { .. } => None,
        }
    }
}

impl From<KernelBuildError> for KernelLoweringError {
    fn from(value: KernelBuildError) -> Self {
        Self::Construction(value)
    }
}

pub(super) fn invalid_handle(entity: KernelEntityKind, foreign: bool) -> KernelBuildError {
    if foreign {
        KernelBuildError::ForeignHandle { entity }
    } else {
        KernelBuildError::InvalidHandle { entity }
    }
}
