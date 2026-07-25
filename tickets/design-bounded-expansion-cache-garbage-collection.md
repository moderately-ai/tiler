---
id: design-bounded-expansion-cache-garbage-collection
title: Design bounded expansion cache garbage collection
status: done
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, implementation/cache]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [cache, durability, concurrency]
---
The research note's sixth follow-up gate: design bounded garbage collection and accounting separately, and stress eviction with active writers and readers at 1, 8, and 32 processes.

`tiler-cache` implements the two pieces that are *not* policy: `evict` takes the per-key lock before removing an entry and retains the lock file, and `sweep_temporaries` removes abandoned temporaries for one key under the same lock and outside a grace period. It deliberately implements **no whole-cache policy at all**, and `Limits` carries no maximum entry count — bounding the entry count means choosing which entry to evict, which is exactly the policy this gate holds open. A field recording a bound nothing enforced would read as a guarantee.

## What this ticket owes

- Decide the accounting: total bytes, entry count, recency, and the best-effort work budget per invocation, kept as disposable metadata that never mutates bundle bytes and is never trusted for hit correctness.
- Decide who runs it and when. An expansion-time collection competes with the compilation it is meant to accelerate.
- Stress it against active writers and readers at 1, 8, and 32 processes, which the existing harness structure already supports.
- A whole-cache purge must either require quiescence or rename the version root out of service; the harness already shows that an arbitrary external recursive deletion loses compile-once suppression while preserving correctness, and a Tiler-provided purge must not promise what `rm -r` cannot.

## Outcome

Closed. The design is [`docs/research/cache/bounded-collection.md`](../docs/research/cache/bounded-collection.md), implemented in `crates/tiler-cache/src/expansion/collect.rs` and staged `pub(crate)` under ADR 0074 convention 7. It is a **correctness-derived contract update** — each decision is derived from one of the five properties `AGENTS.md` names — carrying **a bounded measurement** (the 1/8/32 ladder) and **two explicitly deferred questions with triggers**.

**Accounting.** Total bytes, entry count, per-entry facts, unrecognized files, and quarantined bytes, computed per invocation by `ExpansionCache::account` and never written to disk. Refusing a durable index is what makes the crash story trivial: there is no journal to reconcile, so a killed collector needs no repair rule, and a repair rule that was wrong would either delete live entries or trust a stale size.

**The bound.** `CollectionBound` has two optional ceilings and its default removes nothing. A default value was eliminated because the research note says exact defaults require workload measurement, and a guessed default deletes a user's artifacts invisibly. Every removal is named individually; `accounts_for_every_entry` makes the five dispositions disjoint and total over the selection, asserted inside the collecting process on every round.

**No work budget.** Removed rather than given a number. The collector takes each key lock with `try_lock` and never waits, so there is no unbounded wait to cap — and a held key lock is positive evidence the entry is live, making a contended key a bad candidate as well as a slow one.

**Order.** `OldestPublicationFirst`, by the entry's modification time, which publication sets for free. This is insertion recency, not use recency, and the pathology is stated rather than hidden: a stable working set can be evicted and rebuilt. Least-recently-used was eliminated because it puts a write on the deliberately lock-free hit path.

**Who runs it.** Never automatically and never on the expansion path. Collecting on a miss, on a background thread, and on a fraction of publications were each eliminated with a derivation.

**Purge.** `ExpansionCache::purge` renames `<root>/v1` out of service in one atomic operation and reclaims it. Requiring quiescence was eliminated as unverifiable. It promises strictly more than `rm -r` — no reader ever walks a half-deleted namespace and no live lock inode is unlinked — and explicitly does not promise compile-once across the rename.

**Stress.** `expansion::harness::collection_races_active_processes_at_one_eight_and_thirty_two` runs the ladder the gate asked for, asserting liveness, the disposition partition, that something was actually removed, and that everything surviving still validates. A descriptor opened before the race and read after it still yields the published bytes.

## Deferred, with triggers

- **Use-recency ordering** → `define-supported-expansion-cache-filesystems`. Trigger: that ticket naming a supported filesystem set on which `atime` is maintained with useful granularity. Meanwhile the design assumes insertion recency only.
- **What schedules a collection** → `decide-the-expansion-cache-collection-schedule` (deferred). Trigger: a real caller — a proc-macro frontend or a maintenance command. Meanwhile nothing schedules one.

## Reserved for Tom

The public facade. Every collection type is `pub(crate)`; `accept-the-tiler-cache-public-boundary` now lists the exact surface, including that `CacheOperation` — deliberately not `#[non_exhaustive]` — gained two variants.
