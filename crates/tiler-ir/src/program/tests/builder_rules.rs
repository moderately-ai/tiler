//! `KernelProgramBuilder` structural validation: handles, dependencies, storage,
//! and stage-access rules the builder checks independently of any one contract.

use super::super::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    KernelProgramBuildError, KernelProgramBuilder, KernelProgramDiagnostic,
    MaterializedComponentSpec, MaterializedOrigin, MemorySpace, ProgramEntityKind, StageAccessMode,
    StageLaunch, StorageEncoding, StorageScalar, ValueRole,
};
use super::support::{
    OTHER_SCALE_BITS, SCALE_BITS, TwoStageShape, complete_two_stage, declare_program_contract,
    device, diagnostic, fixture_abi, input_shape, linear_schedule, literal, occurrences,
    output_shape, pointwise_kernel, program_input, publish_two_chain, read, reduction_kernel,
    serial_sum_program, strict, two_chain, two_chain_program, two_stage, value, write_access,
};
use crate::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use crate::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, LogicalAccess,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, RegionId, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, TensorRole,
};
use crate::semantic::{
    EncodedComponentRole, InputKey, OperationAttributes, OutputKey, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, SemanticProgram,
    SemanticProgramBuilder, StrictAffineU4, dequantize_strict_affine_op,
};
use crate::shape::Shape;

fn strict_affine_u4_passthrough_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input::<StrictAffineU4>(InputKey::new("input").expect("key"), Shape::from_dims([5]))
        .expect("encoded input");
    draft
        .output(OutputKey::new("result").expect("key"), input)
        .expect("encoded output");
    draft.build().expect("verified semantic program")
}

fn strict_affine_u4_dequantize_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
    let input = draft
        .input_resolved(
            InputKey::new("input").expect("key"),
            Shape::from_dims([5]),
            StrictAffineU4::resolved_type(),
        )
        .expect("encoded input");
    let output = draft
        .apply(
            dequantize_strict_affine_op(),
            OperationAttributes::empty(),
            &[input],
        )
        .expect("strict affine dequantization")[0];
    draft
        .output_resolved(OutputKey::new("result").expect("key"), output)
        .expect("dense output");
    draft.build().expect("verified semantic program")
}

fn strict_affine_u4_dequantize_kernel() -> VerifiedKernel {
    let logical_elements = 5;
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(17));
    builder
        .iteration_shape(Shape::from_dims([logical_elements]))
        .expect("iteration shape");
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
            ownership: Some(owner),
        },
    ] {
        builder.push_access(access).expect("access");
    }
    for (id, tensor, component_role, element_count) in [
        (
            0,
            TensorRole::Input,
            Some(STRICT_AFFINE_CODES_ROLE),
            logical_elements.div_ceil(2),
        ),
        (1, TensorRole::Input, Some(STRICT_AFFINE_SCALE_ROLE), 1),
        (2, TensorRole::Input, Some(STRICT_AFFINE_ZERO_POINT_ROLE), 1),
        (3, TensorRole::Output, None, logical_elements),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(id),
                tensor,
                component_role,
                kind: BoundsProofKind::LinearRange { element_count },
            })
            .expect("bounds proof");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: logical_elements,
            },
        })
        .expect("ownership proof");
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictAffineU4Dequantize {
                codes_role: STRICT_AFFINE_CODES_ROLE,
                scale_role: STRICT_AFFINE_SCALE_ROLE,
                zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            },
            numerical: strict(),
        })
        .expect("scalar program");
    builder
        .schedule(linear_schedule(logical_elements, owner))
        .expect("schedule");
    lower_scheduled_region(&builder.build().expect("verified schedule"))
        .expect("verified structured kernel")
}

#[test]
fn a_read_without_its_declared_data_dependency_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_output(OutputKey::new("result").expect("key"), wired.output)
        .expect("named output");
    assert_eq!(
        diagnostic(wired.builder),
        KernelProgramDiagnostic::MissingDataDependency
    );
}

