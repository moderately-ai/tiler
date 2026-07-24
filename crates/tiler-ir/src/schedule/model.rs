//! Target-neutral scheduled-region data model, verified wrapper, and identity.
//!
//! A [`ScheduledRegion`] pairs a bounded [`IndexRegion`] with a normalized
//! [`KernelSchedule`] (ADR 0007). The descriptor structs are read-transparent
//! value data; only [`super::ScheduledRegionBuilder::build`] can bind a region
//! into an opaque [`VerifiedScheduledRegion`] after intrinsic verification.

use crate::shape::{Axis, Shape};

use super::error::{ContributorError, ElementCountOverflow};
use super::handles::{BoundsWitnessId, OwnershipWitnessId, RegionId};
use super::numerics::{FlushedZeroSign, NumericalPermission, NumericalRealization, SubnormalMode};

/// The role a boundary tensor plays for one scheduled region.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TensorRole {
    /// A program input consumed by the region.
    Input,
    /// A materialized intermediate produced or consumed by the region.
    Intermediate,
    /// A program output produced by the region.
    Output,
}

/// Whether an access reads or writes its tensor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AccessMode {
    /// The access reads its tensor.
    Read,
    /// The access writes its tensor.
    Write,
}

/// Canonical order in which reduction contributors combine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ContributorOrder {
    /// Contributors combine in ascending original-axis lexicographic order.
    OriginalAxisLexicographic,
}

/// The logical coordinate map a scheduled access realizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LogicalAccess {
    /// One iteration coordinate maps to one linear element position.
    LinearIdentity,
    /// Each output coordinate reads a family of contributor coordinates.
    ReductionContributor {
        /// Shape of the reduced input.
        input_shape: Shape,
        /// Shape of the reduced output.
        output_shape: Shape,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
    },
}

/// One logical tensor access performed by a scheduled region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Access {
    /// Boundary tensor role.
    pub tensor: TensorRole,
    /// Whether the access reads or writes.
    pub mode: AccessMode,
    /// Logical coordinate map.
    pub map: LogicalAccess,
    /// Bounds proof witness attached to this access.
    pub bounds: BoundsWitnessId,
    /// Write-ownership witness, present only for owning writes.
    pub ownership: Option<OwnershipWitnessId>,
}

/// The structure a bounds proof establishes for an access domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoundsProofKind {
    /// A contiguous linear range of `element_count` positions.
    LinearRange {
        /// Number of in-range positions.
        element_count: u64,
    },
    /// A reduction domain relating input and output coordinates.
    ReductionDomain {
        /// Shape of the reduced input.
        input_shape: Shape,
        /// Shape of the reduced output.
        output_shape: Shape,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
    },
}

/// A witnessed proof that an access stays within its tensor bounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundsProof {
    /// Witness identity referenced by the proving access.
    pub id: BoundsWitnessId,
    /// Tensor the proof applies to.
    pub tensor: TensorRole,
    /// Proven domain structure.
    pub kind: BoundsProofKind,
}

/// The structure a write-ownership proof establishes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipProofKind {
    /// Exactly one global invocation writes each of `output_count` positions.
    OneGlobalInvocationPerOutput {
        /// Number of distinct owned output positions.
        output_count: u64,
    },
}

/// A witnessed proof that writes are total and race-free.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OwnershipProof {
    /// Witness identity referenced by the schedule and owning write.
    pub id: OwnershipWitnessId,
    /// Tensor the proof applies to.
    pub tensor: TensorRole,
    /// Proven ownership structure.
    pub kind: OwnershipProofKind,
}

/// The scalar program a region evaluates per output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScalarProgram {
    /// A pointwise scale-then-bias application.
    MultiplyThenAdd {
        /// Scale constant bit pattern.
        scale_bits: u32,
        /// Bias constant bit pattern.
        bias_bits: u32,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Whether contraction is permitted.
        contraction: bool,
    },
    /// A strict serial reduction sum.
    StrictSerialSum {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern.
        empty_identity_bits: u32,
    },
    /// A fused scale-bias-then-serial-sum reduction.
    FusedMultiplyAddSerialSum {
        /// Scale constant bit pattern.
        scale_bits: u32,
        /// Bias constant bit pattern.
        bias_bits: u32,
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Canonical arithmetic NaN bit pattern.
        canonical_nan_bits: u32,
        /// Empty-reduction identity bit pattern.
        empty_identity_bits: u32,
        /// Whether contraction is permitted.
        contraction: bool,
    },
}

