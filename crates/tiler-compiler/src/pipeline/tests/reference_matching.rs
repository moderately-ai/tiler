use super::support::{alternative, alternative_dispatching};
use super::*;

/// A reindex reaches a kernel whose result is the reference evaluator's, bit for bit.
///
/// **The ticket's closing condition for the structural vocabulary.** The program
/// is `out = reverse(a)` on a `[2, 3]` operand, reversing axis 1 — the one
/// admitted within-axis coordinate permutation, and the only reindex form whose
/// decode needs the mirror. It exercises the whole vertical the widening
/// opened: the request boundary derives the coordinate map, the schedule
/// verifier discharges its bijectivity, the region's identity encodes it under
/// an appended tag, and the kernel lowering emits `extent - 1 - c` as real
/// offset arithmetic.
///
/// Bit-compared rather than approximately compared, which a reindex makes an
/// exact claim: the family computes nothing, so every output element must be an
/// input element unchanged. A tolerance here would hide the only way this can be
/// wrong — reading the *wrong* element.
#[test]
fn a_reindex_reaches_a_kernel_matching_the_reference_evaluator() {
    // Four elements, which is the governed baseline profile's declared grid
    // axis: a wider domain would decline for a launch reason and stop being
    // evidence about the access relation.
    let shape = Shape::from_dims([2, 2]);
    // Distinct, exactly representable, and deliberately not symmetric: a
    // palindromic row would make a reversal indistinguishable from an identity
    // read, which is exactly the defect this test exists to catch.
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape.clone())
        .unwrap();
    let reversed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::reverse_axis(Axis::new(1))
            .expect("an axis reversal is an admitted form"),
        input,
    )
    .expect("the standard registry admits the reindex family");
    builder
        .output(OutputKey::new("result").unwrap(), reversed)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a reindex of a declared input compiles");
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let actual = interpret_fused(&fused.kernels[0], &values);

    let key = InputKey::new("input").unwrap();
    let tensor = Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    let expected_bits = match expected[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
            .collect::<Vec<_>>(),
        _ => panic!("expected dense f32 reference output"),
    };
    assert_eq!(
        actual
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        expected_bits,
    );
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught here. Row-major `[2, 2]`
    // reversed on axis 1 is each row read backwards.
    assert_eq!(actual, vec![2.0, 1.0, 8.0, 4.0]);
}

/// `sum(x)` reaches a kernel whose result is the reference evaluator's, bit for
/// bit.
///
/// **This is `admit-a-reduction-over-a-declared-input-tensor`'s user-visible
/// outcome, end to end through the ordinary path.** One occurrence over one
/// declared input, so the recognized partition has one part, the cover places one
/// region, and that region's contributor access binds the input buffer directly —
/// asserted below beside the result, because the plan having *no materialization
/// edge* is the half of the outcome a value comparison cannot see. A synthesized
/// identity prologue would produce the same numbers through a staged temporary
/// whose rounding boundary the program never asked for.
///
/// Bit-compared rather than approximately compared. The contributors are distinct
/// powers of two and every partial sum of a row is exactly representable, so a
/// fold that read one contributor twice, skipped one, or crossed the row boundary
/// lands on a different value rather than coincidentally on the right one — which
/// a tolerance would hide.
#[test]
fn a_reduction_over_a_declared_input_matches_the_reference_evaluator() {
    // Four contributors, which is the governed baseline profile's declared grid
    // axis: a wider domain would decline for a launch reason and stop being
    // evidence about the fold.
    let shape = Shape::from_dims([2, 2]);
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), shape.clone())
        .unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a fold over a declared input compiles");
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    // One region, reading the declared input, with nothing materialized between
    // the program's boundary and its fold.
    assert_eq!(fused.scheduled_regions.len(), 1);
    assert_eq!(
        fused.scheduled_regions[0].region().index.accesses[0].tensor,
        TensorRole::Input,
    );
    assert!(fused.plan.cover().materializations().is_empty());
    let actual = interpret_fused(&fused.kernels[0], &values);

    let key = InputKey::new("input").unwrap();
    let tensor = f32_tensor(shape, &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught: row-major `[2, 2]`
    // folded on axis 1 is each row summed.
    assert_eq!(actual, vec![3.0, 12.0]);
}

