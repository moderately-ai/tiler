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
    /// The emitted barrier count differs from the scheduled requirement.
    BarrierCount {
        /// Barriers the kernel emits.
        emitted: u32,
        /// Barriers the derived requirements admit.
        required: u32,
    },
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
            Self::BarrierCount { .. } => "barrier-count",
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
