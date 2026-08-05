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
- 2026-08-05 — **not fired, and the recheck command above no longer decides it.** The first conjunct is now satisfied: the survey reached `done` with a staged, alternative-retaining formalism selected, and the conclusion landed in the optimizer contract under `land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract`. The second conjunct is what holds this deferred, and it is unmet by a wider margin than a status field shows — there is no benchmark harness in the repository at all, so no cold-search cost has been measured and none can be without first building one. Reproduce the deciding half: `find . -name benches -type d -not -path './target/*'` returns nothing and `grep -rn '\[\[bench\]\]\|criterion' --include=Cargo.toml .` returns nothing. Note also that the formalism selected leaves the *representation* of search state open — semantic exploration may or may not become an e-graph, held by `decide-whether-stage-one-semantic-exploration-adopts-an-e-graph` — so this ticket's "the memo or e-graph" phrasing still has two live subjects rather than one.
