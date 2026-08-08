//! Whole-cache accounting, bounded collection, and the out-of-service purge.
//!
//! # The bound is a declared policy, not a default
//!
//! A collector is a machine for removing things, so the repository rule that
//! nothing may be dropped or silently truncated does not read as "never
//! collect". It reads as: the boundary is explicit, stated, and observable. Two
//! decisions carry that here.
//!
//! **No bound applies unless a caller states it.** [`CollectionBound::UNBOUNDED`]
//! removes nothing and is the crate's `Default`. A byte ceiling, an entry
//! ceiling, and a maximum entry age each exist exactly when a caller states one,
//! so an entry can never vanish because of a number nobody chose — the research
//! note is explicit that "exact defaults require workload measurement", and a
//! *size* default guessed ahead of that measurement would delete a user's
//! compiled artifacts on the strength of a guess, invisibly. [`super::Limits`]
//! therefore still carries no maximum entry count: the bound is an argument to
//! one explicit operation, not a property of a cache that is otherwise only ever
//! read from.
//!
//! [`MaxEntryAge::DEFAULT`] is the one number this crate names, and it is named
//! rather than applied: it is a constant a frontend running an automatic
//! eviction *cites*, it is not `CollectionBound`'s `Default`, [`MaxEntryAge`]
//! implements no `Default`, and no operation here reaches for it. Its ground —
//! a product choice under Tom's 2026-08-04 decision, not a measurement — is
//! stated on the constant.
//!
//! **Every removal is named and every scanned entry is accounted for.**
//! [`CollectionReport::removed`] lists each removed entry individually with the
//! bytes it occupied, and [`CollectionReport::accounts_for_every_entry`] states
//! that the five dispositions are disjoint and total over the selection — a
//! statement about the loop below, which its own documentation bounds, and not
//! evidence that nothing left unreported. That evidence needs the entry files
//! themselves, so it is taken where they can be read: the tests observe the
//! namespace on both sides of every collection and require the entries that
//! disappeared to be exactly the keys the report named. Reaching a bound without
//! satisfying it is [`CollectionOutcome::BoundNotReached`] carrying what is
//! left, never a quiet stop.
//!
//! # Accounting is separate from collection, and disposable
//!
//! [`ExpansionCache::account`] scans and removes nothing, so an operator can
//! measure before deciding. Nothing it produces is written to disk, consulted by
//! a read, or trusted for hit correctness — which is the research note's fifth
//! garbage-collection rule satisfied by construction rather than by discipline.
//!
//! That refusal is also what makes the crash story trivial. A durable index of
//! entry sizes and recency would, after a crash, disagree with the filesystem,
//! and reconciling the two needs a repair rule — a rule whose failure mode is
//! deleting live entries or trusting a stale size. There is no such rule here
//! because there is no such index: the filesystem is the only authority on which
//! entries exist, and a scan is how the collector asks it.
//!
//! # The five properties, under collection
//!
//! **Complete identity.** Collection never derives a key from a subject, never
//! writes an entry, and never composes anything. It reads keys *out of paths*
//! through [`super::layout::key_of_entry_path`], which already refuses a label
//! of the wrong width, a non-lowercase-hexadecimal label, and an entry sitting
//! under a shard that is not its own. A file it cannot parse that way is not
//! treated as an entry: it is **not removed** and it is reported as
//! [`CacheAccounting::unrecognized`]. A collector that deleted what it could not
//! parse would be deleting on the strength of not understanding.
//!
//! **Validation on every hit.** The read path is untouched, and it cannot be
//! reached from here — collection produces no entry, so no read can be made to
//! skip anything. The complete set of transitions collection can cause at a
//! content path is `Hit -> Absent`: it only ever unlinks. It cannot turn a
//! rejection into a hit and it cannot turn an absence into one, so it changes
//! how often a compilation is spared and never what a hit means.
//!
//! **Immutable entries.** The collector never holds a writable descriptor to an
//! entry. Its only mutating operation on a content path is `remove_file`, and
//! accounting reads metadata rather than bytes.
//!
//! **Atomic publication.** Collection publishes nothing, but it must not undo a
//! publication it never measured. Between the scan and the removal a key may be
//! republished, so a selected entry is removed only under its own key lock and
//! only after a re-`stat` agrees with what the scan saw; a replacement is left
//! alone as [`CollectionOutcome`]'s `superseded` count. See
//! the removal-if-unchanged comparison on [`ExpansionCache::collect`] for what
//! that comparison does and does not establish.
//!
//! **Crash and race behaviour.** Stated in full on [`ExpansionCache::collect`]
//! and [`ExpansionCache::purge`].
//!
//! # Who runs a collection, and when
//!
//! **Never on the expansion path, and never scheduled from inside this crate.**
//! A collection is an explicit call that hands its report back. Three shapes
//! were eliminated rather than weighed, and all three eliminations stand:
//!
//! - *Collecting inside `get_or_publish` on a miss* puts a walk of every shard
//!   on the path the cache exists to make fast, and runs it hardest exactly when
//!   the cache is coldest — the state with the most misses is the state with the
//!   most scans, multiplied by however many processes are building.
//! - *A background thread the cache spawns* makes an accelerator start threads
//!   inside a compiler process nobody asked to be concurrent, has no lifetime in
//!   a process that may exit the moment expansion finishes, and returns its
//!   report to nobody — which is the silence this crate is built not to produce.
//! - *Collecting on a fraction of publications* makes the trigger unexplainable.
//!   "Why did my entry go away" would answer "a random draw during an unrelated
//!   build", and a bound has to have a trigger a person can name.
//!
//! **Tom decided on 2026-08-04 that the eviction itself is automatic**, with its
//! policy configured through environment variables and no maintenance command
//! shipped (`decide-the-expansion-cache-collection-schedule`). That supersedes
//! the "never automatically" conclusion of the design record — recorded, with
//! the original rationale preserved, in
//! [`docs/research/cache/bounded-collection.md`](https://github.com/moderately-ai/tiler/blob/main/docs/research/cache/bounded-collection.md).
//! It changes nothing above and nothing in this module's shape:
//!
//! - The automatic caller is the **frontend**, which invokes this operation off
//!   the hit path. Nothing here spawns a thread, consults a clock to decide
//!   whether it is time, or runs during a lookup.
//! - The policy arrives as an explicit typed value — [`CollectionBound`],
//!   carrying [`MaxEntryAge`] when an age is stated. **This crate reads no
//!   environment**, and variable names, parsing, and defaults stay with the
//!   frontend under the ADR 0089 root policy.
//! - Attribution survives automation, which is what the automatic case needs
//!   most: nobody is present to remember what they typed, so
//!   [`CollectionReport::bound`] carries the exact policy and every
//!   [`RemovedEntry`] carries the [`RemovalReason`] that selected it.
//!
//! An entry therefore still leaves for a statable reason — a bound somebody
//! configured, an entry older than the age it stated or among the oldest over a
//! ceiling it stated — and the report still says which entry, which bound, which
//! reason, and which order.
//!
//! # Boundary status
//!
//! [`MaxEntryAge`], [`MaxEntryAgeRefusal`], [`RemovalReason`],
//! [`CollectionBound::max_entry_age`], and [`RemovedEntry::reason`] were
//! accepted on 2026-08-04 as the age extension to the maintenance facade,
//! decided by the orchestrator under Tom's same-day delegation of internal
//! API-shape decisions (recorded in
//! `admit-an-age-bounded-automatic-eviction-into-the-expansion-cache`).
//! Everything else in this module was accepted on 2026-07-31 under
//! `accept-the-expansion-cache-maintenance-boundary`.

