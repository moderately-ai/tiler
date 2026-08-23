use super::super::{
    ConvergenceEvidence, CooperativeTileRule, MAX_COOPERATIVE_PARTICIPANTS, MAX_COOPERATIVE_PHASES,
    MAX_COOPERATIVE_STAGING_SLOTS, RegionId, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
    SynchronizationRule, cooperative_tile, encode_identity, phases_are_reached_by,
    required_subject,
};
use super::support::{
    cooperative_builder, cooperative_builder_parts, cooperative_builder_with,
    cooperative_rejection, cooperative_tile_fixture, cooperative_topology,
    cooperative_topology_with, float_rows, multi_round_builder, multi_round_tile_fixture,
    pointwise_builder, reassociating_numerical, round_boundary_point, round_perturbed,
    tile_staging,
};
use super::support_contraction::{
    TILE_EXTENT, TILE_PARTICIPANTS, admitted_operand_tile, operand_contraction_builder,
    operand_tile_fixture,
};
use crate::schedule::MAX_COOPERATIVE_PARTICIPANT_RANK;
use crate::schedule::cooperative::{
    AntiDependencyEdge, CooperativePhase, CooperativeTile, LocalCoordinateSource, LocalCoordinates,
    ParticipantRange, ParticipantSpace, StagedRead, StagedSpan, StagedWrite, VisibilityEdge,
};
use crate::schedule::handles::{PhaseId, StagingId, SyncPointId};
use crate::schedule::model::ContributorPartition;
use crate::schedule::numerics::NumericalPermission;
use crate::schedule::synchronization::{
    FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationPlacement,
    SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
};
use crate::shape::Shape;
use std::fmt::Write as _;

/// Canonical identity of the one-committer `[2, 6] -> [2]` cooperative fixture.
///
/// Captured against the bytes this tree encodes for that fixture so a
/// later payload move fails this pin rather than only the domain-separator
/// check. The new topology and binding tags must not appear here.
const ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000202000102000000000000000200000000000000020000000000000006000000000000000100000000000000020000000000000001000000010100000000000300020100000001010000000000000000000000020000000002001200000000000000020000000000000002000000000000000600000000000000010000000000000002000000000000000100000001010000000103001100000000000000020000000003000000000000000222000000000000000100000001017fc0000000000000000000000000001574696c65722e746573742e7374726963742d6633327fc000000101010201010101010101000000000000000600000003010000000035000000000000000300000000000000020100000000000000010000000000000003000000000000000100000000000000010000000001000000000000000300000000000000010000000000000002000000000000000000000000000000000000000300000000000000010000000000000000000000010000000000000001000000000000000000000000000000010000000000000000000000010000000000000000000000000000000300000000000000000000000000000001000000000000000000000001000000000000000000000000000000000000000000000003000000000000000100000000010202010002010000000000000001000000000000000000000000000000030100000000000000000000000000000001000000000000000100000001010301000100000000000000060000000301";

/// Applies one edit to the fixture tile and returns the resulting builder.
fn perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
    let mut tile = cooperative_tile_fixture();
    edit(&mut tile);
    cooperative_builder(tile)
}

/// One cooperative tile verifies, and states everything the handoff needs.
#[test]
fn one_cooperative_tile_verifies_and_derives_its_workgroup_storage() {
    let verified = cooperative_builder(cooperative_tile_fixture())
        .build()
        .expect("the cooperative fixture verifies");
    // Three `f32` slots, which is the only workgroup memory this tile asks
    // for and the value a feasibility authority composes against a target's
    // declared threadgroup memory.
    assert_eq!(verified.requirements().local_memory_bytes, 12);
    assert_eq!(verified.requirements().threads_per_workgroup, 3);
    // Six invocations over two output positions: the ownership proof counts
    // the positions, not the invocations.
    assert_eq!(verified.region().schedule.work_items, 6);
    let tile = cooperative_tile(&verified.region().schedule.reduction)
        .expect("the topology carries its tile");
    // The exact dependency a synchronization point would have to discharge.
    assert_eq!(
        tile.visibility_edges(),
        [VisibilityEdge {
            staging: StagingId::FIRST,
            produced_in: PhaseId::FIRST,
            consumed_in: PhaseId::new(1),
        }]
    );
    // The split it consumes, and only that split.
    assert_eq!(
        float_rows(&verified.requirements()).reassociation,
        NumericalPermission::Permitted
    );
    assert_eq!(
        float_rows(&verified.requirements()).permutation,
        NumericalPermission::Forbidden
    );
}

