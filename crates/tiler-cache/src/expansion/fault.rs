//! Deterministic publication-phase faults, for the killed-writer harness.
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

use std::env;
use std::process;

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
