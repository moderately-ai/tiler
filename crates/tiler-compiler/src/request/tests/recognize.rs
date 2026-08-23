use super::super::{
    Axis, AxisDecode, Bf16, BoundaryRead, CompilationRequest, DeclaredInputOrdinal, F32, InputKey,
    LogicalAccess, NormalizedOutput, NormalizedOutputSubject, NormalizedProgram, OutputKey,
    SemanticMemberId, SemanticProgram, SemanticStage, SerialSumContributor, Shape,
    check_output_cover, encode_output_subject, mismatch, output_subject,
    published_and_consumed_overlap, select_supported_strategy, verify_planned_request,
};
use super::support::{contraction_program, laws_of, recognize, recognize_outputs};
use tiler_ir::semantic::{
    Bf16Add, F32Add, F32Constant, F32Multiply, F32RmsNorm, SemanticProgramBuilder,
    StrictSerialF32Sum,
};

/// The two refusals the `dtype-f32` rule split into name different findings.
///
/// **`dtype-recognized` and `dtype-uniform` are not one rule renamed.** The
/// first says this build states no per-point vocabulary for a width the
/// program uses; the second says the program uses two widths at once, which
/// no single scheduled region can carry however well each width is
/// supported. Each is exercised by a program that fails only it, and the
/// admitted neighbours above are what keep the pair from passing for a
/// recognizer that refused everything.
#[test]
fn a_mixed_width_program_and_an_unspelled_width_refuse_by_different_names() {
    // Two recognized widths in one program: the quantized carrier is `bf16`
    // and its declared sibling is `f32`, so no one arithmetic governs it.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let narrow = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let wide = builder
        .input::<F32>(InputKey::new("y").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let narrow_sum = Bf16Add::apply(&mut builder, narrow, narrow).unwrap();
    let wide_sum = F32Add::apply(&mut builder, wide, wide).unwrap();
    builder
        .output(OutputKey::new("narrow").unwrap(), narrow_sum)
        .unwrap();
    builder
        .output(OutputKey::new("wide").unwrap(), wide_sum)
        .unwrap();
    let mixed = builder.build().unwrap();
    assert_eq!(
        recognize(&mixed),
        Err("dtype-uniform"),
        "a program of two widths has no single scalar program",
    );

    // One width this build spells no per-point body in: the strict-affine
    // encoded carrier, a registered value type that names no arithmetic type
    // at all.
    let published = |program: &SemanticProgram| recognize(program);
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let codes = builder
        .input::<tiler_ir::semantic::StrictAffineU4>(
            InputKey::new("codes").unwrap(),
            Shape::from_dims([2, 3]),
        )
        .unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), codes)
        .unwrap();
    let encoded = builder.build().unwrap();
    assert_eq!(
        published(&encoded),
        Err("dtype-recognized"),
        "a value type this build states no per-point vocabulary for is named as such",
    );

    // The neighbour that attributes that refusal to the *width* rather than
    // to the shape: the same program in a recognized width publishes a
    // declared input, which is refused one rule later under `operation-set`.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let value = builder
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), value)
        .unwrap();
    let published_input = builder.build().unwrap();
    assert_eq!(
        published(&published_input),
        Err("operation-set"),
        "the shape alone refuses under its own rule, so the width is what the U4 program \
         was refused for",
    );
}

