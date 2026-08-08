//! The deterministic `EnumerateRegionCandidates` stage.
//!
//! Region formation runs immediately after [`normalize_semantics`] and observes
//! an arbitrary verified semantic DAG. It *proposes* region candidates and does
//! nothing else: it selects no cover, chooses no implementation, lowers no index
//! region, plans nothing physical, and costs nothing.
//!
//! A candidate is more than a set of operation identifiers. Following the
//! accepted fusion contract it carries member operations, boundary inputs,
//! retained outputs, and an allowed-duplication policy, and it is always
//! nonempty, connected, and convex in the operation DAG.
//!
//! The machinery and its guarantees are:
//!
//! - **Complete singleton coverage.** Every operation's singleton region is
//!   emitted before any growth budget can fire, so an unfused plan is never
//!   lost. Growth budgets bound only multi-member regions.
//! - **Connectivity by construction.** Growth adds one operation adjacent to the
//!   current set through a producer/consumer value edge, so every reachable set
//!   is connected. Each connected set is generated exactly once, by the seed
//!   equal to its minimum member ordinal.
//! - **Convexity by filter.** Growth explores connected sets without requiring
//!   intermediate convexity, because convexity is not inherited by subsets;
//!   requiring it during growth would silently lose legal regions. Convexity is
//!   instead decided when a set is emitted, which keeps enumeration complete for
//!   connected convex regions up to the declared budgets.
//! - **Termination.** Every growth step inserts a strictly larger member set
//!   into a per-seed visited set bounded by the member budget, and every step
//!   consumes one unit of the whole-compilation expansion budget.
//! - **Explicit budgets.** [`DeterministicBudgets`] declares member, boundary-
//!   output, live-value, per-seed candidate, and expansion budgets. Every budget
//!   that fires is retained as a typed [`RegionBudgetStop`] and emitted as a
//!   typed explain `BudgetStop`, so a legal alternative lost to a bound is
//!   reported as bounded search loss rather than silently dropped.
//! - **Separated identity.** Region *content* identity canonicalizes the region's
//!   internal computation with members renumbered to region-local positions, so
//!   the same reusable content occurring at a different graph site has the same
//!   content identity. Region *occurrence* identity additionally pins the exact
//!   graph-local members and boundary bindings. The two are never conflated.
//! - **Duplication.** Producer duplication is disabled in this profile, as the
//!   optimizer contract fixes for the first implementation. Overlapping
//!   candidates are still enumerated; whether an overlap may become a cover is a
//!   later authority's question, and [`DuplicationPolicy::Disabled`] tells that
//!   authority the answer is no.
//!
//! [`normalize_semantics`]: crate::normalize::normalize_semantics

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::index::{
    FrozenIndexRealizationLawRegistry, IndexRefinementSubject, NumericalContractIdentity,
    StagedInputSource, VerifiedIndexRegionSequence,
};
use tiler_ir::semantic::{
    CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueView, OpKey,
    OperationAttributes, OperationEffect, SemanticProgram, ValueId,
};
use tiler_ir::shape::Shape;

use crate::explain::{
    EvidenceBasis, ExplainError, ExplainEvent, ExplainFact, ExplainRecordId, ExplainStage,
    ExplainWriter, FactValue, PredicateAssessment, RejectionClass, ResourceKey, RuleRef,
    SubjectKey, SubjectKind,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

/// Stable identity of the region-formation stage rule.
pub(crate) const REGION_FORMATION_RULE: &str = "region.formation.v1";
/// Stable identity of the per-candidate legality rule.
pub(crate) const REGION_CANDIDATE_RULE: &str = "region.candidate.v1";
/// Stable subject key for whole-program region-formation records.
pub(crate) const REGION_FORMATION_SUBJECT: &str = "region-formation:program";
/// Bound on canonical-value nesting accepted by content encoding.
const MAX_CANONICAL_VALUE_DEPTH: u32 = 32;

/// Typed failure of the deterministic region-formation stage.
///
/// Every variant is invalid compiler output rather than a rejected user program.
/// The stage only observes an already verified [`SemanticProgram`], and an
/// illegal *set* is an ordinary [`RegionRejection`] rather than an error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RegionError {
    /// The verified input program violated a stage precondition.
    Structure { rule: &'static str },
    /// A candidate failed recomputation from its own exact contents.
    Invalid { region: String, rule: &'static str },
}

impl RegionError {
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Structure { rule } | Self::Invalid { rule, .. } => rule,
        }
    }

    pub(crate) const fn class(&self) -> &'static str {
        match self {
            Self::Structure { .. } => "structure",
            Self::Invalid { .. } => "invalid",
        }
    }
}

impl fmt::Display for RegionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structure { rule } => write!(
                formatter,
                "compile.region.structure.{rule}: deterministic region formation observed invalid compiler output"
            ),
            Self::Invalid { region, rule } => write!(
                formatter,
                "compile.region.invalid.{rule}: {region} rejected"
            ),
        }
    }
}

impl Error for RegionError {}

/// Graph-local ordinal of one operation in verified topological order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SemanticMemberId(pub(crate) u32);

/// Ordinal of one realization stage within a single semantic occurrence.
///
/// Zero is the occurrence's first stage and is the only ordinal an occurrence
/// realized by one region ever carries. A family whose realization is a region
/// *sequence* — a fold then a normalization, a split reduction's partial pass
/// then its combine — numbers its regions from zero in execution order.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct StageOrdinal(pub(crate) u32);

impl StageOrdinal {
    /// The stage every single-region occurrence carries.
    pub(crate) const FIRST: Self = Self(0);

    /// Returns the ordinal as a plain number for reporting and encoding.
    pub(crate) const fn get(self) -> u32 {
        self.0
    }

    /// Returns the stage immediately after this one.
    ///
    /// Saturating rather than wrapping: a stage count that reached `u32::MAX`
    /// is not a chain any profile builds, and wrapping to zero would make a
    /// later stage indistinguishable from the first one.
    pub(crate) const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// The planner's attribution atom: one realization stage of one occurrence.
///
/// **The atom is a pair rather than a bare occurrence, and the second component
/// is what makes a multi-region realization statable at all.** Region
/// attribution is decided by comparing the exact set an entity claims —
/// [`crate::request::NormalizedOutput::owns_region_members`] against a
/// recognized partition's parts, [`crate::physical::spell_region`] against the
/// region vocabulary, [`crate::cover::verify_cover`]'s duplication accounting
/// against a cover's placements — and with a bare occurrence as the atom, two
/// regions realizing one occurrence in sequence claim *the same set*. The first
/// comparison then answers for both, the second region is unreachable, and a
/// repeated occurrence has no reading other than deliberate duplication. ADR
/// fork `resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`
/// records the derivation and Tom's decision (2026-08-06) to make the atom a
/// pair.
///
/// `Ord` is member-major by field order, so a set of first-stage atoms sorts
/// exactly as its member ordinals did and an occurrence's stages stay adjacent.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SemanticStage {
    member: SemanticMemberId,
    stage: StageOrdinal,
}

impl SemanticStage {
    /// Binds the first — and, for a single-region occurrence, only — stage.
    pub(crate) const fn first(member: SemanticMemberId) -> Self {
        Self {
            member,
            stage: StageOrdinal::FIRST,
        }
    }

    /// Returns the occurrence this atom is a stage of.
    pub(crate) const fn member(self) -> SemanticMemberId {
        self.member
    }

    /// Returns which realization stage of that occurrence this atom is.
    pub(crate) const fn stage(self) -> StageOrdinal {
        self.stage
    }

    /// Returns whether this is the occurrence's first stage.
    ///
    /// The one predicate that distinguishes an atom which *computes* an
    /// occurrence from one which continues it: whole-program coverage, lowering
    /// receipts, and duplication accounting are all obligations of the
    /// occurrence, discharged once by its first stage.
    pub(crate) const fn is_first(self) -> bool {
        self.stage.0 == StageOrdinal::FIRST.0
    }

    /// Returns the atom of the stage immediately after this one.
    pub(crate) const fn next_stage(self) -> Self {
        Self {
            member: self.member,
            stage: self.stage.next(),
        }
    }

    /// Binds an arbitrary stage of one occurrence.
    ///
    /// The caller owns the bound: nothing here checks the ordinal against the
    /// occurrence's realized stage count, because the atom is a coordinate and
    /// the graph carrying the topology is the authority on which coordinates
    /// exist. [`RegionGraph::atom_node`] is where an out-of-range stage refuses.
    pub(crate) const fn at(member: SemanticMemberId, stage: StageOrdinal) -> Self {
        Self { member, stage }
    }
}

/// Graph-local ordinal of one semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SemanticValueId(pub(crate) u32);

/// Returns whether an ordered chain of dispatch claims realizes one subject.
///
/// The obligation a multi-dispatch realization owes and no single dispatch can
/// see: the dispatches between them must compute the subject's occurrences,
/// each exactly once, and nothing else. Two conditions decide it, and the claims
/// are sorted first so both are adjacency tests over one pass.
///
/// - **Every later stage sits directly behind the stage it continues.** A claim
///   of stage `k > 0` requires the element before it to be stage `k - 1` of the
///   same occurrence, which is where it lands once the atoms are sorted. A chain
///   skipping a stage would leave part of a longer realization computed by
///   nobody, and a chain repeating a later stage would compute one part twice —
///   the repeat's predecessor is the identical atom, whose stage is not one
///   less, so it fails here.
/// - **The first stages are exactly the subject.** The subject is ascending and
///   duplicate-free, so a claim outside it, a subject occurrence no dispatch
///   claims, and a first stage claimed twice each break the equality.
///
/// The two together are what the retired comparison against the concatenated
/// claims used to state, restored for an atom that carries a stage: that
/// comparison required the claims to *equal* the subject, which a chain whose
/// later passes name their own stage can no longer satisfy, and whose only
/// alternative spelling was a pass claiming nothing.
pub(crate) fn chain_realizes_subject(
    claims: &mut [SemanticStage],
    subject: &[SemanticStage],
) -> bool {
    claims.sort_unstable();
    for (position, atom) in claims.iter().enumerate() {
        if atom.is_first() {
            continue;
        }
        let continues = position
            .checked_sub(1)
            .and_then(|previous| claims.get(previous))
            .is_some_and(|previous| {
                previous.member() == atom.member()
                    && previous.stage().get().checked_add(1) == Some(atom.stage().get())
            });
        if !continues {
            return false;
        }
    }
    claims
        .iter()
        .copied()
        .filter(|atom| atom.is_first())
        .eq(subject.iter().copied())
}

/// Returns the graph-local ordinal one verified program gives a value.
///
/// **The coordinate is the value's position in the program's own value list**,
/// which is what [`RegionGraph::from_program`] assigns when it builds the same
/// mapping for every value at once. This is its single-lookup form, and it is
/// stated beside the type rather than at its caller so that a stage which holds
/// a [`SemanticProgram`] but no [`RegionGraph`] — program assembly, attributing
/// declared outputs to the regions that publish them — asks the coordinate's
/// owning module rather than re-deriving it.
/// `tests::the_value_ordinal_lookup_indexes_the_graph_view_s_own_record` is what
/// keeps the two spellings in agreement.
///
/// Answers `None` for a value the program does not hold, which is the
/// fail-closed direction: a caller may not treat an unknown handle as ordinal
/// zero.
pub(crate) fn value_ordinal(program: &SemanticProgram, value: ValueId) -> Option<SemanticValueId> {
    let position = program
        .values()
        .position(|candidate| candidate.id() == value)?;
    u32::try_from(position).ok().map(SemanticValueId)
}

/// Producer duplication allowed for one candidate.
///
/// The first implementation fixes this to [`Self::Disabled`]; the exhaustive
/// oracle in this module's tests retains duplication as a completeness witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DuplicationPolicy {
    /// No member may also occur in another region of a chosen cover.
    Disabled,
}

impl DuplicationPolicy {
    const fn tag(self) -> u8 {
        match self {
            Self::Disabled => 1,
        }
    }

    const fn enabled(self) -> bool {
        match self {
            Self::Disabled => false,
        }
    }
}

/// One value the region must export across its boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedOutput {
    /// Graph-local value ordinal of the exported value.
    pub(crate) value: SemanticValueId,
    /// Member that produces the exported value.
    pub(crate) producer: SemanticMemberId,
    /// Zero-based result position on that member.
    pub(crate) result_position: u32,
    /// Whether the value is an ordered named program result.
    pub(crate) named_result: bool,
    /// Whether the value is consumed by an operation outside the region.
    pub(crate) external_consumers: bool,
}

/// Collision-free canonical identity of one region's internal computation.
///
/// Members are renumbered to region-local positions before encoding, so the same
/// reusable content occurring at a different graph site produces equal bytes.
/// Graph-local ordinals are deliberately absent.
///
/// The bytes are held behind an [`Arc`] because an identity is immutable once
/// encoded and is copied far more often than it is built: cover assembly,
/// materialization edges, and cover verification each duplicate one per region
/// per cover. Sharing makes a clone a refcount bump instead of an allocation and
/// a `memcpy` of the whole encoding, and it changes nothing observable —
/// [`Self::as_bytes`] yields the same bytes and the derived `Ord`/`Eq` still
/// compare content, not the pointer.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionContentIdentity {
    canonical: Arc<[u8]>,
}

impl RegionContentIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.canonical
    }

    /// Returns a bounded explain label for this content.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn label(&self) -> String {
        hex_label("region-content:", digest(&self.canonical))
    }
}

/// Collision-free canonical identity of one region occurrence in one graph.
///
/// This is region content plus the exact graph site: member ordinals, boundary
/// input values, and retained output values.
///
/// Shared behind an [`Arc`] for the reason [`RegionContentIdentity`] gives, and
/// more sharply: an occurrence encoding embeds the whole content encoding, so it
/// is the largest identity the cover stages copy.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionOccurrenceIdentity {
    canonical: Arc<[u8]>,
}

impl RegionOccurrenceIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.canonical
    }

    /// Returns a bounded explain label for this occurrence.
    ///
    /// Region formation proves the labels of one compilation's emitted
    /// candidates are pairwise distinct before returning, so within a trace this
    /// label is an injective handle for the occurrence identity.
    fn label(&self) -> Arc<str> {
        Arc::from(hex_label("region:", digest(&self.canonical)))
    }
}

/// One proposed connected convex region over a verified semantic DAG.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionCandidate {
    members: Vec<SemanticStage>,
    boundary_inputs: Vec<SemanticValueId>,
    retained_outputs: Vec<RetainedOutput>,
    duplication: DuplicationPolicy,
    content: RegionContentIdentity,
    occurrence: RegionOccurrenceIdentity,
    /// Shared rather than owned: every cover that places this region copies the
    /// label, so an `Arc<str>` makes that copy a refcount bump.
    label: Arc<str>,
    program_node_count: u32,
}

impl RegionCandidate {
    /// Returns the region's attribution atoms in ascending graph-local order.
    ///
    /// One atom per covered *stage*, not per covered occurrence: [`assemble`] is
    /// the only constructor and builds each atom from a formation node id, and
    /// an occurrence whose registered law realizes a region sequence contributes
    /// one node per stage. A program with no staged member has node ids equal to
    /// member ordinals, so its list is one first-stage atom per occurrence
    /// exactly as it was before stages existed.
    pub(crate) fn members(&self) -> &[SemanticStage] {
        &self.members
    }

    /// Returns the values the region reads from outside itself.
    pub(crate) fn boundary_inputs(&self) -> &[SemanticValueId] {
        &self.boundary_inputs
    }

    /// Returns the ordered values the region must export.
    pub(crate) fn retained_outputs(&self) -> &[RetainedOutput] {
        &self.retained_outputs
    }

    /// Returns the duplication policy this candidate was formed under.
    pub(crate) const fn duplication(&self) -> DuplicationPolicy {
        self.duplication
    }

    /// Returns the site-independent content identity.
    pub(crate) const fn content(&self) -> &RegionContentIdentity {
        &self.content
    }

    /// Returns the graph-occurrence identity.
    pub(crate) const fn occurrence(&self) -> &RegionOccurrenceIdentity {
        &self.occurrence
    }

    /// Returns the bounded explain label of this occurrence.
    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    /// Returns the shared handle to the bounded explain label.
    ///
    /// A cover retains the label of every region it places; handing out the
    /// shared handle keeps that retention free.
    pub(crate) fn label_handle(&self) -> Arc<str> {
        Arc::clone(&self.label)
    }

    /// Returns whether the region covers every stage atom of its program.
    pub(crate) fn covers_whole_program(&self) -> bool {
        u32::try_from(self.members.len()).is_ok_and(|count| count == self.program_node_count)
    }
}

/// A legal set that region formation refused to emit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionRejection {
    /// A path between two members leaves the region and re-enters it.
    NonConvex,
    /// The set is not connected through producer/consumer value edges.
    Disconnected,
    /// A multi-member region contains an operation this profile cannot prove
    /// referentially transparent, so fusing it could change its multiplicity.
    ImpureMember,
    /// The set exceeded a declared deterministic budget.
    Budget(RegionBudgetStop),
}

impl RegionRejection {
    const fn rule(self) -> &'static str {
        match self {
            Self::NonConvex => "convexity",
            Self::Disconnected => "connectivity",
            Self::ImpureMember => "operation-boundary",
            Self::Budget(_) => "budget",
        }
    }
}

/// Deterministic safety budgets that bound region growth.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum RegionBudgetResource {
    /// Semantic occurrences admitted in one region.
    Members,
    /// Retained boundary outputs admitted for one region.
    BoundaryOutputs,
    /// Boundary and member-result values live across one region.
    LiveValues,
    /// Grown candidates admitted for one seed occurrence.
    CandidatesPerSeed,
    /// Candidate expansion attempts admitted for one compilation request.
    Expansions,
}

impl RegionBudgetResource {
    /// Returns the stable resource key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Members => "region-members",
            Self::BoundaryOutputs => "region-boundary-outputs",
            Self::LiveValues => "region-live-values",
            Self::CandidatesPerSeed => "region-candidates-per-seed",
            Self::Expansions => "region-expansions",
        }
    }
}

/// One declared budget and the demand that it refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionBudgetStop {
    /// The budget that fired.
    pub(crate) resource: RegionBudgetResource,
    /// The declared limit.
    pub(crate) limit: u64,
    /// The refused demand observed at the stop point.
    ///
    /// For a per-candidate budget this is the candidate's exact count. For a
    /// growth budget it is the first demand the limit refused, which is a lower
    /// bound on the unexplored space rather than its size.
    pub(crate) actual: u64,
}

/// Explain records that region formation retained for later stages to cite.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RegionFormationRecords {
    /// The stage receipt.
    pub(crate) summary: ExplainRecordId,
    /// The whole-program candidate record, when one was emitted.
    pub(crate) whole_program: Option<ExplainRecordId>,
}

/// The deterministic result of running `EnumerateRegionCandidates` once.
#[derive(Clone, Debug)]
pub(crate) struct RegionFormationOutcome {
    graph: RegionGraph,
    candidates: Vec<RegionCandidate>,
    budget_stops: Vec<RegionBudgetStop>,
    rejections: RegionRejectionTally,
}

/// How many connected sets each structural rule refused.
///
/// Individually explaining every refused set would make the trace grow with the
/// search space, so the stage receipt carries the tally instead.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RegionRejectionTally {
    non_convex: u64,
    disconnected: u64,
    impure_member: u64,
}

impl RegionFormationOutcome {
    /// Returns the derived dataflow view the candidates were formed over.
    pub(crate) const fn graph(&self) -> &RegionGraph {
        &self.graph
    }

    /// Returns every emitted candidate in ascending member order.
    pub(crate) fn candidates(&self) -> &[RegionCandidate] {
        &self.candidates
    }

    /// Returns every budget that stopped a growth path.
    pub(crate) fn budget_stops(&self) -> &[RegionBudgetStop] {
        &self.budget_stops
    }

    /// Returns the candidate covering every operation, when it was emitted.
    ///
    /// A whole-graph set is trivially convex and is formed before growth
    /// starts, so it is absent only when the graph is disconnected, an
    /// operation is not provably pure, or the region's own shape exceeds
    /// `region_members`, `region_boundary_outputs`, or `region_live_values`.
    /// No search bound can remove it.
    pub(crate) fn whole_program_candidate(&self) -> Option<&RegionCandidate> {
        self.candidates
            .iter()
            .find(|candidate| candidate.covers_whole_program())
    }