/// Compiles one two-input `f32` program and returns its kernel's result beside
/// the reference evaluator's, both as bits.
///
/// The two payloads are bound to the reference by *key* and to the kernel by
/// buffer declaration order, which are independent routes to the same
/// correspondence: a compiler that ordered its region's reads against the
/// program's declared inputs would disagree here rather than agree by
/// construction.
fn compiled_and_reference_bits(
    semantic: &SemanticProgram,
    bindings: &[(&str, Shape, &[f32]); 2],
) -> (Vec<u32>, Vec<u32>) {
    let product =
        compile(CompilationRequest::governed(semantic)).expect("the structural program compiles");
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    let actual = interpret_fused_inputs(&fused.kernels[0], &[bindings[0].2, bindings[1].2]);

    let keys: Vec<InputKey> = bindings
        .iter()
        .map(|(key, ..)| InputKey::new(key).unwrap())
        .collect();
    let tensors: Vec<Tensor> = bindings
        .iter()
        .map(|(_, shape, values)| f32_tensor(shape.clone(), values))
        .collect();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            semantic,
            &[
                InputBinding::new(&keys[0], &tensors[0]),
                InputBinding::new(&keys[1], &tensors[1]),
            ],
        )
        .unwrap();
    (bits_of(&actual), tensor_bits(&expected[0]))
}

/// `scaled = contract(a, b) * 2.0` reaches a two-dispatch chain whose result is
/// the reference evaluator's, bit for bit.
///
/// **This is `admit-elementwise-epilogues-over-a-materialized-intermediate`'s
/// user-visible outcome at its smallest honest size.** The contraction stages
/// its result into a program-owned temporary and the epilogue reads it back —
/// one materialization boundary, two dispatches — and the chained interpretation
/// below is what the assembled program's own stage order says to do.
///
/// **Bit-compared rather than approximately compared, and the fixture is chosen
/// so that a wrong chain cannot pass.** The operands are exactly representable
/// and the contraction's two products differ per output position, so an epilogue
/// that read the wrong staged element, scaled before folding, or bound the
/// intermediate to a declared input would disagree in the first row.
#[test]
fn a_contraction_epilogue_chain_matches_the_reference_evaluator() {
    let operand = Shape::from_dims([2, 2]);
    // Distinct powers of two, so every product and every sum below is exact.
    // `right`'s last entry deliberately breaks the symmetry a geometric ladder
    // would have: with it, the four output positions are pairwise distinct, so a
    // chain that transposed the contraction's free indices disagrees.
    let left: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let right: Vec<f32> = vec![16.0, 32.0, 64.0, 256.0];

    let structure = tiler_ir::semantic::ContractionIndexStructure::new(
        [
            [
                tiler_ir::semantic::ContractionIndex::new(19),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
            [
                tiler_ir::semantic::ContractionIndex::new(14),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
        ],
        [
            tiler_ir::semantic::ContractionIndex::new(19),
            tiler_ir::semantic::ContractionIndex::new(14),
        ],
    )
    .unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), operand.clone())
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), operand.clone())
        .unwrap();
    let projected =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, a, b).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, projected, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product =
        compile(CompilationRequest::governed(&semantic)).expect("the epilogue chain compiles");
    let chain = alternative_dispatching(&product, 2);
    // The chain's own structure, stated rather than assumed: one materialized
    // temporary between the two dispatches, and one published output.
    assert_eq!(chain.plan.cover().materializations().len(), 1);

    let staged = interpret_fused_inputs(&chain.kernels[0], &[&left, &right]);
    let actual = interpret_fused(&chain.kernels[1], &staged);

    let a_key = InputKey::new("a").unwrap();
    let b_key = InputKey::new("b").unwrap();
    let a_tensor = f32_tensor(operand.clone(), &left);
    let b_tensor = f32_tensor(operand, &right);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            &semantic,
            &[
                InputBinding::new(&a_key, &a_tensor),
                InputBinding::new(&b_key, &b_tensor),
            ],
        )
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught. Row-major `mk,nk->mn`
    // over these operands contracts axis 1 of both: output `[m][n]` is
    // `a[m][0]*b[n][0] + a[m][1]*b[n][1]`, doubled.
    assert_eq!(actual, vec![160.0, 1152.0, 640.0, 4608.0]);
}