#[test]
fn a_dependency_that_states_an_unrealized_obligation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // A handoff on an allocation holding a single value can never release
    // storage from one value to another: the edge names an obligation its two
    // stages do not realize.
    wired
        .builder
        .push_storage_handoff(wired.reduction, wired.pointwise, wired.temporary_allocation)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MisattributedDependency
    );
}

#[test]
fn an_output_may_not_share_storage_with_another_value() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::SharedOutputStorage);
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::ForbiddenAlias
    );
}

#[test]
fn an_unused_view_or_allocation_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_view(
            wired.temporary,
            ByteWindow {
                offset: 0,
                length: 4,
            },
        )
        .expect("the view is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedView
    );

    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    wired
        .builder
        .push_allocation(device(64, AllocationOwnership::Program))
        .expect("the allocation is locally well formed");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::UnusedAllocation
    );
}

#[test]
fn two_indistinguishable_entities_make_identity_ambiguous_and_are_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    // Two allocations with identical content and identical (empty) bindings
    // cannot be told apart by any canonical key, so identity would be
    // ambiguous rather than merely redundant.
    for _ in 0..2 {
        wired
            .builder
            .push_allocation(device(64, AllocationOwnership::Program))
            .expect("the allocation is locally well formed");
    }
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::AmbiguousCanonicalKey {
            entity: ProgramEntityKind::Allocation,
        }
    );
}

#[test]
fn a_value_with_no_writer_or_two_writers_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);

    // No writer: the reduction stage alone reads a temporary nobody defines.
    let mut builder = KernelProgramBuilder::new(&semantic).expect("builder");
    let abi = fixture_abi(&mut builder);
    let owned = builder
        .push_allocation(device(24, AllocationOwnership::Program))
        .expect("temporary allocation");
    let output_allocation = builder
        .push_allocation(device(8, AllocationOwnership::Program))
        .expect("output allocation");
    let temporary = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                input_shape(),
            ),
            owned,
        )
        .expect("temporary value");
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
    let temporary_view = builder.push_whole_view(temporary).expect("temporary view");
    let output_view = builder.push_whole_view(output).expect("output view");
    builder
        .push_stage(
            &reduction_kernel(1),
            &occurrences(&semantic, 0..5),
            &[
                read(temporary_view, abi.input_bytes),
                write_access(output_view, abi.output_bytes),
            ],
            abi.reduction_launch(),
        )
        .expect("reduction stage");
    builder
        .push_output(OutputKey::new("result").expect("key"), output)
        .expect("named output");
    declare_program_contract(&mut builder);
    assert_eq!(diagnostic(builder), KernelProgramDiagnostic::MissingWriter);

    // Two writers: a third stage redefines the temporary the pointwise stage
    // already fully initializes.
    let mut wired = two_stage(&semantic, TwoStageShape::ReservedCoverage);
    wired
        .builder
        .push_stage(
            &pointwise_kernel(2, OTHER_SCALE_BITS),
            &occurrences(&semantic, 4..5),
            &[
                read(wired.source_view, wired.abi.input_bytes),
                write_access(wired.temporary_view, wired.abi.input_bytes),
            ],
            wired.abi.pointwise_launch(),
        )
        .expect("second writing stage");
    assert_eq!(
        diagnostic(complete_two_stage(wired)),
        KernelProgramDiagnostic::MultipleWriters
    );
}

