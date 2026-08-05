use super::*;

use tiler_ir::semantic::accuracy::{ExactRational, UlpFormat};
use tiler_ir::semantic::{
    RMS_NORM_F32_QWEN3_EPS_BITS, RMS_NORM_F32_SQUARING_OVERFLOW_BITS,
    builtin_scalar_value_type_facts, rms_norm_f32_op, rms_norm_f32_rsqrt_accuracy_contract,
};

use tiler_ir::semantic::{F32RmsNorm, InputKey, OutputKey, SemanticProgramBuilder};

use crate::accuracy::{ConformanceDecision, decide_contract, exact_binary32_candidate};
use crate::evaluate::ReferenceEvaluator;
use crate::tensor::InputBinding;

/// The pinned workload's `eps`, used by every corpus row unless it says otherwise.
const EPS: u32 = RMS_NORM_F32_QWEN3_EPS_BITS;

/// The retained probe's `rsqrt_of_eps_alone`, from `torch.rsqrt` on its CPU row.
///
/// Kept as a named constant so that every place this record disagrees with the
/// certified reference points at the same measurement rather than at a literal.
const RETAINED_TORCH_RSQRT_OF_EPS: u32 = 0x4479_ffff;

/// The retained probe's `rms_subnormal_vector`, which the value above propagates into.
const RETAINED_PROBE_SUBNORMAL_ROW: u32 = 0x0208_1cb9;

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a corpus shape is bounded")
}

/// Runs the reference over one row, returning the exact result payloads.
fn normalize(dims: &[u64], axis: u32, eps: u32, values: &[f32], weights: &[f32]) -> Vec<u32> {
    rms_norm_f32(&shape(dims), Axis::new(axis), eps, values, weights)
        .expect("a corpus row evaluates")
        .into_iter()
        .map(f32::to_bits)
        .collect()
}

// ---------------------------------------------------------------------------
// The certified reciprocal square root
// ---------------------------------------------------------------------------

/// The reference is the correctly rounded value, not the two-rounding composition.
///
/// **This is the discriminating row of the whole corpus.** At `t = eps` the exact
/// reference is `1000.00000126…`; `f32(1 / f32(sqrt(t)))` rounds twice and gives
/// `0x4479ffff`, while the correctly rounded value is `0x447a0000`. The retained
/// reference-semantics probe records `torch.rsqrt` delivering `0x4479ffff` there,
/// so the workload's own implementation agrees with the composition the pinned
/// formula's choice of `rsqrt` exists to exclude. A corpus without this argument
/// would report the two spellings identical — the failure signature this
/// repository distrusts — and the worked example below is exactly such an
/// argument, which is why both are kept.
#[test]
fn the_certified_reciprocal_square_root_separates_rsqrt_from_one_over_sqrt() {
    let eps = f32::from_bits(EPS);
    let certified = certified_rsqrt_f32(eps).expect("the reference is decided");
    assert_eq!(certified.to_bits(), 0x447a_0000);

    let composed = 1.0_f32 / eps.sqrt();
    assert_eq!(composed.to_bits(), 0x4479_ffff);
    assert_ne!(certified.to_bits(), composed.to_bits());

    // The probe's retained observation, restated as the exact value it is, so a
    // reader can see which of the two this reference reproduces.
    assert_eq!(composed.to_bits(), RETAINED_TORCH_RSQRT_OF_EPS);
}

/// The exactly-rational arguments are decided, and they are the powers of two.
///
/// The module header proves that `1/sqrt(t)` is dyadic exactly when `t` is an
/// even power of two, in which case the reference is itself a power of two and
/// therefore representable rather than a midpoint. These rows are that proof's
/// reachable cases: each decides, and each decides to the exact value.
#[test]
fn the_exactly_rational_reciprocal_square_roots_decide_to_powers_of_two() {
    for (argument, expected) in [
        (1.0_f32, 1.0_f32),
        (4.0, 0.5),
        (64.0, 0.125),
        (1024.0, 0.031_25),
        (0.25, 2.0),
        (f32::from_bits(0x0080_0000), f32::from_bits(0x5f00_0000)),
    ] {
        let certified = certified_rsqrt_f32(argument).expect("an exact argument is decided");
        assert_eq!(
            certified.to_bits(),
            expected.to_bits(),
            "1/sqrt({argument:e}) is exactly {expected:e}"
        );
    }
}

