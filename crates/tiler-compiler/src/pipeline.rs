use std::error::Error;
use std::fmt;

use crate::explain::{
    CostDisposition, CostModelKey, CostTerm, EvidenceBasis, ExplainError, ExplainEvent,
    ExplainFact, ExplainLimits, ExplainRecordId, ExplainStage, ExplainWriter, FactValue,
    FailureDescriptor, MAX_TERMINAL_CAUSES, PredicateAssessment, PredicateKey, ProviderRef,
    Quantity, ReasonCode, RejectionClass, RuleRef, SelectionOutcome, SubjectKind, TerminalCause,
    VerifiedEvidenceRef, VerifiedExplainTrace,
};
use crate::fusion::{
    FusionError, FusionNumericalProof, prove_fused_numerics, verify_fused_numerics,
};
use crate::normalize::{
    NORMALIZATION_SUBJECT, NormalizationOutcome, NormalizeError, normalize_semantics,
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
use crate::region::{
    REGION_FORMATION_SUBJECT, RegionError, RegionFormationOutcome, RegionFormationRecords,
    form_region_candidates,
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
    Explained {
        source: Box<CompileError>,
        explain: VerifiedExplainTrace,
    },
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
    Region(RegionError),
    Fusion(FusionError),
    Explain(ExplainError),
    Normalization(NormalizeError),
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
            Self::InvalidCompilerOutput(CompilerOutputError::Region(error)) => error.fmt(formatter),
            Self::InvalidCompilerOutput(CompilerOutputError::Fusion(error)) => error.fmt(formatter),
            Self::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Normalization(error)) => {
                error.fmt(formatter)
            }
            Self::Explained { source, .. } => source.fmt(formatter),
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
            Self::InvalidCompilerOutput(CompilerOutputError::Region(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Fusion(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Normalization(error)) => Some(error),
            Self::Explained { source, .. } => Some(source),
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

impl From<NormalizeError> for CompileError {
    fn from(value: NormalizeError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Normalization(value))
    }
}

impl From<RegionError> for CompileError {
    fn from(value: RegionError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Region(value))
    }
}

impl From<FusionError> for CompileError {
    fn from(value: FusionError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Fusion(value))
    }
}

pub(crate) fn compile(request: CompilationRequest<'_>) -> Result<CompilationProduct, CompileError> {
    let semantic = request.program;
    let shape_environment = request.shape_environment;
    let target_profiles = request.target_profiles.clone();
    let capabilities = request.capabilities;
    let verified = verify_request(request)?;
    verify_semantic_output_type(semantic)?;
    // `NormalizeSemantics` runs after request verification and before region
    // formation. It observes only the verified program and never mutates it.
    let normalization =
        normalize_semantics(semantic, verified.budgets(), verified.numerical_contract())?;
    let Some(normalized) = normalization.normalized_program() else {
        return compile_verified(semantic, &verified, &normalization);
    };
    // A committed rewrite is a new program, so it must independently re-enter
    // the request boundary rather than inheriting the input's verification.
    // Rejection here is invalid compiler output, not an unsupported user
    // program: the input was already admitted.
    let readmitted = verify_request(CompilationRequest {
        program: normalized,
        shape_environment,
        numerical_contract: verified.numerical_contract(),
        budgets: verified.budgets(),
        target_profiles,
        capabilities,
    })
    .map_err(|_| {
        CompileError::from(NormalizeError::InvalidRewrite {
            rule: "request-readmission",
        })
    })?;
    verify_semantic_output_type(normalized)?;
    compile_verified(normalized, &readmitted, &normalization)
}

fn compile_verified(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedCompilationRequest,
    normalization: &NormalizationOutcome,
) -> Result<CompilationProduct, CompileError> {
    let targets = verified
        .target_profiles()
        .iter()
        .copied()
        .map(|target| {
            let target_request = verified.for_target(target)?;
            compile_target(semantic, &target_request, normalization)
        })
        .collect::<Result<_, _>>()?;
    Ok(CompilationProduct { targets })
}

fn compile_target(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    normalization: &NormalizationOutcome,
) -> Result<TargetCompilationProduct, CompileError> {
    let mut explain = ExplainWriter::new(verified, ExplainLimits::default())?;
    match compile_target_with_explain(semantic, verified, normalization, &mut explain) {
        Ok(portfolio) => {
            let expected_alternatives = portfolio
                .alternatives
                .iter()
                .map(|alternative| alternative.stable_id)
                .collect::<Vec<_>>();
            let explain = explain.finish_success(
                &expected_alternatives,
                portfolio.selection.selected_alternative_id,
            )?;
            Ok(TargetCompilationProduct {
                target_profile_key: verified.target_profile().key,
                portfolio,
                explain,
            })
        }
        Err(failure) => {
            let explain = explain.finish_failure(*failure.context)?;
            Err(CompileError::Explained {
                source: failure.source,
                explain,
            })
        }
    }
}

#[derive(Debug)]
struct TargetFailure {
    source: Box<CompileError>,
    context: Box<FailureDescriptor>,
}

fn target_failure(
    source: CompileError,
    stage: ExplainStage,
    reason: impl AsRef<str>,
    subject_kind: SubjectKind,
    subject_key: impl AsRef<str>,
    cause: Option<TerminalCause>,
) -> TargetFailure {
    match FailureDescriptor::new(stage, reason, subject_kind, subject_key, cause) {
        Ok(context) => TargetFailure {
            source: Box::new(source),
            context: Box::new(context),
        },
        Err(error) => TargetFailure {
            source: Box::new(CompileError::InvalidCompilerOutput(
                CompilerOutputError::Explain(error),
            )),
            context: Box::new(
                FailureDescriptor::new(
                    ExplainStage::ProgramVerification,
                    "failure-context-invalid",
                    SubjectKind::KernelProgram,
                    "compiler-explain",
                    None,
                )
                .expect("static fallback failure context is valid"),
            ),
        },
    }
}

fn target_failure_with_causes(
    source: CompileError,
    stage: ExplainStage,
    reason: impl AsRef<str>,
    subject_kind: SubjectKind,
    subject_key: impl AsRef<str>,
    causes: Vec<TerminalCause>,
) -> TargetFailure {
    match FailureDescriptor::with_causes(stage, reason, subject_kind, subject_key, causes) {
        Ok(context) => TargetFailure {
            source: Box::new(source),
            context: Box::new(context),
        },
        Err(error) => target_failure(
            CompileError::InvalidCompilerOutput(CompilerOutputError::Explain(error)),
            ExplainStage::ProgramVerification,
            "failure-context-invalid",
            SubjectKind::KernelProgram,
            "compiler-explain",
            None,
        ),
    }
}

fn record_cause(record: Option<ExplainRecordId>) -> Option<TerminalCause> {
    record.map(TerminalCause::from_record)
}

fn explain_step<T>(
    result: Result<T, CompileError>,
    stage: ExplainStage,
    subject_kind: SubjectKind,
    subject_key: impl AsRef<str>,
    cause: Option<TerminalCause>,
) -> Result<T, TargetFailure> {
    result.map_err(|source| {
        let reason = match &source {
            CompileError::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => {
                format!("explain-{}", explain_error_reason(error))
            }
            _ => "explain-step-source-mismatch".to_owned(),
        };
        target_failure(source, stage, reason, subject_kind, subject_key, cause)
    })
}

