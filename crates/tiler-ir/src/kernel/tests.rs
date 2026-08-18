//! Structured-kernel construction, verification, and identity tests.
//!
//! Positive tests prove that the canonical lowering and an independently
//! hand-built producer kernel reach the same verified product and identity.
//! Each verification rule then has a negative test that builds a deliberately
//! wrong kernel through the public builder and asserts the exact typed
//! diagnostic, so a rejected kernel names the obligation it violated.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::*;
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, ArithmeticType, BoundsProof,
    BoundsProofKind, BoundsWitnessId, ContractionAxisSource, ContributorCoverage, ContributorOrder,
    ContributorPartition, ConvergenceEvidence, CooperativePhase, CooperativeTile,
    ExceptionalValueAssumption, ExecutionBinding, FencedSpaces, KernelSchedule, LaunchPlan,
    LocalCoordinateSource, LocalCoordinates, LogicalAccess, MemoryOrdering, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ParticipantRange,
    ParticipantSpace, PhaseId, PointwiseF32Expression, PointwiseF32ExpressionBuilder,
    ReductionPaddingIdentity, ReductionPass, ReductionTopology, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionBuilder, StagedElement, StagedRead, StagedSpan, StagedWrite,
    StagingId, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth, SubnormalMode,
    SyncPointId, SynchronizationKind, SynchronizationPlacement, SynchronizationPoint,
    SynchronizationScope, SynchronizationSubject, TailPolicy, TensorRole, VerifiedScheduledRegion,
    WorkgroupStaging, element_count,
};
/// The mutable numerical half of a cloned arithmetic region's program.
fn region_numerical_mut(
    region: &mut crate::schedule::ScheduledRegion,
) -> &mut crate::schedule::NumericalRealization {
    match &mut region.index.program {
        RegionProgram::Numerical { numerical, .. } => numerical,
        RegionProgram::PartitionedCopy(_) => panic!("the fixture region is arithmetic"),
    }
}

use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
use crate::shape::{Axis, Shape};

const NAN_BITS: u32 = 0x7fc0_0000;
const SCALE_BITS: u32 = 0x4000_0000;
const BIAS_BITS: u32 = 0x3f80_0000;
const ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX: &str = "74696c65722e6b65726e656c2e763900000000000000018474696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101010000000000000006000000010100000000310000000000000006000000010100000000000000020100030101000000000000000602000301020000000000000006000000000000000101000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010000000200000001000000000000000001010001010101010101010101000000000000000a020201030303030303030000000000000000000000000000000411010000000000000001000000001202000000000000000600000000000000010000000114010000000000000001000000000000000100000002180000000200000000000000000000000000000008160000000000000000000000000000000000000001000000031203400000000000000000000001000000041306000000030000000400000000000000010000000515010000000500000000000000010000000612033f8000000000000000000001000000071305000000060000000700000000000000010000000815010000000800000000000000010000000917000000010000000000000009000000010000000000000000000000000000000000000000";
const LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000100000000000000020000000000000002010001090000000100000000000200020900000001000000010100000000000000000000000200000000010011000000000000000000000001020011000000000000000000000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010100000000000000020000000101000000003100000000000000020000000101";
const LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX: &str = "74696c65722e6b65726e656c2e763900000000000000018474696c65722e7363686564756c652e763700000000000000000100000000000000020000000000000002010001090000000100000000000200020900000001000000010100000000000000000000000200000000010011000000000000000000000001020011000000000000000000000000020000000000000002240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101010000000000000002000000010100000000310000000000000002000000010100000000000000020100030101000000000000000002000301020000000000000000000000000000000101000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010101000000020000000100000000000000000101000101010101010101010100000000000000120202020102020202020203030303030303020000000000000000000000000000000520000000000000000000000001000000001101000000000000000100000001120200000000000000020000000000000001000000021401000000010000000200000000000000010000000318000000030000000000000000000000000000000312020000000000000000000000000000000100000004120200000000000000000000000000000001000000051f000000040000000000000000000000010000000500000000000000010000000700000000000000020000000600000007000000000000000a13020000000100000000000000000000000100000008130100000008000000060000000000000001000000091600000000000000090000000000000000000000010000000a12034000000000000000000000010000000b13060000000a0000000b00000000000000010000000c15010000000c00000000000000010000000d12033f80000000000000000000010000000e13050000000d0000000e00000000000000010000000f15010000000f00000000000000010000001017000000010000000900000010000000010000000000000000000000000000000000000001000000110000000000000000fe00000000000000010000000000000001";

