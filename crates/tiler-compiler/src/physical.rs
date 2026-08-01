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
    Access, AccessMode, BoundsProof, BoundsProofKind, BoundsWitnessId, ContractionAxisSource,
    ContributorOrder, ContributorPartition, ExecutionBinding, IndexRegion, InputOrdinal,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseF32Expression, PointwiseF32Node,
    ReductionTopology, RegionId, ResourceRequirements, ScalarProgram, ScheduledRegion, TailPolicy,
    TensorRole,
};
use tiler_ir::schedule::{
    ArithmeticType, ScheduledRegionBuildError, ScheduledRegionBuilder, ScheduledRegionDiagnostic,
};

use crate::region::SemanticMemberId;
use crate::request::{
    NormalizedContraction, NormalizedProgramSubject, NumericalPermission,
    StrictF32NumericalContract, TargetProfile, VerifiedRequestSubject, VerifiedTargetRequest,
};
use crate::target::feasibility::UnrealizableSynchronization;
use crate::target::feasibility::{
    AvailabilityPhase, AxisRequirement, CapabilityAxis, DeferredSet, FeasibilityError,
    FeasibilityOutcome, FeasibilityProposal, ProvenEvidence, RejectionCause,
};
use crate::target::honourability::{
    DimensionBehaviour, NumericalDimension, NumericalRequirement, UnhonouredDimension,
};

/// The boundary role of a region reading the first declared program input.
///
/// It is the tensor a reduction folds when the recognizer found no elementwise
/// prologue, and the tensor a fused serial sum reads: both are single-access
/// contributor domains over one declared input.
const FIRST_INPUT: TensorRole = TensorRole::Input {
    ordinal: InputOrdinal::FIRST,
};

/// Recovers the scale and bias a fused serial sum's scalar program can spell.
///
/// [`ScalarProgram::FusedMultiplyAddSerialSum`] applies exactly `scale * x +
/// bias` to each contributor, so the fused single-region alternative exists
/// exactly when the recognized prologue *is* that expression over the sole
/// declared input. This decides that by node topology, ordered operands, and the
/// explicit root rather than by the constants alone: an algebraically similar
/// expression with a different association is a different binary32 function, and
/// admitting one here would bind an unproved reassociation to the request's
/// occurrences.
///
/// Returning `None` loses a candidate and never a program — the materialized
/// two-region plan realizes every recognized prologue, and it is what a general
/// prologue compiles through.
///
/// This is the single authority the whole compilation asks: the region builder,
/// the request-subject binding, and the whole-program numerical proof all reach
/// it, so "a fused alternative exists" and "the fused equivalence proof is
/// claimed" cannot disagree.
pub(crate) fn fused_prologue_constants(request: &VerifiedTargetRequest) -> Option<(u32, u32)> {
    affine_prologue(&request.try_serial_sum()?.prologue)
}