use core::fmt;
use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::key::CacheKey;
use super::layout::{self, Layout};
use super::lock::KeyLock;
use super::report::{CacheOperation, CacheUnavailable};
use super::store::ExpansionCache;

/// The greatest age a collection lets an entry reach before removing it.
///
/// A verified value rather than a plain [`Duration`] (ADR 0074 convention 6):
/// its only invariant — that the stated age is not zero — is established by
/// [`Self::new`] and cannot be circumvented, so no [`CollectionBound`] can carry
/// an age that is not a bound.
///
/// # Why zero is the one value refused, and the only one
///
/// A maximum age of zero makes the predicate true for every entry the host can
/// date, including one published this instant. That is not a short retention
/// window; it is "remove everything" said obliquely, and it carries a failure the
/// honest spelling does not — it removes an entry a concurrent build published
/// microseconds ago and is about to hit. A caller that means "remove everything"
/// has two operations that say so: `CollectionBound { max_entries: Some(0), .. }`
/// and [`ExpansionCache::purge`].
///
/// Nothing else is refused, and refusing more would mean choosing a number. A
/// one-second maximum is a legitimate policy for a test or a scratch root, and a
/// minimum floor above it would be exactly the guess this module declines to make
/// for the size ceilings. A *negative* age needs no check at all: [`Duration`] is
/// unsigned, so it is unrepresentable rather than unchecked, and a frontend
/// parsing `-1` from its environment refuses before it reaches a [`Duration`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MaxEntryAge(Duration);

impl MaxEntryAge {
    /// Thirty days: the age a frontend cites when a consumer configured nothing.
    ///
    /// **A product choice under Tom's 2026-08-04 decision, not a measurement**,
    /// and it must not be cited as one. Nothing in this crate applies it: it is
    /// not [`CollectionBound`]'s `Default`, [`MaxEntryAge`] implements no
    /// `Default`, and no operation here reaches for it. A caller that wants it
    /// names it.
    ///
    /// Its ground, stated so it can be argued with rather than inherited:
    ///
    /// - The asymmetry the whole collector is built on points at the *longer*
    ///   end of any plausible range. A wrong eviction costs one recompilation;
    ///   an over-long retention costs disk that the measured rate bounds.
    /// - The measured rate is modest. The self-contained embedding note records
    ///   envelopes of 32,136–47,803 bytes, and the schedule decision records on
    ///   the order of ten to twenty megabytes per editing afternoon — so a
    ///   30-day window holds the steady state to roughly 200–400 MB over about
    ///   twenty working days, small beside the build caches already on the same
    ///   machine.
    /// - Growth is driven less by editing than by re-keying: every Apple
    ///   toolchain update orphans every entry published before it, all at once.
    ///   A 30-day window reclaims each orphaned generation within a month of the
    ///   update that orphaned it.
    /// - A shorter window hands a developer returning from an ordinary absence a
    ///   completely cold cache on their first build back, which is a visible
    ///   cost the shorter end buys nothing for.
    ///
    /// **Corrected 2026-08-06 — the band the second ground cites has moved, and
    /// the window itself is not re-decided here.** Re-running that note's own
    /// producer over the same members against the current artifact encoding
    /// gives **141,532–159,037 bytes** per envelope. Every carried `metallib` is
    /// byte-identical to the 2026-07-31 record, so the growth is entirely
    /// artifact encoding rather than backend output, compiler flags, or a Metal
    /// toolchain difference. The second ground therefore projects roughly
    /// **0.9–1.6 GB** where it says 200–400 MB, and the 200–400 MB it names is
    /// now roughly **1,300–2,800 entries**. Its 2026-07-31 figures are left
    /// above rather than overwritten, because they are what this choice was
    /// argued from; `docs/research/cache/hot-path-efficiency.md`'s Section 9
    /// carries the re-derivation and the attribution, and the collection design
    /// carries the matching correction to the same ground.
    ///
    /// **The projection still supports thirty days, at roughly a quarter of the
    /// margin it had.** Three of the four grounds do not depend on per-entry
    /// size at all — the eviction asymmetry, the re-keying that drives growth,
    /// and the cold first build a shorter window buys — and the second one's
    /// comparison survives in kind rather than by a hair: 0.9–1.6 GB is still
    /// well under the Cargo output a single gate of this workspace produces,
    /// which `AGENTS.md` puts at 7–15 GB. What is gone is the order of
    /// magnitude between them. **One further growth the size of the one just
    /// measured would put the steady state at 4–7 GB**, and the second ground
    /// would then state the opposite of what it is cited for; that is this
    /// window's reconsideration trigger from the disk side, and firing it is a
    /// product decision rather than one this crate makes.
    ///
    /// **What would replace it with a derived number:** a measurement of how
    /// long an entry stays useful — working-set lifetime, not the per-entry size
    /// that exists today. `measure-the-expansion-cache-hot-path-efficiency` is
    /// where that evidence would come from.
    ///
    /// Spelled in hours because `Duration::from_days` is still unstable on the
    /// pinned toolchain, and a constant does not justify a feature gate.
    pub const DEFAULT: Self = Self(Duration::from_hours(30 * 24));

    /// States a maximum entry age, refusing one that is not a bound.
    ///
    /// # Errors
    ///
    /// Returns [`MaxEntryAgeRefusal::Zero`] when `max_age` is zero. The refusal
    /// is the whole response: a collection under a policy this refuses does not
    /// happen, because the policy cannot be constructed, so there is no path on
    /// which a bad age is repaired to a guessed one.
    pub const fn new(max_age: Duration) -> Result<Self, MaxEntryAgeRefusal> {
        if max_age.is_zero() {
            return Err(MaxEntryAgeRefusal::Zero);
        }
        Ok(Self(max_age))
    }

