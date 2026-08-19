//! Unit evidence for the contraction reference's contract derivation and fold.
//!
//! The bit-exact corpus lives in `tests/contraction_conformance.rs`, which drives
//! the public boundary. These cover what that path cannot reach: the sole-decoder
//! derivation of the strict-cell contract, and the seeded fold, which no
//! registered contraction declares and which therefore has no public spelling to
//! drive it through. The perturbation table over the governed fact record lives
//! with the sole decoder in `tiler-ir` — a second per-field reading here is
//! exactly what the accepted successor contract forbids.

use tiler_ir::semantic::{ContractionIndex, ContractionIndexStructure, F32};
use tiler_ir::shape::Shape;

use super::{ContractionContract, ContractionSeed, contract_operands};
use crate::MAX_REFERENCE_TENSOR_ELEMENTS;
use crate::conformance::ReferenceNumericalConformance;
use crate::error::ReferenceOperationError;
use crate::evaluate::{f32_element, f32_elements};
use crate::tensor::Tensor;

/// The `td,od->to` structure the admitted profile names.
fn structure() -> ContractionIndexStructure {
    let index = ContractionIndex::new;
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("`td,od->to` is admitted")
}

fn tensor(dims: [u64; 2], bits: &[u32]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims(dims),
        bits.iter()
            .map(|bits| f32_element(f32::from_bits(*bits)))
            .collect::<Result<_, _>>()
            .expect("a four-byte payload"),
    )
    .expect("the fixture is well formed")
}

