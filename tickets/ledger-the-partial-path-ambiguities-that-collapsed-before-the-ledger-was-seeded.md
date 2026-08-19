---
id: ledger-the-partial-path-ambiguities-that-collapsed-before-the-ledger-was-seeded
title: Ledger the partial-path ambiguities that collapsed before the ledger was seeded
status: todo
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

**Fact — the one live citation on it is correct.** `docs/research/extensions/backend-provider-composition.md` row 10 of the seam table writes `check_provenance` (`payload.rs:289`), and line 289 of `crates/tiler-artifact/src/program/codec/payload.rs` is `pub(crate) fn check_provenance(provenance: &PayloadProvenance)`. The same table cell already spells the sibling citation `codec/payload.rs:257`, so the intended file is not in doubt. There is no defect to repair here — only an unrecorded ambiguity.

The parent ticket rejected deriving the whole ledger from `git log` because it cannot tell a deletion from a rename or decide whether two paths ever coexisted; over these 65 paths it would have invented exactly this one failure and caught nothing. Using it as a *review list* rather than as the rule is the remaining useful step.

## Required work

- Re-derive the ever-deleted set at your own base and re-cut it against the suffixes that resolve uniquely; the counts above are stale the moment anything lands.
- For each overlap, read the citing claim and decide which file it is about. Lengthen the citation until its suffix is unique on its own — for the known one, `codec/payload.rs:289` — rather than relocating a line number mechanically.
- Add the retired suffix to the ledger heredoc in `check-citations.sh` with a comment naming the deletion, and raise `LEDGER_FLOOR` by the number added.
- Perturb the subject: with the entry in place and the citation lengthened, restore the deleted twin under a scratch path and confirm the lengthened citation still resolves; then confirm the bare suffix fails. Quote both.

## Closes when

Every pre-seed collapse with a live citation is either lengthened or ledgered, the floor matches the entry count, and `make citations` is green.
