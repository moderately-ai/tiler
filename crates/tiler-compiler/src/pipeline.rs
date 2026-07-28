//! Top-level compilation orchestration.
//!
//! This root is the compilation story, in order: verify the request, plan
//! transactionally, build one alternative per retained plan, select among them,
//! re-derive the result, and form the product. Everything a stage *uses* lives
//! in a sibling module of the compiler; everything a stage *is* lives here.
//!
//! # The phases, and the invariant each file owns
//!
//! The split is by invariant rather than by size, which is why the files differ
//! by a factor of five.
//!
//! - `planning` — transactional planning and alternative construction. The
//!   boundary is the transaction: nothing it produces is observable until this
//!   root accepts the portfolio it returns.
//! - `trace` — explain-record production. Nothing there decides anything. A
//!   function in `trace` observes a decision this root already made, so a change
//!   to it can alter what a reader is told and never what the compiler chose.
//! - `verify` — re-derivation of the retained portfolio from the program and its
//!   own contents. **It may not reuse a planning intermediate**, because a
//!   verifier handed the value it is checking compares that value to itself and
//!   can never say no. That independence is the most expensive thing in a
//!   compile and it is deliberate, not duplicated work.
//! - `conformance` — the target-neutral optimizer conformance gate, a sibling of
//!   `tests` rather than part of it: it drives the public `compile()` entry
//!   point only and reaches no stage-local constructor, and merging the two
//!   would blur exactly that line.
//!
//! Each of `planning`, `trace`, and `verify` is a private child that glob-imports
//! this root. They are halves of one module, not separate concepts, which is why
//! `pipeline` remains one compiler concept and one internal path.

use std::error::Error;
use std::fmt;

use tiler_ir::identity::push_slice;

