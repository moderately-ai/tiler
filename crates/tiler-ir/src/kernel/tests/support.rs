use super::super::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BlockRef, BufferAccess, BufferParameter,
    Builtin, CompareOp, ConvertOp, ExecutionScope, KernelBufferId, KernelBuilder, KernelConstant,
    KernelDiagnostic, KernelType, KernelValueId, MemoryScope, OperationView, StagingParameter,
    VerifiedKernel,
};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, BoundsProof, BoundsProofKind,
    BoundsWitnessId, ContractionAxisSource, ContributorCoverage, ContributorOrder,
    ContributorPartition, ConvergenceEvidence, CooperativePhase, CooperativeTile,
    ExceptionalValueAssumption, ExecutionBinding, FencedSpaces, KernelSchedule, LaunchPlan,
    LocalCoordinateSource, LocalCoordinates, LogicalAccess, MemoryOrdering, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ParticipantRange,
    ParticipantSpace, PhaseId, PointwiseF32Expression, PointwiseF32ExpressionBuilder,
    ReductionTopology, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    StagedElement, StagedRead, StagedSpan, StagedWrite, StagingId, SubnormalMode, SyncPointId,
    SynchronizationKind, SynchronizationPlacement, SynchronizationPoint, SynchronizationScope,
    SynchronizationSubject, TailPolicy, TensorRole, VerifiedScheduledRegion, WorkgroupStaging,
    element_count,
};
use crate::shape::{Axis, Shape};

/// The mutable numerical half of a cloned arithmetic region's program.
pub(super) fn region_numerical_mut(
    region: &mut crate::schedule::ScheduledRegion,
) -> &mut crate::schedule::NumericalRealization {
    match &mut region.index.program {
        RegionProgram::Numerical { numerical, .. } => numerical,
        RegionProgram::PartitionedCopy(_) => panic!("the fixture region is arithmetic"),
    }
}

pub(super) const NAN_BITS: u32 = 0x7fc0_0000;
pub(super) const SCALE_BITS: u32 = 0x4000_0000;
pub(super) const BIAS_BITS: u32 = 0x3f80_0000;
pub(super) const ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX: &str = "74696c65722e6b65726e656c2e763900000000000000018474696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101010000000000000006000000010100000000310000000000000006000000010100000000000000020100030101000000000000000602000301020000000000000006000000000000000101000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010000000200000001000000000000000001010001010101010101010101000000000000000a020201030303030303030000000000000000000000000000000411010000000000000001000000001202000000000000000600000000000000010000000114010000000000000001000000000000000100000002180000000200000000000000000000000000000008160000000000000000000000000000000000000001000000031203400000000000000000000001000000041306000000030000000400000000000000010000000515010000000500000000000000010000000612033f8000000000000000000001000000071305000000060000000700000000000000010000000815010000000800000000000000010000000917000000010000000000000009000000010000000000000000000000000000000000000000";

pub(super) fn numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        NAN_BITS,
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

pub(super) fn scale_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// A pointwise scale-then-bias region over `shape`.
pub(super) fn pointwise_region(id: RegionId, shape: &Shape) -> VerifiedScheduledRegion {
    pointwise_expression_region(id, shape, scale_bias_expression(SCALE_BITS, BIAS_BITS))
}

pub(super) fn pointwise_expression_region(
    id: RegionId,
    shape: &Shape,
    expression: PointwiseF32Expression,
) -> VerifiedScheduledRegion {
    let elements = crate::schedule::element_count(shape).expect("bounded fixture shape");
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(shape.clone()).unwrap();
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
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression),
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder.build().unwrap()
}

