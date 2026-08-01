---
id: accept-the-cpu-vector-lane-tier-adr
title: Accept or reject the CPU vector-lane tier ADR
status: awaiting-decision
priority: p2
dependencies: [land-the-cpu-vector-lane-tier-adr]
related: [design-the-cpu-vector-lane-tier]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, governance, scheduling, cpu, simd]
---
**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, and it exists so that every ticket conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing the problem after being dispatched. Its permanent status is `awaiting-decision` — a parked state that `tkt ready` excludes and that never satisfies a dependent — until the decision is taken. **An agent that finds this ticket in `todo` should set it to `awaiting-decision` and do nothing else**; it was filed before its ADR had a number and could not be created in a parked state.

**The id is a placeholder.** [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) renames it to `accept-adr-NNNN-cpu-vector-lane-tier` once the number is fixed, which repoints the three dependents below.

## What Tom is deciding

Seven numbered items, drafted in the `## Drafted ADR body` section of [the CPU vector-lane tier](../docs/research/scheduling/cpu-vector-lane-tier.md). The three that are genuine choices rather than derivations, and are where a reader should push first:

- **Item 1**, that an order-preserving horizontal accumulate gets no schedule construct at all. The derivation is that it changes neither the map nor the contributor partition, so it is below the schedule boundary. The cost is that a backend's choice to use one is invisible to `EXPLAIN`, which a reader may think too high a price for the separation it buys.
- **Item 5**, that `ScalableVectorLane` may bind the map and not the contributor partition. This is a structural refusal that holds even under a contract permitting everything, and it means a scalable CPU backend cannot split a long reduction across lanes at all until a symbolic partition vocabulary exists. The deferral naming that vocabulary is recorded with its trigger.
- **Item 7**, that the CPU worker-thread scope stays out. The derivation is that lanes raise no cross-instance obligation and that threading needs a profile declaration rather than a new binding, but a reader who expects the CPU tier to arrive whole may want both in one design.

## What acceptance releases, and what it does not

Acceptance releases the three implementation tickets depending on this node. It registers nothing, declares no profile, and admits no threaded CPU backend. The public boundaries the two research records enumerate — the binding and tail-policy variants, the lane-partition topology, the padding-identity obligation, the kernel-IR lane and mask types, the masked memory operations, the vector realization subject, and the executing-unit component of the numerical honourability key — each still come to Tom at implementation time under ADR 0075.

## Closes when

Tom decides. On acceptance: the ADR's `decision_status` moves, both research records' `disposition` and `adopted_by` move with it, the catalog views are corrected, and any contract sentence whose truth depended on the proposed status is swept.