/// `out = contract(a, b) * a` reaches a chain whose epilogue reads the staged
/// value *and* a declared input, matching the reference bit for bit.
///
/// **This is the case that makes the access-position/declared-association
/// separation observable.** The epilogue's
/// expression has two leaves: leaf `0` is served by the read of the
/// materialized intermediate and leaf `1` by the read of declared input `a`. A
/// compiler that reused local leaf/access position `1` as declared input ordinal
/// `1`, instead of projecting exact [`AccessOrdinal`] `1` through the retained
/// checked [`crate::request::VerifiedRequestSubject`], would bind the leaf to `b`
/// and compute a different program over the same buffers. Intrinsic region
/// verification cannot detect that interface substitution: [`TensorRole::Input`]
/// is a fieldless boundary category, and the shared region carries no
/// declared-input association.
///
/// The fixture is chosen so that substitution is visible: `a` and `b` differ,
/// and the contraction is square so its result shares `a`'s shape. `b` is not
/// symmetric either, so an epilogue reading `b` disagrees rather than
/// coincidentally agreeing.
#[test]
fn an_epilogue_reading_a_staged_value_and_a_declared_input_matches_the_reference() {
    let square = Shape::from_dims([2, 2]);
    let left: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let right: Vec<f32> = vec![16.0, 32.0, 64.0, 256.0];

    let structure = tiler_ir::semantic::ContractionIndexStructure::new(
        [
            [
                tiler_ir::semantic::ContractionIndex::new(19),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
            [
                tiler_ir::semantic::ContractionIndex::new(14),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
        ],
        [
            tiler_ir::semantic::ContractionIndex::new(19),
            tiler_ir::semantic::ContractionIndex::new(14),
        ],
    )
    .unwrap();
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), square.clone())
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), square.clone())
        .unwrap();
    let projected =
        tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, a, b).unwrap();
    let root = F32Multiply::apply(&mut builder, projected, a).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product =
        compile(CompilationRequest::governed(&semantic)).expect("the epilogue chain compiles");
    let chain = alternative_dispatching(&product, 2);
    let staged = interpret_fused_inputs(&chain.kernels[0], &[&left, &right]);
    // The epilogue's two buffers in its own access order: the staged value, then
    // declared input `a`. That order is the region's, not this test's — the
    // assertion below is what would fail if the region bound them the other way.
    let actual = interpret_fused_inputs(&chain.kernels[1], &[&staged, &left]);

    let a_key = InputKey::new("a").unwrap();
    let b_key = InputKey::new("b").unwrap();
    let a_tensor = f32_tensor(square.clone(), &left);
    let b_tensor = f32_tensor(square, &right);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            &semantic,
            &[
                InputBinding::new(&a_key, &a_tensor),
                InputBinding::new(&b_key, &b_tensor),
            ],
        )
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // Stated independently of the oracle: the contraction's result is
    // `[80, 576, 320, 2304]`, multiplied elementwise by `a`.
    assert_eq!(actual, vec![80.0, 1152.0, 1280.0, 18432.0]);
}