/// Every refusal names the exact property that was not recognized.
///
/// The table is the ticket's contract: recognition generalizes, admission
/// does not become silent. Each row is a program the boundary refuses, the
/// rule it refuses under, and — through the accepted neighbour built beside
/// it — a demonstration that the rule can say yes as well as no.
#[test]
fn every_refusal_names_its_unrecognized_property() {
    let shape = || Shape::from_dims([2, 3]);

    // `input-arity`: an all-constant graph has no output-reachable input,
    // and a frozen program drops the unused declaration. The neighbour is
    // the same expression with one leaf replaced by the declared tensor.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let _input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let first = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let second = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let root = F32Add::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let all_constant = builder.build().unwrap();
    assert_eq!(all_constant.input_count(), 0);
    assert_eq!(recognize(&all_constant).unwrap_err(), "input-arity");

    // `output-partition-overlap`: two named outputs one walk would have to
    // publish, because the second names a value the first's walk consumes.
    // The neighbour is the same graph naming only the root, which recognizes
    // — so the rule reads the *sharing* rather than the second output. This
    // row replaced an `output-arity` row: the arity guard is gone, and what
    // refuses this program is the partition obligation it actually violates.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, constant).unwrap();
    let root = F32Add::apply(&mut builder, scaled, constant).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    builder
        .output(OutputKey::new("partial").unwrap(), scaled)
        .unwrap();
    let two_outputs = builder.build().unwrap();
    assert_eq!(two_outputs.output_count(), 2);
    assert_eq!(
        recognize(&two_outputs).unwrap_err(),
        "output-partition-overlap",
    );

    // **Admitted, and this row is the one the structural widening flipped.**
    // A transposition over a declared input becomes the *read map* of the
    // region that consumes it, so `tiler::reindex-f32@1` is recognized
    // rather than refused. The derived relation is asserted rather than
    // merely the admission: a recognizer that admitted the family and bound
    // a dense read would compile the wrong tensor, which is precisely the
    // failure a bare `is_ok()` here would not see.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let permuted = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("a two-axis transposition is an admitted form"),
        input,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), permuted)
        .unwrap();
    let structural = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&structural).expect("a transposition of a declared input is a mapped read")
    else {
        panic!("a reindex over a declared input is an elementwise region");
    };
    // `shape()` is `[2, 3]`, so the transposed result is `[3, 2]` with
    // suffix products `[2, 1]`. Operand axis 1 takes result axis 0's window
    // and operand axis 0 takes result axis 1's, which is the transposition
    // written as a decode per operand axis.
    assert_eq!(
        recognized.reads,
        vec![(
            DeclaredInputOrdinal::new(0),
            LogicalAccess::ReindexBijection {
                operand_shape: Shape::from_dims([2, 3]),
                result_shape: Shape::from_dims([3, 2]),
                axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)],
            },
        )],
    );

    // `structural-operand`: the family is admitted, and what is refused is a
    // structural occurrence over a *computed* value. The region binds one
    // read per declared input and has no access to bind an intermediate it
    // also produces, so this refuses by name rather than materializing the
    // intermediate — which would add exactly the observable rounding
    // boundary the family's admission excludes. It is the neighbour that
    // keeps the row above attributable: both are reindexes, and only the
    // operand differs.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let doubled = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
        .expect("the standard registry admits the silu family");
    let permuted = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("a two-axis transposition is an admitted form"),
        doubled,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), permuted)
        .unwrap();
    let computed = builder.build().unwrap();
    assert_eq!(recognize(&computed).unwrap_err(), "structural-operand");

    // **Admitted, and this row moved here from the refusal inventory.** One
    // declared input read *both* densely and through a relation was refused
    // under `structural-access-conflict`, because the region bound one read
    // per declared input and the expression's two `Input { ordinal: 0 }`
    // nodes shared it — so the mapped relation served both leaves and
    // `a * permute(a)` over `[[1, 2], [4, 8]]` compiled to `[1, 16, 4, 64]`,
    // which is `permute(a) * permute(a)`, where the reference evaluator
    // gives `[1, 8, 8, 64]`. The region now binds two reads of ordinal `0`,
    // and the read list is asserted rather than the admission: a recognizer
    // that admitted the program and bound one read would compile exactly the
    // wrong tensor that a bare `is_ok()` would not see.
    //
    // What still refuses is the pair with no canonical order between its two
    // members — two *structural* relations on one input — which is the
    // neighbour that keeps the admission attributable.
    let mixed = |second_dense: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let reindex = |builder: &mut SemanticProgramBuilder,
                       form: &tiler_ir::semantic::ReindexForm| {
            tiler_ir::semantic::F32Reindex::apply(builder, form, a)
                .expect("the standard registry admits the reindex family")
        };
        let transposed = reindex(
            &mut builder,
            &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
                .expect("a two-axis transposition is an admitted form"),
        );
        let second = if second_dense {
            a
        } else {
            reindex(
                &mut builder,
                &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                    .expect("an axis reversal is an admitted form"),
            )
        };
        let root = F32Multiply::apply(&mut builder, second, transposed).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    let NormalizedOutput::Pointwise(recognized) = recognize(&mixed(true))
        .expect("one declared input may be read densely and through a relation")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    // The dense read leads and the mapped one follows, which is the pair's
    // canonical order and the only one the region verifier admits.
    assert_eq!(
        recognized.reads,
        vec![
            (DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity),
            (
                DeclaredInputOrdinal::new(0),
                LogicalAccess::ReindexBijection {
                    operand_shape: Shape::from_dims([2, 2]),
                    result_shape: Shape::from_dims([2, 2]),
                    axes: vec![AxisDecode::read(1, 2), AxisDecode::read(2, 2)],
                },
            ),
        ],
    );
    assert_eq!(recognized.expression.f32().input_count(), 2);
    assert_eq!(
        recognize(&mixed(false)).unwrap_err(),
        "structural-access-conflict",
    );

    // `structural-access-conflict` again, and this is the *other* half of the
    // widening's boundary: the twice-read tensor is the value an earlier
    // region staged rather than a declared input. What admits the pair above
    // is the ordinal saying which tensor each read binds, and
    // `TensorRole::Intermediate` carries none — so a second staged read has
    // nothing to attribute it to a second materialization edge. Its accepted
    // neighbour is `s * s`, which reads the staged value once and differs by
    // exactly the read that would have no attribution.
    let staged = |mapped: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let folded = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
        let second = if mapped {
            tiler_ir::semantic::F32Reindex::apply(
                &mut builder,
                &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(0))
                    .expect("an axis reversal is an admitted form"),
                folded,
            )
            .expect("the standard registry admits the reindex family")
        } else {
            folded
        };
        let root = F32Multiply::apply(&mut builder, folded, second).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };
    assert!(matches!(
        recognize(&staged(false)),
        Ok(NormalizedOutput::Epilogue(_)),
    ));
    assert_eq!(
        recognize(&staged(true)).unwrap_err(),
        "structural-access-conflict",
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape())
        .unwrap();
    let activated = tiler_ir::semantic::F32Silu::apply(&mut builder, input)
        .expect("the standard registry admits the silu family");
    builder
        .output(OutputKey::new("result").unwrap(), activated)
        .unwrap();
    let unary = builder.build().unwrap();
    let NormalizedOutput::Pointwise(recognized) =
        recognize(&unary).expect("the activation projects into the expression vocabulary")
    else {
        panic!("an elementwise output recognizes as an elementwise program");
    };
    // One occurrence, one declared input read once, and the composition's
    // seven nodes: the projection is the shared body's, not a per-shape one.
    assert_eq!(
        recognized.members,
        vec![SemanticStage::first(SemanticMemberId(0))]
    );
    assert_eq!(recognized.expression.f32().input_count(), 1);
    assert_eq!(recognized.expression.f32().nodes().len(), 7);

    // A contraction with a reachable elementwise epilogue is a *chain*, not
    // a refusal, and the bare contraction beside it is what makes the
    // difference attributable: the two programs differ by exactly the
    // epilogue, and the recognized shape differs by exactly the consumer
    // region.
    let contraction = contraction_program(false);
    assert!(matches!(
        recognize(&contraction),
        Ok(NormalizedOutput::Contraction(_))
    ));
    let with_epilogue = contraction_program(true);
    let Ok(NormalizedOutput::Epilogue(chain)) = recognize(&with_epilogue) else {
        panic!("an elementwise expression over a contraction result is a chain");
    };
    assert!(matches!(*chain.producer, NormalizedOutput::Contraction(_)));
    assert_eq!(
        chain.reads.len(),
        1,
        "the epilogue reads only the staged value"
    );
    assert_eq!(chain.reads[0].0, BoundaryRead::Staged);

    // The one side the discovery used to refuse: a fold whose *contributors*
    // cross a materialization boundary. The producer was already recognized;
    // what was missing was a place on `NormalizedSerialSum` to retain it, and
    // the contributor source is that place — so `sum(sum(x) * 2)` is now the
    // admitted subject rather than the wall.
    //
    // The declared-input neighbour is the same fold over the same scaling of
    // the *declared input*, so the difference between them is exactly where the
    // scaled value comes from — which is what the contributor source names.
    let folded_prologue = |nested: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let (contributors, axis) = if nested {
            let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
            (
                F32Multiply::apply(&mut builder, inner, scale).unwrap(),
                Axis::new(0),
            )
        } else {
            (
                F32Multiply::apply(&mut builder, input, scale).unwrap(),
                Axis::new(1),
            )
        };
        let outer = StrictSerialF32Sum::apply(&mut builder, contributors, [axis]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), outer)
            .unwrap();
        builder.build().unwrap()
    };
    let Ok(NormalizedOutput::SerialSum(declared)) = recognize(&folded_prologue(false)) else {
        panic!("a fold over a pointwise prologue is recognized as a serial sum");
    };
    assert!(matches!(
        declared.contributor,
        SerialSumContributor::PointwisePrologue { .. }
    ));
    let Ok(NormalizedOutput::SerialSum(produced)) = recognize(&folded_prologue(true)) else {
        panic!("a fold over a materialized producer is recognized as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &produced.contributor else {
        panic!("a fold over a nested reduction names a materialized contributor");
    };
    assert!(matches!(
        materialized.producer,
        NormalizedOutput::SerialSum(_)
    ));
    let continuation = materialized
        .continuation
        .as_ref()
        .expect("the `* 2` between the producer and the fold is a continuation");
    assert_eq!(
        continuation
            .reads
            .iter()
            .filter(|(read, _)| *read == BoundaryRead::Staged)
            .count(),
        1,
        "the continuation reads exactly the value the producer staged",
    );

    // The source names the contributor relation rather than the producer
    // family: a staged family reaches the same arm a nested reduction does,
    // and — because the fold's contributor *is* the produced value — with no
    // continuation rather than a synthesized identity one.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let normalized = F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, normalized, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), reduced)
        .unwrap();
    let staged_contributor = builder.build().unwrap();
    let Ok(NormalizedOutput::SerialSum(staged)) = recognize(&staged_contributor) else {
        panic!("a fold over a staged family is recognized as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &staged.contributor else {
        panic!("a fold over a staged family names a materialized contributor");
    };
    assert!(matches!(materialized.producer, NormalizedOutput::Staged(_)));
    assert_eq!(
        materialized.continuation, None,
        "the fold's contributor *is* the produced value, so no region stands between them",
    );

    // `reduction-contributor-depth`: the same shape one materialization
    // boundary deeper, where the fold's producer is itself a fold across an
    // edge. The rule names how deep the chain runs rather than a carrier the
    // normal form lacks, and it is the sides rule that reports it — the
    // producer is recognized through `recognize_epilogue_producer`, which
    // hands `NoEdge`.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2, 2]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(2)]).unwrap();
    let scaled = F32Multiply::apply(&mut builder, inner, scale).unwrap();
    let middle = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    let rescaled = F32Multiply::apply(&mut builder, middle, scale).unwrap();
    let outer = StrictSerialF32Sum::apply(&mut builder, rescaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), outer)
        .unwrap();
    let too_deep = builder.build().unwrap();
    assert_eq!(
        recognize(&too_deep).unwrap_err(),
        "reduction-contributor-depth"
    );

    // Width, not depth, and it keeps its own rule: a contributor walk reaching
    // a *second, different* materialized value has nothing to say which edge
    // each read binds, so it reports `operation-set` after the retain rather
    // than taking the first fold and dropping the second.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let left = StrictSerialF32Sum::apply(&mut builder, first, [Axis::new(1)]).unwrap();
    let right = StrictSerialF32Sum::apply(&mut builder, second, [Axis::new(1)]).unwrap();
    let paired = F32Multiply::apply(&mut builder, left, right).unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, paired, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    let two_edges = builder.build().unwrap();
    assert_eq!(recognize(&two_edges).unwrap_err(), "operation-set");
}

