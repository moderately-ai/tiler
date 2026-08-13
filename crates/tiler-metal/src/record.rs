//! The output record of one deterministic Metal translation unit.
//!
//! A translation unit is the product of a checked emission, so it exposes no
//! `pub` fields and offers no constructor: only
//! [`crate::emit::emit_translation_unit`] produces one. Its readers yield the
//! emitted source, the ordered entry points with their complete binding tables,
//! the numerical compiler selections the source requires, the declared
//! numerical obligations the target cannot realize at all, and the arithmetic
//! types the unit used that the target states no subnormal fact for.
//!
//! The last three are deliberately different kinds. A
//! [`crate::record::MetalNumericalRequirement`] names a compiler selection that
//! *does* deliver an obligation; a [`crate::record::MetalNumericalGap`] says no
//! selection does; and
//! [`crate::record::MetalTranslationUnit::unstated_subnormal_arithmetic`]
//! says nothing is known either way, which is `Unknown` and not a verdict.
//! Keeping hard feasibility apart from a flag choice, and both apart from an
//! unmeasured fact, is what lets
//! [`crate::record::MetalTranslationUnit::require_declared_realization`]
//! fail closed with an explainable reason instead of naming a flag that would
//! not honour the contract or inheriting a measurement made for another dtype.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use core::fmt;

use tiler_ir::kernel::{BufferParameter, CanonicalKernelIdentity};

use crate::diagnostic::MetalEmitError;
use crate::target::{
    MetalEmissionRealization, MetalFloatArithmeticType, MetalTargetFacts,
    MetalUnstatedSubnormalArithmetic,
};

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
    /// Required whenever a realization forbids ordered reassociation,
    /// contributor permutation, or signed-zero elimination, and whenever NaN
    /// or infinity may occur. A caller-declared but unvalidated absence is
    /// deliberately insufficient. `relaxed` and `fast` both apply LLVM's
    /// `reassoc`, `nsz`, `arcp`, and `afn` licences to every emitted `f32`
    /// operation, and `fast` adds `nnan` and `ninf`. `nsz` makes signed zero
    /// unreliable; `reassoc` licenses regrouping same-operation arithmetic,
    /// including reordering a serial reduction; and `nnan` or `ninf` makes
    /// arithmetic on the corresponding exceptional value undefined, so there
    /// is no defined value left for later operations to preserve or canonicalize.
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
    /// `-fmetal-math-fp32-functions=precise`.
    ///
    /// Required whenever an emitted operation is a single-precision elementary
    /// function whose admitted result set is a resolved accuracy contract stated
    /// against Metal's *precise* table. The `fast` selection is a different
    /// contract, not a faster realization of the same one: Table 8.2 gives `exp`
    /// the input-dependent bound `3 + floor(fabs(2 * x))` where Table 8.1 gives a
    /// constant `4 ulp`, and ADR 0042 routes those to different contract forms.
    ///
    /// **This is defence in depth and not the primary control.** Emission writes
    /// the `precise::` namespace explicitly, which selects `air.exp.f32`
    /// regardless of the math mode, so a build that lost this flag still gets the
    /// precise intrinsic. The requirement exists because the flag is what the
    /// *specification's* applicability clause is stated against, and a target
    /// profile that admitted the fast selection would be admitting Table 8.2.
    ///
    /// **Measurement.** The [Metal transcendental emission
    /// probe](../../../spikes/numerics/metal_transcendental_emission/README.md)
    /// records that under `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise`
    /// both `exp(x)` and `precise::exp(x)` lower to `air.exp.f32`, while
    /// `fast::exp(x)` lowers to `air.fast_exp.f32` and the unqualified spelling
    /// selects the `fast_` family with no flags at all. The default is fast math,
    /// so the hazard is one omitted flag wide.
    PreciseFp32Functions,
}