/// An epilogue whose walk reaches declared input `b` before `a` still compiles.
///
/// **Canonical read order is compiler normalization, not an intrinsic schedule
/// rule.** The recognizer's walk mints leaves in operand order, so
/// `projected * b * a` reaches `b` first. `recognize_epilogue` rebuilds that run
/// as the staged value followed by declared inputs in declaration order, then
/// mints the expression against those exact access positions. Program assembly
/// projects each [`AccessOrdinal`] through the retained
/// [`crate::request::VerifiedRequestSubject`]; intrinsic verification sees only
/// the fieldless boundary categories and supplies no declared-input order.
///
/// Its accepted neighbour is `projected * a * b`, whose walk reaches the two
/// inputs in the opposite order. Both compile, and both are bit-compared, so a
/// build that lost the canonical order fails on exactly one of them.
#[test]
fn an_epilogue_reaching_declared_inputs_out_of_order_still_compiles() {
    let square = Shape::from_dims([2, 2]);
    let left: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let right: Vec<f32> = vec![16.0, 32.0, 64.0, 256.0];

    let structure = tiler_ir::semantic::ContractionIndexStructure::new(
        [
            [
                tiler_ir::semantic::ContractionIndex::new(19),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
            [
                tiler_ir::semantic::ContractionIndex::new(14),
                tiler_ir::semantic::ContractionIndex::new(3),
            ],
        ],
        [
            tiler_ir::semantic::ContractionIndex::new(19),
            tiler_ir::semantic::ContractionIndex::new(14),
        ],
    )
    .unwrap();
    let program = |descending: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), square.clone())
            .unwrap();
        let b = builder
            .input::<F32>(InputKey::new("b").unwrap(), square.clone())
            .unwrap();
        let projected =
            tiler_ir::semantic::F32TensorContraction::apply(&mut builder, &structure, a, b)
                .unwrap();
        let (first, second) = if descending { (b, a) } else { (a, b) };
        let scaled = F32Multiply::apply(&mut builder, projected, first).unwrap();
        let root = F32Multiply::apply(&mut builder, scaled, second).unwrap();
        builder
            .output(OutputKey::new("out").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };

    for descending in [false, true] {
        let semantic = program(descending);
        let product = compile(CompilationRequest::governed(&semantic))
            .unwrap_or_else(|error| panic!("descending={descending} refused: {error:?}"));
        let chain = alternative_dispatching(&product, 2);
        let staged = interpret_fused_inputs(&chain.kernels[0], &[&left, &right]);
        // The epilogue's buffers in its own access order: the staged value, then
        // declared input `a`, then `b` — regardless of which the walk reached
        // first, which is the property under test.
        let actual = interpret_fused_inputs(&chain.kernels[1], &[&staged, &left, &right]);

        let a_key = InputKey::new("a").unwrap();
        let b_key = InputKey::new("b").unwrap();
        let a_tensor = f32_tensor(square.clone(), &left);
        let b_tensor = f32_tensor(square.clone(), &right);
        let expected = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(
                &semantic,
                &[
                    InputBinding::new(&a_key, &a_tensor),
                    InputBinding::new(&b_key, &b_tensor),
                ],
            )
            .unwrap();
        assert_eq!(
            bits_of(&actual),
            tensor_bits(&expected[0]),
            "descending={descending}",
        );
    }
}

