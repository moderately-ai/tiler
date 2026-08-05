---
id: separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ
title: Separate the tree and split groupings at a contributor count where their partitions differ
status: deferred
priority: p3
dependencies: []
related: [drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies, raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells, calibrate-and-activate-parallel-reduction-selection, establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
tags: [numerics, reductions, deferred, evidence-gap]
---
## Deferred: the activation trigger has not fired

Filed `deferred` rather than `todo` because the case this ticket asks for **cannot be constructed at any shape the current profile admits.** Do not claim it until the trigger below has fired.

## The gap, stated precisely

**Measurement, 2026-08-02, Apple M4 Max / macOS 27.0 `26A5388g` / `Apple metal version 32023.883` / `nightly-2026-07-19`.** [`drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies`](drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies.md) drove operands `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` through all three alternatives on the qualified host:

| Alternative | Declared partition | Answer |
| --- | --- | --- |
| serial fold | 4 of 1 | `3f800000` |
| single-workgroup tree | 2 of 2 | `3f800001` |
| multi-pass split | 2 of 2 | `3f800001` |

Each matched **its own** declared grouping bit for bit, which is what made it the first observation in the corpus of a reassociation-permitting program producing a different-but-permitted answer.

**Fact — at four contributors the tree and the split declare the *same* partition.** Both take their split from `governed_partition`, and the cooperative tile's `rounds: 1` makes the tree's grouping identical to the split's. So that case separates the two parallel strategies from the serial fold, and **does not separate them from each other**. Nothing about that is a defect in the case; it is a property of the shape.

**Fact — four contributors is the ceiling this profile admits for the shape.** The declared Metal profile's `grid_axis_threads` row is `4`, and the split's pointwise stage launches one invocation per element, so a wider row fails `target.grid-axis` before any plan composes. That bound is what makes a discriminating contributor count unreachable rather than merely unchosen.

## Trigger for reconsideration

**A contributor count at which `governed_partition` yields a different partition for the tree than for the split becomes reachable.** The concrete route is [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md), which widens the grid-axis row that currently caps the shape at four. A tile whose `rounds` exceeds one would also fire it, by making the tree's grouping differ from the split's at a count the profile already admits — check which arrived before assuming it was the grid-axis row.

## What this ticket would then owe

- A contributor count where the two declared partitions genuinely differ, **read from each plan's own published launch geometry** rather than assumed — the existing case established that discipline and it is not to be relaxed.
- Operands grouping-sensitive at *that* count, stated as exact `f32` bit patterns.
- Each strategy's answer matched against **its own** declared grouping via `tiler_reference::strict_partitioned_sum`, not against a tolerance and not against a shared oracle. The existing case's elimination — why a derived bound and why permitted-set membership were both discarded — holds here and must not be re-litigated.
- The discriminating refusal watched firing: one strategy held to the *other's* partition must be refused, and the refused value must itself be legal under the contract. That is the wrong-but-in-range refusal, and it is the whole point of the exercise.

## Explicit non-goals

No cost model and no selection change — [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md) owns those. No new strategy. This ticket adds discriminating *evidence*; it does not widen the grid-axis row itself.

## Graph maintenance

- Filed 2026-08-02 at integration of the producing ticket, which recorded the limit rather than hunting around it and asked whether it should become a ticket. It should: a bounded capability with a named activation trigger belongs on the board rather than in a closed ticket's prose.
- Do **not** convert this to `todo` because the grid-axis ticket is merely claimed. The trigger is a *reachable* discriminating count, which means that work landed and the count verified reachable — check it, do not infer it.

## Trigger check log

- 2026-08-04 — **not fired, and the named concrete route landed and turns out to be insufficient.** The grid-axis row this ticket blamed *did* move: [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) is `done` and the authoritative declaration now carries a measured `grid_axis_threads: 268_435_456` (`crates/tiler-build/src/metal_declaration.rs:225`, was `4`), so wider shapes are reachable and more than one shape now retains all three strategies. **That does not produce a discriminating count.** The ticket's own trigger is a count at which the *tree's* partition differs from the *split's*, and both still read the identical value from one function — `single_workgroup_tree_region` calls `governed_partition(contributors)` (`crates/tiler-compiler/src/physical.rs:1159`) exactly as `partial_reduction_region` does, and `workgroup_tree_tile` fixes `rounds: 1` (`crates/tiler-ir/src/schedule/cooperative.rs:887`), which is precisely the condition the ticket recorded as making the two groupings identical. Widening the row moves the count but not the divergence, at **every** count. The surviving route is therefore the ticket's second one alone: a cooperative tile whose `rounds` exceeds one. The ticket's own instruction — "check which arrived before assuming it was the grid-axis row" — is what caught this. Recheck: `grep -n 'governed_partition(contributors)' crates/tiler-compiler/src/physical.rs` and `grep -n 'rounds: 1' crates/tiler-ir/src/schedule/cooperative.rs`.
