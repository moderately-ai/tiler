//! Bounded conformance evidence for the binary32 `SiLU` reference.
//!
//! **Scope, stated exactly.** Every claim below is about the enumerated inputs and
//! about nothing else. There are 2^32 binary32 arguments; this corpus reaches
//! fourteen of them plus one contiguous 4,096-argument walk across the overflow
//! band, each chosen because it isolates one boundary of the pinned formula: the
//! two signed zeros, the exponential's overflow band, the two infinities, the
//! spelling divergence, and the subnormal question. An exhaustive claim over the
//! family would be
//! [`ExhaustiveFinite`](tiler_ir::semantic::accuracy::ConformanceEvidenceClass::ExhaustiveFinite)
//! evidence and would need its own harness and budget; nothing here establishes
//! one, and nothing here is a claim about any target.

use tiler_ir::semantic::accuracy::{AccuracyContract, ExactRational, UlpFormat};
use tiler_ir::semantic::{
    CanonicalField, CanonicalValueView, F32, FrozenSemanticRegistry,
    SILU_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT, builtin_scalar_value_type_facts, silu_f32_op,
};

use super::{
    EXPONENTIAL_OVERFLOW_GUARD, EXPONENTIAL_UNDERFLOW_GUARD, certified_exp_f32, rounds_to, silu_f32,
};
use crate::accuracy::{
    ConformanceDecision, EnclosureError, EnclosurePrecision, decide_contract, exp_enclosure,
};
use crate::canonicalize_arithmetic_f32;
use crate::error::ReferenceOperationError;

/// The boundary corpus, with every argument written as its exact bit pattern.
///
/// Bit patterns rather than decimal literals because the band boundary is
/// narrower than a decimal reader would guess: `-88.7228` is `0xc2b17213` and
/// `-88.73` is `0xc2b175c3`, and the last normal result and the first exact
/// negative zero lie between them.
const BOUNDARY_CORPUS: &[(u32, u32, &str)] = &[
    (0x0000_0000, 0x0000_0000, "positive zero is preserved"),
    (0x8000_0000, 0x8000_0000, "negative zero is preserved"),
    (0x7f80_0000, 0x7f80_0000, "positive infinity maps to itself"),
    (
        0xff80_0000,
        0x7fc0_0000,
        "negative infinity maps to a NaN; the reference is not total on the extended reals",
    ),
    (
        0xc2b0_0000,
        0x8335_4ddc,
        "-88.0: the division form, one ULP from the sigmoid-product spelling",
    ),
    (
        0xc2b1_7213,
        0x82b1_73cc,
        "-88.7228: a normal result close to the band",
    ),
    (
        0xc2b1_7217,
        0x82b1_726d,
        "-88.722832: the last argument whose exponential is still finite",
    ),
    (
        0xc2b1_75c3,
        0x8000_0000,
        "-88.73: inside the band, exactly negative zero",
    ),
    (
        0xc2b1_7d71,
        0x8000_0000,
        "-88.745: further inside the band, still exactly negative zero",
    ),
    (0x3f80_0000, 0x3f3b_26a8, "1.0, an ordinary interior point"),
    (0xbf80_0000, 0xbe89_b2b1, "-1.0, an ordinary interior point"),
    (
        0x0000_0001,
        0x0000_0000,
        "the least positive subnormal argument halves to positive zero",
    ),
    (
        0x8000_0001,
        0x8000_0000,
        "the greatest negative subnormal argument halves to negative zero",
    ),
    (
        0x7f7f_ffff,
        0x7f7f_ffff,
        "f32::MAX is its own activation, because its divisor rounds to 1.0",
    ),
    (
        0xff7f_ffff,
        0x8000_0000,
        "-f32::MAX is far inside the band, exactly negative zero",
    ),
];

fn silu_bits(argument_bits: u32) -> u32 {
    canonicalize_arithmetic_f32(
        silu_f32(f32::from_bits(argument_bits)).expect("the corpus argument is decidable"),
    )
    .to_bits()
}

