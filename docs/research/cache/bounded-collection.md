---
schema: "tiler-doc/v1"
id: "tiler.research.cache.bounded-collection"
kind: "research"
title: "Bounded expansion cache collection and accounting"
topics: ["cache", "artifacts", "concurrency", "durability"]
catalog_group: "artifacts-build-toolchains"
research_status: "complete"
disposition: "pending"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis", "bounded-measurement"]
informs: ["tiler.contract.artifact-abi", "tiler.contract.metal-backend"]
depends_on: ["tiler.research.cache.crash-race-protocol"]
ticket: "design-bounded-expansion-cache-garbage-collection"
---

# Bounded expansion cache collection and accounting

Status: the sixth follow-up gate of the [crash and race protocol note](crash-and-race-protocol.md). The design below is derived from the five correctness properties `AGENTS.md` states for cache work, implemented in `crates/tiler-cache/src/expansion/collect.rs`, and measured under load at 1, 8, and 32 processes. It is **staged crate-private** under ADR 0074 convention 7: no consumer can reach it until Tom accepts the facade.

## What this gate owed, and what closes it

**Fact.** The note's gate 6 asks to "design bounded GC/accounting separately and stress eviction with active writers/readers at 1, 8, and 32 processes", and its garbage-collection section states six rules. Rules 1 to 4 were already implemented before this ticket — `evict` takes the per-key lock and retains the lock file, `sweep_temporaries` removes abandoned temporaries under the same lock outside a grace period, and a lock-free reader validates through a descriptor it opened itself. Rules 5 and 6 — disposable accounting, and bounded work and storage policies — had no implementation, and `Limits` deliberately carried no maximum entry count because bounding the count means choosing which entry to evict.

**Fact.** That choice is what this note makes, and the answer to "which entry" turns out to matter less than the answer to "under what bound, and who says so".

## The bound is a declared policy with no default

**Inference.** The repository rule is that nothing may be dropped or silently truncated: fail loud, or remove the limit if it is not needed. For a collector that cannot mean "never collect", so it means the boundary must be explicit, stated, and observable. Three candidate defaults were tested against it and two do not survive:

- **A byte or entry ceiling with a default value.** Eliminated. The note itself says "exact defaults require workload measurement", so any value chosen now is a guess; and a default bound deletes a user's compiled artifacts on the strength of that guess, invisibly, presenting as "the build got slower" rather than "your cache was collected". This is the failure the rule names.
- **No collection mechanism at all.** Eliminated. The gate exists, an unbounded cache on a developer machine is a real operational problem, and `Limits` already records that the entry-count bound is held open *by this ticket* — returning "no policy" would not discharge it.
- **Unbounded by default, with an opt-in declared bound and a report naming every removal.** Survives, and is what is implemented.

**Proposal.** `CollectionBound` carries two optional ceilings, `max_total_bytes` and `max_entries`, and `CollectionBound::UNBOUNDED` — which is also its `Default` — removes nothing. Collecting under it is a pure measurement. An entry can therefore never leave because of a number nobody chose.

**Proposal.** Every removal is named. `CollectionReport::removed` lists each removed entry individually with its key and the bytes it occupied, rather than a count, because a count cannot answer the only question an operator asks after an unexpected rebuild. `CollectionReport::accounts_for_every_entry` is the structural form of the rule: the five dispositions — removed, contended, superseded, already absent, failed — are disjoint and total over the selection, so an entry cannot leave without a line in the report that removed it. The collecting process asserts it on every round.

**Proposal.** A collection that cannot reach its bound reports `CollectionOutcome::BoundNotReached` carrying the bytes and entries still held. Stopping short is a legitimate outcome — a contended key is skipped by design — and it is reported rather than presented as success.

## Accounting is separate, and deliberately not durable

**Proposal.** `ExpansionCache::account` scans and changes nothing, so an operator measures before deciding. It reports entry count, total bytes, the per-entry facts, files the entry-path parser refused, and quarantined bytes.

