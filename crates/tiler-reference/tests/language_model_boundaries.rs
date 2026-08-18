//! Bounded composition evidence for the two language-model boundary programs.
//!
//! P1 turns token IDs into a residual stream with one gather. P3 explicitly
//! widens the final RMS-normalization weight, normalizes the residual stream,
//! and applies the strict `td,od->to` vocabulary projection. These are consumer
//! conformance fixtures over existing operation families, not new semantic
//! authority and not compiler, lowering, runtime, or device support.
//!
//! # Exact shape versus evaluated values
//!
//! The exact C1 programs below carry the checkpoint's literal
//! `[151936, 1024]` tied matrix at prefill `T = 10` and decode `T = 1`. They are
//! constructed, semantically validated, and inspected without materializing
//! that matrix: its 155,582,464 elements exceed the reference tensor bound.
//! Evaluation uses the extent-independent analogue `V = 3`, `H = 2`, with the
//! same gather, explicit weight widening, RMS normalization, and strict
//! contraction semantics. Nothing here claims that the analogue materializes
//! C1 or authorizes a larger reference-evaluation budget.
//!
//! # The two logits modes are two programs
//!
//! The projection has two modes and they are not interchangeable. The
//! conformance mode retains every position, which is what the oracle compares
//! against and what the pinned reference produces — its `logits_to_keep=0`
//! becomes `slice(0, None)`, so the value that reads like "keep none" keeps all.
//! A prefill pass that only needs its last token's logits wants one position.
//!
//! They are **two program shapes rather than one program with a switch**:
//! [`build_projection_program`] declares `[T, vocabulary]` and
//! [`build_final_position_projection_program`] declares `[1, vocabulary]`, and a
//! caller reads which it holds off the declared result. A single program that
//! could return either would be two programs presented as one, and its result
//! extent would not be a fact about the program. The all-positions mode is
//! unchanged and stays the default.
//!
//! The final-position mode selects row `T - 1` of the residual stream *before*
//! normalizing and projecting, which is where the residency is: the saving is
//! the `[T, vocabulary]` logits that are never formed, not a `[T, vocabulary]`
//! tensor that is formed and then read from. The selection is an occurrence of
//! the already-admitted `tiler::slice-f32@1` family — injective and not
//! surjective, so outside both `tiler::reindex-f32@1` and
//! `tiler::broadcast-f32@1` — and this fixture invents no family and no
//! semantics.

use tiler_ir::semantic::{BroadcastAxisMapping, BroadcastAxisSource};
use tiler_ir::semantic::{
    BuildError, ContractionIndex, ContractionIndexStructure, F32, F32Broadcast, F32Gather,
    F32Reindex, F32RmsNorm, F32Slice, F32TensorContraction, InputKey, OpKey, OutputKey,
    RMS_NORM_F32_REFERENCE_EPS_BITS, ReindexForm, ResolvedValueType, SemanticProgram,
    SemanticProgramBuilder, SliceAxisSelection, SliceSelection, broadcast_f32_op, gather_f32_op,
    gather_index_resolved_type, reindex_f32_op, rms_norm_f32_op, slice_f32_op,
    strict_tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};
use tiler_reference::{
    EvaluationError, FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator,
    ReferenceOperationError, Tensor, TensorPayloadView,
};

const C1_VOCABULARY: u64 = 151_936;
const C1_HIDDEN: u64 = 1_024;
const ANALOGUE_VOCABULARY: u64 = 3;
const ANALOGUE_HIDDEN: u64 = 2;

/// Positions at the benchmark row's long prefill end.
///
/// A residency figure is a property of one row rather than of the machinery, so
/// every extent that fixes one lives here beside the other fixture sizes. The
/// program construction below reads `t`, `vocabulary`, and `hidden` and names no
/// row of its own.
const B1D_PREFILL_POSITIONS: u64 = 8_192;

/// Bytes one binary32 element occupies in the dense row-major logits contract.
const F32_BYTES: u64 = 4;

