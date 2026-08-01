//! One non-Metal adapter executing a carried payload through the ordinary loader path.
//!
//! # What this suite is evidence for
//!
//! That a consumer's own runtime adapter — selected by naming it, never resolved
//! from a registry — binds a validated artifact to a live execution context,
//! prepares it before the routing commit, and dispatches it with correct
//! resource lifetimes, and that every incompatible and post-commit outcome is
//! typed and explainable.
//!
//! It is an integration test on purpose. It compiles against `tiler-runtime`'s
//! public surface alone, so the adapter here is an out-of-crate implementor in
//! exactly the way a consumer's would be; a `#[cfg(test)]` module beside the
//! trait could reach `pub(crate)` items and would prove nothing about that.
//!
//! # The two halves of every perturbation
//!
//! An artifact-side perturbation varies the fixture's bytes and leaves the
//! adapter sound. An adapter-side perturbation varies what the adapter reports
//! and leaves the bytes sound. Keeping them apart is what makes each refusal
//! attributable: a case that changed both would show that *something* refused.
//!
//! Every case asserts the stage log as well as the outcome, because an assertion
//! made only on the returned error is an assertion about the error. The log says
//! which stages actually ran, which is how "the loader refuses a foreign route
//! requirement owner **without consulting an adapter**" and "no fallback follows
//! the commit" are checked rather than asserted.

mod adapter;
mod fixture;
mod image;

use adapter::{Perturbation, ScalarHostAdapter, Stage};
use fixture::{FixtureSpec, assemble};
use image::{ScalarEntry, ScalarImage, ScalarPayloadRefusal, encode};

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArtifactExecutionPolicy, AvailabilityPhase, BackendKey,
    RecordedArtifactProgramIdentity, RouteFeatureKey, RouteRequirementSubject,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{DecodedProgram, LoadRejection, TargetCompatibility, TargetDeclaration};

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
type Outcome = Result<
    adapter::ScalarCompletion,
    AdapterRouteFailure<adapter::ScalarRefusal, image::ExecutionFault>,
>;

/// Binds the ABI facts a route evaluates its formulas against.
///
/// Read from the artifact's own declared interface rather than restated here: an
/// extent asserted at this call site would replace the artifact's declaration
/// with this test's expectation, and the two halves would then agree because
/// they were told to.
fn bind_facts(program: &DecodedProgram) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    for input in program.inputs() {
        binder
            .bind_input_shape(input.key(), input.shape())
            .expect("the fixture's declared interface binds");
    }
    binder.build()
}

/// Runs one fixture through one adapter, end to end.
fn route(spec: &FixtureSpec, mut host: ScalarHostAdapter) -> (Outcome, Vec<Stage>) {
    let built = assemble(spec);
    let mut program = DecodedProgram::decode(&built.bytes).expect("the fixture artifact decodes");
    let facts = bind_facts(&program);
    let outcome = route_with_adapter(&mut program, &mut host, &built.expected, &facts);
    (outcome, host.stages.clone())
}

