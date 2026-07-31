//! Out-of-crate proof of the caller-declared target-profile boundary.

use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, MAX_NUMERICAL_CONTRACT_PREFERENCES, NumericalContract,
    TargetCompileRefusal, TargetDTypeRefusalDisposition, TargetDeclaredNumericalRefusal,
    TargetNumericalContractRejection, TargetNumericalDeclaredMeans,
    TargetNumericalHonouredBehaviour, TargetNumericalRefusalDisposition,
    TargetNumericalRequirement, compile,
};
use tiler_compiler::target::{
    DTypeDispatchability, DTypeDispatchabilityResolution, DeviceAddressWidth,
    IndexArithmeticSupport, MAX_TARGET_PROFILES_PER_REQUEST, MeasuredFactAuthority,
    ScalarArithmetic, ScalarSupport, TargetCompileProfileMeasurementSource, TargetCompilerBuild,
    TargetCompilerRole, TargetCompilerRoleReference, TargetExecutionEnvironment,
    TargetFactAuthority, TargetFactProducerIdentity, TargetFactSource, TargetFactValidityScope,
    TargetMeasurementContext, TargetNormativeReferenceIdentity, TargetNumericalEvidenceBasis,
    TargetProfile, TargetProfileBuildError, TargetProfileBuilder, TargetProfileKey, TargetRequest,
    TargetRequestError,
};
use tiler_ir::program::abi::AvailabilityPhase;
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

#[derive(Clone, Copy)]
enum DispatchDeclaration {
    CompileProfile(DTypeDispatchability),
    Deferred,
    Absent,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum NumericalDeclarations {
    Strict,
    Absent,
    UnsupportedPreserveWithExactFlush,
}

fn external_guarantee() -> TargetFactSource {
    TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.external-profile-producer.v1".to_owned(), 1).unwrap(),
        TargetNormativeReferenceIdentity::new("test.external-profile-spec.v1".to_owned(), 1)
            .unwrap(),
    )
}

fn deferred_measurement() -> TargetFactSource {
    let compiler = TargetCompilerBuild::new(
        TargetCompilerRole::RuntimeCompiler,
        "test-runtime-compiler".to_owned(),
        "1.0".to_owned(),
        None,
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
    let context = TargetMeasurementContext::new([compiler], environment).unwrap();
    TargetFactSource::measured(
        TargetFactProducerIdentity::new("test.runtime-probe.v1".to_owned(), 1).unwrap(),
        MeasuredFactAuthority::DeviceRuntime,
        [context],
    )
    .unwrap()
}

fn compile_profile_measurement() -> TargetCompileProfileMeasurementSource {
    measurement_on("1.0", "build-1")
}

/// Compile-profile measurement provenance for one exact build and environment.
///
/// The two parameters are the knobs a caller can turn to state that the same
/// refusal was measured somewhere else, without touching what was declared.
fn measurement_on(
    compiler_version: &str,
    platform_build: &str,
) -> TargetCompileProfileMeasurementSource {
    let compiler = TargetCompilerBuild::new(
        TargetCompilerRole::CodeGenerator,
        "test-offline-compiler".to_owned(),
        compiler_version.to_owned(),
        Some("build-1".to_owned()),
    )
    .unwrap();
    let environment = TargetExecutionEnvironment::builder()
        .platform("test-platform".to_owned())
        .platform_version("1.0".to_owned())
        .platform_build(platform_build.to_owned())
        .architecture("test-architecture".to_owned())
        .hardware("test-hardware".to_owned())
        .build()
        .unwrap();
    let context = TargetMeasurementContext::new([compiler], environment).unwrap();
    TargetCompileProfileMeasurementSource::new(
        TargetFactProducerIdentity::new("test.compile-profile-probe.v1".to_owned(), 1).unwrap(),
        [context],
    )
    .unwrap()
}

/// Declares the quantitative half every profile in this file shares.
fn declare_quantitative(builder: &mut TargetProfileBuilder, source: &TargetFactSource) {
    builder
        .declare_max_threads_per_grid_axis(65_535, source.clone())
        .unwrap();
    builder
        .declare_max_threads_per_workgroup(256, source.clone())
        .unwrap();
    builder
        .declare_max_buffer_bindings_per_entry(31, source.clone())
        .unwrap();
    builder
        .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .unwrap();
    builder
        .declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())
        .unwrap();
    builder.declare_device_memory(true, source.clone()).unwrap();
    builder
        .declare_local_memory_bytes(32_768, source.clone())
        .unwrap();
}

