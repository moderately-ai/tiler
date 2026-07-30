---
id: design-autoregressive-state-and-kv-cache
title: Design autoregressive state and KV-cache ownership
status: todo
priority: p1
dependencies: [design-attention-program-vertical]
related: [device-placement-and-memory-domain-contract, transfer-synchronization-and-resource-lifetime-contract, prototype-candle-metal-adapter]
scopes: [research/runtime, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [design, runtime, kv-cache, prefill, decode, language-model]
---
## User-visible outcome

Prefill-then-decode has a designed execution contract: KV state with stated identity, layout, growth, aliasing, and lifetime — kept strictly apart from the immutable artifact and compilation caches, so mutable inference state never contaminates cache identity.

Design the state and execution contract required to run prefill followed by
repeated token decoding. Do not conflate mutable model execution state with the
immutable artifact and compilation caches already owned elsewhere.

## Required design

- Specify the semantic inputs and outputs of prefill and one decode step.
- Define KV-state identity, layout, capacity, valid range, growth, update,
  placement, aliasing, retention, and final-use lifetime.
- State which facts belong to the semantic program, physical plan, artifact,
  runtime instance, and consumer.
- Bound sequence length, batch behavior, masking, and any shape specialization.
- Derive preflight, routing-commit, allocation, dispatch, synchronization, and
  failure behavior across repeated executions.
- Test the design with a small attention example that exposes incorrect
  position, stale-state, partial-update, and cross-device reuse cases.

## Ticket-producing outcome

File vertical tickets for the state representation, artifact/runtime bindings,
prefill path, decode-step path, consumer integration, negative tests, and
end-to-end stateful-attention proof. Public boundaries remain drafts until Tom
reviews their exact implementation.

## Closes when

Ownership and correctness invariants are explicit at every layer; the design
can reject invalid state before program work; prefill and decode have bounded
user-visible outcomes; and the necessary delivery tickets are linked and
dependency ordered.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L5** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L4 delivers a complete transformer block.

**Rests on:** L4.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance (applies to every LM-ladder rung)

- **This rung consumes the selected workload**: pinned `Qwen/Qwen3-0.6B-Base` widened to F32, batch 1, with bounded prompt, context, and decode lengths. Its first state contract is an ordinary dense-decoder KV cache; recurrent and convolution state remain owned by the later Qwen3.5 hybrid ticket. If the workload is superseded after this analysis starts, the analysis is re-derived, not patched — say which parts survived and which did not.
- **Every requirement this analysis finds that Tiler cannot express today becomes a capability ticket**, filed with the exact operation/shape/dtype evidence from the trace, linked here and to the roadmap rung. Do not widen this ticket to implement any of them.
- **On close, update the ladder table in `docs/roadmap.md`** — its rung for this ticket currently reads "none", and nothing updates it automatically (the docs have no gate; a reader is the only check).
