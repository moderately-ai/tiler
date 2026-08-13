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

use adapter::{DispatchFamily, Perturbation, ScalarHostAdapter, Stage};
use fixture::{FixtureSpec, PackagedPlan, assemble, assemble_portfolio};
use image::{ScalarEntry, ScalarImage, ScalarPayloadRefusal, encode};

use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArithmeticType, AvailabilityPhase, BackendKey,
    RecordedArtifactProgramIdentity, RouteFeatureKey, RouteRequirementSubject, TargetProfileKey,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{
    DTypeDispatch, DTypeDispatchResolution, DecodedProgram, ExecutionEnvironment, LoadRejection,
    TargetCompatibility, VariantIneligibility,
};

/// The one delivery position every artifact here is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and these artifacts are built for a single target, so the sole position
/// is zero. Named rather than written as a bare `0` at each call, because the
/// argument decides *which compiled object* is loaded and a literal there says
/// nothing about why that one.
const SOLE_DELIVERY: usize = 0;

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
///
/// The adapter is returned rather than only its stage log. Its storage outlives
/// the route, and reading that storage afterwards is how a test observes both
/// the shared allocation's contents and the state a partially executed route
/// left behind — neither of which any returned value carries.
fn route(spec: &FixtureSpec, mut host: ScalarHostAdapter) -> (Outcome, ScalarHostAdapter) {
    let built = assemble(spec);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the fixture artifact decodes");
    let facts = bind_facts(&program);
    let outcome = route_with_adapter(&mut program, &mut host, &built.expected, &facts);
    (outcome, host)
}

/// Evaluates one semantic program over [`OPERANDS`] through the independent oracle.
fn evaluate(program: &tiler_ir::semantic::SemanticProgram) -> Vec<u32> {
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
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
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

/// Evaluates the packaged semantic program through the independent oracle.
fn reference_bits() -> Vec<u32> {
    evaluate(&fixture::semantic_program())
}

/// Evaluates the pointwise prefix alone, as the oracle for the intermediate.
///
/// The value the materialized member's *first* stage is obliged to write. It is
/// an evaluation of a semantic program rather than a restatement of the
/// interpreter's arithmetic, so an adapter and a fixture that agreed on a wrong
/// intermediate would still fail this.
fn pointwise_reference_bits() -> Vec<u32> {
    evaluate(&fixture::pointwise_semantic_program())
}

/// The stages a route runs when nothing refuses.
///
/// Sizing and allocating are two of them, on the two sides of the routing
/// commit. Their order in this list is the evidence that the seam allocates
/// after committing rather than before: a route that reversed them would produce
/// a log that is the same length and still wrong.
const COMPLETE_ROUTE: [Stage; 8] = [
    Stage::Bind,
    Stage::ValidatePayload,
    Stage::ObserveLiveDevice,
    Stage::PrepareEntries,
    Stage::ObservePreparedEntry,
    Stage::PlanDispatch,
    Stage::AllocateDispatch,
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
    let (outcome, host) = route(&FixtureSpec::default(), ScalarHostAdapter::new(&OPERANDS));
    let completion = outcome.expect("the unperturbed route completes");
    assert_eq!(
        completion.result_bits,
        reference_bits(),
        "the adapter's read-back must agree with the independent oracle bit for bit",
    );
    assert_eq!(completion.executed, fixture::ROWS);
    assert_eq!(completion.profile_key, fixture::PROFILE_KEY);
    assert_eq!(
        host.stages, COMPLETE_ROUTE,
        "the loader drives the adapter's stages in the order their facts become decidable",
    );
    // The empty case, asserted rather than left implicit. A single-entry route
    // has no data dependency between entries and therefore no pairing, and
    // *that* is what the materialized member below exists to stop being the only
    // state this suite ever observes.
    assert!(
        host.shared_placements().is_empty(),
        "a single-entry route pairs no allocations",
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
    let (outcome, host) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    assert!(outcome.is_ok(), "a route requiring nothing still completes");
    assert_eq!(
        host.stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            // No row to report and no predicate to answer, so neither observing
            // stage is *called* — but both stages were entered, which is what
            // `prepare` makes unskippable and what reaching `PrepareEntries` and
            // `PlanDispatch` here proves.
            Stage::PrepareEntries,
            Stage::PlanDispatch,
            Stage::AllocateDispatch,
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
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the fixture decodes");
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

/// Returns the sole reason a one-variant portfolio was filtered out entirely.
///
/// Every case below asserts the *reason* rather than only that nothing routed.
/// A single-variant artifact this host cannot execute produces the same class as
/// a whole foreign portfolio — that uniformity is the point of the filter — so
/// the class alone no longer says which host-relative subject excluded it, and
/// the reason is what a reader repairs against.
fn sole_exclusion(outcome: Outcome, packaged: usize) -> VariantIneligibility {
    let Err(AdapterRouteFailure::Load(LoadRejection::NoEligibleVariant {
        packaged: reported,
        filtered,
    })) = outcome
    else {
        panic!(
            "expected every packaged variant to be filtered, got {:?}",
            outcome.map(|_| ()),
        );
    };
    assert_eq!(
        reported, packaged,
        "the refusal names the artifact's own count"
    );
    assert_eq!(
        filtered.len(),
        packaged,
        "no eligible variant means every one of them was filtered",
    );
    let [only] = filtered.as_slice() else {
        panic!("this fixture packages one variant, and {filtered:?} names another number");
    };
    assert_eq!(only.variant, 0);
    only.reason.clone()
}

/// A context reporting another target family filters the variant's declaration.
#[test]
fn another_profile_key_filters_the_variant_on_its_assessed_profile() {
    let (outcome, host) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ForeignProfileKey),
    );
    assert!(
        matches!(
            sole_exclusion(outcome, 1),
            VariantIneligibility::AssessedProfile {
                classification: TargetCompatibility::ProfileKeyMismatch { .. },
            },
        ),
        "a foreign family excludes the plan on the profile it was assessed against",
    );
    assert_eq!(host.stages, [Stage::Bind]);
}

/// The same family under another exact descriptor is a *separate* exclusion.
///
/// ADR 0043's whole point, carried onto the adapter path: a key alone is not
/// evidence, and a caller that could not tell a rebuild from a wrong artifact
/// would go and fix the wrong thing.
#[test]
fn another_profile_descriptor_filters_the_variant_on_its_assessed_profile() {
    let (outcome, host) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ForeignProfileDescriptor),
    );
    assert!(
        matches!(
            sole_exclusion(outcome, 1),
            VariantIneligibility::AssessedProfile {
                classification: TargetCompatibility::DescriptorMismatch { .. },
            },
        ),
        "the same family under another descriptor is a rebuild, not a wrong artifact",
    );
    assert_eq!(host.stages, [Stage::Bind]);
}

