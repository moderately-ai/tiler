#![allow(
    dead_code,
    reason = "the explain authority itself is on the compile path; what stays unconstructed is the reserved evidence, quantity, disposition, and subject vocabulary the bounded profile does not yet produce, plus the presentation renderer, which only a trace consumer calls"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use tiler_ir::identity::{push_len, push_slice};

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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProviderRef {
    key: ProviderKey,
    revision: u32,
}

/// The compiler's own authority, named once so the constructor and the
/// recognizer below cannot drift apart.
const BUILTIN_PROVIDER_KEY: &str = "tiler.compiler";
const BUILTIN_PROVIDER_REVISION: u32 = 1;

impl ProviderRef {
    pub(crate) fn builtin() -> Self {
        Self {
            key: ProviderKey::new(BUILTIN_PROVIDER_KEY).expect("builtin provider key is valid"),
            revision: BUILTIN_PROVIDER_REVISION,
        }
    }

    /// Whether this reference names the compiler's own authority.
    ///
    /// Equivalent to comparing against [`Self::builtin`], which every retained
    /// record asked for and which allocates a key to answer.
    fn is_builtin(&self) -> bool {
        self.revision == BUILTIN_PROVIDER_REVISION && self.key.as_str() == BUILTIN_PROVIDER_KEY
    }

    /// References the provider that lowered one occurrence.
    ///
    /// The retained revision is the *provider's* output-affecting revision, not
    /// the capability revision: a `ProviderRef` names an authority, and ADR 0072
    /// keeps a provider's identity separate from the revisions of the individual
    /// capabilities it registers.
    pub(crate) fn lowering(provider: &LoweringProviderIdentity) -> Result<Self, ExplainError> {
        Self::registered(provider.provider())
    }

    /// References a registered provider by its governed namespaced identity.
    ///
    /// The namespace and name are joined so two providers sharing a name in
    /// different namespaces stay distinct in explain output, and the provider's
    /// output-affecting revision is retained as provenance (ADR 0072).
    pub(crate) fn registered(
        provider: &tiler_ir::semantic::ProviderIdentity,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            key: ProviderKey::new(format!("{}.{}", provider.namespace(), provider.name()))?,
            revision: provider.revision(),
        })
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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

/// The canonical bytes naming the compilation every subject reference belongs
/// to.
///
/// Shared rather than owned per reference. The blob is the request's full
/// canonical encoding — twenty kilobytes for the governed five-operation
/// program — and a writer hands one subject reference to nearly every record it
/// retains, so copying it per reference dominated the writer's cost.
#[derive(Clone, Debug, Eq)]
pub(crate) struct CompilationSubject {
    canonical: std::sync::Arc<[u8]>,
}

impl PartialEq for CompilationSubject {
    /// Byte equality, decided by pointer identity when the two references share
    /// one allocation.
    ///
    /// The byte comparison remains the definition, so a subject built
    /// independently from an identical request still compares equal and the
    /// cross-compilation guard rejects exactly the subjects it rejected before.
    /// What changes is that the guard's normal case — a subject the writer
    /// itself handed out, checked back against the writer's own — no longer
    /// reads twenty kilobytes to reach a conclusion the shared pointer already
    /// determines.
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.canonical, &other.canonical)
            || self.canonical == other.canonical
    }
}

