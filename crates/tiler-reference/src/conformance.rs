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
//! It does **not** cover contraction, reassociation, permutation, signed-zero
//! elimination, or exceptional-value absence assumptions, and
//! [`ReferenceNumericalConformance::from_realization`] refuses a realization that
//! uses any of them rather than accepting one and ignoring it. The evaluator
//! computes a separately rounded multiply and add, a strict left fold in
//! canonical contributor order, and bit-preserving signed-zero behaviour. Those
//! are one legal realization of a permissive transform contract, but that
//! contract's result set is larger than one value, and an oracle that returned a
//! single value would assert a bitwise equality the contract does not promise.
//! It also has no authority that validates an exceptional-value absence
//! assumption before evaluation. Refusing names each gap instead of hiding it.
//!
//! # The subject: which format the dimensions were resolved for
//!
//! A [`SubnormalMode`] names no format, and the same resolution means different
//! things over different value sets: the measured Apple row flushes `f32`
//! subnormals and preserves `f16` ones under one execution mode, so a conformance
//! carried without its subject can be applied by a capability computing in a
//! format the conformance was never stated about. [`ConformanceSubject`] is that
//! subject, and [`ReferenceEvaluationRequest::conformance_for`] is where a
//! capability states its own arithmetic and is refused a conformance resolved for
//! another.
//!
//! **Only [`ReferenceNumericalConformance::from_realization`] states one.**
//! [`ReferenceNumericalConformance::strict`] and
//! [`ReferenceNumericalConformance::new`] resolve two format-agnostic dimensions
//! and nothing more, so they produce [`ConformanceSubject::Unstated`] and every
//! capability accepts them exactly as it did before this subject existed. That is
//! a real gap and it is named rather than papered over: the agreement check is a
//! *tested guarantee* for a conformance drawn from a declared realization, and
//! `Unstated` is the population it cannot speak for.
//!
//! [`ReferenceEvaluationRequest::conformance_for`]: crate::ReferenceEvaluationRequest::conformance_for

use tiler_ir::schedule::{
    ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    NumericalRealization, SubnormalMode, ValueDomainProvenance,
};
use tiler_ir::semantic::{CANONICAL_BF16_ARITHMETIC_NAN_BITS, CANONICAL_F32_ARITHMETIC_NAN_BITS};

use super::error::ReferenceOperationError;

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
    /// The realization permits ordered reassociation.
    ///
    /// A contract permitting reassociation admits every legal regrouping of the
    /// same-operation operand sequence, which is a result set rather than one
    /// value.
    ReassociationPermitted,
    /// The realization permits reduction contributor permutation.
    PermutationPermitted,
    /// The realization permits eliminating signed-zero distinctions.
    SignedZeroEliminationPermitted,
    /// The realization assumes NaNs absent on evidence this evaluator cannot validate.
    NanAbsenceAssumed {
        /// Authority behind the domain assumption.
        provenance: ValueDomainProvenance,
    },
    /// The realization assumes infinities absent on evidence this evaluator cannot validate.
    InfinityAbsenceAssumed {
        /// Authority behind the domain assumption.
        provenance: ValueDomainProvenance,
    },
    /// This reference performs no arithmetic in the stated subject.
    ///
    /// [`ArithmeticType`] names four formats and this crate computes in two of
    /// them. A conformance resolved for either of the other two could reach no
    /// capability, and no canonical arithmetic NaN payload is declared for either,
    /// so there is nothing to check the realization's own declaration against.
    /// Refusing at the bridge is the fail-closed answer; admitting one and
    /// carrying it would produce a subject no capability could ever agree with.
    ArithmeticNotEvaluable {
        /// The stated subject.
        arithmetic: ArithmeticType,
    },
    /// The realization's declared canonical NaN payload is not the subject's.
    ///
    /// [`NumericalRealization::canonical_arithmetic_nan_bits`] carries "the pattern
    /// of the region's own arithmetic type, zero-extended into this field", so a
    /// `bf16` region declares `0x0000_7fc0` and an `f32` region the whole
    /// `0x7fc0_0000`. A caller stating a subject the realization's own declaration
    /// contradicts has drawn the subject from somewhere other than the region that
    /// declared the realization, and the two readings must not be silently merged
    /// into one conformance.
    DeclaredNanPayloadMismatch {
        /// The stated subject.
        arithmetic: ArithmeticType,
        /// The payload the realization declares.
        declared: u32,
        /// The payload the stated subject's canonical arithmetic NaN is.
        expected: u32,
    },
}

