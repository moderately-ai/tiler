---
id: admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region
title: Admit a second read of one materialized intermediate in an elementwise region
status: todo
priority: p3
dependencies: []
related: [admit-two-reads-of-one-declared-input-in-an-elementwise-region, admit-a-scheduled-region-that-reads-two-materialization-edges]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, identity]
---
## User-visible outcome

`s * reverse(s)` where `s = sum(a, axis 1)` compiles instead of refusing under `structural-access-conflict` — the staged sibling of the two-reads-of-one-input widening.

## Why this exists (vocabulary audit 2026-08-06; the reachable case and its widening were recorded in the closing two-reads ticket rather than filed)

`reads_bind_boundary_tensors_in_order` (`crates/tiler-ir/src/schedule/builder/elementwise.rs`, source anchor `fn reads_bind_boundary_tensors_in_order`) admits at most one `TensorRole::Intermediate` read — correctly, because the role carries no ordinal so a second read cannot be attributed to a materialization edge. The declared-input precedent does not transfer unchanged: an intermediate's "ordinal" numbers edges of one cover, not values of one program.

## Boundaries

Adding a field to `TensorRole::Intermediate` is a public boundary (ADR 0074 5b total maps in three compiler files) — draft + acceptance node, never self-accepted. It is also an identity step: `push_tensor_role` writes a bare Intermediate tag today, so any payload moves every intermediate-touching region — implementers must account for the schedule domain, the parallel structured-kernel encoder, and any other domain that encodes `TensorRole`; Intermediate-touching subjects and every pin recompute, executed completely or not started.

**Correction — 2026-08-19 (identity versions and encoder population).** This paragraph originally named `tiler.schedule.v5` and "current `tiler.kernel.v7`". Both are stale: at this base the domains are `tiler.schedule.v7` and `tiler.kernel.v9` (`crates/tiler-ir/src/domains.rs`, anchors `tiler.schedule.v7` and `tiler.kernel.v9`; `KERNEL_DOMAIN` in `crates/tiler-ir/src/kernel/model.rs` holds the same value). The pinned numbers are replaced with the domains' names rather than with fresh numbers, because the version this ticket steps is whichever is live when it is implemented — pinning a number here is what rotted the first pair. The encoder population is also wider than "schedule and kernel": **four** `fn push_tensor_role` encoders write a bare `Intermediate` tag and would each have to carry `edge_ordinal` — `crates/tiler-ir/src/schedule/model.rs`, `crates/tiler-ir/src/kernel/model.rs`, `crates/tiler-compiler/src/selection.rs`, and `crates/tiler-compiler/src/frontier.rs`. The accepted Option A below is unaffected; the migration it charges the implementer with is larger than the sentence implied. **Unverified at this base:** the "ADR 0074 5b total maps in three compiler files" count above — 11 files under `crates/tiler-compiler/src/` name `TensorRole::Intermediate`, so re-derive it by reading each rather than trusting the three. The dense-leads-mapped canonical order is decided and reused, not re-derived; two structural relations on one intermediate stay refused (nothing ranks two relations).

## Closes when

The pair verifies with different-edge pairs still refused by name; `s * reverse(s)` bit-matches the reference on a reversal-asymmetric fixture; `record_leaf`'s branch narrows rather than deletes; the identity step is complete with pins enumerated; the acceptance node parks.

## Decision Tom needs to make

The source audit above already eliminates “just allow a second `Intermediate`”: the role has no producer-edge identity, so doing that would make two different materialization edges observationally interchangeable. Two honest representation choices remain.

**Option A — replace the unit role with `Intermediate { edge_ordinal }` (recommended).** Every intermediate access states which cover edge the access binds (producer or consumer), including the current single-edge case. Step the schedule identity once and update every exhaustive consumer. This removes the old shorthand rather than maintaining two spellings, at the cost of moving every intermediate-touching schedule identity in this pre-production tree.

**Option B — retain `Intermediate` and add a separately tagged attributed form.** Existing bytes remain available for the one-edge shorthand, while only multi-edge regions use the payload. This avoids moving old identities, but creates two semantic spellings of an intermediate read and forces every verifier and builder to decide when the shorthand is legal. That ambiguity is permanent maintenance cost and makes canonicalization a new obligation.

**Recommendation.** Accept Option A with the field name `edge_ordinal` and the exact meaning “ordinal of the materialization edge in the verified cover account.” Correctness and one canonical spelling outweigh preserving experimental identity bytes. Acceptance authorizes the representation and identity step only; the implementation still owes all failure-path and bit-agreement evidence above.

## Public-boundary acceptance — 2026-08-12

**Decision — Option A accepted by Tom in the live coordination session.** Replace the unit role with the single canonical attributed form `Intermediate { edge_ordinal }`; do not retain an unattributed shorthand. The ordinal names an edge in the verified cover account, not a semantic value, access position, extent, or region-local temporary. Construction and assembly must reject missing, duplicate, foreign, and mis-bound ordinals before lowering, with no inference or default.

This acceptance is shared by both consumers already in the graph: repeated reads of one edge and reads of two distinct edges. The implementation owns one coherent schedule/kernel identity migration across every total encoder and consumer. It must perturb the ordinal while holding shapes and access order fixed, show the resulting mis-binding refusal, and enumerate every moved identity pin on the merged tree.
