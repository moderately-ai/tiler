---
id: record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims
title: Record ADR 0097 as implemented and correct the navigation docs' staging-relation claims
status: in-progress
priority: p2
dependencies: []
related: [implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5, admit-a-two-dimensional-cooperative-staging-relation]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, scheduling, ir, identity]
claimed_from: todo
assignee: agent-adr0097
lease_expires_at: 1785946214
---
## User-visible outcome

A reader of the accepted-decision index and of the two navigation documents learns that the two-dimensional cooperative staging relation is implemented and that the scheduled-region domain is `tiler.schedule.v5` — rather than reading, as they do now, that the relation is a type-system reservation that does not compile and that a `StagedSpan` addresses `stride * l + offset`.

## Why this is a separate ticket

[`implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5`](implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5.md) landed the relation and the `v4` to `v5` identity step, and holds `implementation/ir`, `contracts/artifacts`, `implementation/build`, `implementation/metal`, and `research/runtime`. The three files below are under `contracts/decisions` and `contracts/navigation`, which it does not hold — so it filed this rather than reaching outside its scopes or absorbing the staleness silently.

## What is stale, each read at the implementing commit

- **`docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md`** carries `implementation_status: "not-started"`, and its whole **Implementation boundary** section is now false in every particular. It states that "every construct the decisions name is a type-system reservation that does not compile", that `StagedSpan` has exactly three fields, that `LocalCoordinateSource` has exactly one variant, that `ParticipantSpace`, `MAX_COOPERATIVE_PARTICIPANT_RANK`, `SpanRank`, and `LocalWorkgroupPosition` "occur nowhere under `crates/`", and that "no pinned identity has moved". Each was true at `6f2601a` and each is false now. Note the asymmetry AGENTS.md names: a disclosure required while a decision is unimplemented becomes wrong once it is implemented, and nothing checks either direction.
- **`docs/status.md:22`** states that a `StagedSpan` "addresses `stride * l + offset` over the linear participant coordinate" and that [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) "owns the relation and carries the `tiler.schedule` domain step every candidate widening of it forces". The relation landed and the step is executed; that ticket owns neither any more.
- **`docs/roadmap.md:421`** repeats the same `stride * l + offset` claim and the same ownership attribution inside the contraction row, and additionally frames `tiled` as blocked on the relation. The relation is no longer what blocks it — the second tile relation ([`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md)) and the schedule and emission ([`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)) are.

## What must not be overstated

The relation is **statable**; nothing lowers a rank-two tile. `crates/tiler-ir/src/kernel/lower.rs` refuses a span whose stride vector is not rank one by name, because the canonical body reads a linear local index and has no form for a per-dimension position. So the correct claim is that the *vocabulary* landed, not that a tiled contraction is emittable — those are two of the four maturity claims AGENTS.md forbids conflating.

Two deferrals ADR 0097 records are also unchanged and must survive the edit: the extents' relation to the launch geometry is a product equality only, because `LaunchPlan` carries no threadgroup shape; and the round-dependent span and per-access active-participant subset stay refused.

## Closes when

`implementation_status` on ADR 0097 reflects the landed implementation, its Implementation boundary section describes the tree as it is (or is replaced by a statement of what landed and where), the decisions catalog row agrees, and neither navigation document still asserts the one-dimensional staging relation or attributes the domain step to a ticket that has completed it.
