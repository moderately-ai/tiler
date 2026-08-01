//! Rotary position embedding as a checked composition over admitted families.
//!
//! # What this is, and what it deliberately is not
//!
//! Rotary embedding stays a *graph shape*. Nothing here registers a `Rope`
//! operation, declares a key, or adds a form to any family: the composition's
//! normative reference is the composition, and every occurrence below is
//! `tiler::reindex-f32@1`, `tiler::broadcast-f32@1`, `tiler::multiply-f32@1`, or
//! `tiler::add-f32@1` exactly as those four are already registered. That is the
//! whole claim the [L2 derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md)
//! and the [L4 program](../../../docs/research/program-planning/first-attention-program-vertical.md)
//! rest on when they remove a slice family and a concatenate family from the
//! pinned workload's requirements, and this file turns it from a claim into a
//! program that verifies and evaluates.
//!
//! # The composition, in order
//!
//! Ten occurrences over a `[T, heads, 128]` operand `x`, a `[2, 1]` sign input,
//! and `[T, 128]` `cos` and `sin` inputs. `rotate_half` is the first five.
//!
//! | # | family | form or mapping | shape |
//! | --- | --- | --- | --- |
//! | 1 | `Reindex` | `split-axis` on axis 2 into `(2, 64)` | `[T, h, 128] -> [T, h, 2, 64]` |
//! | 2 | `Reindex` | `reverse-axis` on axis 2 | `[T, h, 2, 64]` |
//! | 3 | `Broadcast` | `replicate, replicate, from-operand 0, stretch-unit 1` | `[2, 1] -> [T, h, 2, 64]` |
//! | 4 | `Multiply` | — | `[T, h, 2, 64]` |
//! | 5 | `Reindex` | `merge-axes` over axes 2 and 3 | `[T, h, 2, 64] -> [T, h, 128]` |
//! | 6 | `Broadcast` | `from-operand 0, replicate, from-operand 1` | `[T, 128] -> [T, h, 128]` |
//! | 7 | `Multiply` | — | `x · cos` |
//! | 8 | `Broadcast` | `from-operand 0, replicate, from-operand 1` | `[T, 128] -> [T, h, 128]` |
//! | 9 | `Multiply` | — | `rotate_half(x) · sin` |
//! | 10 | `Add` | — | `[T, h, 128]` |
//!
//! **Every broadcast is explicit and every axis mapping is stated.** The IR
//! admits no implicit broadcasting and the rank-zero admission on `Multiply` and
//! `Add` covers a rank-zero operand alone, so the two `[T, 128]` tables against
//! `[T, heads, 128]` and the `[2, 1]` sign against `[…, 2, 64]` are each an
//! occurrence of `tiler::broadcast-f32@1` carrying one entry per result axis.
//! The table mapping's interior `replicate` and the sign mapping's
//! `stretch-unit` are different relations, not two spellings of one: the table
//! has no head axis at all, while the sign's second axis exists with extent one.
//!
//! **The sign tensor is a program input, not a constant.** `ConstantF32::infer`
//! pushes `Shape::new([])`, so `tiler::constant-f32@1` produces rank zero only
//! and a two-element dense constant is inexpressible;
//! `the_sign_operand_cannot_be_a_constant` states that as a check rather than
//! leaving it to a reader. The `cos` and `sin`
//! tables are inputs for a different reason — they depend on position and
//! `rope_theta` alone and are host-precomputable, which removes `Sin` and `Cos`
//! from the executed program.
//!
//! # Where the compared values come from, and the boundary
//!
//! **The in-tree expectation is recomputed from the reference's own formula**,
//! `y = x · cos + rotate_half(x) · sin` with `rotate_half(x) = cat(−x₂, x₁)`,
//! by direct coordinate arithmetic rather than by a second run of the coordinate
//! maps — the same independence rule the rest of this crate's conformance is
//! built on. `rotate_half` itself is compared at the *bit* level: negating a
//! normal binary32 flips its sign bit exactly, so the expected payload of every
//! element is an operand payload with one bit changed, and no floating-point
//! arithmetic enters that comparison at all.
//!
//! **The tie to the pinned reference is the attention-block probe's**, at
//! `spikes/program-planning/attention-block-reference/`. Its retained record
//! `results/2026-07-31-c1-attention-block-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`
//! measured this composition against `modeling_qwen3.rotate_half` and
//! `apply_rotary_pos_emb` on a `[1, 16, 10, 128]` operand:
//! `rotate_half_composition_differing_elements` is 0 of 20,480,
//! `rope_q_composition_differing_elements` is 0 of 20,480, and
//! `rope_k_composition_differing_elements` is 0 of 10,240, with
//! `rotate_half_without_the_swap_differing_elements` and
//! `rotate_half_with_reversed_signs_differing_elements` both 20,480.
//!
//! **The boundary, stated rather than hidden.** That probe retains *counts* and
//! eight lane payloads, not the operand tensors, and its operands come from a
//! seeded `torch` generator this crate cannot reproduce. So the full-shape
//! comparison against the pinned reference's own numbers stays out of tree, and
//! what is in tree is: the composition at the workload's own extents — 16-head
//! query and 8-head key at head dimension 128 — against an independently
//! recomputed expectation, reproducing every count the probe records; plus
//! `the_pinned_rotate_half_lanes_are_reproduced`, which drives the probe's eight
//! retained input payloads through the composition and requires its eight
//! retained output payloads back, bit for bit. An in-tree pass is therefore
//! evidence that the composition denotes `cat(−x₂, x₁)` and the rotary formula;
//! that this is what `transformers` 4.51.0 computes is the probe's measurement,
//! and neither substitutes for the other.
//!
//! It establishes nothing about a plan, a kernel, a physical layout, or whether
//! any occurrence here costs a dispatch, and nothing about the rotary table's
//! construction, which is a host computation this file's fixtures deliberately
//! do not imitate.

