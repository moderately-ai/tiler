---
id: scope-a-windowed-kv-append-into-retained-capacity
title: Scope a windowed KV append into retained capacity
status: deferred
priority: p2
dependencies: [prove-the-c1-stateful-attention-vertical, establish-a-dynamic-kv-physical-layout-authority]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization, measure-b1-d-peak-residency-on-a-named-host]
scopes: [research/runtime, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, kv-cache, buffer-reuse, performance, language-model]
---
## User-visible outcome

The decode step avoids survivor-specific replacement work — **if** the selected physical representation shows that work is a binding measured cost and an in-place/windowed update has a complete partial-update recovery contract. Performance and recovery evidence are both required.

## Activation trigger

This ticket remains deferred until all three conditions hold:

1. [`establish-a-dynamic-kv-physical-layout-authority`](establish-a-dynamic-kv-physical-layout-authority.md) has selected a representation and recorded its exact addressing, resource population, alias law, retention, and publication consequences;
2. a reproducible B1 measurement shows that the survivor's replacement traffic, allocation behaviour, or peak residency is a binding cost; and
3. a complete recovery contract makes a post-commit partial update non-destructive or otherwise safely recoverable.

[The L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md) retains the earlier 1,908,178,944-byte read, 1,908,408,320-byte write, 1.60× weight-traffic, and 3,816,587,264-byte peak figures only as arithmetic for the rejected singular dense-allocation candidate. Those figures demonstrate why the question was filed; they do not fire this trigger or predict the survivor's cost.

## What it owes, and none of it is optional

Every survivor-specific in-place or windowed proposal must first state its exact write regions, untouched-region proof, resource alias law, retention, atomic publication, and failure recovery. If the survivor uses the historical caller-bound-input/singular-allocation spelling, it additionally confronts four implemented refusals, three handed over by [the sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) and the fourth found at L5:

1. `ExternalValueWritten` refuses writing a caller-bound input.
2. `MultipleWriters` refuses a second stage writing the rest of a partially written value.
3. Nothing proves the untouched bytes of a partially written value; `WriteOwnershipProof::{CoordinatePermutation, Exhaustive}` prove one access total and injective over its *own* boundary and cannot express "total over a partition and disjoint from a sibling".
4. `verify_storage` refuses a program input and a program output sharing one allocation as `ForbiddenAlias` under that candidate's roles.

These four are historical candidate constraints, not permission to assume one allocation or one write shape. Every alternative owes an exact negative oracle for its own addressing and alias model. The survivor-independent obligation is that **a post-commit failure must not leave plausible partially updated state**. ADR 0033 is explicit that initial transactions are out of place and that mutation requires a separate shadow, undo, versioning, or equivalently safe recovery capability. Relaxing verifier refusals does not supply one.

## Also in scope

This ticket does not choose the cache layout. `[8, S, 128]` row-major versus `[S, 8, 128]`, whole-cache copying, and one singular retained allocation were historical candidates. The layout authority owns their elimination or survival. This ticket consumes the selected representation and asks only whether a safe mutation capability improves it at a named measured row.

## Closes when

After the activation trigger actually fires, either a survivor-specific plan is delivered with its address/layout negative oracle, recovery contract, and measured saving at a named row, or it is refused with a durable reason and a restated trigger. Until then the ticket remains `deferred`. Q-PLAN-015 remains the owner of general in-place execution; this ticket does not widen it.
