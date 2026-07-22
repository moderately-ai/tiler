#![allow(
    dead_code,
    reason = "private draft reserves reviewed stage/evidence views before the public facade"
)]

use std::error::Error;
use std::fmt;

use crate::fusion::FusionNumericalProof;
use crate::request::{LoweringProviderIdentity, VerifiedRequestSubject, VerifiedTargetRequest};

pub(crate) const EXPLAIN_SCHEMA_VERSION: u32 = 1;
pub(crate) const EXPLAIN_RENDERER_VERSION: u32 = 1;
const MAX_KEY_BYTES: usize = 255;
const MAX_RECORDS: u32 = 4_096;
const MAX_CANONICAL_BYTES: u32 = 1024 * 1024;
const MAX_SUBJECTS_PER_RECORD: u32 = 16;
const MAX_CAUSES_PER_RECORD: u32 = 16;
const MAX_FACTS_PER_ASSESSMENT: u32 = 32;
const MAX_COST_TERMS: u32 = 32;

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
    request: VerifiedRequestSubject,
    canonical: Box<[u8]>,
}

impl CompilationSubject {
    pub(crate) fn from_request(request: &VerifiedTargetRequest) -> Self {
        let request = request.subject();
        let canonical = request.canonical_explain_subject_bytes().into_boxed_slice();
        Self { request, canonical }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceBasis {
    NormativeGuarantee,
    CheckedInvariant,
    SoundProof(VerifiedEvidenceRef),
    ExhaustiveFinite,
    Empirical,
    Assumption,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedEvidenceRef(EvidenceReceiptKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceReceiptKind {
    FusionNumerical,
}

impl VerifiedEvidenceRef {
    pub(crate) const fn from_fusion_numerical(_: &FusionNumericalProof) -> Self {
        Self(EvidenceReceiptKind::FusionNumerical)
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
        if matches!(basis, EvidenceBasis::Unknown) {
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
        if matches!(basis, EvidenceBasis::Unknown) {
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
    HigherCost,
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
            | Self::CompilerFailure { stage, .. } => *stage,
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
            }
            | Self::Selection {
                outcome: SelectionOutcome::HigherCost,
                ..
            } => ExplainDisposition::HigherCost,
            Self::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            } => ExplainDisposition::Selected,
            Self::CompilerFailure { .. } => ExplainDisposition::CompilerFailure,
            Self::Truncated { .. } => ExplainDisposition::Truncated,
        }
    }

    fn validate(&self) -> Result<(), ExplainError> {
        match self {
            Self::Check {
                stage, assessment, ..
            } => {
                if matches!(
                    stage,
                    ExplainStage::TargetFeasibility
                        | ExplainStage::Costing
                        | ExplainStage::Selection
                        | ExplainStage::CapabilityResolution
                ) || matches!(assessment.basis, EvidenceBasis::SoundProof(_))
                    && *stage != ExplainStage::NumericalLegality
                {
                    return Err(ExplainError::InvalidStageEvent);
                }
                check_bound(
                    BoundKind::Facts,
                    MAX_FACTS_PER_ASSESSMENT,
                    assessment.facts.len(),
                )?;
            }
            Self::Feasibility {
                required,
                available,
                ..
            } if required.kind() != available.kind() => {
                return Err(ExplainError::QuantityKindMismatch);
            }
            Self::CostAssessment { terms, .. } => {
                if terms.is_empty() {
                    return Err(ExplainError::EmptyCostEvidence);
                }
                check_bound(BoundKind::CostTerms, MAX_COST_TERMS, terms.len())?;
            }
            Self::BudgetStop { .. }
            | Self::Feasibility { .. }
            | Self::DeferredCapability { .. }
            | Self::Selection { .. }
            | Self::CompilerFailure { .. }
            | Self::Truncated { .. } => {}
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ExplainRecordId(u32);

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
        if max_records < 2
            || max_records > MAX_RECORDS
            || max_canonical_bytes < 1_024
            || max_canonical_bytes > MAX_CANONICAL_BYTES
        {
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

#[derive(Clone, Debug)]
pub(crate) struct ExplainWriter {
    subject: CompilationSubject,
    limits: ExplainLimits,
    records: Vec<ExplainRecord>,
    retained_bytes: usize,
    omitted_records: u64,
    omitted_bytes: u64,
}

impl ExplainWriter {
    pub(crate) fn new(
        request: &VerifiedTargetRequest,
        limits: ExplainLimits,
    ) -> Result<Self, ExplainError> {
        let subject = CompilationSubject::from_request(request);
        let retained_bytes = encode_trace(EXPLAIN_SCHEMA_VERSION, &subject, &[]).len();
        if retained_bytes > usize::try_from(limits.max_canonical_bytes).unwrap_or(usize::MAX) {
            return Err(ExplainError::TerminalCapacity);
        }
        Ok(Self {
            subject,
            limits,
            records: Vec::new(),
            retained_bytes,
            omitted_records: 0,
            omitted_bytes: 0,
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
        self.push(rule, subjects, event, causes, false)
    }

    pub(crate) fn push_terminal(
        &mut self,
        rule: RuleRef,
        subjects: Vec<SubjectRef>,
        event: ExplainEvent,
        causes: Vec<ExplainRecordId>,
    ) -> Result<ExplainRecordId, ExplainError> {
        self.push(rule, subjects, event, causes, true)?
            .ok_or(ExplainError::TerminalCapacity)
    }

    fn push(
        &mut self,
        rule: RuleRef,
        subjects: Vec<SubjectRef>,
        event: ExplainEvent,
        causes: Vec<ExplainRecordId>,
        terminal: bool,
    ) -> Result<Option<ExplainRecordId>, ExplainError> {
        event.validate()?;
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
        let mut unique = causes.clone();
        unique.sort_unstable();
        if unique.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ExplainError::DuplicateCause);
        }
        let next = ExplainRecordId(
            u32::try_from(self.records.len()).map_err(|_| ExplainError::TerminalCapacity)?,
        );
        if causes.iter().any(|cause| cause.0 >= next.0) {
            return Err(ExplainError::InvalidCause {
                cause: *causes
                    .iter()
                    .find(|cause| cause.0 >= next.0)
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
        // Current compilation has at most two alternatives, each needing a
        // terminal selection record, plus a possible truncation record.
        let record_reserve = if terminal { 0 } else { 3 };
        let byte_reserve = if terminal { 0 } else { 1_024 };
        let exceeds = self.records.len().saturating_add(1 + record_reserve)
            > usize::try_from(self.limits.max_records).unwrap_or(usize::MAX)
            || self.retained_bytes.saturating_add(bytes + byte_reserve)
                > usize::try_from(self.limits.max_canonical_bytes).unwrap_or(usize::MAX);
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
        self.records.push(record);
        Ok(Some(next))
    }

    pub(crate) fn finish(mut self) -> Result<VerifiedExplainTrace, ExplainError> {
        if self.omitted_records != 0 {
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
        }
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
        let mut output = format!("tiler-explain-v{EXPLAIN_RENDERER_VERSION}\n");
        for record in &self.records {
            use fmt::Write as _;
            let _ = write!(
                output,
                "{} {} {} rule={}@{} provider={}@{} subject=",
                record.id.0,
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
                    let _ = write!(output, "{}", cause.0);
                }
            }
            output.push('\n');
        }
        output
    }

    #[cfg(test)]
    fn verify(&self) -> Result<(), ExplainError> {
        if self.records.is_empty()
            || self.records.len() > usize::try_from(MAX_RECORDS).unwrap_or(usize::MAX)
            || self.canonical_identity.0.len()
                > usize::try_from(MAX_CANONICAL_BYTES).unwrap_or(usize::MAX)
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
            if record.id.0 != u32::try_from(index).unwrap_or(u32::MAX)
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
                    .any(|cause| usize::try_from(cause.0).map_or(true, |cause| cause >= index))
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
                basis_name(assessment.basis)
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
                basis_name(*basis),
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

const fn basis_name(basis: EvidenceBasis) -> &'static str {
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
        SelectionOutcome::HigherCost => "higher-cost",
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExplainError {
    InvalidKey {
        kind: KeyKind,
        bytes: usize,
    },
    InvalidLimits,
    BoundExceeded {
        bound: BoundKind,
        limit: u32,
        actual: u64,
    },
    EmptySubjects,
    CrossCompilationSubject,
    DuplicateCause,
    InvalidCause {
        cause: ExplainRecordId,
        next: ExplainRecordId,
    },
    InvalidStageEvent,
    EvidenceEscalation,
    QuantityKindMismatch,
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
    bytes.extend_from_slice(&record.id.0.to_be_bytes());
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
        bytes.extend_from_slice(&cause.0.to_be_bytes());
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
            encode_basis(bytes, *basis);
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
                SelectionOutcome::HigherCost => 3,
            });
        }
        ExplainEvent::CompilerFailure { stage, reason } => {
            bytes.extend_from_slice(&[7, stage_tag(*stage)]);
            encode_bytes(bytes, reason.as_str().as_bytes());
        }
        ExplainEvent::Truncated {
            omitted_records,
            omitted_bytes,
        } => {
            bytes.push(8);
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
    encode_basis(bytes, assessment.basis);
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

fn encode_basis(bytes: &mut Vec<u8>, basis: EvidenceBasis) {
    bytes.push(match basis {
        EvidenceBasis::NormativeGuarantee => 1,
        EvidenceBasis::CheckedInvariant => 2,
        EvidenceBasis::SoundProof(VerifiedEvidenceRef(EvidenceReceiptKind::FusionNumerical)) => 3,
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

const fn subject_kind_tag(kind: SubjectKind) -> u8 {
    match kind {
        SubjectKind::SemanticProgram => 1,
        SubjectKind::Normalization => 2,
        SubjectKind::Region => 3,
        SubjectKind::Boundary => 4,
        SubjectKind::Candidate => 5,
        SubjectKind::Schedule => 6,
        SubjectKind::Target => 7,
        SubjectKind::Kernel => 8,
        SubjectKind::KernelProgram => 9,
        SubjectKind::ArtifactPlan => 10,
        SubjectKind::Alternative => 11,
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

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn deterministic_trace_is_sealed_and_rendered_separately() {
        let request = request(2.0);
        let mut first = ExplainWriter::new(&request, ExplainLimits::default()).unwrap();
        let parts = admitted(&first, "candidate:a");
        first
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap();
        let trace = first.finish().unwrap();
        assert!(trace.verify().is_ok());
        assert_eq!(trace.render(), trace.render());
        assert!(trace.render().starts_with("tiler-explain-v1\n"));
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
                vec![ExplainRecordId(0)]
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
    }

    #[test]
    fn invalid_stage_and_evidence_escalation_fail_closed() {
        assert_eq!(
            PredicateAssessment::proven("unknown", EvidenceBasis::Unknown),
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
    }

    #[test]
    fn bounds_truncate_details_but_retain_terminal_selection() {
        let request = request(2.0);
        let mut writer =
            ExplainWriter::new(&request, ExplainLimits::new(4, 64 * 1024).unwrap()).unwrap();
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
            .push_terminal(
                RuleRef::builtin("selection.v1").unwrap(),
                vec![rejected],
                ExplainEvent::Selection {
                    policy: SelectionPolicyKey::new("pareto.v1").unwrap(),
                    outcome: SelectionOutcome::Dominated,
                },
                Vec::new(),
            )
            .unwrap();
        let terminal = writer
            .subject(SubjectKind::Alternative, "alternative:a")
            .unwrap();
        writer
            .push_terminal(
                RuleRef::builtin("selection.v1").unwrap(),
                vec![terminal],
                ExplainEvent::Selection {
                    policy: SelectionPolicyKey::new("pareto.v1").unwrap(),
                    outcome: SelectionOutcome::Selected,
                },
                Vec::new(),
            )
            .unwrap();
        let trace = writer.finish().unwrap();
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
        let trace = writer.finish().unwrap();

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
        let rendered = writer.finish().unwrap().render();
        assert!(rendered.contains("disproved:shape-mismatch:checked-invariant:facts=rank:count=3"));
    }
}