/// The four exceptional arguments follow the declared rules rather than the host.
#[test]
fn the_exceptional_arguments_follow_the_declared_rules() {
    assert!(
        certified_rsqrt_f32(f32::NAN)
            .expect("a NaN decides")
            .is_nan()
    );
    // `+inf` is the squaring-overflow route: a zero scale, hence a row of zeros.
    assert_eq!(
        certified_rsqrt_f32(f32::INFINITY)
            .expect("an infinite argument decides")
            .to_bits(),
        0x0000_0000
    );
    assert_eq!(
        certified_rsqrt_f32(0.0)
            .expect("positive zero decides")
            .to_bits(),
        f32::INFINITY.to_bits()
    );
    assert_eq!(
        certified_rsqrt_f32(-0.0)
            .expect("negative zero decides")
            .to_bits(),
        f32::NEG_INFINITY.to_bits()
    );
    assert!(
        certified_rsqrt_f32(-1.0)
            .expect("a negative argument decides")
            .is_nan()
    );
}

/// Every certified value is the one the exact enclosure brackets.
///
/// A sweep over a contiguous binary32 band rather than a hand-picked list, so the
/// agreement is over a population rather than over the arguments someone thought
/// of. The band sits around `1.0`, where a binary32 ULP is `2^-23` and the
/// enclosure's grid is `2^-256`.
#[test]
fn the_certified_value_is_the_one_the_enclosure_brackets() {
    let mut checked = 0_usize;
    for offset in 0..512_u32 {
        let bits = 0x3f80_0000 + offset;
        let argument = f32::from_bits(bits);
        let certified = certified_rsqrt_f32(argument).expect("the band decides");
        let enclosure = rsqrt_enclosure(
            &ExactRational::from_f32(argument).expect("finite"),
            EnclosurePrecision::binary32_corpus(),
        )
        .expect("bracketed");
        assert!(
            rounds_to(&enclosure, certified),
            "1/sqrt({argument:e}) = {certified:e} is not the bracketed value"
        );
        checked += 1;
    }
    assert_eq!(checked, 512, "the sweep covered its whole declared band");
}

/// A grid too coarse to separate the neighbours yields no decision at all.
///
/// The deliberate perturbation behind the refusal: the enclosure widens,
/// `rounds_to` stops admitting the true value, and the only honest answer is
/// [`ReferenceOperationError::UndecidedTranscendentalReference`]. A reference that
/// resolved this toward the nearer side would be one that cannot fail.
#[test]
fn a_coarse_enclosure_decides_nothing_rather_than_guessing() {
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    let coarse = rsqrt_enclosure(
        &ExactRational::from_f32(2.0).expect("finite"),
        EnclosurePrecision::new(4).expect("a stateable grid"),
    )
    .expect("bracketed");
    assert!(
        coarse.width() > format.ulp_scale(coarse.lower()).expect("in range"),
        "a four-bit grid is coarser than one binary32 ULP at 1/sqrt(2)"
    );
    let exact = certified_rsqrt_f32(2.0).expect("the fine grid decides");
    assert!(
        !rounds_to(&coarse, exact),
        "the coarse bracket does not establish the correctly rounded value"
    );
}

// ---------------------------------------------------------------------------
// Reference-evaluating the resolved accuracy contract
// ---------------------------------------------------------------------------

/// The registered `Faithful` contract admits both neighbours and refuses the third.
///
/// The contract is decided against a certified enclosure rather than compared to
/// a constant, and the three rows are the whole of what `Faithful` means: the
/// correctly rounded value conforms, its other bracketing neighbour conforms, and
/// the value one step beyond the pair violates. The third row is what makes the
/// measured `torch.rsqrt` disagreement a *finding* rather than a rounding
/// preference — `0x4479ffff` is that third row at `t = eps`.
#[test]
fn the_registered_faithful_contract_admits_the_pair_and_refuses_the_third_value() {
    let contract = registered_contract();
    let facts = builtin_scalar_value_type_facts(&F32::resolved_type()).expect("governed");
    let format = UlpFormat::from_value_type_facts(&facts).expect("f32 carries the metric");
    contract
        .verify(&facts)
        .expect("the registered contract verifies");

    let argument = f32::from_bits(EPS);
    let enclosure = rsqrt_enclosure(
        &ExactRational::from_f32(argument).expect("finite"),
        EnclosurePrecision::binary32_corpus(),
    )
    .expect("bracketed");
    let inputs = [ExactRational::from_f32(argument).expect("finite")];
    let decide = |bits: u32| {
        let candidate = exact_binary32_candidate(f32::from_bits(bits)).expect("finite");
        decide_contract(&contract, &format, &inputs, &enclosure, &candidate)
    };
    // The exact reference is 1000.00000126…, bracketed by 0x447a0000 and
    // 0x447a0001; both are faithful and the step below the pair is not.
    assert_eq!(decide(0x447a_0000), ConformanceDecision::Conforms);
    assert_eq!(decide(0x447a_0001), ConformanceDecision::Conforms);
    assert_eq!(decide(0x4479_ffff), ConformanceDecision::Violates);
    assert_eq!(decide(0x447a_0002), ConformanceDecision::Violates);
}

