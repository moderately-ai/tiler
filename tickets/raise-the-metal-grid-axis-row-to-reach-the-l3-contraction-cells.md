---
id: raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells
title: Raise the Metal grid-axis row so the L3 contraction cells are reachable
status: in-progress
priority: p2
dependencies: [establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
related: [integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/build, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, contraction, target-profiles]
claimed_from: todo
assignee: agent-grid-axis
lease_expires_at: 1785934520
---
## User-visible outcome

The L3 profile's own contraction cells become reachable through the accepted AOT and runtime route, so the spike's retained `result_sha256` values at those cells can be taken as the independent cross-check they were retained to be.

## Why they are not reachable now

**Fact, measured 2026-08-02.** [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) ran one contraction of the profile's index structure `td,od->to` end to end through the accepted route and bit-compared it against the reference at `2x2x3`. It could not run the profile's own cells: the `direct` realization launches one invocation per output element, every correctness cell publishes at least 1,024 of them (`w_decode_kv` is `M=1, N=1024`), and the declared profile's `grid_axis_threads` row is `4`.

Compiling `w_decode_kv` against `BoundMetalCompileDeclaration::first_macos_apple9` resolves rule `target.grid-axis`, predicate `grid-axis`, `Rejected("target-infeasible")`, `required: Threads(1024), available: Threads(4)`, and the target outcome is `NoFeasiblePlan` before any plan composes. `2x3x3` refuses identically at `required: Threads(6)`, which is what fixes the bound at exactly four rather than at "large".

**Fact — the row is a compile guarantee, not a measured maximum.** `crates/tiler-build/src/metal_declaration.rs`'s `FIRST_MACOS_APPLE9` records it as "the macOS 26.5 SDK's `dispatchThreads:` contract proves extent 4 is representable and establishes no upper bound at all, so 4 is a deliberately conservative compile guarantee rather than a maximum." The spike's own `environment.tsv` reports `device_max_threads_per_threadgroup 1024` on the same Apple M4 Max, and the spike dispatched all six cells there — so the hardware is not the constraint and the ledger row is.

## What this ticket owes

- A named authority for a grid-axis extent above four, in the compile-profile authority ledger's own terms: an SDK sentence, a feature-table row, or a retained measurement with its exact procedure. A number chosen because it makes a cell compile is exactly what the ledger exists to refuse.
- The declaration row moved with that authority, and the descriptor identity recomputed — the profile key and canonical descriptor fold this row, so moving it moves every pinned identity derived from it. That is an identity-domain step: executed completely or not at all.
- Re-run of the contraction vertical at one L3 cell, with the retained `result_sha256` compared on a matching host row. This host already matches the spike's correctness row on every recorded field.

## Non-goals

The tiled realization, which [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) owns. Raising the row is about reaching the cells at all; which realization is fastest there is a separate question with its own measurements.

## Closes when

One L3 correctness cell runs through the accepted AOT and runtime route and its result matches the spike's retained `result_sha256` for that cell, or the ledger records why no authority admits an extent that reaches it.
