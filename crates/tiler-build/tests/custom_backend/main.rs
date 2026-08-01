//! One custom backend that is not Metal, producing a payload through `tiler-build`.
//!
//! # What this suite is evidence for
//!
//! That a statically linked producer outside every crate in the workspace can
//! consume verified compiler output through the promoted build-orchestration
//! seam, publish one canonical payload, and be unable to forge any identity the
//! plan already decided. It is the refutation test
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 names for itself: *a second backend needing to vary something
//! `assemble_artifact` derives rather than something it delegates would show the
//! split is in the wrong place*. This backend is that second backend.
//!
//! It is an integration test on purpose. It compiles against `tiler-build`'s
//! public surface alone, so a `pub(crate)` item is unreachable here in exactly
//! the way it is unreachable to a consumer; a `#[cfg(test)]` module beside the
//! facade could reach into the crate and would prove nothing about the boundary.
//!
//! # The four kinds of case
//!
//! *Positive* cases assert what the produced artifact says, and are written so a
//! change that quietly moved a derived fact would fail them rather than pass.
//! *Determinism* cases assemble twice and compare bytes and identities.
//! *Mutation* cases perturb exactly one producer statement and name the refusal
//! it must produce, and each was watched failing before it was believed.
//! *Derivation* cases assert that a fact the producer never supplied came from
//! the plan — the property that makes forgery structural rather than checked.

mod backend;
mod image;
mod partial_metal;
mod profile;

use backend::{EntryPerturbation, ScalarHostRefusal};
use image::{ScalarImageRefusal, encode};

use tiler_artifact::program::{
    ArtifactExecutionPolicy, DecodedArtifact, PayloadContent, PayloadMetadata, decode_artifact,
};
use tiler_cache::expansion::{ExpansionCache, Resolution};
use tiler_compiler::session::{
    Compilation, CompileRequest, NumericalContract, PlanAlternative, compile, compile_governed,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Builds the verified semantic graph every case here packages a plan for.
fn semantic_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([2, 3]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Compiles that graph against this backend's own declared target profile.
///
/// Nothing here is Metal's, and nothing in `crates/` was changed to admit it.
fn scalar_host_compilation(program: &SemanticProgram) -> Compilation {
    let profile = profile::scalar_host_profile().expect("the scalar-host profile declares");
    compile(CompileRequest::new(
        program,
        NumericalContract::StrictF32,
        TargetRequest::new([profile]).expect("a singleton target request"),
    ))
    .expect("the program compiles against the scalar-host profile")
    .into_targets()
    .pop()
    .expect("one target outcome")
    .into_parts()
    .1
    .expect("the scalar-host target compiles")
}

/// One complete production run: refuse, emit, describe, assemble.
struct Produced {
    artifact: tiler_artifact::program::VerifiedArtifactProgram,
    metadata: PayloadMetadata,
    bytes: Vec<u8>,
}

/// Runs this backend's four stages over one checked plan, in their fixed order.
fn produce(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    perturbation: EntryPerturbation,
    damage: Option<Vec<u8>>,
) -> Result<Produced, ScalarHostRefusal> {
    backend::require_compiled_under(plan.compilation())?;
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels)?;
    let metadata = backend::payload_metadata(&kernels, &image)?;
    let code = damage.unwrap_or_else(|| encode(&image));
    let artifact = backend::assemble(
        semantic,
        plan,
        PayloadContent {
            metadata: metadata.clone(),
            code,
        },
        perturbation,
    )?;
    let bytes = artifact
        .encode()
        .map_err(|error| ScalarHostRefusal::CacheEncoding(error.to_string()))?;
    Ok(Produced {
        artifact,
        metadata,
        bytes,
    })
}

/// Produces one sound artifact, panicking on any refusal.
fn sound(semantic: &SemanticProgram, plan: PlanAlternative<'_>) -> Produced {
    produce(semantic, plan, EntryPerturbation::default(), None)
        .expect("the sound production path completes")
}

fn scratch(label: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "tiler-build-custom-backend-{label}-{}-{:?}",
        std::process::id(),
        std::thread::current().id(),
    ));
    std::fs::create_dir_all(&path).expect("the scratch directory is creatable");
    path
}

// -------------------------------------------------------------------------
// Positive: the payload reaches an artifact and validates itself from bytes
// -------------------------------------------------------------------------

