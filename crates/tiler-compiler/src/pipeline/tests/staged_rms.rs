use super::support::{region_attributions, semantic};
use super::*;

/// Caller-declared profile for staged RMS tests.
///
/// Its capability values and silences mirror the governed profile. The only
/// added authority is a synthetic, fully discharging RMS realization, so these
/// tests exercise the staged compiler without turning the fixture into a Metal
/// capability claim.
fn staged_rms_target_profile() -> TargetProfile {
    use crate::target::{
        DTypeDispatchability, ElementaryRealization, IndexArithmeticSupport, ScalarArithmetic,
        ScalarSupport, TargetFactProducerIdentity, TargetFactSource,
        TargetNormativeReferenceIdentity, TargetProfileBuilder, TargetProfileKey,
    };
    use tiler_ir::program::abi::{
        TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
    };
    use tiler_ir::schedule::{
        ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
        SubnormalMode,
    };
    use tiler_ir::semantic::accuracy::{ConformanceEvidence, ConformanceEvidenceClass};
    use tiler_ir::semantic::{
        builtin_scalar_value_type_facts, rms_norm_f32_rsqrt_accuracy_contract,
    };

    let source = TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.staged-rms-profile.v1".to_owned(), 1).unwrap(),
        TargetNormativeReferenceIdentity::new("test.staged-rms-fixture.v1".to_owned(), 1).unwrap(),
    );
    let mut builder = TargetProfileBuilder::new(
        TargetProfileKey::new("test.staged-rms-discharging.v1".to_owned()).unwrap(),
    );
    builder
        .declare_max_threads_per_grid_axis(4, source.clone())
        .unwrap();
    builder
        .declare_max_threads_per_workgroup_query(
            TargetPropertyQuery::new(
                TargetPropertyKey::new("tiler.target.prepared-entry.max-threads-per-workgroup.v1")
                    .unwrap(),
                AvailabilityPhase::PreparedKernelPreflight,
                TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    builder
        .declare_max_buffer_bindings_per_entry(4, source.clone())
        .unwrap();
    builder
        .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .unwrap();
    builder.declare_device_memory(true, source.clone()).unwrap();
    builder
        .declare_local_memory_bytes(0, source.clone())
        .unwrap();

    let subject = ScalarArithmetic::f32();
    for behaviour in [
        SubnormalMode::Preserve,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        },
    ] {
        builder
            .declare_input_subnormals(
                subject.clone(),
                behaviour,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_result_subnormals(
                subject.clone(),
                behaviour,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
    }
    for permission in [
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
    ] {
        builder
            .declare_contraction(
                subject.clone(),
                permission,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_reassociation(
                subject.clone(),
                permission,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
    }
    builder
        .declare_permutation(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_signed_zero(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_reciprocal_transform(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_approximate_intrinsics(
            subject.clone(),
            ApproximationEnvelope::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_nan_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_infinity_assumptions(
            subject,
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source.clone(),
        )
        .unwrap();

    let contract = rms_norm_f32_rsqrt_accuracy_contract();
    let facts = builtin_scalar_value_type_facts(contract.result_type()).unwrap();
    let verified = contract.verify(&facts).unwrap();
    let evidence = |scope: &str, digest: &[u8]| {
        let reference = |text: &str| tiler_ir::semantic::NormativeDefinitionRef::new(text).unwrap();
        ConformanceEvidence::new(
            ConformanceEvidenceClass::NormativeGuarantee,
            reference(scope),
            reference("synthetic staged RMS fixture, not a target or Metal specification claim"),
            reference("fixture.staged-rms.caller-declaration"),
            reference("tiler test fixture, not a toolchain row"),
            None,
            None,
            None,
            digest,
        )
        .unwrap()
    };
    builder
        .declare_elementary_realization(
            ElementaryRealization::new(
                &verified,
                evidence(
                    "caller bound half for tiler::rms-norm-f32@1",
                    b"fixture:staged-rms-bound-v1",
                ),
                evidence(
                    "caller exceptional half for tiler::rms-norm-f32@1",
                    b"fixture:staged-rms-exceptional-v1",
                ),
                &source,
            )
            .unwrap(),
        )
        .unwrap();
    builder.build().unwrap()
}

fn staged_rms_request(program: &SemanticProgram) -> CompilationRequest<'_> {
    let mut request = CompilationRequest::governed(program);
    request.target_profiles = vec![staged_rms_target_profile()];
    request
}

fn verified_staged_rms_request(
    program: &SemanticProgram,
) -> crate::request::VerifiedCompilationRequest {
    match crate::request::verify_request(staged_rms_request(program)).unwrap() {
        crate::request::VerifiedRequest::Planned(request) => *request,
        crate::request::VerifiedRequest::Refused(slots) => panic!(
            "the staged RMS fixture was refused during request verification: {:?}",
            slots
                .iter()
                .map(crate::request::VerifiedTargetSlot::resolution)
                .collect::<Vec<_>>()
        ),
    }
}

/// A registered staged family compiles end to end and computes the right value.
///
/// **The program is `rms_norm(value, weight) * value`**: a registered elementary
/// family as a program stage a later elementwise pass consumes. Four facts are
/// measured, and the last two are what
/// [`account-for-a-staged-realization-stage-in-the-kernel-program`] moved.
///
/// *It is recognized and its own lowering runs.* The occurrence resolves its
/// index-access capability and `refine_index_region` proves
/// `GovernedRootMeanSquareScaleF32`'s emitted chain realizes it — a two-stage
/// realization handing one value on.
///
/// *Both stages are spelled.* The producing stage is a
/// `ScalarProgram::SquaredSerialSumThenEpilogue` region over the reduced domain,
/// the consuming stage a pointwise pass reading the handed value at its kept
/// coordinates, and both are answered with an implementation — so
/// `region-staged-family-unspellable` no longer appears on either of them. The
/// only regions still declining are the one carrying both stages and the two
/// grouping a stage of the normalization with the consuming multiply, which no
/// recognized partition owns.
///
/// *The program assembles.* The consuming stage covers no occurrence's *first*
/// attribution atom, and program scope admits such a dispatch only under a
/// declaration. [`tiler_ir::program::StagedRealization`] is that declaration and
/// the assembler emits one: the producer, the consumer, the handed `[2]` value,
/// and the normalization occurrence the two jointly realize. This is where the
/// compile used to stop, as `program-assembly/realization-stage-unaccounted`.
///
/// *And the three dispatched kernels compute the normalization, bit for bit.*
/// The program's own stages are interpreted in the program's own execution
/// order, each reading what an earlier stage wrote, and the published output is
/// compared against `tiler-reference`'s evaluation of the same semantic program.
/// It is bit-exact rather than close: the reference divides by the extent rather
/// than by a reciprocal and certifies its reciprocal square root against an
/// exact rational enclosure, so a spelling that rounded a different number of
/// times disagrees in bits.
///
/// The checks that can say no: reverting `spell_staged` to answer
/// `StagedFamilyUnspellable` for both stages puts the three walls back and no
/// plan completes; dropping the `StagedRealization` the assembler pushes returns
/// `UncoveringStage` from whole-program verification and no program is built;
/// and exchanging the pass expression's two multiplies — `x * (w * r)` for
/// `w * (x * r)` — is one function in exact arithmetic and two in binary32,
/// which the bit comparison catches on this fixture.
///
/// [`account-for-a-staged-realization-stage-in-the-kernel-program`]: ../../../tickets/account-for-a-staged-realization-stage-in-the-kernel-program.md
#[test]
fn a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit() {
    let semantic = staged_family_program();
    let product = compile(staged_rms_request(&semantic)).expect("the staged program compiles");
    let outcome = &product.targets[0];
    let target = outcome.compiled().unwrap_or_else(|| {
        panic!(
            "the caller-declared target compiles, but failed as {:?}",
            outcome.failure()
        )
    });
    let explain = &target.explain;
    let selected = target
        .portfolio
        .alternatives
        .iter()
        .find(|alternative| {
            alternative.stable_id == target.portfolio.selection.selected_alternative_id
        })
        .expect("the portfolio's selection names one of its alternatives");
    let core = selected.program.core();

    assert_eq!(core.stages().len(), 3);
    let declarations: Vec<_> = core.staged_realizations().collect();
    assert_eq!(declarations.len(), 1);
    let realization = declarations[0];
    assert_eq!(realization.handed().shape(), &Shape::from_dims([2]));
    assert_eq!(realization.handed().role(), ValueRole::Temporary);
    assert_eq!(
        Some(realization.producer()),
        realization.handed().definition()
    );
    assert_ne!(realization.producer(), realization.consumer());
    assert_eq!(
        realization
            .producer()
            .coverage()
            .iter()
            .map(tiler_ir::program::CoveredOccurrence::occurrence)
            .collect::<Vec<_>>(),
        vec![realization.occurrence()],
    );
    assert!(realization.consumer().coverage().is_empty());

    let stages = explain
        .records()
        .iter()
        .filter_map(|record| {
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                return None;
            };
            if assessment.predicate().as_str() != "kernel.index-region-refines-occurrence" {
                return None;
            }
            assessment
                .facts()
                .iter()
                .find(|fact| fact.key().as_str() == "realization-stages")
                .map(ExplainFact::value)
        })
        .collect::<Vec<_>>();
    assert_eq!(stages, [&FactValue::Count(2)]);

    let attributions = region_attributions(explain);
    let mut answered: Vec<&str> = attributions
        .values()
        .filter(|attribution| attribution.admitted > 0)
        .map(|attribution| attribution.role.as_str())
        .collect();
    answered.sort_unstable();
    assert_eq!(answered, ["epilogue", "staged-family", "staged-family"]);
    let walls: BTreeMap<&str, usize> = attributions
        .values()
        .filter_map(|attribution| attribution.declined_baseline.as_deref())
        .fold(BTreeMap::new(), |mut counts, reason| {
            *counts.entry(reason).or_insert(0) += 1;
            counts
        });
    assert_eq!(
        walls,
        BTreeMap::from([
            ("region-staged-family-unspellable", 1),
            ("region-partial-coverage", 2),
        ]),
    );
    assert_eq!(
        attributions
            .values()
            .filter(|attribution| attribution.role == "staged-family")
            .count(),
        3,
    );

    let shape = Shape::from_dims([2, 2]);
    let value: Vec<f32> = vec![1.0, 3.0, 7.0, 0.5];
    let weight: Vec<f32> = vec![0.25, 11.0, 0.125, 5.0];
    let published = interpret_program(core, &[("value", &value), ("weight", &weight)]);
    assert_eq!(published.len(), 1);
    let value_key = InputKey::new("value").unwrap();
    let weight_key = InputKey::new("weight").unwrap();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            &semantic,
            &[
                InputBinding::new(&value_key, &f32_tensor(shape.clone(), &value)),
                InputBinding::new(&weight_key, &f32_tensor(shape, &weight)),
            ],
        )
        .unwrap();
    assert_eq!(bits_of(&published[0]), tensor_bits(&expected[0]));
}

/// Interprets one verified program in its declared execution order.
fn interpret_program(
    program: &tiler_ir::program::VerifiedKernelProgram,
    bound: &[(&str, &[f32])],
) -> Vec<Vec<f32>> {
    use tiler_ir::program::{MaterializedOrigin, StageAccessMode};

    let values: Vec<_> = program.values().collect();
    let position = |target: tiler_ir::program::MaterializedValueRef<'_>| {
        values
            .iter()
            .position(|value| *value == target)
            .expect("every access addresses a value the program declares")
    };
    let mut contents: Vec<Option<Vec<f32>>> = vec![None; values.len()];
    for (slot, value) in values.iter().enumerate() {
        if let MaterializedOrigin::ProgramInput { key } = value.origin() {
            let payload = bound
                .iter()
                .find(|(name, _)| *name == key.as_str())
                .unwrap_or_else(|| panic!("no payload bound for program input {key:?}"));
            contents[slot] = Some(payload.1.to_vec());
        }
    }
    for stage in program.execution_order() {
        let mut reads = Vec::new();
        let mut written = None;
        for access in stage.accesses() {
            let slot = position(access.view().value());
            match access.mode() {
                StageAccessMode::Read => reads.push(
                    contents[slot]
                        .clone()
                        .expect("a stage reads a value an earlier stage wrote"),
                ),
                StageAccessMode::Write => {
                    assert!(written.replace(slot).is_none());
                }
            }
        }
        let payloads: Vec<&[f32]> = reads.iter().map(Vec::as_slice).collect();
        let produced = interpret_fused_inputs(stage.kernel(), &payloads);
        contents[written.expect("a verified stage declares its owning write")] = Some(produced);
    }
    program
        .outputs()
        .map(|output| {
            contents[position(output.value())]
                .clone()
                .expect("a published output is written by some stage")
        })
        .collect()
}

/// The two staged regions compute the normalization, bit for bit.
///
/// **The vocabulary's own measurement, and it is bit-exact rather than close.**
/// The producing stage folds `x²` over the reduced axis and applies
/// `Rsqrt(a / N + eps)` to the fold's value; the consuming stage reads both
/// operands and that handed value at its kept coordinates and writes
/// `w * (x * r)`. Interpreting the two kernels in order and comparing against
/// `tiler-reference`'s own normalization is what says the regions realize the
/// operation rather than something algebraically nearby — the reference divides
/// by the extent rather than by a reciprocal and certifies its reciprocal square
/// root against an exact rational enclosure, so a spelling that rounded a
/// different number of times disagrees in bits.
///
/// **The regions come from the compile path's own builders**, driven by the same
/// verified request `compile()` builds, and each is resubmitted through
/// `verify_schedule` — the checked path that runs intrinsic verification, the
/// numerical-realization comparison, the request-subject binding, and target
/// feasibility. So this measures what the compiler would dispatch rather than a
/// hand-built region that happens to agree.
///
/// It stops at the regions deliberately, and is kept beside the end-to-end
/// compile rather than replaced by it, because the two fail for different
/// reasons. This one fails when the *vocabulary* computes the wrong function,
/// against regions built directly from the request; the whole-program
/// measurement in
/// [`a_staged_family_program_compiles_and_computes_the_normalization_bit_for_bit`]
/// additionally fails when recognition, formation, cover selection, assembly, or
/// the program-scope declaration goes wrong, and it cannot say which. A
/// regression that reached only the region spelling would leave the compile
/// green here and red there, and one that broke only the assembler the other way
/// round.
///
/// The check that can say no: exchanging the pass expression's two multiplies —
/// `x * (w * r)` for `w * (x * r)` — is one function in exact arithmetic and two
/// in binary32, and the bit comparison fails on this fixture.
#[test]
fn the_staged_regions_compute_the_normalization_bit_for_bit() {
    let shape = Shape::from_dims([2, 2]);
    let value: Vec<f32> = vec![1.0, 3.0, 7.0, 0.5];
    let weight: Vec<f32> = vec![0.25, 11.0, 0.125, 5.0];

    let semantic = staged_norm_only_program();
    let verified = verified_staged_rms_request(&semantic);
    let request = verified.for_target(0).unwrap();
    let staged = request
        .sole_output()
        .staged()
        .expect("the declared output is the normalization occurrence");
    let (fold, fold_members) = crate::physical::staged_fold_region(
        &request,
        staged,
        crate::physical::RegionWrite::Materialized,
    );
    let fold = crate::physical::verify_schedule(
        fold,
        fold_members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the producing stage passes the checked verification path");
    let (pass, pass_members) = crate::physical::staged_pass_region(
        &request,
        staged,
        crate::physical::RegionWrite::ProgramOutput,
    );
    let pass = crate::physical::verify_schedule(
        pass,
        pass_members,
        &request,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    )
    .expect("the consuming stage passes the checked verification path");
    assert_eq!(fold.region().schedule.work_items, 2);
    assert_eq!(pass.region().schedule.work_items, 4);

    let fold_kernel = lower_structured_kernel(&fold).expect("the producing stage lowers");
    let pass_kernel = lower_structured_kernel(&pass).expect("the consuming stage lowers");
    let root = interpret_fused_inputs(&fold_kernel, &[&value]);
    let actual = interpret_fused_inputs(&pass_kernel, &[&value, &weight, &root]);

    let value_key = InputKey::new("value").unwrap();
    let weight_key = InputKey::new("weight").unwrap();
    let expected = ReferenceEvaluator::standard()
        .unwrap()
        .evaluate(
            &semantic,
            &[
                InputBinding::new(&value_key, &f32_tensor(shape.clone(), &value)),
                InputBinding::new(&weight_key, &f32_tensor(shape, &weight)),
            ],
        )
        .unwrap();
    assert_eq!(bits_of(&actual), tensor_bits(&expected[0]));
}

/// The governed profile remains silent, independently of the positive fixture.
#[test]
fn the_governed_profile_still_refuses_staged_rms_before_planning() {
    let semantic = staged_family_program();
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    assert_eq!(
        product.targets[0].failure(),
        Some(&CompileError::UnsupportedCapability(
            RequestError::UnrealizedElementaryAccuracy {
                operation: tiler_ir::semantic::rms_norm_f32_op(),
                target_profile: TargetProfile::governed().profile_key().clone(),
                reason: "accuracy.elementary.no-installed-realization",
                undischarged_half: None,
                undischarged_class: None,
                candidates: Box::new([]),
            }
        )),
    );
}

/// `rms_norm(value, weight)` over `[2, 2]` reduced on axis one, published.
///
/// The normalization as the whole declared output rather than as a chain's
/// producer, so the consuming stage writes the program output and the fixture
/// measures the two staged regions alone.
fn staged_norm_only_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), normalized)
        .unwrap();
    builder.build().unwrap()
}

/// `rms_norm(value, weight) * value` over `[2, 2]` reduced on axis one.
///
/// The extents are two so that the consuming pass fits the governed profile's
/// grid: at `[2, 4]` the epilogue region is refused by `target.grid-axis` and
/// the run would prove nothing about the staged stages.
fn staged_family_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let value = builder
        .input::<F32>(InputKey::new("value").unwrap(), shape.clone())
        .unwrap();
    let weight = builder
        .input::<F32>(InputKey::new("weight").unwrap(), shape)
        .unwrap();
    let normalized = tiler_ir::semantic::F32RmsNorm::apply(
        &mut builder,
        value,
        weight,
        Axis::new(1),
        1.0e-6_f32.to_bits(),
    )
    .unwrap();
    let scaled = F32Multiply::apply(&mut builder, normalized, value).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), scaled)
        .unwrap();
    builder.build().unwrap()
}

