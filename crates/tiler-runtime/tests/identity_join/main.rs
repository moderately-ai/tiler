//! A payload produced in one process, loaded and executed in another.
//!
//! # What this suite is evidence for
//!
//! That a build-time producer and a runtime adapter are joined by durable
//! artifact identity and by nothing else. The producer is a separate program
//! (`crates/tiler-build/examples/identity_join_producer.rs`) that compiles a
//! plan, translates it, publishes it through the promoted cache seam, and writes
//! an envelope and a durable identity record. This suite is the other half: it
//! links no compiler, no emitter, no AOT driver, and no build-time provider — it
//! *cannot*, and [`the_consumer_links_no_compiler_emitter_or_build_provider`]
//! proves that from the resolved dependency graph rather than claiming it — and
//! it routes those bytes to a result it checks against `tiler-reference`.
//!
//! # The six join subjects, and how each one is moved
//!
//! Backend family, executable representation, assessed target profile, payload
//! compatibility profile, compilation subject, and entry mapping. The producer
//! moves exactly one per variant and leaves the rest alone, so a refusal names
//! the subject that moved rather than showing that *something* was wrong. Two
//! further cases move nothing durable at all — the emitted object, whose bytes
//! artifact identity deliberately excludes — and move the caller's expectation
//! instead.
//!
//! Every refusal is asserted to arrive before the routing commit, through
//! `fallback_permitted` and through the adapter's own stage log: a claim made
//! only on the returned error would be a claim about the error.
//!
//! # What does not cross the boundary
//!
//! An envelope and a text record. No callback, no shared memory, no dynamically
//! loaded object, and no Rust value. Nothing the producer allocated, no `TypeId`
//! it minted, and no address it observed can reach this process, which is what
//! makes "no process-local identity entered a durable one" a measurement here
//! rather than an assertion: two producer processes write byte-identical
//! envelopes, and this third process re-derives their identity from the bytes.

mod adapter;
// The consumer's own decoder and interpreter, shared with `tests/adapter_route`
// rather than copied: `tiler.test.scalar-host-image-v1` is one representation,
// and two runtime suites reading it through two decoders would be testing the
// copies. The *producer* deliberately holds a third, independent encoder — see
// its module documentation — because a wire format crossing a process boundary
// is a byte contract rather than a shared Rust type.
#[path = "../adapter_route/image.rs"]
// This suite exercises the fused single-entry route, so the multi-entry helpers
// and the identity-contributor constants that module carries for
// `tests/adapter_route` are unused here.
#[allow(dead_code)]
mod image;
mod producer;
mod sidecar;

use adapter::{COMPLETE_ROUTE, ScalarHostAdapter, Stage};
use producer::{SOUND, Transported};
use sidecar::Sidecar;

use std::collections::BTreeMap;

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArithmeticType, BackendKey, RecordedArtifactProgramIdentity,
    RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{
    DTypeDispatch, DecodedProgram, ExecutionEnvironment, LoadRejection, TargetCompatibility,
    VariantIneligibility,
};

/// The one delivery position every artifact here is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and these artifacts are built for a single target, so the sole position
/// is zero. Named rather than written as a bare `0` at each call, because the
/// argument decides *which compiled object* is loaded and a literal there says
/// nothing about why that one.
const SOLE_DELIVERY: usize = 0;

/// Rows of the packaged input, which is also the output element count.
const ROWS: u64 = 2;
/// Columns of the packaged input, which is the reduction extent.
const COLUMNS: u64 = 3;
/// Bit pattern of the pointwise scale the graph applies, `2.0f32`.
const SCALE_BITS: u32 = 0x4000_0000;
/// Bit pattern of the pointwise bias the graph applies, `1.0f32`.
const BIAS_BITS: u32 = 0x3f80_0000;

/// The operand bits every routed case runs over.
///
/// Chosen so that agreement is a result rather than a coincidence: a negative
/// zero, the least positive subnormal, and a negative operand all pass through
/// the pointwise multiply, and the strict serial reduction forbids reassociating
/// what comes out.
const OPERANDS: [u32; 6] = [
    0x3f80_0000, // 1.0
    0x8000_0000, // -0.0
    0x3f00_0000, // 0.5
    0x0000_0001, // the least positive subnormal
    0xbfc0_0000, // -1.5
    0x4000_0000, // 2.0
];

/// What one perturbation of the route produced.
type Outcome =
    Result<adapter::Completion, AdapterRouteFailure<adapter::Refusal, image::ExecutionFault>>;

