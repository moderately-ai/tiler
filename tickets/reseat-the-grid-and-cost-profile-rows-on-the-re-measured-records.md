---
id: reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records
title: Reseat the grid and cost profile rows on the re-measured records
status: todo
priority: p1
dependencies: [resolve-the-retained-metal-profile-measurement-invocation-authority]
related: []
scopes: [implementation/build, research/target-profiles, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The standard Metal profile's grid-axis and saturated-cost rows cite the 2026-08-18 re-measured records (production `CompileRequest` selection, attested `26A5406e` M4 Max execution row) instead of the withdrawn-authority 2026-08-04/07 records; the authority ledger gains the third-environment tables; every affected pin is recomputed on one tree.

## Why this exists

The accepted (R, R) session completed 2026-08-18: grid record at `spikes/target-profiles/metal-grid-axis-extent/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/` (widest extent 2^28 verified, mutation proofs rerun) and cost record at `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/` (fit retained held-out separation against all four perturbations; fitted encoder 3.0569 us, parallel 1280, step 15.5962 ns), both merged at `39c10c60` with verified custody. The parent ticket's packet assigns row reseating, ledger third-environment tables, per-population sources, and pin recomputation to this carrier. Note the stage-model finding recorded in the parent: new tree cells are not like-for-like with 2026-08-07 tree cells where capped widths differ — the serial-versus-parallel verdict is the comparable quantity; the reseated cost row must state its fitted parameters from the new record only.

## Closes when

Both rows cite the new records with per-population sources; the execution-environment split is truthful (the old `26A5388g` rows' disposition follows the parent packet); ledger tables and every moved pin are recomputed on one tree; `make full` is green; and the parent ticket can close over the completed disposition.