/// `scaled = sum(x * x, axis 1) * 2.0` reaches a three-dispatch chain whose
/// result is the reference evaluator's, bit for bit.
///
/// The reduction half of the same outcome, and it is three dispatches rather
/// than two because the fold's contributors are themselves computed: the
/// prologue stages `x * x`, the fold stages its result, and the epilogue reads
/// that. Both materialization edges are exercised, which is what makes this the
/// case that would catch a chain binding the second edge to the first's buffer.
#[test]
fn a_reduction_epilogue_chain_matches_the_reference_evaluator() {
    let shape = Shape::from_dims([1, 4]);
    // Exactly representable, and no two squares equal, so a fold that dropped or
    // repeated a contributor disagrees.
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let x = builder
        .input::<F32>(InputKey::new("x").unwrap(), shape.clone())
        .unwrap();
    let squared = F32Multiply::apply(&mut builder, x, x).unwrap();
    let reduced =
        tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, squared, [Axis::new(1)])
            .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, reduced, two).unwrap();
    builder
        .output(OutputKey::new("scaled").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product =
        compile(CompilationRequest::governed(&semantic)).expect("the epilogue chain compiles");
    let chain = alternative_dispatching(&product, 3);
    assert_eq!(chain.plan.cover().materializations().len(), 2);

    let prologue = interpret_fused(&chain.kernels[0], &values);
    let folded = interpret_fused(&chain.kernels[1], &prologue);
    let actual = interpret_fused(&chain.kernels[2], &folded);

    let key = InputKey::new("x").unwrap();
    let tensor = f32_tensor(shape, &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // `(1 + 4 + 16 + 64) * 2`, stated independently of the oracle.
    assert_eq!(actual, vec![170.0]);
}

/// A chain whose producer is the *fused* prologue-and-fold region compiles, and
/// both of its spellings match the reference bit for bit.
///
/// **This is what makes threading `RegionWrite` into `fused_region` a
/// requirement rather than a tidy-up.** `sum(a * 2.0 + 1.0, axis 1) * 3.0` has
/// two producer spellings — the prologue and the fold as separate regions, or
/// the affine fold that absorbs the prologue — and a cover may materialize
/// either for the epilogue to read. A `fused_region` that hard-coded
/// `TensorRole::Output` would build a region writing the tensor the cover did
/// not assign, `CoverAssembly::from_plan` would refuse it under
/// `cover-materialization-unnamed`, and the two-dispatch alternative would
/// disappear — silently, because a dropped alternative is not a refusal.
///
/// The stage counts below are therefore the assertion: `[2, 3]` is both
/// spellings retained, and `[3]` alone would be the fused one lost.
#[test]
fn a_chain_over_a_fused_prologue_and_fold_retains_both_producer_spellings() {
    let shape = Shape::from_dims([1, 4]);
    // Exactly representable, and `scale * x + bias` is exact on each, so the
    // fused and unfused producers agree bit for bit and the epilogue's result is
    // attributable to the chain rather than to rounding.
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), shape.clone())
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, a, scale).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, bias).unwrap();
    let reduced =
        tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, shifted, [Axis::new(1)])
            .unwrap();
    let three = F32Constant::apply(&mut builder, 3.0_f32.to_bits()).unwrap();
    let root = F32Multiply::apply(&mut builder, reduced, three).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), root)
        .unwrap();
    let semantic = builder.build().unwrap();

    let product =
        compile(CompilationRequest::governed(&semantic)).expect("the epilogue chain compiles");
    let mut counts: Vec<usize> = product.targets[0]
        .portfolio
        .alternatives
        .iter()
        .map(|alternative| alternative.program.stage_count())
        .collect();
    counts.sort_unstable();
    assert_eq!(
        counts,
        vec![2, 3],
        "the fused producer's chain was lost; only the unfused one survived",
    );

    let key = InputKey::new("a").unwrap();
    let tensor = f32_tensor(shape, &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();

    // The fused producer: one dispatch staging the fold's result, then the
    // epilogue reading it.
    let fused = alternative_dispatching(&product, 2);
    let staged = interpret_fused(&fused.kernels[0], &values);
    assert_eq!(
        bits_of(&interpret_fused(&fused.kernels[1], &staged)),
        tensor_bits(&expected[0]),
    );

    // The unfused producer: prologue, fold, epilogue.
    let unfused = alternative_dispatching(&product, 3);
    let prologue = interpret_fused(&unfused.kernels[0], &values);
    let folded = interpret_fused(&unfused.kernels[1], &prologue);
    assert_eq!(
        bits_of(&interpret_fused(&unfused.kernels[2], &folded)),
        tensor_bits(&expected[0]),
    );

    // `(3 + 5 + 9 + 17) * 3`, stated independently of the oracle.
    assert_eq!(interpret_fused(&fused.kernels[1], &staged), vec![102.0]);
}