// -------------------------------------------------------------------------
// The consumer's own statement of what it expects to run
// -------------------------------------------------------------------------

/// Builds the semantic graph this consumer believes the artifact packages.
///
/// Written here rather than transported. The producer builds the same graph from
/// its own source, and neither half reads the other's construction, so the two
/// agreeing on the routed result is a statement about the artifact rather than
/// about a shared expression. Nothing in the join depends on this: it is the
/// oracle, and the artifact carries its own declared interface.
fn semantic_program() -> tiler_ir::semantic::SemanticProgram {
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    let mut draft = SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let input = draft
        .input::<F32>(input_key(), Shape::from_dims([ROWS, COLUMNS]))
        .expect("the input binds");
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("the scale constant");
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("the bias constant");
    let product = F32Multiply::apply(&mut draft, input, scale).expect("the pointwise product");
    let mapped = F32Add::apply(&mut draft, product, bias).expect("the pointwise sum");
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)])
        .expect("the strict reduction");
    draft
        .output(OutputKey::new("result").expect("a valid output key"), sum)
        .expect("the output binds");
    draft.build().expect("the program verifies")
}

/// Returns the interface key of the one named program input.
fn input_key() -> tiler_ir::semantic::InputKey {
    tiler_ir::semantic::InputKey::new("input").expect("a valid interface key")
}

/// Evaluates the packaged meaning over [`OPERANDS`] through the independent oracle.
fn reference_bits() -> Vec<u32> {
    let program = semantic_program();
    let key = input_key();
    let tensor = Tensor::dense(
        tiler_ir::semantic::F32::resolved_type(),
        tiler_ir::shape::Shape::from_dims([ROWS, COLUMNS]),
        OPERANDS
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    match outputs[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| {
                u32::from_be_bytes(
                    <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
                )
            })
            .collect(),
        payload => panic!("expected a dense f32 reference output, got {payload:?}"),
    }
}

// -------------------------------------------------------------------------
// Configuring the consumer from the durable record
// -------------------------------------------------------------------------

/// Returns the execution environment a host configured by `record` reports.
///
/// This is the join point. The identities come from the record the consumer was
/// built against, never from the envelope being routed — an adapter that read
/// them out of the artifact would agree with every artifact it was ever handed.
fn environment(record: &Sidecar) -> ExecutionEnvironment {
    ExecutionEnvironment {
        target_profile: profile(
            &record.target_profile_key,
            &record.target_profile_descriptor,
        ),
        backend: BackendKey::new(&record.backend).expect("a governed backend key"),
        representation: RepresentationKey::new(&record.representation)
            .expect("a governed representation key"),
        // Deliberately *not* read from the sidecar, unlike every field above.
        // The identities above join a consumer to the artifact it was built
        // against; which dtypes a family can dispatch is a fact about this
        // machine, and taking it from the record would make the host agree with
        // whatever it was handed — the exact failure this suite exists to
        // exclude. This host interprets `f32`, and says only that.
        dtype_dispatch: BTreeMap::from([(ArithmeticType::F32, DTypeDispatch::Dispatchable)]),
    }
}

/// Returns one declared target profile by key and exact descriptor.
fn profile(key: &str, descriptor: &[u8]) -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new(key).expect("a governed profile key"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(descriptor)
            .expect("a descriptor identity"),
    }
}

/// Returns the recorded identity a caller states when it asks for an artifact.
fn recorded(identity: &[u8]) -> RecordedArtifactProgramIdentity {
    RecordedArtifactProgramIdentity::from_bytes(identity)
        .expect("the producing side recorded a well-formed identity")
}

/// Binds the ABI facts a route evaluates its formulas against.
///
/// Read from the artifact's own declared interface rather than restated here: an
/// extent asserted at this call site would replace the artifact's declaration
/// with this suite's expectation, and the two halves would then agree because
/// they were told to.
fn bind_facts(program: &DecodedProgram) -> AbiFacts {
    let mut binder =
        AbiFactBinder::new(tiler_artifact::program::AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_input_shape(input.key(), input.shape())
            .expect("the transported interface binds");
    }
    binder.build()
}

