use std::error::Error;
use std::fmt;

use crate::cover::{CoverEnumeration, CoverError, RegionCover, enumerate_covers};
use crate::explain::{
    CostDisposition, CostModelKey, CostTerm, EvidenceBasis, ExplainError, ExplainEvent,
    ExplainFact, ExplainRecordId, ExplainStage, ExplainWriter, FactValue, FailureDescriptor,
    MAX_TERMINAL_CAUSES, PredicateAssessment, PredicateKey, ProviderRef, Quantity, ReasonCode,
    RejectionClass, RuleRef, SelectionOutcome, SubjectKind, TerminalCause, VerifiedEvidenceRef,
    VerifiedExplainTrace,
};
use crate::frontier::{
    FrontierError, FrontierRegionSubject, GovernedPhysicalProvider, PhysicalImplementationProvider,
    enumerate_frontier,
};
use crate::fusion::{
    FusionError, FusionNumericalProof, prove_fused_numerics, verify_fused_numerics,
};
use crate::fusion_legality::{
    FusionLegality, FusionLegalityError, FusionLegalityProof, FusionNumericalCapabilities,
    derive_fusion_legality, verify_fusion_legality,
};
use crate::lowering::{LoweringError, OccurrenceEvidence, ResolvedLowering, resolve_lowering};
use crate::normalize::{
    NORMALIZATION_SUBJECT, NormalizationOutcome, NormalizeError, normalize_semantics,
};
use crate::physical::{
    PhysicalError, VerifiedKernel, VerifiedScheduledRegion, lower_structured_kernel,
};
use crate::program::{
    ArtifactConstructionPlan, KernelProgram, ProgramError, assert_kernels_match_program,
    build_artifact_plan, build_fused_kernel_program, build_kernel_program, verify_artifact_plan,
    verify_semantic_output_type,
};
use crate::region::{
    REGION_FORMATION_SUBJECT, RegionCandidate, RegionError, RegionFormationOutcome,
    form_region_candidates,
};
use crate::request::{CompilationRequest, RequestError, verify_request};
use crate::selection::{
    CoverFrontiers, PlanStructuralCost, RegionFrontier, SelectedPlan, SelectedPortfolio,
    SelectionError, select_physical_plans, verify_selected_portfolio,
};

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

impl ProgramAlternativeKind {
    /// Classifies a plan by whether one region covers the whole program.
    ///
    /// The classification is a presentation and program-assembly discriminator
    /// derived from the plan's cover, never a separate authority: a plan is fused
    /// exactly when its cover places every operation in one region.
    fn of(cover: &RegionCover, operation_count: u32) -> Self {
        let whole = cover.region_count() == 1
            && cover.regions().first().is_some_and(|region| {
                u32::try_from(region.members().len()).is_ok_and(|count| count == operation_count)
            });
        if whole {
            Self::Fused
        } else {
            Self::Materialized
        }
    }

    #[allow(
        dead_code,
        reason = "stable presentation name of the plan shape, read by diagnostics and by this module's tests"
    )]
    const fn name(self) -> &'static str {
        match self {
            Self::Materialized => "materialized",
            Self::Fused => "fused",
        }
    }
}

/// The numerical-equivalence evidence one retained alternative rests on.
///
/// Every multi-occurrence region of the plan carries a replayable fusion-legality
/// proof, and a whole-program fused region additionally carries the strict-`f32`
/// numerical-equivalence proof the explain trace cites as a sound proof. A plan
/// whose regions are all single occurrences fuses nothing and carries neither.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EquivalenceEvidence {
    /// One legality proof per multi-occurrence region, in cover-region order.
    legality: Vec<(usize, Box<FusionLegalityProof>)>,
    /// The strict-`f32` equivalence proof of a whole-program fused region.
    numerical: Option<Box<FusionNumericalProof>>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl EquivalenceEvidence {
    /// Returns the per-region fusion-legality proofs the plan rests on.
    pub(crate) fn legality(&self) -> &[(usize, Box<FusionLegalityProof>)] {
        &self.legality
    }

    /// Returns the whole-program strict-`f32` numerical equivalence proof.
    pub(crate) fn numerical(&self) -> Option<&FusionNumericalProof> {
        self.numerical.as_deref()
    }
}

