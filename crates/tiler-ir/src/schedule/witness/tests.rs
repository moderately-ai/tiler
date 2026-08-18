//! Evidence for the realization witness vocabulary.
//!
//! Every fixture below is a region the intrinsic verifier admitted, so nothing
//! here aggregates a witness over a plan the builder would have refused.

use super::{
    RealizationWitness, UnevaluableRealization, UnpinnedFreedomSite, UnrecordedFoldContraction,
};
use crate::schedule::ScheduledRegionBuilder;
use crate::schedule::cooperative::{
    ContributorArrival, CooperativePhase, CooperativeTile, LocalCoordinateSource, LocalCoordinates,
    ParticipantRange, ParticipantSpace, StagedElement, StagedRead, StagedSpan, StagedWrite,
    WorkgroupStaging,
};
use crate::schedule::handles::{
    AccessOrdinal, BoundsWitnessId, OwnershipWitnessId, PhaseId, RegionId, StagingId, SyncPointId,
};
use crate::schedule::model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, ContractionAxisSource, ContributorCoverage,
    ContributorOrder, ContributorPartition, ExecutionBinding, KernelSchedule, LaunchPlan,
    LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionPass, ReductionTopology,
    ScalarProgram, TailPolicy, TensorRole, VerifiedScheduledRegion, partial_reduction_axis,
    partial_reduction_shape,
};
use crate::schedule::numerics::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, NumericalPermission,
    NumericalRealization, SubnormalMode,
};
use crate::schedule::pointwise::{PointwiseF32Expression, PointwiseF32ExpressionBuilder};
use crate::schedule::pointwise_bf16::{PointwiseBf16Expression, PointwiseBf16ExpressionBuilder};
use crate::schedule::synchronization::{
    ConvergenceEvidence, FencedSpaces, MemoryOrdering, SynchronizationKind,
    SynchronizationPlacement, SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
};
use crate::shape::{Axis, Shape};

// ---- Numerical realizations -------------------------------------------------

fn strict_numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.witness.strict-f32",
        0x7fc0_0000,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

/// Reassociation permitted and nothing else, so an admission is evidence about
/// that dimension alone.
fn reassociating_numerical() -> NumericalRealization {
    NumericalRealization {
        reassociation: NumericalPermission::Permitted,
        ..strict_numerical()
    }
}

/// Contraction permitted and nothing else, for the same reason.
fn contracting_numerical() -> NumericalRealization {
    NumericalRealization {
        contraction: NumericalPermission::Permitted,
        ..strict_numerical()
    }
}

fn strict_bf16_numerical() -> NumericalRealization {
    NumericalRealization {
        profile_key: "tiler.test.witness.strict-bf16",
        canonical_arithmetic_nan_bits: u32::from(
            crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS,
        ),
        ..strict_numerical()
    }
}

// ---- Pointwise regions ------------------------------------------------------

/// `x * 2.0 + 1.0`: one input, and one multiply an addition consumes.
fn scale_bias_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(2.0_f32.to_bits()).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(1.0_f32.to_bits()).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// `x + 1.0`: one input, and no multiply anywhere.
fn bias_only_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let bias = expression.constant(1.0_f32.to_bits()).unwrap();
    let root = expression.add(input, bias).unwrap();
    expression.build(root).unwrap()
}

fn bf16_scale_bias_expression() -> PointwiseBf16Expression {
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(0x4000).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(0x3f80).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// One dense read, one owning write, over a `[2, 3]` domain.
fn pointwise_region(
    program: ScalarProgram,
    numerical: NumericalRealization,
) -> ScheduledRegionBuilder {
    let elements = 6;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Intermediate)] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder.scalar_program(program).unwrap();
    builder.numerical(numerical).unwrap();
    builder
        .schedule(linear_schedule(elements, ReductionTopology::None))
        .unwrap();
    builder
}

fn linear_schedule(work_items: u64, reduction: ReductionTopology) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: OwnershipWitnessId::new(0),
        reduction,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

// ---- Serial folds -----------------------------------------------------------

const FOLD_AXES: [Axis; 1] = [Axis::new(1)];

