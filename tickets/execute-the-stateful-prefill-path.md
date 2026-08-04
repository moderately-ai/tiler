---
id: execute-the-stateful-prefill-path
title: Execute the conformance prefill invocation and retain its outputs
status: todo
priority: p1
dependencies: [bind-repeated-invocations-over-caller-retained-tensors, integrate-the-attention-block-into-the-runtime, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, execute-the-decode-step-path]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, prefill, consumer-neutral, language-model]
---
## User-visible outcome

The C1 conformance driver runs its ten-token prompt through one attention block
as a single invocation and keeps the two retained output tensors, ready to bind
back as inputs on the next invocation.

## Scope correction — 2026-08-04

Rewritten under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
The outcome read "leaves behind a KV state whose cursor is 10", and the third
required bullet created that state with a `capacity` inside Tiler's runtime.
Tiler retains nothing between invocations, so there is no state to leave behind
and no cursor for Tiler to advance. This is consumer-driver work over ordinary
tensors, and `implementation/runtime` is dropped from its scopes because the
runtime change it declared no longer exists.

## Required behaviour

- Prefill is the **same program** as a decode step, invoked with the cached
  extent at zero. Two programs for one computation was eliminated in
  [the L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md):
  the twenty-two steps are identical and only the bound extents differ, so
  packaging both would duplicate an identity to buy a saving no reachable plan
  realizes. That elimination is unaffected by the supersession.
- A zero-extent operand follows its explicit allocation and ABI policy and is
  not replaced by an implicit null binding. Whether the zero-work concatenation
  dispatch is skipped is answered by the routed launch's own
  `zero_work_skips_dispatch`, not by a convention here.
- The driver allocates whatever it will reuse **before** the invocation and
  binds a dense payload at the exact extent it wrote. It advances its own
  cursor to `T` only on the invocation's observed terminal success, and never on
  submission alone.

## Closes when

The C1 prefill invocation runs at `T = S = 10`; the driver holds the returned
`k_rope` and `v_heads` as its own tensors, 81,920 bytes per layer, matching the
arithmetic; and an invocation whose submission does not reach terminal success
leaves the driver's cursor at 0 with no partially written retained tensor —
watched failing by forcing the failure, not asserted from the success path.
