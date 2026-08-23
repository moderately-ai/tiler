//! The operand-sharing cooperative-contraction fixture cluster of
//! `super::support`, split out purely to keep both files under the
//! size bound the split enforces: these fixtures are shared across
//! `contraction`, `intrinsic`, `proof`, and `tile` exactly as
//! `support`'s own fixtures are.

use super::super::{
    Access, AccessMode, BoundsProof, BoundsProofKind, ContractionAxisSource, ConvergenceEvidence,
    KernelSchedule, LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionTopology, RegionId,
    RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole, required_subject,
};
use super::support::{reassociating_numerical, tile_staging};
use crate::schedule::cooperative::{
    CooperativePhase, CooperativeTile, LocalCoordinateSource, LocalCoordinates, ParticipantRange,
    ParticipantSpace, StagedRead, StagedSpan, StagedWrite, WorkgroupStaging,
};
use crate::schedule::handles::{
    BoundsWitnessId, OwnershipWitnessId, PhaseId, StagingId, SyncPointId,
};
use crate::schedule::model::{ContributorOrder, LaunchPlan};
use crate::schedule::numerics::ArithmeticType;
use crate::schedule::synchronization::{SynchronizationPlacement, SynchronizationPoint};
use crate::shape::Shape;

/// One side of the measured kernel's square tile.
pub(super) const TILE_EXTENT: u64 = 16;

/// The tile's participants, which is also its launched workgroup width.
pub(super) const TILE_PARTICIPANTS: u64 = TILE_EXTENT * TILE_EXTENT;

pub(super) const OUTPUT_EXTENT: u64 = 32;

pub(super) const OUTPUT_BLOCK: u64 = 16;

pub(super) const CONTRACTED_EXTENT: u64 = 16;

pub(super) const CONTRACTED_TILE: u64 = 16;

pub(super) const OUTPUT_POSITIONS: u64 = OUTPUT_EXTENT * OUTPUT_EXTENT;

pub(super) fn operand_tile_fixture() -> CooperativeTile {
    let participants =
        ParticipantSpace::new(&[OUTPUT_BLOCK, OUTPUT_BLOCK]).expect("rank two is within the bound");
    let range = ParticipantRange {
        first: 0,
        count: TILE_PARTICIPANTS,
    };
    let a = StagingId::FIRST;
    let b = StagingId::new(1);
    let tile = CooperativeTile {
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalWorkgroupPosition,
            participants,
        },
        rounds: 1,
        staging: vec![
            tile_staging(TILE_PARTICIPANTS, PhaseId::new(1)),
            WorkgroupStaging {
                id: b,
                ..tile_staging(TILE_PARTICIPANTS, PhaseId::new(1))
            },
        ],
        phases: vec![
            CooperativePhase {
                id: PhaseId::FIRST,
                participation: range,
                writes: vec![
                    StagedWrite {
                        staging: a,
                        span: StagedSpan::new(&[OUTPUT_BLOCK, 1], 0, 1)
                            .expect("rank two is within the bound"),
                    },
                    StagedWrite {
                        staging: b,
                        span: StagedSpan::new(&[1, OUTPUT_BLOCK], 0, 1)
                            .expect("rank two is within the bound"),
                    },
                ],
                reads: Vec::new(),
            },
            CooperativePhase {
                id: PhaseId::new(1),
                participation: range,
                writes: Vec::new(),
                reads: vec![
                    StagedRead {
                        staging: a,
                        span: StagedSpan::new(&[OUTPUT_BLOCK, 0], 0, OUTPUT_BLOCK)
                            .expect("rank two is within the bound"),
                    },
                    StagedRead {
                        staging: b,
                        span: StagedSpan::new(&[0, OUTPUT_BLOCK], 0, OUTPUT_BLOCK)
                            .expect("rank two is within the bound"),
                    },
                ],
            },
        ],
        synchronization: Vec::new(),
        commit: range,
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

pub(super) fn operand_contraction_builder(
    admitted: &crate::schedule::ExactCooperativeContraction,
    tile: CooperativeTile,
) -> ScheduledRegionBuilder {
    let output = Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]);
    let contracted = Shape::from_dims([CONTRACTED_EXTENT]);
    let left = Shape::from_dims([OUTPUT_EXTENT, CONTRACTED_EXTENT]);
    let right = Shape::from_dims([OUTPUT_EXTENT, CONTRACTED_EXTENT]);
    let operand_map = |free_position, operand: Shape| LogicalAccess::ContractionOperand {
        operand_shape: operand,
        output_shape: output.clone(),
        contracted_shape: contracted.clone(),
        sources: vec![
            ContractionAxisSource::Output {
                position: free_position,
            },
            ContractionAxisSource::Contracted { position: 0 },
        ],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(7));
    builder.iteration_shape(output.clone()).unwrap();
    for (witness, map) in [
        (0, operand_map(0, left.clone())),
        (1, operand_map(1, right.clone())),
    ] {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map,
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: OUTPUT_EXTENT * CONTRACTED_EXTENT,
                },
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: OUTPUT_POSITIONS,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: OUTPUT_POSITIONS,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            },
            numerical: reassociating_numerical(),
        })
        .unwrap();
    let threads = u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32");
    builder
        .schedule(KernelSchedule {
            binding: admitted.binding.clone(),
            work_items: OUTPUT_POSITIONS,
            threads_per_workgroup: threads,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::CooperativeContraction {
                tile,
                contracted_shape: contracted,
                contracted_tile: admitted.contracted_tile.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: OUTPUT_POSITIONS,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder
}

pub(super) fn admitted_operand_tile() -> crate::schedule::ExactCooperativeContraction {
    crate::schedule::admit_exact_cooperative_contraction(
        &Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([CONTRACTED_EXTENT]),
        &Shape::from_dims([CONTRACTED_TILE]),
    )
    .expect("the exact 32×32 / 16 tile divides")
}
