---
id: scope-the-scatter-and-indexed-update-family
title: Scope the scatter and indexed-update family
status: deferred
priority: p2
dependencies: []
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, scope-the-effect-signature-opening, scope-the-data-dependent-extent-representation, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/indexing, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, indexing, scatter, deferred]
---
## User-visible outcome

Q-SHAPE-007's unfired half acquires an owner: a scatter is scoped as a **pure** family producing a new value, with its collision order and its write-ownership obligation stated, rather than as the in-place update every ecosystem spelling suggests.

## Why this is deferred rather than open

**Historical Fact — the question named this half as unowned when this ticket was filed.** [Q-SHAPE-007](../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations) said no ticket proposed scatter. That became false as soon as this ticket landed; the 2026-08-09 board audit repairs the durable question to name this ticket as the deferred owner while preserving that the workload trigger has not fired.

**Fact — the purity claim is the corpus's position and is mechanically enforced.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-35 marks it: "**Pure at the semantic level and this is the load-bearing claim**: a scatter produces a new value rather than mutating one." `OperationEffect` has exactly one variant and is deliberately not `#[non_exhaustive]`, so a second effect class is a compile error at three encoders — mutation is unrepresentable rather than merely unimplemented. Every ecosystem spelling that reads as mutation — `index_put_`, `scatter_`, `dynamic_update_slice` — resolves to the pure form or is out of scope.

**Fact — the aliasing obligation is already specified and is not optional.** The [index and access model](../docs/research/indexing/index-access-model.md) `## Aliasing and write ownership` states that ordinary write maps need coverage plus unique ownership and that reduction updates and atomics use separate contracts. So the physical route "is not 'a kernel that writes'; it is a kernel that either proves unique ownership or uses the separate atomic-combine contract, and a scatter admitted without choosing between those two would be admitted without a lowering."

**Fact — determinism is conditional and the condition is a value assumption.** Uniqueness declared but not proved is a value assumption under [ADR 0021](../docs/decisions/0021-validated-value-assumptions.md), not a licence; and a floating-point combiner makes the result order-dependent.

## Activation trigger

A named workload requires an indexed update as a graph operation. The pinned language-model workload does not: its KV growth is a sequence extension expressed by `tiler::concatenate-f32@1`, and its state ownership was corrected to the consumer on 2026-08-04, so the tensors a cache holds cross the boundary as ordinary program inputs and outputs.

## What the work would be, when it starts

The index-mapping structure (the same shape F-34's carries), the three independent data, index, and update types, the sortedness and uniqueness assertions as declared value assumptions under ADR 0021 with their proof-or-runtime-validation route, the combiner's per-collision numerical contract, the stated collision order without which the result is nondeterministic, and the choice between proved unique write ownership and the separate atomic-combine contract — made rather than deferred, because the family has no lowering until it is.

## Explicit non-goals

- Any in-place spelling. Mutation is unrepresentable and widening `OperationEffect` is out of bounds here; [`scope-the-effect-signature-opening`](scope-the-effect-signature-opening.md) owns that vocabulary.
- A data-dependent output shape, which is [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md)'s.
- The gather half, which is delivered work under [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md); a read map may be many-to-one with no ordering hazard and a write map may not.

## Closes when

The family is scoped as pure with a stated collision order, a declared uniqueness assumption routed through ADR 0021, and one of the two write contracts selected — closing Q-SHAPE-007's remaining duplicate-write and write-determinism items, or naming precisely which of them survives.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-29** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-35 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires an indexed update as a graph operation; the one candidate, the KV cache append, is a concatenation whose retained tensors are consumer-owned. Recheck: `rg -n 'scatter' docs/open-questions.md` — Q-SHAPE-007's scatter clause still reads that no ticket proposes one, which this ticket's landing is what changes.
- 2026-08-09 — **not fired, and the owner drift is repaired.** No workload requires an indexed update; KV growth remains concatenation/caller-owned state. `docs/open-questions.md` now names this deferred ticket as the scatter owner instead of retaining the false statement that no ticket proposes one.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -n 'scatter' docs/open-questions.md`, and run at this base it returns **4** lines. A result other than the 4 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
