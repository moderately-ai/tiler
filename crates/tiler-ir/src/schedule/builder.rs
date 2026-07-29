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

use super::error::{
    ScheduleBuildError, ScheduleComponent, ScheduleLimitKind, ScheduledRegionBuildError,
    ScheduledRegionDiagnostic,
};
use super::handles::RegionId;
use super::model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ExecutionBinding, IndexRegion, KernelSchedule, LogicalAccess, OwnershipProof,
    OwnershipProofKind, ReductionTopology, ResourceRequirements, ScalarProgram, ScheduledRegion,
    TailPolicy, TensorRole, VerifiedScheduledRegion, derive_requirements, element_count,
    encode_identity,
};
use super::numerics::{ExceptionalValueAssumption, NumericalRealization};
use super::{MAX_SCHEDULE_ACCESSES, MAX_SCHEDULE_BOUNDS_PROOFS};
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
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            let [read, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_access_and_semantics(region, read, write)
        }
    }
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
        || codes.tensor != TensorRole::Input
        || codes.component_role != Some(*codes_role)
        || codes.mode != AccessMode::Read
        || codes.ownership.is_some()
        || codes.map
            != (LogicalAccess::PackedU4LsbZeroTail {
                logical_elements: region.schedule.work_items,
            })
        || scale.tensor != TensorRole::Input
        || scale.component_role != Some(*scale_role)
        || scale.mode != AccessMode::Read
        || scale.ownership.is_some()
        || scale.map != LogicalAccess::ScalarBroadcast
        || zero_point.tensor != TensorRole::Input
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
    let numerical = &region.index.numerical;
    match (
        &region.index.scalar_program,
        &region.schedule.reduction,
        &read.map,
    ) {
        (
            ScalarProgram::PointwiseF32(expression),
            ReductionTopology::None,
            LogicalAccess::LinearIdentity,
        ) if expression.is_valid()
            && read.tensor == TensorRole::Input
            && matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output) => {}
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
            && read.tensor == TensorRole::Input
            && write.tensor == TensorRole::Output => {}
        _ => return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
    }
    Ok(())
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
        || region.index.ownership_proof.kind
            != (OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: region.schedule.work_items,
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
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LinearIdentity) => {
            *element_count == region.schedule.work_items
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
                && axes == access_axes
                && order == access_order
                && output_shape == &region.index.iteration_shape
                && input_shape.without_axes(axes) == *output_shape
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;

    use crate::schedule::PointwiseF32ExpressionBuilder;
    use crate::schedule::handles::{BoundsWitnessId, OwnershipWitnessId};
    use crate::schedule::model::{ContributorOrder, LaunchPlan};
    use crate::schedule::numerics::{
        ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
        ValueDomainProvenance,
    };
    use crate::shape::{Axis, Shape};

    /// Recorded canonical identity of the strict-`f32` pointwise test region.
    ///
    /// The pointwise program is encoded as a typed, framed topological graph,
    /// so its exact operand order, constants, root, and physical `f32` family are all pinned.
    /// This pin was intentionally rebaselined when the old
    /// `ScalarProgram::MultiplyThenAdd` tag (`0x21`) became the exact
    /// `ScalarProgram::PointwiseF32` expression encoding (`0x24`).
    const STRICT_F32_REGION_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763200000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000000900000000000000010100000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

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
        let input = expression.input().unwrap();
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
                tensor: TensorRole::Input,
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
                tensor: TensorRole::Input,
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
        rejected.accesses[1].tensor = TensorRole::Input;
        rejected.bounds_proofs[1].tensor = TensorRole::Input;
        rejected.ownership_proof.as_mut().unwrap().tensor = TensorRole::Input;
        assert_eq!(
            rejected.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
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

    #[test]
    fn pointwise_identity_canonicalizes_ready_order_and_separates_semantics() {
        fn ready_order(reverse: bool) -> super::super::PointwiseF32Expression {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input().unwrap();
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
            let input = builder.input().unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let inner = builder.add(two, three).unwrap();
            let root = builder.add(input, inner).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        };
        assert_ne!(canonical, association);

        let operand_order = {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input().unwrap();
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
            let input = builder.input().unwrap();
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
            let input = builder.input().unwrap();
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