/// Every boundary-corpus argument reproduces its pinned result exactly.
#[test]
fn the_boundary_corpus_reproduces_its_pinned_bit_patterns() {
    for (argument, expected, what) in BOUNDARY_CORPUS {
        assert_eq!(
            silu_bits(*argument),
            *expected,
            "silu({argument:#010x}) — {what}"
        );
    }
}

/// The band's negative zero comes from an overflowed exponential, not a flush.
///
/// The route is asserted rather than described: the exponential of the negated
/// argument **overflows** to `+inf`, so the divisor is `+inf`, and a finite
/// negative divided by an infinity is exactly `-0.0` by IEEE sign rules. Nothing
/// subnormal occurs anywhere in the chain, so no subnormal policy — of Tiler's or
/// of a target's — acts on this value.
#[test]
fn the_negative_band_is_an_exact_negative_zero_from_an_overflowed_exponential() {
    for argument_bits in [0xc2b1_75c3_u32, 0xc2b1_7d71, 0xc3b1_7218, 0xff7f_ffff] {
        let argument = f32::from_bits(argument_bits);
        assert_eq!(
            certified_exp_f32(-argument),
            Ok(f32::INFINITY),
            "the exponential overflows at {argument_bits:#010x}"
        );
        assert_eq!(
            silu_f32(argument).expect("decidable").to_bits(),
            0x8000_0000,
            "silu({argument_bits:#010x}) is exactly negative zero"
        );
    }
    // Immediately above the band the exponential is still finite, so the two
    // regions are distinguished by the overflow rather than by a threshold this
    // test chose.
    assert!(
        certified_exp_f32(-f32::from_bits(0xc2b1_7217))
            .expect("decidable")
            .is_finite()
    );
}

/// No binary32 `SiLU` argument in the band produces a subnormal result.
///
/// **Bounded, and the bound is the claim.** This walks 4,096 contiguous binary32
/// arguments from the last one whose exponential is finite, which is the only
/// region where a subnormal result could arise at all, and finds the magnitude
/// stepping from a normal value straight to zero. Outside this interval the
/// divisor is at most `2`, so the result's magnitude is at least half the
/// argument's and a subnormal result would need a subnormal argument — which
/// halves to a zero, as the corpus rows for `0x00000001` and `0x80000001` pin.
#[test]
fn the_band_produces_no_subnormal_result() {
    let least_normal = f32::from_bits(0x0080_0000);
    let mut reached_zero = false;
    let mut last_normal_magnitude = f32::INFINITY;
    for offset in 0..4_096_u32 {
        let argument = f32::from_bits(0xc2b1_7217 + offset);
        let result = silu_f32(argument).expect("decidable");
        let magnitude = result.abs();
        assert!(
            magnitude == 0.0 || magnitude >= least_normal,
            "silu({:#010x}) = {:#010x} is subnormal",
            argument.to_bits(),
            result.to_bits()
        );
        if magnitude == 0.0 {
            assert_eq!(result.to_bits(), 0x8000_0000, "the band's zero is negative");
            reached_zero = true;
        } else {
            assert!(!reached_zero, "the band is contiguous");
            last_normal_magnitude = magnitude;
        }
    }
    assert!(reached_zero, "the walk reached the band");
    assert!(
        last_normal_magnitude > least_normal * 20.0,
        "the last normal result is more than twenty times the minimum normal, so the drop to zero \
         skips the subnormal range entirely rather than passing through it"
    );
}

