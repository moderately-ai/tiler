//! The general DAG partition search: legal complete covers of a semantic graph.
//!
//! Region formation proposes connected convex region candidates; this module
//! answers a distinct, strictly *global* question about them: which bounded sets
//! of region occurrences legally cover the whole semantic graph, before any
//! physical implementation is chosen. A cover is enumerated, not selected: this
//! stage chooses no implementation, schedules nothing, and does not claim a
//! complete executable program. It enumerates legal partitionings only.
//!
//! The design keeps the concerns the correctness contract insists on separating:
//!
//! - **Complete coverage, failing closed.** A legal cover assigns every
//!   operation to at least one region and retains every ordered named program
//!   output exactly once. A cover that leaves an operation or a named output
//!   uncovered, that covers an operation more than once without an admitted
//!   duplication policy, that duplicates an operation the legality condition
//!   refuses, or that produces one named output from two regions, is rejected
//!   with a typed [`CoverError`] rather than silently repaired.
//! - **Fan-out is materialized, never split into incomparable partitions.** A
//!   value produced in one region and read by others is one
//!   [`MaterializationEdge`] carrying every consuming region — materialized once
//!   and read across the boundary. A second region computing that value is
//!   *deliberate duplication* recorded in [`CoverDuplication`], never an implicit
//!   consequence of the partition's shape, and never a silent serialization of
//!   the consumers.
//! - **Materialization is a per-edge choice.** For each (produced value,
//!   consuming region) pair the search enumerates both answers: read the value
//!   across the boundary, or recompute its producer inside the consumer. A cover
//!   records the outcome, and [`CoverCost`] is what lets a deliberate
//!   materialization beat a recomputation — a partial duplication pays the
//!   recomputed elements *and* still materializes for the consumers it did not
//!   absorb, so the materializing cover dominates it.
//! - **Hard legality is separate from estimated cost.** A refused candidate
//!   carries a typed [`CoverRefusal`] naming why it is not a legal cover; it never
//!   carries a cost. [`CoverCost`] ranks only covers that are already legal, and
//!   [`CoverEnumeration::non_dominated`] is a pure view that prunes nothing from
//!   the retained set.
//! - **Both the fused and the fully-materialized cover are retained.** The
//!   fully-materialized (all-singleton) cover is emitted unconditionally, and the
//!   fused (whole-program) cover is emitted whenever region formation admitted a
//!   whole-program candidate. Neither can be lost to a budget; the budgets bound
//!   only the additional partitions the search discovers.
//! - **Budgeted and memoized, and an exhausted budget is reported.** The search
//!   memoizes the coverage completions of a covered-set state, which is sound
//!   because admissibility of any candidate depends on that state alone. When a
//!   budget stops the search, [`CoverEnumeration::is_exhaustive`] answers `false`
//!   and the retained covers are an explainable *partial* result — the best found
//!   plus the statement that the space was not exhausted — never a truncated set
//!   presented as complete.
//! - **Deterministic, order-independent identity.** A [`RegionCoverIdentity`]
//!   folds the semantic graph meaning, the exact region occurrences (which bind
//!   both region content and per-region coverage), the deliberate duplication,
//!   and the proposed materialization edges, in a canonical length-prefixed byte
//!   encoding over content-derived coordinates. It excludes transient graph-local
//!   ordinals and never depends on `HashMap`/authoring order.
//!
//! Scope boundary: this authority is a *global* legality enumerator over region
//! candidates. Local physical frontiers ([`crate::frontier`]) are enumerated
//! independently and do not depend on a global cover; joining a complete cover
//! with compatible per-region frontiers is the later complete
//! physical-plan-selection authority. Every item here is a reviewed *draft*
//! boundary, not a stable compiler API, until Tom accepts the exact interface.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::semantic::{SemanticProgram, ValueId};

use crate::region::{
    RegionCandidate, RegionContentIdentity, RegionError, RegionFormationOutcome, RegionGraph,
    RegionOccurrenceIdentity, SemanticMemberId, SemanticStage, SemanticValueId,
};
use crate::request::{BudgetResource, DeterministicBudgets, StrictF32NumericalContract};

/// Canonical domain-separation tag for one region-cover identity.
const COVER_IDENTITY_TAG: &[u8] = b"tiler.compiler.region-cover.v1\0";
/// The governed key naming the partition search's own cost model.
///
/// Deliberately distinct from `tiler.cost.structural.v1`: this model ranks
/// *covers* on facts a cover determines, before any implementation is chosen,
/// and nothing attributed to it may enter a plan-level dominance comparison.
const COVER_COST_MODEL_KEY: &str = "tiler.cost.partition-structural.v1";

/// A deterministic budget that bounds cover enumeration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CoverBudgetResource {
    /// Distinct covers retained for one enumeration request.
    Covers,
    /// Partition-search expansion attempts for one enumeration request.
    Expansions,
    /// Typed refusals retained for one enumeration request.
    ///
    /// The refusal list is what makes a pruned candidate nameable, and it is
    /// bounded for the same reason every other search product is: the refused
    /// space is exponential. A stop here means the *explanation* was truncated,
    /// not the search, and the two are separate resources so a reader can tell
    /// which one ran out.
    Refusals,
}

impl CoverBudgetResource {
    /// Returns the resource a stop here refuses a compilation on, if any.
    ///
    /// `None` for [`Self::Refusals`] alone, and that is the typed form of the
    /// exclusion this budget's own documentation states: a search that explored
    /// the whole space while declining to name every candidate it refused found
    /// everything there was to find, so it truncates no plan and can refuse no
    /// compilation. Returning an `Option` rather than testing the variant at the
    /// consuming site is what keeps the exclusion decidable here — a cover
    /// budget added above must say which side of it it falls on.
    pub(crate) const fn truncating_resource(self) -> Option<BudgetResource> {
        match self {
            Self::Covers => Some(BudgetResource::RegionCovers),
            Self::Expansions => Some(BudgetResource::RegionCoverExpansions),
            Self::Refusals => None,
        }
    }

    /// Returns the stable resource key.
    pub(crate) const fn key(self) -> &'static str {
        match self.truncating_resource() {
            Some(resource) => resource.key(),
            // The explanation budget never reaches a refusal, so it holds no
            // row in the shared vocabulary and keeps its key here — it still
            // names itself in the explain record its stop writes.
            None => "region-cover-refusals",
        }
    }
}

/// One declared cover budget and the demand that it refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoverBudgetStop {
    /// The budget that fired.
    pub(crate) resource: CoverBudgetResource,
    /// The declared limit.
    pub(crate) limit: u64,
    /// The refused demand observed at the stop point.
    ///
    /// This is a lower bound on the unexplored space rather than its size: the
    /// search stops at the first demand the limit refuses.
    pub(crate) actual: u64,
}

/// Why the bounded profile can enumerate no legal complete cover for a program.
///
/// This is a legitimate, honestly reported result — distinct from a
/// [`CoverError`] compiler fault — carried on an otherwise valid
/// [`CoverEnumeration`] whose cover set is empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CoverInfeasibility {
    /// A named program output is exported directly from a boundary input, so no
    /// operation region can cover it. The count is the number of such outputs.
    UnrootedNamedOutput {
        /// The number of named outputs exported directly from a boundary input.
        count: u64,
    },
}

impl CoverInfeasibility {
    /// Returns the stable reason code of the infeasibility.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::UnrootedNamedOutput { .. } => "unrooted-named-output",
        }
    }
}

impl fmt::Display for CoverInfeasibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrootedNamedOutput { count } => write!(
                formatter,
                "cover.infeasible.unrooted-named-output: {count} named output(s) export a boundary input"
            ),
        }
    }
}

/// Whether the partition search may admit deliberate shared-work duplication.
///
/// The two admissions are different *legality* contracts, not two costs: under
/// [`Self::Forbidden`] a legal cover is an exact partition and any second
/// coverage of an operation is [`CoverError::IllegalDuplication`], while under
/// [`Self::PureRecomputation`] a second coverage is legal exactly when
/// [`duplication_refusal`] admits the member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "the compile path states the exact-partition admission for the reason `CoverPolicy::governed` records; the recomputation admission is exercised by this authority's own tests until a physical provider and program assembly can realize a duplicating cover"
)]
pub(crate) enum CoverDuplicationAdmission {
    /// A legal cover is an exact partition.
    Forbidden,
    /// An operation may be computed in several regions when recomputing it
    /// provably preserves the program.
    PureRecomputation,
}

/// The legality contract one enumeration and every verification of its covers
/// run under.
///
/// It carries both halves of that contract — the duplication admission and the
/// resolved numerical contract the recomputation condition is decided against —
/// because they are one answer: whether an occurrence may be computed twice is
/// undecidable from either alone. Carried as a value rather than read from a
/// constant so a caller *states* it, and so [`verify_cover`] checks a cover
/// against the same contract that produced it: a cover legal under one admission
/// is not automatically legal under the other, and re-deriving the admission
/// instead of taking it would let a verification silently apply a weaker rule
/// than the enumeration did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoverPolicy {
    duplication: CoverDuplicationAdmission,
    contract: StrictF32NumericalContract,
}

#[allow(
    dead_code,
    reason = "see `CoverDuplicationAdmission`: the duplication-admitting constructor is exercised by this authority's own tests until the compile path can realize a duplicating cover"
)]
impl CoverPolicy {
    /// The exact-partition contract the compile path enumerates under.
    ///
    /// **Duplication is off on the compile path by derivation, not by
    /// timidity.** A duplicating cover assigns one semantic occurrence to
    /// several region subjects, and a plan over it needs a physical
    /// implementation for each of those subjects. The bounded physical profile
    /// now *answers* for every region a cover places, but the schedule
    /// vocabulary it answers from still spells only the three recognized region
    /// shapes: every region a duplicating cover introduces earns a
    /// `StrategyDeclineCause::UnspellableRegion` and no admitted
    /// implementation. Program assembly implements exactly three plan shapes on
    /// top of that. So every duplicating cover of a governed program would be
    /// enumerated, found unimplementable, and rejected — paying the whole
    /// search, and now the explanation too, to report a refusal. Enabling it is
    /// a region-vocabulary and program-assembly question, not a legality one;
    /// see `tickets/activate-shared-work-duplication-on-the-compile-path.md`.
    pub(crate) const fn governed(contract: StrictF32NumericalContract) -> Self {
        Self {
            duplication: CoverDuplicationAdmission::Forbidden,
            contract,
        }
    }

    /// The contract that admits legal shared-work duplication.
    pub(crate) const fn permitting_shared_work_duplication(
        contract: StrictF32NumericalContract,
    ) -> Self {
        Self {
            duplication: CoverDuplicationAdmission::PureRecomputation,
            contract,
        }
    }

    /// Returns whether this contract admits any duplication at all.
    pub(crate) const fn admits_duplication(self) -> bool {
        matches!(
            self.duplication,
            CoverDuplicationAdmission::PureRecomputation
        )
    }

    /// Returns the resolved numerical contract duplication is decided against.
    pub(crate) const fn numerical_contract(self) -> StrictF32NumericalContract {
        self.contract
    }

    /// Returns the stable key naming this legality contract.
    pub(crate) const fn key(self) -> &'static str {
        match self.duplication {
            CoverDuplicationAdmission::Forbidden => "cover.exact-partition.v1",
            CoverDuplicationAdmission::PureRecomputation => "cover.pure-recomputation.v1",
        }
    }
}

/// Why one operation may not be computed in more than one region.
///
/// Each variant is a *hard* legality answer about the operation and the
/// governing contract, decided before any cover is assembled and never a cost.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum DuplicationRefusal {
    /// The enumeration's legality contract admits no duplication at all.
    PolicyForbids,
    /// The frozen semantic authority did not prove the operation referentially
    /// transparent, so a second evaluation is not a second copy of one value.
    ImpureMember,
    /// The operation produces an ordered named program output. A named output is
    /// written once to a definite destination, so two regions producing it is
    /// two writers of one result rather than two copies of one value.
    NamedResultProducer,
    /// The resolved numerical contract lets two realizations of one occurrence
    /// differ, so the two copies are not provably the same value.
    ///
    /// Never a cost: no target and no cheaper plan makes recomputation
    /// value-preserving under a contract that authorized the divergence.
    ContractGrantsRealizationFreedom,
}

impl DuplicationRefusal {
    /// Returns the stable reason code of the refusal.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::PolicyForbids => "duplication-policy-forbids",
            Self::ImpureMember => "duplication-impure-member",
            Self::NamedResultProducer => "duplication-named-result-producer",
            Self::ContractGrantsRealizationFreedom => "duplication-contract-grants-freedom",
        }
    }
}

impl fmt::Display for DuplicationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cover.duplication.{}", self.reason())
    }
}

/// Why one candidate cover the search reached is not a legal complete cover.
///
/// A refusal names the exact subject it is about and a typed hard reason. It is
/// the pruning half of the search's explanation, and it is deliberately never a
/// cost: a reader distinguishes "this candidate is not legal" from "this legal
/// candidate is beaten" by which of these two channels reports it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CoverRefusal {
    /// A candidate region would have computed an operation another chosen region
    /// already computes, and duplicating that operation is refused.
    Duplication {
        /// Occurrence identity of the region that would have duplicated.
        region: RegionOccurrenceIdentity,
        /// Content-derived canonical position of the duplicated operation.
        position: u32,
        /// The typed legality answer.
        refusal: DuplicationRefusal,
    },
    /// A complete candidate cover placed a region nothing in the cover consumes
    /// and which produces no named program output.
    ///
    /// Only reachable under a duplication-admitting contract, where the search
    /// itself creates the dead region; an exact partition's dead region reflects
    /// a dead operation in the program, which is not this authority's to judge.
    DeadRegion {
        /// Occurrence identity of the region nothing observes.
        region: RegionOccurrenceIdentity,
    },
    /// A complete candidate cover produced one ordered named output from more
    /// than one region.
    AmbiguousNamedOutput {
        /// Ordered position of the named program output.
        output_position: u32,
    },
}

impl CoverRefusal {
    /// Returns the stable reason code of the refusal.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Duplication { refusal, .. } => refusal.reason(),
            Self::DeadRegion { .. } => "dead-region",
            Self::AmbiguousNamedOutput { .. } => "ambiguous-named-output",
        }
    }

    /// Returns a bounded explain label naming the refused subject.
    pub(crate) fn subject_label(&self) -> String {
        match self {
            Self::Duplication { region, .. } | Self::DeadRegion { region } => {
                crate::region::hex_label("region:", digest(region.as_bytes()))
            }
            Self::AmbiguousNamedOutput { output_position } => {
                format!("named-output:{output_position}")
            }
        }
    }
}

impl fmt::Display for CoverRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cover.refused.{}: {}",
            self.reason(),
            self.subject_label()
        )
    }
}

/// The deliberate producer duplication a cover realizes.
///
/// Empty under [`CoverPolicy::governed`], where a legal cover is an exact
/// partition. Under a duplication-admitting contract it holds the
/// content-derived canonical positions of the operations the cover computes in
/// more than one region, ascending and deduplicated, and cover identity binds
/// them: two covers over the same regions cannot differ in duplication, but a
/// cover's duplication is what a reader and a cost model both need named.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverDuplication {
    duplicated: Vec<u32>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl CoverDuplication {
    /// Builds the empty (no deliberate duplication) policy.
    const fn none() -> Self {
        Self {
            duplicated: Vec::new(),
        }
    }

    /// Returns the canonical positions of the deliberately duplicated operations.
    pub(crate) fn duplicated_positions(&self) -> &[u32] {
        &self.duplicated
    }

    /// Returns whether the cover realizes no deliberate duplication.
    pub(crate) fn is_none(&self) -> bool {
        self.duplicated.is_empty()
    }
}