    /// Emits this outcome through the typed explain authority.
    ///
    /// Records form one linear causal chain rooted at `cause`, so no record
    /// accumulates an unbounded cause set.
    pub(crate) fn record(
        &self,
        explain: &mut ExplainWriter,
        cause: ExplainRecordId,
    ) -> Result<RegionFormationRecords, ExplainError> {
        let mut chain = cause;
        for stop in &self.budget_stops {
            let subject = explain.subject(SubjectKind::Region, REGION_FORMATION_SUBJECT)?;
            chain = explain.push_detail(
                RuleRef::builtin(REGION_FORMATION_RULE)?,
                vec![subject],
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::RegionFormation,
                    resource: ResourceKey::new(stop.resource.key())?,
                    limit: stop.limit,
                    actual: stop.actual,
                },
                vec![chain],
            )?;
        }
        // `whole_program` stays optional because a program may have no
        // whole-program candidate at all; that `None` is a fact about the
        // candidate set, not a record that went missing.
        let mut whole_program = None;
        for candidate in self.candidates() {
            let record = record_candidate(explain, candidate, chain)?;
            if candidate.covers_whole_program() {
                whole_program = Some(record);
            }
            chain = record;
        }
        let summary = self.record_summary(explain, chain)?;
        Ok(RegionFormationRecords {
            summary,
            whole_program,
        })
    }

    fn record_summary(
        &self,
        explain: &mut ExplainWriter,
        cause: ExplainRecordId,
    ) -> Result<ExplainRecordId, ExplainError> {
        let assessment = PredicateAssessment::proven(
            "region.singleton-coverage-complete",
            EvidenceBasis::CheckedInvariant,
        )?
        .with_fact(ExplainFact::new(
            "operation-count",
            FactValue::Count(u64::from(self.graph.operation_count())),
        )?)?
        .with_fact(ExplainFact::new(
            "candidate-count",
            FactValue::Count(count(self.candidates.len())),
        )?)?
        .with_fact(ExplainFact::new(
            "rejected-non-convex",
            FactValue::Count(self.rejections.non_convex),
        )?)?
        .with_fact(ExplainFact::new(
            "rejected-disconnected",
            FactValue::Count(self.rejections.disconnected),
        )?)?
        .with_fact(ExplainFact::new(
            "rejected-operation-boundary",
            FactValue::Count(self.rejections.impure_member),
        )?)?
        .with_fact(ExplainFact::new(
            "budget-stops",
            FactValue::Count(count(self.budget_stops.len())),
        )?)?;
        let subject = explain.subject(SubjectKind::Region, REGION_FORMATION_SUBJECT)?;
        explain.push_detail(
            RuleRef::builtin(REGION_FORMATION_RULE)?,
            vec![subject],
            ExplainEvent::Check {
                stage: ExplainStage::RegionFormation,
                assessment,
                rejection: RejectionClass::IntrinsicInvalid,
            },
            vec![cause],
        )
    }
}

/// Emits one admitted candidate through the typed explain authority.
fn record_candidate(
    explain: &mut ExplainWriter,
    candidate: &RegionCandidate,
    cause: ExplainRecordId,
) -> Result<ExplainRecordId, ExplainError> {
    let assessment =
        PredicateAssessment::proven("region.connected-convex", EvidenceBasis::CheckedInvariant)?
            .with_fact(ExplainFact::new(
                "member-count",
                FactValue::Count(count(candidate.members.len())),
            )?)?
            .with_fact(ExplainFact::new(
                "boundary-input-count",
                FactValue::Count(count(candidate.boundary_inputs.len())),
            )?)?
            .with_fact(ExplainFact::new(
                "retained-output-count",
                FactValue::Count(count(candidate.retained_outputs.len())),
            )?)?
            .with_fact(ExplainFact::new(
                "producer-duplication",
                FactValue::Boolean(candidate.duplication().enabled()),
            )?)?
            .with_fact(ExplainFact::new(
                "region-content",
                FactValue::Identity(SubjectKey::new(candidate.content.label())?),
            )?)?;
    let subject = explain.subject(SubjectKind::Candidate, &candidate.label)?;
    explain.push_detail(
        RuleRef::builtin(REGION_CANDIDATE_RULE)?,
        vec![subject],
        ExplainEvent::Check {
            stage: ExplainStage::RegionFormation,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        },
        vec![cause],
    )
}

/// One operation in the derived dataflow view.
#[derive(Clone, Debug)]
struct GraphOperation {
    key: OpKey,
    attributes: OperationAttributes,
    operands: Vec<u32>,
    results: Vec<u32>,
    pure: bool,
}

/// The unique definition site of one value, when it has one.
#[derive(Clone, Copy, Debug)]
struct ValueProducer {
    operation: u32,
    result_position: u32,
}

/// One value in the derived dataflow view.
#[derive(Clone, Debug)]
struct GraphValue {
    type_encoding: Box<[u8]>,
    shape: Shape,
    producer: Option<ValueProducer>,
    input_position: Option<u32>,
    consumers: Vec<u32>,
    named_result: bool,
    /// The intra-occurrence site of a staged realization's published value.
    ///
    /// `Some((member, index))` marks a value that exists only inside the
    /// registered law's realization of `member` — the normalization's published
    /// root, the softmax's row maximum — appended to the value table after
    /// every program value so boundary and identity encodings can name it. It
    /// takes no part in operation adjacency: `producer` and `consumers` stay
    /// empty, because its producer and consumer are *stage atoms* of one
    /// occurrence and the topology map is their authority.
    ///
    /// The index counts the occurrence's *published values*, not the reads of
    /// them: a value two stages read is one appended value carrying one site,
    /// which is why the site is a coordinate a reader can compare against the
    /// occurrence's realization rather than an artefact of how often it was
    /// read.
    synthetic_site: Option<(u32, u32)>,
}

/// The stage structure one registered law gives one occurrence.
///
/// Present only for a member whose resolved realization law realizes a region
/// *sequence*; every absent member is single-stage. Derived from the law's own
/// realization — [`VerifiedIndexRegionSequence::stage_sources`] names each
/// stage's reads and [`intermediates`] each handed value — so the compiler
/// carries the law's topology rather than a second derivation of it.
///
/// [`VerifiedIndexRegionSequence::stage_sources`]: tiler_ir::index::VerifiedIndexRegionSequence::stage_sources
/// [`intermediates`]: tiler_ir::index::VerifiedIndexRegionSequence::intermediates
#[derive(Clone, Debug)]
struct StageTopology {
    /// Number of stages the law realizes this occurrence as. Always at least 2.
    stage_count: u32,
    /// Which stage reads each occurrence operand, as `(operand position, stage)`.
    ///
    /// One entry per `StagedInputSource::Occurrence` across every stage's
    /// source list; an operand read by two stages appears twice.
    operand_stages: Vec<(u32, u32)>,
    /// The published values, one record each, in first-read order.
    intermediates: Vec<SyntheticIntermediate>,
}

/// One value a staged realization publishes, with every stage that reads it.
///
/// **The record is per published value, where [`StagedIntermediate`] is per
/// read.** A sequence records one read at a time — a value read by two stages
/// yields two records agreeing on everything but the consuming boundary — and
/// the realization it describes still has *one* value. Synthesizing per read
/// would append two [`GraphValue`]s for it, and then liveness, boundary
/// derivation, and the identity encodings would each see two independent
/// intermediates where the occurrence has one with two readers.
///
/// The reader list lives on this record rather than in a second per-read
/// relation because every consumer asks a question about one published value
/// and its whole reader set: successors of the producing atom are all of them,
/// the value crosses a region's boundary outward exactly when *any* of them
/// stays outside, and it crosses inward exactly when a covered stage is one of
/// them. A per-read side table would answer none of those without joining back
/// to the value it belongs to, and the join is the very thing whose absence is
/// this record's defect.
///
/// [`StagedIntermediate`]: tiler_ir::index::StagedIntermediate
#[derive(Clone, Debug)]
struct SyntheticIntermediate {
    /// Ordinal of the appended [`GraphValue`] carrying its type and shape.
    value: u32,
    /// Stage that publishes it.
    producer_stage: u32,
    /// Every stage that reads it, ascending and distinct.
    ///
    /// Distinct rather than one entry per read: two boundaries of one stage
    /// reading this value are two reads of one value *by one atom*, and the
    /// topology's atoms are stages.
    readers: Vec<u32>,
    /// Last stage across which the value stays live.
    ///
    /// Carried from [`StagedIntermediate::retained_through`] — the span the
    /// sequence derived from its declared readers and checked — rather than
    /// re-derived here, so the compiler reads one authority's answer instead of
    /// spelling a second. [`RegionGraph::attach_stage_topology`] refuses a
    /// realization whose carried span disagrees with the readers it grouped,
    /// which is what keeps the two from drifting silently.
    ///
    /// [`StagedIntermediate::retained_through`]: tiler_ir::index::StagedIntermediate::retained_through
    retained_through: u32,
}

/// A dataflow view over a verified semantic program.
///
/// The view is derived, never authoritative: it copies nothing that the frozen
/// semantic authority did not already validate, and it exists only so region
/// formation can answer adjacency, convexity, and boundary questions without
/// re-walking handles.
#[derive(Clone, Debug)]
pub(crate) struct RegionGraph {
    operations: Vec<GraphOperation>,
    values: Vec<GraphValue>,
    /// Canonical position of every operation, indexed by graph-local ordinal.
    ///
    /// `tiler-ir` states that handles are transient lookup capabilities rather
    /// than stable identity, and a program's stored operation order follows the
    /// order the caller authored it. Two programs that the IR gives one
    /// canonical graph identity can therefore disagree on which slot holds which
    /// operation. Occurrence identity is expressed in these content-derived
    /// canonical positions so it names a graph site rather than an authoring
    /// accident.
    canonical_positions: Vec<u32>,
    /// Stage structure per staged member; an absent member is single-stage.
    stage_topology: BTreeMap<u32, StageTopology>,
    /// First formation node id of each member, member-major, plus one trailing
    /// entry holding the total node count.
    ///
    /// Formation enumerates dense node ids rather than operation ordinals so a
    /// staged occurrence contributes one node per stage. For a program with no
    /// staged member every member has exactly one node and a node id *is* its
    /// member ordinal, which is what keeps every single-stage enumeration —
    /// growth order, budgets, visited sets, and emitted candidates — identical
    /// to what this stage produced before stages existed.
    node_base: Vec<u32>,
}

impl RegionGraph {
    /// Derives the dataflow view of one verified program.
    pub(crate) fn from_program(program: &SemanticProgram) -> Result<Self, RegionError> {
        #[cfg(test)]
        crate::workcount::REGION_GRAPH_BUILDS.record();
        let ordinals: BTreeMap<ValueId, u32> = program
            .values()
            .enumerate()
            .map(|(ordinal, value)| Ok((value.id(), index(ordinal)?)))
            .collect::<Result<_, RegionError>>()?;
        let mut values: Vec<GraphValue> = program
            .values()
            .map(|value| {
                Ok(GraphValue {
                    type_encoding: value
                        .resolved_type()
                        .canonical_encoding()
                        .as_bytes()
                        .to_vec()
                        .into_boxed_slice(),
                    // Every access, tile, and boundary derived below is stated
                    // over fixed extents, so a symbolic value has no graph
                    // record to build rather than a record with a hole in it.
                    shape: value
                        .shape()
                        .as_static()
                        .ok_or(RegionError::Structure {
                            rule: "symbolic-extent",
                        })?
                        .clone(),
                    producer: None,
                    input_position: None,
                    consumers: Vec::new(),
                    named_result: false,
                    synthetic_site: None,
                })
            })
            .collect::<Result<_, RegionError>>()?;
        for (position, input) in program.inputs().enumerate() {
            let value = ordinal(&ordinals, input.value())?;
            value_mut(&mut values, value)?.input_position = Some(index(position)?);
        }
        let mut operations = Vec::with_capacity(program.operation_count());
        for (position, operation) in program.operations().enumerate() {
            let position = index(position)?;
            let definition = program
                .semantic_registry()
                .operation_definition(operation.key())
                .ok_or(RegionError::Structure {
                    rule: "operation-definition",
                })?;
            let mut operands = Vec::with_capacity(operation.operands().len());
            for operand in operation.operands() {
                operands.push(ordinal(&ordinals, operand)?);
            }
            let mut results = Vec::with_capacity(operation.results().len());
            for (result_position, result) in operation.results().enumerate() {
                let result_position = index(result_position)?;
                let value = ordinal(&ordinals, result)?;
                let slot = value_mut(&mut values, value)?;
                if slot.producer.is_some() {
                    return Err(RegionError::Structure {
                        rule: "duplicate-producer",
                    });
                }
                slot.producer = Some(ValueProducer {
                    operation: position,
                    result_position,
                });
                results.push(value);
            }
            operations.push(GraphOperation {
                key: operation.key().clone(),
                attributes: operation.attributes().clone(),
                operands,
                results,
                // Only a referentially transparent occurrence may be evaluated
                // inside a consumer's iteration space, so an effect class this
                // profile cannot prove transparent blocks fusion rather than
                // being approximated.
                pure: matches!(definition.effect(), OperationEffect::Pure),
            });
        }
        for (position, operation) in operations.iter().enumerate() {
            let position = index(position)?;
            for operand in &operation.operands {
                value_mut(&mut values, *operand)?.consumers.push(position);
            }
        }
        for value in &mut values {
            // Operations are visited in ascending order, so repeated operands of
            // one consumer are adjacent duplicates of an already sorted list.
            value.consumers.dedup();
        }
        for output in program.outputs() {
            let value = ordinal(&ordinals, output.value())?;
            value_mut(&mut values, value)?.named_result = true;
        }
        let mut graph = Self {
            operations,
            values,
            canonical_positions: Vec::new(),
            stage_topology: BTreeMap::new(),
            node_base: Vec::new(),
        };
        let whole: Vec<u32> = (0..graph.operation_count()).collect();
        let order = canonical_member_order(&graph, &whole)?;
        graph.canonical_positions = vec![0; order.len()];
        for (position, member) in order.into_iter().enumerate() {
            let slot = graph
                .canonical_positions
                .get_mut(usize::try_from(member).unwrap_or(usize::MAX))
                .ok_or(RegionError::Structure {
                    rule: "canonical-position",
                })?;
            *slot = index(position)?;
        }
        graph.rebuild_node_base()?;
        Ok(graph)
    }

    /// Derives the dataflow view together with each occurrence's realization
    /// stage structure.
    ///
    /// This is [`Self::from_program`] plus one question per operation, asked of
    /// the registered realization-law authority: does this occurrence's law
    /// realize a region *sequence*, and if so what is its stage topology? A
    /// member whose law is absent, unresolvable, or single-region stays
    /// single-stage exactly as `from_program` leaves it — the law's own
    /// refusals fire later, at refinement, where they are attributable; region
    /// formation only needs the shape of what a realization would be.
    ///
    /// The topology is read off the law's own realized sequence rather than
    /// re-derived: `stage_sources` names each stage's occurrence reads and
    /// handed values, and `intermediates` each handed value's type, shape, and
    /// retention span. One synthetic [`GraphValue`] is appended per *published
    /// value* so boundary derivation and identity encoding can name it; it takes
    /// no part in operation adjacency.
    pub(crate) fn with_realizations(
        program: &SemanticProgram,
        laws: &FrozenIndexRealizationLawRegistry,
        contract: &NumericalContractIdentity,
    ) -> Result<Self, RegionError> {
        let mut graph = Self::from_program(program)?;
        for (member, operation) in program.operations().enumerate() {
            let member = index(member)?;
            // A subject this program cannot derive is refinement's refusal to
            // make, with its own typed reason; formation treats the occurrence
            // as single-stage rather than duplicating it.
            let Ok(subject) =
                IndexRefinementSubject::derive(program, operation.id(), contract.clone())
            else {
                continue;
            };
            let Ok(resolved) = laws.resolve(&subject) else {
                continue;
            };
            // The cheap filter first: nine of the ten registered laws are
            // single-region, and realizing one builds a verified region this
            // constructor would discard.
            if !resolved.realizes_region_sequence() {
                continue;
            }
            let Ok(sequence) = resolved.realize_sequence() else {
                continue;
            };
            graph.attach_stage_topology(member, &sequence)?;
        }
        graph.rebuild_node_base()?;
        Ok(graph)
    }

    /// Derives the dataflow view with one member's realization supplied directly.
    ///
    /// The seam [`Self::with_realizations`] reaches through the registered law
    /// authority; this one takes the verified sequence, so a chain shape no
    /// registered law spells yet is reachable without a law standing in for it.
    /// Both build the topology through [`Self::attach_stage_topology`], so what
    /// a test drives here is the production derivation and not a copy of it.
    #[cfg(test)]
    fn with_staged_realization(
        program: &SemanticProgram,
        member: u32,
        sequence: &VerifiedIndexRegionSequence,
    ) -> Result<Self, RegionError> {
        let mut graph = Self::from_program(program)?;
        graph.attach_stage_topology(member, sequence)?;
        graph.rebuild_node_base()?;
        Ok(graph)
    }

    /// Records one member's realized stage topology and appends its handed values.
    ///
    /// **The sequence's records are grouped by published value before anything
    /// is synthesized.** A record names one *read*, and the producing stage
    /// names the value read: a non-final stage publishes exactly one value, so
    /// two records agreeing on `producer` are two reads of one thing. Grouping
    /// on that key is what makes a value with several readers one appended
    /// [`GraphValue`] with several reader stages rather than several values.
    ///
    /// The group order is the order the reads appear in, so the appended values'
    /// sites are exactly what a per-read walk produced for every chain in which
    /// each published value has one reader — which is every chain any registered
    /// law spells today.
    ///
    /// A realization of fewer than two stages records nothing, because an absent
    /// entry is precisely what "this member is single-stage" means here.
    ///
    /// # Errors
    ///
    /// Returns [`RegionError::Structure`] when a stage's declared sources are
    /// unreachable, when two records of one published value disagree about the
    /// value's type, shape, or retention span, or when the carried span is not
    /// the last stage that reads it. Each is invalid realization state rather
    /// than a topology to reconcile. A refusal leaves the appended values in
    /// place, which is safe because both constructors propagate it and drop the
    /// half-built graph rather than returning one.
    fn attach_stage_topology(
        &mut self,
        member: u32,
        sequence: &VerifiedIndexRegionSequence,
    ) -> Result<(), RegionError> {
        let stage_count = index(sequence.stage_count())?;
        if stage_count < 2 {
            return Ok(());
        }
        let mut operand_stages = Vec::new();
        for stage in 0..sequence.stage_count() {
            let sources = sequence
                .stage_sources(stage)
                .ok_or(RegionError::Structure {
                    rule: "stage-sources",
                })?;
            for source in sources {
                if let StagedInputSource::Occurrence(operand) = source {
                    operand_stages.push((index(*operand)?, index(stage)?));
                }
            }
        }
        let mut intermediates: Vec<SyntheticIntermediate> = Vec::new();
        for handed in sequence.intermediates() {
            let producer_stage = index(handed.producer())?;
            let consumer_stage = index(handed.consumer())?;
            let retained_through = index(handed.retained_through())?;
            let encoding = handed.value_type().canonical_encoding();
            let type_encoding = encoding.as_bytes();
            if let Some(position) = intermediates
                .iter()
                .position(|published| published.producer_stage == producer_stage)
            {
                let value = self.value(intermediates[position].value)?;
                // The grouping key is only the value's name if every record
                // under it describes one value. Checked rather than assumed:
                // the first record is what the appended value carries, so a
                // disagreeing later record would otherwise be dropped.
                if value.type_encoding.as_ref() != type_encoding
                    || value.shape != *handed.shape()
                    || intermediates[position].retained_through != retained_through
                {
                    return Err(RegionError::Structure {
                        rule: "intermediate-identity",
                    });
                }
                let readers = &mut intermediates[position].readers;
                if !readers.contains(&consumer_stage) {
                    readers.push(consumer_stage);
                }
                continue;
            }
            let position = index(intermediates.len())?;
            let value = index(self.values.len())?;
            self.values.push(GraphValue {
                type_encoding: type_encoding.to_vec().into_boxed_slice(),
                shape: handed.shape().clone(),
                producer: None,
                input_position: None,
                consumers: Vec::new(),
                named_result: false,
                synthetic_site: Some((member, position)),
            });
            intermediates.push(SyntheticIntermediate {
                value,
                producer_stage,
                readers: vec![consumer_stage],
                retained_through,
            });
        }
        for published in &mut intermediates {
            // Ordered here rather than inherited from the record walk, so no
            // reader of this topology depends on the order the sequence
            // happened to enumerate its reads in.
            published.readers.sort_unstable();
            if published.readers.last() != Some(&published.retained_through) {
                return Err(RegionError::Structure {
                    rule: "intermediate-retention",
                });
            }
        }
        self.stage_topology.insert(
            member,
            StageTopology {
                stage_count,
                operand_stages,
                intermediates,
            },
        );
        Ok(())
    }

