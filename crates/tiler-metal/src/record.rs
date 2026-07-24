//! The output record of one deterministic Metal translation unit.
//!
//! A translation unit is the product of a checked emission, so it exposes no
//! `pub` fields and offers no constructor: only
//! [`crate::emit::emit_translation_unit`] produces one. Its readers yield the
//! emitted source, the ordered entry points with their complete binding tables,
//! and the numerical compiler realization the source requires.
//!
//! Every public item here is a reviewed *draft* boundary (ADR 0074 §7).

use core::fmt;

use tiler_ir::kernel::{BufferParameter, CanonicalKernelIdentity};

/// One numerical compiler flag this emitted source requires to be correct.
///
/// The Metal backend contract permits a translation-unit-wide flag only when it
/// stays within every affected operation's contract. Requirements are therefore
/// the *union* of what each entry point's numerical realization demands, and
/// each variant names one strictly-stronger-is-safe compiler selection.
///
/// A permission the realization grants is deliberately not a requirement: this
/// set says what the source cannot tolerate, not what a caller must choose.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum MetalNumericalRequirement {
    /// `-fmetal-math-mode=safe`.
    ///
    /// Required whenever a realization forbids reduction reassociation,
    /// preserves subnormals, or the source contains a NaN-canonicalizing
    /// conversion. A relaxed or fast math mode may reassociate, flush
    /// subnormals, and assume the absence of NaNs, and that last assumption
    /// would let the compiler delete the canonicalization test entirely.
    SafeMathMode,
    /// `-ffp-contract=off`.
    ///
    /// Required whenever a realization forbids contraction. Emission
    /// additionally writes every arithmetic operation as its own statement, so
    /// no contraction can form across two structured operations even under
    /// `-ffp-contract=on`; the flag closes the `fast` case.
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
}

impl MetalTranslationUnit {
    pub(crate) const fn new(
        source: String,
        entry_points: Vec<MetalEntryPoint>,
        numerical: Vec<MetalNumericalRequirement>,
    ) -> Self {
        Self {
            source,
            entry_points,
            numerical,
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
}