    /// The stated maximum age.
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        self.0
    }

    /// True when an entry the scan dated at `published` has reached this age by
    /// `now`.
    ///
    /// # An age the host cannot compute is treated as young
    ///
    /// Two inputs leave the age unknown: a modification time the host could not
    /// report at all, and one dated *after* `now` — which happens when a clock
    /// moved backwards between the publication and the collection, when a file
    /// was stamped into the future, or when two machines share a root over a
    /// network filesystem with skewed clocks. Neither is age-selected.
    ///
    /// The direction is not a convenience. It is the same one
    /// [`ExpansionCache::sweep_temporaries`] takes for an abandoned temporary
    /// and the same one this module's selector takes for an undatable entry, and
    /// for the same asymmetry: keeping something collectable costs bounded disk,
    /// and removing something live costs work that has to be done again. It is
    /// also what makes a clock that moved backwards a non-event instead of a
    /// mass eviction — a future-dated entry is protected, and every other entry
    /// is judged on its own age exactly as before.
    ///
    /// The comparison is between two [`Duration`]s and never between two
    /// instants, so a maximum age larger than the time since the epoch is an age
    /// nothing has reached rather than an underflow.
    fn has_expired(self, published: Option<SystemTime>, now: SystemTime) -> bool {
        published
            .and_then(|published| now.duration_since(published).ok())
            .is_some_and(|age| age >= self.0)
    }
}

/// Why a stated maximum entry age is not a bound.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a refusal vocabulary a
/// caller forwards or renders rather than maps totally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum MaxEntryAgeRefusal {
    /// The stated age is zero.
    ///
    /// Carries no data because there is none to carry: the rejected value is the
    /// one the variant names, and a caller reacting to it needs the distinction
    /// rather than the quantity.
    Zero,
}

impl fmt::Display for MaxEntryAgeRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => formatter.write_str(
                "a maximum entry age of zero is not a bound: it selects every entry the host can \
                 date, including one published this instant, so a cache-wide removal has to be \
                 stated as an entry ceiling of zero or as a purge",
            ),
        }
    }
}

impl std::error::Error for MaxEntryAgeRefusal {}

/// What one collection is allowed to leave behind.
///
/// Every ceiling is optional and every one is absent by default. A collection
/// under [`Self::UNBOUNDED`] is a pure measurement: it selects nothing, removes
/// nothing, and reports the accounting it observed.
///
/// # The three ceilings compose, and cannot contradict each other
///
/// [`Self::max_total_bytes`] and [`Self::max_entries`] are *aggregate* ceilings
/// on what the cache retains; [`Self::max_entry_age`] is a *per-entry* predicate
/// over one entry's own evidence. A selection is the union: every entry the age
/// expires, plus the oldest of the remainder that the aggregate ceilings still
/// require removing. Each stated ceiling therefore only ever adds removals, so
/// two of them cannot disagree about an entry — there is no contradictory
/// composition to detect, and none is checked for.
///
/// A caller-constructed leaf value record, so its fields are visible
/// (ADR 0074 convention 6). Its one field with an invariant holds a verified
/// [`MaxEntryAge`] rather than a raw [`Duration`], which is what keeps the record
/// literal-constructible while making an unbounded age unrepresentable.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CollectionBound {
    /// Maximum total bytes of final entries the cache may retain.
    pub max_total_bytes: Option<u64>,
    /// Maximum number of final entries the cache may retain.
    pub max_entries: Option<u64>,
    /// Greatest age a retained entry may have reached, by its own published
    /// modification time.
    ///
    /// Absent by default, and absent in [`Self::UNBOUNDED`]. Stating it is what
    /// an automatic eviction does; [`MaxEntryAge::DEFAULT`] is the constant a
    /// frontend cites when its consumer configured nothing, and it is never
    /// filled in here.
    pub max_entry_age: Option<MaxEntryAge>,
}

impl CollectionBound {
    /// The bound that removes nothing.
    ///
    /// The default, and the only *applied* bound this crate supplies —
    /// [`MaxEntryAge::DEFAULT`] is a constant a caller cites, not a value
    /// anything reaches for. Collecting under `UNBOUNDED` is how a caller
    /// measures a cache without changing it.
    pub const UNBOUNDED: Self = Self {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: None,
    };

    /// True when a cache holding `bytes` across `entries` is within this bound's
    /// *aggregate* ceilings.
    ///
    /// An absent ceiling constrains nothing. This deliberately says nothing
    /// about [`Self::max_entry_age`], which is a property of an individual entry
    /// rather than of a total: a cache inside both aggregate ceilings can still
    /// hold an expired entry, so the selector consults the age separately and
    /// the outcome is decided from both.
    const fn admits(self, bytes: u64, entries: u64) -> bool {
        let within_bytes = match self.max_total_bytes {
            Some(limit) => bytes <= limit,
            None => true,
        };
        let within_entries = match self.max_entries {
            Some(limit) => entries <= limit,
            None => true,
        };
        within_bytes && within_entries
    }

    /// True when this bound can never select anything.
    const fn is_unbounded(self) -> bool {
        self.max_total_bytes.is_none() && self.max_entries.is_none() && self.max_entry_age.is_none()
    }
}

/// The order a bound removes entries in.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): the
/// selector maps it totally, and a variant added without a comparison would
/// otherwise fall into a wildcard that silently reused this one's order.
///
/// One variant, and the single variant is the finding rather than a placeholder.
/// The two orders a reader expects to see beside it were eliminated:
///
/// - **Least recently *used* first** would order by when an entry was last
///   served. Nothing on disk records that. Establishing it means either a
///   sidecar the reader updates — which puts a write on the deliberately
///   lock-free hit path, adds a failure mode to the operation the cache exists
///   to make fast, and creates a partial-sidecar crash surface — or the
///   filesystem's access time, whose maintenance is exactly what
///   `define-supported-expansion-cache-filesystems` owns and is not this
///   ticket's to assume. It is deferred there rather than guessed at.
/// - **Largest first** minimizes removals per byte reclaimed and is the wrong
///   objective: the cost of a wrong eviction is one recompilation regardless of
///   the entry's size, so ordering by size optimizes the metric nobody pays.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CollectionOrder {
    /// Least recently published first, by the entry file's modification time.
    ///
    /// Publication sets it as a side effect of writing the temporary that is
    /// then renamed into place, so it costs no extra I/O and no reader ever
    /// writes to obtain it.
    ///
    /// **It is insertion recency, not use recency, and the difference has a
    /// known cost.** An entry hit on every build is never rewritten, so it ages
    /// exactly like one nobody has wanted since the day it was published; under
    /// a tight bound a stable working set can therefore be evicted and rebuilt.
    /// That is a performance pathology and not a correctness one — every
    /// eviction costs at most a recompilation — and it is the honest price of
    /// refusing to make readers write.
    OldestPublicationFirst,
}

impl CollectionOrder {
    /// Returns this order's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OldestPublicationFirst => "oldest-publication-first",
        }
    }
}

