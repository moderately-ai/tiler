//! Whether the resolved cache root can do what the publication protocol needs.
//!
//! [`ExpansionCache::preflight`] probes the filesystem properties that protocol
//! rests on and reports; it decides nothing, refuses nothing, and is never
//! reached from the cache's own path. That module states why it has no automatic
//! caller *inside* `tiler_cache` — a probe on the lookup path would answer a
//! question whose answer does not change between lookups — and it leaves the
//! asking to whoever chose the root. This module is that caller, and it lives
//! here for [`crate::cache_root`]'s and [`crate::eviction`]'s reason: the root is
//! a frontend decision, so "is that root any good, and who is told" belongs
//! beside the resolution rather than inside a storage protocol that must stay
//! testable without a host.
//!
//! # Why a report and never a refusal
//!
//! Every cache operation already fails closed on its own: a publication that
//! cannot take its lock, rename atomically, or validate is not published, and a
//! hit that does not validate is not served. An unsuitable root therefore costs
//! recompilation and attribution, never a wrong artifact, and failing a build
//! over it would make an optional accelerator a correctness dependency — the
//! same reasoning that makes an unreadable eviction statement a typed refusal
//! *of the eviction*. What a consumer loses without this line is exactly the
//! attribution: entries that quietly never hit, and no way to tell why.
//!
//! # The trigger, and the bound it carries
//!
//! [`crate::aot::open_cache`] probes the root it has just opened, before the
//! expansion uses it, and at most once per process. *Before* rather than after,
//! because the loudest symptom this reports is a cache that never publishes: a
//! probe gated on a publication the way the eviction sweep is would stay silent
//! on exactly the host it exists to describe.
//!
//! [`PreflightGate`] is [`crate::eviction::EvictionGate`]'s rule, and it carries
//! that module's measured meaning per driver: under Cargo the expanding process
//! is `rustc`, one per crate compilation, so a build probes at most once per
//! crate that delivered; under rust-analyzer it is the proc-macro server, one
//! process for the editor session, so a session probes once. The bound that rule
//! carries is stated rather than left to be discovered — a process expanding
//! under two different roots probes the first and says nothing about the second.
//! The alternative is per-process state keyed by path, which buys a case no
//! measured driver produces and costs the one-sentence rule that makes the
//! amortization explainable.
//!
//! A cache with no root is skipped *without claiming the gate*.
//! `TILER_EXPANSION_CACHE_DIR=off` is a consumer's own decision and has nothing
//! to probe, and claiming the gate for it would silence a real root the same
//! process resolved for a later expansion.
//!
//! # The shape of the line
//!
//! The eviction's refusal is the shape this matches rather than a second one, as
//! `docs/integration/frontends.md` describes it: one line on the expanding
//! process's standard error, attributed to the macro that wrote it, naming what
//! is wrong, what it costs, and what to change, at most once per process. That
//! document also records the measured reach of such a line — Cargo forwards it
//! to the terminal for a build that expands, while a fully warm build runs no
//! macro and prints nothing.
//!
//! [`PreflightReport::cross_host_exclusion_caveat`] is deliberately *not*
//! printed. It qualifies a lock property that **holds**, and this line lists
//! only the properties that did not, so printing it here would attach a caveat
//! to a row that is not in the message.

use core::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use tiler_cache::expansion::{ExpansionCache, PreflightReport, PreflightVerdict};

use crate::cache_root::{DISABLE_VALUE, OVERRIDE_VARIABLE};

/// One probed property, as a capability a consumer can read, and the row of
/// [`PreflightReport`] that answers for it.
type ProbedProperty = (&'static str, fn(&PreflightReport) -> PreflightVerdict);

/// Every property [`ExpansionCache::preflight`] probes, phrased as what the
/// publication protocol needs it for.
///
/// One table, read by the renderer and counted by this module's tests, because
/// the table is the claim: a row the cache added and this list did not gain
/// would be a property whose refutation reached no consumer, while
/// [`PreflightReport::all_probed_properties_hold`] still counted it.
///
/// Each phrase names the capability rather than the probe, because the consumer
/// reading the line owns the filesystem and not the check.
const PROPERTIES: [ProbedProperty; 5] = [
    (
        "one filesystem under the whole root, without which a publication's rename crosses devices \
         and fails",
        PreflightReport::same_device,
    ),
    (
        "a create-new that refuses a path already there, which is what makes a temporary file this \
         expansion's own",
        PreflightReport::create_new_excludes,
    ),
    (
        "an advisory lock that excludes a second holder on this host, without which concurrent \
         expansions duplicate the compiler work the cache exists to share",
        PreflightReport::lock_excludes_locally,
    ),
    (
        "a rename that publishes an entry over whatever was there, which is the only operation \
         that makes one visible",
        PreflightReport::rename_publishes,
    ),
    (
        "a modification time on a written file, which is what the automatic eviction orders \
         entries by",
        PreflightReport::modification_time_reported,
    ),
];

/// A resolved root that did not answer for every property, and how it answered.
///
/// Typed and non-erasing under ADR 0074 convention 1: the root and each
/// property's own verdict survive to the message, because "the cache root is
/// unsuitable" tells a consumer neither which directory nor what to change —
/// and because a refuted property and an unrunnable probe have different
/// remedies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnsuitableRoot {
    /// The root the verdicts describe.
    root: PathBuf,
    /// Every property that did not hold, in [`PROPERTIES`] order.
    missing: Vec<(&'static str, PreflightVerdict)>,
}

impl UnsuitableRoot {
    /// The properties this root did not answer for, rendered as one clause.
    fn rendered_missing(&self) -> String {
        self.missing
            .iter()
            .map(|(capability, verdict)| format!("{capability} ({})", rendered(*verdict)))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

impl fmt::Display for UnsuitableRoot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "the expansion cache root `{}` does not answer for every filesystem property the \
             expansion cache's publication protocol rests on: {}. The expansion continues — this \
             region is compiled, validated, and embedded exactly as it would be, and every cache \
             operation fails closed on its own, so an unsuitable root costs repeated compiler work \
             rather than a wrong artifact. `not probed` most often means the root is not writable \
             rather than that the filesystem is unsuitable. Set `{OVERRIDE_VARIABLE}` to an \
             absolute directory path only you can write on a filesystem that answers for these, or \
             to `{DISABLE_VALUE}` to expand without a cache. Reported once per build process",
            self.root.display(),
            self.rendered_missing(),
        )
    }
}

