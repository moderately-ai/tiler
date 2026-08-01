---
id: define-the-model-execution-state-boundary
title: Define the model execution state boundary
status: todo
priority: p1
dependencies: [assemble-the-decoder-layer-program, define-the-runtime-kv-state-boundary]
related: [design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, drive-the-complete-forward-pass-over-three-artifacts, scope-a-windowed-kv-append-into-retained-capacity]
scopes: [contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [design, runtime, state, kv-cache, lifetime, public-boundary, language-model]
---
## User-visible outcome

A model in flight is one named object with one cursor, so "how many positions does this model hold, and is it usable for the next token" has a typed answer that cannot disagree with itself across 28 layers.

**It is a public boundary and therefore Tom's**; a tested implementation is a concrete draft and not implicit approval.

## Required content

Drafted from [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md), which extends [rung L5's state contract](../docs/research/runtime/autoregressive-state-and-kv-cache.md) rather than replacing it.

- **Composition.** One model state owns 28 layer KV states, each exactly the object [`define-the-runtime-kv-state-boundary`](define-the-runtime-kv-state-boundary.md) drafts, plus one model-level cursor `C` and a generation per layer state.
- **The granularity rule, refined.** L5 states that "per-layer programs need per-layer cursors; one program per step needs one." What that rule protects is *observability* — that no consumer can observe a state advanced for some layers and not others — not program size. A per-layer program boundary with one model cursor and a generation per layer satisfies it, provided the 28 allocations and the cursor are replaced **together**, as one step, after one observed terminal success. The combination L5 named — 28 cursors advanced independently by 28 separately observed executions — stays forbidden.
- **The transaction boundary is the token.** All 28 old allocations are retained until that one observation, which is what makes a post-commit failure leave the model bit-identical to what it was. This costs 3,816,587,264 bytes of peak KV residency at B1-d's final step against 1,976,557,568 for the layer transaction, and it buys the non-destructive failure the layer transaction gives up. **This is L6's D-16.**
- **Poisoning is model-level.** A post-commit failure at any of the 30 executions retires the whole model state and names the execution ordinal and the token in flight. It never poisons one layer and leaves the others usable.
- **Typed refusals.** A bind whose `C + T` exceeds any layer state's capacity, before any program work; a bind whose live device and context differ from the adapter's; a bind of a poisoned model state, naming the execution that poisoned it; a bind whose layer states disagree on their generation.

## The question this carries to Tom

**D-16.** Whether the transaction boundary ever moves from the token to the layer. It closes only with both halves together: a measured decode-latency or peak-residency result at a B1 row where the 1,840,029,696-byte (1.714 GiB) difference is the binding constraint, **and** a recovery contract that says what a consumer does with 28 states at mixed cursors. The residency arithmetic alone would justify the change and would not make it safe.

## Closes when

The boundary is drafted with every property and refusal above, D-16 is put to Tom with both options' consequences, and nothing is accepted as public without his answer.
