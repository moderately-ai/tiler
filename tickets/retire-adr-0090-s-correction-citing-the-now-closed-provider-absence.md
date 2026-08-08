---
id: retire-adr-0090-s-correction-citing-the-now-closed-provider-absence
title: Retire ADR 0090 s correction citing the now-closed provider absence
status: in-progress
priority: p2
dependencies: []
related: [refresh-the-forkless-physical-provider-spike-against-the-landed-seam, record-the-landed-physical-provider-seam-in-adrs-0078-and-0090]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, documentation]
claimed_from: todo
assignee: w-terra-adr
lease_expires_at: 1786201850
---

ADR 0090 carries a dated correction citing an absence that has closed. It is the one unmet item in the spike ticket's "Closes when", left for a branch holding `contracts/decisions`.

## Facts

**Reported by the worker that re-ran the spike, not coordinator-verified — check each.** ADR 0090's dated correction cites the forkless physical-provider spike's recorded **no** answer. That answer has flipped: the spike now runs 8/8 green, with an out-of-tree provider in a separate workspace and its own lockfile implementing the trait, installing, being re-verified, and being retained as an additional alternative naming itself.

**Coordinator-verified:** the spike's re-run landed at merge `eddc398d` and its delta touches no gated path.

**A second, narrower absence remains and must not be swept up with the first.** `Compilation::offered_providers` is still lowering-only. So the correction is **half** stale: the installation absence closed, the disclosure absence did not. A repair reading "this is now closed" without qualification would overstate exactly the way the sibling ADR repairs this week overstated.

## What closes this

The correction dated beside, naming which half closed and which did not, with the spike's re-run result and its host recorded. **Do not substitute** — it was true when written, and the practice here is to date a true-when-written claim beside rather than replace it. That is repository practice, stated in several ADRs while applying it and decided by none; cite the practice, not an authority. A retired sentence quoted verbatim **stays greppable**, so say inline that a later hit lands inside your note.

**Do not write anything implying acceptance.** The seam's public surface is a labelled draft under ADR 0075 awaiting Tom in `accept-the-installed-physical-provider-public-surface`.

**Related and unrepaired, reported by the same worker — report if you meet it, do not reach:** `session.rs` documents `offered_providers` as "the complete frozen provider set offered to this compilation" over a lowering-only set, and `plan_artifact.rs` carries that into artifact provenance. The record judged this "not observably wrong today" because nothing could vary the physical environment — **that condition no longer holds.** It is `implementation/compiler`.

**Cite by searchable anchor, run its grep before committing, and use `grep -F`.** Anchors fail as absence four ways: a line break inside them, an emphasis or backtick marker the source lacks, unescaped brackets read as a character class, and a quoted sentence that never appeared contiguously in source. ADR 0090 in particular has had a cited line rot **twice** — `:1513` → `:2092` → `:2208` — so do not repin by number.

**Check this ADR's neighbouring tree claims and name the count.** A sweep of ADR 0090 this week found **9 of 17** tree-claim clusters false or partly false, most predating the landing that prompted it. Assume the neighbours are unexamined rather than clean.

## Fact audit at `7134a732`, 2026-08-08

- **Verified — the stale correction names the prior negative answer.** ADR 0090's correction headed "the elimination stands and its present-tense evidence sentence does not" retains the `no_physical_provider_installation_seam.rs` `E0599` absence and routes the refresh ticket to re-run it. The retained result at `spikes/extensions/forkless-physical-provider/results/2026-08-08-macos-arm64.json` records the positive replacement instead: `cargo nextest run --workspace` produced 8 passed tests at subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec`.
- **Verified — the worker-reported positive result, with its boundary.** The result records a separate-workspace provider that implements, installs, is re-verified, and is retained as an additional alternative; its toolchain is `nightly-2026-07-19`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, on `aarch64-apple-darwin`, macOS `27.0 (26A5388g)`. It is one dated host-and-toolchain measurement, not a portable guarantee.
- **False — the asserted merge is `eddc398d`.** `git show --no-patch eddc398d` names "Close the index layer siting decision" and changes an unrelated ticket. The spike refresh is `2e97120dfe71ec1377d0097955353824cb3f8a6e`; its result deliberately names the earlier crates subject `cb62784c7d8b63aa2e73c9ac490101b748abc0ec` because the refresh changes no `crates/` path.
- **False — a second disclosure absence remains.** `Compilation::offered_physical_providers` is present and populated from the physical installation that the frontier receives. `Compilation::offered_providers` remains lowering-only by deliberate documented subject, not because the physical environment lacks disclosure. The later landing is `disclose-offered-and-selected-physical-provider-sets-separately`.
- **False and out of scope — the reported `session.rs` documentation defect.** `Compilation::offered_providers` now says "complete frozen *lowering* provider set" and directs readers to `offered_physical_providers`. `plan_artifact.rs` still constructs `CompilationEnvironment` from the lowering set; its identity subject is a separate artifact/build question and is not changed here.
- **Verified — the anchor warning.** The retired `session.rs` line citation is documented in ADR 0090 as having moved three times. The repair uses only source-tested `grep -F` anchors and no new line pin.
- **Imprecise — the stated 9-of-17 neighbour census.** The ticket supplies neither the 17 clusters nor a counting rule, so its exact fraction cannot be independently reproduced. I read the ten neighbouring dated correction clusters in ADR 0090: nine carry superseded or partly superseded present-tense tree claims, and one (the 2026-08-01 payload-provenance correction) remains current. This bounded census supports inspecting the target correction but does not validate the ticket's undefined 17-cluster total.