/// A strict serial sum over `axes` of `input`.
pub(super) fn reduction_region(
    id: RegionId,
    input: &Shape,
    axes: &[Axis],
) -> VerifiedScheduledRegion {
    let output = input.without_axes(axes);
    let output_elements = crate::schedule::element_count(&output).expect("bounded fixture shape");
    let contributor_map = LogicalAccess::ReductionContributor {
        input_shape: input.clone(),
        output_shape: output.clone(),
        axes: axes.to_vec(),
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: contributor_map,
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
                input_shape: input.clone(),
                output_shape: input.without_axes(axes),
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
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
                empty_identity_bits: 0.0_f32.to_bits(),
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

/// Declares the canonical pointwise signature.
pub(super) fn pointwise_signature(
    builder: &mut KernelBuilder,
    scheduled: &VerifiedScheduledRegion,
    elements: u64,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: elements,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: elements,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    (read, write)
}

/// Emits the canonical invocation index and its bounds predicate.
pub(super) fn guard(
    builder: &mut KernelBuilder,
    work_items: u64,
) -> (KernelValueId, KernelValueId) {
    let invocation = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let extent = builder.constant(KernelConstant::Index(work_items)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, invocation, extent)
        .unwrap();
    (invocation, active)
}

/// Emits the canonical scale-then-bias arithmetic with its NaN normalizations.
pub(super) fn scale_bias(builder: &mut KernelBuilder, loaded: KernelValueId) -> KernelValueId {
    let scale = builder
        .constant(KernelConstant::F32Bits(SCALE_BITS))
        .unwrap();
    let product = builder
        .binary(BinaryOp::F32Multiply, loaded, scale)
        .unwrap();
    let product = builder
        .convert(ConvertOp::CanonicalizeF32Nan, product)
        .unwrap();
    let bias = builder
        .constant(KernelConstant::F32Bits(BIAS_BITS))
        .unwrap();
    let sum = builder.binary(BinaryOp::F32Add, product, bias).unwrap();
    builder.convert(ConvertOp::CanonicalizeF32Nan, sum).unwrap()
}

/// Builds the canonical pointwise kernel entirely through the public builder.
pub(super) fn canonical_pointwise(
    scheduled: &VerifiedScheduledRegion,
    elements: u64,
) -> KernelBuilder {
    let mut builder = KernelBuilder::new(scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, scheduled, elements);
    let (invocation, active) = guard(&mut builder, elements);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    builder
}

pub(super) fn diagnostics(builder: KernelBuilder) -> Vec<KernelDiagnostic> {
    builder.build().unwrap_err().into_parts().1
}

// ---- Cooperative workgroup tiles ------------------------------------------
//
// The structured-kernel half of the cooperative dataflow: a kernel names the
// local invocation coordinate, declares the workgroup storage its region's tile
// allocates, stages its partials, and realizes the schedule's synchronization
// point — and the verifier proves every one of those against the tile.

/// The synchronization point ordering the cooperative fixture's one handoff.
pub(super) fn cooperative_point() -> SynchronizationPoint {
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

/// The barrier spelling that realizes [`cooperative_point`].
pub(super) fn cooperative_barrier() -> BarrierSpec {
    BarrierSpec {
        point: SyncPointId::FIRST,
        execution_scope: ExecutionScope::Workgroup,
        memory_scope: MemoryScope::Workgroup,
        fenced_spaces: vec![AddressSpace::Workgroup],
        ordering: BarrierOrdering::AcquireRelease,
    }
}

/// Builds the cooperative realization of a `[2, 6] -> [2]` strict serial sum.
///
/// Three participants per workgroup, each folding two contributors into its own
/// staging slot, all three reading the staged set back, one committing.
pub(super) fn cooperative_region() -> VerifiedScheduledRegion {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
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
            scalar: ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: NumericalRealization {
                reassociation: NumericalPermission::Permitted,
                ..numerical()
            },
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
                permits_reassociation: true,
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

/// Declares the boundary signature every cooperative kernel below shares.
pub(super) fn cooperative_signature(
    builder: &mut KernelBuilder,
    scheduled: &VerifiedScheduledRegion,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 12,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 2,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder
        .admit_builtin(Builtin::LocalInvocationIndex)
        .unwrap();
    builder
        .numerical(NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..numerical()
        })
        .unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    (read, write)
}

/// The one staging allocation the cooperative region's tile declares.
pub(super) const COOPERATIVE_STAGING: StagingParameter = StagingParameter {
    staging: StagingId::FIRST,
    element_type: KernelType::F32,
    address_space: AddressSpace::Workgroup,
    element_count: 3,
};

pub(super) fn cooperative_diagnostic(builder: KernelBuilder) -> KernelDiagnostic {
    let diagnostics = builder.build().unwrap_err().diagnostics().to_vec();
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("expected exactly one diagnostic, got {diagnostics:?}")
    };
    *diagnostic
}

/// Returns the phases of every staged write and staged read, in body order.
pub(super) fn staged_accesses(kernel: &VerifiedKernel) -> (Vec<PhaseId>, Vec<PhaseId>) {
    fn walk(block: BlockRef<'_>, writes: &mut Vec<PhaseId>, reads: &mut Vec<PhaseId>) {
        for operation in block.operations() {
            match operation.view() {
                OperationView::StagedStore { phase, .. } => writes.push(phase),
                OperationView::StagedLoad { phase, .. } => reads.push(phase),
                OperationView::Predicated { body, .. } => walk(body, writes, reads),
                OperationView::SerialLoop(serial) => walk(serial.body(), writes, reads),
                _ => {}
            }
        }
    }
    let (mut writes, mut reads) = (Vec::new(), Vec::new());
    walk(kernel.body(), &mut writes, &mut reads);
    (writes, reads)
}

/// The same region with its phases run twice and its slots rewritten.
///
/// Built by re-verifying the single-round fixture's own region rather than by a
/// second literal, so the only differences are the ones the capability requires:
/// each participant now folds one contributor per round instead of two, both
/// points name the round-loop convergence derivation, and a round boundary
/// discharges the rewrite.
pub(super) fn multi_round_cooperative_region() -> VerifiedScheduledRegion {
    let mut region = cooperative_region().region().clone();
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
        .expect("the loop-carried region verifies")
}

/// Counts the binary operations a kernel body contains, nested blocks included.
///
/// The same traversal `loaded_buffers` performs, over a different operation kind:
/// a combine emitted inside the fold's serial loop is what this has to reach, and
/// a walk that stopped at the top level would count zero for every reduction.
pub(super) fn binary_op_counts(kernel: &VerifiedKernel, wanted: BinaryOp) -> usize {
    fn walk(block: BlockRef<'_>, wanted: BinaryOp, found: &mut usize) {
        for operation in block.operations() {
            match operation.view() {
                OperationView::Binary { op, .. } if op == wanted => *found += 1,
                OperationView::Predicated { body, .. } => walk(body, wanted, found),
                OperationView::SerialLoop(serial) => walk(serial.body(), wanted, found),
                _ => {}
            }
        }
    }
    let mut found = 0;
    walk(kernel.body(), wanted, &mut found);
    found
}
/// The `bf16` canonical arithmetic NaN, zero-extended into the 32-bit field.
pub(super) const BF16_NAN_BITS: u32 = crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS as u32;

pub(super) fn bf16_numerical() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-bf16",
        BF16_NAN_BITS,
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

/// The `direct` contraction of `td,od->to` over `[m, k] x [n, k] -> [m, n]`.
pub(super) fn contraction_region(id: RegionId, m: u64, n: u64, k: u64) -> VerifiedScheduledRegion {
    let left = Shape::from_dims([m, k]);
    let right = Shape::from_dims([n, k]);
    let output = Shape::from_dims([m, n]);
    let contracted = Shape::from_dims([k]);
    let output_elements = element_count(&output).unwrap();
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
        let tensor = TensorRole::Input;
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![
                        ContractionAxisSource::Output { position: free },
                        ContractionAxisSource::Contracted { position: 0 },
                    ],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: element_count(operand).unwrap(),
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
            ownership: Some(owner),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
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
            ..linear_schedule(output_elements, owner)
        })
        .unwrap();
    builder.build().unwrap()
}

