//! A partial custom Metal provider: stock emission, its own orchestration.
//!
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 4 fixes that reusing another backend's emitter is an ordinary Cargo edge
//! and must stay one, and its item 11 promotes the orchestration seam alone.
//! This module is the two halves put together: the provider takes stock
//! `tiler_metal::emit`, stock `tiler_build` AOT preparation, and the governed
//! macOS declaration entirely unchanged, and varies only what the *orchestrator*
//! used to assume — it declares a launch-time precondition the standard path
//! does not.
//!
//! # What the assertion is for
//!
//! The two artifacts must agree on the payload digest and disagree on artifact
//! identity. Agreement proves the reused halves really were reused: the payload
//! digest is derived from the compilation subject — the emitted source, the
//! resolved toolchain, the flags, the entry mapping — so an equal digest means
//! this provider ran the same emission and the same AOT preparation the standard
//! path ran. Disagreement proves the varied half really varied, and that a
//! backend's launch statements are folded into artifact identity rather than
//! being decoration.

use tiler_artifact::program::{
    AbiBinaryOp, AbiRoot, ArtifactBuildError, ArtifactProgramBuilder, AvailabilityPhase,
    BindingKind, TargetPropertyKey, VerifiedArtifactProgram,
};
use tiler_build::{
    BackendEntryDeclaration, BoundMetalCompileDeclaration, assemble_plan_artifact,
    metal_compile_request, prepare_metal_payload,
};
use tiler_compiler::session::{Compilation, CompileRequest, NumericalContract, compile};
use tiler_compiler::target::TargetRequest;
use tiler_ir::semantic::SemanticProgram;
use tiler_metal::emit::emit_translation_unit;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::OptimizationLevel;

/// Governed launch-time property this provider's entries place a floor on.
const RESIDENCY_PROPERTY_KEY: &str = "tiler.test.partial-metal.launch.resident-bytes";

/// Resident bytes this provider's entries require at launch.
const RESIDENT_BYTES_FLOOR: u64 = 65_536;

/// Compiles one program against the governed macOS Metal declaration's profile.
///
/// # Panics
///
/// Panics when the authoritative declaration does not compile the program, which
/// is a defect in this file rather than a case under test.
#[must_use]
pub fn metal_compilation(
    declaration: &BoundMetalCompileDeclaration,
    program: &SemanticProgram,
) -> Compilation {
    compile(CompileRequest::new(
        program,
        NumericalContract::FlushSubnormalsToZeroF32,
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request"),
    ))
    .expect("the program compiles against the authoritative Metal profile")
    .into_targets()
    .pop()
    .expect("one target outcome")
    .into_parts()
    .1
    .expect("the authoritative Metal target compiles")
}

/// Produces one artifact through stock Metal emission and this provider's own
/// orchestration.
///
/// # Panics
///
/// Panics on any refusal from the reused halves. A partial provider that cannot
/// drive stock Metal emission is the finding, not a case, and the panic message
/// names which stage refused.
#[must_use]
pub fn assemble(
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    semantic: &SemanticProgram,
    compilation: &Compilation,
) -> VerifiedArtifactProgram {
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<_> = plan.kernels().iter().collect();
    // Stock, unmediated, and reached by an ordinary Cargo edge.
    let unit = emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
        .expect("stock Metal emission accepts the selected kernels");
    let request = metal_compile_request(
        &unit,
        OptimizationLevel::Default,
        declaration.numerical_realization(),
    )
    .expect("stock request derivation accepts the emitted unit");
    let prepared = toolchain
        .prepare(&request)
        .expect("the fake toolchain prepares");
    let payload =
        prepare_metal_payload(&unit, prepared).expect("the emitted and prepared facts agree");
    let compiled = payload.compile().expect("the prepared operation compiles");

    assemble_plan_artifact(
        semantic,
        plan,
        |builder, profile| compiled.push_carried(builder, profile),
        |builder, stage| {
            Ok(BackendEntryDeclaration {
                bindings: stage.accesses().map(|_| BindingKind::Buffer).collect(),
                zero_work_skips_dispatch: true,
                preconditions: vec![residency_precondition(builder)?],
            })
        },
    )
    .expect("the partial provider's artifact assembles")
}

/// Mints this provider's launch-time residency floor on the facade's builder.
fn residency_precondition(
    builder: &mut ArtifactProgramBuilder,
) -> Result<tiler_artifact::program::AbiExprId, ArtifactBuildError> {
    let observed = builder.push_root(AbiRoot::TargetProperty {
        key: TargetPropertyKey::new(RESIDENCY_PROPERTY_KEY).expect("a governed property key"),
        phase: AvailabilityPhase::LaunchPreflight,
    })?;
    let required = builder.push_root(AbiRoot::UnsignedLiteral(RESIDENT_BYTES_FLOOR))?;
    builder.push_binary(AbiBinaryOp::LessOrEqual, required, observed)
}
