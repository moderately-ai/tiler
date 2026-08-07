//! Bounded conformance evidence for `tiler::gather-f32@1`, through the public
//! semantic and reference boundary.
//!
//! # What this covers, exactly
//!
//! The five cases the family's ticket names as its bounded evidence — an index
//! at the first coordinate, an index at the last coordinate, a repeated index, an
//! out-of-range index, and an empty index operand — plus the gathered axis in
//! each position of a rank-three source, a rank-two index operand, a rank-zero
//! index operand, and the exceptional payloads the family must transport rather
//! than compute. Every source is `tiler::f32@1`, every index operand is
//! `tiler::u32@1`, and every extent is static.
//!
//! **What it does not cover.** The pinned model's own `[151936, 1024]` extents
//! are exercised only as a *shape* — at the semantic layer, in the family's own
//! unit tests — and never as a materialized fixture, because 155,582,464 elements
//! is far outside the reference evaluator's governed tensor bound and a
//! conformance row is not a place to discover that. The last-coordinate case
//! below therefore uses `151935`'s structural analogue, the last coordinate of
//! its own axis, which is the property the case exists to check. Also not
//! covered: any compiled or executed realization, any index type but
//! `tiler::u32@1`, scatter, and any data-dependent output shape. **A pass here is
//! evidence about the semantic contract and the reference evaluator, and about
//! nothing below them.** No program containing this family reaches a plan: no
//! lowering capability resolves it and no fusion role classifies it, so an
//! occurrence fails closed at the request boundary by construction.
//!
//! # Why the expectations are written out
//!
//! Each source payload at a coordinate is that coordinate's own row-major linear
//! index, so a result element names the source position it came from and an
//! evaluator reading the wrong element produces a visibly wrong value rather than
//! a coincidence. Each expectation is a literal list of those indices, derived by
//! hand from the index operand rather than by a second implementation of the
//! addressing — a helper that recomputed the offsets would agree with the
//! evaluator for reasons that say nothing about either being right. The retained
//! perturbations each demonstrate the comparison failing.

use tiler_ir::semantic::{
    F32, F32Gather, InputKey, OutputKey, SemanticProgramBuilder, gather_index_resolved_type,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, ReferenceOperationError,
    Tensor, TensorPayloadView,
};

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a covered shape is bounded")
}

fn f32_element(bits: u32) -> ReferenceElement {
    ReferenceElement::from_float_bits(bits.to_be_bytes(), FloatBitOrder::MostSignificantByteFirst)
        .expect("a covered payload is four bytes")
}

fn index_tensor(dims: &[u64], values: &[u32]) -> Tensor {
    let shape = shape(dims);
    assert_eq!(
        shape.element_count(),
        Some(values.len()),
        "a fixture states exactly one coordinate per index position"
    );
    Tensor::dense(
        gather_index_resolved_type(),
        shape,
        values
            .iter()
            .map(|value| {
                ReferenceElement::new(value.to_be_bytes()).expect("an index element is four bytes")
            })
            .collect(),
    )
    .expect("the fixture index tensor is well formed")
}

fn source_tensor(dims: &[u64], bits: &[u32]) -> Tensor {
    let shape = shape(dims);
    assert_eq!(
        shape.element_count(),
        Some(bits.len()),
        "a fixture states exactly one payload per coordinate"
    );
    Tensor::dense(
        F32::resolved_type(),
        shape,
        bits.iter().copied().map(f32_element).collect(),
    )
    .expect("the fixture source tensor is well formed")
}