impl CompilationSubject {
    pub(crate) fn from_request(request: &VerifiedTargetRequest) -> Self {
        let request = request.subject();
        let canonical = std::sync::Arc::from(request.canonical_explain_subject_bytes());
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
            candidate: SubjectKey::new(proof.candidate_label())?,
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

    /// Records a predicate the compilation could not decide.
    ///
    /// Unknown is a third class, not a soft rejection: the predicate was neither
    /// proven nor disproved, so its basis is [`EvidenceBasis::Unknown`] and no
    /// downstream reader may treat its absence of a rejection as a pass.
    pub(crate) fn unknown(
        predicate: impl AsRef<str>,
        reason: ReasonCode,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            predicate: PredicateKey::new(predicate)?,
            assessment: Assessment::Unknown(reason),
            basis: EvidenceBasis::Unknown,
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

    pub(crate) const fn basis(&self) -> &EvidenceBasis {
        &self.basis
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

    pub(crate) const fn value(self) -> u64 {
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

/// How one numerical dimension resolved against a target's declaration.
///
/// A distinct vocabulary from [`FeasibilityOutcome`], which is quantitative and
/// two-valued. This one carries the *means*, which is what a bound comparison
/// cannot express: an emulated dimension is admitted and changes the emitted
/// program, and an unhonourable one names the behaviour the target does honour
/// so a reader can see which contract this target would accept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HonourabilityOutcome {
    /// The target honours the required behaviour by the named means.
    Honoured { means: ReasonCode },
    /// The target declares it cannot honour the required behaviour by the named
    /// means, and honours the named behaviour instead when it honours one.
    Unhonourable {
        means: ReasonCode,
        honoured: Option<ReasonCode>,
    },
    /// Nothing the profile declares speaks to the required behaviour. A third
    /// class, never a rejection: no downstream reader may treat the absence of a
    /// refusal as an admission.
    Undeclared,
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
    /// One numerical dimension assessed against a target's declaration.
    ///
    /// This is the rejection shape ADR 0076 item 5 requires and the record that
    /// replaces `feasibility:strict-f32:rejected:count=1:0`. It names the
    /// dimension, the behaviour the caller's contract required, the means the
    /// profile declares, the behaviour the target does honour, and the declaring
    /// profile — none of which fits [`Self::Feasibility`], whose required and
    /// available fields are quantities compared by magnitude.
    NumericalHonourability {
        dimension: PredicateKey,
        required: ReasonCode,
        outcome: HonourabilityOutcome,
        profile: SubjectKey,
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
}

impl ExplainEvent {
    pub(crate) const fn stage(&self) -> ExplainStage {
        match self {
            Self::Check { stage, .. }
            | Self::BudgetStop { stage, .. }
            | Self::CompilerFailure { stage, .. } => *stage,
            Self::Feasibility { .. } | Self::NumericalHonourability { .. } => {
                ExplainStage::TargetFeasibility
            }
            Self::DeferredCapability { .. } => ExplainStage::CapabilityResolution,
            Self::CostAssessment { .. } => ExplainStage::Costing,
            Self::Selection { .. } => ExplainStage::Selection,
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
            }
            | Self::NumericalHonourability {
                outcome: HonourabilityOutcome::Honoured { .. },
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
            | Self::DeferredCapability { .. }
            // Undeclared is unknown, not rejected: the profile said nothing, so
            // the trace must not read as a refusal a reader could act on.
            | Self::NumericalHonourability {
                outcome: HonourabilityOutcome::Undeclared,
                ..
            } => ExplainDisposition::DeferredUnsupported,
            Self::BudgetStop { .. } => ExplainDisposition::BudgetStopped,
            Self::Feasibility {
                outcome: FeasibilityOutcome::Rejected(_),
                ..
            }
            | Self::NumericalHonourability {
                outcome: HonourabilityOutcome::Unhonourable { .. },
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
        }
    }

    fn validate(&self) -> Result<(), ExplainError> {
        match self {
            Self::Check {
                stage,
                assessment,
                rejection,
            } => {
                // Target feasibility, costing, and selection each own a typed
                // event that carries structure a checked predicate cannot, so a
                // `Check` at those stages would silently lose it. Capability
                // resolution has no such richer event for its admitted and
                // disproved cases — `DeferredCapability` only classes an absent
                // capability — so a checked predicate is its exact vocabulary.
                if matches!(
                    stage,
                    ExplainStage::TargetFeasibility
                        | ExplainStage::Costing
                        | ExplainStage::Selection
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
                            | ExplainStage::CapabilityResolution
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
            // A honourability record has no magnitude relation to validate: its
            // three outcomes are already disjoint, and the means it carries is a
            // governed key rather than a comparable quantity.
            Self::BudgetStop { .. }
            | Self::DeferredCapability { .. }
            | Self::NumericalHonourability { .. }
            | Self::Selection { .. }
            | Self::CompilerFailure { .. } => {}
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

#[derive(Debug)]
pub(crate) struct ExplainWriter {
    subject: CompilationSubject,
    authority: u64,
    request_qualifier: u64,
    allowed_providers: Vec<ProviderRef>,
    records: Vec<ExplainRecord>,
    /// The trace preamble, encoded once and reused as the head of the sealed
    /// identity. It carries the twenty-kilobyte compilation subject, so
    /// re-deriving it per seal would copy the blob a second time.
    identity_prefix: Vec<u8>,
    /// Every retained record's canonical encoding, concatenated in record
    /// order. This is the same byte run [`encode_trace`] would produce, built
    /// as records arrive: a record is encoded once, and its length — which the
    /// byte budget needs before the record is admitted — is the growth this
    /// buffer records rather than a second encoding measured and thrown away.
    encoded_records: Vec<u8>,
    retained_bytes: usize,
    retained_detail_records: usize,
    retained_detail_bytes: usize,
    selection_ledger: BTreeMap<SubjectKey, PendingSelection>,
    terminal_ledger_bytes: usize,
}

#[derive(Clone, Debug)]
struct PendingSelection {
    outcome: SelectionOutcome,
    cause: Option<TerminalCause>,
    authoritative_infeasible: bool,
}

/// A retained record a terminal record may cite as its cause.
///
/// Always a record identifier: a detail record is either retained or the
/// compilation is refused, so there is no dropped cause to re-materialize.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct TerminalCause {
    record: ExplainRecordId,
}

impl TerminalCause {
    pub(crate) const fn from_record(record: ExplainRecordId) -> Self {
        Self { record }
    }

    const fn retained_bytes() -> usize {
        std::mem::size_of::<ExplainRecordId>()
    }
}

#[derive(Clone, Debug)]
struct FailureCauseSet(Vec<TerminalCause>);

impl FailureCauseSet {
    fn new(mut causes: Vec<TerminalCause>) -> Result<Self, ExplainError> {
        check_bound(BoundKind::Causes, MAX_CAUSES_PER_RECORD, causes.len())?;
        causes.sort_unstable();
        if causes.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ExplainError::DuplicateCause);
        }
        Ok(Self(causes))
    }

    fn as_slice(&self) -> &[TerminalCause] {
        &self.0
    }

    fn into_vec(self) -> Vec<TerminalCause> {
        self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FailureDescriptor {
    pub(crate) stage: ExplainStage,
    pub(crate) reason: ReasonCode,
    pub(crate) subject_kind: SubjectKind,
    pub(crate) subject_key: SubjectKey,
    causes: FailureCauseSet,
}

impl FailureDescriptor {
    pub(crate) fn new(
        stage: ExplainStage,
        reason: impl AsRef<str>,
        subject_kind: SubjectKind,
        subject_key: impl AsRef<str>,
        cause: Option<TerminalCause>,
    ) -> Result<Self, ExplainError> {
        let causes = FailureCauseSet::new(cause.into_iter().collect())?;
        Ok(Self {
            stage,
            reason: ReasonCode::new(reason)?,
            subject_kind,
            subject_key: SubjectKey::new(subject_key)?,
            causes,
        })
    }

    pub(crate) fn with_causes(
        stage: ExplainStage,
        reason: impl AsRef<str>,
        subject_kind: SubjectKind,
        subject_key: impl AsRef<str>,
        causes: Vec<TerminalCause>,
    ) -> Result<Self, ExplainError> {
        let causes = FailureCauseSet::new(causes)?;
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
    pub(crate) fn new(request: &VerifiedTargetRequest) -> Result<Self, ExplainError> {
        let subject = CompilationSubject::from_request(request);
        // Every authority whose rules this compilation may attribute to a
        // provider: every provider the request's installed lowering registry
        // admits, plus the compiler's own governed physical-implementation and
        // fusion-capability providers. A rule attributed to any other provider is
        // a provenance forgery and fails closed (ADR 0072).
        let mut allowed_providers = vec![
            ProviderRef::registered(&crate::frontier::GovernedPhysicalProvider::identity())?,
            ProviderRef::registered(
                crate::fusion_legality::FusionNumericalCapabilities::governed().provider(),
            )?,
        ];
        for provider in request.capabilities().lowering().providers() {
            allowed_providers.push(ProviderRef::registered(&provider)?);
        }
        let mut identity_prefix = Vec::new();
        push_trace_preamble(&mut identity_prefix, EXPLAIN_SCHEMA_VERSION, &subject);
        // An empty trace is the preamble plus one record count. Measure that
        // count through `push_len` and drop it again rather than restating its
        // width here, so the framing has one author.
        let preamble_bytes = identity_prefix.len();
        push_len(&mut identity_prefix, 0);
        let retained_bytes = identity_prefix.len();
        identity_prefix.truncate(preamble_bytes);
        if retained_bytes > usize::try_from(MAX_CANONICAL_BYTES).unwrap_or(usize::MAX) {
            return Err(ExplainError::TerminalCapacity);
        }
        Ok(Self {
            authority: NEXT_WRITER_AUTHORITY.fetch_add(1, Ordering::Relaxed),
            request_qualifier: stable_qualifier(&subject.canonical),
            subject,
            allowed_providers,
            records: Vec::new(),
            identity_prefix,
            encoded_records: Vec::new(),
            retained_bytes,
            retained_detail_records: 0,
            retained_detail_bytes: 0,
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
    ) -> Result<ExplainRecordId, ExplainError> {
        if matches!(
            event,
            ExplainEvent::Selection { .. } | ExplainEvent::CompilerFailure { .. }
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
        let record = self.push_detail(rule, vec![subject], event.clone(), causes)?;
        Ok(TerminalCause::from_record(record))
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
            ExplainEvent::Selection { .. } | ExplainEvent::CompilerFailure { .. }
        ) {
            return Err(ExplainError::InvalidEventClass);
        }
        self.push(rule, subjects, event, causes, true)
    }

    fn push(
        &mut self,
        rule: RuleRef,
        mut subjects: Vec<SubjectRef>,
        mut event: ExplainEvent,
        mut causes: Vec<ExplainRecordId>,
        terminal: bool,
    ) -> Result<ExplainRecordId, ExplainError> {
        canonicalize_record_parts(&mut subjects, &mut event, &mut causes)?;
        event.validate()?;
        if !rule.provider.is_builtin() && !self.allowed_providers.contains(&rule.provider) {
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
        // Encoded straight into the retained buffer: the byte budget needs this
        // record's canonical length, and the length of an encoding is what the
        // encoder writing it reports. A record the bounds below refuse is
        // truncated away again, so the buffer holds exactly the retained
        // records and nothing else.
        let committed = self.encoded_records.len();
        push_record(&mut self.encoded_records, &record);
        let bytes = self.encoded_records.len() - committed;
        if terminal && bytes > usize::try_from(MAX_TERMINAL_RECORD_BYTES).unwrap_or(usize::MAX) {
            self.encoded_records.truncate(committed);
            return Err(ExplainError::TerminalCapacity);
        }
        // A trace is complete or it is refused. Exceeding a bound is a typed
        // failure, never a silent drop: a reader who cannot tell which records
        // are missing cannot rely on the ones that remain, and a summary naming
        // only how many were lost does not recover which. The detail bound is
        // the same `MAX_RECORDS`/`MAX_CANONICAL_BYTES` ceiling that
        // `MAX_TRACE_*` is derived from, so the two accountings stay consistent.
        let exceeds = if terminal {
            self.records.len().saturating_add(1)
                > usize::try_from(MAX_TRACE_RECORDS).unwrap_or(usize::MAX)
                || self.retained_bytes.saturating_add(bytes)
                    > usize::try_from(MAX_TRACE_CANONICAL_BYTES).unwrap_or(usize::MAX)
        } else {
            self.retained_detail_records.saturating_add(1)
                > usize::try_from(MAX_RECORDS).unwrap_or(usize::MAX)
                || self.retained_detail_bytes.saturating_add(bytes)
                    > usize::try_from(MAX_CANONICAL_BYTES).unwrap_or(usize::MAX)
        };
        if exceeds {
            self.encoded_records.truncate(committed);
            return Err(if terminal {
                ExplainError::TerminalCapacity
            } else {
                ExplainError::DetailCapacity
            });
        }
        self.retained_bytes += bytes;
        if !terminal {
            self.retained_detail_records += 1;
            self.retained_detail_bytes += bytes;
        }
        self.records.push(record);
        Ok(next)
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
        for (key, pending) in std::mem::take(&mut self.selection_ledger) {
            let cause = pending
                .cause
                .as_ref()
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
        for cause in failure.causes.as_slice() {
            self.validate_terminal_cause(Some(cause))?;
        }
        let admitted_causes = failure.causes.into_vec();
        let mut causes = Vec::with_capacity(admitted_causes.len());
        for cause in admitted_causes {
            causes.push(self.materialize_terminal_cause(&cause)?);
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
            .saturating_add(cause.map_or(0, |_| TerminalCause::retained_bytes()))
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
        match cause {
            Some(cause)
                if cause.record.writer_authority != self.authority
                    || cause.record.request_qualifier != self.request_qualifier =>
            {
                Err(ExplainError::CrossWriterCause)
            }
            _ => Ok(()),
        }
    }

    fn materialize_terminal_cause(
        &mut self,
        cause: &TerminalCause,
    ) -> Result<ExplainRecordId, ExplainError> {
        self.validate_terminal_cause(Some(cause))?;
        Ok(cause.record)
    }

    fn seal(mut self) -> Result<VerifiedExplainTrace, ExplainError> {
        if self.records.is_empty() {
            return Err(ExplainError::EmptyTrace);
        }
        // The same three parts [`encode_trace`] writes, with the first two
        // already in hand: the preamble was encoded when the writer opened, and
        // the records as each was admitted.
        let mut identity = std::mem::take(&mut self.identity_prefix);
        push_len(&mut identity, self.records.len());
        identity.extend_from_slice(&self.encoded_records);
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
        ExplainEvent::NumericalHonourability {
            dimension,
            required,
            outcome,
            profile,
        } => render_honourability(output, dimension, required, outcome, profile),
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
        } => render_cost(output, model, basis, terms, *disposition),
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
    }
}

/// Renders one cost-assessment record and its comma-separated terms.
fn render_cost(
    output: &mut String,
    model: &CostModelKey,
    basis: &EvidenceBasis,
    terms: &[CostTerm],
    disposition: CostDisposition,
) {
    use fmt::Write as _;
    let _ = write!(
        output,
        "cost:{}:{}:{}:",
        model.as_str(),
        basis_name(basis),
        cost_disposition_name(disposition)
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

/// Renders one numerical-honourability record.
///
/// Every part is written, including the declaring profile, because a reader that
/// saw only the dimension and the outcome could not tell which profile made the
/// claim — and a rejection whose declarer is unnamed is not explainable.
fn render_honourability(
    output: &mut String,
    dimension: &PredicateKey,
    required: &ReasonCode,
    outcome: &HonourabilityOutcome,
    profile: &SubjectKey,
) {
    use fmt::Write as _;
    let _ = write!(
        output,
        "honourability:{}:{}:",
        dimension.as_str(),
        required.as_str()
    );
    match outcome {
        HonourabilityOutcome::Honoured { means } => {
            let _ = write!(output, "honoured:{}", means.as_str());
        }
        HonourabilityOutcome::Unhonourable { means, honoured } => {
            let _ = write!(output, "unhonourable:{}", means.as_str());
            if let Some(honoured) = honoured {
                let _ = write!(output, ":honours={}", honoured.as_str());
            }
        }
        HonourabilityOutcome::Undeclared => output.push_str("undeclared"),
    }
    let _ = write!(output, ":profile={}", profile.as_str());
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
    /// A detail record would have exceeded the retained-trace ceiling.
    ///
    /// The compilation is refused rather than the record dropped, so a trace
    /// that exists is complete. Distinct from [`Self::TerminalCapacity`]:
    /// that one bounds the terminal ledger, this one the explanation body.
    DetailCapacity,
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

/// Whether a key contains a character the vocabulary forbids.
///
/// Below `0x80`, `char::is_control` (category `Cc`) is exactly the C0 range plus
/// `DEL`, and `char::is_whitespace` (the `White_Space` property) adds only the
/// space and the C0 characters `is_ascii_control` already covers — so an
/// all-ASCII key reaches the same verdict from its bytes, without decoding or
/// the Unicode property tables. Every key this compiler mints is ASCII; the
/// character scan stays as the definition for any that is not.
fn has_forbidden_character(value: &str) -> bool {
    if value.is_ascii() {
        return value
            .as_bytes()
            .iter()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace());
    }
    value
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
}

fn validate_key(kind: KeyKind, value: &str) -> Result<(), ExplainError> {
    if value.is_empty() || value.len() > MAX_KEY_BYTES || has_forbidden_character(value) {
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

/// Appends the fixed head of a canonical trace: the format tag, the schema
/// version, and the framed compilation subject.
///
/// Split out so [`encode_trace`] and the writer's incrementally built identity
/// share one preamble rather than agreeing by inspection.
fn push_trace_preamble(bytes: &mut Vec<u8>, schema: u32, subject: &CompilationSubject) {
    bytes.extend_from_slice(b"tiler.explain.trace.v1\0");
    bytes.extend_from_slice(&schema.to_be_bytes());
    push_slice(bytes, &subject.canonical);
}

fn encode_trace(schema: u32, subject: &CompilationSubject, records: &[ExplainRecord]) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_trace_preamble(&mut bytes, schema, subject);
    push_len(&mut bytes, records.len());
    for record in records {
        push_record(&mut bytes, record);
    }
    bytes
}

fn push_record(bytes: &mut Vec<u8>, record: &ExplainRecord) {
    bytes.extend_from_slice(&record.id.local.to_be_bytes());
    push_slice(bytes, record.rule.key.as_str().as_bytes());
    bytes.extend_from_slice(&record.rule.revision.to_be_bytes());
    push_slice(bytes, record.rule.provider.key.as_str().as_bytes());
    bytes.extend_from_slice(&record.rule.provider.revision.to_be_bytes());
    push_len(bytes, record.subjects.len());
    for subject in &record.subjects {
        bytes.push(subject_kind_tag(subject.kind));
        push_slice(bytes, subject.key.as_str().as_bytes());
    }
    encode_event(bytes, &record.event);
    push_len(bytes, record.causes.len());
    for cause in &record.causes {
        bytes.extend_from_slice(&cause.local.to_be_bytes());
    }
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
            push_slice(bytes, resource.as_str().as_bytes());
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
            push_slice(bytes, predicate.as_str().as_bytes());
            match outcome {
                FeasibilityOutcome::Admitted => bytes.push(1),
                FeasibilityOutcome::Rejected(reason) => {
                    bytes.push(2);
                    push_slice(bytes, reason.as_str().as_bytes());
                }
            }
            encode_quantity(bytes, *required);
            encode_quantity(bytes, *available);
        }
        ExplainEvent::NumericalHonourability {
            dimension,
            required,
            outcome,
            profile,
        } => encode_honourability(bytes, dimension, required, outcome, profile),
        ExplainEvent::DeferredCapability { predicate, reason } => {
            bytes.push(4);
            push_slice(bytes, predicate.as_str().as_bytes());
            push_slice(bytes, reason.as_str().as_bytes());
        }
        ExplainEvent::CostAssessment {
            model,
            basis,
            terms,
            disposition,
        } => encode_cost(bytes, model, basis, terms, *disposition),
        ExplainEvent::Selection { policy, outcome } => {
            bytes.push(6);
            push_slice(bytes, policy.as_str().as_bytes());
            bytes.push(match outcome {
                SelectionOutcome::Selected => 1,
                SelectionOutcome::Dominated => 2,
                SelectionOutcome::NotSelectedTradeoff => 3,
                SelectionOutcome::Infeasible => 4,
            });
        }
        ExplainEvent::CompilerFailure { stage, reason } => {
            bytes.extend_from_slice(&[7, stage_tag(*stage)]);
            push_slice(bytes, reason.as_str().as_bytes());
        }
    }
}

/// Canonically encodes one cost-assessment record: event tag `5`, then the
/// model, the evidence basis, the disposition, and the length-framed terms.
fn encode_cost(
    bytes: &mut Vec<u8>,
    model: &CostModelKey,
    basis: &EvidenceBasis,
    terms: &[CostTerm],
    disposition: CostDisposition,
) {
    bytes.push(5);
    push_slice(bytes, model.as_str().as_bytes());
    encode_basis(bytes, basis);
    bytes.push(match disposition {
        CostDisposition::Retained => 1,
        CostDisposition::Dominated => 2,
        CostDisposition::HigherCost => 3,
    });
    push_len(bytes, terms.len());
    for term in terms {
        push_slice(bytes, term.metric.as_str().as_bytes());
        encode_quantity(bytes, term.quantity);
    }
}

/// Canonically encodes one numerical-honourability record.
///
/// Event tag `10`, then the dimension, the required behaviour, a one-byte
/// outcome discriminant with its payload, and the declaring profile. The
/// declarer is inside the encoding because two traces that differ only in which
/// profile made a claim are different traces.
fn encode_honourability(
    bytes: &mut Vec<u8>,
    dimension: &PredicateKey,
    required: &ReasonCode,
    outcome: &HonourabilityOutcome,
    profile: &SubjectKey,
) {
    bytes.push(10);
    push_slice(bytes, dimension.as_str().as_bytes());
    push_slice(bytes, required.as_str().as_bytes());
    match outcome {
        HonourabilityOutcome::Honoured { means } => {
            bytes.push(1);
            push_slice(bytes, means.as_str().as_bytes());
        }
        HonourabilityOutcome::Unhonourable { means, honoured } => {
            bytes.push(2);
            push_slice(bytes, means.as_str().as_bytes());
            match honoured {
                Some(honoured) => {
                    bytes.push(1);
                    push_slice(bytes, honoured.as_str().as_bytes());
                }
                None => bytes.push(0),
            }
        }
        HonourabilityOutcome::Undeclared => bytes.push(3),
    }
    push_slice(bytes, profile.as_str().as_bytes());
}

fn encode_assessment(bytes: &mut Vec<u8>, assessment: &PredicateAssessment) {
    push_slice(bytes, assessment.predicate.as_str().as_bytes());
    match &assessment.assessment {
        Assessment::Proven => bytes.push(1),
        Assessment::Disproved(reason) => {
            bytes.push(2);
            push_slice(bytes, reason.as_str().as_bytes());
        }
        Assessment::Unknown(reason) => {
            bytes.push(3);
            push_slice(bytes, reason.as_str().as_bytes());
        }
        Assessment::Deferred(reason) => {
            bytes.push(4);
            push_slice(bytes, reason.as_str().as_bytes());
        }
    }
    encode_basis(bytes, &assessment.basis);
    push_len(bytes, assessment.facts.len());
    for fact in &assessment.facts {
        push_slice(bytes, fact.key.as_str().as_bytes());
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
                push_slice(bytes, value.as_str().as_bytes());
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
            push_slice(bytes, &receipt.compilation);
            push_slice(bytes, receipt.candidate.as_str().as_bytes());
            push_slice(bytes, receipt.provider.key.as_str().as_bytes());
            bytes.extend_from_slice(&receipt.provider.revision.to_be_bytes());
            push_slice(bytes, &receipt.proof);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::prove_fused_numerics;
    use crate::region::form_region_candidates;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    fn program(scale: f32) -> SemanticProgram {
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
        builder.build().unwrap()
    }

    fn request(scale: f32) -> VerifiedTargetRequest {
        let program = program(scale);
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        verified.for_target(verified.target_profiles()[0]).unwrap()
    }

    /// Returns one provider the request's installed lowering registry admits.
    fn governed_lowering_provider(request: &VerifiedTargetRequest) -> ProviderRef {
        let provider = request
            .capabilities()
            .lowering()
            .providers()
            .into_iter()
            .next()
            .expect("the governed registry admits at least one lowering provider");
        ProviderRef::registered(&provider).unwrap()
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
        let mut first = ExplainWriter::new(&request).unwrap();
        let parts = admitted(&first, "candidate:a");
        first
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
            .unwrap();
        let detail_only = ExplainWriter::new(&request).unwrap();
        assert_eq!(
            detail_only.finish_success(&["alternative:test"], "alternative:test"),
            Err(ExplainError::InvalidTerminalLedger)
        );
        let trace = finish_test_trace(first);
        assert!(trace.verify().is_ok());
        assert_eq!(
            trace.render(),
            concat!(
                // Rebaselined when the governed scalar definitions began
                // declaring their numerical facts and conformance identities,
                // and before that when the profile admitted
                // `tiler.scalar::canonicalize-nan-f32@1`. The request subject
                // covers the frozen scalar and lowering-capability authorities,
                // so changing what a governed definition states must move this
                // digest; a value that survived would mean the subject reached
                // the operation keys without reaching their contracts, and
                // growing the governed profile must move it for the same reason.
                //
                // Rebaselined again when the target profile's `supports_strict_f32`
                // boolean was replaced by its per-dimension honourability
                // declaration and the caller's stated contract preference joined
                // the resolved contract in the subject (ADR 0076 items 2 and 3).
                // Both changes are facts the subject must distinguish, so this
                // digest moving is the assertion, not collateral damage.
                //
                // The value below is neither branch's: each rebaselined against
                // its own change alone, so both were stale on the merged tree,
                // where the subject carries the governed definitions' contracts
                // *and* the per-dimension honourability declaration. Pinning
                // either one would assert a subject this tree does not build.
                //
                // Moved again when the lowering-capability registry identity
                // began interning the authority identities its capabilities
                // share instead of restating each in full. That is a change of
                // *spelling*, not of subject: the same registry is described,
                // and the pool plus fixed-width positions distinguish exactly
                // what the inline copies did. The qualifier is a digest of the
                // subject bytes, so a re-spelling has to move it — which is why
                // the registry's domain tag stepped to v2 in the same change.
                // Regenerate with:
                //   cargo nextest run -p tiler-compiler -E \
                //     'test(deterministic_trace_is_sealed_and_rendered_separately)'
                // and read the `left` value the assertion reports.
                "tiler-explain-v2 request=107be925f836ea4e\n",
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
        let mut first = ExplainWriter::new(&first_request).unwrap();
        let second = ExplainWriter::new(&second_request).unwrap();
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
            .unwrap();
        let mut same_request = ExplainWriter::new(&first_request).unwrap();
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
        let mut writer = ExplainWriter::new(&request).unwrap();
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
        let first_program = program(2.0);
        let first_request = request(2.0);
        let second_request = request(3.0);
        let formation = form_region_candidates(
            &first_program,
            first_request.budgets(),
            first_request.numerical_contract(),
        )
        .unwrap();
        let candidate = formation.whole_program_candidate().unwrap();
        let proof = prove_fused_numerics(formation.graph(), &first_request, candidate).unwrap();
        let provider = governed_lowering_provider(&first_request);
        let receipt =
            VerifiedEvidenceRef::from_fusion_numerical(&first_request, &proof, provider.clone())
                .unwrap();
        let mut writer = ExplainWriter::new(&second_request).unwrap();
        let subject = writer
            .subject(SubjectKind::Candidate, candidate.label())
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
            governed_lowering_provider(&first_request),
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
    fn every_detail_is_retained_beside_the_terminal_selection() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request).unwrap();
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
        // Every pushed detail survives to the sealed trace. There is no
        // retention or truncation record to look for, because a trace that
        // could not hold its records is refused rather than shortened.
        assert_eq!(
            trace
                .records()
                .iter()
                .filter(|record| record.rule().key().as_str() == "test.rule")
                .count(),
            3
        );
    }

    #[test]
    fn a_causal_detail_is_cited_by_the_record_itself() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request).unwrap();
        let predecessor_parts = admitted(&writer, "candidate:predecessor");
        let predecessor = writer
            .push_detail(
                predecessor_parts.rule,
                predecessor_parts.subjects,
                predecessor_parts.event,
                predecessor_parts.causes,
            )
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
        writer
            .note_selection(subject, SelectionOutcome::Selected, Some(cause))
            .unwrap();
        let trace = writer
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();
        // The cost record is retained, so the selection cites it directly.
        // Nothing stands in for a dropped predecessor, because none is dropped.
        let cost = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CostAssessment { .. }))
            .unwrap();
        assert_eq!(cost.rule().key().as_str(), "cost.exact-cause");
        assert_eq!(cost.subjects()[0].key().as_str(), "alternative:test");
        assert_eq!(cost.causes(), &[predecessor]);
        let selection = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::Selection { .. }))
            .unwrap();
        assert_eq!(selection.causes(), &[cost.id()]);
    }

    #[test]
    fn failure_trace_has_one_terminal_failure_citing_its_retained_detail() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request).unwrap();
        let parts = admitted(&writer, "candidate:retained");
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
        let detail = trace
            .records()
            .iter()
            .find(|record| record.rule().key().as_str() == "test.rule")
            .unwrap();
        let failure = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::CompilerFailure { .. }))
            .unwrap();
        assert_eq!(failure.causes(), &[detail.id()]);
        assert!(matches!(
            detail.subjects(),
            [subject]
                if subject.kind == SubjectKind::Candidate
                    && subject.key.as_str() == "candidate:retained"
        ));
    }

    #[test]
    fn failure_cause_admission_rejects_semantic_duplicates() {
        let request = request(2.0);
        let mut retained_writer = ExplainWriter::new(&request).unwrap();
        let retained_parts = admitted(&retained_writer, "candidate:retained-duplicate");
        let retained = retained_writer
            .push_causal_detail(
                retained_parts.rule,
                retained_parts.subjects.into_iter().next().unwrap(),
                &retained_parts.event,
                retained_parts.causes,
            )
            .unwrap();
        assert!(matches!(
            FailureDescriptor::with_causes(
                ExplainStage::KernelRefinement,
                "duplicate-retained",
                SubjectKind::Kernel,
                "failed-kernel",
                vec![retained, retained],
            ),
            Err(ExplainError::DuplicateCause)
        ));
        let retained_trace = retained_writer
            .finish_failure(
                FailureDescriptor::new(
                    ExplainStage::KernelRefinement,
                    "single-retained",
                    SubjectKind::Kernel,
                    "failed-kernel",
                    Some(retained),
                )
                .unwrap(),
            )
            .unwrap();
        assert!(retained_trace.verify().is_ok());
    }

    #[test]
    fn terminal_ledger_rejects_duplicates_unknowns_and_max_detail_pressure() {
        let request = request(2.0);
        let mut duplicate = ExplainWriter::new(&request).unwrap();
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

        let mut infeasible = ExplainWriter::new(&request).unwrap();
        assert_eq!(
            infeasible.note_selection(subject.clone(), SelectionOutcome::Infeasible, None),
            Err(ExplainError::InvalidTerminalLedger)
        );
        infeasible
            .note_infeasible_alternative(subject, None)
            .unwrap();

        let mut unknown = ExplainWriter::new(&request).unwrap();
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

        let mut pressured = ExplainWriter::new(&request).unwrap();
        for index in 0..MAX_RECORDS {
            let key = format!("candidate:{index}");
            let parts = admitted(&pressured, &key);
            pressured
                .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
                .unwrap();
        }
        // One past the ceiling is refused, not dropped. This is the guarantee
        // the whole retention design rests on: a sealed trace is complete.
        let excess_parts = admitted(&pressured, "candidate:excess");
        assert_eq!(
            pressured.push_detail(
                excess_parts.rule,
                excess_parts.subjects,
                excess_parts.event,
                excess_parts.causes,
            ),
            Err(ExplainError::DetailCapacity)
        );
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
        // The refused record left nothing behind. The writer encodes each record
        // into the retained buffer to learn its canonical length, so one the
        // bound then refuses has already been written and must be withdrawn;
        // `verify` re-encodes the sealed trace from its records and would find
        // the surplus run.
        assert!(trace.verify().is_ok());

        let mut bounded = ExplainWriter::new(&request).unwrap();
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
        let mut slice_bounded = ExplainWriter::new(&request).unwrap();
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
        let mut writer = ExplainWriter::new(&request).unwrap();
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
    fn maximum_terminal_ledger_seals_within_hard_trace_bounds() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request).unwrap();
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
        // One retained cost detail and one selection per alternative. No
        // truncation summary and no synthesized bridge: both existed only to
        // stand in for records the writer had dropped.
        assert_eq!(
            trace.records().len(),
            usize::try_from(MAX_TERMINAL_LEDGER_RECORDS * 2).unwrap()
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
        let mut writer = ExplainWriter::new(&request).unwrap();
        let parts = admitted(&writer, "candidate:a");
        let first = writer
            .push_detail(parts.rule, parts.subjects, parts.event, parts.causes)
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
        let mut writer = ExplainWriter::new(&request).unwrap();
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
