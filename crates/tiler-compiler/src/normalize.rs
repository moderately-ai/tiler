//! The deterministic `NormalizeSemantics` stage.
//!
//! Normalization runs after request verification and before region formation.
//! It produces one canonical semantic graph for a class of programs that differ
//! only in redundant spelling. Since the routing landed, the stage does that
//! *by driving the rewrite engine in this module* with the common-subexpression
//! rule registered. That rule produces at most one canonical proposal.
//! Costed algebraic alternatives are explored only after this phase.
//!
//! The first profile proves exactly one rule, common-subexpression elimination
//! over referentially transparent operations. Breadth is deliberately not the
//! goal; the machinery and its guarantees are:
//!
//! - **Termination.** Detection is a single forward pass over a finite verified
//!   operation list. There is no fixpoint loop, so termination does not depend
//!   on a measure decreasing.
//! - **Traversal order.** Operations are visited in verified topological order
//!   by ascending graph-local ordinal, and results within an operation by
//!   ascending result position. The earliest occurrence of a congruence class
//!   always becomes its canonical representative.
//! - **Budgets.** [`DeterministicBudgets::normalization_rewrites`] bounds the
//!   rewrites one compilation may commit. Exhaustion abandons the whole rewrite
//!   and keeps the verified input program, so a budget never yields a partially
//!   canonicalized graph.
//! - **Transactional failure.** The input [`SemanticProgram`] is immutable and
//!   is never mutated. A rewrite is built as a separate candidate program and is
//!   adopted only after it passes every postcondition.
//! - **Semantic revalidation.** The candidate is rebuilt through the ordinary
//!   checked [`SemanticProgramBuilder`], so the frozen semantic authority
//!   re-infers and re-validates every operation. The stage never copies verified
//!   structure forward and never trusts its own output structurally.
//! - **Canonical identity.** The adopted program's `SemanticIdentity` is the
//!   canonical identity of the normalized result, and re-running detection on it
//!   is required to find no further rewrite.
//!
//! Reference revalidation is deliberately *not* performed inside this stage.
//! `tiler-reference` is an executable oracle whose cost is proportional to the
//! materialized element count, and the compiler admits programs whose element
//! counts reach billions, so evaluating every rewrite at compile time is not a
//! viable contract. Differential reference equivalence between the input and the
//! normalized program is instead proven by the checked conformance tests in this
//! module, which is where this crate's reference dependency lives.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use tiler_ir::schedule::NumericalPermission;
use tiler_ir::semantic::{
    Definition, OpKey, OperationAttributes, OperationEffect, OperationId, ResolvedValueType,
    SemanticProgram, SemanticProgramBuilder, ValueFact, ValueId, add_f32_op, multiply_f32_op,
};
use tiler_ir::shape::Shape;

use crate::explain::{
    EvidenceBasis, ExplainError, ExplainEvent, ExplainFact, ExplainRecordId, ExplainStage,
    ExplainWriter, FactValue, PredicateAssessment, ProviderRef, ReasonCode, RejectionClass,
    ResourceKey, RuleRef, SubjectKey, SubjectKind, SubjectRef,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};
use crate::rewrite::{
    COMMON_SUBEXPRESSION_RULE, ORDERED_REASSOCIATE_ADD_RULE, ORDERED_REASSOCIATE_MULTIPLY_RULE,
    ProviderDefect, RewriteAssessment, RewriteAssessmentClass, RewriteProposal,
    RewriteRuleIdentity, RewriteRuleProvider, RuleExplain, RuleRegistry, collect_proposals,
};
use crate::{policy::operation_capability, target::honourability::DimensionBehaviour};
use std::sync::Arc;

/// Stable identity of the normalization stage rule.
pub(crate) const NORMALIZE_STAGE_RULE: &str = "normalize.semantics.v1";
/// Stable identity of the one proved rewrite in the first profile.
pub(crate) const NORMALIZE_SHARED_VALUE_RULE: &str = "normalize.common-subexpression.v1";
/// Stable subject key for whole-program normalization records.
pub(crate) const NORMALIZATION_SUBJECT: &str = "normalization:program";
const REWRITE_BUDGET_RESOURCE: &str = "normalization-rewrites";

/// Typed failure of the deterministic normalization stage.
///
/// Every variant is invalid compiler output rather than a rejected user program:
/// the stage only ever observes an already verified [`SemanticProgram`], and a
/// rewrite it produced failing revalidation is a compiler defect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizeError {
    /// The verified input program violated a stage precondition.
    Structure { rule: &'static str },
    /// The checked semantic builder rejected the candidate rewrite.
    Rebuild { rule: &'static str },
    /// The rebuilt program violated a normalization postcondition.
    InvalidRewrite { rule: &'static str },
}

impl NormalizeError {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Structure { rule } | Self::Rebuild { rule } | Self::InvalidRewrite { rule } => {
                rule
            }
        }
    }

    const fn class(self) -> &'static str {
        match self {
            Self::Structure { .. } => "structure",
            Self::Rebuild { .. } => "rebuild",
            Self::InvalidRewrite { .. } => "invalid-rewrite",
        }
    }
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compile.normalize.{}.{}: deterministic normalization produced invalid compiler output",
            self.class(),
            self.reason()
        )
    }
}

impl Error for NormalizeError {}

/// One committed merge of a redundant operation into its canonical occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SharedValueMerge {
    /// Graph-local ordinal of the retained canonical operation.
    canonical: usize,
    /// Graph-local ordinal of the redundant operation it replaced.
    merged: usize,
}

/// The common-subexpression rule's own explain records.
///
/// One [`RuleExplain`] implementation with one emitter: [`NormalizationOutcome`]
/// carries it as an opaque payload (handed over by the adopted proposal) and
/// emits it in `record`. Extracted rather than duplicated because two code paths
/// writing the same governed records under the same rule key would drift, and
/// the drift would be invisible — an explain reader cannot tell which path
/// produced a record.
///
/// The facts are the rule's, not the stage's: `canonical-operation` and
/// `merged-operation` are graph-local ordinals of the *original* program, which
/// is why the engine cannot reconstruct them from a candidate and why the rule
/// hands them over instead.
#[derive(Clone, Debug)]
pub(crate) struct SharedValueExplain {
    merges: Vec<SharedValueMerge>,
}

impl SharedValueExplain {
    /// Captures the merges a rewrite committed.
    pub(crate) const fn new(merges: Vec<SharedValueMerge>) -> Self {
        Self { merges }
    }
}

impl RuleExplain for SharedValueExplain {
    fn record(
        &self,
        explain: &mut ExplainWriter,
        mut cause: ExplainRecordId,
    ) -> Result<ExplainRecordId, ExplainError> {
        for merge in &self.merges {
            let key = format!("normalization:shared-value/operation:{}", merge.merged);
            let subject = explain.subject(SubjectKind::Normalization, &key)?;
            let assessment = PredicateAssessment::proven(
                "normalize.shared-value-identity",
                EvidenceBasis::CheckedInvariant,
            )?
            .with_fact(ExplainFact::new(
                "canonical-operation",
                FactValue::Count(count(merge.canonical)),
            )?)?
            .with_fact(ExplainFact::new(
                "merged-operation",
                FactValue::Count(count(merge.merged)),
            )?)?;
            cause = explain.push_detail(
                RuleRef::builtin(NORMALIZE_SHARED_VALUE_RULE)?,
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::Normalization,
                    assessment,
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                vec![cause],
            )?;
        }
        Ok(cause)
    }
}

/// The deterministic result of running `NormalizeSemantics` once.
#[derive(Clone, Debug)]
pub(crate) struct NormalizationOutcome {
    normalized: Option<SemanticProgram>,
    /// The adopted rewrites' own explain records, opaque to this type.
    ///
    /// Holding payloads rather than `SharedValueMerge`s is what lets this stage
    /// be produced by the rewrite engine: merges are the common-subexpression
    /// rule's own vocabulary, and an engine driving arbitrary rules has none.
    /// A rule hands over what it wants recorded; this emits it and never reads
    /// it.
    rule_explains: Vec<Arc<dyn RuleExplain>>,
    /// Rewrites adopted. Equal to the merge count while one rule is registered,
    /// and a different quantity as soon as a second is.
    rewrite_count: usize,
    operations_before: usize,
    operations_after: usize,
    budget_stop: Option<(u64, u64)>,
    numerical_contract_key: &'static str,
    canonical_graph_digest: u64,
}

impl NormalizationOutcome {
    /// Returns the adopted canonical program, or `None` when the verified input
    /// was already canonical for this profile.
    pub(crate) const fn normalized_program(&self) -> Option<&SemanticProgram> {
        self.normalized.as_ref()
    }

    /// Returns how many rewrites were adopted.
    #[cfg(test)]
    pub(crate) const fn rewrite_count(&self) -> usize {
        self.rewrite_count
    }

    /// Returns the declared budget and the demand that stopped the rewrite.
    #[cfg(test)]
    pub(crate) const fn budget_stop(&self) -> Option<(u64, u64)> {
        self.budget_stop
    }

    /// Emits this outcome through the typed explain authority.
    ///
    /// Records form one linear causal chain rooted at `cause` so a record never
    /// accumulates an unbounded cause set, and the returned identifier is the
    /// stage receipt later stages depend on.
    pub(crate) fn record(
        &self,
        explain: &mut ExplainWriter,
        mut cause: ExplainRecordId,
    ) -> Result<ExplainRecordId, ExplainError> {
        if let Some((limit, actual)) = self.budget_stop {
            let subject = explain.subject(SubjectKind::Normalization, NORMALIZATION_SUBJECT)?;
            cause = explain.push_detail(
                RuleRef::builtin(NORMALIZE_STAGE_RULE)?,
                vec![subject],
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::Normalization,
                    resource: ResourceKey::new(REWRITE_BUDGET_RESOURCE)?,
                    limit,
                    actual,
                },
                vec![cause],
            )?;
        }
        for records in &self.rule_explains {
            cause = records.record(explain, cause)?;
        }
        let assessment = PredicateAssessment::proven(
            "normalize.canonical-fixpoint",
            EvidenceBasis::CheckedInvariant,
        )?
        .with_fact(ExplainFact::new(
            "rewrite-count",
            FactValue::Count(count(self.rewrite_count)),
        )?)?
        .with_fact(ExplainFact::new(
            "operations-before",
            FactValue::Count(count(self.operations_before)),
        )?)?
        .with_fact(ExplainFact::new(
            "operations-after",
            FactValue::Count(count(self.operations_after)),
        )?)?
        .with_fact(ExplainFact::new(
            "numerical-contract",
            FactValue::Identity(SubjectKey::new(self.numerical_contract_key)?),
        )?)?
        .with_fact(ExplainFact::new(
            "canonical-graph-digest",
            FactValue::Identity(SubjectKey::new(format!(
                "{:016x}",
                self.canonical_graph_digest
            ))?),
        )?)?;
        let subject = explain.subject(SubjectKind::Normalization, NORMALIZATION_SUBJECT)?;
        explain.push_detail(
            RuleRef::builtin(NORMALIZE_STAGE_RULE)?,
            vec![subject],
            ExplainEvent::Check {
                stage: ExplainStage::Normalization,
                assessment,
                rejection: RejectionClass::IntrinsicInvalid,
            },
            vec![cause],
        )
    }
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn sole_canonical_alternative<T>(alternatives: &[T]) -> Result<&T, NormalizeError> {
    let [alternative] = alternatives else {
        return Err(NormalizeError::InvalidRewrite {
            rule: "canonical-provider-cardinality",
        });
    };
    Ok(alternative)
}

