use super::*;

use tiler_ir::semantic::accuracy::{ExactRational, UlpFormat};
use tiler_ir::semantic::{
    F32Softmax, InputKey, OutputKey, SemanticProgramBuilder, builtin_scalar_value_type_facts,
    softmax_f32_exponential_accuracy_contract, softmax_f32_op,
};

use crate::accuracy::{
    ConformanceDecision, EnclosurePrecision, decide_contract, exact_binary32_candidate,
    exp_enclosure,
};
use crate::evaluate::{ReferenceEvaluator, decode_f32, f32_element, f32_elements};
use crate::tensor::InputBinding;

/// The reference model's causal-mask fill, `torch.finfo(f32).min`.
const MASK_FILL_BITS: u32 = 0xff7f_ffff;

/// The canonical arithmetic NaN every arithmetic result is canonicalized to.
const CANONICAL_NAN: u32 = 0x7fc0_0000;

/// The reference model's implied normalization constant at the retained worked example.
///
/// One ULP below the correctly rounded `1.0 / d` over the pinned strict left fold,
/// and the whole of the divergence this corpus records. It is not an approximate
/// reciprocal: it is the correctly rounded reciprocal of
/// [`REORDERED_DENOMINATOR`], which the same row's exponentials reach under
/// another contributor order. Named so that every place it appears points at the
/// same measurement rather than at a literal.
const REFERENCE_MODEL_WORKED_EXAMPLE_CONSTANT: u32 = 0x3f2a_4d3a;

/// The worked example's denominator under the contributor order `(e₀, e₂, e₁, e₃)`.
///
/// One ULP above the strict left fold's `0x3fc06957`, and the denominator whose
/// correctly rounded reciprocal is what the reference model multiplied the row by.
const REORDERED_DENOMINATOR: u32 = 0x3fc0_6958;

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a corpus shape is bounded")
}

/// Runs the reference over one row, returning the exact result payloads.
fn softmax(dims: &[u64], axis: u32, values: &[f32]) -> Vec<u32> {
    softmax_f32(&shape(dims), Axis::new(axis), values)
        .expect("a corpus row evaluates")
        .into_iter()
        .map(f32::to_bits)
        .collect()
}

/// The strict left-fold sum of a row, which is the order the identity names.
fn strict_sum(bits: &[u32]) -> u32 {
    let mut total = f32::from_bits(bits[0]);
    for value in &bits[1..] {
        total += f32::from_bits(*value);
    }
    total.to_bits()
}

// ---------------------------------------------------------------------------
// The extrema family, and decision D-2
// ---------------------------------------------------------------------------

/// The reference maximum is the propagating family, not the host's.
///
/// **Rust's `f32::max` is `maxNum`**, the *other* ADR 0023 family: it returns the
/// numeric operand when exactly one operand is NaN. Writing it in the reference
/// would have installed `MaximumNumber` under the name of `Maximum`, and this is
/// the row that separates them. The signed-zero rows separate them from a naive
/// `if a > b` as well, which returns `b` for both orders of `(+0.0, -0.0)`.
#[test]
fn the_reference_maximum_is_not_the_host_maximum() {
    let nan = f32::NAN;
    assert!(maximum_f32(nan, 1.0).is_nan());
    assert!(maximum_f32(1.0, nan).is_nan());
    assert!(maximum_f32(nan, nan).is_nan());
    // The host disagrees at exactly the rows the families are defined to differ
    // on, which is what makes this a discriminating check rather than a restated
    // definition. Compared as payloads throughout this corpus, because what the
    // family decides *is* a bit pattern: the opposite-zero rows below are equal
    // as values and different as answers.
    assert_eq!(nan.max(1.0).to_bits(), 1.0_f32.to_bits());
    assert_eq!(1.0_f32.max(nan).to_bits(), 1.0_f32.to_bits());

    // `-0.0 < +0.0`, deterministically and in both operand orders. Metal's `fmax`
    // is the construct that does *not* promise this.
    assert_eq!(maximum_f32(0.0, -0.0).to_bits(), 0x0000_0000);
    assert_eq!(maximum_f32(-0.0, 0.0).to_bits(), 0x0000_0000);
    assert_eq!(maximum_f32(-0.0, -0.0).to_bits(), 0x8000_0000);
    assert_eq!(maximum_f32(0.0, 0.0).to_bits(), 0x0000_0000);

    // Ordinary and infinite operands, both orders.
    assert_eq!(maximum_f32(1.0, 2.0).to_bits(), 2.0_f32.to_bits());
    assert_eq!(maximum_f32(2.0, 1.0).to_bits(), 2.0_f32.to_bits());
    assert_eq!(
        maximum_f32(f32::INFINITY, 1.0).to_bits(),
        f32::INFINITY.to_bits()
    );
    assert_eq!(
        maximum_f32(f32::NEG_INFINITY, -1.0).to_bits(),
        (-1.0_f32).to_bits()
    );
    assert_eq!(
        maximum_f32(f32::NEG_INFINITY, f32::NEG_INFINITY).to_bits(),
        f32::NEG_INFINITY.to_bits()
    );
}

