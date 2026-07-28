use std::error::Error;
use std::fmt;

use tiler_ir::shape::Shape;

// The target-neutral scheduled-region IR and the backend-consumable structured
// kernel IR, with their intrinsic verifiers and canonical identities, live in
// `tiler_ir::schedule` and `tiler_ir::kernel` (ADR 0070). This module owns only
// the compiler-specific refinements layered on top of a verified region:
// semantic-occurrence binding, request-subject binding, and target feasibility.
// The shared vocabulary is re-exported so existing `crate::physical::*`
// importers continue to resolve.
pub(crate) use tiler_ir::kernel::VerifiedKernel;
pub(crate) use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExecutionBinding, IndexRegion, KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId,
    ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy, TensorRole,
};
use tiler_ir::schedule::{
    ArithmeticType, ScheduledRegionBuildError, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
};

use crate::feasibility::{
    AvailabilityPhase, AxisRequirement, CapabilityAxis, CapabilityFact, CheckedTargetProfile,
    FactAuthority, FactProvenance, FactValidityScope, FeasibilityError, FeasibilityOutcome,
    FeasibilityProposal, ProvenEvidence, RejectionCause, TargetProfileIdentity,
};
use crate::honourability::{
    DimensionBehaviour, NumericalDimension, NumericalRequirement, UnhonouredDimension,
};
use crate::region::SemanticMemberId;
use crate::request::{
    NumericalPermission, PrototypeTargetProfile, StrictF32NumericalContract,
    VerifiedRequestSubject, VerifiedTargetRequest,
};

/// Stable candidate identity used when assessing one scheduled region.
const REGION_PROPOSAL_CANDIDATE: &str = "tiler.prototype.scheduled-region";

/// Stable candidate identity used when resolving a numerical contract alone.
const CONTRACT_PROPOSAL_CANDIDATE: &str = "tiler.prototype.numerical-contract";

/// A verified scheduled region bound to one compilation request.
///
/// This wraps the target-neutral [`tiler_ir::schedule::VerifiedScheduledRegion`]
/// with the compiler-owned refinements the shared IR deliberately excludes: the
/// exact semantic occurrences the region covers, the target profile it was
/// assessed against, and the request subject it belongs to. The inner region is
/// intrinsically verified before any of these bindings are formed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedScheduledRegion {
    verified: tiler_ir::schedule::VerifiedScheduledRegion,
    semantic_members: Vec<SemanticMemberId>,
    target_profile_key: &'static str,
    request_subject: VerifiedRequestSubject,
}

impl VerifiedScheduledRegion {
    pub(crate) fn region(&self) -> &ScheduledRegion {
        self.verified.region()
    }
    /// Returns the shared-IR verified region this compiler binding wraps.
    ///
    /// Structured-kernel lowering consumes the shared verified value directly,
    /// so a kernel can only ever refine an intrinsically verified schedule.
    pub(crate) const fn verified(&self) -> &tiler_ir::schedule::VerifiedScheduledRegion {
        &self.verified
    }
    pub(crate) fn requirements(&self) -> ResourceRequirements {
        self.verified.requirements()
    }
    /// Returns the canonical, transient-ordinal-independent identity of the inner
    /// verified region.
    ///
    /// This is the shared-IR identity (ADR 0070) derived purely from the
    /// normalized schedule content, so equivalent regions proposed by different
    /// physical providers share it. The implementation frontier folds it into a
    /// per-proposal identity that additionally distinguishes provider provenance.
    pub(crate) fn canonical_identity(
        &self,
    ) -> &tiler_ir::schedule::CanonicalScheduledRegionIdentity {
        self.verified.canonical_identity()
    }
    pub(crate) const fn target_profile_key(&self) -> &'static str {
        self.target_profile_key
    }
    /// Returns the exact semantic occurrences this region covers.
    ///
    /// These are graph-local operation ordinals of the verified program, not a
    /// fixed role vocabulary, so a schedule cannot claim coverage of operations
    /// the request boundary did not actually recognize.
    pub(crate) fn semantic_members(&self) -> &[SemanticMemberId] {
        &self.semantic_members
    }
    pub(crate) fn matches_request(&self, request: &VerifiedTargetRequest) -> bool {
        self.request_subject == *request.subject()
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
    /// A numerical dimension the target declares it cannot honour as required.
    ///
    /// A distinct variant rather than a `Target` with two numbers, because the
    /// rejection ADR 0076 item 5 requires names a dimension, a required
    /// behaviour, a declared means, the behaviour the target does honour, and
    /// the declaring profile — none of which is a quantity, and all of which the
    /// retired `strict-f32: required 1, available 0` shape discarded.
    Numerical {
        region: RegionId,
        cause: UnhonouredDimension,
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
                    region.get()
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
                region.get()
            ),
            Self::Numerical { region, cause } => {
                write!(
                    formatter,
                    "schedule.numerics.{}: region {} requires {}, target declares {}",
                    cause.dimension().key(),
                    region.get(),
                    cause.required().key(),
                    cause.means().key(),
                )?;
                if let Some(honoured) = cause.honoured() {
                    write!(formatter, " and honours {}", honoured.key())?;
                }
                write!(formatter, " (profile {})", cause.profile().key())
            }
            Self::Refinement { rule, region } => write!(
                formatter,
                "kernel.refinement.{rule}: kernel for region {} rejected",
                region.get()
            ),
            Self::ShapeProductOverflow { region } => write!(
                formatter,
                "schedule.shape.element-count: region {} exceeds u64",
                region.get()
            ),
        }
    }
}

