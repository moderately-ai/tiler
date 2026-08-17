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

use tiler_ir::semantic::{BroadcastAxisMapping, BroadcastAxisSource};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32Broadcast, F32Gather, F32Reindex,
    F32RmsNorm, F32TensorContraction, InputKey, OpKey, OutputKey, RMS_NORM_F32_REFERENCE_EPS_BITS,
    ReindexForm, ResolvedValueType, SemanticProgram, SemanticProgramBuilder, broadcast_f32_op,
    gather_f32_op, gather_index_resolved_type, reindex_f32_op, rms_norm_f32_op,
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