fn numerical() -> NumericalRealization {
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

fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
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

fn strict_affine_u4_dequantize_region() -> VerifiedScheduledRegion {
    let logical_elements = 5;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(17));
    builder
        .iteration_shape(Shape::from_dims([logical_elements]))
        .unwrap();
    for access in [
        Access {
            tensor: TensorRole::Input,
            component_role: Some(STRICT_AFFINE_CODES_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::PackedU4LsbZeroTail { logical_elements },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input,
            component_role: Some(STRICT_AFFINE_SCALE_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(1),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input,
            component_role: Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(2),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(3),
            ownership: Some(OwnershipWitnessId::new(0)),
        },
    ] {
        builder.push_access(access).unwrap();
    }
    for (id, role, elements) in [
        (
            0,
            Some(STRICT_AFFINE_CODES_ROLE),
            logical_elements.div_ceil(2),
        ),
        (1, Some(STRICT_AFFINE_SCALE_ROLE), 1),
        (2, Some(STRICT_AFFINE_ZERO_POINT_ROLE), 1),
        (3, None, logical_elements),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(id),
                tensor: if id == 3 {
                    TensorRole::Output
                } else {
                    TensorRole::Input
                },
                component_role: role,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: logical_elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictAffineU4Dequantize {
                codes_role: STRICT_AFFINE_CODES_ROLE,
                scale_role: STRICT_AFFINE_SCALE_ROLE,
                zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(
            logical_elements,
            OwnershipWitnessId::new(0),
        ))
        .unwrap();
    builder.build().unwrap()
}

#[test]
fn strict_affine_u4_dequantize_lowers_role_addressed_packed_components() {
    let scheduled = strict_affine_u4_dequantize_region();
    let kernel = lower_scheduled_region(&scheduled).expect("exact target-neutral lowering");
    let buffers: Vec<_> = kernel.buffers().collect();
    assert_eq!(
        buffers
            .iter()
            .map(|buffer| (
                buffer.component_role,
                buffer.element_type,
                buffer.element_count
            ))
            .collect::<Vec<_>>(),
        [
            (Some(STRICT_AFFINE_CODES_ROLE), KernelType::U8, 3),
            (Some(STRICT_AFFINE_SCALE_ROLE), KernelType::F32, 1),
            (Some(STRICT_AFFINE_ZERO_POINT_ROLE), KernelType::U8, 1),
            (None, KernelType::F32, 5),
        ]
    );
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("schedule-derived guard");
    let views: Vec<_> = guarded
        .operations()
        .map(super::model::OperationRef::view)
        .collect();
    assert!(views.iter().any(|view| matches!(
        view,
        OperationView::PackedExtract {
            op: PackedExtractOp::U4LsbZeroTail,
            ..
        }
    )));
    assert!(views.iter().any(|view| matches!(
        view,
        OperationView::Binary {
            op: BinaryOp::I32Subtract,
            ..
        }
    )));
    assert_eq!(
        views
            .iter()
            .filter(|view| matches!(
                view,
                OperationView::Convert {
                    op: ConvertOp::U8ToI32,
                    ..
                }
            ))
            .count(),
        2
    );
    assert!(views.iter().any(|view| matches!(
        view,
        OperationView::Convert {
            op: ConvertOp::I32ToF32,
            ..
        }
    )));
}

#[test]
fn strict_affine_component_roles_cannot_swap() {
    let verified = strict_affine_u4_dequantize_region();
    let mut region = verified.region().clone();
    region.index.accesses.swap(0, 2);
    assert_eq!(
        ScheduledRegionBuilder::from_region(region)
            .build()
            .unwrap_err()
            .diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

#[test]
fn strict_affine_dequantization_rejects_exceptional_value_absence_assumptions() {
    let verified = strict_affine_u4_dequantize_region();
    for assumption in [
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: crate::schedule::ValueDomainProvenance::CompilerProven,
        },
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: crate::schedule::ValueDomainProvenance::RuntimeValidated,
        },
    ] {
        let mut nan_absent = verified.region().clone();
        region_numerical_mut(&mut nan_absent).nan_assumptions = assumption;
        assert_eq!(
            ScheduledRegionBuilder::from_region(nan_absent)
                .build()
                .unwrap_err()
                .diagnostics(),
            [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        let mut infinity_absent = verified.region().clone();
        region_numerical_mut(&mut infinity_absent).infinity_assumptions = assumption;
        assert_eq!(
            ScheduledRegionBuilder::from_region(infinity_absent)
                .build()
                .unwrap_err()
                .diagnostics(),
            [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }
}

fn scale_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(scale_bits).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(bias_bits).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// A pointwise scale-then-bias region over `shape`.
fn pointwise_region(id: RegionId, shape: &Shape) -> VerifiedScheduledRegion {
    pointwise_expression_region(id, shape, scale_bias_expression(SCALE_BITS, BIAS_BITS))
}

fn pointwise_expression_region(
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

/// The accepted fixed-vector map carrier is representable and verified, and
/// deliberately not executable: the canonical lowering refuses it under its
/// own rule rather than deriving a scalar body the binding does not state.
/// Everything downstream of the schedule consumes verified kernels, so this
/// one refusal is what keeps the carrier non-executable end to end while
/// lane-shaped KIR, target requirements, and the real CPU approach are absent.
#[test]
fn the_fixed_vector_map_carrier_is_refused_by_the_lowering_by_name() {
    let scheduled = pointwise_region(RegionId::new(3), &Shape::from_dims([6]));
    let mut region = scheduled.region().clone();
    region.schedule.binding = ExecutionBinding::FixedVectorMap {
        lanes: crate::schedule::VectorLaneCount::new(2).expect("an admitted lane width"),
    };
    region.schedule.launch.grid_threads = 3;
    let vectored = ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("the accepted carrier passes intrinsic verification");
    let error = lower_scheduled_region(&vectored).unwrap_err();
    assert!(
        matches!(
            error,
            KernelLoweringError::Verification(KernelDiagnostic::UnloweredExecutionBinding)
        ),
        "expected the named unlowered-binding refusal, got {error:?}"
    );
}

/// The approved `(a * b) + c` region over three distinct input tensors.
fn three_input_region(elements: u64) -> VerifiedScheduledRegion {
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
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(3),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
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
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder.build().unwrap()
}

/// An epilogue-shaped access list: one staged read, one declared-input read,
/// then the owning output write.
fn mixed_epilogue_region(elements: u64) -> VerifiedScheduledRegion {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let staged = expression.input(AccessOrdinal::FIRST).unwrap();
    let input = expression.input(AccessOrdinal::new(1)).unwrap();
    let root = expression.add(staged, input).unwrap();
    let expression = expression.build(root).unwrap();

    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder
        .iteration_shape(Shape::from_dims([elements]))
        .unwrap();
    for (position, tensor, mode, ownership) in [
        (0, TensorRole::Intermediate, AccessMode::Read, None),
        (1, TensorRole::Input, AccessMode::Read, None),
        (
            2,
            TensorRole::Output,
            AccessMode::Write,
            Some(OwnershipWitnessId::new(0)),
        ),
    ] {
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(position),
                ownership,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(position),
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
            tensor: TensorRole::Output,
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

/// A three-input region lowers to a four-buffer kernel that loads each input.
///
/// The loads are checked against the *buffers they address*, not merely counted:
/// the leaf ordinal is what chooses among the loaded values, and a lowering that
/// reused one load for every leaf would still emit the right operation count.
#[test]
fn a_three_input_pointwise_region_lowers_to_one_buffer_and_load_per_input() {
    let scheduled = three_input_region(4);
    let kernel = lower_scheduled_region(&scheduled).expect("the widened region lowers");

    let declared: Vec<_> = kernel.declared_buffers().collect();
    assert_eq!(declared.len(), 4);
    for (_, parameter) in declared.iter().take(3) {
        assert_eq!(parameter.tensor, TensorRole::Input);
        assert_eq!(parameter.access, BufferAccess::Read);
        assert_eq!(parameter.element_type, KernelType::F32);
    }
    assert_eq!(declared[3].1.tensor, TensorRole::Output);
    assert_eq!(declared[3].1.access, BufferAccess::Write);

    // Every read buffer is loaded, in declaration order, exactly once — so no
    // input is bound and then never read, and none is read twice in another's
    // place.
    let expected: Vec<_> = declared.iter().take(3).map(|(id, _)| *id).collect();
    assert_eq!(loaded_buffers(&kernel), expected);
}

/// Returns, in body order, the buffer each load in the kernel addresses.
fn loaded_buffers(kernel: &VerifiedKernel) -> Vec<VerifiedBufferId> {
    fn walk(block: BlockRef<'_>, loads: &mut Vec<VerifiedBufferId>) {
        for operation in block.operations() {
            match operation.view() {
                OperationView::Load { buffer, .. } | OperationView::GuardedLoad { buffer, .. } => {
                    loads.push(buffer);
                }
                OperationView::Predicated { body, .. } => walk(body, loads),
                OperationView::SerialLoop(serial) => walk(serial.body(), loads),
                OperationView::Builtin { .. }
                | OperationView::Constant { .. }
                | OperationView::Binary { .. }
                | OperationView::Compare { .. }
                | OperationView::Convert { .. }
                | OperationView::Unary { .. }
                | OperationView::PackedExtract { .. }
                | OperationView::Store { .. }
                | OperationView::StagedStore { .. }
                | OperationView::StagedLoad { .. }
                | OperationView::Barrier { .. }
                | OperationView::InputExtent { .. } => {}
            }
        }
    }
    let mut loads = Vec::new();
    walk(kernel.body(), &mut loads);
    loads
}

#[test]
fn pointwise_lowering_preserves_left_and_right_operands_and_canonicalizes_each_operation() {
    let expressions = [
        {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let input = expression.input(AccessOrdinal::FIRST).unwrap();
            let two = expression.constant(2.0_f32.to_bits()).unwrap();
            let sum = expression.add(input, two).unwrap();
            let three = expression.constant(3.0_f32.to_bits()).unwrap();
            let root = expression.multiply(sum, three).unwrap();
            expression.build(root).unwrap()
        },
        {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let input = expression.input(AccessOrdinal::FIRST).unwrap();
            let two = expression.constant(2.0_f32.to_bits()).unwrap();
            let sum = expression.add(two, input).unwrap();
            let three = expression.constant(3.0_f32.to_bits()).unwrap();
            let root = expression.multiply(three, sum).unwrap();
            expression.build(root).unwrap()
        },
    ];

    for (position, expression) in expressions.into_iter().enumerate() {
        let scheduled = pointwise_expression_region(
            RegionId::new(u32::try_from(position).unwrap()),
            &Shape::from_dims([4]),
            expression,
        );
        let kernel = lower_scheduled_region(&scheduled).unwrap();
        let guarded = kernel
            .body()
            .operations()
            .find_map(|operation| match operation.view() {
                OperationView::Predicated { body, .. } => Some(body),
                _ => None,
            })
            .unwrap();
        let operations: Vec<_> = guarded.operations().map(OperationRef::view).collect();
        let binaries: Vec<_> = operations
            .iter()
            .filter_map(|operation| match operation {
                OperationView::Binary { op, lhs, rhs } => Some((*op, *lhs, *rhs)),
                _ => None,
            })
            .collect();
        assert_eq!(
            binaries.iter().map(|(op, _, _)| *op).collect::<Vec<_>>(),
            [BinaryOp::F32Add, BinaryOp::F32Multiply]
        );
        assert_eq!(
            operations
                .iter()
                .filter(|operation| matches!(
                    operation,
                    OperationView::Convert {
                        op: ConvertOp::CanonicalizeF32Nan,
                        ..
                    }
                ))
                .count(),
            2
        );
        let (_, add_lhs, add_rhs) = binaries[0];
        let (_, multiply_lhs, multiply_rhs) = binaries[1];
        if position == 0 {
            assert_eq!(kernel.value_constant(add_lhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(add_rhs).unwrap(),
                Some(KernelConstant::F32Bits(2.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(multiply_lhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(multiply_rhs).unwrap(),
                Some(KernelConstant::F32Bits(3.0_f32.to_bits()))
            );
        } else {
            assert_eq!(
                kernel.value_constant(add_lhs).unwrap(),
                Some(KernelConstant::F32Bits(2.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(add_rhs).unwrap(), None);
            assert_eq!(
                kernel.value_constant(multiply_lhs).unwrap(),
                Some(KernelConstant::F32Bits(3.0_f32.to_bits()))
            );
            assert_eq!(kernel.value_constant(multiply_rhs).unwrap(), None);
        }
    }
}

/// A strict serial sum over `axes` of `input`.
fn reduction_region(id: RegionId, input: &Shape, axes: &[Axis]) -> VerifiedScheduledRegion {
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
fn pointwise_signature(
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
fn guard(builder: &mut KernelBuilder, work_items: u64) -> (KernelValueId, KernelValueId) {
    let invocation = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let extent = builder.constant(KernelConstant::Index(work_items)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, invocation, extent)
        .unwrap();
    (invocation, active)
}

/// Emits the canonical scale-then-bias arithmetic with its NaN normalizations.
fn scale_bias(builder: &mut KernelBuilder, loaded: KernelValueId) -> KernelValueId {
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
fn canonical_pointwise(scheduled: &VerifiedScheduledRegion, elements: u64) -> KernelBuilder {
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

/// Builds the same verified pointwise kernel under a prospective subgroup
/// requirement. No admitted schedule derives a present requirement yet, so the
/// test has to replace both copies of that derived fact before running the real
/// refinement verifier; every other kernel field still comes through the public
/// producer path above.
fn pointwise_with_subgroup_requirement(
    scheduled: &VerifiedScheduledRegion,
    subject: SubgroupRealizationSubject,
) -> VerifiedKernel {
    let mut requirements = scheduled.requirements();
    requirements.subgroup = Some(subject);
    let mut builder = KernelBuilder::from_parts(
        scheduled.region().clone(),
        scheduled.canonical_identity().clone(),
        requirements,
    )
    .unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(requirements).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
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
        .build()
        .expect("the prospective subgroup requirement is identity-bearing")
}

fn diagnostics(builder: KernelBuilder) -> Vec<KernelDiagnostic> {
    builder.build().unwrap_err().into_parts().1
}

#[test]
fn canonical_lowering_produces_a_verified_backend_consumable_kernel() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    assert_eq!(kernel.scheduled_region(), RegionId::new(0));
    assert_eq!(
        kernel.scheduled_region_identity(),
        scheduled.canonical_identity()
    );
    assert_eq!(kernel.numerical(), numerical());
    assert_eq!(kernel.requirements(), scheduled.requirements());
    assert_eq!(kernel.admitted_builtins(), [Builtin::GlobalInvocationIndex]);

    let buffers: Vec<_> = kernel.buffers().collect();
    assert_eq!(
        buffers,
        [
            BufferParameter {
                tensor: TensorRole::Input,
                component_role: None,
                element_type: KernelType::F32,
                address_space: AddressSpace::Device,
                access: BufferAccess::Read,
                element_count: 6,
            },
            BufferParameter {
                tensor: TensorRole::Intermediate,
                component_role: None,
                element_type: KernelType::F32,
                address_space: AddressSpace::Device,
                access: BufferAccess::Write,
                element_count: 6,
            },
        ]
    );

    // Every effect is lexically inside one explicit bounds-predicated region.
    let guarded: Vec<_> = kernel
        .body()
        .operations()
        .filter_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .collect();
    assert_eq!(guarded.len(), 1);
    let effects = guarded[0]
        .operations()
        .filter(|operation| {
            matches!(
                operation.view(),
                OperationView::Load { .. } | OperationView::Store { .. }
            )
        })
        .count();
    assert_eq!(effects, 2);
}

/// Compile-time tripwire for `revisit-kernel-body-single-spelling-gate`.
///
/// The refinement gate re-derives the canonical body with
/// `lower::derive_canonical` — a deterministic function of the scheduled region
/// — and requires structural equality, so the profile admits **exactly one
/// spelling** of a legal kernel. That is correct only while the surface is
/// narrow enough that no two genuinely different bodies are both legal for one
/// region. Past that point derive-and-compare starts rejecting *valid* kernels.
///
/// The ticket names that widening as its trigger for reconsideration, and a
/// trigger nobody is told about is a trigger nobody notices. These matches are
/// exhaustive with no wildcard arm, so adding a variant to any of the closed
/// vocabularies that decide a body's shape is a **compile error here**. Whoever
/// hits it should read that ticket before adding an arm: the fix may be to widen
/// the gate rather than to widen this match.
///
/// Deliberately a spelling check, not a semantic one — it cannot tell that a
/// widened vocabulary admits two bodies, only that the vocabulary widened, which
/// is the point at which a human has to look.
fn body_shaping_vocabulary_is_closed(
    binding: &ExecutionBinding,
    tail: TailPolicy,
    access: &LogicalAccess,
    topology: &ReductionTopology,
    program: &ScalarProgram,
) -> (
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
) {
    (
        match binding {
            ExecutionBinding::GlobalLinearInvocation => "global-linear-invocation",
            ExecutionBinding::BlockedWorkgroup { .. } => "blocked-workgroup",
            // Widens the vocabulary without widening the *lowered* profile:
            // `plan` refuses this binding as `unlowered-execution-binding`
            // before any body is derived, so no scheduled region carrying it
            // has even one legal body, let alone two. Trigger re-checked and
            // not fired; see `revisit-kernel-body-single-spelling-gate`'s
            // 2026-08-18 log entry.
            ExecutionBinding::FixedVectorMap { .. } => "fixed-vector-map",
        },
        match tail {
            TailPolicy::Exact => "exact",
            TailPolicy::Predicated => "predicated",
        },
        match access {
            LogicalAccess::LinearIdentity => "linear-identity",
            LogicalAccess::ScalarBroadcast => "scalar-broadcast",
            LogicalAccess::PackedU4LsbZeroTail { .. } => "packed-u4-lsb-zero-tail",
            LogicalAccess::ReductionContributor { .. } => "reduction-contributor",
            LogicalAccess::ContractionOperand { .. } => "contraction-operand",
            LogicalAccess::ReindexBijection { .. } => "reindex-bijection",
            LogicalAccess::BroadcastReplication { .. } => "broadcast-replication",
            LogicalAccess::ParametricBroadcast { .. } => "parametric-broadcast",
            LogicalAccess::LiveRowMajor { .. } => "live-row-major",
            LogicalAccess::PartitionedCopySource => "partitioned-copy-source",
        },
        match topology {
            ReductionTopology::None => "none",
            ReductionTopology::Serial { .. } => "serial",
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                ..
            } => "multi-pass-partial",
            ReductionTopology::MultiPass {
                pass: ReductionPass::Final,
                ..
            } => "multi-pass-final",
            ReductionTopology::Contraction { .. } => "contraction",
            ReductionTopology::LiveContraction { .. } => "live-contraction",
            ReductionTopology::CooperativeWorkgroup { .. } => "cooperative-workgroup",
            ReductionTopology::CooperativeContraction { .. } => "cooperative-contraction",
        },
        match program {
            ScalarProgram::PointwiseF32(_) => "pointwise-f32",
            ScalarProgram::PointwiseBf16(_) => "pointwise-bf16",
            ScalarProgram::StrictAffineU4Dequantize { .. } => "strict-affine-u4-dequantize",
            ScalarProgram::StrictSerialSum { .. } => "strict-serial-sum",
            ScalarProgram::FusedMultiplyAddSerialSum { .. } => "fused-multiply-add-serial-sum",
            ScalarProgram::SquaredSerialSum { .. } => "squared-serial-sum",
            ScalarProgram::SquaredSerialSumThenEpilogue { .. } => "squared-serial-sum-epilogue",
            ScalarProgram::StrictTensorContraction { .. } => "strict-tensor-contraction",
            ScalarProgram::StrictSerialMaximum { .. } => "strict-serial-maximum",
        },
    )
}

/// The single-spelling gate's precondition still holds.
///
/// One execution binding and one tail policy is the substance of it: with a
/// single way to bind invocations to coordinates and no tail to handle, a
/// scheduled region's body has no legal degree of freedom for a producer to
/// spell differently. See [`body_shaping_vocabulary_is_closed`].
#[test]
fn the_single_spelling_profile_is_still_narrow_enough_for_derive_and_compare() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let region = scheduled.region();
    let names = body_shaping_vocabulary_is_closed(
        &region.schedule.binding,
        region.schedule.tail,
        &region.index.accesses[0].map,
        &region.schedule.reduction,
        match &region.index.program {
            RegionProgram::Numerical { scalar, .. } => scalar,
            RegionProgram::PartitionedCopy(_) => panic!("the fixture region is arithmetic"),
        },
    );
    assert_eq!(
        names,
        (
            "global-linear-invocation",
            "exact",
            "linear-identity",
            "none",
            "pointwise-f32",
        )
    );
}

/// Compile-time tripwire for `add-subgroup-memory-scope-when-collectives-land`.
///
/// A barrier states its execution scope and its memory scope as two separate
/// vocabularies (ADR 0048), and the pair is asymmetric today: [`ExecutionScope`]
/// names a subgroup and [`MemoryScope`] cannot, so subgroup-level visibility is
/// inexpressible and `tiler-metal` refuses every subgroup barrier rather than
/// widening its claim to workgroup visibility. That deferred ticket owns closing
/// the asymmetry.
///
/// Widening either enum is *already* a build error inside this crate: `tag` on
/// each enum and `verify::barrier_subject` are exhaustive, and
/// `#[non_exhaustive]` has no effect on a match in the defining crate. What
/// neither of those errors says is what happens downstream. `barrier_call` in
/// `crates/tiler-metal/src/emit.rs` matches both scopes with wildcard arms, so a
/// widened scope compiles there and every barrier naming it keeps being rejected
/// at run time with a typed `UnsupportedBarrier`. Those wildcards are correct and
/// stay — out of crate `#[non_exhaustive]` requires one, and they are what makes
/// an unhandled scope a typed rejection rather than a panic. This match is the
/// break that carries the instructions: whoever hits it should read that ticket
/// before adding an arm.
///
/// The two scope vocabularies only. [`BarrierOrdering`] and [`AddressSpace`] are
/// wildcarded in the same emitter, but no ticket owns widening either, and a
/// tripwire that names no owner is a build error with nothing to say.
///
/// Deliberately a spelling check and not a semantic one, exactly as
/// [`body_shaping_vocabulary_is_closed`] states: it cannot tell that a widened
/// vocabulary admits a new barrier, only that the vocabulary widened, which is
/// the point at which a human has to look. It cites constructs rather than
/// lines, because every line citation this tripwire inherited had drifted by the
/// time it was read.
fn barrier_scope_vocabulary_is_closed(
    execution: ExecutionScope,
    memory: MemoryScope,
) -> (&'static str, &'static str) {
    (
        match execution {
            ExecutionScope::Subgroup => "subgroup",
            ExecutionScope::Workgroup => "workgroup",
        },
        match memory {
            MemoryScope::Workgroup => "workgroup",
            MemoryScope::Device => "device",
        },
    )
}

/// The barrier scope vocabularies are still the pair the backends were built for.
///
/// Consumes the spelling of a real cooperative handoff rather than literals, so
/// the tripwire is anchored to a barrier the profile actually emits.
#[test]
fn the_barrier_scope_vocabularies_are_still_closed() {
    let spec = cooperative_barrier();
    assert_eq!(
        barrier_scope_vocabulary_is_closed(spec.execution_scope, spec.memory_scope),
        ("workgroup", "workgroup")
    );
}

/// Collects every buffer handle a block's effects reference, descending into
/// predicated bodies.
fn referenced_buffers(block: BlockRef<'_>) -> Vec<VerifiedBufferId> {
    let mut found = Vec::new();
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { buffer, .. }
            | OperationView::GuardedLoad { buffer, .. }
            | OperationView::Store { buffer, .. } => {
                found.push(buffer);
            }
            OperationView::Predicated { body, .. } => found.extend(referenced_buffers(body)),
            _ => {}
        }
    }
    found
}

/// The buffer handles a body references recover the signature, in handle order.
///
/// This is evidence for `pair-verified-buffer-handles-with-signature-ordinals`,
/// not a public guarantee. A backend that must emit an argument-table index per
/// load and store can today only recover the pairing by *sorting handles*,
/// which works solely because a verified handle is `(owner, index)` and every
/// handle of one kernel shares an owner — a private representation detail the
/// derived `Ord` exposes and no contract promises.
///
/// The test pins that the fact is already true, so publishing it is exposing an
/// invariant rather than computing a new one. It deliberately does not assert a
/// *position*, because no public accessor yields one; that is precisely the gap
/// the ticket asks the IR to close.
#[test]
fn referenced_buffer_handles_recover_the_signature_in_handle_order() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    let mut referenced = referenced_buffers(kernel.body());
    referenced.sort_unstable();
    referenced.dedup();

    // Every signature parameter is referenced exactly once by this lowering, so
    // the recovered sequence is the whole signature rather than a prefix of it.
    assert_eq!(referenced.len(), kernel.buffers().len());
    let recovered: Vec<_> = referenced
        .iter()
        .map(|id| kernel.buffer(*id).unwrap())
        .collect();
    assert_eq!(recovered, kernel.buffers().collect::<Vec<_>>());

    // A handle from another kernel is rejected rather than silently resolving
    // to the same ordinal, which is what makes the pairing kernel-scoped.
    let other = lower_scheduled_region(&pointwise_region(
        RegionId::new(1),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    assert!(matches!(
        other.buffer(referenced[0]),
        Err(VerifiedKernelHandleError::ForeignKernel { .. })
    ));
}

#[test]
fn a_producer_built_canonical_kernel_verifies_and_equals_the_lowering() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let produced = canonical_pointwise(&scheduled, 6).build().unwrap();
    let lowered = lower_scheduled_region(&scheduled).unwrap();
    assert_eq!(produced, lowered);
    assert_eq!(
        produced.canonical_identity().as_bytes(),
        lowered.canonical_identity().as_bytes()
    );
}

#[test]
fn identity_is_independent_of_planning_ordinals_and_separates_content() {
    let first = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    let renumbered = lower_scheduled_region(&pointwise_region(
        RegionId::new(7),
        &Shape::from_dims([2, 3]),
    ))
    .unwrap();
    assert_ne!(first.scheduled_region(), renumbered.scheduled_region());
    assert_eq!(
        first.canonical_identity().as_bytes(),
        renumbered.canonical_identity().as_bytes()
    );

    let wider = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 4]),
    ))
    .unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        wider.canonical_identity().as_bytes()
    );

    // A kernel identity separates two regions that differ only in schedule.
    let reduction = lower_scheduled_region(&reduction_region(
        RegionId::new(0),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    ))
    .unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        reduction.canonical_identity().as_bytes()
    );
}

/// A present subgroup requirement appends one self-contained identity subject.
///
/// The ordinary lowering is the absence control: it still ends at the exact
/// pre-subgroup identity bytes. Each constructible subject dimension then moves
/// the whole identity independently. The final byte is the transfer tag, which
/// pins its governed position without pretending that a second typed transfer
/// exists to perturb.
#[test]
fn subgroup_requirement_is_append_only_and_identity_bearing() {
    fn appended_subject<'a>(kernel: &'a VerifiedKernel, absent: &[u8]) -> &'a [u8] {
        kernel
            .canonical_identity()
            .as_bytes()
            .strip_prefix(absent)
            .expect("a present subgroup requirement only appends to the absent identity")
    }

    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let absent = canonical_pointwise(&scheduled, 6).build().unwrap();
    assert_eq!(absent.requirements().subgroup, None);

    let subject = |lanes, arithmetic| {
        SubgroupRealizationSubject::new(
            SubgroupWidth::new(lanes).unwrap(),
            arithmetic,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .unwrap()
    };
    let required =
        pointwise_with_subgroup_requirement(&scheduled, subject(32, ArithmeticType::F32));
    let wider = pointwise_with_subgroup_requirement(&scheduled, subject(64, ArithmeticType::F32));
    let bf16 = pointwise_with_subgroup_requirement(&scheduled, subject(32, ArithmeticType::Bf16));

    let absent_bytes = absent.canonical_identity().as_bytes();
    assert_eq!(
        appended_subject(&required, absent_bytes),
        [0x01, 0x00, 0x00, 0x00, 0x20, 0x03, 0x01],
        "presence, width, arithmetic, and transfer append in governed order"
    );
    assert_eq!(
        appended_subject(&wider, absent_bytes),
        [0x01, 0x00, 0x00, 0x00, 0x40, 0x03, 0x01],
        "width must move the prospective kernel identity"
    );
    assert_eq!(
        appended_subject(&bf16, absent_bytes),
        [0x01, 0x00, 0x00, 0x00, 0x20, 0x02, 0x01],
        "arithmetic must move the prospective kernel identity"
    );

    let mut absent_hex = String::with_capacity(absent_bytes.len().saturating_mul(2));
    for byte in absent_bytes {
        write!(&mut absent_hex, "{byte:02x}").unwrap();
    }
    assert_eq!(
        absent_hex, ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX,
        "adding a conditional subgroup suffix must not move the absent kernel pin"
    );
}

#[test]
fn a_reduction_lowers_to_a_bounded_loop_carrying_one_accumulator() {
    let scheduled = reduction_region(RegionId::new(1), &Shape::from_dims([2, 3]), &[Axis::new(1)]);
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded region");
    let reduction = guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a bounded reduction loop");
    assert_eq!((reduction.start(), reduction.end()), (1, 3));
    assert_eq!(reduction.initial().len(), 1);
    assert_eq!(reduction.accumulators().len(), 1);
    assert_eq!(reduction.yields().len(), 1);
    assert_eq!(
        kernel
            .value_type(reduction.accumulators().next().unwrap())
            .unwrap(),
        KernelType::F32
    );
    assert_eq!(
        kernel.value_type(reduction.induction().unwrap()).unwrap(),
        KernelType::Index
    );
}

#[test]
fn buffer_contract_rejects_a_signature_that_misstates_the_scheduled_access() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 7,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
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
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BufferContract]);
}

#[test]
fn address_space_contract_rejects_a_space_the_schedule_does_not_provide() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Workgroup,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
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
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::AddressSpaceContract]
    );
}

#[test]
fn builtin_contract_rejects_a_kernel_that_never_admits_the_execution_binding() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let position = builder.constant(KernelConstant::Index(0)).unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, position, extent)
        .unwrap();
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, position, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                position,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BuiltinContract]);
}

#[test]
fn numerical_and_resource_declarations_must_equal_the_schedule() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut drifted = KernelBuilder::new(&scheduled).unwrap();
    let read = drifted
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = drifted
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    drifted
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    let mut wrong = numerical();
    wrong.canonical_arithmetic_nan_bits ^= 1;
    drifted.numerical(wrong).unwrap();
    drifted.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut drifted, 6);
    drifted
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
    assert_eq!(
        diagnostics(drifted),
        [KernelDiagnostic::NumericalRealization]
    );

    let mut inflated = KernelBuilder::new(&scheduled).unwrap();
    let read = inflated
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    let write = inflated
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    inflated
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    inflated.numerical(numerical()).unwrap();
    let mut requirements = scheduled.requirements();
    requirements.local_memory_bytes += 1;
    inflated.requirements(requirements).unwrap();
    let (invocation, active) = guard(&mut inflated, 6);
    inflated
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
    assert_eq!(
        diagnostics(inflated),
        [KernelDiagnostic::ResourceRequirements]
    );
}

