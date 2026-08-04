---
id: define-the-runtime-kv-state-boundary
title: Define the runtime KV-state boundary
status: in-progress
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family]
related: [design-autoregressive-state-and-kv-cache, prototype-candle-metal-adapter, transfer-synchronization-and-resource-lifetime-contract, bind-the-kv-cache-through-the-artifact-and-runtime-interface, name-a-host-process-availability-phase]
scopes: [contracts/integrations, contracts/foundation, research/runtime]
shared_scopes: [project/tickets, contracts/navigation]
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

## Device-scope disposition

Research eliminated both a platform handle in the consumer-neutral runtime and wholly adapter-private ad hoc scoping. The sole surviving design is a governed opaque `LiveStateScope` minted by the adapter from its private live device/context and compared by the generic state boundary. This preserves the dependency direction, makes the refusal uniform, and costs only fixed-size identity comparison. Because the constraints leave one survivor, D-15 is not a product-priority question; the exact consequential public surface remains Tom's to accept.

## Outcome — concrete draft, 2026-08-03

The proposed [runtime state boundary](../docs/integration/runtime-state.md) defines the exact public inventory, ownership transitions, exhaustive refusal set, fixed-capacity representation, out-of-place publication, adapter-minted current-route scope, program-and-slot interface identity, generation and cursor authority, poisoning, retention, and explicit destruction contract. Singular named `KvState` objects are owned by one bounded ordered `KvStateSet`; one context-bound route session consumes an artifact-owned complete state-interface manifest, commits once, poisons all members after any failure, and validates all identity-bound replacements/evidence before atomic publish-all-or-none. A transaction-bound adapter reporter supplies typed failure stage/cause or the complete receipt/coherence/readback/retention/replacement record rather than overloading opaque `Completion`/`Failure`. It structurally joins prepared state facts to the existing loader preflight and routing commit, including decoded `C`/`T`/`S` relation validation and storage-fingerprint revalidation. It distinguishes semantic-program, physical-plan, artifact, runtime-instance, adapter, and consumer facts and includes deliberate negative examples for caller omission, missing/extra/duplicate/unordered state members, one stale member, cross-device/context replay, contradictory artifact facts, malformed replacement reports, poison-all on drop, and publish-all-or-none.

`research/runtime` was added because completing this outcome replaces L5's superseded D-15 language and adds the new contract as an informed destination. Shared `contracts/navigation` was added because the hand-maintained documentation and research catalogs must link the new governed document. These are mapped scope declarations for work required by the ticket, not expansion of the product outcome.

The dependent `define-the-model-execution-state-boundary` now explicitly instantiates this generic set as 28 ordered K/V pairs (56 singular cached-tensor members) and owns only exact model composition and token policy. It no longer proposes a second cursor, poison, or publication authority, and its existing dependency on this ticket enforces that order.

No Rust implementation exists and no boundary is self-accepted. Batched/ragged state, prefix sharing, speculative rollback, growing capacity, windowed/in-place append, partial publication, cross-device transfer, multi-stream use, recurrent/convolutional state, and per-layer cursor drift remain explicitly unsupported.

## Public-boundary acceptance packet

Accept both the meaning and concrete spelling inventoried in `docs/integration/runtime-state.md`: adapter-authenticated runtime-instance device/context scope with no platform object in core; non-Clone `KvRouteSession` binding one exact `LiveExecutionContext` once and carrying its adapter/current-scope authority through commit; decoded semantic-graph plus ordered logical-interface plus input-ordinal `StateInterfaceKey`; singular read-only `KvState` members owned and mutated only by bounded ordered `KvStateSet`; artifact-owned complete `DecodedStateInterface` and no caller-authored state population; complete-bijection `KvArtifactStateBindingSet`; prepared set and route tokens joining every exact `C`/`T`/`S` relation and storage fingerprint to the existing loader `Preflight`; one commit that revalidates all members, mints opaque `KvExecutionIdentity` from the snapshots/step/routed dispatch/diagnostic ordinal, and constructs `BoundKvTransaction` with its set poison guard already attached; adapter `StateStorage`/`StateRetention`, `execute_state_transaction`, transaction-bound `StateTransactionReporter`, complete identity-bound replacement/receipt/coherence/readback/retention evidence, opaque report/success/failure carriers, and typed failure stages without restating identity; exhaustive build/refusal/binding/route/report/failure sets including missing/extra/duplicate/unordered/mixed/stale members, coherence, and validation readback; fixed-capacity out-of-place replacement that validates all member-identity-bound replacements before one atomic publish-all-or-none transition; poison-all after every post-commit non-success or unfinished drop; runtime-instance ownership with adapter-private storage/device objects and consumer-owned set lifetime; and no compatibility path for adapter-only ad hoc scoping or independently mutable singular states. The dependent artifact/runtime binding ticket owns the manifest schema/identity step and implementation; the generic set does not choose a model layer count, while the dependent model boundary owns exact 28-layer composition/token policy without a second transaction authority. Acceptance authorizes later implementation only; unsupported cases remain unsupported and rollback moves no Rust or current identity domain until that dependent implementation lands its whole schema step.

## Closes when

The boundary is drafted with every property and refusal above, the eliminated device-scoping candidates and sole survivor are recorded, the exact complete public surface is put to Tom, and nothing is accepted as public without his answer.