/// The bounded index region a schedule maps onto a target machine.
///
/// This carries the iteration domain, logical accesses, bounds and ownership
/// proofs, the scalar program, and the numerical realization. It deliberately
/// does not carry any semantic-graph correlation; binding a region to semantic
/// occurrences is a separate compiler-owned refinement (ADR 0070).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexRegion {
    /// Planning ordinal, excluded from canonical identity.
    pub id: RegionId,
    /// Parallel iteration domain of the region.
    pub iteration_shape: Shape,
    /// Logical accesses, one read followed by one owning write.
    pub accesses: Vec<Access>,
    /// Bounds proofs, one per access.
    pub bounds_proofs: Vec<BoundsProof>,
    /// The single write-ownership proof.
    pub ownership_proof: OwnershipProof,
    /// Scalar program evaluated per output.
    pub scalar_program: ScalarProgram,
    /// Preserved numerical realization.
    pub numerical: NumericalRealization,
}

/// How a region binds execution coordinates to iteration coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionBinding {
    /// One global linear invocation per iteration coordinate.
    GlobalLinearInvocation,
}

/// How iteration-domain tail elements are handled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TailPolicy {
    /// The launch geometry covers the domain exactly with no tail.
    Exact,
}

/// The reduction topology and combination legality of a schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReductionTopology {
    /// The region performs no reduction.
    None,
    /// The region reduces serially over the given axes.
    Serial {
        /// Reduced axes in canonical ascending order.
        axes: Vec<Axis>,
        /// Contributor combination order.
        order: ContributorOrder,
        /// Whether the contract permits reassociation.
        permits_reassociation: bool,
        /// Whether the contract permits contributor permutation.
        permits_permutation: bool,
    },
}

/// The symbolic launch geometry a schedule dispatches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaunchPlan {
    /// Total launched grid threads.
    pub grid_threads: u64,
    /// Threads per workgroup.
    pub threads_per_workgroup: u32,
    /// Whether a zero-work domain skips dispatch.
    pub zero_work_skips_dispatch: bool,
}

/// The normalized schedule that maps a region onto a target machine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelSchedule {
    /// Execution-to-iteration coordinate binding.
    pub binding: ExecutionBinding,
    /// Iteration work items covered by the launch.
    pub work_items: u64,
    /// Threads per workgroup.
    pub threads_per_workgroup: u32,
    /// Tail policy.
    pub tail: TailPolicy,
    /// Ownership witness the owning write must reference.
    pub output_owner: OwnershipWitnessId,
    /// Reduction topology.
    pub reduction: ReductionTopology,
    /// Launch geometry.
    pub launch: LaunchPlan,
}

/// A first-class scheduled region: a bounded index region plus its schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledRegion {
    /// The bounded index region the schedule refines.
    pub index: IndexRegion,
    /// The normalized kernel schedule.
    pub schedule: KernelSchedule,
}

/// Exact or proven resource requirements derived from a verified schedule.
///
/// These feed a separate phased target-feasibility assessment; deriving them is
/// part of intrinsic verification and never a target decision (ADR 0007).
///
/// The four numerical fields carry the region's declared realization forward
/// per dimension rather than as one summary bit. A single `requires_strict_f32`
/// boolean cannot name which dimension a target failed to honour, and the
/// boolean these replaced was derived from contraction and reassociation alone
/// — so a subnormal-preserving contract that permitted both transforms reported
/// no strict-`f32` requirement at all (ADR 0076 item 3). A feasibility
/// authority composes each dimension against what a target profile declares it
/// honours.
///
/// The realization's `profile_key` and canonical NaN bits are deliberately not
/// repeated here: they name the governing contract and a produced value rather
/// than a behaviour a target profile declares honourability for, and they
/// remain on the region's [`NumericalRealization`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceRequirements {
    /// Distinct buffer bindings required at the entry point.
    pub buffer_bindings: u32,
    /// Threads per workgroup required by the launch.
    pub threads_per_workgroup: u32,
    /// Local (threadgroup) memory bytes required.
    pub local_memory_bytes: u64,
    /// Execution barriers required.
    pub barriers: u32,
    /// Whether the region requires a device address space.
    pub requires_device_memory: bool,
    /// Subnormal input handling the region's declared realization requires.
    pub input_subnormals: SubnormalMode,
    /// Subnormal result handling the region's declared realization requires.
    pub result_subnormals: SubnormalMode,
    /// Whether the region's declared realization permits contraction.
    pub contraction: NumericalPermission,
    /// Whether the region's declared realization permits reassociation.
    pub reassociation: NumericalPermission,
}