/// **Fourteen region subjects share one role and are fourteen explain
/// subjects.**
///
/// The role vocabulary has four values and the cover space has seventeen
/// regions, so a role-keyed trace could not name thirteen of them at all —
/// `record_frontier` was called on the first sighting of each role, and the
/// rest emitted nothing. Keying on the region's canonical occurrence makes the
/// deduplication correct rather than lossy.
///
/// The check that can say no is the key itself: reverting the subject key to
/// `region:{role}` collapses these fourteen to one and the count fails.
#[test]
fn region_subjects_sharing_a_role_are_distinct_explain_subjects() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let attributions = region_attributions(&product.targets[0].explain);

    let unrecognized: Vec<&String> = attributions
        .iter()
        .filter(|(_, attribution)| attribution.role == "unrecognized")
        .map(|(key, _)| key)
        .collect();
    assert_eq!(unrecognized.len(), 14);
    // `region_attributions` is a map keyed by the subject, so distinctness is
    // structural — but stating it is what makes the count above a claim about
    // fourteen regions rather than about fourteen records.
    let distinct: std::collections::BTreeSet<&&String> = unrecognized.iter().collect();
    assert_eq!(distinct.len(), unrecognized.len());
    // Each one covers a different occurrence set, which is why one record for
    // all of them was lossy: the declines they carry are not interchangeable.
    assert!(
        attributions
            .values()
            .filter(|attribution| attribution.role == "unrecognized")
            .any(|attribution| attribution.declined_baseline.as_deref()
                == Some("region-partial-fused-program"))
    );
}