/// A widening broadcast reaches a kernel whose result is the reference
/// evaluator's, bit for bit.
///
/// **This is the ticket's user-visible outcome at its smallest honest size.**
/// The program is `out = a * broadcast(w)` with `a` at `[2, 2]` and `w` declared
/// at `[2]` and read at every row — the `[1024]`-against-`[T, 1024]` shape of the
/// normalization weight multiply, which is 113 of the pinned workload's 197
/// broadcast occurrences. Only the extents are smaller: the governed baseline
/// profile declares a four-thread grid axis, so a wider domain would decline for
/// a launch reason and stop being evidence about the access relation.
///
/// **The two reads are at different element counts, and that is the point.** The
/// widened read addresses its *operand's* range — two elements against the
/// region's four — so a region binding it against the domain would address past
/// the weight's end, and a machine holding one payload could not model the
/// program at all.
///
/// Bit-compared rather than approximately compared, for the reason the reindex
/// test states: a broadcast computes nothing, so every weight the multiply reads
/// must be an input element unchanged, and a tolerance would hide the only way
/// this can be wrong — replicating along the wrong axis.
#[test]
fn a_broadcast_reaches_a_kernel_matching_the_reference_evaluator() {
    let domain = Shape::from_dims([2, 2]);
    let weight_shape = Shape::from_dims([2]);
    // Distinct and exactly representable on both sides. The weight's two entries
    // must differ: a uniform weight makes replication along axis 0
    // indistinguishable from replication along axis 1, which is exactly the
    // defect this test exists to catch.
    let activations: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let weights: Vec<f32> = vec![3.0, 5.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let w = builder
        .input::<F32>(InputKey::new("w").unwrap(), weight_shape.clone())
        .unwrap();
    let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
        [
            tiler_ir::shape::Extent::new(2),
            tiler_ir::shape::Extent::new(2),
        ],
        [
            tiler_ir::semantic::BroadcastAxisSource::Replicate,
            tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
        ],
    )
    .expect("one replicated axis over a rank-one operand is an admitted relation");
    let widened = tiler_ir::semantic::F32Broadcast::apply(&mut builder, &mapping, w)
        .expect("the standard registry admits the broadcast family");
    let scaled = F32Multiply::apply(&mut builder, a, widened).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let (actual, expected) = compiled_and_reference_bits(
        &semantic,
        &[("a", domain, &activations), ("w", weight_shape, &weights)],
    );
    assert_eq!(actual, expected);
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught. Row-major `[2, 2]`
    // against a `[2]` weight replicated over axis 0 is `out[i][j] = a[i][j] *
    // w[j]`; replicating over the *other* axis would give `3, 6, 20, 40`, which
    // is why this literal discriminates.
    assert_eq!(actual, bits_of(&[3.0, 10.0, 12.0, 40.0]));
}

