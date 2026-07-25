---
id: decide-the-expansion-cache-collection-schedule
title: Decide what schedules an expansion cache collection
status: deferred
priority: p3
dependencies: [accept-the-tiler-cache-public-boundary]
related: [design-bounded-expansion-cache-garbage-collection, exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [research/cache, implementation/cache]
shared_scopes: []
paths: []
tags: [cache, durability, concurrency]
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
