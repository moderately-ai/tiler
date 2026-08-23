use super::super::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BufferAccess, BufferParameter, Builtin,
    CompareOp, ExecutionScope, KernelBuilder, KernelConstant, KernelDiagnostic, KernelType,
    MemoryScope,
};
use super::support::{
    BIAS_BITS, SCALE_BITS, diagnostics, guard, numerical, pointwise_region, pointwise_signature,
    scale_bias,
};
use crate::schedule::{BoundsWitnessId, OwnershipWitnessId, RegionId, SyncPointId, TensorRole};
use crate::shape::Shape;

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