    /// Recomputes the member-major node index over the current stage topology.
    fn rebuild_node_base(&mut self) -> Result<(), RegionError> {
        let count = self.operations.len();
        let mut base = Vec::with_capacity(count + 1);
        let mut next = 0_u32;
        for member in 0..count {
            base.push(next);
            let stages = self
                .stage_topology
                .get(&index(member)?)
                .map_or(1, |topology| topology.stage_count);
            next = next
                .checked_add(stages)
                .ok_or(RegionError::Structure { rule: "node-count" })?;
        }
        base.push(next);
        self.node_base = base;
        Ok(())
    }

    /// Returns the number of formation nodes — one per stage atom.
    ///
    /// Equal to [`Self::operation_count`] exactly when no member is staged,
    /// which is what keeps every single-stage enumeration identical to the
    /// pre-stage one.
    pub(crate) fn node_count(&self) -> u32 {
        self.node_base.last().copied().unwrap_or(0)
    }

    /// Returns the attribution atom a formation node id denotes.
    pub(crate) fn node_atom(&self, node: u32) -> Result<SemanticStage, RegionError> {
        // The base list is strictly ascending, so the owning member is the last
        // base at or below the node.
        let member = self
            .node_base
            .partition_point(|base| *base <= node)
            .checked_sub(1)
            .ok_or(RegionError::Structure { rule: "node-id" })?;
        if member >= self.operations.len() {
            return Err(RegionError::Structure { rule: "node-id" });
        }
        let base = self.node_base[member];
        let member = index(member)?;
        let stage = node - base;
        Ok(SemanticStage::at(
            SemanticMemberId(member),
            StageOrdinal(stage),
        ))
    }

    /// Returns the formation node id of one attribution atom.
    pub(crate) fn atom_node(&self, atom: SemanticStage) -> Result<u32, RegionError> {
        let member = usize::try_from(atom.member().0).unwrap_or(usize::MAX);
        let base = self
            .node_base
            .get(member)
            .copied()
            .ok_or(RegionError::Structure { rule: "node-id" })?;
        let stage = atom.stage().get();
        if stage >= self.member_stage_count(atom.member().0) {
            return Err(RegionError::Structure { rule: "node-stage" });
        }
        base.checked_add(stage)
            .ok_or(RegionError::Structure { rule: "node-id" })
    }

    /// Returns how many realization stages one member's occurrence has.
    pub(crate) fn member_stage_count(&self, member: u32) -> u32 {
        self.stage_topology
            .get(&member)
            .map_or(1, |topology| topology.stage_count)
    }

    /// Returns the stage of one member that publishes its occurrence results.
    ///
    /// The final stage, by the sequence contract: every earlier stage publishes
    /// exactly one handed value and only the last writes the occurrence's own
    /// results.
    fn result_publishing_stage(&self, member: u32) -> u32 {
        self.member_stage_count(member).saturating_sub(1)
    }

    /// Returns whether one stage of one member reads the operand at `position`.
    ///
    /// A single-stage member reads every operand at its only stage; a staged
    /// member reads it at exactly the stages its law's source lists name.
    fn stage_reads_operand(&self, member: u32, stage: u32, position: u32) -> bool {
        match self.stage_topology.get(&member) {
            None => stage == 0,
            Some(topology) => topology
                .operand_stages
                .iter()
                .any(|(operand, reader)| *operand == position && *reader == stage),
        }
    }

    /// Returns the node id of the atom that publishes one member's results.
    fn result_node(&self, member: u32) -> Result<u32, RegionError> {
        self.atom_node(SemanticStage::at(
            SemanticMemberId(member),
            StageOrdinal(self.result_publishing_stage(member)),
        ))
    }

    /// Appends the node ids of every atom of `consumer` that reads `value`.
    fn reading_nodes(
        &self,
        consumer: u32,
        value: u32,
        into: &mut Vec<u32>,
    ) -> Result<(), RegionError> {
        let operation = self.operation(consumer)?;
        for (position, operand) in operation.operands.iter().enumerate() {
            if *operand != value {
                continue;
            }
            let position = index(position)?;
            for stage in 0..self.member_stage_count(consumer) {
                if self.stage_reads_operand(consumer, stage, position) {
                    into.push(self.atom_node(SemanticStage::at(
                        SemanticMemberId(consumer),
                        StageOrdinal(stage),
                    ))?);
                }
            }
        }
        Ok(())
    }

    /// Appends the directed successors of one node — the atoms that read what
    /// it publishes.
    ///
    /// A result-publishing atom's successors are every atom reading one of the
    /// occurrence's results; any atom's successors additionally include *every*
    /// reading stage of each handed value it publishes, which for a value two
    /// stages read is two edges leaving one producing atom. For a single-stage
    /// program this is exactly the operation's consumer set.
    fn node_successors(&self, node: u32, into: &mut Vec<u32>) -> Result<(), RegionError> {
        let atom = self.node_atom(node)?;
        let member = atom.member().0;
        let stage = atom.stage().get();
        if stage == self.result_publishing_stage(member) {
            for result in &self.operation(member)?.results {
                for consumer in &self.value(*result)?.consumers {
                    self.reading_nodes(*consumer, *result, into)?;
                }
            }
        }
        if let Some(topology) = self.stage_topology.get(&member) {
            for handed in &topology.intermediates {
                if handed.producer_stage != stage {
                    continue;
                }
                for reader in &handed.readers {
                    into.push(self.atom_node(SemanticStage::at(
                        SemanticMemberId(member),
                        StageOrdinal(*reader),
                    ))?);
                }
            }
        }
        Ok(())
    }

    /// Appends the directed predecessors of one node — the atoms publishing
    /// what it reads.
    fn node_predecessors(&self, node: u32, into: &mut Vec<u32>) -> Result<(), RegionError> {
        let atom = self.node_atom(node)?;
        let member = atom.member().0;
        let stage = atom.stage().get();
        let operation = self.operation(member)?;
        for (position, operand) in operation.operands.iter().enumerate() {
            if !self.stage_reads_operand(member, stage, index(position)?) {
                continue;
            }
            if let Some(producer) = self.value(*operand)?.producer {
                into.push(self.result_node(producer.operation)?);
            }
        }
        if let Some(topology) = self.stage_topology.get(&member) {
            for handed in &topology.intermediates {
                // One edge back to the producer per *value* this stage reads,
                // however many of its boundaries read it.
                if handed.readers.contains(&stage) {
                    into.push(self.atom_node(SemanticStage::at(
                        SemanticMemberId(member),
                        StageOrdinal(handed.producer_stage),
                    ))?);
                }
            }
        }
        Ok(())
    }

    /// Returns whether every atom of `consumer` that reads `value` is in the set.
    ///
    /// The consumers list names operation ordinals; under stages the reading
    /// entity is an atom, and a value read by a stage outside the set is an
    /// external consumption even when the consumer's other stages are inside.
    fn consumer_reads_inside(
        &self,
        nodes: &[u32],
        consumer: u32,
        value: u32,
    ) -> Result<bool, RegionError> {
        let operation = self.operation(consumer)?;
        for (position, operand) in operation.operands.iter().enumerate() {
            if *operand != value {
                continue;
            }
            let position = index(position)?;
            for stage in 0..self.member_stage_count(consumer) {
                if !self.stage_reads_operand(consumer, stage, position) {
                    continue;
                }
                let node = self.atom_node(SemanticStage::at(
                    SemanticMemberId(consumer),
                    StageOrdinal(stage),
                ))?;
                if !is_member(nodes, node) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Returns the number of operations in the observed program.
    pub(crate) fn operation_count(&self) -> u32 {
        u32::try_from(self.operations.len()).unwrap_or(u32::MAX)
    }

    /// Returns the content-derived canonical position of one operation.
    fn canonical_position(&self, member: u32) -> Result<u32, RegionError> {
        self.canonical_positions
            .get(usize::try_from(member).unwrap_or(usize::MAX))
            .copied()
            .ok_or(RegionError::Structure {
                rule: "canonical-position",
            })
    }

    /// Returns the canonical site coordinate of one value.
    ///
    /// A produced value is named by its producer's canonical position and result
    /// position; a program input is named by its ordered interface position; a
    /// staged realization's handed value is named by its occurrence's canonical
    /// position and its *published-value* index, under its own tag so no handed
    /// value can collide with a result or an input.
    ///
    /// **A value several stages read has one coordinate**, because the index
    /// counts what the realization published rather than how often it was read.
    /// The coordinate stays injective for one occurrence: published values are
    /// numbered densely from zero in one pass, so no two of them share an index
    /// and no read of one gets an index of its own.
    fn canonical_value(&self, value: u32) -> Result<(u8, u32, u32), RegionError> {
        let value = self.value(value)?;
        if let Some((member, position)) = value.synthetic_site {
            return Ok((3, self.canonical_position(member)?, position));
        }
        if let Some(producer) = value.producer {
            return Ok((
                1,
                self.canonical_position(producer.operation)?,
                producer.result_position,
            ));
        }
        let input = value.input_position.ok_or(RegionError::Structure {
            rule: "unrooted-value",
        })?;
        Ok((2, input, 0))
    }

    fn operation(&self, member: u32) -> Result<&GraphOperation, RegionError> {
        self.operations
            .get(usize::try_from(member).unwrap_or(usize::MAX))
            .ok_or(RegionError::Structure {
                rule: "member-ordinal",
            })
    }

    fn value(&self, value: u32) -> Result<&GraphValue, RegionError> {
        self.values
            .get(usize::try_from(value).unwrap_or(usize::MAX))
            .ok_or(RegionError::Structure {
                rule: "value-ordinal",
            })
    }

    /// Returns nodes adjacent to the node set through one value edge.
    ///
    /// The result is ascending and deduplicated, which is the order growth
    /// relies on to generate each connected set exactly once.
    fn neighbours(&self, nodes: &[u32]) -> Result<Vec<u32>, RegionError> {
        let mut adjacent = Vec::new();
        let mut edges = Vec::new();
        for node in nodes {
            edges.clear();
            self.node_predecessors(*node, &mut edges)?;
            self.node_successors(*node, &mut edges)?;
            for edge in &edges {
                if !is_member(nodes, *edge) {
                    adjacent.push(*edge);
                }
            }
        }
        adjacent.sort_unstable();
        adjacent.dedup();
        Ok(adjacent)
    }

    /// Returns whether `members` is connected through producer/consumer edges.
    fn is_connected(&self, nodes: &[u32]) -> Result<bool, RegionError> {
        let Some(start) = nodes.first().copied() else {
            return Ok(false);
        };
        // Reachedness is marked by position within the node set rather than by
        // graph ordinal, so the mark vector is the size of the region and the
        // whole traversal touches one cache line for a region of any usual size.
        let mut reached = vec![false; nodes.len()];
        let mut count = 0_usize;
        let mut queue = VecDeque::from([start]);
        let mut edges = Vec::new();
        while let Some(node) = queue.pop_front() {
            // Every enqueued id is a set node, so a miss here is invalid
            // compiler state rather than a set element to skip.
            let position = nodes
                .binary_search(&node)
                .map_err(|_| RegionError::Structure {
                    rule: "member-ordinal",
                })?;
            let slot = reached.get_mut(position).ok_or(RegionError::Structure {
                rule: "member-ordinal",
            })?;
            if std::mem::replace(slot, true) {
                continue;
            }
            count = count.saturating_add(1);
            edges.clear();
            self.node_predecessors(node, &mut edges)?;
            self.node_successors(node, &mut edges)?;
            for edge in &edges {
                if is_member(nodes, *edge) {
                    queue.push_back(*edge);
                }
            }
        }
        Ok(count == nodes.len())
    }

    /// Returns whether no directed path leaves the node set and re-enters it.
    ///
    /// The forward closure of the region through non-members is computed once;
    /// the region is non-convex exactly when that closure reaches a member.
    fn is_convex(&self, nodes: &[u32]) -> Result<bool, RegionError> {
        // Indexed by node id, because the closure ranges over the whole graph
        // rather than over the region.
        let mut visited = vec![false; usize::try_from(self.node_count()).unwrap_or(usize::MAX)];
        let mut queue = VecDeque::new();
        let mut edges = Vec::new();
        for node in nodes {
            edges.clear();
            self.node_successors(*node, &mut edges)?;
            for successor in &edges {
                if !is_member(nodes, *successor) && mark(&mut visited, *successor)? {
                    queue.push_back(*successor);
                }
            }
        }
        while let Some(outside) = queue.pop_front() {
            edges.clear();
            self.node_successors(outside, &mut edges)?;
            for successor in &edges {
                if is_member(nodes, *successor) {
                    return Ok(false);
                }
                if mark(&mut visited, *successor)? {
                    queue.push_back(*successor);
                }
            }
        }
        Ok(true)
    }

    /// Returns the content-derived canonical position of one region member.
    ///
    /// This is the site-independent region-local ordering key: two occurrences
    /// of the same content order their members identically, so a legality
    /// derivation can range over members without leaking an authoring accident.
    pub(crate) fn member_canonical_position(
        &self,
        member: SemanticMemberId,
    ) -> Result<u32, RegionError> {
        self.canonical_position(member.0)
    }

    /// Returns the graph-local ordinals of one member's ordered results.
    ///
    /// The partition search needs a member's own results, not the values its
    /// region exports: a duplicated member whose value is consumed inside the
    /// region that recomputed it appears in no region's retained-output list,
    /// and costing it from that list would report the recomputation as free.
    ///
    /// # Errors
    ///
    /// Returns [`RegionError::Structure`] for an ordinal the graph does not hold.
    pub(crate) fn member_result_values(
        &self,
        member: SemanticMemberId,
    ) -> Result<Vec<SemanticValueId>, RegionError> {
        Ok(self
            .operation(member.0)?
            .results
            .iter()
            .map(|value| SemanticValueId(*value))
            .collect())
    }

    /// Returns how many elements one semantic value holds.
    ///
    /// The count is read from the frozen authority's own shape rather than
    /// recomputed from a region's iteration domain, because the partition search
    /// asks it of values it has not scheduled: a materialized cross-region value
    /// and a recomputed one are sized by the value, not by whichever region
    /// happens to produce it.
    ///
    /// # Errors
    ///
    /// Returns [`RegionError::Structure`] for an ordinal the graph does not hold
    /// and for a shape whose element product is not representable. An
    /// unrepresentable count is refused rather than saturated: a saturated size
    /// would be compared against other sizes as though it were measured.
    pub(crate) fn value_element_count(&self, value: SemanticValueId) -> Result<u64, RegionError> {
        let shape = &self.value(value.0)?.shape;
        tiler_ir::schedule::element_count(shape).map_err(|_| RegionError::Structure {
            rule: "value-element-count",
        })
    }

    /// Returns the borrowed semantic-operation facts of one region member.
    ///
    /// The facts are the read-only projection legality derivation needs: the
    /// operation family key, whether the frozen authority proved the operation
    /// pure, and the canonical encodings of its ordered operand and result value
    /// types. It exposes no graph-local ordinal and no mutable state.
    pub(crate) fn member_operation_facts(
        &self,
        member: SemanticMemberId,
    ) -> Result<MemberOperationFacts<'_>, RegionError> {
        let operation = self.operation(member.0)?;
        let operand_types = operation
            .operands
            .iter()
            .map(|value| self.value(*value).map(|graph| graph.type_encoding.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        let result_types = operation
            .results
            .iter()
            .map(|value| self.value(*value).map(|graph| graph.type_encoding.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(MemberOperationFacts {
            key: &operation.key,
            pure: operation.pure,
            operand_types,
            result_types,
        })
    }
}

/// Borrowed semantic-operation facts of one region member.
///
/// This is a derived read-only view: it copies nothing the frozen semantic
/// authority did not already validate and exists only so a legality derivation
/// can inspect one member's operation family, proven purity, and canonical
/// operand/result value-type encodings without re-walking handles.
#[derive(Clone, Debug)]
pub(crate) struct MemberOperationFacts<'a> {
    key: &'a OpKey,
    pure: bool,
    operand_types: Vec<&'a [u8]>,
    result_types: Vec<&'a [u8]>,
}

impl<'a> MemberOperationFacts<'a> {
    /// Returns the operation family key of the member.
    pub(crate) const fn key(&self) -> &'a OpKey {
        self.key
    }

    /// Returns whether the frozen authority proved the operation pure.
    pub(crate) const fn is_pure(&self) -> bool {
        self.pure
    }

    /// Returns the canonical encodings of the ordered operand value types.
    pub(crate) fn operand_type_encodings(&self) -> &[&'a [u8]] {
        &self.operand_types
    }

    /// Returns the canonical encodings of the ordered result value types.
    pub(crate) fn result_type_encodings(&self) -> &[&'a [u8]] {
        &self.result_types
    }
}

/// The derived boundary of one member set.
struct RegionShape {
    boundary_inputs: Vec<u32>,
    retained_outputs: Vec<RetainedOutput>,
    live_values: u64,
}

/// Runs the deterministic region-formation stage over one verified program.
///
/// The program is never mutated. Candidates are returned in ascending member
/// order, and every emitted candidate is connected, convex, and within the
/// declared budgets.
#[cfg(test)]
pub(crate) fn form_region_candidates(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
) -> Result<RegionFormationOutcome, RegionError> {
    form_over_graph(
        RegionGraph::from_program(program)?,
        budgets,
        numerical_contract,
    )
}

/// Runs region formation with each occurrence's realization stage structure.
///
/// This is the production entry: an occurrence whose registered law realizes a
/// region *sequence* enumerates one node per stage, so the cover search sees
/// the family's internal boundary — the capability Tom's Option A′ decision on
/// `resolve-which-authority-mints-a-multi-stage-region-candidate` makes region
/// formation's to mint. [`form_region_candidates`] stays the law-blind entry
/// for callers exercising formation structure alone; the two differ only in
/// the graph they build.
pub(crate) fn form_region_candidates_with_realizations(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    laws: &FrozenIndexRealizationLawRegistry,
) -> Result<RegionFormationOutcome, RegionError> {
    let contract =
        NumericalContractIdentity::try_from_key(numerical_contract.key).map_err(|_| {
            RegionError::Structure {
                rule: "contract-identity",
            }
        })?;
    form_over_graph(
        RegionGraph::with_realizations(program, laws, &contract)?,
        budgets,
        numerical_contract,
    )
}

fn form_over_graph(
    graph: RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
) -> Result<RegionFormationOutcome, RegionError> {
    #[cfg(test)]
    crate::workcount::REGION_FORMATIONS.record();
    let formed = {
        let mut formation = Formation {
            graph: &graph,
            budgets,
            numerical_contract,
            candidates: Vec::new(),
            stops: BTreeMap::new(),
            rejections: RegionRejectionTally::default(),
            expansions: 0,
        };
        formation.retain_singleton_coverage()?;
        formation.retain_whole_program_coverage()?;
        formation.grow()?;
        formation.finish()?
    };
    Ok(RegionFormationOutcome {
        graph,
        candidates: formed.candidates,
        budget_stops: formed.budget_stops,
        rejections: formed.rejections,
    })
}

/// The graph-independent product of one enumeration run.
struct FormedRegions {
    candidates: Vec<RegionCandidate>,
    budget_stops: Vec<RegionBudgetStop>,
    rejections: RegionRejectionTally,
}

/// Recomputes one candidate from its exact atom set and compares it.
///
/// A stored candidate is never trusted structurally: identity, boundaries,
/// retained outputs, and duplication policy are all rederived from the graph.
/// A staged candidate rebuilds exactly as a single-stage one does, because the
/// identity encodings carry the stage trailer whenever any covered member is
/// staged — the premise the earlier `unencoded-member-stage` refusal guarded
/// is now the encoding rather than a wall.
pub(crate) fn verify_candidate(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    candidate: &RegionCandidate,
) -> Result<(), RegionError> {
    let nodes: Vec<u32> = candidate
        .members
        .iter()
        .map(|atom| graph.atom_node(*atom))
        .collect::<Result<_, _>>()
        .map_err(|_| RegionError::Invalid {
            region: candidate.label.to_string(),
            rule: "membership",
        })?;
    if nodes.is_empty() || nodes.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(RegionError::Invalid {
            region: candidate.label.to_string(),
            rule: "membership",
        });
    }
    let rebuilt = form_candidate(graph, budgets, numerical_contract, &nodes)?;
    match rebuilt {
        Err(rejection) => Err(RegionError::Invalid {
            region: candidate.label.to_string(),
            rule: rejection.rule(),
        }),
        Ok(rebuilt) if rebuilt == *candidate => Ok(()),
        Ok(_) => Err(RegionError::Invalid {
            region: candidate.label.to_string(),
            rule: "identity",
        }),
    }
}

/// The deterministic enumeration state for one compilation request.
struct Formation<'a> {
    graph: &'a RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    candidates: Vec<RegionCandidate>,
    stops: BTreeMap<RegionBudgetResource, RegionBudgetStop>,
    rejections: RegionRejectionTally,
    expansions: u64,
}

impl Formation<'_> {
    /// Emits every singleton region before any growth budget may fire.
    ///
    /// Singleton coverage is unconditional: a budget stops a growth path, and it
    /// never removes the unfused plan.
    fn retain_singleton_coverage(&mut self) -> Result<(), RegionError> {
        for node in 0..self.graph.node_count() {
            match form_candidate(self.graph, self.budgets, self.numerical_contract, &[node])? {
                Ok(candidate) => self.candidates.push(candidate),
                Err(rejection) => {
                    return Err(RegionError::Structure {
                        rule: singleton_defect(rejection),
                    });
                }
            }
        }
        Ok(())
    }

    /// Emits the whole-program region before any growth budget may fire.
    ///
    /// Both extremes of the partition lattice are *coverage* rather than
    /// alternatives, and [`crate::cover`] already treats them that way: it
    /// retains the fully-materialized and the fused cover unconditionally and
    /// bounds only the partitions discovered between them. That guarantee is
    /// empty unless this stage hands it both extremes, because the fused cover
    /// is assembled from a whole-program candidate. Growth reaches that
    /// candidate last — it enumerates breadth-first over set size from the
    /// lowest seed — so charging it against `region_expansions` and
    /// `region_candidates_per_seed` made the one candidate every cover-level
    /// guarantee rests on the first casualty of a truncated search, and for a
    /// program whose only implementable cover is the fused one that cost the
    /// plan rather than an alternative.
    ///
    /// Unlike singleton coverage a rejection here is a legal outcome rather
    /// than a compiler defect. A whole-graph set is trivially convex, but it
    /// can be disconnected, hold an operation this profile cannot prove pure,
    /// or exceed a bound on one region's admissible *shape* —
    /// `region_members`, `region_boundary_outputs`, `region_live_values`. None
    /// of those three bounds a search, so a program they refuse is refused by a
    /// declared property of the profile rather than by where enumeration
    /// happened to stop; each is tallied exactly as growth tallies it.
    fn retain_whole_program_coverage(&mut self) -> Result<(), RegionError> {
        let node_count = self.graph.node_count();
        // A one-node program's singleton already covers it, and forming the
        // same set twice would mint a duplicate candidate rather than a second
        // one. A zero-node graph has no region to form at all.
        if node_count < 2 {
            return Ok(());
        }
        let nodes: Vec<u32> = (0..node_count).collect();
        match form_candidate(self.graph, self.budgets, self.numerical_contract, &nodes)? {
            Ok(candidate) => self.candidates.push(candidate),
            Err(rejection) => self.record_rejection(rejection),
        }
        Ok(())
    }

    /// Grows multi-member regions from every seed in stable topological order.
    fn grow(&mut self) -> Result<(), RegionError> {
        for seed in 0..self.graph.node_count() {
            if self.grow_from(seed)? == GrowthOutcome::ExpansionsExhausted {
                return Ok(());
            }
        }
        Ok(())
    }

    /// Grows every connected set whose minimum member is `seed`.
    ///
    /// Restricting additions to ordinals above the seed generates each connected
    /// set exactly once without losing any: every member of such a set is at or
    /// above the seed, and a connected set can always be built by repeatedly
    /// adding a spanning-tree leaf.
    fn grow_from(&mut self, seed: u32) -> Result<GrowthOutcome, RegionError> {
        let member_limit = u64::from(self.budgets.region_members);
        let seed_limit = u64::from(self.budgets.region_candidates_per_seed);
        let expansion_limit = u64::from(self.budgets.region_expansions);
        // Ascending member vectors rather than `BTreeSet`s. A set is cloned once
        // per expansion attempt and again into the visited set, and the whole
        // search runs under the expansion budget, so the representation of one
        // grown set is multiplied by the search space: a `BTreeSet` of a handful
        // of ordinals costs a node allocation of its own, where the vector costs
        // its ordinals. Ordering is unchanged — both compare lexicographically
        // over ascending ordinals.
        let mut visited: BTreeSet<Vec<u32>> = BTreeSet::from([vec![seed]]);
        let mut queue: VecDeque<Vec<u32>> = VecDeque::from([vec![seed]]);
        let mut emitted = 0_u64;
        while let Some(set) = queue.pop_front() {
            let grown = count(set.len()).saturating_add(1);
            if grown > member_limit {
                self.record_stop(RegionBudgetResource::Members, member_limit, grown);
                continue;
            }
            for neighbour in self.graph.neighbours(&set)? {
                if neighbour <= seed {
                    continue;
                }
                self.expansions = self.expansions.saturating_add(1);
                if self.expansions > expansion_limit {
                    self.record_stop(
                        RegionBudgetResource::Expansions,
                        expansion_limit,
                        self.expansions,
                    );
                    return Ok(GrowthOutcome::ExpansionsExhausted);
                }
                let mut next = set.clone();
                // `neighbours` excludes current members, so the neighbour is
                // absent and its insertion point keeps `next` ascending.
                let Err(position) = next.binary_search(&neighbour) else {
                    return Err(RegionError::Structure {
                        rule: "growth-neighbour",
                    });
                };
                next.insert(position, neighbour);
                if !visited.insert(next.clone()) {
                    continue;
                }
                // Formed before the set is queued rather than after, which lets
                // the queue take the vector instead of a copy of it. Nothing
                // observes the difference: every path that leaves between the
                // two discards the queue.
                let formed =
                    form_candidate(self.graph, self.budgets, self.numerical_contract, &next)?;
                queue.push_back(next);
                match formed {
                    Ok(candidate) => {
                        // Whole-program coverage was retained before growth
                        // started, so a grown set that reaches it is the same
                        // candidate rather than a second one — emitting it
                        // again would collide on its own label. It is not
                        // charged against the per-seed bound either, because
                        // coverage is not an alternative.
                        if candidate.covers_whole_program() {
                            continue;
                        }
                        if emitted == seed_limit {
                            self.record_stop(
                                RegionBudgetResource::CandidatesPerSeed,
                                seed_limit,
                                seed_limit.saturating_add(1),
                            );
                            return Ok(GrowthOutcome::SeedComplete);
                        }
                        emitted = emitted.saturating_add(1);
                        self.candidates.push(candidate);
                    }
                    Err(rejection) => self.record_rejection(rejection),
                }
            }
        }
        Ok(GrowthOutcome::SeedComplete)
    }

    fn record_rejection(&mut self, rejection: RegionRejection) {
        // Growth never proposes a disconnected set, but the reason is tallied
        // separately so a future seeding rule cannot silently reclassify it.
        let tally = match rejection {
            RegionRejection::NonConvex => &mut self.rejections.non_convex,
            RegionRejection::Disconnected => &mut self.rejections.disconnected,
            RegionRejection::ImpureMember => &mut self.rejections.impure_member,
            RegionRejection::Budget(stop) => {
                self.record_stop(stop.resource, stop.limit, stop.actual);
                return;
            }
        };
        *tally = tally.saturating_add(1);
    }

    fn record_stop(&mut self, resource: RegionBudgetResource, limit: u64, actual: u64) {
        let stop = self.stops.entry(resource).or_insert(RegionBudgetStop {
            resource,
            limit,
            actual,
        });
        stop.actual = stop.actual.max(actual);
    }

    /// Orders the emitted candidates and proves their explain labels distinct.
    ///
    /// Candidates are ordered by ascending canonical member positions so the
    /// order, like the identities, does not depend on authoring order. The
    /// graph-local member vector is a deterministic secondary key.
    fn finish(self) -> Result<FormedRegions, RegionError> {
        let mut keyed = Vec::with_capacity(self.candidates.len());
        for candidate in self.candidates {
            let key = canonical_positions(
                self.graph,
                candidate.members.iter().map(|atom| atom.member().0),
            )?;
            keyed.push((key, candidate));
        }
        keyed.sort_by(|left, right| (&left.0, &left.1.members).cmp(&(&right.0, &right.1.members)));
        let candidates: Vec<RegionCandidate> =
            keyed.into_iter().map(|(_, candidate)| candidate).collect();
        let labels: BTreeSet<&str> = candidates
            .iter()
            .map(|candidate| &*candidate.label)
            .collect();
        if labels.len() != candidates.len() {
            return Err(RegionError::Structure {
                rule: "region-label-collision",
            });
        }
        Ok(FormedRegions {
            candidates,
            budget_stops: self.stops.into_values().collect(),
            rejections: self.rejections,
        })
    }
}

/// Whether a seed's growth ended normally or exhausted the request budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GrowthOutcome {
    SeedComplete,
    ExpansionsExhausted,
}

