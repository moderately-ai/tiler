---
id: repoint-the-sourced-extent-paths-in-the-four-documents-that-name-them
title: Repoint the sourced extent paths in the four documents that name them
status: todo
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
