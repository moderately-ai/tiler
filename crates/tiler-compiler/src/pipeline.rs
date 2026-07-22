use std::error::Error;
use std::fmt;

use crate::explain::{
    CostDisposition, CostModelKey, CostTerm, EvidenceBasis, ExplainError, ExplainEvent,
    ExplainLimits, ExplainRecordId, ExplainStage, ExplainWriter, PredicateAssessment, PredicateKey,
    ProviderRef, Quantity, ReasonCode, RejectionClass, ResourceKey, RuleRef, SelectionOutcome,
    SelectionPolicyKey, SubjectKind, VerifiedEvidenceRef, VerifiedExplainTrace,
};
use crate::fusion::{
    CandidateError, CandidateKind, FusionNumericalProof, enumerate_candidates,
    prove_fused_numerics, verify_fused_numerics,
};
use crate::physical::{
    PhysicalError, VerifiedScheduledRegion, VerifiedStructuredKernel, build_fused_scheduled_region,
    build_scheduled_regions, lower_structured_kernel,
};
use crate::program::{
    ArtifactConstructionPlan, KernelProgram, ProgramError, assert_kernels_match_program,
    build_artifact_plan, build_fused_kernel_program, build_kernel_program, verify_artifact_plan,
    verify_semantic_output_type,
};
use crate::request::{CompilationRequest, RequestError, verify_request};

const SELECTION_POLICY_KEY: &str = "tiler.selection.structural-pareto.v1";
const STRUCTURAL_COST_MODEL_KEY: &str = "tiler.cost.structural.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompilationProduct {
    pub(crate) targets: Vec<TargetCompilationProduct>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetCompilationProduct {
    pub(crate) target_profile_key: &'static str,
    pub(crate) portfolio: ProgramPortfolio,
    pub(crate) explain: VerifiedExplainTrace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramAlternativeKind {
    Materialized,
    Fused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StructuralCost {
    pub(crate) model_key: &'static str,
    pub(crate) dispatch_count: u32,
    pub(crate) temporary_allocation_count: u32,
    pub(crate) materialized_bytes: u64,
    pub(crate) intermediate_global_reads: u64,
    pub(crate) intermediate_global_writes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EquivalenceEvidence {
    MaterializedReference,
    Fused(Box<FusionNumericalProof>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramAlternative {
    pub(crate) stable_id: &'static str,
    pub(crate) kind: ProgramAlternativeKind,
    pub(crate) scheduled_regions: Vec<VerifiedScheduledRegion>,
    pub(crate) kernels: Vec<VerifiedStructuredKernel>,
    pub(crate) program: KernelProgram,
    pub(crate) artifact_plan: ArtifactConstructionPlan,
    pub(crate) structural_cost: StructuralCost,
    pub(crate) equivalence: EquivalenceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PortfolioSelection {
    pub(crate) policy_key: &'static str,
    pub(crate) selected_alternative_id: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramPortfolio {
    pub(crate) alternatives: Vec<ProgramAlternative>,
    pub(crate) selection: PortfolioSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompileError {
    InvalidRequest(RequestError),
    UnsupportedCapability(RequestError),
    BudgetExhausted(RequestError),
    NoFeasiblePlan(NoFeasiblePlanError),
    InvalidCompilerOutput(CompilerOutputError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NoFeasiblePlanError {
    Request(RequestError),
    Physical(PhysicalError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompilerOutputError {
    Physical(PhysicalError),
    Program(ProgramError),
    Candidate(CandidateError),
    Explain(ExplainError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error)
            | Self::UnsupportedCapability(error)
            | Self::BudgetExhausted(error)
            | Self::NoFeasiblePlan(NoFeasiblePlanError::Request(error)) => error.fmt(formatter),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(error)) => error.fmt(formatter),
            Self::InvalidCompilerOutput(CompilerOutputError::Physical(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Program(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Candidate(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => {
                error.fmt(formatter)
            }
        }
    }
}

impl Error for CompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRequest(error)
            | Self::UnsupportedCapability(error)
            | Self::BudgetExhausted(error)
            | Self::NoFeasiblePlan(NoFeasiblePlanError::Request(error)) => Some(error),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(error))
            | Self::InvalidCompilerOutput(CompilerOutputError::Physical(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Program(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Candidate(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => Some(error),
        }
    }
}

impl From<RequestError> for CompileError {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::UnsupportedCapability { .. } => Self::UnsupportedCapability(value),
            RequestError::ShapeProductOverflow { .. } => {
                Self::NoFeasiblePlan(NoFeasiblePlanError::Request(value))
            }
            RequestError::BudgetExceeded { .. } => Self::BudgetExhausted(value),
            RequestError::UnsupportedRequestVersion
            | RequestError::EmptyTargetSet
            | RequestError::DuplicateTargetProfile
            | RequestError::UnverifiedTargetSelection => Self::InvalidRequest(value),
        }
    }
}

impl From<PhysicalError> for CompileError {
    fn from(value: PhysicalError) -> Self {
        match value {
            PhysicalError::Intrinsic { .. }
            | PhysicalError::Refinement { .. }
            | PhysicalError::ShapeProductOverflow { .. } => {
                Self::InvalidCompilerOutput(CompilerOutputError::Physical(value))
            }
            PhysicalError::Target { .. } => {
                Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(value))
            }
        }
    }
}

impl From<ProgramError> for CompileError {
    fn from(value: ProgramError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Program(value))
    }
}

impl From<ExplainError> for CompileError {
    fn from(value: ExplainError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Explain(value))
    }
}

pub(crate) fn compile(request: CompilationRequest<'_>) -> Result<CompilationProduct, CompileError> {
    let semantic = request.program;
    let verified = verify_request(request)?;
    verify_semantic_output_type(semantic)?;
    let targets = verified
        .target_profiles()
        .iter()
        .copied()
        .map(|target| {
            let target_request = verified.for_target(target)?;
            compile_target(semantic, &target_request)
        })
        .collect::<Result<_, _>>()?;
    Ok(CompilationProduct { targets })
}

fn compile_target(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
) -> Result<TargetCompilationProduct, CompileError> {
    let mut explain = ExplainWriter::new(verified, ExplainLimits::default())?;
    let request_subject = explain.subject(SubjectKind::SemanticProgram, "semantic-program")?;
    let request_record = explain.push_detail(
        RuleRef::builtin("compile.request.general-boundary")?,
        vec![request_subject],
        check(
            ExplainStage::RequestVerification,
            "compile.request.verified",
            EvidenceBasis::CheckedInvariant,
        )?,
        Vec::new(),
    )?;
    let normalization_subject =
        explain.subject(SubjectKind::Normalization, "normalization:serial-sum")?;
    let normalization_record = explain.push_detail(
        RuleRef::builtin("normalize.serial-sum.v1")?,
        vec![normalization_subject],
        check(
            ExplainStage::Normalization,
            "normalize.semantic-equivalence",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(request_record),
    )?;
    let mut alternatives = Vec::new();
    let mut alternative_causes = Vec::new();
    let mut target_rejection = None;
    match build_baseline_alternative(semantic, verified) {
        Ok(baseline) => {
            let cause = record_baseline_explain(&mut explain, verified, normalization_record)?;
            alternative_causes.push((baseline.stable_id, cause));
            alternatives.push(baseline);
        }
        Err(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
            error @ PhysicalError::Target { .. },
        ))) => {
            record_target_rejection(
                &mut explain,
                &error,
                "alternative:materialized-serial-sum.v1",
                normalization_record,
            )?;
            target_rejection = Some(error);
        }
        Err(error) => return Err(error),
    }
    consider_fused_alternative(
        semantic,
        verified,
        &mut alternatives,
        &mut explain,
        normalization_record,
        &mut alternative_causes,
        &mut target_rejection,
    )?;
    if alternatives.is_empty() {
        let error = target_rejection.ok_or({
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "portfolio-empty-without-target-rejection",
                },
            ))
        })?;
        return Err(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
            error,
        )));
    }
    let selected_alternative_id = select_structural_pareto(&alternatives)?;
    verify_portfolio(semantic, verified, &alternatives, selected_alternative_id)?;
    record_cost_and_selection(
        &alternatives,
        selected_alternative_id,
        &alternative_causes,
        &mut explain,
    )?;
    let explain = explain.finish()?;
    Ok(TargetCompilationProduct {
        target_profile_key: verified.target_profile().key,
        portfolio: ProgramPortfolio {
            alternatives,
            selection: PortfolioSelection {
                policy_key: SELECTION_POLICY_KEY,
                selected_alternative_id,
            },
        },
        explain,
    })
}

fn build_baseline_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
) -> Result<ProgramAlternative, CompileError> {
    let baseline_regions = build_scheduled_regions(verified)?;
    let baseline_kernels = baseline_regions
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()?;
    let baseline_program = build_kernel_program(verified, &baseline_regions)?;
    assert_kernels_match_program(
        verified,
        &baseline_regions,
        &baseline_program,
        &baseline_kernels,
    )?;
    let baseline_artifact = build_artifact_plan(
        semantic,
        verified,
        &baseline_regions,
        &baseline_kernels,
        &baseline_program,
        vec![verified.capabilities().materialized_serial_sum],
    )?;
    let input_bytes =
        verified
            .serial_sum()
            .input_elements
            .checked_mul(4)
            .ok_or(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
                RequestError::ShapeProductOverflow {
                    role: "input-bytes",
                },
            )))?;
    Ok(ProgramAlternative {
        stable_id: "alternative:materialized-serial-sum.v1",
        kind: ProgramAlternativeKind::Materialized,
        scheduled_regions: baseline_regions,
        kernels: baseline_kernels,
        program: baseline_program,
        artifact_plan: baseline_artifact,
        structural_cost: StructuralCost {
            model_key: STRUCTURAL_COST_MODEL_KEY,
            dispatch_count: 2,
            temporary_allocation_count: 1,
            materialized_bytes: input_bytes,
            intermediate_global_reads: input_bytes,
            intermediate_global_writes: input_bytes,
        },
        equivalence: EquivalenceEvidence::MaterializedReference,
    })
}