/// A non-Metal producer publishes a decoded, self-validating payload.
///
/// The assertions walk the whole result rather than checking that it exists:
/// the declared backend family and representation are this backend's, the
/// carried object decodes as this backend's image, every artifact entry reaches
/// a symbol the image declares, and the payload's transports are the
/// non-identity map the emitter wrote rather than the slot order.
#[test]
fn a_custom_backend_publishes_a_self_validating_payload() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let produced = sound(&semantic, plan);

    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");
    let [descriptor] = decoded.payloads() else {
        panic!("this backend declares exactly one payload");
    };
    assert_eq!(descriptor.backend.as_str(), profile::BACKEND_KEY);
    assert_eq!(
        descriptor.representation.as_str(),
        profile::REPRESENTATION_KEY
    );
    assert_eq!(
        descriptor.execution_policy,
        ArtifactExecutionPolicy::NativeImage
    );
    assert_eq!(
        backend::validate_from_bytes(&decoded),
        Ok(()),
        "the backend's own from-bytes validation must accept its own payload",
    );

    let variant = decoded.variants().next().expect("one packaged variant");
    let entry = variant.entries().next().expect("one packaged entry");
    assert_eq!(
        entry.transport_slots(),
        Some([1, 0].as_slice()),
        "the payload's transport map is the emitter's, not the binding slot order",
    );
    assert!(
        entry
            .backend_symbol()
            .is_some_and(|symbol| symbol.starts_with("scalar_host_")),
        "every entry must reach this backend's own identity-derived symbol",
    );
}

/// The three statements the backend supplies reach the artifact intact.
///
/// ADR 0090 item 11 names exactly these as what moves into what a backend
/// supplies. Asserting them here is what makes the promotion a change in
/// authority rather than a change in file layout: the launch precondition in
/// particular is a fact the standard Metal path never declares, so an artifact
/// carrying one could not have come from the orchestrator's own assumption.
#[test]
fn the_backend_supplied_launch_statements_reach_the_artifact() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let produced = sound(&semantic, plan);
    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");
    let variant = decoded.variants().next().expect("one packaged variant");

    for entry in variant.entries() {
        assert!(
            entry.zero_work_skips_dispatch(),
            "this backend declares a zero-thread launch skippable",
        );
        assert_eq!(
            entry.launch_preconditions().len(),
            1,
            "this backend declares exactly one launch-time precondition",
        );
        assert_eq!(
            entry.bindings().len(),
            2,
            "one read and one write binding, in kernel buffer-parameter order",
        );
    }
}

// -------------------------------------------------------------------------
// Derivation: the facts a producer never supplied came from the plan
// -------------------------------------------------------------------------

/// Every derived subject follows the compilation rather than the producer.
///
/// The producer states no target profile, no feasibility rules, no selected
/// provider, and no entry key — there is no parameter for any of them — so this
/// asserts the artifact carries the compilation's own values. A facade that
/// accepted producer statements for these would still pass every test above.
#[test]
fn the_derived_subjects_follow_the_compilation_and_not_the_producer() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let produced = sound(&semantic, plan);
    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");
    let variant = decoded.variants().next().expect("one packaged variant");

    assert_eq!(variant.target_profile().key.as_str(), profile::PROFILE_KEY);
    assert_eq!(
        variant.target_profile().descriptor.as_bytes(),
        compilation.target_profile_descriptor(),
        "the exact descriptor identity is the compilation's, not a producer claim",
    );
    assert_eq!(
        variant.feasibility_rules().key.as_str(),
        compilation.feasibility_rule_set_key(),
    );
    assert_eq!(
        variant.feasibility_rules().revision,
        compilation.feasibility_rule_set_revision(),
    );
    let [descriptor] = decoded.payloads() else {
        panic!("this backend declares exactly one payload");
    };
    assert_eq!(
        descriptor.compatibility.descriptor.as_bytes(),
        compilation.target_profile_descriptor(),
        "the payload's compatibility profile is handed to the producer, not chosen by it",
    );

    let identities: Vec<&[u8]> = kernels
        .iter()
        .map(|kernel| kernel.canonical_identity().as_bytes())
        .collect();
    for entry in variant.entries() {
        assert!(
            identities.contains(&entry.backend_entry_key().as_bytes()),
            "every entry key is a stage kernel's canonical identity",
        );
    }
}