impl Error for PhysicalError {}

#[allow(
    dead_code,
    reason = "canonical region constructor the governed physical provider proposes through the frontier; retained as the single definition of each recognized region and exercised by its own tests"
)]
pub(crate) fn build_scheduled_regions(
    request: &VerifiedTargetRequest,
) -> Result<Vec<VerifiedScheduledRegion>, PhysicalError> {
    let (pointwise, pointwise_members) = pointwise_region(request);
    let (reduction, reduction_members) = reduction_region(request);
    Ok(vec![
        verify_schedule(pointwise, pointwise_members, request)?,
        verify_schedule(reduction, reduction_members, request)?,
    ])
}

#[allow(
    dead_code,
    reason = "canonical region constructor the governed physical provider proposes through the frontier; retained as the single definition of each recognized region and exercised by its own tests"
)]
pub(crate) fn build_fused_scheduled_region(
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let (fused, members) = fused_region(request);
    verify_schedule(fused, members, request)
}

/// Builds the canonical materialized pointwise scheduled region for one request.
///
/// This constructs the raw, not-yet-verified region and its recognized pointwise
/// members; it applies no intrinsic, subject-binding, or feasibility gate. The
/// implementation frontier and its providers use it to obtain a canonical region
/// they then re-submit through the ordinary checked verification path, including
/// for a domain the governed profile cannot dispatch.
pub(crate) fn pointwise_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: request.serial_sum().input_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Input,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Intermediate,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Input,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().input_elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Intermediate,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().input_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
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
            numerical: request.numerical_contract().realization(),
        },
        schedule: linear_schedule(
            request.serial_sum().input_elements,
            OwnershipWitnessId::new(0),
        ),
    };
    (region, request.serial_sum().members.pointwise().to_vec())
}

/// Builds the canonical materialized reduction scheduled region for one request.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn reduction_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(1),
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
                    bounds: BoundsWitnessId::new(2),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(3),
                    ownership: Some(OwnershipWitnessId::new(1)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(2),
                    tensor: TensorRole::Intermediate,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(3),
                    tensor: TensorRole::Output,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(1),
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
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: false,
            },
            ..linear_schedule(
                request.serial_sum().output_elements,
                OwnershipWitnessId::new(1),
            )
        },
    };
    (region, request.serial_sum().members.reduction().to_vec())
}

/// Builds the canonical fused whole-program scheduled region for one request.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn fused_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
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
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(1),
                    ownership: Some(OwnershipWitnessId::new(0)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(0),
                    tensor: TensorRole::Input,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: request.serial_sum().input_shape.clone(),
                        output_shape: request.serial_sum().output_shape.clone(),
                        axes: request.serial_sum().reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: TensorRole::Output,
                    kind: BoundsProofKind::LinearRange {
                        element_count: request.serial_sum().output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
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
                // Derived from the contract, exactly as the unfused pointwise and
                // reduction regions derive theirs. Hard-coding `false` here was
                // invisible while every registered contract forbade both
                // freedoms, and would have made this candidate fail the schedule
                // verifier's realization cross-check under one that permits them
                // — losing the fused plan silently rather than wrongly, but
                // losing it for a reason no diagnostic would have named.
                contraction: request.numerical_contract().contraction
                    != NumericalPermission::Forbidden,
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: request.serial_sum().reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                // Permutation stays refused: no contract this build registers
                // permits it, and `crate::policy::unrepresentable_dimension`
                // refuses one that tries, because no scheduled region can record
                // which resolution was chosen.
                permits_permutation: false,
            },
            ..linear_schedule(
                request.serial_sum().output_elements,
                OwnershipWitnessId::new(0),
            )
        },
    };
    (region, request.serial_sum().members.all())
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

