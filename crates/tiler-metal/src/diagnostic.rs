//! Typed fail-closed diagnostics for structured-kernel-to-MSL translation.
//!
//! Emission never produces best-effort source. When a governed structured
//! construct has no realization in the selected Metal profile, translation
//! returns a [`MetalEmitError`](crate::diagnostic::MetalEmitError) naming the
//! rejected entity and a stable rule identifier an explain record can surface.
//!
//! Every rejection here is a *backend* gap, an emission-legality limit, or a
//! malformed input, never a numerical or scheduling decision. The kernel
//! verifier owns refinement, the schedule owns reduction order, and the target
//! profile owns feasibility.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use core::fmt;
use std::error::Error;

use tiler_ir::kernel::{
    AddressSpace, BarrierOrdering, BufferAccess, ExecutionScope, KernelType, MemoryScope,
    VerifiedKernelHandleError,
};

use crate::record::MetalNumericalGap;
use crate::target::MetalUnstatedSubnormalArithmetic;

/// The governed operation family whose member has no Metal realization.
///
/// Each family in the structured kernel IR is a bounded vocabulary that will
/// grow. A widened family reaches this backend as an unrecognized member and is
/// rejected here rather than silently emitted as something else.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalOperationFamily {
    /// A governed launch builtin.
    Builtin,
    /// A typed immediate constant.
    Constant,
    /// A pure binary operation.
    Binary,
    /// A predicate-producing comparison.
    Compare,
    /// A named typed conversion.
    Convert,
}

impl MetalOperationFamily {
    /// Returns the stable rule suffix naming this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::Constant => "constant",
            Self::Binary => "binary",
            Self::Compare => "compare",
            Self::Convert => "convert",
        }
    }
}

impl fmt::Display for MetalOperationFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a governed synchronization point has no Metal barrier realization.
///
/// The structured kernel IR names execution scope, memory scope, fenced address
/// spaces, and ordering separately. Metal's barrier builtins couple execution
/// scope to the visibility they establish, so a portable specification can be
/// unrealizable even though each of its parts is individually meaningful.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum BarrierRejection {
    /// No Metal barrier builtin realizes this execution scope.
    ExecutionScope {
        /// The rejected execution scope.
        scope: ExecutionScope,
    },
    /// No Metal barrier establishes this visibility for this execution scope.
    ///
    /// `threadgroup_barrier` establishes workgroup visibility and
    /// `simdgroup_barrier` establishes SIMD-group visibility. Metal provides no
    /// in-kernel barrier that establishes device-wide visibility, and a
    /// SIMD-group barrier cannot be widened to a workgroup claim.
    MemoryVisibility {
        /// The requested execution scope.
        execution: ExecutionScope,
        /// The requested memory visibility scope.
        memory: MemoryScope,
    },
    /// Metal has no memory-fence flag for this governed address space.
    FencedSpace {
        /// The rejected address space.
        space: AddressSpace,
    },
    /// No Metal barrier establishes this ordering.
    Ordering {
        /// The rejected ordering.
        ordering: BarrierOrdering,
    },
}

impl BarrierRejection {
    /// Returns the stable rule suffix naming this rejection.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExecutionScope { .. } => "execution-scope",
            Self::MemoryVisibility { .. } => "memory-visibility",
            Self::FencedSpace { .. } => "fenced-space",
            Self::Ordering { .. } => "ordering",
        }
    }
}

