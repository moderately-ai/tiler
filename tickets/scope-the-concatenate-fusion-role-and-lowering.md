---
id: scope-the-concatenate-fusion-role-and-lowering
title: Scope the concatenate family's fusion role and lowering
status: in-progress
priority: p1
dependencies: []
related: [scope-an-in-place-append-into-a-caller-retained-allocation, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operation-families, concatenate, fusion, lowering]
claimed_from: todo
assignee: agent-concat-scope
lease_expires_at: 1785960717
---
## User-visible outcome

A program containing `tiler::concatenate-f32@1` derives a fusion legality other than `Unknown` and reaches a lowering, so the sequence-extension family stops being a registered identity that no plan can consume.

## Why this exists

**Fact — the family is registered and reference-evaluated and stops there.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Sequence extension` row records `tiler::concatenate-f32@1` at R4 with the exact-sum extent derivation, the typed refusals, and a bit-preserving evaluator, and states in its own words that "R5 needs a fusion role" and "R6 needs a structured-kernel construct and backend emission". The family "performs no arithmetic, so it deliberately has no `OperationNumericalCapability` row and appears in `UNPLANNED_OPERATIONS`".

**Fact — no ticket owns either rung.** Every other family whose fusion role is missing names its owner in the same row — the contraction's is [`admit-a-fusion-role-for-the-tensor-contraction`](admit-a-fusion-role-for-the-tensor-contraction.md), the structural families' backend rung is [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md). The concatenate row names an owner for the in-place append ([`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md), deferred) and for the extent relation, and none for R5 or R6.

**Fact — the lowering is a genuine fork rather than a missing keystroke, which is why this is a scoping ticket and not an implementation one.** [Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps) records the one live piecewise pressure in the corpus: "lowering the sequence-extension concatenate family needs either a piecewise read or two write roots partitioning one output. The second alternative is available, so the trigger has not fired; it fires if that alternative is eliminated." Choosing between them decides whether Q-SHAPE-006 fires, which is a consequence larger than one family.

**Inference — the demand is live rather than hypothetical.** The decode path appends to two caches per layer per step, and [`execute-the-decode-step-path`](execute-the-decode-step-path.md) and [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md) are `todo` p1 above it. A family at R4 cannot carry them.

## What the work is

Derive, and record with the elimination rather than only the choice: which fusion role the family takes given that it applies no scalar operation and its write map is a windowed partition; whether the lowering is one piecewise read or two write roots over one output, costed against what each does to Q-SHAPE-006; and whether an inner-axis concatenate's loss of the contiguous-window realization is an applicability predicate on a physical candidate rather than a second semantic identity, which the matrix row already asserts and this work must check rather than inherit.

## Explicit non-goals

- The in-place append into a caller-retained allocation, which is [`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md)'s under Q-PLAN-015.
- Any second semantic family. Stacking is a unit-axis insertion followed by a concatenation and is deliberately not a third key.
- Moving a matrix rung. A scoping record delivers nothing.

## Closes when

The fusion role is derived with its legality argument, the lowering alternative is selected with the eliminated one recorded and Q-SHAPE-006's firing condition restated against the choice, and the implementation work is filed as its own ticket with the acceptance boundary named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-07** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which maps every taxonomy family onto the eight delivery rungs and states why this partition is one track rather than several.
- The record owns the partition; [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity and this ticket moves no rung. Do not restate a rung here.
- `research/indexing` is declared because the lowering fork is an access-relation question the index model owns; `contracts/navigation` is **not** declared, because this ticket moves no matrix rung.
