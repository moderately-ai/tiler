---
id: exercise-a-multi-entry-route-with-shared-allocations-through-the-adapter-seam
title: Exercise a multi-entry route with shared allocations through the adapter seam
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: worker-multi-entry
lease_expires_at: 1785563203
---
## User-visible outcome

The adapter seam's multi-entry and shared-allocation paths are exercised by a fixture that reaches them, instead of only by the empty case a single-entry fixture produces.

## Why

**Fact — filed from `route-a-custom-backend-through-an-independently-selected-adapter`'s own boundary statement (2026-07-31).** That landing's fixture planner implements multi-entry routes and shared allocations, but its fixture is single-entry, so both paths are exercised only as the empty case. The Metal proof's materialized route (two dispatches, one shared allocation) is the shape a second fixture stage needs; the serial-sum artifacts already carry it.

## Work

Extend the out-of-crate fixture (`crates/tiler-runtime/tests/adapter_route/`) with a two-stage member — two entries, one shared scratch allocation — and assert the stage log, the shared-allocation lifetime through final device use, and the post-commit failure classification when the second stage halts. Perturb the shared-allocation size and the inter-stage ordering, each watched failing.

## Closes when

Both paths are reached by a passing fixture case with perturbations, and the empty-case tests are kept beside them rather than replaced.