/// Recovers the scale and bias one recognized expression spells, or declines.
fn affine_prologue(expression: &PointwiseF32Expression) -> Option<(u32, u32)> {
    let [
        PointwiseF32Node::Input {
            ordinal: InputOrdinal::FIRST,
        },
        PointwiseF32Node::Constant { bits: scale },
        PointwiseF32Node::Multiply {
            lhs: multiply_lhs,
            rhs: multiply_rhs,
        },
        PointwiseF32Node::Constant { bits: bias },
        PointwiseF32Node::Add {
            lhs: add_lhs,
            rhs: add_rhs,
        },
    ] = expression.nodes()
    else {
        return None;
    };
    (multiply_lhs.index() == 0
        && multiply_rhs.index() == 1
        && add_lhs.index() == 2
        && add_rhs.index() == 3
        && expression.root().index() == 4)
        .then_some((*scale, *bias))
}

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
    /// A synchronization realization no available target fact speaks to.
    ///
    /// Distinct from [`Self::Synchronization`], which carries a fact that says
    /// *no*. This one carries no fact because there is none: the profile has
    /// never been asked about this subject. Both reject before
    /// executable-frontier admission and neither is a cost, but only one of them
    /// can name a refusing authority, and inventing one for the other would be
    /// the false attribution the atomic fact exists to prevent.
    UnrealizedSynchronization {
        region: RegionId,
        subject: tiler_ir::schedule::SynchronizationSubject,
    },
    /// A synchronization realization the target declares it cannot provide.
    ///
    /// A distinct variant for the reason [`Self::Numerical`] is one: the
    /// rejection names a complete subject — kind, arrival scope, publication
    /// scope, fenced domains, ordering — and the profile that refused it, none of
    /// which is a quantity. Reporting it as a `Target` bound would restate an
    /// atomic subject as a number and lose exactly what makes it uncomposable.
    Synchronization {
        region: RegionId,
        cause: Box<UnrealizableSynchronization>,
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
            Self::UnrealizedSynchronization { region, subject } => write!(
                formatter,
                "schedule.synchronization.unrealized: region {} requires {} arriving {}, \
                 publishing {}, fencing{}{}, ordered {}; no available fact declares it",
                region.get(),
                subject.kind.key(),
                subject.execution_scope.key(),
                subject.visibility_scope.key(),
                if subject.fenced_spaces.workgroup {
                    " workgroup"
                } else {
                    ""
                },
                if subject.fenced_spaces.device {
                    " device"
                } else {
                    ""
                },
                subject.ordering.key(),
            ),
            Self::Synchronization { region, cause } => {
                let subject = cause.subject();
                write!(
                    formatter,
                    "schedule.synchronization: region {} requires {} arriving {}, publishing {}, \
                     fencing{}{}, ordered {}; profile {} declares it unrealizable",
                    region.get(),
                    subject.kind.key(),
                    subject.execution_scope.key(),
                    subject.visibility_scope.key(),
                    if subject.fenced_spaces.workgroup {
                        " workgroup"
                    } else {
                        ""
                    },
                    if subject.fenced_spaces.device {
                        " device"
                    } else {
                        ""
                    },
                    subject.ordering.key(),
                    cause.fact().provenance().profile().key(),
                )
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
    let (fused, members) = fused_region(request).ok_or(PhysicalError::Intrinsic {
        rule: "fused-prologue-unspellable",
        region: RegionId::new(0),
    })?;
    verify_schedule(fused, members, request)
}

/// Builds the canonical elementwise scheduled region for one request.
///
/// This constructs the raw, not-yet-verified region and its recognized
/// elementwise members, either as a reduction prologue that writes an
/// intermediate or as a standalone whole-program region that writes the output.
/// Both carry the recognizer's own [`PointwiseF32Expression`] rather than a
/// spelling rebuilt here, which is what lets one builder serve every expression
/// the recognizer admits instead of one shape it was taught.
///
/// # Panics
///
/// Panics when asked for a request whose recognized program is a contraction,
/// which is invalid compiler output rather than a caller error: the frontier
/// offers this region only for an elementwise or reduced-elementwise subject. It applies no
/// intrinsic, subject-binding, or feasibility gate. The implementation frontier
/// and its providers use it to obtain a canonical region they then re-submit
/// through the ordinary checked verification path, including for a domain the
/// governed profile cannot dispatch.
pub(crate) fn pointwise_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let (shape, elements, inputs, write_tensor, expression, members) =
        if let Some(pointwise) = request.pointwise() {
            (
                pointwise.shape.clone(),
                pointwise.elements,
                pointwise.input_keys.len(),
                TensorRole::Output,
                pointwise.expression.clone(),
                pointwise.members.clone(),
            )
        } else {
            let serial = request.serial_sum();
            (
                serial.input_shape.clone(),
                serial.input_elements,
                serial.input_keys.len(),
                TensorRole::Intermediate,
                serial.prologue.clone(),
                serial.members.pointwise().to_vec(),
            )
        };
    // One read per input tensor, its ordinal fixed by its position, then the
    // owning write. The write's bounds witness follows the reads rather than
    // sitting at a constant, so witness numbering is access numbering and two
    // accesses cannot end up proving against one witness.
    let write_witness = u32::try_from(inputs).unwrap_or(u32::MAX);
    let mut accesses: Vec<Access> = (0..write_witness)
        .map(|ordinal| Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::new(ordinal),
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(ordinal),
            ownership: None,
        })
        .collect();
    let mut bounds_proofs: Vec<BoundsProof> = (0..write_witness)
        .map(|ordinal| BoundsProof {
            id: BoundsWitnessId::new(ordinal),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::new(ordinal),
            },
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .collect();
    accesses.push(Access {
        tensor: write_tensor,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(write_witness),
        ownership: Some(OwnershipWitnessId::new(0)),
    });
    bounds_proofs.push(BoundsProof {
        id: BoundsWitnessId::new(write_witness),
        tensor: write_tensor,
        component_role: None,
        kind: BoundsProofKind::LinearRange {
            element_count: elements,
        },
    });
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: shape,
            accesses,
            bounds_proofs,
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

/// Derives one contraction operand's coordinate map from the index structure.
///
/// `declaration` is the *declared input ordinal*, which is what binds the
/// region's buffers, and the structure operand it reads is the one the
/// recognizer bound to it. Each operand axis takes the position of its index in
/// the output tuple, or — when the index is contracted rather than free — in the
/// ascending contracted set. Those are the two spaces a `direct` realization
/// walks, and the structure's own derivation guarantees every operand index is
/// in exactly one of them.
///
/// # Panics
///
/// Panics only if the normalized contraction and its own structure disagree
/// about the operand count or about which indices are free, which the recognizer
/// proved they do not.
fn contraction_operand_sources(
    normalized: &NormalizedContraction,
    declaration: usize,
) -> Vec<ContractionAxisSource> {
    let structure = &normalized.structure;
    let tuple = structure
        .operand(normalized.operand_positions[declaration])
        .expect("the recognizer bound every declared input to a structure operand");
    tuple
        .iter()
        .map(|index| {
            if let Some(position) = structure.output().iter().position(|free| free == index) {
                ContractionAxisSource::Output {
                    position: u32::try_from(position).expect("an output tuple is bounded"),
                }
            } else {
                let position = structure
                    .contracted()
                    .iter()
                    .position(|summed| summed == index)
                    .expect("an operand index is free or contracted by the structure's derivation");
                ContractionAxisSource::Contracted {
                    position: u32::try_from(position).expect("a contracted set is bounded"),
                }
            }
        })
        .collect()
}

/// Builds the canonical single-region contraction for one request.
///
/// The `direct` realization the L3 elimination retains: one invocation per
/// output element, each folding its own contracted sequence in ascending order
/// from the first product. Its only precondition is a nonempty contracted space,
/// which the recognizer already established — there is deliberately no tile or
/// split width to refuse against here, and a check that could never fire would
/// be worse than its absence.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn contraction_region(
    request: &VerifiedTargetRequest,
) -> (ScheduledRegion, Vec<SemanticMemberId>) {
    let normalized = request
        .contraction()
        .expect("a contraction region is built only for a contraction request");
    // Two reads then the owning write, with witness numbering equal to access
    // numbering so two accesses cannot prove against one witness.
    let mut accesses = Vec::with_capacity(3);
    let mut bounds_proofs = Vec::with_capacity(3);
    for declaration in 0..normalized.input_keys.len() {
        let witness = u32::try_from(declaration).unwrap_or(u32::MAX);
        let tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(witness),
        };
        accesses.push(Access {
            tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ContractionOperand {
                operand_shape: normalized.input_shapes[declaration].clone(),
                output_shape: normalized.output_shape.clone(),
                contracted_shape: normalized.contracted_shape.clone(),
                sources: contraction_operand_sources(normalized, declaration),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(witness),
            ownership: None,
        });
        bounds_proofs.push(BoundsProof {
            id: BoundsWitnessId::new(witness),
            tensor,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: normalized.input_elements[declaration],
            },
        });
    }
    let write_witness = u32::try_from(accesses.len()).unwrap_or(u32::MAX);
    accesses.push(Access {
        tensor: TensorRole::Output,
        component_role: None,
        mode: AccessMode::Write,
        map: LogicalAccess::LinearIdentity,
        bounds: BoundsWitnessId::new(write_witness),
        ownership: Some(OwnershipWitnessId::new(0)),
    });
    bounds_proofs.push(BoundsProof {
        id: BoundsWitnessId::new(write_witness),
        tensor: TensorRole::Output,
        component_role: None,
        kind: BoundsProofKind::LinearRange {
            element_count: normalized.output_elements,
        },
    });
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: normalized.output_shape.clone(),
            accesses,
            bounds_proofs,
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: normalized.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictTensorContraction {
                contracted_shape: normalized.contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: normalized.contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                // Derived from the contract rather than hard-coded, exactly as
                // every other region here derives them: the schedule verifier
                // cross-checks both against the region's declared realization,
                // and a constant would lose this candidate under a contract that
                // permits either freedom — silently rather than wrongly, but for
                // a reason no diagnostic would name.
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                permits_permutation: request.numerical_contract().permutation
                    != NumericalPermission::Forbidden,
            },
            ..linear_schedule(normalized.output_elements, OwnershipWitnessId::new(0))
        },
    };
    (region, normalized.members.clone())
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

