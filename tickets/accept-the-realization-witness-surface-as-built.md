---
id: accept-the-realization-witness-surface-as-built
title: Accept the realization witness surface as built
status: done
priority: p1
dependencies: []
related: [implement-the-realization-witness-vocabulary, accept-the-realization-witness-surface]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## What is being accepted

[`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md) accepted items A and B on 2026-08-06 against the surface [the freedom-sites record](../docs/research/reference/plan-freedom-sites.md) Part 7.2 drafted. [`implement-the-realization-witness-vocabulary`](implement-the-realization-witness-vocabulary.md) built it on 2026-08-07 and the built surface **differs from the drafted one in two ways**. `AGENTS.md` holds that a tested public boundary stays a labelled draft until Tom accepts its exact included and excluded surface, so those two deltas park here. **Only Tom closes this.**

## Delta 1 — one accepted signature was narrowed, and it is unlabelled

```rust
// Part 7.2 drafted:
pub fn order(&self) -> ContributorOrder;
// built:
pub fn order(&self) -> Option<ContributorOrder>;
```

**The reason is the record's own mirror class.** A region whose topology is `ReductionTopology::None` combines no contributors, so a total accessor has to return the vocabulary's single variant for a sequence that does not exist — a value two plans agree on for no reason about either, which is exactly the failure Part 1 names as a mirror. `None` states the absence instead.

The coordinator reviewed this and agrees with the reasoning; it is raised because it narrows an item the 2026-08-06 acceptance already covered, not because it looks wrong. **The counterpoint worth weighing:** every caller now handles an `Option` for a case that is statically impossible wherever the region is known to reduce, which is a cost paid at every call site to make one accessor honest. The alternative is a total accessor plus a documented convention that its value is meaningless for a non-reducing region — and a documented convention is precisely what the mirror class exists to reject.

## Delta 2 — seven items exist that Part 7.2 drafted no accessor for

Landed **labelled** `**Draft surface, not yet accepted.**`, following the convention at `crates/tiler-ir/src/index/sourced.rs:221`:

`reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, `unpinned_freedom_site`, and the two payload enums `UnrecordedFoldContraction` and `UnevaluableRealization`.

The first three are not additions in substance: Part 2 names `axes`, `contracted_shape` and `pass` in the field sets of sites 4.1, 4.4 and 4.2, while 7.2 drafts no accessor for them. **`pass` is load-bearing** — a partial and a final pass agree on every other field and commit different values, so a witness without it cannot separate two plans that differ.

## What is unchanged from the acceptance

`RealizationWitness` sits in `tiler_ir::schedule` and is aggregated by `RealizationWitness::of`. `UnpinnedFreedomSite` has **no `Conforms`-shaped arm**, which was item B's whole content. Item C — `ReferenceNumericalConformance::from_witness` — was redirected on 2026-08-06 to the plain-scalar form and is not revisited here; no `tiler-reference` file was touched.

## The choice worth objecting to, beyond the two deltas

`RealizationWitness` **derives no `PartialEq`**, and that is a deliberate consequence of a refutation the implementing ticket produced rather than an omission. The record's Part 5 claims canonicalization makes the canonical form a function of the program rather than of the spelling. It holds for the two mitigations the record names and **fails for a third**: nothing shares an identical constant, so `x * 2.0 + 2.0` spelled with one constant and with two yields two witnesses and two canonical schedule identities for one binary32 function. The failure is of the *converse* — the witness is too fine, never too coarse — so nothing unsound follows and an oracle built on it stays fail-closed. Withholding `PartialEq` is what keeps a caller from reading witness equality as function equality. [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md) owns the repair and the identity question under it.

## Identity

**Nothing moved.** `tiler.schedule.v5` is unchanged, no encoder, tag, or domain separator was touched, and the three standard Metal pins were verified identical before the work and at the final commit: artifact identity `23c46a19…`, cache subject `e89c4d82…`, fixed content 64,542 bytes.

## Closes when

Tom accepts both deltas, accepts with a named exclusion, or rejects either. Nothing releases meanwhile; the seven Delta 2 items stay labelled drafts at their definitions, and the `order` narrowing is in use inside `tiler-ir`.

## Accepted — both deltas, 2026-08-07

**Tom accepted both deltas on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, without exclusion.

**Delta 1 accepted.** `RealizationWitness::order` returns `Option<ContributorOrder>` rather than the drafted total `ContributorOrder`. The narrowing stands on its stated ground: a region whose topology is `ReductionTopology::None` combines no contributors, and a total accessor would return the vocabulary's single variant for a sequence that does not exist — the mirror class the freedom-sites record exists to reject. The accepted cost is that a caller unwraps an `Option` at sites where the region is known to reduce and the `None` is statically unreachable.

**Delta 2 accepted.** The seven items Part 7.2 drafted no accessor for — `reduced_axes`, `contracted_shape`, `pass`, `fold_epilogue`, `unpinned_freedom_site`, and the payload enums `UnrecordedFoldContraction` and `UnevaluableRealization` — are accepted as public surface. **Their `**Draft surface, not yet accepted.**` labels must now be removed**, since that marker states a thing that is no longer true; leaving it would be exactly the stale disclosure this repository keeps finding. `pass` is retained on its stated ground: a partial and a final pass agree on every other field and commit different values, so a witness without it cannot separate two plans that differ.

**Unchanged and not revisited.** `RealizationWitness` in `tiler_ir::schedule`, aggregated by `RealizationWitness::of`; `UnpinnedFreedomSite` with no `Conforms`-shaped arm; item C still redirected to the plain-scalar form, with no `tiler-reference` file touched.

**`RealizationWitness` still derives no `PartialEq`**, and this acceptance does not change that. It is the consequence of the refuted converse in Part 5 — the witness is too fine, never too coarse — and [`share-identical-constants-in-the-pointwise-expression-canonical-form`](share-identical-constants-in-the-pointwise-expression-canonical-form.md) owns the repair and the identity question under it. Deriving `PartialEq` before that lands would let a caller read witness equality as function equality.

## Released work

[`retire-the-draft-labels-on-the-accepted-witness-surface`](retire-the-draft-labels-on-the-accepted-witness-surface.md) — the label removal this acceptance owes, released to its own ticket rather than landed here.
