---
id: sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired
title: Sweep the deferred tickets whose reconsideration triggers have fired
status: todo
priority: p2
dependencies: []
related: [make-adr-acceptance-visible-to-the-work-graph, re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal, add-subgroup-memory-scope-when-collectives-land]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [graph-repair, planning, process]
---
## User-visible outcome

A deferred ticket whose reconsideration trigger has fired is re-activated by a sweep that runs, rather than by whoever happens to reread it — so the parked half of the board stops being a place work goes to be forgotten.

## Why this exists

**Fact — twenty-six tickets are `deferred` and no node owns re-activating one.** Count at base `0017345` with `grep -c "^status: deferred" tickets/*.md | grep -v ":0" | wc -l`, or `tkt list --status deferred`. Most carry an explicit trigger in prose. Nothing reads those triggers on a schedule, and `deferred` never satisfies a dependent, so a fired trigger leaves both the ticket and anything parked behind it stationary.

**Fact — the nearest existing node built a different mechanism.** [`make-adr-acceptance-visible-to-the-work-graph`](make-adr-acceptance-visible-to-the-work-graph.md) established the acceptance convention that `ticketsplease.toml` now documents: a ticket conditional on an ADR depends on that ADR's `accept-*` node, which sits in a parked state so the block is structural and `rollup` names the undecided record as its cause. That solves *decision* gating. It does not read a prose trigger, and a reconsideration trigger is not an ADR acceptance.

**Fact — the counterexample already happened, twice, and one is closed by this wave.** Both trigger clauses on [`decide-whether-to-admit-a-distributivity-permission`](decide-whether-to-admit-a-distributivity-permission.md) had fired — `admit-the-contraction-semantic-profile` registered `tiler::strict-tensor-contraction-f32@1` (`docs/roadmap.md:421`) and `admit-a-reassociating-contract-without-contraction` landed, both `done` — while the ticket stayed `deferred` with [`decide-whether-distributivity-directions-share-one-permission`](decide-whether-distributivity-directions-share-one-permission.md) parked behind it. Tom decided it on 2026-08-01 (declined, with a reopening trigger); it is recorded on that ticket and is this ticket's worked example rather than its remaining work.

## Named starters

- [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md). Its 2026-08-01 addendum derives that a shuffle does **not** fire the trigger and that the firing construct is [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) — a staged handoff between simdgroups within one threadgroup. The narrowing is prose with a live link and no frontmatter edge, so nothing connects the parked ticket to the work that fires it. Reading "should be narrowed to that ticket" as a prescribed `dependencies:` or `related:` edit is interpretation, not instruction; decide it here rather than assuming it.
- **Q-PLAN-011's CPU trigger.** `docs/open-questions.md:334` states the trigger as "the CPU backend enters the active roadmap", which fired: [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md) is `done` and ADR 0093's CPU vector-lane tier is accepted with three implementation tickets filed. This one is shared with [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md), which names it among its four added questions; coordinate, do not duplicate the edit.

## Work

Read every `deferred` ticket, extract its trigger, and evaluate it against the tree. For each: re-activate with the firing evidence recorded and dated; or confirm the trigger has not fired, stating what would fire it in terms a reader can evaluate; or record that the trigger is unevaluable and give it one. Then decide what makes the *next* sweep cheaper than this one — a recorded convention, an edge idiom, or a stated re-run cadence — because a sweep that leaves no mechanism behind guarantees its own repetition.

## Prove the sweep can say no

A uniform verdict over a heterogeneous population is the signature to distrust. Name the population and count it before reporting on it, and check the extraction against a ticket whose trigger has demonstrably *not* fired — if every deferred ticket comes back "trigger fired" or every one comes back "not fired", the extraction is what failed, not the board.

## Closes when

Every `deferred` ticket has been read and its trigger evaluated with the verdict recorded; each fired trigger has produced a status change or a stated reason it did not; the population was counted rather than assumed; and the mechanism that makes the next sweep cheaper is recorded — in AGENTS.md or in a convention this ticket names — rather than left as an intention.
