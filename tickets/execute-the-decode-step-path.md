---
id: execute-the-decode-step-path
title: Execute one decode step over caller-retained tensors
status: todo
priority: p1
dependencies: [execute-the-stateful-prefill-path, evaluate-retained-shape-relations-before-routing-commit, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, integrate-the-autoregressive-decode-loop]
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, conformance, decode, routing, consumer-neutral, language-model, class-conformance-fixture]
---
## User-visible outcome

One decode step binds the tensors the previous invocation returned, extends them
by one position, and returns the extended pair — with its own routing commit,
its own variant selection, and a driver cursor that advances only when the
device says the work finished.

## Scope correction — 2026-08-04

Rewritten under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
The outcome read "reads a published KV state … and publishes a new state", and
the second required bullet demanded a `C + T > capacity` refusal from the
adapter. Tiler holds no state and is handed no capacity, so that refusal is
withdrawn rather than relocated — see the L5 record's superseded typed-refusal
list. Every other bullet was already about ordinary invocation mechanics and is
unchanged in substance. `implementation/runtime` is dropped from the scopes
because no runtime change remains in this ticket; the generic checks it relies
on are owned by
[`evaluate-retained-shape-relations-before-routing-commit`](evaluate-retained-shape-relations-before-routing-commit.md)
and
[`bind-repeated-invocations-over-caller-retained-tensors`](bind-repeated-invocations-over-caller-retained-tensors.md).

## Required behaviour

- The step is a complete route over the already-decoded artifact: bind facts,
  route, validate the payload, answer the device questions, size the dispatch,
  commit, allocate, dispatch. **Each step's routing commit is its own** under
  ADR 0051; a fallback taken at step 5 is a fallback for step 5 alone, and there
  is no fallback after that step's commit.
- Before any of that, the adapter refuses a **bound value** whose live device
  and context are not the ones it bound, naming both. *Corrected 2026-08-04:*
  this bullet also required a `C + T > capacity` refusal; the driver checks its
  own pool bound before it binds, and Tiler has no capacity to compare against.
- Before the routing commit, the invocation's `C`, `T`, and `S` bindings are
  checked against the retained `S == C + T` relation. A mismatch is a typed
  refusal naming all three values; carrying the accepted relation without
  consuming it is not sufficient.
- **Corrected 2026-08-01 by [`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](reconcile-the-pre-commit-allocation-seam-with-adr-0051.md), which split the seam this bullet described.**
  `plan_dispatch` *sizes* the step: it names the bound input resources, sizes
  `[8, S, 128]` for each retained output, and compares every required range
  against this device's declared limits, acquiring nothing. `allocate_dispatch`
  — reached only from the committed `RoutedDispatch` — binds the caller's inputs
  read-only and takes the outputs' resources. Both are retained against the
  submission receipt; a caller-supplied input becomes releasable only after the
  completion condition, never after its last encoder call. An allocation that
  comes back short is a `Failure` at that stage rather than a refusal, so a step
  cannot be retried on the strength of one.
- The driver swaps its retained tensors and advances its cursor **together**, on
  observed terminal success — terminal completion, a post-completion status
  check, coherence, record validation, then interpretation, in ADR 0033's order.
  This costs no extra synchronization because a greedy loop already reads the
  logits back to form the next token.

## Closes when

Decode step 1 executes at `C = 10, T = 1, S = 11` against the tensors prefill
returned and yields a pair at `S = 11`; a step whose dispatch fails after the
commit leaves the bound input tensors bit-identical, returns no observable
output, and leaves the driver's cursor at 10; and the tiled variant is selected
at `S = 16` and only there.