/// The pinned family is associative and commutative over every corpus operand.
///
/// This is the legality claim `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` states,
/// executed: every tree over the same contributors gives the same bits, so the
/// maximum pass may be reassociated and permuted with no permission. The corpus
/// deliberately contains both zeros, both infinities, and a NaN, because those
/// are the only operands at which the property could fail.
#[test]
fn the_pinned_extrema_family_is_associative_and_commutative_on_every_operand() {
    let corpus = [
        0.0_f32,
        -0.0,
        1.0,
        -1.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
    ];
    let same = |left: f32, right: f32| left.to_bits() == right.to_bits();
    for a in corpus {
        for b in corpus {
            assert!(
                same(maximum_f32(a, b), maximum_f32(b, a)),
                "commutativity fails at ({a:?}, {b:?})"
            );
            for c in corpus {
                let left = maximum_f32(maximum_f32(a, b), c);
                let right = maximum_f32(a, maximum_f32(b, c));
                assert!(
                    same(left, right),
                    "associativity fails at ({a:?}, {b:?}, {c:?})"
                );
            }
        }
    }
    // The control: the number-preferring family and a bare `>` comparison are
    // *both* different functions on this corpus, so the assertions above are
    // about this family rather than about any maximum.
    assert!(corpus.iter().any(|a| {
        corpus
            .iter()
            .any(|b| maximum_f32(*a, *b).to_bits() != maximum_number_f32(*a, *b).to_bits())
    }));
}

/// The number-preferring family, defined here only to be compared against.
///
/// `MaximumNumber` as ADR 0023 states it, with the same deterministic zero
/// ordering. It is test-local rather than exported because
/// `tiler::softmax-f32@1` does not use it and exporting an unused extrema family
/// would read as an admitted operation.
fn maximum_number_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() && right.is_nan() {
        return f32::NAN;
    }
    if left.is_nan() {
        return right;
    }
    if right.is_nan() {
        return left;
    }
    #[allow(
        clippy::float_cmp,
        reason = "the extrema family is defined by exact IEEE-754 comparison, exactly as the propagating one this is compared against is"
    )]
    let equal = left == right;
    if equal {
        return f32::from_bits(left.to_bits() & right.to_bits());
    }
    if left > right { left } else { right }
}

/// The two extrema families are indistinguishable through the pinned formula.
///
/// **This is decision D-2's theorem, executed.** The families differ only on a row
/// containing a NaN, and on such a row the sum's `Add` propagates the NaN
/// unconditionally — so the denominator is NaN, the reciprocal is NaN, and every
/// output is NaN under either family. The corpus below evaluates each row *twice*,
/// once with each family, and asserts bit equality; the control is that the two
/// families genuinely disagree about the row *maximum* on the same inputs, so the
/// equality is a property of the composition rather than of two identical
/// functions.
#[test]
fn the_two_extrema_families_are_indistinguishable_through_the_pinned_formula() {
    let rows: [&[f32]; 5] = [
        &[1.0, f32::NAN, 3.0],
        &[f32::NAN, 1.0],
        &[f32::NAN, f32::NAN],
        &[1.0, 2.0, 3.0],
        &[0.0, -0.0],
    ];
    let mut maxima_disagreed = false;
    for row in rows {
        let propagating = softmax_with_family(row, maximum_f32);
        let preferring = softmax_with_family(row, maximum_number_f32);
        assert_eq!(
            propagating, preferring,
            "the two families must agree through the whole formula on {row:?}"
        );
        // The registered reference is the propagating one, and it agrees.
        assert_eq!(propagating, softmax(&[1, row.len() as u64], 1, row));

        let mut strict = row[0];
        let mut lenient = row[0];
        for value in &row[1..] {
            strict = maximum_f32(strict, *value);
            lenient = maximum_number_f32(lenient, *value);
        }
        maxima_disagreed |= strict.to_bits() != lenient.to_bits();
    }
    assert!(
        maxima_disagreed,
        "the corpus must contain a row whose two maxima differ, or the equality above is vacuous"
    );
}