/// Two ordered named outputs whose producers share no occurrence.
///
/// `product = a * b` and `sum = a + b` over the same two declared inputs.
/// The independence is the point: neither output's walk reaches the other's
/// producer, which is exactly the branch the superseded single-output
/// recognition refused under `operation-set` — one walk covered one of the
/// two operations and the program had two.
fn independent_two_output_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, first, second).unwrap();
    let sum = F32Add::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder.output(OutputKey::new("sum").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// Recognition names one implementable region partition per ordered output.
///
/// **The wall this ticket was filed for, observed gone.** The recognition
/// used to read one output, classify it, and require that one walk to cover
/// the program; a second declared output outside the walk therefore refused
/// under `operation-set`, which is what the measurement at `3adc0689`
/// recorded when both arity guards were relaxed. The same program now
/// recognizes into two partitions, each naming its own output key,
/// expression, and members — and the members are disjoint, which is what
/// makes each one a region a cover can place without two regions claiming
/// one occurrence.
///
/// The whole boundary is asserted beside the walk, because the two together
/// are what the claim needs: the same program recognizes into two partitions
/// *and* clears [`select_supported_strategy`], which used to refuse it under
/// `output-arity` before any occurrence was classified. That guard is gone,
/// so the two derivations now agree rather than contradicting each other.
#[test]
fn recognizing_several_ordered_named_outputs_names_one_partition_each() {
    let program = independent_two_output_program();
    assert_eq!(program.output_count(), 2);
    assert_eq!(program.operation_count(), 2);

    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let [product, sum] = recognized.outputs() else {
        panic!("one recognized partition per declared output, in declaration order");
    };
    let product = product
        .pointwise()
        .expect("a multiply is an elementwise output");
    let sum = sum.pointwise().expect("an add is an elementwise output");
    assert_eq!(product.output_key, OutputKey::new("product").unwrap());
    assert_eq!(sum.output_key, OutputKey::new("sum").unwrap());
    // Each walk claims exactly its own producer, and the two sets are
    // disjoint: together they partition the program's occurrences.
    assert_eq!(product.members.len(), 1);
    assert_eq!(sum.members.len(), 1);
    assert_ne!(product.members, sum.members);
    assert_eq!(recognized.all_members().len(), program.operation_count());
    // Two different binary32 functions over the same two reads, so the
    // partitions are distinguished by what they compute and not only by
    // which occurrence they name.
    assert_ne!(product.expression, sum.expression);

    // The same recognition reached through the ordinary boundary, which is
    // where the arity guard stood. Compared by the same fields rather than
    // by whole-value equality, for the reason
    // `two_programs_differing_only_in_output_order_recognize_differently`
    // gives about `ValueId` carrying its graph.
    let admitted =
        select_supported_strategy(&program, &laws_of(&program)).expect("the boundary admits it");
    assert_eq!(
        admitted
            .outputs()
            .iter()
            .map(NormalizedOutput::members)
            .collect::<Vec<_>>(),
        vec![product.members.clone(), sum.members.clone()],
    );
}

/// A cover region resolves to the output whose partition owns it.
///
/// This is the lookup `crate::physical::spell_region` performs, exercised
/// on the one shape that can distinguish it from the whole-program question
/// it replaced: with two declared outputs, "which expression does this
/// region compute" has two answers and the members are what choose between
/// them. The straddling case is the one that must say no — a region covering
/// both outputs' occurrences computes two published results from one owning
/// write, and no scheduled region does that.
#[test]
fn a_region_resolves_to_the_output_whose_partition_owns_it() {
    let program = independent_two_output_program();
    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let [first, second] = recognized.outputs() else {
        panic!("one recognized partition per declared output");
    };
    let first_members = first.members();
    let second_members = second.members();

    assert_eq!(
        recognized
            .output_for_region(&first_members)
            .map(|(at, _)| at),
        Some(0),
    );
    assert_eq!(
        recognized
            .output_for_region(&second_members)
            .map(|(at, _)| at),
        Some(1),
    );
    // The check can say no, in both of the ways a cover can get it wrong: a
    // region straddling the two partitions, and a region covering neither.
    let straddling = recognized.all_members();
    assert_eq!(straddling.len(), 2);
    assert!(recognized.output_for_region(&straddling).is_none());
    assert!(recognized.output_for_region(&[]).is_none());
}

/// The whole-program cover check was widened, not removed, and says no.
///
/// **Both arms are driven against a case that must fail.** The accepted
/// neighbour is the recognized two-output partition itself; each perturbation
/// takes exactly one property away from it.
///
/// *Removal-shaped.* Dropping one occurrence from a walk leaves an
/// occurrence no output claims, which is work the assembled program would
/// silently not compute. Removing the check rather than widening it is
/// exactly what would admit this, so the perturbation is the removal.
///
/// *Overlap-shaped.* Adding one walk's occurrence to another's makes the two
/// partitions claim it twice, which is the shape where one region's owning
/// write would have to serve both a materialization edge and a publication.
#[test]
fn the_output_partition_check_can_say_no_in_both_directions() {
    let program = independent_two_output_program();
    let recognized = recognize_outputs(&program).expect("both outputs are recognized");
    let outputs = recognized.outputs().to_vec();
    // The control: unperturbed, the walks partition the occurrences.
    assert_eq!(check_output_cover(&program, &outputs), Ok(()));

    let mut uncovered = outputs.clone();
    let NormalizedOutput::Pointwise(dropped) = &mut uncovered[1] else {
        panic!("the fixture's second output is elementwise");
    };
    dropped.members.clear();
    assert_eq!(
        check_output_cover(&program, &uncovered),
        mismatch("operation-set"),
        "an occurrence covered by no walk was admitted",
    );

    let mut overlapping = outputs.clone();
    let claimed = outputs[0].members();
    let NormalizedOutput::Pointwise(widened) = &mut overlapping[1] else {
        panic!("the fixture's second output is elementwise");
    };
    widened.members.extend_from_slice(&claimed);
    widened.members.sort_unstable();
    assert_eq!(
        check_output_cover(&program, &overlapping),
        mismatch("output-partition-overlap"),
        "one occurrence claimed by two walks was admitted",
    );
}

/// Two output keys naming one value still refuse under the partition rule.
///
/// **This is the neighbour of the admitted overlap, and it differs from it
/// by exactly the property [`published_and_consumed_overlap`] requires.**
/// Three shapes, each observed refusing under the partition rule rather than
/// being admitted and dropped a layer down:
///
/// - Two output keys naming *one* value. The two walks are equal rather than
///   one being a strict subset of the other, so there is no shorter walk to
///   publish and no boundary to publish at. Whichever region owns that
///   value's write publishes once, and
///   `tiler_ir::program::KernelProgramBuilder` refuses a second publication
///   of one buffer.
/// - A publication *inside* one recognized part. `product` is consumed by
///   the add that `biased` names, and a pointwise walk fusing the multiply
///   and the add has no region boundary between them — the subset is not a
///   *part*, which is the conjunct `owns_region_members` decides.
/// - A published value nothing outside the part reads. This one is stated
///   against [`published_and_consumed_overlap`] directly rather than as a
///   program, and that is a fact worth recording rather than a convenience:
///   for every program the recognizer admits, the value a part publishes
///   *is* the value crossing its boundary, so the conjunct is defence in
///   depth against a future recognizer rather than a live gate. Stating the
///   member sets is what makes it drivable at all.
///
/// Their admitted neighbour is the published-and-consumed program that
/// `crate::pipeline::conformance`'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` compiles,
/// which differs from each by exactly one of those conjuncts.
#[test]
fn an_output_key_pair_naming_one_value_still_refuses_by_name() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let first = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let second = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, first, second).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("alias").unwrap(), product)
        .unwrap();
    let colliding = builder.build().unwrap();
    assert_eq!(colliding.output_count(), 2);
    assert_eq!(colliding.operation_count(), 1);
    assert_eq!(
        recognize_outputs(&colliding).unwrap_err(),
        "output-partition-overlap",
    );

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let other = builder
        .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, input, other).unwrap();
    let biased = F32Add::apply(&mut builder, product, other).unwrap();
    builder
        .output(OutputKey::new("biased").unwrap(), biased)
        .unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    let mid_walk = builder.build().unwrap();
    assert_eq!(
        recognize_outputs(&mid_walk).unwrap_err(),
        "output-partition-overlap",
    );

    // The admitted neighbour, at this same boundary: `scaled` is a strict
    // subset of the fold's walk, is exactly its recognized prologue part,
    // and is the value the fold reads across the boundary.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let published_and_consumed = builder.build().unwrap();
    let recognized = recognize_outputs(&published_and_consumed).expect("the overlap is admitted");
    let claimed: Vec<Vec<SemanticStage>> = recognized
        .outputs()
        .iter()
        .map(NormalizedOutput::members)
        .collect();
    assert_eq!(
        published_and_consumed_overlap(&published_and_consumed, recognized.outputs(), &claimed),
        Some((1, 0)),
    );

    // The crossing conjunct, driven against a stated member set: the shorter
    // walk is the fold's *reduction* part rather than its prologue part — a
    // part in its own right, and still a strict subset — but the value the
    // second output publishes is the multiply's, which no occurrence outside
    // that part reads. Every other conjunct is unchanged.
    let reduction_part = vec![claimed[0].last().copied().expect("the fold claims members")];
    assert_eq!(
        published_and_consumed_overlap(
            &published_and_consumed,
            recognized.outputs(),
            &[claimed[0].clone(), reduction_part],
        ),
        None,
    );
}

