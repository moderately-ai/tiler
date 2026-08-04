---
id: decide-the-expansion-cache-collection-schedule
title: Decide what schedules an expansion cache collection
status: in-progress
priority: p3
dependencies: [accept-the-tiler-cache-public-boundary]
related: [design-bounded-expansion-cache-garbage-collection, exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [research/cache, implementation/cache]
shared_scopes: []
paths: []
tags: [cache, durability, concurrency]
claimed_from: todo
assignee: agent-cache-schedule
lease_expires_at: 1785874216
---
`design-bounded-expansion-cache-garbage-collection` decided that a collection is **never automatic and never on the expansion path**: it is an explicit call returning a report, because a bound has to have a trigger a person can name. It eliminated collecting inside `get_or_publish` on a miss (a walk of every shard on the path the cache exists to make fast, run hardest when the cache is coldest), a background thread the cache spawns (threads inside a compiler process nobody asked to be concurrent, no lifetime in a process that may exit immediately, and a report returned to nobody), and collecting on a fraction of publications (an unexplainable trigger).

What it deliberately did not decide is **what calls it in production**, because no caller exists: there is no proc-macro frontend and no maintenance command, and choosing a schedule against an imagined consumer is designing for a caller nobody can see. So today nothing schedules a collection and a cache grows until someone measures and collects it deliberately — which is recorded rather than hidden.

## Trigger for reconsideration

The arrival of a real caller: a proc-macro frontend, or a `tiler` maintenance command. Either makes the question answerable instead of hypothetical.

## What this ticket would then owe

- Decide whether the schedule is a user-invoked maintenance command, a periodic hook, a build-session boundary, or some combination, and state what each enables and prevents.
- Decide where the bound's *value* comes from — configuration, an environment variable, a default derived from a measured working-set size — noting that `design-bounded-expansion-cache-garbage-collection` refused to pick a default precisely because the note says exact defaults require workload measurement. A schedule that supplies a default bound re-opens that decision and must argue it rather than inherit it.
- Decide what surfaces the report. A collection that removes entries and reports to a log nobody reads is the silence this crate is built not to produce.
- Take the process-pattern evidence from `exercise-the-expansion-cache-under-cargo-and-rust-analyzer`, which is what establishes how many concurrent expansions a real Cargo and rust-analyzer session produces.

Depends on `accept-the-tiler-cache-public-boundary`: every collection type is `pub(crate)` under ADR 0074 convention 7, so nothing outside `tiler-cache` can call a collection at all until that facade is accepted.

## Activated 2026-08-02 — the trigger fired, and the paragraph above it is now false

**Do not read "no caller exists" forward.** The reconsideration trigger was "the arrival of a real caller: a proc-macro frontend, or a `tiler` maintenance command". A proc-macro frontend arrived, and it holds a cache.

**Fact.** `crates/tiler-macros/Cargo.toml:11` declares `proc-macro = true` and `:46` declares `tiler-cache.workspace = true`. `crates/tiler-macros/src/aot.rs:649` is `fn open_cache`, which returns `ExpansionCache::open(root)` or `ExpansionCache::disabled()` depending on the resolved cache-root decision. So the expansion path in production opens the cache this ticket schedules collection for.

**Fact — the closing caveat is also spent.** It said nothing outside `tiler-cache` can call a collection until the facade is accepted. `accept-the-tiler-cache-public-boundary` is `done`, and the API is public: `pub fn collect(&self, bound: &CollectionBound) -> Result<CollectionReport, CacheUnavailable>` at `crates/tiler-cache/src/expansion/collect.rs:628`, with `pub struct CollectionReport` at `:328`. Reproduce:

```sh
rg -n 'pub fn collect|pub struct CollectionReport' crates/tiler-cache/src/expansion/collect.rs
```

**Fact — the process-pattern evidence this ticket was told to take is available.** `exercise-the-expansion-cache-under-cargo-and-rust-analyzer` is `done`, so the concurrent-expansion counts *What this ticket would then owe* draws on exist rather than being awaited.

**Boundary — one part of this is Tom's and must not be self-answered.** Which schedule the collection gets — user-invoked maintenance command, periodic hook, build-session boundary, or a combination — is a product decision about the consumer's experience, and `design-bounded-expansion-cache-garbage-collection` deliberately refused to pick a default bound because exact defaults require workload measurement. Run the elimination against correctness, maintainability, and measured working-set size first and discard what fails; escalate only what genuinely survives, as one atomic question. Status moves `deferred` → `todo` because the elimination is now runnable, not because the answer is settled.
