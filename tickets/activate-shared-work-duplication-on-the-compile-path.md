---
id: activate-shared-work-duplication-on-the-compile-path
title: Activate shared-work duplication on the compile path
status: deferred
priority: p2
dependencies: []
related: [implement-general-dag-partitioning, implement-boundary-property-enforcers, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, partitioning, deferred]
---
## User-visible outcome

The compile path enumerates covers under `CoverPolicy::permitting_shared_work_duplication`, so a program whose shared work is cheaper to recompute than to materialize can be planned that way.

## Why this is deferred rather than todo

`implement-general-dag-partitioning` implemented the legality contract, the search, the cost model, and the explanation for shared-work duplication, and left the compile path enumerating under `CoverPolicy::governed` — the exact-partition contract. That is a physical-provider and program-assembly limit, not a legality one, and it was derived rather than assumed:

**Fact — a duplicating cover assigns one occurrence to several region subjects.** A plan over it needs one admitted physical implementation per subject.

**Fact — the bounded physical provider proposes for exactly three member sets.** `GovernedPhysicalProvider::propose` recognizes the pointwise prologue, the reduction, and the whole program, and returns an empty offer for any other member set (`crates/tiler-compiler/src/frontier.rs`). Every region a duplicating cover introduces is one of those "any other" sets.

**Fact — program assembly implements exactly three plan shapes.** `build_plan_program` matches a one-region fused program, a two-region materialized program, and the three-region split, and rejects anything else as `unsupported-plan-shape` (`crates/tiler-compiler/src/pipeline/planning.rs`).

**Inference.** Enabling the policy today would enumerate every duplicating cover, find each region unimplementable, record a `RegionUnimplemented` rejection, and retain no additional plan — paying the whole search to report a refusal, and inflating the explain trace with it.

## Activation triggers

Any one of these makes this startable; the first two are what actually change the answer.

1. A physical provider proposes implementations for member sets beyond the three recognized ones — the general-region proposer, or a caller-supplied provider through `drive-an-external-physical-implementation-provider-through-compilation`.
2. `build_plan_program` assembles a kernel program from an arbitrary cover shape rather than three enumerated ones.
3. A frontend admits a program whose fan-out producer is cheap enough that recomputation beats materialization on a *measured* device cost, not only on the partition-structural estimate.

## What is already done and must not be re-done

- The legality condition (purity, no named result, no contract-granted realization freedom) with typed refusals: `crates/tiler-compiler/src/cover.rs`.
- The materialize-versus-recompute per-edge enumeration, the `tiler.cost.partition-structural.v1` estimate, and its dominance view.
- The exhaustive small-graph oracle agreement under both contracts.
- The explain channel naming refused candidates and dominated covers separately.

Turning this on is a one-line change at the single call site in `enumerate_complete_plans`; everything above it is what has to arrive first.

## Graph maintenance

- When this lands, `component_cost`'s `RedundantWork` arm stops reporting `Exact(0)` — check the value moves, as its note asks.
- A duplicating plan makes one region's guarantee differ from another's requirement, which is the first case `implement-boundary-property-enforcers` restarts on. Re-read that ticket's restart condition rather than assuming this fires it.

## Trigger check log

- 2026-08-04 — **not fired.** The compile path still enumerates under `CoverPolicy::governed` (`crates/tiler-compiler/src/pipeline/planning.rs:77`, `crates/tiler-compiler/src/pipeline/verify.rs:58`), and `build_plan_program` still matches exactly three shapes and rejects any other as `unsupported-plan-shape` (`crates/tiler-compiler/src/pipeline/planning.rs:897-913`), so triggers 1 and 2 are both unmet; `drive-an-external-physical-implementation-provider-through-compilation` is `todo`. The general DAG partition search that landed 2026-08-04 widened the *cover search*, not the provider or the assembly, so it does not fire this. Recheck: `grep -n 'unsupported-plan-shape' crates/tiler-compiler/src/pipeline/planning.rs && grep -rn 'CoverPolicy::governed' crates/tiler-compiler/src/pipeline/`.
