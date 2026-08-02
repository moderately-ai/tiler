//! Bounded conformance evidence for `tiler::concatenate-f32@1`, through the
//! public semantic and reference boundary.
//!
//! # What this covers, exactly
//!
//! A join on the outermost axis, on an interior axis, and on the innermost axis,
//! at ranks one through three; every admitted operand arity from two through
//! eight; the zero-extent operand the pinned prefill binds; the exceptional
//! payloads the family must transport rather than compute; and the operand
//! ordering that makes two occurrences over the same operands different
//! computations. Every operand is `tiler::f32@1` and every extent is static.
//!
//! **What it does not cover:** any symbolic extent, any rank above three, any
//! dtype but F32, any compiled or executed realization, and the pinned model's
//! own `[8, S, 128]` extents — the shapes here are small stand-ins chosen so the
//! expectations can be written out by hand. A pass here is evidence about the
//! semantic contract and the reference evaluator, not about a plan or a kernel.
//!
//! # Why the expectations are written out
//!
//! Each expectation is a literal sequence of operand payloads, derived by hand
//! from the definition's ordering rule rather than by a second implementation of
//! it. A helper that recomputed the block layout would agree with the evaluator
//! for reasons that say nothing about either being right.
//!
//! Each operand's payloads occupy their own numeric decade, so a result element
//! names the operand it came from as well as its position: an evaluator that read
//! the wrong operand produces a visibly wrong value rather than a coincidence.
//! The retained perturbations below each demonstrate the comparison failing.

use tiler_ir::semantic::{
    F32, F32Concatenate, InputKey, MAX_CONCATENATE_OPERANDS, MIN_CONCATENATE_OPERANDS, OutputKey,
    SemanticProgramBuilder, Value,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn shape(dims: &[u64]) -> Shape {
    Shape::try_from_dims(dims.iter().copied()).expect("a covered shape is bounded")
}

fn element(bits: u32) -> ReferenceElement {
    ReferenceElement::from_float_bits(bits.to_be_bytes(), FloatBitOrder::MostSignificantByteFirst)
        .expect("a covered payload is four bytes")
}

fn tensor(dims: &[u64], bits: &[u32]) -> Tensor {
    let shape = shape(dims);
    assert_eq!(
        shape.element_count(),
        Some(bits.len()),
        "a fixture states exactly one payload per coordinate"
    );
    Tensor::dense(
        F32::resolved_type(),
        shape,
        bits.iter().copied().map(element).collect(),
    )
    .expect("the fixture tensor is well formed")
}

/// Payloads for one operand, in its own decade: operand `k` holds `10k, 10k+1, …`.
fn decade(operand: usize, count: usize) -> Vec<u32> {
    let base = u32::try_from(operand).expect("a covered arity is small") * 10;
    (0..count)
        .map(|position| base + u32::try_from(position).expect("a covered operand is small"))
        .collect()
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

/// Evaluates one concatenation over the supplied operands.
fn join(operands: &[(&[u64], Vec<u32>)], concat_axis: Axis) -> (Vec<u32>, Shape) {
    let keys: Vec<InputKey> = (0..operands.len())
        .map(|position| {
            InputKey::new(format!("operand{position}")).expect("a covered key is bounded")
        })
        .collect();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let values: Vec<Value<F32>> = keys
        .iter()
        .zip(operands)
        .map(|(key, (dims, _))| {
            builder
                .input::<F32>(key.clone(), shape(dims))
                .expect("an F32 input")
        })
        .collect();
    let result = F32Concatenate::apply(&mut builder, &values, concat_axis)
        .expect("a covered occurrence is admitted");
    builder
        .output(OutputKey::new("result").expect("a valid key"), result)
        .expect("an output");
    let program = builder.build().expect("the program is complete");

    let tensors: Vec<Tensor> = operands
        .iter()
        .map(|(dims, bits)| tensor(dims, bits))
        .collect();
    let bindings: Vec<InputBinding<'_>> = keys
        .iter()
        .zip(&tensors)
        .map(|(key, tensor)| InputBinding::new(key, tensor))
        .collect();
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &bindings)
        .expect("a covered program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    (payload_bits(output), output.shape().clone())
}

// --- The outermost axis: whole operands end to end --------------------------

#[test]
fn an_outermost_axis_join_lays_the_operands_end_to_end() {
    // `[2, 3]` and `[1, 3]` on axis 0. Under row-major the concatenated axis is
    // slowest-varying, so the result is operand 0's six payloads followed by
    // operand 1's three, with nothing interleaved.
    let (bits, result) = join(&[(&[2, 3], decade(0, 6)), (&[1, 3], decade(1, 3))], axis(0));
    assert_eq!(result, shape(&[3, 3]));
    assert_eq!(bits, vec![0, 1, 2, 3, 4, 5, 10, 11, 12]);
}

#[test]
fn a_rank_one_join_is_the_operand_sequences_in_order() {
    let (bits, result) = join(&[(&[3], decade(0, 3)), (&[2], decade(1, 2))], axis(0));
    assert_eq!(result, shape(&[5]));
    assert_eq!(bits, vec![0, 1, 2, 10, 11]);
}

// --- An interior and an innermost axis: slab by slab ------------------------

/// The join on an inner axis interleaves the operands, one slab at a time.
///
/// This is the case a naive "append operand 1's buffer to operand 0's" evaluator
/// gets wrong, and it is the case the KV cache actually needs: `[8, S, 128]`
/// extends on axis 1, so each of the eight head slabs takes its own contribution
/// from each operand. The perturbation at the end is that naive evaluator's
/// answer, retained so the expectation above is visibly not vacuous.
#[test]
fn an_interior_axis_join_interleaves_the_operands_slab_by_slab() {
    // `[2, 2, 2]` and `[2, 1, 2]` on axis 1, giving `[2, 3, 2]`.
    //
    // Operand 0 is `[[[0,1],[2,3]], [[4,5],[6,7]]]` and operand 1 is
    // `[[[10,11]], [[12,13]]]`. Slab 0 of the result is operand 0's first two
    // rows then operand 1's one row; slab 1 is the same for their second halves.
    let (bits, result) = join(
        &[(&[2, 2, 2], decade(0, 8)), (&[2, 1, 2], decade(1, 4))],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 3, 2]));
    assert_eq!(bits, vec![0, 1, 2, 3, 10, 11, 4, 5, 6, 7, 12, 13]);

    // Perturbation: the end-to-end layout, which is what an evaluator that
    // concatenated the flat buffers would produce. It differs at six of twelve
    // positions, so the expectation above is a real constraint on the ordering.
    // The four that agree are the operands' first and last slabs, which land at
    // the same offsets under both layouts — which is exactly why a fixture whose
    // payloads did not name their operand could pass while being wrong.
    let end_to_end = vec![0, 1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 13];
    assert_ne!(bits, end_to_end);
    assert_eq!(
        bits.iter()
            .zip(&end_to_end)
            .filter(|(left, right)| left != right)
            .count(),
        6
    );
}

