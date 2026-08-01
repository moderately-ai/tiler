---
id: refuse-a-metal-payload-addressing-resources-the-abi-cannot-declare
title: Refuse a Metal payload addressing resources the artifact ABI cannot declare
status: in-progress
priority: p3
dependencies: [validate-metal-payload-argument-slots-against-declared-bindings]
related: []
scopes: [implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [candle, runtime, artifacts]
claimed_from: todo
assignee: worker-refuse-a-met
lease_expires_at: 1785565713
---
ADR 0090 item 8's third obligation is discharged for buffer arguments by `validate-metal-payload-argument-slots-against-declared-bindings`. This is the remainder it named rather than absorbed.

## User-visible outcome

A `metallib` whose kernel addresses a texture, sampler, or threadgroup resource — none of which the artifact ABI can declare — is refused before the routing commit, rather than being prepared and dispatched with that resource left unbound.

## Why this is open

**Fact — the comparison that landed counts buffers only.** `prepare_pipeline_with_reflection` filters the reflection to `MTLBindingType::Buffer`, and `declared_transport_slots` reads `RoutedBinding::transport_slot`, which is a `[[buffer(N)]]` index. That filter is correct as far as it goes: threadgroup rows are numbered in the disjoint `[[threadgroup(N)]]` namespace, so counting them against buffer slots would refuse a correct object.

**Fact — the artifact ABI models no other resource kind.** `tiler-metal`'s emitter produces `[[buffer(N)]]` parameters plus launch builtins (`emit.rs` `parameter_declaration` / `builtin_declaration`), and `RoutedBinding` carries a transport slot and nothing else. So a reflected texture, sampler, or threadgroup row has no declared counterpart to disagree with.

**Fact — that gap is inside the threat model, not outside it.** The objects this check exists to catch are exactly the ones Tiler's emitter did not produce. An object whose buffer arguments happen to match the declaration but which additionally addresses a texture passes today, and the encoder never binds it — a kernel reading an unbound resource rather than a refusal.

It has not bitten because every object this profile routes comes from Tiler's own emitter, which cannot emit one. That is a reason rather than a guarantee, and it is the same reason the buffer half was open.

## Closes when

- The adapter refuses, before the routing commit, a prepared entry whose reflection reports any binding the artifact ABI cannot declare, under a typed class distinct from a buffer-slot disagreement.
- The refusal names the resource kind and index it found, so a reader can tell a texture from a threadgroup allocation.
- It is watched failing against a real object that addresses such a resource — which needs a hand-written MSL kernel compiled to a `metallib` outside the emitter, since the emitter cannot produce one. If building that object on the qualified row proves impractical, that is recorded as a measurement with its exact procedure and the check is landed with its evidence boundary stated.
- Threadgroup rows are decided explicitly rather than by omission: either they are refused with the rest, or the reason a compiled kernel may legitimately carry one is recorded.
