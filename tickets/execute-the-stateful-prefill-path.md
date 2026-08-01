---
id: execute-the-stateful-prefill-path
title: Execute the stateful prefill path
status: todo
priority: p1
dependencies: [bind-the-kv-cache-through-the-artifact-and-runtime-interface, integrate-the-attention-block-into-the-runtime]
related: [design-autoregressive-state-and-kv-cache, execute-the-decode-step-path]
scopes: [implementation/runtime, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, prefill, kv-cache, language-model]
---
## User-visible outcome

Prefill runs C1's ten-token prompt through one attention block and leaves behind a KV state whose cursor is 10 and whose bytes a decode step can read.

## Required behaviour

- Prefill is the **same program** as a decode step, with `C = 0`. Two programs for one computation was eliminated in [the L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md): the twenty-two steps are identical and only the bound extents differ, so packaging both would duplicate an identity to buy a saving no reachable plan realizes.
- A zero-extent cache operand follows its explicit allocation and ABI policy and is not replaced by an implicit null binding. Whether the zero-work concatenation dispatch is skipped is answered by the routed launch's own `zero_work_skips_dispatch`, not by a convention here.
- The state is created before the execution, with `capacity` from the row's declared maximum context (18 at C1), and its cursor advances to `T` only on the observed terminal success of the prefill submission.

## Closes when

C1 prefill executes at `T = S = 10`, the retained `k_rope` and `v_heads` are published as a state whose cursor is 10 and whose 81,920 bytes per layer match the arithmetic, and a prefill whose submission does not reach terminal success leaves a state with cursor 0 and a poisoned status rather than a partially advanced one.
