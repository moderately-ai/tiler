---
id: drive-the-complete-forward-pass-over-three-artifacts
title: Drive the complete forward pass over three artifacts
status: todo
priority: p1
dependencies: [widen-the-deterministic-budgets-to-the-decoder-layer-program, ingest-the-checkpoint-as-f32-program-inputs, deliver-an-artifact-family-from-a-symbolic-region, integrate-the-autoregressive-decode-loop, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-model-ingestion-and-complete-execution, route-an-embedded-artifact-through-a-consumer-storage-seam, prove-the-c1-complete-model-execution, name-the-execution-ordinal-in-model-level-failures]
scopes: [implementation/runtime, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, model, routing, artifacts, language-model, class-conformance-fixture]
---
## User-visible outcome

Token IDs go in and logits come out: one forward pass runs as 30 executions of 3 compiled artifacts, on one device, in order, against retained tensors the consumer binds and rebinds as ordinary program inputs and outputs.

## Required behaviour

From [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md).

- **The partition.** One embedding execution, 28 layer executions, one head execution. The 28 layer executions are the **same artifact identity** bound to 28 different weight sets; a build producing a fourth identity has specialized on a binding.
- **The layer count is a driver loop bound.** No artifact carries it, and a checkpoint with a different `num_hidden_layers` reuses the layer artifact unchanged.
- **Model-level preflight comes before the first routing commit.** The decision to run compiled or to decline is taken once, over the bound facts of all 30 executions, at the tensor-level preflight [Candle integration](../docs/integration/candle.md) already places there. It is not a held `Preflight`: `DecodedProgram::preflight` takes `&mut self` and the returned borrow carries into the commit, so 28 simultaneous preflights of the layer program are not expressible, and the check that is expressible is over the values and extents about to be bound.
- **After the first commit no execution of that pass may fall back.** A pass in which one execution fell back while another had committed would compose two numerical contracts and report one — the misattribution the Candle contract's numerical-scope section describes — so it refuses, naming both ordinals.
- **One ordered stream, one completion observation per token.** Under ADR 0047's initial profile the 28 layer dispatches share one ordered stream, and the host observes terminal success once, for the logits. *Corrected 2026-08-04 under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md):* the driver — not Tiler — advances its cursor and swaps all 56 retained tensors on that same observation, together or not at all, so no reader of its state observes a partially advanced model.
- **A post-commit failure names the execution ordinal and the token in flight, makes that execution's outputs unobservable, and never selects an alternative.** *Corrected 2026-08-04:* this bullet read "poisons the model state". Tiler holds no model state to poison; the driver owes the refusal to continue from its pre-failure tensors, because the failed step's token was never produced.

## Closes when

One prefill pass and one decode step of the C1 row each run as 30 executions over exactly 3 artifact identities; a deliberately mismatched weight refuses at bind with its interface key before any route; a deliberately refused variant at execution 17 declines the whole pass before the first commit; and a forced post-commit failure reports its ordinal with no observable outputs, after which the driver refuses to continue from its pre-failure tensors rather than producing a plausible next token.