/// The pinned checkpoint's whole F32 weight budget.
///
/// Twenty-eight layers at 62,923,776 bytes is 1,761,865,728; the tied
/// `[151936, 1024]` matrix is 622,329,856 and `model.norm` is 4,096.
const C1_F32_WEIGHT_BYTES: u64 = 2_384_199_680;

#[derive(Debug)]
struct EmbeddingProgram {
    program: SemanticProgram,
    token_ids: InputKey,
    tied_embedding: InputKey,
}

#[derive(Debug)]
struct ProjectionProgram {
    program: SemanticProgram,
    hidden: InputKey,
    norm_weight: InputKey,
    tied_embedding: InputKey,
}

fn shape(dims: impl IntoIterator<Item = u64>) -> Shape {
    Shape::try_from_dims(dims).expect("a fixture shape is representable")
}

fn axis(value: u32) -> Axis {
    Axis::new(value)
}

fn extent(value: u64) -> Extent {
    Extent::new(value)
}

/// The checkpoint projection structure, `td,od->to`.
fn vocabulary_projection_structure() -> ContractionIndexStructure {
    let index = ContractionIndex::new;
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("td,od->to is admitted")
}

fn norm_weight_mapping(t: u64, hidden: u64) -> BroadcastAxisMapping {
    BroadcastAxisMapping::new(
        [extent(t), extent(hidden)],
        [
            BroadcastAxisSource::Replicate,
            BroadcastAxisSource::FromOperand(axis(0)),
        ],
    )
    .expect("[hidden] explicitly replicates over a non-unit position axis")
}

fn widen_norm_weight(
    builder: &mut SemanticProgramBuilder,
    weight: tiler_ir::semantic::Value<F32>,
    t: u64,
    hidden: u64,
) -> tiler_ir::semantic::Value<F32> {
    if t == 1 {
        let form = ReindexForm::insert_unit_axis(axis(0))
            .expect("a leading unit-axis insertion is admitted");
        F32Reindex::apply(builder, &form, weight)
            .expect("[hidden] -> [1, hidden] is a reindex, not a broadcast")
    } else {
        F32Broadcast::apply(builder, &norm_weight_mapping(t, hidden), weight)
            .expect("the norm weight explicitly replicates across positions")
    }
}

fn build_embedding_program(t: u64, vocabulary: u64, hidden: u64) -> EmbeddingProgram {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let token_ids = InputKey::new("token_ids").expect("a bounded input key");
    let tied_embedding = InputKey::new("W_embed").expect("a bounded input key");
    let token_value = builder
        .input_resolved(token_ids.clone(), shape([t]), gather_index_resolved_type())
        .expect("token IDs use the admitted u32 identity");
    let embedding_value = builder
        .input::<F32>(tied_embedding.clone(), shape([vocabulary, hidden]))
        .expect("the embedding table is F32");
    let residual = F32Gather::apply(&mut builder, embedding_value, token_value, axis(0))
        .expect("token IDs gather whole embedding rows");
    builder
        .output(
            OutputKey::new("x0").expect("a bounded output key"),
            residual,
        )
        .expect("P1 has one output");
    builder.validate().expect("P1 verifies semantically");
    let program = builder.build().expect("P1 is complete");
    EmbeddingProgram {
        program,
        token_ids,
        tied_embedding,
    }
}

