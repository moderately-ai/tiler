//! The ADR 0013 runtime plan-determinism subject, exercised end to end.
//!
//! The accepted stability-subject perturbations, each over its own subject with
//! the checks left unchanged: object-only relinking moves the envelope digest
//! and the subject while the pre-compilation identity holds; another routing
//! rank moves only the selected coordinate; another delivery position moves the
//! subject even where the kernel-program identity is shared; a provider
//! descriptor or revision change moves the environment identity and filters a
//! stale observation by name; and the device-free and unregistered paths filter
//! a claimed cell before its guard while `Unclaimed` alternatives stay
//! routable.
//!
//! Every positive case reads the subject through the adapter's captures at the
//! two stages the surface exposes it — the pre-commit `Preflight` and the
//! committed `RoutedDispatch` — and asserts the two are one value, which is the
//! carried-unchanged-across-the-commit obligation checked rather than stated.

use crate::adapter::{ScalarHostAdapter, Stage};
use crate::fixture::{self, FixtureSpec, PackagedPlan, assemble, assemble_portfolio};
use crate::{OPERANDS, Outcome, SOLE_DELIVERY, bind_facts};

use tiler_artifact::program::TargetEnvironmentDeclaration;
use tiler_runtime::adapter::{AdapterRouteFailure, route_with_adapter};
use tiler_runtime::load::{
    DecodedProgram, LoadRejection, TargetEnvironmentIneligibility, TargetEnvironmentObservation,
    VariantIneligibility,
};

/// The canonical plan-subject domain, restated so a moved spelling is caught.
const SUBJECT_DOMAIN: &[u8] = b"tiler.runtime.plan-determinism-subject.v1\0";
/// The compatibility-identity domain the subject's environment tail opens with.
const ENVIRONMENT_DOMAIN: &[u8] = b"tiler.target-environment-compatibility.v1\0";
/// Governed digest width, restated from the wire contract.
const DIGEST_BYTES: usize = 32;

/// The claimed fused member every positive case here starts from.
fn claimed_spec() -> FixtureSpec {
    claimed_with(fixture::environment_declaration())
}

fn claimed_with(declaration: TargetEnvironmentDeclaration) -> FixtureSpec {
    FixtureSpec {
        environment: Some(declaration),
        claim_plan: true,
        ..FixtureSpec::for_plan(PackagedPlan::Fused)
    }
}

/// An adapter registering the scalar host's provider schema.
fn registered_adapter() -> ScalarHostAdapter {
    ScalarHostAdapter::new(&OPERANDS).registering_environment_schema(fixture::environment_schema())
}

/// Routes one already-assembled fixture through one adapter.
fn route_built(
    built: &fixture::Fixture,
    mut host: ScalarHostAdapter,
) -> (Outcome, ScalarHostAdapter) {
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the fixture artifact decodes");
    let facts = bind_facts(&program);
    let outcome = route_with_adapter(&mut program, &mut host, &built.expected, &facts);
    (outcome, host)
}

/// The parsed canonical plan-subject encoding.
///
/// Parsed from the exact wire grammar — domain, fixed-width envelope digest,
/// big-endian rank and delivery, then the length-framed environment identity —
/// so a structural drift in the encoding fails here by name rather than being
/// absorbed by opaque-byte comparisons.
struct ParsedSubject {
    digest: Vec<u8>,
    rank: u32,
    delivery: u32,
    environment: Vec<u8>,
}

fn parse_subject(bytes: &[u8]) -> ParsedSubject {
    assert!(
        bytes.starts_with(SUBJECT_DOMAIN),
        "a plan subject opens with its versioned domain",
    );
    let rest = &bytes[SUBJECT_DOMAIN.len()..];
    let digest = rest[..DIGEST_BYTES].to_vec();
    let rest = &rest[DIGEST_BYTES..];
    let rank = u32::from_be_bytes(rest[..4].try_into().expect("a fixed-width rank"));
    let delivery = u32::from_be_bytes(rest[4..8].try_into().expect("a fixed-width delivery"));
    let framed = u64::from_be_bytes(rest[8..16].try_into().expect("a fixed-width frame"));
    let environment = rest[16..].to_vec();
    assert_eq!(
        u64::try_from(environment.len()).expect("a bounded identity"),
        framed,
        "the environment frame covers exactly the remaining bytes",
    );
    assert!(
        environment.starts_with(ENVIRONMENT_DOMAIN),
        "the environment identity opens with its versioned domain",
    );
    ParsedSubject {
        digest,
        rank,
        delivery,
        environment,
    }
}