/// Builds the canonical fused whole-program scheduled region for one request,
/// when its scalar program can spell the recognized prologue.
///
/// **The fusion is conditional on the vocabulary, not on the family.**
/// [`ScalarProgram::FusedMultiplyAddSerialSum`] applies one scale and one bias
/// per contributor, so this alternative exists exactly when
/// [`affine_prologue`] recovers those two constants from the recognized
/// expression. A general prologue — `sum((a * b) + c)`, or one over two declared
/// inputs — has no fused spelling in this vocabulary, and `None` therefore loses
/// *a candidate* rather than the program: the materialized two-region plan
/// realizes every recognized prologue, including this one.
///
/// Like [`pointwise_region`], this is the raw region and its recognized members;
/// every gate is applied when the frontier resubmits it through the ordinary
/// checked verification path.
pub(crate) fn fused_region(
    request: &VerifiedTargetRequest,
) -> Option<(ScheduledRegion, Vec<SemanticMemberId>)> {
    let (scale_bits, bias_bits) = fused_prologue_constants(request)?;
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(0),
            iteration_shape: request.serial_sum().output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: FIRST_INPUT,
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
                    tensor: FIRST_INPUT,
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
                scale_bits,
                bias_bits,
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
    Some((region, request.serial_sum().members.all()))
}

/// Chooses the split a multi-pass reduction proposal offers for one extent.
///
/// The chosen contributors-per-partition is the divisor of `contributors`
/// nearest to its integer square root from below, which is the balanced exact
/// split: among splits that cover the sequence exactly once each, it keeps both
/// passes' per-invocation folds as short as one choice can.
///
/// It is deliberately *a* choice and not a calibrated one.
/// `calibrate-and-activate-parallel-reduction-selection` owns replacing it with
/// measured evidence, and nothing in this slice makes a split win on
/// preference.
///
/// Returns `None` when no exact split with at least two partitions and at least
/// two contributors per partition exists — every contributor count below four,
/// and every prime one. A partition holding a single contributor folds nothing,
/// so offering it would add a dispatch that does no work, and an inexact split
/// would leave a ragged final partition this profile does not lower.
pub(crate) fn governed_partition(contributors: u64) -> Option<ContributorPartition> {
    if contributors < 4 {
        return None;
    }
    let mut candidate = contributors.isqrt();
    while candidate >= 2 {
        if contributors.is_multiple_of(candidate) {
            let partitions = contributors / candidate;
            if partitions >= 2 {
                return Some(ContributorPartition {
                    partitions,
                    contributors_per_partition: candidate,
                });
            }
        }
        candidate -= 1;
    }
    None
}

/// Builds the canonical partial pass of a split reduction for one request.
///
/// It splits the *materialized* strategy's reduction rather than the fused one:
/// it reads the pointwise temporary and writes the partial tensor, so the split
/// replaces one dispatch with two and leaves the prologue where it was. Fusing
/// the prologue into this pass would additionally have to reconcile the
/// contraction permission the fused scalar program carries, which is a
/// different question from splitting a contributor sequence.
pub(crate) fn partial_reduction_region(
    request: &VerifiedTargetRequest,
    partition: ContributorPartition,
) -> Option<(ScheduledRegion, Vec<SemanticMemberId>)> {
    let subject = request.serial_sum();
    let partial_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)?;
    let partial_elements = subject.output_elements.checked_mul(partition.partitions)?;
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(2),
            iteration_shape: partial_shape,
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(4),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(5),
                    ownership: Some(OwnershipWitnessId::new(2)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(4),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(5),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: partial_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(2),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: partial_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(
                request,
                tiler_ir::schedule::ReductionPass::Partial,
                partition,
                subject.reduction_axes.clone(),
            ),
            ..linear_schedule(partial_elements, OwnershipWitnessId::new(2))
        },
    };
    Some((region, subject.members.reduction().to_vec()))
}

/// Builds the canonical final pass of a split reduction for one request.
///
/// It reduces the single partition axis of the staged partial tensor, so its
/// axes are deliberately not the request's reduction axes: those were already
/// consumed by the partial pass.
pub(crate) fn final_reduction_region(
    request: &VerifiedTargetRequest,
    partition: ContributorPartition,
) -> Option<(ScheduledRegion, Vec<SemanticMemberId>)> {
    let subject = request.serial_sum();
    let partial_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)?;
    let axes = vec![tiler_ir::schedule::partial_reduction_axis(
        &subject.output_shape,
    )?];
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(3),
            iteration_shape: subject.output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: partial_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(6),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(7),
                    ownership: Some(OwnershipWitnessId::new(3)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(6),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: partial_shape,
                        output_shape: subject.output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(7),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(3),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(
                request,
                tiler_ir::schedule::ReductionPass::Final,
                partition,
                axes,
            ),
            ..linear_schedule(subject.output_elements, OwnershipWitnessId::new(3))
        },
    };
    Some((region, Vec::new()))
}

