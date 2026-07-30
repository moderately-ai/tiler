//! Checked compiler-plan orchestration for one Metal artifact.
//!
//! This module is the point where the otherwise independent compiler, Metal
//! emitter, AOT driver, artifact builder, and expansion cache facts meet. It
//! sequences those authorities without deriving a second identity or accepting
//! a free compilation/plan pair: [`PlanAlternative`] retains its owning
//! [`Compilation`], so target, feasibility, and provider facts always come from
//! the same checked product as the kernels and target-neutral program.
//!
//! The initial path intentionally assembles one plan, one Metal payload, no
//! deferred predicates, and no launch-time preconditions. The artifact builder
//! checks those claims against the verified program, so a widened compiler plan
//! fails closed here until this support profile is deliberately extended.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactCodecFailure, ArtifactProgramBuilder, ArtifactVerificationError,
    BackendEntryKey, BackendEntryRef, BindingKind, BindingSpec, CapabilityKey,
    CompilationEnvironment, DecodedArtifact, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, PayloadContent, PayloadId, SelectedProvider,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
    VerifiedArtifactProgram,
};
use tiler_cache::expansion::{ComposedSubject, ExpansionCache, Resolution, SubjectRefusal};
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::semantic::SemanticProgram;
use tiler_metal::diagnostic::MetalEmitError;
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::MetalTargetFacts;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{NumericalRealization, OptimizationLevel};

use crate::{
    AcceptedMetalArtifact, CompiledMetalPayload, MetalArtifactProtocolError, MetalAssemblyError,
    MetalCacheError, accept_or_publish_single_payload_metal_artifact, metal_compile_request,
    prepare_metal_payload,
};

/// Why a checked compiler plan did not produce an accepted Metal artifact.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalPlanBuildError {
    /// The checked structured kernels have no realization in the stated Metal target.
    Emission(MetalEmitError),
    /// Request derivation, AOT preparation, or emission/preparation correspondence failed.
    Preparation(MetalAssemblyError),
    /// The neutral artifact builder rejected one derived declaration.
    ArtifactBuild(ArtifactBuildError),
    /// Whole-artifact verification rejected the assembled program.
    ArtifactVerification(ArtifactVerificationError),
    /// The complete expansion-cache subject could not be composed.
    CacheSubject(SubjectRefusal),
    /// Metal compilation failed inside the cache miss closure.
    CacheCompilation(MetalAssemblyError),
    /// The verified artifact could not be encoded for publication.
    CacheEncoding(ArtifactCodecFailure),
    /// The cache's governed artifact validator rejected the produced envelope.
    CacheArtifact(ArtifactCodecFailure),
    /// A pending, produced, or cached artifact contradicted the prepared plan.
    CacheProtocol(MetalArtifactProtocolError),
}

/// One cache acceptance paired with the producer-side verified artifact.
///
/// Cache hits decode to a validated envelope rather than a
/// [`VerifiedArtifactProgram`], because the cache does not and must not depend
/// on shared kernel IR. This build boundary still holds the checked plan and
/// semantic graph, so it reassembles the producer view from the accepted
/// carried payload and proves the two identities agree before returning.
#[derive(Debug)]
pub struct AcceptedMetalPlanArtifact {
    acceptance: AcceptedMetalArtifact,
    artifact: VerifiedArtifactProgram,
}

impl AcceptedMetalPlanArtifact {
    /// Returns the producer-side verified artifact.
    #[must_use]
    pub const fn artifact(&self) -> &VerifiedArtifactProgram {
        &self.artifact
    }

    /// Returns the cache resolution and its validated envelope.
    #[must_use]
    pub const fn resolution(&self) -> &Resolution {
        self.acceptance.resolution()
    }

    /// Returns the exact composed subject the artifact resolved under.
    #[must_use]
    pub const fn cache_subject(&self) -> &ComposedSubject {
        self.acceptance.cache_subject()
    }

    /// Consumes the result into its cache acceptance and verified artifact.
    #[must_use]
    pub fn into_parts(self) -> (AcceptedMetalArtifact, VerifiedArtifactProgram) {
        (self.acceptance, self.artifact)
    }
}

impl fmt::Display for MetalPlanBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emission(error) => write!(formatter, "Metal emission failed: {error}"),
            Self::Preparation(error) | Self::CacheCompilation(error) => error.fmt(formatter),
            Self::ArtifactBuild(error) => write!(formatter, "artifact assembly failed: {error}"),
            Self::ArtifactVerification(error) => write!(
                formatter,
                "whole-artifact verification failed: {:?}",
                error.diagnostics(),
            ),
            Self::CacheSubject(error) => {
                write!(formatter, "Metal cache subject was refused: {error}")
            }
            Self::CacheEncoding(error) => {
                write!(formatter, "Metal artifact encoding failed: {error}")
            }
            Self::CacheArtifact(error) => {
                write!(
                    formatter,
                    "expansion cache refused the generated artifact: {error}"
                )
            }
            Self::CacheProtocol(error) => error.fmt(formatter),
        }
    }
}

