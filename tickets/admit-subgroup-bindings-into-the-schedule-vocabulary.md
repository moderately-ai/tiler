---
id: admit-subgroup-bindings-into-the-schedule-vocabulary
title: Admit subgroup bindings and their reduction topology into the schedule vocabulary
status: blocked
priority: p2
dependencies: [accept-adr-0094-subgroup-execution-tier, admit-shared-contributor-coverage-and-reduction-padding-identity, decide-the-subgroup-coordinate-binding-and-output-map]
related: [design-the-subgroup-execution-tier, admit-vector-lane-bindings-into-the-schedule-vocabulary, compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, ir, metal, subgroup, execution-hierarchy, public-boundary, decision, needs-tom]
---
## User-visible outcome

A scheduled region can state that its work is spread across the lanes of a subgroup, and the intrinsic verifier discharges — or refuses, by name — every obligation that spread creates: coverage, ownership, the combine order the reduction consumes, and the numerical permissions a lane partition spends.

## Why now

**Fact — the acceptance node claims to release implementation tickets, and there were none to release when it was written.** [`accept-adr-0094-subgroup-execution-tier`](accept-adr-0094-subgroup-execution-tier.md), anchors `is what releases the implementation tickets gated behind it` and `releases the implementation tickets`, makes that claim. [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), anchor `Nine public-boundary items are enumerated`, lists the design-era work; this ticket and its two siblings supply the implementation population the acceptance wording expected.

**Resolved 2026-08-01 — the node closed and this ticket is what it released.** [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) landed `accepted` and the acceptance node is `done` under its final id, which is why the link above no longer reads `accept-the-subgroup-execution-tier-adr`. The paragraph is preserved rather than rewritten because it is the reason this ticket exists; what changed is that the claim it was filed to make true is now true.

**Fact — nine public-boundary items are enumerated for Tom and none is self-accepted.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md), heading `Public-boundary items, enumerated for Tom and not self-accepted`, opens that enumeration; [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md), anchor `the \`ReductionTopology\` variant`, summarizes the population. This ticket owns the schedule-side subset and drafts them; it accepts none.

**Inference — this trio mirrors the CPU vector-lane trio deliberately.** The subgroup record, anchor `becomes the second construct in the vocabulary needing a proved reduction identity`, states that it should land as one concept with the CPU tier's padding identity rather than as two. [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) is the shape to match.

## Implementation keys

- **`ReductionTopology::SubgroupTree` and its stated combine tree**, not a new `ExecutionBinding`. Research §1 sketches `SubgroupTree { partition, width, combine, lane_identity_bits, result_lane, … }` as a `ReductionTopology` sibling of `CooperativeWorkgroup`; research §5 eliminates a subgroup map binding (`ExecutionBinding::GlobalLinearInvocation` covers map work with no new binding). Combine order is stated on the tree (ascending admitted; descending statable and refused under a permutation-forbidding contract) rather than assumed. The record's central negative result is load-bearing here: neither Metal nor WebGPU states the combine order of a subgroup reduction collective (MSL and WGSL are the cited languages), so a topology that leaves the order implicit cannot be admitted under an order-sensitive contract.
- The lane identity and its proof obligation land as **one** concept with the CPU tier's padding identity, per the subgroup record anchor `becomes the second construct in the vocabulary needing a proved reduction identity` — not as a second, parallel spelling. Read [`admit-vector-lane-bindings-into-the-schedule-vocabulary`](admit-vector-lane-bindings-into-the-schedule-vocabulary.md) against this ticket before choosing a shape; two parallel identity lists would be exactly the duplication AGENTS.md warns is intentional-until-proven-otherwise.
- Identity encoding is additive: every new variant takes an appended tag byte, every existing tag and field position stays put, no previously encodable region's bytes move, and the schedule identity domain does not step. `ReductionTopology` and `LocalCoordinateSource` already match exhaustively in `push_schedule` / `tag`; the proof of *this* widening is a new exhaustive arm plus an appended tag (next free reduction tag after `TAG_REDUCTION_COOPERATIVE_WORKGROUP` / `0x35`). An irrefutable `let` on `ExecutionBinding` or `TailPolicy` becoming a match would prove a binding/tail widen — but this ticket does not admit either under research §5.

## Required failure-path evidence

Each observed failing against an accepted neighbour: a subgroup partition whose predicate leaves a coordinate uncovered; wrong `result_lane` / multi-writer commit ownership (only the committing lane stores; the other W−1 lanes work without writing); a lane partition under a reassociation-forbidding contract; a lane partition whose coverage proof fails; a reduction topology whose combine order is unstated under an order-sensitive contract; a descending-stride combine tree under a permutation-forbidding contract; `lane_identity_bits` of `+0.0` under a signed-zero-forbidding contract; and a threadgroup size that is not an exact multiple of the subgroup width (ADR 0094 decision 3).

