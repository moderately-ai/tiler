use super::super::{
    Axis, BoundaryRead, CompilationRequest, DeclaredInputOrdinal, F32, InputKey, NormalizedOutput,
    OutputKey, PointwiseF32Node, SemanticMemberId, SemanticStage, SerialSumContributor, Shape,
    TargetProfile, verify_planned_request,
};
use super::support::{contraction_fed_normalization, normalization_program, program, recognize};
use tiler_ir::semantic::{
    F32Add, F32Constant, F32Multiply, SemanticProgramBuilder, StrictSerialF32Sum,
};

/// A registered family whose law realizes a region sequence is a program
/// stage, both as the declared output and as a chain's producer.
///
/// **The recognition is the law's and the partition is the occurrence's, and
/// both halves are asserted.** `tiler::rms-norm-f32@1` reaches this arm
/// because its registered `IndexRealizationLaw` realizes a region *sequence*
/// — no operation key appears in the recognizer — and what the recognized
/// part claims is the occurrence, once, because region formation is the
/// authority that enumerates the stages. `owns_region_members` therefore
/// answers for whichever stage atoms formation minted, which is what lets a
/// cover region covering one stage resolve to this output at all.
///
/// Watched failing under a deliberate perturbation: removing the
/// `laws.family_realizes_region_sequence(operation.key())` disjunct from
/// `plan_elementwise`'s folding discovery refuses the weighted program under
/// `operation-set`, which is the wall this ticket moved.
#[test]
fn a_registered_staged_family_is_recognized_as_a_program_stage() {
    let eps = 1.0e-6_f32.to_bits();

    // The family as the whole declared output.
    let whole = normalization_program(false, eps);
    let NormalizedOutput::Staged(staged) = recognize(&whole).unwrap() else {
        panic!("a family whose law realizes a region sequence is a staged stage")
    };
    assert_eq!(staged.operation, tiler_ir::semantic::rms_norm_f32_op());
    assert_eq!(staged.member, SemanticMemberId(0));
    assert_eq!(
        staged.operand_reads,
        [
            BoundaryRead::Input(DeclaredInputOrdinal::new(0)),
            BoundaryRead::Input(DeclaredInputOrdinal::new(1))
        ]
    );
    assert_eq!(staged.producer, None);
    assert_eq!(staged.output_shape, Shape::from_dims([2, 2]));
    assert_eq!(staged.output_elements, 4);
    assert!(
        !staged.attributes.is_empty(),
        "the occurrence's axis and eps record reaches the recognized shape"
    );

    // The family as a program stage a later pass consumes: the walk names
    // the value the chain materializes and the producer is this shape.
    let weighted = normalization_program(true, eps);
    let NormalizedOutput::Epilogue(chain) = recognize(&weighted).unwrap() else {
        panic!("an elementwise pass over a staged family's result is a chain")
    };
    let NormalizedOutput::Staged(producer) = chain.producer.as_ref() else {
        panic!("the chain's producer is the staged family")
    };
    assert_eq!(producer.member, SemanticMemberId(0));
    assert_eq!(chain.members, [SemanticStage::first(SemanticMemberId(1))]);

    // The partition: the occurrence once, and every region whose atoms are
    // stages of it.
    let output = NormalizedOutput::Staged(producer.clone());
    assert_eq!(
        output.members(),
        [SemanticStage::first(SemanticMemberId(0))]
    );
    let fold = SemanticStage::first(SemanticMemberId(0));
    let pass = fold.next_stage();
    assert!(output.owns_region_members(&[fold]));
    assert!(output.owns_region_members(&[pass]));
    assert!(output.owns_region_members(&[fold, pass]));
    assert!(
        !output.owns_region_members(&[]),
        "an empty member set is no region of this occurrence"
    );
    assert!(
        !output.owns_region_members(&[fold, SemanticStage::first(SemanticMemberId(1))]),
        "a region straddling the consumer belongs to no single part"
    );
}

