//! The numerical contract a reference evaluation is performed under.
//!
//! # Why the oracle has to be told
//!
//! `docs/correctness-and-testing.md` says a reference comparison "follows the
//! declared numerical contract and conformance level". Until this module existed
//! the reference evaluator could not follow one: it reached no contract at all
//! and computed in whatever the host's `f32` arithmetic does, which preserves
//! subnormals.
//!
//! That is the dangerous direction rather than the safe one. A device whose
//! arithmetic flushes subnormals — which one measured Apple row does in `f32`,
//! under every math mode, while preserving them in `f16` — disagrees with a
//! preserving oracle on every subnormal input, and the oracle is the side that
//! would be called wrong. Stating the contract is what makes the comparison a
//! test of the device against the contract instead of a test of two
//! unstated readings against each other.
//!
//! # What this type does and does not cover
//!
//! It covers the two subnormal dimensions, because those are the ones the
//! reference arithmetic can realize by construction: flushing an operand before
//! an operation and flushing a newly produced result are both exact host
//! operations on the bits.
//!
//! It does **not** cover contraction or reassociation, and
//! [`ReferenceNumericalConformance::from_realization`] refuses a realization that
//! permits either rather than accepting one and ignoring it. The evaluator
//! computes a separately rounded multiply and add and a strict left fold in
//! canonical contributor order; those are one legal realization of a permissive
//! contract, but a permissive contract's *result set* is larger than one value,
//! and an oracle that returned a single value for it would be asserting a
//! bitwise equality the contract does not promise. Refusing names that gap
//! instead of hiding it.

use tiler_ir::schedule::{
    FlushedZeroSign, NumericalPermission, NumericalRealization, SubnormalMode,
};

use std::error::Error;
use std::fmt;

/// Why a declared numerical realization cannot be evaluated by this reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum UnsupportedReferenceContract {
    /// The realization permits fused-multiply-add contraction.
    ///
    /// A contract permitting contraction admits both the separately rounded and
    /// the fused result, so no single value is *the* reference result.
    ContractionPermitted,
    /// The realization permits reduction reassociation.
    ///
    /// A contract permitting reassociation admits every legal regrouping of the
    /// contributor sequence, which is a result set rather than one value.
    ReassociationPermitted,
}

impl UnsupportedReferenceContract {
    /// The stable diagnostic rule this refusal reports under.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ContractionPermitted => "reference.numerics.contraction-permitted",
            Self::ReassociationPermitted => "reference.numerics.reassociation-permitted",
        }
    }
}

impl fmt::Display for UnsupportedReferenceContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: the reference evaluates one value and this contract admits a result set",
            self.rule()
        )
    }
}

impl Error for UnsupportedReferenceContract {}

/// The numerical contract one reference evaluation is performed under.
///
/// Deliberately not `Default`. A conformance setting is a statement about what
/// the evaluated values *mean*, and [`Self::strict`] is that statement written
/// out rather than an absence of one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReferenceNumericalConformance {
    input_subnormals: SubnormalMode,
    result_subnormals: SubnormalMode,
}

impl ReferenceNumericalConformance {
    /// The strict reading: both subnormal dimensions preserved.
    ///
    /// This is what the evaluator computed before it could be told anything, so
    /// naming it changes no result. It is stated as a value rather than left
    /// implicit so that a caller comparing against a flushing device can see that
    /// it is comparing against a *different* contract, not against "the
    /// reference".
    #[must_use]
    pub const fn strict() -> Self {
        Self {
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
        }
    }

    /// States each subnormal dimension independently.
    ///
    /// The two are independent dimensions and neither implies the other (ADR
    /// 0019): input flushing treats an existing subnormal operand as zero before
    /// arithmetic, and result flushing replaces a newly produced subnormal
    /// result.
    #[must_use]
    pub const fn new(input_subnormals: SubnormalMode, result_subnormals: SubnormalMode) -> Self {
        Self {
            input_subnormals,
            result_subnormals,
        }
    }