/// One retained complete plan, assembled through structured KIR into a verified
/// kernel program and a neutral artifact construction plan.
///
/// The alternative *is* the selected physical plan: its stable identifier is the
/// plan's content-derived identity label, and its cost is the plan's exact
/// aggregate structural cost. Nothing here re-decides feasibility or legality;
/// both were settled by the frontier and the fusion-legality authority before the
/// plan was retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramAlternative {
    pub(crate) stable_id: String,
    pub(crate) kind: ProgramAlternativeKind,
    pub(crate) plan: SelectedPlan,
    pub(crate) scheduled_regions: Vec<VerifiedScheduledRegion>,
    pub(crate) kernels: Vec<VerifiedKernel>,
    pub(crate) program: KernelProgram,
    pub(crate) artifact_plan: ArtifactConstructionPlan,
    pub(crate) structural_cost: PlanStructuralCost,
    pub(crate) equivalence: EquivalenceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PortfolioSelection {
    pub(crate) policy_key: &'static str,
    pub(crate) selected_alternative_id: String,
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
    /// No legal complete cover joined with a compatible implementation set.
    Selection(SelectionError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompilerOutputError {
    Physical(PhysicalError),
    Program(ProgramError),
    Region(RegionError),
    Fusion(FusionError),
    Explain(ExplainError),
    Normalization(NormalizeError),
    Cover(CoverError),
    FusionLegality(FusionLegalityError),
    Frontier(FrontierError),
    Selection(SelectionError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error)
            | Self::UnsupportedCapability(error)
            | Self::BudgetExhausted(error)
            | Self::NoFeasiblePlan(NoFeasiblePlanError::Request(error)) => error.fmt(formatter),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(error)) => error.fmt(formatter),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Selection(error))
            | Self::InvalidCompilerOutput(CompilerOutputError::Selection(error)) => {
                error.fmt(formatter)
            }
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
            Self::InvalidCompilerOutput(CompilerOutputError::Cover(error)) => error.fmt(formatter),
            Self::InvalidCompilerOutput(CompilerOutputError::FusionLegality(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Frontier(error)) => {
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
            Self::NoFeasiblePlan(NoFeasiblePlanError::Selection(error))
            | Self::InvalidCompilerOutput(CompilerOutputError::Selection(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Program(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Region(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Fusion(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Explain(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Normalization(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Cover(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::FusionLegality(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Frontier(error)) => Some(error),
            Self::Explained { source, .. } => Some(source),
        }
    }
}

impl From<RequestError> for CompileError {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::UnsupportedCapability { .. } => Self::UnsupportedCapability(value),
            // A shape product that overflows, and a target that honours no
            // stated numerical contract, are both hard refusals about the
            // request rather than malformed requests — and neither is a cost.
            RequestError::ShapeProductOverflow { .. }
            | RequestError::NoResolvableNumericalContract { .. } => {
                Self::NoFeasiblePlan(NoFeasiblePlanError::Request(value))
            }
            RequestError::BudgetExceeded { .. } => Self::BudgetExhausted(value),
            RequestError::UnsupportedRequestVersion
            | RequestError::EmptyTargetSet
            | RequestError::DuplicateTargetProfile
            // Stating no contract at all is a malformed request, distinct from
            // stating one the target cannot honour.
            | RequestError::UnstatedNumericalContract
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
            PhysicalError::Target { .. } | PhysicalError::Numerical { .. } => {
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

impl From<CoverError> for CompileError {
    fn from(value: CoverError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Cover(value))
    }
}

impl From<FusionLegalityError> for CompileError {
    fn from(value: FusionLegalityError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::FusionLegality(value))
    }
}

impl From<FrontierError> for CompileError {
    fn from(value: FrontierError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Frontier(value))
    }
}

impl From<SelectionError> for CompileError {
    fn from(value: SelectionError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Selection(value))
    }
}

pub(crate) fn compile(request: CompilationRequest<'_>) -> Result<CompilationProduct, CompileError> {
    let semantic = request.program;
    let shape_environment = request.shape_environment;
    let target_profiles = request.target_profiles.clone();
    let capabilities = request.capabilities.clone();
    let verified = verify_request(request)?;
    verify_semantic_output_type(semantic)?;
    // `NormalizeSemantics` runs after request verification and before region
    // formation. It observes only the verified program and never mutates it.
    // Normalization observes the contract this compilation actually resolved to,
    // not the caller's preference list: a rewrite's legality depends on the one
    // contract in force, and the alternatives the caller would also have accepted
    // grant it nothing. It is program-scoped and runs before any per-target work,
    // so it fails closed rather than choosing when two targets resolved
    // differently.
    let resolved_contract =
        verified
            .uniform_resolved_contract()
            .ok_or(RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "divergent-resolved-contracts",
            })?;
    let normalization = normalize_semantics(semantic, verified.budgets(), resolved_contract)?;
    let Some(normalized) = normalization.normalized_program() else {
        return compile_verified(semantic, &verified, &normalization);
    };
    // A committed rewrite is a new program, so it must independently re-enter
    // the request boundary rather than inheriting the input's verification.
    // Rejection here is invalid compiler output, not an unsupported user
    // program: the input was already admitted. The caller's *stated* preference
    // is what re-enters, not the contract this run resolved: readmission must
    // repeat the resolution rather than inherit its answer, so a rewrite that
    // changed what the program requires cannot keep a resolution it invalidated.
    let readmitted = verify_request(CompilationRequest {
        program: normalized,
        shape_environment,
        numerical_contracts: verified.numerical_contracts().clone(),
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
    let mut explain = ExplainWriter::new(verified)?;
    match compile_target_with_explain(semantic, verified, normalization, &mut explain) {
        Ok(portfolio) => {
            let expected_alternatives = portfolio
                .alternatives
                .iter()
                .map(|alternative| alternative.stable_id.as_str())
                .collect::<Vec<_>>();
            let explain = explain.finish_success(
                &expected_alternatives,
                &portfolio.selection.selected_alternative_id,
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

/// Wraps the chain link a terminal failure at this step would cite.
///
/// Takes the record itself: a detail record is retained or the compilation is
/// refused, so a step that reached this point has one.
#[allow(
    clippy::unnecessary_wraps,
    reason = "adapts a known chain link into the optional-cause slot a terminal failure takes; the option is the callee's shape, not an uncertainty here"
)]
fn record_cause(record: ExplainRecordId) -> Option<TerminalCause> {
    Some(TerminalCause::from_record(record))
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
        // A numerical honourability rejection is a target-feasibility verdict,
        // not a numerical-legality one: `NumericalLegality` is where a *rewrite*
        // is judged against the contract, and this is the target being judged
        // against the same contract.
        PhysicalError::Target { .. } | PhysicalError::Numerical { .. } => {
            ExplainStage::TargetFeasibility
        }
        PhysicalError::Intrinsic { .. } | PhysicalError::ShapeProductOverflow { .. } => {
            ExplainStage::IntrinsicScheduling
        }
        PhysicalError::Refinement { .. } => ExplainStage::KernelRefinement,
    }
}

/// One region subject the implementation frontier rejected as hard-infeasible.
///
/// The rejection is a *local* target verdict, never a cost and never a global
/// coverage claim: it says this exact region cannot run on this target. It is
/// retained so an empty portfolio can report the exact disproved predicates that
/// made every plan impossible.
#[derive(Clone, Debug)]
struct TargetRejection {
    role: &'static str,
    error: PhysicalError,
    cause: TerminalCause,
}

#[derive(Default)]
struct TargetRejections {
    values: Vec<TargetRejection>,
}

impl TargetRejections {
    /// Retains one region-subject rejection, deduplicated by role and axis.
    fn push(&mut self, rejection: TargetRejection) -> Result<(), TargetFailure> {
        let key = |item: &TargetRejection| (item.role, target_axis(&item.error));
        if self
            .values
            .iter()
            .any(|existing| key(existing) == key(&rejection))
        {
            return Ok(());
        }
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
            .binary_search_by_key(&key(&rejection), key)
            .unwrap_or_else(|insertion| insertion);
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

const fn target_axis(error: &PhysicalError) -> &'static str {
    match error {
        PhysicalError::Target { rule, .. }
        | PhysicalError::Intrinsic { rule, .. }
        | PhysicalError::Refinement { rule, .. } => rule,
        PhysicalError::Numerical { cause, .. } => cause.dimension().key(),
        PhysicalError::ShapeProductOverflow { .. } => "shape-product-overflow",
    }
}

/// The stable presentation role of one cover region.
///
/// The role is derived from the region's *coverage* against the recognized
/// occurrences, so it names what the region means rather than where it appeared
/// in an enumeration. A region the bounded profile does not recognize keeps a
/// distinct role instead of being silently attributed to one it resembles.
fn region_role(
    request: &crate::request::VerifiedTargetRequest,
    members: &[crate::region::SemanticMemberId],
) -> &'static str {
    let recognized = &request.serial_sum().members;
    if members == recognized.pointwise() {
        "pointwise"
    } else if members == recognized.reduction() {
        "reduction"
    } else if members == recognized.all() {
        "whole-program"
    } else {
        "unrecognized"
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
    let region_root = region_records.summary;
    let plans = enumerate_complete_plans(
        semantic,
        verified,
        &formation,
        explain,
        region_root,
        region_records.whole_program,
    )?;
    let mut alternatives = Vec::new();
    let mut alternative_causes = Vec::new();
    let alternative_cause = record_cause(plans.selection_record);
    for plan in plans.portfolio.plans() {
        let kind = ProgramAlternativeKind::of(plan.cover(), formation.graph().operation_count());
        let alternative = build_alternative(
            semantic,
            verified,
            plan,
            kind,
            &plans,
            alternative_cause.as_ref(),
        )?;
        let cause =
            record_alternative_explain(explain, verified, &alternative, plans.selection_record)?;
        alternative_causes.push((alternative.stable_id.clone(), cause));
        alternatives.push(alternative);
    }
    if alternatives.is_empty() {
        if let Some(failure) = plans.rejections.into_failure() {
            return Err(failure);
        }
        return Err(target_failure(
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Selection(
                SelectionError::Structure {
                    rule: "no-complete-plan",
                },
            )),
            ExplainStage::Selection,
            "portfolio-empty-without-target-rejection",
            SubjectKind::KernelProgram,
            "portfolio",
            record_cause(region_root),
        ));
    }
    let selected_alternative_id =
        select_non_dominated(&plans.portfolio, &alternatives).map_err(|source| {
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
        &plans.portfolio,
        &alternatives,
        &selected_alternative_id,
        record_cause(region_root),
    )?;
    record_cost_and_selection(
        &alternatives,
        &selected_alternative_id,
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

/// Everything the complete-plan authorities produced for one target.
struct CompletePlans {
    /// Every recognized occurrence's resolved capability and refinement evidence.
    lowering: ResolvedLowering,
    portfolio: SelectedPortfolio,
    /// One replayable fusion-legality proof per multi-occurrence region, keyed by
    /// the region occurrence it was derived for.
    legality: std::collections::BTreeMap<
        crate::region::RegionOccurrenceIdentity,
        Box<FusionLegalityProof>,
    >,
    /// The whole-program strict-`f32` numerical equivalence proof, when a
    /// whole-program candidate exists and its fusion is legal.
    numerical: Option<Box<FusionNumericalProof>>,
    /// Region subjects the frontier rejected as hard-infeasible on this target.
    rejections: TargetRejections,
    /// The complete-plan selection record every alternative is caused by.
    selection_record: ExplainRecordId,
}

/// Enumerates legal covers, proves their fusion legality, enumerates the local
/// implementation frontier of every cover region, and joins them into complete
/// physical plans.
///
/// The three authorities stay separate exactly as their contracts require:
/// [`enumerate_covers`] answers a strictly global legality question and chooses
/// no implementation; [`derive_fusion_legality`] decides whether a
/// multi-occurrence region may be realized as one fused region at all;
/// [`enumerate_frontier`] answers a strictly local feasibility question for one
/// region and target; and only [`select_physical_plans`] joins them.
#[allow(
    clippy::too_many_lines,
    reason = "keeps the cover, legality, frontier, and join stages and their phase-local failure contexts in one readable transaction"
)]
fn enumerate_complete_plans(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    formation: &RegionFormationOutcome,
    explain: &mut ExplainWriter,
    root: ExplainRecordId,
    whole_program_record: Option<ExplainRecordId>,
) -> Result<CompletePlans, TargetFailure> {
    let budgets = verified.budgets();
    let contract = verified.numerical_contract();
    // Lowering-capability resolution precedes every cover: a cover is a claim
    // about how recognized occurrences are grouped, and grouping occurrences the
    // installed authority cannot lower at all would be enumerating plans nothing
    // could realize.
    let lowering = match resolve_lowering(semantic, verified, formation) {
        Ok(lowering) => lowering,
        Err(source) => {
            let cause = record_lowering_failure(explain, &source, root)?;
            return Err(lowering_failure(&source, cause));
        }
    };
    let lowering_record = record_lowering(explain, &lowering, root)?;
    let enumeration = enumerate_covers(semantic, budgets, contract).map_err(|source| {
        failure_at_source(
            source.into(),
            ExplainStage::CandidateEnumeration,
            record_cause(lowering_record),
        )
    })?;
    let cover_record = record_cover_enumeration(explain, &enumeration, lowering_record)?;

    let capabilities = FusionNumericalCapabilities::governed();
    let mut legality = std::collections::BTreeMap::new();
    let mut illegal = std::collections::BTreeSet::new();
    let mut legality_cause = cover_record;
    for cover in enumeration.covers() {
        for region in cover.regions() {
            if region.members().len() < 2
                || legality.contains_key(region.occurrence())
                || illegal.contains(region.occurrence())
            {
                continue;
            }
            let candidate = formation
                .candidates()
                .iter()
                .find(|candidate| candidate.occurrence() == region.occurrence())
                .ok_or_else(|| {
                    failure_at_source(
                        CompileError::InvalidCompilerOutput(CompilerOutputError::Cover(
                            CoverError::Structure {
                                rule: "cover-region-candidate",
                            },
                        )),
                        ExplainStage::CandidateEnumeration,
                        record_cause(cover_record),
                    )
                })?;
            let cause = if candidate.covers_whole_program() {
                whole_program_record.unwrap_or(legality_cause)
            } else {
                legality_cause
            };
            let outcome =
                derive_fusion_legality(semantic, budgets, contract, &capabilities, candidate)
                    .map_err(|source| {
                        failure_at_source(
                            source.into(),
                            ExplainStage::NumericalLegality,
                            record_cause(cover_record),
                        )
                    })?;
            legality_cause =
                record_fusion_legality(explain, &capabilities, candidate, &outcome, cause)?;
            match outcome {
                FusionLegality::Legal(proof) => {
                    legality.insert(region.occurrence().clone(), proof);
                }
                FusionLegality::Rejected(_) | FusionLegality::Unknown(_) => {
                    illegal.insert(region.occurrence().clone());
                }
            }
        }
    }

    // A whole-program candidate whose fusion is legal additionally carries the
    // strict-`f32` numerical equivalence proof the trace cites as a sound proof.
    let mut numerical = None;
    let mut numerical_cause = legality_cause;
    if let Some(candidate) = formation.whole_program_candidate()
        && !illegal.contains(candidate.occurrence())
    {
        let proof =
            prove_fused_numerics(formation.graph(), verified, candidate).map_err(|error| {
                failure_at_source(
                    error.into(),
                    ExplainStage::NumericalLegality,
                    record_cause(legality_cause),
                )
            })?;
        numerical_cause = record_numerical_equivalence(
            explain,
            verified,
            &lowering,
            candidate,
            &proof,
            legality_cause,
        )?;
        numerical = Some(Box::new(proof));
    }

    let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];
    let mut sources = Vec::new();
    let mut rejections = TargetRejections::default();
    let mut frontier_cause = numerical_cause;
    let mut recorded_roles = std::collections::BTreeMap::new();
    // Covers every one of whose regions was proposed for, but at least one of
    // which the target refused. A reader expects those ruled out by feasibility
    // rather than by a missing capability, so each is noted in the terminal
    // ledger as an infeasible alternative.
    let mut refused_covers: Vec<(String, TerminalCause)> = Vec::new();
    for cover in enumeration.covers() {
        if cover
            .regions()
            .iter()
            .any(|region| illegal.contains(region.occurrence()))
        {
            continue;
        }
        let mut region_frontiers = Vec::with_capacity(cover.region_count());
        let mut proposed_everywhere = true;
        let mut refusal: Option<TerminalCause> = None;
        for region in cover.regions() {
            let role = region_role(verified, region.members());
            let subject = FrontierRegionSubject::new(role, region.members().to_vec());
            let frontier =
                enumerate_frontier(verified, &subject, &providers).map_err(|source| {
                    failure_at_source(
                        source.into(),
                        ExplainStage::IntrinsicScheduling,
                        record_cause(numerical_cause),
                    )
                })?;
            if frontier.admitted().is_empty() && frontier.rejections().is_empty() {
                proposed_everywhere = false;
            }
            // One region role yields one region subject, so its frontier and any
            // rejection it carries are recorded exactly once however many covers
            // place that same region.
            let first_sighting = !recorded_roles.contains_key(role);
            if first_sighting {
                frontier_cause = record_frontier(explain, role, &frontier, frontier_cause)?;
                for rejection in frontier.rejections() {
                    let error = match rejection {
                        crate::frontier::FrontierRejection::Infeasible {
                            axis,
                            required,
                            available,
                            ..
                        } => Some(PhysicalError::Target {
                            rule: axis,
                            region: region_id_of(cover, region),
                            required: *required,
                            available: *available,
                        }),
                        crate::frontier::FrontierRejection::Unhonourable { cause, .. } => {
                            Some(PhysicalError::Numerical {
                                region: region_id_of(cover, region),
                                cause: *cause,
                            })
                        }
                        // A reserved body variant and an inapplicable proposal
                        // are not target verdicts and carry no rejection to
                        // attribute to this region.
                        crate::frontier::FrontierRejection::UnsupportedVariant { .. }
                        | crate::frontier::FrontierRejection::NotApplicable { .. } => None,
                    };
                    if let Some(error) = error {
                        let cause = record_target_rejection(explain, &error, role, frontier_cause)?;
                        recorded_roles.insert(role, Some(cause));
                        rejections.push(TargetRejection { role, error, cause })?;
                    }
                }
                recorded_roles.entry(role).or_insert(None);
            }
            if let Some(Some(cause)) = recorded_roles.get(role) {
                refusal.get_or_insert(*cause);
            }
            region_frontiers.push(RegionFrontier::new(subject, frontier));
        }
        if proposed_everywhere && let Some(cause) = refusal {
            refused_covers.push((cover.identity().label(), cause));
        }
        sources.push(CoverFrontiers::new(cover.clone(), region_frontiers));
    }

    let portfolio =
        select_physical_plans(semantic, budgets, contract, &sources).map_err(|source| {
            failure_at_source(
                source.into(),
                ExplainStage::Selection,
                record_cause(frontier_cause),
            )
        })?;
    for (label, cause) in refused_covers {
        if portfolio
            .plans()
            .iter()
            .all(|plan| plan.cover().identity().label() != label)
        {
            note_infeasible_cover(explain, &label, Some(cause))?;
        }
    }
    let selection_record = record_plan_selection(explain, &portfolio, frontier_cause)?;
    Ok(CompletePlans {
        lowering,
        portfolio,
        legality,
        numerical,
        rejections,
        selection_record,
    })
}

/// Records why one occurrence's lowering could not be established.
///
/// The three classes stay distinct. An absent capability is a deferred
/// capability: the installed authority was never extended to this occurrence. A
/// contended capability is a disproved checked predicate: two extensions
/// contradict each other, which is a defect in the installed authority rather
/// than a gap in it. A refused refinement is a disproved refinement predicate at
/// the kernel stage: a provider was resolved, drove the canonical builder, and
/// the emitted region does not realize the occurrence.
fn record_lowering_failure(
    explain: &mut ExplainWriter,
    source: &LoweringError,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = format!("occurrence:{}", source.member().0);
    let (stage, subject_kind) = match source {
        LoweringError::Refine { .. } => (ExplainStage::KernelRefinement, SubjectKind::Kernel),
        LoweringError::Occurrence { .. } | LoweringError::Resolve { .. } => {
            (ExplainStage::CapabilityResolution, SubjectKind::Capability)
        }
    };
    let reason = source.reason();
    let missing = source.is_missing();
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(subject_kind, &key)?;
            let event = if missing {
                ExplainEvent::DeferredCapability {
                    predicate: PredicateKey::new("capability.index-access-resolved")?,
                    reason: ReasonCode::new(reason)?,
                }
            } else {
                ExplainEvent::Check {
                    stage,
                    assessment: PredicateAssessment::disproved(
                        match stage {
                            ExplainStage::KernelRefinement => {
                                "kernel.index-region-refines-occurrence"
                            }
                            _ => "capability.index-access-resolved",
                        },
                        ReasonCode::new(reason)?,
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::IntrinsicInvalid,
                }
            };
            Ok(explain.push_detail(
                RuleRef::builtin("capability.index-access-resolution.v1")?,
                vec![subject],
                event,
                vec![cause],
            )?)
        })(),
        stage,
        subject_kind,
        &key,
        record_cause(cause),
    )
}

/// Attributes a lowering-stage failure to its exact phase and subject.
///
/// Resolution failures belong to [`ExplainStage::CapabilityResolution`] and
/// refinement refusals to [`ExplainStage::KernelRefinement`]; both are reported
/// as unsupported capabilities rather than as target infeasibility, because the
/// installed authority is what could not lower the program.
fn lowering_failure(source: &LoweringError, cause: ExplainRecordId) -> TargetFailure {
    let stage = match source {
        LoweringError::Refine { .. } => ExplainStage::KernelRefinement,
        LoweringError::Occurrence { .. } | LoweringError::Resolve { .. } => {
            ExplainStage::CapabilityResolution
        }
    };
    target_failure(
        CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
            phase: "lowering",
            rule: source.reason(),
        }),
        stage,
        format!("lowering-{}", source.reason()),
        SubjectKind::Capability,
        format!("occurrence:{}", source.member().0),
        record_cause(cause),
    )
}

/// Returns the planning ordinal a cover region's implementation will carry.
///
/// The ordinal is presentation only; a rejected proposal has no verified region,
/// so the region subject's position in the cover is the stable coordinate to
/// attribute the rejection to.
fn region_id_of(
    cover: &RegionCover,
    region: &crate::cover::CoverRegion,
) -> crate::physical::RegionId {
    let position = cover
        .regions()
        .iter()
        .position(|candidate| candidate.occurrence() == region.occurrence())
        .unwrap_or(0);
    crate::physical::RegionId::new(u32::try_from(position).unwrap_or(u32::MAX))
}

/// Assembles one retained complete plan into KIR, a kernel program, and a plan.
fn build_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    plan: &SelectedPlan,
    kind: ProgramAlternativeKind,
    plans: &CompletePlans,
    cause: Option<&TerminalCause>,
) -> Result<ProgramAlternative, TargetFailure> {
    let CompletePlans {
        lowering,
        legality,
        numerical,
        ..
    } = plans;
    let scheduled = plan_regions(plan);
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause.copied())
        })?;
    let program = build_plan_program(semantic, verified, kind, &scheduled).map_err(|error| {
        failure_at_source(error, ExplainStage::ProgramVerification, cause.copied())
    })?;
    assert_kernels_match_program(verified, &scheduled, &program, &kernels).map_err(|error| {
        failure_at_source(
            error.into(),
            ExplainStage::ProgramVerification,
            cause.copied(),
        )
    })?;
    let artifact_plan = build_artifact_plan(
        semantic,
        verified,
        &scheduled,
        &kernels,
        &program,
        lowering.providers(),
    )
    .map_err(|error| {
        failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause.copied())
    })?;
    let equivalence = EquivalenceEvidence {
        legality: plan
            .cover()
            .regions()
            .iter()
            .enumerate()
            .filter_map(|(position, region)| {
                legality
                    .get(region.occurrence())
                    .map(|proof| (position, proof.clone()))
            })
            .collect(),
        numerical: match kind {
            ProgramAlternativeKind::Fused => numerical.clone(),
            ProgramAlternativeKind::Materialized => None,
        },
    };
    Ok(ProgramAlternative {
        stable_id: plan.identity().label(),
        kind,
        plan: plan.clone(),
        scheduled_regions: scheduled,
        kernels,
        program,
        artifact_plan,
        structural_cost: plan.cost(),
        equivalence,
    })
}

/// Returns a plan's verified scheduled regions in ascending planning order.
///
/// A plan's selections are in canonical occurrence order, which is content
/// derived rather than execution ordered. Downstream program assembly consumes
/// producers before consumers, so the regions are ordered by the planning ordinal
/// the request-subject binding already pinned for each recognized region.
fn plan_regions(plan: &SelectedPlan) -> Vec<VerifiedScheduledRegion> {
    let mut regions: Vec<VerifiedScheduledRegion> = plan
        .selections()
        .iter()
        .map(|selection| selection.implementation().verified().clone())
        .collect();
    regions.sort_by_key(|region| region.region().index.id.get());
    regions
}

/// Assembles the verified kernel program for one plan shape.
///
/// The bounded profile implements exactly two program shapes: a one-region fused
/// program and a two-region materialized program. Any other retained plan shape
/// is invalid compiler output and rejects explicitly rather than being
/// approximated by the closest implemented assembly.
fn build_plan_program(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    kind: ProgramAlternativeKind,
    scheduled: &[VerifiedScheduledRegion],
) -> Result<KernelProgram, CompileError> {
    match (kind, scheduled) {
        (ProgramAlternativeKind::Fused, [region]) => {
            build_fused_kernel_program(semantic, verified, region).map_err(CompileError::from)
        }
        (ProgramAlternativeKind::Materialized, [_, _]) => {
            build_kernel_program(semantic, verified, scheduled).map_err(CompileError::from)
        }
        _ => Err(CompileError::from(ProgramError::Structure {
            rule: "unsupported-plan-shape",
        })),
    }
}