/// The verified tile derives exactly one atomic synchronization requirement.
///
/// One value, not one per point and not five independent dimensions: a
/// region requires one realization however many times it performs it, and a
/// target fact must equal the whole subject rather than any part of it.
#[test]
fn a_synchronized_tile_derives_one_atomic_realization_requirement() {
    let verified = cooperative_builder(cooperative_tile_fixture())
        .build()
        .expect("the cooperative fixture verifies");
    assert_eq!(
        verified.requirements().synchronization,
        Some(SynchronizationSubject {
            kind: SynchronizationKind::ControlBarrier,
            execution_scope: SynchronizationScope::Workgroup,
            visibility_scope: SynchronizationScope::Workgroup,
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: MemoryOrdering::AcquireRelease,
        })
    );
    // The point discharges the tile's one edge, and the derivation agrees
    // with the declaration rather than restating it.
    let tile = cooperative_tile(&verified.region().schedule.reduction)
        .expect("the topology carries its tile");
    let [edge] = tile.visibility_edges()[..] else {
        panic!("the fixture states exactly one handoff")
    };
    assert_eq!(tile.discharging_points(edge).len(), 1);
}

/// A schedule with no cooperative tile derives no synchronization at all.
///
/// Absence rather than a zero: nothing downstream may read this as "zero
/// barriers required", because there is no requirement to read.
#[test]
fn a_zero_synchronization_schedule_derives_no_requirement() {
    let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .expect("the pointwise fixture verifies");
    assert_eq!(verified.requirements().synchronization, None);
}

/// Every synchronization rule of the schedule verifier, driven once each.
///
/// Each row changes exactly one fact of the well-formed fixture, so the
/// diagnostic names the dimension the change touched. The subject rows are
/// what make the target fact atomic rather than composable: a schedule
/// cannot state four correct dimensions and one wrong one and be admitted.
#[test]
fn each_schedule_synchronization_rule_refuses_its_own_defect() {
    /// One named perturbation of the fixture tile and the rule it violates.
    type Perturbation = (
        &'static str,
        Box<dyn Fn(&mut CooperativeTile)>,
        SynchronizationRule,
    );
    let edits: Vec<Perturbation> = vec![
        (
            "an unadmitted operation kind",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].subject.kind = SynchronizationKind::Collective;
            }),
            SynchronizationRule::UnadmittedKind,
        ),
        (
            "a boundary that is not a program point",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].placement = SynchronizationPlacement::PhaseBoundary {
                    preceding: PhaseId::new(1),
                    following: PhaseId::new(2),
                };
            }),
            SynchronizationRule::Placement,
        ),
        (
            "a participant set narrower than the tile's",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].participants = ParticipantRange { first: 0, count: 2 };
            }),
            SynchronizationRule::ParticipantSet,
        ),
        (
            "an arrival scope the handoff does not require",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].subject.execution_scope = SynchronizationScope::Subgroup;
            }),
            SynchronizationRule::ExecutionScope,
        ),
        (
            "a publication scope the handoff does not require",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].subject.visibility_scope = SynchronizationScope::Device;
            }),
            SynchronizationRule::VisibilityScope,
        ),
        (
            "a fence over a memory domain the handoff does not cross",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].subject.fenced_spaces.device = true;
            }),
            SynchronizationRule::FencedSpaces,
        ),
        (
            "an ordering that establishes no happens-before edge",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].subject.ordering = MemoryOrdering::Relaxed;
            }),
            SynchronizationRule::Ordering,
        ),
        (
            "convergence asserted rather than derived",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].convergence = ConvergenceEvidence::CallerAsserted;
            }),
            SynchronizationRule::ConvergenceEvidence,
        ),
        (
            "no point at all for a declared handoff",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization.clear();
            }),
            SynchronizationRule::UndischargedVisibility,
        ),
        (
            "two points over one handoff",
            Box::new(|tile: &mut CooperativeTile| {
                let mut second = tile.synchronization[0];
                second.id = SyncPointId::new(1);
                tile.synchronization.push(second);
            }),
            SynchronizationRule::RedundantPoint,
        ),
        (
            "point ordinals that are not the dense ascending run",
            Box::new(|tile: &mut CooperativeTile| {
                tile.synchronization[0].id = SyncPointId::new(1);
            }),
            SynchronizationRule::PointSequence,
        ),
    ];
    for (name, edit, expected) in edits {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| edit(tile))),
            ScheduledRegionDiagnostic::Synchronization { rule: expected },
            "{name} was admitted"
        );
    }
}

