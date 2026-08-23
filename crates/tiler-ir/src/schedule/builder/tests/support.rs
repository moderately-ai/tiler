use super::super::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, ContractionAxisSource,
    ContributorArrival, ContributorCoverage, ConvergenceEvidence, ExecutionBinding, KernelSchedule,
    LogicalAccess, NumericalRealization, OwnershipProof, OwnershipProofKind,
    ReductionPaddingIdentity, ReductionPass, ReductionTopology, RegionId, RegionProgram,
    ResourceRequirements, ScalarProgram, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
    TailPolicy, TensorRole, element_count, partial_reduction_axis, partial_reduction_shape,
};
use crate::schedule::cooperative::{
    CooperativePhase, CooperativeTile, LocalCoordinateSource, LocalCoordinates, ParticipantRange,
    ParticipantSpace, StagedElement, StagedRead, StagedSpan, StagedWrite, WorkgroupStaging,
};
use crate::schedule::handles::{
    BoundsWitnessId, OwnershipWitnessId, PhaseId, StagingId, SyncPointId,
};
use crate::schedule::model::{
    ContributorOrder, ContributorPartition, CopyElement, CopyMember, LaunchPlan,
    PartitionedCopyProgram, RegionNumericalRequirements,
};
use crate::schedule::numerics::{
    ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, NumericalPermission,
    SubnormalMode,
};
use crate::schedule::synchronization::{
    FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationPlacement,
    SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
};
use crate::schedule::{PointwiseF32Expression, PointwiseF32ExpressionBuilder};
use crate::shape::{Axis, Shape};

/// Recorded canonical identity of the strict-`f32` pointwise test region.
///
/// The pointwise program is encoded as a typed, framed topological graph,
/// so its exact operand order, constants, root, and physical `f32` family are all pinned.
///
/// Rebaselined deliberately at the `tiler.schedule.v7` step, which gave the
/// numerical record its two elementary dimensions — the reciprocal-transform
/// permission and the approximate-intrinsic envelope — between the
/// signed-zero permission and the exceptional-value assumptions.
///
/// The `v6` rebaseline recorded the fieldless-input-role step, which removed
/// the declared-input ordinal payload from fieldless input roles.
///
/// Earlier rebaselines recorded the `tiler.schedule.v4` step, which gave
/// [`CooperativeTile`] its round count; the `v3` step, which gave
/// `TensorRole::Input` and `PointwiseF32Node::Input` their input ordinals,
/// so every input access and bounds proof gained four ordinal bytes and the
/// input leaf's framed length grew from nine to twenty-one; and before that,
/// the old `ScalarProgram::MultiplyThenAdd` tag (`0x21`) becoming the exact
/// `ScalarProgram::PointwiseF32` expression encoding (`0x24`).
pub(super) const STRICT_F32_REGION_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010100000000000000060000000101000000003100000000000000060000000101";

pub(in crate::schedule::builder) fn strict_numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
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

/// Overrides the scalar half of an already-seeded arithmetic program.
///
/// The direct-field spelling these tests used before the program became a
/// sum; panicking on an unseeded or copy-classified builder keeps a
/// fixture defect loud instead of silently minting a program.
pub(super) fn set_scalar(builder: &mut ScheduledRegionBuilder, scalar: ScalarProgram) {
    match builder.program.as_mut() {
        Some(RegionProgram::Numerical { scalar: slot, .. }) => *slot = scalar,
        Some(RegionProgram::PartitionedCopy(_)) | None => {
            panic!("seed an arithmetic program before overriding its scalar half")
        }
    }
}

/// Overrides the numerical half of an already-seeded arithmetic program.
pub(super) fn set_numerical(builder: &mut ScheduledRegionBuilder, numerical: NumericalRealization) {
    match builder.program.as_mut() {
        Some(RegionProgram::Numerical {
            numerical: slot, ..
        }) => *slot = numerical,
        Some(RegionProgram::PartitionedCopy(_)) | None => {
            panic!("seed an arithmetic program before overriding its numerical half")
        }
    }
}

