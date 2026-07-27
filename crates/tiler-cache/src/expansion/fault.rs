//! Deterministic publication-phase faults and orderings, for the killed-writer
//! harness.
//!
//! Two seams live here and they answer the same objection from opposite sides.
//! [`reach`] names the instant a writer dies. [`rendezvous`] names an instant
//! every racing writer can be *held* at, so the harness establishes the order it
//! needs by observing it rather than by sleeping long enough to make it likely.
//!
//! # Why this exists in the crate rather than beside the harness
//!
//! ADR 0050's crash properties are about what a writer leaves behind when it
//! dies *part-way through publication*, so the evidence has to name the point it
//! died at. Nothing outside this crate can name those points: `publish` is a
//! private method, and the two phases that matter most — a temporary half
//! written, and a temporary written and validated but not yet renamed — are
//! interior states no external observer can schedule. A supervisor watching the
//! filesystem could kill *somewhere near* them and would report nine phases
//! having measured rather fewer, which is the failure mode of a test that looks
//! like evidence.
//!
//! # Why it is `cfg(test)` and not a Cargo feature
//!
//! A feature would be public surface on a boundary Tom has not accepted, and
//! Cargo unifies features across a build graph: one unrelated crate enabling it
//! would arm mid-publication aborts inside somebody's production cache. That is
//! not a trade-off, it is a defect with an opt-in spelling. Under `cfg(test)`
//! the seam compiles into this crate's own test binary and nowhere else, and the
//! harness reaches it by re-executing that binary — which is also what makes the
//! child a *real process* running the *real* `ExpansionCache`.
//!
//! # What an armed child does
//!
//! It calls [`process::abort`] rather than [`process::exit`]. A killed writer
//! runs no destructor, unwinds no stack, closes no descriptor deliberately, and
//! flushes no buffer; `exit` would run `atexit` handlers and let the harness
//! measure a tidier death than a crash. `abort` is the closest a process can get
//! to being killed at an instant it chooses.

use std::cell::Cell;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

/// Environment variable naming the phase an armed child aborts at.
///
/// Absent — which is every ordinary test run — [`reach`] does nothing at all.
pub(super) const PHASE_VARIABLE: &str = "TILER_CACHE_FAULT_PHASE";

/// A point in the publication protocol at which a writer may be killed.
///
/// Deliberately **not** `#[non_exhaustive]`: [`Self::as_str`] maps it totally
/// and [`Self::KILL_POINTS`] is the enumeration the harness iterates, so a phase
/// added without a name and without a place in that list fails to compile rather
/// than being silently unmeasured.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) enum Phase {
    /// The per-key lock is held and nothing has been read under it.
    AfterLock,
    /// The post-lock recheck has run and found nothing to serve.
    AfterRecheck,
    /// The `create_new` temporary exists and is empty.
    AfterTempCreate,
    /// Half the encoded bundle has reached the temporary.
    MidWrite,
    /// The whole encoded bundle has reached the temporary.
    AfterWrite,
    /// The temporary has been re-read and validated through its own descriptor.
    AfterTempValidation,
    /// The temporary has been synchronized, under the `Fsync` policy.
    AfterFileSync,
    /// The rename has published the entry.
    AfterRename,
    /// The entry directory has been synchronized, under the `Fsync` policy.
    AfterDirectorySync,
}

impl Phase {
    /// Every phase a writer may be killed at, in publication order.
    pub(super) const KILL_POINTS: [Self; 9] = [
        Self::AfterLock,
        Self::AfterRecheck,
        Self::AfterTempCreate,
        Self::MidWrite,
        Self::AfterWrite,
        Self::AfterTempValidation,
        Self::AfterFileSync,
        Self::AfterRename,
        Self::AfterDirectorySync,
    ];

    /// Returns this phase's stable lowercase identifier.
    ///
    /// An arm that states its constant, never a discriminant read from
    /// declaration order: the identifier crosses a process boundary as text and
    /// appears in the recorded evidence, so reordering the enum must not silently
    /// rename a measured phase.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::AfterLock => "after-lock",
            Self::AfterRecheck => "after-recheck",
            Self::AfterTempCreate => "after-temp-create",
            Self::MidWrite => "mid-write",
            Self::AfterWrite => "after-write",
            Self::AfterTempValidation => "after-temp-validation",
            Self::AfterFileSync => "after-file-sync",
            Self::AfterRename => "after-rename",
            Self::AfterDirectorySync => "after-directory-sync",
        }
    }

    /// Resolves one phase identifier, or `None` for a name no phase carries.
    ///
    /// Searches [`Self::KILL_POINTS`] rather than restating the mapping, so a
    /// phase missing from that list is unreachable here too and cannot be armed
    /// by a harness that still counts it.
    pub(super) fn parse(text: &str) -> Option<Self> {
        Self::KILL_POINTS
            .into_iter()
            .find(|phase| phase.as_str() == text)
    }
}

