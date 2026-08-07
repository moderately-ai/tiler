//! Out-of-crate proof that a pure-BF16 numerical contract is statable and is
//! checked against the exact BF16 arithmetic subject before any planning.
//!
//! # What this file is evidence for, and what it is not
//!
//! **It is evidence that the boundary works.** A caller can state a BF16
//! contract, it carries its own canonical identity, and a profile's BF16
//! declaration — not its `f32` one — is what answers.
//!
//! **It is now also evidence that a BF16 program is planned.** Every positive
//! answer here used to stop at the recognizer's `dtype-f32` rule, which refused
//! any program carrying a non-`f32` value before a subject was normalized;
//! `widen-the-strategy-recognizer-past-the-f32-wall` replaced that rule with a
//! derivation of the program's own arithmetic, and a lowering capability now
//! exists for each of the three registered BF16 families. So a single-occurrence
//! BF16 program reaches a selected `PlanAlternative`, which
//! `a_flush_accepting_bf16_contract_reaches_a_selected_plan` asserts.
//!
//! **It is still not evidence that BF16 *executes*.** Nothing here dispatches:
//! the run belongs to the conformance crate. And a BF16 region covering several
//! occurrences stops one layer further on, at a fusion-legality authority still
//! keyed by the `f32` operation set —
//! `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` asserts
//! that boundary rather than avoiding it, precisely so a reader cannot mistake
//! one planned shape for general support.
//!
//! # Why the profile is built here rather than imported
//!
//! The authoritative measured rows live on `FIRST_MACOS_APPLE9` in
//! `crates/tiler-build`, which depends on this crate and therefore cannot be
//! reached from its tests. The profiles below restate the *same measured
//! behaviour* — BF16 dispatchable, subnormals flushed to the sign-preserving
//! zero — under this file's own test provenance, so what they prove is that the
//! compiler boundary answers correctly for that behaviour. Binding the answer to
//! the authoritative ledger's own rows and measured source is a `tiler-build`
//! test and is tracked separately.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, NumericalContract, TargetCompileRefusal,
    TargetDTypeRefusalDisposition, TargetNumericalDeclaredMeans, TargetNumericalHonouredBehaviour,
    TargetNumericalRefusalDisposition, TargetNumericalRequirement, compile, compile_governed,
};
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetCompileProfileMeasurementSource, TargetCompilerBuild, TargetCompilerRole,
    TargetExecutionEnvironment, TargetFactProducerIdentity, TargetMeasurementContext,
    TargetProfile, TargetProfileBuilder, TargetProfileKey, TargetRequest,
};
use tiler_ir::kernel::{KernelType, lower_scheduled_region};
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ExceptionalValueAssumption, ExecutionBinding, FlushedZeroSign, InputOrdinal, KernelSchedule,
    LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder, ReductionTopology,
    RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalFreedom, SubnormalMode, TailPolicy,
    TensorRole, VerifiedScheduledRegion,
};
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, CANONICAL_BF16_ARITHMETIC_NAN_BITS, F32, F32Add,
    F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// The behaviour the measured Apple row delivers for both `f32` and `bf16`.
const SIGN_PRESERVING_FLUSH: SubnormalMode = SubnormalMode::FlushToZero {
    zero_sign: FlushedZeroSign::PreservesSign,
};

/// Which BF16 rows a profile in this file declares.
#[derive(Clone, Copy, Eq, PartialEq)]
enum Bf16Rows {
    /// Exactly what the authoritative macOS Apple9 ledger declares today: the
    /// dispatchability verdict and the two subnormal tables, and nothing else.
    MeasuredSubnormalsOnly,
    /// The subnormal tables plus every remaining dimension an admitted operation
    /// can consume, so a contract that accepts the flush has a complete answer.
    Complete,
    /// The dispatchability verdict alone, with every numerical row deleted.
    ///
    /// The perturbation subject: dispatch still succeeds, so the request reaches
    /// numerical resolution and the *numerical* declaration is the only thing
    /// missing. Dropping dispatchability too would refuse a step earlier and
    /// prove nothing about the numerical answer.
    DispatchableWithoutNumerics,
}

