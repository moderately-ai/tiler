---
id: decide-the-fixed-strided-contributor-membership-vocabulary
title: Decide the fixed strided contributor-membership vocabulary
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-reassociated-contraction-schedule-alternatives]
scopes: [implementation/ir, implementation/compiler, implementation/metal, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, contraction, numerics, identity, public-boundary, decision, needs-tom]
---
## User-visible outcome

Tom accepts or revises the exact public physical-schedule carrier by which a plan distinguishes a fixed contiguous contributor partition from a fixed lane-strided contributor partition, including the append-only identity rule and the included and excluded surface. Until that decision, no implementation may encode the two trees as one schedule or reinterpret an existing field.

## Why a decision is required

**Fact — the demand is two exact, observed trees.** [`admit-reassociated-contraction-schedule-alternatives`](admit-reassociated-contraction-schedule-alternatives.md) derives the retained corpus at exact base `b30e384497682c91771fcf93c5ce6854054d39a3`: the contiguous candidate folds consecutive intervals and merges them in ascending lane order, while the strided candidate folds `lane, lane + split, ...` and uses the same ascending merge. They return different bits on `split_topology`, so contributor membership is correctness-bearing plan identity.

**Fact — the current carrier states only the first tree.** Public `ContributorPartition` fixes contiguous membership. `ContributorArrival` states how staged partials reach the combiner, not which original contributors each partial contains: its ascending value consumes no permutation, while its two permutation-bearing values are nondeterministic/atomic constructs that the verifier refuses even when permitted. The canonical schedule encoder therefore has no field whose present meaning can distinguish the retained fixed strided tree.

**Inference — a private compiler spelling is insufficient.** [ADR 0012](../docs/decisions/0012-physical-reduction-topology.md) requires the actual partitioning and tree shape to be recorded in physical-plan and artifact identity. A private switch below the scheduled region would make two computations share one public schedule statement and one canonical identity. Reusing an existing value would instead change its documented meaning and the meaning of retained bytes.

## Exact decision requested from Tom

Choose or revise the public carrier that names fixed contributor membership independently from staged-partial arrival, and decide its compatibility strategy:

- whether membership belongs on the existing partition record, on the contraction split topology, or on another explicitly named physical carrier;
- which current topology or topologies may carry it, including whether the two-read contraction has a distinct split form;
- whether old schedule rows retain byte-for-byte identity through a fresh tag or require a schedule-domain revision because a field lands inside an already retained payload;
- which total maps must break on vocabulary growth: schedule verification, kernel lowering and verification, witness projection, identity encoding, explanation, compiler proposal construction, and Metal emission; and
- the exact included and excluded public surface under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), with acceptance provenance.

This ticket records the question and evidence. It deliberately does not draft, select, or self-accept a Rust shape.

## Included surface to decide

- Fixed contiguous versus fixed lane-strided membership over one canonical contributor sequence.
- Exact positive-multiple coverage with every partition nonempty; no padding and no ragged tail.
- Independent reassociation and permutation consumption, with a typed refusal naming the missing dimension.
- Root-only seed semantics and the proof condition under which a constantly true partial `has_value` may be erased.
- Canonical schedule/artifact identity, append-only preservation of every old row where possible, and the required type-sized encoder census.
- Target-neutral verification/lowering plus Metal consumption of the accepted statement.

## Excluded surface

- Distributivity, contraction-chain regrouping, accumulator-width choice, or a new numerical-contract dimension.
- Nondeterministic arrival, atomics, collectives, subgroup semantics, or timing-dependent trees.
- Ragged partitions, padding, masks, runtime split widths, source offsets, backend-specific semantic vocabulary, scheduling policy, or performance-transfer claims from the spike.
- A public access-relation widening used merely to hide a physical partition that the schedule itself does not name.

## Strongest alternatives and counterpoints

1. **Widen the existing partition carrier with a typed membership distinction.** Strongest point: membership is exactly what the partition describes, and both multi-pass and cooperative consumers already read that authority. Strongest counterpoint: the carrier is public and embedded inside retained topology payloads, so the placement may move old bytes or require a domain revision; it also broadens every split family when the measured demand is a contraction.
2. **Admit a contraction-specific split topology.** Strongest point: a fresh topology tag can preserve every old topology byte and makes the two-read contraction's product fold explicit. Strongest counterpoint: it risks duplicating the cooperative staging and partition contracts and creating two authorities for the same cross-invocation dataflow.
3. **Add a separate membership carrier shared by existing split topologies.** Strongest point: contributor membership and staged arrival remain cleanly orthogonal, matching the numerical derivation. Strongest counterpoint: every existing topology and consumer must decide whether and how it carries the new field, increasing the affected population and identity migration surface.
4. **Reuse `ContributorArrival` or keep the distinction private.** Strongest point: no new public spelling. Decisive counterpoint: arrival is not membership, the retained strided merge is fixed rather than nondeterministic or atomic, and a private distinction would collide in schedule identity. This alternative is not correct without redefining accepted bytes and contracts.

## Closes when

Tom's dated decision names the accepted carrier and compatibility rule, records the exact included and excluded surface and acceptance venue, and the dependent implementation ticket is updated to cite that authority. Only Tom moves this ticket from `awaiting-decision`.
