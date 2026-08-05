---
id: scope-search-state-caching-across-shape-families
title: Scope search-state caching across shape families
status: deferred
priority: p3
dependencies: []
related: [derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/cache]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

An expensive search's state — the memo or e-graph, or a winning schedule as a warm start for a neighbouring shape — is reusable across compilations under the same identity discipline the expansion cache already enforces for artifacts, so search cost amortizes across a shape family instead of being paid per shape.

## Why this exists, and why it is deferred

**Fact.** The expansion cache stores *artifacts* keyed by exact compilation identity, with validation on every hit and immutable entries. **Inference.** Search state is a different animal: it is an accelerator whose staleness costs recompilation rather than wrongness, its key is a shape *family* rather than an exact identity, and a wrong hit must degrade to a cold search rather than a wrong plan — the fall-open discipline, where the artifact cache falls closed. The database ancestor is parameterized plan caching and plan-cache invalidation, which the survey mines deliberately. Deferred because no search exists to cache: the formalism ticket decides what the state even is.

## What the record must decide (when fired)

The keying (what a "shape family" is, and what the family key excludes so a hit is ever possible); the staleness story (a cached memo built under an older rewrite vocabulary or profile must be detected — the registry snapshot identities are the existing precedent); the degradation guarantee (never a wrong plan, watched failing); and whether warm-start transfer is worth its complexity against the measured cold-search cost — a question that needs the search to exist and be measured first, on the designated measurement host.

## Trigger

`survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature` reaching `done` with a selected formalism, plus a measured cold-search cost worth amortizing.

## Trigger check log

- 2026-08-05 — **not fired.** The formalism ticket was filed today and is `todo`; no search exists, so no cost exists to amortize. Recheck: `grep -m1 '^status:' tickets/survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature.md`.
