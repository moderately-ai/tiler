//! Checked compiler-plan orchestration for one Metal artifact.
//!
//! This module is the point where the otherwise independent compiler, Metal
//! emitter, AOT driver, artifact builder, and expansion cache facts meet. It
//! sequences those authorities without deriving a second identity or accepting
//! a free compilation/plan pair: [`PlanAlternative`] retains its owning
//! [`Compilation`], so target, feasibility, and provider facts always come from
//! the same checked product as the kernels and target-neutral program.
//!
//! The path assembles one plan and one Metal payload **per declared artifact
//! family**, preserves every compiler-minted prepared-entry target requirement,
//! and declares no launch-time preconditions. The artifact builder mints each
//! requirement's ABI predicate, so this layer never reconstructs comparison
//! direction.
//!
//! # Several families are one compilation, not several artifacts
//!
//! The artifact family is not a compiler-profile axis: the authority ledger
//! records `MetalTargetFacts::platform` as backend-only, so two declarations
//! differing only in platform share a profile key and a byte-identical
//! canonical descriptor. What differs is the emitted unit's target facts and the
//! AOT triple it compiles for. So the declaration run below is one compilation,
//! one selected plan, one kernel program, and one compiled object per family,
//! carried into one envelope at one delivery position each.
//!
//! The order the caller states is the delivery order, preserved end to end and
//! never re-derived here. Whether the object at position `p` really is the one
//! declaration `p` prepared is not assumed either: the cache seam compares each
//! position's decoded metadata against the compilation prepared for it, and two
//! objects placed the other way round disagree on the AOT target triple.

use std::error::Error;
use std::fmt;

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactCodecFailure, ArtifactProgramBuilder, ArtifactVerificationError,
    BindingKind, DecodedArtifact, PayloadContent, PayloadId, TargetProfileRef,
    VerifiedArtifactProgram,
};
use tiler_cache::expansion::{ComposedSubject, ExpansionCache, Resolution, SubjectRefusal};
use tiler_compiler::session::PlanAlternative;
use tiler_ir::program::StageRef;
use tiler_ir::semantic::SemanticProgram;
use tiler_metal::diagnostic::MetalEmitError;
use tiler_metal::emit::emit_translation_unit;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::OptimizationLevel;

use crate::{
    AcceptedArtifact, BackendEntryDeclaration, BoundMetalCompileDeclaration, CompiledMetalPayload,
    MetalArtifactProtocolError, MetalAssemblyError, MetalCacheError, MetalPlanProfileMismatch,
    PlanArtifactError, accept_or_publish_delivered_metal_artifact, assemble_plan_artifact,
    metal_compile_request, prepare_metal_payload,
};

/// Why a checked compiler plan did not produce an accepted Metal artifact.
#[derive(Debug)]
#[non_exhaustive]
pub enum MetalPlanBuildError {
    /// No artifact family was declared, so there is nothing to compile for.
    ///
    /// One selection produces one envelope carrying one payload per built
    /// family, and an envelope carrying none has no entry any consumer could
    /// dispatch. Refused here rather than by the artifact builder so the reason
    /// names the selection a caller wrote instead of the empty realization it
    /// produced.
    NoDeclaredFamily,
    /// The plan was not compiled under the bound declaration's target profile.
    ///
    /// First in this enum because it is checked first: a plan assessed against
    /// another profile must not reach Metal emission at all, since everything
    /// emission then decides — binding capacity, numerical realizability, launch
    /// geometry — was decided against a target the artifact will not declare.
    DeclaredProfile(MetalPlanProfileMismatch),
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
    acceptance: AcceptedArtifact,
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

    /// Returns the validated artifact the cache resolution carries.
    ///
    /// The decoded envelope rather than [`Self::artifact`]: what a loading host
    /// receives is bytes, and the two are proven to name one identity before
    /// this value exists.
    #[must_use]
    pub const fn decoded(&self) -> &DecodedArtifact {
        self.acceptance.decoded()
    }

    /// Consumes the result into its cache acceptance and verified artifact.
    #[must_use]
    pub fn into_parts(self) -> (AcceptedArtifact, VerifiedArtifactProgram) {
        (self.acceptance, self.artifact)
    }
}