/// One value materialized across a region boundary within a cover.
///
/// A value produced inside one region and read by a region that does not compute
/// it is materialized exactly once and read across the boundary. The edge is
/// expressed in content-derived canonical coordinates so it does not depend on
/// authoring order: the producing member's canonical position, the producing
/// region's occurrence identity, and the consuming regions' occurrence
/// identities. A value with several cross-region consumers is one edge with
/// several consumers — conservative fan-out materialization, not duplication and
/// not a serialization of the consumers.
///
/// When several regions compute the value — a deliberate duplication — exactly
/// one copy is the one a non-computing consumer reads, and it is the copy in the
/// owner with the **fewest members**, ties broken by the smallest occurrence
/// identity. Every copy is the same value by the duplication legality condition
/// and the edge's count and size are the same whichever is named, so the rule
/// only has to be deterministic and content-derived. Fewest-members is the one
/// that also keeps the *admitted set* content-derived: the smallest owner is the
/// one with the least other work to justify it, so designating it is what stops
/// an otherwise legal partial duplication from being refused as dead purely
/// because an identity digest happened to order two owners one way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaterializationEdge {
    /// Graph-local ordinal of the materialized value (navigation only; excluded
    /// from identity because it is a transient authoring coordinate).
    value: SemanticValueId,
    /// Content-derived canonical position of the producing member.
    producer_position: u32,
    /// Result position of the materialized value on its producing member.
    result_position: u32,
    /// Elements the materialized value holds.
    ///
    /// Carried on the edge because the cover is the authority that knows it: an
    /// intermediate exists because a cover chose to materialize between two
    /// regions, so its element count is a property of that cover and not of
    /// either region's subject.
    element_count: u64,
    /// Occurrence identity of the region that materializes the value.
    producer: RegionOccurrenceIdentity,
    /// Occurrence identities of the regions that read the value, canonical ascending.
    consumers: Vec<RegionOccurrenceIdentity>,
}

impl MaterializationEdge {
    /// Returns the graph-local ordinal of the materialized value.
    #[allow(
        dead_code,
        reason = "transient authoring coordinate deliberately excluded from identity; read only when mapping an edge back to its authoring value"
    )]
    pub(crate) const fn value(&self) -> SemanticValueId {
        self.value
    }

    /// Returns the canonical position of the producing member.
    pub(crate) const fn producer_position(&self) -> u32 {
        self.producer_position
    }

    /// Returns the result position of the value on its producing member.
    pub(crate) const fn result_position(&self) -> u32 {
        self.result_position
    }

    /// Returns how many elements the materialized value holds.
    pub(crate) const fn element_count(&self) -> u64 {
        self.element_count
    }

    /// Returns the occurrence identity of the materializing region.
    pub(crate) const fn producer(&self) -> &RegionOccurrenceIdentity {
        &self.producer
    }

    /// Returns the occurrence identities of the consuming regions.
    pub(crate) fn consumers(&self) -> &[RegionOccurrenceIdentity] {
        &self.consumers
    }
}

/// One region occurrence placed in a cover.
///
/// It retains the region's exact members (its coverage), its site-independent
/// content identity, its graph-occurrence identity, the ordered named program
/// outputs it retains, and the bounded occurrence label. The occurrence identity
/// is what a cover binds and re-derives, so a placed region cannot silently
/// drift from the candidate region formation admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverRegion {
    members: Vec<SemanticStage>,
    content: RegionContentIdentity,
    occurrence: RegionOccurrenceIdentity,
    /// Graph-local ordinals of the ordered named program outputs this region
    /// retains, ascending.
    ///
    /// **Which named result a region publishes, not merely that it publishes
    /// one.** Program assembly pairs the program's declared outputs with the
    /// regions that write them, and the rest of this record says only that a
    /// region exports *an* output — a region no materialization edge names as a
    /// producer. With one declared output that pairing is forced; with two it is
    /// a guess, and pairing by execution order is exactly the interchangeable-
    /// outputs interface the architectural contract forbids. This is the fact
    /// that makes it a derivation.
    ///
    /// Deliberately **not** folded into [`encode_cover_identity`]. The candidate's
    /// [`RegionOccurrenceIdentity`] already encodes its retained output sites and
    /// the [`RegionContentIdentity`] inside it already encodes their
    /// `named_result` flags, so this is a projection of bytes cover identity
    /// already folds — and [`verify_cover`] proves the projection against the
    /// authoritative candidate. Folding it again would move every cover identity
    /// for no information.
    named_results: Vec<SemanticValueId>,
    /// Shared with the candidate it was placed from: a program's covers place
    /// the same regions repeatedly, and the label is immutable once formed.
    label: Arc<str>,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl CoverRegion {
    /// Returns the region's attribution atoms in ascending graph-local order.
    pub(crate) fn members(&self) -> &[SemanticStage] {
        &self.members
    }

    /// Returns the site-independent region-content identity.
    pub(crate) const fn content(&self) -> &RegionContentIdentity {
        &self.content
    }

    /// Returns the graph-occurrence identity of the region.
    pub(crate) const fn occurrence(&self) -> &RegionOccurrenceIdentity {
        &self.occurrence
    }

    /// Returns the ordered named program outputs this region retains, ascending.
    pub(crate) fn named_results(&self) -> &[SemanticValueId] {
        &self.named_results
    }

    /// Returns the bounded explain label of the region occurrence.
    pub(crate) fn label(&self) -> &str {
        &self.label
    }
}

/// The estimated cost of one legal cover, over facts the cover determines.
///
/// **Estimate, never feasibility.** Every cover carrying one of these is already
/// legal; nothing here can admit or refuse a cover, and a refused candidate
/// carries a [`CoverRefusal`] rather than a large number. It is attributed to
/// [`COVER_COST_MODEL_KEY`] and is deliberately *not* a
/// [`crate::selection::PlanStructuralCost`]: it ranks partitionings before an
/// implementation exists, and mixing the two keys would let a pre-implementation
/// estimate prune a plan.
///
/// The four dimensions are what a cover — and only a cover — decides:
///
/// - `region_count`, the number of separately implementable regions;
/// - `materialization_count`, the cross-region values that must be written and
///   read back;
/// - `materialized_elements`, how large those values are;
/// - `recomputed_elements`, the elements a duplicated operation computes again.
///
/// The last is what makes the materialize-versus-recompute choice decidable
/// rather than merely enumerable. A cover that absorbs *every* consumer of a
/// value materializes nothing and recomputes instead — the two trade, and
/// neither dominates. A cover that absorbs only *some* consumers still
/// materializes for the rest, so it pays the same edge and the recomputation on
/// top, and the materializing cover strictly dominates it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoverCost {
    model_key: &'static str,
    region_count: u64,
    materialization_count: u64,
    materialized_elements: u64,
    recomputed_elements: u64,
}

impl CoverCost {
    /// Returns the cost-model key this estimate is attributed to.
    pub(crate) const fn model_key(&self) -> &'static str {
        self.model_key
    }

    /// Returns the number of separately implementable regions.
    pub(crate) const fn region_count(&self) -> u64 {
        self.region_count
    }

    /// Returns the number of cross-region materializations.
    pub(crate) const fn materialization_count(&self) -> u64 {
        self.materialization_count
    }

    /// Returns the elements those materializations hold in total.
    pub(crate) const fn materialized_elements(&self) -> u64 {
        self.materialized_elements
    }

    /// Returns the elements deliberate duplication computes a second time.
    pub(crate) const fn recomputed_elements(&self) -> u64 {
        self.recomputed_elements
    }

    /// Returns whether this cost strictly dominates `other`.
    ///
    /// The standard Pareto relation over the four dimensions: no dimension is
    /// worse and at least one is strictly better. Estimates attributed to
    /// different models are incomparable, so neither dominates the other.
    pub(crate) fn dominates(&self, other: &Self) -> bool {
        if self.model_key != other.model_key {
            return false;
        }
        let no_worse = self.region_count <= other.region_count
            && self.materialization_count <= other.materialization_count
            && self.materialized_elements <= other.materialized_elements
            && self.recomputed_elements <= other.recomputed_elements;
        let strictly_better = self.region_count < other.region_count
            || self.materialization_count < other.materialization_count
            || self.materialized_elements < other.materialized_elements
            || self.recomputed_elements < other.recomputed_elements;
        no_worse && strictly_better
    }
}

/// Collision-free, order-independent identity of one legal complete cover.
///
/// It folds the semantic graph meaning, the exact region occurrences (which bind
/// per-region content and coverage), the deliberate duplication, and the proposed
/// materialization edges, over content-derived canonical coordinates. Transient
/// graph-local ordinals and enumeration order are deliberately absent.
///
/// The bytes are shared behind an [`Arc`]: a cover identity embeds every placed
/// region's occurrence encoding, so it is the largest single value the
/// enumeration copies, and it is copied once per retained cover to key the
/// retention map. Sharing changes nothing observable — [`Self::as_bytes`] yields
/// the same bytes and the derived `Ord` still compares content.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionCoverIdentity(Arc<[u8]>);

impl RegionCoverIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns a bounded explain label for this cover.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn label(&self) -> String {
        crate::region::hex_label("region-cover:", digest(&self.0))
    }
}

/// One legal complete cover of the semantic region graph.
///
/// Every operation is covered, every ordered named output is produced by exactly
/// one region, and every operation covered more than once is a deliberate,
/// legality-checked duplication. The regions are stored in canonical occurrence
/// order and the materialization edges in canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionCover {
    regions: Vec<CoverRegion>,
    materializations: Vec<MaterializationEdge>,
    duplication: CoverDuplication,
    cost: CoverCost,
    identity: RegionCoverIdentity,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl RegionCover {
    /// Returns the placed regions in canonical occurrence order.
    pub(crate) fn regions(&self) -> &[CoverRegion] {
        &self.regions
    }

    /// Returns the proposed materialization edges in canonical order.
    pub(crate) fn materializations(&self) -> &[MaterializationEdge] {
        &self.materializations
    }

    /// Returns the deliberate duplication the cover realizes.
    pub(crate) const fn duplication(&self) -> &CoverDuplication {
        &self.duplication
    }

    /// Returns the cover's estimated cost (never a legality input).
    pub(crate) const fn cost(&self) -> CoverCost {
        self.cost
    }

    /// Returns the canonical, order-independent cover identity.
    pub(crate) const fn identity(&self) -> &RegionCoverIdentity {
        &self.identity
    }

    /// Returns the number of regions in the cover.
    pub(crate) fn region_count(&self) -> usize {
        self.regions.len()
    }
}

/// The deterministic result of enumerating legal complete covers once.
///
/// The covers are returned in canonical identity order. An empty cover set with
/// a non-empty infeasibility list is a legitimate result: it reports honestly
/// that the bounded profile can cover no partitioning of this program, distinct
/// from a [`CoverError`] compiler fault.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverEnumeration {
    policy: CoverPolicy,
    covers: Vec<RegionCover>,
    refusals: Vec<CoverRefusal>,
    budget_stops: Vec<CoverBudgetStop>,
    infeasibilities: Vec<CoverInfeasibility>,
    operation_count: u32,
    /// One per realization stage atom; equal to `operation_count` exactly when
    /// no member is staged.
    node_count: u32,
}

#[allow(
    dead_code,
    reason = "reviewed draft record accessor exercised by this authority's own tests; the compile path reads the subjects its own verification needs"
)]
impl CoverEnumeration {
    /// Returns the legality contract these covers were enumerated under.
    pub(crate) const fn policy(&self) -> CoverPolicy {
        self.policy
    }

    /// Returns every enumerated legal cover in canonical identity order.
    pub(crate) fn covers(&self) -> &[RegionCover] {
        &self.covers
    }

    /// Returns every typed refusal the search retained, canonical and deduplicated.
    pub(crate) fn refusals(&self) -> &[CoverRefusal] {
        &self.refusals
    }

    /// Returns every budget that stopped a search path.
    pub(crate) fn budget_stops(&self) -> &[CoverBudgetStop] {
        &self.budget_stops
    }

    /// Returns whether the search exhausted the legal space.
    ///
    /// `false` means the retained covers are a **partial** result: every one of
    /// them is legal and completely derived, and the space beyond the budget was
    /// not explored. A caller must not read a partial result as a complete
    /// enumeration, which is why this is a stated fact rather than something
    /// inferred from a cover count.
    ///
    /// A [`CoverBudgetResource::Refusals`] stop deliberately does **not** make
    /// this `false`. That budget bounds the *explanation* — how many distinct
    /// refusals are retained — and a search that explored the whole space while
    /// declining to name every candidate it refused is still exhaustive.
    /// Reporting it as partial would tell a caller that covers might be missing
    /// when none is, and the two stops are separate resources precisely so a
    /// reader can tell which one ran out.
    pub(crate) fn is_exhaustive(&self) -> bool {
        self.budget_stops
            .iter()
            .all(|stop| stop.resource == CoverBudgetResource::Refusals)
    }

    /// Returns why the profile could enumerate no cover, when it could not.
    pub(crate) fn infeasibilities(&self) -> &[CoverInfeasibility] {
        &self.infeasibilities
    }

    /// Returns whether at least one legal complete cover exists in this profile.
    pub(crate) fn is_coverable(&self) -> bool {
        self.infeasibilities.is_empty()
    }

    /// Returns the number of operations every cover must cover.
    pub(crate) const fn operation_count(&self) -> u32 {
        self.operation_count
    }

    /// Returns the covers no other retained cover's estimate dominates.
    ///
    /// A pure view. It prunes nothing from [`Self::covers`], runs strictly after
    /// legality, and can neither establish nor refute it.
    pub(crate) fn non_dominated(&self) -> Vec<&RegionCover> {
        self.covers
            .iter()
            .enumerate()
            .filter(|(index, candidate)| {
                !self
                    .covers
                    .iter()
                    .enumerate()
                    .any(|(other, cover)| *index != other && cover.cost.dominates(&candidate.cost))
            })
            .map(|(_, candidate)| candidate)
            .collect()
    }

    /// Returns the retained covers this view prunes, each with why.
    ///
    /// The companion of [`Self::non_dominated`], and the reason the two are
    /// separate methods: an explanation must name the pruned candidate and the
    /// cover that beat it, and a view that only returns the survivors leaves the
    /// reader to re-derive that pairing.
    pub(crate) fn dominated(&self) -> Vec<(&RegionCover, &RegionCover)> {
        self.covers
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                self.covers
                    .iter()
                    .enumerate()
                    .find(|(other, cover)| *other != index && cover.cost.dominates(&candidate.cost))
                    .map(|(_, dominator)| (candidate, dominator))
            })
            .collect()
    }

    /// Returns the fully-materialized (all-singleton) cover, always retained when
    /// the program is coverable.
    pub(crate) fn fully_materialized_cover(&self) -> Option<&RegionCover> {
        self.covers.iter().find(|cover| {
            u32::try_from(cover.regions.len()).is_ok_and(|count| count == self.node_count)
                && cover.regions.iter().all(|region| region.members.len() == 1)
        })
    }

    /// Returns the fused (single whole-program region) cover, when region
    /// formation admitted a whole-program candidate.
    pub(crate) fn fused_cover(&self) -> Option<&RegionCover> {
        self.covers.iter().find(|cover| {
            cover.regions.len() == 1
                && u32::try_from(cover.regions[0].members.len())
                    .is_ok_and(|count| count == self.node_count)
        })
    }
}

/// A cover that is not a legal complete cover, or invalid cover state.
///
/// The coverage variants classify why a proposed cover is not legal — an
/// uncovered operation, an operation duplicated where duplication is refused, a
/// named output produced by several regions, a region nothing observes, or an
/// unretained named output. The structural variants are compiler faults: a placed
/// region that does not re-derive from the program (a broken occurrence
/// identity), or a cover whose recomputed materialization edges, duplication,
/// cost, or identity do not match. [`Self::class`] distinguishes the two, so
/// malformed cover state is never confused with a legal enumeration result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CoverError {
    /// A placed region failed re-derivation from the program.
    Region(RegionError),
    /// An operation is covered by no region.
    UncoveredMember {
        /// The uncovered operation.
        member: SemanticMemberId,
    },
    /// An operation is covered more than once and duplicating it is refused.
    IllegalDuplication {
        /// The double-covered operation.
        member: SemanticMemberId,
        /// The typed legality answer that refused it.
        refusal: DuplicationRefusal,
    },
    /// A named program output is retained by no region.
    UncoveredNamedOutput {
        /// A stable reason code for the uncovered output.
        reason: &'static str,
    },
    /// A candidate cover placed a region nothing observes, or produced one named
    /// output from several regions.
    ///
    /// Distinct from [`Self::IllegalDuplication`] because neither is about a
    /// duplicated *operation*: the first is a region whose results reach nothing,
    /// and the second is one program result with two writers.
    Unobservable {
        /// The typed refusal.
        refusal: CoverRefusal,
    },
    /// The cover carried structurally invalid state.
    Structure {
        /// A stable rule code.
        rule: &'static str,
    },
}

