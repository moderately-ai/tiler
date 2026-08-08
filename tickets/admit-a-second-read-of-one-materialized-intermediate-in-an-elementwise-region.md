---
id: admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region
title: Admit a second read of one materialized intermediate in an elementwise region
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`s * reverse(s)` where `s = sum(a, axis 1)` compiles instead of refusing under `structural-access-conflict` — the staged sibling of the two-reads-of-one-input widening.

## Why this exists (vocabulary audit 2026-08-06; the reachable case and its widening were recorded in the closing two-reads ticket rather than filed)

`reads_bind_boundary_tensors_in_order` (`crates/tiler-ir/src/schedule/builder.rs:789`) admits at most one `TensorRole::Intermediate` read — correctly, because the role carries no ordinal so a second read cannot be attributed to a materialization edge. The declared-input precedent does not transfer unchanged: an intermediate's "ordinal" numbers edges of one cover, not values of one program.

## Boundaries

Adding a field to `TensorRole::Intermediate` is a public boundary (ADR 0074 5b total maps in three compiler files) — draft + acceptance node, never self-accepted. It is also an identity step: `push_tensor_role` writes a bare tag today, so any payload moves every intermediate-touching region — `tiler.schedule.v5` steps and every pin recomputes, executed completely or not started. The dense-leads-mapped canonical order is decided and reused, not re-derived; two structural relations on one intermediate stay refused (nothing ranks two relations).

## Closes when

The pair verifies with different-edge pairs still refused by name; `s * reverse(s)` bit-matches the reference on a reversal-asymmetric fixture; `record_leaf`'s branch narrows rather than deletes; the identity step is complete with pins enumerated; the acceptance node parks.
