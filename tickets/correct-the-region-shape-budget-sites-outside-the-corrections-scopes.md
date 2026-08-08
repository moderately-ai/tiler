---
id: correct-the-region-shape-budget-sites-outside-the-corrections-scopes
title: Correct the two region-shape budget sites outside the corrections ticket's scopes
status: in-progress
priority: p3
dependencies: []
related: [correct-the-records-the-derived-region-shape-budgets-falsify, derive-the-region-shape-budgets-from-the-declaration]
scopes: [research/region-search, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, budgets]
claimed_from: todo
assignee: coord
lease_expires_at: 1786177034
---
## User-visible outcome

No record outside the five scopes [`correct-the-records-the-derived-region-shape-budgets-falsify`](correct-the-records-the-derived-region-shape-budgets-falsify.md) held still states a superseded region-shape budget or a superseded identity-growth fit.

## Why this exists

**Fact, 2026-08-07.** [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) replaced `DeterministicBudgets::governed`'s three region-shape constants — `region_members` 32, `region_boundary_outputs` 8, `region_live_values` 64 — with 62, 3, and 80, sized at authoring time against `semantic_operations`, the declared output count, and `semantic_values`. `DeterministicBudgets::governed` remains a nullary `const fn` returning integer literals (`crates/tiler-compiler/src/request.rs:1046-1063`); nothing is computed from a request's declaration at run time. [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md) then re-ran the ladder over the widened domain and measured `program_bytes(n) = 3530n + 723` over sixty-one points, 2..=62; `3525n + 727` reproduces no point.

The corrections ticket enumerated its sites from `grep -rn "region_members\|region_live_values\|region_boundary_outputs" docs/ spikes/`. That pattern cannot see a record that states the numbers without naming the fields, and it cannot see a record that quotes only the fit. Two such sites survive, each in a scope that ticket did not hold.

## The two sites, each with its scope and the claim that is now false

**`research/region-search` — [`docs/research/region-search/exhaustive-region-oracle.md`](../docs/research/region-search/exhaustive-region-oracle.md) lines 143–144.** The *First heuristic bounds* list states "maximum 32 semantic occurrences per candidate" and "maximum 8 boundary outputs and 64 live boundary/internal values" — all three superseded values, spelled out without the field names. The list is framed as a proposal ("the initial production search should be bounded"), and the paragraph below it already carries a 2026-08-04 correction distinguishing a bound that "never became real" from one the implementation took; that correction is this record's own convention and is what a fix here should follow. Read that neighbour in full before writing: the honest note is that these three *were* taken and have since been re-sized, which is a different relationship from the frontier bound's, and conflating the two would weaken the correction that already exists.

**`contracts/navigation` — [`docs/status.md`](../docs/status.md) line 30.** "turned kernel-program identity from `134n² + 3650n + 727` bytes into a measured `3525n + 727`". The fit moved to `3530n + 723`; every value is larger by exactly `5n − 4` under an index-refinement encoding step that landed between the two trees, and `(3530n + 723) − (5n − 4) = 3525n + 727` recovers the older ladder by subtraction. The same sentence's crossing claim — 50/51 to 148/149 — was re-solved on the measured constants and **did not move**, so it is correct as written and must not be swept along with the coefficient.

## What this ticket owes

Each site corrected against source rather than against this ticket's summary, following its own file's correction convention rather than importing one. `spikes/program-planning/identity-growth/README.md` is the measurement authority for the fit and its retained result; `crates/tiler-compiler/src/request.rs` is the authority for the budgets and must be read in full rather than in excerpt, which is how the errors this family of tickets exists to fix were introduced.

## Explicit non-goals

Not moving any budget. Not re-running the ladder. Not editing the six records [`correct-the-records-the-derived-region-shape-budgets-falsify`](correct-the-records-the-derived-region-shape-budgets-falsify.md) corrected on 2026-08-07.