/// The backend family and the representation are compared **as a pair**.
///
/// Each half alone is enough to exclude, and both exclude under the same
/// subject: "this host cannot execute these bytes" is one finding with one
/// remedy, and it stays distinct from "this artifact is for another target"
/// above.
///
/// The fixture declares no route requirement here, and that is not tidying. The
/// loader checks a backend-scoped requirement's *owner* before it routes any
/// entry, so a host misreporting its backend family while the variant carries a
/// row owned by the real one refuses as a foreign owner first — a correct
/// ordering, and one that would hide the pair comparison this case is about.
#[test]
fn either_half_of_the_backend_representation_pair_filters_the_variant() {
    let spec = FixtureSpec {
        route_requirements: Vec::new(),
        ..FixtureSpec::default()
    };
    for perturbation in [
        Perturbation::ForeignBackend,
        Perturbation::ForeignRepresentation,
    ] {
        let (outcome, host) = route(
            &spec,
            ScalarHostAdapter::new(&OPERANDS).perturbed(perturbation),
        );
        let VariantIneligibility::UnsupportedRepresentation {
            entry,
            declared_backend,
            declared_representation,
            host_backend,
            host_representation,
        } = sole_exclusion(outcome, 1)
        else {
            panic!("{perturbation:?}: expected an unsupported representation");
        };
        assert_eq!(entry, 0, "{perturbation:?}");
        assert_eq!(declared_backend.as_str(), fixture::BACKEND_KEY);
        assert_eq!(
            declared_representation.as_str(),
            fixture::REPRESENTATION_KEY
        );
        assert!(
            host_backend.as_str() != fixture::BACKEND_KEY
                || host_representation.as_str() != fixture::REPRESENTATION_KEY,
            "{perturbation:?}: the exclusion must name what this host reported",
        );
        assert_eq!(host.stages, [Stage::Bind], "{perturbation:?}");
    }
}

/// A payload built for another profile excludes separately from the variant's.
///
/// Two declarations, classified apart. Deriving either from the other is the
/// inference `BackendPayloadDescriptor::compatibility` exists to forbid, and a
/// caller that saw only one class could not tell a plan *assessed* for another
/// profile from an object *built* for one.
#[test]
fn a_payload_built_for_another_profile_filters_on_the_payload_declaration() {
    let spec = FixtureSpec {
        payload_profile: fixture::profile_named(
            fixture::PROFILE_KEY,
            b"scalar-host-descriptor-other",
        ),
        ..FixtureSpec::default()
    };
    let (outcome, host) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
    assert!(
        matches!(
            sole_exclusion(outcome, 1),
            VariantIneligibility::PayloadProfile {
                entry: 0,
                classification: TargetCompatibility::DescriptorMismatch { .. },
            },
        ),
        "the plan was assessed for this host and its emitted object was not",
    );
    assert_eq!(host.stages, [Stage::Bind]);
}

// `a_payload_requiring_device_translation_is_undeliverable` stood here until
// `route-or-refuse-the-device-translation-execution-policy` retired
// `ArtifactExecutionPolicy::RequiresDeviceTranslation`. The case it covered is
// unrepresentable now rather than refused, and the obligation moved down a
// layer: `the_retired_execution_policy_tag_is_refused_by_name` in
// `tiler-artifact`'s codec tests proves the withdrawn wire tag `0x02` is
// refused by name instead of resolving to the surviving policy.

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
    let (outcome, host) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
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
        host.stages,
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
        let (outcome, host) = route(
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
            host.stages,
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
    let (below, below_host) = route(
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
        below_host.stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            Stage::ObserveLiveDevice,
            Stage::PrepareEntries,
            Stage::ObservePreparedEntry,
        ],
        "nothing is allocated once a prepared-entry property has refused",
    );

    let (at, at_host) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ReportPreparedEntryAtThreshold),
    );
    assert!(at.is_ok(), "the threshold itself must route");
    assert_eq!(at_host.stages, COMPLETE_ROUTE);
}

/// One independent prepared-entry perturbation and the diagnostic it must name.
struct PreparedEntryCase {
    name: &'static str,
    spec: FixtureSpec,
    host: ScalarHostAdapter,
    recognizes: fn(&LoadRejection) -> bool,
    diagnostic: &'static str,
}

/// Unknown prepared-entry ownership cannot be confused with a quantity.
///
/// Each case perturbs one subject independently: provider namespace, provider
/// name, provider revision, property key, result variant, observed value, or
/// entry. The old path that answered every request with the entry's invocation
/// count would admit the key, provider, and variant cases whenever that count
/// equalled the required value — `ROWS == PREPARED_PROPERTY_MINIMUM` is that
/// coincidence, chosen so the refusal cannot be an accidental threshold miss.
#[test]
fn unknown_prepared_entry_ownership_cannot_be_confused_with_a_quantity() {
    let owned_quantity = fixture::ROWS;
    assert_eq!(
        owned_quantity,
        fixture::PREPARED_PROPERTY_MINIMUM,
        "the coincidence must be live: an adapter that ignored ownership would \
         return a quantity that satisfies the required value",
    );

    let foreign_key = FixtureSpec {
        deferred_predicates: vec![fixture::prepared_predicate_owned(
            0,
            fixture::FOREIGN_PREPARED_PROPERTY_KEY,
            fixture::PREPARED_PROPERTY_PROVIDER_NAMESPACE,
            fixture::PREPARED_PROPERTY_PROVIDER_NAME,
            fixture::PREPARED_PROPERTY_PROVIDER_REVISION,
        )],
        ..FixtureSpec::default()
    };
    let foreign_namespace = FixtureSpec {
        deferred_predicates: vec![fixture::prepared_predicate_owned(
            0,
            fixture::PREPARED_PROPERTY_KEY,
            "tiler-other",
            fixture::PREPARED_PROPERTY_PROVIDER_NAME,
            fixture::PREPARED_PROPERTY_PROVIDER_REVISION,
        )],
        ..FixtureSpec::default()
    };
    let foreign_name = FixtureSpec {
        deferred_predicates: vec![fixture::prepared_predicate_owned(
            0,
            fixture::PREPARED_PROPERTY_KEY,
            fixture::PREPARED_PROPERTY_PROVIDER_NAMESPACE,
            "other-prepared-entry",
            fixture::PREPARED_PROPERTY_PROVIDER_REVISION,
        )],
        ..FixtureSpec::default()
    };
    let foreign_revision = FixtureSpec {
        deferred_predicates: vec![fixture::prepared_predicate_owned(
            0,
            fixture::PREPARED_PROPERTY_KEY,
            fixture::PREPARED_PROPERTY_PROVIDER_NAMESPACE,
            fixture::PREPARED_PROPERTY_PROVIDER_NAME,
            2,
        )],
        ..FixtureSpec::default()
    };

    let unowned = |rejection: &LoadRejection| {
        matches!(
            rejection,
            LoadRejection::UnownedPreparedEntryProperty {
                predicate: 0,
                entry: 0,
                ..
            }
        )
    };
    let unsatisfied = |rejection: &LoadRejection| {
        matches!(
            rejection,
            LoadRejection::UnsatisfiedDeferredPredicate {
                predicate: 0,
                entry: 0,
                observed,
                ..
            } if *observed == fixture::PREPARED_PROPERTY_MINIMUM - 1
        )
    };
    let unowned_second = |rejection: &LoadRejection| {
        matches!(
            rejection,
            LoadRejection::UnownedPreparedEntryProperty {
                predicate: 1,
                entry: 1,
                ..
            }
        )
    };

    let cases = [
        PreparedEntryCase {
            name: "key",
            spec: foreign_key,
            host: ScalarHostAdapter::new(&OPERANDS),
            recognizes: unowned,
            diagnostic: "tiler.target.prepared-entry.thread-execution-width from tiler-test::scalar-host-prepared-entry@1",
        },
        PreparedEntryCase {
            name: "provider-namespace",
            spec: foreign_namespace,
            host: ScalarHostAdapter::new(&OPERANDS),
            recognizes: unowned,
            diagnostic: "tiler.target.prepared-entry.max-invocations from tiler-other::scalar-host-prepared-entry@1",
        },
        PreparedEntryCase {
            name: "provider-name",
            spec: foreign_name,
            host: ScalarHostAdapter::new(&OPERANDS),
            recognizes: unowned,
            diagnostic: "tiler.target.prepared-entry.max-invocations from tiler-test::other-prepared-entry@1",
        },
        PreparedEntryCase {
            name: "provider-revision",
            spec: foreign_revision,
            host: ScalarHostAdapter::new(&OPERANDS),
            recognizes: unowned,
            diagnostic: "tiler.target.prepared-entry.max-invocations from tiler-test::scalar-host-prepared-entry@2",
        },
        PreparedEntryCase {
            name: "result-variant",
            spec: FixtureSpec::default(),
            host: ScalarHostAdapter::new(&OPERANDS)
                .perturbed(Perturbation::UnrecognizePreparedEntry),
            recognizes: unowned,
            diagnostic: "runtime.unowned-prepared-entry-property",
        },
        PreparedEntryCase {
            name: "value",
            spec: FixtureSpec::default(),
            host: ScalarHostAdapter::new(&OPERANDS)
                .perturbed(Perturbation::UnderreportPreparedEntry),
            recognizes: unsatisfied,
            diagnostic: "runtime.unsatisfied-deferred-predicate",
        },
        PreparedEntryCase {
            name: "entry",
            spec: FixtureSpec::materialized(),
            host: ScalarHostAdapter::new(&OPERANDS)
                .perturbed(Perturbation::UnrecognizeSecondPreparedEntry),
            recognizes: unowned_second,
            diagnostic: "prepared entry 1",
        },
    ];

    for case in cases {
        let (outcome, host) = route(&case.spec, case.host);
        let Err(AdapterRouteFailure::Load(rejection)) = outcome else {
            panic!(
                "{}: expected a loader rejection, got {outcome:?}",
                case.name
            );
        };
        assert!(
            (case.recognizes)(&rejection),
            "{}: wrong rejection class: {rejection}",
            case.name,
        );
        assert!(
            host.stages.contains(&Stage::ObservePreparedEntry),
            "{}: the adapter must have been asked: {:?}",
            case.name,
            host.stages,
        );
        assert!(
            !host.stages.contains(&Stage::PlanDispatch),
            "{}: nothing is planned once a prepared-entry property has refused: {:?}",
            case.name,
            host.stages,
        );
        let rendered = rejection.to_string();
        assert!(
            rendered.contains(case.diagnostic),
            "{}: diagnostic {rendered:?} must contain {:?}",
            case.name,
            case.diagnostic,
        );
    }
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
        DecodedProgram::decode(&damaged.bytes, SOLE_DELIVERY).unwrap_or_else(|rejection| {
            panic!("{name}: the artifact layer accepts it: {rejection}")
        });

        let (outcome, host) = route(&spec, ScalarHostAdapter::new(&OPERANDS));
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
            host.stages,
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
    let (outcome, host) = route(
        &FixtureSpec::default(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::NoContext),
    );
    assert!(matches!(outcome, Err(AdapterRouteFailure::Context(_))));
    assert_eq!(host.stages, [Stage::Bind]);
}