/// The canonical tree tile is exactly the tile every rule above was driven
/// against.
///
/// The constructor exists so a strategy does not hand-assemble spans,
/// lifetimes, and a point subject, and this is what makes that safe: it is
/// compared against the fixture the whole perturbation table refuses defects
/// of, so the shape a planner emits is the shape those rules were proven on
/// rather than a second shape that merely also verifies.
#[test]
fn the_canonical_tree_tile_is_the_fixture_every_rule_was_driven_against() {
    assert_eq!(
        super::super::super::workgroup_tree_tile(3),
        Some(cooperative_tile_fixture())
    );
    // The point's subject is derived from the tile's own edges rather than
    // restated, so it cannot be constructed wrong.
    let tile =
        super::super::super::workgroup_tree_tile(3).expect("three participants are admitted");
    assert_eq!(
        tile.synchronization[0].subject,
        required_subject(&tile.visibility_edges()).expect("the tile carries one handoff")
    );
    // Below two participants the handoff is within one invocation, which the
    // synchronization authority refuses; the constructor declines rather
    // than emitting a tile that could only be rejected.
    assert_eq!(super::super::super::workgroup_tree_tile(1), None);
    assert_eq!(super::super::super::workgroup_tree_tile(0), None);
    assert_eq!(
        super::super::super::workgroup_tree_tile(MAX_COOPERATIVE_PARTICIPANTS + 1),
        None
    );
    // And a width the enumeration bound admits is built rather than refused,
    // so the bound check is not silently rejecting everything.
    assert!(super::super::super::workgroup_tree_tile(MAX_COOPERATIVE_PARTICIPANTS).is_some());
}

/// Every width the constructor admits verifies as a whole region.
///
/// The constructor states a dataflow; only the verifier decides whether the
/// dataflow, the split, and the launch agree. Driving several widths is what
/// stops the shape from being correct only at the one width the fixture pins.
#[test]
fn the_canonical_tree_tile_verifies_at_every_width_its_split_covers() {
    for (participants, contributors_per_partition) in [(2, 3), (3, 2), (6, 1)] {
        let split = ContributorPartition {
            partitions: participants,
            contributors_per_partition,
        };
        let tile = super::super::super::workgroup_tree_tile(participants)
            .expect("the width is within the enumeration bound");
        let verified = cooperative_builder_with(tile, split)
            .build()
            .unwrap_or_else(|error| {
                panic!(
                    "width {participants} was refused: {:?}",
                    error.diagnostics()
                )
            });
        assert_eq!(
            verified.requirements().local_memory_bytes,
            participants * 4,
            "one f32 slot per participant"
        );
        assert_eq!(
            u64::from(verified.requirements().threads_per_workgroup),
            participants
        );
    }
}

/// A single-participant tile's handoff is within one invocation.
///
/// The semantically redundant barrier this authority exists to eliminate:
/// program order already orders a value an invocation stages and reads back
/// itself, so a point there consumes a target authority for nothing.
#[test]
fn a_single_participant_tile_cannot_carry_a_synchronization_point() {
    let mut tile = cooperative_tile_fixture();
    tile.coordinates.participants =
        ParticipantSpace::new(&[1]).expect("rank one is within the bound");
    for phase in &mut tile.phases {
        phase.participation = ParticipantRange { first: 0, count: 1 };
    }
    tile.staging[0].slots = 1;
    tile.phases[1].reads[0].span.count = 1;
    tile.synchronization[0].participants = ParticipantRange { first: 0, count: 1 };
    let builder = cooperative_builder_with(
        tile,
        ContributorPartition {
            partitions: 1,
            contributors_per_partition: 6,
        },
    );
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::SingleParticipant,
        }
    );
}

/// The convergence derivation refuses a phase not every participant reaches.
///
/// Driven against the derivation directly, and the reason is stated rather
/// than hidden: the tile's own per-phase participation rule refuses a
/// non-uniform phase first, so this rule cannot fire end to end today. It is
/// re-derived here anyway rather than inherited, so a later relaxation of
/// that tile rule breaks this check instead of silently leaving every point
/// convergent by inheritance.
#[test]
fn the_convergence_derivation_refuses_a_phase_a_participant_skips() {
    let mut tile = cooperative_tile_fixture();
    let participants = ParticipantRange { first: 0, count: 3 };
    assert!(phases_are_reached_by(
        &tile,
        &[PhaseId::FIRST, PhaseId::new(1)],
        participants
    ));
    tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
    assert!(!phases_are_reached_by(
        &tile,
        &[PhaseId::FIRST, PhaseId::new(1)],
        participants
    ));
    // And a phase the tile does not have is not reached either, which is
    // what stops a placement naming one from reading as convergent.
    assert!(!phases_are_reached_by(
        &tile,
        &[PhaseId::new(7)],
        participants
    ));
}

/// A tile that rewrites its slots on a later round verifies.
///
/// The capability itself, and the two derivations it turns on: the rewrite
/// is no longer a staging conflict, and the anti-dependency it creates is
/// derived rather than declared. The storage is unchanged from the
/// single-round fixture, which is the point of reusing slots rather than
/// unrolling them into fresh ones.
#[test]
fn a_loop_carried_tile_rewrites_its_slots_and_verifies() {
    let tile = multi_round_tile_fixture();
    assert_eq!(
        tile.anti_dependency_edges(),
        vec![AntiDependencyEdge {
            staging: StagingId::FIRST,
            consumed_in: PhaseId::new(1),
            rewritten_in: PhaseId::FIRST,
        }]
    );
    // One discharger each, and not the same point: the phase boundary orders
    // the publication and the round boundary orders the rewrite.
    let [visibility] = tile.visibility_edges()[..] else {
        panic!("the fixture stages one handoff")
    };
    assert_eq!(
        tile.discharging_points(visibility)
            .iter()
            .map(|point| point.id)
            .collect::<Vec<_>>(),
        vec![SyncPointId::FIRST]
    );
    assert_eq!(
        tile.anti_discharging_points(tile.anti_dependency_edges()[0])
            .iter()
            .map(|point| point.id)
            .collect::<Vec<_>>(),
        vec![SyncPointId::new(1)]
    );

    let verified = multi_round_builder(tile)
        .build()
        .expect("the loop-carried fixture verifies");
    assert_eq!(verified.requirements().local_memory_bytes, 12);
}

