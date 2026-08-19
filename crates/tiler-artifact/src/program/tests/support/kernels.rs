//! Verified kernels and kernel programs the artifact fixtures package.

use super::super::super::AbiRoot;
use super::graphs::{
    BIAS_BITS, CANONICAL_NAN, ELEMENT_BYTES, SCALE_BITS, checked_coverage, coverage_range,
    input_shape, output_shape, strict,
};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    CoveredOccurrence, KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec,
    MemorySpace, RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode,
    StageLaunch, StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram, ViewId,
};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorOrder, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{InputKey, OutputKey, SemanticProgram};
use tiler_ir::shape::{Axis, Shape};

// -------------------------------------------------------------------------
// Shared-IR fixtures
// -------------------------------------------------------------------------

/// Declares the ABI, applicability guard, and routing-commit contract that both
/// single-stage kernel-program fixtures in this file share.
///
/// A verified kernel program states its own entry ABI since
/// `complete-program-identity-with-abi-guards-and-routing`, and folds it into
/// its canonical identity. The quantities are the fused kernel's: a whole
/// `[2, 3]` `f32` read, a whole `[2]` `f32` write, and a launch of two threads
/// at one thread per workgroup.
///
/// This is deliberately *not* the artifact-side ABI a `VariantSpec` declares.
/// That one lives on the artifact's own arena, under its own separately
/// versioned schema, and is asserted against the same program facts.
pub(crate) fn declare_program_contract(
    plan: &mut KernelProgramBuilder,
    read: ViewId,
    write: ViewId,
) -> ([StageAccess; 2], StageLaunch) {
    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("abi literal")
    };
    let read_bytes = literal(ELEMENT_BYTES * 6);
    let write_bytes = literal(ELEMENT_BYTES * 2);
    let grid_threads = literal(2);
    let threads_per_workgroup = literal(1);
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
    plan.applicability_guard(guard)
        .expect("applicability guard");
    for (from, to, fallback_permitted) in [
        (
            RoutingCommitState::Preflight,
            RoutingCommitState::Committed,
            true,
        ),
        (
            RoutingCommitState::Committed,
            RoutingCommitState::Executing,
            false,
        ),
        (
            RoutingCommitState::Executing,
            RoutingCommitState::Published,
            false,
        ),
    ] {
        plan.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })
        .expect("routing-commit transition");
    }
    (
        [
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: read_bytes,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: write_bytes,
            },
        ],
        StageLaunch {
            grid_threads,
            threads_per_workgroup,
        },
    )
}

/// Builds the one fused reduction kernel the packaged plans dispatch.
pub(crate) fn fused_kernel(scale_bits: u32) -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits,
                bias_bits: BIAS_BITS,
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0,
                contraction: false,
            },
            numerical: strict(),
        })
        .unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: 2,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: 2,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

/// Builds the single-stage kernel program the artifact packages.
pub(crate) fn fused_program(semantic: &SemanticProgram, scale_bits: u32) -> VerifiedKernelProgram {
    fused_program_with_coverage(semantic, scale_bits, &checked_coverage(semantic))
}