/// This backend's profile makes the workgroup bound a compile-time fact.
///
/// Metal cannot: only a built pipeline knows its own maximum, so the standard
/// path carries one deferred predicate per entry. Asserting zero here proves the
/// facade carries what the *plan* minted rather than a Metal-shaped assumption,
/// and it is the branch the standard path never exercises.
#[test]
fn a_compile_time_workgroup_bound_mints_no_deferred_predicate() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    assert_eq!(
        plan.prepared_entry_target_requirements().len(),
        0,
        "the scalar-host profile answers the workgroup bound at declaration time",
    );
    let produced = sound(&semantic, plan);
    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");
    let variant = decoded.variants().next().expect("one packaged variant");
    assert_eq!(variant.deferred_predicates().len(), 0);
}

// -------------------------------------------------------------------------
// Determinism, and publication through the expansion cache
// -------------------------------------------------------------------------

/// Two assemblies of one plan produce identical bytes and one identity.
#[test]
fn two_assemblies_of_one_plan_are_byte_identical() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");

    let first = sound(&semantic, plan);
    let second = sound(&semantic, plan);

    assert_eq!(
        first.bytes, second.bytes,
        "the produced envelope must be a function of the plan and the payload",
    );
    assert_eq!(
        first.artifact.canonical_identity().as_bytes(),
        second.artifact.canonical_identity().as_bytes(),
    );
}

/// The payload publishes once and is accepted from the cache thereafter.
///
/// The subject is composed from the payload's derived compilation digest and the
/// artifact's canonical identity, both of which the producer receives rather
/// than states. The hit is checked back against the published identity, so a
/// cache returning another artifact under this subject is a hard refusal rather
/// than a silent substitution.
#[test]
fn a_custom_payload_publishes_then_is_accepted_from_the_cache() {
    let directory = scratch("publish");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let produced = sound(&semantic, plan);
    let digest = produced
        .metadata
        .identity()
        .expect("the metadata derives its own digest");

    let mut outcomes = Vec::new();
    for _ in 0..2 {
        let resolution = backend::accept_or_publish(&cache, &produced.artifact, digest.as_bytes())
            .expect("the custom payload resolves");
        outcomes.push(match resolution {
            Resolution::Published { .. } => "published",
            Resolution::Hit { .. } => "hit",
            Resolution::Uncached { .. } => "uncached",
        });
    }
    assert_eq!(outcomes, ["published", "hit"]);
    let _ = std::fs::remove_dir_all(directory);
}

/// A cache entry holding another artifact under this subject is a hard refusal.
///
/// The perturbation is the *cache*, not the producer: an entry for one
/// artifact's subject is filled with a different artifact's envelope, which is
/// what a corrupted store or a subject that under-keyed its facets would look
/// like. Accepting it would return bytes a host would then execute under the
/// identity it asked for, so the re-check after resolution is the whole
/// protection and it must fire rather than degrade into a rebuild.
#[test]
fn a_cache_entry_naming_another_artifact_is_refused_rather_than_accepted() {
    let directory = scratch("subject-disagreement");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels).expect("the kernels translate");
    let expected = backend::payload_metadata(&kernels, &image).expect("a payload subject");
    let mut other = expected.clone();
    other.provenance.compile_flags = vec!["-other".to_owned()];

    let expected_artifact = backend::assemble(
        &semantic,
        plan,
        PayloadContent {
            metadata: expected.clone(),
            code: encode(&image),
        },
        EntryPerturbation::default(),
    )
    .expect("the expected artifact assembles");
    let other_artifact = backend::assemble(
        &semantic,
        plan,
        PayloadContent {
            metadata: other,
            code: encode(&image),
        },
        EntryPerturbation::default(),
    )
    .expect("the other artifact assembles");
    let digest = expected.identity().expect("a derived payload digest");

    // Fill the expected artifact's own subject with the other artifact's bytes.
    backend::publish_under_foreign_subject(
        &cache,
        &expected_artifact,
        digest.as_bytes(),
        &other_artifact,
    )
    .expect("the perturbed entry publishes");

    let refusal = backend::accept_or_publish(&cache, &expected_artifact, digest.as_bytes())
        .expect_err("a cache entry naming another artifact cannot be accepted");
    assert_eq!(refusal, ScalarHostRefusal::CacheIdentity);
    let _ = std::fs::remove_dir_all(directory);
}

