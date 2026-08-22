---
id: re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree
title: Re-run a merged document's own evidence commands against the merged tree
status: todo
priority: p2
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [graph-hygiene, doc-drift, process]
---
## User-visible outcome

Two merged documents stop handing readers a reproduction command that no longer supports the claim beside it, and the coordinator's merge step gains the check that would have caught both.

## Why this exists

Found 2026-08-22 by the post-chain multi-lens audit, which ranked it its highest-value item — **not because either instance is severe, but because it is the same defect twice in one batch, which makes it a pattern rather than an accident.**

**In both cases a lane published an evidence command that another lane *in the same batch* had already falsified, and neither was caught at merge.** Both were true at the branch base each was written against, and false at the commit each was merged into. Both fail in the direction AGENTS.md calls the dangerous one: a reader is told a check is *absent* when it exists.

**Fact — the plan-freedom site 4.11 correction.** `docs/research/reference/plan-freedom-sites.md`, anchor `returns nothing at this base, over the whole crate rather than only its`. It certifies the site still reserved on the evidence that `grep -rn "CooperativeContraction" crates/tiler-compiler/` returns nothing. True at its cited base `0e28564a`; **already false at its own merge commit `e7a2d0d4`**, because the cost-arm landing had merged earlier at `7a3caca7`. Verified by the coordinator at `7d5fd8ad`: that grep now returns **9** lines. The *classification* survives — none of those hits constructs the topology — but the reproduction handed to the reader no longer supports it.

**Fact — the tile-width spike README.** `spikes/scheduling/metal_contraction_tile_width/README.md`, anchor `make citations` does not reach this directory either. Written at `7fd2f927`, falsified in-range by `3911c827`, which brought spike markdown links into the gate. It states in bold present tense that spike links are unchecked and instructs a future editor to resolve them by hand "because nothing will do it for them". The current run prints `spikes 591 link(s) from the live spike record files above`.

## Required work

- Re-audit both Facts at your base with a per-Fact verdict, running each named command yourself.
- Repair both, **preserving the retired wording in dated corrections** — the claims were true when written and the corrections are about the base, not about the author. Expect grep counts not to shrink.
- Where a claim survives its falsified evidence, say so explicitly and give the reproduction that *does* support it. Site 4.11's classification survives; do not withdraw it, re-evidence it.
- **Sweep the rest of the batch for the same shape**: any document merged in `09474993..f36f7cd9` whose stated command was falsified by a sibling landing. Report findings **and** clean results.

## The coordinator-side half

This is a merge-step defect as much as a document defect. The check that would have caught both: **re-run a merged document's own stated commands against the merged tree, not against the branch base.** Record that where the next cycle reads it — it is cheap, mechanical, and neither instance would have survived it.

## Non-goals

Re-deciding site 4.11's classification, which a lane already re-derived. Changing the citation checker. Any edit to `crates/`.

## Closes when

Both documents carry a reproduction that supports the claim beside it, retired wording is preserved in dated corrections, the batch sweep is reported with its clean results, and the merge-step check is recorded where a later coordinator will read it.
