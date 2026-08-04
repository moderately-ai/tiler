---
id: bind-repeated-invocations-over-caller-retained-tensors
title: Bind repeated invocations over caller-retained tensors from one artifact identity
status: todo
priority: p1
dependencies: [admit-the-sequence-extension-concatenate-family, admit-live-extent-operands-to-payload-indexing, establish-a-dynamic-kv-physical-layout-authority, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, assemble-the-causal-self-attention-block-program, expose-the-dispatch-record-on-a-decoded-artifact, evaluate-retained-shape-relations-before-routing-commit]
scopes: [implementation/artifact, implementation/runtime, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, artifact, runtime, abi, consumer-neutral, language-model, class-generic-capability]
---
## User-visible outcome

A caller that holds a tensor between invocations and rebinds it at a longer
extent each time runs every invocation from **one** artifact identity and one
prepared pipeline — not one of each per extent — and each invocation addresses
exactly the live payload it bound, never the allocation that happens to contain
it.

## Why this ticket was rewritten

**Superseded scope, 2026-08-04, under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).**
This ticket previously read "Bind the KV cache through the artifact and runtime
interface" and required a KV-specific artifact-schema extension: an encoded
state-interface manifest naming cache inputs and layers, a
`DecodedProgram::state_interface` view, `KvArtifactStateBindingSet`,
`KvStateSet`, `KvRoutedOutputIdentity`, and a `StateTransactionReporter`, plus
an artifact canonical-identity version step to carry them.

All of it is withdrawn, and the withdrawal removes work rather than deferring
it. The manifest existed to stop a caller from supplying a partial, reordered,
or duplicated cache-binding population. A program's ordered named inputs and
outputs *already* state that population, and binding already refuses a wrong
count, a wrong key, a wrong rank, a wrong stored scalar, and a wrong literal
extent — for every program, not for caches. A second, KV-named authority over
the same subject would have been the duplication the corpus keeps eliminating,
and it would have put workload vocabulary into the neutral envelope schema.
**The artifact-schema and canonical-identity version step is therefore no longer
required by this ticket**, and `contracts/artifacts` is dropped from its scopes
for that reason.

What survives is the part that was always generic, and the KV workload is only
the occurrence that found it.

## Required behaviour

Consume the exact-live dense representation measured by
[`establish-a-dynamic-kv-physical-layout-authority`](establish-a-dynamic-kv-physical-layout-authority.md).
The caller owns the allocation and its reuse policy; Tiler sees one bound value
per invocation at one bound extent.

- **A routed accessible span is derived from the bound extents, never from the
  allocation length.** A caller may bind a dense payload that occupies a prefix
  of a longer resource, and the invocation must address exactly that payload.
  The retained oracle is the layout record's: at a caller pool of 73,728 bytes
  holding `[8, 14, 128]` and `[8, 15, 128]` F32 payloads, head 1 begins at byte
  7,168 and 7,680 and the accessible spans are 57,344 and 61,440 bytes. An
  implementation that derives a stride from the resource length addresses byte
  9,216, stays in bounds, reads the wrong head, and **must fail** the oracle.
- **Extents are bound at `AvailabilityPhase::LiveDevicePreflight`**, and every
  accessible-range and launch expression is a formula over them evaluated during
  preflight, so an evaluation failure is a refusal rather than a post-commit
  surprise.
- **No kernel may be specialized on a value that is a per-invocation binding.**
  [The runtime execution contract](../docs/research/runtime/runtime-execution-contract.md)
  keys a prepared pipeline on its specialization values, so specializing on a
  bound extent would mint one pipeline per invocation and make a caller's
  mutable quantity part of a cache key. Refuse it at artifact assembly, where the
  specialization values are packaged and the check is decidable. This is a
  general rule about bound extents; it names no workload quantity.
- **Guarded variants discriminate on bound extents at route time, not at build
  time.** Package the tiled realization guarded on a contracted extent
  `≡ 0 (mod 16)` and the direct realization otherwise, selected per invocation
  under `RoutingPolicy::StablePriority`. Across the C1 conformance row's nine
  invocations the tiled guard holds exactly once, at extent 16 — which is what
  makes "one artifact, several plans" a testable claim rather than a slogan.
- **Reuse the live-extent operand transport.**
  [`admit-live-extent-operands-to-payload-indexing`](admit-live-extent-operands-to-payload-indexing.md)
  is the only generic address-operand prerequisite. Do not define a capacity
  stride, a second physical-layout root, a storage-descriptor grammar, or a
  workload-named scalar spelling.

## Non-goals

Any type naming a cache, a cursor, a capacity, a generation, a layer ordinal, a
decode step, or a session. Any artifact-schema row describing retained state.
Any runtime object that outlives one invocation.

## Closes when

One assembled artifact routes at every extent of the C1 conformance row's nine
invocations with one identity and one prepared pipeline; the guard selects the
tiled variant at extent 16 and the direct variant elsewhere; a program
specializing a kernel on a bound extent is refused at artifact assembly with its
own diagnostic, watched failing against a deliberate perturbation; an invocation
binding a payload inside a longer resource addresses the exact live span, and
the wrong-stride interpretation is exercised and fails the retained oracle; and
a test asserts the single artifact identity across all nine invocations so that
a per-invocation compilation fails rather than passes quietly.