#[test]
fn predicate_dominance_rejects_unguarded_and_ungoverned_effects() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    // No predicate at all: the effects are not dominated by bounds evidence.
    let mut unguarded = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut unguarded, &scheduled, 6);
    let invocation = unguarded.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let loaded = unguarded
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    let value = scale_bias(&mut unguarded, loaded);
    unguarded
        .store(
            write,
            invocation,
            value,
            BoundsWitnessId::new(1),
            OwnershipWitnessId::new(0),
        )
        .unwrap();
    assert_eq!(
        diagnostics(unguarded),
        [KernelDiagnostic::PredicateDominance]
    );

    // A predicate that is not the scheduled bounds predicate is also rejected.
    let mut ungoverned = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut ungoverned, &scheduled, 6);
    let invocation = ungoverned.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let wrong_extent = ungoverned.constant(KernelConstant::Index(9)).unwrap();
    let active = ungoverned
        .compare(CompareOp::IndexLessThan, invocation, wrong_extent)
        .unwrap();
    ungoverned
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
    assert_eq!(
        diagnostics(ungoverned),
        [KernelDiagnostic::PredicateDominance]
    );
}

#[test]
fn bounds_and_ownership_evidence_must_be_the_scheduled_witnesses() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut swapped = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut swapped, &scheduled, 6);
    let (invocation, active) = guard(&mut swapped, 6);
    swapped
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(0),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(swapped), [KernelDiagnostic::BoundsEvidence]);

    let mut disowned = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut disowned, &scheduled, 6);
    let (invocation, active) = guard(&mut disowned, 6);
    disowned
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(9),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(disowned), [KernelDiagnostic::OwnershipEvidence]);
}

#[test]
fn output_coverage_requires_exactly_one_owning_commit() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    let mut silent = KernelBuilder::new(&scheduled).unwrap();
    let (read, _write) = pointwise_signature(&mut silent, &scheduled, 6);
    let (invocation, active) = guard(&mut silent, 6);
    silent
        .predicated(active, |builder| {
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(diagnostics(silent), [KernelDiagnostic::OutputCoverage]);

    let mut doubled = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut doubled, &scheduled, 6);
    let (invocation, active) = guard(&mut doubled, 6);
    doubled
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )?;
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(doubled), [KernelDiagnostic::OutputCoverage]);
}

#[test]
fn effect_ordering_requires_the_owning_commit_to_be_last() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
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
            )?;
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::EffectOrdering]);
}

#[test]
fn a_barrier_the_schedule_does_not_require_is_rejected_explicitly() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            builder.barrier(BarrierSpec {
                point: SyncPointId::FIRST,
                execution_scope: ExecutionScope::Workgroup,
                memory_scope: MemoryScope::Device,
                fenced_spaces: vec![AddressSpace::Device],
                ordering: BarrierOrdering::AcquireRelease,
            })?;
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
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::UnexpectedSynchronization]
    );
}

#[test]
fn reduction_contract_requires_the_scheduled_contributor_loop() {
    let scheduled = reduction_region(RegionId::new(1), &Shape::from_dims([2, 3]), &[Axis::new(1)]);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
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
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 2);
    // Commit only the first contributor: structurally well formed, but it does
    // not realize the scheduled three-contributor serial reduction.
    builder
        .predicated(active, |builder| {
            let stride = builder.constant(KernelConstant::Index(3)).unwrap();
            let base = builder.binary(BinaryOp::IndexMultiply, invocation, stride)?;
            let loaded = builder.load(read, base, BoundsWitnessId::new(0))?;
            builder.store(
                write,
                invocation,
                loaded,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::ReductionContract]);
}

#[test]
fn body_refinement_rejects_a_structurally_legal_but_non_canonical_body() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    // Every structural obligation holds, but the numerical contract's NaN
    // normalization is missing after each arithmetic step.
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let scale = builder.constant(KernelConstant::F32Bits(SCALE_BITS))?;
            let product = builder.binary(BinaryOp::F32Multiply, loaded, scale)?;
            let bias = builder.constant(KernelConstant::F32Bits(BIAS_BITS))?;
            let value = builder.binary(BinaryOp::F32Add, product, bias)?;
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    assert_eq!(diagnostics(builder), [KernelDiagnostic::BodyRefinement]);
}

#[test]
fn an_incomplete_kernel_names_its_missing_component() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let builder = KernelBuilder::new(&scheduled).unwrap();
    assert_eq!(
        diagnostics(builder),
        [KernelDiagnostic::IncompleteKernel {
            component: KernelComponent::NumericalRealization,
        }]
    );
}

#[test]
fn a_rejected_kernel_returns_its_builder_intact_for_amend_and_retry() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            builder.load(read, invocation, BoundsWitnessId::new(0))?;
            Ok(())
        })
        .unwrap();
    let (mut recovered, diagnostics) = builder.build().unwrap_err().into_parts();
    assert_eq!(diagnostics, [KernelDiagnostic::OutputCoverage]);
    // The recovered builder still owns its buffers and values, so the caller can
    // append the missing commit instead of restarting construction.
    assert_eq!(recovered.derived_requirements(), scheduled.requirements());
    assert_eq!(
        recovered.store(
            write,
            invocation,
            invocation,
            BoundsWitnessId::new(1),
            OwnershipWitnessId::new(0),
        ),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Index,
        })
    );
}

#[test]
fn a_handle_from_another_builder_or_kernel_is_rejected() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut first = KernelBuilder::new(&scheduled).unwrap();
    let (first_read, _) = pointwise_signature(&mut first, &scheduled, 6);
    let (first_invocation, _) = guard(&mut first, 6);

    let mut second = KernelBuilder::new(&scheduled).unwrap();
    let (second_read, _) = pointwise_signature(&mut second, &scheduled, 6);
    let (second_invocation, _) = guard(&mut second, 6);

    assert_eq!(
        second.load(first_read, second_invocation, BoundsWitnessId::new(0)),
        Err(KernelBuildError::ForeignHandle {
            entity: KernelEntityKind::Buffer,
        })
    );
    assert_eq!(
        second.load(second_read, first_invocation, BoundsWitnessId::new(0)),
        Err(KernelBuildError::ForeignHandle {
            entity: KernelEntityKind::Value,
        })
    );

    let owner = lower_scheduled_region(&scheduled).unwrap();
    let foreign = lower_scheduled_region(&scheduled).unwrap();
    let value = owner
        .body()
        .operations()
        .next()
        .expect("a first operation")
        .results()
        .next()
        .expect("a defined result");
    assert_eq!(owner.value_type(value), Ok(KernelType::Index));
    assert_eq!(
        foreign.value_type(value),
        Err(VerifiedKernelHandleError::ForeignKernel {
            entity: KernelEntityKind::Value,
        })
    );
}

#[test]
fn a_value_defined_in_a_closed_nested_block_leaves_scope() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
    let escaped: Cell<Option<KernelValueId>> = Cell::new(None);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let value = scale_bias(builder, loaded);
            escaped.set(Some(value));
            builder.store(
                write,
                invocation,
                value,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let escaped = escaped.get().expect("a value defined inside the guard");
    assert_eq!(
        builder.convert(ConvertOp::CanonicalizeF32Nan, escaped),
        Err(KernelBuildError::ValueOutOfScope)
    );
}

#[test]
fn locally_decidable_operand_and_signature_rules_reject_at_insertion() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    assert_eq!(
        builder.builtin(Builtin::GlobalInvocationIndex),
        Err(KernelBuildError::UndeclaredBuiltin)
    );
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    assert_eq!(
        builder.admit_builtin(Builtin::GlobalInvocationIndex),
        Err(KernelBuildError::DuplicateAdmittedBuiltin)
    );
    assert_eq!(
        builder.numerical(numerical()),
        Err(KernelBuildError::ComponentAlreadySet {
            component: KernelComponent::NumericalRealization,
        })
    );
    assert_eq!(
        builder.requirements(scheduled.requirements()),
        Err(KernelBuildError::ComponentAlreadySet {
            component: KernelComponent::ResourceRequirements,
        })
    );

    let (invocation, _) = guard(&mut builder, 6);
    assert_eq!(
        builder.binary(BinaryOp::F32Add, invocation, invocation),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Index,
        })
    );
    assert_eq!(
        builder.load(write, invocation, BoundsWitnessId::new(1)),
        Err(KernelBuildError::BufferAccessViolation)
    );
    let loaded = builder
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    assert_eq!(
        builder.store(
            read,
            invocation,
            loaded,
            BoundsWitnessId::new(0),
            OwnershipWitnessId::new(0),
        ),
        Err(KernelBuildError::BufferAccessViolation)
    );
    assert_eq!(
        builder.binary(BinaryOp::IndexDivide, invocation, invocation),
        Err(KernelBuildError::NonConstantDivisor)
    );
    let zero = builder.constant(KernelConstant::Index(0)).unwrap();
    assert_eq!(
        builder.binary(BinaryOp::IndexModulo, invocation, zero),
        Err(KernelBuildError::NonPositiveDivisor)
    );
}

#[test]
fn structured_loop_shape_and_yields_are_checked_at_insertion() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, _write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, _) = guard(&mut builder, 6);
    let seed = builder
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();

    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 2, end: 2 }, &[seed], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::InvalidLoopRange { start: 2, end: 2 }
    );
    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 0, end: 3 }, &[], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::EmptyLoopAccumulators
    );
    assert_eq!(
        builder
            .serial_loop(SerialLoopSpec { start: 0, end: 3 }, &[seed], |_, _| Ok(
                Vec::new()
            ))
            .unwrap_err(),
        KernelBuildError::LoopYieldArity {
            expected: 1,
            actual: 0,
        }
    );
    assert_eq!(
        builder
            .serial_loop(
                SerialLoopSpec { start: 0, end: 3 },
                &[seed],
                |_, parameters| Ok(vec![parameters.induction()]),
            )
            .unwrap_err(),
        KernelBuildError::LoopYieldTypeMismatch {
            position: 0,
            expected: KernelType::F32,
            actual: KernelType::Index,
        }
    );

    // Every failed nested insertion left the builder exactly as it was, so the
    // canonical body can still be completed and verified afterwards.
    let canonical = canonical_pointwise(&scheduled, 6).build().unwrap();
    assert_eq!(canonical, lower_scheduled_region(&scheduled).unwrap());
}

#[test]
fn diagnostics_and_errors_expose_stable_rule_identifiers() {
    assert_eq!(KernelDiagnostic::BodyRefinement.rule(), "body-refinement");
    assert_eq!(
        KernelDiagnostic::UnexpectedSynchronization.rule(),
        "unexpected-synchronization"
    );
    assert_eq!(
        KernelLoweringError::Verification(KernelDiagnostic::OutputCoverage).rule(),
        "output-coverage"
    );
    assert_eq!(
        KernelLoweringError::Construction(KernelBuildError::ValueOutOfScope).rule(),
        "kernel-construction"
    );
    assert_eq!(
        KernelLoweringError::UnsupportedRegion { rule: "fixture" }.rule(),
        "fixture"
    );
}

// ---- Cooperative workgroup tiles ------------------------------------------
//
// The structured-kernel half of the cooperative dataflow: a kernel names the
// local invocation coordinate, declares the workgroup storage its region's tile
// allocates, stages its partials, and realizes the schedule's synchronization
// point — and the verifier proves every one of those against the tile.

