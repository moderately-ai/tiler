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

/// Graph-local ordinal of one semantic value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct SemanticValueId(pub(crate) u32);

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
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionContentIdentity {
    canonical: Box<[u8]>,
}

impl RegionContentIdentity {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.canonical
    }

    /// Returns a bounded explain label for this content.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn key(&self) -> String {
        format!("region-content:{:016x}", digest(&self.canonical))
    }
}

/// Collision-free canonical identity of one region occurrence in one graph.
///
/// This is region content plus the exact graph site: member ordinals, boundary
/// input values, and retained output values.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionOccurrenceIdentity {
    canonical: Box<[u8]>,
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
    fn key(&self) -> String {
        format!("region:{:016x}", digest(&self.canonical))
    }
}

/// One proposed connected convex region over a verified semantic DAG.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionCandidate {
    members: Vec<SemanticMemberId>,
    boundary_inputs: Vec<SemanticValueId>,
    retained_outputs: Vec<RetainedOutput>,
    duplication: DuplicationPolicy,
    content: RegionContentIdentity,
    occurrence: RegionOccurrenceIdentity,
    stable_id: String,
    program_operation_count: u32,
}

impl RegionCandidate {
    /// Returns the region's members in ascending graph-local order.
    pub(crate) fn members(&self) -> &[SemanticMemberId] {
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
    pub(crate) fn stable_id(&self) -> &str {
        &self.stable_id
    }

    /// Returns whether the region covers every operation of its program.
    pub(crate) fn covers_whole_program(&self) -> bool {
        u32::try_from(self.members.len()).is_ok_and(|count| count == self.program_operation_count)
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
    const fn key(self) -> &'static str {
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
    pub(crate) summary: Option<ExplainRecordId>,
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
    /// A whole-graph set is trivially convex, so it is absent only when the
    /// graph is disconnected, an operation is not provably pure, or a budget
    /// stopped that growth path.
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
        cause: Option<ExplainRecordId>,
    ) -> Result<RegionFormationRecords, ExplainError> {
        let mut chain = cause;
        for stop in &self.budget_stops {
            let subject = explain.subject(SubjectKind::Region, REGION_FORMATION_SUBJECT)?;
            chain = explain
                .push_detail(
                    RuleRef::builtin(REGION_FORMATION_RULE)?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::RegionFormation,
                        resource: ResourceKey::new(stop.resource.key())?,
                        limit: stop.limit,
                        actual: stop.actual,
                    },
                    chain.into_iter().collect(),
                )?
                .or(chain);
        }
        let mut whole_program = None;
        for candidate in self.candidates() {
            let record = record_candidate(explain, candidate, chain)?;
            if candidate.covers_whole_program() {
                whole_program = record;
            }
            chain = record.or(chain);
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
        cause: Option<ExplainRecordId>,
    ) -> Result<Option<ExplainRecordId>, ExplainError> {
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
            cause.into_iter().collect(),
        )
    }
}

/// Emits one admitted candidate through the typed explain authority.
fn record_candidate(
    explain: &mut ExplainWriter,
    candidate: &RegionCandidate,
    cause: Option<ExplainRecordId>,
) -> Result<Option<ExplainRecordId>, ExplainError> {
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
                FactValue::Identity(SubjectKey::new(candidate.content.key())?),
            )?)?;
    let subject = explain.subject(SubjectKind::Candidate, &candidate.stable_id)?;
    explain.push_detail(
        RuleRef::builtin(REGION_CANDIDATE_RULE)?,
        vec![subject],
        ExplainEvent::Check {
            stage: ExplainStage::RegionFormation,
            assessment,
            rejection: RejectionClass::IntrinsicInvalid,
        },
        cause.into_iter().collect(),
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
}

