use super::super::{
    KernelDiagnostic, KernelLoweringError, OperationRef, OperationView, lower_scheduled_region,
};
use super::support::{
    cooperative_barrier, cooperative_point, cooperative_region, multi_round_cooperative_region,
    staged_accesses,
};
use crate::schedule::{
    ContributorCoverage, CooperativePhase, LocalCoordinateSource, ParticipantRange,
    ParticipantSpace, PhaseId, ReductionTopology, ScheduledRegionBuilder, StagedRead, StagedSpan,
    StagedWrite, StagingId, SyncPointId, SynchronizationKind, SynchronizationPoint,
    WorkgroupStaging,
};
use std::cell::Cell;

/// The cooperative region derives its workgroup storage into its requirements.
///
/// The value a feasibility authority composes against a target's declared
/// threadgroup memory. Nothing here claims a target supplies it.
#[test]
fn a_cooperative_region_requires_the_workgroup_storage_its_tile_allocates() {
    let scheduled = cooperative_region();
    assert_eq!(scheduled.requirements().local_memory_bytes, 12);
    assert_eq!(scheduled.requirements().threads_per_workgroup, 3);
}

/// The cooperative region lowers to a verified kernel, and the body is exact.
///
/// This is the whole vertical's positive evidence at the KIR layer: a schedule
/// that owns a synchronization point produces a body that stages, fences, and
/// consumes, and the verifier admits it. Every structural claim below is
/// asserted rather than described, because the shape is the correctness
/// argument — the fence sits between the two phases and outside both guards.
#[test]
fn a_cooperative_region_lowers_to_a_staged_fenced_body() {
    let scheduled = cooperative_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the cooperative region lowers");

    // Exactly one barrier, realizing the schedule's one point, at the top level.
    let top: Vec<_> = kernel.body().operations().map(OperationRef::view).collect();
    let barriers: Vec<_> = top
        .iter()
        .filter_map(|view| match view {
            OperationView::Barrier { spec } => Some(*spec),
            _ => None,
        })
        .collect();
    assert_eq!(
        barriers.len(),
        1,
        "the fence is not at the kernel's top level"
    );
    assert_eq!(barriers[0], &cooperative_barrier());

    // The fence sits between the two guarded regions, not inside either.
    let guarded: Vec<usize> = top
        .iter()
        .enumerate()
        .filter(|(_, view)| matches!(view, OperationView::Predicated { .. }))
        .map(|(position, _)| position)
        .collect();
    let fence = top
        .iter()
        .position(|view| matches!(view, OperationView::Barrier { .. }))
        .expect("the body carries a fence");
    assert_eq!(guarded.len(), 2);
    assert!(guarded[0] < fence && fence < guarded[1]);

    // The producing phase writes staging and the consuming phase reads it. Two
    // static reads, not three: the fold seeds at the first slot and its bounded
    // loop carries the remaining `participants - 1`.
    let (writes, reads) = staged_accesses(&kernel);
    assert_eq!(writes, [PhaseId::FIRST]);
    assert_eq!(reads, [PhaseId::new(1); 2]);

    // The kernel declares the synchronization realization its schedule requires,
    // and it is the *derived* one rather than anything the body stated.
    assert_eq!(
        kernel.requirements().synchronization,
        Some(cooperative_point().subject)
    );
}

/// A verifying rank-two tile reaches the lowerer's named shape refusal.
///
/// This is the public boundary ADR 0097 records: the schedule vocabulary can
/// state and verify the tile, while this canonical body has only the linear
/// local coordinate form.
#[test]
fn a_rank_two_cooperative_tile_is_refused_by_lowering_shape() {
    let mut region = cooperative_region().region().clone();
    let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
    else {
        panic!("the fixture carries a cooperative tile")
    };
    tile.coordinates.source = LocalCoordinateSource::LocalWorkgroupPosition;
    tile.coordinates.participants =
        ParticipantSpace::new(&[1, 3]).expect("rank two is within the bound");
    tile.phases[0].writes[0].span =
        StagedSpan::new(&[0, 1], 0, 1).expect("rank two is within the bound");
    tile.phases[1].reads[0].span =
        StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
    let scheduled = ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("the rank-two tile verifies before lowering");

    assert_eq!(
        lower_scheduled_region(&scheduled),
        Err(KernelLoweringError::Verification(
            KernelDiagnostic::CooperativeLoweringShape
        ))
    );
}

