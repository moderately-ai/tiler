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

**Fact — false as written, repaired 2026-08-08 by the worker at base `c81f9257`.** `grep -rn "3530n + 723" docs/` returns **11 occurrences across 6 files**, not ten across five. Ten across five was true at `90a00528`, the base this ticket's Facts were verified at; `56b9ca23`, "Correct the two region-shape budget sites outside the corrections scopes", then added a sixth file. The count was restated unchanged in the dispatch brief at `24f11d4f`, where it was already false. The sixth file is `docs/status.md`, and its occurrence is **quoted history rather than a live claim** — it sits inside a 2026-08-08 correction that already names `3531n + 724` as current and states the displacement chain — so the population of live claims is unaffected. Per-file: `docs/artifact-abi.md` 1, `docs/status.md` 1, `docs/ir.md` 1, `docs/research/artifacts/manifest-fixed-content-growth.md` 3, `docs/research/program-planning/complete-model-ingestion-and-execution.md` 1, `docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` 4. Not all are live claims — some sit inside dated corrections, where the superseded figure is quoted deliberately and **must stay**. Separating those is the work.

**Verified 2026-08-08 by the worker, by reading each occurrence in place rather than by grep.** The live sites are exactly the four the finding worker reported: `docs/artifact-abi.md` (1), `docs/ir.md` (1), `docs/research/artifacts/manifest-fixed-content-growth.md` (3), and `docs/research/program-planning/complete-model-ingestion-and-execution.md` (1) — six occurrences, each a 2026-08-07 correction whose `**Measurement, re-run 2026-08-07 …**` block states the fit in the present tense with no later supersession beneath it. The five quoted occurrences are `docs/status.md` (1) and ADR 0104 (4); ADR 0104's four all sit under its `**Superseded — 2026-08-08 by …repair-the-records-the-sourced-semantic-shape-falsifies**` header note and its `**Extended 2026-08-08 by …**` closing note, so that record needs nothing, as reported. **No open ticket owned these** — every ticket mentioning `3530n` was `done`.

## What closes this

Each **live** claim re-dated to the current fit, with the displacement and its cause named, as the spike's own README does. Quoted-in-correction occurrences left exactly as they are.

**Do not compute the new value.** Take it from the retained run at `spikes/program-planning/identity-growth/results/`, which carries the measurement, its host, and its base. The sibling worker took it from there rather than by arithmetic precisely because the ticket it worked from had done the arithmetic and got a stale answer.

**This is the trap, stated plainly:** the previous sweep replaced a stale figure with what was then current, and it went stale again five days later. Re-dating is not a fix for that — **a fit stated as a live value in six documents will decay again on the next displacement.** Consider whether these sites should name the spike and its retained run instead of restating the coefficients, so the next displacement moves one file. If you conclude restating is right, say why.

**Scope.** Only `contracts/artifacts` is declared here. `docs/ir.md` is `contracts/foundation`, and the two research files are `research/artifacts` and `research/program-planning`. **Report those with a count; do not reach into them** — add the scopes to this ticket and explain, or file siblings.

Cite by searchable anchor, not line number, and **run the anchor's grep before committing to it**. A related caution from the same worker: `docs/status.md` spells a crossing as "between 50 and 51 operations", so an anchor written `50/51` from rendered reading fails as absence.

## Worker outcome, 2026-08-08

**Reference rather than restate, and the argument is this document's own.** `docs/artifact-abi.md`'s live claim now names [the identity-growth spike](../spikes/program-planning/identity-growth/README.md) and its [results index](../spikes/program-planning/identity-growth/results/README.md) as the standing authority for the coefficients and the figures solved from them, states only the conclusions that survived both measured displacements — quadratic coefficient exactly zero, the 64 MiB refusal point orders of magnitude above the governed budget of 62, the 1 MiB embedding crossing still between 148 and 149 at multiplicity two — and dates the one coefficient it still carries to base `cc667626` and its retained `growth.tsv` rather than stating it in the present tense.

Three grounds, none of them novel to this ticket. **The file already decided this**: [`replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin`](replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin.md) retired nine byte figures from two `bf16` Measurement clauses on 2026-08-08 rather than refreshing them, on the reasoning that a number stated in prose and pinned nowhere decays without any gate seeing it, so replacing digits with newer digits rebuilds the defect one identity step later — and these coefficients are exactly that class of figure. **Nothing gates them**: `make citations` resolves links and never reads a number, and no test asserts a fit, so the only mechanism that has ever caught this is a human re-reading six documents, which is what failed on 2026-08-07. **The pattern already exists in the same file**: "The lengths behind that argument, dated to the trees they were taken on rather than stated in the present tense" is how the kernel identity lengths are carried, and a figure attached to a named tree is not falsified by a later one.

The counter-argument, stated rather than skipped: a contract reader wants the magnitude without opening a spike. It is answered rather than waived — every magnitude claim this contract rests on is kept, and only the exact coefficients, which decide nothing here and move every few days, are referred out.

**Remainder.** The five live occurrences outside `contracts/artifacts` are [`re-date-the-five-identity-growth-fit-sites-outside-the-artifacts-scope`](re-date-the-five-identity-growth-fit-sites-outside-the-artifacts-scope.md), filed rather than absorbed: they need `contracts/foundation`, `research/artifacts`, and `research/program-planning`, each on a document that must be read in full and that restates its own derived figures — P1/P2/P3 byte counts, the ×2 ceiling shares, the whole-model extrapolation — which the displacement moves alongside the coefficient. That is a larger job than a re-date and taking `contracts/foundation` exclusively for it would park `docs/ir.md` against unrelated work.
