//! Live-device route requirements: vocabulary, satisfaction, and subjects.

use super::super::BackendFeatureRequirement;
use super::super::{
    ArtifactBuildError, ArtifactEntityKind, ArtifactProgramBuilder, BackendKey,
    CompilationEnvironment, MAX_ROUTE_FEATURE_PAYLOAD_BYTES, RouteFeatureKey,
    RouteRequirementError, RouteRequirementSubject, RouteResourceDimension,
    RouteResourceRequirement,
};
use super::offered_physical;
use super::{
    SCALE_BITS, declare_realization, formulas, fused_program, lowering_provider, payload,
    route_feature, route_resource, selection, semantic_program, variant,
};

/// Every neutral dimension round-trips through its governed tag, distinctly.
///
/// Population-counted against `ALL` rather than against a list written here, so
/// a dimension added to the vocabulary lands in this check instead of leaving it
/// silently covering a subset.
#[test]
fn every_route_resource_dimension_round_trips_through_its_governed_tag() {
    let mut tags = Vec::new();
    for dimension in RouteResourceDimension::ALL {
        let tag = dimension.tag();
        assert_eq!(RouteResourceDimension::from_tag(tag), Some(dimension));
        assert!(!tags.contains(&tag), "tag {tag:#04x} is not distinct");
        tags.push(tag);
    }
    assert_eq!(
        tags.len(),
        RouteResourceDimension::ALL.len(),
        "every dimension the vocabulary names was checked",
    );
    assert_eq!(RouteResourceDimension::from_tag(0x00), None);
    assert_eq!(RouteResourceDimension::from_tag(0xff), None);
}

/// A subgroup width is satisfied by an exactly equal observation and nothing else.
///
/// The **wider** case is the load-bearing one and it is the case the superseded
/// floor accepted: a device executing more threads per subgroup than the route
/// was verified at runs lane arithmetic nothing checked, so admitting it is the
/// silent wrongness this relation exists to refuse. The narrower case is driven
/// beside it so a relation that refused everything could not pass as a fix.
///
/// Populations are named rather than sampled: every dimension the vocabulary
/// carries is exercised, and the count is asserted, so a dimension added without
/// a relation of its own lands here instead of leaving a subset checked.
#[test]
fn a_route_resource_row_is_satisfied_only_by_an_exactly_equal_observation() {
    const REQUIRED: u64 = 32;

    let mut checked = 0;
    for dimension in RouteResourceDimension::ALL {
        let row = RouteResourceRequirement::new(dimension, REQUIRED).expect("a nonzero quantity");
        assert_eq!(row.required(), REQUIRED);
        assert!(
            row.is_satisfied_by(REQUIRED),
            "{dimension} must accept the width the route was verified at",
        );
        assert!(
            !row.is_satisfied_by(REQUIRED - 1),
            "{dimension} must refuse a narrower device",
        );
        assert!(
            !row.is_satisfied_by(REQUIRED * 2),
            "{dimension} must refuse a wider device, which a floor accepted",
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        RouteResourceDimension::ALL.len(),
        "every dimension the vocabulary names was checked",
    );
}

/// Each way a route requirement can be malformed is refused by its own cause.
///
/// Every case is paired with the well-formed neighbour it perturbs, so a
/// rejection is attributable to the one field that changed rather than to a
/// constructor that refuses everything.
#[test]
fn a_malformed_route_requirement_is_refused_by_its_own_cause() {
    assert!(RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, 32).is_ok());
    assert_eq!(
        RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, 0),
        Err(RouteRequirementError::ZeroResourceQuantity {
            dimension: RouteResourceDimension::SubgroupThreads,
        }),
    );

    let owner = || BackendKey::new("tiler.metal").unwrap();
    let key = || RouteFeatureKey::new("tiler.metal.route-requirement.minimum-gpu-family").unwrap();
    assert!(BackendFeatureRequirement::new(owner(), key(), 1, b"apple9").is_ok());
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 0, b"apple9"),
        Err(RouteRequirementError::ZeroFeatureVersion),
    );
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 1, b""),
        Err(RouteRequirementError::EmptyFeaturePayload),
    );
    let oversized = vec![0_u8; MAX_ROUTE_FEATURE_PAYLOAD_BYTES + 1];
    assert_eq!(
        BackendFeatureRequirement::new(owner(), key(), 1, &oversized),
        Err(RouteRequirementError::FeaturePayloadTooLong {
            bytes: MAX_ROUTE_FEATURE_PAYLOAD_BYTES + 1,
            limit: MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
        }),
    );
    assert!(
        BackendFeatureRequirement::new(owner(), key(), 1, &oversized[1..]).is_ok(),
        "the bound admits exactly its own length",
    );
}

/// Two rows constraining one subject are refused at construction.
///
/// The differing quantity is what makes them contradictory: the builder holds
/// two answers to one question and nothing can say which the producer meant.
/// A *different* subject is accepted in the same breath, so the rejection is
/// about the subject rather than about a second row at all.
#[test]
fn two_route_requirements_naming_one_subject_are_refused() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let id = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();

    draft.require_route(id, route_resource(32)).unwrap();
    assert_eq!(
        draft.require_route(id, route_resource(64)),
        Err(ArtifactBuildError::DuplicateRouteRequirementSubject {
            subject: Box::new(RouteRequirementSubject::Resource {
                dimension: RouteResourceDimension::SubgroupThreads,
            }),
        }),
    );
    // A distinct subject is admitted, so what was refused is the repetition.
    draft
        .require_route(
            id,
            route_feature("tiler.metal.route-requirement.a", 1, b"x"),
        )
        .unwrap();
    // The same key at another version is another subject, deliberately: one key
    // at two versions can mean two things, so they are not the same question.
    draft
        .require_route(
            id,
            route_feature("tiler.metal.route-requirement.a", 2, b"x"),
        )
        .unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().unwrap();
    assert_eq!(
        artifact
            .variants()
            .next()
            .expect("one variant")
            .route_requirements()
            .len(),
        3,
    );
}

/// A variant handle another builder minted cannot attach a requirement.
#[test]
fn a_route_requirement_needs_a_variant_this_builder_minted() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let mut first = {
        let environment =
            CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
        ArtifactProgramBuilder::new(&semantic, environment).unwrap()
    };
    let mut second = {
        let environment =
            CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
        ArtifactProgramBuilder::new(&semantic, environment).unwrap()
    };
    for draft in [&mut first, &mut second] {
        draft
            .select_lowering_provider(selection(provider.clone()))
            .unwrap();
    }
    let descriptor = first.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut first);
    let foreign = first
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        second.require_route(foreign, route_resource(32)),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Variant,
        }),
    );
    // The handle is good against the builder that minted it, which is what makes
    // the refusal above about ownership rather than about the handle's shape.
    assert!(first.require_route(foreign, route_resource(32)).is_ok());
}
