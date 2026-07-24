//! The output record of one deterministic Metal translation unit.
//!
//! A translation unit is the product of a checked emission, so it exposes no
//! `pub` fields and offers no constructor: only
//! [`crate::emit::emit_translation_unit`] produces one. Its readers yield the
//! emitted source, the ordered entry points with their complete binding tables,
//! the numerical compiler selections the source requires, and the declared
//! numerical obligations the target cannot realize at all.
//!
//! The last two are deliberately different kinds. A
//! [`crate::record::MetalNumericalRequirement`] names a compiler selection that
//! *does* deliver an obligation; a [`crate::record::MetalNumericalGap`] says no
//! selection does. Keeping hard feasibility apart from a flag choice is what
//! lets [`crate::record::MetalTranslationUnit::require_declared_realization`]
//! fail closed with an explainable reason instead of naming a flag that would
//! not honour the contract.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use core::fmt;

use tiler_ir::kernel::{BufferParameter, CanonicalKernelIdentity};

use crate::diagnostic::MetalEmitError;

/// One numerical compiler flag this emitted source requires to be correct.
///
/// The Metal backend contract permits a translation-unit-wide flag only when it
/// stays within every affected operation's contract. Requirements are therefore
/// the *union* of what each entry point's numerical realization demands, and
/// each variant names one strictly-stronger-is-safe compiler selection.
///
/// A permission the realization grants is deliberately not a requirement: this
/// set says what the source cannot tolerate, not what a caller must choose.
///
/// A requirement here is an obligation emission could **not** discharge in the
/// generated operations. Everything emission *can* carry — exact `f32`
/// immediates, one arithmetic operation per statement, and an integer-only NaN
/// predicate — is deliberately absent from this set, because it holds under
/// every math mode.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalNumericalRequirement {
    /// `-fmetal-math-mode=safe`.
    ///
    /// Required whenever a realization forbids reduction reassociation, and
    /// whenever the source performs `f32` arithmetic whose signed-zero result
    /// or NaN operand is observable. `relaxed` and `fast` both apply LLVM's
    /// `reassoc`, `nsz`, `arcp`, and `afn` licences to every emitted `f32`
    /// operation, and `fast` adds `nnan` and `ninf`. `nsz` makes signed zero
    /// unreliable; `reassoc` licenses reordering a serial reduction; `nnan`
    /// makes an arithmetic result that is a NaN undefined, so there is no
    /// defined value left for a canonicalization to map.
    ///
    /// No emitted operation can discharge any of those, which is why this is a
    /// requirement rather than something the source carries.
    ///
    /// **Measurement.** On an Apple M4 Max under macOS 27.0 (build 26A5388g)
    /// with Metal 32023.883, the emitted scale-then-bias kernel for
    /// `scale = 1.0`, `bias = +0.0` returns `0x00000000` for the operand
    /// `0x80000000` under `-fmetal-math-mode=safe` and `0x80000000` under both
    /// `relaxed` and `fast`. IEEE-754 round-to-nearest requires `0x00000000`.
    /// The divergence holds at every `-O` level (`0`, `1`, `2`, `3`, `s`) and
    /// is independent of `-fmetal-math-fp32-functions`.
    SafeMathMode,
    /// `-ffp-contract=off`.
    ///
    /// Required whenever a realization forbids contraction. Emission
    /// additionally writes every arithmetic operation as its own statement, so
    /// no contraction can form across two structured operations even under
    /// `-ffp-contract=on`; the flag closes the `fast` case.
    ///
    /// **Measurement.** On the environment above, a multiply and an add in two
    /// separate statements over `scale = 1.5`, `bias = 1.0` return the
    /// separately rounded `0x3fc58f9e` for the operand `0x3eb97ef9` under both
    /// `-ffp-contract=off` and `-ffp-contract=on`, and the fused `0x3fc58f9d`
    /// under `-ffp-contract=fast`. The per-statement emission is therefore a
    /// measured defence against `on` and not against `fast`.
    NoFloatingPointContraction,
}

