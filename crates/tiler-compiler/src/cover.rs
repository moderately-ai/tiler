//! Legal complete region-cover enumeration over a verified semantic DAG.
//!
//! Region formation proposes connected convex region candidates; this module
//! answers a distinct, strictly *global* question about them: which bounded sets
//! of region occurrences are legal complete covers of the whole semantic graph,
//! before any physical implementation is chosen. A cover is enumerated, not
//! selected: this stage chooses no implementation, schedules nothing, costs
//! nothing, and does not claim a complete executable program. It enumerates legal
//! partitionings only.
//!
//! The design keeps the concerns the correctness contract insists on separating:
//!
//! - **Complete coverage, failing closed.** A legal cover assigns every operation
//!   to exactly one region and retains every named program output. A cover that
//!   leaves an operation or named output uncovered, or that double-covers an
//!   operation without explicit legal duplication, is rejected with a typed
//!   [`CoverError`] rather than silently repaired.
//! - **Conservative fan-out materialization.** Producer duplication is disabled in
//!   this profile, so a legal cover is an exact partition: whenever a value
//!   produced in one region is read by another, it is a
//!   [`MaterializationEdge`] — materialized once and read across the boundary —
//!   never silently duplicated. Deliberate duplication is a reserved seam,
//!   recorded in [`CoverDuplication`] and bound into cover identity, and it is
//!   empty in this profile.
//! - **Both the fused and the fully-materialized cover are retained.** The
//!   fully-materialized (all-singleton) cover is emitted unconditionally, and the
//!   fused (whole-program) cover is emitted whenever region formation admitted a
//!   whole-program candidate. Neither can be lost to a budget; the budgets bound
//!   only the additional partitions the search discovers.
//! - **Deterministic, order-independent identity.** A [`RegionCoverIdentity`]
//!   folds the semantic graph meaning, the exact region occurrences (which bind
//!   both region content and per-region coverage), the deliberate duplication, and
//!   the proposed materialization edges, in a canonical length-prefixed byte
//!   encoding over content-derived coordinates. It excludes transient graph-local
//!   ordinals and never depends on `HashMap`/authoring order.
//!
//! Scope boundary: this authority is a *global* legality enumerator over region
//! candidates. Local physical frontiers ([`crate::frontier`]) are enumerated
//! independently and do not depend on a global cover; joining a complete cover
//! with compatible per-region frontiers is the later complete
//! physical-plan-selection authority. Every item here is a reviewed *draft*
//! boundary, not a stable compiler API, until Tom accepts the exact interface.

#![allow(
    dead_code,
    reason = "reviewed draft authority; cover enumeration is exercised by its own tests and is not yet wired into the private compile() facade, which the complete physical-plan-selection slice will do"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{SemanticProgram, ValueId};

use crate::region::{
    RegionCandidate, RegionContentIdentity, RegionError, RegionGraph, RegionOccurrenceIdentity,
    SemanticMemberId, SemanticValueId, form_region_candidates,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

/// Canonical domain-separation tag for one region-cover identity.
const COVER_IDENTITY_TAG: &[u8] = b"tiler.compiler.region-cover.v1\0";

/// Deterministic safety budgets that bound cover enumeration.
///
/// The fully-materialized and fused covers are retained unconditionally; these
/// budgets bound only the additional partitions the search discovers, so a legal
/// alternative lost to a bound is reported as a typed [`CoverBudgetStop`] rather
/// than silently dropped. The values are provisional safety limits, not
/// performance conclusions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CoverBudgets {
    /// Distinct covers retained for one enumeration request.
    pub(crate) covers: u32,
    /// Partition-search expansion attempts admitted for one enumeration request.
    pub(crate) expansions: u64,
}

impl CoverBudgets {
    /// The governed provisional budgets for the bounded profile.
    pub(crate) const fn governed() -> Self {
        Self {
            covers: 1024,
            expansions: 100_000,
        }
    }
}

/// A deterministic budget that bounds cover enumeration.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CoverBudgetResource {
    /// Distinct covers retained for one enumeration request.
    Covers,
    /// Partition-search expansion attempts for one enumeration request.
    Expansions,
}