/// Opaque canonical bytes identifying one verified scheduled region.
///
/// The identity is a pure function of the normalized schedule content and is
/// independent of the transient [`RegionId`] and of builder insertion order for
/// equivalent regions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScheduledRegionIdentity(Vec<u8>);

impl CanonicalScheduledRegionIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An immutable, intrinsically verified scheduled region.
///
/// Only [`super::ScheduledRegionBuilder::build`] produces one. It exposes
/// read-only meaning and never mutation, thawing, or unchecked construction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedScheduledRegion {
    region: ScheduledRegion,
    requirements: ResourceRequirements,
    identity: CanonicalScheduledRegionIdentity,
}

impl VerifiedScheduledRegion {
    pub(super) fn new(
        region: ScheduledRegion,
        requirements: ResourceRequirements,
        identity: CanonicalScheduledRegionIdentity,
    ) -> Self {
        Self {
            region,
            requirements,
            identity,
        }
    }

    /// Returns the normalized scheduled region.
    #[must_use]
    pub const fn region(&self) -> &ScheduledRegion {
        &self.region
    }

    /// Returns the derived resource requirements.
    #[must_use]
    pub const fn requirements(&self) -> ResourceRequirements {
        self.requirements
    }

    /// Returns the canonical structural identity.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalScheduledRegionIdentity {
        &self.identity
    }
}

/// Returns the element count of a shape, or `0` when any extent is `0`.
///
/// # Errors
///
/// Returns [`ElementCountOverflow`] when a nonzero extent product exceeds `u64`.
pub fn element_count(shape: &Shape) -> Result<u64, ElementCountOverflow> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape
        .extents()
        .iter()
        .try_fold(1_u64, |count, extent| count.checked_mul(extent.get()))
        .ok_or(ElementCountOverflow)
}