/// The synchronization point ordering the cooperative fixture's one handoff.
fn cooperative_point() -> SynchronizationPoint {
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
fn cooperative_barrier() -> BarrierSpec {
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
fn cooperative_region() -> VerifiedScheduledRegion {
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
fn cooperative_signature(
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
const COOPERATIVE_STAGING: StagingParameter = StagingParameter {
    staging: StagingId::FIRST,
    element_type: KernelType::F32,
    address_space: AddressSpace::Workgroup,
    element_count: 3,
};

fn cooperative_diagnostic(builder: KernelBuilder) -> KernelDiagnostic {
    let diagnostics = builder.build().unwrap_err().diagnostics().to_vec();
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("expected exactly one diagnostic, got {diagnostics:?}")
    };
    *diagnostic
}

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
            super::lower::cooperative_plan_shape_check(&region),
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
        super::lower::cooperative_plan_shape_check(&region),
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

/// Returns the phases of every staged write and staged read, in body order.
fn staged_accesses(kernel: &VerifiedKernel) -> (Vec<PhaseId>, Vec<PhaseId>) {
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

/// One deliberate deviation from the correct cooperative body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BodyChange {
    /// The body the rules below are measured against.
    None,
    /// The fence moves inside the iteration guard.
    FenceInsideTheGuard,
    /// The fence is omitted.
    NoFence,
    /// The staged read is emitted ahead of the fence.
    ReadBeforeTheFence,
    /// The fence names a point the region's tile does not declare.
    UnknownPoint,
    /// The fence fences device memory rather than workgroup memory.
    DeviceFence,
    /// The staged store names the phase that declares no write.
    WrongPhase,
}

/// Hand-builds a cooperative body carrying exactly one deviation.
///
/// The body is deliberately *not* the canonical one — it folds nothing — so the
/// unchanged case fails at the reduction contract. That is the control: each
/// change below moves the diagnostic to its own synchronization rule, which is
/// what proves the rule fired rather than something upstream of it.
fn cooperative_body(change: BodyChange) -> KernelDiagnostic {
    let scheduled = cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = cooperative_signature(&mut builder, &scheduled);
    let staging = builder.declare_staging(COOPERATIVE_STAGING).unwrap();

    let fence = |builder: &mut KernelBuilder| {
        let spec = match change {
            BodyChange::UnknownPoint => BarrierSpec {
                point: SyncPointId::new(1),
                ..cooperative_barrier()
            },
            BodyChange::DeviceFence => BarrierSpec {
                fenced_spaces: vec![AddressSpace::Device],
                ..cooperative_barrier()
            },
            _ => cooperative_barrier(),
        };
        builder.barrier(spec)
    };

    let gid = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let lid = builder.builtin(Builtin::LocalInvocationIndex).unwrap();
    let participants = builder.constant(KernelConstant::Index(3)).unwrap();
    let output = builder
        .binary(BinaryOp::IndexDivide, gid, participants)
        .unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, gid, extent)
        .unwrap();
    builder
        .predicated(active, |builder| {
            let value = builder.load(read, gid, BoundsWitnessId::new(0))?;
            let phase = if change == BodyChange::WrongPhase {
                PhaseId::new(1)
            } else {
                PhaseId::FIRST
            };
            builder.staged_store(staging, lid, value, phase)?;
            if change == BodyChange::FenceInsideTheGuard {
                fence(builder)?;
            }
            Ok(())
        })
        .unwrap();
    if change == BodyChange::ReadBeforeTheFence {
        builder
            .predicated(active, |builder| {
                let zero = builder.constant(KernelConstant::Index(0))?;
                builder.staged_load(staging, zero, PhaseId::new(1))?;
                Ok(())
            })
            .unwrap();
    }
    if !matches!(
        change,
        BodyChange::NoFence | BodyChange::FenceInsideTheGuard
    ) {
        fence(&mut builder).unwrap();
    }
    builder
        .predicated(active, |builder| {
            let one = builder.constant(KernelConstant::Index(1))?;
            let commits = builder.compare(CompareOp::IndexLessThan, lid, one)?;
            builder.predicated(commits, |builder| {
                let zero = builder.constant(KernelConstant::Index(0))?;
                let staged = builder.staged_load(staging, zero, PhaseId::new(1))?;
                builder.store(
                    write,
                    output,
                    staged,
                    BoundsWitnessId::new(1),
                    OwnershipWitnessId::new(0),
                )
            })
        })
        .unwrap();
    cooperative_diagnostic(builder)
}

/// Every synchronization rule of the structured-kernel verifier, driven once.
#[test]
fn each_kernel_synchronization_rule_refuses_its_own_defect() {
    // The control. An unchanged body reaches the reduction contract, so every
    // row below is evidence that its own rule fired first.
    assert_eq!(
        cooperative_body(BodyChange::None),
        KernelDiagnostic::ReductionContract
    );
    for (change, expected) in [
        (
            BodyChange::FenceInsideTheGuard,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            BodyChange::NoFence,
            KernelDiagnostic::UndischargedVisibility,
        ),
        (
            BodyChange::ReadBeforeTheFence,
            KernelDiagnostic::UnorderedStagedHandoff,
        ),
        (
            BodyChange::UnknownPoint,
            KernelDiagnostic::UnexpectedSynchronization,
        ),
        (
            BodyChange::DeviceFence,
            KernelDiagnostic::SynchronizationContract,
        ),
        (
            BodyChange::WrongPhase,
            KernelDiagnostic::StagedAccessEvidence,
        ),
    ] {
        assert_eq!(cooperative_body(change), expected, "{change:?}");
    }
}

/// The same region with its phases run twice and its slots rewritten.
///
/// Built by re-verifying the single-round fixture's own region rather than by a
/// second literal, so the only differences are the ones the capability requires:
/// each participant now folds one contributor per round instead of two, both
/// points name the round-loop convergence derivation, and a round boundary
/// discharges the rewrite.
fn multi_round_cooperative_region() -> VerifiedScheduledRegion {
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

/// A loop-carried tile lowers to a peeled round zero and a round loop.
///
/// Every structural claim is asserted rather than described, because the shape
/// *is* the correctness argument: round zero is emitted ahead of the loop because
/// the fold seeds at its first contributor; the accumulator that carries the
/// round totals is defined at the kernel's top level because a predicated region
/// produces no value that could cross the back edge; and the round boundary sits
/// at the head of the loop body because that is the only position that also
/// orders the peeled round's reads against the loop's first rewrite.
#[test]
fn a_loop_carried_tile_lowers_to_a_peeled_round_body() {
    let scheduled = multi_round_cooperative_region();
    let tile = crate::schedule::cooperative_tile(&scheduled.region().schedule.reduction)
        .expect("the region carries a tile");
    assert_eq!(tile.anti_dependency_edges().len(), 1);
    let kernel = lower_scheduled_region(&scheduled).expect("the loop-carried region lowers");

    let top: Vec<_> = kernel.body().operations().map(OperationRef::view).collect();
    // One barrier at the top level: the peeled round's phase boundary. The round
    // boundary has no top-level realization at all, because `rounds` rounds have
    // `rounds - 1` transitions between them.
    let top_barriers: Vec<_> = top
        .iter()
        .filter_map(|view| match view {
            OperationView::Barrier { spec } => Some(*spec),
            _ => None,
        })
        .collect();
    assert_eq!(top_barriers.len(), 1);
    assert_eq!(top_barriers[0].point, SyncPointId::FIRST);

    // The round loop runs `1..rounds` and carries exactly one `f32`, which is
    // the accumulator the peel seeded.
    let rounds: Vec<_> = top
        .iter()
        .filter_map(|view| match view {
            OperationView::SerialLoop(loops) if loops.end() == 2 => Some(*loops),
            _ => None,
        })
        .collect();
    let [round] = rounds.as_slice() else {
        panic!("expected exactly one round loop at the top level")
    };
    assert_eq!(round.start(), 1);
    assert_eq!(round.accumulators().len(), 1);
    assert_eq!(
        kernel
            .value_type(round.accumulators().next().expect("one accumulator"))
            .expect("the accumulator resolves"),
        KernelType::F32
    );

    // The round boundary is the round body's first operation, ahead of the
    // guarded rewrite; the phase boundary follows it.
    let body: Vec<_> = round.body().operations().map(OperationRef::view).collect();
    let barriers: Vec<(usize, SyncPointId)> = body
        .iter()
        .enumerate()
        .filter_map(|(position, view)| match view {
            OperationView::Barrier { spec } => Some((position, spec.point)),
            _ => None,
        })
        .collect();
    assert_eq!(
        barriers,
        [(0, SyncPointId::new(1)), (2, SyncPointId::FIRST)]
    );
    assert!(matches!(body[1], OperationView::Predicated { .. }));

    // Both rounds stage and consume: two writes and four reads, the reads being
    // a seed and a folded slot in each of the peel and the loop body.
    let (writes, reads) = staged_accesses(&kernel);
    assert_eq!(writes, [PhaseId::FIRST; 2]);
    assert_eq!(reads, [PhaseId::new(1); 4]);
}

/// The barrier-convergence rule admits exactly the nesting a tile authorizes.
///
/// Driven over the predicate directly, because a body cannot reach every row:
/// the depths a canonical body emits are a subset of the admitted ones, and a
/// rule with only its refusals driven would be half-evidenced. The refusals are
/// additionally driven end to end through real bodies by
/// `each_kernel_synchronization_rule_refuses_its_own_defect`'s
/// `FenceInsideTheGuard` row and by
/// `each_loop_carried_synchronization_rule_refuses_its_own_defect`.
#[test]
fn the_barrier_convergence_rule_admits_only_the_nesting_a_tile_authorizes() {
    for (block_depth, loop_depth, rounds, admitted) in [
        // A single-round tile authorizes the top level and nothing else.
        (0, 0, 1, true),
        (1, 0, 1, false),
        (1, 1, 1, false),
        (2, 1, 1, false),
        // A loop-carried tile authorizes the round loop *and* the top level,
        // because the fold's seed peels round zero out of the loop and its
        // barrier is realized there. What stops a stray top-level barrier from
        // riding on that is the realization count, not this predicate.
        (1, 1, 2, true),
        (0, 0, 2, true),
        (2, 2, 2, false),
        // A predicate on the path is refused whatever the round count: the
        // difference between the two depths counts the predicates, and any of
        // them admits a dynamic subset of the participants.
        (1, 0, 2, false),
        (2, 1, 2, false),
        (3, 1, 2, false),
    ] {
        assert_eq!(
            super::verify::barrier_is_convergent(block_depth, loop_depth, rounds),
            admitted,
            "block {block_depth}, loop {loop_depth}, rounds {rounds}"
        );
    }
}

/// The barrier realizing the loop-carried fixture's round boundary.
fn cooperative_round_barrier() -> BarrierSpec {
    BarrierSpec {
        point: SyncPointId::new(1),
        ..cooperative_barrier()
    }
}

/// One deliberate deviation from a loop-carried cooperative body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoopCarriedChange {
    /// The body the rules below are measured against.
    None,
    /// The round boundary moves to the end of the round body.
    RoundBoundaryAtTheTail,
    /// The round boundary is omitted.
    NoRoundBoundary,
    /// The round boundary is additionally realized in the peeled round.
    RoundBoundaryInThePeel,
    /// The round loop runs `0..rounds`, as a body with no peel would.
    UnpeeledRoundLoop,
    /// The peel's fence sits inside a loop that is not the round loop.
    FenceInAnotherLoop,
    /// The round body reads staging ahead of its own phase boundary.
    ReadBeforeTheFence,
}

/// Hand-builds a loop-carried cooperative body carrying exactly one deviation.
///
/// The body is deliberately *not* the canonical one — it folds nothing, so both
/// its per-round contributor fold and its staged folds are missing — which makes
/// the unchanged case fail at the reduction contract. That is the control: each
/// change below moves the diagnostic to its own rule, which is what proves the
/// rule fired rather than something upstream of it.
fn loop_carried_body(change: LoopCarriedChange) -> KernelDiagnostic {
    let scheduled = multi_round_cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = cooperative_signature(&mut builder, &scheduled);
    let staging = builder.declare_staging(COOPERATIVE_STAGING).unwrap();

    let gid = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let lid = builder.builtin(Builtin::LocalInvocationIndex).unwrap();
    let participants = builder.constant(KernelConstant::Index(3)).unwrap();
    let output = builder
        .binary(BinaryOp::IndexDivide, gid, participants)
        .unwrap();
    let extent = builder.constant(KernelConstant::Index(6)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, gid, extent)
        .unwrap();
    let zero = builder.constant(KernelConstant::Index(0)).unwrap();

    // The producing phase, emitted identically in the peel and in the loop.
    let produce = move |builder: &mut KernelBuilder| {
        builder.predicated(active, move |builder| {
            let value = builder.load(read, gid, BoundsWitnessId::new(0))?;
            builder.staged_store(staging, lid, value, PhaseId::FIRST)
        })
    };

    produce(&mut builder).unwrap();
    if change == LoopCarriedChange::FenceInAnotherLoop {
        // A loop at the kernel's top level that is not the round loop, standing
        // in for a contributor fold. Its accumulator is a constant rather than a
        // staged read, so the only rule this row can trip is the round-loop one.
        let accumulator = builder.constant(KernelConstant::F32Bits(0)).unwrap();
        builder
            .serial_loop(
                SerialLoopSpec { start: 1, end: 3 },
                &[accumulator],
                |builder, parameters| {
                    builder.barrier(cooperative_barrier())?;
                    Ok(vec![
                        parameters
                            .accumulator(0)
                            .ok_or(KernelBuildError::EmptyLoopAccumulators)?,
                    ])
                },
            )
            .unwrap();
    } else {
        builder.barrier(cooperative_barrier()).unwrap();
    }
    if change == LoopCarriedChange::RoundBoundaryInThePeel {
        builder.barrier(cooperative_round_barrier()).unwrap();
    }
    let seed = builder.staged_load(staging, zero, PhaseId::new(1)).unwrap();
    let start = u64::from(change != LoopCarriedChange::UnpeeledRoundLoop);
    let results = builder
        .serial_loop(
            SerialLoopSpec { start, end: 2 },
            &[seed],
            move |builder, parameters| {
                let accumulator = parameters
                    .accumulator(0)
                    .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                if !matches!(
                    change,
                    LoopCarriedChange::NoRoundBoundary | LoopCarriedChange::RoundBoundaryAtTheTail
                ) {
                    builder.barrier(cooperative_round_barrier())?;
                }
                produce(builder)?;
                if change == LoopCarriedChange::ReadBeforeTheFence {
                    builder.staged_load(staging, zero, PhaseId::new(1))?;
                }
                builder.barrier(cooperative_barrier())?;
                let staged = builder.staged_load(staging, zero, PhaseId::new(1))?;
                let sum = builder.binary(BinaryOp::F32Add, accumulator, staged)?;
                if change == LoopCarriedChange::RoundBoundaryAtTheTail {
                    builder.barrier(cooperative_round_barrier())?;
                }
                Ok(vec![sum])
            },
        )
        .unwrap();
    let total = results.get(0).unwrap();
    builder
        .predicated(active, |builder| {
            let one = builder.constant(KernelConstant::Index(1))?;
            let commits = builder.compare(CompareOp::IndexLessThan, lid, one)?;
            builder.predicated(commits, |builder| {
                builder.store(
                    write,
                    output,
                    total,
                    BoundsWitnessId::new(1),
                    OwnershipWitnessId::new(0),
                )
            })
        })
        .unwrap();
    cooperative_diagnostic(builder)
}

/// Every rule the round structure adds, driven once against its own defect.
#[test]
fn each_loop_carried_synchronization_rule_refuses_its_own_defect() {
    // The control. An unchanged body reaches the reduction contract, so every
    // row below is evidence that its own rule fired first.
    assert_eq!(
        loop_carried_body(LoopCarriedChange::None),
        KernelDiagnostic::ReductionContract
    );
    for (change, expected) in [
        // The cyclic rule's `b < w` arm is the only one that also orders the
        // peeled round's reads against the loop's first rewrite, so a boundary
        // at the tail satisfies the back edge and still leaves a race.
        (
            LoopCarriedChange::RoundBoundaryAtTheTail,
            KernelDiagnostic::UnorderedStagedRewrite,
        ),
        (
            LoopCarriedChange::NoRoundBoundary,
            KernelDiagnostic::UndischargedAntiDependency,
        ),
        // `rounds` rounds have `rounds - 1` transitions, so a round boundary
        // realized in the peel as well is realized once too often.
        (
            LoopCarriedChange::RoundBoundaryInThePeel,
            KernelDiagnostic::SynchronizationRealization,
        ),
        // The trip-count obligation: the enclosing loop must be the round loop,
        // and a `0..rounds` loop is the shape a body with no peel would emit.
        (
            LoopCarriedChange::UnpeeledRoundLoop,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            LoopCarriedChange::FenceInAnotherLoop,
            KernelDiagnostic::SynchronizationConvergence,
        ),
        (
            LoopCarriedChange::ReadBeforeTheFence,
            KernelDiagnostic::UnorderedStagedHandoff,
        ),
    ] {
        assert_eq!(loop_carried_body(change), expected, "{change:?}");
    }
}

// ---- Executing a cooperative body ------------------------------------------
//
// A verifier proves the body is the canonical refinement of its schedule; it
// does not prove the canonical body computes the declared order. Running it is
// what does, and the machine below reads *only* the structured kernel IR — no
// schedule, no semantic graph — so agreeing with the reference is also the
// evidence that a backend needs nothing else.

/// One typed value produced while interpreting a structured kernel.
#[derive(Clone, Copy, Debug)]
enum KirValue {
    Bool(bool),
    Index(u64),
    F32(f32),
}

impl KirValue {
    fn index(self) -> u64 {
        match self {
            Self::Index(value) => value,
            other => panic!("expected an index-typed value, found {other:?}"),
        }
    }
    fn float(self) -> f32 {
        match self {
            Self::F32(value) => value,
            other => panic!("expected an f32-typed value, found {other:?}"),
        }
    }
    fn boolean(self) -> bool {
        match self {
            Self::Bool(flag) => flag,
            other => panic!("expected a predicate value, found {other:?}"),
        }
    }
}

/// One step of a workgroup's execution, flattened past every rendezvous.
///
/// A barrier separates the lanes' execution, so every construct that *contains*
/// one has to be unrolled into this stream: a lane cannot be advanced through
/// half a round loop by an interpreter that recurses into it. Loops that contain
/// no barrier stay a single [`Self::Operation`] and are interpreted recursively,
/// which is why the staged and contributor folds cost nothing here.
#[derive(Clone, Copy, Debug)]
enum Step<'a> {
    /// An operation one lane executes in place.
    Operation(OperationRef<'a>),
    /// Every lane reaches this point before any lane passes it.
    Rendezvous,
    /// Carry a barrier-containing loop's initial values into its state.
    Seed(SerialLoopRef<'a>),
    /// Bind one iteration's induction variable and accumulator parameters.
    Iterate(SerialLoopRef<'a>, u64),
    /// Read one iteration's yields back into the carried state.
    Yield(SerialLoopRef<'a>),
    /// Publish the carried state as the loop's results.
    Exit(SerialLoopRef<'a>),
}

/// Returns whether a block, or anything nested inside it, contains a barrier.
fn contains_barrier(block: BlockRef<'_>) -> bool {
    block.operations().any(|operation| match operation.view() {
        OperationView::Barrier { .. } => true,
        OperationView::Predicated { body, .. } => contains_barrier(body),
        OperationView::SerialLoop(loops) => contains_barrier(loops.body()),
        _ => false,
    })
}

/// Flattens one block into the step stream a workgroup executes.
fn flatten<'a>(block: BlockRef<'a>, steps: &mut Vec<Step<'a>>) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Barrier { .. } => steps.push(Step::Rendezvous),
            OperationView::SerialLoop(loops) if contains_barrier(loops.body()) => {
                steps.push(Step::Seed(loops));
                for iteration in loops.start()..loops.end() {
                    steps.push(Step::Iterate(loops, iteration));
                    flatten(loops.body(), steps);
                    steps.push(Step::Yield(loops));
                }
                steps.push(Step::Exit(loops));
            }
            _ => steps.push(Step::Operation(operation)),
        }
    }
}

/// One lane's private interpreter state, carried across every rendezvous.
#[derive(Clone, Debug, Default)]
struct Lane {
    values: BTreeMap<VerifiedValueId, KirValue>,
    /// Each barrier-containing loop's carried accumulators, keyed by its own
    /// induction variable — the one value that names a loop uniquely.
    carried: BTreeMap<VerifiedValueId, Vec<KirValue>>,
}

/// A backend-shaped interpreter that reads only the structured kernel IR.
///
/// **Lanes advance one segment at a time, and each lane runs a whole segment
/// before the next lane starts it.** That is the faithful model of a control
/// barrier and it is deliberately unforgiving: a body that read a staged slot in
/// the same segment as another lane's write to it reads whatever that lane had
/// not yet stored, and a body that rewrote a slot in the same segment as another
/// lane's read of it destroys the value the reader was about to take. Both are
/// exactly the races the two synchronization evidence classes exist to prevent,
/// and both surface here as a wrong result rather than as a passing test.
struct KirMachine<'a> {
    kernel: &'a VerifiedKernel,
    input: &'a [f32],
    output: Vec<f32>,
    lane: Lane,
    local: u64,
    staged: Vec<f32>,
}

impl<'a> KirMachine<'a> {
    fn run(kernel: &'a VerifiedKernel, input: &'a [f32]) -> Vec<f32> {
        let mut buffers = kernel.buffers();
        let read = buffers.next().expect("a read buffer parameter");
        let write = buffers.next().expect("a write buffer parameter");
        assert_eq!(input.len(), usize::try_from(read.element_count).unwrap());
        let outputs = usize::try_from(write.element_count).unwrap();
        // Read from the kernel's own staging declaration, so the machine still
        // resolves nothing from the schedule or the graph.
        let slots = kernel
            .staging()
            .next()
            .map_or(1, |staging| staging.element_count.max(1));
        let participants = usize::try_from(slots).unwrap();
        let mut steps = Vec::new();
        flatten(kernel.body(), &mut steps);
        let mut machine = KirMachine {
            kernel,
            input,
            output: vec![f32::NAN; outputs],
            lane: Lane::default(),
            local: 0,
            staged: vec![f32::NAN; participants],
        };
        for workgroup in 0..outputs {
            let mut lanes = vec![Lane::default(); participants];
            machine.staged.fill(f32::NAN);
            for segment in steps.split(|step| matches!(step, Step::Rendezvous)) {
                for (lane, state) in lanes.iter_mut().enumerate() {
                    let lane = u64::try_from(lane).unwrap();
                    machine.lane = std::mem::take(state);
                    machine.local = lane;
                    let invocation = u64::try_from(workgroup).unwrap() * slots + lane;
                    for step in segment {
                        machine.run_step(*step, invocation);
                    }
                    *state = std::mem::take(&mut machine.lane);
                }
            }
        }
        machine.output
    }

    fn run_step(&mut self, step: Step<'a>, invocation: u64) {
        match step {
            Step::Operation(operation) => self.run_operation(operation, invocation),
            // Consumed by the segment split above; a lane never executes one.
            Step::Rendezvous => unreachable!("a rendezvous is a segment boundary"),
            Step::Seed(loops) => {
                let initial: Vec<KirValue> = loops.initial().map(|value| self.get(value)).collect();
                self.lane.carried.insert(Self::loop_key(loops), initial);
            }
            Step::Iterate(loops, iteration) => {
                let key = Self::loop_key(loops);
                let carried = self.lane.carried.get(&key).cloned().expect("a seeded loop");
                self.lane.values.insert(key, KirValue::Index(iteration));
                for (parameter, value) in loops.accumulators().zip(carried) {
                    self.lane.values.insert(parameter, value);
                }
            }
            Step::Yield(loops) => {
                let yielded: Vec<KirValue> = loops.yields().map(|value| self.get(value)).collect();
                self.lane.carried.insert(Self::loop_key(loops), yielded);
            }
            Step::Exit(loops) => {
                let key = Self::loop_key(loops);
                let carried = self.lane.carried.get(&key).cloned().expect("a seeded loop");
                let results: Vec<VerifiedValueId> = self.loop_results(loops);
                for (result, value) in results.into_iter().zip(carried) {
                    self.lane.values.insert(result, value);
                }
            }
        }
    }

    /// Names one barrier-containing loop by its own induction variable.
    fn loop_key(loops: SerialLoopRef<'a>) -> VerifiedValueId {
        loops.induction().expect("an induction variable")
    }

    /// Returns the values a flattened loop defines in its enclosing block.
    ///
    /// Recovered by searching the top-level operations for the loop whose
    /// induction variable matches, because a [`SerialLoopRef`] views the loop's
    /// inputs and body and not the operation that owns it.
    fn loop_results(&self, loops: SerialLoopRef<'a>) -> Vec<VerifiedValueId> {
        let key = Self::loop_key(loops);
        self.kernel
            .body()
            .operations()
            .find(|operation| match operation.view() {
                OperationView::SerialLoop(candidate) => candidate.induction() == Some(key),
                _ => false,
            })
            .expect("a flattened loop is a top-level operation")
            .results()
            .collect()
    }

    fn run_block(&mut self, block: BlockRef<'a>, invocation: u64) {
        for operation in block.operations() {
            self.run_operation(operation, invocation);
        }
    }

    fn run_operation(&mut self, operation: OperationRef<'a>, invocation: u64) {
        let mut results = operation.results();
        match operation.view() {
            OperationView::Builtin { builtin } => {
                let value = match builtin {
                    Builtin::GlobalInvocationIndex => invocation,
                    Builtin::LocalInvocationIndex => self.local,
                };
                self.define(&mut results, KirValue::Index(value));
            }
            OperationView::Constant { value } => {
                let value = match value {
                    KernelConstant::Bool(flag) => KirValue::Bool(flag),
                    KernelConstant::Index(index) => KirValue::Index(index),
                    KernelConstant::F32Bits(bits) => KirValue::F32(f32::from_bits(bits)),
                    // This machine executes cooperative reductions, and every
                    // reduction family in this vocabulary is `f32`; a `bf16`
                    // constant reaching it would mean the fixture had drifted
                    // into a program it cannot model, which is a defect to
                    // report rather than a value to guess at.
                    KernelConstant::Bf16Bits(bits) => {
                        panic!("no cooperative fixture carries the bf16 constant {bits:#06x}")
                    }
                };
                self.define(&mut results, value);
            }
            OperationView::Binary { op, lhs, rhs } => {
                let value = match op {
                    BinaryOp::IndexAdd => {
                        KirValue::Index(self.get(lhs).index() + self.get(rhs).index())
                    }
                    BinaryOp::IndexMultiply => {
                        KirValue::Index(self.get(lhs).index() * self.get(rhs).index())
                    }
                    BinaryOp::IndexDivide => {
                        KirValue::Index(self.get(lhs).index() / self.get(rhs).index())
                    }
                    BinaryOp::IndexModulo => {
                        KirValue::Index(self.get(lhs).index() % self.get(rhs).index())
                    }
                    BinaryOp::F32Add => {
                        KirValue::F32(self.get(lhs).float() + self.get(rhs).float())
                    }
                    BinaryOp::F32Multiply => {
                        KirValue::F32(self.get(lhs).float() * self.get(rhs).float())
                    }
                    other => panic!("unsupported binary operation {other:?}"),
                };
                self.define(&mut results, value);
            }
            OperationView::Compare { op, lhs, rhs } => {
                let value = match op {
                    CompareOp::IndexLessThan => {
                        KirValue::Bool(self.get(lhs).index() < self.get(rhs).index())
                    }
                };
                self.define(&mut results, value);
            }
            OperationView::Convert { op, source } => {
                let value = self.get(source).float();
                let value = match op {
                    ConvertOp::CanonicalizeF32Nan => {
                        if value.is_nan() {
                            f32::from_bits(self.kernel.numerical().canonical_arithmetic_nan_bits)
                        } else {
                            value
                        }
                    }
                    other => panic!("unsupported conversion {other:?}"),
                };
                self.define(&mut results, KirValue::F32(value));
            }
            OperationView::Load { offset, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                let value = KirValue::F32(self.input[offset]);
                self.define(&mut results, value);
            }
            OperationView::GuardedLoad {
                predicate,
                offset,
                inactive,
                ..
            } => {
                let value = if self.get(predicate).boolean() {
                    let offset = usize::try_from(self.get(offset).index()).unwrap();
                    KirValue::F32(self.input[offset])
                } else {
                    self.get(inactive)
                };
                self.define(&mut results, value);
            }
            OperationView::Store { offset, value, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                self.output[offset] = self.get(value).float();
            }
            OperationView::Predicated { predicate, body } => {
                if self.get(predicate).boolean() {
                    self.run_block(body, invocation);
                }
            }
            OperationView::SerialLoop(loops) => {
                let mut carried: Vec<KirValue> =
                    loops.initial().map(|value| self.get(value)).collect();
                let induction = loops.induction().expect("an induction variable");
                let parameters: Vec<_> = loops.accumulators().collect();
                for iteration in loops.start()..loops.end() {
                    self.lane
                        .values
                        .insert(induction, KirValue::Index(iteration));
                    for (parameter, value) in parameters.iter().zip(&carried) {
                        self.lane.values.insert(*parameter, *value);
                    }
                    self.run_block(loops.body(), invocation);
                    carried = loops.yields().map(|value| self.get(value)).collect();
                }
                for (result, value) in results.zip(carried) {
                    self.lane.values.insert(result, value);
                }
            }
            // Flattened into a segment boundary before any lane runs, so a
            // barrier reaching here is one nested below a construct this machine
            // descends into — which the verifier refuses.
            OperationView::Barrier { .. } => panic!("a nested barrier reached the machine"),
            OperationView::StagedStore { offset, value, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                self.staged[offset] = self.get(value).float();
            }
            OperationView::StagedLoad { offset, .. } => {
                let offset = usize::try_from(self.get(offset).index()).unwrap();
                let value = KirValue::F32(self.staged[offset]);
                self.define(&mut results, value);
            }
            other => panic!("unsupported structured operation {other:?}"),
        }
    }

    fn define(&mut self, results: &mut impl Iterator<Item = VerifiedValueId>, value: KirValue) {
        let result = results.next().expect("one defined result");
        self.lane.values.insert(result, value);
    }

    fn get(&self, id: VerifiedValueId) -> KirValue {
        *self
            .lane
            .values
            .get(&id)
            .expect("a value defined before its use")
    }
}

/// Two rows whose sum depends on where the round boundaries fall.
///
/// `5e19` is far enough above the unit ulp that adding one to it is the
/// identity, so a grouping that puts the cancelling pair in one round absorbs
/// the small value beside it and a grouping that splits them does not. The two
/// rows are sensitive in opposite directions, so neither the round-major nor the
/// participant-major grouping can agree with the other by luck on both.
/// Each row also carries a small value in a round the other's cancellation does
/// not reach, so a body that folded round zero's range twice — the shape a
/// dropped round term produces — disagrees on both rows rather than on one.
const REGROUPING_SENSITIVE_ROWS: [[f32; 6]; 2] = [
    [5.0e19, 1.0, -5.0e19, 3.0, 0.0, 0.0],
    [0.0, 5.0e19, 0.0, -5.0e19, 2.0, 0.0],
];

/// The exact value a cooperative tile's declared order computes for one row.
///
/// Written from the declared arithmetic rather than from the emitted body:
/// participant `p` of round `r` folds the contiguous range at index
/// `r * participants + p` seeded at its own first contributor, the staged set is
/// folded in ascending participant order, and the round totals accumulate in
/// ascending round order. Every fold seeds at its first contributor, which is
/// what makes this a reassociation of the declared sequence rather than a sum
/// against an identity element.
fn cooperative_reference(
    row: &[f32],
    participants: usize,
    contributors: usize,
    rounds: usize,
) -> f32 {
    let mut total: Option<f32> = None;
    for round in 0..rounds {
        let mut staged: Option<f32> = None;
        for participant in 0..participants {
            let base = (round * participants + participant) * contributors;
            let mut range = row[base];
            for step in 1..contributors {
                range += row[base + step];
            }
            staged = Some(staged.map_or(range, |value| value + range));
        }
        let round_total = staged.expect("a tile has at least one participant");
        total = Some(total.map_or(round_total, |value| value + round_total));
    }
    total.expect("a tile runs at least one round")
}

/// The same fold with the rounds and participants exchanged.
///
/// Participant `p` of round `r` owning the range at `p * rounds + r` is the
/// other natural reading of a two-dimensional split, and it is the one the
/// contributor arithmetic must *not* compute.
fn participant_major_reference(
    row: &[f32],
    participants: usize,
    contributors: usize,
    rounds: usize,
) -> f32 {
    let mut total: Option<f32> = None;
    for round in 0..rounds {
        let mut staged: Option<f32> = None;
        for participant in 0..participants {
            let base = (participant * rounds + round) * contributors;
            let mut range = row[base];
            for step in 1..contributors {
                range += row[base + step];
            }
            staged = Some(staged.map_or(range, |value| value + range));
        }
        let round_total = staged.expect("a tile has at least one participant");
        total = Some(total.map_or(round_total, |value| value + round_total));
    }
    total.expect("a tile runs at least one round")
}

fn bit_patterns(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

/// The neighbouring round grouping really does compute something else.
///
/// The guard on the conformance test below: an executed kernel agreeing with its
/// declared order is only evidence if some *other* order would have disagreed.
/// This pins that, so an input that made the comparison vacuous fails here rather
/// than silently weakening the claim next door.
#[test]
fn the_declared_round_grouping_is_what_the_agreement_is_evidence_about() {
    for row in &REGROUPING_SENSITIVE_ROWS {
        assert_ne!(
            cooperative_reference(row, 3, 1, 2).to_bits(),
            participant_major_reference(row, 3, 1, 2).to_bits(),
            "the conformance input cannot tell two round groupings apart"
        );
    }
}

/// The single-round body executes to the reference's bits at its declared order.
///
/// Run first and reported separately, because it is what anchors the machine:
/// the single-round shape is already verified, already has a checked-in Metal
/// golden, and its order is not in question — so a disagreement here is a defect
/// in the interpreter rather than in the body under test.
#[test]
fn the_cooperative_body_matches_the_reference_at_its_declared_order() {
    let scheduled = cooperative_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the cooperative region lowers");
    let input: Vec<f32> = REGROUPING_SENSITIVE_ROWS.concat();
    let expected: Vec<f32> = REGROUPING_SENSITIVE_ROWS
        .iter()
        .map(|row| cooperative_reference(row, 3, 2, 1))
        .collect();
    assert_eq!(
        bit_patterns(&KirMachine::run(&kernel, &input)),
        bit_patterns(&expected)
    );
}

/// The loop-carried body executes to the reference's bits at its declared order.
///
/// The ticket's closing evidence. The kernel is *run* rather than inspected:
/// every lane is advanced to each barrier before any lane crosses it, so a body
/// that read a staged slot before its writer produced it, or rewrote one before
/// its readers were finished, would carry a `NaN` or a next-round partial into
/// the fold and fail here rather than pass by accident.
#[test]
fn the_loop_carried_body_matches_the_reference_at_its_declared_order() {
    let scheduled = multi_round_cooperative_region();
    let kernel = lower_scheduled_region(&scheduled).expect("the loop-carried region lowers");
    let input: Vec<f32> = REGROUPING_SENSITIVE_ROWS.concat();
    let expected: Vec<f32> = REGROUPING_SENSITIVE_ROWS
        .iter()
        .map(|row| cooperative_reference(row, 3, 1, 2))
        .collect();
    assert_eq!(
        bit_patterns(&KirMachine::run(&kernel, &input)),
        bit_patterns(&expected)
    );
}

/// A kernel that stages without a cooperative region is refused by name.
#[test]
fn a_staged_access_without_a_tile_is_refused() {
    let scheduled = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = pointwise_signature(&mut builder, &scheduled, 6);
    let (invocation, active) = guard(&mut builder, 6);
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
    // A region with no tile may declare no workgroup storage at all, so the
    // declaration is refused before a staged operation could even name one.
    builder.declare_staging(COOPERATIVE_STAGING).unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::StagingContract
    );
}

/// Staging that does not realize the region's tile is refused.
#[test]
fn staging_that_contradicts_the_region_tile_is_refused() {
    for staging in [
        StagingParameter {
            element_count: 4,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            address_space: AddressSpace::Device,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            element_type: KernelType::U8,
            ..COOPERATIVE_STAGING
        },
        StagingParameter {
            staging: StagingId::new(1),
            ..COOPERATIVE_STAGING
        },
    ] {
        let scheduled = cooperative_region();
        let mut builder = KernelBuilder::new(&scheduled).unwrap();
        cooperative_signature(&mut builder, &scheduled);
        builder.declare_staging(staging).unwrap();
        assert_eq!(
            cooperative_diagnostic(builder),
            KernelDiagnostic::StagingContract,
            "{staging:?} was admitted against the region's tile"
        );
    }

    // The count itself, in both directions.
    let scheduled = cooperative_region();
    let mut missing = KernelBuilder::new(&scheduled).unwrap();
    cooperative_signature(&mut missing, &scheduled);
    assert_eq!(
        cooperative_diagnostic(missing),
        KernelDiagnostic::StagingContract
    );
    let mut extra = KernelBuilder::new(&scheduled).unwrap();
    cooperative_signature(&mut extra, &scheduled);
    extra.declare_staging(COOPERATIVE_STAGING).unwrap();
    extra
        .declare_staging(StagingParameter {
            staging: StagingId::new(1),
            ..COOPERATIVE_STAGING
        })
        .unwrap();
    assert_eq!(
        cooperative_diagnostic(extra),
        KernelDiagnostic::StagingContract
    );
}

/// Declares the boundary signature of the `[2, 3] -> [2]` serial reduction.
///
/// No body follows it in the tests below: the staging and builtin rules run
/// inside signature and cooperative verification, ahead of the body walk, so a
/// body would add operations to a kernel that is already rejected and would
/// obscure which rule the test is about.
fn serial_reduction_signature(builder: &mut KernelBuilder, scheduled: &VerifiedScheduledRegion) {
    builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 6,
        })
        .unwrap();
    builder
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
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
}