#[test]
fn an_innermost_axis_join_interleaves_every_row() {
    // `[2, 2]` and `[2, 1]` on axis 1, giving `[2, 3]`.
    let (bits, result) = join(&[(&[2, 2], decade(0, 4)), (&[2, 1], decade(1, 2))], axis(1));
    assert_eq!(result, shape(&[2, 3]));
    assert_eq!(bits, vec![0, 1, 10, 2, 3, 11]);
}

// --- Operand order is semantic ----------------------------------------------

/// Two occurrences over the same operands, differing only in order, differ.
///
/// Both have the same result *shape*, so nothing in the shape path distinguishes
/// them; the reference evaluator is what makes the ordering rule checkable.
#[test]
fn operand_order_is_semantic_and_the_reversed_join_is_a_different_value() {
    let forward = join(&[(&[2], decade(0, 2)), (&[2], decade(1, 2))], axis(0));
    let reversed = join(&[(&[2], decade(1, 2)), (&[2], decade(0, 2))], axis(0));
    assert_eq!(forward.1, reversed.1, "the two occurrences agree on shape");
    assert_eq!(forward.0, vec![0, 1, 10, 11]);
    assert_eq!(reversed.0, vec![10, 11, 0, 1]);
    assert_ne!(forward.0, reversed.0, "and disagree on every element");
}

// --- The zero-extent operand the pinned prefill binds -----------------------

/// Prefill's `C = 0`, evaluated: the result is the new rows, bit for bit.
///
/// L5 makes prefill an occurrence of the same program with an empty cache bound,
/// so this is a row the pinned workload reaches on every sequence rather than a
/// degenerate case admitted for tidiness. The comparison is against the second
/// operand's own payloads, so an evaluator that dropped, duplicated, or reordered
/// anything would differ.
#[test]
fn a_zero_extent_operand_contributes_nothing_and_the_rest_arrives_unchanged() {
    let new_rows = decade(1, 12);
    // `[2, 0, 2]` joined with `[2, 3, 2]` on axis 1 is exactly `[2, 3, 2]`.
    let (bits, result) = join(
        &[(&[2, 0, 2], Vec::new()), (&[2, 3, 2], new_rows.clone())],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 3, 2]));
    assert_eq!(bits, new_rows);

    // The empty operand in the second position is equally inert, and the empty
    // operand between two others neither shifts nor reorders their contributions.
    let (bits, result) = join(
        &[(&[2, 3, 2], new_rows.clone()), (&[2, 0, 2], Vec::new())],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 3, 2]));
    assert_eq!(bits, new_rows);

    let (bits, result) = join(
        &[
            (&[2, 1, 2], decade(0, 4)),
            (&[2, 0, 2], Vec::new()),
            (&[2, 1, 2], decade(2, 4)),
        ],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 2, 2]));
    assert_eq!(bits, vec![0, 1, 20, 21, 2, 3, 22, 23]);
}