/// A reindex feeding a pointwise multiply reaches a kernel matching the
/// reference evaluator, bit for bit.
///
/// **This is Milestone 2's "reindex plus pointwise fusion" at the smallest
/// domain the governed profile launches.** The program is
/// `out = permute(a) * b`, where the permutation is `rearrange('i j -> j i')` —
/// an einops `rearrange` written in the one form the `Reindex` family spells for
/// it, `permute-axes`. Both operands are declared inputs at `[2, 2]`, so one
/// region carries a structural read and a dense read side by side, which is what
/// "fused" means here: no intermediate is materialized between the rearrangement
/// and the arithmetic, and the transpose contributes an access map rather than a
/// copy kernel.
///
/// **The reindex half is deliberately not the reversal.** `reverse-axis` is the
/// one form whose decode mirrors, and its bit comparison is already
/// [`a_reindex_reaches_a_kernel_matching_the_reference_evaluator`]'s. A permute
/// exercises the divide-and-modulo decode instead, over a *second* read the
/// region addresses densely in the same body — the composition neither of those
/// two tests covers on its own.
#[test]
fn a_reindexed_operand_feeding_a_multiply_matches_the_reference_evaluator() {
    let domain = Shape::from_dims([2, 2]);
    // Powers of two, so every product below is exact and any disagreement is a
    // wrong *element* rather than a rounding. Deliberately not symmetric under
    // transposition: `a` transposed is `1, 4, 2, 8`, so a compiler that dropped
    // the permutation entirely would produce a different tensor.
    let left: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];
    let right: Vec<f32> = vec![3.0, 5.0, 7.0, 11.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let b = builder
        .input::<F32>(InputKey::new("b").unwrap(), domain.clone())
        .unwrap();
    let transposed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("an axis permutation is an admitted form"),
        a,
    )
    .expect("the standard registry admits the reindex family");
    let scaled = F32Multiply::apply(&mut builder, transposed, b).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), scaled)
        .unwrap();
    let semantic = builder.build().unwrap();

    let (actual, expected) = compiled_and_reference_bits(
        &semantic,
        &[("a", domain.clone(), &left), ("b", domain, &right)],
    );
    assert_eq!(actual, expected);
    // Independently of the oracle: `a` transposed is `1, 4, 2, 8`, multiplied
    // elementwise by `3, 5, 7, 11`. Without the permutation the product would be
    // `3, 10, 28, 88`, which is what makes this literal discriminating.
    assert_eq!(actual, bits_of(&[3.0, 20.0, 14.0, 88.0]));
}

/// `out = a * permute(a)` reaches a kernel matching the reference evaluator, bit
/// for bit — one declared input read twice, densely and through a relation.
///
/// **This is the regression for a measured silently wrong result.** At
/// `912b6058` this program compiled and returned `[1, 16, 4, 64]`, which is
/// `permute(a) * permute(a)`: the region bound one read per declared input, the
/// expression's two `Input { ordinal: 0 }` leaves shared it, and the mapped
/// relation served both. The reference gives `[1, 8, 8, 64]`.
/// `admit-elementwise-epilogues-over-a-materialized-intermediate` closed it
/// fail-closed under `structural-access-conflict`; this is the widening that
/// admits the program instead, so the assertion is the *value*, not the
/// admission.
///
/// **The fixture is deliberately asymmetric under transposition**, so the wrong
/// answer is a different tensor rather than coincidentally the right one: `a` is
/// `[[1, 2], [4, 8]]` and its transpose is `[[1, 4], [2, 8]]`. Every product is a
/// power of two and exactly representable, so a bit comparison is the whole
/// comparison and no tolerance can hide a wrong element.
///
/// The region binds *two* read buffers to the one declared input, which is why
/// the payload is passed twice: the kernel's buffer list is the region's read
/// list, and two reads of one tensor are two bindings rather than one shared.
#[test]
fn one_input_read_densely_and_through_a_permutation_matches_the_reference_evaluator() {
    let domain = Shape::from_dims([2, 2]);
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let transposed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("an axis permutation is an admitted form"),
        a,
    )
    .expect("the standard registry admits the reindex family");
    let product = F32Multiply::apply(&mut builder, a, transposed).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), product)
        .unwrap();
    let semantic = builder.build().unwrap();

    let compiled = compile(CompilationRequest::governed(&semantic))
        .expect("one declared input may be read densely and through a relation");
    let fused = alternative(&compiled, ProgramAlternativeKind::Fused);
    assert_eq!(fused.scheduled_regions.len(), 1);
    // Two reads of declared input `0`, dense first, then the write. Asserted
    // because it is the fact the value comparison below depends on: a region
    // that bound one read could not spell this program at all.
    let accesses = &fused.scheduled_regions[0].region().index.accesses;
    assert_eq!(accesses.len(), 3);
    let first_input = TensorRole::Input;
    assert_eq!(accesses[0].tensor, first_input);
    assert_eq!(
        accesses[0].map,
        tiler_ir::schedule::LogicalAccess::LinearIdentity
    );
    assert_eq!(accesses[1].tensor, first_input);
    assert!(matches!(
        accesses[1].map,
        tiler_ir::schedule::LogicalAccess::ReindexBijection { .. }
    ));
    assert!(fused.plan.cover().materializations().is_empty());

    let actual = interpret_fused_inputs(&fused.kernels[0], &[&values, &values]);

    let key = InputKey::new("a").unwrap();
    let tensor = f32_tensor(domain, &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // Stated independently of the oracle, and stated as the *pair*: the measured
    // wrong answer is named beside the right one, so a regression that reinstated
    // the shared read fails on a value this test already knows the meaning of.
    assert_eq!(actual, vec![1.0, 8.0, 8.0, 64.0]);
    assert_ne!(actual, vec![1.0, 16.0, 4.0, 64.0]);
}

