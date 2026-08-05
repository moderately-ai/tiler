---
id: scope-the-data-dependent-extent-representation
title: Scope the data-dependent extent representation
status: deferred
priority: p2
dependencies: []
related: [scope-the-ordering-and-rank-selection-families, scope-the-scatter-and-indexed-update-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, shapes, dynamic-extent, deferred]
---
## User-visible outcome

`RQ-OP-10` is answered: a result whose extent is a function of operand values is represented by a bounded extent plus a validity count, by an explicit future value, or by neither — and the whole class of families that turns on it (`NonZero`, `Compress`, `Unique`, the dynamic `k` of a rank selection) stops being blocked on a question with no owner.

## Why this is deferred rather than open

**Fact — the blocker is a shape mechanism rather than an operation design.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-36 is classified "Neither, today", because "**The result extent is a function of the operand values**, which no accepted shape mechanism expresses"; there is "no physical fallback, because an allocation size is unknown before execution". The [contract memo](../docs/research/semantic-graph/contract-memo.md) already rules `NonZero` "outside the first dynamic shape contract unless its result is bounded and represented by an explicit future mechanism".

**Fact — a compiler-facing portable IR omits the whole class, and that is evidence rather than an anecdote.** ONNX defines `NonZero`, `Compress`, `Unique`, and `TensorScatter` and the Array API defines `nonzero`, `unique_all`, `unique_counts`, `unique_inverse`, `unique_values`, and `count_nonzero`; StableHLO defines none of them. The taxonomy's inference is that the family "is coherent and widely used at the framework level, and it is not expressible as a *tensor* result in a system whose allocations are planned before execution".

**Fact — `RQ-OP-10` names both candidates and what closes it.** "a bounded extent plus a validity count, which keeps allocation static and makes every consumer mask-aware; or an explicit future value... Closes when a named workload requires one, with the consequence for allocation, ABI, and every downstream consumer's shape inference stated for the chosen candidate."

**Inference — the first candidate has a cost the second does not, and it is not paid by this family.** A bounded extent plus a validity count makes *every downstream consumer* mask-aware, which reaches families that have nothing to do with data-dependent extents. That is why this is a shape-and-allocation decision rather than an operation admission, and why it is filed against the shape questions rather than beside `NonZero`.

## Activation trigger

A named workload requires a data-dependent result extent — a filtering step, a deduplication, a variable-length selection, or a rank selection whose `k` is an operand rather than an attribute.

## What the work would be, when it starts

Choose between the two candidates against a named workload, and state for the chosen one: the allocation consequence, the ABI consequence, and — the expensive one — what every downstream consumer's shape inference must now do. Then say which of `NonZero`, `Compress`, `Unique`, and a dynamic-`k` selection the choice actually admits, because they differ in whether the count is one number or one per row.

## Explicit non-goals

- Admitting any of the named operations. The representation decision precedes them and admitting one first would fix the representation by accident.
- Dynamic rank, which is [Q-SHAPE-004](../docs/open-questions.md)'s.
- Device-produced extents and indirect dispatch, which are [Q-SHAPE-005](../docs/open-questions.md)'s and are a different mechanism even where they would serve the same workload.

## Closes when

One representation is chosen against a named workload with its allocation, ABI, and downstream shape-inference consequences stated, and `RQ-OP-10` names the answer — or the class is recorded as permanently outside the product with the reason.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-30** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-36 alone, plus the dynamic-`k` case of F-38 and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires a data-dependent result extent; the pinned workload's every extent is either static or a bound symbol under the sourced-extent profile, and its one growing extent `S` is related additively to `C` and `T` rather than derived from values. Recheck: `rg -n 'Q-SHAPE-004|Q-SHAPE-005' docs/open-questions.md` — both triggers remain stated and unfired.