fn failure_at_source(
    source: CompileError,
    stage: ExplainStage,
    cause: Option<TerminalCause>,
) -> TargetFailure {
    let (reason, subject_kind, subject_key) = failure_source_details(&source);
    target_failure(source, stage, reason, subject_kind, subject_key, cause)
}

fn failure_at_source_with_causes(
    source: CompileError,
    stage: ExplainStage,
    causes: Vec<TerminalCause>,
) -> TargetFailure {
    let (reason, subject_kind, subject_key) = failure_source_details(&source);
    target_failure_with_causes(source, stage, reason, subject_kind, subject_key, causes)
}

const fn physical_error_stage(error: &PhysicalError) -> ExplainStage {
    match error {
        PhysicalError::Target { .. } => ExplainStage::TargetFeasibility,
        PhysicalError::Intrinsic { .. } | PhysicalError::ShapeProductOverflow { .. } => {
            ExplainStage::IntrinsicScheduling
        }
        PhysicalError::Refinement { .. } => ExplainStage::KernelRefinement,
    }
}

#[derive(Clone, Debug)]
struct TargetRejection {
    kind: ProgramAlternativeKind,
    error: PhysicalError,
    cause: TerminalCause,
}

#[derive(Default)]
struct TargetRejections {
    values: Vec<TargetRejection>,
}

impl TargetRejections {
    fn push(&mut self, rejection: TargetRejection) -> Result<(), TargetFailure> {
        if u32::try_from(self.values.len()).unwrap_or(u32::MAX) >= MAX_TERMINAL_CAUSES {
            return Err(target_failure(
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "target-rejection-cause-capacity",
                    },
                )),
                ExplainStage::Selection,
                "target-rejection-cause-capacity",
                SubjectKind::KernelProgram,
                "portfolio",
                None,
            ));
        }
        let insertion = self
            .values
            .binary_search_by_key(&rejection.kind, |existing| existing.kind)
            .unwrap_or_else(|insertion| insertion);
        if self
            .values
            .get(insertion)
            .is_some_and(|existing| existing.kind == rejection.kind)
        {
            return Err(target_failure(
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "duplicate-target-rejection",
                    },
                )),
                ExplainStage::Selection,
                "duplicate-target-rejection",
                SubjectKind::KernelProgram,
                "portfolio",
                None,
            ));
        }
        self.values.insert(insertion, rejection);
        Ok(())
    }

    fn into_failure(self) -> Option<TargetFailure> {
        let representative = self.values.first()?.error.clone();
        let causes = self
            .values
            .into_iter()
            .map(|rejection| rejection.cause)
            .collect();
        Some(failure_at_source_with_causes(
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(representative)),
            ExplainStage::TargetFeasibility,
            causes,
        ))
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "keeps the phase-local failure contexts beside the target compilation transaction"
)]
fn compile_target_with_explain(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    normalization: &NormalizationOutcome,
    explain: &mut ExplainWriter,
) -> Result<ProgramPortfolio, TargetFailure> {
    let request_record = (|| -> Result<_, CompileError> {
        let request_subject = explain.subject(SubjectKind::SemanticProgram, "semantic-program")?;
        Ok(explain.push_detail(
            RuleRef::builtin("compile.request.general-boundary")?,
            vec![request_subject],
            check(
                ExplainStage::RequestVerification,
                "compile.request.verified",
                EvidenceBasis::CheckedInvariant,
            )?,
            Vec::new(),
        )?)
    })()
    .map_err(|source| {
        target_failure(
            source,
            ExplainStage::RequestVerification,
            "explain-request-verification",
            SubjectKind::SemanticProgram,
            "semantic-program",
            None,
        )
    })?;
    let normalization_record = explain_step(
        normalization
            .record(explain, request_record)
            .map_err(CompileError::from),
        ExplainStage::Normalization,
        SubjectKind::Normalization,
        NORMALIZATION_SUBJECT,
        record_cause(request_record),
    )?;
    // `EnumerateRegionCandidates` runs immediately after normalization and only
    // proposes regions. Cover selection, implementation choice, index lowering,
    // physical planning, and costing all remain later authorities.
    let formation =
        form_region_candidates(semantic, verified.budgets(), verified.numerical_contract())
            .map_err(|source| {
                failure_at_source(
                    source.into(),
                    ExplainStage::RegionFormation,
                    record_cause(normalization_record),
                )
            })?;
    let region_records = explain_step(
        formation
            .record(explain, normalization_record)
            .map_err(CompileError::from),
        ExplainStage::RegionFormation,
        SubjectKind::Region,
        REGION_FORMATION_SUBJECT,
        record_cause(normalization_record),
    )?;
    let region_root = region_records.summary.or(normalization_record);
    let mut alternatives = Vec::new();
    let mut alternative_causes = Vec::new();
    let mut target_rejections = TargetRejections::default();
    match build_baseline_alternative(semantic, verified, record_cause(region_root)) {
        Ok(baseline) => {
            let cause = record_baseline_explain(explain, verified, &baseline, region_root)?;
            alternative_causes.push((baseline.stable_id, cause));
            alternatives.push(baseline);
        }
        Err(failure) => match *failure.source {
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                error @ PhysicalError::Target { .. },
            )) => {
                let cause = record_target_rejection(
                    explain,
                    &error,
                    "alternative:materialized-serial-sum.v1",
                    region_root,
                )?;
                target_rejections.push(TargetRejection {
                    kind: ProgramAlternativeKind::Materialized,
                    error,
                    cause,
                })?;
            }
            source => {
                return Err(TargetFailure {
                    source: Box::new(source),
                    context: failure.context,
                });
            }
        },
    }
    consider_fused_alternative(
        semantic,
        verified,
        &formation,
        &mut alternatives,
        explain,
        region_records,
        &mut alternative_causes,
        &mut target_rejections,
    )?;
    if alternatives.is_empty() {
        if target_rejections.values.is_empty() {
            return Err(target_failure(
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-empty-without-target-rejection",
                    },
                )),
                ExplainStage::Selection,
                "portfolio-empty-without-target-rejection",
                SubjectKind::KernelProgram,
                "portfolio",
                record_cause(region_root),
            ));
        }
        return Err(target_rejections
            .into_failure()
            .expect("nonempty target rejection set yields one failure"));
    }
    let selected_alternative_id = select_structural_pareto(&alternatives).map_err(|source| {
        target_failure(
            source,
            ExplainStage::Selection,
            "portfolio-selection",
            SubjectKind::KernelProgram,
            "portfolio",
            record_cause(region_root),
        )
    })?;
    verify_portfolio(
        semantic,
        verified,
        &alternatives,
        selected_alternative_id,
        record_cause(region_root),
    )?;
    record_cost_and_selection(
        &alternatives,
        selected_alternative_id,
        &alternative_causes,
        explain,
    )?;
    Ok(ProgramPortfolio {
        alternatives,
        selection: PortfolioSelection {
            policy_key: SELECTION_POLICY_KEY,
            selected_alternative_id,
        },
    })
}

