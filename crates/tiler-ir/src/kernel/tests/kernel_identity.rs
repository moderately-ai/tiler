use super::super::{
    AddressSpace, BlockRef, BufferAccess, BufferParameter, Builtin, KernelBuilder, KernelType,
    OperationView, VerifiedBufferId, VerifiedKernel, VerifiedKernelHandleError,
    lower_scheduled_region,
};
use super::support::{
    ABSENT_SUBGROUP_KERNEL_IDENTITY_HEX, canonical_pointwise, guard, numerical, pointwise_region,
    reduction_region, scale_bias,
};
use crate::schedule::{
    ArithmeticType, BoundsWitnessId, OwnershipWitnessId, RegionId, SubgroupRealizationSubject,
    SubgroupTransfer, SubgroupWidth, TensorRole, VerifiedScheduledRegion,
};
use crate::shape::{Axis, Shape};
use std::fmt::Write as _;

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
