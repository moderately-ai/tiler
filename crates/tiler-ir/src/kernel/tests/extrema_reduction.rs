use super::super::{BinaryOp, Builtin, KernelConstant, OperationView, lower_scheduled_region};
use super::support::{
    NAN_BITS, binary_op_counts, cooperative_point, cooperative_region, linear_schedule,
    multi_round_cooperative_region, numerical, reduction_region,
};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorCoverage, ContributorOrder, ContributorPartition, ConvergenceEvidence,
    CooperativePhase, CooperativeTile, KernelSchedule, LaunchPlan, LocalCoordinateSource,
    LocalCoordinates, LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    ParticipantRange, ParticipantSpace, PhaseId, ReductionPaddingIdentity, ReductionPass,
    ReductionTopology, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    StagedElement, StagedRead, StagedSpan, StagedWrite, StagingId, SyncPointId,
    SynchronizationPlacement, SynchronizationPoint, TensorRole, VerifiedScheduledRegion,
    WorkgroupStaging,
};
use crate::shape::{Axis, Shape};

/// A zero-extent reduction commits `+0.0` with no fold and no synchronization.
///
/// The authority a cooperative tile must not disturb: the empty result is the
/// reducer's declared `empty_identity_bits`, committed by one invocation from a
/// constant. There is no loop to enter and nothing to stage, which is why the
/// schedule verifier refuses a tile over an empty contributor domain rather
/// than describing a handoff of values no participant produces.
#[test]
fn a_zero_extent_reduction_commits_its_identity_without_a_loop_or_a_barrier() {
    let scheduled = reduction_region(
        RegionId::new(26),
        &Shape::from_dims([2, 0]),
        &[Axis::new(1)],
    );
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    assert_eq!(kernel.staging().len(), 0);
    assert_eq!(kernel.requirements().local_memory_bytes, 0);
    assert_eq!(kernel.admitted_builtins(), [Builtin::GlobalInvocationIndex]);
    let mut stored = None;
    let mut loops = 0;
    let mut barriers = 0;
    for operation in kernel.body().operations() {
        let OperationView::Predicated { body, .. } = operation.view() else {
            continue;
        };
        for inner in body.operations() {
            match inner.view() {
                OperationView::SerialLoop(_) => loops += 1,
                OperationView::Barrier { .. } => barriers += 1,
                OperationView::Store { value, .. } => {
                    stored = kernel.value_constant(value).unwrap();
                }
                _ => {}
            }
        }
    }
    assert_eq!(loops, 0);
    assert_eq!(barriers, 0);
    assert_eq!(stored, Some(KernelConstant::F32Bits(0.0_f32.to_bits())));
}

/// The extrema fold lowers to a bounded loop whose combine is a `Maximum`.
///
/// The shape is the serial sum's — a seed load, a loop over the remaining
/// contributors, a canonicalization after each combine, and one owning store —
/// and the only difference is the combine's operation. That is asserted rather
/// than described, because a lowering that reused `F32Add` here would produce a
/// structurally identical kernel computing a different function.
#[test]
fn the_extrema_fold_lowers_to_a_bounded_loop_combining_with_a_maximum() {
    let scheduled = maximum_reduction_region(RegionId::new(30));
    let kernel = lower_scheduled_region(&scheduled).expect("the extrema region lowers");
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Maximum),
        1,
        "one combine per loop iteration, emitted once"
    );
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Add),
        0,
        "the extrema fold combines with a maximum and never with an addition"
    );

    // The control: the bare serial sum over the same shape emits the reverse.
    let sum = lower_scheduled_region(&reduction_region(
        RegionId::new(31),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    ))
    .expect("the bare sum lowers");
    assert_eq!(binary_op_counts(&sum, BinaryOp::F32Maximum), 0);
    assert_eq!(binary_op_counts(&sum, BinaryOp::F32Add), 1);
}