fn measurement() -> TargetCompileProfileMeasurementSource {
    let compiler = TargetCompilerBuild::new(
        TargetCompilerRole::CodeGenerator,
        "test-offline-compiler".to_owned(),
        "1.0".to_owned(),
        Some("build-1".to_owned()),
    )
    .unwrap();
    let environment = TargetExecutionEnvironment::builder()
        .platform("test-platform".to_owned())
        .platform_version("1.0".to_owned())
        .platform_build("build-1".to_owned())
        .architecture("test-architecture".to_owned())
        .hardware("test-hardware".to_owned())
        .build()
        .unwrap();
    TargetCompileProfileMeasurementSource::new(
        TargetFactProducerIdentity::new("test.bf16-compile-profile-probe.v1".to_owned(), 1)
            .unwrap(),
        [TargetMeasurementContext::new([compiler], environment).unwrap()],
    )
    .unwrap()
}

fn bf16_subject() -> ScalarArithmetic {
    ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())
        .expect("tiler::bf16@1 is the registered value identity of bf16 arithmetic")
}

/// Declares every dimension an admitted operation can consume, at its strict
/// resolution, for one subject.
///
/// The subnormal rows are excluded: they are the dimension under test and each
/// caller below states its own table for them.
fn declare_strict_reshaping(
    builder: &mut TargetProfileBuilder,
    subject: &ScalarArithmetic,
    source: &TargetCompileProfileMeasurementSource,
) {
    for declare in [
        TargetProfileBuilder::declare_measured_contraction,
        TargetProfileBuilder::declare_measured_reassociation,
        TargetProfileBuilder::declare_measured_permutation,
        TargetProfileBuilder::declare_measured_signed_zero,
    ] {
        declare(
            builder,
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    }
    for declare in [
        TargetProfileBuilder::declare_measured_nan_assumptions,
        TargetProfileBuilder::declare_measured_infinity_assumptions,
    ] {
        declare(
            builder,
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    }
}

/// A profile whose `f32` rows are complete and whose BF16 rows are `rows`.
///
/// The `f32` half is always complete and always *preserving*, which is what
/// makes the BF16 assertions below load-bearing: an implementation that read a
/// neighbouring width's declaration would report `Preserve` as honoured for
/// BF16 and every refusal here would disappear.
fn profile(key: &str, rows: Bf16Rows) -> TargetProfile {
    let source = measurement();
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
    builder
        .declare_measured_max_threads_per_grid_axis(65_535, source.clone())
        .unwrap();
    builder
        .declare_measured_max_threads_per_workgroup(256, source.clone())
        .unwrap();
    builder
        .declare_measured_max_buffer_bindings_per_entry(31, source.clone())
        .unwrap();
    builder
        .declare_measured_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .unwrap();
    builder
        .declare_measured_device_address_width(DeviceAddressWidth::Bits64, source.clone())
        .unwrap();
    builder
        .declare_measured_device_memory(true, source.clone())
        .unwrap();
    builder
        .declare_measured_local_memory_bytes(32_768, source.clone())
        .unwrap();

    let f32_subject = ScalarArithmetic::f32();
    builder
        .declare_measured_input_subnormal_behaviour(
            f32_subject.clone(),
            SubnormalMode::Preserve,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_measured_result_subnormal_behaviour(
            f32_subject.clone(),
            SubnormalMode::Preserve,
            source.clone(),
        )
        .unwrap();
    declare_strict_reshaping(&mut builder, &f32_subject, &source);
    builder
        .declare_measured_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source.clone(),
        )
        .unwrap();

    let subject = bf16_subject();
    builder
        .declare_measured_dtype_dispatchability(
            Bf16::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source.clone(),
        )
        .unwrap();
    if rows != Bf16Rows::DispatchableWithoutNumerics {
        // The measured behaviour, installed as a complete exclusive table: this
        // target flushes and therefore *cannot* preserve, which is the fact a
        // strict contract is refused by.
        builder
            .declare_measured_input_subnormal_behaviour(
                subject.clone(),
                SIGN_PRESERVING_FLUSH,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_result_subnormal_behaviour(
                subject.clone(),
                SIGN_PRESERVING_FLUSH,
                source.clone(),
            )
            .unwrap();
        if rows == Bf16Rows::Complete {
            declare_strict_reshaping(&mut builder, &subject, &source);
        }
    }
    builder.build().unwrap()
}

/// A pure-BF16 constant/multiply/add program.
fn bf16_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    // 1.0 and 2.0 in bf16.
    let one = Bf16Constant::apply(&mut builder, 0x3f80).unwrap();
    let two = Bf16Constant::apply(&mut builder, 0x4000).unwrap();
    let scaled = Bf16Multiply::apply(&mut builder, input, one).unwrap();
    let shifted = Bf16Add::apply(&mut builder, scaled, two).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), shifted)
        .unwrap();
    builder.build().unwrap()
}