/// `sum(permute(a) * 2.0 + 1.0)` withholds the affine-fused fold and compiles
/// through the materialized pair, matching the reference bit for bit.
///
/// **The fused single-region alternative would drop the permutation, and only
/// the read list can report it.** `ScalarProgram::FusedMultiplyAddSerialSum`
/// applies `scale * x + bias` to each contributor of a
/// [`LogicalAccess::ReductionContributor`] read of the declared input — it has no
/// place to put a structural relation. The prologue expression it is recovered
/// from is `input(0) * 2.0 + 1.0` whether the read is dense or transposed, so
/// every fact `affine_prologue` inspects agrees for both programs, and fusing
/// would fold `a` where the caller wrote `permute(a)`.
///
/// Found while widening the read list, and fixed rather than filed: the
/// alternative returned a wrong tensor, which the architectural contract does
/// not admit deferring. `fused_prologue_constants` declines the candidate, and
/// declining loses a candidate rather than a program — the materialized pair
/// below realizes it.
///
/// The fixture is asymmetric under transposition, so the dropped permutation is
/// a different tensor rather than the same one: over `[[1, 2], [4, 8]]` the
/// correct rows are `[12, 22]` and the unpermuted fold gives `[8, 26]`.
#[test]
fn an_affine_prologue_read_through_a_relation_declines_the_fused_fold() {
    let domain = Shape::from_dims([2, 2]);
    let values: Vec<f32> = vec![1.0, 2.0, 4.0, 8.0];

    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), domain.clone())
        .unwrap();
    let transposed = tiler_ir::semantic::F32Reindex::apply(
        &mut builder,
        &tiler_ir::semantic::ReindexForm::permute_axes([Axis::new(1), Axis::new(0)])
            .expect("an axis permutation is an admitted form"),
        a,
    )
    .expect("the standard registry admits the reindex family");
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, transposed, two).unwrap();
    let one = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let biased = F32Add::apply(&mut builder, scaled, one).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, biased, [Axis::new(1)]).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    let semantic = builder.build().unwrap();

    let product = compile(CompilationRequest::governed(&semantic))
        .expect("a structurally read affine prologue compiles through the materialized pair");
    // No single-region alternative at all: the fused fold is the only one this
    // program could have, and it is withheld rather than offered and rejected.
    assert!(
        product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .all(|retained| retained.kind != ProgramAlternativeKind::Fused),
        "the affine fusion must be withheld for a prologue read through a relation",
    );

    let chain = alternative_dispatching(&product, 2);
    let staged = interpret_fused(&chain.kernels[0], &values);
    let actual = interpret_fused(&chain.kernels[1], &staged);

    let key = InputKey::new("a").unwrap();
    let tensor = f32_tensor(domain, &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
    // Independently of the oracle, and naming the wrong answer beside the right
    // one: `a` transposed is `[[1, 4], [2, 8]]`, scaled and biased to
    // `[[3, 9], [5, 17]]`, folded on axis 1. Dropping the permutation would fold
    // `[[3, 5], [9, 17]]` instead.
    assert_eq!(actual, vec![12.0, 22.0]);
    assert_ne!(actual, vec![8.0, 26.0]);
}