/// Aborts this process when it was armed to die at `phase`.
///
/// Read from the environment on every call rather than cached, because the cost
/// is irrelevant here and a cache would be one more thing that could disagree
/// with what the harness set.
pub(super) fn reach(phase: Phase) {
    let armed = env::var(PHASE_VARIABLE)
        .ok()
        .and_then(|name| Phase::parse(&name));
    if armed == Some(phase) {
        process::abort();
    }
}

/// Environment variable naming the file this process creates on arriving at the
/// rendezvous.
///
/// Absent, [`rendezvous`] announces nothing and only waits — which is what an
/// unnumbered participant would want, and is not what the harness does.
pub(super) const ARRIVAL_VARIABLE: &str = "TILER_CACHE_FAULT_ARRIVAL";

/// Environment variable naming the file whose appearance releases this process
/// from the rendezvous.
///
/// Absent — which is every ordinary test run — [`rendezvous`] does nothing at
/// all, on the same guard [`reach`] uses.
pub(super) const RELEASE_VARIABLE: &str = "TILER_CACHE_FAULT_RELEASE";

/// How long a parked process waits for a release that never arrives.
///
/// A backstop against a harness that died without releasing, deliberately far
/// longer than the harness's own wait for arrivals. That ordering is what makes
/// the *parent* report a failed rendezvous: it knows how many children it
/// expected and which ones are missing, where an orphan knows only that it
/// waited. Nothing any assertion claims depends on this duration.
const RELEASE_DEADLINE: Duration = Duration::from_secs(60);

/// How often a parked process looks for its release.
const RELEASE_POLL: Duration = Duration::from_millis(2);

/// Announces that this process has reached the rendezvous, then blocks until the
/// harness releases it.
///
/// Called from [`ExpansionCache::resolve`](super::store::ExpansionCache::resolve)
/// at the one point that makes a racing case decidable: the lock-free lookup has
/// run and missed, and no lock has been taken. A process released from there has
/// already committed to the locked path, so what any other process does next
/// cannot turn it into a lock-free hit.
///
/// A process that *hit* returned before this call and never announces itself, so
/// a missing arrival is exactly the statement "some process found an entry
/// before the barrier opened" — which the harness reports rather than absorbs.
pub(super) fn rendezvous() {
    let Ok(release) = env::var(RELEASE_VARIABLE) else {
        return;
    };
    if let Ok(arrival) = env::var(ARRIVAL_VARIABLE) {
        fs::write(&arrival, b"arrived").expect("a rendezvous arrival file is writable");
    }
    let release = Path::new(&release);
    let expiry = Instant::now() + RELEASE_DEADLINE;
    while !release.exists() {
        assert!(
            Instant::now() < expiry,
            "no release reached {} within {RELEASE_DEADLINE:?}",
            release.display(),
        );
        thread::sleep(RELEASE_POLL);
    }
}

/// A filesystem call that can be made to fail without killing the process.
///
/// # Why this is separate from [`Phase`]
///
/// A [`Phase`] names an instant a writer *dies*, and [`reach`] implements that
/// with [`process::abort`]. Neither can make a call return an error, so neither
/// can reach the states this ticket is about: a rename that succeeded followed
/// by a step that failed. Killing the writer at `AfterRename` proves what a
/// crash leaves on disk; it says nothing about what a surviving writer
/// *reports*, which is a different property and the one that was wrong.
///
/// # Why a thread-local and not the environment
///
/// [`reach`] reads an environment variable because its harness re-executes the
/// test binary as a child process, so the arming has to cross a process
/// boundary. These faults are observed in-process by the test that armed them,
/// and a thread-local keeps one test's arming off every other test's writer
/// even when the suite runs them concurrently in one process. It is also
/// impossible to leave armed by accident: [`InjectionGuard`] disarms on drop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Injection {
    /// Synchronizing the containing entry directory, after publication.
    EntryDirectorySync,
    /// Releasing the per-key lock, after publication.
    LockRelease,
}

thread_local! {
    /// The fault this thread is armed to inject, if any.
    static ARMED: Cell<Option<Injection>> = const { Cell::new(None) };
}

/// Arms `injection` on this thread until the returned guard is dropped.
pub(super) fn inject(injection: Injection) -> InjectionGuard {
    ARMED.with(|armed| armed.set(Some(injection)));
    InjectionGuard
}

/// Disarms this thread's injected fault when dropped.
///
/// A guard rather than a bare disarm call, so a test that fails an assertion
/// mid-way still leaves the thread clean for whatever runs next on it.
pub(super) struct InjectionGuard;

impl Drop for InjectionGuard {
    fn drop(&mut self) {
        ARMED.with(|armed| armed.set(None));
    }
}

/// Returns the error this thread is armed to produce at `injection`.
///
/// One-shot: the arming is cleared as it fires, so a retry inside the same call
/// observes the real filesystem rather than the fault a second time.
pub(super) fn injected(injection: Injection) -> Option<io::Error> {
    ARMED.with(|armed| {
        if armed.get() == Some(injection) {
            armed.set(None);
            Some(io::Error::from(io::ErrorKind::Other))
        } else {
            None
        }
    })
}
