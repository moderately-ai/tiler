---
id: preflight-the-structural-gather-output-before-it-is-materialized
title: Preflight the structural gather output before it is materialized
status: in-progress
priority: p3
dependencies: []
related: [map-evaluation-errors-onto-reference-operation-errors-at-the-nine-collapsed-sites]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785636853
---
## User-visible outcome

`structural::gather` refuses an over-budget result before allocating and building it, as every other dense reference family already does.

## Why

**Fact — found while mapping the dense-construction errors (2026-08-01, commit `29a9680` + this branch).** `crates/tiler-reference/src/structural.rs` `gather` takes `result_shape.element_count()`, then `Vec::with_capacity(count)` and pushes one cloned element per result coordinate, and only afterwards calls `Tensor::dense`, which is what refuses `count > MAX_REFERENCE_TENSOR_ELEMENTS`. Every sibling dense family calls `preflight_f32_output` *before* the payload loop: `strict_sum` and `strict_partial_sums` at `evaluate.rs:303` and `:445`, and `ContractionFold::plan` at `contraction.rs:502`. `gather` is the only dense site that does not.

**Fact — the result can exceed its operand.** `gather`'s two callers are `ReindexF32Reference`, which is element-count preserving, and `BroadcastF32Reference`, whose `BroadcastAxisSource::Replicate` axes make the result larger than the operand. So a broadcast of a small tensor is the one dense site where the reference element bound is genuinely reachable.

**Fact — nothing upstream bounds it.** `crates/tiler-ir/src/shape.rs` bounds rank (`MAX_SHAPE_RANK = 4_096`) and nothing else; `MAX_REFERENCE_TENSOR_ELEMENTS` is a reference-implementation bound with no semantic-layer counterpart. Checked by reading `shape.rs` in full and grepping the crate for an element-count limit.

**Inference — the cost is the defect.** A broadcast to `MAX_REFERENCE_TENSOR_ELEMENTS + 1` elements allocates roughly 400 MB of `Vec<ReferenceElement>` and performs sixteen million clones before the constructor says no. The refusal is correct; the work spent reaching it is not, and it is the reason the sibling families preflight.

## Closes when

`gather` preflights its result element count before allocating, and a test drives a replicating broadcast past `MAX_REFERENCE_TENSOR_ELEMENTS` and observes `OutputElementsExceeded` without materializing the result — measured, not asserted, since a test that still materializes would pass while proving nothing. Record whether the preflight belongs in `gather` or in `BroadcastF32Reference` alone: `gather` is shared with the reindex family, which cannot exceed the bound, so a preflight there is defensive for one caller and load-bearing for the other.
