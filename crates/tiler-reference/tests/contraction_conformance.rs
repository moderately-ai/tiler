//! The exceptional-value corpus for `tiler::strict-tensor-contraction-f32@1`,
//! through the public semantic and reference boundary.
//!
//! # Where the numbers come from
//!
//! Every operand vector and every expectation is transcribed from the retained
//! Metal contraction spike, not re-derived here:
//!
//! - operands: `spikes/scheduling/metal_contraction_vertical/contraction_probe.py`,
//!   `semantic_cases()`;
//! - expectations: the `strict_fold` rows of
//!   `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/semantics-candidates.tsv`.
//!
//! Transcribing rather than recomputing is the point. A helper that re-derived
//! the expectation from the same fold the evaluator runs would agree with it for
//! reasons that say nothing about either being right — the independence rule the
//! two oracles in this crate are built on. The spike's values were produced by an
//! exact rational model in a different language and retained as evidence, so
//! agreeing with them is a claim about the *contract*, reproduced across two
//! independent implementations.
//!
//! The probe drives `16 x 16 x 16` so that every realization's structural
//! precondition is satisfied by one dispatch, and only `C[0, 0]` carries the
//! designed dot — every other operand entry is `+0.0`. The cases below drive
//! `[1, 16] x [1, 16] -> [1, 1]`, which is that same cell's products and that
//! same fold with the unpopulated rows dropped.
//!
//! # Which conformance level these results claim
//!
//! Exactly one, stated so a reader knows what a pass buys. The evaluator computes
//! under [`tiler_reference::ReferenceNumericalConformance::strict`]: both
//! subnormal dimensions preserved, separately rounded multiply and add, a strict
//! left fold in the declared contributor order, and bit-preserving signed zeros.
//! Its results are therefore *the* value the declared contract names, not a
//! member of an admitted set — which is only meaningful because the declared
//! signature forbids fusion, reassociation, and permutation. Under a contract
//! permitting any of those the answer would be a result set and no single value
//! would be the reference; the reference refuses such a declaration rather than
//! evaluating one reading of it.
//!
//! **The `+ftz` column of the spike's TSV is not this contract.** Flushing
//! subnormals is a property of the qualified Apple9/F32 target row, not of the
//! operation, which is why `subnormal_product` below expects `0x00400000` and not
//! the `0x00000000` that row produces. A device comparison against this oracle is
//! a comparison against the strict reading, and the flushing dimension has to be
//! declared on the comparison rather than absorbed here.
//!
//! **What a pass here is not.** It is evidence about the semantic contract and
//! the host reference evaluator. It is not evidence about any schedule, any
//! lowering, any compiled kernel, any device, or any model-level tolerance; no
//! such thing is exercised. Nor is it a universal claim: this is an exhaustive
//! pass over eight named exceptional cases, not a proof over the binary32 domain.

use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// The extent of the probe's contracted axis.
const K: usize = 16;

/// `td,od->to` over `[M, K] x [N, K] -> [M, N]` — the admitted index structure.
///
/// The contracted index is the *last* axis of both operands, because the pinned
/// workload's checkpoint stores every projection weight `[out, in]`. The ordinary
/// `[M, K] x [K, N]` spelling is a different structure.
fn td_od_to() -> ContractionIndexStructure {
    let index = ContractionIndex::new;
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("`td,od->to` is admitted")
}

