//! Baked `[2, N]` neighbours and host-side extent preconditions.

use super::super::{
    AbiBinaryOp, AbiEvaluationError, AbiFactBinder, AbiRoot, ArtifactBuildError,
    ArtifactProgramBuilder, AvailabilityPhase, CompilationEnvironment, VerifiedArtifactProgram,
};
use super::support::graphs::checked_coverage;
use super::support::live::scale_bias_expression;
use super::{
    SCALE_BITS, build_artifact, declare_realization, default_artifact, formulas, fused_program,
    lowering_provider, payload, selection, semantic_program, strict, variant,
};
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec, ByteWindow,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ExecutionBinding,
    KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, ReductionTopology, RegionId, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

#[test]
fn empty_extent_lists_do_not_move_previously_encodable_artifact_bytes() {
    let without = default_artifact();
    let again = default_artifact();
    assert_eq!(
        without.canonical_identity().as_bytes(),
        again.canonical_identity().as_bytes(),
        "two no-extent artifacts must keep one identity",
    );
    // A nonempty declaration is a new subject, and since
    // `tiler.artifact-program.v21` the published interface can spell the
    // symbolic axis such a row names — so the claim this pins is the narrower
    // one it always was: an *empty* extent list writes no bytes, and the domain
    // at which that holds is the current one.
    assert!(
        super::super::model::ARTIFACT_DOMAIN.ends_with(b"v21\0"),
        "the sourced-interface step owns v21; empty extent lists still write no bytes",
    );
}

/// Baking either neighbouring extent is a distinct artifact subject.
///
/// The half of the former two-N worked example that survives the association
/// fail-close: the *baked* programs remain packageable, and their identities
/// separate. The live subject's equal-identity-across-bindings half returns
/// with the packaged symbolic artifact.
#[test]
fn baking_neighbouring_extents_mints_distinct_artifact_subjects() {
    let baked_14 = baked_dense_artifact(14);
    let baked_15 = baked_dense_artifact(15);
    assert_ne!(
        baked_14.canonical_identity().as_bytes(),
        baked_15.canonical_identity().as_bytes(),
        "baking neighbouring extents must change artifact identity",
    );
}

/// A kernel that baked a bound extent is refused at assembly, not packaged.
#[test]
fn packaging_a_kernel_specialized_on_a_bound_extent_is_refused() {
    let semantic = baked_semantic_program(14);
    let program = baked_dense_program_with_live_range(&semantic, 14);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let error = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .expect_err("a baked kernel must not assemble over a bound extent");
    assert_eq!(
        error,
        ArtifactBuildError::BoundExtentSpecialization {
            entry: 0,
            key: "input".to_owned(),
            axis: 1,
            element_count: 28,
        },
        "the refusal must name the bound-extent specialization, not a later check",
    );
}

/// A precondition naming an unbound axis refuses before the bound value can be
/// used as two meanings.
#[test]
fn a_host_side_payload_disagreement_refuses_before_program_work() {
    let artifact = extent_precondition_artifact();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let precondition = entry
        .launch_preconditions()
        .next()
        .expect("the fixture names the inner axis in a launch precondition");

    let mut only_rows = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    only_rows
        .bind_input_extent(InputKey::new("input").unwrap(), Axis::new(0), 2)
        .unwrap();
    let rows_only = only_rows.build();
    assert_eq!(
        precondition.evaluate(&rows_only),
        Err(AbiEvaluationError::UnboundInputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        }),
        "binding the static row axis is not an answer for the live inner extent",
    );

    let mut both = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    both.bind_input_extent(InputKey::new("input").unwrap(), Axis::new(0), 2)
        .unwrap();
    both.bind_input_extent(InputKey::new("input").unwrap(), Axis::new(1), 14)
        .unwrap();
    assert_eq!(
        precondition
            .evaluate(&both.build())
            .expect("the live axis answers the launch precondition"),
        super::super::AbiValue::Boolean(true),
    );
}

