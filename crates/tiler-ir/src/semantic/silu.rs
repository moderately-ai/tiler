//! The governed `f32` `SiLU` activation, and the accuracy contract it carries.
//!
//! **Why one atomic key rather than a composition.** `y = x / (1 + Exp(-x))` is
//! four elementary steps, and Tiler admits none of `Exp`, `Sigmoid`, or a general
//! division as a semantic operation. Registering the activation as one key is not
//! a convenience: it is what lets the operation's *identity* carry a resolved
//! accuracy contract for the one inexact step, which ADR 0016 requires and which a
//! composition of unregistered parts could not state at all.
//!
//! **The division form is the operation, and the sigmoid-product form is a
//! different one.** Over the boundary corpus the two agree everywhere except at
//! `-88.0`, where `x / (1 + exp(-x))` reproduces the reference `0x83354ddc` and
//! `x * sigmoid(x)` gives `0x83354ddb` — one ULP apart. Both spellings are
//! conventional and a corpus without an input near the exponential's overflow
//! threshold reports them identical, which is exactly how a key admitted under one
//! spelling would silently deliver the other. [`silu_f32_reference_semantics`]
//! pins the division form, and [`SILU_F32_FACT_EVALUATION_ORDER`] states the three
//! exact ADR 0024 round-to-nearest-ties-to-even boundaries the composition
//! carries.
//!
//! **What the accuracy contract covers, and what it deliberately does not.**
//! [`silu_f32_exponential_accuracy_contract`] resolves the accuracy of the
//! *subordinate exponential only*. Under ADR 0024 the negation is exact by IEEE
//! sign manipulation and the addition and the division each round once, so the
//! exponential is the composition's one inexact element and the only step with a
//! tolerance to state. The contract's `operation` is this key rather than a minted
//! `Exp` key, because no general exponential is admitted here and a contract
//! naming a key nothing registers would be an identity for an operation that does
//! not exist.
//!
//! **The exceptional-value, signed-zero, and subnormal policies are stated
//! independently of the error metric**, as ADR 0042 requires, and they are stated
//! for the composed activation rather than inferred from the exponential's bound.
//! The subnormal band *is* reached — near zero the activation is `fl(x / 2)`, and
//! the subordinate exponential underflows into the band for large positive
//! arguments — so the preservation policy is one a flushing target is genuinely
//! asked to honour rather than a claim about a range the operation never enters.
//! [`SILU_F32_FACT_SUBNORMALS`] carries the measurement for each region separately,
//! because the large-negative tail is the one region where no subnormal arises and
//! a policy quantified over the domain cannot be justified from it. That tail is
//! also the one place a reader might mistake for a flush and is not one: the exact
//! `-0.0` the `-88.73` band produces is a correctly rounded division by an
//! overflowed infinity.

use std::sync::Arc;

use super::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
    AccuracyPredicate, DomainBound, DomainErrorRule, DomainInterval, ExactRational, ExactTolerance,
    ExceptionalValueContract, FiniteOverflowRule, InfiniteReferenceRule, NanReferenceRule,
    OperandOrdinal, ReferenceResultClass, ReferenceResultConstraint, ulp_reference_gap_metric_key,
};
use super::registry::standard_conformance;
use super::{
    AttributeFieldId, CanonicalField, CanonicalValue, F32, NormativeDefinitionRef, OpKey,
    OperationArity, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferenceOutputs, OperationInferenceRequest,
    OperationInferencer, OperationSchema, ProviderDiagnosticCode, RegistryError,
    SemanticRegistryRegistrar, ValueFact,
};

/// Exact binary32 payload of the largest argument whose exponential is finite.
///
/// `exp` overflows binary32 above `ln(f32::MAX) = 88.7228391...`, and this is the
/// largest binary32 value strictly below that threshold; its successor
/// `0x42b17218` is above it. The bound is written as a bit pattern rather than a
/// decimal literal because it is a *representable* value and the exactness is the
/// whole point: the accuracy domain of [`silu_f32_exponential_accuracy_contract`]
/// closes here, and the operand it bounds is always the negation of a binary32
/// input, so no admissible argument falls in the gap between this value and the
/// real threshold.
pub const SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS: u32 = 0x42b1_7217;

