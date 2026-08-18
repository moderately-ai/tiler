//! One custom backend that is not Metal, producing a payload through `tiler-build`.
//!
//! # What this suite is evidence for
//!
//! That a statically linked producer outside every crate in the workspace can
//! consume verified compiler output through the promoted build-orchestration
//! seams — both of them — publish one canonical payload under a complete cache
//! subject, re-accept it from the cache, and be unable to forge any identity the
//! plan already decided. It is the refutation test
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 11 names for itself: *a second backend needing to vary something
//! `assemble_artifact` derives rather than something it delegates would show the
//! split is in the wrong place*. This backend is that second backend, and it
//! shares no code with the Metal path on either seam.
//!
//! It is an integration test on purpose. It compiles against `tiler-build`'s
//! public surface alone, so a `pub(crate)` item is unreachable here in exactly
//! the way it is unreachable to a consumer; a `#[cfg(test)]` module beside the
//! facade could reach into the crate and would prove nothing about the boundary.
//!
//! # The five kinds of case
//!
//! *Positive* cases assert what the produced artifact says, and are written so a
//! change that quietly moved a derived fact would fail them rather than pass.
//! *Determinism* cases assemble twice and compare bytes and identities.
//! *Mutation* cases perturb exactly one producer statement and name the refusal
//! it must produce, and each was watched failing before it was believed.
//! *Derivation* cases assert that a fact the producer never supplied came from
//! the plan — the property that makes forgery structural rather than checked.
//! *Cache* cases move exactly one operand of the promoted cache seam and name
//! the refusal, and each of the seam's own comparisons was separately disabled
//! and watched being caught by exactly one of them.

mod backend;
mod image;
mod partial_metal;
mod profile;

use std::cell::Cell;

use backend::{
    EntryPerturbation, PayloadDeclaration, PreparedScalarPayload, ScalarHostFact, ScalarHostRefusal,
};
use image::{ScalarImageRefusal, encode};

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactExecutionPolicy, CANONICAL_DIMENSIONS, DecodedArtifact,
    DimensionBehaviour, DispositionView, NumericalDimension, NumericalPermission, PayloadContent,
    PayloadMetadata, PayloadPlatform, PolicyLocus, ProvenanceField, RepresentationKey,
    ScalarArithmeticSubject, VerifiedArtifactProgram, decode_artifact,
};
use tiler_build::{
    AcceptedArtifact, CompiledPayloads, DeliveredPayloadCacheError, DeliveredPayloadProtocolError,
    accept_or_publish_delivered_payload_artifact,
};
use tiler_cache::expansion::{
    DebugRetention, ExpansionCache, Resolution, SubjectFacet, SubjectRefusal,
};
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

/// The exact refusal type this backend instantiates the promoted seam with.
///
/// Three authorities and three type arguments: which compilation fact this
/// backend compared, why its own compile step failed, and why its assembly did.
type SeamRefusal = DeliveredPayloadCacheError<ScalarHostFact, ScalarHostRefusal, ScalarHostRefusal>;

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

/// Builds the bare reduction graph: one input and one strict serial sum.
///
/// [`semantic_program`] without the scaling multiply and the bias add, so the
/// only occurrence a plan covers is the fold. The compiler's reduction
/// capability row omits contraction — a strict serial sum's per-contributor step
/// is `accumulator + contributor`, with no product for a fused multiply-add to
/// act on — so the honoured contraction fact founds no position anywhere in this
/// program and the packaged record carries no obligation naming it.
fn bare_reduction_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([2, 3]),
        )
        .expect("the input binds");
    let sum = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).expect("the sum");
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
        NumericalContract::STRICT_F32,
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
    artifact: VerifiedArtifactProgram,
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
        PayloadContent { metadata, code },
        perturbation,
    )?;
    let bytes = artifact
        .encode()
        .map_err(|error| ScalarHostRefusal::CacheEncoding(error.to_string()))?;
    Ok(Produced { artifact, bytes })
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
        entry.transport_slots(0),
        Some([1, 0].as_slice()),
        "the payload's transport map is the emitter's, not the binding slot order",
    );
    assert!(
        entry
            .backend_symbol(0)
            .is_some_and(|symbol| symbol.starts_with("scalar_host_")),
        "every entry must reach this backend's own identity-derived symbol",
    );
}