/// A `[2, 3] -> [2]` squaring fold, with or without the scale epilogue.
///
/// Two fixtures from one constructor, so the epilogue is the *only* difference
/// between the region the test measures and its control — a second constructor
/// could drift in a field the assertion then attributes to the epilogue.
fn squared_fold_region(id: RegionId, epilogue: bool) -> VerifiedScheduledRegion {
    let input = Shape::from_dims([2, 3]);
    let axes = [Axis::new(1)];
    let output = input.without_axes(&axes);
    let output_elements = crate::schedule::element_count(&output).expect("bounded fixture shape");
    let tensor = TensorRole::Input;
    let scalar = if epilogue {
        let mut chain = crate::schedule::PointwiseF32ExpressionBuilder::new();
        let total = chain.input(AccessOrdinal::FIRST).unwrap();
        let extent = chain.constant(3.0_f32.to_bits()).unwrap();
        let mean = chain.divide(total, extent).unwrap();
        let bias = chain.constant(1.0e-6_f32.to_bits()).unwrap();
        let biased = chain.add(mean, bias).unwrap();
        let root = chain.rsqrt(biased).unwrap();
        ScalarProgram::SquaredSerialSumThenEpilogue {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
            epilogue: chain.build(root).unwrap(),
        }
    } else {
        ScalarProgram::SquaredSerialSum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        }
    };
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    builder
        .push_access(Access {
            tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.to_vec(),
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
            tensor,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output,
                axes: axes.to_vec(),
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
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar,
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder.build().unwrap()
}

/// A fold's epilogue is emitted once, after the fold and before the store.
///
/// **Once per output position, not once per contributor**, which is the whole
/// reason the epilogue belongs to this region rather than to the pass that
/// consumes its result: the division, the bias, and the reciprocal square root
/// each appear exactly once in the body while the squaring multiply appears
/// twice — once at the seed and once in the loop. A lowering that had put the
/// chain inside the contributor loop would emit one of each per contributor and
/// compute the same value `N` times per row.
///
/// The bare squaring fold over the same shape is the control: the identical
/// region with the chain absent, so every count that differs is the epilogue's.
#[test]
fn a_folds_epilogue_is_emitted_once_after_the_fold() {
    let scheduled = squared_fold_region(RegionId::new(34), true);
    let kernel = lower_scheduled_region(&scheduled).expect("the epilogue-carrying region lowers");
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Divide),
        1,
        "the mean division is per folded row, not per contributor",
    );
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Add),
        2,
        "one combine inside the loop and one bias addition after it",
    );
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Multiply),
        2,
        "the squaring prologue, at the seed and in the loop",
    );

    let bare = lower_scheduled_region(&squared_fold_region(RegionId::new(34), false))
        .expect("the bare squaring fold lowers");
    assert_eq!(binary_op_counts(&bare, BinaryOp::F32Divide), 0);
    assert_eq!(
        binary_op_counts(&bare, BinaryOp::F32Add),
        1,
        "the combine alone, so the second addition above is the bias",
    );
    assert_eq!(
        binary_op_counts(&bare, BinaryOp::F32Multiply),
        2,
        "the same squaring prologue, so the difference above is the epilogue alone",
    );
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        bare.canonical_identity().as_bytes(),
    );
}

/// The appended binary tag separates kernel identity from the addition's.
///
/// Two kernels differing in nothing but the combine's operation. An appended tag
/// that had collided with `F32Add`'s would make these identities equal, which is
/// the concrete form of "the kernel identity domain did not step": the new tag
/// separates, and every tag below it keeps its meaning.
#[test]
fn the_maximum_tag_separates_kernel_identity_from_the_addition() {
    let maximum = lower_scheduled_region(&maximum_reduction_region(RegionId::new(32)))
        .expect("the extrema region lowers");
    let sum = lower_scheduled_region(&reduction_region(
        RegionId::new(32),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    ))
    .expect("the bare sum lowers");
    assert_ne!(
        maximum.canonical_identity().as_bytes(),
        sum.canonical_identity().as_bytes()
    );
}