## Non-goals

Kernel-IR constructs (`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`). Target profile declarations (`declare-metal-subgroup-realization-facts-in-the-target-profile`). Emission of any kind. The two-level subgroup-to-workgroup composition, which the ADR explicitly excludes and [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) owns. Any performance claim.

## Decision packet — 2026-08-09

ADR 0094 accepted the model, but the research record explicitly left the exact schedule-side public items for Tom. This ticket is awaiting acceptance of the schedule-side surface as one coherent boundary: `ReductionTopology::SubgroupTree` (exact field set and whether `width` is a newtype), `CombineTree` (stated stride order; ascending admitted, descending refused), `lane_identity_bits` plus its two-sided-identity proof obligation (one concept with the CPU padding identity), and a subgroup-lane `LocalCoordinateSource` (no defined relation to `LocalLinearInvocation`). Public-boundary items 1–4 of the research enumeration; this ticket accepts none itself.

**Correction — 2026-08-10.** An earlier draft of this packet listed `ExecutionBinding` among the surfaces Tom must accept here. Research §5 and ADR 0094 decision 1 eliminate a new subgroup execution binding for map work; the public-boundary enumeration does not name any `ExecutionBinding` variant. The schedule-side subset is topology / combine-tree / lane-identity / coordinate-source only. Recommendation: accept that record-derived surface without adding a second subgroup-specific identity vocabulary and without minting `ExecutionBinding::Subgroup*`; Non-goals exclusions above remain binding.

## Closes when

The vocabulary is admitted, every obligation above is checked by a check observed failing, the identity encoding is exhaustive at every site, the record's worked examples are constructible as tests with the verdicts it states, and every public shape has gone to Tom rather than been self-accepted.

## Source-first decision correction — 2026-08-11

The packet above was not constructible as written. `ContributorPartition::covers` means exact real-contributor coverage, so `{ partitions: 32, contributors_per_partition: 4 }` covers 128 and cannot cover the worked example's 101 real contributors. A `lane_identity_bits` field beside that partition does not change the relation. Reinterpreting the existing type would silently change `MultiPass` and `CooperativeWorkgroup`.

`TailPolicy::IdentityPadded` is also the wrong carrier: `KernelSchedule.tail` governs launch/iteration-domain coverage, while every subgroup lane remains active and padding extends the reduction's inner contributor sequence. The shared concept is a typed reduction padding identity inside a contributor-coverage sum, not the schedule's singular launch-tail policy.

The proposed free-standing `LocalCoordinateSource::SubgroupLane` was stale and dead. The enum already has two variants, and its only carrier is `LocalCoordinates` inside `CooperativeTile`; this topology carries no tile. When a workgroup contains several subgroups, a lane alone is additionally insufficient to bind each subgroup to an output, and no portable relation reconstructs subgroup index from the local linear invocation.

## Accepted portion — 2026-08-11

Tom accepted in this conversation the shared, strict contributor-coverage direction:

- Keep `ContributorPartition::covers` exact for every existing topology.
- Add a required tagged `ContributorCoverage`: exact coverage carries no identity; identity-padded coverage carries a typed `ReductionPaddingIdentity`. There is no `Option`, default, empty-identity substitution, or caller-declared proof.
- Derive the padding count as checked `partition.total_contributors() - real_contributors`; do not encode it twice. Padding is a canonical suffix and intrinsic verification proves the stated value is a two-sided identity for the actual family and numerical contract.
- Use validated literal subgroup-width and lane values plus a required closed, allocation-free combine-tree vocabulary. Butterfly steps derive from width. Ascending is the admitted form; descending remains statable and explicitly unadmitted even after permission checks.
- Do not weaken `TailPolicy`, infer any tree/identity in lowering, or fall back after target/preflight refusal.
- Append the new topology under tag `0x36` with a separately tagged coverage payload, preserving every existing schedule byte and the current schedule identity domain.

The exact subgroup-coordinate/output-map carrier was not part of the presented acceptance and a later source audit proved it cannot be omitted for the full `threads_per_workgroup % width == 0` model. `decide-the-subgroup-coordinate-binding-and-output-map` owns that remaining public decision. This ticket is blocked rather than falsely marked accepted or implementation-ready.

**Decision resolved — 2026-08-11.** Tom accepted the required direct subgroup coordinate/output binding: workgroup ordinal, subgroup index within the workgroup, and lane index within the subgroup are explicit topology identity and governed KIR sources. Output ordinal and the sole writer derive from that binding and `result_lane`; no relation is inferred from `LocalLinearInvocation`. This ticket remains blocked only on the shared contributor-coverage implementation.
