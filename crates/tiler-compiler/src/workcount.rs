//! Counters that pin how often a derivation runs, for the tests that guard it.
//!
//! # Why a count is a test and not a benchmark
//!
//! Most of this crate's cost is a pure function of immutable inputs being
//! recomputed inside a loop. A timing test cannot guard against that coming
//! back: timings move with the host, the profile, and the weather, so a
//! regression hides inside the noise until somebody profiles again. A *count*
//! does not move. "One compile derives the region formation once" is a
//! statement about structure, it costs microseconds to check, and it fails
//! loudly the first time a new call site reintroduces the derivation.
//!
//! So these exist to keep an optimization from eroding, not to measure it. The
//! measurement lives beside them and prints; the count asserts.
//!
//! # Thread-local, and why that matters
//!
//! `cargo nextest` runs each test in its own process, but `cargo test` runs
//! them as threads in one, and this crate is exercised both ways. A global
//! counter would make one test's work visible to another and the guards would
//! fail depending on which runner was used. A thread-local counts only the work
//! the observing thread caused.

use std::cell::{Cell, RefCell};

/// One named counter.
pub(crate) struct WorkCounter {
    slot: &'static std::thread::LocalKey<Cell<usize>>,
    name: &'static str,
}

impl WorkCounter {
    /// Names a counter over one thread-local slot.
    pub(crate) const fn new(
        name: &'static str,
        slot: &'static std::thread::LocalKey<Cell<usize>>,
    ) -> Self {
        Self { slot, name }
    }

    /// Records one occurrence of the work this counter names.
    pub(crate) fn record(&self) {
        self.slot.with(|count| count.set(count.get() + 1));
    }

    /// Runs `work` with the counter zeroed and returns its result beside the
    /// number of occurrences it caused.
    ///
    /// Zeroing inside rather than asking the caller to do it is what stops a
    /// guard from accidentally measuring a previous test's leftovers.
    pub(crate) fn observe<T>(&self, work: impl FnOnce() -> T) -> (T, usize) {
        self.slot.with(|count| count.set(0));
        let value = work();
        (value, self.slot.with(Cell::get))
    }

    /// The name this counter reports under.
    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }
}

thread_local! {
    static REQUEST_SUBJECT: Cell<usize> = const { Cell::new(0) };
}

/// Counts full reconstructions of the verified request subject.
///
/// Each one deep-clones the semantic identity, both shapes, the recognized
/// members, and the contract preference. `store-the-verified-request-subject-instead-of-rebuilding-it`
/// is the ticket that reduces it; this is what proves it stays reduced.
pub(crate) static REQUEST_SUBJECT_REBUILDS: WorkCounter =
    WorkCounter::new("request-subject rebuild", &REQUEST_SUBJECT);

thread_local! {
    static REGION_FORMATION: Cell<usize> = const { Cell::new(0) };
}

/// Counts full region-formation derivations.
///
/// Each one runs a whole-program canonicalisation and a growth search bounded by
/// `region_expansions` — ten thousand candidate formations. It is a pure
/// function of the program, budgets, and contract, all fixed for a target
/// compile, so more than one per target is duplicated work by definition.
pub(crate) static REGION_FORMATIONS: WorkCounter =
    WorkCounter::new("region formation", &REGION_FORMATION);

thread_local! {
    static REGION_CANDIDATE_FORMATION: Cell<usize> = const { Cell::new(0) };
}

/// Counts classifications of one node set into a candidate or a rejection.
///
/// **The denominator a per-candidate cost claim needs, and the only honest
/// one.** `form_candidate` is what a region-shape budget decides against, and
/// most of the sets it is handed are refused rather than emitted — so dividing
/// a formation's elapsed time by the *emitted* candidate count prices the
/// rejected sets into the survivors and reports a number that moves with the
/// convexity of the program rather than with the cost of a check.
///
/// `derive-the-region-shape-budgets-from-the-declaration` is what asked for it:
/// widening a shape bound admits larger node sets, and whether that raises the
/// price of one check was the risk the deciding ticket named to measure.
pub(crate) static REGION_CANDIDATE_FORMATIONS: WorkCounter =
    WorkCounter::new("region candidate formation", &REGION_CANDIDATE_FORMATION);

thread_local! {
    static REGION_GRAPH_BUILD: Cell<usize> = const { Cell::new(0) };
}

/// Counts constructions of the whole-program region graph.
///
/// This is the counter a profiler asked for rather than a reading of the code.
/// `RegionGraph::from_program` ends by running `canonical_member_order` over
/// every operation in the program — a colour refinement that rebuilds and
/// re-digests a byte buffer per member per round, so it is quadratic in the
/// program and allocation-heavy. A sampling profile of one compile attributed
/// **10.6% of active self time to `canonical_member_order` alone**, above every
/// other function in the crate, with the allocator and `memmove` traffic it
/// generates on top of that.
///
/// The graph is a pure function of the program, and `RegionFormationOutcome`
/// already owns one. More than one construction per compile is therefore a call
/// site rebuilding a value it could have been handed.
pub(crate) static REGION_GRAPH_BUILDS: WorkCounter =
    WorkCounter::new("region-graph build", &REGION_GRAPH_BUILD);

thread_local! {
    static FRONTIER_ENUMERATION: Cell<usize> = const { Cell::new(0) };
}