/// Evaluates the same semantic program through the independent oracle.
fn reference_bits() -> Vec<u32> {
    let program = fixture::semantic_program();
    let key = fixture::input_key();
    let tensor = Tensor::dense(
        tiler_ir::semantic::F32::resolved_type(),
        tiler_ir::shape::Shape::from_dims([fixture::ROWS, fixture::COLUMNS]),
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

/// The stages a route runs when nothing refuses.
const COMPLETE_ROUTE: [Stage; 7] = [
    Stage::Bind,
    Stage::ValidatePayload,
    Stage::ObserveLiveDevice,
    Stage::PrepareEntries,
    Stage::ObservePreparedEntry,
    Stage::PlanDispatch,
    Stage::Dispatch,
];

// -------------------------------------------------------------------------
// The accepted route
// -------------------------------------------------------------------------

/// A carried payload executes end to end and agrees with the reference bit for bit.
///
/// The comparison is against `tiler-reference`'s evaluation of the *semantic*
/// program the artifact packages a plan for, not against a formula restated in
/// this file. That is what makes the agreement evidence about the route rather
/// than about this test's own arithmetic.
#[test]
fn a_carried_payload_routes_through_a_selected_adapter_and_matches_the_reference() {
    let (outcome, stages) = route(&FixtureSpec::default(), ScalarHostAdapter::new(&OPERANDS));
    let completion = outcome.expect("the unperturbed route completes");
    assert_eq!(
        completion.result_bits,
        reference_bits(),
        "the adapter's read-back must agree with the independent oracle bit for bit",
    );
    assert_eq!(completion.executed, fixture::ROWS);
    assert_eq!(completion.profile_key, fixture::PROFILE_KEY);
    assert_eq!(
        stages, COMPLETE_ROUTE,
        "the loader drives the adapter's stages in the order their facts become decidable",
    );
}

/// The route always takes both device stages, even for a route requiring nothing.
///
/// ADR 0090 item 9: `preflight` alone is sufficient exactly when a variant defers
/// nothing and requires nothing, and once a caller enters `prepare` both stages
/// are mandatory regardless. An adapter path is past the point where the
/// device-free path applies, so an empty requirement list must still pass through
/// — otherwise a device check would only run when the artifact happened to need
/// one.
#[test]
fn a_route_requiring_nothing_still_passes_through_both_device_stages() {
    let spec = FixtureSpec {
        route_requirements: Vec::new(),
        deferred_predicates: Vec::new(),
        ..FixtureSpec::default()
    };
    let (outcome, stages) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    assert!(outcome.is_ok(), "a route requiring nothing still completes");
    assert_eq!(
        stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            // No row to report and no predicate to answer, so neither observing
            // stage is *called* — but both stages were entered, which is what
            // `prepare` makes unskippable and what reaching `PrepareEntries` and
            // `PlanDispatch` here proves.
            Stage::PrepareEntries,
            Stage::PlanDispatch,
            Stage::Dispatch,
        ],
    );
}

// -------------------------------------------------------------------------
// Identity and compatibility: every comparison is the loader's
// -------------------------------------------------------------------------

/// An artifact that is not the one the caller expected is a program mismatch.
#[test]
fn a_foreign_expected_identity_is_a_program_mismatch() {
    let built = assemble(&FixtureSpec::default());
    let mut program = DecodedProgram::decode(&built.bytes).expect("the fixture decodes");
    let facts = bind_facts(&program);
    let mut host = ScalarHostAdapter::new(&OPERANDS);
    // One bit of the real recording, so the value is a well-formed identity
    // under the governed domain and differs only in what it names. A random
    // string would be refused by the recorded-identity constructor and would
    // test that constructor rather than the binding.
    let mut bytes = built.expected.as_bytes().to_vec();
    *bytes.last_mut().expect("a recorded identity is not empty") ^= 0x01;
    let foreign = RecordedArtifactProgramIdentity::from_bytes(&bytes)
        .expect("a perturbed recording is still a recording");
    let outcome = route_with_adapter(&mut program, &mut host, &foreign, &facts);
    assert!(
        matches!(
            outcome,
            Err(AdapterRouteFailure::Load(
                LoadRejection::ProgramMismatch { .. }
            )),
        ),
        "expected a program mismatch, got {:?}",
        outcome.map(|_| ()),
    );
    assert_eq!(
        host.stages,
        [Stage::Bind],
        "identity is decided before the adapter is asked anything about the payload",
    );
}

/// A context reporting another target family refuses the variant's declaration.
#[test]
fn another_profile_key_is_an_incompatible_variant_target() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ForeignProfileKey),
    );
    let Err(AdapterRouteFailure::Load(LoadRejection::IncompatibleTarget {
        declaration,
        classification,
    })) = outcome
    else {
        panic!("expected an incompatible target");
    };
    assert_eq!(declaration, TargetDeclaration::Variant);
    assert!(matches!(
        classification,
        TargetCompatibility::ProfileKeyMismatch { .. }
    ));
    assert_eq!(stages, [Stage::Bind]);
}

