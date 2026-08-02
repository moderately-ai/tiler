---
id: map-evaluation-errors-onto-reference-operation-errors-at-the-nine-collapsed-sites
title: Map evaluation errors onto reference-operation errors at the nine collapsed sites
status: in-progress
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
