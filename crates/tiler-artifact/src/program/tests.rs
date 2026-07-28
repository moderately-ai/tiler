//! Bounded tests for the target-neutral artifact program model.
//!
//! Fixtures package real verified kernel programs over real verified semantic
//! programs, so every rejection is a rejection of a plan that the shared IR
//! itself already accepted. Nothing here asserts that a kernel computes the
//! operations its stage covers; that remains compiler-owned evidence.

use std::sync::Arc;

use tiler_ir::kernel::{
    KernelType, MAX_KERNEL_IDENTITY_BYTES, VerifiedKernel, lower_scheduled_region,
};
use tiler_ir::program::{
    AllocationOwnership, AllocationSpec, ByteWindow, KernelProgramBuilder, MaterializedOrigin,
    MaterializedValueSpec, MemorySpace, RoutingCommitState, RoutingCommitTransition,
    SemanticOccurrence, StageAccess, StageAccessMode, StageLaunch, ValueRole,
    VerifiedKernelProgram, ViewId,
};
use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
    NumericalPermission, NumericalRealization, OwnershipProof, OwnershipProofKind,
    OwnershipWitnessId, PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram,
    ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply, InputKey,
    NormativeDefinitionRef, OpKey, OperationArity, OperationAttributeSchema, OperationConformance,
    OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
    OperationInferenceOutputs, OperationInferenceRequest, OperationInferencer, OperationSchema,
    OutputKey, ProviderDiagnosticCode, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE, RegistryError,
    SemanticProgram, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, TypeKey, ValueFact,
    ValueTypeDefinition, ValueTypeDefinitionKey, add_f32_op, constant_f32_op, multiply_f32_op,
    strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

use super::{
    AbiBinaryOp, AbiEvaluationError, AbiExprId, AbiFactBinder, AbiFacts, AbiRoot, AbiType,
    AbiUnaryOp, AbiValue, ArtifactBuildError, ArtifactDiagnostic, ArtifactEntityKind,
    ArtifactExecutionPolicy, ArtifactKeyKind, ArtifactProgramBuilder, AvailabilityPhase,
    BackendEntryKey, BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind,
    BindingSpec, CapabilityKey, CompilationEnvironment, DeferredPredicateSpec, EntrySpec,
    FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec, PayloadDigest, PayloadId,
    RepresentationKey, SchemaVersion, SelectedProvider, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, TargetPropertyKey, VariantSpec, VerifiedArtifactProgram,
};

// The seven items this suite shares with `crate::proof::tests` are `pub(crate)`
// rather than `pub(super)`; the rest of the fixture set stays module-local. The
// proof sidecar associates with a *real* verified artifact, and a second
// hand-built one would be a second thing to keep correct.
pub(crate) const SCALE_BITS: u32 = 0x4000_0000; // 2.0f32
pub(crate) const OTHER_SCALE_BITS: u32 = 0x4040_0000; // 3.0f32
pub(super) const BIAS_BITS: u32 = 0x3f80_0000; // 1.0f32
pub(super) const CANONICAL_NAN: u32 = 0x7fc0_0000;
pub(super) const ELEMENT_BYTES: u64 = 4;

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
fn declare_program_contract(
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

pub(super) fn strict() -> NumericalRealization {
    NumericalRealization::new(
        "tiler.test.strict-f32",
        CANONICAL_NAN,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

pub(super) fn input_shape() -> Shape {
    Shape::from_dims([2, 3])
}

pub(super) fn output_shape() -> Shape {
    Shape::from_dims([2])
}

pub(super) fn build_graph(draft: SemanticProgramBuilder) -> SemanticProgram {
    build_graph_scaled(draft, 2.0)
}

/// Builds the fixture graph, parameterized by the pointwise scale constant.
///
/// The scale is the cheapest way to obtain a genuinely different semantic graph
/// that keeps the same named interface: an unreached extra input would be
/// compacted away at commit (ADR 0064) and would not change graph identity.
pub(crate) fn build_graph_scaled(
    mut draft: SemanticProgramBuilder,
    scale_value: f32,
) -> SemanticProgram {
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, scale_value.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut draft, 1.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.build().unwrap()
}

pub(crate) fn semantic_program() -> SemanticProgram {
    build_graph(SemanticProgramBuilder::try_standard().unwrap())
}

/// Builds the fixture graph publishing its one reduction under two names.
///
/// `SemanticProgramBuilder::output_resolved` rejects a repeated *key* and not a
/// repeated *value*, so two named outputs may name one value all the way down to
/// one materialized program value and one buffer. That is the case a binding
/// target carrying a single output key would encode wrongly, so the fixture
/// exists to make it reachable rather than argued about.
fn dual_output_semantic_program() -> SemanticProgram {
    let mut draft = SemanticProgramBuilder::try_standard().unwrap();
    let input = draft
        .input::<F32>(InputKey::new("input").unwrap(), input_shape())
        .unwrap();
    let scale = F32Constant::apply(&mut draft, SCALE_BITS).unwrap();
    let bias = F32Constant::apply(&mut draft, BIAS_BITS).unwrap();
    let product = F32Multiply::apply(&mut draft, input, scale).unwrap();
    let mapped = F32Add::apply(&mut draft, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut draft, mapped, [Axis::new(1)]).unwrap();
    draft
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    draft.output(OutputKey::new("copy").unwrap(), sum).unwrap();
    draft.build().unwrap()
}

/// Builds the single-stage plan that publishes one value under both names.
fn dual_output_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
    let kernel = fused_kernel(SCALE_BITS);
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 8,
            alignment: 4,
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
                element_type: KernelType::F32,
                alignment: 4,
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
                element_type: KernelType::F32,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    let (accesses, launch) = declare_program_contract(&mut plan, read, write);
    plan.push_stage(
        &kernel,
        &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
        &accesses,
        launch,
    )
    .unwrap();
    plan.push_output(OutputKey::new("result").unwrap(), result)
        .unwrap();
    plan.push_output(OutputKey::new("copy").unwrap(), result)
        .unwrap();
    plan.build().unwrap()
}

/// Builds the one fused reduction kernel the packaged plans dispatch.
pub(super) fn fused_kernel(scale_bits: u32) -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input,
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
        .scalar_program(ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits: BIAS_BITS,
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
            contraction: false,
        })
        .unwrap();
    region.numerical(strict()).unwrap();
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
    let kernel = fused_kernel(scale_bits);
    let mut plan = KernelProgramBuilder::new(semantic).unwrap();
    let external = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 24,
            alignment: 4,
            memory_space: MemorySpace::Device,
            ownership: AllocationOwnership::External,
        })
        .unwrap();
    let owned = plan
        .push_allocation(AllocationSpec {
            capacity_bytes: 8,
            alignment: 4,
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
                element_type: KernelType::F32,
                alignment: 4,
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
                element_type: KernelType::F32,
                alignment: 4,
                memory_space: MemorySpace::Device,
            },
            owned,
        )
        .unwrap();
    let read = plan.push_whole_view(source).unwrap();
    let write = plan.push_whole_view(result).unwrap();
    let (accesses, launch) = declare_program_contract(&mut plan, read, write);
    plan.push_stage(
        &kernel,
        &(0..5).map(SemanticOccurrence::new).collect::<Vec<_>>(),
        &accesses,
        launch,
    )
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
fn scratch_shape() -> Shape {
    Shape::from_dims([4, 3])
}

