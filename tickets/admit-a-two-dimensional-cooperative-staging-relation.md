---
id: admit-a-two-dimensional-cooperative-staging-relation
title: Admit a cooperative staging relation a two-dimensional tile can state
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, admit-a-cooperative-tile-over-shared-operands, realize-the-tiled-contraction-schedule-and-its-metal-emission, implement-the-single-workgroup-synchronized-reduction-strategy, admit-loop-carried-cooperative-staging]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, identity, deferred]
claimed_from: todo
assignee: agent-staging-v5
lease_expires_at: 1785685152
---
## User-visible outcome

A cooperative tile can state a staged access whose slot set depends on a participant's position in **two** dimensions — the relation every blocked GPU kernel's shared-memory read has, and the one thing the current vocabulary cannot express.

## The gap, with the exact refutation

**Fact.** `StagedSpan` (`crates/tiler-ir/src/schedule/cooperative.rs`) addresses `count` contiguous slots at `stride * l + offset`, and `CooperativeTile::addressed_slots` enumerates it over the linear participant coordinate. `LocalCoordinateSource` has one variant, `LocalLinearInvocation`, and the module's own documentation states that multi-dimensional local coordinates are "absent rather than reserved".

**Fact — the refutation, from [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md).** A 16×16 tile's two staged reads need `base_a(l) = 16 * (l / 16)` and `base_b(l) = 16 * (l % 16)`. `base_a(0) = base_a(1) = 0` forces `stride = 0` while `base_a(16) = 16`; `base_b(0) = 0` and `base_b(1) = 16` force `stride = 16` while `base_b(16) = 0`. And no participant relabelling helps: over a 256-element domain `stride * l + offset` is constant or injective, while both profiles take 16 distinct values with multiplicity 16.

**Fact — two consumers are blocked by exactly this, from opposite directions.** The `tiled` contraction needs it for its operand broadcast. `workgroup_tree_tile` is depth two rather than log-depth for two reasons its own documentation records, and the first — "a span whose stride and count are functions of the round ordinal" — is the same missing dependence stated over the round instead of over a second participant dimension. Whether one relation covers both, or they are two, is part of what this ticket decides.

## What this owns

- The relation's shape. The narrowest form covering the 16×16 reads is the kernel lowering's own `OffsetTerm` — `stride * ((l / divisor) % modulus) + offset` — and the most faithful is a genuinely two-dimensional participant space with a per-dimension stride. They are not the same decision: the first keeps `ParticipantRange` one-dimensional and encodes the tile's shape in a divisor, the second makes the shape a first-class fact a verifier can check against the launch geometry. State what each enables and prevents on a concrete tile before proposing one.
- **Whether the round ordinal enters the relation**, which is the log-depth tree's half of the gap and is deliberately not assumed to be the same axis.
- The identity step. `push_staged_span` (`crates/tiler-ir/src/schedule/model.rs`) writes three unframed big-endian `u64`s with no tag, and `push_participant_range` two. Every candidate — an added field, or a tag byte in front of the existing form — moves every cooperative region's bytes. `tiler.schedule.v4` therefore steps to `v5`, and with it every pinned schedule, kernel, and artifact identity that folds one. Execute it completely or not at all: the version moves at its owning layer, the ledger documents move in the same commit, and every pinned identity is recomputed on the merged tree with each moved pin enumerated.
- Keeping the enumeration decidable. `MAX_COOPERATIVE_PARTICIPANTS` and `MAX_COOPERATIVE_STAGING_SLOTS` exist so disjointness and coverage are decided by enumerating addressed slots rather than by a modular argument. A widened relation must stay enumerable under the same bounds, and the disjointness rule must still refuse two writers reaching one slot inside one round.

## What this does not own

The second tile relation for participants that each commit their own output ([`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md)), the contraction's schedule and Metal body ([`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)), and the log-depth tree's active-participant subset, which is a separate absent capability.

## Activation triggers

Deferred rather than dispatchable, because its central act is an identity-domain step at a public boundary and [AGENTS.md](../AGENTS.md) reserves that for Tom. It becomes work when **either** trigger fires:

1. Tom accepts the `tiler.schedule.v4` → `v5` step and a widened `StagedSpan`/`LocalCoordinates` boundary; **or**
2. a second consumer independently needs it — the log-depth tree under [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) reaching its depth limit is the concrete one — which changes the cost/benefit the first trigger is a judgement about.

## Closes when

A two-dimensional staged access is statable, its disjointness and coverage are still decided by enumeration under the governed bounds, every new rule has been watched refusing its own defect, and the identity step is complete: version moved at its owning layer, ledger updated in the same commit, every moved pin recomputed on the merged tree and enumerated in the report.

## Activated 2026-08-01 — trigger 1 fired, with a co-derivation direction

**Tom accepted the `tiler.schedule.v4` → `v5` step and the widened boundary at the live session, witnessed and executed by the coordinator.** The direction he chose sharpens what this ticket owns: the relation is **co-derived with ADR 0096's two-component coordinate** — one concept serving the tiled contraction's operand broadcast, the two-level composition's staged access by named coordinate component, and the log-depth tree's round-dependent span, rather than three widenings. ADR 0096 decision 4's two-component coordinate constrains the shape fork toward the first-class participant space over the divisor/modulus encoding; the elimination is still this ticket's to run and state, and the exact widened `StagedSpan`/`LocalCoordinates` boundary comes back to Tom as a draft under ADR 0075. The identity step lands once, completely, under the full ledger discipline the body above states.
