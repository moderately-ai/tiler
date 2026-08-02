---
id: accept-adr-0097-two-dimensional-staging-relation
title: Accept or reject the two-dimensional cooperative staging relation ADR
status: awaiting-decision
priority: p2
dependencies: [land-the-two-dimensional-staging-relation-adr]
related: [admit-a-two-dimensional-cooperative-staging-relation, accept-adr-0096-two-level-reduction, implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5]
scopes: [contracts/decisions, contracts/navigation, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, ir, identity, decision, needs-tom]
---
## User-visible outcome

[ADR 0097](../docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md) moves from `proposed` to `accepted`, or is rejected.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision` — a parked state `tkt ready` excludes and that never satisfies a dependent; an agent that finds it in `todo` should set it back and do nothing else.

**This node releases exactly one ticket, and that is checked against the board rather than asserted.** [`implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5`](implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5.md) declares a dependency on this node, and its own body already said what its edges did not: "it does not start until the boundary is accepted". Until this node was filed that ticket depended on [`land-the-two-dimensional-staging-relation-adr`](land-the-two-dimensional-staging-relation-adr.md) — the *drafting* ticket — which is the shape `ticketsplease.toml` forbids by name in the comment above `[workflow.states.awaiting-decision]`: drafting a proposed ADR is a completed outcome, so the drafting ticket goes `done` the moment the file exists, and a dependency on it cannot distinguish "written" from "decided". The edge was repointed here in the same commit that created this node, so the block is structural rather than a worker's judgement.

**This node carries the scopes its own acceptance sweep needs.** The sweep touches `docs/decisions/[0-9]*.md` (`contracts/decisions`), both catalog READMEs (`contracts/navigation`), and the source record's frontmatter (`research/scheduling`), which is exactly what is declared above.

## The genuine choices, stated so Tom can act without re-deriving them

The seven decision items are not equally consequential. Four follow from the implemented vocabulary or from an accepted ADR and left no surviving alternative; three encode priorities and are the real questions.

- **Decision 1 and 2 together — `LocalCoordinates` carries a participant *space* and the participant *range* goes away.** This is the item with the largest surface cost: a public struct changes shape, a field the verifier already pins to zero is removed, and `ParticipantRange` survives only where a contiguous run is genuinely meant (`CooperativePhase::participation`, `SynchronizationPoint::participants`, `CooperativeTile::commit`). The derivation is that the tile's shape must be a fact a rule can check against the launch, which a divisor embedded in an address expression can never be. The alternative is not "keep both"; decision 2's own alternative entry is that keeping both is one fact stated twice and a place for two producers to disagree.
- **Decision 3 — `StagedSpan` gains a per-dimension stride vector, and four public types lose `Copy`.** The `Copy` loss is the cost most likely to be felt at call sites, and it is paid deliberately: the record's fourth alternative records that a rank-two pair preserving `Copy` would need a *second* identity-domain step to reach an ordinary three-dimensional Metal threadgroup. This is the trade to accept or refuse — one step now against a cheaper type today and a second step later.
- **Decision 7 — the `tiler.schedule.v4` → `v5` domain step.** Thirty-one pinned lines across nine files move, six of them outside `crates/tiler-ir`, and five of the six Metal goldens carry no cooperative tile and move anyway through the kernel identity's fold of the scheduled-region bytes. The step is separately covered by the relayed 2026-08-01 acceptance recorded below; what is *not* covered by that relay is the exact spelling the other six items state.

## Before deciding, read this provenance note

**The acceptance already relayed does not reach this record's contents, and the ADR says so in its own status block.** [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) records that Tom accepted the `v4` → `v5` step and the widened boundary at the live session on 2026-08-01, witnessed and executed by the coordinator. That is a **relay** as far as ADR 0097 is concerned — nobody who wrote the file witnessed it — and the same ticket sentence says the exact `StagedSpan`/`LocalCoordinates` boundary "comes back to Tom as a draft under ADR 0075". So the relay settles the *step in principle* and this node settles the *spelling*. Nothing has been released on the relay and no encoding has moved, which is what keeps the rollback below cheap.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted` on ADR 0097, sets `adopted_by: ["ADR-0097"]` on [a two-dimensional cooperative staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) and moves its `disposition` from `pending` to `adopted`, and updates both catalog views in [the decisions index](../docs/decisions/README.md) — the theme row under "Physical planning and lowering" and the chronology row — from `proposed` to `accepted`.

It implements nothing. It admits no `ParticipantSpace`, no second `LocalCoordinateSource` variant, no stride vector, no `SpanRank`, and no `MAX_COOPERATIVE_PARTICIPANT_RANK`, and it moves no version string and no pinned identity. What it releases is [`implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5`](implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5.md), which owns all of that and which must execute the identity step completely or not at all.

## Rollback, kept cheap on purpose

If the acceptance relayed to whoever executes it turns out to be wrong, the repair is one field and three catalog rows: `decision_status` back to `proposed`, the two ADR catalog rows back to `proposed`, the research record's `disposition` back to `pending` with `adopted_by` removed, and this node back to `awaiting-decision`. The one ticket this node releases has not started, because this node parks it.

## Closes when

Tom accepts or rejects it.
