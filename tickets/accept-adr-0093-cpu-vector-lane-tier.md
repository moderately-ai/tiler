---
id: accept-adr-0093-cpu-vector-lane-tier
title: Accept or reject the CPU vector-lane tier ADR
status: done
priority: p2
dependencies: [land-the-cpu-vector-lane-tier-adr]
related: [design-the-cpu-vector-lane-tier]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, governance, scheduling, cpu, simd]
---
**Decided 2026-08-01: Tom accepted.** [ADR 0093](../docs/decisions/0093-bind-vector-lanes-to-the-map-or-the-contributor-partition.md) landed `accepted` and this node is `done`. The rule below is preserved unedited because it governed this ticket for its whole life and governs every node like it; what changed is that the decision was taken, not that the rule relaxed.

> **Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. It is the graph node standing for a decision that has not been made, and it exists so that every ticket conditional on that decision is held out of the ready frontier by a dependency edge rather than by a worker noticing the problem after being dispatched. Its permanent status is `awaiting-decision` — a parked state that `tkt ready` excludes and that never satisfies a dependent — until the decision is taken. **An agent that finds this ticket in `todo` should set it to `awaiting-decision` and do nothing else**; it was filed before its ADR had a number and could not be created in a parked state.

**The id was a placeholder and is no longer.** It was `accept-the-cpu-vector-lane-tier-adr` until 2026-08-01, when [`land-the-cpu-vector-lane-tier-adr`](land-the-cpu-vector-lane-tier-adr.md) resolved the number and ran `tkt rename accept-the-cpu-vector-lane-tier-adr accept-adr-0093-cpu-vector-lane-tier`, which moved the file, rewrote the id, and repointed four references. Two prose links the rename did not reach — in the carrier and in [`design-the-cpu-vector-lane-tier`](design-the-cpu-vector-lane-tier.md) — were repaired by hand in the same change; `tkt rename` repoints frontmatter and not Markdown link targets, which is worth knowing before the next rename.

## The decision, and how to refute this record in one line if the relay was wrong

**Fact — what is recorded.** Tom accepted ADR 0093 at the live review on 2026-08-01, in the same session that accepted [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md). **What he accepted is the model — the ADR's seven numbered decisions — and none of the public-boundary items** the two research records enumerate; those are unchanged and still arrive under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) with the implementation tickets below.

**The provenance of that acceptance, named rather than dressed up.** It reached this branch through the coordinator's dispatch brief, which is the ordinary channel — every "Tom accepted" record in this repository was written by an agent who received the relay rather than by Tom — and it is corroborated by the tree only circumstantially: the base commit `50409b9` is "Record the approved decisions; silence stays deliberate and both parks carry their triggers", and [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) records the same morning review for ADR 0091. **If the relay was wrong the repair is bounded and mechanical:** `decision_status` back to `proposed` in ADR 0093, its two catalog rows back to `proposed`, both research records' `disposition` and `adopted_by` back to `pending`, this node back to `awaiting-decision`, and the two implementation tickets back to `blocked`. Nothing beyond that was released, because the acceptance covers the model and no public spelling — no crate, test, fixture, or spike was touched, and no contract sentence outside the two research records was rewritten under it.

## What Tom is deciding

Seven numbered items, drafted in the `## Drafted ADR body` section of [the CPU vector-lane tier](../docs/research/scheduling/cpu-vector-lane-tier.md). The three that are genuine choices rather than derivations, and are where a reader should push first:

- **Item 1**, that an order-preserving horizontal accumulate gets no schedule construct at all. The derivation is that it changes neither the map nor the contributor partition, so it is below the schedule boundary. The cost is that a backend's choice to use one is invisible to `EXPLAIN`, which a reader may think too high a price for the separation it buys.
- **Item 5**, that `ScalableVectorLane` may bind the map and not the contributor partition. This is a structural refusal that holds even under a contract permitting everything, and it means a scalable CPU backend cannot split a long reduction across lanes at all until a symbolic partition vocabulary exists. The deferral naming that vocabulary is recorded with its trigger.
- **Item 7**, that the CPU worker-thread scope stays out. The derivation is that lanes raise no cross-instance obligation and that threading needs a profile declaration rather than a new binding, but a reader who expects the CPU tier to arrive whole may want both in one design.

## What acceptance releases, and what it does not

~~Acceptance releases the three implementation tickets depending on this node.~~ **Corrected on execution: two, not three.** [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) and [`declare-cpu-vector-realization-facts-in-the-target-profile`](declare-cpu-vector-realization-facts-in-the-target-profile.md) depend on this node directly and moved `blocked` → `todo`. [`admit-lane-typed-values-and-masked-memory-into-the-kernel-ir`](admit-lane-typed-values-and-masked-memory-into-the-kernel-ir.md) depends on the *first of those* and not on this node, so it stays `blocked` and is released by that ticket's completion rather than by this decision. The distinction matters because `blocked` is a parked state `tkt ready` excludes on its own: a dependency being satisfied does not un-park a ticket, so the two releases were status edits and the third would have been wrong to make.

It registers nothing, declares no profile, and admits no threaded CPU backend. The public boundaries the two research records enumerate — the binding and tail-policy variants, the lane-partition topology, the padding-identity obligation, the kernel-IR lane and mask types, the masked memory operations, the vector realization subject, and the executing-unit component of the numerical honourability key — each still come to Tom at implementation time under ADR 0075.

## Closes when

~~Tom decides.~~ **Closed 2026-08-01.** The ADR's `decision_status` reads `accepted`; both research records read `disposition: adopted` with `adopted_by: ["ADR-0093"]`; both catalog views carry the decision as accepted and both research records gained the rows they had never had; the proposal-era disclosures in both records were swept and the retained drafted span was demoted to provenance; and the two directly dependent tickets were released. No contract sentence outside the two research records depended on the proposed status — `docs/backends/cpu.md` was checked and stays `proposed` deliberately, because decision 7 answers its vector half and none of its threading or cache half, and moving it is a public-boundary item rather than a sweep.
