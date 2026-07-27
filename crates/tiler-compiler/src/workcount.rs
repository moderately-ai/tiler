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