use crate::cover::{
    CoverEnumeration, CoverError, RegionCover, RegionCoverIdentity, enumerate_covers,
};
use crate::explain::{
    CostDisposition, CostModelKey, CostTerm, EvidenceBasis, ExplainError, ExplainEvent,
    ExplainFact, ExplainRecordId, ExplainStage, ExplainWriter, FactValue, FailureDescriptor,
    MAX_TERMINAL_CAUSES, PredicateAssessment, PredicateKey, ProviderRef, Quantity, ReasonCode,
    RejectionClass, RuleRef, SelectionOutcome, SubjectKind, TerminalCause, VerifiedEvidenceRef,
    VerifiedExplainTrace,
};
use crate::feasibility::FeasibilityRuleSetIdentity;
use crate::frontier::{
    FrontierError, FrontierRegionSubject, GovernedPhysicalProvider, ImplementationFrontier,
    PhysicalImplementationProvider, enumerate_frontier,
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
    AlgebraicExplorationParts, AlgebraicRuleConfiguration, NORMALIZATION_SUBJECT,
    NormalizationOutcome, NormalizeError, explore_algebraic_alternatives_owned,
    group_by_resolved_contract, normalize_semantics, readmit_alternatives,
    record_rewrite_assessment, rewrite_assessment_key,
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
use crate::rewrite::{
    RewriteAssessment, RewriteProposal, RewriteRuleIdentity, record_adopted_alternatives,
};
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
    /// The caller's stated contract preference, in the order it stated them.
    pub(crate) stated_contracts: Vec<crate::request::StrictF32NumericalContract>,
    /// The one contract this target resolved to.
    pub(crate) resolved_contract: crate::request::StrictF32NumericalContract,
    pub(crate) target_profile_key: &'static str,
    /// Canonical descriptor bytes of the profile every alternative was assessed
    /// against, and the rules they were assessed under.
    ///
    /// Both sit on the target rather than on an alternative because neither is a
    /// function of a plan: one request declares one profile, and the feasibility
    /// authority applies one rule set to every candidate it assesses. They are
    /// lifted from the portfolio rather than re-derived from the request, so the
    /// value a consumer reads is the one the alternatives were actually built
    /// with; [`target_assessment`] proves the portfolio agrees on it.
    pub(crate) target_profile_descriptor: Vec<u8>,
    pub(crate) feasibility_rule_set: FeasibilityRuleSetIdentity,
    pub(crate) portfolio: ProgramPortfolio,
    #[cfg(test)]
    pub(crate) explain: VerifiedExplainTrace,
    #[cfg(test)]
    pub(crate) selection_explain: VerifiedExplainTrace,
    pub(crate) compilation_explain: crate::explain::VerifiedCompilationExplain,
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
/// The alternative owns one selected physical plan under one semantic origin.
/// Its exact identity binds that origin, the rewritten semantic authority, the
/// resolved contract, and the plan identity; its stable string is presentation
/// only. Its cost is the plan's exact aggregate structural cost. Nothing here
/// re-decides feasibility or legality; both were settled by the frontier and
/// the fusion-legality authority before the plan was retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramAlternative {
    pub(crate) stable_id: String,
    pub(crate) identity: ProgramAlternativeIdentity,
    owner_key: String,
    pub(crate) kind: ProgramAlternativeKind,
    pub(crate) plan: SelectedPlan,
    pub(crate) scheduled_regions: Vec<VerifiedScheduledRegion>,
    pub(crate) kernels: Vec<VerifiedKernel>,
    pub(crate) program: KernelProgram,
    pub(crate) artifact_plan: ArtifactConstructionPlan,
    pub(crate) structural_cost: PlanStructuralCost,
    pub(crate) equivalence: EquivalenceEvidence,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProgramAlternativeIdentity(Box<[u8]>);

impl ProgramAlternativeIdentity {
    fn new(
        origin: SemanticAlternativeOrigin,
        semantic: &tiler_ir::semantic::SemanticProgram,
        request: &crate::request::VerifiedTargetRequest,
        plan: &SelectedPlan,
    ) -> Self {
        debug_assert!(plan.identity().is_labelled(&plan.identity().label()));
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"tiler.program-alternative.v1\0");
        match origin {
            SemanticAlternativeOrigin::Baseline => bytes.push(1),
            SemanticAlternativeOrigin::Rewrite(rule) => {
                bytes.push(2);
                rule.encode(&mut bytes);
            }
        }
        let identity = semantic.semantic_identity();
        for component in [
            identity.graph().as_bytes(),
            identity.reached_definitions().as_bytes(),
            identity.admission_provenance().as_bytes(),
            identity.registry_snapshot().as_bytes(),
            request.numerical_contract().key.as_bytes(),
            plan.identity().as_bytes(),
        ] {
            push_slice(&mut bytes, component);
        }
        Self(bytes.into_boxed_slice())
    }

    fn label(&self) -> String {
        let digest = self.0.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
        format!("program-alternative:{digest:016x}")
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SemanticAlternativeOrigin {
    Baseline,
    Rewrite(RewriteRuleIdentity),
}

#[derive(Clone, Copy)]
struct SemanticAlternativeOwner<'a> {
    origin: SemanticAlternativeOrigin,
    key: &'a str,
}

#[derive(Clone, Debug)]
struct SemanticCandidate {
    key: String,
    origin: SemanticAlternativeOrigin,
    proposal: RewriteProposal<tiler_ir::semantic::SemanticProgram>,
    verified: crate::request::VerifiedCompilationRequest,
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
            // Stating a contract this build cannot realize is a malformed
            // request too, and for the same reason: no target was consulted, so
            // it is not a statement that any plan was infeasible.
            | RequestError::UnrepresentableNumericalDimension { .. }
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
    compile_with_algebraic_configuration(request, AlgebraicRuleConfiguration::all())
}

fn compile_with_algebraic_configuration(
    request: CompilationRequest<'_>,
    algebraic_configuration: AlgebraicRuleConfiguration,
) -> Result<CompilationProduct, CompileError> {
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
    let canonical = normalization.normalized_program().unwrap_or(semantic);
    let exploration = explore_algebraic_alternatives_owned(
        canonical.clone(),
        verified.budgets(),
        resolved_contract,
        algebraic_configuration,
    )?;
    let baseline_rule =
        RewriteRuleIdentity::new("tiler.pipeline", "canonical-semantic-baseline.v1", 1)
            .expect("the baseline identity is valid");
    let AlgebraicExplorationParts {
        baseline,
        alternatives,
        assessments,
        budget_stop,
    } = exploration.into_parts();
    let mut proposals = Vec::with_capacity(alternatives.len() + 1);
    proposals.push(RewriteProposal::new(baseline_rule, baseline, 0));
    proposals.extend(alternatives);
    let readmitted = readmit_alternatives(proposals, |candidate| {
        verify_request(CompilationRequest {
            program: candidate,
            shape_environment,
            numerical_contracts: verified.numerical_contracts().clone(),
            budgets: verified.budgets(),
            target_profiles: target_profiles.clone(),
            capabilities: capabilities.clone(),
        })
        .ok()
    })?;
    let candidates = readmitted
        .into_iter()
        .enumerate()
        .map(|(index, (proposal, readmitted))| {
            verify_semantic_output_type(proposal.candidate())?;
            let origin = if index == 0 {
                SemanticAlternativeOrigin::Baseline
            } else {
                SemanticAlternativeOrigin::Rewrite(proposal.rule())
            };
            let key = match origin {
                SemanticAlternativeOrigin::Baseline => "semantic:baseline".to_owned(),
                SemanticAlternativeOrigin::Rewrite(rule) => format!(
                    "semantic:p{}:{}.r{}:{}@{}",
                    rule.provider().len(),
                    rule.provider(),
                    rule.rule().len(),
                    rule.rule(),
                    rule.revision()
                ),
            };
            Ok(SemanticCandidate {
                key,
                origin,
                proposal,
                verified: readmitted,
            })
        })
        .collect::<Result<Vec<_>, CompileError>>()?;
    compile_verified(
        &verified,
        &normalization,
        &assessments,
        budget_stop,
        &candidates,
    )
}

fn compile_verified(
    original: &crate::request::VerifiedCompilationRequest,
    normalization: &NormalizationOutcome,
    assessments: &[RewriteAssessment],
    budget_stop: Option<(u64, u64)>,
    candidates: &[SemanticCandidate],
) -> Result<CompilationProduct, CompileError> {
    let targets = original
        .target_profiles()
        .iter()
        .copied()
        .map(|target| {
            let original_target = original.for_target(target)?;
            compile_semantic_portfolio_target(
                &original_target,
                target,
                normalization,
                assessments,
                budget_stop,
                candidates,
            )
        })
        .collect::<Result<_, _>>()?;
    Ok(CompilationProduct { targets })
}

struct CandidateTargetCompilation {
    key: String,
    request: crate::request::VerifiedTargetRequest,
    portfolio: Option<ProgramPortfolio>,
    explain: VerifiedExplainTrace,
    failure: Option<Box<CompileError>>,
}

struct ExpectedCandidateOwner<'a> {
    key: String,
    origin: SemanticAlternativeOrigin,
    semantic: &'a tiler_ir::semantic::SemanticProgram,
    request: crate::request::VerifiedTargetRequest,
    alternatives: std::collections::BTreeSet<ProgramAlternativeIdentity>,
}

