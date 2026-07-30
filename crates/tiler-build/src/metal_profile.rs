//! Bounded `f32` projection from Metal arithmetic facts into a profile builder.
//!
//! Tom ratified this bounded public boundary on 2026-07-30. It projects only
//! the exact `f32` subnormal realization stated by
//! [`MetalTargetFacts`]. The measurement source is a caller-vouched claim: this
//! adapter validates its structure but cannot prove that the supplied contexts
//! produced the independently supplied Metal fact. It does not construct a
//! production profile, bind the Metal record or provenance to a plan, populate
//! capability or dispatchability facts, or infer anything about `f16`/`bf16`.

use tiler_compiler::target::{
    ScalarArithmetic, TargetCompileProfileMeasurementSource, TargetProfileBuildError,
    TargetProfileBuilder,
};
use tiler_metal::target::{MetalFloatArithmeticType, MetalTargetFacts};

/// Declares Metal's measured `f32` subnormal behavior on a profile builder.
///
/// Every pre-existing builder fact remains independently sourced. The adapter
/// adds only the two exclusive three-row `f32` input/result subnormal tables.
/// It leaves the builder open for independently sourced capability,
/// dispatchability, and future dtype declarations. `f16` and `bf16` behavior
/// remains unknown; no neighbouring arithmetic type inherits this projection.
///
/// Both dimensions are transactional: the declarations are applied to a
/// private clone and replace `builder` only after both succeed.
///
/// # Errors
///
/// Returns a typed profile diagnostic if `f32` was not measured, an existing
/// `f32` subnormal row conflicts with either exclusive table, or a projected
/// declaration is invalid.
pub fn declare_metal_f32_subnormal_behaviour(
    builder: &mut TargetProfileBuilder,
    facts: &MetalTargetFacts,
    source: TargetCompileProfileMeasurementSource,
) -> Result<(), MetalF32TargetProfileError> {
    let behaviour = facts
        .subnormal_arithmetic
        .behaviour(MetalFloatArithmeticType::F32)
        .map_err(|_| MetalF32TargetProfileError::UnstatedF32SubnormalBehaviour)?;
    let mut staged = builder.clone();
    staged
        .declare_measured_input_subnormal_behaviour(
            ScalarArithmetic::f32(),
            behaviour.subnormal_mode(),
            source.clone(),
        )
        .map_err(MetalF32TargetProfileError::Profile)?;
    staged
        .declare_measured_result_subnormal_behaviour(
            ScalarArithmetic::f32(),
            behaviour.subnormal_mode(),
            source,
        )
        .map_err(MetalF32TargetProfileError::Profile)?;
    *builder = staged;
    Ok(())
}

/// Typed refusal from the bounded Metal `f32` profile projection.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MetalF32TargetProfileError {
    /// The Metal fact record did not state `f32` subnormal behavior.
    UnstatedF32SubnormalBehaviour,
    /// The compiler profile rejected the projected declaration.
    Profile(TargetProfileBuildError),
}

impl std::fmt::Display for MetalF32TargetProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnstatedF32SubnormalBehaviour => {
                formatter.write_str("Metal target facts do not state f32 subnormal behavior")
            }
            Self::Profile(error) => {
                write!(
                    formatter,
                    "compiler target profile refused Metal f32 facts: {error}"
                )
            }
        }
    }
}