/// Counts implementation-frontier enumerations.
///
/// `enumerate_frontier` is a pure function of the verified request, the region
/// subject, and the provider set; only the subject varies within a target
/// compile. Every cover that places a given region asked for that region's
/// frontier again, so the count was the number of (cover, region) pairs rather
/// than the number of distinct regions.
///
/// The bound this guards is the *distinct subject* count, so it also pins the
/// memo's key. Keying on the presentation role instead would look like a much
/// better ratio and be wrong: most distinct subjects in the governed program
/// share the role `unrecognized` while covering different occurrences, and the
/// members are what each proposal's request-subject binding is checked against.
pub(crate) static FRONTIER_ENUMERATIONS: WorkCounter =
    WorkCounter::new("frontier enumeration", &FRONTIER_ENUMERATION);

/// Exact physical-frontier and plan-selection work caused by one test subject.
///
/// This is a test-only structural census, not a production metric. It records
/// the independent populations a raw provider outcome can enter so calibration
/// does not equate an emission with verification, retention, sorting, or a
/// complete-plan combination.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PhysicalPlanningCensus {
    pub(crate) provider_invocations: u64,
    pub(crate) proposals: u64,
    pub(crate) declines: u64,
    pub(crate) admission_assessments: u64,
    pub(crate) schedule_verifications: u64,
    pub(crate) admitted_implementations: u64,
    pub(crate) retained_implementations: u64,
    pub(crate) proposal_rejections: u64,
    pub(crate) frontier_rejections: u64,
    pub(crate) admitted_sort_items: u64,
    pub(crate) rejection_sort_items: u64,
    pub(crate) plan_combinations: u64,
    pub(crate) accepted_plan_combinations: u64,
    pub(crate) rejected_plan_combinations: u64,
    pub(crate) retained_complete_plans: u64,
    pub(crate) complete_plan_sort_items: u64,
}

thread_local! {
    static PHYSICAL_PLANNING_CENSUS: RefCell<PhysicalPlanningCensus> =
        RefCell::new(PhysicalPlanningCensus::default());
}

/// Runs `work` from an empty physical-planning census.
pub(crate) fn observe_physical_planning<T>(
    work: impl FnOnce() -> T,
) -> (T, PhysicalPlanningCensus) {
    PHYSICAL_PLANNING_CENSUS
        .with(|census| *census.borrow_mut() = PhysicalPlanningCensus::default());
    let value = work();
    let census = PHYSICAL_PLANNING_CENSUS.with(|census| census.borrow().clone());
    (value, census)
}

pub(crate) fn record_provider_offer(proposals: usize, declines: usize) {
    PHYSICAL_PLANNING_CENSUS.with(|census| {
        let mut census = census.borrow_mut();
        census.provider_invocations = census.provider_invocations.saturating_add(1);
        census.proposals = census
            .proposals
            .saturating_add(u64::try_from(proposals).unwrap_or(u64::MAX));
        census.admission_assessments = census
            .admission_assessments
            .saturating_add(u64::try_from(proposals).unwrap_or(u64::MAX));
        census.declines = census
            .declines
            .saturating_add(u64::try_from(declines).unwrap_or(u64::MAX));
    });
}

pub(crate) fn record_schedule_verification() {
    PHYSICAL_PLANNING_CENSUS.with(|census| {
        let mut census = census.borrow_mut();
        census.schedule_verifications = census.schedule_verifications.saturating_add(1);
    });
}

pub(crate) fn record_frontier_result(admitted: usize, rejections: usize, raw_declines: usize) {
    PHYSICAL_PLANNING_CENSUS.with(|census| {
        let mut census = census.borrow_mut();
        let admitted = u64::try_from(admitted).unwrap_or(u64::MAX);
        let rejections = u64::try_from(rejections).unwrap_or(u64::MAX);
        census.admitted_implementations = census.admitted_implementations.saturating_add(admitted);
        census.retained_implementations = census.retained_implementations.saturating_add(admitted);
        census.proposal_rejections = census.proposal_rejections.saturating_add(
            rejections.saturating_sub(u64::try_from(raw_declines).unwrap_or(u64::MAX)),
        );
        census.frontier_rejections = census.frontier_rejections.saturating_add(rejections);
        census.admitted_sort_items = census.admitted_sort_items.saturating_add(admitted);
        census.rejection_sort_items = census.rejection_sort_items.saturating_add(rejections);
    });
}

pub(crate) fn record_plan_combination(accepted: bool) {
    PHYSICAL_PLANNING_CENSUS.with(|census| {
        let mut census = census.borrow_mut();
        census.plan_combinations = census.plan_combinations.saturating_add(1);
        if accepted {
            census.accepted_plan_combinations = census.accepted_plan_combinations.saturating_add(1);
        } else {
            census.rejected_plan_combinations = census.rejected_plan_combinations.saturating_add(1);
        }
    });
}

pub(crate) fn record_complete_plan_retention(retained: usize) {
    PHYSICAL_PLANNING_CENSUS.with(|census| {
        let mut census = census.borrow_mut();
        let retained = u64::try_from(retained).unwrap_or(u64::MAX);
        census.retained_complete_plans = census.retained_complete_plans.saturating_add(retained);
        census.complete_plan_sort_items = census.complete_plan_sort_items.saturating_add(retained);
    });
}