/// Runs the deterministic normalization stage over one verified program.
///
/// The input is never mutated. When a rewrite is committed the returned outcome
/// carries a separately built and fully revalidated program whose semantic
/// identity is the canonical identity of the normalized result.
pub(crate) fn normalize_semantics(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
) -> Result<NormalizationOutcome, NormalizeError> {
    let operations_before = program.operation_count();
    let mut registry = RuleRegistry::new();
    registry
        .register(Box::new(CommonSubexpressionRule))
        .map_err(|_| NormalizeError::Structure {
            rule: "rule-registration",
        })?;

    // Every engine failure is invalid compiler output. The `NormalizeError`
    // *class* does not survive the round trip — `ProviderDefect` has no business
    // knowing this stage's taxonomy — and that is accepted rather than worked
    // around: `class()` reaches only `Display`, never an explain record or a
    // typed outcome, so the loss is diagnostic text while the actionable reason
    // is preserved exactly.
    // The mapping drops the failing rule's identity: `NormalizeError` has no
    // field for it, and with one hard-coded rule there is nothing to exclude.
    // Algebraic exploration retains its rule identities separately; this
    // canonical phase still has one hard-coded rule and therefore no ambiguity.
    let run = run_rewrite_engine(&registry, program, budgets).map_err(|failure| match failure {
        EngineFailure::Provider(ProviderDefect::Failed { reason, .. })
        | EngineFailure::Revalidation { reason, .. } => {
            NormalizeError::InvalidRewrite { rule: reason }
        }
        EngineFailure::Provider(ProviderDefect::Misattributed { .. }) => {
            NormalizeError::InvalidRewrite {
                rule: "misattributed-proposal",
            }
        }
    })?;

    let unchanged = |budget_stop| NormalizationOutcome {
        normalized: None,
        rule_explains: Vec::new(),
        rewrite_count: 0,
        operations_before,
        operations_after: operations_before,
        budget_stop,
        numerical_contract_key: numerical_contract.key,
        canonical_graph_digest: digest(program),
    };

    let alternatives = match run {
        EngineRun::BudgetStopped { limit, demand } => return Ok(unchanged(Some((limit, demand)))),
        EngineRun::Adopted(alternatives) => alternatives,
    };
    if alternatives.is_empty() {
        return Ok(unchanged(None));
    }
    // This canonical phase registers exactly one rule and that rule emits at
    // most one proposal. Algebraic alternatives are explored only after this
    // phase, where the portfolio owns their comparison.
    let adopted = sole_canonical_alternative(&alternatives)?;
    let normalized = adopted.candidate().clone();
    Ok(NormalizationOutcome {
        operations_before,
        operations_after: normalized.operation_count(),
        rule_explains: adopted.explain().cloned().into_iter().collect(),
        rewrite_count: usize::try_from(adopted.rewrites()).unwrap_or(usize::MAX),
        budget_stop: None,
        numerical_contract_key: numerical_contract.key,
        canonical_graph_digest: digest(&normalized),
        normalized: Some(normalized),
    })
}

/// Canonical equality of one operation occurrence.
///
/// This is exactly the accepted semantic-value equality: operation key, ordered
/// operand identities taken after congruence, canonical attributes, and ordered
/// inferred result types. Source origin — declaration position, handles, and
/// graph ownership — is deliberately absent so it can be preserved for
/// explanation without participating in equality. The numerical contract is a
/// whole-request property in this IR and is checked once by the caller rather
/// than repeated per occurrence.
#[derive(Debug, Eq, Hash, PartialEq)]
struct OperationSignature {
    key: OpKey,
    attributes: OperationAttributes,
    operands: Vec<usize>,
    results: Vec<(ResolvedValueType, Shape)>,
}

/// Congruence classes discovered by one deterministic detection pass.
struct Congruence {
    /// Canonical value ordinal for every graph-local value ordinal.
    representative: Vec<usize>,
    /// Whether each graph-local operation survives normalization.
    retained: Vec<bool>,
    /// Ordered result value ordinals of each graph-local operation.
    operation_results: Vec<Vec<usize>>,
    /// Committed merges in traversal order.
    merges: Vec<SharedValueMerge>,
}

/// The common-subexpression rule, exposed as an external rewrite provider.
///
/// The one rule the stage's engine drives today. It reuses this stage's own
/// detection and rebuild rather than reimplementing them. The provider-versus-
/// stage equality is pinned by `the_provider_proposes_exactly_what_this_stage_produces`
/// — the engine-versus-stage comparison became a tautology when the routing
/// made the stage *be* the engine, and was removed.
///
/// It performs **no** part of the transaction: no budget, no revalidation, no
/// adoption. Those stay in `normalize_semantics`, and a proposal returned here
/// is a candidate nothing has yet accepted.
pub(crate) struct CommonSubexpressionRule;

impl RewriteRuleProvider<SemanticProgram> for CommonSubexpressionRule {
    fn identity(&self) -> RewriteRuleIdentity {
        COMMON_SUBEXPRESSION_RULE.expect("the common-subexpression rule identity is named")
    }

    /// Detects shared values and, if any, rebuilds the program without them.
    ///
    /// The three outcomes are kept distinct, which is the whole reason this
    /// signature is fallible:
    ///
    /// - **no merges** — `Ok(vec![])`, the ordinary "nothing to do" case;
    /// - **detection or rebuild failed** — `Err`, a compiler fault carrying the
    ///   `NormalizeError`'s own stable reason;
    /// - **merges found and rebuilt** — one candidate program.
    ///
    /// The empty case is checked before rebuilding rather than after: rebuilding
    /// a program with no merges yields a copy that is semantically identical to
    /// its input, and proposing it would make the engine revalidate and compare
    /// a program that cannot differ. That is not merely wasteful — a proposal
    /// that is always available would make "this rule applies" meaningless.
    fn propose(
        &self,
        program: &SemanticProgram,
    ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
        let failed = |error: NormalizeError| ProviderDefect::Failed {
            rule: self.identity(),
            reason: error.reason(),
        };
        let congruence = detect_shared_values(program).map_err(failed)?;
        if congruence.merges.is_empty() {
            return Ok(Vec::new());
        }
        let candidate = rebuild(program, &congruence).map_err(failed)?;
        // The rule's own postconditions, checked here rather than by the
        // engine, because they are stated in terms of the `Congruence` — an
        // operation count of exactly `original - merges` means nothing to a
        // rule that does not merge. A generic engine has no congruence and
        // could not ask.
        //
        // This is what stops a rule from proposing a candidate it has not
        // checked, which is the concrete form of "unknown provider behaviour is
        // never optimizable merely because it is registered". The engine still
        // revalidates structurally; that is a different question from whether
        // the rule did what it claims.
        verify_normalized(program, &candidate, &congruence).map_err(failed)?;
        // The rule's own records travel with the proposal and are emitted only
        // if it is adopted; see `RuleExplain`.
        let rewrites = count(congruence.merges.len());
        Ok(vec![
            RewriteProposal::new(self.identity(), candidate, rewrites)
                .with_explain(Arc::new(SharedValueExplain::new(congruence.merges))),
        ])
    }
}

/// Which governed ordered-reassociation rules one exploration evaluates.
///
/// Configuration is explicit per rule rather than one portfolio switch. A
/// disabled member still receives a stable configuration assessment, so its
/// absence cannot be confused with a provider that was never considered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AlgebraicRuleConfiguration {
    add: bool,
    multiply: bool,
}

impl AlgebraicRuleConfiguration {
    /// Enables the complete governed algebraic portfolio.
    pub(crate) fn all() -> Self {
        Self {
            add: false,
            multiply: false,
        }
        .with(
            ORDERED_REASSOCIATE_ADD_RULE.expect("the add rule is named"),
            true,
        )
        .with(
            ORDERED_REASSOCIATE_MULTIPLY_RULE.expect("the multiply rule is named"),
            true,
        )
    }

    /// Enables or disables one governed member.
    #[must_use]
    pub(crate) fn with(mut self, rule: RewriteRuleIdentity, enabled: bool) -> Self {
        if let Some(add) = ORDERED_REASSOCIATE_ADD_RULE
            && rule == add
        {
            self.add = enabled;
        }
        if let Some(multiply) = ORDERED_REASSOCIATE_MULTIPLY_RULE
            && rule == multiply
        {
            self.multiply = enabled;
        }
        self
    }

    fn enabled(self, rule: RewriteRuleIdentity) -> bool {
        if let Some(add) = ORDERED_REASSOCIATE_ADD_RULE
            && rule == add
        {
            return self.add;
        }
        if let Some(multiply) = ORDERED_REASSOCIATE_MULTIPLY_RULE
            && rule == multiply
        {
            return self.multiply;
        }
        false
    }
}

/// Baseline-preserving result of one bounded algebraic exploration.
#[derive(Debug)]
pub(crate) struct AlgebraicExplorationOutcome {
    baseline: SemanticProgram,
    alternatives: Vec<RewriteProposal<SemanticProgram>>,
    assessments: Vec<RewriteAssessment>,
    budget_stop: Option<(u64, u64)>,
}

pub(crate) struct AlgebraicExplorationParts {
    pub(crate) baseline: SemanticProgram,
    pub(crate) alternatives: Vec<RewriteProposal<SemanticProgram>>,
    pub(crate) assessments: Vec<RewriteAssessment>,
    pub(crate) budget_stop: Option<(u64, u64)>,
}

impl AlgebraicExplorationOutcome {
    pub(crate) fn into_parts(self) -> AlgebraicExplorationParts {
        AlgebraicExplorationParts {
            baseline: self.baseline,
            alternatives: self.alternatives,
            assessments: self.assessments,
            budget_stop: self.budget_stop,
        }
    }

    /// The unchanged input candidate, retained regardless of all rule outcomes.
    #[cfg(test)]
    pub(crate) const fn baseline(&self) -> &SemanticProgram {
        &self.baseline
    }

    /// Structurally revalidated algebraic alternatives in canonical rule order.
    #[cfg(test)]
    pub(crate) fn alternatives(&self) -> &[RewriteProposal<SemanticProgram>] {
        &self.alternatives
    }

    /// Stable semantic, numerical, and configuration assessments.
    #[cfg(test)]
    pub(crate) fn assessments(&self) -> &[RewriteAssessment] {
        &self.assessments
    }

    /// The existing transaction's typed budget stop, if it abandoned the run.
    #[cfg(test)]
    pub(crate) const fn budget_stop(&self) -> Option<(u64, u64)> {
        self.budget_stop
    }
}

pub(crate) fn rewrite_assessment_key(assessment: RewriteAssessment) -> String {
    let rule = assessment.rule();
    format!(
        "rewrite.p{}:{}.r{}:{}@{}",
        rule.provider().len(),
        rule.provider(),
        rule.rule().len(),
        rule.rule(),
        rule.revision()
    )
}

pub(crate) fn record_rewrite_assessment(
    explain: &mut ExplainWriter,
    assessment: RewriteAssessment,
    subject: SubjectRef,
    causes: Vec<ExplainRecordId>,
) -> Result<ExplainRecordId, ExplainError> {
    let identity = assessment.rule();
    let stage = match assessment.class() {
        RewriteAssessmentClass::Semantic => ExplainStage::CandidateEnumeration,
        RewriteAssessmentClass::Numerical => ExplainStage::NumericalLegality,
        RewriteAssessmentClass::Configuration => ExplainStage::CapabilityResolution,
    };
    let predicate = match assessment.class() {
        RewriteAssessmentClass::Semantic => "rewrite.semantic-applicable",
        RewriteAssessmentClass::Numerical => "rewrite.numerically-legal",
        RewriteAssessmentClass::Configuration => "rewrite.configuration-enabled",
    };
    let rejection = match assessment.class() {
        RewriteAssessmentClass::Numerical => RejectionClass::NumericalIllegal,
        RewriteAssessmentClass::Semantic | RewriteAssessmentClass::Configuration => {
            RejectionClass::IntrinsicInvalid
        }
    };
    let facts = [
        ExplainFact::new(
            "rewrite-provider",
            FactValue::Identity(SubjectKey::new(identity.provider())?),
        )?,
        ExplainFact::new(
            "rewrite-rule",
            FactValue::Identity(SubjectKey::new(identity.rule())?),
        )?,
        ExplainFact::new(
            "rewrite-revision",
            FactValue::Count(u64::from(identity.revision())),
        )?,
        ExplainFact::new(
            "assessment-reason",
            FactValue::Identity(SubjectKey::new(assessment.reason())?),
        )?,
    ];
    let mut predicate = if assessment.is_accepted() {
        PredicateAssessment::proven(predicate, EvidenceBasis::CheckedInvariant)?
    } else {
        PredicateAssessment::disproved(
            predicate,
            ReasonCode::new(assessment.reason())?,
            EvidenceBasis::CheckedInvariant,
        )?
    };
    for fact in facts {
        predicate = predicate.with_fact(fact)?;
    }
    explain.push_detail(
        RuleRef::provided(
            rewrite_assessment_key(assessment),
            identity.revision(),
            ProviderRef::builtin(),
        )?,
        vec![subject],
        ExplainEvent::Check {
            stage,
            assessment: predicate,
            rejection,
        },
        causes,
    )
}