/// One final entry, as a scan observed it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryFact {
    /// The key the entry is filed under, parsed from its path.
    pub key: CacheKey,
    /// Bytes the entry file occupies.
    pub bytes: u64,
    /// When the entry was published, or `None` when the host cannot report it.
    ///
    /// An unreportable time leaves the entry unordered, and it is sorted
    /// *last* — treated as newest and therefore protected. This is the same
    /// direction [`ExpansionCache::sweep_temporaries`] already chooses for an
    /// unknown age, and for the same reason: keeping something collectable costs
    /// bounded disk, and removing something live costs work that has to be done
    /// again.
    pub published: Option<SystemTime>,
}

/// What a scan found in one cache, without changing any of it.
///
/// Transient by construction. Nothing here is written to disk, and no read path
/// consults it.
#[derive(Clone, Debug, Default)]
pub struct CacheAccounting {
    entries: Vec<EntryFact>,
    total_bytes: u64,
    unrecognized: Vec<PathBuf>,
    quarantine_bytes: u64,
    quarantine_files: u64,
}

impl CacheAccounting {
    /// Every final entry the scan recognized.
    #[must_use]
    pub fn entries(&self) -> &[EntryFact] {
        &self.entries
    }

    /// Number of final entries.
    #[must_use]
    pub fn entry_count(&self) -> u64 {
        self.entries.len() as u64
    }

    /// Total bytes of every final entry.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Files found under the entries tree that are not entry content paths.
    ///
    /// Reported and never removed. A collector deleting what its own parser
    /// refused would be acting on the absence of understanding, and the entry
    /// path parser is deliberately strict enough that an unrecognized file is
    /// news: it means something other than this crate is writing into the
    /// namespace, which an operator should see rather than have tidied away.
    #[must_use]
    pub fn unrecognized(&self) -> &[PathBuf] {
        &self.unrecognized
    }

    /// Bytes retained across every shard's quarantine.
    ///
    /// Counted so the figure is visible, and never collected. Quarantine holds
    /// the exact bytes of entries that failed validation, which is evidence; its
    /// growth is already bounded at the point of *addition* by
    /// [`super::Limits::max_quarantine_bytes`], and reaching that bound is
    /// reported as [`super::QuarantineOutcome::BoundReached`] rather than
    /// silently discarding. Reclaiming retained evidence is a separate explicit
    /// act and this operation does not perform it.
    #[must_use]
    pub const fn quarantine_bytes(&self) -> u64 {
        self.quarantine_bytes
    }

    /// Files retained across every shard's quarantine.
    #[must_use]
    pub const fn quarantine_files(&self) -> u64 {
        self.quarantine_files
    }
}

/// Which of a bound's ceilings selected an entry.
///
/// Exists because attribution has to survive automation. When a person runs a
/// collection they know what they typed, so the bound on the report is enough;
/// when a frontend runs one under a configured policy, nobody is present to
/// remember, and "the cache was over a ceiling" and "this entry was older than
/// the configured age" lead to different corrections. Naming it per removal is
/// what keeps the report able to answer which.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a reason vocabulary a
/// consumer renders or partially classifies, never maps totally onto a value
/// the variant alone determines.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RemovalReason {
    /// The entry's own age had reached [`CollectionBound::max_entry_age`].
    OlderThanMaxEntryAge,
    /// The cache exceeded an aggregate ceiling and this was among its oldest
    /// entries.
    OverSizeCeiling,
}

impl RemovalReason {
    /// Returns this reason's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OlderThanMaxEntryAge => "older-than-max-entry-age",
            Self::OverSizeCeiling => "over-size-ceiling",
        }
    }
}

impl fmt::Display for RemovalReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One entry a collection removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedEntry {
    /// The key whose entry was removed.
    pub key: CacheKey,
    /// Bytes reclaimed, as measured by the scan that selected it.
    pub bytes: u64,
    /// Which ceiling of the stated bound selected this entry.
    pub reason: RemovalReason,
}

/// Whether a collection satisfied the bound it was given.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CollectionOutcome {
    /// The cache was already within the bound, so nothing was selected.
    WithinBound,
    /// Everything selected was removed and the bound is now satisfied.
    BoundReached,
    /// The bound is still exceeded after every selected entry was attempted.
    ///
    /// Not a failure and not silence: a collection skips a key another process
    /// holds the lock on, and skips an entry republished since the scan, so a
    /// busy cache can legitimately end a collection above its bound. Reporting
    /// exactly what is left is what makes a caller able to re-run rather than
    /// wonder.
    ///
    /// An age ceiling reaches this the same way and is counted the same way: an
    /// entry selected as older than [`CollectionBound::max_entry_age`] but not
    /// actually removed leaves the bound unreached, even when the byte and entry
    /// figures below sit inside their own ceilings. The two figures describe the
    /// aggregate, so a caller reading them alone would see a satisfied cache and
    /// re-run for no visible reason; the outcome is what tells it to.
    BoundNotReached {
        /// Bytes of final entries still held.
        bytes: u64,
        /// Final entries still held.
        entries: u64,
    },
}

/// What one collection observed and did.
#[derive(Debug)]
pub struct CollectionReport {
    accounting: CacheAccounting,
    bound: CollectionBound,
    order: CollectionOrder,
    selected: u64,
    removed: Vec<RemovedEntry>,
    contended: u64,
    superseded: u64,
    already_absent: u64,
    failed: Vec<CacheUnavailable>,
    outcome: CollectionOutcome,
}

impl CollectionReport {
    /// What the scan observed before anything was removed.
    #[must_use]
    pub const fn accounting(&self) -> &CacheAccounting {
        &self.accounting
    }

    /// The bound this collection was asked to enforce.
    #[must_use]
    pub const fn bound(&self) -> CollectionBound {
        self.bound
    }

    /// The order the bound selected in.
    #[must_use]
    pub const fn order(&self) -> CollectionOrder {
        self.order
    }

    /// Every entry that was removed, individually, with the bytes it occupied
    /// and the ceiling that selected it.
    ///
    /// A list rather than a count. A removal is the destructive act, so a report
    /// that aggregated them away would be unable to answer the one question an
    /// operator asks after an unexpected rebuild — which entry left, and under
    /// which bound. [`RemovedEntry::reason`] narrows "which bound" to which
    /// *ceiling* of it, which is what an automatic eviction needs: no person was
    /// present to remember what the policy said.
    #[must_use]
    pub fn removed(&self) -> &[RemovedEntry] {
        &self.removed
    }

    /// Bytes reclaimed, summed over [`Self::removed`].
    #[must_use]
    pub fn reclaimed_bytes(&self) -> u64 {
        self.removed.iter().map(|entry| entry.bytes).sum()
    }