impl MetalNumericalRequirement {
    /// Returns the exact compiler flag this requirement demands.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        match self {
            Self::SafeMathMode => "-fmetal-math-mode=safe",
            Self::NoFloatingPointContraction => "-ffp-contract=off",
        }
    }

    /// Returns the stable rule identifier for this requirement.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::SafeMathMode => "safe-math-mode",
            Self::NoFloatingPointContraction => "no-floating-point-contraction",
        }
    }
}

impl fmt::Display for MetalNumericalRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.flag())
    }
}

/// One declared numerical obligation this Metal profile cannot realize at all.
///
/// A gap is not a requirement. A requirement names a compiler selection that
/// *does* deliver the obligation; a gap says no selection does, because the
/// limit is a hard target feasibility fact. Recording it separately keeps
/// feasibility distinct from cost and stops emission from naming a flag that
/// would not actually honour the contract.
///
/// Emission reports gaps rather than rejecting, because a gap is observable
/// only when a value that reaches the limit actually occurs, which emission
/// cannot know. [`MetalTranslationUnit::require_declared_realization`] is the
/// fail-closed step: a caller that needs the declared realization exactly must
/// call it, and it rejects with the naming gap.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalNumericalGap {
    /// The realization preserves subnormals, but the target's `f32` arithmetic
    /// flushes subnormal operands and subnormal results to zero.
    ///
    /// No `-fmetal-math-mode`, `-ffp-contract`, `-fmetal-math-fp32-functions`,
    /// or `-O` selection changes this; see
    /// [`MetalSubnormalArithmetic`](crate::target::MetalSubnormalArithmetic)
    /// for the measurement. A kernel that only materializes values is not
    /// affected, so this gap is recorded only when the kernel performs `f32`
    /// arithmetic.
    SubnormalFlushInArithmetic,
    /// The realization flushes subnormals to zero, but the target's `f32`
    /// arithmetic preserves them.
    ///
    /// This is the converse of
    /// [`MetalNumericalGap::SubnormalFlushInArithmetic`] and it is a gap for
    /// the same reason: emission never narrows, widens, or substitutes the
    /// declared contract to fit a target. Honouring a flush on a preserving
    /// target would require emitting an explicit flush, which is emulation and
    /// is not something this backend expresses today.
    SubnormalPreservationInArithmetic,
    /// The realization flushes subnormals to a stated zero, and the target
    /// declares only *that* its `f32` arithmetic flushes, not which zero it
    /// produces.
    ///
    /// The measured Apple flush preserves the sign of the flushed value
    /// (`0x80400000 * 2.0f` returns `0x80000000`, not `0x00000000`), so the two
    /// zeros are observably different results and a target fact naming neither
    /// establishes neither. Recording a gap is the fail-closed reading.
    /// `declare-metal-numerical-honourability` closes it by replacing the
    /// backend-local fact with a per-dimension honourability declaration that
    /// names the zero, after which only a sign *mismatch* remains a gap.
    UndeclaredFlushedZeroSign,
}

impl MetalNumericalGap {
    /// Returns the stable rule identifier for this gap.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::SubnormalFlushInArithmetic => "subnormal-flush-in-arithmetic",
            Self::SubnormalPreservationInArithmetic => "subnormal-preservation-in-arithmetic",
            Self::UndeclaredFlushedZeroSign => "undeclared-flushed-zero-sign",
        }
    }
}

impl fmt::Display for MetalNumericalGap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.rule())
    }
}

/// One buffer parameter of an emitted entry point and the index it binds at.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MetalBufferBinding {
    index: u32,
    parameter: BufferParameter,
}

impl MetalBufferBinding {
    pub(crate) const fn new(index: u32, parameter: BufferParameter) -> Self {
        Self { index, parameter }
    }

