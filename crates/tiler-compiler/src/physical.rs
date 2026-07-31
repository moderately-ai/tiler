use std::error::Error;
use std::fmt;

use tiler_ir::semantic::F32;
use tiler_ir::shape::Shape;

// The target-neutral scheduled-region IR and the backend-consumable structured
// kernel IR, with their intrinsic verifiers and canonical identities, live in
// `tiler_ir::schedule` and `tiler_ir::kernel` (ADR 0070). This module owns only
// the compiler-specific refinements layered on top of a verified region:
// semantic-occurrence binding, request-subject binding, and target feasibility.
// The shared vocabulary is re-exported so existing `crate::physical::*`
// importers continue to resolve.
use tiler_ir::kernel::KernelType;
pub(crate) use tiler_ir::kernel::VerifiedKernel;
pub(crate) use tiler_ir::schedule::{
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContributorOrder,
    ExecutionBinding, IndexRegion, KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, PointwiseF32Node, ReductionTopology, RegionId,
    ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy, TensorRole,
};
use tiler_ir::schedule::{
    ArithmeticType, ScheduledRegionBuildError, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
};

use crate::feasibility::{
    AvailabilityPhase, AxisRequirement, CapabilityAxis, DeferredSet, FeasibilityError,
    FeasibilityOutcome, FeasibilityProposal, ProvenEvidence, RejectionCause,
};
use crate::honourability::{
    DimensionBehaviour, NumericalDimension, NumericalRequirement, UnhonouredDimension,
};
use crate::region::SemanticMemberId;
use crate::request::{
    NormalizedPointwise, NormalizedPointwiseAssociation, NormalizedPointwiseLeaf,
    NormalizedPointwiseOperation, NormalizedProgramSubject, NumericalPermission,
    StrictF32NumericalContract, TargetProfile, VerifiedRequestSubject, VerifiedTargetRequest,
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
    target_profile: TargetProfile,
    request_subject: VerifiedRequestSubject,
    admission: AdmissionEvidence,
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
    pub(crate) const fn target_profile(&self) -> &TargetProfile {
        &self.target_profile
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

    /// Returns the complete hard-feasibility admission for this exact region.
    pub(crate) const fn admission(&self) -> &AdmissionEvidence {
        &self.admission
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

/// Builds the canonical pointwise scheduled region for one request.
///
/// This constructs the raw, not-yet-verified region and its recognized pointwise
/// members, either as a serial-sum prologue that writes an intermediate or as a
/// standalone whole-program region that writes the output. It applies no
/// intrinsic, subject-binding, or feasibility gate. The implementation frontier
/// and its providers use it to obtain a canonical region they then re-submit
/// through the ordinary checked verification path, including for a domain the
/// governed profile cannot dispatch.
pub(crate) fn pointwise_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let (shape, elements, write_tensor, expression, members) =
        if let Some(pointwise) = request.pointwise() {
            (
                pointwise.shape.clone(),
                pointwise.elements,
                TensorRole::Output,
                normalized_pointwise_expression(pointwise),
                pointwise.members.clone(),
            )
        } else {
            let serial = request.serial_sum();
            (
                serial.input_shape.clone(),
                serial.input_elements,
                TensorRole::Intermediate,
                scale_bias_expression(serial.scale_bits, serial.bias_bits),
                serial.members.pointwise().to_vec(),
            )
        };
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: shape,
            accesses: vec![
                Access {
                    tensor: TensorRole::Input,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(0),
                    ownership: None,
                },
                Access {
                    tensor: write_tensor,
                    component_role: None,
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
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: elements,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(1),
                    tensor: write_tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: write_tensor,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: elements,
                },
            },
            scalar_program: ScalarProgram::PointwiseF32(expression),
            numerical: request.numerical_contract().realization(),
        },
        schedule: linear_schedule(elements, OwnershipWitnessId::new(0)),
    };
    (region, members)
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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
                    component_role: None,
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

/// Builds the canonical five-node scale-then-bias pointwise expression.
///
/// Every insertion is statically within the governed limit and uses handles
/// minted earlier by this builder. Keeping the construction here gives region
/// planning one spelling and lets request binding validate that exact spelling
/// without accepting an algebraically similar but unproved expression.
fn scale_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input()
        .expect("the fixed expression has exactly one input");
    let scale = expression
        .constant(scale_bits)
        .expect("the fixed expression is within the node limit");
    let product = expression
        .multiply(input, scale)
        .expect("both operands belong to this builder");
    let bias = expression
        .constant(bias_bits)
        .expect("the fixed expression is within the node limit");
    let root = expression
        .add(product, bias)
        .expect("both operands belong to this builder");
    expression
        .build(root)
        .expect("every fixed-expression node reaches its root")
}

