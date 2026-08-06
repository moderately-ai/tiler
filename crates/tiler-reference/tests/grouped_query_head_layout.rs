//! The grouped-query head-layout profile: four checked `Reindex` coordinate maps
//! over the already-admitted forms, at the C1 prefill shapes.
//!
//! # What this profile is
//!
//! The pinned checkpoint projects a `[T, 2048]` query, a `[S, 1024]` key, and a
//! `[S, 1024]` value, because sixteen query heads and eight key/value heads each
//! carry the *declared* head dimension 128. Head dimension is declared and not
//! `hidden_size / num_attention_heads = 64`; a planner that divides produces a
//! silently wrong shape on this checkpoint, and the refusal case below is that
//! exact divide meeting a named rule.
//!
//! Four maps carry the block's head layout, and each is a chain of forms
//! `ReindexForm` already admits — a split, an axis permutation, or a merge. No
//! coordinate arithmetic is needed anywhere here, which is what separates this
//! profile from the rotary composition, and no form is added to the family.
//!
//! | map | shapes | forms |
//! | --- | --- | --- |
//! | `query_projection_split` | `[T, 2048] -> [T, 16, 128]` | `split-axis` |
//! | `query_head_group_layout` | `[T, 16, 128] -> [8, 2, T, 128]` | `split-axis`, `permute-axes` |
//! | `key_value_head_layout` | `[S, 1024] -> [8, S, 128]` | `split-axis`, `permute-axes` |
//! | `attention_output_head_merge` | `[8, 2, T, 128] -> [T, 2048]` | `permute-axes`, `merge-axes`, `merge-axes` |
//!
//! Legal because the declared widths factor exactly: 2048 = 16 x 128 and
//! 1024 = 8 x 128, both static, so no symbolic requirement arises. The family
//! decides those products, and a split that misses either extent is refused by
//! name rather than approximated.
//!
//! The key and the value projections have the same width and the same target
//! layout, so they are the *same* map applied to two operands rather than two
//! maps that happen to agree; `the_key_and_value_edges_are_one_map` states that
//! as a check rather than leaving it to a reader of two identical bodies.
//!
//! # The direction, which is the whole content of the profile
//!
//! **The group index is the major axis of the split.** Splitting the sixteen-head
//! axis into `(8, 2)` puts the group first, so query head `h` sits at `(g, r)`
//! with `h = 2g + r`, and the group a head reads is `h / 2`. That is not a
//! convention chosen here: `split-axis` is normatively a row-major factorization
//! with the major factor first, so `(8, 2)` *means* `h = 2g + r` and the reading
//! `h % 8` is a different tensor of the same shape — reachable in this same
//! family by splitting `(2, 8)` and transposing. The reference's `repeat_kv` is
//! repeat-interleave, so `h / 2` is the one that denotes it, and
//! `the_head_split_reproduces_repeat_kv_and_the_tile_reading_does_not` settles
//! it by counting differing bits rather than by asserting it.
//!
//! # What this covers, and what it does not
//!
//! Every extent here is static and every operand is `tiler::f32@1`. `T` and `S`
//! are the C1 prefill row's ten new positions and ten context positions; the
//! extent-one batch axis the pinned reference carries is omitted, because it
//! contributes no element and no coordinate the maps mention. Element counts
//! therefore match the retained probe exactly: 16 × 10 × 128 = 20,480.
//!
//! It establishes nothing about a plan, a kernel, a physical layout, or whether
//! any of these maps costs a dispatch. A `Reindex` makes no storage claim, and
//! the contraction that consumes these layouts is a separate family.
//!
//! # Where the compared counts come from
//!
//! The `repeat_kv` comparison recomputes the reference materialization here,
//! from the repeat-interleave rule alone, rather than reading a retained fixture.
//! The two counts it produces — `0` and `17,920`, over fourteen of the sixteen
//! heads — are then checked against the attention-block probe's retained
//! `gqa_repeat_kv_matches_floor_div_differing_elements`,
//! `gqa_repeat_kv_matches_modulo_differing_elements`, and
//! `gqa_heads_whose_source_differs_between_the_two_readings` rows. Agreement of
//! an independently recomputed count with the probe's is evidence about both;
//! trusting the fixture would have been evidence about neither. The probe lives
//! at `spikes/program-planning/attention-block-reference/`.