**Inference.** Nothing it produces is written to disk, and that refusal is what makes the crash story trivial rather than being a mere restatement of the note's rule 5. A durable index of sizes and recency would, after a crash, disagree with the filesystem; reconciling them needs a repair rule; and a repair rule that is wrong either deletes live entries or trusts a stale size. There is no such rule here because there is no such index. The filesystem is the only authority on which entries exist, and a scan is how the collector asks it.

**Proposal.** Quarantined bytes are counted and never collected. Quarantine holds the exact bytes of entries that failed validation, which is evidence; its growth is already bounded where it is *added* to, and reaching that bound is reported as `QuarantineOutcome::BoundReached` rather than silently discarding. Reclaiming retained evidence would be a separate explicit act, and this design does not perform it.

## Selection order, and the one it cannot choose yet

**Proposal.** `CollectionOrder::OldestPublicationFirst` orders by the entry file's modification time, which publication sets as a side effect of writing the temporary that is renamed into place. It costs no extra I/O and no reader ever writes to obtain it.

**Fact.** This is insertion recency, not use recency, and the difference has a known cost: an entry hit on every build is never rewritten, so it ages exactly like one nobody has wanted since the day it was published. Under a tight bound a stable working set can be evicted and rebuilt.

**Inference.** That is a performance pathology and not a correctness one — every eviction costs at most a recompilation — and it is the price of refusing to make readers write. The two alternatives were eliminated:

- **Least recently used**, maintained by a sidecar the reader updates. Eliminated for now: it puts a write on the deliberately lock-free hit path, adds a failure mode to the operation the cache exists to make fast, and creates a partial-sidecar crash surface, all to improve a metric whose worst case is a recompilation.
- **Largest first.** Eliminated: the cost of a wrong eviction is one recompilation regardless of the entry's size, so ordering by size optimizes a metric nobody pays.

**Deferred question, now closed negatively.** Whether the filesystem's *access* time can supply use recency for free was deferred to `define-supported-expansion-cache-filesystems`, on the trigger of that ticket naming a supported set on which `atime` is maintained with useful granularity. [The supported-filesystem note](supported-filesystems.md) answers no, and for a reason stronger than the `noatime` mount that motivated the deferral: on both measured filesystems `atime` is maintained under a `relatime`-like predicate or not at all, and a cache entry's modification time never changes after publication, so **its access time advances at most once in its whole life — at its first read — and never again.** It is a boolean "read at least once since published", which is precisely the distinction least-recently-used ordering exists *not* to be. `OldestPublicationFirst` therefore stands as the only order, and the pathology above stands with it rather than being hidden.

## No work budget, because the collector never blocks

**Fact.** The note's rule 6 asks for "a best-effort cleanup budget per invocation".

**Inference.** A budget's real purpose would be to cap an unbounded wait: each removal takes the entry's key lock, and a lock held by a live writer can be held for the length of an external compilation. A collector blocking on hundreds of contended keys would stall arbitrarily.

**Proposal.** The collector takes each key lock with `try_lock` and never waits. A held key lock is positive evidence that a process is publishing or evicting that key *right now*, which makes the entry live — the opposite of a collection candidate — so skipping it is better selection as well as better latency. With no blocking wait there is nothing to cap: a collection's latency is its scan plus one non-blocking lock attempt per candidate. The limit is removed rather than given a number, which is what the repository rule prefers.

**Inference.** The cost is that a heavily contended cache may not reach its bound in one pass. That is reported as `BoundNotReached` with the counts, so a caller re-runs rather than wonders.

## Who runs it, and when

**Proposal.** Never automatically, and never on the expansion path. Collection is an explicit call returning a report. The alternatives were eliminated:

- **Inside `get_or_publish` on a miss.** Eliminated: it puts a walk of every shard on the path the cache exists to make fast, and runs it hardest when the cache is coldest — most misses means most scans, multiplied by every building process.
- **A background thread the cache spawns.** Eliminated: it starts threads inside a compiler process nobody asked to be concurrent, has no lifetime in a process that may exit the moment expansion finishes, and returns its report to nobody.
- **On a fraction of publications.** Eliminated: it makes the trigger unexplainable. "Why did my entry go away" would answer "a random draw during an unrelated build", and a bound must have a trigger a person can name.