/// The stable name of the single-workgroup tree strategy in explain output.
pub(crate) const SINGLE_WORKGROUP_TREE_STRATEGY: &str = "tiler.reduction.single-workgroup-tree";

/// Why the governed profile offers no single-workgroup tree of one request's
/// reduction.
///
/// A decline is a fact about *this request*, decided before any region exists,
/// exactly as [`SplitUnavailable`] is. Every reason a *target* cannot run the
/// strategy is deliberately absent from this vocabulary: workgroup memory,
/// workgroup width, and the synchronization realization are resolved by the
/// feasibility authority against the profile, so putting any of them here would
/// let a preference decide legality and would hide the exact refusing bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorkgroupTreeUnavailable {
    /// The resolved numerical contract forbids reassociation.
    ///
    /// The tree regroups the declared contributor sequence, so this is the
    /// permission it consumes. It is the *only* numerical decline here: the
    /// admitted arrival order is fixed by the program, so the strategy consumes
    /// no contributor permutation and a contract withholding permutation
    /// forbids nothing this strategy does.
    ReassociationForbidden,
    /// No exact split of the contributor sequence across participants exists.
    ///
    /// Carries the contributor count, because "which extent admitted none" is
    /// what a reader needs. The tail policy is exact by construction — a ragged
    /// participant would stage a slot the coverage rule requires a writer for,
    /// and a masked lane would break the emitted body's soundness argument — so
    /// an inexact split is declined rather than padded.
    NoAdmissibleParticipantCount {
        /// Contributors one output position folds under this request.
        contributors: u64,
    },
    /// The tree's derived extents, shapes, or tile are not representable.
    Unrepresentable,
}

impl WorkgroupTreeUnavailable {
    /// Returns the stable reason code naming this decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::ReassociationForbidden => "reassociation-forbidden",
            Self::NoAdmissibleParticipantCount { .. } => "no-admissible-participant-count",
            Self::Unrepresentable => "workgroup-tree-unrepresentable",
        }
    }
}

/// Builds the single-workgroup tree reduction of one request, or states why not.
///
/// # The strategy, with every key stated
///
/// One workgroup per output position and `participants` invocations in it.
/// Level 0: every participant serially folds the contiguous contributor range
/// its partition owns and stages the partial in its own slot of workgroup
/// memory. The synchronization point. Level 1: the one committing participant
/// folds the `participants` staged slots in ascending order and performs the
/// region's owning write.
///
/// | Key | Value | Where it is stated |
/// | --- | --- | --- |
/// | topology | depth-two tree, fan-in `contributors_per_partition` then `participants` | [`ContributorPartition`] and [`tiler_ir::schedule::workgroup_tree_tile`] |
/// | active lanes | every participant, then the committing one | the tile's phase participation and its `commit` range |
/// | tail | exact, or declined | [`ContributorPartition::covers`], enforced by the schedule verifier |
/// | workgroup storage | one `f32` slot per participant | the tile's [`tiler_ir::schedule::WorkgroupStaging`] |
/// | accumulation dtype | the resolved contract's arithmetic type | the topology's `accumulation` |
/// | contributor order | original-axis lexicographic within a partition, ascending participant across them | the topology's `order` and `arrival` |
///
/// The participant count is [`governed_partition`]'s balanced exact split — the
/// same *choice*, not a calibrated one, that the multi-pass split makes, and for
/// the same reason: it keeps both levels' folds as short as one choice can, and
/// `calibrate-and-activate-parallel-reduction-selection` owns replacing it with
/// measured evidence. Nothing here makes the tree win.
///
/// # Errors
///
/// Returns the typed [`WorkgroupTreeUnavailable`] the frontier records as a
/// declined strategy. None of them is a compiler fault, and none of them is a
/// target decision.
pub(crate) fn single_workgroup_tree_region(
    request: &VerifiedTargetRequest,
) -> Result<(ScheduledRegion, Vec<SemanticMemberId>), WorkgroupTreeUnavailable> {
    if request.numerical_contract().reassociation == NumericalPermission::Forbidden {
        return Err(WorkgroupTreeUnavailable::ReassociationForbidden);
    }
    let subject = request.serial_sum();
    let contributors =
        reduction_contributors(request).ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let partition = governed_partition(contributors)
        .ok_or(WorkgroupTreeUnavailable::NoAdmissibleParticipantCount { contributors })?;
    let participants = partition.partitions;
    let tile = tiler_ir::schedule::workgroup_tree_tile(participants)
        .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let iteration_shape =
        tiler_ir::schedule::partial_reduction_shape(&subject.output_shape, partition)
            .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let work_items = subject
        .output_elements
        .checked_mul(participants)
        .ok_or(WorkgroupTreeUnavailable::Unrepresentable)?;
    let threads_per_workgroup =
        u32::try_from(participants).map_err(|_| WorkgroupTreeUnavailable::Unrepresentable)?;
    let region = ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(4),
            iteration_shape,
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(8),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(9),
                    ownership: Some(OwnershipWitnessId::new(4)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(8),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape.clone(),
                        output_shape: subject.output_shape.clone(),
                        axes: subject.reduction_axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                // The owned output positions, which is one per *workgroup* and
                // not one per invocation: the tile runs `participants`
                // invocations over each of them.
                BoundsProof {
                    id: BoundsWitnessId::new(9),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.output_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(4),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.output_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: request.numerical_contract().canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: request.numerical_contract().realization(),
        },
        schedule: KernelSchedule {
            threads_per_workgroup,
            reduction: ReductionTopology::CooperativeWorkgroup {
                partition,
                tile,
                axes: subject.reduction_axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: request.numerical_contract().arithmetic,
                permits_reassociation: request.numerical_contract().reassociation
                    != NumericalPermission::Forbidden,
                // Reported as the contract resolves it and deliberately not
                // consulted to admit the strategy: the arrival below is fixed by
                // the program, so a build that later registers a permuting
                // contract does not start admitting trees for the wrong reason.
                permits_permutation: request.numerical_contract().permutation
                    != NumericalPermission::Forbidden,
                arrival: tiler_ir::schedule::ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items, OwnershipWitnessId::new(4))
        },
    };
    Ok((region, subject.members.reduction().to_vec()))
}

