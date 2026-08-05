---
schema: "tiler-doc/v1"
id: "tiler.spike.region-search"
kind: "experiment"
title: "Region-search experiments"
topics: ["fusion", "search", "optimizer", "phase-ordering"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["exhaustive-finite", "executable-model"]
supports: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.region-search.rewrite-search-formalism"]
entrypoints: ["spikes/region-search/exhaustive_oracle.py", "spikes/region-search/phase_ordering_witness.py"]
last_verified: "2026-08-05"
ticket: "region-search-oracle"
---

# Region-search experiments

Two independent executable models live here. They answer different questions and share no code: the oracle asks which fusion regions and covers are *legal*, and the witness asks which search strategies can *reach* a composition. The record retitled this file when the second arrived; the record id is unchanged.

## Exhaustive fusion-region oracle

The Python oracle enumerates legal connected fusion regions and complete
program covers for bounded tiny DAGs, retaining legality and rejection reasons.

Run from the repository root:

```sh
uv run python spikes/region-search/exhaustive_oracle.py
uv run python -O spikes/region-search/exhaustive_oracle.py
```

Both modes produce the same output; verdicts use explicit checks that optimized
Python cannot remove. Its exhaustive claim applies only to the finite graph
bounds and legality language implemented by the harness.

## Phase-ordering witness

The witness models the mutual-enabling structure behind a flash-class attention implementation — an online softmax fold and a dropped score-matrix materialization, each of which the other enables — and runs four search strategies over it. Three miss the composition and one reaches it. It supports [the rewrite-search formalism record](../../docs/research/region-search/rewrite-search-formalism.md), whose Part 4 states what the result does and does not establish.

Run from the repository root:

```sh
uv run python spikes/region-search/phase_ordering_witness.py
uv run python -O spikes/region-search/phase_ordering_witness.py
```

**What it is not.** It is a model of an ordering structure, not of Tiler's IR: it reads no `SemanticProgram`, computes no real cost, and proves no rewrite sound. Its weights are ordinal and chosen to encode the enabling relation. It deliberately does not discriminate a Cascades-style memo from an e-graph — both retain the alternative a greedy rewriter discards, and the model covers them jointly in one strategy.

**Its witnesses have been watched failing.** Removing the legality half of the enabling relation, removing the cost half, and weakening the pruned-frontier strategy into full retention each turn a passing run red with a distinct named failure, which is why a uniform pass over four strategies is worth reading here. The `test_the_discrimination_comes_from_the_enabling_relation` witness runs the cost-side perturbations in-process on every run.