/// First byte of the scratch buffer the two stages exchange their values through.
pub(super) const SCRATCH_OFFSET: u64 = ELEMENT_BYTES * 6;

/// Builds the pointwise region's kernel: one program input to one temporary.
fn pointwise_kernel() -> VerifiedKernel {
    let elements = 6;
    let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
    region.iteration_shape(input_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Input,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
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
    let input = expression.input().unwrap();
    let scale = expression.constant(SCALE_BITS).unwrap();
    let product = expression.multiply(input, scale).unwrap();
    let bias = expression.constant(BIAS_BITS).unwrap();
    let root = expression.add(product, bias).unwrap();
    region
        .scalar_program(ScalarProgram::PointwiseF32(expression.build(root).unwrap()))
        .unwrap();
    region.numerical(strict()).unwrap();
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
fn reduction_kernel() -> VerifiedKernel {
    let axes = vec![Axis::new(1)];
    let mut region = ScheduledRegionBuilder::new(RegionId::new(1));
    region.iteration_shape(output_shape()).unwrap();
    region
        .push_access(Access {
            tensor: TensorRole::Intermediate,
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
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_NAN,
            empty_identity_bits: 0,
        })
        .unwrap();
    region.numerical(strict()).unwrap();
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
struct TwoStageAbi {
    /// Byte count of the `[2, 3]` working set both stages address.
    working_bytes: tiler_ir::program::AbiExprId,
    /// Byte count of the whole `[2]` program output.
    output_bytes: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2, 3]` shape.
    pointwise_threads: tiler_ir::program::AbiExprId,
    /// Launch extent of the stage iterating the `[2]` shape.
    reduction_threads: tiler_ir::program::AbiExprId,
    /// Workgroup width both fixture kernels require.
    one: tiler_ir::program::AbiExprId,
}

/// Declares the ABI, applicability guard, and routing-commit contract of the
/// two-stage fixture.
fn declare_two_stage_contract(plan: &mut KernelProgramBuilder) -> TwoStageAbi {
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
struct TwoStageStorage {
    /// The scratch value both stages address part of.
    temporary: tiler_ir::program::MaterializedValueId,
    /// The published program output.
    result: tiler_ir::program::MaterializedValueId,
    /// Whole view of the externally bound program input.
    read: ViewId,
    /// The partial view: the upper half of a scratch buffer sized for two.
    scratch_view: ViewId,
    /// Whole view of the published program output.
    write: ViewId,
}

/// Declares the input, the oversized scratch temporary, and the program output.
fn wire_two_stage_storage(plan: &mut KernelProgramBuilder) -> TwoStageStorage {
    let device = |capacity_bytes, ownership| AllocationSpec {
        capacity_bytes,
        alignment: 4,
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
        element_type: KernelType::F32,
        alignment: 4,
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
pub(super) fn partial_window_program(semantic: &SemanticProgram) -> VerifiedKernelProgram {
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

    let first = plan
        .push_stage(
            &pointwise,
            &(0..4).map(SemanticOccurrence::new).collect::<Vec<_>>(),
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
            &[SemanticOccurrence::new(4)],
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

// -------------------------------------------------------------------------
// Artifact fixtures
// -------------------------------------------------------------------------

pub(crate) fn lowering_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "fused-serial-sum", revision).unwrap()
}

pub(super) fn spare_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "never-selected", revision).unwrap()
}

pub(super) fn selection(provider: ProviderIdentity) -> SelectedProvider {
    SelectedProvider {
        provider,
        capability: CapabilityKey::new("tiler.capability.fused-serial-sum").unwrap(),
        capability_revision: 1,
    }
}

pub(super) fn payload(tag: u8) -> BackendPayloadDescriptor {
    BackendPayloadDescriptor {
        backend: BackendKey::new("tiler.metal").unwrap(),
        representation: RepresentationKey::new("metallib").unwrap(),
        payload_schema: SchemaVersion::new(1, 0),
        digest: PayloadDigest::from_bytes([tag, 0xb2, 0xc3]).unwrap(),
        compatibility: profile(),
        execution_policy: ArtifactExecutionPolicy::RequiresDeviceTranslation,
    }
}

pub(super) fn profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.baseline").unwrap(),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02]).unwrap(),
    }
}

pub(super) fn rules() -> FeasibilityRuleSetRef {
    FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new("tiler.test.feasibility").unwrap(),
        revision: 1,
    }
}

/// The expression handles every fixture variant is assembled from.
pub(super) struct Formulas {
    /// The literal `1`, used by launch-precondition fixtures.
    pub(super) one: AbiExprId,
    /// The literal `true`, used by deferred-predicate fixtures.
    pub(super) always: AbiExprId,
}

pub(super) fn formulas(draft: &mut ArtifactProgramBuilder) -> Formulas {
    // Only what a caller still *supplies*. The applicability guard, launch
    // geometry, and accessible ranges are derived from the bound program now, so
    // minting the extent and byte-count formulas would leave them unreachable
    // from any use site -- the `UnusedExpression` the artifact refuses, and what
    // made two earlier attempts at this change look like an obligation conflict.
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    Formulas { one, always }
}

pub(super) fn entry(_formulas: &Formulas, payload: PayloadId, key: &[u8]) -> EntrySpec {
    EntrySpec {
        bindings: vec![
            BindingSpec {
                kind: BindingKind::Buffer,
            },
            BindingSpec {
                kind: BindingKind::Buffer,
            },
        ],
        launch: LaunchSpec {
            zero_work_skips_dispatch: true,
            preconditions: Vec::new(),
        },
        implementation: BackendEntryRef {
            payload,
            entry_key: BackendEntryKey::from_bytes(key).unwrap(),
        },
    }
}

pub(super) fn variant(formulas: &Formulas, payload: PayloadId, key: &[u8]) -> VariantSpec {
    VariantSpec {
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(formulas, payload, key)],
    }
}

/// Assembles the canonical one-variant artifact over one packaged program.
pub(crate) fn build_artifact(
    semantic: &SemanticProgram,
    program: &VerifiedKernelProgram,
    selected: ProviderIdentity,
    available: &[ProviderIdentity],
) -> VerifiedArtifactProgram {
    let environment = CompilationEnvironment::new(available.iter().cloned()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(semantic, environment).unwrap();
    draft.select_provider(selection(selected)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft.build().unwrap()
}

/// The two-stage variant whose scratch bindings start at a nonzero offset.
///
/// Nothing here states that offset. The guard, launch geometry, and accessible
/// ranges — the offset included — are derived from the bound program, so the
/// spec only pairs each stage with its backend entry; a producer has no field
/// through which it could restate the placement, honestly or otherwise.
fn partial_window_variant(payload: PayloadId) -> VariantSpec {
    let entry = |key: &[u8]| EntrySpec {
        bindings: vec![
            BindingSpec {
                kind: BindingKind::Buffer,
            },
            BindingSpec {
                kind: BindingKind::Buffer,
            },
        ],
        launch: LaunchSpec {
            zero_work_skips_dispatch: true,
            preconditions: Vec::new(),
        },
        implementation: BackendEntryRef {
            payload,
            entry_key: BackendEntryKey::from_bytes(key).unwrap(),
        },
    };
    VariantSpec {
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(b"pointwise"), entry(b"reduction")],
    }
}

/// Assembles the two-stage artifact whose temporary is bound at a nonzero offset.
pub(super) fn partial_window_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft
        .push_variant(&program, partial_window_variant(descriptor))
        .unwrap();
    draft.build().unwrap()
}

pub(crate) fn default_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

// -------------------------------------------------------------------------
// Verified-product construction and consumability
// -------------------------------------------------------------------------

#[test]
fn builds_a_verified_single_variant_artifact() {
    let artifact = default_artifact();
    assert_eq!(artifact.variants().len(), 1);
    assert_eq!(artifact.payloads().len(), 1);
    assert_eq!(artifact.selected_providers().len(), 1);
    assert_eq!(artifact.schema(), super::ArtifactSchema::GOVERNED);
    assert_eq!(
        artifact.routing_policy(),
        super::RoutingPolicy::StablePriority
    );
    let input = artifact.inputs().next().expect("one declared input");
    assert_eq!(input.key().as_str(), "input");
    assert_eq!(input.shape(), &input_shape());
    assert_eq!(input.element_type(), KernelType::F32);
    let output = artifact.outputs().next().expect("one declared output");
    assert_eq!(output.key().as_str(), "result");
    assert_eq!(output.shape(), &output_shape());
}

#[test]
fn an_entry_reads_its_plan_through_the_shared_ir_alone() {
    let artifact = default_artifact();
    let variant = artifact.variants().next().expect("one variant");
    assert_eq!(variant.routing_rank(), 0);
    assert_eq!(variant.target_profile(), &profile());
    assert_eq!(variant.deferred_predicates().len(), 0);
    let entry = variant.entries().next().expect("one entry");
    assert_eq!(
        entry.kernel_identity(),
        entry.stage().kernel().canonical_identity(),
    );
    assert_eq!(entry.resources().buffer_bindings, 2);
    assert_eq!(entry.numerical(), strict());
    assert!(entry.zero_work_skips_dispatch());
    assert_eq!(entry.backend_entry_key().as_bytes(), b"fused");
    assert_eq!(entry.payload().representation.as_str(), "metallib");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].slot(), 0);
    assert_eq!(bindings[0].kind(), BindingKind::Buffer);
    assert_eq!(bindings[0].element_type(), KernelType::F32);
    assert_eq!(bindings[0].value_role(), ValueRole::Input);
    assert_eq!(bindings[0].alignment(), 4);
    assert_eq!(bindings[0].window().length, 24);
    assert_eq!(bindings[1].value_role(), ValueRole::Output);
    assert_eq!(bindings[1].window().length, 8);
    // The same correspondence the shared-IR walk above reads, spelled as the
    // interface reference the artifact carries — this is the one a consumer
    // holding only bytes can follow.
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        bindings[0].target(),
        super::BindingTarget::ProgramInput(&InputKey::new("input").unwrap()),
    );
    assert_eq!(
        bindings[1].target(),
        super::BindingTarget::ProgramOutput(std::slice::from_ref(&result)),
    );
    // The plan itself is reachable through the shared IR's own views.
    assert_eq!(variant.program().stages().len(), 1);
    assert_eq!(bindings[0].value().required_bytes(), 24);
}

/// One buffer published under two names carries both, rather than one of them.
///
/// The failure this excludes is not a missing accessor. A target carrying a
/// single output key would name whichever the producer's declaration order put
/// first, and a loader would bind a second buffer for the other name — two
/// buffers for one value, with the unbound one never written. Carrying the
/// complete set is what makes "one buffer, two names" expressible at all.
#[test]
fn a_value_published_under_two_names_carries_both_in_its_binding_target() {
    let semantic = dual_output_semantic_program();
    let program = dual_output_program(&semantic);
    let provider = lowering_provider(1);
    let artifact = build_artifact(&semantic, &program, provider.clone(), &[provider]);
    let bindings: Vec<_> = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry")
        .bindings()
        .collect();
    let super::BindingTarget::ProgramOutput(keys) = bindings[1].target() else {
        panic!("the written binding addresses published output storage");
    };
    // Canonically ordered rather than in declaration order, so the artifact's
    // identity does not fold the order a producer happened to publish in.
    assert_eq!(
        keys.iter().map(OutputKey::as_str).collect::<Vec<_>>(),
        ["copy", "result"],
    );
    assert_eq!(bindings[1].value_role(), ValueRole::Output);
}

/// A slot may address part of the value it names, and it says where.
///
/// The plan sizes one program-owned scratch buffer for two working sets and puts
/// the one its two stages exchange in the upper half. Both stages therefore bind
/// the *same* internal value at byte 24 of 48. What this excludes is the failure
/// the refusal it replaces existed to prevent: a record carrying an extent and no
/// placement leaves a loader binding the right buffer at byte zero, which is a
/// silently wrong dispatch rather than a rejection.
#[test]
fn a_binding_may_address_part_of_the_value_it_names() {
    let artifact = partial_window_artifact();
    let facts = bound_facts();
    let entries: Vec<_> = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .collect();
    assert_eq!(entries.len(), 2);
    let pointwise: Vec<_> = entries[0].bindings().collect();
    let reduction: Vec<_> = entries[1].bindings().collect();

    // The scratch slot of each entry: written by the first stage, read by the
    // second, and the same materialized value in both.
    for scratch in [pointwise[1], reduction[0]] {
        assert_eq!(scratch.target(), super::BindingTarget::Internal);
        assert_eq!(scratch.value_role(), ValueRole::Temporary);
        // Partial in the exact sense that matters: the window is shorter than
        // the value, and starts inside it.
        assert_eq!(scratch.value().required_bytes(), ELEMENT_BYTES * 12);
        assert_eq!(scratch.window().offset, SCRATCH_OFFSET);
        assert_eq!(scratch.window().length, ELEMENT_BYTES * 6);
        assert_eq!(
            scratch.accessible_offset().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(SCRATCH_OFFSET),
        );
        assert_eq!(
            scratch.accessible_bytes().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(ELEMENT_BYTES * 6),
        );
    }

    // The interface slots address their values whole, and say that too.
    for whole in [pointwise[0], reduction[1]] {
        assert_eq!(whole.window().offset, 0);
        assert_eq!(
            whole.accessible_offset().evaluate(&facts).unwrap(),
            AbiValue::Unsigned(0),
        );
    }
}

/// Binds the fixture interface's declared shapes as an evaluation environment.
fn bound_facts() -> AbiFacts {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(&InputKey::new("input").unwrap(), &input_shape())
        .unwrap();
    binder.build()
}

#[test]
fn abi_expressions_evaluate_against_bound_runtime_facts() {
    let artifact = default_artifact();
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(&InputKey::new("input").unwrap(), &input_shape())
        .unwrap();
    let facts = binder.build();
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(
        bindings[0].accessible_bytes().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(24),
    );
    assert_eq!(
        bindings[1].accessible_bytes().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(8),
    );
    assert_eq!(
        entry.launch_threads().evaluate(&facts).unwrap(),
        AbiValue::Unsigned(2),
    );
    assert_eq!(entry.launch_threads().value_type(), AbiType::Unsigned);
}

// -------------------------------------------------------------------------
// Identity determinism and order independence
// -------------------------------------------------------------------------

#[test]
fn identity_is_deterministic_for_equal_artifacts() {
    let first = default_artifact();
    let second = default_artifact();
    assert_eq!(first.canonical_identity(), second.canonical_identity());
    assert_eq!(first, second);
}

#[test]
fn identity_ignores_payload_and_provider_declaration_order() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let providers = [lowering_provider(1), lowering_provider(2)];
    let environment = CompilationEnvironment::new(providers.iter().cloned()).unwrap();

    let alternate = fused_program(&semantic, OTHER_SCALE_BITS);

    let assemble = |forward: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        let (first, second) = if forward { (0, 1) } else { (1, 0) };
        draft
            .select_provider(selection(providers[first].clone()))
            .unwrap();
        draft
            .select_provider(selection(providers[second].clone()))
            .unwrap();
        let (primary, spare) = if forward {
            let primary = draft.push_payload(payload(0x01)).unwrap();
            (primary, draft.push_payload(payload(0x02)).unwrap())
        } else {
            let spare = draft.push_payload(payload(0x02)).unwrap();
            (draft.push_payload(payload(0x01)).unwrap(), spare)
        };
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, primary, b"fused"))
            .unwrap();
        draft
            .push_variant(&alternate, variant(&formulas, spare, b"alternate"))
            .unwrap();
        draft.build().unwrap()
    };

    assert_eq!(
        assemble(true).canonical_identity(),
        assemble(false).canonical_identity(),
    );
}

#[test]
fn identity_ignores_expression_assembly_order() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();

    let assemble = |reversed: bool| {
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
        draft.select_provider(selection(provider.clone())).unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        // Assemble the identical formulas through two different node orders.
        let formulas = if reversed {
            // The same two expressions in the opposite declaration order; the
            // variant's ABI is the program's now, so what remains under test is
            // that a caller-supplied expression's declaration order does not
            // reach identity.
            let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
            let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
            Formulas { one, always }
        } else {
            formulas(&mut draft)
        };
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        draft.build().unwrap()
    };

    assert_eq!(
        assemble(false).canonical_identity(),
        assemble(true).canonical_identity(),
    );
}

#[test]
fn the_expression_arena_is_canonically_deduplicated() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let first = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    let second = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    assert_eq!(first, second);
    let sum = draft
        .push_binary(AbiBinaryOp::CheckedAdd, first, second)
        .unwrap();
    let again = draft
        .push_binary(AbiBinaryOp::CheckedAdd, second, first)
        .unwrap();
    assert_eq!(sum, again);
}

// -------------------------------------------------------------------------
// Reached versus unused provenance (ADR 0072)
// -------------------------------------------------------------------------

#[test]
fn a_reached_capability_provider_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let available = [lowering_provider(1), lowering_provider(2)];
    let first = build_artifact(&semantic, &program, lowering_provider(1), &available);
    let second = build_artifact(&semantic, &program, lowering_provider(2), &available);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
}

/// The capability's own revision reaches identity, independently of the provider's.
///
/// `docs/operation-extensions.md` makes the two revisions independent — one
/// provider registers several capabilities that move at different rates — so
/// folding only the provider's left a provider free to change what its lowering
/// emits and produce a byte-identical artifact identity, which is exactly the
/// drift the capability revision exists to catch. Both directions are asserted:
/// the revision moving changes identity, and everything else held equal it is
/// the only thing that did.
#[test]
fn a_reached_capability_revision_changes_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let build = |capability_revision: u32| {
        let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft
            .select_provider(SelectedProvider {
                provider: provider.clone(),
                capability: CapabilityKey::new("tiler.capability.fused-serial-sum").unwrap(),
                capability_revision,
            })
            .unwrap();
        let descriptor = draft.push_payload(payload(0xa1)).unwrap();
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"fused"))
            .unwrap();
        draft.build().unwrap()
    };

    let first = build(1);
    let second = build(2);
    assert_ne!(first.canonical_identity(), second.canonical_identity());
    assert_eq!(
        first.canonical_identity(),
        build(1).canonical_identity(),
        "nothing else in the fixture varies with the revision",
    );
    assert_eq!(
        first.selected_providers()[0].provider,
        second.selected_providers()[0].provider,
        "the provider's own revision is unchanged; only the capability's moved",
    );
}

#[test]
fn an_unused_environment_provider_does_not_change_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let selected = lowering_provider(1);
    let lean = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        std::slice::from_ref(&selected),
    );
    let crowded = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected.clone(), spare_provider(1)],
    );
    let bumped = build_artifact(
        &semantic,
        &program,
        selected.clone(),
        &[selected, spare_provider(7)],
    );
    assert_eq!(lean.canonical_identity(), crowded.canonical_identity());
    assert_eq!(crowded.canonical_identity(), bumped.canonical_identity());
    // The environments genuinely differed; only the reached half was packaged.
    assert_eq!(lean.selected_providers().len(), 1);
    assert_eq!(crowded.selected_providers().len(), 1);
}

#[test]
fn a_reached_semantic_provider_revision_changes_identity() {
    let first = governed_program(1);
    let second = governed_program(2);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph(),
    );
    assert_eq!(
        first.semantic_identity().reached_definitions(),
        second.semantic_identity().reached_definitions(),
    );
    assert_ne!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_artifact = build_artifact(
        &first,
        &fused_program(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_ne!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

#[test]
fn an_unused_semantic_provider_revision_does_not_change_identity() {
    let first = program_with_unused_provider(1);
    let second = program_with_unused_provider(2);
    // The fixture is meaningful only if the two programs really differ.
    assert_ne!(
        first.semantic_identity().registry_snapshot(),
        second.semantic_identity().registry_snapshot(),
    );
    assert_eq!(
        first.semantic_identity().admission_provenance(),
        second.semantic_identity().admission_provenance(),
    );
    let provider = lowering_provider(1);
    let first_artifact = build_artifact(
        &first,
        &fused_program(&first, SCALE_BITS),
        provider.clone(),
        std::slice::from_ref(&provider),
    );
    let second_artifact = build_artifact(
        &second,
        &fused_program(&second, SCALE_BITS),
        provider.clone(),
        &[provider],
    );
    assert_eq!(
        first_artifact.canonical_identity(),
        second_artifact.canonical_identity(),
    );
}

// -------------------------------------------------------------------------
// Cross-program and forged-input rejection
// -------------------------------------------------------------------------

#[test]
fn rejects_a_variant_realizing_another_semantic_graph() {
    let packaged = semantic_program();
    let other = build_graph_scaled(SemanticProgramBuilder::try_standard().unwrap(), 3.0);
    assert_ne!(
        packaged.semantic_identity().graph(),
        other.semantic_identity().graph(),
    );
    let foreign = fused_program(&other, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&packaged, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&foreign, variant(&formulas, descriptor, b"fused")),
        Err(ArtifactBuildError::SemanticSubjectMismatch),
    );
}

#[test]
fn rejects_an_expression_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_formulas = formulas(&mut donor);
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    // Injected through a launch precondition, which is still caller-supplied.
    // The guard and launch geometry are derived from the program now, so they
    // are no longer a way to hand the builder a foreign handle at all.
    spec.entries[0]
        .launch
        .preconditions
        .push(donor_formulas.always);
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Expression,
        }),
    );
}

#[test]
fn rejects_a_payload_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_payload = donor.push_payload(payload(0xa1)).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, donor_payload, b"fused")),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Payload,
        }),
    );
}

// -------------------------------------------------------------------------
// Negative tests, one per insertion-time rule
// -------------------------------------------------------------------------

#[test]
fn rejects_a_provider_the_environment_never_offered() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    assert_eq!(
        draft.select_provider(selection(lowering_provider(9))),
        Err(ArtifactBuildError::ProviderNotAvailable {
            provider: Box::new(lowering_provider(9)),
        }),
    );
}

#[test]
fn rejects_a_deferred_predicate_that_is_already_decided() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::CompileProfile,
            authority: lowering_provider(1),
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::NonDeferredPredicatePhase {
            phase: AvailabilityPhase::CompileProfile,
        }),
    );
}

#[test]
fn rejects_a_deferred_authority_that_was_never_selected() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::LaunchPreflight,
            authority: spare_provider(1),
        }];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::UnselectedDeferredAuthority {
            provider: Box::new(spare_provider(1)),
        }),
    );
}

#[test]
fn accepts_a_deferred_predicate_bound_to_a_selected_authority() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider.clone())).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let property = draft
        .push_root(AbiRoot::TargetProperty {
            key: TargetPropertyKey::new("tiler.target.max-threads-per-workgroup").unwrap(),
            phase: AvailabilityPhase::LaunchPreflight,
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, property)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.deferred_predicates = vec![DeferredPredicateSpec {
        predicate,
        phase: AvailabilityPhase::LaunchPreflight,
        authority: provider.clone(),
    }];
    draft.push_variant(&program, spec).unwrap();
    let artifact = draft.build().unwrap();
    let deferred = artifact
        .variants()
        .next()
        .expect("one variant")
        .deferred_predicates()
        .next()
        .expect("one deferred predicate");
    assert_eq!(deferred.phase(), AvailabilityPhase::LaunchPreflight);
    assert_eq!(deferred.authority(), &provider);
}

#[test]
fn rejects_a_repeated_deferred_predicate() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let predicate = DeferredPredicateSpec {
            predicate: formulas.always,
            phase: AvailabilityPhase::LaunchPreflight,
            authority: lowering_provider(1),
        };
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.deferred_predicates = vec![predicate.clone(), predicate];
        draft.push_variant(program, spec)
    });
    assert_eq!(outcome, Err(ArtifactBuildError::DuplicateDeferredPredicate));
}

#[test]
fn rejects_a_repeated_launch_precondition() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].launch.preconditions = vec![formulas.always, formulas.always];
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::DuplicateLaunchPrecondition { entry: 0 }),
    );
}

#[test]
fn rejects_an_entry_count_that_disagrees_with_the_program() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries.push(entry(formulas, descriptor, b"extra"));
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::EntryCardinality {
            expected: 1,
            actual: 2,
        }),
    );
}

#[test]
fn rejects_a_binding_count_that_disagrees_with_the_kernel_signature() {
    let outcome = with_default_draft(|draft, formulas, descriptor, program| {
        let mut spec = variant(formulas, descriptor, b"fused");
        spec.entries[0].bindings.pop();
        draft.push_variant(program, spec)
    });
    assert_eq!(
        outcome,
        Err(ArtifactBuildError::BindingCardinality {
            entry: 0,
            expected: 2,
            actual: 1,
        }),
    );
}

#[test]
fn rejects_a_duplicate_plan_variant() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, descriptor, b"other")),
        Err(ArtifactBuildError::DuplicateVariant),
    );
}

#[test]
fn rejects_a_repeated_payload_descriptor() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    assert_eq!(
        draft.push_payload(payload(0xa1)),
        Err(ArtifactBuildError::DuplicatePayload),
    );
}

#[test]
fn rejects_a_mistyped_expression_operand() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let number = draft.push_root(AbiRoot::UnsignedLiteral(4)).unwrap();
    assert_eq!(
        draft.push_unary(AbiUnaryOp::Not, number),
        Err(ArtifactBuildError::OperandType {
            expected: AbiType::Boolean,
            actual: AbiType::Unsigned,
        }),
    );
    let predicate = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    assert_eq!(
        draft.push_select(predicate, number, predicate),
        Err(ArtifactBuildError::SelectBranchType {
            if_true: AbiType::Unsigned,
            if_false: AbiType::Boolean,
        }),
    );
}

// -------------------------------------------------------------------------
// Negative tests, one per whole-artifact rule
// -------------------------------------------------------------------------

#[test]
fn rejects_an_empty_portfolio() {
    let semantic = semantic_program();
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let diagnostics = draft.build().expect_err("an empty portfolio is rejected");
    assert!(
        diagnostics
            .diagnostics()
            .contains(&ArtifactDiagnostic::EmptyPortfolio)
    );
}

#[test]
fn rejects_an_artifact_that_selected_no_provider() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    let error = draft.build().expect_err("unattributed plans are rejected");
    assert_eq!(
        error.diagnostics(),
        [ArtifactDiagnostic::MissingSelectedProvider],
    );
    // The builder comes back intact and the failure is recoverable.
    let (mut recovered, _) = error.into_parts();
    recovered
        .select_provider(selection(lowering_provider(1)))
        .unwrap();
    assert_eq!(recovered.build().unwrap().selected_providers().len(), 1);
}

#[test]
fn rejects_an_expression_no_use_site_reaches() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft.push_root(AbiRoot::UnsignedLiteral(999)).unwrap();
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("an unreachable node is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedExpression],
    );
}

#[test]
fn rejects_a_payload_no_entry_realizes() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft.push_payload(payload(0xb1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("an unreferenced payload is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedPayload],
    );
}

#[test]
fn rejects_two_entries_claiming_one_backend_entry() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&first, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft
        .push_variant(&second, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    assert_eq!(
        draft
            .build()
            .expect_err("a non-injective backend mapping is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::DuplicateBackendEntry],
    );
}

// -------------------------------------------------------------------------
// Expression evaluation, phases, and failure classification
// -------------------------------------------------------------------------

#[test]
fn a_conditional_selection_evaluates_only_the_branch_it_takes() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let zero = draft.push_root(AbiRoot::UnsignedLiteral(0)).unwrap();
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let ten = draft.push_root(AbiRoot::UnsignedLiteral(10)).unwrap();
    let unsafe_division = draft
        .push_binary(AbiBinaryOp::FloorDivide, ten, zero)
        .unwrap();
    let nonzero = draft
        .push_binary(AbiBinaryOp::LessOrEqual, one, zero)
        .unwrap();
    let guarded = draft.push_select(nonzero, unsafe_division, ten).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, guarded, &facts),
        Ok(AbiValue::Unsigned(10)),
    );
    assert_eq!(
        evaluate_through_draft(&draft, unsafe_division, &facts),
        Err(AbiEvaluationError::DivisionByZero {
            op: AbiBinaryOp::FloorDivide,
        }),
    );
}

#[test]
fn the_fact_binder_refuses_a_fact_from_a_later_phase() {
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    let error = binder
        .bind_target_property(
            TargetPropertyKey::new("tiler.target.pipeline-registers").unwrap(),
            AvailabilityPhase::PreparedKernelPreflight,
            64,
        )
        .expect_err("a prepared-kernel fact is not observable at live preflight");
    assert_eq!(
        error,
        super::AbiBindingError::PhaseNotReached {
            available_at: AvailabilityPhase::PreparedKernelPreflight,
            reached: AvailabilityPhase::LiveDevicePreflight,
        },
    );
    assert_eq!(
        binder.build().reached_phase(),
        AvailabilityPhase::LiveDevicePreflight,
    );
}

#[test]
fn evaluation_reports_an_unbound_root_rather_than_guessing() {
    // Exercised through a launch precondition rather than the launch geometry.
    // The geometry is derived from the program now and that program's is a
    // constant, so it evaluates without consulting any fact -- which would make
    // this test pass for the wrong reason. A precondition is still
    // caller-supplied and can name a fact that is deliberately left unbound.
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let rows = draft
        .push_root(AbiRoot::InputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        })
        .unwrap();
    let predicate = draft
        .push_binary(AbiBinaryOp::LessOrEqual, formulas.one, rows)
        .unwrap();
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![predicate];
    draft.push_variant(&program, spec).unwrap();
    let artifact = draft.build().unwrap();

    let facts = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build();
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
        .expect("one launch precondition");
    assert_eq!(
        precondition.evaluate(&facts),
        Err(AbiEvaluationError::UnboundInputExtent {
            key: InputKey::new("input").unwrap(),
            axis: Axis::new(0),
        }),
    );
}

#[test]
fn checked_narrowing_rejects_a_value_that_does_not_fit() {
    let semantic = semantic_program();
    let environment = CompilationEnvironment::new([lowering_provider(1)]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let wide = draft
        .push_root(AbiRoot::UnsignedLiteral(u64::from(u32::MAX) + 1))
        .unwrap();
    let narrowed = draft.push_unary(AbiUnaryOp::NarrowU32, wide).unwrap();
    let facts = AbiFactBinder::new(AvailabilityPhase::CompileProfile).build();
    assert_eq!(
        evaluate_through_draft(&draft, narrowed, &facts),
        Err(AbiEvaluationError::NarrowingOverflow {
            op: AbiUnaryOp::NarrowU32,
            value: u64::from(u32::MAX) + 1,
        }),
    );
}

// -------------------------------------------------------------------------
// Received opaque identities are bounded by whoever mints them
// -------------------------------------------------------------------------

/// Each opaque identity is bounded by the authority that derives its subject.
///
/// The 1,121-byte case is the measured one, not a chosen one: it is the
/// canonical kernel identity of a serial `f32` sum reducing two or more
/// contributors, which the shared bound refused while admitting only the
/// degenerate one-contributor reduction. The two digest-shaped identities keep
/// the smaller bound, which is the whole point of separating them — raising one
/// bound for all three would have discarded a real one.
#[test]
fn an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it() {
    let measured_kernel_identity = vec![0x5a; 1_121];
    BackendEntryKey::from_bytes(&measured_kernel_identity)
        .expect("a real reduction's kernel identity is a legal backend entry key");
    assert!(
        measured_kernel_identity.len() > super::MAX_OPAQUE_IDENTITY_BYTES,
        "the case is only a regression test while it exceeds the shared bound",
    );

    assert_eq!(
        BackendEntryKey::from_bytes(vec![0x5a; MAX_KERNEL_IDENTITY_BYTES + 1]),
        Err(ArtifactBuildError::KeyTooLong {
            kind: ArtifactKeyKind::BackendEntry,
            bytes: MAX_KERNEL_IDENTITY_BYTES + 1,
            limit: MAX_KERNEL_IDENTITY_BYTES,
        }),
        "beyond what the shared IR can mint, the refusal is still loud",
    );

    for (bytes, expected) in [
        (
            PayloadDigest::from_bytes(vec![0x5a; super::MAX_OPAQUE_IDENTITY_BYTES + 1]).err(),
            ArtifactKeyKind::PayloadDigest,
        ),
        (
            TargetProfileDescriptorDigest::from_bytes(vec![
                0x5a;
                super::MAX_OPAQUE_IDENTITY_BYTES + 1
            ])
            .err(),
            ArtifactKeyKind::TargetProfileDescriptor,
        ),
    ] {
        assert_eq!(
            bytes,
            Some(ArtifactBuildError::KeyTooLong {
                kind: expected,
                bytes: super::MAX_OPAQUE_IDENTITY_BYTES + 1,
                limit: super::MAX_OPAQUE_IDENTITY_BYTES,
            }),
        );
    }
}

/// The bound admits every entry key the packaged program itself carries.
///
/// An artifact carries one entry's kernel identity twice — as the entry key,
/// and inside the stage subject `stage_key` derives — so the two bounds have to
/// admit the same values or an artifact could be built and not encoded. This
/// asserts the first half against the second at a length the old bound refused.
#[test]
fn an_artifact_encodes_an_entry_key_longer_than_the_digest_bound() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let long_key = vec![0x5a; 1_121];
    draft
        .push_variant(&program, variant(&formulas, descriptor, &long_key))
        .unwrap();
    let artifact = draft.build().unwrap();

    let bytes = artifact.encode().expect("the envelope encodes");
    let decoded = super::decode_artifact(&bytes).expect("the envelope decodes");
    assert_eq!(
        decoded
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .backend_entry_key()
            .as_bytes(),
        long_key.as_slice(),
    );
}

// -------------------------------------------------------------------------
// Test-local helpers
// -------------------------------------------------------------------------

/// Runs one rejection case against the canonical draft state.
fn with_default_draft<T>(
    case: impl FnOnce(
        &mut ArtifactProgramBuilder,
        &Formulas,
        PayloadId,
        &VerifiedKernelProgram,
    ) -> Result<T, ArtifactBuildError>,
) -> Result<T, ArtifactBuildError> {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    case(&mut draft, &formulas, descriptor, &program)
}

/// Evaluates one draft expression by packaging the arena the builder holds.
///
/// Evaluation is a property of the verified product, so this helper builds a
/// throwaway artifact whose only use site is the expression under test.
fn evaluate_through_draft(
    draft: &ArtifactProgramBuilder,
    node: AbiExprId,
    facts: &super::AbiFacts,
) -> Result<AbiValue, AbiEvaluationError> {
    draft.evaluate_draft_expression(node, facts)
}

// -------------------------------------------------------------------------
// Semantic-provider fixtures
// -------------------------------------------------------------------------

fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
    ProviderDiagnosticCode::new(value).unwrap()
}

#[derive(Clone, Copy)]
enum TestOperation {
    Constant,
    Binary,
    Sum,
}

impl OperationInferencer for TestOperation {
    fn infer(
        &self,
        request: OperationInferenceRequest<'_>,
        outputs: &mut OperationInferenceOutputs<'_>,
    ) -> Result<(), OperationInferenceError> {
        let operands = request.operands();
        let attributes = request.attributes();
        match self {
            Self::Constant => {
                outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
            }
            Self::Binary => {
                let left = operands[0].shape();
                let right = operands[1].shape();
                let shape = if left.rank() == 0 {
                    right.clone()
                } else if right.rank() == 0 || left == right {
                    left.clone()
                } else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.binary.shape"),
                        "operands must have equal shapes or include one scalar",
                    )
                    .unwrap());
                };
                outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
            }
            Self::Sum => {
                let Some(CanonicalValueView::Sequence(values)) = attributes
                    .get(REDUCTION_AXES_ATTRIBUTE)
                    .map(CanonicalValue::view)
                else {
                    return Err(OperationInferenceError::new(
                        diagnostic_code("test.sum.axes"),
                        "sum axes must be a sequence",
                    )
                    .unwrap());
                };
                let axes = values
                    .iter()
                    .map(|value| match value.view() {
                        CanonicalValueView::Unsigned {
                            width: CanonicalIntegerWidth::Bits32,
                            bits,
                        } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                            OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-width"),
                                "sum axis exceeds u32",
                            )
                            .unwrap()
                        }),
                        _ => Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axis-kind"),
                            "sum axes must be u32 values",
                        )
                        .unwrap()),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                outputs.try_push(ValueFact::new(
                    F32::resolved_type(),
                    operands[0].shape().without_axes(&axes),
                ))
            }
        }
    }
}

/// A provider the packaged graph actually reaches, with a settable revision.
struct GovernedTestSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for GovernedTestSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_marked_value_type::<F32>(
            ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                NormativeDefinitionRef::new("test binary32 semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ),
            F32::resolved_type(),
        )?;
        register_test_operation(
            registrar,
            constant_f32_op(),
            0,
            [OperationAttributeSchema::required(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValueKind::FloatBits,
            )],
            TestOperation::Constant,
        )?;
        register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
        register_test_operation(
            registrar,
            strict_serial_sum_f32_op(),
            1,
            [OperationAttributeSchema::required(
                REDUCTION_AXES_ATTRIBUTE,
                CanonicalValueKind::Sequence,
            )],
            TestOperation::Sum,
        )
    }
}

fn register_test_operation<const N: usize>(
    registrar: &mut SemanticRegistryRegistrar<'_>,
    key: OpKey,
    operands: u32,
    attributes: [OperationAttributeSchema; N],
    inferencer: TestOperation,
) -> Result<(), RegistryError> {
    registrar.register_operation(OperationDefinition::new(
        key,
        OperationSchema::new(
            OperationArity::exact(operands),
            OperationArity::exact(1),
            attributes,
        )
        .unwrap(),
        NormativeDefinitionRef::new("test governed operation semantics")?,
        OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
        OperationConformance::new(CanonicalValue::boolean(true)),
        OperationEffect::Pure,
        Arc::new(inferencer),
    ))
}

/// A provider the packaged graph never reaches.
struct UnusedSemantics {
    revision: u32,
}

impl SemanticRegistryProvider for UnusedSemantics {
    fn identity(&self) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
    }

    fn register(&self, registrar: &mut SemanticRegistryRegistrar<'_>) -> Result<(), RegistryError> {
        registrar.register_value_type(ValueTypeDefinition::structurally_valid(
            ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler-test", "unused", 1).unwrap()),
            NormativeDefinitionRef::new("unused test semantics")?,
            TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
        ))
    }
}

fn governed_program(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::new();
    registry
        .register_provider(&GovernedTestSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

fn program_with_unused_provider(revision: u32) -> SemanticProgram {
    let mut registry = SemanticRegistryBuilder::standard().unwrap();
    registry
        .register_provider(&UnusedSemantics { revision })
        .unwrap();
    build_graph(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
}

/// Artifact identity grows linearly with the ABI arena, on a chain and on a
/// shared DAG.
///
/// This is the instrument the flattening exists for, mirroring `tiler-ir`'s
/// `abi_identity_size_grows_linearly_with_the_arena`. Under the `v4` encoding a
/// node's key embedded its whole subtree, so the chain was quadratic and the
/// shared DAG **doubled per level** — a 16-level DAG reached megabytes. A
/// constant increment per level is the property that says the arena is written
/// once and referenced by position.
#[test]
fn artifact_identity_size_grows_linearly_with_the_abi_arena() {
    /// Enough levels that a quadratic or exponential curve is unmistakable, and
    /// few enough that a `v4` re-run would still finish.
    const LEVELS: std::ops::Range<usize> = 0..17;

    for shared in [false, true] {
        let mut sizes = Vec::new();
        for levels in LEVELS {
            let semantic = semantic_program();
            let program = fused_program(&semantic, SCALE_BITS);
            let provider = lowering_provider(1);
            let environment =
                CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
            let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
            draft.select_provider(selection(provider)).unwrap();
            let descriptor = draft.push_payload(payload(0xa1)).unwrap();
            let formulas = formulas(&mut draft);

            // Grow the guard, which is a use site, so every added node is
            // reached and verification admits the artifact.
            // Grown through a **launch precondition**, not the applicability
            // guard: the guard is derived from the program now, so it is no
            // longer a caller-supplied place to add arena depth. A precondition
            // is still artifact-owned and still reaches identity, so this
            // measures what it always measured -- identity size against arena
            // size -- through the seam that survives the binding.
            let mut grown = formulas.always;
            for _ in 0..levels {
                grown = if shared {
                    draft.push_binary(AbiBinaryOp::And, grown, grown).unwrap()
                } else {
                    let filler = draft.push_root(AbiRoot::BooleanLiteral(false)).unwrap();
                    draft.push_binary(AbiBinaryOp::Or, grown, filler).unwrap()
                };
            }
            let mut spec = variant(&formulas, descriptor, b"fused");
            spec.entries[0].launch.preconditions = vec![grown];
            draft.push_variant(&program, spec).unwrap();
            let artifact = draft.build().unwrap();

            let nodes = artifact.expressions().len();
            let bytes = artifact.canonical_identity().as_bytes().len();
            let shape = if shared { "SharedDag" } else { "Chain" };
            println!("MEASURE {shape} {levels:>2} levels: {nodes:>3} nodes, {bytes} bytes");
            sizes.push((nodes, bytes));
        }

        let increments: Vec<usize> = sizes
            .windows(2)
            .skip(1)
            .map(|pair| pair[1].1 - pair[0].1)
            .collect();
        assert!(
            increments.windows(2).all(|pair| pair[0] == pair[1]),
            "identity size must grow by a constant per level, measured {increments:?}"
        );
    }
}

/// `adopt_abi` replays a program's arena and resolves every reached position.
///
/// This is the mechanism that makes "a variant's ABI is its program's ABI"
/// checkable instead of a producer convention. The dedup assertion is the part
/// worth having: the builder keys by content, so replaying an arena that names
/// one expression from two positions must yield one handle, or a variant would
/// carry two spellings of one formula and the identity would distinguish them.
#[test]
fn adopting_a_program_abi_replays_every_reached_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let roots: Vec<u32> = (0..u32::try_from(arena.len()).unwrap()).collect();
    let minted = draft.adopt_abi(arena, &roots).expect("the arena replays");

    assert_eq!(minted.len(), arena.len());
    assert!(
        minted.iter().all(Option::is_some),
        "every position was named as a root, so every one must be replayed"
    );

    // Replaying the same arena again must mint no new handles: the builder
    // deduplicates by content, so the second pass resolves to the first's.
    let again = draft
        .adopt_abi(arena, &roots)
        .expect("the arena replays twice");
    assert_eq!(
        minted, again,
        "replay is not idempotent, so content dedup failed"
    );
}

/// A root outside the arena is a typed rejection, not a panic.
#[test]
fn adopting_an_abi_with_an_out_of_range_root_is_rejected() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let arena = program.abi_expressions();
    let beyond = u32::try_from(arena.len()).unwrap();
    assert_eq!(
        draft.adopt_abi(arena, &[beyond]),
        Err(ArtifactBuildError::ExpressionOutOfRange { position: beyond }),
    );
}

/// Does the artifact layer accept a *program-owned* ABI expression?
///
/// This is the question `reconcile-the-artifact-and-program-abi-expression-obligations`
/// exists to answer, isolated from the build path so a wiring fault in a
/// larger change cannot be mistaken for a layer disagreement. It adopts the
/// program's arena and then asks the artifact builder to accept the program's
/// own launch expression at the use site that expression is for.
#[test]
fn probe_whether_a_program_expression_satisfies_the_artifact_obligations() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new(std::iter::once(provider.clone())).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();

    let stage = program.stages().next().expect("one stage");
    let launch = stage.launch();
    let roots = vec![launch.grid_threads, launch.threads_per_workgroup];
    let adopted = draft
        .adopt_abi(program.abi_expressions(), &roots)
        .expect("the program arena replays onto the artifact builder");

    let grid = adopted[usize::try_from(launch.grid_threads).unwrap()]
        .expect("the grid expression was replayed");
    let workgroup = adopted[usize::try_from(launch.threads_per_workgroup).unwrap()]
        .expect("the workgroup expression was replayed");

    println!("PROBE grid handle {grid:?} workgroup handle {workgroup:?}");
    println!(
        "PROBE program arena {} nodes",
        program.abi_expressions().len()
    );
    println!("PROBE artifact arena after replay");

    // The handles are the artifact builder's own, minted by `adopt_abi`, so if
    // anything below fails it is an obligation and not a foreign handle.
    assert_ne!(grid, workgroup, "two distinct launch expressions collapsed");
}