impl CoverError {
    /// Returns the coarse class of the fault: `region`, `coverage`, or `structure`.
    pub(crate) const fn class(&self) -> &'static str {
        match self {
            Self::Region(_) => "region",
            Self::UncoveredMember { .. }
            | Self::IllegalDuplication { .. }
            | Self::UncoveredNamedOutput { .. }
            | Self::Unobservable { .. } => "coverage",
            Self::Structure { .. } => "structure",
        }
    }

    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Region(error) => error.reason(),
            Self::UncoveredMember { .. } => "uncovered-member",
            Self::IllegalDuplication { .. } => "illegal-duplication",
            Self::Unobservable { refusal } => refusal.reason(),
            Self::UncoveredNamedOutput { reason } | Self::Structure { rule: reason } => reason,
        }
    }
}

impl fmt::Display for CoverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Region(error) => write!(formatter, "cover.region: {error}"),
            Self::UncoveredMember { member } => write!(
                formatter,
                "cover.coverage.uncovered-member: operation {} is covered by no region",
                member.0
            ),
            Self::IllegalDuplication { member, refusal } => write!(
                formatter,
                "cover.coverage.illegal-duplication: operation {} is covered more than once ({refusal})",
                member.0
            ),
            Self::UncoveredNamedOutput { reason } => {
                write!(formatter, "cover.coverage.uncovered-named-output.{reason}")
            }
            Self::Unobservable { refusal } => write!(formatter, "cover.coverage.{refusal}"),
            Self::Structure { rule } => write!(formatter, "cover.structure.{rule}"),
        }
    }
}

impl Error for CoverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Region(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RegionError> for CoverError {
    fn from(value: RegionError) -> Self {
        Self::Region(value)
    }
}

/// Enumerates the bounded legal complete covers of one verified program.
///
/// The candidates are re-derived from the program by region formation, so a
/// stale or forged candidate cannot enter a cover. The fully-materialized
/// (all-singleton) cover is emitted unconditionally and the fused (whole-program)
/// cover whenever a whole-program candidate exists; the remaining covers are
/// enumerated by anchoring each region on the minimum uncovered operation,
/// bounded by the request's `region_covers` and `region_cover_expansions`
/// budgets. Under a duplication-admitting `policy` an anchored region may also
/// re-cover operations the branch already covers, provided
/// [`duplication_refusal`] admits each of them. A program whose named output is a
/// bare boundary-input passthrough has no legal cover and yields an `Ok` result
/// with an empty cover set and a recorded [`CoverInfeasibility`].
///
/// # Errors
///
/// Returns a [`CoverError`] when region formation fails or the enumeration
/// observes invalid compiler state (a missing singleton or a candidate-index
/// fault). A program with no legal cover is a successful `Ok`, not an error.
/// The region formation is **taken rather than derived**, because it is a pure
/// function of the program, budgets, and contract this enumeration already runs
/// under, and every caller holds one. Deriving it here re-ran a search bounded
/// by `region_expansions` — ten thousand candidate formations — to reproduce a
/// value sitting in the caller's own frame.
pub(crate) fn enumerate_covers(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    outcome: &RegionFormationOutcome,
    policy: CoverPolicy,
) -> Result<CoverEnumeration, CoverError> {
    let graph = outcome.graph();
    let candidates = outcome.candidates();
    let operation_count = graph.operation_count();

    let infeasibilities = detect_infeasibilities(program);
    if !infeasibilities.is_empty() {
        return Ok(CoverEnumeration {
            policy,
            covers: Vec::new(),
            refusals: Vec::new(),
            budget_stops: Vec::new(),
            infeasibilities,
            operation_count,
            node_count: graph.node_count(),
        });
    }

    let graph_identity = program.semantic_identity().graph().as_bytes().to_vec();
    let legality = DuplicationLegality::derive(graph, candidates, policy)?;
    let named_outputs = named_output_positions(candidates);

    // Candidate indices containing each operation, in the candidates' canonical
    // order, so the anchored search is deterministic.
    let mut containing: Vec<Vec<usize>> =
        vec![Vec::new(); usize::try_from(graph.node_count()).unwrap_or(usize::MAX)];
    for (index, candidate) in candidates.iter().enumerate() {
        for atom in candidate.members() {
            let slot =
                containing
                    .get_mut(atom_index(graph, *atom))
                    .ok_or(CoverError::Structure {
                        rule: "member-ordinal",
                    })?;
            slot.push(index);
        }
    }

    let mut retained: BTreeMap<RegionCoverIdentity, RegionCover> = BTreeMap::new();

    // The fully-materialized (all-singleton) cover is retained unconditionally.
    let singletons = collect_singletons(graph, candidates)?;
    let materialized = assemble_cover(graph, candidates, &graph_identity, &singletons)?;
    retained.insert(materialized.identity.clone(), materialized);

    // The fused (whole-program) cover is retained whenever it exists.
    if let Some(fused_index) = candidates
        .iter()
        .position(RegionCandidate::covers_whole_program)
    {
        let fused = assemble_cover(graph, candidates, &graph_identity, &[fused_index])?;
        retained.entry(fused.identity.clone()).or_insert(fused);
    }

    let mut partitioner = Partitioner {
        graph,
        candidates,
        containing: &containing,
        graph_identity: &graph_identity,
        legality: &legality,
        named_outputs: &named_outputs,
        policy,
        max_covers: usize::try_from(budgets.region_covers).unwrap_or(usize::MAX),
        max_expansions: budgets.region_cover_expansions,
        max_refusals: MAX_RETAINED_REFUSALS,
        retained,
        refusals: BTreeSet::new(),
        stops: BTreeMap::new(),
        expansions: 0,
        covered: vec![0_u32; usize::try_from(graph.node_count()).unwrap_or(usize::MAX)],
        memo: BTreeMap::new(),
        memoized_completions: 0,
    };
    partitioner.run()?;

    let mut covers: Vec<RegionCover> = partitioner.retained.into_values().collect();
    covers.sort_by(|left, right| left.identity.as_bytes().cmp(right.identity.as_bytes()));
    Ok(CoverEnumeration {
        policy,
        covers,
        node_count: graph.node_count(),
        refusals: partitioner.refusals.into_iter().collect(),
        budget_stops: partitioner.stops.into_values().collect(),
        infeasibilities: Vec::new(),
        operation_count,
    })
}

/// Re-derives and validates one proposed cover of a program.
///
/// The program's candidates are re-derived by region formation; each placed
/// region must be one of those authoritative occurrences (preserving occurrence
/// identity), every operation must be covered, every operation covered more than
/// once must be a duplication `policy` and `contract` admit, every ordered named
/// output must be produced by exactly one region, no region may be unobservable
/// under a duplication-admitting policy, and the recomputed materialization
/// edges, duplication, cost, and identity must match. Any deviation fails closed
/// with a typed [`CoverError`].
///
/// # Errors
///
/// Returns a [`CoverError`] whose [`CoverError::class`] is `region` for a broken
/// occurrence identity, `coverage` for an uncovered operation, a refused
/// duplication, an unobservable region, an ambiguous named output, or an
/// unretained named output, and `structure` for a mismatched materialization,
/// duplication, cost, or identity.
/// The formation is taken rather than derived, for the reason
/// [`enumerate_covers`] gives. This runs **once per cover and once per retained
/// plan**, so re-deriving it here was the largest single multiplier on the
/// region search anywhere in the compile path.
pub(crate) fn verify_cover(
    program: &SemanticProgram,
    outcome: &RegionFormationOutcome,
    policy: CoverPolicy,
    cover: &RegionCover,
) -> Result<(), CoverError> {
    let graph = outcome.graph();
    // Keyed by reference. The map is a lookup table over candidates the outcome
    // already owns and outlives this call, so owning the keys copied every
    // candidate's occurrence encoding — the largest identity in the stage — once
    // per placed region per verification, and this runs once per cover and once
    // per retained plan.
    let mut authoritative: BTreeMap<&RegionOccurrenceIdentity, &RegionCandidate> = BTreeMap::new();
    for candidate in outcome.candidates() {
        authoritative.insert(candidate.occurrence(), candidate);
    }

    // 1. Occurrence-identity preservation: each placed region is an authoritative
    //    candidate whose members, content, and label agree.
    let mut resolved: Vec<&RegionCandidate> = Vec::with_capacity(cover.regions.len());
    for region in &cover.regions {
        let candidate =
            authoritative
                .get(&region.occurrence)
                .copied()
                .ok_or(CoverError::Region(RegionError::Invalid {
                    region: region.label.to_string(),
                    rule: "unknown-region-occurrence",
                }))?;
        // The retained named results are checked here rather than folded into
        // cover identity, for the reason `CoverRegion::named_results` records:
        // the occurrence identity already encodes the retained output sites and
        // their `named_result` flags, so this proves the projection agrees with
        // the bytes rather than encoding the same fact twice. Program assembly
        // then attributes each declared output by value on a checked field.
        if region.members.as_slice() != candidate.members()
            || &region.content != candidate.content()
            || region.named_results != retained_named_results(candidate)
            || &*region.label != candidate.label()
        {
            return Err(CoverError::Structure {
                rule: "region-occurrence-mismatch",
            });
        }
        resolved.push(candidate);
    }

    // 2. Coverage multiset: every operation covered, and every operation covered
    //    more than once admitted by the legality condition rather than by the
    //    cover having recorded it.
    let legality = DuplicationLegality::derive(graph, outcome.candidates(), policy)?;
    let mut counts = vec![0_u32; usize::try_from(graph.node_count()).unwrap_or(usize::MAX)];
    for candidate in &resolved {
        for atom in candidate.members() {
            let slot = counts
                .get_mut(atom_index(graph, *atom))
                .ok_or(CoverError::Structure {
                    rule: "member-ordinal",
                })?;
            *slot = slot.saturating_add(1);
        }
    }
    // Every *stage* covered exactly once — the mask obligation. A missing later
    // stage refuses as the member left uncovered, and any stage covered twice
    // is duplication of the occurrence, whose legality is the occurrence's.
    for (node, count) in counts.iter().enumerate() {
        let member = graph
            .node_atom(u32::try_from(node).unwrap_or(u32::MAX))
            .map_err(CoverError::Region)?
            .member();
        match count {
            0 => return Err(CoverError::UncoveredMember { member }),
            1 => {}
            _ => {
                if let Some(refusal) = legality.refusal(member) {
                    return Err(CoverError::IllegalDuplication { member, refusal });
                }
            }
        }
    }

    // 3. Named-output coverage: no bare-input passthrough, every ordered named
    //    output produced by exactly one region.
    if !detect_infeasibilities(program).is_empty() {
        return Err(CoverError::UncoveredNamedOutput {
            reason: "unrooted-named-output",
        });
    }
    let named_outputs = named_output_positions(outcome.candidates());
    check_named_outputs(&named_outputs, &resolved)?;

    // 4. Materialization edges, duplication, and cost recompute exactly.
    let materializations = derive_materializations(graph, &resolved)?;
    if materializations != cover.materializations {
        return Err(CoverError::Structure {
            rule: "materialization-mismatch",
        });
    }

    // 5. Observability: under a duplication-admitting policy the search itself
    //    can place a region nothing reads, so every region must be observed —
    //    decided against the recomputed edges, because which owner of a
    //    duplicated value materializes it is what makes the other owners' copies
    //    observable or not.
    if policy.admits_duplication() {
        check_regions_observed(&resolved, &materializations)?;
    }
    let duplication = derive_duplication(graph, &resolved)?;
    if duplication != cover.duplication {
        return Err(CoverError::Structure {
            rule: "duplication-mismatch",
        });
    }
    let cost = derive_cover_cost(graph, &resolved, &materializations)?;
    if cost != cover.cost {
        return Err(CoverError::Structure {
            rule: "cover-cost-mismatch",
        });
    }

    // 6. Canonical region order and cover identity recompute exactly.
    //
    // Sorting a copy and comparing it decides exactly this predicate: the sort
    // is stable and keyed on the occurrence bytes, so it returns the vector
    // unchanged precisely when those keys are already non-decreasing. Checking
    // the keys directly avoids copying every placed region — members, both
    // identities, and the label — once per verification.
    if cover
        .regions
        .windows(2)
        .any(|pair| pair[0].occurrence.as_bytes() > pair[1].occurrence.as_bytes())
    {
        return Err(CoverError::Structure {
            rule: "region-order",
        });
    }
    let graph_identity = program.semantic_identity().graph().as_bytes();
    let identity = encode_cover_identity(
        graph_identity,
        &cover.regions,
        &cover.duplication,
        &materializations,
    );
    if identity != cover.identity {
        return Err(CoverError::Structure {
            rule: "cover-identity-mismatch",
        });
    }
    Ok(())
}

/// How many typed refusals one enumeration retains.
///
/// The refused space is exponential, so the explanation is bounded and its
/// truncation is reported as a [`CoverBudgetResource::Refusals`] stop rather
/// than silently dropped. Refusals are deduplicated by subject and reason first,
/// so the bound is over *distinct* refusals: a search that rejects one region
/// ten thousand times for one reason spends one slot.
const MAX_RETAINED_REFUSALS: usize = 64;

/// The per-member duplication legality of one program under one contract.
///
/// Derived once and read many times, because it is a function of the operation
/// and the governing contract alone: it does not depend on which cover is being
/// assembled, so re-deciding it per branch would re-derive one answer across the
/// whole search space.
struct DuplicationLegality {
    /// `None` for a member that may be duplicated; the refusal otherwise.
    refusals: Vec<Option<DuplicationRefusal>>,
}

impl DuplicationLegality {
    /// Decides, for each operation, whether computing it twice preserves the
    /// program.
    ///
    /// Three conditions, each checked rather than assumed:
    ///
    /// - the contract must grant no realization freedom, or two realizations of
    ///   one occurrence could differ and the two copies would not be one value;
    /// - the frozen semantic authority must have proved the operation
    ///   referentially transparent, or a second evaluation is a second effect;
    /// - the operation must produce no ordered named program output, because a
    ///   named output has one destination and two producers of it are two
    ///   writers rather than two copies.
    ///
    /// The contract check is first and whole-program: it refuses every member at
    /// once, which is the honest shape — the refusal is about the contract, not
    /// about any operation.
    fn derive(
        graph: &RegionGraph,
        candidates: &[RegionCandidate],
        policy: CoverPolicy,
    ) -> Result<Self, CoverError> {
        let count = usize::try_from(graph.operation_count()).unwrap_or(usize::MAX);
        if !policy.admits_duplication() {
            return Ok(Self {
                refusals: vec![Some(DuplicationRefusal::PolicyForbids); count],
            });
        }
        if policy.numerical_contract().grants_realization_freedom() {
            return Ok(Self {
                refusals: vec![Some(DuplicationRefusal::ContractGrantsRealizationFreedom); count],
            });
        }
        let mut refusals = vec![None; count];
        for position in 0..graph.operation_count() {
            let member = SemanticMemberId(position);
            if !graph.member_operation_facts(member)?.is_pure()
                && let Some(slot) = refusals.get_mut(member_index(member))
            {
                *slot = Some(DuplicationRefusal::ImpureMember);
            }
        }
        // A named result's producer is read from the singleton candidates'
        // retained outputs rather than re-walked from the program: the candidate
        // set is the authority this stage already trusts for what a region
        // exports, and its `named_result` flag is the same fact the coverage
        // check below uses.
        for candidate in candidates {
            for output in candidate.retained_outputs() {
                if output.named_result
                    && let Some(slot) = refusals.get_mut(member_index(output.producer))
                    && slot.is_none()
                {
                    *slot = Some(DuplicationRefusal::NamedResultProducer);
                }
            }
        }
        Ok(Self { refusals })
    }

    /// Returns why this member may not be duplicated, or `None` if it may.
    ///
    /// A member ordinal the graph does not hold reads as refused. That direction
    /// is the fail-closed one: an unknown operation is not one this authority has
    /// proved safe to recompute.
    fn refusal(&self, member: SemanticMemberId) -> Option<DuplicationRefusal> {
        self.refusals
            .get(member_index(member))
            .copied()
            .unwrap_or(Some(DuplicationRefusal::ImpureMember))
    }
}

/// One ordered named program output and the member that produces it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NamedOutput {
    /// Ordered position of the output in the program's declared interface.
    position: u32,
    /// Graph-local ordinal of the exported value.
    value: u32,
}

