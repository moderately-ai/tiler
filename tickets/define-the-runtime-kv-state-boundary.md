---
id: define-the-runtime-kv-state-boundary
title: Define the runtime KV-state boundary
status: in-progress
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family]
related: [design-autoregressive-state-and-kv-cache, prototype-candle-metal-adapter, transfer-synchronization-and-resource-lifetime-contract, bind-the-kv-cache-through-the-artifact-and-runtime-interface, name-a-host-process-availability-phase]
scopes: [contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [design, runtime, kv-cache, lifetime, identity, language-model]
claimed_from: todo
assignee: agent-kv-boundary
lease_expires_at: 1785804091
---
## User-visible outcome

A KV state is a named object with a stated identity, capacity, valid range, and failure status — so "is this cache usable for this step, on this device" is a question with a typed answer instead of a convention.

Draft the boundary [rung L5's state contract](../docs/research/runtime/autoregressive-state-and-kv-cache.md) specifies, as a public surface. **It is a public boundary and therefore Tom's**; a tested implementation is a concrete draft and not implicit approval.

## Required content

- **Identity:** program interface key, layer ordinal, the live device and context the adapter bound, and a generation. Not an artifact subject — no packaged identity, cache key, or canonical descriptor may name a state.
- **Capacity, valid range, cursor:** a fixed `[8, capacity, 128]` allocation and a cursor `C` that is the single authority for how many positions the state holds.
- **Growth and update:** `C` advances by exactly `T` on the observed terminal success of the execution that produced the extended value, and never otherwise; `capacity` does not grow; the update is out of place and publication replaces the allocation and the cursor together.
- **Placement, aliasing, retention, lifetime:** one symbolic affinity's memory domain under ADR 0047's initial profile; old and new allocations distinct, which `verify_storage`'s `ForbiddenAlias` already requires; both retained through exact final device use under ADR 0051; the state owned by the runtime instance and destroyed by the consumer.
- **Typed refusals:** `C + T > capacity` before any program work; a bind whose live device and context differ from the adapter's; a bind of a poisoned state, naming the execution that poisoned it.
- **The poisoned status.** A post-commit failure retires the state rather than leaving a plausible one behind. Under the out-of-place update the bytes are intact, so the reason to refuse is not corruption — it is that the failed step's token was never produced, and a later step binding the pre-failure state would decode a sequence the consumer does not believe it has.

## The one genuine question, stated for Tom

Whether device scoping belongs to a governed runtime type or stays entirely inside each adapter. `tiler-runtime` forbids every platform device API and `LiveExecutionContext` deliberately carries no device handle, which argues adapter; but then every adapter re-implements one refusal, and [the runtime execution contract](../docs/research/runtime/runtime-execution-contract.md)'s `LiveDeviceKey` is already a governed shape for a device identity that is not a device object. This is L5's D-15.

## Closes when

The boundary is drafted with every property and refusal above, the device-scoping question is put to Tom with both options' consequences, and nothing is accepted as public without his answer.