/// Two declared inputs and one expression naming both of the outer ones.
///
/// `product = a * c` and `doubled = b + b` over three declared `[2, 3]`
/// inputs. The first walk reads ordinals `0` and `2`, which is deliberately
/// not a prefix and not contiguous: a region-local renumbering would give
/// its two leaves reads `0` and `1` and the assembled program would multiply
/// `a * b`, and every other recognized fact would agree.
fn non_contiguous_subset_program(outer: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                .unwrap()
        })
        .collect();
    let (paired, doubled) = if outer { (2, 1) } else { (1, 2) };
    let product = F32Multiply::apply(&mut builder, inputs[0], inputs[paired]).unwrap();
    let sum = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("doubled").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// A walk reading a subset carries the program's ordinals, not its own.
///
/// **The read list is the map this ticket asked for.** `mint_elementwise`
/// numbers the expression's leaves by position in the canonical read order,
/// and the read at that position names the declared input ordinal it binds,
/// so `reads` *is* the leaf-ordinal-to-input-ordinal correspondence and
/// nothing further had to be carried. What changed is that it is no longer
/// the identity on `0..declared`.
///
/// The neighbour swaps which of the two later inputs each output reads, so
/// the recognized ordinals move with the program while the expression, the
/// declared keys, the domain, and the member sets all stay put — which is
/// what makes the assertion about the read list rather than about the
/// program being recognized at all.
#[test]
fn a_walk_reading_a_subset_carries_the_program_input_ordinals_it_reached() {
    for (outer, expected) in [(true, 2_u32), (false, 1)] {
        let program = non_contiguous_subset_program(outer);
        assert_eq!(program.input_count(), 3);
        let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
        let [product, doubled] = recognized.outputs() else {
            panic!("the fixture declares two outputs");
        };
        let NormalizedOutput::Pointwise(product) = product else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        let NormalizedOutput::Pointwise(doubled) = doubled else {
            panic!("an elementwise output recognizes as an elementwise program");
        };
        // The declared interface stays whole: the ordinals index it, so a
        // region reading two of three inputs still resolves against all
        // three at assembly.
        assert_eq!(product.input_keys.len(), 3);
        assert_eq!(
            product.reads,
            vec![
                (DeclaredInputOrdinal::new(0), LogicalAccess::LinearIdentity),
                (
                    DeclaredInputOrdinal::new(expected),
                    LogicalAccess::LinearIdentity
                ),
            ],
        );
        assert_eq!(product.expression.f32().input_count(), 2);
        // The other output reads the remaining input at one leaf, twice.
        let other = if outer { 1 } else { 2 };
        assert_eq!(
            doubled.reads,
            vec![(
                DeclaredInputOrdinal::new(other),
                LogicalAccess::LinearIdentity
            )]
        );
        assert_eq!(doubled.expression.f32().input_count(), 1);
    }
}