    /// Selected entries left in place because another process held the key lock.
    ///
    /// A held lock means a writer or an evictor is working on that key right
    /// now, which is evidence the entry is live rather than collectable.
    #[must_use]
    pub const fn contended(&self) -> u64 {
        self.contended
    }

    /// Selected entries left in place because they changed after the scan.
    #[must_use]
    pub const fn superseded(&self) -> u64 {
        self.superseded
    }

    /// Selected entries that were already gone when the lock was taken.
    ///
    /// Ordinary under a second collector or an external deletion, and not
    /// counted as reclaimed: the bytes belong to whoever actually removed them.
    #[must_use]
    pub const fn already_absent(&self) -> u64 {
        self.already_absent
    }

    /// Per-entry failures, each naming the operation and path that failed.
    ///
    /// A collection does not abort on one unusable key; it records it and
    /// continues, so a single unreadable shard cannot stop the rest from being
    /// bounded. The failures reach the caller for the same reason every other
    /// refusal in this crate does.
    #[must_use]
    pub fn failed(&self) -> &[CacheUnavailable] {
        &self.failed
    }

    /// Whether the bound was satisfied.
    #[must_use]
    pub const fn outcome(&self) -> CollectionOutcome {
        self.outcome
    }

    /// Number of entries the bound selected for removal.
    #[must_use]
    pub const fn selected(&self) -> u64 {
        self.selected
    }

    /// True when every selected entry has exactly one recorded disposition.
    ///
    /// A selected entry is removed, contended, superseded, already absent, or
    /// failed, and those five are disjoint and total over the selection.
    ///
    /// # What it can and cannot fail on
    ///
    /// **It is a statement about this module's shape, not about the
    /// filesystem.** The step behind [`ExpansionCache::collect`] sets
    /// [`Self::selected`] to the length of the selection and then walks that
    /// same vector once,
    /// incrementing exactly one of the five counters per element, so both sides
    /// of the equality are one loop's iteration count. No filesystem state, no
    /// lock contention, no concurrent republication, and no unreadable entry can
    /// make it false; the only thing that can is a disposition arm here that
    /// records nothing, which is a defect in this file rather than an input.
    ///
    /// It therefore does **not** establish that nothing left the cache
    /// unreported. That claim is about a population this report does not
    /// contain — the entry files actually present before and after — and it is
    /// checked where that population is obtainable: `expansion::tests`'
    /// `collect_checked` reads the namespace on both sides of every collection
    /// it makes and requires the difference to be exactly the keys
    /// [`Self::removed`] names, and the collecting child in `expansion::harness`
    /// checks the selection against the scan and the stated bound inside the
    /// process that performed it.
    #[must_use]
    pub fn accounts_for_every_entry(&self) -> bool {
        self.removed.len() as u64
            + self.contended
            + self.superseded
            + self.already_absent
            + self.failed.len() as u64
            == self.selected
    }
}

/// What one purge retired and reclaimed.
#[derive(Debug, Default)]
pub struct PurgeReport {
    retired: Option<PathBuf>,
    reclaimed_trees: u64,
    reclaimed_bytes: u64,
    failed: Vec<CacheUnavailable>,
}

impl PurgeReport {
    /// Where the live namespace was renamed to, or `None` when there was none.
    #[must_use]
    pub fn retired(&self) -> Option<&Path> {
        self.retired.as_deref()
    }

    /// Retired namespace trees whose removal completed.
    ///
    /// More than one when an earlier purge died between its rename and its
    /// removal: nothing reads a retired tree, so a later purge reclaims it.
    #[must_use]
    pub const fn reclaimed_trees(&self) -> u64 {
        self.reclaimed_trees
    }

    /// Bytes reclaimed across every retired tree that was removed.
    #[must_use]
    pub const fn reclaimed_bytes(&self) -> u64 {
        self.reclaimed_bytes
    }

    /// Retired trees that could not be removed, with the reason.
    ///
    /// Reported rather than swallowed: a retired tree is already out of service
    /// and cannot affect correctness, but one that silently fails to be removed
    /// leaks disk indefinitely, which is precisely the thing an operator ran
    /// this to prevent.
    #[must_use]
    pub fn failed(&self) -> &[CacheUnavailable] {
        &self.failed
    }
}

impl ExpansionCache {
    /// Measures the cache without changing any of it.
    ///
    /// Takes no lock. Accounting reads directory listings and file metadata,
    /// never entry bytes, so it cannot observe a torn entry and does not need
    /// to: a publication makes an entry visible with one `rename`, so a scan
    /// sees a name that is either the old entry, the new entry, or neither.
    ///
    /// The figures describe the moment each `stat` ran and not a global
    /// snapshot, because taking one would mean quiescing every writer. That is
    /// exactly why nothing durable is derived from them.
    ///
    /// # Errors
    ///
    /// Returns [`CacheUnavailable`] when the entries tree exists and cannot be
    /// listed. A namespace that does not exist yet is an empty cache rather than
    /// an error, matching [`ExpansionCache::open`], which also creates nothing.
    /// A cache built by [`ExpansionCache::disabled`] has no namespace to scan
    /// and accounts for nothing, which is also what makes
    /// [`ExpansionCache::collect`] and [`ExpansionCache::purge`] total over it
    /// without stating the mode themselves.
    pub fn account(&self) -> Result<CacheAccounting, CacheUnavailable> {
        let Some(layout) = self.layout() else {
            return Ok(CacheAccounting::default());
        };
        let mut accounting = CacheAccounting::default();
        let entries_root = layout.entries_root();
        for shard in shards(&entries_root)? {
            for file in files(&shard)? {
                let metadata = match fs::metadata(&file) {
                    Ok(metadata) => metadata,
                    // Vanished between the listing and the `stat`: an ordinary
                    // race with an evictor or an external deletion, and the
                    // entry simply is not there to account for.
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => {
                        return Err(CacheUnavailable::new(
                            CacheOperation::ScanDirectory,
                            file,
                            error,
                        ));
                    }
                };
                if !metadata.is_file() {
                    accounting.unrecognized.push(file);
                    continue;
                }
                // The parser, not the extension, decides what an entry is: it
                // checks the label's exact width, its lowercase hexadecimal
                // alphabet, and that the shard directory is the key's own.
                let Ok(key) = layout::key_of_entry_path(&file) else {
                    accounting.unrecognized.push(file);
                    continue;
                };
                accounting.total_bytes = accounting.total_bytes.saturating_add(metadata.len());
                accounting.entries.push(EntryFact {
                    key,
                    bytes: metadata.len(),
                    published: metadata.modified().ok(),
                });
            }
        }

        let quarantine_root = layout.quarantine_root();
        for shard in shards(&quarantine_root)? {
            for file in files(&shard)? {
                if let Ok(metadata) = fs::metadata(&file)
                    && metadata.is_file()
                {
                    accounting.quarantine_files += 1;
                    accounting.quarantine_bytes =
                        accounting.quarantine_bytes.saturating_add(metadata.len());
                }
            }
        }
        Ok(accounting)
    }