impl CoverBudgetResource {
    /// Returns the stable resource key.
    pub(crate) const fn key(self) -> &'static str {
        match self {
            Self::Covers => "region-covers",
            Self::Expansions => "region-cover-expansions",
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

/// The deliberate producer duplication a cover realizes.
///
/// The first profile disables producer duplication, so a legal cover is an exact
/// partition and this is always empty. The type reserves the seam: a future
/// duplication-enabled profile records the deliberately duplicated occurrences
/// here, and cover identity binds them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverDuplication {
    duplicated: Vec<SemanticMemberId>,
}

impl CoverDuplication {
    /// Builds the empty (no deliberate duplication) policy of the first profile.
    const fn none() -> Self {
        Self {
            duplicated: Vec::new(),
        }
    }

    /// Returns the deliberately duplicated occurrences (empty in this profile).
    pub(crate) fn duplicated_members(&self) -> &[SemanticMemberId] {
        &self.duplicated
    }

    /// Returns whether the cover realizes no deliberate duplication.
    pub(crate) fn is_none(&self) -> bool {
        self.duplicated.is_empty()
    }
}

/// One value materialized across a region boundary within a cover.
///
/// Because producer duplication is disabled, a value produced inside one region
/// and read by others is materialized exactly once and read across the boundary.
/// The edge is expressed in content-derived canonical coordinates so it does not
/// depend on authoring order: the producing member's canonical position, the
/// producing region's occurrence identity, and the consuming regions' occurrence
/// identities. A value with several cross-region consumers is one edge with
/// several consumers — conservative fan-out materialization, not duplication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaterializationEdge {
    /// Graph-local ordinal of the materialized value (navigation only; excluded
    /// from identity because it is a transient authoring coordinate).
    value: SemanticValueId,
    /// Content-derived canonical position of the producing member.
    producer_position: u32,
    /// Result position of the materialized value on its producing member.
    result_position: u32,
    /// Occurrence identity of the region that produces (materializes) the value.
    producer: RegionOccurrenceIdentity,
    /// Occurrence identities of the regions that read the value, canonical ascending.
    consumers: Vec<RegionOccurrenceIdentity>,
}

impl MaterializationEdge {
    /// Returns the graph-local ordinal of the materialized value.
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

    /// Returns the occurrence identity of the producing region.
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
/// content identity, its graph-occurrence identity, and the bounded occurrence
/// label. The occurrence identity is what a cover binds and re-derives, so a
/// placed region cannot silently drift from the candidate region formation
/// admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CoverRegion {
    members: Vec<SemanticMemberId>,
    content: RegionContentIdentity,
    occurrence: RegionOccurrenceIdentity,
    stable_id: String,
}

impl CoverRegion {
    /// Returns the region's members in ascending graph-local order.
    pub(crate) fn members(&self) -> &[SemanticMemberId] {
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

    /// Returns the bounded explain label of the region occurrence.
    pub(crate) fn stable_id(&self) -> &str {
        &self.stable_id
    }
}

/// Collision-free, order-independent identity of one legal complete cover.
///
/// It folds the semantic graph meaning, the exact region occurrences (which bind
/// per-region content and coverage), the deliberate duplication, and the proposed
/// materialization edges, over content-derived canonical coordinates. Transient
/// graph-local ordinals and enumeration order are deliberately absent.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct RegionCoverIdentity(Vec<u8>);

impl RegionCoverIdentity {
    /// Returns the canonical identity bytes.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns a bounded explain label for this cover.
    ///
    /// The label is a digest of the canonical bytes and is presentation only.
    /// Equality decisions always use [`Self::as_bytes`].
    pub(crate) fn key(&self) -> String {
        format!("region-cover:{:016x}", digest(&self.0))
    }
}

/// One legal complete cover of the semantic region graph.
///
/// Every operation is covered by exactly one region and every named output is
/// retained. The regions are stored in canonical occurrence order, the
/// materialization edges in canonical order, and the deliberate duplication is
/// empty in this profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RegionCover {
    regions: Vec<CoverRegion>,
    materializations: Vec<MaterializationEdge>,
    duplication: CoverDuplication,
    identity: RegionCoverIdentity,
}

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
    covers: Vec<RegionCover>,
    budget_stops: Vec<CoverBudgetStop>,
    infeasibilities: Vec<CoverInfeasibility>,
    operation_count: u32,
}

