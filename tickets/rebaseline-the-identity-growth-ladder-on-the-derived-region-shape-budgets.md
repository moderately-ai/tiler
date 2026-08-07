---
id: rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets
title: Rebaseline the identity-growth ladder on the derived region-shape budgets
status: done
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

## Outcome — delivered 2026-08-07 at `a0a9eaeb`

**The harness fired first, as designed.** Run unmodified at `cee4fe1a` it exited **1** with `THE WALL MOVED` twice — 33 operations compiling to a 117,213-byte identity and 62 to 219,583, both where `BudgetExhausted` was required. Everything below rests on that, rather than on a report that the wall had moved.

**Ladder 2..=32 → 2..=62**, so for the first time it is exactly the domain `semantic_operations` names. `WALLS` reduces to the single pre-planning refusal at 63. Sweep exits 0; all four perturbation modes exit 1 with their stated refusal text.

### The finding that was not in the brief, and it is the important one

**The fit's *form* held over the doubled domain; its *coefficients* did not.** Thirty new consecutive points landed on one line with second difference exactly zero — and `3525n + 727` no longer reproduces a single point. Every shared point is larger by exactly **`5n − 4`**, `graph_bytes` is identical at every shared point, and the `program_bytes` and `coverage_bytes` deltas are equal, so the whole move lives in the coverage section and subtracting `5n − 4` recovers the old ladder exactly.

**So the extrapolation now has no out-of-domain confirmation and no way to obtain one.** The thirty new points were *not* predictions of the fitted line, because the line moved under them. That is a sharper answer than the brief asked for — it asked whether the fit still holds over the wider range, and the answer is that the question cannot be asked of this pair of regimes at all. Stated explicitly in the record rather than left for a reader to notice.

The `5n − 4` is attributed to the `tiler.index-region.v11` step's per-assessment fact-source tag — five per multiply and one for the hoisted constant — and **labelled `Inference` rather than `Measurement`**, because it was not bisected. The worker judged the encoder diff plus the exact arithmetic sufficient and said so, rather than dressing it as measured. That is the right call and the right label.

### Two corrections the rerun forced, both pre-existing

- **The README's quadratic confirmation was false.** It claimed `134·9² + 3650·9 + 719` reproduced a compiled nine-operation point; the retained 2026-08-05 file records 9 as a **confirmed wall** and fits `…+ 710`, not 719. **The quadratic encoding never had an out-of-domain confirmation either.** Unrelated to the budget change, in the worker's own file, and corrected in place rather than left.
- **`--perturb=program` had silently lost its planning-phase arm**, since no point in this family refuses after planning any more. It now selects the first wall and still proves a refused compilation aborts the sweep — but not at the planning phase. The worker **declined to invent an unplannable program** to restore it, on the ground that a written-down standing claim is exactly what the current design removed. Filed as [`restore-a-planning-phase-refusal-to-the-identity-growth-harness`](restore-a-planning-phase-refusal-to-the-identity-growth-harness.md).

### The judgement on retained results, and it is well argued

**Stale, and deliberately not regenerated.** Regenerating means checking out the tree each names and rebuilding, which would not restore the file but produce a fifth regime's numbers under an older path name — destroying the only surviving record of the 2026-08-05 and 2026-08-06 encodings. The reconciliations (`graph_bytes` identical, `program_bytes` differing by exactly `5n − 4`) are stronger evidence the old files were read correctly than a regeneration would be. A new `results/README.md` indexes all five regimes with the bound that ended each ladder, so the superseded state stays legible rather than merely retained.

### Derived figures re-derived, including one that could have flipped

51 operations **stopped being an extrapolation** — measured at 180,753 B — though that measures this multiply-chain family rather than the decoder layer, so the coefficient caveat is untouched. Margin ×372 → ×371. **The 148/149 embedding crossing did not move**, which is the one conclusion the new coefficients could plausibly have flipped; it was checked rather than assumed.

**Delta rule confirmed by the coordinator against the merge's own file list**: six files under `spikes/` and `tickets/` only, none under `crates/`, `prototypes/`, or the build-configuration set, so it carries the latest green gate with `tkt lint` rerun.
