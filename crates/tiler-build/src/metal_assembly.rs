//! Assembly of one emitted and prepared Metal payload.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
    BackendKey, BackendPayloadDescriptor, PayloadContent, PayloadDigest, PayloadEntryMapping,
    PayloadId, PayloadMetadata, PayloadProvenance, PayloadSdkIdentity, PayloadTargetObligation,
    RepresentationKey, SchemaVersion, TargetProfileRef, ToolComponent,
};
use tiler_metal::diagnostic::MetalEmitError;
use tiler_metal::record::{MetalNumericalRequirement, MetalTranslationUnit};
use tiler_metal::target::{
    MetalDeploymentMinimum, MetalPlatform, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::diagnostic::DriverError;
use tiler_metal_aot::driver::PreparedCompilation;
use tiler_metal_aot::input::{
    ApplePlatform, CompileRequest, DeploymentMinimum, FpContract, MathMode, MetalTarget,
    MetalTargetError, MslVersion, NumericalRealization, OptimizationLevel,
};

use crate::metal_payload::{COMPILER_ROLE, LINKER_ROLE, SOURCE_REPRESENTATION, TOOLCHAIN};
use crate::{MetalPayloadMismatch, validate_prepared_metal_payload};

pub(crate) const BACKEND: &str = "tiler.metal";
pub(crate) const REPRESENTATION: &str = "metallib";
const NUMERICAL_GAP_OBLIGATION: &str = "tiler.numerical.unhonoured-gap";
const NUMERICAL_REQUIREMENT_OBLIGATION: &str = "tiler.numerical.emission-requirement";
pub(crate) const PAYLOAD_SCHEMA: SchemaVersion = SchemaVersion::new(1, 0);

/// A typed refusal while deriving, preparing, or compiling a Metal payload.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalAssemblyError {
    /// The emitted unit cannot honour its declared numerical realization.
    Emission(MetalEmitError),
    /// The selected compiler realization omits an emitted requirement.
    UnsatisfiedNumericalRequirement {
        /// The first requirement not present in the selected flags.
        requirement: MetalNumericalRequirement,
    },
    /// Apple toolchain preparation or execution failed.
    Driver(DriverError),
    /// The prepared producer facts contradict the emitted payload metadata.
    Correspondence(MetalPayloadMismatch),
    /// The target-neutral artifact model rejected a derived key or payload.
    Artifact(ArtifactBuildError),
    /// The emitted platform, deployment minimum, and language do not form a governed compiler target.
    Target(MetalTargetError),
}

impl fmt::Display for MetalAssemblyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emission(error) => write!(formatter, "Metal emission is not realizable: {error}"),
            Self::UnsatisfiedNumericalRequirement { requirement } => write!(
                formatter,
                "selected Metal compiler realization does not satisfy `{requirement}`"
            ),
            Self::Driver(error) => write!(formatter, "Metal AOT driver failed: {error}"),
            Self::Correspondence(error) => error.fmt(formatter),
            Self::Artifact(error) => write!(formatter, "artifact payload was rejected: {error}"),
            Self::Target(error) => {
                write!(formatter, "Metal compilation target was rejected: {error}")
            }
        }
    }
}

impl Error for MetalAssemblyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emission(error) => Some(error),
            Self::Driver(error) => Some(error),
            Self::Correspondence(error) => Some(error),
            Self::Artifact(error) => Some(error),
            Self::Target(error) => Some(error),
            Self::UnsatisfiedNumericalRequirement { .. } => None,
        }
    }
}

impl From<MetalEmitError> for MetalAssemblyError {
    fn from(error: MetalEmitError) -> Self {
        Self::Emission(error)
    }
}

impl From<DriverError> for MetalAssemblyError {
    fn from(error: DriverError) -> Self {
        Self::Driver(error)
    }
}

impl From<ArtifactBuildError> for MetalAssemblyError {
    fn from(error: ArtifactBuildError) -> Self {
        Self::Artifact(error)
    }
}