/// The same family under another exact descriptor is a *separate* refusal.
///
/// ADR 0043's whole point, carried onto the adapter path: a key alone is not
/// evidence, and a caller that could not tell a rebuild from a wrong artifact
/// would go and fix the wrong thing.
#[test]
fn another_profile_descriptor_is_an_incompatible_variant_target() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ForeignProfileDescriptor),
    );
    let Err(AdapterRouteFailure::Load(LoadRejection::IncompatibleTarget {
        declaration,
        classification,
    })) = outcome
    else {
        panic!("expected an incompatible target");
    };
    assert_eq!(declaration, TargetDeclaration::Variant);
    assert!(matches!(
        classification,
        TargetCompatibility::DescriptorMismatch { .. }
    ));
    assert_eq!(stages, [Stage::Bind]);
}

/// The backend family and the representation are compared **as a pair**.
///
/// Each half alone is enough to refuse, and both refuse as the same class:
/// "this host cannot execute these bytes" is one finding with one remedy, and it
/// stays distinct from "this artifact is for another target" above.
///
/// The fixture declares no route requirement here, and that is not tidying. The
/// loader checks a backend-scoped requirement's *owner* before it routes any
/// entry, so a host misreporting its backend family while the variant carries a
/// row owned by the real one refuses as a foreign owner first — a correct
/// ordering, and one that would hide the pair comparison this case is about.
#[test]
fn either_half_of_the_backend_representation_pair_is_an_unexecutable_payload() {
    let spec = FixtureSpec {
        route_requirements: Vec::new(),
        ..FixtureSpec::default()
    };
    for perturbation in [
        Perturbation::ForeignBackend,
        Perturbation::ForeignRepresentation,
    ] {
        let (outcome, stages) = route(
            &spec,
            ScalarHostAdapter::new(&OPERANDS).perturbed(perturbation),
        );
        let Err(AdapterRouteFailure::Load(LoadRejection::UnexecutablePayload {
            declared_backend,
            declared_representation,
            host_backend,
            host_representation,
        })) = outcome
        else {
            panic!("{perturbation:?}: expected an unexecutable payload");
        };
        assert_eq!(declared_backend, fixture::BACKEND_KEY);
        assert_eq!(declared_representation, fixture::REPRESENTATION_KEY);
        assert!(
            host_backend != fixture::BACKEND_KEY
                || host_representation != fixture::REPRESENTATION_KEY,
            "{perturbation:?}: the refusal must name what this host reported",
        );
        assert_eq!(stages, [Stage::Bind], "{perturbation:?}");
    }
}

/// A payload built for another profile refuses separately from the variant's.
///
/// Two declarations, classified apart. Deriving either from the other is the
/// inference `BackendPayloadDescriptor::compatibility` exists to forbid, and a
/// caller that saw only one class could not tell a plan *assessed* for another
/// profile from an object *built* for one.
#[test]
fn a_payload_built_for_another_profile_refuses_as_the_payload_declaration() {
    let spec = FixtureSpec {
        payload_profile: fixture::profile_named(
            fixture::PROFILE_KEY,
            b"scalar-host-descriptor-other",
        ),
        ..FixtureSpec::default()
    };
    let (outcome, stages) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    let Err(AdapterRouteFailure::Load(LoadRejection::IncompatibleTarget {
        declaration,
        classification,
    })) = outcome
    else {
        panic!("expected an incompatible target");
    };
    assert_eq!(declaration, TargetDeclaration::Payload);
    assert!(matches!(
        classification,
        TargetCompatibility::DescriptorMismatch { .. }
    ));
    assert_eq!(stages, [Stage::Bind]);
}