**Deferred question.** What schedules a collection in production. **Trigger for reconsideration:** the arrival of a proc-macro frontend or a maintenance command — a caller that exists. `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` is where the process-pattern evidence for choosing one will come from. **What this design assumes meanwhile:** nothing schedules it, so a cache grows until someone measures and collects it deliberately.

## The whole-cache purge

**Fact.** The note requires a purge to "either require quiescence or rename the version root out of service", and records that an arbitrary external recursive deletion loses compile-once suppression while preserving correctness.

**Inference.** Quiescence does not survive contact with the problem: no code can establish that no other process is using a configured root, so a purge requiring it would be promising something it cannot check. Only the rename form remains, which is why this is not presented as a choice.

**Proposal.** `ExpansionCache::purge` renames `<root>/v1` to `<root>/v1.out-of-service.<nonce>` in one atomic operation, then removes the retired tree. Because `Layout` joins the version component exactly, nothing this crate reads can resolve into a retired tree — a reader does not have to be told the namespace changed, because the directory it looks in is simply no longer the one holding the old entries.

**Inference.** This is strictly stronger than `rm -r`: after the rename a process arriving next creates a fresh, coherent namespace rather than walking a half-deleted one, and no live lock inode is ever unlinked — which is the failure that splits contenders into two groups that do not exclude each other.

**Proposal.** What it must not promise, and does not: compile-once suppression across the purge. A process holding a lock in the retired tree and one taking a lock in the new tree lock two different inodes and do not exclude each other, so a compilation may be duplicated across the rename. Correctness is unaffected — writers in the retired tree still publish validated entries into a tree that is then discarded, and each returns a validated artifact. A Tiler-provided purge must not promise what `rm -r` cannot.

**Proposal.** A purge that dies has two states and both are safe. Before the rename, nothing happened. After it, the tree is out of service and invisible, and a later purge reclaims it — recovery needs no rule beyond "reclaim what is out of service", because nothing reads a retired tree.

## How each of the five properties survives collection

**Inference**, in each case from the mechanism named, not from intent.

| Property | How collection preserves it |
| --- | --- |
| Complete cache and artifact identity | Collection never derives a key from a subject, writes an entry, or composes anything. It reads keys *out of paths* through the existing entry-path parser, which refuses a label of the wrong width, a non-lowercase-hexadecimal label, and an entry under a shard that is not its own. A file that parser refuses is **not removed** and is reported as unrecognized: a collector deleting what it could not parse would be acting on the absence of understanding. |
| Validation on every hit | The read path is untouched and unreachable from the collector, which produces no entry. The complete set of transitions collection can cause at a content path is `Hit -> Absent`; it only unlinks. It cannot turn a rejection into a hit, nor an absence into one. |
| Immutable entries | The collector never holds a writable descriptor to an entry. Its only mutating operation on a content path is `remove_file`; accounting reads metadata, never bytes. |
| Atomic publication | The collector publishes nothing, and must not undo a publication it never measured. A selected entry is removed only under its own key lock and only after a re-`stat` agrees with the scan on length and modification time; a replacement is left alone and counted as superseded. |
| Crash and race behaviour | Stated below. |

**Measurement boundary.** The re-`stat` is a *report-accuracy* check, not a correctness boundary, and it is not airtight: a replacement matching both the length and the modification time the scan saw would be removed as though it were the original. The consequence is one entry rebuilt that need not have been — the cost every eviction already carries — and no reader can be given wrong bytes by it, because removal only ever produces an absence.

## Racing a reader, and dying mid-collection

**Inference.** A reader occupies exactly one of three positions when a removal lands, and the enumeration is total:

