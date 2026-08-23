use super::super::{
    AddressSpace, BufferAccess, BufferParameter, Builtin, KernelBuilder, KernelDiagnostic,
    KernelType,
};
use super::support::{
    COOPERATIVE_STAGING, cooperative_diagnostic, cooperative_region, numerical, reduction_region,
};
use crate::schedule::{
    NumericalPermission, NumericalRealization, RegionId, TensorRole, VerifiedScheduledRegion,
};
use crate::shape::{Axis, Shape};

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