/// A payload needing device translation is undeliverable through this loader.
#[test]
fn a_payload_requiring_device_translation_is_undeliverable() {
    let spec = FixtureSpec {
        execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
        ..FixtureSpec::default()
    };
    let (outcome, stages) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    assert!(
        matches!(
            outcome,
            Err(AdapterRouteFailure::Load(
                LoadRejection::UndeliverableExecutionPolicy { .. }
            )),
        ),
        "expected an undeliverable execution policy",
    );
    assert_eq!(stages, [Stage::Bind]);
}

/// A row owned by another backend is refused without consulting the adapter.
///
/// The stage log is the substance. The loader decides this from the host's own
/// declaration, and asking an adapter about another backend's namespace would
/// invite it to answer — so the evidence has to be that
/// `observe_live_device` never ran, not that the returned error had the right
/// shape.
#[test]
fn a_foreign_route_requirement_owner_is_refused_without_consulting_the_adapter() {
    let spec = FixtureSpec {
        route_requirements: vec![fixture::host_arithmetic_requirement(
            BackendKey::new("tiler.test.other-backend").expect("a governed backend key"),
        )],
        ..FixtureSpec::default()
    };
    let (outcome, stages) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    assert!(
        matches!(
            outcome,
            Err(AdapterRouteFailure::Load(
                LoadRejection::ForeignRouteRequirementOwner { .. }
            )),
        ),
        "expected a foreign route-requirement owner",
    );
    assert_eq!(
        stages,
        [Stage::Bind],
        "no adapter may be asked about a row owned by a backend this host is not",
    );
}

// -------------------------------------------------------------------------
// The adapter reports; the loader compares
// -------------------------------------------------------------------------

/// Three ways of failing to decide one live-device row stay three refusals.
///
/// Collapsing them would report an adapter gap as a device limitation, or route
/// on a requirement nothing evaluated. The adapter cannot reach any of these
/// three verdicts by answering cleverly: it returns an observation and the
/// loader draws the conclusion.
#[test]
fn each_undecidable_live_device_answer_refuses_by_its_own_class() {
    let expected_subject = RouteRequirementSubject::BackendFeature {
        owner: fixture::backend(),
        key: RouteFeatureKey::new(fixture::HOST_ARITHMETIC_FEATURE)
            .expect("a governed feature key"),
        version: fixture::HOST_ARITHMETIC_VERSION,
    };
    for (perturbation, name) in [
        (Perturbation::UnrecognizeLiveDevice, "unowned"),
        (Perturbation::MisanswerLiveDevice, "misanswered"),
        (Perturbation::RefuseLiveDeviceFeature, "unsatisfied"),
    ] {
        let (outcome, stages) = route(
            &FixtureSpec::default(),
            ScalarHostAdapter::new(&OPERANDS).perturbed(perturbation),
        );
        let Err(AdapterRouteFailure::Load(rejection)) = outcome else {
            panic!("{name}: expected a loader rejection");
        };
        let subject = match (&rejection, name) {
            (LoadRejection::UnownedRouteRequirement { subject, .. }, "unowned")
            | (LoadRejection::MisansweredRouteRequirement { subject, .. }, "misanswered")
            | (LoadRejection::UnsatisfiedRouteRequirement { subject, .. }, "unsatisfied") => {
                subject.clone()
            }
            _ => panic!("{name}: wrong rejection class: {rejection}"),
        };
        assert_eq!(
            subject, expected_subject,
            "{name}: the refusal must name the exact row the artifact declared",
        );
        assert_eq!(
            stages,
            [
                Stage::Bind,
                Stage::ValidatePayload,
                Stage::ObserveLiveDevice
            ],
            "{name}: nothing is prepared once a live-device row has refused",
        );
    }
}