/// Decoded from the registered definition's own facts, never reconstructed.
fn registered_contract() -> tiler_ir::semantic::accuracy::AccuracyContract {
    use tiler_ir::semantic::{CanonicalValueView, FrozenSemanticRegistry};
    let registry = FrozenSemanticRegistry::standard().expect("the standard registry builds");
    let definition = registry
        .operation_definition(&rms_norm_f32_op())
        .expect("the normalization is registered");
    let CanonicalValueView::Record(fields) = definition.canonical_facts().value().view() else {
        panic!("the fact record is a record");
    };
    let carried = fields
        .iter()
        .find(|field| field.id() == tiler_ir::semantic::RMS_NORM_F32_FACT_RSQRT_ACCURACY_CONTRACT)
        .expect("the accuracy-contract fact is registered");
    let decoded =
        tiler_ir::semantic::accuracy::AccuracyContract::from_canonical_value(carried.value())
            .expect("the registered contract decodes");
    assert_eq!(decoded, rms_norm_f32_rsqrt_accuracy_contract());
    decoded
}

// ---------------------------------------------------------------------------
// The bounded conformance corpus
// ---------------------------------------------------------------------------

/// The derivation's retained worked example, at its recorded bit patterns.
///
/// `x = [3.0, 4.0]`, `w = [1.0, 2.0]`, `eps = 1e-6`. Every intermediate the L3′
/// record states is reproduced, which is what makes this a check on the *formula*
/// rather than on the final value alone.
#[test]
fn the_retained_worked_example_reproduces_its_recorded_bits() {
    let result = normalize(&[1, 2], 1, EPS, &[3.0, 4.0], &[1.0, 2.0]);
    assert_eq!(result, vec![0x3f59_3923, 0x4010_d0c2]);

    // The intermediates, in the order the reference states them.
    let squares = [3.0_f32 * 3.0, 4.0_f32 * 4.0];
    assert_eq!(squares[0].to_bits(), 0x4110_0000);
    assert_eq!(squares[1].to_bits(), 0x4180_0000);
    // Written as a sum and then a division, in that order, because that *is*
    // the pinned formula: `f32::midpoint` is a different computation and reading
    // it here would hide which arithmetic the reference performs.
    let total = squares[0] + squares[1];
    assert_eq!(total.to_bits(), 0x41c8_0000);
    let mean = total / 2.0_f32;
    assert_eq!(mean.to_bits(), 0x4148_0000);
    let argument = mean + f32::from_bits(EPS);
    assert_eq!(argument.to_bits(), 0x4148_0001);
    let scale = certified_rsqrt_f32(argument).expect("decided");
    assert_eq!(scale.to_bits(), 0x3e90_d0c2);
    assert_eq!((3.0_f32 * scale).to_bits(), 0x3f59_3923);
    assert_eq!((4.0_f32 * scale).to_bits(), 0x3f90_d0c2);
}

/// The zero row normalizes to zeros, which `eps` is what makes total.
#[test]
fn the_zero_row_normalizes_to_zeros() {
    let result = normalize(&[1, 4], 1, EPS, &[0.0; 4], &[1.0; 4]);
    assert_eq!(result, vec![0x0000_0000; 4]);
    // The perturbation that shows `eps` is load-bearing rather than decorative:
    // without it the scale is the reciprocal square root of exactly zero, which
    // is an infinity, and every output becomes `0 * inf` — a NaN. The refusal at
    // construction is what keeps a program from reaching this, and the arithmetic
    // here is what it is protecting against.
    let without_eps = certified_rsqrt_f32(0.0).expect("decided");
    assert_eq!(without_eps.to_bits(), f32::INFINITY.to_bits());
    assert!((0.0_f32 * without_eps).is_nan());
}