    /// Returns the `[[buffer(N)]]` argument-table index this parameter binds at.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Returns the structured buffer parameter this binding realizes.
    #[must_use]
    pub const fn parameter(self) -> BufferParameter {
        self.parameter
    }
}

/// One emitted Metal entry point and its complete binding table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetalEntryPoint {
    symbol: String,
    kernel: CanonicalKernelIdentity,
    buffers: Vec<MetalBufferBinding>,
}

impl MetalEntryPoint {
    pub(crate) const fn new(
        symbol: String,
        kernel: CanonicalKernelIdentity,
        buffers: Vec<MetalBufferBinding>,
    ) -> Self {
        Self {
            symbol,
            kernel,
            buffers,
        }
    }

    /// Returns the emitted MSL function symbol.
    ///
    /// The symbol is a bounded digest of [`Self::kernel_identity`] and is
    /// presentation only. Equality and cache decisions always use the canonical
    /// identity bytes; emission proves the symbols of one translation unit are
    /// pairwise distinct before returning.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    /// Returns the canonical identity of the kernel this entry point realizes.
    #[must_use]
    pub const fn kernel_identity(&self) -> &CanonicalKernelIdentity {
        &self.kernel
    }

    /// Returns the ordered buffer bindings of this entry point.
    #[must_use]
    pub fn buffers(&self) -> &[MetalBufferBinding] {
        &self.buffers
    }
}

/// One deterministic Metal translation unit emitted from verified kernels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetalTranslationUnit {
    source: String,
    entry_points: Vec<MetalEntryPoint>,
    numerical: Vec<MetalNumericalRequirement>,
    gaps: Vec<MetalNumericalGap>,
}

impl MetalTranslationUnit {
    pub(crate) const fn new(
        source: String,
        entry_points: Vec<MetalEntryPoint>,
        numerical: Vec<MetalNumericalRequirement>,
        gaps: Vec<MetalNumericalGap>,
    ) -> Self {
        Self {
            source,
            entry_points,
            numerical,
            gaps,
        }
    }

    /// Returns the complete emitted Metal Shading Language source.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the entry points in ascending canonical-kernel-identity order.
    #[must_use]
    pub fn entry_points(&self) -> &[MetalEntryPoint] {
        &self.entry_points
    }

    /// Returns the numerical compiler requirements, in ascending governed order.
    ///
    /// A caller compiling this source must select a realization that satisfies
    /// every requirement. The set is deliberately not a complete flag list:
    /// `-fmetal-math-fp32-functions` is unconstrained here because this emission
    /// contains no accuracy-mode-dependent library call, and the offline driver
    /// still requires the caller to state it explicitly.
    #[must_use]
    pub fn numerical_requirements(&self) -> &[MetalNumericalRequirement] {
        &self.numerical
    }

    /// Returns the declared numerical obligations this Metal profile cannot
    /// realize, in ascending governed order.
    ///
    /// An empty slice means every declared obligation is either carried by the
    /// emitted operations or reachable through
    /// [`Self::numerical_requirements`]. A non-empty slice means no compiler
    /// selection honours the contract; see [`MetalNumericalGap`].
    #[must_use]
    pub fn numerical_gaps(&self) -> &[MetalNumericalGap] {
        &self.gaps
    }

    /// Fails closed unless this unit realizes every declared numerical
    /// obligation.
    ///
    /// Emitting a translation unit is not a conformance claim: it says the
    /// structured kernels translated, not that the target can honour their
    /// numerical contract. This is the conformance claim, and a caller that
    /// needs the declared realization exactly must make it before compiling.
    ///
    /// # Errors
    ///
    /// Returns [`MetalEmitError::UnrealizableNumericalObligation`] naming the
    /// first gap in ascending governed order.
    pub fn require_declared_realization(&self) -> Result<(), MetalEmitError> {
        match self.gaps.first() {
            Some(gap) => Err(MetalEmitError::UnrealizableNumericalObligation { gap: *gap }),
            None => Ok(()),
        }
    }
}