impl fmt::Display for MetalPlanBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDeclaredFamily => formatter.write_str(
                "no Metal artifact family was declared, so one plan has nothing to compile for",
            ),
            Self::DeclaredProfile(error) => error.fmt(formatter),
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
            Self::NoDeclaredFamily => None,
            Self::DeclaredProfile(error) => Some(error),
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
/// # The declared profile is verified before any Metal work
///
/// `declaration` supplies the Metal target facts, the selected emission
/// realization, and the numerical realization its measured rows are scoped to —
/// and, first, the compiler profile the plan must have been compiled under.
/// That check runs before [`emit_translation_unit`], because emission decides
/// binding capacity and numerical realizability against those facts, and a plan
/// assessed under a different profile would have those questions answered
/// against a target its artifact will not declare.
///
/// Target facts and the selected source-level realization remain separate:
/// choosing a launch-index declaration establishes no arithmetic width, address
/// width, or launch capacity, and
/// [`BoundMetalCompileDeclaration`]'s own documentation carries the three
/// compile-fail proofs of that. The translation unit retains the selected
/// emission realization, and its exact emitted source becomes part of payload
/// and compilation identity.
///
/// `optimization` stays a separate argument rather than joining the
/// declaration, because no ledger row is scoped to an optimization level: the
/// numerical rows are isolated by the emitted fast-math attributes, which the
/// `-O` level does not change.
///
/// # Errors
///
/// Returns the exact refusing authority. A declared-profile mismatch,
/// unsupported Metal lowering, an unhonourable numerical realization, artifact
/// mismatch, cache subject refusal, compiler failure, and cache protocol
/// failure remain distinct.
pub fn accept_or_publish_metal_plan(
    cache: &ExpansionCache,
    toolchain: &Toolchain,
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    declarations: &[BoundMetalCompileDeclaration],
    optimization: OptimizationLevel,
) -> Result<AcceptedMetalPlanArtifact, MetalPlanBuildError> {
    if declarations.is_empty() {
        return Err(MetalPlanBuildError::NoDeclaredFamily);
    }
    let compilation = plan.compilation();
    let kernels: Vec<_> = plan.kernels().iter().collect();
    // Every declaration is checked against the plan's profile before *any* of
    // them emits, so a selection with one wrong family costs no compiler work
    // rather than one family's worth.
    for declaration in declarations {
        declaration
            .require_compiled_under(
                compilation.target_profile_key(),
                compilation.target_profile_descriptor(),
            )
            .map_err(MetalPlanBuildError::DeclaredProfile)?;
    }
    // One emitted unit and one prepared compilation per delivery position, in
    // the caller's stated order. The units are retained because
    // `prepare_metal_payload` borrows one and the prepared token borrows its
    // request; keeping them alive here is what lets the whole run be prepared
    // before any of it is compiled.
    let mut units = Vec::with_capacity(declarations.len());
    for declaration in declarations {
        units.push(
            emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
                .map_err(MetalPlanBuildError::Emission)?,
        );
    }
    let mut requests = Vec::with_capacity(declarations.len());
    for (declaration, unit) in declarations.iter().zip(&units) {
        requests.push(
            metal_compile_request(unit, optimization, declaration.numerical_realization())
                .map_err(MetalPlanBuildError::Preparation)?,
        );
    }
    let mut payloads = Vec::with_capacity(declarations.len());
    for (unit, request) in units.iter().zip(&requests) {
        let prepared = toolchain
            .prepare(request)
            .map_err(MetalAssemblyError::from)
            .map_err(MetalPlanBuildError::Preparation)?;
        payloads
            .push(prepare_metal_payload(unit, prepared).map_err(MetalPlanBuildError::Preparation)?);
    }

    let pending = assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| {
            payloads
                .iter()
                .map(|payload| payload.push_pending(builder, profile.clone()))
                .collect()
        },
        |_, stage| Ok(metal_entry_declaration(stage)),
    )
    .map_err(MetalPlanBuildError::from)?;

    let acceptance =
        accept_or_publish_delivered_metal_artifact(cache, &pending, payloads, |compiled| {
            assemble_plan_artifact(
                semantic,
                plan,
                |builder, profile| carry_all(builder, &profile, compiled),
                |_, stage| Ok(metal_entry_declaration(stage)),
            )
        })
        .map_err(MetalPlanBuildError::from)?;
    // Read back in delivery order, which the descriptor table does not carry:
    // its order is canonical content order, so the payload a consumer at
    // position `p` loads is only nameable through the artifact's own entries.
    let decoded = acceptance.decoded();
    let mut carried = Vec::with_capacity(declarations.len());
    for delivery in 0..declarations.len() {
        let descriptor = decoded
            .variants()
            .next()
            .and_then(|variant| variant.entries().next())
            .and_then(|entry| entry.payload(delivery))
            .ok_or(MetalPlanBuildError::CacheProtocol(
                MetalArtifactProtocolError::DeliveryPositions {
                    expected: declarations.len(),
                    actual: decoded.delivery_positions(),
                },
            ))?;
        let metadata = decoded
            .payload_metadata(descriptor)
            .ok_or(MetalPlanBuildError::CacheProtocol(
                MetalArtifactProtocolError::MissingPayloadMetadata { delivery },
            ))?
            .clone();
        let code = decoded
            .payload_object(descriptor)
            .ok_or(MetalPlanBuildError::CacheProtocol(
                MetalArtifactProtocolError::MissingPayloadObject { delivery },
            ))?
            .to_vec();
        carried.push(CompiledMetalPayload::from_content(PayloadContent {
            metadata,
            code,
        }));
    }
    let artifact = assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| carry_all(builder, &profile, carried),
        |_, stage| Ok(metal_entry_declaration(stage)),
    )
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