/// Routes one transported envelope through a host configured by `pinned`.
///
/// `expected` is the identity the caller asks for, which is a separate statement
/// from the bytes it was handed — and separating them is what makes the
/// program-mismatch case expressible at all.
fn route(
    transported: &Transported,
    pinned: &Sidecar,
    expected: &[u8],
    adapt: impl FnOnce(ScalarHostAdapter) -> ScalarHostAdapter,
) -> (Outcome, ScalarHostAdapter) {
    let mut program = DecodedProgram::decode(&transported.bytes, SOLE_DELIVERY)
        .expect("the transported envelope decodes from its own bytes");
    let facts = bind_facts(&program);
    let mut host = adapt(ScalarHostAdapter::new(environment(pinned), &OPERANDS));
    let outcome = route_with_adapter(&mut program, &mut host, &recorded(expected), &facts);
    (outcome, host)
}

/// Returns the sole reason a one-variant artifact was filtered out entirely.
fn sole_exclusion(outcome: Outcome) -> VariantIneligibility {
    let Err(AdapterRouteFailure::Load(LoadRejection::NoEligibleVariant { packaged, filtered })) =
        outcome
    else {
        panic!("expected every packaged variant to be filtered");
    };
    assert_eq!(packaged, 1, "this producer packages one variant");
    let [only] = filtered.as_slice() else {
        panic!("one packaged variant filters to one reason, and {filtered:?} is another number");
    };
    assert_eq!(only.variant, 0);
    only.reason.clone()
}

// -------------------------------------------------------------------------
// The accepted cross-process route
// -------------------------------------------------------------------------

/// A payload produced in another process loads, executes, and matches the reference.
///
/// The whole outcome the ticket names, and every part of it is checked rather
/// than assumed: the envelope validates from its own bytes, the identity this
/// process re-derives from those bytes equals the one the producing process
/// recorded, every governed identity in the record is also declared by the
/// envelope, the adapter's stage log is the complete route, and the bits agree
/// with `tiler-reference`'s evaluation of the meaning — computed here, from a
/// graph this process built, by a crate the producer never ran.
#[test]
fn a_payload_produced_in_another_process_loads_executes_and_matches_the_reference() {
    let produced = producer::produce();
    let transported = produced.variant("run-a", SOUND);
    let record = &transported.sidecar;

    // Every load validates from bytes. Nothing about the producing process's
    // prior validation is carried, and nothing here trusts it.
    let program = DecodedProgram::decode(&transported.bytes, SOLE_DELIVERY)
        .expect("the transported envelope validates from its own bytes");
    assert_eq!(
        program.identity().as_bytes(),
        record.artifact_identity.as_slice(),
        "the identity re-derived in this process must be the one the producer recorded",
    );

    // The record is a configuration, never a second source of truth. Each field
    // it carries is compared against the envelope's own declaration, so a record
    // that disagreed would fail rather than be believed.
    let decoded = tiler_artifact::program::decode_artifact(&transported.bytes)
        .expect("the same bytes decode at the artifact layer");
    let [descriptor] = decoded.payloads() else {
        panic!("this producer declares exactly one payload");
    };
    assert_eq!(descriptor.backend.as_str(), record.backend);
    assert_eq!(descriptor.representation.as_str(), record.representation);
    assert_eq!(descriptor.digest.as_bytes(), record.payload_digest);
    assert_eq!(
        descriptor.compatibility.key.as_str(),
        record.payload_compatibility_key,
    );
    assert_eq!(
        descriptor.compatibility.descriptor.as_bytes(),
        record.payload_compatibility_descriptor,
    );
    let variant = decoded.variants().next().expect("one packaged variant");
    assert_eq!(
        variant.target_profile().key.as_str(),
        record.target_profile_key,
    );
    assert_eq!(
        variant.target_profile().descriptor.as_bytes(),
        record.target_profile_descriptor,
    );

    // The entry mapping, which is the subject that joins a packaged entry to a
    // symbol inside the transported object.
    let entry = variant.entries().next().expect("one packaged entry");
    let [mapping] = record.entries.as_slice() else {
        panic!("this producer maps exactly one entry");
    };
    assert_eq!(entry.backend_entry_key().as_bytes(), mapping.key);
    assert_eq!(
        entry.backend_symbol(SOLE_DELIVERY),
        Some(mapping.symbol.as_str())
    );

    let (outcome, host) = route(&transported, record, &record.artifact_identity, |host| host);
    let completion = outcome.expect("the transported artifact routes");
    assert_eq!(
        completion.result_bits,
        reference_bits(),
        "the routed result must agree with the independent oracle bit for bit",
    );
    assert_eq!(completion.executed, ROWS);
    assert_eq!(completion.profile_key, record.target_profile_key);
    assert_eq!(
        host.stages, COMPLETE_ROUTE,
        "the loader drives the adapter's stages in the order their facts become decidable",
    );
    produced.discard();
}