/// A region that stages nothing may declare no workgroup storage.
///
/// Without this a producer could allocate threadgroup memory its schedule never
/// proved, and the derived requirement composed against a target would be the
/// schedule's zero rather than the kernel's real demand.
#[test]
fn a_noncooperative_kernel_declaring_staging_is_refused() {
    let scheduled = reduction_region(
        RegionId::new(24),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    );
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    serial_reduction_signature(&mut builder, &scheduled);
    builder.declare_staging(COOPERATIVE_STAGING).unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::StagingContract
    );
}

/// A cooperative kernel must admit the local invocation coordinate.
///
/// Its participants are named by their position in the workgroup, so a kernel
/// that cannot read that position cannot say which participant it is.
#[test]
fn a_cooperative_kernel_without_the_local_coordinate_is_refused() {
    let scheduled = cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 12,
        })
        .unwrap();
    builder
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
        .numerical(NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..numerical()
        })
        .unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    builder.declare_staging(COOPERATIVE_STAGING).unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::BuiltinContract
    );
}

/// A non-cooperative kernel must not admit the local coordinate either.
#[test]
fn a_noncooperative_kernel_admitting_the_local_coordinate_is_refused() {
    let scheduled = reduction_region(
        RegionId::new(25),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
    );
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    serial_reduction_signature(&mut builder, &scheduled);
    builder
        .admit_builtin(Builtin::LocalInvocationIndex)
        .unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::BuiltinContract
    );
}

/// Workgroup storage is never a buffer *parameter*, whatever it is used for.
///
/// A parameter's position is its argument-table ordinal, so admitting a
/// workgroup buffer would re-base every later ordinal and change what an
/// existing signature position means. The refusal holds for a cooperative
/// region, which does require workgroup storage, so it is a rule about the
/// binding namespace rather than about whether local memory is needed.
#[test]
fn workgroup_storage_is_refused_as_a_buffer_parameter() {
    let scheduled = cooperative_region();
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Workgroup,
            access: BufferAccess::Read,
            element_count: 12,
        })
        .unwrap();
    builder
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
    builder.declare_staging(COOPERATIVE_STAGING).unwrap();
    assert_eq!(
        cooperative_diagnostic(builder),
        KernelDiagnostic::AddressSpaceContract
    );
}

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