/// Declares one compiled payload per delivery position, in the order supplied.
///
/// The order is the whole content of the delivery contract, so it is preserved
/// by consuming the run rather than by keying it: a caller states which family
/// it built first and this places that object at position 0.
fn carry_all(
    builder: &mut ArtifactProgramBuilder,
    profile: &TargetProfileRef,
    compiled: Vec<CompiledMetalPayload>,
) -> Result<Vec<PayloadId>, ArtifactBuildError> {
    compiled
        .into_iter()
        .map(|payload| payload.push_carried(builder, profile.clone()))
        .collect()
}

/// The three launch statements the standard Metal backend makes per stage.
///
/// Every Metal buffer parameter is a buffer binding, a zero-thread Metal
/// dispatch is skippable, and this path declares no launch-time precondition —
/// the workgroup bound it depends on is a *deferred* predicate the compiler
/// minted, which the neutral facade carries from the plan and no backend
/// restates. These are exactly the three statements
/// [ADR 0090](../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
/// item 11 names as not yet neutral; they are now this backend's answer rather
/// than the orchestrator's assumption.
///
/// It takes neither the builder nor a failure path, which is itself the
/// statement: this backend mints no expression of its own and has nothing to
/// refuse here, so the three answers are a total function of the stage.
fn metal_entry_declaration(stage: StageRef<'_>) -> BackendEntryDeclaration {
    BackendEntryDeclaration {
        bindings: stage.accesses().map(|_| BindingKind::Buffer).collect(),
        zero_work_skips_dispatch: true,
        preconditions: Vec::new(),
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

    use tiler_artifact::program::{
        DecodedArtifact, DigestAlgorithm, StageDependencyReason, VerifiedArtifactProgram,
    };
    use tiler_cache::expansion::{ExpansionCache, Resolution};
    use tiler_compiler::session::{
        Compilation, CompileRequest, NumericalContract, PlanAlternative, compile, compile_governed,
    };
    use tiler_compiler::target::TargetRequest;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::emit::emit_translation_unit;
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::OptimizationLevel;

    use super::{MetalPlanBuildError, accept_or_publish_metal_plan, metal_entry_declaration};
    use crate::{
        BoundMetalCompileDeclaration, CompiledMetalPayload, MetalArtifactProtocolError,
        MetalCacheError, MetalPlanProfileMismatch, PreparedMetalPayload,
        accept_or_publish_delivered_metal_artifact, assemble_plan_artifact, metal_compile_request,
        prepare_metal_payload,
    };

    fn semantic_program() -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([2, 2]),
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

    fn declaration() -> BoundMetalCompileDeclaration {
        BoundMetalCompileDeclaration::first_macos_apple9()
            .expect("the authoritative macOS declaration assembles")
    }

    /// Compiles the proof program against the authoritative declaration.
    fn declared_compilation(
        declaration: &BoundMetalCompileDeclaration,
        program: &SemanticProgram,
    ) -> Compilation {
        let batch = compile(CompileRequest::new(
            program,
            NumericalContract::FlushSubnormalsToZeroF32,
            TargetRequest::new([declaration.profile().clone()])
                .expect("a singleton target request"),
        ))
        .expect("the program compiles against the authoritative profile");
        batch
            .into_targets()
            .pop()
            .expect("one target outcome")
            .into_parts()
            .1
            .expect("the authoritative target compiles")
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
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");
        let mut outcomes = Vec::new();

        for _ in 0..2 {
            let accepted = accept_or_publish_metal_plan(
                &cache,
                &toolchain,
                &program,
                plan,
                std::slice::from_ref(&declaration),
                OptimizationLevel::Default,
            )
            .expect("the checked plan resolves");
            outcomes.push(match accepted.resolution() {
                Resolution::Published { .. } => "published",
                Resolution::Hit { .. } => "hit",
                Resolution::Uncached { .. } => "uncached",
            });
        }

        assert_eq!(outcomes, ["published", "hit"]);
        let accepted = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the second cache hit remains readable");
        let variant = accepted
            .decoded()
            .variants()
            .next()
            .expect("one packaged variant");
        assert_eq!(
            variant.deferred_predicates().len(),
            variant.entries().len(),
            "every prepared entry retains its compiler-minted workgroup predicate",
        );
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
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
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
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the selected plan resolves");
        let materialized = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            materialized,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the materialized plan resolves");

        assert_ne!(
            artifact_identity(selected.resolution()),
            artifact_identity(materialized.resolution()),
            "the facade must consume the supplied checked plan rather than reselecting",
        );
        let materialized_variant = materialized
            .decoded()
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

    /// A plan compiled under another profile is refused before Metal emission.
    ///
    /// The perturbation is the *compilation*, not the declaration: the same
    /// program, the same declaration, and a plan assessed against the compiler's
    /// governed prototype profile instead of the authoritative one. The counter
    /// file is the evidence that the refusal preceded emission and every
    /// toolchain invocation — without it, "before emission" would be an
    /// assertion about ordering that the test never observes.
    #[test]
    fn a_plan_compiled_under_another_profile_is_refused_before_emission() {
        let directory = scratch("profile-mismatch");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let declaration = declaration();
        let foreign = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed prototype profile still compiles this program");
        let plan = foreign.selected().expect("one selected plan");

        let error = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect_err("a plan compiled under another profile cannot be emitted here");
        let MetalPlanBuildError::DeclaredProfile(mismatch) = &error else {
            panic!("unexpected refusal: {error:?}");
        };
        assert!(
            matches!(mismatch, MetalPlanProfileMismatch::ProfileKey { .. }),
            "the governed prototype is a different family: {mismatch}",
        );
        assert!(
            !counter.exists(),
            "the profile check must precede emission, preparation, and compiler work",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// The same-key, different-descriptor half is refused too.
    ///
    /// Separate from the case above because the two mean different things and
    /// carry different repairs, and because a check comparing only keys would
    /// pass that one and fail this.
    #[test]
    fn a_stale_descriptor_under_the_declared_key_is_refused() {
        let declaration = declaration();
        let key = declaration.profile().profile_key().as_str().to_owned();
        let mut stale = declaration.profile().canonical_descriptor().to_vec();
        stale.pop();
        let mismatch = declaration
            .require_compiled_under(&key, &stale)
            .expect_err("a truncated descriptor is a different profile revision");
        assert!(
            matches!(mismatch, MetalPlanProfileMismatch::ProfileDescriptor { .. }),
            "unexpected mismatch: {mismatch}",
        );
    }

    /// The standard Metal path's published identities are pinned to exact bytes.
    ///
    /// Two artifacts with different payload bytes can share one canonical
    /// identity, and a structural assertion over variants and entries would pass
    /// through a change that silently moved what a consumer files or loads. The
    /// two byte runs below are the whole producer-visible result of the standard
    /// path: the artifact identity a loading host compares, and the composed
    /// subject the expansion cache keys on.
    ///
    /// They are recorded rather than derived because the point is that they do
    /// **not** move: this path was rewritten to route through the
    /// backend-neutral [`assemble_plan_artifact`] facade, and these values were
    /// captured before that rewrite. A change here is either a deliberate
    /// identity revision — which must move the goldens in the commit that states
    /// why — or the defect this test exists to catch.
    ///
    /// Both moved once since, at the `tiler.artifact-program.v13` step: an
    /// executable entry is realized once per delivery position, so its identity
    /// record writes a counted run of payload keys where it wrote exactly one.
    /// The subject moves with it because it frames the artifact identity.
    ///
    /// [`assemble_plan_artifact`]: crate::assemble_plan_artifact
    #[test]
    fn the_standard_metal_path_publishes_its_recorded_identities() {
        const ARTIFACT_IDENTITY: &str =
            "cee402b825426ba2b39f50c0e548c2c69ef9ced9bf9103c0d61bbc46e9f8853f";
        const CACHE_SUBJECT: &str =
            "3f86db909b123264af67cc58d0b0dc10e79b4a349512aa65f17894a2a65e58f3";

        let directory = scratch("golden");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, _counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");

        let accepted = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the standard Metal path resolves");

        assert_eq!(
            pinned(accepted.artifact().canonical_identity().as_bytes()),
            ARTIFACT_IDENTITY,
            "the standard Metal artifact identity moved",
        );
        assert_eq!(
            pinned(accepted.cache_subject().as_bytes()),
            CACHE_SUBJECT,
            "the standard Metal cache subject moved",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// Renders a golden over an identity preimage rather than over its bytes.
    ///
    /// Both preimages here run to tens of kilobytes, so the recorded form is a
    /// digest under this test's own domain. It is a comparison aid and never an
    /// identity: nothing but this test reads it, and a collision would have to be
    /// found deliberately in SHA-256 to hide a change.
    fn pinned(preimage: &[u8]) -> String {
        DigestAlgorithm::GOVERNED
            .digest(b"tiler.build.metal-plan-golden.v1\0", preimage)
            .label()
    }

    /// One envelope carries one payload per artifact family, at its own position.
    ///
    /// The positive successor of `a_second_artifact_family_cannot_yet_share_one_envelope`,
    /// which measured the neutral artifact model refusing this envelope with
    /// `[ArtifactDiagnostic::UnusedPayload]`. Tom decided on 2026-07-25 that one
    /// selection produces one envelope carrying one payload per built family, so
    /// the whole selection has one identity and a partial delivery is impossible
    /// by construction, and this drives the production seam end to end.
    ///
    /// **An artifact family is still not a compiler-profile axis**, which is
    /// what makes this a question about payloads rather than about profiles. The
    /// two declarations differ in exactly one field —
    /// `MetalTargetFacts::platform` — which the authority ledger's projection
    /// table records as backend-only, so they share a profile key and a
    /// byte-identical descriptor and differ only in the AOT target they compile
    /// for. One compilation, one selected plan, one kernel program, two compiled
    /// objects.
    ///
    /// **What is proven, in the order it is asserted.** The artifact declares
    /// two delivery positions and carries two payloads; each position resolves
    /// to the object built for *that* family's AOT triple, through the
    /// artifact's own entries rather than through the canonically ordered
    /// descriptor table; the cache subject covers both compilations, so it is
    /// not the one-family subject; and the two-family artifact identity is not
    /// the one-family artifact's, because identity folds every carried payload.
    ///
    /// The end-to-end consumer half waits on
    /// `first-authoritative-ios-metal-compile-declaration`: the second family
    /// here is a `#[cfg(test)]` fixture over the macOS declaration's measured
    /// rows, which may not escape `cfg(test)` because those rows were taken on a
    /// macOS host.
    #[test]
    fn one_envelope_carries_one_payload_per_artifact_family() {
        let directory = scratch("two-family-envelope");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, _counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let first = declaration();
        let second = BoundMetalCompileDeclaration::second_artifact_family_fixture()
            .expect("the second artifact family assembles");
        assert_eq!(
            first.profile().profile_key(),
            second.profile().profile_key()
        );
        assert_eq!(
            first.profile().canonical_descriptor(),
            second.profile().canonical_descriptor(),
            "the artifact family does not project into the compiler profile",
        );
        assert_eq!(first.aot_target().triple(), "air64-apple-macos26.0");
        assert_eq!(second.aot_target().triple(), "air64-apple-ios26.0");

        let compilation = declared_compilation(&first, &program);
        let plan = compilation.selected().expect("one selected plan");
        let declarations = [first.clone(), second.clone()];

        let accepted = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            &declarations,
            OptimizationLevel::Default,
        )
        .expect("one selection produces one envelope carrying both families");

        let artifact = accepted.artifact();
        assert_eq!(artifact.delivery_positions(), 2);
        assert_eq!(artifact.payloads().len(), 2);

        // Resolved through the entries, because the descriptor table is ordered
        // by canonical content and says nothing about which object a consumer at
        // a given position would load.
        let decoded = accepted.decoded();
        assert_eq!(decoded.delivery_positions(), 2);
        let entry = decoded
            .variants()
            .next()
            .expect("one packaged variant")
            .entries()
            .next()
            .expect("one packaged entry");
        assert_eq!(entry.delivery_positions(), 2);
        assert_eq!(
            delivered_targets(decoded, 2),
            [
                first.aot_target().triple().clone(),
                second.aot_target().triple().clone(),
            ],
            "each delivery position resolves to the object built for its own family",
        );
        // Resolving through the entries is a check rather than a coincidence,
        // and the reversed selection is what shows it. The descriptor table is
        // ordered by canonical content and does *not* move when the delivery
        // order does, so the same two objects delivered the other way round
        // resolve the other way round while the table stays put. A seam reading
        // a payload positionally from that table would report the same pair for
        // both selections.
        let reversed = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            &[second.clone(), first.clone()],
            OptimizationLevel::Default,
        )
        .expect("the reversed selection resolves as its own artifact");
        assert_eq!(
            delivered_targets(reversed.decoded(), 2),
            [
                second.aot_target().triple().clone(),
                first.aot_target().triple().clone(),
            ],
            "delivery order decides which object a position names",
        );
        assert_eq!(
            table_targets(reversed.decoded(), 2),
            table_targets(decoded, 2),
            "the canonically ordered descriptor table is the same for both",
        );
        // Two distinct objects, which is what "one payload per built family"
        // means: a shared object would leave one family loading another's bytes.
        assert_ne!(entry.payload(0), entry.payload(1));

        // The whole selection, not one family's share of it: both the cache
        // subject and the artifact identity move when the second family is
        // dropped.
        let one_family = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&first),
            OptimizationLevel::Default,
        )
        .expect("the one-family selection also resolves");
        assert_eq!(one_family.artifact().delivery_positions(), 1);
        assert_ne!(
            accepted.cache_subject().as_bytes(),
            one_family.cache_subject().as_bytes(),
            "the cache subject covers the whole selection",
        );
        assert_ne!(
            accepted.artifact().canonical_identity().as_bytes(),
            one_family.artifact().canonical_identity().as_bytes(),
            "identity folds every carried payload, so two families is not one",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// A payload placed at another family's delivery position is a build error.
    ///
    /// The perturbation is the *order*: the same two declarations, the same two
    /// compilations, and the objects carried into the artifact the other way
    /// round. Nothing structural notices on its own — both families share a
    /// compiler profile, a profile descriptor, and a kernel program, and a
    /// wrong-family `metallib` loads and dispatches on the host GPU without
    /// error — so without a refusal the consumer's `#[cfg]` would select
    /// position 0, load the object built for the other family, and get an answer
    /// rather than a diagnostic.
    ///
    /// **It refuses as an identity disagreement, which is the strongest form
    /// available and not a weaker one.** Delivery order is meaning, so artifact
    /// identity folds each entry's payload keys *as stated*: the swapped
    /// assembly is a different artifact from the pending one whose identity
    /// keyed the cache, and the seam refuses it before publication. The two
    /// halves are asserted separately below — that the refusal fires, and that
    /// the two orders really are two identities — because the first without the
    /// second would not say why.
    ///
    /// The second case is the same defect one step earlier: a *pending*
    /// artifact whose payloads sit at the other family's positions is refused
    /// against the prepared run before any compiler work, naming the position
    /// that disagreed.
    #[test]
    fn a_payload_at_another_familys_delivery_position_is_refused() {
        let directory = scratch("swapped-delivery");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, _counter) = counted_toolchain(&directory);
        let program = semantic_program();
        let first = declaration();
        let second = BoundMetalCompileDeclaration::second_artifact_family_fixture()
            .expect("the second artifact family assembles");
        let compilation = declared_compilation(&first, &program);
        let plan = compilation.selected().expect("one selected plan");

        let sound = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            &[first.clone(), second.clone()],
            OptimizationLevel::Default,
        )
        .expect("the sound two-family selection resolves");
        let reversed = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            &[second.clone(), first.clone()],
            OptimizationLevel::Default,
        )
        .expect("the reversed two-family selection also resolves, as its own artifact");
        assert_ne!(
            sound.artifact().canonical_identity().as_bytes(),
            reversed.artifact().canonical_identity().as_bytes(),
            "delivery order is meaning, so two orders are two artifacts",
        );

        // Prepared in one order, carried in the other, under a cache this
        // subject has never been published to: the sound publication above
        // resolved the very same subject, so sharing the cache would hit and the
        // swapped assembly would never be attempted.
        let swapped_cache = ExpansionCache::open(directory.join("swapped"));
        with_prepared_run(&toolchain, plan, &[&first, &second], |prepared| {
            let pending = assemble_plan_artifact(
                &program,
                plan,
                |builder, profile| {
                    prepared
                        .iter()
                        .map(|payload| payload.push_pending(builder, profile.clone()))
                        .collect()
                },
                |_, stage| Ok(metal_entry_declaration(stage)),
            )
            .expect("the pending two-family artifact assembles");
            let refusal = accept_or_publish_delivered_metal_artifact(
                &swapped_cache,
                &pending,
                prepared,
                |compiled: Vec<CompiledMetalPayload>| {
                    let mut swapped = compiled;
                    swapped.reverse();
                    assemble_plan_artifact(
                        &program,
                        plan,
                        |builder, profile| {
                            swapped
                                .into_iter()
                                .map(|payload| payload.push_carried(builder, profile.clone()))
                                .collect()
                        },
                        |_, stage| Ok(metal_entry_declaration(stage)),
                    )
                },
            )
            .expect_err("an object at another family's delivery position cannot publish");
            assert!(
                matches!(
                    refusal,
                    MetalCacheError::Protocol(MetalArtifactProtocolError::ArtifactIdentity),
                ),
                "unexpected refusal: {refusal:?}",
            );
        });

        // The same defect in the pending artifact, refused before any compiler
        // work and naming the position that disagreed.
        with_prepared_run(&toolchain, plan, &[&first, &second], |prepared| {
            let mis_ordered = assemble_plan_artifact(
                &program,
                plan,
                |builder, profile| {
                    prepared
                        .iter()
                        .rev()
                        .map(|payload| payload.push_pending(builder, profile.clone()))
                        .collect()
                },
                |_, stage| Ok(metal_entry_declaration(stage)),
            )
            .expect("the mis-ordered pending artifact assembles");
            let refusal = accept_or_publish_delivered_metal_artifact(
                &cache,
                &mis_ordered,
                prepared,
                |_: Vec<CompiledMetalPayload>| {
                    Err::<VerifiedArtifactProgram, MetalPlanBuildError>(
                        MetalPlanBuildError::NoDeclaredFamily,
                    )
                },
            )
            .expect_err("a pending artifact whose positions are swapped cannot be keyed");
            assert!(
                matches!(
                    refusal,
                    MetalCacheError::Protocol(MetalArtifactProtocolError::PayloadSubject {
                        delivery: 0
                    }),
                ),
                "unexpected refusal: {refusal:?}",
            );
        });
        let _ = std::fs::remove_dir_all(directory);
    }

    /// Reads each delivery position's AOT target, resolved through the entries.
    ///
    /// Through the artifact's own entries and never through the descriptor
    /// table: that table is ordered by canonical content and says nothing about
    /// which object a consumer at a given position would load.
    fn delivered_targets(decoded: &DecodedArtifact, positions: usize) -> Vec<String> {
        let entry = decoded
            .variants()
            .next()
            .expect("one packaged variant")
            .entries()
            .next()
            .expect("one packaged entry");
        (0..positions)
            .map(|delivery| {
                let payload = entry
                    .payload(delivery)
                    .expect("every delivery position realizes this entry");
                decoded
                    .payload_metadata(payload)
                    .expect("every position carries its compilation subject")
                    .provenance
                    .target
                    .clone()
            })
            .collect()
    }

    /// Reads the AOT targets in the descriptor table's own canonical order.
    ///
    /// The order a positional reader would get, which is what the comparison
    /// against [`delivered_targets`] exists to separate from delivery order.
    fn table_targets(decoded: &DecodedArtifact, positions: usize) -> Vec<String> {
        (0..positions)
            .map(|payload| {
                decoded
                    .payload_metadata(payload)
                    .expect("every payload is carried")
                    .provenance
                    .target
                    .clone()
            })
            .collect()
    }

    /// Prepares one compilation per declaration, in order, and runs a body over them.
    ///
    /// A callback rather than a return value because a prepared token borrows
    /// the request that borrows the emitted unit, so the two tables have to
    /// outlive the payloads and there is nowhere above this to keep them.
    fn with_prepared_run<R>(
        toolchain: &Toolchain,
        plan: PlanAlternative<'_>,
        declarations: &[&BoundMetalCompileDeclaration],
        body: impl FnOnce(Vec<PreparedMetalPayload<'_>>) -> R,
    ) -> R {
        let kernels: Vec<_> = plan.kernels().iter().collect();
        let units: Vec<_> = declarations
            .iter()
            .map(|declaration| {
                emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
                    .expect("the unit emits for this family")
            })
            .collect();
        let requests: Vec<_> = declarations
            .iter()
            .zip(&units)
            .map(|(declaration, unit)| {
                metal_compile_request(
                    unit,
                    OptimizationLevel::Default,
                    declaration.numerical_realization(),
                )
                .expect("the request derives")
            })
            .collect();
        let payloads = units
            .iter()
            .zip(&requests)
            .map(|(unit, request)| {
                let token = toolchain
                    .prepare(request)
                    .expect("the fake toolchain prepares");
                prepare_metal_payload(unit, token).expect("the payload binds")
            })
            .collect();
        body(payloads)
    }

    /// The backend's own realization recheck survives a direct emitter call.
    ///
    /// `accept_or_publish_metal_plan` is not the only path to
    /// [`emit_translation_unit`], so the guarantee that a target which cannot
    /// honour the declared contract fails closed has to hold without it. The
    /// kernels here carry a subnormal-preserving contract — reachable only from
    /// a profile that admits one, which the authoritative declaration
    /// deliberately does not — and the declaration's measured flushing facts
    /// refuse them at the backend recheck rather than at this crate's facade.
    #[test]
    fn a_direct_emitter_call_still_fails_closed_on_the_declared_realization() {
        let program = semantic_program();
        let declaration = declaration();
        let strict = compile_governed(&program, NumericalContract::StrictF32)
            .expect("the governed prototype profile admits preserved subnormals");
        let plan = strict.selected().expect("one selected plan");
        let kernels: Vec<_> = plan.kernels().iter().collect();
        let unit =
            emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
                .expect("emission itself succeeds; conformance is a separate question");
        let refusal = unit
            .require_declared_realization()
            .expect_err("a flushing target cannot honour a preserving contract");
        assert_eq!(refusal.rule(), "unrealizable-numerical-obligation");
        assert_eq!(
            unit.numerical_gaps()
                .iter()
                .map(|gap| gap.rule())
                .collect::<Vec<_>>(),
            ["subnormal-flush-in-arithmetic"],
        );
    }
}