/// A single-round tile derives no anti-dependency at all.
///
/// The absence is a claim rather than a missing derivation: no round follows
/// the only one, so nothing overwrites what the consuming phase read.
#[test]
fn a_single_round_tile_derives_no_anti_dependency() {
    assert!(
        cooperative_tile_fixture()
            .anti_dependency_edges()
            .is_empty()
    );
}

/// The rewrite needs its own point, and the handoff's does not serve.
#[test]
fn a_loop_carried_rewrite_with_no_round_boundary_is_refused() {
    assert_eq!(
        cooperative_rejection(round_perturbed(|tile| {
            tile.synchronization.truncate(1);
        })),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::UndischargedAntiDependency,
        }
    );
}

/// A second point over one anti-dependency is two spellings of one program.
#[test]
fn two_points_over_one_anti_dependency_are_refused() {
    assert_eq!(
        cooperative_rejection(round_perturbed(|tile| {
            tile.synchronization.push(SynchronizationPoint {
                id: SyncPointId::new(2),
                ..round_boundary_point()
            });
        })),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::RedundantPoint,
        }
    );
}

/// A round boundary on a tile with one round orders nothing.
///
/// The other side of `RedundantPoint` widening to both evidence classes: a
/// round boundary is not redundant *because* it discharges no visibility
/// edge, but it is redundant when there is no following round for it to
/// separate.
#[test]
fn a_round_boundary_without_a_following_round_is_redundant() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            // The single-round derivation, so the point reaches the
            // redundancy rule instead of failing the evidence class first.
            tile.synchronization.push(SynchronizationPoint {
                convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
                ..round_boundary_point()
            });
        })),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::RedundantPoint,
        }
    );
}

/// The convergence derivation must match the tile's round structure.
///
/// Both directions, because the rule is an equality and a one-sided check
/// would let the stronger claim stand unearned on a single-round tile.
#[test]
fn a_point_naming_the_wrong_convergence_derivation_is_refused() {
    let weak = round_perturbed(|tile| {
        tile.synchronization[0].convergence = ConvergenceEvidence::EveryParticipantReachesThePoint;
    });
    assert_eq!(
        cooperative_rejection(weak),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::ConvergenceEvidence,
        }
    );
    let unearned = perturbed(|tile| {
        tile.synchronization[0].convergence =
            ConvergenceEvidence::EveryParticipantExecutesEveryRound;
    });
    assert_eq!(
        cooperative_rejection(unearned),
        ScheduledRegionDiagnostic::Synchronization {
            rule: SynchronizationRule::ConvergenceEvidence,
        }
    );
}

/// Two writers to one slot inside one round are still a race.
///
/// The rule the round vocabulary relaxes is the one *between* rounds, and
/// this is what proves it did not relax the one inside them: no point sits
/// between two writes of the same phase, so nothing could separate them
/// however many rounds the tile declares.
#[test]
fn overlapping_staged_writes_inside_one_round_are_still_refused() {
    assert_eq!(
        cooperative_rejection(round_perturbed(|tile| {
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[0], 0, 1).expect("rank one is within the bound");
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingConflict,
        }
    );
}

/// Two participants writing one slot is a race the tile can state.
#[test]
fn overlapping_staged_writes_are_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[0], 0, 1).expect("rank one is within the bound");
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingConflict,
        }
    );
}

/// A read after the allocation's declared lifetime ends is rejected.
#[test]
fn a_staged_read_outside_the_declared_lifetime_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.staging[0].live_through = PhaseId::FIRST;
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingLifetime,
        }
    );
}

/// A slot inside the allocation that no participant writes is rejected.
#[test]
fn a_staging_slot_with_no_writer_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.staging[0].slots = 4;
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingCoverage,
        }
    );
}

/// A phase only some participants reach is rejected.
///
/// The rule a barrier depends on: a synchronization point inside a phase
/// the remaining participants skip is divergent, so the phase set has to be
/// uniform before any point can be placed in it.
#[test]
fn a_nonuniformly_reachable_phase_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::PhaseParticipation,
        }
    );
}