/// ULP tolerance the subordinate exponential's resolved contract states.
///
/// **Derivation, not adoption.** Metal's Table 8.1 gives `exp <= 4 ulp` under
/// *Apple's* definition of `ulp`, and ADR 0042 forbids translating a bound across
/// metric definitions by name. Apple's second clause — "otherwise `ulp(x)` is the
/// distance between the two nonequal finite floating-point numbers nearest x" —
/// admits two readings, and the largest ratio of Apple's scale to
/// `tiler::ulp-reference-gap@1`'s over the whole finite domain is two under the
/// reading consistent with the first clause and three under the more literal one.
/// Nothing in the retained specification chooses, so the conservative factor
/// covering both readings is three and the translated bound is `4 * 3 = 12`.
///
/// Stating twelve is therefore a claim about *both* readings rather than a
/// selection between them. A derivation that adopted four, or eight, would be
/// claiming a reading and a domain it would have to name.
pub const SILU_F32_EXPONENTIAL_ULP_TOLERANCE: u64 = 12;

/// Fact field naming the type this operation's arithmetic is performed at.
pub const SILU_F32_FACT_COMPUTATION_TYPE: AttributeFieldId = AttributeFieldId::new(1);
/// Fact field naming the result value type.
pub const SILU_F32_FACT_RESULT_TYPE: AttributeFieldId = AttributeFieldId::new(2);
/// Fact field naming the exact spelling and its three rounding boundaries.
///
/// The *order* is part of the operation, not an implementation note: `x /
/// (1 + Exp(-x))` and `x * Sigmoid(x)` are different binary32 functions.
pub const SILU_F32_FACT_EVALUATION_ORDER: AttributeFieldId = AttributeFieldId::new(3);
/// Fact field carrying the complete resolved accuracy contract of the subordinate exponential.
///
/// The canonical value of [`silu_f32_exponential_accuracy_contract`], written into
/// the definition's facts so that ADR 0016's requirement — transcendental accuracy
/// participates in semantic, plan, artifact, reference, and explain identity — is
/// satisfied by the registry's own definition projection rather than by a second
/// authority beside it.
pub const SILU_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT: AttributeFieldId = AttributeFieldId::new(4);
/// Fact field naming the operation's behaviour on binary32 subnormals.
///
/// **Measurement — the band is reached in two of the domain's three regions, and
/// each region is stated with the argument bits that bound it.** A policy
/// quantified over the whole domain cannot be justified from one region of it, and
/// the tail below is exactly the region an earlier spelling of this fact
/// generalized "unreachable" from.
///
/// **Reached as a *result*, near zero.** For `|x| <= 0x33000000` (`2^-25`) the
/// correctly rounded `e^-x` is exactly `1.0`: `e^-t > 1 - t` for `t > 0` puts it
/// strictly above `1 - 2^-25`, which is the rounding midpoint below `1.0`. So the
/// divisor is exactly `2.0` and the activation is `fl(x / 2)` over that whole
/// region. Every argument from `0x00000002` to `0x00fffffe` in magnitude therefore
/// has a subnormal result — including the *normal* arguments from `0x00800000` up,
/// so a subnormal result does not need a subnormal operand.
/// `silu(0x007fffff)` is `0x00400000` and `silu(0x00800000)` is `0x00400000`;
/// `silu(0x00fffffe)` is `0x007fffff`, the largest subnormal result, and its
/// successor `silu(0x00ffffff)` is `0x00800000`, back in the normal range. At the
/// other end `silu(0x00000001)` is `0x00000000` — round-to-nearest ties-to-even
/// landing on zero, not a flush.
///
/// **Reached as the subordinate exponential's own value, for large positive
/// arguments.** `e^-x` is subnormal from `0x42aeac50` (`87.3365478515625`) through
/// `0x42cff1b4` (`103.97207641601562`), and exactly `+0.0` above that. It is not
/// observable in the result — `fl(1 + subnormal)` is exactly `1.0`, so the
/// activation returns `x` unchanged — but it is a subnormal an arithmetic unit
/// produces, which is the site a flush policy acts on and not a value only read.
///
/// **Not reached in the large-negative tail.** The last argument with a finite
/// divisor is `0xc2b17217`, the negation of
/// [`SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`], and it gives `0x82b1726d` —
/// about `2.6e-37`, over twenty times the minimum normal `0x00800000`. Its
/// successor `0xc2b17218` overflows the exponential and gives exactly `0x80000000`.
/// The result drops from normal straight to `-0.0` with no subnormal band between
/// them, and no tail argument can reach one: a finite divisor is at most
/// `f32::MAX`, so the smallest tail magnitude is `88.72 / f32::MAX`, still normal.
pub const SILU_F32_FACT_SUBNORMALS: AttributeFieldId = AttributeFieldId::new(5);
/// Fact field naming the operation's behaviour on signed zero.
pub const SILU_F32_FACT_SIGNED_ZERO: AttributeFieldId = AttributeFieldId::new(6);
/// Fact field naming the operation's NaN behaviour.
pub const SILU_F32_FACT_NAN_BEHAVIOUR: AttributeFieldId = AttributeFieldId::new(7);
/// Fact field carrying the canonical arithmetic-NaN payload this operation installs.
pub const SILU_F32_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(8);
/// Fact field naming the operation's infinity and overflow behaviour.
///
/// Both infinities are stated because they are not symmetric: `silu(+inf)` is
/// `+inf` and `silu(-inf)` is a NaN, because `-inf / (1 + inf)` is `-inf / inf`.
/// The reference is not total on the extended reals and this records that rather
/// than repairing it.
pub const SILU_F32_FACT_INFINITY_AND_OVERFLOW: AttributeFieldId = AttributeFieldId::new(9);
/// Fact field stating whether ADR 0015's arithmetic contraction is permitted.
pub const SILU_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(10);
/// Fact field stating whether the division may be replaced by a reciprocal multiplication.
///
/// `false`, and it is a *withheld permission* rather than an absent one: replacing
/// `x / d` with `x * (1/d)` rounds twice where the pinned form rounds once, so a
/// realization that took it would compute a different binary32 function.
pub const SILU_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(11);
/// Fact field stating whether an approximate elementary intrinsic may realize the exponential.
///
/// `false`. The resolved accuracy contract is what the exponential must satisfy,
/// and an approximate intrinsic is admissible only through a stated envelope with
/// conformance evidence that it refines that contract — which is a resolution of
/// the caller's numerical contract, not a freedom this definition grants.
pub const SILU_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED: AttributeFieldId =
    AttributeFieldId::new(12);

