//! Partial-view alignment: a stage access is checked against the natural alignment
//! of the scalar it reads or writes, not only against the value's own allocation.

use super::super::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    KernelProgramBuildError, KernelProgramBuilder, MaterializedOrigin, MemorySpace, StorageScalar,
    ValueRole, VerifiedKernelProgram, ViewId,
};
use super::support::{
    SCALE_BITS, checked_coverage, coverage_range, declare_program_contract, device, fixture_abi,
    input_shape, output_shape, pointwise_kernel, program_input, read, reduction_kernel,
    serial_sum_program, strict_contract, value, write_access,
};
use crate::semantic::OutputKey;
use crate::shape::Shape;

/// A two-stage serial-sum whose temporary is larger than the working set so a
/// partial window can start at a chosen byte offset.
fn push_partial_temporary_stage(
    offset: u64,
) -> Result<(KernelProgramBuilder, ViewId), KernelProgramBuildError> {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let source_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("input allocation");
    let temporary_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 32,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            source_allocation,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                Shape::from_dims([8]),
            ),
            temporary_allocation,
        )
        .expect("oversized temporary");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder.push_view(temporary, ByteWindow { offset, length: 24 })?;
    let _output_view = builder.push_whole_view(output).expect("output view");
    builder.push_stage(
        &pointwise_kernel(0, SCALE_BITS),
        &coverage_range(&coverage, 0..4),
        &[
            read(source_view, abi.input_bytes),
            write_access(temporary_view, abi.input_bytes),
        ],
        abi.pointwise_launch(),
    )?;
    Ok((builder, temporary_view))
}

fn complete_partial_temporary_program(offset: u64) -> VerifiedKernelProgram {
    let semantic = serial_sum_program(SCALE_BITS);
    let coverage = checked_coverage(&semantic, &strict_contract());
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let source_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("input allocation");
    let temporary_allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 32,
            alignment: AlignmentGuarantee::new(16).expect("16 is a power of two"),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let source = builder
        .push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            source_allocation,
        )
        .expect("input value");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                Shape::from_dims([8]),
            ),
            temporary_allocation,
        )
        .expect("oversized temporary");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            output_allocation,
        )
        .expect("output value");
    let source_view = builder.push_whole_view(source).expect("input view");
    let temporary_view = builder
        .push_view(temporary, ByteWindow { offset, length: 24 })
        .expect("partial temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    let pointwise = builder
        .push_stage(
            &pointwise_kernel(0, SCALE_BITS),
            &coverage_range(&coverage, 0..4),
            &[
                read(source_view, abi.input_bytes),
                write_access(temporary_view, abi.input_bytes),
            ],
            abi.pointwise_launch(),
        )
        .expect("pointwise stage");
    let reduction = builder
        .push_stage(
            &reduction_kernel(1),
            &coverage_range(&coverage, 4..5),
            &[
                read(temporary_view, abi.input_bytes),
                write_access(output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("reduction stage");
    builder
        .push_data_dependency(pointwise, reduction, temporary)
        .expect("data dependency");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);
    builder.build().expect("verified partial-view program")
}

#[test]
fn a_naturally_aligned_partial_f32_view_builds() {
    let program = complete_partial_temporary_program(4);
    let temporary = program
        .views()
        .find(|view| view.window().offset == 4)
        .expect("the partial temporary view");
    assert_eq!(temporary.alignment().bytes(), 4);
    assert!(
        temporary
            .alignment()
            .satisfies(AlignmentRequirement::natural_for(StorageScalar::F32))
    );
}

#[test]
fn a_one_byte_shifted_f32_view_fails_before_the_stage_is_verified() {
    let error = push_partial_temporary_stage(1)
        .expect_err("a one-byte-shifted F32 view must not reach artifact construction");
    assert_eq!(
        error,
        KernelProgramBuildError::StageAccessAlignment {
            position: 1,
            required: AlignmentRequirement::natural_for(StorageScalar::F32),
            guaranteed: AlignmentGuarantee::new(1).expect("1 is a power of two"),
        }
    );
}
