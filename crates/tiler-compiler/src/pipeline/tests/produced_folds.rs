use super::support::{alternative, plan_formation};
use super::*;

/// `sum(input, [cols])` — the declared-input fold whose whole-program region
/// merges nothing, and the one shape the fused proof exemption is about.
fn declared_input_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// `result = sum(sum(input, [cols]) * 2.0, [rows])` — the fold whose
/// contributors another region materializes.
///
/// `publish_producer` additionally declares the inner fold as a second ordered
/// named output, which makes the producing part both published and consumed.
/// The two spellings share every operation, so a test comparing them is
/// comparing the publication and nothing else.
fn produced_fold(publish_producer: bool) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    let scaled = F32Multiply::apply(&mut builder, inner, two).unwrap();
    let outer = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), outer)
        .unwrap();
    if publish_producer {
        builder
            .output(OutputKey::new("rows").unwrap(), inner)
            .unwrap();
    }
    builder.build().unwrap()
}

/// The fused proof exemption states the declared-input contributor, so a
/// produced fold takes the ordinary `portfolio-equivalence` proof path.
///
/// **The subject is perturbed, not the assertion.** Only the recognized fold's
/// contributor source moves — from `DeclaredInput` to `Materialized` over the
/// same fold's own recognized shape — and the alternative, the formation, and
/// the semantic program are the identical values the control ran against. What
/// the exemption used to read was `serial.prologue.is_none()`, which a produced
/// fold satisfies exactly as `sum(x)` does, so under the old spelling this
/// perturbation would have been *exempted* from the numerical replay while the
/// arm's own comment claimed to be about a fold that merges nothing.
///
/// Reachability, stated so the severity is not overread: no genuine produced-sum
/// plan classifies `Fused` — `ProgramAlternativeKind::of` needs one region to
/// cover every operation, and a produced fold's partition is at least a producer
/// region and a fold — so the live exposure is a forged receipt rather than a
/// wrong compile. That is the verifier's own independence contract, which is
/// why the repair is a correctness requirement and not a cleanup.
#[test]
fn a_produced_folds_fused_receipt_takes_the_ordinary_proof_path() {
    let semantic = declared_input_fold();
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let fused = alternative(&product, ProgramAlternativeKind::Fused);
    assert!(
        fused.equivalence.numerical.is_none(),
        "the control must be the exempt arm — a fused alternative carrying no numerical proof — \
         or this test perturbs a path the exemption never governed",
    );
    let formation = plan_formation(&semantic, &request);

    // The control: `sum(x)` keeps its exemption and verifies.
    assert_eq!(
        crate::pipeline::verify::verify_equivalence(&semantic, &request, &formation, fused),
        Ok(()),
        "the declared-input fold must keep its exemption",
    );

    // The perturbation: the same fold, its contributor source moved to a
    // materialized producer. Nothing else changes.
    let written_by = request.sole_output().clone();
    let mut materialized = request.clone();
    materialized.perturb_serial_sum_contributor(
        crate::request::SerialSumContributor::Materialized(Box::new(
            crate::request::MaterializedContributor {
                producer: written_by,
                continuation: None,
            },
        )),
    );
    let refused =
        crate::pipeline::verify::verify_equivalence(&semantic, &materialized, &formation, fused)
            .expect_err("a produced fold is not exempt from the numerical replay");
    assert_eq!(
        refused.to_string(),
        "program.structure.portfolio-equivalence: rejected",
    );

    // The second control the exemption's own comment states: a fold that *does*
    // carry a prologue is a fusion, so it falls through to the proving arm too.
    let mut prologue = request.clone();
    let scale = {
        let mut expression = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
        let leaf = expression.input(AccessOrdinal::FIRST).unwrap();
        let two = expression.constant(2.0_f32.to_bits()).unwrap();
        let root = expression.multiply(leaf, two).unwrap();
        expression.build(root).unwrap()
    };
    prologue.perturb_serial_sum_contributor(
        crate::request::SerialSumContributor::PointwisePrologue {
            expression: scale,
            reads: vec![(
                crate::request::DeclaredInputOrdinal::new(0),
                tiler_ir::schedule::LogicalAccess::LinearIdentity,
            )],
        },
    );
    assert_eq!(
        crate::pipeline::verify::verify_equivalence(&semantic, &prologue, &formation, fused)
            .expect_err("a fold with a prologue is a fusion")
            .to_string(),
        "program.structure.portfolio-equivalence: rejected",
    );
}