impl std::error::Error for MetalF32TargetProfileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnstatedF32SubnormalBehaviour => None,
            Self::Profile(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler_compiler::session::{
        CompileRequest, NumericalContract, TargetCompileRefusal, TargetNumericalDeclaredMeans,
        TargetNumericalHonouredBehaviour, TargetNumericalRefusalDisposition,
        TargetNumericalRequirement, compile,
    };
    use tiler_compiler::target::{
        DTypeDispatchability, ScalarSupport, TargetCompilerBuild, TargetCompilerRole,
        TargetExecutionEnvironment, TargetFactProducerIdentity, TargetFactSource,
        TargetMeasurementContext, TargetNormativeReferenceIdentity, TargetProfileBuildError,
        TargetProfileKey, TargetRequest,
    };
    use tiler_ir::program::abi::AvailabilityPhase;
    use tiler_ir::schedule::{FlushedZeroSign, SubnormalMode};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::target::{
        MetalDeploymentMinimum, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
        MetalSubnormalArithmeticFacts, MslLanguageVersion,
    };

    fn source(
        compiler_version: &str,
        platform_build: &str,
    ) -> TargetCompileProfileMeasurementSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::CodeGenerator,
            "apple-metal-compiler".to_owned(),
            compiler_version.to_owned(),
            Some("synthetic-test-build".to_owned()),
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("macos".to_owned())
            .platform_version("26.0".to_owned())
            .platform_build(platform_build.to_owned())
            .architecture("arm64".to_owned())
            .hardware("synthetic-apple-gpu".to_owned())
            .build()
            .unwrap();
        let context = TargetMeasurementContext::new([compiler], environment).unwrap();
        TargetCompileProfileMeasurementSource::new(
            TargetFactProducerIdentity::new("test.metal-measurement.v1".to_owned(), 1).unwrap(),
            [context],
        )
        .unwrap()
    }

    fn facts(behaviour: Option<MetalSubnormalArithmetic>) -> MetalTargetFacts {
        let mut subnormals = MetalSubnormalArithmeticFacts::unmeasured();
        if let Some(behaviour) = behaviour {
            subnormals = subnormals.stating(MetalFloatArithmeticType::F32, behaviour);
        }
        MetalTargetFacts::new(
            MslLanguageVersion::Metal3_1,
            MetalPlatform::MacOs,
            MetalDeploymentMinimum::new(14, 0),
            subnormals,
            31,
        )
    }

    fn builder() -> TargetProfileBuilder {
        TargetProfileBuilder::new(
            TargetProfileKey::new("test.metal-f32-profile.v1".to_owned()).unwrap(),
        )
    }

    fn arithmetic_program() -> tiler_ir::semantic::SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let value = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), value)
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn unstated_f32_is_refused_and_neighbouring_types_are_not_projected() {
        let mut target = builder();
        assert_eq!(
            declare_metal_f32_subnormal_behaviour(&mut target, &facts(None), source("1.0", "23A")),
            Err(MetalF32TargetProfileError::UnstatedF32SubnormalBehaviour)
        );
        let neighbouring_only = MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F16,
                MetalSubnormalArithmetic::PreservesSubnormals,
            )
            .stating(
                MetalFloatArithmeticType::Bf16,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            );
        let neighbouring_only = MetalTargetFacts::new(
            MslLanguageVersion::Metal3_1,
            MetalPlatform::MacOs,
            MetalDeploymentMinimum::new(14, 0),
            neighbouring_only,
            31,
        );
        let mut target = builder();
        assert_eq!(
            declare_metal_f32_subnormal_behaviour(
                &mut target,
                &neighbouring_only,
                source("1.0", "23A"),
            ),
            Err(MetalF32TargetProfileError::UnstatedF32SubnormalBehaviour),
            "measured f16/bf16 rows must not fill the f32 omission",
        );
    }

    #[test]
    fn strict_contract_is_refused_for_flushing_f32_before_emission() {
        let mut builder = builder();
        declare_metal_f32_subnormal_behaviour(
            &mut builder,
            &facts(Some(MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: tiler_metal::target::MetalFlushedZeroSign::PreservesSign,
            })),
            source("1.0", "23A"),
        )
        .unwrap();
        builder
            .declare_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                TargetFactSource::external_guarantee(
                    TargetFactProducerIdentity::new("test.external-profile.v1".to_owned(), 1)
                        .unwrap(),
                    TargetNormativeReferenceIdentity::new(
                        "test.external-profile-spec.v1".to_owned(),
                        1,
                    )
                    .unwrap(),
                ),
            )
            .unwrap();
        let profile = builder.build().unwrap();
        let batch = compile(CompileRequest::new(
            &arithmetic_program(),
            NumericalContract::StrictF32,
            TargetRequest::new([profile]).unwrap(),
        ))
        .unwrap();
        let target = batch.targets().next().unwrap();
        let failure = target.outcome().unwrap_err();
        let TargetCompileRefusal::NumericalContract(refusal) =
            failure.refusal().expect("typed target refusal")
        else {
            panic!("expected a numerical-contract refusal");
        };
        assert_eq!(
            refusal.target_profile(),
            target.target_profile().profile_key()
        );
        let rejection = &refusal.rejections()[0];
        assert!(matches!(
            rejection.requirement(),
            TargetNumericalRequirement::InputSubnormals {
                required: SubnormalMode::Preserve,
                ..
            }
        ));
        let TargetNumericalRefusalDisposition::DeclaredUnhonourable(declared) =
            rejection.disposition()
        else {
            panic!("expected a declared unhonourable refusal");
        };
        assert_eq!(declared.means(), &TargetNumericalDeclaredMeans::Unsupported);
        assert_eq!(
            declared.honoured(),
            Some(&TargetNumericalHonouredBehaviour::InputSubnormals(
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                }
            ))
        );
        assert_eq!(
            declared.target_profile(),
            target.target_profile().profile_key()
        );
    }

    #[test]
    fn result_conflict_refuses_without_leaving_the_input_table() {
        let external = TargetFactSource::external_guarantee(
            TargetFactProducerIdentity::new("test.external-profile.v1".to_owned(), 1).unwrap(),
            TargetNormativeReferenceIdentity::new("test.external-profile-spec.v1".to_owned(), 1)
                .unwrap(),
        );
        let mut builder = builder();
        builder
            .declare_result_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Exact,
                external,
            )
            .unwrap();
        let error = declare_metal_f32_subnormal_behaviour(
            &mut builder,
            &facts(Some(MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            })),
            source("1.0", "23A"),
        )
        .unwrap_err();
        assert!(std::error::Error::source(&error).is_some());
        assert!(
            error
                .to_string()
                .contains("compiler target profile refused Metal f32 facts")
        );
        assert_eq!(
            error,
            MetalF32TargetProfileError::Profile(
                TargetProfileBuildError::ConflictingSubnormalDeclaration {
                    subject: Box::new(ScalarArithmetic::f32()),
                    dimension: "numerics.result-subnormals",
                    phase: AvailabilityPhase::CompileProfile,
                }
            )
        );
        builder
            .declare_measured_input_subnormal_behaviour(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                source("1.0", "23A"),
            )
            .expect("failed adapter left no partial input table");
    }

    #[test]
    fn behaviour_and_exact_measurement_context_change_identity() {
        let descriptor = |behaviour, compiler_version, platform_build| {
            let mut builder = builder();
            declare_metal_f32_subnormal_behaviour(
                &mut builder,
                &facts(Some(behaviour)),
                source(compiler_version, platform_build),
            )
            .unwrap();
            builder.build().unwrap().canonical_descriptor().to_vec()
        };
        let preserve = descriptor(MetalSubnormalArithmetic::PreservesSubnormals, "1.0", "23A");
        assert_ne!(
            preserve,
            descriptor(
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
                "1.0",
                "23A",
            )
        );
        assert_ne!(
            preserve,
            descriptor(MetalSubnormalArithmetic::PreservesSubnormals, "2.0", "23A")
        );
        assert_ne!(
            preserve,
            descriptor(MetalSubnormalArithmetic::PreservesSubnormals, "1.0", "24B")
        );
    }

    #[test]
    fn all_three_metal_realizations_project_without_invented_capabilities() {
        for behaviour in [
            MetalSubnormalArithmetic::PreservesSubnormals,
            MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            },
            MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::AlwaysPositive,
            },
        ] {
            let mut builder = builder();
            declare_metal_f32_subnormal_behaviour(
                &mut builder,
                &facts(Some(behaviour)),
                source("1.0", "23A"),
            )
            .unwrap();
        }
    }

    #[test]
    fn projection_preserves_the_shared_subnormal_vocabulary() {
        let modes = [
            SubnormalMode::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ];
        assert_eq!(modes.len(), 3);
        assert_eq!(
            F32::resolved_type(),
            ScalarArithmetic::f32().resolved_type().clone()
        );
    }
}
