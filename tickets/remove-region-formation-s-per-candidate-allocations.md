---
id: remove-region-formation-s-per-candidate-allocations
title: Remove region formation's per-candidate allocations
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [performance]
---

**This ticket has no body and must not be dispatched until it has one.** It was created with `tkt create` and never written; a sibling in the same state reached a worker on 2026-08-08 and the worker reported it immediately — with no Facts to audit, the coordinator's brief becomes the sole authority, which inverts the check the per-Fact audit exists to provide.

## What a worker needs before this is dispatchable

**The claim, stated and cited.** Which allocations, at which construction site, per what candidate — by searchable anchor in `crates/tiler-compiler/`, not by line number.

**Whether it is a correctness or a performance ticket.** AGENTS.md orders correctness, maintainability, then performance, and requires that performance work establish measurement validity first: workload, target, metric, baseline, warm-up, repetitions, noise controls, and oracle, with the dominant cost measured before the narrowest change is made. If this is performance work it needs all of that, and it needs to run on the idle M3 Pro rather than the coordination host.

**A baseline, or an explicit statement that none exists.** Region formation's budgets are a live subject — `region_members`, `region_boundary_outputs`, and `region_live_values` have each moved, and `region_candidates_per_seed` and `region_expansions` have not. A per-candidate allocation claim that predates those moves is stale until re-measured.

**Whether removing the allocations can change a plan.** If it can, this is not a performance ticket at all and the framing must change before anyone starts.

Until those are written, the honest state is that nobody knows what this ticket asks for.