/// A `[2, 6] -> [2]` serial fold over the first declared input tensor.
fn serial_region(
    program: ScalarProgram,
    numerical: NumericalRealization,
) -> ScheduledRegionBuilder {
    let permits_reassociation = numerical.permits_reassociation();
    let permits_permutation = numerical.permits_permutation();
    let input = Shape::from_dims([2, 6]);
    let output = Shape::from_dims([2]);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(1));
    builder.iteration_shape(output.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output,
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    builder.scalar_program(program).unwrap();
    builder.numerical(numerical).unwrap();
    builder
        .schedule(linear_schedule(
            2,
            ReductionTopology::Serial {
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation,
                permits_permutation,
            },
        ))
        .unwrap();
    builder
}

fn bare_sum() -> ScalarProgram {
    ScalarProgram::StrictSerialSum {
        axes: FOLD_AXES.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    }
}

fn squared_sum() -> ScalarProgram {
    ScalarProgram::SquaredSerialSum {
        axes: FOLD_AXES.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    }
}

fn scale_bias_sum() -> ScalarProgram {
    ScalarProgram::FusedMultiplyAddSerialSum {
        scale_bits: 2.0_f32.to_bits(),
        bias_bits: 1.0_f32.to_bits(),
        axes: FOLD_AXES.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
        // The mirror the enumeration names, pinned `false` by the intrinsic
        // verifier whatever the contract resolved.
        contraction: false,
    }
}

/// `Rsqrt(a / 6 + 1e-6)` over the folded value, the shipped epilogue's shape.
fn scale_epilogue() -> PointwiseF32Expression {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let total = builder.input(AccessOrdinal::FIRST).unwrap();
    let extent = builder.constant(6.0_f32.to_bits()).unwrap();
    let mean = builder.divide(total, extent).unwrap();
    let bias = builder.constant(1.0e-6_f32.to_bits()).unwrap();
    let biased = builder.add(mean, bias).unwrap();
    let root = builder.rsqrt(biased).unwrap();
    builder.build(root).unwrap()
}

fn squared_sum_with_epilogue() -> ScalarProgram {
    ScalarProgram::SquaredSerialSumThenEpilogue {
        axes: FOLD_AXES.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
        epilogue: scale_epilogue(),
    }
}

// ---- The multi-dispatch split ----------------------------------------------

const SPLIT: ContributorPartition = ContributorPartition {
    partitions: 3,
    contributors_per_partition: 2,
};

fn partial_pass_region(accumulation: ArithmeticType) -> ScheduledRegionBuilder {
    let partial_elements = 2 * SPLIT.partitions;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(2));
    builder
        .iteration_shape(
            partial_reduction_shape(&Shape::from_dims([2]), SPLIT)
                .expect("a rank-two partial shape is within the governed bound"),
        )
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: Shape::from_dims([2, 6]),
                output_shape: Shape::from_dims([2]),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: Shape::from_dims([2, 6]),
                output_shape: Shape::from_dims([2]),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: partial_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: partial_elements,
            },
        })
        .unwrap();
    builder.scalar_program(bare_sum()).unwrap();
    builder.numerical(reassociating_numerical()).unwrap();
    builder
        .schedule(linear_schedule(
            partial_elements,
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(SPLIT),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation,
                permits_reassociation: true,
                permits_permutation: false,
            },
        ))
        .unwrap();
    builder
}

fn final_pass_region() -> ScheduledRegionBuilder {
    let partial_shape = partial_reduction_shape(&Shape::from_dims([2]), SPLIT)
        .expect("a rank-two partial shape is within the governed bound");
    let axes = vec![partial_reduction_axis(&Shape::from_dims([2])).expect("rank one fits u32")];
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(3));
    builder.iteration_shape(Shape::from_dims([2])).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: partial_shape.clone(),
                output_shape: Shape::from_dims([2]),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: partial_shape,
                output_shape: Shape::from_dims([2]),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        })
        .unwrap();
    builder.numerical(reassociating_numerical()).unwrap();
    builder
        .schedule(linear_schedule(
            2,
            ReductionTopology::MultiPass {
                pass: ReductionPass::Final,
                coverage: ContributorCoverage::Exact(SPLIT),
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
        ))
        .unwrap();
    builder
}

// ---- The cooperative tile ---------------------------------------------------

