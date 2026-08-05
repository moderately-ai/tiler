---
id: probe-e-graph-tractability-over-tilers-semantic-rewrite-vocabulary
title: Probe e-graph tractability over Tiler's semantic rewrite vocabulary
status: deferred
priority: p3
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature, derive-the-capability-set-for-search-discovered-flash-class-attention-kernels]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, search, equality-saturation, experiment, deferred]
---
## User-visible outcome

A measurement, under `spikes/region-search/`, of what an e-graph over Tiler's *semantic* rewrite vocabulary actually costs on the attention program — e-node and e-class growth per iteration, wall time, peak RSS, and whether the flash-shaped term appears before a declared budget stops the run.

## Why this is deferred rather than todo

**Fact.** [The rewrite-search formalism record](../docs/research/region-search/rewrite-search-formalism.md) Part 7 scopes this probe and declines to run it, because its own stop condition (a) is already met: the input rewrite set does not exist. Stage 3's vocabulary is two rules — `tiler.algebraic/ordered-reassociate-{add,multiply}-f32.v1@1`, both instantiations of one schema — and an e-graph over two rules saturates immediately and measures nothing. Running it with invented rules would measure the inventor.

## Scope, so a future claimant does not re-derive it

- **Inputs.** (1) The attention chain as a term over registered operation families. (2) A rewrite set: the implemented reassociation rules plus every rule the flash capability derivation declares for softmax and the contraction, each listed with the numerical permission it consumes recorded *beside* it and deliberately not encoded in the e-graph. (3) Node, e-class, and iteration budgets. (4) A fixed rule order and iteration order.
- **Outputs.** Per iteration: e-node count, e-class count, rules applied, wall time, peak RSS. Terminal: saturated / budget-exhausted / goal-found, and the iteration at which the flash-shaped term first appeared.
- **Method.** Use `egg` directly rather than reimplementing — the point is to measure a representation, not an engine. Two runs per configuration, both reported.
- **Stop conditions.** (a) The rewrite set is not declared — do not invent one. (b) Any rule needs a schedule-space concept (tiling, staging, placement) to state — the probe has become the published RISE experiment; report that and stop. (c) The e-graph must go cyclic and be repaired to express a rewrite — the finding is structural, not quantitative; report and stop.
- **What it decides.** Only whether the deferred component-level e-graph at stage 1 is viable. It cannot revive equality saturation as the whole search, which was eliminated on grounds a tractable algebraic e-graph does not touch.

## Trigger check log

- 2026-08-05 — **not fired.** The stage-3 rewrite vocabulary is two rules and no flash-class rewrite vocabulary is declared anywhere. Reproduce: `grep -rn 'RewriteRuleIdentity::new' crates/ --include='*.rs' | grep -v test` — three production rule identities plus one baseline label, of which two are algebraic.
