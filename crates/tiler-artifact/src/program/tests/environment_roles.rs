//! The two offered provider roles of one compilation, and their independence.
//!
//! These cases exist because the role separation is invisible in artifact
//! bytes: neither offered set is ever serialized (ADR 0072), so nothing in the
//! encoded envelope would notice if the two collapsed back into one. What the
//! separation actually buys is stated here, at the only boundary that can see
//! it.

use super::super::{
    ArtifactBuildError, ArtifactLimitKind, CompilationEnvironment, MAX_OFFERED_LOWERING_PROVIDERS,
    MAX_OFFERED_PHYSICAL_PROVIDERS,
};
use super::{lowering_provider, spare_provider};

#[test]
fn each_role_reports_only_the_providers_offered_in_it() {
    let environment =
        CompilationEnvironment::new([lowering_provider(1)], [spare_provider(7)]).unwrap();

    assert_eq!(
        environment.offered_lowering_providers(),
        [lowering_provider(1)],
    );
    assert_eq!(
        environment.offered_physical_providers(),
        [spare_provider(7)]
    );
}

/// One identity in both roles is two grants, and neither accessor loses it.
#[test]
fn one_identity_may_be_offered_in_both_roles() {
    let environment =
        CompilationEnvironment::new([lowering_provider(1)], [lowering_provider(1)]).unwrap();

    assert_eq!(
        environment.offered_lowering_providers(),
        [lowering_provider(1)],
    );
    assert_eq!(
        environment.offered_physical_providers(),
        [lowering_provider(1)],
    );
}

/// Deduplication is within one role. Offering a provider twice for lowering
/// collapses to one lowering grant and says nothing about its physical role.
#[test]
fn repetition_collapses_within_a_role_and_never_across_one() {
    let environment = CompilationEnvironment::new(
        [lowering_provider(1), lowering_provider(1)],
        std::iter::empty(),
    )
    .unwrap();

    assert_eq!(
        environment.offered_lowering_providers(),
        [lowering_provider(1)],
    );
    assert!(environment.offered_physical_providers().is_empty());
}

/// Each role is canonicalized against itself, so a crowded physical set cannot
/// reorder or pad the lowering one.
#[test]
fn each_role_is_canonically_ordered_independently() {
    let environment = CompilationEnvironment::new(
        [lowering_provider(2), lowering_provider(1)],
        [spare_provider(9), spare_provider(3)],
    )
    .unwrap();

    assert_eq!(
        environment.offered_lowering_providers(),
        [lowering_provider(1), lowering_provider(2)],
    );
    assert_eq!(
        environment.offered_physical_providers(),
        [spare_provider(3), spare_provider(9)],
    );
}

/// The bound is spent by the caller's *collected input*, before deduplication,
/// which is the existing anti-amplification rule carried into both roles.
#[test]
fn an_overrun_lowering_role_names_the_lowering_resource() {
    let crowd = (0..=u32::try_from(MAX_OFFERED_LOWERING_PROVIDERS).unwrap())
        .map(|revision| lowering_provider(revision.max(1)));

    assert_eq!(
        CompilationEnvironment::new(crowd, std::iter::empty()),
        Err(ArtifactBuildError::StructuralLimit {
            resource: ArtifactLimitKind::OfferedLoweringProviders,
            actual: MAX_OFFERED_LOWERING_PROVIDERS + 1,
            limit: MAX_OFFERED_LOWERING_PROVIDERS,
        }),
    );
}

/// The paired case. A refusal that named the lowering resource for a physical
/// overrun would send a caller to the wrong argument.
#[test]
fn an_overrun_physical_role_names_the_physical_resource() {
    let crowd = (0..=u32::try_from(MAX_OFFERED_PHYSICAL_PROVIDERS).unwrap())
        .map(|revision| spare_provider(revision.max(1)));

    assert_eq!(
        CompilationEnvironment::new(std::iter::empty(), crowd),
        Err(ArtifactBuildError::StructuralLimit {
            resource: ArtifactLimitKind::OfferedPhysicalProviders,
            actual: MAX_OFFERED_PHYSICAL_PROVIDERS + 1,
            limit: MAX_OFFERED_PHYSICAL_PROVIDERS,
        }),
    );
}

/// Neither bound is a sum of both, so a full lowering role does not consume any
/// of the physical one.
#[test]
fn the_two_bounds_are_independent_rather_than_shared() {
    let lowering = (1..=u32::try_from(MAX_OFFERED_LOWERING_PROVIDERS).unwrap())
        .map(lowering_provider)
        .collect::<Vec<_>>();
    let physical = (1..=u32::try_from(MAX_OFFERED_PHYSICAL_PROVIDERS).unwrap())
        .map(spare_provider)
        .collect::<Vec<_>>();

    let environment = CompilationEnvironment::new(lowering, physical).unwrap();

    assert_eq!(
        environment.offered_lowering_providers().len(),
        MAX_OFFERED_LOWERING_PROVIDERS,
    );
    assert_eq!(
        environment.offered_physical_providers().len(),
        MAX_OFFERED_PHYSICAL_PROVIDERS,
    );
}
