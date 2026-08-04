---
id: define-the-runtime-kv-state-boundary
title: Define the runtime KV-state boundary
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family, establish-a-dynamic-kv-physical-layout-authority]
related: [design-autoregressive-state-and-kv-cache, prototype-candle-metal-adapter, transfer-synchronization-and-resource-lifetime-contract, bind-the-kv-cache-through-the-artifact-and-runtime-interface, name-a-host-process-availability-phase]
scopes: [contracts/integrations, contracts/foundation, research/runtime, research/program-planning, research/numerics]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, runtime, kv-cache, lifetime, identity, language-model]
---
## User-visible outcome

A KV state is a named object with a stated identity, capacity, valid range, and failure status — so "is this cache usable for this step, on this device" is a question with a typed answer instead of a convention.

Draft the boundary [rung L5's state contract](../docs/research/runtime/autoregressive-state-and-kv-cache.md) specifies, as a public surface. **It is a public boundary and therefore Tom's**; a tested implementation is a concrete draft and not implicit approval.

The physical-layout research now supplies the descriptor this boundary must draft: two alternating capacity-sized buffers per rank-three F32 K or V member, with the active payload packed exact-live head-major at `C*128` and the replacement at `S*128`. Capacity remains runtime pool policy; it is not a payload stride, artifact identity, or specialization value. Reconcile that survivor into the tested boundary before presenting it for acceptance; do not add the rejected capacity-stride root or allocate a fresh compact buffer per extent.

## Required content

- **Identity:** program interface key, layer ordinal, the live device and context the adapter bound, and a generation. Not an artifact subject — no packaged identity, cache key, or canonical descriptor may name a state.
- **Logical capacity, valid extent, cursor:** one positive logical capacity bound and a cursor `C` that is the single authority for how many sequence positions the state holds. The initial physical descriptor owns two capacity-sized F32 buffer banks, eight heads, width 128, an active-bank ordinal, and exact-live dense packing. Batch, raggedness, paging, growing capacity, overlap, and alternative ranks remain unsupported. Buffer length is allocation policy, not permission to index the payload at a capacity-derived stride.
- **Growth and update:** `C` advances by exactly `T` on the observed terminal success of the execution that produced the extended value, and never otherwise; logical `capacity` does not grow; the update is out of place and publication replaces the governed storage population and cursor together.
- **Placement, aliasing, retention, lifetime:** one symbolic affinity's memory domain under ADR 0047's initial profile; one active and one replacement bank per logical K or V member; old and replacement populations disjoint under complete role-labelled alias verification; both retained through exact final device use under ADR 0051; the state owned by the runtime instance and destroyed by the consumer.
- **Typed refusals:** `C + T > capacity` before any program work; a bind whose live device and context differ from the adapter's; a bind of a poisoned state, naming the execution that poisoned it.
- **The poisoned status.** A post-commit failure retires the state rather than leaving a plausible one behind. Under the out-of-place update the bytes are intact, so the reason to refuse is not corruption — it is that the failed step's token was never produced, and a later step binding the pre-failure state would decode a sequence the consumer does not believe it has.

## Device-scope disposition

Research eliminated both a platform handle in the consumer-neutral runtime and wholly adapter-private ad hoc scoping. The sole surviving design is a governed opaque `LiveStateScope` minted by the adapter from its private live device/context and compared by the generic state boundary. This preserves the dependency direction and makes the refusal uniform. Its eventual public spelling remains Tom's to accept as part of the coherent tested boundary.

The `research/runtime`, `research/program-planning`, `research/numerics`, and shared navigation scopes cover the survivor-independent state research and the correction of dense-layout assumptions copied into L1/L6/L8 and the quantized-profile comparison. They declare work already required by this boundary and do not accept a public API or physical representation.

## Closes when

The selected two-bank exact-live representation and its no-new-schema consequence are reconciled into the tested draft; every property and refusal is coherent with that representation; the coherent final public surface is then put to Tom; and nothing is accepted as public without his answer.