/// Names the structural defect a rejected singleton would represent.
///
/// Singleton coverage is unconditional, so any rejection here is a compiler
/// defect rather than a search outcome.
const fn singleton_defect(rejection: RegionRejection) -> &'static str {
    match rejection {
        RegionRejection::NonConvex => "singleton-convexity",
        RegionRejection::Disconnected => "singleton-connectivity",
        RegionRejection::ImpureMember => "singleton-operation-boundary",
        RegionRejection::Budget(_) => "singleton-budget",
    }
}

/// Classifies one node set and assembles its candidate when it is legal.
fn form_candidate(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    nodes: &[u32],
) -> Result<Result<RegionCandidate, RegionRejection>, RegionError> {
    #[cfg(test)]
    crate::workcount::REGION_CANDIDATE_FORMATIONS.record();
    // The set is *required* ascending and distinct rather than sorted into that
    // shape here. Every caller already produces it that way — singleton
    // coverage, growth, and re-verification alike — so a set arriving in another
    // spelling is invalid compiler state rather than something to canonicalize
    // silently, and requiring it is what lets the stage carry a node set as a
    // slice instead of building a `BTreeSet` per candidate.
    if nodes.is_empty() || nodes.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(RegionError::Structure {
            rule: "member-multiset",
        });
    }
    for node in nodes {
        graph.node_atom(*node)?;
    }
    if let Some(rejection) = classify(graph, budgets, nodes)? {
        return Ok(Err(rejection));
    }
    let shape = region_shape(graph, nodes)?;
    // Singleton coverage is unconditional, so a boundary or live-value budget
    // never removes the unfused plan. It can remove the fused one: these bound
    // one region's admissible *shape* rather than a search, and a program whose
    // only implementable cover is fused is refused by them rather than by where
    // enumeration stopped.
    if nodes.len() > 1
        && let Some(rejection) = classify_shape(budgets, &shape)
    {
        return Ok(Err(rejection));
    }
    assemble(graph, numerical_contract, nodes, shape).map(Ok)
}

/// Decides the structural legality rules that do not need boundary derivation.
fn classify(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    nodes: &[u32],
) -> Result<Option<RegionRejection>, RegionError> {
    let member_limit = u64::from(budgets.region_members);
    let member_count = count(nodes.len());
    // A singleton is one atom alone, so its multiplicity and evaluation order
    // are unchanged and no member budget or purity rule can remove it.
    if member_count > 1 {
        if member_count > member_limit {
            return Ok(Some(RegionRejection::Budget(RegionBudgetStop {
                resource: RegionBudgetResource::Members,
                limit: member_limit,
                actual: member_count,
            })));
        }
        for node in nodes {
            let member = graph.node_atom(*node)?.member().0;
            if !graph.operation(member)?.pure {
                return Ok(Some(RegionRejection::ImpureMember));
            }
        }
        if !graph.is_connected(nodes)? {
            return Ok(Some(RegionRejection::Disconnected));
        }
    }
    if !graph.is_convex(nodes)? {
        return Ok(Some(RegionRejection::NonConvex));
    }
    Ok(None)
}

/// Decides the budgets that depend on the derived boundary.
fn classify_shape(budgets: DeterministicBudgets, shape: &RegionShape) -> Option<RegionRejection> {
    let output_limit = u64::from(budgets.region_boundary_outputs);
    let retained = count(shape.retained_outputs.len());
    if retained > output_limit {
        return Some(RegionRejection::Budget(RegionBudgetStop {
            resource: RegionBudgetResource::BoundaryOutputs,
            limit: output_limit,
            actual: retained,
        }));
    }
    let live_limit = u64::from(budgets.region_live_values);
    if shape.live_values > live_limit {
        return Some(RegionRejection::Budget(RegionBudgetStop {
            resource: RegionBudgetResource::LiveValues,
            limit: live_limit,
            actual: shape.live_values,
        }));
    }
    None
}

/// Derives boundary inputs, retained outputs, and live values for one node set.
///
/// The set is formation node ids. For a single-stage program every node is its
/// member ordinal and this derives exactly what it always has. A staged member
/// contributes per-stage: an operand is read by the atoms its law attributes it
/// to, results are published by the final stage, and a handed intermediate
/// crossing the set's stage boundary becomes a boundary input (a reader inside,
/// producer outside) or a retained output (producer inside, some reader outside)
/// exactly as a real value crossing an occurrence boundary would.
fn region_shape(graph: &RegionGraph, nodes: &[u32]) -> Result<RegionShape, RegionError> {
    let mut boundary_inputs = Vec::new();
    let mut retained_outputs = Vec::new();
    let mut member_results = 0_u64;
    for node in nodes {
        let atom = graph.node_atom(*node)?;
        let member = atom.member().0;
        let stage = atom.stage().get();
        let operation = graph.operation(member)?;
        for (position, operand) in operation.operands.iter().enumerate() {
            if !graph.stage_reads_operand(member, stage, index(position)?) {
                continue;
            }
            let produced_inside =
                graph.value(*operand)?.producer.is_some_and(|producer| {
                    match graph.result_node(producer.operation) {
                        Ok(producing) => is_member(nodes, producing),
                        Err(_) => false,
                    }
                });
            if !produced_inside && !boundary_inputs.contains(operand) {
                boundary_inputs.push(*operand);
            }
        }
        if stage == graph.result_publishing_stage(member) {
            member_results = member_results.saturating_add(count(operation.results.len()));
            for (result_position, result) in operation.results.iter().enumerate() {
                let value = graph.value(*result)?;
                let external_consumers = value.consumers.iter().any(|consumer| {
                    !graph
                        .consumer_reads_inside(nodes, *consumer, *result)
                        .unwrap_or(false)
                });
                if value.named_result || external_consumers {
                    retained_outputs.push(RetainedOutput {
                        value: SemanticValueId(*result),
                        producer: SemanticMemberId(member),
                        result_position: index(result_position)?,
                        named_result: value.named_result,
                        external_consumers,
                    });
                }
            }
        }
        // A handed value crossing the set's stage boundary is a real boundary:
        // the producing atom retains it, a reading atom reads it. A value with
        // several readers crosses each direction once — it is one value, and a
        // region exports or imports it once however many stages want it.
        if let Some(topology) = graph.stage_topology.get(&member) {
            for (position, handed) in topology.intermediates.iter().enumerate() {
                let produced_here = handed.producer_stage == stage;
                let consumed_here = handed.readers.contains(&stage);
                if !produced_here && !consumed_here {
                    continue;
                }
                if produced_here {
                    member_results = member_results.saturating_add(1);
                    let mut escapes = false;
                    for reader in &handed.readers {
                        let node = graph.atom_node(SemanticStage::at(
                            SemanticMemberId(member),
                            StageOrdinal(*reader),
                        ))?;
                        if !is_member(nodes, node) {
                            escapes = true;
                            break;
                        }
                    }
                    // Retained when *any* reader stays outside, and retained
                    // once: the outside readers all read the value this region
                    // published, not one copy each.
                    if escapes {
                        retained_outputs.push(RetainedOutput {
                            value: SemanticValueId(handed.value),
                            producer: SemanticMemberId(member),
                            result_position: index(position)?,
                            named_result: false,
                            external_consumers: true,
                        });
                    }
                }
                if consumed_here {
                    let producer_node = graph.atom_node(SemanticStage::at(
                        SemanticMemberId(member),
                        StageOrdinal(handed.producer_stage),
                    ))?;
                    if !is_member(nodes, producer_node) && !boundary_inputs.contains(&handed.value)
                    {
                        boundary_inputs.push(handed.value);
                    }
                }
            }
        }
    }
    let live_values = count(boundary_inputs.len()).saturating_add(member_results);
    Ok(RegionShape {
        boundary_inputs,
        retained_outputs,
        live_values,
    })
}

/// Builds the identity-bearing candidate for one legal node set.
fn assemble(
    graph: &RegionGraph,
    numerical_contract: StrictF32NumericalContract,
    nodes: &[u32],
    shape: RegionShape,
) -> Result<RegionCandidate, RegionError> {
    let duplication = DuplicationPolicy::Disabled;
    let atoms = nodes
        .iter()
        .map(|node| graph.node_atom(*node))
        .collect::<Result<Vec<_>, _>>()?;
    let mut members: Vec<u32> = atoms.iter().map(|atom| atom.member().0).collect();
    members.dedup();
    let content = encode_content(
        graph,
        numerical_contract,
        &members,
        &atoms,
        &shape,
        duplication,
    )?;
    // Occurrence identity inherits the stage distinction through the content
    // bytes it embeds — two node sets differing in any atom differ in the
    // content trailer — and a handed value in either site group encodes under
    // its own canonical tag, so no occurrence-side trailer is needed.
    let occurrence = encode_occurrence(graph, &content, &members, &shape)?;
    let label = occurrence.label();
    Ok(RegionCandidate {
        members: atoms,
        boundary_inputs: shape
            .boundary_inputs
            .iter()
            .map(|value| SemanticValueId(*value))
            .collect(),
        retained_outputs: shape.retained_outputs,
        duplication,
        content,
        occurrence,
        label,
        program_node_count: graph.node_count(),
    })
}

/// Returns whether any covered member's realization is staged.
///
/// This — not the presence of a non-first atom — is what keys the identity
/// trailer, because a candidate covering only the *first* stage of a staged
/// occurrence computes something different from a candidate covering a
/// single-region occurrence of the same operations, and the two must not share
/// bytes. A graph with no staged member never trips it, which is what keeps
/// every pre-stage encoding byte-identical.
fn covers_staged_member(graph: &RegionGraph, members: &[u32]) -> bool {
    members
        .iter()
        .any(|member| graph.member_stage_count(*member) > 1)
}

/// Appends the stage trailer both identity encodings carry for staged sets.
///
/// Emitted exactly when [`covers_staged_member`] answers true, which is a fact
/// of the graph's registered laws rather than of one candidate's atom spelling
/// — so its presence is decidable from the base bytes' member population and
/// the encoding stays injective: two node sets over one member population
/// differ in the per-atom stage list, and a synthetic value crossing the
/// boundary differs in the site-and-facts list. Appending under a domain
/// marker rather than stepping either domain string is the appended-construct
/// shape: no previously encodable candidate's bytes move.
///
/// **What both region-formation domains encode for a value several stages
/// read.** `tiler.compiler.region-content.v1\0` writes it here once, as one
/// canonical site — its occurrence's canonical position and its published-value
/// index — followed by its type and shape, and never as one entry per read;
/// `tiler.compiler.region-occurrence.v1\0` embeds those content bytes
/// length-framed and lists the same value once more in whichever boundary group
/// it crossed. Neither domain carries a reader count, and neither needs one:
/// which stages read a published value follows from the atom list both
/// encodings already pin, through the graph's stage topology that the atoms are
/// coordinates in. So a candidate whose covered stages read one published value
/// twice encodes as one crossing, which is what it is. Neither domain string
/// steps, because for a chain in which every published value has one reader —
/// every chain any registered law spells — per-read and per-value synthesis
/// agree value for value and index for index, so no previously encodable
/// candidate's bytes move.
fn append_stage_trailer(
    bytes: &mut Vec<u8>,
    graph: &RegionGraph,
    canonical: &[u32],
    atoms: &[SemanticStage],
    shape: &RegionShape,
) -> Result<(), RegionError> {
    bytes.extend_from_slice(b"stages\0");
    push_len(bytes, atoms.len());
    for atom in atoms {
        let position = local_position(canonical, atom.member().0, "trailer-local-member")?;
        bytes.extend_from_slice(&position.to_be_bytes());
        bytes.extend_from_slice(&atom.stage().get().to_be_bytes());
    }
    // The synthetic values crossing this candidate's boundary, by canonical
    // site, each with the facts a boundary encoding carries for a real value.
    let mut synthetic: Vec<u32> = shape
        .boundary_inputs
        .iter()
        .chain(shape.retained_outputs.iter().map(|output| &output.value.0))
        .copied()
        .filter(|value| {
            graph
                .value(*value)
                .is_ok_and(|value| value.synthetic_site.is_some())
        })
        .collect();
    synthetic.sort_unstable();
    synthetic.dedup();
    push_len(bytes, synthetic.len());
    for value in synthetic {
        let (tag, first, second) = graph.canonical_value(value)?;
        bytes.push(tag);
        bytes.extend_from_slice(&first.to_be_bytes());
        bytes.extend_from_slice(&second.to_be_bytes());
        encode_value_facts(bytes, graph.value(value)?);
    }
    Ok(())
}

