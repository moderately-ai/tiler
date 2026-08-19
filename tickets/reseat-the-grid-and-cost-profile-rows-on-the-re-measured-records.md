---
id: reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records
title: Reseat the grid and cost profile rows on the re-measured records
status: in-progress
priority: p1
dependencies: [resolve-the-retained-metal-profile-measurement-invocation-authority]
related: []
scopes: [implementation/build, research/target-profiles, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-reseat
lease_expires_at: 1787160394
---
## User-visible outcome

The standard Metal profile's grid-axis and saturated-cost rows cite the 2026-08-18 re-measured records (production `CompileRequest` selection, attested `26A5406e` M4 Max execution row) instead of the withdrawn-authority 2026-08-04/07 records; the authority ledger gains the third-environment tables; every affected pin is recomputed on one tree.

## Why this exists

The accepted (R, R) session completed 2026-08-18: grid record at `spikes/target-profiles/metal-grid-axis-extent/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/` (widest extent 2^28 verified, mutation proofs rerun) and cost record at `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/` (fit retained held-out separation against all four perturbations; fitted encoder 3.0569 us, parallel 1280, step 15.5962 ns), both merged at `39c10c60` with verified custody. The parent ticket's packet assigns row reseating, ledger third-environment tables, per-population sources, and pin recomputation to this carrier. Note the stage-model finding recorded in the parent: new tree cells are not like-for-like with 2026-08-07 tree cells where capped widths differ — the serial-versus-parallel verdict is the comparable quantity; the reseated cost row must state its fitted parameters from the new record only.

## Scope correction — 2026-08-19, most of this ticket already landed

The compilation-selection carrier ([`carry-required-compilation-selection-identity-on-compile-profile-contexts`](carry-required-compilation-selection-identity-on-compile-profile-contexts.md), merged at `320d4a0e`) could not land its per-population source partition truthfully while the grid and cost rows still cited the withdrawn `26A5388g` records — those records have no constructible expected selection under the accepted packet's §5 construction rules — so it carried the **minimal reseat** as part of that atomic change. Already landed there, and **not** remaining work here:

- both rows cite the 2026-08-18 `26A5406e` records, per-population sources carry the request-derived selection, and `MEASURED_SATURATED_FOLD_STEPS` moved 1,056 → 1,280 (verified at integration against the record's `parallel_threads 1.280001e3`);
- every pin the reseat moved is recomputed on that one tree;
- the gate-checked live-pin mirror paragraph in the authority ledger moved with the pins, because the check reads it.

Also landed at integration: dated row corrections at the ledger's grid-axis and saturated-cost sections, so the document no longer positively asserts the superseded value and record while the profile states otherwise.

**What genuinely remains for this ticket**, and the reason it stays open rather than closing as done: the ledger's third-environment tables (the profile's measured rows now span two execution environments and the document still presents one), the per-row prose rewrite of the two retained 2026-08-07/08-04 sections into their current form, the disposition of the superseded rows as the parent packet describes it, and any remaining doc mirrors of the moved values. A worker taking this ticket should re-audit against `320d4a0e` first: three of the four clauses of the original `Closes when` below are already satisfied.

## Closes when

Both rows cite the new records with per-population sources; the execution-environment split is truthful (the old `26A5388g` rows' disposition follows the parent packet); ledger tables and every moved pin are recomputed on one tree; `make full` is green; and the parent ticket can close over the completed disposition.