/// This backend's provenance is complete without an Apple-shaped placeholder.
///
/// The first half is the outcome `generalize-payload-provenance-beyond-the-apple-shape`
/// exists for: an out-of-crate, non-Metal backend states that its toolchain
/// resolved against no SDK, and the decoded payload carries that statement
/// rather than a minted SDK name and a deployment minimum standing in for one.
/// Nothing in `crates/` knows this backend, so the record is neutral rather than
/// accommodating.
///
/// The second half is the other side of the same rule and the reason the first
/// is not a weakening: what a payload owes follows the shape it declares, so
/// this backend still owes its toolchain, target, family, language, and every
/// component field — and dropping one leaves the payload with no identity, named
/// by field. What it does *not* owe are the four fields it has no referent for.
#[test]
fn a_non_metal_backend_states_no_sdk_and_still_owes_the_rest() {
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let produced = sound(&semantic, plan);

    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");
    let metadata = decoded
        .payload_metadata(0)
        .expect("this backend carries its payload");
    assert_eq!(
        metadata.provenance.platform,
        PayloadPlatform::Unversioned,
        "a backend whose toolchain has no SDK must be able to say so",
    );
    metadata
        .identity()
        .expect("a payload that owes no SDK field is complete without one");

    for (field, omit) in [
        (ProvenanceField::Toolchain, omit_toolchain as fn(&mut _)),
        (ProvenanceField::Target, omit_target),
        (ProvenanceField::Family, omit_family),
        (ProvenanceField::Language, omit_language),
        (ProvenanceField::ToolComponentRole, omit_component_role),
        (
            ProvenanceField::ToolComponentVersion,
            omit_component_version,
        ),
    ] {
        let mut incomplete = metadata.clone();
        omit(&mut incomplete);
        assert_eq!(
            incomplete.identity(),
            Err(ArtifactBuildError::IncompletePayloadProvenance { field }),
            "omitting an owed field must be refused by that field's name",
        );
    }
}

fn omit_toolchain(metadata: &mut PayloadMetadata) {
    metadata.provenance.toolchain.clear();
}

fn omit_target(metadata: &mut PayloadMetadata) {
    metadata.provenance.target.clear();
}

fn omit_family(metadata: &mut PayloadMetadata) {
    metadata.provenance.family.clear();
}

fn omit_language(metadata: &mut PayloadMetadata) {
    metadata.provenance.language.clear();
}

fn omit_component_role(metadata: &mut PayloadMetadata) {
    metadata.provenance.components[0].role.clear();
}