/// Verifies one scheduled region and binds it to a compilation request.
///
/// Intrinsic schedule verification runs first, in `tiler_ir::schedule`, and
/// proves domain coverage, ownership, race freedom, tail/launch legality,
/// bounds-proof refinement, reduction contributor/order legality, and
/// zero-domain behaviour before any feasibility query. Only then does the
/// compiler layer its request-subject binding and the single hard-feasibility
/// decision. No cost or provider callback participates.
#[allow(
    dead_code,
    reason = "the predicate-free spelling of the checked verification path; the frontier consumes the predicate-carrying form"
)]
pub(crate) fn verify_schedule(
    region: ScheduledRegion,
    semantic_members: Vec<SemanticMemberId>,
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    verify_schedule_with_feasibility(region, semantic_members, request)
        .map(|(verified, _)| verified)
}

/// Verifies one scheduled region and additionally surfaces the resolved
/// feasibility predicates that a proven target assessment carries.
///
/// This runs the exact checked path [`verify_schedule`] runs — the request-subject
/// precondition, whole-region intrinsic verification, numerical-realization
/// agreement, the request-subject binding, and the single hard-feasibility
/// decision — and additionally returns the evidence of a
/// [`FeasibilityOutcome::Proven`](crate::feasibility::FeasibilityOutcome) verdict.
/// The physical implementation frontier retains it as admission evidence for an
/// enumerated proposal. A provider cannot bypass any of these checks: a
/// [`PhysicalError::Target`] or [`PhysicalError::Numerical`] means the proposal
/// is hard-infeasible (never a cost), and any other [`PhysicalError`] means the
/// provider emitted invalid IR.
pub(crate) fn verify_schedule_with_feasibility(
    region: ScheduledRegion,
    semantic_members: Vec<SemanticMemberId>,
    request: &VerifiedTargetRequest,
) -> Result<(VerifiedScheduledRegion, ProvenEvidence), PhysicalError> {
    let id = region.index.id;
    let subject = request.subject();
    if !request.reconstructs_its_authority()
        || request.target_profile() != PrototypeTargetProfile::governed()
        || !request.numerical_contract().is_governed()
    {
        return intrinsic("request-subject", id);
    }
    let verified = ScheduledRegionBuilder::from_region(region)
        .build()
        .map_err(|error| map_schedule_build_error(&error, id))?;
    if verified.region().index.numerical != request.numerical_contract().realization() {
        return intrinsic("numerical-realization", id);
    }
    verify_region_subject_binding(verified.region(), &semantic_members, subject)?;
    let evidence = assess_region(
        id,
        verified.requirements(),
        // The region implements this request's resolved contract — checked one
        // line above by comparing the region's realization against it — so its
        // arithmetic type is the contract's, not a value re-derived here.
        request.numerical_contract().arithmetic,
        verified.region().schedule.work_items,
        &request.target_profile(),
    )?;
    Ok((
        VerifiedScheduledRegion {
            verified,
            semantic_members,
            target_profile_key: request.target_profile().key,
            request_subject: subject.clone(),
        },
        evidence,
    ))
}

/// Maps an intrinsic schedule-verification failure onto the physical-error
/// contract.
///
/// A domain-product overflow keeps its distinct shape-overflow class; every
/// other intrinsic diagnostic carries its stable rule identifier so the explain
/// trace attributes the exact rejected rule.
fn map_schedule_build_error(error: &ScheduledRegionBuildError, region: RegionId) -> PhysicalError {
    match error.diagnostics().first() {
        Some(ScheduledRegionDiagnostic::ShapeProductOverflow) => {
            PhysicalError::ShapeProductOverflow { region }
        }
        Some(diagnostic) => PhysicalError::Intrinsic {
            rule: diagnostic.rule(),
            region,
        },
        None => PhysicalError::Intrinsic {
            rule: "schedule-verification",
            region,
        },
    }
}