/// A launch beyond what this interpreter admits refuses at preparation.
#[test]
fn a_launch_beyond_the_interpreters_budget_refuses_at_preparation() {
    let (outcome, host) = route(
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
        host.stages,
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
    let (outcome, host) = route(
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
        host.stages,
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
    let (outcome, host) = route(
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
                entry: 0,
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
        host.stages, COMPLETE_ROUTE,
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
        Perturbation::UnrecognizePreparedEntry,
        Perturbation::RefusePreparation,
        Perturbation::UndersizedInput,
    ];
    for perturbation in pre_commit {
        let (outcome, host) = route(
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
        // The allocating stage rather than the dispatch, because that is now the
        // first thing on the other side of the commit: a refusal that reached it
        // would have acquired program storage a fallback then discards, which is
        // exactly what ADR 0051 forbids.
        assert!(
            !host.stages.contains(&Stage::AllocateDispatch),
            "{perturbation:?} must not reach an allocation",
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
        DecodedProgram::decode(&damaged, SOLE_DELIVERY).is_err(),
        "a flipped interior byte must be refused by the artifact layer",
    );
    assert!(
        DecodedProgram::decode(&built.bytes[..midpoint], SOLE_DELIVERY).is_err(),
        "a truncated envelope must be refused by the artifact layer",
    );
}

/// A consumer asking to be a build target the artifact was not built for is refused.
///
/// The bytes are a valid artifact and the caller is asking for a delivery
/// position it does not carry. Refusing is the only fail-closed answer: taking
/// the sole payload instead would hand this consumer the object built for
/// somebody else's target, and `docs/research/apple-targets/artifact-compatibility.md`
/// records that such an object can load and dispatch without error.
///
/// It refuses at the decode, before any route exists, which is what makes the
/// position a property of the program rather than an argument three call sites
/// could disagree about.
#[test]
fn a_delivery_position_the_artifact_does_not_carry_is_refused() {
    let built = assemble(&FixtureSpec::default());
    let program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the fixture decodes");
    assert_eq!(program.delivery_positions(), 1);
    assert_eq!(program.delivery_position(), SOLE_DELIVERY);
    assert_eq!(
        DecodedProgram::decode(&built.bytes, 1)
            .expect_err("this artifact carries no second delivery position"),
        LoadRejection::UnknownDeliveryPosition {
            requested: 1,
            positions: 1,
        },
    );
}

/// The interface a route binds is the artifact's own declaration.
#[test]
fn the_routed_interface_is_the_one_the_artifact_declares() {
    let built = assemble(&FixtureSpec::default());
    let program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the fixture decodes");
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

// -------------------------------------------------------------------------
// The multi-entry route and its shared allocation
// -------------------------------------------------------------------------

/// The stages a two-entry route runs when nothing refuses.
///
/// Every per-entry stage appears twice and every per-route stage once, which is
/// the difference the empty case above cannot show: a seam that validated one
/// payload, prepared once, or asked about one prepared entry and then executed
/// two would produce a *shorter* log and the same result.
const MATERIALIZED_ROUTE: [Stage; 10] = [
    Stage::Bind,
    Stage::ValidatePayload,
    Stage::ValidatePayload,
    Stage::ObserveLiveDevice,
    Stage::PrepareEntries,
    Stage::ObservePreparedEntry,
    Stage::ObservePreparedEntry,
    Stage::PlanDispatch,
    Stage::AllocateDispatch,
    Stage::Dispatch,
];

/// Invocations the materialized route runs, summed over both entries.
const MATERIALIZED_INVOCATIONS: u64 = fixture::ROWS * fixture::COLUMNS + fixture::ROWS;

/// Bytes the shared scratch must reach, derived from the declared extents.
const SCRATCH_BYTES: u64 = fixture::ROWS * fixture::COLUMNS * 4;

/// Bytes [`Perturbation::UndersizeSharedAllocation`] leaves it holding.
const SHORT_SCRATCH_BYTES: u64 = SCRATCH_BYTES - 4;

/// Two entries over one shared scratch produce the same bits as one fused entry.
///
/// The three claims this suite could not previously make, in one route:
///
/// 1. **The stage log spans both entries.** Payload validation and prepared-entry
///    observation each run per entry; binding, live-device qualification,
///    preparation, planning, and dispatch each run once for the route.
/// 2. **One allocation backs both ends of the pairing.** The two slots resolve to
///    the same host allocation, and it is still readable after the last dispatch
///    that used it — holding exactly what the producing stage was obliged to
///    write, checked against `tiler-reference` rather than against the
///    interpreter.
/// 3. **The result is the fused member's, bit for bit.** Two plans over one
///    semantic program must agree with one oracle, and the consumer reading a
///    fresh buffer instead of the producer's would not.
#[test]
fn a_two_stage_route_shares_one_allocation_and_matches_the_reference() {
    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS),
    );
    let completion = outcome.expect("the unperturbed materialized route completes");

    assert_eq!(
        host.stages, MATERIALIZED_ROUTE,
        "the loader drives the per-entry stages once per entry and the rest once",
    );
    assert_eq!(
        completion.result_bits,
        reference_bits(),
        "a materialized plan must agree with the same oracle the fused plan does",
    );
    assert_eq!(
        completion.executed, MATERIALIZED_INVOCATIONS,
        "every invocation of both entries ran",
    );

    let [shared] = host.shared_placements() else {
        panic!(
            "the one data dependency between these stages pairs one allocation, got {:?}",
            host.shared_placements(),
        );
    };
    // The producing end writes and the consuming end reads, and the producer
    // precedes the consumer in execution order. A pairing oriented the other way
    // would still be one allocation and would still be wrong.
    assert_eq!(shared.producer, (0, 1), "entry 0's write slot produces");
    assert_eq!(shared.consumer, (1, 0), "entry 1's read slot consumes");
    assert_eq!(
        host.placement(0, 1).allocation,
        host.placement(1, 0).allocation,
        "both ends of the pairing must resolve to one host allocation",
    );
    assert_eq!(host.placement(0, 1).allocation, shared.allocation);

    // Read after the route returned, so the storage outlived its final device
    // use, and compared against the pointwise oracle, so it holds what the
    // producing stage owed rather than merely something nonzero.
    assert_eq!(
        host.allocation_bits(shared.allocation),
        pointwise_reference_bits(),
        "the shared scratch still holds the producing stage's output",
    );
}

/// A shared allocation short of what the plan sized it for is a terminal failure.
///
/// The one allocation this planner sizes from *two* statements rather than one,
/// which is why it is the one where its own arithmetic has to be checked against
/// what came back instead of trusted. Under ADR 0051 that check happens after the
/// commit, because the allocation does — so it is a `Failure` that forecloses a
/// fallback, and the priced consequence of moving allocation past the commit:
/// the pre-commit stage sized the pair correctly and only the acquisition was
/// short, which is an allocator defect and not a route to take elsewhere.
///
/// The stage log is the evidence that it is post-commit rather than the error
/// class alone: `AllocateDispatch` ran and `Dispatch` did not.
#[test]
fn a_shared_allocation_shorter_than_the_plan_sized_it_fails_after_the_commit() {
    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::UndersizeSharedAllocation),
    );
    let Err(failure) = outcome else {
        panic!("a shared allocation short of the plan's own length must fail");
    };
    assert!(
        matches!(
            failure,
            AdapterRouteFailure::Allocation(image::ExecutionFault::UndersizedStorage {
                entry: 0,
                slot: 1,
                required: SCRATCH_BYTES,
                held: SHORT_SCRATCH_BYTES,
            }),
        ),
        "expected an allocation failure naming the producing end of the pairing: {failure}",
    );
    assert!(
        !failure.fallback_permitted(),
        "the route committed before its storage was acquired, so nothing follows",
    );
    assert_eq!(
        host.stages,
        [
            Stage::Bind,
            Stage::ValidatePayload,
            Stage::ValidatePayload,
            Stage::ObserveLiveDevice,
            Stage::PrepareEntries,
            Stage::ObservePreparedEntry,
            Stage::ObservePreparedEntry,
            Stage::PlanDispatch,
            Stage::AllocateDispatch,
        ],
        "nothing is dispatched once an allocation failed",
    );
}

/// Dispatching the two entries back to front returns a wrong answer, not a refusal.
///
/// The failure mode `SharedAllocation`'s own documentation names as the one place
/// in this stack that would fail open, made concrete: nothing refuses, both
/// entries reach terminal success, every invocation runs, and the bits are
/// wrong — because the reduction read the scratch before the pointwise stage
/// wrote it.
///
/// The scratch is asserted *correct* at the end, which is the sharp part: the
/// storage was right, the pairing was right, and the order alone decided the
/// answer. That is why the accepted case above cannot treat its agreement with
/// the oracle as a property of the allocation.
#[test]
fn dispatching_the_two_entries_out_of_order_returns_a_wrong_answer_rather_than_a_refusal() {
    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::ReverseStageOrder),
    );
    let completion = outcome.expect("a reordered dispatch is not refused by anything");
    assert_eq!(
        host.stages, MATERIALIZED_ROUTE,
        "the route runs to completion; only the encoding order changed",
    );
    assert_eq!(
        completion.executed, MATERIALIZED_INVOCATIONS,
        "both entries reached terminal success",
    );
    assert_ne!(
        completion.result_bits,
        reference_bits(),
        "a consumer dispatched before its producer cannot produce the right answer",
    );

    let [shared] = host.shared_placements() else {
        panic!("the pairing is unchanged by the dispatch order");
    };
    assert_eq!(
        host.allocation_bits(shared.allocation),
        pointwise_reference_bits(),
        "the producing stage wrote the right scratch — after the stage that read it",
    );
}

