---
id: sweep-the-deferred-tickets-whose-reconsideration-triggers-have-fired
title: Sweep the deferred tickets whose reconsideration triggers have fired
status: done
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

## Sweep outcome — 2026-08-04

**Population, counted before reporting on it and by two independent means.** **Fifty-three**, not the twenty-six this ticket's premise recorded at `0017345`. `tkt list --status deferred --format json | python3 -c "import json,sys;print(len(json.load(sys.stdin)['tickets']))"` prints `53`, and `grep -c "^status: deferred" tickets/*.md | grep -v ":0" | wc -l` prints `53` independently. Twelve of the growth are the dtype-family research tracks filed 2026-08-04 by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md), whose triggers were checked at filing and are re-checked here against the tree rather than re-trusted from the record. Every one of the fifty-three was read in full and carries a dated verdict in a `## Trigger check log` section of its own body.

**Verdicts.** One fired, one fired partially, one refuted its own named route, fifty not fired. None was unevaluable — every trigger reduced to a state of the corpus a reader can check.

- **Fired — [`clarify-the-inline-frontend-facades-consumer-scope`](clarify-the-inline-frontend-facades-consumer-scope.md) (p1), reactivated to `todo`.** Its trigger is "Tom lifting that freeze", and no code freeze is recorded in any durable contract: the only two statements of one are that ticket and its `done` dependency, both entered in commit `52e088a2` at 14:55 on 2026-08-04, and in the five hours after it four merged descendant commits added 697 lines of Rust doc comment to `crates/`. The sweep asserts that no freeze constrains the edit, not that an acceptance was relayed; re-parking is one status change and nothing was released on it.
- **Fired, partially — [`audit-dead-code-admissions-after-public-boundary-promotions`](audit-dead-code-admissions-after-public-boundary-promotions.md), reactivated to `todo`.** Its own trigger admits a partial sweep when a subsystem's promotions complete; `tiler-ir`'s and `tiler-metal-aot`'s both did, neither partial sweep was run, and the ticket's 2026-07-28 inventory is stale in both directions (twelve file-scope admissions became eight, with six gone and two new).
- **Fired half, deliberately not reactivated — [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md) (p1).** Its dependency landed; the other conjunct is Tom's acceptance of two public boundaries, and `OwnershipProofKind` still has one variant. Reactivating would assert an acceptance nobody relayed, so the finding is recorded instead — this is the stop condition the sweep was told to honour and it fired exactly once.
- **The most useful negative — [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md).** Its named concrete route *landed* — the Metal grid-axis row moved from `4` to a measured `268_435_456` — and the trigger still did not fire, because the tree and the split read one `governed_partition` and `workgroup_tree_tile` fixes `rounds: 1`, so widening the row moves the reachable count at every value without ever separating the two groupings. The ticket's own instruction not to infer from the grid-axis ticket is what caught it.

**Two other triggers are now closed rather than merely unmet**, which is worth more than either verdict alone because it removes them from every future sweep: `configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction`'s least-recently-used route, refuted by `define-supported-expansion-cache-filesystems`' answer that no supported filesystem maintains access time usefully enough to order a working set; and `resolve-the-generated-facade-path-under-crate-renaming`'s compilation-key route, refuted because the key work landed and decided the subject's complete contents without ever needing the resolved-name question.

## Proving the sweep could say no

The extraction is checked from both ends. A uniform verdict would have been the signature to distrust, and this population is not uniform: fifty not fired, one fired on refuted-premise grounds, one fired partially, one whose *dependency* reaching `done` was mistaken for its trigger by the brief that dispatched the sweep and turned out not to be it, and two whose routes closed. Three separate expectations carried into the sweep were refuted by reading — the grid-axis widening, the two-level reduction as the subgroup trigger's firing construct, and the compilation key as the facade-path trigger's route — which is the evidence that the extraction was reading triggers rather than pattern-matching landings. Where a verdict rests on a tree fact rather than a ticket status, the log line carries the one-line command that reproduces it.

## The mechanism: what makes the next sweep cheaper

This sweep cost fifty-three full-body reads, because every trigger was stated somewhere different in a body averaging four and a half thousand bytes, and because deciding whether one had fired meant re-deriving from source what a previous reader had already derived. Two conventions remove both costs, and **both are installed by this ticket rather than proposed by it**.

1. **The `## Trigger check log` section, now present on every one of the fifty-three tickets this sweep read — the fifty-one still `deferred` and the two it reactivated.** It is the last section of the body, and each entry is one dated line of the form `- YYYY-MM-DD — **fired|not fired|unevaluable.** <evidence> Recheck: <one command>`. The next sweep reads the *last line* of that section per ticket and re-runs the command it names; it opens the full body only when the recheck disagrees or the section is missing. That converts fifty-three long reads into fifty-three one-line checks, and a missing section is a visible defect rather than a silent gap — the population is counted, so a ticket with no log is a ticket the sweep must read, not a ticket it may skip.
2. **The edge idiom: a deferral whose trigger names another ticket declares that ticket in `dependencies:` (when it genuinely cannot start without it) or `related:` (when the other merely fires it).** Eight deferrals named a firing ticket in prose and carried no edge; all eight now carry one. With the idiom held, "which deferrals did today's landings fire?" is a graph query over `tkt list --status deferred --format json` rather than a reading exercise — and the query's answer is a shortlist to check, never a verdict, because [`admit-a-round-dependent-cooperative-staging-span`](admit-a-round-dependent-cooperative-staging-span.md) is this sweep's counterexample: its named ticket is `done` and its trigger — that ticket's tree reaching a *depth limit* — is not.

**Cadence.** Run the sweep when a wave lands any ticket that a deferral names through an edge; with the idiom above that is one query at integration rather than a scheduled review. A calendar cadence was eliminated: the deferred population changes only when work lands, so a timer either fires over an unchanged board or misses the landing that mattered.

**AGENTS.md.** The two conventions belong beside "Maintain the graph as evidence arrives", whose paragraph already governs deferral filing. `AGENTS.md` is outside this ticket's scopes, so the exact sentences are drafted in the worker's report for the integrator to land verbatim rather than paraphrased here.

## Coordination

- Q-PLAN-011's `docs/open-questions.md` edit was made by [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) and is not duplicated here; `docs/` is outside this ticket's scopes in any case.
- The named starter [`add-subgroup-memory-scope-when-collectives-land`](add-subgroup-memory-scope-when-collectives-land.md) is decided: its trigger is rewritten to the construct the two-level reduction record derived, and **no edge to the two-level reduction ticket is added**, because that addendum's proposal was refuted by the next one and an edge would encode a disproved premise as graph structure. The ground is recorded on that ticket.
- One stale source citation was found and could not be fixed in scope: `resolve-the-generated-facade-path-under-crate-renaming` names `FACADE_ANCHOR_PATH` in `crates/tiler-macros/src/lib.rs`, which no longer exists — the anchor is four constants now. Recorded on that ticket's log for whoever holds `implementation/frontend`.