/// The prepared-entry comparison refuses exactly at its own boundary.
///
/// One below the threshold refuses and the threshold itself routes. A test that
/// only checked a far-below value would pass against an off-by-one comparison,
/// which is the mistake this pair exists to make impossible.
#[test]
fn a_prepared_entry_property_refuses_exactly_at_its_boundary() {
    let (below, below_stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::UnderreportPreparedEntry),
    );
    assert!(
        matches!(
            below,
            Err(AdapterRouteFailure::Load(
                LoadRejection::UnsatisfiedDeferredPredicate {
                    predicate: 0,
                    entry: 0,
                    ..
                }
            )),
        ),
        "one below the threshold must refuse",
    );
    assert_eq!(
        below_stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            Stage::ObserveLiveDevice,
            Stage::PrepareEntries,
            Stage::ObservePreparedEntry,
        ],
        "nothing is allocated once a prepared-entry property has refused",
    );

    let (at, at_stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ReportPreparedEntryAtThreshold),
    );
    assert!(at.is_ok(), "the threshold itself must route");
    assert_eq!(at_stages, COMPLETE_ROUTE);
}

// -------------------------------------------------------------------------
// The backend's own payload obligation (ADR 0090 item 8)
// -------------------------------------------------------------------------

/// One payload perturbation: its name, the bytes it produces, and what must refuse.
struct PayloadCase {
    /// What was done to the object, for a failing assertion to name.
    name: &'static str,
    /// The exact carried object bytes.
    code: Vec<u8>,
    /// Whether the backend's refusal is the one this case is about.
    recognizes: fn(&ScalarPayloadRefusal) -> bool,
}

/// Returns the sound image with one field replaced.
fn image_with(mutate: impl FnOnce(&mut ScalarEntry)) -> Vec<u8> {
    let mut image = ScalarImage {
        entries: fixture::sound_image().entries,
    };
    mutate(&mut image.entries[0]);
    encode(&image)
}