/// Evaluates the pinned formula over one row under a chosen extrema family.
///
/// A second evaluator only for the row maximum, so the family can be varied while
/// every other step stays the registered one. It is not a second copy of the
/// operation: everything after the maximum is written once here and once in
/// `row_softmax`, and the test above asserts the registered path agrees.
fn softmax_with_family(row: &[f32], family: fn(f32, f32) -> f32) -> Vec<u32> {
    let mut maximum = row[0];
    for value in &row[1..] {
        maximum = family(maximum, *value);
    }
    let exponentials: Vec<f32> = row
        .iter()
        .map(|score| certified_exp_f32(score - maximum).expect("the corpus decides"))
        .collect();
    let mut denominator = exponentials[0];
    for value in &exponentials[1..] {
        denominator += *value;
    }
    let reciprocal = 1.0_f32 / denominator;
    exponentials
        .iter()
        .map(|value| canonicalize_arithmetic_f32(value * reciprocal).to_bits())
        .collect()
}

/// A NaN score poisons its whole row, which is what the NaN fact states.
#[test]
fn a_single_nan_score_poisons_the_whole_row() {
    assert_eq!(
        softmax(&[1, 3], 1, &[1.0, f32::NAN, 3.0]),
        vec![CANONICAL_NAN; 3]
    );
    // The control: the same row without the NaN is finite everywhere, so the
    // poisoning is caused by the NaN rather than by the row's shape.
    let clean = softmax(&[1, 3], 1, &[1.0, 2.0, 3.0]);
    assert!(clean.iter().all(|bits| f32::from_bits(*bits).is_finite()));
}

// ---------------------------------------------------------------------------
// The resolved accuracy contract
// ---------------------------------------------------------------------------

/// The registered ULP contract admits the certified value and refuses a distant one.
///
/// Decided against a certified enclosure rather than compared to a constant. The
/// argument is `-2.0`, which the worked example reaches, and the third row is the
/// one that makes the bound a bound: a candidate twelve ULP away conforms and one
/// thirteen ULP away does not.
#[test]
fn the_registered_contract_admits_the_certified_value_and_bounds_the_error() {
    let contract = registered_contract();
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    contract
        .verify(&facts)
        .expect("the registered contract verifies");

    let argument = -2.0_f32;
    let enclosure = exp_enclosure(
        &ExactRational::from_f32(argument).expect("finite"),
        EnclosurePrecision::binary32_corpus(),
    )
    .expect("bracketed");
    let inputs = [ExactRational::from_f32(argument).expect("finite")];
    let decide = |bits: u32| {
        let candidate = exact_binary32_candidate(f32::from_bits(bits)).expect("finite");
        decide_contract(&contract, &format, &inputs, &enclosure, &candidate)
    };
    let certified = certified_exp_f32(argument).expect("decided");
    assert_eq!(certified.to_bits(), 0x3e0a_9555);
    assert_eq!(decide(certified.to_bits()), ConformanceDecision::Conforms);
    assert_eq!(
        decide(certified.to_bits() + 12),
        ConformanceDecision::Conforms
    );
    assert_eq!(
        decide(certified.to_bits() + 13),
        ConformanceDecision::Violates
    );
}

/// Decoded from the registered definition's own facts, never reconstructed.
fn registered_contract() -> tiler_ir::semantic::accuracy::AccuracyContract {
    use tiler_ir::semantic::{CanonicalValueView, FrozenSemanticRegistry};
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&softmax_f32_op())
        .expect("the softmax is registered");
    let CanonicalValueView::Record(fields) = definition.canonical_facts().value().view() else {
        panic!("the fact record is a record");
    };
    let carried = fields
        .iter()
        .find(|field| {
            field.id() == tiler_ir::semantic::SOFTMAX_F32_FACT_EXPONENTIAL_ACCURACY_CONTRACT
        })
        .expect("the accuracy-contract fact is registered");
    let decoded =
        tiler_ir::semantic::accuracy::AccuracyContract::from_canonical_value(carried.value())
            .expect("the registered contract decodes");
    assert_eq!(decoded, softmax_f32_exponential_accuracy_contract());
    decoded
}