/// How one verdict reads in a message that lists only the rows which did not
/// hold.
///
/// [`PreflightVerdict::Holds`] is unreachable through [`unsuitable`], which
/// collects the other two and nothing else. It is rendered rather than asserted
/// away, because a row that somehow reached here should read as what it is
/// instead of aborting an expansion that has already produced a correct
/// artifact.
const fn rendered(verdict: PreflightVerdict) -> &'static str {
    match verdict {
        PreflightVerdict::Holds => "holds",
        PreflightVerdict::Refuted => "refuted",
        PreflightVerdict::NotRun => "not probed",
    }
}

/// The once-per-process amortization the automatic probe runs under.
///
/// One flag, because the probe and its message are one act: a root that answers
/// for everything writes nothing, and there is no second event to gate.
///
/// `Relaxed`, deliberately and for [`crate::eviction::EvictionGate`]'s reason.
/// The flag orders nothing and guards no shared data — losing a race means one
/// extra probe, which creates and removes its own files under the cache
/// namespace and touches no entry, or one extra line on standard error.
#[derive(Debug)]
pub(crate) struct PreflightGate {
    probed: AtomicBool,
}

impl PreflightGate {
    /// A gate that has not yet probed anything.
    pub(crate) const fn new() -> Self {
        Self {
            probed: AtomicBool::new(false),
        }
    }

    /// The one gate this process amortizes against.
    ///
    /// A `static` rather than a value threaded from [`crate::expand`], because
    /// the rule is *per process* and an expansion has no other object with that
    /// lifetime. A test constructs its own with [`Self::new`] instead, so no
    /// test's probe depends on whether another test ran first.
    pub(crate) fn process() -> &'static Self {
        static PROCESS: PreflightGate = PreflightGate::new();
        &PROCESS
    }

    /// True for the first caller in this process, and false for every later one.
    fn claim(&self) -> bool {
        !self.probed.swap(true, Ordering::Relaxed)
    }
}

/// Probes `cache`'s root once per process, reporting an unsuitable answer on
/// this process's standard error.
///
/// Returns nothing, because there is nothing a caller may decide from it: the
/// expansion proceeds identically either way, which is the whole of "a report
/// and never a refusal".
pub(crate) fn report_unsuitable_root(gate: &PreflightGate, cache: &ExpansionCache) {
    let _ = reported_to(gate, cache, &mut io::stderr());
}

/// [`report_unsuitable_root`], writing to `out` rather than to the process.
///
/// The seam exists for [`crate::eviction`]'s reason: a check that could only
/// observe the real standard error would be asserting on the harness rather than
/// on what a consumer reads. The report is returned for the same reason, and
/// production ignores it.
fn reported_to(
    gate: &PreflightGate,
    cache: &ExpansionCache,
    out: &mut impl io::Write,
) -> Option<UnsuitableRoot> {
    // Checked before the gate is claimed rather than after: a disabled cache has
    // no root to probe, and consuming the process's one probe on it would
    // silence a real root a later expansion in the same process resolves.
    cache.root()?;
    if !gate.claim() {
        return None;
    }
    let unsuitable = unsuitable(&cache.preflight())?;
    // Best effort. A closed or failing standard error is not a reason to fail an
    // expansion whose artifact is correct either way.
    let _ = writeln!(out, "`tiler::tensor!`: {unsuitable}");
    Some(unsuitable)
}

/// Reads a report as the properties its root did not answer for, or `None` when
/// it answered for all of them.
///
/// A report with no root answers `None` too, and that is not a clean bill of
/// health: a rootless report is a disabled cache, which probed nothing and has
/// no directory to name in a diagnostic.
fn unsuitable(report: &PreflightReport) -> Option<UnsuitableRoot> {
    let root = report.root()?;
    let missing: Vec<(&'static str, PreflightVerdict)> = PROPERTIES
        .into_iter()
        .map(|(capability, answered)| (capability, answered(report)))
        .filter(|(_, verdict)| *verdict != PreflightVerdict::Holds)
        .collect();
    (!missing.is_empty()).then(|| UnsuitableRoot {
        root: root.to_path_buf(),
        missing,
    })
}

#[cfg(test)]
mod tests;
