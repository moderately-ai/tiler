---
id: retire-the-draft-labels-on-the-accepted-witness-surface
title: Retire the draft labels on the accepted witness surface
status: in-progress
priority: p3
dependencies: [accept-the-realization-witness-surface-as-built]
related: [accept-the-realization-witness-surface-as-built, implement-the-realization-witness-vocabulary]
scopes: [implementation/ir, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [docs, public-boundary]
claimed_from: todo
assignee: sol-witness-retirement
lease_expires_at: 1786245873
---
## Fact audit — 2026-08-08 at `4be35e12`

**Verified:** Tom accepted seven Delta-2 public items on 2026-08-07 under [`accept-the-realization-witness-surface-as-built`](accept-the-realization-witness-surface-as-built.md): `reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, `unpinned_freedom_site`, `UnrecordedFoldContraction`, and `UnevaluableRealization`.

**False in the original ticket:** only the five accessors carry `**Draft surface, not yet accepted...**` markers. The two payload enums are accepted and already unlabelled. The accepted population is seven; the marker-removal population is five.

**Imprecise in the original ticket:** deleting only the marker would leave false or malformed present-tense prose: `concrete draft pending Tom's acceptance`, `accessor is the draft`, and `and this one is a new site`. The six source blocks to repair are `order` plus the five marked accessors. Preserve every substantive rationale — especially `pass`'s distinction between partial and final passes — while retiring only the stale acceptance-state framing.

**Record population:** the acceptance ticket incorrectly says all seven landed labelled; the implementation ticket's first Outcome correctly names five markers but its later Outcome says seven labelled drafts; the freedom-sites record still presents the five-accessor extension as awaiting acceptance. Correct these with dated forward notes without rewriting the historical implementation-base facts.

## What this owes

Remove the five exact markers and minimally repair their adjacent present-tense draft clauses. `RealizationWitness::order` calls its accepted optional shape a `Drift correction against the drafted surface`; restate it as the accepted shape while preserving the non-reducing/mirror reason.

In the same carrier, correct [`accept-the-realization-witness-surface-as-built`](accept-the-realization-witness-surface-as-built.md), [`implement-the-realization-witness-vocabulary`](implement-the-realization-witness-vocabulary.md), and [the freedom-sites record](../docs/research/reference/plan-freedom-sites.md) to distinguish seven accepted items from five labelled accessors and to record the later acceptance. One carrier keeps that exact population and chronology coherent.

## Explicitly not in scope

**`RealizationWitness` still derives no `PartialEq`**, and this ticket must not add one. That absence is not a draft marker: it is the consequence of the refuted converse in Part 5, owned by [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md). Deriving it here would let a caller read witness equality as function equality while the duplicated-constant spelling still yields two witnesses for one binary32 function.

No signature changes, no new items, no identity movement — this is a documentation change to an accepted surface.

## Closes when

No marker or false present-tense draft clause remains on the five labelled accessors; the two payload enums remain unedited and unlabelled; `order` reads as the accepted optional shape; all three records distinguish seven accepted items from five markers; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-ir` passes. Reinsert one marker and separately the old `order` phrase, require the source-scoped stale-wording check to fail with each exact hit, then restore.

## Graph maintenance

Filed 2026-08-07 by the coordinator at acceptance. Kept separate because the acceptance node's own rule is that released work lands under its own ticket rather than under the decision.