/// Why the governed profile offers no multi-pass split of one request's
/// reduction.
///
/// A decline is a *fact about this request*, not a cost and not a compiler
/// fault: the strategy applies to the subject, and the reason it was not offered
/// is what a reader needs in order to know the serial alternative stands alone
/// deliberately. Each variant is therefore carried to the frontier and recorded,
/// rather than expressed as the absence of a proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SplitUnavailable {
    /// The resolved numerical contract forbids reassociation.
    ///
    /// A split *is* a reassociation of the declared contributor sequence, so
    /// this is the one permission it consumes. Proposing anyway would emit a
    /// region the schedule verifier rejects as malformed compiler output, which
    /// would report a caller's numerical choice as a Tiler defect.
    ReassociationForbidden,
    /// No exact split of the contributor sequence exists.
    ///
    /// Carries the exact contributor count so a reader can see *which* extent
    /// admitted no balanced split, rather than only that one did not.
    NoAdmissiblePartition {
        /// Contributors one output position folds under this request.
        contributors: u64,
    },
    /// The split's derived extents or shapes are not representable.
    Unrepresentable,
}

impl SplitUnavailable {
    /// Returns the stable reason code naming this decline.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::ReassociationForbidden => "reassociation-forbidden",
            Self::NoAdmissiblePartition { .. } => "no-admissible-partition",
            Self::Unrepresentable => "split-extent-unrepresentable",
        }
    }
}

/// The stable name of the multi-pass split strategy in explain output.
pub(crate) const MULTI_PASS_SPLIT_STRATEGY: &str = "tiler.reduction.multi-pass-split";

/// The ordered raw stages of one governed multi-pass split.
///
/// The stages are raw and not yet verified, exactly like every other constructor
/// in this module: the frontier resubmits each through the ordinary checked path
/// before any of them is admitted.
pub(crate) struct GovernedSplit {
    /// How the contributor sequence is split, retained for cost and identity.
    pub(crate) partition: ContributorPartition,
    /// The partial pass, then the final pass, each with its claimed members.
    pub(crate) stages: Vec<(ScheduledRegion, Vec<SemanticMemberId>)>,
}

/// Chooses and builds the governed multi-pass split of one request's reduction.
///
/// This is the single authority deciding whether a split is offered at all. It
/// runs *before* any region is constructed for the two permissions that decide
/// the question — the contract's reassociation resolution and the existence of
/// an exact partition — because both are properties of the request rather than
/// of a schedule, and a region built for a request that admits neither is a
/// region the verifier would have to reject.
///
/// # Errors
///
/// Returns the typed [`SplitUnavailable`] the frontier records as a declined
/// strategy. None of them is a compiler fault.
pub(crate) fn split_reduction_regions(
    request: &VerifiedTargetRequest,
) -> Result<GovernedSplit, SplitUnavailable> {
    if request.numerical_contract().reassociation == NumericalPermission::Forbidden {
        return Err(SplitUnavailable::ReassociationForbidden);
    }
    let contributors = reduction_contributors(request).ok_or(SplitUnavailable::Unrepresentable)?;
    let partition = governed_partition(contributors)
        .ok_or(SplitUnavailable::NoAdmissiblePartition { contributors })?;
    let partial =
        partial_reduction_region(request, partition).ok_or(SplitUnavailable::Unrepresentable)?;
    let combine =
        final_reduction_region(request, partition).ok_or(SplitUnavailable::Unrepresentable)?;
    Ok(GovernedSplit {
        partition,
        stages: vec![partial, combine],
    })
}

/// Counts the contributors one output position of a request's reduction folds.
///
/// Derived from the reduced axes' extents rather than from
/// `input_elements / output_elements`, because that division is undefined for
/// an empty kept domain and silently wrong for an empty reduced one — both of
/// which are shapes the request boundary admits.
///
/// Returns `None` only when an axis is out of range or the product overflows,
/// neither of which a verified request can produce; the fail-closed answer is
/// still stated rather than assumed.
fn reduction_contributors(request: &VerifiedTargetRequest) -> Option<u64> {
    let subject = request.serial_sum();
    subject
        .reduction_axes
        .iter()
        .try_fold(1_u64, |total, axis| {
            let position = usize::try_from(axis.get()).ok()?;
            let extent = subject.input_shape.extents().get(position)?;
            total.checked_mul(extent.get())
        })
}

/// Reads back the split contract one verified partial pass declares.
///
/// The program assembler needs the partition to declare its
/// [`tiler_ir::program::PartialReduction`], and reading it from the region the
/// pass actually carries — rather than re-deriving it from the request — is what
/// makes the program-scope declaration agree with the schedule that produced it
/// by construction instead of by a second derivation.
pub(crate) fn declared_partial_partition(region: &ScheduledRegion) -> Option<ContributorPartition> {
    match &region.schedule.reduction {
        ReductionTopology::MultiPass {
            pass: tiler_ir::schedule::ReductionPass::Partial,
            partition,
            ..
        } => Some(*partition),
        _ => None,
    }
}