/// Encodes the region's computation with members in canonical local order.
///
/// Graph-local ordinals follow the authored operation order, which two programs
/// with one canonical semantic-graph identity may spell differently. Content
/// identity therefore renumbers members by [`canonical_member_order`] before
/// encoding, so equal content encodes to equal bytes across those spellings.
fn encode_content(
    graph: &RegionGraph,
    numerical_contract: StrictF32NumericalContract,
    members: &[u32],
    atoms: &[SemanticStage],
    shape: &RegionShape,
    duplication: DuplicationPolicy,
) -> Result<RegionContentIdentity, RegionError> {
    let canonical = canonical_member_order(graph, members)?;
    let mut boundary_order = Vec::with_capacity(shape.boundary_inputs.len());
    for member in &canonical {
        for operand in &graph.operation(*member)?.operands {
            if !boundary_is_internal(graph, members, *operand)? && !boundary_order.contains(operand)
            {
                boundary_order.push(*operand);
            }
        }
    }
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.compiler.region-content.v1\0");
    push_slice(&mut bytes, numerical_contract.key.as_bytes());
    bytes.push(duplication.tag());
    push_len(&mut bytes, canonical.len());
    for member in &canonical {
        let operation = graph.operation(*member)?;
        encode_operation_facts(&mut bytes, operation)?;
        push_len(&mut bytes, operation.operands.len());
        for operand in &operation.operands {
            if let Some(producer) = internal_producer(graph, members, *operand)? {
                let position =
                    local_position(&canonical, producer.operation, "content-local-member")?;
                bytes.push(1);
                bytes.extend_from_slice(&position.to_be_bytes());
                bytes.extend_from_slice(&producer.result_position.to_be_bytes());
            } else {
                let position = local_position(&boundary_order, *operand, "content-local-boundary")?;
                bytes.push(2);
                bytes.extend_from_slice(&position.to_be_bytes());
            }
        }
        push_len(&mut bytes, operation.results.len());
        for result in &operation.results {
            encode_value_facts(&mut bytes, graph.value(*result)?);
        }
    }
    push_len(&mut bytes, boundary_order.len());
    for value in &boundary_order {
        encode_value_facts(&mut bytes, graph.value(*value)?);
    }
    // The base tuple list carries real values only; a staged candidate's handed
    // values live in the stage trailer, which every staged candidate carries.
    let mut retained: Vec<(u32, u32, bool, bool)> = shape
        .retained_outputs
        .iter()
        .filter(|output| {
            graph
                .value(output.value.0)
                .is_ok_and(|value| value.synthetic_site.is_none())
        })
        .map(|output| {
            let position = local_position(&canonical, output.producer.0, "content-local-output")?;
            Ok((
                position,
                output.result_position,
                output.named_result,
                output.external_consumers,
            ))
        })
        .collect::<Result<_, RegionError>>()?;
    retained.sort_unstable();
    push_len(&mut bytes, retained.len());
    for (position, result_position, named_result, external_consumers) in retained {
        bytes.extend_from_slice(&position.to_be_bytes());
        bytes.extend_from_slice(&result_position.to_be_bytes());
        bytes.push(u8::from(named_result));
        bytes.push(u8::from(external_consumers));
    }
    if covers_staged_member(graph, members) {
        append_stage_trailer(&mut bytes, graph, &canonical, atoms, shape)?;
    }
    Ok(RegionContentIdentity {
        canonical: bytes.into(),
    })
}

/// Orders a region's members by refined content rather than by graph position.
///
/// Labels are refined over the region's internal dataflow until they stabilize,
/// which is at most once per member on a DAG. The order is sound: two members
/// share a label only when their whole in-region upstream cone agrees on
/// operation identity, attributes, value facts, and operand positions. It is not
/// complete: a residual tie falls back to graph order, which can give two truly
/// interchangeable occurrences different content identities. Splitting shareable
/// content costs a reuse opportunity; conflating distinct content would be a
/// correctness defect, so the incompleteness is deliberately on the safe side.
fn canonical_member_order(graph: &RegionGraph, members: &[u32]) -> Result<Vec<u32>, RegionError> {
    // `members` is already the ascending member set, which is what lets the
    // producer lookup below be a binary search over it instead of the side
    // `BTreeMap` this used to build once per canonicalization — and lets the
    // function read the caller's slice instead of copying it into one.
    // One buffer with a span per member, rather than a `Vec<u8>` per member.
    // The per-member spelling allocated once per member per canonicalization,
    // and this is called for every region candidate.
    let mut base = Vec::new();
    let mut spans = Vec::with_capacity(members.len());
    for member in members {
        let operation = graph.operation(*member)?;
        let start = base.len();
        encode_operation_facts(&mut base, operation)?;
        push_len(&mut base, operation.results.len());
        for result in &operation.results {
            encode_value_facts(&mut base, graph.value(*result)?);
        }
        spans.push(start..base.len());
    }
    let base_of = |position: usize| &base[spans[position].clone()];
    // Each member's `base` bytes are the same in every round, so they are folded
    // once here and the round only folds the part that actually changes. The
    // straightforward spelling — clone the base, append, re-digest the whole
    // buffer — re-allocated and re-hashed the prefix once per member per round,
    // which is quadratic in the member count for a value that never moves. A
    // sampling profile put this function at 10.6% of the compile path's active
    // self time, above every other function in the crate, with the allocator and
    // `memmove` traffic it generated on top of that.
    let base_digests: Vec<u64> = (0..members.len()).map(|p| digest(base_of(p))).collect();
    let mut labels: Vec<u64> = base_digests.clone();
    // Two label buffers swapped per round, rather than a fresh `Vec` per round.
    // Refinement runs up to once per member, so allocating the refined labels
    // inside the loop cost an allocation per member per canonicalization for a
    // buffer whose length never changes.
    let mut refined: Vec<u64> = Vec::with_capacity(members.len());
    let mut round = Vec::new();
    for _ in 0..members.len() {
        refined.clear();
        for (position, member) in members.iter().enumerate() {
            round.clear();
            round.extend_from_slice(&labels[position].to_be_bytes());
            let operation = graph.operation(*member)?;
            push_len(&mut round, operation.operands.len());
            for operand in &operation.operands {
                if let Some(producer) = internal_producer(graph, members, *operand)? {
                    let source = members.binary_search(&producer.operation).map_err(|_| {
                        RegionError::Structure {
                            rule: "canonical-order-member",
                        }
                    })?;
                    round.push(1);
                    round.extend_from_slice(&labels[source].to_be_bytes());
                    round.extend_from_slice(&producer.result_position.to_be_bytes());
                } else {
                    round.push(2);
                    encode_value_facts(&mut round, graph.value(*operand)?);
                }
            }
            refined.push(digest_from(base_digests[position], &round));
        }
        if refined == labels {
            break;
        }
        std::mem::swap(&mut labels, &mut refined);
    }
    let mut order: Vec<usize> = (0..members.len()).collect();
    order.sort_by(|left, right| {
        (labels[*left], base_of(*left), members[*left]).cmp(&(
            labels[*right],
            base_of(*right),
            members[*right],
        ))
    });
    Ok(order
        .into_iter()
        .map(|position| members[position])
        .collect())
}

/// Returns whether one operation ordinal belongs to an ascending member set.
///
/// Member sets are carried as ascending slices rather than `BTreeSet`s. Every
/// structural rule below tests membership once per operand and once per consumer
/// edge, and a set is classified for every candidate the search reaches, so the
/// test is the innermost operation of region formation. A slice of ordinals
/// bounded by the member budget stays in cache and costs no allocation, where the
/// set cost one node allocation per candidate and a pointer chase per test.
fn is_member(members: &[u32], member: u32) -> bool {
    members.binary_search(&member).is_ok()
}

/// Marks one graph ordinal visited, reporting whether it was newly marked.
///
/// An ordinal outside the graph is invalid compiler state, so it fails closed
/// rather than being treated as already visited.
fn mark(visited: &mut [bool], ordinal: u32) -> Result<bool, RegionError> {
    let slot = visited
        .get_mut(usize::try_from(ordinal).unwrap_or(usize::MAX))
        .ok_or(RegionError::Structure {
            rule: "member-ordinal",
        })?;
    Ok(!std::mem::replace(slot, true))
}

/// Returns the region-local position of one ordinal in a canonical order.
///
/// The orders searched here are the canonical member order and the canonical
/// boundary order, both bounded by the declared member and boundary budgets and
/// both built immediately before the lookup. A scan beats the `BTreeMap` that
/// used to index them: the map cost an allocation and a tree walk per region
/// encoding for a table small enough to sit in a cache line, and the encoding
/// runs once per candidate.
fn local_position(order: &[u32], needle: u32, rule: &'static str) -> Result<u32, RegionError> {
    order
        .iter()
        .position(|value| *value == needle)
        .ok_or(RegionError::Structure { rule })
        .and_then(index)
}

fn encode_operation_facts(
    bytes: &mut Vec<u8>,
    operation: &GraphOperation,
) -> Result<(), RegionError> {
    push_slice(bytes, operation.key.namespace().as_bytes());
    push_slice(bytes, operation.key.name().as_bytes());
    bytes.extend_from_slice(&operation.key.semantic_version().to_be_bytes());
    encode_attributes(bytes, &operation.attributes)
}

fn internal_producer(
    graph: &RegionGraph,
    members: &[u32],
    value: u32,
) -> Result<Option<ValueProducer>, RegionError> {
    Ok(graph
        .value(value)?
        .producer
        .filter(|producer| is_member(members, producer.operation)))
}

fn boundary_is_internal(
    graph: &RegionGraph,
    members: &[u32],
    value: u32,
) -> Result<bool, RegionError> {
    Ok(internal_producer(graph, members, value)?.is_some())
}

/// Encodes the exact graph site of one region in canonical coordinates.
///
/// The member set determines the site, so encoding its canonical positions is
/// injective for one program. Boundary and retained values are derived from that
/// set and are encoded as redundant, independently checkable site facts.
///
/// **The positions below name occurrences, and the stage distinction reaches
/// these bytes through the content encoding they embed.** Two candidates over
/// one member population that differ in any atom differ in
/// [`encode_content`]'s stage trailer, and the embedded content bytes are
/// length-prefixed here, so the pair is separated without an occurrence-side
/// trailer of its own. A handed value crossing the boundary in either site group
/// carries its own canonical tag ([`RegionGraph::canonical_value`]'s tag `3`),
/// so it can never be read as a result or an input.
fn encode_occurrence(
    graph: &RegionGraph,
    content: &RegionContentIdentity,
    members: &[u32],
    shape: &RegionShape,
) -> Result<RegionOccurrenceIdentity, RegionError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.compiler.region-occurrence.v1\0");
    push_slice(&mut bytes, content.as_bytes());
    push_len(&mut bytes, members.len());
    for position in canonical_positions(graph, members.iter().copied())? {
        bytes.extend_from_slice(&position.to_be_bytes());
    }
    // Boundary inputs then retained output values, each group length-prefixed
    // and sorted into canonical site order. One reused site buffer, because the
    // array-of-groups spelling had to materialize the retained output ordinals
    // into a vector of their own just to give both groups one type.
    let mut sites = Vec::new();
    push_len(&mut bytes, shape.boundary_inputs.len());
    encode_canonical_sites(
        &mut bytes,
        graph,
        &mut sites,
        shape.boundary_inputs.iter().copied(),
    )?;
    push_len(&mut bytes, shape.retained_outputs.len());
    encode_canonical_sites(
        &mut bytes,
        graph,
        &mut sites,
        shape.retained_outputs.iter().map(|output| output.value.0),
    )?;
    Ok(RegionOccurrenceIdentity {
        canonical: bytes.into(),
    })
}

/// Appends one group of values as sorted canonical site coordinates.
fn encode_canonical_sites(
    bytes: &mut Vec<u8>,
    graph: &RegionGraph,
    sites: &mut Vec<(u8, u32, u32)>,
    values: impl Iterator<Item = u32>,
) -> Result<(), RegionError> {
    sites.clear();
    for value in values {
        sites.push(graph.canonical_value(value)?);
    }
    sites.sort_unstable();
    for (tag, first, second) in sites.iter() {
        bytes.push(*tag);
        bytes.extend_from_slice(&first.to_be_bytes());
        bytes.extend_from_slice(&second.to_be_bytes());
    }
    Ok(())
}

/// Returns the ascending canonical positions of one member set.
fn canonical_positions(
    graph: &RegionGraph,
    members: impl IntoIterator<Item = u32>,
) -> Result<Vec<u32>, RegionError> {
    let mut positions = members
        .into_iter()
        .map(|member| graph.canonical_position(member))
        .collect::<Result<Vec<_>, _>>()?;
    positions.sort_unstable();
    Ok(positions)
}