/// A source whose payload at every coordinate is that coordinate's linear index.
fn positional(dims: &[u64]) -> Tensor {
    let count = shape(dims)
        .element_count()
        .expect("a covered fixture is small");
    let bits: Vec<u32> = (0..count)
        .map(|position| u32::try_from(position).expect("a covered fixture is small"))
        .collect();
    source_tensor(dims, &bits)
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

/// Evaluates one gather, returning the result payloads and shape.
fn gather(
    source: &Tensor,
    index: &Tensor,
    axis: u32,
) -> Result<(Vec<u32>, Shape), ReferenceOperationError> {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let source_key = InputKey::new("source").expect("a covered key is bounded");
    let index_key = InputKey::new("index").expect("a covered key is bounded");
    let source_value = builder
        .input::<F32>(source_key.clone(), source.shape().clone())
        .expect("an F32 source input");
    // The index operand has no Rust marker, so it is declared through the
    // runtime-resolved path against the registry-admitted `tiler::u32@1`.
    let index_value = builder
        .input_resolved(
            index_key.clone(),
            index.shape().clone(),
            gather_index_resolved_type(),
        )
        .expect("a u32 index input");
    let result = F32Gather::apply(&mut builder, source_value, index_value, Axis::new(axis))
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
                InputBinding::new(&source_key, source),
                InputBinding::new(&index_key, index),
            ],
        );
    match outputs {
        Ok(outputs) => {
            let [output] = outputs.as_slice() else {
                panic!("a covered program has one output");
            };
            Ok((payload_bits(output), output.shape().clone()))
        }
        Err(error) => Err(operation_error(&error)),
    }
}

/// Extracts the operation-level cause of an evaluation failure.
fn operation_error(error: &tiler_reference::EvaluationError) -> ReferenceOperationError {
    match error {
        tiler_reference::EvaluationError::Operation { source, .. } => source.clone(),
        other => panic!("a covered refusal is an operation failure, not {other}"),
    }
}

// --- The embedding-lookup shape, at the extents a fixture can carry -----------

/// The workload's own occurrence shape: `[V, H]` gathered on axis 0 by `[T]`.
///
/// `V = 5`, `H = 3`, `T = 4` stands in for `151936`, `1024`, and the conformance
/// row's `T`. What is checked is the composition rule and the row transport, both
/// of which are extent-independent.
#[test]
fn a_token_id_operand_selects_whole_rows_of_the_embedding_matrix() {
    let source = positional(&[5, 3]);
    let (bits, result) = gather(&source, &index_tensor(&[4], &[2, 0, 4, 1]), 0)
        .expect("a covered occurrence evaluates");
    assert_eq!(result, shape(&[4, 3]));
    assert_eq!(bits, vec![6, 7, 8, 0, 1, 2, 12, 13, 14, 3, 4, 5]);

    // Perturbation: the rows in the source's own order, which is what an
    // evaluator that ignored the index operand and copied a prefix would return.
    // It differs at all twelve positions. Row 0 is the one row this index operand
    // selects that a prefix would also produce, and it still differs everywhere
    // because the index operand places it *second* — so the perturbation
    // separates reading the right rows from placing them in the right order,
    // which a permutation-blind evaluator would confuse.
    let ignored_index = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
    assert_eq!(
        bits.iter()
            .zip(&ignored_index)
            .filter(|(left, right)| left != right)
            .count(),
        12
    );
}

/// Token ID 0 reads the first row, and the last valid ID reads the last row.
///
/// The two boundary coordinates are asserted in one occurrence and then each is
/// perturbed separately, because an off-by-one at either end is a different
/// defect: reading `1` for `0` and reading `extent - 2` for `extent - 1` are
/// produced by different mistakes.
#[test]
fn the_first_and_last_valid_coordinates_both_read_their_own_row() {
    let source = positional(&[5, 3]);
    let (bits, result) =
        gather(&source, &index_tensor(&[2], &[0, 4]), 0).expect("a covered occurrence evaluates");
    assert_eq!(result, shape(&[2, 3]));
    assert_eq!(bits, vec![0, 1, 2, 12, 13, 14]);

    // Perturbation one: a one-based reading of the first coordinate.
    assert_ne!(bits[..3], [3, 4, 5]);
    // Perturbation two: a clamping evaluator that treated `extent - 1` as past
    // the end and returned the previous row.
    assert_ne!(bits[3..], [9, 10, 11]);
}