impl CoverEnumeration {
    /// Returns every enumerated legal cover in canonical identity order.
    pub(crate) fn covers(&self) -> &[RegionCover] {
        &self.covers
    }

    /// Returns every budget that stopped a search path.
    pub(crate) fn budget_stops(&self) -> &[CoverBudgetStop] {
        &self.budget_stops
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

    /// Returns the fully-materialized (all-singleton) cover, always retained when
    /// the program is coverable.
    pub(crate) fn fully_materialized_cover(&self) -> Option<&RegionCover> {
        self.covers.iter().find(|cover| {
            u32::try_from(cover.regions.len()).is_ok_and(|count| count == self.operation_count)
                && cover.regions.iter().all(|region| region.members.len() == 1)
        })
    }

    /// Returns the fused (single whole-program region) cover, when region
    /// formation admitted a whole-program candidate.
    pub(crate) fn fused_cover(&self) -> Option<&RegionCover> {
        self.covers.iter().find(|cover| {
            cover.regions.len() == 1
                && u32::try_from(cover.regions[0].members.len())
                    .is_ok_and(|count| count == self.operation_count)
        })
    }
}

/// A cover that is not a legal complete cover, or invalid cover state.
///
/// The coverage variants classify why a proposed cover is not legal — an
/// uncovered operation, an operation double-covered without enabling duplication,
/// or an unretained named output. The structural variants are compiler faults: a
/// placed region that does not re-derive from the program (a broken occurrence
/// identity), or a cover whose recomputed materialization edges or identity do
/// not match. [`Self::class`] distinguishes the two, so malformed cover state is
/// never confused with a legal enumeration result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CoverError {
    /// A placed region failed re-derivation from the program.
    Region(RegionError),
    /// An operation is covered by no region.
    UncoveredMember {
        /// The uncovered operation.
        member: SemanticMemberId,
    },
    /// An operation is covered by more than one region without enabling duplication.
    IllegalDuplication {
        /// The double-covered operation.
        member: SemanticMemberId,
    },
    /// A named program output is retained by no region.
    UncoveredNamedOutput {
        /// A stable reason code for the uncovered output.
        reason: &'static str,
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
            | Self::UncoveredNamedOutput { .. } => "coverage",
            Self::Structure { .. } => "structure",
        }
    }

    /// Returns the stable reason code of the fault.
    pub(crate) const fn reason(&self) -> &'static str {
        match self {
            Self::Region(error) => error.reason(),
            Self::UncoveredMember { .. } => "uncovered-member",
            Self::IllegalDuplication { .. } => "illegal-duplication",
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
            Self::IllegalDuplication { member } => write!(
                formatter,
                "cover.coverage.illegal-duplication: operation {} is covered more than once",
                member.0
            ),
            Self::UncoveredNamedOutput { reason } => {
                write!(formatter, "cover.coverage.uncovered-named-output.{reason}")
            }
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
/// cover whenever a whole-program candidate exists; the remaining exact
/// partitions are enumerated by anchoring each region on the minimum uncovered
/// operation, bounded by [`CoverBudgets`]. A program whose named output is a bare
/// boundary-input passthrough has no legal cover in this profile and yields an
/// `Ok` result with an empty cover set and a recorded [`CoverInfeasibility`].
///
/// # Errors
///
/// Returns a [`CoverError`] when region formation fails or the enumeration
/// observes invalid compiler state (a missing singleton, a double-produced value,
/// or a candidate-index fault). A program with no legal cover is a successful
/// `Ok`, not an error.
pub(crate) fn enumerate_covers(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    cover_budgets: CoverBudgets,
) -> Result<CoverEnumeration, CoverError> {
    let outcome = form_region_candidates(program, budgets, contract)?;
    let graph = outcome.graph();
    let candidates = outcome.candidates();
    let operation_count = graph.operation_count();

    let infeasibilities = detect_infeasibilities(program);
    if !infeasibilities.is_empty() {
        return Ok(CoverEnumeration {
            covers: Vec::new(),
            budget_stops: Vec::new(),
            infeasibilities,
            operation_count,
        });
    }

    let graph_identity = program.semantic_identity().graph().as_bytes().to_vec();

    // Candidate indices containing each operation, in the candidates' canonical
    // order, so the anchored partition search is deterministic.
    let mut containing: Vec<Vec<usize>> =
        vec![Vec::new(); usize::try_from(operation_count).unwrap_or(usize::MAX)];
    for (index, candidate) in candidates.iter().enumerate() {
        for member in candidate.members() {
            let slot = containing
                .get_mut(member_index(*member))
                .ok_or(CoverError::Structure {
                    rule: "member-ordinal",
                })?;
            slot.push(index);
        }
    }

    let mut retained: BTreeMap<RegionCoverIdentity, RegionCover> = BTreeMap::new();

    // The fully-materialized (all-singleton) cover is retained unconditionally.
    let singletons = collect_singletons(candidates, operation_count)?;
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
        max_covers: usize::try_from(cover_budgets.covers).unwrap_or(usize::MAX),
        max_expansions: cover_budgets.expansions,
        retained,
        stops: BTreeMap::new(),
        expansions: 0,
    };
    partitioner.run(operation_count)?;

    let mut covers: Vec<RegionCover> = partitioner.retained.into_values().collect();
    covers.sort_by(|left, right| left.identity.as_bytes().cmp(right.identity.as_bytes()));
    Ok(CoverEnumeration {
        covers,
        budget_stops: partitioner.stops.into_values().collect(),
        infeasibilities: Vec::new(),
        operation_count,
    })
}