/// A declared input no output reads is refused at program scope.
///
/// **The removal-shaped perturbation, and it has to be forged.** The
/// obligation `canonical_input_reads` used to state per walk moved to
/// [`check_output_cover`], and no program the public builder can construct
/// reaches it: a frozen program retains only output-reachable values, the
/// `operation-set` rule claims every retained occurrence for some walk, and
/// every way a walk consumes an operand records a read of it. So the check
/// is driven against a recognized program whose read list has had one entry
/// removed — which is exactly the state deleting the check would admit —
/// and its unforged neighbour is asserted to pass, so a check that refused
/// everything would fail here too.
#[test]
fn a_declared_input_no_output_reads_is_refused_at_program_scope() {
    let program = non_contiguous_subset_program(true);
    let recognized = recognize_outputs(&program).expect("a subset walk is recognized");
    assert_eq!(check_output_cover(&program, recognized.outputs()), Ok(()));

    let mut forged = recognized.clone();
    let NormalizedOutput::Pointwise(product) = &mut forged.outputs[0] else {
        panic!("the first declared output is elementwise");
    };
    product.reads.retain(|(ordinal, _)| *ordinal != 2);
    assert_eq!(
        check_output_cover(&program, &forged.outputs),
        mismatch("input-set"),
    );
}