/// The same program over supplied coverage, so a test can vary only the proof.
pub(crate) fn fused_program_with_coverage(
    semantic: &SemanticProgram,
    scale_bits: u32,
    coverage: &[CoveredOccurrence],
) -> VerifiedKernelProgram {
    let kernel = fused_kernel(scale_bits);
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 8,
            alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::Program,
        })
        .unwrap();
    let source = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::ProgramInput {
                    key: InputKey::new("input").unwrap(),
                },
                role: ValueRole::Input,
                shape: input_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
            },
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            MaterializedValueSpec {
                origin: MaterializedOrigin::Internal,
                role: ValueRole::Output,
                shape: output_shape(),
                storage_scalar: StorageScalar::F32,
                element_type: KernelType::F32,
                encoding: StorageEncoding::Unpacked,
                alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    let (accesses, launch) = declare_program_contract(&mut plan, read, write);
    plan.push_stage(&kernel, coverage, &accesses, launch)
        .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

// -------------------------------------------------------------------------
// The two-stage intermediate-role fixture
// -------------------------------------------------------------------------
//
// Everything below exists because a *partial* binding window is not reachable
// through the single-stage fixtures above, and the reason is exact rather than
// an omission. `check_origin` pins a program input value's shape to the declared
// interface shape and `push_output` pins a published output value's, while
// `push_stage` requires each access to address exactly its buffer's element
// count — so an input or output value is always addressed whole. Only a
// `ValueRole::Temporary` value can be larger than what one stage addresses, and
// a stage binding one needs a kernel declaring a `TensorRole::Intermediate`
// buffer. A verified kernel refines the canonical lowering of a scheduled
// region, and of the three admitted region refinements the only two that name
// an intermediate role are the pointwise write and the reduction read, which
// live in different regions. So the smallest plan that can address part of a
// value is two stages, and these are those two stages.

/// The scratch shape a partial binding window addresses part of.
///
/// Twice the `[2, 3]` working set the stages exchange, so the plan can place
/// that working set in the upper half of one program-owned buffer. Nothing about
/// a temporary requires a stage to address the whole of it, and `push_view`
/// admits any window inside the value, so this is a plan the shared IR accepts
/// rather than one contrived to defeat a check.
pub(crate) fn scratch_shape() -> Shape {
    Shape::from_dims([4, 3])
}

/// First byte of the scratch buffer the two stages exchange their values through.
pub(crate) const SCRATCH_OFFSET: u64 = ELEMENT_BYTES * 6;

/// Builds the pointwise region's kernel: one program input to one temporary.
pub(crate) fn pointwise_kernel() -> VerifiedKernel {
    let elements = 6;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(input_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
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
        region
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
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(expression.build(root).unwrap()),
            numerical: strict(),
        })
        .unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: elements,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: elements,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

/// Builds the reduction region's kernel: one temporary to one program output.
pub(crate) fn reduction_kernel() -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(1));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input_shape(),
                output_shape: output_shape(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    region
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_NAN,
                empty_identity_bits: 0,
            },
            numerical: strict(),
        })
        .unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: 2,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::Serial {
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: 2,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

/// The program-level ABI quantities the two-stage fixture's stages name.
///
/// These are handles on the *kernel program*'s own arena, which the artifact's
/// same-named handle type is deliberately not interchangeable with; the artifact
/// declares its own expressions for the same quantities and each is proven
/// against the program separately.
pub(crate) struct TwoStageAbi {
    /// Byte count of the `[2, 3]` working set both stages address.
    pub(crate) working_bytes: tiler_ir::program::AbiExprId,
    /// Byte count of the whole `[2]` program output.
    pub(crate) output_bytes: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2, 3]` shape.
    pub(crate) pointwise_threads: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2]` shape.
    pub(crate) reduction_threads: tiler_ir::program::AbiExprId,
    /// Workgroup width both fixture kernels require.
    pub(crate) one: tiler_ir::program::AbiExprId,
}

/// Declares the ABI, applicability guard, and routing-commit contract of the
/// two-stage fixture.
pub(crate) fn declare_two_stage_contract(plan: &mut KernelProgramBuilder) -> TwoStageAbi {
    let mut literal = |value: u64| {
        plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
            .expect("abi literal")
    };
    let abi = TwoStageAbi {
        working_bytes: literal(ELEMENT_BYTES * 6),
        output_bytes: literal(ELEMENT_BYTES * 2),
        pointwise_threads: literal(6),
        reduction_threads: literal(2),
        one: literal(1),
    };
    let guard = plan
        .push_abi_root(AbiRoot::BooleanLiteral(true))
        .expect("guard predicate");
    plan.applicability_guard(guard).unwrap();
    for (from, to, fallback_permitted) in [
        (
            RoutingCommitState::Preflight,
            RoutingCommitState::Committed,
            true,
        ),
        (
            RoutingCommitState::Committed,
            RoutingCommitState::Executing,
            false,
        ),
        (
            RoutingCommitState::Executing,
            RoutingCommitState::Published,
            false,
        ),
    ] {
        plan.push_routing_commit_transition(RoutingCommitTransition {
            from,
            to,
            fallback_permitted,
        })
        .unwrap();
    }
    abi
}