/// The same shape of program in `f32`, as the neighbour every BF16 claim is
/// separated from.
fn f32_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let one = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let two = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, one).unwrap();
    let shifted = F32Add::apply(&mut builder, scaled, two).unwrap();
    builder
        .output(OutputKey::new("out").unwrap(), shifted)
        .unwrap();
    builder.build().unwrap()
}

/// Returns the single target's numerical refusal, or panics naming what came
/// back instead.
fn numerical_refusal(
    program: &SemanticProgram,
    contract: NumericalContract,
    profile: TargetProfile,
) -> tiler_compiler::session::TargetNumericalContractRefusal {
    let batch = compile(CompileRequest::new(
        program,
        contract,
        TargetRequest::new([profile]).unwrap(),
    ))
    .expect("a target-local numerical refusal is a batch outcome, not a request error");
    let target = batch.targets().next().expect("one requested target");
    let failure = target
        .outcome()
        .as_ref()
        .expect_err("the target refused")
        .refusal()
        .expect("a typed target refusal");
    match failure {
        TargetCompileRefusal::NumericalContract(refusal) => refusal.clone(),
        other => panic!("expected a numerical-contract refusal, got {other:?}"),
    }
}

/// A strict subnormal-preserving BF16 request is refused by the measured flush.
///
/// This is the refusal the whole boundary exists to produce, and every field is
/// asserted rather than the variant alone: the subject is `bf16` paired with
/// `tiler::bf16@1`, the requirement is the caller's `Preserve`, the disposition
/// is the profile's own declared refusal rather than silence, the declared means
/// is `Unsupported`, and the behaviour the profile *does* honour is the measured
/// sign-preserving flush.
#[test]
fn a_strict_bf16_contract_is_refused_by_the_measured_sign_preserving_flush() {
    let refusal = numerical_refusal(
        &bf16_program(),
        NumericalContract::STRICT_BF16,
        profile("test.bf16-measured-flush.v1", Bf16Rows::Complete),
    );
    let [rejection] = refusal.rejections() else {
        panic!("one stated contract, one rejection");
    };
    assert_eq!(
        rejection.contract_key(),
        NumericalContract::STRICT_BF16.key(),
        "the rejection names the exact contract the caller stated",
    );
    let TargetNumericalRequirement::InputSubnormals { subject, required } = rejection.requirement()
    else {
        panic!(
            "the canonical-first unhonourable dimension is input subnormals, got {:?}",
            rejection.requirement()
        );
    };
    assert_eq!(subject.arithmetic(), ArithmeticType::Bf16);
    assert_eq!(subject.resolved_type(), &Bf16::resolved_type());
    assert_eq!(*required, SubnormalMode::Preserve);

    let TargetNumericalRefusalDisposition::DeclaredUnhonourable(declared) = rejection.disposition()
    else {
        panic!(
            "a declared row must refuse by name, never degrade to Unknown: {:?}",
            rejection.disposition()
        );
    };
    assert_eq!(declared.subject().arithmetic(), ArithmeticType::Bf16);
    assert_eq!(declared.subject().resolved_type(), &Bf16::resolved_type());
    assert_eq!(*declared.means(), TargetNumericalDeclaredMeans::Unsupported);
    assert_eq!(
        declared.honoured(),
        Some(&TargetNumericalHonouredBehaviour::InputSubnormals(
            SIGN_PRESERVING_FLUSH
        )),
        "the refusal reports the behaviour the target measurably delivers",
    );
}

