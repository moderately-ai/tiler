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

**Fact, 2026-08-07.** [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) replaced `DeterministicBudgets::governed`'s three region-shape constants — `region_members` 32, `region_boundary_outputs` 8, `region_live_values` 64 — with 62, 3, and 80, sized at authoring time against `semantic_operations`, the declared output count, and `semantic_values`. `DeterministicBudgets::governed` remains a nullary `const fn` returning integer literals (`crates/tiler-compiler/src/request.rs:1046-1063`); nothing is computed from a request's declaration at run time. [`rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets`](rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets.md) then re-ran the ladder over the widened domain and measured `program_bytes(n) = 3530n + 723` over sixty-one points, 2..=62; `3525n + 727` reproduces no point. **Dated beside, 2026-08-08: that fit was true when this ticket was written and expired the next day — the ladder now measures `3531n + 724`.** See Fact 3 of the audit below; the domain and the `3525n + 727` verdict are unaffected.

The corrections ticket enumerated its sites from `grep -rn "region_members\|region_live_values\|region_boundary_outputs" docs/ spikes/`. That pattern cannot see a record that states the numbers without naming the fields, and it cannot see a record that quotes only the fit. Two such sites survive, each in a scope that ticket did not hold.

## The two sites, each with its scope and the claim that is now false

**`research/region-search` — [`docs/research/region-search/exhaustive-region-oracle.md`](../docs/research/region-search/exhaustive-region-oracle.md) lines 143–144.** The *First heuristic bounds* list states "maximum 32 semantic occurrences per candidate" and "maximum 8 boundary outputs and 64 live boundary/internal values" — all three superseded values, spelled out without the field names. The list is framed as a proposal ("the initial production search should be bounded"), and the paragraph below it already carries a 2026-08-04 correction distinguishing a bound that "never became real" from one the implementation took; that correction is this record's own convention and is what a fix here should follow. Read that neighbour in full before writing: the honest note is that these three *were* taken and have since been re-sized, which is a different relationship from the frontier bound's, and conflating the two would weaken the correction that already exists.

**`contracts/navigation` — [`docs/status.md`](../docs/status.md) line 30.** "turned kernel-program identity from `134n² + 3650n + 727` bytes into a measured `3525n + 727`". The fit moved to `3530n + 723`; every value is larger by exactly `5n − 4` under an index-refinement encoding step that landed between the two trees, and `(3530n + 723) − (5n − 4) = 3525n + 727` recovers the older ladder by subtraction. **Dated beside, 2026-08-08: a *second* displacement of exactly `n + 1` has since carried the fit to `3531n + 724`, so the correction states that and not `3530n + 723`.** The same sentence's crossing claim — 50/51 to 148/149 — was re-solved on the measured constants and **did not move**, so it is correct as written and must not be swept along with the coefficient.

## What this ticket owes

Each site corrected against source rather than against this ticket's summary, following its own file's correction convention rather than importing one. `spikes/program-planning/identity-growth/README.md` is the measurement authority for the fit and its retained result; `crates/tiler-compiler/src/request.rs` is the authority for the budgets and must be read in full rather than in excerpt, which is how the errors this family of tickets exists to fix were introduced.

## Explicit non-goals

Not moving any budget. Not re-running the ladder. Not editing the six records [`correct-the-records-the-derived-region-shape-budgets-falsify`](correct-the-records-the-derived-region-shape-budgets-falsify.md) corrected on 2026-08-07.

## Per-Fact audit at base `fad00b73`, before any edit

Every claim above re-read at this base rather than carried. **The ticket was written on 2026-08-07 and its fit figure expired the next day**, which is the one repair that changes what a site should say.

