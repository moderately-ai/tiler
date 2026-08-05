//! Out-of-crate proof that a pure-BF16 numerical contract is statable and is
//! checked against the exact BF16 arithmetic subject before any planning.
//!
//! # What this file is evidence for, and what it is not
//!
//! **It is evidence that the boundary works.** A caller can state a BF16
//! contract, it carries its own canonical identity, and a profile's BF16
//! declaration — not its `f32` one — is what answers.
//!
//! **It is not evidence that BF16 executes.** Nothing below the request
//! boundary realizes BF16: there is no capability row, no lowering capability,
//! and no scheduled-region vocabulary for it. Every positive answer here stops
//! at the recognizer's `dtype-f32` rule, which is asserted rather than avoided
//! precisely so a reader cannot mistake feasibility for support.
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
    TargetNumericalDeclaredMeans, TargetNumericalHonouredBehaviour,
    TargetNumericalRefusalDisposition, TargetNumericalRequirement, compile,
};
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetCompileProfileMeasurementSource, TargetCompilerBuild, TargetCompilerRole,
    TargetExecutionEnvironment, TargetFactProducerIdentity, TargetMeasurementContext,
    TargetProfile, TargetProfileBuilder, TargetProfileKey, TargetRequest,
};
use tiler_ir::schedule::{
    ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, F32, F32Add, F32Constant, F32Multiply, InputKey,
    OutputKey, SemanticProgram, SemanticProgramBuilder,
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

/// A flush-accepting BF16 request passes numerical feasibility and stops at the
/// next independently unsupported layer.
///
/// **That layer is the recognizer's `dtype-f32` rule, and naming it is the
/// point.** The contract resolved, the target admitted it, and the request then
/// failed for a reason that has nothing to do with numerics: this build's
/// bounded strategy recognizes `f32` programs only. Reporting anything softer
/// here would imply BF16 execution that nothing below this boundary provides.
#[test]
fn a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall() {
    let failure = compile(CompileRequest::new(
        &bf16_program(),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16,
        TargetRequest::new([profile("test.bf16-flush-accepted.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect_err("no strategy recognizes a bf16 program");
    assert_eq!(
        failure.class(),
        CompileFailureClass::UnsupportedCapability { rule: "dtype-f32" }
    );
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
/// The profile below declares a *preserving* `f32` table, so `STRICT_F32`
/// resolves on it. Stating that contract for a BF16 program therefore reaches
/// the recognizer rather than being refused numerically — the `f32` answer was
/// given to the `f32` question — and the BF16 program is still unsupported. The
/// converse is the test above it: the BF16 contract is refused by the BF16 row
/// on the very same profile whose `f32` row preserves.
#[test]
fn an_f32_contract_is_resolved_against_f32_rows_only() {
    let failure = compile(CompileRequest::new(
        &bf16_program(),
        NumericalContract::STRICT_F32,
        TargetRequest::new([profile("test.bf16-f32-contract.v1", Bf16Rows::Complete)]).unwrap(),
    ))
    .expect_err("no strategy recognizes a bf16 program");
    assert_eq!(
        failure.class(),
        CompileFailureClass::UnsupportedCapability { rule: "dtype-f32" },
        "a strict f32 contract is honoured by the f32 rows and never consults the bf16 ones",
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