/// Re-derives and validates one proposed cover of a program.
///
/// The program's candidates are re-derived by region formation; each placed
/// region must be one of those authoritative occurrences (preserving occurrence
/// identity), the operations must be covered exactly once (no uncovered member,
/// no illegal duplication), every producer-backed named output must be retained,
/// and the recomputed materialization edges and cover identity must match. Any
/// deviation fails closed with a typed [`CoverError`].
///
/// # Errors
///
/// Returns a [`CoverError`] whose [`CoverError::class`] is `region` for a broken
/// occurrence identity, `coverage` for an uncovered or double-covered operation
/// or an unretained named output, and `structure` for a mismatched
/// materialization or identity.
pub(crate) fn verify_cover(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    contract: StrictF32NumericalContract,
    cover: &RegionCover,
) -> Result<(), CoverError> {
    let outcome = form_region_candidates(program, budgets, contract)?;
    let graph = outcome.graph();
    let operation_count = graph.operation_count();

    let mut authoritative: BTreeMap<RegionOccurrenceIdentity, &RegionCandidate> = BTreeMap::new();
    for candidate in outcome.candidates() {
        authoritative.insert(candidate.occurrence().clone(), candidate);
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
                    region: region.stable_id.clone(),
                    rule: "unknown-region-occurrence",
                }))?;
        if region.members.as_slice() != candidate.members()
            || &region.content != candidate.content()
            || region.stable_id != candidate.stable_id()
        {
            return Err(CoverError::Structure {
                rule: "region-occurrence-mismatch",
            });
        }
        resolved.push(candidate);
    }

    // 2. Coverage multiset: every operation covered exactly once.
    let mut counts = vec![0_u32; usize::try_from(operation_count).unwrap_or(usize::MAX)];
    for candidate in &resolved {
        for member in candidate.members() {
            let slot = counts
                .get_mut(member_index(*member))
                .ok_or(CoverError::Structure {
                    rule: "member-ordinal",
                })?;
            *slot = slot.saturating_add(1);
        }
    }
    for (position, count) in counts.iter().enumerate() {
        let member = SemanticMemberId(u32::try_from(position).unwrap_or(u32::MAX));
        match count {
            0 => return Err(CoverError::UncoveredMember { member }),
            1 => {}
            _ => return Err(CoverError::IllegalDuplication { member }),
        }
    }

    // 3. Named-output coverage: no bare-input passthrough, every producer-backed
    //    named output retained.
    if !detect_infeasibilities(program).is_empty() {
        return Err(CoverError::UncoveredNamedOutput {
            reason: "unrooted-named-output",
        });
    }
    let required = named_output_values(outcome.candidates());
    let retained = named_output_values(resolved.iter().copied());
    if !required.is_subset(&retained) {
        return Err(CoverError::UncoveredNamedOutput {
            reason: "dropped-named-output",
        });
    }

    // 4. Materialization edges recompute exactly.
    let materializations = derive_materializations(graph, &resolved)?;
    if materializations != cover.materializations {
        return Err(CoverError::Structure {
            rule: "materialization-mismatch",
        });
    }

    // 5. Deliberate duplication is empty in this profile.
    if !cover.duplication.is_none() {
        return Err(CoverError::Structure {
            rule: "unexpected-duplication",
        });
    }

    // 6. Canonical region order and cover identity recompute exactly.
    let mut regions = cover.regions.clone();
    regions.sort_by(|left, right| left.occurrence.as_bytes().cmp(right.occurrence.as_bytes()));
    if regions != cover.regions {
        return Err(CoverError::Structure {
            rule: "region-order",
        });
    }
    let graph_identity = program.semantic_identity().graph().as_bytes();
    let identity = encode_cover_identity(
        graph_identity,
        &regions,
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

/// The deterministic anchored exact-partition search state.
struct Partitioner<'a> {
    graph: &'a RegionGraph,
    candidates: &'a [RegionCandidate],
    containing: &'a [Vec<usize>],
    graph_identity: &'a [u8],
    max_covers: usize,
    max_expansions: u64,
    retained: BTreeMap<RegionCoverIdentity, RegionCover>,
    stops: BTreeMap<CoverBudgetResource, CoverBudgetStop>,
    expansions: u64,
}