/// A moved compilation subject files under a new key rather than reusing one.
///
/// The producer cannot suppress this: the digest is derived from the metadata's
/// canonical bytes by the artifact layer, so perturbing one provenance field
/// moves the payload digest, the artifact identity, and the cache subject
/// together. A design that let a producer stamp its own digest would let a
/// changed compilation land on the earlier compilation's cache entry.
#[test]
fn a_perturbed_compilation_subject_moves_every_derived_identity() {
    let directory = scratch("subject-move");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels).expect("the kernels translate");

    let sound_metadata = backend::payload_metadata(&kernels, &image).expect("a payload subject");
    let mut moved_metadata = sound_metadata.clone();
    moved_metadata.provenance.compile_flags = vec!["-scalar-host-opt".to_owned()];

    let mut identities = Vec::new();
    let mut digests = Vec::new();
    for metadata in [sound_metadata, moved_metadata] {
        let digest = metadata.identity().expect("a derived payload digest");
        let artifact = backend::assemble(
            &semantic,
            plan,
            PayloadContent {
                metadata,
                code: encode(&image),
            },
            EntryPerturbation::default(),
        )
        .expect("both subjects assemble");
        backend::accept_or_publish(&cache, &artifact, digest.as_bytes())
            .expect("both subjects resolve");
        identities.push(artifact.canonical_identity().as_bytes().to_vec());
        digests.push(digest.as_bytes().to_vec());
    }

    assert_ne!(digests[0], digests[1], "the payload digest must move");
    assert_ne!(
        identities[0], identities[1],
        "the artifact identity must move with the payload digest",
    );
    let _ = std::fs::remove_dir_all(directory);
}

// -------------------------------------------------------------------------
// Mutation: every producer statement that must be refused
// -------------------------------------------------------------------------

/// A plan compiled under another profile is refused before any translation.
///
/// The perturbation is the *compilation*: the same graph and the same backend,
/// compiled against the compiler's governed prototype profile. The refusal names
/// the key rather than the descriptor, because a foreign target and a stale
/// revision of this one carry different remedies.
#[test]
fn a_plan_compiled_under_another_profile_is_refused_before_translation() {
    let semantic = semantic_program();
    let foreign = compile_governed(&semantic, NumericalContract::StrictF32)
        .expect("the governed prototype profile compiles this program");
    let plan = foreign.selected().expect("one selected plan");
    let refusal = produce(&semantic, plan, EntryPerturbation::default(), None)
        .err()
        .expect("a plan compiled under another profile cannot be translated here");
    assert!(
        matches!(refusal, ScalarHostRefusal::ForeignProfileKey { .. }),
        "unexpected refusal: {refusal:?}",
    );
}

/// A declared binding count other than the stage's is a typed refusal.
///
/// This is the one cardinality the facade cannot make structural: the number of
/// *entries* is fixed by calling the backend once per stage, but how many
/// bindings each entry declares is genuinely the backend's statement, and the
/// artifact builder proves it against the stage's own buffer parameters.
#[test]
fn a_binding_count_that_disagrees_with_the_stage_is_refused() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let refusal = produce(
        &semantic,
        plan,
        EntryPerturbation {
            bindings: Some(3),
            forbid_zero_work_skip: false,
        },
        None,
    )
    .err()
    .expect("a surplus binding declaration cannot be packaged");
    let ScalarHostRefusal::Assembly(message) = &refusal else {
        panic!("unexpected refusal: {refusal:?}");
    };
    assert_eq!(
        message,
        "artifact assembly failed: BindingCardinality { entry: 0, expected: 2, actual: 3 }",
        "the refusal must name the entry and both counts, not merely that something failed",
    );
}

