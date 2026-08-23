use super::super::{InputExtentParameter, KernelBuildError, KernelBuilder};
use super::support::{linear_schedule, live_row_major_region, numerical, pointwise_signature};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32ExpressionBuilder, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    TensorRole, VerifiedScheduledRegion,
};
use crate::shape::{Axis, Shape};

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
