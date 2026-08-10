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

use super::cooperative::{
    AntiDependencyEdge, ContributorArrival, CooperativeTile, ParticipantRange, ParticipantSpace,
    StagedSpan, VisibilityEdge,
};
use super::error::{
    CooperativeTileRule, ScheduleBuildError, ScheduleComponent, ScheduleLimitKind,
    ScheduledRegionBuildError, ScheduledRegionDiagnostic,
};
use super::handles::{InputOrdinal, RegionId, StagingId};
use super::model::{
    Access, AccessMode, BoundsProof, BoundsProofKind, CanonicalScheduledRegionIdentity,
    ContractionAxisSource, ContributorOrder, ExecutionBinding, IndexRegion, KernelSchedule,
    LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionPass, ReductionTopology,
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
    /// The fold's declared contributor domain, wherever the plan placed it: the
    /// first input tensor when the program folds it directly, or a materialized
    /// intermediate when a prologue region wrote it.
    DeclaredDomain,
}

impl ContributorTensor {
    /// Returns whether one read's boundary tensor discharges this obligation.
    fn admits(self, tensor: TensorRole) -> bool {
        match self {
            Self::Exactly(required) => tensor == required,
            Self::DeclaredDomain => tensor == TensorRole::Intermediate || tensor == FIRST_INPUT,
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
    // The operands bind two distinct program inputs in canonical declaration
    // order. They need not be the first two: a contraction may read a subset of
    // a wider declared interface. Strict ascent separately refuses a repeated
    // tensor and a descending spelling of the same pair, so every admitted
    // region has one access order and keeps the program's ABI ordinals.
    let (
        TensorRole::Input {
            ordinal: left_ordinal,
        },
        TensorRole::Input {
            ordinal: right_ordinal,
        },
    ) = (left.tensor, right.tensor)
    else {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    };
    if left_ordinal.get() >= right_ordinal.get()
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
/// Two obligations make an N-input region safe, and they are about different
/// things. **The count**: there must be exactly as many reads as the expression
/// has input leaves, or an expression could read an ordinal no access binds — a
/// load through a buffer the signature never declares. The expression's own
/// verifier already proved its ordinals are the dense `0..n`, so leaf `i` is
/// served by read `i` and the pairing is exhaustive rather than a sample.
/// **The binding**: each read must name a boundary tensor in the canonical order
/// [`reads_bind_boundary_tensors_in_order`] states, or a consumer that binds
/// buffers positionally would bind the wrong one without noticing. Two reads may
/// name one declared input when they address it differently, which that rule
/// states and bounds.
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
        || write.map != LogicalAccess::LinearIdentity
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
    {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    Ok(())
}

/// Returns whether one pointwise region's reads bind boundary tensors in the
/// canonical order.
///
/// **A read's position and its boundary role are separate facts.** A read's
/// *position* is the expression leaf it serves: `crate::kernel`'s
/// `emit_pointwise` looks a leaf's ordinal up among the values loaded in access
/// order, so position is what pairs a leaf with a buffer. A read's *tensor role*
/// is which boundary tensor that buffer binds, and [`TensorRole::Input`]'s
/// ordinal names one declared program input rather than the access position —
/// `CoverAssembly::from_plan` in `tiler-compiler` binds it against the program's
/// declared interface. Requiring the two to be equal made an elementwise
/// epilogue inexpressible: a region reading a materialized intermediate and the
/// program's third input has leaves `0` and `1`, and its second read must still
/// say `Input { ordinal: 2 }` or name the wrong tensor.
///
/// What that separation protects survives as three rules:
///
/// - **Declared input ordinals never descend, and a repeat is a dense read
///   followed by a mapped one.** A descending pair would be a second spelling of
///   one computation — one region with two identities. A repeat is admitted
///   because one expression may read one tensor through two different relations:
///   `a * permute(a)` needs a dense read *and* a reindexed read of declared
///   input `0`, and binding one access to both leaves is what once made that
///   program compile as `permute(a) * permute(a)`. The pair carries its own
///   canonical order rather than needing one imposed: the dense read leads, so
///   the two encodings of the pair are not both admissible and the region keeps
///   one spelling. Two *structural* relations on one ordinal stay refused for
///   the same reason the order exists — nothing ranks two relations against each
///   other, so the pair would have two spellings and no canonical one.
/// - **At most one read binds the materialized intermediate.** The role carries
///   no ordinal, so a second read leaves nothing to say which materialization
///   edge it binds — which is exactly why the repeated-read admission above
///   cannot extend to it. `CoverAssembly::from_plan` refuses that a layer up
///   under `cover-intermediate-read-attribution`; stating it here is what stops
///   an intrinsically ambiguous region from being built at all, for a producer
///   that never passes through a cover.
/// - **A program output is never read.** Refused by name rather than under a
///   wildcard, so a role added to the vocabulary later is a build error here
///   instead of silently inheriting an admission nobody checked it for.
fn reads_bind_boundary_tensors_in_order(reads: &[Access]) -> bool {
    let mut previous_input: Option<(u32, &LogicalAccess)> = None;
    let mut intermediate_reads = 0_usize;
    for read in reads {
        match read.tensor {
            TensorRole::Input { ordinal } => {
                let ordinal = ordinal.get();
                if let Some((previous_ordinal, previous_map)) = previous_input {
                    if ordinal < previous_ordinal {
                        return false;
                    }
                    // The repeat's canonical spelling, and the whole of what
                    // separates two admissible reads of one tensor from one read
                    // written twice: the dense read leads and the mapped one
                    // follows. Two dense reads address identically and so serve
                    // interchangeable leaves, and a mapped read ahead of a dense
                    // one is the same pair reversed.
                    if ordinal == previous_ordinal
                        && (*previous_map != LogicalAccess::LinearIdentity
                            || read.map == LogicalAccess::LinearIdentity)
                    {
                        return false;
                    }
                }
                previous_input = Some((ordinal, &read.map));
            }
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
/// obligation. Three maps satisfy that and no others do:
///
/// - [`LogicalAccess::LinearIdentity`], the dense one-to-one read.
/// - [`LogicalAccess::ReindexBijection`], whose decodes are required to tile the
///   iteration domain exactly, so every operand element is read once.
/// - [`LogicalAccess::BroadcastReplication`], whose decodes are required to name
///   distinct result axes and leave at least one replicated.
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
        LogicalAccess::LinearIdentity => true,
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
    // Every arm below differs in what it reads and agrees on what it writes.
    // The read obligation is per family, because a family's scalar program is
    // what decides which tensor holds its contributors — three of these four
    // bind the first input tensor, two because they carry a prologue over the
    // original input and the extrema fold because its pass reads the original
    // scores, while the bare sum reads whichever tensor holds its declared
    // contributor domain. The write obligation is [`CommittedTensor::CoverAssigned`] at all
    // four, because no fold's algebra distinguishes committing the caller's
    // answer from committing a value a later region reads, so widening one arm
    // and not its siblings would state a difference between them that does not
    // exist — and would drop the *fused* alternative for every reduction whose
    // result an epilogue consumes while keeping the materialized-prologue one.
    match (
        &region.index.scalar_program,
        &region.schedule.reduction,
        &read.map,
    ) {
        // The one family here whose contributor tensor is not fixed by its scalar
        // program, for the reason [`ContributorTensor::DeclaredDomain`] states:
        // the fold carries no prologue, so it reads whichever tensor holds its
        // declared contributor domain.
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
            && ContributorTensor::DeclaredDomain.admits(read.tensor)
            && CommittedTensor::CoverAssigned.admits(write.tensor) => {}
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
            && CommittedTensor::CoverAssigned.admits(write.tensor) => {}
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
            && CommittedTensor::CoverAssigned.admits(write.tensor) => {}
        // The squaring fold that carries its own epilogue. Every fold obligation
        // above is repeated here — the arm is the `SquaredSerialSum` one, and the
        // epilogue does not change what the *fold* reads or writes — plus the two
        // this variant owns:
        //
        // - **One leaf, which is the folded value.** This region reads one
        //   boundary tensor, so an epilogue with a second leaf would name a buffer
        //   nothing binds; the lowering supplies the accumulator for ordinal zero
        //   and has nothing to supply for ordinal one.
        // - **The epilogue must transform something.** A root that *is* the input
        //   leaf computes nothing, which is a second spelling of
        //   `SquaredSerialSum` and would give one program two identities.
        //
        // `is_valid` is required beside them for the reason a pointwise region
        // requires it: the expression arrives from a producer, and a malformed one
        // would reach the lowering as a forward reference or an unreachable node.
        (
            ScalarProgram::SquaredSerialSumThenEpilogue {
                axes,
                order,
                empty_identity_bits,
                epilogue,
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
            && CommittedTensor::CoverAssigned.admits(write.tensor)
            && epilogue.is_valid()
            && epilogue.input_count() == 1
            && !matches!(
                epilogue
                    .nodes()
                    .get(usize::try_from(epilogue.root().index()).unwrap_or(usize::MAX)),
                Some(super::pointwise::PointwiseF32Node::Input { .. })
            ) => {}
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
            && CommittedTensor::CoverAssigned.admits(write.tensor) => {}
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

/// What one scalar program's own algebra decides about a parallel split.
///
/// Both parallel topologies need exactly these facts and check different
/// structures over them, so they are derived once per topology
/// ([`multi_pass_family`], [`cooperative_family`]) rather than destructured
/// inline: a family admitted by one admission and not the other would otherwise
/// be a difference nobody states.
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
    /// Boundary-tensor obligation this pass's single read must discharge.
    ///
    /// There is deliberately no write counterpart on this struct, and the
    /// absence is the claim: a read's tensor varies by *family*, because a
    /// family's prologue is what decides whether it reads the original input,
    /// while a write's varies only by *pass* — every committing pass carries
    /// [`CommittedTensor::CoverAssigned`] and a split's staging pass carries
    /// [`CommittedTensor::Exactly`], identically for every family. Carrying it
    /// here would let a family declare a write target it has no authority over,
    /// and would invite two families to disagree about one cover's decision.
    read_tensor: ContributorTensor,
}

/// Decides one family's empty-domain obligation against a pass's contributors.
///
/// The identity-seeded arm requires the strict sum's `+0.0`, which every family
/// carrying an identity here shares — required at each admission rather than at
/// one of them, so a split cannot introduce a second empty-domain answer. The
/// identity-less arm requires a non-empty domain, which is what replaces the
/// constant the family has no correct value for.
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
/// **Exact coverage is a premise of that argument, not a detail of it**, and it
/// is what every split this vocabulary admits supplies:
/// [`super::model::ContributorPartition::covers`] rejects anything else. A split
/// covering a *padded* sequence has partitions whose real contributors may be
/// none, so the factor argument does not reach it and the family's stated padding
/// identity is what would discharge it instead — the separation
/// [`ScalarProgram::StrictSerialMaximum`] records. Nothing here admits such a
/// split; this notes which premise a later one would have to replace.
const fn empty_domain_is_satisfied(contract: EmptyDomainContract, contributors: u64) -> bool {
    match contract {
        EmptyDomainContract::Identity { bits } => bits == 0.0_f32.to_bits(),
        EmptyDomainContract::NoIdentity => contributors != 0,
    }
}

/// Resolves what one pass of a multi-dispatch split may be, from its program.
///
/// A fused prologue belongs to the pass that reads the original inputs: the final
/// pass reads partials, so re-applying scale and bias there would scale each
/// partial a second time and squaring one would square an already-folded value.
/// Both therefore admit a partial pass alone, and the final pass that consumes
/// their partials is an ordinary [`ScalarProgram::StrictSerialSum`] region.
///
/// **The two passes of a bare sum have different obligations, and the asymmetry
/// is structural rather than conservative.** The partial pass folds the region's
/// declared contributor domain, which lives in whichever tensor the plan placed
/// it — the first input for `sum(x)`, an intermediate for a materialized prologue
/// — so it carries [`ContributorTensor::DeclaredDomain`]. The final pass folds
/// values the partial pass *staged*, and those exist only because it staged them,
/// so its read is exactly the intermediate. Widening the final pass too would let
/// a region claim a declared input holds partials no dispatch wrote there.
///
/// **The extrema family is the other one here whose two passes read different
/// tensors.** Its partial pass reads the original scores exactly as the serial
/// extrema pass does, and its final pass folds the staged partials under the
/// *same* family — which is what makes the split a reassociation of one fold
/// rather than two reductions composed. A partial pass that read the original
/// scores and *summed* them is not thereby admitted as a sum: the sum's partial
/// pass has its own contributor domain to prove, and a mis-specified extrema
/// partial states the extrema family's split rather than that domain.
fn multi_pass_family(program: &ScalarProgram, pass: ReductionPass) -> Option<SplitFamily<'_>> {
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
            read_tensor: match pass {
                ReductionPass::Partial => ContributorTensor::DeclaredDomain,
                ReductionPass::Final => ContributorTensor::Exactly(TensorRole::Intermediate),
            },
        }),
        ScalarProgram::FusedMultiplyAddSerialSum {
            axes,
            order,
            empty_identity_bits,
            contraction,
            ..
        } => match pass {
            ReductionPass::Partial if !contraction => Some(SplitFamily {
                axes,
                order,
                empty_domain: EmptyDomainContract::Identity {
                    bits: *empty_identity_bits,
                },
                consumes_reassociation: true,
                read_tensor: ContributorTensor::Exactly(FIRST_INPUT),
            }),
            ReductionPass::Partial | ReductionPass::Final => None,
        },
        ScalarProgram::SquaredSerialSum {
            axes,
            order,
            empty_identity_bits,
            ..
        } => match pass {
            ReductionPass::Partial => Some(SplitFamily {
                axes,
                order,
                empty_domain: EmptyDomainContract::Identity {
                    bits: *empty_identity_bits,
                },
                consumes_reassociation: true,
                read_tensor: ContributorTensor::Exactly(FIRST_INPUT),
            }),
            ReductionPass::Final => None,
        },
        ScalarProgram::StrictSerialMaximum { axes, order, .. } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::NoIdentity,
            consumes_reassociation: false,
            read_tensor: ContributorTensor::Exactly(match pass {
                ReductionPass::Partial => FIRST_INPUT,
                ReductionPass::Final => TensorRole::Intermediate,
            }),
        }),
        // One answer, two different reasons. **A fold carrying an epilogue admits
        // no pass of a split**, and that refusal is the program's algebra rather
        // than caution: the epilogue applies to the *complete* fold, so a partial
        // pass applying it would transform a fragment and one that did not would
        // be an ordinary `SquaredSerialSum` wearing this variant's name — a split
        // of this family is two scalar programs rather than one partitioned,
        // which no split this vocabulary states can express. **And no pointwise
        // program folds anything**, at either width.
        ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::PointwiseF32(_)
        | ScalarProgram::PointwiseBf16(_)
        | ScalarProgram::StrictAffineU4Dequantize { .. }
        | ScalarProgram::StrictTensorContraction { .. } => None,
    }
}

/// Resolves what a cooperative tile may fold, from its scalar program.
///
/// A tile reads the original inputs and commits the reduction's own output in one
/// dispatch, so every family whose prologue belongs to the pass that reads the
/// inputs is admissible — there is no later pass here for a prologue to be
/// applied twice in. That is also why the extrema family needs no pass
/// distinction: a tile *is* both halves of the split.
///
/// It is also why the bare sum carries [`ContributorTensor::DeclaredDomain`] with
/// no pass distinction where [`multi_pass_family`] gives it one. A tile stages its
/// partials in workgroup memory, which is not a boundary tensor at all, so the
/// region's single read is the declared contributor domain and nothing here folds
/// a staged intermediate through a boundary access.
fn cooperative_family(program: &ScalarProgram) -> Option<SplitFamily<'_>> {
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
            read_tensor: ContributorTensor::DeclaredDomain,
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
            read_tensor: ContributorTensor::Exactly(FIRST_INPUT),
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
            read_tensor: ContributorTensor::Exactly(FIRST_INPUT),
        }),
        // The extrema fold reads the original scores, as its serial and partial
        // passes do, and stages one maximum per participant. Every slot it reads
        // back holds a real contributor's value rather than an identity, which is
        // what `empty_domain_is_satisfied` records the derivation for.
        ScalarProgram::StrictSerialMaximum { axes, order, .. } => Some(SplitFamily {
            axes,
            order,
            empty_domain: EmptyDomainContract::NoIdentity,
            consumes_reassociation: false,
            read_tensor: ContributorTensor::Exactly(FIRST_INPUT),
        }),
        // One answer, two different reasons, as in [`multi_pass_family`]. **A fold
        // carrying an epilogue** applies it to the complete fold, so a
        // participant's share is not a value it may be applied to, and a tile
        // that applied it once at the end would still have staged partials of a
        // program this variant does not name. **And no pointwise program folds
        // anything**, at either width.
        ScalarProgram::SquaredSerialSumThenEpilogue { .. }
        | ScalarProgram::PointwiseF32(_)
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
/// is `f32` today — `multi_pass_family` and `cooperative_family` both refuse the
/// pointwise programs — so the derivation changes no outcome now; what it
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
    let family = multi_pass_family(&region.index.scalar_program, *pass)
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
        || !family.read_tensor.admits(read.tensor)
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
    if !empty_domain_is_satisfied(family.empty_domain, contributors) {
        return Err(ScheduledRegionDiagnostic::NumericalOrAccessRefinement);
    }
    let partial_shape = partial_reduction_shape(output_shape, *partition)
        .ok_or(ScheduledRegionDiagnostic::NumericalOrAccessRefinement)?;

    let admitted = match pass {
        // The partial pass proves the split covers its own contributor
        // sequence exactly once each, and stages one partial per partition.
        //
        // Its write is the one fold write in this module the cover does not
        // choose, for the reason [`CommittedTensor::Exactly`] states: a partial
        // is an unfolded fragment of the reduction, so committing one to a
        // declared program output would publish a value that is not the fold's
        // result under any cover.
        ReductionPass::Partial => {
            partition.covers(contributors)
                && region.index.iteration_shape == partial_shape
                && CommittedTensor::Exactly(TensorRole::Intermediate).admits(write.tensor)
        }
        // The final pass proves it combines exactly one contributor per
        // partition of that same split, reading the staged partial tensor.
        //
        // It commits the *reduction's* result, so where that result goes is the
        // cover's decision exactly as it is for the serial fold this split
        // replaces. Fixing it to the program output would leave a split
        // alternative unspellable for every reduction whose result an epilogue
        // consumes — the alternative silently lost, since nothing else in the
        // pipeline would report a strategy the vocabulary cannot express.
        ReductionPass::Final => {
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
    let family = cooperative_family(&region.index.scalar_program)
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
        || !family.read_tensor.admits(read.tensor)
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
    if !empty_domain_is_satisfied(family.empty_domain, contributors) {
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
    // The split covers the sequence once across *every* round. Participant `p`
    // on round `r` folds the contiguous range at index `r * participants + p`,
    // so `partitions * contributors_per_partition * rounds` is the whole
    // sequence and the coverage stays contiguous and ascending — the split is a
    // reassociation of the declared order and never a permutation of it, exactly
    // as it is at one round. Without the round factor a tile could declare that
    // its phases run several times while its split accounts for one of them,
    // and every contributor after the first round would be folded again.
    //
    // `covers` is deliberately not extended to know about rounds: it is the
    // multi-pass split's rule, where the partitions are the whole story, and
    // teaching it a second dimension would give one method two meanings. Its
    // zero-partition refusal is reused rather than restated.
    let covered = (partition.partitions != 0)
        .then(|| partition.total_contributors())
        .flatten()
        .and_then(|total| total.checked_mul(tile.rounds));
    // The iteration domain appends one axis per *participant*, not one per
    // partition of the whole fold: the launch runs one invocation per (output,
    // participant) pair whatever the round count, because rounds are a loop
    // inside each invocation rather than more invocations.
    if covered != Some(contributors)
        || partition.partitions != participants
        || partial_reduction_shape(output_shape, *partition)
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
    let space = tile.coordinates.participants;
    // The space itself and its agreement with the launch were decided by
    // `verify_participant_space` before any proof arithmetic read the count, so
    // this propagates the product rather than re-deciding it — a second copy of
    // either rule here is one that could never say no. It is a refusal rather
    // than an `expect` for the reason the sibling site above states.
    let participant_count = space
        .participants()
        .ok_or_else(|| cooperative(CooperativeTileRule::LocalCoordinates))?;
    // The linearized run the phases, the points, and the commit are stated over.
    // They are runs rather than spaces because each is a claim about which
    // invocations reach a program point, not about the shape they are arranged
    // in.
    let participants = ParticipantRange {
        first: 0,
        count: participant_count,
    };
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
    let Some(tile) = cooperative_tile(&region.schedule.reduction) else {
        return Some(work_items);
    };
    let participants = tile.coordinates.participants.participants()?;
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
        ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
        SubnormalMode, ValueDomainProvenance,
    };
    use crate::schedule::synchronization::{
        FencedSpaces, MemoryOrdering, SynchronizationKind, SynchronizationPlacement,
        SynchronizationPoint, SynchronizationScope, SynchronizationSubject,
    };
    use crate::schedule::{PointwiseF32Expression, PointwiseF32ExpressionBuilder};
    use crate::shape::{Axis, Shape};

    /// Recorded canonical identity of the strict-`f32` pointwise test region.
    ///
    /// The pointwise program is encoded as a typed, framed topological graph,
    /// so its exact operand order, constants, root, and physical `f32` family are all pinned.
    ///
    /// Rebaselined deliberately at the `tiler.schedule.v5` step, which widened
    /// the cooperative staging relation to two dimensions: this region stages
    /// nothing, so its *payload* is untouched and only the separator moved — a
    /// claim `the_staging_relation_step_moves_only_the_domain_separator` proves
    /// rather than asserts, by comparing the two constants byte for byte past
    /// the tag.
    ///
    /// Earlier rebaselines recorded the `tiler.schedule.v4` step, which gave
    /// [`CooperativeTile`] its round count; the `v3` step, which gave
    /// `TensorRole::Input` and `PointwiseF32Node::Input` their input ordinals,
    /// so every input access and bounds proof gained four ordinal bytes and the
    /// input leaf's framed length grew from nine to twenty-one; and before that,
    /// the old `ScalarProgram::MultiplyThenAdd` tag (`0x21`) becoming the exact
    /// `ScalarProgram::PointwiseF32` expression encoding (`0x24`).
    const STRICT_F32_REGION_IDENTITY_HEX: &str = "74696c65722e7363686564756c652e7635000000000000000002000000000000000200000000000000030000000000000002010000000000010100000000000200020100000001010000000000000000000000020000000001000000000011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

    /// The same region's identity under `tiler.schedule.v4`.
    ///
    /// Retained rather than deleted, because it is what makes the `v5` step's
    /// blast radius a measured fact instead of an assurance: everything after
    /// the separator is byte-identical, so no region that stages nothing moved
    /// for any reason other than the version.
    ///
    /// **Rebaselined from the `v3` value at the `v5` step, and the rebaseline is
    /// the point rather than housekeeping.** Carried forward unchanged this
    /// constant would have made the retained comparison a `v5`-against-`v3` one
    /// — a claim about two separator steps combined, which is strictly weaker
    /// than a claim about either: a payload change at one step exactly undone at
    /// the next satisfies it. Moving it to the `v4` value keeps the comparison
    /// proving exactly one step. That discards the `v3` datum deliberately; its
    /// whole content was the `v3` to `v4` claim, which the commit that made it
    /// already carries.
    const STRICT_F32_REGION_IDENTITY_HEX_V4: &str = "74696c65722e7363686564756c652e7634000000000000000002000000000000000200000000000000030000000000000002010000000000010100000000000200020100000001010000000000000000000000020000000001000000000011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

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

    /// The reads must name non-descending declared inputs, and two dense reads
    /// of one input are an ambiguous binding rather than a program.
    ///
    /// Each perturbation leaves every other fact — access count, modes, proofs,
    /// expression — intact, so this isolates the binding rule from the arity
    /// rule below.
    ///
    /// **The ordinals need not be the dense prefix `0..n`, and the third case
    /// pins that deliberately.** The ordinal names the declared input tensor the
    /// read binds, not the access position, so a region reading inputs `0`, `1`,
    /// and `7` of a wider interface is well formed — and it has to be, because an
    /// elementwise epilogue reading a materialized intermediate alongside the
    /// program's later inputs cannot name a prefix at all.
    #[test]
    fn read_accesses_must_name_non_descending_declared_inputs() {
        let mut permuted = three_input_builder(4);
        permuted.accesses.swap(0, 1);
        permuted.bounds_proofs.swap(0, 1);
        assert_eq!(
            permuted.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        // A repeated ordinal whose two reads address identically: both are
        // `LinearIdentity`, so nothing distinguishes the leaves they serve and
        // one input is left unbound. Refused, and it is the neighbour of the
        // admitted pair below rather than a different rule.
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

        // Ascending with a gap: the third leaf binds the eighth declared input.
        let mut sparse = three_input_builder(4);
        sparse.accesses[2].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(7),
        };
        sparse.bounds_proofs[2].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(7),
        };
        assert!(sparse.build().is_ok());
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

    /// One declared input may be read twice when the two reads address it
    /// differently, and the pair has exactly one canonical spelling.
    ///
    /// This is the region behind `a * permute(a)`: two expression leaves mean
    /// two different tensors derived from one declared input, so they need two
    /// reads with two relations. Binding one access to both leaves is what made
    /// that program compile as `permute(a) * permute(a)` and return a wrong
    /// tensor, so the admission and its bound are the same rule.
    ///
    /// The three refusals are what the widening must not lose. **Reversed**: the
    /// mapped read ahead of the dense one is the same pair written the other way
    /// round, and admitting both would give one region two identities. **Two
    /// relations**: nothing ranks two structural relations against each other,
    /// so that pair has no canonical order at all. **A repeated intermediate**:
    /// the role carries no ordinal, so the attribution that makes the input pair
    /// unambiguous is exactly what it lacks.
    #[test]
    fn one_declared_input_may_be_read_densely_and_through_a_relation() {
        let control = three_input_builder(4).build().unwrap();

        let mut paired = three_input_builder(4);
        paired.accesses[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
        paired.accesses[1].map = whole_extent_reindex(4, true);
        paired.bounds_proofs[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
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
        reversed.accesses[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
        reversed.bounds_proofs[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
        assert_eq!(
            reversed.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

        let mut two_relations = three_input_builder(4);
        two_relations.accesses[0].map = whole_extent_reindex(4, false);
        two_relations.accesses[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
        two_relations.accesses[1].map = whole_extent_reindex(4, true);
        two_relations.bounds_proofs[1].tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(0),
        };
        assert_eq!(
            two_relations.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
        );

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

    /// Builds a valid `mk,nk->mn` contraction over the named program inputs.
    fn contraction_builder(left_ordinal: u32, right_ordinal: u32) -> ScheduledRegionBuilder {
        let operand = Shape::from_dims([2, 3]);
        let output = Shape::from_dims([2, 2]);
        let contracted = Shape::from_dims([3]);
        let left = TensorRole::Input {
            ordinal: InputOrdinal::new(left_ordinal),
        };
        let right = TensorRole::Input {
            ordinal: InputOrdinal::new(right_ordinal),
        };
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

    /// Contraction reads retain program ordinals and require strict ascent.
    ///
    /// Repeat and descent are perturbed independently, with their proof tensor
    /// changed beside the access so each malformed fixture fails only the
    /// canonical-ordinal rule rather than proof/reference agreement.
    #[test]
    fn contraction_input_ordinals_may_skip_but_may_not_repeat_or_descend() {
        let skipped = contraction_builder(0, 2)
            .build()
            .expect("two distinct ascending program ordinals need not be dense");
        let dense = contraction_builder(0, 1)
            .build()
            .expect("the dense control verifies");
        assert_ne!(
            skipped.canonical_identity(),
            dense.canonical_identity(),
            "the program input ordinals participate in schedule identity",
        );

        let repeated = contraction_builder(0, 0).build().unwrap_err();
        assert_eq!(
            repeated.diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a contraction read the same declared input twice",
        );

        let descending = contraction_builder(2, 0).build().unwrap_err();
        assert_eq!(
            descending.diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "a contraction encoded one input pair in descending order",
        );
    }

    /// The scale a root-mean-square normalization's producing stage computes.
    ///
    /// `Rsqrt(a / N + eps)` over the fold's value, which is input ordinal zero.
    /// The shipped instance of a fold epilogue, spelled here from the physical
    /// vocabulary rather than from any law: what this module verifies is the
    /// *schedule*, and it has no opinion on which semantic operation the chain
    /// realizes.
    fn scale_epilogue() -> PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let total = builder.input(InputOrdinal::FIRST).unwrap();
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
        let leaf = builder.input(InputOrdinal::FIRST).unwrap();
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
        let total = builder.input(InputOrdinal::FIRST).unwrap();
        let other = builder.input(InputOrdinal::new(1)).unwrap();
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
        let total = other.input(InputOrdinal::FIRST).unwrap();
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
        assert!(multi_pass_family(&scalar, ReductionPass::Partial).is_none());
        assert!(multi_pass_family(&scalar, ReductionPass::Final).is_none());
        assert!(cooperative_family(&scalar).is_none());

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
        into_extrema_split(&mut builder, vec![Axis::new(1)], FIRST_INPUT);
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
            partition,
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
                partition,
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
        builder.accesses[0].tensor = FIRST_INPUT;
        builder.bounds_proofs[0].tensor = FIRST_INPUT;
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
            partition,
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
                partition,
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
        let ReductionTopology::MultiPass { partition, .. } = verified.region().schedule.reduction
        else {
            panic!("the extrema partial fixture schedules a multi-pass split")
        };
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

    /// The second declared input tensor, which no reduction family binds.
    const SECOND_INPUT: TensorRole = TensorRole::Input {
        ordinal: InputOrdinal::new(1),
    };

    /// A bare serial sum folds a declared input or a materialized domain.
    ///
    /// **The widening, and its exact width.** `ScalarProgram::StrictSerialSum`
    /// carries no prologue, so it says how contributors combine and nothing about
    /// where they live: `sum(x)` over the first input tensor and the same fold
    /// over a prologue region's materialized result are one scalar program over
    /// two tensors, and both verify. What the widening is *not* is "any tensor" —
    /// a second declared input is refused, because a family reading one
    /// contributor domain has no ordinal for it.
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

        let mut second = serial_reduction_builder(bare_sum(vec![Axis::new(1)]));
        read_from(&mut second, SECOND_INPUT);
        assert_eq!(
            second.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
            "one contributor domain has no second input ordinal to bind",
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
            write_to(&mut into_input, FIRST_INPUT);
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
        write_to(&mut into_input, FIRST_INPUT);
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
        read_from(&mut partial, FIRST_INPUT);
        assert!(
            partial.build().is_ok(),
            "a prologue-less fold's partial pass reads the input the fold folds",
        );

        let mut combine = final_pass_builder(SPLIT);
        read_from(&mut combine, FIRST_INPUT);
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
        read_from(&mut input, FIRST_INPUT);
        assert!(input.build().is_ok());

        let mut second = cooperative_builder(cooperative_tile_fixture());
        read_from(&mut second, SECOND_INPUT);
        assert_eq!(
            second.build().unwrap_err().diagnostics(),
            [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
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
        write_to(&mut into_input, FIRST_INPUT);
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
                ScheduledRegionDiagnostic::NumericalOrAccessRefinement,
            ),
            // The same partition count, three covered where the access supplies
            // six.
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
            6,
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
    /// which is a different computation and not the declared reduction.
    #[test]
    fn a_split_that_ignores_the_round_count_is_refused() {
        let mut tile = cooperative_tile_fixture();
        tile.rounds = 2;
        assert_eq!(
            cooperative_rejection(cooperative_builder_with(tile, SPLIT)),
            ScheduledRegionDiagnostic::CooperativeTile {
                rule: CooperativeTileRule::ContributorSplit,
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

    /// A region that stages nothing moved for the version and nothing else.
    ///
    /// The blast-radius proof for `tiler.schedule.v5`. The participant space and
    /// the per-dimension stride vector both land inside the `0x35` cooperative
    /// payload, which a region with no tile never reaches, so the only bytes
    /// that may differ from `v4` are the eighteen of the domain separator — and
    /// that is checked here by comparing the two recorded identities past it
    /// rather than by asserting it in prose.
    ///
    /// The comparison is against the immediately preceding domain and not an
    /// older one, which is what keeps it a one-step claim: two separator changes
    /// agreeing past the tag say nothing about whether the payload moved at
    /// either step individually.
    ///
    /// It is deliberately *not* an append proof. The step replaces an unframed
    /// fixed-width run with a length-framed one, so there is no position to
    /// append to; [`encode_identity`] records that, and records separately why
    /// the `v4` step's own append was unavailable.
    #[test]
    fn the_staging_relation_step_moves_only_the_domain_separator() {
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
            STRICT_F32_REGION_IDENTITY_HEX_V4[..SEPARATOR]
        );
        assert_eq!(
            STRICT_F32_REGION_IDENTITY_HEX[SEPARATOR..],
            STRICT_F32_REGION_IDENTITY_HEX_V4[SEPARATOR..]
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
