//! Target-neutral numerical-realization vocabulary for scheduled regions.
//!
//! A scheduled region preserves the declared numerical contract of the
//! computation it implements (ADR 0007). These types describe that contract in
//! target-neutral terms so both the compiler request boundary and the schedule
//! IR share one vocabulary rather than duplicating it.

/// Treatment of subnormal floating-point values crossing the region boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubnormalMode {
    /// Subnormal inputs and results are preserved exactly.
    Preserve,
}

/// Whether a numeric-reshaping transform is permitted by the contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumericalPermission {
    /// The transform is forbidden and must not change observable results.
    Forbidden,
}

/// The declared numerical realization a scheduled region must preserve.
///
/// The fields are read-transparent value data: a producer may read or assemble
/// one, but only the checked schedule builder can bind it into a
/// [`super::VerifiedScheduledRegion`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NumericalRealization {
    /// Stable key of the governing numerical contract.
    pub profile_key: &'static str,
    /// Canonical arithmetic NaN bit pattern for produced values.
    pub canonical_arithmetic_nan_bits: u32,
    /// Treatment of subnormal inputs.
    pub input_subnormals: SubnormalMode,
    /// Treatment of subnormal results.
    pub result_subnormals: SubnormalMode,
    /// Whether contraction (e.g. fused multiply-add) is permitted.
    pub contraction: NumericalPermission,
    /// Whether reduction reassociation is permitted.
    pub reassociation: NumericalPermission,
}

impl NumericalRealization {
    /// Assembles a numerical realization from its declared parts.
    #[must_use]
    pub const fn new(
        profile_key: &'static str,
        canonical_arithmetic_nan_bits: u32,
        input_subnormals: SubnormalMode,
        result_subnormals: SubnormalMode,
        contraction: NumericalPermission,
        reassociation: NumericalPermission,
    ) -> Self {
        Self {
            profile_key,
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
        }
    }

    /// Returns whether contraction is permitted by this realization.
    #[must_use]
    pub const fn permits_contraction(self) -> bool {
        !matches!(self.contraction, NumericalPermission::Forbidden)
    }

    /// Returns whether reduction reassociation is permitted by this realization.
    #[must_use]
    pub const fn permits_reassociation(self) -> bool {
        !matches!(self.reassociation, NumericalPermission::Forbidden)
    }
}
