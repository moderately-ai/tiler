---
id: re-evidence-the-remaining-falsified-greps-in-the-program-planning-records
title: Re-evidence the remaining falsified greps in the program-planning records
status: done
priority: p2
dependencies: []
related: []
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [doc-drift, falsified-evidence]
---
## User-visible outcome

The remaining program-planning records hand a reader commands whose stated output matches what they produce, so re-running one confirms the record instead of appearing to reverse it.

## Why this exists

Filed 2026-08-22 by the coordinator as the sibling-scan residue of [`repair-the-flash-class-records-falsified-supplied-greps`](repair-the-flash-class-records-falsified-supplied-greps.md), which landed as `c07f908f`. That lane repaired its own record in full — two falsified greps re-evidenced and **21** drifted pins turned into anchored citations — and reported these as outside its scope rather than reaching for them. Two are verified by the coordinator at `d19c3b40`; the rest are that lane's readings, marked as such.

**Fact — `minimum-correct-physical-realization-profile.md:84` is a falsified grep of exactly the shape this chain exists to fix, and it is the worst of the set.** Verified by the coordinator. The record states the check `grep -rn '\.rejections()' crates/ --include='*.rs'` *"returns seven sites"* and then enumerates all seven by role. `grep -rn "\.rejections()" crates/ | wc -l` now returns **70**. The pattern is unanchored, so it matches every unrelated `.rejections()` method in the tree — `frontier.rejections()`, `portfolio.rejections()`, `refusal.rejections()` and others. This is the hazard AGENTS.md records under `Anchor the pattern too`.

**The conclusion very likely survives and must be re-evidenced, not withdrawn.** The Fact it supports is *"no production code ever reads it"*, about `SelectedPortfolio::rejections()` specifically, and its three named call sites are stated to be inside a `#[cfg(test)]` module. Re-derive that claim against the *qualified* receiver rather than the bare method name, and prefer a structural reproduction over a count — a bare count is the shape that rots into apparent reversal.

**Fact — `first-metal-lm-workload.md:284` has drifted past its own correction.** Verified by the coordinator. The paragraph is itself a dated repair of an earlier false "returns no output at all" claim, and it records `303 lines across 45 files` at base `428d201d`. The lane reports the same command now returns **312**. So a record that was corrected once has drifted again on the replacement number — which is the argument for the anchored, structural re-evidencing the parent ticket used, rather than a second fresh count that will rot the same way. Note the paragraph's own closing instruction is already *"to read the hits rather than count them"*, so the repair is in the spirit the record itself states.

**Reported by `worker-flash`, unverified by the coordinator:**

- `minimum-correct-physical-realization-profile.md` around line 70: a `grep -n 'semantic_members =='` claim whose raw count is now 15 rather than 4. That lane judged it likely **not** a live defect, because it sits inside a section explicitly marked as a preserved historical snapshot no longer true of the working tree. Confirm that framing before editing — repairing a deliberate historical snapshot would be a defect, not a fix.
- `complete-model-ingestion-and-execution.md` around line 120: `grep -n 'push_storage_scalar'` is described as returning "the definition and both call sites"; the actual grep returns 6 lines including an import and two test lines. Ambiguous whether the prose meant exhaustive or illustrative. Decide by reading, and if it is illustrative, say so rather than inflating the number.
- `flash-class-capability-set.md` lines 72 and 110 claim *"ten `roles.insert` calls"* and *"registers ten families"*, while the actual `roles.insert` count in `governed()` is now **15**. The delivering lane deliberately left this because resolving it means re-deriving axis 2's already-corrected state, which is a conclusion-level question rather than a pin repair. **This one is in `flash-class-capability-set.md`, which that ticket owned** — it is listed here because it was consciously deferred, not missed.

## Required work

- Re-audit every Fact above at your base with a per-Fact verdict, running each command yourself and reporting its actual output. Two are the coordinator's and two are a worker's; treat both as secondhand.
- **Re-evidence rather than withdraw.** Prefer a structural claim that stays true over a bare count. Anchor every pattern and say which unit you report — `grep -c` counts lines, `grep -o … | wc -l` counts occurrences.
- Replace drifted pins with anchors, and run each anchor's grep against the file it names before committing to it.
- **Preserve retired wording** in dated corrections; grep counts cannot shrink across a successful repair.
- For the "ten families" question, decide by reading whether the conclusion still holds. If it does not, stop and report rather than adjusting the number — that is a conclusion change, not a pin repair.

## Non-goals

`crates/`. The flash-class record's seven citations and 21 pins already repaired by `c07f908f`. Re-deciding any program-planning conclusion.

## Closes when

Every supplied command's stated output matches what it produces, each surviving conclusion carries a reproduction that supports it, drifted pins are anchors, the "ten families" question is either resolved by reading or escalated, and the scan is reported with its clean results.