/// A `[2, 3] -> [2]` extrema fold over the first input tensor.
fn maximum_reduction_region(id: RegionId) -> VerifiedScheduledRegion {
    let input = Shape::from_dims([2, 3]);
    let axes = [Axis::new(1)];
    let output = input.without_axes(&axes);
    let output_elements = crate::schedule::element_count(&output).expect("bounded fixture shape");
    let tensor = TensorRole::Input;
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    builder
        .push_access(Access {
            tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.to_vec(),
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
            tensor,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output,
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialMaximum {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder.build().unwrap()
}

/// The partial pass of a `[2, 6] -> [2]` extrema fold split three ways.
///
/// A *strict* realization, because a split of this family spends no
/// reassociation permission — the schedule verifier's admission rests on the
/// family's algebra rather than on the contract, and a fixture that relaxed the
/// contract would not exercise that.
fn maximum_partial_pass_region() -> VerifiedScheduledRegion {
    let input = Shape::from_dims([2, 6]);
    let output = Shape::from_dims([2]);
    let axes = [Axis::new(1)];
    let partition = ContributorPartition {
        partitions: 3,
        contributors_per_partition: 2,
    };
    let iteration = crate::schedule::partial_reduction_shape(&output, partition)
        .expect("a rank-two partial shape is within the governed bound");
    let partial_elements =
        crate::schedule::element_count(&iteration).expect("bounded fixture shape");
    let tensor = TensorRole::Input;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(33));
    builder.iteration_shape(iteration).unwrap();
    builder
        .push_access(Access {
            tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.to_vec(),
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
            tensor,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output,
                axes: axes.to_vec(),
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
            scalar: ScalarProgram::StrictSerialMaximum {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(partition),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: crate::schedule::ArithmeticType::F32,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(partial_elements, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder.build().unwrap()
}

/// Identity-padded coverage is representable and verified; this lowering has
/// no body that injects the stated identity, so it refuses rather than folding
/// padding slots as real contributors.
#[test]
fn a_padded_split_is_representable_and_not_lowered() {
    let mut region = maximum_partial_pass_region().region().clone();
    let ReductionTopology::MultiPass { coverage, .. } = &mut region.schedule.reduction else {
        panic!("the fixture is a multi-pass split");
    };
    *coverage = ContributorCoverage::IdentityPadded {
        partition: ContributorPartition {
            partitions: 3,
            contributors_per_partition: 3,
        },
        identity: ReductionPaddingIdentity::F32(0xff80_0000),
    };
    let verified = ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("a suffix-padded extrema split verifies");
    assert_eq!(
        lower_scheduled_region(&verified)
            .expect_err("padded coverage is not lowered")
            .rule(),
        "padded-contributor-coverage"
    );
}

/// The cooperative realization of a `[2, 6] -> [2]` extrema fold.
///
/// The sum fixture's tile, participant space, and synchronization point over the
/// identity-less family and a strict realization: three participants each folding
/// two contributors into their own slot, all three reading the staged set back,
/// one committing.
fn cooperative_maximum_region() -> VerifiedScheduledRegion {
    let tensor = TensorRole::Input;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(34));
    builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
    builder
        .push_access(Access {
            tensor,
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
            tensor,
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
            scalar: ScalarProgram::StrictSerialMaximum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            threads_per_workgroup: 3,
            reduction: ReductionTopology::CooperativeWorkgroup {
                coverage: ContributorCoverage::Exact(ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 2,
                }),
                tile: CooperativeTile {
                    rounds: 1,
                    coordinates: LocalCoordinates {
                        source: LocalCoordinateSource::LocalLinearInvocation,
                        participants: ParticipantSpace::new(&[3])
                            .expect("rank one is within the bound"),
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
                                span: StagedSpan::new(&[1], 0, 1)
                                    .expect("rank one is within the bound"),
                            }],
                            reads: Vec::new(),
                        },
                        CooperativePhase {
                            id: PhaseId::new(1),
                            participation: ParticipantRange { first: 0, count: 3 },
                            writes: Vec::new(),
                            reads: vec![StagedRead {
                                staging: StagingId::FIRST,
                                span: StagedSpan::new(&[0], 0, 3)
                                    .expect("rank one is within the bound"),
                            }],
                        },
                    ],
                    synchronization: vec![cooperative_point()],
                    commit: ParticipantRange { first: 0, count: 1 },
                },
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: crate::schedule::ArithmeticType::F32,
                permits_reassociation: false,
                permits_permutation: false,
                arrival: crate::schedule::ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: 6,
                threads_per_workgroup: 3,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(6, OwnershipWitnessId::new(0))
        })
        .unwrap();
    builder.build().unwrap()
}

/// The partial pass of an extrema split lowers with partitioned addressing.
///
/// The two facts that make it a *split* of *this* family rather than either one
/// alone: the invocation index is divided into an output coordinate and a
/// partition ordinal — which the unsplit serial extrema fold never emits — and
/// the fold that consumes the result combines with a maximum. A body that split
/// correctly and added would be structurally right and numerically wrong.
#[test]
fn a_split_extrema_partial_pass_lowers_with_partitioned_addressing_and_a_maximum() {
    let kernel = lower_scheduled_region(&maximum_partial_pass_region())
        .expect("the extrema partial pass lowers");
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Maximum),
        1,
        "one combine per loop iteration over the partition's two contributors"
    );
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Add),
        0,
        "a split of the extrema fold never combines with an addition"
    );
    let divides = binary_op_counts(&kernel, BinaryOp::IndexDivide);
    let modulos = binary_op_counts(&kernel, BinaryOp::IndexModulo);
    assert_eq!((divides, modulos), (2, 2));

    // The control: the unsplit serial extrema fold over the same family emits
    // neither, so the split arithmetic is the topology's and not the family's.
    let serial = lower_scheduled_region(&maximum_reduction_region(RegionId::new(35)))
        .expect("the serial extrema region lowers");
    assert_eq!(binary_op_counts(&serial, BinaryOp::IndexDivide), 0);
    assert_eq!(binary_op_counts(&serial, BinaryOp::IndexModulo), 0);
}

