---
id: preflight-the-structural-gather-output-before-it-is-materialized
title: Preflight the structural gather output before it is materialized
status: review
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

## Outcome

**Fact — the preflight is in `gather`, not in `BroadcastF32Reference`.** `crates/tiler-reference/src/structural.rs` calls `preflight_f32_output(count)` immediately after `result_shape.element_count()` and before `row_major_strides` and `Vec::with_capacity`. The placement follows the allocation rather than the caller: `gather` is where the payload is reserved and where an element is cloned per result coordinate, so a check there cannot be bypassed by a later third caller, while a check in `BroadcastF32Reference` would leave the allocation unguarded the moment another family gathers. It is defensive for `ReindexF32Reference`, whose result is a permutation of an operand already inside the bound, and load-bearing for `BroadcastF32Reference`, whose `Replicate` axes make the result larger than the operand.

**Measurement — the non-materialization is observed, not asserted.** `a_replicating_broadcast_is_refused_before_its_result_is_materialized` in `crates/tiler-reference/src/tests.rs` evaluates `[3] -> [2^58, 3]` through the standard evaluator. `3 * 2^58` elements of the 24-byte `ReferenceElement` exceed `isize::MAX` bytes, which the test asserts before building the program, so the payload is not a representable allocation and a `gather` that reserved first cannot reach any refusal. Against the un-preflighted path the test panicked with `capacity overflow` at `alloc/src/raw_vec/mod.rs:28`; with the preflight it returns `EvaluationError::Operation` carrying `OutputElementsExceeded { limit: 16_777_216, actual: 864_691_128_455_135_232 }`. An in-bound rank pad in the same test evaluates and replicates, so the refusal discriminates the element count rather than the mapping.

**Fact — the `dense_result_error` reachability note was corrected in the same change.** `crates/tiler-reference/src/error.rs` claimed thirteen of fourteen sites were defensive and named `structural::gather` as the one site that could reach the element bound through a family. That is no longer true, and the paragraph now says every site is defensive and states why the mapping is still tested against the constructor directly.

**Fact — nothing upstream bounds element counts, unchanged.** `crates/tiler-ir/src/shape.rs` bounds rank alone; `Extent::new` admits any `u64` and `BroadcastAxisMapping` bounds axis count, not extent. The test relies on exactly that, and it remains out of scope here.
