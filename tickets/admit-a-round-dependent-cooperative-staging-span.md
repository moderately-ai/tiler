---
id: admit-a-round-dependent-cooperative-staging-span
title: Admit a round-dependent cooperative staging span
status: deferred
priority: p2
dependencies: []
related: [admit-a-two-dimensional-cooperative-staging-relation, implement-the-single-workgroup-synchronized-reduction-strategy, admit-loop-carried-cooperative-staging]
scopes: [implementation/ir, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, identity, deferred]
---
## User-visible outcome

A cooperative tile can state a staged access whose stride and count are functions of the **round ordinal** — the half of the log-depth tree's gap that a participant-space widening does not reach.

## Why this is separate from the participant-space widening, derived rather than assumed

[A two-dimensional cooperative staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) §2 runs the derivation. Its result, in three facts:

**Fact — the tiled contraction does not need this.** `spikes/scheduling/metal_contraction_vertical/kernels.metal:128-133` writes `a_tile[local_m * TILE + local_n]` and `b_tile[local_n * TILE + local_m]` inside the `k0` loop with the identical slot indices it used before the loop at lines 116-119. What varies with the round is the device address the value is loaded from, never the staged slot. So the participant-space widening is complete for that consumer and this one is genuinely a second capability rather than a deferred half of the first.

**Fact — the log-depth tree does need it, and needs a second thing too.** `crates/tiler-ir/src/schedule/cooperative.rs:70-75` states the tree needs "a per-access active-participant subset, separate from a phase's `participation`" **and** "a span whose stride and count are functions of the round ordinal, since each level halves them", and that "both are absent rather than reserved".

**Inference — a round-dependent span alone changes the decision procedure the whole cooperative verifier rests on.** `crates/tiler-ir/src/schedule/builder.rs:1205-1210` documents the occupancy map as spanning "the phase sequence once, which is exactly one round — every phase runs on every round — so this needs no round dimension and gains none". That is sound *because* a span is the same on every round. A round-dependent span writes different slots on different rounds, so the coverage half — every in-range slot written on every round — becomes false as stated, and coverage has to be re-derived as a union over rounds while disjointness stays per round.

## What this owns

- The relation's shape: whether the round ordinal is a term in the span's address expression, a per-round span list, or a stride/count pair scaled by a stated function of the round — and what each makes decidable.
- **The re-derived coverage rule.** Union-over-rounds coverage with per-round disjointness, stated so each half can be watched refusing its own defect, and stated so it still refuses a slot no round ever writes.
- **The per-access active-participant relation, or an explicit derivation that this widening does not need one.** ADR 0096 decision 3 *derives* its narrowing from the width and the result lane rather than declaring a subset, which is a different resolution from the log-depth tree's; whichever this lands, it lands with the round dependence rather than after it, because a round-dependent span over a full participant set does not narrow anything.
- Whether this steps the scheduled-region identity domain again, and if so its own complete blast-radius enumeration on the tree it lands into.

## What this does not own

The participant-space widening ([`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md)), the two-level reduction topology, and the log-depth tree's strategy and Metal body.

## Activation triggers

Deferred rather than dispatchable. It becomes work when **either** fires:

1. The log-depth tree reaches its depth limit under [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) — the concrete second consumer, and the one that makes the active-participant relation unavoidable; **or**
2. a double-buffered cooperative tile is wanted, whose staged slot layout rotates per round and which therefore needs the round dependence without needing the active-participant subset. That is the cheaper half and would change what this ticket's smallest useful slice is.

## Closes when

A round-dependent staged access is statable, coverage is decided as a union over rounds while disjointness stays per round under the governed enumeration bounds, every new rule has been watched refusing its own defect, and any identity step it forces is complete with every moved pin recomputed on the merged tree and enumerated in the report.

## Trigger check log

- 2026-08-04 — **not fired.** Trigger 1 names the log-depth tree reaching its depth limit under [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md), which is now `done` — but it landed a **depth-two** tree, not a log-depth one: `workgroup_tree_tile` fixes `rounds: 1` (`crates/tiler-ir/src/schedule/cooperative.rs:887`), and the only occurrence of "log-depth" in the crate is the absent-capability note at `cooperative.rs:79`. A dependency reaching `done` is therefore not this trigger; the depth limit is. Trigger 2 is unmet too — no double-buffered cooperative tile is proposed anywhere in the graph. Recheck: `grep -n 'rounds: 1' crates/tiler-ir/src/schedule/cooperative.rs`.