fn external_profile(
    key: &str,
    dispatch: DispatchDeclaration,
    numerics: NumericalDeclarations,
) -> TargetProfile {
    let source = external_guarantee();
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
    declare_quantitative(&mut builder, &source);
    if numerics != NumericalDeclarations::Absent {
        let subject = ScalarArithmetic::f32();
        builder
            .declare_input_subnormals(
                subject.clone(),
                SubnormalMode::Preserve,
                if numerics == NumericalDeclarations::UnsupportedPreserveWithExactFlush {
                    ScalarSupport::Unsupported
                } else {
                    ScalarSupport::Exact
                },
                source.clone(),
            )
            .unwrap();
        if numerics == NumericalDeclarations::UnsupportedPreserveWithExactFlush {
            builder
                .declare_input_subnormals(
                    subject.clone(),
                    SubnormalMode::FlushToZero {
                        zero_sign: FlushedZeroSign::PreservesSign,
                    },
                    ScalarSupport::Exact,
                    source.clone(),
                )
                .unwrap();
        }
        builder
            .declare_result_subnormals(
                subject.clone(),
                SubnormalMode::Preserve,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_contraction(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_reassociation(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
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
    }
    match dispatch {
        DispatchDeclaration::CompileProfile(verdict) => {
            builder
                .declare_dtype_dispatchability(F32::resolved_type(), verdict, source)
                .unwrap();
        }
        DispatchDeclaration::Deferred => {
            builder
                .declare_dtype_dispatchability(
                    F32::resolved_type(),
                    DTypeDispatchability::Dispatchable,
                    deferred_measurement(),
                )
                .unwrap();
        }
        DispatchDeclaration::Absent => {}
    }
    builder.build().unwrap()
}

fn semantic_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 1]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn measured_compile_profile_source_is_admitted_by_each_fact_family() {
    let source = compile_profile_measurement();
    let mut builder = TargetProfileBuilder::new(
        TargetProfileKey::new("test.measured-public-boundary.v1".to_owned()).unwrap(),
    );
    builder
        .declare_measured_max_threads_per_workgroup(256, source.clone())
        .unwrap();
    builder
        .declare_measured_contraction(
            ScalarArithmetic::f32(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_measured_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source,
        )
        .unwrap();
    let profile = builder.build().unwrap();
    assert!(!profile.canonical_descriptor().is_empty());
    assert_eq!(
        profile.dtype_dispatchability(&F32::resolved_type(), AvailabilityPhase::CompileProfile),
        DTypeDispatchabilityResolution::Dispatchable
    );
}

#[test]
fn an_external_profile_compiles_and_exact_dtype_refusals_remain_target_local() {
    let program = semantic_program();
    let supported = external_profile(
        "test.external-supported.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let absent = external_profile(
        "test.external-absent.v1",
        DispatchDeclaration::Absent,
        NumericalDeclarations::Strict,
    );
    let unsupported = external_profile(
        "test.external-unsupported.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Unsupported),
        NumericalDeclarations::Strict,
    );
    let deferred = external_profile(
        "test.external-deferred.v1",
        DispatchDeclaration::Deferred,
        NumericalDeclarations::Strict,
    );
    let numerical_unknown = external_profile(
        "test.external-numerical-unknown.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Absent,
    );
    let targets = TargetRequest::new([
        supported.clone(),
        absent.clone(),
        unsupported.clone(),
        deferred.clone(),
        numerical_unknown.clone(),
    ])
    .unwrap();
    let request = CompileRequest::preferring(
        &program,
        [
            NumericalContract::StrictF32,
            NumericalContract::FlushSubnormalsToZeroF32,
        ],
        targets,
    )
    .unwrap();
    let batch = compile(request).expect("target-local refusals preserve the outer request");
    let outcomes = batch.targets().collect::<Vec<_>>();
    assert_eq!(outcomes.len(), 5);
    assert_eq!(outcomes[0].target_profile(), &supported);
    assert!(outcomes[0].outcome().is_ok());

    for (index, profile, expected) in [
        (1, &absent, TargetDTypeRefusalDisposition::Unknown),
        (2, &unsupported, TargetDTypeRefusalDisposition::Unsupported),
        (
            3,
            &deferred,
            TargetDTypeRefusalDisposition::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            },
        ),
    ] {
        assert_eq!(outcomes[index].target_profile(), profile);
        let failure = outcomes[index].outcome().unwrap_err();
        let TargetCompileRefusal::DTypeDispatch(refusal) = failure
            .refusal()
            .expect("dtype refusal retains typed detail")
        else {
            panic!("expected a dtype dispatch refusal");
        };
        assert_eq!(refusal.target_profile(), profile.profile_key());
        assert_eq!(refusal.resolved_type(), &F32::resolved_type());
        assert_eq!(refusal.disposition(), expected);
        assert!(failure.explain().is_none());
    }

    assert_eq!(outcomes[4].target_profile(), &numerical_unknown);
    let numerical = outcomes[4].outcome().unwrap_err();
    let TargetCompileRefusal::NumericalContract(refusal) = numerical
        .refusal()
        .expect("numerical refusal retains typed detail")
    else {
        panic!("expected a numerical-contract refusal");
    };
    assert_eq!(refusal.target_profile(), numerical_unknown.profile_key());
    assert_eq!(refusal.rejections().len(), 2);
    assert_eq!(
        refusal
            .rejections()
            .iter()
            .map(TargetNumericalContractRejection::contract_key)
            .collect::<Vec<_>>(),
        ["tiler.strict-f32.v1", "tiler.flush-f32.v1"],
        "typed numerical detail preserves the caller's exact contract order",
    );
    assert_eq!(
        refusal.rejections()[0].disposition(),
        &TargetNumericalRefusalDisposition::Unknown
    );
    assert_eq!(
        refusal.rejections()[1].disposition(),
        &TargetNumericalRefusalDisposition::Unknown
    );
    assert!(matches!(
        refusal.rejections()[0].requirement(),
        TargetNumericalRequirement::InputSubnormals {
            required: SubnormalMode::Preserve,
            ..
        }
    ));
    assert!(matches!(
        refusal.rejections()[1].requirement(),
        TargetNumericalRequirement::InputSubnormals {
            required: SubnormalMode::FlushToZero { .. },
            ..
        }
    ));
}

#[test]
fn declared_numerical_refusal_exposes_exact_subject_means_honoured_and_profile() {
    let program = semantic_program();
    let profile = external_profile(
        "test.external-declared-refusal.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::UnsupportedPreserveWithExactFlush,
    );
    let targets = TargetRequest::new([profile.clone()]).unwrap();
    let request = CompileRequest::new(&program, NumericalContract::StrictF32, targets);
    let batch = compile(request).unwrap();
    let failure = batch.targets().next().unwrap().outcome().unwrap_err();
    assert!(std::error::Error::source(failure).is_some());
    assert!(failure.to_string().contains("target compilation refused"));
    let TargetCompileRefusal::NumericalContract(refusal) = failure.refusal().unwrap() else {
        panic!("expected a numerical refusal");
    };
    let rejection = &refusal.rejections()[0];
    let TargetNumericalRefusalDisposition::DeclaredUnhonourable(declared) = rejection.disposition()
    else {
        panic!("expected a declared refusal");
    };
    assert_eq!(
        declared.subject().arithmetic(),
        tiler_ir::schedule::ArithmeticType::F32
    );
    assert_eq!(declared.subject().resolved_type(), &F32::resolved_type());
    assert_eq!(declared.means(), &TargetNumericalDeclaredMeans::Unsupported);
    assert_eq!(
        declared.honoured(),
        Some(&TargetNumericalHonouredBehaviour::InputSubnormals(
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign
            }
        ))
    );
    assert_eq!(declared.target_profile(), profile.profile_key());
}

/// A profile that flushes input subnormals, measured on an exact build.
///
/// `declare_measured_input_subnormal_behaviour` writes the complete exclusive
/// table for that dimension, so preservation is *declared unsupported* under the
/// measured source rather than merely absent — the only shape whose refusal has
/// a fact to cite.
fn measured_flushing_profile(
    key: &str,
    measurement: TargetCompileProfileMeasurementSource,
) -> TargetProfile {
    let guarantee = external_guarantee();
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
    declare_quantitative(&mut builder, &guarantee);
    let subject = ScalarArithmetic::f32();
    builder
        .declare_measured_input_subnormal_behaviour(
            subject.clone(),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            measurement,
        )
        .unwrap();
    builder
        .declare_result_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            guarantee.clone(),
        )
        .unwrap();
    for permission in [
        TargetProfileBuilder::declare_contraction as PermissionDeclaration,
        TargetProfileBuilder::declare_reassociation,
        TargetProfileBuilder::declare_permutation,
        TargetProfileBuilder::declare_signed_zero,
    ] {
        permission(
            &mut builder,
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            guarantee.clone(),
        )
        .unwrap();
    }
    builder
        .declare_nan_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            guarantee.clone(),
        )
        .unwrap();
    builder
        .declare_infinity_assumptions(
            subject,
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            guarantee.clone(),
        )
        .unwrap();
    builder
        .declare_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            guarantee,
        )
        .unwrap();
    builder.build().unwrap()
}

