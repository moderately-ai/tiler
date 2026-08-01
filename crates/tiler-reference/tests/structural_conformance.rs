//! Bounded conformance evidence for `tiler::reindex-f32@1` and
//! `tiler::broadcast-f32@1`, through the public semantic and reference boundary.
//!
//! # What this covers, exactly
//!
//! **Reindex**, all six admitted forms: `permute-axes` at ranks two and four,
//! `split-axis` at ranks one and two, `merge-axes` at ranks two and three,
//! `insert-unit-axis` and `remove-unit-axis` at ranks one and two, and
//! `reverse-axis` at ranks one and three. **Broadcast**, all three admitted
//! relations: `from-operand`, `replicate` leading, interior, and repeated, and
//! `stretch-unit`. Every operand is `tiler::f32@1` and every extent is static.
//!
//! **What it does not cover:** any symbolic extent, any rank above four, any
//! dtype but F32, any compiled or executed realization, and any composition of
//! more than three occurrences. A pass here is evidence about the semantic
//! contract and the reference evaluator, not about a plan or a kernel.
//!
//! # Why the expectations are written out
//!
//! Each expectation is a literal permutation of the operand's elements, derived
//! by hand from the mapping's definition rather than by a second implementation
//! of the mapping. A helper that recomputed the coordinate map would agree with
//! the evaluator for reasons that say nothing about either being right, which is
//! the same independence rule the two oracles in this crate are built on.
//!
//! The fixtures are ascending distinct integers, so a coordinate map that reads
//! the wrong element produces the wrong value rather than a coincidence, and the
//! retained perturbations below each demonstrate the comparison failing.

use tiler_ir::semantic::{
    BroadcastAxisMapping, BroadcastAxisSource, BuildError, F32, F32Broadcast, F32Reindex, InputKey,
    OutputKey, RegistryError, ReindexForm, SemanticProgramBuilder, Value,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: u64) -> Extent {
    Extent::new(value)
}

fn replicate() -> BroadcastAxisSource {
    BroadcastAxisSource::Replicate
}

fn from_operand(value: u32) -> BroadcastAxisSource {
    BroadcastAxisSource::FromOperand(axis(value))
}

fn stretch_unit(value: u32) -> BroadcastAxisSource {
    BroadcastAxisSource::StretchUnit(axis(value))
}

fn mapping(result: &[u64], sources: &[BroadcastAxisSource]) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        result.iter().copied().map(Extent::new),
        sources.iter().copied(),
    )
    .expect("a covered mapping is admitted")
}