/// The state one search branch has covered, as a per-member coverage count.
type CoverageMask = Vec<u32>;

/// What the memo knows about the coverage completions of one state.
enum MemoEntry {
    /// Every completion of this state, as candidate index sets.
    Completions(Vec<Vec<usize>>),
    /// The state has more completions than the memo will hold; recompute.
    Unbounded,
}

/// The deterministic anchored cover search state.
struct Partitioner<'a> {
    graph: &'a RegionGraph,
    candidates: &'a [RegionCandidate],
    containing: &'a [Vec<usize>],
    graph_identity: &'a [u8],
    legality: &'a DuplicationLegality,
    named_outputs: &'a [NamedOutput],
    policy: CoverPolicy,
    max_covers: usize,
    max_expansions: u64,
    max_refusals: usize,
    retained: BTreeMap<RegionCoverIdentity, RegionCover>,
    refusals: BTreeSet<CoverRefusal>,
    stops: BTreeMap<CoverBudgetResource, CoverBudgetStop>,
    expansions: u64,
    /// How many chosen regions on the current branch cover each operation.
    ///
    /// A count rather than a flag, because a duplication-admitting branch covers
    /// an operation several times and backtracking must undo exactly one of
    /// them. One vector mutated in place and undone on backtrack, rather than a
    /// set copied per branch: the search visits a node per expansion under the
    /// `region_cover_expansions` budget, so a per-branch copy is multiplied by
    /// the whole search space.
    covered: CoverageMask,
    /// Coverage completions already derived, keyed by the state that produced
    /// them.
    ///
    /// **Sound because admissibility depends on the state alone.** A candidate
    /// is admissible from a state exactly when it contains the state's minimum
    /// uncovered operation and every already-covered operation it re-covers is
    /// duplication-legal — both read only the coverage counts and the
    /// per-member legality, never the path that produced them. So two branches
    /// reaching one state have the same completions, and a completion may be
    /// replayed under a different prefix. The prefixes are disjoint from their
    /// completions (a re-chosen candidate contains no uncovered operation, so
    /// the anchor rule refuses it), which is why prefix and completion compose
    /// into a candidate set rather than collapsing.
    memo: BTreeMap<CoverageMask, MemoEntry>,
    /// How many completion vectors the memo holds, bounded by `max_covers`.
    memoized_completions: usize,
}

/// Whether a search branch should continue or the whole search must stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flow {
    Continue,
    Stop,
}

/// What one visit of a state produced.
struct Visit {
    flow: Flow,
    /// Every coverage completion of the visited state, when the visit derived
    /// all of them. `None` means the visit was cut short — by a budget, or by
    /// the completion list exceeding what the memo will hold — so the caller
    /// must not record its own completions as complete either.
    completions: Option<Vec<Vec<usize>>>,
}

impl Partitioner<'_> {
    fn run(&mut self) -> Result<(), CoverError> {
        let mut chosen: Vec<usize> = Vec::new();
        self.visit(&mut chosen)?;
        Ok(())
    }

    /// Extends `chosen` with every region anchored on the minimum uncovered
    /// operation, and returns the visited state's coverage completions.
    ///
    /// The anchor strictly advances at every step, which is what terminates the
    /// search: a chosen candidate always covers the current minimum uncovered
    /// operation, so the next minimum is strictly larger. Under an
    /// exact-partition contract each cover is generated exactly once; under a
    /// duplication-admitting one a set whose regions between them contain the
    /// anchor in several places can be reached by more than one order, and the
    /// retention map keyed on cover identity is what makes that a wasted
    /// expansion rather than a repeated cover.
    fn visit(&mut self, chosen: &mut Vec<usize>) -> Result<Visit, CoverError> {
        let state = self.covered.clone();
        if state.iter().all(|count| *count > 0) {
            // The state is complete, so its one completion is the empty set.
            let flow = self.emit(chosen)?;
            return Ok(Visit {
                flow,
                completions: Some(vec![Vec::new()]),
            });
        }
        if let Some(completions) = self.memoized_completions_for(&state) {
            // The coverage is applied around the replay, not only the chosen
            // list: `emit` augments a complete cover with regions whose members
            // are already covered, and it decides that from the coverage state.
            // Replaying a completion without applying it would offer the
            // augmentation a state the cover does not have.
            let candidates = self.candidates;
            for completion in &completions {
                let before = chosen.len();
                for &index in completion {
                    if let Some(candidate) = candidates.get(index) {
                        self.mark(candidate, true);
                    }
                }
                chosen.extend_from_slice(completion);
                let flow = self.emit(chosen)?;
                chosen.truncate(before);
                for &index in completion {
                    if let Some(candidate) = candidates.get(index) {
                        self.mark(candidate, false);
                    }
                }
                if flow == Flow::Stop {
                    return Ok(Visit {
                        flow: Flow::Stop,
                        completions: None,
                    });
                }
            }
            return Ok(Visit {
                flow: Flow::Continue,
                completions: Some(completions),
            });
        }
        let anchor = state
            .iter()
            .position(|count| *count == 0)
            .ok_or(CoverError::Structure {
                rule: "anchor-ordinal",
            })?;
        // Both are shared borrows of data that outlives the search, so reading
        // them out here keeps the anchored index list and the candidate usable
        // across the recursive `&mut self` call without copying either.
        let candidates = self.candidates;
        let anchored = self
            .containing
            .get(anchor)
            .ok_or(CoverError::Structure {
                rule: "anchor-ordinal",
            })?
            .as_slice();
        let mut completions: Vec<Vec<usize>> = Vec::new();
        let mut derived_all = true;
        for &index in anchored {
            self.expansions = self.expansions.saturating_add(1);
            if self.expansions > self.max_expansions {
                self.record_stop(
                    CoverBudgetResource::Expansions,
                    self.max_expansions,
                    self.expansions,
                );
                return Ok(Visit {
                    flow: Flow::Stop,
                    completions: None,
                });
            }
            let candidate = candidates.get(index).ok_or(CoverError::Structure {
                rule: "candidate-index",
            })?;
            if let Some((member, cause)) = self.refused_duplication(candidate) {
                // A blanket policy refusal is the exact-partition rule itself
                // rather than a fact about this candidate, and every overlapping
                // candidate earns one. Recording them would fill the explanation
                // with one repeated sentence and crowd out — under the retained
                // refusal bound — the refusals that are about the program.
                if cause != DuplicationRefusal::PolicyForbids {
                    let position = self.graph.member_canonical_position(member)?;
                    self.record_refusal(CoverRefusal::Duplication {
                        region: candidate.occurrence().clone(),
                        position,
                        refusal: cause,
                    });
                }
                continue;
            }
            self.mark(candidate, true);
            chosen.push(index);
            let visit = self.visit(chosen)?;
            chosen.pop();
            self.mark(candidate, false);
            if visit.flow == Flow::Stop {
                return Ok(Visit {
                    flow: Flow::Stop,
                    completions: None,
                });
            }
            match visit.completions {
                Some(child) if derived_all => {
                    for completion in child {
                        let mut extended = Vec::with_capacity(completion.len() + 1);
                        extended.push(index);
                        extended.extend_from_slice(&completion);
                        completions.push(extended);
                    }
                    // The completion list is bounded by the same budget the memo
                    // is: a state with more completions than the search will
                    // retain covers is one whose completions are not worth
                    // carrying up, and dropping them here is what keeps the
                    // search's memory bounded by a declared number rather than
                    // by the shape of the graph.
                    if completions.len() > self.max_covers {
                        derived_all = false;
                        completions = Vec::new();
                    }
                }
                Some(_) | None => derived_all = false,
            }
        }
        if derived_all {
            self.remember(state, completions.clone());
            return Ok(Visit {
                flow: Flow::Continue,
                completions: Some(completions),
            });
        }
        self.memo.insert(state, MemoEntry::Unbounded);
        Ok(Visit {
            flow: Flow::Continue,
            completions: None,
        })
    }

    /// Returns the first member this candidate would re-cover illegally.
    ///
    /// A member ordinal outside the coverage vector reads as covered, so the
    /// branch is refused exactly as an out-of-range membership test refused it.
    /// Neither can happen: `enumerate_covers` already rejected an out-of-range
    /// member while building `containing`.
    fn refused_duplication(
        &self,
        candidate: &RegionCandidate,
    ) -> Option<(SemanticMemberId, DuplicationRefusal)> {
        candidate.members().iter().find_map(|atom| {
            let covered = self
                .covered
                .get(atom_index(self.graph, *atom))
                .copied()
                .unwrap_or(u32::MAX);
            (covered > 0)
                .then(|| self.legality.refusal(atom.member()))
                .flatten()
                .map(|cause| (atom.member(), cause))
        })
    }

    /// Adds or removes one region's coverage on the current branch.
    fn mark(&mut self, candidate: &RegionCandidate, covering: bool) {
        for atom in candidate.members() {
            if let Some(slot) = self.covered.get_mut(atom_index(self.graph, *atom)) {
                *slot = if covering {
                    slot.saturating_add(1)
                } else {
                    slot.saturating_sub(1)
                };
            }
        }
    }

    /// Returns this state's memoized completions, when the memo holds them all.
    fn memoized_completions_for(&self, state: &CoverageMask) -> Option<Vec<Vec<usize>>> {
        match self.memo.get(state) {
            Some(MemoEntry::Completions(completions)) => Some(completions.clone()),
            Some(MemoEntry::Unbounded) | None => None,
        }
    }

    /// Records what a state's completions are, or that the memo will not hold them.
    fn remember(&mut self, state: CoverageMask, completions: Vec<Vec<usize>>) {
        if self.memoized_completions.saturating_add(completions.len()) > self.max_covers {
            self.memo.insert(state, MemoEntry::Unbounded);
            return;
        }
        self.memoized_completions = self.memoized_completions.saturating_add(completions.len());
        self.memo.insert(state, MemoEntry::Completions(completions));
    }

    /// Emits the complete coverage on `chosen` and every augmentation of it.
    ///
    /// **The anchored search alone is not complete over covers.** It admits a
    /// candidate only when that candidate covers the branch's minimum uncovered
    /// operation, so a region every one of whose operations is already covered
    /// can never be chosen — and such a region is not idle. In the fan-out
    /// fixture, `{shared}` beside `{constant, shared, left}` is exactly it: the
    /// large region absorbs its own copy while the small one materializes for
    /// the remaining consumer, which is one of the two ways to spell that
    /// partial duplication. Enumerating only the anchored half would have lost
    /// it, and the exhaustive oracle is what said so.
    ///
    /// Completeness is exact rather than approximate: run the anchor rule over
    /// any legal cover `S` and it selects a subset `B` of `S` covering
    /// everything, leaving every remaining region of `S` with all its operations
    /// covered by `B`. So every legal cover is an anchored base plus an
    /// augmentation, which is what this enumerates. Augmenting candidates are
    /// taken in increasing index order, so each augmented set is generated once,
    /// and the index strictly advances, so the recursion terminates.
    fn emit(&mut self, chosen: &mut Vec<usize>) -> Result<Flow, CoverError> {
        self.augment(chosen, 0)
    }

    /// Retains the current coverage, then extends it by pure-duplicate regions.
    fn augment(&mut self, chosen: &mut Vec<usize>, from: usize) -> Result<Flow, CoverError> {
        if self.retain(chosen)? == Flow::Stop {
            return Ok(Flow::Stop);
        }
        if !self.policy.admits_duplication() {
            // Every augmenting region duplicates by construction, so an
            // exact-partition contract has nothing to add and the scan is not
            // worth its expansions.
            return Ok(Flow::Continue);
        }
        let candidates = self.candidates;
        for index in from..candidates.len() {
            if chosen.contains(&index) {
                continue;
            }
            let candidate = candidates.get(index).ok_or(CoverError::Structure {
                rule: "candidate-index",
            })?;
            // An out-of-range member reads as *covered*, the mirror of the
            // default [`Self::refused_duplication`] takes, so a member the
            // coverage vector cannot hold never makes a candidate look like an
            // augmentation. It cannot happen, for the reason that method's doc
            // states.
            let covers_new = candidate.members().iter().any(|atom| {
                self.covered
                    .get(atom_index(self.graph, *atom))
                    .copied()
                    .unwrap_or(1)
                    == 0
            });
            if covers_new {
                // Not an augmentation: the anchored search reaches every region
                // that covers something new, and admitting one here would
                // generate the same cover a second time.
                continue;
            }
            if let Some((member, cause)) = self.refused_duplication(candidate) {
                // The blanket policy refusal goes unrecorded here for the reason
                // the anchored loop above states: it is the exact-partition rule
                // itself rather than a fact about this candidate.
                if cause != DuplicationRefusal::PolicyForbids {
                    let position = self.graph.member_canonical_position(member)?;
                    self.record_refusal(CoverRefusal::Duplication {
                        region: candidate.occurrence().clone(),
                        position,
                        refusal: cause,
                    });
                }
                continue;
            }
            self.expansions = self.expansions.saturating_add(1);
            if self.expansions > self.max_expansions {
                self.record_stop(
                    CoverBudgetResource::Expansions,
                    self.max_expansions,
                    self.expansions,
                );
                return Ok(Flow::Stop);
            }
            self.mark(candidate, true);
            chosen.push(index);
            let flow = self.augment(chosen, index + 1)?;
            chosen.pop();
            self.mark(candidate, false);
            if flow == Flow::Stop {
                return Ok(Flow::Stop);
            }
        }
        Ok(Flow::Continue)
    }

    fn retain(&mut self, chosen: &[usize]) -> Result<Flow, CoverError> {
        // A complete coverage is not yet a legal cover: the global conditions —
        // one producer per ordered named output, and no region the cover leaves
        // unobserved — are properties of the whole set and cannot be decided
        // while it is being built.
        let mut resolved: Vec<&RegionCandidate> = Vec::with_capacity(chosen.len());
        for &index in chosen {
            resolved.push(self.candidates.get(index).ok_or(CoverError::Structure {
                rule: "candidate-index",
            })?);
        }
        let materializations = derive_materializations(self.graph, &resolved)?;
        if let Err(refusal) = classify_global_legality(
            self.named_outputs,
            &resolved,
            &materializations,
            self.policy,
        ) {
            self.record_refusal(refusal);
            return Ok(Flow::Continue);
        }
        let cover =
            assemble_resolved_cover(self.graph, &resolved, self.graph_identity, materializations)?;
        if self.retained.contains_key(&cover.identity) {
            return Ok(Flow::Continue);
        }
        if self.retained.len() >= self.max_covers {
            let limit = count(self.max_covers);
            self.record_stop(CoverBudgetResource::Covers, limit, limit.saturating_add(1));
            return Ok(Flow::Stop);
        }
        self.retained.insert(cover.identity.clone(), cover);
        Ok(Flow::Continue)
    }

    /// Retains one distinct refusal, reporting the bound rather than dropping it.
    fn record_refusal(&mut self, refusal: CoverRefusal) {
        if self.refusals.contains(&refusal) {
            return;
        }
        if self.refusals.len() >= self.max_refusals {
            let limit = count(self.max_refusals);
            self.record_stop(
                CoverBudgetResource::Refusals,
                limit,
                limit.saturating_add(1),
            );
            return;
        }
        self.refusals.insert(refusal);
    }

    fn record_stop(&mut self, resource: CoverBudgetResource, limit: u64, actual: u64) {
        let stop = self.stops.entry(resource).or_insert(CoverBudgetStop {
            resource,
            limit,
            actual,
        });
        stop.actual = stop.actual.max(actual);
    }
}