/// A malformed participant space is rejected, in each way it can be stated.
///
/// The three the space's constructor admits and the verifier refuses: a
/// rank-zero space, which names no participants at all; a zero extent, whose
/// product is zero so no invocation has a coordinate; and a product that
/// overflows `u64`, which no launch could hold. A *rank* above
/// `MAX_COOPERATIVE_PARTICIPANT_RANK` is deliberately not among them —
/// `ParticipantSpace::new` refuses it, so it cannot reach this rule, and the
/// separate assertion below is what keeps that claim from being an
/// assurance.
#[test]
fn an_invalid_participant_space_is_rejected() {
    let malformed = [Vec::new(), vec![0_u64], vec![3, 0], vec![u64::MAX, 2]];
    for extents in malformed {
        let space = ParticipantSpace::new(&extents).expect("every case is within the rank bound");
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.coordinates.participants = space;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::LocalCoordinates,
            },
            "extents {extents:?} were admitted as a participant space"
        );
    }
    assert_eq!(
        ParticipantSpace::new(&[2; MAX_COOPERATIVE_PARTICIPANT_RANK + 1]),
        None,
        "a rank above the governed bound was represented"
    );
    assert_eq!(
        StagedSpan::new(&[1; MAX_COOPERATIVE_PARTICIPANT_RANK + 1], 0, 1),
        None,
        "a stride vector above the governed bound was represented"
    );
}

/// The split the two-dimensional fixture covers its contributors with.
const TILED_SPLIT: ContributorPartition = ContributorPartition {
    partitions: TILE_PARTICIPANTS,
    contributors_per_partition: 2,
};

/// A verifying tile over a 16x16 participant space.
///
/// Its staged accesses are the rank-two spelling of the rank-one fixture's:
/// each participant writes its own slot, and every participant reads the
/// whole staged set. The `[1, 16]` write is the *transposed* form — the
/// exact profile `16 * (l % 16) + (l / 16)` that no single-term relation
/// over a linear coordinate expresses — so the fixture is a statement of the
/// thing this widening exists for rather than a rank-one tile wearing two
/// extents.
fn tiled_tile_fixture() -> CooperativeTile {
    let participants =
        ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT]).expect("rank two is within the bound");
    let range = ParticipantRange {
        first: 0,
        count: TILE_PARTICIPANTS,
    };
    let tile = CooperativeTile {
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalWorkgroupPosition,
            participants,
        },
        rounds: 1,
        staging: vec![tile_staging(TILE_PARTICIPANTS, PhaseId::new(1))],
        phases: vec![
            CooperativePhase {
                id: PhaseId::FIRST,
                participation: range,
                writes: vec![StagedWrite {
                    staging: StagingId::FIRST,
                    span: StagedSpan::new(&[1, TILE_EXTENT], 0, 1)
                        .expect("rank two is within the bound"),
                }],
                reads: Vec::new(),
            },
            CooperativePhase {
                id: PhaseId::new(1),
                participation: range,
                writes: Vec::new(),
                reads: vec![StagedRead {
                    staging: StagingId::FIRST,
                    span: StagedSpan::new(&[0, 0], 0, TILE_PARTICIPANTS)
                        .expect("rank two is within the bound"),
                }],
            },
        ],
        synchronization: Vec::new(),
        commit: ParticipantRange { first: 0, count: 1 },
    };
    let subject =
        required_subject(&tile.visibility_edges()).expect("the handoff states one subject");
    CooperativeTile {
        synchronization: vec![SynchronizationPoint {
            id: SyncPointId::FIRST,
            subject,
            placement: SynchronizationPlacement::PhaseBoundary {
                preceding: PhaseId::FIRST,
                following: PhaseId::new(1),
            },
            participants: range,
            convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
        }],
        ..tile
    }
}

/// Applies one edit to the two-dimensional fixture and builds it.
fn tiled_perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
    let mut tile = tiled_tile_fixture();
    edit(&mut tile);
    cooperative_builder_parts(
        TILED_SPLIT,
        TILE_PARTICIPANTS * TILED_SPLIT.contributors_per_partition,
        cooperative_topology_with(tile, TILED_SPLIT),
        reassociating_numerical(),
    )
}

/// A tile over a two-dimensional participant space verifies.
#[test]
fn a_two_dimensional_cooperative_tile_verifies() {
    let verified = tiled_perturbed(|_| {})
        .build()
        .expect("the two-dimensional fixture verifies");
    assert_eq!(
        verified.requirements().threads_per_workgroup,
        u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32")
    );
    // The extent product is the participant count and the launched width;
    // the shape is what the rank-one form could not state.
    let tile = cooperative_tile(&verified.region().schedule.reduction)
        .expect("the topology carries its tile");
    assert_eq!(tile.coordinates.participants.rank(), 2);
    assert_eq!(
        tile.coordinates.participants.extents(),
        [TILE_EXTENT, TILE_EXTENT]
    );
    assert_eq!(
        tile.coordinates.participants.participants(),
        Some(TILE_PARTICIPANTS)
    );
}

