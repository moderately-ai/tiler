---
id: drive-the-complete-forward-pass-over-three-artifacts
title: Drive the complete forward pass over three artifacts
status: todo
priority: p1
dependencies: [widen-the-deterministic-budgets-to-the-decoder-layer-program, ingest-the-checkpoint-as-f32-program-inputs, define-the-model-execution-state-boundary, deliver-an-artifact-family-from-a-symbolic-region, integrate-the-autoregressive-decode-loop, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, route-an-embedded-artifact-through-a-consumer-storage-seam, prove-the-c1-complete-model-execution, name-the-execution-ordinal-in-model-level-failures]
scopes: [implementation/runtime, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, model, routing, artifacts, language-model]
---
## User-visible outcome

Token IDs go in and logits come out: one forward pass runs as 30 executions of 3 compiled artifacts against one model state, on one device, in order.

## Required behaviour

From [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).

- **The partition.** One embedding execution, 28 layer executions, one head execution. The 28 layer executions are the **same artifact identity** bound to 28 different weight sets; a build producing a fourth identity has specialized on a binding.
- **The layer count is a driver loop bound.** No artifact carries it, and a checkpoint with a different `num_hidden_layers` reuses the layer artifact unchanged.
- **Model-level preflight comes before the first routing commit.** The decision to run compiled or to decline is taken once, over the bound facts of all 30 executions, at the tensor-level preflight [Candle integration](../docs/integration/candle.md) already places there. It is not a held `Preflight`: `DecodedProgram::preflight` takes `&mut self` and the returned borrow carries into the commit, so 28 simultaneous preflights of the layer program are not expressible, and the check that is expressible is over the values and extents about to be bound.
- **After the first commit no execution of that pass may fall back.** A pass in which one execution fell back while another had committed would compose two numerical contracts and report one — the misattribution the Candle contract's numerical-scope section describes — so it refuses, naming both ordinals.
- **One ordered stream, one completion observation per token.** Under ADR 0047's initial profile the 28 layer dispatches share one ordered stream; the host observes terminal success once, for the logits, and the model cursor and all 28 allocations are published on that same observation.
- **A post-commit failure poisons the model state**, names the execution ordinal and the token in flight, and never selects an alternative.

## Closes when

One prefill pass and one decode step of the C1 row each run as 30 executions over exactly 3 artifact identities; a deliberately mismatched weight refuses at bind with its interface key before any route; a deliberately refused variant at execution 17 declines the whole pass before the first commit; and a forced post-commit failure poisons the model state rather than leaving a plausible one.