fn tile_point() -> SynchronizationPoint {
    SynchronizationPoint {
        id: SyncPointId::FIRST,
        subject: SynchronizationSubject {
            kind: SynchronizationKind::ControlBarrier,
            execution_scope: SynchronizationScope::Workgroup,
            visibility_scope: SynchronizationScope::Workgroup,
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: MemoryOrdering::AcquireRelease,
        },
        placement: SynchronizationPlacement::PhaseBoundary {
            preceding: PhaseId::FIRST,
            following: PhaseId::new(1),
        },
        participants: ParticipantRange { first: 0, count: 3 },
        convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
    }
}

fn single_round_tile() -> CooperativeTile {
    CooperativeTile {
        synchronization: vec![tile_point()],
        rounds: 1,
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalLinearInvocation,
            participants: ParticipantSpace::new(&[3]).expect("rank one is within the bound"),
        },
        staging: vec![WorkgroupStaging {
            id: StagingId::FIRST,
            element: StagedElement::F32,
            slots: 3,
            live_from: PhaseId::FIRST,
            live_through: PhaseId::new(1),
        }],
        phases: vec![
            CooperativePhase {
                id: PhaseId::FIRST,
                participation: ParticipantRange { first: 0, count: 3 },
                writes: vec![StagedWrite {
                    staging: StagingId::FIRST,
                    span: StagedSpan::new(&[1], 0, 1).expect("rank one is within the bound"),
                }],
                reads: Vec::new(),
            },
            CooperativePhase {
                id: PhaseId::new(1),
                participation: ParticipantRange { first: 0, count: 3 },
                writes: Vec::new(),
                reads: vec![StagedRead {
                    staging: StagingId::FIRST,
                    span: StagedSpan::new(&[0], 0, 3).expect("rank one is within the bound"),
                }],
            },
        ],
        commit: ParticipantRange { first: 0, count: 1 },
    }
}

/// The same tile run twice, with the round boundary its rewrite requires.
fn loop_carried_tile() -> CooperativeTile {
    CooperativeTile {
        rounds: 2,
        synchronization: vec![
            SynchronizationPoint {
                convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
                ..tile_point()
            },
            SynchronizationPoint {
                id: SyncPointId::new(1),
                placement: SynchronizationPlacement::RoundBoundary,
                convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
                ..tile_point()
            },
        ],
        ..single_round_tile()
    }
}

/// The `[2, contracted] -> [2]` reduction realized on one workgroup of three.
fn cooperative_region(
    tile: CooperativeTile,
    split: ContributorPartition,
    contracted: u64,
    numerical: NumericalRealization,
) -> ScheduledRegionBuilder {
    let permits_reassociation = numerical.permits_reassociation();
    let permits_permutation = numerical.permits_permutation();
    let participants = split.partitions;
    let work_items = 2 * participants;
    let threads = u32::try_from(participants).expect("the fixture width fits u32");
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(4));
    builder
        .iteration_shape(
            partial_reduction_shape(&Shape::from_dims([2]), split)
                .expect("a rank-two cooperative domain is within the governed bound"),
        )
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: Shape::from_dims([2, contracted]),
                output_shape: Shape::from_dims([2]),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: Shape::from_dims([2, contracted]),
                output_shape: Shape::from_dims([2]),
                axes: FOLD_AXES.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    builder.scalar_program(bare_sum()).unwrap();
    builder.numerical(numerical).unwrap();
    builder
        .schedule(KernelSchedule {
            threads_per_workgroup: threads,
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(
                work_items,
                ReductionTopology::CooperativeWorkgroup {
                    coverage: ContributorCoverage::Exact(split),
                    tile,
                    axes: FOLD_AXES.to_vec(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation,
                    permits_permutation,
                    arrival: ContributorArrival::AscendingParticipant,
                },
            )
        })
        .unwrap();
    builder
}

// ---- The contraction --------------------------------------------------------

/// `ik,kj->ij` over `i = 2`, `k = 4`, `j = 3`.
fn contraction_region(numerical: NumericalRealization) -> ScheduledRegionBuilder {
    let permits_reassociation = numerical.permits_reassociation();
    let permits_permutation = numerical.permits_permutation();
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([4]);
    let left = Shape::from_dims([2, 4]);
    let right = Shape::from_dims([4, 3]);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(5));
    builder.iteration_shape(output.clone()).unwrap();
    for (operand, sources, witness) in [
        (
            left.clone(),
            vec![
                ContractionAxisSource::Output { position: 0 },
                ContractionAxisSource::Contracted { position: 0 },
            ],
            0_u32,
        ),
        (
            right.clone(),
            vec![
                ContractionAxisSource::Contracted { position: 0 },
                ContractionAxisSource::Output { position: 1 },
            ],
            1,
        ),
    ] {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand,
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources,
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
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
    for (witness, elements) in [(0_u32, 8_u64), (1, 12)] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 6 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictTensorContraction {
            contracted_shape: contracted.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
        })
        .unwrap();
    builder.numerical(numerical).unwrap();
    builder
        .schedule(linear_schedule(
            6,
            ReductionTopology::Contraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation,
                permits_permutation,
            },
        ))
        .unwrap();
    builder
}

