use super::super::{
    AddressSpace, BinaryOp, BufferAccess, BufferParameter, Builtin, KernelBuilder, KernelConstant,
    KernelDiagnostic, KernelType, OperationView, lower_scheduled_region,
};
use super::support::{diagnostics, guard, numerical, reduction_region};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, RegionId, TensorRole};
use crate::shape::{Axis, Shape};

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