use tiler_ir::semantic::{
    BroadcastAxisMapping, BroadcastAxisSource, BuildError, CanonicalField, CanonicalValue, F32,
    F32Add, F32Broadcast, F32Constant, F32Multiply, F32Reindex, InputKey, OpKey,
    OperationAttributes, OutputKey, REINDEX_FORM_AXIS, REINDEX_FORM_KIND,
    REINDEX_MAPPING_ATTRIBUTE, RegistryError, ReindexForm, SemanticProgram, SemanticProgramBuilder,
    Value, add_f32_op, broadcast_f32_op, multiply_f32_op, reindex_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

// --- the workload's own extents ---------------------------------------------

/// New positions of the C1 prefill row.
const T: usize = 10;
/// Query heads the pinned checkpoint declares.
const QUERY_HEADS: usize = 16;
/// Key and value heads, which are also the groups.
const KEY_VALUE_HEADS: usize = 8;
/// The *declared* head dimension, not `hidden_size / num_attention_heads`.
const HEAD_DIM: usize = 128;
/// Halves of the head axis the split produces: the size-two axis D-10 reverses.
const HALVES: usize = 2;
/// Width of one half.
const HALF: usize = HEAD_DIM / HALVES;

/// `−1.0` in binary32: the sign the first half of `cat(−x₂, x₁)` carries.
const NEGATIVE_ONE: u32 = 0xbf80_0000;
/// `+1.0` in binary32: the sign the second half carries.
const POSITIVE_ONE: u32 = 0x3f80_0000;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: usize) -> Extent {
    Extent::new(u64::try_from(value).expect("a workload extent fits a u64"))
}

fn workload_shape<const N: usize>(dims: [usize; N]) -> Shape {
    Shape::try_from_dims(
        dims.into_iter()
            .map(|dim| u64::try_from(dim).expect("a workload extent fits a u64")),
    )
    .expect("a workload shape is admitted")
}

// --- the forms and mappings the composition is built from -------------------

/// `[T, h, 128] -> [T, h, 2, 64]`: the half split, major factor first.
///
/// Row-major with the major factor first means result `(t, h, i, j)` reads lane
/// `64i + j`, so `i = 0` is the first half and `i = 1` the second — which is
/// what makes the reference's `x[..., :64]` and `x[..., 64:]` the two
/// coordinates of one axis rather than two slices.
fn half_split() -> ReindexForm {
    ReindexForm::split_axis(axis(2), [extent(HALVES), extent(HALF)]).expect("128 = 2 x 64")
}

/// The within-axis coordinate swap on the size-two axis, in the admitted form.
///
/// `reverse-axis` is `i -> extent − 1 − i`, and at extent two that is `i -> 1 − i`
/// exactly. D-10 admits this map and no other within-axis permutation, so the
/// composition consumes the settled form rather than asking for a new one.
fn within_axis_swap() -> ReindexForm {
    ReindexForm::reverse_axis(axis(2)).expect("the D-10 form is well shaped")
}

/// `[T, h, 2, 64] -> [T, h, 128]`: the merge that inverts the split.
fn half_merge() -> ReindexForm {
    ReindexForm::merge_axes([axis(2), axis(3)]).expect("axes 2 and 3 are an adjacent run")
}