fn build_projection_program(t: u64, vocabulary: u64, hidden: u64) -> ProjectionProgram {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let hidden_key = InputKey::new("h").expect("a bounded input key");
    let norm_weight = InputKey::new("w_norm").expect("a bounded input key");
    let tied_embedding = InputKey::new("W_embed").expect("a bounded input key");
    let hidden_value = builder
        .input::<F32>(hidden_key.clone(), shape([t, hidden]))
        .expect("the hidden state is F32");
    let norm_weight_value = builder
        .input::<F32>(norm_weight.clone(), shape([hidden]))
        .expect("the normalization weight is F32");
    let embedding_value = builder
        .input::<F32>(tied_embedding.clone(), shape([vocabulary, hidden]))
        .expect("the tied projection weight is F32");

    let widened_weight = widen_norm_weight(&mut builder, norm_weight_value, t, hidden);
    let normalized = F32RmsNorm::apply(
        &mut builder,
        hidden_value,
        widened_weight,
        axis(1),
        RMS_NORM_F32_REFERENCE_EPS_BITS,
    )
    .expect("RMS normalization receives the explicitly widened weight");
    let logits = F32TensorContraction::apply(
        &mut builder,
        &vocabulary_projection_structure(),
        normalized,
        embedding_value,
    )
    .expect("[T, hidden] x [vocabulary, hidden] contracts as td,od->to");
    builder
        .output(
            OutputKey::new("logits").expect("a bounded output key"),
            logits,
        )
        .expect("P3 has one output");
    builder.validate().expect("P3 verifies semantically");
    let program = builder.build().expect("P3 is complete");
    ProjectionProgram {
        program,
        hidden: hidden_key,
        norm_weight,
        tied_embedding,
    }
}

/// Selects the final position of a `[t, hidden]` residual stream.
///
/// The family's structure is total, so this states one entry per operand axis in
/// axis order: a one-coordinate window at `t - 1` on the position axis, and the
/// hidden axis whole. Rank is preserved and no `remove-unit-axis` reindex
/// follows, because the projection wants a `[1, hidden]` operand rather than a
/// `[hidden]` one — the extent-one axis this selection leaves behind is the
/// position axis the result declares.
fn final_position_selection(t: u64) -> SliceSelection {
    SliceSelection::new([
        SliceAxisSelection::static_window(t - 1, extent(1)),
        SliceAxisSelection::WholeAxis,
    ])
    .expect("a one-coordinate window beside a whole axis is a well-formed selection")
}

/// P3's final-position mode: the same three inputs, a `[1, vocabulary]` result.
///
/// The selection is written first, so the normalization and the projection each
/// see one position and the `[T, vocabulary]` logits are never formed. That is
/// sound because both are row-independent: `tiler::rms-norm-f32@1` reduces over
/// the hidden axis, so each position's result depends on that position's row
/// alone, and `td,od->to` sums over `d` for each `(t, o)` independently. The
/// widened weight is the same vector in both modes — replicated across `T`
/// positions there, across the one selected position here — so the selected
/// row meets identical operands either way.
///
/// # Errors
///
/// Returns the construction refusal without mutating the graph. `t = 1` is the
/// reachable one: selecting the only position covers its axis, which is the
/// `whole-axis` relation, and the family refuses that spelling by name.
fn build_final_position_projection_program(
    t: u64,
    vocabulary: u64,
    hidden: u64,
) -> Result<ProjectionProgram, BuildError> {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let hidden_key = InputKey::new("h").expect("a bounded input key");
    let norm_weight = InputKey::new("w_norm").expect("a bounded input key");
    let tied_embedding = InputKey::new("W_embed").expect("a bounded input key");
    let hidden_value = builder
        .input::<F32>(hidden_key.clone(), shape([t, hidden]))
        .expect("the hidden state is F32");
    let norm_weight_value = builder
        .input::<F32>(norm_weight.clone(), shape([hidden]))
        .expect("the normalization weight is F32");
    let embedding_value = builder
        .input::<F32>(tied_embedding.clone(), shape([vocabulary, hidden]))
        .expect("the tied projection weight is F32");

    let final_position = F32Slice::apply(&mut builder, &final_position_selection(t), hidden_value)?;
    // After the selection the stream carries one position, so the explicit
    // widening is the decode-shaped unit-axis insertion in both prefill and
    // decode. That is a consequence of the selection rather than a second mode.
    let widened_weight = widen_norm_weight(&mut builder, norm_weight_value, 1, hidden);
    let normalized = F32RmsNorm::apply(
        &mut builder,
        final_position,
        widened_weight,
        axis(1),
        RMS_NORM_F32_REFERENCE_EPS_BITS,
    )
    .expect("RMS normalization receives the explicitly widened weight");
    let logits = F32TensorContraction::apply(
        &mut builder,
        &vocabulary_projection_structure(),
        normalized,
        embedding_value,
    )
    .expect("[1, hidden] x [vocabulary, hidden] contracts as td,od->to");
    builder
        .output(
            OutputKey::new("logits").expect("a bounded output key"),
            logits,
        )
        .expect("the final-position mode has one output");
    builder
        .validate()
        .expect("the final-position program verifies semantically");
    let program = builder
        .build()
        .expect("the final-position program is complete");
    Ok(ProjectionProgram {
        program,
        hidden: hidden_key,
        norm_weight,
        tied_embedding,
    })
}