1. **It has already returned.** `lookup` copies the validated envelope into the `CachedEntry` it returns, so a caller holding one owns its bytes and the file is irrelevant to it.
2. **It has opened the entry and is still reading.** The descriptor was opened before the unlink, and Darwin's `unlink(2)` defers reclaiming the file while a process still has it open. The read completes and yields exactly the published bytes.
3. **It has not opened yet.** `File::open` reports the entry absent, which is `MissReason::Absent` — the one miss the reporting module calls "not evidence of a problem" — and the caller rebuilds.

There is no fourth position and no window in which a reader observes a partially collected entry, because `unlink` of one file has no intermediate state a reader can name.

**Inference.** A process killed mid-collection needs no recovery, and this is a consequence of refusing durable accounting rather than a separate design step. A collection is a sequence of independent single-file unlinks, each under its own lock, with no journal, no in-progress marker, and nothing to reconcile. The namespace left behind is indistinguishable from one where the collection had been given a looser bound, and the kernel releases the lock when the last descriptor closes — the same mechanism, and the same absence of a stale-lock rule, that covers a killed writer.

## Measurement

**Measurement.** `expansion::harness::collection_races_active_processes_at_one_eight_and_thirty_two` runs the ladder the gate asks for. At each of 1, 8, and 32 writer processes it seeds eight entries, then races real publishing processes against a real collecting process holding `max_entries = 2` for twelve rounds, while the parent reads throughout. Half the writers share one key, which keeps a lock held almost continuously; half take their own, which keeps new entries appearing under the collector. Four things are asserted at every scale:

1. every child completes — a child killed at its deadline would mean a collector and a writer deadlocked, which is what non-blocking locks exclude;
2. every collection accounted for its whole selection, asserted inside the collecting process on every round;
3. something was actually removed, so the case cannot pass vacuously;
4. everything that survived still validates, walked from the namespace rather than from a list of remembered keys.

**Measurement.** A descriptor opened before the race and read after it yields exactly the published bytes, across a real process boundary, which is position 2 above measured rather than argued.

**Measurement boundary.** This is a bounded measurement on one host and is not a portable guarantee. It does not cover power loss, storage-controller reordering, a network filesystem, disk-full partial writes, or Windows deletion semantics. How *often* the contended, superseded, and already-absent dispositions are reached under load is a property of the host's scheduling; the ladder records the totals in its evidence row rather than asserting on them, and each disposition has deterministic coverage in `expansion::tests`, which holds the lock and replaces the entry itself instead of waiting for luck.

**Fact.** The threaded suite covers the selector, both ceilings, the named-removal invariant, the unrecognized-file rule, lock-file retention, concurrent collectors not double-counting, and every purge case including reclaiming a tree an earlier purge left behind.

## What this design does not settle

- **The default durability policy** is unchanged and still `measure-expansion-cache-durability-policies`'s. Collection is orthogonal to it: no ordering or flushing decision changes what a removal does.
- **Supported filesystems**, and with them whether `atime` could supply use recency, are settled by [the supported-filesystem note](supported-filesystems.md); the access-time answer is no, and the ordering above is unchanged by it.
- **Framing fuzzing** and **deterministic I/O fault injection** remain `fuzz-the-expansion-cache-framing-paths` and `inject-deterministic-expansion-cache-io-failures`. The collector reads no entry bytes, so the framing paths it touches are the scan's `stat` calls rather than the decoder.
- **The public facade.** Every type here is `pub(crate)` under ADR 0074 convention 7. Promoting it is `accept-the-tiler-cache-public-boundary`, and until that happens no consumer can collect anything.

## Traceability

Closes gate 6 of the [crash and race protocol note](crash-and-race-protocol.md). Preserves ADR 0050's garbage-collection sentence — "Internal GC retains lock files and takes the key lock before eviction" — and the identical statements in [`docs/artifact-abi.md`](../../artifact-abi.md) and [`docs/backends/metal.md`](../../backends/metal.md), none of which this design changes. Implemented in `crates/tiler-cache/src/expansion/collect.rs`.