    /// Brings the cache within `bound`, removing the oldest published entries
    /// first, and reports every entry it removed.
    ///
    /// Under [`CollectionBound::UNBOUNDED`] this selects nothing and is a pure
    /// measurement.
    ///
    /// # What the three ceilings each select
    ///
    /// [`CollectionBound::max_entry_age`] selects **per entry**, from that
    /// entry's own published modification time measured against this call's
    /// single clock reading; [`CollectionBound::max_total_bytes`] and
    /// [`CollectionBound::max_entries`] select **in aggregate**, taking the
    /// oldest of whatever the age left behind until the totals fit. The two
    /// compose as a union and each removal names which selected it, so a cache
    /// inside both aggregate ceilings still has its expired entries removed, and
    /// an entry removed to fit a ceiling is never reported as an expiry.
    ///
    /// The clock is read once, before the scan is interpreted, so every entry in
    /// one collection is judged against one instant. Reading it per entry would
    /// make an entry's fate depend on where the walk reached it.
    ///
    /// # It never blocks, and therefore needs no work budget
    ///
    /// Each selected entry is removed under its own key lock taken with a
    /// non-blocking `try_acquire`. A key another process holds is skipped and
    /// counted, never waited for. The research note's sixth gate asks for "a
    /// best-effort cleanup budget per invocation", and a non-blocking collector
    /// does not need one: its latency is its scan plus one lock attempt per
    /// candidate, with no unbounded wait to cap. Removing the limit is preferred
    /// to choosing a number for it.
    ///
    /// # Racing a reader
    ///
    /// A reader is never blocked, never told, and never harmed, in all three of
    /// the positions it can be in when a removal lands:
    ///
    /// - **Already returned.** [`ExpansionCache::lookup`] copies the validated
    ///   envelope into the [`super::CachedEntry`] it returns, so a caller
    ///   holding one owns its bytes and the file is irrelevant to it.
    /// - **Open but still reading.** The reader validated through a descriptor
    ///   it opened before the unlink. Removing a directory entry on the Unix and
    ///   Darwin hosts this crate targets does not reclaim the inode while a
    ///   descriptor is open, so the read completes and yields exactly the
    ///   published bytes. The crate's cross-process harness measures this
    ///   against the production bundle, across a real process boundary.
    /// - **Not yet opened.** `File::open` reports the entry absent, which is
    ///   [`super::MissReason::Absent`] — the one miss the reporting module calls
    ///   "not evidence of a problem" — and the caller rebuilds.
    ///
    /// There is no fourth position, and no window in which a reader observes a
    /// partially collected entry, because `unlink` of one file has no
    /// intermediate state a reader can name.
    ///
    /// # Racing a writer, and racing another collector
    ///
    /// The key lock serializes a removal against a publication's rename, and the
    /// re-`stat` under that lock is what keeps a collection from undoing a
    /// publication it never measured; the comparison and its deliberate
    /// non-airtightness are documented on the private removal step. Two collectors may select
    /// overlapping sets; the loser finds the entry gone and records
    /// [`CollectionReport::already_absent`], so the bytes are credited once to
    /// whoever actually removed them.
    ///
    /// # Dying part-way through
    ///
    /// There is nothing to recover. A collection is a sequence of independent
    /// single-file unlinks, each under its own lock, with no journal, no
    /// in-progress marker, and no durable accounting to reconcile. A process
    /// killed at any point leaves a namespace indistinguishable from one where
    /// the collection had simply been given a looser bound, and the kernel
    /// releases its lock when its last descriptor closes — the same mechanism,
    /// and the same absence of a stale-lock rule, the per-key lock relies on
    /// for a killed writer.
    ///
    /// # Errors
    ///
    /// Returns [`CacheUnavailable`] only when the scan itself cannot proceed.
    /// A failure on one key is recorded in [`CollectionReport::failed`] and the
    /// collection continues.
    pub fn collect(&self, bound: &CollectionBound) -> Result<CollectionReport, CacheUnavailable> {
        self.collect_at(bound, SystemTime::now())
    }

    /// [`Self::collect`], with the instant every age is measured against supplied
    /// rather than read from the host clock.
    ///
    /// The seam exists so an age test can be a statement about the predicate
    /// instead of about how fast a test ran: an entry exactly at the boundary,
    /// and an entry dated after the collecting process's own reading, are both
    /// unreachable through a wall-clock `now` without a margin, and a margin is
    /// what turns a deterministic test into a dice roll. The public entry point
    /// above pins `now` to [`SystemTime::now`], so no caller can collect against
    /// an instant of its own choosing.
    pub(crate) fn collect_at(
        &self,
        bound: &CollectionBound,
        now: SystemTime,
    ) -> Result<CollectionReport, CacheUnavailable> {
        let accounting = self.account()?;
        let order = CollectionOrder::OldestPublicationFirst;

        let selected = select(&accounting, *bound, now);
        let expired_selected = selected
            .iter()
            .filter(|entry| entry.reason == RemovalReason::OlderThanMaxEntryAge)
            .count();
        let mut report = CollectionReport {
            bound: *bound,
            order,
            selected: selected.len() as u64,
            removed: Vec::new(),
            contended: 0,
            superseded: 0,
            already_absent: 0,
            failed: Vec::new(),
            outcome: CollectionOutcome::WithinBound,
            accounting,
        };

        for entry in &selected {
            match self.remove_if_unchanged(&entry.fact) {
                Ok(Disposition::Removed) => report.removed.push(RemovedEntry {
                    key: entry.fact.key,
                    bytes: entry.fact.bytes,
                    reason: entry.reason,
                }),
                Ok(Disposition::Contended) => report.contended += 1,
                Ok(Disposition::Superseded) => report.superseded += 1,
                Ok(Disposition::AlreadyAbsent) => report.already_absent += 1,
                Err(unavailable) => report.failed.push(unavailable),
            }
        }

        // The final state is computed from what was *actually* removed rather
        // than from what selection projected, because a contended or superseded
        // key leaves bytes the projection had already spent.
        let retained_bytes = report
            .accounting
            .total_bytes
            .saturating_sub(report.reclaimed_bytes());
        let retained_entries = report
            .accounting
            .entry_count()
            .saturating_sub(report.removed.len() as u64);
        // An expired entry that was not removed still violates the age ceiling,
        // and the aggregate figures cannot say so — they are totals, and a cache
        // inside both of them can still hold an entry older than the stated age.
        // Crediting only actual removals matches how the byte figure above
        // treats `already_absent`: the disposition that resolved the violation
        // belongs to whoever performed it, and a caller re-runs rather than
        // trusts a projection.
        let expired_removed = report
            .removed
            .iter()
            .filter(|entry| entry.reason == RemovalReason::OlderThanMaxEntryAge)
            .count();
        report.outcome = if selected.is_empty() {
            CollectionOutcome::WithinBound
        } else if bound.admits(retained_bytes, retained_entries)
            && expired_removed == expired_selected
        {
            CollectionOutcome::BoundReached
        } else {
            CollectionOutcome::BoundNotReached {
                bytes: retained_bytes,
                entries: retained_entries,
            }
        };
        Ok(report)
    }