fn tensor(dims: [u64; 2], bits: &[u32]) -> Tensor {
    let shape = Shape::from_dims(dims);
    assert_eq!(
        shape.element_count(),
        Some(bits.len()),
        "a fixture states every element"
    );
    Tensor::dense(
        F32::resolved_type(),
        shape,
        bits.iter()
            .map(|bits| {
                ReferenceElement::from_float_bits(
                    bits.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("an f32 payload is four bytes")
            })
            .collect(),
    )
    .expect("the fixture is well formed")
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a covered result is a dense f32 tensor");
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Evaluates one contraction through the public builder and reference evaluator.
fn contract(left: &Tensor, right: &Tensor) -> Vec<u32> {
    let structure = td_od_to();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let left_key = InputKey::new("left").expect("a valid key");
    let right_key = InputKey::new("right").expect("a valid key");
    let left_value = builder
        .input::<F32>(left_key.clone(), left.shape().clone())
        .expect("an F32 input");
    let right_value = builder
        .input::<F32>(right_key.clone(), right.shape().clone())
        .expect("an F32 input");
    let result = F32TensorContraction::apply(&mut builder, &structure, left_value, right_value)
        .expect("a covered occurrence is admitted");
    builder
        .output(OutputKey::new("result").expect("a valid key"), result)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(
            &program,
            &[
                InputBinding::new(&left_key, left),
                InputBinding::new(&right_key, right),
            ],
        )
        .expect("a covered program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    payload_bits(output)
}

/// Evaluates one `K`-wide dot as the single cell of a `[1, 1]` result.
fn dot(left: &[u32; K], right: &[u32; K]) -> u32 {
    let bits = contract(&tensor([1, K as u64], left), &tensor([1, K as u64], right));
    let [value] = bits.as_slice() else {
        panic!("a `[1, 1]` result has one element");
    };
    *value
}

/// Pads a designed prefix with the `+0.0` the probe's `pad` supplies.
fn pad(prefix: &[u32]) -> [u32; K] {
    let mut padded = [0x0000_0000_u32; K];
    padded[..prefix.len()].copy_from_slice(prefix);
    padded
}

// --- the eight cases --------------------------------------------------------

/// Execution witness: every topology agrees, so a failure here rules out the
/// whole column rather than discriminating within it.
#[test]
fn the_execution_witness_is_exactly_six() {
    assert_eq!(dot(&pad(&[0x4000_0000]), &pad(&[0x4040_0000])), 0x40c0_0000);
}

/// Order absorption: a `2^24` leading contributor absorbs `1.5` increments under
/// a left fold and does not under a wider or regrouped accumulation.
///
/// The retained candidates separate here: `strict_fold` and `zero_seed_fold` give
/// `0x4b800007`, the splits give `0x4b800006`, and the exact-then-round and
/// reversed topologies give `0x4b800005`. Reproducing the first is what says the
/// evaluator folds left in ascending order rather than accumulating widely.
#[test]
fn a_leading_large_contributor_absorbs_under_the_ascending_left_fold() {
    let left = pad(&[
        0x4b80_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
    ]);
    let right = pad(&[0x3f80_0000; 8]);
    assert_eq!(dot(&left, &right), 0x4b80_0007);

    // The perturbation, and a second retained candidate value: over the reversed
    // contributor sequence the same products fold to the spike's `reversed_fold`
    // value. Reversal is a permutation, which this family forbids, so a
    // permuting evaluator would return this instead — and it does not.
    let mut reversed_left = left;
    reversed_left.reverse();
    let mut reversed_right = right;
    reversed_right.reverse();
    assert_eq!(dot(&reversed_left, &reversed_right), 0x4b80_0005);
}

/// The separately-rounded-against-fused discriminator, in ADR 0015's sense of
/// contraction: the accumulator already holds `1.0` when an inexact product
/// arrives, which is the step a fused multiply-add would collapse to one
/// rounding.
///
/// `strict_fold` gives `0x3fc58f9e` and every fused candidate gives `0x3fc58f9d`.
/// With the inexact product first the two coincide and the case would report a
/// fusion-free evaluator whatever it did, so the order is load-bearing.
#[test]
fn a_separately_rounded_product_and_add_is_not_the_fused_value() {
    assert_eq!(
        dot(
            &pad(&[0x3f80_0000, 0x3eb9_7ef9]),
            &pad(&[0x3f80_0000, 0x3fc0_0000])
        ),
        0x3fc5_8f9e
    );
}

/// The searched vector separating strict, contiguous-split, strided-split, fused,
/// and infinitely wide accumulation — every one of those pairwise distinct.
///
/// This is the case that separates the contiguous split (`0xbb1d0683`) from the
/// strided one (`0xbb1d0672`); `order_absorption` does not.
#[test]
fn the_searched_split_separator_reproduces_the_strict_value() {
    assert_eq!(dot(&SPLIT_LEFT, &SPLIT_RIGHT), 0xbb1d_0482);

    // The reversed sequence, which is the spike's `reversed_fold` value here.
    let mut reversed_left = SPLIT_LEFT;
    reversed_left.reverse();
    let mut reversed_right = SPLIT_RIGHT;
    reversed_right.reverse();
    assert_eq!(dot(&reversed_left, &reversed_right), 0xbb1d_0494);
}

/// The signed-zero accumulator seed, where the seed is the only difference.
///
/// Every product is `-0.0`. A fold seeded from the first product returns
/// `0x80000000`; one seeded at `+0.0` returns `0x00000000`, because
/// `fl(+0.0 + -0.0)` is `+0.0`. The idiomatic accumulator-starts-at-zero loop
/// computes the second value, and this is the only case in the corpus that
/// notices.
#[test]
fn the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero() {
    assert_eq!(
        dot(&[0xbf80_0000; K], &[0x0000_0000; K]),
        0x8000_0000,
        "a `+0.0`-seeded fold returns 0x00000000 here"
    );
}

/// A non-canonical quiet NaN payload entering as a contributor.
///
/// Only the payload-propagating topology returns `0x7fc0dead`; every
/// canonicalizing one returns the governed `0x7fc00000`. The operand itself is
/// *read* unchanged — canonicalization applies to a produced value — so the
/// canonical result is evidence that the multiply's result was canonicalized
/// rather than that the operand was rewritten.
#[test]
fn a_non_canonical_nan_payload_is_replaced_by_the_governed_one() {
    assert_eq!(
        dot(
            &pad(&[0x7fc0_dead, 0x3f80_0000]),
            &pad(&[0x3f80_0000, 0x3f80_0000])
        ),
        0x7fc0_0000
    );
}

/// `inf * 0` formed *inside* the reduction rather than handed to it.
///
/// The distinction matters: an evaluator that only canonicalized NaN operands
/// would pass the payload case above and fail this one, because here no operand
/// is a NaN and the NaN is produced by the multiply.
#[test]
fn a_nan_the_reduction_forms_itself_is_canonicalized_too() {
    assert_eq!(
        dot(
            &pad(&[0x7f80_0000, 0x3f80_0000]),
            &pad(&[0x0000_0000, 0x3f80_0000])
        ),
        0x7fc0_0000
    );
}

/// A subnormal product, `2^-126 * 0.5 = 2^-127`.
///
/// Preserved, because the reference computes the strict reading. The measured
/// Apple9/F32 row flushes it to `0x00000000` under every math mode; that is a
/// target realization and is deliberately not this operation's contract.
#[test]
fn a_subnormal_product_is_preserved_rather_than_flushed() {
    assert_eq!(dot(&pad(&[0x0080_0000]), &pad(&[0x3f00_0000])), 0x0040_0000);
}

// --- the structure, beyond one cell -----------------------------------------

/// The fold is per output coordinate, over the structure's own index bindings.
///
/// Every case above reads `C[0, 0]`, which an evaluator that ignored the output
/// coordinate entirely would still pass. This drives `[2, 16] x [2, 16] -> [2, 2]`
/// with the witness in row zero and the split separator in row one, so the two
/// diagonal cells must reproduce two different retained values and the two
/// off-diagonal cells must equal neither.
#[test]
fn each_output_cell_folds_its_own_contributor_pair() {
    let mut left = [0x0000_0000_u32; 2 * K];
    let mut right = [0x0000_0000_u32; 2 * K];
    left[..K].copy_from_slice(&pad(&[0x4000_0000]));
    right[..K].copy_from_slice(&pad(&[0x4040_0000]));
    left[K..].copy_from_slice(&SPLIT_LEFT);
    right[K..].copy_from_slice(&SPLIT_RIGHT);

    let result = contract(
        &tensor([2, K as u64], &left),
        &tensor([2, K as u64], &right),
    );
    let [zero_zero, zero_one, one_zero, one_one] = result.as_slice() else {
        panic!("a `[2, 2]` result has four elements");
    };
    assert_eq!(*zero_zero, 0x40c0_0000, "row 0 against column 0 is 2 * 3");
    assert_eq!(
        *one_one, 0xbb1d_0482,
        "row 1 against column 1 is the split separator"
    );
    for cross in [zero_one, one_zero] {
        assert_ne!(*cross, 0x40c0_0000);
        assert_ne!(*cross, 0xbb1d_0482);
    }
}

/// The left operand of the searched split separator, exactly as the probe states it.
const SPLIT_LEFT: [u32; K] = [
    0xae8f_cc10,
    0xbef8_ce2c,
    0x3119_8e79,
    0xc009_21ca,
    0x3d9d_1929,
    0x4400_7fca,
    0x41b6_7583,
    0xbb63_d3c2,
    0x4328_602b,
    0x3cb3_5a07,
    0x2d94_1111,
    0xbab1_dbdb,
    0x44ab_a077,
    0x394a_4e66,
    0x3123_ac20,
    0xb603_7546,
];

/// The right operand of the searched split separator.
const SPLIT_RIGHT: [u32; K] = [
    0xb5d8_9c01,
    0xb26d_6247,
    0x3929_72f3,
    0x3a8b_ee84,
    0x30a8_8108,
    0x348c_ef8b,
    0x35b6_5100,
    0x440b_2b3f,
    0xb42c_ea8a,
    0xaf12_8d46,
    0xc613_5504,
    0xc4b2_44a6,
    0xb43b_1f42,
    0x360d_d62a,
    0xc5b5_87e5,
    0xb2f0_4f93,
];