// ---------------------------------------------------------------------------
// The bounded conformance corpus
// ---------------------------------------------------------------------------

/// The derivation's retained worked example, and the divergence it exposes.
///
/// **The pinned formula and the reference model disagree at this row, and the
/// disagreement is one constant.** Scores `[1.0, 2.0, 3.0, mask]`. Every
/// intermediate the L3′ record states is reproduced exactly — the maximum, the
/// shifted scores, the four exponentials including the masked position's exact
/// `+0.0`, and the denominator `0x3fc06957`. From there the record's recorded
/// outputs require a constant of `0x3f2a4d3a`, while the correctly rounded
/// `1.0 / d` is `0x3f2a4d3b`; the record's row-sum of `0x3f7ffffe` follows from
/// the same one-ULP-low constant, and under the pinned formula the row sums to
/// exactly `0x3f800000`.
///
/// **The constant is a reordered sum, not an approximate reciprocal, and this
/// test executes that attribution rather than narrating it.** `0x3f2a4d3a` is the
/// correctly rounded reciprocal of `0x3fc06958`, which these same four
/// exponentials reach under the contributor order `(e₀, e₂, e₁, e₃)` — asserted
/// below from the exponentials themselves, so a claim about the reference model's
/// arithmetic is decided by arithmetic here. The reference is therefore evaluating
/// the pinned *formula* over a permuted contributor sequence, which is exactly the
/// freedom `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` withholds: reproducing these bits
/// would be performing an unpermitted reassociation, not passing a check.
///
/// **Measured, not inferred.** In the retained probe's own pinned environment,
/// `torch.nn.functional.softmax` on this row returns the record's bits, while
/// both `e * (1/d)` and `e / d` computed from the reference's *own* `e` and `d`
/// return this reference's bits. Recorded here rather than tuned away.
#[test]
fn the_retained_worked_example_reproduces_the_pinned_formula() {
    let mask = f32::from_bits(MASK_FILL_BITS);
    let result = softmax(&[1, 4], 1, &[1.0, 2.0, 3.0, mask]);
    assert_eq!(
        result,
        vec![0x3db8_61f3, 0x3e7a_9a1a, 0x3f2a_4d3b, 0x0000_0000]
    );

    // The intermediates, in the order the reference states them. Every one of
    // these is the record's own recorded value.
    let maximum = maximum_f32(maximum_f32(maximum_f32(1.0, 2.0), 3.0), mask);
    assert_eq!(maximum.to_bits(), 0x4040_0000);
    let shifted = [
        1.0_f32 - maximum,
        2.0 - maximum,
        3.0 - maximum,
        mask - maximum,
    ];
    assert_eq!(
        shifted.map(f32::to_bits),
        [0xc000_0000, 0xbf80_0000, 0x0000_0000, MASK_FILL_BITS]
    );
    let exponentials = shifted.map(|value| certified_exp_f32(value).expect("decided"));
    assert_eq!(
        exponentials.map(f32::to_bits),
        [0x3e0a_9555, 0x3ebc_5ab2, 0x3f80_0000, 0x0000_0000],
        "the masked position contributes exactly +0.0"
    );
    let denominator = exponentials[0] + exponentials[1] + exponentials[2] + exponentials[3];
    assert_eq!(denominator.to_bits(), 0x3fc0_6957);

    // The one step at which this reference and the reference model part company.
    let reciprocal = 1.0_f32 / denominator;
    assert_eq!(reciprocal.to_bits(), 0x3f2a_4d3b);
    assert_ne!(
        reciprocal.to_bits(),
        REFERENCE_MODEL_WORKED_EXAMPLE_CONSTANT
    );
    assert_eq!(
        reciprocal.to_bits() - 1,
        REFERENCE_MODEL_WORKED_EXAMPLE_CONSTANT,
        "the reference model's implied constant is exactly one ULP below the correctly rounded reciprocal"
    );
    // The record's own recorded outputs are what that constant produces, which is
    // how this test shows the divergence is one scalar rather than the exponential
    // or the individual products.
    let model = f32::from_bits(REFERENCE_MODEL_WORKED_EXAMPLE_CONSTANT);
    assert_eq!(
        exponentials
            .iter()
            .map(|value| (value * model).to_bits())
            .collect::<Vec<u32>>(),
        vec![0x3db8_61f2, 0x3e7a_9a18, 0x3f2a_4d3a, 0x0000_0000]
    );

    // And where that scalar comes from: the same four exponentials summed in the
    // order `(e₀, e₂, e₁, e₃)` give a denominator one ULP above the strict left
    // fold's, whose *correctly rounded* reciprocal is the reference model's
    // constant exactly. So the divergence is the sum's contributor order — the
    // freedom `SOFTMAX_F32_FACT_SUM_FOLD_ORDER` withholds — rather than an
    // approximate reciprocal, and this assertion is what makes that attribution
    // executable instead of narrated.
    let reordered = ((exponentials[0] + exponentials[2]) + exponentials[1]) + exponentials[3];
    assert_eq!(reordered.to_bits(), REORDERED_DENOMINATOR);
    assert_ne!(reordered.to_bits(), denominator.to_bits());
    assert_eq!(
        (1.0_f32 / reordered).to_bits(),
        REFERENCE_MODEL_WORKED_EXAMPLE_CONSTANT,
        "the reference model's constant is the correctly rounded reciprocal of the reordered denominator"
    );

    // The divide spelling is *not* what separates them: on this row it agrees
    // with the pinned reciprocal form, so the record's bits are not the division
    // either. The form question is settled by the narrow rows below.
    assert_eq!(
        exponentials
            .iter()
            .map(|value| (value / denominator).to_bits())
            .collect::<Vec<u32>>(),
        result
    );
}

