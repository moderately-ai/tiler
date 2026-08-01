---
id: scope-a-windowed-kv-append-into-retained-capacity
title: Scope a windowed KV append into retained capacity
status: deferred
priority: p2
dependencies: [prove-the-c1-stateful-attention-vertical]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization]
scopes: [research/runtime, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, kv-cache, buffer-reuse, performance, language-model]
---
## User-visible outcome

The decode step stops copying the whole cache every token — **if** the copy is measured to be the dominant cost and a partial-update recovery contract exists. Both halves are required; the residency arithmetic alone justifies the plan and does not make it safe.

## Activation trigger

A measured decode-latency or peak-residency result at a B1 row where the cache copy dominates. [The L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md) states the arithmetic that makes the trigger reachable: at B1-d the out-of-place update reads 1,908,178,944 bytes and writes 1,908,408,320 bytes per token — 1.60× the model's own F32 weight traffic — and doubles peak KV residency to 3,816,587,264 bytes if one program holds every layer's cache. At C1 the same figures are 0.27% of weight traffic, which is why this must not be scheduled on C1 evidence.

## What it owes, and none of it is optional

Four implemented refusals, three of them handed over by [the sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md) and the fourth found at L5:

1. `ExternalValueWritten` refuses writing a caller-bound input.
2. `MultipleWriters` refuses a second stage writing the rest of a partially written value.
3. Nothing proves the untouched bytes of a partially written value; `WriteOwnershipProof::{CoordinatePermutation, Exhaustive}` prove one access total and injective over its *own* boundary and cannot express "total over a partition and disjoint from a sibling".
4. `verify_storage` refuses a program input and a program output sharing one allocation as `ForbiddenAlias`.

And one obligation the out-of-place update gets free: **a post-commit failure under a windowed write leaves the retained state partially updated**, with nothing to prove which bytes are new. ADR 0033 is explicit that initial transactions are out of place and that mutation requires a separate shadow or undo capability. Relaxing the four refusals does not supply one.

## Also in scope

L5's D-14, the cache's sequence-axis layout. `[8, S, 128]` is chosen on contraction locality and is correct while the whole tensor is copied; `[S, 8, 128]` makes the append one contiguous window and removes a permute, at eight times the stride between consecutive contracted positions. The trade has two arithmetic halves and no measurement on either.

## Closes when

Either the plan is delivered with its recovery contract and a measured saving at a named row, or it is refused with a durable reason and a restated trigger. Q-PLAN-015 remains the owner of general in-place execution; this ticket does not widen it.
