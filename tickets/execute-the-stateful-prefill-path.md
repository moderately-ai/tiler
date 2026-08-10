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
tags: [implementation, conformance, prefill, consumer-neutral, language-model, class-conformance-fixture]
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

- Prefill uses the decode-shaped program invoked with the cached extent at zero (`C = 0`). At a fixed `T`, that empty-cache binding has the same occurrence signature as a nonempty cache — only extents move (`a_nonempty_cache_changes_no_occurrence` in `crates/tiler-reference/tests/decoder_layer.rs`). Packaging a second program solely along the cache axis was eliminated in [the L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md): it would duplicate an identity to buy a saving no reachable plan realizes. That cache-axis elimination is unaffected by the 2026-08-04 supersession. This ticket does **not** claim one unconditional artifact identity across C1 prefill at `T = 10` and C1 decode at `T = 1`; that T-axis question is owned by L5/L6 D-19 and [`define-the-widening-relation-over-a-symbolic-broadcast-extent`](define-the-widening-relation-over-a-symbolic-broadcast-extent.md).
- A zero-extent operand follows its explicit allocation and ABI policy and is not replaced by an implicit null binding. Whether the zero-work concatenation dispatch is skipped is answered by the routed launch's own `zero_work_skips_dispatch`, not by a convention here.
- The driver allocates whatever it will reuse **before** the invocation and binds a dense payload at the exact extent it wrote. It advances its own cursor to `T` only on the invocation's observed terminal success, and never on submission alone.

## Correction — 2026-08-10

**Correction — 2026-08-10.** Ticket-audit wave. Required behaviour's first bullet absorbed L5's 2026-08-05 narrowing under [`decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`](decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md) (L5 anchors: `narrowed, not withdrawn`; `a_nonempty_cache_changes_no_occurrence`). The retired absolute — "the twenty-two steps are identical and only the bound extents differ" as covering C1 prefill versus C1 decode — is struck for the T half; the cache half still holds. Close still targets only prefill at `T = S = 10`; no D-19 delivery is owed under this id.

## Closes when

The C1 prefill invocation runs at `T = S = 10`; the driver holds the returned `k_rope` and `v_heads` as its own tensors, 81,920 bytes per layer, matching the arithmetic; and an invocation whose submission does not reach terminal success leaves the driver's cursor at 0 with no partially written retained tensor — watched failing by forcing the failure, not asserted from the success path.
