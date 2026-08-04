---
id: admit-an-age-bounded-automatic-eviction-into-the-expansion-cache
title: Admit an age-bounded automatic eviction into the expansion cache
status: in-progress
priority: p2
dependencies: []
related: [decide-the-expansion-cache-collection-schedule, wire-the-env-configured-eviction-policy-through-the-deliver-path, measure-the-expansion-cache-hot-path-efficiency]
scopes: [implementation/cache, research/cache]
shared_scopes: [project/tickets]
paths: []
tags: [cache, eviction, durability, decision-execution]
claimed_from: todo
assignee: agent-cache-evict
lease_expires_at: 1785875503
---
## User-visible outcome

The expansion cache can evict entries older than a caller-stated age, so a developer machine no longer accumulates entries forever, and the delivering frontend can invoke that eviction automatically under an environment-configured policy without `tiler-cache` ever reading the environment itself.

## The decision this executes, and what it supersedes

Tom decided on 2026-08-04 (recorded in [`decide-the-expansion-cache-collection-schedule`](decide-the-expansion-cache-collection-schedule.md)): no shipped maintenance CLI; automatic age-based eviction of old entries, customizable via environment variables. This explicitly supersedes the "a collection is never automatic" schedule conclusion of the bounded-collection design record (`docs/research/cache/bounded-collection.md`) — the first deliverable is that supersession written into the record, preserving the original rationale (it was correct against the alternatives it considered; the product owner has since weighted background hygiene above per-act attribution).

What the elimination established and this ticket must NOT weaken:

- **The collector's concurrency and crash safety are properties of the collector, not the schedule** (measured at 1/8/32 writer processes; `KeyLock::try_acquire`, re-`stat` before unlink, no journal). Age-based selection must go through the same locked, re-validated removal path.
- **Never on the hot path.** The `get_or_publish`-miss shard walk was refused on performance grounds and no new evidence reopens that. The eviction entry point is a separate call the frontend issues off the hit path (see the successor wiring ticket).
- **`tiler-cache` never reads the environment.** The age bound arrives as an explicit typed value (extend `CollectionBound` or add a sibling age policy type — the exact public shape is a draft for Tom under ADR 0074 §7). Environment-variable names, parsing, and defaults live with the frontend, which already owns environment reading under the ADR 0089 root policy.
- **Explainability survives automation.** `CollectionReport` already names every removal; the automatic caller decides what to do with it, but the mechanism must keep every removal attributable to the stated age policy. Fail closed on an unparseable or contradictory policy: refuse to evict, never guess.

## Implementation keys

- Age is measured from the entry's own filesystem evidence (the modification time the collector already re-`stat`s), not from a durable index — the design record's refusal of a second authority stands.
- A default age must exist for the zero-configuration case (Tom authorized defaults by choosing automation). Choose it explicitly, state its ground in the record (per-entry sizes 32–48 KB and ~10–20 MB per editing afternoon are the measured inputs), and make it overridable; document that the default is a product choice, not a measurement.
- Preserve `CollectionBound::UNBOUNDED` semantics for existing callers; an age bound composes with, not replaces, the size bound.
- Deliberate perturbations: an entry exactly at the age boundary; a clock that moves backwards (mtime in the future — must not panic, must not mass-evict); an eviction racing a publisher re-publishing the same key; a policy stating zero/negative age (typed refusal).

## Closes when

The age-bounded eviction is a tested draft on the public cache boundary (draft for Tom, not self-accepted), the bounded-collection record carries the explicit supersession with rationale preserved, all perturbations pass, and the wiring ticket can consume the typed policy without `tiler-cache` gaining any environment read.
