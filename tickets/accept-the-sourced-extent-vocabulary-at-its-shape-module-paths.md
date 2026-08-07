---
id: accept-the-sourced-extent-vocabulary-at-its-shape-module-paths
title: Accept the sourced extent vocabulary at its shape module paths
status: awaiting-decision
priority: p1
dependencies: []
related: [relocate-the-sourced-extent-vocabulary-to-the-shape-module]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## What is being accepted

Five public items changed module path on 2026-08-07 under [`relocate-the-sourced-extent-vocabulary-to-the-shape-module`](relocate-the-sourced-extent-vocabulary-to-the-shape-module.md). **No signature, field, variant or behaviour changed** — only where each is named. Under ADR 0075 a changed public path is a public boundary exactly as a changed signature is, so this parks until Tom closes it. **Only Tom closes it.**

| Item | Was | Is |
| --- | --- | --- |
| `SourcedExtent` | `tiler_ir::index::SourcedExtent` | `tiler_ir::shape::SourcedExtent` |
| `SourcedShape` | `tiler_ir::index::SourcedShape` | `tiler_ir::shape::SourcedShape` |
| `ExtentSources` | `tiler_ir::index::ExtentSources` | `tiler_ir::shape::ExtentSources` |
| `ExtentSourceError` | `tiler_ir::index::ExtentSourceError` | `tiler_ir::shape::ExtentSourceError` |
| `EXTENT_PHASE_CEILING` | `tiler_ir::index::EXTENT_PHASE_CEILING` | `tiler_ir::shape::EXTENT_PHASE_CEILING` |

**No compatibility re-export was left behind**, deliberately: this is pre-alpha with no external consumers, and `AGENTS.md` requires a complete replacement to remove the superseded path rather than preserve it. A re-export would reinstate the second spelling the relocation exists to remove. Fourteen call sites moved with it.

## What deliberately did NOT move, and the argument is the interesting half

The ticket named **six** items. Only five moved, and the sixth was argued rather than forgotten.

**`SymbolicExtentError` stays in `index`.** It is `Source(ExtentSourceError) | Structural(IndexBuildError) | ShapeVocabulary(ShapeError)`. Moving it to `shape` would make the crate's *base* vocabulary name `crate::index::IndexBuildError` — **inverting the exact layering the relocation argues for** — and it would not deliver the sharing anyway: a second consumer refusing a sourced extent puts *its own* build error in the structural slot, so it needs its own union. Only `ExtentSourceError` is the shared authority, and that is what moved. This is the same argument the `ShapeVocabulary` variant already carries one level down.

**`SourcedIndexInteger` stays in `index`** for the same reason, and the ticket could not have named it: it did not exist when the ticket was filed, arriving with the `v9 → v10` step. It is `IndexInteger | ShapeSymbol`, so relocating it inverts the layering identically. It keeps its own draft label and its own pending acceptance.

## The evidence that the move is pure

**Canonical bytes proved identical, not argued identical.** The worker materialized the base tree as a plain directory, installed an identical throwaway probe in both trees, and compared encodings: one wholly static region (1,090 bytes) and one carrying a symbolic boundary, a symbolic divisor and a symbolic coefficient (1,466 bytes) — `diff` reports identical, same SHA-256 both sides.

`INDEX_REGION_DOMAIN` stays `tiler.index-region.v11`. The pinned population was enumerated — 8 files, 35 literals, none touched — and the standard Metal identity test was run explicitly rather than inferred from a green suite. Test-site count is 3,056 at base and at head, matching per-file across all eleven touched files, so nothing was dropped in the move. One refusal was observed failing and restored.

## The choice worth objecting to

**Whether `shape` is the right home at all.** The relocation's case is that these five are about *extents and their provenance* rather than about index expressions, and that `index` naming them made the shape module depend upward. The counterpoint: `ExtentSources` is consumed almost entirely by index-region construction and proof, so the items now live away from every one of their callers. That is a real cost in navigability, traded for a layering the crate can state.

If you would rather they had stayed, the cost of reverting rises with every consumer added — but nothing else has been built on the new paths yet.

## Closes when

Tom accepts the five paths, accepts with a named exclusion, or rejects. Nothing releases meanwhile; the items are in use inside `tiler-ir` at their new paths and no old path survives.