impl Error for MetalPlanBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emission(error) => Some(error),
            Self::Preparation(error) | Self::CacheCompilation(error) => Some(error),
            Self::ArtifactBuild(error) => Some(error),
            Self::ArtifactVerification(error) => Some(error),
            Self::CacheSubject(error) => Some(error),
            Self::CacheEncoding(error) | Self::CacheArtifact(error) => Some(error),
            Self::CacheProtocol(error) => Some(error),
        }
    }
}

/// Emits, prepares, assembles, and cache-resolves one checked Metal plan.
///
/// The plan is owner-linked: all compilation-wide facts are read through
/// [`PlanAlternative::compilation`], so callers cannot pair a plan from one
/// compilation with another compilation's target profile or provider
/// environment. `semantic` remains an explicit input because the compiler does
/// not retain the graph; the artifact builder verifies it against the plan's
/// target-neutral program before returning.
///
/// `target`, `optimization`, and `numerical` are explicit build policy. The
/// emitted target facts and prepared compiler target are carried as payload
/// obligations in addition to the compiler plan's target-profile reference;
/// neither is inferred from the other.
///
/// # Errors
///
/// Returns the exact refusing authority. Unsupported Metal lowering, an
/// unhonourable numerical realization, artifact mismatch, cache subject
/// refusal, compiler failure, and cache protocol failure remain distinct.
pub fn accept_or_publish_metal_plan(
    cache: &ExpansionCache,
    toolchain: &Toolchain,
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    target: &MetalTargetFacts,
    optimization: OptimizationLevel,
    numerical: NumericalRealization,
) -> Result<AcceptedMetalPlanArtifact, MetalPlanBuildError> {
    let kernels: Vec<_> = plan.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, target).map_err(MetalPlanBuildError::Emission)?;
    let request = metal_compile_request(&unit, optimization, numerical)
        .map_err(MetalPlanBuildError::Preparation)?;
    let prepared = toolchain
        .prepare(&request)
        .map_err(MetalAssemblyError::from)
        .map_err(MetalPlanBuildError::Preparation)?;
    let payload =
        prepare_metal_payload(&unit, prepared).map_err(MetalPlanBuildError::Preparation)?;
    let pending = assemble_artifact(semantic, plan, |builder, profile| {
        payload.push_pending(builder, profile)
    })
    .map_err(MetalPlanBuildError::from)?;

    let acceptance =
        accept_or_publish_single_payload_metal_artifact(cache, &pending, payload, |compiled| {
            assemble_artifact(semantic, plan, |builder, profile| {
                compiled.push_carried(builder, profile)
            })
        })
        .map_err(MetalPlanBuildError::from)?;
    let decoded = resolution_artifact(acceptance.resolution());
    let metadata = decoded
        .payload_metadata(0)
        .ok_or(MetalPlanBuildError::CacheProtocol(
            MetalArtifactProtocolError::MissingPayloadMetadata,
        ))?
        .clone();
    let code = decoded
        .payload_object(0)
        .ok_or(MetalPlanBuildError::CacheProtocol(
            MetalArtifactProtocolError::MissingPayloadObject,
        ))?
        .to_vec();
    let compiled = CompiledMetalPayload::from_content(PayloadContent { metadata, code });
    let artifact = assemble_artifact(semantic, plan, |builder, profile| {
        compiled.push_carried(builder, profile)
    })
    .map_err(MetalPlanBuildError::from)?;
    if artifact.canonical_identity().as_bytes() != decoded.identity().as_bytes() {
        return Err(MetalPlanBuildError::CacheProtocol(
            MetalArtifactProtocolError::ArtifactIdentity,
        ));
    }
    Ok(AcceptedMetalPlanArtifact {
        acceptance,
        artifact,
    })
}

const fn resolution_artifact(resolution: &Resolution) -> &DecodedArtifact {
    match resolution {
        Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => entry.artifact(),
        Resolution::Uncached { artifact, .. } => artifact,
    }
}