/// Whether a search branch should continue or the whole search must stop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Flow {
    Continue,
    Stop,
}

impl Partitioner<'_> {
    fn run(&mut self, operation_count: u32) -> Result<(), CoverError> {
        let uncovered: BTreeSet<u32> = (0..operation_count).collect();
        let mut chosen: Vec<usize> = Vec::new();
        self.visit(&uncovered, &mut chosen)?;
        Ok(())
    }

    /// Extends `chosen` with every region anchored on the minimum uncovered
    /// operation, so each exact partition is generated exactly once.
    fn visit(
        &mut self,
        uncovered: &BTreeSet<u32>,
        chosen: &mut Vec<usize>,
    ) -> Result<Flow, CoverError> {
        let Some(&anchor) = uncovered.iter().next() else {
            return self.emit(chosen);
        };
        let anchored = self
            .containing
            .get(usize::try_from(anchor).unwrap_or(usize::MAX))
            .ok_or(CoverError::Structure {
                rule: "anchor-ordinal",
            })?
            .clone();
        for index in anchored {
            self.expansions = self.expansions.saturating_add(1);
            if self.expansions > self.max_expansions {
                self.record_stop(
                    CoverBudgetResource::Expansions,
                    self.max_expansions,
                    self.expansions,
                );
                return Ok(Flow::Stop);
            }
            let members: Vec<u32> = match self.candidates.get(index) {
                Some(candidate) => candidate.members().iter().map(|member| member.0).collect(),
                None => {
                    return Err(CoverError::Structure {
                        rule: "candidate-index",
                    });
                }
            };
            if members.iter().all(|member| uncovered.contains(member)) {
                let mut next = uncovered.clone();
                for member in &members {
                    next.remove(member);
                }
                chosen.push(index);
                let flow = self.visit(&next, chosen)?;
                chosen.pop();
                if flow == Flow::Stop {
                    return Ok(Flow::Stop);
                }
            }
        }
        Ok(Flow::Continue)
    }

    fn emit(&mut self, chosen: &[usize]) -> Result<Flow, CoverError> {
        let cover = assemble_cover(self.graph, self.candidates, self.graph_identity, chosen)?;
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

    fn record_stop(&mut self, resource: CoverBudgetResource, limit: u64, actual: u64) {
        let stop = self.stops.entry(resource).or_insert(CoverBudgetStop {
            resource,
            limit,
            actual,
        });
        stop.actual = stop.actual.max(actual);
    }
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
    let duplication = CoverDuplication::none();
    let mut regions: Vec<CoverRegion> = resolved
        .iter()
        .map(|candidate| CoverRegion {
            members: candidate.members().to_vec(),
            content: candidate.content().clone(),
            occurrence: candidate.occurrence().clone(),
            stable_id: candidate.stable_id().to_owned(),
        })
        .collect();
    regions.sort_by(|left, right| left.occurrence.as_bytes().cmp(right.occurrence.as_bytes()));
    let identity = encode_cover_identity(graph_identity, &regions, &duplication, &materializations);
    Ok(RegionCover {
        regions,
        materializations,
        duplication,
        identity,
    })
}