use tiler_ir::semantic::{
    BuildError, F32, F32Reindex, InputKey, OutputKey, RegistryError, ReindexForm,
    SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

// --- the profile's declared constants ---------------------------------------

/// New positions of the C1 prefill row.
const T: usize = 10;
/// Context positions of the C1 prefill row.
const S: usize = 10;
/// Query heads the checkpoint declares.
const QUERY_HEADS: usize = 16;
/// Key/value heads, which are also the groups.
const GROUPS: usize = 8;
/// Query heads per group: `num_key_value_groups`.
const REPEATS: usize = 2;
/// The *declared* head dimension, not `hidden_size / num_attention_heads`.
const HEAD_DIM: usize = 128;
/// Width of the query projection, as the checkpoint's `q_proj` declares it.
const QUERY_WIDTH: usize = 2048;
/// Width of the key and value projections, as `k_proj` and `v_proj` declare them.
const KEY_VALUE_WIDTH: usize = 1024;
/// The head dimension a planner reaches by dividing, which this checkpoint refutes.
const DIVIDED_HEAD_DIM: usize = 64;

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: usize) -> Extent {
    Extent::new(u64::try_from(value).expect("a profile extent fits a u64"))
}

fn profile_shape<const N: usize>(dims: [usize; N]) -> Shape {
    Shape::try_from_dims(
        dims.into_iter()
            .map(|dim| u64::try_from(dim).expect("a profile extent fits a u64")),
    )
    .expect("a profile shape is admitted")
}

// --- the four maps ----------------------------------------------------------

/// `[T, 2048] -> [T, 16, 128]`: the query projection read as heads of width 128.
///
/// Legal because 2048 = 16 × 128 exactly; a product short of the extent is a
/// slice and a product past it is not total, and both are refused by name.
fn query_projection_split() -> Vec<ReindexForm> {
    vec![
        ReindexForm::split_axis(axis(1), [extent(QUERY_HEADS), extent(HEAD_DIM)])
            .expect("2048 = 16 x 128"),
    ]
}

/// `[T, 16, 128] -> [8, 2, T, 128]`: the head axis grouped, then moved outermost.
///
/// The split is `(8, 2)` with the group major, so the head at `(g, r)` is
/// `2g + r`; the permutation then reads `[T, 8, 2, 128]` as `[8, 2, T, 128]`.
fn query_head_group_layout() -> Vec<ReindexForm> {
    vec![
        ReindexForm::split_axis(axis(1), [extent(GROUPS), extent(REPEATS)])
            .expect("16 = 8 x 2, group major"),
        ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(3)])
            .expect("[T, g, r, d] -> [g, r, T, d]"),
    ]
}

/// `[S, 1024] -> [8, S, 128]`: the key or value projection as one row per group.
///
/// One map serving two edges. The key and value projections are both `[S, 1024]`
/// and both feed a `[g, s, d]` operand, so a second spelling would be two names
/// for one map rather than two maps.
fn key_value_head_layout() -> Vec<ReindexForm> {
    vec![
        ReindexForm::split_axis(axis(1), [extent(GROUPS), extent(HEAD_DIM)])
            .expect("1024 = 8 x 128"),
        ReindexForm::permute_axes([axis(1), axis(0), axis(2)]).expect("[S, g, d] -> [g, S, d]"),
    ]
}

/// `[8, 2, T, 128] -> [T, 2048]`: the inverse of the query layout, for the output.
///
/// The permutation restores `[T, 8, 2, 128]`, the first merge rebuilds the
/// sixteen-head axis from the group-major pair, and the second merges heads with
/// the head dimension back into the projection width.
fn attention_output_head_merge() -> Vec<ReindexForm> {
    vec![
        ReindexForm::permute_axes([axis(2), axis(0), axis(1), axis(3)])
            .expect("[g, r, T, d] -> [T, g, r, d]"),
        ReindexForm::merge_axes([axis(1), axis(2)]).expect("(g, r) -> h"),
        ReindexForm::merge_axes([axis(1), axis(2)]).expect("(h, d) -> the projection width"),
    ]
}

/// The whole query path, projection width to grouped head layout.
fn query_head_layout() -> Vec<ReindexForm> {
    let mut forms = query_projection_split();
    forms.extend(query_head_group_layout());
    forms
}

// --- evaluation helpers -----------------------------------------------------

