---
id: survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature
title: Survey and select the rewrite-search formalism against the optimizer literature
status: done
priority: p2
dependencies: []
related: [derive-the-capability-set-for-search-discovered-flash-class-attention-kernels, land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract, probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary, decide-whether-stage-one-semantic-exploration-adopts-an-e-graph, close-the-enforcer-input-property-exclusion-gap, acquire-the-six-flagged-optimizer-literature-sources]
scopes: [research/region-search, contracts/navigation]
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

## Scope declaration added by this work

`contracts/navigation` was added to `scopes` because landing a governed research record obliges the same change to edit the catalog that indexes it, and both catalog files — `docs/research/README.md` and `spikes/README.md` — are in that scope while `docs/research/region-search/**` is not. AGENTS.md requires the catalog edit in the same change as the metadata behind it, so the scope is required by already-authorized work rather than an expansion of it. **Verified before adding, not assumed:** no live ticket held `contracts/navigation` — `tkt list --status in-progress` returned five tickets whose declared `scopes` are `implementation/reference`, six `research/*` scopes, `research/indexing`, this ticket, and a seventh set under `implementation/*` plus `contracts/{numerics,artifacts,decisions}`; none names `contracts/navigation`. The branch `tkt/carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus` exists with no ticket file and an empty diff against its merge-base with `origin/main`, so it holds nothing.

`contracts/optimizer` was **not** added, deliberately. The contract sentence this record answers lives in `docs/compiler/optimizer.md`, and the metadata contract does not require a reciprocal `evidence` entry for a research record's `informs` edge — the only rule is that adopted or partially-adopted research has an `informs` or `adopted_by` destination, which this record satisfies. The contract edit was a separate ticket (the land carrier). **At filing** this paragraph said the record's `disposition` stays `pending` until that carrier lands; the carrier has since completed and the record frontmatter is `disposition: "adopted"`.

## Graph maintenance

**The record landed:** [`docs/research/region-search/rewrite-search-formalism.md`](../docs/research/region-search/rewrite-search-formalism.md), with 30 preserved or fingerprinted sources under [`sources/`](../docs/research/region-search/sources/README.md) (`docs/research/region-search/sources/verify-sources.sh` reports `OK: 30 records verified (10 vendored, 20 metadata-only, 0 pending-acquisition)`) and an executable phase-ordering witness at [`spikes/region-search/phase_ordering_witness.py`](../spikes/region-search/phase_ordering_witness.py).

**The formalism question is decided, not packeted.** Two candidates did not survive: destructive cost-pruned rewriting and equality saturation as the whole search. A third, a Cascades memo as the whole search, is eliminated on the weakest step in the derivation — an argument from failure to construct a three-authority ordering inside a single property-keyed memo — and the record says so and answers Orca as the strongest objection. The survivor is a staged, alternative-retaining search whose durable new commitment is the prohibition it carries: **no semantic alternative is pruned on estimated cost, at any stage.** Nothing here needed a decision from Tom: the eliminations follow from obligations an accepted contract already carries.

**Filed by this work:**

- [`land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract`](land-the-rewrite-search-formalism-conclusion-in-the-optimizer-contract.md) — `done`. The carrier for the contract edit this ticket's scopes could not reach; closed by landing the formalism selection and no-cost-pruning invariant in `docs/compiler/optimizer.md` and moving the record disposition to `adopted`.
- [`probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary`](probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary.md) — `deferred` with a trigger log. Filed deferred rather than todo because its own stop condition is already met: the input rewrite set does not exist.
- [`decide-whether-stage-one-semantic-exploration-adopts-an-e-graph`](decide-whether-stage-one-semantic-exploration-adopts-an-e-graph.md) — `deferred`, depending on the probe.
- [`close-the-enforcer-input-property-exclusion-gap`](close-the-enforcer-input-property-exclusion-gap.md) — `done`. A gap the Volcano reading exposed (the contract's cycle check weaker than Volcano's excluding property vector); closed on its own outcome.
- [`acquire-the-six-flagged-optimizer-literature-sources`](acquire-the-six-flagged-optimizer-literature-sources.md) — `done`. Closed by acquiring the six flagged sources; final source census matches the verify-sources OK line above.

**Checks run.** `docs/research/region-search/sources/verify-sources.sh` (authority for the live census: 30 / 10 vendored / 20 metadata-only / 0 pending-acquisition), watched failing on five perturbations in a scratch copy; both spike entrypoints in normal and `-O` mode; the phase-ordering witnesses watched failing on three perturbations; `tkt lint`; `git diff --check`; `tkt guard`.

**One correction this work made to its own output, recorded because it generalizes.** The elimination of equality saturation originally argued that contract-dependent e-class membership forces one e-graph per numerical contract. `colored-egraphs-arxiv-2305.19203v1`, found and read after that sentence was written, refutes exactly that: multiple mutually inconsistent assumptions share one structure at measured overhead. The reason survives on its other half — a colored e-graph records which conclusions hold under which assumption and not *why* one was refused, and Tiler's refusals are typed reasons. The record keeps the correction visible rather than the refuted sentence, which is the only reason a reader can tell the elimination was tested rather than asserted.

**Correction — 2026-08-10.** Post-close dependent completions left this ticket's board snapshot stale. Graph maintenance had frozen a mid-campaign census (29 / 7 vendored / 16 metadata-only / 6 pending-acquisition) and filed-ticket statuses (`todo` for land, close-enforcer, and acquire; disposition `pending` until the carrier). At the audit base those three children are `done`, the verify-sources population is 30 / 10 / 20 / 0, and the research record disposition is `adopted`. The prose above is the corrected live board; the mid-campaign figures survive only in this note.