/// A fold retains whichever declared input its contributor names.
///
/// The two programs have the same declaration, output families, shapes, and
/// operation order. The contributor ordinal is the relevant difference, and
/// it reaches both normalization and the output-subject bytes rather than
/// being renumbered to the fold region's only read.
#[test]
fn a_fold_over_a_later_declared_input_retains_its_ordinal() {
    let folded = |first: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let inputs: Vec<_> = ["a", "b"]
            .into_iter()
            .map(|key| {
                builder
                    .input::<F32>(InputKey::new(key).unwrap(), Shape::from_dims([2, 3]))
                    .unwrap()
            })
            .collect();
        let (folded, doubled) = if first { (0, 1) } else { (1, 0) };
        let sum = StrictSerialF32Sum::apply(&mut builder, inputs[folded], [Axis::new(1)]).unwrap();
        let pair = F32Add::apply(&mut builder, inputs[doubled], inputs[doubled]).unwrap();
        builder
            .output(OutputKey::new("folded").unwrap(), sum)
            .unwrap();
        builder
            .output(OutputKey::new("doubled").unwrap(), pair)
            .unwrap();
        builder.build().unwrap()
    };
    let recognized = [
        recognize_outputs(&folded(true)).expect("a fold over input zero"),
        recognize_outputs(&folded(false)).expect("a fold over input one"),
    ];
    let mut encoded = Vec::new();
    for (ordinal, outputs) in recognized.iter().enumerate() {
        let [normalized, _] = outputs.outputs() else {
            panic!("the fixture declares two outputs");
        };
        let NormalizedOutput::SerialSum(fold) = normalized else {
            panic!("a reduction output recognizes as a serial sum");
        };
        assert_eq!(
            fold.contributor,
            SerialSumContributor::DeclaredInput(DeclaredInputOrdinal::new(
                u32::try_from(ordinal).unwrap()
            ))
        );

        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(normalized));
        encoded.push(bytes);
    }
    assert_ne!(encoded[0], encoded[1]);
}