/// A forged entry mapping is refused when the envelope is decoded.
///
/// The producer states which *symbol* realizes each entry key; the entry key
/// itself is derived. Renaming the mapped key is therefore the only forgery
/// available here, and it is caught: the artifact builds and encodes, and the
/// decoder refuses an entry that reaches no mapping. That the refusal is at
/// decode rather than at build is the honest boundary — the builder never
/// re-reads a carried payload's metadata.
#[test]
fn a_forged_entry_mapping_is_refused_when_the_envelope_decodes() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels).expect("the kernels translate");
    let mut metadata = backend::payload_metadata(&kernels, &image).expect("a payload subject");
    metadata.entries[0].entry_key =
        tiler_artifact::program::BackendEntryKey::from_bytes(b"a key no kernel minted")
            .expect("an opaque key");

    let artifact = backend::assemble(
        &semantic,
        plan,
        PayloadContent {
            metadata,
            code: encode(&image),
        },
        EntryPerturbation::default(),
    )
    .expect("the artifact layer does not re-read a carried mapping at build time");
    let bytes = artifact.encode().expect("the artifact encodes");
    let failure =
        decode_artifact(&bytes).expect_err("an entry reaching no mapping must be refused");
    assert_eq!(
        format!("{failure}"),
        "artifact.invalid: UnmappedBackendEntry { payload: 0 }",
        "the refusal must name the payload whose mapping omitted the entry",
    );
}

/// Declaring one payload twice is a typed duplicate rather than two payloads.
#[test]
fn declaring_one_payload_twice_is_refused() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels).expect("the kernels translate");
    let metadata = backend::payload_metadata(&kernels, &image).expect("a payload subject");
    let content = PayloadContent {
        metadata,
        code: encode(&image),
    };

    let refusal = tiler_build::assemble_plan_artifact(
        &semantic,
        plan,
        |builder, profile| {
            builder.push_carried_payload(
                backend::backend(),
                backend::representation(),
                backend::PAYLOAD_SCHEMA,
                profile.clone(),
                ArtifactExecutionPolicy::NativeImage,
                content.clone(),
            )?;
            builder.push_carried_payload(
                backend::backend(),
                backend::representation(),
                backend::PAYLOAD_SCHEMA,
                profile,
                ArtifactExecutionPolicy::NativeImage,
                content,
            )
        },
        |_, stage| {
            Ok(tiler_build::BackendEntryDeclaration {
                bindings: vec![
                    tiler_artifact::program::BindingKind::Buffer;
                    stage.accesses().len()
                ],
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            })
        },
    )
    .expect_err("two identical payload declarations cannot both be admitted");
    assert_eq!(
        format!("{refusal}"),
        "artifact assembly failed: DuplicatePayload",
        "a repeated descriptor is a typed duplicate rather than a second payload",
    );
}

/// A damaged payload passes every artifact-layer check and this backend's none.
///
/// Artifact identity deliberately excludes the emitted object, so the damaged
/// artifact carries the *same* canonical identity as the sound one — which is
/// precisely why the backend has to validate from bytes, and precisely what a
/// design that expected the artifact layer to catch it would get wrong.
#[test]
fn a_damaged_payload_shares_one_identity_and_is_caught_only_by_the_backend() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let sound_run = sound(&semantic, plan);

    for (label, damage, expected) in damage_cases(&semantic, plan) {
        let damaged = produce(&semantic, plan, EntryPerturbation::default(), Some(damage))
            .unwrap_or_else(|error| panic!("{label}: a damaged object still assembles: {error:?}"));
        assert_eq!(
            damaged.artifact.canonical_identity().as_bytes(),
            sound_run.artifact.canonical_identity().as_bytes(),
            "{label}: artifact identity must exclude the emitted object",
        );
        let decoded: DecodedArtifact =
            decode_artifact(&damaged.bytes).expect("the damaged envelope still decodes");
        assert_eq!(
            backend::validate_from_bytes(&decoded),
            Err(expected),
            "{label}: the backend's own validation must refuse it",
        );
    }
}