/// `[2, 1] -> [T, heads, 2, 64]`: the sign operand's explicit axis mapping.
///
/// Two leading rank pads, a one-to-one correspondence on the size-two axis, and
/// an extent-one *stretch* on the second — the operand has that axis, so
/// widening it is `stretch-unit` and not `replicate`, and stating it the other
/// way is refused rather than normalized.
fn sign_mapping(heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(T), extent(heads), extent(HALVES), extent(HALF)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::StretchUnit(axis(1)),
        ],
    )
    .expect("the sign mapping accounts for every result axis")
}

/// `[T, 128] -> [T, heads, 128]`: the rotary tables' explicit axis mapping.
///
/// An *interior* rank pad. The table has a position axis and a lane axis and no
/// head axis at all, so the head axis is `replicate` while both of the table's
/// own axes are one-to-one — and the mapping names them in ascending order,
/// which is what a broadcast is allowed to do and a reordering is not.
fn table_mapping(heads: usize) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(T), extent(heads), extent(HEAD_DIM)],
        [
            BroadcastAxisSource::FromOperand(axis(0)),
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(1)),
        ],
    )
    .expect("the table mapping accounts for every result axis")
}

// --- the composition --------------------------------------------------------

/// Whether the composition carries the within-axis swap.
///
/// [`Swap::Dropped`] is the first retained perturbation: split and merge back
/// with nothing between them returns the operand, so the composition becomes
/// `cat(−x₁, x₂)` — identically shaped, and wrong at every element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Swap {
    Present,
    Dropped,
}

/// The four input handles one rotary program declares.
#[derive(Clone, Copy)]
struct RotaryOperands {
    operand: Value<F32>,
    sign: Value<F32>,
    cosine: Value<F32>,
    sine: Value<F32>,
}

/// Emits occurrences 1 through 5: `rotate_half(x) = cat(−x₂, x₁)`.
fn rotate_half(
    builder: &mut SemanticProgramBuilder,
    operands: RotaryOperands,
    heads: usize,
    swap: Swap,
) -> Value<F32> {
    let split = F32Reindex::apply(builder, &half_split(), operands.operand)
        .expect("128 = 2 x 64 on the head-dimension axis");
    let swapped = match swap {
        Swap::Present => F32Reindex::apply(builder, &within_axis_swap(), split)
            .expect("the size-two axis reverses"),
        Swap::Dropped => split,
    };
    let signs = F32Broadcast::apply(builder, &sign_mapping(heads), operands.sign)
        .expect("the sign operand broadcasts over the half width");
    let signed =
        F32Multiply::apply(builder, swapped, signs).expect("both operands are [T, h, 2, 64]");
    F32Reindex::apply(builder, &half_merge(), signed).expect("the two inner axes merge")
}

/// Emits the whole composition: `y = x · cos + rotate_half(x) · sin`.
fn rotary(
    builder: &mut SemanticProgramBuilder,
    operands: RotaryOperands,
    heads: usize,
    swap: Swap,
) -> (Value<F32>, Value<F32>) {
    let rotated = rotate_half(builder, operands, heads, swap);
    let mapping = table_mapping(heads);
    let cosine = F32Broadcast::apply(builder, &mapping, operands.cosine)
        .expect("the cosine table broadcasts over the head axis");
    let direct = F32Multiply::apply(builder, operands.operand, cosine)
        .expect("both operands are [T, h, 128]");
    let sine = F32Broadcast::apply(builder, &mapping, operands.sine)
        .expect("the sine table broadcasts over the head axis");
    let rotated_term =
        F32Multiply::apply(builder, rotated, sine).expect("both operands are [T, h, 128]");
    let sum = F32Add::apply(builder, direct, rotated_term).expect("both operands are [T, h, 128]");
    (rotated, sum)
}

// --- fixtures ---------------------------------------------------------------