/// Builds a tensor whose element `i` carries the big-endian bits of `bits(i)`.
///
/// The payloads are bit patterns rather than numbers: nothing here computes, so
/// a distinct integer per element is exactly the fixture that makes a
/// wrong-element read visible instead of coincidental.
fn tensor_of(shape: &Shape, bits: impl Fn(usize) -> u32) -> Tensor {
    let count = shape.element_count().expect("a profile shape is bounded");
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

/// Builds a tensor of ascending distinct payloads, so element `i` names index `i`.
fn ascending(shape: &Shape) -> Tensor {
    tensor_of(shape, |index| {
        u32::try_from(index).expect("a profile operand is small")
    })
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a profile result is a dense f32 tensor");
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

/// Applies a form chain to `operand` through the semantic and reference boundary.
///
/// Every form is applied by the registered operation authority, so a chain that
/// reached a result is a chain whose every occurrence was admitted against its
/// own operand's shape, and the result shape is the family's derivation rather
/// than a declaration made here.
fn evaluate_chain(operand: &Tensor, forms: &[ReindexForm]) -> (Vec<u32>, Shape) {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let mut value = builder
        .input::<F32>(
            InputKey::new("operand").expect("a valid key"),
            operand.shape().clone(),
        )
        .expect("an F32 input");
    for form in forms {
        value = F32Reindex::apply(&mut builder, form, value).expect("a profile form is admitted");
    }
    builder
        .output(OutputKey::new("result").expect("a valid key"), value)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let key = InputKey::new("operand").expect("a valid key");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(&program, &[InputBinding::new(&key, operand)])
        .expect("a profile program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a profile program has one output");
    };
    (payload_bits(output), output.shape().clone())
}

/// Asserts that `values` reads each of `0..count` exactly once.
///
/// Totality over the declared output domain and bijectivity onto the operand's
/// domain are one check on a distinct-payload fixture: a result of the derived
/// length whose payload is a permutation of the operand's indices read every
/// operand element exactly once and invented none.
fn assert_total_bijection(values: &[u32], count: usize, map: &str) {
    assert_eq!(
        values.len(),
        count,
        "{map} produced the wrong element count"
    );
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let expected: Vec<u32> =
        (0..u32::try_from(count).expect("a profile operand is small")).collect();
    assert_eq!(
        sorted, expected,
        "{map} is not a bijection onto its operand's domain"
    );
}

// --- the derived shapes -----------------------------------------------------

/// A form chain's result shape is derived from its operand, never declared.
///
/// [`ReindexForm::result_shape`] decides each occurrence against the shape it
/// actually receives, so a chain's final shape is the composition of those
/// derivations and nothing a caller stated. The four head-layout maps are the
/// worked instance: this file's header tabulates what each produces, and this is
/// where that table is checked rather than believed.
#[test]
fn each_map_derives_its_declared_result_shape() {
    let shape_after = |operand: [usize; 2], forms: &[ReindexForm]| -> Shape {
        let mut shape = profile_shape(operand);
        for form in forms {
            shape = form
                .result_shape(&shape)
                .expect("a profile form is admitted against its operand");
        }
        shape
    };

    assert_eq!(
        shape_after([T, QUERY_WIDTH], &query_projection_split()),
        profile_shape([T, QUERY_HEADS, HEAD_DIM])
    );
    assert_eq!(
        shape_after([T, QUERY_WIDTH], &query_head_layout()),
        profile_shape([GROUPS, REPEATS, T, HEAD_DIM])
    );
    assert_eq!(
        shape_after([S, KEY_VALUE_WIDTH], &key_value_head_layout()),
        profile_shape([GROUPS, S, HEAD_DIM])
    );

    let mut output = profile_shape([GROUPS, REPEATS, T, HEAD_DIM]);
    for form in &attention_output_head_merge() {
        output = form
            .result_shape(&output)
            .expect("a profile form is admitted against its operand");
    }
    assert_eq!(output, profile_shape([T, QUERY_WIDTH]));
}

// --- the maps, element for element ------------------------------------------

/// A split's factor order is semantics, and a shape check cannot see it.
///
/// `split-axis` is normatively a row-major factorization with the major factor
/// first, so the *order* of the two extents decides which operand element every
/// result coordinate reads while leaving the result shape untouched. The worked
/// instance is the query projection's sixteen-head axis split `(8, 2)`, which
/// means `h = 2g + r`; the identically shaped `(2, 8)` reading is evaluated
/// beside it and differs at fourteen of the sixteen heads, so the element
/// comparison discriminates the two readings and the extents never could.
#[test]
fn the_query_layout_reads_head_two_g_plus_r_at_every_coordinate() {
    let operand = profile_shape([T, QUERY_WIDTH]);
    let (values, shape) = evaluate_chain(&ascending(&operand), &query_head_layout());
    assert_eq!(shape, profile_shape([GROUPS, REPEATS, T, HEAD_DIM]));

    // Hand-derived from the profile's own statement rather than from the forms:
    // result `(g, r, t, d)` reads projection column `h * 128 + d` of row `t`,
    // with `h = 2g + r`. A helper that re-ran split and permute would agree with
    // the evaluator for reasons that say nothing about either being right.
    let mut expected = Vec::with_capacity(GROUPS * REPEATS * T * HEAD_DIM);
    for group in 0..GROUPS {
        for repeat in 0..REPEATS {
            let head = REPEATS * group + repeat;
            for position in 0..T {
                for lane in 0..HEAD_DIM {
                    expected.push(
                        u32::try_from(position * QUERY_WIDTH + head * HEAD_DIM + lane)
                            .expect("a profile operand is small"),
                    );
                }
            }
        }
    }
    assert_eq!(values, expected);
    assert_total_bijection(&values, T * QUERY_WIDTH, "the query head layout");

    // The projection split alone moves nothing in row-major order, so its whole
    // content is the shape — which is exactly what makes the *factor order* of
    // the following split, not this one, the load-bearing choice.
    let (split_values, split_shape) =
        evaluate_chain(&ascending(&operand), &query_projection_split());
    assert_eq!(split_shape, profile_shape([T, QUERY_HEADS, HEAD_DIM]));
    assert_eq!(
        split_values,
        (0..u32::try_from(T * QUERY_WIDTH).expect("a profile operand is small"))
            .collect::<Vec<_>>()
    );

    // The perturbation that a shape check cannot see. Splitting the head axis
    // `(2, 8)` and transposing the two head axes reaches the *same* rank-four
    // shape by the `h % 8` reading, so the assertion above discriminates the
    // direction rather than passing on the extents.
    let mut tile_reading = query_projection_split();
    tile_reading.push(
        ReindexForm::split_axis(axis(1), [extent(REPEATS), extent(GROUPS)])
            .expect("16 = 2 x 8, repeat major"),
    );
    tile_reading.push(
        ReindexForm::permute_axes([axis(2), axis(1), axis(0), axis(3)])
            .expect("[T, r, g, d] -> [g, r, T, d]"),
    );
    let (tiled, tiled_shape) = evaluate_chain(&ascending(&operand), &tile_reading);
    assert_eq!(
        tiled_shape, shape,
        "the wrong reading is identically shaped"
    );
    assert_ne!(tiled, values);
    // Fourteen of the sixteen heads read a different projection column, which is
    // the same fourteen the `repeat_kv` comparison counts.
    assert_eq!(
        tiled
            .chunks(T * HEAD_DIM)
            .zip(values.chunks(T * HEAD_DIM))
            .filter(|(left, right)| left != right)
            .count(),
        14
    );
}

/// A split composed with a permutation is a total bijection onto the operand.
///
/// A `Reindex` moves coordinates and neither invents nor drops an element, so a
/// distinct-payload operand returns as a permutation of its own indices at the
/// derived length — totality over the result domain and bijectivity onto the
/// operand's are one check on such a fixture. The worked instance is the key and
/// value projections' `[S, 1024] -> [8, S, 128]` map, whose per-coordinate
/// expectation is written from the profile's own statement rather than from a
/// second run of the forms.
#[test]
fn the_key_layout_gives_one_row_per_group() {
    let operand = profile_shape([S, KEY_VALUE_WIDTH]);
    let (values, shape) = evaluate_chain(&ascending(&operand), &key_value_head_layout());
    assert_eq!(shape, profile_shape([GROUPS, S, HEAD_DIM]));

    // Result `(g, s, d)` reads projection column `g * 128 + d` of row `s`.
    let mut expected = Vec::with_capacity(GROUPS * S * HEAD_DIM);
    for group in 0..GROUPS {
        for position in 0..S {
            for lane in 0..HEAD_DIM {
                expected.push(
                    u32::try_from(position * KEY_VALUE_WIDTH + group * HEAD_DIM + lane)
                        .expect("a profile operand is small"),
                );
            }
        }
    }
    assert_eq!(values, expected);
    assert_total_bijection(&values, S * KEY_VALUE_WIDTH, "the key/value head layout");
}

/// Two edges carrying the same layout are one map, decided by canonical encoding.
///
/// A form's identity is its canonical encoding, so "these two edges share a
/// coordinate map" is a decidable fact rather than a reader's impression of two
/// identical bodies — and the evaluated results are compared too, so the claim
/// covers what the maps denote and not only how they are spelled. The worked
/// instance is the key and the value projection, which have the same width and
/// the same target layout; were the value edge ever to need its own spelling,
/// this is what would fail.
#[test]
fn the_key_and_value_edges_are_one_map() {
    // The two projections have the same width and the same target layout, so the
    // profile spells one map and applies it twice. Comparing the canonical
    // encodings states that as a fact: were the value edge ever to need its own
    // spelling, this is what would fail rather than a reader noticing.
    let key = key_value_head_layout();
    let value = key_value_head_layout();
    let key_codes: Vec<_> = key.iter().map(ReindexForm::canonical_encoding).collect();
    let value_codes: Vec<_> = value.iter().map(ReindexForm::canonical_encoding).collect();
    assert_eq!(key_codes, value_codes);

    let operand = profile_shape([S, KEY_VALUE_WIDTH]);
    let (key_values, key_shape) = evaluate_chain(&ascending(&operand), &key);
    let (value_values, value_shape) = evaluate_chain(&ascending(&operand), &value);
    assert_eq!((key_values, key_shape), (value_values, value_shape));
}

/// A layout composed with its inverse is the identity, on payload and on shape.
///
/// `merge-axes` inverts `split-axis` and a permutation inverts its own order, so
/// a chain followed by its reverse returns the operand exactly — which is the
/// property anything that rebuilds a flat width from a structured layout depends
/// on. The worked instance is `[8, 2, T, 128] -> [T, 2048]`, checked first
/// coordinate by coordinate against a hand-derived decomposition and then as the
/// round trip the attention output takes.
#[test]
fn the_output_merge_inverts_the_query_layout() {
    let operand = profile_shape([GROUPS, REPEATS, T, HEAD_DIM]);
    let (values, shape) = evaluate_chain(&ascending(&operand), &attention_output_head_merge());
    assert_eq!(shape, profile_shape([T, QUERY_WIDTH]));

    // Result `(t, c)` decomposes `c` as `h * 128 + d` and reads
    // `(h / 2, h % 2, t, d)` of the head-major operand.
    let mut expected = Vec::with_capacity(T * QUERY_WIDTH);
    for position in 0..T {
        for column in 0..QUERY_WIDTH {
            let head = column / HEAD_DIM;
            let lane = column % HEAD_DIM;
            let group = head / REPEATS;
            let repeat = head % REPEATS;
            expected.push(
                u32::try_from(
                    group * REPEATS * T * HEAD_DIM
                        + repeat * T * HEAD_DIM
                        + position * HEAD_DIM
                        + lane,
                )
                .expect("a profile operand is small"),
            );
        }
    }
    assert_eq!(values, expected);
    assert_total_bijection(&values, GROUPS * REPEATS * T * HEAD_DIM, "the output merge");

    // The round trip, which is the property the attention output depends on: a
    // query laid out and merged back is the projection it started as, on both
    // the payload and the shape.
    let projection = profile_shape([T, QUERY_WIDTH]);
    let mut round_trip = query_head_layout();
    round_trip.extend(attention_output_head_merge());
    let (values, shape) = evaluate_chain(&ascending(&projection), &round_trip);
    assert_eq!(shape, projection);
    assert_eq!(
        values,
        (0..u32::try_from(T * QUERY_WIDTH).expect("a profile operand is small"))
            .collect::<Vec<_>>()
    );
}

// --- h = 2g + r against `repeat_kv` -----------------------------------------

/// The tile reading: split the head axis `(2, 8)` and transpose, so `g = h % 8`.
///
/// Retained as the perturbation, not as an alternative. It is expressible in the
/// same family and produces an identically shaped `[8, 2, S, 128]` tensor, which
/// is precisely why a shape check cannot separate the two readings.
fn tile_head_split() -> Vec<ReindexForm> {
    vec![
        ReindexForm::split_axis(axis(0), [extent(REPEATS), extent(GROUPS)])
            .expect("16 = 2 x 8, repeat major"),
        ReindexForm::permute_axes([axis(1), axis(0), axis(2), axis(3)])
            .expect("[r, g, S, d] -> [g, r, S, d]"),
    ]
}

/// The profile's reading: split the head axis `(8, 2)`, so `h = 2g + r`.
fn interleave_head_split() -> Vec<ReindexForm> {
    vec![
        ReindexForm::split_axis(axis(0), [extent(GROUPS), extent(REPEATS)])
            .expect("16 = 8 x 2, group major"),
    ]
}

/// A repetition constant along an axis is a layout rather than an operation.
///
/// When a materialized tensor holds the same element at every coordinate of an
/// axis, a `Reindex` reaches it without computing anything — that is the general
/// sense in which such a repetition is free, and it is a claim about the
/// coordinate map rather than about a storage decision. Which of two candidate
/// maps denotes the materialization has to be settled by counting differing
/// elements, because both produce the identical `[8, 2, S, 128]` shape. The
/// worked instance is `repeat_kv` recomputed here from the repeat-interleave rule
/// alone: `h = 2g + r` differs at zero of the 20,480 elements and the `h % 8`
/// reading at 17,920 across fourteen heads.
#[test]
fn the_head_split_reproduces_repeat_kv_and_the_tile_reading_does_not() {
    // `repeat_kv` recomputed here from the repeat-interleave rule alone: head `h`
    // of the materialized `[16, S, 128]` tensor is group `h / 2`, every element.
    // This is the reference's composition, restated; nothing is read from a
    // retained fixture, and the counts it yields are compared to the probe's
    // afterwards.
    let repeated = profile_shape([QUERY_HEADS, S, HEAD_DIM]);
    let repeat_kv = tensor_of(&repeated, |index| {
        let lane = index % HEAD_DIM;
        let position = (index / HEAD_DIM) % S;
        let head = index / (S * HEAD_DIM);
        u32::try_from((head / REPEATS) * S * HEAD_DIM + position * HEAD_DIM + lane)
            .expect("a profile operand is small")
    });

    // What the layout claims: at `(g, r, s, d)` the materialized tensor holds the
    // *group's* element, identically for every `r`. That is the sense in which
    // the repetition is free — it is the layout, not an operation.
    let mut group_constant = Vec::with_capacity(GROUPS * REPEATS * S * HEAD_DIM);
    for group in 0..GROUPS {
        for _ in 0..REPEATS {
            for position in 0..S {
                for lane in 0..HEAD_DIM {
                    group_constant.push(
                        u32::try_from(group * S * HEAD_DIM + position * HEAD_DIM + lane)
                            .expect("a profile operand is small"),
                    );
                }
            }
        }
    }
    assert_eq!(group_constant.len(), QUERY_HEADS * S * HEAD_DIM);

    let laid_out = profile_shape([GROUPS, REPEATS, S, HEAD_DIM]);
    let (interleaved, interleaved_shape) = evaluate_chain(&repeat_kv, &interleave_head_split());
    let (tiled, tiled_shape) = evaluate_chain(&repeat_kv, &tile_head_split());

    // Identically shaped, which is the point: only the elements separate them.
    assert_eq!(interleaved_shape, laid_out);
    assert_eq!(tiled_shape, laid_out);

    let differing = |values: &[u32]| -> usize {
        values
            .iter()
            .zip(&group_constant)
            .filter(|(left, right)| left != right)
            .count()
    };
    let differing_heads = |values: &[u32]| -> usize {
        values
            .chunks(S * HEAD_DIM)
            .zip(group_constant.chunks(S * HEAD_DIM))
            .filter(|(left, right)| left != right)
            .count()
    };

    // The comparison and its perturbation, in bits. `20_480` is the C1 element
    // count the probe records; `0` and `17_920` over fourteen heads are its
    // `gqa_repeat_kv_matches_floor_div_differing_elements`,
    // `gqa_repeat_kv_matches_modulo_differing_elements`, and
    // `gqa_heads_whose_source_differs_between_the_two_readings` rows, reproduced
    // by an independent recomputation.
    assert_eq!(interleaved.len(), 20_480);
    assert_eq!(differing(&interleaved), 0, "h = 2g + r denotes repeat_kv");
    assert_eq!(differing_heads(&interleaved), 0);
    assert_eq!(
        differing(&tiled),
        17_920,
        "the h % 8 reading differs from repeat_kv, and a shape check would have passed it"
    );
    assert_eq!(differing_heads(&tiled), 14);
    assert_ne!(interleaved, tiled);

    // And the fixture is discriminating rather than uniform: the two readings
    // agree at exactly the two heads where `h / 2` and `h % 8` coincide.
    assert_eq!(
        interleaved
            .chunks(S * HEAD_DIM)
            .zip(tiled.chunks(S * HEAD_DIM))
            .filter(|(left, right)| left == right)
            .count(),
        2
    );
}

/// A coordinate map's index table is recoverable by evaluating the map itself.
///
/// Driving an operand of distinct ascending payloads through a chain and reading
/// back where each landed derives the table the map denotes, so the mapping is
/// *observed* rather than restated beside the forms that produce it. The worked
/// instance is the query-head-to-key-head table, which comes back as
/// `0 0 1 1 … 7 7` — floor division — for the group-major split, against `h % 8`
/// for the tile reading.
#[test]
fn the_query_head_to_key_head_table_is_floor_division() {
    // The mapping table the probe records, derived from the maps themselves: a
    // probe of sixteen ascending head indices, split, tells which slot each head
    // landed in, and its group is that slot's major coordinate. The two trailing
    // unit axes carry no element and exist only so both readings' forms apply at
    // the rank they are written for.
    let heads = profile_shape([QUERY_HEADS, 1, 1]);
    let group_of = |forms: &[ReindexForm]| -> Vec<usize> {
        let (values, shape) = evaluate_chain(&ascending(&heads), forms);
        assert_eq!(shape, profile_shape([GROUPS, REPEATS, 1, 1]));
        (0..QUERY_HEADS)
            .map(|head| {
                let slot = values
                    .iter()
                    .position(|value| {
                        usize::try_from(*value).is_ok_and(|head_index| head_index == head)
                    })
                    .expect("every head appears exactly once");
                slot / REPEATS
            })
            .collect()
    };

    // `gqa_query_head_to_key_head` in the retained record: 0 0 1 1 ... 7 7.
    assert_eq!(
        group_of(&interleave_head_split()),
        vec![0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7]
    );
    // The tile reading's table, which is `h % 8` and is wrong at fourteen heads.
    assert_eq!(
        group_of(&tile_head_split()),
        (0..QUERY_HEADS)
            .map(|head| head % GROUPS)
            .collect::<Vec<_>>()
    );
}

// --- the profile's malformed neighbours --------------------------------------

/// Returns the provider diagnostic code a refused application carried.
///
/// Asserting the code rather than only `is_err` is what makes each refusal below
/// evidence about the rule it names: a poisoned builder or a bound violation
/// would be an error too, and neither would be the check.
fn refusal_code(error: &BuildError) -> String {
    let BuildError::SemanticRegistry(RegistryError::RejectedOperationApplication(rejection)) =
        error
    else {
        panic!("a profile refusal is a provider-attributed rejection, not {error}");
    };
    rejection.source_error().code().as_str().to_owned()
}

/// A split whose factors do not exhaust the axis fails closed, under its own code.
///
/// `split-axis` admits a factorization only when the product is the axis extent
/// exactly, and it distinguishes the two ways of missing: a product short of the
/// extent reads a prefix and is refused as `reindex.split.not-surjective`, a
/// product past it is refused as `reindex.split.not-total`. Neither is
/// reinterpreted as a slice or a pad, and the code is asserted rather than only
/// the failure, so each refusal is evidence about the rule it names. The worked
/// instances are the divide this checkpoint refutes — sixteen heads of
/// `hidden_size / num_attention_heads = 64` against a 2,048-wide projection — and
/// the query head count applied to a 1,024-wide key projection. Both admitted
/// neighbours follow, so the refusals discriminate rather than reject every split
/// the profile presents.
#[test]
fn a_split_whose_factors_miss_the_axis_extent_refuses_by_name() {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let query = builder
        .input::<F32>(
            InputKey::new("query").expect("a valid key"),
            profile_shape([T, QUERY_WIDTH]),
        )
        .expect("an F32 input");
    let key = builder
        .input::<F32>(
            InputKey::new("key").expect("a valid key"),
            profile_shape([S, KEY_VALUE_WIDTH]),
        )
        .expect("an F32 input");

    // The divide the evidence prerequisite names: sixteen heads of
    // `hidden_size / num_attention_heads = 64` account for 1024 of the 2048
    // columns, so the map reads a prefix and is a slice.
    let divided = ReindexForm::split_axis(axis(1), [extent(QUERY_HEADS), extent(DIVIDED_HEAD_DIM)])
        .expect("the form itself is well shaped");
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &divided, query).unwrap_err()),
        "reindex.split.not-surjective",
        "16 x 64 is half a 2048-wide projection, so the mapping is a slice"
    );

    // The query head count applied to the key projection: 16 x 128 reads past the
    // end of a 1024-wide axis, so the mapping is not total.
    let query_heads_on_key =
        ReindexForm::split_axis(axis(1), [extent(QUERY_HEADS), extent(HEAD_DIM)])
            .expect("the form itself is well shaped");
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &query_heads_on_key, key).unwrap_err()),
        "reindex.split.not-total",
        "16 x 128 exceeds a 1024-wide projection, so the mapping is not total"
    );

    // The admitted neighbours, so the two refusals are known to discriminate
    // rather than to refuse every split this profile presents.
    assert!(F32Reindex::apply(&mut builder, &query_projection_split()[0], query).is_ok());
    assert!(F32Reindex::apply(&mut builder, &key_value_head_layout()[0], key).is_ok());
}

