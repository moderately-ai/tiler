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

use super::blocked::{
    participant_space_matches_block, prove_blocked_bijection, prove_blocked_predicated_cover,
};
use super::cooperative::{
    AntiDependencyEdge, ContributorArrival, CooperativeTile, ParticipantRange, ParticipantSpace,
    StagedSpan, VisibilityEdge,
};
use super::error::{
    BlockedWorkgroupRule, ContributorCoverageRule, CooperativeTileRule, ScheduleBuildError,
    ScheduleComponent, ScheduleLimitKind, ScheduledRegionBuildError, ScheduledRegionDiagnostic,
    VectorLaneRule,
};
#[cfg(test)]
use super::handles::AccessOrdinal;
use super::handles::{RegionId, StagingId};
use super::model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContractionAxisSource, ContributorCoverage, ContributorOrder, ContributorPartition,
    ExecutionBinding, IndexRegion, KernelSchedule, LogicalAccess, OwnershipProof,
    OwnershipProofKind, ReductionPaddingIdentity, ReductionPass, ReductionTopology,
    ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy, TensorRole,
    VerifiedScheduledRegion, contributor_count, cooperative_tile, derive_requirements,
    element_count, encode_identity, partial_reduction_axis, partial_reduction_shape,
    region_arithmetic_type,
};
use super::numerics::{ArithmeticType, ExceptionalValueAssumption, NumericalRealization};
use super::synchronization::{ConvergenceEvidence, SynchronizationRule, required_subject};
use super::{
    MAX_COOPERATIVE_PARTICIPANTS, MAX_COOPERATIVE_PHASE_ACCESSES, MAX_COOPERATIVE_PHASES,
    MAX_COOPERATIVE_ROUNDS, MAX_COOPERATIVE_STAGING_SLOTS, MAX_COOPERATIVE_SYNCHRONIZATION_POINTS,
    MAX_SCHEDULE_ACCESSES, MAX_SCHEDULE_BOUNDS_PROOFS,
};
use crate::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
use crate::shape::Axis;

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

/// Which boundary tensor one fold's contributor read is required to bind.
///
/// Two obligations rather than one role, because two different facts decide it.
/// A family whose scalar program *carries its own prologue* reads the original
/// input, since that is what the prologue applies to; a pass that folds values an
/// earlier dispatch staged reads the intermediate holding them. Both are exact,
/// and a region binding anything else is describing a different computation.
///
/// [`ScalarProgram::StrictSerialSum`] states neither. It says how contributors
/// combine and nothing about where they live, so `sum(x)` over a declared input
/// and the same fold over a materialized prologue's result are one scalar program
/// over two tensors. Requiring the intermediate would make the vocabulary unable
/// to express the first without an identity prologue region — a materialization,
/// and its observable rounding boundary, that no caller's program asked for — and
/// requiring the input would lose the second. Admitting both is what makes the
/// region's own access the thing that says which, rather than a rule guessing it
/// from the fold.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContributorTensor {
    /// This boundary tensor and no other.
    Exactly(TensorRole),
    /// Any input access; declared-interface association belongs to the compiler.
    DeclaredInput,
    /// The fold's declared contributor domain, wherever the plan placed it: the
    /// input tensor the program folds directly, or a materialized intermediate
    /// when a prologue region wrote it.
    DeclaredDomain,
}

impl ContributorTensor {
    /// Returns whether one read's boundary tensor discharges this obligation.
    fn admits(self, tensor: TensorRole) -> bool {
        match self {
            Self::Exactly(required) => tensor == required,
            Self::DeclaredInput => matches!(tensor, TensorRole::Input),
            Self::DeclaredDomain => {
                tensor == TensorRole::Intermediate || Self::DeclaredInput.admits(tensor)
            }
        }
    }
}

/// Which boundary tensor one fold's owning write is required to commit to.
///
/// The write counterpart of [`ContributorTensor`], and it splits on a different
/// fact. A read's tensor is decided by the *scalar program*: a family carrying its
/// own prologue reads the original input, and a pass folding staged partials reads
/// the intermediate holding them. A write's tensor is decided by neither the
/// scalar program nor the family — no fold's algebra says whether its result is
/// the caller's answer or a value a later region consumes. That is a property of
/// the surrounding cover, so the vocabulary must let the region's own access state
/// it rather than fix it per family.
///
/// What *is* fixed is the write of a pass that exists only to stage. A split's
/// partial pass produces partials its final pass folds; they are not any output,
/// and a partial pass committing one to a declared program output would publish
/// an unfolded fragment as the program's answer. That pass therefore carries
/// [`Self::Exactly`] and every committing pass carries [`Self::CoverAssigned`],
/// which is the asymmetry a reader should be able to derive rather than discover.
///
/// Neither variant admits [`TensorRole::Input`]. A region writing a declared input
/// would mutate a tensor the caller owns, whatever it folded to get there.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommittedTensor {
    /// This boundary tensor and no other, because the pass's role in a split
    /// decides it rather than the cover.
    Exactly(TensorRole),
    /// Whichever of the two internal boundary tensors the cover assigned this
    /// region: a declared program output when the region publishes one, or a
    /// materialized intermediate when a later region consumes the value.
    CoverAssigned,
}

impl CommittedTensor {
    /// Returns whether one write's boundary tensor discharges this obligation.
    fn admits(self, tensor: TensorRole) -> bool {
        match self {
            Self::Exactly(required) => tensor == required,
            Self::CoverAssigned => {
                matches!(tensor, TensorRole::Intermediate | TensorRole::Output)
            }
        }
    }
}

/// Runs the intrinsic schedule verifier over an assembled region.
fn verify_intrinsic(region: &ScheduledRegion) -> Result<(), ScheduledRegionDiagnostic> {
    let iteration_count = element_count(&region.index.iteration_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    let schedule = &region.schedule;
    if schedule.launch.threads_per_workgroup != schedule.threads_per_workgroup
        || schedule.threads_per_workgroup == 0
        || !schedule.launch.zero_work_skips_dispatch
        || schedule.work_items != iteration_count
    {
        return Err(ScheduledRegionDiagnostic::LaunchCoverage);
    }
    match schedule.tail {
        TailPolicy::Exact => {
            // The exact grid population is the binding's invocation
            // population. The scalar and blocked bindings run one invocation
            // per iteration coordinate; the fixed-vector map runs one packet
            // per `lanes` coordinates, and its packet arithmetic is proved in
            // the binding match below rather than compared against a count it
            // is deliberately not equal to.
            if !matches!(schedule.binding, ExecutionBinding::FixedVectorMap { .. })
                && schedule.launch.grid_threads != iteration_count
            {
                return Err(ScheduledRegionDiagnostic::LaunchCoverage);
            }
        }
        TailPolicy::Predicated => {
            // The fixed-vector map admits `Exact` alone, refused under its own
            // name before the cooperative pairing rule: a producer that asked
            // for a predicated vector tail needs to be told the tail is the
            // defect, not the topology.
            if matches!(schedule.binding, ExecutionBinding::FixedVectorMap { .. }) {
                return Err(vector_lane(VectorLaneRule::ExactTailRequired));
            }
            if !matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) || !matches!(schedule.binding, ExecutionBinding::BlockedWorkgroup { .. })
            {
                return Err(ScheduledRegionDiagnostic::LaunchCoverage);
            }
        }
    }
    match &schedule.binding {
        ExecutionBinding::GlobalLinearInvocation => {
            if matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) {
                return Err(blocked(BlockedWorkgroupRule::BindingRequired));
            }
        }
        // The accepted fixed-vector map slice: `Exact` tail (proved above),
        // the map-parallel topologies alone, checked `N mod W == 0`, and the
        // accepted launch identity `work_items = N`, `grid_threads = N / W`.
        // Packet `p`, lane `l` owns output `p * W + l`, which stays inside the
        // domain exactly because the division is exact. Nothing here reads
        // the numerical realization: grouping independent outputs into
        // packets changes no operand, rounding site, or contributor order
        // (ADR 0093 decision 2), so no permission is consulted or consumed.
        // Nothing here consults a target either — every fact is the region's
        // own literal `N`, `W`, and launch.
        ExecutionBinding::FixedVectorMap { lanes } => {
            match schedule.reduction {
                ReductionTopology::None | ReductionTopology::Serial { .. } => {}
                // Refused by name, not by wildcard semantics: every other
                // topology awaits its own accepted vector boundary, and the
                // arms are spelled out so a topology added later is a build
                // error here rather than a silent admission.
                ReductionTopology::MultiPass { .. }
                | ReductionTopology::Contraction { .. }
                | ReductionTopology::LiveContraction { .. }
                | ReductionTopology::CooperativeWorkgroup { .. }
                | ReductionTopology::CooperativeContraction { .. } => {
                    return Err(vector_lane(VectorLaneRule::UnsupportedReduction));
                }
            }
            let lane_count = lanes.get();
            // Divisibility is a fact of `N` and `W` alone, decided before the
            // launch is read so a nondivisible domain is named as such rather
            // than as whatever launch the producer happened to state. A zero
            // domain is a multiple of every admitted width and dispatches
            // nothing under `zero_work_skips_dispatch`.
            if !iteration_count.is_multiple_of(lane_count) {
                return Err(vector_lane(VectorLaneRule::NondivisibleCoverage));
            }
            // The launch is checked as `grid_threads * W == N` with checked
            // multiplication rather than as a division, so an overflowing
            // packet product and a wrong packet count are distinct refusals.
            // With divisibility already proved, equality holds exactly for
            // `grid_threads = N / W` — and `grid_threads = N` with a
            // reinterpreted builtin is exactly what the second refusal
            // catches, for every `W >= 2` the lane type can hold.
            let Some(covered) = schedule.launch.grid_threads.checked_mul(lane_count) else {
                return Err(vector_lane(VectorLaneRule::PacketArithmeticOverflow));
            };
            if covered != iteration_count {
                return Err(vector_lane(VectorLaneRule::PacketPopulation));
            }
        }
        ExecutionBinding::BlockedWorkgroup { block, workgroups } => {
            if !matches!(
                schedule.reduction,
                ReductionTopology::CooperativeContraction { .. }
            ) {
                return Err(blocked(BlockedWorkgroupRule::BindingForbidden));
            }
            match schedule.tail {
                TailPolicy::Exact => prove_blocked_bijection(
                    &region.index.iteration_shape,
                    block,
                    workgroups,
                    schedule.work_items,
                    schedule.threads_per_workgroup,
                    schedule.launch.grid_threads,
                )?,
                TailPolicy::Predicated => prove_blocked_predicated_cover(
                    &region.index.iteration_shape,
                    block,
                    workgroups,
                    schedule.work_items,
                    schedule.threads_per_workgroup,
                    schedule.launch.grid_threads,
                )?,
            }
        }
    }
    // A cooperative tile's participant space is decided here, beside launch
    // coverage, rather than with the rest of the tile's rules — because the
    // participant count it determines is load-bearing *before* those rules run.
    // `owned_output_positions` divides the work items by that count during proof
    // verification, so a malformed space or one disagreeing with the launch
    // would otherwise surface as a proof-reference mismatch: fail-closed, but
    // naming the ownership proof for a defect entirely in the tile.
    if let Some(tile) = cooperative_tile(&schedule.reduction) {
        verify_participant_space(
            tile.coordinates.participants,
            schedule.threads_per_workgroup,
        )?;
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
        // The same access contract at a different width, plus the one obligation
        // that is `bf16`'s alone: the region's declared canonical arithmetic NaN
        // payload must be its own format's.
        ScalarProgram::PointwiseBf16(expression) => {
            let Some((write, reads)) = region.index.accesses.split_last() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_pointwise_bf16(region, expression, reads, write)
        }
        // A fold carrying an epilogue reads one tensor like every other fold: the
        // epilogue's own leaf is the folded value rather than a boundary, which
        // is why it joins this group instead of the pointwise one whose read
        // count is its expression's leaf count.
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::StrictSerialMaximum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. } => {
            let [read, write] = region.index.accesses.as_slice() else {
                return Err(ScheduledRegionDiagnostic::AccessCount);
            };
            verify_access_and_semantics(region, read, write)
        }
        // The registered operation's exact-two semantic signature requires two
        // reads, and this scheduled form preserves that arity. ADR 0087's fifth
        // structural rule is independent: it limits one index's participation
        // across operands, not the operation's operand count.
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
    if matches!(
        region.schedule.reduction,
        ReductionTopology::CooperativeContraction { .. }
    ) {
        return verify_cooperative_contraction(region, left, right, write);
    }
    if matches!(
        region.schedule.reduction,
        ReductionTopology::LiveContraction { .. }
    ) {
        return verify_live_contraction(region, left, right, write);
    }
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
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
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

/// Verifies a contraction whose contracted extent is a live input-axis operand.
///
/// The accepted `LiveContraction` / `ContractionOperand` spelling: free indices
/// and the output stay static, the scalar program's contracted shape is empty
/// rather than a specialized `S`, and the named input axis is the inner trip
/// count. Baking `S` into the operand shapes, the scalar program, or the
/// topology is a different region — `ReductionTopology::Contraction` — and a
/// different identity.
fn verify_live_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::LiveContraction {
        live_access,
        live_axis,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = &region.index.scalar_program
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = &region.index.numerical;
    if contracted_shape.rank() != 0
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
    {
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
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_proof_records(region, &[left, right], write)?;
    element_count(&region.index.iteration_shape)
        .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;

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
            || access_contracted.rank() != 0
            || access_order != order
            || sources.len() != operand_shape.rank()
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        element_count(operand_shape)
            .map_err(|_| ScheduledRegionDiagnostic::ShapeProductOverflow)?;
        if operand_shape
            .extents()
            .iter()
            .any(|extent| extent.get() == 0)
        {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        let mut seen_output = vec![false; output_shape.rank()];
        for (axis, source) in sources.iter().enumerate() {
            let ContractionAxisSource::Output { position } = source else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            let position = usize::try_from(*position)
                .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
            let (Some(extent), Some(slot)) = (
                output_shape.extents().get(position),
                seen_output.get_mut(position),
            ) else {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            };
            if std::mem::replace(slot, true) || operand_shape.extents()[axis] != *extent {
                return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
            }
            output_covered[position] = true;
        }
    }
    if output_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let Some(named) = usize::try_from(live_access.get())
        .ok()
        .and_then(|position| region.index.accesses.get(position))
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if named.mode != AccessMode::Read || !matches!(named.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let LogicalAccess::ContractionOperand { operand_shape, .. } = &named.map else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let expected_axis = u32::try_from(operand_shape.rank())
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    if live_axis.get() != expected_axis {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Verifies the operand-sharing cooperative contraction and its blocked map.
///
/// The sibling of [`verify_cooperative_semantics`]. That gate proves a
/// one-committer reduction tile; this one proves a contraction whose
/// invocations each own an output position and cooperate only by staging
/// operand tiles. The two share the dataflow half of
/// [`verify_cooperative_tile`] and nothing of the ownership theorem.
fn verify_cooperative_contraction(
    region: &ScheduledRegion,
    left: &Access,
    right: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::CooperativeContraction {
        tile,
        contracted_shape: scheduled_contracted,
        contracted_tile,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let ScalarProgram::StrictTensorContraction {
        contracted_shape,
        order,
        ..
    } = &region.index.scalar_program
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let numerical = &region.index.numerical;
    if contracted_shape != scheduled_contracted
        || order != scheduled_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || !*permits_reassociation
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_accumulation_width(*accumulation, &region.index.scalar_program)?;
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
    if !matches!(left.tensor, TensorRole::Input) || !matches!(right.tensor, TensorRole::Input) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    if left.component_role.is_some() || right.component_role.is_some() {
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
        if seen_contracted.iter().any(|read| !read) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
    }
    if output_covered.iter().any(|read| !read) || contracted_covered.iter().any(|read| !read) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let ExecutionBinding::BlockedWorkgroup { block, .. } = &region.schedule.binding else {
        return Err(blocked(BlockedWorkgroupRule::BindingRequired));
    };
    if !participant_space_matches_block(&tile.coordinates.participants, block) {
        return Err(blocked(BlockedWorkgroupRule::ParticipantBlockMismatch));
    }
    if contracted_tile.rank() != contracted_shape.rank() {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let mut tile_count = 1_u64;
    for (extent, tile_extent) in contracted_shape
        .extents()
        .iter()
        .zip(contracted_tile.extents())
    {
        let extent = extent.get();
        let tile_extent = tile_extent.get();
        if tile_extent == 0 || !extent.is_multiple_of(tile_extent) {
            return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
        }
        tile_count = tile_count
            .checked_mul(extent / tile_extent)
            .ok_or(ScheduledRegionDiagnostic::ShapeProductOverflow)?;
    }
    if tile.rounds == 0 || tile.rounds > MAX_COOPERATIVE_ROUNDS {
        return Err(cooperative(CooperativeTileRule::RoundStructure));
    }
    if tile.rounds != tile_count {
        return Err(cooperative(CooperativeTileRule::ContributorSplit));
    }
    verify_operand_tile(tile)
}

/// Verifies an N-input physical `f32` pointwise region.
///
/// The whole obligation is the shared access contract below; this width states
/// nothing of its own. Its canonical arithmetic NaN payload is *not* checked
/// here, and that asymmetry with `bf16` is deliberate rather than an omission:
/// an `f32` region's payload is already compared against the request's own
/// numerical contract by `tiler-compiler`'s subject binding, whereas a 16-bit
/// payload sitting in a 32-bit field has no such comparison anywhere and would
/// otherwise be an unstated reading.
fn verify_pointwise_f32(
    region: &ScheduledRegion,
    expression: &super::pointwise::PointwiseF32Expression,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    verify_pointwise_region(
        region,
        expression.input_count(),
        expression.is_valid(),
        reads,
        write,
    )
}

/// Verifies an N-input physical `bf16` pointwise region.
///
/// The access contract is the shared one below; what is stated here is the
/// obligation that belongs to this width alone.
///
/// **The region's declared canonical arithmetic NaN payload must be `bf16`'s
/// own, zero-extended.** [`NumericalRealization::canonical_arithmetic_nan_bits`]
/// is a 32-bit field and `bf16`'s canonical arithmetic NaN is the 16-bit
/// [`CANONICAL_BF16_ARITHMETIC_NAN_BITS`](crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
/// so which reading applies would otherwise be an unstated invariant every
/// consumer had to guess — and the one consumer that matters guesses wrongly by
/// default, because a lowering that canonicalized to `0x7fc0_0000` would write a
/// pattern no `bf16` value can hold. Requiring it here makes the reading checked
/// at the boundary that produces the region rather than assumed at the boundary
/// that emits it.
fn verify_pointwise_bf16(
    region: &ScheduledRegion,
    expression: &super::pointwise_bf16::PointwiseBf16Expression,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if u32::from(crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS)
        != region.index.numerical.canonical_arithmetic_nan_bits
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    verify_pointwise_region(
        region,
        expression.input_count(),
        expression.is_valid(),
        reads,
        write,
    )
}

/// The access contract every pointwise region satisfies, at any width.
///
/// Three obligations make an N-input region safe, and they are about different
/// things. **The count**: there must be exactly as many reads as the expression
/// has input leaves, or an expression could read a position no access binds — a
/// load through a buffer the signature never declares. The expression's own
/// verifier already proved its access coordinates are the dense `0..n`, so leaf `i` is
/// served by read `i` and the pairing is exhaustive rather than a sample.
/// **The binding category**: each read must name an input or the sole attributed
/// materialized intermediate. Exact declared-input association is absent here
/// and is projected later from the compiler's checked request subject.
/// **The addressing regime**: every access is static, or every access is
/// `LiveRowMajor` on the same inner axis. Canonical lowering has one body-wide
/// live loop and one offset for every boundary effect, so admitting one live map
/// beside a static or differently live map would execute an access relation the
/// region did not prove.
///
/// Shared by the two width-specific verifiers above rather than written twice:
/// the obligation is about *accesses*, and nothing in it reads an element type.
/// The expression's own validity and input count arrive as already-derived facts
/// so this function never has to classify which vocabulary it is looking at,
/// which is what keeps the dispatch above exhaustive instead of pushing a second
/// match in here.
fn verify_pointwise_region(
    region: &ScheduledRegion,
    input_count: usize,
    expression_is_valid: bool,
    reads: &[Access],
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    if reads.is_empty() || reads.len() != input_count {
        return Err(ScheduledRegionDiagnostic::AccessCount);
    }
    if reads
        .iter()
        .any(|read| read.mode != AccessMode::Read || read.ownership.is_some())
        || write.mode != AccessMode::Write
        || !matches!(
            write.map,
            LogicalAccess::LinearIdentity | LogicalAccess::LiveRowMajor { .. }
        )
        || write.ownership != Some(region.schedule.output_owner)
    {
        return Err(ScheduledRegionDiagnostic::AccessContract);
    }
    let read_refs: Vec<&Access> = reads.iter().collect();
    verify_proof_records(region, &read_refs, write)?;
    if !expression_is_valid
        || !matches!(region.schedule.reduction, ReductionTopology::None)
        || !matches!(write.tensor, TensorRole::Intermediate | TensorRole::Output)
        || !reads_bind_boundary_tensors_in_order(reads)
        || !reads
            .iter()
            .all(|read| pointwise_read_map_is_admissible(&read.map, &region.index.iteration_shape))
        || !pointwise_accesses_choose_one_addressing_regime(reads, write)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Returns whether all pointwise boundary effects use one emitted topology.
///
/// Static reads may each keep their own admitted coordinate relation: canonical
/// lowering derives their offsets independently inside the ordinary invocation
/// guard. A live row-major region is different. It has one body-wide loop bound,
/// stride, and element offset, so every read and the owning write must state the
/// same live axis. No access becomes authority for a sibling here; disagreement
/// is refused before a verified region or canonical identity exists.
fn pointwise_accesses_choose_one_addressing_regime(reads: &[Access], write: &Access) -> bool {
    match &write.map {
        LogicalAccess::LinearIdentity => reads
            .iter()
            .all(|read| !matches!(read.map, LogicalAccess::LiveRowMajor { .. })),
        LogicalAccess::LiveRowMajor { inner_axis } => reads.iter().all(|read| {
            matches!(
                &read.map,
                LogicalAccess::LiveRowMajor {
                    inner_axis: read_axis
                } if read_axis == inner_axis
            )
        }),
        LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. }
        | LogicalAccess::ReindexBijection { .. }
        | LogicalAccess::BroadcastReplication { .. }
        | LogicalAccess::ParametricBroadcast { .. } => false,
    }
}

/// Returns whether one pointwise region's reads use admissible boundary categories.
///
/// A read's access position is the expression leaf it serves. `TensorRole::Input`
/// is intentionally fieldless; the compiler binds each exact position against
/// its already-verified request subject. Two local rules remain intrinsic:
///
/// - **At most one read binds the materialized intermediate.** A second read
///   leaves nothing to say which materialization
///   edge it binds — which is exactly why the repeated-read admission above
///   cannot extend to it. `CoverAssembly::from_plan` refuses that a layer up
///   under `cover-intermediate-read-attribution`; stating it here is what stops
///   an intrinsically ambiguous region from being built at all, for a producer
///   that never passes through a cover.
/// - **A program output is never read.** Refused by name rather than under a
///   wildcard, so a role added to the vocabulary later is a build error here
///   instead of silently inheriting an admission nobody checked it for.
fn reads_bind_boundary_tensors_in_order(reads: &[Access]) -> bool {
    let mut intermediate_reads = 0_usize;
    for read in reads {
        match read.tensor {
            TensorRole::Input => {}
            TensorRole::Intermediate => {
                intermediate_reads += 1;
                if intermediate_reads > 1 {
                    return false;
                }
            }
            TensorRole::Output => return false,
        }
    }
    true
}

/// Returns whether one read map is admissible on a pointwise region.
///
/// A pointwise region evaluates one scalar program per output position, so a
/// read may address its operand however it likes *provided* the addressing is
/// a total function of the iteration coordinate with a discharged bounds
/// obligation. Five maps satisfy that and no others do:
///
/// - [`LogicalAccess::LinearIdentity`], the dense one-to-one read.
/// - [`LogicalAccess::ReindexBijection`], whose decodes are required to tile the
///   iteration domain exactly, so every operand element is read once.
/// - [`LogicalAccess::BroadcastReplication`], whose decodes are required to name
///   distinct result axes and leave at least one replicated.
/// - [`LogicalAccess::ParametricBroadcast`], the accepted sourced carrier.
///   Structural rank agreement is checked here; the environment proof is
///   [`super::parametric::interpret_parametric_broadcast`].
/// - [`LogicalAccess::LiveRowMajor`], provided the region-wide regime check
///   above proves every access names the same live inner axis.
///
/// Both structural maps must state the region's own iteration shape as their
/// result shape. That is what stops a region from carrying an access relation
/// derived against some *other* domain — the decodes are divisors of this
/// region's linear coordinate, so a result shape that disagreed with the
/// iteration shape would make every divisor address the wrong window while still
/// satisfying its own internal consistency.
///
/// The remaining maps are refused by name rather than by a wildcard, so a map
/// added to the vocabulary later is a build error here instead of silently
/// inheriting a pointwise admission it was never checked for.
fn pointwise_read_map_is_admissible(
    map: &LogicalAccess,
    iteration_shape: &crate::shape::Shape,
) -> bool {
    match map {
        LogicalAccess::LinearIdentity | LogicalAccess::LiveRowMajor { .. } => true,
        LogicalAccess::ReindexBijection {
            operand_shape,
            result_shape,
            axes,
        } => {
            result_shape == iteration_shape
                && super::model::reindex_decodes_are_bijective(operand_shape, result_shape, axes)
        }
        LogicalAccess::BroadcastReplication {
            operand_shape,
            result_shape,
            axes,
        } => {
            result_shape == iteration_shape
                && super::model::broadcast_decodes_are_replicating(
                    operand_shape,
                    result_shape,
                    axes,
                )
        }
        LogicalAccess::ParametricBroadcast { .. } => {
            super::parametric::parametric_broadcast_read_is_admissible(map, iteration_shape.rank())
        }
        // A scalar broadcast reads a rank-zero parameter and belongs to the
        // decode program; a packed carrier belongs to it too; and the two
        // reduction relations address a contributor domain a pointwise region
        // does not have.
        LogicalAccess::ScalarBroadcast
        | LogicalAccess::PackedU4LsbZeroTail { .. }
        | LogicalAccess::ReductionContributor { .. }
        | LogicalAccess::ContractionOperand { .. } => false,
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
    verify_serial_semantics(region, read, write)
}

/// Verifies the conjunction every serial fold shares.
///
/// The family derivation owns the axes, contributor order, contributor tensor,
/// and empty-domain contract. This gate owns the serial topology's agreement
/// with those facts and deliberately does not read
/// [`SplitFamily::consumes_reassociation`]: a serial fold preserves the
/// declared contributor grouping and spends no reassociation permission.
fn verify_serial_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let numerical = &region.index.numerical;
    let ReductionTopology::Serial {
        axes: scheduled_axes,
        order: scheduled_order,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
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
    let family = split_family(&region.index.scalar_program)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::Serial)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || output_shape != &region.index.iteration_shape
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
        || !CommittedTensor::CoverAssigned.admits(write.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // Preserve the serial admission boundary this refactor inherited. An
    // identity-seeded fold validates the identity it carries and does not need
    // to count contributors; only an identity-less fold owes a proven non-empty
    // domain. Counting every family here would turn contributor-count
    // canonicality and overflow into new serial-sum refusals unrelated to the
    // empty-domain value this check owns.
    let contributors = match family.empty_domain {
        EmptyDomainContract::Identity { .. } => None,
        EmptyDomainContract::NoIdentity => Some(
            contributor_count(axes, &read.map)
                .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?,
        ),
    };
    if !empty_domain_is_satisfied(family.empty_domain, contributors) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    // The sole serial-only family residual. A fold carrying an epilogue applies
    // it to the complete folded value, which is why `split_family` refuses every
    // parallel topology for it. Serial admission still owes three facts the fold
    // family does not: the expression is valid, its one leaf is the accumulator,
    // and it transforms that leaf instead of respelling `SquaredSerialSum`.
    if let ScalarProgram::SquaredSerialSumThenEpilogue { epilogue, .. } =
        &region.index.scalar_program
        && (!epilogue.is_valid()
            || epilogue.input_count() != 1
            || matches!(
                epilogue
                    .nodes()
                    .get(usize::try_from(epilogue.root().index()).unwrap_or(usize::MAX)),
                Some(super::pointwise::PointwiseF32Node::Input { .. })
            ))
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// What one reduction family commits when its contributor domain is empty.
///
/// Two obligations rather than two values of one field. An identity-seeded family
/// names a bit pattern it commits; an identity-less one has no *empty-domain*
/// value it could commit, so what it owes is a *precondition on the domain*
/// instead of a constant — a statement about the empty case alone, and not about
/// whether the family's algebra has a neutral element, which
/// [`ScalarProgram::StrictSerialMaximum`] keeps apart. A typed enum for the
/// reason [`SplitFamily`] is a struct: the exhaustive match that decides it is
/// what forces a family added later to state which obligation it carries rather
/// than inherit whichever it resembles.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EmptyDomainContract {
    /// The family commits these bits when the reduced domain is empty.
    Identity {
        /// Empty-reduction identity bit pattern the scalar program declares.
        bits: u32,
    },
    /// The family has no identity, so a non-empty domain is its precondition.
    NoIdentity,
}

/// Which topology asks for one reduction family's contributor tensor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FamilyTopology {
    /// The complete fold runs serially in one region.
    Serial,
    /// One pass of a split across dispatches.
    MultiPass(ReductionPass),
    /// The complete split runs cooperatively in one workgroup.
    Cooperative,
}

/// Which parallel forms one reduction family can realize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParallelFamily {
    /// The partial pass and cooperative tile are admitted; `final_pass` records
    /// whether this same scalar program can also combine staged partials.
    Split { final_pass: bool },
    /// Only the serial topology is meaningful for this scalar program.
    SerialOnly,
}

/// What one scalar program's own algebra decides about every reduction topology.
///
/// Derived once by [`split_family`] and read by the serial, multi-pass, and
/// cooperative admissions. A family admitted by one and not another is therefore
/// an explicit [`ParallelFamily`] decision rather than a difference between
/// independently maintained match tables.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SplitFamily<'a> {
    /// Reduced axes the scalar program declares.
    axes: &'a [Axis],
    /// Contributor combination order the scalar program declares.
    order: &'a ContributorOrder,
    /// The family's empty-domain obligation.
    empty_domain: EmptyDomainContract,
    /// Whether splitting this family's contributor sequence spends the
    /// contract's reassociation permission.
    ///
    /// True for every sum here, and false for the pinned extrema family alone.
    /// The exception is the family's own algebra rather than a relaxation of any
    /// contract: `Maximum` is associative and commutative on *every* binary32
    /// input — NaN is absorbing and `-0.0 < +0.0` is a total order — so every
    /// tree over the same contributors returns the same bits and a split changes
    /// no observable value. Requiring the permission for it would make a legal
    /// split need one the operation never spends, which is exactly the asymmetry
    /// `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY` states against
    /// `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`.
    ///
    /// The permission is still *recorded* and cross-checked against the region's
    /// declared realization whatever this says, exactly as
    /// [`ReductionTopology::Contraction`] records a permission it does not
    /// consume: a topology disagreeing with its own contract is incoherent
    /// however the fold behaves.
    consumes_reassociation: bool,
    /// Boundary-tensor obligation the complete fold or first split level reads.
    ///
    /// There is deliberately no write counterpart on this struct, and the
    /// absence is the claim: a read's tensor varies by *family*, because a
    /// family's prologue is what decides whether it reads the original input,
    /// while a write's varies only by *pass* — every committing pass carries
    /// [`CommittedTensor::CoverAssigned`] and a split's staging pass carries
    /// [`CommittedTensor::Exactly`], identically for every family. Carrying it
    /// here would let a family declare a write target it has no authority over,
    /// and would invite two families to disagree about one cover's decision.
    contributor_tensor: ContributorTensor,
    /// Parallel forms this family admits.
    parallel: ParallelFamily,
}

impl SplitFamily<'_> {
    /// Derives the contributor tensor one topology reads, or refuses that form.
    ///
    /// The serial fold, split partial pass, and cooperative tile all read the
    /// family's own contributor domain. A final pass, when the family admits one,
    /// reads exactly the intermediate its partial pass staged. Nothing here reads
    /// [`Self::consumes_reassociation`], so deriving a serial tensor cannot give
    /// that field serial meaning.
    const fn read_tensor(self, topology: FamilyTopology) -> Option<ContributorTensor> {
        match topology {
            FamilyTopology::Serial => Some(self.contributor_tensor),
            FamilyTopology::MultiPass(ReductionPass::Partial) | FamilyTopology::Cooperative => {
                match self.parallel {
                    ParallelFamily::Split { .. } => Some(self.contributor_tensor),
                    ParallelFamily::SerialOnly => None,
                }
            }
            FamilyTopology::MultiPass(ReductionPass::Final) => match self.parallel {
                ParallelFamily::Split { final_pass: true } => {
                    Some(ContributorTensor::Exactly(TensorRole::Intermediate))
                }
                ParallelFamily::Split { final_pass: false } | ParallelFamily::SerialOnly => None,
            },
        }
    }
}

/// Decides one family's empty-domain obligation against an optional count.
///
/// The identity-seeded arm requires the strict sum's `+0.0`, which every family
/// carrying an identity here shares — required at each admission rather than at
/// one of them, so a split cannot introduce a second empty-domain answer. The
/// identity-less arm requires a non-empty domain, which is what replaces the
/// constant the family has no correct value for.
///
/// `contributors` is absent only at the serial admission of an identity-seeded
/// fold, where the count is irrelevant to the identity check and deriving it
/// would impose a new canonical-axes and overflow obligation. Parallel
/// admissions already require a count for their split structure and pass it
/// through. The identity-less arm requires a present, nonzero count.
///
/// **Non-emptiness of the whole sequence is non-emptiness of every partition
/// under an exactly covering split**, which is why this needs no per-partition
/// statement and no `has_value` flag on the partials. The split contract fixes
/// `partitions * contributors_per_partition` (times the round count, for a tile)
/// as *exactly* the contributor count, and refuses a zero partition count; a
/// product of nonzero factors equalling a nonzero total forces every factor
/// nonzero, so each partition folds at least one contributor and each staged
/// partial is a real maximum. A carried `has_value` would be a runtime flag that
/// is constantly true — storage in every slot and a branch in every combine, for
/// a fact the verifier settles here.
///
/// **Exact coverage is a premise of that argument, not a detail of it.**
/// [`super::model::ContributorPartition::covers`] still rejects anything else,
/// and [`verify_contributor_coverage`] keeps that meaning on
/// [`ContributorCoverage::Exact`]. A split covering a *padded* sequence has
/// partitions whose real contributors may be none, so the factor argument does
/// not reach it: [`ContributorCoverage::IdentityPadded`] states the family's
/// padding identity and the verifier derives two-sided neutrality before
/// admitting the split.
const fn empty_domain_is_satisfied(
    contract: EmptyDomainContract,
    contributors: Option<u64>,
) -> bool {
    match contract {
        EmptyDomainContract::Identity { bits } => bits == 0.0_f32.to_bits(),
        EmptyDomainContract::NoIdentity => {
            matches!(contributors, Some(contributors) if contributors != 0)
        }
    }
}

/// Derives one reduction family's topology-independent algebraic facts.
///
/// The serial, multi-pass, and cooperative admissions all read this one table.
/// The first split level always reads `contributor_tensor`; a final pass, when
/// admitted, reads the intermediate that level staged. A fused or squared
/// prologue admits no final pass because applying it to a partial would apply it
/// twice. The epilogue-carrying fold is explicitly serial-only because its
/// epilogue applies to the complete fold rather than to a fragment.
fn split_family(program: &ScalarProgram) -> Option<SplitFamily<'_>> {
    match program {
        ScalarProgram::StrictSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredDomain,
            parallel: ParallelFamily::Split { final_pass: true },
        }),
        ScalarProgram::FusedMultiplyAddSerialSum {
            axes,
            order,
            empty_identity_bits,
            contraction,
            ..
        } => (!contraction).then_some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: false },
        }),
        ScalarProgram::SquaredSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: false },
        }),
        ScalarProgram::SquaredSerialSumThenEpilogue {
            axes,
            order,
            empty_identity_bits,
            ..
        } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::Identity {
                bits: *empty_identity_bits,
            },
            consumes_reassociation: true,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::SerialOnly,
        }),
        ScalarProgram::StrictSerialMaximum { axes, order, .. } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::NoIdentity,
            consumes_reassociation: false,
            contributor_tensor: ContributorTensor::DeclaredInput,
            parallel: ParallelFamily::Split { final_pass: true },
        }),
        // No pointwise or decode program folds anything, and the contraction
        // owns a distinct two-read topology rather than this one-read family.
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}