/// The two conventional spellings are different binary32 functions.
///
/// **Measurement — where they diverge, and where they do not.** Over this corpus
/// the two agree at every ordinary interior argument and differ at exactly three,
/// all in the deep-negative tail: `-88.0` by one ULP (`0x83354ddc` against
/// `0x83354ddb`, the divergence the L3′ probe pins) and the two arguments just
/// above the overflow band by three ULPs each. That is where the difference must
/// live: the product form rounds twice — once at the reciprocal and once at the
/// multiply — where the division form rounds once, and the reciprocal's own
/// rounding is worth the most when `1 + exp(-x)` is large.
///
/// So a corpus without an argument near the exponential's overflow threshold
/// reports the two spellings identical, which is exactly how a key admitted under
/// one would silently deliver the other. Both values are computed with the *same*
/// certified exponential, so the difference is the spelling and not two libraries
/// disagreeing.
#[test]
fn the_sigmoid_product_spelling_differs_at_the_pinned_input() {
    let sigmoid_product = |argument: f32| -> f32 {
        let exponential = certified_exp_f32(-argument).expect("decidable");
        argument * (1.0_f32 / (1.0_f32 + exponential))
    };
    let divergent = f32::from_bits(0xc2b0_0000);
    assert_eq!(
        silu_f32(divergent).expect("decidable").to_bits(),
        0x8335_4ddc
    );
    assert_eq!(sigmoid_product(divergent).to_bits(), 0x8335_4ddb);

    let mut divergences = Vec::new();
    let mut compared = 0_usize;
    for (argument, _, _) in BOUNDARY_CORPUS {
        let argument = f32::from_bits(*argument);
        if !argument.is_finite() {
            continue;
        }
        compared += 1;
        let division = silu_f32(argument).expect("decidable").to_bits();
        let product = sigmoid_product(argument).to_bits();
        if division != product {
            divergences.push((argument.to_bits(), division, product));
        }
    }
    assert_eq!(compared, 13, "every finite corpus argument was compared");
    assert_eq!(
        divergences,
        vec![
            (0xc2b0_0000, 0x8335_4ddc, 0x8335_4ddb),
            (0xc2b1_7213, 0x82b1_73cc, 0x82b1_73cf),
            (0xc2b1_7217, 0x82b1_726d, 0x82b1_7270),
        ],
        "the divergence set is exactly these three arguments, all in the deep-negative tail"
    );
}

/// Every argument the guards hand to the enclosure is inside its magnitude bound.
///
/// `certified_exp_f32` refuses nothing of its own: it decides `+inf` at or above
/// [`EXPONENTIAL_OVERFLOW_GUARD`], `+0.0` at or below
/// [`EXPONENTIAL_UNDERFLOW_GUARD`], and hands everything strictly between to
/// [`exp_enclosure`], which refuses an argument whose exponential would exceed its
/// governed result magnitude. That refusal is a bound on the *magnitude*, so
/// admitting the greatest magnitude the guards can pass admits every argument they
/// pass — and the two representable extremes are checked rather than argued about.
///
/// Worth a test rather than a sentence because the two sides move independently: a
/// guard widened outward, or the budget narrowed, turns an argument the format
/// reaches into
/// [`ReferenceOperationError::UndecidedTranscendentalReference`], which is a
/// refusal a caller cannot act on rather than a bound it can.
#[test]
fn every_argument_the_guards_admit_is_inside_the_enclosure_bound() {
    let greatest = f32::from_bits(EXPONENTIAL_OVERFLOW_GUARD.to_bits() - 1);
    let least = -f32::from_bits((-EXPONENTIAL_UNDERFLOW_GUARD).to_bits() - 1);
    assert!(
        greatest < EXPONENTIAL_OVERFLOW_GUARD && least > EXPONENTIAL_UNDERFLOW_GUARD,
        "both extremes must sit strictly inside the guards that select them"
    );
    let mut checked = 0_usize;
    for argument in [greatest, least] {
        let exact = ExactRational::from_f32(argument).expect("finite");
        assert!(
            exp_enclosure(&exact, EnclosurePrecision::binary32_corpus()).is_ok(),
            "the enclosure must bracket {:#010x}, which the guards admit",
            argument.to_bits()
        );
        assert!(
            certified_exp_f32(argument).is_ok(),
            "and the reference must decide {:#010x} rather than reporting undecided",
            argument.to_bits()
        );
        checked += 1;
    }
    assert_eq!(checked, 2, "both extremes were checked");

    // The refusal is reachable, and only from outside the guards: the corpus's own
    // widest argument is past the budget, and the underflow guard decides it before
    // the enclosure is ever asked.
    assert_eq!(
        exp_enclosure(
            &ExactRational::from_f32(-f32::MAX).expect("finite"),
            EnclosurePrecision::binary32_corpus(),
        )
        .as_ref()
        .err()
        .map(EnclosureError::diagnostic_code),
        Some("reference.enclosure.argument-too-large")
    );
    assert_eq!(certified_exp_f32(-f32::MAX), Ok(0.0));
}