/// Builds a tensor whose element `i` carries the big-endian bits of `bits(i)`.
fn tensor_of(shape: &Shape, bits: impl Fn(usize) -> u32) -> Tensor {
    let count = shape.element_count().expect("a workload shape is bounded");
    let elements = (0..count)
        .map(|index| {
            ReferenceElement::from_float_bits(
                bits(index).to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .expect("an f32 payload is four bytes")
        })
        .collect();
    Tensor::dense(F32::resolved_type(), shape.clone(), elements).expect("the tensor is well formed")
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a rotary result is a dense f32 tensor");
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

/// Deterministic pairwise-distinct normal binary32 payloads of magnitude in `[1, 2)`.
///
/// The mantissa is `(index + salt) · K mod 2²³` with `K` odd, so it is injective
/// in the index over every fixture this file builds — no two elements of one
/// operand carry the same payload. That is what makes the perturbation counts
/// below exact rather than probable: `cat(−x₂, x₁)` and `cat(−x₁, x₂)` differ at
/// an element exactly when the two halves' payloads differ there, and injectivity
/// says they always do. Every payload is normal and nonzero, so negation is a
/// sign-bit flip and no product or sum underflows, overflows, or is exceptional.
fn sample_bits(salt: u32, index: usize) -> u32 {
    let mixed = u32::try_from(index)
        .expect("a fixture index is small")
        .wrapping_add(salt)
        .wrapping_mul(2_654_435_761);
    (mixed & 0x8000_0000) | POSITIVE_ONE | (mixed & 0x007f_ffff)
}

/// The operand, sign, and table tensors one comparison binds.
struct RotaryFixture {
    operand: Tensor,
    sign: Tensor,
    cosine: Tensor,
    sine: Tensor,
}

/// Whether the sign operand carries `[−1, +1]` or the reversed `[+1, −1]`.
///
/// [`SignOrder::Reversed`] is the second retained perturbation, and it is an
/// operand perturbation rather than a graph one: the program is byte-identical
/// and only the eight bytes of the sign input change, which is exactly the
/// mistake a hand-written `rotate_half` makes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SignOrder {
    Negated,
    Reversed,
}

fn fixture(heads: usize, sign: SignOrder) -> RotaryFixture {
    let signs = match sign {
        SignOrder::Negated => [NEGATIVE_ONE, POSITIVE_ONE],
        SignOrder::Reversed => [POSITIVE_ONE, NEGATIVE_ONE],
    };
    RotaryFixture {
        operand: tensor_of(&workload_shape([T, heads, HEAD_DIM]), |index| {
            sample_bits(1, index)
        }),
        sign: tensor_of(&workload_shape([HALVES, 1]), |index| signs[index]),
        cosine: tensor_of(&workload_shape([T, HEAD_DIM]), |index| {
            sample_bits(2, index)
        }),
        sine: tensor_of(&workload_shape([T, HEAD_DIM]), |index| {
            sample_bits(3, index)
        }),
    }
}

// --- evaluation -------------------------------------------------------------

/// The two payload streams one evaluation of the composition produces.
struct RotaryResult {
    rotate_half: Vec<u32>,
    rotary: Vec<u32>,
    shape: Shape,
}

fn operand_key() -> InputKey {
    InputKey::new("operand").expect("a valid key")
}

fn sign_key() -> InputKey {
    InputKey::new("sign").expect("a valid key")
}

fn cosine_key() -> InputKey {
    InputKey::new("cos").expect("a valid key")
}

fn sine_key() -> InputKey {
    InputKey::new("sin").expect("a valid key")
}

/// Declares the four inputs and returns them together with the builder's handles.
fn declare(builder: &mut SemanticProgramBuilder, heads: usize) -> RotaryOperands {
    RotaryOperands {
        operand: builder
            .input::<F32>(operand_key(), workload_shape([T, heads, HEAD_DIM]))
            .expect("an F32 input"),
        sign: builder
            .input::<F32>(sign_key(), workload_shape([HALVES, 1]))
            .expect("an F32 input"),
        cosine: builder
            .input::<F32>(cosine_key(), workload_shape([T, HEAD_DIM]))
            .expect("an F32 input"),
        sine: builder
            .input::<F32>(sine_key(), workload_shape([T, HEAD_DIM]))
            .expect("an F32 input"),
    }
}

/// Builds the composition as a two-output program.
///
/// Both `rotate_half(x)` and the whole rotary result are named outputs of one
/// program rather than two programs: multi-result support is part of the
/// semantic contract, and it keeps every occurrence — and every perturbation of
/// one — shared between the two comparisons instead of duplicated.
fn build_program(heads: usize, swap: Swap) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let operands = declare(&mut builder, heads);
    let (rotated, sum) = rotary(&mut builder, operands, heads, swap);
    builder
        .output(OutputKey::new("rotate-half").expect("a valid key"), rotated)
        .expect("an output");
    builder
        .output(OutputKey::new("rotary").expect("a valid key"), sum)
        .expect("an output");
    builder.build().expect("the composition is complete")
}