impl From<MetalTargetError> for MetalAssemblyError {
    fn from(error: MetalTargetError) -> Self {
        Self::Target(error)
    }
}

/// One emitted payload bound to the exact prepared compilation that may produce it.
#[derive(Debug)]
pub struct PreparedMetalPayload<'request> {
    prepared: PreparedCompilation<'request>,
    metadata: PayloadMetadata,
    digest: PayloadDigest,
}

impl<'request> PreparedMetalPayload<'request> {
    pub(crate) fn compilation_identity_bytes(&self) -> &[u8] {
        self.prepared.identity().as_bytes()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        PreparedCompilation<'request>,
        PayloadMetadata,
        PayloadDigest,
    ) {
        (self.prepared, self.metadata, self.digest)
    }

    pub(crate) const fn digest(&self) -> &PayloadDigest {
        &self.digest
    }

    /// Returns the compilation subject used for pending artifact identity.
    #[must_use]
    pub const fn metadata(&self) -> &PayloadMetadata {
        &self.metadata
    }

    /// Declares this not-yet-compiled payload in an artifact builder.
    ///
    /// # Errors
    ///
    /// Returns the artifact builder's typed insertion failure.
    pub fn push_pending(
        &self,
        builder: &mut ArtifactProgramBuilder,
        compatibility: TargetProfileRef,
    ) -> Result<PayloadId, ArtifactBuildError> {
        builder.push_payload(BackendPayloadDescriptor {
            backend: BackendKey::new(BACKEND)?,
            representation: RepresentationKey::new(REPRESENTATION)?,
            payload_schema: PAYLOAD_SCHEMA,
            compatibility,
            execution_policy: ArtifactExecutionPolicy::NativeImage,
            digest: self.digest.clone(),
        })
    }

    /// Compiles the prepared operation and binds its object to the checked metadata.
    ///
    /// # Errors
    ///
    /// Returns the AOT driver's typed compilation failure.
    pub fn compile(self) -> Result<CompiledMetalPayload, MetalAssemblyError> {
        let (prepared, metadata, _digest) = self.into_parts();
        CompiledMetalPayload::compile_prepared(prepared, metadata)
    }
}

impl CompiledMetalPayload {
    pub(crate) fn compile_prepared(
        prepared: PreparedCompilation<'_>,
        metadata: PayloadMetadata,
    ) -> Result<Self, MetalAssemblyError> {
        let compiled = prepared.compile()?;
        Ok(Self {
            content: PayloadContent {
                metadata,
                code: compiled.metallib,
            },
        })
    }
}

/// One compiled Metal object paired with its checked target-neutral metadata.
#[derive(Debug)]
pub struct CompiledMetalPayload {
    content: PayloadContent,
}

impl CompiledMetalPayload {
    /// Returns the carried payload content.
    #[must_use]
    pub const fn content(&self) -> &PayloadContent {
        &self.content
    }

    /// Carries this compiled payload in an artifact builder.
    ///
    /// # Errors
    ///
    /// Returns the artifact builder's typed insertion failure.
    pub fn push_carried(
        self,
        builder: &mut ArtifactProgramBuilder,
        compatibility: TargetProfileRef,
    ) -> Result<PayloadId, ArtifactBuildError> {
        builder.push_carried_payload(
            BackendKey::new(BACKEND)?,
            RepresentationKey::new(REPRESENTATION)?,
            PAYLOAD_SCHEMA,
            compatibility,
            ArtifactExecutionPolicy::NativeImage,
            self.content,
        )
    }
}

/// Derives the only AOT request corresponding to an emitted unit and explicit compiler choices.
///
/// # Errors
///
/// Refuses an unrealizable unit or a numerical selection that omits any emitted requirement.
pub fn metal_compile_request(
    unit: &MetalTranslationUnit,
    optimization: OptimizationLevel,
    numerical: NumericalRealization,
) -> Result<CompileRequest, MetalAssemblyError> {
    validate_numerical_selection(unit, numerical)?;
    Ok(CompileRequest::new(
        unit.source(),
        compile_target(*unit.target())?,
        optimization,
        numerical,
    ))
}

