use std::error::Error;
use std::fmt;

use tiler_ir::shape::{Axis, Shape};

use crate::request::{
    NumericalPermission, PrototypeTargetProfile, StrictF32NumericalContract, SubnormalMode,
    VerifiedTargetRequest,
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
    pub(crate) region: ScheduledRegion,
    pub(crate) requirements: ResourceRequirements,
    pub(crate) target_profile_key: &'static str,
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
    pub(crate) kernel: StructuredKernel,
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
        verify_schedule(pointwise_region(request), &request.target_profile)?,
        verify_schedule(reduction_region(request), &request.target_profile)?,
    ])
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
                canonical_nan_bits: request.numerical_contract.canonical_arithmetic_nan_bits,
                contraction: request.numerical_contract.contraction
                    != NumericalPermission::Forbidden,
            },
            numerical: request.numerical_contract.into(),
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
                canonical_nan_bits: request.numerical_contract.canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract.into(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract.reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: false,
            },
            ..linear_schedule(request.serial_sum().output_elements, OwnershipWitnessId(1))
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
    target: &PrototypeTargetProfile,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let id = region.index.id;
    if region.index.numerical != StrictF32NumericalContract::governed().into() {
        return intrinsic("numerical-realization", id);
    }
    let iteration_count = element_count(&region.index.iteration_shape, id)?;
    if region.schedule.binding != ExecutionBinding::GlobalLinearInvocation
        || region.schedule.tail != TailPolicy::Exact
        || region.schedule.work_items != iteration_count
        || region.schedule.launch.grid_threads != iteration_count
        || region.schedule.launch.threads_per_workgroup != region.schedule.threads_per_workgroup
        || !region.schedule.launch.zero_work_skips_dispatch
    {
        return intrinsic("launch-coverage", id);
    }
    let [read, write] = region.index.accesses.as_slice() else {
        return intrinsic("access-count", id);
    };
    verify_access_and_semantics(&region, read, write)?;

    let requirements = ResourceRequirements {
        buffer_bindings: 2,
        threads_per_workgroup: region.schedule.threads_per_workgroup,
        local_memory_bytes: 0,
        barriers: 0,
        requires_device_memory: true,
        requires_strict_f32: true,
    };
    assess_target(id, requirements, region.schedule.work_items, target)?;
    Ok(VerifiedScheduledRegion {
        region,
        requirements,
        target_profile_key: target.key,
    })
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
    let (expected_read_bounds, expected_write_bounds, expected_owner) = match id {
        RegionId(0) => (
            BoundsWitnessId(0),
            BoundsWitnessId(1),
            OwnershipWitnessId(0),
        ),
        RegionId(1) => (
            BoundsWitnessId(2),
            BoundsWitnessId(3),
            OwnershipWitnessId(1),
        ),
        _ => return intrinsic("region-id", id),
    };
    if read.bounds != expected_read_bounds
        || write.bounds != expected_write_bounds
        || region.schedule.output_owner != expected_owner
    {
        return intrinsic("access-evidence", id);
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
    let body = match &region.index.scalar_program {
        ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits,
            canonical_nan_bits,
            contraction,
        } => StructuredBody::PredicatedPointwise {
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
        },
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
        } => {
            let contributor_count = axes.iter().try_fold(1_u64, |count, axis| {
                let index = usize::try_from(axis.get()).expect("u32 fits every supported host");
                count.checked_mul(match &read.map {
                    LogicalAccess::ReductionContributor { input_shape, .. } => {
                        input_shape.extents()[index].get()
                    }
                    LogicalAccess::LinearIdentity => 0,
                })
            });
            let Some(contributor_count) = contributor_count else {
                return Err(PhysicalError::ShapeProductOverflow {
                    region: region.index.id,
                });
            };
            if contributor_count == 0 {
                StructuredBody::EmptyReduction {
                    value_type: KernelValueType::F32,
                    output_count: region.schedule.work_items,
                    identity_bits: *empty_identity_bits,
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
                    axes: axes.clone(),
                    order: *order,
                    input_bounds: read.bounds,
                    output_bounds: write.bounds,
                    output_ownership: region.schedule.output_owner,
                    combine: BinaryF32::Add,
                    canonical_nan_bits: *canonical_nan_bits,
                }
            }
        }
    };
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
        ) => empty_reduction_body_refines(body, scalar, scheduled, write),
        (
            body @ StructuredBody::NonEmptySerialReduction { .. },
            scalar @ ScalarProgram::StrictSerialSum { .. },
        ) => nonempty_reduction_body_refines(body, scalar, scheduled, read, write),
        _ => false,
    }
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
        ..
    } = body
    else {
        return false;
    };
    let ScalarProgram::MultiplyThenAdd {
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
        && *operations == [BinaryF32::Multiply, BinaryF32::Add]
        && canonical_nan_bits == expected_nan
        && contraction == expected_contraction
}

fn empty_reduction_body_refines(
    body: &StructuredBody,
    scalar: &ScalarProgram,
    scheduled: &VerifiedScheduledRegion,
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
        ..
    } = scalar
    else {
        return false;
    };
    *value_type == KernelValueType::F32
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
        request.for_target(request.target_profiles[0])
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
    fn schedule_and_kernel_fail_closed_on_target_and_refinement_mismatches() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();

        let mut target = request.target_profile;
        target.max_buffer_bindings_per_entry = 1;
        assert_eq!(
            verify_schedule(regions[0].region.clone(), &target),
            Err(PhysicalError::Target {
                rule: "buffer-bindings",
                region: RegionId(0),
                required: 2,
                available: 1,
            })
        );

        let mut invalid_schedule = regions[1].region.clone();
        invalid_schedule.schedule.reduction = ReductionTopology::None;
        assert_eq!(
            verify_schedule(invalid_schedule, &request.target_profile),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId(1),
            })
        );

        let mut invalid_access = regions[0].region.clone();
        invalid_access.index.accesses[0].bounds = BoundsWitnessId(9);
        assert_eq!(
            verify_schedule(invalid_access, &request.target_profile),
            Err(PhysicalError::Intrinsic {
                rule: "access-evidence",
                region: RegionId(0),
            })
        );

        let mut invalid_proof = regions[0].region.clone();
        invalid_proof.index.bounds_proofs[0].kind =
            BoundsProofKind::LinearRange { element_count: 5 };
        assert_eq!(
            verify_schedule(invalid_proof, &request.target_profile),
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
            verify_schedule(invalid_numerics, &request.target_profile),
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
}