impl UnsupportedReferenceContract {
    /// The stable diagnostic rule this refusal reports under.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::ContractionPermitted => "reference.numerics.contraction-permitted",
            Self::ReassociationPermitted => "reference.numerics.reassociation-permitted",
            Self::PermutationPermitted => "reference.numerics.permutation-permitted",
            Self::SignedZeroEliminationPermitted => {
                "reference.numerics.signed-zero-elimination-permitted"
            }
            Self::NanAbsenceAssumed { .. } => "reference.numerics.nan-absence-assumed",
            Self::InfinityAbsenceAssumed { .. } => "reference.numerics.infinity-absence-assumed",
            Self::ArithmeticNotEvaluable { .. } => "reference.numerics.arithmetic-not-evaluable",
            Self::DeclaredNanPayloadMismatch { .. } => {
                "reference.numerics.declared-nan-payload-mismatch"
            }
        }
    }
}

impl fmt::Display for UnsupportedReferenceContract {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractionPermitted
            | Self::ReassociationPermitted
            | Self::PermutationPermitted
            | Self::SignedZeroEliminationPermitted => write!(
                formatter,
                "{}: the reference evaluates one value and this contract admits a result set",
                self.rule()
            ),
            Self::NanAbsenceAssumed { provenance }
            | Self::InfinityAbsenceAssumed { provenance } => write!(
                formatter,
                "{}: the reference cannot validate the {provenance:?} domain assumption",
                self.rule()
            ),
            Self::ArithmeticNotEvaluable { arithmetic } => write!(
                formatter,
                "{}: this reference performs no {} arithmetic",
                self.rule(),
                arithmetic.canonical_type_key()
            ),
            Self::DeclaredNanPayloadMismatch {
                arithmetic,
                declared,
                expected,
            } => write!(
                formatter,
                "{}: the realization declares {declared:#010x} and {} canonicalizes to \
                 {expected:#010x}",
                self.rule(),
                arithmetic.canonical_type_key()
            ),
        }
    }
}

impl Error for UnsupportedReferenceContract {}

/// The arithmetic type a conformance's subnormal dimensions were resolved for.
///
/// Deliberately a named vocabulary rather than an `Option<ArithmeticType>`: the
/// absent case is a *statement about the conformance* — that it resolves two
/// format-agnostic dimensions and identifies no value set — and every consumer
/// matches it exhaustively, so admitting a third state would be a build error at
/// each agreement check instead of a case silently classified with one of these
/// two.
///
/// Not `#[non_exhaustive]`, for the reason
/// [`ArithmeticType`]'s own definition states: widening it must stop the build at
/// every site that decides whether a capability may apply a conformance.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConformanceSubject {
    /// The conformance names no arithmetic type.
    ///
    /// What [`ReferenceNumericalConformance::strict`] and
    /// [`ReferenceNumericalConformance::new`] produce. A capability applies such a
    /// conformance because there is no disagreement to detect, which is exactly
    /// the reading every capability performed before the subject existed.
    Unstated,
    /// The conformance was resolved for exactly one arithmetic type.
    ///
    /// Reachable only through [`ReferenceNumericalConformance::from_realization`],
    /// so a stated subject is always one a declared realization was bridged under
    /// rather than one a caller asserted beside an unrelated pair of modes.
    Arithmetic(ArithmeticType),
}

