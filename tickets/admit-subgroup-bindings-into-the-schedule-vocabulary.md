---
id: admit-subgroup-bindings-into-the-schedule-vocabulary
title: Admit subgroup bindings and their reduction topology into the schedule vocabulary
status: awaiting-decision
priority: p2
dependencies: [accept-adr-0094-subgroup-execution-tier]
related: [design-the-subgroup-execution-tier, admit-vector-lane-bindings-into-the-schedule-vocabulary, compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, ir, metal, subgroup, execution-hierarchy, public-boundary, decision, needs-tom]
---
## User-visible outcome

A scheduled region can state that its work is spread across the lanes of a subgroup, and the intrinsic verifier discharges — or refuses, by name — every obligation that spread creates: coverage, ownership, the combine order the reduction consumes, and the numerical permissions a lane partition spends.

## Why now

**Fact — the acceptance node claims to release implementation tickets, and there are none to release.** [`accept-adr-0094-subgroup-execution-tier`](accept-adr-0094-subgroup-execution-tier.md):15 says acceptance "is what releases the implementation tickets gated behind it" and `:31` repeats that it "releases the implementation tickets". [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md):65 lists the four tickets that design filed — the ADR carrier, the acceptance node, the two-level composition, and the route-dimension correction — and none is an implementation ticket. This ticket and its two siblings are what make that claim true.

**Resolved 2026-08-01 — the node closed and this ticket is what it released.** [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) landed `accepted` and the acceptance node is `done` under its final id, which is why the link above no longer reads `accept-the-subgroup-execution-tier-adr`. The paragraph is preserved rather than rewritten because it is the reason this ticket exists; what changed is that the claim it was filed to make true is now true.

**Fact — nine public-boundary items are enumerated for Tom and none is self-accepted.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md):334 opens "Public-boundary items, enumerated for Tom and not self-accepted" — the line moved from `:333` when ADR 0094's acceptance sweep added the model-versus-surfaces sentence under that heading, and the heading's own claim is unchanged; `design-the-subgroup-execution-tier.md:65` summarizes them as including "the `ReductionTopology` variant, the `CombineTree` vocabulary, `lane_identity_bits` and its proof obligation, a subgroup-lane `LocalCoordinateSource`, a `SubgroupRealization` subject and its builder method, and the `RouteResourceDimension` change". This ticket owns the schedule-side subset and drafts them; it accepts none.

**Inference — this trio mirrors the CPU vector-lane trio deliberately.** The subgroup record at `:396` states that the lane identity "becomes the second construct in the vocabulary needing a proved reduction identity, and it should land as one concept with the CPU tier's padding identity rather than as two". [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) is the shape to match — including its status idiom, `todo` gated on its ADR acceptance ticket rather than parked, so acceptance releases it structurally.

## Implementation keys

- The subgroup execution binding and the reduction topology the record derives, with the combine order stated rather than assumed. The record's central negative result is load-bearing here: neither Metal nor WGSL states the combine order of a subgroup reduction collective, so a topology that leaves the order implicit cannot be admitted under an order-sensitive contract.
- The lane identity and its proof obligation land as **one** concept with the CPU tier's padding identity, per `subgroup-execution-tier.md:391` — not as a second, parallel spelling. Read [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) against this ticket before choosing a shape; two parallel identity lists would be exactly the duplication AGENTS.md warns is intentional-until-proven-otherwise.
- Identity encoding is additive: every new variant takes an appended tag byte, every existing tag and field position stays put, no previously encodable region's bytes move, and the schedule identity domain does not step. The encoder's irrefutable `let` destructuring becoming a match is the build error that proves the widening reached it.

## Required failure-path evidence

Each observed failing against an accepted neighbour: a subgroup partition whose predicate leaves a coordinate uncovered; two lanes owning one output; a lane partition under a reassociation-forbidding contract; a lane partition whose coverage proof fails; and a reduction topology whose combine order is unstated under an order-sensitive contract.

## Non-goals

Kernel-IR constructs (`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`). Target profile declarations (`declare-metal-subgroup-realization-facts-in-the-target-profile`). Emission of any kind. The two-level subgroup-to-workgroup composition, which the ADR explicitly excludes and [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) owns. Any performance claim.

## Decision packet — 2026-08-09

ADR 0094 accepted the model, but the research record explicitly left the exact schedule-side public items for Tom. This ticket is awaiting acceptance of the listed `ExecutionBinding`, reduction-topology, `CombineTree`, lane-identity, and coordinate-source surface as one coherent boundary. Recommendation: accept the record-derived surface without adding a second subgroup-specific identity vocabulary; its exclusions above remain binding.

## Closes when

The vocabulary is admitted, every obligation above is checked by a check observed failing, the identity encoding is exhaustive at every site, the record's worked examples are constructible as tests with the verdicts it states, and every public shape has gone to Tom rather than been self-accepted.