fn failure_source_details(error: &CompileError) -> (String, SubjectKind, String) {
    match error {
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
            rule,
            region,
            ..
        }))
        | CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Target { rule, region, .. },
        )) => (
            format!("target-{rule}"),
            SubjectKind::Region,
            format!("failed-region:{}", region.0),
        ),
        CompileError::InvalidCompilerOutput(
            CompilerOutputError::Region(error)
            | CompilerOutputError::Fusion(FusionError::Region(error)),
        ) => (
            format!("region-{}-{}", error.class(), error.reason()),
            SubjectKind::Region,
            match error {
                RegionError::Structure { .. } => REGION_FORMATION_SUBJECT.to_owned(),
                RegionError::Invalid { region, .. } => region.clone(),
            },
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Fusion(
            error @ FusionError::Invalid { region, .. },
        )) => (
            format!("fusion-{}", error.reason()),
            SubjectKind::Candidate,
            region.clone(),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Intrinsic { rule, region },
        )) => (
            format!("intrinsic-{rule}"),
            SubjectKind::Region,
            format!("failed-region:{}", region.0),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Refinement { rule, region },
        )) => (
            format!("refinement-{rule}"),
            SubjectKind::Kernel,
            format!("failed-region:{}", region.0),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(error)) => {
            program_failure_details(error)
        }
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::ShapeProductOverflow { region },
        )) => (
            "shape-product-overflow".to_owned(),
            SubjectKind::Region,
            format!("failed-region:{}", region.0),
        ),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(_)) => (
            "invalid-no-feasible-physical-class".to_owned(),
            SubjectKind::KernelProgram,
            "compiler-output".to_owned(),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => (
            format!("explain-{}", explain_error_reason(error)),
            SubjectKind::KernelProgram,
            "compiler-explain".to_owned(),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Normalization(error)) => (
            format!("normalize-{}", error.reason()),
            SubjectKind::Normalization,
            NORMALIZATION_SUBJECT.to_owned(),
        ),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(error))
        | CompileError::InvalidRequest(error)
        | CompileError::UnsupportedCapability(error)
        | CompileError::BudgetExhausted(error) => request_failure_details(error),
        CompileError::Explained { .. } => (
            "nested-explained-error".to_owned(),
            SubjectKind::KernelProgram,
            "compiler-explain".to_owned(),
        ),
    }
}

fn request_failure_details(error: &RequestError) -> (String, SubjectKind, String) {
    let reason = match error {
        RequestError::UnsupportedRequestVersion => "request-version".to_owned(),
        RequestError::EmptyTargetSet => "target-set-empty".to_owned(),
        RequestError::DuplicateTargetProfile => "target-profile-duplicate".to_owned(),
        RequestError::UnverifiedTargetSelection => "target-selection-unverified".to_owned(),
        RequestError::BudgetExceeded {
            resource,
            limit,
            actual,
        } => format!("budget-{resource}-{limit}-{actual}"),
        RequestError::UnsupportedCapability { phase, rule } => {
            format!("unsupported-{phase}-{rule}")
        }
        RequestError::ShapeProductOverflow { role } => format!("shape-product-overflow-{role}"),
    };
    (
        reason,
        SubjectKind::SemanticProgram,
        "semantic-program".to_owned(),
    )
}

fn program_failure_details(error: &ProgramError) -> (String, SubjectKind, String) {
    let (reason, subject) = match error {
        ProgramError::HostExpression { rule, expression } => (
            format!("host-expression-{rule}"),
            format!("host-expression:{}", expression.index()),
        ),
        ProgramError::Structure { rule } => {
            (format!("structure-{rule}"), "kernel-program".to_owned())
        }
        ProgramError::Dependency { rule } => {
            (format!("dependency-{rule}"), "kernel-program".to_owned())
        }
        ProgramError::Storage { rule } => (format!("storage-{rule}"), "kernel-program".to_owned()),
        ProgramError::Abi { rule, stage } => {
            (format!("abi-{rule}"), format!("stage:{}", stage.index()))
        }
        ProgramError::Routing { rule } => (format!("routing-{rule}"), "kernel-program".to_owned()),
    };
    (reason, SubjectKind::KernelProgram, subject)
}

fn explain_error_reason(error: &ExplainError) -> &'static str {
    match error {
        ExplainError::InvalidKey { .. } => "invalid-key",
        ExplainError::InvalidLimits => "invalid-limits",
        ExplainError::InvalidTerminalLedger => "invalid-terminal-ledger",
        ExplainError::TerminalLedgerCapacity => "terminal-ledger-capacity",
        ExplainError::InvalidEventClass => "invalid-event-class",
        ExplainError::BoundExceeded { .. } => "bound-exceeded",
        ExplainError::EmptySubjects => "empty-subjects",
        ExplainError::CrossCompilationSubject => "cross-compilation-subject",
        ExplainError::DuplicateCause => "duplicate-cause",
        ExplainError::DuplicateSubject => "duplicate-subject",
        ExplainError::DuplicateFact => "duplicate-fact",
        ExplainError::DuplicateCostTerm => "duplicate-cost-term",
        ExplainError::CrossWriterCause => "cross-writer-cause",
        ExplainError::InvalidCause { .. } => "invalid-cause",
        ExplainError::InvalidStageEvent => "invalid-stage-event",
        ExplainError::EvidenceEscalation => "evidence-escalation",
        ExplainError::EvidenceSubjectMismatch => "evidence-subject-mismatch",
        ExplainError::ProviderAuthorityMismatch => "provider-authority-mismatch",
        ExplainError::QuantityKindMismatch => "quantity-kind-mismatch",
        ExplainError::InvalidQuantityRelation => "invalid-quantity-relation",
        ExplainError::UnknownQuantityUnit => "unknown-quantity-unit",
        ExplainError::EmptyCostEvidence => "empty-cost-evidence",
        ExplainError::TerminalCapacity => "terminal-capacity",
        ExplainError::EmptyTrace => "empty-trace",
        ExplainError::StaleIdentity => "stale-identity",
    }
}

fn build_baseline_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    cause: Option<TerminalCause>,
) -> Result<ProgramAlternative, TargetFailure> {
    let baseline_regions = build_scheduled_regions(verified).map_err(|error| {
        let stage = physical_error_stage(&error);
        failure_at_source(error.into(), stage, cause.clone())
    })?;
    let baseline_kernels = baseline_regions
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause.clone())
        })?;
    let baseline_program = build_kernel_program(verified, &baseline_regions).map_err(|error| {
        failure_at_source(
            error.into(),
            ExplainStage::ProgramVerification,
            cause.clone(),
        )
    })?;
    assert_kernels_match_program(
        verified,
        &baseline_regions,
        &baseline_program,
        &baseline_kernels,
    )
    .map_err(|error| {
        failure_at_source(
            error.into(),
            ExplainStage::ProgramVerification,
            cause.clone(),
        )
    })?;
    let baseline_artifact = build_artifact_plan(
        semantic,
        verified,
        &baseline_regions,
        &baseline_kernels,
        &baseline_program,
        vec![verified.capabilities().materialized_serial_sum],
    )
    .map_err(|error| {
        failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause.clone())
    })?;
    let input_bytes = verified
        .serial_sum()
        .input_elements
        .checked_mul(4)
        .ok_or_else(|| {
            failure_at_source(
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
                    RequestError::ShapeProductOverflow {
                        role: "input-bytes",
                    },
                )),
                ExplainStage::Costing,
                cause,
            )
        })?;
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