/// Builds a tensor of ascending distinct integer payloads.
fn ascending(shape: &Shape) -> Tensor {
    let count = shape.element_count().expect("a covered shape is bounded");
    let elements = (0..count)
        .map(|value| {
            ReferenceElement::from_float_bits(
                u32::try_from(value)
                    .expect("a covered operand is small")
                    .to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .expect("an ascending payload is four bytes")
        })
        .collect();
    Tensor::dense(F32::resolved_type(), shape.clone(), elements).expect("the tensor is well formed")
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

/// Evaluates a one-input program built by `build` over ascending payloads.
fn evaluate(
    input_shape: &[u64],
    build: impl FnOnce(&mut SemanticProgramBuilder, Value<F32>) -> Value<F32>,
) -> (Vec<u32>, Shape) {
    let shape = Shape::try_from_dims(input_shape.iter().copied()).expect("a covered shape");
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let input = builder
        .input::<F32>(
            InputKey::new("operand").expect("a valid key"),
            shape.clone(),
        )
        .expect("an F32 input");
    let result = build(&mut builder, input);
    builder
        .output(OutputKey::new("result").expect("a valid key"), result)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let tensor = ascending(&shape);
    let key = InputKey::new("operand").expect("a valid key");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .expect("a covered program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    (payload_bits(output), output.shape().clone())
}

fn reindex(input_shape: &[u64], form: &ReindexForm) -> (Vec<u32>, Shape) {
    evaluate(input_shape, |builder, input| {
        F32Reindex::apply(builder, form, input).expect("a covered form is admitted")
    })
}

fn broadcast(input_shape: &[u64], mapping: &BroadcastAxisMapping) -> (Vec<u32>, Shape) {
    evaluate(input_shape, |builder, input| {
        F32Broadcast::apply(builder, mapping, input).expect("a covered mapping is admitted")
    })
}

// --- permute-axes -----------------------------------------------------------

#[test]
fn permute_axes_matches_a_materialized_transpose_at_ranks_two_and_four() {
    // `[2, 3]` transposed. Row-major 0..6 is read column-major.
    assert_eq!(
        reindex(
            &[2, 3],
            &ReindexForm::permute_axes([axis(1), axis(0)]).unwrap()
        ),
        (vec![0, 3, 1, 4, 2, 5], Shape::from_dims([3, 2]))
    );

    // The grouped-query head-layout permutation `[T, g, r, d] -> [g, r, T, d]`,
    // at `T = 2, g = 2, r = 2, d = 2`. Element `(t, g, r, d)` of the operand sits
    // at `8t + 4g + 2r + d`; result position `(g, r, t, d)` therefore reads
    // `8t + 4g + 2r + d` for result index `8g + 4r + 2t + d`.
    let (values, shape) = reindex(
        &[2, 2, 2, 2],
        &ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(3)]).unwrap(),
    );
    assert_eq!(shape, Shape::from_dims([2, 2, 2, 2]));
    assert_eq!(
        values,
        vec![0, 1, 8, 9, 2, 3, 10, 11, 4, 5, 12, 13, 6, 7, 14, 15]
    );
}

// --- split-axis -------------------------------------------------------------

#[test]
fn split_axis_is_row_major_with_the_major_factor_first() {
    // A reshape moves no element in row-major order, so the payload is unchanged
    // and the *shape* carries the whole result. That is the property, not a weak
    // assertion: a split that reversed its factors would still leave the payload
    // untouched and would produce `[2, 3]` where `[3, 2]` is required.
    assert_eq!(
        reindex(
            &[6],
            &ReindexForm::split_axis(axis(0), [extent(3), extent(2)]).unwrap()
        ),
        ((0..6).collect::<Vec<_>>(), Shape::from_dims([3, 2]))
    );

    // The head split at rank two: `[T, 2048] -> [T, 16, 128]`, scaled to
    // `[2, 6] -> [2, 3, 2]`.
    assert_eq!(
        reindex(
            &[2, 6],
            &ReindexForm::split_axis(axis(1), [extent(3), extent(2)]).unwrap()
        ),
        ((0..12).collect::<Vec<_>>(), Shape::from_dims([2, 3, 2]))
    );

    // The direction is load-bearing, and this is where it is visible. The
    // grouped-query split of a head axis into `(groups, repeats)` makes the group
    // index the *major* coordinate: composing the split with a permutation that
    // reads the group axis reproduces `h / repeats`, and the alternative reading
    // `h % groups` is a different tensor. Splitting `[4]` into `(2, 2)` and
    // transposing gives `[0, 2, 1, 3]` — head `h` at position `2r + g` reading
    // `h = 2g + r` — while the tile reading would give `[0, 1, 2, 3]`.
    let split = ReindexForm::split_axis(axis(0), [extent(2), extent(2)]).unwrap();
    let permute = ReindexForm::permute_axes([axis(1), axis(0)]).unwrap();
    let (values, shape) = evaluate(&[4], |builder, input| {
        let split = F32Reindex::apply(builder, &split, input).expect("4 = 2 x 2");
        F32Reindex::apply(builder, &permute, split).expect("a transpose")
    });
    assert_eq!(shape, Shape::from_dims([2, 2]));
    assert_eq!(
        values,
        vec![0, 2, 1, 3],
        "a row-major split makes the first factor the major axis, so the group \
         index is h / repeats and not h % groups"
    );
}

// --- merge-axes -------------------------------------------------------------

#[test]
fn merge_axes_inverts_its_split_at_ranks_two_and_three() {
    assert_eq!(
        reindex(
            &[3, 2],
            &ReindexForm::merge_axes([axis(0), axis(1)]).unwrap()
        ),
        ((0..6).collect::<Vec<_>>(), Shape::from_dims([6]))
    );
    // An inner run, which a stride order computed outermost-first would get
    // wrong while the rank-two case still passed.
    assert_eq!(
        reindex(
            &[2, 2, 3],
            &ReindexForm::merge_axes([axis(1), axis(2)]).unwrap()
        ),
        ((0..12).collect::<Vec<_>>(), Shape::from_dims([2, 6]))
    );
    // Split then merge is the identity on both the payload and the shape, which
    // is the attention-output round trip `[T, 16, 128] -> [T, 2048]`.
    let split = ReindexForm::split_axis(axis(1), [extent(3), extent(2)]).unwrap();
    let merge = ReindexForm::merge_axes([axis(1), axis(2)]).unwrap();
    let (values, shape) = evaluate(&[2, 6], |builder, input| {
        let split = F32Reindex::apply(builder, &split, input).expect("6 = 3 x 2");
        F32Reindex::apply(builder, &merge, split).expect("an adjacent pair merges")
    });
    assert_eq!(shape, Shape::from_dims([2, 6]));
    assert_eq!(values, (0..12).collect::<Vec<_>>());
}

// --- unit-axis insertion and removal ----------------------------------------

#[test]
fn unit_axis_insertion_and_removal_move_no_element() {
    assert_eq!(
        reindex(&[3], &ReindexForm::insert_unit_axis(axis(0)).unwrap()),
        (vec![0, 1, 2], Shape::from_dims([1, 3]))
    );
    assert_eq!(
        reindex(&[3], &ReindexForm::insert_unit_axis(axis(1)).unwrap()),
        (vec![0, 1, 2], Shape::from_dims([3, 1]))
    );
    assert_eq!(
        reindex(&[3, 1], &ReindexForm::remove_unit_axis(axis(1)).unwrap()),
        (vec![0, 1, 2], Shape::from_dims([3]))
    );
    assert_eq!(
        reindex(&[1, 3], &ReindexForm::remove_unit_axis(axis(0)).unwrap()),
        (vec![0, 1, 2], Shape::from_dims([3]))
    );
}

// --- reverse-axis, the form decision D-10 admits ----------------------------

#[test]
fn reverse_axis_reproduces_the_rotary_coordinate_swap() {
    // At rank one the reversal is visible whole.
    assert_eq!(
        reindex(&[4], &ReindexForm::reverse_axis(axis(0)).unwrap()),
        (vec![3, 2, 1, 0], Shape::from_dims([4]))
    );

    // At the shape `rotate_half` uses: a size-two axis inside `[…, 2, 64]`,
    // scaled to `[2, 2, 3]`. Reversing axis 1 swaps the two three-element halves
    // within each outer block and moves nothing across blocks.
    assert_eq!(
        reindex(&[2, 2, 3], &ReindexForm::reverse_axis(axis(1)).unwrap()),
        (
            vec![3, 4, 5, 0, 1, 2, 9, 10, 11, 6, 7, 8],
            Shape::from_dims([2, 2, 3])
        )
    );
}

/// The `rotate_half` structure, end to end over admitted forms.
///
/// Split the innermost axis in half, reverse the size-two axis, merge back. The
/// sign multiply the full composition needs is a separate family and is not part
/// of this ticket's evidence; the *structural* half is, and it is what D-10
/// decides.
#[test]
fn the_rotate_half_structure_composes_over_admitted_forms_alone() {
    let split = ReindexForm::split_axis(axis(1), [extent(2), extent(3)]).unwrap();
    let swap = ReindexForm::reverse_axis(axis(1)).unwrap();
    let merge = ReindexForm::merge_axes([axis(1), axis(2)]).unwrap();
    let (values, shape) = evaluate(&[2, 6], |builder, input| {
        let split = F32Reindex::apply(builder, &split, input).expect("6 = 2 x 3");
        let swapped = F32Reindex::apply(builder, &swap, split).expect("the D-10 form");
        F32Reindex::apply(builder, &merge, swapped).expect("an adjacent pair merges")
    });
    assert_eq!(shape, Shape::from_dims([2, 6]));
    assert_eq!(
        values,
        vec![3, 4, 5, 0, 1, 2, 9, 10, 11, 6, 7, 8],
        "each row's two halves exchange places, which is the structure of \
         cat(-x2, x1) with the negation left to the sign multiply"
    );

    // The perturbation. With the swap removed the composition is a split
    // immediately merged back, which returns its operand — so the comparison
    // above discriminates the swap rather than passing for free.
    let (unswapped, _) = evaluate(&[2, 6], |builder, input| {
        let split = F32Reindex::apply(builder, &split, input).expect("6 = 2 x 3");
        F32Reindex::apply(builder, &merge, split).expect("an adjacent pair merges")
    });
    assert_eq!(unswapped, (0..12).collect::<Vec<_>>());
    assert_ne!(unswapped, values);
}

// --- broadcast --------------------------------------------------------------

#[test]
fn a_rank_pad_replicates_the_operand_over_every_added_axis() {
    // The RMS-normalization weight, `[1024]` against `[T, 1024]`, scaled.
    assert_eq!(
        broadcast(&[3], &mapping(&[2, 3], &[replicate(), from_operand(0)])),
        (vec![0, 1, 2, 0, 1, 2], Shape::from_dims([2, 3]))
    );
    // The per-head weight, `[128]` against `[T, 16, 128]`: two leading pads.
    assert_eq!(
        broadcast(
            &[2],
            &mapping(&[2, 2, 2], &[replicate(), replicate(), from_operand(0)])
        ),
        (vec![0, 1, 0, 1, 0, 1, 0, 1], Shape::from_dims([2, 2, 2]))
    );
    // The rotary tables, `[T, 128]` against `[T, 16, 128]`: an *interior* pad,
    // which a leading-pad-only implementation would get wrong.
    assert_eq!(
        broadcast(
            &[2, 3],
            &mapping(&[2, 2, 3], &[from_operand(0), replicate(), from_operand(1)])
        ),
        (
            vec![0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5],
            Shape::from_dims([2, 2, 3])
        )
    );
    // The causal mask, `[T, S]` against `[g, r, T, S]`, scaled.
    let (values, shape) = broadcast(
        &[2, 2],
        &mapping(
            &[2, 2, 2, 2],
            &[replicate(), replicate(), from_operand(0), from_operand(1)],
        ),
    );
    assert_eq!(shape, Shape::from_dims([2, 2, 2, 2]));
    assert_eq!(values, [0, 1, 2, 3].repeat(4));
}

#[test]
fn a_unit_stretch_widens_one_operand_axis_in_place() {
    // The rotary sign operand, `[2, 1]` against `[…, 2, 64]`, scaled to
    // `[2, 1] -> [2, 3]`. Each operand element is repeated along the widened
    // axis rather than across it, which is the difference from a rank pad.
    assert_eq!(
        broadcast(
            &[2, 1],
            &mapping(&[2, 3], &[from_operand(0), stretch_unit(1)])
        ),
        (vec![0, 0, 0, 1, 1, 1], Shape::from_dims([2, 3]))
    );
    // The same result shape reached by a rank pad from a *different* operand
    // produces a different tensor, which is what makes the two relations
    // separately load-bearing rather than two spellings of one idea.
    assert_eq!(
        broadcast(&[3], &mapping(&[2, 3], &[replicate(), from_operand(0)])),
        (vec![0, 1, 2, 0, 1, 2], Shape::from_dims([2, 3]))
    );
    // A stretch at the full workload rank, with the two leading pads present.
    assert_eq!(
        broadcast(
            &[2, 1],
            &mapping(&[2, 2, 3], &[replicate(), from_operand(0), stretch_unit(1)])
        ),
        (
            vec![0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1],
            Shape::from_dims([2, 2, 3])
        )
    );
}

// --- the negative cases -----------------------------------------------------

/// Returns the provider diagnostic code a refused application carried.
///
/// Asserting the code rather than only `is_err` is what makes each refusal below
/// evidence about the rule it names: a poisoned builder, a foreign handle, or a
/// bound violation would all be errors too, and none of them would be the check.
fn refusal_code(error: &BuildError) -> String {
    let BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(rejection)) =
        error
    else {
        panic!("a structural refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

#[test]
fn the_four_required_refusals_fire_at_construction() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let operand = builder
        .input::<F32>(
            InputKey::new("operand").expect("a valid key"),
            Shape::from_dims([2, 6]),
        )
        .expect("an F32 input");
    let rank_one = builder
        .input::<F32>(
            InputKey::new("weight").expect("a valid key"),
            Shape::from_dims([3]),
        )
        .expect("an F32 input");
    let unit = builder
        .input::<F32>(
            InputKey::new("sign").expect("a valid key"),
            Shape::from_dims([2, 1]),
        )
        .expect("an F32 input");

    // A non-total mapping: the split's factors read past the end of the axis.
    let not_total = ReindexForm::split_axis(axis(1), [extent(2), extent(5)]).unwrap();
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &not_total, operand).unwrap_err()),
        "reindex.split.not-total",
        "2 x 5 exceeds a six-wide axis, so the mapping is not total"
    );

    // A non-bijective reindex: the split's factors read a prefix, which is a
    // slice rather than a reindex.
    let not_bijective = ReindexForm::split_axis(axis(1), [extent(2), extent(2)]).unwrap();
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &not_bijective, operand).unwrap_err()),
        "reindex.split.not-surjective",
        "2 x 2 falls short of a six-wide axis, so the mapping is a slice"
    );

    // An implicit rank pad: the mapping accounts for the operand's axes rather
    // than the result's, so a result axis has no source. Refused before an
    // occurrence exists, because it is a property of the mapping alone.
    assert!(
        BroadcastAxisMapping::new([extent(2), extent(3)], [from_operand(0)]).is_err(),
        "a mapping must account for every result axis"
    );
    // And the same shortfall reaching an occurrence, through a mapping that is
    // well formed for a rank-one operand and applied to a rank-two one.
    let dropped = mapping(&[2, 3], &[replicate(), from_operand(0)]);
    assert_eq!(
        refusal_code(&F32Broadcast::apply(&mut builder, &dropped, unit).unwrap_err()),
        "broadcast.mapping.operand-axes-unconsumed",
        "a `[2, 1]` operand has an axis this mapping never consumes"
    );

    // An extent-one stretch presented without an axis mapping stating it.
    let unstated = mapping(&[2, 2, 3], &[replicate(), from_operand(0), from_operand(1)]);
    assert_eq!(
        refusal_code(&F32Broadcast::apply(&mut builder, &unstated, unit).unwrap_err()),
        "broadcast.mapping.extent-disagreement",
        "the `[2, 1]` operand's second axis is widened, and only a stretch-unit \
         relation may say so"
    );

    // The admitted neighbours of each refusal, so the checks above are known to
    // be discriminating rather than uniformly refusing.
    let admitted = ReindexForm::split_axis(axis(1), [extent(2), extent(3)]).unwrap();
    assert!(F32Reindex::apply(&mut builder, &admitted, operand).is_ok());
    assert!(F32Broadcast::apply(&mut builder, &dropped, rank_one).is_ok());
    let stated = mapping(&[2, 2, 3], &[replicate(), from_operand(0), stretch_unit(1)]);
    assert!(F32Broadcast::apply(&mut builder, &stated, unit).is_ok());
}