/// A halt in the second entry is a post-commit failure naming that entry.
///
/// The partial-execution case a single-entry route cannot be in: one entry
/// reached terminal success and the next did not. The classification names the
/// entry, because "one of two invocations ran" is the same sentence for a route
/// that did nothing first and one that did everything first, and those leave the
/// caller's storage in different states.
#[test]
fn a_halt_in_the_second_entry_is_a_post_commit_failure_naming_that_entry() {
    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS)
            .perturbed(Perturbation::HaltSecondEntryAfterOneInvocation),
    );
    let Err(failure) = outcome else {
        panic!("a halted run must not report a completion");
    };
    assert!(
        matches!(
            failure,
            AdapterRouteFailure::Dispatch(image::ExecutionFault::Incomplete {
                entry: 1,
                executed: 1,
                expected: fixture::ROWS,
            }),
        ),
        "expected a terminal-success failure attributed to the second entry: {failure}",
    );
    assert!(
        !failure.fallback_permitted(),
        "nothing follows a committed dispatch that failed, however far it got",
    );
    assert_eq!(
        host.stages, MATERIALIZED_ROUTE,
        "the failure is reached through the whole route rather than short of it",
    );

    // The first entry completed. Its output is in the shared scratch, and the
    // second entry's partial writes are in the output storage: one row of two.
    let [shared] = host.shared_placements() else {
        panic!("the pairing is unchanged by a halted dispatch");
    };
    assert_eq!(
        host.allocation_bits(shared.allocation),
        pointwise_reference_bits(),
        "the entry before the halted one reached terminal success",
    );
    let written = host.allocation_bits(host.placement(1, 1).allocation);
    assert_eq!(
        written,
        vec![reference_bits()[0], 0],
        "the halted entry wrote the one row it ran and left the rest as allocated",
    );
}

// -------------------------------------------------------------------------
// Selecting across backend families
// -------------------------------------------------------------------------