/// Both claimants of a published-and-consumed part resolve to one region.
///
/// **This is the check behind the decided tie-break.**
/// [`NormalizedProgram::output_for_region`] scans in declaration order and
/// takes the first match, and the admitted overlap makes two outputs own one
/// member set — so "first" is only correct because the two claimants are
/// recognitions of one value over one occurrence set and therefore spell the
/// same region. That argument is worth less than a check that says no when
/// it stops holding, which is what this is: the same member set is resolved
/// against each claimant in turn, and the two regions the physical layer
/// builds from those resolutions are compared whole.
///
/// The two spellings are reached through different arms — the fold's
/// prologue part and the pointwise output's own walk — so an agreement here
/// is about the recognitions rather than about one code path being called
/// twice.
#[test]
fn both_claimants_of_a_published_and_consumed_part_spell_one_region() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let reduced = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("reduced").unwrap(), reduced)
        .unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let program = builder.build().unwrap();
    let recognized = recognize_outputs(&program).expect("the overlap is admitted");
    let [fold, publication] = recognized.outputs() else {
        panic!("one recognized partition per declared output");
    };
    let shared = publication.members();

    // Both own it, which is the state the tie-break exists for.
    assert!(fold.owns_region_members(&shared));
    assert!(publication.owns_region_members(&shared));
    assert_eq!(
        recognized.output_for_region(&shared).map(|(at, _)| at),
        Some(0),
        "the first declared claimant is the one the scan returns",
    );

    // And they spell one region. Compared through the request the physical
    // layer actually reads, at the write the cover assigns a published-and-
    // consumed region.
    let request = verify_planned_request(CompilationRequest::governed(&program))
        .unwrap()
        .for_target(0)
        .unwrap();
    let staging = crate::physical::RegionWrite::MaterializedAndPublished;
    let (from_fold, fold_members) = crate::physical::pointwise_region(&request, fold, staging);
    let (from_publication, publication_members) =
        crate::physical::pointwise_region(&request, publication, staging);
    assert_eq!(from_fold, from_publication);
    assert_eq!(fold_members, publication_members);
    assert_eq!(fold_members, shared);
}

