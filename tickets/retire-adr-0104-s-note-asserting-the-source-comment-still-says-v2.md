---
id: retire-adr-0104-s-note-asserting-the-source-comment-still-says-v2
title: Retire ADR 0104 s note asserting the source comment still says v2
status: done
priority: p2
dependencies: []
related: [step-the-coverage-identity-comment-s-stale-semantic-graph-domain]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, identity, documentation]
---

ADR 0104 carries a note explaining why its quotation was left alone. **The repair it was waiting for has landed, so the note is now the stale part.**

## Facts, coordinator-verified at the merge that landed the repair

**Fact.** The note is anchored by `The stale text is the source comment, not this record`, and it justifies leaving the quotation untouched on the ground that the doc comment on `IndexRefinementExecutableCoverageIdentity` **still says `v2`**.

**Fact.** It no longer does. `step-the-coverage-identity-comment-s-stale-semantic-graph-domain` stepped it to `v3`, dated beside, with the retired spelling quoted in its own note.

**Fact — the direction matters and a coordinator brief got it backwards.** I told that worker the ADR's quotation would become "accurate again by construction" once the source was repaired. The opposite is true, and this ADR says so itself: the quotation was faithful *because* the source said `v2`. Repairing the source is what makes **this note** stale. The worker caught the inversion and reported it rather than working to the brief.

**Fact.** ADR 0104 contains **two** occurrences of `tiler.semantic-graph.v2`, not one: the quotation, and a `Superseded — 2026-08-08` header reading "stepped `tiler.semantic-graph.v2` to `v3`" — a correct historical statement that is **not** a quotation and must not be touched.

## Worker's per-Fact audit, re-read at base `b9cf969b1e71fbf45f675f7a08a6b9008eed87b1`

| Ticket Fact | Verdict | Evidence at this base |
| --- | --- | --- |
| The ADR note says the source comment still says `v2` and is anchored by `The stale text is the source comment, not this record` | **verified** | The short fixed-string anchor resolves once in ADR 0104, inside the note beside the quotation. |
| The source no longer says `v2`; the sibling ticket stepped it to `v3`, dated beside, and retained the retired spelling in its note | **verified** | `refinement.rs` has the live, break-free anchor `are not re-encoded:` naming `tiler.semantic-graph.v3` and the dated correction anchor `The retired spelling` quoting `tiler.semantic-graph.v2`. Commit `b6e425ba713788b811986c6b6e919b766053260f` made that source repair on 2026-08-08; `c507f5b6514d28744dec901f9295aa4fd70ba4a5` later closed the sibling and filed this remainder. Both are ancestors of this base. |
| Repairing the source makes this note stale rather than making the ADR quotation current | **verified** | `git show b6e425ba -- crates/tiler-ir/src/index/refinement.rs` replaces the live `v2` sentence with `v3` and deliberately adds the retired `v2` sentence to the correction note. The ADR quotation therefore remains faithful to the historical source while its present-tense note no longer describes the live paragraph. |
| ADR 0104 has exactly two `tiler.semantic-graph.v2` occurrences before this repair | **verified** | `rg -n -F 'tiler.semantic-graph.v2' docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` returns the `Superseded — 2026-08-08` header and the quotation at `Stop restating the graph per record`, and nothing else. |

**No Fact repair changes this ticket's purpose.** All four Facts hold at this base; the edit remains the dated retirement of the now-stale ADR note, with the quotation and historical supersession header preserved.

## ADR 0104 neighbouring tree-claim census

**Count and rule: four clusters.** `grep -c '^\*\*Fact' docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` returns **4**. The census treats each paragraph beginning with the record's explicit `**Fact —` label as one tree-claim cluster; it excludes dated measurements, decision/proposal text, and paragraphs already introduced as `Corrected`, `Superseded`, or `Extended`, so historical evidence is not silently reclassified as a live claim.

