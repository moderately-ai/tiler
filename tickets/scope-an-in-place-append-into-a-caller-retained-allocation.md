---
id: scope-an-in-place-append-into-a-caller-retained-allocation
title: Scope an in-place append into a caller-retained allocation
status: deferred
priority: p2
dependencies: [prove-the-c1-stateful-attention-vertical, establish-a-dynamic-kv-physical-layout-authority, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization, measure-b1-d-peak-residency-on-a-named-host]
scopes: [research/runtime, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [research, buffer-reuse, performance, consumer-neutral, language-model, class-performance-study]
---
## User-visible outcome

An invocation writes a new region into an allocation the caller already holds,
instead of producing a whole replacement value — **if** a measurement shows the
replacement work is a binding cost and an in-place update has a complete
partial-update recovery contract. Performance and recovery evidence are both
required.

## Scope correction — 2026-08-04

Renamed and rewritten under
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
The ticket read "Scope a windowed KV append into retained capacity", where
"retained capacity" meant an allocation a Tiler runtime state owned. There is no
such state: the allocation is the **caller's**, and a windowed write into it is a
write into a caller-bound program input. That makes the question generic — it is
about writing part of a value the caller supplied — and it is the same question
for any workload that reuses a buffer across invocations. The KV workload
supplies the motivating arithmetic and nothing else.

Q-PLAN-015 remains the owner of general in-place execution; this ticket does not
widen it. What this ticket carries is whether the conformance workload's measured
behaviour ever *fires* that question, and what a safe answer would owe.

## Activation trigger

This ticket remains deferred until all three conditions hold:

1. [`establish-a-dynamic-kv-physical-layout-authority`](establish-a-dynamic-kv-physical-layout-authority.md)
   has selected a representation and recorded its exact addressing, resource
   population, alias law, retention, and publication consequences — **fired**:
   [Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md)
   selects two capacity-sized head-major pool banks per logical member while
   packing each active payload densely at its live extent, and the banks are the
   caller's;
2. a reproducible measurement shows that the survivor's replacement traffic,
   allocation behaviour, or peak residency is a binding cost — **not fired**; and
3. a complete recovery contract makes a post-commit partial write
   non-destructive or otherwise safely recoverable — **not fired**.

[The L5 record](../docs/research/runtime/autoregressive-state-and-kv-cache.md)
retains the earlier 1,908,178,944-byte read, 1,908,408,320-byte write, 1.60×
weight-traffic, and 3,816,587,264-byte peak figures only as arithmetic for the
rejected singular dense-allocation candidate. Those figures demonstrate why the
question was filed; they do not fire this trigger or predict the survivor's cost.

## What it owes, and none of it is optional

Any in-place or windowed proposal must first state its exact write regions,
untouched-region proof, resource alias law, retention, publication, and failure
recovery. Four implemented refusals stand in the way of the historical
caller-bound-input spelling, three handed over by
[the sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md)
and the fourth found at L5:

1. `ExternalValueWritten` refuses writing a caller-bound input — which is now the
   *whole* subject rather than one candidate's incidental shape, because after
   the supersession every retained allocation is caller-bound.
2. `MultipleWriters` refuses a second stage writing the rest of a partially
   written value.
3. Nothing proves the untouched bytes of a partially written value;
   `WriteOwnershipProof::{CoordinatePermutation, Exhaustive}` prove one access
   total and injective over its *own* boundary and cannot express "total over a
   partition and disjoint from a sibling".
4. `verify_storage` refuses a program input and a program output sharing one
   allocation as `ForbiddenAlias`.

The survivor-independent obligation is that **a post-commit failure must not
leave a plausible partially updated allocation in the caller's hands**. ADR 0033
is explicit that initial transactions are out of place and that mutation requires
a separate shadow, undo, versioning, or equivalently safe recovery capability.
Relaxing verifier refusals does not supply one, and the supersession makes the
requirement sharper rather than weaker: the caller cannot inspect the allocation
to learn which bytes are new, and Tiler retains nothing that could tell it.

## Also in scope

This ticket does not choose a layout. The layout authority owns that elimination.
This ticket consumes the selected representation and asks only whether a safe
mutation capability improves it at a named measured row.

## Closes when

After the activation trigger actually fires, either a plan is delivered with its
address/layout negative oracle, recovery contract, and measured saving at a named
row, or it is refused with a durable reason and a restated trigger. Until then
the ticket remains `deferred`.
