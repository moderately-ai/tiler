---
id: accept-adr-0096-two-level-reduction
title: Accept or reject the two-level subgroup-then-workgroup reduction ADR
status: awaiting-decision
priority: p2
dependencies: [land-the-two-level-reduction-adr]
related: [compose-the-two-level-subgroup-and-workgroup-reduction, accept-adr-0094-subgroup-execution-tier]
scopes: [contracts/decisions, contracts/navigation, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, subgroup, decision, needs-tom]
---
## User-visible outcome

[ADR 0096](../docs/decisions/0096-compose-the-subgroup-and-workgroup-reduction-levels.md) moves from `proposed` to `accepted`, or is rejected.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision` — a parked state `tkt ready` excludes and that never satisfies a dependent; an agent that finds it in `todo` should set it back and do nothing else.

**This node releases nothing, and that is a fact about the graph rather than a hedge.** Checked against the board rather than asserted: no ticket declares a dependency on this node, and no implementation ticket for the two-level composition exists to declare one. The two subgroup implementation tickets that do exist — [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md) and [`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`](admit-subgroup-typed-values-and-collectives-into-the-kernel-ir.md) — each name the two-level composition in their **non-goals**, so neither is gated on this decision and neither may drift into it. The contrast with [`accept-adr-0094-subgroup-execution-tier`](accept-adr-0094-subgroup-execution-tier.md) is deliberate and is the lesson that node had to correct twice: it claimed to release implementation tickets that did not exist at acceptance. This node claims to release nothing, because nothing is there.

**This node carries the scopes its own acceptance sweep needs**, which `accept-adr-0094-subgroup-execution-tier` did not — it declared `scopes: []`, so the sweep it describes had to be executed under a different ticket's scopes, a wart [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md)'s traceability section records by name. The sweep touches `docs/decisions/[0-9]*.md` (`contracts/decisions`), both catalog READMEs (`contracts/navigation`), and the source record's frontmatter (`research/scheduling`), which is exactly what is declared above.

## The genuine choices, stated so Tom can act without re-deriving them

The eight decision items are not equally consequential. Five follow from primary specifications or from the implemented vocabulary and left no surviving alternative; three encode priorities and are the real questions.

- **Item 2 — the composition is a sixth `ReductionTopology` variant rather than an optional field on the cooperative one.** This is the item with the largest surface cost: a new public variant, a new topology tag `0x36`, and a second verifier path whose staging coverage rule is the *inverse* of the existing one. The derivation is that one variant carrying both rules keyed by an option produces obligations the verifier cannot tell apart — the shape ADR 0094's last alternative already refuses. The alternative is not "make it a field"; it is accepting a verifier that cannot decide which bijection it is checking.
- **Item 4 — a participant's local coordinate gains a second component.** `LocalCoordinateSource` has exactly one variant today and is deliberately *not* `#[non_exhaustive]`, because "the identity encoder maps this totally". Widening it is therefore an identity-domain change at a type that was closed on purpose, and it is the item most likely to cost more than it looks. The derivation is that a one-dimensional coordinate cannot name the structure the composition combines over, and that neither of the two repairs the producing ticket proposed survives.
- **Item 5 — the contributor-block coordinate is a stated field, not a lowering convention.** This is the sharpest correctness result in the record and it has no analogue at either tier below: the same schedule text, at identical instruction count, consumes reassociation alone or an implementation-defined permutation depending on a fact two vendor specifications decline to fix. Making it a field is what stops that difference from being silent. The cost is a planning-visible coordinate that a reader will initially mistake for a lowering detail.

## Before deciding, read this correction

**One ground the ADR's transferred span rests on went stale between derivation and landing, and the ADR records it rather than editing the span.** Decision item 8 refuses a second staging round on the ground that the cooperative profile "does not model" a per-round lifetime. That was true at the record's base `2aa0824` and is false at the ADR's base `2119b20`: [`admit-loop-carried-cooperative-staging`](admit-loop-carried-cooperative-staging.md) landed at `e4d2aa7`, `CooperativeTile` now carries a `rounds` field, and the module states that "rewriting one slot across rounds is *not* what blocks it". **What item 8 decides survives — this composition is derived for one round — but its stated reason does not**, and whether a multi-round composition is now derivable is an open question the ADR records rather than a settled exclusion. Accepting item 8 accepts the scope restriction, not the obsolete justification.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted` on ADR 0096, sets `adopted_by: ["ADR-0096"]` on [the two-level subgroup-then-workgroup reduction](../docs/research/scheduling/two-level-subgroup-workgroup-reduction.md) and moves its `disposition` from `pending` to `adopted`, and updates both catalog views — the ADR's theme and chronology rows in [the decisions index](../docs/decisions/README.md) and the research record's row in [the research catalog](../docs/research/README.md), whose `informs` list gains the ADR the way every adopted record's does.

It registers nothing else. It declares no target profile row, admits no `ReductionTopology` variant, no `LocalCoordinateSource` component, and no `MemoryScope::Subgroup`, and it accepts **none of the seven public-boundary items** the research record enumerates — each arrives at Tom individually under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) with the implementation ticket that reaches it. Filing that implementation work is a separate act from this decision, and this node does not perform it.

## Rollback, kept cheap on purpose

If the acceptance relayed to whoever executes it turns out to be wrong, the repair is one field and two catalog rows: `decision_status` back to `proposed`, the two ADR catalog rows back to `proposed`, the research record's `disposition` back to `pending` with `adopted_by` removed, and this node back to `awaiting-decision`. Nothing is released on it, so nothing else has to be unwound.

## Closes when

Tom accepts or rejects it.
