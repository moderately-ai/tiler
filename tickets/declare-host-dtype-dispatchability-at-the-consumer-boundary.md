---
id: declare-host-dtype-dispatchability-at-the-consumer-boundary
title: Give every consumer a host-earned dtype-dispatchability declaration
status: todo
priority: p2
dependencies: [validate-bf16-at-the-runtime-routing-boundary]
related: [decide-per-dtype-dispatchability-as-a-target-capability]
scopes: [implementation/frontend, implementation/build, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, runtime, routing, fail-closed, authority]
---
## User-visible outcome

A consumer's `ExecutionEnvironment` states which dtypes *this machine's* target family dispatches, derived from something the machine or its build earned, rather than restating what the artifact's producer declared. Until then the runtime's dtype refusal cannot fire on the two paths a real consumer takes.

## Why the restatement is a gap and not a design

**Fact.** `validate-bf16-at-the-runtime-routing-boundary` added `ExecutionEnvironment::dtype_dispatch` and made an undispatchable dtype filter a variant before ADR 0051's routing commit. Every consumer literal it had to fill in restates a producer declaration:

- `crates/tiler/src/route.rs::execution_environment` reads the macro-emitted `RouteFacts`, which carries **no** dtype fact at all, so it states `f32` alone on the reasoning that `tiler-compiler` already refused any request whose dtype the selected profile did not resolve `Dispatchable` — a restatement of the compile gate, not an observation.
- `prototypes/candle-metal-adapter/src/proof.rs::declared_route_environment` transcribes `tiler-build`'s `FIRST_MACOS_APPLE9` ledger rows, and says of itself that it is "producer-declared equality, NOT host-earned eligibility".

**Inference.** A comparison whose two sides come from one authority refuses nothing. This is the same shape as `ExecutionEnvironment::classify` on those paths, and it is why the dtype check's value today is the *named* refusal it can produce rather than a barrier those paths did not already have.

**Fact.** ADR 0086 is directly on point about what may and may not stand in for a host-earned fact: a public execution-environment row is a necessary validity scope and explicitly not sufficient authority, and item 4 excludes a list of substitutes by name. A dtype row asserted at a call site is that failure mode with a different subject.

## Implementation keys

- Emit the selected profile's declared dtype-dispatchability rows into `RouteFacts`, so `execution_environment` restates an emitted fact instead of asserting one. This is what a `bf16` inline region needs before it can route at all — today it would be refused, correctly, for a reason only a source comment records.
- Decide, and record, whether the frontend's inline-region path can offer a *host-earned* row at all, given that it binds no device before `route_with_adapter` returns. If it cannot, say so in the contract rather than leaving the restatement looking like an observation.
- Give `BoundMetalCompileDeclaration` an accessor for its dispatchability rows so the Candle prototype reads them rather than transcribing them; it already holds `bf16_dispatchability`.
- Keep silence fail-closed. A consumer that cannot yet earn a row states nothing and refuses, which is the existing behaviour and must not be relaxed into a permissive default.

## Required evidence

- An inline region whose dtype the emitted rows do not admit is refused, and the refusal is observed failing.
- A perturbation of the emitted row changes the routing outcome, so the fact is load-bearing rather than carried.
- Each remaining restatement, if any survives, is named in the contract with what would make it host-earned.

## Closes when

No consumer asserts a dtype-dispatchability row at a call site without either an emitted fact or a bound-device observation behind it, or the surviving restatements are recorded as such in a durable contract with their authority gap stated.
