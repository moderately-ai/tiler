//! Test-only counts that pin how often a verified environment solves or answers.
//!
//! Thread-local so `cargo test` sharing a process cannot leak one test's work
//! into another. `observe` zeros the slot before running, so a guard never
//! measures a previous leftover.

use std::cell::Cell;

/// One named counter over a thread-local slot.
pub(crate) struct WorkCounter {
    slot: &'static std::thread::LocalKey<Cell<usize>>,
    #[cfg(test)]
    name: &'static str,
}

impl WorkCounter {
    const fn new(
        #[cfg_attr(not(test), allow(unused_variables))] name: &'static str,
        slot: &'static std::thread::LocalKey<Cell<usize>>,
    ) -> Self {
        Self {
            slot,
            #[cfg(test)]
            name,
        }
    }

    pub(crate) fn record(&self) {
        self.slot.with(|count| count.set(count.get() + 1));
    }

    /// Runs `work` with this counter zeroed and returns its result beside the
    /// number of occurrences it caused.
    #[cfg(test)]
    pub(crate) fn observe<T>(&self, work: impl FnOnce() -> T) -> (T, usize) {
        self.slot.with(|count| count.set(0));
        let value = work();
        (value, self.slot.with(Cell::get))
    }

    #[cfg(test)]
    pub(crate) const fn name(&self) -> &'static str {
        self.name
    }
}

thread_local! {
    static SEMANTIC_CLOSURE: Cell<usize> = const { Cell::new(0) };
    static GUARD_HYPOTHESIS: Cell<usize> = const { Cell::new(0) };
    static INTERVAL: Cell<usize> = const { Cell::new(0) };
    static EQUALITY: Cell<usize> = const { Cell::new(0) };
    static POSITIVITY: Cell<usize> = const { Cell::new(0) };
    static DETERMINED: Cell<usize> = const { Cell::new(0) };
}

/// Counts semantic-closure solves. A verified environment construction records
/// exactly one; proof queries must record none.
pub(crate) static SEMANTIC_CLOSURE_SOLVES: WorkCounter =
    WorkCounter::new("semantic-closure solve", &SEMANTIC_CLOSURE);

/// Counts guard-hypothesis solves over authored semantic relations plus one
/// guard. Independent of the semantic-closure count.
pub(crate) static GUARD_HYPOTHESIS_SOLVES: WorkCounter =
    WorkCounter::new("guard-hypothesis solve", &GUARD_HYPOTHESIS);

/// Counts [`super::ShapeEnv::extent_interval`] reads of the retained summary.
pub(crate) static INTERVAL_QUERIES: WorkCounter = WorkCounter::new("interval query", &INTERVAL);

/// Counts [`super::ShapeEnv::proves_equal`] reads of the retained summary.
pub(crate) static EQUALITY_QUERIES: WorkCounter = WorkCounter::new("equality query", &EQUALITY);

/// Counts [`super::ShapeEnv::proves_positive`] reads of the retained summary.
pub(crate) static POSITIVITY_QUERIES: WorkCounter =
    WorkCounter::new("positivity query", &POSITIVITY);

/// Counts determined-value reads of the retained summary.
pub(crate) static DETERMINED_QUERIES: WorkCounter =
    WorkCounter::new("determined-value query", &DETERMINED);

/// The six censuses observed together, so a fixture can name every population.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CensusSnapshot {
    pub semantic_closure: usize,
    pub guard_hypothesis: usize,
    pub interval: usize,
    pub equality: usize,
    pub positivity: usize,
    pub determined: usize,
}

#[cfg(test)]
fn reset_all() {
    for counter in [
        &SEMANTIC_CLOSURE_SOLVES,
        &GUARD_HYPOTHESIS_SOLVES,
        &INTERVAL_QUERIES,
        &EQUALITY_QUERIES,
        &POSITIVITY_QUERIES,
        &DETERMINED_QUERIES,
    ] {
        counter.slot.with(|count| count.set(0));
    }
}

#[cfg(test)]
fn snapshot() -> CensusSnapshot {
    CensusSnapshot {
        semantic_closure: SEMANTIC_CLOSURE_SOLVES.slot.with(Cell::get),
        guard_hypothesis: GUARD_HYPOTHESIS_SOLVES.slot.with(Cell::get),
        interval: INTERVAL_QUERIES.slot.with(Cell::get),
        equality: EQUALITY_QUERIES.slot.with(Cell::get),
        positivity: POSITIVITY_QUERIES.slot.with(Cell::get),
        determined: DETERMINED_QUERIES.slot.with(Cell::get),
    }
}

/// Runs `work` with every census zeroed and returns its result beside the
/// counts it caused.
#[cfg(test)]
pub(crate) fn observe_all<T>(work: impl FnOnce() -> T) -> (T, CensusSnapshot) {
    reset_all();
    let value = work();
    (value, snapshot())
}