#[test]
fn a_handle_minted_by_another_program_builder_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let foreign = two_stage(&semantic, TwoStageShape::Canonical);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);

    assert_eq!(
        wired
            .builder
            .push_data_dependency(wired.pointwise, wired.reduction, foreign.temporary)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_data_dependency(foreign.pointwise, wired.reduction, wired.temporary)
            .expect_err("a foreign stage handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Stage,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_whole_view(foreign.source)
            .expect_err("a foreign value handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Value,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_storage_handoff(wired.pointwise, wired.reduction, foreign.output_allocation)
            .expect_err("a foreign allocation handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::Allocation,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(3, SCALE_BITS),
                &occurrences(&semantic, 4..5),
                &[
                    read(foreign.source_view, wired.abi.input_bytes),
                    write_access(foreign.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("a foreign view handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::View,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &pointwise_kernel(3, SCALE_BITS),
                &occurrences(&semantic, 4..5),
                &[
                    read(wired.source_view, foreign.abi.input_bytes),
                    write_access(wired.temporary_view, wired.abi.input_bytes),
                ],
                wired.abi.pointwise_launch(),
            )
            .expect_err("a foreign ABI expression handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::AbiExpression,
        }
    );
    assert_eq!(
        wired
            .builder
            .applicability_guard(foreign.abi.input_bytes)
            .expect_err("a foreign ABI expression handle is rejected"),
        KernelProgramBuildError::ForeignHandle {
            entity: ProgramEntityKind::AbiExpression,
        }
    );
}

#[test]
fn a_stage_access_must_realize_its_bound_kernel_signature() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::ShiftedCoverage);
    let kernel = pointwise_kernel(2, OTHER_SCALE_BITS);

    let bytes = wired.abi.input_bytes;
    let launch = wired.abi.pointwise_launch();
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[read(wired.source_view, bytes)],
                launch,
            )
            .expect_err("access arity is checked"),
        KernelProgramBuildError::StageAccessArity {
            expected: 2,
            actual: 1,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[
                    read(wired.temporary_view, bytes),
                    write_access(wired.temporary_view, bytes),
                ],
                launch,
            )
            .expect_err("tensor roles are checked"),
        KernelProgramBuildError::StageTensorRole {
            position: 0,
            expected: TensorRole::Input,
            actual: ValueRole::Temporary,
        }
    );
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[
                    write_access(wired.source_view, bytes),
                    write_access(wired.temporary_view, bytes),
                ],
                launch,
            )
            .expect_err("access modes are checked"),
        KernelProgramBuildError::StageAccessMode {
            position: 0,
            expected: StageAccessMode::Read,
            actual: StageAccessMode::Write,
        }
    );

    let partial = wired
        .builder
        .push_view(
            wired.source,
            ByteWindow {
                offset: 0,
                length: 8,
            },
        )
        .expect("partial view");
    assert_eq!(
        wired
            .builder
            .push_stage(
                &kernel,
                &occurrences(&semantic, 3..4),
                &[
                    read(partial, bytes),
                    write_access(wired.temporary_view, bytes)
                ],
                launch,
            )
            .expect_err("addressed extents are checked"),
        KernelProgramBuildError::StageElementCount {
            position: 0,
            expected: 6,
            actual: 2,
        }
    );
    assert!(matches!(
        wired.builder.push_view(
            wired.source,
            ByteWindow {
                offset: 16,
                length: 16,
            }
        ),
        Err(KernelProgramBuildError::ViewOutOfRange { .. })
    ));
}

#[test]
fn storage_reuse_is_admitted_only_with_an_explicit_handoff() {
    let semantic = two_chain_program();
    let program = publish_two_chain(two_chain(&semantic, true))
        .build()
        .expect("reuse with an explicit handoff verifies");
    assert_eq!(program.stages().len(), 4);
    assert_eq!(program.allocations().len(), 5);
    assert_eq!(program.outputs().len(), 2);

    // The shared allocation carries exactly the two internal temporaries.
    let shared = program
        .allocations()
        .find(|allocation| allocation.values().count() == 2)
        .expect("one shared allocation");
    assert!(
        shared
            .values()
            .all(|value| value.role() == ValueRole::Temporary)
    );

    // Without the handoff the reuse is unproven and the program fails closed.
    let rejected = diagnostic(publish_two_chain(two_chain(&semantic, false)));
    assert!(
        matches!(
            rejected,
            KernelProgramDiagnostic::ReuseMissingHandoff
                | KernelProgramDiagnostic::ReuseLifetimeOverlap
        ),
        "unexpected diagnostic: {rejected:?}"
    );
}