#[test]
fn every_operand_empty_makes_an_empty_result_rather_than_a_refusal() {
    let (bits, result) = join(
        &[(&[2, 0, 2], Vec::new()), (&[2, 0, 2], Vec::new())],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 0, 2]));
    assert!(bits.is_empty());
}

// --- Bit preservation --------------------------------------------------------

/// The family transports exceptional payloads rather than computing over them.
///
/// The crate-wide arithmetic NaN canonicalization exists for operations that
/// *produce* a result. Applying it here would rewrite a non-canonical NaN a
/// program only moved, which is the one thing a structural family must never do —
/// so every payload below must arrive at exactly the bits it left with.
#[test]
fn exceptional_payloads_cross_the_join_unchanged() {
    let exceptional = vec![
        0x7FC0_0001, // a non-canonical quiet NaN
        0x7F80_0001, // a signalling NaN
        0xFFC0_0000, // a negative quiet NaN
        0x8000_0000, // negative zero
    ];
    let more = vec![
        0x0000_0001, // the smallest positive subnormal
        0x8000_0001, // its negation
        0xFF80_0000, // negative infinity
        0x7F80_0000, // positive infinity
    ];
    let (bits, result) = join(
        &[(&[4], exceptional.clone()), (&[4], more.clone())],
        axis(0),
    );
    assert_eq!(result, shape(&[8]));
    let mut expected = exceptional;
    expected.extend(&more);
    assert_eq!(
        bits, expected,
        "every payload is cloned byte for byte; nothing is canonicalized, \
         quieted, flushed, or re-encoded"
    );
    // Negative zero survives as negative zero rather than collapsing to +0.0,
    // which a decode-and-re-encode evaluator comparing values would lose.
    assert_eq!(bits[3], 0x8000_0000);
    assert_ne!(bits[3], 0x0000_0000);
}

// --- The admitted arity range ------------------------------------------------

/// Every arity the schema admits has a reference capability behind it.
///
/// The reference registry keys a capability on an *exact* resolved signature, so
/// an arity the semantic schema admitted and the provider never registered would
/// verify and then fail to evaluate. This walks the whole declared range rather
/// than a sample, so widening the family without widening the provider fails
/// here rather than at a consumer.
#[test]
fn every_admitted_arity_evaluates_through_a_registered_capability() {
    for arity in MIN_CONCATENATE_OPERANDS..=MAX_CONCATENATE_OPERANDS {
        let operands: Vec<(&[u64], Vec<u32>)> = (0..arity)
            .map(|position| {
                (
                    &[2_u64][..],
                    decade(
                        usize::try_from(position).expect("a covered arity is small"),
                        2,
                    ),
                )
            })
            .collect();
        let (bits, result) = join(&operands, axis(0));
        assert_eq!(result, shape(&[2 * u64::from(arity)]));
        let expected: Vec<u32> = (0..arity)
            .flat_map(|position| {
                decade(
                    usize::try_from(position).expect("a covered arity is small"),
                    2,
                )
            })
            .collect();
        assert_eq!(bits, expected, "at arity {arity}");
    }
}

// --- The decode step, end to end ---------------------------------------------

/// One decode step's KV append, at a hand-checkable stand-in for `[8, S, 128]`.
///
/// Two heads and a width of two rather than eight and 128, so the expectation is
/// written out rather than generated. The structure is the pinned one: the cached
/// context is `[H, C, W]`, the step contributes `[H, 1, W]`, and the join is on
/// axis 1.
#[test]
fn the_decode_step_append_matches_a_hand_written_expectation() {
    // cache `[2, 2, 2]`: head 0 holds positions (0,1) and (2,3); head 1 holds
    // (4,5) and (6,7). The step's new row is (10,11) for head 0 and (12,13) for
    // head 1, so each head gains exactly its own row at the end of its own slab.
    let (bits, result) = join(
        &[(&[2, 2, 2], decade(0, 8)), (&[2, 1, 2], decade(1, 4))],
        axis(1),
    );
    assert_eq!(result, shape(&[2, 3, 2]));
    assert_eq!(
        bits,
        vec![
            0, 1, 2, 3, 10, 11, // head 0: two cached positions, then the new one
            4, 5, 6, 7, 12, 13, // head 1: the same
        ]
    );
    // Perturbation: giving head 1 the row that belongs to head 0 — the mistake a
    // single shared offset into the new-rows buffer would make — differs at the
    // two positions that carry head 1's contribution.
    let crossed = vec![0, 1, 2, 3, 10, 11, 4, 5, 6, 7, 10, 11];
    assert_ne!(bits, crossed);
    assert_eq!(
        bits.iter()
            .zip(&crossed)
            .filter(|(left, right)| left != right)
            .count(),
        2
    );
}