/// Verifies that a parallel topology combines at the width its region computes
/// in.
///
/// **The single accumulation authority for every topology that declares one.**
/// [`ReductionTopology::MultiPass`] and
/// [`ReductionTopology::CooperativeWorkgroup`] are the two variants carrying an
/// `accumulation` field, and both reach this function rather than repeating the
/// comparison, so a change admitting a narrower accumulator for one strategy
/// cannot leave the other refusing it.
///
/// The required width is *derived* from the region's own scalar program rather
/// than compared against a literal `F32`. Every family that reaches either gate
/// is `f32` today — `split_family` refuses the pointwise programs for every
/// topology — so the derivation changes no outcome now; what it
/// changes is that a `bf16` reduction admitted later must state its accumulator
/// instead of inheriting an `f32` one nobody re-checked.
///
/// # Errors
///
/// Returns [`ScheduledRegionDiagnostic::AccumulationWidth`] carrying both
/// widths. Refusing by its own name rather than as a shared refinement failure
/// is criterion 3 of `implement-parallel-reduction-strategies`: the accumulation
/// dtype is an explicit part of the strategy, and a strategy accumulating at a
/// width the contract does not admit is rejected with a typed reason.
fn verify_accumulation_width(
    declared: ArithmeticType,
    program: &ScalarProgram,
) -> Result<(), ScheduledRegionDiagnostic> {
    let required = region_arithmetic_type(program);
    if declared != required {
        return Err(ScheduledRegionDiagnostic::AccumulationWidth { declared, required });
    }
    Ok(())
}

/// Verifies one topology's contributor coverage against the real sequence.
///
/// `rounds` is `1` for a multi-pass split — the partitions are the whole story —
/// and the tile's declared round count for a cooperative one. Exact coverage
/// reuses [`ContributorPartition::covers`] when there is no extra round factor,
/// so that method keeps the meaning every existing consumer already applies.
/// Identity-padded coverage derives the pad count by checked subtraction and
/// requires a canonical suffix: the last unit of the covered sequence still
/// holds a real contributor, and a zero-length pad is exact coverage under
/// another name.
fn verify_contributor_coverage(
    coverage: ContributorCoverage,
    contributors: u64,
    rounds: u64,
    program: &ScalarProgram,
    numerical: &NumericalRealization,
) -> Result<(), ScheduledRegionDiagnostic> {
    match coverage {
        ContributorCoverage::Exact(partition) => {
            verify_exact_coverage(partition, contributors, rounds)
        }
        ContributorCoverage::IdentityPadded {
            partition,
            identity,
        } => {
            verify_padded_coverage(partition, contributors, rounds)?;
            verify_padding_identity(identity, program, numerical)
        }
    }
}

fn verify_exact_coverage(
    partition: ContributorPartition,
    contributors: u64,
    rounds: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if rounds == 0 {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    }
    if rounds == 1 {
        if partition.total_contributors().is_none() {
            return Err(coverage_rule(ContributorCoverageRule::Overflow));
        }
        if !partition.covers(contributors) {
            return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
        }
        return Ok(());
    }
    if partition.partitions == 0 {
        return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
    }
    let Some(total) = partition.total_contributors() else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    let Some(covered) = total.checked_mul(rounds) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if covered != contributors {
        return Err(coverage_rule(ContributorCoverageRule::ExactCoverage));
    }
    Ok(())
}

fn verify_padded_coverage(
    partition: ContributorPartition,
    contributors: u64,
    rounds: u64,
) -> Result<(), ScheduledRegionDiagnostic> {
    if rounds == 0 || partition.partitions == 0 {
        return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
    }
    let Some(per_round) = partition.total_contributors() else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    let Some(capacity) = per_round.checked_mul(rounds) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if capacity < contributors {
        return Err(coverage_rule(
            ContributorCoverageRule::CapacityBelowRealCount,
        ));
    }
    if capacity == contributors {
        return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
    }
    // Canonical suffix: only the last unit may be ragged, and a unit with no
    // real contributor is refused. For `rounds == 1` that is `C > 0`.
    let Some(prefix) = per_round.checked_mul(rounds - 1) else {
        return Err(coverage_rule(ContributorCoverageRule::Overflow));
    };
    if contributors <= prefix {
        return Err(coverage_rule(
            ContributorCoverageRule::NoncanonicalPlacement,
        ));
    }
    Ok(())
}

fn verify_padding_identity(
    identity: ReductionPaddingIdentity,
    program: &ScalarProgram,
    numerical: &NumericalRealization,
) -> Result<(), ScheduledRegionDiagnostic> {
    let required = region_arithmetic_type(program);
    if identity.arithmetic_type() != required {
        return Err(coverage_rule(
            ContributorCoverageRule::ArithmeticTypeMismatch,
        ));
    }
    let Some(combiner) = reduction_combiner(program) else {
        return Err(coverage_rule(ContributorCoverageRule::TwoSidedNeutrality));
    };
    if !identity_is_two_sided_neutral(identity, combiner, numerical) {
        return Err(coverage_rule(ContributorCoverageRule::TwoSidedNeutrality));
    }
    Ok(())
}

/// The binary combiner a padded split injects into.
///
/// Derived from the scalar program rather than declared beside the identity:
/// the identity is a statement about this combiner, and a second field would be
/// a place for a producer to name the wrong one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionCombiner {
    Add,
    Maximum,
}