/// Returns the governed elementwise binary32 `SiLU` operation key.
///
/// # Panics
///
/// Panics only if Tiler's compile-time governed key violates its own canonical
/// identity grammar.
#[must_use]
pub fn silu_f32_op() -> OpKey {
    OpKey::new("tiler", "silu-f32", 1).expect("the governed SiLU operation key is valid")
}

/// Returns the immutable reference semantics this key pins.
///
/// Names the division form explicitly, and names the three ADR 0024 rounding
/// boundaries, because a reference that said only "the `SiLU` activation" would
/// admit the sigmoid-product spelling that differs from it.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound registration would reject it under.
#[must_use]
pub fn silu_f32_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "tiler::silu-f32@1; y = x / (1 + Exp(-x)) over IEEE 754-2019 binary32, in that exact order; \
         the negation is exact sign manipulation, the addition and the division each round once \
         under round-to-nearest ties-to-even (ADR 0024); deliberately not x * Sigmoid(x), which is \
         a different binary32 function at 0xc2b00000",
    )
    .expect("the governed SiLU reference semantics are canonical")
}

/// Returns the immutable reference semantics of the subordinate exponential.
///
/// A separate reference from the activation's own, because the accuracy contract
/// resolves the exponential and a contract whose reference named the whole
/// activation would be stating a tolerance on a composition whose other two steps
/// ADR 0024 already fixes exactly.
///
/// # Panics
///
/// Panics only if this crate's own compile-time reference text violates the
/// canonical bound.
#[must_use]
pub fn silu_f32_exponential_reference_semantics() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new(
        "the natural exponential e^t on the reals, evaluated at t = -x as the one inexact \
         subordinate step of tiler::silu-f32@1; this is not a registered general Exp operation and \
         mints no key of its own",
    )
    .expect("the governed subordinate exponential reference is canonical")
}