/// Every payload defect is the backend's refusal, and the artifact layer accepts the bytes.
///
/// The second half is the load-bearing one and it is asserted directly: the
/// artifact carrying a damaged object **decodes**, **verifies**, and re-derives
/// the *same* canonical identity as the artifact carrying a sound one, because
/// artifact identity excludes the emitted object's bytes. There is therefore no
/// artifact-layer check that could have caught any of these, which is what makes
/// ADR 0090 item 8 a backend obligation rather than a division of labour someone
/// chose.
#[test]
fn every_payload_defect_is_the_backends_refusal_and_the_artifact_layer_accepts_the_bytes() {
    let sound = assemble(&FixtureSpec::default());
    let cases = vec![
        PayloadCase {
            name: "a foreign domain separator",
            code: {
                let mut bytes = encode(&fixture::sound_image());
                bytes[0] ^= 0xff;
                bytes
            },
            recognizes: |refusal| matches!(refusal, ScalarPayloadRefusal::ForeignDomain { .. }),
        },
        PayloadCase {
            name: "a schema this build does not read",
            code: {
                let mut bytes = encode(&fixture::sound_image());
                bytes[16..18].copy_from_slice(&2_u16.to_le_bytes());
                bytes
            },
            recognizes: |refusal| {
                matches!(
                    refusal,
                    ScalarPayloadRefusal::UnsupportedSchema { major: 2, .. }
                )
            },
        },
        PayloadCase {
            name: "a truncated object",
            code: {
                let mut bytes = encode(&fixture::sound_image());
                bytes.truncate(bytes.len() - 4);
                bytes
            },
            recognizes: |refusal| matches!(refusal, ScalarPayloadRefusal::Truncated { .. }),
        },
        PayloadCase {
            name: "trailing bytes after the last field",
            code: {
                let mut bytes = encode(&fixture::sound_image());
                bytes.extend_from_slice(&[0x00, 0x01, 0x02]);
                bytes
            },
            recognizes: |refusal| {
                matches!(refusal, ScalarPayloadRefusal::TrailingBytes { extra: 3 })
            },
        },
        PayloadCase {
            name: "a transport outside the entry's bindings",
            code: image_with(|entry| entry.read_transport = 7),
            recognizes: |refusal| {
                matches!(refusal, ScalarPayloadRefusal::TransportOutOfRange { .. })
            },
        },
        PayloadCase {
            name: "the read and write transports exchanged",
            code: image_with(|entry| {
                entry.read_transport = 0;
                entry.write_transport = 1;
            }),
            recognizes: |refusal| {
                matches!(refusal, ScalarPayloadRefusal::AccessModeMismatch { .. })
            },
        },
        PayloadCase {
            name: "an entry the artifact's symbol does not name",
            code: image_with(|entry| entry.symbol = "some_other_symbol".to_owned()),
            recognizes: |refusal| matches!(refusal, ScalarPayloadRefusal::SymbolAbsent { .. }),
        },
        PayloadCase {
            name: "extents wider than the route's published range",
            code: image_with(|entry| entry.columns = 5),
            recognizes: |refusal| {
                matches!(
                    refusal,
                    ScalarPayloadRefusal::UndersizedAccess { role: "read", .. }
                )
            },
        },
    ];

    for PayloadCase {
        name,
        code,
        recognizes,
    } in cases
    {
        let spec = FixtureSpec {
            code,
            ..FixtureSpec::default()
        };
        let damaged = assemble(&spec);
        assert_ne!(
            damaged.bytes, sound.bytes,
            "{name}: the perturbation must reach the transported bytes",
        );
        assert_eq!(
            damaged.expected.as_bytes(),
            sound.expected.as_bytes(),
            "{name}: artifact identity excludes the object, so both artifacts are the same one",
        );
        DecodedProgram::decode(&damaged.bytes).unwrap_or_else(|rejection| {
            panic!("{name}: the artifact layer accepts it: {rejection}")
        });

        let (outcome, stages) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
        let Err(AdapterRouteFailure::Payload { entry, refusal }) = &outcome else {
            panic!(
                "{name}: expected a backend payload refusal, got {:?}",
                outcome.map(|_| ())
            );
        };
        assert_eq!(*entry, 0, "{name}");
        let adapter::ScalarRefusal::Payload(refusal) = refusal else {
            panic!("{name}: expected the backend's own payload classification: {refusal}");
        };
        assert!(
            recognizes(refusal),
            "{name}: wrong classification: {refusal}"
        );
        assert!(
            outcome
                .as_ref()
                .err()
                .is_some_and(AdapterRouteFailure::fallback_permitted),
            "{name}: a payload refusal arrives while a fallback is still permitted",
        );
        assert_eq!(
            stages,
            [Stage::Bind, Stage::ValidatePayload],
            "{name}: payload validation runs before the first live-device question",
        );
    }
}

// -------------------------------------------------------------------------
// The adapter's own pre-commit refusals
// -------------------------------------------------------------------------

/// An adapter that binds no context refuses before the artifact is routed.
#[test]
fn an_unbound_execution_context_refuses_before_anything_is_routed() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::NoContext),
    );
    assert!(matches!(outcome, Err(AdapterRouteFailure::Context(_))));
    assert_eq!(stages, [Stage::Bind]);
}

/// A launch beyond what this interpreter admits refuses at preparation.
#[test]
fn a_launch_beyond_the_interpreters_budget_refuses_at_preparation() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::RefusePreparation),
    );
    assert!(
        matches!(
            outcome,
            Err(AdapterRouteFailure::Preparation(
                adapter::ScalarRefusal::LaunchBeyondBudget { entry: 0, .. }
            )),
        ),
        "expected a preparation refusal naming the entry",
    );
    assert_eq!(
        stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            Stage::ObserveLiveDevice,
            Stage::PrepareEntries,
        ],
    );
}