/// Verifier-admitted variants outside the canonical cooperative body refuse by
/// the lowering's named shape rule.
///
/// These subjects are built from the successful fixture, re-verified through
/// the public schedule builder, then lowered through the public entry point.
/// Each is consequently a real representable-but-not-emittable schedule rather
/// than a malformed private fixture.
#[test]
fn cooperative_lowering_refuses_verified_shape_variants() {
    let shape = KernelDiagnostic::CooperativeLoweringShape;
    let checked = Cell::new(0_u8);
    let refusal = |label: &str, edit: &dyn Fn(&mut crate::schedule::ScheduledRegion)| {
        let mut region = cooperative_region().region().clone();
        edit(&mut region);
        checked.set(checked.get().saturating_add(1));
        let scheduled = ScheduledRegionBuilder::from_region(region)
            .build()
            .expect("the perturbed cooperative schedule verifies before lowering");
        assert_eq!(
            lower_scheduled_region(&scheduled),
            Err(KernelLoweringError::Verification(shape)),
            "{label}"
        );
    };

    refusal("a second complete staging allocation", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        let staging = WorkgroupStaging {
            id: StagingId::new(1),
            ..tile.staging[0]
        };
        let write_span = tile.phases[0].writes[0].span;
        let read_span = tile.phases[1].reads[0].span;
        tile.staging.push(staging);
        tile.phases[0].writes.push(StagedWrite {
            staging: staging.id,
            span: write_span,
        });
        tile.phases[1].reads.push(StagedRead {
            staging: staging.id,
            span: read_span,
        });
    });
    refusal("a third phase", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases.push(CooperativePhase {
            id: PhaseId::new(2),
            participation: ParticipantRange { first: 0, count: 3 },
            writes: Vec::new(),
            reads: Vec::new(),
        });
    });
    refusal("a valid two-slot producing write", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.staging[0].slots = 6;
        tile.phases[0].writes[0].span =
            StagedSpan::new(&[2], 0, 2).expect("rank one is within the bound");
        tile.phases[1].reads[0].span =
            StagedSpan::new(&[0], 0, 6).expect("rank one is within the bound");
    });
    refusal("a partial consuming read", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[1].reads[0].span.count = 1;
    });
    refusal("a non-prefix commit", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.commit.first = 1;
    });

    assert_eq!(
        checked.get(),
        5,
        "the verifier-admitted cooperative refusal census changed"
    );
}