/// A repeated index is admitted and the row is transported twice, bit for bit.
///
/// This is the many-to-one read the normative definition admits. The two copies
/// are asserted equal to each other *and* to the source row, because an evaluator
/// that produced two rows by two different routes could agree with the source at
/// one of them.
#[test]
fn a_repeated_index_reads_one_row_twice() {
    let source = positional(&[5, 3]);
    let (bits, result) = gather(&source, &index_tensor(&[3], &[3, 3, 3]), 0)
        .expect("a covered occurrence evaluates");
    assert_eq!(result, shape(&[3, 3]));
    assert_eq!(bits, vec![9, 10, 11, 9, 10, 11, 9, 10, 11]);
    assert_eq!(bits[..3], bits[3..6]);
    assert_eq!(bits[3..6], bits[6..]);
}

/// An out-of-range index refuses, and is neither clamped nor wrapped.
///
/// Three perturbations of the *subject*, each separately: one past the end, far
/// past the end, and `u32::MAX`. Each must refuse with the same named error
/// carrying its own value, and none may return a tensor. A clamping evaluator
/// would return row 4 for all three and a wrapping one would return rows 0, 1,
/// and 0 — so the refusal is what distinguishes the stated posture from both.
#[test]
fn an_out_of_range_index_refuses_and_is_neither_clamped_nor_wrapped() {
    let source = positional(&[5, 3]);
    for (value, position) in [(5_u32, 0_usize), (97, 0), (u32::MAX, 0)] {
        let error = gather(&source, &index_tensor(&[1], &[value]), 0)
            .expect_err("an out-of-range index is refused");
        assert_eq!(
            error,
            ReferenceOperationError::GatherIndexOutOfBounds {
                position,
                value: u64::from(value),
                extent: 5,
            },
            "{value} must refuse naming its own value"
        );
    }
}

/// The refusal names the offending element's position, not merely that one exists.
#[test]
fn an_out_of_range_refusal_names_the_offending_element_position() {
    let source = positional(&[5, 3]);
    let error = gather(&source, &index_tensor(&[4], &[1, 2, 9, 3]), 0)
        .expect_err("an out-of-range index is refused");
    assert_eq!(
        error,
        ReferenceOperationError::GatherIndexOutOfBounds {
            position: 2,
            value: 9,
            extent: 5,
        }
    );
}

/// An empty index operand evaluates and produces a result with no elements.
///
/// The prefill boundary can bind `T = 0`. It is checked as an *evaluation* rather
/// than only as a shape, because an evaluator that divided by the index count or
/// indexed a first element would fail here and nowhere else.
#[test]
fn an_empty_index_operand_yields_a_result_with_no_elements() {
    let source = positional(&[5, 3]);
    let (bits, result) =
        gather(&source, &index_tensor(&[0], &[]), 0).expect("an empty gather evaluates");
    assert_eq!(result, shape(&[0, 3]));
    assert!(bits.is_empty());
}

// --- The composition rule, at each axis position ------------------------------