fn verify_region_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticMemberId],
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let normalized = subject.normalized();
    if !tiler_ir::schedule::axes_are_canonical(
        normalized.reduction_axes(),
        normalized.input_shape().rank(),
    ) || element_count(normalized.input_shape(), region.index.id)? != normalized.input_elements()
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
            semantic_members == normalized.members().pointwise()
                && region.index.id == RegionId::new(0)
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
            semantic_members == normalized.members().reduction()
                && region.index.id == RegionId::new(1)
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
            semantic_members == normalized.members().all()
                && region.index.id == RegionId::new(0)
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

/// Assesses one scheduled region against the typed feasibility authority.
///
/// This is the single hard-feasibility decision for the bounded serial-Sum path.
/// It builds an immutable checked target profile and a typed candidate proposal,
/// then maps the four-outcome result onto the existing physical-error contract:
/// a proven candidate yields its evidence (consumed by the explain admitted
/// trace); a rejected candidate yields the canonical representative cause, as a
/// [`PhysicalError::Numerical`] when the target declares it cannot honour a
/// dimension and as a [`PhysicalError::Target`] when a capability bound is
/// exceeded. The governed baseline declares only compile-profile-resolvable
/// predicates, so a deferred or unknown verdict — like a malformed profile or
/// proposal — signals that the checked contract drifted from the prototype
/// limits and fails closed as an intrinsic error rather than admitting an
/// unproven plan. Cost never enters this decision.
/// Why a resource assessment did not prove feasibility, **unattributed**.
///
/// The verdict without the blame. `assess_region` attributes it to a
/// `RegionId`; an opaque physical call attributes the same verdict to the call
/// that proposed it. One feasibility decision, two attributions — which is what
/// ADR 0043's single decision requires, since the *verdict* is what must be
/// shared and only the subject differs.
///
/// Carrying a `RegionId` in here instead would force any caller that is not a
/// region to invent one, and a feasibility rejection attributed to a region that
/// does not exist is worse than no attribution at all: a reader chasing it finds
/// nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the unattributed verdict; its second caller is the opaque-call admission being built"
)]
pub(crate) enum ResourceVerdict {
    /// The target profile or the proposal itself was malformed.
    Intrinsic(FeasibilityError),
    /// The target refused the proposal, with the representative cause.
    Rejected(RejectionCause),
    /// The assessment neither proved nor refused.
    Unresolved,
}

/// Assesses exact resource requirements against a target, attributing nothing.
///
/// The shared half of the feasibility decision. Every caller runs this; each
/// then maps a [`ResourceVerdict`] onto its own error vocabulary.
#[allow(
    dead_code,
    reason = "the shared feasibility core; the opaque-call admission is its second caller"
)]
pub(crate) fn assess_resources(
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
    target: &PrototypeTargetProfile,
) -> Result<ProvenEvidence, ResourceVerdict> {
    let profile = checked_target_profile(target).map_err(ResourceVerdict::Intrinsic)?;
    let proposal = region_proposal(requirements, arithmetic, work_items)
        .map_err(ResourceVerdict::Intrinsic)?;
    match profile.assess(&proposal, AvailabilityPhase::CompileProfile) {
        FeasibilityOutcome::Proven(evidence) => Ok(evidence),
        FeasibilityOutcome::Rejected(rejection) => {
            Err(ResourceVerdict::Rejected(rejection.representative()))
        }
        FeasibilityOutcome::Deferred(_) | FeasibilityOutcome::Unknown(_) => {
            Err(ResourceVerdict::Unresolved)
        }
    }
}

/// Assesses one region's resources, attributing any verdict to that region.
pub(crate) fn assess_region(
    region: RegionId,
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
    target: &PrototypeTargetProfile,
) -> Result<ProvenEvidence, PhysicalError> {
    assess_resources(requirements, arithmetic, work_items, target).map_err(
        |verdict| match verdict {
            ResourceVerdict::Intrinsic(error) => feasibility_intrinsic(error, region),
            ResourceVerdict::Rejected(RejectionCause::Numerical(cause)) => {
                PhysicalError::Numerical { region, cause }
            }
            ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => {
                PhysicalError::Target {
                    rule: predicate.axis().key(),
                    region,
                    required: predicate.required().value(),
                    available: predicate.available().value(),
                }
            }
            ResourceVerdict::Unresolved => PhysicalError::Intrinsic {
                rule: "target-assessment-unresolved",
                region,
            },
        },
    )
}