impl MetalNumericalRequirement {
    /// Returns the exact compiler flag this requirement demands.
    #[must_use]
    pub const fn flag(self) -> &'static str {
        match self {
            Self::SafeMathMode => "-fmetal-math-mode=safe",
            Self::NoFloatingPointContraction => "-ffp-contract=off",
            Self::PreciseFp32Functions => "-fmetal-math-fp32-functions=precise",
        }
    }

    /// Returns the stable rule identifier for this requirement.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::SafeMathMode => "safe-math-mode",
            Self::NoFloatingPointContraction => "no-floating-point-contraction",
            Self::PreciseFp32Functions => "precise-fp32-functions",
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
///
/// # This is not a second authority on what the target honours
///
/// A profile declaration in the compiler is the authority on *what a target
/// honours*, and this backend-local step survives alongside it rather than
/// competing with it. There is exactly one statement of the Metal fact per
/// arithmetic type —
/// [`MetalSubnormalArithmeticFacts`](crate::target::MetalSubnormalArithmeticFacts),
/// whose measurements are recorded on the type — and every arm of
/// `crate::emit::subnormal_gap` is derived from the single value that record
/// holds for the arithmetic type being emitted, so there is no second opinion
/// here that could diverge from the profile's, and no arm reads a fact stated
/// for a different type.
///
/// What this step adds is a different *question*, not a second answer to the
/// same one:
///
/// - A profile declaration is a claim about a **target and a contract**:
///   whether the dimensions the contract names are honoured. It is answerable
///   before emission, which is what lets the compiler reject early.
/// - A gap is a claim about **this translation unit**: whether the operations
///   actually emitted incur a dimension the target does not honour. The
///   comparison is only reached from emitted floating-point arithmetic, and it
///   reads the fact stated for that operation's own arithmetic type, so a
///   kernel that only materializes values conforms on a flushing target —
///   which the measurement supports, because a load-then-store round trip
///   preserves every subnormal bit pattern. Collapsing the two would either
///   refuse that kernel or approve an arithmetic one; both are wrong answers.
///
/// The dependency graph makes keeping it non-optional. `tiler-metal` depends on
/// `tiler-ir` and `tiler-artifact` and deliberately not on `tiler-compiler`, so
/// a compiler-side rejection is not reachable from here and cannot be relied on
/// to have run: [`crate::emit::emit_translation_unit`] is a public entry point
/// that a caller can drive from `tiler-ir` alone. Retiring this step would
/// leave that path emitting source under a contract the target refuses, with no
/// conformance claim anywhere. The two checkpoints are therefore ordered rather
/// than redundant — the profile declaration governs admission, and this governs
/// the unit that admission produced.
///
/// # A gap names the mismatch and not the arithmetic type it arose in
///
/// The set is per translation unit, exactly as it is per region rather than per
/// subnormal dimension, so a unit performing arithmetic in two types that
/// mismatched differently would record both gaps without saying which type
/// produced which. That is a loss of explanation, never a wrong verdict: each
/// gap is derived from the fact stated for its own arithmetic type, and the
/// conformance claim fails on any of them. It is unreachable today, because the
/// structured kernel IR resolves one floating-point element type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalNumericalGap {
    /// The realization preserves subnormals, but the target flushes subnormal
    /// operands and subnormal results to zero in the arithmetic type this unit
    /// used.
    ///
    /// No `-fmetal-math-mode`, `-ffp-contract`, `-fmetal-math-fp32-functions`,
    /// or `-O` selection changes this for the measured `f32` row; see
    /// [`MetalSubnormalArithmeticFacts`](crate::target::MetalSubnormalArithmeticFacts)
    /// for that measurement and for the `f16` row that does *not* flush. A
    /// kernel that only materializes values is not affected, so this gap is
    /// recorded only when the kernel performs floating-point arithmetic, and
    /// then only against the fact stated for that arithmetic's own type.
    SubnormalFlushInArithmetic,
    /// The realization flushes subnormals to zero, but the target preserves
    /// them in the arithmetic type this unit used.
    ///
    /// This is the converse of
    /// [`MetalNumericalGap::SubnormalFlushInArithmetic`] and it is a gap for
    /// the same reason: emission never narrows, widens, or substitutes the
    /// declared contract to fit a target. Honouring a flush on a preserving
    /// target would require emitting an explicit flush, which is emulation and
    /// is not something this backend expresses today.
    SubnormalPreservationInArithmetic,
    /// The realization flushes subnormals to one zero and the target flushes
    /// to the other.
    ///
    /// The two zeros are observably different results, not different
    /// precisions: the measured Apple flush preserves the sign of the flushed
    /// value (`0x80400000 * 2.0f` returns `0x80000000`), so a program that
    /// asked for `AlwaysPositive` would read `0x80000000` where it required
    /// `0x00000000`. Honouring a flush therefore requires the signs to agree,
    /// and a mismatch fails closed rather than being reported as a relaxation.
    FlushedZeroSignMismatch,
}

impl MetalNumericalGap {
    /// Returns the stable rule identifier for this gap.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::SubnormalFlushInArithmetic => "subnormal-flush-in-arithmetic",
            Self::SubnormalPreservationInArithmetic => "subnormal-preservation-in-arithmetic",
            Self::FlushedZeroSignMismatch => "flushed-zero-sign-mismatch",
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
    input_extents: u32,
}

