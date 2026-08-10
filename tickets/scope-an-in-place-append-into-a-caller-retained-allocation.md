---
id: scope-an-in-place-append-into-a-caller-retained-allocation
title: Scope an in-place append into a caller-retained allocation
status: deferred
priority: p2
dependencies: [prove-the-c1-stateful-attention-vertical, establish-a-dynamic-kv-physical-layout-authority, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-autoregressive-state-and-kv-cache, scope-the-sequence-extending-tensor-family, design-model-level-qualification-and-optimization, measure-b1-d-peak-residency-on-a-named-host, admit-a-partitioned-write-ownership-contract, accept-the-partitioned-write-ownership-proof-boundary]
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
recovery. Four standing barriers block the historical caller-bound-input
spelling, three named by
[the sequence-extending family record](../docs/research/shapes/sequence-extending-tensor-family.md)
and the fourth found at L5:

1. `ExternalValueWritten` refuses writing a caller-bound input — which is now the
   *whole* subject rather than one candidate's incidental shape, because after
   the supersession every retained allocation is caller-bound.
2. `MultipleWriters` refuses a second stage writing the rest of a partially
   written value.
3. Nothing proves that unwritten regions of a **caller-bound input** retain prior
   content after a partial in-place write.
   `WriteOwnershipProof::PartitionMember` (landed by
   [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md)
   and accepted at
   [`accept-the-partitioned-write-ownership-proof-boundary`](accept-the-partitioned-write-ownership-proof-boundary.md))
   proves partition-relative totality plus joint disjoint coverage of an
   **output** among co-owning roots. That closes the concatenate-style
   partitioned-output gap once attributed only to
   `CoordinatePermutation` and `Exhaustive`. It does not prove preservation of
   prior content in unwritten regions of a caller-bound input, and it does not
   relax `ExternalValueWritten` or `ForbiddenAlias`.
4. `verify_storage` refuses a program input and a program output sharing one
   allocation as `ForbiddenAlias`.

The survivor-independent obligation is that **a post-commit failure must not
leave a plausible partially updated allocation in the caller's hands**. ADR 0033
is explicit that initial transactions are out of place and that mutation requires
a separate shadow, undo, versioning, or equivalently safe recovery capability.
Relaxing verifier refusals does not supply one, and the supersession makes the
requirement sharper rather than weaker: the caller cannot inspect the allocation
to learn which bytes are new, and Tiler retains nothing that could tell it.

**Correction — 2026-08-10.** Item 3 above no longer claims that only
`CoordinatePermutation` and `Exhaustive` exist.
`WriteOwnershipProof::{CoordinatePermutation, Exhaustive, PartitionMember}` are
live; `PartitionMember` is the third proof kind and discharges joint partitioned
**output** ownership only. The remaining gap for this ticket is proving
preservation of prior content in unwritten regions of a **caller-bound input**,
together with the still-standing `ExternalValueWritten`, `ForbiddenAlias`, and
recovery obligations. Trigger conditions 2 and 3 remain unmet; the activation
trigger is still **not fired**.

## Also in scope

This ticket does not choose a layout. The layout authority owns that elimination.
This ticket consumes the selected representation and asks only whether a safe
mutation capability improves it at a named measured row.

## Closes when

After the activation trigger actually fires, either a plan is delivered with its
address/layout negative oracle, recovery contract, and measured saving at a named
row, or it is refused with a durable reason and a restated trigger. Until then
the ticket remains `deferred`.

## Trigger check log

- 2026-08-04 — **not fired.** Condition 1 is recorded fired above; conditions 2 and 3 remain unmet on the same day — no reproducible measurement shows the survivor's replacement traffic, allocation behaviour, or peak residency is a binding cost, and no recovery contract makes a post-commit partial write safely recoverable. All three must hold. Recheck: the three numbered conditions above.
- 2026-08-09 — **not fired.** The dynamic layout authority remains settled, but the C1 stateful vertical is still `todo`; no named-host measurement demonstrates that replacement traffic/allocation/residency is the survivor's binding cost, and no shadow, undo, versioning, or equivalent recovery contract has landed. Conditions 2 and 3 remain independently unmet.
- 2026-08-10 — **not fired.** `WriteOwnershipProof::PartitionMember` has landed and is an accepted public surface (`admit-a-partitioned-write-ownership-contract`, `accept-the-partitioned-write-ownership-proof-boundary`); that closes the partitioned-**output** ownership gap formerly misstated as absent in "What it owes" item 3. Condition 1 remains fired; conditions 2 and 3 are still unmet — no named-host binding-cost measurement of the survivor's replacement traffic/allocation/residency, and no shadow/undo/versioning recovery contract for post-commit partial write into a caller-held allocation. All three conditions must hold.