pub(super) fn live_row_major_region(rows: u64) -> VerifiedScheduledRegion {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(Shape::from_dims([rows])).unwrap();
    let inner = Axis::new(1);
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajorSource { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor,
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
                kind: BoundsProofKind::LiveExtentReach,
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(scale_bias_expression(SCALE_BITS, BIAS_BITS)),
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(rows, OwnershipWitnessId::new(0)))
        .unwrap();
    builder.build().unwrap()
}

pub(super) fn operand_tile(rounds: u64) -> CooperativeTile {
    crate::schedule::blocked_operand_tile(16, rounds).expect("a 16-wide operand tile is statable")
}

/// Which axis order each operand of a cooperative contraction declares.
///
/// The vocabulary expresses four combinations at rank two — each operand is
/// either `[free, K]` or `[K, free]` — and the blocked emission once addressed
/// every one of them as if it were `[M, K]` and `[N, K]`. Naming the other three
/// is what this parameter exists for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct OperandLayouts {
    /// The left operand is `[K, M]` rather than `[M, K]`.
    pub(super) left_transposed: bool,
    /// The right operand is `[K, N]` rather than `[N, K]`.
    pub(super) right_transposed: bool,
}

/// The pair the fixtures used before any other was expressible: `[M, K]`, `[N, K]`.
pub(super) const ROW_MAJOR_OPERANDS: OperandLayouts = OperandLayouts {
    left_transposed: false,
    right_transposed: false,
};