fn check(
    stage: ExplainStage,
    predicate: &str,
    basis: EvidenceBasis,
) -> Result<ExplainEvent, ExplainError> {
    Ok(ExplainEvent::Check {
        stage,
        assessment: PredicateAssessment::proven(predicate, basis)?,
        rejection: if stage == ExplainStage::NumericalLegality {
            RejectionClass::NumericalIllegal
        } else {
            RejectionClass::IntrinsicInvalid
        },
    })
}

fn optional_cause(cause: Option<ExplainRecordId>) -> Vec<ExplainRecordId> {
    cause.into_iter().collect()
}

fn record_baseline_explain(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    root: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, CompileError> {
    let pointwise = explain.subject(
        SubjectKind::Region,
        "alternative:materialized-serial-sum.v1/region:0",
    )?;
    let sum = explain.subject(
        SubjectKind::Region,
        "alternative:materialized-serial-sum.v1/region:1",
    )?;
    let first = explain.push_detail(
        RuleRef::provided(
            "compile.region.pointwise",
            1,
            ProviderRef::lowering(request.capabilities().materialized_serial_sum)?,
        )?,
        vec![pointwise.clone()],
        check(
            ExplainStage::RegionFormation,
            "region.semantic-coverage",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(root),
    )?;
    let second = explain.push_detail(
        RuleRef::provided(
            "compile.region.strict-sum",
            1,
            ProviderRef::lowering(request.capabilities().materialized_serial_sum)?,
        )?,
        vec![sum.clone()],
        check(
            ExplainStage::RegionFormation,
            "region.semantic-coverage",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(root),
    )?;
    let boundary = explain.subject(SubjectKind::Boundary, "pointwise-to-sum")?;
    let mut boundary_causes = optional_cause(first);
    boundary_causes.extend(optional_cause(second));
    let boundary = explain.push_detail(
        RuleRef::builtin("compile.boundary.materialized")?,
        vec![boundary],
        check(
            ExplainStage::RegionFormation,
            "boundary.materialized",
            EvidenceBasis::CheckedInvariant,
        )?,
        boundary_causes,
    )?;
    record_baseline_refinement(explain, request, boundary)
}

fn record_baseline_refinement(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    boundary: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, CompileError> {
    let schedule = explain.subject(
        SubjectKind::Schedule,
        "alternative:materialized-serial-sum.v1/schedule",
    )?;
    let schedule = explain.push_detail(
        RuleRef::builtin("schedule.coverage-and-ownership")?,
        vec![schedule],
        check(
            ExplainStage::IntrinsicScheduling,
            "schedule.intrinsic-valid",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(boundary),
    )?;
    let target = explain.subject(SubjectKind::Target, request.target_profile().key)?;
    let target = explain.push_detail(
        RuleRef::builtin("target.prototype-target-neutral-baseline.v1")?,
        vec![target],
        ExplainEvent::Feasibility {
            predicate: PredicateKey::new("grid-axis")?,
            outcome: crate::explain::FeasibilityOutcome::Admitted,
            required: Quantity::Threads(
                request
                    .serial_sum()
                    .input_elements
                    .max(request.serial_sum().output_elements),
            ),
            available: Quantity::Threads(request.target_profile().max_threads_per_grid_axis),
        },
        optional_cause(schedule),
    )?;
    let kernel = explain.subject(
        SubjectKind::Kernel,
        "alternative:materialized-serial-sum.v1/kernels",
    )?;
    let kernel = explain.push_detail(
        RuleRef::builtin("kernel.schedule-refinement")?,
        vec![kernel],
        check(
            ExplainStage::KernelRefinement,
            "kernel.exact-refinement",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(target),
    )?;
    let program = explain.subject(
        SubjectKind::KernelProgram,
        "alternative:materialized-serial-sum.v1/program",
    )?;
    let program = explain.push_detail(
        RuleRef::builtin("program.two-stage-materialized")?,
        vec![program],
        check(
            ExplainStage::ProgramVerification,
            "program.verified",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(kernel),
    )?;
    let artifact = explain.subject(
        SubjectKind::ArtifactPlan,
        "alternative:materialized-serial-sum.v1/artifact",
    )?;
    explain
        .push_detail(
            RuleRef::builtin("artifact.neutral-construction-plan")?,
            vec![artifact],
            check(
                ExplainStage::ArtifactPlanning,
                "artifact.plan-verified",
                EvidenceBasis::CheckedInvariant,
            )?,
            optional_cause(program),
        )
        .map_err(Into::into)
}

#[allow(
    clippy::too_many_lines,
    reason = "keeps one fused-alternative transaction and its explain causes together"
)]
fn consider_fused_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    alternatives: &mut Vec<ProgramAlternative>,
    explain: &mut ExplainWriter,
    root: Option<ExplainRecordId>,
    alternative_causes: &mut Vec<(&'static str, Option<ExplainRecordId>)>,
    target_rejection: &mut Option<PhysicalError>,
) -> Result<(), CompileError> {
    match enumerate_candidates(verified) {
        Err(CandidateError::Budget { limit, actual }) => {
            let subject = explain.subject(SubjectKind::Candidate, "bounded-recognizer")?;
            explain.push_detail(
                RuleRef::builtin("fusion.candidates")?,
                vec![subject],
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::CandidateEnumeration,
                    resource: ResourceKey::new("fusion-candidates")?,
                    limit: u64::from(limit),
                    actual: u64::try_from(actual).unwrap_or(u64::MAX),
                },
                optional_cause(root),
            )?;
        }
        Err(error @ CandidateError::Invalid { .. }) => {
            return Err(CompileError::InvalidCompilerOutput(
                CompilerOutputError::Candidate(error),
            ));
        }
        Ok(candidates) => {
            let mut fused_candidate_record = None;
            for candidate in &candidates {
                let subject = explain.subject(SubjectKind::Candidate, &candidate.stable_id)?;
                let record = explain.push_detail(
                    RuleRef::builtin("fusion.candidate.legal")?,
                    vec![subject],
                    check(
                        ExplainStage::CandidateEnumeration,
                        "candidate.legal",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    optional_cause(root),
                )?;
                if candidate.kind == CandidateKind::FusedSerialSum {
                    fused_candidate_record = record;
                }
            }
            let Some(fused_candidate) = candidates
                .iter()
                .find(|candidate| candidate.kind == CandidateKind::FusedSerialSum)
            else {
                return Err(CompileError::InvalidCompilerOutput(
                    CompilerOutputError::Candidate(CandidateError::Invalid {
                        candidate: "bounded-recognizer".to_owned(),
                        rule: "missing-fused-candidate",
                    }),
                ));
            };
            if let Some(provider) = verified.capabilities().fused_serial_sum {
                let proof = prove_fused_numerics(verified, fused_candidate).map_err(|error| {
                    CompileError::InvalidCompilerOutput(CompilerOutputError::Candidate(error))
                })?;
                let candidate_subject =
                    explain.subject(SubjectKind::Candidate, &fused_candidate.stable_id)?;
                let proof_record = explain.push_detail(
                    RuleRef::provided(
                        "fusion.strict-f32-equivalence",
                        1,
                        ProviderRef::lowering(provider)?,
                    )?,
                    vec![candidate_subject],
                    check(
                        ExplainStage::NumericalLegality,
                        "fusion.strict-f32-equivalence",
                        EvidenceBasis::SoundProof(VerifiedEvidenceRef::from_fusion_numerical(
                            &proof,
                        )),
                    )?,
                    optional_cause(fused_candidate_record),
                )?;
                match build_fused_scheduled_region(verified) {
                    Err(error @ PhysicalError::Target { .. }) => {
                        record_target_rejection(
                            explain,
                            &error,
                            "alternative:fused-serial-sum.v1",
                            proof_record.or(root),
                        )?;
                        target_rejection.get_or_insert(error);
                    }
                    Err(error) => return Err(error.into()),
                    Ok(fused_region) => {
                        let fused_kernel = lower_structured_kernel(&fused_region)?;
                        let fused_program = build_fused_kernel_program(verified, &fused_region)?;
                        assert_kernels_match_program(
                            verified,
                            std::slice::from_ref(&fused_region),
                            &fused_program,
                            std::slice::from_ref(&fused_kernel),
                        )?;
                        let fused_artifact = build_artifact_plan(
                            semantic,
                            verified,
                            std::slice::from_ref(&fused_region),
                            std::slice::from_ref(&fused_kernel),
                            &fused_program,
                            vec![provider],
                        )?;
                        let cause =
                            record_fused_refinement(explain, verified, proof_record.or(root))?;
                        alternative_causes.push(("alternative:fused-serial-sum.v1", cause));
                        alternatives.push(fused_alternative(
                            fused_region,
                            fused_kernel,
                            fused_program,
                            fused_artifact,
                            proof,
                        ));
                    }
                }
            } else {
                let subject =
                    explain.subject(SubjectKind::Candidate, &fused_candidate.stable_id)?;
                explain.push_detail(
                    RuleRef::builtin("fusion.capability-resolution")?,
                    vec![subject],
                    ExplainEvent::DeferredCapability {
                        predicate: PredicateKey::new("fusion.provider-available")?,
                        reason: ReasonCode::new("provider-unavailable")?,
                    },
                    optional_cause(fused_candidate_record),
                )?;
            }
        }
    }
    Ok(())
}

fn record_target_rejection(
    explain: &mut ExplainWriter,
    error: &PhysicalError,
    alternative: &str,
    cause: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, CompileError> {
    let PhysicalError::Target {
        rule,
        region,
        required,
        available,
    } = error
    else {
        unreachable!("target rejection records require a target-feasibility error")
    };
    let subject = explain.subject(
        SubjectKind::Region,
        format!("{alternative}/region:{}", region.0),
    )?;
    explain
        .push_detail(
            RuleRef::builtin(format!("target.{rule}"))?,
            vec![subject],
            ExplainEvent::Feasibility {
                predicate: PredicateKey::new(*rule)?,
                outcome: crate::explain::FeasibilityOutcome::Rejected(ReasonCode::new(
                    "target-infeasible",
                )?),
                required: target_quantity(rule, *required)?,
                available: target_quantity(rule, *available)?,
            },
            optional_cause(cause),
        )
        .map_err(Into::into)
}

fn target_quantity(rule: &str, value: u64) -> Result<Quantity, ExplainError> {
    match rule {
        "grid-axis" | "threads-per-workgroup" => Ok(Quantity::Threads(value)),
        "buffer-bindings" => Ok(Quantity::Bindings(value)),
        "capability" => Ok(Quantity::Count(value)),
        _ => Err(ExplainError::UnknownQuantityUnit),
    }
}

fn record_fused_refinement(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    root: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, CompileError> {
    let schedule = explain.subject(
        SubjectKind::Schedule,
        "alternative:fused-serial-sum.v1/schedule",
    )?;
    let schedule = explain.push_detail(
        RuleRef::builtin("schedule.fused-serial-sum")?,
        vec![schedule],
        check(
            ExplainStage::IntrinsicScheduling,
            "schedule.intrinsic-valid",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(root),
    )?;
    let target = explain.subject(SubjectKind::Target, request.target_profile().key)?;
    let target = explain.push_detail(
        RuleRef::builtin("target.fused-serial-sum")?,
        vec![target],
        ExplainEvent::Feasibility {
            predicate: PredicateKey::new("grid-axis")?,
            outcome: crate::explain::FeasibilityOutcome::Admitted,
            required: Quantity::Threads(request.serial_sum().output_elements),
            available: Quantity::Threads(request.target_profile().max_threads_per_grid_axis),
        },
        optional_cause(schedule),
    )?;
    let kernel = explain.subject(
        SubjectKind::Kernel,
        "alternative:fused-serial-sum.v1/kernel",
    )?;
    let kernel = explain.push_detail(
        RuleRef::builtin("kernel.fused-schedule-refinement")?,
        vec![kernel],
        check(
            ExplainStage::KernelRefinement,
            "kernel.exact-refinement",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(target),
    )?;
    let program = explain.subject(
        SubjectKind::KernelProgram,
        "alternative:fused-serial-sum.v1/program",
    )?;
    let program = explain.push_detail(
        RuleRef::builtin("program.fused-serial-sum")?,
        vec![program],
        check(
            ExplainStage::ProgramVerification,
            "program.verified",
            EvidenceBasis::CheckedInvariant,
        )?,
        optional_cause(kernel),
    )?;
    let artifact = explain.subject(
        SubjectKind::ArtifactPlan,
        "alternative:fused-serial-sum.v1/artifact",
    )?;
    explain
        .push_detail(
            RuleRef::builtin("artifact.fused-construction-plan")?,
            vec![artifact],
            check(
                ExplainStage::ArtifactPlanning,
                "artifact.plan-verified",
                EvidenceBasis::CheckedInvariant,
            )?,
            optional_cause(program),
        )
        .map_err(Into::into)
}

fn fused_alternative(
    region: VerifiedScheduledRegion,
    kernel: VerifiedStructuredKernel,
    program: KernelProgram,
    artifact_plan: ArtifactConstructionPlan,
    proof: FusionNumericalProof,
) -> ProgramAlternative {
    ProgramAlternative {
        stable_id: "alternative:fused-serial-sum.v1",
        kind: ProgramAlternativeKind::Fused,
        scheduled_regions: vec![region],
        kernels: vec![kernel],
        program,
        artifact_plan,
        structural_cost: StructuralCost {
            model_key: STRUCTURAL_COST_MODEL_KEY,
            dispatch_count: 1,
            temporary_allocation_count: 0,
            materialized_bytes: 0,
            intermediate_global_reads: 0,
            intermediate_global_writes: 0,
        },
        equivalence: EquivalenceEvidence::Fused(Box::new(proof)),
    }
}

fn record_cost_and_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative_id: &str,
    causes: &[(&'static str, Option<ExplainRecordId>)],
    explain: &mut ExplainWriter,
) -> Result<(), CompileError> {
    for alternative in alternatives {
        let subject = explain.subject(SubjectKind::Alternative, alternative.stable_id)?;
        let cost = alternative.structural_cost;
        let terms = vec![
            CostTerm::new(
                "dispatch-count",
                Quantity::Count(u64::from(cost.dispatch_count)),
            )?,
            CostTerm::new(
                "temporary-allocation-count",
                Quantity::Count(u64::from(cost.temporary_allocation_count)),
            )?,
            CostTerm::new(
                "materialized-bytes",
                Quantity::Bytes(cost.materialized_bytes),
            )?,
            CostTerm::new(
                "intermediate-global-reads",
                Quantity::Bytes(cost.intermediate_global_reads),
            )?,
            CostTerm::new(
                "intermediate-global-writes",
                Quantity::Bytes(cost.intermediate_global_writes),
            )?,
        ];
        let cause = causes
            .iter()
            .find_map(|(id, cause)| (*id == alternative.stable_id).then_some(*cause))
            .flatten();
        let cost_record = explain.push_detail(
            RuleRef::builtin(STRUCTURAL_COST_MODEL_KEY)?,
            vec![subject.clone()],
            ExplainEvent::CostAssessment {
                model: CostModelKey::new(STRUCTURAL_COST_MODEL_KEY)?,
                basis: EvidenceBasis::CheckedInvariant,
                terms,
                disposition: CostDisposition::Retained,
            },
            optional_cause(cause),
        )?;
        let outcome = if alternative.stable_id == selected_alternative_id {
            SelectionOutcome::Selected
        } else if alternatives
            .iter()
            .find(|item| item.stable_id == selected_alternative_id)
            .is_some_and(|selected| {
                structurally_dominates(selected.structural_cost, alternative.structural_cost)
            })
        {
            SelectionOutcome::Dominated
        } else {
            SelectionOutcome::HigherCost
        };
        explain.push_terminal(
            RuleRef::builtin(SELECTION_POLICY_KEY)?,
            vec![subject],
            ExplainEvent::Selection {
                policy: SelectionPolicyKey::new(SELECTION_POLICY_KEY)?,
                outcome,
            },
            optional_cause(cost_record),
        )?;
    }
    Ok(())
}

fn select_structural_pareto(
    alternatives: &[ProgramAlternative],
) -> Result<&'static str, CompileError> {
    let Some(first) = alternatives.first() else {
        return Err(CompileError::InvalidCompilerOutput(
            CompilerOutputError::Program(ProgramError::Structure {
                rule: "portfolio-empty",
            }),
        ));
    };
    let mut selected = first;
    for candidate in alternatives.iter().skip(1) {
        if structurally_dominates(candidate.structural_cost, selected.structural_cost) {
            selected = candidate;
        }
    }
    Ok(selected.stable_id)
}

fn verify_portfolio(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    alternatives: &[ProgramAlternative],
    selected_id: &str,
) -> Result<(), CompileError> {
    if alternatives.is_empty()
        || alternatives
            .iter()
            .map(|alternative| alternative.stable_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != alternatives.len()
    {
        return Err(ProgramError::Structure {
            rule: "portfolio-identity",
        }
        .into());
    }
    for alternative in alternatives {
        verify_alternative(semantic, request, alternative)?;
    }
    let recomputed = select_structural_pareto(alternatives)?;
    if selected_id != recomputed
        || !alternatives
            .iter()
            .any(|item| item.stable_id == selected_id)
    {
        return Err(ProgramError::Structure {
            rule: "portfolio-selection",
        }
        .into());
    }
    Ok(())
}

struct ExpectedAlternative {
    stable_id: &'static str,
    cost: StructuralCost,
    scheduled: Vec<VerifiedScheduledRegion>,
    kernels: Vec<VerifiedStructuredKernel>,
    program: KernelProgram,
    artifact: ArtifactConstructionPlan,
}

fn rederive_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    kind: ProgramAlternativeKind,
) -> Result<ExpectedAlternative, CompileError> {
    let (stable_id, cost) = match kind {
        ProgramAlternativeKind::Materialized => {
            let materialized_bytes = request.serial_sum().input_elements.checked_mul(4).ok_or(
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-cost-overflow",
                    },
                )),
            )?;
            (
                "alternative:materialized-serial-sum.v1",
                StructuralCost {
                    model_key: STRUCTURAL_COST_MODEL_KEY,
                    dispatch_count: 2,
                    temporary_allocation_count: 1,
                    materialized_bytes,
                    intermediate_global_reads: materialized_bytes,
                    intermediate_global_writes: materialized_bytes,
                },
            )
        }
        ProgramAlternativeKind::Fused => (
            "alternative:fused-serial-sum.v1",
            StructuralCost {
                model_key: STRUCTURAL_COST_MODEL_KEY,
                dispatch_count: 1,
                temporary_allocation_count: 0,
                materialized_bytes: 0,
                intermediate_global_reads: 0,
                intermediate_global_writes: 0,
            },
        ),
    };
    let scheduled = match kind {
        ProgramAlternativeKind::Materialized => build_scheduled_regions(request)?,
        ProgramAlternativeKind::Fused => vec![build_fused_scheduled_region(request)?],
    };
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()?;
    let program = match kind {
        ProgramAlternativeKind::Materialized => build_kernel_program(request, &scheduled)?,
        ProgramAlternativeKind::Fused => build_fused_kernel_program(request, &scheduled[0])?,
    };
    let providers = match kind {
        ProgramAlternativeKind::Materialized => {
            vec![request.capabilities().materialized_serial_sum]
        }
        ProgramAlternativeKind::Fused => request
            .capabilities()
            .fused_serial_sum
            .into_iter()
            .collect(),
    };
    let artifact =
        build_artifact_plan(semantic, request, &scheduled, &kernels, &program, providers)?;
    Ok(ExpectedAlternative {
        stable_id,
        cost,
        scheduled,
        kernels,
        program,
        artifact,
    })
}

fn verify_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
) -> Result<(), CompileError> {
    let expected = rederive_alternative(semantic, request, alternative.kind)?;
    if alternative.stable_id != expected.stable_id || alternative.structural_cost != expected.cost {
        return Err(ProgramError::Structure {
            rule: "portfolio-cost-or-identity",
        }
        .into());
    }
    if alternative.scheduled_regions != expected.scheduled {
        return Err(ProgramError::Structure {
            rule: "portfolio-schedule-binding",
        }
        .into());
    }
    if alternative.kernels != expected.kernels {
        return Err(ProgramError::Structure {
            rule: "portfolio-kernel-binding",
        }
        .into());
    }
    if alternative.program != expected.program {
        return Err(ProgramError::Structure {
            rule: "portfolio-program-binding",
        }
        .into());
    }
    if alternative.artifact_plan != expected.artifact {
        return Err(ProgramError::Structure {
            rule: "portfolio-artifact-receipt",
        }
        .into());
    }
    verify_artifact_plan(
        &alternative.artifact_plan,
        semantic,
        request,
        &expected.scheduled,
        &expected.kernels,
        &expected.program,
        expected.artifact.lowering_providers().to_vec(),
    )?;
    verify_equivalence(request, alternative)
}

fn verify_equivalence(
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
) -> Result<(), CompileError> {
    match &alternative.equivalence {
        EquivalenceEvidence::MaterializedReference
            if alternative.kind == ProgramAlternativeKind::Materialized => {}
        EquivalenceEvidence::Fused(proof) if alternative.kind == ProgramAlternativeKind::Fused => {
            let candidates = enumerate_candidates(request).map_err(|error| {
                CompileError::InvalidCompilerOutput(CompilerOutputError::Candidate(error))
            })?;
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.kind == CandidateKind::FusedSerialSum)
                .ok_or({
                    CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                        ProgramError::Structure {
                            rule: "portfolio-fused-candidate",
                        },
                    ))
                })?;
            verify_fused_numerics(request, candidate, proof).map_err(|error| {
                CompileError::InvalidCompilerOutput(CompilerOutputError::Candidate(error))
            })?;
            if alternative.scheduled_regions.len() != 1
                || alternative.scheduled_regions[0]
                    .region()
                    .index
                    .semantic_members
                    != candidate.members.as_slice()
            {
                return Err(ProgramError::Structure {
                    rule: "portfolio-candidate-schedule-binding",
                }
                .into());
            }
        }
        _ => {
            return Err(ProgramError::Structure {
                rule: "portfolio-equivalence",
            }
            .into());
        }
    }
    Ok(())
}

const fn structurally_dominates(candidate: StructuralCost, incumbent: StructuralCost) -> bool {
    let no_worse = candidate.dispatch_count <= incumbent.dispatch_count
        && candidate.temporary_allocation_count <= incumbent.temporary_allocation_count
        && candidate.materialized_bytes <= incumbent.materialized_bytes
        && candidate.intermediate_global_reads <= incumbent.intermediate_global_reads
        && candidate.intermediate_global_writes <= incumbent.intermediate_global_writes;
    let strictly_better = candidate.dispatch_count < incumbent.dispatch_count
        || candidate.temporary_allocation_count < incumbent.temporary_allocation_count
        || candidate.materialized_bytes < incumbent.materialized_bytes
        || candidate.intermediate_global_reads < incumbent.intermediate_global_reads
        || candidate.intermediate_global_writes < incumbent.intermediate_global_writes;
    no_worse && strictly_better
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explain::ExplainDisposition;
    use crate::physical::{ContributorOrder, RegionId, StructuredBody, TensorRole};
    use crate::program::{DependencyReason, MaterializedValueId, ValueRole};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_reference::{
        FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor,
        TensorPayloadView,
    };

    fn semantic(reverse_constants: bool) -> SemanticProgram {
        semantic_case(
            Shape::from_dims([2, 3]),
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            reverse_constants,
        )
    }

    fn semantic_case(
        shape: Shape,
        scale_bits: u32,
        bias_bits: u32,
        reverse_constants: bool,
    ) -> SemanticProgram {
        semantic_case_with_axis(
            shape,
            scale_bits,
            bias_bits,
            reverse_constants,
            Axis::new(1),
        )
    }

    fn semantic_case_with_axis(
        shape: Shape,
        scale_bits: u32,
        bias_bits: u32,
        reverse_constants: bool,
        reduction_axis: Axis,
    ) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let (scale, bias) = if reverse_constants {
            let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
            let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
            (scale, bias)
        } else {
            let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
            let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
            (scale, bias)
        };
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [reduction_axis]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn interpret_fused(kernel: &VerifiedStructuredKernel, input: &[f32]) -> Vec<f32> {
        match &kernel.kernel().body {
            StructuredBody::FusedEmptyReduction {
                output_count,
                identity_bits,
                ..
            } => vec![f32::from_bits(*identity_bits); usize::try_from(*output_count).unwrap()],
            StructuredBody::FusedNonEmptySerialReduction {
                output_count,
                contributor_count,
                scale_bits,
                bias_bits,
                canonical_nan_bits,
                contraction,
                prologue_operations,
                ..
            } => {
                assert!(!contraction);
                assert_eq!(
                    prologue_operations,
                    &[
                        crate::physical::BinaryF32::Multiply,
                        crate::physical::BinaryF32::Add,
                    ]
                );
                let scale = f32::from_bits(*scale_bits);
                let bias = f32::from_bits(*bias_bits);
                let canonicalize = |value: f32| {
                    if value.is_nan() {
                        f32::from_bits(*canonical_nan_bits)
                    } else {
                        value
                    }
                };
                let contributors = usize::try_from(*contributor_count).unwrap();
                let outputs = usize::try_from(*output_count).unwrap();
                assert_eq!(input.len(), outputs * contributors);
                input
                    .chunks_exact(contributors)
                    .map(|chunk| {
                        let mut mapped = chunk.iter().map(|value| {
                            let product = canonicalize(*value * scale);
                            canonicalize(product + bias)
                        });
                        let first = mapped.next().expect("non-empty fused reduction");
                        canonicalize(mapped.fold(first, |sum, value| canonicalize(sum + value)))
                    })
                    .collect()
            }
            _ => panic!("expected fused structured body"),
        }
    }

    fn assert_fused_matches_reference(
        shape: Shape,
        values: Vec<f32>,
        scale_bits: u32,
        bias_bits: u32,
    ) {
        let semantic = semantic_case(shape.clone(), scale_bits, bias_bits, false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let fused = &product.targets[0].portfolio.alternatives[1];
        let actual = interpret_fused(&fused.kernels[0], &values);
        let key = InputKey::new("input").unwrap();
        let tensor = Tensor::dense(
            F32::resolved_type(),
            shape,
            values
                .into_iter()
                .map(|value| {
                    ReferenceElement::from_float_bits(
                        value.to_bits().to_be_bytes(),
                        FloatBitOrder::MostSignificantByteFirst,
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap();
        let expected = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            match expected[0].payload() {
                TensorPayloadView::Dense(elements) => elements
                    .iter()
                    .map(|element| {
                        u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap())
                    })
                    .collect::<Vec<_>>(),
                _ => panic!("expected dense f32 reference output"),
            }
        );
    }

    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "keeps the exact explain snapshot beside the end-to-end product invariants"
    )]
    fn product_is_deterministic_and_preserves_the_materialized_boundary() {
        let first = semantic(false);
        let second = semantic(true);
        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        let first = compile(CompilationRequest::governed(&first)).unwrap();
        let second = compile(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(first, second);
        let first = &first.targets[0];
        assert_eq!(
            first.explain.render(),
            concat!(
                "tiler-explain-v1\n",
                "0 request-verification admitted rule=compile.request.general-boundary@1 provider=tiler.compiler@1 subject=semantic-program:semantic-program event=check:compile.request.verified:proven:checked-invariant causes=-\n",
                "1 normalization admitted rule=normalize.serial-sum.v1@1 provider=tiler.compiler@1 subject=normalization:normalization:serial-sum event=check:normalize.semantic-equivalence:proven:checked-invariant causes=0\n",
                "2 region-formation admitted rule=compile.region.pointwise@1 provider=tiler.prototype.materialized-serial-sum@1 subject=region:alternative:materialized-serial-sum.v1/region:0 event=check:region.semantic-coverage:proven:checked-invariant causes=1\n",
                "3 region-formation admitted rule=compile.region.strict-sum@1 provider=tiler.prototype.materialized-serial-sum@1 subject=region:alternative:materialized-serial-sum.v1/region:1 event=check:region.semantic-coverage:proven:checked-invariant causes=1\n",
                "4 region-formation admitted rule=compile.boundary.materialized@1 provider=tiler.compiler@1 subject=boundary:pointwise-to-sum event=check:boundary.materialized:proven:checked-invariant causes=2,3\n",
                "5 intrinsic-scheduling admitted rule=schedule.coverage-and-ownership@1 provider=tiler.compiler@1 subject=schedule:alternative:materialized-serial-sum.v1/schedule event=check:schedule.intrinsic-valid:proven:checked-invariant causes=4\n",
                "6 target-feasibility admitted rule=target.prototype-target-neutral-baseline.v1@1 provider=tiler.compiler@1 subject=target:tiler.prototype-target-neutral-baseline.v1 event=feasibility:grid-axis:admitted:threads=6:65535 causes=5\n",
                "7 kernel-refinement admitted rule=kernel.schedule-refinement@1 provider=tiler.compiler@1 subject=kernel:alternative:materialized-serial-sum.v1/kernels event=check:kernel.exact-refinement:proven:checked-invariant causes=6\n",
                "8 program-verification admitted rule=program.two-stage-materialized@1 provider=tiler.compiler@1 subject=kernel-program:alternative:materialized-serial-sum.v1/program event=check:program.verified:proven:checked-invariant causes=7\n",
                "9 artifact-planning admitted rule=artifact.neutral-construction-plan@1 provider=tiler.compiler@1 subject=artifact-plan:alternative:materialized-serial-sum.v1/artifact event=check:artifact.plan-verified:proven:checked-invariant causes=8\n",
                "10 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:scale-constant event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "11 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:multiply event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "12 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:bias-constant event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "13 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:add event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "14 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:strict-sum event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "15 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:scale-constant+multiply+bias-constant+add event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "16 candidate-enumeration admitted rule=fusion.candidate.legal@1 provider=tiler.compiler@1 subject=candidate:candidate:scale-constant+multiply+bias-constant+add+strict-sum event=check:candidate.legal:proven:checked-invariant causes=1\n",
                "17 numerical-legality admitted rule=fusion.strict-f32-equivalence@1 provider=tiler.prototype.fused-serial-sum@1 subject=candidate:candidate:scale-constant+multiply+bias-constant+add+strict-sum event=check:fusion.strict-f32-equivalence:proven:sound-proof causes=16\n",
                "18 intrinsic-scheduling admitted rule=schedule.fused-serial-sum@1 provider=tiler.compiler@1 subject=schedule:alternative:fused-serial-sum.v1/schedule event=check:schedule.intrinsic-valid:proven:checked-invariant causes=17\n",
                "19 target-feasibility admitted rule=target.fused-serial-sum@1 provider=tiler.compiler@1 subject=target:tiler.prototype-target-neutral-baseline.v1 event=feasibility:grid-axis:admitted:threads=2:65535 causes=18\n",
                "20 kernel-refinement admitted rule=kernel.fused-schedule-refinement@1 provider=tiler.compiler@1 subject=kernel:alternative:fused-serial-sum.v1/kernel event=check:kernel.exact-refinement:proven:checked-invariant causes=19\n",
                "21 program-verification admitted rule=program.fused-serial-sum@1 provider=tiler.compiler@1 subject=kernel-program:alternative:fused-serial-sum.v1/program event=check:program.verified:proven:checked-invariant causes=20\n",
                "22 artifact-planning admitted rule=artifact.fused-construction-plan@1 provider=tiler.compiler@1 subject=artifact-plan:alternative:fused-serial-sum.v1/artifact event=check:artifact.plan-verified:proven:checked-invariant causes=21\n",
                "23 costing retained rule=tiler.cost.structural.v1@1 provider=tiler.compiler@1 subject=alternative:alternative:materialized-serial-sum.v1 event=cost:tiler.cost.structural.v1:checked-invariant:retained:dispatch-count:count=2,temporary-allocation-count:count=1,materialized-bytes:bytes=24,intermediate-global-reads:bytes=24,intermediate-global-writes:bytes=24 causes=9\n",
                "24 selection dominance-pruned rule=tiler.selection.structural-pareto.v1@1 provider=tiler.compiler@1 subject=alternative:alternative:materialized-serial-sum.v1 event=selection:tiler.selection.structural-pareto.v1:dominated causes=23\n",
                "25 costing retained rule=tiler.cost.structural.v1@1 provider=tiler.compiler@1 subject=alternative:alternative:fused-serial-sum.v1 event=cost:tiler.cost.structural.v1:checked-invariant:retained:dispatch-count:count=1,temporary-allocation-count:count=0,materialized-bytes:bytes=0,intermediate-global-reads:bytes=0,intermediate-global-writes:bytes=0 causes=22\n",
                "26 selection selected rule=tiler.selection.structural-pareto.v1@1 provider=tiler.compiler@1 subject=alternative:alternative:fused-serial-sum.v1 event=selection:tiler.selection.structural-pareto.v1:selected causes=25\n",
            )
        );
        assert_eq!(first.portfolio.alternatives.len(), 2);
        assert_eq!(
            first.portfolio.selection.selected_alternative_id,
            "alternative:fused-serial-sum.v1"
        );
        let materialized = &first.portfolio.alternatives[0];
        let fused = &first.portfolio.alternatives[1];
        assert_eq!(materialized.program.stages().len(), 2);
        assert_eq!(
            materialized.program.buffer_plan().values[1].role,
            ValueRole::Temporary
        );
        assert_eq!(
            materialized.program.dependencies()[0].reason,
            DependencyReason::Data(MaterializedValueId(1))
        );
        assert_eq!(
            materialized.kernels[0].kernel().buffers[1].tensor,
            TensorRole::Intermediate
        );
        assert_eq!(
            materialized.kernels[1].kernel().buffers[0].tensor,
            TensorRole::Intermediate
        );
        assert!(matches!(
            materialized.kernels[1].kernel().body,
            StructuredBody::NonEmptySerialReduction {
                order: ContributorOrder::OriginalAxisLexicographic,
                loop_start: 1,
                loop_end: 3,
                ..
            }
        ));
        assert_eq!(fused.program.stages().len(), 1);
        assert_eq!(fused.program.buffer_plan().values.len(), 2);
        assert_eq!(
            materialized.structural_cost,
            StructuralCost {
                model_key: STRUCTURAL_COST_MODEL_KEY,
                dispatch_count: 2,
                temporary_allocation_count: 1,
                materialized_bytes: 24,
                intermediate_global_reads: 24,
                intermediate_global_writes: 24,
            }
        );
        assert_eq!(
            fused.structural_cost,
            StructuralCost {
                model_key: STRUCTURAL_COST_MODEL_KEY,
                dispatch_count: 1,
                temporary_allocation_count: 0,
                materialized_bytes: 0,
                intermediate_global_reads: 0,
                intermediate_global_writes: 0,
            }
        );
        assert_eq!(
            materialized.artifact_plan.lowering_providers(),
            [crate::request::CompilerCapabilitySnapshot::governed().materialized_serial_sum]
        );
        assert_eq!(
            fused.artifact_plan.lowering_providers(),
            [crate::request::CompilerCapabilitySnapshot::governed()
                .fused_serial_sum
                .unwrap()]
        );
        assert!(matches!(
            fused.kernels[0].kernel().body,
            StructuredBody::FusedNonEmptySerialReduction {
                contributor_count: 3,
                loop_start: 1,
                loop_end: 3,
                ..
            }
        ));
        assert!(first.explain.records().iter().any(|record| {
            record.rule().key().as_str() == "compile.boundary.materialized"
                && record.event().disposition() == ExplainDisposition::Admitted
        }));
    }

    #[test]
    fn valid_but_unsupported_program_has_a_capability_failure() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let semantic = builder.build().unwrap();
        let error = compile(CompilationRequest::governed(&semantic)).unwrap_err();
        assert_eq!(
            error,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "signature",
            })
        );
        assert_eq!(
            error.to_string(),
            "compile.unsupported.strategy.signature: no installed capability can compile this valid semantic program"
        );
    }

    #[test]
    fn budget_exhaustion_is_not_reported_as_unsupported() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.budgets.semantic_operations = 4;
        let error = compile(request).unwrap_err();
        assert_eq!(
            error,
            CompileError::BudgetExhausted(RequestError::BudgetExceeded {
                resource: "semantic-operations",
                limit: 4,
                actual: 5,
            })
        );
    }

    #[test]
    fn malformed_request_is_not_reported_as_missing_capability() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.target_profiles.clear();
        assert_eq!(
            compile(request),
            Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
        );
    }

    #[test]
    fn forged_same_key_target_facts_are_rejected_at_the_request_boundary() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.target_profiles[0].max_threads_per_grid_axis = 1;
        let error = compile(request).unwrap_err();
        assert_eq!(
            error,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "target",
                rule: "prototype-target-neutral-baseline-v1",
            })
        );
    }

    #[test]
    fn missing_provider_and_fusion_budget_retain_the_verified_baseline() {
        let semantic = semantic(false);
        let mut missing_provider = CompilationRequest::governed(&semantic);
        missing_provider.capabilities.fused_serial_sum = None;
        let product = compile(missing_provider).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert_eq!(
            product.targets[0]
                .portfolio
                .selection
                .selected_alternative_id,
            "alternative:materialized-serial-sum.v1"
        );
        assert!(product.targets[0].explain.records().iter().any(|record| {
            record.rule().key().as_str() == "fusion.capability-resolution"
                && record.event().disposition() == ExplainDisposition::DeferredUnsupported
        }));

        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.fusion_candidates = 6;
        let product = compile(bounded).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert!(product.targets[0].explain.records().iter().any(|record| {
            record.rule().key().as_str() == "fusion.candidates"
                && record.event().disposition() == ExplainDisposition::BudgetStopped
        }));
    }

    #[test]
    fn infeasible_baseline_does_not_suppress_a_feasible_fused_plan() {
        let semantic = semantic_case_with_axis(
            Shape::from_dims([70_000, 2]),
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            false,
            Axis::new(0),
        );

        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let target = &product.targets[0];
        assert_eq!(target.portfolio.alternatives.len(), 1);
        assert_eq!(
            target.portfolio.alternatives[0].kind,
            ProgramAlternativeKind::Fused
        );
        assert!(target.explain.records().iter().any(|record| {
            record.rule().key().as_str() == "target.grid-axis"
                && record.subjects()[0].key().as_str()
                    == "alternative:materialized-serial-sum.v1/region:0"
                && record.event().disposition() == ExplainDisposition::RejectedTarget
                && matches!(
                    record.event(),
                    ExplainEvent::Feasibility {
                        required: Quantity::Threads(140_000),
                        available: Quantity::Threads(65_535),
                        ..
                    }
                )
        }));
    }

    #[test]
    fn structural_policy_requires_pareto_dominance_instead_of_guessing_latency() {
        let incumbent = StructuralCost {
            model_key: STRUCTURAL_COST_MODEL_KEY,
            dispatch_count: 2,
            temporary_allocation_count: 0,
            materialized_bytes: 0,
            intermediate_global_reads: 0,
            intermediate_global_writes: 0,
        };
        let tradeoff = StructuralCost {
            model_key: STRUCTURAL_COST_MODEL_KEY,
            dispatch_count: 1,
            temporary_allocation_count: 1,
            materialized_bytes: 4,
            intermediate_global_reads: 4,
            intermediate_global_writes: 4,
        };
        assert!(!structurally_dominates(tradeoff, incumbent));
        assert!(structurally_dominates(
            StructuralCost {
                dispatch_count: 1,
                ..incumbent
            },
            incumbent
        ));
    }

    #[test]
    fn structured_fused_body_interpreter_matches_reference_evaluator() {
        assert_fused_matches_reference(
            Shape::from_dims([2, 3]),
            vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, 0.0],
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
        );
        assert_fused_matches_reference(
            Shape::from_dims([4, 1]),
            vec![-0.0, f32::from_bits(1), f32::INFINITY, f32::NAN],
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
        );
        assert_fused_matches_reference(
            Shape::from_dims([2, 0]),
            Vec::new(),
            f32::NAN.to_bits(),
            f32::NEG_INFINITY.to_bits(),
        );
        let contraction_input = 1.000_000_1_f32;
        let contraction_scale = 1.000_000_1_f32;
        let contraction_bias = -1.000_000_2_f32;
        assert_ne!(
            (contraction_input * contraction_scale + contraction_bias).to_bits(),
            contraction_input
                .mul_add(contraction_scale, contraction_bias)
                .to_bits(),
            "the conformance vector must distinguish separate operations from FMA"
        );
        assert_fused_matches_reference(
            Shape::from_dims([1, 2]),
            vec![contraction_input, -1.0],
            contraction_scale.to_bits(),
            contraction_bias.to_bits(),
        );
    }

    #[test]
    fn portfolio_selection_and_evidence_are_recomputed_from_exact_contents() {
        let semantic = semantic(false);
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let request = request.for_target(request.target_profiles()[0]).unwrap();
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let target = &product.targets[0];
        let alternatives = &target.portfolio.alternatives;
        let selected = target.portfolio.selection.selected_alternative_id;

        assert!(verify_portfolio(&semantic, &request, alternatives, selected).is_ok());
        assert!(verify_portfolio(&semantic, &request, &[], selected).is_err());
        assert!(verify_portfolio(&semantic, &request, alternatives, "stale-selection").is_err());

        let mut forged = alternatives.clone();
        forged[0].structural_cost.dispatch_count = 0;
        assert!(verify_portfolio(&semantic, &request, &forged, selected).is_err());
    }

    #[test]
    fn intrinsic_physical_failures_are_invalid_output_not_empty_frontiers() {
        let error = CompileError::from(PhysicalError::Intrinsic {
            rule: "forged",
            region: RegionId(0),
        });
        assert!(matches!(
            error,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
                PhysicalError::Intrinsic { .. }
            ))
        ));
    }
}
