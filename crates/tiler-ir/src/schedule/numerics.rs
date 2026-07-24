//! Target-neutral numerical-realization vocabulary for scheduled regions.
//!
//! A scheduled region preserves the declared numerical contract of the
//! computation it implements (ADR 0007). These types describe that contract in
//! target-neutral terms so both the compiler request boundary and the schedule
//! IR share one vocabulary rather than duplicating it.
//!
//! The vocabulary is the one ADR 0019 and ADR 0011 accept: subnormal input and
//! subnormal result handling are independent dimensions, each resolving to
//! preservation or an explicit flush-to-zero behaviour, and each numeric
//! transform is an independently resolved permission. A target that couples two
//! of these dimensions in one execution mode declares that coupling on its own
//! profile; it never collapses the semantic dimensions here (ADR 0019).
//!
//! None of these enums is `#[non_exhaustive]`, and that is load-bearing rather
//! than incidental. Every consumer that encodes one into canonical identity or
//! matches one to decide target support does so with an exhaustive match, so
//! widening the vocabulary is a build error at each such site instead of a
//! silent identity collision or a silently dropped obligation (ADR 0074
//! convention 5b, ADR 0076 item 6).

/// The zero a flush-to-zero behaviour produces.
///
/// A flush-to-zero mode that does not state which zero it produces cannot be
/// checked against measured hardware and cannot be reference-evaluated, because
/// binary32 has two zeros and they are observably different values (ADR 0076
/// item 1). The sign is carried here, on the behaviour itself, rather than
/// resolved from a separate signed-zero permission: a permission may leave the
/// sign of a zero *unspecified*, and an unspecified flush result is exactly the
/// under-specification this vocabulary exists to remove.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FlushedZeroSign {
    /// The produced zero carries the sign of the value it replaced.
    ///
    /// **Measurement.** Apple M4 Max, macOS 27.0, `Apple metal version
    /// 32023.883`: an emitted `x * 2.0f` returns `0x80000000` for the operand
    /// `0x80400000`, not `0x00000000`.
    PreservesSign,
    /// Every flushed value produces positive zero regardless of its own sign.
    AlwaysPositive,
}

/// Treatment of subnormal floating-point values crossing the region boundary.
///
/// The two dimensions of [`NumericalRealization`] that use this type — inputs
/// and results — are resolved independently (ADR 0019).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubnormalMode {
    /// Subnormal values are preserved exactly, retaining gradual underflow.
    Preserve,
    /// Subnormal values are replaced by a zero of the stated sign.
    ///
    /// For the input dimension this treats an existing subnormal operand as
    /// zero before arithmetic; for the result dimension it replaces a newly
    /// produced subnormal result. The two are observably different behaviours
    /// and neither implies the other.
    FlushToZero {
        /// Which zero the flush produces.
        zero_sign: FlushedZeroSign,
    },
}

/// Whether a numeric-reshaping transform is permitted by the contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumericalPermission {
    /// The transform is forbidden and must not change observable results.
    Forbidden,
    /// The transform is permitted and its results may differ from the strict
    /// reading.
    ///
    /// A permission is granted per dimension and never implies another: one
    /// permitted transform authorizes exactly the freedom it names (ADR 0011).
    Permitted,
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
        permits(self.contraction)
    }

    /// Returns whether reduction reassociation is permitted by this realization.
    #[must_use]
    pub const fn permits_reassociation(self) -> bool {
        permits(self.reassociation)
    }
}

/// Returns whether a permission grants its transform.
///
/// Matched exhaustively rather than written as a negated `matches!`, so a
/// widened [`NumericalPermission`] stops the build here instead of being
/// silently classified with `Forbidden`.
const fn permits(permission: NumericalPermission) -> bool {
    match permission {
        NumericalPermission::Forbidden => false,
        NumericalPermission::Permitted => true,
    }
}