/// Returns the resolved ADR 0042 accuracy contract of the subordinate exponential.
///
/// # The four decisions this function makes, each refutable on its own
///
/// **Form.** `BoundedPiecewise` with a single constant ULP clause. Metal's Table
/// 8.1 states a constant bound for the precise `exp`, which ADR 0042 routes to
/// this form; the *fast* family's input-dependent `3 + floor(fabs(2 * x))` would
/// route to a `NamedElementaryProfileKey` instead, and conflating the two is the
/// substitution ADR 0076 forbids.
///
/// **Metric and tolerance.** `tiler::ulp-reference-gap@1` at
/// [`SILU_F32_EXPONENTIAL_ULP_TOLERANCE`], derived rather than adopted; see that
/// constant.
///
/// **Domain.** Closed above at [`SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS`],
/// which is where the exponential's reference leaves binary32's finite range and
/// therefore where `tiler::ulp-reference-gap@1` stops being defined. `SiLU` reaches
/// those arguments — the whole `-88.73` band does — so the domain must exclude
/// them and let the [`FiniteOverflowRule`] carry them instead. The gap between
/// this bound and the real threshold contains no binary32 value, and the
/// exponential's argument here is always the negation of a binary32 operand, so
/// the exclusion loses no admissible input.
///
/// **Reference-result class.** `Positive`, justified by the exponential being
/// strictly positive on the reals. It is stated so a later relative clause over
/// the same region would be defined; it is not itself an accuracy claim.
///
/// # Panics
///
/// Panics only if this crate's own compile-time contract violates the grammar its
/// own vocabulary defines, which [`silu_f32_exponential_accuracy_contract`]'s
/// registration would reject as well.
#[must_use]
pub fn silu_f32_exponential_accuracy_contract() -> AccuracyContract {
    let ceiling =
        ExactRational::from_f32(f32::from_bits(SILU_F32_EXPONENTIAL_ARGUMENT_CEILING_BITS))
            .expect("the governed exponential ceiling is a finite binary32 value");
    let ordinary = DomainInterval::new(
        OperandOrdinal::new(0),
        DomainBound::Unbounded,
        DomainBound::Closed(ceiling),
    )
    .expect("the governed exponential domain admits every argument at or below its ceiling");
    let clause = AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), ordinary.clone())],
        ReferenceResultConstraint::new(
            [ReferenceResultClass::Positive],
            None,
            Some(
                NormativeDefinitionRef::new(
                    "e^t is strictly positive at every real t, so the reference result is never \
                     zero and never negative on this clause's whole region",
                )
                .expect("the governed positivity justification is canonical"),
            ),
        )
        .expect("the governed reference-result constraint is canonical"),
        AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(SILU_F32_EXPONENTIAL_ULP_TOLERANCE),
        ),
    )
    .expect("the governed exponential clause is canonical");
    AccuracyContract::new(
        silu_f32_op(),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        silu_f32_exponential_reference_semantics(),
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([ordinary], [clause])
                .expect("the governed exponential domain is canonical"),
        ),
        silu_f32_exponential_exceptional_contract(),
    )
}

/// Returns the subordinate exponential's independent exceptional-value contract.
///
/// Stated separately from the error metric and from the activation's own
/// exceptional behaviour, because ADR 0042 makes those three different claims and
/// `refines` refuses outright when two contracts state different ones — so a
/// realization must reproduce this record exactly rather than approximate it.
///
/// - a NaN argument has a NaN reference, and the operation installs its canonical
///   arithmetic NaN;
/// - an infinite reference is the infinity of its own sign, which is `+inf` for
///   `exp(+inf)`;
/// - the admitted ordinary domain is bounded above, and an argument beyond it is a
///   *finite overflow* rather than a domain error, so [`DomainErrorRule`] governs
///   only the arguments the operand type cannot produce;
/// - a finite reference above binary32's finite range yields `+inf`. This is the
///   route the `-88.73` band takes, and it is a positive claim: `1 + inf` is `inf`
///   and a finite negative divided by `inf` is exactly `-0.0`, with no rounding
///   and no flush anywhere in the chain.
#[must_use]
pub const fn silu_f32_exponential_exceptional_contract() -> ExceptionalValueContract {
    ExceptionalValueContract::new(
        NanReferenceRule::CanonicalNan,
        InfiniteReferenceRule::SignedInfinity,
        DomainErrorRule::CanonicalNan,
        FiniteOverflowRule::SignedInfinity,
    )
}