/// Defensive cooperative-plan refusals are driven from the successful fixture.
///
/// The schedule verifier deliberately rejects these malformed staging subjects
/// first, so the test-only projection calls the actual `cooperative_plan` and
/// proves the lowering retains its own named boundary. Each subject changes one
/// independently mutable input to one refusal group; the count is a floor over
/// the separately tested clauses, not a hand-written claim that all source
/// branches are dynamically reachable.
#[test]
fn cooperative_plan_refuses_each_defensive_lowering_shape() {
    let shape = KernelDiagnostic::CooperativeLoweringShape;
    let checked = Cell::new(0_u8);
    let refusal = |label: &str, edit: &dyn Fn(&mut crate::schedule::ScheduledRegion)| {
        let mut region = cooperative_region().region().clone();
        edit(&mut region);
        checked.set(checked.get().saturating_add(1));
        assert_eq!(
            super::super::lower::cooperative_plan_shape_check(&region),
            Err(shape),
            "{label}"
        );
    };

    refusal("an otherwise-unused second staging allocation", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.staging.push(WorkgroupStaging {
            id: StagingId::new(1),
            ..tile.staging[0]
        });
    });
    refusal(
        "an extra producing read violates the access layout",
        &|region| {
            let ReductionTopology::CooperativeWorkgroup { tile, .. } =
                &mut region.schedule.reduction
            else {
                panic!("the fixture carries a cooperative tile")
            };
            let read_span = tile.phases[1].reads[0].span;
            tile.phases[0].reads.push(StagedRead {
                staging: StagingId::FIRST,
                span: read_span,
            });
        },
    );
    refusal(
        "a producing staging ID that differs from the allocation",
        &|region| {
            let ReductionTopology::CooperativeWorkgroup { tile, .. } =
                &mut region.schedule.reduction
            else {
                panic!("the fixture carries a cooperative tile")
            };
            tile.phases[0].writes[0].staging = StagingId::new(1);
        },
    );
    refusal(
        "a consuming staging ID that differs from the allocation",
        &|region| {
            let ReductionTopology::CooperativeWorkgroup { tile, .. } =
                &mut region.schedule.reduction
            else {
                panic!("the fixture carries a cooperative tile")
            };
            tile.phases[1].reads[0].staging = StagingId::new(1);
        },
    );
    refusal(
        "staged accesses agreeing on an undeclared staging ID",
        &|region| {
            let ReductionTopology::CooperativeWorkgroup { tile, .. } =
                &mut region.schedule.reduction
            else {
                panic!("the fixture carries a cooperative tile")
            };
            tile.phases[0].writes[0].staging = StagingId::new(1);
            tile.phases[1].reads[0].staging = StagingId::new(1);
        },
    );
    refusal("an overflowing participant product", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.coordinates.participants =
            ParticipantSpace::new(&[u64::MAX, 2]).expect("rank two is within the bound");
    });
    refusal("a producing span with an extra rank dimension", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[0].writes[0].span =
            StagedSpan::new(&[1, 0], 0, 1).expect("rank two is within the bound");
    });
    refusal("a consuming span with an extra rank dimension", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[1].reads[0].span =
            StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
    });
    refusal("a zero producing stride", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[0].writes[0].span =
            StagedSpan::new(&[0], 0, 1).expect("rank one is within the bound");
    });
    refusal("a nonzero consuming stride", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[1].reads[0].span =
            StagedSpan::new(&[1], 0, 3).expect("rank one is within the bound");
    });
    refusal("a two-slot producing write", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[0].writes[0].span.count = 2;
    });
    refusal("no visibility edge", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.phases[1].id = PhaseId::FIRST;
    });
    refusal("no point discharging the visibility edge", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.synchronization.clear();
    });
    refusal("two points discharging the visibility edge", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.synchronization.push(SynchronizationPoint {
            id: SyncPointId::new(1),
            ..tile.synchronization[0]
        });
    });
    refusal("an unsupported barrier spelling", &|region| {
        let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        tile.synchronization[0].subject.kind = SynchronizationKind::Atomic;
    });
    let mut region = multi_round_cooperative_region().region().clone();
    let ReductionTopology::CooperativeWorkgroup { tile, .. } = &mut region.schedule.reduction
    else {
        panic!("the fixture carries a cooperative tile")
    };
    tile.synchronization.truncate(1);
    checked.set(checked.get().saturating_add(1));
    assert_eq!(
        super::super::lower::cooperative_plan_shape_check(&region),
        Err(shape),
        "no point discharging the round anti-dependency"
    );
    refusal("an overflowing contributors-per-round product", &|region| {
        let ReductionTopology::CooperativeWorkgroup { coverage, .. } =
            &mut region.schedule.reduction
        else {
            panic!("the fixture carries a cooperative tile")
        };
        let ContributorCoverage::Exact(partition) = coverage else {
            panic!("the fixture is exact coverage")
        };
        partition.contributors_per_partition = u64::MAX;
    });
    assert_eq!(
        checked.get(),
        17,
        "the direct cooperative-plan refusal census changed"
    );
}