/// Returns the identity of the first structurally non-dominated alternative.
///
/// Domination is the Pareto relation the selection authority already computed
/// over exact structural counts; it is never a scalar latency total order. When
/// several plans are mutually non-dominated the canonical identity order breaks
/// the tie deterministically, so the choice is reproducible without inventing a
/// preference between incomparable trade-offs.
fn select_non_dominated(
    portfolio: &SelectedPortfolio,
    alternatives: &[ProgramAlternative],
) -> Result<String, CompileError> {
    let retained = portfolio.non_dominated();
    let selected = retained
        .iter()
        .map(|plan| plan.identity().label())
        .find(|label| {
            alternatives
                .iter()
                .any(|alternative| &alternative.stable_id == label)
        });
    selected.ok_or(CompileError::InvalidCompilerOutput(
        CompilerOutputError::Program(ProgramError::Structure {
            rule: "portfolio-empty",
        }),
    ))
}

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive match keeps every compile-error class beside the exact reason code, subject kind, and subject key it is attributed to"
)]
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
            format!("failed-region:{}", region.get()),
        ),
        // The reason names the dimension and the required behaviour, not a pair
        // of numbers: `numerics-input-subnormals-preserve` is what replaces
        // `target-strict-f32`, and it is readable without the profile in hand.
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Numerical {
            cause,
            region,
        }))
        | CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Numerical { cause, region },
        )) => (
            format!(
                "{}-{}",
                cause.dimension().key().replace('.', "-"),
                cause.required().key()
            ),
            SubjectKind::Region,
            format!("failed-region:{}", region.get()),
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
            format!("failed-region:{}", region.get()),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::Refinement { rule, region },
        )) => (
            format!("refinement-{rule}"),
            SubjectKind::Kernel,
            format!("failed-region:{}", region.get()),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Program(error)) => {
            program_failure_details(error)
        }
        CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
            PhysicalError::ShapeProductOverflow { region },
        )) => (
            "shape-product-overflow".to_owned(),
            SubjectKind::Region,
            format!("failed-region:{}", region.get()),
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
        CompileError::InvalidCompilerOutput(CompilerOutputError::Cover(error)) => (
            format!("cover-{}-{}", error.class(), error.reason()),
            SubjectKind::Candidate,
            "region-cover".to_owned(),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::FusionLegality(error)) => (
            format!("fusion-legality-{}", error.reason()),
            SubjectKind::Candidate,
            "fusion-legality".to_owned(),
        ),
        CompileError::InvalidCompilerOutput(CompilerOutputError::Frontier(error)) => (
            format!("frontier-{}", error.reason()),
            SubjectKind::Schedule,
            "implementation-frontier".to_owned(),
        ),
        CompileError::NoFeasiblePlan(NoFeasiblePlanError::Selection(error))
        | CompileError::InvalidCompilerOutput(CompilerOutputError::Selection(error)) => (
            format!("selection-{}-{}", error.class(), error.reason()),
            SubjectKind::KernelProgram,
            "portfolio".to_owned(),
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
        RequestError::UnstatedNumericalContract => "numerics-contract-unstated".to_owned(),
        // The reason names the first stated entry's first failing dimension. The
        // whole per-entry list is on the error's `Display`; a reason code is one
        // stable token, and truncating it to the caller's first choice keeps it
        // stable as the list grows.
        RequestError::NoResolvableNumericalContract { rejections, .. } => {
            rejections.first().map_or_else(
                || "numerics-contract-unresolvable".to_owned(),
                |rejection| {
                    format!(
                        "numerics-unhonourable-{}-{}",
                        rejection.dimension().key().replace('.', "-"),
                        rejection.required().key()
                    )
                },
            )
        }
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
        ProgramError::Storage { rule } => (format!("storage-{rule}"), "kernel-program".to_owned()),
        ProgramError::Abi { rule, stage } => {
            (format!("abi-{rule}"), format!("stage:{}", stage.index()))
        }
        ProgramError::Routing { rule } => (format!("routing-{rule}"), "kernel-program".to_owned()),
        // A shared kernel-program rejection is invalid compiler output; the
        // stable rule identifier comes from the layer that rejected it.
        ProgramError::CoreConstruction(_) | ProgramError::CoreVerification(_) => (
            format!("core-{}", error.rule()),
            "kernel-program".to_owned(),
        ),
    };
    (reason, SubjectKind::KernelProgram, subject)
}

fn explain_error_reason(error: &ExplainError) -> &'static str {
    match error {
        ExplainError::InvalidKey { .. } => "invalid-key",
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
        ExplainError::DetailCapacity => "detail-capacity",
        ExplainError::TerminalCapacity => "terminal-capacity",
        ExplainError::EmptyTrace => "empty-trace",
        ExplainError::StaleIdentity => "stale-identity",
    }
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
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(subject_kind, subject_key)?;
            Ok(explain.push_detail(
                RuleRef::builtin(rule)?,
                vec![subject],
                check_with_count(stage, predicate, fact, count)?,
                vec![cause],
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

/// Records the bounded cover enumeration, its budget stops, and infeasibility.
fn record_cover_enumeration(
    explain: &mut ExplainWriter,
    enumeration: &CoverEnumeration,
    root: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = record_count_step(
        explain,
        "cover.enumeration.v1",
        SubjectKind::Candidate,
        "region-cover",
        ExplainStage::CandidateEnumeration,
        "cover.complete-and-legal",
        "cover-count",
        enumeration.covers().len(),
        root,
    )?;
    for stop in enumeration.budget_stops() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Candidate, "region-cover")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("cover.enumeration.v1")?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::CandidateEnumeration,
                        resource: crate::explain::ResourceKey::new(stop.resource.key())?,
                        limit: stop.limit,
                        actual: stop.actual,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Candidate,
            "region-cover",
            record_cause(cause),
        )?;
    }
    for infeasibility in enumeration.infeasibilities() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Candidate, "region-cover")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("cover.enumeration.v1")?,
                    vec![subject],
                    ExplainEvent::DeferredCapability {
                        predicate: PredicateKey::new("cover.complete-and-legal")?,
                        reason: ReasonCode::new(infeasibility.reason())?,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::Candidate,
            "region-cover",
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

/// Records one region's typed fusion-legality outcome.
///
/// A legal region is an admitted check attributed to the capability provider that
/// declared the member operations' fusion roles; a rejection is a disproved
/// numerical-legality check, and an unknown is a deferred capability. The three
/// stay distinct classes rather than collapsing into one "not fused" verdict.
fn record_fusion_legality(
    explain: &mut ExplainWriter,
    capabilities: &FusionNumericalCapabilities,
    candidate: &RegionCandidate,
    outcome: &FusionLegality,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = candidate.label().to_owned();
    explain_step(
        (|| -> Result<_, CompileError> {
            let provider = ProviderRef::registered(capabilities.provider())?;
            let rule = RuleRef::provided("fusion.legality.v1", capabilities.revision(), provider)?;
            let subject = explain.subject(SubjectKind::Candidate, &key)?;
            let event = match outcome {
                FusionLegality::Legal(_) => ExplainEvent::Check {
                    stage: ExplainStage::NumericalLegality,
                    assessment: PredicateAssessment::proven(
                        "fusion.obligations-discharged",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::NumericalIllegal,
                },
                FusionLegality::Rejected(rejection) => ExplainEvent::Check {
                    stage: ExplainStage::NumericalLegality,
                    assessment: PredicateAssessment::disproved(
                        "fusion.obligations-discharged",
                        ReasonCode::new(rejection.reason())?,
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    rejection: RejectionClass::NumericalIllegal,
                },
                FusionLegality::Unknown(unknown) => ExplainEvent::DeferredCapability {
                    predicate: PredicateKey::new("fusion.obligations-discharged")?,
                    reason: ReasonCode::new(unknown.reason())?,
                },
            };
            Ok(explain.push_detail(rule, vec![subject], event, vec![cause])?)
        })(),
        ExplainStage::NumericalLegality,
        SubjectKind::Candidate,
        &key,
        record_cause(cause),
    )
}

/// Records every recognized occurrence's resolved capability and its evidence.
///
/// Two records per occurrence at most, and they are deliberately different
/// classes. The [`ExplainStage::CapabilityResolution`] record is an admitted
/// checked invariant attributed to the resolved provider: the installed registry
/// resolved exactly one index-access capability for this occurrence. The
/// [`ExplainStage::KernelRefinement`] record is either the exhaustive finite
/// evidence that the provider's region realizes the occurrence, or — when the
/// exhaustive access proof could not afford the region — a typed budget stop
/// naming the resource, its limit, and the required amount, plus an explicit
/// `Unknown` assessment. The budget stop is never a rejection: nothing about the
/// region was disproved, the analysis stopped.
fn record_lowering(
    explain: &mut ExplainWriter,
    lowering: &ResolvedLowering,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    for occurrence in lowering.occurrences() {
        let key = occurrence.subject_key();
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let provider = ProviderRef::lowering(occurrence.provider())?;
                let rule = RuleRef::provided(
                    "capability.index-access-resolution.v1",
                    occurrence.provider().capability_revision().get(),
                    provider,
                )?;
                let subject = explain.subject(SubjectKind::Capability, &key)?;
                Ok(explain.push_detail(
                    rule,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::CapabilityResolution,
                        assessment: PredicateAssessment::proven(
                            "capability.index-access-resolved",
                            EvidenceBasis::CheckedInvariant,
                        )?,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CapabilityResolution,
            SubjectKind::Capability,
            &key,
            record_cause(cause),
        )?;
        cause = record_refinement(explain, occurrence, cause)?;
    }
    Ok(cause)
}

/// Records one occurrence's refinement evidence or its recorded proof gap.
fn record_refinement(
    explain: &mut ExplainWriter,
    occurrence: &crate::lowering::OccurrenceLowering,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = occurrence.subject_key();
    match occurrence.evidence() {
        OccurrenceEvidence::Refined(refinement) => {
            let identity = refinement_label(refinement);
            explain_step(
                (|| -> Result<_, CompileError> {
                    let provider = ProviderRef::lowering(occurrence.provider())?;
                    let rule = RuleRef::provided(
                        "kernel.index-region-refinement.v1",
                        occurrence.provider().capability_revision().get(),
                        provider,
                    )?;
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        rule,
                        vec![subject],
                        ExplainEvent::Check {
                            stage: ExplainStage::KernelRefinement,
                            assessment: PredicateAssessment::proven(
                                "kernel.index-region-refines-occurrence",
                                EvidenceBasis::ExhaustiveFinite,
                            )?
                            .with_fact(ExplainFact::new(
                                "refinement-identity",
                                FactValue::Identity(crate::explain::SubjectKey::new(identity)?),
                            )?)?,
                            rejection: RejectionClass::IntrinsicInvalid,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )
        }
        OccurrenceEvidence::BudgetStopped(stop) => {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin("kernel.index-region-refinement.v1")?,
                        vec![subject],
                        ExplainEvent::BudgetStop {
                            stage: ExplainStage::KernelRefinement,
                            resource: crate::explain::ResourceKey::new(stop.resource_key())?,
                            limit: stop.limit,
                            actual: u64::try_from(stop.required).unwrap_or(u64::MAX),
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )?;
            explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Kernel, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin("kernel.index-region-refinement.v1")?,
                        vec![subject],
                        ExplainEvent::Check {
                            stage: ExplainStage::KernelRefinement,
                            assessment: PredicateAssessment::unknown(
                                "kernel.index-region-refines-occurrence",
                                ReasonCode::new("proof-budget-exhausted")?,
                            )?,
                            rejection: RejectionClass::IntrinsicInvalid,
                        },
                        vec![cause],
                    )?)
                })(),
                ExplainStage::KernelRefinement,
                SubjectKind::Kernel,
                &key,
                record_cause(cause),
            )
        }
    }
}

/// Returns the stable presentation label of one refinement occurrence identity.
///
/// The label is a presentation handle over the identity's trailing bytes, never
/// the identity itself: the canonical bytes stay in the retained
/// [`crate::legality::IndexRefinement`], which is what any downstream check
/// compares.
fn refinement_label(refinement: &crate::legality::IndexRefinement) -> String {
    use std::fmt::Write as _;

    let bytes = refinement.identity().as_bytes();
    let tail = bytes.len().saturating_sub(8);
    let mut label = String::from("refinement:");
    for byte in &bytes[tail..] {
        let _ = write!(label, "{byte:02x}");
    }
    label
}

/// Records the whole-program strict-`f32` numerical equivalence sound proof.
///
/// The proof is attributed to the provider that lowers the reduction occurrence,
/// because that is the operation whose reassociation the proof forbids. A
/// program with no recognized reduction has no fused equivalence claim to make.
fn record_numerical_equivalence(
    explain: &mut ExplainWriter,
    verified: &crate::request::VerifiedTargetRequest,
    lowering: &ResolvedLowering,
    candidate: &RegionCandidate,
    proof: &FusionNumericalProof,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = candidate.label().to_owned();
    explain_step(
        (|| -> Result<_, CompileError> {
            let reduction = verified.serial_sum().members.reduction();
            let provider = lowering
                .occurrences()
                .iter()
                .find(|occurrence| reduction.contains(&occurrence.member()))
                .map(crate::lowering::OccurrenceLowering::provider)
                .ok_or_else(|| {
                    CompileError::from(ProgramError::Structure {
                        rule: "reduction-provider-missing",
                    })
                })?;
            let provider_ref = ProviderRef::lowering(provider)?;
            let subject = explain.subject(SubjectKind::Candidate, &key)?;
            Ok(explain.push_detail(
                RuleRef::provided("fusion.strict-f32-equivalence", 1, provider_ref.clone())?,
                vec![subject],
                check(
                    ExplainStage::NumericalLegality,
                    "fusion.strict-f32-equivalence",
                    EvidenceBasis::SoundProof(VerifiedEvidenceRef::from_fusion_numerical(
                        verified,
                        proof,
                        provider_ref,
                    )?),
                )?,
                vec![cause],
            )?)
        })(),
        ExplainStage::NumericalLegality,
        SubjectKind::Candidate,
        &key,
        record_cause(cause),
    )
}

/// Records one region subject's bounded implementation frontier.
fn record_frontier(
    explain: &mut ExplainWriter,
    role: &'static str,
    frontier: &crate::frontier::ImplementationFrontier,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let key = format!("region:{role}");
    record_count_step(
        explain,
        "frontier.enumeration.v1",
        SubjectKind::Schedule,
        &key,
        ExplainStage::IntrinsicScheduling,
        "frontier.locally-feasible",
        "admitted-count",
        frontier.admitted().len(),
        cause,
    )
}

/// Records the complete-plan join: how many valid plans the portfolio retained.
fn record_plan_selection(
    explain: &mut ExplainWriter,
    portfolio: &SelectedPortfolio,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut cause = record_count_step(
        explain,
        "selection.complete-plan.v1",
        SubjectKind::KernelProgram,
        "portfolio",
        ExplainStage::CandidateEnumeration,
        "selection.plans-complete-and-composed",
        "plan-count",
        portfolio.plans().len(),
        cause,
    )?;
    for stop in portfolio.budget_stops() {
        cause = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::KernelProgram, "portfolio")?;
                Ok(explain.push_detail(
                    RuleRef::builtin("selection.complete-plan.v1")?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::CandidateEnumeration,
                        resource: crate::explain::ResourceKey::new(stop.resource.key())?,
                        limit: stop.limit,
                        actual: stop.actual,
                    },
                    vec![cause],
                )?)
            })(),
            ExplainStage::CandidateEnumeration,
            SubjectKind::KernelProgram,
            "portfolio",
            record_cause(cause),
        )?;
    }
    Ok(cause)
}

/// Records one region subject's hard-infeasible target rejection.
///
/// A capability rejection keeps the quantitative feasibility record; a numerical
/// one takes the honourability record, which is the only shape that can carry a
/// dimension, a required behaviour, a declared means, an honoured alternative,
/// and a declaring profile.
fn record_target_rejection(
    explain: &mut ExplainWriter,
    error: &PhysicalError,
    role: &'static str,
    cause: ExplainRecordId,
) -> Result<TerminalCause, TargetFailure> {
    let key = format!("region:{role}");
    let (rule_key, event) = match error {
        PhysicalError::Target {
            rule,
            required,
            available,
            ..
        } => (
            format!("target.{rule}"),
            (|| -> Result<_, CompileError> {
                Ok(ExplainEvent::Feasibility {
                    predicate: PredicateKey::new(*rule)?,
                    outcome: crate::explain::FeasibilityOutcome::Rejected(ReasonCode::new(
                        "target-infeasible",
                    )?),
                    required: target_quantity(rule, *required)?,
                    available: target_quantity(rule, *available)?,
                })
            })(),
        ),
        PhysicalError::Numerical { cause, .. } => (
            format!("target.{}", cause.dimension().key()),
            (|| -> Result<_, CompileError> {
                Ok(ExplainEvent::NumericalHonourability {
                    dimension: PredicateKey::new(cause.dimension().key())?,
                    required: ReasonCode::new(cause.required().key())?,
                    outcome: crate::explain::HonourabilityOutcome::Unhonourable {
                        means: ReasonCode::new(cause.means().key())?,
                        honoured: cause
                            .honoured()
                            .map(|honoured| ReasonCode::new(honoured.key()))
                            .transpose()?,
                    },
                    profile: crate::explain::SubjectKey::new(cause.profile().key())?,
                })
            })(),
        ),
        PhysicalError::Intrinsic { .. }
        | PhysicalError::Refinement { .. }
        | PhysicalError::ShapeProductOverflow { .. } => {
            unreachable!("target rejection records require a target-feasibility error")
        }
    };
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Region, &key)?;
            Ok(explain.push_causal_detail(
                RuleRef::builtin(rule_key)?,
                subject,
                &event?,
                vec![cause],
            )?)
        })(),
        ExplainStage::TargetFeasibility,
        SubjectKind::Region,
        &key,
        record_cause(cause),
    )
}

