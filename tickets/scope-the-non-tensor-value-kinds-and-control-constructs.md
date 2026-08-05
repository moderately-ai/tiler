---
id: scope-the-non-tensor-value-kinds-and-control-constructs
title: Scope the non-tensor value kinds and control constructs
status: deferred
priority: p2
dependencies: []
related: [scope-the-effect-signature-opening, scope-the-ordering-and-rank-selection-families, multi-device-and-sharding-scope-gate, scope-the-data-dependent-extent-representation, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, value-kinds, control-flow, deferred]
---
## User-visible outcome

[Q-SEM-012](../docs/open-questions.md#q-sem-012--semantic-modules-calls-and-control-flow) acquires a ticket: tokens, tuples, sequences, optionals, shape values, and futures are scoped as *graph value kinds* and regions as a separate mechanism, so that the first workload needing one does not get a tensor with a comment.

## Why this is deferred rather than open

**Fact — these are value kinds rather than element types, and both axes already say so.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-47 records that "Tokens, tuples, sequences, optionals, shapes, and futures are graph value kinds rather than tensor element types", that "a data-dependent trip count is the shape problem of F-36 in another guise", and that "control flow changes what determinism even means, because a branch may be value-dependent". The dtype axis routes the same members off its own axis to this row.

**Fact — the region half is bounded by an accepted position.** [IR](../docs/ir.md) states the initial compilation unit "has no semantic functions/calls, recursion, region-bearing control flow, data-dependent branches, or semantic loops", and the [contract memo](../docs/research/semantic-graph/contract-memo.md) records that "General sequences, maps, optional values, graph-valued attributes, nested regions, calls, and control flow are not required to prove the tensor optimizer architecture".

**Fact — one family is already waiting on the region half specifically.** `RQ-OP-11` asks whether sorting takes a fixed order or a caller-supplied comparator region, and records that a region "would be the first nested region in the public graph and touches canonical identity, verification, and the extension seam" — consequences that "far exceed one family". So the region question has at least one identified consumer and it is not control flow.

**Inference — the two halves are separable and should be scoped as two.** A value kind can be added without a region (a token, a tuple result), and a region can be added without a new value kind (a comparator over ordinary scalars). Treating them as one "advanced features" bucket is exactly what the taxonomy's conclusion 7 warns against.

## Activation trigger

Q-SEM-012's own trigger: a workload requires reusable graph functions, interprocedural optimization, recursion, or structured control flow — **or**, for the value-kind half alone, a family this record already tracks requires a non-tensor result, the two identified candidates being a comparator region for sorting and an effect token for collectives.

## What the work would be, when it starts

Scope the two halves separately. For the value kinds: which kinds exist, how each participates in canonical identity, what the ABI carries for one, and what a verifier proves about its lifetime — with shape and index values treated as distinct newtypes even where their physical representation coincides, which the corpus already requires. For regions: what a nested region does to canonical identity, to verification, and to the extension seam, priced against the one identified consumer rather than in the abstract, and with the determinism consequence of a value-dependent branch stated before any branch exists.

## Explicit non-goals

- The effect vocabulary, which is [`scope-the-effect-signature-opening`](scope-the-effect-signature-opening.md)'s; a token is a value kind and an effect is a signature, and needing both is not the same as them being one thing.
- Data-dependent extents, which are [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md)'s even though a data-dependent trip count is the same problem wearing a control-flow hat.
- Any semantic loop across invocations, which Vision excludes.

## Closes when

The value-kind half and the region half are scoped separately, each against a named consumer, with the canonical-identity, verification, ABI, and determinism consequences stated — closing Q-SEM-012 or restating it to name only what remains.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-38** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-47 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired, on either route.** No workload requires graph functions, recursion, or structured control flow, and neither identified value-kind consumer is live: `RQ-OP-11`'s comparator region is deferred under [`scope-the-ordering-and-rank-selection-families`](scope-the-ordering-and-rank-selection-families.md) and the collective token under [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md). Recheck: `rg -n 'Q-SEM-012' -A4 docs/open-questions.md`.