/// Counts the binary operations a kernel body contains, nested blocks included.
///
/// The same traversal `loaded_buffers` performs, over a different operation kind:
/// a combine emitted inside the fold's serial loop is what this has to reach, and
/// a walk that stopped at the top level would count zero for every reduction.
fn binary_op_counts(kernel: &VerifiedKernel, wanted: BinaryOp) -> usize {
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

// ---------------------------------------------------------------------------
// BF16
// ---------------------------------------------------------------------------

/// `bf16` 2.0, 1.0, and the `f32` canonical arithmetic NaN payload's width error.
const BF16_SCALE_BITS: u16 = 0x4000;
const BF16_BIAS_BITS: u16 = 0x3f80;
/// The `bf16` canonical arithmetic NaN, zero-extended into the 32-bit field.
const BF16_NAN_BITS: u32 = crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS as u32;

fn bf16_numerical() -> NumericalRealization {
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

/// `(x * 2.0) + 1.0` in `bf16`, the direct sibling of [`scale_bias_expression`].
fn bf16_scale_bias_expression() -> crate::schedule::PointwiseBf16Expression {
    let mut expression = crate::schedule::PointwiseBf16ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(BF16_SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BF16_BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// A `bf16` pointwise region over `shape`, built through the public builder.
fn bf16_pointwise_region(id: RegionId, shape: &Shape) -> VerifiedScheduledRegion {
    bf16_pointwise_builder(id, shape).build().unwrap()
}

fn bf16_pointwise_builder(id: RegionId, shape: &Shape) -> ScheduledRegionBuilder {
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
            scalar: ScalarProgram::PointwiseBf16(bf16_scale_bias_expression()),
            numerical: bf16_numerical(),
        })
        .unwrap();
    builder
        .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}

/// A BF16 region lowers to a kernel that is BF16 in every position.
///
/// The check is on every position rather than on the arithmetic alone: a
/// vocabulary that admitted the type but declared `f32` buffers, or that reused
/// the `f32` canonicalization, would compute something the region does not mean
/// while still passing an "is there a BF16 kernel" question.
#[test]
fn a_bf16_pointwise_region_lowers_to_a_kernel_that_is_bf16_at_every_position() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let kernel = lower_scheduled_region(&scheduled).unwrap();

    assert_eq!(
        kernel.buffers().collect::<Vec<_>>(),
        [
            BufferParameter {
                tensor: TensorRole::Input,
                component_role: None,
                element_type: KernelType::Bf16,
                address_space: AddressSpace::Device,
                access: BufferAccess::Read,
                element_count: 6,
            },
            BufferParameter {
                tensor: TensorRole::Intermediate,
                component_role: None,
                element_type: KernelType::Bf16,
                address_space: AddressSpace::Device,
                access: BufferAccess::Write,
                element_count: 6,
            },
        ]
    );
    assert_eq!(binary_op_counts(&kernel, BinaryOp::Bf16Multiply), 1);
    assert_eq!(binary_op_counts(&kernel, BinaryOp::Bf16Add), 1);
    // The `f32` neighbours are absent, so the arithmetic is a function of the
    // region's width rather than of whatever the emitter reached for first.
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Multiply), 0);
    assert_eq!(binary_op_counts(&kernel, BinaryOp::F32Add), 0);

    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("the canonical body guards its effects");
    let mut constants = Vec::new();
    let mut conversions = Vec::new();
    for operation in guarded.operations() {
        match operation.view() {
            OperationView::Constant { value } => constants.push(value),
            OperationView::Convert { op, .. } => conversions.push(op),
            _ => {}
        }
    }
    assert_eq!(
        constants,
        [
            KernelConstant::Bf16Bits(BF16_SCALE_BITS),
            KernelConstant::Bf16Bits(BF16_BIAS_BITS),
        ],
        "each constant carries its own sixteen-bit payload unchanged"
    );
    assert_eq!(
        conversions,
        [
            ConvertOp::CanonicalizeBf16Nan,
            ConvertOp::CanonicalizeBf16Nan
        ],
        "every arithmetic result is canonicalized at the bf16 width"
    );

    // The produced kernel is a refinement a hand-built producer reaches too,
    // which is what makes the lowering checked rather than definitional.
    let produced = canonical_bf16_pointwise(&scheduled, 6).build().unwrap();
    assert_eq!(produced, kernel);
    assert_eq!(
        produced.canonical_identity().as_bytes(),
        kernel.canonical_identity().as_bytes()
    );
}

/// Builds the canonical BF16 pointwise kernel by hand.
fn canonical_bf16_pointwise(scheduled: &VerifiedScheduledRegion, elements: u64) -> KernelBuilder {
    let mut builder = KernelBuilder::new(scheduled).unwrap();
    let (read, write) =
        bf16_pointwise_signature(&mut builder, scheduled, elements, KernelType::Bf16);
    let (invocation, active) = guard(&mut builder, elements);
    builder
        .predicated(active, |builder| {
            let loaded = builder.load(read, invocation, BoundsWitnessId::new(0))?;
            let scale = builder.constant(KernelConstant::Bf16Bits(BF16_SCALE_BITS))?;
            let product = builder.binary(BinaryOp::Bf16Multiply, loaded, scale)?;
            let product = builder.convert(ConvertOp::CanonicalizeBf16Nan, product)?;
            let bias = builder.constant(KernelConstant::Bf16Bits(BF16_BIAS_BITS))?;
            let biased = builder.binary(BinaryOp::Bf16Add, product, bias)?;
            let biased = builder.convert(ConvertOp::CanonicalizeBf16Nan, biased)?;
            builder.store(
                write,
                invocation,
                biased,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    builder
}

/// Declares the BF16 pointwise signature at a caller-chosen element type.
///
/// The type is a parameter so the refusal test below can vary exactly one thing.
fn bf16_pointwise_signature(
    builder: &mut KernelBuilder,
    scheduled: &VerifiedScheduledRegion,
    elements: u64,
    element_type: KernelType,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: elements,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Intermediate,
            component_role: None,
            element_type,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: elements,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(bf16_numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    (read, write)
}

/// A kernel mixing BF16 and F32 is refused by name, at both places it can mix.
///
/// **Two mixes, two authorities.** A *signature* that binds `f32` buffers to a
/// BF16 region is refused by whole-kernel verification as `BufferContract`,
/// because the expected element type is derived from the region's scalar program.
/// A *body* that feeds an `f32` value into BF16 arithmetic, or applies the `f32`
/// canonicalization to a BF16 value, never reaches verification at all: the
/// builder's insertion-time type check names both the expected and the actual
/// type.
///
/// Each refusal is stated beside a passing neighbour that differs in exactly one
/// argument, so it is visibly a function of the width rather than of the fixture
/// having stopped working.
#[test]
fn a_kernel_mixing_bf16_and_f32_is_refused_by_name() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));

    // The signature mix: `f32` buffers under a BF16 region.
    let mut mixed = KernelBuilder::new(&scheduled).unwrap();
    let (read, write) = bf16_pointwise_signature(&mut mixed, &scheduled, 6, KernelType::F32);
    let (invocation, active) = guard(&mut mixed, 6);
    mixed
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
    assert_eq!(diagnostics(mixed), [KernelDiagnostic::BufferContract]);
    // The same body over BF16 buffers verifies, so the refusal is the element
    // type and not the shape of the kernel.
    assert!(canonical_bf16_pointwise(&scheduled, 6).build().is_ok());

    // The body mix: an `f32` value as a BF16 operand, and the `f32`
    // canonicalization over a BF16 value.
    let mut body = KernelBuilder::new(&scheduled).unwrap();
    let (read, _) = bf16_pointwise_signature(&mut body, &scheduled, 6, KernelType::Bf16);
    let (invocation, _) = guard(&mut body, 6);
    let loaded = body
        .load(read, invocation, BoundsWitnessId::new(0))
        .unwrap();
    let f32_constant = body.constant(KernelConstant::F32Bits(SCALE_BITS)).unwrap();
    assert_eq!(
        body.binary(BinaryOp::Bf16Multiply, loaded, f32_constant),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::Bf16,
            actual: KernelType::F32,
        })
    );
    assert_eq!(
        body.convert(ConvertOp::CanonicalizeF32Nan, loaded),
        Err(KernelBuildError::TypeMismatch {
            expected: KernelType::F32,
            actual: KernelType::Bf16,
        })
    );
    // The BF16 constant in the same position is admitted, so the refusals are
    // about the operand width rather than about the builder having stopped.
    let bf16_constant = body
        .constant(KernelConstant::Bf16Bits(BF16_SCALE_BITS))
        .unwrap();
    assert!(
        body.binary(BinaryOp::Bf16Multiply, loaded, bf16_constant)
            .is_ok()
    );
}

/// A BF16 region declaring the `f32` canonical NaN payload is refused.
///
/// The invariant `NumericalRealization::canonical_arithmetic_nan_bits` documents
/// — the region's own arithmetic type's pattern, zero-extended — is checked here
/// rather than assumed, so the reading cannot silently differ between the
/// verifier and the lowering that installs the payload.
#[test]
fn a_bf16_region_declaring_the_f32_canonical_nan_payload_is_refused() {
    let mut wrong_width = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]))
        .region()
        .clone();
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = NAN_BITS;
    assert_eq!(
        ScheduledRegionBuilder::from_region(wrong_width.clone())
            .build()
            .unwrap_err()
            .diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    // A payload that is the right pattern in the wrong place is refused too: the
    // sixteen bits go in the low half, not the high one a truncating reader
    // would produce.
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = BF16_NAN_BITS << 16;
    assert_eq!(
        ScheduledRegionBuilder::from_region(wrong_width.clone())
            .build()
            .unwrap_err()
            .diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    // Restoring the declared payload readmits the region, so the refusal is the
    // payload and not the perturbation having broken the fixture.
    region_numerical_mut(&mut wrong_width).canonical_arithmetic_nan_bits = BF16_NAN_BITS;
    assert!(
        ScheduledRegionBuilder::from_region(wrong_width)
            .build()
            .is_ok()
    );
}

/// The BF16 vocabulary appended, so no `f32` subject's identity bytes moved.
///
/// Stated as an inequality between two whole identities rather than as a claim
/// about tags: the two regions differ only in the width of their arithmetic, so
/// identities that agreed would mean the scalar-program tag or a node payload
/// was not encoded at all. The `f32` side's own byte pin lives with the schedule
/// builder (`STRICT_F32_REGION_IDENTITY_HEX`), which this widening leaves
/// unchanged.
#[test]
fn a_bf16_kernel_and_its_f32_sibling_do_not_share_identity() {
    let shape = Shape::from_dims([2, 3]);
    let bf16 = lower_scheduled_region(&bf16_pointwise_region(RegionId::new(0), &shape)).unwrap();
    let f32 = lower_scheduled_region(&pointwise_region(RegionId::new(0), &shape)).unwrap();
    assert_ne!(
        bf16.canonical_identity().as_bytes(),
        f32.canonical_identity().as_bytes()
    );
    // Both identities open with the same domain separator: the `bf16` widening
    // was an append, not a version step. The domain has since moved to `v7` for
    // an unrelated reason -- the derived index-arithmetic requirement landing
    // inside the fixed resource-requirement record -- which is why this names
    // the current domain while the comment above still describes the widening
    // this test is about.
    let domain = b"tiler.kernel.v9\0";
    assert!(bf16.canonical_identity().as_bytes().starts_with(domain));
    assert!(f32.canonical_identity().as_bytes().starts_with(domain));
}

/// The BF16 region's derived subnormal freedom is unproven, at both layers.
#[test]
fn a_bf16_region_proves_no_subnormal_freedom() {
    let scheduled = bf16_pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    assert_eq!(
        scheduled.subnormal_freedom(),
        crate::schedule::SubnormalFreedom::Unproven
    );
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    assert_eq!(
        kernel.subnormal_freedom(),
        crate::schedule::SubnormalFreedom::Unproven
    );
    // The one freedom this vocabulary states does not reach bf16 even where it
    // holds, which is why the region above could not have inherited one.
    assert!(
        !crate::schedule::SubnormalFreedom::StrictAffineNormalScaleDecode
            .discharges(crate::schedule::ArithmeticType::Bf16)
    );
}

/// The `direct` contraction of `td,od->to` over `[m, k] x [n, k] -> [m, n]`.
fn contraction_region(id: RegionId, m: u64, n: u64, k: u64) -> VerifiedScheduledRegion {
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

fn contraction_loop(kernel: &VerifiedKernel) -> SerialLoopRef<'_> {
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded contraction");
    guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a first-product contraction loop")
}

fn count_canonicalizations(kernel: &VerifiedKernel) -> usize {
    fn walk(block: BlockRef<'_>, count: &mut usize) {
        for operation in block.operations() {
            match operation.view() {
                OperationView::Convert {
                    op: ConvertOp::CanonicalizeF32Nan,
                    ..
                } => *count += 1,
                OperationView::Predicated { body, .. } => walk(body, count),
                OperationView::SerialLoop(serial) => walk(serial.body(), count),
                _ => {}
            }
        }
    }
    let mut count = 0;
    walk(kernel.body(), &mut count);
    count
}

/// The canonical lowering is a first-product separately-rounded fold.
///
/// This is the owning KIR classification: there is no fused multiply-add
/// construct and the loop starts at the first product. A simdgroup matrix
/// instruction is never formed here, so it is not a realization of `@1`.
#[test]
fn the_contraction_lowers_to_a_first_product_separately_rounded_fold() {
    let scheduled = contraction_region(RegionId::new(9), 2, 3, 4);
    let kernel = lower_scheduled_region(&scheduled).expect("the direct contraction lowers");
    let reduction = contraction_loop(&kernel);
    assert_eq!(
        (reduction.start(), reduction.end()),
        (1, 4),
        "the accumulator must start at the first product"
    );
    assert_eq!(
        count_canonicalizations(&kernel),
        3,
        "the seed product, the fold product, and the fold sum each canonicalize"
    );
}

/// A `+0.0` seed is `reduction-contract`, not a realization of `@1`.
///
/// The subject is the loop start. Fusion and NaN sites are left unperturbed so
/// this refusal cannot be confused with either of those obligations.
#[test]
fn a_positive_zero_seeded_contraction_loop_is_reduction_contract() {
    let scheduled = contraction_region(RegionId::new(9), 2, 3, 4);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 8,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
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
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            let seed = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
            let results = builder.serial_loop(
                SerialLoopSpec { start: 0, end: 4 },
                &[seed],
                |builder, parameters| {
                    let accumulator = parameters
                        .accumulator(0)
                        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                    let product = builder.convert(ConvertOp::CanonicalizeF32Nan, accumulator)?;
                    let sum = builder.binary(BinaryOp::F32Add, accumulator, product)?;
                    let sum = builder.convert(ConvertOp::CanonicalizeF32Nan, sum)?;
                    Ok(vec![sum])
                },
            )?;
            let total = results
                .get(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            builder.store(
                write,
                invocation,
                total,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("a +0.0-seeded contraction loop must not verify");
    assert_eq!(
        error.diagnostics(),
        [KernelDiagnostic::ReductionContract],
        "the seed subject must fail as reduction-contract, not as a later catch-all: {error:?}"
    );
    assert_eq!(
        error.diagnostics()[0].rule(),
        "reduction-contract",
        "the quoted seed refusal is the stable rule id"
    );
}

fn live_row_major_region(rows: u64) -> VerifiedScheduledRegion {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
    builder.iteration_shape(Shape::from_dims([rows])).unwrap();
    let inner = Axis::new(1);
    builder
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
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
                kind: BoundsProofKind::LinearRange { element_count: 0 },
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

#[derive(Clone, Copy)]
enum PointwiseWidth {
    F32,
    Bf16,
}

fn two_input_pointwise_program(width: PointwiseWidth) -> ScalarProgram {
    match width {
        PointwiseWidth::F32 => {
            let mut expression = PointwiseF32ExpressionBuilder::new();
            let left = expression.input(AccessOrdinal::FIRST).unwrap();
            let right = expression.input(AccessOrdinal::new(1)).unwrap();
            let root = expression.add(left, right).unwrap();
            ScalarProgram::PointwiseF32(expression.build(root).unwrap())
        }
        PointwiseWidth::Bf16 => {
            let mut expression = crate::schedule::PointwiseBf16ExpressionBuilder::new();
            let left = expression.input(AccessOrdinal::FIRST).unwrap();
            let right = expression.input(AccessOrdinal::new(1)).unwrap();
            let root = expression.add(left, right).unwrap();
            ScalarProgram::PointwiseBf16(expression.build(root).unwrap())
        }
    }
}

fn two_input_pointwise_builder(
    width: PointwiseWidth,
    rows: u64,
    read_maps: [LogicalAccess; 2],
    write_map: LogicalAccess,
) -> ScheduledRegionBuilder {
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder.iteration_shape(Shape::from_dims([rows])).unwrap();
    for (position, map) in read_maps.into_iter().enumerate() {
        let elements = match &map {
            LogicalAccess::LinearIdentity => rows,
            LogicalAccess::LiveRowMajor { .. } => 0,
            _ => panic!("the focused fixture only constructs identity and live accesses"),
        };
        let witness = u32::try_from(position).unwrap();
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
                    element_count: elements,
                },
            })
            .unwrap();
    }
    let write_elements = match &write_map {
        LogicalAccess::LinearIdentity => rows,
        LogicalAccess::LiveRowMajor { .. } => 0,
        _ => panic!("the focused fixture only constructs identity and live accesses"),
    };
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: write_map,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: write_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: two_input_pointwise_program(width),
            numerical: match width {
                PointwiseWidth::F32 => numerical(),
                PointwiseWidth::Bf16 => bf16_numerical(),
            },
        })
        .unwrap();
    builder
        .schedule(linear_schedule(rows, OwnershipWitnessId::new(0)))
        .unwrap();
    builder
}

fn identity_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    hex
}

fn assert_pointwise_access_refinement(builder: ScheduledRegionBuilder, subject: &str) {
    let error = builder
        .build()
        .expect_err("a mixed live-row-major access list must fail intrinsically");
    assert_eq!(
        error.diagnostics(),
        [crate::schedule::ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "the {subject} perturbation must stop at the owning schedule rule: {error:?}"
    );
    assert_eq!(
        error.diagnostics()[0].rule(),
        "numerical-or-access-refinement",
        "the {subject} refusal must retain the stable diagnostic"
    );
}

/// A static read cannot inherit the live offset selected by its sibling.
///
/// The parent verifier admitted this exact two-read subject and canonical
/// lowering minted a verified F32 and BF16 kernel whose buffer sizes were
/// `[2, 0, 0]`, then loaded the two-element first buffer at `row * N + col`.
/// Varying both widths proves their shared verifier is the refusing boundary.
#[test]
fn a_mixed_live_row_major_read_is_refused_for_f32_and_bf16() {
    let inner = Axis::new(1);
    for width in [PointwiseWidth::F32, PointwiseWidth::Bf16] {
        assert_pointwise_access_refinement(
            two_input_pointwise_builder(
                width,
                2,
                [
                    LogicalAccess::LinearIdentity,
                    LogicalAccess::LiveRowMajor { inner_axis: inner },
                ],
                LogicalAccess::LiveRowMajor { inner_axis: inner },
            ),
            "read",
        );
    }
}

/// A static owning write cannot sit inside the live loop selected by its reads.
#[test]
fn a_mixed_live_row_major_write_is_refused() {
    let inner = Axis::new(1);
    assert_pointwise_access_refinement(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [
                LogicalAccess::LiveRowMajor { inner_axis: inner },
                LogicalAccess::LiveRowMajor { inner_axis: inner },
            ],
            LogicalAccess::LinearIdentity,
        ),
        "write",
    );
}