/// An axis order that is not a permutation fails closed, wherever it is decidable.
///
/// Which point refuses is decided by what the rule depends on, and both are
/// exercised. A repeated axis is a property of the order alone, so it never
/// reaches an occurrence and is refused at construction under
/// `reindex.permute.not-a-permutation`; an axis the operand does not have needs
/// the operand and is refused at the occurrence under the same rule; and a
/// well-formed order of the wrong length is refused under `reindex.permute.rank`,
/// which is its own reason rather than the same one. The worked instances are the
/// head-layout permutation with its group axis typed twice, an axis 4 against a
/// rank-four operand, and the key layout's rank-three order against the rank-four
/// query layout — with the profile's own permutation as the admitted neighbour.
#[test]
fn an_axis_order_that_is_not_a_permutation_refuses_by_name() {
    // A repeated axis is a property of the order alone, so it never reaches an
    // occurrence: the head-layout permutation with its group axis typed twice
    // reads one axis twice and drops another.
    let repeated = ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(1)])
        .expect_err("a repeated axis is not a permutation");
    assert_eq!(
        repeated.diagnostic_code(),
        "reindex.permute.not-a-permutation"
    );

    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let grouped = builder
        .input::<F32>(
            InputKey::new("grouped").expect("a valid key"),
            profile_shape([T, GROUPS, REPEATS, HEAD_DIM]),
        )
        .expect("an F32 input");

    // Distinct axes of the right count, one of which the operand does not have.
    // That is the remaining way an order fails to be a permutation, and it is
    // decided against the occurrence rather than at construction.
    let out_of_range = ReindexForm::permute_axes([axis(1), axis(2), axis(0), axis(4)])
        .expect("distinct axes make a well-formed order");
    assert_eq!(
        refusal_code(&F32Reindex::apply(&mut builder, &out_of_range, grouped).unwrap_err()),
        "reindex.permute.not-a-permutation",
        "a rank-four operand has no axis 4"
    );

    // The key layout's rank-three permutation applied to the rank-four query
    // layout: a well-formed order of the wrong rank, under its own rule.
    assert_eq!(
        refusal_code(
            &F32Reindex::apply(&mut builder, &key_value_head_layout()[1], grouped).unwrap_err()
        ),
        "reindex.permute.rank",
        "the key layout's order names three axes and the query layout has four"
    );

    // The admitted neighbour: the profile's own head-layout permutation.
    assert!(F32Reindex::apply(&mut builder, &query_head_group_layout()[1], grouped).is_ok());
}