fn omit_component_version(metadata: &mut PayloadMetadata) {
    metadata.provenance.components[0].version.clear();
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

/// A dimension the target honours and no packaged route consumes says so.
///
/// **The one producer assertion the neutral artifact cannot re-check, reached
/// from a real compilation for the first time.** A disposition is *derived* by
/// the artifact builder from the obligations that arrive — nothing translates
/// one — so `NotRequired` here means the compiler emitted no obligation for
/// contraction at all. That state became reachable when the delivered-realization
/// producer narrowed rows to occurrences whose operation can consume the
/// dimension: before, every honoured dimension had a row at every covered
/// occurrence and no compiled program could produce this record.
///
/// [`bare_reduction_program`] is that program. Contraction is still *asked of the
/// target* — the region proposal carries all four realization dimensions on
/// every candidate, and the profile honours them or the plan would not exist —
/// and it is still *stated* in the record, whose resolution is asserted below
/// beside the disposition. What is absent is any claim that a packaged route
/// relies on the target at a position, because a strict serial sum has no
/// product for a fused multiply-add to act on.
///
/// [`semantic_program`] is packaged beside it through the identical backend,
/// profile, and contract and comes back `Required`, so the difference is the
/// program's operations rather than the translation, the target, or the
/// dimension.
#[test]
fn an_unconsumed_honoured_dimension_is_packaged_as_not_required() {
    let semantic = bare_reduction_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let produced = sound(&semantic, plan);
    let decoded = decode_artifact(&produced.bytes).expect("the produced envelope decodes");

    let subject = ScalarArithmeticSubject::f32().identity();
    let delivered = decoded
        .delivered_realization()
        .scalar_arithmetic(&subject)
        .expect("the packaged f32 contract");

    // Every dimension, so a count cannot hide a dimension that moved arms.
    let mut required = Vec::new();
    let mut not_required = Vec::new();
    for dimension in CANONICAL_DIMENSIONS {
        match delivered.assessment(dimension) {
            DispositionView::Required(obligations) => {
                assert!(
                    !obligations.is_empty(),
                    "a `Required` disposition names a non-empty range",
                );
                required.push((dimension, obligations.len()));
            }
            DispositionView::NotRequired => not_required.push(dimension),
        }
    }
    // The direction that must never be reached: the fold reads operands, produces
    // results, and regroups its contributor sequence, so a `NotRequired` on any
    // of the consumed realized dimensions would be the artifact asserting that no
    // packaged route needs the target to honour a freedom this program genuinely
    // exercises.
    for dimension in [
        NumericalDimension::InputSubnormals,
        NumericalDimension::ResultSubnormals,
        NumericalDimension::Reassociation,
        NumericalDimension::Permutation,
        NumericalDimension::SignedZero,
        NumericalDimension::NanAssumptions,
        NumericalDimension::InfinityAssumptions,
    ] {
        assert!(
            !not_required.contains(&dimension),
            "the packaged fold consumes {dimension}, so `NotRequired` here is a \
             false producer assertion the neutral artifact cannot re-check",
        );
    }
    assert_eq!(
        required,
        [
            (NumericalDimension::InputSubnormals, 1),
            (NumericalDimension::ResultSubnormals, 1),
            (NumericalDimension::Reassociation, 1),
            (NumericalDimension::Permutation, 1),
            (NumericalDimension::SignedZero, 1),
            (NumericalDimension::NanAssumptions, 1),
            (NumericalDimension::InfinityAssumptions, 1),
        ],
        "one obligation each, at the one occurrence this program packages",
    );
    assert_eq!(
        not_required.len(),
        4,
        "eleven dimensions, seven of them required by the one covered fold",
    );
    assert!(
        not_required.contains(&NumericalDimension::Contraction),
        "the honoured contraction fact founds no position in this program, so \
         the derived disposition is the `NotRequired` assertion: {not_required:?}",
    );

    // The disposition is not a silence about the contract. The record still
    // states what contraction resolves to for this subject; what it withholds is
    // a target fact nothing packaged relies on.
    assert_eq!(
        delivered.resolution(NumericalDimension::Contraction),
        DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        "the strict contract's own resolution is carried whether or not a route \
         requires the dimension",
    );
    let DispositionView::Required(obligations) =
        delivered.assessment(NumericalDimension::Reassociation)
    else {
        panic!("the fold's regrouping is required by the packaged route");
    };
    assert_eq!(
        obligations[0].locus().locus(),
        PolicyLocus::Accumulator,
        "a fold's regrouping is a property of its accumulator",
    );

    // The same backend, profile, and contract over a program whose pointwise
    // multiply and add can fuse.
    let consuming = semantic_program();
    let consuming_compilation = scalar_host_compilation(&consuming);
    let consuming_plan = consuming_compilation.selected().expect("one selected plan");
    let consuming_produced = sound(&consuming, consuming_plan);
    let consuming_decoded =
        decode_artifact(&consuming_produced.bytes).expect("the produced envelope decodes");
    let DispositionView::Required(contraction) = consuming_decoded
        .delivered_realization()
        .scalar_arithmetic(&subject)
        .expect("the packaged f32 contract")
        .assessment(NumericalDimension::Contraction)
    else {
        panic!("the multiply and the add each found a contraction position");
    };
    assert_eq!(
        contraction.len(),
        2,
        "the two pointwise arithmetic occurrences, and not the fold",
    );
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

// -------------------------------------------------------------------------
// The promoted cache seam: one arrangement, and every operand a case moves
// -------------------------------------------------------------------------

/// Takes the sole compiled payload of a one-position delivery run.
///
/// Every case here declares one delivery position, so a run of any other length
/// is a defect in the case rather than something to handle.
fn sole(contents: Vec<PayloadContent>) -> PayloadContent {
    let [content] = <[PayloadContent; 1]>::try_from(contents)
        .expect("this backend declares exactly one delivery position");
    content
}

/// One complete arrangement of what `accept_or_publish_delivered_payload_artifact` takes.
///
/// A record rather than eight positional arguments per case, because every
/// perturbation below is one field of it and naming the field is what says which
/// statement was moved.
struct SeamRun<'run> {
    semantic: &'run SemanticProgram,
    plan: PlanAlternative<'run>,
    /// The descriptor-only artifact whose canonical identity the subject names.
    pending: &'run VerifiedArtifactProgram,
    /// What this backend declares its sole payload must say.
    declaration: &'run PayloadDeclaration,
    /// What the miss closure returns.
    compiled: PayloadContent,
    /// What `correspondence` compares a carried payload's metadata against.
    expected: &'run PayloadMetadata,
    /// The launch statements the miss assembly makes.
    perturbation: EntryPerturbation,
    /// The debug text the miss closure states, empty in every case but one.
    retained: DebugRetention,
    /// Counts the miss closure's invocations.
    compilations: &'run Cell<usize>,
}

/// Drives one arrangement through the promoted seam.
fn resolve(cache: &ExpansionCache, run: SeamRun<'_>) -> Result<AcceptedArtifact, SeamRefusal> {
    let SeamRun {
        semantic,
        plan,
        pending,
        declaration,
        compiled,
        expected,
        perturbation,
        retained,
        compilations,
    } = run;
    accept_or_publish_delivered_payload_artifact(
        cache,
        pending,
        std::slice::from_ref(&declaration.declared()),
        |_, actual| backend::correspondence(expected, actual),
        || {
            compilations.set(compilations.get() + 1);
            Ok::<CompiledPayloads, ScalarHostRefusal>(CompiledPayloads {
                contents: vec![compiled],
                retained,
            })
        },
        |contents| backend::assemble(semantic, plan, sole(contents), perturbation),
    )
}

/// One prepared payload and the pending artifact assembled from its declaration.
struct CacheFixture {
    prepared: PreparedScalarPayload,
    pending: VerifiedArtifactProgram,
}

impl CacheFixture {
    fn new(semantic: &SemanticProgram, plan: PlanAlternative<'_>) -> Self {
        let kernels: Vec<&VerifiedKernel> = plan.kernels().iter().collect();
        let prepared = backend::prepare(&kernels).expect("the kernels translate and describe");
        let pending = backend::assemble_pending(
            semantic,
            plan,
            &prepared.declaration,
            EntryPerturbation::default(),
        )
        .expect("the pending artifact assembles");
        Self { prepared, pending }
    }

    /// The content a sound compile step produces.
    fn compiled(&self) -> PayloadContent {
        PayloadContent {
            metadata: self.prepared.metadata.clone(),
            code: encode(&self.prepared.image),
        }
    }

    /// The sound arrangement every case below starts from.
    fn run<'run>(
        &'run self,
        semantic: &'run SemanticProgram,
        plan: PlanAlternative<'run>,
        compilations: &'run Cell<usize>,
    ) -> SeamRun<'run> {
        SeamRun {
            semantic,
            plan,
            pending: &self.pending,
            declaration: &self.prepared.declaration,
            compiled: self.compiled(),
            expected: &self.prepared.metadata,
            perturbation: EntryPerturbation::default(),
            retained: DebugRetention::none(),
            compilations,
        }
    }
}