// -------------------------------------------------------------------------
// Stability, and the absence of process-local identity
// -------------------------------------------------------------------------

/// Two producer processes write one envelope, and the second run is a cache hit.
///
/// The measurement behind "no `TypeId`, vtable, function, allocation, or
/// registration address enters durable identity". The two runs are separate
/// operating-system processes with separate address-space layouts, separate heap
/// arrangements, and separate working directories and environments; anything
/// process-scoped that had reached an identity would move between them, and
/// nothing does — not one byte of the envelope, and not one field of the record
/// except the process identifier the producer writes deliberately so that this
/// case can prove the two runs were two processes.
///
/// The second run resolving as a **hit** is the other half: it read an entry the
/// first process published, and the promoted cache seam re-validated that entry
/// from its bytes rather than trusting the publication. A run that had rebuilt
/// instead would produce the same bytes and prove nothing about the hit path.
#[test]
fn two_producer_processes_write_one_envelope_and_the_second_run_hits_the_cache() {
    let produced = producer::produce();
    let first = produced.variant("run-a", SOUND);
    let second = produced.variant("run-b", SOUND);

    assert_ne!(
        first.sidecar.producer_pid, second.sidecar.producer_pid,
        "the two runs must be two processes, or this case measures nothing",
    );
    assert_eq!(first.sidecar.resolution, "published");
    assert_eq!(
        second.sidecar.resolution, "hit",
        "the second process must resolve the first one's published subject",
    );
    assert_eq!(
        first.bytes, second.bytes,
        "two processes producing one plan must write one envelope",
    );

    // Compared field by field rather than by comparing the two records whole:
    // the process identifier is expected to differ, and a whole-record
    // comparison would have to exclude it anyway — at which point naming the
    // subjects is what says which ones were checked.
    assert_eq!(
        first.sidecar.artifact_identity,
        second.sidecar.artifact_identity
    );
    assert_eq!(first.sidecar.cache_subject, second.sidecar.cache_subject);
    assert_eq!(first.sidecar.payload_digest, second.sidecar.payload_digest);
    assert_eq!(first.sidecar.backend, second.sidecar.backend);
    assert_eq!(first.sidecar.representation, second.sidecar.representation);
    assert_eq!(
        first.sidecar.target_profile_descriptor,
        second.sidecar.target_profile_descriptor,
    );
    assert_eq!(first.sidecar.entries, second.sidecar.entries);

    // And a third process — this one — re-derives the same identity from the
    // bytes alone. If any producer-local value had entered it, a reader that
    // never ran the producer could not agree.
    let program = DecodedProgram::decode(&first.bytes, SOLE_DELIVERY)
        .expect("the transported envelope validates");
    assert_eq!(
        program.identity().as_bytes(),
        first.sidecar.artifact_identity.as_slice(),
    );
    produced.discard();
}

/// A revision of the producer moves the compilation subject and nothing else.
///
/// The other half of stability: identical behaviour must be stable, and changed
/// behaviour must move the *right* subject. One output-affecting producer
/// statement changes here, and it reaches the payload digest, the artifact
/// identity, and the composed cache subject together — while the backend, the
/// representation, the assessed profile, and the entry mapping hold still.
///
/// The emitted-object case is the deliberate asymmetry beside it: object bytes
/// are outside artifact identity, so a moved object moves nothing durable at all
/// and only the backend's own from-bytes validation can see it.
#[test]
fn a_producer_revision_moves_the_compilation_subject_and_the_object_moves_nothing() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let revised = produced.variant("run-a", "moved-compilation-subject");
    let restated = produced.variant("run-a", "moved-emitted-object");

    assert_ne!(
        sound.sidecar.payload_digest, revised.sidecar.payload_digest,
        "a moved compilation subject must move the payload digest",
    );
    assert_ne!(
        sound.sidecar.artifact_identity, revised.sidecar.artifact_identity,
        "the artifact identity must move with the payload digest",
    );
    assert_ne!(
        sound.sidecar.cache_subject, revised.sidecar.cache_subject,
        "the composed cache subject must move with both",
    );
    assert_eq!(sound.sidecar.backend, revised.sidecar.backend);
    assert_eq!(sound.sidecar.representation, revised.sidecar.representation);
    assert_eq!(
        sound.sidecar.target_profile_descriptor,
        revised.sidecar.target_profile_descriptor,
    );
    assert_eq!(sound.sidecar.entries, revised.sidecar.entries);

    assert_ne!(
        sound.bytes, restated.bytes,
        "the moved object must reach the transported bytes",
    );
    assert_eq!(
        sound.sidecar.artifact_identity, restated.sidecar.artifact_identity,
        "artifact identity excludes the emitted object, so both are the same artifact",
    );
    assert_eq!(
        sound.sidecar.payload_digest,
        restated.sidecar.payload_digest
    );

    // The revised artifact is not broken — it is a different one. Routed under
    // its *own* identity it runs to the same answer, which is what makes the
    // refusal below a statement about the join rather than about the artifact.
    let (outcome, _) = route(
        &revised,
        &sound.sidecar,
        &revised.sidecar.artifact_identity,
        |host| host,
    );
    assert_eq!(
        outcome.expect("the revised artifact is valid").result_bits,
        reference_bits(),
    );
    produced.discard();
}