impl MetalEntryPoint {
    pub(crate) const fn new(
        symbol: String,
        kernel: CanonicalKernelIdentity,
        buffers: Vec<MetalBufferBinding>,
        input_extents: u32,
    ) -> Self {
        Self {
            symbol,
            kernel,
            buffers,
            input_extents,
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

    /// Returns how many live input-extent operands follow the buffer table.
    ///
    /// Each occupies `[[buffer(buffer_count + ordinal)]]` under the accepted
    /// Metal `eN` ABI.
    #[must_use]
    pub const fn input_extent_count(&self) -> u32 {
        self.input_extents
    }
}

/// One deterministic Metal translation unit emitted from verified kernels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetalTranslationUnit {
    target: MetalTargetFacts,
    emission: MetalEmissionRealization,
    source: String,
    entry_points: Vec<MetalEntryPoint>,
    numerical: Vec<MetalNumericalRequirement>,
    gaps: Vec<MetalNumericalGap>,
    unstated: Vec<MetalFloatArithmeticType>,
}

impl MetalTranslationUnit {
    pub(crate) const fn new(
        target: MetalTargetFacts,
        emission: MetalEmissionRealization,
        source: String,
        entry_points: Vec<MetalEntryPoint>,
        numerical: Vec<MetalNumericalRequirement>,
        gaps: Vec<MetalNumericalGap>,
        unstated: Vec<MetalFloatArithmeticType>,
    ) -> Self {
        Self {
            target,
            emission,
            source,
            entry_points,
            numerical,
            gaps,
            unstated,
        }
    }

    /// Returns the exact target facts this translation unit was emitted from.
    #[must_use]
    pub const fn target(&self) -> &MetalTargetFacts {
        &self.target
    }

    /// Returns the source-level realization selected for this translation unit.
    ///
    /// This is deliberately separate from [`Self::target`]: selecting a
    /// `uint` launch parameter does not establish 32-bit index arithmetic,
    /// device-address width, or a concrete launch limit.
    #[must_use]
    pub const fn emission_realization(&self) -> MetalEmissionRealization {
        self.emission
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
    /// A non-empty slice means no compiler selection honours the contract; see
    /// [`MetalNumericalGap`].
    ///
    /// An empty slice is **not** on its own a conformance claim. Gaps are only
    /// computed for arithmetic types the target states a fact for, so a unit
    /// whose arithmetic reached an unstated type has an incomplete gap set —
    /// read [`Self::unstated_subnormal_arithmetic`] beside this, or use
    /// [`Self::require_declared_realization`], which checks both in the order
    /// that makes the answer sound.
    #[must_use]
    pub fn numerical_gaps(&self) -> &[MetalNumericalGap] {
        &self.gaps
    }

    /// Returns the arithmetic types this unit used that the target states no
    /// subnormal fact for, in ascending governed order.
    ///
    /// This is the `Unknown` class. It is neither a delivered obligation nor a
    /// gap: nothing here says the target cannot honour the contract, only that
    /// no measurement says whether it can. The measured Apple row flushes in
    /// `f32` and preserves in `f16`, so there is no neighbouring fact to
    /// substitute and no behaviour that is safe to assume, and the fail-closed
    /// answer is to say so rather than to pick one.
    ///
    /// A type appears here only when the unit actually emitted arithmetic in
    /// it. A target that leaves `f16` unstated is fully conformant for a unit
    /// that performs no `f16` arithmetic.
    #[must_use]
    pub fn unstated_subnormal_arithmetic(&self) -> &[MetalFloatArithmeticType] {
        &self.unstated
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
    /// Returns [`MetalEmitError::UnstatedSubnormalArithmetic`] naming the first
    /// unstated arithmetic type, before considering any gap. An unstated fact
    /// makes the gap set *incomplete* rather than merely adding one more
    /// entry to it, so a gap reported from an incomplete comparison would
    /// present a partial answer as a total one; the missing measurement is also
    /// the actionable thing.
    ///
    /// Otherwise returns [`MetalEmitError::UnrealizableNumericalObligation`]
    /// naming the first gap in ascending governed order.
    pub fn require_declared_realization(&self) -> Result<(), MetalEmitError> {
        if let Some(arithmetic_type) = self.unstated.first() {
            return Err(MetalEmitError::UnstatedSubnormalArithmetic {
                unstated: MetalUnstatedSubnormalArithmetic::for_type(*arithmetic_type),
            });
        }
        match self.gaps.first() {
            Some(gap) => Err(MetalEmitError::UnrealizableNumericalObligation { gap: *gap }),
            None => Ok(()),
        }
    }
}