/// Routes one claimed fixture and returns the committed subject's bytes.
fn routed_subject(built: &fixture::Fixture, host: ScalarHostAdapter) -> Vec<u8> {
    let (outcome, host) = route_built(built, host);
    outcome.expect("the attested claimed route completes");
    let at_plan = host
        .subject_at_plan()
        .expect("an attested claimed route publishes its subject on the preflight");
    let at_dispatch = host
        .subject_at_dispatch()
        .expect("the committed route carries the subject");
    assert_eq!(
        at_plan, at_dispatch,
        "the subject is carried unchanged across the one-way commit",
    );
    at_dispatch.to_vec()
}

// -------------------------------------------------------------------------
// The positive path
// -------------------------------------------------------------------------

/// One claimed member, one registered adapter, one subject — structurally
/// exact, present at both surfaces, identical across the commit.
#[test]
fn an_attested_claimed_route_carries_one_exact_subject_across_the_commit() {
    let built = assemble(&claimed_spec());
    let subject = routed_subject(&built, registered_adapter());
    let parsed = parse_subject(&subject);
    assert_eq!(parsed.rank, 0, "the sole member routes at rank zero");
    assert_eq!(parsed.delivery, 0, "the sole delivery position is zero");
    assert_eq!(parsed.digest.len(), DIGEST_BYTES);

    // The positive control the movement cases below lean on: reassembling and
    // re-routing the identical artifact yields the identical subject.
    assert_eq!(
        subject,
        routed_subject(&assemble(&claimed_spec()), registered_adapter()),
        "one artifact, one route, one subject",
    );
}

/// An unclaimed route completes with no subject: `None` is a state, not a
/// failure, and nothing invents a claim the artifact never proved.
#[test]
fn an_unclaimed_route_carries_no_subject() {
    let built = assemble(&FixtureSpec::for_plan(PackagedPlan::Fused));
    let (outcome, host) = route_built(&built, registered_adapter());
    outcome.expect("the unclaimed route completes");
    assert_eq!(host.subject_at_plan(), None);
    assert_eq!(host.subject_at_dispatch(), None);
}

// -------------------------------------------------------------------------
// Fail-closed filtering, before the guard and before the commit
// -------------------------------------------------------------------------

/// Pulls the one plan-determinism filter out of a no-eligible-variant refusal.
fn filtered_reason(outcome: Outcome) -> TargetEnvironmentIneligibility {
    let failure = outcome.expect_err("the claimed cell must filter");
    let AdapterRouteFailure::Load(LoadRejection::NoEligibleVariant { packaged, filtered }) =
        failure
    else {
        panic!("a filtered claim is a no-eligible-variant refusal: {failure}");
    };
    assert_eq!(packaged, 1, "the fixture packages exactly one member");
    assert_eq!(filtered.len(), 1, "every packaged member is accounted for");
    let VariantIneligibility::PlanDeterminismEnvironment {
        delivery_position,
        reason,
    } = filtered[0].reason.clone()
    else {
        panic!("the claim filters on its environment: {}", filtered[0]);
    };
    assert_eq!(delivery_position, SOLE_DELIVERY);
    reason
}

