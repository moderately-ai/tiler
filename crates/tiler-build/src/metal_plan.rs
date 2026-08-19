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
//!
//! [`Compilation`]: tiler_compiler::session::Compilation

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

use crate::realization::RealizationTranslationError;
use crate::{
    AcceptedArtifact, BackendEntryDeclaration, BoundMetalCompileDeclaration, CompiledMetalPayload,
    MetalArtifactProtocolError, MetalAssemblyError, MetalCacheError, MetalPlanProfileMismatch,
    PlanArtifactError, PlanDeterminismDeclaration, accept_or_publish_delivered_metal_artifact,
    assemble_plan_artifact, metal_compile_request, prepare_metal_payload,
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
    /// The plan's delivered-realization evidence did not translate.
    ///
    /// Its own variant rather than an [`Self::ArtifactBuild`]: the refusal is
    /// the compiler evidence disagreeing with the artifact's profile, or
    /// offering no policy subject to bind the packaged entries to — neither of
    /// which the artifact builder ever sees.
    ArtifactRealization(RealizationTranslationError),
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
            Self::ArtifactRealization(error) => write!(
                formatter,
                "delivered-realization translation failed: {error}",
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
            Self::ArtifactRealization(error) => Some(error),
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
        // No accepted Metal receipt authority exists (ADR 0086), so every
        // Metal artifact lands admitting nothing.
        PlanDeterminismDeclaration::Unclaimed,
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
                PlanDeterminismDeclaration::Unclaimed,
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
        PlanDeterminismDeclaration::Unclaimed,
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
/// [ADR 0090](../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
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
            PlanArtifactError::Realization(error) => Self::ArtifactRealization(error),
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

    use tiler_artifact::program::{DigestAlgorithm, StageDependencyReason};
    use tiler_cache::expansion::{ExpansionCache, Resolution};
    use tiler_compiler::session::{
        Compilation, CompileRequest, NumericalContract, compile, compile_governed,
    };
    use tiler_compiler::target::TargetRequest;
    use tiler_ir::program::abi::{AbiRoot, ExprNode};
    use tiler_ir::semantic::{
        ContractionIndex, ContractionIndexStructure, F32, F32Add, F32Constant, F32Multiply,
        F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_metal::emit::emit_translation_unit;
    use tiler_metal_aot::diagnostic::MAX_RETAINED_OUTPUT_BYTES;
    use tiler_metal_aot::driver::Toolchain;
    use tiler_metal_aot::input::OptimizationLevel;

    use super::{MetalPlanBuildError, accept_or_publish_metal_plan};
    use crate::{BoundMetalCompileDeclaration, MetalPlanProfileMismatch};

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
        warning_toolchain(directory, "", "")
    }

    /// A counted fake toolchain whose stages write the supplied text to standard
    /// error and still succeed.
    ///
    /// A fake tool is what makes a warning deterministic: the real `metal` warns
    /// on whatever it chooses to warn about, so a case built on real warning text
    /// would be asserting on this host's compiler release rather than on the
    /// retention. Empty text writes nothing at all, which is how
    /// [`counted_toolchain`] remains this one fixture rather than a second one to
    /// keep in step with it.
    fn warning_toolchain(
        directory: &Path,
        metal_warning: &str,
        metallib_warning: &str,
    ) -> (Toolchain, PathBuf) {
        let counter = directory.join("compiler-invocations");
        let metal = directory.join("metal");
        let metallib = directory.join("metallib");
        let launcher = directory.join("xcrun");
        write_executable(
            &metal,
            &format!(
                "#!/bin/sh\n\
                 if [ \"$1\" = \"--version\" ]; then echo 'Metal plan-v1'; exit 0; fi\n\
                 printf '%s' '{metal_warning}' >&2\n\
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
                 printf '%s' '{metallib_warning}' >&2\n\
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

    /// What the fake front end says, and what the fake linker says.
    ///
    /// Deliberately different sentences: a retention that named one run for both
    /// stages, or bound them in collection order, would report one of these under
    /// the other's label and a reader would act on the wrong tool's opinion.
    const METAL_WARNING: &str = "warning: the front end has an opinion";
    const METALLIB_WARNING: &str = "warning: the linker has another";

    /// Whether one byte run occurs inside another.
    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    /// Reads the entry a resolution carries, whichever way it resolved.
    fn entry_of(resolution: &Resolution) -> &tiler_cache::expansion::CachedEntry {
        match resolution {
            Resolution::Hit { entry, .. } | Resolution::Published { entry, .. } => entry,
            Resolution::Uncached { .. } => panic!("the fixture cache stores entries"),
        }
    }

    /// A succeeding compilation's stage output comes back from a validated hit.
    ///
    /// The read is from the *hit*, not from the run that produced the text, and
    /// the invocation counter proves the hit compiled nothing — so what is
    /// asserted came off disk through the cache's own validation rather than out
    /// of a compiler this call ran. Both stages are asserted by label, because
    /// the fact worth having is which tool said what.
    #[test]
    fn a_succeeding_stages_output_returns_from_a_validated_cache_hit() {
        let directory = scratch("retention-hit");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, counter) = warning_toolchain(&directory, METAL_WARNING, METALLIB_WARNING);
        let program = semantic_program();
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");

        let published = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the warning compilation resolves");
        assert!(matches!(
            published.resolution(),
            Resolution::Published { .. }
        ));

        let hit = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the stored entry resolves");
        let Resolution::Hit { entry, .. } = hit.resolution() else {
            panic!("the second run hits");
        };
        assert_eq!(
            std::fs::read_to_string(&counter)
                .expect("the miss wrote its counter")
                .lines()
                .count(),
            2,
            "a hit must not re-enter the compiler to acquire retained text",
        );

        let retained = entry.retained_debug();
        assert_eq!(
            retained.runs().len(),
            2,
            "one compilation is two stages, and both ran: {retained:?}",
        );
        let front_end = retained
            .run("tiler.metal.0.metal")
            .expect("the front end's run survived the round trip");
        let linker = retained
            .run("tiler.metal.0.metallib")
            .expect("the linker's run survived the round trip");
        assert_eq!(front_end.as_bytes(), METAL_WARNING.as_bytes());
        assert_eq!(linker.as_bytes(), METALLIB_WARNING.as_bytes());
        assert!(!front_end.is_truncated());
        assert!(front_end.is_valid_utf8());
        let _ = std::fs::remove_dir_all(directory);
    }

    /// A stage that said nothing is retained as an empty run, not a missing one.
    ///
    /// Both stages ran, so the entry names both. An absent linker run would read
    /// as an entry published by a build that retained nothing for it, which is a
    /// different fact and the one [`DebugRetention::is_empty`] already answers.
    #[test]
    fn a_silent_stage_is_retained_as_an_empty_run() {
        let directory = scratch("retention-silent");
        let cache = ExpansionCache::open(directory.join("cache"));
        let (toolchain, _counter) = warning_toolchain(&directory, METAL_WARNING, "");
        let program = semantic_program();
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");

        let published = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the half-quiet compilation resolves");
        let retained = entry_of(published.resolution()).retained_debug();

        assert_eq!(retained.runs().len(), 2);
        assert!(!retained.is_empty());
        let linker = retained
            .run("tiler.metal.0.metallib")
            .expect("a silent stage is still a run this backend names");
        assert!(linker.is_empty());
        assert_eq!(linker.total_bytes(), 0);
        assert!(
            !retained
                .run("tiler.metal.0.metal")
                .expect("the front end spoke")
                .is_empty(),
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// A stage that outwrote the capture bound states the total it had, not the
    /// prefix that survived.
    ///
    /// `tiler_metal_aot::diagnostic::MAX_RETAINED_OUTPUT_BYTES` and
    /// `tiler_cache::expansion::MAX_RETAINED_RUN_BYTES` are both 16 KiB, so a
    /// stage that wrote more arrives at the retention already truncated and
    /// exactly at the bound. Everything else here is short, where a stage's total
    /// and its retained length agree — so a producer that derived the total from
    /// the bytes it was handed would present that prefix as the whole diagnostic
    /// and stay green in every other case. This is the case where the two numbers
    /// must differ, and the quiet linker beside it is what keeps the assertion
    /// from being satisfied by a total that is simply always larger.
    ///
    /// Read from the *hit*, so the stated total is one the cache encoded, wrote,
    /// re-read, and re-validated rather than one held in memory since capture.
    #[test]
    fn a_stage_that_outwrote_the_capture_bound_states_the_total_it_had() {
        let directory = scratch("retention-truncated");
        let cache = ExpansionCache::open(directory.join("cache"));
        let written = MAX_RETAINED_OUTPUT_BYTES + 512;
        let (toolchain, _counter) =
            warning_toolchain(&directory, &"x".repeat(written), METALLIB_WARNING);
        let program = semantic_program();
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");

        for _ in 0..2 {
            let _ = accept_or_publish_metal_plan(
                &cache,
                &toolchain,
                &program,
                plan,
                std::slice::from_ref(&declaration),
                OptimizationLevel::Default,
            )
            .expect("the verbose compilation resolves");
        }
        let hit = accept_or_publish_metal_plan(
            &cache,
            &toolchain,
            &program,
            plan,
            std::slice::from_ref(&declaration),
            OptimizationLevel::Default,
        )
        .expect("the stored entry resolves");
        let Resolution::Hit { entry, .. } = hit.resolution() else {
            panic!("the third run hits");
        };

        let retained = entry.retained_debug();
        let front_end = retained
            .run("tiler.metal.0.metal")
            .expect("the front end's run survived the round trip");
        assert_eq!(
            front_end.as_bytes().len(),
            MAX_RETAINED_OUTPUT_BYTES,
            "the driver's bound is what reaches the retention",
        );
        assert_eq!(
            front_end.total_bytes(),
            written as u64,
            "the run must state what the stage wrote, not the prefix that survived",
        );
        assert!(
            front_end.is_truncated(),
            "a prefix presented as the whole is the failure this states the total to prevent",
        );

        let linker = retained
            .run("tiler.metal.0.metallib")
            .expect("the linker's run survived the round trip");
        assert_eq!(
            linker.total_bytes(),
            METALLIB_WARNING.len() as u64,
            "a stage under the bound states its own length",
        );
        assert!(!linker.is_truncated());
        let _ = std::fs::remove_dir_all(directory);
    }

    /// Retaining a stage's output moves no identity and enters no envelope.
    ///
    /// Two compilations of one plan differing *only* in what their tools wrote to
    /// standard error: same versions, same flags, same source. They must compose
    /// one subject, file under one key, and produce one artifact identity, or a
    /// host whose compiler warns would miss every entry a quiet host published.
    /// The envelope is searched for the warning text as well, because the way
    /// that guarantee dies is a producer folding a diagnostic into payload
    /// metadata, where it would reach the payload digest and the artifact
    /// identity through the canonical bytes the digest is taken over.
    #[test]
    fn retaining_a_stages_output_moves_no_identity() {
        let directory = scratch("retention-identity");
        let quiet_directory = directory.join("quiet");
        let warned_directory = directory.join("warned");
        std::fs::create_dir_all(&quiet_directory).expect("the quiet directory is creatable");
        std::fs::create_dir_all(&warned_directory).expect("the warned directory is creatable");
        let program = semantic_program();
        let declaration = declaration();
        let compilation = declared_compilation(&declaration, &program);
        let plan = compilation.selected().expect("one selected plan");

        let quiet_cache = ExpansionCache::open(quiet_directory.join("cache"));
        let (quiet_toolchain, _quiet_counter) = counted_toolchain(&quiet_directory);
        let warned_cache = ExpansionCache::open(warned_directory.join("cache"));
        let (warned_toolchain, _warned_counter) =
            warning_toolchain(&warned_directory, METAL_WARNING, METALLIB_WARNING);

        let mut accepted = Vec::new();
        for (cache, toolchain) in [
            (&quiet_cache, &quiet_toolchain),
            (&warned_cache, &warned_toolchain),
        ] {
            accepted.push(
                accept_or_publish_metal_plan(
                    cache,
                    toolchain,
                    &program,
                    plan,
                    std::slice::from_ref(&declaration),
                    OptimizationLevel::Default,
                )
                .expect("both compilations resolve"),
            );
        }
        let [quiet, warned] = <[_; 2]>::try_from(accepted).expect("two resolutions");

        assert_eq!(
            quiet.cache_subject().as_bytes(),
            warned.cache_subject().as_bytes(),
            "a warning is not a compilation input, so it cannot compose a second subject",
        );
        assert_eq!(
            *entry_of(quiet.resolution()).key(),
            *entry_of(warned.resolution()).key(),
            "a warning that moved the key would make a warned build miss every entry",
        );
        assert_eq!(
            quiet.artifact().canonical_identity().as_bytes(),
            warned.artifact().canonical_identity().as_bytes(),
        );

        // The text is in the retention beside the entry and nowhere inside the
        // artifact the entry carries.
        let warned_entry = entry_of(warned.resolution());
        assert!(
            warned_entry
                .retained_debug()
                .run("tiler.metal.0.metal")
                .is_some(),
        );
        assert!(
            !contains(warned_entry.envelope_bytes(), METAL_WARNING.as_bytes()),
            "tool output inside the envelope would reach the payload digest and the artifact \
             identity",
        );
        assert!(!contains(
            warned_entry.envelope_bytes(),
            METALLIB_WARNING.as_bytes(),
        ));
        assert_eq!(
            entry_of(quiet.resolution()).retained_debug().runs().len(),
            2,
            "a quiet compilation retains two empty runs rather than nothing",
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
    /// two byte runs below are the whole producer-visible *identity* of the
    /// standard path: the artifact identity a loading host compares, and the
    /// composed subject the expansion cache keys on. A third pinned value, which
    /// is a byte count rather than an identity, is introduced by its own
    /// paragraph below.
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
    /// **Both moved again at `tiler.kernel-program.v9`, which binds every
    /// covered occurrence to the reached-only executable-coverage identity of
    /// the refinement receipt that proved it.** Unlike the four steps above,
    /// this one is not a separator over unchanged bytes: each of the five
    /// coverage records in this program's one stage gained a length-framed
    /// evidence run, so the program section grew and so did everything that
    /// folds it. The artifact stage key stepped with it to
    /// `tiler.artifact-program.stage.v3`, because an entry writes that subject
    /// itself rather than only through the nested program identity; the
    /// artifact domain and the manifest schema did not, because both frame the
    /// complete stepped key with its own separator.
    ///
    /// **And again when the grid-axis row became a measurement.** The
    /// profile's canonical descriptor is folded into artifact identity and the
    /// cache subject, and that row moved twice over in one step: its value went
    /// from 4 to 268,435,456, and its source went from an external guarantee
    /// naming the macOS SDK dispatch header to the profile's own measurement
    /// source. The descriptor *shrank* by 150 bytes as a result, because the
    /// retired normative reference was the grid row's only user while the
    /// measured source it joined was already carried by the dispatchability and
    /// numerical rows. The two steps landed on sibling branches and composed at
    /// integration, where both pins were recomputed on the merged tree.
    ///
    /// **And both moved again at `tiler.kernel-program.v10`, which folds a
    /// program's declared publishing-copy contracts.** This program declares
    /// none, and its identity moves anyway: the section is written
    /// unconditionally, so a zero-copy program grows an eight-byte zero count
    /// and every program's bytes move. That is the whole content of the step,
    /// and it is why the step was taken instead of an appended conditional
    /// section — an appended one would leave the section's presence
    /// positionally ambiguous and constrain every future appended section. The
    /// artifact stage key did *not* step with it, unlike at the v9 step: a
    /// publishing copy is a program-scope declaration and no entry writes that
    /// subject itself, so the artifact domain and the manifest schema both hold,
    /// each framing the complete stepped program identity with its own
    /// separator.
    ///
    /// **And both moved again at `tiler.kernel-program.v11`, which folds a
    /// program's declared staged-realization contracts.** This program declares
    /// none either, and the shape of the move is exactly the v10 one: a fourth
    /// program-scope declaration section written unconditionally, so a program
    /// with no staged chain grows a second eight-byte zero count and every
    /// program's bytes move. Nothing below or beside the program domain steps,
    /// and for the reasons the v10 paragraph records: a staged realization is a
    /// program-scope declaration and no artifact entry writes that subject
    /// itself, so `tiler.artifact-program.stage.v3`,
    /// `tiler.artifact-program.v15`, and manifest schema 14.0 all hold, each
    /// framing the complete stepped program identity with its own separator.
    ///
    /// **And both moved again at ADR 0104's coverage fold, which is the first
    /// step in this ledger that moved these values without stepping a domain
    /// above the one it changed.** Each of the five coverage records in this
    /// program's one stage used to open with a whole framed
    /// `SemanticGraphIdentity`; it opens with a thirty-two-byte governed digest
    /// of that identity now, so this program's kernel-program identity *shrank*
    /// — and every program's does, from `134n² + 3650n + 727` bytes to
    /// `3525n + 727`, quadratic in operation count to linear. The domain that
    /// stepped is `tiler.ir.index-refinement-executable-coverage.v1` to `v2`,
    /// with its staged sibling. Nothing above it stepped:
    /// `tiler.kernel-program.v11`, `tiler.kernel-program.stage.v2`,
    /// `tiler.artifact-program.stage.v3`, `tiler.artifact-program.v15`, and
    /// manifest schema 15.0 each frame the complete stepped coverage identity
    /// with their own separator and re-derive no subset of it, so a `v1` and a
    /// `v2` fold cannot collide and there is no reading for a stepped domain
    /// above to protect. That is the `tiler.schedule.v4` and
    /// `tiler.contract.f32.v2` shape rather than the `tiler.kernel-program.v9`
    /// one, which changed the coverage record's own grammar here and had to step
    /// with it.
    ///
    /// **And all three moved again when the emitted provenance header widened
    /// past one width — the first step in this ledger that moved these values
    /// while stepping no identity domain and no profile row.** `tiler-metal`'s
    /// `assemble` used to write `every f32 immediate is its exact bit pattern`;
    /// it writes `every floating-point immediate` now, because all three
    /// properties that sentence carries hold at `bf16` exactly as at `f32` and
    /// the backend emits both widths. The emitted source is artifact content, so
    /// eleven more bytes of comment reach every value pinned here: no encoding
    /// changed shape, nothing was renamed, and no version below could have
    /// absorbed the difference. It is also the cleanest available reading of
    /// what the third pin measures — the fixed content moved by exactly the
    /// eleven bytes the sentence grew, which a framing step would not have
    /// produced.
    ///
    /// **And all three moved again when the compiler began founding an
    /// obligation's policy locus on the operation at its occurrence — the first
    /// step in this ledger that moved these values by changing what the producer
    /// *says* rather than how anything is framed.** `tiler.artifact-program.v15`
    /// folds the delivered-realization record's canonical bytes, and that record
    /// carries one row per `(subject, dimension, occurrence, locus)`. The
    /// compiler used to emit every honoured dimension at
    /// `PolicyLocus::Computation` of every covered occurrence; it emits each at
    /// the position the occurrence's own operation founds, and none at an
    /// occurrence whose operation cannot consume the dimension. For this
    /// fixture's program — two constants, a multiply, an add, and a strict
    /// serial sum — that takes twenty rows to eleven: the two constants consume
    /// no numerical freedom, and the sum consumes no contraction. The byte count
    /// fell by exactly 180, nine dropped rows at the twenty bytes an obligation
    /// row encodes to, which is the reading a relocation alone would not have
    /// produced. No domain stepped, because the record's grammar did not change:
    /// the same row layout, the same locus tag space, and every dimension still
    /// resolving `Required`, so no disposition changed width either.
    ///
    /// **A third value is pinned below, and it is deliberately not an
    /// identity.** `FIXED_CONTENT_BYTES` is the published envelope's *fixed
    /// content*: its encoded width less the backend object bytes it carries,
    /// which is what this artifact would encode to if its compiled object were
    /// empty. The subtraction is exact rather than approximate, because section
    /// framing is fixed width — one `u32` position and one `u64` length ahead of
    /// every section's bytes — so removing an object's bytes removes nothing but
    /// them. It is subtracted rather than assumed absent because this fixture's
    /// fake toolchain does emit an object: eight bytes, `MTLBplan`, against a
    /// 64,707-byte envelope. Reading it through the decoded envelope the cache
    /// resolution carries, rather than by re-encoding the producer-side artifact,
    /// keeps the pinned quantity the one a loading host actually receives.
    ///
    /// **What moves it, and why it is worth a third assertion when two
    /// identities are already pinned.** It moves whenever the encoded *size* of
    /// anything the manifest or a framed section carries moves — so most
    /// identity-domain steps in the ledger above move it too, and the two pins
    /// above would have caught those anyway. What it catches that they cannot is
    /// a size-moving change to the wire that moves no subject, and manifest
    /// schema `15.0` is the worked example: replacing the manifest's trailing
    /// identity preimage with a thirty-two-byte digest took 49.2% out of a
    /// measured fixture's fixed content while every pinned identity, golden, and
    /// cache subject in the workspace held unchanged. That is the class of change
    /// that reaches a consumer as bytes and reached this test as nothing. Watched
    /// failing rather than assumed to work: lengthening `MANIFEST_DOMAIN` by one
    /// byte in `tiler-artifact`'s envelope encoder leaves both identity
    /// assertions passing and fails this one with `left: 64700`.
    ///
    /// **What it does not catch, since a byte count is not a digest of the
    /// bytes.** A wire change that moves content without moving its width — a
    /// reordering, a swapped fixed-width field, a renamed domain of the same
    /// length — passes here. Widening this to a digest over the envelope was
    /// rejected because the fixture's compiled object is a fake toolchain's
    /// eight bytes, so such a pin would rebaseline on test-fixture edits that say
    /// nothing about the encoding. The three consumers that price against
    /// envelope size are the expansion cache's fail-closed per-hit validation,
    /// the 1 MiB per-invocation inline-embedding ceiling, and the cache's
    /// steady-state footprint, and all three price the *size*; none of them owned
    /// the number, which is how it grew 4.4× over 107 landings before a research
    /// sweep attributed it.
    /// `docs/research/artifacts/manifest-fixed-content-growth.md` is that
    /// attribution and this pin is its Section 6 recommendation.
    ///
    /// **The counterpoint, carried rather than dropped, because it bounds what
    /// this pin may be read as covering.** A pin on one fixture at one operation
    /// count measures a coefficient, not a curve: it is blind to program-size
    /// growth, which is the growth that will actually consume the embedding
    /// ceiling. The demonstration is in that record's own ladder — `36d05128`
    /// raised the governed `semantic_operations` budget from 8 to 62, admitting
    /// by size a program whose envelope the then-quadratic encoding put at ~2.8×
    /// the per-invocation ceiling, and moved a fixed fixture's fixed content by
    /// exactly zero. The two folds since landed make that arithmetic historical
    /// and not the blind spot: identity is linear now and the crossing sits near
    /// 148 operations, while a fixture pinned at one operation count still cannot
    /// report a budget being widened underneath it. Covering that is the
    /// embedding-ceiling trigger on the coverage-digest deferral, not this pin;
    /// this value is the cheap half that fires on the encoding, and the trigger
    /// is the half that fires on the program.
    ///
    /// The values are recorded rather than written in because a sibling branch
    /// may move the same three pins from its own base, and two branch-local
    /// rebaselines cannot compose: a pinned identity is recomputed on the tree
    /// the step lands into, never taken from either side.
    /// `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` is the
    /// sibling that depends on this row for exactly that reason. The constants
    /// below were recomputed on 2026-08-07 **on the merged tree carrying two
    /// steps that landed the same day**, and this paragraph is the worked
    /// instance of the warning above rather than an exception to it. Each branch
    /// moved these three pins from its own base and neither branch's values
    /// survive: `admit-symbolic-index-expression-coefficients` took the
    /// `tiler.index-region.v10` step, making a linear combination's coefficient
    /// a tagged `SourcedIndexInteger` where `v9` wrote a bare integer, and
    /// `derive-per-locus-numerical-obligations` narrowed the delivered
    /// realization's obligation rows to the occurrences that found them. The
    /// values below were computed after both were merged, over the base that
    /// already held the header widening, the coverage fold, the v11 step, the
    /// v10 step, the v9 step, the v8 step, and the measured grid-axis row.
    ///
    /// The envelope moves in two directions that partly cancel, so the net is
    /// not a single step's growth: the coefficient tagging adds one byte per
    /// coefficient this fixture's regions spell, and the locus derivation drops
    /// the obligation rows no occurrence founds at twenty bytes each. Both are
    /// encoding-predicted, and neither is a layout move — no domain stepped in
    /// either branch. The arithmetic closes exactly, which is the evidence for
    /// that claim rather than an assertion of it: **64,710 + 12 − 180 = 64,542**,
    /// the header-widening base plus the coefficient tags less the nine dropped
    /// obligation rows at twenty bytes each. A layout move on either side would
    /// not have summed.
    ///
    /// **The 2026-08-07 cost-row step is what these constants hold now**, and it
    /// is the first movement here caused by a *profile* row rather than by an
    /// encoding or a program. `activate-measured-reduction-selection-from-a-target-cost-row`
    /// declares the measured saturated-parallel-fold-step row on
    /// `BoundMetalCompileDeclaration::first_macos_apple9`, which lengthens that
    /// profile's canonical descriptor by exactly 100 bytes — the cost-row
    /// section's length-prefixed 33-byte domain separator, a row count, the
    /// length-prefixed 34-byte row key, a fixed-width `u64`, and a one-byte
    /// compact source index, with no source-table growth because the row shares
    /// the measured source the grid-axis, dispatchability, and numerical rows
    /// already carry. The published envelope embeds that descriptor seven times,
    /// so the fixed content grew by exactly **700**: 64,542 + 700 = 65,242.
    /// **That the arithmetic closes is the evidence no layout moved**, exactly as
    /// it was for the two steps below. A profile declaring no cost row encodes
    /// byte for byte what it encoded before the family existed —
    /// `complete_descriptor` states the derivation and
    /// `the_declared_profile_states_the_measured_cost_row` drives both halves —
    /// so nothing but this one profile's identity moved.
    ///
    /// **The 2026-08-07 `tiler.index-region.v11` step is what these constants
    /// hold now.**
    /// `bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`
    /// gives every discharged index-domain assessment a one-byte
    /// `IndexDomainFactSource` tag naming whether the argument that closed it
    /// read the region's shape environment. Every region carrying a discharged
    /// predicate therefore re-encodes, and this fixture is one of them even
    /// though it names no symbol at all: its every new tag reads `Program`,
    /// which is exactly the claim the tag exists to make legible.
    ///
    /// **The delta's *form* is encoding-predicted and its *count* is measured**,
    /// and the distinction is stated rather than blurred because only the form
    /// was derived here. The form is one byte per discharged index-domain
    /// assessment, appended unconditionally so the slot is fixed-width; the
    /// count is the 52 such assessments this fixture's embedded regions carry,
    /// read off the move rather than derived from the program. **65,242 + 52 =
    /// 65,294.** That the growth is a whole number of single bytes with no
    /// residue is the evidence no layout moved — the same standard the two
    /// steps above meet, one notch weaker because the multiplicand was not
    /// independently counted. The neighbouring `v10` step's 12 coefficient tags
    /// are the comparable population and were recorded the same way.
    ///
    /// Superseded values, for a reader reconciling an older record:
    /// the measured cost row, which is what these constants held immediately
    /// before the `v11` step,
    /// `357f06767e459ea99fb45a11d6aaffd01f46051a941ec2f1e3eed54ae4290b73` /
    /// `c626e43b6cfc64ccb828f0394c0a641e0d01d7f54bcb3b506cdc3b8651dac59b` /
    /// 65,242 bytes;
    /// the per-locus obligation derivation composed with the symbolic-coefficient
    /// step, which is what these constants held immediately before the cost row,
    /// `23c46a19f6bc601d35bf4ca653e890372da3079b1bb60526220dc3b3221dcdd0` /
    /// `e89c4d826149c9d103e2ed8392968c0c519df454e23e7793932bc33bc86b1595` /
    /// 64,542 bytes;
    /// the symbolic-coefficient step alone, which is what these constants held
    /// on `main` immediately before the locus derivation merged,
    /// `65adeb81d7ab30d73ba099403d9214effcfc2de963a51b39872a92fcfe7e4f5e` /
    /// `b34fc5562db3eb1a8a6d280faa26fb5aef7a9b632609c4daf5cad12692ffe8f4` /
    /// 64,722 bytes;
    /// the header widening, which is what these constants held immediately
    /// before either of those two steps,
    /// `17a16aa4d15b35a0eae7e382b9e96ea3fca7c01a5a1c80495600aace20f2e63d` /
    /// `a3d44827bf86b5979f3d79eaf7e9392f997255ae88376edfb6f8f304e51cdfe8` /
    /// 64,710 bytes;
    /// the coverage fold, which is what these constants held immediately before
    /// the header widened,
    /// `2b0162eb461edeaa8069a022e54057572bf7992970205a5a33f1efee2df896ca` /
    /// `8e48d6fbfca8c490c883a557be2c7c5dfcb8264a751c84e585c574d4cd12f186` /
    /// 64,699 bytes;
    /// v11-without-the-fold, which is what these constants held immediately
    /// before the coverage fold,
    /// `e57b8852b4a9172057dba08f4758574b96fe140a0f2d974390e890dc7425c59d` /
    /// `f107cd81f779decff8c2bb15fd61881a2e79ad004457b042fcbfdea25ad97c88`;
    /// v10, which is what they held immediately before the v11 step,
    /// `e3ac0aee9e9ce35b23edc2ee49ce7fdb4b40cabbb34774b782b7325d4455fa34` /
    /// `14cbccad74c0d2f1c4a05f295a6b04e87aa45aa13be86460e810e76ff478a263`;
    /// v9, which is what they held immediately before the v10 step,
    /// `d22c0d11f8486a15b3df7651feee543eb5d0f8d398a7eb9047ae45b15f9ce832` /
    /// `6dee9552e5fb3c0cefe12cacab8d15153fd0909923bf7c93f2d5f92c5d679d68`;
    /// v8-and-grid-row, which is what they held immediately before
    /// the v9 step,
    /// `886ed671cb98364ed0e020e7e2d51d69db1cd210d11f11d8ed7ee2c82f403892` /
    /// `f23ac9ddf349011f751e3128a8d89d7c423d4f155a37d9a03d1d8838deb64ba1`,
    /// and, from the two sibling branches that composed into it, v8-only
    /// `2b15415053d8f688de094d7f4490b90fa001463717affc686ee1fe3692786a81` /
    /// `b0803f2a48f41aa03baed4a136f7e44ddb3dbafac39bc560673c2bb7f8801ae9`,
    /// grid-row-only
    /// `3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69` /
    /// `8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`,
    /// and pre-both
    /// `124981346c0bd593f19154f7ec3df26588179e0c7b446a995bbe4a7a92ba25bd` /
    /// `94dfde30611c9021da8e4a71f9b6824f3af1ff09ec68daa4c65d05bfc63e6370`.
    /// The first two superseded entries above are triples and they are the only
    /// ones: every entry after them is a pair because the byte count did not
    /// exist yet at any of them, so none of those is a triple whose third value
    /// went missing, and each row a later step supersedes carries all three.
    /// Regenerate on the merged tree with:
    ///
    /// ```text
    /// cargo nextest run -p tiler-build -E \
    ///   'test(the_standard_metal_path_publishes_its_recorded_identities)'
    /// ```
    ///
    /// and take each assertion's `left` value in turn, in the order they are
    /// asserted — artifact identity, then cache subject, then fixed content —
    /// because each has to be moved before the run reaches the next. Three runs,
    /// not one, and the count is the reason to correct all three in one commit
    /// rather than leaving a later one to a follow-up.
    ///
    /// **A moved `FIXED_CONTENT_BYTES` is checkable rather than merely copied.**
    /// The graph identity appears in this envelope **twice** — once as the
    /// artifact program's own carried subject and once inside the nested kernel
    /// program's subject — so an encoding change costing `n` bytes per semantic
    /// extent moves this constant by `2 * n * 7`, the fixture reaching seven
    /// extents across its input and four reachable results. `v2 → v3` cost one
    /// tag byte per extent and moved it by exactly fourteen. A delta that does
    /// not factor that way is a second change riding along, not this one.
    ///
    /// [`assemble_plan_artifact`]: crate::assemble_plan_artifact
    #[test]
    fn the_standard_metal_path_publishes_its_recorded_identities() {
        // **Hex step after the ADR 0013 stability subject.** The
        // `tiler.artifact-program.v20` / manifest `20.0` / guard-and-routing
        // `2.0` step adds one environment-presence byte to the payload row and
        // one plan-determinism scope run to the variant row — Metal declares
        // no environment and claims no cell, so the new bytes are the absence
        // spellings — and both identities move for the stepped domain and
        // component schema. The superseded pair was
        // `9ec0c14925a24cc85ab489863936bfcb2c488771c320e7e58e729bdc1450c8e7` /
        // `a3b0054639592c319bdc3b28566e8f6c2adc961eab72b488a66dd39f8acfeea0`
        // at 77,256 bytes; the 2,181-byte descriptor holds, because the
        // target profile does not move at this step.
        //
        // **The earlier hex step was the elementary numerical dimensions.** The
        // coordinated `tiler.schedule.v7` / `tiler.kernel.v9` /
        // `tiler.artifact-program.v19` / manifest `19.0` step carries the
        // reciprocal-transform permission and the approximate-intrinsic
        // envelope in every numerical record, and this profile gains its two
        // measured `Forbidden` rows, so both identities, the descriptor, and
        // the fixed content all move together. The superseded pair was
        // `7024654cdc51ea3a0f5d8c6c6f24ef603a9faae03fdf081bf497fee025284ac0` /
        // `44ce29d76c7b4d9fe188269ae898a3dcd9729db1ae256fab65086896d2e2145f`
        // at 77,096 bytes and a 2,169-byte descriptor.
        //
        // The earlier hex step was the feasibility rule-set key v6 → v7, the
        // prepared subgroup-width confirmation vocabulary, which moved both
        // identities while descriptor length and fixed content held; its
        // superseded pair was
        // `d8877dd9284de4e6ea58ec97067008311e90f7df714459b3fd4f9fce8b70447d` /
        // `da08d9006f071e38244d0ea765f563dce425cb934057d847a0f03bd88b5aa5b8`
        // at the same 77,096 bytes.
        const ARTIFACT_IDENTITY: &str =
            "a90c65750ba0bb7122b0553025ed9f1a5c5f4b9ac6fdd4c390cd293c01f82274";
        const CACHE_SUBJECT: &str =
            "ea3e346e59397cd83b16bf8672a1337651de2845cbdb7b9a604bf3443e9b1717";
        // **Hex step after the feasibility rule-set key v5 → v6.** Descriptor
        // length and fixed content stay at the workgroup-tree-width-policy
        // values: silent profiles write no subgroup section, and the key
        // string is the same length.
        // **77,061 after the closed workgroup-tree-width policy.** The previous
        // 76,291 pin was the eight-dimension region-feasibility projection.
        // The policy section is 70 bytes; the envelope delta is 770, which is
        // `11 * 70` rather than the historical seven raw-descriptor embeddings
        // alone. Domain tags are unchanged.
        // **77,112 after the retained shape environment.** The empty
        // environment's identity is 43 bytes; the manifest frames it as 51.
        // 77,061 + 51 = 77,112. Domain tag length is unchanged at v17.
        // **77,062 after fieldless input roles.** Removing the declared ordinal
        // from repeated schedule and kernel role records saves 50 bytes in this
        // fixed corpus. The artifact and manifest grammars do not move; they
        // frame the complete stepped nested identities.
        // **77,054 after the structured selected-capability subject.** Dropping
        // the redundant `tiler.capability.` text prefix saves eight bytes in
        // this one-provider envelope while the replacement fields are framed
        // independently.
        // **91,891 after the required compilation-selection carrier.**
        // 77,266 + 14,625, read off the move rather than fully factored,
        // matching the elementary-dimensions entry's own stated practice one
        // notch below. The two contributors are the complete profile
        // descriptor's step from 2,181 to 3,296 bytes (the per-population
        // measured sources and their framed 287-byte production selections) at
        // each of its identity-bearing embeddings, and the delivered-
        // realization record's evidence rows, each of which frames that same
        // descriptor and ends in the schema-4 source whose context now carries
        // its framed selection.
        // **77,266 after the ADR 0013 stability subject.** 77,256 + 10: one
        // environment-presence byte on the single payload row, plus the
        // variant's plan-determinism scope run — an eight-byte count and one
        // `Unclaimed` tag for the one delivery position. A delta that does not
        // factor as `payloads + 8 + positions` is a second change riding along.
        // **77,256 after the elementary numerical dimensions.** 77,096 + 160.
        // Eighty-four is the descriptor's twelve new bytes (two six-byte
        // honourability rows) across its seven identity-bearing embeddings;
        // four is the two per-entry records (resources and numerical facts, one
        // permission tag and one envelope tag each); the remaining seventy-two
        // is the stepped nested identities — the schedule identity grows by its
        // two inserted tag bytes and the kernel identity by six (the framed
        // schedule subject plus its own numerical and requirement records) —
        // folded through the kernel-program and stage subjects at their
        // embedding multiplicities, read off the move rather than derived,
        // which is one notch weaker and is stated rather than blurred.
        const FIXED_CONTENT_BYTES: usize = 91_891;

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
        let published = accepted.decoded();

        // **No backend route row is minted for the index-arithmetic
        // requirement, and this is where that is checked rather than asserted in
        // prose.** The requirement is derivable from the verified program, so
        // `crates/tiler-artifact/src/program/requirement.rs`'s admission test
        // excludes it: a row restating it would be a second producer authority
        // that could contradict the dispatch record. The standard Metal path
        // emits no *additional* backend feature either, so the correct
        // population is zero and not one.
        //
        // Counted per variant rather than summed, so a second variant carrying
        // rows could not be hidden by a first variant carrying none.
        let variants = published.variants().count();
        assert_eq!(variants, 1, "the standard Metal path publishes one variant");
        for variant in published.variants() {
            assert!(
                variant.route_requirements().is_empty(),
                "the standard Metal path mints no live-device route row; found {:?}",
                variant.route_requirements(),
            );
        }

        let envelope = published
            .re_encode()
            .expect("the published envelope re-encodes");
        let objects: usize = (0..published.payloads().len())
            .filter_map(|payload| published.payload_object(payload))
            .map(<[u8]>::len)
            .sum();
        assert_eq!(
            envelope.len() - objects,
            FIXED_CONTENT_BYTES,
            "the standard Metal envelope's fixed content moved",
        );
        let _ = std::fs::remove_dir_all(directory);
    }

    /// The authority ledger's present-tense pin paragraph names the same
    /// values this test pins. A hex or byte-count edit here that leaves the
    /// ledger on yesterday's numbers is the failure this check exists for.
    #[test]
    fn the_authority_ledger_mirrors_the_live_standard_metal_pins() {
        const ARTIFACT_IDENTITY: &str =
            "a90c65750ba0bb7122b0553025ed9f1a5c5f4b9ac6fdd4c390cd293c01f82274";
        const CACHE_SUBJECT: &str =
            "ea3e346e59397cd83b16bf8672a1337651de2845cbdb7b9a604bf3443e9b1717";
        let ledger = include_str!(
            "../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md"
        );
        let today = ledger
            .split("**What those pins are today")
            .nth(1)
            .and_then(|rest| rest.split("**The 2026-08-07").next())
            .expect("the live pin paragraph is present and bounded");
        assert!(
            today.contains(ARTIFACT_IDENTITY),
            "the live pin paragraph does not name ARTIFACT_IDENTITY",
        );
        assert!(
            today.contains(CACHE_SUBJECT),
            "the live pin paragraph does not name CACHE_SUBJECT",
        );
        assert!(
            today.contains("fixed content is 91,891 bytes"),
            "the live pin paragraph does not name FIXED_CONTENT_BYTES",
        );
        assert!(
            today.contains("3,296"),
            "the live pin paragraph does not name the descriptor length",
        );
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
    /// **Four contributors, because below that nothing splits.** Both partition
    /// rules require at least two partitions of at least two contributors each
    /// — `governed_partition` for the split and `capped_tree_partition` for the
    /// tree, which admit exactly the same extents and differ only in the width
    /// they choose within them — so four is the smallest extent at which a
    /// split or a tree exists to be retained at all. It is also the smallest extent above
    /// `correct-the-declined-strategy-record-for-an-unsplittable-reduction`,
    /// which records a sub-four reduction failing with `InvalidCompilerOutput`
    /// under a reassociation-permitting contract; this fixture is sized above
    /// that defect rather than around it.
    ///
    /// **One output, and the reason is now history rather than a constraint.**
    /// When this fixture was written the profile's grid-axis row admitted four
    /// threads, and the materialized pointwise stage — one invocation per
    /// *element* — sat exactly at that limit, so a second output doubled it to
    /// eight and failed `target.grid-axis` before any plan composed. That row is
    /// now a measured 268,435,456, so the shape is no longer forced; it is kept
    /// because this test is about *which strategies a contract retains*, and the
    /// smallest shape that retains all three is the one that isolates that
    /// question from everything a larger program would also exercise. The domain
    /// the widened row opened is reported by
    /// `the_measured_grid_axis_admits_more_than_one_three_strategy_shape` below.
    fn reassociating_program() -> SemanticProgram {
        reduction_program(1, 4)
    }

    /// A `rows x contributors` multiply-add prologue feeding a trailing-axis sum.
    ///
    /// The same program family the retained reduction-crossover sweep drives,
    /// parameterized on both extents so the domain report below and the
    /// portfolio test above cannot drift apart by describing different programs.
    /// The prologue is what makes the multi-pass split expressible at all: the
    /// split divides the *materialized* reduction, so a bare sum is a different
    /// program.
    fn reduction_program(rows: u64, contributors: u64) -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let input = builder
            .input::<F32>(
                InputKey::new("input").expect("the input key is valid"),
                Shape::from_dims([rows, contributors]),
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

    /// **The measured-calibration domain, reported against the profile that
    /// calibration measures.**
    ///
    /// [`calibrate-and-activate-parallel-reduction-selection`] needs a crossover:
    /// a shape at which the winning reduction strategy changes. That needs at
    /// least two shapes on which all three strategies exist and can be timed. Its
    /// 2026-08-02 outcome recorded that exactly one existed, and named the cause
    /// — the grid-axis row capped the prologue's one-invocation-per-element
    /// launch at four, and both partition rules withhold their parallel
    /// strategy below four contributors, so
    /// `4 <= contributors <= rows * contributors <= bound` closed on `(1, 4)`.
    ///
    /// **This test replaces a trigger that could not have fired.**
    /// `tiler_compiler::target::tests::only_one_shape_admits_all_three_reduction_strategies`
    /// was written as that trigger, but it reads the bound from
    /// `TargetProfileBuilder::governed`, the *target-neutral prototype baseline*,
    /// while calibration measures against `BoundMetalCompileDeclaration::first_macos_apple9`.
    /// The two agreed at four, which hid the difference. They do not agree now
    /// and cannot be made to: a macOS Apple9 measurement is evidence about one
    /// target and can never source a row that stands in for every target, so the
    /// prototype row stays where it is and the trigger has to live here, in the
    /// crate that can see the profile it is about.
    ///
    /// **Read from compilation rather than derived.** The domain is observed by
    /// compiling each candidate shape and counting the strategies its portfolio
    /// retains, using the same two structural observables as the test above. A
    /// reimplementation of the arithmetic would agree with itself while the
    /// compiler disagreed.
    ///
    /// The refusal case at the end is what stops this from being a check that
    /// passes whatever the row says: a shape whose work items exceed the declared
    /// bound must still be refused, on the grid axis, by name.
    ///
    /// [`calibrate-and-activate-parallel-reduction-selection`]:
    ///     ../../../tickets/calibrate-and-activate-parallel-reduction-selection.md
    #[test]
    fn the_measured_grid_axis_admits_more_than_one_three_strategy_shape() {
        let declaration = declaration();
        let targets =
            || TargetRequest::new([declaration.profile().clone()]).expect("a singleton request");

        // Contributor counts admitting a balanced exact partition, so a shape
        // that is absent from the domain is absent for a target reason rather
        // than for `correct-the-declined-strategy-record-for-an-unsplittable-reduction`.
        let candidates = [(1, 4), (1, 8), (2, 4), (4, 16), (64, 64)];
        let mut domain = Vec::new();
        for (rows, contributors) in candidates {
            let program = reduction_program(rows, contributors);
            let compilation = compiled(
                NumericalContract::FLUSH_AND_REASSOCIATE_F32,
                &program,
                targets(),
            );
            let shape = portfolio_shape(&compilation);
            let split = shape.iter().any(|(stages, _)| *stages == 3);
            let tree = shape.iter().any(|(_, width)| *width > 1);
            let serial = shape
                .iter()
                .any(|(stages, width)| *stages < 3 && *width == 1);
            if split && tree && serial {
                domain.push((rows, contributors));
            }
        }
        assert!(
            domain.len() > 1,
            "the three-strategy domain is {domain:?}: a crossover needs at least two shapes on \
             which every alternative exists and can be timed, so calibration is still blocked",
        );
        assert!(
            domain.contains(&(1, 4)),
            "the widened domain must extend the one shape the superseded row admitted rather \
             than replace it: {domain:?}",
        );

        // The axis still refuses above its declared bound, and says so by name.
        // Shapes are symbolic, so this costs no more to compile than the others.
        // Mutation-proved at the boundary rather than far from it: `(2,
        // 134_217_728)` is exactly 268,435,456 work items and compiles, so the
        // refusal below is the bound doing its job and not a shape that fails
        // for some unrelated reason.
        let refusal = compile(CompileRequest::new(
            &reduction_program(2, 268_435_456),
            NumericalContract::FLUSH_AND_REASSOCIATE_F32,
            targets(),
        ))
        .expect("the batch resolves")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1
        .expect_err("536,870,912 work items exceed the declared grid-axis bound");
        let rendered = refusal
            .explain()
            .map_or_else(|| format!("{refusal:?}"), |report| report.render());
        assert!(
            rendered.contains("grid-axis:rejected"),
            "a shape above the declared bound was refused, but not on the grid axis: {rendered}",
        );
    }

    /// The L3 correctness profile's six cells, as `(id, m, n, k)`.
    ///
    /// Transcribed from the retained realization probe's `workload.tsv`
    /// (`spikes/scheduling/metal_contraction_vertical/results/`
    /// `2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/`),
    /// which is where the `result_sha256` values a device comparison checks
    /// against also live. All six rather than the smallest, because the row that
    /// gates them is a single bound and a test that checked one cell would not
    /// notice a bound that admitted `w_decode_kv`'s 1,024 output elements and
    /// refused `w_prefill_mlp_in`'s 393,216.
    const L3_CORRECTNESS_CELLS: [(&str, u64, u64, u64); 6] = [
        ("w_decode_kv", 1, 1024, 1024),
        ("w_vocab_slice", 1, 8192, 1024),
        ("w_prefill_q", 10, 2048, 1024),
        ("w_prefill_mlp_in", 128, 3072, 1024),
        ("w_prefill_mlp_out", 128, 1024, 3072),
        ("w_prefill_o", 128, 1024, 2048),
    ];

    /// The L3 profile's index structure, `td,od->to`.
    ///
    /// Spelled with the same arbitrary frontend index labels
    /// `prototypes/serial-sum-compile` and `prototypes/serial-sum-run` use, so
    /// all three reach one canonical encoding through the renaming-invariant
    /// rule ADR 0087 requires rather than by each happening to write the
    /// canonical labels.
    fn contraction_structure() -> ContractionIndexStructure {
        ContractionIndexStructure::new(
            [
                [ContractionIndex::new(19), ContractionIndex::new(3)],
                [ContractionIndex::new(14), ContractionIndex::new(3)],
            ],
            [ContractionIndex::new(19), ContractionIndex::new(14)],
        )
        .expect("the profile's index structure passes every structural admission rule")
    }

    /// Builds `activations[m, k] x weights[n, k] -> projected[m, n]`.
    fn contraction_program(m: u64, n: u64, k: u64) -> SemanticProgram {
        let mut builder =
            SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
        let activations = builder
            .input::<F32>(
                InputKey::new("activations").expect("the activations key is valid"),
                Shape::from_dims([m, k]),
            )
            .expect("the activations operand binds");
        let weights = builder
            .input::<F32>(
                InputKey::new("weights").expect("the weights key is valid"),
                Shape::from_dims([n, k]),
            )
            .expect("the weights operand binds");
        let projected = F32TensorContraction::apply(
            &mut builder,
            &contraction_structure(),
            activations,
            weights,
        )
        .expect("the contraction applies");
        builder
            .output(
                OutputKey::new("projected").expect("the output key is valid"),
                projected,
            )
            .expect("the output binds");
        builder.build().expect("the program verifies")
    }

    /// **The L3 profile's own contraction cells compose a selected plan.**
    ///
    /// This is the reachability half of
    /// `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`, and it
    /// exists because that reachability was an *inference* from one number
    /// against another until something compiled the cells. The measured
    /// grid-axis row is 268,435,456 and `w_decode_kv` needs 1,024 threads, so
    /// the arithmetic is obvious — and the arithmetic is exactly what
    /// [`integrate-the-contraction-vertical-into-the-runtime`] could not run,
    /// because a bound admitting an extent and a *plan composing* at that extent
    /// are different claims and only the second is what a cell needs.
    ///
    /// **What this does and does not establish.** It establishes that each cell
    /// reaches a selected physical plan through the ordinary compiler entry
    /// point against the authoritative declaration. It dispatches nothing, so it
    /// says nothing about the executed bits and does not touch the retained
    /// `result_sha256` values; a device comparison at one cell is
    /// [`publish-an-l3-contraction-cell-through-the-accepted-route`], which
    /// needs the two prototype crates this crate cannot reach.
    ///
    /// **The refusal half is what stops this from passing whatever the row
    /// says.** `2 x 3 x 3` is named because the ticket recorded it refusing at
    /// `required: Threads(6)` under the superseded four-thread row, so it is the
    /// smallest witness that the old refusal is gone rather than merely
    /// out-scaled. The boundary pair at the end is mutation-proof in both
    /// directions: `16,384 x 16,384` is exactly 268,435,456 output elements and
    /// composes, and one column more refuses on `grid-axis` by name — so a
    /// refusal here is the bound doing its job rather than a shape that failed
    /// for some unrelated reason.
    ///
    /// Shapes are symbolic, so all six cells cost no more to compile than one.
    ///
    /// [`integrate-the-contraction-vertical-into-the-runtime`]:
    ///     ../../../tickets/integrate-the-contraction-vertical-into-the-runtime.md
    /// [`publish-an-l3-contraction-cell-through-the-accepted-route`]:
    ///     ../../../tickets/publish-an-l3-contraction-cell-through-the-accepted-route.md
    #[test]
    fn the_measured_grid_axis_admits_every_l3_contraction_cell() {
        let declaration = declaration();
        let targets =
            || TargetRequest::new([declaration.profile().clone()]).expect("a singleton request");

        for (id, m, n, k) in L3_CORRECTNESS_CELLS {
            let compilation = compiled(
                NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
                &contraction_program(m, n, k),
                targets(),
            );
            assert!(
                compilation.selected().is_some(),
                "the L3 cell {id} ({m}x{n}x{k}) compiled without reaching a selected plan",
            );
        }

        // The cell whose superseded refusal the owning ticket recorded verbatim.
        let small = compiled(
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            &contraction_program(2, 3, 3),
            targets(),
        );
        assert!(
            small.selected().is_some(),
            "2x3x3 published six output elements and was refused at `required: Threads(6)` under \
             the superseded four-thread row; it must compose now",
        );

        // The boundary, both sides. Exactly the declared bound composes...
        let at_bound = compiled(
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            &contraction_program(16_384, 16_384, 2),
            targets(),
        );
        assert!(
            at_bound.selected().is_some(),
            "268,435,456 output elements is exactly the declared grid-axis bound and must compose, \
             or the refusal below would not be attributable to the bound",
        );
        // ...and one element past it is refused, on the grid axis, by name.
        let refusal = compile(CompileRequest::new(
            &contraction_program(16_384, 16_385, 2),
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            targets(),
        ))
        .expect("the batch resolves")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1
        .expect_err("268,451,840 output elements exceed the declared grid-axis bound");
        let rendered = refusal
            .explain()
            .map_or_else(|| format!("{refusal:?}"), |report| report.render());
        assert!(
            rendered.contains("grid-axis:rejected"),
            "a contraction above the declared bound was refused, but not on the grid axis: \
             {rendered}",
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

    // The two-family delivery tests that lived here — one envelope carrying
    // one payload per artifact family, the per-position retention census, and
    // the swapped-position refusals — rested on the test-only second-family
    // fixture, whose premise ("the artifact family does not project into the
    // compiler profile") the required compilation-selection provenance
    // falsifies: a second family's production selection can never equal the
    // macOS-measured rows' recorded selection, so the fixture now refuses by
    // name (`a_second_artifact_family_cannot_wear_this_profiles_measured_rows`
    // in `metal_declaration`) instead of assembling. The multi-position
    // machinery evidence they carried is owed again under
    // `restore-multi-family-metal-delivery-evidence-under-per-family-profiles`.

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
