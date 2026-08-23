use super::super::{
    RegionId, ScheduleBuildError, ScheduleComponent, ScheduledRegionBuilder,
    ScheduledRegionDiagnostic,
};
use super::support::partitioned_copy_builder;
use crate::shape::Shape;

#[test]
fn setting_a_component_twice_is_a_local_insertion_error() {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
    assert_eq!(
        builder.iteration_shape(Shape::from_dims([4])),
        Err(ScheduleBuildError::ComponentAlreadySet {
            component: ScheduleComponent::IterationShape,
        })
    );
}

#[test]
fn incomplete_region_reports_the_missing_component() {
    let error = ScheduledRegionBuilder::new(RegionId::new(0))
        .build()
        .unwrap_err();
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::IncompleteRegion {
            component: ScheduleComponent::IterationShape,
        }]
    );
}

#[test]
fn a_builder_missing_only_the_program_reports_the_region_program_component() {
    let mut builder = partitioned_copy_builder(&Shape::from_dims([4]), 0, &[(0, 1), (1, 3)]);
    builder.program = None;
    assert_eq!(
        builder.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::IncompleteRegion {
            component: ScheduleComponent::RegionProgram,
        }]
    );
}