/// Notes one cover as an infeasible alternative in the terminal ledger.
fn note_infeasible_cover(
    explain: &mut ExplainWriter,
    label: &str,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Alternative, label)?;
            explain.note_infeasible_alternative(subject, cause)?;
            Ok(())
        })(),
        ExplainStage::Selection,
        SubjectKind::Alternative,
        label,
        cause,
    )
}

fn record_target_admissions(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    mut cause: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
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
        let key = format!("{}/region:{}", alternative.stable_id, region.index.id.get());
        for predicate in admitted.predicates() {
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
                        vec![cause],
                    )?)
                })(),
                ExplainStage::TargetFeasibility,
                SubjectKind::Region,
                &key,
                record_cause(cause),
            )?;
        }
        // The admitted trace records the *means* of each honoured dimension, not
        // only that it was honoured. An emulated dimension is admitted and emits
        // different operations, so a trace that carried only the verdict would
        // leave a reader unable to tell one from native support.
        for honoured in admitted.honoured() {
            cause = explain_step(
                (|| -> Result<_, CompileError> {
                    let subject = explain.subject(SubjectKind::Region, &key)?;
                    Ok(explain.push_detail(
                        RuleRef::builtin(format!("target.{}", honoured.dimension().key()))?,
                        vec![subject],
                        ExplainEvent::NumericalHonourability {
                            dimension: PredicateKey::new(honoured.dimension().key())?,
                            required: ReasonCode::new(honoured.behaviour().key())?,
                            outcome: crate::explain::HonourabilityOutcome::Honoured {
                                means: ReasonCode::new(honoured.means().key())?,
                            },
                            profile: crate::explain::SubjectKey::new(honoured.profile().key())?,
                        },
                        vec![cause],
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
        "index-bits" | "device-memory" | "barriers" => Ok(Quantity::Count(value)),
        _ => Err(ExplainError::UnknownQuantityUnit),
    }
}

/// Records one retained alternative's per-layer admitted evidence.
fn record_alternative_explain(
    explain: &mut ExplainWriter,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    root: ExplainRecordId,
) -> Result<ExplainRecordId, TargetFailure> {
    let mut boundary_causes = Vec::new();
    for scheduled in &alternative.scheduled_regions {
        let region_id = scheduled.region().index.id.get();
        let key = format!("{}/region:{region_id}", alternative.stable_id);
        let record = explain_step(
            (|| -> Result<_, CompileError> {
                let subject = explain.subject(SubjectKind::Region, &key)?;
                Ok(explain.push_detail(
                    RuleRef::provided(
                        "compile.region.verified",
                        1,
                        ProviderRef::registered(&GovernedPhysicalProvider::identity())?,
                    )?,
                    vec![subject],
                    check(
                        ExplainStage::RegionFormation,
                        "region.semantic-coverage",
                        EvidenceBasis::CheckedInvariant,
                    )?,
                    vec![root],
                )?)
            })(),
            ExplainStage::RegionFormation,
            SubjectKind::Region,
            &key,
            record_cause(root),
        )?;
        boundary_causes.push(record);
    }
    let key = format!("{}/boundary", alternative.stable_id);
    let boundary = explain_step(
        (|| -> Result<_, CompileError> {
            let subject = explain.subject(SubjectKind::Boundary, &key)?;
            Ok(explain.push_detail(
                RuleRef::builtin("compile.plan.boundary")?,
                vec![subject],
                check_with_count(
                    ExplainStage::RegionFormation,
                    "boundary.handoffs-satisfied",
                    "handoff-count",
                    alternative.plan.handoffs().len(),
                )?,
                boundary_causes,
            )?)
        })(),
        ExplainStage::RegionFormation,
        SubjectKind::Boundary,
        &key,
        record_cause(root),
    )?;
    let key = format!("{}/schedules", alternative.stable_id);
    let schedule = record_count_step(
        explain,
        "schedule.plan-regions",
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
        "kernel.plan-refinement",
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
        "program.plan-verified",
        SubjectKind::KernelProgram,
        &key,
        ExplainStage::ProgramVerification,
        "program.verified",
        "stage-count",
        alternative.program.stage_count(),
        kernel,
    )?;
    let key = format!("{}/artifact", alternative.stable_id);
    record_count_step(
        explain,
        "artifact.plan-construction",
        SubjectKind::ArtifactPlan,
        &key,
        ExplainStage::ArtifactPlanning,
        "artifact.plan-verified",
        "provider-count",
        alternative.artifact_plan.lowering_providers().len(),
        program,
    )
}

fn record_cost_and_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative_id: &str,
    causes: &[(String, ExplainRecordId)],
    explain: &mut ExplainWriter,
) -> Result<(), TargetFailure> {
    for alternative in alternatives {
        let cost = alternative.structural_cost;
        let cause = causes
            .iter()
            .find_map(|(id, cause)| (*id == alternative.stable_id).then_some(*cause));
        let (subject, cost_record) = explain_step(
            (|| -> Result<_, CompileError> {
                let subject =
                    explain.subject(SubjectKind::Alternative, alternative.stable_id.as_str())?;
                let terms = vec![
                    CostTerm::new("dispatch-count", Quantity::Count(cost.dispatch_count()))?,
                    CostTerm::new(
                        "launched-threads",
                        Quantity::Threads(cost.launched_threads()),
                    )?,
                    CostTerm::new("temporary-bytes", Quantity::Bytes(cost.temporary_bytes()))?,
                    CostTerm::new(
                        "materialization-count",
                        Quantity::Count(cost.materialization_count()),
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
            alternative.stable_id.as_str(),
            cause.map(TerminalCause::from_record),
        )?;
        let outcome = if alternative.stable_id == selected_alternative_id {
            SelectionOutcome::Selected
        } else if alternatives
            .iter()
            .find(|item| item.stable_id == selected_alternative_id)
            .is_some_and(|selected| {
                selected
                    .structural_cost
                    .dominates(&alternative.structural_cost)
            })
        {
            SelectionOutcome::Dominated
        } else {
            SelectionOutcome::NotSelectedTradeoff
        };
        explain_step(
            explain
                .note_selection(subject, outcome, Some(cost_record))
                .map_err(Into::into),
            ExplainStage::Selection,
            SubjectKind::Alternative,
            alternative.stable_id.as_str(),
            Some(cost_record),
        )?;
    }
    Ok(())
}

/// Re-derives the retained portfolio from the program and its own contents.
///
/// The complete-plan authority re-verifies every plan's cover and re-assembles
/// each plan from its selections; this additionally re-derives each alternative's
/// KIR, kernel program, and artifact plan and requires them to reproduce the
/// receipt exactly. A tampered plan, cost, program, or artifact receipt therefore
/// fails closed instead of being carried into a compilation product.
fn verify_portfolio(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    portfolio: &SelectedPortfolio,
    alternatives: &[ProgramAlternative],
    selected_id: &str,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    verify_selected_portfolio(
        semantic,
        request.budgets(),
        request.numerical_contract(),
        portfolio,
    )
    .map_err(|source| failure_at_source(source.into(), ExplainStage::Selection, cause))?;
    if alternatives.is_empty()
        || alternatives
            .iter()
            .map(|alternative| alternative.stable_id.as_str())
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
        verify_alternative(semantic, request, alternative, cause)?;
    }
    let recomputed = select_non_dominated(portfolio, alternatives)
        .map_err(|source| failure_at_source(source, ExplainStage::Selection, cause))?;
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

/// Re-derives one alternative's structured, program, and artifact layers.
fn verify_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
    cause: Option<TerminalCause>,
) -> Result<(), TargetFailure> {
    if alternative.stable_id != alternative.plan.identity().label()
        || alternative.structural_cost != alternative.plan.cost()
        || alternative.kind
            != ProgramAlternativeKind::of(
                alternative.plan.cover(),
                total_members(&alternative.plan),
            )
    {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-cost-or-identity",
            }
            .into(),
            ExplainStage::Costing,
            cause,
        ));
    }
    let scheduled = plan_regions(&alternative.plan);
    if alternative.scheduled_regions != scheduled {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-schedule-binding",
            }
            .into(),
            ExplainStage::IntrinsicScheduling,
            cause,
        ));
    }
    let kernels = scheduled
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            let stage = physical_error_stage(&error);
            failure_at_source(error.into(), stage, cause)
        })?;
    if alternative.kernels != kernels {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-kernel-binding",
            }
            .into(),
            ExplainStage::KernelRefinement,
            cause,
        ));
    }
    let program = build_plan_program(semantic, request, alternative.kind, &scheduled)
        .map_err(|error| failure_at_source(error, ExplainStage::ProgramVerification, cause))?;
    if alternative.program != program {
        return Err(failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-program-binding",
            }
            .into(),
            ExplainStage::ProgramVerification,
            cause,
        ));
    }
    // The plan's own recorded provenance is checked against the request's
    // installed registry rather than against itself, so a receipt naming a
    // provider the registry never resolved fails closed here.
    let providers = crate::lowering::resolve_capabilities(semantic, request).map_err(|_| {
        failure_at_source(
            ProgramError::Structure {
                rule: "portfolio-provider-resolution",
            }
            .into(),
            ExplainStage::CapabilityResolution,
            cause,
        )
    })?;
    verify_artifact_plan(
        &alternative.artifact_plan,
        semantic,
        request,
        &scheduled,
        &kernels,
        &program,
        providers,
    )
    .map_err(|error| failure_at_source(error.into(), ExplainStage::ArtifactPlanning, cause))?;
    verify_equivalence(semantic, request, alternative)
        .map_err(|source| failure_at_source(source, ExplainStage::NumericalLegality, cause))
}

/// Returns the number of semantic occurrences a plan's cover covers.
fn total_members(plan: &SelectedPlan) -> u32 {
    u32::try_from(
        plan.cover()
            .regions()
            .iter()
            .map(|region| region.members().len())
            .sum::<usize>(),
    )
    .unwrap_or(u32::MAX)
}

