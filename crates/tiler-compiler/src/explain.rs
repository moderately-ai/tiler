#![allow(
    dead_code,
    reason = "the explain authority itself is on the compile path; what stays unconstructed is the reserved evidence, quantity, disposition, and subject vocabulary the bounded profile does not yet produce, plus the presentation renderer, which only a trace consumer calls"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{
    AvailabilityPhase, PreparedEntryTargetRequirement, TargetPropertyRequirementRelation,
};
use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::ResolvedValueType;

use crate::fusion::FusionNumericalProof;
use crate::request::{LoweringProviderIdentity, VerifiedTargetRequest};
use crate::target::honourability::NumericalRefusalEvidence;

// Schema v9 appends the complete refusing honourability fact — its declared
// behaviour, means, availability phase, authority, validity scope, versioned
// authority identity, and governed-guarantee or measured
// compiler-build/environment basis — to every unhonourable record. Under v8 two
// profiles refusing the same behaviour on different measured builds produced
// identical trace identities. v8 appended exact prepared-entry deferred target
// requirements; v7 appends the bits quantity used for exact widths; v6 adds the
// complete resolved dtype to numerical honourability; v5 appended opaque-call
// and provider subject kinds, the NotApplicable check class and disposition, and
// the arithmetic dtype to numerical honourability. Every earlier tag retains its
// v4 value. Renderer v7 spells that same refusal provenance; renderer v6
// appended the deferred-requirement spelling; renderer v5 appended the `bits`
// unit without changing any existing spelling.
pub(crate) const EXPLAIN_SCHEMA_VERSION: u32 = 9;
pub(crate) const EXPLAIN_RENDERER_VERSION: u32 = 7;
const COMPILATION_EXPLAIN_SCHEMA_VERSION: u32 = 1;
const COMPILATION_EXPLAIN_RENDERER_VERSION: u32 = 1;
const MAX_COMPILATION_EXPLAIN_CANDIDATES: usize = 256;
const MAX_COMPILATION_EXPLAIN_CANONICAL_BYTES: usize = 256 * 1024 * 1024;
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
const PHYSICAL_SELECTION_POLICY: &str = "tiler.selection.structural-pareto.v1";
const SEMANTIC_PORTFOLIO_SELECTION_POLICY: &str = "tiler.selection.semantic-portfolio.v1";
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
    SemanticDischarge,
    ProgramVerification,
    ArtifactPlanning,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ExplainDisposition {
    Admitted,
    /// Admitted to executable planning with an exact pre-routing query.
    DeferredAdmitted,
    RejectedIntrinsic,
    RejectedNumerical,
    RejectedTarget,
    DeferredUnsupported,
    BudgetStopped,
    Reported,
    Retained,
    DominancePruned,
    HigherCost,
    PreferencePruned,
    NotSelectedTradeoff,
    Selected,
    CompilerFailure,
    /// A candidate-enumeration predicate proved this proposal does not apply.
    NotApplicable,
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
    /// One exact governed opaque-call identity.
    OpaqueCall,
    /// One exact governed proposal-provider identity.
    Provider,
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
    subject_kind: SubjectKind,
    subject: SubjectKey,
    provider: ProviderRef,
    proof: Box<[u8]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceReceiptKind {
    FusionNumerical,
    IndexDomain,
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
            subject_kind: SubjectKind::Candidate,
            subject: SubjectKey::new(proof.candidate_label())?,
            provider,
            proof: proof.canonical_explain_evidence_bytes().into_boxed_slice(),
        })
    }

    pub(crate) fn from_index_domain(
        subject: &SubjectRef,
        proof: &tiler_ir::index::IndexRefinementDomainProof,
    ) -> Self {
        Self {
            kind: EvidenceReceiptKind::IndexDomain,
            compilation: subject.compilation.canonical.to_vec().into_boxed_slice(),
            subject_kind: subject.kind,
            subject: subject.key.clone(),
            provider: ProviderRef::builtin(),
            proof: proof.identity().as_bytes().into(),
        }
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

    /// Records a predicate whose deciding evidence belongs to a later phase.
    pub(crate) fn deferred(
        predicate: impl AsRef<str>,
        reason: ReasonCode,
    ) -> Result<Self, ExplainError> {
        Ok(Self {
            predicate: PredicateKey::new(predicate)?,
            assessment: Assessment::Deferred(reason),
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

    pub(crate) const fn reason(&self) -> Option<&ReasonCode> {
        match &self.assessment {
            Assessment::Proven => None,
            Assessment::Disproved(reason)
            | Assessment::Unknown(reason)
            | Assessment::Deferred(reason) => Some(reason),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Quantity {
    Count(u64),
    Bytes(u64),
    Bits(u64),
    Threads(u64),
    Bindings(u64),
    Operations(u64),
    Registers(u64),
    Nanoseconds(u64),
}

impl Quantity {
    const fn kind(self) -> u8 {
        match self {
            Self::Count(_) => 1,
            Self::Bytes(_) => 2,
            Self::Threads(_) => 3,
            Self::Bindings(_) => 4,
            Self::Operations(_) => 5,
            Self::Registers(_) => 6,
            Self::Nanoseconds(_) => 7,
            Self::Bits(_) => 8,
        }
    }

    pub(crate) const fn value(self) -> u64 {
        match self {
            Self::Count(value)
            | Self::Bytes(value)
            | Self::Bits(value)
            | Self::Threads(value)
            | Self::Bindings(value)
            | Self::Operations(value)
            | Self::Registers(value)
            | Self::Nanoseconds(value) => value,
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
    Reported,
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

/// How one complete synchronization subject resolved against a target.
///
/// A third vocabulary, and not [`FeasibilityOutcome`]: a subject is matched by
/// equality and has no magnitude, so the quantitative `required > available`
/// relation that record validates has nothing to range over here. Three values
/// rather than two, for the reason [`HonourabilityOutcome`] has three: the
/// absence of a refusal is not an admission, and a reader must be able to see
/// that a target simply never spoke to this realization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SynchronizationOutcome {
    /// A fact declares the target realizes exactly this subject.
    Realized { profile: SubjectKey },
    /// A fact declares the target does not realize it.
    Unrealizable { profile: SubjectKey },
    /// Nothing available declares anything about this exact subject.
    ///
    /// This is where a profile carrying facts about *neighbouring* subjects
    /// lands, and the record says so rather than reporting the closest one:
    /// naming a near miss in an explanation invites a reader to compose it.
    Undeclared,
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
    ///
    /// `evidence` is the exact checked fact that refused, carried from the
    /// feasibility authority rather than rebuilt here. `means` restates that
    /// fact's means as an explain reason code, which is what the record's key
    /// vocabulary indexes on; the evidence is what makes the record explainable,
    /// because it names the authority, validity scope, and measured builds and
    /// environments the refusal rests on.
    Unhonourable {
        means: ReasonCode,
        honoured: Option<ReasonCode>,
        evidence: NumericalRefusalEvidence,
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
    /// The proposal's applicability predicate was disproved.
    NotApplicable,
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
    /// One unresolved hard predicate admitted through an exact prepared-entry
    /// query before routing commit.
    ///
    /// `required` carries the explain vocabulary's typed unit; `requirement`
    /// carries the executable relation and complete versioned query identity.
    /// Their values are checked equal before the record is admitted.
    DeferredTargetRequirement {
        /// Zero-based program-entry ordinal whose prepared subject is queried.
        entry: u32,
        predicate: PredicateKey,
        required: Quantity,
        requirement: PreparedEntryTargetRequirement,
    },
    /// One numerical dimension assessed against a target's declaration.
    ///
    /// This is the rejection shape ADR 0076 item 5 requires and the record that
    /// replaces `feasibility:strict-f32:rejected:count=1:0`. It names the
    /// dimension, arithmetic dtype, behaviour the caller's contract required,
    /// means the profile declares, behaviour the target does honour, and
    /// declaring profile — none of which fits [`Self::Feasibility`], whose
    /// required and available fields are quantities compared by magnitude.
    NumericalHonourability {
        dimension: PredicateKey,
        /// Arithmetic dtype in which the behaviour must be honoured.
        arithmetic: ArithmeticType,
        /// Complete resolved semantic dtype subject.
        resolved_type: ResolvedValueType,
        required: ReasonCode,
        outcome: HonourabilityOutcome,
        profile: SubjectKey,
    },
    /// One complete synchronization realization assessed against a target.
    ///
    /// The whole subject in one record, deliberately not five records and not a
    /// [`Self::Feasibility`] row per dimension: a subject is matched by equality
    /// and has no magnitude, and splitting it across rows would render an
    /// explanation from which a reader could conclude that four of its five
    /// dimensions were "admitted" — the exact composition the atomic fact exists
    /// to prevent, reproduced in the explanation instead of in the authority.
    ///
    /// **A candidate requiring no synchronization emits no record at all.** The
    /// absence is not an omission to be filled with a zero row; it is what the
    /// retired barrier-count axis used to report as `required 0`.
    SynchronizationRealization {
        /// Governed key of the required operation kind.
        kind: ReasonCode,
        /// Invocations that must arrive at the point.
        execution_scope: ReasonCode,
        /// Invocations across which its fenced effects publish.
        visibility_scope: ReasonCode,
        /// Whether workgroup memory is among the fenced domains.
        fences_workgroup: bool,
        /// Whether device memory is among the fenced domains.
        fences_device: bool,
        /// Governed key of the required ordering.
        ordering: ReasonCode,
        /// How the target resolved the whole subject.
        outcome: SynchronizationOutcome,
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
    SemanticSelection {
        policy: SelectionPolicyKey,
        outcome: SelectionOutcome,
        candidate: CompilationSubject,
    },
    PreferencePruned {
        preferred_contract: ReasonCode,
        candidate_contract: ReasonCode,
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
            Self::Feasibility { .. }
            | Self::DeferredTargetRequirement { .. }
            | Self::NumericalHonourability { .. }
            | Self::SynchronizationRealization { .. } => ExplainStage::TargetFeasibility,
            Self::DeferredCapability { .. } => ExplainStage::CapabilityResolution,
            Self::CostAssessment { .. } => ExplainStage::Costing,
            Self::Selection { .. }
            | Self::SemanticSelection { .. }
            | Self::PreferencePruned { .. } => ExplainStage::Selection,
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
            }
            | Self::SynchronizationRealization {
                outcome: SynchronizationOutcome::Realized { .. },
                ..
            } => ExplainDisposition::Admitted,
            Self::DeferredTargetRequirement { .. } => ExplainDisposition::DeferredAdmitted,
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
                        assessment: Assessment::Disproved(_),
                        ..
                    },
                rejection: RejectionClass::NotApplicable,
                ..
            } => ExplainDisposition::NotApplicable,
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
            }
            // Undeclared for the same reason and with the same consequence: a
            // profile that never spoke to this exact realization has not refused
            // it, and a reader must not act on the silence either way.
            | Self::SynchronizationRealization {
                outcome: SynchronizationOutcome::Undeclared,
                ..
            } => ExplainDisposition::DeferredUnsupported,
            Self::BudgetStop { .. } => ExplainDisposition::BudgetStopped,
            Self::CostAssessment {
                disposition: CostDisposition::Reported,
                ..
            } => ExplainDisposition::Reported,
            Self::Feasibility {
                outcome: FeasibilityOutcome::Rejected(_),
                ..
            }
            | Self::NumericalHonourability {
                outcome: HonourabilityOutcome::Unhonourable { .. },
                ..
            }
            | Self::SynchronizationRealization {
                outcome: SynchronizationOutcome::Unrealizable { .. },
                ..
            }
            | Self::Selection {
                outcome: SelectionOutcome::Infeasible,
                ..
            }
            | Self::SemanticSelection {
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
            }
            | Self::SemanticSelection {
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
            }
            | Self::SemanticSelection {
                outcome: SelectionOutcome::NotSelectedTradeoff,
                ..
            } => ExplainDisposition::NotSelectedTradeoff,
            Self::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            }
            | Self::SemanticSelection {
                outcome: SelectionOutcome::Selected,
                ..
            } => ExplainDisposition::Selected,
            Self::PreferencePruned { .. } => ExplainDisposition::PreferencePruned,
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
                    && !matches!(
                        stage,
                        ExplainStage::NumericalLegality | ExplainStage::SemanticDischarge
                    )
                {
                    return Err(ExplainError::InvalidStageEvent);
                }
                let rejection_matches_stage = matches!(
                    (stage, rejection),
                    (
                        ExplainStage::NumericalLegality,
                        RejectionClass::NumericalIllegal
                    ) | (
                        ExplainStage::CandidateEnumeration,
                        RejectionClass::NotApplicable
                    ) | (
                        ExplainStage::RequestVerification
                            | ExplainStage::Normalization
                            | ExplainStage::RegionFormation
                            | ExplainStage::CandidateEnumeration
                            | ExplainStage::CapabilityResolution
                            | ExplainStage::IntrinsicScheduling
                            | ExplainStage::KernelRefinement
                            | ExplainStage::SemanticDischarge
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
            Self::DeferredTargetRequirement {
                required,
                requirement,
                ..
            } => {
                if required.value() != requirement.required() {
                    return Err(ExplainError::RequirementQuantityMismatch);
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
            // A subject that fences nothing publishes nothing, so a record
            // claiming one was assessed states a check no realization could
            // satisfy and no requirement could have produced.
            Self::SynchronizationRealization {
                fences_workgroup,
                fences_device,
                ..
            } if !fences_workgroup && !fences_device => {
                return Err(ExplainError::InvalidQuantityRelation);
            }
            // A honourability record has no magnitude relation to validate: its
            // three outcomes are already disjoint, and the means it carries is a
            // governed key rather than a comparable quantity. A synchronization
            // record has none for the same reason, and one more: its subject is
            // matched by equality, so there is no `required > available` to check.
            Self::BudgetStop { .. }
            | Self::DeferredCapability { .. }
            | Self::NumericalHonourability { .. }
            | Self::SynchronizationRealization { .. }
            | Self::Selection { .. }
            | Self::SemanticSelection { .. }
            | Self::PreferencePruned { .. }
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
    candidate: Option<CompilationSubject>,
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
            ExplainEvent::Selection { .. }
                | ExplainEvent::SemanticSelection { .. }
                | ExplainEvent::CompilerFailure { .. }
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
            ExplainEvent::Selection { .. }
                | ExplainEvent::SemanticSelection { .. }
                | ExplainEvent::CompilerFailure { .. }
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
                || subjects[0].kind != receipt.subject_kind
                || subjects[0].key != receipt.subject)
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
        let terminal_record_limit =
            if matches!(record.event, ExplainEvent::SemanticSelection { .. }) {
                MAX_CANONICAL_BYTES.saturating_add(MAX_TERMINAL_RECORD_BYTES)
            } else {
                MAX_TERMINAL_RECORD_BYTES
            };
        if terminal && bytes > usize::try_from(terminal_record_limit).unwrap_or(usize::MAX) {
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
        self,
        alternatives: &[&str],
        selected: &str,
    ) -> Result<VerifiedExplainTrace, ExplainError> {
        self.finish_success_with_policy(alternatives, selected, PHYSICAL_SELECTION_POLICY)
    }

    /// Seals a top-level selection over independently readmitted semantic
    /// candidates.
    ///
    /// Its dedicated policy keeps these entries distinguishable from the
    /// physical-alternative ledger sealed by [`Self::finish_success`]. The
    /// composite explanation validates this ledger against its keyed candidate
    /// traces before accepting it.
    pub(crate) fn finish_semantic_portfolio(
        self,
        candidates: &[&str],
        selected: &str,
    ) -> Result<VerifiedExplainTrace, ExplainError> {
        self.finish_success_with_policy(candidates, selected, SEMANTIC_PORTFOLIO_SELECTION_POLICY)
    }

    fn finish_success_with_policy(
        mut self,
        alternatives: &[&str],
        selected: &str,
        selection_policy: &str,
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
        if selection_policy == SEMANTIC_PORTFOLIO_SELECTION_POLICY
            && expected.len() > MAX_COMPILATION_EXPLAIN_CANDIDATES
        {
            return Err(ExplainError::TerminalLedgerCapacity);
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
                || (selection_policy == SEMANTIC_PORTFOLIO_SELECTION_POLICY)
                    != pending.candidate.is_some()
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
            let policy = SelectionPolicyKey::new(selection_policy)?;
            let event = match pending.candidate {
                Some(candidate) => ExplainEvent::SemanticSelection {
                    policy,
                    outcome: pending.outcome,
                    candidate,
                },
                None => ExplainEvent::Selection {
                    policy,
                    outcome: pending.outcome,
                },
            };
            self.push_terminal(
                RuleRef::builtin(selection_policy)?,
                vec![subject],
                event,
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
            .filter(|record| {
                matches!(
                    record.event,
                    ExplainEvent::Selection { .. } | ExplainEvent::SemanticSelection { .. }
                )
            })
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
        self.admit_selection(key, SelectionOutcome::Infeasible, cause, true, None)
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
        self.admit_selection(key, outcome, cause, false, None)?;
        Ok(())
    }

    pub(crate) fn note_semantic_selection(
        &mut self,
        subject: SubjectRef,
        candidate: &VerifiedTargetRequest,
        outcome: SelectionOutcome,
        cause: Option<TerminalCause>,
    ) -> Result<(), ExplainError> {
        if subject.compilation != self.subject || subject.kind != SubjectKind::Alternative {
            return Err(ExplainError::CrossCompilationSubject);
        }
        if outcome == SelectionOutcome::Infeasible
            || self.selection_ledger.contains_key(&subject.key)
        {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        self.admit_selection(
            subject.key,
            outcome,
            cause,
            false,
            Some(CompilationSubject::from_request(candidate)),
        )
    }

    pub(crate) fn note_semantic_infeasible(
        &mut self,
        subject: SubjectRef,
        candidate: &VerifiedTargetRequest,
        cause: Option<TerminalCause>,
    ) -> Result<(), ExplainError> {
        if subject.compilation != self.subject || subject.kind != SubjectKind::Alternative {
            return Err(ExplainError::CrossCompilationSubject);
        }
        self.admit_selection(
            subject.key,
            SelectionOutcome::Infeasible,
            cause,
            true,
            Some(CompilationSubject::from_request(candidate)),
        )
    }

    fn admit_selection(
        &mut self,
        key: SubjectKey,
        outcome: SelectionOutcome,
        cause: Option<TerminalCause>,
        authoritative_infeasible: bool,
        candidate: Option<CompilationSubject>,
    ) -> Result<(), ExplainError> {
        if self.selection_ledger.contains_key(&key) {
            return Err(ExplainError::InvalidTerminalLedger);
        }
        self.validate_terminal_cause(cause.as_ref())?;
        let entry_bytes = key
            .as_str()
            .len()
            .saturating_add(cause.map_or(0, |_| TerminalCause::retained_bytes()))
            .saturating_add(
                candidate
                    .as_ref()
                    .map_or(0, |candidate| candidate.canonical.len()),
            )
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
                candidate,
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

/// The verified explanation of one compilation-wide semantic portfolio.
///
/// Opaque because the record vocabulary remains compiler-private. The value
/// binds one top-level selection trace, qualified by the original request, to
/// every per-semantic-candidate trace that selection compared. Each candidate
/// trace remains sealed under its own readmitted request, so a sound proof is
/// never transplanted into the original request's writer merely to make the
/// portfolio look like one trace.
///
/// Candidate order is canonical correlation-key then request-subject order. The
/// canonical identity embeds each candidate's bounded correlation key, full
/// request subject, and collision-free trace identity beside the selection
/// trace, with length framing at every boundary. The key connects a
/// semantic-selection entry to its sealed trace only inside this composite; it
/// is not a standalone identity and must never be used in place of the full
/// request subject. A digest is never used for equality.
#[derive(Clone, Eq, PartialEq)]
pub struct VerifiedCompilationExplain {
    selection: std::sync::Arc<VerifiedExplainTrace>,
    candidates: Box<[SemanticCandidateExplain]>,
    binding: SelectionBinding,
    canonical_identity: Box<[u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SemanticCandidateExplain {
    key: SubjectKey,
    trace: std::sync::Arc<VerifiedExplainTrace>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectionBinding {
    Singleton,
    SemanticPortfolio,
}

impl VerifiedCompilationExplain {
    /// Adapts one already-sealed singleton trace to the composite boundary.
    ///
    /// The trace is shared rather than cloned because it serves as both the
    /// selection trace and the sole candidate trace. The live algebraic
    /// portfolio uses [`Self::from_traces`] with a distinct top-level selection
    /// trace and one trace per independently readmitted candidate.
    pub(crate) fn one_candidate(trace: VerifiedExplainTrace) -> Self {
        let trace = std::sync::Arc::new(trace);
        let key = selected_candidate_key(&trace)
            .expect("a successful verified trace has one selected alternative");
        Self::assemble(
            std::sync::Arc::clone(&trace),
            vec![SemanticCandidateExplain { key, trace }],
            SelectionBinding::Singleton,
        )
        .expect("one verified trace fits the compilation-explain bounds")
    }

    /// Binds a top-level selection trace to the readmitted semantic candidates
    /// it selected among.
    ///
    /// The selection remains rooted in the original request while candidate
    /// traces are rooted in their independently readmitted, post-normalization
    /// requests. Each bounded key is a correlation handle local to this sealed
    /// composite, not an identity: construction requires its exact equality
    /// with the dedicated semantic selection ledger and still encodes every
    /// full request subject beside it. Duplicate keys and duplicate candidate
    /// request subjects are refused rather than silently deduplicated.
    pub(crate) fn from_traces(
        selection: VerifiedExplainTrace,
        candidates: Vec<(SubjectKey, VerifiedExplainTrace)>,
    ) -> Result<Self, CompilationExplainError> {
        Self::assemble(
            std::sync::Arc::new(selection),
            candidates
                .into_iter()
                .map(|(key, trace)| SemanticCandidateExplain {
                    key,
                    trace: std::sync::Arc::new(trace),
                })
                .collect(),
            SelectionBinding::SemanticPortfolio,
        )
    }

    fn assemble(
        selection: std::sync::Arc<VerifiedExplainTrace>,
        mut candidates: Vec<SemanticCandidateExplain>,
        binding: SelectionBinding,
    ) -> Result<Self, CompilationExplainError> {
        if candidates.is_empty() {
            return Err(CompilationExplainError::Empty);
        }
        if candidates.len() > MAX_COMPILATION_EXPLAIN_CANDIDATES {
            return Err(CompilationExplainError::CandidateCapacity);
        }
        candidates.sort_by(|left, right| {
            (&left.key, &left.trace.compilation_subject.canonical)
                .cmp(&(&right.key, &right.trace.compilation_subject.canonical))
        });
        if candidates.windows(2).any(|pair| pair[0].key == pair[1].key) {
            return Err(CompilationExplainError::DuplicateCandidateKey);
        }
        if has_duplicate_candidate_subjects(&candidates) {
            return Err(CompilationExplainError::DuplicateCandidate);
        }
        let candidate_keys = candidates
            .iter()
            .map(|candidate| candidate.key.clone())
            .collect::<BTreeSet<_>>();
        let mut semantic_bindings = None;
        let selection_keys = match binding {
            SelectionBinding::Singleton => {
                let key = selected_candidate_key(&selection)?;
                BTreeSet::from([key])
            }
            SelectionBinding::SemanticPortfolio => {
                let bindings = semantic_selection_bindings(&selection)?;
                let keys = bindings.keys().cloned().collect();
                semantic_bindings = Some(bindings);
                keys
            }
        };
        if selection_keys != candidate_keys {
            return Err(CompilationExplainError::CandidateKeyMismatch);
        }
        if semantic_bindings.is_some_and(|bindings| {
            candidates.iter().any(|candidate| {
                bindings.get(&candidate.key) != Some(&candidate.trace.compilation_subject)
            })
        }) {
            return Err(CompilationExplainError::CandidateSubjectMismatch);
        }
        let candidates = candidates.into_boxed_slice();
        let canonical_identity = encode_compilation_explain(&selection, &candidates, binding)?;
        Ok(Self {
            selection,
            candidates,
            binding,
            canonical_identity,
        })
    }

    /// Returns how many independently sealed semantic candidates participate.
    #[must_use]
    pub fn semantic_candidate_count(&self) -> usize {
        self.candidates.len()
    }

    /// Renders the complete compilation explanation deterministically.
    ///
    /// The spelling is diagnostic rather than a parse contract. The top-level
    /// selection appears first, followed by candidates in canonical key then
    /// request-subject order; each nested trace retains its own renderer header
    /// and request qualifier.
    #[must_use]
    pub fn render(&self) -> String {
        use fmt::Write as _;

        let mut output = format!(
            "tiler-compilation-explain-v{COMPILATION_EXPLAIN_RENDERER_VERSION} semantic-candidates={}\n",
            self.candidates.len()
        );
        output.push_str("top-level-selection\n");
        output.push_str(&self.selection.render());
        for (index, candidate) in self.candidates.iter().enumerate() {
            let _ = writeln!(
                output,
                "semantic-candidate {index} key={}",
                candidate.key.as_str()
            );
            output.push_str(&candidate.trace.render());
        }
        output
    }

    #[cfg(test)]
    fn identity(&self) -> &[u8] {
        &self.canonical_identity
    }

    #[cfg(test)]
    fn verify(&self) -> Result<(), CompilationExplainError> {
        let selection_keys = match self.binding {
            SelectionBinding::Singleton => {
                selected_candidate_key(&self.selection).map(|key| BTreeSet::from([key]))
            }
            SelectionBinding::SemanticPortfolio => semantic_selection_bindings(&self.selection)
                .and_then(|bindings| {
                    if self.candidates.iter().all(|candidate| {
                        bindings.get(&candidate.key) == Some(&candidate.trace.compilation_subject)
                    }) {
                        Ok(bindings.into_keys().collect())
                    } else {
                        Err(CompilationExplainError::CandidateSubjectMismatch)
                    }
                }),
        }
        .map_err(|_| CompilationExplainError::StaleIdentity)?;
        let candidate_keys = self
            .candidates
            .iter()
            .map(|candidate| candidate.key.clone())
            .collect::<BTreeSet<_>>();
        if self.candidates.is_empty()
            || self.candidates.len() > MAX_COMPILATION_EXPLAIN_CANDIDATES
            || selection_keys != candidate_keys
            || self.candidates.windows(2).any(|pair| {
                (&pair[0].key, &pair[0].trace.compilation_subject.canonical)
                    >= (&pair[1].key, &pair[1].trace.compilation_subject.canonical)
            })
            || self
                .candidates
                .windows(2)
                .any(|pair| pair[0].key == pair[1].key)
            || has_duplicate_candidate_subjects(&self.candidates)
            || self.selection.verify().is_err()
            || self
                .candidates
                .iter()
                .any(|candidate| candidate.trace.verify().is_err())
            || encode_compilation_explain(&self.selection, &self.candidates, self.binding)?.as_ref()
                != self.canonical_identity.as_ref()
        {
            return Err(CompilationExplainError::StaleIdentity);
        }
        Ok(())
    }
}

impl fmt::Debug for VerifiedCompilationExplain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VerifiedCompilationExplain")
            .field("semantic_candidates", &self.candidates.len())
            .field("selection_records", &self.selection.records().len())
            .finish_non_exhaustive()
    }
}

fn encode_compilation_explain(
    selection: &VerifiedExplainTrace,
    candidates: &[SemanticCandidateExplain],
    binding: SelectionBinding,
) -> Result<Box<[u8]>, CompilationExplainError> {
    encode_compilation_explain_with_capacity(
        selection,
        candidates,
        binding,
        MAX_COMPILATION_EXPLAIN_CANONICAL_BYTES,
    )
}

fn encode_compilation_explain_with_capacity(
    selection: &VerifiedExplainTrace,
    candidates: &[SemanticCandidateExplain],
    binding: SelectionBinding,
    maximum_bytes: usize,
) -> Result<Box<[u8]>, CompilationExplainError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.explain.compilation.v1\0");
    bytes.extend_from_slice(&COMPILATION_EXPLAIN_SCHEMA_VERSION.to_be_bytes());
    bytes.push(match binding {
        SelectionBinding::Singleton => 1,
        SelectionBinding::SemanticPortfolio => 2,
    });
    push_slice(&mut bytes, selection.identity().as_bytes());
    push_len(&mut bytes, candidates.len());
    for candidate in candidates {
        push_slice(&mut bytes, candidate.key.as_str().as_bytes());
        push_slice(&mut bytes, &candidate.trace.compilation_subject.canonical);
        push_slice(&mut bytes, candidate.trace.identity().as_bytes());
        check_compilation_explain_capacity(bytes.len(), maximum_bytes)?;
    }
    check_compilation_explain_capacity(bytes.len(), maximum_bytes)?;
    Ok(bytes.into_boxed_slice())
}

fn has_duplicate_candidate_subjects(candidates: &[SemanticCandidateExplain]) -> bool {
    let mut subjects = BTreeSet::new();
    candidates
        .iter()
        .any(|candidate| !subjects.insert(candidate.trace.compilation_subject.canonical.as_ref()))
}

fn selected_candidate_key(
    trace: &VerifiedExplainTrace,
) -> Result<SubjectKey, CompilationExplainError> {
    let mut selected = None;
    for record in trace.records() {
        match record.event() {
            ExplainEvent::Selection {
                outcome: SelectionOutcome::Selected,
                ..
            } => {
                let [subject] = record.subjects() else {
                    return Err(CompilationExplainError::InvalidSelectionTrace);
                };
                if subject.kind() != SubjectKind::Alternative || selected.is_some() {
                    return Err(CompilationExplainError::InvalidSelectionTrace);
                }
                selected = Some(subject.key().clone());
            }
            ExplainEvent::CompilerFailure { .. } => {
                return Err(CompilationExplainError::InvalidSelectionTrace);
            }
            _ => {}
        }
    }
    selected.ok_or(CompilationExplainError::InvalidSelectionTrace)
}

fn semantic_selection_bindings(
    trace: &VerifiedExplainTrace,
) -> Result<BTreeMap<SubjectKey, CompilationSubject>, CompilationExplainError> {
    let mut bindings = BTreeMap::new();
    let mut selected = 0_usize;
    for record in trace.records() {
        match record.event() {
            ExplainEvent::SemanticSelection {
                policy,
                outcome,
                candidate,
            } if policy.as_str() == SEMANTIC_PORTFOLIO_SELECTION_POLICY => {
                let [subject] = record.subjects() else {
                    return Err(CompilationExplainError::InvalidSelectionTrace);
                };
                if subject.kind() != SubjectKind::Alternative
                    || bindings
                        .insert(subject.key().clone(), candidate.clone())
                        .is_some()
                {
                    return Err(CompilationExplainError::InvalidSelectionTrace);
                }
                selected += usize::from(*outcome == SelectionOutcome::Selected);
            }
            ExplainEvent::CompilerFailure { .. } => {
                return Err(CompilationExplainError::InvalidSelectionTrace);
            }
            _ => {}
        }
    }
    if bindings.is_empty() || selected != 1 {
        return Err(CompilationExplainError::InvalidSelectionTrace);
    }
    Ok(bindings)
}

fn check_compilation_explain_capacity(
    actual: usize,
    maximum: usize,
) -> Result<(), CompilationExplainError> {
    if actual > maximum {
        return Err(CompilationExplainError::CanonicalCapacity);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompilationExplainError {
    InvalidSelectionTrace,
    Empty,
    DuplicateCandidateKey,
    CandidateKeyMismatch,
    CandidateSubjectMismatch,
    DuplicateCandidate,
    CandidateCapacity,
    CanonicalCapacity,
    StaleIdentity,
}

impl fmt::Display for CompilationExplainError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "compilation explain: {self:?}")
    }
}

impl Error for CompilationExplainError {}

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
        ExplainEvent::DeferredTargetRequirement {
            entry,
            predicate,
            required,
            requirement,
        } => {
            let query = requirement.query();
            let provider = query.provider();
            let _ = write!(
                output,
                "feasibility:{}:deferred:entry={entry}:{}:{}={}:query={}@{}:provider={}::{}@{}",
                predicate.as_str(),
                target_requirement_relation_name(requirement.relation()),
                quantity_name(*required),
                required.value(),
                query.key().as_str(),
                availability_phase_name(query.available_at()),
                provider.namespace(),
                provider.name(),
                provider.revision(),
            );
        }
        ExplainEvent::NumericalHonourability {
            dimension,
            arithmetic,
            resolved_type,
            required,
            outcome,
            profile,
        } => render_honourability(
            output,
            dimension,
            *arithmetic,
            resolved_type,
            required,
            outcome,
            profile,
        ),
        // The whole subject on one line, and the fenced domains spelled as a
        // set rather than a count: a reader has to be able to see that "device"
        // is absent, which a number of fenced spaces would hide.
        ExplainEvent::SynchronizationRealization {
            kind,
            execution_scope,
            visibility_scope,
            fences_workgroup,
            fences_device,
            ordering,
            outcome,
        } => {
            let mut fences = Vec::new();
            if *fences_workgroup {
                fences.push("workgroup");
            }
            if *fences_device {
                fences.push("device");
            }
            let _ = write!(
                output,
                "synchronization:{}:arrive={}:publish={}:fence={}:order={}:{}",
                kind.as_str(),
                execution_scope.as_str(),
                visibility_scope.as_str(),
                fences.join("+"),
                ordering.as_str(),
                match outcome {
                    SynchronizationOutcome::Realized { profile } =>
                        format!("realized:profile={}", profile.as_str()),
                    SynchronizationOutcome::Unrealizable { profile } =>
                        format!("unrealizable:profile={}", profile.as_str()),
                    SynchronizationOutcome::Undeclared => "undeclared".to_owned(),
                }
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
        } => render_cost(output, model, basis, terms, *disposition),
        ExplainEvent::Selection { policy, outcome } => {
            let _ = write!(
                output,
                "selection:{}:{}",
                policy.as_str(),
                selection_name(*outcome)
            );
        }
        ExplainEvent::SemanticSelection {
            policy,
            outcome,
            candidate,
        } => {
            let _ = write!(
                output,
                "semantic-selection:{}:{}:request={:016x}",
                policy.as_str(),
                selection_name(*outcome),
                stable_qualifier(&candidate.canonical)
            );
        }
        ExplainEvent::PreferencePruned {
            preferred_contract,
            candidate_contract,
        } => {
            let _ = write!(
                output,
                "contract-preference-pruned:preferred={}:candidate={}",
                preferred_contract.as_str(),
                candidate_contract.as_str()
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
/// Every part is written, including arithmetic dtype and declaring profile,
/// because honourability can differ by dtype and a rejection whose declarer is
/// unnamed is not explainable.
fn render_honourability(
    output: &mut String,
    dimension: &PredicateKey,
    _arithmetic: ArithmeticType,
    resolved_type: &ResolvedValueType,
    required: &ReasonCode,
    outcome: &HonourabilityOutcome,
    profile: &SubjectKey,
) {
    use fmt::Write as _;
    let _ = write!(output, "honourability:{}:", dimension.as_str());
    if let Some(key) = resolved_type.nominal_key() {
        let _ = write!(output, "{key}");
    } else {
        for byte in resolved_type.canonical_encoding().as_bytes() {
            let _ = write!(output, "{byte:02x}");
        }
    }
    let _ = write!(output, ":{}:", required.as_str());
    match outcome {
        HonourabilityOutcome::Honoured { means } => {
            let _ = write!(output, "honoured:{}", means.as_str());
        }
        HonourabilityOutcome::Unhonourable {
            means,
            honoured,
            evidence,
        } => {
            let _ = write!(output, "unhonourable:{}", means.as_str());
            if let Some(honoured) = honoured {
                let _ = write!(output, ":honours={}", honoured.as_str());
            }
            output.push(':');
            evidence.render(output);
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
        ExplainStage::SemanticDischarge => "semantic-discharge",
        ExplainStage::ProgramVerification => "program-verification",
        ExplainStage::ArtifactPlanning => "artifact-planning",
    }
}

const fn disposition_name(disposition: ExplainDisposition) -> &'static str {
    match disposition {
        ExplainDisposition::Admitted => "admitted",
        ExplainDisposition::DeferredAdmitted => "deferred-admitted",
        ExplainDisposition::RejectedIntrinsic => "rejected-intrinsic",
        ExplainDisposition::RejectedNumerical => "rejected-numerical",
        ExplainDisposition::RejectedTarget => "rejected-target",
        ExplainDisposition::DeferredUnsupported => "deferred-unsupported",
        ExplainDisposition::BudgetStopped => "budget-stopped",
        ExplainDisposition::Reported => "reported",
        ExplainDisposition::Retained => "retained",
        ExplainDisposition::DominancePruned => "dominance-pruned",
        ExplainDisposition::HigherCost => "higher-cost",
        ExplainDisposition::PreferencePruned => "preference-pruned",
        ExplainDisposition::NotSelectedTradeoff => "not-selected-tradeoff",
        ExplainDisposition::Selected => "selected",
        ExplainDisposition::CompilerFailure => "compiler-failure",
        ExplainDisposition::NotApplicable => "not-applicable",
    }
}

const fn target_requirement_relation_name(
    relation: TargetPropertyRequirementRelation,
) -> &'static str {
    match relation {
        TargetPropertyRequirementRelation::ObservedAtLeastRequired => "observed-at-least-required",
        TargetPropertyRequirementRelation::ObservedEqualsRequired => "observed-equals-required",
        TargetPropertyRequirementRelation::RequiredImpliesObserved => "required-implies-observed",
    }
}

const fn availability_phase_name(phase: AvailabilityPhase) -> &'static str {
    match phase {
        AvailabilityPhase::CompileProfile => "compile-profile",
        AvailabilityPhase::ArtifactEvidence => "artifact-evidence",
        AvailabilityPhase::LiveDevicePreflight => "live-device-preflight",
        AvailabilityPhase::PreparedKernelPreflight => "prepared-kernel-preflight",
        AvailabilityPhase::LaunchPreflight => "launch-preflight",
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
        SubjectKind::OpaqueCall => "opaque-call",
        SubjectKind::Provider => "provider",
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
        Quantity::Bits(_) => "bits",
        Quantity::Threads(_) => "threads",
        Quantity::Bindings(_) => "bindings",
        Quantity::Operations(_) => "operations",
        Quantity::Registers(_) => "registers",
        Quantity::Nanoseconds(_) => "nanoseconds",
    }
}

const fn cost_disposition_name(disposition: CostDisposition) -> &'static str {
    match disposition {
        CostDisposition::Reported => "reported",
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
    RequirementQuantityMismatch,
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
                    RejectionClass::NotApplicable => 3,
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
        ExplainEvent::DeferredTargetRequirement {
            entry,
            predicate,
            required,
            requirement,
        } => {
            bytes.push(8);
            bytes.extend_from_slice(&entry.to_be_bytes());
            push_slice(bytes, predicate.as_str().as_bytes());
            encode_quantity(bytes, *required);
            push_slice(bytes, &requirement.canonical_bytes());
        }
        ExplainEvent::NumericalHonourability {
            dimension,
            arithmetic,
            resolved_type,
            required,
            outcome,
            profile,
        } => encode_honourability(
            bytes,
            dimension,
            *arithmetic,
            resolved_type,
            required,
            outcome,
            profile,
        ),
        // Event tag `13`, appended: every earlier record keeps its tag and its
        // field layout, so no previously encoded trace's bytes move.
        ExplainEvent::SynchronizationRealization {
            kind,
            execution_scope,
            visibility_scope,
            fences_workgroup,
            fences_device,
            ordering,
            outcome,
        } => {
            bytes.push(13);
            push_slice(bytes, kind.as_str().as_bytes());
            push_slice(bytes, execution_scope.as_str().as_bytes());
            push_slice(bytes, visibility_scope.as_str().as_bytes());
            bytes.push(u8::from(*fences_workgroup));
            bytes.push(u8::from(*fences_device));
            push_slice(bytes, ordering.as_str().as_bytes());
            match outcome {
                SynchronizationOutcome::Realized { profile } => {
                    bytes.push(1);
                    push_slice(bytes, profile.as_str().as_bytes());
                }
                SynchronizationOutcome::Unrealizable { profile } => {
                    bytes.push(2);
                    push_slice(bytes, profile.as_str().as_bytes());
                }
                SynchronizationOutcome::Undeclared => bytes.push(3),
            }
        }
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
        ExplainEvent::SemanticSelection {
            policy,
            outcome,
            candidate,
        } => {
            bytes.push(11);
            push_slice(bytes, policy.as_str().as_bytes());
            bytes.push(match outcome {
                SelectionOutcome::Selected => 1,
                SelectionOutcome::Dominated => 2,
                SelectionOutcome::NotSelectedTradeoff => 3,
                SelectionOutcome::Infeasible => 4,
            });
            push_slice(bytes, &candidate.canonical);
        }
        ExplainEvent::PreferencePruned {
            preferred_contract,
            candidate_contract,
        } => {
            bytes.push(12);
            push_slice(bytes, preferred_contract.as_str().as_bytes());
            push_slice(bytes, candidate_contract.as_str().as_bytes());
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
        CostDisposition::Reported => 4,
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
/// Event tag `10`, then the dimension, arithmetic dtype, required behaviour, a
/// one-byte outcome discriminant with its payload, and the declaring profile.
/// Arithmetic and declarer are inside the encoding because changing either
/// changes the claim.
fn encode_honourability(
    bytes: &mut Vec<u8>,
    dimension: &PredicateKey,
    arithmetic: ArithmeticType,
    resolved_type: &ResolvedValueType,
    required: &ReasonCode,
    outcome: &HonourabilityOutcome,
    profile: &SubjectKey,
) {
    bytes.push(10);
    push_slice(bytes, dimension.as_str().as_bytes());
    bytes.push(arithmetic.tag());
    push_slice(bytes, resolved_type.canonical_encoding().as_bytes());
    push_slice(bytes, required.as_str().as_bytes());
    match outcome {
        HonourabilityOutcome::Honoured { means } => {
            bytes.push(1);
            push_slice(bytes, means.as_str().as_bytes());
        }
        HonourabilityOutcome::Unhonourable {
            means,
            honoured,
            evidence,
        } => {
            bytes.push(2);
            push_slice(bytes, means.as_str().as_bytes());
            match honoured {
                Some(honoured) => {
                    bytes.push(1);
                    push_slice(bytes, honoured.as_str().as_bytes());
                }
                None => bytes.push(0),
            }
            // The complete refusing fact, through the honourability authority's
            // own encoder. Two profiles refusing the same behaviour on different
            // measured builds are two different claims, and a trace identity
            // that could not tell them apart would let one compilation's
            // explanation stand in for the other's.
            push_slice(bytes, &evidence.canonical_bytes());
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
                EvidenceReceiptKind::IndexDomain => 2,
            });
            push_slice(bytes, &receipt.compilation);
            bytes.push(subject_kind_tag(receipt.subject_kind));
            push_slice(bytes, receipt.subject.as_str().as_bytes());
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
        SubjectKind::OpaqueCall => 13,
        SubjectKind::Provider => 14,
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
        ExplainStage::SemanticDischarge => 14,
    }
}

const fn disposition_tag(disposition: ExplainDisposition) -> u8 {
    match disposition {
        ExplainDisposition::Admitted => 1,
        ExplainDisposition::DeferredAdmitted => 16,
        ExplainDisposition::RejectedIntrinsic => 2,
        ExplainDisposition::RejectedNumerical => 3,
        ExplainDisposition::RejectedTarget => 4,
        ExplainDisposition::DeferredUnsupported => 5,
        ExplainDisposition::BudgetStopped => 6,
        ExplainDisposition::Reported => 13,
        ExplainDisposition::Retained => 7,
        ExplainDisposition::DominancePruned => 8,
        ExplainDisposition::HigherCost => 9,
        ExplainDisposition::PreferencePruned => 14,
        ExplainDisposition::NotSelectedTradeoff => 10,
        ExplainDisposition::Selected => 11,
        ExplainDisposition::CompilerFailure => 12,
        ExplainDisposition::NotApplicable => 15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fusion::prove_fused_numerics;
    use crate::region::form_region_candidates;
    use crate::request::{CompilationRequest, verify_planned_request};
    use tiler_ir::program::abi::{
        PreparedEntryTargetRequirement, TargetPropertyKey, TargetPropertyProviderIdentity,
        TargetPropertyQuery, TargetPropertyRequirementRelation,
    };
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
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
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

    #[derive(Clone, Copy)]
    struct DeferredTargetRequirementFixture {
        entry: u32,
        predicate: &'static str,
        required: Quantity,
        property: &'static str,
        relation: TargetPropertyRequirementRelation,
        provider_namespace: &'static str,
        provider_name: &'static str,
        provider_revision: u32,
    }

    fn deferred_target_requirement_event(
        fixture: DeferredTargetRequirementFixture,
    ) -> ExplainEvent {
        let query = TargetPropertyQuery::new(
            TargetPropertyKey::new(fixture.property).unwrap(),
            AvailabilityPhase::PreparedKernelPreflight,
            TargetPropertyProviderIdentity::new(
                fixture.provider_namespace,
                fixture.provider_name,
                fixture.provider_revision,
            )
            .unwrap(),
        )
        .unwrap();
        let requirement =
            PreparedEntryTargetRequirement::new(query, fixture.required.value(), fixture.relation)
                .unwrap();
        ExplainEvent::DeferredTargetRequirement {
            entry: fixture.entry,
            predicate: PredicateKey::new(fixture.predicate).unwrap(),
            required: fixture.required,
            requirement,
        }
    }

    #[test]
    fn explain_vocabulary_is_append_only_and_versioned() {
        assert_eq!(EXPLAIN_SCHEMA_VERSION, 9);
        assert_eq!(EXPLAIN_RENDERER_VERSION, 7);
        assert_eq!(subject_kind_tag(SubjectKind::Alternative), 12);
        assert_eq!(subject_kind_tag(SubjectKind::OpaqueCall), 13);
        assert_eq!(subject_kind_tag(SubjectKind::Provider), 14);
        assert_eq!(disposition_tag(ExplainDisposition::PreferencePruned), 14);
        assert_eq!(disposition_tag(ExplainDisposition::NotApplicable), 15);
        assert_eq!(disposition_tag(ExplainDisposition::DeferredAdmitted), 16);
        assert_eq!(subject_kind_name(SubjectKind::OpaqueCall), "opaque-call");
        assert_eq!(subject_kind_name(SubjectKind::Provider), "provider");
        assert_eq!(
            disposition_name(ExplainDisposition::NotApplicable),
            "not-applicable"
        );
        assert_eq!(
            disposition_name(ExplainDisposition::DeferredAdmitted),
            "deferred-admitted"
        );
        let mut deferred = Vec::new();
        encode_event(
            &mut deferred,
            &deferred_target_requirement_event(DeferredTargetRequirementFixture {
                entry: 0,
                predicate: "threads-per-workgroup",
                required: Quantity::Threads(1),
                property: "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
                relation: TargetPropertyRequirementRelation::ObservedAtLeastRequired,
                provider_namespace: "tiler",
                provider_name: "prepared-entry-properties",
                provider_revision: 1,
            }),
        );
        assert_eq!(deferred[0], 8);
    }

    #[test]
    fn deferred_target_requirement_identity_and_rendering_are_complete() {
        let fixture = DeferredTargetRequirementFixture {
            entry: 0,
            predicate: "threads-per-workgroup",
            required: Quantity::Threads(1),
            property: "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
            relation: TargetPropertyRequirementRelation::ObservedAtLeastRequired,
            provider_namespace: "tiler",
            provider_name: "prepared-entry-properties",
            provider_revision: 1,
        };
        let baseline = deferred_target_requirement_event(fixture);
        assert_eq!(baseline.validate(), Ok(()));
        assert_eq!(baseline.stage(), ExplainStage::TargetFeasibility);
        assert_eq!(baseline.disposition(), ExplainDisposition::DeferredAdmitted);
        let mut baseline_identity = Vec::new();
        encode_event(&mut baseline_identity, &baseline);
        for changed in [
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                entry: 1,
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                predicate: "grid-axis",
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                required: Quantity::Count(1),
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                required: Quantity::Threads(2),
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                relation: TargetPropertyRequirementRelation::ObservedEqualsRequired,
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                property: "tiler.target.prepared-entry.neighbouring-limit.v1",
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                provider_namespace: "neighbour",
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                provider_name: "neighbouring-provider",
                ..fixture
            }),
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                provider_revision: 2,
                ..fixture
            }),
        ] {
            let mut changed_identity = Vec::new();
            encode_event(&mut changed_identity, &changed);
            assert_ne!(baseline_identity, changed_identity);
        }

        let mut rendered = String::new();
        render_event(&mut rendered, &baseline);
        assert_eq!(
            rendered,
            "feasibility:threads-per-workgroup:deferred:entry=0:observed-at-least-required:threads=1:query=tiler.target.prepared-entry.max-threads-per-workgroup.v1@prepared-kernel-preflight:provider=tiler::prepared-entry-properties@1"
        );

        let mut mismatched = baseline;
        let ExplainEvent::DeferredTargetRequirement { required, .. } = &mut mismatched else {
            panic!("fixture is one deferred target requirement");
        };
        *required = Quantity::Threads(2);
        assert_eq!(
            mismatched.validate(),
            Err(ExplainError::RequirementQuantityMismatch)
        );
    }

    #[test]
    fn bit_width_is_not_a_dimensionless_count_in_identity_or_rendering() {
        let event = |quantity| ExplainEvent::Feasibility {
            predicate: PredicateKey::new("device-address-bits").unwrap(),
            outcome: FeasibilityOutcome::Admitted,
            required: quantity,
            available: quantity,
        };
        let count = event(Quantity::Count(64));
        let bits = event(Quantity::Bits(64));
        assert_eq!(count.validate(), Ok(()));
        assert_eq!(bits.validate(), Ok(()));

        let mut count_identity = Vec::new();
        let mut bits_identity = Vec::new();
        encode_event(&mut count_identity, &count);
        encode_event(&mut bits_identity, &bits);
        assert_ne!(
            count_identity, bits_identity,
            "a width in bits shared identity with a dimensionless count"
        );

        let mut rendered = String::new();
        render_event(&mut rendered, &bits);
        assert_eq!(
            rendered,
            "feasibility:device-address-bits:admitted:bits=64:64"
        );
    }

    #[test]
    fn not_applicable_is_only_a_candidate_enumeration_disproof() {
        let event = |stage| ExplainEvent::Check {
            stage,
            assessment: PredicateAssessment::disproved(
                "opaque-call.applicable",
                ReasonCode::new("opaque-call.not-applicable").unwrap(),
                EvidenceBasis::CheckedInvariant,
            )
            .unwrap(),
            rejection: RejectionClass::NotApplicable,
        };
        let valid = event(ExplainStage::CandidateEnumeration);
        assert_eq!(valid.disposition(), ExplainDisposition::NotApplicable);
        assert_eq!(valid.validate(), Ok(()));
        assert_eq!(
            event(ExplainStage::IntrinsicScheduling).validate(),
            Err(ExplainError::InvalidStageEvent)
        );
    }

    /// Builds one refusal from honest declared evidence.
    ///
    /// There is no provenance-free path to an [`ExplainEvent`] refusal, here or
    /// in production: the event carries the exact fact, and a fact exists only
    /// once a declaration has been attributed to a declaring profile.
    fn unhonoured(
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
        source: std::sync::Arc<crate::target::honourability::FactSourceProvenance>,
    ) -> crate::target::honourability::UnhonouredDimension {
        use crate::target::honourability::{
            DeclaredBehaviour, DimensionBehaviour, HonouringMeans, NumericalDimension,
        };
        let required =
            DimensionBehaviour::Transform(tiler_ir::schedule::NumericalPermission::Forbidden);
        crate::target::honourability::UnhonouredDimension::new(
            DeclaredBehaviour::new(
                NumericalDimension::Contraction,
                arithmetic,
                resolved_type,
                required,
                HonouringMeans::Unsupported,
                source,
            )
            .attributed_to(crate::target::feasibility::TargetProfileIdentity::new(
                "tiler.test.profile.v1",
            )),
            required,
            Some(DimensionBehaviour::Transform(
                tiler_ir::schedule::NumericalPermission::Permitted,
            )),
        )
    }

    #[test]
    fn honourability_complete_dtype_is_canonical_identity() {
        let event = |cause: &crate::target::honourability::UnhonouredDimension| {
            ExplainEvent::NumericalHonourability {
                dimension: PredicateKey::new("numerics.contraction").unwrap(),
                arithmetic: cause.arithmetic(),
                resolved_type: cause.resolved_type().clone(),
                required: ReasonCode::new("forbidden").unwrap(),
                outcome: HonourabilityOutcome::Unhonourable {
                    means: ReasonCode::new("unsupported").unwrap(),
                    honoured: Some(ReasonCode::new("permitted").unwrap()),
                    evidence: cause.evidence(),
                },
                profile: SubjectKey::new("tiler.test.profile.v1").unwrap(),
            }
        };
        let source = crate::target::honourability::governed_profile_source();
        let f16 = event(&unhonoured(
            ArithmeticType::F16,
            ResolvedValueType::nominal(
                tiler_ir::semantic::TypeKey::new("tiler", "f16", 1).unwrap(),
            ),
            std::sync::Arc::clone(&source),
        ));
        let f32 = event(&unhonoured(
            ArithmeticType::F32,
            F32::resolved_type(),
            std::sync::Arc::clone(&source),
        ));
        let neighbouring_f32 = event(&unhonoured(
            ArithmeticType::F32,
            ResolvedValueType::nominal(
                tiler_ir::semantic::TypeKey::new("test", "neighbouring-f32", 1).unwrap(),
            ),
            source,
        ));
        let mut f16_bytes = Vec::new();
        let mut f32_bytes = Vec::new();
        let mut neighbouring_f32_bytes = Vec::new();
        encode_event(&mut f16_bytes, &f16);
        encode_event(&mut f32_bytes, &f32);
        encode_event(&mut neighbouring_f32_bytes, &neighbouring_f32);
        assert_ne!(
            f16_bytes, f32_bytes,
            "two dtype-specific honourability claims shared canonical identity"
        );
        assert_ne!(
            f32_bytes, neighbouring_f32_bytes,
            "same-arithmetic neighbouring resolved types shared explain identity"
        );
        let mut rendered = String::new();
        render_event(&mut rendered, &f16);
        assert!(rendered.contains("tiler::f16@1"));
    }

    /// An unhonourable record spells and identifies its complete provenance.
    ///
    /// The record is what a reader acts on, so a refusal whose authority,
    /// validity scope, and measured builds and environments are absent from
    /// both the rendering and the identity is not explainable: two profiles
    /// refusing the same behaviour on different measured builds would produce
    /// one trace, and a reader could not tell which one it was reading.
    #[test]
    fn an_unhonourable_record_carries_the_complete_refusal_provenance() {
        use crate::request::StrictF32NumericalContract;
        use crate::target::TargetProfile;
        use crate::target::feasibility::{FeasibilityOutcome, RejectionCause};
        use crate::target::honourability::{
            UnhonouredDimension, governed_profile_source, measured_profile_source,
        };

        fn refusal(
            source: std::sync::Arc<crate::target::honourability::FactSourceProvenance>,
        ) -> UnhonouredDimension {
            let profile = TargetProfile::refusing_preserved_subnormals_for_test(
                "test.explain-refusal.v1",
                source,
            );
            let FeasibilityOutcome::Rejected(rejection) =
                crate::physical::assess_contract(&profile, StrictF32NumericalContract::governed())
                    .expect("the refusing test profile is intrinsically valid")
            else {
                panic!("a declared refusal disproves a hard predicate");
            };
            let RejectionCause::Numerical(cause) = rejection.representative() else {
                panic!("a contract-only proposal states no capability requirement");
            };
            cause
        }

        fn record(cause: &UnhonouredDimension) -> ExplainEvent {
            ExplainEvent::NumericalHonourability {
                dimension: PredicateKey::new(cause.dimension().key()).unwrap(),
                arithmetic: cause.arithmetic(),
                resolved_type: cause.resolved_type().clone(),
                required: ReasonCode::new(cause.required().key()).unwrap(),
                outcome: HonourabilityOutcome::Unhonourable {
                    means: ReasonCode::new(cause.means().key()).unwrap(),
                    honoured: cause
                        .honoured()
                        .map(|honoured| ReasonCode::new(honoured.key()).unwrap()),
                    evidence: cause.evidence(),
                },
                profile: SubjectKey::new(cause.profile().key()).unwrap(),
            }
        }

        let render = |event: &ExplainEvent| {
            let mut rendered = String::new();
            render_event(&mut rendered, event);
            rendered
        };
        let encode = |event: &ExplainEvent| {
            let mut bytes = Vec::new();
            encode_event(&mut bytes, event);
            bytes
        };

        let baseline = refusal(measured_profile_source("test.probe.v1", "1.0", "build-1"));
        let baseline_event = record(&baseline);
        let baseline_render = render(&baseline_event);
        for expected in [
            "authority=measured-profile",
            "validity=measured-environment",
            "phase=compile-profile",
            "authority-identity=test.probe.v1@1",
            "basis=measurement:contexts=1",
            "code-generator=test-offline-compiler@1.0",
            "env=test-platform/1.0/build-1/test-architecture/test-hardware",
        ] {
            assert!(
                baseline_render.contains(expected),
                "the rendered refusal omitted {expected}: {baseline_render}",
            );
        }

        for (label, perturbed) in [
            ("authority and validity", refusal(governed_profile_source())),
            (
                "authority identity",
                refusal(measured_profile_source(
                    "test.other-probe.v1",
                    "1.0",
                    "build-1",
                )),
            ),
            (
                "compiler build",
                refusal(measured_profile_source("test.probe.v1", "2.0", "build-1")),
            ),
            (
                "execution environment",
                refusal(measured_profile_source("test.probe.v1", "1.0", "build-2")),
            ),
        ] {
            assert_eq!(
                perturbed.required(),
                baseline.required(),
                "{label} changed what the caller required",
            );
            let event = record(&perturbed);
            assert_ne!(
                baseline_render,
                render(&event),
                "{label} left the rendered refusal unchanged",
            );
            assert_ne!(
                encode(&baseline_event),
                encode(&event),
                "{label} left the refusal's canonical identity unchanged",
            );
        }
    }

    #[test]
    fn rejected_typed_feasibility_requires_a_strictly_exceeded_bound() {
        let event = |required, available| ExplainEvent::Feasibility {
            predicate: PredicateKey::new("buffer-bindings").unwrap(),
            outcome: FeasibilityOutcome::Rejected(ReasonCode::new("target-infeasible").unwrap()),
            required: Quantity::Bindings(required),
            available: Quantity::Bindings(available),
        };
        assert_eq!(event(u64::from(u32::MAX), 2).validate(), Ok(()));
        assert_eq!(
            event(2, u64::from(u32::MAX)).validate(),
            Err(ExplainError::InvalidQuantityRelation)
        );
        assert_eq!(
            event(2, 2).validate(),
            Err(ExplainError::InvalidQuantityRelation)
        );
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

    fn finish_semantic_selection_trace(
        request: &VerifiedTargetRequest,
        candidates: &[(&str, SelectionOutcome, &VerifiedTargetRequest)],
        selected: &str,
    ) -> VerifiedExplainTrace {
        let mut writer = ExplainWriter::new(request).unwrap();
        for (key, outcome, candidate) in candidates {
            let subject = writer.subject(SubjectKind::Alternative, key).unwrap();
            writer
                .note_semantic_selection(subject, candidate, *outcome, None)
                .unwrap();
        }
        let keys = candidates
            .iter()
            .map(|(key, _, _)| *key)
            .collect::<Vec<_>>();
        writer.finish_semantic_portfolio(&keys, selected).unwrap()
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
                // Rebaselined on the numerical-policies merge. Both parents
                // had moved this pin independently — main for the registry
                // identity interning (domain tag v2), the branch for the
                // contract widening from four dimensions to the eleven
                // `docs/numerical-semantics.md` names, each keyed by its
                // arithmetic type. A pinned digest must be recomputed on the
                // merged tree rather than taken from either side; regenerate
                // with:
                //   cargo nextest run -p tiler-compiler -E \
                //     'test(deterministic_trace_is_sealed_and_rendered_separately)'
                // and read the `left` value the assertion reports.
                //
                // Rebaselined when governed operation definitions gained typed
                // semantic-precondition declarations. The request subject
                // reaches the definition projection, so preserving the prior
                // qualifier would erase a runtime-relevant semantic contract.
                // Rebaselined again when registry v7 bound the host-sealed
                // static-evidence authority into registry provenance.
                // Rebaselined when each target honourability declaration gained
                // its structured, versioned source provenance and the request
                // subject encoded the canonical deduplicated source table.
                // Rebaselined from `f11ac7524742db81` when the complete target
                // declaration gained distinct external normative provenance
                // and sparse quantitative rows, each bound to its own source.
                // The governed profile states the same seven facts, but the
                // request now commits to every row binding rather than one
                // shared-source shortcut.
                // Rebaselined again when each numerical row began binding its
                // complete resolved dtype subject and governed F32 dispatch
                // became an explicit phase-qualified target fact. Both are
                // request-subject facts, so retaining the old digest would
                // collide targets with different feasibility or dispatch.
                // Rebaselined for provenance schema v3 and the complete/source
                // descriptor domains v6/v4. Old `a3aff309fd6cc3ba`; regenerate
                // with the focused nextest command above and take `left`.
                // Rebaselined when complete profile declaration v7 replaced
                // the nested checked descriptor and second provenance table
                // with one canonical source table shared by quantitative,
                // numerical, and exact-dispatch rows.
                // Rebaselined when the invented barrier-count target axis was
                // removed and complete profile declaration v8 reserved its
                // retired tag. The request subject commits to that target
                // declaration, so preserving the old qualifier would erase an
                // identity-affecting target contract change.
                // Rebaselined when complete profile declaration v9 replaced
                // conflated index width with operation-complete u64 arithmetic
                // support and an explicitly absent address-width authority.
                // Rebaselined when the workgroup limit moved from a compile-time
                // global fact to a prepared-entry target requirement. The
                // request subject now binds the complete query and its provider.
                // Rebaselined from `83b9baadbea45e19` when the governed
                // deterministic budgets widened to three regions and four
                // buffers for the split-reduction program shape. Every budget is
                // inside the request subject, so this digest *must* move: two
                // requests admitting different program shapes are different
                // requests, and a value that survived would mean the subject
                // reached the budgets it names without reaching their values.
                // Rebaselined from `09d719dd4c2c2f37` when the standard semantic
                // registry began registering the complete accepted built-in
                // dtype catalog. The request subject covers the frozen semantic
                // registry snapshot, and that snapshot encodes every registered
                // definition, so admitting thirty-one further identities must
                // move this digest: two requests whose semantic authority admits
                // different value types are different requests. The same change
                // gave the strict-affine scheme's normative reference its own
                // key, which is a definition contract inside the same snapshot.
                // Nothing about the trace's own content changed, which is why
                // the two record lines below are unchanged.
                // Rebaselined from `928bbdb50eb505ed` when the standard semantic
                // registry began registering `tiler::strict-tensor-contraction-f32@1`.
                // The request subject covers the frozen semantic registry
                // snapshot, which encodes every registered *operation* along with
                // its schema, facts, and conformance identity, so admitting one
                // further operation family must move this digest exactly as
                // admitting thirty-one further value types did. It moves a second
                // time over: the lowering-registry identity in the same subject
                // binds the semantic snapshot the capability registry froze
                // against. No lowering capability was registered for the new
                // family, so a contraction still fails closed at resolution, and
                // the trace's own two record lines are again unchanged.
                // Rebaselined from `e1e95ea1d50a918f` when the standard semantic
                // registry began registering `tiler::reindex-f32@1` and
                // `tiler::broadcast-f32@1`. Both halves of the subject move this
                // time: the semantic snapshot admits two further operation
                // families, and — unlike the contraction — the lowering registry
                // admits an index-access capability for each, so the
                // lowering-registry identity the subject binds changes as well.
                // The trace's own two record lines are once more unchanged,
                // because nothing about explain's content moved.
                // Rebaselined from `bddeaf899938ede4` when the governed target
                // profile raised its declared buffer-binding bound from two to
                // four, so a region reading several input tensors is feasible on
                // it. The subject binds the complete target declaration, so a
                // declared capability moving must move this digest: two requests
                // compiled against targets that admit different signatures are
                // different requests. The trace's own two record lines are
                // unchanged again — the fixture's program has one input and one
                // output, and nothing about explain's content moved.
                // Rebaselined from `0b7759de2d9b5756` when the strict-affine
                // scale domain narrowed to positive *normal* f32. Two facts
                // inside the frozen semantic registry snapshot moved together:
                // the strict-affine value contract gained its scale-domain
                // field, and `assemble-` and `quantize-strict-affine` gained
                // semantic-precondition declarations for that domain. The
                // subject covers the snapshot, which encodes every registered
                // definition's contract and its precondition declarations, so
                // this digest *must* move — a request whose semantic authority
                // admits a different set of scale values is a different
                // request. The trace's own two record lines are unchanged;
                // nothing about explain's content moved.
                // Rebaselined from `bae4788d2fc79631` when the standard semantic
                // registry began registering `tiler::constant-bf16@1`,
                // `tiler::multiply-bf16@1`, and `tiler::add-bf16@1`. Only the
                // semantic half of the subject moves: the snapshot admits three
                // further operation families, each with its own schema, facts,
                // and normative reference. Nothing on the compiler side moved
                // with it — no capability row, no lowering capability, and no
                // target declaration names bf16 — so a bf16 program still fails
                // closed everywhere past the semantic layer, and this digest is
                // the only pin that moved. The trace's own two record lines are
                // unchanged; nothing about explain's content moved.
                // Rebaselined from `b610aff7e1907c00` when the standard semantic
                // registry began registering `tiler::silu-f32@1`. Both halves of
                // the subject move this time, which is what distinguishes this
                // step from the bf16 one above. The semantic half moves because
                // the snapshot admits one further operation family whose facts
                // carry a complete ADR 0042 accuracy contract — the first
                // registered definition that does — so the definition projection
                // folds a resolved tolerance, a metric key, a domain, and four
                // exceptional-value rules that no earlier snapshot contained. The
                // compiler half moves because the governed scalar registry gained
                // `tiler.scalar::divide-f32@1` and `tiler.scalar::exp-f32@1` and
                // the governed lowering capabilities gained a seventh row. The
                // trace's own two record lines are unchanged; nothing about
                // explain's content moved.
                // Rebaselined from `50c735514f5d51ca` when the standard semantic
                // registry began registering `tiler::rms-norm-f32@1`. Only the
                // *semantic* half of the subject moves this time, which is the
                // difference from the activation's step above: the snapshot
                // admits one further family whose facts carry a second resolved
                // ADR 0042 contract — and one stated in a different contract
                // form, `Faithful` rather than a ULP bound, so the definition
                // projection folds a shape no earlier snapshot contained. Its
                // schema also carries the first `FloatBits` attribute in the
                // registry, the exact `eps` payload. Nothing on the compiler side
                // moved with it: the governed scalar registry gained no key, the
                // governed lowering capabilities gained no row, and the
                // normalization's fusion role and capability row are not part of
                // this subject. The trace's own two record lines are unchanged;
                // nothing about explain's content moved.
                // Rebaselined from `b8ffa37f3d2dc86b` when the governed lowering
                // capabilities gained an eighth row, for
                // `tiler::strict-tensor-contraction-f32@1`. Only the *compiler*
                // half of the subject moves, which is the exact inverse of the
                // step that first registered that family: the semantic snapshot
                // already admitted the contraction and did not move again, and
                // the lowering-registry identity the subject binds now covers one
                // further index-access capability. The governed scalar registry
                // gained no key — the emission reaches `multiply-f32` and
                // `add-f32`, both already registered. The trace's own two record
                // lines are unchanged; nothing about explain's content moved.
                // Rebaselined from `4d9f4773575b6679` when the target profile
                // gained its synchronization-realization declaration. Only the
                // *compiler* half of the subject moves, and it moves even though
                // the governed profile declares no realization: the complete
                // declaration stepped to
                // `tiler.target-profile.declaration.v11`, whose row family writes
                // its own domain separator and a count, so "this target says
                // nothing about synchronization" is now a recorded fact rather
                // than an absence recoverable from bytes that never stated it.
                // Two requests compiled against a target that has been asked and
                // one that has not are different requests. The semantic half did
                // not move — no operation family, contract, or dtype changed —
                // and the trace's own two record lines are unchanged, because a
                // program requiring no synchronization emits no record.
                // Rebaselined from `1ac2bf9aeef5d035` when the standard semantic
                // registry began registering `tiler::softmax-f32@1`. Only the
                // *semantic* half of the subject moves, as it did for the
                // normalization: the snapshot admits one further family whose
                // facts carry a third resolved ADR 0042 contract — a ULP bound
                // like the activation's, but over a domain closed at zero rather
                // than in the exponential's overflow band, so the definition
                // projection folds a clause no earlier snapshot contained.
                // Nothing on the compiler side moved with it: the governed
                // scalar registry gained no key, the governed lowering
                // capabilities gained no row, and neither the softmax's fusion
                // role nor its capability row nor the third installed elementary
                // realization is part of this subject. The trace's own two
                // record lines are unchanged; nothing about explain's content
                // moved.
                // Rebaselined from `a532d35f0cfdd29a` when the request boundary
                // replaced its three whole-program templates with the general
                // occurrence recognizer. Only the *request* half of the subject
                // moves — no semantic definition, capability, or target
                // declaration changed — and it moves because the recognized
                // program is what the subject encodes: the serial-sum arm's two
                // constant fields became the recognized prologue expression, the
                // pointwise arm's fixed leaf triple became the general node run,
                // the serial-sum arm gained its first sub-tag, and the enclosing
                // domain stepped to `tiler.compiler.request-subject.v3`. Two
                // requests whose recognized programs differ are different
                // requests, and a digest that survived a change to *what the
                // subject records about the program* would mean the qualifier
                // reached the strategy without reaching its content. The same
                // change moved the governed `buffers` budget from four to six,
                // which is inside the subject for the reason the split-program
                // widening already recorded. The trace's own two record lines are
                // unchanged; nothing about explain's content moved.
                // Rebaselined from `701c39d4a41e1a22` when the numerical contract
                // became a composed dimension vector and its key became the
                // canonical injective encoding of that vector under
                // `tiler.contract.f32.v2`. The resolved contract and the caller's
                // stated preference are both inside the request subject, and the
                // key is encoded there beside the dimensions it names, so every
                // request's qualifier moves — including this one, whose contract
                // is the strict resolution and whose dimension values did not
                // change. That is the intended consequence of an identity-domain
                // step rather than collateral: a subject minted under the four
                // hand-written names described a contract vocabulary that no
                // longer exists, so a cache or a trace holding one must miss
                // rather than match. The trace's own two record lines are
                // unchanged; nothing about explain's content moved.
                // Moved again on 2026-08-02 by `tiler::concatenate-f32@1`. The
                // request subject folds the registry snapshot, so admitting one
                // further semantic family moves every request's qualifier — the
                // ledger sentence above already said an admitted family must,
                // and this is that sentence being kept rather than a surprise.
                // Recomputed from an observed run on the merged tree, not copied
                // from the branch that added the family.
                // Moved again on 2026-08-03 when the additive relation replaced
                // concatenate's interim normative wording. The request subject
                // folds the complete registered definition, not only its key,
                // so that definition change must move the qualifier even though
                // the concatenate key itself stayed fixed. Recomputed from this
                // merged tree after the full gate exposed the exact blast radius.
                // Moved from `7e413a7d10b92e3b` when the request subject began
                // binding the independently frozen semantic-realization
                // authority and stepped from `request-subject.v3` to `v4`.
                // Lowering resolution alone cannot distinguish two installed
                // realization authorities, so retaining the old digest would permit
                // a request authenticated under one law set to replay under
                // another. Recomputed from this complete branch tree. Moved
                // again when the authority became an operation-bound immutable
                // law snapshot that lowering installers cannot replace.
                // Moved again when strict-affine U4 dequantization gained its
                // governed scalar definition, semantic realization-law row,
                // and lowering capability. The request subject binds all three
                // frozen authorities, so even this unrelated multiply request
                // must miss an authority snapshot that predates the new row.
                // Recomputed from this complete branch tree by first observing
                // the exact failing value, never copied from another branch.
                // Moved from `c91fc7c907eed554` when the recognized program
                // became one implementable region partition *per ordered named
                // output* and the request subject stepped from
                // `request-subject.v4` to `v5` to length-frame that list. Only
                // the *request* half of the subject moves — no semantic
                // definition, capability, target declaration, or budget changed,
                // and this fixture's program still declares one output whose
                // recognized arm encodes exactly the bytes it did. What moved is
                // that the arm is now preceded by a count, which is the whole
                // content of the domain step: a subject minted under `v4`
                // described a recognition that could name only one output, so a
                // cache or a trace holding one must miss rather than match.
                // Recomputed on this branch tree from the observed failing
                // value; it is the only pinned identity the request subject
                // reaches, every other request-subject assertion in the corpus
                // being relational. The trace's own two record lines are
                // unchanged; nothing about explain's content moved.
                "tiler-explain-v7 request=45467875b9574962\n",
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
        let subject = writer
            .subject(SubjectKind::KernelProgram, "plan:reported")
            .unwrap();
        assert_eq!(
            writer.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![subject.clone()],
                ExplainEvent::Check {
                    stage: ExplainStage::Costing,
                    assessment: PredicateAssessment::proven(
                        "cost.reported",
                        EvidenceBasis::CheckedInvariant
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid
                },
                Vec::new()
            ),
            Err(ExplainError::InvalidStageEvent)
        );
        assert!(
            writer
                .push_detail(
                    RuleRef::builtin("cost.reported").unwrap(),
                    vec![subject],
                    ExplainEvent::CostAssessment {
                        model: CostModelKey::new("cost.reported").unwrap(),
                        basis: EvidenceBasis::Assumption,
                        terms: vec![
                            CostTerm::new("compile-time", Quantity::Nanoseconds(1)).unwrap()
                        ],
                        disposition: CostDisposition::Reported,
                    },
                    Vec::new(),
                )
                .is_ok(),
            "a non-pruning report is admitted through the costing event only"
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
    fn contract_preference_pruning_is_never_reported_as_cost() {
        let request = request(2.0);
        let mut writer = ExplainWriter::new(&request).unwrap();
        let subject = writer
            .subject(SubjectKind::Alternative, "semantic:pruned")
            .unwrap();
        writer
            .push_detail(
                RuleRef::builtin("semantic.contract-preference").unwrap(),
                vec![subject],
                ExplainEvent::PreferencePruned {
                    preferred_contract: ReasonCode::new("contract.preferred").unwrap(),
                    candidate_contract: ReasonCode::new("contract.pruned").unwrap(),
                },
                Vec::new(),
            )
            .unwrap();
        let trace = finish_test_trace(writer);
        let record = trace
            .records()
            .iter()
            .find(|record| matches!(record.event(), ExplainEvent::PreferencePruned { .. }))
            .unwrap();
        assert_eq!(
            record.event().disposition(),
            ExplainDisposition::PreferencePruned
        );
        let rendered = trace.render();
        assert!(rendered.contains("selection preference-pruned"));
        assert!(!rendered.contains("selection higher-cost"));
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

        let mut stale_schema = trace.clone();
        stale_schema.schema_version = 2;
        assert_eq!(stale_schema.verify(), Err(ExplainError::StaleIdentity));

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

    #[test]
    fn compilation_explain_canonically_binds_selection_and_candidate_traces() {
        let first_request = request(2.0);
        let second_request = request(3.0);
        let first = finish_test_trace(ExplainWriter::new(&first_request).unwrap());
        let second = finish_test_trace(ExplainWriter::new(&second_request).unwrap());
        let selection_request = request(4.0);
        let selection = finish_semantic_selection_trace(
            &selection_request,
            &[
                ("semantic:first", SelectionOutcome::Selected, &first_request),
                (
                    "semantic:second",
                    SelectionOutcome::Dominated,
                    &second_request,
                ),
            ],
            "semantic:first",
        );
        let forward = VerifiedCompilationExplain::from_traces(
            selection.clone(),
            vec![
                (SubjectKey::new("semantic:first").unwrap(), first.clone()),
                (SubjectKey::new("semantic:second").unwrap(), second.clone()),
            ],
        )
        .unwrap();
        let reversed = VerifiedCompilationExplain::from_traces(
            selection,
            vec![
                (SubjectKey::new("semantic:second").unwrap(), second),
                (SubjectKey::new("semantic:first").unwrap(), first),
            ],
        )
        .unwrap();

        assert_eq!(forward.semantic_candidate_count(), 2);
        assert_eq!(forward.identity(), reversed.identity());
        assert_eq!(forward.render(), reversed.render());
        assert!(forward.verify().is_ok());
        assert!(
            forward
                .render()
                .starts_with("tiler-compilation-explain-v1 semantic-candidates=2\n")
        );
        assert_eq!(
            forward
                .render()
                .matches("tiler-explain-v7 request=")
                .count(),
            3,
            "the top-level selection and both complete candidate traces render",
        );
    }

    #[test]
    fn compilation_explain_rejects_incomplete_or_ambiguous_bindings() {
        let first_request = request(2.0);
        let second_request = request(3.0);
        let third_request = request(4.0);
        let first = finish_test_trace(ExplainWriter::new(&first_request).unwrap());
        let second = finish_test_trace(ExplainWriter::new(&second_request).unwrap());
        let third = finish_test_trace(ExplainWriter::new(&third_request).unwrap());
        let selection_request = request(5.0);
        let selection = finish_semantic_selection_trace(
            &selection_request,
            &[
                ("semantic:a", SelectionOutcome::Selected, &first_request),
                ("semantic:b", SelectionOutcome::Dominated, &second_request),
            ],
            "semantic:a",
        );
        let failure = ExplainWriter::new(&request(2.0))
            .unwrap()
            .finish_failure(
                FailureDescriptor::new(
                    ExplainStage::Selection,
                    "selection-failed",
                    SubjectKind::Alternative,
                    "alternative:failed",
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        assert_eq!(
            VerifiedCompilationExplain::from_traces(
                failure.clone(),
                vec![(SubjectKey::new("semantic:a").unwrap(), failure)],
            ),
            Err(CompilationExplainError::InvalidSelectionTrace),
        );
        assert_eq!(
            VerifiedCompilationExplain::from_traces(selection.clone(), Vec::new()),
            Err(CompilationExplainError::Empty),
        );
        assert_eq!(
            VerifiedCompilationExplain::from_traces(
                selection.clone(),
                vec![
                    (SubjectKey::new("semantic:a").unwrap(), first.clone()),
                    (SubjectKey::new("semantic:a").unwrap(), second.clone()),
                ],
            ),
            Err(CompilationExplainError::DuplicateCandidateKey),
        );
        assert_eq!(
            VerifiedCompilationExplain::from_traces(
                selection.clone(),
                vec![
                    (SubjectKey::new("semantic:a").unwrap(), first.clone()),
                    (SubjectKey::new("semantic:b").unwrap(), first.clone()),
                ],
            ),
            Err(CompilationExplainError::DuplicateCandidate),
        );
        assert_eq!(
            VerifiedCompilationExplain::from_traces(
                selection.clone(),
                vec![
                    (SubjectKey::new("semantic:a").unwrap(), first.clone()),
                    (SubjectKey::new("semantic:c").unwrap(), second.clone()),
                ],
            ),
            Err(CompilationExplainError::CandidateKeyMismatch),
        );
        assert_eq!(
            VerifiedCompilationExplain::from_traces(
                selection,
                vec![
                    (SubjectKey::new("semantic:a").unwrap(), second),
                    (SubjectKey::new("semantic:b").unwrap(), third),
                ],
            ),
            Err(CompilationExplainError::CandidateSubjectMismatch),
        );

        let shared = std::sync::Arc::new(first);
        let candidate = SemanticCandidateExplain {
            key: SubjectKey::new("semantic:a").unwrap(),
            trace: std::sync::Arc::clone(&shared),
        };
        assert_eq!(
            VerifiedCompilationExplain::assemble(
                std::sync::Arc::clone(&shared),
                vec![candidate.clone(); MAX_COMPILATION_EXPLAIN_CANDIDATES.saturating_add(1)],
                SelectionBinding::Singleton,
            ),
            Err(CompilationExplainError::CandidateCapacity),
        );
        assert!(
            matches!(
                encode_compilation_explain_with_capacity(
                    &shared,
                    &[candidate],
                    SelectionBinding::Singleton,
                    0,
                ),
                Err(CompilationExplainError::CanonicalCapacity),
            ),
            "canonical encoding itself enforces the byte ceiling",
        );
        assert!(
            check_compilation_explain_capacity(
                MAX_COMPILATION_EXPLAIN_CANONICAL_BYTES,
                MAX_COMPILATION_EXPLAIN_CANONICAL_BYTES,
            )
            .is_ok(),
            "the exact canonical byte ceiling is admitted",
        );
    }

    #[test]
    fn compilation_explain_identity_detects_tampering() {
        let trace = finish_test_trace(ExplainWriter::new(&request(2.0)).unwrap());
        let mut explain = VerifiedCompilationExplain::one_candidate(trace);
        assert!(explain.verify().is_ok());
        explain.canonical_identity[0] ^= 1;
        assert_eq!(
            explain.verify(),
            Err(CompilationExplainError::StaleIdentity),
        );
    }
}