/// Output order reaches the recognized program, not only the semantic graph.
///
/// Two programs holding the same operations and the same two output keys,
/// differing only in which `output()` call came first, recognize into lists
/// that are unequal *and* unequal in order — the first entry of one is the
/// second entry of the other. The request subject encodes that list
/// length-framed in this order, so a permuted declaration cannot reach one
/// subject; the semantic half of the same claim is pinned in
/// `crates/tiler-compiler/tests/multi_output_boundary.rs`.
///
/// **The subject half is asserted here too, and it was not reachable until
/// `output-arity` was relaxed:** a subject is minted only for a request the
/// boundary admitted, and that guard admitted no two-output program at all.
/// Both orders now mint one, and the two subjects name their outputs in the
/// order their programs declared them.
///
/// **Measurement boundary, and it is a limit on what any test here can
/// claim.** The subject's *output list* is compared against the program's
/// declared keys, not its canonical bytes. The previous version of this
/// comment predicted the encoded form would become checkable once the guard
/// moved, and it does not: the subject folds the semantic graph identity,
/// output order is already part of that identity, and no two programs can
/// differ *only* in the recognized list — so two subjects' bytes differ
/// whatever the list order, observed by sorting the arms in
/// [`VerifiedRequestSubject::canonical_explain_subject_bytes`] and watching
/// the inequality still hold. A check that cannot say no is not evidence.
/// The list comparison is anchored to the declared keys for the same reason:
/// comparing the two subjects only to each other survives a list reversed
/// for both, which was also observed.
///
/// The recognized entries are compared by the fields the subject encodes
/// rather than by the whole recognized value, because a [`ValueId`] carries
/// the graph it was built in: two separately built programs never share one,
/// so whole-value equality would report a difference this test is not about
/// and would hold whatever the order.
#[test]
fn two_programs_differing_only_in_output_order_recognize_differently() {
    fn ordered(product_first: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let second = builder
            .input::<F32>(InputKey::new("b").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, first, second).unwrap();
        let sum = F32Add::apply(&mut builder, first, second).unwrap();
        let product_key = OutputKey::new("product").unwrap();
        let sum_key = OutputKey::new("sum").unwrap();
        if product_first {
            builder.output(product_key, product).unwrap();
            builder.output(sum_key, sum).unwrap();
        } else {
            builder.output(sum_key, sum).unwrap();
            builder.output(product_key, product).unwrap();
        }
        builder.build().unwrap()
    }

    /// The per-output facts the request subject encodes, in list order.
    fn encoded(recognized: &NormalizedProgram) -> Vec<(OutputKey, Vec<SemanticStage>)> {
        recognized
            .outputs()
            .iter()
            .map(|output| {
                let pointwise = output.pointwise().expect("an elementwise output");
                (pointwise.output_key.clone(), pointwise.members.clone())
            })
            .collect()
    }

    let product_first = encoded(&recognize_outputs(&ordered(true)).expect("recognized"));
    let sum_first = encoded(&recognize_outputs(&ordered(false)).expect("recognized"));
    assert_ne!(
        product_first, sum_first,
        "output order must reach the recognized program, not only presentation",
    );
    assert_eq!(product_first[0], sum_first[1]);
    assert_eq!(product_first[1], sum_first[0]);
    // The check can say no: re-declaring the same order reproduces the
    // recognition, so the inequality above is about the order and not about
    // rebuilding the program.
    assert_eq!(
        product_first,
        encoded(&recognize_outputs(&ordered(true)).expect("recognized")),
    );

    // The same claim about the *subject*, minted through the ordinary
    // boundary rather than from the walk alone, and anchored to the
    // program's own declared order rather than only to the other subject.
    // Comparing the two subjects to each other is not enough: a subject list
    // reversed for *both* programs still swaps entry for entry, so that
    // relation holds while the interface is backwards. The declared keys are
    // the fixed point a reversal moves away from.
    for product_first in [true, false] {
        let program = ordered(product_first);
        let declared: Vec<OutputKey> = program
            .outputs()
            .map(|output| output.key().clone())
            .collect();
        let request = verify_planned_request(CompilationRequest::governed(&program))
            .expect("the boundary admits an ordered two-output program");
        let request = request.for_target(0).expect("one governed target");
        let subject: Vec<OutputKey> = request
            .subject()
            .normalized()
            .outputs()
            .iter()
            .map(|output| match output {
                NormalizedOutputSubject::Pointwise(normalized) => normalized.output_key.clone(),
                _ => panic!("both outputs of the fixture are elementwise"),
            })
            .collect();
        assert_eq!(
            subject, declared,
            "the request subject does not name the outputs in declaration order",
        );
    }
}
