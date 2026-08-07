---
id: retire-the-draft-labels-on-the-accepted-witness-surface
title: Retire the draft labels on the accepted witness surface
status: todo
priority: p3
dependencies: []
related: [accept-the-realization-witness-surface-as-built, implement-the-realization-witness-vocabulary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [docs, public-boundary]
---
## What this owes

Seven public items in `crates/tiler-ir/src/schedule/witness.rs` carry `**Draft surface, not yet accepted.**` in their rustdoc. **Tom accepted them on 2026-08-07** under [`accept-the-realization-witness-surface-as-built`](accept-the-realization-witness-surface-as-built.md), so the label now states something false and must go:

`reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, `unpinned_freedom_site`, and the payload enums `UnrecordedFoldContraction` and `UnevaluableRealization`.

Remove the marker only. Do not reword the surrounding rationale — the reasons those accessors exist survive their acceptance, and `pass`'s in particular (a partial and a final pass agree on every other field and commit different values) is the argument for keeping it, not for having drafted it.

## Also owed in the same change

`RealizationWitness::order`'s doc calls itself a "**Drift correction against the drafted surface**". That framing is now historical: the narrowed signature *is* the accepted surface. Restate it as the accepted shape with its reason — a non-reducing region combines no contributors, so a total accessor would return the vocabulary's single variant for a sequence that does not exist — rather than as a deviation from a draft.

## Explicitly not in scope

**`RealizationWitness` still derives no `PartialEq`**, and this ticket must not add one. That absence is not a draft marker: it is the consequence of the refuted converse in Part 5, owned by [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md). Deriving it here would let a caller read witness equality as function equality while the duplicated-constant spelling still yields two witnesses for one binary32 function.

No signature changes, no new items, no identity movement — this is a documentation change to an accepted surface.

## Closes when

No `Draft surface` marker remains on the seven accepted items, `order`'s doc reads as the accepted shape rather than a correction, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir` passes, and a grep for the marker in that file returns nothing.

## Graph maintenance

Filed 2026-08-07 by the coordinator at acceptance. Kept separate because the acceptance node's own rule is that released work lands under its own ticket rather than under the decision.