/// Derives the cross-region materialization edges of one complete partition.
///
/// Each retained value is mapped to its unique producing region; a value another
/// region reads across the boundary becomes one edge, carrying every consuming
/// region. A value produced by two regions, or read as a boundary input by its
/// own producer, is invalid partition state and fails closed.
fn derive_materializations(
    graph: &RegionGraph,
    regions: &[&RegionCandidate],
) -> Result<Vec<MaterializationEdge>, CoverError> {
    let mut producers: BTreeMap<u32, (usize, SemanticMemberId, u32)> = BTreeMap::new();
    for (region_index, region) in regions.iter().enumerate() {
        for output in region.retained_outputs() {
            if producers
                .insert(
                    output.value.0,
                    (region_index, output.producer, output.result_position),
                )
                .is_some()
            {
                return Err(CoverError::Structure {
                    rule: "double-produced-value",
                });
            }
        }
    }

    let mut consumers: BTreeMap<u32, BTreeSet<usize>> = BTreeMap::new();
    for (region_index, region) in regions.iter().enumerate() {
        for input in region.boundary_inputs() {
            if let Some(&(producer_region, _, _)) = producers.get(&input.0) {
                if producer_region == region_index {
                    return Err(CoverError::Structure {
                        rule: "internal-boundary-input",
                    });
                }
                consumers.entry(input.0).or_default().insert(region_index);
            }
        }
    }

    let mut edges: Vec<MaterializationEdge> = Vec::new();
    for (value, (producer_region, producer_member, result_position)) in &producers {
        let Some(consumer_regions) = consumers.get(value) else {
            continue;
        };
        let producer_position = graph.member_canonical_position(*producer_member)?;
        let mut consumer_occurrences: Vec<RegionOccurrenceIdentity> = consumer_regions
            .iter()
            .map(|&region_index| regions[region_index].occurrence().clone())
            .collect();
        consumer_occurrences.sort();
        edges.push(MaterializationEdge {
            value: SemanticValueId(*value),
            producer_position,
            result_position: *result_position,
            producer: regions[*producer_region].occurrence().clone(),
            consumers: consumer_occurrences,
        });
    }
    edges.sort_by(|left, right| {
        let mut left_key = Vec::new();
        encode_materialization(&mut left_key, left);
        let mut right_key = Vec::new();
        encode_materialization(&mut right_key, right);
        left_key.cmp(&right_key)
    });
    Ok(edges)
}

