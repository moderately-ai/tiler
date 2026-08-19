//! The live input-extent kernel and the single-stage plan that binds it.

use super::super::super::{AbiBinaryOp, AbiRoot};
use super::graphs::{BIAS_BITS, SCALE_BITS, checked_coverage, input_shape, output_shape, strict};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{InputKey, OutputKey, SemanticProgram};
use tiler_ir::shape::{Axis, Shape};

/// A kernel whose body consumes one live input-axis extent.
///
/// Iteration is the static outer product; only axis 1 of the declared input is
/// live. The write is the program output so a single-stage artifact can bind it.
pub(crate) fn live_extent_kernel() -> VerifiedKernel {
    let rows = 2;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(Shape::from_dims([rows])).unwrap();
    let inner = Axis::new(1);
    region
        .push_access(Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajorSource { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [(0, TensorRole::Input), (1, TensorRole::Output)] {
        region
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .unwrap();
    }
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .unwrap();
    region
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::PointwiseF32(scale_bias_expression()),
            numerical: strict(),
        })
        .unwrap();
    region
        .schedule(KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: rows,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: rows,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    lower_scheduled_region(&region.build().unwrap()).unwrap()
}

pub(crate) fn scale_bias_expression() -> tiler_ir::schedule::PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression.input(AccessOrdinal::FIRST).unwrap();
    let scale = expression.constant(SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    expression.build(root).unwrap()
}

/// The single-stage live-operand plan over the fixture's fixed semantic graph.
///
/// Still constructible: the kernel-program layer binds it, and the packaging
/// interface work owns that layer's own association. What it can no longer do
/// is reach a verified artifact — `push_variant` refuses it by name, which is
/// the association fail-close the tests above assert.
pub(crate) fn live_extent_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = live_extent_kernel();
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(24, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(24, AllocationOwnership::Program))
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
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: 0,
            },
        )
        .unwrap();
    let zero = plan.push_abi_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let two = plan.push_abi_root(AbiRoot::UnsignedLiteral(2)).unwrap();
    let one = plan.push_abi_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let live_n = plan
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let accessible = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, zero, live_n)
        .unwrap();
    let guard = plan.push_abi_root(AbiRoot::BooleanLiteral(true)).unwrap();
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
    plan.push_stage(
        &kernel,
        &checked_coverage(semantic),
        &[
            StageAccess {
                view: read,
                mode: StageAccessMode::Read,
                accessible_bytes: accessible,
            },
            StageAccess {
                view: write,
                mode: StageAccessMode::Write,
                accessible_bytes: accessible,
            },
        ],
        StageLaunch {
            grid_threads: two,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}
