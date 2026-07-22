#![allow(
    dead_code,
    reason = "private draft reserves reviewed stage/evidence views before the public facade"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::fusion::FusionNumericalProof;
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};

pub(crate) const EXPLAIN_SCHEMA_VERSION: u32 = 2;
pub(crate) const EXPLAIN_RENDERER_VERSION: u32 = 2;
const MAX_KEY_BYTES: usize = 255;
const MAX_RECORDS: u32 = 4_096;
const MAX_CANONICAL_BYTES: u32 = 1024 * 1024;
const MAX_TERMINAL_LEDGER_RECORDS: u32 = MAX_RECORDS;
const MAX_TERMINAL_LEDGER_BYTES: u32 = MAX_CANONICAL_BYTES;
const MAX_TERMINAL_RECORD_BYTES: u32 = 1_024;
const MAX_TRACE_RECORDS: u32 = MAX_RECORDS + MAX_TERMINAL_LEDGER_RECORDS * 2 + 1;
const MAX_TRACE_CANONICAL_BYTES: u32 = MAX_CANONICAL_BYTES * 2
    + MAX_TERMINAL_LEDGER_RECORDS * 2 * MAX_TERMINAL_RECORD_BYTES
    + MAX_TERMINAL_RECORD_BYTES;
const MAX_SUBJECTS_PER_RECORD: u32 = 16;
pub(crate) const MAX_TERMINAL_CAUSES: u32 = 16;
const MAX_CAUSES_PER_RECORD: u32 = MAX_TERMINAL_CAUSES;
const MAX_FACTS_PER_ASSESSMENT: u32 = 32;
const MAX_COST_TERMS: u32 = 32;
static NEXT_WRITER_AUTHORITY: AtomicU64 = AtomicU64::new(1);

