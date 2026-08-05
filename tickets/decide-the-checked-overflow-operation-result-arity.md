---
id: decide-the-checked-overflow-operation-result-arity
title: Decide the checked-overflow integer operation's result arity
status: deferred
priority: p2
dependencies: [define-the-integer-numerical-contract-and-honourability-subject]
related: [derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, integers, deferred, arity]
---
## User-visible outcome

A checked-overflow integer operation has one decided shape — one result plus a validated precondition, or two results with an explicit overflow predicate — so that the integer honourability work knows what it is declaring a target honourable *about*.

## Why this is deferred rather than open

**Fact — the question is stated and unowned.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-01` asks whether a checked-overflow integer operation "return[s] one result plus a validated precondition, or two results with an explicit overflow predicate", names F-08 as the family it blocks, and states the closure test: "A worked two-operand program under each shape, showing what a downstream consumer must do with the overflow information; closes when one shape is chosen and its interaction with [ADR 0021](../docs/decisions/0021-validated-value-assumptions.md)'s proof-or-runtime-validation obligation is stated."

**Fact — the nearest ticket excludes it by name.** [`define-the-integer-numerical-contract-and-honourability-subject`](define-the-integer-numerical-contract-and-honourability-subject.md) lists under its explicit non-goals "`RQ-OP-01`'s arity question for a checked-overflow operation, which the operation axis owns and which this work must consume rather than re-decide." Nothing on the operation axis owned it until this ticket.

**Inference — deciding it before the integer track fires would decide it without the evidence that decides it.** The two shapes differ in what a *consumer* must do, and the closure test is a worked program. Absent a named integer workload there is no consumer to write one for, and a shape chosen from taste would bind [ADR 0039](../docs/decisions/0039-explicit-integer-overflow-operations.md)'s checked family to an arity nothing tested.

## Activation trigger

[`define-the-integer-numerical-contract-and-honourability-subject`](define-the-integer-numerical-contract-and-honourability-subject.md)'s own trigger fires — a named tensor workload selects an exact width, an operation family, an overflow behaviour, a storage, a target, and a corpus — **and** that workload's family list includes the checked form. A wrapping, saturating, or widening workload does not fire this: those three have one result under either answer.

## What the work would be, when it starts

Write the worked two-operand program twice, once per shape, and state for each what the consumer must do with the overflow information, what the second result costs in allocation and in fusion legality when nothing reads it, and how the one-result shape's precondition is discharged — proved at construction or carried as a runtime validation obligation under ADR 0021. Choose, record the elimination, and hand the answer to the integer track rather than implementing anything here.

## Explicit non-goals

- The honourability subject, which the integer track owns.
- Any other overflow family. Wrapping, saturating, and widening are decided by ADR 0039 and are not reopened.
- A public spelling, which is Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md).

## Closes when

One shape is chosen against a worked program under each, its ADR 0021 interaction is stated, and the integer track's ticket cites the answer rather than re-deriving it — **or** the checked family is removed from the intended surface by an explicit decision.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-10** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which maps every taxonomy family onto the eight delivery rungs and states why this partition is one track rather than several.
- The record owns the partition; [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity and this ticket moves no rung. Do not restate a rung here.
- The trigger is another ticket's trigger, so that ticket is in `dependencies:` rather than `related:`: this work cannot start without it.

## Trigger check log

- 2026-08-05 — **not fired.** The integer track's own log records its trigger unmet on 2026-08-04 and nothing has changed it: no registered operation admits a general integer operand, and the only integer keys in the semantic layer are dtype identities. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys, of which the operation keys are the F32, BF16, activation, structural, contraction, and strict-affine quantization sets and no integer arithmetic key at all.