fn verified(builder: ScheduledRegionBuilder) -> VerifiedScheduledRegion {
    builder
        .build()
        .expect("every witness fixture is a region the intrinsic verifier admits")
}

// ---- Aggregation ------------------------------------------------------------

/// Every topology hands back exactly the site fields it states, and no others.
///
/// The population is named and counted, so a loop that ranged over nothing
/// cannot pass: six regions, one per topology this vocabulary states plus both
/// passes of the split.
#[test]
fn every_topology_aggregates_the_site_fields_it_states() {
    let pointwise = verified(pointwise_region(
        ScalarProgram::PointwiseF32(scale_bias_expression()),
        strict_numerical(),
    ));
    let serial = verified(serial_region(bare_sum(), strict_numerical()));
    let partial = verified(partial_pass_region(ArithmeticType::F32));
    let combine = verified(final_pass_region());
    let tile = verified(cooperative_region(
        single_round_tile(),
        SPLIT,
        6,
        reassociating_numerical(),
    ));
    let contraction = verified(contraction_region(strict_numerical()));
    let population = [&pointwise, &serial, &partial, &combine, &tile, &contraction];
    assert_eq!(population.len(), 6, "the aggregated population changed");
    // Every region hands back its own declared realization, which is the two
    // subnormal sites and the permission vector the refusals read.
    for region in population {
        assert_eq!(
            RealizationWitness::of(region).realization(),
            &region.region().index.numerical,
        );
    }

    let witness = RealizationWitness::of(&pointwise);
    assert_eq!(witness.order(), None);
    assert!(witness.reduced_axes().is_empty());
    assert_eq!(witness.contracted_shape(), None);
    assert_eq!(witness.contributor_partition(), None);
    assert_eq!(witness.pass(), None);
    assert_eq!(witness.accumulation(), ArithmeticType::F32);
    assert_eq!(witness.arrival(), None);
    assert_eq!(witness.rounds(), None);
    assert_eq!(witness.pointwise_f32(), Some(&scale_bias_expression()));
    assert_eq!(witness.fold_epilogue(), None);

    let witness = RealizationWitness::of(&serial);
    assert_eq!(
        witness.order(),
        Some(ContributorOrder::OriginalAxisLexicographic)
    );
    assert_eq!(witness.reduced_axes(), FOLD_AXES);
    assert_eq!(witness.contributor_partition(), None);
    assert_eq!(witness.pass(), None);
    // Nothing declares a width here, so the fold combines at the width its own
    // scalar program's arithmetic is in.
    assert_eq!(witness.accumulation(), ArithmeticType::F32);
    assert_eq!(witness.pointwise_f32(), None);

    let witness = RealizationWitness::of(&partial);
    assert_eq!(witness.contributor_partition(), Some(SPLIT));
    assert_eq!(witness.pass(), Some(ReductionPass::Partial));
    assert_eq!(witness.accumulation(), ArithmeticType::F32);
    assert_eq!(witness.arrival(), None);
    assert_eq!(witness.rounds(), None);

    let witness = RealizationWitness::of(&combine);
    assert_eq!(witness.contributor_partition(), Some(SPLIT));
    assert_eq!(witness.pass(), Some(ReductionPass::Final));
    // The two passes of one split agree on every other field and differ here,
    // which is why the accessor exists at all.
    assert_ne!(
        RealizationWitness::of(&partial).pass(),
        RealizationWitness::of(&combine).pass(),
    );

    let witness = RealizationWitness::of(&tile);
    assert_eq!(witness.contributor_partition(), Some(SPLIT));
    assert_eq!(witness.pass(), None);
    assert_eq!(
        witness.arrival(),
        Some(ContributorArrival::AscendingParticipant)
    );
    assert_eq!(witness.rounds(), Some(1));
    assert_eq!(witness.reduced_axes(), FOLD_AXES);

    let witness = RealizationWitness::of(&contraction);
    assert_eq!(witness.contracted_shape(), Some(&Shape::from_dims([4])));
    assert!(
        witness.reduced_axes().is_empty(),
        "a contraction states a contracted shape rather than an axis set"
    );
    assert_eq!(witness.contributor_partition(), None);
    assert_eq!(
        witness.order(),
        Some(ContributorOrder::OriginalAxisLexicographic)
    );
}