fn output_shape(program: &SemanticProgram) -> Shape {
    let output = program.outputs().next().expect("one output");
    program
        .value(output.value())
        .expect("the output belongs to the program")
        .shape()
        .as_static()
        .expect("these programs use literal shapes")
        .clone()
}

fn input_shapes(program: &SemanticProgram) -> Vec<Shape> {
    program
        .inputs()
        .map(|input| {
            program
                .value(input.value())
                .expect("an input belongs to its program")
                .shape()
                .as_static()
                .expect("these programs use literal shapes")
                .clone()
        })
        .collect()
}

fn input_types(program: &SemanticProgram) -> Vec<ResolvedValueType> {
    program
        .inputs()
        .map(|input| {
            program
                .value(input.value())
                .expect("an input belongs to its program")
                .resolved_type()
                .clone()
        })
        .collect()
}

fn operation_keys(program: &SemanticProgram) -> Vec<OpKey> {
    program
        .operations()
        .map(|operation| operation.key().clone())
        .collect()
}

/// The compiler's governed budget check derives this actual from the program
/// declaration. This fixture records the arithmetic without invoking the
/// compiler recognizer, which does not support either whole program.
fn derived_buffer_actual(program: &SemanticProgram) -> usize {
    program.input_count() + program.output_count() * 4
}

/// Bytes the dense row-major logits contract occupies, read off the declaration.
///
/// Derived from the program's own declared result rather than from a written
/// figure, so a residency claim cannot drift from the shape the program states.
fn declared_logit_bytes(program: &SemanticProgram) -> u64 {
    let elements = output_shape(program)
        .element_count()
        .expect("a declared logits extent product is representable");
    u64::try_from(elements).expect("a declared element count fits an unsigned 64-bit byte figure")
        * F32_BYTES
}

fn f32_element(value: f32) -> ReferenceElement {
    ReferenceElement::from_float_bits(
        value.to_bits().to_be_bytes(),
        FloatBitOrder::MostSignificantByteFirst,
    )
    .expect("an F32 payload is four bytes")
}

fn f32_tensor(dims: impl IntoIterator<Item = u64>, values: &[f32]) -> Tensor {
    let shape = shape(dims);
    assert_eq!(shape.element_count(), Some(values.len()));
    Tensor::dense(
        F32::resolved_type(),
        shape,
        values.iter().copied().map(f32_element).collect(),
    )
    .expect("a bounded F32 tensor")
}

fn u32_tensor(dims: impl IntoIterator<Item = u64>, values: &[u32]) -> Tensor {
    let shape = shape(dims);
    assert_eq!(shape.element_count(), Some(values.len()));
    Tensor::dense(
        gather_index_resolved_type(),
        shape,
        values
            .iter()
            .map(|value| ReferenceElement::new(value.to_be_bytes()).expect("four bytes"))
            .collect(),
    )
    .expect("a bounded u32 tensor")
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a boundary result is dense")
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("a boundary element is four bytes"),
            )
        })
        .collect()
}

