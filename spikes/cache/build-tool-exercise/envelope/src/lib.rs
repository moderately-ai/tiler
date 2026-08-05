//! Compiles and encodes one real artifact envelope.
//!
//! This is the expensive step the expansion cache exists to spare, and it is a
//! genuine one: it runs `tiler-compiler`'s governed session and encodes the
//! result through `tiler-artifact`, so bytes produced here are accepted by the
//! real `decode_artifact` that
//! [`ExpansionCache`](tiler_cache::expansion::ExpansionCache) validates every hit
//! against.
//!
//! # Why nothing here is memoized
//!
//! A `OnceLock` around the compilation would make repeat expansions inside one
//! long-lived proc-macro server cheap — and would hide the exact quantity this
//! spike measures. The cache's claim is that it suppresses duplicate
//! *compilation*; an in-process memo would suppress it first and report a
//! success the cache did not earn.
//!
//! # What it does not use
//!
//! No Metal toolchain and no device. `tiler-compiler` depends only on
//! `tiler-ir`, so the whole path is host computation and the spike runs anywhere
//! the workspace builds. The payload is declared by descriptor rather than
//! carried, which is what keeps a real compiled object out of the picture — the
//! envelope is assembled to be identified, which is all a cache key and a
//! validation pass need.

use tiler_artifact::program::{
    ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey, BackendEntryRef, BackendKey,
    BackendPayloadDescriptor, BindingKind, BindingSpec, CapabilityKey, CompilationEnvironment,
    EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec, PayloadDigest,
    RepresentationKey, SchemaVersion, SelectedProvider, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, VariantSpec, VerifiedArtifactProgram,
};
use tiler_compiler::session::{
    Compilation, NumericalContract, PlanAlternative, compile_governed,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Rows of the exercised program's input.
const ROWS: u64 = 4;
/// Columns of the exercised program's input.
const COLUMNS: u64 = 3;

/// Compiles, assembles, and encodes one artifact envelope.
///
/// # Panics
///
/// Panics when the governed program fails to compile or the assembled artifact
/// fails to verify or encode. Each of those is a defect in this spike or in the
/// workspace it builds against, never a cache outcome, so failing loudly here
/// keeps it distinguishable from the cache falling open.
#[must_use]
pub fn encoded_envelope() -> Vec<u8> {
    let semantic = serial_sum_program(ROWS, COLUMNS);
    let compilation = compile_governed(&semantic, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
        .expect("the governed program compiles");
    let plan = compilation.selected().expect("a selected plan alternative");
    assemble(&semantic, &compilation, plan)
        .encode()
        .expect("the envelope encodes")
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Packages one plan alternative and a declared payload as an artifact.
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
) -> VerifiedArtifactProgram {
    let profile = TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .expect("the compiler mints a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .expect("the compiler mints a profile descriptor"),
    };
    let rules = FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())
            .expect("the compiler mints a governed rule-set key"),
        revision: compilation.feasibility_rule_set_revision(),
    };

    let environment = CompilationEnvironment::new(
        plan.selected_capabilities()
            .map(|selected| selected.provider().clone()),
    )
    .expect("the offered providers compose an environment");
    let mut builder =
        ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
    for selected in plan.selected_capabilities() {
        builder
            .select_provider(SelectedProvider {
                provider: selected.provider().clone(),
                capability: CapabilityKey::new(selected.capability_key())
                    .expect("the compiler mints a governed capability key"),
                capability_revision: selected.capability_revision(),
            })
            .expect("a selected provider was offered");
    }

    let payload = builder
        .push_payload(BackendPayloadDescriptor {
            backend: BackendKey::new("tiler.metal").expect("a governed backend key"),
            representation: RepresentationKey::new("metallib")
                .expect("a governed representation key"),
            payload_schema: SchemaVersion::new(1, 0),
            digest: PayloadDigest::from_bytes([0xe1, 0xe2, 0xe3])
                .expect("a bounded payload digest"),
            compatibility: profile.clone(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
        })
        .expect("the declared payload is accepted");

    let program = plan.abi().kernel_program();

    let entries: Vec<EntrySpec> = program
        .stages()
        .map(|stage| EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                // Not a choice: every verified scheduled region carries it.
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payloads: vec![payload],
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .expect("the packaged kernel identity fits a backend entry key"),
            },
        })
        .collect();

    builder
        .push_variant(
            program,
            VariantSpec {
                target_profile: profile,
                feasibility_rules: rules,
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .expect("the variant packages the plan it was built from");
    builder.build().expect("the assembled artifact verifies")
}
