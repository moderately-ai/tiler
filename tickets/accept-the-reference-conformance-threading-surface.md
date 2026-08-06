---
id: accept-the-reference-conformance-threading-surface
title: Accept the reference conformance threading surface
status: done
priority: p2
dependencies: []
related: []
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: []
---
## The decision

**Only Tom closes this ticket**; it parks at `awaiting-decision` carrying the exact surface. [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md) (merged 2026-08-06) landed eleven additive public items, no existing signature moved: `ReferenceEvaluator::{under, conformance}`, `ReferenceEvaluationRequest::conformance`, `strict_partial_sums_under`, `strict_partitioned_sum_under`, `{silu_f32, certified_exp_f32, certified_rsqrt_f32, rms_norm_f32, softmax_f32}_under`, and `StagedStrictTensorContractionF32::{under, conformance}`. The shape is the crate's own accepted `IndexRegionEvaluator::new`/`under` precedent applied uniformly — one decision repeated, not eleven; changing existing signatures instead was rejected on scope (cross-crate callers fire a stop condition), and a `ConformedReference` facade on architecture (two spellings of one value). Evidence: the per-path decision table in the producing ticket, both threading reverts watched failing, no pinned identity moved (the conformance is a per-evaluation parameter, outside reference identity, with the derivation at `compute_reference_identity`).

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.