/// The measured 16x16 kernel's four staged accesses are all statable.
///
/// Each with a *contiguous* count, which is what `StagedSpan` addresses, and
/// each enumerating exactly the slots the kernel's own source indexes. The
/// two writes address one slot per participant and the two reads address
/// sixteen contiguous slots per participant, so the widened relation states
/// the tiling rather than encoding it.
///
/// The stride table is ADR 0097's, and this is the substitution that turns
/// it from arithmetic on paper into an observed enumeration.
#[test]
fn the_measured_tile_kernels_four_staged_accesses_are_all_statable() {
    let space =
        ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT]).expect("rank two is within the bound");
    let slots = |strides: &[u64], count: u64| {
        CooperativeTile::addressed_slots(
            space,
            StagedSpan::new(strides, 0, count).expect("rank two is within the bound"),
        )
        .expect("every address is representable")
    };

    // `a_tile[local_m * TILE + local_n]`: one slot per participant, and the
    // 256 participants cover the 256 slots exactly once.
    let a_write = slots(&[TILE_EXTENT, 1], 1);
    assert_eq!(a_write.len(), 256);
    let mut sorted = a_write.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 256);
    assert_eq!(a_write[0], 0);
    // Participant (0, 1) is linear index 1 and holds slot 1.
    assert_eq!(a_write[1], 1);
    // Participant (1, 0) is linear index 16 and holds slot 16.
    assert_eq!(a_write[16], 16);

    // `b_tile[local_n * TILE + local_m]`: the transpose, and the exact pair
    // of points that refutes every single-term relation over a linear
    // coordinate — `w(1) = 16` while `w(16) = 1`.
    let b_write = slots(&[1, TILE_EXTENT], 1);
    assert_eq!(b_write.len(), 256);
    assert_eq!(b_write[0], 0);
    assert_eq!(b_write[1], 16);
    assert_eq!(b_write[16], 1);
    let mut sorted = b_write.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 256, "the transposed write is a bijection");

    // `a_tile[local_m * TILE + kk]`, `kk` in `0..16`: sixteen contiguous
    // slots per participant, many-to-one in the column dimension.
    let a_read = slots(&[TILE_EXTENT, 0], TILE_EXTENT);
    assert_eq!(a_read.len(), 256 * 16);
    assert_eq!(&a_read[..16], &(0..16).collect::<Vec<_>>()[..]);
    // Participant (1, 0), linear index 16, reads the run beginning at 16.
    assert_eq!(
        &a_read[16 * 16..16 * 16 + 16],
        &(16..32).collect::<Vec<_>>()[..]
    );

    // `b_tile[local_n * TILE + kk]`: the transpose of the read, so
    // participant (0, 1) — linear index 1 — reads the run beginning at 16.
    let b_read = slots(&[0, TILE_EXTENT], TILE_EXTENT);
    assert_eq!(b_read.len(), 256 * 16);
    assert_eq!(&b_read[..16], &(0..16).collect::<Vec<_>>()[..]);
    assert_eq!(&b_read[16..32], &(16..32).collect::<Vec<_>>()[..]);
}