fn outcome(resolution: &Resolution) -> &'static str {
    match resolution {
        Resolution::Published { .. } => "published",
        Resolution::Hit { .. } => "hit",
        Resolution::Uncached { .. } => "uncached",
    }
}

/// The payload publishes once, is accepted thereafter, and compiles exactly once.
///
/// The subject is composed by the seam from the payload's derived compilation
/// digest and the pending artifact's canonical identity, neither of which the
/// producer states. The counter is what makes "miss-only compilation" a
/// measurement rather than a claim: a second call that recompiled would still
/// publish the same bytes and every other assertion here would pass.
#[test]
fn a_custom_payload_publishes_then_is_accepted_from_the_cache() {
    let directory = scratch("publish");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let mut outcomes = Vec::new();
    for _ in 0..2 {
        let accepted = resolve(&cache, fixture.run(&semantic, plan, &compilations))
            .expect("the custom payload resolves through the promoted seam");
        outcomes.push(outcome(accepted.resolution()));
        assert_eq!(
            backend::validate_from_bytes(accepted.decoded()),
            Ok(()),
            "the backend's own from-bytes obligation still runs on the accepted result",
        );
        assert_eq!(
            accepted.decoded().identity().as_bytes(),
            fixture.pending.canonical_identity().as_bytes(),
        );
    }
    assert_eq!(outcomes, ["published", "hit"]);
    assert_eq!(
        compilations.get(),
        1,
        "the hit must not re-enter the compile step",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A backend's retained debug text reaches the entry and comes back from the hit.
///
/// The producer side of the cache's debug-retention section, exercised by a
/// backend that shares no code with the Metal path. Three facts are asserted
/// together because each one is what makes the others meaningful: the text is
/// readable from a *validated hit* rather than from the publication that wrote
/// it; the hit did not recompile, so the text came off disk; and the retention
/// did not move the key, so the entry a retaining build resolves to is the entry
/// a non-retaining build would have.
///
/// The canonical source is deliberately *not* asserted here, because it is not
/// what this section carries: this backend's compiled payload keeps its source
/// in the payload metadata inside the envelope, which is where any consumer reads
/// it from on any resolution.
#[test]
fn a_retained_diagnostic_survives_publication_and_returns_from_the_hit() {
    let directory = scratch("retention");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);
    let retained = DebugRetention::none()
        .retaining("scalar-host.notes", b"note: two entries lowered")
        .expect("a governed label");

    let published = resolve(
        &cache,
        SeamRun {
            retained: retained.clone(),
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect("the retaining producer resolves");
    assert_eq!(outcome(published.resolution()), "published");
    let published_key = match published.resolution() {
        Resolution::Published { entry, .. } | Resolution::Hit { entry, .. } => *entry.key(),
        Resolution::Uncached { .. } => panic!("the fixture cache stores entries"),
    };

    // A second run that states no retention at all still resolves to the same
    // entry, and reads back what the *publishing* build retained.
    let hit = resolve(&cache, fixture.run(&semantic, plan, &compilations))
        .expect("the stored entry resolves");
    assert_eq!(outcome(hit.resolution()), "hit");
    assert_eq!(
        compilations.get(),
        1,
        "a hit must not re-enter the compile step to acquire retained text",
    );
    let Resolution::Hit { entry, .. } = hit.resolution() else {
        panic!("the second run hits");
    };
    assert_eq!(
        *entry.key(),
        published_key,
        "retaining must not move the key, or a debug build would miss every entry",
    );
    assert_eq!(entry.retained_debug(), &retained);
    let run = entry
        .retained_debug()
        .run("scalar-host.notes")
        .expect("the labelled run survived the round trip");
    assert_eq!(run.as_bytes(), b"note: two entries lowered");
    assert!(!run.is_truncated());

    // And the compiled source is readable from the same hit *without* the
    // retention, because it travels in the envelope: it is part of the payload
    // metadata the payload digest is taken over. That is why no retention here
    // carries a copy of it — an unkeyed second copy could disagree with this one
    // and nothing could refuse the disagreement. One payload means one
    // descriptor, so index zero is the sole carried payload.
    assert_eq!(
        entry
            .artifact()
            .payload_metadata(0)
            .expect("the sole carried payload keeps its compilation metadata")
            .source,
        fixture.prepared.metadata.source,
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// An entry published without retention is a hit with nothing to show.
///
/// The absent case at the seam. A producer that turns retention on after entries
/// exist keeps hitting them: the compile step does not run, so nothing is
/// retained, and the entry is not rewritten under an unchanged key.
#[test]
fn a_hit_on_an_entry_published_without_retention_shows_nothing() {
    let directory = scratch("retention-absent");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    resolve(&cache, fixture.run(&semantic, plan, &compilations))
        .expect("the plain producer publishes");
    let hit = resolve(
        &cache,
        SeamRun {
            retained: DebugRetention::none()
                .retaining("scalar-host.notes", b"note: never reached")
                .expect("a governed label"),
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect("the stored entry resolves");

    assert_eq!(outcome(hit.resolution()), "hit");
    assert_eq!(compilations.get(), 1);
    let Resolution::Hit { entry, .. } = hit.resolution() else {
        panic!("the second run hits");
    };
    assert!(
        entry.retained_debug().is_empty(),
        "an entry published without retention has nothing to show, and is a hit all the same",
    );
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
    let sound_fixture = CacheFixture::new(&semantic, plan);

    let mut moved = sound_fixture.prepared.metadata.clone();
    moved.provenance.compile_flags = vec!["-scalar-host-opt".to_owned()];
    let moved_fixture = fixture_for(&semantic, plan, &sound_fixture, moved);

    let mut subjects = Vec::new();
    let mut identities = Vec::new();
    let compilations = Cell::new(0);
    for fixture in [&sound_fixture, &moved_fixture] {
        let accepted = resolve(&cache, fixture.run(&semantic, plan, &compilations))
            .expect("both subjects resolve");
        subjects.push(accepted.cache_subject().as_bytes().to_vec());
        identities.push(fixture.pending.canonical_identity().as_bytes().to_vec());
    }

    assert_ne!(
        sound_fixture.prepared.declaration.digest, moved_fixture.prepared.declaration.digest,
        "the payload digest must move",
    );
    assert_ne!(
        identities[0], identities[1],
        "the artifact identity must move with the payload digest",
    );
    assert_ne!(
        subjects[0], subjects[1],
        "the composed cache subject must move with both",
    );
    assert_eq!(
        compilations.get(),
        2,
        "two subjects are two misses, not one entry serving both",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// Rebuilds a fixture around a deliberately perturbed compilation subject.
fn fixture_for(
    semantic: &SemanticProgram,
    plan: PlanAlternative<'_>,
    sound: &CacheFixture,
    metadata: PayloadMetadata,
) -> CacheFixture {
    let mut declaration = sound.prepared.declaration.clone();
    declaration.digest = metadata
        .identity()
        .expect("the moved metadata derives its digest");
    declaration.compilation = declaration.digest.as_bytes().to_vec();
    let pending =
        backend::assemble_pending(semantic, plan, &declaration, EntryPerturbation::default())
            .expect("the moved pending artifact assembles");
    CacheFixture {
        prepared: PreparedScalarPayload {
            image: sound.prepared.image.clone(),
            metadata,
            declaration,
        },
        pending,
    }
}

// -------------------------------------------------------------------------
// Every refusal the promoted cache seam can produce, watched failing
// -------------------------------------------------------------------------

/// A declared payload the pending artifact does not carry is refused first.
///
/// Point one of three: before a subject exists, the sole pending payload must be
/// the one this backend says it prepared. The declaration is data, so this is
/// the seam's own comparison and its own refusal — a backend cannot lose the
/// check by supplying a permissive closure.
#[test]
fn a_declared_payload_the_pending_artifact_does_not_carry_is_refused() {
    let directory = scratch("declared-descriptor");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let mut declaration = fixture.prepared.declaration.clone();
    declaration.representation = RepresentationKey::new("tiler.test.scalar-host-image-v2")
        .expect("a governed representation key");
    let refusal = resolve(
        &cache,
        SeamRun {
            declaration: &declaration,
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("a pending payload of another representation cannot be keyed");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::PayloadDescriptor { delivery: 0 }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(
        compilations.get(),
        0,
        "the pending check must precede every compile step",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A declared digest other than the pending payload's is refused as a subject.
///
/// Separate from the case above because the two mean different things: a
/// different descriptor is a payload of another kind, and a different digest is
/// this kind of payload naming a compilation the pending artifact did not
/// describe. A check comparing only the governed keys would pass this one.
#[test]
fn a_declared_digest_other_than_the_pending_payloads_is_refused() {
    let directory = scratch("declared-digest");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let mut moved = fixture.prepared.metadata.clone();
    moved.provenance.link_flags = vec!["-other".to_owned()];
    let mut declaration = fixture.prepared.declaration.clone();
    declaration.digest = moved
        .identity()
        .expect("the moved metadata derives its digest");
    let refusal = resolve(
        &cache,
        SeamRun {
            declaration: &declaration,
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("a declaration naming another compilation cannot be keyed");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::PayloadSubject { delivery: 0 }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(compilations.get(), 0);
    let _ = std::fs::remove_dir_all(directory);
}

/// A compilation facet with no bytes is refused rather than under-keyed.
///
/// The facet is opaque to the cache, which is exactly why an empty one has to be
/// a typed refusal: nothing downstream could tell a key that named no
/// compilation from one that named a compilation producing no bytes.
#[test]
fn a_compilation_facet_with_no_bytes_is_refused() {
    let directory = scratch("empty-facet");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let mut declaration = fixture.prepared.declaration.clone();
    declaration.compilation = Vec::new();
    let refusal = resolve(
        &cache,
        SeamRun {
            declaration: &declaration,
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("a facet with no canonical bytes cannot compose a subject");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Subject(SubjectRefusal::EmptyRun {
                facet: SubjectFacet::BackendCompilations,
                index: 0,
            }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(compilations.get(), 0);
    let _ = std::fs::remove_dir_all(directory);
}

/// A failing compile step stays a compilation failure, not a protocol defect.
///
/// The distinction is the reason the seam carries three type parameters rather
/// than one: an environment failure may legitimately be retried and a
/// contradicted declaration may not, so collapsing them would make a defect that
/// must never become a rebuild indistinguishable from one that can.
#[test]
fn a_failing_compile_step_is_not_a_protocol_defect() {
    let directory = scratch("compile-failure");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);

    let refusal: SeamRefusal = accept_or_publish_delivered_payload_artifact(
        &cache,
        &fixture.pending,
        std::slice::from_ref(&fixture.prepared.declaration.declared()),
        |_, actual| backend::correspondence(&fixture.prepared.metadata, actual),
        || Err::<CompiledPayloads, _>(ScalarHostRefusal::UnsupportedAccessPattern { entry: 0 }),
        |contents| {
            backend::assemble(
                &semantic,
                plan,
                sole(contents),
                EntryPerturbation::default(),
            )
        },
    )
    .expect_err("a compile step that refuses cannot publish");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Compile(ScalarHostRefusal::UnsupportedAccessPattern { entry: 0 }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(
        format!("{refusal}"),
        "payload compilation failed: kernel 0 is not one read and one write",
        "the neutral seam must name the stage and forward the backend's own words",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// An assembly the neutral facade refuses stays an assembly refusal.
#[test]
fn an_assembly_the_facade_refuses_stays_an_assembly_refusal() {
    let directory = scratch("assembly-failure");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let refusal = resolve(
        &cache,
        SeamRun {
            perturbation: EntryPerturbation {
                bindings: Some(3),
                forbid_zero_work_skip: false,
            },
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("a surplus binding declaration cannot be published");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Assemble(ScalarHostRefusal::Assembly(_)),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(
        compilations.get(),
        1,
        "the assembly refusal must follow the compile step it consumes",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A produced artifact whose identity moved is refused *before* publication.
///
/// Point two of three, and the first check inside the miss closure. The
/// perturbation declares a zero-thread launch unskippable, which the plan admits
/// and which folds into artifact identity — so the produced artifact is
/// perfectly valid and simply is not the one the subject was composed from.
///
/// The second half is what makes this a distinct case rather than a slower
/// spelling of the post-resolution check: both refuse with the same kind, so the
/// only observable difference is whether the wrong artifact reached the store.
/// A sound run afterwards must therefore *publish* — a hit would mean the
/// refused run had already filed the perturbed envelope under this subject,
/// where every later process would read it.
#[test]
fn a_produced_artifact_whose_identity_moved_is_refused_before_publication() {
    let directory = scratch("produced-identity");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);
    let compilations = Cell::new(0);

    let refusal = resolve(
        &cache,
        SeamRun {
            perturbation: EntryPerturbation {
                bindings: None,
                forbid_zero_work_skip: true,
            },
            ..fixture.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("an artifact other than the pending one cannot be published under its subject");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::ArtifactIdentity),
        ),
        "unexpected refusal: {refusal:?}",
    );
    let accepted = resolve(&cache, fixture.run(&semantic, plan, &compilations))
        .expect("the sound arrangement resolves afterwards");
    assert_eq!(
        outcome(accepted.resolution()),
        "published",
        "the refused run must have left this subject unpublished",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A compilation other than the expected one is named fact by fact.
///
/// The refinement this seam delegates. Everything here is self-consistent — the
/// declaration, the pending artifact, and the produced artifact all describe the
/// perturbed compilation, so identity agrees and the descriptor comparison
/// passes — and the only authority that can notice is the backend, which says
/// *which* fact moved rather than that something did.
#[test]
fn a_compilation_other_than_the_one_expected_is_named_fact_by_fact() {
    let directory = scratch("correspondence");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let sound = CacheFixture::new(&semantic, plan);

    let mut moved = sound.prepared.metadata.clone();
    moved
        .source
        .extend_from_slice(b"a kernel this plan does not have");
    let perturbed = fixture_for(&semantic, plan, &sound, moved);
    let compilations = Cell::new(0);

    let refusal = resolve(
        &cache,
        SeamRun {
            // The whole run describes the perturbed compilation; only what the
            // backend was told to *expect* is the sound one.
            expected: &sound.prepared.metadata,
            ..perturbed.run(&semantic, plan, &compilations)
        },
    )
    .expect_err("a payload describing another compilation cannot be published");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::Correspondence {
                delivery: 0,
                cause: ScalarHostFact::Source,
            }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// An artifact carrying object bytes other than the compiled ones is refused.
///
/// Artifact identity deliberately excludes the emitted object, so this artifact
/// carries the same identity and the same descriptor as the sound one. Only the
/// seam's own comparison against the exact bytes the compile step returned
/// separates them, and without it a producer could publish one compilation's
/// subject over another compilation's object.
#[test]
fn an_artifact_carrying_other_object_bytes_is_refused_before_publication() {
    let directory = scratch("object-substitution");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);

    let refusal: SeamRefusal = accept_or_publish_delivered_payload_artifact(
        &cache,
        &fixture.pending,
        std::slice::from_ref(&fixture.prepared.declaration.declared()),
        |_, actual| backend::correspondence(&fixture.prepared.metadata, actual),
        || Ok::<CompiledPayloads, ScalarHostRefusal>(vec![fixture.compiled()].into()),
        |contents| {
            let mut substituted = sole(contents);
            substituted.code.push(0);
            backend::assemble(&semantic, plan, substituted, EntryPerturbation::default())
        },
    )
    .expect_err("an object other than the compiled one cannot be published");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::PayloadObject { delivery: 0 }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// An artifact that declares its payload without carrying it is refused.
///
/// A pending descriptor and a carried one are the same descriptor, so identity
/// agrees and nothing before this notices. What is missing is the content, and
/// publishing an envelope naming a backend object it does not contain would put
/// an unexecutable entry in the cache under a subject that promises one.
#[test]
fn an_artifact_carrying_no_payload_content_is_refused_before_publication() {
    let directory = scratch("uncarried-payload");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);

    let refusal: SeamRefusal = accept_or_publish_delivered_payload_artifact(
        &cache,
        &fixture.pending,
        std::slice::from_ref(&fixture.prepared.declaration.declared()),
        |_, actual| backend::correspondence(&fixture.prepared.metadata, actual),
        || Ok::<CompiledPayloads, ScalarHostRefusal>(vec![fixture.compiled()].into()),
        |_| {
            backend::assemble_pending(
                &semantic,
                plan,
                &fixture.prepared.declaration,
                EntryPerturbation::default(),
            )
        },
    )
    .expect_err("an artifact that carries no payload cannot be published");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::MissingPayloadMetadata {
                delivery: 0
            }),
        ),
        "unexpected refusal: {refusal:?}",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A cache entry holding another artifact under this subject is a hard refusal.
///
/// Point three of three. The perturbation is the *cache*, not the producer: an
/// entry for one artifact's subject is filled with a different artifact's
/// envelope, which is what a corrupted store or a subject that under-keyed its
/// facets would look like. The substituted artifact carries the same payload, so
/// every payload check passes and the identity comparison is the whole
/// protection. Accepting it would return bytes a host executes under the
/// identity it asked for, so it must fire rather than degrade into a rebuild.
#[test]
fn a_cache_entry_naming_another_artifact_is_refused_rather_than_accepted() {
    let directory = scratch("subject-disagreement");
    let cache = ExpansionCache::open(directory.join("cache"));
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);

    let other = backend::assemble(
        &semantic,
        plan,
        fixture.compiled(),
        EntryPerturbation {
            bindings: None,
            forbid_zero_work_skip: true,
        },
    )
    .expect("the other artifact assembles");
    backend::publish_under_foreign_subject(
        &cache,
        &fixture.pending,
        &fixture.prepared.declaration.compilation,
        &other,
    )
    .expect("the perturbed entry publishes");

    let compilations = Cell::new(0);
    let refusal = resolve(&cache, fixture.run(&semantic, plan, &compilations))
        .expect_err("a cache entry naming another artifact cannot be accepted");

    assert!(
        matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::ArtifactIdentity),
        ),
        "unexpected refusal: {refusal:?}",
    );
    assert_eq!(
        compilations.get(),
        0,
        "a contradicted hit is a hard refusal, never a rebuild",
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// A cache entry whose payload moved is refused by the same rules a miss is.
///
/// Point three runs the whole payload validation again rather than trusting what
/// publication proved, because the entry a hit returns need not be the entry
/// this process published. The two cases are the two halves of the split: a
/// fact the backend compares is named by the backend, and a fact it deliberately
/// leaves to the digest is caught one step later by the seam.
#[test]
fn a_cache_entry_whose_payload_moved_is_refused_after_resolution() {
    let directory = scratch("resolved-payload");
    let semantic = semantic_program();
    let compilation = scalar_host_compilation(&semantic);
    let plan = compilation.selected().expect("one selected plan");
    let fixture = CacheFixture::new(&semantic, plan);

    let mut named = fixture.prepared.metadata.clone();
    named.provenance.target.push_str("-other");
    let mut unnamed = fixture.prepared.metadata.clone();
    unnamed
        .obligations
        .push(tiler_artifact::program::PayloadTargetObligation {
            key: "tiler.test.scalar-host.obligation".to_owned(),
            value: "1".to_owned(),
        });

    for (index, (label, metadata)) in [
        ("a fact this backend compares", named),
        ("a fact it leaves to the digest", unnamed),
    ]
    .into_iter()
    .enumerate()
    {
        let cache = ExpansionCache::open(directory.join(format!("cache-{index}")));
        let moved = backend::assemble(
            &semantic,
            plan,
            PayloadContent {
                metadata,
                code: encode(&fixture.prepared.image),
            },
            EntryPerturbation::default(),
        )
        .expect("the moved artifact assembles");
        backend::publish_under_foreign_subject(
            &cache,
            &fixture.pending,
            &fixture.prepared.declaration.compilation,
            &moved,
        )
        .expect("the perturbed entry publishes");

        let compilations = Cell::new(0);
        let refusal = resolve(&cache, fixture.run(&semantic, plan, &compilations))
            .expect_err("a cache entry carrying another payload cannot be accepted");
        let expected_by_the_backend = matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::Correspondence {
                delivery: 0,
                cause: ScalarHostFact::Target,
            }),
        );
        let expected_by_the_seam = matches!(
            refusal,
            SeamRefusal::Protocol(DeliveredPayloadProtocolError::PayloadSubject { delivery: 0 }),
        );
        assert!(
            if index == 0 {
                expected_by_the_backend
            } else {
                expected_by_the_seam
            },
            "{label}: unexpected refusal: {refusal:?}",
        );
        assert_eq!(compilations.get(), 0, "{label}: a hit is never rebuilt");
    }
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
    let foreign = compile_governed(&semantic, NumericalContract::STRICT_F32)
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
            // Two payloads and one delivery position: the second is declared
            // and never realized, which is the unreferenced-payload refusal
            // rather than a second artifact family.
            let first = builder.push_carried_payload(
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
            )?;
            Ok(vec![first])
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
        std::slice::from_ref(&declaration),
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
        Access, AccessMode, AccessOrdinal, ApproximationEnvelope, BoundsProof, BoundsProofKind,
        BoundsWitnessId, ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan,
        LogicalAccess, NumericalPermission, NumericalRealization, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
        RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, SubnormalMode, TailPolicy,
        TensorRole,
    };

    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region
        .iteration_shape(Shape::from_dims([1]))
        .expect("the iteration shape binds");
    let accesses = [
        (TensorRole::Input, AccessMode::Read, 0, None),
        (TensorRole::Input, AccessMode::Read, 1, None),
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
        .input(AccessOrdinal::FIRST)
        .expect("the first pointwise input");
    let second = expression
        .input(AccessOrdinal::new(1))
        .expect("the second pointwise input");
    let root = expression.add(first, second).expect("the pointwise sum");
    let expression = expression.build(root).expect("the expression verifies");
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression),
            numerical: NumericalRealization::new(
                "tiler.test.scalar-host-wide",
                0x7fc0_0000,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        })
        .expect("the scalar program binds");
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