/// The canonical arithmetic NaN payload this reference evaluates one format under.
///
/// `None` for a format this crate performs no arithmetic in, which is what makes
/// [`UnsupportedReferenceContract::ArithmeticNotEvaluable`] a refusal rather than
/// a payload guess. Exhaustive rather than written with a wildcard, so admitting a
/// third arithmetic type is a build error here instead of a silent refusal.
const fn evaluable_canonical_nan_bits(arithmetic: ArithmeticType) -> Option<u32> {
    match arithmetic {
        ArithmeticType::F32 => Some(CANONICAL_F32_ARITHMETIC_NAN_BITS),
        // Zero-extended by a cast rather than by `u32::from`, which is not yet
        // callable in a `const fn`. The widening is exact in either spelling, and
        // it is the same zero-extension the realization's own field documents.
        ArithmeticType::Bf16 => Some(CANONICAL_BF16_ARITHMETIC_NAN_BITS as u32),
        ArithmeticType::F16 | ArithmeticType::F64 => None,
    }
}

/// The numerical contract one reference evaluation is performed under.
///
/// Deliberately not `Default`. A conformance setting is a statement about what
/// the evaluated values *mean*, and [`Self::strict`] is that statement written
/// out rather than an absence of one.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReferenceNumericalConformance {
    subject: ConformanceSubject,
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
    ///
    /// Its subject is [`ConformanceSubject::Unstated`]: preservation of subnormals
    /// is the same statement over every value set, so this constructor has no
    /// format to name and states none.
    #[must_use]
    pub const fn strict() -> Self {
        Self::unsubjected(SubnormalMode::Preserve, SubnormalMode::Preserve)
    }

    /// States each subnormal dimension independently.
    ///
    /// The two are independent dimensions and neither implies the other (ADR
    /// 0019): input flushing treats an existing subnormal operand as zero before
    /// arithmetic, and result flushing replaces a newly produced subnormal
    /// result.
    ///
    /// Its subject is [`ConformanceSubject::Unstated`]. A caller holding two loose
    /// modes has not said which format resolved them, and inventing a subject here
    /// would let a capability's agreement check pass on an assertion nothing
    /// declared. [`Self::from_realization`] is the constructor that has a region's
    /// declaration to draw one from.
    #[must_use]
    pub const fn new(input_subnormals: SubnormalMode, result_subnormals: SubnormalMode) -> Self {
        Self::unsubjected(input_subnormals, result_subnormals)
    }

    const fn unsubjected(
        input_subnormals: SubnormalMode,
        result_subnormals: SubnormalMode,
    ) -> Self {
        Self {
            subject: ConformanceSubject::Unstated,
            input_subnormals,
            result_subnormals,
        }
    }

    /// Derives the conformance from a region's declared numerical realization,
    /// stated about the region's own arithmetic type.
    ///
    /// # The subject is an argument rather than a field of the realization
    ///
    /// A region's arithmetic type is a total function of its scalar program, so it
    /// is the *region* that answers this and not the realization it carries — and
    /// giving [`NumericalRealization`] a subject field would be an identity-domain
    /// migration to restate something the schedule layer already derives. The
    /// caller holding the region therefore states it here, and the realization's
    /// own [`NumericalRealization::canonical_arithmetic_nan_bits`] is what checks
    /// the statement: that field carries the canonical arithmetic NaN pattern of
    /// the region's own type, so a subject the declaration contradicts is refused
    /// rather than carried.
    ///
    /// A caller holding a verified scheduled region reads both arguments off one
    /// object rather than assembling them from two:
    /// [`RealizationWitness::of`](tiler_ir::schedule::RealizationWitness::of) gives
    /// [`realization`](tiler_ir::schedule::RealizationWitness::realization) and
    /// [`accumulation`](tiler_ir::schedule::RealizationWitness::accumulation) — the
    /// region's own arithmetic type, which the intrinsic schedule verifier already
    /// requires a declared accumulation to agree with. Sourcing the two separately
    /// is what would let them drift.
    ///
    /// # Errors
    ///
    /// Returns [`UnsupportedReferenceContract::ArithmeticNotEvaluable`] for a
    /// format this reference performs no arithmetic in, and
    /// [`UnsupportedReferenceContract::DeclaredNanPayloadMismatch`] when the stated
    /// subject disagrees with the realization's own declaration. Both are checked
    /// before the transform permissions, because a realization whose subject is
    /// unresolved is not an object whose freedoms mean anything yet.
    ///
    /// Returns the remaining [`UnsupportedReferenceContract`] variants when the
    /// realization permits a transform whose result is a set rather than one
    /// value. The refusal is the point: accepting such a contract and evaluating
    /// the strict reading anyway would produce an oracle that silently answers a
    /// question it was not asked.
    pub const fn from_realization(
        realization: &NumericalRealization,
        arithmetic: ArithmeticType,
    ) -> Result<Self, UnsupportedReferenceContract> {
        let NumericalRealization {
            profile_key: _,
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            nan_assumptions,
            infinity_assumptions,
        } = *realization;
        let Some(expected) = evaluable_canonical_nan_bits(arithmetic) else {
            return Err(UnsupportedReferenceContract::ArithmeticNotEvaluable { arithmetic });
        };
        if canonical_arithmetic_nan_bits != expected {
            return Err(UnsupportedReferenceContract::DeclaredNanPayloadMismatch {
                arithmetic,
                declared: canonical_arithmetic_nan_bits,
                expected,
            });
        }
        match contraction {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::ContractionPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        match reassociation {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::ReassociationPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        match permutation {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::PermutationPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        match signed_zero {
            NumericalPermission::Permitted => {
                return Err(UnsupportedReferenceContract::SignedZeroEliminationPermitted);
            }
            NumericalPermission::Forbidden => {}
        }
        match nan_assumptions {
            ExceptionalValueAssumption::MakeNoAssumption => {}
            ExceptionalValueAssumption::AssumeAbsent { provenance } => {
                return Err(UnsupportedReferenceContract::NanAbsenceAssumed { provenance });
            }
        }
        match infinity_assumptions {
            ExceptionalValueAssumption::MakeNoAssumption => {}
            ExceptionalValueAssumption::AssumeAbsent { provenance } => {
                return Err(UnsupportedReferenceContract::InfinityAbsenceAssumed { provenance });
            }
        }
        Ok(Self {
            subject: ConformanceSubject::Arithmetic(arithmetic),
            input_subnormals,
            result_subnormals,
        })
    }

    /// The arithmetic type these dimensions were resolved for, if any was stated.
    #[must_use]
    pub const fn subject(self) -> ConformanceSubject {
        self.subject
    }

    /// Returns this conformance to a capability computing in `arithmetic`.
    ///
    /// The one agreement check. A capability's arithmetic type is fixed by its own
    /// construction — every operand and result it admits is one resolved type — so
    /// it can state it here, and a conformance resolved for another format is
    /// refused instead of having its modes applied to values that format's rule was
    /// never stated about.
    ///
    /// An [`ConformanceSubject::Unstated`] conformance is returned to every
    /// capability. That is not the check passing: it is the check having nothing to
    /// compare, which the module header names as this guarantee's boundary.
    pub(crate) const fn checked_for(
        self,
        arithmetic: ArithmeticType,
    ) -> Result<Self, ReferenceOperationError> {
        match self.subject {
            ConformanceSubject::Unstated => Ok(self),
            ConformanceSubject::Arithmetic(stated) => {
                // Compared by tag rather than `==`, because `PartialEq::eq` is not
                // a const function; the tags are the identity encoding this
                // vocabulary already defines as injective.
                if stated.tag() == arithmetic.tag() {
                    Ok(self)
                } else {
                    Err(ReferenceOperationError::ConformanceSubject {
                        capability: arithmetic,
                        stated,
                    })
                }
            }
        }
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
    use super::{ConformanceSubject, ReferenceNumericalConformance, UnsupportedReferenceContract};
    use crate::ReferenceOperationError;
    use tiler_ir::schedule::{
        ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
        NumericalRealization, SubnormalMode, ValueDomainProvenance,
    };
    use tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS;

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
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
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
            ReferenceNumericalConformance::from_realization(
                &realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Permitted,
                    NumericalPermission::Forbidden,
                ),
                ArithmeticType::F32
            ),
            Err(UnsupportedReferenceContract::ContractionPermitted)
        );
        assert_eq!(
            ReferenceNumericalConformance::from_realization(
                &realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Permitted,
                ),
                ArithmeticType::F32
            ),
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

    /// Every newly carried freedom is refused independently rather than ignored.
    #[test]
    fn every_new_dimension_is_accounted_for_by_the_reference_boundary() {
        let mut checked = 0_usize;
        let strict = realization(
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
        );
        for (declared, expected) in [
            (
                NumericalRealization {
                    permutation: NumericalPermission::Permitted,
                    ..strict
                },
                UnsupportedReferenceContract::PermutationPermitted,
            ),
            (
                NumericalRealization {
                    signed_zero: NumericalPermission::Permitted,
                    ..strict
                },
                UnsupportedReferenceContract::SignedZeroEliminationPermitted,
            ),
            (
                NumericalRealization {
                    nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                        provenance: ValueDomainProvenance::CompilerProven,
                    },
                    ..strict
                },
                UnsupportedReferenceContract::NanAbsenceAssumed {
                    provenance: ValueDomainProvenance::CompilerProven,
                },
            ),
            (
                NumericalRealization {
                    infinity_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                        provenance: ValueDomainProvenance::RuntimeValidated,
                    },
                    ..strict
                },
                UnsupportedReferenceContract::InfinityAbsenceAssumed {
                    provenance: ValueDomainProvenance::RuntimeValidated,
                },
            ),
        ] {
            assert_eq!(
                ReferenceNumericalConformance::from_realization(&declared, ArithmeticType::F32),
                Err(expected),
            );
            checked += 1;
        }
        assert_eq!(checked, 4, "every newly carried freedom was exercised");
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
        let conformance =
            ReferenceNumericalConformance::from_realization(&declared, ArithmeticType::F32)
                .expect("a strict realization is evaluable");
        assert_eq!(
            conformance.input_subnormals(),
            flush(FlushedZeroSign::PreservesSign)
        );
        assert_eq!(conformance.result_subnormals(), SubnormalMode::Preserve);
    }

    /// A realization declaring one format's NaN payload is `bf16`'s zero-extended.
    fn bf16_realization(input: SubnormalMode, result: SubnormalMode) -> NumericalRealization {
        NumericalRealization {
            canonical_arithmetic_nan_bits: u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            ..realization(
                input,
                result,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
            )
        }
    }

    /// The bridge carries the subject it is told, for each format it evaluates.
    ///
    /// Both directions, because a bridge that hard-coded either answer would still
    /// satisfy one of them: the `f32` realization must produce the `f32` subject and
    /// the `bf16` realization the `bf16` one.
    #[test]
    fn the_bridge_carries_the_subject_it_is_told() {
        let mut checked = 0_usize;
        for (declared, arithmetic) in [
            (
                realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                ),
                ArithmeticType::F32,
            ),
            (
                bf16_realization(SubnormalMode::Preserve, SubnormalMode::Preserve),
                ArithmeticType::Bf16,
            ),
        ] {
            let conformance =
                ReferenceNumericalConformance::from_realization(&declared, arithmetic)
                    .expect("a strict realization of an evaluable format bridges");
            assert_eq!(
                conformance.subject(),
                ConformanceSubject::Arithmetic(arithmetic)
            );
            checked += 1;
        }
        assert_eq!(checked, 2, "both evaluable formats bridged");
        // The two subject-free constructors state no format, which is what keeps
        // every existing caller's conformance applicable by every capability.
        assert_eq!(
            ReferenceNumericalConformance::strict().subject(),
            ConformanceSubject::Unstated
        );
        assert_eq!(
            ReferenceNumericalConformance::new(
                flush(FlushedZeroSign::AlwaysPositive),
                SubnormalMode::Preserve
            )
            .subject(),
            ConformanceSubject::Unstated
        );
    }

    /// A subject the realization's own declaration contradicts is refused.
    ///
    /// The check that makes the stated subject an *agreement* rather than an
    /// assertion: the realization carries the canonical arithmetic NaN pattern of
    /// the region's own type, so the two readings are compared instead of the
    /// caller's being taken on trust.
    #[test]
    fn a_subject_the_declaration_contradicts_is_refused() {
        assert_eq!(
            ReferenceNumericalConformance::from_realization(
                &bf16_realization(SubnormalMode::Preserve, SubnormalMode::Preserve),
                ArithmeticType::F32
            ),
            Err(UnsupportedReferenceContract::DeclaredNanPayloadMismatch {
                arithmetic: ArithmeticType::F32,
                declared: u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                expected: 0x7fc0_0000,
            })
        );
        assert_eq!(
            ReferenceNumericalConformance::from_realization(
                &realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                ),
                ArithmeticType::Bf16
            ),
            Err(UnsupportedReferenceContract::DeclaredNanPayloadMismatch {
                arithmetic: ArithmeticType::Bf16,
                declared: 0x7fc0_0000,
                expected: u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            })
        );
    }

    /// A format this reference computes nothing in is refused at the bridge.
    ///
    /// The whole vocabulary is walked rather than the two rejected members named,
    /// so admitting a third arithmetic type moves this population instead of
    /// leaving a case nothing decided.
    #[test]
    fn a_format_this_reference_cannot_evaluate_is_refused_at_the_bridge() {
        let mut evaluable = 0_usize;
        let mut refused = 0_usize;
        for arithmetic in ArithmeticType::ALL {
            let declared = NumericalRealization {
                canonical_arithmetic_nan_bits: match arithmetic {
                    ArithmeticType::Bf16 => u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                    ArithmeticType::F16 | ArithmeticType::F32 | ArithmeticType::F64 => 0x7fc0_0000,
                },
                ..realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                )
            };
            match ReferenceNumericalConformance::from_realization(&declared, arithmetic) {
                Ok(conformance) => {
                    assert_eq!(
                        conformance.subject(),
                        ConformanceSubject::Arithmetic(arithmetic)
                    );
                    evaluable += 1;
                }
                Err(refusal) => {
                    assert_eq!(
                        refusal,
                        UnsupportedReferenceContract::ArithmeticNotEvaluable { arithmetic },
                        "{arithmetic:?} was refused for the wrong reason",
                    );
                    refused += 1;
                }
            }
        }
        assert_eq!(evaluable, 2, "this reference evaluates f32 and bf16");
        assert_eq!(refused, 2, "and refuses the other two by name");
        assert!(
            UnsupportedReferenceContract::ArithmeticNotEvaluable {
                arithmetic: ArithmeticType::F64,
            }
            .to_string()
            .contains("tiler::f64@1")
        );
    }

    /// A capability is handed a conformance only when its subject is its own.
    ///
    /// Walked over the whole vocabulary in both positions, so the population that
    /// must agree and the population that must be refused are both counted rather
    /// than sampled.
    #[test]
    fn a_capability_is_refused_a_conformance_resolved_for_another_format() {
        let mut agreed = 0_usize;
        let mut refused = 0_usize;
        for stated in [ArithmeticType::F32, ArithmeticType::Bf16] {
            let declared = match stated {
                ArithmeticType::Bf16 => {
                    bf16_realization(SubnormalMode::Preserve, SubnormalMode::Preserve)
                }
                ArithmeticType::F16 | ArithmeticType::F32 | ArithmeticType::F64 => realization(
                    SubnormalMode::Preserve,
                    SubnormalMode::Preserve,
                    NumericalPermission::Forbidden,
                    NumericalPermission::Forbidden,
                ),
            };
            let conformance = ReferenceNumericalConformance::from_realization(&declared, stated)
                .expect("an evaluable format bridges");
            for capability in ArithmeticType::ALL {
                if capability == stated {
                    assert_eq!(conformance.checked_for(capability), Ok(conformance));
                    agreed += 1;
                } else {
                    assert_eq!(
                        conformance.checked_for(capability),
                        Err(ReferenceOperationError::ConformanceSubject { capability, stated })
                    );
                    refused += 1;
                }
            }
        }
        assert_eq!(agreed, 2, "each stated subject agreed with its own format");
        assert_eq!(refused, 6, "and was refused by the other three");
        // A conformance carrying no subject reaches every capability, which is the
        // named boundary of this guarantee rather than the check passing.
        for capability in ArithmeticType::ALL {
            assert_eq!(
                ReferenceNumericalConformance::strict().checked_for(capability),
                Ok(ReferenceNumericalConformance::strict())
            );
        }
    }
}