/// Binds emitted metadata to a prepared compilation before any compiler work.
///
/// # Errors
///
/// Returns a typed artifact-key failure or the exact correspondence mismatch.
pub fn prepare_metal_payload<'request>(
    unit: &MetalTranslationUnit,
    prepared: PreparedCompilation<'request>,
) -> Result<PreparedMetalPayload<'request>, MetalAssemblyError> {
    validate_numerical_selection(unit, prepared.request().numerical)?;
    let metadata = payload_metadata(unit, prepared.provenance())?;
    validate_prepared_metal_payload(&prepared, &metadata)
        .map_err(MetalAssemblyError::Correspondence)?;
    let digest = metadata.identity()?;
    Ok(PreparedMetalPayload {
        prepared,
        metadata,
        digest,
    })
}

fn validate_numerical_selection(
    unit: &MetalTranslationUnit,
    numerical: NumericalRealization,
) -> Result<(), MetalAssemblyError> {
    unit.require_declared_realization()?;
    for requirement in unit.numerical_requirements() {
        let satisfied = match requirement {
            MetalNumericalRequirement::SafeMathMode => numerical.math_mode == MathMode::Safe,
            MetalNumericalRequirement::NoFloatingPointContraction => {
                numerical.fp_contract == FpContract::Off
            }
            _ => false,
        };
        if !satisfied {
            return Err(MetalAssemblyError::UnsatisfiedNumericalRequirement {
                requirement: *requirement,
            });
        }
    }
    Ok(())
}

fn payload_metadata(
    unit: &MetalTranslationUnit,
    provenance: &tiler_metal_aot::record::ArtifactProvenance,
) -> Result<PayloadMetadata, MetalAssemblyError> {
    let mut entries = Vec::with_capacity(unit.entry_points().len());
    for entry in unit.entry_points() {
        entries.push(PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(entry.kernel_identity().as_bytes())?,
            symbol: entry.symbol().to_owned(),
            transports: entry
                .buffers()
                .iter()
                .map(|binding| binding.index())
                .collect(),
        });
    }
    entries.sort_by(|left, right| left.entry_key.as_bytes().cmp(right.entry_key.as_bytes()));

    let mut obligations: Vec<PayloadTargetObligation> = unit
        .numerical_requirements()
        .iter()
        .map(|requirement| PayloadTargetObligation {
            key: NUMERICAL_REQUIREMENT_OBLIGATION.to_owned(),
            value: requirement.to_string(),
        })
        .chain(
            unit.numerical_gaps()
                .iter()
                .map(|gap| PayloadTargetObligation {
                    key: NUMERICAL_GAP_OBLIGATION.to_owned(),
                    value: gap.rule().to_owned(),
                }),
        )
        .collect();
    obligations.sort();
    obligations.dedup();

    let target = compile_target(*unit.target())?;
    Ok(PayloadMetadata {
        source_representation: RepresentationKey::new(SOURCE_REPRESENTATION)?,
        source: unit.source().as_bytes().to_vec(),
        provenance: PayloadProvenance {
            toolchain: TOOLCHAIN.to_owned(),
            target: target.triple(),
            family: unit.target().platform.as_str().to_owned(),
            language: unit.target().language.semantic_name().to_owned(),
            deployment_major: unit.target().deployment_minimum.major(),
            deployment_minor: unit.target().deployment_minimum.minor(),
            components: vec![
                ToolComponent {
                    role: COMPILER_ROLE.to_owned(),
                    version: provenance.fingerprint.metal_version.clone(),
                },
                ToolComponent {
                    role: LINKER_ROLE.to_owned(),
                    version: provenance.fingerprint.metallib_version.clone(),
                },
            ],
            sdk: PayloadSdkIdentity {
                name: provenance.sdk.canonical_name.clone(),
                version: provenance.sdk.version.clone(),
                build: provenance.sdk.build.clone(),
            },
            compile_flags: provenance.compile_flags.clone(),
            link_flags: provenance.link_flags.clone(),
        },
        entries,
        obligations,
    })
}