/// Assesses one numerical contract alone against a target's declaration.
///
/// The request boundary resolves a caller's stated preference through this: the
/// proposal carries the contract's four dimensions and *no* capability
/// requirement, because whether a target honours a contract is a fact about the
/// contract and the target, independent of any region, schedule, or cost. A
/// region is assessed again later against the same authority, which is
/// defence in depth rather than a second decision.
pub(crate) fn assess_contract(
    target: &PrototypeTargetProfile,
    contract: StrictF32NumericalContract,
) -> Result<FeasibilityOutcome, FeasibilityError> {
    let profile = checked_target_profile(target)?;
    let proposal = FeasibilityProposal::new(
        CONTRACT_PROPOSAL_CANDIDATE,
        Vec::new(),
        contract.dimension_requirements(),
    )?;
    Ok(profile.assess(&proposal, AvailabilityPhase::CompileProfile))
}

/// Returns the canonical descriptor bytes of one target profile.
///
/// Derived from the same [`checked_target_profile`] the feasibility assessment
/// uses, so the descriptor an artifact records is the profile a variant was
/// actually assessed against rather than one recomputed from a key that
/// happens to match.
///
/// This is only half of what ADR 0043 requires an artifact to record. The other
/// half — which feasibility rules compared the candidate against these facts —
/// is [`crate::feasibility::GOVERNED_FEASIBILITY_RULE_SET`], and it is not
/// derived per target because the rules do not vary by target.
pub(crate) fn target_profile_descriptor(
    target: &PrototypeTargetProfile,
) -> Result<Vec<u8>, FeasibilityError> {
    let descriptor = checked_target_profile(target)?.canonical_descriptor();
    // Refused here rather than at packaging. `tiler-artifact` bounds what it
    // will hold, but it can only report a length; this crate can name the
    // profile that declared too much, which is the difference between a
    // diagnostic someone can act on and one they have to trace back.
    if descriptor.len() > crate::feasibility::MAX_TARGET_PROFILE_DESCRIPTOR_BYTES {
        return Err(FeasibilityError::DescriptorTooLong {
            key: target.key,
            actual: descriptor.len(),
        });
    }
    Ok(descriptor)
}

/// Builds the immutable checked profile for the prototype baseline target.
///
/// The prototype profile has no explicitly stageable local memory or barriers,
/// so those axes carry a conservative compile-time ceiling of zero. Every axis is
/// a compile-profile guarantee, keeping the bounded serial-Sum candidate provable
/// without any later-phase query. The target's numerical honourability
/// declaration is lifted here too, each line attributed to the declaring profile
/// exactly as a capability bound is.
fn checked_target_profile(
    target: &PrototypeTargetProfile,
) -> Result<CheckedTargetProfile, FeasibilityError> {
    let identity = TargetProfileIdentity::new(target.key);
    let fact = |axis: CapabilityAxis, bound: u64| {
        CapabilityFact::new(
            axis,
            bound,
            AvailabilityPhase::CompileProfile,
            FactAuthority::GovernedProfile,
            FactValidityScope::PortableProfile,
            FactProvenance::declared_by(identity),
        )
    };
    CheckedTargetProfile::new(
        identity,
        vec![
            fact(
                CapabilityAxis::GridAxisThreads,
                target.max_threads_per_grid_axis,
            ),
            fact(
                CapabilityAxis::WorkgroupThreads,
                u64::from(target.max_threads_per_workgroup),
            ),
            fact(
                CapabilityAxis::BufferBindings,
                u64::from(target.max_buffer_bindings_per_entry),
            ),
            fact(CapabilityAxis::IndexWidthBits, u64::from(target.index_bits)),
            fact(
                CapabilityAxis::DeviceAddressSpace,
                u64::from(target.supports_device_memory),
            ),
            fact(CapabilityAxis::LocalMemoryBytes, 0),
            fact(CapabilityAxis::Barriers, 0),
        ],
        target
            .numerical
            .iter()
            .map(|declared| declared.attributed_to(identity))
            .collect(),
    )
}