/// Decides the whole-cover legality conditions no single region can see.
///
/// Two of them, and both are about *observability* rather than coverage:
///
/// - every ordered named program output must be produced by exactly one region,
///   because a named output is written once to a definite destination;
/// - under a duplication-admitting contract, no region may be one nothing
///   observes. That check is deliberately skipped under an exact-partition
///   contract: there, a region whose results reach nothing reflects a dead
///   operation in the program, which normalization owns and this authority must
///   not start refusing programs over.
///
/// **The first is defence in depth and is currently unreachable from the search,
/// stated so rather than presented as a live check.** A second producer of a
/// named output would have to be a second region containing that output's
/// producing member, and [`duplication_refusal`] refuses that member with
/// [`DuplicationRefusal::NamedResultProducer`] before a cover containing both
/// can be assembled. It is kept because it is the condition, not the
/// consequence: the day the duplication condition admits a named-result producer
/// — a profile that writes one output from several regions under an ownership
/// discipline — this is what stops the two writers being accepted silently, and
/// it is driven directly by
/// `the_named_output_and_observability_checks_can_say_no`.
fn classify_global_legality(
    named_outputs: &[NamedOutput],
    resolved: &[&RegionCandidate],
    materializations: &[MaterializationEdge],
    policy: CoverPolicy,
) -> Result<(), CoverRefusal> {
    for output in named_outputs {
        if named_output_producers(resolved, *output) > 1 {
            return Err(CoverRefusal::AmbiguousNamedOutput {
                output_position: output.position,
            });
        }
    }
    if !policy.admits_duplication() {
        return Ok(());
    }
    for candidate in resolved {
        if !region_is_observed(candidate, materializations) {
            return Err(CoverRefusal::DeadRegion {
                region: candidate.occurrence().clone(),
            });
        }
    }
    Ok(())
}

/// Counts the regions producing one ordered named program output.
fn named_output_producers(resolved: &[&RegionCandidate], output: NamedOutput) -> usize {
    resolved
        .iter()
        .filter(|candidate| {
            candidate
                .retained_outputs()
                .iter()
                .any(|retained| retained.value.0 == output.value && retained.named_result)
        })
        .count()
}

/// Returns whether anything observes a region.
///
/// A region is observed when it produces an ordered named program output, or
/// when it is the *designated materializing owner* of at least one edge. The
/// second half is why this reads the derived edges rather than the raw
/// consumer lists: a duplicated value has several owners and only one of them
/// materializes it, so an owner whose copy every reader gets from somewhere else
/// computes something nothing can observe. That is work the search itself
/// invented, and it is the exact redundancy a duplication-admitting contract has
/// to refuse rather than merely cost.
fn region_is_observed(
    candidate: &RegionCandidate,
    materializations: &[MaterializationEdge],
) -> bool {
    candidate
        .retained_outputs()
        .iter()
        .any(|output| output.named_result)
        || materializations
            .iter()
            .any(|edge| edge.producer() == candidate.occurrence())
}

/// Fails a claimed-legal cover whose named outputs are not produced exactly once.
fn check_named_outputs(
    named_outputs: &[NamedOutput],
    resolved: &[&RegionCandidate],
) -> Result<(), CoverError> {
    for output in named_outputs {
        match named_output_producers(resolved, *output) {
            0 => {
                return Err(CoverError::UncoveredNamedOutput {
                    reason: "dropped-named-output",
                });
            }
            1 => {}
            _ => {
                return Err(CoverError::Unobservable {
                    refusal: CoverRefusal::AmbiguousNamedOutput {
                        output_position: output.position,
                    },
                });
            }
        }
    }
    Ok(())
}

/// Fails a claimed-legal cover carrying a region nothing observes.
fn check_regions_observed(
    resolved: &[&RegionCandidate],
    materializations: &[MaterializationEdge],
) -> Result<(), CoverError> {
    for candidate in resolved {
        if !region_is_observed(candidate, materializations) {
            return Err(CoverError::Unobservable {
                refusal: CoverRefusal::DeadRegion {
                    region: candidate.occurrence().clone(),
                },
            });
        }
    }
    Ok(())
}

/// Assembles one cover from a chosen set of candidate indices.
fn assemble_cover(
    graph: &RegionGraph,
    candidates: &[RegionCandidate],
    graph_identity: &[u8],
    chosen: &[usize],
) -> Result<RegionCover, CoverError> {
    let mut resolved: Vec<&RegionCandidate> = Vec::with_capacity(chosen.len());
    for &index in chosen {
        let candidate = candidates.get(index).ok_or(CoverError::Structure {
            rule: "candidate-index",
        })?;
        resolved.push(candidate);
    }
    let materializations = derive_materializations(graph, &resolved)?;
    assemble_resolved_cover(graph, &resolved, graph_identity, materializations)
}

/// Returns the ordered named program outputs one candidate retains, ascending.
///
/// The single derivation of the field [`CoverRegion::named_results`] carries, so
/// that populating a cover and verifying one ask the same question of the same
/// authority rather than two descriptions of it. Ascending and deduplicated
/// because a retained output list is keyed by value: two entries for one value
/// would be one export recorded twice, and the order must not depend on the
/// candidate's own authoring order.
fn retained_named_results(candidate: &RegionCandidate) -> Vec<SemanticValueId> {
    let mut values: Vec<SemanticValueId> = candidate
        .retained_outputs()
        .iter()
        .filter(|output| output.named_result)
        .map(|output| output.value)
        .collect();
    values.sort_unstable();
    values.dedup();
    values
}

/// Assembles one cover from already-resolved candidates and their derived edges.
///
/// The edges are taken rather than re-derived because the search decides a
/// cover's global legality from them before it retains the cover, and deriving
/// them twice per emitted cover was the whole of that check's cost.
fn assemble_resolved_cover(
    graph: &RegionGraph,
    resolved: &[&RegionCandidate],
    graph_identity: &[u8],
    materializations: Vec<MaterializationEdge>,
) -> Result<RegionCover, CoverError> {
    let duplication = derive_duplication(graph, resolved)?;
    let cost = derive_cover_cost(graph, resolved, &materializations)?;
    let mut regions: Vec<CoverRegion> = resolved
        .iter()
        .map(|candidate| CoverRegion {
            members: candidate.members().to_vec(),
            content: candidate.content().clone(),
            occurrence: candidate.occurrence().clone(),
            named_results: retained_named_results(candidate),
            label: candidate.label_handle(),
        })
        .collect();
    regions.sort_by(|left, right| left.occurrence.as_bytes().cmp(right.occurrence.as_bytes()));
    let identity = encode_cover_identity(graph_identity, &regions, &duplication, &materializations);
    Ok(RegionCover {
        regions,
        materializations,
        duplication,
        cost,
        identity,
    })
}

/// Derives the deliberately duplicated operations of one cover.
///
/// Expressed in content-derived canonical positions, ascending and
/// deduplicated, so the record does not depend on authoring order and can be
/// folded into cover identity.
///
/// **Keyed on the attribution atom, which is what separates a duplication from a
/// split.** Two regions computing one occurrence's *same* stage compute the same
/// value twice, and that is the recomputation this record names. Two regions
/// carrying two *different* stages of one occurrence realize it once between
/// them — a fold region and the normalization over its result each do part of
/// the work — so the occurrence appears twice in the cover's atoms and is not
/// duplicated at all. With a bare occurrence as the key the two are the same
/// observation, and every multi-region realization would be reported as a
/// recomputation the exact-partition contract then refuses.
fn derive_duplication(
    graph: &RegionGraph,
    regions: &[&RegionCandidate],
) -> Result<CoverDuplication, CoverError> {
    let mut seen: BTreeSet<SemanticStage> = BTreeSet::new();
    let mut duplicated: BTreeSet<u32> = BTreeSet::new();
    for region in regions {
        for atom in region.members() {
            if !seen.insert(*atom) {
                duplicated.insert(graph.member_canonical_position(atom.member())?);
            }
        }
    }
    if duplicated.is_empty() {
        return Ok(CoverDuplication::none());
    }
    Ok(CoverDuplication {
        duplicated: duplicated.into_iter().collect(),
    })
}

/// Derives the cross-region materialization edges of one complete cover.
///
/// Each retained value is mapped to the regions that produce it; a value read by
/// a region that does not itself produce it becomes one edge carrying every such
/// consuming region, materialized by the producing region with the smallest
/// occurrence identity. A value read as a boundary input by its own producer is
/// invalid cover state and fails closed.
fn derive_materializations(
    graph: &RegionGraph,
    regions: &[&RegionCandidate],
) -> Result<Vec<MaterializationEdge>, CoverError> {
    // Sorted vectors rather than maps, ordered by value ordinal exactly as the
    // maps were. A cover carries a handful of retained outputs and boundary
    // inputs, and this runs once per assembled cover and once per verification,
    // so the maps' node allocations — one per map plus one per consumed value
    // for the inner set — cost more than the lookups they served.
    let mut producers: Vec<(u32, usize, SemanticMemberId, u32)> = Vec::new();
    for (region_index, region) in regions.iter().enumerate() {
        for output in region.retained_outputs() {
            producers.push((
                output.value.0,
                region_index,
                output.producer,
                output.result_position,
            ));
        }
    }
    // Sorted by value, then by the producing region's member count, then by its
    // occurrence bytes, so the first entry for a value is the designated
    // materializing owner. A stable sort on the value alone would have made the
    // owner depend on enumeration order, which is exactly what cover identity
    // may not carry.
    producers.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| {
                regions[left.1]
                    .members()
                    .len()
                    .cmp(&regions[right.1].members().len())
            })
            .then_with(|| {
                regions[left.1]
                    .occurrence()
                    .as_bytes()
                    .cmp(regions[right.1].occurrence().as_bytes())
            })
            .then_with(|| left.1.cmp(&right.1))
    });

    // Scanned rather than binary-searched, because a value may now have several
    // producing regions and the question is whether *this* region is one of
    // them — a membership test over the entries for one value, not a lookup of
    // the single entry. A cover carries a handful of retained outputs, so the
    // scan is over single-digit lengths; the sort above is what the canonical
    // owner needs, not what this test needs.
    let mut consumers: Vec<(u32, usize)> = Vec::new();
    for (region_index, region) in regions.iter().enumerate() {
        for input in region.boundary_inputs() {
            let mut produced_elsewhere = false;
            for entry in &producers {
                if entry.0 != input.0 {
                    continue;
                }
                if entry.1 == region_index {
                    return Err(CoverError::Structure {
                        rule: "internal-boundary-input",
                    });
                }
                produced_elsewhere = true;
            }
            if produced_elsewhere {
                consumers.push((input.0, region_index));
            }
        }
    }
    consumers.sort_unstable();
    consumers.dedup();

    let mut edges: Vec<MaterializationEdge> = Vec::new();
    let mut previous_value: Option<u32> = None;
    for &(value, producer_region, producer_member, result_position) in &producers {
        // Only the canonical owner — the first entry for this value under the
        // sort above — materializes it. The later owners are duplications whose
        // copies are read inside their own regions.
        if previous_value == Some(value) {
            continue;
        }
        previous_value = Some(value);
        let start = consumers.partition_point(|entry| entry.0 < value);
        let consuming = &consumers[start..];
        let end = consuming.partition_point(|entry| entry.0 == value);
        if end == 0 {
            continue;
        }
        let producer_position = graph.member_canonical_position(producer_member)?;
        let element_count = graph.value_element_count(SemanticValueId(value))?;
        let mut consumer_occurrences: Vec<RegionOccurrenceIdentity> = consuming[..end]
            .iter()
            .map(|&(_, region_index)| regions[region_index].occurrence().clone())
            .collect();
        consumer_occurrences.sort();
        edges.push(MaterializationEdge {
            value: SemanticValueId(value),
            producer_position,
            result_position,
            element_count,
            producer: regions[producer_region].occurrence().clone(),
            consumers: consumer_occurrences,
        });
    }
    // Ordered by the canonical encoding, built once into one buffer with a span
    // per edge, rather than inside the comparator. The comparator spelling
    // encoded *both* operands into two fresh buffers on every comparison, so an
    // n-edge sort paid O(n log n) encodings and allocations to order n values —
    // and an edge key embeds whole occurrence identities, so each of those was a
    // large copy. Most covers here carry one or two edges, which is why the
    // trivial case returns before encoding anything at all. The resulting order
    // is identical: the key is the same byte string either way.
    if edges.len() < 2 {
        return Ok(edges);
    }
    let mut keys = Vec::new();
    let mut spans = Vec::with_capacity(edges.len());
    for edge in &edges {
        let start = keys.len();
        encode_materialization(&mut keys, edge);
        spans.push(start..keys.len());
    }
    let mut order: Vec<usize> = (0..edges.len()).collect();
    order.sort_by(|left, right| keys[spans[*left].clone()].cmp(&keys[spans[*right].clone()]));
    let mut sorted: Vec<Option<MaterializationEdge>> = edges.into_iter().map(Some).collect();
    order
        .into_iter()
        .map(|position| {
            sorted
                .get_mut(position)
                .and_then(Option::take)
                .ok_or(CoverError::Structure {
                    rule: "materialization-order",
                })
        })
        .collect()
}

/// Derives one cover's estimated cost from facts the cover itself determines.
///
/// `recomputed_elements` counts the results of every *repeated* occurrence of a
/// member, weighted by how many elements each result holds. The member's first
/// region in the cover's canonical order is the original and contributes
/// nothing; each later one contributes the whole cost of computing it again.
/// Weighting by elements rather than counting bare occurrences is what makes the
/// dimension comparable with `materialized_elements`, which is the comparison
/// the materialize-versus-recompute choice turns on.
fn derive_cover_cost(
    graph: &RegionGraph,
    regions: &[&RegionCandidate],
    materializations: &[MaterializationEdge],
) -> Result<CoverCost, CoverError> {
    let mut materialized_elements = 0_u64;
    for edge in materializations {
        materialized_elements = materialized_elements.saturating_add(edge.element_count);
    }
    let mut seen: BTreeSet<SemanticStage> = BTreeSet::new();
    let mut recomputed_elements = 0_u64;
    // The canonical order the "first region is the original" rule ranges over,
    // derived here rather than taken from the caller's argument order: the
    // assembled cover sorts its regions by occurrence bytes, so the cost must be
    // computed over that same order or a cover and its own verification would
    // disagree about which copy was the original.
    let mut ordered: Vec<&&RegionCandidate> = regions.iter().collect();
    ordered.sort_by(|left, right| {
        left.occurrence()
            .as_bytes()
            .cmp(right.occurrence().as_bytes())
    });
    for region in ordered {
        // Keyed on the attribution atom for the reason `derive_duplication` is:
        // a second *stage* of an occurrence is the rest of that occurrence's
        // work, not a second evaluation of work already done, so pricing it as
        // recomputation would charge a multi-region realization for computing
        // itself once.
        for atom in region.members() {
            if seen.insert(*atom) {
                continue;
            }
            // The member's *own* results, read from the graph rather than from
            // the region's exported list. The two differ exactly in the case
            // duplication creates: a recomputed member whose value is consumed
            // inside the region that recomputed it appears in no retained-output
            // list, and costing it from that list would report the recomputation
            // as free — which is the case this dimension exists to price.
            recomputed_elements =
                recomputed_elements.saturating_add(member_result_elements(graph, atom.member())?);
        }
    }
    Ok(CoverCost {
        model_key: COVER_COST_MODEL_KEY,
        region_count: count(regions.len()),
        materialization_count: count(materializations.len()),
        materialized_elements,
        recomputed_elements,
    })
}

/// Returns how many elements one member's results hold in total.
fn member_result_elements(
    graph: &RegionGraph,
    member: SemanticMemberId,
) -> Result<u64, CoverError> {
    let mut total = 0_u64;
    for value in graph.member_result_values(member)? {
        total = total.saturating_add(graph.value_element_count(value)?);
    }
    Ok(total)
}