/// The damaged-object cases, each perturbing one property of the image alone.
fn damage_cases(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
) -> Vec<(&'static str, Vec<u8>, ScalarHostRefusal)> {
    let _ = semantic;
    let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    let image = backend::emit(&kernels).expect("the kernels translate");
    let sound_bytes = encode(&image);

    let mut foreign = sound_bytes.clone();
    foreign[0] = b'x';

    let mut truncated = sound_bytes.clone();
    truncated.pop();

    let mut trailing = sound_bytes.clone();
    trailing.push(0);

    let mut aliased = image.clone();
    aliased.entries[0].transports = vec![0, 0];

    let mut renamed = image.clone();
    "scalar_host_0000000000000000".clone_into(&mut renamed.entries[0].symbol);

    let mut moved = image;
    moved.entries[0].transports = vec![0, 1];

    vec![
        (
            "a foreign domain separator",
            foreign,
            ScalarHostRefusal::Payload(ScalarImageRefusal::ForeignDomain),
        ),
        (
            "a truncated image",
            truncated,
            ScalarHostRefusal::Payload(ScalarImageRefusal::Truncated),
        ),
        (
            "trailing bytes",
            trailing,
            ScalarHostRefusal::Payload(ScalarImageRefusal::TrailingBytes),
        ),
        (
            "two bindings on one transport",
            encode(&aliased),
            ScalarHostRefusal::Payload(ScalarImageRefusal::AliasedTransport),
        ),
        (
            "a symbol the artifact does not name",
            encode(&renamed),
            ScalarHostRefusal::UnmappedSymbol,
        ),
        (
            "a transport map the artifact does not state",
            encode(&moved),
            ScalarHostRefusal::TransportDisagreement,
        ),
    ]
}

// -------------------------------------------------------------------------
// Partial composition: stock Metal emission, one provider's own orchestration
// -------------------------------------------------------------------------

