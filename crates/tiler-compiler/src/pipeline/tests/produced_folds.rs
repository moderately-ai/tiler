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
    let semantic = builder.build().unwrap();
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

/// A produced fold's own fold stays resolvable, so the whole-program numerical
/// proof is still attributed to the provider that lowers it.
///
/// **The assertion pins what the carrier keeps rather than what it changes.**
/// `record_numerical_equivalence` resolves the reduction whose reassociation the
/// proof forbids by asking `output_for_region` for the candidate's occurrences
/// and then `try_serial_sum` for that output's fold. A produced sum is still the
/// serial-sum arm, so both answers survive; a carrier that had moved produced
/// sums to a *different* `NormalizedOutput` variant would silently degrade this
/// to `reduction-provider-missing`, attributing no provider to a proof that
/// exists. Pinning it is what would say so.
#[test]
fn a_produced_folds_reduction_stays_resolvable_for_the_numerical_proof() {
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
    let semantic = builder.build().unwrap();
    let verified = verify_planned_request(CompilationRequest::governed(&semantic)).unwrap();
    let request = verified.for_target(verified.target_profiles()[0]).unwrap();

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
