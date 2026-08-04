---
id: prove-the-c1-stateful-attention-vertical
title: Prove the C1 stateful attention vertical end to end
status: todo
priority: p1
dependencies: [test-the-autoregressive-state-failure-cases, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, retain-the-c1-attention-block-conformance-evidence, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [implementation/candle, implementation/runtime, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, attention, consumer-neutral, language-model, class-conformance-fixture]
---
## User-visible outcome

**This is rung L5's user-visible outcome.** One causal self-attention block runs C1's prefill and all eight decode steps on Metal, with the driver holding the retained key and value tensors between invocations, and its results are compared bit for bit with the normative reference at every one of the nine executions. *Corrected 2026-08-04 under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md):* the outcome read "against a real KV state" — there is no Tiler KV state, and what the rung proves is that repeated invocations over caller-retained ordinary tensors reproduce the reference.

## Required content

- Nine executions: prefill at `T = S = 10`, then decode steps at `S = 11 … 18`. The comparison is per execution, not only at the end, because a cache that is wrong at step 3 and self-consistent afterwards passes an end-only check.
- The comparison is on exact bit patterns rather than an epsilon. The program declares a numerical contract; a result that is close but not equal has violated it.
- The retained evidence records the driver's retained bytes per step (81,920 through 147,456 per layer for the K and V pair), the variant selected at each `S`, and the single artifact identity across all nine.
- The measurement boundary is stated: one host, one toolchain, one block rather than the model, and one seed for any synthetic operand.

## Closes when

All nine executions agree with the reference, a deliberate perturbation of the driver's retained tensors at one step makes exactly that step and its successors disagree, and the retained record says what it does not establish — which is everything about the other twenty-seven layers and about the model.