fn evaluate(heads: usize, swap: Swap, fixture: &RotaryFixture) -> RotaryResult {
    let program = build_program(heads, swap);
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(
            &program,
            &[
                InputBinding::new(&operand_key(), &fixture.operand),
                InputBinding::new(&sign_key(), &fixture.sign),
                InputBinding::new(&cosine_key(), &fixture.cosine),
                InputBinding::new(&sine_key(), &fixture.sine),
            ],
        )
        .expect("the composition evaluates");
    let [rotate_half, rotary] = outputs.as_slice() else {
        panic!("the composition has two outputs");
    };
    assert_eq!(rotate_half.shape(), rotary.shape());
    RotaryResult {
        rotate_half: payload_bits(rotate_half),
        rotary: payload_bits(rotary),
        shape: rotary.shape().clone(),
    }
}

// --- the independent expectations -------------------------------------------

/// `cat(−x₂, x₁)`, derived at the bit level from the operand's own payloads.
///
/// No floating-point arithmetic enters this: every operand payload is a normal
/// binary32, so `−v` is `v` with its sign bit flipped, and the expected result
/// is a permutation of the operand's payloads with one bit changed in half of
/// them. The composition reaches the same bits through a reversal and a multiply
/// by `±1`, which is a different route to the same answer rather than the same
/// route twice.
fn expected_rotate_half(operand: &[u32], heads: usize) -> Vec<u32> {
    let mut expected = Vec::with_capacity(T * heads * HEAD_DIM);
    for row in 0..T * heads {
        let base = row * HEAD_DIM;
        for lane in 0..HALF {
            expected.push(operand[base + HALF + lane] ^ 0x8000_0000);
        }
        for lane in 0..HALF {
            expected.push(operand[base + lane]);
        }
    }
    expected
}

/// `y = x · cos + rotate_half(x) · sin`, evaluated coordinate by coordinate.
///
/// Two separate roundings and then a third, matching the reference's own
/// separate multiply-multiply-add: Rust never contracts `a * b + c * d` into a
/// fused multiply-add, and the governed `tiler::multiply-f32@1` and
/// `tiler::add-f32@1` forbid the contraction, so the two agree on every bit or
/// neither is right.
fn expected_rotary(
    operand: &[u32],
    rotated: &[u32],
    cosine: &[u32],
    sine: &[u32],
    heads: usize,
) -> Vec<u32> {
    let mut expected = Vec::with_capacity(T * heads * HEAD_DIM);
    for position in 0..T {
        for head in 0..heads {
            for lane in 0..HEAD_DIM {
                let index = (position * heads + head) * HEAD_DIM + lane;
                let table = position * HEAD_DIM + lane;
                let direct = f32::from_bits(operand[index]) * f32::from_bits(cosine[table]);
                let turned = f32::from_bits(rotated[index]) * f32::from_bits(sine[table]);
                expected.push((direct + turned).to_bits());
            }
        }
    }
    expected
}

fn differing(left: &[u32], right: &[u32]) -> usize {
    assert_eq!(left.len(), right.len());
    left.iter()
        .zip(right)
        .filter(|(left, right)| left != right)
        .count()
}

// --- the composition's shape ------------------------------------------------

#[test]
fn the_composition_is_ten_occurrences_over_four_admitted_families() {
    let program = build_program(QUERY_HEADS, Swap::Present);
    let keys: Vec<OpKey> = program
        .operations()
        .map(|operation| operation.key().clone())
        .collect();
    assert_eq!(
        keys,
        vec![
            reindex_f32_op(),
            reindex_f32_op(),
            broadcast_f32_op(),
            multiply_f32_op(),
            reindex_f32_op(),
            broadcast_f32_op(),
            multiply_f32_op(),
            broadcast_f32_op(),
            multiply_f32_op(),
            add_f32_op(),
        ],
        "rotary embedding is a shape over the four already-registered families \
         and admits no new key"
    );
    assert_eq!(program.input_count(), 4);
    assert_eq!(program.output_count(), 2);

    // The first perturbation, stated as a structural fact rather than assumed:
    // dropping the swap removes exactly one occurrence and nothing else, so the
    // counts it produces below measure the reversal and not a second difference
    // that came along with it.
    let without_swap = build_program(QUERY_HEADS, Swap::Dropped);
    assert_eq!(without_swap.operation_count(), keys.len() - 1);
    assert_eq!(
        without_swap
            .operations()
            .filter(|operation| operation.key() == &reindex_f32_op())
            .count(),
        2
    );

    // The derived shapes, from the forms alone rather than from the graph: each
    // family derives its result from its operand, so agreeing here is agreeing
    // about what the composition means.
    let operand = workload_shape([T, QUERY_HEADS, HEAD_DIM]);
    let split = half_split()
        .result_shape(&operand)
        .expect("128 = 2 x 64 on the head-dimension axis");
    assert_eq!(split, workload_shape([T, QUERY_HEADS, HALVES, HALF]));
    assert_eq!(
        within_axis_swap()
            .result_shape(&split)
            .expect("the size-two axis reverses"),
        split,
        "a within-axis reversal changes the reading order and not the shape"
    );
    assert_eq!(
        half_merge()
            .result_shape(&split)
            .expect("the two inner axes merge"),
        operand
    );
    assert_eq!(
        sign_mapping(QUERY_HEADS)
            .result_shape(&workload_shape([HALVES, 1]))
            .expect("the sign operand is [2, 1]"),
        split
    );
    assert_eq!(
        table_mapping(QUERY_HEADS)
            .result_shape(&workload_shape([T, HEAD_DIM]))
            .expect("a rotary table is [T, 128]"),
        operand
    );
}