/// The device-free public paths filter a claimed cell as unverified, and an
/// `Unclaimed` alternative behind it stays routable.
#[test]
fn a_device_free_path_filters_a_claimed_cell_and_routes_the_unclaimed_alternative() {
    // The single claimed member: preflight refuses with the exact class.
    let built = assemble(&claimed_spec());
    let mut program =
        DecodedProgram::decode(&built.bytes, SOLE_DELIVERY).expect("the artifact decodes");
    let facts = bind_facts(&program);
    let refusal = program
        .preflight(&fixture::scalar_host(), &built.expected, &facts)
        .expect_err("a caller-stated environment cannot mint a live attestation");
    let LoadRejection::NoEligibleVariant { filtered, .. } = &refusal else {
        panic!("the claimed cell filters rather than failing later: {refusal}");
    };
    assert!(
        matches!(
            &filtered[0].reason,
            VariantIneligibility::PlanDeterminismEnvironment {
                reason: TargetEnvironmentIneligibility::Unattested,
                ..
            },
        ),
        "the device-free filter names the unattested class: {}",
        filtered[0],
    );

    // The claimed member ahead of an unclaimed two-stage sibling: the claim
    // filters, the sibling routes, and no subject is invented for it. The
    // sibling drops its deferred predicates and route requirements, because a
    // device-free preflight refuses those on their own terms and this case is
    // about the plan-determinism filter alone.
    let portfolio = assemble_portfolio(&[
        claimed_spec(),
        FixtureSpec {
            deferred_predicates: Vec::new(),
            route_requirements: Vec::new(),
            ..FixtureSpec::for_plan(PackagedPlan::Materialized)
        },
    ]);
    let mut program =
        DecodedProgram::decode(&portfolio.bytes, SOLE_DELIVERY).expect("the portfolio decodes");
    let facts = bind_facts(&program);
    let preflight = program
        .preflight(&fixture::scalar_host(), &portfolio.expected, &facts)
        .expect("the unclaimed alternative stays routable");
    assert_eq!(
        preflight.entries().len(),
        2,
        "the two-stage materialized member is the one that routed",
    );
    assert!(
        preflight.plan_determinism_subject().is_none(),
        "an unclaimed route promises nothing",
    );
}

/// An adapter registering no schema filters the claim as provider-unavailable,
/// without ever being asked to observe.
#[test]
fn an_unregistered_provider_filters_the_claim_before_any_observation() {
    let built = assemble(&claimed_spec());
    let (outcome, host) = route_built(&built, ScalarHostAdapter::new(&OPERANDS));
    assert!(
        matches!(
            filtered_reason(outcome),
            TargetEnvironmentIneligibility::ProviderUnavailable,
        ),
        "an unsupported provider filters by name",
    );
    assert!(
        !host.stages.contains(&Stage::ObserveTargetEnvironment),
        "an unregistered adapter is never asked to observe",
    );

    // The unclaimed alternative still routes through the same adapter.
    let portfolio = assemble_portfolio(&[
        claimed_spec(),
        FixtureSpec::for_plan(PackagedPlan::Materialized),
    ]);
    let (outcome, host) = route_built(&portfolio, ScalarHostAdapter::new(&OPERANDS));
    outcome.expect("the unclaimed alternative routes");
    assert_eq!(host.subject_at_dispatch(), None);
}

/// An unavailable observation filters the claim; it is not a context failure.
#[test]
fn an_unavailable_observation_filters_the_claim_but_not_the_route() {
    let built = assemble(&claimed_spec());
    let host =
        registered_adapter().observing_environment(TargetEnvironmentObservation::Unavailable {
            reason: "the process observer is offline".to_owned(),
        });
    let (outcome, _) = route_built(&built, host);
    let TargetEnvironmentIneligibility::ObservationUnavailable {
        provider,
        schema,
        reason,
    } = filtered_reason(outcome)
    else {
        panic!("an unavailable observation filters by name");
    };
    assert_eq!(provider.as_ref(), &fixture::environment_provider());
    assert_eq!((schema.major(), schema.minor()), (1, 0));
    assert_eq!(reason, "the process observer is offline");
}

/// A declaration under an unregistered provider revision or schema version
/// filters by the exact mismatch class.
#[test]
fn declared_provider_and_schema_mismatches_filter_by_name() {
    // The provider-revision half of the accepted perturbation: revision 2 is
    // declared and revision 1 is registered.
    let built = assemble(&claimed_with(fixture::revised_provider_declaration()));
    let TargetEnvironmentIneligibility::ProviderMismatch {
        declared,
        registered,
    } = filtered_reason(route_built(&built, registered_adapter()).0)
    else {
        panic!("an unregistered provider revision filters by name");
    };
    assert_eq!(declared.revision(), 2);
    assert_eq!(registered.revision(), 1);

    // The schema-version half: 2.0 declared, 1.0 registered.
    let revised_schema = TargetEnvironmentDeclaration::new(
        fixture::environment_provider(),
        tiler_artifact::program::SchemaVersion::new(2, 0),
        fixture::environment_declaration().descriptor().clone(),
    )
    .expect("a nonzero schema major");
    let built = assemble(&claimed_with(revised_schema));
    let TargetEnvironmentIneligibility::SchemaMismatch {
        declared,
        registered,
    } = filtered_reason(route_built(&built, registered_adapter()).0)
    else {
        panic!("an unregistered schema version filters by name");
    };
    assert_eq!((declared.major(), declared.minor()), (2, 0));
    assert_eq!((registered.major(), registered.minor()), (1, 0));
}