/// Returns the exact fact record the governed `SiLU` definition carries.
///
/// Built by the same constructor the registration uses rather than restated, so a
/// consumer parameterizing itself on the declared record and the registered
/// definition cannot disagree about what was declared.
///
/// # Panics
///
/// Panics only if this crate's own compile-time fact record violates the canonical
/// value grammar, which registration would reject as well.
#[must_use]
pub fn silu_f32_facts() -> CanonicalValue {
    CanonicalValue::record([
        CanonicalField::new(SILU_F32_FACT_COMPUTATION_TYPE, f32_value_type()),
        CanonicalField::new(SILU_F32_FACT_RESULT_TYPE, f32_value_type()),
        CanonicalField::new(
            SILU_F32_FACT_EVALUATION_ORDER,
            fact(
                "divide-x-by-one-plus-exp-of-negated-x; \
                 negation-exact-then-add-rounds-once-then-divide-rounds-once-under-ties-to-even",
            ),
        ),
        CanonicalField::new(
            SILU_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT,
            silu_f32_exponential_accuracy_contract()
                .to_canonical_value()
                .expect("the governed SiLU accuracy contract is canonical"),
        ),
        CanonicalField::new(
            SILU_F32_FACT_SUBNORMALS,
            fact(
                "preserved-by-this-contract-and-reached-as-a-result-near-zero-where-the-reference-\
                 is-x-over-two-and-as-the-subordinate-exponential-for-large-positive-arguments-\
                 and-flushed-on-a-declared-flushing-realization-a-recorded-divergence",
            ),
        ),
        CanonicalField::new(
            SILU_F32_FACT_SIGNED_ZERO,
            fact("ieee-754-signed-zero-rules-silu-of-negative-zero-is-negative-zero"),
        ),
        CanonicalField::new(
            SILU_F32_FACT_NAN_BEHAVIOUR,
            fact("quiet-nan-propagates-and-every-arithmetic-nan-result-is-canonicalized"),
        ),
        CanonicalField::new(
            SILU_F32_FACT_CANONICAL_NAN_BITS,
            super::registry::canonical_f32_bits(super::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        CanonicalField::new(
            SILU_F32_FACT_INFINITY_AND_OVERFLOW,
            fact("positive-infinity-maps-to-positive-infinity-and-negative-infinity-maps-to-nan"),
        ),
        CanonicalField::new(
            SILU_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            SILU_F32_FACT_RECIPROCAL_TRANSFORM_PERMITTED,
            CanonicalValue::boolean(false),
        ),
        CanonicalField::new(
            SILU_F32_FACT_APPROXIMATE_INTRINSIC_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
    .expect("the governed SiLU facts are canonical")
}

pub(super) fn register_standard_silu(
    registrar: &mut SemanticRegistryRegistrar<'_>,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        silu_f32_op(),
        OperationSchema::new(OperationArity::exact(1), OperationArity::exact(1), [])
            .expect("the governed SiLU operation schema is valid"),
        silu_f32_reference_semantics(),
        OperationDefinitionFacts::new(silu_f32_facts()),
        standard_conformance("silu-f32"),
        OperationEffect::Pure,
        Arc::new(SiluF32),
    ))
    // No algebraic capability is declared, and the absence is derived rather than
    // deferred. SiLU is neither associative nor commutative — it is unary — and
    // the ordered-associativity law `tiler::add-f32@1` declares has no meaning for
    // it. Declaring nothing reads as unknown rather than as the inverse law.
}

struct SiluF32;

impl OperationInferencer for SiluF32 {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        if operands.len() != 1 {
            return Err(op_error(
                "silu.f32.arity",
                "the binary32 SiLU activation requires exactly one operand",
            ));
        }
        if !request.attributes().fields().is_empty() {
            return Err(op_error(
                "silu.f32.attributes",
                "the binary32 SiLU activation has no attributes",
            ));
        }
        let expected = F32::resolved_type();
        if operands[0].resolved_type() != &expected {
            return Err(op_error(
                "silu.f32.implicit-promotion",
                "the binary32 SiLU activation admits no implicit promotion; an operand of another \
                 type is not converted to tiler::f32@1",
            ));
        }
        let shape = operands[0].shape().clone();
        outputs.try_push(ValueFact::new(expected, shape))
    }
}

fn f32_value_type() -> CanonicalValue {
    CanonicalValue::value_type(F32::resolved_type())
}

fn fact(value: &'static str) -> CanonicalValue {
    CanonicalValue::utf8(value).expect("the governed SiLU fact is bounded")
}

fn op_error(code: &str, message: &str) -> OperationInferenceError {
    OperationInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("the governed SiLU diagnostic code is canonical"),
        message,
    )
    .expect("the governed SiLU diagnostic message is canonical")
}

#[cfg(test)]
mod tests;