/// A fold epilogue is aggregated at its own accessor, not the program's.
#[test]
fn the_fold_epilogue_is_a_site_of_its_own() {
    let region = verified(serial_region(
        squared_sum_with_epilogue(),
        strict_numerical(),
    ));
    let witness = RealizationWitness::of(&region);
    assert_eq!(witness.fold_epilogue(), Some(&scale_epilogue()));
    assert_eq!(
        witness.pointwise_f32(),
        None,
        "the region's scalar program is a fold, not a pointwise expression",
    );
}

/// The declared accumulation is the region's own element width, always.
///
/// Site 4.8's spend population is empty at this base, and this is the evidence:
/// the intrinsic verifier refuses a narrower declared width, so no verified
/// region can carry an accumulation the reference cannot answer for.
#[test]
fn a_split_cannot_declare_an_accumulation_the_region_does_not_perform() {
    let admitted = verified(partial_pass_region(ArithmeticType::F32));
    assert_eq!(
        RealizationWitness::of(&admitted).accumulation(),
        ArithmeticType::F32
    );
    for narrowed in [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F64,
    ] {
        assert!(
            partial_pass_region(narrowed).build().is_err(),
            "a {narrowed:?} accumulation over an f32 fold was admitted",
        );
    }
}

// ---- Refusals ---------------------------------------------------------------

/// Each unrecorded fold adjacency is named, and a fold with no multiply is not.
#[test]
fn every_unrecorded_fold_contraction_is_named_by_its_adjacency() {
    let cases = [
        (
            verified(serial_region(squared_sum(), contracting_numerical())),
            UnrecordedFoldContraction::SquaredContributor,
        ),
        (
            verified(serial_region(
                squared_sum_with_epilogue(),
                contracting_numerical(),
            )),
            UnrecordedFoldContraction::SquaredContributor,
        ),
        (
            verified(serial_region(scale_bias_sum(), contracting_numerical())),
            UnrecordedFoldContraction::ScaleBiasContributor,
        ),
        (
            verified(contraction_region(contracting_numerical())),
            UnrecordedFoldContraction::ContractedProduct,
        ),
    ];
    assert_eq!(cases.len(), 4, "the adjacency population changed");
    let mut named = Vec::new();
    for (region, operation) in &cases {
        assert_eq!(
            RealizationWitness::of(region).unpinned_freedom_site(),
            Some(UnpinnedFreedomSite::ContractionUnrecorded {
                operation: *operation
            }),
        );
        named.push(*operation);
    }
    named.sort_unstable_by_key(|operation| format!("{operation:?}"));
    named.dedup();
    assert_eq!(named.len(), 3, "every adjacency variant must be watched");

    // The same folds under a contract that forbids contraction leave nothing
    // open: a freedom site is a place a *granted* permission could be spent.
    for program in [squared_sum(), scale_bias_sum()] {
        let region = verified(serial_region(program, strict_numerical()));
        assert_eq!(
            RealizationWitness::of(&region).unpinned_freedom_site(),
            None
        );
    }
    // And a fold whose step has no multiply has nothing for the permission to
    // reach, so it stays determined under a permitting contract.
    let sum = verified(serial_region(bare_sum(), contracting_numerical()));
    assert_eq!(RealizationWitness::of(&sum).unpinned_freedom_site(), None);
}