/// A staged family reading a value another region *computes* refuses by name.
///
/// **This is the neighbour that keeps the widening below attributable, and
/// its rule survives the widening with a narrower meaning.** A multiply's
/// result is no materialization edge — [`materializes_its_result`] is the one
/// statement of where an edge may sit, and it says the expression vocabulary
/// evaluates a multiply per point — so admitting it here would be a second
/// account of that fact, and materializing it would add exactly the
/// observable rounding boundary the caller's program never asked for. Only
/// the operand differs between this program and
/// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`].
///
/// Watched failing under a deliberate perturbation: replacing
/// `materializes_its_result(&root, laws)` with `true` admits the walk to
/// [`recognize_epilogue_producer`], which refuses the same program under
/// `operation-set` — a true statement about the producing family and not
/// about this occurrence's operand, and the reason the guard states the
/// operand rule itself.
#[test]
fn a_staged_family_reading_a_computed_value_refuses_by_name() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let doubled = F32Multiply::apply(&mut builder, value, value).unwrap();
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        doubled,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), normalized)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(recognize(&program).unwrap_err(), "staged-operand");
}

/// A staged family reading a materialized intermediate is recognized, and
/// the operand's boundary role is the recognized shape's.
///
/// **The admission this ticket exists for.** `rms_norm(matmul(a, b), w)`
/// reads its first operand across a materialization edge, which used to be
/// refused under `staged-operand` because nothing in the recognized staged
/// shape could record that operand zero is served by an edge rather than by a
/// declared buffer. Both halves are asserted, because either alone would be
/// consistent with a defect: the operand run names the boundary tensor per
/// operand, and the producer is carried so that the contraction's occurrence
/// is claimed by this output's walk — without which [`check_output_cover`]
/// refuses the program under `operation-set` for an occurrence no walk owns.
///
/// The partition is asserted too, on both sides of the edge, because it is
/// what lets a cover place two regions here: the occurrence's own stages and
/// the contraction's part are all this output's, and a set mixing them is
/// nobody's.
///
/// Watched failing under a deliberate perturbation: dropping the
/// `producer` field from [`NormalizedOutput::members`]'s staged arm — so the
/// walk claims only its own occurrence — refuses this program under
/// `operation-set`, which is exactly the coverage obligation the producer is
/// carried to discharge.
#[test]
fn a_staged_family_reading_a_materialized_intermediate_is_recognized() {
    let program = contraction_fed_normalization(false, false);
    assert_eq!(program.operation_count(), 2);
    let recognized = recognize(&program).expect("the staged operand is recognized");
    let NormalizedOutput::Staged(staged) = &recognized else {
        panic!("a normalization output recognizes as a staged family")
    };
    // The operand's source, carried by the recognized shape: operand zero is
    // the edge and operand one is the independent third declared input.
    assert_eq!(
        staged.operand_reads,
        [
            BoundaryRead::Staged,
            BoundaryRead::Input(DeclaredInputOrdinal::new(2))
        ]
    );
    assert_eq!(staged.member, SemanticMemberId(1));
    // The producer, recognized as the shape a standalone contraction output
    // would be, so every region builder the contraction already has applies
    // to it unchanged.
    let producer = staged
        .producer
        .as_deref()
        .expect("a staged operand carries the shape producing it");
    assert!(producer.contraction().is_some());
    assert_eq!(
        producer.members(),
        [SemanticStage::first(SemanticMemberId(0))]
    );

    // The whole partition: the contraction's part, and the occurrence's own
    // stages. The population is counted, so an assertion about the parts is
    // an assertion about the whole program's occurrences.
    assert_eq!(recognized.members().len(), program.operation_count());
    let fold = SemanticStage::first(SemanticMemberId(1));
    for part in [
        vec![SemanticStage::first(SemanticMemberId(0))],
        vec![fold],
        vec![fold.next_stage()],
    ] {
        assert!(
            recognized.owns_region_members(&part),
            "{part:?} is a part of this output's partition",
        );
    }
    assert!(
        !recognized.owns_region_members(&[SemanticStage::first(SemanticMemberId(0)), fold]),
        "a region straddling the materialization edge is no part",
    );

    // Which declared input each side reads, and at which count. Both are
    // read by the occurrence's own operand run *and* by the producer, and
    // the two agree at `[2, 2]`, so the accessor answers rather than
    // refusing.
    for ordinal in [0, 1, 2] {
        assert!(recognized.reads_declared_input(DeclaredInputOrdinal::new(ordinal)));
        assert_eq!(
            recognized.input_elements_at(DeclaredInputOrdinal::new(ordinal)),
            Some(4),
        );
    }
    assert!(!recognized.reads_declared_input(DeclaredInputOrdinal::new(3)));
    assert_eq!(recognized.max_input_elements(), 4);

    // **The boundary this widening does not move, asserted rather than
    // implied.** The consuming stage would read the occurrence's operand
    // edge *and* the value the producing stage handed it, and
    // `TensorRole::Intermediate` carries no ordinal, so
    // [`crate::physical::staged_plan`] declines the occurrence outright. Its
    // control is the same law over two declared operands, whose plan exists
    // — without which this `None` would be evidence that the plan derivation
    // had stopped working rather than evidence about the edge.
    assert_eq!(crate::physical::staged_plan(staged), None);
    let declared = normalization_program(false, 1.0e-6_f32.to_bits());
    let NormalizedOutput::Staged(control) = recognize(&declared).unwrap() else {
        panic!("a normalization output recognizes as a staged family")
    };
    assert!(crate::physical::staged_plan(&control).is_some());
}