| Anchor | Verdict against the current tree |
| --- | --- |
| `the restatement is at one site` | **verified as accepted-time context; superseded in the current tree and already bounded by the Status and Decision.** At the ADR's authoring commit `09d1666a`, `encode_executable_coverage_identity` opened with the framed graph preimage. It now writes `DigestAlgorithm::GOVERNED.digest(COVERAGE_GRAPH_DIGEST_DOMAIN, subject.graph.as_bytes())`, exactly the execution the record decides and its Status announces. |
| `the encoded copy has no reader anywhere in the workspace` | **verified as the one still-live cluster.** The opaque type exposes only `as_bytes`; its two `compile_fail` doctests still hold the missing constructors, and current uses are larger-identity writers, capacity sizing, or the identity-growth measurement. `ForeignCoverageGraph` still reads `CoveredOccurrence::graph`, not encoded coverage bytes. |
| `the ceiling that binds is not the one` | **verified as accepted-time measurement; superseded and already dated.** The current retained ladder and the ADR's 2026-08-08 header/Bounds extension replace the pre-fold 695 and 50/51 figures with the linear `3531n + 724` result, 19,006-operation 64 MiB crossing, and unchanged 148/149 embedding crossing. |
| `the crate that mints the coverage identity cannot reach` | **verified as the accepted-time blocker; answered and already bounded.** At `09d1666a`, `tiler-ir` depended only on the three `num-*` crates and `tiler-artifact` owned the digest. Today `tiler-ir` depends on the new bottom crate `tiler-digest`; the ADR's `Answered by Tom on 2026-08-06` paragraph and Status record that execution. |

**Result: no neighboring repair belongs in this ticket.** One cluster remains true of the live tree; the other three are true historical premises whose changed state is already explicit in the same record. Updating them would widen this note repair into rewriting the accepted decision's historical derivation.

## What closes this

The note restated to record that the source was repaired and when, so a reader can tell the quotation is a **historical** one rather than a current reading. **Do not edit the quotation itself** — it remains a faithful record of what the comment said, and the sibling deliberately preserved that by quoting the retired spelling in its own dated note.

**Establish the treatment from history**: this note was true when written, so it is dated beside rather than substituted. That is repository practice — several ADRs state it while applying it and none decides it; cite the practice, not an authority. Say inline that a grep for the retired spelling now lands inside a note, since three of this ADR's occurrences will be exactly that.

**A caveat on anchors, verified both ways by the sibling.** The ADR's *full* quoted sentence does **not** grep in `refinement.rs` and never did — the doc comment wraps it across three lines at 80 columns, so it returned 0 before and after. Only the short fragment resolves. Choose a short, break-free anchor and **run its grep before committing to it**; note also that unescaped brackets read as a character class, so `grep -F` where a citation contains them.

**Check this ADR's other claims about the tree and name the count.** A prior sweep of a sibling ADR found 9 of 17 tree-claim clusters false, most predating the landing that prompted the ticket — so assume the neighbours here are unexamined rather than clean.

## Outcome — repaired 2026-08-08 at base `b9cf969b1e71fbf45f675f7a08a6b9008eed87b1`

ADR 0104 retains the original note because it was true when added and now dates a correction beside it. The correction names the source-repair ticket and commit `b6e425ba`, records that the live break-free source anchor `are not re-encoded:` now names `tiler.semantic-graph.v3`, and explains why the quotation remains a faithful historical one.

`rg -n -F 'tiler.semantic-graph.v2' docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md` returns exactly **three** occurrences after the repair: the historical supersession header, the preserved quotation, and the correction that makes those hits interpretable. The four-cluster neighbouring audit above found no second repair in scope.

This is a docs/ticket-only delta and touches none of the paths that invalidate reuse of the latest green full gate under `AGENTS.md`; this worker therefore does not rerun `make full`. The required delta checks passed: `git diff --check`, `tkt lint --format json`, and `make citations`. The branch guard is rerun against the exact base after commit, when it can inspect the committed branch diff.
