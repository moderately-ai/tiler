//! Whole-cache accounting, bounded collection, and the out-of-service purge.
//!
//! # The bound is a declared policy, not a default
//!
//! A collector is a machine for removing things, so the repository rule that
//! nothing may be dropped or silently truncated does not read as "never
//! collect". It reads as: the boundary is explicit, stated, and observable. Two
//! decisions carry that here.
//!
//! **There is no default bound.** [`CollectionBound::UNBOUNDED`] removes
//! nothing, and it is the only bound this crate supplies. A byte or entry
//! ceiling exists exactly when a caller states one, so an entry can never vanish
//! because of a number nobody chose — the research note is explicit that "exact
//! defaults require workload measurement", and a default guessed ahead of that
//! measurement would delete a user's compiled artifacts on the strength of a
//! guess, invisibly. [`super::Limits`] therefore still carries no maximum entry
//! count: the bound is an argument to one explicit operation, not a property of
//! a cache that is otherwise only ever read from.
//!
//! **Every removal is named and every scanned entry is accounted for.**
//! [`CollectionReport::removed`] lists each removed entry individually with the
//! bytes it occupied, and [`CollectionReport::accounts_for_every_entry`] is the
//! structural form of the rule: the dispositions partition the scan exactly, so
//! an entry cannot leave the cache without appearing in the report that removed
//! it. Reaching a bound without satisfying it is
//! [`CollectionOutcome::BoundNotReached`] carrying what is left, never a quiet
//! stop.
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
//! **Never automatically, and never on the expansion path.** A collection is an
//! explicit call that hands its report back, and the alternatives were
//! eliminated rather than weighed:
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
//! What remains is an explicit operation, which is also what makes the trigger
//! statable: an entry leaves because somebody ran a collection under a bound
//! they chose, and the report says which entry, which bound, and which order.
//! Scheduling it stays with the caller. The delivering proc-macro path opens
//! this cache from `tiler_macros::aot::deliver`, and the elimination recorded
//! in `decide-the-expansion-cache-collection-schedule` leaves an explicit
//! invocation under a caller-stated bound as the only admissible schedule;
//! whether Tiler itself ships a maintenance command that issues one is an
//! undecided product question, so no schedule lives in this crate.

use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::key::CacheKey;
use super::layout::{self, Layout};
use super::lock::KeyLock;
use super::report::{CacheOperation, CacheUnavailable};
use super::store::ExpansionCache;

/// What one collection is allowed to leave behind.
///
/// Both ceilings are optional and both are absent by default. A collection under
/// [`Self::UNBOUNDED`] is a pure measurement: it selects nothing, removes
/// nothing, and reports the accounting it observed.
///
/// A caller-constructed leaf value record, so its fields are visible
/// (ADR 0074 convention 6).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct CollectionBound {
    /// Maximum total bytes of final entries the cache may retain.
    pub max_total_bytes: Option<u64>,
    /// Maximum number of final entries the cache may retain.
    pub max_entries: Option<u64>,
}

impl CollectionBound {
    /// The bound that removes nothing.
    ///
    /// The default, and the only bound this crate supplies. Collecting under it
    /// is how a caller measures a cache without changing it.
    pub const UNBOUNDED: Self = Self {
        max_total_bytes: None,
        max_entries: None,
    };

    /// True when a cache holding `bytes` across `entries` is within this bound.
    ///
    /// An absent ceiling constrains nothing, so an unbounded bound is satisfied
    /// by every cache and selects nothing.
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
        self.max_total_bytes.is_none() && self.max_entries.is_none()
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

/// One entry a collection removed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemovedEntry {
    /// The key whose entry was removed.
    pub key: CacheKey,
    /// Bytes reclaimed, as measured by the scan that selected it.
    pub bytes: u64,
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

    /// Every entry that was removed, individually, with the bytes it occupied.
    ///
    /// A list rather than a count. A removal is the destructive act, so a report
    /// that aggregated them away would be unable to answer the one question an
    /// operator asks after an unexpected rebuild — which entry left, and under
    /// which bound.
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
    /// The mechanical form of the rule that nothing leaves without being named:
    /// a selected entry is removed, contended, superseded, already absent, or
    /// failed, and those five are disjoint and total over the selection. A
    /// collection that dropped an entry it did not report would break this
    /// equality, which is why the tests assert it on every collection they make
    /// rather than only checking a removal count.
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
        let accounting = self.account()?;
        let order = CollectionOrder::OldestPublicationFirst;

        let selected = select(&accounting, *bound);
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

        for fact in &selected {
            match self.remove_if_unchanged(fact) {
                Ok(Disposition::Removed) => report.removed.push(RemovedEntry {
                    key: fact.key,
                    bytes: fact.bytes,
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
        report.outcome = if selected.is_empty() {
            CollectionOutcome::WithinBound
        } else if bound.admits(retained_bytes, retained_entries) {
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

/// Chooses the entries a bound requires removing, oldest publication first.
///
/// Selection is arithmetic over the scan and touches no file, which is what lets
/// the removal loop below be the only place a decision becomes destructive.
fn select(accounting: &CacheAccounting, bound: CollectionBound) -> Vec<EntryFact> {
    if bound.is_unbounded() || bound.admits(accounting.total_bytes(), accounting.entry_count()) {
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
    for fact in ordered {
        if bound.admits(bytes, entries) {
            break;
        }
        bytes = bytes.saturating_sub(fact.bytes);
        entries = entries.saturating_sub(1);
        selected.push(fact);
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