fn check_with_count(
    stage: ExplainStage,
    predicate: &str,
    fact: &str,
    count: usize,
) -> Result<ExplainEvent, ExplainError> {
    Ok(ExplainEvent::Check {
        stage,
        assessment: PredicateAssessment::proven(predicate, EvidenceBasis::CheckedInvariant)?
            .with_fact(ExplainFact::new(
                fact,
                FactValue::Count(u64::try_from(count).unwrap_or(u64::MAX)),
            )?)?,
        rejection: RejectionClass::IntrinsicInvalid,
    })
}

#[allow(
    clippy::too_many_arguments,
    reason = "the helper keeps each typed emitter's complete phase and subject context explicit"
)]
fn record_count_step(
    explain: &mut ExplainWriter,
    rule: &str,
    subject_kind: SubjectKind,
    subject_key: &str,
    stage: ExplainStage,
    predicate: &str,
    fact: &str,
    count: usize,
    cause: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, TargetFailure> {
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(subject_kind, subject_key)?;
            Ok(explain.push_detail(
                RuleRef::builtin(rule)?,
                vec![subject],
                check_with_count(stage, predicate, fact, count)?,
                optional_cause(cause),
            )?)
        })(),
        stage,
        subject_kind,
        subject_key,
        record_cause(cause),
    )
}

fn optional_cause(cause: Option<ExplainRecordId>) -> Vec<ExplainRecordId> {
    cause.into_iter().collect()
}

fn record_baseline_explain(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    root: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, TargetFailure> {
    let mut boundary_causes = Vec::new();
    for scheduled in &alternative.scheduled_regions {
        let region_id = scheduled.region().index.id.0;
        let key = format!("{}/region:{region_id}", alternative.stable_id);
        let record = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Region, &key)?;
                Ok(explain.push_detail(
                    RuleRef::provided(
                        "compile.region.verified",
                        1,
                        ProviderRef::lowering(request.capabilities().materialized_serial_sum)?,
                    )?,
                    vec![subject],
                    check(
                        ExplainStage::RegionFormation,
                        "region.semantic-coverage",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    optional_cause(root),
                )?)
            })(),
            ExplainStage::RegionFormation,
            SubjectKind::Region,
            &key,
            record_cause(root),
        )?;
        boundary_causes.extend(optional_cause(record));
    }
    let key = format!("{}/materialized-boundary", alternative.stable_id);
    let boundary = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Boundary, &key)?;
            Ok(explain.push_detail(
                RuleRef::builtin("compile.boundary.materialized")?,
                vec![subject],
                check_with_count(
                    ExplainStage::RegionFormation,
                    "boundary.materialized",
                    "dependency-count",
                    alternative.program.dependencies().len(),
                )?,
                boundary_causes,
            )?)
        })(),
        ExplainStage::RegionFormation,
        SubjectKind::Boundary,
        &key,
        record_cause(root),
    )?;
    record_baseline_refinement(explain, request, alternative, boundary)
}

fn record_baseline_refinement(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    boundary: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, TargetFailure> {
    let key = format!("{}/schedules", alternative.stable_id);
    let schedule = record_count_step(
        explain,
        "schedule.coverage-and-ownership",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "schedule.intrinsic-valid",
        "schedule-count",
        alternative.scheduled_regions.len(),
        boundary,
    )?;
    let target = record_target_admissions(explain, request, alternative, schedule)?;
    let key = format!("{}/kernels", alternative.stable_id);
    let kernel = record_count_step(
        explain,
        "kernel.schedule-refinement",
        SubjectKind::Kernel,
        &key,
        ExplainStage::KernelRefinement,
        "kernel.exact-refinement",
        "kernel-count",
        alternative.kernels.len(),
        target,
    )?;
    let key = format!("{}/program", alternative.stable_id);
    let program = record_count_step(
        explain,
        "program.two-stage-materialized",
        SubjectKind::KernelProgram,
        &key,
        ExplainStage::ProgramVerification,
        "program.verified",
        "stage-count",
        alternative.program.stages().len(),
        kernel,
    )?;
    let key = format!("{}/artifact", alternative.stable_id);
    record_count_step(
        explain,
        "artifact.neutral-construction-plan",
        SubjectKind::ArtifactPlan,
        &key,
        ExplainStage::ArtifactPlanning,
        "artifact.plan-verified",
        "provider-count",
        alternative.artifact_plan.lowering_providers().len(),
        program,
    )
}

#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    reason = "keeps one fused-alternative transaction and its explain causes together"
)]
fn consider_fused_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    formation: &RegionFormationOutcome,
    alternatives: &mut Vec<ProgramAlternative>,
    explain: &mut ExplainWriter,
    records: RegionFormationRecords,
    alternative_causes: &mut Vec<(&'static str, Option<ExplainRecordId>)>,
    target_rejections: &mut TargetRejections,
) -> Result<(), TargetFailure> {
    let root = records.summary;
    match formation.whole_program_candidate() {
        // A whole-graph set is trivially convex, so for a request the boundary
        // already admitted the only reason it can be absent is a budget that
        // stopped that growth path. Its typed stop is already recorded, and the
        // unfused baseline survives.
        None if !formation.budget_stops().is_empty() => {}
        None => {
            return Err(failure_at_source(
                CompileError::InvalidCompilerOutput(CompilerOutputError::Region(
                    RegionError::Structure {
                        rule: "missing-whole-program-region",
                    },
                )),
                ExplainStage::RegionFormation,
                record_cause(root),
            ));
        }
        Some(fused_candidate) => {
            let fused_candidate_record = records.whole_program;
            if let Some(provider) = verified.capabilities().fused_serial_sum {
                let proof = prove_fused_numerics(formation.graph(), verified, fused_candidate)
                    .map_err(|error| {
                        failure_at_source(
                            error.into(),
                            ExplainStage::NumericalLegality,
                            record_cause(fused_candidate_record.or(root)),
                        )
                    })?;
                let proof_record = (|| -> Result<_, CompileError> {
                    let provider_ref = ProviderRef::lowering(provider)?;
                    let subject =
                        explain.subject(SubjectKind::Candidate, fused_candidate.stable_id())?;
                    Ok(explain.push_detail(
                        RuleRef::provided(
                            "fusion.strict-f32-equivalence",
                            1,
                            provider_ref.clone(),
                        )?,
                        vec![subject],
                        check(
                            ExplainStage::NumericalLegality,
                            "fusion.strict-f32-equivalence",
                            EvidenceBasis::SoundProof(VerifiedEvidenceRef::from_fusion_numerical(
                                verified,
                                &proof,
                                provider_ref,
                            )?),
                        )?,
                        optional_cause(fused_candidate_record),
                    )?)
                })()
                .map_err(|source| {
                    failure_at_source(
                        source,
                        ExplainStage::NumericalLegality,
                        record_cause(fused_candidate_record.or(root)),
                    )
                })?;
                match build_fused_scheduled_region(verified) {
                    Err(error @ PhysicalError::Target { .. }) => {
                        let cause = record_target_rejection(
                            explain,
                            &error,
                            "alternative:fused-serial-sum.v1",
                            proof_record.or(root),
                        )?;
                        target_rejections.push(TargetRejection {
                            kind: ProgramAlternativeKind::Fused,
                            error,
                            cause,
                        })?;
                    }
                    Err(error) => {
                        let stage = physical_error_stage(&error);
                        return Err(failure_at_source(
                            error.into(),
                            stage,
                            record_cause(proof_record.or(root)),
                        ));
                    }
                    Ok(fused_region) => {
                        let fused_kernel =
                            lower_structured_kernel(&fused_region).map_err(|error| {
                                let stage = physical_error_stage(&error);
                                failure_at_source(
                                    error.into(),
                                    stage,
                                    record_cause(proof_record.or(root)),
                                )
                            })?;
                        let fused_program = build_fused_kernel_program(verified, &fused_region)
                            .map_err(|error| {
                                failure_at_source(
                                    error.into(),
                                    ExplainStage::ProgramVerification,
                                    record_cause(proof_record.or(root)),
                                )
                            })?;
                        assert_kernels_match_program(
                            verified,
                            std::slice::from_ref(&fused_region),
                            &fused_program,
                            std::slice::from_ref(&fused_kernel),
                        )
                        .map_err(|error| {
                            failure_at_source(
                                error.into(),
                                ExplainStage::ProgramVerification,
                                record_cause(proof_record.or(root)),
                            )
                        })?;
                        let fused_artifact = build_artifact_plan(
                            semantic,
                            verified,
                            std::slice::from_ref(&fused_region),
                            std::slice::from_ref(&fused_kernel),
                            &fused_program,
                            vec![provider],
                        )
                        .map_err(|error| {
                            failure_at_source(
                                error.into(),
                                ExplainStage::ArtifactPlanning,
                                record_cause(proof_record.or(root)),
                            )
                        })?;
                        let alternative = fused_alternative(
                            fused_region,
                            fused_kernel,
                            fused_program,
                            fused_artifact,
                            proof,
                        );
                        let cause = record_fused_refinement(
                            explain,
                            verified,
                            &alternative,
                            proof_record.or(root),
                        )?;
                        alternative_causes.push(("alternative:fused-serial-sum.v1", cause));
                        alternatives.push(alternative);
                    }
                }
            } else {
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(
                        SubjectKind::Capability,
                        "fused-serial-sum/provider:tiler.prototype.fused-serial-sum@1",
                    )?;
                    explain.push_detail(
                        RuleRef::builtin("fusion.capability-resolution")?,
                        vec![subject],
                        ExplainEvent::DeferredCapability {
                            predicate: PredicateKey::new("fusion.provider-available")?,
                            reason: ReasonCode::new("provider-unavailable")?,
                        },
                        optional_cause(fused_candidate_record),
                    )?;
                    Ok(())
                })()
                .map_err(|source| {
                    failure_at_source(
                        source,
                        ExplainStage::CapabilityResolution,
                        record_cause(fused_candidate_record.or(root)),
                    )
                })?;
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
) -> Result<TerminalCause, TargetFailure> {
    let PhysicalError::Target {
        rule,
        region,
        required,
        available,
    } = error
    else {
        unreachable!("target rejection records require a target-feasibility error")
    };
    let key = format!("{alternative}/region:{}", region.0);
    let rejected = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Region, &key)?;
            Ok(explain.push_causal_detail(
                RuleRef::builtin(format!("target.{rule}"))?,
                subject,
                &ExplainEvent::Feasibility {
                    predicate: PredicateKey::new(*rule)?,
                    outcome: crate::explain::FeasibilityOutcome::Rejected(ReasonCode::new(
                        "target-infeasible",
                    )?),
                    required: target_quantity(rule, *required)?,
                    available: target_quantity(rule, *available)?,
                },
                optional_cause(cause),
            )?)
        })(),
        ExplainStage::TargetFeasibility,
        SubjectKind::Region,
        &key,
        record_cause(cause),
    )?;
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Alternative, alternative)?;
            explain.note_infeasible_alternative(subject, Some(rejected.clone()))?;
            Ok(())
        })(),
        ExplainStage::Selection,
        SubjectKind::Alternative,
        alternative,
        Some(rejected.clone()),
    )?;
    Ok(rejected)
}