/// Replays every retained numerical-equivalence and fusion-legality proof.
fn verify_equivalence(
    semantic: &tiler_ir::semantic::SemanticProgram,
    request: &crate::request::VerifiedTargetRequest,
    alternative: &ProgramAlternative,
) -> Result<(), CompileError> {
    let formation =
        form_region_candidates(semantic, request.budgets(), request.numerical_contract())?;
    let capabilities = FusionNumericalCapabilities::governed();
    // Every multi-occurrence region must carry exactly one replayable legality
    // proof; a fused region without one would be an unproven fusion.
    let expected: Vec<usize> = alternative
        .plan
        .cover()
        .regions()
        .iter()
        .enumerate()
        .filter_map(|(position, region)| (region.members().len() > 1).then_some(position))
        .collect();
    if alternative
        .equivalence
        .legality
        .iter()
        .map(|(position, _)| *position)
        .collect::<Vec<_>>()
        != expected
    {
        return Err(ProgramError::Structure {
            rule: "portfolio-equivalence",
        }
        .into());
    }
    for (position, proof) in &alternative.equivalence.legality {
        let region =
            alternative
                .plan
                .cover()
                .regions()
                .get(*position)
                .ok_or(ProgramError::Structure {
                    rule: "portfolio-equivalence",
                })?;
        let candidate = formation
            .candidates()
            .iter()
            .find(|candidate| candidate.occurrence() == region.occurrence())
            .ok_or(ProgramError::Structure {
                rule: "portfolio-equivalence",
            })?;
        verify_fusion_legality(
            semantic,
            request.budgets(),
            request.numerical_contract(),
            &capabilities,
            candidate,
            proof,
        )?;
    }
    match (
        alternative.kind,
        alternative.equivalence.numerical.as_deref(),
    ) {
        (ProgramAlternativeKind::Materialized, None) => Ok(()),
        (ProgramAlternativeKind::Fused, Some(proof)) => {
            let candidate = formation.whole_program_candidate().ok_or({
                CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                    ProgramError::Structure {
                        rule: "portfolio-fused-region",
                    },
                ))
            })?;
            verify_fused_numerics(formation.graph(), request, candidate, proof)?;
            if alternative.scheduled_regions.len() != 1
                || alternative.scheduled_regions[0].semantic_members() != candidate.members()
            {
                return Err(ProgramError::Structure {
                    rule: "portfolio-candidate-schedule-binding",
                }
                .into());
            }
            Ok(())
        }
        _ => Err(ProgramError::Structure {
            rule: "portfolio-equivalence",
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A retained root record the stage chain hangs from.
    fn test_root(explain: &mut ExplainWriter) -> ExplainRecordId {
        let subject = explain
            .subject(SubjectKind::SemanticProgram, "semantic-program")
            .unwrap();
        explain
            .push_detail(
                RuleRef::builtin("test.root").unwrap(),
                vec![subject],
                check(
                    ExplainStage::RequestVerification,
                    "test.root",
                    EvidenceBasis::CheckedInvariant,
                )
                .unwrap(),
                Vec::new(),
            )
            .unwrap()
    }
    use crate::explain::ExplainDisposition;
    use crate::physical::{RegionId, TensorRole};
    use crate::request::CompilerCapabilitySnapshot;
    use std::collections::BTreeMap;
    use tiler_ir::kernel::{BinaryOp, CompareOp, ConvertOp, KernelConstant, OperationView};
    use tiler_ir::program::{DependencyReasonView, ValueRole};
    use tiler_ir::semantic::{
        CANONICAL_F32_ARITHMETIC_NAN_BITS, F32, F32Add, F32Constant, F32Multiply, InputKey,
        OutputKey, SemanticProgram, SemanticProgramBuilder, StrictSerialF32Sum,
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

    /// One typed value produced while interpreting a structured kernel.
    #[derive(Clone, Copy, Debug)]
    enum KirValue {
        Bool(bool),
        Index(u64),
        F32(f32),
    }

    impl KirValue {
        fn index(self) -> u64 {
            match self {
                Self::Index(value) => value,
                other => panic!("expected an index-typed value, found {other:?}"),
            }
        }
        fn float(self) -> f32 {
            match self {
                Self::F32(value) => value,
                other => panic!("expected an f32-typed value, found {other:?}"),
            }
        }
        fn boolean(self) -> bool {
            match self {
                Self::Bool(value) => value,
                other => panic!("expected a predicate value, found {other:?}"),
            }
        }
    }

    /// A backend-shaped interpreter that reads only the structured kernel IR.
    ///
    /// It resolves nothing from the semantic graph, the request, or the
    /// schedule: buffer roles and extents, addressing, predication, reduction
    /// order, and NaN canonicalization all come from the kernel itself. That is
    /// the property the KIR layer exists to guarantee, so exercising it against
    /// the reference evaluator is the end-to-end proof that a backend needs no
    /// graph-specific reconstruction.
    struct KirMachine<'a> {
        kernel: &'a VerifiedKernel,
        input: &'a [f32],
        output: Vec<f32>,
        values: BTreeMap<tiler_ir::kernel::VerifiedValueId, KirValue>,
    }

    impl<'a> KirMachine<'a> {
        fn run(kernel: &'a VerifiedKernel, input: &'a [f32]) -> Vec<f32> {
            let mut buffers = kernel.buffers();
            let read = buffers.next().expect("a read buffer parameter");
            let write = buffers.next().expect("a write buffer parameter");
            assert_eq!(read.access, tiler_ir::kernel::BufferAccess::Read);
            assert_eq!(write.access, tiler_ir::kernel::BufferAccess::Write);
            assert_eq!(input.len(), usize::try_from(read.element_count).unwrap());
            let outputs = usize::try_from(write.element_count).unwrap();
            let mut machine = KirMachine {
                kernel,
                input,
                output: vec![f32::NAN; outputs],
                values: BTreeMap::new(),
            };
            for invocation in 0..u64::try_from(outputs).unwrap() {
                machine.values.clear();
                machine.run_block(kernel.body(), invocation);
            }
            machine.output
        }

        fn run_block(&mut self, block: tiler_ir::kernel::BlockRef<'a>, invocation: u64) {
            for operation in block.operations() {
                let mut results = operation.results();
                match operation.view() {
                    OperationView::Builtin { .. } => {
                        self.define(&mut results, KirValue::Index(invocation));
                    }
                    OperationView::Constant { value } => {
                        let value = match value {
                            KernelConstant::Bool(flag) => KirValue::Bool(flag),
                            KernelConstant::Index(index) => KirValue::Index(index),
                            KernelConstant::F32Bits(bits) => KirValue::F32(f32::from_bits(bits)),
                            other => panic!("unsupported constant {other:?}"),
                        };
                        self.define(&mut results, value);
                    }
                    OperationView::Binary { op, lhs, rhs } => {
                        let value = match op {
                            BinaryOp::IndexAdd => {
                                KirValue::Index(self.get(lhs).index() + self.get(rhs).index())
                            }
                            BinaryOp::IndexMultiply => {
                                KirValue::Index(self.get(lhs).index() * self.get(rhs).index())
                            }
                            BinaryOp::IndexDivide => {
                                KirValue::Index(self.get(lhs).index() / self.get(rhs).index())
                            }
                            BinaryOp::IndexModulo => {
                                KirValue::Index(self.get(lhs).index() % self.get(rhs).index())
                            }
                            BinaryOp::F32Add => {
                                KirValue::F32(self.get(lhs).float() + self.get(rhs).float())
                            }
                            BinaryOp::F32Multiply => {
                                KirValue::F32(self.get(lhs).float() * self.get(rhs).float())
                            }
                            other => panic!("unsupported binary operation {other:?}"),
                        };
                        self.define(&mut results, value);
                    }
                    OperationView::Compare { op, lhs, rhs } => {
                        let value = match op {
                            CompareOp::IndexLessThan => {
                                KirValue::Bool(self.get(lhs).index() < self.get(rhs).index())
                            }
                            other => panic!("unsupported comparison {other:?}"),
                        };
                        self.define(&mut results, value);
                    }
                    OperationView::Convert { op, source } => {
                        let value = self.get(source).float();
                        let value = match op {
                            ConvertOp::CanonicalizeF32Nan => {
                                if value.is_nan() {
                                    f32::from_bits(
                                        self.kernel.numerical().canonical_arithmetic_nan_bits,
                                    )
                                } else {
                                    value
                                }
                            }
                            other => panic!("unsupported conversion {other:?}"),
                        };
                        self.define(&mut results, KirValue::F32(value));
                    }
                    OperationView::Load { offset, .. } => {
                        let offset = usize::try_from(self.get(offset).index()).unwrap();
                        let value = KirValue::F32(self.input[offset]);
                        self.define(&mut results, value);
                    }
                    OperationView::Store { offset, value, .. } => {
                        let offset = usize::try_from(self.get(offset).index()).unwrap();
                        self.output[offset] = self.get(value).float();
                    }
                    OperationView::Predicated { predicate, body } => {
                        if self.get(predicate).boolean() {
                            self.run_block(body, invocation);
                        }
                    }
                    OperationView::SerialLoop(reduction) => {
                        let mut carried: Vec<KirValue> =
                            reduction.initial().map(|value| self.get(value)).collect();
                        let induction = reduction.induction().expect("an induction variable");
                        let parameters: Vec<_> = reduction.accumulators().collect();
                        for step in reduction.start()..reduction.end() {
                            self.values.insert(induction, KirValue::Index(step));
                            for (parameter, value) in parameters.iter().zip(&carried) {
                                self.values.insert(*parameter, *value);
                            }
                            self.run_block(reduction.body(), invocation);
                            carried = reduction.yields().map(|value| self.get(value)).collect();
                        }
                        for (result, value) in results.zip(carried) {
                            self.values.insert(result, value);
                        }
                    }
                    OperationView::Barrier { .. } => {}
                    other => panic!("unsupported structured operation {other:?}"),
                }
            }
        }

        fn define(
            &mut self,
            results: &mut impl Iterator<Item = tiler_ir::kernel::VerifiedValueId>,
            value: KirValue,
        ) {
            let result = results.next().expect("one defined result");
            self.values.insert(result, value);
        }

        fn get(&self, id: tiler_ir::kernel::VerifiedValueId) -> KirValue {
            *self
                .values
                .get(&id)
                .expect("a value defined before its use")
        }
    }

    pub(super) fn interpret_fused(kernel: &VerifiedKernel, input: &[f32]) -> Vec<f32> {
        KirMachine::run(kernel, input)
    }

    /// Returns the bounded range of the kernel's single guarded reduction loop.
    pub(super) fn reduction_loop(kernel: &VerifiedKernel) -> Option<(u64, u64)> {
        kernel
            .body()
            .operations()
            .filter_map(|operation| match operation.view() {
                OperationView::Predicated { body, .. } => Some(body),
                _ => None,
            })
            .flat_map(tiler_ir::kernel::BlockRef::operations)
            .find_map(|operation| match operation.view() {
                OperationView::SerialLoop(reduction) => Some((reduction.start(), reduction.end())),
                _ => None,
            })
    }

    /// Returns the one retained alternative of the requested plan shape.
    fn alternative(
        product: &CompilationProduct,
        kind: ProgramAlternativeKind,
    ) -> &ProgramAlternative {
        let mut matching = product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .filter(|alternative| alternative.kind == kind);
        let found = matching
            .next()
            .unwrap_or_else(|| panic!("a retained {} alternative", kind.name()));
        assert!(
            matching.next().is_none(),
            "the bounded profile retains exactly one {} alternative",
            kind.name()
        );
        found
    }

    /// Returns the kind of the alternative the portfolio selected.
    fn selected_kind(product: &CompilationProduct) -> ProgramAlternativeKind {
        let target = &product.targets[0];
        target
            .portfolio
            .alternatives
            .iter()
            .find(|alternative| {
                alternative.stable_id == target.portfolio.selection.selected_alternative_id
            })
            .expect("the selected identity names a retained alternative")
            .kind
    }

    /// Counts every retained explain record by its stable rule key.
    fn rule_counts(trace: &VerifiedExplainTrace) -> BTreeMap<&str, usize> {
        trace
            .records()
            .iter()
            .fold(BTreeMap::new(), |mut counts, record| {
                *counts.entry(record.rule().key().as_str()).or_insert(0) += 1;
                counts
            })
    }

    fn assert_fused_matches_reference(
        shape: Shape,
        values: Vec<f32>,
        scale_bits: u32,
        bias_bits: u32,
    ) {
        assert_fused_axis_matches_reference(shape, values, scale_bits, bias_bits, Axis::new(1));
    }

    fn assert_fused_axis_matches_reference(
        shape: Shape,
        values: Vec<f32>,
        scale_bits: u32,
        bias_bits: u32,
        reduction_axis: Axis,
    ) {
        let semantic =
            semantic_case_with_axis(shape.clone(), scale_bits, bias_bits, false, reduction_axis);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let fused = alternative(&product, ProgramAlternativeKind::Fused);
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
        let target = &first.targets[0];
        let rendered = target.explain.render();
        assert!(rendered.starts_with("tiler-explain-v2 request="));
        assert!(rendered.contains("feasibility:threads-per-workgroup:admitted"));
        assert!(rendered.contains("feasibility:buffer-bindings:admitted"));
        assert!(rendered.contains("event=selection:tiler.selection.structural-pareto.v1:selected"));
        assert_eq!(target.portfolio.alternatives.len(), 2);
        assert_eq!(selected_kind(&first), ProgramAlternativeKind::Fused);
        let materialized = alternative(&first, ProgramAlternativeKind::Materialized);
        let fused = alternative(&first, ProgramAlternativeKind::Fused);
        assert_eq!(materialized.program.stage_count(), 2);
        let temporary = materialized
            .program
            .core()
            .values()
            .nth(1)
            .expect("the cross-stage temporary");
        assert_eq!(temporary.role(), ValueRole::Temporary);
        assert!(matches!(
            materialized
                .program
                .core()
                .dependencies()
                .next()
                .expect("one data dependency")
                .reason(),
            DependencyReasonView::Data(value) if value == temporary
        ));
        assert_eq!(
            materialized.kernels[0].buffers().nth(1).unwrap().tensor,
            TensorRole::Intermediate
        );
        assert_eq!(
            materialized.kernels[1].buffers().next().unwrap().tensor,
            TensorRole::Intermediate
        );
        assert_eq!(reduction_loop(&materialized.kernels[1]), Some((1, 3)));
        assert_eq!(fused.program.stage_count(), 1);
        assert_eq!(fused.program.core().values().len(), 2);
        // The exact aggregate structural cost is the sum of the per-region
        // estimates plus the cover's deliberate cross-region materializations.
        assert_eq!(materialized.structural_cost.dispatch_count(), 2);
        assert_eq!(materialized.structural_cost.launched_threads(), 8);
        assert_eq!(materialized.structural_cost.temporary_bytes(), 24);
        assert_eq!(materialized.structural_cost.materialization_count(), 1);
        assert_eq!(fused.structural_cost.dispatch_count(), 1);
        assert_eq!(fused.structural_cost.launched_threads(), 2);
        assert_eq!(fused.structural_cost.temporary_bytes(), 0);
        assert_eq!(fused.structural_cost.materialization_count(), 0);
        assert!(
            fused
                .structural_cost
                .dominates(&materialized.structural_cost)
        );
        // Lowering provenance is the set of providers the installed registry
        // resolved for the recognized occurrences. Both plan shapes cover the
        // same occurrences, so both name the same four governed providers: the
        // alternatives differ in their cover, not in who lowers each operation.
        // Provider and operation are named separately rather than one derived
        // from the other: they coincide by naming convention in the governed
        // registry, and a test that split the provider name would assert the
        // convention instead of the resolution.
        let expected_providers: Vec<_> = [
            ("governed-index-access.add-f32", "add-f32"),
            ("governed-index-access.constant-f32", "constant-f32"),
            ("governed-index-access.multiply-f32", "multiply-f32"),
            (
                "governed-index-access.strict-serial-sum-f32",
                "strict-serial-sum-f32",
            ),
        ]
        .into_iter()
        .map(|(provider, operation)| {
            crate::request::LoweringProviderIdentity::new(
                tiler_ir::semantic::ProviderIdentity::new("tiler", provider, 1).unwrap(),
                // The governed key names the capability family and the
                // operation it lowers, never the provider, which is recorded
                // beside it.
                format!("tiler.capability.index-access.tiler.{operation}.v1"),
                crate::capability::LoweringCapabilityRevision::new(1).unwrap(),
            )
        })
        .collect();
        assert_eq!(
            materialized.artifact_plan.lowering_providers(),
            expected_providers
        );
        assert_eq!(fused.artifact_plan.lowering_providers(), expected_providers);
        assert_eq!(reduction_loop(&fused.kernels[0]), Some((1, 3)));
        assert!(target.explain.records().iter().any(|record| {
            record.rule().key().as_str() == "compile.plan.boundary"
                && record.event().disposition() == ExplainDisposition::Admitted
        }));
        // The materialized plan discharges exactly one cross-region handoff; the
        // fused plan materializes nothing across a boundary.
        assert_eq!(materialized.plan.handoffs().len(), 1);
        assert!(fused.plan.handoffs().is_empty());
        // Both alternatives are the exact selected plans, so their stable
        // identity is the plan's content-derived identity label.
        for alternative in &target.portfolio.alternatives {
            assert_eq!(alternative.stable_id, alternative.plan.identity().label());
        }
    }

    /// Every draft authority the conformance gate wires must speak the explain
    /// vocabulary; a silent authority cannot be audited.
    #[test]
    fn every_wired_authority_emits_its_typed_explain_records() {
        let semantic = semantic(false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let trace = &product.targets[0].explain;
        // The exhaustive snapshot: every rule the wired compile path emits, and
        // exactly how many records each contributes. A new authority that stays
        // explain-silent, or one that becomes chatty, fails here.
        assert_eq!(
            rule_counts(trace),
            BTreeMap::from([
                ("compile.request.general-boundary", 1),
                ("normalize.semantics.v1", 1),
                ("region.formation.v1", 1),
                ("region.candidate.v1", 17),
                // One resolution and one refinement per recognized occurrence.
                ("capability.index-access-resolution.v1", 5),
                ("kernel.index-region-refinement.v1", 5),
                ("cover.enumeration.v1", 1),
                ("fusion.legality.v1", 12),
                ("fusion.strict-f32-equivalence", 1),
                ("frontier.enumeration.v1", 4),
                ("selection.complete-plan.v1", 1),
                ("compile.region.verified", 3),
                ("compile.plan.boundary", 2),
                ("schedule.plan-regions", 2),
                ("kernel.plan-refinement", 2),
                ("program.plan-verified", 2),
                ("artifact.plan-construction", 2),
                ("target.barriers", 3),
                ("target.buffer-bindings", 3),
                ("target.device-memory", 3),
                ("target.grid-axis", 3),
                ("target.index-bits", 3),
                ("target.local-memory-bytes", 3),
                // The four per-dimension honourability records replace the one
                // `target.strict-f32` predicate, which is the whole point of
                // retiring it: three regions each now report which dimension was
                // assessed and by what means, where one boolean reported neither.
                ("target.numerics.contraction", 3),
                ("target.numerics.input-subnormals", 3),
                ("target.numerics.reassociation", 3),
                ("target.numerics.result-subnormals", 3),
                ("target.threads-per-workgroup", 3),
                ("tiler.cost.structural.v1", 2),
                ("tiler.selection.structural-pareto.v1", 2),
            ])
        );
        for (rule, fact_key, expected) in [
            ("normalize.semantics.v1", "rewrite-count", 0),
            ("region.formation.v1", "candidate-count", 17),
            ("region.formation.v1", "operation-count", 5),
            ("cover.enumeration.v1", "cover-count", 16),
            ("selection.complete-plan.v1", "plan-count", 2),
        ] {
            let record = trace
                .records()
                .iter()
                .find(|record| record.rule().key().as_str() == rule)
                .unwrap_or_else(|| panic!("missing typed count emitter {rule}"));
            let ExplainEvent::Check { assessment, .. } = record.event() else {
                panic!("typed count emitter {rule} must be a checked assertion");
            };
            assert!(assessment.predicate().as_str().contains('.'));
            let actual = assessment
                .facts()
                .iter()
                .find(|fact| fact.key().as_str() == fact_key)
                .map(|fact| fact.value().clone());
            assert_eq!(
                actual,
                Some(FactValue::Count(expected)),
                "{rule}/{fact_key}"
            );
        }
        // Every recognized occurrence resolved a lowering capability and carries
        // exhaustive finite refinement evidence attributed to the same provider.
        for (rule, stage, basis) in [
            (
                "capability.index-access-resolution.v1",
                ExplainStage::CapabilityResolution,
                EvidenceBasis::CheckedInvariant,
            ),
            (
                "kernel.index-region-refinement.v1",
                ExplainStage::KernelRefinement,
                EvidenceBasis::ExhaustiveFinite,
            ),
        ] {
            let records: Vec<_> = trace
                .records()
                .iter()
                .filter(|record| record.rule().key().as_str() == rule)
                .collect();
            assert_eq!(records.len(), 5, "{rule}");
            for record in records {
                assert_eq!(record.event().disposition(), ExplainDisposition::Admitted);
                assert_eq!(record.event().stage(), stage);
                let ExplainEvent::Check { assessment, .. } = record.event() else {
                    panic!("{rule} must be a checked assertion");
                };
                assert_eq!(assessment.basis(), &basis);
                // Attribution is the resolved lowering provider, never the
                // compiler: an out-of-crate provider owns this claim.
                assert_ne!(record.rule().provider(), &ProviderRef::builtin());
            }
        }
        // Fusion legality is attributed to the capability provider that declared
        // the member operations' roles, never to the compiler itself.
        let legality = trace
            .records()
            .iter()
            .find(|record| record.rule().key().as_str() == "fusion.legality.v1")
            .expect("a fusion-legality record");
        assert_eq!(legality.event().disposition(), ExplainDisposition::Admitted);
        assert!(trace.render().starts_with("tiler-explain-v2 request="));
    }

    /// Asserts the honourability half of the end-to-end explain conformance.
    ///
    /// The numerical dimensions left the quantitative predicate space when
    /// `strict-f32` was retired, so they are counted through their own typed
    /// record. Each names the dimension, the behaviour the resolved contract
    /// required, the means the profile declares, and the declaring profile — and
    /// the admitted trace asserts the *means*, because a proven verdict alone
    /// would not distinguish native support from emulation.
    fn assert_honoured_dimensions_are_exhaustive(trace: &crate::explain::VerifiedExplainTrace) {
        let mut honoured = BTreeMap::new();
        for record in trace.records() {
            let ExplainEvent::NumericalHonourability {
                dimension,
                required,
                outcome,
                profile,
            } = record.event()
            else {
                continue;
            };
            assert_eq!(
                outcome,
                &crate::explain::HonourabilityOutcome::Honoured {
                    means: crate::explain::ReasonCode::new("supported-exactly").unwrap(),
                }
            );
            assert_eq!(
                profile.as_str(),
                "tiler.prototype-target-neutral-baseline.v1"
            );
            *honoured
                .entry((dimension.as_str(), required.as_str()))
                .or_insert(0_usize) += 1;
        }
        assert_eq!(
            honoured,
            BTreeMap::from([
                (("numerics.contraction", "forbidden"), 3),
                (("numerics.input-subnormals", "preserve"), 3),
                (("numerics.reassociation", "forbidden"), 3),
                (("numerics.result-subnormals", "preserve"), 3),
            ])
        );
        assert!(trace.render().contains(
            "honourability:numerics.input-subnormals:preserve:honoured:supported-exactly:profile=tiler.prototype-target-neutral-baseline.v1"
        ));
    }

    #[test]
    fn end_to_end_explain_emitter_has_exhaustive_typed_conformance() {
        let semantic = semantic(false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let trace = &product.targets[0].explain;

        let mut target_predicates = BTreeMap::new();
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
                "index-bits" | "device-memory" | "barriers" => {
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
            BTreeMap::from([
                ("barriers", 3),
                ("buffer-bindings", 3),
                ("device-memory", 3),
                ("grid-axis", 3),
                ("index-bits", 3),
                ("local-memory-bytes", 3),
                ("threads-per-workgroup", 3),
            ])
        );

        assert_honoured_dimensions_are_exhaustive(trace);

        let selections = trace
            .records()
            .iter()
            .filter_map(|record| match record.event() {
                ExplainEvent::Selection { outcome, .. } => {
                    Some((record.subjects()[0].key().as_str().to_owned(), *outcome))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
        let fused = alternative(&product, ProgramAlternativeKind::Fused);
        assert_eq!(
            selections.get(&materialized.stable_id),
            Some(&SelectionOutcome::Dominated)
        );
        assert_eq!(
            selections.get(&fused.stable_id),
            Some(&SelectionOutcome::Selected)
        );
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

    /// A shared constant read by two operations is graph fan-out, and a legal
    /// cover must materialize it once rather than duplicate its producer.
    #[test]
    fn shared_constant_fan_out_is_materialized_once_and_never_duplicated() {
        let shared = shared_constant_semantic(Shape::from_dims([2, 3]), 2.0_f32.to_bits());
        let product = compile(CompilationRequest::governed(&shared)).unwrap();
        for alternative in &product.targets[0].portfolio.alternatives {
            assert!(
                alternative.plan.cover().duplication().is_none(),
                "producer duplication is disabled in this profile"
            );
            // Every cross-region value is one materialization edge with one or
            // more consumers, never one edge per consumer.
            let edges = alternative.plan.cover().materializations();
            let distinct = edges
                .iter()
                .map(crate::cover::MaterializationEdge::producer_position)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(edges.len(), distinct.len());
            assert_eq!(
                alternative.plan.handoffs().len(),
                edges.len(),
                "every materialization edge is discharged by exactly one handoff"
            );
        }
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

    /// An installed authority that lowers nothing is a deferred capability, and
    /// it stops the compilation instead of quietly producing a narrower
    /// portfolio: an occurrence nobody can lower has no valid plan at all.
    #[test]
    fn a_registry_without_capabilities_defers_and_fails_closed() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.capabilities = CompilerCapabilitySnapshot::without_capabilities();
        let error = compile(request).unwrap_err();
        let CompileError::Explained { source, explain } = error else {
            panic!("target compilation failures retain their explain trace");
        };
        assert_eq!(
            *source,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "lowering",
                rule: "missing-capability",
            })
        );
        assert!(explain.records().iter().any(|record| {
            record.rule().key().as_str() == "capability.index-access-resolution.v1"
                && record.event().disposition() == ExplainDisposition::DeferredUnsupported
        }));
        let failure = explain
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .expect("a terminal failure record");
        assert!(matches!(
            failure.event(),
            ExplainEvent::CompilerFailure {
                stage: ExplainStage::CapabilityResolution,
                reason,
            } if reason.as_str() == "lowering-missing-capability"
        ));
    }

    #[test]
    fn region_budget_retains_the_verified_baseline() {
        let semantic = semantic(false);
        // A zero per-seed growth budget leaves only singleton candidates, and the
        // bounded profile implements no singleton region. Every plan therefore
        // depends on a region that was never formed, so compilation fails closed
        // with a typed no-complete-plan error rather than implementing a region
        // region formation never proposed.
        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.region_candidates_per_seed = 0;
        let error = compile(bounded).unwrap_err();
        let CompileError::Explained { source, explain } = error else {
            panic!("target compilation failures retain their explain trace");
        };
        assert!(matches!(
            *source,
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Selection(
                SelectionError::Structure {
                    rule: "no-complete-plan"
                }
            ))
        ));
        assert!(explain.records().iter().any(|record| {
            record.rule().key().as_str() == "region.formation.v1"
                && record.event().disposition() == ExplainDisposition::BudgetStopped
        }));
        assert_eq!(
            explain
                .records()
                .iter()
                .filter(|record| record.rule().key().as_str() == "region.candidate.v1")
                .count(),
            5
        );
    }

    /// A cover budget never loses the two covers the enumerator retains
    /// unconditionally — the all-singleton and the whole-program cover — and any
    /// discovered partition it does lose is reported as a typed budget stop.
    ///
    /// The bounded profile implements no singleton region, so the all-singleton
    /// cover yields no plan. Losing the discovered two-region partition therefore
    /// costs the materialized alternative, which is exactly what the typed stop
    /// makes visible instead of silently narrowing the portfolio.
    #[test]
    fn cover_budget_stops_are_reported_without_losing_either_extreme() {
        let semantic = semantic(false);
        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.region_covers = 1;
        let product = compile(bounded).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert_eq!(selected_kind(&product), ProgramAlternativeKind::Fused);
        assert!(product.targets[0].explain.records().iter().any(|record| {
            record.rule().key().as_str() == "cover.enumeration.v1"
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
                && record.subjects()[0].key().as_str() == "region:pointwise"
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
        // The cover whose pointwise region the target refused is retained in the
        // terminal ledger as an infeasible alternative rather than disappearing.
        assert!(target.explain.records().iter().any(|record| {
            matches!(
                record.event(),
                ExplainEvent::Selection {
                    outcome: SelectionOutcome::Infeasible,
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
        assert!(matches!(
            *source,
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(
                PhysicalError::Target { .. }
            ))
        ));
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
        assert!(!causal_rejections.is_empty());
        assert!(
            causal_rejections.iter().all(|record| {
                record.event().disposition() == ExplainDisposition::RejectedTarget
            })
        );
        // Every recognized region role the target refused is named exactly once.
        let mut subjects = causal_rejections
            .iter()
            .map(|record| record.subjects()[0].key().as_str())
            .collect::<Vec<_>>();
        subjects.sort_unstable();
        assert_eq!(
            subjects,
            [
                "region:pointwise",
                "region:reduction",
                "region:whole-program"
            ]
        );
    }

    #[test]
    fn target_rejections_are_deduplicated_by_region_role_and_axis() {
        let semantic = semantic(false);
        let request = verify_request(CompilationRequest::governed(&semantic)).unwrap();
        let request = request.for_target(request.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&request).unwrap();
        let pointwise = PhysicalError::Target {
            rule: "grid-axis",
            region: RegionId::new(0),
            required: 65_536,
            available: 65_535,
        };
        let fused = PhysicalError::Target {
            rule: "threads-per-workgroup",
            region: RegionId::new(1),
            required: 2,
            available: 1,
        };
        let root = test_root(&mut explain);
        let pointwise_cause =
            record_target_rejection(&mut explain, &pointwise, "pointwise", root).unwrap();
        let fused_cause =
            record_target_rejection(&mut explain, &fused, "whole-program", root).unwrap();
        let mut rejections = TargetRejections::default();
        rejections
            .push(TargetRejection {
                role: "whole-program",
                error: fused.clone(),
                cause: fused_cause,
            })
            .unwrap();
        rejections
            .push(TargetRejection {
                role: "pointwise",
                error: pointwise,
                cause: pointwise_cause,
            })
            .unwrap();
        // The same role and axis observed on another cover adds no second cause.
        rejections
            .push(TargetRejection {
                role: "whole-program",
                error: fused,
                cause: fused_cause,
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
    fn physical_error_stages_are_attributed_to_their_exact_phase() {
        assert_eq!(
            physical_error_stage(&PhysicalError::Target {
                rule: "grid-axis",
                region: RegionId::new(0),
                required: 2,
                available: 1,
            }),
            ExplainStage::TargetFeasibility
        );
        assert_eq!(
            physical_error_stage(&PhysicalError::Intrinsic {
                rule: "fixture",
                region: RegionId::new(0),
            }),
            ExplainStage::IntrinsicScheduling
        );
        assert_eq!(
            physical_error_stage(&PhysicalError::ShapeProductOverflow {
                region: RegionId::new(0),
            }),
            ExplainStage::IntrinsicScheduling
        );
        assert_eq!(
            physical_error_stage(&PhysicalError::Refinement {
                rule: "fixture",
                region: RegionId::new(0),
            }),
            ExplainStage::KernelRefinement
        );
    }

    #[test]
    fn structural_policy_requires_pareto_dominance_instead_of_guessing_latency() {
        let semantic = semantic(false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
        let fused = alternative(&product, ProgramAlternativeKind::Fused);
        // Fusion is strictly better on every structural dimension here, so it
        // dominates; the reverse comparison must not hold.
        assert!(
            fused
                .structural_cost
                .dominates(&materialized.structural_cost)
        );
        assert!(
            !materialized
                .structural_cost
                .dominates(&fused.structural_cost)
        );
        // Dominance is a partial order: a plan never dominates itself.
        assert!(!fused.structural_cost.dominates(&fused.structural_cost));
        // The selection is the first non-dominated plan in canonical order, so
        // it is exactly the plan the portfolio's own Pareto view retains.
        let retained = product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .filter(|candidate| {
                !product.targets[0]
                    .portfolio
                    .alternatives
                    .iter()
                    .any(|other| other.structural_cost.dominates(&candidate.structural_cost))
            })
            .map(|candidate| candidate.stable_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            retained,
            [product.targets[0]
                .portfolio
                .selection
                .selected_alternative_id
                .clone()]
        );
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

    /// A lone contributor's NaN payload must not survive the reduction boundary.
    ///
    /// The strict serial sum canonicalizes at its result boundary "even when the
    /// contributor sequence is a singleton" (`docs/numerical-semantics.md`, ADR
    /// 0055). A reduced axis of extent one is exactly where that rule is
    /// load-bearing rather than redundant: no combine has run, so nothing else
    /// has canonicalized the value being written.
    ///
    /// `structured_fused_body_interpreter_matches_reference_evaluator` cannot
    /// see this. Its `[4, 1]` vector carries `f32::NAN`, which already *is*
    /// `CANONICAL_F32_ARITHMETIC_NAN_BITS`, and it interprets the fused kernel,
    /// whose scale/bias prologue canonicalizes the seed regardless. This case
    /// interprets the materialized alternative's bare `StrictSerialSum` kernel
    /// and supplies the payload directly.
    #[test]
    fn a_singleton_reduction_canonicalizes_a_lone_non_canonical_nan() {
        let shape = Shape::from_dims([4, 1]);
        let semantic = semantic_case(shape.clone(), 1.0_f32.to_bits(), 0.0_f32.to_bits(), false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let materialized = alternative(&product, ProgramAlternativeKind::Materialized);
        let reduction = &materialized.kernels[1];
        assert_eq!(
            reduction.buffers().next().unwrap().tensor,
            TensorRole::Intermediate,
            "the second materialized kernel reduces the materialized intermediate"
        );

        // The intermediate is an ordinary runtime buffer whose declared element
        // domain is every binary32 pattern, not only the ones this program's own
        // prologue happens to produce.
        let intermediate = vec![
            f32::from_bits(0x7fc0_1234),
            f32::from_bits(0xffc0_0000),
            -0.0_f32,
            f32::from_bits(1),
        ];
        let actual: Vec<u32> = interpret_fused(reduction, &intermediate)
            .iter()
            .map(|value| value.to_bits())
            .collect();

        let key = InputKey::new("input").unwrap();
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let bare = builder.input::<F32>(key.clone(), shape.clone()).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, bare, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        let bare_sum = builder.build().unwrap();
        let tensor = Tensor::dense(
            F32::resolved_type(),
            shape,
            intermediate
                .iter()
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
        let evaluated = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&bare_sum, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        let expected: Vec<u32> = match evaluated[0].payload() {
            TensorPayloadView::Dense(elements) => elements
                .iter()
                .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
                .collect(),
            _ => panic!("expected dense f32 reference output"),
        };
        assert_eq!(
            expected,
            [
                CANONICAL_F32_ARITHMETIC_NAN_BITS,
                CANONICAL_F32_ARITHMETIC_NAN_BITS,
                (-0.0_f32).to_bits(),
                1,
            ],
            "the boundary rule rewrites both NaN payloads and preserves every other one"
        );
        assert_eq!(
            actual, expected,
            "the compiled kernel must realize that rule"
        );
    }

    /// The structured addressing must realize a non-trailing reduced axis.
    ///
    /// A leading reduced axis makes the contributor stride differ from one, and
    /// a middle reduced axis additionally forces the kept coordinate to be
    /// recovered with an explicit index division and remainder. Both are lowered
    /// as ordinary index arithmetic, so interpreting the emitted operations must
    /// still reproduce the reference evaluator exactly.
    #[test]
    fn structured_addressing_realizes_non_trailing_reduction_axes() {
        assert_fused_axis_matches_reference(
            Shape::from_dims([3, 2]),
            vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, 0.0],
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            Axis::new(0),
        );
        assert_fused_axis_matches_reference(
            Shape::from_dims([2, 3, 2]),
            (0..12_u8).map(|value| f32::from(value) - 4.0).collect(),
            0.5_f32.to_bits(),
            (-0.25_f32).to_bits(),
            Axis::new(1),
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
        let selected = target.portfolio.selection.selected_alternative_id.clone();
        let portfolio = plan_portfolio(&semantic, &request);

        assert!(
            verify_portfolio(
                &semantic,
                &request,
                &portfolio,
                alternatives,
                &selected,
                None
            )
            .is_ok()
        );
        assert!(verify_portfolio(&semantic, &request, &portfolio, &[], &selected, None).is_err());
        let selection = verify_portfolio(
            &semantic,
            &request,
            &portfolio,
            alternatives,
            "stale-selection",
            None,
        )
        .unwrap_err();
        assert_eq!(selection.context.stage, ExplainStage::Selection);
        assert_eq!(
            selection.context.reason.as_str(),
            "structure-portfolio-selection"
        );

        let mut forged = alternatives.clone();
        forged[0].stable_id = "forged-plan".to_owned();
        let identity = verify_portfolio(&semantic, &request, &portfolio, &forged, &selected, None)
            .unwrap_err();
        assert_eq!(identity.context.stage, ExplainStage::Costing);

        let mut forged_artifact = alternatives.clone();
        forged_artifact[0].artifact_plan = forged_artifact[1].artifact_plan.clone();
        let artifact = verify_portfolio(
            &semantic,
            &request,
            &portfolio,
            &forged_artifact,
            &selected,
            None,
        )
        .unwrap_err();
        assert_eq!(artifact.context.stage, ExplainStage::ArtifactPlanning);

        let mut forged_numerics = alternatives.clone();
        forged_numerics[0].equivalence = forged_numerics[1].equivalence.clone();
        let numerical = verify_portfolio(
            &semantic,
            &request,
            &portfolio,
            &forged_numerics,
            &selected,
            None,
        )
        .unwrap_err();
        assert_eq!(numerical.context.stage, ExplainStage::NumericalLegality);
        assert_eq!(
            numerical.context.reason.as_str(),
            "structure-portfolio-equivalence"
        );
    }

    /// Re-derives the selected portfolio for a verified target request.
    fn plan_portfolio(
        semantic: &SemanticProgram,
        request: &crate::request::VerifiedTargetRequest,
    ) -> crate::selection::SelectedPortfolio {
        let mut explain = ExplainWriter::new(request).unwrap();
        let formation =
            form_region_candidates(semantic, request.budgets(), request.numerical_contract())
                .unwrap();
        let root = test_root(&mut explain);
        enumerate_complete_plans(semantic, request, &formation, &mut explain, root, None)
            .map_or_else(
                |_| panic!("the governed request enumerates complete plans"),
                |plans| plans.portfolio,
            )
    }

    #[test]
    fn intrinsic_physical_failures_are_invalid_output_not_empty_frontiers() {
        let error = CompileError::from(PhysicalError::Intrinsic {
            rule: "forged",
            region: RegionId::new(0),
        });
        assert!(matches!(
            error,
            CompileError::InvalidCompilerOutput(CompilerOutputError::Physical(
                PhysicalError::Intrinsic { .. }
            ))
        ));
    }
}

/// The target-neutral optimizer conformance gate.
///
/// Everything here drives the ordinary `compile()` entry point. Nothing reaches
/// past it into a stage-local constructor, and no fixture is admitted by a
/// `cfg(test)` shortcut: the operation definitions come from a registry provider
/// written entirely against `tiler-ir`'s public surface, exactly as an
/// out-of-crate consumer would supply them.
#[cfg(test)]
mod conformance {
    use std::sync::Arc;

    use super::tests::{interpret_fused, reduction_loop};
    use super::{
        CompilationProduct, CompileError, ProgramAlternative, ProgramAlternativeKind, compile,
    };
    use crate::capability::{
        IndexAccessLoweringContext, IndexAccessLoweringProvider, LoweringCapabilityRegistryBuilder,
        LoweringCapabilityRevision, LoweringEmitError, LoweringSignature,
    };
    use crate::cover::RegionCover;
    use crate::explain::{
        EvidenceBasis, ExplainDisposition, ExplainEvent, ExplainStage, ProviderRef,
    };
    use crate::region::form_region_candidates;
    use crate::request::{
        CompilationRequest, CompilerCapabilitySnapshot, RequestError, verify_request,
    };
    use tiler_ir::index::{DomainRole, FrozenScalarRegistry, ScalarAttributes};
    use tiler_ir::semantic::{
        CanonicalIntegerWidth, CanonicalValue, CanonicalValueKind, CanonicalValueView, F32,
        F32_CONSTANT_BITS_ATTRIBUTE, InputKey, NormativeDefinitionRef, OpKey, OperationArity,
        OperationAttributeSchema, OperationConformance, OperationDefinition,
        OperationDefinitionFacts, OperationEffect, OperationInferenceError, OperationInferencer,
        OperationSchema, OutputKey, ProviderDiagnosticCode, ProviderIdentity,
        REDUCTION_AXES_ATTRIBUTE, RegistryError, SemanticProgram, SemanticProgramBuilder,
        SemanticRegistryBuilder, SemanticRegistryProvider, SemanticRegistryRegistrar,
        TypeDefinitionFacts, TypeKey, ValueFact, ValueTypeDefinition, ValueTypeDefinitionKey,
        add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// The shape-inference behaviour one externally registered operation declares.
    #[derive(Clone, Copy)]
    enum ExternalOperation {
        Constant,
        Binary,
        Sum,
    }

    impl OperationInferencer for ExternalOperation {
        fn infer(
            &self,
            request: tiler_ir::semantic::OperationInferenceRequest<'_>,
            outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            match self {
                Self::Constant => {
                    outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
                }
                Self::Binary => {
                    let left = operands[0].shape();
                    let right = operands[1].shape();
                    let shape = if left.rank() == 0 {
                        right.clone()
                    } else if right.rank() == 0 || left == right {
                        left.clone()
                    } else {
                        return Err(OperationInferenceError::new(
                            ProviderDiagnosticCode::new("external.binary.shape").unwrap(),
                            "operands must have equal shapes or include one scalar",
                        )
                        .unwrap());
                    };
                    outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
                }
                Self::Sum => {
                    let Some(CanonicalValueView::Sequence(values)) = request
                        .attributes()
                        .get(REDUCTION_AXES_ATTRIBUTE)
                        .map(CanonicalValue::view)
                    else {
                        return Err(OperationInferenceError::new(
                            ProviderDiagnosticCode::new("external.sum.axes").unwrap(),
                            "sum axes must be a sequence",
                        )
                        .unwrap());
                    };
                    let axes = values
                        .iter()
                        .map(|value| match value.view() {
                            CanonicalValueView::Unsigned {
                                width: CanonicalIntegerWidth::Bits32,
                                bits,
                            } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                                OperationInferenceError::new(
                                    ProviderDiagnosticCode::new("external.sum.axis-width").unwrap(),
                                    "sum axis exceeds u32",
                                )
                                .unwrap()
                            }),
                            _ => Err(OperationInferenceError::new(
                                ProviderDiagnosticCode::new("external.sum.axis-kind").unwrap(),
                                "sum axes must be u32 values",
                            )
                            .unwrap()),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    outputs.try_push(ValueFact::new(
                        F32::resolved_type(),
                        operands[0].shape().without_axes(&axes),
                    ))
                }
            }
        }
    }

    /// An out-of-crate semantic provider that defines the whole operation set.
    ///
    /// Its revision is the output-affecting provider revision ADR 0072 keeps
    /// separate from graph meaning, so the same graph admitted at two revisions
    /// is the exact identity-conformance subject this gate asserts.
    struct ExternalSemantics {
        revision: u32,
    }

    impl SemanticRegistryProvider for ExternalSemantics {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("acme", "external-f32-semantics", self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_marked_value_type::<F32>(
                ValueTypeDefinition::structurally_valid(
                    ValueTypeDefinitionKey::Nominal(TypeKey::new("tiler", "f32", 1).unwrap()),
                    NormativeDefinitionRef::new("external binary32 semantics")?,
                    TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
                ),
                F32::resolved_type(),
            )?;
            register(
                registrar,
                constant_f32_op(),
                0,
                &[OperationAttributeSchema::required(
                    F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )],
                ExternalOperation::Constant,
            )?;
            register(
                registrar,
                multiply_f32_op(),
                2,
                &[],
                ExternalOperation::Binary,
            )?;
            register(registrar, add_f32_op(), 2, &[], ExternalOperation::Binary)?;
            register(
                registrar,
                strict_serial_sum_f32_op(),
                1,
                &[OperationAttributeSchema::required(
                    REDUCTION_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
                ExternalOperation::Sum,
            )
        }
    }

    fn register(
        registrar: &mut SemanticRegistryRegistrar<'_>,
        key: OpKey,
        operands: u32,
        attributes: &[OperationAttributeSchema],
        inferencer: ExternalOperation,
    ) -> Result<(), RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            key,
            OperationSchema::new(
                OperationArity::exact(operands),
                OperationArity::exact(1),
                attributes.to_vec(),
            )
            .expect("the external operation schema is valid"),
            NormativeDefinitionRef::new("external governed operation semantics")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(inferencer),
        ))
    }

    /// Builds a scale-bias-then-serial-sum program from the external registry.
    ///
    /// Every operation the graph contains is defined by [`ExternalSemantics`];
    /// nothing in it comes from `SemanticProgramBuilder::try_standard`.
    fn external_program(
        revision: u32,
        shape: Shape,
        axes: &[Axis],
        share_constant: bool,
    ) -> SemanticProgram {
        external_program_with_bias(revision, shape, axes, share_constant, 1.0_f32.to_bits())
    }

    /// Builds the same program with an explicit bias constant bit pattern.
    ///
    /// A bias equal to the scale gives two *distinct* constant occurrences with
    /// identical content, which is the region content/occurrence separation
    /// subject; the default fixture keeps them distinguishable instead.
    fn external_program_with_bias(
        revision: u32,
        shape: Shape,
        axes: &[Axis],
        share_constant: bool,
        bias_bits: u32,
    ) -> SemanticProgram {
        let mut registry = SemanticRegistryBuilder::new();
        registry
            .register_provider(&ExternalSemantics { revision })
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let scale =
            tiler_ir::semantic::F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = if share_constant {
            scale
        } else {
            tiler_ir::semantic::F32Constant::apply(&mut builder, bias_bits).unwrap()
        };
        let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = tiler_ir::semantic::F32Add::apply(&mut builder, product, bias).unwrap();
        let sum =
            tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, mapped, axes.to_vec())
                .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn alternative(
        product: &CompilationProduct,
        kind: ProgramAlternativeKind,
    ) -> &ProgramAlternative {
        product.targets[0]
            .portfolio
            .alternatives
            .iter()
            .find(|alternative| alternative.kind == kind)
            .expect("the requested plan shape is retained")
    }

    /// Asserts a cover assigns every operation to exactly one region.
    fn assert_complete_partition(cover: &RegionCover, operation_count: u32) {
        let mut members: Vec<u32> = cover
            .regions()
            .iter()
            .flat_map(|region| region.members().iter().map(|member| member.0))
            .collect();
        members.sort_unstable();
        let distinct = members
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            members.len(),
            distinct.len(),
            "no operation is double covered"
        );
        assert_eq!(
            u32::try_from(members.len()).unwrap(),
            operation_count,
            "no operation is left uncovered"
        );
    }

    /// The gate's core claim: an externally defined operation set compiles end to
    /// end through the ordinary path, and every implemented layer is present.
    #[test]
    fn externally_registered_operations_compile_through_the_ordinary_path() {
        let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
        let product = compile(CompilationRequest::governed(&program)).unwrap();
        let target = &product.targets[0];
        assert_eq!(
            target.target_profile_key,
            "tiler.prototype-target-neutral-baseline.v1"
        );
        assert_eq!(target.portfolio.alternatives.len(), 2);

        for alternative in &target.portfolio.alternatives {
            // Complete legal cover, one implementation per region, verified KIR,
            // a neutral kernel program, and an artifact construction plan.
            assert_complete_partition(
                alternative.plan.cover(),
                u32::try_from(program.operation_count()).unwrap(),
            );
            assert_eq!(
                alternative.plan.selections().len(),
                alternative.plan.cover().region_count()
            );
            assert_eq!(
                alternative.kernels.len(),
                alternative.scheduled_regions.len()
            );
            assert_eq!(
                alternative.program.stage_count(),
                alternative.scheduled_regions.len()
            );
            assert!(!alternative.artifact_plan.lowering_providers().is_empty());
            // Every retained plan rests on hard-feasibility evidence, never cost.
            assert!(!alternative.plan.guards().is_empty());
            // Every fused region carries a replayable fusion-legality proof.
            let fused_regions = alternative
                .plan
                .cover()
                .regions()
                .iter()
                .filter(|region| region.members().len() > 1)
                .count();
            assert_eq!(alternative.equivalence.legality().len(), fused_regions);
        }
        assert!(
            alternative(&product, ProgramAlternativeKind::Fused)
                .equivalence
                .numerical()
                .is_some(),
            "a whole-program fused plan carries its strict-f32 equivalence proof"
        );
        assert_eq!(
            reduction_loop(&alternative(&product, ProgramAlternativeKind::Fused).kernels[0]),
            Some((1, 3))
        );
        // The verified KIR alone drives the backend-shaped interpreter.
        let values = vec![1.0, -2.0, 3.5, 0.5, -0.0, 0.0];
        let fused = interpret_fused(
            &alternative(&product, ProgramAlternativeKind::Fused).kernels[0],
            &values,
        );
        assert_eq!(fused.len(), 2);
    }

    /// Two non-isomorphic graph shapes — a rank-2 trailing reduction and a rank-3
    /// interior reduction — both compile, and neither borrows the other's plan.
    #[test]
    fn non_isomorphic_graph_shapes_produce_distinct_verified_plans() {
        let rank_two = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
        let rank_three = external_program(1, Shape::from_dims([2, 3, 2]), &[Axis::new(1)], false);
        assert_ne!(
            rank_two.semantic_identity().graph(),
            rank_three.semantic_identity().graph(),
            "the two fixtures must be non-isomorphic graphs"
        );

        let first = compile(CompilationRequest::governed(&rank_two)).unwrap();
        let second = compile(CompilationRequest::governed(&rank_three)).unwrap();
        for product in [&first, &second] {
            assert_eq!(product.targets[0].portfolio.alternatives.len(), 2);
        }
        // Distinct semantics yield distinct plan identities at every layer.
        let left = alternative(&first, ProgramAlternativeKind::Fused);
        let right = alternative(&second, ProgramAlternativeKind::Fused);
        assert_ne!(left.plan.identity(), right.plan.identity());
        assert_ne!(left.stable_id, right.stable_id);
        assert_ne!(
            left.scheduled_regions[0].canonical_identity().as_bytes(),
            right.scheduled_regions[0].canonical_identity().as_bytes()
        );
        assert_ne!(left.kernels[0], right.kernels[0]);
        assert_ne!(left.artifact_plan, right.artifact_plan);
    }

    /// Graph fan-out: one constant read by two operations is materialized once.
    #[test]
    fn shared_producer_fan_out_compiles_without_duplicating_the_producer() {
        let shared = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], true);
        assert_eq!(shared.operation_count(), 4);
        let product = compile(CompilationRequest::governed(&shared)).unwrap();
        for alternative in &product.targets[0].portfolio.alternatives {
            assert_complete_partition(
                alternative.plan.cover(),
                u32::try_from(shared.operation_count()).unwrap(),
            );
            assert!(alternative.plan.cover().duplication().is_none());
        }
    }

    /// An ordered multi-output program is not silently approximated; the bounded
    /// profile rejects it explicitly at the request boundary.
    #[test]
    fn ordered_multi_output_programs_reject_explicitly() {
        let mut registry = SemanticRegistryBuilder::new();
        registry
            .register_provider(&ExternalSemantics { revision: 1 })
            .unwrap();
        let mut builder = SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale =
            tiler_ir::semantic::F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let product = tiler_ir::semantic::F32Multiply::apply(&mut builder, input, scale).unwrap();
        let sum =
            tiler_ir::semantic::StrictSerialF32Sum::apply(&mut builder, product, [Axis::new(1)])
                .unwrap();
        builder
            .output(OutputKey::new("reduced").unwrap(), sum)
            .unwrap();
        builder
            .output(OutputKey::new("scaled").unwrap(), product)
            .unwrap();
        let multi_output = builder.build().unwrap();
        assert_eq!(multi_output.output_count(), 2);

        let error = compile(CompilationRequest::governed(&multi_output)).unwrap_err();
        assert_eq!(
            error,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "signature",
            })
        );
    }

    /// ADR 0072 identity conformance for a provider-only revision change.
    ///
    /// The same graph admitted by two revisions of the same external provider
    /// keeps its graph meaning and its reached definition projection, changes its
    /// admission provenance and registry snapshot, and — because neither is
    /// structural content — reproduces every structural layer byte for byte.
    #[test]
    fn provider_only_revision_changes_provenance_and_not_structure() {
        let first = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
        let second = external_program(2, Shape::from_dims([2, 3]), &[Axis::new(1)], false);

        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        assert_eq!(
            first.semantic_identity().reached_definitions(),
            second.semantic_identity().reached_definitions()
        );
        assert_ne!(
            first.semantic_identity().admission_provenance(),
            second.semantic_identity().admission_provenance()
        );
        assert_ne!(
            first.semantic_identity().registry_snapshot(),
            second.semantic_identity().registry_snapshot()
        );

        let first = compile(CompilationRequest::governed(&first)).unwrap();
        let second = compile(CompilationRequest::governed(&second)).unwrap();
        for kind in [
            ProgramAlternativeKind::Materialized,
            ProgramAlternativeKind::Fused,
        ] {
            let left = alternative(&first, kind);
            let right = alternative(&second, kind);
            // Pure structural content is identical: index/schedule identity, KIR,
            // the complete-plan receipt, and the plan's aggregate cost.
            assert_eq!(
                left.scheduled_regions[0].canonical_identity().as_bytes(),
                right.scheduled_regions[0].canonical_identity().as_bytes()
            );
            assert_eq!(left.kernels, right.kernels);
            assert_eq!(left.plan.identity(), right.plan.identity());
            assert_eq!(left.stable_id, right.stable_id);
            assert_eq!(left.structural_cost, right.structural_cost);
            // Selected-provider provenance is retained and unchanged: a semantic
            // provider revision is not a lowering-provider revision.
            assert_eq!(
                left.artifact_plan.lowering_providers(),
                right.artifact_plan.lowering_providers()
            );
            // The artifact construction plan retains the four-subject semantic
            // identity bundle atomically, so a changed admission subject is
            // visible there rather than being silently discarded.
            assert_ne!(left.artifact_plan, right.artifact_plan);
        }
        // The explain trace is bound to the exact compilation subject, so the two
        // request digests differ while the record sequence does not.
        assert_ne!(
            first.targets[0].explain.render(),
            second.targets[0].explain.render()
        );
    }

    /// Equal region *content* is reused across distinct graph *occurrences*.
    ///
    /// The two pointwise constants of the unshared program are structurally
    /// identical singleton regions, so region formation must give them one
    /// content identity and two distinct occurrence identities (ADR 0072).
    #[test]
    fn identical_region_content_keeps_distinct_occurrence_identities() {
        let program = external_program_with_bias(
            1,
            Shape::from_dims([2, 3]),
            &[Axis::new(1)],
            false,
            2.0_f32.to_bits(),
        );
        let request = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = request.for_target(request.target_profiles()[0]).unwrap();
        let formation =
            form_region_candidates(&program, target.budgets(), target.numerical_contract())
                .unwrap();
        let constants: Vec<_> = formation
            .candidates()
            .iter()
            .filter(|candidate| {
                candidate.members().len() == 1 && candidate.boundary_inputs().is_empty()
            })
            .collect();
        assert_eq!(
            constants.len(),
            2,
            "the fixture has exactly two constant occurrences"
        );
        assert_eq!(
            constants[0].content(),
            constants[1].content(),
            "structurally identical regions share one content identity"
        );
        assert_ne!(
            constants[0].occurrence(),
            constants[1].occurrence(),
            "distinct graph occurrences keep distinct occurrence identities"
        );
    }

    /// Every enumerated cover a plan rests on is a complete legal partition, and
    /// every retained plan implements each of its regions exactly once.
    #[test]
    fn complete_plan_coverage_is_exact_at_every_retained_plan() {
        for (shape, axes) in [
            (Shape::from_dims([2, 3]), vec![Axis::new(1)]),
            (Shape::from_dims([3, 2]), vec![Axis::new(0)]),
            (Shape::from_dims([2, 3, 2]), vec![Axis::new(1)]),
        ] {
            let program = external_program(1, shape, &axes, false);
            let product = compile(CompilationRequest::governed(&program)).unwrap();
            for alternative in &product.targets[0].portfolio.alternatives {
                assert_complete_partition(
                    alternative.plan.cover(),
                    u32::try_from(program.operation_count()).unwrap(),
                );
                let mut occurrences: Vec<_> = alternative
                    .plan
                    .selections()
                    .iter()
                    .map(|selection| selection.occurrence().clone())
                    .collect();
                occurrences.sort();
                let distinct = occurrences.len();
                occurrences.dedup();
                assert_eq!(
                    occurrences.len(),
                    distinct,
                    "no region occurrence is implemented twice"
                );
                // Every materialization edge the cover proposes is discharged by
                // exactly one satisfied cross-region handoff.
                assert_eq!(
                    alternative.plan.handoffs().len(),
                    alternative.plan.cover().materializations().len()
                );
            }
        }
    }

    // Externally registered *lowering* capabilities.
    //
    // Everything below composes a lowering-capability registry through the
    // public `capability` surface, exactly as an out-of-crate consumer would,
    // and drives it through the ordinary `compile()` entry point.

    /// An out-of-crate index-access lowering for `tiler.multiply-f32`.
    ///
    /// It reads every extent and every broadcast from the occurrence facts, so
    /// one registration covers every program shape. Nothing in it touches a
    /// crate-internal item.
    struct ExternalMultiplyLowering;

    impl IndexAccessLoweringProvider for ExternalMultiplyLowering {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = context.occurrence().results()[0].shape().clone();
            let value_type = context.occurrence().results()[0].value_type().clone();
            let inputs = context.occurrence().inputs().to_vec();
            let operands = context.occurrence().operands().to_vec();
            let mut dimensions = Vec::new();
            for extent in shape.extents() {
                dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
            }
            let mut coordinates = Vec::new();
            for dimension in &dimensions {
                coordinates.push(context.dimension_expr(*dimension)?);
            }
            let mut tensors = Vec::new();
            for input in &inputs {
                tensors
                    .push(context.input_tensor(input.value_type().clone(), input.shape().clone())?);
            }
            let mut values = Vec::new();
            for position in &operands {
                let value = if inputs[*position].shape().rank() == 0 {
                    context.read(tensors[*position], &[], &[])?
                } else {
                    context.read(tensors[*position], &dimensions, &coordinates)?
                };
                values.push(value);
            }
            let product = context.apply(
                tiler_ir::index::multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &values,
            )?;
            let product = product.get(0).expect("multiply yields one result");
            let output = context.output_tensor(value_type, shape)?;
            let write = context.write(output, &dimensions, &coordinates)?;
            context.output(write, product)
        }
    }

    fn external_lowering_provider() -> ProviderIdentity {
        ProviderIdentity::new("acme", "external-multiply-lowering", 3).unwrap()
    }

    /// Composes a registry from the governed families plus an external one.
    ///
    /// `substitute` replaces the governed `tiler.multiply-f32` capability;
    /// otherwise the external capability is registered *beside* it, which is the
    /// contended-capability case.
    fn registry_with_external_multiply(
        substitute: bool,
        implementation: Arc<dyn IndexAccessLoweringProvider>,
    ) -> CompilerCapabilitySnapshot {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let mut builder = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        );
        for capability in crate::governed::governed_index_access_capabilities().unwrap() {
            if substitute && capability.operation() == &multiply_f32_op() {
                continue;
            }
            capability.register(&mut builder).unwrap();
        }
        builder
            .register_index_access(
                external_lowering_provider(),
                multiply_f32_op(),
                LoweringSignature::new(
                    [F32::resolved_type(), F32::resolved_type()],
                    [F32::resolved_type()],
                )
                .unwrap(),
                &[tiler_ir::index::multiply_f32_scalar_op()],
                LoweringCapabilityRevision::new(7).unwrap(),
                implementation,
            )
            .unwrap();
        CompilerCapabilitySnapshot::new(builder.freeze(), scalars)
    }

    /// The lowering half of the gate: an out-of-crate provider lowers a
    /// recognized occurrence end to end, and the artifact plan names it.
    #[test]
    fn an_externally_registered_lowering_provider_drives_the_compile_path() {
        let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
        let mut request = CompilationRequest::governed(&program);
        request.capabilities =
            registry_with_external_multiply(true, Arc::new(ExternalMultiplyLowering));
        let product = compile(request).unwrap();
        let target = &product.targets[0];
        assert_eq!(target.portfolio.alternatives.len(), 2);

        let external = crate::request::LoweringProviderIdentity::new(
            external_lowering_provider(),
            "tiler.capability.index-access.tiler.multiply-f32.v1".to_owned(),
            LoweringCapabilityRevision::new(7).unwrap(),
        );
        for alternative in &target.portfolio.alternatives {
            assert!(
                alternative
                    .artifact_plan
                    .lowering_providers()
                    .contains(&external),
                "the artifact plan records the external provider that lowered multiply"
            );
        }
        // The external provider's own capability revision is what the resolution
        // record is attributed at, not the governed one.
        assert!(target.explain.records().iter().any(|record| {
            record.rule().key().as_str() == "capability.index-access-resolution.v1"
                && record.rule().provider()
                    == &ProviderRef::registered(&external_lowering_provider()).unwrap()
        }));
    }

    /// Two providers claiming one occurrence is a contradiction, not a choice.
    #[test]
    fn contended_lowering_capabilities_fail_closed_with_a_distinct_error() {
        let program = external_program(1, Shape::from_dims([2, 3]), &[Axis::new(1)], false);
        let mut request = CompilationRequest::governed(&program);
        request.capabilities =
            registry_with_external_multiply(false, Arc::new(ExternalMultiplyLowering));
        let error = compile(request).unwrap_err();
        let CompileError::Explained { source, explain } = error else {
            panic!("target compilation failures retain their explain trace");
        };
        assert_eq!(
            *source,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "lowering",
                rule: "ambiguous-capability",
            })
        );
        // A contradiction is a disproved check, never a deferred capability: the
        // authority was extended, and its extensions disagree.
        assert!(explain.records().iter().any(|record| {
            record.rule().key().as_str() == "capability.index-access-resolution.v1"
                && record.event().disposition() == ExplainDisposition::RejectedIntrinsic
        }));
        assert!(!explain.records().iter().any(|record| {
            record.rule().key().as_str() == "capability.index-access-resolution.v1"
                && record.event().disposition() == ExplainDisposition::DeferredUnsupported
        }));
    }

    /// An out-of-crate lowering whose write the interval proof cannot settle.
    ///
    /// The write coordinate is a chain of `wraps` moduli, so it is neither a
    /// coordinate permutation nor interval-provable in one step. Verification
    /// therefore has to enumerate the access domain, at `points × plan_len`
    /// evaluated cells — which is exactly the budget
    /// `tiler_ir::index::MAX_EXHAUSTIVE_PROOF_CELLS` governs.
    struct WrappedWriteMultiplyLowering {
        wraps: usize,
    }

    impl IndexAccessLoweringProvider for WrappedWriteMultiplyLowering {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let shape = context.occurrence().results()[0].shape().clone();
            let value_type = context.occurrence().results()[0].value_type().clone();
            let inputs = context.occurrence().inputs().to_vec();
            let operands = context.occurrence().operands().to_vec();
            let mut dimensions = Vec::new();
            for extent in shape.extents() {
                dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
            }
            let mut coordinates = Vec::new();
            for dimension in &dimensions {
                coordinates.push(context.dimension_expr(*dimension)?);
            }
            let mut tensors = Vec::new();
            for input in &inputs {
                tensors
                    .push(context.input_tensor(input.value_type().clone(), input.shape().clone())?);
            }
            let mut values = Vec::new();
            for position in &operands {
                let value = if inputs[*position].shape().rank() == 0 {
                    context.read(tensors[*position], &[], &[])?
                } else {
                    context.read(tensors[*position], &dimensions, &coordinates)?
                };
                values.push(value);
            }
            let product = context.apply(
                tiler_ir::index::multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &values,
            )?;
            let product = product.get(0).expect("multiply yields one result");
            let output = context.output_tensor(value_type, shape.clone())?;
            let mut written = coordinates.clone();
            let leading = shape.extents()[0].get();
            for _ in 0..self.wraps {
                written[0] = context.modulo(written[0], leading)?;
            }
            let write = context.write(output, &dimensions, &written)?;
            context.output(write, product)
        }
    }

    /// The governed lowerings are interval-provable at any recognized size.
    ///
    /// Their writes are coordinate permutations and their reads are bounded by
    /// their own dimensions, so verification never enters the exhaustive path and
    /// the proof budget is never charged. This is the measured fact that lets
    /// refinement be attempted for every occurrence rather than gated on size.
    #[test]
    fn governed_lowerings_never_charge_the_exhaustive_proof_budget() {
        let program = external_program(1, Shape::from_dims([70_000, 2]), &[Axis::new(0)], false);
        let product = compile(CompilationRequest::governed(&program)).unwrap();
        assert!(!product.targets[0].explain.records().iter().any(|record| {
            record.rule().key().as_str() == "kernel.index-region-refinement.v1"
                && record.event().disposition() != ExplainDisposition::Admitted
        }));
    }

    /// A refinement the proof budget cannot afford is a recorded `Unknown` gap.
    ///
    /// The compilation is otherwise valid and must stand: nothing about the
    /// emitted region was disproved, the exhaustive access proof simply stopped.
    /// Rejecting the plan here would report an exhausted analysis budget as hard
    /// infeasibility, so the trace instead carries the typed budget stop naming
    /// the resource, its limit, and the required amount, beside an explicit
    /// unknown assessment of the refinement predicate.
    #[test]
    fn a_refinement_the_proof_budget_cannot_afford_is_recorded_not_rejected() {
        // 65_535 domain points and an eighteen-expression evaluation plan need
        // 1_179_630 cells, above the 1_048_576 the exhaustive proof admits.
        let program = external_program(1, Shape::from_dims([65_535, 1]), &[Axis::new(1)], false);
        let mut request = CompilationRequest::governed(&program);
        request.capabilities = registry_with_external_multiply(
            true,
            Arc::new(WrappedWriteMultiplyLowering { wraps: 16 }),
        );
        let product = compile(request).unwrap();
        let trace = &product.targets[0].explain;

        let stop = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::BudgetStop { .. }))
            .expect("the stopped refinement is recorded as a typed budget stop");
        assert!(matches!(
            stop.event(),
            ExplainEvent::BudgetStop {
                stage: ExplainStage::KernelRefinement,
                resource,
                limit: 1_048_576,
                actual: 1_179_630,
            } if resource.as_str() == "index-proof-cells"
        ));

        // The unproven predicate is recorded as unknown, never as admitted.
        let unknown = trace
            .records()
            .iter()
            .find(|record| {
                record.rule().key().as_str() == "kernel.index-region-refinement.v1"
                    && matches!(
                        record.event(),
                        ExplainEvent::Check {
                            assessment,
                            ..
                        } if assessment.basis() == &EvidenceBasis::Unknown
                    )
            })
            .expect("the absent refinement is recorded as an unknown gap");
        assert_eq!(
            unknown.event().disposition(),
            ExplainDisposition::DeferredUnsupported
        );

        // The remaining occurrences still carry exhaustive finite evidence, and
        // the plan the budget stop applies to is still retained.
        assert_eq!(
            trace
                .records()
                .iter()
                .filter(|record| {
                    record.rule().key().as_str() == "kernel.index-region-refinement.v1"
                        && record.event().disposition() == ExplainDisposition::Admitted
                })
                .count(),
            4
        );
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 2);
    }
}
