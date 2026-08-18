//! Caller-declared target profiles for staged RMS boundary tests.
//!
//! The quantitative, numerical, dispatchability, and deliberately silent
//! synchronization and cost declarations mirror `TargetProfile::governed`.
//! The only added authority is one synthetic, fully discharging realization of
//! `tiler::rms-norm-f32@1`; it is test evidence, not a Metal target claim.

use tiler_compiler::target::{
    DTypeDispatchability, ElementaryRealization, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuilder, TargetProfileKey,
};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    SubnormalMode,
};
use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, ConformanceEvidence, ConformanceEvidenceClass,
    ReferenceRoundingRule, VerifiedAccuracyContract,
};
use tiler_ir::semantic::{
    F32, builtin_scalar_value_type_facts, rms_norm_f32_op, rms_norm_f32_rsqrt_accuracy_contract,
    rms_norm_f32_rsqrt_exceptional_contract, silu_f32_exponential_reference_semantics,
};

/// Which elementary row the otherwise-identical fixture profile declares.
#[allow(
    dead_code,
    reason = "each integration fixture selects only the profile perturbations it owns"
)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum RmsRealizationFixture {
    /// The required RMS contract and two evidence halves that discharge.
    Discharging,
    /// No RMS row at all.
    Absent,
    /// A same-operation row whose reference semantics do not refine RMS.
    Unrefined,
}

fn external_source() -> TargetFactSource {
    TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.staged-rms-profile.v1".to_owned(), 1).unwrap(),
        TargetNormativeReferenceIdentity::new("test.staged-rms-fixture.v1".to_owned(), 1).unwrap(),
    )
}

fn verified(contract: &AccuracyContract) -> VerifiedAccuracyContract {
    let facts = builtin_scalar_value_type_facts(contract.result_type()).unwrap();
    contract.verify(&facts).unwrap()
}

fn discharging_evidence(scope: &str, digest: &[u8]) -> ConformanceEvidence {
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
}

fn realization(fixture: RmsRealizationFixture, source: &TargetFactSource) -> ElementaryRealization {
    let contract = match fixture {
        RmsRealizationFixture::Discharging => verified(&rms_norm_f32_rsqrt_accuracy_contract()),
        RmsRealizationFixture::Unrefined => verified(&AccuracyContract::new(
            rms_norm_f32_op(),
            vec![F32::resolved_type()],
            F32::resolved_type(),
            silu_f32_exponential_reference_semantics(),
            AccuracyContractForm::CorrectlyRounded {
                rounding: ReferenceRoundingRule::NearestTiesToEven,
            },
            rms_norm_f32_rsqrt_exceptional_contract(),
        )),
        RmsRealizationFixture::Absent => unreachable!("an absent row is not constructed"),
    };
    ElementaryRealization::new(
        &contract,
        discharging_evidence(
            "caller bound half for tiler::rms-norm-f32@1",
            b"fixture:staged-rms-bound-v1",
        ),
        discharging_evidence(
            "caller exceptional half for tiler::rms-norm-f32@1",
            b"fixture:staged-rms-exceptional-v1",
        ),
        source,
    )
    .unwrap()
}

/// Builds the caller profile used by staged RMS boundary tests.
pub(crate) fn staged_rms_profile(fixture: RmsRealizationFixture) -> TargetProfile {
    let source = external_source();
    let key = match fixture {
        RmsRealizationFixture::Discharging => "test.staged-rms-discharging.v1",
        RmsRealizationFixture::Absent => "test.staged-rms-absent.v1",
        RmsRealizationFixture::Unrefined => "test.staged-rms-unrefined.v1",
    };
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());

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
        .declare_reciprocal_transform(
            subject.clone(),
            NumericalPermission::Permitted,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_approximate_intrinsics(
            subject.clone(),
            ApproximationEnvelope::BackendElementary,
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

    if !matches!(fixture, RmsRealizationFixture::Absent) {
        builder
            .declare_elementary_realization(realization(fixture, &source))
            .unwrap();
    }
    builder.build().unwrap()
}