type PermissionDeclaration = fn(
    &mut TargetProfileBuilder,
    ScalarArithmetic,
    NumericalPermission,
    ScalarSupport,
    TargetFactSource,
) -> Result<(), TargetProfileBuildError>;

/// Reads one declared refusal's exact measured evidence back out.
fn measured_declared_refusal(profile: &TargetProfile) -> TargetDeclaredNumericalRefusal {
    let program = semantic_program();
    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::StrictF32,
        TargetRequest::new([profile.clone()]).unwrap(),
    ))
    .unwrap();
    let failure = batch.targets().next().unwrap().outcome().unwrap_err();
    let TargetCompileRefusal::NumericalContract(refusal) = failure
        .refusal()
        .expect("a pre-trace contract refusal retains typed detail")
    else {
        panic!("expected a numerical-contract refusal");
    };
    let TargetNumericalRefusalDisposition::DeclaredUnhonourable(declared) =
        refusal.rejections()[0].disposition()
    else {
        panic!("a measured unsupported declaration is a declared refusal");
    };
    declared.as_ref().clone()
}

/// A declared refusal exposes the exact checked evidence, and provenance alone
/// moves it.
///
/// This is what separates a verdict from evidence at the public boundary. A
/// caller cannot act on "this target refuses preserved subnormals" — every
/// target that flushes says that. It can act on "this was measured by *this*
/// producer, on *this* compiler build, in *this* environment", because it can
/// compare that against its own deployment. So the refusal must carry the whole
/// of it, and two refusals measured on different builds must not be equal.
#[test]
fn a_declared_refusal_exposes_its_measured_evidence_and_provenance_alone_moves_it() {
    let baseline = measured_declared_refusal(&measured_flushing_profile(
        "test.measured-refusal.v1",
        measurement_on("1.0", "build-1"),
    ));

    assert_eq!(
        baseline.declared(),
        &TargetNumericalRequirement::InputSubnormals {
            subject: baseline.subject().clone(),
            required: SubnormalMode::Preserve,
        },
        "the refusing declaration speaks about the behaviour the caller asked for",
    );
    assert_eq!(baseline.means(), &TargetNumericalDeclaredMeans::Unsupported);
    assert_eq!(
        baseline.honoured(),
        Some(&TargetNumericalHonouredBehaviour::InputSubnormals(
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign
            }
        )),
    );

    let evidence = baseline.evidence();
    assert_eq!(evidence.available_at(), AvailabilityPhase::CompileProfile);
    assert_eq!(evidence.authority(), TargetFactAuthority::MeasuredProfile);
    assert_eq!(
        evidence.validity(),
        TargetFactValidityScope::MeasuredEnvironment
    );
    assert_eq!(
        evidence.authority_identity().key(),
        "test.compile-profile-probe.v1"
    );
    assert_eq!(evidence.authority_identity().revision(), 1);
    assert_eq!(evidence.target_profile(), baseline.target_profile());

    let TargetNumericalEvidenceBasis::Measurement { contexts } = evidence.basis() else {
        panic!("a measured declaration rests on measurement contexts");
    };
    assert_eq!(contexts.len(), 1);
    let context = contexts.get(0).unwrap();
    assert_eq!(context.compiler_builds().len(), 1);
    let build = context.compiler_builds().get(0).unwrap();
    assert_eq!(build.role(), TargetCompilerRoleReference::CodeGenerator);
    assert_eq!(build.implementation(), "test-offline-compiler");
    assert_eq!(build.version(), "1.0");
    assert_eq!(build.build(), Some("build-1"));
    let environment = context.environment();
    assert_eq!(environment.platform(), "test-platform");
    assert_eq!(environment.platform_version(), "1.0");
    assert_eq!(environment.platform_build(), "build-1");
    assert_eq!(environment.architecture(), "test-architecture");
    assert_eq!(environment.hardware(), "test-hardware");

    // Only the measurement moves. What the caller required, what the target
    // declares, and what it honours instead are unchanged, and the refusal is
    // still not equal to the one measured elsewhere.
    for (label, measurement) in [
        ("compiler build", measurement_on("2.0", "build-1")),
        ("execution environment", measurement_on("1.0", "build-2")),
    ] {
        let perturbed = measured_declared_refusal(&measured_flushing_profile(
            "test.measured-refusal.v1",
            measurement,
        ));
        assert_eq!(
            perturbed.declared(),
            baseline.declared(),
            "{label} changed the declared behaviour",
        );
        assert_eq!(
            perturbed.means(),
            baseline.means(),
            "{label} changed the declared means",
        );
        assert_ne!(perturbed, baseline, "{label} left the refusal unchanged");
    }
}