#[derive(Clone, Copy, Debug)]
struct OrderedReassociationRule {
    identity: RewriteRuleIdentity,
    operation: fn() -> OpKey,
}

impl OrderedReassociationRule {
    fn add() -> Self {
        Self {
            identity: ORDERED_REASSOCIATE_ADD_RULE.expect("the add rule is named"),
            operation: add_f32_op,
        }
    }

    fn multiply() -> Self {
        Self {
            identity: ORDERED_REASSOCIATE_MULTIPLY_RULE.expect("the multiply rule is named"),
            operation: multiply_f32_op,
        }
    }

    fn evaluate(
        self,
        program: &SemanticProgram,
        contract: &StrictF32NumericalContract,
    ) -> Result<PreparedAlgebraicRule, NormalizeError> {
        let operation = (self.operation)();
        let definition = program
            .semantic_registry()
            .operation_definition(&operation)
            .ok_or(NormalizeError::Structure {
                rule: "algebraic-operation-definition",
            })?;
        if !definition
            .algebraic_capabilities()
            .declares_ordered_associativity()
        {
            return Ok(PreparedAlgebraicRule::declined(
                operation,
                RewriteAssessment::declined(
                    self.identity,
                    RewriteAssessmentClass::Semantic,
                    "semantic.ordered-associativity-undeclared",
                ),
            ));
        }
        let Some(capability) = operation_capability(&operation) else {
            return Ok(PreparedAlgebraicRule::declined(
                operation,
                RewriteAssessment::declined(
                    self.identity,
                    RewriteAssessmentClass::Semantic,
                    "semantic.reassociation-not-consumable",
                ),
            ));
        };
        let Some(_site) = find_ordered_reassociation(program, &operation)? else {
            return Ok(PreparedAlgebraicRule::declined(
                operation,
                RewriteAssessment::declined(
                    self.identity,
                    RewriteAssessmentClass::Semantic,
                    "semantic.no-left-associated-chain",
                ),
            ));
        };
        let semantic = RewriteAssessment::accepted(
            self.identity,
            RewriteAssessmentClass::Semantic,
            "semantic.ordered-chain-preserved",
        );
        if capability.effective(
            crate::target::honourability::NumericalDimension::Reassociation,
            contract,
        ) != Some(DimensionBehaviour::Transform(
            NumericalPermission::Permitted,
        )) {
            return Ok(PreparedAlgebraicRule {
                operation,
                assessments: vec![
                    semantic,
                    RewriteAssessment::declined(
                        self.identity,
                        RewriteAssessmentClass::Numerical,
                        "numerical.reassociation-forbidden",
                    ),
                ],
            });
        }
        let numerical = RewriteAssessment::accepted(
            self.identity,
            RewriteAssessmentClass::Numerical,
            "numerical.reassociation-permitted",
        );
        let assessments = vec![semantic, numerical];
        Ok(PreparedAlgebraicRule {
            assessments,
            operation,
        })
    }
}

#[derive(Clone, Debug)]
struct PreparedAlgebraicRule {
    assessments: Vec<RewriteAssessment>,
    operation: OpKey,
}

impl PreparedAlgebraicRule {
    fn declined(operation: OpKey, assessment: RewriteAssessment) -> Self {
        Self {
            operation,
            assessments: vec![assessment],
        }
    }
}

impl RewriteRuleProvider<SemanticProgram> for PreparedAlgebraicRule {
    fn identity(&self) -> RewriteRuleIdentity {
        self.assessments[0].rule()
    }

    fn propose(
        &self,
        program: &SemanticProgram,
    ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
        if !self
            .assessments
            .iter()
            .all(|assessment| assessment.is_accepted())
        {
            return Ok(Vec::new());
        }
        let definition = program
            .semantic_registry()
            .operation_definition(&self.operation);
        if !definition.is_some_and(|definition| {
            definition
                .algebraic_capabilities()
                .declares_ordered_associativity()
        }) {
            return Ok(Vec::new());
        }
        let Some(site) = find_ordered_reassociation(program, &self.operation).map_err(|error| {
            ProviderDefect::Failed {
                rule: self.identity(),
                reason: error.reason(),
            }
        })?
        else {
            return Ok(Vec::new());
        };
        let candidate = rebuild_ordered_reassociation(program, &site).map_err(|error| {
            ProviderDefect::Failed {
                rule: self.identity(),
                reason: error.reason(),
            }
        })?;
        Ok(vec![
            RewriteProposal::new(self.identity(), candidate, 1).with_explain(Arc::new(
                AlgebraicRuleExplain {
                    assessments: self.assessments.clone(),
                },
            )),
        ])
    }
}

#[derive(Debug)]
struct AlgebraicRuleExplain {
    assessments: Vec<RewriteAssessment>,
}

impl RuleExplain for AlgebraicRuleExplain {
    fn record(
        &self,
        explain: &mut ExplainWriter,
        mut cause: ExplainRecordId,
    ) -> Result<ExplainRecordId, ExplainError> {
        debug_assert!(
            self.assessments
                .iter()
                .all(|assessment| assessment.is_accepted()),
            "only an accepted algebraic proposal may carry an adopted payload"
        );
        for assessment in &self.assessments {
            let subject =
                explain.subject(SubjectKind::Capability, rewrite_assessment_key(*assessment))?;
            cause = record_rewrite_assessment(explain, *assessment, subject, vec![cause])?;
        }
        Ok(cause)
    }
}

#[derive(Clone, Debug)]
struct OrderedReassociationSite {
    root: OperationId,
    operation: OpKey,
    attributes: OperationAttributes,
    ordered_leaves: [ValueId; 3],
    original_result: ValueId,
}

fn find_ordered_reassociation(
    program: &SemanticProgram,
    operation: &OpKey,
) -> Result<Option<OrderedReassociationSite>, NormalizeError> {
    for root in program
        .operations()
        .filter(|candidate| candidate.key() == operation)
    {
        let root_operands: Vec<_> = root.operands().collect();
        let root_results: Vec<_> = root.results().collect();
        let ([left_result, right], [original_result]) =
            (root_operands.as_slice(), root_results.as_slice())
        else {
            continue;
        };
        let Definition::OperationResult {
            operation: left_operation,
            ..
        } = program
            .value(*left_result)
            .map_err(|_| NormalizeError::Structure {
                rule: "algebraic-left-result",
            })?
            .definition()
        else {
            continue;
        };
        let left = program
            .operation(left_operation)
            .map_err(|_| NormalizeError::Structure {
                rule: "algebraic-left-operation",
            })?;
        if left.key() != operation || left.attributes() != root.attributes() {
            continue;
        }
        let left_operands: Vec<_> = left.operands().collect();
        let left_results: Vec<_> = left.results().collect();
        let ([first, second], [produced_left]) =
            (left_operands.as_slice(), left_results.as_slice())
        else {
            continue;
        };
        if produced_left != left_result {
            continue;
        }
        let ordered_leaves = [*first, *second, *right];
        if !reassociated_signature_is_valid(
            program,
            operation,
            root.attributes(),
            ordered_leaves,
            *original_result,
        )? {
            continue;
        }
        return Ok(Some(OrderedReassociationSite {
            root: root.id(),
            operation: operation.clone(),
            attributes: root.attributes().clone(),
            ordered_leaves,
            original_result: *original_result,
        }));
    }
    Ok(None)
}

fn value_fact(program: &SemanticProgram, value: ValueId) -> Result<ValueFact, NormalizeError> {
    let value = program
        .value(value)
        .map_err(|_| NormalizeError::Structure {
            rule: "algebraic-value",
        })?;
    Ok(ValueFact::new(
        value.resolved_type().clone(),
        value.shape().clone(),
    ))
}

fn reassociated_signature_is_valid(
    program: &SemanticProgram,
    operation: &OpKey,
    attributes: &OperationAttributes,
    [first, second, third]: [ValueId; 3],
    original_result: ValueId,
) -> Result<bool, NormalizeError> {
    let second = value_fact(program, second)?;
    let third = value_fact(program, third)?;
    let Ok(right_results) =
        program
            .semantic_registry()
            .infer_operation(operation, &[second, third], attributes)
    else {
        return Ok(false);
    };
    let [right] = right_results.as_slice() else {
        return Ok(false);
    };
    let first = value_fact(program, first)?;
    let Ok(outer_results) =
        program
            .semantic_registry()
            .infer_operation(operation, &[first, right.clone()], attributes)
    else {
        return Ok(false);
    };
    let [outer] = outer_results.as_slice() else {
        return Ok(false);
    };
    let original = value_fact(program, original_result)?;
    Ok(outer.resolved_type() == original.resolved_type() && outer.shape() == original.shape())
}

fn rebuild_ordered_reassociation(
    program: &SemanticProgram,
    site: &OrderedReassociationSite,
) -> Result<SemanticProgram, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut builder = SemanticProgramBuilder::try_new(program.semantic_registry().clone())
        .map_err(|_| NormalizeError::Rebuild {
            rule: "algebraic-builder-create",
        })?;
    let mut mapped = vec![None; program.value_count()];
    for input in program.inputs() {
        let value = program
            .value(input.value())
            .map_err(|_| NormalizeError::Structure {
                rule: "algebraic-input-value",
            })?;
        let rebuilt = builder
            .input_resolved(
                input.key().clone(),
                value.shape().clone(),
                value.resolved_type().clone(),
            )
            .map_err(|_| NormalizeError::Rebuild {
                rule: "algebraic-input",
            })?;
        mapped[ordinal(&ordinals, input.value())?] = Some(rebuilt);
    }
    let lookup = |value: ValueId, mapped: &[Option<ValueId>]| {
        mapped
            .get(ordinal(&ordinals, value)?)
            .copied()
            .flatten()
            .ok_or(NormalizeError::InvalidRewrite {
                rule: "algebraic-unmapped-value",
            })
    };
    for operation in program.operations() {
        if operation.id() == site.root {
            let [first, second, third] = site.ordered_leaves;
            let right = builder
                .apply(
                    site.operation.clone(),
                    site.attributes.clone(),
                    &[lookup(second, &mapped)?, lookup(third, &mapped)?],
                )
                .map_err(|_| NormalizeError::Rebuild {
                    rule: "algebraic-right-operation",
                })?;
            let [right] = right.as_slice() else {
                return Err(NormalizeError::InvalidRewrite {
                    rule: "algebraic-right-result-arity",
                });
            };
            let outer = builder
                .apply(
                    site.operation.clone(),
                    site.attributes.clone(),
                    &[lookup(first, &mapped)?, *right],
                )
                .map_err(|_| NormalizeError::Rebuild {
                    rule: "algebraic-outer-operation",
                })?;
            let [outer] = outer.as_slice() else {
                return Err(NormalizeError::InvalidRewrite {
                    rule: "algebraic-outer-result-arity",
                });
            };
            mapped[ordinal(&ordinals, site.original_result)?] = Some(*outer);
            continue;
        }
        let operands = operation
            .operands()
            .map(|operand| lookup(operand, &mapped))
            .collect::<Result<Vec<_>, _>>()?;
        let rebuilt = builder
            .apply(
                operation.key().clone(),
                operation.attributes().clone(),
                &operands,
            )
            .map_err(|_| NormalizeError::Rebuild {
                rule: "algebraic-original-operation",
            })?;
        let original_results: Vec<_> = operation.results().collect();
        if rebuilt.len() != original_results.len() {
            return Err(NormalizeError::InvalidRewrite {
                rule: "algebraic-result-arity",
            });
        }
        for (original, rebuilt) in original_results.into_iter().zip(rebuilt) {
            mapped[ordinal(&ordinals, original)?] = Some(rebuilt);
        }
    }
    for output in program.outputs() {
        builder
            .output_resolved(output.key().clone(), lookup(output.value(), &mapped)?)
            .map_err(|_| NormalizeError::Rebuild {
                rule: "algebraic-output",
            })?;
    }
    builder.build().map_err(|_| NormalizeError::Rebuild {
        rule: "algebraic-semantic-verification",
    })
}