/// Routes one portfolio device-free and returns the entries it selected.
///
/// Deliberately `preflight` rather than the adapter path. Selection is decided
/// before a device is bound and before any adapter is consulted, so asserting it
/// through a value that no adapter contributed to is what makes the result a
/// statement about the loader. The members used here defer nothing and require
/// nothing, so `preflight` reaches a `Preflight` rather than publishing rows.
fn select(
    members: &[FixtureSpec],
    host: &tiler_runtime::load::ExecutionEnvironment,
) -> Vec<String> {
    let built = assemble_portfolio(members);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let preflight = program
        .preflight(host, &built.expected, &facts)
        .unwrap_or_else(|rejection| panic!("the portfolio must select a route: {rejection}"));
    // The entry symbols, because they name the plan rather than the family: a
    // filter that selected the right backend and the wrong plan would agree with
    // an assertion made on the payload alone.
    preflight
        .entries()
        .iter()
        .map(|entry| entry.entry_symbol().to_owned())
        .collect()
}

/// Returns a portfolio member that defers nothing and requires nothing.
///
/// Both are answered after selection, and a member carrying either would refuse
/// on the device-free path before a selection assertion could be made. Stripping
/// them is what keeps these cases about which variant was chosen.
fn selectable(spec: FixtureSpec) -> FixtureSpec {
    FixtureSpec {
        route_requirements: Vec::new(),
        deferred_predicates: Vec::new(),
        ..spec
    }
}

/// An incompatible first variant does not hide a later one this host can run.
///
/// **The case the previous selection order got wrong.** It took the first
/// variant whose guard held and compared the host against that variant's payload
/// afterwards, so this portfolio refused on the Metal member — on a host that
/// could run the scalar member packaged directly behind it. Ineligibility is now
/// a filter, so the Metal member is never a candidate and rank 1 is selected.
#[test]
fn a_later_variant_is_selected_when_the_earlier_family_is_not_this_host() {
    let portfolio = [
        selectable(FixtureSpec::metal(PackagedPlan::Fused)),
        selectable(FixtureSpec::materialized()),
    ];
    assert_eq!(
        select(&portfolio, &fixture::scalar_host()),
        [fixture::POINTWISE_SYMBOL, fixture::REDUCTION_SYMBOL],
        "the scalar host must route to the scalar member behind the Metal one",
    );
}

/// The same portfolio selects the Metal member on a Metal host.
///
/// The other half of the previous case, and the one that shows the filter is not
/// simply "prefer the last variant": nothing about the artifact changed, only
/// what the host stated about itself, and the selection followed it.
#[test]
fn the_same_portfolio_selects_the_other_family_on_a_host_that_states_it() {
    let portfolio = [
        selectable(FixtureSpec::metal(PackagedPlan::Fused)),
        selectable(FixtureSpec::materialized()),
    ];
    assert_eq!(
        select(&portfolio, &fixture::metal_host()),
        [fixture::ENTRY_SYMBOL],
        "the Metal host must route to the Metal member ahead of the scalar one",
    );
}

/// Stable priority decides among the eligible, and only among the eligible.
///
/// Two members of one family, both executable here, in both declaration orders.
/// The selected plan follows the order the producer packaged rather than
/// anything about the plans themselves — which a test fixing one order could not
/// distinguish from a loader that happened to prefer fused or materialized
/// plans.
#[test]
fn stable_priority_selects_the_first_eligible_variant_in_declaration_order() {
    let fused = selectable(FixtureSpec::default());
    let materialized = selectable(FixtureSpec::materialized());
    assert_eq!(
        select(
            &[fused.clone(), materialized.clone()],
            &fixture::scalar_host()
        ),
        [fixture::ENTRY_SYMBOL],
        "rank 0 wins when both are eligible",
    );
    assert_eq!(
        select(&[materialized, fused], &fixture::scalar_host()),
        [fixture::POINTWISE_SYMBOL, fixture::REDUCTION_SYMBOL],
        "reversing the packaging reverses the choice, so order is meaning",
    );
}

/// Filtering removes candidates without reordering the ones that remain.
///
/// A foreign member ahead of two eligible ones. The answer must be the *first*
/// eligible member rather than the last, which is what separates a filter from a
/// search for something that fits.
#[test]
fn filtering_an_earlier_variant_does_not_reorder_the_eligible_ones() {
    let portfolio = [
        selectable(FixtureSpec::metal(PackagedPlan::FusedInapplicable)),
        selectable(FixtureSpec::default()),
        selectable(FixtureSpec::materialized()),
    ];
    assert_eq!(
        select(&portfolio, &fixture::scalar_host()),
        [fixture::ENTRY_SYMBOL],
        "the highest-ranked *eligible* member is selected, not the last one standing",
    );
}

/// An eligible variant the producer's guard excludes still falls through.
///
/// The guard-driven fallthrough the filter had to preserve rather than replace.
/// Rank 0 here is a plan this host can execute and the producer packaged under a
/// guard that never holds; rank 1 is the same host and an applicable guard.
#[test]
fn an_eligible_variant_whose_guard_is_false_falls_through_to_the_next() {
    let portfolio = [
        selectable(FixtureSpec::for_plan(PackagedPlan::FusedInapplicable)),
        selectable(FixtureSpec::materialized()),
    ];
    assert_eq!(
        select(&portfolio, &fixture::scalar_host()),
        [fixture::POINTWISE_SYMBOL, fixture::REDUCTION_SYMBOL],
        "a guard that does not hold is the producer's own answer, and the walk continues",
    );
}