/// An adjacency the plan states exactly leaves only the backend's order open.
#[test]
fn a_stated_expression_adjacency_leaves_the_backend_order_undeclared() {
    let fused = verified(pointwise_region(
        ScalarProgram::PointwiseF32(scale_bias_expression()),
        contracting_numerical(),
    ));
    assert_eq!(
        RealizationWitness::of(&fused).unpinned_freedom_site(),
        Some(UnpinnedFreedomSite::BackendOrderUndeclared),
    );
    let bf16 = verified(pointwise_region(
        ScalarProgram::PointwiseBf16(bf16_scale_bias_expression()),
        NumericalRealization {
            contraction: NumericalPermission::Permitted,
            ..strict_bf16_numerical()
        },
    ));
    assert_eq!(
        RealizationWitness::of(&bf16).unpinned_freedom_site(),
        Some(UnpinnedFreedomSite::BackendOrderUndeclared),
    );

    // The same region with no multiply for an addition to consume states no
    // adjacency, so the dropped `-ffp-contract=off` has nothing to change.
    let unfused = verified(pointwise_region(
        ScalarProgram::PointwiseF32(bias_only_expression()),
        contracting_numerical(),
    ));
    assert_eq!(
        RealizationWitness::of(&unfused).unpinned_freedom_site(),
        None
    );
}

/// Each unevaluable construct is named, and each holds for its own reason.
#[test]
fn every_unevaluable_realization_is_named_by_its_construct() {
    let cases = [
        (
            verified(pointwise_region(
                ScalarProgram::PointwiseF32(scale_bias_expression()),
                reassociating_numerical(),
            )),
            UnevaluableRealization::PointwiseExpression,
        ),
        (
            verified(pointwise_region(
                ScalarProgram::PointwiseBf16(bf16_scale_bias_expression()),
                NumericalRealization {
                    reassociation: NumericalPermission::Permitted,
                    ..strict_bf16_numerical()
                },
            )),
            UnevaluableRealization::PointwiseExpression,
        ),
        (
            verified(serial_region(
                squared_sum_with_epilogue(),
                reassociating_numerical(),
            )),
            UnevaluableRealization::FoldEpilogueExpression,
        ),
        (
            verified(cooperative_region(
                loop_carried_tile(),
                ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 1,
                },
                6,
                reassociating_numerical(),
            )),
            UnevaluableRealization::LoopCarriedCooperativeTile { rounds: 2 },
        ),
    ];
    assert_eq!(cases.len(), 4, "the unevaluable population changed");
    let mut named = Vec::new();
    for (region, reason) in &cases {
        assert_eq!(
            RealizationWitness::of(region).unpinned_freedom_site(),
            Some(UnpinnedFreedomSite::RealizationNotEvaluable { reason: *reason }),
        );
        named.push(*reason);
    }
    named.sort_unstable_by_key(|reason| format!("{reason:?}"));
    named.dedup();
    assert_eq!(named.len(), 3, "every unevaluable variant must be watched");

    // The two expression sites are permission-gated and the tile is not: under a
    // forbidding contract the minted expression is a total function of the
    // caller's own program, while a multi-round tile's contributor order is
    // unstatable whatever the contract says.
    for program in [
        ScalarProgram::PointwiseF32(scale_bias_expression()),
        ScalarProgram::PointwiseF32(bias_only_expression()),
    ] {
        let region = verified(pointwise_region(program, strict_numerical()));
        assert_eq!(
            RealizationWitness::of(&region).unpinned_freedom_site(),
            None
        );
    }
    let single_round = verified(cooperative_region(
        single_round_tile(),
        SPLIT,
        6,
        reassociating_numerical(),
    ));
    assert_eq!(
        RealizationWitness::of(&single_round).unpinned_freedom_site(),
        None,
        "a single-round tile is the flat blocked split the reference states",
    );
}

/// The refusal names the region-specific site ahead of the target-wide one.
#[test]
fn the_first_named_site_is_the_one_that_is_about_this_region() {
    // Both permissions granted over a fold with an adjacency: the fold's own
    // unrecorded choice leads.
    let both = verified(serial_region(
        squared_sum(),
        NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..contracting_numerical()
        },
    ));
    assert_eq!(
        RealizationWitness::of(&both).unpinned_freedom_site(),
        Some(UnpinnedFreedomSite::ContractionUnrecorded {
            operation: UnrecordedFoldContraction::SquaredContributor
        }),
    );
    // Both permissions granted over a pointwise expression: the unevaluable
    // grouping leads the backend's undeclared order.
    let expression = verified(pointwise_region(
        ScalarProgram::PointwiseF32(scale_bias_expression()),
        NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..contracting_numerical()
        },
    ));
    assert_eq!(
        RealizationWitness::of(&expression).unpinned_freedom_site(),
        Some(UnpinnedFreedomSite::RealizationNotEvaluable {
            reason: UnevaluableRealization::PointwiseExpression
        }),
    );
}