fn result_bits(tensor: &Tensor) -> Vec<u32> {
    f32_elements(tensor)
        .expect("a dense f32 result")
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

#[test]
fn the_governed_descriptor_derives_the_unseeded_binary32_fold() {
    let contract = ContractionContract::governed().expect("the governed contract derives");
    assert_eq!(contract.seed, ContractionSeed::FirstProduct);
    assert_eq!(contract.accumulator_type, F32::resolved_type());
    assert_eq!(contract.result_type, F32::resolved_type());
    assert_eq!(contract.canonical_nan_bits, 0x7fc0_0000);
}

/// The seed is a read parameter, and the two seeds compute different values.
///
/// This is the `negative_zero_seed` vector: every product is `-0.0`, so the
/// unseeded fold returns `0x80000000` and the `+0.0`-seeded one returns
/// `0x00000000`. It is retained as the permanent form of the regression the
/// ticket required watched failing — with the seed wrong, the corpus expectation
/// is the value on the right.
#[test]
fn the_first_product_seed_and_a_positive_zero_seed_disagree_on_signed_zero() {
    let structure = structure();
    let left = tensor([1, 16], &[0xbf80_0000; 16]);
    let right = tensor([1, 16], &[0x0000_0000; 16]);
    let governed = ContractionContract::governed().expect("the governed contract derives");
    assert_eq!(
        result_bits(
            &contract_operands(
                &governed,
                &structure,
                &left,
                &right,
                MAX_REFERENCE_TENSOR_ELEMENTS,
                ReferenceNumericalConformance::strict()
            )
            .unwrap()
        ),
        vec![0x8000_0000]
    );
    let seeded = governed.with_seed(ContractionSeed::Initial(0.0));
    assert_eq!(
        result_bits(
            &contract_operands(
                &seeded,
                &structure,
                &left,
                &right,
                MAX_REFERENCE_TENSOR_ELEMENTS,
                ReferenceNumericalConformance::strict()
            )
            .unwrap()
        ),
        vec![0x0000_0000]
    );
}

/// The declared empty-domain refusal, reached where only a direct application can.
///
/// A built program cannot carry it: the semantic inferencer refuses a zero
/// contracted extent at construction. Driving the evaluator directly is what
/// proves this branch can say no rather than being unreachable prose.
#[test]
fn an_empty_contracted_domain_is_refused_rather_than_returning_a_seed() {
    let contract = ContractionContract::governed().expect("the governed contract derives");
    let left = tensor([1, 0], &[]);
    let right = tensor([1, 0], &[]);
    assert_eq!(
        contract_operands(
            &contract,
            &structure(),
            &left,
            &right,
            MAX_REFERENCE_TENSOR_ELEMENTS,
            ReferenceNumericalConformance::strict()
        ),
        Err(ReferenceOperationError::InvalidApplication)
    );
    // The admitted neighbour, so the refusal above discriminates the empty domain
    // rather than the fixture.
    let left = tensor([1, 1], &[0x4000_0000]);
    let right = tensor([1, 1], &[0x4040_0000]);
    assert_eq!(
        result_bits(
            &contract_operands(
                &contract,
                &structure(),
                &left,
                &right,
                MAX_REFERENCE_TENSOR_ELEMENTS,
                ReferenceNumericalConformance::strict()
            )
            .unwrap()
        ),
        vec![0x40c0_0000]
    );
}

/// The fold's work bound refuses under its own variant, not the storage bound's.
///
/// Three cases, because one refusal proves nothing about *which* bound spoke.
/// With `d = 1` the output is the whole iteration space, so the stored-element
/// bound is what refuses — the meaning `OutputElementsExceeded` documents. With
/// `d = 2` and an output deliberately under that bound, no shape and no stored
/// result is over any limit; only the fold's step count is, and only the work
/// bound can name it. The third case is the same structure at a small extent, so
/// the two refusals discriminate the bound rather than the fixture family.
///
/// The allowance every case here passes is the default one, so this is also the
/// statement that an evaluator nobody told otherwise refuses exactly what it
/// always refused.
///
/// Every fixture is built from extents: both refusals happen before the fold
/// allocates its result or takes a step, so the test costs two small operands.
#[test]
fn an_iteration_space_over_the_bound_is_refused_as_iteration_work() {
    let contract = ContractionContract::governed().expect("the governed contract derives");
    let ones = |count: usize| vec![0x3f80_0000_u32; count];

    // `td,od->to` with `d = 1`: `t * o` steps producing `t * o` elements.
    let left = tensor([4096, 1], &ones(4096));
    let right = tensor([4097, 1], &ones(4097));
    assert_eq!(
        contract_operands(
            &contract,
            &structure(),
            &left,
            &right,
            MAX_REFERENCE_TENSOR_ELEMENTS,
            ReferenceNumericalConformance::strict()
        ),
        Err(ReferenceOperationError::OutputElementsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: 4096 * 4097,
        })
    );

    // `d = 2`: 8,389,712 output elements, under the limit, folded over two
    // contributors each — 16,779,424 multiply-accumulate steps, over it.
    let left = tensor([2896, 2], &ones(5792));
    let right = tensor([2897, 2], &ones(5794));
    // The premise the case rests on: this output is *not* over the storage bound,
    // so the refusal below can only be the work bound.
    const { assert!(2896 * 2897 <= MAX_REFERENCE_TENSOR_ELEMENTS) };
    assert_eq!(
        contract_operands(
            &contract,
            &structure(),
            &left,
            &right,
            MAX_REFERENCE_TENSOR_ELEMENTS,
            ReferenceNumericalConformance::strict()
        ),
        Err(ReferenceOperationError::IterationStepsExceeded {
            limit: MAX_REFERENCE_TENSOR_ELEMENTS,
            actual: 2896 * 2897 * 2,
        })
    );

    // The admitted neighbour: the same structure and the same `d = 2` fold, at an
    // extent whose work fits.
    let left = tensor([2, 2], &ones(4));
    let right = tensor([3, 2], &ones(6));
    assert_eq!(
        result_bits(
            &contract_operands(
                &contract,
                &structure(),
                &left,
                &right,
                MAX_REFERENCE_TENSOR_ELEMENTS,
                ReferenceNumericalConformance::strict()
            )
            .unwrap()
        ),
        vec![0x4000_0000; 6]
    );
}

