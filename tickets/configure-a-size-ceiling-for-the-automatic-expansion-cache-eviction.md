---
id: configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction
title: Configure a size ceiling for the automatic expansion cache eviction
status: deferred
priority: p3
dependencies: []
related: [wire-the-env-configured-eviction-policy-through-the-deliver-path, admit-an-age-bounded-automatic-eviction-into-the-expansion-cache, measure-the-expansion-cache-hot-path-efficiency]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [cache, eviction, frontend, deferred]
---
## Why this is deferred rather than done

`wire-the-env-configured-eviction-policy-through-the-deliver-path` made exactly one ceiling configurable — `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, parsed into `CollectionBound { max_entry_age: Some(..), .. }` with both aggregate ceilings absent. `CollectionBound` carries two more, `max_total_bytes` and `max_entries`, and neither has an environment variable. That is a deliberate exclusion with a derivation, recorded here so the next reader does not read the absence as an oversight.

**The decision was age-based.** Tom decided on 2026-08-04 (`decide-the-expansion-cache-collection-schedule`) that "the cache evicts old entries automatically". An age is a per-entry predicate over the entry's own evidence; a byte or entry ceiling is a property of a total, and it selects victims by *publication* recency.

**Publication recency is a documented pathology, and an unmeasured one.** `CollectionOrder::OldestPublicationFirst`'s own documentation states it: "an entry hit on every build is never rewritten, so it ages exactly like one nobody has wanted since the day it was published; under a tight bound a stable working set can therefore be evicted and rebuilt". Under an age ceiling that costs nothing a consumer notices, because an entry only leaves when it is genuinely old. Under a size ceiling it is the *ordinary* case: a developer who sets a small ceiling on a large cache evicts their hot set on every build. Shipping a variable for that without a working-set measurement would be handing a consumer a foot-gun whose failure mode presents as "the build got slower".

**A name is a public surface.** Each variable added is a spelling, a parse, a diagnostic, and a support obligation, and ADR 0089's naming argument applies: the day one of these exists, `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`'s precise spelling is what leaves room for `..._MAX_TOTAL_BYTES` and `..._MAX_ENTRIES` to mean exactly what they say.

## Activation triggers

Any one of these fires it:

- `measure-the-expansion-cache-hot-path-efficiency` (or a successor) produces a **working-set lifetime** measurement — how long an entry stays useful — which is the evidence `MaxEntryAge::DEFAULT`'s own documentation names as what would replace a product choice with a derived number. A ceiling ordered by publication recency is defensible once the working set is measured rather than guessed.
- A report of a real cache exceeding a disk budget *despite* the age bound. The age policy bounds growth by time, not by bytes, so a consumer expanding many distinct regions inside one age window is the case it cannot answer.
- A least-recently-*used* order becomes available. `define-supported-expansion-cache-filesystems` owns the access-time question the collector deferred; with use recency, a size ceiling stops evicting the hot set and the objection above disappears.

## What it would then owe

- The two variable names, parsed into `max_total_bytes` and `max_entries`, absent by default and never guessed — the design record's refusal of a default *size* ceiling survives Tom's decision unchanged and must not be re-opened by this work.
- A byte spelling with units, refusing an unsuffixed count for `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`'s reason: a bare number is the ambiguity that deletes the wrong amount.
- Composition evidence: `CollectionBound` runs the age pass first and the aggregate pass over what it left, so the three ceilings compose as a union. A test asserting the exact `(key, reason)` sequence already exists in `tiler-cache`; the frontend owes the analogous end-to-end one.
- The frontend contract section in `docs/integration/frontends.md` extended with the new names, their absence-by-default, and the pathology stated plainly.