fn assemble_artifact(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    declare_payload: impl FnOnce(
        &mut ArtifactProgramBuilder,
        TargetProfileRef,
    ) -> Result<PayloadId, ArtifactBuildError>,
) -> Result<VerifiedArtifactProgram, PlanArtifactError> {
    let compilation = plan.compilation();
    let profile = target_profile(compilation)?;
    let rules = feasibility_rules(compilation)?;
    let environment = CompilationEnvironment::new(compilation.offered_providers().iter().cloned())?;
    let mut builder = ArtifactProgramBuilder::new(semantic, environment)?;

    for selected in plan.selected_capabilities() {
        builder.select_provider(SelectedProvider {
            provider: selected.provider().clone(),
            capability: CapabilityKey::new(selected.capability_key())?,
            capability_revision: selected.capability_revision(),
        })?;
    }

    let payload_id = declare_payload(&mut builder, profile.clone())?;
    let program = plan.abi().kernel_program();
    let mut entries = Vec::with_capacity(program.stages().len());
    for stage in program.stages() {
        entries.push(EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payload: payload_id,
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )?,
            },
        });
    }

    builder.push_variant(
        program,
        VariantSpec {
            target_profile: profile,
            feasibility_rules: rules,
            deferred_predicates: Vec::new(),
            entries,
        },
    )?;
    builder.build().map_err(PlanArtifactError::Verification)
}

fn target_profile(compilation: &Compilation) -> Result<TargetProfileRef, ArtifactBuildError> {
    Ok(TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())?,
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )?,
    })
}

fn feasibility_rules(
    compilation: &Compilation,
) -> Result<FeasibilityRuleSetRef, ArtifactBuildError> {
    Ok(FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())?,
        revision: compilation.feasibility_rule_set_revision(),
    })
}

#[derive(Debug)]
enum PlanArtifactError {
    Build(ArtifactBuildError),
    Verification(ArtifactVerificationError),
}

impl From<ArtifactBuildError> for PlanArtifactError {
    fn from(error: ArtifactBuildError) -> Self {
        Self::Build(error)
    }
}

impl From<PlanArtifactError> for MetalPlanBuildError {
    fn from(error: PlanArtifactError) -> Self {
        match error {
            PlanArtifactError::Build(error) => Self::ArtifactBuild(error),
            PlanArtifactError::Verification(error) => Self::ArtifactVerification(error),
        }
    }
}

