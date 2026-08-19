//! Effective order profile of one governed contraction occurrence.
//!
//! ADR 0014 requires two independent facts before an order transform is legal:
//! an operation-declared algebraic capability and an independently resolved
//! numerical permission. For `tiler::tensor-contraction-f32@1` the first fact
//! is the reduction descriptor's order-freedom maxima and the second is the
//! occurrence's raw [`NumericalRealization`] ceiling. This module is their
//! **only join**: [`crate::semantic::ContractionF32ReductionDescriptor::resolve`]
//! is the sole constructor of [`EffectiveContractionF32Profile`], so neither
//! fact can substitute for the other — a request may withhold a supported
//! freedom and cannot grant an unsupported one.
//!
//! The ceiling is retained byte for byte, including its `profile_key` and all
//! of its dimensions, because the key is the injective encoding of the
//! dimension vector it sits beside (`docs/numerical-semantics.md`, "the
//! contract key is derived, not chosen"): copying the key while forcing fields
//! would manufacture a value that no longer matches its own contract identity.
//! The effective view is derived beside the stored ceiling, never written into
//! it.

use std::error::Error;
use std::fmt;

use crate::semantic::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, ContractionF32OrderFreedom,
    ContractionF32ReductionDescriptor, ContractionF32ResultClass,
};

use super::numerics::NumericalRealization;

/// Why one ceiling cannot resolve against the governed contraction descriptor.
///
/// Exhaustive with exactly one variant. Descriptor malformation is impossible
/// at this boundary because [`ContractionF32ReductionDescriptor::resolve`] is a
/// method on the already decoded opaque descriptor, and an unsupported
/// operation freedom is an effective `false`, not a construction error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectiveContractionF32ProfileError {
    /// The ceiling's canonical arithmetic NaN payload is not the descriptor's.
    CanonicalNanMismatch {
        /// The descriptor's canonical payload.
        expected: u32,
        /// The ceiling's declared payload.
        actual: u32,
    },
}

impl fmt::Display for EffectiveContractionF32ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CanonicalNanMismatch { expected, actual } => write!(
                formatter,
                "ceiling declares canonical arithmetic NaN {actual:#010x}, and the governed contraction installs {expected:#010x}"
            ),
        }
    }
}

impl Error for EffectiveContractionF32ProfileError {}

/// The effective order contract of one governed contraction occurrence.
///
/// Opaque and `Copy`. Constructible only by
/// [`ContractionF32ReductionDescriptor::resolve`]: there is no `new`, no
/// `Default`, no mutable field, and no path from a raw request that bypasses
/// the descriptor. It stores the raw ceiling byte for byte plus the derived
/// result class; [`Self::ceiling`] returns that stored ceiling unchanged, and
/// `ceiling().profile_key` therefore always remains truthful about
/// `ceiling()`'s own fields.
///
/// Schedule retains the raw [`NumericalRealization`] for contract identity;
/// for a contraction occurrence a consumer derives or consumes this carrier
/// before legality or reference decisions rather than treating the raw ceiling
/// alone as operation authority. The future per-occurrence restriction ADR
/// 0011 mentions does not exist at this base and is not defaulted here;
/// admitting it later must change the resolver signature and identities
/// explicitly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectiveContractionF32Profile {
    ceiling: NumericalRealization,
    result_class: ContractionF32ResultClass,
}

impl EffectiveContractionF32Profile {
    /// Returns the complete retained ceiling, unchanged.
    #[must_use]
    pub const fn ceiling(&self) -> NumericalRealization {
        self.ceiling
    }

    /// Returns the derived result class of this occurrence.
    ///
    /// [`ContractionF32ResultClass::StrictLeftFold`] under effective forbidden
    /// reassociation — bit-identical to the retired strict key's answer — and
    /// [`ContractionF32ResultClass::OrderedFullBinaryTrees`] under effective
    /// permitted reassociation.
    #[must_use]
    pub const fn result_class(&self) -> ContractionF32ResultClass {
        self.result_class
    }

    /// Returns whether ADR 0015 arithmetic contraction is effective.
    ///
    /// Always `false`: the operation maximum forbids it, whatever the stored
    /// ceiling permits. The stored ceiling itself is unaltered.
    #[must_use]
    pub const fn permits_arithmetic_contraction(&self) -> bool {
        false
    }

    /// Returns whether ordered reassociation is effective.
    #[must_use]
    pub const fn permits_reassociation(&self) -> bool {
        match self.result_class {
            ContractionF32ResultClass::StrictLeftFold => false,
            ContractionF32ResultClass::OrderedFullBinaryTrees => true,
        }
    }

    /// Returns whether contributor permutation is effective.
    ///
    /// Always `false`: the operation maximum is `unsupported`, and a ceiling
    /// cannot grant a freedom the operation withholds.
    #[must_use]
    pub const fn permits_permutation(&self) -> bool {
        false
    }

    /// Returns whether signed-zero elimination is effective.
    ///
    /// Always `false`, for the reason [`Self::permits_permutation`] states.
    #[must_use]
    pub const fn permits_signed_zero_elimination(&self) -> bool {
        false
    }
}

impl ContractionF32ReductionDescriptor {
    /// Resolves one caller ceiling against this operation's declared maxima.
    ///
    /// Resolution is exact: the complete ceiling is retained unchanged; the
    /// ceiling's canonical arithmetic NaN payload must equal the descriptor's
    /// `0x7fc00000`; the effective arithmetic-contraction, permutation, and
    /// signed-zero freedoms are `false` because the operation maxima forbid
    /// them; effective reassociation is `true` only when the descriptor
    /// maximum is `permission-gated` **and** the stored ceiling permits it;
    /// and the result class is [`ContractionF32ResultClass::StrictLeftFold`]
    /// for effective forbidden reassociation and
    /// [`ContractionF32ResultClass::OrderedFullBinaryTrees`] for effective
    /// permitted reassociation.
    ///
    /// # Errors
    ///
    /// Returns [`EffectiveContractionF32ProfileError::CanonicalNanMismatch`]
    /// when the ceiling's canonical arithmetic NaN payload is not the
    /// descriptor's.
    pub fn resolve(
        &self,
        ceiling: NumericalRealization,
    ) -> Result<EffectiveContractionF32Profile, EffectiveContractionF32ProfileError> {
        let expected = self.canonical_nan_bits();
        debug_assert_eq!(expected, CANONICAL_F32_ARITHMETIC_NAN_BITS);
        if ceiling.canonical_arithmetic_nan_bits != expected {
            return Err(EffectiveContractionF32ProfileError::CanonicalNanMismatch {
                expected,
                actual: ceiling.canonical_arithmetic_nan_bits,
            });
        }
        let effective_reassociation = match self.reassociation() {
            ContractionF32OrderFreedom::Unsupported => false,
            ContractionF32OrderFreedom::PermissionGated => ceiling.permits_reassociation(),
        };
        let result_class = if effective_reassociation {
            ContractionF32ResultClass::OrderedFullBinaryTrees
        } else {
            ContractionF32ResultClass::StrictLeftFold
        };
        Ok(EffectiveContractionF32Profile {
            ceiling,
            result_class,
        })
    }
}