/// Collects the singleton candidate covering each operation exactly once.
fn collect_singletons(
    candidates: &[RegionCandidate],
    operation_count: u32,
) -> Result<Vec<usize>, CoverError> {
    let mut by_member: BTreeMap<u32, usize> = BTreeMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if let [member] = candidate.members()
            && by_member.insert(member.0, index).is_some()
        {
            return Err(CoverError::Structure {
                rule: "duplicate-singleton",
            });
        }
    }
    let mut singletons = Vec::with_capacity(usize::try_from(operation_count).unwrap_or(usize::MAX));
    for member in 0..operation_count {
        let index = by_member
            .get(&member)
            .copied()
            .ok_or(CoverError::Structure {
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

/// Returns the graph-local ordinals of the named outputs the regions retain.
fn named_output_values<'a>(
    regions: impl IntoIterator<Item = &'a RegionCandidate>,
) -> BTreeSet<u32> {
    let mut named = BTreeSet::new();
    for region in regions {
        for output in region.retained_outputs() {
            if output.named_result {
                named.insert(output.value.0);
            }
        }
    }
    named
}

fn encode_cover_identity(
    graph_identity: &[u8],
    regions: &[CoverRegion],
    duplication: &CoverDuplication,
    materializations: &[MaterializationEdge],
) -> RegionCoverIdentity {
    let mut bytes = COVER_IDENTITY_TAG.to_vec();
    encode_bytes(&mut bytes, graph_identity);
    encode_len(&mut bytes, regions.len());
    for region in regions {
        encode_bytes(&mut bytes, region.occurrence.as_bytes());
    }
    // Producer duplication is disabled in this profile, so no member positions are
    // emitted. A future duplication-enabled profile must encode canonical
    // positions here rather than transient graph-local ordinals.
    encode_len(&mut bytes, duplication.duplicated.len());
    debug_assert!(duplication.duplicated.is_empty());
    encode_len(&mut bytes, materializations.len());
    for edge in materializations {
        encode_materialization(&mut bytes, edge);
    }
    RegionCoverIdentity(bytes)
}

fn encode_materialization(output: &mut Vec<u8>, edge: &MaterializationEdge) {
    output.extend_from_slice(&edge.producer_position.to_be_bytes());
    output.extend_from_slice(&edge.result_position.to_be_bytes());
    encode_bytes(output, edge.producer.as_bytes());
    encode_len(output, edge.consumers.len());
    for consumer in &edge.consumers {
        encode_bytes(output, consumer.as_bytes());
    }
}

fn member_index(member: SemanticMemberId) -> usize {
    usize::try_from(member.0).unwrap_or(usize::MAX)
}

fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) {
    encode_len(output, value.len());
    output.extend_from_slice(value);
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&count(value).to_be_bytes());
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
        CoverBudgetResource, CoverBudgets, CoverEnumeration, CoverError, CoverInfeasibility,
        enumerate_covers, verify_cover,
    };
    use crate::region::{RegionError, SemanticMemberId, form_region_candidates};
    use crate::request::{DeterministicBudgets, StrictF32NumericalContract};
    use std::collections::BTreeSet;
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};

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
        enumerate_covers(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            CoverBudgets::governed(),
        )
        .unwrap()
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
                    .map(|region| region.members().iter().map(|member| member.0).collect())
                    .collect()
            })
            .collect()
    }

    /// An independent exhaustive oracle over every subset of candidate regions.
    ///
    /// It brute-forces the powerset and keeps the subsets that cover every
    /// operation exactly once, so agreement with the anchored search is evidence
    /// rather than a tautology. It is exponential and restricted to tiny fixtures.
    fn oracle_partitions(
        candidates: &[Vec<u32>],
        operation_count: u32,
    ) -> BTreeSet<BTreeSet<Vec<u32>>> {
        let total = u32::try_from(candidates.len()).unwrap();
        assert!(total <= 20, "the oracle is restricted to tiny fixtures");
        let mut partitions = BTreeSet::new();
        for mask in 1_u32..(1_u32 << total) {
            let chosen: Vec<&Vec<u32>> = (0..total)
                .filter(|bit| mask & (1_u32 << bit) != 0)
                .map(|bit| &candidates[bit as usize])
                .collect();
            let mut counts = vec![0_u32; operation_count as usize];
            for region in &chosen {
                for &member in *region {
                    counts[member as usize] += 1;
                }
            }
            if counts.iter().all(|&count| count == 1) {
                partitions.insert(chosen.iter().map(|region| (*region).clone()).collect());
            }
        }
        partitions
    }

    fn candidate_member_sets(program: &SemanticProgram) -> Vec<Vec<u32>> {
        form_region_candidates(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap()
        .candidates()
        .iter()
        .map(|candidate| candidate.members().iter().map(|member| member.0).collect())
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
                enumeration.budget_stops().is_empty(),
                "the tiny fixtures must fit the governed cover budgets"
            );
            assert!(enumeration.is_coverable());
            let expected = oracle_partitions(
                &candidate_member_sets(&program),
                enumeration.operation_count(),
            );
            assert_eq!(
                cover_partitions(&enumeration),
                expected,
                "the anchored search lost or invented a partition"
            );
        }
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
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            materialized,
        )
        .unwrap();
        verify_cover(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            fused,
        )
        .unwrap();
    }

    #[test]
    fn fan_out_is_conservatively_materialized_across_regions() {
        // The shared producer feeds two consumer regions. With duplication
        // disabled its value is materialized once and read by both — one edge with
        // two consumers — never duplicated into either consumer.
        let program = shared_producer_program();
        let enumeration = enumerate(&program);
        let materialized = enumeration.fully_materialized_cover().unwrap();

        let shared_edge = materialized
            .materializations()
            .iter()
            .find(|edge| edge.consumers().len() == 2)
            .expect("the shared producer fans out to two consumer regions");
        assert_eq!(shared_edge.consumers().len(), 2);
        // No cover in the profile realizes deliberate duplication.
        for cover in enumeration.covers() {
            assert!(cover.duplication().is_none());
            assert!(cover.duplication().duplicated_members().is_empty());
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
                    .flat_map(|region| region.members().iter().map(|member| member.0))
                    .collect();
                assert_eq!(
                    covered, all_operations,
                    "a cover left an operation uncovered"
                );
                // Re-derivation confirms every named output is retained too.
                verify_cover(
                    &program,
                    DeterministicBudgets::governed(),
                    StrictF32NumericalContract::governed(),
                    cover,
                )
                .unwrap();
            }
        }
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
        let outcome = form_region_candidates(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap();
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

        // Tampering a region's recorded stable label breaks occurrence identity.
        let mut forged = enumeration.fully_materialized_cover().unwrap().clone();
        forged.regions[0].stable_id.push_str("-forged");
        let error = verify_cover(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
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
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
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
    }

    #[test]
    fn verify_cover_rejects_an_uncovered_member_and_illegal_duplication() {
        let program = shared_producer_program();
        let enumeration = enumerate(&program);
        let budgets = DeterministicBudgets::governed();
        let contract = StrictF32NumericalContract::governed();

        // Dropping a region leaves its operation uncovered.
        let mut incomplete = enumeration.fully_materialized_cover().unwrap().clone();
        incomplete.regions.pop();
        let error = verify_cover(&program, budgets, contract, &incomplete).unwrap_err();
        assert!(matches!(error, CoverError::UncoveredMember { .. }));
        assert_eq!(error.class(), "coverage");

        // Adding an overlapping authentic region double-covers an operation.
        let overlapping_pair = enumeration
            .covers()
            .iter()
            .flat_map(super::RegionCover::regions)
            .filter(|region| region.members().len() > 1)
            .cloned()
            .collect::<Vec<_>>();
        // {shared, left} and {shared, right} both cover the shared producer.
        let mut duplicated = enumeration.fully_materialized_cover().unwrap().clone();
        let doubled: Vec<_> = overlapping_pair
            .into_iter()
            .filter(|region| region.members().iter().any(|member| member.0 == 1))
            .collect();
        assert!(
            doubled.len() >= 2,
            "the shared producer has overlapping regions"
        );
        duplicated.regions.push(doubled[0].clone());
        let error = verify_cover(&program, budgets, contract, &duplicated).unwrap_err();
        assert!(matches!(error, CoverError::IllegalDuplication { .. }));
        assert_eq!(error.class(), "coverage");
    }

    #[test]
    fn cover_budget_stops_report_bounded_loss_and_keep_the_required_covers() {
        let program = serial_sum_program();
        let mut budgets = CoverBudgets::governed();
        budgets.covers = 1;
        let enumeration = enumerate_covers(
            &program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
            budgets,
        )
        .unwrap();

        // The unconditional fully-materialized and fused covers survive the bound.
        assert!(enumeration.fully_materialized_cover().is_some());
        assert!(enumeration.fused_cover().is_some());
        // The lost alternatives are reported as a typed budget stop.
        assert!(
            enumeration
                .budget_stops()
                .iter()
                .any(|stop| stop.resource == CoverBudgetResource::Covers)
        );
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

        let structure = CoverError::Structure {
            rule: "cover-identity-mismatch",
        };
        assert_eq!(structure.class(), "structure");
        assert_eq!(
            structure.to_string(),
            "cover.structure.cover-identity-mismatch"
        );
    }

    /// Exercises the draft accessors so the surface is covered, not latently dead.
    #[test]
    fn draft_accessors_are_exercised() {
        let program = serial_sum_program();
        let enumeration = enumerate(&program);
        let cover = enumeration.fully_materialized_cover().unwrap();
        assert!(!cover.identity().key().is_empty());
        assert_eq!(CoverBudgetResource::Covers.key(), "region-covers");
        assert_eq!(
            CoverBudgetResource::Expansions.key(),
            "region-cover-expansions"
        );
        assert_eq!(
            CoverInfeasibility::UnrootedNamedOutput { count: 1 }.reason(),
            "unrooted-named-output"
        );
        for edge in cover.materializations() {
            let _ = edge.value();
            let _ = edge.producer_position();
            let _ = edge.result_position();
            let _ = edge.producer();
        }
        for region in cover.regions() {
            let _ = region.content();
            let _ = region.stable_id();
        }
    }
}