/// A portfolio of one foreign family fails closed and names every exclusion.
#[test]
fn a_portfolio_this_host_cannot_execute_fails_closed() {
    let built = assemble_portfolio(&[
        selectable(FixtureSpec::metal(PackagedPlan::Fused)),
        selectable(FixtureSpec::metal(PackagedPlan::Materialized)),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let Err(LoadRejection::NoEligibleVariant { packaged, filtered }) =
        program.preflight(&fixture::scalar_host(), &built.expected, &facts)
    else {
        panic!("a portfolio of another backend family must not route on this host");
    };
    assert_eq!(packaged, 2);
    assert_eq!(
        filtered
            .iter()
            .map(|entry| entry.variant)
            .collect::<Vec<_>>(),
        [0, 1],
        "every packaged variant is named, in routing-rank order",
    );
    for excluded in &filtered {
        assert!(
            matches!(
                excluded.reason,
                VariantIneligibility::UnsupportedRepresentation {
                    ref declared_backend,
                    ..
                } if declared_backend.as_str() == fixture::METAL_BACKEND_KEY,
            ),
            "each exclusion names the family this host is not: {excluded}",
        );
    }
}

/// An eligible portfolio whose guards all say no is a *different* refusal.
///
/// The distinction the two selection classes exist for. This host can execute
/// what is packaged and the producer excluded it, which is the opposite repair
/// from the case above — and a loader that reported one class for both would
/// send a reader to find another build when the fix is to bind different facts.
#[test]
fn an_eligible_portfolio_with_no_applicable_guard_refuses_as_inapplicable() {
    let built = assemble_portfolio(&[
        selectable(FixtureSpec::metal(PackagedPlan::Fused)),
        selectable(FixtureSpec::for_plan(PackagedPlan::FusedInapplicable)),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let Err(LoadRejection::NoApplicableVariant { packaged, filtered }) =
        program.preflight(&fixture::scalar_host(), &built.expected, &facts)
    else {
        panic!("an eligible variant whose guard is false is not an ineligible portfolio");
    };
    assert_eq!(packaged, 2);
    let [excluded] = filtered.as_slice() else {
        panic!("exactly the Metal member is filtered here, got {filtered:?}");
    };
    assert_eq!(
        excluded.variant, 0,
        "the refusal still says which variant this host could not have run",
    );
}

/// An **eligible** variant's unanswerable guard aborts rather than falling through.
///
/// The fail-closed property the filter had to preserve. Rank 0 is a plan this
/// host can execute, packaged under a guard that reads the input extent, and the
/// caller binds nothing — so the guard is *unanswerable* rather than false. A
/// walk that skipped it would route to rank 1, silently substituting a plan the
/// producer ranked lower because the caller bound too little, and report a
/// successful route.
///
/// The subject is asserted, not just the class: the whole value of the rejection
/// is naming which formula went unanswered, and a refusal that pointed at rank 1
/// would send a reader to the wrong guard.
#[test]
fn an_eligible_variants_unanswerable_guard_refuses_instead_of_falling_through() {
    let built = assemble_portfolio(&[
        selectable(FixtureSpec::for_plan(PackagedPlan::FusedExtentGuarded)),
        selectable(FixtureSpec::materialized()),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    // Deliberately nothing bound. Every other case in this suite reads the
    // artifact's own declared interface; this one is about the caller that did
    // not.
    let unbound = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
    let outcome = program.preflight(&fixture::scalar_host(), &built.expected, &unbound);
    assert!(
        matches!(
            outcome,
            Err(LoadRejection::AbiEvaluation {
                subject: tiler_runtime::load::AbiSubject::ApplicabilityGuard { variant: 0 },
                ..
            }),
        ),
        "an unanswerable guard on an eligible variant must refuse, naming its own rank: {:?}",
        outcome.map(|_| ()),
    );
}

/// An **ineligible** variant's unanswerable guard is never evaluated at all.
///
/// The other side of the same coin, and the one the filter changed. The same
/// under-bound caller, the same unanswerable guard — but on a variant of a
/// backend family this host is not, so it is not a candidate and substitutes
/// nothing. Aborting here would let a portfolio's Metal member make a scalar host
/// unroutable through a formula that host was never going to evaluate.
#[test]
fn an_ineligible_variants_unanswerable_guard_does_not_abort_the_walk() {
    let built = assemble_portfolio(&[
        selectable(FixtureSpec::metal(PackagedPlan::FusedExtentGuarded)),
        selectable(FixtureSpec::materialized()),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let unbound = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
    let preflight = program
        .preflight(&fixture::scalar_host(), &built.expected, &unbound)
        .expect("a filtered variant's guard is not the caller's obligation to answer");
    assert_eq!(
        preflight
            .entries()
            .iter()
            .map(tiler_runtime::load::RoutedEntry::entry_symbol)
            .collect::<Vec<_>>(),
        [fixture::POINTWISE_SYMBOL, fixture::REDUCTION_SYMBOL],
    );
}

/// A selected cross-family route executes end to end through the adapter.
///
/// Selection is not a separate stage a consumer opts into: the same
/// `route_with_adapter` path that the single-variant cases take carries a
/// portfolio through, and the bits it returns agree with the same oracle. A
/// filter that chose correctly and then routed the *other* member's payload
/// would pass every assertion above and fail this one.
#[test]
fn a_portfolio_routes_its_selected_member_through_the_adapter_to_the_reference() {
    let built = assemble_portfolio(&[
        FixtureSpec::metal(PackagedPlan::Fused),
        FixtureSpec::materialized(),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let mut host = ScalarHostAdapter::new(&OPERANDS);
    let completion = route_with_adapter(&mut program, &mut host, &built.expected, &facts)
        .expect("the scalar member of the portfolio routes");
    assert_eq!(
        completion.result_bits,
        reference_bits(),
        "the selected member must agree with the independent oracle bit for bit",
    );
    assert_eq!(
        host.stages, MATERIALIZED_ROUTE,
        "the two-entry member was the one dispatched, and every per-entry stage ran twice",
    );
}

/// A filtered portfolio refuses before the adapter is asked anything, and a
/// fallback is still permitted.
///
/// Where selection sits relative to the routing commit, asserted through the
/// stage log rather than through the returned error. Eligibility is decided from
/// what the adapter reported when it bound a context and from nothing after it,
/// so a portfolio this host cannot execute must cost one `bind` and no payload
/// validation, no preparation, and no allocation — and ADR 0051 still permits
/// the caller to try another artifact.
#[test]
fn a_filtered_portfolio_refuses_at_binding_and_still_permits_a_fallback() {
    let built = assemble_portfolio(&[
        FixtureSpec::metal(PackagedPlan::Fused),
        FixtureSpec::metal(PackagedPlan::Materialized),
    ]);
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let mut host = ScalarHostAdapter::new(&OPERANDS);
    let outcome = route_with_adapter(&mut program, &mut host, &built.expected, &facts);
    let Err(failure) = outcome else {
        panic!("a portfolio of another backend family must not route here");
    };
    assert!(
        matches!(
            failure,
            AdapterRouteFailure::Load(LoadRejection::NoEligibleVariant { .. }),
        ),
        "expected the loader's own selection refusal: {failure}",
    );
    assert!(
        failure.fallback_permitted(),
        "selection is decided long before the commit: {failure}",
    );
    assert_eq!(
        host.stages,
        [Stage::Bind],
        "nothing is validated, prepared, or allocated for a portfolio this host cannot execute",
    );
}

/// The materialized member's outcomes fall on the side of the commit they belong on.
///
/// The same population claim the fused member makes, over the perturbations only
/// a multi-entry route can reach. Kept separate rather than merged because the
/// two members refuse in different places, and a single list would hide which
/// member a regression came from.
///
/// Both populations are written out, because the whole content of this ticket's
/// change is *which* list a perturbation belongs to: a shared allocation that
/// comes back short is no longer recoverable, and a caller's undersized operand
/// still is. Asserting only the recoverable half would leave the move invisible
/// here.
#[test]
fn the_multi_entry_outcomes_sit_on_the_side_of_the_commit_they_belong_on() {
    for perturbation in [
        Perturbation::RefusePreparation,
        Perturbation::UndersizedInput,
    ] {
        let (outcome, host) = route(
            &FixtureSpec::materialized(),
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
            !host.stages.contains(&Stage::AllocateDispatch),
            "{perturbation:?} must not reach an allocation",
        );
    }

    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS).perturbed(Perturbation::UndersizeSharedAllocation),
    );
    let failure = outcome.expect_err("an undersized shared allocation must fail");
    assert!(
        !failure.fallback_permitted(),
        "an allocation is acquired after the commit, so nothing follows it: {failure}",
    );
    assert!(
        host.stages.contains(&Stage::AllocateDispatch),
        "the failure is reached through the allocating stage",
    );
    assert!(
        !host.stages.contains(&Stage::Dispatch),
        "a route whose storage failed must not be dispatched",
    );
}

// -------------------------------------------------------------------------
// Per-dtype dispatchability, before the routing commit
// -------------------------------------------------------------------------
//
// The measured case these stand for: finding 26 of the Apple numerical-behaviour
// record has the iOS Simulator compiling and linking every `bfloat` module and
// *then* failing pipeline creation. That failure lands at
// `AvailabilityPhase::PreparedKernelPreflight`, one phase after ADR 0051's
// one-way routing commit, so a design that discovered it there would already
// have committed and could not fall back. Resolving the dtype from what the host
// states about its family moves the refusal ahead of selection.
//
// Every case here asserts the *phase* through the adapter's own stage log rather
// than only the returned class, because a refusal that moved one phase later
// still refuses and still looks green — which the ticket names as the property
// most likely to regress silently.
//
// **What these fixtures vary is the recorded arithmetic**, which is the whole of
// what a loader reads: see `FixtureSpec::arithmetic`. No case here claims BF16
// executes, and `docs/dtype-support.md` records BF16 backend execution as absent.

/// Returns the sole dtype exclusion a one-variant portfolio was filtered for.
///
/// Asserts the class and hands back the resolution, so a case states which of
/// the two refusing resolutions it expects instead of restating the destructure.
fn sole_dtype_exclusion(
    outcome: Outcome,
) -> (ArithmeticType, DTypeDispatchResolution, TargetProfileKey) {
    match sole_exclusion(outcome, 1) {
        VariantIneligibility::UndispatchableDType {
            entry,
            arithmetic,
            resolution,
            host_profile,
        } => {
            assert_eq!(entry, 0, "this fixture packages one entry");
            (arithmetic, resolution, host_profile)
        }
        other => panic!("expected a dtype exclusion, got {other}"),
    }
}

/// A family stating BF16 unsupported refuses a BF16 route before the commit.
#[test]
fn a_family_that_refuses_bf16_filters_a_bf16_variant_before_the_commit() {
    let (outcome, host) = route(
        &FixtureSpec::default().recording(ArithmeticType::Bf16),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::RefusesBf16),
    );
    let (arithmetic, resolution, profile) = sole_dtype_exclusion(outcome);
    assert_eq!(arithmetic, ArithmeticType::Bf16);
    assert_eq!(resolution, DTypeDispatchResolution::Unsupported);
    assert_eq!(
        profile.as_str(),
        fixture::PROFILE_KEY,
        "the refusal names the family that refused, not the one the artifact was built for",
    );
    assert_eq!(
        host.stages,
        [Stage::Bind],
        "the dtype is decided inside selection, so nothing is validated, prepared, or allocated",
    );
}

/// A family that states nothing about BF16 refuses it too, and separately.
///
/// The `Unknown` control. ADR 0043's disposal of an unknown predicate applied
/// rather than amended: nobody measured this family for this dtype, so the route
/// is refused rather than attempted. A design that admitted silence would reach
/// the simulator's pipeline-creation failure with no fallback left.
#[test]
fn a_family_that_never_measured_bf16_filters_a_bf16_variant_before_the_commit() {
    let (outcome, host) = route(
        &FixtureSpec::default().recording(ArithmeticType::Bf16),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::UnmeasuredForBf16),
    );
    let (arithmetic, resolution, _) = sole_dtype_exclusion(outcome);
    assert_eq!(arithmetic, ArithmeticType::Bf16);
    assert_eq!(resolution, DTypeDispatchResolution::Unknown);
    assert_eq!(host.stages, [Stage::Bind]);
}

/// The two refusals are told apart by what they carry, not only by refusing.
///
/// Both fail closed, and a caller acts differently on each: a measured negative
/// says no rebuild will help, and an unmeasured family says go and measure it.
/// A refusal that reported one class for both would send half of its readers to
/// the wrong repair, so the *rendered* forms are compared as well — a reader
/// reads the message, not the discriminant.
#[test]
fn a_refuted_bf16_family_and_an_unmeasured_one_are_distinguishable() {
    let spec = FixtureSpec::default().recording(ArithmeticType::Bf16);
    let refused = route(
        &spec,
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::RefusesBf16),
    );
    let unmeasured = route(
        &spec,
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::UnmeasuredForBf16),
    );
    let refused = sole_exclusion(refused.0, 1);
    let unmeasured = sole_exclusion(unmeasured.0, 1);
    assert_ne!(refused, unmeasured);
    assert_ne!(refused.to_string(), unmeasured.to_string());
    for reason in [&refused, &unmeasured] {
        let text = reason.to_string();
        assert!(
            text.contains("tiler::bf16@1"),
            "{text:?} must name the dtype under its governed key",
        );
        assert!(
            text.contains(fixture::PROFILE_KEY),
            "{text:?} must name the family that refused",
        );
    }
}

/// A family that dispatches BF16 routes the same artifact end to end.
///
/// What stops every case above from being a blanket refusal: the only thing that
/// changed is the host's own row, and the identical bytes now reach the commit
/// and every post-commit stage.
#[test]
fn a_family_that_dispatches_bf16_routes_the_same_artifact() {
    let (outcome, host) = route(
        &FixtureSpec::default().recording(ArithmeticType::Bf16),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::DispatchesBf16),
    );
    outcome.expect("a family declaring bf16 dispatchable routes a bf16-recorded artifact");
    assert_eq!(
        host.stages, COMPLETE_ROUTE,
        "a dispatchable dtype leaves every later stage exactly where it was",
    );
}

/// An `f32` route is unaffected on all three families.
///
/// The mechanism is dtype-neutral, so this asserts the neutrality rather than
/// assuming it: each family declares `f32` dispatchable and differs only about
/// BF16, and an implementation that refused on the *presence* of a declaration —
/// or on any dtype once one was refused — would fail here.
#[test]
fn an_f32_route_is_unaffected_by_every_bf16_verdict() {
    for family in [
        DispatchFamily::DispatchesBf16,
        DispatchFamily::RefusesBf16,
        DispatchFamily::UnmeasuredForBf16,
    ] {
        let (outcome, host) = route(
            &FixtureSpec::default(),
            ScalarHostAdapter::new(&OPERANDS).on_family(family),
        );
        let completion = outcome.unwrap_or_else(|failure| {
            panic!("an f32 route must be unaffected by {family:?}: {failure}")
        });
        assert_eq!(completion.result_bits, reference_bits(), "{family:?}");
        assert_eq!(host.stages, COMPLETE_ROUTE, "{family:?}");
    }
}

/// A portfolio ranking BF16 first falls through to the width this family runs.
///
/// The filter's own semantics reaching the dtype: an undispatchable variant is a
/// non-candidate rather than a refusal, so its guard is never evaluated and the
/// producer's next-ranked plan is selected. A terminal refusal here would make a
/// portfolio that packages a fallback width unroutable — the exact defect
/// `select-executable-variants-across-registered-backend-families` corrected for
/// backend families.
#[test]
fn a_portfolio_ranking_bf16_first_falls_through_to_a_dispatchable_width() {
    let portfolio = [
        selectable(FixtureSpec::for_plan(PackagedPlan::Fused)).recording(ArithmeticType::Bf16),
        selectable(FixtureSpec::materialized()),
    ];
    let mut host = fixture::scalar_host();
    host.dtype_dispatch
        .insert(ArithmeticType::Bf16, DTypeDispatch::Unsupported);
    assert_eq!(
        select(&portfolio, &host),
        [fixture::POINTWISE_SYMBOL, fixture::REDUCTION_SYMBOL],
        "a width this family cannot dispatch is a non-candidate, not the end of the walk",
    );
    // The same portfolio on a family that dispatches both selects rank 0, so the
    // fall-through above is the host's row and not the guard or the ranking.
    assert_eq!(
        select(&portfolio, &fixture::scalar_host()),
        [fixture::ENTRY_SYMBOL],
        "a family dispatching both widths takes the producer's first choice",
    );
}

/// Every packaged entry's dtype is resolved, not only the first one's.
///
/// The materialized member packages two entries, and this records the *second*
/// at BF16 while leaving the first at `f32`. A check resolving one dtype per
/// variant, or stopping at the first entry, would route this — so an exclusion
/// naming entry 1 is what makes "every entry" a measured property rather than a
/// restatement of the loop's shape.
///
/// **The within-variant ordinal permutation is not covered, and saying so is the
/// point.** The delivered-realization record keys its bindings in canonical
/// stage-key order while every position reported here is in execution order, and
/// for every variant this suite can build those two orders coincide — the
/// materialized member's included. So reading the wrong one is caught by nothing
/// here, and `CanonicalEntryOrdinals` is written against the ordinal space the
/// artifact layer documents rather than against a test. What this case does pin
/// is the *flat* ordinal within a variant, since the second entry's binding must
/// be found at all.
#[test]
fn a_later_entrys_dtype_is_resolved_as_well_as_the_first() {
    let (outcome, host) = route(
        &FixtureSpec::materialized().recording_each(&[ArithmeticType::F32, ArithmeticType::Bf16]),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::RefusesBf16),
    );
    let Err(AdapterRouteFailure::Load(LoadRejection::NoEligibleVariant { filtered, .. })) = outcome
    else {
        panic!("a variant whose second entry is bf16 must not route on a family that refuses it");
    };
    let [only] = filtered.as_slice() else {
        panic!("this fixture packages one variant, and {filtered:?} names another number");
    };
    assert!(
        matches!(
            only.reason,
            VariantIneligibility::UndispatchableDType {
                entry: 1,
                arithmetic: ArithmeticType::Bf16,
                resolution: DTypeDispatchResolution::Unsupported,
                ..
            },
        ),
        "expected the second entry's dtype exclusion, got {}",
        only.reason,
    );
    assert_eq!(host.stages, [Stage::Bind]);
    // The same two-entry member with both entries at `f32` routes whole on the
    // same family, so the refusal above is the recorded width and not the entry
    // count, the plan, or the shared allocation the materialized member carries.
    let (outcome, host) = route(
        &FixtureSpec::materialized(),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::RefusesBf16),
    );
    outcome.expect("an f32 two-entry route is unaffected by a bf16 refusal");
    assert!(host.stages.contains(&Stage::Dispatch));
}

/// A dtype refusal still permits the fallback ADR 0051 allows.
///
/// The property the phase ordering exists for, asserted through the failure's own
/// classification rather than inferred from the stage log: reaching the commit is
/// what forecloses a fallback, and this refusal is reached before selection ends.
#[test]
fn a_dtype_refusal_arrives_while_a_fallback_is_still_permitted() {
    let (outcome, _) = route(
        &FixtureSpec::default().recording(ArithmeticType::Bf16),
        ScalarHostAdapter::new(&OPERANDS).on_family(DispatchFamily::RefusesBf16),
    );
    let failure = outcome.expect_err("a refused dtype must not route");
    assert!(
        failure.fallback_permitted(),
        "a dtype is decided inside selection, long before the commit: {failure}",
    );
}

/// Dense F32 `[2, N]`: semantic `(row = 1, column = 0)` is element `N`, so bytes `4N`.
const fn dense_f32_row_major_bytes(row: u64, column: u64, inner_extent: u64) -> u64 {
    4 * (row * inner_extent + column)
}

fn live_extent_facts(n: u64) -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_extent(fixture::input_key(), tiler_ir::shape::Axis::new(1), n)
        .expect("the live axis binds");
    binder.build()
}

