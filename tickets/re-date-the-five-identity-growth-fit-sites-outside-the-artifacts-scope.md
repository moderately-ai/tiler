---
id: re-date-the-five-identity-growth-fit-sites-outside-the-artifacts-scope
title: Re-date the five identity-growth fit sites outside the artifacts scope
status: done
priority: p2
dependencies: []
related: [re-date-the-six-identity-growth-fit-sites-one-displacement-behind]
scopes: [contracts/foundation, research/artifacts, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [identity, documentation, measurement]
---
Five identity-growth fit occurrences outside `contracts/artifacts` still state `3530n + 723` as the **live** fit. It was displaced by exactly `n + 1` on 2026-08-08 and the ladder now measures `3531n + 724`. [`re-date-the-six-identity-growth-fit-sites-one-displacement-behind`](re-date-the-six-identity-growth-fit-sites-one-displacement-behind.md) held only `contracts/artifacts` and repaired the sixth, in `docs/artifact-abi.md`; these five were deliberately left rather than edited across a scope boundary.

## Facts, verified 2026-08-08 at base `c81f9257` by reading each occurrence in place

**Fact.** `grep -rn "3530n + 723" docs/` returns **11 occurrences across 6 files** at that base. **Six are live claims and five are quoted history.** One live claim — `docs/artifact-abi.md` — is repaired by the sibling above. The five below remain.

**Repaired 2026-08-08 by the worker at base `1438c867` — the Fact above counts lines and calls them occurrences, and its number has moved.** `grep -rn` reports one line per match however many times the string appears on it, and several of these lines carry it twice, so `c81f9257` held **11 lines / 15 occurrences** and this base holds **12 lines / 16 occurrences** across the same six files. The twelfth line is in `docs/artifact-abi.md`, written by this ticket's own sibling repair `775d314f` (merged `9b999a35`) after these Facts were verified. Every per-file number below is a line count on the same convention. **The live population is five either way**, so nothing this ticket exists for changes; the per-Fact audit is below.

**Fact — the five live occurrences outside `contracts/artifacts`, each a 2026-08-07 correction whose measurement block states the fit in the present tense with no later supersession beneath it.**

| File | Scope | Count | Anchor |
| --- | --- | --- | --- |
| [`docs/ir.md`](../docs/ir.md) | `contracts/foundation` | 1 | `**Measurement, re-run 2026-08-07 over the whole admitted domain — sixty-one points, 2..=62 operations:** kernel-program identity is exactly` |
| [`docs/research/artifacts/manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) | `research/artifacts` | 3 | `is now the reverse of the truth`; `the ×2 crossing is`; `the sharpest inversion this correction makes` |
| [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) | `research/program-planning` | 1 | `Superseded a third time, on 2026-08-07 by` |

**Fact — two files need nothing and their occurrences must stay.** [`docs/status.md`](../docs/status.md) (`contracts/navigation`, 1 occurrence) already carries a 2026-08-08 correction that names `3531n + 724` as current and states the displacement chain; its `3530n + 723` is quoted inside that chain. [`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`](../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) (`contracts/decisions`, 4 occurrences) carries both a `**Superseded — 2026-08-08**` header note and an `**Extended 2026-08-08**` closing note; all four occurrences sit under them. **Deleting either would destroy history, which is the failure mode in the other direction.**

**Fact — the value comes from the retained run and never from arithmetic.** `spikes/program-planning/identity-growth/results/2026-08-08-post-sourced-semantic-shape-apple-m4-max-macos27.0-26A5388g/growth.tsv`, base `cc667626`, Apple M4 Max, macOS 27.0 build `26A5388g`, repository toolchain pin: `program_bytes(n) = 3531n + 724`, residual zero at all sixty-one points, `graph_bytes(n) = 135n + 149`, widest measured point 219,646 bytes at 62 operations. [`results/README.md`](../spikes/program-planning/identity-growth/results/README.md) carries the full displacement chain and the bound on each attribution.

**Fact — the derived figures these five documents carry move with the slope and the conclusions do not.** The spike records the fitted 64 MiB refusal point moving 19,038 → 19,011 → 19,006 operations, and the 1 MiB per-invocation embedding crossing staying between **148 and 149** at multiplicity two through both displacements. Each of the five documents restates its own solved figures — P1/P2/P3 byte counts, the ×2 shares, the whole-model extrapolation — so a re-date that touches only the coefficient leaves those inconsistent with it.

## What closes this

Each of the five live claims stops being a live claim, and the quoted occurrences in `docs/status.md` and ADR 0104 are left exactly as they are.

**Apply the sibling's decision rather than re-deciding it.** `docs/artifact-abi.md` now names the spike and its retained run as the standing authority and states only the displacement-invariant conclusions, dating any unavoidable coefficient to the tree it was measured on. That decision was taken because these figures are pinned by no test, `make citations` resolves links and never checks a number, and this is the third spelling of one curve in four days — so refreshing the digits rebuilds the defect one encoding step later. The same reasoning applies to all five sites here; a document that concludes otherwise should say why in its own correction.

**Watch the derived figures, not only the coefficient.** The previous sweep's failure was partly that a re-date is cheap and re-solving every dependent figure is not. Either re-solve them from the retained run or move them to the spike with the coefficient.

Cite by searchable anchor and **run the anchor's grep before committing to it**. `docs/status.md` spells a crossing as "between 50 and 51 operations", so an anchor written `50/51` from rendered reading fails as absence — the more dangerous reading, because it looks like the text was removed.

## Per-Fact audit, by the worker at base `1438c867`

Every Fact was re-read at this base before any edit. One is imprecise in a way that would have misled a counter; the rest hold.

| Fact | Verdict | Evidence |
| --- | --- | --- |
| The `grep -rn` count | **Imprecise, and stale here** — repaired below | `grep -rn` counts *matching lines*, not occurrences, and several lines carry the figure twice. At `c81f9257` the command returns **11 lines / 15 occurrences**; at this base it returns **12 lines / 16 occurrences** across the same six files. The extra line is in `docs/artifact-abi.md`, written by the sibling's own repair `775d314f` (merged `9b999a35`) after the Facts were verified. **The live population is unchanged at five**, so nothing this ticket is for moves. |
| The five live occurrences and their anchors | **Verified** | All five anchors resolve exactly once under `grep -cF` at this base. Each sits in a 2026-08-07 correction stating the fit in the present tense with no later supersession beneath it, and all five were written by one commit, `e5bfaba4` (2026-08-07), whose tree the spike README then read `3530n + 723` on — so all five are **true when written** and take the dated-beside treatment rather than substitution. The table's per-file counts are line counts on the same convention: `docs/ir.md`'s single line carries the string twice, once as the claim and once inside `(3530n + 723) − (5n − 4) = 3525n + 727`. |
| `docs/status.md` and ADR 0104 need nothing | **Verified** | `docs/status.md`'s occurrence sits inside a `**Corrected 2026-08-08**` block that names `3531n + 724` as the tree's current fit and states the whole chain. ADR 0104's four lines — `:22` and `:24`, `:75` and `:77` — sit under its `**Superseded — 2026-08-08**` and `**Extended 2026-08-08**` notes. Neither file was opened for writing, and neither scope was declared. |
| The value comes from the retained run | **Verified, and used that way** | Read out of `results/2026-08-08-post-sourced-semantic-shape-…/growth.tsv` rather than computed: `n = 2 → 7,786`, `n = 11 → 39,565`, `n = 51 → 180,805`, `n = 62 → 219,646`. `graph_bytes(n) = 135n + 149` and the refusal point `19,006` are read from the spike README and the results index. **No figure in this branch's diff was obtained by arithmetic.** |
| The derived figures move and the conclusions do not | **Verified** | The spike records 19,038 → 19,011 → 19,006, the 148/149 crossing unmoved at multiplicity two through both displacements, ×371 in bytes and ×373 in operation count, 3.60 MiB and 5.6% for the whole-model program, and 7.2× over the embedding ceiling. Every one of those is stated in the spike, so each note carries the conclusion and refers the coefficient out. |

**`graph_bytes` — the check the originating ticket did not ask for.** `134n + 149` appears in exactly **one** of the three files, `docs/ir.md`, as part of the same live claim; neither research file carries it. `docs/artifact-abi.md` and ADR 0104 carry it under 2026-08-08 notes and are out of scope.

## Neighbouring census: seven live sites, not five

Two live stale claims sit outside the five this ticket lists, both found by reading the whole file rather than by grepping the fit, and both **true when written** and therefore dated beside:

1. **`docs/research/artifacts/manifest-fixed-content-growth.md`, the headline `Inference` before Section 1** — anchor `so it is the consumer to watch even now that no admitted program reaches it`. It states the 64 MiB bound binding at **19,038** operations, in the present tense, and no correction reaches it: Section 5's 2026-08-07 block says in terms that it corrects "the four paragraphs above and no earlier one". Introduced by `b5789d70` (2026-08-06), when the spike did read 19,038. Two displacements behind.
2. **`docs/research/program-planning/complete-model-ingestion-and-execution.md`, the `Claims are labelled` preamble** — anchor `which reproduces the arithmetic total below rather than replacing it`. It names P3's identity as **7,777 bytes** and the eleven-operation program as **39,502**, and dates the fit "last re-derived 2026-08-07". Introduced by `068b9e1f` (2026-08-06), when the retained `post-explain-ceiling` ladder measured exactly those two values. Two displacements behind on both figures and one day behind on the date.

`docs/ir.md` has no such neighbour: `1154` is the only other fit-bearing paragraph and `1156` already retires it.

## Outcome — repaired 2026-08-08 at base `1438c867`

**Reference over restatement, applied unchanged from the sibling.** Seven dated `2026-08-08` blocks, one per live site, each naming [the identity-growth spike](../spikes/program-planning/identity-growth/README.md) and its [results index](../spikes/program-planning/identity-growth/results/README.md) as the standing authority for the coefficients and for which compiler tree each retained ladder measured, each keeping only the conclusions that survived both measured displacements, and each dating any coefficient it cannot avoid to base `cc667626` and its retained `growth.tsv`. **No site argued for restating**, so no site departs from the sibling's decision.

| Site | Anchor | Live or quoted | Treatment |
| --- | --- | --- | --- |
| [`docs/ir.md`](../docs/ir.md) | `kernel-program identity is exactly` | **live** | Coefficients referred out; the paragraph's own conclusion — quadratic coefficient exactly zero over the governed 2..=62 domain with the wall at 63 — restated without digits. This is the contract site, so it also says explicitly that a coefficient it must carry is a reading of one tree the contract does not track. |
| [`manifest-fixed-content-growth.md`](../docs/research/artifacts/manifest-fixed-content-growth.md) headline | `so it is the consumer to watch even now` | **live**, unlisted | `19,038` dated beside; the ordering it licenses restated. |
| … Section 5 | `is now the reverse of the truth` | **live** | Coefficient and all four solved figures referred out; the 148/149 crossing, the ×371 margin, the 41.9% share, and the ordering kept. |
| … Section 6 | `the ×2 crossing is` | **live** | Short note; Section 5's block holds the derivation, as that section's own 2026-08-07 correction already does for its predecessor. |
| … Section 8 | `the sharpest inversion this correction makes` | **live** | Both items' verdicts restated; the second displacement strengthens the no-out-of-domain-confirmation item rather than weakening it. |
| [`complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md) preamble | `which reproduces the arithmetic total below` | **live**, unlisted | `7,777`, `39,502`, and the re-derivation date dated beside. |
| … `Superseded a third time` block | `Superseded a third time, on 2026-08-07 by` | **live** | Superseded a fourth time; every P1/P2/P3 and whole-model conclusion kept without its digits. |
| [`docs/status.md`](../docs/status.md), ADR 0104 | — | **quoted** | Untouched, as the Facts require. Out of scope and not opened for writing. |

**Anchor preservation, and the disclosure.** The diff is **14 insertions and 0 deletions** (`git diff --numstat`), so every pre-existing byte in all three files is unchanged and every `git log -S` anchor still resolves to its original commit. Two consequences are disclosed rather than hidden:

- **Retired figures are quoted, so their counts rise.** In `docs/ir.md`, `3530n + 723` goes 2 → 3 occurrences and `graph_bytes(n) = 134n + 149` 1 → 2. In the manifest note, `3530n + 723` 4 → 7, `19,038` 6 → 9, `19,011` 2 → 5, `219,583` 3 → 5, `1,046,326` and `1,053,386` 2 → 4 each, `180,753` and `361,506` 1 → 2 each. In the L6 record, `3530n + 723`, `19,011`, `180,753`, `361,506`, `3,770,763`, `7,541,526`, `4,253`, `7,783`, `39,502` each 1 → 2 and `7,777` 2 → 3. Each note says inline that a later hit lands inside it, so a hit is evidence the string is present rather than that the claim stands. `git log -S` on any of them will now name this commit as well as the original.
- **One anchor collision was removed rather than disclosed.** A first draft of the Section 6 note reproduced `the ×2 crossing is`, taking this ticket's own anchor from 1 to 2; it was reworded to `the crossing at that multiplicity remains between`. A ten-word overlap scan of every inserted line against the pre-edit file then found seven more near-quotations of existing sentences — including `What this licenses is still the ordering`, `it binds without a typed refusal from the artifact layer`, and `which is the correction these paragraphs exist to make` — and each was reworded. **The scan now reports zero ten-word overlaps in all three files**, so no inserted prose blunts an anchor into the text it was written to locate. All five anchors this ticket supplies still return exactly `1`.

**Checks.** `./check-citations.sh` exits 0 (944 pinned citations, 6,149 local links). `git diff --check` clean. `tkt lint` and `tkt guard --base 1438c867…` recorded at the commit. **Carry reasoning:** the changed-file list is `docs/ir.md`, `docs/research/artifacts/manifest-fixed-content-growth.md`, `docs/research/program-planning/complete-model-ingestion-and-execution.md`, and this ticket — no path under `crates/`, `prototypes/`, `Cargo.toml`, `Cargo.lock`, `.config/`, `Makefile`, `rust-toolchain.toml`, `rustfmt.toml`, `deps.sh`, or `check-citations.sh`, so the latest green gate carries and the two checks AGENTS.md names for a carry, `tkt lint` and `make citations`, were both run.

**Scopes.** All three exclusive scopes used — `contracts/foundation` for `docs/ir.md`, `research/artifacts` for the manifest note, `research/program-planning` for the L6 record — plus `project/tickets` for this file. None unused; nothing under `spikes/` was edited, and the spike README and results index were read as authority only.