pub(super) fn cooperative_contraction_region(
    output_m: u64,
    output_n: u64,
    contracted: u64,
    tail: TailPolicy,
) -> VerifiedScheduledRegion {
    cooperative_contraction_region_with_layouts(
        output_m,
        output_n,
        contracted,
        tail,
        ROW_MAJOR_OPERANDS,
    )
}

pub(super) fn cooperative_contraction_region_with_layouts(
    output_m: u64,
    output_n: u64,
    contracted: u64,
    tail: TailPolicy,
    layouts: OperandLayouts,
) -> VerifiedScheduledRegion {
    // A rank-two output's row axis is `0` and its column axis is `1`, which is
    // the empty-batch-prefix case of the rank-general builder below.
    let free = |position| ContractionAxisSource::Output { position };
    let inner = ContractionAxisSource::Contracted { position: 0 };
    let operand = |position, transposed| {
        if transposed {
            vec![inner, free(position)]
        } else {
            vec![free(position), inner]
        }
    };
    blocked_contraction_region(
        &Shape::from_dims([output_m, output_n]),
        contracted,
        &operand(0, layouts.left_transposed),
        &operand(1, layouts.right_transposed),
        tail,
    )
}

/// Builds a blocked cooperative contraction over an output of any rank.
///
/// The participants occupy the output's trailing two axes and the *block* takes
/// the output's rank with every leading extent one, which is the only
/// arrangement available: `MAX_COOPERATIVE_PARTICIPANT_RANK` is three, so a
/// rank-four participant space is unrepresentable rather than unimplemented.
/// A rank-two output is the batched output with no batch axes, so the fixtures
/// that predate batching reach this same builder rather than a parallel one.
///
/// **Each operand's shape is derived from the sources it declares**, not stated
/// beside them, so a fixture cannot let the two disagree: the schedule verifier
/// requires each operand extent to equal the extent of the axis its source
/// names, and a fixture that could violate that would be refused before reaching
/// the lowering these tests exercise.
pub(super) fn blocked_contraction_region(
    output: &Shape,
    contracted: u64,
    left_sources: &[ContractionAxisSource],
    right_sources: &[ContractionAxisSource],
    tail: TailPolicy,
) -> VerifiedScheduledRegion {
    let block = 16;
    let prefix = output.rank() - 2;
    let contracted_shape = Shape::from_dims([contracted]);
    // One workgroup per coordinate on each batch axis — the leading extent of
    // one — and a `16x16` tile over the trailing pair.
    let block_shape = Shape::try_new(
        std::iter::repeat_n(crate::shape::Extent::new(1), prefix)
            .chain([
                crate::shape::Extent::new(block),
                crate::shape::Extent::new(block),
            ])
            .collect::<Vec<_>>(),
    )
    .expect("a batched block is representable");
    let admitted = match tail {
        TailPolicy::Exact => {
            let admitted = crate::schedule::admit_exact_cooperative_contraction(
                output,
                &block_shape,
                &contracted_shape,
                &Shape::from_dims([block]),
            )
            .expect("exact admission");
            let elements = output
                .extents()
                .iter()
                .map(|extent| extent.get())
                .product::<u64>();
            (
                admitted.binding,
                admitted.contracted_tile,
                admitted.rounds,
                elements,
                elements,
            )
        }
        TailPolicy::Predicated => {
            let admitted = crate::schedule::admit_predicated_cooperative_contraction(
                output,
                &block_shape,
                &contracted_shape,
                &Shape::from_dims([block]),
            )
            .expect("predicated admission");
            (
                admitted.binding,
                admitted.contracted_tile,
                admitted.rounds,
                admitted.work_items,
                admitted.grid_threads,
            )
        }
    };
    let (binding, contracted_tile, rounds, work_items, grid_threads) = admitted;
    let output = output.clone();
    // The extent of whichever axis a source names, which is what makes the
    // operand shape a derivation rather than a second statement.
    let source_extent = |source: &ContractionAxisSource| match source {
        ContractionAxisSource::Output { position } => output.extents()[*position as usize].get(),
        ContractionAxisSource::Contracted { position } => {
            contracted_shape.extents()[*position as usize].get()
        }
    };
    let operand_map = |sources: &[ContractionAxisSource]| {
        let operand_shape = Shape::try_new(
            sources
                .iter()
                .map(|source| crate::shape::Extent::new(source_extent(source)))
                .collect::<Vec<_>>(),
        )
        .expect("a derived operand shape is representable");
        LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape: output.clone(),
            contracted_shape: contracted_shape.clone(),
            sources: sources.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
        }
    };
    let operand_elements =
        |sources: &[ContractionAxisSource]| sources.iter().map(source_extent).product::<u64>();
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(21));
    builder.iteration_shape(output.clone()).unwrap();
    for (witness, extent, map) in [
        (0, operand_elements(left_sources), operand_map(left_sources)),
        (
            1,
            operand_elements(right_sources),
            operand_map(right_sources),
        ),
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
                    element_count: extent,
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
                element_count: work_items,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: work_items,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: NumericalRealization::new(
                "tiler.test.strict-f32",
                NAN_BITS,
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Permitted,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        })
        .unwrap();
    let threads = u32::try_from(block * block).expect("256 fits");
    builder
        .schedule(KernelSchedule {
            binding,
            work_items,
            threads_per_workgroup: threads,
            tail,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::CooperativeContraction {
                tile: operand_tile(rounds),
                contracted_shape,
                contracted_tile,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: crate::schedule::ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    builder
        .build()
        .expect("the cooperative contraction verifies")
}

/// The element offsets round zero's two operand loads compute, for one invocation.
///
/// Interprets the *index* arithmetic of the derived body's top-level block, which
/// is exactly where the peeled round-zero operand loads sit — the round loop and
/// the committing store are nested blocks, and are deliberately not walked. The
/// offsets come back in emission order: left operand, then right.
///
/// **This reads the address the kernel will compute, not the map it was built
/// from, and nothing cheaper distinguishes the two.** A transposition is a
/// bijection of the operand's own index space, so an operand addressed by the
/// wrong layout still lands inside its buffer at a valid element: no bounds
/// proof, no element count, and no verifier can see it. Only the arithmetic can.
pub(super) fn round_zero_operand_offsets(
    scheduled: &VerifiedScheduledRegion,
    global: u64,
    local: u64,
) -> Vec<u64> {
    use super::super::model::{BinaryOp, Builtin, KernelConstant, OperationKind};
    let data = super::super::lower::derive_canonical(
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect("the canonical body exists");
    let mut values: std::collections::BTreeMap<u32, u64> = std::collections::BTreeMap::new();
    let mut offsets = Vec::new();
    let block = data.blocks.first().expect("a body has a top-level block");
    let define =
        |values: &mut std::collections::BTreeMap<u32, u64>, results: &[u32], value: u64| {
            let [result] = results else {
                panic!("an index-producing operation yields exactly one value");
            };
            values.insert(*result, value);
        };
    for operation in &block.operations {
        match &operation.kind {
            OperationKind::Builtin { builtin } => {
                let value = match builtin {
                    Builtin::GlobalInvocationIndex => global,
                    Builtin::LocalInvocationIndex => local,
                };
                define(&mut values, &operation.results, value);
            }
            OperationKind::Constant {
                value: KernelConstant::Index(value),
            } => define(&mut values, &operation.results, *value),
            OperationKind::Binary { op, lhs, rhs } => {
                // A non-index operand resolves to nothing, which is how the
                // arithmetic of the fold is skipped without naming it.
                let (Some(lhs), Some(rhs)) = (values.get(lhs).copied(), values.get(rhs).copied())
                else {
                    continue;
                };
                let value = match op {
                    BinaryOp::IndexAdd => lhs + rhs,
                    BinaryOp::IndexSubtract => lhs - rhs,
                    BinaryOp::IndexMultiply => lhs * rhs,
                    BinaryOp::IndexDivide => lhs / rhs,
                    BinaryOp::IndexModulo => lhs % rhs,
                    _ => continue,
                };
                define(&mut values, &operation.results, value);
            }
            OperationKind::Load { offset, .. } | OperationKind::GuardedLoad { offset, .. } => {
                offsets.push(*values.get(offset).expect(
                    "an operand load's offset is index arithmetic over the launch builtins",
                ));
            }
            _ => {}
        }
    }
    offsets
}

/// The addresses the *declared* maps say round zero's two loads must read.
///
/// Derived here from the layout alone rather than by calling the lowering's own
/// term builder, so the two derivations are independent and a shared mistake
/// cannot make them agree. The block geometry — participants on the output's
/// trailing pair, participant `(m, n)` fetching the left tile's column `n` and
/// the right tile's column `m` — is the tile's staging relation and is the same
/// for every layout; only the stride each coordinate carries changes.
pub(super) fn declared_operand_offsets(
    output_m: u64,
    output_n: u64,
    contracted: u64,
    layouts: OperandLayouts,
    global: u64,
    local: u64,
) -> [u64; 2] {
    let block = 16;
    let workgroups_n = output_n.div_ceil(block);
    let workgroup = global / (block * block);
    let row = (workgroup / workgroups_n) * block + local / block;
    let column = (workgroup % workgroups_n) * block + local % block;
    let left_k = local % block;
    let right_k = local / block;
    let left = if layouts.left_transposed {
        left_k * output_m + row
    } else {
        row * contracted + left_k
    };
    let right = if layouts.right_transposed {
        right_k * output_n + column
    } else {
        column * contracted + right_k
    };
    [left, right]
}

pub(super) fn guarded_load_count(kernel: &VerifiedKernel) -> usize {
    fn walk(block: BlockRef<'_>) -> usize {
        block
            .operations()
            .map(|operation| match operation.view() {
                OperationView::GuardedLoad { .. } => 1,
                OperationView::Predicated { body, .. } => walk(body),
                OperationView::SerialLoop(serial) => walk(serial.body()),
                _ => 0,
            })
            .sum()
    }
    walk(kernel.body())
}