fn normalized_pointwise_expression(normalized: &NormalizedPointwise) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let mut lower_leaf = |leaf| match leaf {
        NormalizedPointwiseLeaf::Input => expression
            .input()
            .expect("the normalized expression has exactly one input"),
        NormalizedPointwiseLeaf::Constant(bits) => expression
            .constant(bits)
            .expect("the normalized expression is within the node limit"),
    };
    let [first, second, third] = normalized.leaves.map(&mut lower_leaf);
    let combine = |builder: &mut PointwiseF32ExpressionBuilder, lhs, rhs| {
        match normalized.operation {
            NormalizedPointwiseOperation::Add => builder.add(lhs, rhs),
            NormalizedPointwiseOperation::Multiply => builder.multiply(lhs, rhs),
        }
        .expect("normalized operands belong to this builder")
    };
    let root = match normalized.association {
        NormalizedPointwiseAssociation::Left => {
            let inner = combine(&mut expression, first, second);
            combine(&mut expression, inner, third)
        }
        NormalizedPointwiseAssociation::Right => {
            let inner = combine(&mut expression, second, third);
            combine(&mut expression, first, inner)
        }
    };
    expression
        .build(root)
        .expect("every normalized-expression node reaches its root")
}

/// Checks the exact canonical scale-then-bias expression recognized by the
/// governed serial-sum request.
///
/// This deliberately checks node topology, ordered operands, constant bits,
/// and the explicit root. Matching only the two constants would let a provider
/// bind an unproved reassociation or a different arithmetic operation to the
/// request's semantic occurrences.
fn scale_bias_expression_matches(
    expression: &PointwiseF32Expression,
    scale_bits: u32,
    bias_bits: u32,
) -> bool {
    let [
        PointwiseF32Node::Input,
        PointwiseF32Node::Constant { bits: actual_scale },
        PointwiseF32Node::Multiply {
            lhs: multiply_lhs,
            rhs: multiply_rhs,
        },
        PointwiseF32Node::Constant { bits: actual_bias },
        PointwiseF32Node::Add {
            lhs: add_lhs,
            rhs: add_rhs,
        },
    ] = expression.nodes()
    else {
        return false;
    };
    *actual_scale == scale_bits
        && *actual_bias == bias_bits
        && multiply_lhs.index() == 0
        && multiply_rhs.index() == 1
        && add_lhs.index() == 2
        && add_rhs.index() == 3
        && expression.root().index() == 4
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
}