    /// Removes one entry under its key lock, and only if it is the entry the
    /// scan measured.
    ///
    /// # What the comparison establishes
    ///
    /// The lock alone is not enough. A key may be republished between the scan
    /// that selected it and the moment the lock is taken, and removing the
    /// replacement would credit the report with bytes belonging to a file it
    /// never saw. Comparing the length and modification time against the scan is
    /// what keeps the report true.
    ///
    /// **It is a report-accuracy check and not a correctness boundary, and the
    /// distinction is worth stating because the comparison is not airtight.** A
    /// replacement that happened to match both the length and the modification
    /// time of what the scan saw would be removed as though it were the original.
    /// The consequence is one entry rebuilt that need not have been, which is
    /// the cost every eviction already carries; no reader can be given wrong
    /// bytes by it, because removal only ever produces an absence.
    pub(crate) fn remove_if_unchanged(
        &self,
        fact: &EntryFact,
    ) -> Result<Disposition, CacheUnavailable> {
        // A cache that stores nothing produces no [`EntryFact`], so a selection
        // over it is empty and this is unreachable from [`Self::collect`].
        // Reported rather than asserted: the disposition is exactly true — there
        // was no entry — and a panic would be the wrong failure mode for a seam
        // whose whole contract is that a removal it cannot perform is named.
        let Some(layout) = self.layout() else {
            return Ok(Disposition::AlreadyAbsent);
        };
        let lock_path = layout.lock_path(&fact.key);
        // Not `prepare_directories`: a collector creates nothing. A key with a
        // published entry already has its lock shard, and one that does not is
        // reported rather than repaired.
        let lock = match KeyLock::try_acquire(&lock_path) {
            Ok(Some(lock)) => lock,
            Ok(None) => return Ok(Disposition::Contended),
            Err(error) => {
                return Err(CacheUnavailable::new(
                    CacheOperation::AcquireLock,
                    lock_path,
                    error,
                ));
            }
        };

        let entry = layout.entry_path(&fact.key);
        let disposition = match fs::metadata(&entry) {
            Ok(metadata)
                if metadata.len() == fact.bytes && metadata.modified().ok() == fact.published =>
            {
                match fs::remove_file(&entry) {
                    Ok(()) => Disposition::Removed,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        Disposition::AlreadyAbsent
                    }
                    Err(error) => {
                        return Err(CacheUnavailable::new(
                            CacheOperation::RemoveEntry,
                            entry,
                            error,
                        ));
                    }
                }
            }
            Ok(_) => Disposition::Superseded,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Disposition::AlreadyAbsent,
            Err(error) => {
                return Err(CacheUnavailable::new(
                    CacheOperation::ScanDirectory,
                    entry,
                    error,
                ));
            }
        };
        // Released by dropping the descriptor, on every path including the early
        // returns above — and deliberately *not* through `KeyLock::release`,
        // which is what the sibling `evict` uses. `release` reports an unlock
        // error, and reporting one here would reclassify an entry this call had
        // already removed as a failure, so the report would under-count the
        // bytes it reclaimed and name a removal that happened as one that did
        // not. The removal is the fact worth recording; the unlock is a
        // descriptor closing, which the kernel does whether or not anyone asks.
        //
        // The lock *file* is retained, always. Unlinking a locked file lets a
        // later process create a different inode at the same path and take an
        // independent lock while an earlier process still holds the first, which
        // splits contenders into two groups that do not exclude each other.
        drop(lock);
        Ok(disposition)
    }

    /// Takes the whole namespace out of service in one rename, then reclaims it.
    ///
    /// # Why a rename rather than a recursive delete
    ///
    /// The research note requires a whole-cache purge to "either require
    /// quiescence or rename the version root out of service", and quiescence
    /// does not survive contact with the problem: no code here can establish
    /// that no other process is using a configured root, so a purge that
    /// required it would be promising something it cannot check.
    ///
    /// The rename can be checked. It is one atomic operation after which
    /// `<root>/v1` does not exist, so a process arriving afterwards creates a
    /// fresh, coherent namespace and never walks a half-deleted one. That is
    /// strictly more than `rm -r` offers: a recursive delete races directory
    /// creation, and — as the harness records — can unlink a *live lock inode*,
    /// which is the failure the layout module explains, splitting
    /// contenders into two groups that do not exclude each other.
    ///
    /// # What it deliberately does not promise
    ///
    /// **Compile-once suppression does not survive a purge, and this must not
    /// claim otherwise.** A process holding a lock in the retired tree and a
    /// process taking a lock in the new one are locking two different inodes and
    /// do not exclude each other, so a compilation may be duplicated across the
    /// rename. Correctness is unaffected: writers in the retired tree still
    /// publish validated entries, into a tree that is then discarded, and each
    /// returns a validated artifact. This is the same bound the note places on
    /// external deletion, and a Tiler-provided purge must not promise what
    /// `rm -r` cannot.
    ///
    /// # Dying part-way through
    ///
    /// Two states, both safe. Before the rename, nothing happened. After it, the
    /// tree is out of service — invisible to every reader, because the layout joins the version component
    /// exactly —
    /// and a later purge reclaims it. There is no third state, because the
    /// rename is atomic and the removal that follows it operates on something
    /// nothing else reads.
    ///
    /// # Errors
    ///
    /// Returns [`CacheUnavailable`] when the cache root exists and cannot be
    /// listed, or when the live namespace exists and cannot be retired. A tree
    /// that is retired but cannot be removed is recorded in
    /// [`PurgeReport::failed`] and does not fail the purge, because it is
    /// already out of service. A cache built by [`ExpansionCache::disabled`] has
    /// no namespace to retire and reclaims nothing.
    pub fn purge(&self) -> Result<PurgeReport, CacheUnavailable> {
        let Some(layout) = self.layout() else {
            return Ok(PurgeReport::default());
        };
        let mut report = PurgeReport::default();
        let version_root = layout.version_root();

        match fs::metadata(&version_root) {
            Ok(_) => {
                let nonce = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |elapsed| elapsed.as_nanos());
                let retired = layout.out_of_service_path(nonce);
                match fs::rename(&version_root, &retired) {
                    Ok(()) => report.retired = Some(retired),
                    // Retired by a concurrent purge between the `stat` and the
                    // rename. Nothing is left in service, which is the outcome
                    // this call wanted.
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(CacheUnavailable::new(
                            CacheOperation::RetireNamespace,
                            version_root,
                            error,
                        ));
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(CacheUnavailable::new(
                    CacheOperation::ScanDirectory,
                    version_root,
                    error,
                ));
            }
        }

        // Reclaim every retired tree, including any an earlier purge left behind
        // by dying between its rename and its removal.
        let listing = match fs::read_dir(layout.root()) {
            Ok(listing) => listing,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(report),
            Err(error) => {
                return Err(CacheUnavailable::new(
                    CacheOperation::ScanDirectory,
                    layout.root().to_path_buf(),
                    error,
                ));
            }
        };
        for found in listing {
            let Ok(found) = found else { continue };
            let name = found.file_name();
            let Some(name) = name.to_str() else { continue };
            if !Layout::is_out_of_service(name) {
                continue;
            }
            let path = found.path();
            let bytes = tree_bytes(&path);
            match fs::remove_dir_all(&path) {
                Ok(()) => {
                    report.reclaimed_trees += 1;
                    report.reclaimed_bytes = report.reclaimed_bytes.saturating_add(bytes);
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => report.failed.push(CacheUnavailable::new(
                    CacheOperation::RemoveRetired,
                    path,
                    error,
                )),
            }
        }
        Ok(report)
    }
}