fn record_target_admissions(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    mut cause: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, TargetFailure> {
    let profile = request.target_profile();
    for scheduled in &alternative.scheduled_regions {
        let region = scheduled.region();
        // Re-derive the admitted feasibility facts from the single feasibility
        // authority rather than a parallel check list, so the admitted trace
        // cannot drift from the decision that admitted the region. A verified
        // region has already proven feasible, so a non-proven verdict here is an
        // internal inconsistency and fails closed via the physical-error stage.
        let admitted = crate::physical::assess_region(
            region.index.id,
            scheduled.requirements(),
            region.schedule.work_items,
            &profile,
        )
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, record_cause(cause))
        })?;
        for predicate in admitted {
            let key = format!("{}/region:{}", alternative.stable_id, region.index.id.0);
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!("target.{}", predicate.axis().key()))?,
                        vec![subject],
                        ExplainEvent::Feasibility {
                            predicate: PredicateKey::new(predicate.axis().key())?,
                            outcome: crate::explain::FeasibilityOutcome::Admitted,
                            required: predicate.required(),
                            available: predicate.available(),
                        },
                        optional_cause(cause),
                    )?)
                })(),
                ExplainStage::TargetFeasibility,
                SubjectKind::Region,
                &key,
                record_cause(cause),
            )?;
        }
    }
    Ok(cause)
}

fn target_quantity(rule: &str, value: u64) -> Result<Quantity, ExplainError> {
    match rule {
        "grid-axis" | "threads-per-workgroup" => Ok(Quantity::Threads(value)),
        "buffer-bindings" => Ok(Quantity::Bindings(value)),
        "local-memory-bytes" => Ok(Quantity::Bytes(value)),
        "index-bits" | "device-memory" | "strict-f32" | "barriers" => Ok(Quantity::Count(value)),
        _ => Err(ExplainError::UnknownQuantityUnit),
    }
}

fn record_fused_refinement(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    root: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, TargetFailure> {
    let key = format!("{}/schedules", alternative.stable_id);
    let schedule = record_count_step(
        explain,
        "schedule.fused-serial-sum",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "schedule.intrinsic-valid",
        "schedule-count",
        alternative.scheduled_regions.len(),
        root,
    )?;
    let target = record_target_admissions(explain, request, alternative, schedule)?;
    let key = format!("{}/kernels", alternative.stable_id);
    let kernel = record_count_step(
        explain,
        "kernel.fused-schedule-refinement",
        SubjectKind::Kernel,
        &key,
        ExplainStage::KernelRefinement,
        "kernel.exact-refinement",
        "kernel-count",
        alternative.kernels.len(),
        target,
    )?;
    let key = format!("{}/program", alternative.stable_id);
    let program = record_count_step(
        explain,
        "program.fused-serial-sum",
        SubjectKind::KernelProgram,
        &key,
        ExplainStage::ProgramVerification,
        "program.verified",
        "stage-count",
        alternative.program.stages().len(),
        kernel,
    )?;
    let key = format!("{}/artifact", alternative.stable_id);
    record_count_step(
        explain,
        "artifact.fused-construction-plan",
        SubjectKind::ArtifactPlan,
        &key,
        ExplainStage::ArtifactPlanning,
        "artifact.plan-verified",
        "provider-count",
        alternative.artifact_plan.lowering_providers().len(),
        program,
    )
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
) -> Result<(), TargetFailure> {
    for alternative in alternatives {
        let cost = alternative.structural_cost;
        let cause = causes
            .iter()
            .find_map(|(id, cause)| (*id == alternative.stable_id).then_some(*cause))
            .flatten();
        let (subject, cost_record) = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Alternative, alternative.stable_id)?;
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
                let record = explain.push_causal_detail(
                    RuleRef::builtin(STRUCTURAL_COST_MODEL_KEY)?,
                    subject.clone(),
                    &ExplainEvent::CostAssessment {
                        model: CostModelKey::new(STRUCTURAL_COST_MODEL_KEY)?,
                        basis: EvidenceBasis::CheckedInvariant,
                        terms,
                        disposition: CostDisposition::Retained,
                    },
                    optional_cause(cause),
                )?;
                Ok((subject, record))
            })(),
            ExplainStage::Costing,
            SubjectKind::Alternative,
            alternative.stable_id,
            record_cause(cause),
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
            SelectionOutcome::NotSelectedTradeoff
        };
        explain_step(
            explain
                .note_selection(subject, outcome, Some(cost_record.clone()))
                .map_err(Into::into),
            ExplainStage::Selection,
            SubjectKind::Alternative,
            alternative.stable_id,
            Some(cost_record),
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
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    if alternatives.is_empty()
        || alternatives
            .iter()
            .map(|alternative| alternative.stable_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != alternatives.len()
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-identity",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    for alternative in alternatives {
        verify_alternative(semantic, request, alternative, cause.clone())?;
    }
    let recomputed = select_structural_pareto(alternatives)
        .map_err(|source| failure_at_source(source, ExplainStage::Selection, cause.clone()))?;
    if selected_id != recomputed
        || !alternatives
            .iter()
            .any(|item| item.stable_id == selected_id)
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-selection",
            }
            .into(),
            ExplainStage::Selection,
            cause,
        ));
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