/// Builds the typed candidate proposal for one scheduled region.
///
/// The candidate requires 64-bit indexing and the device address space whenever
/// its resource requirements demand it; the prototype baseline needs no local
/// memory or barriers. Its numerical requirements are the region's declared
/// realization carried forward **per dimension** rather than collapsed into one
/// summary bit — the collapse the retired `StrictF32Arithmetic` axis forced, and
/// which could neither name a failing dimension nor express emulation.
fn region_proposal(
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
) -> Result<FeasibilityProposal, FeasibilityError> {
    FeasibilityProposal::new(
        REGION_PROPOSAL_CANDIDATE,
        vec![
            AxisRequirement::new(CapabilityAxis::GridAxisThreads, work_items),
            AxisRequirement::new(
                CapabilityAxis::WorkgroupThreads,
                u64::from(requirements.threads_per_workgroup),
            ),
            AxisRequirement::new(
                CapabilityAxis::BufferBindings,
                u64::from(requirements.buffer_bindings),
            ),
            AxisRequirement::new(CapabilityAxis::IndexWidthBits, 64),
            AxisRequirement::new(
                CapabilityAxis::DeviceAddressSpace,
                u64::from(requirements.requires_device_memory),
            ),
            AxisRequirement::new(
                CapabilityAxis::LocalMemoryBytes,
                requirements.local_memory_bytes,
            ),
            AxisRequirement::new(CapabilityAxis::Barriers, u64::from(requirements.barriers)),
        ],
        vec![
            NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                arithmetic,
                DimensionBehaviour::Subnormals(requirements.input_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::ResultSubnormals,
                arithmetic,
                DimensionBehaviour::Subnormals(requirements.result_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::Contraction,
                arithmetic,
                DimensionBehaviour::Transform(requirements.contraction),
            ),
            NumericalRequirement::new(
                NumericalDimension::Reassociation,
                arithmetic,
                DimensionBehaviour::Transform(requirements.reassociation),
            ),
        ],
    )
}

/// Maps a feasibility intrinsic error onto the physical-error contract.
///
/// A malformed profile or proposal is a contract violation, not a feasibility
/// outcome, so it fails closed as an intrinsic scheduling error.
const fn feasibility_intrinsic(error: FeasibilityError, region: RegionId) -> PhysicalError {
    let rule = match error {
        FeasibilityError::MalformedProfile { .. } => "target-profile-malformed",
        FeasibilityError::MalformedProposal { .. } => "target-proposal-malformed",
        // A profile too large to describe is a declaration defect in the same
        // class as a malformed one: it is a fact about the profile, decided
        // before any candidate is considered, and no other plan makes it
        // describable.
        FeasibilityError::DescriptorTooLong { .. } => "target-profile-descriptor-too-long",
    };
    PhysicalError::Intrinsic { rule, region }
}

/// Lowers one verified scheduled region to its verified structured kernel.
///
/// The structured kernel IR, its canonical lowering, and its verifier live in
/// [`tiler_ir::kernel`] (ADR 0070). This compiler entry point only forwards an
/// already request-bound verified region and re-attributes a lowering failure
/// to the region for the explain trace: a rejected lowering is a compiler
/// output defect, never a feasibility outcome.
pub(crate) fn lower_structured_kernel(
    scheduled: &VerifiedScheduledRegion,
) -> Result<VerifiedKernel, PhysicalError> {
    tiler_ir::kernel::lower_scheduled_region(scheduled.verified()).map_err(|error| {
        PhysicalError::Refinement {
            rule: error.rule(),
            region: scheduled.region().index.id,
        }
    })
}

/// Counts the elements of a shape, attributing any overflow to the region.
fn element_count(shape: &Shape, region: RegionId) -> Result<u64, PhysicalError> {
    tiler_ir::schedule::element_count(shape)
        .map_err(|_| PhysicalError::ShapeProductOverflow { region })
}

fn intrinsic<T>(rule: &'static str, region: RegionId) -> Result<T, PhysicalError> {
    Err(PhysicalError::Intrinsic { rule, region })
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    /// The governed profile's canonical descriptor, pinned byte for byte.
    ///
    /// **This is a refactor guard, not a golden for its own sake.** The
    /// descriptor is encoded into `VerifiedRequestSubject`'s canonical explain
    /// subject and carried out through `Compilation::target_profile_descriptor`
    /// into the artifact's `TargetProfileDescriptorDigest`, so one changed byte
    /// moves every artifact identity and invalidates every cache entry. The
    /// producer's two-process determinism test and the serial-sum artifact
    /// identity would both catch it, but only after a whole compile-and-package
    /// cycle and without saying which field moved.
    ///
    /// `admit-a-caller-declared-target-profile` has to turn this type from a
    /// `Copy` struct of `&'static` fields into an owned one, touching roughly
    /// thirty sites. This exists so that refactor fails here, immediately and
    /// with a diff, rather than downstream.
    ///
    /// Regenerate only when the encoding is *deliberately* changed: print
    /// `target_profile_descriptor(&PrototypeTargetProfile::governed())` as hex
    /// and step whatever domain tag the change requires in the same commit.
    #[test]
    fn the_governed_descriptor_bytes_do_not_move() {
        // Rebaselined on the numerical-policies merge and recomputed on the
        // merged tree (never taken from a branch): the contract widened from
        // four numerical dimensions to the docs/numerical-semantics.md set,
        // each keyed by the arithmetic type it resolves, so the descriptor's
        // numerical rows grew from 8 to 12 entries and gained the type key.
        // Every artifact identity and cache entry derived from the descriptor
        // moves with it, which is this test's message and the reason the move
        // is stated here rather than silent.
        const GOVERNED: &str = concat!(
            "000000000000002374696c65722e7461726765742d70726f66696c652e646573",
            "63726970746f722e763300000000000000002a74696c65722e70726f746f7479",
            "70652d7461726765742d6e65757472616c2d626173656c696e652e7631000000",
            "000000000701000000000000ffff010101020000000000000001010101030000",
            "0000000000020101010400000000000000400101010500000000000000010101",
            "0107000000000000000001010108000000000000000001010100000000000000",
            "0c01030101010101010103010201010101020301010101010102030102010101",
            "0103030201010101010303020201010101040302010101010104030202010101",
            "010503020101010101060302010101010109030401010101010a030401010101",
            "01",
        );

        let descriptor =
            target_profile_descriptor(&crate::request::PrototypeTargetProfile::governed())
                .expect("the governed profile describes");
        let mut actual = String::with_capacity(descriptor.len() * 2);
        for byte in &descriptor {
            write!(actual, "{byte:02x}").expect("writing to a String cannot fail");
        }
        assert_eq!(
            actual, GOVERNED,
            "the governed target profile's canonical descriptor moved; every artifact \
             identity and cache entry derived from it moves with it",
        );
    }
    use super::*;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::kernel::{KernelConstant, OperationRef, OperationView};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::Axis;

    /// Returns the bounded loop range of the kernel's guarded region, if any.
    fn loop_bounds(kernel: &VerifiedKernel) -> Option<(u64, u64)> {
        guarded_operations(kernel).find_map(|view| match view {
            OperationView::SerialLoop(reduction) => Some((reduction.start(), reduction.end())),
            _ => None,
        })
    }

    /// Returns the constant the kernel commits, when it stores an immediate.
    fn stored_constant(kernel: &VerifiedKernel) -> Option<KernelConstant> {
        guarded_operations(kernel).find_map(|view| match view {
            OperationView::Store { value, .. } => kernel.value_constant(value).ok().flatten(),
            _ => None,
        })
    }

    fn guarded_operations(kernel: &VerifiedKernel) -> impl Iterator<Item = OperationView<'_>> {
        kernel
            .body()
            .operations()
            .filter_map(|operation| match operation.view() {
                OperationView::Predicated { body, .. } => Some(body),
                _ => None,
            })
            .flat_map(|body| body.operations().map(OperationRef::view))
    }

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

        assert_eq!(regions[0].region().schedule.work_items, 6);
        assert_eq!(regions[1].region().schedule.work_items, 2);
        // Each kernel retains the exact identity of the region it refines.
        assert_eq!(pointwise.scheduled_region(), RegionId::new(0));
        assert_eq!(reduction.scheduled_region(), RegionId::new(1));
        assert_eq!(
            pointwise.scheduled_region_identity(),
            regions[0].canonical_identity()
        );
        assert_eq!(
            reduction.scheduled_region_identity(),
            regions[1].canonical_identity()
        );
        // The reduction realizes the scheduled contributor order as an explicit
        // bounded loop; the pointwise region carries none.
        assert_eq!(loop_bounds(&reduction), Some((1, 3)));
        assert_eq!(loop_bounds(&pointwise), None);
    }

    #[test]
    fn scheduled_regions_carry_a_transient_independent_identity() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        // Equivalent normalized regions built from a fresh request share bytes.
        let rebuilt = build_scheduled_regions(&request).unwrap();
        for (first, second) in regions.iter().zip(&rebuilt) {
            assert_eq!(
                first.verified.canonical_identity().as_bytes(),
                second.verified.canonical_identity().as_bytes()
            );
        }
        // The two distinct regions of one program have distinct identities.
        assert_ne!(
            regions[0].verified.canonical_identity().as_bytes(),
            regions[1].verified.canonical_identity().as_bytes()
        );
    }

    #[test]
    fn empty_reduction_lowers_to_explicit_positive_zero_stores() {
        let request = request(Shape::from_dims([2, 0]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();
        // An empty reduction commits the proved identity directly: no loop and
        // no contributor load remain for a backend to interpret.
        assert_eq!(loop_bounds(&reduction), None);
        assert_eq!(
            stored_constant(&reduction),
            Some(KernelConstant::F32Bits(0.0_f32.to_bits()))
        );
    }

    #[test]
    fn schedule_and_kernel_fail_closed_on_refinement_mismatches() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();

        let mut invalid_schedule = regions[1].region().clone();
        invalid_schedule.schedule.reduction = ReductionTopology::None;
        assert_eq!(
            verify_schedule(
                invalid_schedule,
                regions[1].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId::new(1),
            })
        );

        let mut invalid_access = regions[0].region().clone();
        invalid_access.index.accesses[0].bounds = BoundsWitnessId::new(9);
        assert_eq!(
            verify_schedule(
                invalid_access,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "proof-reference",
                region: RegionId::new(0),
            })
        );

        let mut invalid_proof = regions[0].region().clone();
        invalid_proof.index.bounds_proofs[0].kind =
            BoundsProofKind::LinearRange { element_count: 5 };
        assert_eq!(
            verify_schedule(
                invalid_proof,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "bounds-proof",
                region: RegionId::new(0),
            })
        );

        let mut invalid_numerics = regions[0].region().clone();
        invalid_numerics
            .index
            .numerical
            .canonical_arithmetic_nan_bits ^= 1;
        assert_eq!(
            verify_schedule(
                invalid_numerics,
                regions[0].semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-realization",
                region: RegionId::new(0),
            })
        );
    }

    #[test]
    fn reduction_access_and_proof_shapes_are_bound_to_the_verified_request() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let fused = build_fused_scheduled_region(&request).unwrap();

        for (mut forged, members) in [
            (
                regions[1].region().clone(),
                regions[1].semantic_members().to_vec(),
            ),
            (fused.region().clone(), fused.semantic_members().to_vec()),
        ] {
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
                verify_schedule(forged, members, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    region,
                })
            );
        }
    }

    #[test]
    fn fused_schedule_rejects_numerical_corruption() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();
        let mut invalid_schedule = scheduled.region().clone();
        let ScalarProgram::FusedMultiplyAddSerialSum { contraction, .. } =
            &mut invalid_schedule.index.scalar_program
        else {
            panic!("expected fused scalar program")
        };
        *contraction = true;
        assert_eq!(
            verify_schedule(
                invalid_schedule,
                scheduled.semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "numerical-or-access-refinement",
                region: RegionId::new(0),
            })
        );
    }

    #[test]
    fn malformed_axes_zero_launch_and_late_zero_products_fail_without_panicking() {
        let request = request(Shape::from_dims([2, 3]), [Axis::new(1)]);
        let scheduled = build_fused_scheduled_region(&request).unwrap();

        let mut zero_threads = scheduled.region().clone();
        zero_threads.schedule.threads_per_workgroup = 0;
        zero_threads.schedule.launch.threads_per_workgroup = 0;
        assert!(matches!(
            verify_schedule(
                zero_threads,
                scheduled.semantic_members().to_vec(),
                &request
            ),
            Err(PhysicalError::Intrinsic {
                rule: "launch-coverage",
                ..
            })
        ));

        for axes in [vec![Axis::new(1), Axis::new(1)], vec![Axis::new(99)]] {
            let mut malformed = scheduled.region().clone();
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
                verify_schedule(malformed, scheduled.semantic_members().to_vec(), &request),
                Err(PhysicalError::Intrinsic { .. })
            ));
        }
    }
}