/// A cooperative extrema tile folds and stages with a maximum at both levels.
///
/// The tile folds twice — each participant's own contributor share, then the
/// staged set — and the combiner has to reach both. A lowering that carried the
/// family only to the first would stage correct partials and reduce them with an
/// addition, which is the exact defect the two counts below refuse.
#[test]
fn a_cooperative_extrema_tile_folds_and_stages_with_a_maximum() {
    let kernel = lower_scheduled_region(&cooperative_maximum_region())
        .expect("the cooperative extrema region lowers");
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Maximum),
        2,
        "one combine in the partition fold and one in the staged fold"
    );
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Add), 0);
    assert_eq!(kernel.requirements().local_memory_bytes, 12);

    // The control: the same tile over the strict serial sum emits the reverse at
    // both levels, so the combiner is read from the program rather than fixed.
    let summed =
        lower_scheduled_region(&cooperative_region()).expect("the cooperative sum region lowers");
    assert_eq!(binary_op_counts(&summed, BinaryOp::F32Maximum), 0);
    assert_eq!(binary_op_counts(&summed, BinaryOp::F32Add), 2);
}

/// A loop-carried extrema tile combines with a maximum at every level.
///
/// The round loop is the third place the fold's operation has to reach — after
/// each participant's own share and the staged set — because a tile whose phases
/// repeat carries an accumulator across the back edge. Its per-round width is one
/// contributor, so the partition fold emits no combine at all and every maximum
/// counted below belongs to the staged fold or the round accumulator: the peel's
/// staged fold, the loop body's staged fold, and the round combine.
#[test]
fn a_loop_carried_extrema_tile_carries_its_maximum_across_rounds() {
    let kernel = lower_scheduled_region(&multi_round_maximum_region())
        .expect("the loop-carried extrema region lowers");
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Maximum), 3);
    assert_eq!(
        binary_op_counts(&kernel, BinaryOp::F32Add),
        0,
        "the round accumulator combines with the family's own operation"
    );

    // The control: the same tile over the strict serial sum emits the reverse.
    let summed = lower_scheduled_region(&multi_round_cooperative_region())
        .expect("the loop-carried sum region lowers");
    assert_eq!(binary_op_counts(&summed, BinaryOp::F32Maximum), 0);
    assert_eq!(binary_op_counts(&summed, BinaryOp::F32Add), 3);
}

/// The extrema tile with its phases run twice and its slots rewritten.
///
/// The same transformation [`multi_round_cooperative_region`] applies to the sum
/// fixture, over the identity-less family: one contributor per participant per
/// round, both points naming the round-loop convergence derivation, and a round
/// boundary discharging the rewrite.
fn multi_round_maximum_region() -> VerifiedScheduledRegion {
    let mut region = cooperative_maximum_region().region().clone();
    let ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } =
        &mut region.schedule.reduction
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    let ContributorCoverage::Exact(partition) = coverage else {
        panic!("the fixture is exact coverage")
    };
    partition.contributors_per_partition = 1;
    tile.rounds = 2;
    tile.synchronization[0].convergence = ConvergenceEvidence::EveryParticipantExecutesEveryRound;
    tile.synchronization.push(SynchronizationPoint {
        id: SyncPointId::new(1),
        placement: SynchronizationPlacement::RoundBoundary,
        convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
        ..cooperative_point()
    });
    ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("the loop-carried extrema region verifies")
}