impl RegionGraph {
    /// Derives the dataflow view of one verified program.
    pub(crate) fn from_program(program: &SemanticProgram) -> Result<Self, RegionError> {
        let ordinals: BTreeMap<ValueId, u32> = program
            .values()
            .enumerate()
            .map(|(ordinal, value)| Ok((value.id(), index(ordinal)?)))
            .collect::<Result<_, RegionError>>()?;
        let mut values: Vec<GraphValue> = program
            .values()
            .map(|value| GraphValue {
                type_encoding: value
                    .resolved_type()
                    .canonical_encoding()
                    .as_bytes()
                    .to_vec()
                    .into_boxed_slice(),
                shape: value.shape().clone(),
                producer: None,
                input_position: None,
                consumers: Vec::new(),
                named_result: false,
            })
            .collect();
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
        };
        let whole: BTreeSet<u32> = (0..graph.operation_count()).collect();
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
        Ok(graph)
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
    /// position; a program input is named by its ordered interface position.
    fn canonical_value(&self, value: u32) -> Result<(u8, u32, u32), RegionError> {
        let value = self.value(value)?;
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

    /// Returns operations adjacent to `members` through one value edge.
    fn neighbours(&self, members: &BTreeSet<u32>) -> Result<BTreeSet<u32>, RegionError> {
        let mut adjacent = BTreeSet::new();
        for member in members {
            let operation = self.operation(*member)?;
            for operand in &operation.operands {
                if let Some(producer) = self.value(*operand)?.producer
                    && !members.contains(&producer.operation)
                {
                    adjacent.insert(producer.operation);
                }
            }
            for result in &operation.results {
                for consumer in &self.value(*result)?.consumers {
                    if !members.contains(consumer) {
                        adjacent.insert(*consumer);
                    }
                }
            }
        }
        Ok(adjacent)
    }

    /// Returns whether `members` is connected through producer/consumer edges.
    fn is_connected(&self, members: &BTreeSet<u32>) -> Result<bool, RegionError> {
        let Some(start) = members.first().copied() else {
            return Ok(false);
        };
        let mut reached = BTreeSet::new();
        let mut queue = VecDeque::from([start]);
        while let Some(member) = queue.pop_front() {
            if !reached.insert(member) {
                continue;
            }
            let operation = self.operation(member)?;
            for operand in &operation.operands {
                if let Some(producer) = self.value(*operand)?.producer
                    && members.contains(&producer.operation)
                {
                    queue.push_back(producer.operation);
                }
            }
            for result in &operation.results {
                for consumer in &self.value(*result)?.consumers {
                    if members.contains(consumer) {
                        queue.push_back(*consumer);
                    }
                }
            }
        }
        Ok(reached.len() == members.len())
    }

    /// Returns whether no directed path leaves `members` and re-enters it.
    ///
    /// The forward closure of the region through non-members is computed once;
    /// the region is non-convex exactly when that closure reaches a member.
    fn is_convex(&self, members: &BTreeSet<u32>) -> Result<bool, RegionError> {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        for member in members {
            for result in &self.operation(*member)?.results {
                for consumer in &self.value(*result)?.consumers {
                    if !members.contains(consumer) && visited.insert(*consumer) {
                        queue.push_back(*consumer);
                    }
                }
            }
        }
        while let Some(outside) = queue.pop_front() {
            for result in &self.operation(outside)?.results {
                for consumer in &self.value(*result)?.consumers {
                    if members.contains(consumer) {
                        return Ok(false);
                    }
                    if visited.insert(*consumer) {
                        queue.push_back(*consumer);
                    }
                }
            }
        }
        Ok(true)
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
pub(crate) fn form_region_candidates(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
) -> Result<RegionFormationOutcome, RegionError> {
    let graph = RegionGraph::from_program(program)?;
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

/// Recomputes one candidate from its exact member set and compares it.
///
/// A stored candidate is never trusted structurally: identity, boundaries,
/// retained outputs, and duplication policy are all rederived from the graph.
pub(crate) fn verify_candidate(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    candidate: &RegionCandidate,
) -> Result<(), RegionError> {
    let members: Vec<u32> = candidate.members.iter().map(|member| member.0).collect();
    if members.is_empty() || members.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(RegionError::Invalid {
            region: candidate.stable_id.clone(),
            rule: "membership",
        });
    }
    let rebuilt = form_candidate(graph, budgets, numerical_contract, &members)?;
    match rebuilt {
        Err(rejection) => Err(RegionError::Invalid {
            region: candidate.stable_id.clone(),
            rule: rejection.rule(),
        }),
        Ok(rebuilt) if rebuilt == *candidate => Ok(()),
        Ok(_) => Err(RegionError::Invalid {
            region: candidate.stable_id.clone(),
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
        for member in 0..self.graph.operation_count() {
            match form_candidate(self.graph, self.budgets, self.numerical_contract, &[member])? {
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

    /// Grows multi-member regions from every seed in stable topological order.
    fn grow(&mut self) -> Result<(), RegionError> {
        for seed in 0..self.graph.operation_count() {
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
        let mut visited = BTreeSet::from([BTreeSet::from([seed])]);
        let mut queue = VecDeque::from([BTreeSet::from([seed])]);
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
                next.insert(neighbour);
                if !visited.insert(next.clone()) {
                    continue;
                }
                let members: Vec<u32> = next.iter().copied().collect();
                queue.push_back(next);
                match form_candidate(self.graph, self.budgets, self.numerical_contract, &members)? {
                    Ok(candidate) => {
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
            let key =
                canonical_positions(self.graph, candidate.members.iter().map(|member| member.0))?;
            keyed.push((key, candidate));
        }
        keyed.sort_by(|left, right| (&left.0, &left.1.members).cmp(&(&right.0, &right.1.members)));
        let candidates: Vec<RegionCandidate> =
            keyed.into_iter().map(|(_, candidate)| candidate).collect();
        let labels: BTreeSet<&str> = candidates
            .iter()
            .map(|candidate| candidate.stable_id.as_str())
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

/// Classifies one member set and assembles its candidate when it is legal.
fn form_candidate(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
    members: &[u32],
) -> Result<Result<RegionCandidate, RegionRejection>, RegionError> {
    let membership: BTreeSet<u32> = members.iter().copied().collect();
    if membership.len() != members.len() || membership.is_empty() {
        return Err(RegionError::Structure {
            rule: "member-multiset",
        });
    }
    for member in &membership {
        graph.operation(*member)?;
    }
    if let Some(rejection) = classify(graph, budgets, &membership)? {
        return Ok(Err(rejection));
    }
    let shape = region_shape(graph, &membership)?;
    // Singleton coverage is unconditional, so a boundary or live-value budget
    // bounds fused growth without ever removing the unfused plan.
    if membership.len() > 1
        && let Some(rejection) = classify_shape(budgets, &shape)
    {
        return Ok(Err(rejection));
    }
    assemble(graph, numerical_contract, &membership, shape).map(Ok)
}

/// Decides the structural legality rules that do not need boundary derivation.
fn classify(
    graph: &RegionGraph,
    budgets: DeterministicBudgets,
    members: &BTreeSet<u32>,
) -> Result<Option<RegionRejection>, RegionError> {
    let member_limit = u64::from(budgets.region_members);
    let member_count = count(members.len());
    // A singleton is the operation alone, so its multiplicity and evaluation
    // order are unchanged and no member budget or purity rule can remove it.
    if member_count > 1 {
        if member_count > member_limit {
            return Ok(Some(RegionRejection::Budget(RegionBudgetStop {
                resource: RegionBudgetResource::Members,
                limit: member_limit,
                actual: member_count,
            })));
        }
        for member in members {
            if !graph.operation(*member)?.pure {
                return Ok(Some(RegionRejection::ImpureMember));
            }
        }
        if !graph.is_connected(members)? {
            return Ok(Some(RegionRejection::Disconnected));
        }
    }
    if !graph.is_convex(members)? {
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

/// Derives boundary inputs, retained outputs, and live values for one set.
fn region_shape(graph: &RegionGraph, members: &BTreeSet<u32>) -> Result<RegionShape, RegionError> {
    let mut boundary_inputs = Vec::new();
    let mut retained_outputs = Vec::new();
    let mut member_results = 0_u64;
    for member in members {
        let operation = graph.operation(*member)?;
        for operand in &operation.operands {
            let produced_inside = graph
                .value(*operand)?
                .producer
                .is_some_and(|producer| members.contains(&producer.operation));
            if !produced_inside && !boundary_inputs.contains(operand) {
                boundary_inputs.push(*operand);
            }
        }
        member_results = member_results.saturating_add(count(operation.results.len()));
        for (result_position, result) in operation.results.iter().enumerate() {
            let value = graph.value(*result)?;
            let external_consumers = value
                .consumers
                .iter()
                .any(|consumer| !members.contains(consumer));
            if value.named_result || external_consumers {
                retained_outputs.push(RetainedOutput {
                    value: SemanticValueId(*result),
                    producer: SemanticMemberId(*member),
                    result_position: index(result_position)?,
                    named_result: value.named_result,
                    external_consumers,
                });
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

/// Builds the identity-bearing candidate for one legal member set.
fn assemble(
    graph: &RegionGraph,
    numerical_contract: StrictF32NumericalContract,
    members: &BTreeSet<u32>,
    shape: RegionShape,
) -> Result<RegionCandidate, RegionError> {
    let duplication = DuplicationPolicy::Disabled;
    let content = encode_content(graph, numerical_contract, members, &shape, duplication)?;
    let occurrence = encode_occurrence(graph, &content, members, &shape)?;
    let stable_id = occurrence.key();
    Ok(RegionCandidate {
        members: members
            .iter()
            .map(|member| SemanticMemberId(*member))
            .collect(),
        boundary_inputs: shape
            .boundary_inputs
            .iter()
            .map(|value| SemanticValueId(*value))
            .collect(),
        retained_outputs: shape.retained_outputs,
        duplication,
        content,
        occurrence,
        stable_id,
        program_operation_count: graph.operation_count(),
    })
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
    members: &BTreeSet<u32>,
    shape: &RegionShape,
    duplication: DuplicationPolicy,
) -> Result<RegionContentIdentity, RegionError> {
    let canonical = canonical_member_order(graph, members)?;
    let local: BTreeMap<u32, u32> = canonical
        .iter()
        .enumerate()
        .map(|(position, member)| Ok((*member, index(position)?)))
        .collect::<Result<_, RegionError>>()?;
    let mut boundary_order = Vec::with_capacity(shape.boundary_inputs.len());
    for member in &canonical {
        for operand in &graph.operation(*member)?.operands {
            if !boundary_is_internal(graph, members, *operand)? && !boundary_order.contains(operand)
            {
                boundary_order.push(*operand);
            }
        }
    }
    let boundary_local: BTreeMap<u32, u32> = boundary_order
        .iter()
        .enumerate()
        .map(|(position, value)| Ok((*value, index(position)?)))
        .collect::<Result<_, RegionError>>()?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.compiler.region-content.v1\0");
    encode_bytes(&mut bytes, numerical_contract.key.as_bytes());
    bytes.push(duplication.tag());
    encode_count(&mut bytes, canonical.len());
    for member in &canonical {
        let operation = graph.operation(*member)?;
        encode_operation_facts(&mut bytes, operation)?;
        encode_count(&mut bytes, operation.operands.len());
        for operand in &operation.operands {
            if let Some(producer) = internal_producer(graph, members, *operand)? {
                let position = local
                    .get(&producer.operation)
                    .ok_or(RegionError::Structure {
                        rule: "content-local-member",
                    })?;
                bytes.push(1);
                bytes.extend_from_slice(&position.to_be_bytes());
                bytes.extend_from_slice(&producer.result_position.to_be_bytes());
            } else {
                let position = boundary_local.get(operand).ok_or(RegionError::Structure {
                    rule: "content-local-boundary",
                })?;
                bytes.push(2);
                bytes.extend_from_slice(&position.to_be_bytes());
            }
        }
        encode_count(&mut bytes, operation.results.len());
        for result in &operation.results {
            encode_value_facts(&mut bytes, graph.value(*result)?);
        }
    }
    encode_count(&mut bytes, boundary_order.len());
    for value in &boundary_order {
        encode_value_facts(&mut bytes, graph.value(*value)?);
    }
    let mut retained: Vec<(u32, u32, bool, bool)> = shape
        .retained_outputs
        .iter()
        .map(|output| {
            let position =
                local
                    .get(&output.producer.0)
                    .copied()
                    .ok_or(RegionError::Structure {
                        rule: "content-local-output",
                    })?;
            Ok((
                position,
                output.result_position,
                output.named_result,
                output.external_consumers,
            ))
        })
        .collect::<Result<_, RegionError>>()?;
    retained.sort_unstable();
    encode_count(&mut bytes, retained.len());
    for (position, result_position, named_result, external_consumers) in retained {
        bytes.extend_from_slice(&position.to_be_bytes());
        bytes.extend_from_slice(&result_position.to_be_bytes());
        bytes.push(u8::from(named_result));
        bytes.push(u8::from(external_consumers));
    }
    Ok(RegionContentIdentity {
        canonical: bytes.into_boxed_slice(),
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
fn canonical_member_order(
    graph: &RegionGraph,
    members: &BTreeSet<u32>,
) -> Result<Vec<u32>, RegionError> {
    let ordinals: Vec<u32> = members.iter().copied().collect();
    let positions: BTreeMap<u32, usize> = ordinals
        .iter()
        .enumerate()
        .map(|(position, member)| (*member, position))
        .collect();
    let mut base = Vec::with_capacity(ordinals.len());
    for member in &ordinals {
        let operation = graph.operation(*member)?;
        let mut bytes = Vec::new();
        encode_operation_facts(&mut bytes, operation)?;
        encode_count(&mut bytes, operation.results.len());
        for result in &operation.results {
            encode_value_facts(&mut bytes, graph.value(*result)?);
        }
        base.push(bytes);
    }
    let mut labels: Vec<u64> = base.iter().map(|bytes| digest(bytes)).collect();
    for _ in 0..ordinals.len() {
        let mut refined = Vec::with_capacity(ordinals.len());
        for (position, member) in ordinals.iter().enumerate() {
            let mut bytes = base[position].clone();
            bytes.extend_from_slice(&labels[position].to_be_bytes());
            let operation = graph.operation(*member)?;
            encode_count(&mut bytes, operation.operands.len());
            for operand in &operation.operands {
                if let Some(producer) = internal_producer(graph, members, *operand)? {
                    let source =
                        positions
                            .get(&producer.operation)
                            .ok_or(RegionError::Structure {
                                rule: "canonical-order-member",
                            })?;
                    bytes.push(1);
                    bytes.extend_from_slice(&labels[*source].to_be_bytes());
                    bytes.extend_from_slice(&producer.result_position.to_be_bytes());
                } else {
                    bytes.push(2);
                    encode_value_facts(&mut bytes, graph.value(*operand)?);
                }
            }
            refined.push(digest(&bytes));
        }
        if refined == labels {
            break;
        }
        labels = refined;
    }
    let mut order: Vec<usize> = (0..ordinals.len()).collect();
    order.sort_by(|left, right| {
        (labels[*left], &base[*left], ordinals[*left]).cmp(&(
            labels[*right],
            &base[*right],
            ordinals[*right],
        ))
    });
    Ok(order
        .into_iter()
        .map(|position| ordinals[position])
        .collect())
}

fn encode_operation_facts(
    bytes: &mut Vec<u8>,
    operation: &GraphOperation,
) -> Result<(), RegionError> {
    encode_bytes(bytes, operation.key.namespace().as_bytes());
    encode_bytes(bytes, operation.key.name().as_bytes());
    bytes.extend_from_slice(&operation.key.semantic_version().to_be_bytes());
    encode_attributes(bytes, &operation.attributes)
}

fn internal_producer(
    graph: &RegionGraph,
    members: &BTreeSet<u32>,
    value: u32,
) -> Result<Option<ValueProducer>, RegionError> {
    Ok(graph
        .value(value)?
        .producer
        .filter(|producer| members.contains(&producer.operation)))
}

fn boundary_is_internal(
    graph: &RegionGraph,
    members: &BTreeSet<u32>,
    value: u32,
) -> Result<bool, RegionError> {
    Ok(internal_producer(graph, members, value)?.is_some())
}

/// Encodes the exact graph site of one region in canonical coordinates.
///
/// The member set determines the site, so encoding its canonical positions is
/// injective for one program. Boundary and retained values are derived from that
/// set and are encoded as redundant, independently checkable site facts.
fn encode_occurrence(
    graph: &RegionGraph,
    content: &RegionContentIdentity,
    members: &BTreeSet<u32>,
    shape: &RegionShape,
) -> Result<RegionOccurrenceIdentity, RegionError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.compiler.region-occurrence.v1\0");
    encode_bytes(&mut bytes, content.as_bytes());
    encode_count(&mut bytes, members.len());
    for position in canonical_positions(graph, members.iter().copied())? {
        bytes.extend_from_slice(&position.to_be_bytes());
    }
    for values in [
        &shape.boundary_inputs,
        &shape
            .retained_outputs
            .iter()
            .map(|output| output.value.0)
            .collect::<Vec<_>>(),
    ] {
        encode_count(&mut bytes, values.len());
        let mut sites = values
            .iter()
            .map(|value| graph.canonical_value(*value))
            .collect::<Result<Vec<_>, _>>()?;
        sites.sort_unstable();
        for (tag, first, second) in sites {
            bytes.push(tag);
            bytes.extend_from_slice(&first.to_be_bytes());
            bytes.extend_from_slice(&second.to_be_bytes());
        }
    }
    Ok(RegionOccurrenceIdentity {
        canonical: bytes.into_boxed_slice(),
    })
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
    encode_bytes(bytes, &value.type_encoding);
    encode_count(bytes, value.shape.rank());
    for extent in value.shape.extents() {
        bytes.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn encode_attributes(
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
    encode_count(bytes, fields.len());
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
            encode_bytes(bytes, resolved.canonical_encoding().as_bytes());
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
            encode_bytes(bytes, float.format().namespace().as_bytes());
            encode_bytes(bytes, float.format().name().as_bytes());
            bytes.extend_from_slice(&float.format().semantic_version().to_be_bytes());
            encode_bytes(bytes, float.bits());
        }
        CanonicalValueView::Bytes(value) => {
            bytes.push(6);
            encode_bytes(bytes, value);
        }
        CanonicalValueView::Utf8(value) => {
            bytes.push(7);
            encode_bytes(bytes, value.as_bytes());
        }
        CanonicalValueView::Sequence(values) => {
            bytes.push(8);
            encode_count(bytes, values.len());
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

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_count(output, value.len());
    output.extend_from_slice(value);
}

fn encode_count(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&count(value).to_be_bytes());
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
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explain::ExplainLimits;
    use crate::request::{CompilationRequest, verify_request};
    use std::collections::BTreeMap as OracleMap;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

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

    fn member_sets(outcome: &RegionFormationOutcome) -> Vec<Vec<u32>> {
        outcome
            .candidates()
            .iter()
            .map(|candidate| candidate.members().iter().map(|member| member.0).collect())
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
        assert!(!graph.is_convex(&BTreeSet::from([1, 2, 4])).unwrap());
        assert!(graph.is_convex(&BTreeSet::from([1, 2, 3, 4])).unwrap());
        assert!(!graph.is_connected(&BTreeSet::from([0, 4])).unwrap());
    }

    #[test]
    fn shared_producers_retain_ordered_multi_result_boundary_outputs() {
        let program = shared_producer_program();
        let outcome = form(&program);
        let whole = outcome
            .candidates()
            .iter()
            .find(|candidate| candidate.members().iter().map(|m| m.0).eq([1, 2, 3]))
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
            .find(|candidate| candidate.members().iter().map(|m| m.0).eq([1, 2]))
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
                .find(|candidate| candidate.members().iter().map(|m| m.0).eq(members.clone()))
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
            .find(|candidate| candidate.members().iter().map(|m| m.0).eq([2]))
            .unwrap();
        let right = outcome
            .candidates()
            .iter()
            .find(|candidate| candidate.members().iter().map(|m| m.0).eq([3]))
            .unwrap();
        let shared = outcome
            .candidates()
            .iter()
            .find(|candidate| candidate.members().iter().map(|m| m.0).eq([1]))
            .unwrap();

        // `left` multiplies its two boundary reads; `shared` does too, at a
        // different graph site with different retained-output reasons.
        assert_ne!(left.content(), right.content());
        assert_ne!(left.occurrence(), shared.occurrence());
        assert_ne!(left.stable_id(), shared.stable_id());

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
                .find(|candidate| candidate.members().iter().map(|m| m.0).eq([member]))
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

    #[test]
    fn budget_stops_report_bounded_search_loss_and_keep_singleton_coverage() {
        let program = serial_sum_program();
        let complete = oracle_legal_sets(&program);

        let mut budgets = DeterministicBudgets::governed();
        budgets.region_candidates_per_seed = 0;
        let outcome = form_with(&program, budgets);
        let emitted: BTreeSet<Vec<u32>> = member_sets(&outcome).into_iter().collect();
        assert_eq!(emitted.len(), program.operation_count());
        for member in 0..u32::try_from(program.operation_count()).unwrap() {
            assert!(emitted.contains(&vec![member]));
        }
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
        for member in 0..u32::try_from(program.operation_count()).unwrap() {
            assert!(member_sets(&outcome).contains(&vec![member]));
        }

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

    #[test]
    fn enumeration_is_deterministic_and_independent_of_authoring_order() {
        let first = form(&serial_sum_program());
        let second = form(&serial_sum_program());
        assert_eq!(member_sets(&first), member_sets(&second));
        assert_eq!(
            first
                .candidates()
                .iter()
                .map(RegionCandidate::stable_id)
                .collect::<Vec<_>>(),
            second
                .candidates()
                .iter()
                .map(RegionCandidate::stable_id)
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
        forged.stable_id.push_str("-forged");
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

        let diamond = diamond_program();
        let diamond_outcome = form(&diamond);
        let mut nonconvex = diamond_outcome.candidates()[0].clone();
        nonconvex.members = vec![
            SemanticMemberId(1),
            SemanticMemberId(2),
            SemanticMemberId(4),
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
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target, ExplainLimits::default()).unwrap();

        let records = outcome.record(&mut explain, None).unwrap();
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
            .find(|record| record.id() == records.summary.unwrap())
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
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target, ExplainLimits::default()).unwrap();

        outcome.record(&mut explain, None).unwrap();
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
}