/// Returns whether `axes` is a strictly ascending in-range axis set.
#[must_use]
pub fn axes_are_canonical(axes: &[Axis], rank: usize) -> bool {
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

/// Counts the reduction contributors a reduction-contributor access combines.
///
/// Returns `0` when any reduced extent is `0` (an empty reduction).
///
/// # Errors
///
/// Returns a [`ContributorError`] when the access is not a reduction access,
/// the axes are not canonical, an axis is out of range, or the contributor
/// product overflows `u64`.
pub fn contributor_count(axes: &[Axis], access: &LogicalAccess) -> Result<u64, ContributorError> {
    let LogicalAccess::ReductionContributor { input_shape, .. } = access else {
        return Err(ContributorError::NotReductionAccess);
    };
    if !axes_are_canonical(axes, input_shape.rank()) {
        return Err(ContributorError::NonCanonicalAxes);
    }
    let extents = axes
        .iter()
        .map(|axis| {
            usize::try_from(axis.get())
                .ok()
                .and_then(|index| input_shape.extents().get(index))
                .map(|extent| extent.get())
                .ok_or(ContributorError::AxisOutOfRange)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if extents.contains(&0) {
        return Ok(0);
    }
    extents
        .into_iter()
        .try_fold(1_u64, u64::checked_mul)
        .ok_or(ContributorError::Overflow)
}

/// Derives the resource requirements of a verified region.
///
/// Bindings follow the region's access count; the launch fixes the thread
/// count; the bounded profile stages no local memory or barriers. The numerical
/// realization is carried forward whole rather than reduced to a predicate:
/// deriving one bit here would decide, inside intrinsic verification, which
/// dimensions a target is allowed to be asked about, and that decision belongs
/// to the feasibility authority that knows what the target declares.
pub(super) fn derive_requirements(region: &ScheduledRegion) -> ResourceRequirements {
    let buffer_bindings = u32::try_from(region.index.accesses.len()).unwrap_or(u32::MAX);
    ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup: region.schedule.threads_per_workgroup,
        local_memory_bytes: 0,
        barriers: 0,
        requires_device_memory: true,
        input_subnormals: region.index.numerical.input_subnormals,
        result_subnormals: region.index.numerical.result_subnormals,
        contraction: region.index.numerical.contraction,
        reassociation: region.index.numerical.reassociation,
    }
}

const TAG_LINEAR_IDENTITY: u8 = 0x01;
const TAG_REDUCTION_CONTRIBUTOR: u8 = 0x02;
const TAG_LINEAR_RANGE: u8 = 0x11;
const TAG_REDUCTION_DOMAIN: u8 = 0x12;
const TAG_SCALAR_MUL_ADD: u8 = 0x21;
const TAG_SCALAR_SERIAL_SUM: u8 = 0x22;
const TAG_SCALAR_FUSED_SUM: u8 = 0x23;
const TAG_REDUCTION_NONE: u8 = 0x31;
const TAG_REDUCTION_SERIAL: u8 = 0x32;

fn push_shape(bytes: &mut Vec<u8>, shape: &Shape) {
    bytes.extend_from_slice(&(shape.rank() as u64).to_be_bytes());
    for extent in shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn push_axes(bytes: &mut Vec<u8>, axes: &[Axis]) {
    bytes.extend_from_slice(&(axes.len() as u64).to_be_bytes());
    for axis in axes {
        bytes.extend_from_slice(&axis.get().to_be_bytes());
    }
}

fn push_order(bytes: &mut Vec<u8>, order: ContributorOrder) {
    let ContributorOrder::OriginalAxisLexicographic = order;
    bytes.push(0x01);
}

fn push_tensor_role(bytes: &mut Vec<u8>, role: TensorRole) {
    bytes.push(match role {
        TensorRole::Input => 0x01,
        TensorRole::Intermediate => 0x02,
        TensorRole::Output => 0x03,
    });
}

fn push_logical_access(bytes: &mut Vec<u8>, access: &LogicalAccess) {
    match access {
        LogicalAccess::LinearIdentity => bytes.push(TAG_LINEAR_IDENTITY),
        LogicalAccess::ReductionContributor {
            input_shape,
            output_shape,
            axes,
            order,
        } => {
            bytes.push(TAG_REDUCTION_CONTRIBUTOR);
            push_shape(bytes, input_shape);
            push_shape(bytes, output_shape);
            push_axes(bytes, axes);
            push_order(bytes, *order);
        }
    }
}

/// Encodes one subnormal dimension.
///
/// The match is exhaustive over a non-`#[non_exhaustive]` enum, so widening the
/// vocabulary is a build error here rather than an identity collision between
/// two regions that differ only in subnormal treatment (ADR 0076 item 6). The
/// flush arm encodes its zero sign, because the sign is part of the behaviour
/// and two flushes producing different zeros are different realizations.
fn push_subnormal(bytes: &mut Vec<u8>, mode: SubnormalMode) {
    bytes.push(match mode {
        SubnormalMode::Preserve => 0x01,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        } => 0x02,
        SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        } => 0x03,
    });
}

/// Encodes one transform permission.
///
/// Encoded as a tagged value rather than as the derived `permits_*` boolean it
/// used to be: a boolean is a projection, and a projection cannot fail closed
/// when the projected enum grows.
fn push_permission(bytes: &mut Vec<u8>, permission: NumericalPermission) {
    bytes.push(match permission {
        NumericalPermission::Forbidden => 0x01,
        NumericalPermission::Permitted => 0x02,
    });
}

/// Encodes the complete numerical realization a region declares.
///
/// Every field is encoded, including both subnormal dimensions. `profile_key`
/// is encoded alongside them and never in place of them: a key names a contract
/// but does not carry its field values, so relying on the key to distinguish
/// two realizations would be an unstated invariant (ADR 0076 item 6).
fn push_numerical(bytes: &mut Vec<u8>, numerical: &NumericalRealization) {
    bytes.extend_from_slice(numerical.profile_key.as_bytes());
    bytes.push(0x00);
    bytes.extend_from_slice(&numerical.canonical_arithmetic_nan_bits.to_be_bytes());
    push_subnormal(bytes, numerical.input_subnormals);
    push_subnormal(bytes, numerical.result_subnormals);
    push_permission(bytes, numerical.contraction);
    push_permission(bytes, numerical.reassociation);
}

fn push_scalar_program(bytes: &mut Vec<u8>, program: &ScalarProgram) {
    match program {
        ScalarProgram::MultiplyThenAdd {
            scale_bits,
            bias_bits,
            canonical_nan_bits,
            contraction,
        } => {
            bytes.push(TAG_SCALAR_MUL_ADD);
            bytes.extend_from_slice(&scale_bits.to_be_bytes());
            bytes.extend_from_slice(&bias_bits.to_be_bytes());
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.push(u8::from(*contraction));
        }
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
        } => {
            bytes.push(TAG_SCALAR_SERIAL_SUM);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
        }
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits,
            bias_bits,
            axes,
            order,
            canonical_nan_bits,
            empty_identity_bits,
            contraction,
        } => {
            bytes.push(TAG_SCALAR_FUSED_SUM);
            bytes.extend_from_slice(&scale_bits.to_be_bytes());
            bytes.extend_from_slice(&bias_bits.to_be_bytes());
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.extend_from_slice(&canonical_nan_bits.to_be_bytes());
            bytes.extend_from_slice(&empty_identity_bits.to_be_bytes());
            bytes.push(u8::from(*contraction));
        }
    }
}