// -------------------------------------------------------------------------
// The accepted subject perturbations
// -------------------------------------------------------------------------

/// Object-only relinking: one pre-compilation identity, two envelope digests,
/// two subjects.
///
/// The artifact-digest perturbation: only the emitted object bytes move, so
/// the canonical artifact identity — and with it the expansion-cache subject,
/// which is a function of that pre-compilation identity — stays equal, while
/// the object-bearing envelope digest and the plan subject both move. Exactly
/// one parsed segment moves, which is what separates this from a coordinate or
/// environment change.
#[test]
fn object_only_relinking_moves_the_envelope_digest_and_the_subject() {
    let first = assemble(&claimed_spec());
    let relinked = assemble(&FixtureSpec {
        code: crate::image::encode(&{
            let mut image = fixture::sound_image();
            // One constant of the carried object: a relink that emits a
            // different instruction stream for the same compilation subject.
            image.entries[0].scale_bits = f32::to_bits(3.0);
            image
        }),
        ..claimed_spec()
    });
    assert_eq!(
        first.expected.as_bytes(),
        relinked.expected.as_bytes(),
        "artifact identity is a pre-compilation subject and excludes the object",
    );

    let subject = parse_subject(&routed_subject(&first, registered_adapter()));
    let relinked_subject = parse_subject(&routed_subject(&relinked, registered_adapter()));
    assert_ne!(
        subject.digest, relinked_subject.digest,
        "object-only relinking must move the envelope digest",
    );
    assert_eq!(subject.rank, relinked_subject.rank);
    assert_eq!(subject.delivery, relinked_subject.delivery);
    assert_eq!(
        subject.environment, relinked_subject.environment,
        "the declared environment does not move with the object",
    );
}

/// Another routing rank: one envelope, one environment, two coordinates.
///
/// The selected-variant perturbation, rank half: the same claimed portfolio is
/// routed twice with different bound extents, so the aligned member is
/// selected at `N = 16` and the general member at `N = 8`. The digest and the
/// environment hold; the rank and the projected kernel-program identity move.
#[test]
fn another_routing_rank_moves_only_the_selected_coordinate() {
    let claimed_live = |spec: FixtureSpec| FixtureSpec {
        environment: Some(fixture::environment_declaration()),
        claim_plan: true,
        ..spec
    };
    let built = assemble_portfolio(&[
        claimed_live(FixtureSpec::live_extent_aligned()),
        claimed_live(FixtureSpec::live_extent()),
    ]);
    let pool_elements =
        usize::try_from(crate::retained_pool_bytes() / 4).expect("the pool is small");
    let input: Vec<u32> = (0..pool_elements)
        .map(|index| f32::from(u16::try_from(index).expect("a small pool") + 1).to_bits())
        .collect();

    let mut subjects = Vec::new();
    let mut kernel_programs = Vec::new();
    for extent in [16_u64, 8] {
        let mut program = DecodedProgram::decode(&built.bytes, SOLE_DELIVERY)
            .expect("the claimed portfolio decodes");
        let mut host = ScalarHostAdapter::new(&input)
            .registering_environment_schema(fixture::environment_schema());
        route_with_adapter(
            &mut program,
            &mut host,
            &built.expected,
            &crate::live_extent_facts(extent),
        )
        .unwrap_or_else(|failure| panic!("N={extent} must route: {failure}"));
        subjects.push(parse_subject(
            host.subject_at_dispatch()
                .expect("a claimed route's subject"),
        ));
        kernel_programs.push(
            host.subject_kernel_program()
                .expect("the subject projects its kernel program")
                .to_vec(),
        );
    }

    assert_eq!(subjects[0].rank, 0, "N=16 selects the aligned member");
    assert_eq!(
        subjects[1].rank, 1,
        "N=8 falls through to the general member"
    );
    assert_eq!(
        subjects[0].digest, subjects[1].digest,
        "one envelope, one digest",
    );
    assert_eq!(
        subjects[0].environment, subjects[1].environment,
        "one declared environment class",
    );
    assert_ne!(
        kernel_programs[0], kernel_programs[1],
        "the coordinate fixes the variant, so the projection moves with it",
    );
}