/// One live stride cannot become the authority for a different live axis.
#[test]
fn disagreeing_live_row_major_axes_are_refused() {
    let inner = Axis::new(1);
    assert_pointwise_access_refinement(
        two_input_pointwise_builder(
            PointwiseWidth::F32,
            2,
            [
                LogicalAccess::LiveRowMajor { inner_axis: inner },
                LogicalAccess::LiveRowMajor {
                    inner_axis: Axis::new(2),
                },
            ],
            LogicalAccess::LiveRowMajor { inner_axis: inner },
        ),
        "axis",
    );
}

/// The refusal narrows only mixed subjects; both valid regimes stay byte-exact.
#[test]
fn static_and_same_axis_live_pointwise_identities_remain_exact() {
    let static_schedule = pointwise_region(RegionId::new(0), &Shape::from_dims([2, 3]));
    let static_kernel = lower_scheduled_region(&static_schedule).unwrap();
    assert_eq!(
        identity_hex(static_kernel.canonical_identity().as_bytes()),
        ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX,
        "the existing all-static kernel pin must not move"
    );

    let live = live_row_major_region(2);
    let live_kernel = lower_scheduled_region(&live).unwrap();
    assert_eq!(
        identity_hex(live.canonical_identity().as_bytes()),
        LIVE_ROW_MAJOR_SCHEDULE_IDENTITY_HEX,
        "the existing all-live schedule bytes must not move"
    );
    assert_eq!(
        identity_hex(live_kernel.canonical_identity().as_bytes()),
        LIVE_ROW_MAJOR_KERNEL_IDENTITY_HEX,
        "the existing all-live kernel bytes must not move"
    );

    let inner = Axis::new(1);
    for width in [PointwiseWidth::F32, PointwiseWidth::Bf16] {
        let scheduled = two_input_pointwise_builder(
            width,
            2,
            [
                LogicalAccess::LiveRowMajor { inner_axis: inner },
                LogicalAccess::LiveRowMajor { inner_axis: inner },
            ],
            LogicalAccess::LiveRowMajor { inner_axis: inner },
        )
        .build()
        .expect("same-axis all-live accesses remain valid");
        lower_scheduled_region(&scheduled).expect("both valid all-live widths still lower");
    }
}

/// One compiled payload consumes a live input extent; baking the neighbour
/// value is a different kernel.
#[test]
fn a_live_row_major_kernel_reads_the_declared_extent_and_does_not_bake_it() {
    let scheduled = live_row_major_region(2);
    let kernel = lower_scheduled_region(&scheduled).unwrap();
    let extents: Vec<_> = kernel.input_extents().collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].access, AccessOrdinal::FIRST);
    assert_eq!(extents[0].axis, Axis::new(1));
    let baked_14 = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 14]),
    ))
    .unwrap();
    let baked_15 = lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([2, 15]),
    ))
    .unwrap();
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes(),
        "baking N = 14 must change identity"
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change identity"
    );
    let again = lower_scheduled_region(&live_row_major_region(2)).unwrap();
    assert_eq!(
        kernel.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes()
    );
    // Dense F32 [2, N]: semantic (row = 1, column = 0) is element N, so bytes
    // 4N. The live operand is the stride; baking 14 or 15 is a different
    // kernel, shown above.
    assert_eq!(dense_f32_row_major_bytes(1, 0, 14), 56);
    assert_eq!(dense_f32_row_major_bytes(1, 0, 15), 60);
}

fn count_element_access_placement(
    block: BlockRef<'_>,
    in_live_loop: bool,
    inside: &mut usize,
    outside: &mut usize,
) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { .. }
            | OperationView::GuardedLoad { .. }
            | OperationView::Store { .. } => {
                if in_live_loop {
                    *inside += 1;
                } else {
                    *outside += 1;
                }
            }
            OperationView::Predicated { body, .. } => {
                count_element_access_placement(body, in_live_loop, inside, outside);
            }
            OperationView::SerialLoop(loop_ref) => {
                count_element_access_placement(loop_ref.body(), true, inside, outside);
            }
            _ => {}
        }
    }
}

#[test]
fn every_live_row_major_element_access_is_inside_its_live_range() {
    let kernel = lower_scheduled_region(&live_row_major_region(2)).unwrap();
    let mut inside = 0;
    let mut outside = 0;
    count_element_access_placement(kernel.body(), false, &mut inside, &mut outside);
    assert_eq!(inside, 2, "the one load and one store form the census");
    assert_eq!(
        outside, 0,
        "a zero-trip live range must leave no executable element access"
    );
}

const fn dense_f32_row_major_bytes(row: u64, column: u64, inner_extent: u64) -> u64 {
    4 * (row * inner_extent + column)
}

#[test]
fn declaring_a_non_input_extent_is_refused() {
    let scheduled = live_row_major_region(2);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let error = builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::new(1),
            axis: Axis::new(1),
        })
        .unwrap_err();
    assert_eq!(error, KernelBuildError::InputExtentNotInput);
}

/// Live-extent operands use the complete access list without filtering inputs.
#[test]
fn an_extent_operand_names_one_exact_epilogue_access() {
    let scheduled = mixed_epilogue_region(4);
    let declare = |access, axis| {
        KernelBuilder::new(&scheduled)
            .unwrap()
            .declare_input_extent(InputExtentParameter { access, axis })
    };

    assert_eq!(
        declare(AccessOrdinal::FIRST, Axis::new(0)),
        Err(KernelBuildError::InputExtentNotInput),
        "the staged read at access 0 is not silently skipped",
    );
    assert!(
        declare(AccessOrdinal::new(1), Axis::new(0)).is_ok(),
        "access 1 is the exact declared-input read",
    );
    assert_eq!(
        declare(AccessOrdinal::new(2), Axis::new(0)),
        Err(KernelBuildError::InputExtentNotInput),
        "the final owning write is nameable and refused as non-input",
    );
    assert_eq!(
        declare(AccessOrdinal::new(3), Axis::new(0)),
        Err(KernelBuildError::InputExtentAccessOutOfRange),
    );
    assert_eq!(
        declare(AccessOrdinal::new(1), Axis::new(1)),
        Err(KernelBuildError::InputExtentWrongAxis),
    );
}

#[test]
fn declaring_the_same_input_extent_twice_is_refused() {
    let scheduled = live_row_major_region(2);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let parameter = InputExtentParameter {
        access: AccessOrdinal::FIRST,
        axis: Axis::new(1),
    };
    builder.declare_input_extent(parameter).unwrap();
    let error = builder.declare_input_extent(parameter).unwrap_err();
    assert_eq!(error, KernelBuildError::DuplicateInputExtent);
}

#[test]
fn an_unused_live_extent_is_refused_at_verification() {
    let scheduled = live_row_major_region(2);
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    pointwise_signature(&mut builder, &scheduled, 0);
    builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(1),
        })
        .unwrap();
    let error = builder.build().unwrap_err();
    assert!(
        error.diagnostics().iter().any(|diagnostic| {
            diagnostic.rule() == "unused-input-extent" || diagnostic.rule() == "body-refinement"
        }),
        "{:?}",
        error.diagnostics()
    );
}

fn live_contraction_region(id: RegionId) -> VerifiedScheduledRegion {
    let left = Shape::from_dims([2]);
    let right = Shape::from_dims([3]);
    let output = Shape::from_dims([2, 3]);
    let contracted = Shape::from_dims([]);
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
                    sources: vec![ContractionAxisSource::Output { position: free }],
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
                kind: BoundsProofKind::LinearRange { element_count: 0 },
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
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: NAN_BITS,
            },
            numerical: numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::FIRST,
                live_axis: Axis::new(1),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .unwrap();
    builder.build().unwrap()
}

fn live_contraction_loop(kernel: &VerifiedKernel) -> SerialLoopRef<'_> {
    let guarded = kernel
        .body()
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::Predicated { body, .. } => Some(body),
            _ => None,
        })
        .expect("a guarded live contraction");
    guarded
        .operations()
        .find_map(|operation| match operation.view() {
            OperationView::SerialLoop(reduction) => Some(reduction),
            _ => None,
        })
        .expect("a live-bound contributor loop")
}

fn count_live_input_loads(
    block: BlockRef<'_>,
    live: VerifiedBufferId,
    seed: &mut u64,
    body: &mut u64,
    in_loop: bool,
) {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Load { buffer, .. } if buffer == live => {
                if in_loop {
                    *body = body.saturating_add(1);
                } else {
                    *seed = seed.saturating_add(1);
                }
            }
            OperationView::Predicated { body: nested, .. } => {
                count_live_input_loads(nested, live, seed, body, in_loop);
            }
            OperationView::SerialLoop(serial) => {
                count_live_input_loads(serial.body(), live, seed, body, true);
            }
            _ => {}
        }
    }
}

fn live_input_load_sites(kernel: &VerifiedKernel) -> (u64, u64) {
    let live_buffer = kernel
        .declared_buffers()
        .find_map(|(id, buffer)| (buffer.tensor == TensorRole::Input).then_some(id))
        .expect("the named live input is a buffer");
    let mut seed = 0;
    let mut body = 0;
    count_live_input_loads(kernel.body(), live_buffer, &mut seed, &mut body, false);
    (seed, body)
}

fn live_contraction_loads(kernel: &VerifiedKernel, bound: u64) -> u64 {
    let (seed, body) = live_input_load_sites(kernel);
    let remaining = bound
        .checked_sub(1)
        .expect("preflight must refuse an empty strict contraction before execution");
    seed.checked_add(remaining.checked_mul(body).unwrap())
        .unwrap()
}

/// Neighbouring live extents move the load-count oracle and leave identity still.
#[test]
fn a_live_contraction_consumes_s_as_the_contributor_bound_without_baking_it() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let kernel = lower_scheduled_region(&scheduled).expect("the live contraction lowers");
    let extents: Vec<_> = kernel.input_extents().collect();
    assert_eq!(extents.len(), 1);
    assert_eq!(extents[0].access, AccessOrdinal::FIRST);
    assert_eq!(extents[0].axis, Axis::new(1));

    let reduction = live_contraction_loop(&kernel);
    assert!(
        matches!(reduction.start_bound(), LoopBound::Value(_)),
        "the fold start is the first-product constant, not a baked trip count"
    );
    assert!(
        matches!(reduction.end_bound(), LoopBound::Value(_)),
        "the contributor bound must be the live operand, not a literal S"
    );

    let loads_1 = live_contraction_loads(&kernel, 1);
    let loads_14 = live_contraction_loads(&kernel, 14);
    let loads_15 = live_contraction_loads(&kernel, 15);
    assert_eq!(
        loads_1, 1,
        "S = 1 must execute exactly the unseeded fold's first contributor"
    );
    assert_eq!(
        loads_14, 14,
        "S = 14 must perform exactly 14 loads of the live input"
    );
    assert_eq!(
        loads_15, 15,
        "S = 15 must perform exactly 15 loads of the live input"
    );
    assert_ne!(
        loads_14, loads_15,
        "neighbouring extents must move the load-count oracle"
    );

    let again = lower_scheduled_region(&live_contraction_region(RegionId::new(11))).unwrap();
    assert_eq!(
        kernel.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "re-lowering the same live contraction must keep identity"
    );
    assert_eq!(
        scheduled.canonical_identity(),
        live_contraction_region(RegionId::new(12)).canonical_identity(),
        "the live value is excluded from schedule identity"
    );

    let baked_14 = lower_scheduled_region(&contraction_region(RegionId::new(9), 2, 3, 14)).unwrap();
    let baked_15 = lower_scheduled_region(&contraction_region(RegionId::new(9), 2, 3, 15)).unwrap();
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_14.canonical_identity().as_bytes(),
        "baking S = 14 must change identity"
    );
    assert_ne!(
        kernel.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking S = 15 must change identity"
    );
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change identity"
    );
    assert!(
        kernel
            .canonical_identity()
            .as_bytes()
            .starts_with(b"tiler.kernel.v9\0"),
        "the live contraction stays on the current kernel domain"
    );
}

/// Omitting the scheduled live operand fails at kernel verification.
#[test]
fn an_omitted_live_contraction_extent_is_input_extent_contract() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            builder.store(
                write,
                invocation,
                product,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("omitting the live operand must not verify");
    assert_eq!(
        error.diagnostics()[0],
        KernelDiagnostic::InputExtentContract,
        "omitted live operand: {:?}",
        error.diagnostics()
    );
    assert_eq!(error.diagnostics()[0].rule(), "input-extent-contract");
}

/// Declaring the live operand and never reading it fails at verification.
#[test]
fn an_unused_live_contraction_extent_is_unused_input_extent() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let _left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let _right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let _write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(1),
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("an unread live operand must not verify");
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.rule() == "unused-input-extent"),
        "unused live operand: {:?}",
        error.diagnostics()
    );
}

/// An axis the scheduled live input does not have is refused at the builder.
#[test]
fn a_wrong_axis_live_contraction_extent_is_refused_at_declaration() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let error = builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(5),
        })
        .expect_err("axis 5 is outside the live input's rank");
    assert_eq!(error, KernelBuildError::InputExtentWrongAxis);
    assert_eq!(format!("{error:?}"), "InputExtentWrongAxis");
}