fn live_extent_environment() -> ExecutionEnvironment {
    let mut dtype_dispatch = std::collections::BTreeMap::new();
    dtype_dispatch.insert(ArithmeticType::F32, DTypeDispatch::Dispatchable);
    ExecutionEnvironment {
        target_profile: fixture::profile(),
        backend: fixture::backend(),
        representation: fixture::representation(),
        dtype_dispatch,
    }
}

#[test]
fn one_live_extent_payload_and_pipeline_indexes_dense_f32_at_two_n() {
    let spec = FixtureSpec::live_extent();
    let built = assemble(&spec);
    assert_eq!(
        built.expected.as_bytes(),
        assemble(&FixtureSpec::live_extent()).expected.as_bytes(),
        "artifact identity excludes the bound N",
    );
    assert_ne!(
        built.expected.as_bytes(),
        assemble(&FixtureSpec::default()).expected.as_bytes(),
        "a baked static neighbour is a different artifact subject",
    );

    let mut addresses = Vec::new();
    let mut pipeline_subjects = Vec::new();
    for n in [14_u64, 15] {
        let mut program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY)
            .expect("the live-extent artifact decodes");
        let preflight = program
            .preflight(
                &live_extent_environment(),
                &built.expected,
                &live_extent_facts(n),
            )
            .expect("both neighbouring extents preflight the same artifact");
        let entry = &preflight.entries()[0];
        assert_eq!(entry.entry_symbol(), "live_row_major");
        assert_eq!(entry.extent_parameters().len(), 1);
        assert_eq!(entry.extent_parameters()[0].value(), n);
        assert_eq!(entry.extent_parameters()[0].transport_slot(), 2);
        assert_eq!(entry.launch().grid_threads(), fixture::ROWS);
        let address = dense_f32_row_major_bytes(1, 0, entry.extent_parameters()[0].value());
        addresses.push(address);
        pipeline_subjects.push(entry.entry_symbol().to_owned());

        let elements = usize::try_from(fixture::ROWS * n).expect("the two-N fixture stays small");
        let input: Vec<u32> = (0..elements)
            .map(|index| u32::try_from(index).expect("the two-N fixture stays small") + 1)
            .collect();
        let element = usize::try_from(address / 4).expect("the two-N fixture stays small");
        let mapped = f32::from_bits(input[element]) * f32::from_bits(fixture::SCALE_BITS)
            + f32::from_bits(fixture::BIAS_BITS);
        assert_ne!(
            mapped.to_bits(),
            0,
            "the oracle at N={n} must read a distinctive input element",
        );
    }
    assert_eq!(
        addresses,
        [56, 60],
        "semantic (row = 1, column = 0) at N=14 and N=15",
    );
    assert_eq!(pipeline_subjects[0], pipeline_subjects[1]);
}

#[test]
fn a_live_extent_host_side_payload_disagreement_refuses_before_program_work() {
    let built = assemble(&FixtureSpec::live_extent());
    let mut program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY)
        .expect("the live-extent artifact decodes");
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_extent(
            fixture::input_key(),
            tiler_ir::shape::Axis::new(0),
            fixture::ROWS,
        )
        .expect("the static row axis binds");
    let rejection = program
        .preflight(&live_extent_environment(), &built.expected, &binder.build())
        .expect_err("binding the static row axis is not an answer for the live inner extent");
    assert!(
        matches!(
            rejection,
            LoadRejection::AbiEvaluation { .. } | LoadRejection::UnboundInputExtent { .. }
        ),
        "a disagreement must refuse before program work, got {rejection}",
    );
    let text = rejection.to_string();
    assert!(
        text.contains("Axis(1)") || text.contains("axis 1") || text.contains("UnboundInputExtent"),
        "the refusal must name the missing live axis: {text}",
    );
}