/// Explores the governed algebraic portfolio without choosing or routing it.
///
/// Every enabled rule contributes at most one whole-program proposal, so the
/// alternative count is bounded by the registered rule count rather than by
/// program size. The unchanged baseline is always retained. All proposals pass
/// through [`run_rewrite_engine`], which remains the sole revalidation and
/// budget authority.
pub(crate) fn explore_algebraic_alternatives_owned(
    program: SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    configuration: AlgebraicRuleConfiguration,
) -> Result<AlgebraicExplorationOutcome, NormalizeError> {
    let mut registry = RuleRegistry::new();
    let mut assessments = Vec::new();
    for rule in [
        OrderedReassociationRule::add(),
        OrderedReassociationRule::multiply(),
    ] {
        if !configuration.enabled(rule.identity) {
            assessments.push(RewriteAssessment::declined(
                rule.identity,
                RewriteAssessmentClass::Configuration,
                "configuration.rule-disabled",
            ));
            continue;
        }
        let prepared = rule.evaluate(&program, &contract)?;
        assessments.extend_from_slice(&prepared.assessments);
        registry
            .register(Box::new(prepared))
            .map_err(|_| NormalizeError::Structure {
                rule: "algebraic-rule-registration",
            })?;
    }
    let (alternatives, budget_stop) = match run_rewrite_engine(&registry, &program, budgets)
        .map_err(|failure| match failure {
            EngineFailure::Provider(ProviderDefect::Failed { reason, .. })
            | EngineFailure::Revalidation { reason, .. } => {
                NormalizeError::InvalidRewrite { rule: reason }
            }
            EngineFailure::Provider(ProviderDefect::Misattributed { .. }) => {
                NormalizeError::InvalidRewrite {
                    rule: "algebraic-misattributed-proposal",
                }
            }
        })? {
        EngineRun::Adopted(alternatives) => (alternatives, None),
        EngineRun::BudgetStopped { limit, demand } => (Vec::new(), Some((limit, demand))),
    };
    Ok(AlgebraicExplorationOutcome {
        baseline: program,
        alternatives,
        assessments,
        budget_stop,
    })
}

#[cfg(test)]
fn explore_algebraic_alternatives(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    configuration: AlgebraicRuleConfiguration,
) -> Result<AlgebraicExplorationOutcome, NormalizeError> {
    explore_algebraic_alternatives_owned(program.clone(), budgets, contract, configuration)
}

fn detect_shared_values(program: &SemanticProgram) -> Result<Congruence, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut congruence = Congruence {
        representative: (0..program.value_count()).collect(),
        retained: vec![true; program.operation_count()],
        operation_results: Vec::with_capacity(program.operation_count()),
        merges: Vec::new(),
    };
    let mut canonical: HashMap<OperationSignature, usize> = HashMap::new();
    let facts: Vec<(ResolvedValueType, Shape)> = program
        .values()
        .map(|value| (value.resolved_type().clone(), value.shape().clone()))
        .collect();
    for (index, operation) in program.operations().enumerate() {
        let results = operation
            .results()
            .map(|result| ordinal(&ordinals, result))
            .collect::<Result<Vec<_>, _>>()?;
        congruence.operation_results.push(results.clone());
        let definition = program
            .semantic_registry()
            .operation_definition(operation.key())
            .ok_or(NormalizeError::Structure {
                rule: "operation-definition",
            })?;
        // Only a referentially transparent occurrence may be replaced by an
        // earlier one. An effect class this profile cannot prove transparent is
        // left untouched rather than approximated, which is the fail-closed
        // direction for an optimization.
        if !matches!(definition.effect(), OperationEffect::Pure) {
            continue;
        }
        let mut operands = Vec::with_capacity(operation.operands().len());
        for operand in operation.operands() {
            operands.push(congruence.representative[ordinal(&ordinals, operand)?]);
        }
        let mut result_facts = Vec::with_capacity(results.len());
        for result in &results {
            result_facts.push(
                facts
                    .get(*result)
                    .ok_or(NormalizeError::Structure {
                        rule: "result-value",
                    })?
                    .clone(),
            );
        }
        let signature = OperationSignature {
            key: operation.key().clone(),
            attributes: operation.attributes().clone(),
            operands,
            results: result_facts,
        };
        if let Some(existing) = canonical.get(&signature).copied() {
            congruence.retained[index] = false;
            congruence.merges.push(SharedValueMerge {
                canonical: existing,
                merged: index,
            });
            let canonical_results = &congruence.operation_results[existing];
            if canonical_results.len() != results.len() {
                return Err(NormalizeError::Structure {
                    rule: "congruent-result-arity",
                });
            }
            for (position, result) in results.iter().enumerate() {
                congruence.representative[*result] = canonical_results[position];
            }
        } else {
            canonical.insert(signature, index);
        }
    }
    Ok(congruence)
}

fn ordinal(ordinals: &HashMap<ValueId, usize>, value: ValueId) -> Result<usize, NormalizeError> {
    ordinals
        .get(&value)
        .copied()
        .ok_or(NormalizeError::Structure {
            rule: "value-ordinal",
        })
}

/// Builds the candidate normalized program through the checked semantic builder.
///
/// Every operation is re-applied through the frozen authority, so result types
/// and shapes are re-inferred rather than copied from the input program.
fn rebuild(
    program: &SemanticProgram,
    congruence: &Congruence,
) -> Result<SemanticProgram, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut builder = SemanticProgramBuilder::try_new(program.semantic_registry().clone())
        .map_err(|_| NormalizeError::Rebuild {
            rule: "builder-create",
        })?;
    let mut mapped: Vec<Option<ValueId>> = vec![None; program.value_count()];
    for input in program.inputs() {
        let position = ordinal(&ordinals, input.value())?;
        let value = program
            .value(input.value())
            .map_err(|_| NormalizeError::Structure {
                rule: "input-value",
            })?;
        let rebuilt = builder
            .input_resolved(
                input.key().clone(),
                value.shape().clone(),
                value.resolved_type().clone(),
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "input" })?;
        mapped[position] = Some(rebuilt);
    }
    for (index, operation) in program.operations().enumerate() {
        if !congruence.retained[index] {
            continue;
        }
        let mut operands = Vec::with_capacity(operation.operands().len());
        for operand in operation.operands() {
            operands.push(resolve(congruence, &mapped, ordinal(&ordinals, operand)?)?);
        }
        let results = builder
            .apply(
                operation.key().clone(),
                operation.attributes().clone(),
                &operands,
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "operation" })?;
        let expected = &congruence.operation_results[index];
        if results.len() != expected.len() {
            return Err(NormalizeError::InvalidRewrite {
                rule: "result-arity",
            });
        }
        for (position, result) in results.into_iter().enumerate() {
            mapped[expected[position]] = Some(result);
        }
    }
    for output in program.outputs() {
        let value = resolve(congruence, &mapped, ordinal(&ordinals, output.value())?)?;
        builder
            .output_resolved(output.key().clone(), value)
            .map_err(|_| NormalizeError::Rebuild { rule: "output" })?;
    }
    builder.build().map_err(|_| NormalizeError::Rebuild {
        rule: "semantic-verification",
    })
}

/// Why a rewrite-engine run produced nothing usable.
///
/// The two cases are kept apart because they say different things about who is
/// at fault. A provider defect is a registered rule violating its contract; a
/// revalidation failure is a candidate that did not survive the frozen
/// authority. Both abandon the run, and a caller that conflated them could not
/// tell a broken rule from a rule that produced something invalid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EngineFailure {
    /// A registered provider violated its contract.
    Provider(ProviderDefect),
    /// A proposed candidate did not survive structural revalidation.
    ///
    /// Carries the rule so the failure names which provider produced the
    /// candidate; without it a caller would know a rewrite was invalid and not
    /// which rule to exclude.
    Revalidation {
        /// The rule whose candidate failed.
        rule: RewriteRuleIdentity,
        /// The `NormalizeError`'s stable reason.
        reason: &'static str,
    },
}

/// What one rewrite-engine run produced.
///
/// A dedicated outcome rather than an `Option`, because an abandoned run has
/// something to say: [`NormalizationOutcome::budget_stop`] reports the limit and
/// the demand that exceeded it, and a bare `None` cannot carry the second. That
/// figure is what makes a budget-stop explain record actionable — "stopped" is
/// not a fact anyone can act on; "stopped at 2, wanted 3" is.
#[derive(Debug)]
pub(crate) enum EngineRun<Program> {
    /// Every proposal fit the budget and survived revalidation.
    Adopted(Vec<RewriteProposal<Program>>),
    /// The run was abandoned whole; no rewrite was applied.
    BudgetStopped {
        /// The declared budget.
        limit: u64,
        /// Rewrites demanded, which exceeded it.
        demand: u64,
    },
}

impl<Program> EngineRun<Program> {
    /// The adopted alternatives, panicking if the run was abandoned.
    ///
    /// For tests that have already established the budget admits the run.
    /// Production code matches on the variants, because a budget stop is an
    /// outcome to report rather than one to unwrap past.
    #[cfg(test)]
    pub(crate) fn expect(self, message: &str) -> Vec<RewriteProposal<Program>> {
        match self {
            Self::Adopted(alternatives) => alternatives,
            Self::BudgetStopped { limit, demand } => {
                panic!("{message} (budget {limit}, demand {demand})")
            }
        }
    }
}

/// One rewrite-engine run: every registered rule proposes, every candidate is
/// revalidated, and the survivors are returned as alternatives.
///
/// # The all-or-nothing contract, preserved
///
/// A budget stop or a revalidation failure abandons the **whole run** and
/// returns nothing, exactly as [`normalize_semantics`] abandons a rewrite rather
/// than committing part of one. Returning the alternatives collected so far
/// would be a partial result that reads like a complete one — the caller cannot
/// tell a run that found two alternatives from one that found five and lost
/// three.
///
/// # What the budget counts
///
/// Proposals, against `normalization_rewrites`. That is the same resource
/// [`normalize_semantics`] bounds, and counting proposals rather than accepted
/// alternatives is what keeps the bound meaningful: revalidation happens *after*
/// the count, so a rule cannot buy extra budget by proposing candidates that
/// fail.
///
/// # What this does not do
///
/// It does not adopt. Choosing among alternatives is the caller's, and this
/// returns every candidate that survived rather than a winner — an engine that
/// picked one would be making a cost decision with no cost model in scope.
pub(crate) fn run_rewrite_engine(
    registry: &RuleRegistry<SemanticProgram>,
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
) -> Result<EngineRun<SemanticProgram>, EngineFailure> {
    let proposals = collect_proposals(registry, program).map_err(EngineFailure::Provider)?;

    // Rewrites, not proposals: one proposal can represent many rewrites, and
    // counting proposals would let a rule commit past a budget meant to forbid
    // it. This is the same resource the pre-engine stage bounded and must stay
    // the same quantity, or routing through the engine would silently relax it.
    //
    // Bounded per alternative, NOT summed across them. The proposals are
    // mutually exclusive candidates derived from the same input — at most one
    // is ever adopted — so the rewrites actually committed are one proposal's,
    // never the total. Summing over-charged: a legal pair of alternatives that
    // each fit the budget would have been refused together the moment a second
    // rule registered. The reported demand is the largest offender's, which is
    // the smallest budget that would have admitted every alternative.
    let limit = u64::from(budgets.normalization_rewrites);
    if let Some(demand) = proposals
        .iter()
        .map(RewriteProposal::rewrites)
        .filter(|rewrites| *rewrites > limit)
        .max()
    {
        return Ok(EngineRun::BudgetStopped { limit, demand });
    }

    let mut alternatives = Vec::with_capacity(proposals.len());
    for proposal in proposals {
        let revalidated = revalidate_structurally(proposal.candidate()).map_err(|error| {
            EngineFailure::Revalidation {
                rule: proposal.rule(),
                reason: error.reason(),
            }
        })?;
        // The revalidated program replaces the provider's, not merely checks it.
        // Adopting the candidate the provider handed over while only *verifying*
        // a rebuild of it would keep whatever the provider actually constructed,
        // and the rebuild is the version the frozen authority produced.
        // The rewrite count and the rule's explain payload both carry over:
        // revalidation rebuilds the *program*, and changes neither how many
        // rewrites produced it nor what the rule wants recorded about them.
        //
        // Dropping the payload here is a real bug this had, caught by the
        // explain census the moment the compile path started using the engine —
        // the rewrite still happened and its records silently stopped being
        // emitted, which no test of the engine in isolation could see.
        let mut adopted = RewriteProposal::new(proposal.rule(), revalidated, proposal.rewrites());
        if let Some(records) = proposal.explain() {
            adopted = adopted.with_explain(records.clone());
        }
        alternatives.push(adopted);
    }
    Ok(EngineRun::Adopted(alternatives))
}