/// Builds the reduction topology one pass of a split declares.
///
/// Both permissions are read from the resolved contract and carried
/// independently. Permutation is reported as the contract resolves it rather
/// than hardcoded, because the split neither needs nor consumes it: the schedule
/// verifier admits the topology on reassociation alone, so a build that later
/// registers a permuting contract does not silently start admitting splits for
/// the wrong reason.
fn multi_pass_topology(
    request: &VerifiedTargetRequest,
    pass: tiler_ir::schedule::ReductionPass,
    partition: ContributorPartition,
    axes: Vec<tiler_ir::shape::Axis>,
) -> ReductionTopology {
    ReductionTopology::MultiPass {
        pass,
        partition,
        axes,
        order: ContributorOrder::OriginalAxisLexicographic,
        accumulation: request.numerical_contract().arithmetic,
        permits_reassociation: request.numerical_contract().reassociation
            != NumericalPermission::Forbidden,
        permits_permutation: request.numerical_contract().permutation
            != NumericalPermission::Forbidden,
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
                // The recognized expression itself, compared whole. It binds
                // node topology, ordered operands, constant bits, shared reads,
                // and the explicit root, so a provider cannot substitute an
                // algebraically similar but unproved expression for it.
                && expression == &normalized.expression
        }
        (
            NormalizedProgramSubject::Contraction(normalized),
            ScalarProgram::StrictTensorContraction {
                contracted_shape,
                canonical_nan_bits,
                ..
            },
        ) => {
            // Every quantity the region carries is re-derived from the subject
            // and compared, including both operands' coordinate maps: a region
            // whose access relation differs from the recognized structure's
            // would compute a different contraction over the same buffers, and
            // the intrinsic verifier — which sees only the region — cannot
            // notice that.
            element_count(&normalized.output_shape, region.index.id)? == normalized.output_elements
                && element_count(&normalized.contracted_shape, region.index.id)?
                    == normalized.contracted_elements
                && semantic_members == normalized.members
                && region.index.id == RegionId::new(0)
                && region.index.iteration_shape == normalized.output_shape
                && contracted_shape == &normalized.contracted_shape
                && *canonical_nan_bits == subject.numerical_contract().canonical_arithmetic_nan_bits
                && contraction_accesses_match(&region.index.accesses, normalized)
        }
        // Either whole-program subject paired with any other scalar program is a
        // forged pairing: each is bound above against the one program its
        // recognizer produces, so answering `false` here is the fail-closed
        // answer rather than a deferral.
        (NormalizedProgramSubject::Pointwise(_) | NormalizedProgramSubject::Contraction(_), _) => {
            false
        }
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
            // A split reduction's two passes bind to the same subject as the
            // fused region does — they realize the same occurrences by a
            // different physical route — but neither has the fused region's
            // iteration shape, so they are matched on their own terms rather
            // than by relaxing the single-dispatch rules below.
            if matches!(
                region.schedule.reduction,
                ReductionTopology::MultiPass { .. }
            ) {
                return verify_multi_pass_subject_binding(
                    region,
                    semantic_members,
                    normalized,
                    subject,
                );
            }
            // A single-workgroup tree realizes the same occurrences as the
            // materialized reduction region by a different physical route, and
            // like a partial pass it iterates the output shape once per
            // participant, so it is matched on its own terms rather than by
            // relaxing the single-dispatch rules below.
            if matches!(
                region.schedule.reduction,
                ReductionTopology::CooperativeWorkgroup { .. }
            ) {
                return verify_workgroup_tree_subject_binding(
                    region,
                    semantic_members,
                    normalized,
                    subject,
                );
            }
            match scalar {
                ScalarProgram::PointwiseF32(expression) => {
                    // The recognized prologue itself, compared whole: node
                    // topology, ordered operands, constant bits, shared reads,
                    // and the explicit root. A provider cannot substitute an
                    // algebraically similar but unproved expression for it.
                    normalized.prologue() == expression
                        && semantic_members == normalized.members().pointwise()
                        && region.index.id == RegionId::new(0)
                        && region.index.iteration_shape == *normalized.input_shape()
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
                        // Re-derived from the recognized prologue rather than
                        // read back: the fused scalar program is admitted only
                        // for the one expression it can spell, so a prologue
                        // that is not that expression has no fused form at all.
                        && affine_prologue(normalized.prologue())
                            == Some((*scale_bits, *bias_bits))
                        && axes == normalized.reduction_axes()
                        && reduction_access_matches(&region.index.accesses[0], normalized)
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
                }
                // None of these is produced by the recognized whole-program
                // shapes this arm verifies. The strict-affine one is refused
                // upstream; the squaring-prologue sum belongs to
                // `tiler::rms-norm-f32@1` and the extrema fold to
                // `tiler::softmax-f32@1`, neither of which the recognizer admits;
                // and a contraction binds to its own subject variant above, so a
                // serial-sum subject claiming one is a forged pairing. Answering
                // `false` is the fail-closed answer rather than a deferral.
                //
                // The extrema fold could not bind here even if the recognizer
                // admitted it: this arm's subject carries an empty-domain
                // identity, and the identity-less family has none to compare.
                ScalarProgram::StrictAffineU4Dequantize { .. }
                | ScalarProgram::SquaredSerialSum { .. }
                | ScalarProgram::StrictSerialMaximum { .. }
                | ScalarProgram::StrictTensorContraction { .. } => false,
            }
        }
    };
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Binds one pass of a split reduction to the request subject it refines.
///
/// The partial pass claims the reduction occurrence, exactly as the
/// materialized strategy's single reduction region does; the final pass claims
/// none, because that occurrence is already covered and claiming it again would
/// double-cover the graph. That asymmetry is what lets the two passes together
/// contribute the same coverage the one region they replace contributed.
fn verify_multi_pass_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticMemberId],
    normalized: &crate::request::NormalizedSerialSumSubject,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let ReductionTopology::MultiPass {
        pass, partition, ..
    } = &region.schedule.reduction
    else {
        return intrinsic("request-binding", region.index.id);
    };
    let expected = match pass {
        tiler_ir::schedule::ReductionPass::Partial => {
            matches!(
                &region.index.scalar_program,
                ScalarProgram::StrictSerialSum { axes, canonical_nan_bits, .. }
                    if axes == normalized.reduction_axes()
                        && *canonical_nan_bits
                            == subject.numerical_contract().canonical_arithmetic_nan_bits
            ) && semantic_members == normalized.members().reduction()
                && region.index.id == RegionId::new(2)
                && reduction_access_matches(&region.index.accesses[0], normalized)
                && tiler_ir::schedule::partial_reduction_shape(
                    normalized.output_shape(),
                    *partition,
                )
                .is_some_and(|shape| shape == region.index.iteration_shape)
        }
        tiler_ir::schedule::ReductionPass::Final => {
            matches!(
                &region.index.scalar_program,
                ScalarProgram::StrictSerialSum { canonical_nan_bits, .. }
                    if *canonical_nan_bits
                        == subject.numerical_contract().canonical_arithmetic_nan_bits
            ) && semantic_members.is_empty()
                && region.index.id == RegionId::new(3)
                && region.index.iteration_shape == *normalized.output_shape()
        }
        // A pass role this compilation does not construct binds to no subject.
        // Refusing rather than guessing keeps a role added later from being
        // silently accepted under the rules the two known roles were checked
        // against.
        _ => false,
    };
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Binds one single-workgroup tree region to the request subject it refines.
///
/// It claims the reduction occurrence, exactly as the materialized strategy's
/// single reduction region does and as the split's partial pass does: the tree
/// *replaces* that one region rather than adding a stage, so there is no second
/// region to leave the occurrence to.
///
/// The participant count is re-derived from the request rather than read from
/// the topology, because reading it back would make this check agree with
/// whatever the provider chose instead of with what the request admits — the one
/// thing a subject binding exists to stop.
fn verify_workgroup_tree_subject_binding(
    region: &ScheduledRegion,
    semantic_members: &[SemanticMemberId],
    normalized: &crate::request::NormalizedSerialSumSubject,
    subject: &VerifiedRequestSubject,
) -> Result<(), PhysicalError> {
    let ReductionTopology::CooperativeWorkgroup { partition, .. } = &region.schedule.reduction
    else {
        return intrinsic("request-binding", region.index.id);
    };
    let expected = matches!(
        &region.index.scalar_program,
        ScalarProgram::StrictSerialSum { axes, canonical_nan_bits, .. }
            if axes == normalized.reduction_axes()
                && *canonical_nan_bits
                    == subject.numerical_contract().canonical_arithmetic_nan_bits
    ) && semantic_members == normalized.members().reduction()
        && region.index.id == RegionId::new(4)
        && reduction_access_matches(&region.index.accesses[0], normalized)
        && tiler_ir::schedule::partial_reduction_shape(normalized.output_shape(), *partition)
            .is_some_and(|shape| shape == region.index.iteration_shape);
    if !expected {
        return intrinsic("request-binding", region.index.id);
    }
    Ok(())
}