    /// Derives the conformance from a region's declared numerical realization.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedReferenceContract`] when the realization permits a
    /// transform whose result is a set rather than one value. The refusal is the
    /// point: accepting such a contract and evaluating the strict reading anyway
    /// would produce an oracle that silently answers a question it was not asked.
    pub const fn from_realization(
        realization: &NumericalRealization,
    ) -> Result<Self, UnsupportedReferenceContract> {
        match realization.contraction {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::ContractionPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        match realization.reassociation {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::ReassociationPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        Ok(Self::new(
            realization.input_subnormals,
            realization.result_subnormals,
        ))
    }

    /// The declared treatment of subnormal operands.
    #[must_use]
    pub const fn input_subnormals(self) -> SubnormalMode {
        self.input_subnormals
    }

    /// The declared treatment of newly produced subnormal results.
    #[must_use]
    pub const fn result_subnormals(self) -> SubnormalMode {
        self.result_subnormals
    }

    /// Applies the input dimension to one operand, before arithmetic.
    #[must_use]
    pub fn apply_to_operand(self, value: f32) -> f32 {
        apply(self.input_subnormals, value)
    }

    /// Applies the result dimension to one newly produced arithmetic result.
    #[must_use]
    pub fn apply_to_result(self, value: f32) -> f32 {
        apply(self.result_subnormals, value)
    }
}

/// Applies one subnormal dimension to one value.
///
/// Exhaustive over both vocabularies rather than written with a wildcard, so
/// widening either is a build error here instead of a dimension silently
/// resolved as preservation.
///
/// `f32::is_subnormal` is false for both zeros and for every normal value, so
/// this replaces exactly the values the contract names and nothing else. The
/// sign question is not incidental: binary32 has two zeros, they are observably
/// different values, and the measured Apple flush produces the zero of the
/// flushed value's own sign — `0x80400000 * 2.0f` returns `0x80000000`, not
/// `0x00000000`.
fn apply(mode: SubnormalMode, value: f32) -> f32 {
    match mode {
        SubnormalMode::Preserve => value,
        SubnormalMode::FlushToZero { zero_sign } => {
            if value.is_subnormal() {
                match zero_sign {
                    FlushedZeroSign::PreservesSign => {
                        if value.is_sign_negative() {
                            -0.0_f32
                        } else {
                            0.0_f32
                        }
                    }
                    FlushedZeroSign::AlwaysPositive => 0.0_f32,
                }
            } else {
                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ReferenceNumericalConformance, UnsupportedReferenceContract};
    use tiler_ir::schedule::{
        FlushedZeroSign, NumericalPermission, NumericalRealization, SubnormalMode,
    };

    const fn flush(zero_sign: FlushedZeroSign) -> SubnormalMode {
        SubnormalMode::FlushToZero { zero_sign }
    }

    fn realization(
        input: SubnormalMode,
        result: SubnormalMode,
        contraction: NumericalPermission,
        reassociation: NumericalPermission,
    ) -> NumericalRealization {
        NumericalRealization::new(
            "tiler.test.contract.v1",
            0x7fc0_0000,
            input,
            result,
            contraction,
            reassociation,
        )
    }

    #[test]
    fn the_strict_reading_changes_no_value() {
        let strict = ReferenceNumericalConformance::strict();
        for bits in [0x0000_0001_u32, 0x0040_0000, 0x8040_0000, 0x3f80_0000] {
            let value = f32::from_bits(bits);
            assert_eq!(strict.apply_to_operand(value).to_bits(), bits);
            assert_eq!(strict.apply_to_result(value).to_bits(), bits);
        }
    }

    /// A sign-preserving flush produces the zero of the flushed value's own sign.
    #[test]
    fn a_sign_preserving_flush_keeps_the_sign_of_the_value_it_replaces() {
        let conformance = ReferenceNumericalConformance::new(
            flush(FlushedZeroSign::PreservesSign),
            flush(FlushedZeroSign::PreservesSign),
        );
        assert_eq!(
            conformance
                .apply_to_operand(f32::from_bits(0x0040_0000))
                .to_bits(),
            0x0000_0000
        );
        assert_eq!(
            conformance
                .apply_to_result(f32::from_bits(0x8040_0000))
                .to_bits(),
            0x8000_0000
        );
        // A normal value and a zero are untouched by either dimension.
        assert_eq!(
            conformance
                .apply_to_operand(f32::from_bits(0x0080_0000))
                .to_bits(),
            0x0080_0000
        );
        assert_eq!(
            conformance
                .apply_to_result(f32::from_bits(0x8000_0000))
                .to_bits(),
            0x8000_0000
        );
    }

    /// An always-positive flush erases the sign, which is a different behaviour.
    #[test]
    fn an_always_positive_flush_is_not_the_sign_preserving_one() {
        let conformance = ReferenceNumericalConformance::new(
            flush(FlushedZeroSign::AlwaysPositive),
            flush(FlushedZeroSign::AlwaysPositive),
        );
        assert_eq!(
            conformance
                .apply_to_operand(f32::from_bits(0x8040_0000))
                .to_bits(),
            0x0000_0000
        );
    }

    /// The two dimensions are independent: setting one does not set the other.
    #[test]
    fn the_two_subnormal_dimensions_are_resolved_independently() {
        let conformance = ReferenceNumericalConformance::new(
            SubnormalMode::Preserve,
            flush(FlushedZeroSign::PreservesSign),
        );
        assert_eq!(
            conformance
                .apply_to_operand(f32::from_bits(0x0040_0000))
                .to_bits(),
            0x0040_0000
        );
        assert_eq!(
            conformance
                .apply_to_result(f32::from_bits(0x0040_0000))
                .to_bits(),
            0x0000_0000
        );
    }

    /// A permissive realization is refused rather than evaluated strictly.
    #[test]
    fn a_contract_admitting_a_result_set_is_refused_by_name() {
        assert_eq!(
            ReferenceNumericalConformance::from_realization(&realization(
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Permitted,
                NumericalPermission::Forbidden,
            )),
            Err(UnsupportedReferenceContract::ContractionPermitted)
        );
        assert_eq!(
            ReferenceNumericalConformance::from_realization(&realization(
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Permitted,
            )),
            Err(UnsupportedReferenceContract::ReassociationPermitted)
        );
        assert_eq!(
            UnsupportedReferenceContract::ContractionPermitted.rule(),
            "reference.numerics.contraction-permitted"
        );
        assert!(
            UnsupportedReferenceContract::ReassociationPermitted
                .to_string()
                .contains("result set")
        );
    }

    /// A strict realization carries both subnormal dimensions across unchanged.
    #[test]
    fn a_strict_realization_carries_both_subnormal_dimensions_forward() {
        let declared = realization(
            flush(FlushedZeroSign::PreservesSign),
            SubnormalMode::Preserve,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
        );
        let conformance = ReferenceNumericalConformance::from_realization(&declared)
            .expect("a strict realization is evaluable");
        assert_eq!(
            conformance.input_subnormals(),
            flush(FlushedZeroSign::PreservesSign)
        );
        assert_eq!(conformance.result_subnormals(), SubnormalMode::Preserve);
    }
}