/// `merge-axes` requires an adjacent run, and decides that without an operand.
///
/// Adjacency is a property of the named axes alone, so a merge over a gap is
/// refused at construction under `reindex.merge.non-adjacent-axes` rather than
/// deferred to an occurrence: merging across an intervening axis is a permutation
/// composed with a merge, and the family makes the caller spell it as one. The
/// worked instance is the output inverse written without first moving the
/// position axis out from between the group and repetition axes, with the
/// profile's own merge — the same axes after the permutation — as the admitted
/// neighbour.
#[test]
fn a_merge_over_non_adjacent_axes_refuses_by_name() {
    // The output inverse spelled without first moving the position axis out from
    // between the group and repeat axes: over a `[8, T, 2, 128]` layout the head
    // axes are axis 0 and axis 2, and merging them is a permutation composed with
    // a merge rather than a merge. Adjacency is a property of the axes alone, so
    // the refusal lands at construction and no operand is consulted.
    let non_adjacent = ReindexForm::merge_axes([axis(0), axis(2)])
        .expect_err("axis 0 and axis 2 are not an adjacent run");
    assert_eq!(
        non_adjacent.diagnostic_code(),
        "reindex.merge.non-adjacent-axes"
    );

    // The admitted neighbour, which is the profile's own merge: after the
    // permutation the group and repeat axes are adjacent.
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let permuted = builder
        .input::<F32>(
            InputKey::new("permuted").expect("a valid key"),
            profile_shape([T, GROUPS, REPEATS, HEAD_DIM]),
        )
        .expect("an F32 input");
    assert!(F32Reindex::apply(&mut builder, &attention_output_head_merge()[1], permuted).is_ok());
}