/// A fold over one window is walked in several, and the allowance still refuses.
///
/// The fixture is chosen so that the *only* bound in play is the work bound: at
/// `t = o = 512` and `d = 65` the result is 262,144 elements and each operand
/// 33,280, all far under every storage limit, while the fold is 17,039,360
/// multiply-accumulate steps — over the 16,777,216 one window may walk. So a
/// single window cannot cover this result, and 258,111 output elements is the
/// widest one that can.
///
/// **The expectation is the linear index itself**, which is what makes a window
/// boundary observable rather than merely survived. The operands are built so
/// that `out[t][o] = 512 * t + o`, and that is exactly the row-major position of
/// the element — so a window that started at the wrong offset, folded a short
/// run, or repeated one, disagrees at the element where it went wrong instead of
/// producing a plausible constant. Every value is an integer below `2^24` and is
/// therefore exact in binary32.
///
/// The two refusals are the check saying no on both sides of the number it was
/// given: one step of allowance short of the fold declines it, and the fold's own
/// step count admits it.
#[test]
fn a_fold_over_one_window_is_walked_in_several_when_the_allowance_admits_it() {
    const T: usize = 512;
    const O: usize = 512;
    const D: usize = 65;
    const OUTPUTS: usize = T * O;
    const STEPS: usize = OUTPUTS * D;
    // The premises: only the work bound is in play, and one window cannot cover
    // the result.
    const { assert!(OUTPUTS <= MAX_REFERENCE_TENSOR_ELEMENTS) };
    const { assert!(STEPS > MAX_REFERENCE_TENSOR_ELEMENTS) };

    let contract = ContractionContract::governed().expect("the governed contract derives");
    // `a[t] = (t, 1, 0, ...)` and `b[o] = (512, o, 0, ...)`, so the fold's
    // ascending contributor sequence is `512 * t`, then `o`, then sixty-three
    // exact zeros.
    let left: Vec<u32> = (0..T)
        .flat_map(|t| {
            (0..D).map(move |d| match d {
                0 => exact(t),
                1 => exact(1),
                _ => 0,
            })
        })
        .collect();
    let right: Vec<u32> = (0..O)
        .flat_map(|o| {
            (0..D).map(move |d| match d {
                0 => exact(O),
                1 => exact(o),
                _ => 0,
            })
        })
        .collect();
    let left = tensor([T as u64, D as u64], &left);
    let right = tensor([O as u64, D as u64], &right);

    // The plan, so "it was walked in several windows" is a checked number rather
    // than an inference from the result having arrived.
    let planned = super::StagedStrictTensorContractionF32::governed(&structure(), &left, &right)
        .expect("the governed contraction plans");
    assert_eq!(planned.contracted_count(), D);
    assert_eq!(planned.output_count(), OUTPUTS);
    assert_eq!(
        planned.slab_output_count(),
        MAX_REFERENCE_TENSOR_ELEMENTS / D
    );
    assert_eq!(planned.slab_count(), 2);
    assert!(
        planned.slab_output_count() * planned.contracted_count() <= MAX_REFERENCE_TENSOR_ELEMENTS,
        "no window may walk more than one uninterrupted walk is held to"
    );

    // One step short of the fold, the allowance declines it and names both numbers.
    assert_eq!(
        contract_operands(
            &contract,
            &structure(),
            &left,
            &right,
            STEPS - 1,
            ReferenceNumericalConformance::strict()
        ),
        Err(ReferenceOperationError::IterationStepsExceeded {
            limit: STEPS - 1,
            actual: STEPS,
        })
    );

    let folded = contract_operands(
        &contract,
        &structure(),
        &left,
        &right,
        STEPS,
        ReferenceNumericalConformance::strict(),
    )
    .expect("the stated allowance admits this fold");
    let folded = result_bits(&folded);
    assert_eq!(folded.len(), OUTPUTS, "every output element is produced");
    // Reported by position rather than as a whole-vector equality: a window that
    // went wrong disagrees from one element onward, and 262,144 patterns in a
    // panic message would bury which element that was.
    let first_wrong = folded
        .iter()
        .enumerate()
        .find(|(linear, bits)| **bits != exact(*linear));
    assert!(
        first_wrong.is_none(),
        "the windowed fold disagrees with the linear index at {first_wrong:?}"
    );
}

/// Encodes a nonnegative integer below `2^24` as its exact binary32 pattern.
fn exact(value: usize) -> u32 {
    assert!(value < 1 << 24, "the fixture stays exact in binary32");
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer below 2^24 is exactly representable in binary32"
    )]
    let value = value as f32;
    value.to_bits()
}

/// A disagreeing extent on a shared index is refused, not silently truncated.
#[test]
fn a_contracted_extent_that_disagrees_between_operands_is_refused() {
    let contract = ContractionContract::governed().expect("the governed contract derives");
    let left = tensor([1, 3], &[0x3f80_0000; 3]);
    let right = tensor([1, 2], &[0x3f80_0000; 2]);
    assert_eq!(
        contract_operands(
            &contract,
            &structure(),
            &left,
            &right,
            MAX_REFERENCE_TENSOR_ELEMENTS,
            ReferenceNumericalConformance::strict()
        ),
        Err(ReferenceOperationError::InvalidApplication)
    );
}
