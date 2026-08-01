//! Transactional builder and intrinsic verifier for scheduled regions.
//!
//! Construction follows the ADR 0071 discipline: insertions check local
//! invariants, and the consuming [`ScheduledRegionBuilder::build`] runs
//! whole-region intrinsic verification before returning an opaque
//! [`VerifiedScheduledRegion`]. The verifier proves domain coverage, output
//! ownership and race freedom, tail and launch legality, bounds-proof
//! refinement, reduction contributor and order legality, numerical/access
//! agreement, and zero-domain behaviour. No later cost or feasibility query can
//! repair a schedule this verifier rejects.

use super::cooperative::{
    ContributorArrival, CooperativeTile, ParticipantRange, StagedSpan, VisibilityEdge,
};
use super::error::{
    CooperativeTileRule, ScheduleBuildError, ScheduleComponent, ScheduleLimitKind,
    ScheduledRegionBuildError, ScheduledRegionDiagnostic,
};
use super::handles::{InputOrdinal, RegionId, StagingId};
use super::model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContractionAxisSource, ExecutionBinding, IndexRegion, KernelSchedule, LogicalAccess,
    OwnershipProof, OwnershipProofKind, ReductionPass, ReductionTopology, ResourceRequirements,
    ScalarProgram, ScheduledRegion, TailPolicy, TensorRole, VerifiedScheduledRegion,
    contributor_count, cooperative_tile, derive_requirements, element_count, encode_identity,
    partial_reduction_axis, partial_reduction_shape,
};
use super::numerics::{ArithmeticType, ExceptionalValueAssumption, NumericalRealization};
use super::synchronization::{ConvergenceEvidence, SynchronizationRule, required_subject};
use super::{
    MAX_COOPERATIVE_PARTICIPANTS, MAX_COOPERATIVE_PHASE_ACCESSES, MAX_COOPERATIVE_PHASES,
    MAX_COOPERATIVE_STAGING_SLOTS, MAX_COOPERATIVE_SYNCHRONIZATION_POINTS, MAX_SCHEDULE_ACCESSES,
    MAX_SCHEDULE_BOUNDS_PROOFS,
};
use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};

/// A transactional scheduled-region builder with private storage.
///
/// Accumulate the iteration domain, logical accesses, proofs, scalar program,
/// numerical realization, and kernel schedule, then call
/// [`ScheduledRegionBuilder::build`] to verify and freeze the region.
#[derive(Clone, Debug)]
pub struct ScheduledRegionBuilder {
    id: RegionId,
    iteration_shape: Option<crate::shape::Shape>,
    accesses: Vec<Access>,
    bounds_proofs: Vec<BoundsProof>,
    ownership_proof: Option<OwnershipProof>,
    scalar_program: Option<ScalarProgram>,
    numerical: Option<NumericalRealization>,
    schedule: Option<KernelSchedule>,
}

impl ScheduledRegionBuilder {
    /// Opens a fresh builder for the given planning ordinal.
    #[must_use]
    pub fn new(id: RegionId) -> Self {
        Self {
            id,
            iteration_shape: None,
            accesses: Vec::new(),
            bounds_proofs: Vec::new(),
            ownership_proof: None,
            scalar_program: None,
            numerical: None,
            schedule: None,
        }
    }

    /// Seeds a builder from an already-assembled region.
    ///
    /// This convenience delegates to the same insertion and verification path;
    /// it does not bypass [`ScheduledRegionBuilder::build`].
    #[must_use]
    pub fn from_region(region: ScheduledRegion) -> Self {
        let ScheduledRegion { index, schedule } = region;
        Self {
            id: index.id,
            iteration_shape: Some(index.iteration_shape),
            accesses: index.accesses,
            bounds_proofs: index.bounds_proofs,
            ownership_proof: Some(index.ownership_proof),
            scalar_program: Some(index.scalar_program),
            numerical: Some(index.numerical),
            schedule: Some(schedule),
        }
    }

    /// Sets the parallel iteration domain.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn iteration_shape(
        &mut self,
        shape: crate::shape::Shape,
    ) -> Result<(), ScheduleBuildError> {
        set_once(
            &mut self.iteration_shape,
            shape,
            ScheduleComponent::IterationShape,
        )
    }

    /// Appends one logical access.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::StructuralLimit`] when the access limit is
    /// exceeded.
    pub fn push_access(&mut self, access: Access) -> Result<(), ScheduleBuildError> {
        push_bounded(
            &mut self.accesses,
            access,
            ScheduleLimitKind::Accesses,
            MAX_SCHEDULE_ACCESSES,
        )
    }

    /// Appends one bounds proof.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::StructuralLimit`] when the bounds-proof
    /// limit is exceeded.
    pub fn push_bounds_proof(&mut self, proof: BoundsProof) -> Result<(), ScheduleBuildError> {
        push_bounded(
            &mut self.bounds_proofs,
            proof,
            ScheduleLimitKind::BoundsProofs,
            MAX_SCHEDULE_BOUNDS_PROOFS,
        )
    }

    /// Sets the single write-ownership proof.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn ownership_proof(&mut self, proof: OwnershipProof) -> Result<(), ScheduleBuildError> {
        set_once(
            &mut self.ownership_proof,
            proof,
            ScheduleComponent::OwnershipProof,
        )
    }

    /// Sets the scalar program.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn scalar_program(&mut self, program: ScalarProgram) -> Result<(), ScheduleBuildError> {
        set_once(
            &mut self.scalar_program,
            program,
            ScheduleComponent::ScalarProgram,
        )
    }

    /// Sets the preserved numerical realization.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn numerical(&mut self, numerical: NumericalRealization) -> Result<(), ScheduleBuildError> {
        set_once(
            &mut self.numerical,
            numerical,
            ScheduleComponent::NumericalRealization,
        )
    }

    /// Sets the normalized kernel schedule.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn schedule(&mut self, schedule: KernelSchedule) -> Result<(), ScheduleBuildError> {
        set_once(
            &mut self.schedule,
            schedule,
            ScheduleComponent::KernelSchedule,
        )
    }

    /// Verifies the whole region and freezes it, or returns the intact builder.
    ///
    /// # Errors
    ///
    /// Returns a [`ScheduledRegionBuildError`] carrying every intrinsic
    /// diagnostic and the recoverable builder when verification fails.
    pub fn build(self) -> Result<VerifiedScheduledRegion, ScheduledRegionBuildError> {
        match self.assemble_and_verify() {
            Ok((region, requirements, identity)) => {
                Ok(VerifiedScheduledRegion::new(region, requirements, identity))
            }
            Err(diagnostics) => Err(ScheduledRegionBuildError {
                builder: Box::new(self),
                diagnostics,
            }),
        }
    }

    fn assemble_and_verify(
        &self,
    ) -> Result<
        (
            ScheduledRegion,
            ResourceRequirements,
            CanonicalScheduledRegionIdentity,
        ),
        Vec<ScheduledRegionDiagnostic>,
    > {
        let region = self.assemble().map_err(|diagnostic| vec![diagnostic])?;
        verify_intrinsic(&region).map_err(|diagnostic| vec![diagnostic])?;
        let requirements = derive_requirements(&region);
        let identity = encode_identity(&region);
        Ok((region, requirements, identity))
    }

    fn assemble(&self) -> Result<ScheduledRegion, ScheduledRegionDiagnostic> {
        let iteration_shape = self
            .iteration_shape
            .clone()
            .ok_or(incomplete(ScheduleComponent::IterationShape))?;
        let ownership_proof = self
            .ownership_proof
            .ok_or(incomplete(ScheduleComponent::OwnershipProof))?;
        let scalar_program = self
            .scalar_program
            .clone()
            .ok_or(incomplete(ScheduleComponent::ScalarProgram))?;
        let numerical = self
            .numerical
            .ok_or(incomplete(ScheduleComponent::NumericalRealization))?;
        let schedule = self
            .schedule
            .clone()
            .ok_or(incomplete(ScheduleComponent::KernelSchedule))?;
        Ok(ScheduledRegion {
            index: IndexRegion {
                id: self.id,
                iteration_shape,
                accesses: self.accesses.clone(),
                bounds_proofs: self.bounds_proofs.clone(),
                ownership_proof,
                scalar_program,
                numerical,
            },
            schedule,
        })
    }
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    component: ScheduleComponent,
) -> Result<(), ScheduleBuildError> {
    if slot.is_some() {
        return Err(ScheduleBuildError::ComponentAlreadySet { component });
    }
    *slot = Some(value);
    Ok(())
}

fn push_bounded<T>(
    storage: &mut Vec<T>,
    value: T,
    resource: ScheduleLimitKind,
    limit: usize,
) -> Result<(), ScheduleBuildError> {
    if storage.len() >= limit {
        return Err(ScheduleBuildError::StructuralLimit {
            resource,
            actual: storage.len() + 1,
            limit,
        });
    }
    storage.push(value);
    Ok(())
}

const fn incomplete(component: ScheduleComponent) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::IncompleteRegion { component }
}

/// The boundary role of a region that reads exactly one program input tensor.
///
/// Named once rather than spelled at each site because these families are
/// *defined* by reading a single tensor: the strict-affine dequantize reads three
/// components of one encoded tensor, and the reduction families read one
/// contributor domain. A second input tensor in either is a different scalar
/// program, not a wider spelling of these.
const FIRST_INPUT: TensorRole = TensorRole::Input {
    ordinal: InputOrdinal::FIRST,
};

