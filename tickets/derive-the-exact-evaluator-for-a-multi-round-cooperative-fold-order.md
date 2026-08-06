---
id: derive-the-exact-evaluator-for-a-multi-round-cooperative-fold-order
title: Derive the exact evaluator for a multi-round cooperative fold order
status: deferred
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ, accept-adr-0100-multi-round-reduction-composition]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, reference, reductions, conformance]
---
## User-visible outcome

An exact host evaluator for every reduction order a schedule can declare, so that a plan whose topology the reference cannot evaluate is refused by name rather than compared against the nearest order that happens to exist.

## Why this is a deferral rather than work

**Fact — `strict_partitioned_sum` expresses exactly one realization shape.** It folds `partition * chunk + within` (`crates/tiler-reference/src/evaluate.rs:484`), i.e. blocked uniform contiguous partitions, serial within a partition, ascending across them, at the element width. `ContributorPartition::covers` admits only an exact product, so a non-uniform split is unrepresentable by construction.

**Fact — three declared shapes fall outside it.** `ReductionTopology::CooperativeWorkgroup` (`crates/tiler-ir/src/schedule/model.rs`) documents that on a loop-carried tile "participant `p` of round `r` owns the contiguous range at index `r * partitions + p`" over `partitions * contributors_per_partition * tile.rounds` contributors — a different index map from the flat one above. Both `MultiPass` and `CooperativeWorkgroup` carry `accumulation: ArithmeticType`, "the width every combining step is performed at", and the oracle has no such parameter. And any future non-uniform split is a third.

**Fact — none of the three is reachable *from a compiler-constructed plan*, and the distinction matters.** `grep -rn 'CooperativeTile' crates/tiler-compiler/src --include='*.rs'` returns three lines, all naming `tiler_ir::schedule::workgroup_tree_tile` (`physical.rs:1363` in a doc table, `physical.rs:1401`, `target.rs:3272`), and that constructor's body hard-codes `rounds: 1` (`crates/tiler-ir/src/schedule/cooperative.rs:887`). Accumulation is likewise pinned: `physical.rs:1654` sets `accumulation: request.numerical_contract().arithmetic`, so no plan can declare a width other than its element type.

**Fact — but the schedule vocabulary already admits and verifies a multi-round tile.** `crates/tiler-ir/src/schedule/builder.rs:4767` constructs `multi_round_tile_fixture` with `rounds: 2` and its tests verify it as a schedule. **Inference — so this deferral is one compiler construction away from firing rather than one ADR away**, which is a materially nearer position than ADR 0100's `implementation_status: not-started` alone suggests, and it is why the trigger check below names the compiler's construction sites rather than searching for a literal.

**Fact — filed `deferred` rather than `todo` because the board must not offer non-work.** There is nothing to evaluate until a topology outside the shape exists.

## Trigger

Either: a `ReductionTopology` in `crates/tiler-ir` realizes ADR 0100's multi-round composition with `rounds > 1` reachable from a constructed plan; or a plan declares an `accumulation` width other than its element type; or a non-uniform split is admitted.

## What this ticket must produce once fired

- An exact evaluator per newly admitted shape, written as that shape's definition rather than as an approximation of it, with the index map stated and matched against the topology's own documented map.
- A `RealizationNotEvaluable` refusal for a topology no evaluator covers, watched failing.
- A case at which the new shape and the existing blocked shape produce different bits, so the addition is evidence about something.

## Trigger check log

- 2026-08-05 — **not fired, and the check names its population rather than relying on an empty result.** `grep -rn 'CooperativeTile' crates/tiler-compiler/src --include='*.rs'` returns exactly **three** lines, all naming `workgroup_tree_tile`, whose body fixes `rounds: 1` — so every tile any compiler path can build is single-round. A count other than three, or a line naming any other constructor, is the fired verdict. **The first form of this check was wrong and is recorded rather than replaced silently**: `grep -rn 'rounds: *[2-9]' crates/ --include='*.rs'` returns three hits, of which two (`crates/tiler-compiler/src/legality.rs:1617`, `crates/tiler-compiler/src/pipeline/conformance.rs:1322`) are an unrelated `rounds` field on a test lowering fixture and one (`crates/tiler-ir/src/schedule/builder.rs:4767`) is a `tiler-ir` schedule test. An emptiness check over that pattern would have read as fired when it is not, and a naming check over the compiler's construction sites is what distinguishes the two.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as refusal class 2's population.