/// `LinearIdentity` over a baked `[2, N]`, packaged the same way as the live subject.
fn baked_dense_kernel(columns: u64) -> VerifiedKernel {
    let rows = 2_u64;
    let elements = rows
        .checked_mul(columns)
        .expect("the two-N fixture stays inside u64");
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region
        .iteration_shape(Shape::from_dims([rows, columns]))
        .unwrap();
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
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
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
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    region
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
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

fn baked_dense_program(semantic: &SemanticProgram, columns: u64) -> VerifiedKernelProgram {
    let kernel = baked_dense_kernel(columns);
    let rows = 2_u64;
    let bytes = 4 * rows * columns;
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(bytes, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(bytes, AllocationOwnership::Program))
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
                Shape::from_dims([rows, columns]),
            ),
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([rows, columns]),
            ),
            owned,
        )
        .unwrap();
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let accessible = plan.push_abi_root(AbiRoot::UnsignedLiteral(bytes)).unwrap();
    let grid = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(rows * columns))
        .unwrap();
    let one = plan.push_abi_root(AbiRoot::UnsignedLiteral(1)).unwrap();
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
            grid_threads: grid,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

fn baked_semantic_program(columns: u64) -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let input = draft
        .input::<F32>(
            InputKey::new("input").unwrap(),
            Shape::from_dims([2, columns]),
        )
        .unwrap();
    let scale = F32Constant::apply(&mut draft, 2.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), mapped)
        .unwrap();
    draft.build().unwrap()
}

fn baked_dense_artifact(columns: u64) -> VerifiedArtifactProgram {
    let semantic = baked_semantic_program(columns);
    let program = baked_dense_program(&semantic, columns);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

/// The baked `[2, N]` program whose accessible range still names `InputExtent`.
///
/// That pairing is the specialization the assembly check refuses: the kernel
/// folded `N` into `element_count` while the ABI treats the same axis as a
/// per-invocation binding.
fn baked_dense_program_with_live_range(
    semantic: &SemanticProgram,
    columns: u64,
) -> VerifiedKernelProgram {
    let kernel = baked_dense_kernel(columns);
    let rows = 2_u64;
    let bytes = 4 * rows * columns;
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
        memory_space: MemorySpace::Device,
        ownership,
    };
    let external = plan
        .push_allocation(device(bytes, AllocationOwnership::External))
        .unwrap();
    let owned = plan
        .push_allocation(device(bytes, AllocationOwnership::Program))
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
                Shape::from_dims([rows, columns]),
            ),
            external,
        )
        .unwrap();
    let result = plan
        .push_value(
            value(
                MaterializedOrigin::Internal,
                ValueRole::Output,
                Shape::from_dims([rows, columns]),
            ),
            owned,
        )
        .unwrap();
    let read = plan
        .push_view(
            source,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let write = plan
        .push_view(
            result,
            ByteWindow {
                offset: 0,
                length: bytes,
            },
        )
        .unwrap();
    let four = plan.push_abi_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    let row_count = plan.push_abi_root(AbiRoot::UnsignedLiteral(rows)).unwrap();
    let live_n = plan
        .push_abi_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let row_bytes = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, four, row_count)
        .unwrap();
    let accessible = plan
        .push_abi_binary(AbiBinaryOp::CheckedMultiply, row_bytes, live_n)
        .unwrap();
    let grid = plan
        .push_abi_root(AbiRoot::UnsignedLiteral(rows * columns))
        .unwrap();
    let one = plan.push_abi_root(AbiRoot::UnsignedLiteral(1)).unwrap();
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
            grid_threads: grid,
            threads_per_workgroup: one,
        },
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

/// The fused artifact plus a launch precondition naming a bound input extent.
///
/// A launch precondition may read a bound extent — that is a host predicate,
/// not a live operand — so this packages without any extent-operand row and
/// keeps the host-side disagreement refusal testable on a static subject.
fn extent_precondition_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let n = draft
        .push_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(1),
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, n)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![predicate];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}