/// Runs the intrinsic schedule verifier over an assembled region.
fn verify_intrinsic(region: &ScheduledRegion) -> Result<(), ScheduledRegionDiagnostic> {
    let iteration_count = element_count(&region.index.iteration_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    let schedule = &region.schedule;
    if schedule.binding != ExecutionBinding::GlobalLinearInvocation
        || schedule.tail != TailPolicy::Exact
        || schedule.work_items != iteration_count
        || schedule.launch.grid_threads != iteration_count
        || schedule.launch.threads_per_workgroup != schedule.threads_per_workgroup
        || schedule.threads_per_workgroup == 0
        || !schedule.launch.zero_work_skips_dispatch
    {
        return Err(ScheduledRegionDiagnostic::LaunchCoverage);
    }
    match &region.index.scalar_program {
        ScalarProgram::StrictAffineU4Dequantize { .. } => {
            let [codes, scale, zero_point, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_strict_affine_u4_dequantize(region, codes, scale, zero_point, write)
        }
        // A pointwise region reads one boundary tensor per expression leaf, so
        // its access count is a property of its scalar program rather than a
        // constant. The reduction families still read exactly one tensor: their
        // multi-input stories are separately owned, and a second read here would
        // leave the contributor domain unable to say which access it counts.
        ScalarProgram::PointwiseF32(expression) => {
            let Some((write, reads)) = region.index.accesses.split_last() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_pointwise_f32(region, expression, reads, write)
        }
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::StrictSerialMaximum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            let [read, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_access_and_semantics(region, read, write)
        }
        // A contraction reads exactly two operands. That count is the family's
        // definition and not a bound this profile happens to impose: ADR 0087's
        // fifth structural rule refuses an index shared by more than two
        // operands, so a third read would be an occurrence the semantic registry
        // already refused.
        ScalarProgram::StrictTensorContraction { .. } => {
            let [left, right, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_contraction(region, left, right, write)
        }
    }
}

/// Verifies a two-operand strict tensor contraction region.
///
/// Every obligation is stated against the region's *own* three declarations of
/// the contracted space — the scalar program's, the schedule topology's, and
/// each operand access's — and they are required to agree. A producer that
/// stated one of them differently would otherwise fold a different number of
/// contributors than it addressed.
fn verify_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = &region.index.scalar_program
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let ReductionTopology::Contraction {
        contracted_shape: scheduled_contracted,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = &region.index.numerical;
    if contracted_shape != scheduled_contracted
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The one precondition this realization has. The registered family declares
    // an empty contracted domain refused rather than identity-valued, so a
    // contracted space with no points has no result to commit — and a rank-zero
    // contracted shape has one point, not none, so the check is on the element
    // count rather than on the rank.
    let contracted_points = element_count(contracted_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    if contracted_points == 0 {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.mode != AccessMode::Read
        || right.mode != AccessMode::Read
        || left.ownership.is_some()
        || right.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    // The two operands bind the program's first two input tensors positionally,
    // in the order the structure names them. Without this a region could read
    // one tensor twice while the second buffer went unread, and every consumer
    // that binds buffers positionally would bind the wrong one.
    if left.tensor
        != (TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        })
        || right.tensor
            != (TensorRole::Input {
                ordinal: InputOrdinal::new(1),
            })
        || left.component_role.is_some()
        || right.component_role.is_some()
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[left, right], write)?;

    let mut contracted_covered = vec![false; contracted_shape.rank()];
    let mut output_covered = vec![false; region.index.iteration_shape.rank()];
    for access in [left, right] {
        let LogicalAccess::ContractionOperand {
            operand_shape,
            output_shape,
            contracted_shape: access_contracted,
            sources,
            order: access_order,
        } = &access.map
        else {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        };
        if output_shape != &region.index.iteration_shape
            || access_contracted != contracted_shape
            || access_order != order
            || sources.len() != operand_shape.rank()
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        // Every operand axis names one in-range coordinate whose extent it
        // agrees with, and no two axes of one operand name the same coordinate.
        // Extent agreement is what makes the row-major linearization stay inside
        // the operand, which is the whole content of its bounds proof.
        let mut seen_output = vec![false; output_shape.rank()];
        let mut seen_contracted = vec![false; contracted_shape.rank()];
        for (axis, source) in sources.iter().enumerate() {
            let (shape, seen, covered) = match source {
                ContractionAxisSource::Output { .. } => {
                    (output_shape, &mut seen_output, &mut output_covered)
                }
                ContractionAxisSource::Contracted { .. } => (
                    contracted_shape,
                    &mut seen_contracted,
                    &mut contracted_covered,
                ),
            };
            let position = match source {
                ContractionAxisSource::Output { position }
                | ContractionAxisSource::Contracted { position } => usize::try_from(*position)
                    .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?,
            };
            let (Some(extent), Some(slot)) =
                (shape.extents().get(position), seen.get_mut(position))
            else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            if std::mem::replace(slot, true) || operand_shape.extents()[axis] != *extent {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            }
            covered[position] = true;
        }
        // A contracted coordinate this operand does not read would make the
        // operand invariant in it — an outer product summed over a free index,
        // not a contraction. ADR 0087's second rule refuses exactly that
        // structure, and this is where a region claiming one is caught.
        if seen_contracted.iter().any(|read| !read) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
    }
    // Every output coordinate must be read by at least one operand: one that no
    // operand reads would make every output position along it hold the same
    // value, which is a broadcast the structure never declared.
    if output_covered.iter().any(|read| !read) || contracted_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies an N-input physical `f32` pointwise region.
///
/// The obligation that makes the widening safe is the *correspondence*: read
/// access `i` must be `TensorRole::Input { ordinal: i }`, and there must be
/// exactly as many reads as the expression has input leaves. Both halves are
/// needed. Without the count, an expression could read an ordinal no access
/// binds — a load through a buffer the signature never declares. Without the
/// per-position role, two accesses could name one tensor while a third went
/// unread, and every consumer that binds buffers positionally would bind the
/// wrong one without noticing.
///
/// The expression's own verifier already proved its ordinals are the dense
/// `0..n`, so pairing by position is exhaustive rather than a sample.
fn verify_pointwise_f32(
    region: &ScheduledRegion,
    expression: &super::pointwise::PointwiseF32Expression,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if reads.is_empty() || reads.len() != expression.input_count() {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    }
    if reads
        .iter()
        .any(|read| read.mode != AccessMode::Read || read.ownership.is_some())
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    let read_refs: Vec<&Access> = reads.iter().collect();
    verify_proof_records(region, &read_refs, write)?;
    let ordinals_bind_in_order = reads.iter().enumerate().all(|(position, read)| {
        u32::try_from(position).is_ok_and(|ordinal| {
            read.tensor
                == TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                }
        })
    });
    if !expression.is_valid()
        || !matches!(region.schedule.reduction, ReductionTopology::None)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || !ordinals_bind_in_order
        || reads
            .iter()
            .any(|read| read.map != LogicalAccess::LinearIdentity)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

fn verify_strict_affine_u4_dequantize(
    region: &ScheduledRegion,
    codes: &Access,
    scale: &Access,
    zero_point: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ScalarProgram::StrictAffineU4Dequantize {
        codes_role,
        scale_role,
        zero_point_role,
    } = &region.index.scalar_program
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = region.index.numerical;
    if *codes_role != STRICT_AFFINE_CODES_ROLE
        || *scale_role != STRICT_AFFINE_SCALE_ROLE
        || *zero_point_role != STRICT_AFFINE_ZERO_POINT_ROLE
        || codes.tensor != FIRST_INPUT
        || codes.component_role != Some(*codes_role)
        || codes.mode != AccessMode::Read
        || codes.ownership.is_some()
        || codes.map
            != (LogicalAccess::PackedU4LsbZeroTail {
                logical_elements: region.schedule.work_items,
            })
        || scale.tensor != FIRST_INPUT
        || scale.component_role != Some(*scale_role)
        || scale.mode != AccessMode::Read
        || scale.ownership.is_some()
        || scale.map != LogicalAccess::ScalarBroadcast
        || zero_point.tensor != FIRST_INPUT
        || zero_point.component_role != Some(*zero_point_role)
        || zero_point.mode != AccessMode::Read
        || zero_point.ownership.is_some()
        || zero_point.map != LogicalAccess::ScalarBroadcast
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || write.component_role.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
        || !matches!(region.schedule.reduction, ReductionTopology::None)
        || numerical.input_subnormals != super::SubnormalMode::Preserve
        || numerical.result_subnormals != super::SubnormalMode::Preserve
        || numerical.permits_contraction()
        || numerical.permits_reassociation()
        || numerical.permits_permutation()
        || numerical.permits_signed_zero_elimination()
        || numerical.nan_assumptions != ExceptionalValueAssumption::MakeNoAssumption
        || numerical.infinity_assumptions != ExceptionalValueAssumption::MakeNoAssumption
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[codes, scale, zero_point], write)
}

fn verify_access_and_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if read.mode != AccessMode::Read
        || read.ownership.is_some()
        || write.mode != AccessMode::Write
        || write.map != LogicalAccess::LinearIdentity
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    verify_proof_records(region, &[read], write)?;
    if matches!(
        region.schedule.reduction,
        ReductionTopology::MultiPass { .. }
    ) {
        return verify_multi_pass_semantics(region, read, write);
    }
    if matches!(
        region.schedule.reduction,
        ReductionTopology::CooperativeWorkgroup { .. }
    ) {
        return verify_cooperative_semantics(region, read, write);
    }
    let numerical = &region.index.numerical;
    match (
        &region.index.scalar_program,
        &region.schedule.reduction,
        &read.map,
    ) {
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
            && *permits_reassociation == numerical.permits_reassociation()
            && *permits_permutation == numerical.permits_permutation()
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
            && *permits_reassociation == numerical.permits_reassociation()
            && *permits_permutation == numerical.permits_permutation()
            && !contraction
            && *empty_identity_bits == 0.0_f32.to_bits()
            && output_shape == &region.index.iteration_shape
            && input_shape.without_axes(axes) == *output_shape
            && read.tensor == FIRST_INPUT
            && write.tensor == TensorRole::Output => {}
        // The squaring prologue reads the *original* input, exactly as the
        // scale-bias one does, so its read binds the first input tensor rather
        // than an intermediate. Its obligations are otherwise the strict serial
        // sum's, because it is that reduction over an elementwise prologue and
        // not a second reducer.
        (
            ScalarProgram::SquaredSerialSum {
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
            && *permits_reassociation == numerical.permits_reassociation()
            && *permits_permutation == numerical.permits_permutation()
            && *empty_identity_bits == 0.0_f32.to_bits()
            && output_shape == &region.index.iteration_shape
            && input_shape.without_axes(axes) == *output_shape
            && read.tensor == FIRST_INPUT
            && write.tensor == TensorRole::Output => {}
        // The extrema fold. Every obligation the sums carry is carried here too
        // *except* the empty-domain identity, which this family has no field for
        // and no correct value of — the non-emptiness check below replaces it.
        // Its read binds the first input tensor, because `tiler::softmax-f32@1`'s
        // maximum pass reads the original scores rather than an intermediate.
        //
        // The two order permissions are still required to agree with the
        // realization, and that is deliberate rather than an oversight: this fold
        // is order-*insensitive*, so the permissions do not constrain what a
        // schedule may do to it — but the region still declares them, and a
        // declaration that disagreed with its own realization would be
        // incoherent whatever the fold's legality.
        (
            ScalarProgram::StrictSerialMaximum { axes, order, .. },
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
            && *permits_reassociation == numerical.permits_reassociation()
            && *permits_permutation == numerical.permits_permutation()
            && output_shape == &region.index.iteration_shape
            && input_shape.without_axes(axes) == *output_shape
            && read.tensor == FIRST_INPUT
            && write.tensor == TensorRole::Output => {}
        _ => return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
    }
    // The one precondition the extrema fold has and no sum does. The family is
    // identity-less, so a reduced domain with no contributors has no value the
    // region could commit — the same shape of refusal the contraction states for
    // its unseeded fold, checked on the contributor count rather than on the rank
    // because a rank-zero reduced domain has one contributor, not none.
    if let ScalarProgram::StrictSerialMaximum { axes, .. } = &region.index.scalar_program {
        let contributors = contributor_count(axes, &read.map)
            .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
        if contributors == 0 {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
    }
    Ok(())
}

/// Verifies one pass of a split, multi-dispatch reduction.
///
/// The two passes are checked together here rather than as two more arms of the
/// serial match because every obligation they carry is stated relative to the
/// same [`ContributorPartition`]: the partial pass proves the split covers its
/// contributor sequence exactly, and the final pass proves it combines exactly
/// one contributor per partition of that same split. Splitting them across
/// unrelated arms would let one pass be verified against a partition the other
/// never agreed to.
fn verify_multi_pass_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::MultiPass {
        pass,
        partition,
        axes: scheduled_axes,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = &region.index.numerical;
    // Reassociation is what the split consumes, and it is checked on its own:
    // contributor order is preserved by construction, so a permitted
    // permutation neither grants nor substitutes for this permission.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || !*permits_reassociation
        // The bounded profile combines at the element width. A narrower
        // accumulation is a different computation and is refused rather than
        // silently accepted as equivalent.
        || *accumulation != ArithmeticType::F32
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    // A fused prologue belongs to the pass that reads the original inputs. The
    // final pass reads partial sums, so re-applying scale and bias there would
    // scale each partial a second time.
    let (axes, order, empty_identity_bits, read_tensor) = match (&region.index.scalar_program, pass)
    {
        (
            ScalarProgram::StrictSerialSum {
                axes,
                order,
                empty_identity_bits,
                ..
            },
            ReductionPass::Partial | ReductionPass::Final,
        ) => (axes, order, *empty_identity_bits, TensorRole::Intermediate),
        (
            ScalarProgram::FusedMultiplyAddSerialSum {
                axes,
                order,
                empty_identity_bits,
                contraction,
                ..
            },
            ReductionPass::Partial,
        ) if !contraction => (axes, order, *empty_identity_bits, FIRST_INPUT),
        // The squaring prologue likewise belongs to the pass that reads the
        // original inputs: squaring a partial sum in the final pass would square
        // an already-folded value.
        (
            ScalarProgram::SquaredSerialSum {
                axes,
                order,
                empty_identity_bits,
                ..
            },
            ReductionPass::Partial,
        ) => (axes, order, *empty_identity_bits, FIRST_INPUT),
        _ => return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
    };

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if axes != scheduled_axes
        || axes != access_axes
        || order != scheduled_order
        || order != access_order
        || empty_identity_bits != 0.0_f32.to_bits()
        || input_shape.without_axes(axes) != *output_shape
        || read.tensor != read_tensor
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let partial_shape = partial_reduction_shape(output_shape, *partition)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;

    let admitted = match pass {
        // The partial pass proves the split covers its own contributor
        // sequence exactly once each, and stages one partial per partition.
        ReductionPass::Partial => {
            partition.covers(contributors)
                && region.index.iteration_shape == partial_shape
                && write.tensor == TensorRole::Intermediate
        }
        // The final pass proves it combines exactly one contributor per
        // partition of that same split, reading the staged partial tensor.
        ReductionPass::Final => {
            partial_reduction_axis(output_shape)
                .is_some_and(|axis| axes.as_slice() == [axis].as_slice())
                && *input_shape == partial_shape
                && contributors == partition.partitions
                && region.index.iteration_shape == *output_shape
                && write.tensor == TensorRole::Output
        }
    };
    if admitted {
        Ok(())
    } else {
        Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)
    }
}

/// Verifies one cooperative workgroup reduction tile and its split.
///
/// Two obligations that stay apart. The *semantic* half proves the tile realizes
/// the region's declared reduction: the split covers the contributor sequence
/// exactly once each, the participants are the partitions, the iteration domain
/// runs one invocation per (output, participant) pair, and the reassociation the
/// split performs is one the contract permits. The *dataflow* half, in
/// [`verify_cooperative_tile`], proves the staging itself is well formed
/// independently of what is being reduced.
///
/// Neither half authorizes the handoff. The tile's visibility edges are
/// recorded, never discharged: the structured-kernel verifier refuses a kernel
/// whose region carries one, so a well-formed tile is a representation of a
/// dataflow rather than a claim that any target can run it.
fn verify_cooperative_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::CooperativeWorkgroup {
        partition,
        tile,
        axes: scheduled_axes,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
        arrival,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = &region.index.numerical;
    // Reassociation is what a split consumes, exactly as it is for a multi-pass
    // one, and it is required rather than merely recorded: a contract that
    // forbids reassociation forbids this strategy outright.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || !*permits_reassociation
        || *accumulation != ArithmeticType::F32
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The second permission, checked on its own and only where the arrival
    // actually consumes it. The order matters: a permitted-but-unrealizable
    // arrival must reach the admission rule below rather than be reported as a
    // numerical refusal, and an unpermitted one must name the permission rather
    // than the construct.
    if arrival.requires_permutation() && !numerical.permits_permutation() {
        return Err(cooperative(CooperativeTileRule::ArrivalPermission));
    }
    // The combining level folds the staged slots in ascending order through one
    // serial loop, which is the only arrival this vocabulary can order: the
    // constructs the others need are unadmitted synchronization kinds.
    if *arrival != ContributorArrival::AscendingParticipant {
        return Err(cooperative(CooperativeTileRule::UnadmittedArrival));
    }

    // A cooperative tile reads the original inputs and commits the reduction's
    // own output in one dispatch, so every family whose prologue belongs to the
    // pass that reads the inputs is admissible — there is no later pass here for
    // a prologue to be applied twice in.
    let (axes, order, empty_identity_bits, read_tensor) = match &region.index.scalar_program {
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => (axes, order, *empty_identity_bits, TensorRole::Intermediate),
        ScalarProgram::FusedMultiplyAddSerialSum {
            axes,
            order,
            empty_identity_bits,
            contraction,
            ..
        } if !contraction => (axes, order, *empty_identity_bits, FIRST_INPUT),
        ScalarProgram::SquaredSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => (axes, order, *empty_identity_bits, FIRST_INPUT),
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. }
        // The extrema fold is refused here structurally rather than by policy:
        // this destructuring binds an `empty_identity_bits` every admitted
        // program must carry, and the identity-less family has no such field. A
        // cooperative extrema tile is legal in principle — the fold is
        // order-insensitive — and admitting one needs a staged-partial contract
        // that does not rest on an identity, which is the reduction-strategies
        // work rather than a widening this variant may assume.
        | ScalarProgram::StrictSerialMaximum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
    };

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if axes != scheduled_axes
        || axes != access_axes
        || order != scheduled_order
        || order != access_order
        // The strict sum's empty result is `+0.0`, and every family here shares
        // it. Required at the same place the serial and multi-pass arms require
        // it, so a tile cannot introduce a second empty-domain answer.
        || empty_identity_bits != 0.0_f32.to_bits()
        || input_shape.without_axes(axes) != *output_shape
        || read.tensor != read_tensor
        || write.tensor != TensorRole::Output
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    // An empty contributor domain commits the declared identity from one
    // invocation with no fold, which is what the serial topology already does
    // and what needs no staging at all. A tile over it would declare a
    // visibility edge for values no participant produces.
    if contributors == 0 {
        return Err(cooperative(CooperativeTileRule::EmptyContributorDomain));
    }
    // The iteration domain is the output shape with one trailing participant
    // axis — the same layout a partial pass uses, and for the same reason: it
    // makes the participant ordinal the innermost coordinate of the invocation
    // index, so a participant's local coordinate is derivable without a second
    // layout rule.
    let participants = tile.coordinates.participants.count;
    if !partition.covers(contributors)
        || partition.partitions != participants
        || partial_reduction_shape(output_shape, *partition)
            .is_none_or(|shape| region.index.iteration_shape != shape)
    {
        return Err(cooperative(CooperativeTileRule::ContributorSplit));
    }
    verify_cooperative_tile(region, tile)
}

/// Verifies one cooperative tile's cross-invocation dataflow.
///
/// Every rule here is decided by enumerating the slots each participant
/// addresses, which is why the governed bounds are checked first: enumeration is
/// what makes disjointness and coverage exact instead of a modular argument, and
/// an unbounded tile would make it unbounded work.
fn verify_cooperative_tile(
    region: &ScheduledRegion,
    tile: &CooperativeTile,
) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = tile.coordinates.participants;
    if participants.count > MAX_COOPERATIVE_PARTICIPANTS
        || tile.phases.len() > MAX_COOPERATIVE_PHASES
        || tile
            .phases
            .iter()
            .any(|phase| phase.writes.len() > MAX_COOPERATIVE_PHASE_ACCESSES)
        || tile
            .phases
            .iter()
            .any(|phase| phase.reads.len() > MAX_COOPERATIVE_PHASE_ACCESSES)
        || tile
            .staging
            .iter()
            .try_fold(0_u64, |total, staging| total.checked_add(staging.slots))
            .is_none_or(|slots| slots > MAX_COOPERATIVE_STAGING_SLOTS)
    {
        return Err(cooperative(CooperativeTileRule::StructuralLimit));
    }
    // Local coordinates are the dense run the participants occupy. A tile whose
    // run does not start at zero leaves the workgroup's lower invocations with
    // no coordinate, so nothing could say what they execute.
    if participants.first != 0 || participants.count == 0 || participants.end().is_none() {
        return Err(cooperative(CooperativeTileRule::LocalCoordinates));
    }
    // Uniform convergence. Every launched invocation of the workgroup is a
    // participant, so a synchronization point placed in any phase is one they
    // all reach.
    if participants.count != u64::from(region.schedule.threads_per_workgroup) {
        return Err(cooperative(CooperativeTileRule::ParticipantConvergence));
    }
    // Exactly one participant performs the region's owning write, which is what
    // makes `OneGlobalInvocationPerOutput` true of a workgroup that runs several
    // invocations over one output position.
    if tile.commit.count != 1 || !participants.contains_range(tile.commit) {
        return Err(cooperative(CooperativeTileRule::CommitOwnership));
    }

    let phase_count = u32::try_from(tile.phases.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;
    if tile.phases.is_empty()
        || tile
            .phases
            .iter()
            .enumerate()
            .any(|(position, phase)| u32::try_from(position) != Ok(phase.id.get()))
    {
        return Err(cooperative(CooperativeTileRule::PhaseSequence));
    }
    // Nonuniform reachability is stated per phase precisely so it can be
    // refused: a synchronization point inside a phase some participants skip is
    // divergent, and this is where a tile that would place one is caught.
    if tile
        .phases
        .iter()
        .any(|phase| phase.participation != participants)
    {
        return Err(cooperative(CooperativeTileRule::PhaseParticipation));
    }

    let staging_count = u32::try_from(tile.staging.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;
    if tile.staging.is_empty()
        || tile
            .staging
            .iter()
            .enumerate()
            .any(|(position, staging)| u32::try_from(position) != Ok(staging.id.get()))
    {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    tile.local_memory_bytes()
        .ok_or_else(|| cooperative(CooperativeTileRule::StructuralLimit))?;
    // A lifetime that ends before it begins, or that names a phase the tile does
    // not have, cannot bound anything.
    if tile.staging.iter().any(|staging| {
        staging.live_from > staging.live_through || staging.live_through.get() >= phase_count
    }) {
        return Err(cooperative(CooperativeTileRule::StagingLifetime));
    }

    // One writer per slot, and every in-range slot written. The two are checked
    // together over one occupancy map because they are the two halves of the
    // same statement: the participants' writes are a bijection onto the
    // allocation's slots.
    let mut written: Vec<Vec<bool>> = tile
        .staging
        .iter()
        .map(|staging| {
            vec![
                false;
                usize::try_from(staging.slots)
                    .unwrap_or(usize::MAX)
                    .min(usize::try_from(MAX_COOPERATIVE_STAGING_SLOTS).unwrap_or(usize::MAX))
            ]
        })
        .collect();
    for phase in &tile.phases {
        for write in &phase.writes {
            let staging = resolve_staging(tile, write.staging, staging_count)?;
            if phase.id < staging.live_from || phase.id > staging.live_through {
                return Err(cooperative(CooperativeTileRule::StagingLifetime));
            }
            let occupancy = written
                .get_mut(usize::try_from(write.staging.get()).unwrap_or(usize::MAX))
                .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
            for slot in addressed_slots(participants, write.span, staging.slots)? {
                let slot = occupancy
                    .get_mut(usize::try_from(slot).unwrap_or(usize::MAX))
                    .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
                // A second write to a live slot needs a per-round lifetime and a
                // per-round visibility edge this profile does not model, so a
                // rewriting tree is refused here rather than approximated.
                if std::mem::replace(slot, true) {
                    return Err(cooperative(CooperativeTileRule::StagingConflict));
                }
            }
        }
    }
    if written
        .iter()
        .any(|occupancy| occupancy.iter().any(|written| !written))
    {
        return Err(cooperative(CooperativeTileRule::StagingCoverage));
    }

    for phase in &tile.phases {
        for read in &phase.reads {
            let staging = resolve_staging(tile, read.staging, staging_count)?;
            if phase.id < staging.live_from || phase.id > staging.live_through {
                return Err(cooperative(CooperativeTileRule::StagingLifetime));
            }
            addressed_slots(participants, read.span, staging.slots)?;
            // Coverage above already proved every in-range slot has a writer, so
            // what remains is the *ordering*: the writer must be in an earlier
            // phase, or the read observes values its own phase is still
            // producing and no synchronization point could ever separate them.
            if !tile.phases.iter().any(|producer| {
                producer.id < phase.id
                    && producer
                        .writes
                        .iter()
                        .any(|write| write.staging == read.staging)
            }) {
                return Err(cooperative(CooperativeTileRule::StagedProducer));
            }
        }
    }

    let edges = tile.visibility_edges();
    if edges.is_empty() {
        return Err(cooperative(CooperativeTileRule::NoVisibilityEdge));
    }
    verify_synchronization(tile, &edges)
}

/// Verifies the synchronization authority that orders one tile's handoffs.
///
/// Runs after the dataflow rules and reads their results, which is why it takes
/// the derived edges rather than recomputing them: the edges are the obligation,
/// and this decides whether anything legally discharges each one. Every rule is
/// stated against a fact a point *declares*, so each is separately perturbable
/// — the model can express an unadmitted kind, a boundary that is not a program
/// point, a narrowed participant set, a weaker ordering, or a convergence claim
/// with nothing behind it.
fn verify_synchronization(
    tile: &CooperativeTile,
    edges: &[VisibilityEdge],
) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = tile.coordinates.participants;
    // A tile whose participants are one invocation stages values it reads back
    // itself: program order already orders the handoff, so a point there is the
    // semantically redundant barrier this authority exists to eliminate.
    if participants.count < 2 {
        return Err(synchronization(SynchronizationRule::SingleParticipant));
    }
    if tile.synchronization.len() > MAX_COOPERATIVE_SYNCHRONIZATION_POINTS {
        return Err(synchronization(SynchronizationRule::StructuralLimit));
    }
    if tile
        .synchronization
        .iter()
        .enumerate()
        .any(|(position, point)| u32::try_from(position) != Ok(point.id.get()))
    {
        return Err(synchronization(SynchronizationRule::PointSequence));
    }

    // The one realization every point of this tile must state. Derived from the
    // edges rather than read from any point, so the points are checked against
    // the dependency instead of against each other.
    let required = required_subject(edges)
        .ok_or_else(|| synchronization(SynchronizationRule::UndischargedVisibility))?;
    let phase_count = u32::try_from(tile.phases.len())
        .map_err(|_| cooperative(CooperativeTileRule::StructuralLimit))?;

    for point in &tile.synchronization {
        // The kind is checked first and separately, because a point naming an
        // unadmitted construct fails for a reason none of the dimension checks
        // below would name: its contract is undefined here, so comparing its
        // scopes against a control barrier's would be comparing the wrong thing.
        if point.subject.kind != required.kind {
            return Err(synchronization(SynchronizationRule::UnadmittedKind));
        }
        let preceding = point.placement.preceding();
        let following = point.placement.following();
        // A real program point sits between two consecutive phases. A "boundary"
        // spanning a phase is not one: that phase's own effects would fall on an
        // undetermined side of the fence.
        if following.get() >= phase_count || preceding.get().checked_add(1) != Some(following.get())
        {
            return Err(synchronization(SynchronizationRule::Placement));
        }
        if point.participants != participants {
            return Err(synchronization(SynchronizationRule::ParticipantSet));
        }
        if point.subject.execution_scope != required.execution_scope {
            return Err(synchronization(SynchronizationRule::ExecutionScope));
        }
        if point.subject.visibility_scope != required.visibility_scope {
            return Err(synchronization(SynchronizationRule::VisibilityScope));
        }
        if point.subject.fenced_spaces != required.fenced_spaces {
            return Err(synchronization(SynchronizationRule::FencedSpaces));
        }
        if point.subject.ordering != required.ordering {
            return Err(synchronization(SynchronizationRule::Ordering));
        }
        // The evidence class, then the derivation it names. A caller's assertion
        // is refused whatever the tile looks like; the derived class is only as
        // good as the re-derivation below, which is why both run.
        if point.convergence != ConvergenceEvidence::EveryParticipantReachesThePoint {
            return Err(synchronization(SynchronizationRule::ConvergenceEvidence));
        }
        if !phases_are_reached_by(tile, &[preceding, following], participants) {
            return Err(synchronization(SynchronizationRule::Convergence));
        }
        // A point that orders nothing consumes a target authority for an
        // operation the program has no reason to perform.
        if !edges.iter().any(|edge| point.discharges(*edge)) {
            return Err(synchronization(SynchronizationRule::RedundantPoint));
        }
    }

    // Exactly one point per edge. Zero leaves a race; two are two schedules
    // spelling one realization, which would give one program two identities.
    for edge in edges {
        match tile.discharging_points(*edge).len() {
            1 => {}
            0 => return Err(synchronization(SynchronizationRule::UndischargedVisibility)),
            _ => return Err(synchronization(SynchronizationRule::RedundantPoint)),
        }
    }
    Ok(())
}

/// Returns whether every named phase is reached by exactly `participants`.
///
/// The convergence derivation, written over the tile's own per-phase
/// participation rather than assumed from the tile-wide rule that currently
/// implies it. Keeping it explicit is what lets a phase-participation change
/// break this check rather than silently leaving a point convergent by
/// inheritance.
fn phases_are_reached_by(
    tile: &CooperativeTile,
    phases: &[super::handles::PhaseId],
    participants: ParticipantRange,
) -> bool {
    phases.iter().all(|id| {
        tile.phases
            .iter()
            .find(|phase| phase.id == *id)
            .is_some_and(|phase| phase.participation == participants)
    })
}

const fn synchronization(rule: SynchronizationRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::Synchronization { rule }
}

/// Resolves one staged access's allocation, refusing an ordinal the tile lacks.
fn resolve_staging(
    tile: &CooperativeTile,
    id: StagingId,
    staging_count: u32,
) -> Result<super::cooperative::WorkgroupStaging, ScheduledRegionDiagnostic> {
    if id.get() >= staging_count {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    tile.staging
        .get(usize::try_from(id.get()).unwrap_or(usize::MAX))
        .copied()
        .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))
}

/// Returns every slot the participants address through one span.
///
/// An address that overflows `u64` and one that merely exceeds the allocation
/// are the same refusal, because both mean the span leaves the storage the tile
/// declared.
fn addressed_slots(
    participants: ParticipantRange,
    span: StagedSpan,
    slots: u64,
) -> Result<Vec<u64>, ScheduledRegionDiagnostic> {
    if span.count == 0 {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    let addressed = CooperativeTile::addressed_slots(participants, span)
        .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
    if addressed.iter().any(|slot| *slot >= slots) {
        return Err(cooperative(CooperativeTileRule::StagingCapacity));
    }
    Ok(addressed)
}

const fn cooperative(rule: CooperativeTileRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::CooperativeTile { rule }
}

/// Returns the boundary output positions one region's owning write covers.
///
/// Equal to the work-item count for every topology in which one invocation owns
/// one output. A cooperative tile runs one invocation per (output, participant)
/// pair, so its owned set is `participants` times smaller — and the ownership
/// proof, the write's bounds proof, and the write's linear index all read this
/// value rather than the work-item count, which would otherwise claim ownership
/// of positions the region never writes.
fn owned_output_positions(region: &ScheduledRegion) -> Option<u64> {
    let work_items = region.schedule.work_items;
    let Some(tile) = cooperative_tile(&region.schedule.reduction) else {
        return Some(work_items);
    };
    let participants = tile.coordinates.participants.count;
    if participants == 0 || !work_items.is_multiple_of(participants) {
        return None;
    }
    Some(work_items / participants)
}

/// Returns the reduction output shape this region's iteration domain realizes.
///
/// A serial or final pass iterates the reduction's own output; a partial pass
/// iterates it once per partition, so its iteration shape carries one trailing
/// axis the reduction domain does not. A cooperative tile has the same trailing
/// axis, one coordinate per participant, for the same reason. Reading the domain
/// back from the iteration shape is what lets one bounds-proof rule serve all
/// three.
fn reduction_output_shape(region: &ScheduledRegion) -> Option<crate::shape::Shape> {
    let shape = &region.index.iteration_shape;
    let trailing_partitions = match &region.schedule.reduction {
        ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            partition,
            ..
        }
        | ReductionTopology::CooperativeWorkgroup { partition, .. } => partition.partitions,
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::MultiPass { .. } => return Some(shape.clone()),
    };
    let kept = shape.rank().checked_sub(1)?;
    let trailing = shape.extents().get(kept)?;
    (trailing.get() == trailing_partitions)
        .then(|| crate::shape::Shape::try_new(shape.extents()[..kept].iter().copied()).ok())
        .flatten()
}

fn verify_proof_records(
    region: &ScheduledRegion,
    reads: &[&Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let Some((write_proof, read_proofs)) = region.index.bounds_proofs.split_last() else {
        return Err(ScheduledRegionDiagnostic::BoundsProofCount);
    };
    if read_proofs.len() != reads.len()
        || read_proofs.iter().zip(reads).any(|(proof, read)| {
            proof.id != read.bounds
                || proof.tensor != read.tensor
                || proof.component_role != read.component_role
        })
        || write_proof.id != write.bounds
        || write_proof.tensor != write.tensor
        || write_proof.component_role != write.component_role
        || read_proofs.iter().any(|proof| proof.id == write_proof.id)
        || region.index.ownership_proof.id != region.schedule.output_owner
        || region.index.ownership_proof.tensor != write.tensor
        || owned_output_positions(region).is_none_or(|output_count| {
            region.index.ownership_proof.kind
                != (OwnershipProofKind::OneGlobalInvocationPerOutput { output_count })
        })
    {
        return Err(ScheduledRegionDiagnostic::ProofReference);
    }
    if read_proofs
        .iter()
        .zip(reads)
        .any(|(proof, read)| !bounds_proof_refines_access(proof, &read.map, region))
        || !bounds_proof_refines_access(write_proof, &write.map, region)
    {
        return Err(ScheduledRegionDiagnostic::BoundsProof);
    }
    Ok(())
}

fn bounds_proof_refines_access(
    proof: &BoundsProof,
    access: &LogicalAccess,
    region: &ScheduledRegion,
) -> bool {
    match (&proof.kind, access) {
        // The owned positions rather than the work items: they are the same
        // number for every topology that runs one invocation per output, and a
        // cooperative tile's write covers one position per workgroup rather than
        // one per invocation.
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LinearIdentity) => {
            owned_output_positions(region).is_some_and(|owned| *element_count == owned)
        }
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::ScalarBroadcast) => {
            *element_count == 1
        }
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::PackedU4LsbZeroTail { logical_elements },
        ) => {
            *logical_elements == region.schedule.work_items
                && *element_count == logical_elements.div_ceil(2)
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
                && reduction_output_shape(region).is_some_and(|domain| *output_shape == domain)
                && axes == access_axes
                && order == access_order
                && input_shape.without_axes(axes) == *output_shape
        }
        // A contraction operand's proven domain is the contiguous linear range
        // of its own elements, exactly as an identity-mapped access's is. It
        // pairs with `LinearRange` for that reason rather than needing a fourth
        // proof structure: which of those positions the access touches is what
        // the map states, and `verify_contraction` proves every coordinate the
        // map derives is in range by requiring per-axis extent agreement.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ContractionOperand { operand_shape, .. },
        ) => super::model::element_count(operand_shape)
            .is_ok_and(|elements| *element_count == elements),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    use crate::schedule::PointwiseF32ExpressionBuilder;
    use crate::schedule::cooperative::{
        CooperativePhase, CooperativeTile, LocalCoordinateSource, LocalCoordinates,
        ParticipantRange, StagedElement, StagedRead, StagedSpan, StagedWrite, VisibilityEdge,
        WorkgroupStaging,
    };
    use crate::schedule::handles::{
        BoundsWitnessId, OwnershipWitnessId, PhaseId, StagingId, SyncPointId,
    };
    use crate::schedule::model::{ContributorOrder, ContributorPartition, LaunchPlan};
    use crate::schedule::numerics::{
        ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
        SubnormalMode, ValueDomainProvenance,
    };
    use crate::schedule::synchronization::{
        FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationPlacement,
        SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
    };
    use crate::shape::{Axis, Shape};

    /// Recorded canonical identity of the strict-`f32` pointwise test region.
    ///
    /// The pointwise program is encoded as a typed, framed topological graph,
    /// so its exact operand order, constants, root, and physical `f32` family are all pinned.
    ///
    /// Rebaselined deliberately at the `tiler.schedule.v3` step, which gave
    /// `TensorRole::Input` and `PointwiseF32Node::Input` their input ordinals:
    /// the domain separator itself moves, every input access and bounds proof
    /// gains four ordinal bytes, and the input leaf's framed length grows from
    /// nine to twenty-one. An earlier rebaseline recorded the old
    /// `ScalarProgram::MultiplyThenAdd` tag (`0x21`) becoming the exact
    /// `ScalarProgram::PointwiseF32` expression encoding (`0x24`).
    const STRICT_F32_REGION_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e7633000000000000000002000000000000000200000000000000030000000000000002010000000000010100000000000200020100000001010000000000000000000000020000000001000000000011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

    fn strict_numerical() -> NumericalRealization {
        NumericalRealization::new(
            "tiler.test.strict-f32",
            0x7fc0_0000,
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

    fn scale_bias_expression(
        scale_bits: u32,
        bias_bits: u32,
    ) -> super::super::PointwiseF32Expression {
        let mut expression = PointwiseF32ExpressionBuilder::new();
        let input = expression.input(InputOrdinal::FIRST).unwrap();
        let scale = expression.constant(scale_bits).unwrap();
        let product = expression.multiply(input, scale).unwrap();
        let bias = expression.constant(bias_bits).unwrap();
        let root = expression.add(product, bias).unwrap();
        expression.build(root).unwrap()
    }

    fn pointwise_builder(id: RegionId, shape: Shape, elements: u64) -> ScheduledRegionBuilder {
        let mut builder = ScheduledRegionBuilder::new(id);
        builder.iteration_shape(shape).unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::FIRST,
                },
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::PointwiseF32(scale_bias_expression(
                2.0_f32.to_bits(),
                1.0_f32.to_bits(),
            )))
            .unwrap();
        builder.numerical(strict_numerical()).unwrap();
        builder
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
        builder
    }

    #[test]
    fn valid_pointwise_region_verifies_and_derives_requirements() {
        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        assert_eq!(verified.region().schedule.work_items, 6);
        assert_eq!(verified.requirements().buffer_bindings, 2);
        assert!(verified.requirements().requires_device_memory);
        // The realization reaches the requirements record per dimension rather
        // than as a predicate, so a feasibility authority can name the exact
        // dimension a target failed to honour (ADR 0076 item 3).
        let requirements = verified.requirements();
        assert_eq!(requirements.input_subnormals, SubnormalMode::Preserve);
        assert_eq!(requirements.result_subnormals, SubnormalMode::Preserve);
        assert_eq!(requirements.contraction, NumericalPermission::Forbidden);
        assert_eq!(requirements.reassociation, NumericalPermission::Forbidden);
        assert_eq!(requirements.permutation, NumericalPermission::Forbidden);
        assert_eq!(requirements.signed_zero, NumericalPermission::Forbidden);
        assert_eq!(
            requirements.nan_assumptions,
            ExceptionalValueAssumption::MakeNoAssumption
        );
        assert_eq!(
            requirements.infinity_assumptions,
            ExceptionalValueAssumption::MakeNoAssumption
        );
    }

    /// A contract that permits both transforms still carries its subnormal
    /// obligation into the requirements record.
    ///
    /// The `requires_strict_f32` predicate this replaced read contraction and
    /// reassociation only, so exactly this realization derived `false` and
    /// would have been admitted on a target declaring no strict-`f32` support
    /// while still demanding preserved subnormals.
    #[test]
    fn a_relaxed_transform_contract_still_carries_its_subnormal_obligation() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder.numerical = Some(NumericalRealization::new(
            "tiler.test.relaxed-transforms-preserved-subnormals",
            0x7fc0_0000,
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Permitted,
            NumericalPermission::Permitted,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
        ));
        let carried = builder.build().unwrap().requirements();
        assert_eq!(carried.contraction, NumericalPermission::Permitted);
        assert_eq!(carried.reassociation, NumericalPermission::Permitted);
        assert_eq!(carried.input_subnormals, SubnormalMode::Preserve);
        assert_eq!(carried.result_subnormals, SubnormalMode::Preserve);
    }

    #[test]
    fn pointwise_f32_admits_output_and_rejects_other_destination_roles() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder.accesses[1].tensor = TensorRole::Output;
        builder.bounds_proofs[1].tensor = TensorRole::Output;
        builder.ownership_proof.as_mut().unwrap().tensor = TensorRole::Output;
        assert!(builder.build().is_ok());

        let mut rejected = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        rejected.accesses[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        };
        rejected.bounds_proofs[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        };
        rejected.ownership_proof.as_mut().unwrap().tensor = TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        };
        assert_eq!(
            rejected.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// Builds the approved `(a * b) + c` region over three input tensors.
    ///
    /// The three reads carry ordinals `0`, `1`, and `2` in access order, one
    /// bounds proof each, and a write of the program output.
    fn three_input_builder(elements: u64) -> ScheduledRegionBuilder {
        let mut expression = PointwiseF32ExpressionBuilder::new();
        let a = expression.input(InputOrdinal::new(0)).unwrap();
        let b = expression.input(InputOrdinal::new(1)).unwrap();
        let c = expression.input(InputOrdinal::new(2)).unwrap();
        let product = expression.multiply(a, b).unwrap();
        let root = expression.add(product, c).unwrap();
        let expression = expression.build(root).unwrap();

        let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
        builder
            .iteration_shape(Shape::from_dims([elements]))
            .unwrap();
        for ordinal in 0..3 {
            builder
                .push_access(Access {
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(ordinal),
                    },
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(ordinal),
                    ownership: None,
                })
                .unwrap();
        }
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(3),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        for ordinal in 0..3 {
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(ordinal),
                    tensor: TensorRole::Input {
                        ordinal: InputOrdinal::new(ordinal),
                    },
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: elements,
                    },
                })
                .unwrap();
        }
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(3),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::PointwiseF32(expression))
            .unwrap();
        builder.numerical(strict_numerical()).unwrap();
        builder
            .schedule(linear_schedule(elements, OwnershipWitnessId::new(0)))
            .unwrap();
        builder
    }

    #[test]
    fn a_three_input_pointwise_region_verifies_and_binds_one_buffer_per_read() {
        let verified = three_input_builder(4).build().unwrap();
        assert_eq!(verified.requirements().buffer_bindings, 4);
        assert_eq!(verified.region().index.accesses.len(), 4);
    }

    /// Read `i` must be input `i`: a permuted binding is a different program.
    ///
    /// Swapping two ordinals leaves every other fact — access count, modes,
    /// proofs, expression — intact, so this isolates the correspondence rule
    /// from the arity rule below.
    #[test]
    fn read_accesses_must_carry_their_own_ordinal_in_order() {
        let mut permuted = three_input_builder(4);
        permuted.accesses.swap(0, 1);
        permuted.bounds_proofs.swap(0, 1);
        assert_eq!(
            permuted.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        // A repeated ordinal leaves one input unbound while two reads name one
        // tensor, which the same rule refuses.
        let mut repeated = three_input_builder(4);
        repeated.accesses[2].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(1),
        };
        repeated.bounds_proofs[2].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(1),
        };
        assert_eq!(
            repeated.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// An expression leaf with no read access behind it is refused by count.
    ///
    /// Without this the kernel lowering would look up input `2` among two
    /// loaded values, and the region would have promised a buffer its signature
    /// never declares.
    #[test]
    fn a_pointwise_region_reads_exactly_one_tensor_per_expression_leaf() {
        let mut short = three_input_builder(4);
        short.accesses.remove(2);
        short.bounds_proofs.remove(2);
        assert_eq!(
            short.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::AccessCount]
        );

        // The converse: an access no leaf reads is refused by the same rule.
        let mut long = three_input_builder(4);
        long.accesses.insert(
            3,
            Access {
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::new(3),
                },
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(4),
                ownership: None,
            },
        );
        long.bounds_proofs.insert(
            3,
            BoundsProof {
                id: BoundsWitnessId::new(4),
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::new(3),
                },
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 4 },
            },
        );
        assert_eq!(
            long.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::AccessCount]
        );
    }

    /// Two regions differing only in which input a leaf reads differ in identity.
    ///
    /// `(a * b) + c` and `(a * a) + c` compute different things, and before the
    /// ordinal reached the encoding neither the role nor the leaf could say so.
    #[test]
    fn input_ordinals_separate_canonical_scheduled_region_identity() {
        let three = three_input_builder(4).build().unwrap();

        let mut expression = PointwiseF32ExpressionBuilder::new();
        let a = expression.input(InputOrdinal::new(0)).unwrap();
        let b = expression.input(InputOrdinal::new(1)).unwrap();
        // The same shape of program, but the product squares its first input.
        let product = expression.multiply(a.clone(), a).unwrap();
        let root = expression.add(product, b).unwrap();
        let squared = expression.build(root).unwrap();

        let mut builder = three_input_builder(4);
        builder.accesses.remove(2);
        builder.bounds_proofs.remove(2);
        builder.accesses[2].bounds = BoundsWitnessId::new(3);
        builder.scalar_program = Some(ScalarProgram::PointwiseF32(squared));
        let two = builder.build().unwrap();

        assert_ne!(
            three.canonical_identity().as_bytes(),
            two.canonical_identity().as_bytes()
        );
    }

    fn identity_with_pointwise_expression(
        expression: super::super::PointwiseF32Expression,
    ) -> Vec<u8> {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder.scalar_program = Some(ScalarProgram::PointwiseF32(expression));
        builder
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec()
    }

    /// The reciprocal square root is a distinct node from the exponential.
    ///
    /// Both are one-argument elementary functions over one input, so nothing but
    /// the node tag distinguishes their expressions. An appended tag that had
    /// collided with `Exp`'s would make these two identities equal, which is the
    /// concrete form of "the schedule domain did not step": the new tag
    /// separates, and every tag below it keeps its meaning.
    #[test]
    fn the_reciprocal_square_root_node_separates_identity_from_the_exponential() {
        fn elementary(reciprocal_square_root: bool) -> super::super::PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let root = if reciprocal_square_root {
                builder.rsqrt(input).unwrap()
            } else {
                builder.exp(input).unwrap()
            };
            builder.build(root).unwrap()
        }
        assert_ne!(
            identity_with_pointwise_expression(elementary(true)),
            identity_with_pointwise_expression(elementary(false))
        );
    }

    #[test]
    fn pointwise_identity_canonicalizes_ready_order_and_separates_semantics() {
        fn ready_order(reverse: bool) -> super::super::PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let (two, three) = if reverse {
                let three = builder.constant(3.0_f32.to_bits()).unwrap();
                let two = builder.constant(2.0_f32.to_bits()).unwrap();
                (two, three)
            } else {
                let two = builder.constant(2.0_f32.to_bits()).unwrap();
                let three = builder.constant(3.0_f32.to_bits()).unwrap();
                (two, three)
            };
            let (add, product) = if reverse {
                let product = builder.multiply(input.clone(), three).unwrap();
                let add = builder.add(input, two).unwrap();
                (add, product)
            } else {
                let add = builder.add(input.clone(), two).unwrap();
                let product = builder.multiply(input, three).unwrap();
                (add, product)
            };
            let root = builder.add(add, product).unwrap();
            builder.build(root).unwrap()
        }

        let canonical = identity_with_pointwise_expression(ready_order(false));
        assert_eq!(
            canonical,
            identity_with_pointwise_expression(ready_order(true))
        );

        let association = {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let inner = builder.add(two, three).unwrap();
            let root = builder.add(input, inner).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        };
        assert_ne!(canonical, association);

        let operand_order = {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let add = builder.add(two, input.clone()).unwrap();
            let product = builder.multiply(three, input).unwrap();
            let root = builder.add(add, product).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        };
        assert_ne!(canonical, operand_order);

        let constant_bits = {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let two = builder.constant((-2.0_f32).to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let add = builder.add(input.clone(), two).unwrap();
            let product = builder.multiply(input, three).unwrap();
            let root = builder.add(add, product).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        };
        assert_ne!(canonical, constant_bits);
    }

    #[test]
    fn pointwise_identity_separates_signed_zero_and_nan_payload_bits() {
        fn literal_identity(bits: u32) -> Vec<u8> {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(InputOrdinal::FIRST).unwrap();
            let constant = builder.constant(bits).unwrap();
            let root = builder.add(input, constant).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        }

        assert_ne!(
            literal_identity(0.0_f32.to_bits()),
            literal_identity((-0.0_f32).to_bits())
        );
        assert_ne!(literal_identity(0x7fc0_0001), literal_identity(0x7fc0_0002));
    }

    /// Every numerical dimension separates canonical scheduled-region identity.
    ///
    /// The encoder previously wrote `profile_key`, the NaN bits, and two
    /// derived permission booleans, so two regions differing only in a
    /// subnormal dimension collided. Each realization below holds `profile_key`
    /// fixed precisely so the key cannot stand in for the field values it names
    /// (ADR 0076 item 6). The subject is `encode_identity` rather than the
    /// builder because the schedule verifier separately constrains the scalar
    /// program to agree with the contraction permission, and varying both would
    /// stop isolating the numerical field.
    #[test]
    fn every_numerical_dimension_separates_scheduled_region_identity() {
        let region = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap()
            .region()
            .clone();
        let baseline = NumericalRealization::new(
            "tiler.test.identity-probe",
            0x7fc0_0000,
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
        );
        let preserving_sign = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        };
        let always_positive = SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::AlwaysPositive,
        };
        let realizations = [
            baseline,
            NumericalRealization {
                input_subnormals: preserving_sign,
                ..baseline
            },
            NumericalRealization {
                result_subnormals: preserving_sign,
                ..baseline
            },
            // The flushed zero's sign is part of the behaviour, so two flushes
            // producing different zeros are different realizations.
            NumericalRealization {
                input_subnormals: always_positive,
                ..baseline
            },
            NumericalRealization {
                contraction: NumericalPermission::Permitted,
                ..baseline
            },
            NumericalRealization {
                reassociation: NumericalPermission::Permitted,
                ..baseline
            },
            NumericalRealization {
                permutation: NumericalPermission::Permitted,
                ..baseline
            },
            NumericalRealization {
                signed_zero: NumericalPermission::Permitted,
                ..baseline
            },
            NumericalRealization {
                nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CompilerProven,
                },
                ..baseline
            },
            NumericalRealization {
                infinity_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::RuntimeValidated,
                },
                ..baseline
            },
            NumericalRealization {
                nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                    provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
                },
                ..baseline
            },
        ];

        let mut seen: Vec<CanonicalScheduledRegionIdentity> = Vec::new();
        for realization in realizations {
            let mut candidate = region.clone();
            candidate.index.numerical = realization;
            let identity = encode_identity(&candidate);
            assert!(
                !seen.contains(&identity),
                "{realization:?} collided with an earlier realization"
            );
            seen.push(identity);
        }
    }

    /// The exact canonical identity of the governed strict-`f32` test region.
    ///
    /// Completing the encoding over both subnormal dimensions and re-encoding
    /// each permission as a tagged value changed these bytes. Pinning them
    /// keeps a later reordering or omission from slipping past the distinctness
    /// test above, which only proves that the six realizations differ from each
    /// other.
    #[test]
    fn the_strict_f32_region_has_its_recorded_canonical_identity() {
        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        let hex =
            verified
                .canonical_identity()
                .as_bytes()
                .iter()
                .fold(String::new(), |mut hex, byte| {
                    let _ = write!(hex, "{byte:02x}");
                    hex
                });
        assert_eq!(hex, STRICT_F32_REGION_IDENTITY_HEX);
    }

    #[test]
    fn equivalent_regions_with_different_ids_share_identity() {
        let first = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        let second = pointwise_builder(RegionId::new(7), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        assert_ne!(first.region().index.id, second.region().index.id);
        assert_eq!(
            first.canonical_identity().as_bytes(),
            second.canonical_identity().as_bytes()
        );
    }

    #[test]
    fn distinct_content_has_distinct_identity() {
        let first = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        let second = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 4]), 8)
            .build()
            .unwrap();
        assert_ne!(
            first.canonical_identity().as_bytes(),
            second.canonical_identity().as_bytes()
        );
    }

    #[test]
    fn zero_domain_pointwise_region_verifies() {
        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0)
            .build()
            .unwrap();
        assert_eq!(verified.region().schedule.work_items, 0);
    }

    #[test]
    fn launch_that_undercounts_the_domain_is_rejected() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder
            .schedule
            .as_mut()
            .expect("schedule was set")
            .work_items = 5;
        let error = builder.build().unwrap_err();
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::LaunchCoverage]
        );
    }

    #[test]
    fn write_without_ownership_is_rejected_by_the_access_contract() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder.accesses[1].ownership = None;
        let error = builder.build().unwrap_err();
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::AccessContract]
        );
    }

    #[test]
    fn dangling_bounds_witness_is_rejected_by_proof_reference() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        builder.accesses[0].bounds = BoundsWitnessId::new(9);
        let error = builder.build().unwrap_err();
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::ProofReference]
        );
        // The builder is recovered intact for amend-and-retry.
        let (recovered, _) = error.into_parts();
        assert_eq!(recovered.accesses.len(), 2);
    }

    #[test]
    fn setting_a_component_twice_is_a_local_insertion_error() {
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(0));
        builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
        assert_eq!(
            builder.iteration_shape(Shape::from_dims([4])),
            Err(ScheduleBuildError::ComponentAlreadySet {
                component: ScheduleComponent::IterationShape,
            })
        );
    }

    #[test]
    fn incomplete_region_reports_the_missing_component() {
        let error = ScheduledRegionBuilder::new(RegionId::new(0))
            .build()
            .unwrap_err();
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::IncompleteRegion {
                component: ScheduleComponent::IterationShape,
            }]
        );
    }

    /// A realization that permits exactly the freedoms a split consumes.
    ///
    /// Reassociation is permitted and every other dimension stays at its strict
    /// resolution, so a region admitted under it is admitted for reassociation
    /// alone. Permutation in particular stays forbidden, which is what makes
    /// the admission tests below evidence of independence rather than of a
    /// generally relaxed contract.
    fn reassociating_numerical() -> NumericalRealization {
        NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..strict_numerical()
        }
    }

    /// The split every multi-pass fixture below declares: `6 = 3 x 2`.
    const SPLIT: ContributorPartition = ContributorPartition {
        partitions: 3,
        contributors_per_partition: 2,
    };

    /// Builds the partial pass of a `[2, 6] -> [2]` reduction split three ways.
    fn partial_pass_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
        let partial_elements = 2 * partition.partitions;
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(2));
        builder
            .iteration_shape(
                partial_reduction_shape(&Shape::from_dims([2]), partition)
                    .expect("a rank-two partial shape is within the governed bound"),
            )
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ReductionContributor {
                    input_shape: Shape::from_dims([2, 6]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::ReductionDomain {
                    input_shape: Shape::from_dims([2, 6]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: partial_elements,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: partial_elements,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            })
            .unwrap();
        builder.numerical(reassociating_numerical()).unwrap();
        builder
            .schedule(KernelSchedule {
                reduction: ReductionTopology::MultiPass {
                    pass: ReductionPass::Partial,
                    partition,
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                ..linear_schedule(partial_elements, OwnershipWitnessId::new(0))
            })
            .unwrap();
        builder
    }

    /// Builds a `[2, 6] -> [2]` serial reduction over the first input tensor.
    ///
    /// The shape the extrema fixtures below share. A *serial* topology rather
    /// than a split, because the serial arm is the only one the identity-less
    /// fold is admitted under; the refusal of every other topology is asserted
    /// separately rather than assumed.
    fn serial_reduction_builder(scalar: ScalarProgram) -> ScheduledRegionBuilder {
        let input = Shape::from_dims([2, 6]);
        let output = Shape::from_dims([2]);
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(41));
        builder.iteration_shape(output.clone()).unwrap();
        builder
            .push_access(Access {
                tensor: FIRST_INPUT,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ReductionContributor {
                    input_shape: input.clone(),
                    output_shape: output.clone(),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: FIRST_INPUT,
                component_role: None,
                kind: BoundsProofKind::ReductionDomain {
                    input_shape: input,
                    output_shape: output,
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 2 },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
            })
            .unwrap();
        builder.scalar_program(scalar).unwrap();
        builder.numerical(strict_numerical()).unwrap();
        builder
            .schedule(KernelSchedule {
                reduction: ReductionTopology::Serial {
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                ..linear_schedule(2, OwnershipWitnessId::new(0))
            })
            .unwrap();
        builder
    }

    /// The extrema fold this family embeds, over the shared fixture shape.
    fn maximum_scalar() -> ScalarProgram {
        ScalarProgram::StrictSerialMaximum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
        }
    }

    /// The extrema fold verifies as a serial pass reading the original input.
    #[test]
    fn the_extrema_fold_verifies_as_a_serial_pass() {
        let region = serial_reduction_builder(maximum_scalar())
            .build()
            .expect("an extrema serial pass verifies");
        assert!(matches!(
            region.region().index.scalar_program,
            ScalarProgram::StrictSerialMaximum { .. }
        ));
    }

    /// The extrema fold does not share identity with the bare serial sum.
    ///
    /// The two regions differ in nothing but their scalar program — same access
    /// relation, same contributor order, same numerical realization — so an
    /// appended scalar-program tag that had collided with an existing one would
    /// make these equal. It is the check behind "the schedule domain did not
    /// step": the new tag separates, and every earlier tag keeps its meaning.
    ///
    /// The sum reads an intermediate where the extrema fold reads the first
    /// input, so the bare-sum control is built with that one field changed and
    /// nothing else.
    #[test]
    fn the_extrema_fold_has_its_own_canonical_identity() {
        let maximum = serial_reduction_builder(maximum_scalar())
            .build()
            .expect("the extrema pass verifies");
        let mut bare = serial_reduction_builder(ScalarProgram::StrictSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        });
        bare.accesses[0].tensor = TensorRole::Intermediate;
        bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
        let bare = bare.build().expect("the bare pass verifies");
        assert_ne!(maximum.canonical_identity(), bare.canonical_identity());
    }

    /// An empty reduced domain is refused, because the family has no identity.
    ///
    /// **This is the one obligation the extrema fold has and no sum does.** A sum
    /// commits `+0.0`; `Maximum` has no value it could commit, so the region is
    /// refused rather than given a default. The control is the *same shape* under
    /// the bare sum, which verifies — so the refusal is about the family and not
    /// about the zero extent.
    #[test]
    fn an_empty_reduced_domain_is_refused_for_the_identity_less_fold() {
        let empty_input = Shape::from_dims([2, 0]);
        let widen = |builder: &mut ScheduledRegionBuilder| {
            let LogicalAccess::ReductionContributor { input_shape, .. } =
                &mut builder.accesses[0].map
            else {
                panic!("the fixture reads a reduction contributor");
            };
            *input_shape = empty_input.clone();
            let BoundsProofKind::ReductionDomain { input_shape, .. } =
                &mut builder.bounds_proofs[0].kind
            else {
                panic!("the fixture proves a reduction domain");
            };
            *input_shape = empty_input.clone();
        };

        let mut maximum = serial_reduction_builder(maximum_scalar());
        widen(&mut maximum);
        assert_eq!(
            maximum.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "an identity-less fold over an empty domain has no value to commit"
        );

        // The control: the identical region under the bare sum verifies, because
        // that family declares `+0.0` for the empty case.
        let mut bare = serial_reduction_builder(ScalarProgram::StrictSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        });
        bare.accesses[0].tensor = TensorRole::Intermediate;
        bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
        widen(&mut bare);
        assert!(bare.build().is_ok());
    }

    /// Every topology but the serial one is refused for the extrema fold.
    ///
    /// **Fail-closed rather than a legality claim**, and the distinction is worth
    /// stating: the fold is order-insensitive, so a split or a cooperative tile
    /// over it is legal in principle. What is missing is a staged-partial
    /// contract that does not rest on an empty-domain identity, which is
    /// `implement-parallel-reduction-strategies`' work. Refusing until then is
    /// the conservative direction, and this asserts the refusal so that widening
    /// it has to be deliberate.
    #[test]
    fn every_topology_but_the_serial_one_is_refused_for_the_extrema_fold() {
        let mut split = serial_reduction_builder(maximum_scalar());
        split.schedule.as_mut().unwrap().reduction = ReductionTopology::MultiPass {
            pass: ReductionPass::Partial,
            partition: SPLIT,
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
        };
        assert!(split.build().is_err());

        let mut none = serial_reduction_builder(maximum_scalar());
        none.schedule.as_mut().unwrap().reduction = ReductionTopology::None;
        assert!(none.build().is_err());

        // The control: the unmodified serial fixture verifies, so the refusals
        // above are about the topology rather than about the fixture.
        assert!(serial_reduction_builder(maximum_scalar()).build().is_ok());
    }

    /// Builds the final pass that combines those partials into `[2]`.
    fn final_pass_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
        let partial_shape = partial_reduction_shape(&Shape::from_dims([2]), partition)
            .expect("a rank-two partial shape is within the governed bound");
        let axes = vec![partial_reduction_axis(&Shape::from_dims([2])).expect("rank one fits u32")];
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(3));
        builder.iteration_shape(Shape::from_dims([2])).unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ReductionContributor {
                    input_shape: partial_shape.clone(),
                    output_shape: Shape::from_dims([2]),
                    axes: axes.clone(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::ReductionDomain {
                    input_shape: partial_shape,
                    output_shape: Shape::from_dims([2]),
                    axes: axes.clone(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 2 },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            })
            .unwrap();
        builder.numerical(reassociating_numerical()).unwrap();
        builder
            .schedule(KernelSchedule {
                reduction: ReductionTopology::MultiPass {
                    pass: ReductionPass::Final,
                    partition,
                    axes,
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                ..linear_schedule(2, OwnershipWitnessId::new(0))
            })
            .unwrap();
        builder
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

    /// Both passes of a split verify, and neither needs a barrier to do so.
    ///
    /// The partial pass runs one invocation per (output, partition) pair and the
    /// final pass one per output; the values move between them through the
    /// materialized partial tensor alone, which is what makes the split a
    /// dispatch-boundary strategy rather than a workgroup one.
    #[test]
    fn both_passes_of_a_split_reduction_verify() {
        let partial = partial_pass_builder(SPLIT).build().unwrap();
        let combine = final_pass_builder(SPLIT).build().unwrap();
        assert_eq!(partial.region().schedule.work_items, 6);
        assert_eq!(combine.region().schedule.work_items, 2);
        assert_eq!(partial.requirements().local_memory_bytes, 0);
        assert_eq!(combine.requirements().local_memory_bytes, 0);
        // The split reports the freedom it consumes and only that freedom.
        assert_eq!(
            partial.requirements().reassociation,
            NumericalPermission::Permitted
        );
        assert_eq!(
            partial.requirements().permutation,
            NumericalPermission::Forbidden
        );
    }

    /// A split has an identity distinct from the serial reduction it replaces.
    #[test]
    fn a_split_pass_is_not_identical_to_a_serial_pass() {
        let split = partial_pass_builder(SPLIT).build().unwrap();
        let mut serial = partial_pass_builder(SPLIT);
        // The same region under the same contract, differing only in whether
        // its contributor sequence is split.
        serial.schedule.as_mut().unwrap().reduction = ReductionTopology::Serial {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: true,
            permits_permutation: false,
        };
        // The serial reading of that region is itself rejected, and the rule it
        // names is the bounds proof: a serial reduction's iteration domain *is*
        // its reduction domain, so a proof over `[2]` no longer refines an
        // access whose region iterates `[2, 3]`. The two topologies are
        // therefore not interchangeable even before identity is compared.
        assert_eq!(
            serial.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::BoundsProof]
        );
        let final_pass = final_pass_builder(SPLIT).build().unwrap();
        assert_ne!(
            split.canonical_identity().as_bytes(),
            final_pass.canonical_identity().as_bytes()
        );
    }

    /// Reassociation is what a split consumes, and denying it rejects the split.
    #[test]
    fn a_split_is_rejected_when_reassociation_is_denied() {
        for mut builder in [partial_pass_builder(SPLIT), final_pass_builder(SPLIT)] {
            builder.numerical = Some(strict_numerical());
            let ReductionTopology::MultiPass {
                permits_reassociation,
                ..
            } = &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *permits_reassociation = false;
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
            );
        }
    }

    /// Permutation is a separate permission the split neither needs nor uses.
    ///
    /// Both directions are driven, because checking one permission and
    /// consuming the other is invisible when only the permitted case is tested:
    /// a contract that permits permutation but forbids reassociation must still
    /// reject the split, and one that forbids permutation but permits
    /// reassociation must still admit it.
    #[test]
    fn permutation_neither_admits_nor_blocks_a_split() {
        let mut permuting_only = partial_pass_builder(SPLIT);
        permuting_only.numerical = Some(NumericalRealization {
            permutation: NumericalPermission::Permitted,
            ..strict_numerical()
        });
        let ReductionTopology::MultiPass {
            permits_reassociation,
            permits_permutation,
            ..
        } = &mut permuting_only.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *permits_reassociation = false;
        *permits_permutation = true;
        assert_eq!(
            permuting_only.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "permutation must not stand in for the reassociation a split consumes"
        );

        // The complementary direction: the default fixture already forbids
        // permutation and permits reassociation, and it verifies.
        assert!(partial_pass_builder(SPLIT).build().is_ok());
    }

    /// A split whose product does not cover the contributor sequence rejects.
    ///
    /// The two cases reach different rules, and both are the right one. A split
    /// that changes the *partition count* also changes the partial tensor the
    /// region iterates, so its bounds proof stops refining its access first; a
    /// split that keeps the count and misstates the per-partition share reaches
    /// the coverage check itself. Driving both is what shows neither an
    /// over-covering nor an under-covering split can slip through on the other
    /// one's silence.
    #[test]
    fn an_inexact_split_is_rejected() {
        for (partition, expected) in [
            // Six contributors, five covered, and a partial tensor of five.
            (
                ContributorPartition {
                    partitions: 5,
                    contributors_per_partition: 1,
                },
                ScheduledRegionDiagnostic::BoundsProof,
            ),
            // A split of nothing covers nothing, and stages nothing.
            (
                ContributorPartition {
                    partitions: 0,
                    contributors_per_partition: 2,
                },
                ScheduledRegionDiagnostic::BoundsProof,
            ),
            // Three partitions, as the region stages, but nine contributors
            // claimed where the access supplies six.
            (
                ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 3,
                },
                ScheduledRegionDiagnostic::NumericalOrAccessRefinement,
            ),
            // The same count, four covered where the access supplies six.
            (
                ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 1,
                },
                ScheduledRegionDiagnostic::NumericalOrAccessRefinement,
            ),
        ] {
            let mut builder = partial_pass_builder(SPLIT);
            let ReductionTopology::MultiPass {
                partition: declared,
                ..
            } = &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *declared = partition;
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [expected],
                "{partition:?} does not cover six contributors exactly once each"
            );
        }
    }

    /// An accumulation narrower than the element width is rejected, not accepted.
    #[test]
    fn a_narrowed_accumulation_width_is_rejected() {
        for narrower in [ArithmeticType::F16, ArithmeticType::Bf16] {
            let mut builder = partial_pass_builder(SPLIT);
            let ReductionTopology::MultiPass { accumulation, .. } =
                &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *accumulation = narrower;
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
                "{narrower:?} is narrower than the width the contract admits"
            );
        }
    }

    /// The final pass must combine exactly one contributor per partition.
    #[test]
    fn a_final_pass_reading_the_wrong_partition_count_is_rejected() {
        let mut builder = final_pass_builder(SPLIT);
        // A partial tensor with a fourth partition the split never produced.
        let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
        else {
            panic!("expected a reduction access")
        };
        *input_shape = Shape::from_dims([2, 4]);
        let BoundsProofKind::ReductionDomain { input_shape, .. } =
            &mut builder.bounds_proofs[0].kind
        else {
            panic!("expected a reduction proof")
        };
        *input_shape = Shape::from_dims([2, 4]);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// A partial pass that writes the program output is not a partial pass.
    #[test]
    fn a_partial_pass_may_not_write_the_program_output() {
        let mut builder = partial_pass_builder(SPLIT);
        builder.accesses[1].tensor = TensorRole::Output;
        builder.bounds_proofs[1].tensor = TensorRole::Output;
        builder.ownership_proof.as_mut().unwrap().tensor = TensorRole::Output;
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// Turns a split partial pass into the squaring-prologue reduction.
    ///
    /// The prologue reads the original input, exactly as the scale-bias one
    /// does, so the read access and its proof move from the intermediate to the
    /// first input tensor along with the scalar program.
    fn squared_partial_pass_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
        let mut builder = partial_pass_builder(partition);
        builder.accesses[0].tensor = TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        };
        builder.bounds_proofs[0].tensor = TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        };
        builder.scalar_program = Some(ScalarProgram::SquaredSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        });
        builder
    }

    /// The squaring-prologue reduction verifies, reading the original input.
    #[test]
    fn the_squaring_prologue_reduction_verifies_as_a_partial_pass() {
        let region = squared_partial_pass_builder(SPLIT)
            .build()
            .expect("a squaring-prologue partial pass verifies");
        assert!(matches!(
            region.region().index.scalar_program,
            ScalarProgram::SquaredSerialSum { .. }
        ));
    }

    /// An accumulation narrower than the declared width is rejected here too.
    ///
    /// **This is `tiler::rms-norm-f32@1`'s accumulator refusal, fired.** The
    /// operation declares `tiler::f32@1` in its definition facts and criterion 3
    /// of `implement-parallel-reduction-strategies` requires a narrower strategy
    /// to be rejected with a typed reason. The check is the schedule verifier's
    /// single accumulation authority rather than a second copy beside it, and
    /// this exercises it on the program the normalization actually schedules —
    /// so a change that admitted a narrower accumulator for the squaring
    /// prologue alone would fail here even while the bare sum's own test passed.
    #[test]
    fn a_narrowed_accumulation_width_is_rejected_for_the_squaring_prologue() {
        for narrower in [ArithmeticType::F16, ArithmeticType::Bf16] {
            let mut builder = squared_partial_pass_builder(SPLIT);
            let ReductionTopology::MultiPass { accumulation, .. } =
                &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *accumulation = narrower;
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
                "{narrower:?} is narrower than the width tiler::rms-norm-f32@1 declares"
            );
        }
        // The control: the same region at the declared width verifies, so the
        // refusals above are about the accumulator rather than about the
        // program.
        assert!(squared_partial_pass_builder(SPLIT).build().is_ok());
    }

    /// The squaring prologue may not be applied in the final pass.
    ///
    /// Squaring a partial sum would square an already-folded value, so the
    /// prologue belongs to the pass that reads the original inputs. The refusal
    /// is what stops a split from applying it twice.
    #[test]
    fn the_squaring_prologue_may_not_carry_the_final_pass() {
        let mut builder = final_pass_builder(SPLIT);
        builder.scalar_program = Some(ScalarProgram::SquaredSerialSum {
            axes: match &builder.scalar_program {
                Some(ScalarProgram::StrictSerialSum { axes, .. }) => axes.clone(),
                other => panic!("expected the final pass's serial sum, not {other:?}"),
            },
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        });
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// The squaring prologue does not share identity with the bare serial sum.
    ///
    /// The two regions differ in nothing but their scalar program — same access
    /// relation, same contributor order, same numerical realization — so an
    /// appended scalar-program tag that had collided with an existing one would
    /// make these equal. It is the check behind "the schedule domain did not
    /// step": the new tag separates, and every earlier tag keeps its meaning.
    #[test]
    fn the_squaring_prologue_reduction_has_its_own_canonical_identity() {
        let squared = squared_partial_pass_builder(SPLIT)
            .build()
            .expect("the squaring-prologue pass verifies");
        let mut bare = squared_partial_pass_builder(SPLIT);
        bare.accesses[0].tensor = TensorRole::Intermediate;
        bare.bounds_proofs[0].tensor = TensorRole::Intermediate;
        bare.scalar_program = Some(ScalarProgram::StrictSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        });
        let bare = bare.build().expect("the bare pass verifies");
        assert_ne!(squared.canonical_identity(), bare.canonical_identity());
    }

    /// Every field of a split separates canonical scheduled-region identity.
    #[test]
    fn every_split_field_separates_scheduled_region_identity() {
        let baseline = partial_pass_builder(SPLIT)
            .build()
            .unwrap()
            .region()
            .clone();
        let mut seen = vec![encode_identity(&baseline)];
        for reduction in [
            ReductionTopology::MultiPass {
                pass: ReductionPass::Final,
                partition: SPLIT,
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                partition: ContributorPartition {
                    partitions: 2,
                    contributors_per_partition: 3,
                },
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                partition: SPLIT,
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F64,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                partition: SPLIT,
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: true,
            },
            ReductionTopology::Serial {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: true,
                permits_permutation: false,
            },
        ] {
            let mut candidate = baseline.clone();
            candidate.schedule.reduction = reduction.clone();
            let identity = encode_identity(&candidate);
            assert!(
                !seen.contains(&identity),
                "{reduction:?} collided with an earlier topology"
            );
            seen.push(identity);
        }
    }

    // ---- Cooperative workgroup tiles -------------------------------------
    //
    // Every fixture below is the same `[2, 6] -> [2]` reduction the split tests
    // use, realized cooperatively: three participants per workgroup, each
    // folding two contributors into its own staging slot, and one commit. The
    // perturbation tests each change exactly one fact of that fixture, so a
    // rejection names the rule the change violated rather than a difference the
    // fixture happened to carry.

    /// The staging allocation every cooperative fixture below declares.
    fn tile_staging(slots: u64, live_through: PhaseId) -> WorkgroupStaging {
        WorkgroupStaging {
            id: StagingId::FIRST,
            element: StagedElement::F32,
            slots,
            live_from: PhaseId::FIRST,
            live_through,
        }
    }

    /// The point that orders the fixture's one handoff.
    ///
    /// Every field is the value the tile's own dependency derives, so a
    /// perturbation test changes exactly one of them and the rejection names the
    /// dimension it changed.
    fn tile_point() -> SynchronizationPoint {
        SynchronizationPoint {
            id: SyncPointId::FIRST,
            subject: SynchronizationSubject {
                kind: SynchronizationKind::ControlBarrier,
                execution_scope: SynchronizationScope::Workgroup,
                visibility_scope: SynchronizationScope::Workgroup,
                fenced_spaces: FencedSpaces {
                    workgroup: true,
                    device: false,
                },
                ordering: MemoryOrdering::AcquireRelease,
            },
            placement: SynchronizationPlacement::PhaseBoundary {
                preceding: PhaseId::FIRST,
                following: PhaseId::new(1),
            },
            participants: ParticipantRange { first: 0, count: 3 },
            convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
        }
    }

    /// The well-formed tile: write your own slot, then read the whole set.
    fn cooperative_tile_fixture() -> CooperativeTile {
        CooperativeTile {
            synchronization: vec![tile_point()],
            coordinates: LocalCoordinates {
                source: LocalCoordinateSource::LocalLinearInvocation,
                participants: ParticipantRange { first: 0, count: 3 },
            },
            staging: vec![tile_staging(3, PhaseId::new(1))],
            phases: vec![
                CooperativePhase {
                    id: PhaseId::FIRST,
                    participation: ParticipantRange { first: 0, count: 3 },
                    writes: vec![StagedWrite {
                        staging: StagingId::FIRST,
                        span: StagedSpan {
                            stride: 1,
                            offset: 0,
                            count: 1,
                        },
                    }],
                    reads: Vec::new(),
                },
                CooperativePhase {
                    id: PhaseId::new(1),
                    participation: ParticipantRange { first: 0, count: 3 },
                    writes: Vec::new(),
                    reads: vec![StagedRead {
                        staging: StagingId::FIRST,
                        span: StagedSpan {
                            stride: 0,
                            offset: 0,
                            count: 3,
                        },
                    }],
                },
            ],
            commit: ParticipantRange { first: 0, count: 1 },
        }
    }

    fn cooperative_topology(tile: CooperativeTile) -> ReductionTopology {
        cooperative_topology_with(tile, SPLIT)
    }

    fn cooperative_topology_with(
        tile: CooperativeTile,
        partition: ContributorPartition,
    ) -> ReductionTopology {
        cooperative_topology_arriving(tile, partition, ContributorArrival::AscendingParticipant)
    }

    fn cooperative_topology_arriving(
        tile: CooperativeTile,
        partition: ContributorPartition,
        arrival: ContributorArrival,
    ) -> ReductionTopology {
        ReductionTopology::CooperativeWorkgroup {
            partition,
            tile,
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            accumulation: ArithmeticType::F32,
            permits_reassociation: true,
            permits_permutation: false,
            arrival,
        }
    }

    /// Builds the cooperative realization of the `[2, 6] -> [2]` reduction.
    ///
    /// One workgroup per output position, three invocations per workgroup, so
    /// the iteration domain is the output shape with the participant axis
    /// appended — the same layout a partial pass uses, which is what keeps the
    /// participant ordinal the innermost coordinate of the invocation index.
    fn cooperative_builder(tile: CooperativeTile) -> ScheduledRegionBuilder {
        cooperative_builder_with(tile, SPLIT)
    }

    /// The same fixture over an explicit split, for the widths `SPLIT` fixes.
    fn cooperative_builder_with(
        tile: CooperativeTile,
        split: ContributorPartition,
    ) -> ScheduledRegionBuilder {
        cooperative_builder_parts(
            split,
            cooperative_topology_with(tile, split),
            reassociating_numerical(),
        )
    }

    /// The fixture region under one chosen arrival and permutation resolution.
    ///
    /// Both are varied together because the rule under test is exactly their
    /// composition: the topology records what the contract resolved, and the
    /// verifier requires the two to agree before it asks what the arrival
    /// consumes.
    fn arriving_builder(
        arrival: ContributorArrival,
        permutation_permitted: bool,
    ) -> ScheduledRegionBuilder {
        let numerical = NumericalRealization {
            permutation: if permutation_permitted {
                NumericalPermission::Permitted
            } else {
                NumericalPermission::Forbidden
            },
            ..reassociating_numerical()
        };
        let ReductionTopology::CooperativeWorkgroup {
            partition,
            tile,
            axes,
            order,
            accumulation,
            permits_reassociation,
            ..
        } = cooperative_topology_arriving(cooperative_tile_fixture(), SPLIT, arrival)
        else {
            panic!("the cooperative fixture builds a cooperative topology")
        };
        cooperative_builder_parts(
            SPLIT,
            ReductionTopology::CooperativeWorkgroup {
                partition,
                tile,
                axes,
                order,
                accumulation,
                permits_reassociation,
                permits_permutation: permutation_permitted,
                arrival,
            },
            numerical,
        )
    }

    fn cooperative_builder_parts(
        split: ContributorPartition,
        reduction: ReductionTopology,
        numerical: NumericalRealization,
    ) -> ScheduledRegionBuilder {
        let participants = split.partitions;
        let work_items = 2 * participants;
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(4));
        builder
            .iteration_shape(
                partial_reduction_shape(&Shape::from_dims([2]), split)
                    .expect("a rank-two cooperative domain is within the governed bound"),
            )
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ReductionContributor {
                    input_shape: Shape::from_dims([2, 6]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::ReductionDomain {
                    input_shape: Shape::from_dims([2, 6]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
            })
            .unwrap();
        // Two positions, not six: the write covers one output per workgroup, and
        // the ownership proof below says the same number.
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 2 },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            })
            .unwrap();
        builder.numerical(numerical).unwrap();
        let threads = u32::try_from(participants).expect("the fixture's width fits u32");
        builder
            .schedule(KernelSchedule {
                threads_per_workgroup: threads,
                reduction,
                launch: LaunchPlan {
                    grid_threads: work_items,
                    threads_per_workgroup: threads,
                    zero_work_skips_dispatch: true,
                },
                ..linear_schedule(work_items, OwnershipWitnessId::new(0))
            })
            .unwrap();
        builder
    }

    /// Applies one edit to the fixture tile and returns the resulting builder.
    fn perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
        let mut tile = cooperative_tile_fixture();
        edit(&mut tile);
        cooperative_builder(tile)
    }

    fn cooperative_rejection(builder: ScheduledRegionBuilder) -> ScheduledRegionDiagnostic {
        let diagnostics = builder.build().unwrap_err().diagnostics().to_vec();
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected exactly one diagnostic, got {diagnostics:?}")
        };
        *diagnostic
    }

    /// One cooperative tile verifies, and states everything the handoff needs.
    #[test]
    fn one_cooperative_tile_verifies_and_derives_its_workgroup_storage() {
        let verified = cooperative_builder(cooperative_tile_fixture())
            .build()
            .expect("the cooperative fixture verifies");
        // Three `f32` slots, which is the only workgroup memory this tile asks
        // for and the value a feasibility authority composes against a target's
        // declared threadgroup memory.
        assert_eq!(verified.requirements().local_memory_bytes, 12);
        assert_eq!(verified.requirements().threads_per_workgroup, 3);
        // Six invocations over two output positions: the ownership proof counts
        // the positions, not the invocations.
        assert_eq!(verified.region().schedule.work_items, 6);
        let tile = cooperative_tile(&verified.region().schedule.reduction)
            .expect("the topology carries its tile");
        // The exact dependency a synchronization point would have to discharge.
        assert_eq!(
            tile.visibility_edges(),
            [VisibilityEdge {
                staging: StagingId::FIRST,
                produced_in: PhaseId::FIRST,
                consumed_in: PhaseId::new(1),
            }]
        );
        // The split it consumes, and only that split.
        assert_eq!(
            verified.requirements().reassociation,
            NumericalPermission::Permitted
        );
        assert_eq!(
            verified.requirements().permutation,
            NumericalPermission::Forbidden
        );
    }

    /// The verified tile derives exactly one atomic synchronization requirement.
    ///
    /// One value, not one per point and not five independent dimensions: a
    /// region requires one realization however many times it performs it, and a
    /// target fact must equal the whole subject rather than any part of it.
    #[test]
    fn a_synchronized_tile_derives_one_atomic_realization_requirement() {
        let verified = cooperative_builder(cooperative_tile_fixture())
            .build()
            .expect("the cooperative fixture verifies");
        assert_eq!(
            verified.requirements().synchronization,
            Some(SynchronizationSubject {
                kind: SynchronizationKind::ControlBarrier,
                execution_scope: SynchronizationScope::Workgroup,
                visibility_scope: SynchronizationScope::Workgroup,
                fenced_spaces: FencedSpaces {
                    workgroup: true,
                    device: false,
                },
                ordering: MemoryOrdering::AcquireRelease,
            })
        );
        // The point discharges the tile's one edge, and the derivation agrees
        // with the declaration rather than restating it.
        let tile = cooperative_tile(&verified.region().schedule.reduction)
            .expect("the topology carries its tile");
        let [edge] = tile.visibility_edges()[..] else {
            panic!("the fixture states exactly one handoff")
        };
        assert_eq!(tile.discharging_points(edge).len(), 1);
    }

    /// A schedule with no cooperative tile derives no synchronization at all.
    ///
    /// Absence rather than a zero: nothing downstream may read this as "zero
    /// barriers required", because there is no requirement to read.
    #[test]
    fn a_zero_synchronization_schedule_derives_no_requirement() {
        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .expect("the pointwise fixture verifies");
        assert_eq!(verified.requirements().synchronization, None);
    }

    /// Every synchronization rule of the schedule verifier, driven once each.
    ///
    /// Each row changes exactly one fact of the well-formed fixture, so the
    /// diagnostic names the dimension the change touched. The subject rows are
    /// what make the target fact atomic rather than composable: a schedule
    /// cannot state four correct dimensions and one wrong one and be admitted.
    #[test]
    fn each_schedule_synchronization_rule_refuses_its_own_defect() {
        /// One named perturbation of the fixture tile and the rule it violates.
        type Perturbation = (
            &'static str,
            Box<dyn Fn(&mut CooperativeTile)>,
            SynchronizationRule,
        );
        let edits: Vec<Perturbation> = vec![
            (
                "an unadmitted operation kind",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].subject.kind = SynchronizationKind::Collective;
                }),
                SynchronizationRule::UnadmittedKind,
            ),
            (
                "a boundary that is not a program point",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].placement = SynchronizationPlacement::PhaseBoundary {
                        preceding: PhaseId::new(1),
                        following: PhaseId::new(2),
                    };
                }),
                SynchronizationRule::Placement,
            ),
            (
                "a participant set narrower than the tile's",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].participants = ParticipantRange { first: 0, count: 2 };
                }),
                SynchronizationRule::ParticipantSet,
            ),
            (
                "an arrival scope the handoff does not require",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].subject.execution_scope =
                        SynchronizationScope::Subgroup;
                }),
                SynchronizationRule::ExecutionScope,
            ),
            (
                "a publication scope the handoff does not require",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].subject.visibility_scope = SynchronizationScope::Device;
                }),
                SynchronizationRule::VisibilityScope,
            ),
            (
                "a fence over a memory domain the handoff does not cross",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].subject.fenced_spaces.device = true;
                }),
                SynchronizationRule::FencedSpaces,
            ),
            (
                "an ordering that establishes no happens-before edge",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].subject.ordering = MemoryOrdering::Relaxed;
                }),
                SynchronizationRule::Ordering,
            ),
            (
                "convergence asserted rather than derived",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].convergence = ConvergenceEvidence::CallerAsserted;
                }),
                SynchronizationRule::ConvergenceEvidence,
            ),
            (
                "no point at all for a declared handoff",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization.clear();
                }),
                SynchronizationRule::UndischargedVisibility,
            ),
            (
                "two points over one handoff",
                Box::new(|tile: &mut CooperativeTile| {
                    let mut second = tile.synchronization[0];
                    second.id = SyncPointId::new(1);
                    tile.synchronization.push(second);
                }),
                SynchronizationRule::RedundantPoint,
            ),
            (
                "point ordinals that are not the dense ascending run",
                Box::new(|tile: &mut CooperativeTile| {
                    tile.synchronization[0].id = SyncPointId::new(1);
                }),
                SynchronizationRule::PointSequence,
            ),
        ];
        for (name, edit, expected) in edits {
            assert_eq!(
                cooperative_rejection(perturbed(|tile| edit(tile))),
                ScheduledRegionDiagnostic::Synchronization { rule: expected },
                "{name} was admitted"
            );
        }
    }

    /// The canonical tree tile is exactly the tile every rule above was driven
    /// against.
    ///
    /// The constructor exists so a strategy does not hand-assemble spans,
    /// lifetimes, and a point subject, and this is what makes that safe: it is
    /// compared against the fixture the whole perturbation table refuses defects
    /// of, so the shape a planner emits is the shape those rules were proven on
    /// rather than a second shape that merely also verifies.
    #[test]
    fn the_canonical_tree_tile_is_the_fixture_every_rule_was_driven_against() {
        assert_eq!(
            super::super::workgroup_tree_tile(3),
            Some(cooperative_tile_fixture())
        );
        // The point's subject is derived from the tile's own edges rather than
        // restated, so it cannot be constructed wrong.
        let tile = super::super::workgroup_tree_tile(3).expect("three participants are admitted");
        assert_eq!(
            tile.synchronization[0].subject,
            required_subject(&tile.visibility_edges()).expect("the tile carries one handoff")
        );
        // Below two participants the handoff is within one invocation, which the
        // synchronization authority refuses; the constructor declines rather
        // than emitting a tile that could only be rejected.
        assert_eq!(super::super::workgroup_tree_tile(1), None);
        assert_eq!(super::super::workgroup_tree_tile(0), None);
        assert_eq!(
            super::super::workgroup_tree_tile(MAX_COOPERATIVE_PARTICIPANTS + 1),
            None
        );
        // And a width the enumeration bound admits is built rather than refused,
        // so the bound check is not silently rejecting everything.
        assert!(super::super::workgroup_tree_tile(MAX_COOPERATIVE_PARTICIPANTS).is_some());
    }

    /// Every width the constructor admits verifies as a whole region.
    ///
    /// The constructor states a dataflow; only the verifier decides whether the
    /// dataflow, the split, and the launch agree. Driving several widths is what
    /// stops the shape from being correct only at the one width the fixture pins.
    #[test]
    fn the_canonical_tree_tile_verifies_at_every_width_its_split_covers() {
        for (participants, contributors_per_partition) in [(2, 3), (3, 2), (6, 1)] {
            let split = ContributorPartition {
                partitions: participants,
                contributors_per_partition,
            };
            let tile = super::super::workgroup_tree_tile(participants)
                .expect("the width is within the enumeration bound");
            let verified = cooperative_builder_with(tile, split)
                .build()
                .unwrap_or_else(|error| {
                    panic!(
                        "width {participants} was refused: {:?}",
                        error.diagnostics()
                    )
                });
            assert_eq!(
                verified.requirements().local_memory_bytes,
                participants * 4,
                "one f32 slot per participant"
            );
            assert_eq!(
                u64::from(verified.requirements().threads_per_workgroup),
                participants
            );
        }
    }

    /// The two permissions stay independent, and the arrival is what separates
    /// them.
    ///
    /// The admitted arrival is fixed by the program, so it consumes
    /// reassociation alone and a contract forbidding permutation admits it. An
    /// arrival the program does not fix consumes permutation *as well*, and the
    /// two refusals are distinct: withholding the permission names the
    /// permission, and granting it still names the construct nothing realizes.
    #[test]
    fn an_unfixed_arrival_order_consumes_permutation_and_is_refused_by_name() {
        // The control: the same fixture with the admitted arrival verifies under
        // a contract that forbids permutation, so neither refusal below is
        // something the fixture would have earned anyway.
        assert!(
            arriving_builder(ContributorArrival::AscendingParticipant, false)
                .build()
                .is_ok()
        );
        // And granting permutation neither breaks nor is required by it: the
        // recorded permission simply tracks the contract.
        assert!(
            arriving_builder(ContributorArrival::AscendingParticipant, true)
                .build()
                .is_ok()
        );
        for arrival in [
            ContributorArrival::NondeterministicArrival,
            ContributorArrival::AtomicAccumulation,
        ] {
            assert!(arrival.requires_permutation());
            assert_eq!(
                cooperative_rejection(arriving_builder(arrival, false)),
                ScheduledRegionDiagnostic::CooperativeTile {
                    rule: CooperativeTileRule::ArrivalPermission,
                },
                "{} was admitted under a contract that forbids permutation",
                arrival.key()
            );
            // Granting permutation moves the refusal to the construct, which is
            // the check that would be dead if the two were collapsed.
            assert_eq!(
                cooperative_rejection(arriving_builder(arrival, true)),
                ScheduledRegionDiagnostic::CooperativeTile {
                    rule: CooperativeTileRule::UnadmittedArrival,
                },
                "{} was admitted as a realizable construct",
                arrival.key()
            );
        }
    }

    /// A single-participant tile's handoff is within one invocation.
    ///
    /// The semantically redundant barrier this authority exists to eliminate:
    /// program order already orders a value an invocation stages and reads back
    /// itself, so a point there consumes a target authority for nothing.
    #[test]
    fn a_single_participant_tile_cannot_carry_a_synchronization_point() {
        let mut tile = cooperative_tile_fixture();
        tile.coordinates.participants = ParticipantRange { first: 0, count: 1 };
        for phase in &mut tile.phases {
            phase.participation = ParticipantRange { first: 0, count: 1 };
        }
        tile.staging[0].slots = 1;
        tile.phases[1].reads[0].span.count = 1;
        tile.synchronization[0].participants = ParticipantRange { first: 0, count: 1 };
        let builder = cooperative_builder_with(
            tile,
            ContributorPartition {
                partitions: 1,
                contributors_per_partition: 6,
            },
        );
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::SingleParticipant,
            }
        );
    }

    /// The convergence derivation refuses a phase not every participant reaches.
    ///
    /// Driven against the derivation directly, and the reason is stated rather
    /// than hidden: the tile's own per-phase participation rule refuses a
    /// non-uniform phase first, so this rule cannot fire end to end today. It is
    /// re-derived here anyway rather than inherited, so a later relaxation of
    /// that tile rule breaks this check instead of silently leaving every point
    /// convergent by inheritance.
    #[test]
    fn the_convergence_derivation_refuses_a_phase_a_participant_skips() {
        let mut tile = cooperative_tile_fixture();
        let participants = tile.coordinates.participants;
        assert!(phases_are_reached_by(
            &tile,
            &[PhaseId::FIRST, PhaseId::new(1)],
            participants
        ));
        tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
        assert!(!phases_are_reached_by(
            &tile,
            &[PhaseId::FIRST, PhaseId::new(1)],
            participants
        ));
        // And a phase the tile does not have is not reached either, which is
        // what stops a placement naming one from reading as convergent.
        assert!(!phases_are_reached_by(
            &tile,
            &[PhaseId::new(7)],
            participants
        ));
    }

    /// Two participants writing one slot is a race the tile can state.
    #[test]
    fn overlapping_staged_writes_are_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[0].writes[0].span.stride = 0;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingConflict,
            }
        );
    }

    /// A read after the allocation's declared lifetime ends is rejected.
    #[test]
    fn a_staged_read_outside_the_declared_lifetime_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.staging[0].live_through = PhaseId::FIRST;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingLifetime,
            }
        );
    }

    /// A slot inside the allocation that no participant writes is rejected.
    #[test]
    fn a_staging_slot_with_no_writer_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.staging[0].slots = 4;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingCoverage,
            }
        );
    }

    /// A phase only some participants reach is rejected.
    ///
    /// The rule a barrier depends on: a synchronization point inside a phase
    /// the remaining participants skip is divergent, so the phase set has to be
    /// uniform before any point can be placed in it.
    #[test]
    fn a_nonuniformly_reachable_phase_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::PhaseParticipation,
            }
        );
    }

    /// A participant run that does not start at local coordinate zero is rejected.
    #[test]
    fn an_invalid_local_coordinate_space_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.coordinates.participants.first = 1;
                for phase in &mut tile.phases {
                    phase.participation.first = 1;
                }
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::LocalCoordinates,
            }
        );
    }

    /// Storage too small for the slots the participants address is rejected.
    #[test]
    fn insufficient_staging_storage_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.staging[0].slots = 2;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingCapacity,
            }
        );
    }

    /// A staged read in the phase that writes it has no producer to observe.
    #[test]
    fn a_staged_read_with_no_producing_phase_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                let read = tile.phases[1].reads.remove(0);
                tile.phases[0].reads.push(read);
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagedProducer,
            }
        );
    }

    /// A tile that stages values nobody reads performs no cooperation.
    #[test]
    fn a_tile_with_no_visibility_edge_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[1].reads.clear();
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::NoVisibilityEdge,
            }
        );
    }

    /// Participants must be the whole workgroup, or a barrier would diverge.
    #[test]
    fn a_participant_set_narrower_than_the_workgroup_is_rejected() {
        let mut builder = cooperative_builder(cooperative_tile_fixture());
        let schedule = builder.schedule.as_mut().unwrap();
        schedule.threads_per_workgroup = 6;
        schedule.launch.threads_per_workgroup = 6;
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ParticipantConvergence,
            }
        );
    }

    /// More than one committing participant contradicts the ownership proof.
    #[test]
    fn a_tile_committing_from_every_participant_is_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.commit = ParticipantRange { first: 0, count: 3 };
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::CommitOwnership,
            }
        );
    }

    /// A split that does not cover the contributor sequence is rejected.
    ///
    /// The partition count is held at the participant count, so this isolates
    /// the coverage half: three participants folding three contributors each
    /// would combine nine of the six the access declares.
    #[test]
    fn a_split_that_does_not_cover_the_contributors_is_rejected() {
        let mut builder = cooperative_builder(cooperative_tile_fixture());
        let ReductionTopology::CooperativeWorkgroup { partition, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a cooperative topology")
        };
        *partition = ContributorPartition {
            partitions: 3,
            contributors_per_partition: 3,
        };
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ContributorSplit,
            }
        );
    }

    /// The ownership proof counts output positions, never invocations.
    ///
    /// Without this the cooperative region would have claimed one owned position
    /// per invocation — six for two outputs — and every consumer reading the
    /// proof would have sized the output tensor three times too large.
    #[test]
    fn a_cooperative_region_owns_one_position_per_workgroup() {
        let mut builder = cooperative_builder(cooperative_tile_fixture());
        builder.ownership_proof = Some(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 },
        });
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ProofReference]
        );
    }

    /// A zero-extent input keeps the reducer's identity and stages nothing.
    ///
    /// The empty result is `+0.0`, which every arm of this verifier requires as
    /// `empty_identity_bits`, and the serial topology commits it from one
    /// invocation with no fold — so the empty case needs no staging, no phase,
    /// and no visibility edge. A cooperative tile over the same domain is
    /// refused rather than made to stage values no participant produces.
    #[test]
    fn a_zero_extent_reduction_keeps_its_identity_without_a_tile() {
        let mut serial = ScheduledRegionBuilder::new(RegionId::new(5));
        serial.iteration_shape(Shape::from_dims([2])).unwrap();
        serial
            .push_access(Access {
                tensor: TensorRole::Intermediate,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ReductionContributor {
                    input_shape: Shape::from_dims([2, 0]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        serial
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(1),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        serial
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(0),
                tensor: TensorRole::Intermediate,
                component_role: None,
                kind: BoundsProofKind::ReductionDomain {
                    input_shape: Shape::from_dims([2, 0]),
                    output_shape: Shape::from_dims([2]),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
            })
            .unwrap();
        serial
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 2 },
            })
            .unwrap();
        serial
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
            })
            .unwrap();
        serial
            .scalar_program(ScalarProgram::StrictSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            })
            .unwrap();
        serial.numerical(strict_numerical()).unwrap();
        serial
            .schedule(KernelSchedule {
                reduction: ReductionTopology::Serial {
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                ..linear_schedule(2, OwnershipWitnessId::new(0))
            })
            .unwrap();
        let empty = serial
            .clone()
            .build()
            .expect("the empty reduction verifies");
        assert_eq!(empty.requirements().local_memory_bytes, 0);
        let ScalarProgram::StrictSerialSum {
            empty_identity_bits,
            ..
        } = &empty.region().index.scalar_program
        else {
            panic!("expected a strict serial sum")
        };
        assert_eq!(*empty_identity_bits, 0.0_f32.to_bits());

        // The same empty domain declared cooperative, with every launch,
        // ownership, and proof fact left exactly as the well-formed fixture
        // states them: nothing to stage, so the tile is refused instead of
        // describing a handoff of values that do not exist.
        let mut cooperative = cooperative_builder(cooperative_tile_fixture());
        let empty_contributors = LogicalAccess::ReductionContributor {
            input_shape: Shape::from_dims([2, 0]),
            output_shape: Shape::from_dims([2]),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        cooperative.accesses[0].map = empty_contributors;
        cooperative.bounds_proofs[0].kind = BoundsProofKind::ReductionDomain {
            input_shape: Shape::from_dims([2, 0]),
            output_shape: Shape::from_dims([2]),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        assert_eq!(
            cooperative_rejection(cooperative),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::EmptyContributorDomain,
            }
        );
    }

    /// A region that stages nothing encodes exactly the bytes it always did.
    ///
    /// The append proof for `tiler.schedule.v3`: the cooperative topology takes
    /// a fresh tag, so no previously encodable region's bytes move and no cache
    /// or artifact holding an earlier identity has to miss. This is the same
    /// pinned strict-`f32` region the domain-step test above compares, asserted
    /// here for the reason that pin exists at all.
    #[test]
    fn the_cooperative_topology_moves_no_earlier_region_identity() {
        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        let mut hex = String::new();
        for byte in verified.canonical_identity().as_bytes() {
            write!(&mut hex, "{byte:02x}").unwrap();
        }
        assert_eq!(hex, STRICT_F32_REGION_IDENTITY_HEX);
    }

    /// Every field of a tile separates canonical scheduled-region identity.
    ///
    /// A dataflow that stages more, phases differently, or commits from another
    /// participant is a different program, so a tile field left out of the
    /// encoding would let two of these share identity.
    #[test]
    fn every_cooperative_tile_field_separates_scheduled_region_identity() {
        let baseline = cooperative_builder(cooperative_tile_fixture())
            .build()
            .unwrap()
            .region()
            .clone();
        let mut seen = vec![encode_identity(&baseline)];
        let variants: Vec<CooperativeTile> = vec![
            perturb_tile(|tile| tile.staging[0].slots = 4),
            perturb_tile(|tile| tile.staging[0].live_through = PhaseId::FIRST),
            perturb_tile(|tile| tile.phases[0].writes[0].span.stride = 2),
            perturb_tile(|tile| tile.phases[0].writes[0].span.offset = 1),
            perturb_tile(|tile| tile.phases[1].reads[0].span.count = 2),
            perturb_tile(|tile| tile.commit = ParticipantRange { first: 2, count: 1 }),
            perturb_tile(|tile| {
                tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
            }),
            perturb_tile(|tile| tile.coordinates.participants.count = 4),
        ];
        for tile in variants {
            let mut candidate = baseline.clone();
            candidate.schedule.reduction = cooperative_topology(tile.clone());
            let identity = encode_identity(&candidate);
            assert!(
                !seen.contains(&identity),
                "{tile:?} collided with an earlier tile"
            );
            seen.push(identity);
        }
    }

    /// The enumeration bounds refuse a tile they could not decide.
    ///
    /// Coverage and disjointness are decided by walking every addressed slot, so
    /// the bounds are what keep that decision finite. Driven here rather than
    /// assumed, because a limit nothing has been seen to trip is a limit that
    /// might not be reached at all.
    #[test]
    fn a_tile_beyond_a_governed_enumeration_bound_is_rejected() {
        let overlong_phases = perturbed(|tile| {
            let template = tile.phases[1].clone();
            for ordinal in 2..=u32::try_from(MAX_COOPERATIVE_PHASES).unwrap() {
                tile.phases.push(CooperativePhase {
                    id: PhaseId::new(ordinal),
                    ..template.clone()
                });
            }
        });
        assert_eq!(
            cooperative_rejection(overlong_phases),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StructuralLimit,
            }
        );

        let oversized_storage = perturbed(|tile| {
            tile.staging[0].slots = MAX_COOPERATIVE_STAGING_SLOTS.saturating_add(1);
        });
        assert_eq!(
            cooperative_rejection(oversized_storage),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StructuralLimit,
            }
        );
    }

    fn perturb_tile(edit: impl FnOnce(&mut CooperativeTile)) -> CooperativeTile {
        let mut tile = cooperative_tile_fixture();
        edit(&mut tile);
        tile
    }

    #[test]
    fn reduction_contributor_count_handles_late_zero_extents() {
        let access = LogicalAccess::ReductionContributor {
            input_shape: Shape::from_dims([u64::MAX, 2, 0]),
            output_shape: Shape::from_dims([]),
            axes: vec![Axis::new(0), Axis::new(1), Axis::new(2)],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        assert_eq!(
            crate::schedule::contributor_count(
                &[Axis::new(0), Axis::new(1), Axis::new(2)],
                &access
            ),
            Ok(0)
        );
    }
}
