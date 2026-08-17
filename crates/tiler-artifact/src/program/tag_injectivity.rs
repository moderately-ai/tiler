//! Shared assertions for this crate's private governed-tag tests.

use std::collections::HashMap;
use std::fmt::Debug;
use std::mem::variant_count;

use tiler_ir::schedule::{
    ArithmeticType, IndexArithmetic, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth,
};

use super::keys::{BackendKey, RouteFeatureKey};
use super::model::{
    ArtifactExecutionPolicy, BindingKind, RoutingPolicy, StageDependencyReason,
    index_arithmetic_from_tag, index_arithmetic_tag, subgroup_transfer_from_tag,
};
use super::realization::{AssessmentDisposition, RecordFamily};
use super::requirement::{
    BackendFeatureRequirement, RouteRequirement, RouteResourceDimension, RouteResourceRequirement,
};

pub(super) fn assert_tag_table<T: Copy + Debug>(table: &str, values: &[T], tag: impl Fn(T) -> u8) {
    assert!(!values.is_empty(), "{table} walked no variants");
    let mut seen = HashMap::with_capacity(values.len());
    for &value in values {
        let byte = tag(value);
        if let Some(previous) = seen.insert(byte, value) {
            panic!("{table} tag {byte:#04x} is shared by {value:?} and {previous:?}");
        }
    }
    assert_eq!(
        seen.len(),
        values.len(),
        "{table} walked a different population than its exhaustive array",
    );
}

pub(super) fn assert_tag_table_ref<T: Debug>(table: &str, values: &[T], tag: impl Fn(&T) -> u8) {
    assert!(!values.is_empty(), "{table} walked no variants");
    let mut seen = HashMap::with_capacity(values.len());
    for value in values {
        let byte = tag(value);
        if let Some(previous) = seen.insert(byte, value) {
            panic!("{table} tag {byte:#04x} is shared by {value:?} and {previous:?}");
        }
    }
    assert_eq!(
        seen.len(),
        values.len(),
        "{table} walked a different population than its exhaustive array",
    );
}

pub(super) fn assert_tag_table_with_inverse<T: Copy + Debug + Eq>(
    table: &str,
    values: &[T],
    tag: impl Fn(T) -> u8,
    from_tag: impl Fn(u8) -> Option<T>,
) {
    assert_tag_table(table, values, &tag);
    let mut claimed = [false; 256];
    for &value in values {
        let byte = tag(value);
        claimed[usize::from(byte)] = true;
        assert_eq!(
            from_tag(byte),
            Some(value),
            "{table} tag {byte:#04x} does not round-trip",
        );
    }
    let mut refused = 0_usize;
    for byte in u8::MIN..=u8::MAX {
        if claimed[usize::from(byte)] {
            continue;
        }
        assert_eq!(
            from_tag(byte),
            None,
            "{table} accepts unclaimed tag {byte:#04x}",
        );
        refused += 1;
    }
    assert_eq!(refused, 256 - values.len());
}

#[test]
fn program_model_tag_tables_are_injective_and_inverse_complete() {
    const ROUTING_POLICIES: [RoutingPolicy; variant_count::<RoutingPolicy>()] =
        [RoutingPolicy::StablePriority];
    const EXECUTION_POLICIES: [ArtifactExecutionPolicy;
        variant_count::<ArtifactExecutionPolicy>()] = [ArtifactExecutionPolicy::NativeImage];
    const BINDING_KINDS: [BindingKind; variant_count::<BindingKind>()] = [BindingKind::Buffer];
    const DEPENDENCY_REASONS: [StageDependencyReason; variant_count::<StageDependencyReason>()] = [
        StageDependencyReason::Data,
        StageDependencyReason::StorageHandoff,
    ];
    const INDEX_ARITHMETIC: [IndexArithmetic; variant_count::<IndexArithmetic>()] =
        [IndexArithmetic::CompleteU64];

    assert_tag_table_with_inverse(
        "RoutingPolicy",
        &ROUTING_POLICIES,
        RoutingPolicy::tag,
        RoutingPolicy::from_tag,
    );
    assert_tag_table_with_inverse(
        "ArtifactExecutionPolicy",
        &EXECUTION_POLICIES,
        ArtifactExecutionPolicy::tag,
        ArtifactExecutionPolicy::from_tag,
    );
    assert_tag_table_with_inverse(
        "BindingKind",
        &BINDING_KINDS,
        BindingKind::tag,
        BindingKind::from_tag,
    );
    assert_tag_table_with_inverse(
        "StageDependencyReason",
        &DEPENDENCY_REASONS,
        StageDependencyReason::tag,
        StageDependencyReason::from_tag,
    );
    assert_tag_table_with_inverse(
        "artifact IndexArithmetic",
        &INDEX_ARITHMETIC,
        index_arithmetic_tag,
        index_arithmetic_from_tag,
    );
}

#[test]
fn realization_tag_tables_are_injective_and_inverse_complete_where_available() {
    const FAMILIES: [RecordFamily; variant_count::<RecordFamily>()] =
        [RecordFamily::ScalarArithmetic];
    const DISPOSITIONS: [AssessmentDisposition; variant_count::<AssessmentDisposition>()] = [
        AssessmentDisposition::NotRequired,
        AssessmentDisposition::Required { first: 0, len: 1 },
    ];

    assert_tag_table_with_inverse(
        "RecordFamily",
        &FAMILIES,
        RecordFamily::tag,
        RecordFamily::from_tag,
    );
    assert_tag_table(
        "AssessmentDisposition",
        &DISPOSITIONS,
        AssessmentDisposition::tag,
    );
}

#[test]
fn route_requirement_tag_tables_are_injective_and_inverse_complete_where_available() {
    const DIMENSIONS: [RouteResourceDimension; variant_count::<RouteResourceDimension>()] =
        [RouteResourceDimension::SubgroupThreads];
    assert_tag_table_with_inverse(
        "RouteResourceDimension",
        &DIMENSIONS,
        RouteResourceDimension::tag,
        RouteResourceDimension::from_tag,
    );

    let resource = RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, 32)
        .expect("a nonzero resource requirement");
    let feature = BackendFeatureRequirement::new(
        BackendKey::new("tiler.metal").expect("a governed backend key"),
        RouteFeatureKey::new("tiler.metal.minimum-gpu-family")
            .expect("a governed route feature key"),
        1,
        [1],
    )
    .expect("a versioned nonempty backend feature requirement");
    let requirements: [RouteRequirement; variant_count::<RouteRequirement>()] = [
        RouteRequirement::Resource(resource),
        RouteRequirement::BackendFeature(feature),
    ];
    assert_tag_table_ref("RouteRequirement", &requirements, RouteRequirement::tag);
}

#[test]
fn subgroup_transfer_inverse_is_coupled_to_the_public_subject_encoder() {
    const TRANSFERS: [SubgroupTransfer; variant_count::<SubgroupTransfer>()] =
        [SubgroupTransfer::InRangeXorShuffle];

    let encoded_tag = |transfer| {
        let width = SubgroupWidth::new(2).expect("the smallest XOR width is valid");
        let subject = SubgroupRealizationSubject::new(width, ArithmeticType::F16, transfer)
            .expect("every enumerated transfer must define the test width");
        let mut bytes = Vec::new();
        subject.encode(&mut bytes);
        assert_eq!(bytes.len(), 6, "the public subject encoding is fixed-width");
        bytes[5]
    };
    assert_tag_table_with_inverse(
        "artifact subgroup transfer",
        &TRANSFERS,
        encoded_tag,
        subgroup_transfer_from_tag,
    );
}