/// The two shapes a staged operand still refuses, each by its own name.
///
/// **Both are asserted rather than left implicit, because one admitted shape
/// reads as general support unless its boundary is stated.** Their admitted
/// neighbour is
/// [`a_staged_family_reading_a_materialized_intermediate_is_recognized`]'s
/// program, which differs from each by exactly the property named:
///
/// - *A second operand supplied by a materialization edge.*
///   `rms_norm(m, m)` gives one occurrence two `TensorRole::Intermediate`
///   reads, and that role carries no ordinal, so nothing says which edge each
///   binds. `staged-operand-conflict`.
/// - *An occurrence already at the far side of an edge reading its own.*
///   `rms_norm(matmul(a, b), w) * w` makes the normalization an epilogue
///   chain's producer, so admitting its operand edge would be a recognized
///   chain two materialization boundaries deep. `staged-operand-depth`, the
///   depth rule's one guard, stated at [`StagedOperandAdmission`].
///
/// Each was watched failing before it was restored: with the
/// `producer.is_some()` guard deleted the first program is recognized with
/// two `BoundaryRead::Staged` operands and one producer, and with the
/// `StagedOperandAdmission::NoEdge` guard deleted the second is recognized as
/// a two-boundary chain — both admissions no region vocabulary here can
/// spell.
///
/// **The second perturbation was rerun on 2026-08-08 and its cost measured**,
/// because "no region vocabulary can spell it" is a claim about a stage this
/// assertion cannot see. Handing `recognize_epilogue_producer`'s call site
/// `OneEdge` recognizes the program as
/// `Epilogue { producer: Staged { producer: Some(Contraction), operand_reads:
/// [Staged, Input(2)] } }` — a well-formed nesting — and this row is the
/// *only* one of the crate's 784 tests that moves. End to end the program
/// then refuses `NoFeasiblePlan` rather than compiling.
/// `crates/tiler-compiler/tests/recognized_chain_depth_boundary.rs` holds
/// that measurement and the trigger that reopens it.
#[test]
fn a_staged_operand_still_refuses_a_second_edge_and_a_deeper_chain() {
    assert_eq!(
        recognize(&contraction_fed_normalization(false, true)).unwrap_err(),
        "staged-operand-conflict",
    );
    assert_eq!(
        recognize(&contraction_fed_normalization(true, false)).unwrap_err(),
        "staged-operand-depth",
    );
}

#[test]
fn governed_request_selects_the_supported_serial_sum_strategy() {
    let program = program();
    let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
    let [recognized] = verified.normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let normalized = recognized.serial_sum();
    assert_eq!(normalized.input_shape, Shape::from_dims([2, 3]));
    assert_eq!(normalized.output_shape, Shape::from_dims([2]));
    assert_eq!(normalized.reduction_axes, [Axis::new(1)]);
    assert_eq!(normalized.input_elements, 6);
    assert_eq!(normalized.output_elements, 2);
    assert_eq!(normalized.input_keys, [InputKey::new("input").unwrap()]);
    // The prologue is the recognized expression, not two constants: it is
    // `input * 2.0 + 1.0` in the physical node vocabulary, and the affine
    // pair the fused region needs is recovered from it rather than stored
    // beside it.
    let prologue = normalized
        .contributor
        .prologue()
        .expect("a fold over a computed contributor has a prologue");
    assert_eq!(prologue.input_count(), 1);
    assert!(matches!(
        prologue.nodes(),
        [
            PointwiseF32Node::Input { .. },
            PointwiseF32Node::Constant { bits: scale },
            PointwiseF32Node::Multiply { .. },
            PointwiseF32Node::Constant { bits: bias },
            PointwiseF32Node::Add { .. },
        ] if *scale == 2.0_f32.to_bits() && *bias == 1.0_f32.to_bits()
    ));
    assert_eq!(
        verified
            .target_slots
            .iter()
            .map(|slot| &slot.target_profile)
            .collect::<Vec<_>>(),
        [&TargetProfile::governed()]
    );
}