#[test]
fn successful_batch_slots_share_the_frozen_provider_set() {
    let program = semantic_program();
    let first = external_profile(
        "test.external-shared-providers-a.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let second = external_profile(
        "test.external-shared-providers-b.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let batch = compile(CompileRequest::new(
        &program,
        NumericalContract::StrictF32,
        TargetRequest::new([first, second]).unwrap(),
    ))
    .unwrap();
    let outcomes = batch.targets().collect::<Vec<_>>();
    let first = outcomes[0].outcome().unwrap().offered_providers();
    let second = outcomes[1].outcome().unwrap().offered_providers();
    assert_eq!(first, second);
    assert!(std::ptr::eq(first.as_ptr(), second.as_ptr()));
}

#[test]
fn numerical_preference_cardinality_and_uniqueness_fail_as_invalid_requests() {
    let program = semantic_program();
    let profile = external_profile(
        "test.external-preference-validation.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let targets = || TargetRequest::new([profile.clone()]).unwrap();
    let duplicate = CompileRequest::preferring(
        &program,
        [NumericalContract::StrictF32, NumericalContract::StrictF32],
        targets(),
    )
    .unwrap_err();
    assert_eq!(
        duplicate.class(),
        CompileFailureClass::InvalidRequest {
            rule: "compile.request.numerics.duplicate"
        }
    );

    let consumed = std::cell::Cell::new(0);
    let over_limit = std::iter::repeat_with(|| {
        consumed.set(consumed.get() + 1);
        NumericalContract::StrictF32
    });
    let overflow = CompileRequest::preferring(&program, over_limit, targets()).unwrap_err();
    assert_eq!(consumed.get(), MAX_NUMERICAL_CONTRACT_PREFERENCES + 1);
    assert_eq!(
        overflow.class(),
        CompileFailureClass::InvalidRequest {
            rule: "compile.request.numerics.too-many"
        }
    );
}

#[test]
fn target_request_cardinality_is_bounded_and_stops_at_the_first_excess_profile() {
    let profiles = (0..=MAX_TARGET_PROFILES_PER_REQUEST)
        .map(|index| {
            external_profile(
                &format!("test.external-cardinality-{index}.v1"),
                DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
                NumericalDeclarations::Strict,
            )
        })
        .collect::<Vec<_>>();

    let admitted =
        TargetRequest::new(profiles[..MAX_TARGET_PROFILES_PER_REQUEST].iter().cloned()).unwrap();
    assert_eq!(admitted.profiles().len(), MAX_TARGET_PROFILES_PER_REQUEST);
    assert_eq!(
        TargetRequest::new(profiles),
        Err(TargetRequestError::TooManyProfiles {
            actual: MAX_TARGET_PROFILES_PER_REQUEST + 1,
            max: MAX_TARGET_PROFILES_PER_REQUEST,
        })
    );

    let consumed = std::cell::Cell::new(0);
    let unbounded = std::iter::repeat_with(|| {
        consumed.set(consumed.get() + 1);
        TargetProfile::governed()
    });
    assert_eq!(
        TargetRequest::new(unbounded),
        Err(TargetRequestError::TooManyProfiles {
            actual: MAX_TARGET_PROFILES_PER_REQUEST + 1,
            max: MAX_TARGET_PROFILES_PER_REQUEST,
        })
    );
    assert_eq!(consumed.get(), MAX_TARGET_PROFILES_PER_REQUEST + 1);
}

#[test]
fn duplicate_target_request_error_names_the_key_and_positions() {
    let duplicate = external_profile(
        "test.external-duplicate-detail.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let distinct = external_profile(
        "test.external-duplicate-distinct.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    assert_eq!(
        TargetRequest::new([duplicate.clone(), distinct, duplicate.clone()]),
        Err(TargetRequestError::DuplicateProfile {
            profile: duplicate.profile_key().clone(),
            first: 0,
            duplicate: 2,
        })
    );
}

#[test]
fn duplicate_target_request_error_selects_the_first_two_caller_positions() {
    let duplicate = external_profile(
        "test.external-duplicate-order.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let first_distinct = external_profile(
        "test.external-duplicate-order-a.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    let second_distinct = external_profile(
        "test.external-duplicate-order-b.v1",
        DispatchDeclaration::CompileProfile(DTypeDispatchability::Dispatchable),
        NumericalDeclarations::Strict,
    );
    assert_eq!(
        TargetRequest::new([
            duplicate.clone(),
            first_distinct,
            duplicate.clone(),
            second_distinct,
            duplicate.clone(),
        ]),
        Err(TargetRequestError::DuplicateProfile {
            profile: duplicate.profile_key().clone(),
            first: 0,
            duplicate: 2,
        })
    );
}
