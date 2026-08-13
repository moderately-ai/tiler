//! Bounded conformance evidence for `tiler::slice-f32@1`, through the public
//! semantic and reference boundary.
//!
//! # What this covers, exactly
//!
//! A window on the outermost axis, on an interior axis, and on the innermost
//! axis, at ranks one through four; a selection restricting several axes at once;
//! the extent-one result a single-position selection leaves behind and the
//! reindex that removes it; two selections composed; the exceptional payloads the
//! family must transport rather than compute; and the construction refusals a
//! caller reaches through the same facade. Every operand is `tiler::f32@1` and
//! every extent is static.
//!
//! **What it does not cover:** source-bearing offsets (those live in the crate
//! tests that construct a `ShapeEnv`), any strided selection, any rank above
//! four, any dtype but F32, any compiled or executed realization, and the pinned
//! model's own extents. The shapes here are small stand-ins chosen so the
//! expectations can be written out by hand. A pass here is evidence about the
//! semantic contract and the reference evaluator, not about a plan or a kernel.
//!
//! # Why the expectations are written out
//!
//! Each operand's payload at a coordinate is that coordinate's own row-major
//! linear index, so a result element names the operand position it came from and
//! an evaluator reading the wrong element produces a visibly wrong value rather
//! than a coincidence. Each expectation is then a literal list of those indices,
//! derived by hand from the selection rather than by a second implementation of
//! the addressing. A helper that recomputed the offsets would agree with the
//! evaluator for reasons that say nothing about either being right. The retained
//! perturbations each demonstrate the comparison failing.