/// A produced fold's continuation region is reported as an epilogue, never as
/// the whole program.
///
/// **A named hand-work site the type change does not force**, and the wrong
/// answer would be a trace telling a reader that one of at least three regions
/// *is* the program. The producer's own regions keep the roles they would carry
/// standing alone, which is what makes two traces comparable.
#[test]
fn a_produced_folds_region_roles_name_the_part_rather_than_the_program() {
    let semantic = produced_fold(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();

    let fold = request.sole_output().serial_sum();
    let materialized = fold
        .contributor
        .materialized()
        .expect("the fixture folds a materialized contributor");
    let continuation = materialized
        .continuation
        .as_ref()
        .expect("the `* 2` is a continuation region");

    assert_eq!(region_role(&request, &continuation.members), "epilogue");
    assert_eq!(region_role(&request, fold.members.reduction()), "reduction");
    assert_eq!(
        region_role(&request, &materialized.producer.members()),
        "reduction",
        "the producer's fold keeps the role it would carry standing alone",
    );
}

/// A produced fold's own fold stays resolvable, so a numerical proof over it
/// would still be attributed to the provider that lowers it.
///
/// **The assertion pins what the carrier keeps rather than what it changes.**
/// `record_numerical_equivalence` resolves the reduction whose reassociation the
/// proof forbids by asking `output_for_region` for the candidate's occurrences
/// and then `try_serial_sum` for that output's fold. A produced sum is still the
/// serial-sum arm, so both answers survive; a carrier that had moved produced
/// sums to a *different* `NormalizedOutput` variant would silently degrade this
/// to `reduction-provider-missing`, attributing no provider to a proof that
/// exists. Pinning it is what would say so.
///
/// **The lookup is restated here rather than driven through the function, and
/// the reason is reachability rather than convenience.** `planning` records the
/// proof only for a whole-program candidate whose implemented output carries
/// fused prologue constants, and a fold whose contributors another region
/// materializes has no prologue at all — so no produced fold reaches
/// `record_numerical_equivalence` on any path. The guard's own first conjunct
/// declines one step earlier still: this program's whole-program candidate
/// covers four occurrences that no single recognized output owns, so
/// `output_for_region` answers `None` for it. Nor could the call be assembled by
/// hand: its `FusionNumericalProof` argument comes from `prove_fused_numerics`
/// over a fused region a produced fold has no spelling for. The guard is
/// therefore asserted below beside the lookup, so a change that opened it would
/// redden here rather than silently start recording a proof about a fold this
/// test only ever inspected.
#[test]
fn a_produced_folds_reduction_stays_resolvable_for_the_numerical_proof() {
    let semantic = produced_fold(false);
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();

    // The guard that keeps the recording path away from this population.
    assert_eq!(
        crate::physical::fused_prologue_constants(request.sole_output()),
        None,
        "a materialized contributor has no prologue to fuse, so the recording guard never opens",
    );
    let formation = plan_formation(&semantic, &request);
    assert!(
        formation
            .whole_program_candidate()
            .is_some_and(|candidate| request.output_for_region(candidate.members()).is_none()),
        "the whole-program candidate must exist and own no recognized output, which is where the \
         guard declines first",
    );

    let fold = request.sole_output().serial_sum();
    let reduction = fold.members.reduction().to_vec();
    let resolved = request
        .output_for_region(&reduction)
        .and_then(|(_, output)| output.try_serial_sum())
        .map(|serial| serial.members.reduction().to_vec());
    assert_eq!(
        resolved,
        Some(reduction),
        "a produced fold must stay the serial-sum arm, or the proof loses its provider",
    );

    // And the producer's own fold resolves to the same output — the partition
    // is one output's end to end, which is what `check_output_cover` requires.
    let producer_members = fold
        .contributor
        .materialized()
        .expect("the fixture folds a materialized contributor")
        .producer
        .members();
    assert!(request.output_for_region(&producer_members).is_some());
}

/// A produced fold whose *producer* is also a declared output compiles, and both
/// publications agree with the reference bit for bit.
///
/// **Publishing a value another output consumes reaches this boundary only
/// through `published_and_consumed_overlap`,** whose load-bearing conjunct is
/// that the shorter walk is one whole *part* of the longer walk's recognized
/// partition. What makes the producing fold a part here is the exhaustive
/// contributor source's `Materialized` arm retaining the producing shape, so
/// this population is admitted by a change whose subject was the contributor
/// fields. `crate::pipeline::conformance`'s
/// `a_published_and_consumed_intermediate_compiles_and_agrees` is the neighbour
/// where the published part is a *pointwise prologue*; this is the same overlap
/// where it is a *fold*, and it had no in-tree assertion until this test.
///
/// **What makes it say no, each observed by perturbing the subject.** Making
/// `published_and_consumed_overlap` decline returns `UnsupportedCapability {
/// phase: "strategy", rule: "output-partition-overlap" }` from
/// `verify_planned_request`, which is the refusal this shape carried before the
/// arm existed. Resolving every fold's contributor to `TensorRole::Input`, the
/// derivation `crate::physical::subject_contributor_tensor` retired, leaves the
/// consuming fold bound to a declared buffer no cover writes and the target
/// fails `no-complete-plan`. The interpreted values are the third: the input is
/// chosen so that no two elements, no two row sums, and no partial sum coincide,
/// so a fold bound to the wrong staged buffer disagrees rather than matching by
/// accident.
#[test]
fn a_produced_folds_published_producer_compiles_and_agrees() {
    let values: [f32; 4] = [1.0, 2.0, 4.0, 8.0];
    let semantic = produced_fold(true);
    assert_eq!(semantic.output_count(), 2);
    assert_eq!(semantic.operation_count(), 4);

    // The recognized shape the admission rests on: the consuming output still
    // folds a materialized contributor, and the value the second output
    // publishes is that contributor's producer.
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();
    let consuming = request
        .output_at(0)
        .try_serial_sum()
        .expect("`result` folds");
    let producer = &consuming
        .contributor
        .materialized()
        .expect("the fixture folds a materialized contributor")
        .producer;
    assert_eq!(request.output_at(1).members(), producer.members());

    let product =
        compile(CompilationRequest::governed(&semantic)).expect("the published producer compiles");
    assert_eq!(product.targets[0].failure(), None);
    let retained: Vec<&ProgramAlternative> =
        product.targets[0].portfolio.alternatives.iter().collect();
    let [alternative] = retained.as_slice() else {
        panic!(
            "expected one retained alternative, found {}",
            retained.len()
        );
    };

    // Four dispatches over three cover regions: the producing fold writing the
    // edge, the copy publishing `rows` from it, the `* 2.0` continuation, and
    // the consuming fold. Two materializations rather than one, because the
    // continuation stages its own result for the fold above it.
    assert_eq!(alternative.plan.cover().region_count(), 3);
    assert_eq!(alternative.plan.cover().materializations().len(), 2);
    assert_eq!(alternative.kernels.len(), 4);
    assert_eq!(
        alternative
            .program
            .core()
            .outputs()
            .map(|output| output.key().as_str().to_owned())
            .collect::<Vec<_>>(),
        vec!["result".to_owned(), "rows".to_owned()],
    );
    // Exactly one stage covers no occurrence, and it is the one the publishing
    // copy names — the publication is a second dispatch of the producing region
    // rather than a recomputation of the fold.
    let copies: Vec<_> = alternative.program.core().publishing_copies().collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(
        alternative
            .program
            .core()
            .stages()
            .filter(|stage| stage.coverage().is_empty())
            .count(),
        1,
    );
    assert!(copies[0].publisher().coverage().is_empty());
    assert!(!copies[0].source_stage().coverage().is_empty());

    // The four dispatches in declaration order, each reading what the one before
    // it staged.
    let staged = interpret_fused(&alternative.kernels[0], &values);
    let rows = interpret_fused(&alternative.kernels[1], &staged);
    let scaled = interpret_fused(&alternative.kernels[2], &staged);
    let result = interpret_fused(&alternative.kernels[3], &scaled);

    let key = InputKey::new("input").unwrap();
    let tensor = f32_tensor(Shape::from_dims([2, 2]), &values);
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
        .unwrap();
    assert_eq!(bits_of(&result), tensor_bits(&expected[0]));
    assert_eq!(bits_of(&rows), tensor_bits(&expected[1]));
    // Stated independently of the oracle as well, so a reference evaluator that
    // agreed with a wrong compiler would still be caught.
    assert_eq!(rows, vec![3.0, 12.0]);
    assert_eq!(scaled, vec![6.0, 24.0]);
    assert_eq!(result, vec![30.0]);
}