/// The gathered axis in each of a rank-three source's three positions.
///
/// The interior and innermost cases are the ones a row-copying evaluator gets
/// wrong: only the outermost axis has contiguous rows under row-major, so
/// gathering on an interior axis reads a strided sequence of the source's
/// storage.
#[test]
fn the_result_composes_the_index_shape_at_each_axis_position() {
    let source = positional(&[2, 3, 4]);

    // Axis 0: whole `[3, 4]` slabs, slab 1 then slab 0.
    let (bits, result) = gather(&source, &index_tensor(&[2], &[1, 0]), 0).expect("axis 0 gathers");
    assert_eq!(result, shape(&[2, 3, 4]));
    assert_eq!(
        bits,
        vec![
            12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
        ]
    );

    // Axis 1: within each of the two slabs, rows 2 and 0 of three.
    let (bits, result) = gather(&source, &index_tensor(&[2], &[2, 0]), 1).expect("axis 1 gathers");
    assert_eq!(result, shape(&[2, 2, 4]));
    assert_eq!(
        bits,
        vec![8, 9, 10, 11, 0, 1, 2, 3, 20, 21, 22, 23, 12, 13, 14, 15]
    );

    // Axis 2: within each of the six rows, columns 3 and 1 of four. This is the
    // fully strided case — every gathered element is one element wide.
    let (bits, result) = gather(&source, &index_tensor(&[2], &[3, 1]), 2).expect("axis 2 gathers");
    assert_eq!(result, shape(&[2, 3, 2]));
    assert_eq!(bits, vec![3, 1, 7, 5, 11, 9, 15, 13, 19, 17, 23, 21]);
}

/// A rank-two index operand widens the result by one axis rather than flattening.
#[test]
fn a_rank_two_index_operand_composes_both_of_its_axes() {
    let source = positional(&[5, 2]);
    let (bits, result) = gather(&source, &index_tensor(&[2, 2], &[0, 4, 1, 3]), 0)
        .expect("a rank-two index gathers");
    assert_eq!(result, shape(&[2, 2, 2]));
    assert_eq!(bits, vec![0, 1, 8, 9, 2, 3, 6, 7]);
}

/// A rank-zero index operand drops the gathered axis.
#[test]
fn a_rank_zero_index_operand_drops_the_gathered_axis() {
    let source = positional(&[5, 3]);
    let (bits, result) =
        gather(&source, &index_tensor(&[], &[2]), 0).expect("a rank-zero index gathers");
    assert_eq!(result, shape(&[3]));
    assert_eq!(bits, vec![6, 7, 8]);
}

// --- Bit preservation ---------------------------------------------------------

/// Every exceptional payload crosses a gather unchanged.
///
/// A non-canonical NaN, a signalling NaN, a negative zero, and a subnormal are
/// each placed in the source and gathered. The family computes nothing, so each
/// must arrive with its exact bits — a canonicalizing evaluator would rewrite the
/// two NaNs, a flushing one the subnormal, and an evaluator that decoded and
/// re-encoded through host `f32` would lose the NaN payloads.
#[test]
fn every_exceptional_payload_crosses_the_gather_unchanged() {
    const QUIET_NAN_WITH_PAYLOAD: u32 = 0x7FC0_1234;
    const SIGNALLING_NAN: u32 = 0x7F80_0001;
    const NEGATIVE_ZERO: u32 = 0x8000_0000;
    const SMALLEST_SUBNORMAL: u32 = 0x0000_0001;
    let payloads = [
        QUIET_NAN_WITH_PAYLOAD,
        SIGNALLING_NAN,
        NEGATIVE_ZERO,
        SMALLEST_SUBNORMAL,
    ];
    let source = source_tensor(&[4], &payloads);

    // Gathered in reverse, so an evaluator that returned its operand unchanged
    // would also fail the order check rather than passing this test by accident.
    let (bits, result) =
        gather(&source, &index_tensor(&[4], &[3, 2, 1, 0]), 0).expect("the payloads gather");
    assert_eq!(result, shape(&[4]));
    assert_eq!(
        bits,
        vec![
            SMALLEST_SUBNORMAL,
            NEGATIVE_ZERO,
            SIGNALLING_NAN,
            QUIET_NAN_WITH_PAYLOAD
        ]
    );

    // Each rewrite a computing evaluator would perform, checked separately.
    assert_ne!(bits[3], 0x7FC0_0000, "the NaN payload is not canonicalized");
    assert_ne!(bits[2], 0x7FC0_0001, "the signalling NaN is not quieted");
    assert_ne!(bits[1], 0x0000_0000, "the zero keeps its sign");
    assert_ne!(bits[0], 0x0000_0000, "the subnormal is not flushed");
}