#[test]
fn a_dependency_cycle_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    // The opposite handoff is locally well formed and realized — the second
    // chain's reader precedes the first chain's writer — but together with the
    // forward handoff it closes a cycle.
    chains
        .builder
        .push_storage_handoff(chains.second_reduce, chains.first_map, chains.shared)
        .expect("the edge is locally well formed");
    assert_eq!(
        diagnostic(publish_two_chain(chains)),
        KernelProgramDiagnostic::DependencyCycle
    );
}

#[test]
fn a_missing_named_output_is_rejected() {
    let semantic = two_chain_program();
    let mut chains = two_chain(&semantic, true);
    chains
        .builder
        .push_output(OutputKey::new("sum_a").expect("key"), chains.first_output)
        .expect("first named output");
    // The second declared semantic output is never published.
    assert_eq!(
        diagnostic(chains.builder),
        KernelProgramDiagnostic::MissingNamedOutput
    );
}

#[test]
fn an_output_key_outside_the_bound_interface_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    assert!(matches!(
        wired
            .builder
            .push_output(OutputKey::new("other").expect("key"), wired.output),
        Err(KernelProgramBuildError::UnknownOutputKey { .. })
    ));
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("other"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::UnknownProgramInput { .. })
    ));
    // The one declared input is already claimed by another materialized value.
    assert!(matches!(
        wired.builder.push_value(
            value(program_input("input"), ValueRole::Input, input_shape()),
            wired.output_allocation,
        ),
        Err(KernelProgramBuildError::DuplicateProgramInput { .. })
    ));
    // A temporary claiming a program input is a role/origin contradiction.
    assert_eq!(
        wired
            .builder
            .push_value(
                value(program_input("input"), ValueRole::Temporary, input_shape()),
                wired.temporary_allocation,
            )
            .expect_err("role and origin must agree"),
        KernelProgramBuildError::ValueRoleOrigin {
            role: ValueRole::Temporary,
        }
    );
}

#[test]
fn an_internal_component_without_a_logical_group_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let mut wired = two_stage(&semantic, TwoStageShape::Canonical);
    let role = EncodedComponentRole::new(77);
    let error = wired
        .builder
        .push_component_value(
            MaterializedComponentSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Temporary,
                component_role: role,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
            },
            wired.temporary_allocation,
        )
        .expect_err("ungrouped internal components must fail closed");
    assert_eq!(
        error,
        KernelProgramBuildError::UngroupedInternalComponent { role }
    );
}

#[test]
fn physical_storage_scalar_and_kernel_access_type_are_checked_separately() {
    let semantic = strict_affine_u4_passthrough_program();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");
    let allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 20,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("allocation");
    let spec = |storage_scalar, element_type| MaterializedComponentSpec {
        origin: program_input("input"),
        role: ValueRole::Input,
        component_role: STRICT_AFFINE_CODES_ROLE,
        shape: Shape::from_dims([5]),
        storage_scalar,
        element_type,
        encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        alignment: AlignmentRequirement::natural_for(StorageScalar::U8),
        memory_space: MemorySpace::Device,
    };

    assert_eq!(
        builder
            .push_component_value(spec(StorageScalar::F32, KernelType::U8), allocation)
            .expect_err("a float scalar cannot carry packed codes"),
        KernelProgramBuildError::StorageEncodingScalar {
            scalar: StorageScalar::F32,
            encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        }
    );
    assert_eq!(
        builder
            .push_component_value(spec(StorageScalar::U8, KernelType::Bool), allocation)
            .expect_err("a boolean access must not stand in for an unsigned byte"),
        KernelProgramBuildError::StorageAccessType {
            scalar: StorageScalar::U8,
            encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
            expected: KernelType::U8,
            actual: KernelType::Bool,
        }
    );
}