| # | Claim | Verdict |
| --- | --- | --- |
| 1 | `derive-the-region-shape-budgets-from-the-declaration` replaced `region_members` 32, `region_boundary_outputs` 8, `region_live_values` 64 with 62, 3, 80 | **verified.** `git show 4eb78100^:crates/tiler-compiler/src/request.rs` reads `32`, `8`, `64`; `pub(crate) const fn governed` at this base reads `62`, `3`, `80`. |
| 2 | `governed` remains a nullary `const fn` returning integer literals (`request.rs:1046-1063`); nothing computed from a request's declaration at run time | **verified**, and the ordinal is exact at this base. Anchored on `pub(crate) const fn governed` instead, since the ordinal is what rots. |
| 3 | The ladder measured `program_bytes(n) = 3530n + 723` over sixty-one points, 2..=62 | **false at this base, true when written.** [`repair-the-records-the-sourced-semantic-shape-falsifies`](repair-the-records-the-sourced-semantic-shape-falsifies.md) re-ran the ladder on 2026-08-08 at base `cc667626` — an ancestor of this base — and measured **`3531n + 724`**, retained at `results/2026-08-08-post-sourced-semantic-shape-…/growth.tsv`. `3530n + 723` describes bases `cee4fe1a` and `25e76d5d`. **Working this ticket as written would have replaced one stale fit with another.** The site correction below states `3531n + 724`, regenerated from the spike's own retained run rather than derived arithmetically here. |
| 4 | `3525n + 727` reproduces no point | **verified**, and now by two displacements (`5n − 4` then `n + 1`) rather than one. |
| 5 | The corrections ticket's grep cannot see a record stating the numbers without naming the fields, and two such sites survive outside its five scopes | **verified.** That ticket's own repaired Fact 3 names both, and a fresh sweep of `docs/` and `spikes/` found no third. |
| 6 | `exhaustive-region-oracle.md` lines 143–144 spell all three superseded values; the paragraph below carries a 2026-08-04 correction distinguishing a bound that never became real | **verified**, both ordinals exact. Anchors `maximum 32 semantic occurrences per candidate` and `the frontier bound above never became real` each return exactly one file. |
| 7 | `docs/status.md` line 30 quotes `3525n + 727`; the same sentence's 50/51 → 148/149 crossing did not move and must not be swept along | **verified on both halves**, and the crossing survives the *second* displacement too: `2 × (3531·148 + 724) = 1,046,624` and `2 × (3531·149 + 724) = 1,053,686` still bracket the 1,048,576-byte ceiling. The "50/51" and "148/149" spellings are this ticket's shorthand and are not searchable in `docs/status.md`; the source reads "between 50 and 51 operations to between 148 and 149". |

**Unenumerated, and outside every scope this ticket holds.** The 2026-08-07 corrections that ticket wrote each state `3530n + 723` as the live fit, and all of them are now one displacement behind: `docs/artifact-abi.md` (`contracts/artifacts`, 1), `docs/ir.md` (`contracts/foundation`, 1), `docs/research/artifacts/manifest-fixed-content-growth.md` (`research/artifacts`, 3), and `docs/research/program-planning/complete-model-ingestion-and-execution.md` `:162` (`research/program-planning`, 1) — **six sites in four files across four scopes**. ADR 0104 already carries the 2026-08-08 supersession, and the `research/program-planning` site is one the sourced-shape repair did not enumerate. Not touched here; a narrow follow-up owns them.

## Outcome — 2026-08-08 at base `fad00b73`

Both sites corrected, each by the ever-true test [ADR 0106](../docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md)'s Context correction states: a claim true when written is **dated beside**, a claim never true at any commit is **substituted** with the retired wording quoted.

| Site | Repair | Ever-true verdict |
| --- | --- | --- |
| `docs/research/region-search/exhaustive-region-oracle.md`, the *First heuristic bounds* list | Four dated blocks after the existing 2026-08-04 correction: the three bounds *did* become real and have been re-sized (one narrower), the derivation is authoring-side rather than run-time, the list's other two budget numbers are unmoved, and the `region_members = 32` refusal is a bounded measurement over one family. Both list lines retained verbatim. | `region_members: 32`, `region_boundary_outputs: 8`, `region_live_values: 64` were the literals from `bc371d6d` to `4eb78100` → **dated beside**, and explicitly *not* the frontier bound's substitution |
| `docs/status.md`, the `Fact — 2026-08-06: the governed content digest is its own crate` block | Dated correction carrying both displacements to `3531n + 724`, holding both crossings unchanged with their arithmetic, and bounding the fit to its family, contract, profile, and tree. `3525n + 727` retained in place. | measured on 2026-08-06 → **dated beside**; stated inline, because the two corrections earlier in that bullet both substituted and a later reader would otherwise read the difference as an oversight |

**The retained wording stays greppable, and both notes say so where it matters.** A grep hit on "maximum 32 semantic occurrences per candidate" now finds a retired proposal rather than a live bound, which the region-search note states inline so absence-by-grep is not read as presence-of-claim.

**No `crates/` or `prototypes/` file is touched**; `crates/tiler-compiler/src/request.rs` was read to describe it. The delta is `docs/` and `tickets/` only, so it touches no gated path and the latest green gate carries; `tkt lint` and `./check-citations.sh` were rerun regardless.