macro_rules! key_type {
    ($name:ident, $kind:expr) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(crate) struct $name(String);

        impl $name {
            pub(crate) fn new(value: impl AsRef<str>) -> Result<Self, ExplainError> {
                validate_key($kind, value.as_ref())?;
                Ok(Self(value.as_ref().to_owned()))
            }

            pub(crate) fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

key_type!(RuleKey, KeyKind::Rule);
key_type!(ReasonCode, KeyKind::Reason);
key_type!(ProviderKey, KeyKind::Provider);
key_type!(PredicateKey, KeyKind::Predicate);
key_type!(ResourceKey, KeyKind::Resource);
key_type!(CostModelKey, KeyKind::CostModel);
key_type!(CostMetricKey, KeyKind::CostMetric);
key_type!(SubjectKey, KeyKind::Subject);
key_type!(FactKey, KeyKind::Fact);
key_type!(SelectionPolicyKey, KeyKind::SelectionPolicy);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KeyKind {
    Rule,
    Reason,
    Provider,
    Predicate,
    Resource,
    CostModel,
    CostMetric,
    Subject,
    Fact,
    SelectionPolicy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ExplainStage {
    RequestVerification,
    Normalization,
    RegionFormation,
    CandidateEnumeration,
    CapabilityResolution,
    NumericalLegality,
    IntrinsicScheduling,
    TargetFeasibility,
    Costing,
    Selection,
    KernelRefinement,
    ProgramVerification,
    ArtifactPlanning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainDisposition {
    Admitted,
    RejectedIntrinsic,
    RejectedNumerical,
    RejectedTarget,
    DeferredUnsupported,
    BudgetStopped,
    Retained,
    DominancePruned,
    HigherCost,
    NotSelectedTradeoff,
    Selected,
    CompilerFailure,
    Truncated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SubjectKind {
    SemanticProgram,
    Normalization,
    Region,
    Boundary,
    Candidate,
    Capability,
    Schedule,
    Target,
    Kernel,
    KernelProgram,
    ArtifactPlan,
    Alternative,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProviderRef {
    key: ProviderKey,
    revision: u32,
}

impl ProviderRef {
    pub(crate) fn builtin() -> Self {
        Self {
            key: ProviderKey::new("tiler.compiler").expect("builtin provider key is valid"),
            revision: 1,
        }
    }

    pub(crate) fn lowering(provider: LoweringProviderIdentity) -> Result<Self, ExplainError> {
        Ok(Self {
            key: ProviderKey::new(provider.key)?,
            revision: provider.revision,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RuleRef {
    key: RuleKey,
    revision: u32,
    provider: ProviderRef,
}

impl RuleRef {
    pub(crate) fn builtin(key: impl AsRef<str>) -> Result<Self, ExplainError> {
        Ok(Self {
            key: RuleKey::new(key)?,
            revision: 1,
            provider: ProviderRef::builtin(),
        })
    }

    pub(crate) fn provided(
        key: impl AsRef<str>,
        revision: u32,
        provider: ProviderRef,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            key: RuleKey::new(key)?,
            revision,
            provider,
        })
    }

    pub(crate) const fn key(&self) -> &RuleKey {
        &self.key
    }

    pub(crate) const fn provider(&self) -> &ProviderRef {
        &self.provider
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompilationSubject {
    canonical: Box<[u8]>,
}

impl CompilationSubject {
    pub(crate) fn from_request(request: &VerifiedTargetRequest) -> Self {
        let request = request.subject();
        let canonical = request.canonical_explain_subject_bytes().into_boxed_slice();
        Self { canonical }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubjectRef {
    compilation: CompilationSubject,
    kind: SubjectKind,
    key: SubjectKey,
}

impl SubjectRef {
    pub(crate) const fn kind(&self) -> SubjectKind {
        self.kind
    }

    pub(crate) const fn key(&self) -> &SubjectKey {
        &self.key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceBasis {
    NormativeGuarantee,
    CheckedInvariant,
    SoundProof(VerifiedEvidenceRef),
    ExhaustiveFinite,
    Empirical,
    Assumption,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedEvidenceRef {
    kind: EvidenceReceiptKind,
    compilation: Box<[u8]>,
    candidate: SubjectKey,
    provider: ProviderRef,
    proof: Box<[u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceReceiptKind {
    FusionNumerical,
}

impl VerifiedEvidenceRef {
    pub(crate) fn from_fusion_numerical(
        request: &VerifiedTargetRequest,
        proof: &FusionNumericalProof,
        provider: ProviderRef,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            kind: EvidenceReceiptKind::FusionNumerical,
            compilation: request
                .subject()
                .canonical_explain_subject_bytes()
                .into_boxed_slice(),
            candidate: SubjectKey::new(proof.candidate_stable_id())?,
            provider,
            proof: proof.canonical_explain_evidence_bytes().into_boxed_slice(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Assessment {
    Proven,
    Disproved(ReasonCode),
    Unknown(ReasonCode),
    Deferred(ReasonCode),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FactValue {
    Count(u64),
    Bytes(u64),
    Threads(u64),
    Bindings(u64),
    Boolean(bool),
    Identity(SubjectKey),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainFact {
    key: FactKey,
    value: FactValue,
}

impl ExplainFact {
    pub(crate) fn new(key: impl AsRef<str>, value: FactValue) -> Result<Self, ExplainError> {
        Ok(Self {
            key: FactKey::new(key)?,
            value,
        })
    }

    pub(crate) const fn key(&self) -> &FactKey {
        &self.key
    }

    pub(crate) const fn value(&self) -> &FactValue {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PredicateAssessment {
    predicate: PredicateKey,
    assessment: Assessment,
    basis: EvidenceBasis,
    facts: Vec<ExplainFact>,
}

impl PredicateAssessment {
    pub(crate) fn proven(
        predicate: impl AsRef<str>,
        basis: EvidenceBasis,
    ) -> Result<Self, ExplainError> {
        if !matches!(
            basis,
            EvidenceBasis::NormativeGuarantee
                | EvidenceBasis::CheckedInvariant
                | EvidenceBasis::SoundProof(_)
                | EvidenceBasis::ExhaustiveFinite
        ) {
            return Err(ExplainError::EvidenceEscalation);
        }
        Ok(Self {
            predicate: PredicateKey::new(predicate)?,
            assessment: Assessment::Proven,
            basis,
            facts: Vec::new(),
        })
    }

    pub(crate) fn disproved(
        predicate: impl AsRef<str>,
        reason: ReasonCode,
        basis: EvidenceBasis,
    ) -> Result<Self, ExplainError> {
        if !matches!(
            basis,
            EvidenceBasis::NormativeGuarantee
                | EvidenceBasis::CheckedInvariant
                | EvidenceBasis::SoundProof(_)
                | EvidenceBasis::ExhaustiveFinite
        ) {
            return Err(ExplainError::EvidenceEscalation);
        }
        Ok(Self {
            predicate: PredicateKey::new(predicate)?,
            assessment: Assessment::Disproved(reason),
            basis,
            facts: Vec::new(),
        })
    }

    pub(crate) fn with_fact(mut self, fact: ExplainFact) -> Result<Self, ExplainError> {
        check_bound(
            BoundKind::Facts,
            MAX_FACTS_PER_ASSESSMENT,
            self.facts.len() + 1,
        )?;
        self.facts.push(fact);
        Ok(self)
    }

    pub(crate) fn facts(&self) -> &[ExplainFact] {
        &self.facts
    }

    pub(crate) const fn predicate(&self) -> &PredicateKey {
        &self.predicate
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Quantity {
    Count(u64),
    Bytes(u64),
    Threads(u64),
    Bindings(u64),
}

impl Quantity {
    const fn kind(self) -> u8 {
        match self {
            Self::Count(_) => 1,
            Self::Bytes(_) => 2,
            Self::Threads(_) => 3,
            Self::Bindings(_) => 4,
        }
    }

    const fn value(self) -> u64 {
        match self {
            Self::Count(value)
            | Self::Bytes(value)
            | Self::Threads(value)
            | Self::Bindings(value) => value,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CostTerm {
    metric: CostMetricKey,
    quantity: Quantity,
}

impl CostTerm {
    pub(crate) fn new(metric: impl AsRef<str>, quantity: Quantity) -> Result<Self, ExplainError> {
        Ok(Self {
            metric: CostMetricKey::new(metric)?,
            quantity,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CostDisposition {
    Retained,
    Dominated,
    HigherCost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SelectionOutcome {
    Selected,
    Dominated,
    NotSelectedTradeoff,
    Infeasible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FeasibilityOutcome {
    Admitted,
    Rejected(ReasonCode),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RejectionClass {
    IntrinsicInvalid,
    NumericalIllegal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExplainEvent {
    Check {
        stage: ExplainStage,
        assessment: PredicateAssessment,
        rejection: RejectionClass,
    },
    BudgetStop {
        stage: ExplainStage,
        resource: ResourceKey,
        limit: u64,
        actual: u64,
    },
    Feasibility {
        predicate: PredicateKey,
        outcome: FeasibilityOutcome,
        required: Quantity,
        available: Quantity,
    },
    DeferredCapability {
        predicate: PredicateKey,
        reason: ReasonCode,
    },
    CostAssessment {
        model: CostModelKey,
        basis: EvidenceBasis,
        terms: Vec<CostTerm>,
        disposition: CostDisposition,
    },
    Selection {
        policy: SelectionPolicyKey,
        outcome: SelectionOutcome,
    },
    CompilerFailure {
        stage: ExplainStage,
        reason: ReasonCode,
    },
    CausalBridge {
        stage: ExplainStage,
        disposition: ExplainDisposition,
    },
    Truncated {
        omitted_records: u64,
        omitted_bytes: u64,
    },
}

impl ExplainEvent {
    pub(crate) const fn stage(&self) -> ExplainStage {
        match self {
            Self::Check { stage, .. }
            | Self::BudgetStop { stage, .. }
            | Self::CompilerFailure { stage, .. }
            | Self::CausalBridge { stage, .. } => *stage,
            Self::Feasibility { .. } => ExplainStage::TargetFeasibility,
            Self::DeferredCapability { .. } => ExplainStage::CapabilityResolution,
            Self::CostAssessment { .. } => ExplainStage::Costing,
            Self::Selection { .. } | Self::Truncated { .. } => ExplainStage::Selection,
        }
    }

    pub(crate) const fn disposition(&self) -> ExplainDisposition {
        match self {
            Self::Check {
                assessment:
                    PredicateAssessment {
                        assessment: Assessment::Proven,
                        ..
                    },
                ..
            }
            | Self::Feasibility {
                outcome: FeasibilityOutcome::Admitted,
                ..
            } => ExplainDisposition::Admitted,
            Self::Check {
                assessment:
                    PredicateAssessment {
                        assessment: Assessment::Disproved(_),
                        ..
                    },
                rejection: RejectionClass::IntrinsicInvalid,
                ..
            } => ExplainDisposition::RejectedIntrinsic,
            Self::Check {
                assessment:
                    PredicateAssessment {
                        assessment: Assessment::Disproved(_),
                        ..
                    },
                rejection: RejectionClass::NumericalIllegal,
                ..
            } => ExplainDisposition::RejectedNumerical,
            Self::Check {
                assessment:
                    PredicateAssessment {
                        assessment: Assessment::Unknown(_) | Assessment::Deferred(_),
                        ..
                    },
                ..
            }
            | Self::DeferredCapability { .. } => ExplainDisposition::DeferredUnsupported,
            Self::BudgetStop { .. } => ExplainDisposition::BudgetStopped,
            Self::Feasibility {
                outcome: FeasibilityOutcome::Rejected(_),
                ..
            }
            | Self::Selection {
                outcome: SelectionOutcome::Infeasible,
                ..
            } => ExplainDisposition::RejectedTarget,
            Self::CostAssessment {
                disposition: CostDisposition::Retained,
                ..
            } => ExplainDisposition::Retained,
            Self::CostAssessment {
                disposition: CostDisposition::Dominated,
                ..
            }
            | Self::Selection {
                outcome: SelectionOutcome::Dominated,
                ..
            } => ExplainDisposition::DominancePruned,
            Self::CostAssessment {
                disposition: CostDisposition::HigherCost,
                ..
            } => ExplainDisposition::HigherCost,
            Self::Selection {
                outcome: SelectionOutcome::NotSelectedTradeoff,
                ..
            } => ExplainDisposition::NotSelectedTradeoff,
            Self::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            } => ExplainDisposition::Selected,
            Self::CompilerFailure { .. } => ExplainDisposition::CompilerFailure,
            Self::CausalBridge { disposition, .. } => *disposition,
            Self::Truncated { .. } => ExplainDisposition::Truncated,
        }
    }

    fn validate(&self) -> Result<(), ExplainError> {
        match self {
            Self::Check {
                stage,
                assessment,
                rejection,
            } => {
                if matches!(
                    stage,
                    ExplainStage::TargetFeasibility
                        | ExplainStage::Costing
                        | ExplainStage::Selection
                        | ExplainStage::CapabilityResolution
                ) || matches!(&assessment.basis, EvidenceBasis::SoundProof(_))
                    && *stage != ExplainStage::NumericalLegality
                {
                    return Err(ExplainError::InvalidStageEvent);
                }
                let rejection_matches_stage = matches!(
                    (stage, rejection),
                    (
                        ExplainStage::NumericalLegality,
                        RejectionClass::NumericalIllegal
                    ) | (
                        ExplainStage::RequestVerification
                            | ExplainStage::Normalization
                            | ExplainStage::RegionFormation
                            | ExplainStage::CandidateEnumeration
                            | ExplainStage::IntrinsicScheduling
                            | ExplainStage::KernelRefinement
                            | ExplainStage::ProgramVerification
                            | ExplainStage::ArtifactPlanning,
                        RejectionClass::IntrinsicInvalid
                    )
                );
                if !rejection_matches_stage {
                    return Err(ExplainError::InvalidStageEvent);
                }
                check_bound(
                    BoundKind::Facts,
                    MAX_FACTS_PER_ASSESSMENT,
                    assessment.facts.len(),
                )?;
            }
            Self::Feasibility {
                outcome,
                required,
                available,
                ..
            } => {
                if required.kind() != available.kind() {
                    return Err(ExplainError::QuantityKindMismatch);
                }
                let exceeds = required.value() > available.value();
                if matches!(outcome, FeasibilityOutcome::Admitted) == exceeds {
                    return Err(ExplainError::InvalidQuantityRelation);
                }
            }
            Self::CostAssessment { basis, terms, .. } => {
                if matches!(basis, EvidenceBasis::SoundProof(_) | EvidenceBasis::Unknown) {
                    return Err(ExplainError::EvidenceEscalation);
                }
                if terms.is_empty() {
                    return Err(ExplainError::EmptyCostEvidence);
                }
                check_bound(BoundKind::CostTerms, MAX_COST_TERMS, terms.len())?;
            }
            Self::BudgetStop { limit, actual, .. } if actual <= limit => {
                return Err(ExplainError::InvalidQuantityRelation);
            }
            Self::BudgetStop { .. }
            | Self::DeferredCapability { .. }
            | Self::Selection { .. }
            | Self::CompilerFailure { .. }
            | Self::CausalBridge { .. }
            | Self::Truncated { .. } => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ExplainRecordId {
    local: u32,
    writer_authority: u64,
    request_qualifier: u64,
}

impl PartialEq for ExplainRecordId {
    fn eq(&self, other: &Self) -> bool {
        self.local == other.local
    }
}

impl Eq for ExplainRecordId {}

impl PartialOrd for ExplainRecordId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExplainRecordId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.local.cmp(&other.local)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainRecord {
    id: ExplainRecordId,
    rule: RuleRef,
    subjects: Vec<SubjectRef>,
    event: ExplainEvent,
    causes: Vec<ExplainRecordId>,
}

impl ExplainRecord {
    pub(crate) const fn id(&self) -> ExplainRecordId {
        self.id
    }

    pub(crate) const fn event(&self) -> &ExplainEvent {
        &self.event
    }

    pub(crate) const fn rule(&self) -> &RuleRef {
        &self.rule
    }

    pub(crate) fn subjects(&self) -> &[SubjectRef] {
        &self.subjects
    }

    pub(crate) fn causes(&self) -> &[ExplainRecordId] {
        &self.causes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExplainLimits {
    max_records: u32,
    max_canonical_bytes: u32,
}

impl ExplainLimits {
    pub(crate) const fn new(
        max_records: u32,
        max_canonical_bytes: u32,
    ) -> Result<Self, ExplainError> {
        if max_records > MAX_RECORDS || max_canonical_bytes > MAX_CANONICAL_BYTES {
            return Err(ExplainError::InvalidLimits);
        }
        Ok(Self {
            max_records,
            max_canonical_bytes,
        })
    }
}

impl Default for ExplainLimits {
    fn default() -> Self {
        Self {
            max_records: 256,
            max_canonical_bytes: 64 * 1024,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExplainWriter {
    subject: CompilationSubject,
    authority: u64,
    request_qualifier: u64,
    allowed_providers: Vec<ProviderRef>,
    limits: ExplainLimits,
    records: Vec<ExplainRecord>,
    retained_bytes: usize,
    retained_detail_records: usize,
    retained_detail_bytes: usize,
    omitted_records: u64,
    omitted_bytes: u64,
    selection_ledger: BTreeMap<SubjectKey, PendingSelection>,
    terminal_ledger_bytes: usize,
}

#[derive(Clone, Debug)]
struct PendingSelection {
    outcome: SelectionOutcome,
    cause: Option<TerminalCause>,
    authoritative_infeasible: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct TerminalCause {
    kind: TerminalCauseKind,
}

#[derive(Clone, Debug)]
enum TerminalCauseKind {
    Record(ExplainRecordId),
    Omitted {
        rule: RuleRef,
        subject_kind: SubjectKind,
        subject_key: SubjectKey,
        stage: ExplainStage,
        disposition: ExplainDisposition,
        causes: Vec<ExplainRecordId>,
    },
}

impl TerminalCause {
    pub(crate) const fn from_record(record: ExplainRecordId) -> Self {
        Self {
            kind: TerminalCauseKind::Record(record),
        }
    }

    fn retained_bytes(&self) -> usize {
        match &self.kind {
            TerminalCauseKind::Record(_) => std::mem::size_of::<ExplainRecordId>(),
            TerminalCauseKind::Omitted {
                rule,
                subject_key,
                causes,
                ..
            } => rule
                .key
                .as_str()
                .len()
                .saturating_add(rule.provider.key.as_str().len())
                .saturating_add(subject_key.as_str().len())
                .saturating_add(
                    causes
                        .len()
                        .saturating_mul(std::mem::size_of::<ExplainRecordId>()),
                )
                .saturating_add(16),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FailureDescriptor {
    pub(crate) stage: ExplainStage,
    pub(crate) reason: ReasonCode,
    pub(crate) subject_kind: SubjectKind,
    pub(crate) subject_key: SubjectKey,
    causes: Vec<TerminalCause>,
}

impl FailureDescriptor {
    pub(crate) fn new(
        stage: ExplainStage,
        reason: impl AsRef<str>,
        subject_kind: SubjectKind,
        subject_key: impl AsRef<str>,
        cause: Option<TerminalCause>,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            stage,
            reason: ReasonCode::new(reason)?,
            subject_kind,
            subject_key: SubjectKey::new(subject_key)?,
            causes: cause.into_iter().collect(),
        })
    }

    pub(crate) fn with_causes(
        stage: ExplainStage,
        reason: impl AsRef<str>,
        subject_kind: SubjectKind,
        subject_key: impl AsRef<str>,
        causes: Vec<TerminalCause>,
    ) -> Result<Self, ExplainError> {
        check_bound(BoundKind::Causes, MAX_CAUSES_PER_RECORD, causes.len())?;
        Ok(Self {
            stage,
            reason: ReasonCode::new(reason)?,
            subject_kind,
            subject_key: SubjectKey::new(subject_key)?,
            causes,
        })
    }
}

impl ExplainWriter {
    pub(crate) fn new(
        request: &VerifiedTargetRequest,
        limits: ExplainLimits,
    ) -> Result<Self, ExplainError> {
        let subject = CompilationSubject::from_request(request);
        let mut allowed_providers = vec![ProviderRef::lowering(
            request.capabilities().materialized_serial_sum,
        )?];
        if let Some(provider) = request.capabilities().fused_serial_sum {
            allowed_providers.push(ProviderRef::lowering(provider)?);
        }
        let retained_bytes = encode_trace(EXPLAIN_SCHEMA_VERSION, &subject, &[]).len();
        if retained_bytes > usize::try_from(MAX_CANONICAL_BYTES).unwrap_or(usize::MAX) {
            return Err(ExplainError::TerminalCapacity);
        }
        Ok(Self {
            authority: NEXT_WRITER_AUTHORITY.fetch_add(1, Ordering::Relaxed),
            request_qualifier: stable_qualifier(&subject.canonical),
            subject,
            allowed_providers,
            limits,
            records: Vec::new(),
            retained_bytes,
            retained_detail_records: 0,
            retained_detail_bytes: 0,
            omitted_records: 0,
            omitted_bytes: 0,
            selection_ledger: BTreeMap::new(),
            terminal_ledger_bytes: 0,
        })
    }

    pub(crate) fn subject(
        &self,
        kind: SubjectKind,
        key: impl AsRef<str>,
    ) -> Result<SubjectRef, ExplainError> {
        Ok(SubjectRef {
            compilation: self.subject.clone(),
            kind,
            key: SubjectKey::new(key)?,
        })
    }

    pub(crate) fn push_detail(
        &mut self,
        rule: RuleRef,
        subjects: Vec<SubjectRef>,
        event: ExplainEvent,
        causes: Vec<ExplainRecordId>,
    ) -> Result<Option<ExplainRecordId>, ExplainError> {
        if matches!(
            event,
            ExplainEvent::Selection { .. }
                | ExplainEvent::CompilerFailure { .. }
                | ExplainEvent::CausalBridge { .. }
                | ExplainEvent::Truncated { .. }
        ) {
            return Err(ExplainError::InvalidEventClass);
        }
        self.push(rule, subjects, event, causes, false)
    }

    pub(crate) fn push_causal_detail(
        &mut self,
        rule: RuleRef,
        subject: SubjectRef,
        event: &ExplainEvent,
        mut causes: Vec<ExplainRecordId>,
    ) -> Result<TerminalCause, ExplainError> {
        causes.sort_unstable();
        let retained = self.push_detail(
            rule.clone(),
            vec![subject.clone()],
            event.clone(),
            causes.clone(),
        )?;
        Ok(match retained {
            Some(record) => TerminalCause::from_record(record),
            None => TerminalCause {
                kind: TerminalCauseKind::Omitted {
                    rule,
                    subject_kind: subject.kind,
                    subject_key: subject.key,
                    stage: event.stage(),
                    disposition: event.disposition(),
                    causes,
                },
            },
        })
    }

    fn push_terminal(
        &mut self,
        rule: RuleRef,
        subjects: Vec<SubjectRef>,
        event: ExplainEvent,
        causes: Vec<ExplainRecordId>,
    ) -> Result<ExplainRecordId, ExplainError> {
        if !matches!(
            event,
            ExplainEvent::Selection { .. }
                | ExplainEvent::CompilerFailure { .. }
                | ExplainEvent::CausalBridge { .. }
                | ExplainEvent::Truncated { .. }
        ) {
            return Err(ExplainError::InvalidEventClass);
        }
        self.push(rule, subjects, event, causes, true)?
            .ok_or(ExplainError::TerminalCapacity)
    }

    fn push(
        &mut self,
        rule: RuleRef,
        mut subjects: Vec<SubjectRef>,
        mut event: ExplainEvent,
        mut causes: Vec<ExplainRecordId>,
        terminal: bool,
    ) -> Result<Option<ExplainRecordId>, ExplainError> {
        canonicalize_record_parts(&mut subjects, &mut event, &mut causes)?;
        event.validate()?;
        if rule.provider != ProviderRef::builtin()
            && !self.allowed_providers.contains(&rule.provider)
        {
            return Err(ExplainError::ProviderAuthorityMismatch);
        }
        if subjects.is_empty() {
            return Err(ExplainError::EmptySubjects);
        }
        check_bound(BoundKind::Subjects, MAX_SUBJECTS_PER_RECORD, subjects.len())?;
        check_bound(BoundKind::Causes, MAX_CAUSES_PER_RECORD, causes.len())?;
        if subjects
            .iter()
            .any(|subject| subject.compilation != self.subject)
        {
            return Err(ExplainError::CrossCompilationSubject);
        }
        if let ExplainEvent::Check {
            assessment:
                PredicateAssessment {
                    basis: EvidenceBasis::SoundProof(receipt),
                    ..
                },
            ..
        } = &event
            && (receipt.compilation.as_ref() != self.subject.canonical.as_ref()
                || receipt.provider != rule.provider
                || subjects.len() != 1
                || subjects[0].kind != SubjectKind::Candidate
                || subjects[0].key != receipt.candidate)
        {
            return Err(ExplainError::EvidenceSubjectMismatch);
        }
        if causes.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ExplainError::DuplicateCause);
        }
        if causes.iter().any(|cause| {
            cause.writer_authority != self.authority
                || cause.request_qualifier != self.request_qualifier
        }) {
            return Err(ExplainError::CrossWriterCause);
        }
        let next = ExplainRecordId {
            local: u32::try_from(self.records.len()).map_err(|_| ExplainError::TerminalCapacity)?,
            writer_authority: self.authority,
            request_qualifier: self.request_qualifier,
        };
        if causes.iter().any(|cause| cause.local >= next.local) {
            return Err(ExplainError::InvalidCause {
                cause: *causes
                    .iter()
                    .find(|cause| cause.local >= next.local)
                    .expect("checked cause exists"),
                next,
            });
        }
        let record = ExplainRecord {
            id: next,
            rule,
            subjects,
            event,
            causes,
        };
        let bytes = encode_record(&record).len();
        if terminal && bytes > usize::try_from(MAX_TERMINAL_RECORD_BYTES).unwrap_or(usize::MAX) {
            return Err(ExplainError::TerminalCapacity);
        }
        let exceeds = if terminal {
            self.records.len().saturating_add(1)
                > usize::try_from(MAX_TRACE_RECORDS).unwrap_or(usize::MAX)
                || self.retained_bytes.saturating_add(bytes)
                    > usize::try_from(MAX_TRACE_CANONICAL_BYTES).unwrap_or(usize::MAX)
        } else {
            self.retained_detail_records.saturating_add(1)
                > usize::try_from(self.limits.max_records).unwrap_or(usize::MAX)
                || self.retained_detail_bytes.saturating_add(bytes)
                    > usize::try_from(self.limits.max_canonical_bytes).unwrap_or(usize::MAX)
        };
        if exceeds && !terminal {
            self.omitted_records = self.omitted_records.saturating_add(1);
            self.omitted_bytes = self
                .omitted_bytes
                .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
            return Ok(None);
        }
        if exceeds {
            return Err(ExplainError::TerminalCapacity);
        }
        self.retained_bytes += bytes;
        if !terminal {
            self.retained_detail_records += 1;
            self.retained_detail_bytes += bytes;
        }
        self.records.push(record);
        Ok(Some(next))
    }

    pub(crate) fn finish_success(
        mut self,
        alternatives: &[&str],
        selected: &str,
    ) -> Result<VerifiedExplainTrace, ExplainError> {
        check_terminal_ledger_bound(alternatives.len(), alternatives.iter().map(|key| key.len()))?;
        let mut expected = BTreeSet::new();
        for alternative in alternatives {
            let key = SubjectKey::new(alternative)?;
            if !expected.insert(key) {
                return Err(ExplainError::InvalidTerminalLedger);
            }
        }
        for (key, pending) in &self.selection_ledger {
            if pending.authoritative_infeasible && !expected.insert(key.clone()) {
                return Err(ExplainError::InvalidTerminalLedger);
            }
        }
        let selected = SubjectKey::new(selected)?;
        if !expected.contains(&selected)
            || self.selection_ledger.len() != expected.len()
            || self.selection_ledger.keys().ne(expected.iter())
        {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        for (key, pending) in &self.selection_ledger {
            let should_select = key == &selected;
            let is_infeasible = pending.authoritative_infeasible;
            if (pending.outcome == SelectionOutcome::Selected) != should_select
                || (pending.outcome == SelectionOutcome::Infeasible) != is_infeasible
            {
                return Err(ExplainError::InvalidTerminalLedger);
            }
        }
        self.append_truncation_summary()?;
        for (key, pending) in std::mem::take(&mut self.selection_ledger) {
            let cause = pending
                .cause
                .map(|cause| self.materialize_terminal_cause(cause))
                .transpose()?;
            let subject = self.subject(SubjectKind::Alternative, key.as_str())?;
            self.push_terminal(
                RuleRef::builtin("tiler.selection.structural-pareto.v1")?,
                vec![subject],
                ExplainEvent::Selection {
                    policy: SelectionPolicyKey::new("tiler.selection.structural-pareto.v1")?,
                    outcome: pending.outcome,
                },
                cause.into_iter().collect(),
            )?;
        }
        self.seal()
    }

    pub(crate) fn finish_failure(
        mut self,
        failure: FailureDescriptor,
    ) -> Result<VerifiedExplainTrace, ExplainError> {
        self.selection_ledger.clear();
        self.terminal_ledger_bytes = 0;
        self.append_truncation_summary()?;
        for cause in &failure.causes {
            self.validate_terminal_cause(Some(cause))?;
        }
        let mut causes = Vec::with_capacity(failure.causes.len());
        for cause in failure.causes {
            causes.push(self.materialize_terminal_cause(cause)?);
        }
        let subject = self.subject(failure.subject_kind, failure.subject_key.as_str())?;
        self.push_terminal(
            RuleRef::builtin("compile.failure")?,
            vec![subject],
            ExplainEvent::CompilerFailure {
                stage: failure.stage,
                reason: failure.reason,
            },
            causes,
        )?;
        let failures = self
            .records
            .iter()
            .filter(|record| matches!(record.event, ExplainEvent::CompilerFailure { .. }))
            .count();
        let selections = self
            .records
            .iter()
            .filter(|record| matches!(record.event, ExplainEvent::Selection { .. }))
            .count();
        if failures != 1 || selections != 0 {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        self.seal()
    }

    pub(crate) fn note_infeasible_alternative(
        &mut self,
        subject: SubjectRef,
        cause: Option<TerminalCause>,
    ) -> Result<(), ExplainError> {
        if subject.compilation != self.subject || subject.kind != SubjectKind::Alternative {
            return Err(ExplainError::CrossCompilationSubject);
        }
        let key = subject.key;
        self.admit_selection(key, SelectionOutcome::Infeasible, cause, true)
    }

    pub(crate) fn note_selection(
        &mut self,
        subject: SubjectRef,
        outcome: SelectionOutcome,
        cause: Option<TerminalCause>,
    ) -> Result<(), ExplainError> {
        if subject.compilation != self.subject || subject.kind != SubjectKind::Alternative {
            return Err(ExplainError::CrossCompilationSubject);
        }
        let key = subject.key;
        if outcome == SelectionOutcome::Infeasible || self.selection_ledger.contains_key(&key) {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        self.admit_selection(key, outcome, cause, false)?;
        Ok(())
    }

    fn admit_selection(
        &mut self,
        key: SubjectKey,
        outcome: SelectionOutcome,
        cause: Option<TerminalCause>,
        authoritative_infeasible: bool,
    ) -> Result<(), ExplainError> {
        if self.selection_ledger.contains_key(&key) {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        self.validate_terminal_cause(cause.as_ref())?;
        let entry_bytes = key
            .as_str()
            .len()
            .saturating_add(cause.as_ref().map_or(0, TerminalCause::retained_bytes))
            .saturating_add(16);
        let next_count = self.selection_ledger.len().saturating_add(1);
        let next_bytes = self.terminal_ledger_bytes.saturating_add(entry_bytes);
        check_terminal_ledger_bound(next_count, [next_bytes])?;
        self.selection_ledger.insert(
            key,
            PendingSelection {
                outcome,
                cause,
                authoritative_infeasible,
            },
        );
        self.terminal_ledger_bytes = next_bytes;
        Ok(())
    }

    fn validate_terminal_cause(&self, cause: Option<&TerminalCause>) -> Result<(), ExplainError> {
        match cause.map(|cause| &cause.kind) {
            Some(TerminalCauseKind::Record(cause))
                if cause.writer_authority != self.authority
                    || cause.request_qualifier != self.request_qualifier =>
            {
                Err(ExplainError::CrossWriterCause)
            }
            Some(TerminalCauseKind::Omitted { rule, .. })
                if rule.provider != ProviderRef::builtin()
                    && !self.allowed_providers.contains(&rule.provider) =>
            {
                Err(ExplainError::ProviderAuthorityMismatch)
            }
            Some(TerminalCauseKind::Omitted { causes, .. })
                if causes.iter().any(|cause| {
                    cause.writer_authority != self.authority
                        || cause.request_qualifier != self.request_qualifier
                }) =>
            {
                Err(ExplainError::CrossWriterCause)
            }
            _ => Ok(()),
        }
    }

    fn materialize_terminal_cause(
        &mut self,
        cause: TerminalCause,
    ) -> Result<ExplainRecordId, ExplainError> {
        self.validate_terminal_cause(Some(&cause))?;
        match cause.kind {
            TerminalCauseKind::Record(record) => Ok(record),
            TerminalCauseKind::Omitted {
                rule,
                subject_kind,
                subject_key,
                stage,
                disposition,
                causes,
            } => {
                let subject = self.subject(subject_kind, subject_key.as_str())?;
                self.push_terminal(
                    rule,
                    vec![subject],
                    ExplainEvent::CausalBridge { stage, disposition },
                    causes,
                )
            }
        }
    }

    fn append_truncation_summary(&mut self) -> Result<(), ExplainError> {
        if self.omitted_records == 0 {
            return Ok(());
        }
        let subject = self.subject(SubjectKind::KernelProgram, "explain-report")?;
        self.push_terminal(
            RuleRef::builtin("explain.retention")?,
            vec![subject],
            ExplainEvent::Truncated {
                omitted_records: self.omitted_records,
                omitted_bytes: self.omitted_bytes,
            },
            Vec::new(),
        )?;
        Ok(())
    }

    fn seal(self) -> Result<VerifiedExplainTrace, ExplainError> {
        if self.records.is_empty() {
            return Err(ExplainError::EmptyTrace);
        }
        let identity = encode_trace(EXPLAIN_SCHEMA_VERSION, &self.subject, &self.records);
        Ok(VerifiedExplainTrace {
            schema_version: EXPLAIN_SCHEMA_VERSION,
            compilation_subject: self.subject,
            records: self.records.into_boxed_slice(),
            canonical_identity: ExplainIdentity(identity.into_boxed_slice()),
        })
    }
}

fn canonicalize_record_parts(
    subjects: &mut [SubjectRef],
    event: &mut ExplainEvent,
    causes: &mut [ExplainRecordId],
) -> Result<(), ExplainError> {
    subjects.sort_by(|left, right| (left.kind, &left.key).cmp(&(right.kind, &right.key)));
    if subjects
        .windows(2)
        .any(|pair| pair[0].kind == pair[1].kind && pair[0].key == pair[1].key)
    {
        return Err(ExplainError::DuplicateSubject);
    }
    causes.sort_unstable();
    match event {
        ExplainEvent::Check { assessment, .. } => {
            assessment
                .facts
                .sort_by(|left, right| left.key.cmp(&right.key));
            if assessment
                .facts
                .windows(2)
                .any(|pair| pair[0].key == pair[1].key)
            {
                return Err(ExplainError::DuplicateFact);
            }
        }
        ExplainEvent::CostAssessment { terms, .. } => {
            terms.sort_by(|left, right| left.metric.cmp(&right.metric));
            if terms
                .windows(2)
                .any(|pair| pair[0].metric == pair[1].metric)
            {
                return Err(ExplainError::DuplicateCostTerm);
            }
        }
        _ => {}
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainIdentity(Box<[u8]>);

impl ExplainIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedExplainTrace {
    schema_version: u32,
    compilation_subject: CompilationSubject,
    records: Box<[ExplainRecord]>,
    canonical_identity: ExplainIdentity,
}

impl VerifiedExplainTrace {
    pub(crate) fn records(&self) -> &[ExplainRecord] {
        &self.records
    }

    pub(crate) const fn identity(&self) -> &ExplainIdentity {
        &self.canonical_identity
    }

    pub(crate) fn render(&self) -> String {
        let mut output = format!(
            "tiler-explain-v{EXPLAIN_RENDERER_VERSION} request={:016x}\n",
            stable_qualifier(&self.compilation_subject.canonical)
        );
        for record in &self.records {
            use fmt::Write as _;
            let _ = write!(
                output,
                "{} {} {} rule={}@{} provider={}@{} subject=",
                record.id.local,
                stage_name(record.event.stage()),
                disposition_name(record.event.disposition()),
                record.rule.key.as_str(),
                record.rule.revision,
                record.rule.provider.key.as_str(),
                record.rule.provider.revision,
            );
            for (index, subject) in record.subjects.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                let _ = write!(
                    output,
                    "{}:{}",
                    subject_kind_name(subject.kind),
                    subject.key.as_str()
                );
            }
            output.push_str(" event=");
            render_event(&mut output, &record.event);
            output.push_str(" causes=");
            if record.causes.is_empty() {
                output.push('-');
            } else {
                for (index, cause) in record.causes.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    let _ = write!(output, "{}", cause.local);
                }
            }
            output.push('\n');
        }
        output
    }

    #[cfg(test)]
    fn verify(&self) -> Result<(), ExplainError> {
        if self.records.is_empty()
            || self.records.len() > usize::try_from(MAX_TRACE_RECORDS).unwrap_or(usize::MAX)
            || self.canonical_identity.0.len()
                > usize::try_from(MAX_TRACE_CANONICAL_BYTES).unwrap_or(usize::MAX)
            || self.schema_version != EXPLAIN_SCHEMA_VERSION
            || encode_trace(
                self.schema_version,
                &self.compilation_subject,
                &self.records,
            )
            .as_slice()
                != self.canonical_identity.0.as_ref()
        {
            return Err(ExplainError::StaleIdentity);
        }
        for (index, record) in self.records.iter().enumerate() {
            let mut unique_causes = record.causes.clone();
            unique_causes.sort_unstable();
            if record.id.local != u32::try_from(index).unwrap_or(u32::MAX)
                || record.subjects.is_empty()
                || record.subjects.len()
                    > usize::try_from(MAX_SUBJECTS_PER_RECORD).unwrap_or(usize::MAX)
                || record.causes.len()
                    > usize::try_from(MAX_CAUSES_PER_RECORD).unwrap_or(usize::MAX)
                || unique_causes.windows(2).any(|pair| pair[0] == pair[1])
                || record
                    .subjects
                    .iter()
                    .any(|subject| subject.compilation != self.compilation_subject)
                || record
                    .causes
                    .iter()
                    .any(|cause| usize::try_from(cause.local).map_or(true, |cause| cause >= index))
            {
                return Err(ExplainError::StaleIdentity);
            }
            record.event.validate()?;
        }
        Ok(())
    }
}

fn render_event(output: &mut String, event: &ExplainEvent) {
    use fmt::Write as _;
    match event {
        ExplainEvent::Check { assessment, .. } => {
            let _ = write!(
                output,
                "check:{}:{}:{}",
                assessment.predicate.as_str(),
                assessment_text(&assessment.assessment),
                basis_name(&assessment.basis)
            );
            if !assessment.facts.is_empty() {
                output.push_str(":facts=");
                for (index, fact) in assessment.facts.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    let _ = write!(output, "{}:", fact.key.as_str());
                    render_fact_value(output, &fact.value);
                }
            }
        }
        ExplainEvent::BudgetStop {
            resource,
            limit,
            actual,
            ..
        } => {
            let _ = write!(output, "budget-stop:{}:{limit}:{actual}", resource.as_str());
        }
        ExplainEvent::Feasibility {
            predicate,
            outcome,
            required,
            available,
        } => {
            let _ = write!(
                output,
                "feasibility:{}:{}:{}={}:{}",
                predicate.as_str(),
                feasibility_text(outcome),
                quantity_name(*required),
                required.value(),
                available.value()
            );
        }
        ExplainEvent::DeferredCapability { predicate, reason } => {
            let _ = write!(
                output,
                "deferred:{}:{}",
                predicate.as_str(),
                reason.as_str()
            );
        }
        ExplainEvent::CostAssessment {
            model,
            basis,
            terms,
            disposition,
        } => {
            let _ = write!(
                output,
                "cost:{}:{}:{}:",
                model.as_str(),
                basis_name(basis),
                cost_disposition_name(*disposition)
            );
            for (index, term) in terms.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                let _ = write!(
                    output,
                    "{}:{}={}",
                    term.metric.as_str(),
                    quantity_name(term.quantity),
                    term.quantity.value()
                );
            }
        }
        ExplainEvent::Selection { policy, outcome } => {
            let _ = write!(
                output,
                "selection:{}:{}",
                policy.as_str(),
                selection_name(*outcome)
            );
        }
        ExplainEvent::CompilerFailure { reason, .. } => {
            let _ = write!(output, "compiler-failure:{}", reason.as_str());
        }
        ExplainEvent::CausalBridge { disposition, .. } => {
            let _ = write!(output, "omitted-cause:{}", disposition_name(*disposition));
        }
        ExplainEvent::Truncated {
            omitted_records,
            omitted_bytes,
        } => {
            let _ = write!(output, "truncated:{omitted_records}:{omitted_bytes}");
        }
    }
}

fn render_fact_value(output: &mut String, value: &FactValue) {
    use fmt::Write as _;
    match value {
        FactValue::Count(value) => {
            let _ = write!(output, "count={value}");
        }
        FactValue::Bytes(value) => {
            let _ = write!(output, "bytes={value}");
        }
        FactValue::Threads(value) => {
            let _ = write!(output, "threads={value}");
        }
        FactValue::Bindings(value) => {
            let _ = write!(output, "bindings={value}");
        }
        FactValue::Boolean(value) => {
            let _ = write!(output, "boolean={value}");
        }
        FactValue::Identity(value) => {
            let _ = write!(output, "identity={}", value.as_str());
        }
    }
}

const fn stage_name(stage: ExplainStage) -> &'static str {
    match stage {
        ExplainStage::RequestVerification => "request-verification",
        ExplainStage::Normalization => "normalization",
        ExplainStage::RegionFormation => "region-formation",
        ExplainStage::CandidateEnumeration => "candidate-enumeration",
        ExplainStage::CapabilityResolution => "capability-resolution",
        ExplainStage::NumericalLegality => "numerical-legality",
        ExplainStage::IntrinsicScheduling => "intrinsic-scheduling",
        ExplainStage::TargetFeasibility => "target-feasibility",
        ExplainStage::Costing => "costing",
        ExplainStage::Selection => "selection",
        ExplainStage::KernelRefinement => "kernel-refinement",
        ExplainStage::ProgramVerification => "program-verification",
        ExplainStage::ArtifactPlanning => "artifact-planning",
    }
}

const fn disposition_name(disposition: ExplainDisposition) -> &'static str {
    match disposition {
        ExplainDisposition::Admitted => "admitted",
        ExplainDisposition::RejectedIntrinsic => "rejected-intrinsic",
        ExplainDisposition::RejectedNumerical => "rejected-numerical",
        ExplainDisposition::RejectedTarget => "rejected-target",
        ExplainDisposition::DeferredUnsupported => "deferred-unsupported",
        ExplainDisposition::BudgetStopped => "budget-stopped",
        ExplainDisposition::Retained => "retained",
        ExplainDisposition::DominancePruned => "dominance-pruned",
        ExplainDisposition::HigherCost => "higher-cost",
        ExplainDisposition::NotSelectedTradeoff => "not-selected-tradeoff",
        ExplainDisposition::Selected => "selected",
        ExplainDisposition::CompilerFailure => "compiler-failure",
        ExplainDisposition::Truncated => "truncated",
    }
}

const fn subject_kind_name(kind: SubjectKind) -> &'static str {
    match kind {
        SubjectKind::SemanticProgram => "semantic-program",
        SubjectKind::Normalization => "normalization",
        SubjectKind::Region => "region",
        SubjectKind::Boundary => "boundary",
        SubjectKind::Candidate => "candidate",
        SubjectKind::Capability => "capability",
        SubjectKind::Schedule => "schedule",
        SubjectKind::Target => "target",
        SubjectKind::Kernel => "kernel",
        SubjectKind::KernelProgram => "kernel-program",
        SubjectKind::ArtifactPlan => "artifact-plan",
        SubjectKind::Alternative => "alternative",
    }
}

fn assessment_text(assessment: &Assessment) -> String {
    match assessment {
        Assessment::Proven => "proven".to_owned(),
        Assessment::Disproved(reason) => format!("disproved:{}", reason.as_str()),
        Assessment::Unknown(reason) => format!("unknown:{}", reason.as_str()),
        Assessment::Deferred(reason) => format!("deferred:{}", reason.as_str()),
    }
}

const fn basis_name(basis: &EvidenceBasis) -> &'static str {
    match basis {
        EvidenceBasis::NormativeGuarantee => "normative-guarantee",
        EvidenceBasis::CheckedInvariant => "checked-invariant",
        EvidenceBasis::SoundProof(_) => "sound-proof",
        EvidenceBasis::ExhaustiveFinite => "exhaustive-finite",
        EvidenceBasis::Empirical => "empirical",
        EvidenceBasis::Assumption => "assumption",
        EvidenceBasis::Unknown => "unknown",
    }
}

fn feasibility_text(outcome: &FeasibilityOutcome) -> String {
    match outcome {
        FeasibilityOutcome::Admitted => "admitted".to_owned(),
        FeasibilityOutcome::Rejected(reason) => format!("rejected:{}", reason.as_str()),
    }
}

const fn quantity_name(quantity: Quantity) -> &'static str {
    match quantity {
        Quantity::Count(_) => "count",
        Quantity::Bytes(_) => "bytes",
        Quantity::Threads(_) => "threads",
        Quantity::Bindings(_) => "bindings",
    }
}

const fn cost_disposition_name(disposition: CostDisposition) -> &'static str {
    match disposition {
        CostDisposition::Retained => "retained",
        CostDisposition::Dominated => "dominated",
        CostDisposition::HigherCost => "higher-cost",
    }
}

const fn selection_name(outcome: SelectionOutcome) -> &'static str {
    match outcome {
        SelectionOutcome::Selected => "selected",
        SelectionOutcome::Dominated => "dominated",
        SelectionOutcome::NotSelectedTradeoff => "not-selected-tradeoff",
        SelectionOutcome::Infeasible => "infeasible",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExplainError {
    InvalidKey {
        kind: KeyKind,
        bytes: usize,
    },
    InvalidLimits,
    InvalidTerminalLedger,
    TerminalLedgerCapacity,
    InvalidEventClass,
    BoundExceeded {
        bound: BoundKind,
        limit: u32,
        actual: u64,
    },
    EmptySubjects,
    CrossCompilationSubject,
    DuplicateCause,
    DuplicateSubject,
    DuplicateFact,
    DuplicateCostTerm,
    CrossWriterCause,
    InvalidCause {
        cause: ExplainRecordId,
        next: ExplainRecordId,
    },
    InvalidStageEvent,
    EvidenceEscalation,
    EvidenceSubjectMismatch,
    ProviderAuthorityMismatch,
    QuantityKindMismatch,
    InvalidQuantityRelation,
    UnknownQuantityUnit,
    EmptyCostEvidence,
    TerminalCapacity,
    EmptyTrace,
    StaleIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundKind {
    Subjects,
    Causes,
    Facts,
    CostTerms,
}

impl fmt::Display for ExplainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "explain: {self:?}")
    }
}

impl Error for ExplainError {}

fn validate_key(kind: KeyKind, value: &str) -> Result<(), ExplainError> {
    if value.is_empty()
        || value.len() > MAX_KEY_BYTES
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ExplainError::InvalidKey {
            kind,
            bytes: value.len(),
        });
    }
    Ok(())
}

fn check_bound(bound: BoundKind, limit: u32, actual: usize) -> Result<(), ExplainError> {
    let actual = u64::try_from(actual).unwrap_or(u64::MAX);
    if actual > u64::from(limit) {
        return Err(ExplainError::BoundExceeded {
            bound,
            limit,
            actual,
        });
    }
    Ok(())
}

fn check_terminal_ledger_bound(
    count: usize,
    byte_components: impl IntoIterator<Item = usize>,
) -> Result<(), ExplainError> {
    let bytes = byte_components
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
        .ok_or(ExplainError::TerminalLedgerCapacity)?;
    if count > usize::try_from(MAX_TERMINAL_LEDGER_RECORDS).unwrap_or(usize::MAX)
        || bytes > usize::try_from(MAX_TERMINAL_LEDGER_BYTES).unwrap_or(usize::MAX)
    {
        return Err(ExplainError::TerminalLedgerCapacity);
    }
    Ok(())
}

fn encode_trace(schema: u32, subject: &CompilationSubject, records: &[ExplainRecord]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.explain.trace.v1\0");
    bytes.extend_from_slice(&schema.to_be_bytes());
    encode_bytes(&mut bytes, &subject.canonical);
    bytes.extend_from_slice(
        &u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for record in records {
        bytes.extend_from_slice(&encode_record(record));
    }
    bytes
}

fn encode_record(record: &ExplainRecord) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&record.id.local.to_be_bytes());
    encode_bytes(&mut bytes, record.rule.key.as_str().as_bytes());
    bytes.extend_from_slice(&record.rule.revision.to_be_bytes());
    encode_bytes(&mut bytes, record.rule.provider.key.as_str().as_bytes());
    bytes.extend_from_slice(&record.rule.provider.revision.to_be_bytes());
    bytes.extend_from_slice(
        &u64::try_from(record.subjects.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for subject in &record.subjects {
        bytes.push(subject_kind_tag(subject.kind));
        encode_bytes(&mut bytes, subject.key.as_str().as_bytes());
    }
    encode_event(&mut bytes, &record.event);
    bytes.extend_from_slice(
        &u64::try_from(record.causes.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for cause in &record.causes {
        bytes.extend_from_slice(&cause.local.to_be_bytes());
    }
    bytes
}

fn encode_event(bytes: &mut Vec<u8>, event: &ExplainEvent) {
    match event {
        ExplainEvent::Check {
            stage,
            assessment,
            rejection,
        } => {
            bytes.extend_from_slice(&[
                1,
                stage_tag(*stage),
                match rejection {
                    RejectionClass::IntrinsicInvalid => 1,
                    RejectionClass::NumericalIllegal => 2,
                },
            ]);
            encode_assessment(bytes, assessment);
        }
        ExplainEvent::BudgetStop {
            stage,
            resource,
            limit,
            actual,
        } => {
            bytes.extend_from_slice(&[2, stage_tag(*stage)]);
            encode_bytes(bytes, resource.as_str().as_bytes());
            bytes.extend_from_slice(&limit.to_be_bytes());
            bytes.extend_from_slice(&actual.to_be_bytes());
        }
        ExplainEvent::Feasibility {
            predicate,
            outcome,
            required,
            available,
        } => {
            bytes.push(3);
            encode_bytes(bytes, predicate.as_str().as_bytes());
            match outcome {
                FeasibilityOutcome::Admitted => bytes.push(1),
                FeasibilityOutcome::Rejected(reason) => {
                    bytes.push(2);
                    encode_bytes(bytes, reason.as_str().as_bytes());
                }
            }
            encode_quantity(bytes, *required);
            encode_quantity(bytes, *available);
        }
        ExplainEvent::DeferredCapability { predicate, reason } => {
            bytes.push(4);
            encode_bytes(bytes, predicate.as_str().as_bytes());
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
        ExplainEvent::CostAssessment {
            model,
            basis,
            terms,
            disposition,
        } => {
            bytes.push(5);
            encode_bytes(bytes, model.as_str().as_bytes());
            encode_basis(bytes, basis);
            bytes.push(match disposition {
                CostDisposition::Retained => 1,
                CostDisposition::Dominated => 2,
                CostDisposition::HigherCost => 3,
            });
            bytes.extend_from_slice(&u64::try_from(terms.len()).unwrap_or(u64::MAX).to_be_bytes());
            for term in terms {
                encode_bytes(bytes, term.metric.as_str().as_bytes());
                encode_quantity(bytes, term.quantity);
            }
        }
        ExplainEvent::Selection { policy, outcome } => {
            bytes.push(6);
            encode_bytes(bytes, policy.as_str().as_bytes());
            bytes.push(match outcome {
                SelectionOutcome::Selected => 1,
                SelectionOutcome::Dominated => 2,
                SelectionOutcome::NotSelectedTradeoff => 3,
                SelectionOutcome::Infeasible => 4,
            });
        }
        ExplainEvent::CompilerFailure { stage, reason } => {
            bytes.extend_from_slice(&[7, stage_tag(*stage)]);
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
        ExplainEvent::CausalBridge { stage, disposition } => {
            bytes.extend_from_slice(&[8, stage_tag(*stage)]);
            bytes.push(disposition_tag(*disposition));
        }
        ExplainEvent::Truncated {
            omitted_records,
            omitted_bytes,
        } => {
            bytes.push(9);
            bytes.extend_from_slice(&omitted_records.to_be_bytes());
            bytes.extend_from_slice(&omitted_bytes.to_be_bytes());
        }
    }
}

fn encode_assessment(bytes: &mut Vec<u8>, assessment: &PredicateAssessment) {
    encode_bytes(bytes, assessment.predicate.as_str().as_bytes());
    match &assessment.assessment {
        Assessment::Proven => bytes.push(1),
        Assessment::Disproved(reason) => {
            bytes.push(2);
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
        Assessment::Unknown(reason) => {
            bytes.push(3);
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
        Assessment::Deferred(reason) => {
            bytes.push(4);
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
    }
    encode_basis(bytes, &assessment.basis);
    bytes.extend_from_slice(
        &u64::try_from(assessment.facts.len())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for fact in &assessment.facts {
        encode_bytes(bytes, fact.key.as_str().as_bytes());
        match &fact.value {
            FactValue::Count(value) => {
                bytes.push(1);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            FactValue::Bytes(value) => {
                bytes.push(2);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            FactValue::Threads(value) => {
                bytes.push(3);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            FactValue::Bindings(value) => {
                bytes.push(4);
                bytes.extend_from_slice(&value.to_be_bytes());
            }
            FactValue::Boolean(value) => {
                bytes.extend_from_slice(&[5, u8::from(*value)]);
            }
            FactValue::Identity(value) => {
                bytes.push(6);
                encode_bytes(bytes, value.as_str().as_bytes());
            }
        }
    }
}

fn encode_basis(bytes: &mut Vec<u8>, basis: &EvidenceBasis) {
    bytes.push(match basis {
        EvidenceBasis::NormativeGuarantee => 1,
        EvidenceBasis::CheckedInvariant => 2,
        EvidenceBasis::SoundProof(receipt) => {
            bytes.push(3);
            bytes.push(match receipt.kind {
                EvidenceReceiptKind::FusionNumerical => 1,
            });
            encode_bytes(bytes, &receipt.compilation);
            encode_bytes(bytes, receipt.candidate.as_str().as_bytes());
            encode_bytes(bytes, receipt.provider.key.as_str().as_bytes());
            bytes.extend_from_slice(&receipt.provider.revision.to_be_bytes());
            encode_bytes(bytes, &receipt.proof);
            return;
        }
        EvidenceBasis::ExhaustiveFinite => 4,
        EvidenceBasis::Empirical => 5,
        EvidenceBasis::Assumption => 6,
        EvidenceBasis::Unknown => 7,
    });
}

fn encode_quantity(bytes: &mut Vec<u8>, quantity: Quantity) {
    bytes.push(quantity.kind());
    bytes.extend_from_slice(&quantity.value().to_be_bytes());
}

fn encode_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    bytes.extend_from_slice(value);
}

fn stable_qualifier(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

const fn subject_kind_tag(kind: SubjectKind) -> u8 {
    match kind {
        SubjectKind::SemanticProgram => 1,
        SubjectKind::Normalization => 2,
        SubjectKind::Region => 3,
        SubjectKind::Boundary => 4,
        SubjectKind::Candidate => 5,
        SubjectKind::Capability => 6,
        SubjectKind::Schedule => 7,
        SubjectKind::Target => 8,
        SubjectKind::Kernel => 9,
        SubjectKind::KernelProgram => 10,
        SubjectKind::ArtifactPlan => 11,
        SubjectKind::Alternative => 12,
    }
}

const fn stage_tag(stage: ExplainStage) -> u8 {
    match stage {
        ExplainStage::RequestVerification => 1,
        ExplainStage::Normalization => 2,
        ExplainStage::RegionFormation => 3,
        ExplainStage::CandidateEnumeration => 4,
        ExplainStage::CapabilityResolution => 5,
        ExplainStage::NumericalLegality => 6,
        ExplainStage::IntrinsicScheduling => 7,
        ExplainStage::TargetFeasibility => 8,
        ExplainStage::Costing => 9,
        ExplainStage::Selection => 10,
        ExplainStage::KernelRefinement => 11,
        ExplainStage::ProgramVerification => 12,
        ExplainStage::ArtifactPlanning => 13,
    }
}

const fn disposition_tag(disposition: ExplainDisposition) -> u8 {
    match disposition {
        ExplainDisposition::Admitted => 1,
        ExplainDisposition::RejectedIntrinsic => 2,
        ExplainDisposition::RejectedNumerical => 3,
        ExplainDisposition::RejectedTarget => 4,
        ExplainDisposition::DeferredUnsupported => 5,
        ExplainDisposition::BudgetStopped => 6,
        ExplainDisposition::Retained => 7,
        ExplainDisposition::DominancePruned => 8,
        ExplainDisposition::HigherCost => 9,
        ExplainDisposition::NotSelectedTradeoff => 10,
        ExplainDisposition::Selected => 11,
        ExplainDisposition::CompilerFailure => 12,
        ExplainDisposition::Truncated => 13,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::{CandidateKind, enumerate_candidates, prove_fused_numerics};
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn request(scale: f32) -> VerifiedTargetRequest {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, scale.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let output = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), output)
            .unwrap();
        let program = builder.build().unwrap();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        verified.for_target(verified.target_profiles()[0]).unwrap()
    }

    fn admitted(writer: &ExplainWriter, key: &str) -> ExplainRecordParts {
        ExplainRecordParts {
            rule: RuleRef::builtin("test.rule").unwrap(),
            subjects: vec![writer.subject(SubjectKind::Candidate, key).unwrap()],
            event: ExplainEvent::Check {
                stage: ExplainStage::CandidateEnumeration,
                assessment: PredicateAssessment::proven(
                    "candidate.legal",
                    EvidenceBasis::CheckedInvariant,
                )
                .unwrap(),
                rejection: RejectionClass::IntrinsicInvalid,
            },
            causes: Vec::new(),
        }
    }

    struct ExplainRecordParts {
        rule: RuleRef,
        subjects: Vec<SubjectRef>,
        event: ExplainEvent,
        causes: Vec<ExplainRecordId>,
    }

    fn finish_test_trace(mut writer: ExplainWriter) -> VerifiedExplainTrace {
        let subject = writer
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        writer
            .note_selection(subject, SelectionOutcome::Selected, None)
            .unwrap();
        writer
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap()
    }

    #[test]
    fn deterministic_trace_is_sealed_and_rendered_separately() {
        let request = request(2.0);
        let mut first = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let parts = admitted(&first, "candidate:a");
        first
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap();
        let detail_only = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        assert_eq!(
            detail_only.finish_success(&["alternative:test"], "alternative:test"),
            Err(ExplainError::InvalidTerminalLedger)
        );
        let trace = finish_test_trace(first);
        assert!(trace.verify().is_ok());
        assert_eq!(
            trace.render(),
            concat!(
                "tiler-explain-v2 request=9d93c7444c678abd\n",
                "0 candidate-enumeration admitted rule=test.rule@1 provider=tiler.compiler@1 subject=candidate:candidate:a event=check:candidate.legal:proven:checked-invariant causes=-\n",
                "1 selection selected rule=tiler.selection.structural-pareto.v1@1 provider=tiler.compiler@1 subject=alternative:alternative:test event=selection:tiler.selection.structural-pareto.v1:selected causes=-\n",
            )
        );
        assert!(!trace.identity().0.is_empty());
    }

    #[test]
    fn cross_request_subjects_causes_and_units_fail_closed() {
        let first_request = request(2.0);
        let second_request = request(3.0);
        let mut first = ExplainWriter::new(&first_request, ExplainLimits::default()).unwrap();
        let second = ExplainWriter::new(&second_request, ExplainLimits::default()).unwrap();
        let foreign = second.subject(SubjectKind::Region, "region:0").unwrap();
        assert_eq!(
            first.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![foreign],
                ExplainEvent::Check {
                    stage: ExplainStage::RegionFormation,
                    assessment: PredicateAssessment::proven(
                        "region.legal",
                        EvidenceBasis::CheckedInvariant
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid
                },
                Vec::new()
            ),
            Err(ExplainError::CrossCompilationSubject)
        );
        let own = first.subject(SubjectKind::Region, "region:0").unwrap();
        assert!(matches!(
            first.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![own.clone()],
                ExplainEvent::Check {
                    stage: ExplainStage::RegionFormation,
                    assessment: PredicateAssessment::proven(
                        "region.legal",
                        EvidenceBasis::CheckedInvariant
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid
                },
                vec![ExplainRecordId {
                    local: 0,
                    writer_authority: first.authority,
                    request_qualifier: first.request_qualifier,
                }]
            ),
            Err(ExplainError::InvalidCause { .. })
        ));
        assert_eq!(
            first.push_detail(
                RuleRef::builtin("target.limit").unwrap(),
                vec![own],
                ExplainEvent::Feasibility {
                    predicate: PredicateKey::new("grid-axis").unwrap(),
                    outcome: FeasibilityOutcome::Rejected(
                        ReasonCode::new("target-limit").unwrap(),
                    ),
                    required: Quantity::Threads(2),
                    available: Quantity::Bytes(2)
                },
                Vec::new()
            ),
            Err(ExplainError::QuantityKindMismatch)
        );

        let parts = admitted(&first, "candidate:first");
        let first_cause = first
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap()
            .unwrap();
        let mut same_request =
            ExplainWriter::new(&first_request, ExplainLimits::default()).unwrap();
        let parts = admitted(&same_request, "candidate:other-writer-root");
        same_request
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap();
        let parts = admitted(&same_request, "candidate:other-writer-child");
        assert_eq!(
            same_request.push_detail(parts.rule, parts.subjects, parts.event, vec![first_cause]),
            Err(ExplainError::CrossWriterCause)
        );

        let parts = admitted(&first, "candidate:foreign-provider");
        assert_eq!(
            first.push_detail(
                RuleRef::provided(
                    "foreign.rule",
                    1,
                    ProviderRef {
                        key: ProviderKey::new("foreign.provider").unwrap(),
                        revision: 1,
                    },
                )
                .unwrap(),
                parts.subjects,
                parts.event,
                Vec::new(),
            ),
            Err(ExplainError::ProviderAuthorityMismatch)
        );
    }

    #[test]
    fn invalid_stage_and_evidence_escalation_fail_closed() {
        assert_eq!(
            PredicateAssessment::proven("unknown", EvidenceBasis::Unknown),
            Err(ExplainError::EvidenceEscalation)
        );
        assert_eq!(
            PredicateAssessment::proven("measured", EvidenceBasis::Empirical),
            Err(ExplainError::EvidenceEscalation)
        );
        assert_eq!(
            PredicateAssessment::proven("assumed", EvidenceBasis::Assumption),
            Err(ExplainError::EvidenceEscalation)
        );
        assert_eq!(
            PredicateAssessment::disproved(
                "assumed-false",
                ReasonCode::new("assumption").unwrap(),
                EvidenceBasis::Assumption,
            ),
            Err(ExplainError::EvidenceEscalation)
        );
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let subject = writer
            .subject(SubjectKind::Candidate, "candidate:a")
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::Selection,
                    assessment: PredicateAssessment::proven(
                        "selected",
                        EvidenceBasis::CheckedInvariant
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid
                },
                Vec::new()
            ),
            Err(ExplainError::InvalidStageEvent)
        );
        let alternative = writer
            .subject(SubjectKind::Alternative, "alternative:invalid-detail")
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::builtin("selection.invalid").unwrap(),
                vec![alternative],
                ExplainEvent::Selection {
                    policy: SelectionPolicyKey::new("selection.invalid").unwrap(),
                    outcome: SelectionOutcome::Selected,
                },
                Vec::new(),
            ),
            Err(ExplainError::InvalidEventClass)
        );
        let candidate = writer
            .subject(SubjectKind::Candidate, "candidate:invalid-terminal")
            .unwrap();
        assert_eq!(
            writer.push_terminal(
                RuleRef::builtin("check.invalid").unwrap(),
                vec![candidate],
                ExplainEvent::Check {
                    stage: ExplainStage::CandidateEnumeration,
                    assessment: PredicateAssessment::proven(
                        "candidate.legal",
                        EvidenceBasis::CheckedInvariant,
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                Vec::new(),
            ),
            Err(ExplainError::InvalidEventClass)
        );
        let subject = writer
            .subject(SubjectKind::Candidate, "candidate:b")
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::Normalization,
                    assessment: PredicateAssessment::disproved(
                        "normalization.invalid",
                        ReasonCode::new("invalid").unwrap(),
                        EvidenceBasis::CheckedInvariant,
                    )
                    .unwrap(),
                    rejection: RejectionClass::NumericalIllegal,
                },
                Vec::new(),
            ),
            Err(ExplainError::InvalidStageEvent)
        );
    }

    #[test]
    fn sound_proof_receipts_are_bound_to_request_candidate_and_provider() {
        let first_request = request(2.0);
        let second_request = request(3.0);
        let candidates = enumerate_candidates(&first_request).unwrap();
        let candidate = candidates
            .iter()
            .find(|candidate| candidate.kind == CandidateKind::FusedSerialSum)
            .unwrap();
        let proof = prove_fused_numerics(&first_request, candidate).unwrap();
        let provider =
            ProviderRef::lowering(first_request.capabilities().fused_serial_sum.unwrap()).unwrap();
        let receipt =
            VerifiedEvidenceRef::from_fusion_numerical(&first_request, &proof, provider.clone())
                .unwrap();
        let mut writer = ExplainWriter::new(&second_request, ExplainLimits::default()).unwrap();
        let subject = writer
            .subject(SubjectKind::Candidate, candidate.stable_id.as_str())
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::provided("fusion.strict-f32-equivalence", 1, provider).unwrap(),
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::NumericalLegality,
                    assessment: PredicateAssessment::proven(
                        "fusion.strict-f32-equivalence",
                        EvidenceBasis::SoundProof(receipt),
                    )
                    .unwrap(),
                    rejection: RejectionClass::NumericalIllegal,
                },
                Vec::new(),
            ),
            Err(ExplainError::EvidenceSubjectMismatch)
        );
        let invalid_cost_receipt = VerifiedEvidenceRef::from_fusion_numerical(
            &first_request,
            &proof,
            ProviderRef::lowering(first_request.capabilities().fused_serial_sum.unwrap()).unwrap(),
        )
        .unwrap();
        let subject = writer
            .subject(SubjectKind::Alternative, "alternative:invalid-cost-proof")
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::builtin("cost.invalid-proof").unwrap(),
                vec![subject],
                ExplainEvent::CostAssessment {
                    model: CostModelKey::new("cost.invalid-proof").unwrap(),
                    basis: EvidenceBasis::SoundProof(invalid_cost_receipt),
                    terms: vec![CostTerm::new("dispatches", Quantity::Count(1)).unwrap()],
                    disposition: CostDisposition::Retained,
                },
                Vec::new(),
            ),
            Err(ExplainError::EvidenceEscalation)
        );
    }

    #[test]
    fn bounds_truncate_details_but_retain_terminal_selection() {
        let request = request(2.0);
        let mut writer =
            ExplainWriter::new(&request, ExplainLimits::new(1, 64 * 1024).unwrap()).unwrap();
        for key in ["candidate:a", "candidate:b", "candidate:c"] {
            let parts = admitted(&writer, key);
            writer
                .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
                .unwrap();
        }
        let rejected = writer
            .subject(SubjectKind::Alternative, "alternative:baseline")
            .unwrap();
        writer
            .note_selection(rejected, SelectionOutcome::Dominated, None)
            .unwrap();
        let terminal = writer
            .subject(SubjectKind::Alternative, "alternative:a")
            .unwrap();
        writer
            .note_selection(terminal, SelectionOutcome::Selected, None)
            .unwrap();
        let trace = writer
            .finish_success(&["alternative:baseline", "alternative:a"], "alternative:a")
            .unwrap();
        assert!(trace.records().iter().any(|record| matches!(
            record.event(),
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            }
        )));
        assert!(trace.records().iter().any(|record| matches!(
            record.event(),
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Dominated,
                ..
            }
        )));
        assert!(
            trace
                .records()
                .iter()
                .any(|record| matches!(record.event(), ExplainEvent::Truncated { .. }))
        );
    }

    #[test]
    fn truncation_is_a_sibling_and_omitted_terminal_causes_keep_exact_authority() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(
            &request,
            ExplainLimits::new(1, MAX_CANONICAL_BYTES).unwrap(),
        )
        .unwrap();
        let predecessor_parts = admitted(&writer, "candidate:predecessor");
        let predecessor = writer
            .push_detail(
                predecessor_parts.rule,
                predecessor_parts.subjects,
                predecessor_parts.event,
                predecessor_parts.causes,
            )
            .unwrap()
            .unwrap();
        let subject = writer
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        let cause = writer
            .push_causal_detail(
                RuleRef::builtin("cost.exact-cause").unwrap(),
                subject.clone(),
                &ExplainEvent::CostAssessment {
                    model: CostModelKey::new("cost.exact-cause").unwrap(),
                    basis: EvidenceBasis::CheckedInvariant,
                    terms: vec![CostTerm::new("dispatches", Quantity::Count(1)).unwrap()],
                    disposition: CostDisposition::Retained,
                },
                vec![predecessor],
            )
            .unwrap();
        assert!(matches!(cause.kind, TerminalCauseKind::Omitted { .. }));
        writer
            .note_selection(subject, SelectionOutcome::Selected, Some(cause))
            .unwrap();
        let trace = writer
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();
        let truncation = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::Truncated { .. }))
            .unwrap();
        assert!(truncation.causes().is_empty());
        let bridge = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CausalBridge { .. }))
            .unwrap();
        assert_eq!(bridge.rule().key().as_str(), "cost.exact-cause");
        assert_eq!(bridge.subjects()[0].key().as_str(), "alternative:test");
        let selection = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::Selection { .. }))
            .unwrap();
        assert_eq!(selection.causes(), &[bridge.id()]);
        assert_ne!(selection.causes(), &[truncation.id()]);
        assert_eq!(bridge.causes(), &[predecessor]);
    }

    #[test]
    fn failure_trace_has_one_terminal_failure_and_survives_zero_detail_limits() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::new(0, 0).unwrap()).unwrap();
        let parts = admitted(&writer, "candidate:omitted");
        let cause = writer
            .push_causal_detail(
                parts.rule,
                parts.subjects.into_iter().next().unwrap(),
                &parts.event,
                parts.causes,
            )
            .unwrap();
        let trace = writer
            .finish_failure(
                FailureDescriptor::new(
                    ExplainStage::KernelRefinement,
                    "invalid-compiler-output",
                    SubjectKind::Kernel,
                    "failed-kernel",
                    Some(cause),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(
            trace
                .records()
                .iter()
                .filter(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
                .count(),
            1
        );
        assert!(trace.records().iter().any(|record| matches!(
            record.event(),
            ExplainEvent::Truncated {
                omitted_records: 1,
                ..
            }
        )));
        let bridge = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CausalBridge { .. }))
            .unwrap();
        let failure = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .unwrap();
        assert_eq!(failure.causes(), &[bridge.id()]);
        assert!(matches!(
            bridge.event(),
            ExplainEvent::CausalBridge {
                stage: ExplainStage::CandidateEnumeration,
                disposition: ExplainDisposition::Admitted,
            }
        ));
        assert_eq!(bridge.rule().key.as_str(), "test.rule");
        assert!(matches!(
            bridge.subjects(),
            [subject]
                if subject.kind == SubjectKind::Candidate
                    && subject.key.as_str() == "candidate:omitted"
        ));
    }

    #[test]
    fn terminal_ledger_rejects_duplicates_unknowns_and_max_detail_pressure() {
        let request = request(2.0);
        let mut duplicate = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let subject = duplicate
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        duplicate
            .note_selection(subject.clone(), SelectionOutcome::Selected, None)
            .unwrap();
        assert_eq!(
            duplicate.note_selection(subject.clone(), SelectionOutcome::Dominated, None),
            Err(ExplainError::InvalidTerminalLedger)
        );
        assert!(
            duplicate
                .finish_success(&["alternative:test"], "alternative:test")
                .is_ok()
        );

        let mut infeasible = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        assert_eq!(
            infeasible.note_selection(subject.clone(), SelectionOutcome::Infeasible, None),
            Err(ExplainError::InvalidTerminalLedger)
        );
        infeasible
            .note_infeasible_alternative(subject, None)
            .unwrap();

        let mut unknown = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let subject = unknown
            .subject(SubjectKind::Alternative, "alternative:unknown")
            .unwrap();
        unknown
            .note_selection(subject, SelectionOutcome::Selected, None)
            .unwrap();
        assert_eq!(
            unknown.finish_success(&["alternative:test"], "alternative:test"),
            Err(ExplainError::InvalidTerminalLedger)
        );

        let mut pressured = ExplainWriter::new(
            &request,
            ExplainLimits::new(MAX_RECORDS, MAX_CANONICAL_BYTES).unwrap(),
        )
        .unwrap();
        for index in 0..MAX_RECORDS {
            let key = format!("candidate:{index}");
            let parts = admitted(&pressured, &key);
            pressured
                .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
                .unwrap();
        }
        let subject = pressured
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        pressured
            .note_selection(subject, SelectionOutcome::Selected, None)
            .unwrap();
        let trace = pressured
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();
        assert!(trace.records().iter().any(|record| matches!(
            record.event(),
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            }
        )));

        let mut bounded = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        for index in 0..MAX_TERMINAL_LEDGER_RECORDS {
            let subject = bounded
                .subject(SubjectKind::Alternative, format!("alternative:{index}"))
                .unwrap();
            bounded
                .note_selection(subject, SelectionOutcome::Dominated, None)
                .unwrap();
        }
        let excess = bounded
            .subject(SubjectKind::Alternative, "alternative:excess")
            .unwrap();
        assert_eq!(
            bounded.note_selection(excess, SelectionOutcome::Dominated, None),
            Err(ExplainError::TerminalLedgerCapacity)
        );

        let mut alternatives = vec!["alternative:test"; MAX_TERMINAL_LEDGER_RECORDS as usize + 1];
        alternatives[0] = "alternative:selected";
        let mut slice_bounded = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let selected = slice_bounded
            .subject(SubjectKind::Alternative, "alternative:selected")
            .unwrap();
        slice_bounded
            .note_selection(selected, SelectionOutcome::Selected, None)
            .unwrap();
        assert_eq!(
            slice_bounded.finish_success(&alternatives, "alternative:selected"),
            Err(ExplainError::TerminalLedgerCapacity)
        );
    }

    #[test]
    fn feasibility_and_budget_events_enforce_numeric_truth() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let subject = writer.subject(SubjectKind::Region, "region:0").unwrap();
        for (outcome, required, available) in [
            (FeasibilityOutcome::Admitted, 2, 1),
            (
                FeasibilityOutcome::Rejected(ReasonCode::new("too-large").unwrap()),
                1,
                1,
            ),
        ] {
            assert_eq!(
                writer.push_detail(
                    RuleRef::builtin("target.grid-axis").unwrap(),
                    vec![subject.clone()],
                    ExplainEvent::Feasibility {
                        predicate: PredicateKey::new("grid-axis").unwrap(),
                        outcome,
                        required: Quantity::Threads(required),
                        available: Quantity::Threads(available),
                    },
                    Vec::new(),
                ),
                Err(ExplainError::InvalidQuantityRelation)
            );
        }
        assert!(
            writer
                .push_detail(
                    RuleRef::builtin("target.grid-axis").unwrap(),
                    vec![subject.clone()],
                    ExplainEvent::Feasibility {
                        predicate: PredicateKey::new("grid-axis").unwrap(),
                        outcome: FeasibilityOutcome::Admitted,
                        required: Quantity::Threads(1),
                        available: Quantity::Threads(1),
                    },
                    Vec::new(),
                )
                .is_ok()
        );
        for (limit, actual) in [(1, 1), (2, 1)] {
            assert_eq!(
                writer.push_detail(
                    RuleRef::builtin("budget.test").unwrap(),
                    vec![subject.clone()],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::CandidateEnumeration,
                        resource: ResourceKey::new("candidates").unwrap(),
                        limit,
                        actual,
                    },
                    Vec::new(),
                ),
                Err(ExplainError::InvalidQuantityRelation)
            );
        }
    }

    #[test]
    fn maximum_terminal_ledger_with_omitted_causes_seals_within_hard_trace_bounds() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::new(0, 0).unwrap()).unwrap();
        let keys = (0..MAX_TERMINAL_LEDGER_RECORDS)
            .map(|index| format!("alternative:{index}"))
            .collect::<Vec<_>>();
        for (index, key) in keys.iter().enumerate() {
            let subject = writer.subject(SubjectKind::Alternative, key).unwrap();
            let cause = writer
                .push_causal_detail(
                    RuleRef::builtin("cost.maximum-ledger").unwrap(),
                    subject.clone(),
                    &ExplainEvent::CostAssessment {
                        model: CostModelKey::new("cost.maximum-ledger").unwrap(),
                        basis: EvidenceBasis::CheckedInvariant,
                        terms: vec![CostTerm::new("dispatches", Quantity::Count(1)).unwrap()],
                        disposition: CostDisposition::Retained,
                    },
                    Vec::new(),
                )
                .unwrap();
            writer
                .note_selection(
                    subject,
                    if index == 0 {
                        SelectionOutcome::Selected
                    } else {
                        SelectionOutcome::Dominated
                    },
                    Some(cause),
                )
                .unwrap();
        }
        let alternatives = keys.iter().map(String::as_str).collect::<Vec<_>>();
        let trace = writer.finish_success(&alternatives, &keys[0]).unwrap();
        assert_eq!(
            trace.records().len(),
            usize::try_from(MAX_TERMINAL_LEDGER_RECORDS * 2 + 1).unwrap()
        );
        assert!(
            trace.identity().as_bytes().len()
                <= usize::try_from(MAX_TRACE_CANONICAL_BYTES).unwrap()
        );
        assert!(trace.verify().is_ok());
    }

    #[test]
    fn stale_identity_and_reordered_records_are_rejected() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let parts = admitted(&writer, "candidate:a");
        let first = writer
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap()
            .unwrap();
        let mut parts = admitted(&writer, "candidate:b");
        parts.causes.push(first);
        writer
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap();
        let trace = finish_test_trace(writer);

        let mut stale_digest = trace.clone();
        stale_digest.canonical_identity.0[0] ^= 1;
        assert_eq!(stale_digest.verify(), Err(ExplainError::StaleIdentity));

        let mut changed_provider = trace.clone();
        changed_provider.records[0].rule.provider.revision += 1;
        assert_eq!(changed_provider.verify(), Err(ExplainError::StaleIdentity));

        let mut changed_reason = trace.clone();
        let ExplainEvent::Check { assessment, .. } = &mut changed_reason.records[0].event else {
            panic!("fixture uses a check event");
        };
        assessment.assessment = Assessment::Disproved(ReasonCode::new("changed-reason").unwrap());
        assert_eq!(changed_reason.verify(), Err(ExplainError::StaleIdentity));

        let mut reordered = trace.clone();
        reordered.records.swap(0, 1);
        assert_eq!(reordered.verify(), Err(ExplainError::StaleIdentity));

        let mut duplicate_cause = trace;
        duplicate_cause.records[1].causes = vec![first, first];
        duplicate_cause.canonical_identity = ExplainIdentity(
            encode_trace(
                duplicate_cause.schema_version,
                &duplicate_cause.compilation_subject,
                &duplicate_cause.records,
            )
            .into_boxed_slice(),
        );
        assert_eq!(duplicate_cause.verify(), Err(ExplainError::StaleIdentity));
    }

    #[test]
    fn keys_and_rendered_reasons_are_typed_and_bounded() {
        assert!(matches!(
            RuleKey::new("contains whitespace"),
            Err(ExplainError::InvalidKey {
                kind: KeyKind::Rule,
                ..
            })
        ));
        assert!(matches!(
            ReasonCode::new("x".repeat(MAX_KEY_BYTES + 1)),
            Err(ExplainError::InvalidKey {
                kind: KeyKind::Reason,
                ..
            })
        ));

        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let subject = writer
            .subject(SubjectKind::Candidate, "candidate:a")
            .unwrap();
        let assessment = PredicateAssessment::disproved(
            "candidate.legal",
            ReasonCode::new("shape-mismatch").unwrap(),
            EvidenceBasis::CheckedInvariant,
        )
        .unwrap()
        .with_fact(ExplainFact::new("rank", FactValue::Count(3)).unwrap())
        .unwrap();
        writer
            .push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![subject],
                ExplainEvent::Check {
                    stage: ExplainStage::CandidateEnumeration,
                    assessment,
                    rejection: RejectionClass::IntrinsicInvalid,
                },
                Vec::new(),
            )
            .unwrap();
        let rendered = finish_test_trace(writer).render();
        assert!(rendered.contains("disproved:shape-mismatch:checked-invariant:facts=rank:count=3"));
    }
}