/// Deleting the BF16 declaration turns the same refusal into `Unknown`.
///
/// The perturbation that proves the assertion above can say no for the reason it
/// claims. Without it, a profile that declared nothing at all would produce a
/// refusal too, and the test would pass while proving only that BF16 is
/// unsupported somewhere.
#[test]
fn removing_the_bf16_declaration_degrades_the_refusal_to_unknown() {
    let refusal = numerical_refusal(
        &bf16_program(),
        NumericalContract::STRICT_BF16,
        profile(
            "test.bf16-undeclared.v1",
            Bf16Rows::DispatchableWithoutNumerics,
        ),
    );
    let [rejection] = refusal.rejections() else {
        panic!("one stated contract, one rejection");
    };
    assert_eq!(
        *rejection.disposition(),
        TargetNumericalRefusalDisposition::Unknown,
        "silence about bf16 is Unknown; a complete f32 table must not answer for it",
    );
    let TargetNumericalRequirement::InputSubnormals { subject, .. } = rejection.requirement()
    else {
        panic!("the reported dimension is still input subnormals");
    };
    assert_eq!(
        subject.resolved_type(),
        &Bf16::resolved_type(),
        "the unanswered question names the bf16 subject, not a substituted f32 one",
    );
}

/// The one-occurrence BF16 program `out = x + y`.
///
/// **One occurrence, and the count is the subject rather than an accident.** A
/// region covering two or more occurrences is put to `derive_fusion_legality`
/// before any cover is enumerated, and that authority is still keyed by the
/// `f32` operation set, so a multi-occurrence BF16 region is `Unknown` and every
/// cover placing it is skipped —
/// `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` is where
/// that boundary is asserted. A single-occurrence region is never put to it, so
/// this is the shape whose plan the widened recognizer can actually reach.
fn bf16_single_operation_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let left = builder
        .input::<Bf16>(InputKey::new("x").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let right = builder
        .input::<Bf16>(InputKey::new("y").unwrap(), Shape::from_dims([2, 2]))
        .unwrap();
    let sum = Bf16Add::apply(&mut builder, left, right).unwrap();
    builder.output(OutputKey::new("out").unwrap(), sum).unwrap();
    builder.build().unwrap()
}

