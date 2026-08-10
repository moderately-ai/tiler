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

**Closed after Tom's acceptance (see below).** While open this node parked at `awaiting-decision` carrying the exact surface; only Tom could close it. [`apply-the-declared-numerical-conformance-on-every-reference-evaluation-path`](apply-the-declared-numerical-conformance-on-every-reference-evaluation-path.md) (merged 2026-08-06) landed twelve additive public items, no existing signature moved: `ReferenceEvaluator::{under, conformance}`, `ReferenceEvaluationRequest::conformance`, `strict_partial_sums_under`, `strict_partitioned_sum_under`, `{silu_f32, certified_exp_f32, certified_rsqrt_f32, rms_norm_f32, softmax_f32}_under`, and `StagedStrictTensorContractionF32::{under, conformance}`. The shape is the crate's own accepted `IndexRegionEvaluator::new`/`under` precedent applied uniformly — one decision repeated, not twelve; changing existing signatures instead was rejected on scope (cross-crate callers fire a stop condition), and a `ConformedReference` facade on architecture (two spellings of one value). Evidence: the per-path decision table in the producing ticket, both threading reverts watched failing, no pinned identity moved (the conformance is a per-evaluation parameter, outside reference identity, with the derivation at `compute_reference_identity`).

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.

**Correction — 2026-08-10.** The opening present-tense parking sentence ("Only Tom closes this ticket; it parks at `awaiting-decision`") was leftover framing after closure; status is `done` and the node is not parking. The acceptance-day inventory listed twelve named public items (the prior "eleven" count was wrong against its own list). The accepted name `ReferenceEvaluationRequest::conformance` was later replaced by `ReferenceEvaluationRequest::conformance_for(ArithmeticType)` under [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md); that is pre-alpha vocabulary evolution under the non-stabilization clause above, not a reopen of this accept node. The additive `_under`/`under` shape and the other named items remain.

## Fact audit — 2026-08-10

- Board: `status: done` is correct; parking language is historical only.
- Count: inventory expands to twelve public items, not eleven (same list the producing ticket carried).
- Live request accessor: `conformance_for(ArithmeticType)` at this base; acceptance-day `Request::conformance` spelling is historical inventory.
- No metadata or graph change required; no remainder ticket.
