---
id: define-the-runtime-kv-state-boundary
title: Define the runtime KV-state boundary
status: closed
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family, establish-a-dynamic-kv-physical-layout-authority, supersede-the-runtime-owned-kv-state-design, reclassify-language-model-work-as-a-conformance-track]
related: [design-autoregressive-state-and-kv-cache, prototype-candle-metal-adapter, transfer-synchronization-and-resource-lifetime-contract, name-a-host-process-availability-phase, bind-repeated-invocations-over-caller-retained-tensors]
scopes: [contracts/integrations, contracts/foundation, research/runtime, research/program-planning, research/numerics]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [design, runtime, kv-cache, lifetime, identity, language-model, supersession, class-obsolete]
closed_reason: superseded
closed_note: The runtime owns no KV state; superseded by supersede-the-runtime-owned-kv-state-design. Draft branch preserved as review evidence.
---
## Superseded — 2026-08-04

**This ticket is closed as superseded by
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
It is not closed as done, and it satisfies no dependent.** Nothing here should be
started; the sections below are preserved so that a reader can see what was
concluded, why, and what replaced it.

### What it asked for

A public Tiler boundary naming a KV state as an object with an identity
(program interface key, layer ordinal, live device and context, generation), a
logical capacity, a cursor `C` that is the single authority for how many
sequence positions the state holds, a growth rule advancing `C` by `T` on
observed terminal success, out-of-place publication replacing storage and cursor
together, a typed `C + T > capacity` refusal, a typed live-device/context
refusal through a governed opaque `LiveStateScope`, and a terminal poisoned
status refusing every later bind. Its physical descriptor was the survivor
selected by
[`establish-a-dynamic-kv-physical-layout-authority`](establish-a-dynamic-kv-physical-layout-authority.md):
two alternating capacity-sized F32 buffer banks per rank-three K or V member,
exact-live head-major packing, no capacity stride.

### Why it is superseded

The derivation was internally sound; the *owner* was wrong. Tiler is a
consumer-agnostic tensor compiler and execution toolkit. Its runtime executes
one artifact invocation from explicit bindings, returns explicit outputs, and
retains nothing across invocations. A KV state is a transformer-serving session
object: it names a layer ordinal, a sequence cursor, and a decode generation,
and it exists only between one consumer's invocations. Publishing it as a Tiler
type would put workload vocabulary into the runtime's public surface, give the
runtime a lifetime it otherwise does not have, and make every non-transformer
consumer pay for a concept it cannot use. A consumer expresses prefill-then-decode
by holding ordinary tensors between invocations and binding them as ordinary
program inputs — which the artifact and runtime interface already supports, with
extents bound per invocation and one artifact identity across the family.

### Where its content went

- **Retained and already generic** — bindings are explicit, extents are bound
  per invocation, retained shape relations are checked before the routing commit
  ([`evaluate-retained-shape-relations-before-routing-commit`](evaluate-retained-shape-relations-before-routing-commit.md)),
  a program input and a program output may not share one allocation, every
  resource an invocation addressed is retained through its exact final device
  use under ADR 0051, and a bound value's live device and context must match the
  adapter's.
- **Generalized rather than dropped** — the device-scoping refusal. Its subject
  becomes *a bound value* rather than a state object, so the `LiveStateScope`
  spelling is withdrawn while the obligation stands for every invocation.
- **Moved to the consumer** — capacity, the cursor, the generation, the buffer
  banks and their active-bank ordinal, and the terminal failure status. The
  physical measurements that selected the two-bank exact-live representation are
  retained in full at
  [Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md);
  only the owner of the pool changed, and no measurement depended on it.
- **Withdrawn outright** — the `C + T > capacity` Tiler refusal. Tiler is handed
  one tensor per invocation at one bound extent and has no capacity to compare
  against. The semantic bound `S ≤ max_position_embeddings` is a different
  refusal and is unaffected.

### The preserved draft

The unmerged branch `tkt/define-the-runtime-kv-state-boundary` is retained as
review evidence and must not be merged or deleted. It is the concrete draft that
made the ownership question decidable — three independent API reviews at
`fc242fd1`, `59b0e4d8`, and `dca26e5a` — and a later reader evaluating any
proposal to reintroduce runtime-held state should read it rather than
reconstruct it.

### What would reopen the question

Nothing about transformers. A reopening needs evidence that the *generic*
runtime must retain typed state across invocations for a reason no consumer can
discharge — for example, a device-resident resource whose correctness depends on
a lifetime longer than one invocation and that the adapter cannot scope. That
would be a consumer-neutral runtime-state question, and it supersedes this
ticket's framing rather than restoring it.
