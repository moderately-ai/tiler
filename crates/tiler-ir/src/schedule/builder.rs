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
use super::numerics::NumericalRealization;
use super::{MAX_SCHEDULE_ACCESSES, MAX_SCHEDULE_BOUNDS_PROOFS};

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
    let [read, write] = region.index.accesses.as_slice() else {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    };
    verify_access_and_semantics(region, read, write)
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
    verify_proof_records(region, read, write)?;
    let numerical = &region.index.numerical;
    match (
        &region.index.scalar_program,
        &region.schedule.reduction,
        &read.map,
    ) {
        (
            ScalarProgram::MultiplyThenAdd { contraction, .. },
            ReductionTopology::None,
            LogicalAccess::LinearIdentity,
        ) if *contraction == numerical.permits_contraction()
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
            && *permits_reassociation == numerical.permits_reassociation()
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
        _ => return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement),
    }
    Ok(())
}

fn verify_proof_records(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let [read_proof, write_proof] = region.index.bounds_proofs.as_slice() else {
        return Err(ScheduledRegionDiagnostic::BoundsProofCount);
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
        return Err(ScheduledRegionDiagnostic::ProofReference);
    }
    if !bounds_proof_refines_access(read_proof, &read.map, region)
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
    use crate::schedule::handles::{BoundsWitnessId, OwnershipWitnessId};
    use crate::schedule::model::{ContributorOrder, LaunchPlan};
    use crate::schedule::numerics::{NumericalPermission, SubnormalMode};
    use crate::shape::{Axis, Shape};

    fn strict_numerical() -> NumericalRealization {
        NumericalRealization::new(
            "tiler.test.strict-f32",
            0x7fc0_0000,
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
        )
    }

    fn pointwise_builder(id: RegionId, shape: Shape, elements: u64) -> ScheduledRegionBuilder {
        let mut builder = ScheduledRegionBuilder::new(id);
        builder.iteration_shape(shape).unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                mode: AccessMode::Read,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(0),
                ownership: None,
            })
            .unwrap();
        builder
            .push_access(Access {
                tensor: TensorRole::Intermediate,
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
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(1),
                tensor: TensorRole::Intermediate,
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
            .scalar_program(ScalarProgram::MultiplyThenAdd {
                scale_bits: 2.0_f32.to_bits(),
                bias_bits: 1.0_f32.to_bits(),
                canonical_nan_bits: 0x7fc0_0000,
                contraction: false,
            })
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
        assert!(verified.requirements().requires_strict_f32);
        assert!(verified.requirements().requires_device_memory);
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
