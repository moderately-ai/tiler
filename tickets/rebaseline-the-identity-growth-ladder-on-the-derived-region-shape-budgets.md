---
id: rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets
title: Rebaseline the identity-growth ladder on the derived region-shape budgets
status: todo
priority: p2
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration, widen-the-identity-growth-ladder-to-the-governed-operation-budget, measure-executable-coverage-identity-growth-against-the-program-identity-bound]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, identity, measurement]
---
## User-visible outcome

[`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md) runs to completion again, and its ladder covers the domain the compilation path now admits: the `region_members` wall at thirty-three is gone, so the family's reachable domain is 2..=62 and the first refusal is `semantic_operations` at sixty-three.

## Why this exists — the harness is designed to fire, and it will

**Fact, 2026-08-07.** [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) made `DeterministicBudgets::governed`'s `region_members` a derivation from `semantic_operations` (62 rather than the constant 32), `region_live_values` a derivation from `semantic_values` (80 rather than 64), and `region_boundary_outputs` the declared output count (3 rather than 8). The whole 33..=62 range now compiles as one whole-program region.

`crates/tiler-compiler/tests/region_search_budget_coverage.rs`'s `the_population_the_member_bound_refused_compiles_as_one_whole_program_region` is the in-repository evidence, and it measures the whole range through the public `compile_governed` boundary.

The spike's `OPERATIONS` ladder (2..=32) and its `WALLS` table therefore both misstate the tree:

- `OPERATIONS` claims to be "every program size the ordinary compilation path admits for this program family". It is now a truncation of it.
- `WALLS`'s first entry — `{operations: 33, class: BudgetExhausted, reaches_planning: true, why: "region_members (32) is the largest region this profile forms …"}` — compiles instead of refusing, which is the harness's designed non-zero exit.
- `WALLS`'s second entry at 62 — "the governed `semantic_operations` maximum, which the region-size bound refuses long before its own budget would" — compiles too.
- `WALLS`'s third entry at 63 is unchanged and still correct.

The harness has fired for this reason twice before and both firings were the outcome it exists to produce; see this ticket's predecessor [`widen-the-identity-growth-ladder-to-the-governed-operation-budget`](widen-the-identity-growth-ladder-to-the-governed-operation-budget.md).

## Why the compiler ticket did not do it

Two reasons, both stated so a reader can refute them rather than accept a deferral.

1. **Scope.** `spikes/program-planning/**` is `research/program-planning`; the budget change was `implementation/compiler`. Widening the ladder is not an edit but a *re-measurement* — a new retained `results/<date>-<host>/growth.tsv`, a refitted curve, and a re-derived verdict — which is a research deliverable rather than a compiler one.
2. **The fit changes materially and the record's conclusions rest on it.** The domain roughly doubles from thirty-one points to sixty-one. ADR 0104's confirmed figures (`3525n + 727`, the 19,038-operation refusal point, the 148/149 embedding crossing, 219,277 bytes at 62) were all extrapolations past a wall at thirty-two; sixty-two is now *inside* the domain, so the 219,277-byte figure stops being an extrapolation and becomes a measurement — or is falsified. Nobody should guess which.

**Feasibility is known and is not the constraint.** Measured on the branch that moved the budgets: the chain family compiles at 2ms (n=2) through 73ms (n=62) through `compile_governed`, and the whole eighteen-point probe took 0.6s. A contiguous 2..=62 ladder is affordable.

## What this ticket owes

- `OPERATIONS` widened to the measured reachable domain with its doc comment restating the derivation, not a new constant.
- `WALLS` reduced to the entries that still refuse, each compiled and required to refuse with its class and phase. The 63-operation entry's `why` is the only one that survives unchanged.
- A re-run and a newly retained result beside the existing ones, which are evidence at their own commits and are not overwritten.
- The README's ladder table, wall table, verdict, and boundary section re-derived, with `last_verified` moved.
- All four `--perturb` modes watched failing, as its predecessor did.

## Explicit non-goals

Not moving any budget. Not correcting the records outside `spikes/program-planning/` and `docs/research/program-planning/` — [`correct-the-records-the-derived-region-shape-budgets-falsify`](correct-the-records-the-derived-region-shape-budgets-falsify.md) owns those and depends on this run's numbers.

## Closes when

The harness exits 0 over the widened domain, a new result is retained beside the old ones, and the README states which figures reproduced, which moved, and how far the ladder reached with the reason it stopped there.