use tiler_ir::semantic::{
    F32, F32Reindex, F32Slice, InputKey, OutputKey, ReindexForm, SemanticProgramBuilder,
    SliceAxisSelection, SliceSelection, Value,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

const WHOLE: SliceAxisSelection = SliceAxisSelection::WholeAxis;

fn window(offset: u64, extent: u64) -> SliceAxisSelection {
    SliceAxisSelection::static_window(offset, Extent::new(extent))
}

fn selection(axes: &[SliceAxisSelection]) -> SliceSelection {
    SliceSelection::new(axes.iter().cloned()).expect("a covered selection is admitted")
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

/// An operand whose payload at every coordinate is that coordinate's linear index.
fn positional(dims: &[u64]) -> (Vec<u64>, Vec<u32>) {
    let count = shape(dims)
        .element_count()
        .expect("a covered fixture is small");
    let bits = (0..count)
        .map(|position| u32::try_from(position).expect("a covered fixture is small"))
        .collect();
    (dims.to_vec(), bits)
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

/// Evaluates one selection over one operand.
fn select(operand: &(Vec<u64>, Vec<u32>), axes: &[SliceAxisSelection]) -> (Vec<u32>, Shape) {
    let (dims, bits) = operand;
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let key = InputKey::new("source").expect("a covered key is bounded");
    let input = builder
        .input::<F32>(key.clone(), shape(dims))
        .expect("an F32 input");
    let result = F32Slice::apply(&mut builder, &selection(axes), input)
        .expect("a covered occurrence is admitted");
    builder
        .output(OutputKey::new("result").expect("a valid key"), result)
        .expect("an output");
    let program = builder.build().expect("the program is complete");

    let source = tensor(dims, bits);
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[InputBinding::new(&key, &source)])
        .expect("a covered program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    (payload_bits(output), output.shape().clone())
}

// --- One window, at each position in the rank ---------------------------------

#[test]
fn a_rank_one_window_reads_a_run_of_coordinates_from_its_offset() {
    let (bits, result) = select(&positional(&[8]), &[window(2, 3)]);
    assert_eq!(result, shape(&[3]));
    assert_eq!(bits, vec![2, 3, 4]);

    // Perturbation: the same window read from zero, which is what an evaluator
    // that decoded the extent and dropped the offset would produce. It differs at
    // all three positions.
    let ignored_offset = vec![0, 1, 2];
    assert_eq!(
        bits.iter()
            .zip(&ignored_offset)
            .filter(|(left, right)| left != right)
            .count(),
        3
    );
}

#[test]
fn an_outermost_axis_window_reads_whole_rows() {
    // `[4, 3]`, rows one and two.
    let (bits, result) = select(&positional(&[4, 3]), &[window(1, 2), WHOLE]);
    assert_eq!(result, shape(&[2, 3]));
    assert_eq!(bits, vec![3, 4, 5, 6, 7, 8]);
}

/// The innermost-axis window is the case a block-copying evaluator gets wrong.
///
/// Under row-major only the slowest-varying axis has a contiguous window, so a
/// selection on the innermost axis reads a strided sequence of the operand's
/// storage. The perturbation is the contiguous prefix such an evaluator would
/// return.
#[test]
fn an_innermost_axis_window_reads_a_strided_sequence_of_the_operand() {
    // `[3, 4]`, columns one and two of every row.
    let (bits, result) = select(&positional(&[3, 4]), &[WHOLE, window(1, 2)]);
    assert_eq!(result, shape(&[3, 2]));
    assert_eq!(bits, vec![1, 2, 5, 6, 9, 10]);

    let contiguous_prefix = vec![0, 1, 2, 3, 4, 5];
    assert_ne!(bits, contiguous_prefix);
    assert_eq!(
        bits.iter()
            .zip(&contiguous_prefix)
            .filter(|(left, right)| left != right)
            .count(),
        6,
        "the two layouts disagree at every position, so the expectation is not vacuous"
    );
}

#[test]
fn an_interior_axis_window_reads_the_same_run_out_of_every_slab() {
    // `[2, 3, 2]`, the second and third rows of each of the two slabs.
    let (bits, result) = select(&positional(&[2, 3, 2]), &[WHOLE, window(1, 2), WHOLE]);
    assert_eq!(result, shape(&[2, 2, 2]));
    assert_eq!(bits, vec![2, 3, 4, 5, 8, 9, 10, 11]);
}

#[test]
fn a_selection_restricting_several_axes_reads_their_intersection() {
    // `[2, 2, 3, 2]` with axis 0 fixed at one, axis 2 cut to its first two
    // coordinates, and axis 3 fixed at one. Row-major strides are 12, 6, 2, 1, so
    // the four selected coordinates are 12 + 6·i1 + 2·i2 + 1.
    let (bits, result) = select(
        &positional(&[2, 2, 3, 2]),
        &[window(1, 1), WHOLE, window(0, 2), window(1, 1)],
    );
    assert_eq!(result, shape(&[1, 2, 2, 1]));
    assert_eq!(bits, vec![13, 15, 19, 21]);
}

/// An operand already empty on an unrestricted axis evaluates to no elements.
///
/// The empty-*window* rule refuses a selection that states emptiness; it says
/// nothing about an operand that has no elements, which is a shape the program
/// had before the selection was written. This is the evaluator half of that
/// separation: the walk issues no read and the result is a well-formed empty
/// tensor of the derived shape rather than a refusal or a panic.
#[test]
fn an_operand_empty_on_an_unrestricted_axis_evaluates_to_an_empty_result() {
    let (bits, result) = select(&(vec![0, 4], Vec::new()), &[WHOLE, window(1, 2)]);
    assert_eq!(result, shape(&[0, 2]));
    assert!(bits.is_empty());
}

// --- Rank preservation, and the reindex that undoes it ------------------------

/// A single-position selection leaves an extent-one axis, and a reindex removes it.
///
/// This is the composition the family's definition names: a selection restricts
/// coordinates and drops no axis, so the rank change a consumer usually wants is a
/// second occurrence rather than a second spelling of this one.
#[test]
fn a_single_position_selection_leaves_an_extent_one_axis_for_a_reindex_to_remove() {
    let (bits, result) = select(&positional(&[4, 3]), &[window(2, 1), WHOLE]);
    assert_eq!(result, shape(&[1, 3]));
    assert_eq!(bits, vec![6, 7, 8]);

    let (dims, source_bits) = positional(&[4, 3]);
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let key = InputKey::new("source").expect("a covered key is bounded");
    let input = builder
        .input::<F32>(key.clone(), shape(&dims))
        .expect("an F32 input");
    let row = F32Slice::apply(&mut builder, &selection(&[window(2, 1), WHOLE]), input)
        .expect("the selection is admitted");
    let squeezed = F32Reindex::apply(
        &mut builder,
        &ReindexForm::remove_unit_axis(Axis::new(0)).expect("the removal is well shaped"),
        row,
    )
    .expect("the extent-one axis the selection left is removable");
    builder
        .output(OutputKey::new("result").expect("a valid key"), squeezed)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    assert_eq!(program.operation_count(), 2);

    let source = tensor(&dims, &source_bits);
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[InputBinding::new(&key, &source)])
        .expect("the composition evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    assert_eq!(output.shape(), &shape(&[3]));
    assert_eq!(payload_bits(output), vec![6, 7, 8]);
}

/// Two selections compose into the selection of their composition.
///
/// The offsets add, which is the property that makes a chain of occurrences the
/// right spelling of a repeated restriction rather than a reason to admit a
/// second attribute form.
#[test]
fn two_selections_compose_by_adding_their_offsets() {
    let (dims, source_bits) = positional(&[8]);
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let key = InputKey::new("source").expect("a covered key is bounded");
    let input = builder
        .input::<F32>(key.clone(), shape(&dims))
        .expect("an F32 input");
    let outer: Value<F32> = F32Slice::apply(&mut builder, &selection(&[window(2, 5)]), input)
        .expect("the first selection is admitted");
    let inner = F32Slice::apply(&mut builder, &selection(&[window(1, 2)]), outer)
        .expect("the second selection is admitted");
    builder
        .output(OutputKey::new("result").expect("a valid key"), inner)
        .expect("an output");
    let program = builder.build().expect("the program is complete");

    let source = tensor(&dims, &source_bits);
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[InputBinding::new(&key, &source)])
        .expect("the composition evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    assert_eq!(output.shape(), &shape(&[2]));
    // Offsets 2 and 1 compose to 3, so the result is the operand's coordinates 3
    // and 4 — not 1 and 2, which is what reading the second offset against the
    // original operand would give.
    assert_eq!(payload_bits(output), vec![3, 4]);
    assert_eq!(
        select(&positional(&[8]), &[window(3, 2)]).0,
        payload_bits(output),
        "and it is exactly the single selection at the composed offset"
    );
}

// --- Exceptional payloads cross the selection unchanged -----------------------

/// The family transports bits and computes nothing, including the awkward ones.
///
/// A non-canonical quiet NaN, a signalling NaN, a negative zero, a subnormal, and
/// an infinity are selected and compared bit for bit. An evaluator that decoded
/// and re-encoded them — or that applied the crate's arithmetic NaN
/// canonicalization — would return the canonical NaN payload for the first two and
/// fail here.
#[test]
fn every_exceptional_payload_crosses_the_selection_unchanged() {
    let payloads = vec![
        0x3F80_0000, // 1.0, unselected
        0x7FC0_0001, // a non-canonical quiet NaN
        0xFF80_0001, // a signalling NaN with the sign bit set
        0x8000_0000, // negative zero
        0x0000_0001, // the smallest positive subnormal
        0xFF80_0000, // negative infinity
        0x4000_0000, // 2.0, unselected
    ];
    let (bits, result) = select(&(vec![7], payloads.clone()), &[window(1, 5)]);
    assert_eq!(result, shape(&[5]));
    assert_eq!(bits, payloads[1..6].to_vec());
    // The canonicalization an arithmetic family applies would replace both NaN
    // payloads; neither moved.
    assert_eq!(bits[0], 0x7FC0_0001);
    assert_eq!(bits[1], 0xFF80_0001);
}

// --- The refusals a caller reaches through the same facade ---------------------

/// Every construction refusal fires at the authoring boundary, before evaluation.
///
/// The reference evaluator is never reached by any of these, which is the point:
/// a selection that leaves its axis is a program that does not exist rather than a
/// program that returns a clamped tensor.
#[test]
fn a_refused_selection_never_reaches_the_evaluator() {
    let attempt = |axes: &[SliceAxisSelection], dims: &[u64]| -> String {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the standard builder opens");
        let input = builder
            .input::<F32>(
                InputKey::new("source").expect("a covered key is bounded"),
                shape(dims),
            )
            .expect("an F32 input");
        let error = F32Slice::apply(&mut builder, &selection(axes), input)
            .expect_err("the occurrence is refused");
        error.to_string()
    };

    // A window that leaves its axis. The clamping convention would have returned a
    // `[3, 3]` result here.
    assert!(
        attempt(&[window(2, 5), WHOLE], &[4, 3]).contains("leaves the operand's declared extent"),
        "an out-of-bounds selection is refused rather than clamped"
    );
    // A selection written against the wrong rank.
    assert!(
        attempt(&[window(1, 2)], &[4, 3]).contains("one entry per operand axis"),
        "a selection states one entry per operand axis"
    );
    // A window covering its axis.
    assert!(
        attempt(&[window(0, 4), window(0, 2)], &[4, 3]).contains("whole-axis"),
        "a window covering its axis is the whole-axis relation"
    );

    // And the two rules that refuse before an operand is consulted at all, so no
    // program can carry them.
    assert!(SliceSelection::new([WHOLE, WHOLE]).is_err());
    assert!(SliceSelection::new([window(0, 0), WHOLE]).is_err());
}