// -------------------------------------------------------------------------
// Every moved join subject, refused before the routing commit
// -------------------------------------------------------------------------

/// A backend family this host has no adapter for is filtered at binding.
#[test]
fn a_declared_backend_family_this_host_does_not_have_is_filtered() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "foreign-backend");
    assert_ne!(moved.sidecar.backend, sound.sidecar.backend);

    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &moved.sidecar.artifact_identity,
        |host| host,
    );
    let VariantIneligibility::UnsupportedRepresentation {
        declared_backend,
        host_backend,
        ..
    } = sole_exclusion(outcome)
    else {
        panic!("expected an unsupported representation");
    };
    assert_eq!(declared_backend.as_str(), moved.sidecar.backend);
    assert_eq!(host_backend.as_str(), sound.sidecar.backend);
    assert_eq!(
        host.stages,
        [Stage::Bind],
        "nothing is validated or prepared for an artifact this host cannot execute",
    );

    // The other direction of the same subject: the artifact is the sound one and
    // the *host* is the one with no adapter for its family. A consumer with no
    // matching adapter must fail closed exactly as a mismatched artifact does.
    let unmatched = BackendKey::new("tiler.test.no-such-backend").expect("a governed backend key");
    let (outcome, host) = route(
        &sound,
        &sound.sidecar,
        &sound.sidecar.artifact_identity,
        |adapter| adapter.for_backend(unmatched),
    );
    assert!(
        matches!(
            sole_exclusion(outcome),
            VariantIneligibility::UnsupportedRepresentation { .. },
        ),
        "a host with no adapter for the declared family must not route",
    );
    assert_eq!(host.stages, [Stage::Bind]);
    produced.discard();
}

/// An executable representation this host does not read is filtered at binding.
#[test]
fn a_declared_representation_this_host_does_not_read_is_filtered() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "foreign-representation");
    assert_ne!(moved.sidecar.representation, sound.sidecar.representation);

    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &moved.sidecar.artifact_identity,
        |host| host,
    );
    let VariantIneligibility::UnsupportedRepresentation {
        declared_representation,
        host_representation,
        ..
    } = sole_exclusion(outcome)
    else {
        panic!("expected an unsupported representation");
    };
    assert_eq!(
        declared_representation.as_str(),
        moved.sidecar.representation
    );
    assert_eq!(host_representation.as_str(), sound.sidecar.representation);
    assert_eq!(host.stages, [Stage::Bind]);
    produced.discard();
}

/// A plan assessed against another revision of this target is filtered on that profile.
///
/// The producer cannot forge this subject: `assemble_plan_artifact` derives the
/// variant's profile from the owner-linked compilation, so moving it required
/// compiling against another declared revision — which is exactly the state a
/// stale artifact is in. The key is unchanged and only the exact descriptor
/// moved, which is the distinction that separates "rebuild this" from "this is
/// the wrong artifact".
#[test]
fn a_plan_assessed_against_another_profile_revision_is_filtered_on_its_profile() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "foreign-target-profile");
    assert_eq!(
        moved.sidecar.target_profile_key, sound.sidecar.target_profile_key,
        "only the exact descriptor moves, so the key must be the same one",
    );
    assert_ne!(
        moved.sidecar.target_profile_descriptor,
        sound.sidecar.target_profile_descriptor,
    );

    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &moved.sidecar.artifact_identity,
        |host| host,
    );
    assert!(
        matches!(
            sole_exclusion(outcome),
            VariantIneligibility::AssessedProfile {
                classification: TargetCompatibility::DescriptorMismatch { .. },
            },
        ),
        "another revision of this profile is a rebuild, not a wrong artifact",
    );
    assert_eq!(host.stages, [Stage::Bind]);

    // The consumer-side half of the same subject, so the case does not depend on
    // which process moved it: the sound artifact, and a host that states another
    // revision of the profile it was assessed for.
    let (outcome, host) = route(
        &sound,
        &sound.sidecar,
        &sound.sidecar.artifact_identity,
        |adapter| {
            adapter.on_profile(profile(
                &moved.sidecar.target_profile_key,
                &moved.sidecar.target_profile_descriptor,
            ))
        },
    );
    assert!(matches!(
        sole_exclusion(outcome),
        VariantIneligibility::AssessedProfile { .. },
    ));
    assert_eq!(host.stages, [Stage::Bind]);
    produced.discard();
}