/// The normalization form is the reciprocal multiply, at the widths that isolate it.
///
/// **Width two and width three are where an element-by-element comparison is
/// answerable**, because from width four upward the reference model's own
/// contributor order moves the constant and its outputs belong to neither
/// spelling's bits. At these widths the retained probe counts every discriminating
/// element matching the reciprocal form and none matching the division, and this
/// row is one such element carried as bits. The probe's single-constant rows carry
/// the form at every measured width, since a division by a denominator is not one
/// scalar multiple of the numerators; that argument is order-insensitive and is
/// recorded in the module documentation rather than here.
#[test]
fn the_normalization_multiplies_by_the_reciprocal_rather_than_dividing() {
    // `[0.0, 2.0]`: a row where the two spellings and the reference agree, and
    // where the row sum is *not* one — see the row-sum test below.
    let result = softmax(&[1, 2], 1, &[0.0, 2.0]);
    assert_eq!(result, vec![0x3df4_20a8, 0x3f61_7bea]);

    // The two spellings, computed from the same exponentials, so the comparison
    // is of the normalization step alone.
    let exponentials = [
        certified_exp_f32(-2.0).expect("decided"),
        certified_exp_f32(0.0).expect("decided"),
    ];
    let denominator = exponentials[0] + exponentials[1];
    let reciprocal = 1.0_f32 / denominator;
    for (index, exponential) in exponentials.iter().enumerate() {
        assert_eq!(result[index], (exponential * reciprocal).to_bits());
    }
    // The corpus must contain an element where the two spellings disagree, or the
    // assertion above would pass under either form. This one does.
    let differs = exponentials
        .iter()
        .any(|value| (value / denominator).to_bits() != (value * reciprocal).to_bits());
    assert!(
        differs,
        "the corpus must contain an element separating the reciprocal form from the division"
    );
}

/// A row's outputs do not sum to exactly one, and the deviation goes both ways.
///
/// The fact `SOFTMAX_F32_FACT_ROW_SUM` states, carried as bits. Both rows are
/// width two or three, where the reference model agrees with the pinned formula at
/// every element, so each is simultaneously a property of this reference and a
/// measured property of the workload's own.
#[test]
fn a_rows_outputs_do_not_sum_to_exactly_one() {
    let below = softmax(&[1, 2], 1, &[0.0, 2.0]);
    assert_eq!(strict_sum(&below), 0x3f7f_ffff);

    let above = softmax(&[1, 3], 1, &[0.0, 1.0, 0.0]);
    assert_eq!(above, vec![0x3e59_0736, 0x3f13_7c66, 0x3e59_0736]);
    assert_eq!(strict_sum(&above), 0x3f80_0001);

    // Both directions, so even a one-sided check would be wrong.
    assert!(f32::from_bits(strict_sum(&below)) < 1.0);
    assert!(f32::from_bits(strict_sum(&above)) > 1.0);
}