/// A negative zero element keeps its sign through the normalization.
#[test]
fn a_negative_zero_element_normalizes_to_a_negative_zero() {
    let result = normalize(&[1, 2], 1, EPS, &[-0.0, 0.0], &[1.0, 1.0]);
    assert_eq!(result, vec![0x8000_0000, 0x0000_0000]);
}

/// The subnormal row, and the two divergences it carries.
///
/// **This reference gives `0x02081cba` where the retained probe records
/// `0x02081cb9`.** The whole difference is the reciprocal square root: the
/// squares of `1e-40` underflow to exactly `+0.0`, so the argument is `eps`
/// alone, and the two implementations differ there by one step for the reason the
/// module header states. The row is recorded, not tuned.
///
/// **A second divergence lives on the target side and is not visible here.** On a
/// realization that flushes input subnormals the elements reach the squaring as
/// zeros and the row normalizes to zeros. That is a declared realization
/// difference under ADR 0076 rather than a defect, and this reference — which
/// preserves subnormals — cannot observe it.
#[test]
fn the_subnormal_row_normalizes_to_a_normal_value_and_records_two_divergences() {
    let subnormal = f32::from_bits(0x0001_16c2);
    assert!(subnormal.is_subnormal());
    let result = normalize(&[1, 4], 1, EPS, &[subnormal; 4], &[1.0; 4]);
    assert_eq!(result, vec![0x0208_1cba; 4]);

    // The retained probe's value, and the exact reason it differs.
    assert_ne!(result[0], RETAINED_PROBE_SUBNORMAL_ROW);
    let composed_scale = 1.0_f32 / f32::from_bits(EPS).sqrt();
    assert_eq!(
        (subnormal * composed_scale).to_bits(),
        RETAINED_PROBE_SUBNORMAL_ROW
    );

    // The squares underflow, so the mean of squares is exactly zero and the
    // argument is `eps` alone — which is why this row and the zero row share a
    // reciprocal square root.
    assert_eq!((subnormal * subnormal).to_bits(), 0x0000_0000);
    // A flushing realization would see zeros here, giving a row of zeros; the
    // preserving reference sees a normal result. Both are legal and they differ.
    assert!(f32::from_bits(result[0]).is_normal());
}

/// A row above the squaring-overflow threshold is defined and produces zeros.
///
/// Decision **D-3**, exercised rather than described: the mean of squares is
/// `+inf`, the scale is exactly `+0.0`, and every output is a signed zero. The
/// output is finite, plausible, and wrong — which is the point, and which the
/// operation reproduces because the reference model does.
#[test]
fn a_row_above_the_squaring_overflow_threshold_normalizes_to_signed_zeros() {
    let big = 1e20_f32;
    assert!(big > f32::from_bits(RMS_NORM_F32_SQUARING_OVERFLOW_BITS));
    assert!(!(big * big).is_finite());
    let result = normalize(&[1, 4], 1, EPS, &[big, -big, big, big], &[1.0; 4]);
    assert_eq!(
        result,
        vec![0x0000_0000, 0x8000_0000, 0x0000_0000, 0x0000_0000],
        "the sign of each zero follows its element's sign"
    );

    // The threshold itself is the boundary: at it the square is finite and the
    // row normalizes to a nonzero value, one step above it the square overflows.
    let threshold = f32::from_bits(RMS_NORM_F32_SQUARING_OVERFLOW_BITS);
    let at = normalize(&[1, 1], 1, EPS, &[threshold], &[1.0]);
    assert_ne!(at[0], 0x0000_0000);
    let successor = f32::from_bits(RMS_NORM_F32_SQUARING_OVERFLOW_BITS + 1);
    let beyond = normalize(&[1, 1], 1, EPS, &[successor], &[1.0]);
    assert_eq!(beyond[0], 0x0000_0000);
}