/// The occupancy map still refuses two writers reaching one slot.
///
/// ADR 0097's own case, watched failing rather than asserted: perturbing the
/// transposed write's strides from `[1, 16]` to `[16, 16]` sends participant
/// `(0, 1)` and participant `(1, 0)` both to slot 16, and no point separates
/// two writes inside one phase. Disjointness is keyed on *slots*, so
/// re-indexing the participant domain does not weaken it.
#[test]
fn two_writers_reaching_one_slot_in_one_round_are_still_refused() {
    let space =
        ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT]).expect("rank two is within the bound");
    let colliding =
        StagedSpan::new(&[TILE_EXTENT, TILE_EXTENT], 0, 1).expect("rank two is within the bound");
    // The collision itself, before the rule that refuses it: two distinct
    // participants, one slot.
    let addressed =
        CooperativeTile::addressed_slots(space, colliding).expect("every address is representable");
    assert_eq!(addressed[1], 16, "participant (0, 1) addresses slot 16");
    assert_eq!(addressed[16], 16, "participant (1, 0) addresses slot 16");

    // The allocation is widened to hold the perturbed span's furthest
    // address — `15 * 16 + 15 * 16` — so that the capacity rule, which is a
    // different refusal, does not fire first and hide the one under test.
    // Coverage would fail on this tile too, and does not get the chance:
    // the occupancy walk refuses the second writer before it finishes.
    assert_eq!(
        cooperative_rejection(tiled_perturbed(|tile| {
            tile.staging[0].slots = 15 * TILE_EXTENT + 15 * TILE_EXTENT + 1;
            tile.phases[0].writes[0].span = colliding;
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingConflict,
        }
    );
}

/// The workgroup-width equality is over the extent *product*, and still fires.
///
/// Perturbing the extents to a shape whose product is not the launched width
/// is refused; perturbing them to a *different shape with the same product*
/// is not, which is the whole content of the rule generalizing from a count
/// to a product. The launch plan carries no threadgroup shape to compare
/// against, and ADR 0097 records that as a stated deferral rather than an
/// omission — so this test pins what the rule does decide, not what a reader
/// might hope it does.
#[test]
fn the_workgroup_width_equality_is_over_the_extent_product() {
    for extents in [
        vec![TILE_EXTENT, TILE_EXTENT - 1],
        vec![TILE_EXTENT, TILE_EXTENT + 1],
        vec![TILE_EXTENT, 1],
    ] {
        let space = ParticipantSpace::new(&extents).expect("rank two is within the bound");
        assert_eq!(
            cooperative_rejection(tiled_perturbed(|tile| {
                tile.coordinates.participants = space;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ParticipantConvergence,
            },
            "extents {extents:?} were admitted against a launch of {TILE_PARTICIPANTS}"
        );
    }
    // And from the other side: holding the space and narrowing the launch
    // is the perturbation the rank-one fixture already used, so the rule is
    // reachable from either fact it relates.
    let mut builder = tiled_perturbed(|_| {});
    let schedule = builder
        .schedule
        .as_mut()
        .expect("the fixture sets a schedule");
    schedule.threads_per_workgroup = 255;
    schedule.launch.threads_per_workgroup = 255;
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::ParticipantConvergence,
        }
    );
    // A different arrangement of the same 256 participants passes the
    // equality, because the product is what it compares.
    tiled_perturbed(|tile| {
        tile.coordinates.participants =
            ParticipantSpace::new(&[4, 64]).expect("rank two is within the bound");
        tile.phases[0].writes[0].span =
            StagedSpan::new(&[64, 1], 0, 1).expect("rank two is within the bound");
    })
    .build()
    .expect("a 4x64 arrangement of the same participants verifies");
}

/// A staged span whose rank disagrees with the tile's is refused by name.
///
/// Both directions, because the two are different mistakes a producer makes
/// and neither is wrong on its own terms: a rank-two stride vector over a
/// rank-one space, and a rank-one one over a rank-two space. The read side
/// is checked separately from the write side, because a read's addressed set
/// is discarded and a rule that only fired on writes would admit the exact
/// silently-wrong broadcast this vocabulary exists to refuse.
#[test]
fn a_staged_span_whose_rank_disagrees_with_the_tile_is_refused() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[1, 0], 0, 1).expect("rank two is within the bound");
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::SpanRank,
        }
    );
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.phases[1].reads[0].span =
                StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::SpanRank,
        }
    );
    // And the same disagreement from the other side: a rank-two space whose
    // spans still state one stride each.
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.coordinates.participants =
                ParticipantSpace::new(&[3, 1]).expect("rank two is within the bound");
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::SpanRank,
        }
    );
}

/// Storage too small for the slots the participants address is rejected.
#[test]
fn insufficient_staging_storage_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.staging[0].slots = 2;
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagingCapacity,
        }
    );
}

/// A staged read in the phase that writes it has no producer to observe.
#[test]
fn a_staged_read_with_no_producing_phase_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            let read = tile.phases[1].reads.remove(0);
            tile.phases[0].reads.push(read);
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StagedProducer,
        }
    );
}

/// A tile that stages values nobody reads performs no cooperation.
#[test]
fn a_tile_with_no_visibility_edge_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.phases[1].reads.clear();
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::NoVisibilityEdge,
        }
    );
}

/// Participants must be the whole workgroup, or a barrier would diverge.
#[test]
fn a_participant_set_narrower_than_the_workgroup_is_rejected() {
    let mut builder = cooperative_builder(cooperative_tile_fixture());
    let schedule = builder.schedule.as_mut().unwrap();
    schedule.threads_per_workgroup = 6;
    schedule.launch.threads_per_workgroup = 6;
    assert_eq!(
        cooperative_rejection(builder),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::ParticipantConvergence,
        }
    );
}

/// More than one committing participant contradicts the ownership proof.
#[test]
fn a_tile_committing_from_every_participant_is_rejected() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.commit = ParticipantRange { first: 0, count: 3 };
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::CommitOwnership,
        }
    );
}