/// Groups readmitted alternatives by the numerical contract each resolved to.
///
/// # Why alternatives must not be compared across groups
///
/// Readmission re-resolves the numerical contract from the caller's stated
/// preference, so two alternatives of one program can resolve differently. A
/// cheaper alternative under a weaker contract is not a better alternative —
/// it is a different answer to a different question, and ranking the two
/// together would let a rewrite buy speed by quietly relaxing what the caller
/// asked for.
///
/// This is the same rule `PlanStructuralCost::dominates` already applies when it
/// returns `false` across differing cost-model keys: incomparable things must
/// not be ordered, and the way to enforce that is to keep them out of one
/// another's comparison rather than to remember not to compare them.
///
/// Groups are returned in first-appearance order over an input that is already
/// in canonical rule order, so the result is deterministic without imposing an
/// order on contract keys — which have none, being an open vocabulary.
///
/// A single group is the ordinary case and carries no special meaning; more
/// than one means the caller must choose *within* a group and then choose
/// between groups on the contract, not on cost.
pub(crate) fn group_by_resolved_contract<Item>(
    alternatives: Vec<Item>,
    contract_key: impl Fn(&Item) -> &'static str,
) -> Vec<(&'static str, Vec<Item>)> {
    let mut groups: Vec<(&'static str, Vec<Item>)> = Vec::new();
    for alternative in alternatives {
        let key = contract_key(&alternative);
        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, members)) => members.push(alternative),
            None => groups.push((key, vec![alternative])),
        }
    }
    groups
}

/// Rebuilds a program through the checked semantic builder, changing nothing.
///
/// The engine's half of revalidation. Every operation is re-applied through the
/// frozen authority, so result types and shapes are **re-inferred rather than
/// copied**, and a candidate whose structure does not survive that inference is
/// rejected here rather than adopted.
///
/// # Why this is not `rebuild` with an identity congruence
///
/// It could be, and it deliberately is not. `rebuild` reads `congruence.retained`
/// to drop operations and `congruence.operation_results` to remap values; an
/// identity congruence is a `Congruence` whose fields all say "change nothing",
/// which is a value only this call site would ever construct and which the
/// congruence's own invariants do not describe. Passing one would mean the
/// generic path's correctness depended on a CSE-shaped structure being filled
/// in a way CSE never fills it.
///
/// The duplication is one loop, and it is the honest cost of the two paths
/// answering different questions: `rebuild` asks what the rewrite produces, and
/// this asks whether an arbitrary program still validates.
///
/// # Where this belongs long term
///
/// It uses only `tiler-ir`'s builder and program, so that crate is its natural
/// home — but adding it there is a public API change on the semantic authority,
/// which ADR 0075 reserves. It stays private here until a second consumer
/// justifies the promotion.
pub(crate) fn revalidate_structurally(
    program: &SemanticProgram,
) -> Result<SemanticProgram, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut builder = SemanticProgramBuilder::try_new(program.semantic_registry().clone())
        .map_err(|_| NormalizeError::Rebuild {
            rule: "builder-create",
        })?;
    let mut mapped: Vec<Option<ValueId>> = vec![None; program.value_count()];

    let lookup = |mapped: &[Option<ValueId>], position: usize| -> Result<ValueId, NormalizeError> {
        mapped
            .get(position)
            .copied()
            .flatten()
            .ok_or(NormalizeError::InvalidRewrite {
                rule: "unmapped-value",
            })
    };

    for input in program.inputs() {
        let position = ordinal(&ordinals, input.value())?;
        let value = program
            .value(input.value())
            .map_err(|_| NormalizeError::Structure {
                rule: "input-value",
            })?;
        let rebuilt = builder
            .input_resolved(
                input.key().clone(),
                value.shape().clone(),
                value.resolved_type().clone(),
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "input" })?;
        mapped[position] = Some(rebuilt);
    }

    for operation in program.operations() {
        let mut operands = Vec::with_capacity(operation.operands().len());
        for operand in operation.operands() {
            operands.push(lookup(&mapped, ordinal(&ordinals, operand)?)?);
        }
        let results = builder
            .apply(
                operation.key().clone(),
                operation.attributes().clone(),
                &operands,
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "operation" })?;
        // Re-inference must produce the same number of results the input
        // carried. A mismatch means the frozen authority disagrees with the
        // program's own structure, which is a rejection rather than something
        // to reconcile.
        if results.len() != operation.results().len() {
            return Err(NormalizeError::InvalidRewrite {
                rule: "result-arity",
            });
        }
        for (result, original) in results.into_iter().zip(operation.results()) {
            mapped[ordinal(&ordinals, original)?] = Some(result);
        }
    }

    for output in program.outputs() {
        let value = lookup(&mapped, ordinal(&ordinals, output.value())?)?;
        builder
            .output_resolved(output.key().clone(), value)
            .map_err(|_| NormalizeError::Rebuild { rule: "output" })?;
    }

    builder.build().map_err(|_| NormalizeError::Rebuild {
        rule: "semantic-verification",
    })
}

fn resolve(
    congruence: &Congruence,
    mapped: &[Option<ValueId>],
    position: usize,
) -> Result<ValueId, NormalizeError> {
    let canonical = *congruence
        .representative
        .get(position)
        .ok_or(NormalizeError::Structure {
            rule: "representative",
        })?;
    mapped
        .get(canonical)
        .copied()
        .flatten()
        .ok_or(NormalizeError::InvalidRewrite {
            rule: "unmapped-value",
        })
}