fn push_access(bytes: &mut Vec<u8>, access: &Access) {
    push_tensor_role(bytes, access.tensor);
    bytes.push(match access.mode {
        AccessMode::Read => 0x01,
        AccessMode::Write => 0x02,
    });
    push_logical_access(bytes, &access.map);
    bytes.extend_from_slice(&access.bounds.get().to_be_bytes());
    match access.ownership {
        None => bytes.push(0x00),
        Some(owner) => {
            bytes.push(0x01);
            bytes.extend_from_slice(&owner.get().to_be_bytes());
        }
    }
}

fn push_bounds_proof(bytes: &mut Vec<u8>, proof: &BoundsProof) {
    bytes.extend_from_slice(&proof.id.get().to_be_bytes());
    push_tensor_role(bytes, proof.tensor);
    match &proof.kind {
        BoundsProofKind::LinearRange { element_count } => {
            bytes.push(TAG_LINEAR_RANGE);
            bytes.extend_from_slice(&element_count.to_be_bytes());
        }
        BoundsProofKind::ReductionDomain {
            input_shape,
            output_shape,
            axes,
            order,
        } => {
            bytes.push(TAG_REDUCTION_DOMAIN);
            push_shape(bytes, input_shape);
            push_shape(bytes, output_shape);
            push_axes(bytes, axes);
            push_order(bytes, *order);
        }
    }
}

fn push_schedule(bytes: &mut Vec<u8>, schedule: &KernelSchedule) {
    let ExecutionBinding::GlobalLinearInvocation = schedule.binding;
    bytes.push(0x01);
    bytes.extend_from_slice(&schedule.work_items.to_be_bytes());
    bytes.extend_from_slice(&schedule.threads_per_workgroup.to_be_bytes());
    let TailPolicy::Exact = schedule.tail;
    bytes.push(0x01);
    bytes.extend_from_slice(&schedule.output_owner.get().to_be_bytes());
    match &schedule.reduction {
        ReductionTopology::None => bytes.push(TAG_REDUCTION_NONE),
        ReductionTopology::Serial {
            axes,
            order,
            permits_reassociation,
            permits_permutation,
        } => {
            bytes.push(TAG_REDUCTION_SERIAL);
            push_axes(bytes, axes);
            push_order(bytes, *order);
            bytes.push(u8::from(*permits_reassociation));
            bytes.push(u8::from(*permits_permutation));
        }
    }
    bytes.extend_from_slice(&schedule.launch.grid_threads.to_be_bytes());
    bytes.extend_from_slice(&schedule.launch.threads_per_workgroup.to_be_bytes());
    bytes.push(u8::from(schedule.launch.zero_work_skips_dispatch));
}

/// Encodes the canonical identity of a normalized scheduled region.
///
/// The encoding excludes the transient [`RegionId`] so equivalent normalized
/// schedules produced by different planning histories share identity.
pub(super) fn encode_identity(region: &ScheduledRegion) -> CanonicalScheduledRegionIdentity {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.schedule.v1");
    push_shape(&mut bytes, &region.index.iteration_shape);
    bytes.extend_from_slice(&(region.index.accesses.len() as u64).to_be_bytes());
    for access in &region.index.accesses {
        push_access(&mut bytes, access);
    }
    bytes.extend_from_slice(&(region.index.bounds_proofs.len() as u64).to_be_bytes());
    for proof in &region.index.bounds_proofs {
        push_bounds_proof(&mut bytes, proof);
    }
    bytes.extend_from_slice(&region.index.ownership_proof.id.get().to_be_bytes());
    push_tensor_role(&mut bytes, region.index.ownership_proof.tensor);
    let OwnershipProofKind::OneGlobalInvocationPerOutput { output_count } =
        region.index.ownership_proof.kind;
    bytes.extend_from_slice(&output_count.to_be_bytes());
    push_scalar_program(&mut bytes, &region.index.scalar_program);
    push_numerical(&mut bytes, &region.index.numerical);
    push_schedule(&mut bytes, &region.schedule);
    CanonicalScheduledRegionIdentity(bytes)
}
