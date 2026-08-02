---
id: execute-the-decode-step-path
title: Execute one decode step against a published KV state
status: todo
priority: p1
dependencies: [execute-the-stateful-prefill-path]
related: [design-autoregressive-state-and-kv-cache, integrate-the-autoregressive-decode-loop]
scopes: [implementation/runtime, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, decode, kv-cache, routing, language-model]
---
## User-visible outcome

One decode step reads a published KV state, extends it by one position, and publishes a new state — with its own routing commit, its own variant selection, and a cursor that advances only when the device says the work finished.

## Required behaviour

- The step is a complete route over the already-decoded artifact: bind facts, route, validate the payload, answer the device questions, size the dispatch, commit, allocate, dispatch. **Each step's routing commit is its own** under ADR 0051; a fallback taken at step 5 is a fallback for step 5 alone, and there is no fallback after that step's commit.
- Before any of that, the adapter refuses a state whose live device and context are not the ones it bound, and refuses `C + T > capacity` — a hard feasibility refusal with the required context and the capacity in it, never an expensive cost.
- **Corrected 2026-08-01 by [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](reconcile-the-pre-commit-allocation-seam-with-adr-0051.md), which split the seam this bullet described.** `plan_dispatch` *sizes* the step: it names the old allocation at the cache slots, sizes `[8, S, 128]` for each retained output, and compares every required range against this device's declared limits, acquiring nothing. `allocate_dispatch` — reached only from the committed `RoutedDispatch` — binds the old allocation read-only and takes the fresh one. Both are retained against the submission receipt; the old one becomes releasable only after the completion condition, never after its last encoder call. An allocation that comes back short is a `Failure` at that stage rather than a refusal, so a step cannot be retried on the strength of one.
- The cursor advances, and the allocation is replaced, **together**, on observed terminal success — terminal completion, a post-completion status check, coherence, record validation, then interpretation, in ADR 0033's order. This costs no extra synchronization because a greedy loop already reads the logits back to form the next token.

## Closes when

Decode step 1 executes at `C = 10, T = 1, S = 11` against the prefill state and publishes a state with cursor 11; a step whose dispatch fails after the commit leaves the input state bit-identical and poisoned; and the tiled variant is selected at `S = 16` and only there.
