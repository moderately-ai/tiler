#![allow(
    dead_code,
    reason = "the explain authority itself is on the compile path; what stays unconstructed is the reserved evidence, quantity, disposition, and subject vocabulary the bounded profile does not yet produce, plus the presentation renderer, which only a trace consumer calls"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(test)]
use std::cell::RefCell;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{
    AvailabilityPhase, PreparedEntryTargetRequirement, TargetPropertyRequirementRelation,
};
use tiler_ir::schedule::ArithmeticType;
use tiler_ir::semantic::{ProviderIdentity, ResolvedValueType};

use crate::fusion::FusionNumericalProof;
use crate::request::{BudgetResource, LoweringProviderIdentity, VerifiedTargetRequest};
use crate::target::honourability::NumericalRefusalEvidence;

// The two numbers below version different things, under one rule: a version
// steps when something a reader already had changes, and does not step when
// something new merely becomes expressible.
//
// `EXPLAIN_SCHEMA_VERSION` versions the canonical trace *encoding*, so its job
// is injectivity — distinct traces never share bytes, and bytes written under
// one version are never read as a different value under another. It steps when
// a change moves or reinterprets a previously encodable record's bytes. It does
// *not* step when a fresh event, subject, disposition, or quantity tag is
// appended under the existing per-tag framing: every earlier record keeps its
// tag and its field layout, so a reader that reaches the new tag is reading a
// record the earlier vocabulary could not express, never an earlier record
// under a new interpretation. That is the rule the sibling identity domains
// state at their own tag sites (`tiler-ir`'s schedule, kernel, and program
// models; `tiler-artifact`'s stage key), and the rule `request.rs`'s
// `canonical_explain_subject_bytes` states in the negative when it steps to
// `v5` because "the per-tag injectivity argument that would license the cheaper
// option does not close".
//
// `EXPLAIN_RENDERER_VERSION` versions the *presentation*, so its job is that a
// change a reader of an existing trace can see is announced. The same rule
// takes the same shape: it steps when an existing record's spelling changes,
// and not when a record type the earlier vocabulary could not produce receives
// its first spelling, because no trace renders differently for it.
//
// So a version does not promise the vocabulary, and this is the inference to
// refuse: two traces sealed under one schema version by different builds may
// differ in which tags they can contain, and a reader must never derive a tag
// set from a version. The ledger below is the vocabulary's only authority.
//
// Nothing depends on the promise being refused, which is what makes the rule
// safe here rather than merely consistent with its siblings. Every consumer of
// `EXPLAIN_SCHEMA_VERSION` is in this file, and none reads it as a vocabulary:
// `push_trace_preamble` folds the value into the identity bytes; `seal` stores
// it on the sealed trace; `VerifiedExplainTrace::verify` compares it — the only
// comparison there is — against this same build's constant, so it can detect a
// stale identity and nothing else; and the pin in
// `explain_vocabulary_is_append_only_and_versioned` holds it to this ledger. No
// decoder exists, here or anywhere: a trace is never serialized, never embedded
// in an artifact envelope, never cached, and leaves the crate only as an opaque
// `VerifiedCompilationExplain` and a rendered string that ADR 0074 and
// `docs/compiler/optimizer.md` both refuse as a parse target. Should the
// trigger those documents name ever fire and a second crate have to read
// canonical traces, an appended tag's payload is one a decoder cannot frame or
// skip, so it fails closed on the unknown tag — which is the outcome a version
// step would have bought, reached without one.
//
// Ledger, newest first. A step marked *forced* moved a previously encodable
// record's bytes or an existing record's spelling; one marked *unforced* did
// not, and is labelled so this file's history is not read as its rule.
//
// - Event tag 15, the deferred subgroup-width confirmation, with its first
//   `feasibility:subgroup-width-confirmation:` spelling: appended under the
//   already-published v11/v9 and correctly moving neither, by the rule above.
//   Capability tag 8 keeps its bytes exactly — the subgroup arm is a record the
//   earlier vocabulary could not express, carried whole (width, arithmetic,
//   transfer, entry, complete requirement) rather than disguised as an axis.
// - Schema v11, *forced*: an opaque-call proposal's existing binding subject
//   now writes the exact full access-list coordinate rather than an input role
//   ordinal. The same parameter names can therefore bind different local reads
//   without colliding, and output and intermediate positions remain
//   distinguishable. Renderer v9, *forced*: the existing binding spelling moves
//   from `input#N` to `access#N`. The composite schema and renderer stay v1 for
//   the same reason as the provider-identity step below: they frame the complete
//   nested trace and do not duplicate its binding fields.
// - Schema v10, *forced*: every previously encodable rule head and SoundProof
//   receipt moved from a flattened `namespace.name` provider key plus revision
//   to an explicit authority-class tag. The compiler arm writes the governed
//   revision constant; the registered arm length-frames namespace and name and
//   writes revision. Distinct structured identities that collided under the
//   dotted join, and the compiler authority versus a registered
//   `tiler`/`compiler` identity, therefore become distinct bytes. Renderer v8,
//   *forced*: every existing record's `provider=` spelling now carries the
//   authority class and the unambiguous `namespace::name@revision` display.
//   The composite renderer header stays v1: it versions the wrapping spelling
//   (`top-level-selection`, `semantic-candidate`), and each nested trace
//   retains its own `tiler-explain-vN` header, so nested provider spelling is
//   announced there. The composite schema stays v1 because it length-frames
//   nested trace identities and does not duplicate provider fields.
// - Event tag 13, the complete synchronization-realization subject, with
//   renderer v7's `synchronization:` line: appended under the already-published
//   v9/v7 and correctly moving neither, by the rule above.
// - Schema v9, *forced*: the complete refusing honourability fact — declared
//   behaviour, means, availability phase, authority, validity scope, versioned
//   authority identity, and governed-guarantee or measured
//   compiler-build/environment basis — joined every unhonourable record inside
//   event tag 10's payload, so records that already encoded moved. Under v8 two
//   profiles refusing the same behaviour on different measured builds produced
//   identical trace identities. Renderer v7, *forced*: it respells records that
//   already rendered.
// - Schema v8 and renderer v6, *unforced*: exact prepared-entry deferred target
//   requirements arrived as event tag 8, disposition tag 16, and a first
//   spelling. Nothing earlier moved, so by the rule above neither number needed
//   to step. This is the closest precedent to event tag 13 and it is history,
//   not authority.
// - Schema v7 and renderer v5, *unforced*: the bits quantity used for exact
//   widths arrived as quantity kind 8 and a first unit spelling, with no
//   existing spelling changed. Same shape as v8.
// - Schema v6, *forced*: the complete resolved dtype joined numerical
//   honourability inside tag 10's payload. The renderer correctly did not step
//   — v4 already published the nominal dtype spelling.
// - Schema v5 and renderer v4, *forced*: the arithmetic dtype joined numerical
//   honourability inside tag 10's payload and changed its spelling. The same
//   landing also appended the opaque-call and provider subject kinds and the
//   NotApplicable rejection class and disposition, which alone would have
//   stepped nothing.
//
// Every tag assigned at v4 retains its v4 value, and every addition since has
// taken a value above the range then in use — verified by diffing the stage,
// disposition, subject-kind, and quantity-kind tables and the event tags
// against the v4 tree. Event tag 9 is unused: it named the omitted-record
// summary that the complete-or-refused trace contract removed at v1. The gap is
// history rather than a reservation.
// - Renderer v10, *forced*, schema unmoved: fact-source provenance stepped its
//   own schema from 3 to 4, so every rendered source line's `source-schema=`
//   spelling changed, the compile-profile measurement basis now renders as
//   `basis=compile-profile-measurement` with a `compilation-selection=` hex
//   run per context, and the governed/external triples render under the closed
//   validation. The trace schema did not move: a rendered source is one framed
//   payload whose own schema word announces itself.
pub(crate) const EXPLAIN_SCHEMA_VERSION: u32 = 11;
pub(crate) const EXPLAIN_RENDERER_VERSION: u32 = 10;
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