#[allow(
    clippy::too_many_lines,
    reason = "keeps each rederived layer beside its exact phase-local failure context"
)]
fn rederive_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    kind: ProgramAlternativeKind,
    cause: Option<TerminalCause>,
) -> Result<ExpectedAlternative, TargetFailure> {
    let (stable_id, cost) = match kind {
        ProgramAlternativeKind::Materialized => {
            let materialized_bytes = request
                .serial_sum()
                .input_elements
                .checked_mul(4)
                .ok_or_else(|| {
                    failure_at_source(
                        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                            ProgramError::Structure {
                                rule: "portfolio-cost-overflow",
                            },
                        )),
                        ExplainStage::Costing,
                        cause.clone(),
                    )
                })?;
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
        ProgramAlternativeKind::Materialized => {
            build_scheduled_regions(request).map_err(|error| {
                let stage = physical_error_stage(&error);
                failure_at_source(error.into(), stage, cause.clone())
            })?
        }
        ProgramAlternativeKind::Fused => {
            vec![build_fused_scheduled_region(request).map_err(|error| {
                let stage = physical_error_stage(&error);
                failure_at_source(error.into(), stage, cause.clone())
            })?]
        }
    };
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause.clone())
        })?;
    let program = match kind {
        ProgramAlternativeKind::Materialized => build_kernel_program(request, &scheduled),
        ProgramAlternativeKind::Fused => build_fused_kernel_program(request, &scheduled[0]),
    }
    .map_err(|error| {
        failure_at_source(
            error.into(),
            ExplainStage::ProgramVerification,
            cause.clone(),
        )
    })?;
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
    let artifact = build_artifact_plan(
        semantic, request, &scheduled, &kernels, &program, providers,
    )
    .map_err(|error| failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause))?;
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
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    let expected = rederive_alternative(semantic, request, alternative.kind, cause.clone())?;
    if alternative.stable_id != expected.stable_id || alternative.structural_cost != expected.cost {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-cost-or-identity",
            }
            .into(),
            ExplainStage::Costing,
            cause,
        ));
    }
    if alternative.scheduled_regions != expected.scheduled {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-schedule-binding",
            }
            .into(),
            ExplainStage::IntrinsicScheduling,
            cause,
        ));
    }
    if alternative.kernels != expected.kernels {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-kernel-binding",
            }
            .into(),
            ExplainStage::KernelRefinement,
            cause,
        ));
    }
    if alternative.program != expected.program {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-program-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    if alternative.artifact_plan != expected.artifact {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-artifact-receipt",
            }
            .into(),
            ExplainStage::ArtifactPlanning,
            cause,
        ));
    }
    verify_artifact_plan(
        &alternative.artifact_plan,
        semantic,
        request,
        &expected.scheduled,
        &expected.kernels,
        &expected.program,
        expected.artifact.lowering_providers().to_vec(),
    )
    .map_err(|error| {
        failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause.clone())
    })?;
    verify_equivalence(semantic, request, alternative)
        .map_err(|source| failure_at_source(source, ExplainStage::NumericalLegality, cause))
}

