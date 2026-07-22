use std::error::Error;
use std::fmt;

use crate::fusion::{
    CandidateError, CandidateKind, FusionNumericalProof, enumerate_candidates,
    prove_fused_numerics, verify_fused_numerics,
};
use crate::physical::{
    PhysicalError, RegionId, VerifiedScheduledRegion, VerifiedStructuredKernel,
    build_fused_scheduled_region, build_scheduled_regions, lower_structured_kernel,
};
use crate::program::{
    ArtifactConstructionPlan, KernelProgram, ProgramError, assert_kernels_match_program,
    build_artifact_plan, build_fused_kernel_program, build_kernel_program, verify_artifact_plan,
    verify_semantic_output_type,
};
use crate::request::{CompilationRequest, RequestError, verify_request};

const SELECTION_POLICY_KEY: &str = "tiler.selection.structural-pareto.v1";
const STRUCTURAL_COST_MODEL_KEY: &str = "tiler.cost.structural.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainPhase {
    RequestVerification,
    RegionFormation,
    IntrinsicSchedule,
    TargetFeasibility,
    KernelRefinement,
    ProgramVerification,
    ArtifactPlanning,
    CandidateEnumeration,
    NumericalLegality,
    PortfolioSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainOutcome {
    Accepted,
    Rejected,
    Selected,
    NotSelected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainRecord {
    pub(crate) phase: ExplainPhase,
    pub(crate) rule: &'static str,
    pub(crate) subject: ExplainSubject,
    pub(crate) outcome: ExplainOutcome,
    pub(crate) evidence: ExplainEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExplainSubject {
    SemanticProgram,
    Region(RegionId),
    Regions(Vec<RegionId>),
    Boundary(&'static str),
    Candidate(String),
    CandidateRegion { stable_id: String, region: RegionId },
    Alternative(&'static str),
    KernelProgram,
    ArtifactPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceClass {
    ValidatedInvariant,
    SoundNumericalProof,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainEvidence {
    Predicate {
        class: EvidenceClass,
    },
    Budget {
        limit: u32,
        actual: usize,
    },
    Feasibility {
        required: u64,
        available: u64,
    },
    Cost {
        model_key: &'static str,
        cost: StructuralCost,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompilationProduct {
    pub(crate) targets: Vec<TargetCompilationProduct>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetCompilationProduct {
    pub(crate) target_profile_key: &'static str,
    pub(crate) portfolio: ProgramPortfolio,
    pub(crate) explain: Vec<ExplainRecord>,
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
    let mut explain = request_explain();
    let mut alternatives = Vec::new();
    let mut target_rejection = None;
    match build_baseline_alternative(semantic, verified) {
        Ok(baseline) => {
            explain.extend(baseline_explain());
            alternatives.push(baseline);
        }
        Err(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
            error @ PhysicalError::Target { .. },
        ))) => {
            explain.push(target_rejection_record(&error, None));
            target_rejection = Some(error);
        }
        Err(error) => return Err(error),
    }
    consider_fused_alternative(
        semantic,
        verified,
        &mut alternatives,
        &mut explain,
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
    record_selection(&alternatives, selected_alternative_id, &mut explain);
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

fn request_explain() -> Vec<ExplainRecord> {
    vec![accepted(
        ExplainPhase::RequestVerification,
        "compile.request.general-boundary",
        ExplainSubject::SemanticProgram,
    )]
}

fn baseline_explain() -> Vec<ExplainRecord> {
    vec![
        accepted(
            ExplainPhase::RegionFormation,
            "compile.region.pointwise",
            ExplainSubject::Region(RegionId(0)),
        ),
        accepted(
            ExplainPhase::RegionFormation,
            "compile.region.strict-sum",
            ExplainSubject::Region(RegionId(1)),
        ),
        accepted(
            ExplainPhase::RegionFormation,
            "compile.boundary.materialized",
            ExplainSubject::Boundary("pointwise-to-sum"),
        ),
        accepted(
            ExplainPhase::IntrinsicSchedule,
            "schedule.coverage-and-ownership",
            ExplainSubject::Regions(vec![RegionId(0), RegionId(1)]),
        ),
        accepted(
            ExplainPhase::TargetFeasibility,
            "target.prototype-target-neutral-baseline.v1",
            ExplainSubject::Regions(vec![RegionId(0), RegionId(1)]),
        ),
        accepted(
            ExplainPhase::KernelRefinement,
            "kernel.schedule-refinement",
            ExplainSubject::Regions(vec![RegionId(0), RegionId(1)]),
        ),
        accepted(
            ExplainPhase::ProgramVerification,
            "program.two-stage-materialized",
            ExplainSubject::KernelProgram,
        ),
        accepted(
            ExplainPhase::ArtifactPlanning,
            "artifact.neutral-construction-plan",
            ExplainSubject::ArtifactPlan,
        ),
    ]
}

fn consider_fused_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    alternatives: &mut Vec<ProgramAlternative>,
    explain: &mut Vec<ExplainRecord>,
    target_rejection: &mut Option<PhysicalError>,
) -> Result<(), CompileError> {
    match enumerate_candidates(verified) {
        Err(CandidateError::Budget { limit, actual }) => explain.push(ExplainRecord {
            phase: ExplainPhase::CandidateEnumeration,
            rule: "fusion.candidates.budget",
            subject: ExplainSubject::Candidate("bounded-recognizer".to_owned()),
            outcome: ExplainOutcome::Rejected,
            evidence: ExplainEvidence::Budget { limit, actual },
        }),
        Err(error @ CandidateError::Invalid { .. }) => {
            return Err(CompileError::InvalidCompilerOutput(
                CompilerOutputError::Candidate(error),
            ));
        }
        Ok(candidates) => {
            for candidate in &candidates {
                explain.push(accepted(
                    ExplainPhase::CandidateEnumeration,
                    "fusion.candidate.legal",
                    ExplainSubject::Candidate(candidate.stable_id.clone()),
                ));
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
                explain.push(accepted(
                    ExplainPhase::NumericalLegality,
                    "fusion.strict-f32-equivalence",
                    ExplainSubject::Candidate(fused_candidate.stable_id.clone()),
                ));
                match build_fused_scheduled_region(verified) {
                    Err(error @ PhysicalError::Target { .. }) => {
                        explain.push(target_rejection_record(
                            &error,
                            Some(&fused_candidate.stable_id),
                        ));
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
                explain.push(rejected(
                    ExplainPhase::KernelRefinement,
                    "fusion.provider-unavailable",
                    ExplainSubject::Candidate(fused_candidate.stable_id.clone()),
                ));
            }
        }
    }
    Ok(())
}

fn target_rejection_record(error: &PhysicalError, candidate: Option<&str>) -> ExplainRecord {
    let PhysicalError::Target {
        rule,
        region,
        required,
        available,
    } = error
    else {
        unreachable!("target rejection records require a target-feasibility error")
    };
    ExplainRecord {
        phase: ExplainPhase::TargetFeasibility,
        rule,
        subject: candidate.map_or(ExplainSubject::Region(*region), |stable_id| {
            ExplainSubject::CandidateRegion {
                stable_id: stable_id.to_owned(),
                region: *region,
            }
        }),
        outcome: ExplainOutcome::Rejected,
        evidence: ExplainEvidence::Feasibility {
            required: *required,
            available: *available,
        },
    }
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

fn record_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative_id: &str,
    explain: &mut Vec<ExplainRecord>,
) {
    for alternative in alternatives {
        explain.push(ExplainRecord {
            phase: ExplainPhase::PortfolioSelection,
            rule: SELECTION_POLICY_KEY,
            subject: ExplainSubject::Alternative(alternative.stable_id),
            outcome: if alternative.stable_id == selected_alternative_id {
                ExplainOutcome::Selected
            } else {
                ExplainOutcome::NotSelected
            },
            evidence: ExplainEvidence::Cost {
                model_key: STRUCTURAL_COST_MODEL_KEY,
                cost: alternative.structural_cost,
            },
        });
    }
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

fn accepted(phase: ExplainPhase, rule: &'static str, subject: ExplainSubject) -> ExplainRecord {
    ExplainRecord {
        phase,
        rule,
        subject,
        outcome: ExplainOutcome::Accepted,
        evidence: ExplainEvidence::Predicate {
            class: if phase == ExplainPhase::NumericalLegality {
                EvidenceClass::SoundNumericalProof
            } else {
                EvidenceClass::ValidatedInvariant
            },
        },
    }
}

fn rejected(phase: ExplainPhase, rule: &'static str, subject: ExplainSubject) -> ExplainRecord {
    ExplainRecord {
        phase,
        rule,
        subject,
        outcome: ExplainOutcome::Rejected,
        evidence: ExplainEvidence::Predicate {
            class: EvidenceClass::ValidatedInvariant,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical::{ContributorOrder, StructuredBody, TensorRole};
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
        assert!(first.explain.iter().any(|record| {
            record.rule == "compile.boundary.materialized"
                && record.outcome == ExplainOutcome::Accepted
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
        assert!(product.targets[0].explain.iter().any(|record| {
            record.rule == "fusion.provider-unavailable"
                && record.outcome == ExplainOutcome::Rejected
        }));

        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.fusion_candidates = 6;
        let product = compile(bounded).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert!(product.targets[0].explain.iter().any(|record| {
            record.rule == "fusion.candidates.budget" && record.outcome == ExplainOutcome::Rejected
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
        assert!(target.explain.iter().any(|record| {
            record.rule == "grid-axis"
                && record.subject == ExplainSubject::Region(RegionId(0))
                && record.outcome == ExplainOutcome::Rejected
                && record.evidence
                    == ExplainEvidence::Feasibility {
                        required: 140_000,
                        available: 65_535,
                    }
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
