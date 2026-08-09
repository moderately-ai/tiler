---
id: admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region
title: Admit a second read of one materialized intermediate in an elementwise region
status: awaiting-decision
priority: p3
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, identity]
---
## User-visible outcome

`s * reverse(s)` where `s = sum(a, axis 1)` compiles instead of refusing under `structural-access-conflict` — the staged sibling of the two-reads-of-one-input widening.

## Why this exists (vocabulary audit 2026-08-06; the reachable case and its widening were recorded in the closing two-reads ticket rather than filed)

`reads_bind_boundary_tensors_in_order` (`crates/tiler-ir/src/schedule/builder.rs:789`) admits at most one `TensorRole::Intermediate` read — correctly, because the role carries no ordinal so a second read cannot be attributed to a materialization edge. The declared-input precedent does not transfer unchanged: an intermediate's "ordinal" numbers edges of one cover, not values of one program.

## Boundaries

Adding a field to `TensorRole::Intermediate` is a public boundary (ADR 0074 5b total maps in three compiler files) — draft + acceptance node, never self-accepted. It is also an identity step: `push_tensor_role` writes a bare tag today, so any payload moves every intermediate-touching region — `tiler.schedule.v5` steps and every pin recomputes, executed completely or not started. The dense-leads-mapped canonical order is decided and reused, not re-derived; two structural relations on one intermediate stay refused (nothing ranks two relations).

## Closes when

The pair verifies with different-edge pairs still refused by name; `s * reverse(s)` bit-matches the reference on a reversal-asymmetric fixture; `record_leaf`'s branch narrows rather than deletes; the identity step is complete with pins enumerated; the acceptance node parks.

## Decision Tom needs to make

The source audit above already eliminates “just allow a second `Intermediate`”: the role has no producer-edge identity, so doing that would make two different materialization edges observationally interchangeable. Two honest representation choices remain.

**Option A — replace the unit role with `Intermediate { edge_ordinal }` (recommended).** Every intermediate access states which cover edge it reads, including the current single-edge case. Step the schedule identity once and update every exhaustive consumer. This removes the old shorthand rather than maintaining two spellings, at the cost of moving every intermediate-touching schedule identity in this pre-production tree.

**Option B — retain `Intermediate` and add a separately tagged attributed form.** Existing bytes remain available for the one-edge shorthand, while only multi-edge regions use the payload. This avoids moving old identities, but creates two semantic spellings of an intermediate read and forces every verifier and builder to decide when the shorthand is legal. That ambiguity is permanent maintenance cost and makes canonicalization a new obligation.

**Recommendation.** Accept Option A with the field name `edge_ordinal` and the exact meaning “ordinal of the materialization edge in the verified cover account.” Correctness and one canonical spelling outweigh preserving experimental identity bytes. Acceptance authorizes the representation and identity step only; the implementation still owes all failure-path and bit-agreement evidence above.