/// **The coverage gap reaches a production reader, once per region and with the
/// cover multiplicity it replaced.**
///
/// `PlanRejection::RegionUnimplemented` has always been constructed and
/// `SelectedPortfolio::rejections()` had no caller outside `selection.rs`'s own
/// test module, so the one authority that states the gap was compiled away.
/// This drives the reader that now emits it, and checks each record is caused
/// by the frontier enumeration for its own region rather than by whatever
/// record happened to be last.
///
/// **The `blocked-covers` sum is the load-bearing assertion.** The rule used to
/// emit one record per (cover, region) pair — thirty-eight on this fixture, and
/// about 2,300 on an eleven-operation chain, which exhausted the trace's
/// canonical-byte ceiling and refused a legal program as
/// `InvalidCompilerOutput`. Fourteen records whose counts sum to thirty-eight
/// is the statement that the summary lost the repetition and kept the
/// population: a per-cover count that silently stopped counting would leave the
/// record count passing and this sum wrong.
///
/// The check that can say no is the emission: removing the `record_coverage_gaps`
/// call leaves the rejections constructed and the trace empty of them, and the
/// count below fails.
#[test]
fn the_coverage_gap_reaches_the_trace_once_per_region() {
    let semantic = semantic(false);
    let product = compile(CompilationRequest::governed(&semantic)).unwrap();
    let trace = &product.targets[0].explain;
    let attributions = region_attributions(trace);

    let gaps: Vec<&crate::explain::ExplainRecord> = trace
        .records()
        .iter()
        .filter(|record| record.rule().key().as_str() == "selection.region-coverage.v1")
        .collect();
    assert_eq!(
        gaps.len(),
        14,
        "the governed program reports one coverage gap per unimplemented region",
    );
    let mut blocked = 0_u64;
    for gap in &gaps {
        let ExplainEvent::Check { assessment, .. } = gap.event() else {
            panic!("a coverage gap is a checked predicate");
        };
        assert_eq!(
            assessment.predicate().as_str(),
            "selection.region-implemented"
        );
        assert!(
            assessment
                .reason()
                .is_some_and(|reason| reason.as_str() == "region-unimplemented")
        );
        // One subject: the region that had no implementation. The cover no
        // longer appears, because the answer is the region's own — its
        // frontier admitted nothing, whichever cover placed it — and the
        // covers it blocked are a quantity rather than a second subject.
        let subjects: Vec<&str> = gap
            .subjects()
            .iter()
            .map(|subject| subject.key().as_str())
            .collect();
        assert_eq!(subjects.len(), 1);
        let covers = assessment
            .facts()
            .iter()
            .find(|fact| fact.key().as_str() == "blocked-covers")
            .map(crate::explain::ExplainFact::value)
            .expect("a coverage gap counts the covers it blocked");
        let crate::explain::FactValue::Count(covers) = covers else {
            panic!("a blocked-cover tally is a count");
        };
        assert!(*covers > 0, "a gap that blocked no cover was not a gap");
        blocked += *covers;
        let attribution = attributions
            .get(subjects[0])
            .expect("a coverage gap names a region whose frontier was enumerated");
        assert_eq!(attribution.admitted, 0);
        assert_eq!(
            gap.causes(),
            [attribution.enumeration_tail],
            "the coverage gap is not caused by its own region's frontier enumeration",
        );
    }
    assert_eq!(
        blocked, 38,
        "the summarized records account for every (cover, region) pair the per-cover form emitted",
    );
    // The regions named are exactly the ones nothing implemented.
    let named: std::collections::BTreeSet<&str> = gaps
        .iter()
        .map(|gap| gap.subjects()[0].key().as_str())
        .collect();
    assert_eq!(named.len(), gaps.len(), "a region reported its gap twice");
    assert!(
        named
            .iter()
            .all(|key| attributions[*key].declined_baseline.is_some()),
        "a region was reported unimplemented without a decline explaining why",
    );
}
