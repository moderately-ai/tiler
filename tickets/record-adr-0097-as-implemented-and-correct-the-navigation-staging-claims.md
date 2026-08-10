---
id: record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims
title: Record ADR 0097 as implemented and correct the navigation docs' staging-relation claims
status: done
priority: p2
dependencies: []
related: [implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5, admit-a-two-dimensional-cooperative-staging-relation, test-the-cooperative-lowering-shape-refusal]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, scheduling, ir, identity]
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

## Outcome, 2026-08-05, read at `92a8a64e`

`implementation_status` moved `not-started` to `implemented`, on the ground that all seven decisions compile and verify and decision 7's identity step is executed at `a395852a`. The Implementation boundary section was rewritten rather than patched, naming each claim it retired instead of deleting them, and it keeps three maturity claims apart: implemented-and-verified for the seven constructs, tested-guarantee for the five named rank-two tests, and implemented-without-a-test for the rank-two *lowering* refusal. The record's measurement-boundary and work-record paragraphs also carried stale status language ("the decision Tom has not yet made", "does not start until that acceptance lands") that the status paragraph had contradicted since 2026-08-02, and both are corrected in place with the date.

**`docs/decisions/README.md` needed no edit, and the check is one line.** `grep -n 'implement\|not-started\|partial\|spike-only' docs/decisions/README.md` returns nothing: neither catalog view encodes `implementation_status` for any ADR, so 0097's two rows say `accepted` — its unchanged `decision_status` — and already agree. Adding a status word to one row of a uniform hand-maintained view would have made the view inconsistent rather than more accurate.

**One gap was found and filed rather than absorbed.** `grep -rn 'CooperativeLoweringShape' crates/` returns the variant, its diagnostic string, and its single binding in `cooperative_plan`, and no test in the workspace — so the refusal that keeps a rank-two tile out of the emitted body has never been watched refusing. [`test-the-cooperative-lowering-shape-refusal`](test-the-cooperative-lowering-shape-refusal.md) owns it at `todo`; it is out of this ticket's scopes, which reach no `crates/` path.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Outcome 2026-08-05's close conditions on ADR 0097 `implementation_status`, the rewritten Implementation boundary maturity split, the decisions catalog (no `implementation_status` vocabulary), and the navigation docs' retirement of the one-dimensional `StagedSpan` / step-ownership claims still hold. After that Outcome, live board-status labels in the navigation deliverables this ticket corrected drifted: [`docs/status.md`](../docs/status.md) still says the second tile relation and [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) are "both `deferred`", and [`docs/roadmap.md`](../docs/roadmap.md)'s contraction support-matrix row still says the second tile relation is "still `deferred` under" [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md). At re-read, that ticket is `awaiting-decision` and the realize ticket remains `deferred` — so one half of each pair is false. ADR 0097's Implementation boundary also claims the four-name existence grep "returns six files"; at re-read the same pattern matches seven files under `crates/`. Those three prose sites are residual navigation/ADR maintenance outside this wave's ticket-only edit set; this ticket stays `done` on the staging-relation / domain / catalog close conditions. `related` now includes the gap ticket for graph symmetry with its reverse edge.