/// Verifies one scheduled region and additionally surfaces the resolved
/// feasibility evidence that an admissible target assessment carries.
///
/// This runs the exact checked path [`verify_schedule`] runs — the request-subject
/// precondition, whole-region intrinsic verification, numerical-realization
/// agreement, the request-subject binding, and the single hard-feasibility
/// decision — and additionally returns either complete proof or compiler-minted
/// deferred obligations.
/// The physical implementation frontier retains it as admission evidence for an
/// enumerated proposal. A provider cannot bypass any of these checks: a
/// [`PhysicalError::Target`] or [`PhysicalError::Numerical`] means the proposal
/// is hard-infeasible (never a cost), and any other [`PhysicalError`] means the
/// provider emitted invalid IR.
pub(crate) fn verify_schedule_with_feasibility(
    region: ScheduledRegion,
    semantic_members: Vec<SemanticMemberId>,
    request: &VerifiedTargetRequest,
) -> Result<VerifiedScheduledRegion, PhysicalError> {
    let id = region.index.id;
    let subject = request.subject();
    if !request.reconstructs_its_authority() || !request.numerical_contract().is_governed() {
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
        request.target_profile(),
    )?;
    Ok(VerifiedScheduledRegion {
        verified,
        semantic_members,
        target_profile: request.target_profile().clone(),
        request_subject: subject.clone(),
        admission: evidence,
    })
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
    let expected = match (subject.normalized(), &region.index.scalar_program) {
        (
            NormalizedProgramSubject::Pointwise(normalized),
            ScalarProgram::PointwiseF32(expression),
        ) => {
            element_count(&normalized.shape, region.index.id)? == normalized.elements
                && semantic_members == normalized.members
                && region.index.id == RegionId::new(0)
                && region.index.iteration_shape == normalized.shape
                && expression == &normalized_pointwise_expression(normalized)
        }
        (NormalizedProgramSubject::Pointwise(_), _) => false,
        (NormalizedProgramSubject::SerialSum(normalized), scalar) => {
            if !tiler_ir::schedule::axes_are_canonical(
                normalized.reduction_axes(),
                normalized.input_shape().rank(),
            ) || element_count(normalized.input_shape(), region.index.id)?
                != normalized.input_elements()
                || element_count(normalized.output_shape(), region.index.id)?
                    != normalized.output_elements()
                || normalized
                    .input_shape()
                    .without_axes(normalized.reduction_axes())
                    != *normalized.output_shape()
            {
                return intrinsic("request-subject-shape", region.index.id);
            }
            match scalar {
                ScalarProgram::PointwiseF32(expression) => {
                    semantic_members == normalized.members().pointwise()
                        && region.index.id == RegionId::new(0)
                        && region.index.iteration_shape == *normalized.input_shape()
                        && scale_bias_expression_matches(
                            expression,
                            normalized.scale_bits(),
                            normalized.bias_bits(),
                        )
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
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
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
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
                }
                ScalarProgram::StrictAffineU4Dequantize { .. } => false,
            }
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AdmissionEvidence {
    /// Every hard predicate was resolved before planning.
    Proven(ProvenEvidence),
    /// Every unresolved predicate has a typed query path before routing commit.
    Deferred(DeferredSet),
}

impl AdmissionEvidence {
    /// The checks already proven at compile time.
    pub(crate) const fn proven(&self) -> &ProvenEvidence {
        match self {
            Self::Proven(evidence) => evidence,
            Self::Deferred(deferred) => deferred.proven(),
        }
    }

    /// The remaining compiler-minted obligations, when there are any.
    pub(crate) const fn deferred(&self) -> Option<&DeferredSet> {
        match self {
            Self::Proven(_) => None,
            Self::Deferred(deferred) => Some(deferred),
        }
    }

    /// The capability checks already resolved at compile time.
    pub(crate) fn predicates(&self) -> &[crate::feasibility::ResolvedPredicate] {
        self.proven().predicates()
    }

    /// The numerical dimensions already honoured at compile time.
    pub(crate) fn honoured(&self) -> &[crate::honourability::HonouredDimension] {
        self.proven().honoured()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the unattributed verdict; its second caller is the opaque-call admission being built"
)]
pub(crate) enum ResourceVerdict {
    /// The target profile or the proposal itself was malformed.
    Intrinsic(FeasibilityError),
    /// The target refused the proposal, with the representative cause.
    Rejected(RejectionCause),
    /// At least one predicate has no admissible fact or query path.
    Unknown,
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
    target: &TargetProfile,
) -> Result<AdmissionEvidence, ResourceVerdict> {
    let proposal = region_proposal(requirements, arithmetic, work_items)
        .map_err(ResourceVerdict::Intrinsic)?;
    match target
        .checked()
        .assess(&proposal, AvailabilityPhase::CompileProfile)
    {
        FeasibilityOutcome::Proven(evidence) => Ok(AdmissionEvidence::Proven(evidence)),
        FeasibilityOutcome::Deferred(deferred) if deferred.dimensions().is_empty() => {
            Ok(AdmissionEvidence::Deferred(deferred))
        }
        FeasibilityOutcome::Deferred(_) | FeasibilityOutcome::Unknown(_) => {
            Err(ResourceVerdict::Unknown)
        }
        FeasibilityOutcome::Rejected(rejection) => {
            Err(ResourceVerdict::Rejected(rejection.representative()))
        }
    }
}

/// Assesses one region's resources, attributing any verdict to that region.
pub(crate) fn assess_region(
    region: RegionId,
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
    target: &TargetProfile,
) -> Result<AdmissionEvidence, PhysicalError> {
    assess_resources(requirements, arithmetic, work_items, target).map_err(
        |verdict| match verdict {
            ResourceVerdict::Intrinsic(error) => feasibility_intrinsic(&error, region),
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
            ResourceVerdict::Unknown => PhysicalError::Intrinsic {
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
    target: &TargetProfile,
    contract: StrictF32NumericalContract,
) -> Result<FeasibilityOutcome, FeasibilityError> {
    let proposal = FeasibilityProposal::new(
        CONTRACT_PROPOSAL_CANDIDATE,
        Vec::new(),
        contract.dimension_requirements(),
    )?;
    Ok(target
        .checked()
        .assess(&proposal, AvailabilityPhase::CompileProfile))
}

/// Returns the canonical descriptor bytes of one target profile.
///
/// Borrowed from the same immutable checked profile the feasibility assessment
/// uses, so this path never reconstructs or revalidates target facts.
///
/// This is only half of what ADR 0043 requires an artifact to record. The other
/// half — which feasibility rules compared the candidate against these facts —
/// is [`crate::feasibility::GOVERNED_FEASIBILITY_RULE_SET`], and it is not
/// derived per target because the rules do not vary by target.
#[cfg(test)]
pub(crate) fn target_profile_descriptor(target: &TargetProfile) -> &[u8] {
    target.canonical_descriptor()
}

/// Builds the typed candidate proposal for one scheduled region.
///
/// The candidate requires complete support for the governed unsigned-64 KIR
/// index operation family and the device address space whenever its resource
/// requirements demand it. It does not infer a device address width from that
/// arithmetic type. The prototype baseline needs no local memory and introduces
/// no synchronization obligation. Its numerical requirements are the region's declared
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
            index_arithmetic_requirement(KernelType::Index)
                .expect("the governed KIR index type has an arithmetic requirement"),
            AxisRequirement::new(
                CapabilityAxis::DeviceAddressSpace,
                u64::from(requirements.requires_device_memory),
            ),
            AxisRequirement::new(
                CapabilityAxis::LocalMemoryBytes,
                requirements.local_memory_bytes,
            ),
        ],
        vec![
            NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                arithmetic,
                F32::resolved_type(),
                DimensionBehaviour::Subnormals(requirements.input_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::ResultSubnormals,
                arithmetic,
                F32::resolved_type(),
                DimensionBehaviour::Subnormals(requirements.result_subnormals),
            ),
            NumericalRequirement::new(
                NumericalDimension::Contraction,
                arithmetic,
                F32::resolved_type(),
                DimensionBehaviour::Transform(requirements.contraction),
            ),
            NumericalRequirement::new(
                NumericalDimension::Reassociation,
                arithmetic,
                F32::resolved_type(),
                DimensionBehaviour::Transform(requirements.reassociation),
            ),
        ],
    )
}

/// Derives the hard arithmetic requirement of one governed KIR value type.
///
/// Exhaustive so a new KIR type is a build error until its target requirement
/// is classified. Storage availability alone never satisfies this predicate.
const fn index_arithmetic_requirement(value_type: KernelType) -> Option<AxisRequirement> {
    match value_type {
        KernelType::Index => Some(AxisRequirement::new(CapabilityAxis::IndexArithmeticU64, 1)),
        KernelType::Bool | KernelType::U8 | KernelType::F32 | KernelType::I32 => None,
    }
}

/// Maps a feasibility intrinsic error onto the physical-error contract.
///
/// A malformed profile or proposal is a contract violation, not a feasibility
/// outcome, so it fails closed as an intrinsic scheduling error.
fn feasibility_intrinsic(error: &FeasibilityError, region: RegionId) -> PhysicalError {
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
    /// `target_profile_descriptor(&TargetProfile::governed())` as hex
    /// and step whatever domain tag the change requires in the same commit.
    #[test]
    fn the_governed_descriptor_bytes_do_not_move() {
        // Rebaselined to the complete v10 declaration after separating a
        // future prepared-entry workgroup query from compile-profile facts and
        // replacing the grid placeholder with the API-backed bound four.
        // Device-address width remains absent because no current KIR operation
        // consumes it and the governed authority does not establish it.
        // Every artifact identity and cache entry derived from it moves with it. Regenerate with `cargo nextest run -p tiler-compiler -E 'test(the_governed_descriptor_bytes_do_not_move)'` and take `left`.
        const GOVERNED: &str = "000000000000002574696c65722e7461726765742d70726f66696c652e6465636c61726174696f6e2e76313000000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e7631000000000000002574696c65722e7461726765742d70726f66696c652e666163742d736f75726365732e7634000000000000000001000000000000007400000003010101000000000000002a74696c65722e676f7665726e65642d7461726765742d70726f66696c652d617574686f726974792e76310000000101000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e76310000000100000000000000050000000000000009677269642d61786973040000000000000000000000000000000f6275666665722d62696e64696e6773020000000000000000000000000000000d6465766963652d6d656d6f727901000000000000000000000000000000126c6f63616c2d6d656d6f72792d62797465730000000000000000000000000000000014696e6465782d61726974686d657469632d75363401000000000000000000000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f70657274696573000000010000000000000001000000000000004303000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c6572000000000000000366333200000001000000000000000c000101010100000101020100000201010100000201020100000302010100000302020100000402010100000402020100000502010100000602010100000904010100000a04010100000000000000002e74696c65722e7461726765742d70726f66696c652e64747970652d64697370617463686162696c6974792e7632000000000000000001000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c65720000000000000003663332000000010100";

        let profile = crate::request::TargetProfile::governed();
        let descriptor = target_profile_descriptor(&profile);
        let mut actual = String::with_capacity(descriptor.len() * 2);
        for byte in descriptor {
            write!(actual, "{byte:02x}").expect("writing to a String cannot fail");
        }
        assert_eq!(
            actual, GOVERNED,
            "the governed target profile's canonical descriptor moved; every artifact \
             identity and cache entry derived from it moves with it",
        );
    }
    use super::*;
    use crate::request::{CompilationRequest, StrictF32NumericalContract, verify_request};
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
        request.for_target(0).unwrap()
    }

    fn pointwise_request() -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let first = F32Constant::apply(&mut builder, 1.0e20_f32.to_bits()).unwrap();
        let second = F32Constant::apply(&mut builder, (-1.0e20_f32).to_bits()).unwrap();
        let left = F32Add::apply(&mut builder, input, first).unwrap();
        let root = F32Add::apply(&mut builder, left, second).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let program = builder.build().unwrap();
        let request = verify_request(CompilationRequest::governed_under(
            &program,
            StrictF32NumericalContract::governed_relaxed(),
        ))
        .unwrap();
        request.for_target(0).unwrap()
    }

    #[test]
    fn fixed_schedules_and_kernels_refine_the_two_regions() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
        let regions = build_scheduled_regions(&request).unwrap();
        let pointwise = lower_structured_kernel(&regions[0]).unwrap();
        let reduction = lower_structured_kernel(&regions[1]).unwrap();

        assert_eq!(regions[0].region().schedule.work_items, 4);
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
        assert_eq!(loop_bounds(&reduction), Some((1, 2)));
        assert_eq!(loop_bounds(&pointwise), None);
    }

    #[test]
    fn scheduled_regions_carry_a_transient_independent_identity() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
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
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
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

        let mut wrong_expression = regions[0].region().clone();
        wrong_expression.index.scalar_program = ScalarProgram::PointwiseF32(scale_bias_expression(
            request.serial_sum().bias_bits,
            request.serial_sum().scale_bits,
        ));
        assert_eq!(
            verify_schedule(
                wrong_expression,
                regions[0].semantic_members().to_vec(),
                &request,
            ),
            Err(PhysicalError::Intrinsic {
                rule: "request-binding",
                region: RegionId::new(0),
            })
        );
    }

    #[test]
    fn pointwise_schedule_requires_exact_expression_and_complete_ordered_coverage() {
        let request = pointwise_request();
        let (raw, members) = pointwise_region(&request);
        let region = verify_schedule(raw, members, &request).unwrap();
        let expected = [
            SemanticMemberId(0),
            SemanticMemberId(1),
            SemanticMemberId(2),
            SemanticMemberId(3),
        ];
        assert_eq!(region.semantic_members(), expected);

        let mut wrong_expression = region.region().clone();
        wrong_expression.index.scalar_program = ScalarProgram::PointwiseF32(scale_bias_expression(
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
        ));
        assert!(matches!(
            verify_schedule(
                wrong_expression,
                region.semantic_members().to_vec(),
                &request,
            ),
            Err(PhysicalError::Intrinsic {
                rule: "request-binding",
                ..
            })
        ));

        for forged in [
            expected[..3].to_vec(),
            vec![expected[1], expected[0], expected[2], expected[3]],
            vec![
                expected[0],
                expected[1],
                expected[2],
                expected[3],
                SemanticMemberId(4),
            ],
        ] {
            assert!(matches!(
                verify_schedule(region.region().clone(), forged, &request),
                Err(PhysicalError::Intrinsic {
                    rule: "request-binding",
                    ..
                })
            ));
        }
    }

    #[test]
    fn reduction_access_and_proof_shapes_are_bound_to_the_verified_request() {
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
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
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
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
        let request = request(Shape::from_dims([2, 2]), [Axis::new(1)]);
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