/// A row of equal large scores is finite, where the naive form is NaN.
///
/// **The maximum subtraction, exercised rather than described.** This is finite
/// against NaN rather than a tolerance: `exp(1000)` overflows binary32, so a
/// quotient of raw exponentials is `inf / inf`.
#[test]
fn a_row_of_equal_large_scores_is_finite_where_the_naive_form_is_not() {
    assert_eq!(
        softmax(&[1, 2], 1, &[1000.0, 1000.0]),
        vec![0x3f00_0000, 0x3f00_0000]
    );
    // The naive quotient of exponentials, which the pinned formula exists to
    // exclude. `certified_exp_f32` reports the overflow rather than hiding it.
    let raw = certified_exp_f32(1000.0).expect("the overflow guard decides");
    assert_eq!(raw.to_bits(), f32::INFINITY.to_bits());
    assert!((raw / (raw + raw)).is_nan());

    // The subtraction is what confines every exponential argument to `(-inf, 0]`,
    // which is the region the registered accuracy contract admits.
    assert_eq!((1000.0_f32 - 1000.0).to_bits(), 0x0000_0000);
}

/// The underflow band: a contributor about 104 below the maximum contributes zero.
///
/// **Three positions, one per region of the band**, because the two retained
/// thresholds cut the tail into three and a row that touched only one of them
/// would report a property of that region as a property of the operation. At 87
/// below the maximum the contribution is the *smallest normal* magnitude; at 88
/// below it is a subnormal; at 104 below it is exactly `+0.0`. The denominator is
/// exactly `1.0` here — every tail contribution is far below one half-ULP of it —
/// so the reciprocal is exactly one and each output *is* its exponential, which
/// is what makes the row read directly as the band.
#[test]
fn the_underflow_band_contributes_exactly_zero_below_about_one_hundred_and_four() {
    let result = softmax(&[1, 4], 1, &[0.0, -104.0, -88.0, -87.0]);
    assert_eq!(
        result,
        vec![0x3f80_0000, 0x0000_0000, 0x0041_edc4, 0x00b3_3687]
    );
    assert_eq!(f32::from_bits(result[1]).to_bits(), 0x0000_0000);
    assert!(
        f32::from_bits(result[2]).is_subnormal(),
        "88 below the maximum is inside the subnormal band"
    );
    assert!(
        f32::from_bits(result[3]).is_normal(),
        "87 below the maximum is still normal, so the band's upper edge is between them"
    );

    // The retained probe's two measured thresholds, reproduced by the certified
    // exponential rather than asserted from the record.
    assert_eq!(
        certified_exp_f32(f32::from_bits(0xc2cf_f1b5))
            .expect("decided")
            .to_bits(),
        0x0000_0000,
        "below about -103.97 the exponential is exactly +0.0"
    );
    assert!(
        certified_exp_f32(f32::from_bits(0xc2ae_ac50))
            .expect("decided")
            .is_subnormal(),
        "at about -87.34 the exponential is already a subnormal"
    );
}

/// A masked contributor contributes exactly `+0.0` and receives exactly `+0.0`.
#[test]
fn a_masked_contributor_contributes_and_receives_exactly_positive_zero() {
    let mask = f32::from_bits(MASK_FILL_BITS);
    assert_eq!(
        softmax(&[1, 3], 1, &[0.5, mask, mask]),
        vec![0x3f80_0000, 0x0000_0000, 0x0000_0000]
    );
}

