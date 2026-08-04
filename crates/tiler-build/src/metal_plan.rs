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
    use tiler_ir::program::abi::{AbiRoot, ExprNode};
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
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
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
        let foreign = compile_governed(&program, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
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
    /// Both moved at the `tiler.artifact-program.v13` step: an executable entry
    /// is realized once per delivery position, so its identity record writes a
    /// counted run of payload keys where it wrote exactly one. The subject moves
    /// with it because it frames the artifact identity.
    ///
    /// Both moved again at `v14`, and this program is exactly the case the step
    /// exists to separate: it performs **no** synchronization, and its entry now
    /// records that absence rather than leaving it unstated. Three domains moved
    /// together and each for the same reason — `tiler.kernel.v5` to `v6` for the
    /// field in the resource record, `tiler.artifact-program.v13` to `v14` for
    /// the same field in the entry record, and
    /// `tiler.target-profile.declaration.v10` to `v11` for the row family a
    /// profile now declares realizations in, which the artifact folds as its
    /// target-profile descriptor digest. A zero-synchronization program's
    /// identity moving is the intended consequence: a `v13` subject described an
    /// entry that could not state a synchronization obligation at all, and a
    /// `v10` declaration described a target that could not be asked, so a cache
    /// holding either must miss rather than match.
    ///
    /// Both moved once more when the declared Metal profile gained its
    /// synchronization row. This program still performs no synchronization, and
    /// that is exactly why the move is the intended one: the row is a *target*
    /// realization, so it enters the profile descriptor the artifact folds
    /// whether or not a given program consumes it. A cache entry published
    /// against a profile that could not state the barrier must miss rather than
    /// match, because the profile it was assessed under no longer exists.
    ///
    /// And both moved again at `tiler.schedule.v4`, which gave a cooperative
    /// tile its round count. This program carries no tile at all, and its
    /// identity moves anyway — that is what a domain separator costs and what
    /// distinguishes the step from the appended tags the schedule encoder has
    /// otherwise preferred. It reaches here through the fold: the artifact
    /// identity frames each entry's kernel-program identity, which frames the
    /// kernel identity, which frames the scheduled-region identity bytes whole,
    /// separator included. Only the *content* moved; no domain between here and
    /// the schedule needed a version of its own, because each folds the bytes
    /// below it by reference rather than re-deriving a subset of them. The
    /// recorded values below were recomputed on the tree carrying both this
    /// step and the synchronization row, because each branch's own rebaseline
    /// saw only its half.
    ///
    ///
    /// **Both moved again when the numerical contract became a composed
    /// dimension vector, and neither value below was rebaselined on this
    /// branch.** The contract key is the canonical encoding of that vector now,
    /// under `tiler.contract.f32.v2`, so it differs from every `v1` preset name;
    /// the scheduled region writes it beside the realization fields it names, and
    /// the kernel, program, and artifact identities fold those bytes by
    /// reference. No encoding above the key moved, and every identity through it
    /// does — which is the intended consequence of an identity-domain step, since
    /// a cache entry published against a contract vocabulary that no longer
    /// exists must miss rather than match.
    ///
    /// **Both moved again at `tiler.schedule.v5`, which widened the cooperative
    /// staging relation to two dimensions.** This program carries no cooperative
    /// tile either, so its identity moves for the eighteen separator bytes alone
    /// — the same fold, and the same cost a domain separator has.
    ///
    /// **Both moved again at `tiler.kernel-program.v8`, which folds a program's
    /// published outputs in the semantic interface's order rather than sorted by
    /// record content.** This program publishes one output, so its own output
    /// section is byte-identical either way and the eighteen separator bytes are
    /// the whole of the move — the same shape as the schedule step above, one
    /// fold further up. The artifact frames the kernel-program identity whole,
    /// separator included, so neither the artifact domain nor the manifest
    /// schema needed a step of its own.
    ///
    /// The values are recorded rather than written in because a sibling branch
    /// may move the same two pins from its own base, and two branch-local
    /// rebaselines cannot compose: a pinned identity is recomputed on the tree
    /// the step lands into, never taken from either side. Observed on this branch:
    /// `ARTIFACT_IDENTITY`
    /// `2b15415053d8f688de094d7f4490b90fa001463717affc686ee1fe3692786a81`,
    /// `CACHE_SUBJECT`
    /// `b0803f2a48f41aa03baed4a136f7e44ddb3dbafac39bc560673c2bb7f8801ae9`.
    /// Regenerate on the merged tree with:
    ///
    /// ```text
    /// cargo nextest run -p tiler-build -E \
    ///   'test(the_standard_metal_path_publishes_its_recorded_identities)'
    /// ```
    ///
    /// and take each assertion's `left` value in turn — the cache subject is
    /// asserted second, so the artifact identity has to be moved first for the
    /// run to reach it.
    ///
    /// [`assemble_plan_artifact`]: crate::assemble_plan_artifact
    #[test]
    fn the_standard_metal_path_publishes_its_recorded_identities() {
        const ARTIFACT_IDENTITY: &str =
            "2b15415053d8f688de094d7f4490b90fa001463717affc686ee1fe3692786a81";
        const CACHE_SUBJECT: &str =
            "b0803f2a48f41aa03baed4a136f7e44ddb3dbafac39bc560673c2bb7f8801ae9";

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

    /// A reassociating sum over four contributors, on one output.
    ///
    /// **Four contributors, because below that nothing splits.**
    /// `governed_partition` requires at least two partitions of at least two
    /// contributors each, so four is the smallest extent at which a split or a
    /// tree exists to be retained at all. It is also the smallest extent above
    /// `correct-the-declined-strategy-record-for-an-unsplittable-reduction`,
    /// which records a sub-four reduction failing with `InvalidCompilerOutput`
    /// under a reassociation-permitting contract; this fixture is sized above
    /// that defect rather than around it.
    ///
    /// **One output, because the widest stage has to fit the grid axis.** This
    /// profile's grid-axis row admits four threads. The measured launches are:
    /// the fused reduction, one invocation per output; the tree, one per
    /// participant per output, so two; the split's partial stage, one per
    /// partition per output, so two; and the materialized pointwise stage, one
    /// per *element*, so four — the widest, and exactly at the limit. A second
    /// output doubles the pointwise stage to eight and the whole compilation
    /// fails `target.grid-axis` before any plan composes, which is what the
    /// two-output shape did the first time a parallel strategy was reachable
    /// here at all.
    fn reassociating_program() -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([1, 4]),
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

    /// **A flush-and-reassociate contract reaches a parallel reduction
    /// portfolio on the authoritative Apple profile.**
    ///
    /// The positive successor of
    /// `no_registered_contract_both_flushes_subnormals_and_permits_reassociation`,
    /// and it keeps that test's two-halves record of what the gap was, because
    /// the shape of the gap is what makes this result mean anything.
    ///
    /// **What the gap was.** The four registered contracts were
    /// `tiler.strict-f32.v1`, `tiler.flush-f32.v1`, `tiler.relaxed-f32.v1`, and
    /// `tiler.reassociate-f32.v1`, and none of them both flushed subnormals and
    /// permitted reassociation: the two granting regrouping were built on the
    /// strict reading and required *preserved* subnormals, which Apple `f32`
    /// arithmetic measurably flushes in every math mode, while the one this
    /// hardware delivers widened subnormals alone. `CompileRequest` accepted only
    /// that four-value preset enumeration, so no caller outside `tiler-compiler`
    /// could state the combination either. Every parallel reduction strategy
    /// regroups the declared contributor sequence, so on this target all of them
    /// were unreachable — not for want of a target fact, but for want of a
    /// contract a caller was able to name. Both halves of that record are still
    /// driven below, first and second, before the positive claim.
    ///
    /// **What closed it.** `NumericalContract` is composed from its dimensions
    /// rather than chosen from a list, so subnormal flushing and ordered
    /// regrouping are resolved independently and
    /// `NumericalContract::FLUSH_AND_REASSOCIATE_F32` is an ordinary statement.
    /// Nothing about this profile changed.
    ///
    /// **What is asserted, in order.** The strict-based reassociating contract is
    /// still refused, and still on `InputSubnormals` rather than on the
    /// regrouping. The flush-only contract still compiles and still retains no
    /// split, because it grants no regrouping. And the composed contract compiles
    /// and its portfolio retains, beside the serial fold, both the multi-pass
    /// split and the single-workgroup tree — the two strategies the declaration's
    /// synchronization realization and permitted resolution of reassociation
    /// exist to make reachable.
    ///
    /// **How each strategy is recognized through the public surface.** The
    /// `session` boundary exposes a plan alternative's kernels and its ABI
    /// entries, not its reduction topology, so each strategy is identified by an
    /// observable it alone has. The multi-pass split is the only alternative with
    /// **three** stages — pointwise, partial, and final. The single-workgroup
    /// tree is the only one declaring an entry wider than **one thread per
    /// workgroup**: it launches one invocation per participant inside one
    /// workgroup, where every independent-invocation region declares a width of
    /// one. The serial fold declares neither, and is what the flush-only contract
    /// retains on its own.
    ///
    /// **Four contributors, deliberately.** `governed_partition` splits nothing
    /// below four, and
    /// `correct-the-declined-strategy-record-for-an-unsplittable-reduction`
    /// records that a sub-four contributor reduction fails with
    /// `InvalidCompilerOutput` under a reassociation-permitting contract. That is
    /// a separate defect with its own ticket; this fixture is sized above it
    /// rather than around it.
    #[test]
    fn a_flush_and_reassociate_contract_reaches_a_parallel_portfolio() {
        let declaration = declaration();
        let program = reassociating_program();
        let targets =
            || TargetRequest::new([declaration.profile().clone()]).expect("a singleton request");

        // The strict-based regrouping contract is refused, and not for the
        // regrouping.
        let reassociating = compile(CompileRequest::new(
            &program,
            NumericalContract::REASSOCIATE_F32,
            targets(),
        ))
        .expect("the batch resolves")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1;
        let refusal = reassociating
            .expect_err("Apple f32 flushes subnormals, so the strict-based contract cannot hold");
        let rendered = format!("{refusal:?}");
        assert!(
            rendered.contains("InputSubnormals"),
            "the refusal moved off the subnormal dimension: {rendered}",
        );

        // The flush-only contract still reaches a plan, and neither parallel
        // strategy is in it, because it grants no regrouping.
        let flushing = compiled(
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            &program,
            targets(),
        );
        let flushing_shape = portfolio_shape(&flushing);
        assert!(
            !flushing_shape.iter().any(|(stages, _)| *stages == 3),
            "a multi-pass split reached a portfolio under a contract granting no regrouping: \
             {flushing_shape:?}",
        );
        assert!(
            !flushing_shape.iter().any(|(_, width)| *width > 1),
            "a cooperative entry reached a portfolio under a contract granting no regrouping: \
             {flushing_shape:?}",
        );
        assert!(
            !flushing_shape.is_empty(),
            "the flush contract is what this hardware delivers, so it must retain a plan",
        );

        // The composed contract reaches the parallel portfolio, and retains both
        // strategies beside the serial fold rather than replacing it.
        let composed = compiled(
            NumericalContract::FLUSH_AND_REASSOCIATE_F32,
            &program,
            targets(),
        );
        let composed_shape = portfolio_shape(&composed);
        assert!(
            composed_shape.iter().any(|(stages, _)| *stages == 3),
            "the multi-pass split is not in the portfolio: {composed_shape:?}",
        );
        assert!(
            composed_shape.iter().any(|(_, width)| *width > 1),
            "the single-workgroup tree is not in the portfolio: {composed_shape:?}",
        );
        assert!(
            composed_shape
                .iter()
                .any(|(stages, width)| *stages < 3 && *width == 1),
            "the serial fold was replaced rather than kept beside the parallel strategies: \
             {composed_shape:?}",
        );
        assert!(
            composed_shape.len() > flushing_shape.len(),
            "the composed contract retained no alternative the flush-only contract did not: \
             {composed_shape:?} against {flushing_shape:?}",
        );
    }

    /// Compiles one program under one contract against one target request.
    fn compiled(
        contract: NumericalContract,
        program: &SemanticProgram,
        targets: TargetRequest,
    ) -> Compilation {
        compile(CompileRequest::new(program, contract, targets))
            .expect("the batch resolves")
            .into_targets()
            .pop()
            .expect("one target outcome")
            .into_parts()
            .1
            .expect("the contract is honourable on this profile")
    }

    /// Each retained alternative's stage count and widest declared workgroup.
    ///
    /// Two observables per alternative, both read through the public boundary,
    /// because neither alone separates the three strategies: the split is the
    /// only three-stage alternative and the tree is the only one declaring a
    /// workgroup wider than one thread.
    ///
    /// `AbiEntry::threads_per_workgroup` returns an *arena position*, which its
    /// own documentation states and which is easy to read as a width — the
    /// mistake is silent, because a position is a plausible small number. The
    /// value is resolved here by indexing the alternative's expression arena and
    /// reading the literal, and a node that is not an unsigned literal is a
    /// failure rather than a skip: this profile declares every launch quantity as
    /// a literal, so a formula appearing here would mean the derivation moved and
    /// this check had stopped measuring what it names.
    fn portfolio_shape(compilation: &Compilation) -> Vec<(usize, u64)> {
        compilation
            .alternatives()
            .map(|alternative| {
                let abi = alternative.abi();
                let expressions = abi.expressions();
                let width = abi
                    .entries()
                    .map(|entry| {
                        let position = usize::try_from(entry.threads_per_workgroup())
                            .expect("an arena position fits a usize");
                        match expressions.get(position) {
                            Some(ExprNode::Root(AbiRoot::UnsignedLiteral(width))) => *width,
                            other => {
                                panic!("the workgroup width is not a declared literal: {other:?}")
                            }
                        }
                    })
                    .max()
                    .unwrap_or(0);
                (alternative.kernels().len(), width)
            })
            .collect()
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
        let strict = compile_governed(&program, NumericalContract::STRICT_F32)
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