#[derive(Debug)]
struct PreferredGroupEvaluation<Item, Output> {
    selected_contract: Option<&'static str>,
    evaluated: Vec<Output>,
    pruned: Vec<(Item, &'static str)>,
}

fn evaluate_preferred_groups<Item, Output, Error>(
    stated: &[crate::request::StrictF32NumericalContract],
    mut groups: Vec<(&'static str, Vec<Item>)>,
    mut evaluate: impl FnMut(Item) -> Result<Output, Error>,
    feasible: impl Fn(&Output) -> bool,
    unstated: impl Fn(&'static str) -> Error,
) -> Result<PreferredGroupEvaluation<Item, Output>, Error> {
    let mut evaluated = Vec::new();
    let mut pruned = Vec::new();
    let mut selected_contract = None;
    for contract in stated {
        let Some(position) = groups.iter().position(|(key, _)| *key == contract.key) else {
            continue;
        };
        let (_, group) = groups.remove(position);
        if let Some(preferred) = selected_contract {
            pruned.extend(group.into_iter().map(|item| (item, preferred)));
            continue;
        }
        let start = evaluated.len();
        for item in group {
            evaluated.push(evaluate(item)?);
        }
        if evaluated[start..].iter().any(&feasible) {
            selected_contract = Some(contract.key);
        }
    }
    if let Some((key, _)) = groups.first() {
        return Err(unstated(key));
    }
    Ok(PreferredGroupEvaluation {
        selected_contract,
        evaluated,
        pruned,
    })
}

fn compile_candidate_target(
    candidate: &SemanticCandidate,
    verified: &crate::request::VerifiedTargetRequest,
) -> Result<CandidateTargetCompilation, CompileError> {
    let mut explain = ExplainWriter::new(verified)?;
    match compile_target_with_explain(candidate, verified, &mut explain) {
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
            Ok(CandidateTargetCompilation {
                key: candidate.key.clone(),
                request: verified.clone(),
                portfolio: Some(portfolio),
                explain,
                failure: None,
            })
        }
        Err(failure) => {
            let explain = explain.finish_failure(*failure.context)?;
            if matches!(failure.source.as_ref(), CompileError::NoFeasiblePlan(_)) {
                return Ok(CandidateTargetCompilation {
                    key: candidate.key.clone(),
                    request: verified.clone(),
                    portfolio: None,
                    explain,
                    failure: Some(failure.source),
                });
            }
            Err(CompileError::Explained {
                source: failure.source,
                explain,
            })
        }
    }
}

fn compile_semantic_portfolio_target(
    original: &crate::request::VerifiedTargetRequest,
    target: crate::request::PrototypeTargetProfile,
    normalization: &NormalizationOutcome,
    assessments: &[RewriteAssessment],
    budget_stop: Option<(u64, u64)>,
    candidates: &[SemanticCandidate],
) -> Result<TargetCompilationProduct, CompileError> {
    let targeted = candidates
        .iter()
        .map(|candidate| {
            let request = candidate.verified.for_target(target)?;
            Ok((candidate, request))
        })
        .collect::<Result<Vec<_>, RequestError>>()?;
    let groups =
        group_by_resolved_contract(targeted, |(_, request)| request.numerical_contract().key);
    let evaluation = evaluate_preferred_groups(
        original.numerical_contracts().stated(),
        groups,
        |(candidate, request)| {
            let compiled = compile_candidate_target(candidate, &request)?;
            Ok::<_, CompileError>((compiled, candidate, request))
        },
        |(compiled, _, _)| compiled.portfolio.is_some(),
        |_| {
            CompileError::InvalidCompilerOutput(CompilerOutputError::Program(
                ProgramError::Structure {
                    rule: "semantic-portfolio-unstated-contract",
                },
            ))
        },
    )?;
    let Some(selected_contract) = evaluation.selected_contract else {
        let compiled = evaluation
            .evaluated
            .iter()
            .map(|(compiled, _, _)| compiled)
            .collect::<Vec<_>>();
        let baseline = compiled
            .iter()
            .find(|candidate| candidate.key == "semantic:baseline")
            .ok_or(RequestError::UnsupportedCapability {
                phase: "numerics",
                rule: "semantic-portfolio-contract-group",
            })?;
        return Err(CompileError::Explained {
            source: baseline
                .failure
                .clone()
                .expect("an infeasible baseline retains its failure"),
            explain: baseline.explain.clone(),
        });
    };
    let mut compiled = evaluation
        .evaluated
        .into_iter()
        .map(|(compiled, _, _)| compiled)
        .collect::<Vec<_>>();
    let pruned = evaluation
        .pruned
        .into_iter()
        .map(|((candidate, request), preferred)| (candidate, request, preferred))
        .collect::<Vec<_>>();
    let mut alternative_candidates = std::collections::BTreeMap::new();
    let mut expected = Vec::new();
    for candidate in &compiled {
        if let Some(portfolio) = &candidate.portfolio {
            let semantic = candidates
                .iter()
                .find(|owner| owner.key == candidate.key)
                .expect("every evaluated candidate retains its semantic owner");
            for alternative in &portfolio.alternatives {
                alternative_candidates.insert(alternative.identity.clone(), candidate.key.clone());
            }
            expected.push(ExpectedCandidateOwner {
                key: candidate.key.clone(),
                origin: semantic.origin,
                semantic: semantic.proposal.candidate(),
                request: candidate.request.clone(),
                alternatives: portfolio
                    .alternatives
                    .iter()
                    .map(|alternative| alternative.identity.clone())
                    .collect(),
            });
        }
    }
    let mut alternatives = compiled
        .iter_mut()
        .filter_map(|candidate| candidate.portfolio.as_mut())
        .flat_map(|portfolio| std::mem::take(&mut portfolio.alternatives))
        .collect::<Vec<_>>();
    if alternatives.is_empty() {
        let baseline = compiled
            .iter()
            .find(|candidate| candidate.key == "semantic:baseline")
            .expect("the semantic portfolio always retains its baseline");
        return Err(CompileError::Explained {
            source: baseline
                .failure
                .clone()
                .expect("an infeasible candidate retains its failure"),
            explain: baseline.explain.clone(),
        });
    }
    alternatives.sort_by(|left, right| left.identity.cmp(&right.identity));
    let selected = select_global_non_dominated(&alternatives)?
        .stable_id
        .clone();
    let portfolio = ProgramPortfolio {
        alternatives,
        selection: PortfolioSelection {
            policy_key: SELECTION_POLICY_KEY,
            selected_alternative_id: selected.clone(),
        },
    };
    verify_global_portfolio(&portfolio, &expected)?;
    let mut selection = ExplainWriter::new(original)?;
    let request_record = record_request_verification(&mut selection)?;
    let mut cause = normalization.record(&mut selection, request_record)?;
    cause =
        record_algebraic_exploration(&mut selection, cause, assessments, budget_stop, candidates)?;
    for (candidate, request, preferred) in &pruned {
        let subject = selection.subject(SubjectKind::Alternative, &candidate.key)?;
        cause = selection.push_detail(
            RuleRef::builtin("semantic.contract-preference")?,
            vec![subject],
            ExplainEvent::PreferencePruned {
                preferred_contract: ReasonCode::new(preferred)?,
                candidate_contract: ReasonCode::new(request.numerical_contract().key)?,
            },
            vec![cause],
        )?;
    }
    let selected_identity = &portfolio
        .alternatives
        .iter()
        .find(|alternative| alternative.stable_id == selected)
        .expect("the selected alternative belongs to the portfolio")
        .identity;
    let selected_candidate = alternative_candidates
        .get(selected_identity)
        .expect("every flattened alternative retains its semantic owner")
        .as_str();
    for candidate in &compiled {
        let subject = selection.subject(SubjectKind::Alternative, &candidate.key)?;
        if candidate.portfolio.is_some() {
            let retains_tradeoff = portfolio.alternatives.iter().any(|alternative| {
                alternative_candidates.get(&alternative.identity) == Some(&candidate.key)
                    && globally_non_dominated(alternative, &portfolio.alternatives)
            });
            selection.note_semantic_selection(
                subject,
                &candidate.request,
                if candidate.key == selected_candidate {
                    SelectionOutcome::Selected
                } else if retains_tradeoff {
                    SelectionOutcome::NotSelectedTradeoff
                } else {
                    SelectionOutcome::Dominated
                },
                Some(TerminalCause::from_record(cause)),
            )?;
        } else {
            selection.note_semantic_infeasible(
                subject,
                &candidate.request,
                Some(TerminalCause::from_record(cause)),
            )?;
        }
    }
    let keys = compiled
        .iter()
        .filter(|candidate| candidate.portfolio.is_some())
        .map(|candidate| candidate.key.as_str())
        .collect::<Vec<_>>();
    let selection = selection.finish_semantic_portfolio(&keys, selected_candidate)?;
    #[cfg(test)]
    let selection_explain = selection.clone();
    #[cfg(test)]
    let explain = compiled
        .iter()
        .find(|candidate| candidate.key == selected_candidate)
        .expect("the selected semantic candidate has a trace")
        .explain
        .clone();
    let compilation_explain = crate::explain::VerifiedCompilationExplain::from_traces(
        selection,
        compiled
            .into_iter()
            .map(|candidate| {
                Ok((
                    crate::explain::SubjectKey::new(candidate.key)?,
                    candidate.explain,
                ))
            })
            .collect::<Result<Vec<_>, ExplainError>>()?,
    )
    .map_err(|_| ExplainError::StaleIdentity)?;
    let (target_profile_descriptor, feasibility_rule_set) = target_assessment(&portfolio)?;
    Ok(TargetCompilationProduct {
        stated_contracts: original.numerical_contracts().stated().to_vec(),
        resolved_contract: original
            .numerical_contracts()
            .stated()
            .iter()
            .find(|contract| contract.key == selected_contract)
            .copied()
            .expect("selected contract came from the stated preference"),
        target_profile_key: original.target_profile().key,
        target_profile_descriptor,
        feasibility_rule_set,
        portfolio,
        #[cfg(test)]
        explain,
        #[cfg(test)]
        selection_explain,
        compilation_explain,
    })
}

fn record_algebraic_exploration(
    selection: &mut ExplainWriter,
    mut cause: ExplainRecordId,
    assessments: &[RewriteAssessment],
    budget_stop: Option<(u64, u64)>,
    candidates: &[SemanticCandidate],
) -> Result<ExplainRecordId, ExplainError> {
    for assessment in assessments {
        let adopted = candidates.iter().find(|candidate| {
            candidate.origin == SemanticAlternativeOrigin::Rewrite(assessment.rule())
        });
        let subject = selection.subject(
            if adopted.is_some() {
                SubjectKind::Alternative
            } else {
                SubjectKind::Capability
            },
            adopted.map_or_else(
                || rewrite_assessment_key(*assessment),
                |candidate| candidate.key.clone(),
            ),
        )?;
        cause = record_rewrite_assessment(selection, *assessment, subject, vec![cause])?;
    }
    if let Some((limit, actual)) = budget_stop {
        let subject = selection.subject(SubjectKind::Normalization, "algebraic-portfolio")?;
        cause = selection.push_detail(
            RuleRef::builtin("rewrite.portfolio-budget")?,
            vec![subject],
            ExplainEvent::BudgetStop {
                stage: ExplainStage::CandidateEnumeration,
                resource: crate::explain::ResourceKey::new("normalization-rewrites")?,
                limit,
                actual,
            },
            vec![cause],
        )?;
    }
    Ok(cause)
}

fn select_global_non_dominated(
    alternatives: &[ProgramAlternative],
) -> Result<&ProgramAlternative, CompileError> {
    let identities = alternatives
        .iter()
        .map(|alternative| &alternative.identity)
        .collect::<std::collections::BTreeSet<_>>();
    let labels = alternatives
        .iter()
        .map(|alternative| alternative.stable_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if alternatives.is_empty()
        || identities.len() != alternatives.len()
        || labels.len() != alternatives.len()
    {
        return Err(ProgramError::Structure {
            rule: "semantic-portfolio-identity",
        }
        .into());
    }
    alternatives
        .iter()
        .filter(|candidate| globally_non_dominated(candidate, alternatives))
        .min_by(|left, right| left.identity.cmp(&right.identity))
        .ok_or_else(|| {
            ProgramError::Structure {
                rule: "semantic-portfolio-empty",
            }
            .into()
        })
}

fn globally_non_dominated(
    candidate: &ProgramAlternative,
    alternatives: &[ProgramAlternative],
) -> bool {
    !alternatives.iter().any(|other| {
        other.identity != candidate.identity
            && other.structural_cost.dominates(&candidate.structural_cost)
    })
}

fn verify_global_portfolio(
    portfolio: &ProgramPortfolio,
    expected: &[ExpectedCandidateOwner<'_>],
) -> Result<(), CompileError> {
    let expected_identities = expected
        .iter()
        .flat_map(|owner| owner.alternatives.iter())
        .collect::<std::collections::BTreeSet<_>>();
    let actual_identities = portfolio
        .alternatives
        .iter()
        .map(|alternative| &alternative.identity)
        .collect::<std::collections::BTreeSet<_>>();
    if expected_identities != actual_identities {
        return Err(ProgramError::Structure {
            rule: "semantic-portfolio-owner-set",
        }
        .into());
    }
    for alternative in &portfolio.alternatives {
        let owner = expected
            .iter()
            .find(|owner| owner.alternatives.contains(&alternative.identity))
            .ok_or(ProgramError::Structure {
                rule: "semantic-portfolio-owner",
            })?;
        let identity = ProgramAlternativeIdentity::new(
            owner.origin,
            owner.semantic,
            &owner.request,
            &alternative.plan,
        );
        if alternative.owner_key != owner.key || alternative.identity != identity {
            return Err(ProgramError::Structure {
                rule: "semantic-portfolio-owner-binding",
            }
            .into());
        }
    }
    verify_global_selection(portfolio)
}

fn verify_global_selection(portfolio: &ProgramPortfolio) -> Result<(), CompileError> {
    let selected = select_global_non_dominated(&portfolio.alternatives)?;
    if portfolio.selection.policy_key != SELECTION_POLICY_KEY
        || portfolio.selection.selected_alternative_id != selected.stable_id
    {
        return Err(ProgramError::Structure {
            rule: "semantic-portfolio-selection",
        }
        .into());
    }
    Ok(())
}

/// Lifts the compilation-invariant assessment identities off a target's portfolio.
///
/// The profile descriptor and the feasibility rule set are properties of the
/// target request, not of a plan, so they are read once per target and carried
/// beside the portfolio rather than once per alternative. Every alternative
/// still records its own copy, and this refuses a portfolio whose alternatives
/// disagree instead of picking one: the returned pair becomes half of an
/// artifact's `TargetProfileRef` and its whole `FeasibilityRuleSetRef` under ADR
/// 0072, so a silent choice between two candidate identities would put a claim
/// into artifact identity that some retained alternative never made.
///
/// A disagreement is a Tiler defect rather than a rejected program, which is why
/// it classifies as invalid compiler output. `verify_alternative` independently
/// re-derives each plan from the one request, so reaching either refusal here
/// means that verification did not run or did not hold.
fn target_assessment(
    portfolio: &ProgramPortfolio,
) -> Result<(Vec<u8>, FeasibilityRuleSetIdentity), CompileError> {
    let Some((first, rest)) = portfolio.alternatives.split_first() else {
        return Err(ProgramError::Structure {
            rule: "portfolio-empty",
        }
        .into());
    };
    let descriptor = first.artifact_plan.target_profile_descriptor();
    let rules = first.artifact_plan.feasibility_rule_set();
    if rest.iter().any(|alternative| {
        alternative.artifact_plan.target_profile_descriptor() != descriptor
            || alternative.artifact_plan.feasibility_rule_set() != rules
    }) {
        return Err(ProgramError::Structure {
            rule: "portfolio-target-assessment",
        }
        .into());
    }
    Ok((descriptor.to_vec(), rules))
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
    if let Some(pointwise) = request.pointwise() {
        return if members == pointwise.members {
            "whole-program"
        } else {
            "unrecognized"
        };
    }
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

fn record_request_verification(
    explain: &mut ExplainWriter,
) -> Result<ExplainRecordId, CompileError> {
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
}

#[allow(
    clippy::too_many_lines,
    reason = "keeps the phase-local failure contexts beside the target compilation transaction"
)]
fn compile_target_with_explain(
    candidate: &SemanticCandidate,
    verified: &crate::request::VerifiedTargetRequest,
    explain: &mut ExplainWriter,
) -> Result<ProgramPortfolio, TargetFailure> {
    let semantic = candidate.proposal.candidate();
    let request_record = record_request_verification(explain).map_err(|source| {
        target_failure(
            source,
            ExplainStage::RequestVerification,
            "explain-request-verification",
            SubjectKind::SemanticProgram,
            "semantic-program",
            None,
        )
    })?;
    let adopted = match candidate.origin {
        SemanticAlternativeOrigin::Baseline => &[][..],
        SemanticAlternativeOrigin::Rewrite(_) => std::slice::from_ref(&candidate.proposal),
    };
    let semantic_record = explain_step(
        record_adopted_alternatives(adopted, explain, request_record).map_err(CompileError::from),
        ExplainStage::CandidateEnumeration,
        SubjectKind::Alternative,
        &candidate.key,
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
                    record_cause(semantic_record),
                )
            })?;
    let region_records = explain_step(
        formation
            .record(explain, semantic_record)
            .map_err(CompileError::from),
        ExplainStage::RegionFormation,
        SubjectKind::Region,
        REGION_FORMATION_SUBJECT,
        record_cause(semantic_record),
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
        let alternative = build_alternative_for_origin(
            semantic,
            verified,
            SemanticAlternativeOwner {
                origin: candidate.origin,
                key: &candidate.key,
            },
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
    let selected_alternative_id = select_non_dominated(&plans.portfolio, &alternatives)
        .map_err(|source| {
            target_failure(
                source,
                ExplainStage::Selection,
                "portfolio-selection",
                SubjectKind::KernelProgram,
                "portfolio",
                record_cause(region_root),
            )
        })?
        .to_owned();
    verify_portfolio(
        semantic,
        verified,
        &formation,
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
                        "numerics-unhonourable-{}-{}-{}",
                        rejection.dimension().key().replace('.', "-"),
                        rejection
                            .arithmetic()
                            .canonical_type_key()
                            .replace("::", "-"),
                        rejection.required().key()
                    )
                },
            )
        }
        RequestError::UnrepresentableNumericalDimension { cause } => format!(
            "numerics-unrepresentable-{}-{}-{}",
            cause.dimension().key().replace('.', "-"),
            cause.arithmetic().canonical_type_key().replace("::", "-"),
            cause.required().key()
        ),
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

use crate::component_cost::{
    ANALYTICAL_MODEL_KEY, CANONICAL_COMPONENTS, ComponentCost, CostComponent, CostUnit, CostValue,
    analytical_plan_cost,
};

mod planning;
mod trace;
mod verify;

#[cfg(test)]
use planning::build_alternative;
use planning::{
    build_alternative_for_origin, build_plan_program, enumerate_complete_plans, plan_region_order,
    select_non_dominated,
};
use trace::{
    note_infeasible_cover, record_alternative_explain, record_cost_and_selection,
    record_cover_enumeration, record_frontier, record_fusion_legality, record_lowering,
    record_numerical_equivalence, record_plan_selection, record_target_rejection,
};
use verify::verify_portfolio;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod conformance;