/// A flush-accepting BF16 request is planned, and reaches a selected plan.
///
/// **This is the wall's re-foundation, and the assertion inverted.** It used to
/// assert `dtype-f32`: the contract resolved, the target admitted it, and the
/// request then failed because the recognizer refused every program carrying a
/// non-`f32` value before a subject was normalized. That rule is gone. The
/// recognizer now derives the program's one arithmetic type and admits the two
/// widths it can spell a per-point body in, so this request is recognized,
/// covered, planned, and selected — and what the assertion below states is that
/// a BF16 program reaches a `PlanAlternative`, which is the thing the rule made
/// structurally unreachable.
///
/// **What still refuses, and from whose authority, is asserted by its
/// neighbours** rather than implied here:
/// `a_strict_bf16_contract_is_refused_by_the_measured_sign_preserving_flush` for
/// the profile's declared row,
/// `an_f32_contract_does_not_answer_for_a_bf16_program` for the contract's own,
/// and `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` for
/// the layer this ticket deliberately did not widen.
#[test]
fn a_flush_accepting_bf16_contract_reaches_a_selected_plan() {
    let batch = compile(CompileRequest::new(
        &bf16_single_operation_program(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
        TargetRequest::new([profile("test.bf16-flush-accepted.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect("a planned bf16 request is a batch outcome");
    let target = batch.targets().next().expect("one requested target");
    let outcome = target.outcome();
    let compilation = outcome
        .as_ref()
        .expect("the recognizer admits a bf16 program its profile and contract support");
    assert_eq!(
        compilation.resolved_numerical_contract_key(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.key(),
        "the plan is compiled under the bf16 contract the caller stated",
    );
    // The population, counted rather than described: an enumeration that lost
    // its only alternative would leave `selected` answering about nothing.
    assert_eq!(
        compilation.alternatives().len(),
        1,
        "the bf16 program has exactly one enumerated alternative",
    );
    assert!(
        compilation.selected().is_some(),
        "a bf16 program reaches a selected plan alternative",
    );
}

/// A BF16 region covering several occurrences stops at fusion legality.
///
/// **The remaining wall, asserted where it is rather than left to be
/// discovered.** `FusionNumericalCapabilities::governed` maps the six `f32`
/// operation keys to fusion roles and nothing else, so
/// `derive_fusion_legality` resolves no capability for a `bf16` member and
/// answers `Unknown`; every cover placing that region is then skipped and the
/// selection has no complete plan. The refusal is `NoFeasiblePlan` rather than
/// an unsupported capability because a target *was* consulted and every
/// enumerated cover was ruled out.
///
/// Widening it means giving BF16 regions their own legality rather than
/// inheriting binary32's, which
/// [`establish-bf16-optimizer-legality`](../../../tickets/establish-bf16-optimizer-legality.md)
/// owns and this file deliberately does not: reassociation error is bounded by
/// the significand, and Finding 28 of the Apple numerical behaviour record
/// measures a target whose contraction behaviour differs between `f16` and
/// `bf16` — so a capability row copied from the `f32` set would be a legality
/// claim nothing proved.
///
/// The single-occurrence neighbour above is what keeps this about the *fusion*
/// boundary rather than about BF16 being unplannable.
#[test]
fn a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall() {
    let program = bf16_program();
    assert_eq!(
        program.operation_count(),
        4,
        "the fixture's region covers several occurrences, which is what puts it to fusion legality",
    );
    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
        TargetRequest::new([profile("test.bf16-fusion-wall.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect("an enumeration that rules every cover out is a target outcome, not a request error");
    let target = batch.targets().next().expect("one requested target");
    let outcome = target.outcome();
    let failure = outcome
        .as_ref()
        .expect_err("no cover survives a region whose fusion legality is unknown");
    assert_eq!(
        failure.class(),
        CompileFailureClass::NoFeasiblePlan,
        "the recognizer admitted the program and the enumeration ruled every cover out",
    );
}

/// The accepted BF16 contract's own dimensions schedule and lower a BF16 region,
/// and the request boundary now reaches one too.
///
/// **The asymmetry this test was written to record is gone, and the assertion
/// records its removal rather than being deleted.** The
/// `admit-bf16-into-the-schedule-and-kernel-vocabulary` widening made a BF16
/// scheduled region and its structured kernel constructible and verifiable and
/// did not touch `select_supported_strategy`, whose `dtype-f32` rule refused
/// every program carrying a non-`f32` value before a subject was normalized —
/// so the region vocabulary admitted a region no request could ask for. The
/// recognizer is widened and the first assertion below is its counterpart: the
/// request boundary now produces a plan for a BF16 program, so the two layers
/// meet.
///
/// The second half is unchanged and still load-bearing: the realization each
/// region carries is *derived from the accepted contract* rather than
/// transcribed, so a contract dimension that moved would move this region with
/// it instead of leaving a stale copy that still passes.
#[test]
fn the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_now_reaches() {
    // Changed: the request boundary reaches a region rather than refusing before
    // one exists.
    let batch = compile(CompileRequest::new(
        &bf16_single_operation_program(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
        TargetRequest::new([profile("test.bf16-region-wall.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect("a planned bf16 request is a batch outcome");
    assert!(
        batch
            .targets()
            .next()
            .expect("one requested target")
            .outcome()
            .as_ref()
            .is_ok_and(|compilation| compilation.selected().is_some()),
        "the recognizer's wall is gone and the request reaches the region vocabulary",
    );

    // Unchanged: the same contract's dimensions describe a region that verifies
    // and a kernel that lowers.
    for contract in [
        NumericalContract::STRICT_BF16,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
    ] {
        assert_eq!(contract.arithmetic(), ArithmeticType::Bf16);
        let region = bf16_region_under(contract);
        assert_eq!(
            region.subnormal_freedom(),
            SubnormalFreedom::Unproven,
            "nothing bounds a dense bf16 payload away from the subnormal range",
        );
        let kernel = lower_scheduled_region(&region).expect("the bf16 region lowers");
        for buffer in kernel.buffers() {
            assert_eq!(
                buffer.element_type,
                KernelType::Bf16,
                "every boundary of a bf16 region is bf16",
            );
        }
        assert_eq!(
            kernel.numerical().input_subnormals,
            contract.input_subnormals(),
            "the kernel preserves the contract's own subnormal resolution",
        );
    }
}

/// Assembles the `(x * 3.0) + (-0.0)` BF16 region under one stated contract.
///
/// Every numerical dimension is read from `contract` rather than written down,
/// so this fixture cannot drift from the accepted vector it claims to realize.
fn bf16_region_under(contract: NumericalContract) -> VerifiedScheduledRegion {
    const ELEMENTS: u64 = 4;
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(InputOrdinal::FIRST).unwrap();
    let scale = expression.constant(0x4040).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(0x8000).unwrap();
    let root = expression.add(product, bias).unwrap();
    let expression = expression.build(root).unwrap();

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([ELEMENTS]))
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Output),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: ELEMENTS,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: ELEMENTS,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::PointwiseBf16(expression))
        .unwrap();
    builder
        .numerical(NumericalRealization::new(
            "tiler.test.bf16-region",
            u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS),
            contract.input_subnormals(),
            contract.result_subnormals(),
            contract.contraction(),
            contract.reassociation(),
            contract.permutation(),
            contract.signed_zero(),
            contract.nan_assumptions(),
            contract.infinity_assumptions(),
        ))
        .unwrap();
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: ELEMENTS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: ELEMENTS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder.build().unwrap()
}

/// The authoritative ledger's BF16 rows do not yet cover every consumable
/// dimension, and the request says so exactly.
///
/// **A measurement boundary, recorded rather than papered over.** The macOS
/// Apple9 declaration states BF16 dispatchability and the two subnormal tables
/// and nothing else, so a contract that clears the subnormal dimensions still
/// meets an undeclared one. `Unknown` is the correct answer — the measurement
/// covers subnormals, not contraction — and it is asserted here so that widening
/// the ledger's rows changes this test rather than passing silently.
#[test]
fn the_measured_subnormal_rows_alone_leave_the_remaining_dimensions_unknown() {
    let refusal = numerical_refusal(
        &bf16_program(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
        profile(
            "test.bf16-subnormals-only.v1",
            Bf16Rows::MeasuredSubnormalsOnly,
        ),
    );
    let [rejection] = refusal.rejections() else {
        panic!("one stated contract, one rejection");
    };
    assert_eq!(
        *rejection.disposition(),
        TargetNumericalRefusalDisposition::Unknown
    );
    let TargetNumericalRequirement::Contraction { subject, required } = rejection.requirement()
    else {
        panic!(
            "the first undeclared consumable dimension is contraction, got {:?}",
            rejection.requirement()
        );
    };
    assert_eq!(subject.resolved_type(), &Bf16::resolved_type());
    assert_eq!(*required, NumericalPermission::Forbidden);
}

/// An `f32` contract does not answer for a BF16 program, in either direction.
///
/// **The claim is unchanged and the refusal that carries it moved, which is what
/// re-founding this assertion means.** The profile below declares a *preserving*
/// `f32` table, so `STRICT_F32` resolves on it, and the point has always been
/// that the `f32` answer was given to the `f32` question and says nothing about
/// the BF16 program. What used to catch the pairing afterwards was the
/// recognizer's `dtype-f32` rule — an accident of that rule's breadth rather
/// than a statement about contracts — and the recognizer now admits the program.
/// So the refusal is the *contract's* own: a contract's arithmetic is part of
/// its identity (ADR 0076 item 6) and a target's honourability rows are keyed by
/// subject, so no profile could be asked whether it honours an `f32` contract
/// for a `bf16` program, and the request is refused before any profile is
/// consulted.
///
/// Its converse is the test above it — the BF16 contract is refused by the BF16
/// row on the very same profile whose `f32` row preserves — and its positive
/// neighbour is `a_flush_accepting_bf16_contract_reaches_a_selected_plan`, which
/// is what keeps this about the *pairing* rather than about BF16 being
/// unplannable.
#[test]
fn an_f32_contract_does_not_answer_for_a_bf16_program() {
    let failure = compile(CompileRequest::new(
        &bf16_program(),
        NumericalContract::STRICT_F32,
        TargetRequest::new([profile("test.bf16-f32-contract.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect_err("no stated contract resolves the program's arithmetic");
    assert_eq!(
        failure.class(),
        CompileFailureClass::InvalidRequest {
            rule: "compile.request.numerics.inapplicable"
        },
        "a strict f32 contract is honoured by the f32 rows and never consults the bf16 ones",
    );
    assert!(
        failure.explain().is_none(),
        "the refusal precedes every target, so no target-qualified trace is sealed",
    );
}

/// A BF16 contract changes no existing `f32` compile outcome, and shares no key.
///
/// Two claims in one test because they are the same claim at two levels: the
/// identities are disjoint, and the behaviour is unchanged. The `f32` program
/// compiles under `STRICT_F32` on a profile that also carries complete BF16
/// rows, so the BF16 declarations are demonstrably present and demonstrably
/// inert for an `f32` request.
#[test]
fn a_bf16_contract_neither_shares_an_f32_key_nor_moves_an_f32_outcome() {
    let mut keys = vec![
        NumericalContract::STRICT_F32.key(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32.key(),
        NumericalContract::RELAXED_F32.key(),
        NumericalContract::REASSOCIATE_F32.key(),
        NumericalContract::FLUSH_AND_REASSOCIATE_F32.key(),
        NumericalContract::STRICT_BF16.key(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16.key(),
    ];
    let named = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), named, "two named contracts share one key");

    // The vectors are identical and the keys are not, which is the whole
    // separation: same dimensions, different arithmetic, different contract.
    assert_ne!(
        NumericalContract::STRICT_F32.key(),
        NumericalContract::STRICT_BF16.key()
    );
    assert_eq!(
        NumericalContract::STRICT_F32.input_subnormals(),
        NumericalContract::STRICT_BF16.input_subnormals()
    );
    assert_eq!(
        NumericalContract::STRICT_BF16.arithmetic(),
        ArithmeticType::Bf16
    );
    assert_eq!(
        NumericalContract::STRICT_F32.arithmetic(),
        ArithmeticType::F32
    );

    let batch = compile(CompileRequest::new(
        &f32_program(),
        NumericalContract::STRICT_F32,
        TargetRequest::new([profile("test.bf16-inert-for-f32.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect("the f32 request is admitted");
    let target = batch.targets().next().expect("one requested target");
    let outcome = target.outcome();
    let compilation = outcome
        .as_ref()
        .expect("a profile carrying bf16 rows still compiles an f32 program");
    assert_eq!(
        compilation.resolved_numerical_contract_key(),
        NumericalContract::STRICT_F32.key(),
        "the f32 program compiled under the f32 contract's own unchanged key",
    );
}

/// The convenience entry reports the refusal the general path reports.
///
/// **The claim is the typed detail, not the class.** `compile_governed` composes
/// the governed singleton target request and calls `compile`, so the two paths
/// below are the same compilation stated twice — and the assertion is that both
/// hand back the same `TargetCompileRefusal`, not merely the same
/// `CompileFailureClass`. A convenience entry that projected the target failure
/// onto its inner class-and-trace pair would satisfy every class assertion in
/// this file while telling a caller nothing about *which* dtype could not
/// dispatch on *which* profile.
///
/// **Why a BF16 request is what reaches it.** `TargetProfile::governed` declares
/// dispatchability for `f32` and for nothing else, so a pure-BF16 program is
/// refused at dtype dispatch before any trace is sealed — the refusal shape that
/// carries recoverable typed detail and nothing else does. Every other test in
/// this file states its own profile; this one deliberately does not, because the
/// built-in governed profile is the target `compile_governed` bakes in and the
/// entry point cannot be exercised against any other.
#[test]
fn the_governed_convenience_entry_carries_the_same_typed_refusal_as_the_general_path() {
    let program = bf16_program();

    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::STRICT_BF16,
        TargetRequest::new([TargetProfile::governed()]).unwrap(),
    ))
    .expect("a target-local dtype refusal is a batch outcome, not a request error");
    let general = batch
        .targets()
        .next()
        .expect("one requested target")
        .outcome()
        .expect_err("the governed profile dispatches f32 alone");

    let convenience = compile_governed(&program, NumericalContract::STRICT_BF16)
        .expect_err("the governed profile dispatches f32 alone");

    assert_eq!(convenience.class(), general.class());
    assert!(
        convenience.explain().is_none() && general.explain().is_none(),
        "a dtype-dispatch refusal precedes the trace boundary on both paths",
    );

    let TargetCompileRefusal::DTypeDispatch(refusal) = convenience
        .refusal()
        .expect("the convenience entry retains the pre-trace refusal detail")
    else {
        panic!(
            "the governed profile's bf16 refusal is a dispatch refusal, got {:?}",
            convenience.refusal(),
        );
    };
    assert_eq!(
        refusal.resolved_type(),
        &Bf16::resolved_type(),
        "the refusal names the exact type that could not dispatch",
    );
    assert_eq!(
        refusal.disposition(),
        TargetDTypeRefusalDisposition::Unknown,
        "the governed profile names no bf16 row at all, which is Unknown rather than Unsupported",
    );
    assert_eq!(
        refusal.target_profile(),
        TargetProfile::governed().profile_key(),
        "the refusal names the profile the convenience entry selected",
    );

    assert_eq!(
        convenience.refusal(),
        general.refusal(),
        "the two entries report one refusal, not two summaries of it",
    );
}