/// What became of one selected entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Disposition {
    /// The entry the scan measured was removed.
    Removed,
    /// Another process held the key's lock.
    Contended,
    /// The entry changed after the scan measured it.
    Superseded,
    /// No entry was there when the lock was taken.
    AlreadyAbsent,
}

/// One entry a bound selected, and which ceiling selected it.
#[derive(Clone, Debug)]
struct Selected {
    /// The entry as the scan measured it, which is what the locked removal
    /// re-`stat`s against.
    fact: EntryFact,
    /// The ceiling that put it here.
    reason: RemovalReason,
}

/// Chooses the entries a bound requires removing, oldest publication first.
///
/// Selection is arithmetic over the scan and touches no file, which is what lets
/// the removal loop above be the only place a decision becomes destructive. It
/// runs in two passes because the ceilings are two different kinds of statement:
/// the age is a predicate over one entry's own evidence, and the byte and entry
/// ceilings are properties of a total. Taking the age first is what makes the
/// aggregate pass operate on the totals the age will actually leave behind,
/// rather than removing an entry to fit a ceiling that expiry was about to
/// satisfy anyway.
fn select(accounting: &CacheAccounting, bound: CollectionBound, now: SystemTime) -> Vec<Selected> {
    // Nothing to decide when no ceiling exists at all, and — when only the
    // aggregate ceilings were stated — when the cache already fits inside them.
    // A stated age has to be walked regardless of the totals, because a cache
    // within every aggregate ceiling can still hold an expired entry.
    if bound.is_unbounded()
        || (bound.max_entry_age.is_none()
            && bound.admits(accounting.total_bytes(), accounting.entry_count()))
    {
        return Vec::new();
    }
    let mut ordered = accounting.entries.clone();
    // Oldest publication first, with an undatable entry treated as *newest* so
    // it is never selected ahead of one that can be dated. `Option`'s own
    // ordering puts `None` first, which is the opposite direction, so the
    // comparison is written out rather than derived.
    //
    // Ties break on the key, so selection is deterministic rather than dependent
    // on the order a directory listing happened to return — two runs over one
    // cache must choose the same entries.
    ordered.sort_by(|left, right| {
        let by_age = match (left.published, right.published) {
            (Some(left), Some(right)) => left.cmp(&right),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (None, None) => Ordering::Equal,
        };
        by_age.then_with(|| left.key.cmp(&right.key))
    });

    let mut bytes = accounting.total_bytes();
    let mut entries = accounting.entry_count();
    let mut selected = Vec::new();
    let mut survived_the_age = Vec::new();
    for fact in ordered {
        let expired = bound
            .max_entry_age
            .is_some_and(|max_age| max_age.has_expired(fact.published, now));
        if expired {
            bytes = bytes.saturating_sub(fact.bytes);
            entries = entries.saturating_sub(1);
            selected.push(Selected {
                fact,
                reason: RemovalReason::OlderThanMaxEntryAge,
            });
        } else {
            survived_the_age.push(fact);
        }
    }
    for fact in survived_the_age {
        if bound.admits(bytes, entries) {
            break;
        }
        bytes = bytes.saturating_sub(fact.bytes);
        entries = entries.saturating_sub(1);
        selected.push(Selected {
            fact,
            reason: RemovalReason::OverSizeCeiling,
        });
    }
    selected
}

/// Lists the shard directories of one namespace tree.
///
/// An absent tree is an empty cache rather than an error: [`ExpansionCache`]
/// creates nothing when it is opened, so a root that has never been published to
/// is the ordinary starting state.
fn shards(root: &Path) -> Result<Vec<PathBuf>, CacheUnavailable> {
    let listing = match fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(CacheUnavailable::new(
                CacheOperation::ScanDirectory,
                root.to_path_buf(),
                error,
            ));
        }
    };
    let mut shards = Vec::new();
    for found in listing {
        let found = found.map_err(|error| {
            CacheUnavailable::new(CacheOperation::ScanDirectory, root.to_path_buf(), error)
        })?;
        if found.path().is_dir() {
            shards.push(found.path());
        }
    }
    Ok(shards)
}

/// Lists the files of one shard directory.
fn files(shard: &Path) -> Result<Vec<PathBuf>, CacheUnavailable> {
    let listing = match fs::read_dir(shard) {
        Ok(listing) => listing,
        // Removed by a concurrent purge or an external deletion while this scan
        // was walking. An absent shard contributes nothing.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(CacheUnavailable::new(
                CacheOperation::ScanDirectory,
                shard.to_path_buf(),
                error,
            ));
        }
    };
    let mut files = Vec::new();
    for found in listing {
        let found = found.map_err(|error| {
            CacheUnavailable::new(CacheOperation::ScanDirectory, shard.to_path_buf(), error)
        })?;
        files.push(found.path());
    }
    Ok(files)
}

/// Sums the bytes of one directory tree, best effort.
///
/// Feeds a reclamation figure in a report. An unreadable directory counts as
/// zero rather than failing, because a tree that is already out of service must
/// be removed whether or not it can be measured.
fn tree_bytes(root: &Path) -> u64 {
    let Ok(listing) = fs::read_dir(root) else {
        return 0;
    };
    let mut total = 0_u64;
    for found in listing.flatten() {
        let Ok(metadata) = found.metadata() else {
            continue;
        };
        total = total.saturating_add(if metadata.is_dir() {
            tree_bytes(&found.path())
        } else {
            metadata.len()
        });
    }
    total
}