/// Decision **D-1**: a fully masked row, under both mask conventions.
///
/// **This is the only case that tests D-1 at all.** The measurement the L4 design
/// added records that replacing the finite fill with `-inf` over the whole C1
/// score tensor changes 0 of 1,600 outputs, so a corpus that only ran C1 would
/// pass with either convention installed. The synthetic row is therefore
/// mandatory rather than extra coverage.
///
/// The family applies **no repair**: each answer is what the one pinned formula
/// returns on the caller's own values. Under the finite fill the row is a row of
/// equal finite numbers, so the maximum subtraction gives exact zeros, every
/// exponential is exactly one, and the result is uniform. Under `-inf` the
/// subtraction is `-inf - -inf`, which is a NaN, and the row is NaN throughout.
#[test]
fn a_fully_masked_row_follows_the_pinned_formula_under_either_mask_convention() {
    let mask = f32::from_bits(MASK_FILL_BITS);

    // The retained probe's width-three row, under the workload's own fill.
    assert_eq!(softmax(&[1, 3], 1, &[mask; 3]), vec![0x3eaa_aaab; 3]);
    // The attention-block probe's width-ten row, which is the C1 row's width.
    assert_eq!(softmax(&[1, 10], 1, &[mask; 10]), vec![0x3dcc_cccd; 10]);
    // And the `-inf` convention, which the same formula maps to NaNs.
    assert_eq!(
        softmax(&[1, 10], 1, &[f32::NEG_INFINITY; 10]),
        vec![CANONICAL_NAN; 10]
    );

    // The mechanism, so the two answers read as one formula rather than two rules:
    // the finite fill shifts to an exact zero and the infinite one to a NaN.
    assert_eq!((mask - mask).to_bits(), 0x0000_0000);
    assert!((f32::NEG_INFINITY - f32::NEG_INFINITY).is_nan());

    // The uniform values are the reciprocals of the widths, and they are not the
    // exact rationals a reader might expect: neither a third nor a tenth is
    // representable, so a check comparing against `1.0 / n` computed some other
    // way could disagree.
    assert_eq!((1.0_f32 / 3.0).to_bits(), 0x3eaa_aaab);
    assert_eq!((1.0_f32 / 10.0).to_bits(), 0x3dcc_cccd);
}

/// Both signed zeros are ordinary scores, and the maximum's ordering is invisible.
///
/// The row `[+0.0, -0.0]` is where the extrema families' zero rule could show
/// through, and it does not: `Exp` maps both zeros to exactly `1.0`, so the sign
/// of the maximum cannot reach the output. Stated as a corpus row because it is
/// the reason the Metal fixup's zero clause is about the *reduction* rather than
/// about this operation's result.
#[test]
fn both_signed_zeros_are_ordinary_scores() {
    assert_eq!(
        softmax(&[1, 2], 1, &[0.0, -0.0]),
        vec![0x3f00_0000, 0x3f00_0000]
    );
    assert_eq!(
        certified_exp_f32(0.0).expect("decided").to_bits(),
        0x3f80_0000
    );
    assert_eq!(
        certified_exp_f32(-0.0).expect("decided").to_bits(),
        0x3f80_0000
    );
}

/// A zero-length reduced axis yields a zero-length result rather than an error.
///
/// The identity-less maximum never faces an empty contributor domain, because a
/// row with no contributors produces no output either. That is the property
/// `SOFTMAX_F32_FACT_EMPTY_REDUCED_AXIS` states, and it is what places the case
/// outside the reduction contract's empty-domain rules.
#[test]
fn an_empty_reduced_axis_preserves_the_empty_shape() {
    assert!(softmax(&[2, 0], 1, &[]).is_empty());
    // A non-empty tensor whose *other* axis is zero behaves the same way.
    assert!(softmax(&[0, 4], 1, &[]).is_empty());
}