/// Checks every postcondition before the candidate program may be adopted.
fn verify_normalized(
    original: &SemanticProgram,
    normalized: &SemanticProgram,
    congruence: &Congruence,
) -> Result<(), NormalizeError> {
    let expected_operations = original
        .operation_count()
        .checked_sub(congruence.merges.len())
        .ok_or(NormalizeError::InvalidRewrite {
            rule: "operation-count",
        })?;
    if normalized.operation_count() != expected_operations
        || normalized.input_count() != original.input_count()
        || normalized.output_count() != original.output_count()
    {
        return Err(NormalizeError::InvalidRewrite {
            rule: "operation-count",
        });
    }
    for (before, after) in original.inputs().zip(normalized.inputs()) {
        let before_value = original
            .value(before.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        let after_value = normalized
            .value(after.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        if before.key() != after.key()
            || before_value.shape() != after_value.shape()
            || before_value.resolved_type() != after_value.resolved_type()
        {
            return Err(NormalizeError::InvalidRewrite {
                rule: "input-interface",
            });
        }
    }
    for (before, after) in original.outputs().zip(normalized.outputs()) {
        let before_value = original
            .value(before.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        let after_value = normalized
            .value(after.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        if before.key() != after.key()
            || before_value.shape() != after_value.shape()
            || before_value.resolved_type() != after_value.resolved_type()
        {
            return Err(NormalizeError::InvalidRewrite {
                rule: "output-interface",
            });
        }
    }
    let before = original.semantic_identity();
    let after = normalized.semantic_identity();
    // Removing a redundant occurrence changes graph meaning but never the set of
    // reached semantic definitions, the providers that admitted them, or the
    // registry snapshot that validated them.
    if before.reached_definitions() != after.reached_definitions()
        || before.admission_provenance() != after.admission_provenance()
        || before.registry_snapshot() != after.registry_snapshot()
    {
        return Err(NormalizeError::InvalidRewrite {
            rule: "semantic-authority",
        });
    }
    if before.graph() == after.graph() {
        return Err(NormalizeError::InvalidRewrite {
            rule: "graph-identity",
        });
    }
    if !detect_shared_values(normalized)?.merges.is_empty() {
        return Err(NormalizeError::InvalidRewrite { rule: "fixpoint" });
    }
    Ok(())
}

fn digest(program: &SemanticProgram) -> u64 {
    program
        .semantic_identity()
        .graph()
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE, F32Add, F32Constant, F32Multiply,
        InputKey, OutputKey, SemanticProgramBuilder, StrictSerialF32Sum, Value,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_reference::{
        FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor,
        TensorPayloadView,
    };

    #[test]
    fn canonical_phase_refuses_a_provider_cardinality_defect() {
        assert_eq!(
            sole_canonical_alternative(&[1_u8, 2]),
            Err(NormalizeError::InvalidRewrite {
                rule: "canonical-provider-cardinality"
            })
        );
        assert_eq!(sole_canonical_alternative(&[1_u8]), Ok(&1));
    }

    /// A retained root record the stage chain hangs from.
    ///
    /// The real pipeline always has one — the request-verification receipt —
    /// so the stage recorders take a record rather than an option.
    fn test_root(explain: &mut ExplainWriter) -> ExplainRecordId {
        let subject = explain
            .subject(SubjectKind::SemanticProgram, "semantic-program")
            .unwrap();
        explain
            .push_detail(
                RuleRef::builtin("test.root").unwrap(),
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::RequestVerification,
                    assessment: PredicateAssessment::proven(
                        "test.root",
                        EvidenceBasis::CheckedInvariant,
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                Vec::new(),
            )
            .unwrap()
    }

    /// Builds the governed serial-sum program.
    ///
    /// `share_constants` selects whether the scale and bias constants are one
    /// authored value or two identical redundant occurrences.
    fn program(scale: f32, bias: f32, share_constants: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale_value = F32Constant::apply(&mut builder, scale.to_bits()).unwrap();
        let bias_value = if share_constants {
            scale_value
        } else {
            F32Constant::apply(&mut builder, bias.to_bits()).unwrap()
        };
        let product = F32Multiply::apply(&mut builder, input, scale_value).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias_value).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn normalize(program: &SemanticProgram) -> NormalizationOutcome {
        normalize_semantics(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap()
    }

    fn evaluate(program: &SemanticProgram, values: &[f32]) -> Vec<u32> {
        let key = InputKey::new("input").unwrap();
        let tensor = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([2, 3]),
            values
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
        let outputs = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(program, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        match outputs[0].payload() {
            TensorPayloadView::Dense(elements) => elements
                .iter()
                .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
                .collect(),
            _ => panic!("expected a dense f32 reference output"),
        }
    }

    fn algebraic_chain(operation: &OpKey, leaves: &[f32]) -> SemanticProgram {
        assert!((3..=6).contains(&leaves.len()));
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let values: Vec<_> = leaves
            .iter()
            .map(|value| F32Constant::apply(&mut builder, value.to_bits()).unwrap())
            .collect();
        let mut values = values.into_iter();
        let first = values.next().unwrap();
        let second = values.next().unwrap();
        let mut accumulated = apply_binary(operation, &mut builder, first, second);
        for value in values {
            accumulated = apply_binary(operation, &mut builder, accumulated, value);
        }
        builder
            .output(OutputKey::new("result").unwrap(), accumulated)
            .unwrap();
        builder.build().unwrap()
    }

    fn portfolio_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let [
            add_first,
            add_second,
            add_third,
            multiply_first,
            multiply_second,
            multiply_third,
        ] = [
            1.0e20_f32,
            -1.0e20,
            1.0,
            2.0_f32.powi(100),
            2.0_f32.powi(50),
            2.0_f32.powi(-50),
        ]
        .map(|value| F32Constant::apply(&mut builder, value.to_bits()).unwrap());
        let add_left = F32Add::apply(&mut builder, add_first, add_second).unwrap();
        let add = F32Add::apply(&mut builder, add_left, add_third).unwrap();
        let multiply_left =
            F32Multiply::apply(&mut builder, multiply_first, multiply_second).unwrap();
        let multiply = F32Multiply::apply(&mut builder, multiply_left, multiply_third).unwrap();
        builder.output(OutputKey::new("add").unwrap(), add).unwrap();
        builder
            .output(OutputKey::new("multiply").unwrap(), multiply)
            .unwrap();
        builder.build().unwrap()
    }

    fn apply_binary(
        operation: &OpKey,
        builder: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Value<F32> {
        if operation == &add_f32_op() {
            F32Add::apply(builder, left, right).unwrap()
        } else if operation == &multiply_f32_op() {
            F32Multiply::apply(builder, left, right).unwrap()
        } else {
            panic!("fixture asked for an unsupported binary operation")
        }
    }

    fn output_value(program: &SemanticProgram, key: &str) -> ValueId {
        program
            .outputs()
            .find(|output| output.key().as_str() == key)
            .expect("named output")
            .value()
    }

    fn ordered_constant_leaves(
        program: &SemanticProgram,
        operation: &OpKey,
        value: ValueId,
    ) -> Vec<u32> {
        let Definition::OperationResult {
            operation: producer,
            ..
        } = program.value(value).unwrap().definition()
        else {
            panic!("an algebraic output must be operation-produced")
        };
        let producer = program.operation(producer).unwrap();
        if producer.key() == operation {
            return producer
                .operands()
                .flat_map(|operand| ordered_constant_leaves(program, operation, operand))
                .collect();
        }
        assert_eq!(producer.key(), &tiler_ir::semantic::constant_f32_op());
        let Some(CanonicalValueView::FloatBits(bits)) = producer
            .attributes()
            .get(F32_CONSTANT_BITS_ATTRIBUTE)
            .map(tiler_ir::semantic::CanonicalValue::view)
        else {
            panic!("fixture leaf must be an f32 constant")
        };
        vec![u32::from_be_bytes(
            <[u8; 4]>::try_from(bits.bits()).unwrap(),
        )]
    }

    #[derive(Clone, Debug)]
    enum Grouping {
        Leaf(usize),
        Pair(Box<Self>, Box<Self>),
    }

    fn all_ordered_groupings(start: usize, end: usize) -> Vec<Grouping> {
        if end - start == 1 {
            return vec![Grouping::Leaf(start)];
        }
        let mut groupings = Vec::new();
        for split in (start + 1)..end {
            for left in all_ordered_groupings(start, split) {
                for right in all_ordered_groupings(split, end) {
                    groupings.push(Grouping::Pair(Box::new(left.clone()), Box::new(right)));
                }
            }
        }
        groupings
    }

    fn build_grouping(
        grouping: &Grouping,
        operation: &OpKey,
        builder: &mut SemanticProgramBuilder,
        leaves: &[Value<F32>],
    ) -> Value<F32> {
        match grouping {
            Grouping::Leaf(index) => leaves[*index],
            Grouping::Pair(left, right) => {
                let left = build_grouping(left, operation, builder, leaves);
                let right = build_grouping(right, operation, builder, leaves);
                apply_binary(operation, builder, left, right)
            }
        }
    }

    fn exact_ordered_result_set(operation: &OpKey, leaves: &[u32]) -> Vec<u32> {
        assert!(
            (3..=6).contains(&leaves.len()),
            "the exhaustive result-set oracle is deliberately bounded"
        );
        let evaluator = ReferenceEvaluator::standard().unwrap();
        let mut results = Vec::new();
        for grouping in all_ordered_groupings(0, leaves.len()) {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let values: Vec<_> = leaves
                .iter()
                .map(|bits| F32Constant::apply(&mut builder, *bits).unwrap())
                .collect();
            let result = build_grouping(&grouping, operation, &mut builder, &values);
            builder
                .output(OutputKey::new("result").unwrap(), result)
                .unwrap();
            let program = builder.build().unwrap();
            let evaluated = evaluator.evaluate(&program, &[]).unwrap();
            let TensorPayloadView::Dense(elements) = evaluated[0].payload() else {
                panic!("f32 output is dense")
            };
            results.push(u32::from_be_bytes(
                <[u8; 4]>::try_from(elements[0].as_bytes()).unwrap(),
            ));
        }
        results.sort_unstable();
        results.dedup();
        results
    }

    fn evaluate_scalar_output(program: &SemanticProgram, key: &str) -> u32 {
        let evaluator = ReferenceEvaluator::standard().unwrap();
        let outputs = evaluator.evaluate(program, &[]).unwrap();
        let index = program
            .outputs()
            .position(|output| output.key().as_str() == key)
            .unwrap();
        let TensorPayloadView::Dense(elements) = outputs[index].payload() else {
            panic!("f32 output is dense")
        };
        u32::from_be_bytes(<[u8; 4]>::try_from(elements[0].as_bytes()).unwrap())
    }

    #[test]
    fn algebraic_rules_are_separate_baseline_preserving_alternatives() {
        let program = portfolio_program();
        let outcome = explore_algebraic_alternatives(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed_relaxed(),
            AlgebraicRuleConfiguration::all(),
        )
        .unwrap();

        assert_eq!(
            outcome.baseline().semantic_identity(),
            program.semantic_identity(),
            "exploration changed or dropped the unchanged baseline"
        );
        assert_eq!(outcome.alternatives().len(), 2);
        assert_eq!(
            outcome
                .alternatives()
                .iter()
                .map(RewriteProposal::rule)
                .collect::<Vec<_>>(),
            [
                ORDERED_REASSOCIATE_ADD_RULE.unwrap(),
                ORDERED_REASSOCIATE_MULTIPLY_RULE.unwrap(),
            ],
            "the two algebraic alternatives lost separate canonical identities"
        );
        assert!(
            outcome
                .alternatives()
                .iter()
                .all(|alternative| alternative.candidate().operation_count()
                    == program.operation_count()),
            "reassociation retained its replaced left child as dead work"
        );
        assert!(
            outcome
                .alternatives()
                .iter()
                .all(|alternative| alternative.explain().is_some()),
            "an accepted algebraic alternative lost its composite explain payload"
        );
    }

    #[test]
    fn strict_and_flush_contracts_decline_numerically_after_semantic_acceptance() {
        let program = algebraic_chain(&add_f32_op(), &[1.0e20, -1.0e20, 1.0]);
        for contract in [
            StrictF32NumericalContract::governed(),
            StrictF32NumericalContract::governed_flush_to_zero(),
        ] {
            let outcome = explore_algebraic_alternatives(
                &program,
                DeterministicBudgets::governed(),
                contract,
                AlgebraicRuleConfiguration::all(),
            )
            .unwrap();
            assert!(outcome.alternatives().is_empty());
            let add: Vec<_> = outcome
                .assessments()
                .iter()
                .copied()
                .filter(|assessment| assessment.rule() == ORDERED_REASSOCIATE_ADD_RULE.unwrap())
                .collect();
            assert_eq!(add.len(), 2);
            assert_eq!(add[0].class(), RewriteAssessmentClass::Semantic);
            assert!(add[0].is_accepted());
            assert_eq!(add[1].class(), RewriteAssessmentClass::Numerical);
            assert!(!add[1].is_accepted());
            assert_eq!(add[1].reason(), "numerical.reassociation-forbidden");
        }
    }

    #[test]
    fn a_nonmatching_program_declines_semantically_without_a_numerical_claim() {
        let program = algebraic_chain(&add_f32_op(), &[1.0, 2.0, 3.0]);
        let outcome = explore_algebraic_alternatives(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed_relaxed(),
            AlgebraicRuleConfiguration::all().with(ORDERED_REASSOCIATE_ADD_RULE.unwrap(), false),
        )
        .unwrap();
        let multiply: Vec<_> = outcome
            .assessments()
            .iter()
            .copied()
            .filter(|assessment| assessment.rule() == ORDERED_REASSOCIATE_MULTIPLY_RULE.unwrap())
            .collect();
        assert_eq!(multiply.len(), 1);
        assert_eq!(multiply[0].class(), RewriteAssessmentClass::Semantic);
        assert_eq!(multiply[0].reason(), "semantic.no-left-associated-chain");
        assert!(!multiply[0].is_accepted());
    }

    #[test]
    fn a_prepared_provider_rebuilds_only_from_the_program_it_is_given() {
        let evaluated = algebraic_chain(&add_f32_op(), &[1.0e20, -1.0e20, 1.0]);
        let provider = OrderedReassociationRule::add()
            .evaluate(&evaluated, &StrictF32NumericalContract::governed_relaxed())
            .unwrap();

        let nonmatching = algebraic_chain(&multiply_f32_op(), &[2.0, 3.0, 4.0]);
        assert!(
            provider.propose(&nonmatching).unwrap().is_empty(),
            "the provider returned the candidate cached from its evaluation program"
        );

        let supplied = algebraic_chain(&add_f32_op(), &[4.0, 5.0, 6.0]);
        let proposals = provider.propose(&supplied).unwrap();
        assert_eq!(proposals.len(), 1);
        assert_eq!(
            ordered_constant_leaves(
                proposals[0].candidate(),
                &add_f32_op(),
                output_value(proposals[0].candidate(), "result")
            ),
            [4.0_f32.to_bits(), 5.0_f32.to_bits(), 6.0_f32.to_bits()],
            "the candidate did not derive from the program supplied to propose"
        );
    }

    #[test]
    fn disabling_add_leaves_multiply_enabled_and_explained() {
        let program = portfolio_program();
        let configuration =
            AlgebraicRuleConfiguration::all().with(ORDERED_REASSOCIATE_ADD_RULE.unwrap(), false);
        let outcome = explore_algebraic_alternatives(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed_relaxed(),
            configuration,
        )
        .unwrap();
        assert_eq!(outcome.alternatives().len(), 1);
        assert_eq!(
            outcome.alternatives()[0].rule(),
            ORDERED_REASSOCIATE_MULTIPLY_RULE.unwrap()
        );
        assert!(outcome.assessments().iter().any(|assessment| {
            assessment.rule() == ORDERED_REASSOCIATE_ADD_RULE.unwrap()
                && assessment.class() == RewriteAssessmentClass::Configuration
                && assessment.reason() == "configuration.rule-disabled"
                && !assessment.is_accepted()
        }));
    }

    #[test]
    fn algebraic_search_uses_the_existing_observable_budget_stop() {
        let program = portfolio_program();
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;
        let outcome = explore_algebraic_alternatives(
            &program,
            budgets,
            StrictF32NumericalContract::governed_relaxed(),
            AlgebraicRuleConfiguration::all(),
        )
        .unwrap();
        assert_eq!(outcome.budget_stop(), Some((0, 1)));
        assert!(outcome.alternatives().is_empty());
        assert_eq!(
            outcome.baseline().semantic_identity(),
            program.semantic_identity()
        );
    }

    #[test]
    fn a_shared_left_child_is_retained_for_its_other_observer() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let first = F32Constant::apply(&mut builder, 1.0e20_f32.to_bits()).unwrap();
        let second = F32Constant::apply(&mut builder, (-1.0e20_f32).to_bits()).unwrap();
        let third = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let left = F32Add::apply(&mut builder, first, second).unwrap();
        let root = F32Add::apply(&mut builder, left, third).unwrap();
        builder
            .output(OutputKey::new("left").unwrap(), left)
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        let program = builder.build().unwrap();

        let outcome = explore_algebraic_alternatives(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed_relaxed(),
            AlgebraicRuleConfiguration::all(),
        )
        .unwrap();
        let rewritten = outcome
            .alternatives()
            .iter()
            .find(|alternative| alternative.rule() == ORDERED_REASSOCIATE_ADD_RULE.unwrap())
            .unwrap()
            .candidate();

        assert_eq!(
            rewritten.operation_count(),
            program.operation_count() + 1,
            "the observed left child was dropped instead of retained"
        );
        assert_eq!(
            evaluate_scalar_output(rewritten, "left"),
            evaluate_scalar_output(&program, "left")
        );
    }

    #[test]
    fn reassociated_outputs_are_exact_members_of_the_ordered_result_set() {
        for (operation, leaves) in [
            (add_f32_op(), vec![1.0e20_f32, -1.0e20, 1.0]),
            (
                multiply_f32_op(),
                vec![2.0_f32.powi(100), 2.0_f32.powi(50), 2.0_f32.powi(-50)],
            ),
        ] {
            let original = algebraic_chain(&operation, &leaves);
            let original_order =
                ordered_constant_leaves(&original, &operation, output_value(&original, "result"));
            let outcome = explore_algebraic_alternatives(
                &original,
                DeterministicBudgets::governed(),
                StrictF32NumericalContract::governed_relaxed(),
                AlgebraicRuleConfiguration::all(),
            )
            .unwrap();
            let rule = if operation == add_f32_op() {
                ORDERED_REASSOCIATE_ADD_RULE.unwrap()
            } else {
                ORDERED_REASSOCIATE_MULTIPLY_RULE.unwrap()
            };
            let rewritten = outcome
                .alternatives()
                .iter()
                .find(|alternative| alternative.rule() == rule)
                .unwrap()
                .candidate();
            let rewritten_order =
                ordered_constant_leaves(rewritten, &operation, output_value(rewritten, "result"));
            assert_eq!(
                rewritten_order, original_order,
                "reassociation changed the ordered leaf sequence"
            );
            let result_set = exact_ordered_result_set(&operation, &original_order);
            let actual = evaluate_scalar_output(rewritten, "result");
            assert!(
                result_set.contains(&actual),
                "rewritten bits {actual:#010x} are not in the exact ordered result set {result_set:#010x?}"
            );
        }
    }

    #[test]
    #[should_panic(expected = "the exhaustive result-set oracle is deliberately bounded")]
    fn exact_result_set_oracle_refuses_an_unreviewed_leaf_count() {
        exact_ordered_result_set(&add_f32_op(), &[0; 7]);
    }

    #[test]
    fn already_canonical_programs_are_left_untouched() {
        let distinct = program(2.0, 1.0, false);
        let outcome = normalize(&distinct);

        assert!(outcome.normalized_program().is_none());
        assert_eq!(outcome.rewrite_count(), 0);
        assert_eq!(outcome.budget_stop(), None);
        assert_eq!(outcome.operations_before, outcome.operations_after);
    }

    #[test]
    fn identical_pure_invocations_normalize_to_one_semantic_value() {
        let duplicated = program(2.0, 2.0, false);
        assert_eq!(duplicated.operation_count(), 5);
        let outcome = normalize(&duplicated);
        let normalized = outcome
            .normalized_program()
            .expect("a redundant constant is rewritten");

        // The merge *contents* are now the rule's, not the stage's: the outcome
        // carries opaque payloads so an engine driving arbitrary rules can
        // produce one. Asserted at the level that still owns the vocabulary.
        assert_eq!(
            detect_shared_values(&duplicated)
                .expect("detection succeeds")
                .merges,
            [SharedValueMerge {
                canonical: 0,
                merged: 1,
            }]
        );
        assert_eq!(outcome.rewrite_count(), 1);
        assert_eq!(normalized.operation_count(), 4);
        assert_eq!(normalized.value_count(), duplicated.value_count() - 1);
        // The surviving constant feeds both pointwise operations.
        let constant = normalized
            .operations()
            .next()
            .expect("the canonical constant is first in topological order");
        let constant_result = constant.results().next().unwrap();
        assert_eq!(
            normalized
                .operations()
                .filter(|operation| operation
                    .operands()
                    .any(|operand| operand == constant_result))
                .count(),
            2
        );
    }

    #[test]
    fn normalization_converges_on_one_canonical_graph_identity() {
        let duplicated = program(2.0, 2.0, false);
        let shared = program(2.0, 2.0, true);
        assert_ne!(
            duplicated.semantic_identity().graph(),
            shared.semantic_identity().graph(),
            "the fixture must start from genuinely different graphs"
        );

        let normalized = normalize(&duplicated);
        let normalized = normalized
            .normalized_program()
            .expect("the duplicated program is rewritten");
        let already_canonical = normalize(&shared);

        assert!(already_canonical.normalized_program().is_none());
        assert_eq!(
            normalized.semantic_identity().graph(),
            shared.semantic_identity().graph()
        );
        assert_eq!(
            normalized.semantic_identity().reached_definitions(),
            shared.semantic_identity().reached_definitions()
        );
    }

    #[test]
    fn normalization_is_idempotent_and_deterministic() {
        let outcome = normalize(&program(2.0, 2.0, false));
        let normalized = outcome.normalized_program().unwrap().clone();

        // Renormalizing the result reaches the declared fixpoint.
        let again = normalize(&normalized);
        assert!(again.normalized_program().is_none());
        assert_eq!(again.canonical_graph_digest, digest(&normalized));

        // Two independent runs over equal inputs agree on identity and merges.
        let repeated = normalize(&program(2.0, 2.0, false));
        assert_eq!(
            repeated.normalized_program().unwrap().semantic_identity(),
            normalized.semantic_identity()
        );
        assert_eq!(repeated.rewrite_count(), outcome.rewrite_count());
        assert_eq!(
            repeated.canonical_graph_digest,
            outcome.canonical_graph_digest
        );
    }

    #[test]
    fn normalized_program_matches_the_reference_evaluator_bitwise() {
        let values = [1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, f32::NAN];
        for (scale, bias) in [
            (2.0_f32, 2.0_f32),
            (0.0, 0.0),
            (f32::NAN, f32::NAN),
            (f32::INFINITY, f32::INFINITY),
            (-0.0, -0.0),
        ] {
            let original = program(scale, bias, false);
            let outcome = normalize(&original);
            let normalized = outcome
                .normalized_program()
                .expect("identical constants are always merged");

            assert_eq!(
                evaluate(&original, &values),
                evaluate(normalized, &values),
                "normalization must preserve exact reference semantics"
            );
        }
    }

    #[test]
    fn negative_zero_and_nan_constants_are_distinguished_by_canonical_bits() {
        // `-0.0 == 0.0` and `NaN != NaN` under float comparison, so a rule that
        // compared decoded floats instead of canonical attribute bytes would
        // merge or split these incorrectly.
        let signed_zero = program(0.0, -0.0, false);
        assert!(normalize(&signed_zero).normalized_program().is_none());

        let nan = program(f32::NAN, f32::NAN, false);
        let outcome = normalize(&nan);
        assert_eq!(outcome.rewrite_count(), 1);
        assert_eq!(
            evaluate(&nan, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            evaluate(
                outcome.normalized_program().unwrap(),
                &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
            )
        );
    }

    /// **The pin for `generalize-the-normalize-transaction-to-alternatives`.**
    ///
    /// The provider must produce exactly what this stage produces, compared on
    /// the canonical bytes of the program's `SemanticIdentity` rather than on a
    /// summary such as the merge count — two different programs can share a
    /// merge count, and a pin that could not tell them apart would not be a
    /// pin. The bytes rather than a digest of them, so a collision cannot make
    /// two different programs compare equal.
    ///
    /// When the engine drives this rule and nothing else, its result must equal
    /// this. If that ever diverges, the engine changed the rewrite rather than
    /// merely rehosting it.
    #[test]
    fn the_provider_proposes_exactly_what_this_stage_produces() {
        let duplicated = program(2.0, 2.0, false);
        let expected = normalize(&duplicated);
        let expected_program = expected
            .normalized_program()
            .expect("the duplicated program normalizes");

        let proposals = CommonSubexpressionRule
            .propose(&duplicated)
            .expect("detection and rebuild succeed");
        assert_eq!(
            proposals.len(),
            1,
            "the rule proposed {} candidates",
            proposals.len()
        );
        assert_eq!(
            proposals[0].rule(),
            CommonSubexpressionRule.identity(),
            "the proposal is not attributed to the rule that made it"
        );
        assert_eq!(
            proposals[0]
                .candidate()
                .semantic_identity()
                .graph()
                .as_bytes(),
            expected_program.semantic_identity().graph().as_bytes(),
            "the provider's candidate differs from this stage's normalized program"
        );
    }

    /// A program with nothing to merge yields no proposal, not an empty rewrite.
    ///
    /// Without this the pin above would pass against a rule that proposed a
    /// candidate unconditionally, since a program with no merges rebuilds to a
    /// copy of itself and would compare equal.
    #[test]
    fn a_program_with_no_shared_values_proposes_nothing() {
        let distinct = program(2.0, 3.0, false);
        assert!(
            normalize(&distinct).normalized_program().is_none(),
            "the fixture has a merge, so this test proves nothing"
        );
        assert!(
            CommonSubexpressionRule
                .propose(&distinct)
                .expect("detection succeeds")
                .is_empty(),
            "a rule with nothing to do proposed a candidate"
        );
    }

    /// Structural revalidation preserves a program's identity exactly.
    ///
    /// This is the property the engine relies on: round-tripping a candidate
    /// must not change it, or every alternative would differ from itself and
    /// the pin above could never hold. Compared on canonical identity bytes,
    /// for the same reason the pin is.
    #[test]
    fn structural_revalidation_preserves_the_program() {
        for original in [program(2.0, 2.0, false), program(2.0, 3.0, false)] {
            let round_tripped =
                revalidate_structurally(&original).expect("a valid program revalidates");
            assert_eq!(
                round_tripped.semantic_identity().graph().as_bytes(),
                original.semantic_identity().graph().as_bytes(),
                "revalidation changed the program"
            );
        }
    }

    /// Revalidation is a real rebuild, not a clone.
    ///
    /// A `revalidate_structurally` that returned its input would pass the test
    /// above and would revalidate nothing. This drives it against a program the
    /// *rewrite* produced — one built by a different path than the fixture — and
    /// requires the round trip to reach the same identity, which a clone would
    /// also do, so the discriminating part is that the operation and value
    /// counts survive re-inference rather than being copied.
    #[test]
    fn revalidation_re_infers_rather_than_copying() {
        let duplicated = program(2.0, 2.0, false);
        let normalized = normalize(&duplicated);
        let rewritten = normalized
            .normalized_program()
            .expect("the duplicated program normalizes");

        let round_tripped =
            revalidate_structurally(rewritten).expect("the rewritten program revalidates");
        assert_eq!(
            round_tripped.operation_count(),
            rewritten.operation_count(),
            "re-inference changed the operation count"
        );
        assert_eq!(
            round_tripped.value_count(),
            rewritten.value_count(),
            "re-inference changed the value count"
        );
        assert_eq!(
            round_tripped.semantic_identity().graph().as_bytes(),
            rewritten.semantic_identity().graph().as_bytes(),
        );
        // The rewritten program is genuinely smaller than its input, so this
        // test is not silently running on the unrewritten fixture.
        assert!(rewritten.operation_count() < duplicated.operation_count());
    }

    fn cse_registry() -> RuleRegistry<SemanticProgram> {
        let mut registry = RuleRegistry::new();
        registry
            .register(Box::new(CommonSubexpressionRule))
            .expect("one rule");
        registry
    }

    /// The engine adopts exactly one correctly-attributed CSE alternative.
    ///
    /// A previous version of this test also compared the alternative's identity
    /// bytes against `normalize_semantics`'s output. That comparison became a
    /// tautology when the routing made the stage *be* the engine — both sides
    /// were the same `run_rewrite_engine(&{CSE}, program, governed())` call and
    /// the assertion could no longer fail. The identity property is pinned by
    /// `the_provider_proposes_exactly_what_this_stage_produces`, which compares
    /// the stage against the *provider* and stays a real comparison, backed by
    /// the two structural-revalidation tests. What survives here are the
    /// assertions that were never tautological: count, attribution, and the
    /// budget admitting the run.
    #[test]
    fn the_engine_with_only_cse_adopts_one_attributed_alternative() {
        let duplicated = program(2.0, 2.0, false);
        let alternatives = run_rewrite_engine(
            &cse_registry(),
            &duplicated,
            DeterministicBudgets::governed(),
        )
        .expect("no failure")
        .expect("the budget admits one proposal");

        assert_eq!(alternatives.len(), 1);
        assert_eq!(alternatives[0].rule(), CommonSubexpressionRule.identity());
        assert!(
            alternatives[0].explain().is_some(),
            "the adopted alternative lost its rule explain payload"
        );
    }

    /// An exhausted budget abandons the whole run rather than truncating it.
    ///
    /// `Ok(None)` and `Ok(Some(vec![]))` are different answers: the first says
    /// the run was abandoned, the second that nothing applied. A caller that
    /// received an empty vector for a budget stop would record "no rewrite
    /// available" for a program that had one.
    #[test]
    fn an_exhausted_budget_abandons_the_engine_run() {
        let duplicated = program(2.0, 2.0, false);
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;

        assert!(
            matches!(
                run_rewrite_engine(&cse_registry(), &duplicated, budgets).expect("no failure"),
                EngineRun::BudgetStopped {
                    limit: 0,
                    demand: 1
                }
            ),
            "an exhausted budget returned alternatives, or lost the demand figure"
        );
    }

    /// A program with nothing to rewrite yields an empty alternative set, not an
    /// abandoned run.
    ///
    /// The other half of the distinction above. Without this, an engine that
    /// always returned `None` would pass the budget test.
    #[test]
    fn a_program_with_no_rewrite_yields_no_alternatives() {
        let distinct = program(2.0, 3.0, false);
        let EngineRun::Adopted(alternatives) =
            run_rewrite_engine(&cse_registry(), &distinct, DeterministicBudgets::governed())
                .expect("no failure")
        else {
            panic!("the run was abandoned");
        };
        assert!(
            alternatives.is_empty(),
            "a program with nothing to rewrite produced an alternative"
        );
    }

    /// A provider defect abandons the run and is distinguishable from a
    /// revalidation failure.
    ///
    /// `EngineFailure` exists to keep those apart — a broken rule and a rule
    /// that produced something invalid call for different responses — so the
    /// distinction is asserted rather than assumed from the type having two
    /// variants.
    #[test]
    fn a_provider_defect_abandons_the_engine_run() {
        struct Broken;
        impl RewriteRuleProvider<SemanticProgram> for Broken {
            fn identity(&self) -> RewriteRuleIdentity {
                RewriteRuleIdentity::new("test", "broken", 1).expect("named")
            }
            fn propose(
                &self,
                _program: &SemanticProgram,
            ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
                Err(ProviderDefect::Failed {
                    rule: self.identity(),
                    reason: "builder-create",
                })
            }
        }

        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Broken)).expect("one rule");

        let failure = run_rewrite_engine(
            &registry,
            &program(2.0, 2.0, false),
            DeterministicBudgets::governed(),
        )
        .expect_err("a broken provider fails the run");

        assert!(
            matches!(
                failure,
                EngineFailure::Provider(ProviderDefect::Failed { .. })
            ),
            "a provider defect was not reported as one: {failure:?}"
        );
        assert!(
            !matches!(failure, EngineFailure::Revalidation { .. }),
            "a provider defect was reported as a revalidation failure"
        );
    }

    /// A misattributing provider is also a provider defect, not a silent drop.
    ///
    /// The engine inherits `collect_proposals`' attribution contract; this
    /// confirms the inheritance rather than assuming the `?` carries it.
    #[test]
    fn a_misattributing_provider_fails_the_engine_run() {
        struct Liar;
        impl RewriteRuleProvider<SemanticProgram> for Liar {
            fn identity(&self) -> RewriteRuleIdentity {
                RewriteRuleIdentity::new("test", "liar", 1).expect("named")
            }
            fn propose(
                &self,
                program: &SemanticProgram,
            ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
                Ok(vec![RewriteProposal::new(
                    CommonSubexpressionRule.identity(),
                    program.clone(),
                    1,
                )])
            }
        }

        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Liar)).expect("one rule");

        let failure = run_rewrite_engine(
            &registry,
            &program(2.0, 2.0, false),
            DeterministicBudgets::governed(),
        )
        .expect_err("a misattributing provider fails the run");

        assert!(
            matches!(
                failure,
                EngineFailure::Provider(ProviderDefect::Misattributed { .. })
            ),
            "a misattributed proposal reached the engine: {failure:?}"
        );
    }

    /// Alternatives resolving to one contract stay in one group.
    #[test]
    fn one_resolved_contract_yields_one_group() {
        let groups =
            group_by_resolved_contract(vec![("a", "strict"), ("b", "strict")], |item| item.1);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "strict");
        assert_eq!(groups[0].1.len(), 2);
    }

    /// Alternatives resolving to different contracts are kept apart.
    ///
    /// Without the grouping these would be ranked together, and a cheaper
    /// alternative under a weaker contract would win — a rewrite buying speed
    /// by relaxing what the caller asked for.
    #[test]
    fn diverging_contracts_are_not_placed_in_one_group() {
        let groups = group_by_resolved_contract(
            vec![("a", "strict"), ("b", "flush"), ("c", "strict")],
            |item| item.1,
        );
        assert_eq!(groups.len(), 2, "diverging contracts were merged");
        assert_eq!(groups[0].0, "strict");
        assert_eq!(
            groups[0].1.len(),
            2,
            "same-contract alternatives were split"
        );
        assert_eq!(groups[1].0, "flush");
        assert_eq!(groups[1].1.len(), 1);
    }

    /// Grouping preserves input order within and across groups.
    ///
    /// The input arrives in canonical rule order, so first-appearance grouping
    /// is deterministic without imposing an order on contract keys — which have
    /// none, being an open vocabulary. A grouping that sorted by key would be
    /// inventing one.
    #[test]
    fn grouping_is_deterministic_without_ordering_contract_keys() {
        let forward =
            group_by_resolved_contract(vec![("a", "z"), ("b", "y"), ("c", "z")], |item| item.1);
        assert_eq!(
            forward.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
            ["z", "y"],
            "grouping imposed an order on the contract keys"
        );
        assert_eq!(
            forward[0].1.iter().map(|item| item.0).collect::<Vec<_>>(),
            ["a", "c"],
            "members lost their input order"
        );
    }

    /// The budget counts rewrites, not proposals.
    ///
    /// This is the regression that motivated giving a proposal a rewrite count.
    /// One rule returns one proposal representing three rewrites; a budget of
    /// two must stop it. An engine counting proposals sees one against two and
    /// proceeds — committing three rewrites past a budget that forbade them,
    /// with nothing reporting the difference.
    ///
    /// The permitted case is asserted too, so a budget that refused everything
    /// would fail here rather than pass.
    #[test]
    fn the_budget_counts_rewrites_rather_than_proposals() {
        struct Bulk;
        impl RewriteRuleProvider<SemanticProgram> for Bulk {
            fn identity(&self) -> RewriteRuleIdentity {
                RewriteRuleIdentity::new("test", "bulk", 1).expect("named")
            }
            fn propose(
                &self,
                program: &SemanticProgram,
            ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
                // One proposal, three rewrites.
                Ok(vec![RewriteProposal::new(
                    self.identity(),
                    program.clone(),
                    3,
                )])
            }
        }

        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Bulk)).expect("one rule");
        let subject = program(2.0, 2.0, false);

        let mut under = DeterministicBudgets::governed();
        under.normalization_rewrites = 2;
        assert!(
            matches!(
                run_rewrite_engine(&registry, &subject, under).expect("no failure"),
                EngineRun::BudgetStopped {
                    limit: 2,
                    demand: 3
                }
            ),
            "three rewrites were admitted under a budget of two, so the budget \
             counted proposals rather than rewrites"
        );

        let mut exact = DeterministicBudgets::governed();
        exact.normalization_rewrites = 3;
        assert_eq!(
            run_rewrite_engine(&registry, &subject, exact)
                .expect("no failure")
                .expect("a budget equal to the demand admits it")
                .len(),
            1,
            "a budget exactly meeting the demand refused it"
        );
    }

    /// Two alternatives that each fit the budget are both admitted, even when
    /// their sum would not fit.
    ///
    /// The regression this pins: the budget summed rewrite counts across
    /// proposals, but proposals are mutually exclusive candidates — at most one
    /// is adopted — so the sum over-charged and a legal pair was refused the
    /// moment a second rule existed. Each of the two rules below proposes 3
    /// rewrites under a budget of 4: individually legal, jointly 6 > 4. The
    /// engine must admit both; a summing budget stops with demand 6.
    #[test]
    fn alternatives_are_budgeted_independently_not_summed() {
        struct Bulk(&'static str);
        impl RewriteRuleProvider<SemanticProgram> for Bulk {
            fn identity(&self) -> RewriteRuleIdentity {
                RewriteRuleIdentity::new("test", self.0, 1).expect("named")
            }
            fn propose(
                &self,
                program: &SemanticProgram,
            ) -> Result<Vec<RewriteProposal<SemanticProgram>>, ProviderDefect> {
                Ok(vec![RewriteProposal::new(
                    self.identity(),
                    program.clone(),
                    3,
                )])
            }
        }

        let mut registry = RuleRegistry::new();
        registry.register(Box::new(Bulk("a"))).expect("distinct");
        registry.register(Box::new(Bulk("b"))).expect("distinct");
        let subject = program(2.0, 2.0, false);

        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 4;
        let EngineRun::Adopted(alternatives) =
            run_rewrite_engine(&registry, &subject, budgets).expect("no failure")
        else {
            panic!("two individually-legal alternatives were refused as a sum");
        };
        assert_eq!(alternatives.len(), 2);

        // And a budget below either still stops, reporting the largest
        // offender's demand — the smallest budget that would admit everything.
        budgets.normalization_rewrites = 2;
        assert!(
            matches!(
                run_rewrite_engine(&registry, &subject, budgets).expect("no failure"),
                EngineRun::BudgetStopped {
                    limit: 2,
                    demand: 3
                }
            ),
            "an over-budget alternative was admitted, or the demand figure moved"
        );
    }

    #[test]
    fn an_exhausted_rewrite_budget_abandons_the_whole_rewrite() {
        let duplicated = program(2.0, 2.0, false);
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;
        let outcome =
            normalize_semantics(&duplicated, budgets, StrictF32NumericalContract::governed())
                .unwrap();

        assert!(outcome.normalized_program().is_none());
        assert_eq!(outcome.rewrite_count(), 0);
        assert_eq!(outcome.budget_stop(), Some((0, 1)));
        assert_eq!(outcome.operations_after, duplicated.operation_count());
    }

    #[test]
    fn stage_records_are_typed_bounded_and_causally_chained() {
        let duplicated = program(2.0, 2.0, false);
        let outcome = normalize(&duplicated);
        let normalized = outcome.normalized_program().unwrap();
        let verified = verify_request(CompilationRequest::governed(normalized)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target).unwrap();

        let root = test_root(&mut explain);
        let receipt = outcome.record(&mut explain, root).unwrap();
        let alternative = explain
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        explain
            .note_selection(
                alternative,
                crate::explain::SelectionOutcome::Selected,
                None,
            )
            .unwrap();
        let trace = explain
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();

        let merge = trace
            .records()
            .iter()
            .find(|record| record.rule().key().as_str() == NORMALIZE_SHARED_VALUE_RULE)
            .expect("the committed merge is explained");
        let summary = trace
            .records()
            .iter()
            .find(|record| record.id() == receipt)
            .expect("the stage receipt is retained");
        assert_eq!(summary.causes(), [merge.id()]);
        assert_eq!(summary.subjects()[0].key().as_str(), NORMALIZATION_SUBJECT);
        let ExplainEvent::Check { assessment, .. } = summary.event() else {
            panic!("the stage receipt is a checked assertion");
        };
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "rewrite-count" && matches!(fact.value(), FactValue::Count(1))
        }));
        // Compared against the contract's own key rather than a literal: the key
        // is derived from the dimension vector, so a literal here would have to
        // be rebaselined by hand every time the scheme moved and would not be
        // checking the thing it names — that the fact carries *this* contract's
        // identity.
        let strict = StrictF32NumericalContract::governed().key;
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "numerical-contract"
                && matches!(fact.value(), FactValue::Identity(key)
                    if key.as_str() == strict)
        }));
        assert!(trace.render().contains("normalization admitted"));
    }

    #[test]
    fn budget_stop_is_rendered_as_a_typed_normalization_event() {
        let duplicated = program(2.0, 2.0, false);
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;
        let outcome =
            normalize_semantics(&duplicated, budgets, StrictF32NumericalContract::governed())
                .unwrap();
        let verified = verify_request(CompilationRequest::governed(&duplicated)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target).unwrap();

        let root = test_root(&mut explain);
        outcome.record(&mut explain, root).unwrap();
        let alternative = explain
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        explain
            .note_selection(
                alternative,
                crate::explain::SelectionOutcome::Selected,
                None,
            )
            .unwrap();
        let trace = explain
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();

        assert!(trace.records().iter().any(|record| {
            matches!(
                record.event(),
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::Normalization,
                    limit: 0,
                    actual: 1,
                    ..
                }
            )
        }));
        assert!(
            trace
                .render()
                .contains("budget-stop:normalization-rewrites:0:1")
        );
    }

    #[test]
    fn errors_report_their_exact_class_and_rule() {
        let error = NormalizeError::InvalidRewrite { rule: "fixpoint" };
        assert_eq!(error.reason(), "fixpoint");
        assert_eq!(
            error.to_string(),
            "compile.normalize.invalid-rewrite.fixpoint: deterministic normalization produced invalid compiler output"
        );
        assert_eq!(
            NormalizeError::Structure {
                rule: "value-ordinal"
            }
            .to_string(),
            "compile.normalize.structure.value-ordinal: deterministic normalization produced invalid compiler output"
        );
        assert_eq!(
            NormalizeError::Rebuild { rule: "input" }.to_string(),
            "compile.normalize.rebuild.input: deterministic normalization produced invalid compiler output"
        );
    }
}