/// Another delivery position: one envelope, one rank, one kernel program, two
/// subjects.
///
/// The selected-variant perturbation, delivery half: one member realized by
/// two compiled objects. Selecting the other position moves the subject even
/// though the kernel-program identity is shared, because the position fixes
/// *which executable objects run* — which is exactly why kernel-program
/// identity is a projection and not the coordinate.
#[test]
fn another_delivery_position_moves_the_subject_with_a_shared_kernel_program() {
    let built = fixture::assemble_two_delivery_claimed();
    let mut subjects = Vec::new();
    let mut kernel_programs = Vec::new();
    for delivery in [0_usize, 1] {
        let mut program =
            DecodedProgram::decode(&built.bytes, delivery).expect("both positions decode");
        let facts = bind_facts(&program);
        let mut host = registered_adapter();
        route_with_adapter(&mut program, &mut host, &built.expected, &facts)
            .unwrap_or_else(|failure| panic!("delivery {delivery} must route: {failure}"));
        subjects.push(parse_subject(
            host.subject_at_dispatch()
                .expect("a claimed route's subject"),
        ));
        kernel_programs.push(
            host.subject_kernel_program()
                .expect("the subject projects its kernel program")
                .to_vec(),
        );
    }

    assert_eq!(subjects[0].delivery, 0);
    assert_eq!(subjects[1].delivery, 1);
    assert_eq!(subjects[0].rank, subjects[1].rank, "one member, one rank");
    assert_eq!(
        subjects[0].digest, subjects[1].digest,
        "one envelope, one digest",
    );
    assert_eq!(
        subjects[0].environment, subjects[1].environment,
        "one declared environment class",
    );
    assert_eq!(
        kernel_programs[0], kernel_programs[1],
        "two positions are one plan, so the projected program is shared",
    );
    assert!(
        !subjects[0].digest.is_empty(),
        "the digest segment is populated"
    );
}

/// A moved provider descriptor: the environment identity moves the subject,
/// and a stale live observation filters by name.
///
/// The target-environment perturbation: one descriptor field moves, everything
/// else holds. Routed against the *old* observation the claim filters as an
/// exact environment mismatch; routed against the matching observation it
/// yields a subject whose environment segment — and only that segment — moved.
#[test]
fn a_moved_provider_descriptor_moves_the_subject_and_filters_a_stale_observation() {
    let declared = assemble(&claimed_spec());
    let altered = assemble(&claimed_with(fixture::altered_environment_declaration()));

    // The stale observation: the adapter still observes the first class.
    let TargetEnvironmentIneligibility::EnvironmentMismatch {
        declared: d,
        observed,
    } = filtered_reason(route_built(&altered, registered_adapter()).0)
    else {
        panic!("a stale observation filters as an exact environment mismatch");
    };
    assert_eq!(d.as_bytes(), fixture::ALTERED_ENVIRONMENT_DESCRIPTOR);
    assert_eq!(observed.as_bytes(), fixture::ENVIRONMENT_DESCRIPTOR);

    // The matching observation: the subject's environment segment moves.
    let subject = parse_subject(&routed_subject(&declared, registered_adapter()));
    let altered_host =
        registered_adapter().observing_environment(TargetEnvironmentObservation::Observed(
            tiler_artifact::program::TargetEnvironmentDescriptor::new(
                fixture::ALTERED_ENVIRONMENT_DESCRIPTOR,
            )
            .expect("a bounded fixture descriptor"),
        ));
    let altered_subject = parse_subject(&routed_subject(&altered, altered_host));
    assert_ne!(
        subject.environment, altered_subject.environment,
        "one moved descriptor field is another compatibility class",
    );
    assert_eq!(subject.rank, altered_subject.rank);
    assert_eq!(subject.delivery, altered_subject.delivery);
    // The digest also moves — the declaration is identity-bearing — which is
    // stated rather than asserted away: the perturbation's subject is the
    // environment, and the digest movement is its packaged consequence.
    assert_ne!(subject.digest, altered_subject.digest);
}
