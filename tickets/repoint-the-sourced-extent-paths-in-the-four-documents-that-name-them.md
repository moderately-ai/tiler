---
id: repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them
title: Repoint the sourced extent paths in the four documents that name them
status: done
priority: p3
dependencies: []
related: [relocate-the-sourced-extent-vocabulary-to-the-shape-module]
scopes: [contracts/foundation, research/shapes, research/numerics]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [docs, doc-drift]
---
## What is stale

Five items moved from `tiler_ir::index::` to `tiler_ir::shape::` on 2026-08-07 (`relocate-the-sourced-extent-vocabulary-to-the-shape-module`), with **no compatibility re-export** — the old paths do not exist. Four documents still name them, each outside that ticket's scopes, which is why they were reported rather than fixed:

- `docs/ir.md` — `contracts/foundation`
- `docs/roadmap.md` and `docs/open-questions.md` — `contracts/navigation`
- `docs/research/shapes/symbolic-semantic-extents.md` — `research/shapes`
- `docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md` — `research/numerics`

Verify each at your own base; do not trust this list to be exhaustive, and report anything else you find.

## Two defects in the shapes record beyond the path

Both found by the relocating worker and neither caused by it:

1. **Its A1 lists the wrong six-item set.** Only **five** items moved. `SymbolicExtentError` deliberately stayed in `index`, because it unions `IndexBuildError` and moving it would make the crate's base vocabulary name `crate::index::IndexBuildError` — inverting the layering the relocation exists to establish. `SourcedIndexInteger` stayed for the same reason. Correct the set **and** the reason, since a list without the argument invites the next reader to "finish" the move.
2. **Its line-32 Fact — "grepping the semantic module returns nothing" — was already false at base**, before the relocation: `semantic/slice.rs` and `semantic/softmax/tests.rs` both name `SourcedExtent`. That is a stale premise independent of this move and should be repaired as such rather than folded into the path update.

## How to repair

**Do not blind-substitute the path.** For each site, read what the sentence claims: some name the path incidentally and take a straight repoint; others make a claim *about where the vocabulary lives*, which the relocation changed and which needs restating rather than rewriting. A dated correction is the convention where the sentence was true when written — `docs/compiler/optimizer.md` and `docs/architecture.md` both carry that shape from this week.

The paths are **not** accepted yet: [`accept-the-sourced-extent-vocabulary-at-its-shape-module-paths`](accept-the-sourced-extent-vocabulary-at-its-shape-module-paths.md) is parked for Tom. So state the current path as current, and do not describe it as settled.

## Closes when

No document names a `tiler_ir::index::` path for the five moved items; the shapes record's item set and its false Fact are both corrected with reasons; each site is repointed or restated according to what it claims; and any further stale site found is fixed or reported.

## Outcome — delivered 2026-08-07 at `1b4d4bae`

**Only one document carried a fully qualified `tiler_ir::index::<item>` path.** The other four sites made *location claims in prose*, so the ticket's "do not blind-substitute" instruction was load-bearing for four of five — a find-and-replace would have repaired nothing and left the claims false.

**Two of the four files this ticket named had no relocation-caused staleness at all.** `docs/roadmap.md` and `docs/open-questions.md` contain no `tiler_ir::index` and cite `index/model.rs`, which did not move. My ticket was wrong to list them; their real drift is **coefficient-era** and a different cause.

**A fifth site was found beyond the four named** — `transformer-operation-and-shape-surface.md`, whose sentence located the vocabulary by module and file line.

### The shapes record's two defects, and the second is subtler than reported

The wrong six-item set was corrected to five **with the argument**, not just the count — `SymbolicExtentError` unions `IndexBuildError` so moving it inverts the layering, and it delivers no sharing because a second consumer puts its own build error in that slot.

The line-32 Fact was verified false **at base**, before the relocation, by re-running its check at a pre-relocation commit. But the distinction the worker drew is the valuable part: **the Fact's *claim* survives and only its *check* was wrong.** Both hits are doc-comment prose *about* the index vocabulary, not a symbol reaching a semantic value. So it was repaired as a broken check — the reproducible form now excludes comment lines — rather than as a false claim, and its positive control was dated rather than silently bumped from six files to ten.

### The judgement worth keeping

It repointed six drifted line ordinals in a paragraph it restated, and **deliberately did not repoint two others**, because those citations sit beneath a claim that is still false: *repointing a citation attached to a false claim makes it read as freshly verified.* That reasoning is recorded in the document rather than only in the report.

### Released, with the trap named

[`correct-the-symbolic-coefficient-era-index-vocabulary-claims`](correct-the-symbolic-coefficient-era-index-vocabulary-claims.md) — six sites still say a bound symbol cannot be an index coefficient or addend, which the coefficient admission falsified. The ticket names the trap explicitly: **the literal wording survives for `SourcedExtent` while the claim it supports does not**, so find-and-replace produces true-but-misleading sentences. Repairing it also needs re-deriving what the sub-tensor-selection symbolic-offset trigger now blocks on.

### A board defect it relayed, and the coordinator's error

It hit `admit-symbolic-index-expression-coefficients` as a live `contracts/foundation` collision, checked whether that was concurrent editing, and correctly diagnosed **stale ticket state**: the deliverables were already ancestors of its base. **That ticket had landed hours earlier and I closed only its acceptance node, leaving the implementation ticket `in-progress` with a live claim.** Now closed with its outcome recorded. `tkt reconcile` then surfaced an orphan branch from the released sourced-shape ticket, verified to hold zero commits before deletion; the board and git now agree.

**Delta rule confirmed against the merge's own file list:** five files under `docs/` and `tickets/`, none under the build-configuration set.