/// The floating-point rows an arithmetic fixture's requirements derive.
///
/// A read-side mirror of the sum, so per-dimension assertions stay one
/// field access; the copy arm panics because every fixture using this is
/// arithmetic.
pub(super) struct FloatRows {
    pub(super) input_subnormals: SubnormalMode,
    pub(super) result_subnormals: SubnormalMode,
    pub(super) contraction: NumericalPermission,
    pub(super) reassociation: NumericalPermission,
    pub(super) permutation: NumericalPermission,
    pub(super) signed_zero: NumericalPermission,
    pub(super) nan_assumptions: ExceptionalValueAssumption,
    pub(super) infinity_assumptions: ExceptionalValueAssumption,
}

pub(super) fn float_rows(requirements: &ResourceRequirements) -> FloatRows {
    let RegionNumericalRequirements::FloatingPoint {
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        nan_assumptions,
        infinity_assumptions,
        ..
    } = requirements.numerical
    else {
        panic!("an arithmetic fixture derives floating-point requirement rows");
    };
    FloatRows {
        input_subnormals,
        result_subnormals,
        contraction,
        reassociation,
        permutation,
        signed_zero,
        nan_assumptions,
        infinity_assumptions,
    }
}

pub(super) fn scale_bias_expression(
    scale_bits: u32,
    bias_bits: u32,
) -> super::super::super::PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

pub(super) fn pointwise_builder(
    id: RegionId,
    shape: Shape,
    elements: u64,
) -> ScheduledRegionBuilder {
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(shape).unwrap();
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
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(scale_bias_expression(
                2.0_f32.to_bits(),
                1.0_f32.to_bits(),
            )),
            numerical: strict_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder
}

/// Builds the approved `(a * b) + c` region over three input tensors.
///
/// The three reads carry ordinals `0`, `1`, and `2` in access order, one
/// bounds proof each, and a write of the program output.
pub(super) fn three_input_builder(elements: u64) -> ScheduledRegionBuilder {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let a = expression.input(AccessOrdinal::new(0)).unwrap();
    let b = expression.input(AccessOrdinal::new(1)).unwrap();
    let c = expression.input(AccessOrdinal::new(2)).unwrap();
    let product = expression.multiply(a, b).unwrap();
    let root = expression.add(product, c).unwrap();
    let expression = expression.build(root).unwrap();

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([elements]))
        .unwrap();
    for ordinal in 0..3 {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(ordinal),
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
            bounds: BoundsWitnessId::new(3),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for ordinal in 0..3 {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(ordinal),
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
            id: BoundsWitnessId::new(3),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression),
            numerical: strict_numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}

/// Returns an admitted lane count for the accepted fixed-vector map tests.
pub(super) fn admitted_lanes(width: u64) -> super::super::super::model::VectorLaneCount {
    super::super::super::model::VectorLaneCount::new(width).expect("an admitted lane width")
}

/// Rebinds a builder's schedule to the fixed-vector map with the accepted
/// launch identity: `work_items` untouched, `grid_threads` as stated.
pub(super) fn into_fixed_vector_map(
    builder: &mut ScheduledRegionBuilder,
    width: u64,
    grid_threads: u64,
) {
    let schedule = builder.schedule.as_mut().expect("schedule was set");
    schedule.binding = ExecutionBinding::FixedVectorMap {
        lanes: admitted_lanes(width),
    };
    schedule.launch.grid_threads = grid_threads;
}

/// A realization that permits exactly the freedoms a split consumes.
///
/// Reassociation is permitted and every other dimension stays at its strict
/// resolution, so a region admitted under it is admitted for reassociation
/// alone. Permutation in particular stays forbidden, which is what makes
/// the admission tests below evidence of independence rather than of a
/// generally relaxed contract.
pub(super) fn reassociating_numerical() -> NumericalRealization {
    NumericalRealization {
        reassociation: NumericalPermission::Permitted,
        ..strict_numerical()
    }
}