#[test]
fn the_sign_operand_cannot_be_a_constant() {
    // Why the `[2, 1]` sign enters as an input. `tiler::constant-f32@1` produces
    // rank zero and nothing else, so a two-element dense constant is not
    // expressible in the family; the alternative is not a worse constant but no
    // constant at all, and eight bytes of program input is what replaces it.
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let constant = F32Constant::apply(&mut builder, NEGATIVE_ONE).expect("a scalar constant");
    builder
        .output(OutputKey::new("constant").expect("a valid key"), constant)
        .expect("an output");
    let program = builder.build().expect("the constant program is complete");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[])
        .expect("a constant program evaluates");
    let [scalar] = outputs.as_slice() else {
        panic!("the constant program has one output");
    };
    assert_eq!(scalar.shape().rank(), 0);
    assert_eq!(payload_bits(scalar), vec![NEGATIVE_ONE]);

    // And the sign mapping refuses that operand by name, because a rank-zero
    // value has neither of the two axes the mapping consumes. No broadcast of
    // one constant could have stood in regardless: the two halves carry
    // *different* signs, and a broadcast replicates one value rather than
    // inventing a second.
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let constant = F32Constant::apply(&mut builder, NEGATIVE_ONE).expect("a scalar constant");
    assert_eq!(
        refusal_code(
            &F32Broadcast::apply(&mut builder, &sign_mapping(QUERY_HEADS), constant).unwrap_err()
        ),
        "broadcast.mapping.operand-axes-unconsumed"
    );

    // The admitted neighbour: the `[2, 1]` input the composition actually
    // declares, so the refusal above discriminates the operand rather than the
    // mapping.
    let sign = builder
        .input::<F32>(sign_key(), workload_shape([HALVES, 1]))
        .expect("an F32 input");
    assert!(F32Broadcast::apply(&mut builder, &sign_mapping(QUERY_HEADS), sign).is_ok());
}

// --- the comparison, at the workload's own extents --------------------------

/// Runs the whole comparison at one head count and returns the four counts.
///
/// Returned rather than asserted inside so each caller states its own numbers
/// literally: a shared assertion helper would let one head count's evidence
/// stand in for the other's.
struct RotaryCounts {
    elements: usize,
    rotate_half_differing: usize,
    rotary_differing: usize,
    rotate_half_without_swap: usize,
    rotate_half_reversed_signs: usize,
    rotary_without_swap: usize,
    rotary_reversed_signs: usize,
}

fn compare(heads: usize) -> RotaryCounts {
    let admitted = fixture(heads, SignOrder::Negated);
    let reversed = fixture(heads, SignOrder::Reversed);
    let operand = payload_bits(&admitted.operand);
    let cosine = payload_bits(&admitted.cosine);
    let sine = payload_bits(&admitted.sine);

    let expected_rotated = expected_rotate_half(&operand, heads);
    let expected = expected_rotary(&operand, &expected_rotated, &cosine, &sine, heads);

    let result = evaluate(heads, Swap::Present, &admitted);
    assert_eq!(result.shape, workload_shape([T, heads, HEAD_DIM]));

    // The two perturbations, at identical shapes. Neither is a different program
    // shape a reader could have spotted: one drops a single occurrence and the
    // other changes eight bytes of input.
    let without_swap = evaluate(heads, Swap::Dropped, &admitted);
    let reversed_signs = evaluate(heads, Swap::Present, &reversed);
    assert_eq!(without_swap.shape, result.shape);
    assert_eq!(reversed_signs.shape, result.shape);

    RotaryCounts {
        elements: expected.len(),
        rotate_half_differing: differing(&result.rotate_half, &expected_rotated),
        rotary_differing: differing(&result.rotary, &expected),
        rotate_half_without_swap: differing(&without_swap.rotate_half, &expected_rotated),
        rotate_half_reversed_signs: differing(&reversed_signs.rotate_half, &expected_rotated),
        rotary_without_swap: differing(&without_swap.rotary, &expected),
        rotary_reversed_signs: differing(&reversed_signs.rotary, &expected),
    }
}

