---
id: remove-region-formation-s-per-candidate-allocations
title: Remove region formation's per-candidate allocations
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
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

## Current-state correction — 2026-08-09

The 2026-08-08 dispatch warning above audited the originally bodyless ticket as though it still represented pending work. It did not: commit `dfe909f4f7f7a1bc4b52d070f66c1d2065ea3b70` (`Carry region membership as sorted vectors and share identity bytes`) had already delivered the requested allocation-churn repair on 2026-07-27, and `6ca0e1cb9d1bb06c70a92a43e0268d7dad7851cf` then closed this ticket. The warning remains as a historical account of the filing defect, not as the ticket's current state or a reason to dispatch it again.

The landed implementation is source-verifiable at the following current anchors:

- `form_candidate` requires the candidate member slice to be ascending and distinct, which lets region formation preserve set semantics without constructing a `BTreeSet` for every candidate;
- `is_member` uses binary search over that bounded canonical slice, and `local_position` scans the immediately constructed bounded order instead of allocating a `BTreeMap` for each encoding;
- cover enumeration mutates one `covered` mask in place and undoes it on backtracking instead of cloning an uncovered set for every branch;
- `derive_materializations` uses sorted vectors rather than per-cover maps and sets; and
- `RegionCoverIdentity` and immutable region labels share their bytes behind `Arc` while retaining content-based equality, ordering, and the same `as_bytes()` result.

Those changes preserve the relevant observable semantics: canonical ordering and the ascending/distinct member invariant replace set canonicalization, verification recomputes and compares the same identity bytes, and cover search still explores the same disjoint candidates under the same budgets. This audit records the implementation and makes `status: done` truthful. It does **not** claim a new benchmark, a current allocation count, or a measured speedup beyond the implementation evidence recorded in the landing commit.
