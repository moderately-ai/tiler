---
id: restore-a-planning-phase-refusal-to-the-identity-growth-harness
title: Restore a planning-phase refusal to the identity-growth harness
status: todo
priority: p3
dependencies: []
related: [rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets, derive-the-region-shape-budgets-from-the-declaration]
scopes: [research/program-planning]
shared_scopes: []
paths: []
tags: [research, program-planning, evidence]
---
## User-visible outcome

`spikes/program-planning/identity-growth`'s `--perturb=program` mode watches a compilation that **verified, planned, and reached no kernel program** abort the sweep, rather than only a compilation the request-verification gate refused before any target compiles.

## Why this exists

**Fact, 2026-08-07.** The mode takes its program from `WALLS` rather than writing one down, deliberately: its predecessor was a hand-written reverse-axis `tiler::reindex-f32@1` whose justification expired silently when all six `ReindexFormKind` arms became recognized, at which point the perturbation stopped perturbing while its exit code stayed 1. Reading the point out of the wall table means the same run also probes it, so the mode cannot silently stop testing what it says it tests.

`unplannable_program` therefore selected the first wall with `reaches_planning: true`. After [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) and [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md), **the table holds exactly one wall and it refuses before planning**: 63 operations on `semantic_operations = 62`, raised at request verification with no target-qualified trace. The selection was changed to `WALLS.first()` so the mode still runs, and the coverage it lost is recorded in the harness's own doc comment and in the spike README's boundary section rather than left to be discovered.

Measured on base `cee4fe1a`:

```text
$ cd spikes/program-planning/identity-growth && cargo run --release -- --perturb=program
REFUSED at operations=2: the compilation batch refused: CompileFailure { class: BudgetExhausted, explain: "absent (refused before a target-qualified trace)" }
```

`explain: "absent"` is the evidence: the abort is the request-verification one.

## What is still covered and what is not

Covered: a refused compilation stops the sweep instead of leaving a gap in the ladder. That arm is the same code path in `main` either way.

Not covered: the later abort. A compilation that verifies the request, enters the target loop, seals a per-target trace, and still reaches no verified kernel program exercises strictly more of `compile_once` — `into_targets`, `into_parts`, and the `outcome.map_err` arm that reads a target-slot refusal's class and trace. Nothing in the harness exercises those failure arms now.

## What this ticket owes

- A point this build refuses **after** planning, for the governed profile and a program the wall table can also probe, so the mode keeps its no-standing-claim property. Whether one exists is the first question: every region-shape bound is now a derivation over the declaration, and the search bounds cost alternatives rather than plans.
- If one exists: a `WALLS` entry with `reaches_planning: true`, `unplannable_program` reselecting on the phase, and a rerun.
- If none exists: say so with the enumeration behind it, and decide explicitly whether the mode should keep a written-down program with a stated expiry condition or whether the phase coverage is genuinely unavailable under the governed profile. A written-down program is what the current design rejects, so choosing it needs a reason and a trigger.

## Explicit non-goals

Not moving any budget to manufacture a wall. Not weakening the rule that the perturbed program is cross-checked by the same run.

## Closes when

Either the harness watches a planning-phase abort again with the wall table cross-checking it, or the record states which programs were enumerated, that none refuses after planning under the governed profile, and what would change that.