/// Caller storage shorter than the route's published range refuses before the commit.
#[test]
fn undersized_caller_storage_refuses_before_the_commit() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::UndersizedInput),
    );
    assert!(
        matches!(
            outcome,
            Err(AdapterRouteFailure::Plan(
                adapter::ScalarRefusal::UndersizedStorage { entry: 0, .. }
            )),
        ),
        "expected a planning refusal naming the entry and slot",
    );
    assert_eq!(
        stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            Stage::ObserveLiveDevice,
            Stage::PrepareEntries,
            Stage::ObservePreparedEntry,
            Stage::PlanDispatch,
        ],
        "the last refusal still arrives before the dispatch",
    );
}

// -------------------------------------------------------------------------
// After the commit
// -------------------------------------------------------------------------

/// A dispatch that does not reach terminal success is reported, not retried.
///
/// The route committed, so ADR 0051 forbids selecting another plan; the type
/// system already makes that unreachable inside the route, and
/// `fallback_permitted` is how a caller learns the same thing from the outside.
#[test]
fn a_post_commit_dispatch_failure_is_reported_and_forecloses_a_fallback() {
    let (outcome, stages) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::HaltAfterOneInvocation),
    );
    let Err(failure) = outcome else {
        panic!("a halted run must not report a completion");
    };
    assert!(
        matches!(
            failure,
            AdapterRouteFailure::Dispatch(image::ExecutionFault::Incomplete {
                executed: 1,
                expected: 2,
            }),
        ),
        "expected an observed terminal-success failure: {failure}",
    );
    assert!(
        !failure.fallback_permitted(),
        "nothing follows a committed dispatch that failed",
    );
    assert_eq!(
        stages, COMPLETE_ROUTE,
        "the failure is reached through the whole route rather than short of it",
    );
}

/// Every pre-commit refusal permits a fallback and the post-commit one does not.
///
/// Asserted over the whole population this suite produces rather than case by
/// case, so a stage that started reporting the wrong side of the commit is
/// visible here even if its own test still passes.
#[test]
fn the_commit_is_the_only_boundary_that_forecloses_a_fallback() {
    let pre_commit = [
        Perturbation::NoContext,
        Perturbation::ForeignProfileKey,
        Perturbation::ForeignBackend,
        Perturbation::UnrecognizeLiveDevice,
        Perturbation::UnderreportPreparedEntry,
        Perturbation::RefusePreparation,
        Perturbation::UndersizedInput,
    ];
    for perturbation in pre_commit {
        let (outcome, stages) = route(
            &FixtureSpec::default(),
            ScalarHostAdapter::new(&OPERANDS).perturbed(perturbation),
        );
        let failure = outcome
            .err()
            .unwrap_or_else(|| panic!("{perturbation:?} must refuse"));
        assert!(
            failure.fallback_permitted(),
            "{perturbation:?} is reached before the commit: {failure}",
        );
        assert!(
            !stages.contains(&Stage::Dispatch),
            "{perturbation:?} must not reach a dispatch",
        );
    }
}

/// Bytes the artifact layer refuses never reach the adapter at all.
#[test]
fn a_damaged_envelope_never_reaches_the_adapter() {
    let built = assemble(&FixtureSpec::default());
    let mut damaged = built.bytes.clone();
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0xff;
    assert!(
        DecodedProgram::decode(&damaged).is_err(),
        "a flipped interior byte must be refused by the artifact layer",
    );
    assert!(
        DecodedProgram::decode(&built.bytes[..midpoint]).is_err(),
        "a truncated envelope must be refused by the artifact layer",
    );
}

/// The interface a route binds is the artifact's own declaration.
#[test]
fn the_routed_interface_is_the_one_the_artifact_declares() {
    let built = assemble(&FixtureSpec::default());
    let program = DecodedProgram::decode(&built.bytes).expect("the fixture decodes");
    let inputs: Vec<_> = program.inputs().collect();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].key(), &fixture::input_key());
    let extents: Vec<u64> = inputs[0]
        .shape()
        .extents()
        .iter()
        .map(|extent| extent.get())
        .collect();
    assert_eq!(
        extents,
        [fixture::ROWS, fixture::COLUMNS],
        "the fixture's operands are sized by what the artifact declares",
    );
}