/// An object built for another profile is filtered on the payload's own declaration.
///
/// Separate from the case above and classified apart, because the two mean
/// different things: a plan *assessed* for another profile and an object *built*
/// for one send a reader to fix different things.
#[test]
fn a_payload_built_for_another_profile_is_filtered_on_the_payload_declaration() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "foreign-payload-compatibility");
    assert_eq!(
        moved.sidecar.target_profile_descriptor, sound.sidecar.target_profile_descriptor,
        "the assessed profile must hold still, or this case moves two subjects",
    );
    assert_ne!(
        moved.sidecar.payload_compatibility_descriptor,
        sound.sidecar.payload_compatibility_descriptor,
    );

    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &moved.sidecar.artifact_identity,
        |host| host,
    );
    assert!(
        matches!(
            sole_exclusion(outcome),
            VariantIneligibility::PayloadProfile {
                entry: 0,
                classification: TargetCompatibility::DescriptorMismatch { .. },
            },
        ),
        "the plan was assessed for this host and its emitted object was not",
    );
    assert_eq!(host.stages, [Stage::Bind]);
    produced.discard();
}

/// An artifact that is not the one the caller asked for is a program mismatch.
///
/// The compilation subject, as a *join* rather than as a property of one
/// artifact. Both artifacts here are valid and both are self-consistent; what
/// separates them is that this consumer was configured against one of them, and
/// the identity comparison is decided before anything is validated, prepared, or
/// allocated.
#[test]
fn an_artifact_other_than_the_recorded_one_is_a_program_mismatch() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let revised = produced.variant("run-a", "moved-compilation-subject");

    let (outcome, host) = route(
        &revised,
        &sound.sidecar,
        &sound.sidecar.artifact_identity,
        |host| host,
    );
    let failure = outcome.expect_err("another artifact must not route");
    assert!(
        matches!(
            failure,
            AdapterRouteFailure::Load(LoadRejection::ProgramMismatch { .. }),
        ),
        "expected a program mismatch: {failure}",
    );
    assert!(failure.fallback_permitted());
    assert_eq!(
        host.stages,
        [Stage::Bind],
        "identity is decided before the adapter is asked anything about the payload",
    );
    produced.discard();
}

/// An entry mapping reaching no packaged entry is refused by the loader's decode.
///
/// Both halves refused it. The producing process's own cache orchestration would
/// not publish it — the sidecar carries the refusal it produced — and this
/// process refuses the same bytes from its own decode, having trusted nothing
/// about the first refusal. That is what "validate every load from bytes" costs
/// and what it buys.
#[test]
fn an_entry_mapping_reaching_no_packaged_entry_is_refused_from_bytes() {
    let produced = producer::produce();
    let moved = produced.variant("run-a", "unmapped-entry-key");
    assert_eq!(moved.sidecar.resolution, "bypassed");

    let Err(rejection) = DecodedProgram::decode(&moved.bytes, SOLE_DELIVERY) else {
        panic!("an entry reaching no mapping must be refused");
    };
    assert!(
        format!("{rejection}").contains("UnmappedBackendEntry"),
        "the refusal must name the unmapped entry: {rejection}",
    );
    produced.discard();
}