#[test]
fn the_query_operand_matches_the_rotary_formula_at_sixteen_heads() {
    let counts = compare(QUERY_HEADS);

    // 16 x 10 x 128, the probe's `rope_q_element_count`.
    assert_eq!(counts.elements, 20_480);
    assert_eq!(
        counts.rotate_half_differing, 0,
        "the split, the reversal, the sign multiply, and the merge denote cat(-x2, x1)"
    );
    assert_eq!(
        counts.rotary_differing, 0,
        "x * cos + rotate_half(x) * sin, bit for bit"
    );

    // The retained perturbations. Both counts are exact rather than probable:
    // the fixture's payloads are pairwise distinct, so the two halves differ at
    // every lane, and reversing the sign negates every element of a tensor with
    // no zero in it.
    assert_eq!(
        counts.rotate_half_without_swap, 20_480,
        "without the swap the composition is cat(-x1, x2), which is identically shaped"
    );
    assert_eq!(
        counts.rotate_half_reversed_signs, 20_480,
        "with the signs reversed the composition is cat(x2, -x1), which is the negation"
    );
    assert_eq!(counts.rotary_without_swap, 20_480);
    assert_eq!(counts.rotary_reversed_signs, 20_480);
}

#[test]
fn the_key_operand_matches_the_rotary_formula_at_eight_heads() {
    let counts = compare(KEY_VALUE_HEADS);

    // 8 x 10 x 128: the key operand, which the probe counts separately because
    // grouped-query attention gives it half the query's heads and the same head
    // dimension and the same tables.
    assert_eq!(counts.elements, 10_240);
    assert_eq!(counts.rotate_half_differing, 0);
    assert_eq!(counts.rotary_differing, 0);
    assert_eq!(counts.rotate_half_without_swap, 10_240);
    assert_eq!(counts.rotate_half_reversed_signs, 10_240);
    assert_eq!(counts.rotary_without_swap, 10_240);
    assert_eq!(counts.rotary_reversed_signs, 10_240);
}

// --- the pinned lanes -------------------------------------------------------

/// The eight payloads the attention-block probe retained on either side of `rotate_half`.
///
/// From
/// `spikes/program-planning/attention-block-reference/results/2026-07-31-c1-attention-block-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`,
/// rows `rotate_half_input_lanes_0_3`, `rotate_half_input_lanes_64_67`,
/// `rotate_half_output_lanes_0_3`, and `rotate_half_output_lanes_64_67`. These
/// are `transformers` 4.51.0's own bits, transcribed rather than recomputed;
/// everything else in this file is derived in tree.
const PINNED_INPUT_LANES_0_3: [u32; 4] = [0x3e7e_d70a, 0x3fe3_8215, 0xbef3_454d, 0x3f0e_7721];
const PINNED_INPUT_LANES_64_67: [u32; 4] = [0x3f80_ccb2, 0xbf1d_b431, 0x3e56_2f44, 0x3f24_d1ed];
const PINNED_OUTPUT_LANES_0_3: [u32; 4] = [0xbf80_ccb2, 0x3f1d_b431, 0xbe56_2f44, 0xbf24_d1ed];
const PINNED_OUTPUT_LANES_64_67: [u32; 4] = [0x3e7e_d70a, 0x3fe3_8215, 0xbef3_454d, 0x3f0e_7721];