/// A partial Metal provider reuses stock emission and varies orchestration alone.
///
/// The equal payload digest is what proves the reuse was real rather than
/// coincidental — it is derived from the emitted source, the resolved toolchain,
/// the exact flags, and the entry mapping, so an equal digest means the same
/// emission and the same AOT preparation ran. The unequal artifact identity is
/// what proves the varied half varied. Both halves reached the provider by an
/// ordinary Cargo edge with nothing mediating them.
#[test]
fn a_partial_metal_provider_reuses_stock_emission_and_varies_only_orchestration() {
    let directory = scratch("partial-metal");
    let cache = ExpansionCache::open(directory.join("cache"));
    let toolchain = fake_metal_toolchain(&directory);
    let declaration = tiler_build::BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the authoritative macOS declaration assembles");
    let semantic = semantic_program();
    let compilation = partial_metal::metal_compilation(&declaration, &semantic);
    let plan = compilation.selected().expect("one selected plan");

    let standard = tiler_build::accept_or_publish_metal_plan(
        &cache,
        &toolchain,
        &semantic,
        plan,
        &declaration,
        tiler_metal_aot::input::OptimizationLevel::Default,
    )
    .expect("the standard Metal path resolves");
    let partial = partial_metal::assemble(&toolchain, &declaration, &semantic, &compilation);

    let [standard_payload] = standard.artifact().payloads() else {
        panic!("the standard Metal path declares exactly one payload");
    };
    let [partial_payload] = partial.payloads() else {
        panic!("the partial provider declares exactly one payload");
    };
    assert_eq!(
        standard_payload.digest, partial_payload.digest,
        "the reused emission and AOT preparation must yield one compilation subject",
    );
    assert_eq!(standard_payload.backend, partial_payload.backend);
    assert_eq!(
        standard_payload.representation,
        partial_payload.representation
    );
    assert_ne!(
        standard.artifact().canonical_identity().as_bytes(),
        partial.canonical_identity().as_bytes(),
        "a backend's launch statements are folded into artifact identity",
    );

    let bytes = partial.encode().expect("the partial artifact encodes");
    let decoded = decode_artifact(&bytes).expect("the partial artifact decodes");
    let entry = decoded
        .variants()
        .next()
        .expect("one packaged variant")
        .entries()
        .next()
        .expect("one packaged entry");
    assert_eq!(
        entry.launch_preconditions().len(),
        1,
        "the provider's own launch precondition reaches the envelope",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// Writes the fake Apple toolchain the Metal halves resolve through.
///
/// The suite runs offline and must not depend on an installed Xcode, so the
/// tools are shell scripts that report fixed versions and write fixed object
/// bytes. What is under test is orchestration, not the Apple compiler.
fn fake_metal_toolchain(directory: &std::path::Path) -> tiler_metal_aot::driver::Toolchain {
    use std::os::unix::fs::PermissionsExt as _;

    let write = |path: &std::path::Path, body: &str| {
        std::fs::write(path, body).expect("the fake tool is writable");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("the fake tool is executable");
    };
    let metal = directory.join("metal");
    let metallib = directory.join("metallib");
    let launcher = directory.join("xcrun");
    write(
        &metal,
        "#!/bin/sh\n\
         if [ \"$1\" = \"--version\" ]; then echo 'Metal partial-v1'; exit 0; fi\n\
         while [ \"$#\" -gt 0 ]; do\n\
           if [ \"$1\" = \"-o\" ]; then shift; printf AIR > \"$1\"; exit 0; fi\n\
           shift\n\
         done\n\
         exit 1\n",
    );
    write(
        &metallib,
        "#!/bin/sh\n\
         if [ \"$1\" = \"--version\" ]; then echo 'metallib partial-v1'; exit 0; fi\n\
         while [ \"$#\" -gt 0 ]; do\n\
           if [ \"$1\" = \"-o\" ]; then shift; printf MTLBpartial > \"$1\"; exit 0; fi\n\
           shift\n\
         done\n\
         exit 1\n",
    );
    write(
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
    tiler_metal_aot::driver::Toolchain::with_launcher(launcher)
}

/// A kernel this bounded backend cannot place is refused during translation.
///
/// Emission is where a backend's own limits live, and this one binds exactly one
/// read and one write. The wide kernel is built by hand rather than compiled,
/// because this backend's own profile declares two buffer bindings and
/// feasibility therefore never hands it a wider one — which is the point. A
/// translator must not assume the check upstream of it ran; the arm exists to
/// refuse a kernel that reached it anyway, and an arm that cannot be watched
/// failing is not evidence of anything.
#[test]
fn a_kernel_this_backend_cannot_place_is_refused_during_translation() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let placeable: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
    assert!(
        backend::emit(&placeable).is_ok(),
        "the accepted neighbour: the compiled kernel translates",
    );

    let wide = wide_kernel();
    let refusal = backend::emit(&[placeable[0], &wide])
        .expect_err("a three-buffer signature cannot be placed by this backend");
    assert_eq!(
        refusal,
        ScalarHostRefusal::UnsupportedBufferCount {
            entry: 1,
            buffers: 3
        },
        "the refusal must name which kernel it was rather than that something failed",
    );
}

/// Builds a three-buffer kernel this backend cannot place.
///
/// Two reads and one write over a single element. Nothing compiles to this
/// against the scalar-host profile; it exists so the emitter's refusal arm has a
/// case that must fail.
fn wide_kernel() -> VerifiedKernel {
    use tiler_ir::kernel::lower_scheduled_region;
    use tiler_ir::schedule::{
        Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId,
        ExceptionalValueAssumption, ExecutionBinding, InputOrdinal, KernelSchedule, LaunchPlan,
        LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
        RegionId, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
    };

    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region
        .iteration_shape(Shape::from_dims([1]))
        .expect("the iteration shape binds");
    let accesses = [
        (
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            AccessMode::Read,
            0,
            None,
        ),
        (
            TensorRole::Input {
                ordinal: InputOrdinal::new(1),
            },
            AccessMode::Read,
            1,
            None,
        ),
        (
            TensorRole::Intermediate,
            AccessMode::Write,
            2,
            Some(OwnershipWitnessId::new(0)),
        ),
    ];
    for (tensor, mode, bounds, ownership) in accesses {
        region
            .push_access(Access {
                tensor,
                component_role: None,
                mode,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(bounds),
                ownership,
            })
            .expect("the access binds");
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(bounds),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 1 },
            })
            .expect("the bounds proof binds");
    }
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 1 },
        })
        .expect("the ownership proof binds");

    let mut expression = PointwiseF32ExpressionBuilder::new();
    let first = expression
        .input(InputOrdinal::FIRST)
        .expect("the first pointwise input");
    let second = expression
        .input(InputOrdinal::new(1))
        .expect("the second pointwise input");
    let root = expression.add(first, second).expect("the pointwise sum");
    let expression = expression.build(root).expect("the expression verifies");
    region
        .scalar_program(ScalarProgram::PointwiseF32(expression))
        .expect("the scalar program binds");
    region
        .numerical(NumericalRealization::new(
            "tiler.test.scalar-host-wide",
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
    region
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
    lower_scheduled_region(&region.build().expect("the region verifies"))
        .expect("the region lowers")
}