#[test]
fn packed_program_views_are_bounded_to_the_complete_component() {
    let semantic = strict_affine_u4_passthrough_program();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");
    let allocation = builder
        .push_allocation(AllocationSpec {
            capacity_bytes: 3,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::U8),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .expect("allocation");
    let value = builder
        .push_component_value(
            MaterializedComponentSpec {
                origin: program_input("input"),
                role: ValueRole::Input,
                component_role: STRICT_AFFINE_CODES_ROLE,
                shape: Shape::from_dims([5]),
                storage_scalar: StorageScalar::U8,
                element_type: KernelType::U8,
                encoding: StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
                alignment: AlignmentRequirement::natural_for(StorageScalar::U8),
                memory_space: MemorySpace::Device,
            },
            allocation,
        )
        .expect("packed codes");
    assert_eq!(
        builder
            .push_view(
                value,
                ByteWindow {
                    offset: 0,
                    length: 2,
                },
            )
            .expect_err("a partial packed byte view has no logical ownership proof"),
        KernelProgramBuildError::PartialPackedView {
            offset: 0,
            length: 2,
            value_bytes: 3,
        }
    );
    builder
        .push_whole_view(value)
        .expect("the whole packed component is stage-visible");
}

#[test]
fn strict_affine_stage_bindings_are_addressed_by_component_role() {
    let semantic = strict_affine_u4_dequantize_program();
    let kernel = strict_affine_u4_dequantize_kernel();
    let mut builder = KernelProgramBuilder::new(&semantic).expect("program builder");

    let mut component = |role, shape, storage_scalar, element_type, encoding, bytes| {
        let allocation = builder
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(storage_scalar),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::External,
            })
            .expect("component allocation");
        let value = builder
            .push_component_value(
                MaterializedComponentSpec {
                    origin: program_input("input"),
                    role: ValueRole::Input,
                    component_role: role,
                    shape,
                    storage_scalar,
                    element_type,
                    encoding,
                    alignment: AlignmentRequirement::natural_for(storage_scalar),
                    memory_space: MemorySpace::Device,
                },
                allocation,
            )
            .expect("materialized component");
        builder.push_whole_view(value).expect("component view")
    };
    let codes = component(
        STRICT_AFFINE_CODES_ROLE,
        Shape::from_dims([5]),
        StorageScalar::U8,
        KernelType::U8,
        StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
        3,
    );
    let scale = component(
        STRICT_AFFINE_SCALE_ROLE,
        Shape::new([]),
        StorageScalar::F32,
        KernelType::F32,
        StorageEncoding::Unpacked,
        4,
    );
    let zero_point = component(
        STRICT_AFFINE_ZERO_POINT_ROLE,
        Shape::new([]),
        StorageScalar::U8,
        KernelType::U8,
        StorageEncoding::Unpacked,
        1,
    );
    let output_allocation = builder
        .push_allocation(device(20, AllocationOwnership::Program))
        .expect("output allocation");
    let output = builder
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([5]),
            ),
            output_allocation,
        )
        .expect("dense output");
    let output = builder.push_whole_view(output).expect("output view");

    let codes_bytes = literal(&mut builder, 3);
    let scale_bytes = literal(&mut builder, 4);
    let zero_point_bytes = literal(&mut builder, 1);
    let output_bytes = literal(&mut builder, 20);
    let grid_threads = literal(&mut builder, 5);
    let threads_per_workgroup = literal(&mut builder, 1);
    let error = builder
        .push_stage(
            &kernel,
            &occurrences(&semantic, 0..1),
            &[
                read(zero_point, zero_point_bytes),
                read(scale, scale_bytes),
                read(codes, codes_bytes),
                write_access(output, output_bytes),
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect_err("same-width input components must not bind by position");
    assert_eq!(
        error,
        KernelProgramBuildError::StageComponentRole {
            position: 0,
            expected: Some(STRICT_AFFINE_CODES_ROLE),
            actual: Some(STRICT_AFFINE_ZERO_POINT_ROLE),
        }
    );
}
