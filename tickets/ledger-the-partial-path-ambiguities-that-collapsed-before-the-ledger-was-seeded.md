---
id: ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded
title: Ledger the partial-path ambiguities that collapsed before the ledger was seeded
status: done
priority: p3
dependencies: []
related: [stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin]
scopes: [implementation/workspace, research/extensions]
shared_scopes: [project/tickets]
paths: [check-citations.sh, docs/research/extensions/backend-provider-composition.md]
tags: [gates, citations, correctness]
---
## User-visible outcome

The retired-ambiguity ledger in `check-citations.sh` also covers the basename families that collapsed before it was seeded on 2026-08-19, so a citation on one of those suffixes cannot resolve against a survivor either.

## Why this exists — filed 2026-08-19 from the ledger landing

**Fact.** The ledger seeded by [`stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin`](stop-the-citation-checkers-ambiguity-skip-resolving-against-a-basename-twin.md) records suffixes the checker observed ambiguous *while a live citation rested on them*. A family that collapsed before the seed leaves no such observation, so those suffixes resolve today with nothing recorded to stop them. The seeded ledger carries one such entry by hand, `refinement.rs`, because that collapse is the one the parent ticket documents.

**Measurement 2026-08-19 at `bda38064`.** `git log --diff-filter=D --name-only --pretty=format: HEAD | sort -u` returns 65 ever-deleted tracked paths. Cross-cut against the 99 distinct suffixes that resolve by unique suffix today, exactly one overlaps: `payload.rs`, whose only historical twin is the deleted `prototypes/serial-sum-compile/src/payload.rs`.

**Measurement re-derived 2026-08-19 at `ea321967`, which is the base this work was done from.** The same command returns 66 ever-deleted paths, and `git log --no-renames --diff-filter=D` — the wider list, since `git log` detects renames by default and a renamed-away path collapses a family exactly as a deleted one does — returns 82. The suffixes that resolve by unique suffix are still 99 distinct, now carrying 479 citations rather than the 470 the script header records at `bda38064`. Both cuts leave the same single overlap, `payload.rs`. So the shape of the measurement holds and every count in it moved.

**Fact — the citing claim names the codec file, and the reading that establishes that also corrects this ticket.** `docs/research/extensions/backend-provider-composition.md` row 10 of the seam table names `check_provenance`, and the same table cell already spells its sibling citation `codec/payload.rs:257`, so the intended file is not in doubt. There is no defect in the row — only an unrecorded ambiguity.

**Correction 2026-08-19 at `ea321967`.** The paragraph above previously asserted that the cited line of `crates/tiler-artifact/src/program/codec/payload.rs` is `pub(crate) fn check_provenance(provenance: &PayloadProvenance)`. That is false at this base and was already false at `bda38064`: the signature sits two lines further down, and the cited line is the third line of its `# Errors` doc comment. It is true at `51e9374a`, and that is the point — the table lives under the heading `Thirteen-row maturity audit, 2026-08-05 at base` and its own preamble says a stale statement `is corrected here rather than edited in place`, so its five line pins are a dated reading and not a claim about today's tree. A worker who trusted the original wording would have "repaired" a correct dated audit into a wrong one. **The line numbers in that row must not be relocated; only the path is lengthened.**

**Correction 2026-08-19 at `ea321967` — two live citations rested on the suffix, not one.** The heading above read `the one live citation on it is correct`. At this base two live records resolved through the bare suffix: the seam-table row, and this ticket's own Fact paragraph, which spelled the pin inside a code span and was therefore itself a citation the checker resolves. Both had to be lengthened before the ledger entry could land, because a ledgered suffix matching exactly one file fails.

The parent ticket rejected deriving the whole ledger from `git log` because it cannot tell a deletion from a rename or decide whether two paths ever coexisted; over these 65 paths it would have invented exactly this one failure and caught nothing. Using it as a *review list* rather than as the rule is the remaining useful step.

**Correction 2026-08-19 at `ea321967` — the failure was not invented, and the reading is what shows it.** The parent's rejection of the mechanism stands untouched; what does not survive is the clause that the one overlap would be a fabrication. Reading its history rather than its deletion line: `7e01f3b7` added `prototypes/serial-sum-compile/src/payload.rs` on 2026-07-25 beside the codec file, `2d2a7bd7` removed it on 2026-07-28 with 152 lines deleted and no matching addition anywhere in that commit, `git ls-tree 2d2a7bd7^` carries both paths while `git ls-tree 2d2a7bd7` carries one, and inside that window a `todo` ticket pinned the bare suffix — `tickets/stop-recomputing-pure-derivations-in-the-codec.md` at `3dacabce`, which frontmatter at that commit shows was live. The family really was ambiguous with a live citation resting on it, which is the ledger's own admission criterion; the only thing missing was a run to see it, because this script did not exist until `7e3a7367` on 2026-08-07. That is exactly what "collapsed before the ledger was seeded" names, and it is the distinction the mechanism cannot draw and a reading can.

## Required work

- Re-derive the ever-deleted set at your own base and re-cut it against the suffixes that resolve uniquely; the counts above are stale the moment anything lands.
- For each overlap, read the citing claim and decide which file it is about. Lengthen the citation until its suffix is unique on its own — for the known one, `codec/payload.rs:289` — rather than relocating a line number mechanically.
- Add the retired suffix to the ledger heredoc in `check-citations.sh` with a comment naming the deletion, and raise `LEDGER_FLOOR` by the number added.
- Perturb the subject: with the entry in place and the citation lengthened, restore the deleted twin under a scratch path and confirm the lengthened citation still resolves; then confirm the bare suffix fails. Quote both.

## Closes when

Every pre-seed collapse with a live citation is either lengthened or ledgered, the floor matches the entry count, and `make citations` is green.

## Outcome

**The overlap set, re-derived at `ea321967`.** One suffix, `payload.rs`, under both the default review list (66 ever-deleted paths) and the wider `--no-renames` one (82). No second entry was warranted and none was invented.

**What was lengthened.** `docs/research/extensions/backend-provider-composition.md` row 10 now spells `codec/payload.rs:289`, matching the sibling citation in its own cell. The line pin is deliberately unchanged: it is correct at the base that section declares, and the section states that stale statements are corrected below it rather than edited in place. This ticket's own Fact paragraph carried the second citation on the bare suffix and no longer pins one.

**What was ledgered.** `payload.rs`, with a comment naming the deletion, the not-a-rename evidence, and the live citation inside the coexistence window. `LEDGER_FLOOR` 41 → 42, and the census reads `42 ledger entry(s) against a floor of 42`.

**Perturbations, all three run against the subject.** With the entry present and the twin absent, a planted bare pin fails: `payload.rs is on the retired-ambiguity ledger in check-citations.sh, and exactly one tracked file ends with it now`, exit 1. With the entry deleted and the twin still absent, that same planted pin produces zero `FAIL` lines and resolves silently against the survivor — the defect — while the floor fires `SHORT  the retired-ambiguity ledger population reached 41 entry(s), below its floor of 42.`, exit 1. With the entry present and the twin restored under a staged scratch path, the run is green at exit 0, the lengthened citation still resolves because exactly one tracked file ends with `codec/payload.rs`, and the bare pin is `SKIP ... (ambiguous, 2 candidates, on the ledger)` rather than a false failure.

**Remainder, not done here.** The thirteen-row audit's line pins are dated to `51e9374a` and 505 files under `crates/` have changed between that base and `ea321967`, so most of that table's pins no longer name what they named. That is a re-audit of the record, not a citation-checker defect — the checker only tests that a line is inside its file — and it belongs in its own ticket.