impl fmt::Display for BarrierRejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fail-closed rejection from structured-kernel-to-MSL translation.
///
/// Distinct failure kinds are distinct variants carrying the rejected entity,
/// never a preformatted message. [`MetalEmitError::rule`] returns the stable
/// identifier an explain record surfaces.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MetalEmitError {
    /// A governed structured value type has no Metal realization.
    UnsupportedType {
        /// The rejected value type.
        value_type: KernelType,
    },
    /// A governed address space cannot be a Metal kernel-parameter space.
    ///
    /// Metal exposes `device` and `constant` buffer parameters through the
    /// `[[buffer(N)]]` argument table. Workgroup storage is a separate
    /// `[[threadgroup(N)]]` binding namespace that a buffer parameter cannot
    /// name, and invocation-private storage is not a parameter space at all.
    UnsupportedAddressSpace {
        /// The rejected address space.
        space: AddressSpace,
    },
    /// The buffer's access mode contradicts its Metal address space.
    UnsupportedBufferAccess {
        /// The declared address space.
        space: AddressSpace,
        /// The rejected access mode.
        access: BufferAccess,
    },
    /// A member of a governed operation family has no Metal realization.
    UnsupportedOperation {
        /// The family whose member was rejected.
        family: MetalOperationFamily,
    },
    /// The kernel body contains a structured operation this backend does not
    /// recognize.
    ///
    /// The structured operation vocabulary is `#[non_exhaustive]`, so a widened
    /// IR reaches an already-compiled backend as an unrecognized operation.
    /// Emission stops rather than skipping it.
    UnrecognizedOperation,
    /// A governed synchronization point has no Metal barrier realization.
    UnsupportedBarrier {
        /// Why no barrier realizes the specification.
        reason: BarrierRejection,
    },
    /// The kernel's canonical arithmetic NaN pattern is not a NaN encoding.
    ///
    /// A NaN-canonicalizing conversion whose target is a finite or infinite
    /// value is not a canonicalization; emitting it would produce a kernel that
    /// compiles and computes the wrong thing.
    InvalidCanonicalNan {
        /// The rejected bit pattern.
        bits: u32,
    },
    /// The target cannot realize a declared numerical obligation at all.
    ///
    /// This is a hard feasibility rejection, not a cost: no compiler flag
    /// selection honours the obligation on this target. It is returned by
    /// [`MetalTranslationUnit::require_declared_realization`](crate::record::MetalTranslationUnit::require_declared_realization),
    /// the step at which a caller claims conformance, rather than by emission,
    /// because whether the limit is observable depends on the values a dispatch
    /// actually sees.
    UnrealizableNumericalObligation {
        /// The obligation no compiler selection realizes.
        gap: MetalNumericalGap,
    },
    /// The unit performs arithmetic in a type the target states no subnormal
    /// fact for.
    ///
    /// This is `Unknown`, not a gap: a gap says the target cannot honour the
    /// obligation, and this says nothing is known about whether it can. The two
    /// are kept apart because the measured behaviours disagree by arithmetic
    /// type — Apple `f32` flushes and Apple `f16` preserves — so reading a
    /// neighbouring type's fact would be a guess, and defaulting to either
    /// behaviour would either refuse a correct plan or approve an incorrect
    /// one. It is returned by
    /// [`MetalTranslationUnit::require_declared_realization`](crate::record::MetalTranslationUnit::require_declared_realization)
    /// ahead of any gap, because a gap set computed while a fact is missing is
    /// incomplete and an empty one is therefore not a conformance claim.
    UnstatedSubnormalArithmetic {
        /// The arithmetic type no subnormal fact was stated for.
        unstated: MetalUnstatedSubnormalArithmetic,
    },
    /// The kernel signature needs more buffer bindings than the target admits.
    BufferBindingLimit {
        /// Bindings the signature requires.
        required: usize,
        /// Bindings the target profile admits.
        limit: u32,
    },
    /// The structured body violated a local well-formedness rule emission
    /// depends on.
    ///
    /// The kernel verifier proves each of these, so reaching one means the
    /// verified product and this backend disagree. Translation stops instead of
    /// emitting a partially bound body.
    MalformedKernel {
        /// Stable identifier of the violated rule.
        rule: &'static str,
    },
    /// An operand named a structured value with no emitted definition.
    UnresolvedValue,
    /// Two structurally distinct kernels derived the same entry-point symbol.
    ///
    /// Entry-point symbols are a bounded digest of canonical kernel identity.
    /// The digest is presentation-only; distinctness is proven before any
    /// source is returned rather than assumed.
    SymbolCollision {
        /// The colliding symbol.
        symbol: String,
    },
    /// A verified handle did not resolve against the kernel that minted it.
    Handle(VerifiedKernelHandleError),
}

impl MetalEmitError {
    /// Returns the stable rule identifier for this rejection.
    #[must_use]
    pub const fn rule(&self) -> &'static str {
        match self {
            Self::UnsupportedType { .. } => "unsupported-type",
            Self::UnsupportedAddressSpace { .. } => "unsupported-address-space",
            Self::UnsupportedBufferAccess { .. } => "unsupported-buffer-access",
            Self::UnsupportedOperation { .. } => "unsupported-operation",
            Self::UnrecognizedOperation => "unrecognized-operation",
            Self::UnsupportedBarrier { .. } => "unsupported-barrier",
            Self::InvalidCanonicalNan { .. } => "invalid-canonical-nan",
            Self::UnrealizableNumericalObligation { .. } => "unrealizable-numerical-obligation",
            // Delegated rather than repeated: the reason type owns the
            // identifier, so the two cannot drift apart.
            Self::UnstatedSubnormalArithmetic { unstated } => unstated.rule(),
            Self::BufferBindingLimit { .. } => "buffer-binding-limit",
            Self::MalformedKernel { .. } => "malformed-kernel",
            Self::UnresolvedValue => "unresolved-value",
            Self::SymbolCollision { .. } => "symbol-collision",
            Self::Handle(_) => "kernel-handle",
        }
    }
}

impl fmt::Display for MetalEmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedType { value_type } => {
                write!(f, "{}: {value_type:?}", self.rule())
            }
            Self::UnsupportedAddressSpace { space } => {
                write!(f, "{}: {space:?}", self.rule())
            }
            Self::UnsupportedBufferAccess { space, access } => {
                write!(f, "{}: {access:?} in {space:?}", self.rule())
            }
            Self::UnsupportedOperation { family } => write!(f, "{}: {family}", self.rule()),
            Self::UnsupportedBarrier { reason } => write!(f, "{}: {reason}", self.rule()),
            Self::InvalidCanonicalNan { bits } => write!(f, "{}: {bits:#010x}", self.rule()),
            Self::UnrealizableNumericalObligation { gap } => write!(f, "{}: {gap}", self.rule()),
            // The reason type already renders as `rule: type`.
            Self::UnstatedSubnormalArithmetic { unstated } => write!(f, "{unstated}"),
            Self::BufferBindingLimit { required, limit } => {
                write!(f, "{}: {required} of {limit}", self.rule())
            }
            Self::MalformedKernel { rule } => write!(f, "{}: {rule}", self.rule()),
            Self::SymbolCollision { symbol } => write!(f, "{}: {symbol}", self.rule()),
            Self::UnrecognizedOperation | Self::UnresolvedValue | Self::Handle(_) => {
                f.write_str(self.rule())
            }
        }
    }
}

impl Error for MetalEmitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Handle(source) => Some(source),
            _ => None,
        }
    }
}

impl From<VerifiedKernelHandleError> for MetalEmitError {
    fn from(value: VerifiedKernelHandleError) -> Self {
        Self::Handle(value)
    }
}