/// Requires both operand reads to realize the recognized structure exactly.
///
/// Checked per declared input ordinal rather than as a set: the ordinal is the
/// buffer position, so two accesses carrying the right pair of maps in the wrong
/// order would bind each operand to the other's tensor and still look complete.
fn contraction_accesses_match(accesses: &[Access], normalized: &NormalizedContraction) -> bool {
    let Some((_, reads)) = accesses.split_last() else {
        return false;
    };
    if reads.len() != normalized.input_shapes.len() {
        return false;
    }
    reads.iter().enumerate().all(|(declaration, read)| {
        u32::try_from(declaration).is_ok_and(|ordinal| {
            read.tensor
                == TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                }
        }) && read.map
            == LogicalAccess::ContractionOperand {
                operand_shape: normalized.input_shapes[declaration].clone(),
                output_shape: normalized.output_shape.clone(),
                contracted_shape: normalized.contracted_shape.clone(),
                sources: contraction_operand_sources(normalized, declaration),
                order: ContributorOrder::OriginalAxisLexicographic,
            }
    })
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
    pub(crate) fn predicates(&self) -> &[crate::target::feasibility::ResolvedPredicate] {
        self.proven().predicates()
    }

    /// The numerical dimensions already honoured at compile time.
    pub(crate) fn honoured(&self) -> &[crate::target::honourability::HonouredDimension] {
        self.proven().honoured()
    }

    /// The synchronization realization already established at compile time.
    ///
    /// `None` for a region that requires none, which is what keeps a
    /// zero-synchronization program's explanation free of a manufactured row.
    pub(crate) const fn synchronization(
        &self,
    ) -> Option<&crate::target::feasibility::RealizedSynchronization> {
        self.proven().synchronization()
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
    /// The proposal requires a synchronization realization nothing declares.
    ///
    /// Separated from [`Self::Unknown`] because the two blame different things.
    /// A capability or dimension with no path is a gap in the profile's own
    /// vocabulary, which a caller reports as an unresolved assessment. A
    /// synchronization subject nothing speaks to is a *complete, well-formed*
    /// requirement this target has simply never been asked about — the exact
    /// case `admit-the-first-typed-synchronization-point-and-atomic-target-authority`
    /// specified as "`Unknown` … before executable-frontier admission" — and
    /// reporting it as an unresolved assessment would attribute a target's
    /// silence to the provider that emitted valid IR.
    UnrealizedSynchronization(tiler_ir::schedule::SynchronizationSubject),
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
        // An unknown that names a synchronization subject keeps that subject.
        // The `if` order is deliberate: a candidate can be unknown on a
        // capability *and* on its synchronization, and the synchronization is
        // the more specific answer — it names a complete subject rather than a
        // missing bound, so reporting it loses nothing a reader could act on.
        FeasibilityOutcome::Unknown(unknown) if unknown.synchronization().is_some() => {
            Err(ResourceVerdict::UnrealizedSynchronization(
                unknown
                    .synchronization()
                    .expect("the guard proved the unknown names a subject")
                    .subject(),
            ))
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
            ResourceVerdict::Rejected(RejectionCause::Synchronization(cause)) => {
                PhysicalError::Synchronization {
                    region,
                    cause: Box::new(cause),
                }
            }
            ResourceVerdict::Rejected(RejectionCause::Capability(predicate)) => {
                PhysicalError::Target {
                    rule: predicate.axis().key(),
                    region,
                    required: predicate.required().value(),
                    available: predicate.available().value(),
                }
            }
            ResourceVerdict::UnrealizedSynchronization(subject) => {
                PhysicalError::UnrealizedSynchronization { region, subject }
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
/// is [`crate::target::feasibility::GOVERNED_FEASIBILITY_RULE_SET`], and it is not
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
/// arithmetic type. Its numerical requirements are the region's declared
/// realization carried forward **per dimension** rather than collapsed into one
/// summary bit — the collapse the retired `StrictF32Arithmetic` axis forced, and
/// which could neither name a failing dimension nor express emulation.
///
/// The synchronization requirement is carried the *opposite* way: forward as one
/// atomic subject rather than per dimension, because each of its dimensions is
/// separately true of some realization and their conjunction is what the region
/// needs. It is also carried **conditionally**, and that is what keeps the
/// absence canonical: a region requiring no synchronization composes no
/// requirement at all, so no predicate is resolved, no target fact is consulted,
/// and no explain row exists to be a manufactured zero — exactly as
/// `index_arithmetic_requirement` yields nothing for a value type with no
/// arithmetic obligation, and as the retired barrier-count axis must never again
/// yield `required 0`.
fn region_proposal(
    requirements: ResourceRequirements,
    arithmetic: ArithmeticType,
    work_items: u64,
) -> Result<FeasibilityProposal, FeasibilityError> {
    FeasibilityProposal::new_with_synchronization(
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
        requirements.synchronization,
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

    /// Builds the five-node `input * scale + bias` expression as a forgery.
    ///
    /// Test-only and deliberately not shared with the region builders: those
    /// carry whatever expression the recognizer produced, and a helper they also
    /// used could not be substituted for one of them here.
    fn test_affine_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
        let mut expression = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
        let input = expression.input(InputOrdinal::FIRST).unwrap();
        let scale = expression.constant(scale_bits).unwrap();
        let product = expression.multiply(input, scale).unwrap();
        let bias = expression.constant(bias_bits).unwrap();
        let root = expression.add(product, bias).unwrap();
        expression.build(root).unwrap()
    }
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
        // Rebaselined when the governed profile raised its declared
        // buffer-binding bound from two to four — the widest signature the
        // bounded profile can assemble now that a region may read several input
        // tensors. Exactly one byte of the `buffer-bindings` row moves; the
        // declaration's shape and its domain tag are unchanged, so no domain
        // steps with it.
        //
        // An earlier rebaseline recorded the complete v10 declaration after
        // separating a future prepared-entry workgroup query from
        // compile-profile facts and replacing the grid placeholder with the
        // API-backed bound four. Device-address width remains absent because no
        // current KIR operation consumes it and the governed authority does not
        // establish it.
        // Rebaselined again at the `tiler.target-profile.declaration.v11` step,
        // which appends the synchronization-realization row family. The governed
        // profile declares *no* row, and its bytes still move: the family writes
        // its own domain separator and a count, so "this target says nothing
        // about synchronization" becomes a recorded fact. That is the step's
        // purpose — a `v10` declaration could not distinguish a target that had
        // been asked from one that had not.
        // Every artifact identity and cache entry derived from it moves with it. Regenerate with `cargo nextest run -p tiler-compiler -E 'test(the_governed_descriptor_bytes_do_not_move)'` and take `left`.
        const GOVERNED: &str = "000000000000002574696c65722e7461726765742d70726f66696c652e6465636c61726174696f6e2e76313100000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e7631000000000000002574696c65722e7461726765742d70726f66696c652e666163742d736f75726365732e7634000000000000000001000000000000007400000003010101000000000000002a74696c65722e676f7665726e65642d7461726765742d70726f66696c652d617574686f726974792e76310000000101000000000000002a74696c65722e70726f746f747970652d7461726765742d6e65757472616c2d626173656c696e652e76310000000100000000000000050000000000000009677269642d61786973040000000000000000000000000000000f6275666665722d62696e64696e6773040000000000000000000000000000000d6465766963652d6d656d6f727901000000000000000000000000000000126c6f63616c2d6d656d6f72792d62797465730000000000000000000000000000000014696e6465782d61726974686d657469632d75363401000000000000000000000000000000010000000000000015746872656164732d7065722d776f726b67726f7570000000000000009274696c65722e7461726765742d70726f70657274792d71756572792e763100000000000000003874696c65722e7461726765742e70726570617265642d656e7472792e6d61782d746872656164732d7065722d776f726b67726f75702e763104000000000000000574696c6572000000000000001970726570617265642d656e7472792d70726f70657274696573000000010000000000000001000000000000004303000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c6572000000000000000366333200000001000000000000000c000101010100000101020100000201010100000201020100000302010100000302020100000402010100000402020100000502010100000602010100000904010100000a04010100000000000000002e74696c65722e7461726765742d70726f66696c652e64747970652d64697370617463686162696c6974792e7632000000000000000001000000000000003a74696c65722e7265736f6c7665642d76616c75652d747970652e76330001000000000000000574696c65720000000000000003663332000000010100000000000000003474696c65722e7461726765742d70726f66696c652e73796e6368726f6e697a6174696f6e2d7265616c697a6174696f6e2e7631000000000000000000";

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

        // The scale and the bias exchanged: the same two constants in the same
        // two node positions, applied the other way round. It is a different
        // binary32 function, and the binding compares the whole expression
        // rather than its constant set, so it must be refused.
        let (scale, bias) = fused_prologue_constants(&request).expect("the fixture is affine");
        let mut wrong_expression = regions[0].region().clone();
        wrong_expression.index.scalar_program =
            ScalarProgram::PointwiseF32(test_affine_expression(bias, scale));
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
        wrong_expression.index.scalar_program = ScalarProgram::PointwiseF32(
            test_affine_expression(2.0_f32.to_bits(), 1.0_f32.to_bits()),
        );
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