fn compile_target(facts: MetalTargetFacts) -> Result<MetalTarget, MetalTargetError> {
    MetalTarget::new(
        apple_platform(facts.platform),
        deployment_minimum(facts.deployment_minimum),
        msl_version(facts.language),
    )
}

const fn apple_platform(family: MetalPlatform) -> ApplePlatform {
    match family {
        MetalPlatform::MacOs => ApplePlatform::MacOs,
        MetalPlatform::IOsDevice => ApplePlatform::IOsDevice,
        MetalPlatform::IOsSimulator => ApplePlatform::IOsSimulator,
        MetalPlatform::MacCatalyst => ApplePlatform::MacCatalyst,
        MetalPlatform::TvOsDevice => ApplePlatform::TvOsDevice,
        MetalPlatform::TvOsSimulator => ApplePlatform::TvOsSimulator,
        MetalPlatform::VisionOsDevice => ApplePlatform::VisionOsDevice,
        MetalPlatform::VisionOsSimulator => ApplePlatform::VisionOsSimulator,
        MetalPlatform::WatchOsDevice => ApplePlatform::WatchOsDevice,
        MetalPlatform::WatchOsSimulator => ApplePlatform::WatchOsSimulator,
    }
}

const fn msl_version(language: MslLanguageVersion) -> MslVersion {
    match language {
        MslLanguageVersion::Metal1_0 => MslVersion::Metal1_0,
        MslLanguageVersion::Metal1_1 => MslVersion::Metal1_1,
        MslLanguageVersion::Metal1_2 => MslVersion::Metal1_2,
        MslLanguageVersion::Metal2_0 => MslVersion::Metal2_0,
        MslLanguageVersion::Metal2_1 => MslVersion::Metal2_1,
        MslLanguageVersion::Metal2_2 => MslVersion::Metal2_2,
        MslLanguageVersion::Metal2_3 => MslVersion::Metal2_3,
        MslLanguageVersion::Metal2_4 => MslVersion::Metal2_4,
        MslLanguageVersion::Metal3_0 => MslVersion::Metal3_0,
        MslLanguageVersion::Metal3_1 => MslVersion::Metal3_1,
        MslLanguageVersion::Metal3_2 => MslVersion::Metal3_2,
        MslLanguageVersion::Metal4_0 => MslVersion::Metal4_0,
    }
}