/// The certified exponential admits only a provably correctly rounded value.
#[test]
fn the_certified_exponential_agrees_with_the_pinned_value_at_one() {
    assert_eq!(certified_exp_f32(1.0), Ok(f32::from_bits(0x402d_f854)));
    assert_eq!(certified_exp_f32(0.0), Ok(1.0));
}

/// The rounding decision refuses a candidate that is not correctly rounded.
///
/// This is the perturbation that proves the certification can say no: both
/// immediate neighbours of the true value are rejected by the same predicate that
/// accepts it, so the acceptance is a property of the enclosure rather than of the
/// candidate having been produced by a library.
#[test]
fn a_candidate_off_by_one_ulp_is_not_admitted_as_correctly_rounded() {
    let enclosure = exp_enclosure(
        &ExactRational::from_f32(1.0).expect("finite"),
        EnclosurePrecision::binary32_corpus(),
    )
    .expect("bracketed");
    let exact = 0x402d_f854_u32;
    assert!(rounds_to(&enclosure, f32::from_bits(exact)));
    assert!(!rounds_to(&enclosure, f32::from_bits(exact + 1)));
    assert!(!rounds_to(&enclosure, f32::from_bits(exact - 1)));
}

/// A grid too coarse to separate the neighbours yields no decision at all.
///
/// The enclosure widens, `rounds_to` stops admitting the true value, and the only
/// thing the module can then report is
/// [`ReferenceOperationError::UndecidedTranscendentalReference`]. A reference that
/// resolved this toward the nearer side would be one that cannot fail.
#[test]
fn a_coarse_enclosure_decides_nothing_rather_than_guessing() {
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    let coarse = exp_enclosure(
        &ExactRational::from_f32(1.0).expect("finite"),
        EnclosurePrecision::new(4),
    )
    .expect("bracketed");
    assert!(
        coarse.width() > format.ulp_scale(coarse.lower()).expect("in range"),
        "a four-bit grid is coarser than one binary32 ULP at e"
    );
    assert!(
        !rounds_to(&coarse, f32::from_bits(0x402d_f854)),
        "the coarse bracket does not establish the correctly rounded value"
    );
    assert_eq!(
        ReferenceOperationError::UndecidedTranscendentalReference.to_string(),
        "the certified enclosure does not establish which value the transcendental reference \
         rounds to"
    );
}

// ---------------------------------------------------------------------------
// Reference-evaluating the resolved accuracy contract
// ---------------------------------------------------------------------------

