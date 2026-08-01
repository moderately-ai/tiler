---
id: bound-the-reference-contraction-iteration-space
title: Name the reference contraction's iteration-space bound in its own diagnostic
status: done
priority: p3
dependencies: []
related: [admit-the-contraction-normative-reference]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, numerics, contraction]
---
The contraction reference bounds its multiply-accumulate work — `output_count * contracted_count`, which is larger than either operand and bounded by neither tensor limit the operands already passed — and reports the refusal as `ReferenceOperationError::ShapeTooLarge`, whose documented meaning is that *shape arithmetic* exceeded host limits.

The refusal is correct and fails closed. The diagnostic is not: a caller reading `ShapeTooLarge` learns that a shape was too large, when what happened is that a well-formed pair of in-bounds operands named an iteration space this host oracle will not walk. In a crate whose contract is explainable refusal, that gap is worth closing.

## Required delivery

A typed variant naming the bounded resource and carrying its limit and first rejected size, on the pattern `OutputElementsExceeded` already sets, reached by the contraction fold and by anything else that later bounds iteration work rather than storage. `ReferenceOperationError` is `#[non_exhaustive]`, so the variant is additive; it is still a public boundary and goes to Tom with the rest of that batch.

## Closes when

A contraction whose iteration space exceeds the bound is refused under a variant that names the iteration space, with a regression that watches the old and new bounds discriminate — and the existing `ShapeTooLarge` sites keep their meaning rather than being widened to absorb it.

## Outcome

**Fact — the variant.** `ReferenceOperationError::IterationStepsExceeded { limit, actual }`, additive on a `#[non_exhaustive]` public enum (verified at `crates/tiler-reference/src/error.rs:239`), displaying as `reference operation iteration space has {actual} steps, exceeding {limit}` — the `{quantity} … exceeding {limit}` idiom the three `Output*Exceeded` variants already set. It names iteration *work* rather than a stored result, so any later site bounding steps rather than storage reaches it.

**Fact — the site.** `contract_operands` now computes `output_count.saturating_mul(contracted_count)` and refuses over `MAX_REFERENCE_TENSOR_ELEMENTS` under the new variant. Saturating rather than checked: a product too large for `usize` must still refuse, and `usize::MAX` exceeds the limit, so saturation reports a floor of the work instead of turning an unnameable count into a wrapped small number the fold would then walk. The field documents that floor rather than claiming exactness.

**Measurement — the refusal watched failing.** With the site's error temporarily returned as `ShapeTooLarge` and nothing else changed, `an_iteration_space_over_the_bound_is_refused_as_iteration_work` fails on exactly the discriminating case:

```text
thread '…an_iteration_space_over_the_bound_is_refused_as_iteration_work' panicked at
crates/tiler-reference/src/contraction/tests.rs:281:5:
assertion `left == right` failed
  left: Err(ShapeTooLarge)
 right: Err(IterationStepsExceeded { limit: 16777216, actual: 16779424 })
```

The perturbation was reverted. The regression's other two cases pin the discrimination: at `d = 1` the output *is* the iteration space and the stored-element bound refuses first (`OutputElementsExceeded`, 16,781,312 elements), and at `d = 2` with 8,389,712 output elements — under that bound — only the fold's 16,779,424 steps are over a limit. Both refusals land before the fold allocates or steps, so the fixtures are two operands of 5,792 and 5,794 elements and the test runs in 18 ms.

**Fact — the borrowed-uses sweep.** All 39 remaining `ShapeTooLarge` construction sites in `crates/tiler-reference/src` were enumerated by `grep -rn ShapeTooLarge crates/tiler-reference/src` and read in their functions. Three groups, and the first two keep the documented meaning: `Shape::element_count() == None` and `Shape::try_new` failures (shape arithmetic overflowing `u64`), and `usize::try_from(extent)` plus the row-major stride and offset `checked_mul`/`checked_add` chains (linear addressing derived entirely from extents, which does not fit this host). `reserve_output_work`'s `map_err` at `evaluate.rs:777` is in the first group too: `collect_unseen_tensor_work` raises `ShapeTooLarge` and nothing else, so the map preserves rather than collapses. The third group is a genuine collapse and is **not** fixed here: nine `Tensor::dense(…).map_err(|_| ShapeTooLarge)` sites (`evaluate.rs` ×6, `structural.rs` ×1, `contraction.rs` ×2) flatten `EvaluationError::{ShapeTooLarge, ResourceExceeded, ElementCount}` into the shape name. Traced for reachability: every such site either passed `preflight_f32_output` first or is element-count preserving, and each builds a payload of exactly the shape's element count, so only the `ShapeTooLarge` cause is reachable today and the collapse is defensive. Fixing it means choosing a public `EvaluationError` → `ReferenceOperationError` mapping across nine sites, which is a separate decision from this ticket's outcome; recorded here rather than absorbed.

**Fact — one doc claim corrected.** `Tensor::dense`'s `# Errors` named only `ElementCount` and `ShapeTooLarge` while the body also returns `ResourceExceeded` for the element and retained-byte limits. Corrected in place; it is the same vocabulary this sweep read, and a stated error contract that omits a returned variant is a false claim rather than a terse one.

**New public item, for Tom.** `ReferenceOperationError::IterationStepsExceeded { limit: usize, actual: usize }` — additive on a `#[non_exhaustive]` enum, so no external match breaks. No other public signature changed.

**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted `ReferenceOperationError::IterationStepsExceeded` (additive on the `#[non_exhaustive]` enum, saturating count that only under-reports). Recorded for Tom's morning review.