/// The composed program: a multi-input elementwise expression feeding a
/// strict serial reduction.
///
/// **This is the shape no normalization matched.** The superseded serial-sum
/// template demanded exactly one declared input and the exact four- or
/// five-operation `x * scale + bias` prologue; the superseded pointwise
/// template refused anything containing a reduction. `sum((a * b) + c)` over
/// three declared inputs is neither, and it is admitted here on the strength
/// of its occurrences: two recognized elementwise families composing into one
/// expression, feeding one recognized reduction.
#[test]
fn a_multi_input_elementwise_expression_feeding_a_reduction_is_recognized() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[1]).unwrap();
    let biased = F32Add::apply(&mut builder, product, inputs[2]).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let program = builder.build().unwrap();

    let NormalizedOutput::SerialSum(recognized) =
        recognize(&program).expect("the composed program is recognized")
    else {
        panic!("a program whose output is a reduction recognizes as one");
    };
    assert_eq!(recognized.input_keys.len(), 3);
    assert_eq!(recognized.input_shape, Shape::from_dims([2, 3]));
    assert_eq!(recognized.output_shape, Shape::from_dims([2]));
    assert_eq!(
        recognized
            .contributor
            .prologue()
            .expect("a fold over a computed contributor has a prologue")
            .input_count(),
        3,
        "one leaf per declared input tensor",
    );
    // Three elementwise occurrences in the prologue is exactly two — the
    // multiply and the add — with no constant, and the reduction is the
    // third occurrence of the program.
    assert_eq!(recognized.members.pointwise().len(), 2);
    assert_eq!(recognized.members.all().len(), program.operation_count());
    // No fused spelling exists: `FusedMultiplyAddSerialSum` applies one
    // scalar constant and one scalar bias, and this prologue applies neither.
    let verified = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    assert_eq!(
        crate::physical::fused_prologue_constants(verified.sole_output()),
        None
    );
}

/// A reduction over a declared input is recognized with no prologue.
///
/// `sum(x)` is the simplest fold there is, and it used to be the one shape
/// this recognizer refused for a wall *below* it: `verify_access_and_semantics`
/// required a `ScalarProgram::StrictSerialSum` region's contributor access to
/// read `TensorRole::Intermediate`, so a region folding the input directly was
/// rejected as malformed. That arm now admits the fold's declared contributor
/// domain, and the absence of a prologue is recorded as `None` rather than as
/// an identity expression — which is what keeps a cover from spelling the copy
/// kernel the refusal existed to avoid.
///
/// Its neighbour is the same fold with one elementwise occurrence between the
/// input and the sum, asserted beside it so the `None` is attributable to the
/// missing prologue rather than to the fold.
#[test]
fn a_reduction_over_a_declared_input_is_recognized_with_no_prologue() {
    let fold = |prologue: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let contributor = if prologue {
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            F32Multiply::apply(&mut builder, input, scale).unwrap()
        } else {
            input
        };
        let sum = StrictSerialF32Sum::apply(&mut builder, contributor, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    };
    let bare = fold(false);
    assert_eq!(bare.operation_count(), 1);
    let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&bare) else {
        panic!("a fold over a declared input is recognized as a serial sum");
    };
    // The source is *named* rather than inferred from absent fields: the arm
    // itself is what says this fold reads a declared input, and it carries the
    // recognized ordinal.
    assert!(matches!(
        recognized.contributor,
        SerialSumContributor::DeclaredInput(ordinal) if ordinal == 0
    ));
    assert_eq!(recognized.contributor.prologue(), None);
    assert_eq!(recognized.contributor.prologue_reads(), []);
    // One part, not two: the empty prologue part is not a member set a cover
    // region may match, which is what `prologue_members` states.
    assert_eq!(recognized.prologue_members(), None);
    assert_eq!(recognized.continuation_members(), None);
    assert_eq!(recognized.members.reduction().len(), 1);

    let neighbour = fold(true);
    let Ok(NormalizedOutput::SerialSum(recognized)) = recognize(&neighbour) else {
        panic!("a fold over a computed contributor is recognized as a serial sum");
    };
    assert!(matches!(
        recognized.contributor,
        SerialSumContributor::PointwisePrologue { .. }
    ));
    assert_eq!(recognized.prologue_members().map(<[_]>::len), Some(2));
    assert_eq!(recognized.continuation_members(), None);
}