impl From<MetalCacheError<PlanArtifactError>> for MetalPlanBuildError {
    fn from(error: MetalCacheError<PlanArtifactError>) -> Self {
        match error {
            MetalCacheError::Subject(error) => Self::CacheSubject(error),
            MetalCacheError::Compile(error) => Self::CacheCompilation(error),
            MetalCacheError::Assemble(error) => error.into(),
            MetalCacheError::Encode(error) => Self::CacheEncoding(error),
            MetalCacheError::CacheArtifact(error) => Self::CacheArtifact(error),
            MetalCacheError::Protocol(error) => Self::CacheProtocol(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tiler_artifact::program::StageDependencyReason;
    use tiler_cache::expansion::{ExpansionCache, Resolution};
    use tiler_compiler::session::{NumericalContract, compile_governed};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::target::{
        LaunchIndexRealization, MetalDeploymentMinimum, MetalFloatArithmeticType,
        MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
        MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
    };
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::{NumericalRealization, OptimizationLevel};

    use super::{MetalPlanBuildError, accept_or_publish_metal_plan, resolution_artifact};

    fn semantic_program() -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([4, 3]),
            )
            .expect("the input binds");
        let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
        let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
        let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
        let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)])
            .expect("the sum applies");
        builder
            .output(
                OutputKey::new("result").expect("the output key is valid"),
                sum,
            )
            .expect("the output binds");
        builder.build().expect("the program verifies")
    }

    fn target_facts() -> MetalTargetFacts {
        MetalTargetFacts::new(
            MslLanguageVersion::Metal3_1,
            MetalPlatform::MacOs,
            MetalDeploymentMinimum::new(14, 0),
            LaunchIndexRealization::ThreadPositionInGridUInt,
            MetalSubnormalArithmeticFacts::unmeasured()
                .stating(
                    MetalFloatArithmeticType::F32,
                    MetalSubnormalArithmetic::FlushesToZero {
                        zero_sign: MetalFlushedZeroSign::PreservesSign,
                    },
                )
                .stating(
                    MetalFloatArithmeticType::F16,
                    MetalSubnormalArithmetic::PreservesSubnormals,
                ),
            31,
        )
    }

    fn scratch(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "tiler-build-plan-{label}-{}-{:?}",
            std::process::id(),
            std::thread::current().id(),
        ));
        std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
        path
    }

    fn write_executable(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::write(path, body).expect("the fake tool is writable");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("the fake tool is executable");
    }

    fn counted_toolchain(directory: &Path) -> (Toolchain, PathBuf) {
        let counter = directory.join("compiler-invocations");
        let metal = directory.join("metal");
        let metallib = directory.join("metallib");
        let launcher = directory.join("xcrun");
        write_executable(
            &metal,
            &format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = \"--version\" ]; then echo 'Metal plan-v1'; exit 0; fi\n\
                 printf 'metal\\n' >> '{}'\n\
                 while [ \"$#\" -gt 0 ]; do\n\
                   if [ \"$1\" = \"-o\" ]; then shift; printf AIR > \"$1\"; exit 0; fi\n\
                   shift\n\
                 done\n\
                 exit 1\n",
                counter.display(),
            ),
        );
        write_executable(
            &metallib,
            &format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = \"--version\" ]; then echo 'metallib plan-v1'; exit 0; fi\n\
                 printf 'metallib\\n' >> '{}'\n\
                 while [ \"$#\" -gt 0 ]; do\n\
                   if [ \"$1\" = \"-o\" ]; then shift; printf MTLBplan > \"$1\"; exit 0; fi\n\
                   shift\n\
                 done\n\
                 exit 1\n",
                counter.display(),
            ),
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
        (Toolchain::with_launcher(launcher), counter)
    }

    fn artifact_identity(resolution: &Resolution) -> Vec<u8> {
        match resolution {
            Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => {
                entry.artifact().identity().as_bytes().to_vec()
            }
            Resolution::Uncached { artifact, .. } => artifact.identity().as_bytes().to_vec(),
        }
    }

    #[test]
    fn a_checked_plan_publishes_then_hits_without_recompiling() {
        let directory = scratch("hit");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("one selected plan");
        let mut outcomes = Vec::new();

        for _ in 0..2 {
            let accepted = accept_or_publish_metal_plan(
                &cache,
                &toolchain,
                &program,
                plan,
                &target_facts(),
                OptimizationLevel::Default,
                NumericalRealization::strict_baseline(),
            )
            .expect("the checked plan resolves");
            outcomes.push(match accepted.resolution() {
                Resolution::Published { .. } => "published",
                Resolution::Hit { .. } => "hit",
                Resolution::Uncached { .. } => "uncached",
            });
        }

        assert_eq!(outcomes, ["published", "hit"]);
        assert_eq!(
            std::fs::read_to_string(counter)
                .expect("the miss wrote its counter")
                .lines()
                .count(),
            2,
            "one metal and one metallib invocation prove the hit skipped compilation",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn distinct_owner_linked_plans_produce_distinct_artifacts() {
        let directory = scratch("plan-selection");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, _counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let selected = compilation.selected().expect("one selected plan");
        let materialized = compilation
            .alternatives()
            .find(|plan| !plan.is_fused())
            .expect("one materialized plan");

        let selected = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            selected,
            &target_facts(),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
        .expect("the selected plan resolves");
        let materialized = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            materialized,
            &target_facts(),
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
        .expect("the materialized plan resolves");

        assert_ne!(
            artifact_identity(selected.resolution()),
            artifact_identity(materialized.resolution()),
            "the facade must consume the supplied checked plan rather than reselecting",
        );
        let materialized_variant = resolution_artifact(materialized.resolution())
            .variants()
            .next()
            .expect("one materialized variant");
        let order: Vec<_> = materialized_variant
            .execution_order()
            .map(|entry| entry.stage_key().to_vec())
            .collect();
        assert!(
            order.len() > 1,
            "the retained materialized plan must remain multi-stage after acceptance",
        );
        for edge in materialized_variant.stage_dependencies() {
            assert_eq!(edge.reason(), StageDependencyReason::Data);
            let predecessor = order
                .iter()
                .position(|stage| stage.as_slice() == edge.predecessor().stage_key())
                .expect("the dependency predecessor is sequenced");
            let successor = order
                .iter()
                .position(|stage| stage.as_slice() == edge.successor().stage_key())
                .expect("the dependency successor is sequenced");
            assert!(
                predecessor < successor,
                "the decoded execution order must discharge every data dependency",
            );
        }
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn emission_refuses_an_insufficient_target_before_compiler_work() {
        let directory = scratch("emission-refusal");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let compilation = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let plan = compilation.selected().expect("one selected plan");
        let mut insufficient = target_facts();
        insufficient.buffer_binding_limit = 1;

        let error = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            &insufficient,
            OptimizationLevel::Default,
            NumericalRealization::strict_baseline(),
        )
        .expect_err("a target with one binding cannot emit this plan");
        assert!(
            matches!(
                error,
                MetalPlanBuildError::Emission(ref cause)
                    if cause.rule() == "buffer-binding-limit"
            ),
            "unexpected refusal: {error:?}",
        );
        assert!(
            !counter.exists(),
            "emission failure must precede toolchain preparation and compiler work",
        );
        let _ = std::fs::remove_dir_all(directory);
    }
}
