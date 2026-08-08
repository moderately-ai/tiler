---
id: re-date-the-six-identity-growth-fit-sites-one-displacement-behind
title: Re-date the six identity growth fit sites one displacement behind
status: in-progress
priority: p2
dependencies: []
related: [correct-the-region-shape-budget-sites-outside-the-corrections-scopes, repair-the-records-the-sourced-semantic-shape-falsifies]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation, measurement]
claimed_from: todo
assignee: coord
lease_expires_at: 1786178177
---

Six sites state `3530n + 723` as the **live** identity-growth fit. It was displaced by exactly `n + 1` on 2026-08-08 and is now `3531n + 724`. Every one of the six was written by a 2026-08-07 correction sweep, so this is a correction that has itself gone one displacement stale.

## Facts, coordinator-verified at `90a00528`

**Fact.** `spikes/program-planning/identity-growth/README.md` records `program_bytes(n) = 3531n + 724` with **residual 0 at all sixty-one points**, and states the displacement chain in full: `3525n + 727` displaced by `5n − 4` under an index-refinement encoding step, giving `3530n + 723`; that displaced by `n + 1` under the `tiler.semantic-graph.v2 → v3` extent tagging, giving the current form.

**Fact.** `grep -rn "3530n + 723" docs/` returns **10 occurrences across 5 files** at this base. Not all are live claims — some sit inside dated corrections, where the superseded figure is quoted deliberately and **must stay**. Separating those is the work.

**Reported by the worker that found this, not coordinator-verified:** the live sites are in `docs/artifact-abi.md` (1), `docs/ir.md` (1), `docs/research/artifacts/manifest-fixed-content-growth.md` (3), and `docs/research/program-planning/complete-model-ingestion-and-execution.md` (1). ADR 0104 already carries a 2026-08-08 supersession and needs nothing. **No open ticket owned these** — every ticket mentioning `3530n` was `done`.

## What closes this

Each **live** claim re-dated to the current fit, with the displacement and its cause named, as the spike's own README does. Quoted-in-correction occurrences left exactly as they are.

**Do not compute the new value.** Take it from the retained run at `spikes/program-planning/identity-growth/results/`, which carries the measurement, its host, and its base. The sibling worker took it from there rather than by arithmetic precisely because the ticket it worked from had done the arithmetic and got a stale answer.

**This is the trap, stated plainly:** the previous sweep replaced a stale figure with what was then current, and it went stale again five days later. Re-dating is not a fix for that — **a fit stated as a live value in six documents will decay again on the next displacement.** Consider whether these sites should name the spike and its retained run instead of restating the coefficients, so the next displacement moves one file. If you conclude restating is right, say why.

**Scope.** Only `contracts/artifacts` is declared here. `docs/ir.md` is `contracts/foundation`, and the two research files are `research/artifacts` and `research/program-planning`. **Report those with a count; do not reach into them** — add the scopes to this ticket and explain, or file siblings.

Cite by searchable anchor, not line number, and **run the anchor's grep before committing to it**. A related caution from the same worker: `docs/status.md` spells a crossing as "between 50 and 51 operations", so an anchor written `50/51` from rendered reading fails as absence.