/// Both workload extent classes, at rows whose results differ.
///
/// 1024 and 128 are the two extents the workload's 113 occurrences use — 57 and
/// 56 respectively. The uniform rows exercise the exactly-rational reciprocal
/// square root and an `eps` addition that falls below half an ULP and therefore
/// changes nothing; the holed rows put the argument at a non-power-of-two value
/// so the two extents produce *different* results and a swapped extent is
/// detectable.
#[test]
fn both_workload_extent_classes_normalize_to_their_recorded_bits() {
    // Extent 1024: 1024 elements of 32.0 give a mean of squares of exactly
    // 1024.0, whose `eps` addition is below half an ULP and therefore an
    // identity, and whose reciprocal square root is exactly 2^-5.
    let uniform_1024 = normalize(&[1, 1024], 1, EPS, &[32.0; 1024], &[1.0; 1024]);
    assert_eq!(uniform_1024, vec![0x3f80_0000; 1024]);
    assert_eq!(
        certified_rsqrt_f32(1024.0).expect("decided").to_bits(),
        0x3d00_0000
    );
    assert_eq!(
        (1024.0_f32 + f32::from_bits(EPS)).to_bits(),
        1024.0_f32.to_bits()
    );

    // Extent 128: 128 elements of 8.0 give a mean of squares of exactly 64.0 and
    // a reciprocal square root of exactly 2^-3.
    let uniform_128 = normalize(&[1, 128], 1, EPS, &[8.0; 128], &[1.0; 128]);
    assert_eq!(uniform_128, vec![0x3f80_0000; 128]);
    assert_eq!(
        certified_rsqrt_f32(64.0).expect("decided").to_bits(),
        0x3e00_0000
    );

    // The holed rows: one zero element makes the mean 1023.0 and 63.0
    // respectively, neither a power of two, so the reciprocal square roots are
    // irrational and the two extents disagree.
    let mut values = [32.0_f32; 1024];
    values[1023] = 0.0;
    let holed_1024 = normalize(&[1, 1024], 1, EPS, &values, &[1.0; 1024]);
    assert_eq!(holed_1024[0], 0x3f80_1003);
    assert_eq!(holed_1024[1023], 0x0000_0000);

    let mut values = [8.0_f32; 128];
    values[127] = 0.0;
    let holed_128 = normalize(&[1, 128], 1, EPS, &values, &[1.0; 128]);
    assert_eq!(holed_128[0], 0x3f80_80c1);
    assert_eq!(holed_128[127], 0x0000_0000);

    assert_ne!(
        holed_1024[0], holed_128[0],
        "the two extent classes must be distinguishable by their results"
    );
}

