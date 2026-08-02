---
id: map-evaluation-errors-onto-reference-operation-errors-at-the-nine-collapsed-sites
title: Map evaluation errors onto reference-operation errors at the nine collapsed sites
status: review
priority: p3
dependencies: []
related: []
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785634941
---
## User-visible outcome

The nine `Tensor::dense(…).map_err(|_| ShapeTooLarge)` sites report the cause the evaluation actually produced instead of flattening `EvaluationError::{ShapeTooLarge, ResourceExceeded, ElementCount}` into the shape name.

## Why

**Fact — found by `bound-the-reference-contraction-iteration-space`'s borrowed-uses sweep (2026-08-01).** Nine sites (`evaluate.rs` ×6, `structural.rs` ×1, `contraction.rs` ×2) collapse the evaluation error. Traced at that landing: every site either passed `preflight_f32_output` first or is element-count preserving, so only the `ShapeTooLarge` cause is reachable today — the collapse is defensive, not a live wrong diagnostic. Closing it means choosing one public `EvaluationError` → `ReferenceOperationError` mapping applied across all nine sites, which is a small public-vocabulary decision rather than a mechanical edit.

## Closes when

One mapping is chosen with its derivation, all nine sites use it, and a test demonstrates a non-shape cause reporting under its own name (constructing the case may require relaxing a preflight in the test — if no non-shape cause is reachable even in tests, record that and close by documenting the defensive collapse at each site instead).

## Outcome

**Fact — the population is fourteen, not nine (2026-08-01).** Counted by reading every `Tensor::dense` call in `crates/tiler-reference/src/` and its error handling, not by grep alone: `evaluate.rs` ×6 (lines 277, 305, 316, 364, 447, 506), `structural.rs` ×1 (136), `contraction.rs` ×1 (414), `silu.rs` ×1 (95), `softmax.rs` ×1 (108), `rms_norm.rs` ×1 (125), `bf16.rs` ×1 (723), `quantization.rs` ×2 (176, 207). The `contraction.rs` ×2 in the Why above counts `shape_of` at line 917, which discards a `tiler_ir::shape::ShapeError` from `Shape::try_new` rather than an `EvaluationError` — that mapping is exact and stays as it is. The four scalar families (`silu`, `softmax`, `rms_norm`, `bf16`) and the two `quantization` sites were not in the original sweep; the quantization pair collapsed into `InvalidApplication` rather than `ShapeTooLarge`, which is the same defect under a different name, and they are included.

Left alone deliberately, with the check that established it: `reserve_output_work` (`evaluate.rs:843`) discards an `EvaluationError` from `collect_unseen_tensor_work`, whose complete error set is `{ShapeTooLarge}` (read in full at `evaluate.rs:760-793`), so its `|_| ShapeTooLarge` is exact rather than collapsed. Every `Tensor::scalar` and `Tensor::compound` site keeps its own reporting: `Tensor::scalar` cannot fail at all (shape `[]`, one element of at most `MAX_REFERENCE_ELEMENT_BYTES`), and `Tensor::compound` bounds component count and recursive depth, for which the operation vocabulary has no name — mapping it through a dense mapping would have invented answers.

**The mapping, and it lands entirely on existing variants, so no public-boundary item arises under ADR 0075.** `dense_result_error` in `crates/tiler-reference/src/error.rs` is `pub(crate)`, total over `EvaluationError` by naming every variant rather than by a wildcard, and maps by the quantity the constructor rejected:

| `Tensor::dense` cause | reported as |
| --- | --- |
| `EvaluationError::ShapeTooLarge` | `ReferenceOperationError::ShapeTooLarge` |
| `ResourceExceeded { resource: TensorElements, .. }` | `OutputElementsExceeded { limit, actual }` |
| `ResourceExceeded { resource: TensorBytes, .. }` | `OutputResourceExceeded { limit, actual }` |
| `EvaluationError::ElementCount { .. }` | `InvalidApplication` |

Derivation. The five values `Tensor::dense` can return were read off its body (`tensor.rs:164-206`), not inferred from its documentation. The two resource rows are the variants `preflight_f32_output` already raises for those same two quantities against those same two constants (`evaluate.rs:534-554`), so a family that preflights and one that does not now refuse an over-budget result under one name and carrying one pair of numbers — that agreement is the whole reason the mapping is by quantity rather than by constructor. `ElementCount` means the implementation produced a payload whose length disagrees with the result shape it declared for itself; both facts are its own, so it is invalid state rather than an exceeded bound, and `InvalidApplication` is the name the structural and contraction families already give every other recompute disagreement (`structural.rs:13-18`, `contraction.rs:385-389`). Rejected: `ResultCount { expected, actual }`, whose field names fit but which means the number of ordered *result tensors* at `registry.rs:200` and `:230` — reusing it would have replaced one collapse with another. Rejected: a new `ResultElementCount` variant, which is additive growth a coordinator may merge under ADR 0075 but which spends a public name on a case no site can reach.

**Fact — the ticket's fallback clause is the one that fires, and both halves are done.** Thirteen of the fourteen sites are fully defensive, and the fourteenth is only reachable at a cost no test should pay. Every site takes the result shape's element count successfully before constructing, so `ShapeTooLarge` — the name all fourteen previously reported — is the one cause that cannot arrive at any of them. The retained-byte bound is unreachable by arithmetic: every site builds elements of at most four bytes, so a payload weighs at most `4 * MAX_REFERENCE_TENSOR_ELEMENTS` = 67,108,864 bytes, which is exactly `MAX_REFERENCE_TENSOR_BYTES` and never over it. The element bound is refused ahead of the constructor by `preflight_f32_output` in both reductions, both split-reduction passes, and `ContractionFold::plan` (`contraction.rs:502`), and cannot be exceeded by the families whose result shape is an operand's. That leaves `structural::gather` under a replicating broadcast, where reaching it costs materializing more than `MAX_REFERENCE_TENSOR_ELEMENTS` elements — roughly 400 MB and sixteen million clones — before the constructor is called. Recorded rather than attempted, and the materialization itself is filed as `preflight-the-structural-gather-output-before-it-is-materialized`.

So the test drives the mapping against `Tensor::dense` itself, which is the exact function whose failures the fourteen sites convert. `every_dense_construction_cause_reports_under_its_own_name` (`src/tests.rs`) produces three of the four causes by calling the constructor — an overflowing `element_count`, the element bound, and a payload that disagrees with its shape, all without allocating — states the fourth for the reason above, asserts each maps to its own name, and asserts the four answers are pairwise distinct, because a mapping that answered one name for everything would satisfy every individual row by accident. Watched failing first: re-collapsing the `TensorElements` row to `ShapeTooLarge` produced `left: [ShapeTooLarge, ShapeTooLarge, InvalidApplication, OutputResourceExceeded { .. }]` against the expected four distinct names.

The per-site reachability above is documented once, on `dense_result_error`, rather than fourteen times at the call sites.
