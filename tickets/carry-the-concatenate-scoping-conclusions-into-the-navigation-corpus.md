---
id: carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus
title: Carry the concatenate scoping conclusions into the navigation corpus
status: todo
priority: p2
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, navigation, catalog, carrier]
---
## User-visible outcome

The three navigation surfaces that [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) falsified or left unreconciled agree with it, so a reader reaching Q-SHAPE-006, the support matrix, or the research catalog is not told something the corpus no longer believes.

## Why this exists

**Fact — the record's own scopes cannot reach any of the three.** [`scope-the-concatenate-fusion-role-and-lowering`](scope-the-concatenate-fusion-role-and-lowering.md) declares `research/indexing`, which maps to `docs/research/indexing/**` and `spikes/indexing/**`. Every surface below is `contracts/navigation`.

**Fact — the roadmap was additionally held by a live claim.** At the record's landing, `tkt/record-the-landed-bf16-carrier-in-the-dtype-ledger` had `docs/roadmap.md` in its branch diff (`git diff --name-only d5960e81 tkt/record-the-landed-bf16-carrier-in-the-dtype-ledger`), so file-level disjointness failed there and no partial navigation edit was made. Splitting the three would have left the catalog, the question, and the matrix disagreeing with each other in different directions, which is worse than all three being stale together.

## What the work is

**One — the research catalog row.** `docs/research/README.md` lists one indexing record. Add the row for `indexing/concatenate-fusion-role-and-lowering.md` in the same shape as its neighbour, reading its `disposition`, `evidence_classes`, and `informs` off the record's own frontmatter rather than restating them. Check whether `docs/design-map.md` needs the same reconciliation.

**Two — Q-SHAPE-006's live-pressure bullet.** `docs/open-questions.md` currently says of the concatenate lowering that "The second alternative is available, so the trigger has not fired; it fires if that alternative is eliminated." The record supplies a verbatim-landable replacement under its own "Q-SHAPE-006's firing condition, restated" heading. **The transfer is byte-identical** — a transfer that edits is a fork. The drafted text's paths are written relative to `docs/open-questions.md`, which is where it lands and where they resolve; they do not resolve from the record, and the record says so beside the span rather than repointing them.

**Three — the support matrix's next column.** `docs/roadmap.md`'s `Sequence extension` row says "R5 needs a fusion role" and names no owner, which is the defect the scoping ticket identified as its own reason to exist — every other family whose fusion role is missing names its owner in the same row. Name [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) for R5 and the two lowering tickets for the rung between R5 and R6, and cite the record for the derivation. **Move no rung.** Nothing has been delivered; this is owner bookkeeping and a rung moved here would be a maturity claim nobody earned.

## Explicit non-goals

- Moving any support-matrix rung.
- Restating the elimination. Each surface cites the record; none reproduces its argument.
- Editing the delivery graph's O-07 cells. That record's disposition is `pending` and its cells describe the state at `b63dd5d0`; correcting it is a different owner's and a different scope.

## Closes when

The catalog row exists and agrees with the record's frontmatter, Q-SHAPE-006's bullet is the drafted text byte for byte, the matrix row names its owners with no rung moved, and every local link in the three edits resolves.

## Graph maintenance

- File-level disjointness against any live `contracts/navigation` claim must be re-verified at dispatch time against that worker's actual branch diff, not assumed from this ticket's body — the conflict recorded above was true at `d5960e81` and says nothing about later.