/// The normalized axis is honoured rather than assumed to be the last one.
///
/// A `[2, 3]` tensor normalized over axis 0 and over axis 1 gives different
/// results from the same data, so an implementation that always folded the
/// trailing axis would fail here.
#[test]
fn the_declared_axis_selects_which_rows_are_normalized() {
    let values = [1.0_f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let weights = [1.0_f32; 6];
    let over_rows = normalize(&[2, 3], 1, EPS, &values, &weights);
    let over_columns = normalize(&[2, 3], 0, EPS, &values, &weights);
    assert_ne!(over_rows, over_columns);

    // Axis 1 folds `[1, 2, 3]` and `[4, 5, 6]`; axis 0 folds `[1, 4]`, `[2, 5]`,
    // and `[3, 6]`. The first element's scale therefore differs, and it is
    // recomputed here from the pinned formula rather than compared to a constant.
    // Folded then divided, in that order, rather than through any host helper
    // that would compute a mean by a different route.
    let mean_row = (1.0_f32 * 1.0 + 2.0 * 2.0 + 3.0 * 3.0) / 3.0_f32;
    let column_total = 1.0_f32 * 1.0 + 4.0 * 4.0;
    let mean_column = column_total / 2.0_f32;
    let scale_row = certified_rsqrt_f32(mean_row + f32::from_bits(EPS)).expect("decided");
    let scale_column = certified_rsqrt_f32(mean_column + f32::from_bits(EPS)).expect("decided");
    assert_eq!(over_rows[0], (1.0_f32 * scale_row).to_bits());
    assert_eq!(over_columns[0], (1.0_f32 * scale_column).to_bits());
}

/// The weight is applied after the normalization, never folded into the scale.
#[test]
fn the_weight_multiplies_the_normalized_value_rather_than_the_scale() {
    let values = [3.0_f32, 4.0];
    // `0.1` is chosen because it *discriminates*: at this element the two
    // associations differ by one ULP, so a realization that folded the weight
    // into the scale would fail here. A weight of `7.0` does not discriminate,
    // which is exactly the uniform-agreement signature this corpus avoids.
    let weights = [0.1_f32, 11.0];
    let result = normalize(&[1, 2], 1, EPS, &values, &weights);
    // Folded then divided, in that order: the pinned formula sums the squares
    // and then divides by the extent, which is a different computation from any
    // host mean helper and is what this row is checking.
    let total = 9.0_f32 + 16.0;
    let scale = certified_rsqrt_f32(total / 2.0_f32 + f32::from_bits(EPS)).expect("decided");
    for index in 0..2 {
        assert_eq!(
            result[index],
            (weights[index] * (values[index] * scale)).to_bits()
        );
    }
    // The alternative association `(w * x) * r` is a different binary32 function
    // in general; here at least one element differs, so the corpus discriminates.
    let differs =
        (0..2).any(|index| ((weights[index] * values[index]) * scale).to_bits() != result[index]);
    assert!(
        differs,
        "the corpus must contain an element where the two associations disagree"
    );
}

/// An empty normalized axis yields an empty result rather than an error.
#[test]
fn an_empty_normalized_axis_preserves_the_empty_shape() {
    // No element exists, so no output does either; the shape is preserved and
    // empty. The case is decided rather than discovered.
    let result = normalize(&[2, 0], 1, EPS, &[], &[]);
    assert!(result.is_empty());
}

// ---------------------------------------------------------------------------
// The registered evaluator, and its refusals
// ---------------------------------------------------------------------------

/// The registered evaluator reproduces the worked example through a real program.
///
/// The end-to-end path rather than the direct call: a semantic program built with
/// `F32RmsNorm::apply`, evaluated by the standard reference evaluator, which
/// resolves the capability by key and signature. A registration that existed but
/// dispatched elsewhere would fail here rather than pass a presence check.
#[test]
fn the_registered_evaluator_reproduces_the_worked_example_end_to_end() {
    let shape = shape(&[1, 2]);
    let mut graph = SemanticProgramBuilder::try_standard().expect("the standard builder");
    let value = graph
        .input::<F32>(InputKey::new("x").expect("a key"), shape.clone())
        .expect("an input");
    let weight = graph
        .input::<F32>(InputKey::new("w").expect("a key"), shape.clone())
        .expect("an input");
    let normalized = F32RmsNorm::apply(&mut graph, value, weight, Axis::new(1), EPS)
        .expect("the occurrence is well formed");
    graph
        .output(OutputKey::new("y").expect("a key"), normalized)
        .expect("an output");
    let program = graph.build().expect("the program builds");

    let values = dense(&shape, &[3.0, 4.0]);
    let weights = dense(&shape, &[1.0, 2.0]);
    let value_key = InputKey::new("x").expect("a key");
    let weight_key = InputKey::new("w").expect("a key");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator")
        .evaluate(
            &program,
            &[
                InputBinding::new(&value_key, &values),
                InputBinding::new(&weight_key, &weights),
            ],
        )
        .expect("the program evaluates");
    let [result] = outputs.as_slice() else {
        panic!("the program has one output");
    };
    let bits: Vec<u32> = f32_elements(result)
        .expect("a dense f32 result")
        .iter()
        .map(|element| decode_f32(element).expect("decodes").to_bits())
        .collect();
    assert_eq!(bits, vec![0x3f59_3923, 0x4010_d0c2]);
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

/// A non-positive or non-finite `eps` refuses in the evaluator too.
///
/// The semantic inferencer already refuses these at construction; this function
/// is reachable without the registry, so the refusal is restated where it can
/// still fire. Each row was observed to refuse.
#[test]
fn an_inadmissible_eps_refuses_in_the_evaluator() {
    for bits in [
        0x0000_0000_u32,
        0x8000_0000,
        0xb586_37bd,
        0x7f80_0000,
        0x7fc0_0000,
    ] {
        assert!(
            rms_norm_f32(
                &shape(&[1, 2]),
                Axis::new(1),
                bits,
                &[1.0, 2.0],
                &[1.0, 1.0]
            )
            .is_err(),
            "eps {bits:#010x} must refuse"
        );
    }
    // The control: the governed payload evaluates.
    assert!(rms_norm_f32(&shape(&[1, 2]), Axis::new(1), EPS, &[1.0, 2.0], &[1.0, 1.0]).is_ok());
}

/// An axis outside the shape, or a payload of the wrong length, refuses.
#[test]
fn a_malformed_application_refuses_rather_than_guessing() {
    assert!(
        rms_norm_f32(&shape(&[1, 2]), Axis::new(2), EPS, &[1.0, 2.0], &[1.0, 1.0]).is_err(),
        "an out-of-range axis refuses"
    );
    assert!(
        rms_norm_f32(&shape(&[1, 2]), Axis::new(1), EPS, &[1.0], &[1.0, 1.0]).is_err(),
        "a short value payload refuses"
    );
    assert!(
        rms_norm_f32(&shape(&[1, 2]), Axis::new(1), EPS, &[1.0, 2.0], &[1.0]).is_err(),
        "a short weight payload refuses"
    );
}
