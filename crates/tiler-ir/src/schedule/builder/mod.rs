//! Transactional builder and intrinsic verifier for scheduled regions.
//!
//! Construction follows the ADR 0071 discipline: insertions check local
//! invariants, and the consuming [`ScheduledRegionBuilder::build`] runs
//! whole-region intrinsic verification before returning an opaque
//! [`VerifiedScheduledRegion`]. The verifier proves domain coverage, output
//! ownership and race freedom, tail and launch legality, bounds-proof
//! refinement, reduction contributor and order legality, numerical/access
//! agreement, zero-domain behaviour, and — for a cooperative tile — its
//! participant space, staging dataflow, and the synchronization realization its
//! visibility and anti-dependency edges require. No later cost or feasibility
//! query can repair a schedule this verifier rejects.
//!
//! This file owns the transactional surface — the only place builder state
//! exists — and the submodules below own the verification. The seams between
//! them are the obligations themselves: [`intrinsic`] holds what every region
//! owes whatever it computes and dispatches the rest; [`copy`], [`contraction`],
//! [`elementwise`], and [`reduction`] hold one program family each; [`family`]
//! and [`coverage`] hold what a fold's algebra and its split decide before any
//! topology reads them; [`tile`] holds cross-invocation dataflow and its
//! handoff order; [`proof`] holds the proof records every family discharges;
//! and [`diagnostics`] holds the refusal vocabulary they share.

mod contraction;
mod copy;
mod coverage;
mod diagnostics;
mod elementwise;
mod family;
mod intrinsic;
mod proof;
mod reduction;
mod tile;

#[cfg(test)]
mod structural_relation_tests;
#[cfg(test)]
mod tests;

use crate::schedule::error::{
    ScheduleBuildError, ScheduleComponent, ScheduleLimitKind, ScheduledRegionBuildError,
    ScheduledRegionDiagnostic,
};
use crate::schedule::handles::RegionId;
use crate::schedule::model::{
    Access, BoundsProof, CanonicalScheduledRegionIdentity, IndexRegion, KernelSchedule,
    OwnershipProof, RegionProgram, ResourceRequirements, ScheduledRegion, VerifiedScheduledRegion,
    derive_requirements, encode_identity,
};
use crate::schedule::{MAX_SCHEDULE_ACCESSES, MAX_SCHEDULE_BOUNDS_PROOFS};

use diagnostics::incomplete;
use intrinsic::verify_intrinsic;

// The module's own suites read the verifier's vocabulary through `use super::*`,
// so the spine is where the names they take from this module are stated. Each
// import stays private to this file and reaches the suites only because a child
// module may name what its ancestor imported.
#[cfg(test)]
use crate::schedule::cooperative::ContributorArrival;
#[cfg(test)]
use crate::schedule::error::{
    BlockedWorkgroupRule, ContributorCoverageRule, CooperativeTileRule, VectorLaneRule,
};
#[cfg(test)]
use crate::schedule::handles::AccessOrdinal;
#[cfg(test)]
use crate::schedule::model::{
    AccessMode, BoundsProofKind, ContractionAxisSource, ContributorCoverage, ExecutionBinding,
    LogicalAccess, OwnershipProofKind, ReductionPaddingIdentity, ReductionPass, ReductionTopology,
    ScalarProgram, TailPolicy, TensorRole, cooperative_tile, element_count, partial_reduction_axis,
    partial_reduction_shape,
};
#[cfg(test)]
use crate::schedule::numerics::NumericalRealization;
#[cfg(test)]
use crate::schedule::synchronization::{
    ConvergenceEvidence, SynchronizationRule, required_subject,
};
#[cfg(test)]
use crate::schedule::{
    MAX_COOPERATIVE_PARTICIPANTS, MAX_COOPERATIVE_PHASES, MAX_COOPERATIVE_ROUNDS,
    MAX_COOPERATIVE_STAGING_SLOTS,
};
#[cfg(test)]
use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
#[cfg(test)]
use family::{FamilyTopology, ParallelFamily, split_family};
#[cfg(test)]
use tile::phases_are_reached_by;

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
    program: Option<RegionProgram>,
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
            program: None,
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
            program: Some(index.program),
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

    /// Sets the region program.
    ///
    /// The single slot that replaced the former scalar-program and
    /// numerical-realization pair: an arithmetic program arrives with its
    /// realization and a copy program arrives without one, so no mixed or
    /// half-classified state is expressible in the builder.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleBuildError::ComponentAlreadySet`] if already set.
    pub fn program(&mut self, program: RegionProgram) -> Result<(), ScheduleBuildError> {
        set_once(&mut self.program, program, ScheduleComponent::RegionProgram)
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
        let program = self
            .program
            .clone()
            .ok_or(incomplete(ScheduleComponent::RegionProgram))?;
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
                program,
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
