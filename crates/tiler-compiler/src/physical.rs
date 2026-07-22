use std::error::Error;
use std::fmt;

use tiler_ir::shape::{Axis, Shape};

use crate::fusion::SemanticOccurrence;
use crate::request::{
    NumericalPermission, PrototypeTargetProfile, StrictF32NumericalContract, SubnormalMode,
    VerifiedRequestSubject, VerifiedTargetRequest,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionId(pub(crate) u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct BoundsWitnessId(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct OwnershipWitnessId(u8);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum TensorRole {
    Input,
    Intermediate,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AccessMode {
    Read,
    Write,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LogicalAccess {
    LinearIdentity,
    ReductionContributor {
        input_shape: Shape,
        output_shape: Shape,
        axes: Vec<Axis>,
        order: ContributorOrder,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ContributorOrder {
    OriginalAxisLexicographic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Access {
    pub(crate) tensor: TensorRole,
    pub(crate) mode: AccessMode,
    pub(crate) map: LogicalAccess,
    pub(crate) bounds: BoundsWitnessId,
    pub(crate) ownership: Option<OwnershipWitnessId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BoundsProofKind {
    LinearRange {
        element_count: u64,
    },
    ReductionDomain {
        input_shape: Shape,
        output_shape: Shape,
        axes: Vec<Axis>,
        order: ContributorOrder,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundsProof {
    pub(crate) id: BoundsWitnessId,
    pub(crate) tensor: TensorRole,
    pub(crate) kind: BoundsProofKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnershipProofKind {
    OneGlobalInvocationPerOutput { output_count: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OwnershipProof {
    pub(crate) id: OwnershipWitnessId,
    pub(crate) tensor: TensorRole,
    pub(crate) kind: OwnershipProofKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScalarProgram {
    MultiplyThenAdd {
        scale_bits: u32,
        bias_bits: u32,
        canonical_nan_bits: u32,
        contraction: bool,
    },
    StrictSerialSum {
        axes: Vec<Axis>,
        order: ContributorOrder,
        canonical_nan_bits: u32,
        empty_identity_bits: u32,
    },
    FusedMultiplyAddSerialSum {
        scale_bits: u32,
        bias_bits: u32,
        axes: Vec<Axis>,
        order: ContributorOrder,
        canonical_nan_bits: u32,
        empty_identity_bits: u32,
        contraction: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexRegion {
    pub(crate) id: RegionId,
    pub(crate) iteration_shape: Shape,
    pub(crate) accesses: Vec<Access>,
    pub(crate) bounds_proofs: Vec<BoundsProof>,
    pub(crate) ownership_proof: OwnershipProof,
    pub(crate) scalar_program: ScalarProgram,
    pub(crate) numerical: NumericalRealization,
    pub(crate) semantic_members: Vec<SemanticOccurrence>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NumericalRealization {
    pub(crate) profile_key: &'static str,
    pub(crate) canonical_arithmetic_nan_bits: u32,
    pub(crate) input_subnormals: SubnormalMode,
    pub(crate) result_subnormals: SubnormalMode,
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
}

impl From<StrictF32NumericalContract> for NumericalRealization {
    fn from(profile: StrictF32NumericalContract) -> Self {
        Self {
            profile_key: profile.key,
            canonical_arithmetic_nan_bits: profile.canonical_arithmetic_nan_bits,
            input_subnormals: profile.input_subnormals,
            result_subnormals: profile.result_subnormals,
            contraction: profile.contraction,
            reassociation: profile.reassociation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExecutionBinding {
    GlobalLinearInvocation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TailPolicy {
    Exact,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ReductionTopology {
    None,
    Serial {
        axes: Vec<Axis>,
        order: ContributorOrder,
        permits_reassociation: bool,
        permits_permutation: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct KernelSchedule {
    pub(crate) binding: ExecutionBinding,
    pub(crate) work_items: u64,
    pub(crate) threads_per_workgroup: u32,
    pub(crate) tail: TailPolicy,
    pub(crate) output_owner: OwnershipWitnessId,
    pub(crate) reduction: ReductionTopology,
    pub(crate) launch: LaunchPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LaunchPlan {
    pub(crate) grid_threads: u64,
    pub(crate) threads_per_workgroup: u32,
    pub(crate) zero_work_skips_dispatch: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScheduledRegion {
    pub(crate) index: IndexRegion,
    pub(crate) schedule: KernelSchedule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ResourceRequirements {
    pub(crate) buffer_bindings: u32,
    pub(crate) threads_per_workgroup: u32,
    pub(crate) local_memory_bytes: u64,
    pub(crate) barriers: u32,
    pub(crate) requires_device_memory: bool,
    pub(crate) requires_strict_f32: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedScheduledRegion {
    region: ScheduledRegion,
    requirements: ResourceRequirements,
    target_profile_key: &'static str,
    request_subject: VerifiedRequestSubject,
}

impl VerifiedScheduledRegion {
    pub(crate) const fn region(&self) -> &ScheduledRegion {
        &self.region
    }
    pub(crate) const fn requirements(&self) -> ResourceRequirements {
        self.requirements
    }
    pub(crate) const fn target_profile_key(&self) -> &'static str {
        self.target_profile_key
    }
    pub(crate) fn matches_request(&self, request: &VerifiedTargetRequest) -> bool {
        self.request_subject == request.subject()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KernelValueType {
    IndexU64,
    Bool,
    F32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BufferAccess {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KernelBuffer {
    pub(crate) tensor: TensorRole,
    pub(crate) access: BufferAccess,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryF32 {
    Multiply,
    Add,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StructuredBody {
    PredicatedPointwise {
        index_type: KernelValueType,
        predicate_type: KernelValueType,
        value_type: KernelValueType,
        extent: u64,
        input_bounds: BoundsWitnessId,
        output_bounds: BoundsWitnessId,
        output_ownership: OwnershipWitnessId,
        scale_bits: u32,
        bias_bits: u32,
        operations: Vec<BinaryF32>,
        canonical_nan_bits: u32,
        contraction: bool,
    },
    EmptyReduction {
        value_type: KernelValueType,
        output_count: u64,
        identity_bits: u32,
        output_bounds: BoundsWitnessId,
        output_ownership: OwnershipWitnessId,
    },
    NonEmptySerialReduction {
        value_type: KernelValueType,
        output_count: u64,
        contributor_count: u64,
        loop_start: u64,
        loop_end: u64,
        axes: Vec<Axis>,
        order: ContributorOrder,
        input_bounds: BoundsWitnessId,
        output_bounds: BoundsWitnessId,
        output_ownership: OwnershipWitnessId,
        combine: BinaryF32,
        canonical_nan_bits: u32,
    },
    FusedEmptyReduction {
        value_type: KernelValueType,
        output_count: u64,
        identity_bits: u32,
        output_bounds: BoundsWitnessId,
        output_ownership: OwnershipWitnessId,
    },
    FusedNonEmptySerialReduction {
        value_type: KernelValueType,
        output_count: u64,
        contributor_count: u64,
        loop_start: u64,
        loop_end: u64,
        axes: Vec<Axis>,
        order: ContributorOrder,
        input_bounds: BoundsWitnessId,
        output_bounds: BoundsWitnessId,
        output_ownership: OwnershipWitnessId,
        scale_bits: u32,
        bias_bits: u32,
        prologue_operations: Vec<BinaryF32>,
        combine: BinaryF32,
        canonical_nan_bits: u32,
        contraction: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StructuredKernel {
    pub(crate) scheduled_region: RegionId,
    pub(crate) buffers: Vec<KernelBuffer>,
    pub(crate) admitted_builtin: ExecutionBinding,
    pub(crate) body: StructuredBody,
    pub(crate) requirements: ResourceRequirements,
    pub(crate) numerical: NumericalRealization,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedStructuredKernel {
    kernel: StructuredKernel,
}

impl VerifiedStructuredKernel {
    pub(crate) const fn kernel(&self) -> &StructuredKernel {
        &self.kernel
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PhysicalError {
    Intrinsic {
        rule: &'static str,
        region: RegionId,
    },
    Target {
        rule: &'static str,
        region: RegionId,
        required: u64,
        available: u64,
    },
    Refinement {
        rule: &'static str,
        region: RegionId,
    },
    ShapeProductOverflow {
        region: RegionId,
    },
}

impl fmt::Display for PhysicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intrinsic { rule, region } => {
                write!(
                    formatter,
                    "schedule.intrinsic.{rule}: region {} rejected",
                    region.0
                )
            }
            Self::Target {
                rule,
                region,
                required,
                available,
            } => write!(
                formatter,
                "schedule.target.{rule}: region {} requires {required}, available {available}",
                region.0
            ),
            Self::Refinement { rule, region } => write!(
                formatter,
                "kernel.refinement.{rule}: kernel for region {} rejected",
                region.0
            ),
            Self::ShapeProductOverflow { region } => write!(
                formatter,
                "schedule.shape.element-count: region {} exceeds u64",
                region.0
            ),
        }
    }
}

impl Error for PhysicalError {}

pub(crate) fn build_scheduled_regions(
    request: &VerifiedTargetRequest,
) -> Result<Vec<VerifiedScheduledRegion>, PhysicalError> {
    Ok(vec![
        verify_schedule(pointwise_region(request), request)?,
        verify_schedule(reduction_region(request), request)?,
    ])
}

pub(crate) fn build_fused_scheduled_region(
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    verify_schedule(fused_region(request), request)
}

fn pointwise_region(request: &VerifiedTargetRequest) -> ScheduledRegion {
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId(0),
            iteration_shape: request.serial_sum().input_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Intermediate,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId(1),
                    ownership: Some(OwnershipWitnessId(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId(0),
                    tensor: TensorRole::Input,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().input_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId(1),
                    tensor: TensorRole::Intermediate,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().input_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId(0),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: request.serial_sum().input_elements,
                },
            },
            scalar_program: ScalarProgram::MultiplyThenAdd {
                scale_bits: request.serial_sum().scale_bits,
                bias_bits: request.serial_sum().bias_bits,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                contraction: request.numerical_contract().contraction
                    != NumericalPermission::Forbidden,
            },
            numerical: request.numerical_contract().into(),
            semantic_members: vec![
                SemanticOccurrence::ScaleConstant,
                SemanticOccurrence::Multiply,
                SemanticOccurrence::BiasConstant,
                SemanticOccurrence::Add,
            ],
        },
        schedule: linear_schedule(request.serial_sum().input_elements, OwnershipWitnessId(0)),
    }
}

fn reduction_region(request: &VerifiedTargetRequest) -> ScheduledRegion {
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId(1),
            iteration_shape: request.serial_sum().output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId(2),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId(3),
                    ownership: Some(OwnershipWitnessId(1)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId(2),
                    tensor: TensorRole::Intermediate,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId(3),
                    tensor: TensorRole::Output,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId(1),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: request.serial_sum().output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().into(),
            semantic_members: vec![SemanticOccurrence::StrictSum],
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: false,
            },
            ..linear_schedule(request.serial_sum().output_elements, OwnershipWitnessId(1))
        },
    }
}

fn fused_region(request: &VerifiedTargetRequest) -> ScheduledRegion {
    ScheduledRegion {
        index: IndexRegion {
            id: RegionId(0),
            iteration_shape: request.serial_sum().output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId(1),
                    ownership: Some(OwnershipWitnessId(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId(0),
                    tensor: TensorRole::Input,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId(1),
                    tensor: TensorRole::Output,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: request.serial_sum().output_elements,
                },
            },
            scalar_program: ScalarProgram::FusedMultiplyAddSerialSum {
                scale_bits: request.serial_sum().scale_bits,
                bias_bits: request.serial_sum().bias_bits,
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
                contraction: false,
            },
            numerical: request.numerical_contract().into(),
            semantic_members: SemanticOccurrence::ALL.to_vec(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(request.serial_sum().output_elements, OwnershipWitnessId(0))
        },
    }
}

fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

pub(crate) fn verify_schedule(
    region: ScheduledRegion,
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let id = region.index.id;
    let subject = request.subject();
    if !request.is_authoritative()
        || request.target_profile() != PrototypeTargetProfile::governed()
        || request.numerical_contract() != StrictF32NumericalContract::governed()
    {
        return intrinsic("request-subject", id);
    }
    if region.index.numerical != request.numerical_contract().into() {
        return intrinsic("numerical-realization", id);
    }
    let iteration_count = element_count(&region.index.iteration_shape, id)?;
    if region.schedule.binding != ExecutionBinding::GlobalLinearInvocation
        || region.schedule.tail != TailPolicy::Exact
        || region.schedule.work_items != iteration_count
        || region.schedule.launch.grid_threads != iteration_count
        || region.schedule.launch.threads_per_workgroup != region.schedule.threads_per_workgroup
        || region.schedule.threads_per_workgroup == 0
        || !region.schedule.launch.zero_work_skips_dispatch
    {
        return intrinsic("launch-coverage", id);
    }
    let [read, write] = region.index.accesses.as_slice() else {
        return intrinsic("access-count", id);
    };
    verify_access_and_semantics(&region, read, write)?;
    verify_region_subject_binding(&region, &subject)?;

    let requirements = ResourceRequirements {
        buffer_bindings: 2,
        threads_per_workgroup: region.schedule.threads_per_workgroup,
        local_memory_bytes: 0,
        barriers: 0,
        requires_device_memory: true,
        requires_strict_f32: true,
    };
    assess_target(
        id,
        requirements,
        region.schedule.work_items,
        &request.target_profile(),
    )?;
    Ok(VerifiedScheduledRegion {
        region,
        requirements,
        target_profile_key: request.target_profile().key,
        request_subject: subject,
    })
}

fn verify_region_subject_binding(
    region: &ScheduledRegion,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let normalized = subject.normalized();
    if !axes_are_canonical(normalized.reduction_axes(), normalized.input_shape().rank())
        || element_count(normalized.input_shape(), region.index.id)? != normalized.input_elements()
        || element_count(normalized.output_shape(), region.index.id)?
            != normalized.output_elements()
        || normalized
            .input_shape()
            .without_axes(normalized.reduction_axes())
            != *normalized.output_shape()
    {
        return intrinsic("request-subject-shape", region.index.id);
    }
    let expected = match &region.index.scalar_program {
        ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits,
            canonical_nan_bits,
            contraction,
        } => {
            region.index.semantic_members
                == [
                    SemanticOccurrence::ScaleConstant,
                    SemanticOccurrence::Multiply,
                    SemanticOccurrence::BiasConstant,
                    SemanticOccurrence::Add,
                ]
                && region.index.id == RegionId(0)
                && region.index.iteration_shape == *normalized.input_shape()
                && *scale_bits == normalized.scale_bits()
                && *bias_bits == normalized.bias_bits()
                && *canonical_nan_bits == subject.numerical_contract().canonical_arithmetic_nan_bits
                && *contraction
                    == (subject.numerical_contract().contraction != NumericalPermission::Forbidden)
        }
        ScalarProgram::StrictSerialSum {
            axes,
            canonical_nan_bits,
            ..
        } => {
            region.index.semantic_members == [SemanticOccurrence::StrictSum]
                && region.index.id == RegionId(1)
                && region.index.iteration_shape == *normalized.output_shape()
                && axes == normalized.reduction_axes()
                && reduction_access_matches(&region.index.accesses[0], normalized)
                && *canonical_nan_bits == subject.numerical_contract().canonical_arithmetic_nan_bits
        }
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            axes,
            canonical_nan_bits,
            ..
        } => {
            region.index.semantic_members == SemanticOccurrence::ALL
                && region.index.id == RegionId(0)
                && region.index.iteration_shape == *normalized.output_shape()
                && *scale_bits == normalized.scale_bits()
                && *bias_bits == normalized.bias_bits()
                && axes == normalized.reduction_axes()
                && reduction_access_matches(&region.index.accesses[0], normalized)
                && *canonical_nan_bits == subject.numerical_contract().canonical_arithmetic_nan_bits
        }
    };
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

fn reduction_access_matches(
    access: &Access,
    normalized: &crate::request::NormalizedSerialSumSubject,
) -> bool {
    matches!(
        &access.map,
        LogicalAccess::ReductionContributor { input_shape, output_shape, axes, .. }
            if input_shape == normalized.input_shape()
                && output_shape == normalized.output_shape()
                && axes == normalized.reduction_axes()
    )
}

fn verify_access_and_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), PhysicalError> {
    let id = region.index.id;
    if read.mode != AccessMode::Read
        || read.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
    {
        return intrinsic("access-contract", id);
    }
    verify_proof_records(region, read, write)?;
    match (
        &region.index.scalar_program,
        &region.schedule.reduction,
        &read.map,
    ) {
        (
            ScalarProgram::MultiplyThenAdd { contraction, .. },
            ReductionTopology::None,
            LogicalAccess::LinearIdentity,
        ) if *contraction
            == (region.index.numerical.contraction != NumericalPermission::Forbidden)
            && read.tensor == TensorRole::Input
            && write.tensor == TensorRole::Intermediate => {}
        (
            ScalarProgram::StrictSerialSum {
                axes,
                order,
                empty_identity_bits,
                ..
            },
            ReductionTopology::Serial {
                axes: scheduled_axes,
                order: scheduled_order,
                permits_reassociation,
                permits_permutation,
            },
            LogicalAccess::ReductionContributor {
                input_shape,
                output_shape,
                axes: access_axes,
                order: access_order,
            },
        ) if axes == scheduled_axes
            && axes == access_axes
            && order == scheduled_order
            && order == access_order
            && *permits_reassociation
                == (region.index.numerical.reassociation != NumericalPermission::Forbidden)
            && !permits_permutation
            && *empty_identity_bits == 0.0_f32.to_bits()
            && output_shape == &region.index.iteration_shape
            && input_shape.without_axes(axes) == *output_shape
            && read.tensor == TensorRole::Intermediate
            && write.tensor == TensorRole::Output => {}
        (
            ScalarProgram::FusedMultiplyAddSerialSum {
                axes,
                order,
                empty_identity_bits,
                contraction,
                ..
            },
            ReductionTopology::Serial {
                axes: scheduled_axes,
                order: scheduled_order,
                permits_reassociation,
                permits_permutation,
            },
            LogicalAccess::ReductionContributor {
                input_shape,
                output_shape,
                axes: access_axes,
                order: access_order,
            },
        ) if axes == scheduled_axes
            && axes == access_axes
            && order == scheduled_order
            && order == access_order
            && !permits_reassociation
            && !permits_permutation
            && !contraction
            && *empty_identity_bits == 0.0_f32.to_bits()
            && output_shape == &region.index.iteration_shape
            && input_shape.without_axes(axes) == *output_shape
            && read.tensor == TensorRole::Input
            && write.tensor == TensorRole::Output => {}
        _ => return intrinsic("numerical-or-access-refinement", id),
    }
    Ok(())
}

fn verify_proof_records(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), PhysicalError> {
    let [read_proof, write_proof] = region.index.bounds_proofs.as_slice() else {
        return intrinsic("bounds-proof-count", region.index.id);
    };
    if read_proof.id != read.bounds
        || read_proof.tensor != read.tensor
        || write_proof.id != write.bounds
        || write_proof.tensor != write.tensor
        || read_proof.id == write_proof.id
        || region.index.ownership_proof.id != region.schedule.output_owner
        || region.index.ownership_proof.tensor != write.tensor
        || region.index.ownership_proof.kind
            != (OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: region.schedule.work_items,
            })
    {
        return intrinsic("proof-reference", region.index.id);
    }
    if !bounds_proof_refines_access(read_proof, &read.map, region)
        || !bounds_proof_refines_access(write_proof, &write.map, region)
    {
        return intrinsic("bounds-proof", region.index.id);
    }
    Ok(())
}

fn bounds_proof_refines_access(
    proof: &BoundsProof,
    access: &LogicalAccess,
    region: &ScheduledRegion,
) -> bool {
    match (&proof.kind, access) {
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LinearIdentity) => {
            *element_count == region.schedule.work_items
        }
        (
            BoundsProofKind::ReductionDomain {
                input_shape,
                output_shape,
                axes,
                order,
            },
            LogicalAccess::ReductionContributor {
                input_shape: access_input,
                output_shape: access_output,
                axes: access_axes,
                order: access_order,
            },
        ) => {
            input_shape == access_input
                && output_shape == access_output
                && axes == access_axes
                && order == access_order
                && output_shape == &region.index.iteration_shape
                && input_shape.without_axes(axes) == *output_shape
        }
        _ => false,
    }
}

fn assess_target(
    region: RegionId,
    requirements: ResourceRequirements,
    work_items: u64,
    target: &PrototypeTargetProfile,
) -> Result<(), PhysicalError> {
    if work_items > target.max_threads_per_grid_axis {
        return target_error(
            "grid-axis",
            region,
            work_items,
            target.max_threads_per_grid_axis,
        );
    }
    if requirements.threads_per_workgroup > target.max_threads_per_workgroup {
        return target_error(
            "threads-per-workgroup",
            region,
            u64::from(requirements.threads_per_workgroup),
            u64::from(target.max_threads_per_workgroup),
        );
    }
    if requirements.buffer_bindings > target.max_buffer_bindings_per_entry {
        return target_error(
            "buffer-bindings",
            region,
            u64::from(requirements.buffer_bindings),
            u64::from(target.max_buffer_bindings_per_entry),
        );
    }
    if target.index_bits != 64
        || (requirements.requires_device_memory && !target.supports_device_memory)
        || (requirements.requires_strict_f32 && !target.supports_strict_f32)
        || requirements.local_memory_bytes != 0
        || requirements.barriers != 0
    {
        return target_error("capability", region, 1, 0);
    }
    Ok(())
}

pub(crate) fn lower_structured_kernel(
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedStructuredKernel, PhysicalError> {
    let region = &scheduled.region;
    let [read, write] = region.index.accesses.as_slice() else {
        return refinement("access-count", region.index.id);
    };
    let body = lower_structured_body(region, read, write)?;
    verify_kernel(
        StructuredKernel {
            scheduled_region: region.index.id,
            buffers: vec![
                KernelBuffer {
                    tensor: read.tensor,
                    access: BufferAccess::Read,
                },
                KernelBuffer {
                    tensor: write.tensor,
                    access: BufferAccess::Write,
                },
            ],
            admitted_builtin: region.schedule.binding,
            body,
            requirements: scheduled.requirements,
            numerical: region.index.numerical,
        },
        scheduled,
    )
}

fn lower_structured_body(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<StructuredBody, PhysicalError> {
    match &region.index.scalar_program {
        ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits,
            canonical_nan_bits,
            contraction,
        } => Ok(StructuredBody::PredicatedPointwise {
            index_type: KernelValueType::IndexU64,
            predicate_type: KernelValueType::Bool,
            value_type: KernelValueType::F32,
            extent: region.schedule.work_items,
            input_bounds: read.bounds,
            output_bounds: write.bounds,
            output_ownership: region.schedule.output_owner,
            scale_bits: *scale_bits,
            bias_bits: *bias_bits,
            operations: vec![BinaryF32::Multiply, BinaryF32::Add],
            canonical_nan_bits: *canonical_nan_bits,
            contraction: *contraction,
        }),
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
        } => lower_serial_reduction(
            region,
            read,
            write,
            axes,
            *order,
            *canonical_nan_bits,
            *empty_identity_bits,
        ),
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
            contraction,
        } => lower_fused_reduction(
            region,
            read,
            write,
            FusedReductionSpec {
                scale_bits: *scale_bits,
                bias_bits: *bias_bits,
                axes,
                order: *order,
                canonical_nan_bits: *canonical_nan_bits,
                empty_identity_bits: *empty_identity_bits,
                contraction: *contraction,
            },
        ),
    }
}

fn lower_serial_reduction(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
    axes: &[Axis],
    order: ContributorOrder,
    canonical_nan_bits: u32,
    empty_identity_bits: u32,
) -> Result<StructuredBody, PhysicalError> {
    let contributor_count = contributor_count(axes, &read.map, region.index.id)?;
    Ok(if contributor_count == 0 {
        StructuredBody::EmptyReduction {
            value_type: KernelValueType::F32,
            output_count: region.schedule.work_items,
            identity_bits: empty_identity_bits,
            output_bounds: write.bounds,
            output_ownership: region.schedule.output_owner,
        }
    } else {
        StructuredBody::NonEmptySerialReduction {
            value_type: KernelValueType::F32,
            output_count: region.schedule.work_items,
            contributor_count,
            loop_start: 1,
            loop_end: contributor_count,
            axes: axes.to_vec(),
            order,
            input_bounds: read.bounds,
            output_bounds: write.bounds,
            output_ownership: region.schedule.output_owner,
            combine: BinaryF32::Add,
            canonical_nan_bits,
        }
    })
}

#[derive(Clone, Copy)]
struct FusedReductionSpec<'a> {
    scale_bits: u32,
    bias_bits: u32,
    axes: &'a [Axis],
    order: ContributorOrder,
    canonical_nan_bits: u32,
    empty_identity_bits: u32,
    contraction: bool,
}

fn lower_fused_reduction(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
    spec: FusedReductionSpec<'_>,
) -> Result<StructuredBody, PhysicalError> {
    let contributor_count = contributor_count(spec.axes, &read.map, region.index.id)?;
    Ok(if contributor_count == 0 {
        StructuredBody::FusedEmptyReduction {
            value_type: KernelValueType::F32,
            output_count: region.schedule.work_items,
            identity_bits: spec.empty_identity_bits,
            output_bounds: write.bounds,
            output_ownership: region.schedule.output_owner,
        }
    } else {
        StructuredBody::FusedNonEmptySerialReduction {
            value_type: KernelValueType::F32,
            output_count: region.schedule.work_items,
            contributor_count,
            loop_start: 1,
            loop_end: contributor_count,
            axes: spec.axes.to_vec(),
            order: spec.order,
            input_bounds: read.bounds,
            output_bounds: write.bounds,
            output_ownership: region.schedule.output_owner,
            scale_bits: spec.scale_bits,
            bias_bits: spec.bias_bits,
            prologue_operations: vec![BinaryF32::Multiply, BinaryF32::Add],
            combine: BinaryF32::Add,
            canonical_nan_bits: spec.canonical_nan_bits,
            contraction: spec.contraction,
        }
    })
}

pub(crate) fn verify_kernel(
    kernel: StructuredKernel,
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedStructuredKernel, PhysicalError> {
    let id = scheduled.region.index.id;
    let [read, write] = scheduled.region.index.accesses.as_slice() else {
        return refinement("access-count", id);
    };
    if kernel.scheduled_region != id
        || kernel.admitted_builtin != scheduled.region.schedule.binding
        || kernel.requirements != scheduled.requirements
        || kernel.numerical != scheduled.region.index.numerical
        || kernel.buffers
            != [
                KernelBuffer {
                    tensor: read.tensor,
                    access: BufferAccess::Read,
                },
                KernelBuffer {
                    tensor: write.tensor,
                    access: BufferAccess::Write,
                },
            ]
    {
        return refinement("signature-or-requirements", id);
    }
    if !body_refines_schedule(&kernel.body, scheduled, read, write) {
        return refinement("body", id);
    }
    Ok(VerifiedStructuredKernel { kernel })
}

fn body_refines_schedule(
    body: &StructuredBody,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    match (body, &scheduled.region.index.scalar_program) {
        (
            body @ StructuredBody::PredicatedPointwise { .. },
            scalar @ ScalarProgram::MultiplyThenAdd { .. },
        ) => pointwise_body_refines(body, scalar, scheduled, read, write),
        (
            body @ StructuredBody::EmptyReduction { .. },
            scalar @ ScalarProgram::StrictSerialSum { .. },
        ) => empty_reduction_body_refines(body, scalar, scheduled, read, write),
        (
            body @ StructuredBody::NonEmptySerialReduction { .. },
            scalar @ ScalarProgram::StrictSerialSum { .. },
        ) => nonempty_reduction_body_refines(body, scalar, scheduled, read, write),
        (
            body @ StructuredBody::FusedEmptyReduction { .. },
            scalar @ ScalarProgram::FusedMultiplyAddSerialSum { .. },
        ) => fused_empty_body_refines(body, scalar, scheduled, read, write),
        (
            body @ StructuredBody::FusedNonEmptySerialReduction { .. },
            scalar @ ScalarProgram::FusedMultiplyAddSerialSum { .. },
        ) => fused_nonempty_body_refines(body, scalar, scheduled, read, write),
        _ => false,
    }
}

fn contributor_count(
    axes: &[Axis],
    access: &LogicalAccess,
    region: RegionId,
) -> Result<u64, PhysicalError> {
    let LogicalAccess::ReductionContributor { input_shape, .. } = access else {
        return intrinsic("contributor-access", region);
    };
    if !axes_are_canonical(axes, input_shape.rank()) {
        return intrinsic("contributor-axes", region);
    }
    let extents = axes
        .iter()
        .map(|axis| {
            usize::try_from(axis.get())
                .ok()
                .and_then(|index| input_shape.extents().get(index))
                .map(|extent| extent.get())
                .ok_or(PhysicalError::Intrinsic {
                    rule: "contributor-axis",
                    region,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if extents.contains(&0) {
        return Ok(0);
    }
    extents.into_iter().try_fold(1_u64, |count, extent| {
        count.checked_mul(extent).ok_or(PhysicalError::Intrinsic {
            rule: "contributor-product",
            region,
        })
    })
}

fn axes_are_canonical(axes: &[Axis], rank: usize) -> bool {
    let mut previous = None;
    axes.iter().all(|axis| {
        let Ok(index) = usize::try_from(axis.get()) else {
            return false;
        };
        let canonical = index < rank && previous.is_none_or(|previous| previous < axis.get());
        previous = Some(axis.get());
        canonical
    })
}

fn fused_empty_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    let StructuredBody::FusedEmptyReduction {
        value_type,
        output_count,
        identity_bits,
        output_bounds,
        output_ownership,
    } = body
    else {
        return false;
    };
    let ScalarProgram::FusedMultiplyAddSerialSum {
        empty_identity_bits,
        ..
    } = scalar
    else {
        return false;
    };
    *value_type == KernelValueType::F32
        && contributor_count(
            match scalar {
                ScalarProgram::FusedMultiplyAddSerialSum { axes, .. } => axes,
                _ => return false,
            },
            &read.map,
            scheduled.region.index.id,
        ) == Ok(0)
        && *output_count == scheduled.region.schedule.work_items
        && identity_bits == empty_identity_bits
        && *output_bounds == write.bounds
        && *output_ownership == scheduled.region.schedule.output_owner
}

fn fused_nonempty_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    let StructuredBody::FusedNonEmptySerialReduction {
        value_type,
        output_count,
        contributor_count,
        loop_start,
        loop_end,
        axes,
        order,
        input_bounds,
        output_bounds,
        output_ownership,
        scale_bits,
        bias_bits,
        prologue_operations,
        combine,
        canonical_nan_bits,
        contraction,
    } = body
    else {
        return false;
    };
    let ScalarProgram::FusedMultiplyAddSerialSum {
        scale_bits: expected_scale,
        bias_bits: expected_bias,
        axes: expected_axes,
        order: expected_order,
        canonical_nan_bits: expected_nan,
        contraction: expected_contraction,
        ..
    } = scalar
    else {
        return false;
    };
    *value_type == KernelValueType::F32
        && *output_count == scheduled.region.schedule.work_items
        && self::contributor_count(expected_axes, &read.map, scheduled.region.index.id)
            == Ok(*contributor_count)
        && *contributor_count > 0
        && *loop_start == 1
        && *loop_end == *contributor_count
        && axes == expected_axes
        && order == expected_order
        && *input_bounds == read.bounds
        && *output_bounds == write.bounds
        && *output_ownership == scheduled.region.schedule.output_owner
        && scale_bits == expected_scale
        && bias_bits == expected_bias
        && *prologue_operations == [BinaryF32::Multiply, BinaryF32::Add]
        && *combine == BinaryF32::Add
        && canonical_nan_bits == expected_nan
        && contraction == expected_contraction
}

fn pointwise_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    let StructuredBody::PredicatedPointwise {
        index_type,
        predicate_type,
        value_type,
        extent,
        input_bounds,
        output_bounds,
        output_ownership,
        operations,
        canonical_nan_bits,
        contraction,
        scale_bits,
        bias_bits,
    } = body
    else {
        return false;
    };
    let ScalarProgram::MultiplyThenAdd {
        scale_bits: expected_scale,
        bias_bits: expected_bias,
        canonical_nan_bits: expected_nan,
        contraction: expected_contraction,
        ..
    } = scalar
    else {
        return false;
    };
    *index_type == KernelValueType::IndexU64
        && *predicate_type == KernelValueType::Bool
        && *value_type == KernelValueType::F32
        && *extent == scheduled.region.schedule.work_items
        && *input_bounds == read.bounds
        && *output_bounds == write.bounds
        && *output_ownership == scheduled.region.schedule.output_owner
        && scale_bits == expected_scale
        && bias_bits == expected_bias
        && *operations == [BinaryF32::Multiply, BinaryF32::Add]
        && canonical_nan_bits == expected_nan
        && contraction == expected_contraction
}

fn empty_reduction_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    let StructuredBody::EmptyReduction {
        value_type,
        output_count,
        identity_bits,
        output_bounds,
        output_ownership,
    } = body
    else {
        return false;
    };
    let ScalarProgram::StrictSerialSum {
        empty_identity_bits,
        axes,
        ..
    } = scalar
    else {
        return false;
    };
    *value_type == KernelValueType::F32
        && contributor_count(axes, &read.map, scheduled.region.index.id) == Ok(0)
        && *output_count == scheduled.region.schedule.work_items
        && identity_bits == empty_identity_bits
        && *output_bounds == write.bounds
        && *output_ownership == scheduled.region.schedule.output_owner
}

fn nonempty_reduction_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
    read: &Access,
    write: &Access,
) -> bool {
    let StructuredBody::NonEmptySerialReduction {
        value_type,
        output_count,
        contributor_count,
        loop_start,
        loop_end,
        axes,
        order,
        input_bounds,
        output_bounds,
        output_ownership,
        combine,
        canonical_nan_bits,
    } = body
    else {
        return false;
    };
    let ScalarProgram::StrictSerialSum {
        axes: expected_axes,
        order: expected_order,
        canonical_nan_bits: expected_nan,
        ..
    } = scalar
    else {
        return false;
    };
    *value_type == KernelValueType::F32
        && *output_count == scheduled.region.schedule.work_items
        && self::contributor_count(expected_axes, &read.map, scheduled.region.index.id)
            == Ok(*contributor_count)
        && *contributor_count > 0
        && *loop_start == 1
        && *loop_end == *contributor_count
        && axes == expected_axes
        && order == expected_order
        && *input_bounds == read.bounds
        && *output_bounds == write.bounds
        && *output_ownership == scheduled.region.schedule.output_owner
        && *combine == BinaryF32::Add
        && canonical_nan_bits == expected_nan
}

fn element_count(shape: &Shape, region: RegionId) -> Result<u64, PhysicalError> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape.extents().iter().try_fold(1_u64, |count, extent| {
        count
            .checked_mul(extent.get())
            .ok_or(PhysicalError::ShapeProductOverflow { region })
    })
}

fn intrinsic<T>(rule: &'static str, region: RegionId) -> Result<T, PhysicalError> {
    Err(PhysicalError::Intrinsic { rule, region })
}

fn target_error<T>(
    rule: &'static str,
    region: RegionId,
    required: u64,
    available: u64,
) -> Result<T, PhysicalError> {
    Err(PhysicalError::Target {
        rule,
        region,
        required,
        available,
    })
}

fn refinement<T>(rule: &'static str, region: RegionId) -> Result<T, PhysicalError> {
    Err(PhysicalError::Refinement { rule, region })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };

    fn request(shape: Shape, axes: impl IntoIterator<Item = Axis>) -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, axes).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed(&program)).unwrap();
        request.for_target(request.target_profiles()[0]).unwrap()
    }

    #[test]
    fn fixed_schedules_and_kernels_refine_the_two_regions() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let pointwise = lower_structured_kernel(&regions[0]).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();

        assert_eq!(regions[0].region.schedule.work_items, 6);
        assert_eq!(regions[1].region.schedule.work_items, 2);
        let StructuredBody::PredicatedPointwise { operations, .. } = pointwise.kernel.body else {
            panic!("expected pointwise body");
        };
        assert_eq!(operations, [BinaryF32::Multiply, BinaryF32::Add]);
        assert!(matches!(
            reduction.kernel.body,
            StructuredBody::NonEmptySerialReduction {
                contributor_count: 3,
                loop_start: 1,
                loop_end: 3,
                ..
            }
        ));
    }

    #[test]
    fn empty_reduction_lowers_to_explicit_positive_zero_stores() {
        let request = request(Shape::from_dims([2, 0]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();
        assert!(matches!(
            reduction.kernel.body,
            StructuredBody::EmptyReduction {
                output_count: 2,
                identity_bits: 0,
                ..
            }
        ));
    }

    #[test]
    fn schedule_and_kernel_fail_closed_on_refinement_mismatches() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();

        let mut invalid_schedule = regions[1].region.clone();
        invalid_schedule.schedule.reduction = ReductionTopology::None;
        assert_eq!(
            verify_schedule(invalid_schedule, &request),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId(1),
            })
        );

        let mut invalid_access = regions[0].region.clone();
        invalid_access.index.accesses[0].bounds = BoundsWitnessId(9);
        assert_eq!(
            verify_schedule(invalid_access, &request),
            Err(PhysicalError::Intrinsic {
                rule: "proof-reference",
                region: RegionId(0),
            })
        );

        let mut invalid_proof = regions[0].region.clone();
        invalid_proof.index.bounds_proofs[0].kind =
            BoundsProofKind::LinearRange { element_count: 5 };
        assert_eq!(
            verify_schedule(invalid_proof, &request),
            Err(PhysicalError::Intrinsic {
                rule: "bounds-proof",
                region: RegionId(0),
            })
        );

        let mut invalid_numerics = regions[0].region.clone();
        invalid_numerics
            .index
            .numerical
            .canonical_arithmetic_nan_bits ^= 1;
        assert_eq!(
            verify_schedule(invalid_numerics, &request),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-realization",
                region: RegionId(0),
            })
        );

        let mut invalid_kernel = lower_structured_kernel(&regions[0]).unwrap().kernel;
        let StructuredBody::PredicatedPointwise {
            output_ownership, ..
        } = &mut invalid_kernel.body
        else {
            panic!("expected pointwise body")
        };
        *output_ownership = OwnershipWitnessId(9);
        assert_eq!(
            verify_kernel(invalid_kernel, &regions[0]),
            Err(PhysicalError::Refinement {
                rule: "body",
                region: RegionId(0),
            })
        );
    }

    #[test]
    fn reduction_access_and_proof_shapes_are_bound_to_the_verified_request() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let fused = build_fused_scheduled_region(&request).unwrap();

        for mut forged in [regions[1].region.clone(), fused.region.clone()] {
            let region = forged.index.id;
            let LogicalAccess::ReductionContributor { input_shape, .. } =
                &mut forged.index.accesses[0].map
            else {
                panic!("expected reduction access")
            };
            *input_shape = Shape::from_dims([2, 4]);
            let BoundsProofKind::ReductionDomain { input_shape, .. } =
                &mut forged.index.bounds_proofs[0].kind
            else {
                panic!("expected reduction proof")
            };
            *input_shape = Shape::from_dims([2, 4]);

            assert_eq!(
                verify_schedule(forged, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    region,
                })
            );
        }
    }

    #[test]
    fn fused_schedule_and_kernel_reject_numerical_and_body_corruption() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();
        let mut invalid_schedule = scheduled.region.clone();
        let ScalarProgram::FusedMultiplyAddSerialSum { contraction, .. } =
            &mut invalid_schedule.index.scalar_program
        else {
            panic!("expected fused scalar program")
        };
        *contraction = true;
        assert_eq!(
            verify_schedule(invalid_schedule, &request),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId(0),
            })
        );

        let mut invalid_kernel = lower_structured_kernel(&scheduled).unwrap().kernel;
        let StructuredBody::FusedNonEmptySerialReduction {
            prologue_operations,
            ..
        } = &mut invalid_kernel.body
        else {
            panic!("expected fused reduction body")
        };
        prologue_operations.reverse();
        assert_eq!(
            verify_kernel(invalid_kernel, &scheduled),
            Err(PhysicalError::Refinement {
                rule: "body",
                region: RegionId(0),
            })
        );
    }

    #[test]
    fn malformed_axes_zero_launch_and_late_zero_products_fail_without_panicking() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();

        let mut zero_threads = scheduled.region.clone();
        zero_threads.schedule.threads_per_workgroup = 0;
        zero_threads.schedule.launch.threads_per_workgroup = 0;
        assert!(matches!(
            verify_schedule(zero_threads, &request),
            Err(PhysicalError::Intrinsic {
                rule: "launch-coverage",
                ..
            })
        ));

        for axes in [vec![Axis::new(1), Axis::new(1)], vec![Axis::new(99)]] {
            let mut malformed = scheduled.region.clone();
            if let ScalarProgram::FusedMultiplyAddSerialSum {
                axes: scalar_axes, ..
            } = &mut malformed.index.scalar_program
            {
                *scalar_axes = axes.clone();
            }
            if let ReductionTopology::Serial {
                axes: schedule_axes,
                ..
            } = &mut malformed.schedule.reduction
            {
                *schedule_axes = axes.clone();
            }
            if let LogicalAccess::ReductionContributor {
                axes: access_axes, ..
            } = &mut malformed.index.accesses[0].map
            {
                *access_axes = axes.clone();
            }
            if let BoundsProofKind::ReductionDomain {
                axes: proof_axes, ..
            } = &mut malformed.index.bounds_proofs[0].kind
            {
                *proof_axes = axes;
            }
            assert!(matches!(
                verify_schedule(malformed, &request),
                Err(PhysicalError::Intrinsic { .. })
            ));
        }

        let late_zero = LogicalAccess::ReductionContributor {
            input_shape: Shape::from_dims([u64::MAX, 2, 0]),
            output_shape: Shape::from_dims([]),
            axes: vec![Axis::new(0), Axis::new(1), Axis::new(2)],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        assert_eq!(
            contributor_count(
                &[Axis::new(0), Axis::new(1), Axis::new(2)],
                &late_zero,
                RegionId(7),
            ),
            Ok(0)
        );
    }

    #[test]
    fn structured_kernel_constants_and_contributor_counts_are_rederived() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();

        let mut pointwise = lower_structured_kernel(&regions[0]).unwrap().kernel;
        let StructuredBody::PredicatedPointwise { scale_bits, .. } = &mut pointwise.body else {
            panic!("expected pointwise body")
        };
        *scale_bits ^= 1;
        assert!(matches!(
            verify_kernel(pointwise, &regions[0]),
            Err(PhysicalError::Refinement { rule: "body", .. })
        ));

        let mut reduction = lower_structured_kernel(&regions[1]).unwrap().kernel;
        let StructuredBody::NonEmptySerialReduction {
            contributor_count,
            loop_end,
            ..
        } = &mut reduction.body
        else {
            panic!("expected reduction body")
        };
        *contributor_count += 1;
        *loop_end += 1;
        assert!(matches!(
            verify_kernel(reduction, &regions[1]),
            Err(PhysicalError::Refinement { rule: "body", .. })
        ));
    }
}