/// Authority named by a retained explain record.
///
/// A closed sum so the compiler's own authority cannot collide with a
/// registered identity, and so a registered identity keeps the namespace/name
/// boundary `ProviderIdentity` already validated. Presentation text is not an
/// arm of this type.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProviderRef {
    Compiler,
    Registered(ProviderIdentity),
}

/// Display spelling of the compiler arm. Presentation only; equality and
/// canonical bytes use the enum discriminant, not this string.
const COMPILER_PROVIDER_DISPLAY: &str = "tiler.compiler";
/// Governed revision of the compiler arm. Callers cannot set it; the encoder
/// and renderer both read this constant.
const COMPILER_PROVIDER_REVISION: u32 = 1;
const PROVIDER_REF_COMPILER: u8 = 1;
const PROVIDER_REF_REGISTERED: u8 = 2;

impl ProviderRef {
    pub(crate) fn builtin() -> Self {
        Self::Compiler
    }

    /// References the provider that lowered one occurrence.
    ///
    /// The retained revision is the *provider's* output-affecting revision, not
    /// the capability revision: a `ProviderRef` names an authority, and ADR 0072
    /// keeps a provider's identity separate from the revisions of the individual
    /// capabilities it registers.
    pub(crate) fn lowering(provider: &LoweringProviderIdentity) -> Self {
        Self::registered(provider.provider())
    }

