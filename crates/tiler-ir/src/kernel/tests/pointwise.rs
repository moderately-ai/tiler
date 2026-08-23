use super::super::{
    AddressSpace, BinaryOp, BlockRef, BufferAccess, BufferParameter, Builtin, ConvertOp,
    KernelConstant, KernelDiagnostic, KernelLoweringError, KernelType, OperationRef, OperationView,
    VerifiedBufferId, VerifiedKernel, lower_scheduled_region,
};
use super::support::{linear_schedule, numerical, pointwise_expression_region, pointwise_region};
use crate::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ExecutionBinding, LogicalAccess, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseF32ExpressionBuilder, RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder,
    TensorRole, VerifiedScheduledRegion,
};
use crate::shape::Shape;

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