fn encode_value_facts(bytes: &mut Vec<u8>, value: &GraphValue) {
    push_slice(bytes, &value.type_encoding);
    push_len(bytes, value.shape.rank());
    for extent in value.shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

/// Appends one occurrence's attribute record in canonical bytes.
///
/// Shared with [`crate::request`] rather than restated there: a recognized
/// occurrence whose family the region vocabulary cannot spell still has to reach
/// the request subject with its attributes intact — `tiler::rms-norm-f32@1`'s
/// `eps` payload is part of what the occurrence *means* — and a second encoder
/// for the same canonical values would be a second authority over the same
/// bytes. It stays fallible for the reason [`encode_canonical_value`] states:
/// the canonical vocabulary is non-exhaustive, so a value this profile cannot
/// encode must refuse rather than produce an identity that drops part of the
/// operation's meaning.
pub(crate) fn encode_attributes(
    bytes: &mut Vec<u8>,
    attributes: &OperationAttributes,
) -> Result<(), RegionError> {
    encode_fields(bytes, attributes.fields(), 0)
}

fn encode_fields(
    bytes: &mut Vec<u8>,
    fields: &[CanonicalField],
    depth: u32,
) -> Result<(), RegionError> {
    push_len(bytes, fields.len());
    for field in fields {
        bytes.extend_from_slice(&field.id().get().to_be_bytes());
        encode_canonical_value(bytes, field.value(), depth)?;
    }
    Ok(())
}

/// Encodes one canonical attribute value, failing closed on unknown shapes.
///
/// `CanonicalValueView` and `CanonicalIntegerWidth` are non-exhaustive, so a
/// value this profile cannot encode rejects the region rather than producing an
/// identity that silently ignores part of the operation's meaning.
fn encode_canonical_value(
    bytes: &mut Vec<u8>,
    value: &CanonicalValue,
    depth: u32,
) -> Result<(), RegionError> {
    if depth >= MAX_CANONICAL_VALUE_DEPTH {
        return Err(RegionError::Structure {
            rule: "canonical-attribute-depth",
        });
    }
    match value.view() {
        CanonicalValueView::Type(resolved) => {
            bytes.push(1);
            push_slice(bytes, resolved.canonical_encoding().as_bytes());
        }
        CanonicalValueView::Bool(value) => {
            bytes.extend_from_slice(&[2, u8::from(value)]);
        }
        CanonicalValueView::Signed { width, bits } => {
            bytes.extend_from_slice(&[3, integer_width_tag(width)?]);
            bytes.extend_from_slice(&bits.to_be_bytes());
        }
        CanonicalValueView::Unsigned { width, bits } => {
            bytes.extend_from_slice(&[4, integer_width_tag(width)?]);
            bytes.extend_from_slice(&bits.to_be_bytes());
        }
        CanonicalValueView::FloatBits(float) => {
            bytes.push(5);
            push_slice(bytes, float.format().namespace().as_bytes());
            push_slice(bytes, float.format().name().as_bytes());
            bytes.extend_from_slice(&float.format().semantic_version().to_be_bytes());
            push_slice(bytes, float.bits());
        }
        CanonicalValueView::Bytes(value) => {
            bytes.push(6);
            push_slice(bytes, value);
        }
        CanonicalValueView::Utf8(value) => {
            bytes.push(7);
            push_slice(bytes, value.as_bytes());
        }
        CanonicalValueView::Sequence(values) => {
            bytes.push(8);
            push_len(bytes, values.len());
            for item in values {
                encode_canonical_value(bytes, item, depth.saturating_add(1))?;
            }
        }
        CanonicalValueView::Record(fields) => {
            bytes.push(9);
            encode_fields(bytes, fields, depth.saturating_add(1))?;
        }
        _ => {
            return Err(RegionError::Structure {
                rule: "canonical-attribute-kind",
            });
        }
    }
    Ok(())
}

const fn integer_width_tag(width: CanonicalIntegerWidth) -> Result<u8, RegionError> {
    match width {
        CanonicalIntegerWidth::Bits8 => Ok(8),
        CanonicalIntegerWidth::Bits16 => Ok(16),
        CanonicalIntegerWidth::Bits32 => Ok(32),
        CanonicalIntegerWidth::Bits64 => Ok(64),
        _ => Err(RegionError::Structure {
            rule: "canonical-integer-width",
        }),
    }
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn index(value: usize) -> Result<u32, RegionError> {
    u32::try_from(value).map_err(|_| RegionError::Structure {
        rule: "graph-ordinal",
    })
}

fn ordinal(ordinals: &BTreeMap<ValueId, u32>, value: ValueId) -> Result<u32, RegionError> {
    ordinals.get(&value).copied().ok_or(RegionError::Structure {
        rule: "value-ordinal",
    })
}

fn value_mut(values: &mut [GraphValue], value: u32) -> Result<&mut GraphValue, RegionError> {
    values
        .get_mut(usize::try_from(value).unwrap_or(usize::MAX))
        .ok_or(RegionError::Structure {
            rule: "value-ordinal",
        })
}

fn digest(bytes: &[u8]) -> u64 {
    digest_from(0xcbf2_9ce4_8422_2325, bytes)
}

/// Renders one digest as a prefixed, zero-padded, lowercase hex label.
///
/// Byte-for-byte what `format!("{prefix}{digest:016x}")` produced. It is spelled
/// out because region formation labels every candidate it forms — the retained
/// ones and the re-verified ones alike — and a `core::fmt` dispatch carrying
/// zero-padding logic costs far more per call than writing sixteen digits.
pub(crate) fn hex_label(prefix: &str, digest: u64) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut label = String::with_capacity(prefix.len().saturating_add(16));
    label.push_str(prefix);
    for nibble in (0..16).rev() {
        let index = usize::try_from((digest >> (nibble * 4)) & 0xf).unwrap_or(0);
        label.push(char::from(DIGITS.get(index).copied().unwrap_or(b'0')));
    }
    label
}

/// Continues an FNV-1a fold over further bytes.
///
/// FNV-1a is a left fold over bytes, so `digest_from(digest(prefix), suffix)`
/// equals `digest(prefix || suffix)` exactly — the state after consuming a
/// prefix *is* that prefix's digest. That identity is what lets
/// [`canonical_member_order`] hash each member's fixed prefix once instead of
/// once per refinement round, without changing a single resulting label.
fn digest_from(state: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(state, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request::{CompilationRequest, verify_planned_request};
    use std::collections::BTreeMap as OracleMap;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// Every way a chain of claims can fail to realize its subject.
    ///
    /// The rule's arms are otherwise driven only by the one chain this profile
    /// builds — a split reduction's two passes — which exercises the accepting
    /// path and none of the refusing ones. Each case below is a chain whose
    /// stages would each verify individually, so nothing but this rule can say
    /// no to any of them.
    #[test]
    fn a_chain_realizes_its_subject_only_when_every_stage_is_accounted_for() {
        let first = SemanticStage::first(SemanticMemberId(1));
        let second = SemanticStage::first(SemanticMemberId(2));
        let subject = [first, second];

        // The single-dispatch case and the split's shape: one claim per subject
        // occurrence, plus any number of later stages sitting behind theirs.
        for claims in [
            vec![first, second],
            vec![second, first],
            vec![first, second, second.next_stage()],
            vec![
                first,
                first.next_stage(),
                first.next_stage().next_stage(),
                second,
            ],
        ] {
            let mut claims = claims;
            assert!(
                chain_realizes_subject(&mut claims, &subject),
                "a chain claiming every occurrence once must realize its subject"
            );
        }

        for (claims, why) in [
            (vec![first], "an occurrence no stage claims"),
            (
                vec![first, second, SemanticStage::first(SemanticMemberId(3))],
                "an occurrence outside the subject",
            ),
            (vec![first, first, second], "one first stage claimed twice"),
            (
                vec![first, second, second.next_stage(), second.next_stage()],
                "one later stage claimed twice",
            ),
            (
                vec![first, second, second.next_stage().next_stage()],
                "a later stage continuing a stage nothing computed",
            ),
            (
                vec![first, second, first.next_stage().next_stage()],
                "a skipped stage on an occurrence that is claimed",
            ),
        ] {
            let mut claims = claims;
            assert!(
                !chain_realizes_subject(&mut claims, &subject),
                "the rule admitted a chain with {why}"
            );
        }
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

    /// The governed serial-sum program with two distinct pointwise constants.
    fn serial_sum_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// The normalized serial-sum program whose pointwise constant is shared.
    fn shared_constant_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let mapped = F32Add::apply(&mut builder, product, constant).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    /// A diamond over operations 1..4 with a private constant at operation 0.
    ///
    /// `1 -> 2 -> 4` and `1 -> 3 -> 4`, so `{1, 2, 4}` must be non-convex.
    fn diamond_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let shared = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let left = F32Multiply::apply(&mut builder, shared, shared).unwrap();
        let right = F32Add::apply(&mut builder, shared, shared).unwrap();
        let joined = F32Add::apply(&mut builder, left, right).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), joined)
            .unwrap();
        builder.build().unwrap()
    }

    /// A shared producer with two consumers that are both named results.
    fn shared_producer_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let shared = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let left = F32Multiply::apply(&mut builder, shared, shared).unwrap();
        let right = F32Add::apply(&mut builder, shared, shared).unwrap();
        builder
            .output(OutputKey::new("left").unwrap(), left)
            .unwrap();
        builder
            .output(OutputKey::new("right").unwrap(), right)
            .unwrap();
        builder.build().unwrap()
    }

    fn form(program: &SemanticProgram) -> RegionFormationOutcome {
        form_with(program, DeterministicBudgets::governed())
    }

    fn form_with(
        program: &SemanticProgram,
        budgets: DeterministicBudgets,
    ) -> RegionFormationOutcome {
        form_region_candidates(program, budgets, StrictF32NumericalContract::governed()).unwrap()
    }

    /// A normalization feeding a pointwise consumer: constant, `rms_norm`, multiply.
    ///
    /// The normalization's registered law realizes two stages, so this program's
    /// four formation nodes over three operations are the smallest staged node
    /// space the standard registries can produce.
    fn rms_norm_program() -> SemanticProgram {
        use tiler_ir::semantic::{
            RMS_NORM_EPS_BITS_ATTRIBUTE, RMS_NORM_REDUCED_AXES_ATTRIBUTE, multiply_f32_op,
            rms_norm_f32_axis_attribute, rms_norm_f32_eps_attribute, rms_norm_f32_op,
        };
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let value = builder
            .input_resolved(
                InputKey::new("value").unwrap(),
                Shape::from_dims([2, 4]),
                F32::resolved_type(),
            )
            .unwrap();
        let weight = builder
            .input_resolved(
                InputKey::new("weight").unwrap(),
                Shape::from_dims([2, 4]),
                F32::resolved_type(),
            )
            .unwrap();
        let attributes = OperationAttributes::new([
            CanonicalField::new(
                RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                rms_norm_f32_axis_attribute(Axis::new(1)),
            ),
            CanonicalField::new(
                RMS_NORM_EPS_BITS_ATTRIBUTE,
                rms_norm_f32_eps_attribute(1.0e-6_f32.to_bits()),
            ),
        ])
        .unwrap();
        let normalized = builder
            .apply(rms_norm_f32_op(), attributes, &[value, weight])
            .unwrap()[0];
        let scaled = builder
            .apply(
                multiply_f32_op(),
                OperationAttributes::new([]).unwrap(),
                &[normalized, value],
            )
            .unwrap()[0];
        builder
            .output_resolved(OutputKey::new("result").unwrap(), scaled)
            .unwrap();
        builder.build().unwrap()
    }

    fn form_staged(program: &SemanticProgram) -> RegionFormationOutcome {
        let laws = tiler_ir::index::FrozenIndexRealizationLawRegistry::from_semantic(
            program.semantic_registry().clone(),
            tiler_ir::index::FrozenScalarRegistry::standard().unwrap(),
        )
        .unwrap();
        form_region_candidates_with_realizations(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            &laws,
        )
        .unwrap()
    }

    /// Returns the graph ordinal of the one normalization member.
    fn rms_member(outcome: &RegionFormationOutcome) -> u32 {
        (0..outcome.graph().operation_count())
            .find(|member| outcome.graph().member_stage_count(*member) > 1)
            .expect("the fixture registers one staged family")
    }

    #[test]
    fn a_staged_occurrence_enumerates_one_node_per_stage() {
        let program = rms_norm_program();
        let outcome = form_staged(&program);
        let graph = outcome.graph();
        // Two operations, one of them two-stage: three nodes.
        assert_eq!(graph.operation_count(), 2);
        assert_eq!(graph.node_count(), 3);
        let member = rms_member(&outcome);
        assert_eq!(graph.member_stage_count(member), 2);
        let topology = graph.stage_topology.get(&member).unwrap();
        // One handed value, published by the fold and read by the scale pass,
        // one element per folded row.
        assert_eq!(topology.intermediates.len(), 1);
        let handed = &topology.intermediates[0];
        assert_eq!(handed.producer_stage, 0);
        assert_eq!(handed.readers, vec![1]);
        assert_eq!(handed.retained_through, 1);
        let synthetic = graph.value(handed.value).unwrap();
        assert_eq!(synthetic.shape, Shape::from_dims([2]));
        assert_eq!(synthetic.synthetic_site, Some((member, 0)));
        // The value operand is read by both stages; the weight by the pass alone.
        let reads = |position: u32| -> Vec<u32> {
            (0..2)
                .filter(|stage| graph.stage_reads_operand(member, *stage, position))
                .collect()
        };
        assert_eq!(
            reads(0),
            vec![0, 1],
            "the fold squares the value and the pass scales it"
        );
        assert_eq!(reads(1), vec![1], "the weight belongs to the pass alone");
        // Every node round-trips through its atom, and coordinates past the
        // topology refuse rather than alias.
        for node in 0..graph.node_count() {
            let atom = graph.node_atom(node).unwrap();
            assert_eq!(graph.atom_node(atom).unwrap(), node);
        }
        assert!(graph.node_atom(graph.node_count()).is_err());
        assert!(
            graph
                .atom_node(SemanticStage::at(SemanticMemberId(member), StageOrdinal(2)))
                .is_err(),
            "a stage the realization does not have is not an atom"
        );
    }

    #[test]
    fn staged_candidates_cover_stages_and_split_boundaries_carry_the_handed_value() {
        let program = rms_norm_program();
        let outcome = form_staged(&program);
        let graph = outcome.graph();
        let member = rms_member(&outcome);
        let handed = graph.stage_topology.get(&member).unwrap().intermediates[0].value;
        let fold = SemanticStage::at(SemanticMemberId(member), StageOrdinal(0));
        let pass = SemanticStage::at(SemanticMemberId(member), StageOrdinal(1));

        let candidate_for = |atoms: &[SemanticStage]| {
            outcome
                .candidates()
                .iter()
                .find(|candidate| candidate.members() == atoms)
        };

        // The fold alone: reads the value input, not the weight, and retains
        // the handed value for the uncovered pass.
        let fold_alone = candidate_for(&[fold]).expect("the fold's singleton is enumerated");
        assert_eq!(
            fold_alone.retained_outputs().len(),
            1,
            "the fold publishes the handed value and nothing else"
        );
        assert_eq!(
            fold_alone.retained_outputs()[0].value,
            SemanticValueId(handed)
        );
        assert!(
            fold_alone
                .boundary_inputs()
                .iter()
                .all(|value| value.0 != handed),
            "the fold does not read what it publishes"
        );
        let weight_ordinal = graph.operation(member).unwrap().operands[1];
        assert!(
            fold_alone
                .boundary_inputs()
                .iter()
                .all(|value| value.0 != weight_ordinal),
            "the weight is not a boundary of a candidate that never reads it"
        );

        // The pass alone: reads the value, the weight, and the handed value.
        let pass_alone = candidate_for(&[pass]).expect("the pass's singleton is enumerated");
        assert!(
            pass_alone
                .boundary_inputs()
                .contains(&SemanticValueId(handed)),
            "the pass reads the handed value across the stage boundary"
        );
        assert!(
            pass_alone
                .boundary_inputs()
                .contains(&SemanticValueId(weight_ordinal))
        );

        // Both stages together: the handed value is internal — neither boundary
        // nor retained — and the occurrence's real result is retained for the
        // outside multiply.
        let whole = candidate_for(&[fold, pass]).expect("the whole occurrence is enumerated");
        assert!(
            whole
                .boundary_inputs()
                .iter()
                .chain(whole.retained_outputs().iter().map(|output| &output.value))
                .all(|value| value.0 != handed),
            "a handed value both published and consumed inside crosses no boundary"
        );
        assert_eq!(whole.retained_outputs().len(), 1);

        // The fold cannot fuse with the downstream multiply around the pass:
        // the value path fold -> pass -> multiply re-enters, so the set is
        // non-convex and formation never emits it.
        let multiply_node = (0..graph.node_count())
            .find(|node| {
                graph.node_atom(*node).is_ok_and(|atom| {
                    atom.member().0 != member
                        && !graph
                            .operation(atom.member().0)
                            .unwrap()
                            .operands
                            .is_empty()
                })
            })
            .expect("the multiply consumes operands");
        let fold_node = graph.atom_node(fold).unwrap();
        let mut set = [fold_node, multiply_node];
        set.sort_unstable();
        let formed = form_candidate(
            graph,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            &set,
        )
        .unwrap();
        assert!(
            matches!(
                formed,
                Err(RegionRejection::NonConvex | RegionRejection::Disconnected)
            ),
            "the fold and the downstream consumer cannot enclose the uncovered pass"
        );
    }

    #[test]
    fn the_stage_trailer_separates_what_shared_bytes_would_conflate() {
        let program = rms_norm_program();
        let staged = form_staged(&program);
        let member = rms_member(&staged);
        let fold = SemanticStage::at(SemanticMemberId(member), StageOrdinal(0));
        let pass = SemanticStage::at(SemanticMemberId(member), StageOrdinal(1));
        let content_of = |outcome: &RegionFormationOutcome, atoms: &[SemanticStage]| {
            outcome
                .candidates()
                .iter()
                .find(|candidate| candidate.members() == atoms)
                .map(|candidate| candidate.content().as_bytes().to_vec())
        };
        let fold_bytes = content_of(&staged, &[fold]).unwrap();
        let pass_bytes = content_of(&staged, &[pass]).unwrap();
        let whole_bytes = content_of(&staged, &[fold, pass]).unwrap();
        assert_ne!(fold_bytes, pass_bytes);
        assert_ne!(fold_bytes, whole_bytes);
        assert_ne!(pass_bytes, whole_bytes);

        // The hazard the trailer condition exists for: a candidate covering
        // only the *first* stage of the staged occurrence must not share bytes
        // with a law-blind candidate over the same operation — the two compute
        // different things. The law-blind formation is the stand-in for a
        // program whose registry carries no law for the family.
        let blind = form(&program);
        let blind_bytes = content_of(&blind, &[SemanticStage::first(SemanticMemberId(member))])
            .expect("the law-blind formation covers the member single-stage");
        assert_ne!(
            blind_bytes, fold_bytes,
            "first-stage-only bytes must not collide with single-stage bytes"
        );
        assert_ne!(blind_bytes, whole_bytes);
    }

    #[test]
    fn an_unstaged_program_forms_identically_with_and_without_the_law_authority() {
        for program in [
            serial_sum_program(),
            diamond_program(),
            shared_producer_program(),
        ] {
            let blind = form(&program);
            let staged = form_staged(&program);
            assert_eq!(
                blind.candidates().len(),
                staged.candidates().len(),
                "a program with no staged member enumerates the same candidates"
            );
            for (left, right) in blind.candidates().iter().zip(staged.candidates()) {
                assert_eq!(left, right, "every candidate is byte-identical");
            }
        }
    }

    #[test]
    fn staged_candidates_rederive_from_their_exact_atoms() {
        let program = rms_norm_program();
        let outcome = form_staged(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        for candidate in outcome.candidates() {
            verify_candidate(outcome.graph(), budgets, contract, candidate).unwrap();
        }
        let member = rms_member(&outcome);
        let fold = SemanticStage::at(SemanticMemberId(member), StageOrdinal(0));
        let pass = SemanticStage::at(SemanticMemberId(member), StageOrdinal(1));
        // A candidate wearing another atom set's identity is refused: the fold
        // singleton relabelled as the pass rebuilds to different bytes.
        let mut forged = outcome
            .candidates()
            .iter()
            .find(|candidate| candidate.members() == [fold])
            .unwrap()
            .clone();
        forged.members = vec![pass];
        assert!(matches!(
            verify_candidate(outcome.graph(), budgets, contract, &forged),
            Err(RegionError::Invalid {
                rule: "identity",
                ..
            })
        ));
    }

    /// The frozen scalar authority the hand-built stages are emitted under.
    fn staged_scalars() -> tiler_ir::index::FrozenScalarRegistry {
        tiler_ir::index::FrozenScalarRegistry::standard().unwrap()
    }

    /// Returns the governed constant's attribute record for one exact payload.
    fn exact_f32_attributes(bits: u32) -> tiler_ir::index::ScalarAttributes {
        use tiler_ir::semantic::{F32_CONSTANT_BITS_ATTRIBUTE, TypeKey};
        tiler_ir::index::ScalarAttributes::new(
            CanonicalValue::record([CanonicalField::new(
                F32_CONSTANT_BITS_ATTRIBUTE,
                CanonicalValue::float_bits(
                    TypeKey::new("tiler", "f32", 1).unwrap(),
                    bits.to_be_bytes(),
                )
                .unwrap(),
            )])
            .unwrap(),
        )
        .unwrap()
    }

    /// Emits `out[r] = fold(+, in[r, *])` over `[rows, columns]`.
    ///
    /// A real reduction rather than a pointwise stand-in, because a chain built
    /// from shape-preserving stages alone could not tell a per-row handed value
    /// from a per-point one, and which is which is what the boundary assertions
    /// below turn on.
    fn row_fold_region(rows: u64, columns: u64) -> tiler_ir::index::VerifiedIndexRegion {
        use tiler_ir::index::{
            DomainRole, IndexRegionBuilder, ScalarAttributes, TensorRole, add_f32_scalar_op,
            constant_f32_scalar_op,
        };
        use tiler_ir::shape::Extent;

        let mut builder = IndexRegionBuilder::new(staged_scalars()).unwrap();
        let row = builder
            .dimension(DomainRole::Parallel, Extent::new(rows))
            .unwrap();
        let column = builder
            .dimension(DomainRole::Reduction, Extent::new(columns))
            .unwrap();
        let row_coordinate = builder.dimension_expr(row).unwrap();
        let column_coordinate = builder.dimension_expr(column).unwrap();
        let input = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([rows]),
            )
            .unwrap();
        let contributor = builder
            .read(input, &[row, column], &[row_coordinate, column_coordinate])
            .unwrap();
        let seed = builder
            .apply(
                constant_f32_scalar_op(),
                exact_f32_attributes(0.0_f32.to_bits()),
                &[],
            )
            .unwrap()
            .get(0)
            .unwrap();
        let folded = builder
            .reduce(&[column], &[seed], &[contributor], |body| {
                let state = body.state(0).unwrap();
                let value = body.contributor(0).unwrap();
                let accumulated = body
                    .apply(
                        add_f32_scalar_op(),
                        ScalarAttributes::empty(),
                        &[state, value],
                    )?
                    .get(0)
                    .unwrap();
                body.yield_values(&[accumulated])
            })
            .unwrap()
            .get(0)
            .unwrap();
        let write = builder.write(output, &[row], &[row_coordinate]).unwrap();
        builder.output(write, folded).unwrap();
        builder.build().unwrap()
    }

    /// Emits `out[r, c] = mul(full[r, c], per_row[r])` over `[rows, columns]`.
    ///
    /// Its input boundary order is `(full, per row)`, which is the order its
    /// stage's sources are declared in.
    fn row_pointwise_region(rows: u64, columns: u64) -> tiler_ir::index::VerifiedIndexRegion {
        use tiler_ir::index::{
            DomainRole, IndexRegionBuilder, ScalarAttributes, TensorRole, multiply_f32_scalar_op,
        };
        use tiler_ir::shape::Extent;

        let mut builder = IndexRegionBuilder::new(staged_scalars()).unwrap();
        let row = builder
            .dimension(DomainRole::Parallel, Extent::new(rows))
            .unwrap();
        let column = builder
            .dimension(DomainRole::Parallel, Extent::new(columns))
            .unwrap();
        let row_coordinate = builder.dimension_expr(row).unwrap();
        let column_coordinate = builder.dimension_expr(column).unwrap();
        let full = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .unwrap();
        let per_row = builder
            .tensor(
                TensorRole::Input,
                F32::resolved_type(),
                Shape::from_dims([rows]),
            )
            .unwrap();
        let output = builder
            .tensor(
                TensorRole::Output,
                F32::resolved_type(),
                Shape::from_dims([rows, columns]),
            )
            .unwrap();
        let element = builder
            .read(full, &[row, column], &[row_coordinate, column_coordinate])
            .unwrap();
        let scale = builder.read(per_row, &[row], &[row_coordinate]).unwrap();
        let product = builder
            .apply(
                multiply_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[element, scale],
            )
            .unwrap()
            .get(0)
            .unwrap();
        let write = builder
            .write(output, &[row, column], &[row_coordinate, column_coordinate])
            .unwrap();
        builder.output(write, product).unwrap();
        builder.build().unwrap()
    }

    /// One `[3, 4]` multiply occurrence, the site the staged chain is attached to.
    fn multi_reader_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([3, 4]))
            .unwrap();
        let product = F32Multiply::apply(&mut builder, input, input).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        builder.build().unwrap()
    }

    /// The four-stage chain whose second stage's value has two readers.
    ///
    /// This is the softmax's staging: `S0` folds a per-row value from the
    /// occurrence input; `S1` reads the input and that value and publishes a
    /// per-point value `e`; `S2` folds `e` into a per-row `d`; and `S3` reads
    /// `e` **again**, alongside `d`, and writes the occurrence's result. The
    /// sequence's own records are the four reads `[(0,1), (1,2), (1,3), (2,3)]`,
    /// over three published values.
    ///
    /// Hand built rather than resolved from a registered law, because no
    /// registered law spells a multi-reader chain yet — the softmax's law is
    /// separate work — and the topology is exactly what region formation reads.
    /// The stages stand for the softmax's in their *boundaries*; the arithmetic
    /// they carry is the fold and the product, which formation never looks at.
    fn multi_reader_sequence() -> VerifiedIndexRegionSequence {
        VerifiedIndexRegionSequence::try_new(
            vec![
                row_fold_region(3, 4),
                row_pointwise_region(3, 4),
                row_fold_region(3, 4),
                row_pointwise_region(3, 4),
            ],
            vec![
                vec![StagedInputSource::Occurrence(0)],
                vec![
                    StagedInputSource::Occurrence(1),
                    StagedInputSource::Intermediate(0),
                ],
                vec![StagedInputSource::Intermediate(1)],
                vec![
                    StagedInputSource::Intermediate(1),
                    StagedInputSource::Intermediate(2),
                ],
            ],
        )
        .expect("the per-point value survives the fold that consumes it")
    }

    fn form_hand_staged(
        program: &SemanticProgram,
        member: u32,
        sequence: &VerifiedIndexRegionSequence,
    ) -> RegionFormationOutcome {
        form_over_graph(
            RegionGraph::with_staged_realization(program, member, sequence).unwrap(),
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap()
    }

    /// The atom of one stage of the hand-staged fixture's only member.
    fn staged_atom(stage: u32) -> SemanticStage {
        SemanticStage::at(SemanticMemberId(0), StageOrdinal(stage))
    }

    /// One published value read by two stages is one synthetic value.
    ///
    /// **The record the sequence hands over is per read, and the realization it
    /// describes has one value.** Four reads over three published values here,
    /// so a per-read synthesis appends four graph values and gives stage one's
    /// published value two of them — two independent intermediates where the
    /// occurrence has one with two readers.
    ///
    /// Observed failing: synthesizing per record instead of per published value
    /// makes `intermediates` four records and the appended synthetic values four
    /// rather than three, with two distinct ordinals produced by stage one.
    #[test]
    fn one_published_value_read_by_two_stages_is_one_synthetic_value() {
        let program = multi_reader_program();
        let sequence = multi_reader_sequence();
        // The sequence layer's own granularity, asserted so the grouping below
        // is proved to be doing something rather than reading an already
        // grouped list.
        assert_eq!(
            sequence
                .intermediates()
                .iter()
                .map(|read| (read.producer(), read.consumer(), read.retained_through()))
                .collect::<Vec<_>>(),
            [(0, 1, 1), (1, 2, 3), (1, 3, 3), (2, 3, 3)]
        );

        let outcome = form_hand_staged(&program, 0, &sequence);
        let graph = outcome.graph();
        assert_eq!(graph.operation_count(), 1);
        assert_eq!(graph.node_count(), 4);
        let topology = graph.stage_topology.get(&0).unwrap();
        assert_eq!(topology.intermediates.len(), 3);

        let published: Vec<(u32, Vec<u32>, u32)> = topology
            .intermediates
            .iter()
            .map(|handed| {
                (
                    handed.producer_stage,
                    handed.readers.clone(),
                    handed.retained_through,
                )
            })
            .collect();
        assert_eq!(
            published,
            [
                (0, vec![1], 1),
                // The value that survives a stage producing something else, with
                // both its readers and the span the sequence checked.
                (1, vec![2, 3], 3),
                (2, vec![3], 3),
            ]
        );

        // One appended graph value per published value, each at its own site.
        let synthetic: Vec<(u32, Option<(u32, u32)>)> = graph
            .values
            .iter()
            .enumerate()
            .filter(|(_, value)| value.synthetic_site.is_some())
            .map(|(ordinal, value)| (index(ordinal).unwrap(), value.synthetic_site))
            .collect();
        assert_eq!(synthetic.len(), 3);
        assert_eq!(
            synthetic
                .iter()
                .map(|(_, site)| site.unwrap())
                .collect::<Vec<_>>(),
            [(0, 0), (0, 1), (0, 2)]
        );
        assert_eq!(
            topology
                .intermediates
                .iter()
                .map(|handed| handed.value)
                .collect::<Vec<_>>(),
            synthetic
                .iter()
                .map(|(ordinal, _)| *ordinal)
                .collect::<Vec<_>>()
        );
        // Both readers read the per-point shape their producer published.
        assert_eq!(
            graph.value(topology.intermediates[1].value).unwrap().shape,
            Shape::from_dims([3, 4])
        );

        // Two edges leave the producing atom, one per reading stage, and each
        // reader has exactly one edge back to it.
        let mut successors = Vec::new();
        graph
            .node_successors(graph.atom_node(staged_atom(1)).unwrap(), &mut successors)
            .unwrap();
        assert_eq!(
            successors,
            vec![
                graph.atom_node(staged_atom(2)).unwrap(),
                graph.atom_node(staged_atom(3)).unwrap()
            ]
        );
        let mut predecessors = Vec::new();
        graph
            .node_predecessors(graph.atom_node(staged_atom(3)).unwrap(), &mut predecessors)
            .unwrap();
        assert_eq!(
            predecessors,
            vec![
                graph.atom_node(staged_atom(1)).unwrap(),
                graph.atom_node(staged_atom(2)).unwrap()
            ]
        );
    }

    /// A two-reader intermediate crosses each region boundary once.
    ///
    /// **This is what the per-read synthesis misdescribes.** A region covering
    /// the producer and one reader would retain a *different* synthetic value
    /// than the one it fed to the reader inside it, a region covering both
    /// readers would import the value twice under two ordinals, and the
    /// producing region would count it twice among its live values.
    #[test]
    fn a_two_reader_intermediate_crosses_each_region_boundary_once() {
        let program = multi_reader_program();
        let sequence = multi_reader_sequence();
        let outcome = form_hand_staged(&program, 0, &sequence);
        let graph = outcome.graph();
        let topology = graph.stage_topology.get(&0).unwrap();
        let seeded = topology.intermediates[0].value;
        let per_point = topology.intermediates[1].value;
        let per_row = topology.intermediates[2].value;

        let candidate_for = |atoms: &[SemanticStage]| {
            outcome
                .candidates()
                .iter()
                .find(|candidate| candidate.members() == atoms)
                .unwrap_or_else(|| panic!("{atoms:?} is a legal set formation must emit"))
        };
        let synthetic_of = |values: &[SemanticValueId]| -> Vec<u32> {
            values
                .iter()
                .filter(|value| {
                    graph
                        .value(value.0)
                        .is_ok_and(|value| value.synthetic_site.is_some())
                })
                .map(|value| value.0)
                .collect()
        };

        // Producer plus one reader: the value leaves once, as the same value the
        // covered reader read.
        let published_and_folded = candidate_for(&[staged_atom(1), staged_atom(2)]);
        assert_eq!(
            synthetic_of(
                &published_and_folded
                    .retained_outputs()
                    .iter()
                    .map(|output| output.value)
                    .collect::<Vec<_>>()
            ),
            vec![per_point, per_row],
            "the per-point value leaves once for the uncovered reader, beside the fold's own"
        );
        assert_eq!(
            synthetic_of(published_and_folded.boundary_inputs()),
            vec![seeded],
            "only the value the uncovered first stage published is imported; the \
             covered reader reads what the covered producer published"
        );

        // Both readers, producer outside: one import, however many stages read it.
        let both_readers = candidate_for(&[staged_atom(2), staged_atom(3)]);
        assert_eq!(
            synthetic_of(both_readers.boundary_inputs()),
            vec![per_point],
            "one published value imported once by the two stages that read it"
        );
        assert_eq!(
            synthetic_of(
                &both_readers
                    .retained_outputs()
                    .iter()
                    .map(|output| output.value)
                    .collect::<Vec<_>>()
            ),
            Vec::<u32>::new(),
            "the fold's value is read by a covered stage and leaves nothing"
        );

        // The final stage alone imports both values it reads, once each.
        let last = candidate_for(&[staged_atom(3)]);
        assert_eq!(
            synthetic_of(last.boundary_inputs()),
            vec![per_point, per_row]
        );

        // The whole realization: every handed value is internal.
        let whole = candidate_for(&[
            staged_atom(0),
            staged_atom(1),
            staged_atom(2),
            staged_atom(3),
        ]);
        assert!(synthetic_of(whole.boundary_inputs()).is_empty());
        assert!(
            synthetic_of(
                &whole
                    .retained_outputs()
                    .iter()
                    .map(|output| output.value)
                    .collect::<Vec<_>>()
            )
            .is_empty()
        );

        // Liveness, which the per-read synthesis inflates: the pair publishes
        // two values and reads two from outside, so four values are live across
        // it — not the five a second copy of the per-point value would make.
        let nodes = [
            graph.atom_node(staged_atom(1)).unwrap(),
            graph.atom_node(staged_atom(2)).unwrap(),
        ];
        assert_eq!(region_shape(graph, &nodes).unwrap().live_values, 4);

        // Every emitted candidate rebuilds from its own atoms, identity trailer
        // included.
        for candidate in outcome.candidates() {
            verify_candidate(
                graph,
                DeterministicBudgets::governed(),
                StrictF32NumericalContract::governed(),
                candidate,
            )
            .unwrap();
        }
    }

    fn member_sets(outcome: &RegionFormationOutcome) -> Vec<Vec<u32>> {
        outcome
            .candidates()
            .iter()
            .map(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .collect()
            })
            .collect()
    }

    /// An independent exhaustive oracle over every nonempty operation subset.
    ///
    /// This deliberately re-derives connectivity and convexity from the program
    /// rather than reusing [`RegionGraph`], so agreement is evidence instead of
    /// a tautology. It is exponential and restricted to tiny fixtures.
    #[allow(
        clippy::too_many_lines,
        reason = "keeps the independent oracle definition readable in one place"
    )]
    fn oracle_legal_sets(program: &SemanticProgram) -> BTreeSet<Vec<u32>> {
        let operations: Vec<(Vec<ValueId>, Vec<ValueId>)> = program
            .operations()
            .map(|operation| {
                (
                    operation.operands().collect(),
                    operation.results().collect(),
                )
            })
            .collect();
        let pure: Vec<bool> = program
            .operations()
            .map(|operation| {
                matches!(
                    program
                        .semantic_registry()
                        .operation_definition(operation.key())
                        .unwrap()
                        .effect(),
                    OperationEffect::Pure
                )
            })
            .collect();
        let mut producer: OracleMap<ValueId, u32> = OracleMap::new();
        for (position, (_, results)) in operations.iter().enumerate() {
            for result in results {
                producer.insert(*result, u32::try_from(position).unwrap());
            }
        }
        let users = |value: ValueId| -> Vec<u32> {
            operations
                .iter()
                .enumerate()
                .filter(|(_, (operands, _))| operands.contains(&value))
                .map(|(position, _)| u32::try_from(position).unwrap())
                .collect()
        };
        let total = u32::try_from(operations.len()).unwrap();
        assert!(total <= 12, "the oracle is restricted to tiny fixtures");
        let mut legal = BTreeSet::new();
        for mask in 1_u32..(1 << total) {
            let selected: Vec<u32> = (0..total).filter(|bit| mask & (1 << bit) != 0).collect();
            let chosen: BTreeSet<u32> = selected.iter().copied().collect();
            if selected.len() > 1 && selected.iter().any(|member| !pure[*member as usize]) {
                continue;
            }
            // Connectivity over the undirected producer/consumer skeleton.
            let mut reached = BTreeSet::from([selected[0]]);
            let mut changed = true;
            while changed {
                changed = false;
                for member in &chosen {
                    if reached.contains(member) {
                        continue;
                    }
                    let touches = operations[*member as usize]
                        .0
                        .iter()
                        .filter_map(|operand| producer.get(operand))
                        .any(|source| reached.contains(source))
                        || operations[*member as usize]
                            .1
                            .iter()
                            .flat_map(|result| users(*result))
                            .any(|user| reached.contains(&user));
                    if touches {
                        reached.insert(*member);
                        changed = true;
                    }
                }
            }
            if reached != chosen {
                continue;
            }
            // Convexity: a forward path may not leave the set and re-enter it.
            let mut leaves_and_reenters = false;
            for start in &chosen {
                let mut work: Vec<(u32, bool)> = operations[*start as usize]
                    .1
                    .iter()
                    .flat_map(|result| users(*result))
                    .map(|user| (user, !chosen.contains(&user)))
                    .collect();
                let mut seen = BTreeSet::new();
                while let Some((node, left)) = work.pop() {
                    if !seen.insert((node, left)) {
                        continue;
                    }
                    if left && chosen.contains(&node) {
                        leaves_and_reenters = true;
                        break;
                    }
                    for result in &operations[node as usize].1 {
                        for user in users(*result) {
                            work.push((user, left || !chosen.contains(&user)));
                        }
                    }
                }
                if leaves_and_reenters {
                    break;
                }
            }
            if !leaves_and_reenters {
                legal.insert(selected);
            }
        }
        legal
    }

    /// Every exact cover in which only overlapping producers may be duplicated.
    ///
    /// The first profile disables duplication, so this stays an oracle-only
    /// completeness witness: it shows the alternative exists and that region
    /// formation reports it as unavailable rather than omitting it silently.
    fn oracle_duplicated_covers(
        program: &SemanticProgram,
        legal: &BTreeSet<Vec<u32>>,
    ) -> Vec<(Vec<Vec<u32>>, BTreeSet<u32>)> {
        let total = u32::try_from(program.operation_count()).unwrap();
        let candidates: Vec<&Vec<u32>> = legal.iter().collect();
        let mut covers = Vec::new();
        for mask in 1_u32..(1 << u32::try_from(candidates.len()).unwrap()) {
            let chosen: Vec<Vec<u32>> = (0..candidates.len())
                .filter(|index| mask & (1 << index) != 0)
                .map(|index| candidates[index].clone())
                .collect();
            let mut occurrences: OracleMap<u32, u32> =
                (0..total).map(|member| (member, 0)).collect();
            for region in &chosen {
                for member in region {
                    *occurrences.get_mut(member).unwrap() += 1;
                }
            }
            if occurrences.values().any(|amount| *amount == 0) {
                continue;
            }
            let overlaps: BTreeSet<u32> = occurrences
                .iter()
                .filter(|(_, amount)| **amount > 1)
                .map(|(member, _)| *member)
                .collect();
            covers.push((chosen, overlaps));
        }
        covers
    }

    #[test]
    fn enumeration_matches_the_exhaustive_oracle_without_budget_pressure() {
        for program in [
            serial_sum_program(),
            shared_constant_program(),
            diamond_program(),
            shared_producer_program(),
        ] {
            let outcome = form(&program);
            assert!(
                outcome.budget_stops().is_empty(),
                "the tiny fixtures must fit the governed budgets"
            );
            let emitted: BTreeSet<Vec<u32>> = member_sets(&outcome).into_iter().collect();
            assert_eq!(
                emitted,
                oracle_legal_sets(&program),
                "bounded enumeration lost a legal region without a budget stop"
            );
        }
    }

    #[test]
    fn every_emitted_candidate_is_oracle_legal_and_singletons_are_complete() {
        for program in [
            serial_sum_program(),
            shared_constant_program(),
            diamond_program(),
            shared_producer_program(),
        ] {
            let outcome = form(&program);
            let legal = oracle_legal_sets(&program);
            for members in member_sets(&outcome) {
                assert!(legal.contains(&members), "emitted an oracle-illegal region");
            }
            for member in 0..u32::try_from(program.operation_count()).unwrap() {
                assert!(
                    member_sets(&outcome).contains(&vec![member]),
                    "singleton coverage is incomplete"
                );
            }
        }
    }

    #[test]
    fn convexity_rejects_a_path_that_leaves_and_reenters_the_region() {
        let program = diamond_program();
        let outcome = form(&program);
        let emitted: BTreeSet<Vec<u32>> = member_sets(&outcome).into_iter().collect();

        assert!(!emitted.contains(&vec![1, 2, 4]));
        assert!(!emitted.contains(&vec![1, 3, 4]));
        assert!(emitted.contains(&vec![1, 2, 3, 4]));
        assert!(outcome.rejections.non_convex > 0);

        let graph = outcome.graph();
        assert!(!graph.is_convex(&[1, 2, 4]).unwrap());
        assert!(graph.is_convex(&[1, 2, 3, 4]).unwrap());
        assert!(!graph.is_connected(&[0, 4]).unwrap());
    }

    #[test]
    fn shared_producers_retain_ordered_multi_result_boundary_outputs() {
        let program = shared_producer_program();
        let outcome = form(&program);
        let whole = outcome
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .eq([1, 2, 3])
            })
            .expect("the multi-output region is legal");

        let retained: Vec<(u32, bool, bool)> = whole
            .retained_outputs()
            .iter()
            .map(|output| {
                (
                    output.producer.0,
                    output.named_result,
                    output.external_consumers,
                )
            })
            .collect();
        assert_eq!(retained, [(2, true, false), (3, true, false)]);
        assert_eq!(whole.boundary_inputs().len(), 2);

        // The producer's own value is retained when a consumer stays outside.
        let split = outcome
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .eq([1, 2])
            })
            .expect("the partial region is legal");
        let retained: Vec<(u32, bool, bool)> = split
            .retained_outputs()
            .iter()
            .map(|output| {
                (
                    output.producer.0,
                    output.named_result,
                    output.external_consumers,
                )
            })
            .collect();
        assert_eq!(retained, [(1, false, true), (2, true, false)]);
    }

    #[test]
    fn overlapping_candidates_are_retained_while_duplication_stays_disabled() {
        let program = shared_producer_program();
        let outcome = form(&program);
        let legal = oracle_legal_sets(&program);
        let covers = oracle_duplicated_covers(&program, &legal);

        // The oracle keeps the duplicated cover as the completeness witness.
        let duplicated = covers
            .iter()
            .find(|(chosen, overlaps)| {
                overlaps == &BTreeSet::from([1])
                    && chosen.iter().collect::<BTreeSet<_>>()
                        == BTreeSet::from([&vec![0], &vec![1, 2], &vec![1, 3]])
            })
            .expect("an explicitly duplicable shared producer has a duplicated cover");
        assert_eq!(duplicated.1, BTreeSet::from([1]));

        // Region formation still proposes both overlapping candidates, and each
        // one declares that this profile may not realize them as a cover.
        for members in [vec![1, 2], vec![1, 3]] {
            let candidate = outcome
                .candidates()
                .iter()
                .find(|candidate| {
                    candidate
                        .members()
                        .iter()
                        .map(|atom| atom.member().0)
                        .eq(members.clone())
                })
                .expect("overlapping candidates are retained");
            assert_eq!(candidate.duplication(), DuplicationPolicy::Disabled);
        }
    }

    #[test]
    fn region_content_identity_is_separate_from_graph_occurrence_identity() {
        let program = shared_producer_program();
        let outcome = form(&program);
        let left = outcome
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .eq([2])
            })
            .unwrap();
        let right = outcome
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .eq([3])
            })
            .unwrap();
        let shared = outcome
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .members()
                    .iter()
                    .map(|atom| atom.member().0)
                    .eq([1])
            })
            .unwrap();

        // `left` multiplies its two boundary reads; `shared` does too, at a
        // different graph site with different retained-output reasons.
        assert_ne!(left.content(), right.content());
        assert_ne!(left.occurrence(), shared.occurrence());
        assert_ne!(left.label(), shared.label());

        // The same content at a different site keeps one content identity.
        let first = form(&serial_sum_program());
        let second = form(&serial_sum_program());
        let contents: Vec<&RegionContentIdentity> = first
            .candidates()
            .iter()
            .map(RegionCandidate::content)
            .collect();
        let repeated: Vec<&RegionContentIdentity> = second
            .candidates()
            .iter()
            .map(RegionCandidate::content)
            .collect();
        assert_eq!(contents, repeated);
    }

    #[test]
    fn identical_content_at_distinct_sites_shares_one_content_identity() {
        // `multiply(x, c)` occurs twice over identical value facts. The two
        // occurrences differ in graph site but describe the same computation.
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let first = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let second = F32Multiply::apply(&mut builder, first, constant).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), second)
            .unwrap();
        let program = builder.build().unwrap();
        let outcome = form(&program);

        let singleton = |outcome: &RegionFormationOutcome, member: u32| {
            outcome
                .candidates()
                .iter()
                .find(|candidate| {
                    candidate
                        .members()
                        .iter()
                        .map(|atom| atom.member().0)
                        .eq([member])
                })
                .unwrap()
                .clone()
        };
        // Operation 1 exports to an in-region consumer; operation 2 exports the
        // named result, so their retained-output reasons differ and their
        // content identities must too.
        assert_ne!(
            singleton(&outcome, 1).content(),
            singleton(&outcome, 2).content()
        );
        assert_ne!(
            singleton(&outcome, 1).occurrence(),
            singleton(&outcome, 2).occurrence()
        );

        // Two independently built copies of one program share both identities.
        let repeat = form(&program);
        assert_eq!(
            singleton(&outcome, 1).content(),
            singleton(&repeat, 1).content()
        );
        assert_eq!(
            singleton(&outcome, 1).occurrence(),
            singleton(&repeat, 1).occurrence()
        );
    }

    #[test]
    fn region_content_is_independent_of_equal_identity_authoring_order() {
        // The two constants are authored in opposite orders. `tiler-ir` gives
        // both programs one canonical graph identity, so the whole-program
        // region content must agree even though the stored operation order does
        // not.
        let build = |reverse: bool| {
            let mut builder = SemanticProgramBuilder::try_standard().unwrap();
            let input = builder
                .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
                .unwrap();
            let (scale, bias) = if reverse {
                let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
                let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
                (scale, bias)
            } else {
                let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
                let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
                (scale, bias)
            };
            let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
            let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
            let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
            builder
                .output(OutputKey::new("result").unwrap(), sum)
                .unwrap();
            builder.build().unwrap()
        };
        let first = build(false);
        let second = build(true);
        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );

        let first = form(&first);
        let second = form(&second);
        let first_whole = first.whole_program_candidate().unwrap();
        let second_whole = second.whole_program_candidate().unwrap();
        assert_eq!(first_whole.content(), second_whole.content());
        assert_eq!(first_whole.occurrence(), second_whole.occurrence());
    }

    /// Both extremes of the partition lattice survive every search bound, and
    /// each bound that fires is reported as a typed stop.
    ///
    /// The two search bounds — `region_candidates_per_seed` and
    /// `region_expansions` — bound which *discovered* partitions exist and can
    /// remove neither the unfused nor the fused plan. The three shape bounds —
    /// `region_members`, `region_boundary_outputs`, `region_live_values` — bound
    /// one region's admissible shape and can remove the fused extreme, which is
    /// a declared property of the profile rather than a place enumeration
    /// stopped. Each arm below asserts which of the two it is.
    #[test]
    fn budget_stops_report_bounded_search_loss_and_keep_both_coverage_extremes() {
        let program = serial_sum_program();
        let complete = oracle_legal_sets(&program);
        let nodes = u32::try_from(program.operation_count()).unwrap();

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_candidates_per_seed = 0;
        let outcome = form_with(&program, budgets);
        let emitted: BTreeSet<Vec<u32>> = member_sets(&outcome).into_iter().collect();
        assert_eq!(emitted.len(), program.operation_count() + 1);
        for member in 0..nodes {
            assert!(emitted.contains(&vec![member]));
        }
        assert!(
            emitted.contains(&(0..nodes).collect::<Vec<u32>>()),
            "a search bound must not cost the fused extreme"
        );
        assert!(emitted.len() < complete.len());
        assert!(
            outcome.budget_stops().iter().any(|stop| stop.resource
                == RegionBudgetResource::CandidatesPerSeed
                && stop.limit == 0
                && stop.actual == 1),
            "lost alternatives must be reported as a typed budget stop"
        );

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_members = 2;
        let outcome = form_with(&program, budgets);
        assert!(
            member_sets(&outcome)
                .iter()
                .all(|members| members.len() <= 2)
        );
        assert!(
            outcome.whole_program_candidate().is_none(),
            "a shape bound below the program's size does refuse the fused region"
        );
        assert!(
            outcome
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == RegionBudgetResource::Members)
        );

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_expansions = 1;
        let outcome = form_with(&program, budgets);
        assert!(
            outcome
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == RegionBudgetResource::Expansions
                    && stop.limit == 1
                    && stop.actual == 2)
        );
        for member in 0..nodes {
            assert!(member_sets(&outcome).contains(&vec![member]));
        }
        assert!(
            outcome.whole_program_candidate().is_some(),
            "an exhausted expansion budget must not cost the fused extreme"
        );

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_boundary_outputs = 0;
        let outcome = form_with(&program, budgets);
        assert_eq!(member_sets(&outcome).len(), program.operation_count());
        assert!(
            outcome
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == RegionBudgetResource::BoundaryOutputs)
        );

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_live_values = 1;
        let outcome = form_with(&program, budgets);
        assert!(
            outcome
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == RegionBudgetResource::LiveValues)
        );
    }

    /// One shared constant and `operations - 1` multiplies chained through it.
    ///
    /// The family `region_members` was measured on
    /// (`spikes/program-planning/identity-growth`), rebuilt here because this
    /// module's bounds are the ones that refused it. Its recognized partition
    /// is the whole program, so the whole-program candidate is the only region
    /// a plan for it can be spelled from.
    fn multiply_chain_program(operations: usize) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let mut current = input;
        for _ in 1..operations {
            current = F32Multiply::apply(&mut builder, current, constant).unwrap();
        }
        builder
            .output(OutputKey::new("result").unwrap(), current)
            .unwrap();
        let program = builder.build().unwrap();
        assert_eq!(program.operation_count(), operations);
        program
    }

    /// `inputs` declared inputs summed left to right, then `extra` self-adds.
    ///
    /// The two knobs move a region's live-value count independently: a further
    /// declared input adds one boundary input *and* one member result, while a
    /// self-add adds a member result alone. That is what lets a whole-program
    /// region land on `region_live_values` exactly and one step past it,
    /// without the member count reaching `region_members` and reporting first.
    fn live_value_program(inputs: usize, extra: usize) -> SemanticProgram {
        assert!(inputs >= 2, "the first add needs two operands");
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let declared: Vec<_> = (0..inputs)
            .map(|ordinal| {
                builder
                    .input::<F32>(
                        InputKey::new(format!("input{ordinal}")).unwrap(),
                        Shape::from_dims([4]),
                    )
                    .unwrap()
            })
            .collect();
        let mut current = declared[0];
        for operand in &declared[1..] {
            current = F32Add::apply(&mut builder, current, *operand).unwrap();
        }
        for _ in 0..extra {
            current = F32Add::apply(&mut builder, current, current).unwrap();
        }
        builder
            .output(OutputKey::new("result").unwrap(), current)
            .unwrap();
        builder.build().unwrap()
    }

    /// Four chained products each read by an operation outside their region.
    ///
    /// The one shape that puts four values across one region's boundary while
    /// leaving every other bound far below its limit: `{t1, t2, t3, t4}` is
    /// connected through the chain and convex — the two consumers are leaves
    /// that never re-enter — and each of its four results is read from outside
    /// it, so the region exports four values and the program declares one.
    fn four_escaping_results_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let first = F32Multiply::apply(&mut builder, input, constant).unwrap();
        let second = F32Multiply::apply(&mut builder, first, constant).unwrap();
        let third = F32Multiply::apply(&mut builder, second, constant).unwrap();
        let fourth = F32Multiply::apply(&mut builder, third, constant).unwrap();
        let lower = F32Add::apply(&mut builder, first, second).unwrap();
        let upper = F32Add::apply(&mut builder, third, fourth).unwrap();
        let root = F32Multiply::apply(&mut builder, lower, upper).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    }

    /// Each derived shape bound admits the region on it and refuses the next.
    ///
    /// **The bound is what these three now are, and a bound that cannot bind is
    /// not one.** `DeterministicBudgets::governed` derives `region_members`
    /// from `semantic_operations`, `region_live_values` from `semantic_values`,
    /// and `region_boundary_outputs` from the declared output count, so each is
    /// wider than the constant it replaced and each must still say no. Every
    /// arm is a *pair*: the region sitting exactly on the bound is emitted, and
    /// the region one step past it is refused with a typed stop naming the
    /// resource, its limit, and the demand it refused.
    ///
    /// The three arms are driven at region formation rather than through
    /// `verify_request`, and deliberately: two of the three derivations are the
    /// program-scoped bound itself, so a program large enough to reach them is
    /// refused by `check_program_budgets` first and the refusal a caller sees
    /// names `semantic-operations` or `semantic-values`. That shadowing is the
    /// collapse `DeterministicBudgets::governed` records; what this test has to
    /// show is that the region bound is still a live gate underneath it.
    #[test]
    fn each_derived_region_shape_bound_admits_its_own_size_and_refuses_one_more() {
        let governed = DeterministicBudgets::governed();
        let stop = |outcome: &RegionFormationOutcome, resource| {
            outcome
                .budget_stops()
                .iter()
                .find(|stop| stop.resource == resource)
                .copied()
        };

        // `region_members` is 62. A sixty-two-operation chain forms its whole
        // program as one region; a sixty-three-operation one does not.
        let admitted = form_with(&multiply_chain_program(62), governed);
        assert!(
            admitted.whole_program_candidate().is_some(),
            "the derived member bound admits a region of exactly its own size",
        );
        assert_eq!(stop(&admitted, RegionBudgetResource::Members), None);
        let refused = form_with(&multiply_chain_program(63), governed);
        assert!(refused.whole_program_candidate().is_none());
        assert_eq!(
            stop(&refused, RegionBudgetResource::Members),
            Some(RegionBudgetStop {
                resource: RegionBudgetResource::Members,
                limit: 62,
                actual: 63,
            }),
        );

        // `region_boundary_outputs` is 3, and it is the one derivation that is
        // not a program-scoped bound in disguise: the whole-program region of
        // the fixture exports one value, and the four-member region inside it
        // exports four. Its three-member prefix exports three and is emitted.
        let outcome = form_with(&four_escaping_results_program(), governed);
        let emitted = member_sets(&outcome);
        assert!(
            emitted.contains(&vec![1, 2, 3]),
            "a region exporting exactly the declared output count is emitted",
        );
        assert!(
            !emitted.contains(&vec![1, 2, 3, 4]),
            "a region exporting one value more than the program declares is refused",
        );
        assert_eq!(
            stop(&outcome, RegionBudgetResource::BoundaryOutputs),
            Some(RegionBudgetStop {
                resource: RegionBudgetResource::BoundaryOutputs,
                limit: 3,
                actual: 4,
            }),
        );
        assert!(
            outcome.whole_program_candidate().is_some(),
            "the bound refuses a grown candidate and never the fused extreme",
        );

        // `region_live_values` is 80. Forty declared inputs and one self-add
        // put exactly eighty values across the whole-program region — forty
        // boundary inputs and forty member results — and a second self-add
        // puts eighty-one.
        let admitted = form_with(&live_value_program(40, 1), governed);
        assert!(admitted.whole_program_candidate().is_some());
        assert_eq!(stop(&admitted, RegionBudgetResource::LiveValues), None);
        let refused = form_with(&live_value_program(40, 2), governed);
        assert!(refused.whole_program_candidate().is_none());
        assert_eq!(
            stop(&refused, RegionBudgetResource::LiveValues),
            Some(RegionBudgetStop {
                resource: RegionBudgetResource::LiveValues,
                limit: 80,
                actual: 81,
            }),
        );
    }

    /// Forms three times and reports the fastest run's cost per checked set.
    ///
    /// The fastest of the three rather than the mean: every repetition does
    /// identical work — region formation is a pure function of the program,
    /// budgets, and contract — so the spread between them is the host's, and
    /// the minimum is the estimator a busy host disturbs least.
    fn formation_cost(
        program: &SemanticProgram,
        budgets: DeterministicBudgets,
    ) -> (RegionFormationOutcome, usize, u128) {
        let mut best: Option<(RegionFormationOutcome, usize, u128)> = None;
        for _ in 0..3 {
            let started = std::time::Instant::now();
            let (outcome, checked) = crate::workcount::REGION_CANDIDATE_FORMATIONS
                .observe(|| form_with(program, budgets));
            let nanos = started.elapsed().as_nanos() / checked as u128;
            if best.as_ref().is_none_or(|(_, _, slowest)| nanos < *slowest) {
                best = Some((outcome, checked, nanos));
            }
        }
        best.expect("three repetitions leave a fastest one")
    }

    /// The wider admissible region does not raise the cost of one candidate.
    ///
    /// **The one risk the deciding ticket named to measure rather than
    /// assume.** `region_candidates_per_seed` and `region_expansions` bound
    /// search *work* and neither moved, so what a wider shape bound could cost
    /// is the price of checking each candidate that is now legal.
    ///
    /// The deterministic half is the assertion and the timing half only
    /// corroborates it, for the reason `crate::workcount` states: a count does
    /// not move with the host. Below the superseded `region_members` of 32 the
    /// emitted candidate population is *identical* under both budget sets, and
    /// `form_candidate` is a pure function of the graph, the budgets, and the
    /// contract — so an unchanged population over an unchanged code path is
    /// unchanged per-candidate work, whatever the clock says.
    ///
    /// The printed rows carry the newly admitted sizes too, because "the
    /// previously admitted population pays nothing" is only half the question a
    /// reader asks. The denominator is
    /// [`crate::workcount::REGION_CANDIDATE_FORMATIONS`] — the number of node
    /// sets actually checked — rather than the emitted candidate count, which
    /// would price every rejected set into the survivors. Each row is the
    /// fastest of three repetitions, which is the estimator least disturbed by
    /// a busy host; the host is named in the ticket, not here, because a
    /// recorded timing is evidence about the host that took it.
    #[test]
    fn the_derived_shape_bounds_leave_the_previously_admitted_candidates_untouched() {
        let governed = DeterministicBudgets::governed();
        // The constants these three replaced, kept here rather than in the
        // profile: the comparison is against what the profile *was*, and a
        // reader must be able to see both sides of it in one place.
        let superseded = DeterministicBudgets {
            region_members: 32,
            region_boundary_outputs: 8,
            region_live_values: 64,
            ..governed
        };
        for operations in [8, 16, 24, 32] {
            let program = multiply_chain_program(operations);
            let (old, old_checked, old_nanos) = formation_cost(&program, superseded);
            let (new, new_checked, new_nanos) = formation_cost(&program, governed);
            assert_eq!(
                member_sets(&old),
                member_sets(&new),
                "the derived bounds changed which candidates a {operations}-operation \
                 chain forms, so the populations are not comparable",
            );
            assert_eq!(
                old_checked, new_checked,
                "the derived bounds changed how many node sets are checked at \
                 {operations} operations",
            );
            println!(
                "MEASURE chain n={operations} checked={new_checked} \
                 emitted={} superseded={old_nanos}ns/check derived={new_nanos}ns/check",
                new.candidates().len(),
            );
        }
        // The range the derivation admits and the constants refused. No
        // comparison row exists for these: under the superseded bounds the
        // whole-program candidate was never formed at all.
        for operations in [40, 48, 62] {
            let program = multiply_chain_program(operations);
            let (outcome, checked, nanos) = formation_cost(&program, governed);
            assert!(
                form_with(&program, superseded)
                    .whole_program_candidate()
                    .is_none(),
                "the superseded constant refused this size, which is the point",
            );
            assert!(outcome.whole_program_candidate().is_some());
            println!(
                "MEASURE chain n={operations} checked={checked} emitted={} \
                 derived={nanos}ns/check",
                outcome.candidates().len(),
            );
        }
    }

    #[test]
    fn enumeration_is_deterministic_and_independent_of_authoring_order() {
        let first = form(&serial_sum_program());
        let second = form(&serial_sum_program());
        assert_eq!(member_sets(&first), member_sets(&second));
        assert_eq!(
            first
                .candidates()
                .iter()
                .map(RegionCandidate::label)
                .collect::<Vec<_>>(),
            second
                .candidates()
                .iter()
                .map(RegionCandidate::label)
                .collect::<Vec<_>>()
        );
        assert_eq!(first.candidates().len(), 17);
        assert_eq!(form(&shared_constant_program()).candidates().len(), 10);
    }

    #[test]
    fn candidates_are_rederived_from_their_exact_contents() {
        let program = serial_sum_program();
        let outcome = form(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();
        for candidate in outcome.candidates() {
            verify_candidate(outcome.graph(), budgets, contract, candidate).unwrap();
        }

        let whole = outcome.whole_program_candidate().unwrap();
        assert_eq!(whole.members().len(), program.operation_count());

        let mut forged = whole.clone();
        forged.label = Arc::from(format!("{}-forged", forged.label));
        assert!(matches!(
            verify_candidate(outcome.graph(), budgets, contract, &forged),
            Err(RegionError::Invalid {
                rule: "identity",
                ..
            })
        ));

        let mut forged = whole.clone();
        forged.retained_outputs.clear();
        assert!(matches!(
            verify_candidate(outcome.graph(), budgets, contract, &forged),
            Err(RegionError::Invalid {
                rule: "identity",
                ..
            })
        ));

        let mut forged = whole.clone();
        forged.members.swap(0, 1);
        assert!(matches!(
            verify_candidate(outcome.graph(), budgets, contract, &forged),
            Err(RegionError::Invalid {
                rule: "membership",
                ..
            })
        ));

        // A candidate carrying a stage its member's realization does not have
        // is refused as bad membership: the graph's stage topology is the
        // authority on which atoms exist, and every member of this program is
        // single-stage. A *real* later stage rebuilds like any other atom set,
        // because the identity encodings carry the stage trailer for any
        // candidate touching a staged member.
        let mut staged = whole.clone();
        staged.members[0] = staged.members[0].next_stage();
        assert!(matches!(
            verify_candidate(outcome.graph(), budgets, contract, &staged),
            Err(RegionError::Invalid {
                rule: "membership",
                ..
            })
        ));

        let diamond = diamond_program();
        let diamond_outcome = form(&diamond);
        let mut nonconvex = diamond_outcome.candidates()[0].clone();
        nonconvex.members = vec![
            SemanticStage::first(SemanticMemberId(1)),
            SemanticStage::first(SemanticMemberId(2)),
            SemanticStage::first(SemanticMemberId(4)),
        ];
        assert!(matches!(
            verify_candidate(diamond_outcome.graph(), budgets, contract, &nonconvex),
            Err(RegionError::Invalid {
                rule: "convexity",
                ..
            })
        ));
    }

    #[test]
    fn a_different_numerical_contract_changes_region_content_identity() {
        let program = serial_sum_program();
        let governed = form(&program);
        let mut contract = StrictF32NumericalContract::governed();
        contract.key = "tiler.test-contract.v1";
        let other =
            form_region_candidates(&program, DeterministicBudgets::governed(), contract).unwrap();

        assert_eq!(member_sets(&governed), member_sets(&other));
        assert_ne!(
            governed.candidates()[0].content(),
            other.candidates()[0].content()
        );
    }

    #[test]
    fn stage_records_are_typed_bounded_and_causally_chained() {
        let program = serial_sum_program();
        let outcome = form(&program);
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target).unwrap();

        let root = test_root(&mut explain);
        let records = outcome.record(&mut explain, root).unwrap();
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

        assert_eq!(
            trace
                .records()
                .iter()
                .filter(|record| record.rule().key().as_str() == REGION_CANDIDATE_RULE)
                .count(),
            17
        );
        let summary = trace
            .records()
            .iter()
            .find(|record| record.id() == records.summary)
            .unwrap();
        assert_eq!(
            summary.subjects()[0].key().as_str(),
            REGION_FORMATION_SUBJECT
        );
        let ExplainEvent::Check { assessment, .. } = summary.event() else {
            panic!("the stage receipt is a checked assertion");
        };
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "candidate-count" && matches!(fact.value(), FactValue::Count(17))
        }));
        let whole = trace
            .records()
            .iter()
            .find(|record| record.id() == records.whole_program.unwrap())
            .unwrap();
        let ExplainEvent::Check { assessment, .. } = whole.event() else {
            panic!("candidate records are checked assertions");
        };
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "producer-duplication"
                && matches!(fact.value(), FactValue::Boolean(false))
        }));
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "region-content"
                && matches!(fact.value(), FactValue::Identity(key)
                    if key.as_str().starts_with("region-content:"))
        }));
        assert!(trace.render().contains("region-formation admitted"));
    }

    #[test]
    fn budget_stops_are_rendered_as_typed_region_formation_events() {
        let program = serial_sum_program();
        let mut budgets = DeterministicBudgets::governed();
        budgets.region_candidates_per_seed = 0;
        let outcome = form_with(&program, budgets);
        let verified = verify_planned_request(CompilationRequest::governed(&program)).unwrap();
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

        assert!(
            trace
                .render()
                .contains("budget-stop:region-candidates-per-seed:0:1")
        );
    }

    #[test]
    fn errors_report_their_exact_class_and_rule() {
        let error = RegionError::Structure {
            rule: "value-ordinal",
        };
        assert_eq!(error.reason(), "value-ordinal");
        assert_eq!(error.class(), "structure");
        assert_eq!(
            error.to_string(),
            "compile.region.structure.value-ordinal: deterministic region formation observed invalid compiler output"
        );
        let error = RegionError::Invalid {
            region: "region:0000000000000000".to_owned(),
            rule: "convexity",
        };
        assert_eq!(error.class(), "invalid");
        assert_eq!(
            error.to_string(),
            "compile.region.invalid.convexity: region:0000000000000000 rejected"
        );
    }

    /// The single-lookup value ordinal is the one the graph view assigns.
    ///
    /// [`value_ordinal`] and [`RegionGraph::from_program`]'s bulk mapping are two
    /// spellings of one coordinate, and nothing in the type system holds them
    /// together — so this reads the graph's own per-value record *through* the
    /// helper's answer. A helper returning a different ordinal indexes a
    /// different value's record, and every arm below then reports about the
    /// wrong slot.
    ///
    /// Observed failing: shifting [`value_ordinal`] by one slot fails this test,
    /// because the fixture's declared input sits at one end of its value list
    /// and its declared output at the other with unmarked values between them.
    #[test]
    fn the_value_ordinal_lookup_indexes_the_graph_view_s_own_record() {
        let program = serial_sum_program();
        let graph = RegionGraph::from_program(&program).expect("the fixture is a valid graph");

        for (position, input) in program.inputs().enumerate() {
            let ordinal = value_ordinal(&program, input.value()).expect("a declared input");
            assert_eq!(
                graph.values[usize::try_from(ordinal.0).unwrap()].input_position,
                Some(u32::try_from(position).unwrap()),
            );
        }
        for output in program.outputs() {
            let ordinal = value_ordinal(&program, output.value()).expect("a declared output");
            assert!(graph.values[usize::try_from(ordinal.0).unwrap()].named_result);
        }

        // The marks are sparse, which is what makes the assertions above
        // discriminating rather than vacuous.
        let marked = graph
            .values
            .iter()
            .filter(|value| value.named_result || value.input_position.is_some())
            .count();
        assert_eq!(marked, 2);
        assert!(graph.values.len() > marked);

        // A value of another program is not one this program holds, which is the
        // fail-closed direction the helper answers `None` for.
        let other = shared_constant_program();
        let foreign = other.outputs().next().expect("one declared output").value();
        assert_eq!(value_ordinal(&program, foreign), None);
    }
}