fn reduction_combiner(program: &ScalarProgram) -> Option<ReductionCombiner> {
    match program {
        ScalarProgram::StrictSerialSum { .. }
        | ScalarProgram::FusedMultiplyAddSerialSum { .. }
        | ScalarProgram::SquaredSerialSum { .. }
        | ScalarProgram::SquaredSerialSumThenEpilogue { .. } => Some(ReductionCombiner::Add),
        ScalarProgram::StrictSerialMaximum { .. } => Some(ReductionCombiner::Maximum),
        ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}

/// Derives two-sided neutrality of `identity` under the region's combiner.
///
/// For IEEE-754 binary32 addition the only possible identities are the two
/// zeros; the witness set is therefore a case analysis, not a sample. `-0.0`
/// is two-sided-neutral with signed zero observable.
/// `+0.0 + (-0.0)` is `+0.0`, so `+0.0` is admitted only when signed-zero
/// elimination is permitted. For the NaN-propagating maximum family with
/// `-0.0 < +0.0`, `-inf` is the unique two-sided identity once each combine
/// is followed by the family's canonicalization.
fn identity_is_two_sided_neutral(
    identity: ReductionPaddingIdentity,
    combiner: ReductionCombiner,
    numerical: &NumericalRealization,
) -> bool {
    match identity {
        ReductionPaddingIdentity::F32(bits) => {
            f32_identity_is_two_sided_neutral(bits, combiner, numerical)
        }
        ReductionPaddingIdentity::F16(_)
        | ReductionPaddingIdentity::Bf16(_)
        | ReductionPaddingIdentity::F64(_) => false,
    }
}

fn f32_identity_is_two_sided_neutral(
    identity: u32,
    combiner: ReductionCombiner,
    numerical: &NumericalRealization,
) -> bool {
    const WITNESSES: [u32; 9] = [
        0x0000_0000, // +0.0
        0x8000_0000, // -0.0
        0x3f80_0000, // 1.0
        0xbf80_0000, // -1.0
        0x0000_0001, // smallest subnormal
        0x0080_0000, // smallest positive normal
        0x7f80_0000, // +inf
        0xff80_0000, // -inf
        0x7fc0_0001, // a non-canonical quiet NaN
    ];
    let combine = match combiner {
        ReductionCombiner::Add => f32_add_bits,
        ReductionCombiner::Maximum => f32_maximum_bits,
    };
    let canonical = numerical.canonical_arithmetic_nan_bits;
    for operand in WITNESSES {
        let left = canonicalize_f32(combine(identity, operand), canonical);
        let right = canonicalize_f32(combine(operand, identity), canonical);
        let expected = canonicalize_f32(operand, canonical);
        if !f32_observably_equal(left, expected, numerical)
            || !f32_observably_equal(right, expected, numerical)
        {
            return false;
        }
    }
    true
}

fn f32_add_bits(lhs: u32, rhs: u32) -> u32 {
    (f32::from_bits(lhs) + f32::from_bits(rhs)).to_bits()
}

fn f32_maximum_bits(lhs: u32, rhs: u32) -> u32 {
    let left = f32::from_bits(lhs);
    let right = f32::from_bits(rhs);
    if left.is_nan() {
        return lhs;
    }
    if right.is_nan() {
        return rhs;
    }
    match left.partial_cmp(&right) {
        Some(std::cmp::Ordering::Less) => rhs,
        Some(std::cmp::Ordering::Equal) => {
            // IEEE 754-2018 maximum orders `-0.0 < +0.0`; `partial_cmp` does not.
            if left == 0.0 && (lhs ^ rhs) == 0x8000_0000 {
                lhs & 0x7fff_ffff
            } else {
                lhs
            }
        }
        Some(std::cmp::Ordering::Greater) | None => lhs,
    }
}

fn canonicalize_f32(bits: u32, canonical_nan: u32) -> u32 {
    if f32::from_bits(bits).is_nan() {
        canonical_nan
    } else {
        bits
    }
}

fn f32_observably_equal(lhs: u32, rhs: u32, numerical: &NumericalRealization) -> bool {
    if lhs == rhs {
        return true;
    }
    if numerical.permits_signed_zero_elimination() {
        let left = f32::from_bits(lhs);
        let right = f32::from_bits(rhs);
        if left == 0.0 && right == 0.0 {
            return true;
        }
    }
    false
}

const fn coverage_rule(rule: ContributorCoverageRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::ContributorCoverage { rule }
}

/// Verifies one pass of a split, multi-dispatch reduction.
///
/// The two passes are checked together here rather than as two more arms of the
/// serial match because every obligation they carry is stated relative to the
/// same [`super::model::ContributorPartition`]: the partial pass proves the split
/// covers its contributor sequence exactly, and the final pass proves it combines
/// exactly one contributor per partition of that same split. Splitting them across
/// unrelated arms would let one pass be verified against a partition the other
/// never agreed to.
fn verify_multi_pass_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::MultiPass {
        pass,
        coverage,
        axes: scheduled_axes,
        order: scheduled_order,
        accumulation,
        permits_reassociation,
        permits_permutation,
    } = &region.schedule.reduction
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let family = split_family(&region.index.scalar_program)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::MultiPass(*pass))
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let numerical = &region.index.numerical;
    // Reassociation is what a split of an order-*sensitive* fold consumes, and it
    // is checked on its own: contributor order is preserved by construction, so a
    // permitted permutation neither grants nor substitutes for this permission.
    // The extrema family consumes nothing, for the reason
    // [`SplitFamily::consumes_reassociation`] records — but it still declares the
    // permissions, and a declaration disagreeing with its own realization would be
    // incoherent whatever the fold's legality.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || (family.consumes_reassociation && !*permits_reassociation)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The accumulator, refused under its own name. Checked after the permissions
    // so a region wrong on both reports the permission, which is the ordering
    // the cooperative gate already follows for `ArrivalPermission`.
    verify_accumulation_width(*accumulation, &region.index.scalar_program)?;

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    // The family's empty-domain obligation, decided against the sequence this
    // pass's split covers. It sits after the contributor count rather than in the
    // agreement block above because the identity-less arm is a statement *about*
    // that count, and the identity-carrying arm's constant is checked here for the
    // same reason it is checked in the serial arms: one empty-domain answer.
    if !empty_domain_is_satisfied(family.empty_domain, Some(contributors)) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let partition = coverage.partition();
    let partial_shape = partial_reduction_shape(output_shape, partition)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;

    let admitted = match pass {
        // The partial pass proves the split covers its own contributor
        // sequence — exactly, or as a suffix-padded extension whose identity
        // the verifier derives — and stages one partial per partition.
        //
        // Its write is the one fold write in this module the cover does not
        // choose, for the reason [`CommittedTensor::Exactly`] states: a partial
        // is an unfolded fragment of the reduction, so committing one to a
        // declared program output would publish a value that is not the fold's
        // result under any cover.
        ReductionPass::Partial => {
            verify_contributor_coverage(
                *coverage,
                contributors,
                1,
                &region.index.scalar_program,
                &region.index.numerical,
            )?;
            region.index.iteration_shape == partial_shape
                && CommittedTensor::Exactly(TensorRole::Intermediate).admits(write.tensor)
        }
        // The final pass proves it combines exactly one contributor per
        // partition of that same split, reading the staged partial tensor.
        //
        // Padding is a fact of the first-level split. The final pass's
        // contributors *are* the staged partials, including any identity-valued
        // ones the partial pass already wrote, so a padded final pass would
        // invent extra partials the tensor does not hold.
        ReductionPass::Final => {
            if matches!(coverage, ContributorCoverage::IdentityPadded { .. }) {
                return Err(coverage_rule(ContributorCoverageRule::PaddedCoverage));
            }
            partial_reduction_axis(output_shape).is_some_and(|axis| axes == [axis].as_slice())
                && *input_shape == partial_shape
                && contributors == partition.partitions
                && region.index.iteration_shape == *output_shape
                && CommittedTensor::CoverAssigned.admits(write.tensor)
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
/// split performs is one the contract permits — or one the family's own algebra
/// makes free, which is the extrema fold alone. The *dataflow* half, in
/// [`verify_cooperative_tile`], proves the staging itself is well formed
/// independently of what is being reduced.
///
/// Neither half authorizes the handoff against a *machine*. The tile's derived
/// edges are discharged here by the points the tile declares, and the
/// structured-kernel verifier separately proves the emitted body puts the
/// realizing barrier between the two effects — but whether any target can
/// perform the realization those points require is a feasibility question this
/// module never answers.
fn verify_cooperative_semantics(
    region: &ScheduledRegion,
    read: &Access,
    write: &Access,
) -> Result<(), ScheduledRegionDiagnostic> {
    let ReductionTopology::CooperativeWorkgroup {
        coverage,
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
    let family = split_family(&region.index.scalar_program)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let read_tensor = family
        .read_tensor(FamilyTopology::Cooperative)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    let numerical = &region.index.numerical;
    // Reassociation is what a split of an order-sensitive fold consumes, exactly
    // as it is for a multi-pass one, and for those families it is required rather
    // than merely recorded: a contract that forbids reassociation forbids the
    // strategy outright. The extrema family spends nothing, so a strict contract
    // admits a tile over it — the whole asymmetry this vocabulary owes the
    // softmax's two passes.
    if *permits_reassociation != numerical.permits_reassociation()
        || *permits_permutation != numerical.permits_permutation()
        || (family.consumes_reassociation && !*permits_reassociation)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The region's own element width, refused under its own name, for the reason
    // the multi-pass gate above states.
    verify_accumulation_width(*accumulation, &region.index.scalar_program)?;
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

    let LogicalAccess::ReductionContributor {
        input_shape,
        output_shape,
        axes: access_axes,
        order: access_order,
    } = &read.map
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    let axes = family.axes;
    if axes != scheduled_axes.as_slice()
        || axes != access_axes.as_slice()
        || family.order != scheduled_order
        || family.order != access_order
        || input_shape.without_axes(axes) != *output_shape
        || !read_tensor.admits(read.tensor)
        // A tile is both halves of a split in one dispatch, so its single write
        // is the fold's committing write and the cover decides where it lands —
        // there is no staging pass here whose target the split structure fixes,
        // because a tile stages in workgroup memory rather than a boundary
        // tensor.
        || !CommittedTensor::CoverAssigned.admits(write.tensor)
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }

    let contributors = contributor_count(axes, &read.map)
        .map_err(|_| ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;
    // An empty contributor domain commits the declared identity from one
    // invocation with no fold, which is what the serial topology already does
    // and what needs no staging at all. A tile over it would declare a
    // visibility edge for values no participant produces. This refusal is also
    // where the identity-less family's own precondition is discharged, which is
    // why `empty_domain_is_satisfied` below can only decide the other arm here.
    if contributors == 0 {
        return Err(cooperative(CooperativeTileRule::EmptyContributorDomain));
    }
    // The strict sum's empty result is `+0.0`, and every identity-carrying family
    // here shares it. Required at the same place the serial and multi-pass
    // admissions require it, so a tile cannot introduce a second empty-domain
    // answer.
    if !empty_domain_is_satisfied(family.empty_domain, Some(contributors)) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    // The iteration domain is the output shape with one trailing participant
    // axis — the same layout a partial pass uses, and for the same reason: it
    // makes the participant ordinal the innermost coordinate of the invocation
    // index, so a participant's local coordinate is derivable without a second
    // layout rule.
    //
    // The trailing axis is one coordinate per *participant*, so it takes the
    // extent product rather than the space's shape: the invocation index is
    // linear whatever shape the tile arranges its participants in.
    //
    // `verify_participant_space` already refused a space whose product does not
    // exist, so this propagates rather than decides — written as a refusal and
    // not an `expect` because the two authorities are ordered by `verify_intrinsic`
    // rather than by a type, and a reordering should cost a diagnostic instead
    // of a panic.
    let Some(participants) = tile.coordinates.participants.participants() else {
        return Err(cooperative(CooperativeTileRule::LocalCoordinates));
    };
    // The round count is bounded before anything multiplies by it. A tile whose
    // phases never run stages nothing yet would still derive staged accesses,
    // visibility edges, and a synchronization requirement — a complete
    // obligation over a program that executes nothing — and one beyond the bound
    // would overflow the product below. Both would otherwise surface as a split
    // mismatch, which names the wrong field.
    if tile.rounds == 0 || tile.rounds > MAX_COOPERATIVE_ROUNDS {
        return Err(cooperative(CooperativeTileRule::RoundStructure));
    }
    // The split covers the sequence once across *every* round — exactly, or as
    // a suffix-padded extension. Participant `p` on round `r` folds the
    // contiguous range at index `r * participants + p`. `covers` is
    // deliberately not extended to know about rounds: it is the multi-pass
    // split's rule, where the partitions are the whole story, and teaching it
    // a second dimension would give one method two meanings.
    //
    // Coverage arithmetic is named separately from the participant/shape
    // agreement [`CooperativeTileRule::ContributorSplit`] still owns: a tile
    // whose split does not match its workgroup is a different defect from a
    // tile whose coverage statement is wrong.
    let partition = coverage.partition();
    verify_contributor_coverage(
        *coverage,
        contributors,
        tile.rounds,
        &region.index.scalar_program,
        &region.index.numerical,
    )?;
    // The iteration domain appends one axis per *participant*, not one per
    // partition of the whole fold: the launch runs one invocation per (output,
    // participant) pair whatever the round count, because rounds are a loop
    // inside each invocation rather than more invocations.
    if partition.partitions != participants
        || partial_reduction_shape(output_shape, partition)
            .is_none_or(|shape| region.index.iteration_shape != shape)
    {
        return Err(cooperative(CooperativeTileRule::ContributorSplit));
    }
    verify_cooperative_tile(tile)
}

/// Verifies one cooperative tile's cross-invocation dataflow.
///
/// Every rule here is decided by enumerating the slots each participant
/// addresses, which is why the governed bounds are checked first: enumeration is
/// what makes disjointness and coverage exact instead of a modular argument, and
/// an unbounded tile would make it unbounded work.
fn verify_cooperative_tile(tile: &CooperativeTile) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = cooperative_participants(tile)?;
    // Exactly one participant performs the region's owning write, which is what
    // makes `OneGlobalInvocationPerOutput` true of a workgroup that runs several
    // invocations over one output position.
    if tile.commit.count != 1 || !participants.contains_range(tile.commit) {
        return Err(cooperative(CooperativeTileRule::CommitOwnership));
    }
    verify_cooperative_tile_dataflow(tile, participants)
}

/// Verifies an operand-sharing tile: every participant commits its own write.
fn verify_operand_tile(tile: &CooperativeTile) -> Result<(), ScheduledRegionDiagnostic> {
    let participants = cooperative_participants(tile)?;
    if tile.commit != participants {
        return Err(cooperative(CooperativeTileRule::OperandTileCommit));
    }
    verify_cooperative_tile_dataflow(tile, participants)
}

fn cooperative_participants(
    tile: &CooperativeTile,
) -> Result<ParticipantRange, ScheduledRegionDiagnostic> {
    let space = tile.coordinates.participants;
    let participant_count = space
        .participants()
        .ok_or_else(|| cooperative(CooperativeTileRule::LocalCoordinates))?;
    Ok(ParticipantRange {
        first: 0,
        count: participant_count,
    })
}

fn verify_cooperative_tile_dataflow(
    tile: &CooperativeTile,
    participants: ParticipantRange,
) -> Result<(), ScheduledRegionDiagnostic> {
    let space = tile.coordinates.participants;
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

    // One writer per slot *within one round*, and every in-range slot written on
    // every round. The two are checked together over one occupancy map because
    // they are the two halves of the same statement: the participants' writes
    // are a bijection onto the allocation's slots.
    //
    // The map spans the phase sequence once, which is exactly one round — every
    // phase runs on every round — so this needs no round dimension and gains
    // none. A tile with several rounds rewrites these slots on the next one, and
    // that is the capability rather than the defect; what the map still refuses
    // is two writers reaching one slot inside a single round, where no point
    // could separate them.
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
            for slot in addressed_slots(space, write.span, staging.slots)? {
                let slot = occupancy
                    .get_mut(usize::try_from(slot).unwrap_or(usize::MAX))
                    .ok_or_else(|| cooperative(CooperativeTileRule::StagingCapacity))?;
                // Two writers to one slot inside one round: nothing orders them,
                // because a point sits between phases and both writes would be
                // on the same side of every one that could.
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
            // The result is discarded because coverage has already proved every
            // in-range slot has a writer, so a read's addressed set needs no
            // second occupancy pass — but the call is *not* discardable: it is
            // what refuses a read whose stride vector disagrees with the tile's
            // rank, and a read whose addressed slot leaves the allocation. A
            // read admitted on either would emit a silently wrong broadcast,
            // which is exactly the defect class a widened relation must not
            // inherit.
            addressed_slots(space, read.span, staging.slots)?;
            // Coverage above already proved every in-range slot has a writer, so
            // what remains is the *ordering*: the writer must be in an earlier
            // phase of the same round, or the read observes values its own phase
            // is still producing and no synchronization point could ever
            // separate them. A loop-carried tile does not widen this. Admitting
            // a same- or later-phase writer would leave the read observing the
            // previous round's value, which the first round has not written at
            // all.
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
    verify_synchronization(tile, participants, &edges, &tile.anti_dependency_edges())
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
///
/// Both derived evidence classes arrive here and are held to the same standard.
/// A visibility edge and an anti-dependency each need exactly one discharging
/// point, and a point earns its place by discharging at least one of either —
/// which is what lets a round boundary, whose whole job is the anti-dependency,
/// be something other than redundant.
///
/// `participants` is the linearized run the caller derived from the tile's
/// participant space, passed rather than re-derived so the two authorities
/// cannot disagree about how many invocations a space holds.
fn verify_synchronization(
    tile: &CooperativeTile,
    participants: ParticipantRange,
    edges: &[VisibilityEdge],
    anti: &[AntiDependencyEdge],
) -> Result<(), ScheduledRegionDiagnostic> {
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
    // The last phase of a round, which is the far side of a round boundary. The
    // caller already proved the ordinals are the dense run `0..phase_count` and
    // that the sequence is nonempty, so the subtraction is exact.
    let last_phase = super::handles::PhaseId::new(
        phase_count
            .checked_sub(1)
            .ok_or_else(|| cooperative(CooperativeTileRule::PhaseSequence))?,
    );

    for point in &tile.synchronization {
        // The kind is checked first and separately, because a point naming an
        // unadmitted construct fails for a reason none of the dimension checks
        // below would name: its contract is undefined here, so comparing its
        // scopes against a control barrier's would be comparing the wrong thing.
        if point.subject.kind != required.kind {
            return Err(synchronization(SynchronizationRule::UnadmittedKind));
        }
        // The phases this point separates, which a placement either names or
        // leaves to the tile. A phase boundary must name two consecutive
        // existing phases: a "boundary" spanning a phase is not a program point,
        // because that phase's own effects would fall on an undetermined side of
        // the fence. A round boundary names none — the phases it separates are
        // the sequence's last and first, which the tile already fixed — so there
        // is nothing here for it to get wrong and the check has nothing to say
        // about it.
        let bounded = match (point.placement.preceding(), point.placement.following()) {
            (Some(preceding), Some(following)) => {
                if following.get() >= phase_count
                    || preceding.get().checked_add(1) != Some(following.get())
                {
                    return Err(synchronization(SynchronizationRule::Placement));
                }
                vec![preceding, following]
            }
            (None, None) => vec![last_phase, super::handles::PhaseId::FIRST],
            // Unreachable while every placement names both ordinals or neither,
            // and refused rather than assumed so a half-named placement added
            // later cannot silently pick one of the two arms above.
            _ => return Err(synchronization(SynchronizationRule::Placement)),
        };
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
        // is refused whatever the tile looks like, and so is the single-round
        // derivation on a tile whose phases repeat — reaching a point is not the
        // same as reaching the same *instance* of it once there are several. The
        // derived class is only as good as the re-derivation below, which is why
        // both run.
        if point.convergence != ConvergenceEvidence::required_for_rounds(tile.rounds) {
            return Err(synchronization(SynchronizationRule::ConvergenceEvidence));
        }
        if !phases_are_reached_by(tile, &bounded, participants) {
            return Err(synchronization(SynchronizationRule::Convergence));
        }
        // A point that orders nothing consumes a target authority for an
        // operation the program has no reason to perform. Both classes count: a
        // round boundary discharges no visibility edge by construction, and
        // refusing it for that would make the loop-carried tile unstatable.
        if !edges.iter().any(|edge| point.discharges(*edge))
            && !anti.iter().any(|edge| point.discharges_anti(*edge))
        {
            return Err(synchronization(SynchronizationRule::RedundantPoint));
        }
    }

    // Exactly one point per edge, in each class. Zero leaves a race — a read of
    // values never published, or a rewrite over values still being read; two are
    // two schedules spelling one realization, which would give one program two
    // identities.
    for edge in edges {
        match tile.discharging_points(*edge).len() {
            1 => {}
            0 => return Err(synchronization(SynchronizationRule::UndischargedVisibility)),
            _ => return Err(synchronization(SynchronizationRule::RedundantPoint)),
        }
    }
    for edge in anti {
        match tile.anti_discharging_points(*edge).len() {
            1 => {}
            0 => {
                return Err(synchronization(
                    SynchronizationRule::UndischargedAntiDependency,
                ));
            }
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
/// declared. A stride vector whose rank disagrees with the participant space's
/// is a *different* refusal and is decided first: the span and the space are
/// each well formed alone and disagree only with one another, so folding it into
/// the capacity refusal would report a storage fault for a shape disagreement.
fn addressed_slots(
    participants: ParticipantSpace,
    span: StagedSpan,
    slots: u64,
) -> Result<Vec<u64>, ScheduledRegionDiagnostic> {
    if span.rank() != participants.rank() {
        return Err(cooperative(CooperativeTileRule::SpanRank));
    }
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

const fn blocked(rule: BlockedWorkgroupRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::BlockedWorkgroup { rule }
}

const fn vector_lane(rule: VectorLaneRule) -> ScheduledRegionDiagnostic {
    ScheduledRegionDiagnostic::VectorLaneBinding { rule }
}

/// Decides a cooperative tile's participant space and its agreement with the
/// launch.
///
/// Two rules, together because they are the two ways a space can be wrong before
/// anything reads the participant count: the space is not a space at all, or it
/// is a well-formed space over a different number of invocations than the
/// workgroup launches.
///
/// A rank above
/// [`MAX_COOPERATIVE_PARTICIPANT_RANK`](super::MAX_COOPERATIVE_PARTICIPANT_RANK)
/// is deliberately not decided here: `ParticipantSpace::new` makes it
/// unrepresentable, so a check for it could never fail.
fn verify_participant_space(
    space: ParticipantSpace,
    threads_per_workgroup: u32,
) -> Result<(), ScheduledRegionDiagnostic> {
    // An empty space names no participants; a zero extent gives it none; and a
    // product that overflows `u64` is a count no launch could hold. Each is a
    // malformed statement rather than a disagreement with something else, so
    // they share the rule that names the space.
    let Some(participants) = space
        .participants()
        .filter(|_| space.rank() != 0 && space.extents().iter().all(|extent| *extent != 0))
    else {
        return Err(cooperative(CooperativeTileRule::LocalCoordinates));
    };
    // Uniform convergence. Every launched invocation of the workgroup is a
    // participant, so a synchronization point placed in any phase is one they
    // all reach. The extent *product* is what the launch is compared against —
    // the shape the participants are arranged in is a fact the launch geometry
    // does not yet carry, which is why this stays a product equality and why a
    // `[4, 64]` tile and a `[16, 16]` tile are equally admissible against a
    // 256-thread workgroup.
    if participants != u64::from(threads_per_workgroup) {
        return Err(cooperative(CooperativeTileRule::ParticipantConvergence));
    }
    Ok(())
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
    // Ownership is a fact of the topology, not of the mere presence of a tile.
    // [`ReductionTopology::CooperativeWorkgroup`] runs one invocation per
    // (output, participant) pair and one committer writes; the operand-sharing
    // sibling owns one position per invocation. Inferring the first from
    // `cooperative_tile` would silently undersize the operand-sharing write.
    match &region.schedule.reduction {
        ReductionTopology::CooperativeWorkgroup { tile, .. } => {
            let participants = tile.coordinates.participants.participants()?;
            if participants == 0 || !work_items.is_multiple_of(participants) {
                return None;
            }
            Some(work_items / participants)
        }
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::LiveContraction { .. }
        | ReductionTopology::CooperativeContraction { .. } => Some(work_items),
    }
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
            coverage,
            ..
        }
        | ReductionTopology::CooperativeWorkgroup { coverage, .. } => {
            coverage.partition().partitions
        }
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::CooperativeContraction { .. }
        | ReductionTopology::LiveContraction { .. }
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
        // number for every topology that runs one invocation per output. A
        // one-committer cooperative tile's write covers one position per
        // workgroup; the operand-sharing sibling owns one position per
        // invocation. `owned_output_positions` decides from the topology, not
        // from the mere presence of a tile.
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LinearIdentity) => {
            owned_output_positions(region).is_some_and(|owned| *element_count == owned)
        }
        (BoundsProofKind::LinearRange { element_count }, LogicalAccess::LiveRowMajor { .. }) => {
            *element_count == 0
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
        // A live contraction's operand buffers are sized by the live inner
        // extent, which the schedule does not specialize. The proof records
        // that absence as a zero linear range, the same convention
        // `LiveRowMajor` uses. The static `ContractionOperand` arm below still
        // compares a concrete operand product, so a live region cannot inherit
        // that check and silently bake `S`.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ContractionOperand { .. },
        ) if matches!(
            region.schedule.reduction,
            ReductionTopology::LiveContraction { .. }
        ) =>
        {
            *element_count == 0
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
        // Both structural relations prove the same domain a contraction operand
        // does, and for the same reason: the access ranges over its operand's own
        // contiguous element range, and *which* of those positions each iteration
        // coordinate touches is what the map states and what
        // `pointwise_read_map_is_admissible` proved in range. A reindex and a
        // replication differ in how many times a position is touched, which is a
        // fact about the map and not about the domain the proof bounds — so a
        // separate proof structure here would carry no information the map does
        // not already carry.
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ReindexBijection { operand_shape, .. }
            | LogicalAccess::BroadcastReplication { operand_shape, .. },
        ) => super::model::element_count(operand_shape)
            .is_ok_and(|elements| *element_count == elements),
        (
            BoundsProofKind::LinearRange { element_count },
            LogicalAccess::ParametricBroadcast { operand_shape, .. },
        ) => operand_shape.as_static().is_some_and(|shape| {
            super::model::element_count(shape).is_ok_and(|elements| *element_count == elements)
        }),
        _ => false,
    }
}

#[cfg(test)]
mod structural_relation_tests {
    use crate::schedule::{
        AxisDecode, broadcast_decodes_are_replicating, reindex_decodes_are_bijective,
    };
    use crate::shape::Shape;

    /// The two admission rules, tested directly rather than through a program.
    ///
    /// **These predicates are the region verifier's whole defence for the
    /// structural relations, and the compile path cannot exercise their refusing
    /// half.** `BroadcastAxisMapping` and `ReindexForm` already refuse a
    /// non-widening mapping and a non-bijective form at the *semantic* boundary,
    /// so no program the recognizer can build reaches these `false` returns. That
    /// makes them unreachable through the compiler and still load-bearing here:
    /// `tiler-ir` verifies regions from any producer, including one that builds a
    /// `ScheduledRegion` by hand and submits it to `from_region`, and a rule with
    /// no test is a rule that silently stops holding.
    #[test]
    fn the_reindex_rule_admits_a_tiling_and_refuses_everything_else() {
        let operand = Shape::from_dims([2, 3]);
        let transposed = Shape::from_dims([3, 2]);
        // The transposition: operand axis 1 takes result axis 0's window
        // (divisor 2), operand axis 0 takes result axis 1's (divisor 1). Sorted
        // by descending divisor the windows telescope 2*3 == 6 and 1*2 == 2.
        let admitted = vec![AxisDecode::read(1, 2), AxisDecode::read(2, 3)];
        assert!(reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &admitted
        ));
        // Mirroring preserves bijectivity: `c -> modulus - 1 - c` is a bijection
        // of any axis onto itself, so a reversal tiles exactly what its
        // unmirrored twin does.
        let mirrored = vec![
            AxisDecode::read(1, 2),
            AxisDecode {
                divisor: 2,
                modulus: 3,
                mirrored: true,
            },
        ];
        assert!(reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &mirrored
        ));

        // An overlap: both windows claim divisor 1, so two linear coordinates
        // collide on one operand element and the map is not injective.
        let overlapping = vec![AxisDecode::read(1, 2), AxisDecode::read(1, 3)];
        assert!(!reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &overlapping
        ));
        // A gap: the windows are disjoint but leave the coordinate `2..6`
        // unreachable, so the map is injective and not surjective — a slice
        // rather than a reindex.
        let gapped = vec![AxisDecode::read(1, 2), AxisDecode::read(4, 3)];
        assert!(!reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &gapped
        ));
        // **The telescoping rule specifically**, which needs three axes to
        // exercise: with two, a broken tiling always fails the total-window
        // check first. On a `[2, 2, 2]` operand the top window is `4 * 2 == 8`
        // and the bottom divisor is `1`, so both end checks pass — and two axes
        // still claim divisor `1`, which only the telescoping loop detects.
        let cube = Shape::from_dims([2, 2, 2]);
        let untelescoped = vec![
            AxisDecode::read(4, 2),
            AxisDecode::read(1, 2),
            AxisDecode::read(1, 2),
        ];
        assert!(!reindex_decodes_are_bijective(&cube, &cube, &untelescoped));
        // Its admitted neighbour, differing only in the middle window, so the
        // refusal above reads the overlap rather than the shape.
        let telescoped = vec![
            AxisDecode::read(4, 2),
            AxisDecode::read(2, 2),
            AxisDecode::read(1, 2),
        ];
        assert!(reindex_decodes_are_bijective(&cube, &cube, &telescoped));

        // A modulus that is not the operand axis's own extent.
        let wrong_modulus = vec![AxisDecode::read(1, 3), AxisDecode::read(2, 3)];
        assert!(!reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &wrong_modulus
        ));
        // A result domain of a different size cannot be in bijection at all.
        assert!(!reindex_decodes_are_bijective(
            &operand,
            &Shape::from_dims([2, 2]),
            &admitted
        ));
        // One decode per operand axis, never fewer.
        assert!(!reindex_decodes_are_bijective(
            &operand,
            &transposed,
            &admitted[..1]
        ));
    }

    #[test]
    fn the_broadcast_rule_requires_a_real_widening_of_named_result_axes() {
        // A `[2]` weight read across a `[2, 2]` activation: the weight's only
        // axis takes result axis 1's window, and result axis 0 is replicated.
        let operand = Shape::from_dims([2]);
        let widened = Shape::from_dims([2, 2]);
        let admitted = vec![AxisDecode::read(1, 2)];
        assert!(broadcast_decodes_are_replicating(
            &operand, &widened, &admitted
        ));

        // **The widening rule.** A replication that covers the whole result
        // domain is a dense read, and admitting it here would give one region
        // two identities.
        assert!(!broadcast_decodes_are_replicating(
            &operand,
            &Shape::from_dims([2]),
            &admitted
        ));
        // A rank that grew only by an extent-one axis widens nothing either,
        // which is why the rule is stated on element counts rather than ranks.
        assert!(!broadcast_decodes_are_replicating(
            &operand,
            &Shape::from_dims([1, 2]),
            &admitted
        ));
        // A broadcast replicates and never reverses; mirroring belongs to the
        // reindex family, and admitting it here would let one composition be
        // spelled two ways.
        let reversing = vec![AxisDecode {
            divisor: 1,
            modulus: 2,
            mirrored: true,
        }];
        assert!(!broadcast_decodes_are_replicating(
            &operand, &widened, &reversing
        ));
        // A divisor that names no whole result axis is a partial window, which
        // this relation does not admit.
        let partial = vec![AxisDecode::read(3, 2)];
        assert!(!broadcast_decodes_are_replicating(
            &operand, &widened, &partial
        ));
        // Two operand axes may not read one result axis: that is a reindex-style
        // decode of one coordinate into two, not a replication.
        let doubled = vec![AxisDecode::read(1, 2), AxisDecode::read(1, 2)];
        assert!(!broadcast_decodes_are_replicating(
            &Shape::from_dims([2, 2]),
            &Shape::from_dims([2, 2, 2]),
            &doubled
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;
    use std::mem::variant_count;

    use crate::schedule::MAX_COOPERATIVE_PARTICIPANT_RANK;
    use crate::schedule::cooperative::{
        AntiDependencyEdge, CooperativePhase, CooperativeTile, LocalCoordinateSource,
        LocalCoordinates, ParticipantRange, ParticipantSpace, StagedElement, StagedRead,
        StagedSpan, StagedWrite, VisibilityEdge, WorkgroupStaging,
    };
    use crate::schedule::handles::{
        BoundsWitnessId, OwnershipWitnessId, PhaseId, StagingId, SyncPointId,
    };
    use crate::schedule::model::{ContributorOrder, ContributorPartition, LaunchPlan};
    use crate::schedule::numerics::{
        ApproximationEnvelope, ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign,
        NumericalPermission, SubnormalMode, ValueDomainProvenance,
    };
    use crate::schedule::synchronization::{
        FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationPlacement,
        SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
    };
    use crate::schedule::{
        PointwiseBf16ExpressionBuilder, PointwiseF32Expression, PointwiseF32ExpressionBuilder,
    };
    use crate::shape::{Axis, Shape};

    /// Recorded canonical identity of the strict-`f32` pointwise test region.
    ///
    /// The pointwise program is encoded as a typed, framed topological graph,
    /// so its exact operand order, constants, root, and physical `f32` family are all pinned.
    ///
    /// Rebaselined deliberately at the `tiler.schedule.v7` step, which gave the
    /// numerical record its two elementary dimensions — the reciprocal-transform
    /// permission and the approximate-intrinsic envelope — between the
    /// signed-zero permission and the exceptional-value assumptions.
    ///
    /// The `v6` rebaseline recorded the fieldless-input-role step, which removed
    /// the declared-input ordinal payload from fieldless input roles.
    ///
    /// Earlier rebaselines recorded the `tiler.schedule.v4` step, which gave
    /// [`CooperativeTile`] its round count; the `v3` step, which gave
    /// `TensorRole::Input` and `PointwiseF32Node::Input` their input ordinals,
    /// so every input access and bounds proof gained four ordinal bytes and the
    /// input leaf's framed length grew from nine to twenty-one; and before that,
    /// the old `ScalarProgram::MultiplyThenAdd` tag (`0x21`) becoming the exact
    /// `ScalarProgram::PointwiseF32` expression encoding (`0x24`).
    const STRICT_F32_REGION_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc00000010101010101010101010100000000000000060000000101000000003100000000000000060000000101";
    /// Canonical identity of the one-committer `[2, 6] -> [2]` cooperative fixture.
    ///
    /// Captured against the bytes this tree encodes for that fixture so a
    /// later payload move fails this pin rather than only the domain-separator
    /// check. The new topology and binding tags must not appear here.
    const ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e763700000000000000000200000000000000020000000000000003000000000000000202000102000000000000000200000000000000020000000000000006000000000000000100000000000000020000000000000001000000010100000000000300020100000001010000000000000000000000020000000002001200000000000000020000000000000002000000000000000600000000000000010000000000000002000000000000000100000001010000000103001100000000000000020000000003000000000000000222000000000000000100000001017fc0000000000000000000000000001574696c65722e746573742e7374726963742d6633327fc000000101010201010101010101000000000000000600000003010000000035000000000000000300000000000000020100000000000000010000000000000003000000000000000100000000000000010000000001000000000000000300000000000000010000000000000002000000000000000000000000000000000000000300000000000000010000000000000000000000010000000000000001000000000000000000000000000000010000000000000000000000010000000000000000000000000000000300000000000000000000000000000001000000000000000000000001000000000000000000000000000000000000000000000003000000000000000100000000010202010002010000000000000001000000000000000000000000000000030100000000000000000000000000000001000000000000000100000001010301000100000000000000060000000301";

    /// The same region's identity under `tiler.schedule.v6`.
    ///
    /// Retained rather than deleted, because it is what makes the `v7` step's
    /// blast radius a measured fact instead of an assurance: the separator
    /// moves *and* the payload moves by exactly the two inserted
    /// elementary-dimension bytes, so the retained comparison shows the step
    /// changed precisely what its grammar argument claims and nothing else.
    ///
    /// **Rebaselined from the `v5` value at the `v7` step, and the rebaseline is
    /// the point rather than housekeeping.** Carried forward unchanged this
    /// constant would have made the retained comparison a `v7`-against-`v5` one
    /// — a claim about two separator steps combined, which is strictly weaker
    /// than a claim about either: a payload change at one step exactly undone at
    /// the next satisfies it. Moving it to the `v6` value keeps the comparison
    /// proving exactly one step. That discards the `v5` datum deliberately; its
    /// whole content was the earlier step's claim, which the commit that made it
    /// already carries.
    const STRICT_F32_REGION_IDENTITY_HEX_V6: &str = "74696c65722e7363686564756c652e763600000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

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
            NumericalPermission::Forbidden,
            ApproximationEnvelope::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
        )
    }

    fn scale_bias_expression(
        scale_bits: u32,
        bias_bits: u32,
    ) -> super::super::PointwiseF32Expression {
        let mut expression = PointwiseF32ExpressionBuilder::new();
        let input = expression.input(AccessOrdinal::FIRST).unwrap();
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
            NumericalPermission::Forbidden,
            ApproximationEnvelope::Forbidden,
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

    /// Builds the approved `(a * b) + c` region over three input tensors.
    ///
    /// The three reads carry ordinals `0`, `1`, and `2` in access order, one
    /// bounds proof each, and a write of the program output.
    fn three_input_builder(elements: u64) -> ScheduledRegionBuilder {
        let mut expression = PointwiseF32ExpressionBuilder::new();
        let a = expression.input(AccessOrdinal::new(0)).unwrap();
        let b = expression.input(AccessOrdinal::new(1)).unwrap();
        let c = expression.input(AccessOrdinal::new(2)).unwrap();
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
                    tensor: TensorRole::Input,
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
                    tensor: TensorRole::Input,
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

    /// Input roles do not carry program-interface association.
    ///
    /// Reordering otherwise identical input accesses remains intrinsically
    /// well-formed. The compiler owns the checked association between each
    /// exact local access and the declared program input it serves.
    #[test]
    fn input_access_roles_are_fieldless_and_positioned_by_the_access_list() {
        let mut permuted = three_input_builder(4);
        permuted.accesses.swap(0, 1);
        permuted.bounds_proofs.swap(0, 1);
        assert!(permuted.build().is_ok());
        let verified = three_input_builder(4).build().unwrap();
        assert!(
            verified.region().index.accesses[..3]
                .iter()
                .all(|access| access.tensor == TensorRole::Input)
        );
    }

    /// A rank-one reindex over the whole extent, mirrored or not.
    ///
    /// A single decode spanning the domain tiles it, so both spellings are
    /// bijections a pointwise region admits; neither is `LinearIdentity`, and the
    /// two are different relations. Those are the only properties the
    /// repeated-read cases below need.
    fn whole_extent_reindex(elements: u64, mirrored: bool) -> LogicalAccess {
        let shape = crate::shape::Shape::from_dims([elements]);
        LogicalAccess::ReindexBijection {
            operand_shape: shape.clone(),
            result_shape: shape,
            axes: vec![crate::schedule::AxisDecode {
                divisor: 1,
                modulus: elements,
                mirrored,
            }],
        }
    }

    /// One input may be read twice when the two reads address it differently.
    ///
    /// This is the region behind `a * permute(a)`: two expression leaves mean
    /// two different tensors derived from one declared input, so they need two
    /// reads with two relations. Binding one access to both leaves is what made
    /// that program compile as `permute(a) * permute(a)` and return a wrong
    /// tensor, so the admission and its bound are the same rule.
    ///
    /// Local access order is identity-bearing and the compiler binds that order
    /// to its checked request subject. A repeated intermediate remains refused:
    /// the role carries no ordinal, so the attribution that makes the input pair
    /// unambiguous is exactly what it lacks.
    #[test]
    fn one_declared_input_may_be_read_densely_and_through_a_relation() {
        let control = three_input_builder(4).build().unwrap();

        let mut paired = three_input_builder(4);
        paired.accesses[1].tensor = TensorRole::Input;
        paired.accesses[1].map = whole_extent_reindex(4, true);
        paired.bounds_proofs[1].tensor = TensorRole::Input;
        let verified = paired.build().unwrap();
        // Three reads and a write still bind four buffers: a second read of one
        // declared input is a second binding, not a shared one.
        assert_eq!(verified.requirements().buffer_bindings, 4);
        // The pair reaches the encoding, so the region that reads input `0`
        // twice is a different region from the one that reads inputs `0` and
        // `1` — not one region with two spellings.
        assert_ne!(
            verified.canonical_identity().as_bytes(),
            control.canonical_identity().as_bytes()
        );

        let mut reversed = three_input_builder(4);
        reversed.accesses[0].map = whole_extent_reindex(4, true);
        reversed.accesses[1].tensor = TensorRole::Input;
        reversed.bounds_proofs[1].tensor = TensorRole::Input;
        let reversed = reversed.build().unwrap();
        assert_ne!(reversed.canonical_identity(), verified.canonical_identity());

        let mut two_relations = three_input_builder(4);
        two_relations.accesses[0].map = whole_extent_reindex(4, false);
        two_relations.accesses[1].tensor = TensorRole::Input;
        two_relations.accesses[1].map = whole_extent_reindex(4, true);
        two_relations.bounds_proofs[1].tensor = TensorRole::Input;
        assert!(two_relations.build().is_ok());

        let mut two_intermediates = three_input_builder(4);
        for position in 0..2 {
            two_intermediates.accesses[position].tensor = TensorRole::Intermediate;
            two_intermediates.bounds_proofs[position].tensor = TensorRole::Intermediate;
        }
        two_intermediates.accesses[1].map = whole_extent_reindex(4, true);
        assert_eq!(
            two_intermediates.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );
    }

    /// An elementwise region may read one materialized intermediate, and only
    /// one.
    ///
    /// This is the consumer half of a `producer -> intermediate -> epilogue`
    /// chain: the region carries the epilogue's own expression and binds one of
    /// its leaves to a tensor an earlier region wrote. Every other obligation is
    /// discharged exactly as the input-reading control's is — the same bounds
    /// proof, the same ownership proof, the same map — so a widening rather than
    /// a relaxation.
    ///
    /// The two refusals are what the widening must not lose. A second
    /// intermediate read is ambiguous rather than merely unsupported:
    /// `TensorRole::Intermediate` carries no ordinal, so nothing says which
    /// materialization edge each read binds. A read of the program output is
    /// refused for a different reason — a region does not consume what it
    /// publishes — and both report the access-refinement rule.
    #[test]
    fn an_elementwise_region_may_read_one_materialized_intermediate() {
        let control = three_input_builder(4).build().unwrap();

        let mut epilogue = three_input_builder(4);
        epilogue.accesses[0].tensor = TensorRole::Intermediate;
        epilogue.bounds_proofs[0].tensor = TensorRole::Intermediate;
        let verified = epilogue.build().unwrap();
        assert_eq!(verified.requirements().buffer_bindings, 4);
        // The read's boundary role reaches the encoding, so the epilogue and its
        // input-reading control are distinct regions rather than one region with
        // two spellings.
        assert_ne!(
            verified.canonical_identity().as_bytes(),
            control.canonical_identity().as_bytes()
        );

        let mut two_intermediates = three_input_builder(4);
        for position in 0..2 {
            two_intermediates.accesses[position].tensor = TensorRole::Intermediate;
            two_intermediates.bounds_proofs[position].tensor = TensorRole::Intermediate;
        }
        assert_eq!(
            two_intermediates.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        let mut reads_output = three_input_builder(4);
        reads_output.accesses[0].tensor = TensorRole::Output;
        reads_output.bounds_proofs[0].tensor = TensorRole::Output;
        assert_eq!(
            reads_output.build().unwrap_err().diagnostics(),
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
                tensor: TensorRole::Input,
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
                tensor: TensorRole::Input,
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
        let a = expression.input(AccessOrdinal::new(0)).unwrap();
        let b = expression.input(AccessOrdinal::new(1)).unwrap();
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
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
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
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
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
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let inner = builder.add(two, three).unwrap();
            let root = builder.add(input, inner).unwrap();
            identity_with_pointwise_expression(builder.build(root).unwrap())
        };
        assert_ne!(canonical, association);

        let operand_order = {
            let mut builder = PointwiseF32ExpressionBuilder::new();
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
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
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
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
            let input = builder.input(AccessOrdinal::FIRST).unwrap();
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
            NumericalPermission::Forbidden,
            ApproximationEnvelope::Forbidden,
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
    /// test above, which only proves that its eleven realizations differ from
    /// each other.
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

    /// Returns an admitted lane count for the accepted fixed-vector map tests.
    fn admitted_lanes(width: u64) -> super::super::model::VectorLaneCount {
        super::super::model::VectorLaneCount::new(width).expect("an admitted lane width")
    }

    /// Rebinds a builder's schedule to the fixed-vector map with the accepted
    /// launch identity: `work_items` untouched, `grid_threads` as stated.
    fn into_fixed_vector_map(builder: &mut ScheduledRegionBuilder, width: u64, grid_threads: u64) {
        let schedule = builder.schedule.as_mut().expect("schedule was set");
        schedule.binding = ExecutionBinding::FixedVectorMap {
            lanes: admitted_lanes(width),
        };
        schedule.launch.grid_threads = grid_threads;
    }

    /// The accepted exact fixed-vector map admits pointwise work under a fully
    /// strict contract: `work_items = N`, `grid_threads = N / W`, and no
    /// numerical permission is consumed or required by grouping independent
    /// outputs into packets.
    #[test]
    fn the_fixed_vector_map_admits_exact_pointwise_work_under_a_strict_contract() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 2, 3);
        let verified = builder.build().unwrap();
        assert_eq!(verified.region().schedule.work_items, 6);
        assert_eq!(verified.region().schedule.launch.grid_threads, 3);
        // The ownership population stays the scalar-output population: packet
        // `p`, lane `l` is the one owning invocation of output `2p + l`.
        assert_eq!(
            verified.region().index.ownership_proof.kind,
            OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 6 }
        );
        // The strict contract passes through untouched — the admission read
        // none of it, so nothing was consumed, relaxed, or newly required.
        let requirements = verified.requirements();
        assert_eq!(requirements.contraction, NumericalPermission::Forbidden);
        assert_eq!(requirements.reassociation, NumericalPermission::Forbidden);
        assert_eq!(requirements.permutation, NumericalPermission::Forbidden);
        assert_eq!(requirements.signed_zero, NumericalPermission::Forbidden);
    }

    /// The strict serial fold across independent outputs is the second and
    /// last admitted pairing: the fold inside each output stays serial and
    /// order-preserving, and the lanes group only the independent outputs.
    #[test]
    fn the_fixed_vector_map_admits_the_strict_serial_fold_across_independent_outputs() {
        let mut builder = serial_reduction_builder(ScalarProgram::StrictSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0,
        });
        into_fixed_vector_map(&mut builder, 2, 1);
        let verified = builder.build().unwrap();
        assert_eq!(verified.region().schedule.work_items, 2);
        assert_eq!(verified.region().schedule.launch.grid_threads, 1);
        assert_eq!(
            verified.requirements().reassociation,
            NumericalPermission::Forbidden
        );
        assert_eq!(
            verified.requirements().permutation,
            NumericalPermission::Forbidden
        );
    }

    /// Lane counts zero and one are refused at construction, each under its
    /// own name: invalidity and the duplicate scalar spelling are different
    /// refusals a producer must be able to tell apart.
    #[test]
    fn lane_counts_zero_and_one_are_refused_at_construction_by_name() {
        use crate::schedule::error::VectorLaneCountError;

        let zero = super::super::model::VectorLaneCount::new(0).unwrap_err();
        assert_eq!(zero, VectorLaneCountError::Zero);
        assert_eq!(zero.rule(), "vector-lane-count-zero");
        let one = super::super::model::VectorLaneCount::new(1).unwrap_err();
        assert_eq!(one, VectorLaneCountError::ScalarSpelling);
        assert_eq!(one.rule(), "vector-lane-count-scalar-spelling");
        assert_eq!(
            super::super::model::VectorLaneCount::new(2).unwrap().get(),
            2
        );
    }

    /// `N mod W != 0` is refused by its own rule: the verifier never rounds
    /// the iteration count, masks implicitly, or peels a scalar tail.
    #[test]
    fn a_nondivisible_fixed_vector_domain_is_refused_by_name() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 4, 1);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::VectorLaneBinding {
                rule: VectorLaneRule::NondivisibleCoverage
            }]
        );
    }

    /// `grid_threads = N` with a reinterpreted builtin is exactly the launch
    /// identity the acceptance forbids, and it is refused as a wrong packet
    /// population rather than admitted for an emitter to reinterpret.
    #[test]
    fn a_fixed_vector_launch_keeping_the_scalar_grid_is_refused() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 2, 6);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::VectorLaneBinding {
                rule: VectorLaneRule::PacketPopulation
            }]
        );
    }

    /// An overflowing packet product is a product that does not exist, named
    /// apart from a wrong packet count.
    #[test]
    fn overflowing_fixed_vector_packet_arithmetic_is_refused_by_name() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 2, u64::MAX);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::VectorLaneBinding {
                rule: VectorLaneRule::PacketArithmeticOverflow
            }]
        );
    }

    /// A wrong output-owner population keeps its existing independent refusal:
    /// the ownership proof must still cover exactly the `N` scalar outputs.
    #[test]
    fn a_fixed_vector_region_with_a_wrong_owner_population_is_refused() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 2, 3);
        builder
            .ownership_proof
            .as_mut()
            .expect("proof was set")
            .kind = OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 3 };
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ProofReference]
        );
    }

    /// Every unadmitted reduction/binding pairing is one refusal, reached
    /// independently of coverage and launch arithmetic.
    #[test]
    fn an_unsupported_fixed_vector_reduction_pairing_is_refused_by_name() {
        let mut builder = contraction_builder();
        into_fixed_vector_map(&mut builder, 2, 2);
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::VectorLaneBinding {
                rule: VectorLaneRule::UnsupportedReduction
            }]
        );
    }

    /// The fixed-vector map admits `Exact` alone; a predicated tail is refused
    /// under the binding's own rule rather than as a launch-coverage failure.
    #[test]
    fn a_non_exact_fixed_vector_tail_is_refused_by_name() {
        let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
        into_fixed_vector_map(&mut builder, 2, 3);
        builder.schedule.as_mut().expect("schedule was set").tail = TailPolicy::Predicated;
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::VectorLaneBinding {
                rule: VectorLaneRule::ExactTailRequired
            }]
        );
    }

    /// Perturbing the binding tag and the lane count each separates canonical
    /// identity, isolated on a zero domain where every other schedule byte —
    /// including the packet population — is identical.
    #[test]
    fn the_fixed_vector_binding_tag_and_lane_count_separate_identity() {
        let scalar = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0)
            .build()
            .unwrap();
        let mut two = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0);
        into_fixed_vector_map(&mut two, 2, 0);
        let two = two.build().unwrap();
        let mut three = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0);
        into_fixed_vector_map(&mut three, 3, 0);
        let three = three.build().unwrap();

        // Binding tag: the vector region differs from the scalar one although
        // work items, launch, accesses, program, and contract are all equal.
        assert_ne!(
            scalar.canonical_identity().as_bytes(),
            two.canonical_identity().as_bytes()
        );
        // Lane count: two widths are two programs and never share bytes.
        assert_ne!(
            two.canonical_identity().as_bytes(),
            three.canonical_identity().as_bytes()
        );
        // The appended arm is a tag plus a fixed-width count: the vector
        // encoding is exactly eight bytes (the lane count) longer than the
        // scalar one, so the widening moved no earlier field.
        assert_eq!(
            two.canonical_identity().as_bytes().len(),
            scalar.canonical_identity().as_bytes().len() + 8
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
                    coverage: ContributorCoverage::Exact(partition),
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

    /// Builds a `[2, 6] -> [2]` serial reduction over input zero.
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
                tensor: TensorRole::Input,
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
                tensor: TensorRole::Input,
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

    /// Builds a valid `mk,nk->mn` contraction over the named program inputs.
    fn contraction_builder() -> ScheduledRegionBuilder {
        let operand = Shape::from_dims([2, 3]);
        let output = Shape::from_dims([2, 2]);
        let contracted = Shape::from_dims([3]);
        let left = TensorRole::Input;
        let right = TensorRole::Input;
        let operand_map = |free_position| LogicalAccess::ContractionOperand {
            operand_shape: operand.clone(),
            output_shape: output.clone(),
            contracted_shape: contracted.clone(),
            sources: vec![
                ContractionAxisSource::Output {
                    position: free_position,
                },
                ContractionAxisSource::Contracted { position: 0 },
            ],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(42));
        builder.iteration_shape(output.clone()).unwrap();
        for (witness, tensor, map) in [(0, left, operand_map(0)), (1, right, operand_map(1))] {
            builder
                .push_access(Access {
                    tensor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map,
                    bounds: BoundsWitnessId::new(witness),
                    ownership: None,
                })
                .unwrap();
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(witness),
                    tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange { element_count: 6 },
                })
                .unwrap();
        }
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(2),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(2),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 4 },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            })
            .unwrap();
        builder.numerical(strict_numerical()).unwrap();
        builder
            .schedule(KernelSchedule {
                reduction: ReductionTopology::Contraction {
                    contracted_shape: contracted,
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                ..linear_schedule(4, OwnershipWitnessId::new(0))
            })
            .unwrap();
        builder
    }

    /// Contraction inputs are distinguished by exact access position, not role payload.
    #[test]
    fn contraction_inputs_are_distinguished_by_access_position() {
        let verified = contraction_builder().build().unwrap();
        assert_eq!(
            verified.region().index.accesses[0].tensor,
            TensorRole::Input
        );
        assert_eq!(
            verified.region().index.accesses[1].tensor,
            TensorRole::Input
        );
    }

    fn live_contraction_builder(
        live_access: u32,
        live_axis: u32,
        output: [u64; 2],
    ) -> ScheduledRegionBuilder {
        let left_shape = Shape::from_dims([output[0]]);
        let right_shape = Shape::from_dims([output[1]]);
        let output_shape = Shape::from_dims(output);
        let contracted = Shape::from_dims([]);
        let left = TensorRole::Input;
        let right = TensorRole::Input;
        let owner = OwnershipWitnessId::new(0);
        let output_elements = element_count(&output_shape).unwrap_or(0);
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(42));
        builder.iteration_shape(output_shape.clone()).unwrap();
        for (witness, tensor, operand, free) in
            [(0, left, left_shape, 0_u32), (1, right, right_shape, 1)]
        {
            builder
                .push_access(Access {
                    tensor,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ContractionOperand {
                        operand_shape: operand,
                        output_shape: output_shape.clone(),
                        contracted_shape: contracted.clone(),
                        sources: vec![ContractionAxisSource::Output { position: free }],
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(witness),
                    ownership: None,
                })
                .unwrap();
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(witness),
                    tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange { element_count: 0 },
                })
                .unwrap();
        }
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(2),
                ownership: Some(owner),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(2),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: output_elements,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: owner,
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: output_elements,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            })
            .unwrap();
        builder.numerical(strict_numerical()).unwrap();
        builder
            .schedule(KernelSchedule {
                reduction: ReductionTopology::LiveContraction {
                    live_access: AccessOrdinal::new(live_access),
                    live_axis: Axis::new(live_axis),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    permits_reassociation: false,
                    permits_permutation: false,
                },
                ..linear_schedule(output_elements, owner)
            })
            .unwrap();
        builder
    }

    /// A well-formed live contraction is schedule-verified; a swapped live
    /// axis is not.
    #[test]
    fn a_live_contraction_admits_the_named_inner_axis_and_refuses_a_swapped_symbol() {
        let verified = live_contraction_builder(0, 1, [2, 3])
            .build()
            .expect("the named inner axis of input 0 is the live contracted bound");
        assert!(matches!(
            verified.region().schedule.reduction,
            ReductionTopology::LiveContraction {
                live_axis,
                ..
            } if live_axis == Axis::new(1)
        ));

        let swapped = live_contraction_builder(0, 0, [2, 3])
            .build()
            .expect_err("naming the free axis as the live bound must fail");
        assert_eq!(
            swapped.diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "swapped-symbol live axis: {swapped}"
        );
        assert_eq!(
            swapped.diagnostics()[0].rule(),
            "numerical-or-access-refinement"
        );
    }

    /// An axis the named input does not have is refused at schedule verification.
    #[test]
    fn a_live_contraction_refuses_a_wrong_live_axis() {
        let error = live_contraction_builder(0, 5, [2, 3])
            .build()
            .expect_err("axis 5 is outside the live input's rank");
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "wrong-axis live contraction: {error}"
        );
        assert_eq!(
            error.diagnostics()[0].rule(),
            "numerical-or-access-refinement"
        );
    }

    /// An overflowing static output product is refused by name.
    #[test]
    fn a_live_contraction_refuses_an_overflowing_output_product() {
        let error = live_contraction_builder(0, 1, [u64::MAX, 2])
            .build()
            .expect_err("a [u64::MAX, 2] output product must overflow");
        assert_eq!(
            error.diagnostics(),
            [ScheduledRegionDiagnostic::ShapeProductOverflow],
            "overflowing live contraction: {error}"
        );
        assert_eq!(error.diagnostics()[0].rule(), "shape-product-overflow");
    }

    /// The scale a root-mean-square normalization's producing stage computes.
    ///
    /// `Rsqrt(a / N + eps)` over the fold's value, which is local access zero.
    /// The shipped instance of a fold epilogue, spelled here from the physical
    /// vocabulary rather than from any law: what this module verifies is the
    /// *schedule*, and it has no opinion on which semantic operation the chain
    /// realizes.
    fn scale_epilogue() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let total = builder.input(AccessOrdinal::FIRST).unwrap();
        let extent = builder.constant(6.0_f32.to_bits()).unwrap();
        let mean = builder.divide(total, extent).unwrap();
        let bias = builder.constant(1.0e-6_f32.to_bits()).unwrap();
        let biased = builder.add(mean, bias).unwrap();
        let root = builder.rsqrt(biased).unwrap();
        builder.build(root).unwrap()
    }

    /// The squaring fold carrying that epilogue, over the shared fixture shape.
    fn squared_sum_with_epilogue(epilogue: PointwiseF32Expression) -> ScalarProgram {
        ScalarProgram::SquaredSerialSumThenEpilogue {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
            epilogue,
        }
    }

    /// A squaring fold carrying an epilogue verifies as a serial pass.
    ///
    /// The control every refusal below is stated against: the fold's own
    /// obligations are the squaring sum's, unchanged, and the epilogue adds two
    /// of its own without changing what the region reads or writes — one read of
    /// the contributor domain, one owning write.
    #[test]
    fn a_fold_carrying_an_epilogue_verifies_as_a_serial_pass() {
        let region = serial_reduction_builder(squared_sum_with_epilogue(scale_epilogue()))
            .build()
            .expect("a squaring fold with a scalar epilogue verifies");
        assert!(matches!(
            region.region().index.scalar_program,
            ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        ));
        // One read and one write, exactly as the bare fold declares: the
        // epilogue's leaf is the folded value, so it binds no buffer.
        assert_eq!(region.region().index.accesses.len(), 2);
        assert_eq!(region.requirements().buffer_bindings, 2);
    }

    /// An epilogue that computes nothing is refused rather than admitted.
    ///
    /// **The canonicality rule this variant owes.** An expression whose root is
    /// its own input leaf returns the fold's value unchanged, which is exactly
    /// what [`ScalarProgram::SquaredSerialSum`] computes — so admitting it would
    /// give one program two spellings and two canonical identities, and a cache
    /// holding either would miss the other for the same computation.
    #[test]
    fn a_fold_epilogue_that_computes_nothing_is_refused() {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let leaf = builder.input(AccessOrdinal::FIRST).unwrap();
        let identity = builder.build(leaf).unwrap();
        assert_eq!(
            serial_reduction_builder(squared_sum_with_epilogue(identity))
                .build()
                .unwrap_err()
                .diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        );
    }

    /// An epilogue naming a second input is refused rather than bound.
    ///
    /// A fold region reads exactly one boundary tensor, and the epilogue's sole
    /// ordinal is the folded value rather than a buffer — so a second leaf names
    /// an input nothing binds, and the lowering would have no value to supply for
    /// it. Refusing it here is what keeps that from being a handle error deep in
    /// the kernel builder.
    #[test]
    fn a_fold_epilogue_reading_a_second_input_is_refused() {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let total = builder.input(AccessOrdinal::FIRST).unwrap();
        let other = builder.input(AccessOrdinal::new(1)).unwrap();
        let sum = builder.add(total, other).unwrap();
        let two_leaves = builder.build(sum).unwrap();
        assert_eq!(two_leaves.input_count(), 2);
        assert_eq!(
            serial_reduction_builder(squared_sum_with_epilogue(two_leaves))
                .build()
                .unwrap_err()
                .diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        );
    }

    /// A fold carrying an epilogue does not share identity with the bare fold.
    ///
    /// The two regions differ in nothing but their scalar program — same access
    /// relation, same contributor order, same numerical realization — so an
    /// appended tag that had collided with `0x26` would make these equal. It is
    /// the check behind "the schedule domain did not step": the new tag
    /// separates, and every earlier tag keeps its meaning.
    ///
    /// The second pair separates two *epilogues*: a chain dividing by six and one
    /// dividing by seven are different functions, so the expression payload has
    /// to reach the identity bytes rather than only the tag.
    #[test]
    fn a_fold_epilogue_separates_scheduled_region_identity() {
        let bare = serial_reduction_builder(ScalarProgram::SquaredSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        })
        .build()
        .unwrap();
        let scaled = serial_reduction_builder(squared_sum_with_epilogue(scale_epilogue()))
            .build()
            .unwrap();
        assert_ne!(
            bare.canonical_identity().as_bytes(),
            scaled.canonical_identity().as_bytes(),
        );

        let mut other = PointwiseF32ExpressionBuilder::new();
        let total = other.input(AccessOrdinal::FIRST).unwrap();
        let extent = other.constant(7.0_f32.to_bits()).unwrap();
        let mean = other.divide(total, extent).unwrap();
        let bias = other.constant(1.0e-6_f32.to_bits()).unwrap();
        let biased = other.add(mean, bias).unwrap();
        let root = other.rsqrt(biased).unwrap();
        let seventh =
            serial_reduction_builder(squared_sum_with_epilogue(other.build(root).unwrap()))
                .build()
                .unwrap();
        assert_ne!(
            scaled.canonical_identity().as_bytes(),
            seventh.canonical_identity().as_bytes(),
        );
    }

    /// No parallel topology may split a fold that carries an epilogue.
    ///
    /// **The refusal is the family's algebra rather than caution.** The epilogue
    /// applies to the *complete* fold, so a partial pass applying it would
    /// transform a fragment and one that did not would be computing
    /// [`ScalarProgram::SquaredSerialSum`] under this variant's name. Both split
    /// admissions therefore answer `None` for it, and the topology is refused at
    /// the same rule an unadmitted family is.
    #[test]
    fn a_fold_carrying_an_epilogue_admits_no_parallel_topology() {
        let scalar = squared_sum_with_epilogue(scale_epilogue());
        let family = split_family(&scalar).expect("the serial family is derived");
        assert_eq!(family.parallel, ParallelFamily::SerialOnly);
        assert!(
            family
                .read_tensor(FamilyTopology::MultiPass(ReductionPass::Partial))
                .is_none()
        );
        assert!(
            family
                .read_tensor(FamilyTopology::MultiPass(ReductionPass::Final))
                .is_none()
        );
        assert!(family.read_tensor(FamilyTopology::Cooperative).is_none());

        // Stated against a partial pass that is otherwise *correct*: the fixture
        // is the squaring fold's own verified partial pass with its scalar
        // program exchanged for the epilogue-carrying one, so the family is the
        // only difference between an admitted region and this refusal.
        let mut split = squared_partial_pass_builder(SPLIT);
        split.scalar_program = Some(squared_sum_with_epilogue(scale_epilogue()));
        assert_eq!(
            split.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a fold whose epilogue applies to the whole value has no partial pass",
        );
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

    /// A topology that describes no fold is refused for the extrema family.
    ///
    /// The parallel topologies are admitted (below); these two are not, and the
    /// reasons are different in kind. [`ReductionTopology::None`] says the region
    /// performs no reduction, which contradicts a scalar program that is one.
    /// [`ReductionTopology::Contraction`] folds a *contracted index space* stated
    /// by the topology, which a one-tensor reduction access does not have.
    /// Neither is a conservative refusal waiting to be widened.
    #[test]
    fn a_topology_that_describes_no_fold_is_refused_for_the_extrema_family() {
        let mut none = serial_reduction_builder(maximum_scalar());
        none.schedule.as_mut().unwrap().reduction = ReductionTopology::None;
        assert_eq!(
            none.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        let mut contraction = serial_reduction_builder(maximum_scalar());
        contraction.schedule.as_mut().unwrap().reduction = ReductionTopology::Contraction {
            contracted_shape: Shape::from_dims([6]),
            order: ContributorOrder::OriginalAxisLexicographic,
            permits_reassociation: false,
            permits_permutation: false,
        };
        assert_eq!(
            contraction.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        // The control: the unmodified serial fixture verifies, so the refusals
        // above are about the topology rather than about the fixture.
        assert!(serial_reduction_builder(maximum_scalar()).build().is_ok());
    }

    /// A split that covers no contributor, for the empty-domain fixtures below.
    ///
    /// `partitions` stays nonzero — [`ContributorPartition::covers`] refuses a
    /// zero partition count outright — so the empty case is expressed by the
    /// per-partition width alone, which is exactly the shape an identity-seeded
    /// family is allowed to have and an identity-less one is not.
    const EMPTY_SPLIT: ContributorPartition = ContributorPartition {
        partitions: 3,
        contributors_per_partition: 0,
    };

    /// Rewrites one pass of a sum split into the extrema fold's own.
    ///
    /// The three edits are the whole difference, and each is load-bearing: the
    /// read binds the original scores where a sum's partial pass binds an
    /// intermediate, the program is the identity-less fold, and the realization
    /// is the *strict* one — reassociation forbidden — because a split of this
    /// family spends no permission. A fixture that relaxed the contract would
    /// prove the topology admissible without proving the interesting half of it.
    fn into_extrema_split(builder: &mut ScheduledRegionBuilder, axes: Vec<Axis>, read: TensorRole) {
        builder.accesses[0].tensor = read;
        builder.bounds_proofs[0].tensor = read;
        builder.scalar_program = Some(ScalarProgram::StrictSerialMaximum {
            axes,
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
        });
        builder.numerical = Some(strict_numerical());
        let Some(ReductionTopology::MultiPass {
            permits_reassociation,
            ..
        }) = builder
            .schedule
            .as_mut()
            .map(|schedule| &mut schedule.reduction)
        else {
            panic!("the fixture schedules a multi-pass split")
        };
        *permits_reassociation = false;
    }

    /// The partial pass of an extrema split: fold the scores, stage one maximum.
    fn extrema_partial_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
        let mut builder = partial_pass_builder(partition);
        into_extrema_split(&mut builder, vec![Axis::new(1)], TensorRole::Input);
        builder
    }

    /// The final pass of an extrema split: fold the staged maxima into the result.
    fn extrema_final_builder(partition: ContributorPartition) -> ScheduledRegionBuilder {
        let axes = vec![partial_reduction_axis(&Shape::from_dims([2])).expect("rank one fits u32")];
        let mut builder = final_pass_builder(partition);
        into_extrema_split(&mut builder, axes, TensorRole::Intermediate);
        builder
    }

    /// The cooperative tile over the extrema fold, under a strict contract.
    fn extrema_cooperative_builder() -> ScheduledRegionBuilder {
        let ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            arrival,
            ..
        } = cooperative_topology(cooperative_tile_fixture())
        else {
            panic!("the cooperative fixture builds a cooperative topology")
        };
        let mut builder = cooperative_builder_parts(
            SPLIT,
            6,
            ReductionTopology::CooperativeWorkgroup {
                coverage,
                tile,
                axes,
                order,
                accumulation,
                permits_reassociation: false,
                permits_permutation: false,
                arrival,
            },
            strict_numerical(),
        );
        builder.accesses[0].tensor = TensorRole::Input;
        builder.bounds_proofs[0].tensor = TensorRole::Input;
        builder.scalar_program = Some(maximum_scalar());
        builder
    }

    /// Both passes of an extrema split verify under a contract that forbids
    /// reassociation, and the same split of a *sum* still refuses.
    ///
    /// **This is the asymmetry the softmax's two passes owe.** The pinned extrema
    /// family is associative and commutative on every binary32 input, so a split
    /// of it changes no observable value and spends no permission —
    /// `SOFTMAX_F32_FACT_MAXIMUM_FOLD_LEGALITY`. The denominator's sum is the
    /// other fact, `SOFTMAX_F32_FACT_SUM_FOLD_ORDER`, and the control here is
    /// what keeps the widening from reading as a relaxation of both: the same
    /// split shape, the same strict realization, the same fixture — and the sum
    /// is refused.
    #[test]
    fn an_extrema_split_verifies_under_a_strict_contract_and_a_sum_split_does_not() {
        let partial = extrema_partial_builder(SPLIT)
            .build()
            .expect("an extrema partial pass verifies under a strict contract");
        let combine = extrema_final_builder(SPLIT)
            .build()
            .expect("an extrema final pass verifies under a strict contract");
        assert_eq!(partial.region().schedule.work_items, 6);
        assert_eq!(combine.region().schedule.work_items, 2);
        // The split is admitted without spending anything, which is the claim.
        assert_eq!(
            partial.requirements().reassociation,
            NumericalPermission::Forbidden
        );
        assert_eq!(
            partial.requirements().permutation,
            NumericalPermission::Forbidden
        );

        // The perturbation that fires: the same split of the sum, under the same
        // strict realization, is still refused.
        let mut summed = partial_pass_builder(SPLIT);
        summed.numerical = Some(strict_numerical());
        let Some(ReductionTopology::MultiPass {
            permits_reassociation,
            ..
        }) = summed
            .schedule
            .as_mut()
            .map(|schedule| &mut schedule.reduction)
        else {
            panic!("the fixture schedules a multi-pass split")
        };
        *permits_reassociation = false;
        assert_eq!(
            summed.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "splitting an ordered sum still consumes the reassociation permission"
        );
    }

    /// A cooperative tile over the extrema fold verifies under a strict contract.
    ///
    /// The same claim as the split's, at the topology whose partials never leave
    /// the workgroup: the staged fold seeds at the first slot, which is
    /// admissible for an identity-less family exactly because the tile's staging
    /// coverage and the exact launch prove every slot was written by a
    /// participant that folded at least one contributor.
    #[test]
    fn a_cooperative_extrema_tile_verifies_under_a_strict_contract() {
        let verified = extrema_cooperative_builder()
            .build()
            .expect("an extrema tile verifies under a strict contract");
        assert_eq!(verified.requirements().local_memory_bytes, 12);
        assert_eq!(
            verified.requirements().reassociation,
            NumericalPermission::Forbidden
        );

        // The control: the fixture's own sum, under the same strict realization
        // and the same tile, is refused.
        let ReductionTopology::CooperativeWorkgroup {
            coverage,
            tile,
            axes,
            order,
            accumulation,
            arrival,
            ..
        } = cooperative_topology(cooperative_tile_fixture())
        else {
            panic!("the cooperative fixture builds a cooperative topology")
        };
        let summed = cooperative_builder_parts(
            SPLIT,
            6,
            ReductionTopology::CooperativeWorkgroup {
                coverage,
                tile,
                axes,
                order,
                accumulation,
                permits_reassociation: false,
                permits_permutation: false,
                arrival,
            },
            strict_numerical(),
        );
        assert_eq!(
            summed.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a tile over an ordered sum still consumes the reassociation permission"
        );
    }

    /// A split of the identity-less fold is refused over an empty domain.
    ///
    /// The obligation that replaces the empty-domain constant the family has no
    /// correct value for, checked where a split could otherwise hide it: a
    /// partition covering no contributor has nothing to stage. The control is the
    /// *same* split under the bare sum, which verifies because that family
    /// commits `+0.0` — so the refusal is about the family and not about the
    /// zero extent or the zero-width partition.
    #[test]
    fn an_empty_split_is_refused_for_the_identity_less_fold() {
        let empty_input = Shape::from_dims([2, 0]);
        let empty = |builder: &mut ScheduledRegionBuilder| {
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

        let mut maximum = extrema_partial_builder(EMPTY_SPLIT);
        empty(&mut maximum);
        assert_eq!(
            maximum.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "no partition of an identity-less fold may cover nothing"
        );

        let mut bare = partial_pass_builder(EMPTY_SPLIT);
        empty(&mut bare);
        assert!(bare.build().is_ok());
    }

    /// An extrema partial pass respelled as a sum verifies as *that sum*.
    ///
    /// **This assertion inverted when a bare fold gained its declared input, and
    /// the inversion narrows what the intrinsic verifier claims rather than losing
    /// a check.** It was previously refused because every sum admitted as a partial
    /// pass had to read an intermediate; a bare sum now folds whichever boundary
    /// tensor holds its declared contributor domain, so this region is a coherent
    /// partial pass of a prologue-less sum — the same accesses, the same split, a
    /// different fold. That it was *authored* as an extrema pass is not a fact the
    /// region carries: which occurrences a region claims is the compiler's subject
    /// binding, and an intrinsic rule guessing intent from the read would have to
    /// refuse the legal program this widening exists to admit.
    ///
    /// What still separates the two spellings is identity, asserted here beside
    /// the admission so they can never be interchanged downstream.
    #[test]
    fn an_extrema_partial_pass_respelled_as_a_sum_verifies_as_that_sum() {
        let mut summed = extrema_partial_builder(SPLIT);
        summed.numerical = Some(reassociating_numerical());
        let Some(ReductionTopology::MultiPass {
            permits_reassociation,
            ..
        }) = summed
            .schedule
            .as_mut()
            .map(|schedule| &mut schedule.reduction)
        else {
            panic!("the fixture schedules a multi-pass split")
        };
        // Reassociation permitted, because a split of a sum consumes it where a
        // split of the extrema fold does not: without it the region would be
        // refused for the permission and say nothing about the boundary role.
        *permits_reassociation = true;
        summed.scalar_program = Some(bare_sum(vec![Axis::new(1)]));
        let summed = summed
            .build()
            .expect("a bare sum folding the first input is a coherent partial pass");

        // The control: the extrema program over the identical region verifies,
        // and the two are not one region under two names.
        let extrema = extrema_partial_builder(SPLIT)
            .build()
            .expect("the extrema partial pass verifies");
        assert_ne!(summed.canonical_identity(), extrema.canonical_identity());
    }

    /// A split extrema region shares identity with neither neighbour.
    ///
    /// The concrete form of the step verdict. Admitting the parallel topologies
    /// introduced no tag and moved no field: an extrema split encodes under the
    /// scalar-program tag `0x28` and the topology tag `0x33`, both already in
    /// their existing positions, and the pair was simply unreachable before. So
    /// no previously encodable region's bytes moved — which
    /// `the_strict_f32_region_has_its_recorded_canonical_identity` pins — while
    /// the newly reachable regions still separate from every neighbour they could
    /// be confused with: the same fold serially, the same split summed, and the
    /// other pass of their own split.
    #[test]
    fn a_split_extrema_region_has_its_own_canonical_identity() {
        let partial = extrema_partial_builder(SPLIT).build().unwrap();
        let combine = extrema_final_builder(SPLIT).build().unwrap();
        let serial = serial_reduction_builder(maximum_scalar()).build().unwrap();
        let summed = partial_pass_builder(SPLIT).build().unwrap();
        let tile = extrema_cooperative_builder().build().unwrap();
        let identities = [
            partial.canonical_identity(),
            combine.canonical_identity(),
            serial.canonical_identity(),
            summed.canonical_identity(),
            tile.canonical_identity(),
        ];
        for (position, identity) in identities.iter().enumerate() {
            assert!(
                !identities[..position].contains(identity),
                "identity {position} collided with an earlier region"
            );
        }
    }

    /// The pinned NaN-propagating extrema family, restated for this test.
    ///
    /// `maximum_f32` in `crates/tiler-reference/src/softmax.rs` is the authority
    /// and evaluates it for the registered operation; this crate cannot call it,
    /// because `tiler-reference` depends on `tiler-ir` and not the other way
    /// round. So the schedule-level evidence restates the two rules that make the
    /// family what it is — NaN is absorbing, and `-0.0 < +0.0` is a total order —
    /// and the control in
    /// [`a_split_of_the_extrema_fold_agrees_with_the_serial_fold_bit_for_bit`]
    /// fails if this is `maxNum` (Rust's `f32::max`) instead, which is the other
    /// ADR 0023 family and the one a careless restatement would land on.
    fn maximum_f32(left: f32, right: f32) -> f32 {
        if left.is_nan() || right.is_nan() {
            return f32::NAN;
        }
        #[allow(
            clippy::float_cmp,
            reason = "the extrema family is defined by exact IEEE-754 comparison"
        )]
        let equal = left == right;
        if equal {
            // Equal under IEEE comparison means two identical values or the pair
            // `(-0.0, +0.0)` in some order. The bitwise `and` selects `+0.0` for
            // the second without branching on which side it arrived from, and is
            // the identity for the first.
            return f32::from_bits(left.to_bits() & right.to_bits());
        }
        if left > right { left } else { right }
    }

    /// The operands at which associativity could fail, and nothing else.
    const EXTREMA_CORPUS: [f32; 7] = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::NAN,
    ];

    /// Folds one contiguous run left to right, as the emitted serial loop does.
    fn fold(values: &[f32], combine: fn(f32, f32) -> f32) -> f32 {
        values
            .iter()
            .copied()
            .reduce(combine)
            .expect("every fold below is over a non-empty run")
    }

    /// Folds a sequence through the partition boundaries a split declares.
    fn fold_split(values: &[f32], width: usize, combine: fn(f32, f32) -> f32) -> f32 {
        let partials: Vec<f32> = values
            .chunks(width)
            .map(|partition| fold(partition, combine))
            .collect();
        fold(&partials, combine)
    }

    /// The split and the serial fold agree bit for bit on every corpus sequence.
    ///
    /// The legality claim executed at the schedule level, over the split a
    /// *verified* region declares rather than one this test invents: the
    /// partition width comes back out of the built region's topology, so a change
    /// to what the verifier admits changes what this folds. Every assignment of
    /// the corpus to the six contributor positions is enumerated, which is
    /// exhaustive over the operands the property could fail at.
    ///
    /// Two controls, because the agreement is worth nothing without them. The
    /// *same* split boundaries applied to an ordered sum change its bits, so the
    /// split shape is one a reassociation difference can travel through; and the
    /// family restated here is not `f32::max`, so the agreement is this family's
    /// rather than any maximum's.
    #[test]
    fn a_split_of_the_extrema_fold_agrees_with_the_serial_fold_bit_for_bit() {
        let verified = extrema_partial_builder(SPLIT).build().unwrap();
        let ReductionTopology::MultiPass { coverage, .. } = verified.region().schedule.reduction
        else {
            panic!("the extrema partial fixture schedules a multi-pass split")
        };
        let partition = coverage.partition();
        let width = usize::try_from(partition.contributors_per_partition)
            .expect("the fixture's partition width fits usize");
        let contributors = usize::try_from(
            partition
                .total_contributors()
                .expect("the fixture's split does not overflow"),
        )
        .expect("the fixture's contributor count fits usize");

        let mut sequence = vec![0.0_f32; contributors];
        let corpus = EXTREMA_CORPUS.len();
        for encoded in 0..corpus.pow(u32::try_from(contributors).expect("six fits u32")) {
            let mut remaining = encoded;
            for slot in &mut sequence {
                *slot = EXTREMA_CORPUS[remaining % corpus];
                remaining /= corpus;
            }
            assert_eq!(
                fold_split(&sequence, width, maximum_f32).to_bits(),
                fold(&sequence, maximum_f32).to_bits(),
                "the split disagrees with the serial fold at {sequence:?}"
            );
        }

        // The first control. `1.0 + 2^-24` rounds back to `1.0` under
        // ties-to-even, so the serial fold absorbs every addend and returns
        // `1.0`; the split adds the small terms to each other first, where they
        // are exact, and the partials then reach the result. The corpus above
        // cannot show this — every one of its values is exact under addition —
        // so the control needs its own sequence rather than a search.
        let half_ulp = f32::EPSILON / 2.0;
        let absorbing = vec![1.0_f32, half_ulp, half_ulp, half_ulp, half_ulp, half_ulp];
        assert_eq!(absorbing.len(), contributors);
        let add = |left: f32, right: f32| left + right;
        assert_eq!(fold(&absorbing, add).to_bits(), 1.0_f32.to_bits());
        assert_ne!(
            fold_split(&absorbing, width, add).to_bits(),
            fold(&absorbing, add).to_bits(),
            "these split boundaries cannot expose a reassociation difference at all"
        );

        // The second control: the family folded above is the NaN-propagating one
        // and not `maxNum`, which returns the number beside a NaN.
        assert!(
            EXTREMA_CORPUS
                .iter()
                .any(
                    |left| EXTREMA_CORPUS
                        .iter()
                        .any(|right| maximum_f32(*left, *right).to_bits()
                            != left.max(*right).to_bits())
                ),
            "the family folded here is indistinguishable from `maxNum` on this corpus"
        );
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
                    coverage: ContributorCoverage::Exact(partition),
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

    /// The bare fold this family's fixtures declare, over one reduced axis.
    fn bare_sum(axes: Vec<Axis>) -> ScalarProgram {
        ScalarProgram::StrictSerialSum {
            axes,
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        }
    }

    /// Rebinds a reduction fixture's contributor read to another boundary tensor.
    ///
    /// The access and its bounds proof move together because
    /// [`verify_proof_records`] requires them to name one tensor: separating them
    /// would report the proof reference and prove nothing about the boundary role
    /// under test.
    fn read_from(builder: &mut ScheduledRegionBuilder, tensor: TensorRole) {
        builder.accesses[0].tensor = tensor;
        builder.bounds_proofs[0].tensor = tensor;
    }

    /// One inhabitant of every [`ScalarProgram`] variant and its expected
    /// reduction-family classification.
    ///
    /// The array is sized from the type rather than from a hand-written count:
    /// widening the scalar vocabulary without classifying its new inhabitant is
    /// therefore a compile error here instead of a smaller census that stays
    /// green.
    struct ScalarProgramFamilyCase {
        name: &'static str,
        program: ScalarProgram,
        parallel: Option<ParallelFamily>,
    }

    fn scalar_program_family_population()
    -> [ScalarProgramFamilyCase; variant_count::<ScalarProgram>()] {
        let mut bf16 = PointwiseBf16ExpressionBuilder::new();
        let bf16_input = bf16.input(AccessOrdinal::FIRST).unwrap();
        let bf16 = bf16.build(bf16_input).unwrap();
        [
            ScalarProgramFamilyCase {
                name: "pointwise f32",
                program: ScalarProgram::PointwiseF32(scale_bias_expression(
                    1.0_f32.to_bits(),
                    0.0_f32.to_bits(),
                )),
                parallel: None,
            },
            ScalarProgramFamilyCase {
                name: "pointwise bf16",
                program: ScalarProgram::PointwiseBf16(bf16),
                parallel: None,
            },
            ScalarProgramFamilyCase {
                name: "strict affine u4 decode",
                program: ScalarProgram::StrictAffineU4Dequantize {
                    codes_role: STRICT_AFFINE_CODES_ROLE,
                    scale_role: STRICT_AFFINE_SCALE_ROLE,
                    zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
                },
                parallel: None,
            },
            ScalarProgramFamilyCase {
                name: "strict serial sum",
                program: bare_sum(vec![Axis::new(1)]),
                parallel: Some(ParallelFamily::Split { final_pass: true }),
            },
            ScalarProgramFamilyCase {
                name: "scale-bias prologue",
                program: ScalarProgram::FusedMultiplyAddSerialSum {
                    scale_bits: 1.0_f32.to_bits(),
                    bias_bits: 0.0_f32.to_bits(),
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                    canonical_nan_bits: 0x7fc0_0000,
                    empty_identity_bits: 0.0_f32.to_bits(),
                    contraction: false,
                },
                parallel: Some(ParallelFamily::Split { final_pass: false }),
            },
            ScalarProgramFamilyCase {
                name: "squaring prologue",
                program: ScalarProgram::SquaredSerialSum {
                    axes: vec![Axis::new(1)],
                    order: ContributorOrder::OriginalAxisLexicographic,
                    canonical_nan_bits: 0x7fc0_0000,
                    empty_identity_bits: 0.0_f32.to_bits(),
                },
                parallel: Some(ParallelFamily::Split { final_pass: false }),
            },
            ScalarProgramFamilyCase {
                name: "squaring prologue with epilogue",
                program: squared_sum_with_epilogue(scale_epilogue()),
                parallel: Some(ParallelFamily::SerialOnly),
            },
            ScalarProgramFamilyCase {
                name: "strict tensor contraction",
                program: ScalarProgram::StrictTensorContraction {
                    contracted_shape: Shape::from_dims([6]),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    canonical_nan_bits: 0x7fc0_0000,
                },
                parallel: None,
            },
            ScalarProgramFamilyCase {
                name: "extrema fold",
                program: maximum_scalar(),
                parallel: Some(ParallelFamily::Split { final_pass: true }),
            },
        ]
    }

    /// Makes a parallel fixture's numerical declaration agree with one family.
    ///
    /// Every sum family consumes reassociation; maximum is order-insensitive and
    /// consumes none. The edit moves the topology declaration and realization
    /// together so the contributor-tensor comparison below is the only varying
    /// admission fact.
    fn declare_family_reassociation(builder: &mut ScheduledRegionBuilder, program: &ScalarProgram) {
        if !matches!(program, ScalarProgram::StrictSerialMaximum { .. }) {
            return;
        }
        builder.numerical = Some(strict_numerical());
        match &mut builder
            .schedule
            .as_mut()
            .expect("the fixture has a schedule")
            .reduction
        {
            ReductionTopology::MultiPass {
                permits_reassociation,
                ..
            }
            | ReductionTopology::CooperativeWorkgroup {
                permits_reassociation,
                ..
            } => *permits_reassociation = false,
            ReductionTopology::None
            | ReductionTopology::Serial { .. }
            | ReductionTopology::Contraction { .. }
            | ReductionTopology::CooperativeContraction { .. }
            | ReductionTopology::LiveContraction { .. } => {
                panic!("the fixture has a parallel reduction")
            }
        }
    }

    fn partial_family_builder(
        program: ScalarProgram,
        read_tensor: TensorRole,
    ) -> ScheduledRegionBuilder {
        let mut builder = partial_pass_builder(SPLIT);
        read_from(&mut builder, read_tensor);
        declare_family_reassociation(&mut builder, &program);
        builder.scalar_program = Some(program);
        builder
    }

    fn cooperative_family_builder(
        program: ScalarProgram,
        read_tensor: TensorRole,
    ) -> ScheduledRegionBuilder {
        let mut builder = cooperative_builder(cooperative_tile_fixture());
        read_from(&mut builder, read_tensor);
        declare_family_reassociation(&mut builder, &program);
        builder.scalar_program = Some(program);
        builder
    }

    /// The scalar-program population derives exactly five serial fold families,
    /// four of which also state a parallel split.
    #[test]
    fn the_scalar_program_population_derives_five_serial_and_four_parallel_families() {
        let population = scalar_program_family_population();
        assert_eq!(
            population
                .iter()
                .filter(|case| case.parallel.is_some())
                .count(),
            5,
            "five ScalarProgram variants are serial fold families",
        );
        assert_eq!(
            population
                .iter()
                .filter(|case| matches!(case.parallel, Some(ParallelFamily::Split { .. })))
                .count(),
            4,
            "four serial families also state a parallel split",
        );
        for case in population {
            let derived = split_family(&case.program).map(|family| family.parallel);
            assert_eq!(derived, case.parallel, "{} classification", case.name);
        }
    }

    /// Every family shared by the three topologies admits the same boundary
    /// contributor tensors.
    ///
    /// Three roles cover the complete fieldless predicate vocabulary: an input,
    /// the materialized intermediate, and the refused output. The expected answer comes from the family derivation and
    /// is checked independently through each production admission, so changing
    /// only the serial gate's read predicate makes this test fail even though the
    /// family table and both parallel gates still agree.
    #[test]
    fn shared_families_admit_the_same_contributor_tensors_in_every_topology() {
        let tensors = [
            TensorRole::Input,
            TensorRole::Intermediate,
            TensorRole::Output,
        ];
        for case in scalar_program_family_population()
            .into_iter()
            .filter(|case| matches!(case.parallel, Some(ParallelFamily::Split { .. })))
        {
            let family = split_family(&case.program).expect("the case is a fold family");
            for tensor in tensors {
                let expected = family
                    .read_tensor(FamilyTopology::Serial)
                    .expect("every fold has a serial contributor tensor")
                    .admits(tensor);

                let mut serial = serial_reduction_builder(case.program.clone());
                read_from(&mut serial, tensor);
                let serial_admitted = serial.build().is_ok();
                let partial_admitted = partial_family_builder(case.program.clone(), tensor)
                    .build()
                    .is_ok();
                let cooperative_admitted = cooperative_family_builder(case.program.clone(), tensor)
                    .build()
                    .is_ok();

                assert_eq!(
                    serial_admitted, expected,
                    "serial {} reading {tensor:?}",
                    case.name,
                );
                assert_eq!(
                    partial_admitted, expected,
                    "partial {} reading {tensor:?}",
                    case.name,
                );
                assert_eq!(
                    cooperative_admitted, expected,
                    "cooperative {} reading {tensor:?}",
                    case.name,
                );
            }
        }
    }

    /// A fused family that requests contraction remains outside every fold
    /// topology; shared derivation must not erase this per-family residual.
    #[test]
    fn a_contracted_fused_program_is_not_a_reduction_family() {
        let mut contracted = scalar_program_family_population()
            .into_iter()
            .find(|case| case.name == "scale-bias prologue")
            .expect("the population contains the fused family")
            .program;
        let ScalarProgram::FusedMultiplyAddSerialSum { contraction, .. } = &mut contracted else {
            panic!("the named population member is the fused family")
        };
        *contraction = true;
        assert!(split_family(&contracted).is_none());
        assert!(
            serial_reduction_builder(contracted.clone())
                .build()
                .is_err()
        );
        assert!(
            partial_family_builder(contracted.clone(), TensorRole::Input)
                .build()
                .is_err()
        );
        assert!(
            cooperative_family_builder(contracted, TensorRole::Input)
                .build()
                .is_err()
        );
    }

    /// Restates the complete serial fixture over one input shape and axis list.
    ///
    /// Every construction and consumption site moves together so the resulting
    /// region isolates whether serial empty-domain admission newly requires a
    /// contributor count. The base serial arms compared these facts structurally
    /// but did not canonicalize or multiply the axes of an identity-seeded fold.
    fn restate_serial_reduction_domain(
        builder: &mut ScheduledRegionBuilder,
        input: Shape,
        axes: Vec<Axis>,
    ) {
        let output = input.without_axes(&axes);
        let output_elements = element_count(&output).expect("the retained fixture shape fits u64");
        builder.iteration_shape = Some(output.clone());

        let LogicalAccess::ReductionContributor {
            input_shape,
            output_shape,
            axes: access_axes,
            ..
        } = &mut builder.accesses[0].map
        else {
            panic!("the serial fixture has a contributor access")
        };
        *input_shape = input.clone();
        *output_shape = output.clone();
        *access_axes = axes.clone();

        let BoundsProofKind::ReductionDomain {
            input_shape,
            output_shape,
            axes: proof_axes,
            ..
        } = &mut builder.bounds_proofs[0].kind
        else {
            panic!("the serial fixture has a contributor proof")
        };
        *input_shape = input;
        *output_shape = output;
        *proof_axes = axes.clone();
        builder.bounds_proofs[1].kind = BoundsProofKind::LinearRange {
            element_count: output_elements,
        };
        builder.ownership_proof.as_mut().unwrap().kind =
            OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            };

        match builder
            .scalar_program
            .as_mut()
            .expect("the serial fixture has a scalar program")
        {
            ScalarProgram::StrictSerialSum {
                axes: scalar_axes, ..
            }
            | ScalarProgram::FusedMultiplyAddSerialSum {
                axes: scalar_axes, ..
            }
            | ScalarProgram::SquaredSerialSum {
                axes: scalar_axes, ..
            }
            | ScalarProgram::SquaredSerialSumThenEpilogue {
                axes: scalar_axes, ..
            }
            | ScalarProgram::StrictSerialMaximum {
                axes: scalar_axes, ..
            } => *scalar_axes = axes.clone(),
            ScalarProgram::PointwiseF32(_)
            | ScalarProgram::PointwiseBf16(_)
            | ScalarProgram::StrictAffineU4Dequantize { .. }
            | ScalarProgram::StrictTensorContraction { .. } => {
                panic!("the fixture has a serial fold program")
            }
        }
        let schedule = builder
            .schedule
            .as_mut()
            .expect("the serial fixture has a schedule");
        let ReductionTopology::Serial {
            axes: scheduled_axes,
            ..
        } = &mut schedule.reduction
        else {
            panic!("the serial fixture has a serial topology")
        };
        *scheduled_axes = axes;
        schedule.work_items = output_elements;
        schedule.launch.grid_threads = output_elements;
    }

    /// Identity-seeded serial folds preserve the exact base admission boundary:
    /// empty-domain verification validates their identity without counting the
    /// contributors.
    ///
    /// Duplicate and out-of-range axes and an overflowing reduced-extent product
    /// are not endorsed as a new contract here; they are deliberately pinned as
    /// admitted because this private refactor may not narrow the pre-existing
    /// serial set. Maximum is the adjacent control: its missing identity makes a
    /// successful contributor count load-bearing, so the same duplicate axes are
    /// refused. A wrong sum identity is the other control and stays refused even
    /// though no count is derived.
    #[test]
    fn identity_seeded_serial_folds_do_not_require_a_contributor_count() {
        let duplicate_axes = vec![Axis::new(1), Axis::new(1)];
        for (name, scalar) in serial_fold_families()
            .into_iter()
            .filter(|(_, scalar)| !matches!(scalar, ScalarProgram::StrictSerialMaximum { .. }))
        {
            let mut builder = serial_reduction_builder(scalar);
            restate_serial_reduction_domain(
                &mut builder,
                Shape::from_dims([2, 6]),
                duplicate_axes.clone(),
            );
            builder
                .build()
                .unwrap_or_else(|error| panic!("{name} narrowed on duplicate axes: {error:?}"));
        }

        for (name, input, axes) in [
            (
                "out-of-range axis",
                Shape::from_dims([2, 6]),
                vec![Axis::new(2)],
            ),
            (
                "overflowing contributor product",
                Shape::from_dims([u64::MAX, 2]),
                vec![Axis::new(0), Axis::new(1)],
            ),
        ] {
            let mut builder = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
            restate_serial_reduction_domain(&mut builder, input, axes);
            builder
                .build()
                .unwrap_or_else(|error| panic!("serial sum narrowed on {name}: {error:?}"));
        }

        let mut maximum = serial_reduction_builder(maximum_scalar());
        restate_serial_reduction_domain(&mut maximum, Shape::from_dims([2, 6]), duplicate_axes);
        assert_eq!(
            maximum.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "an identity-less fold still owes a countable, non-empty domain",
        );

        let mut wrong_identity = bare_sum(vec![Axis::new(1)]);
        let ScalarProgram::StrictSerialSum {
            empty_identity_bits,
            ..
        } = &mut wrong_identity
        else {
            unreachable!()
        };
        *empty_identity_bits = (-0.0_f32).to_bits();
        assert_eq!(
            serial_reduction_builder(wrong_identity)
                .build()
                .unwrap_err()
                .diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "skipping the count does not skip identity validation",
        );
    }

    /// A bare serial sum folds an input access or a materialized domain.
    ///
    /// **The widening, and its exact width.** `ScalarProgram::StrictSerialSum`
    /// carries no prologue, so it says how contributors combine and nothing about
    /// where they live: `sum(x)` over any declared input tensor and the same fold
    /// over a prologue region's materialized result are one scalar program over
    /// several possible boundary tensors. What the widening is *not* is "any tensor" —
    /// a program output remains refused because no fold reads one as a
    /// contributor domain.
    #[test]
    fn a_bare_serial_sum_folds_a_declared_input_or_a_materialized_domain() {
        assert!(
            serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
                .build()
                .is_ok(),
            "a fold over the first declared input has no prologue region to read",
        );

        let mut materialized = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
        read_from(&mut materialized, TensorRole::Intermediate);
        assert!(
            materialized.build().is_ok(),
            "the prologue-carrying plan still folds the intermediate it staged",
        );

        let input = serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
            .build()
            .unwrap();
        assert_eq!(input.region().index.accesses[0].tensor, TensorRole::Input);

        let mut output = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
        read_from(&mut output, TensorRole::Output);
        assert_eq!(
            output.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a fold cannot read its contributor domain from a program output",
        );
    }

    /// A bare fold still proves its contributor access against its own reduction.
    ///
    /// The widening moved which *tensor* the read may bind and nothing else. The
    /// declared reduction and the access relation still have to state the same
    /// reduced axes, so a region folding a declared input over one axis while
    /// addressing another is refused exactly as the intermediate-reading one always
    /// was.
    ///
    /// The fold's own declaration moves here rather than the access's, because the
    /// bounds proof refines the *access*: perturbing the access alone is caught one
    /// authority earlier and would report the proof reference instead of the
    /// disagreement under test.
    #[test]
    fn a_bare_fold_over_an_input_still_proves_its_contributor_access() {
        let mut mismatched = serial_reduction_builder(bare_sum(vec![Axis::new(0)]));
        let Some(ReductionTopology::Serial { axes, .. }) = mismatched
            .schedule
            .as_mut()
            .map(|schedule| &mut schedule.reduction)
        else {
            panic!("the fixture schedules a serial reduction");
        };
        *axes = vec![Axis::new(0)];
        assert_eq!(
            mismatched.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a fold declaring one reduced axis while addressing another is not that fold",
        );
    }

    /// Rebinds a region's owning write to another boundary tensor.
    ///
    /// The write access, its bounds proof, and the ownership proof move together
    /// because [`verify_proof_records`] requires all three to name one tensor:
    /// moving fewer would report the proof reference and prove nothing about the
    /// boundary role under test. The write is the last access by the same
    /// convention [`verify_intrinsic`] destructures it under.
    fn write_to(builder: &mut ScheduledRegionBuilder, tensor: TensorRole) {
        let write = builder.accesses.len() - 1;
        builder.accesses[write].tensor = tensor;
        builder.bounds_proofs[write].tensor = tensor;
        builder.ownership_proof.as_mut().unwrap().tensor = tensor;
    }

    /// Every serial fold family, over the shared serial fixture's reduced axis.
    ///
    /// Named and returned as a population rather than asserted one family at a
    /// time, so the test below counts what it covered: a write rule that reached
    /// three of these five would otherwise pass a spot check and leave the rest
    /// silently narrower.
    fn serial_fold_families() -> Vec<(&'static str, ScalarProgram)> {
        let axes = vec![Axis::new(1)];
        vec![
            ("strict serial sum", bare_sum(axes.clone())),
            ("extrema fold", maximum_scalar()),
            (
                "squaring prologue",
                ScalarProgram::SquaredSerialSum {
                    axes: axes.clone(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    canonical_nan_bits: 0x7fc0_0000,
                    empty_identity_bits: 0.0_f32.to_bits(),
                },
            ),
            (
                "squaring prologue with an epilogue",
                squared_sum_with_epilogue(scale_epilogue()),
            ),
            (
                "scale-bias prologue",
                ScalarProgram::FusedMultiplyAddSerialSum {
                    scale_bits: 1.0_f32.to_bits(),
                    bias_bits: 0.0_f32.to_bits(),
                    axes,
                    order: ContributorOrder::OriginalAxisLexicographic,
                    canonical_nan_bits: 0x7fc0_0000,
                    empty_identity_bits: 0.0_f32.to_bits(),
                    contraction: false,
                },
            ),
        ]
    }

    /// Every serial fold may commit its result to a materialized intermediate.
    ///
    /// **The widening, and its exact width.** Where a fold's result goes is a
    /// property of the surrounding cover and not of the fold: `sum(x * x)` whose
    /// value the caller asked for and the same fold whose value an epilogue
    /// scales are one computation committing to two boundary tensors. All five
    /// families widen together because none of their algebras distinguishes the
    /// two — admitting only the bare sum would say a squaring prologue's result
    /// is inherently the program's answer, which is false, and would leave the
    /// *fused* alternative unspellable for every reduction an epilogue consumes
    /// while the materialized-prologue alternative compiled.
    ///
    /// What the widening is *not* is "any tensor": a write to a declared input
    /// stays refused, because a region committing there would mutate a tensor the
    /// caller owns whatever it folded to get there.
    #[test]
    fn every_serial_fold_family_may_commit_to_a_materialized_intermediate() {
        let families = serial_fold_families();
        assert_eq!(
            families.len(),
            5,
            "the serial match has five fold arms, and each must be driven",
        );
        for (name, scalar) in families {
            assert!(
                serial_reduction_builder(scalar.clone()).build().is_ok(),
                "the output-writing control for the {name} must verify, \
                 or neither case below is evidence",
            );

            let mut staged = serial_reduction_builder(scalar.clone());
            write_to(&mut staged, TensorRole::Intermediate);
            assert!(
                staged.build().is_ok(),
                "the {name} has a producer region for the value an epilogue reads",
            );

            let mut into_input = serial_reduction_builder(scalar);
            write_to(&mut into_input, TensorRole::Input);
            assert_eq!(
                into_input.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
                "no fold commits its result into a tensor the caller owns ({name})",
            );
        }
    }

    /// Committing to an intermediate is a distinct region, not a free relabel.
    ///
    /// The write role reaches `encode_identity` through the access list, the
    /// write's bounds proof, and the ownership proof, so a staged fold and a
    /// published one are different canonical regions. Without this, a plan that
    /// materialized a fold's result and one that published it could share a
    /// cache entry and one would be served for the other.
    #[test]
    fn the_committed_tensor_separates_scheduled_region_identity() {
        let published = serial_reduction_builder(bare_sum(vec![Axis::new(1)]))
            .build()
            .unwrap();
        let mut builder = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
        write_to(&mut builder, TensorRole::Intermediate);
        let staged = builder.build().unwrap();
        assert_ne!(
            published.canonical_identity().as_bytes(),
            staged.canonical_identity().as_bytes(),
        );
    }

    /// A split's committing pass chooses its write tensor; its staging pass does not.
    ///
    /// **The asymmetry is the assertion, and it is the write counterpart of the
    /// read asymmetry [`only_the_partial_pass_of_a_split_may_fold_a_declared_input`]
    /// pins.** The final pass commits the reduction's own result, so the cover
    /// decides where it lands exactly as it does for the serial fold this split
    /// replaces — which is what keeps a split alternative available for a
    /// reduction whose result an epilogue consumes. The partial pass commits an
    /// unfolded fragment, which is no cover's output;
    /// [`a_partial_pass_may_not_write_the_program_output`] is the narrow pin for
    /// that half and is unchanged by this widening.
    #[test]
    fn only_the_committing_pass_of_a_split_chooses_its_write_tensor() {
        assert!(
            final_pass_builder(SPLIT).build().is_ok(),
            "the output-writing control must verify, or neither case below is evidence",
        );

        let mut staged = final_pass_builder(SPLIT);
        write_to(&mut staged, TensorRole::Intermediate);
        assert!(
            staged.build().is_ok(),
            "a split fold whose result an epilogue reads stages it from its final pass",
        );

        let mut into_input = final_pass_builder(SPLIT);
        write_to(&mut into_input, TensorRole::Input);
        assert_eq!(
            into_input.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "no pass commits its result into a tensor the caller owns",
        );

        let mut partial = partial_pass_builder(SPLIT);
        write_to(&mut partial, TensorRole::Output);
        assert_eq!(
            partial.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a partial is an unfolded fragment and is no cover's declared output",
        );
    }

    /// A split's partial pass may fold a declared input; its final pass may not.
    ///
    /// **The asymmetry is the assertion.** The partial pass folds the region's
    /// declared contributor domain, which lives wherever the plan put it. The final
    /// pass folds values the partial pass *staged*, and those exist only because it
    /// staged them — so a final pass claiming a declared input holds them describes
    /// a handoff no dispatch performed.
    #[test]
    fn only_the_partial_pass_of_a_split_may_fold_a_declared_input() {
        assert!(
            partial_pass_builder(SPLIT).build().is_ok(),
            "the intermediate-reading control must verify, or neither case below is evidence",
        );
        assert!(final_pass_builder(SPLIT).build().is_ok());

        let mut partial = partial_pass_builder(SPLIT);
        read_from(&mut partial, TensorRole::Input);
        assert!(
            partial.build().is_ok(),
            "a prologue-less fold's partial pass retains the declared input it folds",
        );

        let mut combine = final_pass_builder(SPLIT);
        read_from(&mut combine, TensorRole::Input);
        assert_eq!(
            combine.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "no declared input holds partials a dispatch staged",
        );
    }

    /// A cooperative tile may fold a declared input, and only a declared one.
    ///
    /// The tile stages its partials in workgroup memory rather than in a boundary
    /// tensor, so its single read is the declared contributor domain whatever the
    /// plan staged — which is why it carries no pass distinction where the
    /// multi-pass split has one.
    #[test]
    fn a_cooperative_tile_may_fold_a_declared_input() {
        assert!(
            cooperative_builder(cooperative_tile_fixture())
                .build()
                .is_ok(),
            "the intermediate-reading control must verify, or neither case below is evidence",
        );

        let mut input = cooperative_builder(cooperative_tile_fixture());
        read_from(&mut input, TensorRole::Input);
        assert!(input.build().is_ok());
    }

    /// A fused affine fold reads an input access in every supported topology.
    #[test]
    fn an_affine_fold_reads_any_declared_input_in_serial_and_parallel_forms() {
        let affine = ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: 2.0_f32.to_bits(),
            bias_bits: 1.0_f32.to_bits(),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
            contraction: false,
        };

        let mut serial = serial_reduction_builder(affine.clone());
        read_from(&mut serial, TensorRole::Input);
        serial
            .build()
            .expect("the serial affine fold reads input one");

        let mut partial = partial_pass_builder(SPLIT);
        partial.scalar_program = Some(affine.clone());
        read_from(&mut partial, TensorRole::Input);
        partial
            .build()
            .expect("the affine partial pass reads input one");

        let mut cooperative = cooperative_builder(cooperative_tile_fixture());
        cooperative.scalar_program = Some(affine);
        read_from(&mut cooperative, TensorRole::Input);
        cooperative
            .build()
            .expect("the affine cooperative tile reads input one");
    }

    /// Parallel affine folds read a declared input, never an intermediate.
    ///
    /// The bare sum's parallel forms admit an intermediate because it may hold a
    /// materialized prologue. The affine family carries that prologue inside its
    /// scalar program, so admitting the intermediate would apply the affine body
    /// to a value that was already transformed or to an unbound staging edge.
    #[test]
    fn affine_parallel_folds_reject_an_intermediate_contributor() {
        let affine = ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: 2.0_f32.to_bits(),
            bias_bits: 1.0_f32.to_bits(),
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
            contraction: false,
        };

        let mut partial = partial_pass_builder(SPLIT);
        partial.scalar_program = Some(affine.clone());
        assert_eq!(
            partial.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "an affine partial pass cannot read a materialized contributor",
        );

        let mut cooperative = cooperative_builder(cooperative_tile_fixture());
        cooperative.scalar_program = Some(affine);
        assert_eq!(
            cooperative.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "an affine cooperative tile cannot read a materialized contributor",
        );
    }

    /// Family-specific input-role rules remain exact across serial families.
    #[test]
    fn squared_and_maximum_folds_require_an_input_access() {
        for scalar in [
            ScalarProgram::SquaredSerialSum {
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            squared_sum_with_epilogue(scale_epilogue()),
            maximum_scalar(),
        ] {
            let region = serial_reduction_builder(scalar.clone());
            assert!(region.build().is_ok());
            let mut intermediate = serial_reduction_builder(scalar);
            read_from(&mut intermediate, TensorRole::Intermediate);
            assert_eq!(
                intermediate.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::NumericalOrAccessRefinement,]
            );
        }
    }

    /// The squared parallel family keeps its own input-role rule.
    ///
    /// This is separate from the serial control above because the split and
    /// cooperative family tables are independent match arms. Widening either to
    /// every declared input would otherwise leave the serial check green.
    #[test]
    fn squared_parallel_folds_require_an_input_access() {
        let mut partial = squared_partial_pass_builder(SPLIT);
        read_from(&mut partial, TensorRole::Intermediate);
        assert_eq!(
            partial.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a squared partial pass cannot read an intermediate",
        );

        let squared = ScalarProgram::SquaredSerialSum {
            axes: vec![Axis::new(1)],
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: 0x7fc0_0000,
            empty_identity_bits: 0.0_f32.to_bits(),
        };
        let mut cooperative = cooperative_builder(cooperative_tile_fixture());
        cooperative.scalar_program = Some(squared);
        read_from(&mut cooperative, TensorRole::Intermediate);
        assert_eq!(
            cooperative.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a squared cooperative tile cannot read an intermediate",
        );
    }

    /// The maximum parallel family keeps its own input-role rule.
    ///
    /// Maximum has independent split and cooperative family-table arms, so the
    /// serial maximum control does not prove that either parallel obligation
    /// still refuses a later declared input.
    #[test]
    fn maximum_parallel_folds_require_an_input_access() {
        let mut partial = extrema_partial_builder(SPLIT);
        read_from(&mut partial, TensorRole::Intermediate);
        assert_eq!(
            partial.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a maximum partial pass cannot read an intermediate",
        );

        let mut cooperative = extrema_cooperative_builder();
        read_from(&mut cooperative, TensorRole::Intermediate);
        assert_eq!(
            cooperative.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a maximum cooperative tile cannot read an intermediate",
        );
    }

    /// A cooperative tile may commit its result to a materialized intermediate.
    ///
    /// A tile is both halves of a split in one dispatch, so its single write is
    /// the fold's committing write and carries the same cover-assigned obligation
    /// the serial fold and the split's final pass carry. It has no staging pass
    /// whose target the split structure fixes, because it stages in workgroup
    /// memory rather than in a boundary tensor — which is why it needs no pass
    /// distinction here, exactly as it needs none for its read.
    #[test]
    fn a_cooperative_tile_may_commit_to_a_materialized_intermediate() {
        assert!(
            cooperative_builder(cooperative_tile_fixture())
                .build()
                .is_ok(),
            "the output-writing control must verify, or neither case below is evidence",
        );

        let mut staged = cooperative_builder(cooperative_tile_fixture());
        write_to(&mut staged, TensorRole::Intermediate);
        assert!(
            staged.build().is_ok(),
            "a tiled fold whose result an epilogue reads stages it from its commit",
        );

        let mut into_input = cooperative_builder(cooperative_tile_fixture());
        write_to(&mut into_input, TensorRole::Input);
        assert_eq!(
            into_input.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "no tile commits its result into a tensor the caller owns",
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
    /// The cases reach two different rules, and both are the right one. A split
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
                ScheduledRegionDiagnostic::ContributorCoverage {
                    rule: ContributorCoverageRule::ExactCoverage,
                },
            ),
            // The same partition count, three covered where the access supplies
            // six.
            (
                ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 1,
                },
                ScheduledRegionDiagnostic::ContributorCoverage {
                    rule: ContributorCoverageRule::ExactCoverage,
                },
            ),
        ] {
            let mut builder = partial_pass_builder(SPLIT);
            let ReductionTopology::MultiPass { coverage, .. } =
                &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *coverage = ContributorCoverage::Exact(partition);
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [expected],
                "{partition:?} does not cover six contributors exactly once each"
            );
        }
    }

    const NEG_ZERO: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0x8000_0000);
    const POS_ZERO: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0x0000_0000);
    const NEG_INF: ReductionPaddingIdentity = ReductionPaddingIdentity::F32(0xff80_0000);
    const PADDED_SPLIT: ContributorPartition = ContributorPartition {
        partitions: 3,
        contributors_per_partition: 3,
    };

    fn set_coverage(builder: &mut ScheduledRegionBuilder, coverage: ContributorCoverage) {
        let ReductionTopology::MultiPass {
            coverage: declared, ..
        } = &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *declared = coverage;
    }

    fn padded_partial(identity: ReductionPaddingIdentity) -> ScheduledRegionBuilder {
        let mut builder = partial_pass_builder(PADDED_SPLIT);
        set_coverage(
            &mut builder,
            ContributorCoverage::IdentityPadded {
                partition: PADDED_SPLIT,
                identity,
            },
        );
        builder
    }

    /// Exact coverage of a previously encodable split keeps the pre-coverage
    /// layout: the identity of two Exact regions is byte-identical, and a
    /// padded sibling is a strict extension rather than a reinterpretation.
    #[test]
    fn exact_multi_pass_encodings_remain_byte_identical_and_padding_appends() {
        let exact = partial_pass_builder(SPLIT)
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec();
        let again = partial_pass_builder(SPLIT)
            .build()
            .unwrap()
            .canonical_identity()
            .as_bytes()
            .to_vec();
        assert_eq!(exact, again, "exact coverage is a closed encoding");

        let padded = padded_partial(NEG_ZERO)
            .build()
            .expect("a suffix-padded add split with -0.0 verifies")
            .canonical_identity()
            .as_bytes()
            .to_vec();
        assert_ne!(exact, padded);
        assert!(
            padded.len() > exact.len(),
            "the padded arm appends a local tag and identity; exact writes neither"
        );
    }

    /// Coverage tag: claiming a pad on an exactly covering split is padded
    /// coverage, not exact coverage under another name.
    #[test]
    fn a_zero_length_pad_is_refused_as_padded_coverage() {
        let mut builder = partial_pass_builder(SPLIT);
        set_coverage(
            &mut builder,
            ContributorCoverage::IdentityPadded {
                partition: SPLIT,
                identity: NEG_ZERO,
            },
        );
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::PaddedCoverage,
            }]
        );
    }

    /// Partition capacity: a pad whose split is shorter than the real sequence
    /// is refused by name.
    #[test]
    fn a_pad_below_the_real_count_is_refused() {
        let mut builder = partial_pass_builder(SPLIT);
        set_coverage(
            &mut builder,
            ContributorCoverage::IdentityPadded {
                partition: ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: 1,
                },
                identity: NEG_ZERO,
            },
        );
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::CapacityBelowRealCount,
            }]
        );
    }

    /// Arithmetic type: a well-formed `bf16` identity on an `f32` fold is a
    /// named mismatch, not an unrepresentable one.
    #[test]
    fn a_padding_identity_of_the_wrong_arithmetic_type_is_refused() {
        assert_eq!(
            padded_partial(ReductionPaddingIdentity::Bf16(0x8000))
                .build()
                .unwrap_err()
                .diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::ArithmeticTypeMismatch,
            }]
        );
    }

    /// Identity bits: `+0.0` is the empty-domain result, not the additive pad,
    /// when signed zero is observable.
    #[test]
    fn plus_zero_is_not_a_two_sided_additive_identity_under_strict_signed_zero() {
        assert_eq!(
            padded_partial(POS_ZERO).build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::TwoSidedNeutrality,
            }]
        );
    }

    /// Signed-zero permission: the same `+0.0` bits are admitted once
    /// elimination is permitted, because the two zeros are then observably equal.
    #[test]
    fn plus_zero_is_neutral_when_signed_zero_elimination_is_permitted() {
        let mut builder = padded_partial(POS_ZERO);
        builder.numerical = Some(NumericalRealization {
            signed_zero: NumericalPermission::Permitted,
            ..reassociating_numerical()
        });
        builder
            .build()
            .expect("+0.0 is observably neutral under signed-zero elimination");
    }

    /// Family: `-0.0` is the additive pad and `-inf` is the maximum pad; each
    /// is refused on the other family.
    #[test]
    fn padding_identity_is_family_specific() {
        let mut maximum = extrema_partial_builder(PADDED_SPLIT);
        set_coverage(
            &mut maximum,
            ContributorCoverage::IdentityPadded {
                partition: PADDED_SPLIT,
                identity: NEG_ZERO,
            },
        );
        assert_eq!(
            maximum.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::TwoSidedNeutrality,
            }]
        );

        assert_eq!(
            padded_partial(NEG_INF).build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::TwoSidedNeutrality,
            }]
        );

        let mut admitted = extrema_partial_builder(PADDED_SPLIT);
        set_coverage(
            &mut admitted,
            ContributorCoverage::IdentityPadded {
                partition: PADDED_SPLIT,
                identity: NEG_INF,
            },
        );
        admitted
            .build()
            .expect("-inf is the two-sided identity of the NaN-propagating maximum");
    }

    /// An all-padding sequence has no real prefix and is not a canonical suffix.
    #[test]
    fn an_all_padding_split_is_refused_as_noncanonical_placement() {
        let mut builder = partial_pass_builder(SPLIT);
        let LogicalAccess::ReductionContributor { input_shape, .. } = &mut builder.accesses[0].map
        else {
            panic!("the fixture reads a reduction contributor");
        };
        *input_shape = Shape::from_dims([2, 0]);
        let BoundsProofKind::ReductionDomain {
            input_shape: proof_shape,
            ..
        } = &mut builder.bounds_proofs[0].kind
        else {
            panic!("the fixture proves a reduction domain");
        };
        *proof_shape = Shape::from_dims([2, 0]);
        set_coverage(
            &mut builder,
            ContributorCoverage::IdentityPadded {
                partition: SPLIT,
                identity: NEG_ZERO,
            },
        );
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::NoncanonicalPlacement,
            }]
        );
    }

    /// Partition capacity overflow is named rather than folded into a coverage miss.
    #[test]
    fn an_overflowing_padded_capacity_is_refused() {
        let mut builder = partial_pass_builder(SPLIT);
        set_coverage(
            &mut builder,
            ContributorCoverage::IdentityPadded {
                partition: ContributorPartition {
                    partitions: 3,
                    contributors_per_partition: u64::MAX,
                },
                identity: NEG_ZERO,
            },
        );
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::Overflow,
            }]
        );
    }

    /// A padded final pass invents partials the tensor does not hold.
    #[test]
    fn a_padded_final_pass_is_refused() {
        let mut builder = final_pass_builder(SPLIT);
        let ReductionTopology::MultiPass { coverage, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a split topology")
        };
        *coverage = ContributorCoverage::IdentityPadded {
            partition: SPLIT,
            identity: NEG_ZERO,
        };
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::PaddedCoverage,
            }]
        );
    }

    /// An accumulation narrower than the element width is rejected, not accepted.
    ///
    /// **Refused under its own name**, which criterion 3 of
    /// `implement-parallel-reduction-strategies` requires: the diagnostic names
    /// the accumulator and carries both widths, so a producer can tell this from
    /// the wrong axis set or the wrong contributor order that
    /// [`ScheduledRegionDiagnostic::NumericalOrAccessRefinement`] also reports.
    /// A wider declaration is refused by the same rule, and it is driven here
    /// because "narrower" is the criterion's wording and not the check's.
    #[test]
    fn a_narrowed_accumulation_width_is_rejected() {
        for wrong in [
            ArithmeticType::F16,
            ArithmeticType::Bf16,
            ArithmeticType::F64,
        ] {
            let mut builder = partial_pass_builder(SPLIT);
            let ReductionTopology::MultiPass { accumulation, .. } =
                &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a split topology")
            };
            *accumulation = wrong;
            assert_eq!(
                builder.build().unwrap_err().diagnostics(),
                [ScheduledRegionDiagnostic::AccumulationWidth {
                    declared: wrong,
                    required: ArithmeticType::F32,
                }],
                "{wrong:?} is not the width this region computes in"
            );
        }
        // The control: the same builder at the declared width verifies, so the
        // refusals above are about the accumulator and not about the fixture.
        assert!(partial_pass_builder(SPLIT).build().is_ok());
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
        builder.accesses[0].tensor = TensorRole::Input;
        builder.bounds_proofs[0].tensor = TensorRole::Input;
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
                [ScheduledRegionDiagnostic::AccumulationWidth {
                    declared: narrower,
                    required: ArithmeticType::F32,
                }],
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
                coverage: ContributorCoverage::Exact(SPLIT),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(ContributorPartition {
                    partitions: 2,
                    contributors_per_partition: 3,
                }),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(SPLIT),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F64,
                permits_reassociation: true,
                permits_permutation: false,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::Exact(SPLIT),
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: true,
            },
            ReductionTopology::MultiPass {
                pass: ReductionPass::Partial,
                coverage: ContributorCoverage::IdentityPadded {
                    partition: PADDED_SPLIT,
                    identity: NEG_ZERO,
                },
                axes: vec![Axis::new(1)],
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
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
            rounds: 1,
            coordinates: LocalCoordinates {
                source: LocalCoordinateSource::LocalLinearInvocation,
                participants: ParticipantSpace::new(&[3]).expect("rank one is within the bound"),
            },
            staging: vec![tile_staging(3, PhaseId::new(1))],
            phases: vec![
                CooperativePhase {
                    id: PhaseId::FIRST,
                    participation: ParticipantRange { first: 0, count: 3 },
                    writes: vec![StagedWrite {
                        staging: StagingId::FIRST,
                        span: StagedSpan::new(&[1], 0, 1).expect("rank one is within the bound"),
                    }],
                    reads: Vec::new(),
                },
                CooperativePhase {
                    id: PhaseId::new(1),
                    participation: ParticipantRange { first: 0, count: 3 },
                    writes: Vec::new(),
                    reads: vec![StagedRead {
                        staging: StagingId::FIRST,
                        span: StagedSpan::new(&[0], 0, 3).expect("rank one is within the bound"),
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
            coverage: ContributorCoverage::Exact(partition),
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
            6,
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
            coverage,
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
            6,
            ReductionTopology::CooperativeWorkgroup {
                coverage,
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

    /// The fixture region, over a contracted extent the caller states.
    ///
    /// `contracted` is a parameter rather than the fixture's own `6` because the
    /// two-dimensional tiles below need a participant count the `[2, 6]` domain
    /// cannot split — the reduction shape, the contributor coverage, and the
    /// launch width are one arithmetic, and a fixture that fixed one of them
    /// would make the other two unstatable.
    fn cooperative_builder_parts(
        split: ContributorPartition,
        contracted: u64,
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
                    input_shape: Shape::from_dims([2, contracted]),
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
                    input_shape: Shape::from_dims([2, contracted]),
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

    /// The tile's accumulator is refused under the same name the split's is.
    ///
    /// **The second of the two sites, driven separately.**
    /// `verify_accumulation_width` is the single authority both parallel gates
    /// reach, so a test on the split alone would pass while the tile's own call
    /// was deleted. This asserts the tile refuses, with the same diagnostic and
    /// the same payload, on a topology whose other fields are untouched.
    ///
    /// The tile's control is `one_cooperative_tile_verifies_and_derives_its_workgroup_storage`
    /// below, which builds this exact fixture unperturbed.
    #[test]
    fn a_cooperative_tile_declaring_the_wrong_accumulation_width_is_rejected() {
        for wrong in [
            ArithmeticType::F16,
            ArithmeticType::Bf16,
            ArithmeticType::F64,
        ] {
            let mut builder = cooperative_builder(cooperative_tile_fixture());
            let ReductionTopology::CooperativeWorkgroup { accumulation, .. } =
                &mut builder.schedule.as_mut().unwrap().reduction
            else {
                panic!("expected a cooperative topology")
            };
            *accumulation = wrong;
            assert_eq!(
                cooperative_rejection(builder),
                ScheduledRegionDiagnostic::AccumulationWidth {
                    declared: wrong,
                    required: ArithmeticType::F32,
                },
                "{wrong:?} is not the width this tile's region computes in"
            );
        }
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
        tile.coordinates.participants =
            ParticipantSpace::new(&[1]).expect("rank one is within the bound");
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
        let participants = ParticipantRange { first: 0, count: 3 };
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

    /// The loop-carried split: three participants, one contributor each, twice.
    ///
    /// The same `[2, 6] -> [2]` reduction and the same launch as `SPLIT`, with
    /// the six contributors covered as `3 * 1 * 2` instead of `3 * 2 * 1`. Keeping
    /// the launch identical is what makes the round count the only difference
    /// between the two fixtures.
    const ROUND_SPLIT: ContributorPartition = ContributorPartition {
        partitions: 3,
        contributors_per_partition: 1,
    };

    /// The point that orders the fixture's rewrite, at the round boundary.
    fn round_boundary_point() -> SynchronizationPoint {
        SynchronizationPoint {
            id: SyncPointId::new(1),
            placement: SynchronizationPlacement::RoundBoundary,
            convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
            ..tile_point()
        }
    }

    /// The loop-carried tile: the single-round fixture, run twice.
    ///
    /// Structurally identical to [`cooperative_tile_fixture`] apart from the
    /// round count, the second point, and the convergence class both points now
    /// have to name — which is the whole content of the capability.
    fn multi_round_tile_fixture() -> CooperativeTile {
        CooperativeTile {
            rounds: 2,
            synchronization: vec![
                SynchronizationPoint {
                    convergence: ConvergenceEvidence::EveryParticipantExecutesEveryRound,
                    ..tile_point()
                },
                round_boundary_point(),
            ],
            ..cooperative_tile_fixture()
        }
    }

    fn multi_round_builder(tile: CooperativeTile) -> ScheduledRegionBuilder {
        cooperative_builder_with(tile, ROUND_SPLIT)
    }

    /// Applies one edit to the loop-carried fixture and returns its builder.
    fn round_perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
        let mut tile = multi_round_tile_fixture();
        edit(&mut tile);
        multi_round_builder(tile)
    }

    /// A tile that rewrites its slots on a later round verifies.
    ///
    /// The capability itself, and the two derivations it turns on: the rewrite
    /// is no longer a staging conflict, and the anti-dependency it creates is
    /// derived rather than declared. The storage is unchanged from the
    /// single-round fixture, which is the point of reusing slots rather than
    /// unrolling them into fresh ones.
    #[test]
    fn a_loop_carried_tile_rewrites_its_slots_and_verifies() {
        let tile = multi_round_tile_fixture();
        assert_eq!(
            tile.anti_dependency_edges(),
            vec![AntiDependencyEdge {
                staging: StagingId::FIRST,
                consumed_in: PhaseId::new(1),
                rewritten_in: PhaseId::FIRST,
            }]
        );
        // One discharger each, and not the same point: the phase boundary orders
        // the publication and the round boundary orders the rewrite.
        let [visibility] = tile.visibility_edges()[..] else {
            panic!("the fixture stages one handoff")
        };
        assert_eq!(
            tile.discharging_points(visibility)
                .iter()
                .map(|point| point.id)
                .collect::<Vec<_>>(),
            vec![SyncPointId::FIRST]
        );
        assert_eq!(
            tile.anti_discharging_points(tile.anti_dependency_edges()[0])
                .iter()
                .map(|point| point.id)
                .collect::<Vec<_>>(),
            vec![SyncPointId::new(1)]
        );

        let verified = multi_round_builder(tile)
            .build()
            .expect("the loop-carried fixture verifies");
        assert_eq!(verified.requirements().local_memory_bytes, 12);
    }

    /// A single-round tile derives no anti-dependency at all.
    ///
    /// The absence is a claim rather than a missing derivation: no round follows
    /// the only one, so nothing overwrites what the consuming phase read.
    #[test]
    fn a_single_round_tile_derives_no_anti_dependency() {
        assert!(
            cooperative_tile_fixture()
                .anti_dependency_edges()
                .is_empty()
        );
    }

    /// The rewrite needs its own point, and the handoff's does not serve.
    #[test]
    fn a_loop_carried_rewrite_with_no_round_boundary_is_refused() {
        assert_eq!(
            cooperative_rejection(round_perturbed(|tile| {
                tile.synchronization.truncate(1);
            })),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::UndischargedAntiDependency,
            }
        );
    }

    /// A second point over one anti-dependency is two spellings of one program.
    #[test]
    fn two_points_over_one_anti_dependency_are_refused() {
        assert_eq!(
            cooperative_rejection(round_perturbed(|tile| {
                tile.synchronization.push(SynchronizationPoint {
                    id: SyncPointId::new(2),
                    ..round_boundary_point()
                });
            })),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::RedundantPoint,
            }
        );
    }

    /// A round boundary on a tile with one round orders nothing.
    ///
    /// The other side of `RedundantPoint` widening to both evidence classes: a
    /// round boundary is not redundant *because* it discharges no visibility
    /// edge, but it is redundant when there is no following round for it to
    /// separate.
    #[test]
    fn a_round_boundary_without_a_following_round_is_redundant() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                // The single-round derivation, so the point reaches the
                // redundancy rule instead of failing the evidence class first.
                tile.synchronization.push(SynchronizationPoint {
                    convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
                    ..round_boundary_point()
                });
            })),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::RedundantPoint,
            }
        );
    }

    /// The convergence derivation must match the tile's round structure.
    ///
    /// Both directions, because the rule is an equality and a one-sided check
    /// would let the stronger claim stand unearned on a single-round tile.
    #[test]
    fn a_point_naming_the_wrong_convergence_derivation_is_refused() {
        let weak = round_perturbed(|tile| {
            tile.synchronization[0].convergence =
                ConvergenceEvidence::EveryParticipantReachesThePoint;
        });
        assert_eq!(
            cooperative_rejection(weak),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::ConvergenceEvidence,
            }
        );
        let unearned = perturbed(|tile| {
            tile.synchronization[0].convergence =
                ConvergenceEvidence::EveryParticipantExecutesEveryRound;
        });
        assert_eq!(
            cooperative_rejection(unearned),
            ScheduledRegionDiagnostic::Synchronization {
                rule: SynchronizationRule::ConvergenceEvidence,
            }
        );
    }

    /// Two writers to one slot inside one round are still a race.
    ///
    /// The rule the round vocabulary relaxes is the one *between* rounds, and
    /// this is what proves it did not relax the one inside them: no point sits
    /// between two writes of the same phase, so nothing could separate them
    /// however many rounds the tile declares.
    #[test]
    fn overlapping_staged_writes_inside_one_round_are_still_refused() {
        assert_eq!(
            cooperative_rejection(round_perturbed(|tile| {
                tile.phases[0].writes[0].span =
                    StagedSpan::new(&[0], 0, 1).expect("rank one is within the bound");
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingConflict,
            }
        );
    }

    /// A round count of zero, or beyond the governed bound, is refused.
    #[test]
    fn a_round_count_outside_the_governed_profile_is_refused() {
        for rounds in [0, MAX_COOPERATIVE_ROUNDS.saturating_add(1)] {
            assert_eq!(
                cooperative_rejection(round_perturbed(|tile| tile.rounds = rounds)),
                ScheduledRegionDiagnostic::CooperativeTile {
                    rule: CooperativeTileRule::RoundStructure,
                },
                "round count {rounds}"
            );
        }
    }

    /// A split that ignores the round count folds every contributor twice.
    ///
    /// The single-round split covers the sequence once; declared on a two-round
    /// tile it would have each participant fold the same range on both rounds,
    /// which is a different computation and not the declared reduction. Named
    /// as exact-coverage rather than as a tile-shape mismatch: the participants
    /// and iteration domain still agree, and the product does not.
    #[test]
    fn a_split_that_ignores_the_round_count_is_refused() {
        let mut tile = cooperative_tile_fixture();
        tile.rounds = 2;
        assert_eq!(
            cooperative_rejection(cooperative_builder_with(tile, SPLIT)),
            ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::ExactCoverage,
            }
        );
    }

    /// Two participants writing one slot is a race the tile can state.
    #[test]
    fn overlapping_staged_writes_are_rejected() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[0].writes[0].span =
                    StagedSpan::new(&[0], 0, 1).expect("rank one is within the bound");
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

    /// A malformed participant space is rejected, in each way it can be stated.
    ///
    /// The three the space's constructor admits and the verifier refuses: a
    /// rank-zero space, which names no participants at all; a zero extent, whose
    /// product is zero so no invocation has a coordinate; and a product that
    /// overflows `u64`, which no launch could hold. A *rank* above
    /// `MAX_COOPERATIVE_PARTICIPANT_RANK` is deliberately not among them —
    /// `ParticipantSpace::new` refuses it, so it cannot reach this rule, and the
    /// separate assertion below is what keeps that claim from being an
    /// assurance.
    #[test]
    fn an_invalid_participant_space_is_rejected() {
        let malformed = [Vec::new(), vec![0_u64], vec![3, 0], vec![u64::MAX, 2]];
        for extents in malformed {
            let space =
                ParticipantSpace::new(&extents).expect("every case is within the rank bound");
            assert_eq!(
                cooperative_rejection(perturbed(|tile| {
                    tile.coordinates.participants = space;
                })),
                ScheduledRegionDiagnostic::CooperativeTile {
                    rule: CooperativeTileRule::LocalCoordinates,
                },
                "extents {extents:?} were admitted as a participant space"
            );
        }
        assert_eq!(
            ParticipantSpace::new(&[2; MAX_COOPERATIVE_PARTICIPANT_RANK + 1]),
            None,
            "a rank above the governed bound was represented"
        );
        assert_eq!(
            StagedSpan::new(&[1; MAX_COOPERATIVE_PARTICIPANT_RANK + 1], 0, 1),
            None,
            "a stride vector above the governed bound was represented"
        );
    }

    // ---- The two-dimensional staging relation ----------------------------
    //
    // A 16x16 participant space, which is the shape ADR 0097 admits and the
    // shape the measured `contract_tiled` kernel
    // (`spikes/scheduling/metal_contraction_vertical/kernels.metal`) launches.
    // The fixtures below differ from the rank-one ones above in the participant
    // shape, the span ranks, and the contracted extent the split needs — and in
    // nothing else, so a rejection names the widened rule rather than a
    // difference the fixture happened to carry.

    /// One side of the measured kernel's square tile.
    const TILE_EXTENT: u64 = 16;
    /// The tile's participants, which is also its launched workgroup width.
    const TILE_PARTICIPANTS: u64 = TILE_EXTENT * TILE_EXTENT;
    /// The split the two-dimensional fixture covers its contributors with.
    const TILED_SPLIT: ContributorPartition = ContributorPartition {
        partitions: TILE_PARTICIPANTS,
        contributors_per_partition: 2,
    };

    /// A verifying tile over a 16x16 participant space.
    ///
    /// Its staged accesses are the rank-two spelling of the rank-one fixture's:
    /// each participant writes its own slot, and every participant reads the
    /// whole staged set. The `[1, 16]` write is the *transposed* form — the
    /// exact profile `16 * (l % 16) + (l / 16)` that no single-term relation
    /// over a linear coordinate expresses — so the fixture is a statement of the
    /// thing this widening exists for rather than a rank-one tile wearing two
    /// extents.
    fn tiled_tile_fixture() -> CooperativeTile {
        let participants = ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT])
            .expect("rank two is within the bound");
        let range = ParticipantRange {
            first: 0,
            count: TILE_PARTICIPANTS,
        };
        let tile = CooperativeTile {
            coordinates: LocalCoordinates {
                source: LocalCoordinateSource::LocalWorkgroupPosition,
                participants,
            },
            rounds: 1,
            staging: vec![tile_staging(TILE_PARTICIPANTS, PhaseId::new(1))],
            phases: vec![
                CooperativePhase {
                    id: PhaseId::FIRST,
                    participation: range,
                    writes: vec![StagedWrite {
                        staging: StagingId::FIRST,
                        span: StagedSpan::new(&[1, TILE_EXTENT], 0, 1)
                            .expect("rank two is within the bound"),
                    }],
                    reads: Vec::new(),
                },
                CooperativePhase {
                    id: PhaseId::new(1),
                    participation: range,
                    writes: Vec::new(),
                    reads: vec![StagedRead {
                        staging: StagingId::FIRST,
                        span: StagedSpan::new(&[0, 0], 0, TILE_PARTICIPANTS)
                            .expect("rank two is within the bound"),
                    }],
                },
            ],
            synchronization: Vec::new(),
            commit: ParticipantRange { first: 0, count: 1 },
        };
        let subject =
            required_subject(&tile.visibility_edges()).expect("the handoff states one subject");
        CooperativeTile {
            synchronization: vec![SynchronizationPoint {
                id: SyncPointId::FIRST,
                subject,
                placement: SynchronizationPlacement::PhaseBoundary {
                    preceding: PhaseId::FIRST,
                    following: PhaseId::new(1),
                },
                participants: range,
                convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
            }],
            ..tile
        }
    }

    /// Applies one edit to the two-dimensional fixture and builds it.
    fn tiled_perturbed(edit: impl FnOnce(&mut CooperativeTile)) -> ScheduledRegionBuilder {
        let mut tile = tiled_tile_fixture();
        edit(&mut tile);
        cooperative_builder_parts(
            TILED_SPLIT,
            TILE_PARTICIPANTS * TILED_SPLIT.contributors_per_partition,
            cooperative_topology_with(tile, TILED_SPLIT),
            reassociating_numerical(),
        )
    }

    /// A tile over a two-dimensional participant space verifies.
    #[test]
    fn a_two_dimensional_cooperative_tile_verifies() {
        let verified = tiled_perturbed(|_| {})
            .build()
            .expect("the two-dimensional fixture verifies");
        assert_eq!(
            verified.requirements().threads_per_workgroup,
            u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32")
        );
        // The extent product is the participant count and the launched width;
        // the shape is what the rank-one form could not state.
        let tile = cooperative_tile(&verified.region().schedule.reduction)
            .expect("the topology carries its tile");
        assert_eq!(tile.coordinates.participants.rank(), 2);
        assert_eq!(
            tile.coordinates.participants.extents(),
            [TILE_EXTENT, TILE_EXTENT]
        );
        assert_eq!(
            tile.coordinates.participants.participants(),
            Some(TILE_PARTICIPANTS)
        );
    }

    /// The measured 16x16 kernel's four staged accesses are all statable.
    ///
    /// Each with a *contiguous* count, which is what `StagedSpan` addresses, and
    /// each enumerating exactly the slots the kernel's own source indexes. The
    /// two writes address one slot per participant and the two reads address
    /// sixteen contiguous slots per participant, so the widened relation states
    /// the tiling rather than encoding it.
    ///
    /// The stride table is ADR 0097's, and this is the substitution that turns
    /// it from arithmetic on paper into an observed enumeration.
    #[test]
    fn the_measured_tile_kernels_four_staged_accesses_are_all_statable() {
        let space = ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT])
            .expect("rank two is within the bound");
        let slots = |strides: &[u64], count: u64| {
            CooperativeTile::addressed_slots(
                space,
                StagedSpan::new(strides, 0, count).expect("rank two is within the bound"),
            )
            .expect("every address is representable")
        };

        // `a_tile[local_m * TILE + local_n]`: one slot per participant, and the
        // 256 participants cover the 256 slots exactly once.
        let a_write = slots(&[TILE_EXTENT, 1], 1);
        assert_eq!(a_write.len(), 256);
        let mut sorted = a_write.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 256);
        assert_eq!(a_write[0], 0);
        // Participant (0, 1) is linear index 1 and holds slot 1.
        assert_eq!(a_write[1], 1);
        // Participant (1, 0) is linear index 16 and holds slot 16.
        assert_eq!(a_write[16], 16);

        // `b_tile[local_n * TILE + local_m]`: the transpose, and the exact pair
        // of points that refutes every single-term relation over a linear
        // coordinate — `w(1) = 16` while `w(16) = 1`.
        let b_write = slots(&[1, TILE_EXTENT], 1);
        assert_eq!(b_write.len(), 256);
        assert_eq!(b_write[0], 0);
        assert_eq!(b_write[1], 16);
        assert_eq!(b_write[16], 1);
        let mut sorted = b_write.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 256, "the transposed write is a bijection");

        // `a_tile[local_m * TILE + kk]`, `kk` in `0..16`: sixteen contiguous
        // slots per participant, many-to-one in the column dimension.
        let a_read = slots(&[TILE_EXTENT, 0], TILE_EXTENT);
        assert_eq!(a_read.len(), 256 * 16);
        assert_eq!(&a_read[..16], &(0..16).collect::<Vec<_>>()[..]);
        // Participant (1, 0), linear index 16, reads the run beginning at 16.
        assert_eq!(
            &a_read[16 * 16..16 * 16 + 16],
            &(16..32).collect::<Vec<_>>()[..]
        );

        // `b_tile[local_n * TILE + kk]`: the transpose of the read, so
        // participant (0, 1) — linear index 1 — reads the run beginning at 16.
        let b_read = slots(&[0, TILE_EXTENT], TILE_EXTENT);
        assert_eq!(b_read.len(), 256 * 16);
        assert_eq!(&b_read[..16], &(0..16).collect::<Vec<_>>()[..]);
        assert_eq!(&b_read[16..32], &(16..32).collect::<Vec<_>>()[..]);
    }

    /// The occupancy map still refuses two writers reaching one slot.
    ///
    /// ADR 0097's own case, watched failing rather than asserted: perturbing the
    /// transposed write's strides from `[1, 16]` to `[16, 16]` sends participant
    /// `(0, 1)` and participant `(1, 0)` both to slot 16, and no point separates
    /// two writes inside one phase. Disjointness is keyed on *slots*, so
    /// re-indexing the participant domain does not weaken it.
    #[test]
    fn two_writers_reaching_one_slot_in_one_round_are_still_refused() {
        let space = ParticipantSpace::new(&[TILE_EXTENT, TILE_EXTENT])
            .expect("rank two is within the bound");
        let colliding = StagedSpan::new(&[TILE_EXTENT, TILE_EXTENT], 0, 1)
            .expect("rank two is within the bound");
        // The collision itself, before the rule that refuses it: two distinct
        // participants, one slot.
        let addressed = CooperativeTile::addressed_slots(space, colliding)
            .expect("every address is representable");
        assert_eq!(addressed[1], 16, "participant (0, 1) addresses slot 16");
        assert_eq!(addressed[16], 16, "participant (1, 0) addresses slot 16");

        // The allocation is widened to hold the perturbed span's furthest
        // address — `15 * 16 + 15 * 16` — so that the capacity rule, which is a
        // different refusal, does not fire first and hide the one under test.
        // Coverage would fail on this tile too, and does not get the chance:
        // the occupancy walk refuses the second writer before it finishes.
        assert_eq!(
            cooperative_rejection(tiled_perturbed(|tile| {
                tile.staging[0].slots = 15 * TILE_EXTENT + 15 * TILE_EXTENT + 1;
                tile.phases[0].writes[0].span = colliding;
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::StagingConflict,
            }
        );
    }

    /// The workgroup-width equality is over the extent *product*, and still fires.
    ///
    /// Perturbing the extents to a shape whose product is not the launched width
    /// is refused; perturbing them to a *different shape with the same product*
    /// is not, which is the whole content of the rule generalizing from a count
    /// to a product. The launch plan carries no threadgroup shape to compare
    /// against, and ADR 0097 records that as a stated deferral rather than an
    /// omission — so this test pins what the rule does decide, not what a reader
    /// might hope it does.
    #[test]
    fn the_workgroup_width_equality_is_over_the_extent_product() {
        for extents in [
            vec![TILE_EXTENT, TILE_EXTENT - 1],
            vec![TILE_EXTENT, TILE_EXTENT + 1],
            vec![TILE_EXTENT, 1],
        ] {
            let space = ParticipantSpace::new(&extents).expect("rank two is within the bound");
            assert_eq!(
                cooperative_rejection(tiled_perturbed(|tile| {
                    tile.coordinates.participants = space;
                })),
                ScheduledRegionDiagnostic::CooperativeTile {
                    rule: CooperativeTileRule::ParticipantConvergence,
                },
                "extents {extents:?} were admitted against a launch of {TILE_PARTICIPANTS}"
            );
        }
        // And from the other side: holding the space and narrowing the launch
        // is the perturbation the rank-one fixture already used, so the rule is
        // reachable from either fact it relates.
        let mut builder = tiled_perturbed(|_| {});
        let schedule = builder
            .schedule
            .as_mut()
            .expect("the fixture sets a schedule");
        schedule.threads_per_workgroup = 255;
        schedule.launch.threads_per_workgroup = 255;
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ParticipantConvergence,
            }
        );
        // A different arrangement of the same 256 participants passes the
        // equality, because the product is what it compares.
        tiled_perturbed(|tile| {
            tile.coordinates.participants =
                ParticipantSpace::new(&[4, 64]).expect("rank two is within the bound");
            tile.phases[0].writes[0].span =
                StagedSpan::new(&[64, 1], 0, 1).expect("rank two is within the bound");
        })
        .build()
        .expect("a 4x64 arrangement of the same participants verifies");
    }

    /// A staged span whose rank disagrees with the tile's is refused by name.
    ///
    /// Both directions, because the two are different mistakes a producer makes
    /// and neither is wrong on its own terms: a rank-two stride vector over a
    /// rank-one space, and a rank-one one over a rank-two space. The read side
    /// is checked separately from the write side, because a read's addressed set
    /// is discarded and a rule that only fired on writes would admit the exact
    /// silently-wrong broadcast this vocabulary exists to refuse.
    #[test]
    fn a_staged_span_whose_rank_disagrees_with_the_tile_is_refused() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[0].writes[0].span =
                    StagedSpan::new(&[1, 0], 0, 1).expect("rank two is within the bound");
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::SpanRank,
            }
        );
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.phases[1].reads[0].span =
                    StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::SpanRank,
            }
        );
        // And the same disagreement from the other side: a rank-two space whose
        // spans still state one stride each.
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.coordinates.participants =
                    ParticipantSpace::new(&[3, 1]).expect("rank two is within the bound");
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::SpanRank,
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
        let ReductionTopology::CooperativeWorkgroup { coverage, .. } =
            &mut builder.schedule.as_mut().unwrap().reduction
        else {
            panic!("expected a cooperative topology")
        };
        *coverage = ContributorCoverage::Exact(ContributorPartition {
            partitions: 3,
            contributors_per_partition: 3,
        });
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::ContributorCoverage {
                rule: ContributorCoverageRule::ExactCoverage,
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

    /// The `v7` step moves the domain and exactly the elementary payload bytes.
    ///
    /// Compared against the retained `v6` identity structurally rather than by
    /// bare inequality: the payload delta must be precisely the two inserted
    /// one-byte rows — the reciprocal-transform permission and the
    /// approximate-intrinsic envelope — between the signed-zero permission and
    /// the NaN assumption, so a step that moved anything else fails here
    /// instead of hiding inside "the bytes differ".
    #[test]
    fn the_elementary_dimension_step_moves_domain_and_payload() {
        // Eighteen bytes of `tiler.schedule.vN\0`, so thirty-six hex digits.
        const SEPARATOR: usize = 36;

        let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
            .build()
            .unwrap();
        let mut hex = String::new();
        for byte in verified.canonical_identity().as_bytes() {
            write!(&mut hex, "{byte:02x}").unwrap();
        }
        assert_eq!(hex, STRICT_F32_REGION_IDENTITY_HEX);
        assert_ne!(
            STRICT_F32_REGION_IDENTITY_HEX[..SEPARATOR],
            STRICT_F32_REGION_IDENTITY_HEX_V6[..SEPARATOR]
        );
        // The two spellings differ by exactly four hex digits — the two
        // inserted permission/envelope tag bytes — at one position inside the
        // numerical record. Locate the insertion by the longest common prefix
        // and check the suffixes re-align after it.
        let new = &STRICT_F32_REGION_IDENTITY_HEX[SEPARATOR..];
        let old = &STRICT_F32_REGION_IDENTITY_HEX_V6[SEPARATOR..];
        assert_eq!(new.len(), old.len() + 4, "two one-byte rows were inserted");
        let prefix = new
            .as_bytes()
            .iter()
            .zip(old.as_bytes())
            .take_while(|(new, old)| new == old)
            .count();
        assert_eq!(
            &new[prefix + 4..],
            &old[prefix..],
            "every byte after the two inserted rows is carried unchanged"
        );
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
            perturb_tile(|tile| {
                tile.phases[0].writes[0].span =
                    StagedSpan::new(&[2], 0, 1).expect("rank one is within the bound");
            }),
            perturb_tile(|tile| tile.phases[0].writes[0].span.offset = 1),
            perturb_tile(|tile| tile.phases[1].reads[0].span.count = 2),
            perturb_tile(|tile| tile.commit = ParticipantRange { first: 2, count: 1 }),
            perturb_tile(|tile| {
                tile.phases[1].participation = ParticipantRange { first: 0, count: 2 };
            }),
            perturb_tile(|tile| {
                tile.coordinates.participants =
                    ParticipantSpace::new(&[4]).expect("rank one is within the bound");
            }),
            // The participant *shape* separates identity too, not only the
            // count it determines: a tile whose 3 participants are arranged
            // `[3, 1]` states a different relation from one arranged `[3]`, and
            // the span ranks that go with each differ, so the two must not share
            // bytes even though both launch three invocations.
            perturb_tile(|tile| {
                tile.coordinates.participants =
                    ParticipantSpace::new(&[3, 1]).expect("rank two is within the bound");
                tile.phases[0].writes[0].span =
                    StagedSpan::new(&[1, 0], 0, 1).expect("rank two is within the bound");
                tile.phases[1].reads[0].span =
                    StagedSpan::new(&[0, 0], 0, 3).expect("rank two is within the bound");
            }),
            // The round count separates identity like every other tile field: a
            // schedule that rewrites its staging is a different program from one
            // that stages once, and the two must not share bytes.
            perturb_tile(|tile| tile.rounds = 2),
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

    // ---- Operand-sharing cooperative contraction -------------------------
    //
    // Exact-divisible first pass: a 32×32 output blocked 16×16, K = 16 tiled
    // by 16, every participant committing its own output. The staged accesses
    // are ADR 0097's four measured spans. The one-committer fixtures above are
    // untouched.

    const OUTPUT_EXTENT: u64 = 32;
    const OUTPUT_BLOCK: u64 = 16;
    const CONTRACTED_EXTENT: u64 = 16;
    const CONTRACTED_TILE: u64 = 16;
    const OUTPUT_POSITIONS: u64 = OUTPUT_EXTENT * OUTPUT_EXTENT;

    fn operand_tile_fixture() -> CooperativeTile {
        let participants = ParticipantSpace::new(&[OUTPUT_BLOCK, OUTPUT_BLOCK])
            .expect("rank two is within the bound");
        let range = ParticipantRange {
            first: 0,
            count: TILE_PARTICIPANTS,
        };
        let a = StagingId::FIRST;
        let b = StagingId::new(1);
        let tile = CooperativeTile {
            coordinates: LocalCoordinates {
                source: LocalCoordinateSource::LocalWorkgroupPosition,
                participants,
            },
            rounds: 1,
            staging: vec![
                tile_staging(TILE_PARTICIPANTS, PhaseId::new(1)),
                WorkgroupStaging {
                    id: b,
                    ..tile_staging(TILE_PARTICIPANTS, PhaseId::new(1))
                },
            ],
            phases: vec![
                CooperativePhase {
                    id: PhaseId::FIRST,
                    participation: range,
                    writes: vec![
                        StagedWrite {
                            staging: a,
                            span: StagedSpan::new(&[OUTPUT_BLOCK, 1], 0, 1)
                                .expect("rank two is within the bound"),
                        },
                        StagedWrite {
                            staging: b,
                            span: StagedSpan::new(&[1, OUTPUT_BLOCK], 0, 1)
                                .expect("rank two is within the bound"),
                        },
                    ],
                    reads: Vec::new(),
                },
                CooperativePhase {
                    id: PhaseId::new(1),
                    participation: range,
                    writes: Vec::new(),
                    reads: vec![
                        StagedRead {
                            staging: a,
                            span: StagedSpan::new(&[OUTPUT_BLOCK, 0], 0, OUTPUT_BLOCK)
                                .expect("rank two is within the bound"),
                        },
                        StagedRead {
                            staging: b,
                            span: StagedSpan::new(&[0, OUTPUT_BLOCK], 0, OUTPUT_BLOCK)
                                .expect("rank two is within the bound"),
                        },
                    ],
                },
            ],
            synchronization: Vec::new(),
            commit: range,
        };
        let subject =
            required_subject(&tile.visibility_edges()).expect("the handoff states one subject");
        CooperativeTile {
            synchronization: vec![SynchronizationPoint {
                id: SyncPointId::FIRST,
                subject,
                placement: SynchronizationPlacement::PhaseBoundary {
                    preceding: PhaseId::FIRST,
                    following: PhaseId::new(1),
                },
                participants: range,
                convergence: ConvergenceEvidence::EveryParticipantReachesThePoint,
            }],
            ..tile
        }
    }

    fn operand_contraction_builder(
        admitted: &crate::schedule::ExactCooperativeContraction,
        tile: CooperativeTile,
    ) -> ScheduledRegionBuilder {
        let output = Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]);
        let contracted = Shape::from_dims([CONTRACTED_EXTENT]);
        let left = Shape::from_dims([OUTPUT_EXTENT, CONTRACTED_EXTENT]);
        let right = Shape::from_dims([OUTPUT_EXTENT, CONTRACTED_EXTENT]);
        let operand_map = |free_position, operand: Shape| LogicalAccess::ContractionOperand {
            operand_shape: operand,
            output_shape: output.clone(),
            contracted_shape: contracted.clone(),
            sources: vec![
                ContractionAxisSource::Output {
                    position: free_position,
                },
                ContractionAxisSource::Contracted { position: 0 },
            ],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(7));
        builder.iteration_shape(output.clone()).unwrap();
        for (witness, map) in [
            (0, operand_map(0, left.clone())),
            (1, operand_map(1, right.clone())),
        ] {
            builder
                .push_access(Access {
                    tensor: TensorRole::Input,
                    component_role: None,
                    mode: AccessMode::Read,
                    map,
                    bounds: BoundsWitnessId::new(witness),
                    ownership: None,
                })
                .unwrap();
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(witness),
                    tensor: TensorRole::Input,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: OUTPUT_EXTENT * CONTRACTED_EXTENT,
                    },
                })
                .unwrap();
        }
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(2),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(2),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: OUTPUT_POSITIONS,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: OUTPUT_POSITIONS,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            })
            .unwrap();
        builder.numerical(reassociating_numerical()).unwrap();
        let threads = u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32");
        builder
            .schedule(KernelSchedule {
                binding: admitted.binding.clone(),
                work_items: OUTPUT_POSITIONS,
                threads_per_workgroup: threads,
                tail: TailPolicy::Exact,
                output_owner: OwnershipWitnessId::new(0),
                reduction: ReductionTopology::CooperativeContraction {
                    tile,
                    contracted_shape: contracted,
                    contracted_tile: admitted.contracted_tile.clone(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                launch: LaunchPlan {
                    grid_threads: OUTPUT_POSITIONS,
                    threads_per_workgroup: threads,
                    zero_work_skips_dispatch: true,
                },
            })
            .unwrap();
        builder
    }

    fn admitted_operand_tile() -> crate::schedule::ExactCooperativeContraction {
        crate::schedule::admit_exact_cooperative_contraction(
            &Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([CONTRACTED_EXTENT]),
            &Shape::from_dims([CONTRACTED_TILE]),
        )
        .expect("the exact 32×32 / 16 tile divides")
    }

    /// An exactly tiled output domain verifies under the blocked binding.
    #[test]
    fn an_exact_cooperative_contraction_verifies_under_the_blocked_binding() {
        let admitted = admitted_operand_tile();
        assert_eq!(admitted.rounds, 1);
        let verified = operand_contraction_builder(&admitted, operand_tile_fixture())
            .build()
            .expect("the exact-divisible operand-sharing tile verifies");
        assert_eq!(verified.region().schedule.work_items, OUTPUT_POSITIONS);
        assert_eq!(
            verified.region().index.ownership_proof.kind,
            OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: OUTPUT_POSITIONS,
            }
        );
        let tile = cooperative_tile(&verified.region().schedule.reduction)
            .expect("the topology carries its tile");
        assert_eq!(tile.commit.count, TILE_PARTICIPANTS);
        assert_eq!(verified.requirements().threads_per_workgroup, 256);
    }

    /// Preflight refuses a non-divisible output block by name.
    #[test]
    fn a_non_divisible_output_block_is_refused_in_preflight() {
        let refusal = crate::schedule::admit_exact_cooperative_contraction(
            &Shape::from_dims([33, OUTPUT_EXTENT]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([CONTRACTED_EXTENT]),
            &Shape::from_dims([CONTRACTED_TILE]),
        )
        .expect_err("33 is not divisible by 16");
        assert_eq!(
            refusal,
            crate::schedule::CooperativeContractionAdmission::OutputBlockNotDivisible {
                axis: 0,
                output: 33,
                block: OUTPUT_BLOCK,
            }
        );
        assert_eq!(
            refusal.rule(),
            "cooperative-contraction-output-block-not-divisible"
        );
    }

    /// Preflight refuses a non-divisible contracted tile by name.
    #[test]
    fn a_non_divisible_contracted_tile_is_refused_in_preflight() {
        let refusal = crate::schedule::admit_exact_cooperative_contraction(
            &Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([17]),
            &Shape::from_dims([CONTRACTED_TILE]),
        )
        .expect_err("17 is not divisible by 16");
        assert_eq!(
            refusal,
            crate::schedule::CooperativeContractionAdmission::ContractedTileNotDivisible {
                axis: 0,
                contracted: 17,
                tile: CONTRACTED_TILE,
            }
        );
        assert_eq!(
            refusal.rule(),
            "cooperative-contraction-contracted-tile-not-divisible"
        );
    }

    /// Two invocations claiming one output is an overlap, not a gap.
    #[test]
    fn a_blocked_map_with_an_overlapping_axis_is_refused() {
        let mut builder =
            operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
        let ExecutionBinding::BlockedWorkgroup { workgroups, .. } =
            &mut builder.schedule.as_mut().unwrap().binding
        else {
            panic!("the fixture carries the blocked binding")
        };
        *workgroups = Shape::from_dims([3, 2]);
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::BlockedWorkgroup {
                rule: BlockedWorkgroupRule::MappingOverlap,
            }
        );
    }

    /// An output coordinate with no preimage is a gap, not an overlap.
    #[test]
    fn a_blocked_map_with_a_gapped_axis_is_refused() {
        let mut builder =
            operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
        let ExecutionBinding::BlockedWorkgroup { workgroups, .. } =
            &mut builder.schedule.as_mut().unwrap().binding
        else {
            panic!("the fixture carries the blocked binding")
        };
        *workgroups = Shape::from_dims([1, 2]);
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::BlockedWorkgroup {
                rule: BlockedWorkgroupRule::MappingGap,
            }
        );
    }

    /// Ownership is not `work_items / participants` merely because a tile is present.
    #[test]
    fn a_helper_that_infers_reduction_ownership_from_a_tile_is_refused() {
        let mut builder =
            operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
        // The false helper would report 1024 / 256 = 4 owned positions.
        builder.ownership_proof = Some(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 4 },
        });
        builder.bounds_proofs[2].kind = BoundsProofKind::LinearRange { element_count: 4 };
        assert_eq!(
            builder.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::ProofReference]
        );
    }

    /// The one-committer tile still refuses every participant committing.
    #[test]
    fn the_one_committer_tile_still_refuses_every_participant_committing() {
        assert_eq!(
            cooperative_rejection(perturbed(|tile| {
                tile.commit = ParticipantRange { first: 0, count: 3 };
            })),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::CommitOwnership,
            }
        );
    }

    /// The operand-sharing tile refuses a one-committer range.
    #[test]
    fn the_operand_sharing_tile_refuses_a_single_committer() {
        let mut tile = operand_tile_fixture();
        tile.commit = ParticipantRange { first: 0, count: 1 };
        assert_eq!(
            cooperative_rejection(operand_contraction_builder(&admitted_operand_tile(), tile)),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::OperandTileCommit,
            }
        );
    }

    fn predicated_operand_builder(
        output_m: u64,
        output_n: u64,
        contracted: u64,
    ) -> (
        crate::schedule::PredicatedCooperativeContraction,
        ScheduledRegionBuilder,
    ) {
        let admitted = crate::schedule::admit_predicated_cooperative_contraction(
            &Shape::from_dims([output_m, output_n]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([contracted]),
            &Shape::from_dims([CONTRACTED_TILE]),
        )
        .expect("the predicated launch is representable");
        let output = Shape::from_dims([output_m, output_n]);
        let contracted_shape = Shape::from_dims([contracted]);
        let left = Shape::from_dims([output_m, contracted]);
        let right = Shape::from_dims([output_n, contracted]);
        let work_items = output_m.checked_mul(output_n).expect("M×N fits");
        let operand_map = |free_position, operand: Shape| LogicalAccess::ContractionOperand {
            operand_shape: operand,
            output_shape: output.clone(),
            contracted_shape: contracted_shape.clone(),
            sources: vec![
                ContractionAxisSource::Output {
                    position: free_position,
                },
                ContractionAxisSource::Contracted { position: 0 },
            ],
            order: ContributorOrder::OriginalAxisLexicographic,
        };
        let mut tile = operand_tile_fixture();
        tile.rounds = admitted.rounds;
        let mut builder = ScheduledRegionBuilder::new(RegionId::new(7));
        builder.iteration_shape(output.clone()).unwrap();
        for (witness, ordinal, map) in [
            (0, 0, operand_map(0, left.clone())),
            (1, 1, operand_map(1, right.clone())),
        ] {
            builder
                .push_access(Access {
                    tensor: TensorRole::Input,
                    component_role: None,
                    mode: AccessMode::Read,
                    map,
                    bounds: BoundsWitnessId::new(witness),
                    ownership: None,
                })
                .unwrap();
            builder
                .push_bounds_proof(BoundsProof {
                    id: BoundsWitnessId::new(witness),
                    tensor: TensorRole::Input,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: match ordinal {
                            0 => output_m.checked_mul(contracted).expect("MK fits"),
                            _ => output_n.checked_mul(contracted).expect("NK fits"),
                        },
                    },
                })
                .unwrap();
        }
        builder
            .push_access(Access {
                tensor: TensorRole::Output,
                component_role: None,
                mode: AccessMode::Write,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(2),
                ownership: Some(OwnershipWitnessId::new(0)),
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(2),
                tensor: TensorRole::Output,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: work_items,
                },
            })
            .unwrap();
        builder
            .ownership_proof(OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: work_items,
                },
            })
            .unwrap();
        builder
            .scalar_program(ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            })
            .unwrap();
        builder.numerical(reassociating_numerical()).unwrap();
        let threads = u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32");
        builder
            .schedule(KernelSchedule {
                binding: admitted.binding.clone(),
                work_items: admitted.work_items,
                threads_per_workgroup: threads,
                tail: TailPolicy::Predicated,
                output_owner: OwnershipWitnessId::new(0),
                reduction: ReductionTopology::CooperativeContraction {
                    tile,
                    contracted_shape,
                    contracted_tile: admitted.contracted_tile.clone(),
                    order: ContributorOrder::OriginalAxisLexicographic,
                    accumulation: ArithmeticType::F32,
                    permits_reassociation: true,
                    permits_permutation: false,
                },
                launch: LaunchPlan {
                    grid_threads: admitted.grid_threads,
                    threads_per_workgroup: threads,
                    zero_work_skips_dispatch: true,
                },
            })
            .unwrap();
        (admitted, builder)
    }

    /// Exact and Predicated [32, 32] blocks under the same binding stay distinct.
    #[test]
    fn exact_and_predicated_neighbours_keep_distinct_identities() {
        let exact = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture())
            .build()
            .expect("the exact neighbour verifies");
        let (_, predicated_builder) =
            predicated_operand_builder(OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT);
        let predicated = predicated_builder
            .build()
            .expect("the predicated neighbour verifies");
        assert_eq!(
            exact.region().schedule.work_items,
            predicated.region().schedule.work_items
        );
        assert_eq!(
            exact.region().schedule.launch.grid_threads,
            predicated.region().schedule.launch.grid_threads
        );
        assert_ne!(
            exact.canonical_identity().as_bytes(),
            predicated.canonical_identity().as_bytes()
        );
        let mut exact_hex = String::new();
        let mut predicated_hex = String::new();
        for byte in exact.canonical_identity().as_bytes() {
            write!(&mut exact_hex, "{byte:02x}").unwrap();
        }
        for byte in predicated.canonical_identity().as_bytes() {
            write!(&mut predicated_hex, "{byte:02x}").unwrap();
        }
        assert!(exact_hex.contains("01"), "Exact keeps tail tag 0x01");
        assert!(
            predicated_hex.contains("02"),
            "Predicated appends tail tag 0x02"
        );
    }

    /// Partial free extents, exact neighbours, zero work, overflow, and nondivisible K.
    #[test]
    fn predicated_admission_covers_the_required_shapes() {
        let cases = [
            (1, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
            (10, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
            (OUTPUT_EXTENT, 10, CONTRACTED_EXTENT, true),
            (10, 10, CONTRACTED_EXTENT, true),
            (OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
            (0, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
        ];
        for (m, n, k, ok) in cases {
            let admitted = crate::schedule::admit_predicated_cooperative_contraction(
                &Shape::from_dims([m, n]),
                &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
                &Shape::from_dims([k]),
                &Shape::from_dims([CONTRACTED_TILE]),
            );
            assert_eq!(admitted.is_ok(), ok, "M={m} N={n} K={k}");
            if let Ok(admitted) = admitted {
                if m == 0 || n == 0 {
                    assert_eq!(admitted.work_items, 0);
                    assert_eq!(admitted.grid_threads, 0);
                } else {
                    assert_eq!(admitted.work_items, m * n);
                    assert!(admitted.grid_threads >= admitted.work_items);
                    assert_eq!(admitted.grid_threads % TILE_PARTICIPANTS, 0);
                }
                let (_, builder) = predicated_operand_builder(m, n, k);
                builder
                    .build()
                    .unwrap_or_else(|error| panic!("M={m} N={n} K={k} refused: {error:?}"));
            }
        }
        let overflow = crate::schedule::admit_predicated_cooperative_contraction(
            &Shape::from_dims([u64::MAX, u64::MAX]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([CONTRACTED_EXTENT]),
            &Shape::from_dims([CONTRACTED_TILE]),
        );
        assert_eq!(
            overflow,
            Err(crate::schedule::CooperativeContractionAdmission::ShapeProductOverflow)
        );
        let nondivisible_k = crate::schedule::admit_predicated_cooperative_contraction(
            &Shape::from_dims([10, 10]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([17]),
            &Shape::from_dims([CONTRACTED_TILE]),
        );
        assert_eq!(
            nondivisible_k
                .expect_err("17 is not divisible by 16")
                .rule(),
            "cooperative-contraction-contracted-tile-not-divisible"
        );
    }

    /// Predicated never rewrites itself to Exact when the block happens to divide.
    #[test]
    fn a_divisible_predicated_proposal_does_not_normalize_to_exact() {
        let (admitted, builder) =
            predicated_operand_builder(OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT);
        let verified = builder
            .build()
            .expect("divisible Predicated still verifies");
        assert_eq!(verified.region().schedule.tail, TailPolicy::Predicated);
        assert_eq!(admitted.grid_threads, OUTPUT_POSITIONS);
        assert_ne!(
            format!("{:?}", verified.region().schedule.tail),
            format!("{:?}", TailPolicy::Exact)
        );
    }

    /// The blocked binding is required; `GlobalLinearInvocation` is not a default.
    #[test]
    fn a_cooperative_contraction_without_the_blocked_binding_is_refused() {
        let mut builder =
            operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture());
        builder.schedule.as_mut().unwrap().binding = ExecutionBinding::GlobalLinearInvocation;
        assert_eq!(
            cooperative_rejection(builder),
            ScheduledRegionDiagnostic::BlockedWorkgroup {
                rule: BlockedWorkgroupRule::BindingRequired,
            }
        );
    }

    /// The existing one-committer fixture is re-pinned under the elementary
    /// numerical dimensions.
    #[test]
    fn existing_one_committer_schedule_encodings_keep_their_bytes() {
        let verified = cooperative_builder(cooperative_tile_fixture())
            .build()
            .expect("the one-committer fixture still verifies");
        let bytes = verified.canonical_identity().as_bytes();
        assert!(
            bytes.starts_with(b"tiler.schedule.v7\0"),
            "the schedule domain must carry the elementary-dimension step"
        );
        assert!(
            bytes.contains(&0x35),
            "the one-committer topology tag must still appear"
        );
        assert!(
            !bytes[18..].contains(&0x37),
            "the new topology tag must not appear in an old region's payload; \
             the separator is excluded because `v7` spells the byte 0x37"
        );
        // Binding tag 0x01 sits at a known offset after the numerical payload;
        // the new 0x02 binding is an appended alternative, so an old region
        // that still carries GlobalLinearInvocation cannot encode it.
        let mut hex = String::new();
        for byte in bytes {
            write!(&mut hex, "{byte:02x}").unwrap();
        }
        // Pin of the one-committer `[2, 6] -> [2]` cooperative fixture at
        // `4333df31`. A payload move here is a domain step nobody authorized.
        assert_eq!(hex, ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX);
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