const fn deployment_minimum(minimum: MetalDeploymentMinimum) -> DeploymentMinimum {
    DeploymentMinimum::new(minimum.major(), minimum.minor())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tiler_artifact::program::{
        ArtifactProgramBuilder, CompilationEnvironment, TargetProfileDescriptorDigest,
        TargetProfileKey, TargetProfileRef,
    };
    use tiler_ir::kernel::lower_scheduled_region;
    use tiler_ir::schedule::{
        Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
        ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
        NumericalPermission, NumericalRealization as DeclaredNumericalRealization, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId, ScalarProgram,
        ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    };
    use tiler_ir::semantic::{F32, InputKey, OutputKey, ProviderIdentity, SemanticProgramBuilder};
    use tiler_ir::shape::Shape;
    use tiler_metal::emit::emit_translation_unit;
    use tiler_metal::target::{
        LaunchIndexRealization, MetalDeploymentMinimum, MetalFloatArithmeticType,
        MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
        MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
    };
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::{
        CompileRequest, Fp32Functions, FpContract, MathMode, MetalTargetError,
        NumericalRealization, OptimizationLevel,
    };

    use super::{MetalAssemblyError, metal_compile_request, prepare_metal_payload};
    use crate::MetalPayloadFact;

    fn facts(minimum: u16) -> MetalTargetFacts {
        MetalTargetFacts::new(
            MslLanguageVersion::Metal3_1,
            MetalPlatform::MacOs,
            MetalDeploymentMinimum::new(minimum, 0),
            LaunchIndexRealization::ThreadPositionInGridUInt,
            MetalSubnormalArithmeticFacts::unmeasured(),
            31,
        )
    }

    fn unit(minimum: u16) -> tiler_metal::record::MetalTranslationUnit {
        emit_translation_unit(&[], &facts(minimum)).expect("an empty translation unit emits")
    }

    fn arithmetic_unit() -> tiler_metal::record::MetalTranslationUnit {
        arithmetic_unit_with_subnormal_arithmetic(MetalSubnormalArithmetic::PreservesSubnormals)
    }

    fn arithmetic_unit_with_subnormal_arithmetic(
        subnormal_arithmetic: MetalSubnormalArithmetic,
    ) -> tiler_metal::record::MetalTranslationUnit {
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
        builder
            .iteration_shape(Shape::from_dims([1]))
            .expect("the shape binds");
        for (tensor, mode, bounds, ownership) in [
            (TensorRole::Input, AccessMode::Read, 0, None),
            (
                TensorRole::Intermediate,
                AccessMode::Write,
                1,
                Some(OwnershipWitnessId::new(0)),
            ),
        ] {
            builder
                .push_access(Access {
                    tensor,
                    mode,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(bounds),
                    ownership,
                })
                .expect("the access binds");
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(bounds),
                    tensor,
                    kind: BoundsProofKind::LinearRange { element_count: 1 },
                })
                .expect("the bounds proof binds");
        }
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 1 },
            })
            .expect("the ownership proof binds");
        builder
            .scalar_program(ScalarProgram::MultiplyThenAdd {
                scale_bits: 1.0_f32.to_bits(),
                bias_bits: 0.0_f32.to_bits(),
                canonical_nan_bits: 0x7fc0_0000,
                contraction: false,
            })
            .expect("the scalar program binds");
        builder
            .numerical(DeclaredNumericalRealization::new(
                "tiler.test.metal-assembly",
                0x7fc0_0000,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ))
            .expect("the numerical realization binds");
        builder
            .schedule(KernelSchedule {
                binding: ExecutionBinding::GlobalLinearInvocation,
                work_items: 1,
                threads_per_workgroup: 1,
                tail: TailPolicy::Exact,
                output_owner: OwnershipWitnessId::new(0),
                reduction: ReductionTopology::None,
                launch: LaunchPlan {
                    grid_threads: 1,
                    threads_per_workgroup: 1,
                    zero_work_skips_dispatch: true,
                },
            })
            .expect("the schedule binds");
        let region = builder.build().expect("the region verifies");
        let kernel = lower_scheduled_region(&region).expect("the region lowers");
        let mut facts = facts(14);
        facts.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured()
            .stating(MetalFloatArithmeticType::F32, subnormal_arithmetic);
        emit_translation_unit(&[&kernel], &facts).expect("the arithmetic unit emits")
    }

    fn write_executable(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::write(path, body).expect("the fake tool is writable");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("the fake tool is executable");
    }

    fn toolchain(directory: &Path) -> Toolchain {
        let metal = directory.join("metal");
        let metallib = directory.join("metallib");
        let launcher = directory.join("xcrun");
        write_executable(
            &metal,
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo 'Metal assembly-v1'; exit 0; fi\n\
             while [ \"$#\" -gt 0 ]; do\n\
               if [ \"$1\" = \"-o\" ]; then shift; printf AIR > \"$1\"; exit 0; fi\n\
               shift\n\
             done\n\
             exit 1\n",
        );
        write_executable(
            &metallib,
            "#!/bin/sh\n\
             if [ \"$1\" = \"--version\" ]; then echo 'metallib assembly-v1'; exit 0; fi\n\
             while [ \"$#\" -gt 0 ]; do\n\
               if [ \"$1\" = \"-o\" ]; then shift; printf MTLBassembly > \"$1\"; exit 0; fi\n\
               shift\n\
             done\n\
             exit 1\n",
        );
        write_executable(
            &launcher,
            &format!(
                "#!/bin/sh\n\
                 shift 2\n\
                 case \"$1\" in\n\
                   --find) if [ \"$2\" = \"metal\" ]; then echo '{}'; else echo '{}'; fi ;;\n\
                   --show-sdk-path) echo /SDKs/MacOSX.sdk ;;\n\
                   --show-sdk-version) echo 26.5 ;;\n\
                   --show-sdk-build-version) echo 25F70 ;;\n\
                   *) exit 1 ;;\n\
                 esac\n",
                metal.display(),
                metallib.display(),
            ),
        );
        Toolchain::with_launcher(launcher)
    }

    fn scratch(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tiler-build-assembly-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
        path
    }

    fn artifact_builder() -> ArtifactProgramBuilder {
        let mut semantic =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = semantic
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([1]),
            )
            .expect("the input binds");
        semantic
            .output(
                OutputKey::new("output").expect("the output key is valid"),
                input,
            )
            .expect("the output binds");
        let semantic = semantic.build().expect("the semantic program verifies");
        let environment = CompilationEnvironment::new(std::iter::empty::<ProviderIdentity>())
            .expect("an empty environment is valid");
        ArtifactProgramBuilder::new(&semantic, environment)
            .expect("the artifact builder derives its subject")
    }

    fn profile() -> TargetProfileRef {
        TargetProfileRef {
            key: TargetProfileKey::new("tiler.test.metal-assembly")
                .expect("the profile key is valid"),
            descriptor: TargetProfileDescriptorDigest::from_bytes([1, 2, 3])
                .expect("the profile descriptor is valid"),
        }
    }

    #[test]
    fn one_prepared_compilation_becomes_pending_and_carried_payloads() {
        let directory = scratch("complete");
        let unit = arithmetic_unit();
        let request = metal_compile_request(
            &unit,
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
        .expect("the emitted unit has one governed compile request");
        let prepared = toolchain(&directory)
            .prepare(&request)
            .expect("the fake toolchain prepares");
        let expected_provenance = prepared.provenance().clone();
        let payload =
            prepare_metal_payload(&unit, prepared).expect("the emitted and prepared facts agree");

        let mut pending_builder = artifact_builder();
        payload
            .push_pending(&mut pending_builder, profile())
            .expect("the checked subject can be declared pending");

        let compiled = payload.compile().expect("the prepared operation compiles");
        assert_eq!(&compiled.content().code[..4], b"MTLB");
        assert_eq!(
            compiled.content().metadata.provenance.target,
            expected_provenance.target_triple,
        );
        assert_eq!(
            compiled.content().metadata.provenance.compile_flags,
            expected_provenance.compile_flags,
        );

        let mut carried_builder = artifact_builder();
        compiled
            .push_carried(&mut carried_builder, profile())
            .expect("the checked object can be carried");
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn preparation_refuses_a_source_other_than_the_emitted_unit() {
        let directory = scratch("source-mismatch");
        let unit = unit(14);
        let mut request = metal_compile_request(
            &unit,
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
        .expect("the baseline request is valid");
        request.source.push_str("// fault injection");
        let prepared = toolchain(&directory)
            .prepare(&request)
            .expect("the perturbed request prepares");
        let error = prepare_metal_payload(&unit, prepared)
            .expect_err("the perturbed source must be refused");
        assert!(matches!(
            error,
            MetalAssemblyError::Correspondence(mismatch)
                if mismatch.fact() == MetalPayloadFact::Source
        ));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn request_derivation_refuses_a_numerical_selection_missing_an_emitted_requirement() {
        let unit = arithmetic_unit();
        let relaxed =
            NumericalRealization::new(MathMode::Fast, Fp32Functions::Fast, FpContract::Fast);
        assert!(matches!(
            metal_compile_request(&unit, OptimizationLevel::Default, relaxed),
            Err(MetalAssemblyError::UnsatisfiedNumericalRequirement { .. }),
        ));
    }

    #[test]
    fn preparation_refuses_a_numerical_selection_missing_an_emitted_requirement() {
        let directory = scratch("prepared-numerical-requirement");
        let unit = arithmetic_unit();
        let request = CompileRequest::new(
            unit.source(),
            super::compile_target(*unit.target()).expect("the emitted target is valid"),
            OptimizationLevel::Default,
            NumericalRealization::new(MathMode::Fast, Fp32Functions::Fast, FpContract::Fast),
        );
        let prepared = toolchain(&directory)
            .prepare(&request)
            .expect("the deliberately mismatched request prepares");
        assert!(matches!(
            prepare_metal_payload(&unit, prepared),
            Err(MetalAssemblyError::UnsatisfiedNumericalRequirement { .. }),
        ));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn preparation_refuses_an_unrealizable_emitted_unit() {
        let directory = scratch("prepared-numerical-gap");
        let unit =
            arithmetic_unit_with_subnormal_arithmetic(MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            });
        let request = CompileRequest::new(
            unit.source(),
            super::compile_target(*unit.target()).expect("the emitted target is valid"),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        );
        let prepared = toolchain(&directory)
            .prepare(&request)
            .expect("the unrealizable unit's request still prepares");
        assert!(matches!(
            prepare_metal_payload(&unit, prepared),
            Err(MetalAssemblyError::Emission(_)),
        ));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn request_derivation_refuses_a_below_floor_target() {
        let unit = unit(13);
        assert!(matches!(
            metal_compile_request(
                &unit,
                OptimizationLevel::Default,
                NumericalRealization::strict_baseline(),
            ),
            Err(MetalAssemblyError::Target(
                MetalTargetError::DeploymentMinimumTooLow { .. }
            )),
        ));
    }

    #[test]
    fn target_conversion_preserves_every_artifact_family() {
        for platform in MetalPlatform::ALL {
            let facts = MetalTargetFacts::new(
                MslLanguageVersion::Metal4_0,
                platform,
                MetalDeploymentMinimum::new(26, 0),
                LaunchIndexRealization::ThreadPositionInGridUInt,
                MetalSubnormalArithmeticFacts::unmeasured(),
                31,
            );
            let target = super::compile_target(facts)
                .expect("MSL 4.0 is admitted for every represented family");
            assert_eq!(target.platform().as_str(), platform.as_str());
        }
    }

    #[test]
    fn target_conversion_preserves_every_semantic_language_revision() {
        for language in MslLanguageVersion::ALL {
            let platform = match language {
                MslLanguageVersion::Metal1_0
                | MslLanguageVersion::Metal1_1
                | MslLanguageVersion::Metal1_2
                | MslLanguageVersion::Metal2_0
                | MslLanguageVersion::Metal2_1
                | MslLanguageVersion::Metal2_2
                | MslLanguageVersion::Metal2_3
                | MslLanguageVersion::Metal2_4 => MetalPlatform::IOsDevice,
                MslLanguageVersion::Metal3_0
                | MslLanguageVersion::Metal3_1
                | MslLanguageVersion::Metal3_2
                | MslLanguageVersion::Metal4_0 => MetalPlatform::MacOs,
            };
            let facts = MetalTargetFacts::new(
                language,
                platform,
                MetalDeploymentMinimum::new(26, 0),
                LaunchIndexRealization::ThreadPositionInGridUInt,
                MetalSubnormalArithmeticFacts::unmeasured(),
                31,
            );
            let target = super::compile_target(facts)
                .expect("the high minimum admits every selected revision");
            assert_eq!(target.msl_version().revision(), language.revision());
        }
    }
}