fn verify_equivalence(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
) -> Result<(), CompileError> {
    match &alternative.equivalence {
        EquivalenceEvidence::MaterializedReference
            if alternative.kind == ProgramAlternativeKind::Materialized => {}
        EquivalenceEvidence::Fused(proof) if alternative.kind == ProgramAlternativeKind::Fused => {
            let formation =
                form_region_candidates(semantic, request.budgets(), request.numerical_contract())?;
            let candidate = formation.whole_program_candidate().ok_or({
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-fused-region",
                    },
                ))
            })?;
            verify_fused_numerics(formation.graph(), request, candidate, proof)?;
            if alternative.scheduled_regions.len() != 1
                || alternative.scheduled_regions[0]
                    .region()
                    .index
                    .semantic_members
                    != candidate.members()
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

    /// Builds the serial-sum program with one constant shared by both operands.
    ///
    /// This is the canonical spelling that `NormalizeSemantics` produces from a
    /// program that authored the same constant twice.
    fn shared_constant_semantic(shape: Shape, constant_bits: u32) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let constant = F32Constant::apply(&mut builder, constant_bits).unwrap();
        let product = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let mapped = F32Add::apply(&mut builder, product, constant).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
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
        let rendered = first.explain.render();
        assert!(rendered.starts_with("tiler-explain-v2 request="));
        assert!(rendered.contains("feasibility:threads-per-workgroup:admitted"));
        assert!(rendered.contains("feasibility:buffer-bindings:admitted"));
        assert!(rendered.contains("event=selection:tiler.selection.structural-pareto.v1:selected"));
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
    #[allow(
        clippy::too_many_lines,
        reason = "one exhaustive fixture checks every current typed emitter family"
    )]
    fn end_to_end_explain_emitter_has_exhaustive_typed_conformance() {
        let semantic = semantic(false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let trace = &product.targets[0].explain;
        let rule_counts =
            trace
                .records()
                .iter()
                .fold(std::collections::BTreeMap::new(), |mut counts, record| {
                    *counts
                        .entry(record.rule().key().as_str())
                        .or_insert(0_usize) += 1;
                    counts
                });
        assert_eq!(
            rule_counts,
            std::collections::BTreeMap::from([
                ("artifact.fused-construction-plan", 1),
                ("artifact.neutral-construction-plan", 1),
                ("compile.boundary.materialized", 1),
                ("compile.region.verified", 2),
                ("compile.request.general-boundary", 1),
                ("fusion.strict-f32-equivalence", 1),
                ("kernel.fused-schedule-refinement", 1),
                ("kernel.schedule-refinement", 1),
                ("normalize.semantics.v1", 1),
                ("program.fused-serial-sum", 1),
                ("program.two-stage-materialized", 1),
                ("region.candidate.v1", 17),
                ("region.formation.v1", 1),
                ("schedule.coverage-and-ownership", 1),
                ("schedule.fused-serial-sum", 1),
                ("target.barriers", 3),
                ("target.buffer-bindings", 3),
                ("target.device-memory", 3),
                ("target.grid-axis", 3),
                ("target.index-bits", 3),
                ("target.local-memory-bytes", 3),
                ("target.strict-f32", 3),
                ("target.threads-per-workgroup", 3),
                ("tiler.cost.structural.v1", 2),
                ("tiler.selection.structural-pareto.v1", 2),
            ])
        );
        let expected_counts = [
            ("normalize.semantics.v1", "rewrite-count", 0),
            ("region.formation.v1", "candidate-count", 17),
            ("region.formation.v1", "operation-count", 5),
            ("compile.boundary.materialized", "dependency-count", 1),
            ("schedule.coverage-and-ownership", "schedule-count", 2),
            ("kernel.schedule-refinement", "kernel-count", 2),
            ("program.two-stage-materialized", "stage-count", 2),
            ("artifact.neutral-construction-plan", "provider-count", 1),
            ("schedule.fused-serial-sum", "schedule-count", 1),
            ("kernel.fused-schedule-refinement", "kernel-count", 1),
            ("program.fused-serial-sum", "stage-count", 1),
            ("artifact.fused-construction-plan", "provider-count", 1),
        ];
        for (rule, fact_key, expected) in expected_counts {
            let record = trace
                .records()
                .iter()
                .find(|record| record.rule().key().as_str() == rule)
                .unwrap_or_else(|| panic!("missing typed count emitter {rule}"));
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                panic!("typed count emitter {rule} must be a checked assertion");
            };
            assert!(assessment.predicate().as_str().contains('.'));
            assert!(assessment.facts().iter().any(|fact| {
                fact.key().as_str() == fact_key
                    && matches!(fact.value(), FactValue::Count(value) if *value == expected)
            }));
        }

        let mut target_predicates = std::collections::BTreeMap::new();
        for record in trace.records() {
            let ExplainEvent::Feasibility {
                predicate,
                outcome: crate::explain::FeasibilityOutcome::Admitted,
                required,
                available,
            } = record.event()
            else {
                continue;
            };
            let unit_is_exact = match predicate.as_str() {
                "grid-axis" | "threads-per-workgroup" => {
                    matches!(
                        (required, available),
                        (Quantity::Threads(_), Quantity::Threads(_))
                    )
                }
                "buffer-bindings" => matches!(
                    (required, available),
                    (Quantity::Bindings(_), Quantity::Bindings(_))
                ),
                "local-memory-bytes" => {
                    matches!(
                        (required, available),
                        (Quantity::Bytes(_), Quantity::Bytes(_))
                    )
                }
                "index-bits" | "device-memory" | "strict-f32" | "barriers" => {
                    matches!(
                        (required, available),
                        (Quantity::Count(_), Quantity::Count(_))
                    )
                }
                other => panic!("unexpected target predicate {other}"),
            };
            assert!(unit_is_exact);
            *target_predicates
                .entry(predicate.as_str())
                .or_insert(0_usize) += 1;
        }
        assert_eq!(
            target_predicates,
            std::collections::BTreeMap::from([
                ("barriers", 3),
                ("buffer-bindings", 3),
                ("device-memory", 3),
                ("grid-axis", 3),
                ("index-bits", 3),
                ("local-memory-bytes", 3),
                ("strict-f32", 3),
                ("threads-per-workgroup", 3),
            ])
        );

        let selections = trace
            .records()
            .iter()
            .filter_map(|record| match record.event() {
                ExplainEvent::Selection { outcome, .. } => {
                    Some((record.subjects()[0].key().as_str(), *outcome))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            selections.get("alternative:materialized-serial-sum.v1"),
            Some(&SelectionOutcome::Dominated)
        );
        assert_eq!(
            selections.get("alternative:fused-serial-sum.v1"),
            Some(&SelectionOutcome::Selected)
        );
        assert!(trace.render().starts_with("tiler-explain-v2 request="));
    }

    #[test]
    fn normalization_converges_duplicated_and_shared_constants_on_one_portfolio() {
        let shape = Shape::from_dims([2, 3]);
        let bits = 2.0_f32.to_bits();
        let duplicated = semantic_case(shape.clone(), bits, bits, false);
        let shared = shared_constant_semantic(shape, bits);
        assert_eq!(duplicated.operation_count(), 5);
        assert_eq!(shared.operation_count(), 4);
        assert_ne!(
            duplicated.semantic_identity().graph(),
            shared.semantic_identity().graph()
        );

        let from_duplicated = compile(CompilationRequest::governed(&duplicated)).unwrap();
        let from_shared = compile(CompilationRequest::governed(&shared)).unwrap();

        // Both spellings normalize to the same canonical program, so every
        // downstream physical decision and receipt is identical.
        assert_eq!(
            from_duplicated.targets[0].portfolio,
            from_shared.targets[0].portfolio
        );

        // The traces differ only in what normalization actually did.
        let rewrite_counts = |product: &CompilationProduct| {
            product.targets[0]
                .explain
                .records()
                .iter()
                .find(|record| record.rule().key().as_str() == "normalize.semantics.v1")
                .and_then(|record| match record.event() {
                    ExplainEvent::Check { assessment, .. } => Some(
                        assessment
                            .facts()
                            .iter()
                            .find(|fact| fact.key().as_str() == "rewrite-count")
                            .map(|fact| fact.value().clone())
                            .unwrap(),
                    ),
                    _ => None,
                })
                .unwrap()
        };
        assert_eq!(rewrite_counts(&from_duplicated), FactValue::Count(1));
        assert_eq!(rewrite_counts(&from_shared), FactValue::Count(0));
        assert!(
            from_duplicated.targets[0]
                .explain
                .records()
                .iter()
                .any(
                    |record| record.rule().key().as_str() == "normalize.common-subexpression.v1"
                        && record.event().disposition() == ExplainDisposition::Admitted
                )
        );
        assert!(
            !from_shared.targets[0]
                .explain
                .records()
                .iter()
                .any(|record| record.rule().key().as_str() == "normalize.common-subexpression.v1")
        );
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
    fn missing_provider_and_region_budget_retain_the_verified_baseline() {
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

        // A growth budget stops the fused region without removing the complete
        // singleton coverage the unfused baseline depends on.
        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.region_candidates_per_seed = 0;
        let product = compile(bounded).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert_eq!(
            product.targets[0]
                .portfolio
                .selection
                .selected_alternative_id,
            "alternative:materialized-serial-sum.v1"
        );
        assert!(product.targets[0].explain.records().iter().any(|record| {
            record.rule().key().as_str() == "region.formation.v1"
                && record.event().disposition() == ExplainDisposition::BudgetStopped
        }));
        assert_eq!(
            product.targets[0]
                .explain
                .records()
                .iter()
                .filter(|record| record.rule().key().as_str() == "region.candidate.v1")
                .count(),
            5
        );
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
    fn no_feasible_plan_retains_a_typed_terminal_failure_trace() {
        let semantic = semantic_case_with_axis(
            Shape::from_dims([70_000, 70_000]),
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            false,
            Axis::new(1),
        );
        let error = compile(CompilationRequest::governed(&semantic)).unwrap_err();
        let CompileError::Explained { source, explain } = error else {
            panic!("target compilation failures retain their explain trace");
        };
        let CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
            region,
            ..
        })) = *source
        else {
            panic!("source retains the exact selected target rejection");
        };
        assert_eq!(
            explain
                .records()
                .iter()
                .filter(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
                .count(),
            1
        );
        let failure = explain
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .unwrap();
        assert!(matches!(
            failure.event(),
            ExplainEvent::CompilerFailure {
                stage: ExplainStage::TargetFeasibility,
                reason,
            } if reason.as_str() == "target-grid-axis"
        ));
        assert_eq!(failure.causes().len(), 2);
        let causal_rejections = failure
            .causes()
            .iter()
            .map(|cause| {
                explain
                    .records()
                    .iter()
                    .find(|record| record.id() == *cause)
                    .expect("every failure cause is a retained exact target rejection")
            })
            .collect::<Vec<_>>();
        assert!(
            causal_rejections.iter().all(|record| {
                record.event().disposition() == ExplainDisposition::RejectedTarget
            })
        );
        assert_eq!(
            causal_rejections
                .iter()
                .map(|record| record.subjects()[0].key().as_str())
                .collect::<Vec<_>>(),
            [
                format!("alternative:materialized-serial-sum.v1/region:{}", region.0),
                format!("alternative:fused-serial-sum.v1/region:{}", region.0),
            ]
        );
    }

    #[test]
    fn no_feasible_failure_aggregates_distinct_alternative_rejection_predicates() {
        let semantic = semantic(false);
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let request = request.for_target(request.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let materialized = PhysicalError::Target {
            rule: "grid-axis",
            region: RegionId(0),
            required: 65_536,
            available: 65_535,
        };
        let fused = PhysicalError::Target {
            rule: "threads-per-workgroup",
            region: RegionId(1),
            required: 2,
            available: 1,
        };
        let materialized_cause = record_target_rejection(
            &mut explain,
            &materialized,
            "alternative:materialized-serial-sum.v1",
            None,
        )
        .unwrap();
        let fused_cause = record_target_rejection(
            &mut explain,
            &fused,
            "alternative:fused-serial-sum.v1",
            None,
        )
        .unwrap();
        let mut rejections = TargetRejections::default();
        rejections
            .push(TargetRejection {
                kind: ProgramAlternativeKind::Fused,
                error: fused,
                cause: fused_cause,
            })
            .unwrap();
        rejections
            .push(TargetRejection {
                kind: ProgramAlternativeKind::Materialized,
                error: materialized,
                cause: materialized_cause,
            })
            .unwrap();
        let failure = rejections.into_failure().unwrap();
        let trace = explain.finish_failure(*failure.context).unwrap();
        let terminal = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .unwrap();
        assert_eq!(terminal.causes().len(), 2);
        let predicates = terminal
            .causes()
            .iter()
            .map(|cause| {
                trace
                    .records()
                    .iter()
                    .find(|record| record.id() == *cause)
                    .and_then(|record| match record.event() {
                        ExplainEvent::Feasibility { predicate, .. } => Some(predicate.as_str()),
                        _ => None,
                    })
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(predicates, ["grid-axis", "threads-per-workgroup"]);
    }

    #[test]
    fn rederivation_uses_the_physical_error_stage_for_both_alternative_kinds() {
        let semantic = semantic_case_with_axis(
            Shape::from_dims([70_000, 70_000]),
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            false,
            Axis::new(1),
        );
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let request = request.for_target(request.target_profiles()[0]).unwrap();
        for kind in [
            ProgramAlternativeKind::Materialized,
            ProgramAlternativeKind::Fused,
        ] {
            let Err(failure) = rederive_alternative(&semantic, &request, kind, None) else {
                panic!("oversized alternative must fail target feasibility");
            };
            assert_eq!(failure.context.stage, ExplainStage::TargetFeasibility);
            assert_eq!(failure.context.reason.as_str(), "target-grid-axis");
            assert!(matches!(
                *failure.source,
                CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                    PhysicalError::Target {
                        rule: "grid-axis",
                        ..
                    }
                ))
            ));
        }
        assert_eq!(
            physical_error_stage(&PhysicalError::Intrinsic {
                rule: "fixture",
                region: RegionId(0),
            }),
            ExplainStage::IntrinsicScheduling
        );
        assert_eq!(
            physical_error_stage(&PhysicalError::ShapeProductOverflow {
                region: RegionId(0),
            }),
            ExplainStage::IntrinsicScheduling
        );
        assert_eq!(
            physical_error_stage(&PhysicalError::Refinement {
                rule: "fixture",
                region: RegionId(0),
            }),
            ExplainStage::KernelRefinement
        );
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

        let semantic = semantic(false);
        let verified = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut alternatives = compile(CompilationRequest::governed(&semantic))
            .unwrap()
            .targets
            .remove(0)
            .portfolio
            .alternatives;
        alternatives[1].structural_cost.temporary_allocation_count = 2;
        let selected = select_structural_pareto(&alternatives).unwrap();
        assert_eq!(selected, "alternative:materialized-serial-sum.v1");
        let mut explain = ExplainWriter::new(&target, ExplainLimits::default()).unwrap();
        record_cost_and_selection(&alternatives, selected, &[], &mut explain).unwrap();
        let ids = alternatives
            .iter()
            .map(|alternative| alternative.stable_id)
            .collect::<Vec<_>>();
        let trace = explain.finish_success(&ids, selected).unwrap();
        assert!(trace.records().iter().any(|record| {
            record.subjects()[0].key().as_str() == "alternative:fused-serial-sum.v1"
                && matches!(
                    record.event(),
                    ExplainEvent::Selection {
                        outcome: SelectionOutcome::NotSelectedTradeoff,
                        ..
                    }
                )
        }));
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

        assert!(verify_portfolio(&semantic, &request, alternatives, selected, None).is_ok());
        assert!(verify_portfolio(&semantic, &request, &[], selected, None).is_err());
        let selection =
            verify_portfolio(&semantic, &request, alternatives, "stale-selection", None)
                .unwrap_err();
        assert_eq!(selection.context.stage, ExplainStage::Selection);
        assert_eq!(
            selection.context.reason.as_str(),
            "structure-portfolio-selection"
        );

        let mut forged = alternatives.clone();
        forged[0].structural_cost.dispatch_count = 0;
        assert!(verify_portfolio(&semantic, &request, &forged, selected, None).is_err());

        let mut forged_artifact = alternatives.clone();
        forged_artifact[0].artifact_plan = forged_artifact[1].artifact_plan.clone();
        let artifact =
            verify_portfolio(&semantic, &request, &forged_artifact, selected, None).unwrap_err();
        assert_eq!(artifact.context.stage, ExplainStage::ArtifactPlanning);
        assert_eq!(
            artifact.context.reason.as_str(),
            "structure-portfolio-artifact-receipt"
        );

        let mut forged_numerics = alternatives.clone();
        forged_numerics[0].equivalence = forged_numerics[1].equivalence.clone();
        let numerical =
            verify_portfolio(&semantic, &request, &forged_numerics, selected, None).unwrap_err();
        assert_eq!(numerical.context.stage, ExplainStage::NumericalLegality);
        assert_eq!(
            numerical.context.reason.as_str(),
            "structure-portfolio-equivalence"
        );
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
