---
id: declare-metal-subgroup-realization-facts-in-the-target-profile
title: Declare Metal subgroup realization facts as atomic target facts
status: todo
priority: p2
dependencies: [accept-the-subgroup-execution-tier-adr]
related: [design-the-subgroup-execution-tier, declare-cpu-vector-realization-facts-in-the-target-profile, correct-the-subgroup-threads-route-dimension-meaning]
scopes: [implementation/compiler, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, metal, subgroup, execution-hierarchy, feasibility, public-boundary]
---
## User-visible outcome

A target profile states what a Metal device's subgroups actually do — as atomic declared facts a feasibility predicate reads — so a subgroup schedule is admitted or refused against declared target properties instead of against an assumption compiled into the backend.

## Why now

**Fact — the acceptance node releases nothing today.** [`accept-the-subgroup-execution-tier-adr`](accept-the-subgroup-execution-tier-adr.md):15 and `:31` claim acceptance "releases the implementation tickets gated behind it"; [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md):65 lists four filed tickets and none is an implementation ticket. This is the target-profile third of what makes that claim true.

**Fact — the record enumerates a `SubgroupRealization` subject and its builder method among nine public-boundary items for Tom.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md):333 opens that enumeration; `design-the-subgroup-execution-tier.md:65` names the subject and the builder method explicitly. This ticket drafts them and accepts none.

**Fact — one route dimension in the landed vocabulary is already wrong for these routes, and its correction is independent.** `design-the-subgroup-execution-tier.md:65` records that `RouteResourceDimension::SubgroupThreads` is a floor over "threads one subgroup must execute in lockstep", and that lockstep within a subgroup is not a property current GPU families guarantee — the CUDA guide's independent-thread-scheduling text withdraws it explicitly. [`correct-the-subgroup-threads-route-dimension-meaning`](correct-the-subgroup-threads-route-dimension-meaning.md) owns that and is independent of whether this design is accepted; do not absorb it here, and do not declare a fact whose meaning that ticket is still fixing.

**Inference — declaring these as facts rather than as backend code is what ADR 0090 item 1 decided.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records the accepted split: profiles declare what a target can do, providers propose what to do with it, and the host performs every comparison. A subgroup width or a shuffle capability hardcoded in `tiler-metal` would put a target fact on the provider side of that split.

## Implementation keys

- The realization facts the record derives, each atomic and separately declared, so a feasibility predicate can name exactly which one refused. A bundled "supports subgroups" boolean is the shape to avoid: it cannot explain a refusal.
- Hard feasibility stays separate from estimated cost. A subgroup schedule a target cannot realize is rejected with an explainable reason, never priced at an infinite or arbitrary cost.
- Where a fact is not observable on the tested hosts, it is `Unknown` and refuses, not defaulted. Facts about a tested host stay distinct from portable guarantees.
- Mirror [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md)'s shape; the two tiers' profile declarations should read as one system.

## Required failure-path evidence

Each observed failing against an accepted neighbour: a subgroup schedule whose width the profile does not declare; a schedule requiring a shuffle the profile does not declare; a profile declaring a fact the target family cannot support; and an `Unknown` fact reaching feasibility and refusing rather than defaulting.

## Non-goals

Schedule bindings and kernel-IR constructs (their own tickets). Emission. The `RouteResourceDimension::SubgroupThreads` correction. Any measured subgroup performance claim — this ticket declares facts, it does not benchmark them.

## Closes when

The facts are declared and read by feasibility, every refusal above is checked by a check observed failing, no fact is defaulted where it is unobserved, and the profile subject and its builder method have gone to Tom rather than been self-accepted.