/// A live extent introduced inside the iteration predicate is too late to bind
/// the contributor loop.
#[test]
fn a_late_phase_live_contraction_extent_is_input_extent_contract() {
    let scheduled = live_contraction_region(RegionId::new(9));
    let mut builder = KernelBuilder::new(&scheduled).unwrap();
    let left = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let right = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 0,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 6,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical()).unwrap();
    builder.requirements(scheduled.requirements()).unwrap();
    let extent = builder
        .declare_input_extent(InputExtentParameter {
            access: AccessOrdinal::FIRST,
            axis: Axis::new(1),
        })
        .unwrap();
    let (invocation, active) = guard(&mut builder, 6);
    builder
        .predicated(active, |builder| {
            let bound = builder.input_extent(extent)?;
            let start = builder.constant(KernelConstant::Index(1))?;
            let left_value = builder.load(left, invocation, BoundsWitnessId::new(0))?;
            let right_value = builder.load(right, invocation, BoundsWitnessId::new(1))?;
            let product = builder.binary(BinaryOp::F32Multiply, left_value, right_value)?;
            let seed = builder.convert(ConvertOp::CanonicalizeF32Nan, product)?;
            let results =
                builder.serial_loop_range(start, bound, &[seed], |_builder, parameters| {
                    let accumulator = parameters
                        .accumulator(0)
                        .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
                    Ok(vec![accumulator])
                })?;
            let total = results
                .get(0)
                .ok_or(KernelBuildError::EmptyLoopAccumulators)?;
            builder.store(
                write,
                invocation,
                total,
                BoundsWitnessId::new(2),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    let error = builder
        .build()
        .expect_err("a live operand read inside the predicate is too late");
    assert!(
        error.diagnostics().iter().any(|diagnostic| {
            diagnostic.rule() == "input-extent-contract"
                || diagnostic.rule() == "reduction-contract"
        }),
        "late-phase live operand: {:?}",
        error.diagnostics()
    );
}

fn operand_tile(rounds: u64) -> CooperativeTile {
    let block = 16;
    let participants = ParticipantSpace::new(&[block, block]).expect("rank two");
    let range = ParticipantRange {
        first: 0,
        count: block * block,
    };
    let a = StagingId::FIRST;
    let b = StagingId::new(1);
    let tile = CooperativeTile {
        coordinates: LocalCoordinates {
            source: LocalCoordinateSource::LocalWorkgroupPosition,
            participants,
        },
        rounds,
        staging: vec![
            WorkgroupStaging {
                id: a,
                element: StagedElement::F32,
                slots: block * block,
                live_from: PhaseId::FIRST,
                live_through: PhaseId::new(1),
            },
            WorkgroupStaging {
                id: b,
                element: StagedElement::F32,
                slots: block * block,
                live_from: PhaseId::FIRST,
                live_through: PhaseId::new(1),
            },
        ],
        phases: vec![
            CooperativePhase {
                id: PhaseId::FIRST,
                participation: range,
                writes: vec![
                    StagedWrite {
                        staging: a,
                        span: StagedSpan::new(&[block, 1], 0, 1).expect("rank two"),
                    },
                    StagedWrite {
                        staging: b,
                        span: StagedSpan::new(&[1, block], 0, 1).expect("rank two"),
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
                        span: StagedSpan::new(&[block, 0], 0, block).expect("rank two"),
                    },
                    StagedRead {
                        staging: b,
                        span: StagedSpan::new(&[0, block], 0, block).expect("rank two"),
                    },
                ],
            },
        ],
        synchronization: Vec::new(),
        commit: range,
    };
    let subject = crate::schedule::required_subject(&tile.visibility_edges())
        .expect("the handoff states one subject");
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

fn cooperative_contraction_region(
    output_m: u64,
    output_n: u64,
    contracted: u64,
    tail: TailPolicy,
) -> VerifiedScheduledRegion {
    let block = 16;
    let admitted = match tail {
        TailPolicy::Exact => {
            let admitted = crate::schedule::admit_exact_cooperative_contraction(
                &Shape::from_dims([output_m, output_n]),
                &Shape::from_dims([block, block]),
                &Shape::from_dims([contracted]),
                &Shape::from_dims([block]),
            )
            .expect("exact admission");
            (
                admitted.binding,
                admitted.contracted_tile,
                admitted.rounds,
                output_m * output_n,
                output_m * output_n,
            )
        }
        TailPolicy::Predicated => {
            let admitted = crate::schedule::admit_predicated_cooperative_contraction(
                &Shape::from_dims([output_m, output_n]),
                &Shape::from_dims([block, block]),
                &Shape::from_dims([contracted]),
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
    let output = Shape::from_dims([output_m, output_n]);
    let contracted_shape = Shape::from_dims([contracted]);
    let operand_map = |free_position, operand: Shape| LogicalAccess::ContractionOperand {
        operand_shape: operand,
        output_shape: output.clone(),
        contracted_shape: contracted_shape.clone(),
        sources: vec![
            ContractionAxisSource::Output {
                position: free_position,
            },
            ContractionAxisSource::Contracted { position: 0 },
        ],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(21));
    builder.iteration_shape(output.clone()).unwrap();
    for (witness, extent, map) in [
        (
            0,
            output_m * contracted,
            operand_map(0, Shape::from_dims([output_m, contracted])),
        ),
        (
            1,
            output_n * contracted,
            operand_map(1, Shape::from_dims([output_n, contracted])),
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

fn guarded_load_count(kernel: &VerifiedKernel) -> usize {
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

fn declining_backend(operation: OperationView<'_>) -> Result<(), &'static str> {
    match operation {
        OperationView::GuardedLoad { .. } => Err("unrecognized-operation"),
        _ => Ok(()),
    }
}

fn count_declined_guarded_loads(block: BlockRef<'_>, declined: &mut usize) {
    for operation in block.operations() {
        if declining_backend(operation.view()).is_err() {
            *declined = declined.saturating_add(1);
        }
        match operation.view() {
            OperationView::Predicated { body, .. } => count_declined_guarded_loads(body, declined),
            OperationView::SerialLoop(serial) => {
                count_declined_guarded_loads(serial.body(), declined);
            }
            _ => {}
        }
    }
}

/// Predicated and Exact kernels under the same binding stay distinct, and
/// Predicated carries `GuardedLoad`.
#[test]
fn predicated_contraction_lowers_with_guarded_loads() {
    let exact = cooperative_contraction_region(32, 32, 16, TailPolicy::Exact);
    let predicated = cooperative_contraction_region(32, 32, 16, TailPolicy::Predicated);
    let exact_kernel = lower_scheduled_region(&exact).expect("exact tiled contraction lowers");
    let predicated_kernel =
        lower_scheduled_region(&predicated).expect("predicated tiled contraction lowers");
    assert_eq!(guarded_load_count(&exact_kernel), 0);
    assert!(guarded_load_count(&predicated_kernel) >= 2);
    assert_ne!(
        exact_kernel.canonical_identity().as_bytes(),
        predicated_kernel.canonical_identity().as_bytes()
    );
}

/// A backend that does not name `GuardedLoad` declines; there is no Load rewrite.
#[test]
fn a_backend_that_declines_guarded_load_has_no_source_fallback() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let kernel = lower_scheduled_region(&scheduled).expect("predicated kernel lowers");
    let mut declined = 0_usize;
    count_declined_guarded_loads(kernel.body(), &mut declined);
    assert!(declined >= 2, "declined {declined} GuardedLoad operations");
    assert!(
        !kernel
            .body()
            .operations()
            .any(|operation| { matches!(operation.view(), OperationView::Load { .. }) }),
        "no ordinary operand load may stand in for GuardedLoad"
    );
}

fn swap_guarded_load_predicates(data: &mut super::model::KernelData) {
    for block in &mut data.blocks {
        let mut predicates = Vec::new();
        for operation in &block.operations {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = operation.kind {
                predicates.push(predicate);
            }
        }
        if predicates.len() < 2 {
            continue;
        }
        let first = predicates[0];
        let second = predicates[1];
        for operation in &mut block.operations {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = &mut operation.kind
            {
                if *predicate == first {
                    *predicate = second;
                } else if *predicate == second {
                    *predicate = first;
                }
            }
        }
    }
}

fn replace_guarded_with_ordinary(data: &mut super::model::KernelData) {
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::model::OperationKind::GuardedLoad {
                buffer,
                offset,
                bounds,
                ..
            } = operation.kind
            {
                operation.kind = super::model::OperationKind::Load {
                    buffer,
                    offset,
                    bounds,
                };
            }
        }
    }
}

fn enclose_staged_stores(data: &mut super::model::KernelData) {
    let Some(predicate) = data.blocks.iter().find_map(|block| {
        block.operations.iter().find_map(|operation| {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = operation.kind {
                Some(predicate)
            } else {
                None
            }
        })
    }) else {
        return;
    };
    let mut stores = Vec::new();
    if let Some(block) = data.blocks.get_mut(0) {
        let rest = std::mem::take(&mut block.operations);
        let mut kept = Vec::new();
        for operation in rest {
            if matches!(
                operation.kind,
                super::model::OperationKind::StagedStore { .. }
            ) {
                stores.push(operation);
            } else {
                kept.push(operation);
            }
        }
        block.operations = kept;
    }
    if stores.is_empty() {
        return;
    }
    let body = u32::try_from(data.blocks.len()).expect("block index fits");
    data.blocks.push(super::model::BlockData {
        parameters: Vec::new(),
        operations: stores,
    });
    data.blocks[0].operations.push(super::model::OperationData {
        kind: super::model::OperationKind::Predicated { predicate, body },
        results: Vec::new(),
    });
}

fn enclose_barriers(data: &mut super::model::KernelData) {
    let Some(predicate) = data.blocks.iter().find_map(|block| {
        block.operations.iter().find_map(|operation| {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = operation.kind {
                Some(predicate)
            } else {
                None
            }
        })
    }) else {
        return;
    };
    let mut barriers = Vec::new();
    if let Some(block) = data.blocks.get_mut(0) {
        let rest = std::mem::take(&mut block.operations);
        let mut kept = Vec::new();
        for operation in rest {
            if matches!(operation.kind, super::model::OperationKind::Barrier { .. }) {
                barriers.push(operation);
            } else {
                kept.push(operation);
            }
        }
        block.operations = kept;
    }
    if barriers.is_empty() {
        return;
    }
    let body = u32::try_from(data.blocks.len()).expect("block index fits");
    data.blocks.push(super::model::BlockData {
        parameters: Vec::new(),
        operations: barriers,
    });
    data.blocks[0].operations.push(super::model::OperationData {
        kind: super::model::OperationKind::Predicated { predicate, body },
        results: Vec::new(),
    });
}

fn drop_inner_store_guard(data: &mut super::model::KernelData) {
    // Replace the innermost Predicated-around-store with its body operations
    // spliced into the parent, so the store has only one axis guard.
    let mut splice: Option<(usize, usize, u32)> = None;
    for (block_index, block) in data.blocks.iter().enumerate() {
        for (op_index, operation) in block.operations.iter().enumerate() {
            if let super::model::OperationKind::Predicated { body, .. } = operation.kind
                && data.blocks.get(body as usize).is_some_and(|inner| {
                    inner.operations.iter().any(|nested| {
                        matches!(nested.kind, super::model::OperationKind::Store { .. })
                    })
                })
            {
                splice = Some((block_index, op_index, body));
            }
        }
    }
    let Some((block_index, op_index, body)) = splice else {
        return;
    };
    let inner = std::mem::take(&mut data.blocks[body as usize].operations);
    data.blocks[block_index].operations.remove(op_index);
    for (offset, operation) in inner.into_iter().enumerate() {
        data.blocks[block_index]
            .operations
            .insert(op_index + offset, operation);
    }
}

fn verify_mutated(
    scheduled: &VerifiedScheduledRegion,
    edit: impl FnOnce(&mut super::model::KernelData),
) -> KernelDiagnostic {
    let mut data = super::lower::derive_canonical(
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect("canonical body exists");
    edit(&mut data);
    super::verify::verify_kernel(
        &data,
        scheduled.region(),
        scheduled.canonical_identity(),
        scheduled.requirements(),
    )
    .expect_err("the mutated subject must fail")
}

fn set_first_guarded_predicate(data: &mut super::model::KernelData, use_column: bool) {
    let axis = {
        let mut row = None;
        let mut column = None;
        for block in &data.blocks {
            for operation in &block.operations {
                if let super::model::OperationKind::Compare {
                    op: super::model::CompareOp::IndexLessThan,
                    ..
                } = operation.kind
                {
                    if row.is_none() {
                        row = operation.results.first().copied();
                    } else if column.is_none() {
                        column = operation.results.first().copied();
                    }
                }
            }
        }
        (row, column)
    };
    let wanted = if use_column { axis.1 } else { axis.0 };
    let Some(wanted) = wanted else {
        return;
    };
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = &mut operation.kind
            {
                *predicate = wanted;
                return;
            }
        }
    }
}

fn set_second_guarded_predicate(data: &mut super::model::KernelData, use_row: bool) {
    let axis = {
        let mut row = None;
        let mut column = None;
        for block in &data.blocks {
            for operation in &block.operations {
                if let super::model::OperationKind::Compare {
                    op: super::model::CompareOp::IndexLessThan,
                    ..
                } = operation.kind
                {
                    if row.is_none() {
                        row = operation.results.first().copied();
                    } else if column.is_none() {
                        column = operation.results.first().copied();
                    }
                }
            }
        }
        (row, column)
    };
    let wanted = if use_row { axis.0 } else { axis.1 };
    let Some(wanted) = wanted else {
        return;
    };
    let mut seen = 0_u8;
    for block in &mut data.blocks {
        for operation in &mut block.operations {
            if let super::model::OperationKind::GuardedLoad { predicate, .. } = &mut operation.kind
            {
                seen = seen.saturating_add(1);
                if seen == 2 {
                    *predicate = wanted;
                    return;
                }
            }
        }
    }
}

/// A column guard on the left load is the left-load refusal.
#[test]
fn a_column_guard_on_the_left_load_is_the_left_load_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, |data| set_first_guarded_predicate(data, true));
    assert_eq!(
        diagnostic.rule(),
        "left-load-guard",
        "left-load refusal: {}",
        diagnostic.rule()
    );
}

/// A row guard on the right load is the right-load refusal.
#[test]
fn a_row_guard_on_the_right_load_is_the_right_load_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, |data| set_second_guarded_predicate(data, true));
    assert_eq!(
        diagnostic.rule(),
        "right-load-guard",
        "right-load refusal: {}",
        diagnostic.rule()
    );
}

/// Swapping row and column predicates names a specific load refusal.
#[test]
fn swapped_axis_guards_name_the_left_and_right_load_refusals() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, swap_guarded_load_predicates);
    let text = diagnostic.rule();
    assert!(
        text == "left-load-guard" || text == "right-load-guard",
        "swapped guards: {text}"
    );
}

/// An ordinary load in place of either `GuardedLoad` fails bounds refinement.
#[test]
fn ordinary_load_in_place_of_guarded_load_fails_bounds_refinement() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, replace_guarded_with_ordinary);
    assert_eq!(
        diagnostic.rule(),
        "bounds-evidence",
        "ordinary load refusal: {}",
        diagnostic.rule()
    );
}

/// Guarding a staged store is incomplete staging.
#[test]
fn a_predicated_staged_store_is_incomplete_staging() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, enclose_staged_stores);
    assert_eq!(
        diagnostic.rule(),
        "incomplete-staging",
        "guarded staged store: {}",
        diagnostic.rule()
    );
}

/// A phase barrier under a predicate fails convergence.
#[test]
fn a_barrier_under_a_predicate_fails_convergence() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, enclose_barriers);
    assert_eq!(
        diagnostic.rule(),
        "synchronization-convergence",
        "guarded barrier: {}",
        diagnostic.rule()
    );
}

/// Dropping one store-side axis guard is the write refusal.
#[test]
fn an_incomplete_output_guard_is_the_write_refusal() {
    let scheduled = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    let diagnostic = verify_mutated(&scheduled, drop_inner_store_guard);
    assert_eq!(
        diagnostic.rule(),
        "output-store-guard",
        "write refusal: {}",
        diagnostic.rule()
    );
}

/// Every active output has one writer; inactive invocations write nothing;
/// staging is initialized; filler is not observed.
#[test]
fn predicated_contraction_ownership_and_filler_are_unobservable() {
    let scheduled = cooperative_contraction_region(10, 16, 16, TailPolicy::Predicated);
    let kernel = lower_scheduled_region(&scheduled).expect("predicated kernel lowers");
    let m_ext = 10_usize;
    let n_ext = 16_usize;
    let k_ext = 16_usize;
    let block = 16_usize;
    let grid = usize::try_from(scheduled.region().schedule.launch.grid_threads).unwrap();
    let mut left = vec![0.0_f32; m_ext.saturating_mul(k_ext)];
    let mut right = vec![0.0_f32; n_ext.saturating_mul(k_ext)];
    for m in 0..m_ext {
        for k in 0..k_ext {
            left[m.saturating_mul(k_ext).saturating_add(k)] =
                f32::from(u8::try_from(m.saturating_add(1)).expect("m fits u8"));
        }
    }
    for n in 0..n_ext {
        for k in 0..k_ext {
            right[n.saturating_mul(k_ext).saturating_add(k)] =
                f32::from(u8::try_from(n.saturating_add(2)).expect("n fits u8"));
        }
    }
    let mut output = vec![f32::NAN; m_ext.saturating_mul(n_ext)];
    let mut writers = vec![0_u32; m_ext.saturating_mul(n_ext)];
    let mut a_tile = vec![f32::NAN; block.saturating_mul(block)];
    let mut b_tile = vec![f32::NAN; block.saturating_mul(block)];
    for gid in 0..grid {
        let lid = gid % block.saturating_mul(block);
        let local_n = lid % block;
        let local_m = lid / block;
        let row_active = local_m < m_ext;
        let col_active = local_n < n_ext;
        let a = if row_active {
            left[local_m.saturating_mul(k_ext).saturating_add(local_n)]
        } else {
            0.0
        };
        let b = if col_active {
            right[local_n.saturating_mul(k_ext).saturating_add(local_m)]
        } else {
            0.0
        };
        a_tile[local_m.saturating_mul(block).saturating_add(local_n)] = a;
        b_tile[local_n.saturating_mul(block).saturating_add(local_m)] = b;
    }
    assert!(
        a_tile.iter().all(|value| !value.is_nan()),
        "every A staging slot is initialized"
    );
    assert!(
        b_tile.iter().all(|value| !value.is_nan()),
        "every B staging slot is initialized"
    );
    for gid in 0..grid {
        let lid = gid % block.saturating_mul(block);
        let local_n = lid % block;
        let local_m = lid / block;
        if local_m >= m_ext || local_n >= n_ext {
            continue;
        }
        let mut acc = 0.0_f32;
        for kk in 0..block {
            let a = a_tile[local_m.saturating_mul(block).saturating_add(kk)];
            let b = b_tile[local_n.saturating_mul(block).saturating_add(kk)];
            assert!(
                a != 0.0 || left[local_m.saturating_mul(k_ext).saturating_add(kk)] == 0.0,
                "active output observed an inactive A filler"
            );
            acc += a * b;
        }
        let slot = local_m.saturating_mul(n_ext).saturating_add(local_n);
        output[slot] = acc;
        writers[slot] = writers[slot].saturating_add(1);
    }
    for (slot, count) in writers.iter().enumerate() {
        assert_eq!(*count, 1, "output slot {slot} writers={count}");
        let m = slot / n_ext;
        let n = slot % n_ext;
        let expected: f32 = (0..k_ext)
            .map(|k| {
                left[m.saturating_mul(k_ext).saturating_add(k)]
                    * right[n.saturating_mul(k_ext).saturating_add(k)]
            })
            .sum();
        assert!(
            (output[slot] - expected).abs() <= f32::EPSILON,
            "slot {slot}"
        );
    }
    let _ = kernel;
}

/// Tail and `GuardedLoad` tags move identity without touching old Exact pins.
#[test]
fn tail_and_guarded_load_tags_are_identity_bearing() {
    let exact = cooperative_contraction_region(32, 32, 16, TailPolicy::Exact);
    let predicated = cooperative_contraction_region(32, 32, 16, TailPolicy::Predicated);
    let partial = cooperative_contraction_region(10, 32, 16, TailPolicy::Predicated);
    assert_ne!(
        exact.canonical_identity().as_bytes(),
        predicated.canonical_identity().as_bytes()
    );
    assert_ne!(
        predicated.canonical_identity().as_bytes(),
        partial.canonical_identity().as_bytes()
    );
    let exact_kernel = lower_scheduled_region(&exact).unwrap();
    let predicated_kernel = lower_scheduled_region(&predicated).unwrap();
    assert_ne!(
        exact_kernel.canonical_identity().as_bytes(),
        predicated_kernel.canonical_identity().as_bytes()
    );
}

/// A moved topology field separates schedule and kernel identity together.
///
/// The ADR 0013 topology perturbation at the layers this crate owns: the
/// single-round and loop-carried cooperative regions differ only in the
/// coherent field set a verifiable `rounds` change forces — the round count,
/// its per-round contributor partition, and its round-boundary synchronization
/// — while inputs, accesses, and expression stay the fixture's own bytes. The
/// scheduled-region identities separate, each lowered kernel retains exactly
/// its own region's identity, and the kernel identities separate with them,
/// which is the chain that carries a topology choice into kernel-program,
/// artifact, and envelope identity. The per-field population — that *every*
/// cooperative tile field separates scheduled-region identity on its own — is
/// `schedule::builder`'s `every_cooperative_tile_field_separates_scheduled_region_identity`.
#[test]
fn a_topology_change_separates_schedule_and_kernel_identity_together() {
    let single = cooperative_region();
    let multi = multi_round_cooperative_region();
    assert_ne!(
        single.canonical_identity().as_bytes(),
        multi.canonical_identity().as_bytes(),
        "two topologies are two scheduled-region identities",
    );
    let single_kernel =
        lower_scheduled_region(&single).expect("the single-round cooperative region lowers");
    let multi_kernel =
        lower_scheduled_region(&multi).expect("the loop-carried cooperative region lowers");
    assert_eq!(
        single_kernel.scheduled_region_identity().as_bytes(),
        single.canonical_identity().as_bytes(),
        "a kernel retains exactly its own region's identity",
    );
    assert_eq!(
        multi_kernel.scheduled_region_identity().as_bytes(),
        multi.canonical_identity().as_bytes(),
        "a kernel retains exactly its own region's identity",
    );
    assert_ne!(
        single_kernel.canonical_identity().as_bytes(),
        multi_kernel.canonical_identity().as_bytes(),
        "the topology choice is folded through kernel identity",
    );
}
