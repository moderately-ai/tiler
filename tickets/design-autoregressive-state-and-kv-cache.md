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