/// The storage the two-stage fixture's stages exchange values through.
pub(crate) struct TwoStageStorage {
    /// The scratch value both stages address part of.
    pub(crate) temporary: tiler_ir::program::MaterializedValueId,
    /// The published program output.
    pub(crate) result: tiler_ir::program::MaterializedValueId,
    /// Whole view of the externally bound program input.
    pub(crate) read: ViewId,
    /// The partial view: the upper half of a scratch buffer sized for two.
    pub(crate) scratch_view: ViewId,
    /// Whole view of the published program output.
    pub(crate) write: ViewId,
}

/// Declares the input, the oversized scratch temporary, and the program output.
pub(crate) fn wire_two_stage_storage(plan: &mut KernelProgramBuilder) -> TwoStageStorage {
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(24, AllocationOwnership::External))
        .unwrap();
    // Twice the working set: the scratch value is what makes a partial window
    // expressible, so its allocation is sized for the value and not the window.
    let scratch = plan
        .push_allocation(device(48, AllocationOwnership::Program))
        .unwrap();
    let owned = plan
        .push_allocation(device(8, AllocationOwnership::Program))
        .unwrap();
    let value = |origin, role, shape| MaterializedValueSpec {
        origin,
        role,
        shape,
        storage_scalar: StorageScalar::F32,
        element_type: KernelType::F32,
        encoding: StorageEncoding::Unpacked,
        alignment: AlignmentRequirement::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
    };
    let source = plan
        .push_value(
            value(
                MaterializedOrigin::ProgramInput {
                    key: InputKey::new("input").unwrap(),
                },
                ValueRole::Input,
                input_shape(),
            ),
            external,
        )
        .unwrap();
    let temporary = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Temporary,
                scratch_shape(),
            ),
            scratch,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                output_shape(),
            ),
            owned,
        )
        .unwrap();
    TwoStageStorage {
        temporary,
        result,
        read: plan.push_whole_view(source).unwrap(),
        scratch_view: plan
            .push_view(
                temporary,
                ByteWindow {
                    offset: SCRATCH_OFFSET,
                    length: ELEMENT_BYTES * 6,
                },
            )
            .unwrap(),
        write: plan.push_whole_view(result).unwrap(),
    }
}

/// Builds the two-stage plan whose temporary is addressed at a nonzero offset.
///
/// The scratch buffer holds twice the working set and both stages address its
/// upper half through one shared view, so every binding of that value carries an
/// offset of [`SCRATCH_OFFSET`] rather than zero.
pub(crate) fn partial_window_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let pointwise = pointwise_kernel();
    let reduction = reduction_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let TwoStageAbi {
        working_bytes,
        output_bytes,
        pointwise_threads,
        reduction_threads,
        one,
    } = declare_two_stage_contract(&mut plan);
    let TwoStageStorage {
        temporary,
        result,
        read,
        scratch_view,
        write,
    } = wire_two_stage_storage(&mut plan);
    let coverage = checked_coverage(semantic);

    let first = plan
        .push_stage(
            &pointwise,
            &coverage_range(&coverage, 0..4),
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: working_bytes,
                },
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Write,
                    accessible_bytes: working_bytes,
                },
            ],
            StageLaunch {
                grid_threads: pointwise_threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    let second = plan
        .push_stage(
            &reduction,
            &coverage_range(&coverage, 4..5),
            &[
                StageAccess {
                    view: scratch_view,
                    mode: StageAccessMode::Read,
                    accessible_bytes: working_bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: output_bytes,
                },
            ],
            StageLaunch {
                grid_threads: reduction_threads,
                threads_per_workgroup: one,
            },
        )
        .unwrap();
    plan.push_data_dependency(first, second, temporary).unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}
