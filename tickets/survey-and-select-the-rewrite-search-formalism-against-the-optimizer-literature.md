---
id: survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature
title: Survey and select the rewrite-search formalism against the optimizer literature
status: todo
priority: p2
dependencies: []
related: [derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The optimizer's exploration skeleton stops being "a Cascades-style memo is one possible implementation technique, not a committed design" (the optimizer contract's own words) and becomes a selected formalism with a recorded elimination, grounded in the primary literature rather than re-derived from intuition — because the formalism decides whether flash-class compositions are *reachable* by search at all, or hidden behind phase ordering.

## Why this exists

**Fact.** No board item owns the exploration algorithm. The optimizer contract commits to the search's *obligations* — typed identities, explain output for accepted and rejected candidates, deterministic budgets, hard feasibility separate from cost — and deliberately not to its mechanism. **Inference.** The mechanism choice is load-bearing for the discovery thesis: a phase-ordered rewriter applies fusion before reassociation or vice versa, and the flash-shaped candidate exists only in the composition of both; a formalism that holds all rewrites simultaneously (an e-graph) or memoizes alternatives without commitment (Cascades) does not have that failure mode, at different costs.

## The literature-survey obligation, which is the ticket's core

Preserve and read the primary sources per the source-record discipline (`docs/research/numerics/sources/README.md` is the working precedent for acquisition, licence verdicts, and digests; a sibling record under this scope may establish its own manifest the same way). The survey must cover, at minimum, each with its exact relevance question:

- **Database optimizer lineage** — Volcano and Cascades (Graefe), and Selinger-style dynamic programming: the memo structure, group/expression separation, guidance rules, and how cost-based pruning composes with completeness. This repository's architecture is explicitly "DataFusion for tensor compute"; the survey states what the DB memo does and does not transfer when candidates carry physical feasibility predicates and numerical permissions rather than only cost.
- **Equality saturation** — the original equality-saturation work (Tate et al.), egg (Willsey et al.), egglog, and their tensor-graph applications (Tensat; TASO's superoptimization is adjacent as search-with-verification): whether e-graph saturation over Tiler's declared rewrites is tractable at the vocabulary's size, how extraction interacts with a cost authority that does not exist yet, and what saturation does to explain output and deterministic budgets (a saturated e-graph explains "everything legal was present"; a budgeted one must explain what was not).
- **Tensor-compiler search** — Halide's autoscheduler lineage, TVM/Ansor's hierarchical search: what they searched over (schedules, not algebraic rewrites) and why that split (rewrite space vs schedule space, searched by different machinery) may or may not be the right factoring here.
- **Phase-ordering literature** — enough to state precisely which orderings hide which compositions, with the attention chain as the worked example.

## What the record must decide or defer

The selected formalism (or a staged combination — e.g. saturation over the rewrite space feeding a memo over the physical space) with the elimination stated so a reader can refute it; how identities, explain, budgets, and fail-closed feasibility map onto the formalism's structures; what the smallest bounded experiment proving tractability at the current vocabulary size looks like (an e-graph over the registered rewrite set is buildable as a spike without touching `crates/`); and what remains deferred with triggers. Graph augmentation is a fully acceptable outcome where evidence is thin; the formalism *choice* itself may end as a decision packet for Tom if two candidates genuinely survive.

## Non-goals

Implementing the search; any `crates/` edit; committing a cost model (that is `research/cost-model`'s thread); re-deriving what a preserved paper states.

## Closes when

The record exists under `docs/research/region-search/` with preserved-source citations, the formalism question is decided, packeted for Tom, or explicitly deferred with a trigger, the phase-ordering risk is stated against the attention worked example, and any bounded experiment it proposes is filed with inputs, outputs, and stop conditions.