/// The registered contract is reference-evaluated, end to end, at real inputs.
///
/// Milestone 1 forbids admitting a transcendental "before its accuracy contract
/// is canonically serialized and reference-evaluated end to end", and this is the
/// second half of that. The contract is decoded from the *registered definition's
/// own facts* rather than reconstructed, and then decided against a certified
/// enclosure at each corpus argument. Nothing here trusts a host library: the
/// enclosure is exact rational arithmetic and the decision is a three-way
/// comparison that can answer `Undecided`.
#[test]
fn the_registered_contract_decides_conformance_against_a_certified_enclosure() {
    let contract = registered_contract();
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    contract
        .verify(&facts)
        .expect("the registered contract verifies");

    let mut decided = 0_usize;
    for (argument_bits, _, _) in BOUNDARY_CORPUS {
        let argument = f32::from_bits(*argument_bits);
        // The contract resolves the *subordinate exponential*, whose argument is
        // the negated activation operand and whose ordinary domain excludes the
        // overflow region, so the corpus rows inside the band are the
        // finite-overflow contract's subject rather than this clause's.
        if !argument.is_finite() {
            continue;
        }
        let exponent_argument = -argument;
        // Two exclusions, and each names a different boundary. Above the
        // contract's ceiling the reference overflows and the finite-overflow rule
        // governs instead of this clause. Below `-104` the reference is smaller
        // than half the least subnormal, so `certified_exp_f32`'s underflow guard
        // answers `+0.0` without consulting an enclosure at all — and the corpus
        // argument that reaches this branch is `-f32::MAX`, whose exponential is
        // past the enclosure's own result-magnitude budget and is refused there
        // too. The contract still holds in that region; this *evaluator* does not
        // decide it, and saying so is the point.
        if exponent_argument > f32::from_bits(0x42b1_7217) || exponent_argument < -104.0 {
            continue;
        }
        let exact = ExactRational::from_f32(exponent_argument).expect("finite");
        let enclosure = exp_enclosure(&exact, EnclosurePrecision::binary32_corpus())
            .expect("the corpus argument is bracketed");
        let candidate = certified_exp_f32(exponent_argument).expect("decidable");
        let candidate = ExactRational::from_f32(candidate).expect("finite");
        assert_eq!(
            decide_contract(&contract, &format, &[exact], &enclosure, &candidate),
            ConformanceDecision::Conforms,
            "the correctly rounded exponential conforms at {argument_bits:#010x}"
        );
        decided += 1;
    }
    assert!(
        decided >= 6,
        "the corpus reached the ordinary domain at {decided} arguments, which is the population \
         this claim covers"
    );
}

/// A candidate beyond the stated tolerance is refused, at a named input.
///
/// The check that the check can say no. Thirteen ULPs is one past the contract's
/// twelve, and the same decision path that admits the correctly rounded value
/// rejects it — so `Conforms` above is a property of the bound rather than of the
/// comparison being unable to fail.
#[test]
fn a_candidate_beyond_the_stated_tolerance_violates_the_registered_contract() {
    let contract = registered_contract();
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    let exact = ExactRational::from_f32(1.0).expect("finite");
    let enclosure =
        exp_enclosure(&exact, EnclosurePrecision::binary32_corpus()).expect("bracketed");
    let correct = 0x402d_f854_u32;
    for (offset, expected) in [
        (0_u32, ConformanceDecision::Conforms),
        (12, ConformanceDecision::Conforms),
        (13, ConformanceDecision::Violates),
        (64, ConformanceDecision::Violates),
    ] {
        let candidate = ExactRational::from_f32(f32::from_bits(correct + offset)).expect("finite");
        assert_eq!(
            decide_contract(
                &contract,
                &format,
                std::slice::from_ref(&exact),
                &enclosure,
                &candidate
            ),
            expected,
            "a candidate {offset} ULPs above the reference"
        );
    }
}

/// Returns the accuracy contract as the semantic registry actually carries it.
///
/// Decoded from the registered definition's facts rather than rebuilt, so this
/// evaluates what a consumer would read rather than what this crate would like it
/// to be.
fn registered_contract() -> AccuracyContract {
    let registry = FrozenSemanticRegistry::standard().expect("the governed registry composes");
    let facts = registry
        .operation_facts(&silu_f32_op())
        .expect("the activation carries facts");
    let CanonicalValueView::Record(fields) = facts.value().view() else {
        panic!("a governed fact record is a record");
    };
    let carried = fields
        .iter()
        .find(|field| field.id() == SILU_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT)
        .map(CanonicalField::value)
        .expect("the facts carry the accuracy contract");
    AccuracyContract::from_canonical_value(carried).expect("the carried contract decodes")
}