/// Every field of a tile separates canonical scheduled-region identity.
///
/// A dataflow that stages more, phases differently, or commits from another
/// participant is a different program, so a tile field left out of the
/// encoding would let two of these share identity.
#[test]
fn every_cooperative_tile_field_separates_scheduled_region_identity() {
    let baseline = cooperative_builder(cooperative_tile_fixture())
        .build()
        .unwrap()
        .region()
        .clone();
    let mut seen = vec![encode_identity(&baseline)];
    let variants: Vec<CooperativeTile> = vec![
        perturb_tile(|tile| tile.staging[0].slots = 4),
        perturb_tile(|tile| tile.staging[0].live_through = PhaseId::FIRST),
        perturb_tile(|tile| {
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[2], 0, 1).expect("rank one is within the bound");
        }),
        perturb_tile(|tile| tile.phases[0].writes[0].span.offset = 1),
        perturb_tile(|tile| tile.phases[1].reads[0].span.count = 2),
        perturb_tile(|tile| tile.commit = ParticipantRange { first: 2, count: 1 }),
        perturb_tile(|tile| {
            tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
        }),
        perturb_tile(|tile| {
            tile.coordinates.participants =
                ParticipantSpace::new(&[4]).expect("rank one is within the bound");
        }),
        // The participant *shape* separates identity too, not only the
        // count it determines: a tile whose 3 participants are arranged
        // `[3, 1]` states a different relation from one arranged `[3]`, and
        // the span ranks that go with each differ, so the two must not share
        // bytes even though both launch three invocations.
        perturb_tile(|tile| {
            tile.coordinates.participants =
                ParticipantSpace::new(&[3, 1]).expect("rank two is within the bound");
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[1, 0], 0, 1).expect("rank two is within the bound");
            tile.phases[1].reads[0].span =
                StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
        }),
        // The round count separates identity like every other tile field: a
        // schedule that rewrites its staging is a different program from one
        // that stages once, and the two must not share bytes.
        perturb_tile(|tile| tile.rounds = 2),
    ];
    for tile in variants {
        let mut candidate = baseline.clone();
        candidate.schedule.reduction = cooperative_topology(tile.clone());
        let identity = encode_identity(&candidate);
        assert!(
            !seen.contains(&identity),
            "{tile:?} collided with an earlier tile"
        );
        seen.push(identity);
    }
}

/// The enumeration bounds refuse a tile they could not decide.
///
/// Coverage and disjointness are decided by walking every addressed slot, so
/// the bounds are what keep that decision finite. Driven here rather than
/// assumed, because a limit nothing has been seen to trip is a limit that
/// might not be reached at all.
#[test]
fn a_tile_beyond_a_governed_enumeration_bound_is_rejected() {
    let overlong_phases = perturbed(|tile| {
        let template = tile.phases[1].clone();
        for ordinal in 2..=u32::try_from(MAX_COOPERATIVE_PHASES).unwrap() {
            tile.phases.push(CooperativePhase {
                id: PhaseId::new(ordinal),
                ..template.clone()
            });
        }
    });
    assert_eq!(
        cooperative_rejection(overlong_phases),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StructuralLimit,
        }
    );

    let oversized_storage = perturbed(|tile| {
        tile.staging[0].slots = MAX_COOPERATIVE_STAGING_SLOTS.saturating_add(1);
    });
    assert_eq!(
        cooperative_rejection(oversized_storage),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::StructuralLimit,
        }
    );
}

fn perturb_tile(edit: impl FnOnce(&mut CooperativeTile)) -> CooperativeTile {
    let mut tile = cooperative_tile_fixture();
    edit(&mut tile);
    tile
}

/// The one-committer tile still refuses every participant committing.
#[test]
fn the_one_committer_tile_still_refuses_every_participant_committing() {
    assert_eq!(
        cooperative_rejection(perturbed(|tile| {
            tile.commit = ParticipantRange { first: 0, count: 3 };
        })),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::CommitOwnership,
        }
    );
}

/// The operand-sharing tile refuses a one-committer range.
#[test]
fn the_operand_sharing_tile_refuses_a_single_committer() {
    let mut tile = operand_tile_fixture();
    tile.commit = ParticipantRange { first: 0, count: 1 };
    assert_eq!(
        cooperative_rejection(operand_contraction_builder(&admitted_operand_tile(), tile)),
        ScheduledRegionDiagnostic::CooperativeTile {
            rule: CooperativeTileRule::OperandTileCommit,
        }
    );
}

/// The existing one-committer fixture is re-pinned under the elementary
/// numerical dimensions.
#[test]
fn existing_one_committer_schedule_encodings_keep_their_bytes() {
    let verified = cooperative_builder(cooperative_tile_fixture())
        .build()
        .expect("the one-committer fixture still verifies");
    let bytes = verified.canonical_identity().as_bytes();
    assert!(
        bytes.starts_with(b"tiler.schedule.v7\0"),
        "the schedule domain must carry the elementary-dimension step"
    );
    assert!(
        bytes.contains(&0x35),
        "the one-committer topology tag must still appear"
    );
    assert!(
        !bytes[18..].contains(&0x37),
        "the new topology tag must not appear in an old region's payload; \
         the separator is excluded because `v7` spells the byte 0x37"
    );
    // Binding tag 0x01 sits at a known offset after the numerical payload;
    // the new 0x02 binding is an appended alternative, so an old region
    // that still carries GlobalLinearInvocation cannot encode it.
    let mut hex = String::new();
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    // Pin of the one-committer `[2, 6] -> [2]` cooperative fixture at
    // `4333df31`. A payload move here is a domain step nobody authorized.
    assert_eq!(hex, ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX);
}