#[test]
fn the_pinned_rotate_half_lanes_are_reproduced() {
    // The probe's eight retained input payloads, placed at the lanes it read
    // them from on the first position of the first head. The remaining lanes
    // come from this file's own fixture, because the probe retained eight of a
    // hundred and twenty-eight and inventing the other hundred and twenty would
    // be a fabricated fixture rather than a pinned one.
    let pinned = |index: usize| match index {
        0..=3 => PINNED_INPUT_LANES_0_3[index],
        64..=67 => PINNED_INPUT_LANES_64_67[index - 64],
        _ => sample_bits(1, index),
    };
    let operand = tensor_of(&workload_shape([T, QUERY_HEADS, HEAD_DIM]), pinned);
    let admitted = RotaryFixture {
        operand: operand.clone(),
        ..fixture(QUERY_HEADS, SignOrder::Negated)
    };

    let result = evaluate(QUERY_HEADS, Swap::Present, &admitted);
    assert_eq!(result.rotate_half[0..4], PINNED_OUTPUT_LANES_0_3);
    assert_eq!(result.rotate_half[64..68], PINNED_OUTPUT_LANES_64_67);

    // And the perturbations move exactly these lanes too, so the eight-lane
    // comparison is discriminating rather than a coincidence of the fixture.
    let without_swap = evaluate(QUERY_HEADS, Swap::Dropped, &admitted);
    assert_ne!(without_swap.rotate_half[0..4], PINNED_OUTPUT_LANES_0_3);
    assert_ne!(without_swap.rotate_half[64..68], PINNED_OUTPUT_LANES_64_67);

    let reversed_signs = RotaryFixture {
        operand,
        ..fixture(QUERY_HEADS, SignOrder::Reversed)
    };
    let reversed = evaluate(QUERY_HEADS, Swap::Present, &reversed_signs);
    assert_ne!(reversed.rotate_half[0..4], PINNED_OUTPUT_LANES_0_3);
    assert_ne!(reversed.rotate_half[64..68], PINNED_OUTPUT_LANES_64_67);
}

// --- the explainable refusal ------------------------------------------------

/// Returns the provider diagnostic code a refused application carried.
///
/// Asserting the code rather than only `is_err` is what makes the refusal below
/// evidence about the rule it names: a poisoned builder, a foreign handle, or a
/// bound violation would all be errors too, and none of them would be the check.
fn refusal_code(error: &BuildError) -> String {
    let BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(rejection)) =
        error
    else {
        panic!("a form refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// Builds a reindex mapping attribute naming a form outside the admitted set.
///
/// A within-axis *rotation* `i -> (i + k) mod n`, which the accepted index
/// vocabulary can express quasi-affinely and which D-10 leaves deliberately
/// unadmitted. At extent two it even agrees with the reversal on every
/// coordinate, so nothing about the resulting tensor would reveal the
/// substitution — the family has to refuse it by name or not at all.
fn rotate_axis_attribute() -> OperationAttributes {
    let form = CanonicalValue::record([
        CanonicalField::new(
            REINDEX_FORM_KIND,
            CanonicalValue::utf8("rotate-axis").expect("a bounded name"),
        ),
        CanonicalField::new(REINDEX_FORM_AXIS, CanonicalValue::unsigned_u32(2)),
    ])
    .expect("a two-field record is canonical");
    OperationAttributes::new([CanonicalField::new(REINDEX_MAPPING_ATTRIBUTE, form)])
        .expect("one attribute is canonical")
}

#[test]
fn an_unadmitted_within_axis_map_refuses_by_name_at_construction() {
    // Decoding alone refuses it, before any operand exists: which form a mapping
    // names is a property of the attribute.
    let attributes = rotate_axis_attribute();
    let value = attributes
        .get(REINDEX_MAPPING_ATTRIBUTE)
        .expect("the mapping attribute");
    let rejected = ReindexForm::from_canonical_value(value).expect_err("rotate-axis is unadmitted");
    assert_eq!(rejected.diagnostic_code(), "reindex.form.unadmitted-kind");
    assert!(
        rejected.to_string().contains("rotate-axis"),
        "the refusal names the rejected form rather than reporting an anonymous invalidity: {rejected}"
    );

    // And through an occurrence, at the exact point of the composition where the
    // swap belongs — a `[T, 16, 2, 64]` operand and axis 2. The refusal names the
    // form, not totality: the map *is* total and bijective, and is refused all
    // the same because the family admits forms rather than properties.
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let operands = declare(&mut builder, QUERY_HEADS);
    let split = F32Reindex::apply(&mut builder, &half_split(), operands.operand)
        .expect("128 = 2 x 64 on the head-dimension axis");
    let refused = builder
        .apply(reindex_f32_op(), rotate_axis_attribute(), &[split.erase()])
        .expect_err("rotate-axis is unadmitted at an occurrence too");
    assert_eq!(refusal_code(&refused), "reindex.form.unadmitted-kind");

    // The admitted neighbour, so the refusal is known to discriminate rather
    // than to refuse every within-axis map presented here.
    assert!(F32Reindex::apply(&mut builder, &within_axis_swap(), split).is_ok());
}