/// Collects the singleton candidate covering each operation exactly once.
fn collect_singletons(
    graph: &RegionGraph,
    candidates: &[RegionCandidate],
) -> Result<Vec<usize>, CoverError> {
    let mut by_node: BTreeMap<u32, usize> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        // A singleton is one atom. Under staged realizations the materialized
        // cover is one region per *stage atom* — a staged occurrence's unfused
        // plan is its stages placed separately, exactly as formation's
        // unconditional singleton coverage emits them.
        if let [atom] = candidate.members() {
            let node = graph.atom_node(*atom).map_err(CoverError::Region)?;
            if by_node.insert(node, index).is_some() {
                return Err(CoverError::Structure {
                    rule: "duplicate-singleton",
                });
            }
        }
    }
    let mut singletons =
        Vec::with_capacity(usize::try_from(graph.node_count()).unwrap_or(usize::MAX));
    for node in 0..graph.node_count() {
        let index = by_node.get(&node).copied().ok_or(CoverError::Structure {
            rule: "missing-singleton",
        })?;
        singletons.push(index);
    }
    Ok(singletons)
}

/// Detects named outputs no operation region can cover.
fn detect_infeasibilities(program: &SemanticProgram) -> Vec<CoverInfeasibility> {
    let inputs: BTreeSet<ValueId> = program.inputs().map(|input| input.value()).collect();
    let mut unrooted = 0_u64;
    for output in program.outputs() {
        if inputs.contains(&output.value()) {
            unrooted = unrooted.saturating_add(1);
        }
    }
    if unrooted == 0 {
        Vec::new()
    } else {
        vec![CoverInfeasibility::UnrootedNamedOutput { count: unrooted }]
    }
}

/// Returns the ordered named program outputs a cover must produce exactly once.
///
/// Derived from the candidates rather than the program handles, because the
/// candidates are the authority this stage already trusts for what a region
/// exports, and their `named_result` flag is the same fact every coverage check
/// here reads.
///
/// **`position` is the rank in ascending value ordinal, which is a stable
/// reporting coordinate and not the output's declared position.** A program may
/// declare a later-produced value first — `pipeline::conformance`'s
/// reduction-epilogue fixture publishes `reduced` before `scaled` — so the two
/// disagree. Nothing here decides anything by declared position: this coordinate
/// reaches only [`CoverRefusal::AmbiguousNamedOutput`]'s explain text. Program
/// assembly, which does need the declared order, attributes by value ordinal
/// against [`CoverRegion::named_results`] instead.
fn named_output_positions(candidates: &[RegionCandidate]) -> Vec<NamedOutput> {
    let mut values: BTreeSet<u32> = BTreeSet::new();
    for candidate in candidates {
        for output in candidate.retained_outputs() {
            if output.named_result {
                values.insert(output.value.0);
            }
        }
    }
    values
        .into_iter()
        .enumerate()
        .map(|(position, value)| NamedOutput {
            position: u32::try_from(position).unwrap_or(u32::MAX),
            value,
        })
        .collect()
}

fn encode_cover_identity(
    graph_identity: &[u8],
    regions: &[CoverRegion],
    duplication: &CoverDuplication,
    materializations: &[MaterializationEdge],
) -> RegionCoverIdentity {
    let mut bytes = COVER_IDENTITY_TAG.to_vec();
    push_slice(&mut bytes, graph_identity);
    push_len(&mut bytes, regions.len());
    for region in regions {
        push_slice(&mut bytes, region.occurrence.as_bytes());
    }
    // Content-derived canonical positions rather than transient graph-local
    // ordinals, for the same reason every other coordinate here is: two programs
    // the IR gives one canonical graph identity may hold their operations in
    // different slots.
    push_len(&mut bytes, duplication.duplicated.len());
    for position in &duplication.duplicated {
        bytes.extend_from_slice(&position.to_be_bytes());
    }
    push_len(&mut bytes, materializations.len());
    for edge in materializations {
        encode_materialization(&mut bytes, edge);
    }
    RegionCoverIdentity(bytes.into())
}

fn encode_materialization(output: &mut Vec<u8>, edge: &MaterializationEdge) {
    output.extend_from_slice(&edge.producer_position.to_be_bytes());
    output.extend_from_slice(&edge.result_position.to_be_bytes());
    output.extend_from_slice(&edge.element_count.to_be_bytes());
    push_slice(output, edge.producer.as_bytes());
    push_len(output, edge.consumers.len());
    for consumer in &edge.consumers {
        push_slice(output, consumer.as_bytes());
    }
}

/// Indexes the per-*occurrence* vectors duplication legality keeps.
///
/// Duplication legality is a fact about the occurrence rather than about a
/// stage of it — the legality of recomputing an operation is a property of the
/// operation and the contract — so its vectors stay sized by the operation
/// count and an atom indexes them through [`SemanticStage::member`]. Coverage
/// masks and the candidate index are per-*atom* and go through [`atom_index`]:
/// the cover's obligation is that every realization stage is computed exactly
/// once, which is the mask obligation the single-stage counting could not
/// state.
fn member_index(member: SemanticMemberId) -> usize {
    usize::try_from(member.0).unwrap_or(usize::MAX)
}