/// The split every multi-pass fixture below declares: `6 = 3 x 2`.
pub(super) const SPLIT: ContributorPartition = ContributorPartition {
    partitions: 3,
    contributors_per_partition: 2,
};

/// Builds the partial pass of a `[2, 6] -> [2]` reduction split three ways.
pub(super) fn partial_pass_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
    let partial_elements = 2 * partition.partitions;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(2));
    builder
        .iteration_shape(
            partial_reduction_shape(&Shape::from_dims([2]), partition)
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
                axes: vec![Axis::new(1)],
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
                axes: vec![Axis::new(1)],
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
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: reassociating_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(partition),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ..linear_schedule(partial_elements, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder
}

/// Builds a `[2, 6] -> [2]` serial reduction over input zero.
///
/// The shape the extrema fixtures below share. A *serial* topology rather
/// than a split, because the serial arm is the only one the identity-less
/// fold is admitted under; the refusal of every other topology is asserted
/// separately rather than assumed.
pub(super) fn serial_reduction_builder(scalar: ScalarProgram) -> ScheduledRegionBuilder {
    let input = Shape::from_dims([2, 6]);
    let output = Shape::from_dims([2]);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(41));
    builder.iteration_shape(output.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: vec![Axis::new(1)],
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
                axes: vec![Axis::new(1)],
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
        .program(RegionProgram::Numerical {
            scalar,
            numerical: strict_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(2, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder
}

/// Builds a valid `mk,nk->mn` contraction over the named program inputs.
pub(super) fn contraction_builder() -> ScheduledRegionBuilder {
    let operand = Shape::from_dims([2, 3]);
    let output = Shape::from_dims([2, 2]);
    let contracted = Shape::from_dims([3]);
    let left = TensorRole::Input;
    let right = TensorRole::Input;
    let operand_map = |free_position| LogicalAccess::ContractionOperand {
        operand_shape: operand.clone(),
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
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(42));
    builder.iteration_shape(output.clone()).unwrap();
    for (witness, tensor, map) in [(0, left, operand_map(0)), (1, right, operand_map(1))] {
        builder
            .push_access(Access {
                tensor,
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
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 6 },
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
            kind: BoundsProofKind::LinearRange { element_count: 4 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            },
            numerical: strict_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(4, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder
}

/// The scale a root-mean-square normalization's producing stage computes.
///
/// `Rsqrt(a / N + eps)` over the fold's value, which is local access zero.
/// The shipped instance of a fold epilogue, spelled here from the physical
/// vocabulary rather than from any law: what this module verifies is the
/// *schedule*, and it has no opinion on which semantic operation the chain
/// realizes.
pub(super) fn scale_epilogue() -> PointwiseF32Expression {
    let mut builder = PointwiseF32ExpressionBuilder::new();
    let total = builder.input(AccessOrdinal::FIRST).unwrap();
    let extent = builder.constant(6.0_f32.to_bits()).unwrap();
    let mean = builder.divide(total, extent).unwrap();
    let bias = builder.constant(1.0e-6_f32.to_bits()).unwrap();
    let biased = builder.add(mean, bias).unwrap();
    let root = builder.rsqrt(biased).unwrap();
    builder.build(root).unwrap()
}

/// The squaring fold carrying that epilogue, over the shared fixture shape.
pub(super) fn squared_sum_with_epilogue(epilogue: PointwiseF32Expression) -> ScalarProgram {
    ScalarProgram::SquaredSerialSumThenEpilogue {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
        epilogue,
    }
}

/// The extrema fold this family embeds, over the shared fixture shape.
pub(super) fn maximum_scalar() -> ScalarProgram {
    ScalarProgram::StrictSerialMaximum {
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
    }
}

/// Rewrites one pass of a sum split into the extrema fold's own.
///
/// The three edits are the whole difference, and each is load-bearing: the
/// read binds the original scores where a sum's partial pass binds an
/// intermediate, the program is the identity-less fold, and the realization
/// is the *strict* one — reassociation forbidden — because a split of this
/// family spends no permission. A fixture that relaxed the contract would
/// prove the topology admissible without proving the interesting half of it.
pub(super) fn into_extrema_split(
    builder: &mut ScheduledRegionBuilder,
    axes: Vec<Axis>,
    read: TensorRole,
) {
    builder.accesses[0].tensor = read;
    builder.bounds_proofs[0].tensor = read;
    set_scalar(
        builder,
        ScalarProgram::StrictSerialMaximum {
            axes,
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
        },
    );
    set_numerical(builder, strict_numerical());
    let Some(ReductionTopology::MultiPass {
        permits_reassociation,
        ..
    }) = builder
        .schedule
        .as_mut()
        .map(|schedule| &mut schedule.reduction)
    else {
        panic!("the fixture schedules a multi-pass split")
    };
    *permits_reassociation = false;
}

/// The partial pass of an extrema split: fold the scores, stage one maximum.
pub(super) fn extrema_partial_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
    let mut builder = partial_pass_builder(partition);
    into_extrema_split(&mut builder, vec![Axis::new(1)], TensorRole::Input);
    builder
}

/// The cooperative tile over the extrema fold, under a strict contract.
pub(super) fn extrema_cooperative_builder() -> ScheduledRegionBuilder {
    let ReductionTopology::CooperativeWorkgroup {
        coverage,
        tile,
        axes,
        order,
        accumulation,
        arrival,
        ..
    } = cooperative_topology(cooperative_tile_fixture())
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    let mut builder = cooperative_builder_parts(
        SPLIT,
        6,
        ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation: false,
            permits_permutation: false,
            arrival,
        },
        strict_numerical(),
    );
    builder.accesses[0].tensor = TensorRole::Input;
    builder.bounds_proofs[0].tensor = TensorRole::Input;
    set_scalar(&mut builder, maximum_scalar());
    builder
}

/// Builds the final pass that combines those partials into `[2]`.
pub(super) fn final_pass_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
    let partial_shape = partial_reduction_shape(&Shape::from_dims([2]), partition)
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: reassociating_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::MultiPass {
                pass: ReductionPass::Final,
                coverage: ContributorCoverage::Exact(partition),
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ..linear_schedule(2, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder
}

pub(super) fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// The bare fold this family's fixtures declare, over one reduced axis.
pub(super) fn bare_sum(axes: Vec<Axis>) -> ScalarProgram {
    ScalarProgram::StrictSerialSum {
        axes,
        order: ContributorOrder::OriginalAxisLexicographic,
        canonical_nan_bits: 0x7fc0_0000,
        empty_identity_bits: 0.0_f32.to_bits(),
    }
}

/// Rebinds a reduction fixture's contributor read to another boundary tensor.
///
/// The access and its bounds proof move together because
/// [`verify_proof_records`] requires them to name one tensor: separating them
/// would report the proof reference and prove nothing about the boundary role
/// under test.
pub(super) fn read_from(builder: &mut ScheduledRegionBuilder, tensor: TensorRole) {
    builder.accesses[0].tensor = tensor;
    builder.bounds_proofs[0].tensor = tensor;
}

pub(super) const NEG_ZERO: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0x8000_0000);

pub(super) const PADDED_SPLIT: ContributorPartition = ContributorPartition {
    partitions: 3,
    contributors_per_partition: 3,
};

/// Turns a split partial pass into the squaring-prologue reduction.
///
/// The prologue reads the original input, exactly as the scale-bias one
/// does, so the read access and its proof move from the intermediate to the
/// first input tensor along with the scalar program.
pub(super) fn squared_partial_pass_builder(
    partition: ContributorPartition,
) -> ScheduledRegionBuilder {
    let mut builder = partial_pass_builder(partition);
    builder.accesses[0].tensor = TensorRole::Input;
    builder.bounds_proofs[0].tensor = TensorRole::Input;
    set_scalar(
        &mut builder,
        ScalarProgram::SquaredSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
    );
    builder
}

/// The staging allocation every cooperative fixture below declares.
pub(super) fn tile_staging(slots: u64, live_through: PhaseId) -> WorkgroupStaging {
    WorkgroupStaging {
        id: StagingId::FIRST,
        element: StagedElement::F32,
        slots,
        live_from: PhaseId::FIRST,
        live_through,
    }
}

/// The point that orders the fixture's one handoff.
///
/// Every field is the value the tile's own dependency derives, so a
/// perturbation test changes exactly one of them and the rejection names the
/// dimension it changed.
pub(super) fn tile_point() -> SynchronizationPoint {
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

/// The well-formed tile: write your own slot, then read the whole set.
pub(super) fn cooperative_tile_fixture() -> CooperativeTile {
    CooperativeTile {
        synchronization: vec![tile_point()],
        rounds: 1,
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalLinearInvocation,
            participants: ParticipantSpace::new(&[3]).expect("rank one is within the bound"),
        },
        staging: vec![tile_staging(3, PhaseId::new(1))],
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

pub(super) fn cooperative_topology(tile: CooperativeTile) -> ReductionTopology {
    cooperative_topology_with(tile, SPLIT)
}

pub(super) fn cooperative_topology_with(
    tile: CooperativeTile,
    partition: ContributorPartition,
) -> ReductionTopology {
    cooperative_topology_arriving(tile, partition, ContributorArrival::AscendingParticipant)
}

pub(super) fn cooperative_topology_arriving(
    tile: CooperativeTile,
    partition: ContributorPartition,
    arrival: ContributorArrival,
) -> ReductionTopology {
    ReductionTopology::CooperativeWorkgroup {
        coverage: ContributorCoverage::Exact(partition),
        tile,
        axes: vec![Axis::new(1)],
        order: ContributorOrder::OriginalAxisLexicographic,
        accumulation: ArithmeticType::F32,
        permits_reassociation: true,
        permits_permutation: false,
        arrival,
    }
}

/// Builds the cooperative realization of the `[2, 6] -> [2]` reduction.
///
/// One workgroup per output position, three invocations per workgroup, so
/// the iteration domain is the output shape with the participant axis
/// appended — the same layout a partial pass uses, which is what keeps the
/// participant ordinal the innermost coordinate of the invocation index.
pub(super) fn cooperative_builder(tile: CooperativeTile) -> ScheduledRegionBuilder {
    cooperative_builder_with(tile, SPLIT)
}

/// The same fixture over an explicit split, for the widths `SPLIT` fixes.
pub(super) fn cooperative_builder_with(
    tile: CooperativeTile,
    split: ContributorPartition,
) -> ScheduledRegionBuilder {
    cooperative_builder_parts(
        split,
        6,
        cooperative_topology_with(tile, split),
        reassociating_numerical(),
    )
}

/// The fixture region, over a contracted extent the caller states.
///
/// `contracted` is a parameter rather than the fixture's own `6` because the
/// two-dimensional tiles below need a participant count the `[2, 6]` domain
/// cannot split — the reduction shape, the contributor coverage, and the
/// launch width are one arithmetic, and a fixture that fixed one of them
/// would make the other two unstatable.
pub(super) fn cooperative_builder_parts(
    split: ContributorPartition,
    contracted: u64,
    reduction: ReductionTopology,
    numerical: NumericalRealization,
) -> ScheduledRegionBuilder {
    let participants = split.partitions;
    let work_items = 2 * participants;
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
                axes: vec![Axis::new(1)],
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
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    // Two positions, not six: the write covers one output per workgroup, and
    // the ownership proof below says the same number.
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical,
        })
        .unwrap();
    let threads = u32::try_from(participants).expect("the fixture's width fits u32");
    builder
        .schedule(KernelSchedule {
            threads_per_workgroup: threads,
            reduction,
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder
}

pub(super) fn cooperative_rejection(builder: ScheduledRegionBuilder) -> ScheduledRegionDiagnostic {
    let diagnostics = builder.build().unwrap_err().diagnostics().to_vec();
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("expected exactly one diagnostic, got {diagnostics:?}")
    };
    *diagnostic
}

/// The loop-carried split: three participants, one contributor each, twice.
///
/// The same `[2, 6] -> [2]` reduction and the same launch as `SPLIT`, with
/// the six contributors covered as `3 * 1 * 2` instead of `3 * 2 * 1`. Keeping
/// the launch identical is what makes the round count the only difference
/// between the two fixtures.
pub(super) const ROUND_SPLIT: ContributorPartition = ContributorPartition {
    partitions: 3,
    contributors_per_partition: 1,
};

/// The point that orders the fixture's rewrite, at the round boundary.
pub(super) fn round_boundary_point() -> SynchronizationPoint {
    SynchronizationPoint {
        id: SyncPointId::new(1),
        placement: SynchronizationPlacement::RoundBoundary,
        convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
        ..tile_point()
    }
}

/// The loop-carried tile: the single-round fixture, run twice.
///
/// Structurally identical to [`cooperative_tile_fixture`] apart from the
/// round count, the second point, and the convergence class both points now
/// have to name — which is the whole content of the capability.
pub(super) fn multi_round_tile_fixture() -> CooperativeTile {
    CooperativeTile {
        rounds: 2,
        synchronization: vec![
            SynchronizationPoint {
                convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
                ..tile_point()
            },
            round_boundary_point(),
        ],
        ..cooperative_tile_fixture()
    }
}

pub(super) fn multi_round_builder(tile: CooperativeTile) -> ScheduledRegionBuilder {
    cooperative_builder_with(tile, ROUND_SPLIT)
}

/// Applies one edit to the loop-carried fixture and returns its builder.
pub(super) fn round_perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
    let mut tile = multi_round_tile_fixture();
    edit(&mut tile);
    multi_round_builder(tile)
}

/// Builds a verifiable partitioned-copy region.
///
/// `members[k]` is `(source ordinal, extent)`. One read per distinct
/// ordinal (dense `0..reads`), each carrying the fieldless copy-source map
/// and a `LinearRange` proof of its member-derived source element count;
/// the one owning write is a program output under `LinearIdentity`.
pub(super) fn partitioned_copy_builder(
    shape: &Shape,
    axis: u32,
    members: &[(u32, u64)],
) -> ScheduledRegionBuilder {
    let elements = element_count(shape).expect("the fixture domain is finite");
    let reads = members
        .iter()
        .map(|(source, _)| source + 1)
        .max()
        .unwrap_or(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(shape.clone()).unwrap();
    for read in 0..reads {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::PartitionedCopySource,
                bounds: BoundsWitnessId::new(read),
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
            bounds: BoundsWitnessId::new(reads),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for read in 0..reads {
        let extent = members
            .iter()
            .find(|(source, _)| *source == read)
            .map_or(0, |(_, extent)| *extent);
        let source_elements = shape
            .extents()
            .iter()
            .enumerate()
            .map(|(position, source_extent)| {
                if position == usize::try_from(axis).unwrap() {
                    extent
                } else {
                    source_extent.get()
                }
            })
            .try_fold(1_u64, u64::checked_mul)
            .expect("the fixture source domain is finite");
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(read),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: source_elements,
                },
            })
            .unwrap();
    }
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(reads),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::PartitionedCopy(PartitionedCopyProgram {
            element: CopyElement::F32,
            axis: Axis::new(axis),
            members: members
                .iter()
                .map(|(source, extent)| CopyMember {
                    source: AccessOrdinal::new(*source),
                    extent: *extent,
                })
                .collect(),
        }))
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}