// ---- The canonical-form claim ----------------------------------------------

/// The two mitigations the record names do hold, at the whole-region level.
///
/// [The freedom-sites record](../../../../../docs/research/reference/plan-freedom-sites.md)
/// Part 5 states that one leaf per access ordinal, shared on repeat request, plus
/// a deterministic root-first-derived topological order, make the canonical form
/// a function of the program rather than of the spelling — and states the claim
/// **untested**. This tests it at the extent those two mitigations cover: two
/// spellings that differ in the order independent subtrees were minted and in how
/// many times each input leaf was asked for produce one witness and one canonical
/// region identity.
#[test]
fn the_two_named_canonicalization_mitigations_hold() {
    fn spelled_left() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let three = builder.constant(3.0_f32.to_bits()).unwrap();
        let scaled = builder.multiply(input.clone(), two).unwrap();
        let biased = builder.add(input, three).unwrap();
        let root = builder.add(scaled, biased).unwrap();
        builder.build(root).unwrap()
    }
    fn spelled_right() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        // The independent subtrees are minted in the other order, and the input
        // leaf is asked for once per use rather than cloned.
        let three = builder.constant(3.0_f32.to_bits()).unwrap();
        let first = builder.input(AccessOrdinal::FIRST).unwrap();
        let biased = builder.add(first, three).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let again = builder.input(AccessOrdinal::FIRST).unwrap();
        let scaled = builder.multiply(again, two).unwrap();
        let root = builder.add(scaled, biased).unwrap();
        builder.build(root).unwrap()
    }
    assert_eq!(spelled_left(), spelled_right());

    let left = verified(pointwise_region(
        ScalarProgram::PointwiseF32(spelled_left()),
        strict_numerical(),
    ));
    let right = verified(pointwise_region(
        ScalarProgram::PointwiseF32(spelled_right()),
        strict_numerical(),
    ));
    assert_eq!(
        RealizationWitness::of(&left).pointwise_f32(),
        RealizationWitness::of(&right).pointwise_f32(),
    );
    assert_eq!(left.canonical_identity(), right.canonical_identity());
}

/// A duplicated constant is a spelling the canonical form does not collapse.
///
/// **This refutes the record's Part 5 claim in the general form it is stated
/// in.** The two mitigations it names cover input leaves and mint order; nothing
/// shares a *constant*, so `x * 2.0 + 2.0` spelled with one constant value and
/// spelled with two produces two node vectors, two witnesses, and two canonical
/// region identities for one binary32 function.
///
/// It is the converse property that fails, not the determination property: the
/// witness is too *fine* here rather than too coarse, so a caller comparing two
/// witnesses may conclude they differ when the bits do not — never that they
/// agree when the bits differ. That is why the witness derives no `PartialEq`,
/// and why nothing here rebuilds the expression vocabulary to share constants:
/// doing so would move every schedule identity of every region that mints a
/// repeated constant, which is an identity-domain step this ticket carries no
/// evidence for.
#[test]
fn a_duplicated_constant_is_a_spelling_the_canonical_form_does_not_collapse() {
    fn shared_constant() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let scaled = builder.multiply(input, two.clone()).unwrap();
        let root = builder.add(scaled, two).unwrap();
        builder.build(root).unwrap()
    }
    fn repeated_constant() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let scaled = builder.multiply(input, two).unwrap();
        let two_again = builder.constant(2.0_f32.to_bits()).unwrap();
        let root = builder.add(scaled, two_again).unwrap();
        builder.build(root).unwrap()
    }
    assert_eq!(shared_constant().nodes().len(), 4);
    assert_eq!(repeated_constant().nodes().len(), 5);
    assert_ne!(shared_constant(), repeated_constant());

    let shared = verified(pointwise_region(
        ScalarProgram::PointwiseF32(shared_constant()),
        strict_numerical(),
    ));
    let repeated = verified(pointwise_region(
        ScalarProgram::PointwiseF32(repeated_constant()),
        strict_numerical(),
    ));
    assert_ne!(
        RealizationWitness::of(&shared).pointwise_f32(),
        RealizationWitness::of(&repeated).pointwise_f32(),
    );
    assert_ne!(shared.canonical_identity(), repeated.canonical_identity());
}