/// A mapping naming a symbol the object does not define is the backend's refusal.
///
/// The artifact layer accepts these bytes — it carries an entry symbol and never
/// looks inside the object for it — so the only authority that can notice is the
/// backend, at exactly the point ADR 0090 item 8 places the obligation.
#[test]
fn a_mapping_naming_an_absent_symbol_is_the_backends_refusal() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "foreign-entry-symbol");

    DecodedProgram::decode(&moved.bytes, SOLE_DELIVERY)
        .expect("the artifact layer accepts an entry symbol it cannot check");

    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &moved.sidecar.artifact_identity,
        |host| host,
    );
    let Err(AdapterRouteFailure::Payload { entry, refusal }) = &outcome else {
        panic!("expected the backend's own payload refusal");
    };
    assert_eq!(*entry, 0);
    assert!(
        matches!(
            refusal,
            adapter::Refusal::Payload(image::ScalarPayloadRefusal::SymbolAbsent { .. }),
        ),
        "wrong classification: {refusal}",
    );
    assert!(
        outcome
            .as_ref()
            .err()
            .is_some_and(AdapterRouteFailure::fallback_permitted),
    );
    assert_eq!(
        host.stages,
        [Stage::Bind, Stage::ValidatePayload],
        "payload validation runs before anything is prepared or allocated",
    );
    produced.discard();
}

/// A moved object under an unmoved identity is caught only by the backend.
///
/// The artifact carrying it decodes, verifies, and re-derives the *same*
/// canonical identity as the sound one, so no comparison the loader makes could
/// separate them and the caller's recorded expectation is satisfied. Only the
/// backend looking at the object's own bytes refuses.
#[test]
fn a_moved_object_under_an_unmoved_identity_is_caught_only_by_the_backend() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);
    let moved = produced.variant("run-a", "moved-emitted-object");

    let program = DecodedProgram::decode(&moved.bytes, SOLE_DELIVERY)
        .expect("the artifact layer accepts the damaged object");
    assert_eq!(
        program.identity().as_bytes(),
        sound.sidecar.artifact_identity.as_slice(),
        "the two artifacts are the same one as far as every durable identity goes",
    );

    // Routed under the *sound* artifact's recorded identity, which it satisfies.
    let (outcome, host) = route(
        &moved,
        &sound.sidecar,
        &sound.sidecar.artifact_identity,
        |host| host,
    );
    let Err(AdapterRouteFailure::Payload { entry, refusal }) = &outcome else {
        panic!("expected the backend's own payload refusal");
    };
    assert_eq!(*entry, 0);
    assert!(
        matches!(
            refusal,
            adapter::Refusal::Payload(image::ScalarPayloadRefusal::TrailingBytes { extra: 3 }),
        ),
        "wrong classification: {refusal}",
    );
    assert!(
        outcome
            .as_ref()
            .err()
            .is_some_and(AdapterRouteFailure::fallback_permitted),
    );
    assert_eq!(host.stages, [Stage::Bind, Stage::ValidatePayload]);
    produced.discard();
}

/// Bytes the artifact layer refuses never reach the adapter at all.
#[test]
fn a_damaged_envelope_never_reaches_the_adapter() {
    let produced = producer::produce();
    let sound = produced.variant("run-a", SOUND);

    let mut damaged = sound.bytes.clone();
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0xff;
    assert!(
        DecodedProgram::decode(&damaged, SOLE_DELIVERY).is_err(),
        "a flipped interior byte must be refused by the artifact layer",
    );
    assert!(
        DecodedProgram::decode(&sound.bytes[..midpoint], SOLE_DELIVERY).is_err(),
        "a truncated envelope must be refused by the artifact layer",
    );
    produced.discard();
}

// -------------------------------------------------------------------------
// The process boundary, proven from the resolved dependency graph
// -------------------------------------------------------------------------

/// Packages this suite's binary must not be able to reach, and why.
///
/// The claim is that the consumer half constructs no compiler, emitter, AOT
/// driver, or build-time provider object. Asserting that by reading this
/// suite's `use` statements would only say what it happens to call today. What
/// is asserted instead is that the *types do not exist here*: none of these
/// packages is in `tiler-runtime`'s resolved dependency closure, dev-dependencies
/// included, so no target of this package links them and no source file in it
/// could name one.
const FORBIDDEN_PACKAGES: [&str; 5] = [
    "tiler-build",
    "tiler-compiler",
    "tiler-cache",
    "tiler-metal",
    "tiler-metal-aot",
];

/// Packages this suite's binary does reach, so the check has a population.
const REQUIRED_PACKAGES: [&str; 3] = ["tiler-artifact", "tiler-ir", "tiler-reference"];