/// The declared axis selects which rows are normalized.
///
/// A `[2, 3]` tensor reduced over axis 0 and over axis 1 gives different results
/// from the same data, so an implementation that always folded the trailing axis
/// would fail here.
#[test]
fn the_declared_axis_selects_which_rows_are_reduced() {
    let values = [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let over_rows = softmax(&[2, 3], 1, &values);
    let over_columns = softmax(&[2, 3], 0, &values);
    assert_ne!(over_rows, over_columns);

    // Axis 1 reduces `[1, 2, 3]` and `[4, 5, 6]`; axis 0 reduces `[1, 4]`,
    // `[2, 5]`, and `[3, 6]`. The first element's answer is recomputed here from
    // the pinned formula rather than compared to a constant.
    let row_max = 3.0_f32;
    let row = [
        certified_exp_f32(1.0 - row_max).expect("decided"),
        certified_exp_f32(2.0 - row_max).expect("decided"),
        certified_exp_f32(3.0 - row_max).expect("decided"),
    ];
    let row_denominator = row[0] + row[1] + row[2];
    assert_eq!(
        over_rows[0],
        (row[0] * (1.0_f32 / row_denominator)).to_bits()
    );

    let column_max = 4.0_f32;
    let column = [
        certified_exp_f32(1.0 - column_max).expect("decided"),
        certified_exp_f32(4.0 - column_max).expect("decided"),
    ];
    let column_denominator = column[0] + column[1];
    assert_eq!(
        over_columns[0],
        (column[0] * (1.0_f32 / column_denominator)).to_bits()
    );
}

/// The C1 row's shape, at the exact extents the conformance evidence covers.
///
/// **The declared population, stated rather than implied.** Every row above is at
/// an extent this test enumerates: 2, 3, 4, and 10 contributors, plus the
/// zero-length case. Nothing here generalizes to the workload's growing `S`, which
/// is exercised only at the static values a program can state — a semantic shape
/// carries a magnitude and not a symbol, so there is no generic extent to test.
#[test]
fn the_conformance_evidence_covers_exactly_these_reduced_extents() {
    let covered = [0_u64, 2, 3, 4, 10];
    for extent in covered {
        let values = vec![0.5_f32; usize::try_from(extent).expect("a bounded extent")];
        let result = softmax(&[1, extent], 1, &values);
        assert_eq!(result.len(), values.len());
        if extent > 0 {
            // A uniform row is uniform, whatever the extent, which is the one
            // property that does hold across all of them.
            assert!(result.iter().all(|bits| *bits == result[0]));
        }
    }
    // The C1 conformance row's own score shape, reduced over its last axis.
    let c1 = softmax(&[8, 2, 10, 10], 3, &vec![0.25_f32; 1_600]);
    assert_eq!(c1.len(), 1_600);
    assert!(c1.iter().all(|bits| *bits == 0x3dcc_cccd));
}

// ---------------------------------------------------------------------------
// The registered evaluator, and its refusals
// ---------------------------------------------------------------------------

/// The registered evaluator reproduces the worked example through a real program.
///
/// The end-to-end path rather than the direct call: a semantic program built with
/// `F32Softmax::apply`, evaluated by the standard reference evaluator, which
/// resolves the capability by key and signature. A registration that existed but
/// dispatched elsewhere would fail here rather than pass a presence check.
#[test]
fn the_registered_evaluator_reproduces_the_worked_example_end_to_end() {
    let shape = shape(&[1, 4]);
    let mut graph = SemanticProgramBuilder::try_standard().expect("the standard builder");
    let scores = graph
        .input::<F32>(InputKey::new("s").expect("a key"), shape.clone())
        .expect("an input");
    let weights =
        F32Softmax::apply(&mut graph, scores, Axis::new(1)).expect("the occurrence is well formed");
    graph
        .output(OutputKey::new("y").expect("a key"), weights)
        .expect("an output");
    let program = graph.build().expect("the program builds");

    let mask = f32::from_bits(MASK_FILL_BITS);
    let payload = dense(&shape, &[1.0, 2.0, 3.0, mask]);
    let key = InputKey::new("s").expect("a key");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator")
        .evaluate(&program, &[InputBinding::new(&key, &payload)])
        .expect("the program evaluates");
    let [result] = outputs.as_slice() else {
        panic!("the program has one output");
    };
    let bits: Vec<u32> = f32_elements(result)
        .expect("a dense f32 result")
        .iter()
        .map(|element| decode_f32(element).expect("decodes").to_bits())
        .collect();
    assert_eq!(
        bits,
        vec![0x3db8_61f3, 0x3e7a_9a1a, 0x3f2a_4d3b, 0x0000_0000]
    );
}

fn dense(shape: &Shape, values: &[f32]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        shape.clone(),
        values
            .iter()
            .map(|value| f32_element(*value))
            .collect::<Result<Vec<_>, _>>()
            .expect("a corpus payload"),
    )
    .expect("a corpus tensor")
}

/// A malformed application refuses rather than guessing.
///
/// Each row was observed to refuse, and the control below shows the same call
/// succeeding with an admissible axis and payload.
#[test]
fn a_malformed_application_refuses_rather_than_guessing() {
    assert!(
        softmax_f32(&shape(&[1, 2]), Axis::new(2), &[1.0, 2.0]).is_err(),
        "an out-of-range axis refuses"
    );
    assert!(
        softmax_f32(&shape(&[1, 2]), Axis::new(1), &[1.0]).is_err(),
        "a short payload refuses"
    );
    assert!(
        softmax_f32(&shape(&[1, 2]), Axis::new(1), &[1.0, 2.0, 3.0]).is_err(),
        "a long payload refuses"
    );
    assert!(softmax_f32(&shape(&[1, 2]), Axis::new(1), &[1.0, 2.0]).is_ok());
}