/// Indexes the per-*atom* vectors the search keeps.
///
/// The formation graph's node space is the authority on which atoms exist; an
/// atom the graph does not hold indexes past every vector, which reads as
/// covered on the duplication side and never matches a mutable slot — the
/// documented defaults each site relies on.
fn atom_index(graph: &RegionGraph, atom: SemanticStage) -> usize {
    graph.atom_node(atom).map_or(usize::MAX, |node| {
        usize::try_from(node).unwrap_or(usize::MAX)
    })
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CoverBudgetResource, CoverCost, CoverEnumeration, CoverError, CoverInfeasibility,
        CoverPolicy, CoverRefusal, DuplicationRefusal, RegionCover, assemble_cover,
        enumerate_covers, verify_cover,
    };
    use crate::region::{
        RegionCandidate, RegionError, RegionFormationOutcome, SemanticMemberId, SemanticStage,
        StageOrdinal, form_region_candidates,
    };
    use crate::request::{DeterministicBudgets, StrictF32NumericalContract};
    use std::collections::{BTreeMap, BTreeSet};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

    /// The formation these cases run under, derived once per call site.
    ///
    /// The entry points take it rather than deriving it, so a test supplies the
    /// same value the compile path would have threaded in.
    fn formation_of(program: &SemanticProgram) -> RegionFormationOutcome {
        form_region_candidates(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .expect("the fixture forms regions")
    }

    /// A normalization feeding a pointwise consumer, formed with its law's
    /// stage structure: the smallest staged cover space the standard
    /// registries produce.
    fn rms_norm_program() -> SemanticProgram {
        use tiler_ir::semantic::{
            CanonicalField, OperationAttributes, RMS_NORM_EPS_BITS_ATTRIBUTE,
            RMS_NORM_REDUCED_AXES_ATTRIBUTE, multiply_f32_op, rms_norm_f32_axis_attribute,
            rms_norm_f32_eps_attribute, rms_norm_f32_op,
        };
        use tiler_ir::shape::Axis;
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

    fn staged_formation_of(program: &SemanticProgram) -> RegionFormationOutcome {
        let laws = tiler_ir::index::FrozenIndexRealizationLawRegistry::from_semantic(
            program.semantic_registry().clone(),
            tiler_ir::index::FrozenScalarRegistry::standard().unwrap(),
        )
        .unwrap();
        crate::region::form_region_candidates_with_realizations(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            &laws,
        )
        .unwrap()
    }

    /// The mask obligation: every realization stage covered exactly once.
    ///
    /// Four claims over the staged fixture. The materialized cover is one
    /// region per stage atom rather than per operation; a cover placing one
    /// stage beside the downstream consumer — the shape the stage-atom
    /// decision exists to admit — is enumerated and verifies; a cover missing
    /// a later stage refuses as the member left uncovered; and a cover
    /// covering one stage twice refuses as duplication of the occurrence.
    #[test]
    fn a_staged_programs_covers_account_for_every_stage() {
        let program = rms_norm_program();
        let outcome = staged_formation_of(&program);
        let graph = outcome.graph();
        let enumeration =
            enumerate_covers(&program, exhaustive_budgets(), &outcome, exact_partition()).unwrap();
        assert!(enumeration.is_coverable());
        for cover in enumeration.covers() {
            verify_cover(&program, &outcome, exact_partition(), cover).unwrap();
        }

        let materialized = enumeration
            .fully_materialized_cover()
            .expect("the all-singleton cover is retained");
        assert_eq!(
            u32::try_from(materialized.regions.len()).unwrap(),
            graph.node_count(),
            "the unfused plan places every stage atom separately"
        );
        assert!(
            enumeration.fused_cover().is_some(),
            "the whole-program candidate covers every stage atom"
        );

        // The staged member and its two atoms.
        let member = (0..graph.operation_count())
            .find(|member| graph.member_stage_count(*member) > 1)
            .unwrap();
        let fold = SemanticStage::at(SemanticMemberId(member), StageOrdinal(0));
        let pass = fold.next_stage();
        assert!(
            enumeration.covers().iter().any(|cover| {
                cover.regions.iter().any(|region| {
                    region.members.len() > 1
                        && region.members.contains(&pass)
                        && !region.members.contains(&fold)
                })
            }),
            "some cover fuses the pass into a region its fold is not part of"
        );

        // The refusals, on hand-assembled covers of real candidates.
        let candidates = outcome.candidates();
        let singleton = |atom: SemanticStage| {
            candidates
                .iter()
                .position(|candidate| candidate.members() == [atom])
                .expect("every atom's singleton is enumerated")
        };
        let multiply = (0..graph.operation_count())
            .find(|other| *other != member)
            .unwrap();
        let fold_index = singleton(fold);
        let pass_index = singleton(pass);
        let multiply_index = singleton(SemanticStage::first(SemanticMemberId(multiply)));
        let graph_identity = program.semantic_identity().graph().as_bytes().to_vec();

        let missing_pass = assemble_cover(
            graph,
            candidates,
            &graph_identity,
            &[fold_index, multiply_index],
        )
        .unwrap();
        assert!(matches!(
            verify_cover(&program, &outcome, exact_partition(), &missing_pass),
            Err(CoverError::UncoveredMember { member: uncovered }) if uncovered.0 == member
        ));

        let doubled_pass = assemble_cover(
            graph,
            candidates,
            &graph_identity,
            &[fold_index, pass_index, pass_index, multiply_index],
        )
        .unwrap();
        assert!(matches!(
            verify_cover(&program, &outcome, exact_partition(), &doubled_pass),
            Err(CoverError::IllegalDuplication { member: doubled, .. }) if doubled.0 == member
        ));
    }

    /// The exact-partition contract the compile path enumerates under.
    fn exact_partition() -> CoverPolicy {
        CoverPolicy::governed(StrictF32NumericalContract::governed())
    }

    /// The contract admitting legal shared-work duplication.
    fn with_duplication() -> CoverPolicy {
        CoverPolicy::permitting_shared_work_duplication(StrictF32NumericalContract::governed())
    }

    /// The governed serial-sum chain: two pointwise constants, multiply, add, sum.
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
    ///
    /// The fan-out fixture: operation 1 produces a value operations 2 and 3 both
    /// read, and neither consumer may be duplicated because each is a named
    /// result. It is the smallest program in which materializing, absorbing one
    /// consumer, and absorbing both are three different covers.
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

    /// A rooted result plus a named output that exports a boundary input directly.
    fn passthrough_output_program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let constant = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, constant).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), product)
            .unwrap();
        builder
            .output(OutputKey::new("passthrough").unwrap(), input)
            .unwrap();
        builder.build().unwrap()
    }

    fn enumerate(program: &SemanticProgram) -> CoverEnumeration {
        enumerate_under(program, exact_partition())
    }

    fn enumerate_under(program: &SemanticProgram, policy: CoverPolicy) -> CoverEnumeration {
        enumerate_covers(
            program,
            exhaustive_budgets(),
            &formation_of(program),
            policy,
        )
        .unwrap()
    }

    /// Budgets wide enough that these fixtures' searches run to exhaustion.
    ///
    /// Stated by the test rather than inherited from the governed profile,
    /// because the governed numbers are sized for the compile path's
    /// exact-partition contract and a duplication-admitting search over a
    /// five-operation graph legitimately exceeds them. A test that asserts
    /// agreement with an exhaustive oracle has to give the search the room to be
    /// exhaustive; the budget-stop case is asserted separately, on its own
    /// deliberately tight budget.
    fn exhaustive_budgets() -> DeterministicBudgets {
        let mut budgets = DeterministicBudgets::governed();
        budgets.region_covers = u32::MAX;
        budgets.region_cover_expansions = u64::MAX;
        budgets
    }

    /// Each cover as the set of its regions' ascending member vectors.
    fn cover_partitions(enumeration: &CoverEnumeration) -> BTreeSet<BTreeSet<Vec<u32>>> {
        enumeration
            .covers()
            .iter()
            .map(|cover| {
                cover
                    .regions()
                    .iter()
                    .map(|region| {
                        region
                            .members()
                            .iter()
                            .map(|atom| atom.member().0)
                            .collect()
                    })
                    .collect()
            })
            .collect()
    }

    /// An independent exhaustive oracle over every subset of candidate regions.
    ///
    /// It brute-forces the powerset and keeps the subsets satisfying the *stated*
    /// legality conditions, re-derived here from each candidate's members,
    /// boundary inputs, and retained outputs rather than by calling the search's
    /// own derivations. Agreement with the anchored search is therefore evidence
    /// about the search rather than a tautology. It is exponential and restricted
    /// to tiny fixtures.
    ///
    /// The conditions, in the order the module documents them: every operation
    /// covered; every operation covered more than once admitted by `duplicable`;
    /// every named program output produced by exactly one region; and, when
    /// duplication is admitted, every region observed — producing a named output,
    /// or being the designated materializing owner of a value some region outside
    /// its owner set reads.
    fn oracle_covers(
        candidates: &[RegionCandidate],
        operation_count: u32,
        duplicable: &dyn Fn(u32) -> bool,
        admits_duplication: bool,
    ) -> BTreeSet<BTreeSet<Vec<u32>>> {
        let total = u32::try_from(candidates.len()).unwrap();
        assert!(
            total <= 20,
            "the oracle is restricted to tiny fixtures; {total} candidates is {} subsets",
            1_u64 << total
        );
        let named: BTreeSet<u32> = candidates
            .iter()
            .flat_map(RegionCandidate::retained_outputs)
            .filter(|output| output.named_result)
            .map(|output| output.value.0)
            .collect();
        let mut covers = BTreeSet::new();
        for mask in 1_u32..(1_u32 << total) {
            let chosen: Vec<&RegionCandidate> = (0..total)
                .filter(|bit| mask & (1_u32 << bit) != 0)
                .map(|bit| &candidates[bit as usize])
                .collect();
            let mut counts = vec![0_u32; operation_count as usize];
            for region in &chosen {
                for atom in region.members() {
                    counts[atom.member().0 as usize] += 1;
                }
            }
            if counts.contains(&0) {
                continue;
            }
            if counts.iter().enumerate().any(|(member, count)| {
                *count > 1 && (!admits_duplication || !u32::try_from(member).is_ok_and(&duplicable))
            }) {
                continue;
            }
            if named.iter().any(|value| {
                chosen
                    .iter()
                    .filter(|region| {
                        region
                            .retained_outputs()
                            .iter()
                            .any(|output| output.named_result && output.value.0 == *value)
                    })
                    .count()
                    != 1
            }) {
                continue;
            }
            if admits_duplication && !chosen.iter().all(|region| observed(region, &chosen)) {
                continue;
            }
            covers.insert(
                chosen
                    .iter()
                    .map(|region| {
                        region
                            .members()
                            .iter()
                            .map(|atom| atom.member().0)
                            .collect()
                    })
                    .collect(),
            );
        }
        covers
    }

    /// The oracle's own reading of the observability condition.
    ///
    /// Regions are identified by their member vectors, which are unique across
    /// candidates because region formation generates each connected set once.
    fn observed(region: &RegionCandidate, chosen: &[&RegionCandidate]) -> bool {
        if region
            .retained_outputs()
            .iter()
            .any(|output| output.named_result)
        {
            return true;
        }
        region.retained_outputs().iter().any(|output| {
            let mut owners: Vec<&&RegionCandidate> = chosen
                .iter()
                .filter(|other| {
                    other
                        .retained_outputs()
                        .iter()
                        .any(|theirs| theirs.value.0 == output.value.0)
                })
                .collect();
            let read_outside = chosen.iter().any(|other| {
                !owners
                    .iter()
                    .any(|owner| owner.members() == other.members())
                    && other
                        .boundary_inputs()
                        .iter()
                        .any(|input| input.0 == output.value.0)
            });
            if !read_outside {
                return false;
            }
            owners.sort_by(|left, right| {
                left.members()
                    .len()
                    .cmp(&right.members().len())
                    .then_with(|| {
                        left.occurrence()
                            .as_bytes()
                            .cmp(right.occurrence().as_bytes())
                    })
            });
            owners
                .first()
                .is_some_and(|owner| owner.members() == region.members())
        })
    }

    /// The candidates of one program, as the oracle consumes them.
    fn candidates_of(program: &SemanticProgram) -> Vec<RegionCandidate> {
        formation_of(program).candidates().to_vec()
    }

    /// The members whose duplication the legality condition admits.
    ///
    /// Re-derived here from the program rather than read out of the search: a
    /// member may be recomputed when it is pure and produces no named output,
    /// which the fixtures make checkable by hand.
    fn duplicable_members(program: &SemanticProgram) -> BTreeSet<u32> {
        let formation = formation_of(program);
        let named: BTreeSet<u32> = formation
            .candidates()
            .iter()
            .flat_map(RegionCandidate::retained_outputs)
            .filter(|output| output.named_result)
            .map(|output| output.producer.0)
            .collect();
        (0..formation.graph().operation_count())
            .filter(|member| {
                !named.contains(member)
                    && formation
                        .graph()
                        .member_operation_facts(SemanticMemberId(*member))
                        .unwrap()
                        .is_pure()
            })
            .collect()
    }

    #[test]
    fn enumeration_matches_the_exhaustive_partition_oracle() {
        for program in [
            serial_sum_program(),
            shared_constant_program(),
            diamond_program(),
            shared_producer_program(),
        ] {
            let enumeration = enumerate(&program);
            assert!(
                enumeration.is_exhaustive(),
                "the tiny fixtures must fit the governed cover budgets"
            );
            assert!(enumeration.is_coverable());
            let expected = oracle_covers(
                &candidates_of(&program),
                enumeration.operation_count(),
                &|_| false,
                false,
            );
            assert_eq!(
                cover_partitions(&enumeration),
                expected,
                "the anchored search lost or invented a partition"
            );
        }
    }

    /// The general-DAG condition: the same agreement under duplication.
    ///
    /// The admitted set now includes covers that compute one operation in
    /// several regions, and the oracle admits exactly the ones the stated
    /// legality condition admits. A search that duplicated something the
    /// condition refuses, or that missed a legal duplication, disagrees here.
    #[test]
    fn duplicating_enumeration_matches_the_exhaustive_cover_oracle() {
        for program in [
            shared_producer_program(),
            diamond_program(),
            shared_constant_program(),
        ] {
            let enumeration = enumerate_under(&program, with_duplication());
            assert!(
                enumeration.is_exhaustive(),
                "the tiny fixtures must fit the governed cover budgets"
            );
            let duplicable = duplicable_members(&program);
            let expected = oracle_covers(
                &candidates_of(&program),
                enumeration.operation_count(),
                &|member| duplicable.contains(&member),
                true,
            );
            assert_eq!(
                cover_partitions(&enumeration),
                expected,
                "the duplication-admitting search lost or invented a cover"
            );
            // Every admitted cover re-derives under the same contract, so the
            // agreement is over covers the verifier also accepts.
            for cover in enumeration.covers() {
                verify_cover(&program, &formation_of(&program), with_duplication(), cover).unwrap();
            }
        }
    }

    /// The oracle comparison can say no.
    ///
    /// A test that only ever asserts equality would pass just as happily against
    /// an oracle that returned the search's own answer. Perturbing the admitted
    /// set by one cover must make the comparison fail, in both directions.
    #[test]
    fn the_oracle_comparison_rejects_a_perturbed_admitted_set() {
        let program = shared_producer_program();
        let enumeration = enumerate_under(&program, with_duplication());
        let duplicable = duplicable_members(&program);
        let expected = oracle_covers(
            &candidates_of(&program),
            enumeration.operation_count(),
            &|member| duplicable.contains(&member),
            true,
        );
        let admitted = cover_partitions(&enumeration);
        assert_eq!(admitted, expected);

        let mut dropped = admitted.clone();
        let removed = dropped.iter().next().cloned().expect("a cover exists");
        dropped.remove(&removed);
        assert_ne!(dropped, expected, "a lost cover must be detected");

        let mut invented = admitted;
        invented.insert(BTreeSet::from([
            vec![0_u32],
            vec![1],
            vec![2],
            vec![3],
            vec![4],
        ]));
        assert_ne!(invented, expected, "an invented cover must be detected");
    }

    #[test]
    fn the_fused_and_fully_materialized_covers_are_both_retained() {
        let program = serial_sum_program();
        let enumeration = enumerate(&program);
        let operation_count = enumeration.operation_count();

        let materialized = enumeration
            .fully_materialized_cover()
            .expect("the all-singleton cover is retained unconditionally");
        assert_eq!(
            u32::try_from(materialized.region_count()).unwrap(),
            operation_count
        );
        // The fully-materialized cover materializes every internal value.
        assert!(!materialized.materializations().is_empty());
        assert!(materialized.duplication().is_none());

        let fused = enumeration
            .fused_cover()
            .expect("the connected program has a whole-program cover");
        assert_eq!(fused.region_count(), 1);
        // A single fused region crosses no boundary, so it materializes nothing.
        assert!(fused.materializations().is_empty());

        // Both are distinct legal covers.
        assert_ne!(materialized.identity(), fused.identity());
        verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            materialized,
        )
        .unwrap();
        verify_cover(&program, &formation_of(&program), exact_partition(), fused).unwrap();
    }

    /// Fan-out: one value, one edge, every consumer on it.
    ///
    /// The two failure modes the condition names are both asserted: the value is
    /// not duplicated into incomparable partitions (the exact-partition contract
    /// admits no duplication at all, and the edge carries both consumers rather
    /// than one), and the consumers are not serialized (they are two regions of
    /// one cover reading one materialized value, not a chain).
    #[test]
    fn fan_out_is_materialized_once_and_read_by_every_consumer() {
        let program = shared_producer_program();
        let enumeration = enumerate(&program);
        let materialized = enumeration.fully_materialized_cover().unwrap();

        let shared_edge = materialized
            .materializations()
            .iter()
            .find(|edge| edge.consumers().len() == 2)
            .expect("the shared producer fans out to two consumer regions");
        assert_eq!(shared_edge.consumers().len(), 2);
        assert_eq!(
            shared_edge.element_count(),
            6,
            "the shared value is the [2, 3] pointwise result"
        );
        // The two consumers are distinct regions of the same cover.
        assert_ne!(shared_edge.consumers()[0], shared_edge.consumers()[1]);
        for cover in enumeration.covers() {
            assert!(cover.duplication().is_none());
            assert!(cover.duplication().duplicated_positions().is_empty());
        }
    }

    #[test]
    fn every_operation_and_named_output_is_covered_by_each_cover() {
        for program in [
            serial_sum_program(),
            shared_producer_program(),
            diamond_program(),
        ] {
            let enumeration = enumerate(&program);
            let operation_count = enumeration.operation_count();
            let all_operations: BTreeSet<u32> = (0..operation_count).collect();
            for cover in enumeration.covers() {
                let covered: BTreeSet<u32> = cover
                    .regions()
                    .iter()
                    .flat_map(|region| region.members().iter().map(|atom| atom.member().0))
                    .collect();
                assert_eq!(
                    covered, all_operations,
                    "a cover left an operation uncovered"
                );
                // Re-derivation confirms every named output is retained too.
                verify_cover(&program, &formation_of(&program), exact_partition(), cover).unwrap();
            }
        }
    }

    /// Ordered multi-result outputs are planned as graph outputs, not one root.
    ///
    /// The two-output fixture is planned without either output being reduced
    /// away, and a cover naming fewer outputs than the program declares is
    /// rejected rather than accepted as a subset.
    #[test]
    fn multi_result_outputs_are_retained_and_a_dropped_one_is_rejected() {
        let program = shared_producer_program();
        let enumeration = enumerate(&program);
        assert_eq!(
            program.outputs().len(),
            2,
            "the fixture declares two ordered named outputs"
        );

        // Every cover retains both, and no cover is a single root region unless
        // that one region produces both.
        for cover in enumeration.covers() {
            verify_cover(&program, &formation_of(&program), exact_partition(), cover).unwrap();
        }

        // A cover naming fewer outputs than the program declares: drop the
        // region producing `right` and re-cover its operation with a region that
        // does not export it. No such candidate exists, so the drop is expressed
        // the only way it can be — by removing the region — and the coverage
        // check refuses it before the output check would.
        let mut dropped = enumeration.fully_materialized_cover().unwrap().clone();
        dropped.regions.pop();
        let error = verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            &dropped,
        )
        .unwrap_err();
        assert_eq!(error.class(), "coverage");

        // The direct form: a cover whose regions cover everything but whose
        // retained set is missing one named output is refused as a dropped
        // output rather than accepted as a subset.
        let formation = formation_of(&program);
        let named: BTreeSet<u32> = formation
            .candidates()
            .iter()
            .flat_map(RegionCandidate::retained_outputs)
            .filter(|output| output.named_result)
            .map(|output| output.value.0)
            .collect();
        assert_eq!(named.len(), 2, "both outputs are producer-backed");

        // Each cover states *which* named result each region retains, which is
        // what program assembly attributes the declared outputs by. Across a
        // cover the retained sets are disjoint and their union is exactly the
        // program's declared outputs — the projection of the same fact
        // `check_named_outputs` proves over the candidates.
        for cover in enumeration.covers() {
            let mut retained: Vec<u32> = cover
                .regions()
                .iter()
                .flat_map(|region| region.named_results().iter().map(|value| value.0))
                .collect();
            let distinct = retained.len();
            retained.sort_unstable();
            retained.dedup();
            assert_eq!(
                retained.len(),
                distinct,
                "a named output was retained twice"
            );
            assert_eq!(
                retained.into_iter().collect::<BTreeSet<u32>>(),
                named,
                "the cover's retained named results are not the program's outputs",
            );
        }

        // The projection is *checked* rather than trusted: a cover claiming a
        // region retains a named result its candidate does not fails closed at
        // the same step that binds members, content, and label.
        let mut forged = enumeration.fully_materialized_cover().unwrap().clone();
        let publishing = forged
            .regions
            .iter()
            .position(|region| !region.named_results.is_empty())
            .expect("some region retains a named output");
        forged.regions[publishing].named_results.clear();
        let error = verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            &forged,
        )
        .unwrap_err();
        assert_eq!(error.class(), "structure");
        assert_eq!(error.reason(), "region-occurrence-mismatch");
    }

    #[test]
    fn a_bare_input_passthrough_output_has_no_legal_cover() {
        let program = passthrough_output_program();
        let enumeration = enumerate(&program);
        assert!(enumeration.covers().is_empty());
        assert!(!enumeration.is_coverable());
        assert_eq!(
            enumeration.infeasibilities(),
            [CoverInfeasibility::UnrootedNamedOutput { count: 1 }]
        );
    }

    #[test]
    fn occurrence_identity_is_preserved_and_a_tampered_region_is_rejected() {
        let program = shared_producer_program();
        let enumeration = enumerate(&program);
        let outcome = formation_of(&program);
        let authoritative: BTreeSet<Vec<u8>> = outcome
            .candidates()
            .iter()
            .map(|candidate| candidate.occurrence().as_bytes().to_vec())
            .collect();
        for cover in enumeration.covers() {
            for region in cover.regions() {
                assert!(
                    authoritative.contains(region.occurrence().as_bytes()),
                    "a placed region is not an authoritative occurrence"
                );
            }
        }

        // Tampering a region's recorded occurrence label breaks occurrence identity.
        let mut forged = enumeration.fully_materialized_cover().unwrap().clone();
        forged.regions[0].label =
            std::sync::Arc::from(format!("{}-forged", forged.regions[0].label));
        let error = verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            &forged,
        )
        .unwrap_err();
        assert_eq!(error.class(), "structure");
        assert_eq!(error.reason(), "region-occurrence-mismatch");
    }

    #[test]
    fn a_cover_from_another_program_fails_occurrence_re_derivation() {
        // A structurally different program has different region occurrences, so a
        // cover's placed regions are not authoritative there and fail closed.
        let cover = enumerate(&serial_sum_program())
            .fully_materialized_cover()
            .unwrap()
            .clone();
        let error = verify_cover(
            &diamond_program(),
            &formation_of(&diamond_program()),
            exact_partition(),
            &cover,
        )
        .unwrap_err();
        assert_eq!(error.class(), "region");
        assert!(matches!(
            error,
            CoverError::Region(RegionError::Invalid { .. })
        ));
    }

    #[test]
    fn cover_identity_is_deterministic_and_independent_of_authoring_order() {
        // Two runs of one program agree.
        let program = serial_sum_program();
        let first = enumerate(&program);
        let second = enumerate(&program);
        let identities = |enumeration: &CoverEnumeration| -> Vec<Vec<u8>> {
            enumeration
                .covers()
                .iter()
                .map(|cover| cover.identity().as_bytes().to_vec())
                .collect()
        };
        assert_eq!(identities(&first), identities(&second));

        // Two programs with one canonical graph identity but opposite authoring
        // order enumerate the same cover identities.
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
        let forward = build(false);
        let reverse = build(true);
        assert_eq!(
            forward.semantic_identity().graph(),
            reverse.semantic_identity().graph()
        );
        assert_eq!(
            identities(&enumerate(&forward)),
            identities(&enumerate(&reverse))
        );

        // A duplicating enumeration is deterministic too, and its duplication is
        // bound into identity: the same regions with a member computed twice is
        // a different cover from the exact partition.
        let duplicating = enumerate_under(&program, with_duplication());
        assert_eq!(
            identities(&duplicating),
            identities(&enumerate_under(&program, with_duplication()))
        );
    }

    #[test]
    fn verify_cover_rejects_an_uncovered_member_and_illegal_duplication() {
        let program = shared_producer_program();
        let enumeration = enumerate(&program);

        // Dropping a region leaves its operation uncovered.
        let mut incomplete = enumeration.fully_materialized_cover().unwrap().clone();
        incomplete.regions.pop();
        let error = verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            &incomplete,
        )
        .unwrap_err();
        assert!(matches!(error, CoverError::UncoveredMember { .. }));
        assert_eq!(error.class(), "coverage");

        // Adding an overlapping authentic region double-covers an operation.
        let overlapping_pair = enumeration
            .covers()
            .iter()
            .flat_map(RegionCover::regions)
            .filter(|region| region.members().len() > 1)
            .cloned()
            .collect::<Vec<_>>();
        // {shared, left} and {shared, right} both cover the shared producer.
        let mut duplicated = enumeration.fully_materialized_cover().unwrap().clone();
        let doubled: Vec<_> = overlapping_pair
            .into_iter()
            .filter(|region| region.members().iter().any(|atom| atom.member().0 == 1))
            .collect();
        assert!(
            doubled.len() >= 2,
            "the shared producer has overlapping regions"
        );
        duplicated.regions.push(doubled[0].clone());
        let error = verify_cover(
            &program,
            &formation_of(&program),
            exact_partition(),
            &duplicated,
        )
        .unwrap_err();
        assert!(matches!(
            error,
            CoverError::IllegalDuplication {
                refusal: DuplicationRefusal::PolicyForbids,
                ..
            }
        ));
        assert_eq!(error.class(), "coverage");
    }

    /// Shared-work duplication is a candidate the search chooses, with its
    /// legality condition stated and checked.
    #[test]
    fn shared_work_duplication_is_a_candidate_the_search_chooses() {
        let program = shared_producer_program();
        let formation = formation_of(&program);
        let shared = SemanticMemberId(1);
        let shared_position = formation.graph().member_canonical_position(shared).unwrap();

        // Under the exact-partition contract no cover duplicates anything.
        for cover in enumerate(&program).covers() {
            assert!(cover.duplication().is_none());
        }

        // Under the duplication-admitting contract the search finds covers that
        // compute the shared producer in more than one region, and every one of
        // them names exactly that operation as duplicated.
        let duplicating = enumerate_under(&program, with_duplication());
        let chosen: Vec<&RegionCover> = duplicating
            .covers()
            .iter()
            .filter(|cover| !cover.duplication().is_none())
            .collect();
        assert!(
            !chosen.is_empty(),
            "the search must be able to choose a legal duplication"
        );
        let duplicable_positions: BTreeSet<u32> = duplicable_members(&program)
            .iter()
            .map(|member| {
                formation
                    .graph()
                    .member_canonical_position(SemanticMemberId(*member))
                    .unwrap()
            })
            .collect();
        assert!(
            duplicable_positions.contains(&shared_position),
            "the shared producer is pure and produces no named result"
        );
        let mut duplicated_anywhere: BTreeSet<u32> = BTreeSet::new();
        for cover in &chosen {
            for position in cover.duplication().duplicated_positions() {
                assert!(
                    duplicable_positions.contains(position),
                    "a cover duplicated an operation the legality condition refuses"
                );
                duplicated_anywhere.insert(*position);
            }
            assert!(cover.cost().recomputed_elements() > 0);
        }
        assert!(
            duplicated_anywhere.contains(&shared_position),
            "the fan-out producer is the duplication this fixture exists for"
        );

        // The exact-partition contract refuses exactly those covers, and it
        // refuses them by legality rather than by cost.
        let exact = cover_partitions(&enumerate(&program));
        for cover in &chosen {
            let members: BTreeSet<Vec<u32>> = cover
                .regions()
                .iter()
                .map(|region| {
                    region
                        .members()
                        .iter()
                        .map(|atom| atom.member().0)
                        .collect()
                })
                .collect();
            assert!(!exact.contains(&members));
        }
    }

    /// Each duplication refusal states its own condition.
    #[test]
    fn duplication_refusals_state_which_condition_refused_them() {
        let program = shared_producer_program();

        // A named result's producer may not be recomputed: two regions producing
        // it are two writers of one program result.
        let duplicating = enumerate_under(&program, with_duplication());
        let named_refusals: Vec<&CoverRefusal> = duplicating
            .refusals()
            .iter()
            .filter(|refusal| {
                matches!(
                    refusal,
                    CoverRefusal::Duplication {
                        refusal: DuplicationRefusal::NamedResultProducer,
                        ..
                    }
                )
            })
            .collect();
        assert!(
            !named_refusals.is_empty(),
            "the fixture's two named results are both refused for duplication"
        );
        for refusal in &named_refusals {
            assert_eq!(refusal.reason(), "duplication-named-result-producer");
            assert!(!refusal.subject_label().is_empty());
        }

        // A contract granting realization freedom refuses every member, because
        // two realizations of one occurrence could then differ.
        let relaxed = CoverPolicy::permitting_shared_work_duplication(
            StrictF32NumericalContract::governed_relaxed(),
        );
        let relaxed_enumeration = enumerate_under(&program, relaxed);
        assert!(
            relaxed_enumeration
                .covers()
                .iter()
                .all(|cover| cover.duplication().is_none()),
            "a contract that authorized a transform may not also authorize recomputation"
        );
        assert!(
            relaxed_enumeration
                .refusals()
                .iter()
                .any(|refusal| { refusal.reason() == "duplication-contract-grants-freedom" }),
            "the refusal must name the contract rather than leave the absence unexplained"
        );
    }

    /// A deliberate materialization can win on cost.
    ///
    /// The materializing cover and the partially-absorbing one place the same
    /// number of regions, cross the same number of boundaries, and move the same
    /// bytes; the second additionally recomputes the shared producer. So the
    /// first strictly dominates — which is the whole point of modelling the
    /// choice per edge rather than reading it off the partition's shape.
    #[test]
    fn a_deliberate_materialization_dominates_a_partial_recomputation() {
        let program = shared_producer_program();
        let enumeration = enumerate_under(&program, with_duplication());

        let materializing = find_cover(&enumeration, &[vec![0], vec![1], vec![2], vec![3]])
            .expect("the exact partition is enumerated");
        let partial = find_cover(&enumeration, &[vec![0], vec![1], vec![2], vec![1, 3]])
            .expect("absorbing one consumer is a legal cover");

        assert_eq!(
            materializing.cost().region_count(),
            partial.cost().region_count()
        );
        assert_eq!(
            materializing.cost().materialization_count(),
            partial.cost().materialization_count()
        );
        assert_eq!(
            materializing.cost().materialized_elements(),
            partial.cost().materialized_elements()
        );
        assert_eq!(materializing.cost().recomputed_elements(), 0);
        assert!(partial.cost().recomputed_elements() > 0);
        assert!(
            materializing.cost().dominates(&partial.cost()),
            "the deliberate materialization must win"
        );
        assert!(!partial.cost().dominates(&materializing.cost()));

        // The dominance view prunes the beaten cover and names what beat it,
        // while retention keeps both: legality never depends on cost.
        let pruned: Vec<Vec<u8>> = enumeration
            .dominated()
            .iter()
            .map(|(cover, _)| cover.identity().as_bytes().to_vec())
            .collect();
        assert!(pruned.contains(&partial.identity().as_bytes().to_vec()));
        assert!(
            enumeration
                .covers()
                .iter()
                .any(|cover| cover.identity() == partial.identity()),
            "a dominated cover is still retained"
        );

        // Absorbing *both* consumers is a different trade rather than a loss: it
        // materializes less and recomputes more, so neither dominates.
        let absorbing = find_cover(&enumeration, &[vec![0], vec![1, 2], vec![1, 3]])
            .expect("absorbing both consumers is a legal cover");
        assert!(!materializing.cost().dominates(&absorbing.cost()));
        assert!(!absorbing.cost().dominates(&materializing.cost()));

        // Every cover the view prunes is one another retained cover beats, and
        // the two views partition the retained set.
        let non_dominated: BTreeSet<Vec<u8>> = enumeration
            .non_dominated()
            .iter()
            .map(|cover| cover.identity().as_bytes().to_vec())
            .collect();
        let dominated: BTreeSet<Vec<u8>> = pruned.into_iter().collect();
        assert!(non_dominated.is_disjoint(&dominated));
        assert_eq!(
            non_dominated.len() + dominated.len(),
            enumeration.covers().len()
        );
    }

    /// A truncated explanation is not a truncated search.
    ///
    /// The refusal budget bounds how many distinct refusals are named, not how
    /// much of the space was explored, so it must not make the enumeration
    /// report itself partial. Driven over all three resources so the predicate
    /// is shown to distinguish them rather than to ignore budget stops.
    #[test]
    fn a_truncated_explanation_does_not_report_a_truncated_search() {
        let stopped = |resource| CoverEnumeration {
            policy: exact_partition(),
            covers: Vec::new(),
            refusals: Vec::new(),
            budget_stops: vec![super::CoverBudgetStop {
                resource,
                limit: 1,
                actual: 2,
            }],
            infeasibilities: Vec::new(),
            operation_count: 0,
            node_count: 0,
        };
        assert!(
            stopped(CoverBudgetResource::Refusals).is_exhaustive(),
            "a bounded refusal list must not be reported as a bounded search"
        );
        assert!(!stopped(CoverBudgetResource::Covers).is_exhaustive());
        assert!(!stopped(CoverBudgetResource::Expansions).is_exhaustive());
    }

    /// The two whole-cover checks can say no.
    ///
    /// Both are ordered behind a check that fires first on every input the
    /// search can build — the named-output check behind the duplication refusal
    /// for a named-result producer, and the verification-side observability
    /// check behind the enumerator never assembling a dead cover — so neither is
    /// reachable through `enumerate_covers`. A check nothing has shown can fail
    /// is a check a reader may not rely on, so both are driven directly against
    /// an input that must fail them.
    #[test]
    fn the_named_output_and_observability_checks_can_say_no() {
        use super::{NamedOutput, check_named_outputs, check_regions_observed};

        let program = shared_producer_program();
        let formation = formation_of(&program);
        let producer = formation
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .retained_outputs()
                    .iter()
                    .any(|output| output.named_result)
            })
            .expect("a candidate produces a named output");
        let named = producer
            .retained_outputs()
            .iter()
            .find(|output| output.named_result)
            .expect("the candidate's named output");
        let output = NamedOutput {
            position: 0,
            value: named.value.0,
        };

        // One producer passes; the same region counted twice is two writers of
        // one program result.
        check_named_outputs(&[output], &[producer]).unwrap();
        let error = check_named_outputs(&[output], &[producer, producer]).unwrap_err();
        assert_eq!(error.class(), "coverage");
        assert_eq!(error.reason(), "ambiguous-named-output");

        // A region producing a named output is observed with no edge at all; a
        // region producing none, with no edge naming it, is not.
        check_regions_observed(&[producer], &[]).unwrap();
        let internal = formation
            .candidates()
            .iter()
            .find(|candidate| {
                candidate
                    .retained_outputs()
                    .iter()
                    .all(|output| !output.named_result)
            })
            .expect("the fixture has a region producing no named output");
        let error = check_regions_observed(&[internal], &[]).unwrap_err();
        assert_eq!(error.class(), "coverage");
        assert_eq!(error.reason(), "dead-region");
    }

    /// A cover whose regions between them compute something nothing observes is
    /// refused, and the refusal names the region.
    #[test]
    fn a_cover_with_an_unobserved_region_is_refused_with_its_reason() {
        let program = shared_producer_program();
        let enumeration = enumerate_under(&program, with_duplication());
        // Absorbing both consumers *and* keeping a standalone copy of the shared
        // producer leaves that copy with no reader at all.
        assert!(
            find_cover(&enumeration, &[vec![0], vec![1], vec![1, 2], vec![1, 3]]).is_none(),
            "a redundant standalone copy is not a legal cover"
        );
        assert!(
            enumeration
                .refusals()
                .iter()
                .any(|refusal| refusal.reason() == "dead-region"),
            "the refusal must be stated rather than left as an absence"
        );
    }

    /// An exhausted budget yields an explainable partial result.
    #[test]
    fn cover_budget_stops_report_bounded_loss_and_keep_the_required_covers() {
        let program = serial_sum_program();
        let mut budgets = DeterministicBudgets::governed();
        budgets.region_covers = 1;
        let enumeration = enumerate_covers(
            &program,
            budgets,
            &formation_of(&program),
            exact_partition(),
        )
        .unwrap();

        // The unconditional fully-materialized and fused covers survive the bound.
        assert!(enumeration.fully_materialized_cover().is_some());
        assert!(enumeration.fused_cover().is_some());
        // The lost alternatives are reported as a typed budget stop, and the
        // result says of itself that it is partial.
        assert!(
            enumeration
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == CoverBudgetResource::Covers)
        );
        assert!(
            !enumeration.is_exhaustive(),
            "a budget-stopped search must not present itself as complete"
        );
        // Every cover it did retain is complete and legal, not truncated.
        for cover in enumeration.covers() {
            verify_cover(&program, &formation_of(&program), exact_partition(), cover).unwrap();
        }
        // The unbounded run says the opposite of itself, so the flag is not a
        // constant.
        assert!(enumerate(&program).is_exhaustive());
    }

    #[test]
    fn errors_report_their_exact_class_and_reason() {
        let region = CoverError::Region(RegionError::Structure {
            rule: "value-ordinal",
        });
        assert_eq!(region.class(), "region");
        assert_eq!(region.reason(), "value-ordinal");

        let uncovered = CoverError::UncoveredMember {
            member: SemanticMemberId(2),
        };
        assert_eq!(uncovered.class(), "coverage");
        assert_eq!(uncovered.reason(), "uncovered-member");
        assert!(
            uncovered
                .to_string()
                .contains("operation 2 is covered by no region")
        );

        let duplication = CoverError::IllegalDuplication {
            member: SemanticMemberId(3),
            refusal: DuplicationRefusal::ImpureMember,
        };
        assert_eq!(duplication.class(), "coverage");
        assert_eq!(duplication.reason(), "illegal-duplication");
        assert!(
            duplication
                .to_string()
                .contains("cover.duplication.duplication-impure-member")
        );

        let unobservable = CoverError::Unobservable {
            refusal: CoverRefusal::AmbiguousNamedOutput { output_position: 1 },
        };
        assert_eq!(unobservable.class(), "coverage");
        assert_eq!(unobservable.reason(), "ambiguous-named-output");

        let structure = CoverError::Structure {
            rule: "cover-identity-mismatch",
        };
        assert_eq!(structure.class(), "structure");
        assert_eq!(
            structure.to_string(),
            "cover.structure.cover-identity-mismatch"
        );
    }

    /// Finds the enumerated cover whose region member-sets exactly match.
    fn find_cover<'a>(
        enumeration: &'a CoverEnumeration,
        expected: &[Vec<u32>],
    ) -> Option<&'a RegionCover> {
        let want: BTreeSet<Vec<u32>> = expected.iter().cloned().collect();
        enumeration.covers().iter().find(|cover| {
            let have: BTreeSet<Vec<u32>> = cover
                .regions()
                .iter()
                .map(|region| {
                    region
                        .members()
                        .iter()
                        .map(|atom| atom.member().0)
                        .collect()
                })
                .collect();
            have == want
        })
    }

    /// Exercises the draft accessors so the surface is covered, not latently dead.
    #[test]
    fn draft_accessors_are_exercised() {
        let program = serial_sum_program();
        let enumeration = enumerate(&program);
        let cover = enumeration.fully_materialized_cover().unwrap();
        assert!(!cover.identity().label().is_empty());
        assert_eq!(enumeration.policy().key(), "cover.exact-partition.v1");
        assert_eq!(with_duplication().key(), "cover.pure-recomputation.v1");
        assert!(with_duplication().admits_duplication());
        assert!(!exact_partition().admits_duplication());
        assert_eq!(CoverBudgetResource::Covers.key(), "region-covers");
        assert_eq!(
            CoverBudgetResource::Expansions.key(),
            "region-cover-expansions"
        );
        assert_eq!(CoverBudgetResource::Refusals.key(), "region-cover-refusals");
        assert_eq!(
            CoverInfeasibility::UnrootedNamedOutput { count: 1 }.reason(),
            "unrooted-named-output"
        );
        assert_eq!(
            DuplicationRefusal::PolicyForbids.to_string(),
            "cover.duplication.duplication-policy-forbids"
        );
        for edge in cover.materializations() {
            let _ = edge.value();
            let _ = edge.producer_position();
            let _ = edge.result_position();
            let _ = edge.element_count();
            let _ = edge.producer();
        }
        for region in cover.regions() {
            let _ = region.content();
            let _ = region.label();
        }
        let cost = cover.cost();
        assert_eq!(cost.model_key(), "tiler.cost.partition-structural.v1");
        assert!(cost.region_count() > 0);
        assert!(cost.materialization_count() > 0);
        assert!(cost.materialized_elements() > 0);
        assert_eq!(cost.recomputed_elements(), 0);
        // Costs attributed to different models never dominate each other.
        let foreign = CoverCost {
            model_key: "tiler.cost.structural.v1",
            region_count: 1,
            materialization_count: 0,
            materialized_elements: 0,
            recomputed_elements: 0,
        };
        assert!(!foreign.dominates(&cost));
        assert!(!cost.dominates(&foreign));
        // The refusal channel and the enumeration's own book-keeping.
        let mut by_reason: BTreeMap<&str, usize> = BTreeMap::new();
        for refusal in enumeration.refusals() {
            *by_reason.entry(refusal.reason()).or_default() += 1;
        }
        assert!(by_reason.is_empty(), "an exact partition refuses no cover");
    }
}