/// The consumer's binary cannot contain a compiler, emitter, or build provider.
///
/// `Cargo.lock` is read rather than the manifests, because it is what Cargo
/// actually resolved and it merges normal, build, and development dependencies
/// into one edge list per package — which is the closure a test binary links.
/// The closure is walked transitively rather than checked one edge deep: an
/// indirect edge would put the same types in this binary as a direct one.
///
/// Three positive controls stop this from passing vacuously. The closure must
/// contain the packages this suite demonstrably uses; every forbidden name must
/// be a package the parse can actually find; and `tiler-build`'s own closure
/// must contain the compiler, so the forbidden names are ones an edge *could*
/// reach rather than strings nothing in the workspace resolves.
#[test]
fn the_consumer_links_no_compiler_emitter_or_build_provider() {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock"))
        .expect("the workspace lockfile is readable from this package");
    let packages = parse_lock_packages(&lock);

    for name in FORBIDDEN_PACKAGES.iter().chain(REQUIRED_PACKAGES.iter()) {
        assert!(
            packages.contains_key(name),
            "`{name}` is not in the parsed lockfile; the parse or the workspace is wrong, not the \
             dependency direction",
        );
    }
    let producer_closure = closure(&packages, "tiler-build");
    assert!(
        producer_closure.contains("tiler-compiler"),
        "`tiler-build` no longer reaches `tiler-compiler`, so this test is asserting the absence \
         of an edge nothing in the workspace holds",
    );

    let consumer_closure = closure(&packages, "tiler-runtime");
    for name in REQUIRED_PACKAGES {
        assert!(
            consumer_closure.contains(name),
            "`tiler-runtime` no longer reaches `{name}`, which this suite uses; the closure walk \
             is wrong",
        );
    }
    let offenders: Vec<&str> = FORBIDDEN_PACKAGES
        .into_iter()
        .filter(|name| consumer_closure.contains(name))
        .collect();
    assert!(
        offenders.is_empty(),
        "this suite's binary must not be able to name a build-time type, but `tiler-runtime` now \
         reaches: {offenders:?}",
    );
}

/// Returns every package reachable from `root`, including `root` itself.
fn closure<'lock>(
    packages: &std::collections::BTreeMap<&'lock str, Vec<&'lock str>>,
    root: &'lock str,
) -> std::collections::BTreeSet<&'lock str> {
    let mut reached = std::collections::BTreeSet::new();
    let mut pending = vec![root];
    while let Some(name) = pending.pop() {
        if !reached.insert(name) {
            continue;
        }
        if let Some(dependencies) = packages.get(name) {
            pending.extend(dependencies.iter().copied());
        }
    }
    reached
}

/// Extracts every `[[package]]` block's name and direct dependency names.
///
/// The lockfile grammar this relies on is narrow and stable: `[[package]]` opens
/// a block, `name = "…"` names it, and `dependencies = [` opens a list of one
/// quoted entry per line terminated by `]`. Anything else, including a
/// non-package table, closes the block being read. Hand-parsed for the reason
/// `crates/tiler/tests/dependency_direction.rs` gives: the grammar needed is
/// narrow, and a JSON dependency is a cost nothing here justifies.
fn parse_lock_packages(lock: &str) -> std::collections::BTreeMap<&str, Vec<&str>> {
    let mut packages = std::collections::BTreeMap::new();
    let mut current: Option<(&str, Vec<&str>)> = None;
    let mut in_dependencies = false;

    for line in lock.lines() {
        let trimmed = line.trim();
        if in_dependencies {
            if trimmed == "]" {
                in_dependencies = false;
            } else if let Some((_, dependencies)) = current.as_mut() {
                dependencies.push(dependency_name(trimmed));
            }
            continue;
        }
        if trimmed.starts_with('[') {
            if let Some((name, dependencies)) = current.take() {
                packages.insert(name, dependencies);
            }
            if trimmed == "[[package]]" {
                current = Some(("", Vec::new()));
            }
            continue;
        }
        let Some((name, _)) = current.as_mut() else {
            continue;
        };
        if let Some(value) = trimmed.strip_prefix("name = ") {
            *name = unquote(value);
        } else if trimmed == "dependencies = [" {
            in_dependencies = true;
        }
    }
    if let Some((name, dependencies)) = current {
        packages.insert(name, dependencies);
    }
    packages
}

/// Reads one dependency entry, dropping the version Cargo appends when a package
/// is resolved at more than one version.
fn dependency_name(entry: &str) -> &str {
    let unquoted = unquote(entry.trim_end_matches(','));
    unquoted.split_whitespace().next().unwrap_or(unquoted)
}

/// Strips the surrounding quotes from a lockfile string value.
fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"')
}
