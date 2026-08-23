use super::super::{
    BinaryOp, ConvertOp, KernelType, OperationView, PackedExtractOp, lower_scheduled_region,
};
use super::support::{linear_schedule, numerical, region_numerical_mut};
use crate::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExceptionalValueAssumption,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionBuilder, TensorRole, VerifiedScheduledRegion,
};
use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
use crate::shape::Shape;

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
        .map(super::super::model::OperationRef::view)
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
