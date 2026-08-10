---
id: prove-the-c1-stateful-attention-vertical
title: Prove the C1 stateful attention vertical end to end
status: todo
priority: p1
dependencies: [test-the-autoregressive-state-failure-cases, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, retain-the-c1-attention-block-conformance-evidence, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, attention, consumer-neutral, language-model, class-conformance-fixture]
---
## User-visible outcome

**This is rung L5's user-visible outcome.** One causal self-attention block runs C1's prefill and all eight decode steps on Metal, with the driver holding the retained key and value tensors between invocations, and its results are compared bit for bit with the normative reference — `tiler-reference` evaluation of the same decode-shaped block program under the same numerical contract on the stated host — at every one of the nine executions. The qwen model-logit fixture and the prefill-only attention-block probe are not substitutes for these nine block outputs. *Corrected 2026-08-04 under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md):* the outcome read "against a real KV state" — there is no Tiler KV state, and what the rung proves is that repeated invocations over caller-retained ordinary tensors reproduce the reference.

## Scope correction — 2026-08-10

Landed under the authority of [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md) (2026-08-04 rewrite of L5 delivery tickets 5–9 as consumer work). This is a consumer conformance fixture over ordinary tensors: it introduces no runtime-owned KV or session type and does not edit the L5 research record under `docs/research/runtime/**`. `implementation/runtime` and `research/runtime` are dropped from the scopes because neither maps to an edit surface this ticket owns; the primary scope is `implementation/candle`. Sibling prefill and decode-step tickets already carried this drop at supersession; this ticket had only the outcome rewrite and receives the scope drop here.

## Required content

- Nine executions: prefill at `T = S = 10`, then decode steps at `S = 11 … 18`. The comparison is per execution, not only at the end, because a cache that is wrong at step 3 and self-consistent afterwards passes an end-only check.
- The comparison is on exact bit patterns rather than an epsilon, against the named `tiler-reference` oracle above. The program declares a numerical contract; a result that is close but not equal has violated it.
- The retained evidence records the driver's retained bytes per step (81,920 through 147,456 per layer for the K and V pair), the variant selected at each `S`, and the measured artifact identity count under the conditioned claim below.
- **Artifact identity (conditioned).** The eight decode steps (`T = 1`, `S = 11…18`) must share one artifact identity — L5 cache-identity invariant 2's anti-specialization negative test (`eight decode steps at C1 must produce exactly one artifact identity`). Prefill at `T = 10` sharing that same identity is **not** asserted absolutely: under today's fixed-extent vocabulary each distinct `(T, C, S)` is its own semantic graph identity, and one identity across a family of `T` bindings additionally needs symbolic broadcast-result extents and a defined meaning at extent one (L6 D-19 / the 2026-08-05 L5 narrowing under [`decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`](decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md); L5 retained-item table: the retained item stands for `C` and not for `T`). Record the measured identity count and which condition is met rather than asserting nine-as-one.
- The variant recorded at each `S` is the packaged multi-variant routing selection available from [`bind-repeated-invocations-over-caller-retained-tensors`](bind-repeated-invocations-over-caller-retained-tensors.md) / the route. "Tiled" here means that selection (among C1's nine extents only `S = 16` qualifies the structure-3 tiled realization for a positive multiple of 16), not the deferred Metal tiled contraction body under [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) (still `deferred` at the 2026-08-10 audit base).
- The measurement boundary is stated: one host, one toolchain, one block rather than the model, and one seed for any synthetic operand.

## Correction — 2026-08-10

**Correction — 2026-08-10.** Ticket-audit wave. (1) Required content no longer asserts an unconditional "single artifact identity across all nine"; the eight-decode half is the load-bearing L5 invariant, and prefill sharing that identity is conditional on D-19 / symbolic-extent packaging (L5 2026-08-05 narrowing under decide-whether-one-decoder-layer-graph). (2) The normative reference is named as `tiler-reference` evaluation of the decode-shaped block program under the same numerical contract; qwen logits and the prefill-only attention-block probe are excluded as substitutes. (3) Per-`S` "variant selected" is the packaged multi-variant route selection, not deferred Metal tiled emission. (4) Scopes narrowed to `implementation/candle` (Scope correction section).

## Closes when

All nine executions agree with the named reference, a deliberate perturbation of the driver's retained tensors at one step makes exactly that step and its successors disagree, the retained record holds the per-step bytes, per-`S` packaged variant, and conditioned identity count above, and the retained record says what it does not establish — which is everything about the other twenty-seven layers and about the model.
