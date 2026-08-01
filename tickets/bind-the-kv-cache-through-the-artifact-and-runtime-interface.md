---
id: bind-the-kv-cache-through-the-artifact-and-runtime-interface
title: Bind the KV cache through the artifact and runtime interface
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family, define-the-runtime-kv-state-boundary]
related: [design-autoregressive-state-and-kv-cache, assemble-the-causal-self-attention-block-program, expose-the-dispatch-record-on-a-decoded-artifact]
scopes: [implementation/artifact, implementation/runtime, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, kv-cache, language-model]
---
## User-visible outcome

The cache crosses the program boundary as ordered named inputs and outputs whose extents are bound per execution — so eight decode steps run from **one** artifact identity and one prepared pipeline, not eight of each.

## Required behaviour

- `k_cache` and `v_cache` are named program inputs of shape `[8, C, 128]`; `k_rope` and `v_heads` stay the retained outputs L4 named, at `[8, S, 128]`. Nothing about capacity, the cursor, or the allocation crosses the boundary.
- `C`, `T`, and `S` are bound as input-axis extents at `AvailabilityPhase::LiveDevicePreflight`, and every accessible-range and launch expression is a formula over them evaluated during preflight, so an evaluation failure is a refusal rather than a post-commit surprise.
- **No kernel may be specialized on `C`, `S`, or any cursor-derived quantity.** [The runtime execution contract](../docs/research/runtime/runtime-execution-contract.md) keys a prepared pipeline on its specialization values, so specializing on `S` would mint one pipeline per decode step and make a mutable inference quantity part of a cache key. Refuse it at artifact assembly, where the specialization values are packaged and the check is decidable.
- Two variants are packaged for the value contraction and selected per execution by an applicability guard over `S`: the tiled realization guarded on `S ≡ 0 (mod 16)`, the direct realization otherwise. Across C1's nine executions the tiled guard holds exactly once, at `S = 16`.

## Closes when

One assembled artifact routes at every C1 `S` from 10 to 18 with one identity; the guard selects tiled at `S = 16` and direct elsewhere; a program specializing on `S` is refused with its own diagnostic; and a test asserts the single identity across all nine executions so that a per-step compilation would fail rather than pass quietly.
