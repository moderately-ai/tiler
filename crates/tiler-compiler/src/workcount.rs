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

use std::cell::Cell;

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
