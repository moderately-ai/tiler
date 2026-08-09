---
id: route-the-contraction-conformance-through-the-staged-oracle
title: Route the compiler's contraction conformance through the staged oracle
status: done
priority: p2
dependencies: [reclassify-language-model-work-as-a-conformance-track]
related: [bound-the-reference-contraction-comparison-for-the-profile-cells, realize-the-contraction-through-the-appendable-direct-path]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, contraction, language-model, class-generic-capability]
---
## User-visible outcome

`crates/tiler-compiler/src/governed/contraction_conformance.rs` states a boundary that no longer exists, and the four cells it names as uncompared are now reachable. Either it compares them or it says, with a reason, why this crate declines to pay for them — but it stops citing an open ticket that closed.

## What changed underneath it

**Fact — `bound-the-reference-contraction-comparison-for-the-profile-cells` (2026-08-01).** `tiler_reference::StagedStrictTensorContractionF32` folds the governed contraction one output slab at a time, each slab under the same `MAX_REFERENCE_TENSOR_ELEMENTS` work bound the unstaged path is held to. All six L3 profile cells reproduce their retained `direct` `result_sha256` through it; the measured cost is 10.8 s in the dev profile and 5.5 s in release for the whole profile, at a 484 MB peak resident set.

Three statements in that file are now stale and are the reason this ticket exists rather than a comment fix:

- its "The four cells nothing here compares, and why" section names this closed ticket as the owner of an unsettled boundary;
- `the_four_prefill_cells_are_refused_by_the_references_work_bound` still asserts the refusal, which is *correct and worth keeping* — the unstaged path does still refuse — but its documentation frames the refusal as the end of the road rather than as the protection the staged path deliberately leaves standing;
- `ADMITTED_CELLS` and `REFUSED_CELLS` encode a partition of the profile that no longer describes what the reference can do.

## What this must decide

Whether this crate's conformance file compares four more cells at roughly 10 s, compares them behind an `#[ignore]` with a recorded invocation, or declines and says so. All three are defensible; picking one silently is not. Note that `tiler-reference` already runs the six-cell comparison against the reference itself, so what this file would add is the *emitted-lowering* half — and only where the index-region oracle can run, which is a separate bound (see the related ticket).

## Closes when

The file's boundary statement matches what the reference can now do, its cell partition is the current one, and its choice about the four cells is stated with its cost.

## Outcome audit — 2026-08-09

Already delivered; no new source edit is owed. `crates/tiler-compiler/src/governed/contraction_conformance.rs`, anchor `The reference's four-cell boundary was settled`, states that the unstaged evaluator still refuses the four oversized folds and the staged evaluator reaches them. `the_four_prefill_cells_are_refused_by_the_unstaged_fold_and_reached_by_the_staged_one` drives both halves and compares the cheapest staged cell with its retained digest. `ADMITTED_CELLS` and `REFUSED_CELLS` are now explicitly the **unstaged** partition rather than a claim about total reference reach. The module also records why it does not pay for all four here: the four together are 1.1 × 10⁹ steps, while `tiler-reference` owns the full six-cell comparison. The ticket's closure conditions are therefore met and the stale ready node is closed as `done`.