    /// References a registered provider by its already-validated identity.
    ///
    /// The identity is retained whole so namespace, name, and revision stay
    /// distinct. Construction cannot refuse a legal `ProviderIdentity`.
    pub(crate) fn registered(provider: &ProviderIdentity) -> Self {
        Self::Registered(provider.clone())
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

/// How one transient-residency predicate resolved against a target's budget.
///
/// A fourth vocabulary, and deliberately not a third arm of
/// [`FeasibilityOutcome`]. That one is two-valued because the record carrying it
/// holds a *required* and an *available* quantity and validates the outcome
/// against their comparison — so every value it can take is a statement about
/// two known numbers. This predicate's distinguishing case is that the second
/// number does not exist: no target profile in the tree declares a
/// transient-memory limit, so the budget is absent rather than large or small.
/// Encoding that absence as an `available` of zero would manufacture a refusal,
/// and as an `available` of `u64::MAX` would manufacture an admission; both
/// invent an authority no profile supplied.
///
/// Three values for the reason [`SynchronizationOutcome`] and
/// [`HonourabilityOutcome`] have three: the absence of a refusal is not an
/// admission, and a reader must be able to see that nothing ever bounded this
/// plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResidencyOutcome {
    /// A declared budget provably covers the requirement.
    Fits,
    /// A declared budget provably does not cover the requirement.
    Exceeds,
    /// No profile declares a budget. Neither an admission nor a refusal.
    BudgetUndeclared,
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
    /// One subgroup-width confirmation admitted through an exact prepared-entry
    /// query before routing commit.
    ///
    /// The complete atomic subject in one record — width, arithmetic, and
    /// transfer — beside the executable requirement, deliberately not a
    /// [`Self::DeferredTargetRequirement`] with an invented axis key: the width
    /// alone could not explain which arithmetic/transfer realization authorized
    /// the query, and a capability spelling would present the confirmation as
    /// an independently satisfiable quantitative fact. The required width is
    /// derived from the subject and checked equal before the record is
    /// admitted.
    DeferredSubgroupWidthConfirmation {
        /// Zero-based program-entry ordinal whose prepared pipeline is queried.
        entry: u32,
        /// Literal width in lanes, equal to the requirement's required value.
        width: u32,
        /// Exact arithmetic type carried through the transfer.
        arithmetic: ArithmeticType,
        /// Governed key of the required transfer.
        transfer: ReasonCode,
        /// The complete executable relation and versioned query identity.
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
    /// One complete subgroup realization assessed against a target.
    ///
    /// The whole subject in one record, deliberately not three records: a
    /// subject is matched by equality and has no magnitude, and splitting it
    /// across rows would render an explanation from which a reader could
    /// conclude that two of its three dimensions were "admitted".
    ///
    /// **A candidate requiring no subgroup emits no record at all.**
    SubgroupRealization {
        /// Literal width in lanes.
        width: u32,
        /// Exact arithmetic type carried through the transfer.
        arithmetic: ArithmeticType,
        /// Governed key of the required transfer.
        transfer: ReasonCode,
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
    /// One plan's transient-residency requirement against a declared budget.
    ///
    /// Separate from [`Self::Feasibility`] rather than an outcome added to it,
    /// because that record's `available` is not optional and its validation ties
    /// the outcome to `required > available`. A predicate whose budget may be
    /// absent cannot be stated there without supplying a stand-in number, and
    /// every stand-in is a claim: see [`ResidencyOutcome`].
    ///
    /// `tensors` is the plan's `n` — how many score-shaped intermediates the
    /// plan holds live at once. It is carried because "this plan needs too much"
    /// and "this *rung* of this plan needs too much" are different findings.
    TransientResidency {
        predicate: PredicateKey,
        required: Quantity,
        available: Option<Quantity>,
        tensors: u32,
        outcome: ResidencyOutcome,
    },
}

impl ExplainEvent {
    pub(crate) const fn stage(&self) -> ExplainStage {
        match self {
            Self::Check { stage, .. }
            | Self::BudgetStop { stage, .. }
            | Self::CompilerFailure { stage, .. } => *stage,
            Self::Feasibility { .. }
            | Self::TransientResidency { .. }
            | Self::DeferredTargetRequirement { .. }
            | Self::DeferredSubgroupWidthConfirmation { .. }
            | Self::NumericalHonourability { .. }
            | Self::SynchronizationRealization { .. }
            | Self::SubgroupRealization { .. } => ExplainStage::TargetFeasibility,
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
            | Self::TransientResidency {
                outcome: ResidencyOutcome::Fits,
                ..
            }
            | Self::NumericalHonourability {
                outcome: HonourabilityOutcome::Honoured { .. },
                ..
            }
            | Self::SynchronizationRealization {
                outcome: SynchronizationOutcome::Realized { .. },
                ..
            }
            | Self::SubgroupRealization {
                outcome: SynchronizationOutcome::Realized { .. },
                ..
            } => ExplainDisposition::Admitted,
            Self::DeferredTargetRequirement { .. }
            | Self::DeferredSubgroupWidthConfirmation { .. } => {
                ExplainDisposition::DeferredAdmitted
            }
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
            }
            | Self::SubgroupRealization {
                outcome: SynchronizationOutcome::Undeclared,
                ..
            }
            // The requirement is exact and no budget exists to compare it
            // against, so the plan is neither admitted nor refused. It lands
            // here rather than beside the refusals below because a reader acting
            // on it must widen or declare a budget, not change the plan.
            | Self::TransientResidency {
                outcome: ResidencyOutcome::BudgetUndeclared,
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
            | Self::TransientResidency {
                outcome: ResidencyOutcome::Exceeds,
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
            | Self::SubgroupRealization {
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
            // Three separate consistency rules, because this record can be
            // incoherent in three independent ways and collapsing them would let
            // a wrong record through on the strength of a right one.
            Self::TransientResidency {
                required,
                available,
                outcome,
                ..
            } => {
                // A residency requirement is a byte count in both halves. The
                // shared-kind check the `Feasibility` arm performs is not enough
                // on its own here, because a record with no budget has no second
                // kind to agree with.
                if !matches!(required, Quantity::Bytes(_)) {
                    return Err(ExplainError::UnknownQuantityUnit);
                }
                match (outcome, available) {
                    // A comparison outcome without a budget would state a verdict
                    // the record does not carry the evidence for.
                    (ResidencyOutcome::Fits | ResidencyOutcome::Exceeds, None)
                    // ...and a budget alongside `BudgetUndeclared` would carry
                    // evidence the verdict denies exists.
                    | (ResidencyOutcome::BudgetUndeclared, Some(_)) => {
                        return Err(ExplainError::InvalidQuantityRelation);
                    }
                    (ResidencyOutcome::BudgetUndeclared, None) => {}
                    (ResidencyOutcome::Fits | ResidencyOutcome::Exceeds, Some(budget)) => {
                        if !matches!(budget, Quantity::Bytes(_)) {
                            return Err(ExplainError::QuantityKindMismatch);
                        }
                        let exceeds = required.value() > budget.value();
                        if matches!(outcome, ResidencyOutcome::Exceeds) != exceeds {
                            return Err(ExplainError::InvalidQuantityRelation);
                        }
                    }
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
            // The subject and its requirement are one claim: the required
            // width is derived from the subject, so a record whose two halves
            // disagree — or whose relation is not the exact equality ADR 0094
            // decision 7 states — is refused rather than admitted with a
            // narrative the requirement contradicts.
            Self::DeferredSubgroupWidthConfirmation {
                width, requirement, ..
            } => {
                if u64::from(*width) != requirement.required() {
                    return Err(ExplainError::RequirementQuantityMismatch);
                }
                if requirement.relation()
                    != TargetPropertyRequirementRelation::ObservedEqualsRequired
                {
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
            | Self::SubgroupRealization { .. }
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
    allowed_providers: BTreeSet<ProviderIdentity>,
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
    /// Test-only construction control attached at the real writer boundary.
    ///
    /// The control can change only the two inputs production already passes to
    /// [`detail_capacity`]. It cannot construct an [`ExplainError`], the
    /// internal capacity carrier, or a public failure class, so public-path
    /// tests still traverse the production arithmetic and mapping.
    #[cfg(test)]
    detail_capacity_control: DetailCapacityWriterControlForTest,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DetailCapacityWriterOpeningForTest {
    pub(crate) target_profile_key: String,
    pub(crate) numerical_contract_key: String,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DetailCapacityAttemptForTest {
    pub(crate) retained_records: usize,
    pub(crate) retained_bytes: usize,
    pub(crate) attempted_record_bytes: usize,
}

#[cfg(test)]
impl DetailCapacityAttemptForTest {
    pub(crate) fn attempted_records(self) -> u64 {
        u64::try_from(self.retained_records.saturating_add(1)).unwrap_or(u64::MAX)
    }

    pub(crate) fn attempted_bytes(self) -> u64 {
        u64::try_from(
            self.retained_bytes
                .saturating_add(self.attempted_record_bytes),
        )
        .unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DetailCapacityObservationForTest {
    pub(crate) writer_openings: Vec<DetailCapacityWriterOpeningForTest>,
    pub(crate) selected_writer_attempts: Vec<DetailCapacityAttemptForTest>,
}

#[cfg(test)]
#[derive(Debug)]
struct DetailCapacityControlStateForTest {
    target_profile_key: String,
    selected_target_writer: usize,
    record_limit: u32,
    canonical_byte_limit: u32,
    target_writers_opened: usize,
    observation: DetailCapacityObservationForTest,
}

#[cfg(test)]
#[derive(Debug)]
struct DetailCapacityWriterControlForTest {
    selected: bool,
    record_limit: u32,
    canonical_byte_limit: u32,
}

#[cfg(test)]
thread_local! {
    static DETAIL_CAPACITY_CONTROL_FOR_TEST: RefCell<Option<DetailCapacityControlStateForTest>> = const { RefCell::new(None) };
}

#[cfg(test)]
struct ResetDetailCapacityControlForTest;

#[cfg(test)]
impl Drop for ResetDetailCapacityControlForTest {
    fn drop(&mut self) {
        DETAIL_CAPACITY_CONTROL_FOR_TEST.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}

/// Runs one real compile path with test-only limits on one target writer.
///
/// Selection is by target key and target-local writer ordinal so an earlier
/// target may finish before a later target reaches the controlled writer. The
/// observation records raw construction inputs, not an expected public
/// payload; [`detail_capacity`] remains the only authority that decides the arm
/// and constructs [`ExplainDetailCapacity`].
#[cfg(test)]
pub(crate) fn with_detail_capacity_limits_for_test<Output>(
    target_profile_key: &str,
    selected_target_writer: usize,
    record_limit: u32,
    canonical_byte_limit: u32,
    action: impl FnOnce() -> Output,
) -> (Output, DetailCapacityObservationForTest) {
    DETAIL_CAPACITY_CONTROL_FOR_TEST.with(|slot| {
        let mut slot = slot.borrow_mut();
        assert!(slot.is_none(), "detail-capacity test controls cannot nest");
        *slot = Some(DetailCapacityControlStateForTest {
            target_profile_key: target_profile_key.to_owned(),
            selected_target_writer,
            record_limit,
            canonical_byte_limit,
            target_writers_opened: 0,
            observation: DetailCapacityObservationForTest {
                writer_openings: Vec::new(),
                selected_writer_attempts: Vec::new(),
            },
        });
    });
    let reset = ResetDetailCapacityControlForTest;
    let output = action();
    let observation = DETAIL_CAPACITY_CONTROL_FOR_TEST.with(|slot| {
        slot.borrow()
            .as_ref()
            .expect("the detail-capacity control remains installed")
            .observation
            .clone()
    });
    drop(reset);
    (output, observation)
}

#[cfg(test)]
fn detail_capacity_writer_control_for_test(
    request: &VerifiedTargetRequest,
) -> DetailCapacityWriterControlForTest {
    DETAIL_CAPACITY_CONTROL_FOR_TEST.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(state) = slot.as_mut() else {
            return DetailCapacityWriterControlForTest {
                selected: false,
                record_limit: MAX_RECORDS,
                canonical_byte_limit: MAX_CANONICAL_BYTES,
            };
        };
        let target_profile_key = request.target_profile().profile_key().as_str();
        state
            .observation
            .writer_openings
            .push(DetailCapacityWriterOpeningForTest {
                target_profile_key: target_profile_key.to_owned(),
                numerical_contract_key: request.numerical_contract().key.to_owned(),
            });
        let selected = target_profile_key == state.target_profile_key
            && state.target_writers_opened == state.selected_target_writer;
        if target_profile_key == state.target_profile_key {
            state.target_writers_opened = state.target_writers_opened.saturating_add(1);
        }
        DetailCapacityWriterControlForTest {
            selected,
            record_limit: if selected {
                state.record_limit
            } else {
                MAX_RECORDS
            },
            canonical_byte_limit: if selected {
                state.canonical_byte_limit
            } else {
                MAX_CANONICAL_BYTES
            },
        }
    })
}

#[cfg(test)]
fn record_detail_capacity_attempt_for_test(
    control: &DetailCapacityWriterControlForTest,
    retained_records: usize,
    retained_bytes: usize,
    attempted_record_bytes: usize,
) {
    if !control.selected {
        return;
    }
    DETAIL_CAPACITY_CONTROL_FOR_TEST.with(|slot| {
        slot.borrow_mut()
            .as_mut()
            .expect("a selected writer has an installed detail-capacity control")
            .observation
            .selected_writer_attempts
            .push(DetailCapacityAttemptForTest {
                retained_records,
                retained_bytes,
                attempted_record_bytes,
            });
    });
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
        #[cfg(test)]
        let detail_capacity_control = detail_capacity_writer_control_for_test(request);
        // The authorities this compilation may attribute a rule to *besides* its
        // own: every provider the request's installed lowering registry admits,
        // plus the compiler's governed physical-implementation and
        // fusion-capability providers. The compiler arm is not listed here —
        // `push` admits `ProviderRef::Compiler` ahead of this membership test —
        // so the closed set is this structured set plus that one authority. A
        // rule attributed to any registered identity outside that set is a
        // provenance forgery and fails closed (ADR 0072).
        let mut allowed_providers = BTreeSet::new();
        allowed_providers.insert(crate::frontier::GovernedPhysicalProvider::identity());
        allowed_providers.insert(
            crate::fusion_legality::FusionNumericalCapabilities::governed()
                .provider()
                .clone(),
        );
        allowed_providers.extend(request.capabilities().lowering().providers());
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
            #[cfg(test)]
            detail_capacity_control,
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
        match &rule.provider {
            ProviderRef::Compiler => {}
            ProviderRef::Registered(identity) if self.allowed_providers.contains(identity) => {}
            ProviderRef::Registered(_) => return Err(ExplainError::ProviderAuthorityMismatch),
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
        let capacity = if terminal {
            (self.records.len().saturating_add(1)
                > usize::try_from(MAX_TRACE_RECORDS).unwrap_or(usize::MAX)
                || self.retained_bytes.saturating_add(bytes)
                    > usize::try_from(MAX_TRACE_CANONICAL_BYTES).unwrap_or(usize::MAX))
            .then_some(ExplainError::TerminalCapacity)
        } else {
            #[cfg(test)]
            record_detail_capacity_attempt_for_test(
                &self.detail_capacity_control,
                self.retained_detail_records,
                self.retained_detail_bytes,
                bytes,
            );
            #[cfg(test)]
            let (record_limit, canonical_byte_limit) = (
                self.detail_capacity_control.record_limit,
                self.detail_capacity_control.canonical_byte_limit,
            );
            #[cfg(not(test))]
            let (record_limit, canonical_byte_limit) = (MAX_RECORDS, MAX_CANONICAL_BYTES);
            detail_capacity(
                self.retained_detail_records,
                self.retained_detail_bytes,
                bytes,
                record_limit,
                canonical_byte_limit,
            )
            .map(ExplainError::DetailCapacity)
        };
        if let Some(error) = capacity {
            self.encoded_records.truncate(committed);
            return Err(error);
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

/// Returns the first detail-capacity arm the attempted retained prefix exceeds.
///
/// Record-first ordering is part of the refusal contract. If one record would
/// exceed both independent limits, the public payload names records because
/// that is the first comparison construction made; it does not claim the byte
/// arm passed. Limits are parameters only so the negative controls can place
/// each one exactly around one fixed attempted prefix. Production passes the
/// two unchanged build constants.
fn detail_capacity(
    retained_records: usize,
    retained_bytes: usize,
    attempted_record_bytes: usize,
    record_limit: u32,
    canonical_byte_limit: u32,
) -> Option<ExplainDetailCapacity> {
    let attempted_records = u64::try_from(retained_records.saturating_add(1)).unwrap_or(u64::MAX);
    if attempted_records > u64::from(record_limit) {
        return Some(ExplainDetailCapacity {
            resource: BudgetResource::ExplainDetailRecords,
            limit: u64::from(record_limit),
            reported: attempted_records,
        });
    }
    let attempted_bytes =
        u64::try_from(retained_bytes.saturating_add(attempted_record_bytes)).unwrap_or(u64::MAX);
    if attempted_bytes > u64::from(canonical_byte_limit) {
        return Some(ExplainDetailCapacity {
            resource: BudgetResource::ExplainDetailCanonicalBytes,
            limit: u64::from(canonical_byte_limit),
            reported: attempted_bytes,
        });
    }
    None
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
                "{} {} {} rule={}@{} provider=",
                record.id.local,
                stage_name(record.event.stage()),
                disposition_name(record.event.disposition()),
                record.rule.key.as_str(),
                record.rule.revision,
            );
            render_provider_ref(&mut output, &record.rule.provider);
            output.push_str(" subject=");
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
        ExplainEvent::TransientResidency {
            predicate,
            required,
            available,
            tensors,
            outcome,
        } => {
            let _ = write!(
                output,
                "transient-residency:{}:{}:tensors={tensors}:{}={}",
                predicate.as_str(),
                residency_text(*outcome),
                quantity_name(*required),
                required.value(),
            );
            // Rendered as an explicit word rather than omitted, so that a budget
            // absent from the record cannot be mistaken for a budget the renderer
            // merely did not print.
            match available {
                Some(budget) => {
                    let _ = write!(output, ":budget={}", budget.value());
                }
                None => output.push_str(":budget=undeclared"),
            }
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
        // A first spelling for a record the earlier vocabulary could not
        // produce; no existing trace renders differently, so the renderer does
        // not step. The whole atomic subject is on the line for the reason the
        // `subgroup:` spelling puts it there: a reader must see which
        // realization authorized the deferred query, not only its width.
        ExplainEvent::DeferredSubgroupWidthConfirmation {
            entry,
            width,
            arithmetic,
            transfer,
            requirement,
        } => {
            let query = requirement.query();
            let provider = query.provider();
            let _ = write!(
                output,
                "feasibility:subgroup-width-confirmation:deferred:entry={entry}:{}:width={width}:arithmetic={}:transfer={}:query={}@{}:provider={}::{}@{}",
                target_requirement_relation_name(requirement.relation()),
                arithmetic.canonical_type_key(),
                transfer.as_str(),
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
        ExplainEvent::SubgroupRealization {
            width,
            arithmetic,
            transfer,
            outcome,
        } => {
            let _ = write!(
                output,
                "subgroup:width={width}:arithmetic={}:transfer={}:{}",
                arithmetic.canonical_type_key(),
                transfer.as_str(),
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
/// Every rendered part is written, including the complete resolved dtype and
/// declaring profile. The separate arithmetic enum remains canonical identity
/// input because honourability can differ by arithmetic dtype; a rejection
/// whose declarer is unnamed is not explainable.
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

/// The three residency spellings, each distinct from every other refusal word
/// this renderer emits. `budget-undeclared` deliberately shares no prefix with
/// `rejected`, because the whole point of the third value is that a reader must
/// not act on it as a refusal.
const fn residency_text(outcome: ResidencyOutcome) -> &'static str {
    match outcome {
        ResidencyOutcome::Fits => "within-budget",
        ResidencyOutcome::Exceeds => "rejected:exceeds-budget",
        ResidencyOutcome::BudgetUndeclared => "budget-undeclared",
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

/// Exact construction state at the first detail the writer could not retain.
///
/// This is not the complete trace's demand: construction stopped at this
/// attempted prefix. It is carried through the internal error chain so the
/// public boundary never has to reconstruct an arm or quantity from a reason
/// string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExplainDetailCapacity {
    resource: BudgetResource,
    limit: u64,
    reported: u64,
}

impl ExplainDetailCapacity {
    pub(crate) const fn resource(self) -> BudgetResource {
        self.resource
    }

    pub(crate) const fn limit(self) -> u64 {
        self.limit
    }

    pub(crate) const fn reported(self) -> u64 {
        self.reported
    }

    #[cfg(test)]
    pub(crate) const fn for_test(resource: BudgetResource, limit: u64, reported: u64) -> Self {
        Self {
            resource,
            limit,
            reported,
        }
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
    DetailCapacity(ExplainDetailCapacity),
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

fn encode_provider_ref(bytes: &mut Vec<u8>, provider: &ProviderRef) {
    match provider {
        ProviderRef::Compiler => {
            bytes.push(PROVIDER_REF_COMPILER);
            bytes.extend_from_slice(&COMPILER_PROVIDER_REVISION.to_be_bytes());
        }
        ProviderRef::Registered(identity) => {
            bytes.push(PROVIDER_REF_REGISTERED);
            push_slice(bytes, identity.namespace().as_bytes());
            push_slice(bytes, identity.name().as_bytes());
            bytes.extend_from_slice(&identity.revision().to_be_bytes());
        }
    }
}

fn render_provider_ref(output: &mut String, provider: &ProviderRef) {
    use fmt::Write as _;
    match provider {
        ProviderRef::Compiler => {
            let _ = write!(
                output,
                "compiler:{COMPILER_PROVIDER_DISPLAY}@{COMPILER_PROVIDER_REVISION}"
            );
        }
        ProviderRef::Registered(identity) => {
            let _ = write!(output, "registered:{identity}");
        }
    }
}

fn push_record(bytes: &mut Vec<u8>, record: &ExplainRecord) {
    bytes.extend_from_slice(&record.id.local.to_be_bytes());
    push_slice(bytes, record.rule.key.as_str().as_bytes());
    bytes.extend_from_slice(&record.rule.revision.to_be_bytes());
    encode_provider_ref(bytes, &record.rule.provider);
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
        // Tag 16, the next unused value. `9` is free of any current event but is
        // not taken here: nothing in this file records why it is absent, and a
        // tag whose retirement is unexplained may already mean something to a
        // decoder of an older trace. Every tag 1..=15 keeps its meaning and its
        // bytes, so no previously encodable record encodes differently and the
        // `tiler.explain.compilation.v1` and `tiler.explain.trace.v1` domains do
        // not step.
        ExplainEvent::TransientResidency {
            predicate,
            required,
            available,
            tensors,
            outcome,
        } => {
            bytes.push(16);
            push_slice(bytes, predicate.as_str().as_bytes());
            bytes.push(match outcome {
                ResidencyOutcome::Fits => 1,
                ResidencyOutcome::Exceeds => 2,
                ResidencyOutcome::BudgetUndeclared => 3,
            });
            bytes.extend_from_slice(&tensors.to_be_bytes());
            encode_quantity(bytes, *required);
            // The presence of a budget is encoded before its value, so a record
            // with no budget cannot encode to the same bytes as one whose budget
            // happens to be zero.
            match available {
                Some(budget) => {
                    bytes.push(1);
                    encode_quantity(bytes, *budget);
                }
                None => bytes.push(0),
            }
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
        // Appended tag `15`: tags `1` through `14` keep their values and field
        // layouts, and tag `8` in particular stays byte-for-byte the capability
        // spelling — a subgroup confirmation is a record the earlier vocabulary
        // could not express, never a capability record under a new
        // interpretation, so the schema does not step.
        ExplainEvent::DeferredSubgroupWidthConfirmation {
            entry,
            width,
            arithmetic,
            transfer,
            requirement,
        } => {
            bytes.push(15);
            bytes.extend_from_slice(&entry.to_be_bytes());
            bytes.extend_from_slice(&width.to_be_bytes());
            bytes.push(arithmetic.tag());
            push_slice(bytes, transfer.as_str().as_bytes());
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
        // Event tag `13`, appended rather than inserted, and the schema version
        // deliberately did not step with it: tags `1` through `12` keep their
        // values and their field layouts, so no previously encodable trace's
        // bytes move and a reader that reaches `13` is reading a record the
        // earlier vocabulary could not express, never an earlier record under a
        // new interpretation. The rule and its consumers are stated once at the
        // version block; renderer v7 gave this record its first
        // `synchronization:` spelling and did not step for the same reason.
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
        // Appended tag `14`: tags `1` through `13` keep their values and field
        // layouts. A candidate that requires no subgroup never emits this
        // record, so previously encodable traces stay byte-identical.
        ExplainEvent::SubgroupRealization {
            width,
            arithmetic,
            transfer,
            outcome,
        } => {
            bytes.push(14);
            bytes.extend_from_slice(&width.to_be_bytes());
            bytes.push(arithmetic.tag());
            push_slice(bytes, transfer.as_str().as_bytes());
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
            encode_provider_ref(bytes, &receipt.provider);
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
        ProviderRef::registered(&provider)
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
        assert_eq!(EXPLAIN_SCHEMA_VERSION, 11);
        assert_eq!(EXPLAIN_RENDERER_VERSION, 10);
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
        let capability_event =
            deferred_target_requirement_event(DeferredTargetRequirementFixture {
                entry: 0,
                predicate: "threads-per-workgroup",
                required: Quantity::Threads(1),
                property: "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
                relation: TargetPropertyRequirementRelation::ObservedAtLeastRequired,
                provider_namespace: "tiler",
                provider_name: "prepared-entry-properties",
                provider_revision: 1,
            });
        encode_event(&mut deferred, &capability_event);
        assert_eq!(deferred[0], 8);
        // The byte-level control for the deferred-subject generalization: the
        // whole capability record — tag, entry, framed predicate key, quantity,
        // framed requirement — keeps its exact pre-subgroup layout, so no
        // previously encodable capability trace moves.
        let ExplainEvent::DeferredTargetRequirement {
            entry,
            predicate,
            required,
            requirement,
        } = &capability_event
        else {
            panic!("the fixture is one capability deferred requirement");
        };
        let mut legacy = vec![8];
        legacy.extend_from_slice(&entry.to_be_bytes());
        push_slice(&mut legacy, predicate.as_str().as_bytes());
        legacy.push(required.kind());
        legacy.extend_from_slice(&required.value().to_be_bytes());
        push_slice(&mut legacy, &requirement.canonical_bytes());
        assert_eq!(deferred, legacy, "capability event tag 8 moved");
        // The append the version block's rule was written against: tag 13 is
        // pinned here because it is what makes "appended, and the schema did not
        // step" a checked claim rather than a comment.
        let synchronization = ExplainEvent::SynchronizationRealization {
            kind: ReasonCode::new("control-barrier").unwrap(),
            execution_scope: ReasonCode::new("workgroup").unwrap(),
            visibility_scope: ReasonCode::new("workgroup").unwrap(),
            fences_workgroup: true,
            fences_device: false,
            ordering: ReasonCode::new("acquire-release").unwrap(),
            outcome: SynchronizationOutcome::Undeclared,
        };
        assert_eq!(synchronization.validate(), Ok(()));
        let mut realization = Vec::new();
        encode_event(&mut realization, &synchronization);
        assert_eq!(realization[0], 13);
        // Tag 15 is the appended subgroup-width confirmation; pinning it holds
        // "appended, and the schema did not step" to a checked claim.
        let mut confirmation = Vec::new();
        encode_event(
            &mut confirmation,
            &deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture::default()),
        );
        assert_eq!(confirmation[0], 15);
    }

    fn encoded_provider(provider: &ProviderRef) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_provider_ref(&mut bytes, provider);
        bytes
    }

    fn rendered_provider(provider: &ProviderRef) -> String {
        let mut rendered = String::new();
        render_provider_ref(&mut rendered, provider);
        rendered
    }

    #[test]
    fn registered_provider_refs_keep_dotted_component_boundaries() {
        let left = ProviderRef::registered(&ProviderIdentity::new("a.b", "c", 1).unwrap());
        let right = ProviderRef::registered(&ProviderIdentity::new("a", "b.c", 1).unwrap());
        assert_ne!(left, right);
        assert_ne!(encoded_provider(&left), encoded_provider(&right));
        assert_eq!(rendered_provider(&left), "registered:a.b::c@1");
        assert_eq!(rendered_provider(&right), "registered:a::b.c@1");
    }

    #[test]
    fn registered_provider_refs_accept_maximum_legal_components() {
        let namespace = "n".repeat(255);
        let name = "m".repeat(255);
        let maximum = ProviderIdentity::new(&namespace, &name, 1).unwrap();
        assert!(
            ProviderIdentity::new(format!("{namespace}x"), "m", 1).is_err(),
            "255 bytes is the live component ceiling this fixture occupies",
        );
        let maximum = ProviderRef::registered(&maximum);
        assert_eq!(
            encoded_provider(&maximum).len(),
            1 + 8 + 255 + 8 + 255 + 4,
            "the tagged registered encoding frames both maximum components and the revision",
        );
        assert_eq!(
            rendered_provider(&maximum),
            format!("registered:{namespace}::{name}@1")
        );
        assert!(
            RuleRef::provided("test.rule", 1, maximum).is_ok(),
            "a legal maximum identity is a representable explain authority"
        );
    }

    #[test]
    fn registered_provider_refs_keep_revision_in_canonical_bytes() {
        let revision_one = ProviderRef::registered(&ProviderIdentity::new("a", "b", 1).unwrap());
        let revision_two = ProviderRef::registered(&ProviderIdentity::new("a", "b", 2).unwrap());
        assert_ne!(revision_one, revision_two);
        assert_ne!(
            encoded_provider(&revision_one),
            encoded_provider(&revision_two)
        );
        assert_eq!(rendered_provider(&revision_one), "registered:a::b@1");
        assert_eq!(rendered_provider(&revision_two), "registered:a::b@2");
    }

    #[test]
    fn compiler_authority_is_not_a_registered_tiler_compiler_identity() {
        let compiler = ProviderRef::builtin();
        let registered =
            ProviderRef::registered(&ProviderIdentity::new("tiler", "compiler", 1).unwrap());
        assert_ne!(compiler, registered);
        assert_ne!(encoded_provider(&compiler), encoded_provider(&registered));
        assert_eq!(
            encoded_provider(&compiler),
            [PROVIDER_REF_COMPILER]
                .into_iter()
                .chain(COMPILER_PROVIDER_REVISION.to_be_bytes())
                .collect::<Vec<_>>()
        );
        assert_eq!(rendered_provider(&compiler), "compiler:tiler.compiler@1");
        assert_eq!(
            rendered_provider(&registered),
            "registered:tiler::compiler@1"
        );
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

    #[derive(Clone, Copy)]
    struct DeferredSubgroupConfirmationFixture {
        entry: u32,
        width: u32,
        arithmetic: ArithmeticType,
        transfer: &'static str,
        required: u64,
        property: &'static str,
        relation: TargetPropertyRequirementRelation,
        provider_namespace: &'static str,
        provider_name: &'static str,
        provider_revision: u32,
    }

    impl Default for DeferredSubgroupConfirmationFixture {
        fn default() -> Self {
            Self {
                entry: 0,
                width: 32,
                arithmetic: ArithmeticType::F32,
                transfer: "in-range-xor-shuffle",
                required: 32,
                property: "tiler.test.prepared-entry.subgroup-width.v1",
                relation: TargetPropertyRequirementRelation::ObservedEqualsRequired,
                provider_namespace: "tiler",
                provider_name: "prepared-entry-properties",
                provider_revision: 1,
            }
        }
    }

    fn deferred_subgroup_confirmation_event(
        fixture: DeferredSubgroupConfirmationFixture,
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
        ExplainEvent::DeferredSubgroupWidthConfirmation {
            entry: fixture.entry,
            width: fixture.width,
            arithmetic: fixture.arithmetic,
            transfer: ReasonCode::new(fixture.transfer).unwrap(),
            requirement: PreparedEntryTargetRequirement::new(
                query,
                fixture.required,
                fixture.relation,
            )
            .unwrap(),
        }
    }

    /// The appended subgroup confirmation carries its complete atomic subject,
    /// every subject and query field is identity-bearing, and a record whose
    /// two halves disagree is refused rather than admitted.
    #[test]
    fn deferred_subgroup_width_confirmation_identity_and_rendering_are_complete() {
        let fixture = DeferredSubgroupConfirmationFixture::default();
        let baseline = deferred_subgroup_confirmation_event(fixture);
        assert_eq!(baseline.validate(), Ok(()));
        assert_eq!(baseline.stage(), ExplainStage::TargetFeasibility);
        assert_eq!(baseline.disposition(), ExplainDisposition::DeferredAdmitted);
        let mut baseline_identity = Vec::new();
        encode_event(&mut baseline_identity, &baseline);
        // Each perturbation moves exactly one field of the subject, the entry,
        // or the executable query, with the checks unchanged. The width and
        // required value move together because they are one derived claim; the
        // case where they disagree is the validation refusal below, not an
        // identity case. The transfer perturbation uses a neighbouring key
        // because the governed vocabulary currently has one variant.
        for (dimension, changed) in [
            (
                "entry",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    entry: 1,
                    ..fixture
                }),
            ),
            (
                "width",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    width: 64,
                    required: 64,
                    ..fixture
                }),
            ),
            (
                "arithmetic",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    arithmetic: ArithmeticType::Bf16,
                    ..fixture
                }),
            ),
            (
                "transfer",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    transfer: "neighbouring-transfer",
                    ..fixture
                }),
            ),
            (
                "query key",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    property: "tiler.test.prepared-entry.neighbouring-width.v1",
                    ..fixture
                }),
            ),
            (
                "provider namespace",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    provider_namespace: "neighbour",
                    ..fixture
                }),
            ),
            (
                "provider name",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    provider_name: "neighbouring-provider",
                    ..fixture
                }),
            ),
            (
                "provider revision",
                deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                    provider_revision: 2,
                    ..fixture
                }),
            ),
        ] {
            let mut changed_identity = Vec::new();
            encode_event(&mut changed_identity, &changed);
            assert_ne!(
                baseline_identity, changed_identity,
                "perturbing {dimension} alone left the identity bytes unchanged",
            );
        }

        let mut rendered = String::new();
        render_event(&mut rendered, &baseline);
        assert_eq!(
            rendered,
            "feasibility:subgroup-width-confirmation:deferred:entry=0:observed-equals-required:width=32:arithmetic=tiler::f32@1:transfer=in-range-xor-shuffle:query=tiler.test.prepared-entry.subgroup-width.v1@prepared-kernel-preflight:provider=tiler::prepared-entry-properties@1"
        );

        // The two refusals: a width the requirement does not require, and a
        // relation that is not the exact equality the confirmation states.
        assert_eq!(
            deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                required: 64,
                ..fixture
            })
            .validate(),
            Err(ExplainError::RequirementQuantityMismatch)
        );
        assert_eq!(
            deferred_subgroup_confirmation_event(DeferredSubgroupConfirmationFixture {
                relation: TargetPropertyRequirementRelation::ObservedAtLeastRequired,
                ..fixture
            })
            .validate(),
            Err(ExplainError::InvalidQuantityRelation)
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
                    means: ReasonCode::new(cause.means().label()).unwrap(),
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
            "basis=compile-profile-measurement:contexts=1",
            "code-generator=test-offline-compiler@1.0",
            "env=test-platform/1.0/build-1/test-architecture/test-hardware",
            // `test-selection.v1` as two lowercase hexadecimal digits per byte:
            // the rendered evidence names the exact selection, not a summary.
            "compilation-selection=746573742d73656c656374696f6e2e7631",
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
            (
                "compilation selection",
                refusal(
                    crate::target::honourability::measured_profile_source_with_selection(
                        "test.probe.v1",
                        "1.0",
                        "build-1",
                        b"test-selection.v2",
                    ),
                ),
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
                // The pinned request qualifier. The request subject binds the
                // frozen semantic, scalar, lowering-capability, and
                // semantic-realization authorities (the last folding the
                // count-prefixed law sidecar), the complete target
                // declaration, the recognized per-output program, and every
                // deterministic budget. A change to any of those moves this
                // digest for every governed compilation — including this
                // fixture's unrelated multiply — and that movement is the
                // assertion working rather than collateral damage: two
                // requests built against different authorities, targets,
                // programs, or budgets are different requests, and a digest
                // that survived such a change would mean the subject reached
                // a name without reaching its content.
                //
                // Appended tags under per-tag framing move this value without
                // stepping any encoding version; only previously encodable
                // bytes moving requires a step. When the value moves,
                // recompute it on the tree the change lands in — never copy
                // it from a producing branch — with
                //   cargo nextest run -p tiler-compiler -E \
                //     'test(deterministic_trace_is_sealed_and_rendered_separately)'
                // and take the `left` value the assertion reports. The cause
                // belongs in the commit that moves it, not appended here.
                "tiler-explain-v10 request=8bdb7dd58e3aa485\n",
                "0 candidate-enumeration admitted rule=test.rule@1 provider=compiler:tiler.compiler@1 subject=candidate:candidate:a event=check:candidate.legal:proven:checked-invariant causes=-\n",
                "1 selection selected rule=tiler.selection.structural-pareto.v1@1 provider=compiler:tiler.compiler@1 subject=alternative:alternative:test event=selection:tiler.selection.structural-pareto.v1:selected causes=-\n",
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
                    ProviderRef::registered(
                        &ProviderIdentity::new("foreign", "provider", 1).unwrap(),
                    ),
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
        assert!(matches!(
            pressured.push_detail(
                excess_parts.rule,
                excess_parts.subjects,
                excess_parts.event,
                excess_parts.causes,
            ),
            Err(ExplainError::DetailCapacity(capacity))
                if capacity.resource() == BudgetResource::ExplainDetailRecords
                    && capacity.limit() == u64::from(MAX_RECORDS)
                    && capacity.reported() == u64::from(MAX_RECORDS) + 1
        ));
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

    /// Each construction arm admits the attempted prefix on its exact limit,
    /// refuses it when that limit moves one lower, and simultaneous pressure
    /// reports the record arm because construction checks it first.
    #[test]
    fn detail_capacity_arms_are_independent_and_record_first() {
        const RETAINED_RECORDS: usize = 37;
        const RETAINED_BYTES: usize = 80;
        const ATTEMPTED_RECORD_BYTES: usize = 21;
        const ATTEMPTED_RECORDS: u32 = 38;
        const ATTEMPTED_BYTES: u32 = 101;

        assert_eq!(
            detail_capacity(
                RETAINED_RECORDS,
                RETAINED_BYTES,
                ATTEMPTED_RECORD_BYTES,
                ATTEMPTED_RECORDS,
                ATTEMPTED_BYTES,
            ),
            None,
            "an attempted prefix exactly on both limits is admitted",
        );

        assert_eq!(
            detail_capacity(
                RETAINED_RECORDS,
                RETAINED_BYTES,
                ATTEMPTED_RECORD_BYTES,
                ATTEMPTED_RECORDS - 1,
                ATTEMPTED_BYTES,
            ),
            Some(ExplainDetailCapacity::for_test(
                BudgetResource::ExplainDetailRecords,
                u64::from(ATTEMPTED_RECORDS - 1),
                u64::from(ATTEMPTED_RECORDS),
            )),
            "moving only the record limit one below the attempted prefix names records",
        );

        assert_eq!(
            detail_capacity(
                RETAINED_RECORDS,
                RETAINED_BYTES,
                ATTEMPTED_RECORD_BYTES,
                ATTEMPTED_RECORDS,
                ATTEMPTED_BYTES - 1,
            ),
            Some(ExplainDetailCapacity::for_test(
                BudgetResource::ExplainDetailCanonicalBytes,
                u64::from(ATTEMPTED_BYTES - 1),
                u64::from(ATTEMPTED_BYTES),
            )),
            "moving only the byte limit one below the attempted prefix names canonical bytes",
        );

        assert_eq!(
            detail_capacity(
                RETAINED_RECORDS,
                RETAINED_BYTES,
                ATTEMPTED_RECORD_BYTES,
                ATTEMPTED_RECORDS - 1,
                ATTEMPTED_BYTES - 1,
            ),
            Some(ExplainDetailCapacity::for_test(
                BudgetResource::ExplainDetailRecords,
                u64::from(ATTEMPTED_RECORDS - 1),
                u64::from(ATTEMPTED_RECORDS),
            )),
            "simultaneous pressure must report the first, record-count comparison",
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
        changed_provider.records[0].rule.provider =
            ProviderRef::registered(&ProviderIdentity::new("tiler", "compiler", 1).unwrap());
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
                .matches("tiler-explain-v10 request=")
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

    /// The five findings a reader must be able to tell apart.
    ///
    /// Named for what a reader is expected to *do* about each, because that is
    /// what makes them five findings rather than five spellings of "no plan":
    /// a missing fusion role is answered by registering a role, a budget stop by
    /// widening a bound, a target rejection by choosing another target or plan,
    /// an undeclared residency budget by declaring one, and a dominance pruning
    /// by nothing at all — the candidate lost to a better one that was kept.
    const FIVE_FINDINGS: usize = 5;

    fn five_distinguishable_findings() -> Vec<(&'static str, ExplainEvent)> {
        vec![
            (
                "missing-fusion-role",
                // Fails closed to `FusionLegality::Unknown`, which is why this is
                // an `Unknown` assessment and not a `Disproved` one: no authority
                // refused the fusion, the registry simply declares no role.
                ExplainEvent::Check {
                    stage: ExplainStage::CapabilityResolution,
                    assessment: PredicateAssessment::unknown(
                        "fusion.numerical-role",
                        ReasonCode::new("unsupported-operation-capability").unwrap(),
                    )
                    .unwrap(),
                    rejection: RejectionClass::IntrinsicInvalid,
                },
            ),
            (
                "budget-stop",
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::RegionFormation,
                    resource: ResourceKey::new("region-members").unwrap(),
                    limit: 62,
                    actual: 63,
                },
            ),
            (
                "target-rejection",
                ExplainEvent::Feasibility {
                    predicate: PredicateKey::new("target.workgroup-threads").unwrap(),
                    outcome: FeasibilityOutcome::Rejected(
                        ReasonCode::new("workgroup-threads-exceeded").unwrap(),
                    ),
                    required: Quantity::Threads(2_048),
                    available: Quantity::Threads(1_024),
                },
            ),
            (
                "unknown-residency-feasibility",
                ExplainEvent::TransientResidency {
                    predicate: PredicateKey::new("target.transient-residency").unwrap(),
                    // B1-d prefill, the `n = 1` best case.
                    required: Quantity::Bytes(5_444_206_600),
                    available: None,
                    tensors: 1,
                    outcome: ResidencyOutcome::BudgetUndeclared,
                },
            ),
            (
                "dominance-pruning",
                ExplainEvent::CostAssessment {
                    model: CostModelKey::new("cost.analytical").unwrap(),
                    basis: EvidenceBasis::CheckedInvariant,
                    terms: vec![CostTerm::new("work-span", Quantity::Operations(4_096)).unwrap()],
                    disposition: CostDisposition::Dominated,
                },
            ),
        ]
    }

    /// Each of the five renders as its own finding, and none renders as "not
    /// fused".
    ///
    /// **What it would take for this to say *no*, and that the case is
    /// reachable.** It says no if any two of the five produce the same rendered
    /// line, or if any one of them renders text a reader could read as a bare
    /// absence of fusion. Both arms are reachable and are demonstrated by
    /// perturbation rather than asserted: collapsing `residency_text`'s three
    /// arms to one string reddens the distinctness assertion, and giving any
    /// finding the words "not fused" reddens the second.
    ///
    /// The population is sized from [`FIVE_FINDINGS`] rather than by counting
    /// the vector, so deleting a case is a failure here instead of a census that
    /// silently shrinks while still reporting no collision.
    #[test]
    fn the_five_findings_render_distinguishably_and_none_reads_as_not_fused() {
        let request = request(2.0);
        let findings = five_distinguishable_findings();
        assert_eq!(
            findings.len(),
            FIVE_FINDINGS,
            "a finding was dropped from the distinguishability population"
        );

        let mut rendered = Vec::new();
        for (key, event) in findings {
            let mut writer = ExplainWriter::new(&request).unwrap();
            let subject = writer
                .subject(SubjectKind::Candidate, format!("candidate:{key}"))
                .unwrap();
            writer
                .push_detail(
                    RuleRef::builtin("test.rule").unwrap(),
                    vec![subject],
                    event,
                    Vec::new(),
                )
                .unwrap();
            let text = finish_test_trace(writer).render();
            let line = text
                .lines()
                .find(|line| line.contains(&format!("candidate:{key}")))
                .unwrap_or_else(|| panic!("{key}: no rendered line names its subject"))
                .to_owned();

            // No finding may present as a bare absence of fusion.
            assert!(
                !line.contains("not fused") && !line.contains("not-fused"),
                "{key} rendered as an unqualified absence of fusion: {line}"
            );
            rendered.push((key, line));
        }

        // Every pair differs. Compared pairwise with the offending pair named,
        // because a deduplicated-length assertion reports only that two
        // collapsed and not which two.
        for (index, (left_key, left)) in rendered.iter().enumerate() {
            for (right_key, right) in &rendered[index + 1..] {
                assert_ne!(
                    left, right,
                    "{left_key} and {right_key} render identically: {left}"
                );
            }
        }
    }

    /// The three residency verdicts render as three lines, and the undeclared
    /// one is not spelled as a rejection.
    #[test]
    fn the_residency_outcomes_render_as_three_distinct_findings() {
        let request = request(2.0);
        let cases = [
            (ResidencyOutcome::Fits, Some(Quantity::Bytes(u64::MAX))),
            (ResidencyOutcome::Exceeds, Some(Quantity::Bytes(1))),
            (ResidencyOutcome::BudgetUndeclared, None),
        ];
        assert_eq!(
            std::mem::variant_count::<ResidencyOutcome>(),
            cases.len(),
            "an outcome was added to the vocabulary without a rendering case"
        );

        let mut lines = Vec::new();
        for (outcome, available) in cases {
            let mut writer = ExplainWriter::new(&request).unwrap();
            let subject = writer
                .subject(SubjectKind::Candidate, "candidate:residency")
                .unwrap();
            writer
                .push_detail(
                    RuleRef::builtin("test.rule").unwrap(),
                    vec![subject],
                    ExplainEvent::TransientResidency {
                        predicate: PredicateKey::new("target.transient-residency").unwrap(),
                        required: Quantity::Bytes(5_444_206_600),
                        available,
                        tensors: 1,
                        outcome,
                    },
                    Vec::new(),
                )
                .unwrap();
            lines.push(finish_test_trace(writer).render());
        }

        assert!(lines[0].contains("transient-residency:target.transient-residency:within-budget"));
        assert!(lines[1].contains("rejected:exceeds-budget"));
        assert!(lines[2].contains("budget-undeclared"));
        // The undeclared line states the absence rather than omitting the field,
        // and does not carry the refusal word.
        assert!(lines[2].contains(":budget=undeclared"));
        assert!(
            !lines[2].contains("rejected"),
            "the undeclared verdict rendered as a rejection: {}",
            lines[2]
        );
        // ...and it still reports the exact requirement.
        assert!(lines[2].contains("bytes=5444206600"));
        assert!(lines[2].contains("tensors=1"));
    }

    /// A residency record whose halves disagree is refused, in each of the three
    /// independent ways it can disagree.
    #[test]
    fn an_incoherent_residency_record_is_refused() {
        let request = request(2.0);
        let attempt = |required: Quantity, available: Option<Quantity>, outcome| {
            let mut writer = ExplainWriter::new(&request).unwrap();
            let subject = writer
                .subject(SubjectKind::Candidate, "candidate:residency")
                .unwrap();
            writer.push_detail(
                RuleRef::builtin("test.rule").unwrap(),
                vec![subject],
                ExplainEvent::TransientResidency {
                    predicate: PredicateKey::new("target.transient-residency").unwrap(),
                    required,
                    available,
                    tensors: 1,
                    outcome,
                },
                Vec::new(),
            )
        };

        // A verdict that claims a comparison, with nothing to compare against.
        assert_eq!(
            attempt(Quantity::Bytes(10), None, ResidencyOutcome::Fits),
            Err(ExplainError::InvalidQuantityRelation)
        );
        assert_eq!(
            attempt(Quantity::Bytes(10), None, ResidencyOutcome::Exceeds),
            Err(ExplainError::InvalidQuantityRelation)
        );
        // A verdict that denies a budget exists, carrying one.
        assert_eq!(
            attempt(
                Quantity::Bytes(10),
                Some(Quantity::Bytes(10)),
                ResidencyOutcome::BudgetUndeclared
            ),
            Err(ExplainError::InvalidQuantityRelation)
        );
        // A verdict that contradicts its own two numbers, in both directions.
        assert_eq!(
            attempt(
                Quantity::Bytes(10),
                Some(Quantity::Bytes(100)),
                ResidencyOutcome::Exceeds
            ),
            Err(ExplainError::InvalidQuantityRelation)
        );
        assert_eq!(
            attempt(
                Quantity::Bytes(100),
                Some(Quantity::Bytes(10)),
                ResidencyOutcome::Fits
            ),
            Err(ExplainError::InvalidQuantityRelation)
        );
        // A residency stated in something other than bytes.
        assert_eq!(
            attempt(
                Quantity::Count(10),
                None,
                ResidencyOutcome::BudgetUndeclared
            ),
            Err(ExplainError::UnknownQuantityUnit)
        );
        assert_eq!(
            attempt(
                Quantity::Bytes(10),
                Some(Quantity::Count(100)),
                ResidencyOutcome::Fits
            ),
            Err(ExplainError::QuantityKindMismatch)
        );

        // ...and the coherent records are admitted, so the refusals above
        // discriminate rather than refusing every residency record.
        assert!(
            attempt(
                Quantity::Bytes(10),
                None,
                ResidencyOutcome::BudgetUndeclared
            )
            .is_ok()
        );
        assert!(
            attempt(
                Quantity::Bytes(10),
                Some(Quantity::Bytes(100)),
                ResidencyOutcome::Fits
            )
            .is_ok()
        );
        assert!(
            attempt(
                Quantity::Bytes(100),
                Some(Quantity::Bytes(10)),
                ResidencyOutcome::Exceeds
            )
            .is_ok()
        );
    }

    /// The undeclared verdict is dispositioned as neither admitted nor refused.
    ///
    /// This is the claim that keeps such a candidate out of an executable
    /// frontier: `DeferredUnsupported` is the same disposition the two other
    /// `Undeclared` outcomes carry, and it is neither `Admitted` — which would
    /// let an unbounded plan run — nor `RejectedTarget`, which would assert a
    /// disproof no profile supplied.
    #[test]
    fn the_undeclared_residency_verdict_admits_nothing_and_refuses_nothing() {
        let event = |outcome, available| ExplainEvent::TransientResidency {
            predicate: PredicateKey::new("target.transient-residency").unwrap(),
            required: Quantity::Bytes(10),
            available,
            tensors: 1,
            outcome,
        };
        assert_eq!(
            event(ResidencyOutcome::BudgetUndeclared, None).disposition(),
            ExplainDisposition::DeferredUnsupported
        );
        // The two comparison outcomes keep the ordinary dispositions, so the
        // third is distinguished by its verdict rather than by this record kind.
        assert_eq!(
            event(ResidencyOutcome::Fits, Some(Quantity::Bytes(100))).disposition(),
            ExplainDisposition::Admitted
        );
        assert_eq!(
            event(ResidencyOutcome::Exceeds, Some(Quantity::Bytes(1))).disposition(),
            ExplainDisposition::RejectedTarget
        );
    }

    /// An undeclared budget and a declared budget of zero encode differently.
    ///
    /// The distinction the whole third verdict rests on: "nothing bounds this
    /// plan" and "this target permits no transient memory at all" are different
    /// claims, and a canonical encoding that conflated them would let a trace
    /// prove the wrong one.
    #[test]
    fn an_undeclared_budget_does_not_encode_as_a_zero_budget() {
        let mut undeclared = Vec::new();
        encode_event(
            &mut undeclared,
            &ExplainEvent::TransientResidency {
                predicate: PredicateKey::new("target.transient-residency").unwrap(),
                required: Quantity::Bytes(1),
                available: None,
                tensors: 1,
                outcome: ResidencyOutcome::BudgetUndeclared,
            },
        );
        let mut zero = Vec::new();
        encode_event(
            &mut zero,
            &ExplainEvent::TransientResidency {
                predicate: PredicateKey::new("target.transient-residency").unwrap(),
                required: Quantity::Bytes(1),
                available: Some(Quantity::Bytes(0)),
                tensors: 1,
                outcome: ResidencyOutcome::Exceeds,
            },
        );
        assert_ne!(undeclared, zero);
    }
}