fn evaluate(
    program: &SemanticProgram,
    bindings: &[InputBinding<'_>],
) -> Result<Vec<Tensor>, EvaluationError> {
    ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(program, bindings)
}

#[test]
fn exact_c1_prefill_and_decode_programs_have_the_measured_shapes_and_counts() {
    const TIED_MATRIX_ELEMENTS: u64 = C1_VOCABULARY * C1_HIDDEN;
    const TIED_MATRIX_BYTES: u64 = TIED_MATRIX_ELEMENTS * 4;
    assert_eq!(TIED_MATRIX_ELEMENTS, 155_582_464);
    assert_eq!(TIED_MATRIX_BYTES, 622_329_856);

    for t in [10, 1] {
        let p1 = build_embedding_program(t, C1_VOCABULARY, C1_HIDDEN);
        assert_eq!(p1.program.input_count(), 2);
        assert_eq!(p1.program.operation_count(), 1);
        assert_eq!(p1.program.value_count(), 3);
        assert_eq!(p1.program.output_count(), 1);
        assert_eq!(derived_buffer_actual(&p1.program), 6);
        assert_eq!(
            p1.program
                .inputs()
                .map(|input| input.key().as_str().to_owned())
                .collect::<Vec<_>>(),
            vec!["token_ids", "W_embed"]
        );
        assert_eq!(
            input_shapes(&p1.program),
            vec![shape([t]), shape([C1_VOCABULARY, C1_HIDDEN])]
        );
        assert_eq!(
            input_types(&p1.program),
            vec![gather_index_resolved_type(), F32::resolved_type()]
        );
        assert_eq!(output_shape(&p1.program), shape([t, C1_HIDDEN]));
        assert_eq!(
            p1.program.outputs().next().expect("x0").key().as_str(),
            "x0"
        );
        assert_eq!(operation_keys(&p1.program), vec![gather_f32_op()]);

        let p3 = build_projection_program(t, C1_VOCABULARY, C1_HIDDEN);
        assert_eq!(p3.program.input_count(), 3);
        assert_eq!(p3.program.operation_count(), 3);
        assert_eq!(p3.program.value_count(), 6);
        assert_eq!(p3.program.output_count(), 1);
        assert_eq!(derived_buffer_actual(&p3.program), 7);
        assert_eq!(
            p3.program
                .inputs()
                .map(|input| input.key().as_str().to_owned())
                .collect::<Vec<_>>(),
            vec!["h", "w_norm", "W_embed"]
        );
        assert_eq!(
            input_shapes(&p3.program),
            vec![
                shape([t, C1_HIDDEN]),
                shape([C1_HIDDEN]),
                shape([C1_VOCABULARY, C1_HIDDEN]),
            ]
        );
        assert_eq!(
            input_types(&p3.program),
            vec![
                F32::resolved_type(),
                F32::resolved_type(),
                F32::resolved_type(),
            ]
        );
        let projection = p3.program.operations().next_back().expect("the projection");
        let projection_weight = p3
            .program
            .inputs()
            .nth(2)
            .expect("W_embed is the third input")
            .value();
        assert_eq!(
            projection.operands().nth(1),
            Some(projection_weight),
            "td,od->to reads W_embed as its od operand"
        );
        assert_eq!(output_shape(&p3.program), shape([t, C1_VOCABULARY]));
        assert_eq!(
            p3.program.outputs().next().expect("logits").key().as_str(),
            "logits"
        );
        let widening = if t == 1 {
            reindex_f32_op()
        } else {
            broadcast_f32_op()
        };
        assert_eq!(
            operation_keys(&p3.program),
            vec![
                widening,
                rms_norm_f32_op(),
                strict_tensor_contraction_f32_op(),
            ]
        );
    }
}

#[test]
fn bounded_prefill_and_decode_analogues_use_one_tied_tensor_and_match_literal_results() {
    // One consumer-owned tensor supplies both programs. Its three rows are
    // e0, e1, and e0 + e1: P1 can visibly select/reorder them, while P3's three
    // output columns expose each normalized component and their strict sum.
    let tied_embedding = f32_tensor(
        [ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN],
        &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let norm_weight = f32_tensor([ANALOGUE_HIDDEN], &[1.0, 2.0]);
    let evaluator = ReferenceEvaluator::standard().expect("the standard evaluator opens");

    for (t, token_values, hidden_values, expected_embeddings, expected_logits) in [
        (
            2,
            vec![2, 0],
            vec![3.0, 4.0, 0.0, 0.0],
            vec![
                1.0_f32.to_bits(),
                1.0_f32.to_bits(),
                1.0_f32.to_bits(),
                0.0_f32.to_bits(),
            ],
            vec![0x3f59_3923, 0x4010_d0c2, 0x4047_1f0b, 0, 0, 0],
        ),
        (
            1,
            vec![2],
            vec![3.0, 4.0],
            vec![1.0_f32.to_bits(), 1.0_f32.to_bits()],
            vec![0x3f59_3923, 0x4010_d0c2, 0x4047_1f0b],
        ),
    ] {
        let p1 = build_embedding_program(t, ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN);
        let p3 = build_projection_program(t, ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN);
        let token_ids = u32_tensor([t], &token_values);
        let hidden = f32_tensor([t, ANALOGUE_HIDDEN], &hidden_values);

        let p1_bindings = [
            InputBinding::new(&p1.token_ids, &token_ids),
            InputBinding::new(&p1.tied_embedding, &tied_embedding),
        ];
        let p3_bindings = [
            InputBinding::new(&p3.hidden, &hidden),
            InputBinding::new(&p3.norm_weight, &norm_weight),
            InputBinding::new(&p3.tied_embedding, &tied_embedding),
        ];

        // Binding identity belongs to the consumer, not to either graph. The
        // exact same Tensor subject is borrowed at both interfaces, while the
        // graph-local input values remain necessarily distinct.
        assert!(std::ptr::eq(
            p1_bindings[1].tensor(),
            p3_bindings[2].tensor()
        ));
        let p1_embedding_value = p1.program.inputs().nth(1).expect("W_embed").value();
        let p3_embedding_value = p3.program.inputs().nth(2).expect("W_embed").value();
        assert_ne!(p1_embedding_value, p3_embedding_value);

        let p1_outputs = evaluator
            .evaluate(&p1.program, &p1_bindings)
            .expect("the bounded gather evaluates");
        let [residual] = p1_outputs.as_slice() else {
            panic!("P1 has one result")
        };
        assert_eq!(payload_bits(residual), expected_embeddings);

        let p3_outputs = evaluator
            .evaluate(&p3.program, &p3_bindings)
            .expect("the bounded normalization and projection evaluate");
        let [logits] = p3_outputs.as_slice() else {
            panic!("P3 has one result")
        };
        // Independent literal oracle: the RMS-normalization worked example
        // [3, 4] with weight [1, 2] is [0x3f593923, 0x4010d0c2]. Projection by
        // e0, e1, e0+e1 yields those two values and their one F32 addition,
        // 0x40471f0b. The zero row remains all positive zero.
        assert_eq!(payload_bits(logits), expected_logits);
    }
}

#[test]
fn the_two_logits_modes_are_distinguished_by_their_declared_output_shape() {
    for t in [10, B1D_PREFILL_POSITIONS] {
        let all_positions = build_projection_program(t, C1_VOCABULARY, C1_HIDDEN);
        let final_position = build_final_position_projection_program(t, C1_VOCABULARY, C1_HIDDEN)
            .expect("the final position of a multi-position prefill is a proper sub-region");

        // Same declared interface. The mode is not a fourth input, not an
        // attribute a caller sets, and not a value read at evaluation time, so
        // there is nothing at this boundary a consumer could switch.
        assert_eq!(
            input_shapes(&final_position.program),
            input_shapes(&all_positions.program)
        );
        assert_eq!(
            input_types(&final_position.program),
            input_types(&all_positions.program)
        );
        assert_eq!(
            final_position
                .program
                .inputs()
                .map(|input| input.key().as_str().to_owned())
                .collect::<Vec<_>>(),
            vec!["h", "w_norm", "W_embed"]
        );

        // Different declared results, which is the whole distinction.
        assert_eq!(
            output_shape(&all_positions.program),
            shape([t, C1_VOCABULARY])
        );
        assert_eq!(
            output_shape(&final_position.program),
            shape([1, C1_VOCABULARY])
        );
        assert_ne!(
            output_shape(&final_position.program),
            output_shape(&all_positions.program),
            "a prefill row must not have to run a program to learn which mode it holds"
        );
        assert_eq!(
            final_position
                .program
                .outputs()
                .next()
                .expect("logits")
                .key()
                .as_str(),
            "logits"
        );

        // The selection is the first occurrence, so the wide logits are never
        // formed. The widening is the unit-axis reindex at every `t`, because
        // the stream carries one position by the time it is reached.
        assert_eq!(
            operation_keys(&final_position.program),
            vec![
                slice_f32_op(),
                reindex_f32_op(),
                rms_norm_f32_op(),
                strict_tensor_contraction_f32_op(),
            ]
        );
        assert_eq!(final_position.program.input_count(), 3);
        assert_eq!(final_position.program.operation_count(), 4);
        assert_eq!(final_position.program.value_count(), 7);
        assert_eq!(final_position.program.output_count(), 1);
        assert_eq!(derived_buffer_actual(&final_position.program), 7);
    }
}

/// The saving the benchmark row buys, derived from the two declared shapes.
///
/// The figures are asserted rather than only computed because the record this
/// ticket inherited carried a 4,096-byte error in each of them, and a written
/// figure is exactly what drifted. Reproduce independently with
/// `8192 * 151936 * 4` and `151936 * 4`.
#[test]
fn the_benchmark_prefill_saving_follows_from_the_two_declared_shapes() {
    let all_positions = build_projection_program(B1D_PREFILL_POSITIONS, C1_VOCABULARY, C1_HIDDEN);
    let final_position =
        build_final_position_projection_program(B1D_PREFILL_POSITIONS, C1_VOCABULARY, C1_HIDDEN)
            .expect("the benchmark prefill row selects one of 8,192 positions");

    let every_position = declared_logit_bytes(&all_positions.program);
    let one_position = declared_logit_bytes(&final_position.program);
    assert_eq!(every_position, 4_978_638_848);
    assert_eq!(one_position, 607_744);
    assert_eq!(every_position - one_position, 4_978_031_104);
    assert!(
        every_position - one_position > C1_F32_WEIGHT_BYTES,
        "the saving is larger than the model's whole F32 weight set"
    );
}

#[test]
fn the_final_position_mode_returns_exactly_the_all_positions_mode_s_last_row() {
    let tied_embedding = f32_tensor(
        [ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN],
        &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let norm_weight = f32_tensor([ANALOGUE_HIDDEN], &[1.0, 2.0]);
    let evaluator = ReferenceEvaluator::standard().expect("the standard evaluator opens");
    let vocabulary = usize::try_from(ANALOGUE_VOCABULARY).expect("a bounded analogue vocabulary");

    // The worked row sits last in the first case and first in the second, so a
    // program that read a fixed position rather than `T - 1` fails one of them
    // whichever fixed position it read.
    for (t, hidden_values, expected_final) in [
        (
            3_u64,
            vec![0.0, 0.0, 0.0, 0.0, 3.0, 4.0],
            vec![0x3f59_3923_u32, 0x4010_d0c2, 0x4047_1f0b],
        ),
        (2, vec![3.0, 4.0, 0.0, 0.0], vec![0_u32, 0, 0]),
    ] {
        let all_positions = build_projection_program(t, ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN);
        let final_position =
            build_final_position_projection_program(t, ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN)
                .expect("a multi-position analogue admits the selection");
        let hidden = f32_tensor([t, ANALOGUE_HIDDEN], &hidden_values);

        let all_outputs = evaluator
            .evaluate(
                &all_positions.program,
                &[
                    InputBinding::new(&all_positions.hidden, &hidden),
                    InputBinding::new(&all_positions.norm_weight, &norm_weight),
                    InputBinding::new(&all_positions.tied_embedding, &tied_embedding),
                ],
            )
            .expect("the all-positions mode evaluates");
        let [all_logits] = all_outputs.as_slice() else {
            panic!("the all-positions mode has one result")
        };
        assert_eq!(*all_logits.shape(), shape([t, ANALOGUE_VOCABULARY]));

        let final_outputs = evaluator
            .evaluate(
                &final_position.program,
                &[
                    InputBinding::new(&final_position.hidden, &hidden),
                    InputBinding::new(&final_position.norm_weight, &norm_weight),
                    InputBinding::new(&final_position.tied_embedding, &tied_embedding),
                ],
            )
            .expect("the final-position mode evaluates");
        let [final_logits] = final_outputs.as_slice() else {
            panic!("the final-position mode has one result")
        };
        assert_eq!(*final_logits.shape(), shape([1, ANALOGUE_VOCABULARY]));

        let all_bits = payload_bits(all_logits);
        let final_bits = payload_bits(final_logits);

        // The independent literal oracle, before any comparison between the two
        // modes: the retained RMS worked example for `[3, 4]` with weight
        // `[1, 2]` is `[0x3f593923, 0x4010d0c2]`, and projecting it through the
        // rows e0, e1, and e0 + e1 yields those two values and their one strict
        // F32 addition, 0x40471f0b. An all-zero row stays positive zero.
        assert_eq!(final_bits, expected_final);

        // And the relation the mode rests on: selecting before normalizing and
        // projecting returns, bit for bit, what projecting every position and
        // reading the last row returns.
        assert_eq!(final_bits, all_bits[all_bits.len() - vocabulary..]);
        assert_ne!(
            all_bits[..vocabulary],
            final_bits[..],
            "the fixture's first and last positions differ, so a fixed-row read cannot pass"
        );
    }
}

/// A decode step has no final position to select, and the family says so.
///
/// At `T = 1` a one-coordinate window covers its axis, which *is* the
/// `whole-axis` relation, so the selection denotes no slice and is refused by
/// name. Nothing is lost: the decode program already declares
/// `[1, vocabulary]`, so the two modes coincide there and the mode distinction
/// is a prefill one. The refusal is what keeps one map from having two
/// spellings.
#[test]
fn the_final_position_mode_is_not_statable_at_a_single_position() {
    let error = build_final_position_projection_program(1, C1_VOCABULARY, C1_HIDDEN)
        .expect_err("selecting the only position restricts no axis");
    let message = error.to_string();
    assert!(
        message.contains("slice.selection.window-is-whole-axis"),
        "{message}"
    );
    assert!(
        message.contains("covers all 1 of its coordinates"),
        "{message}"
    );
    assert_eq!(
        output_shape(&build_projection_program(1, C1_VOCABULARY, C1_HIDDEN).program),
        shape([1, C1_VOCABULARY]),
        "the decode program already declares the one-position result"
    );
}

#[test]
fn an_out_of_range_token_id_reaches_the_exact_named_gather_refusal() {
    let p1 = build_embedding_program(2, ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN);
    let token_ids = u32_tensor([2], &[0, 3]);
    let tied_embedding = f32_tensor(
        [ANALOGUE_VOCABULARY, ANALOGUE_HIDDEN],
        &[1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
    );
    let error = evaluate(
        &p1.program,
        &[
            InputBinding::new(&p1.token_ids, &token_ids),
            InputBinding::new(&p1.tied_embedding, &tied_embedding),
        ],
    )
    .expect_err("an out-of-range token ID must refuse");
    let EvaluationError::Operation {
        operation, source, ..
    } = error
    else {
        panic!("the token-ID refusal must be an operation failure: {error}")
    };
    assert_eq!(operation, gather_f32_op());
    assert_eq!(
        source,
        ReferenceOperationError::GatherIndexOutOfBounds {
            position: 1,
            value: 3,
            extent: ANALOGUE_VOCABULARY,
        }
    );
    assert_eq!(
        source.to_string(),
        "gather index element 1 holds 3 and the gathered axis has extent 3, so it names no \
         coordinate; an out-of-range index is refused rather than clamped to the axis or wrapped \
         modulo its extent"
    );
}
