---
id: admit-vector-lane-bindings-into-the-schedule-vocabulary
title: Admit vector-lane bindings and their tail policies into the schedule vocabulary
status: awaiting-decision
priority: p2
dependencies: [accept-adr-0093-cpu-vector-lane-tier, admit-shared-contributor-coverage-and-reduction-padding-identity]
related: [design-the-cpu-vector-lane-tier, represent-cooperative-workgroup-reduction-dataflow, declare-cpu-vector-realization-facts-in-the-target-profile, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir]
scopes: [implementation/ir, implementation/compiler, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, ir, cpu, simd, execution-hierarchy, public-boundary, decision, needs-tom]
---
## User-visible outcome

A scheduled region can state that its work is spread across vector lanes, and the intrinsic verifier discharges — or refuses, by name — every obligation that spread creates: coverage, ownership, bounds under the chosen tail policy, and the numerical permissions a lane partition consumes.

## Why now

**Fact.** `ExecutionBinding` has one variant, `GlobalLinearInvocation`; `TailPolicy` has one variant, `Exact`; `ReductionTopology` has five variants and none is a vector topology. Nothing in the implemented vocabulary represents a vector lane. All three enums are `#[non_exhaustive]` under ADR 0074 convention 5a, so each widening is additive by construction.

The design and its eliminations are [the CPU vector-lane tier](../docs/research/scheduling/cpu-vector-lane-tier.md); this ticket implements it and should not re-derive it.

## Implementation keys

- **`ExecutionBinding::FixedVectorLane { lanes }` and `ExecutionBinding::ScalableVectorLane`.** The width is a literal on the fixed variant, for the identity and intrinsic-verification reasons the record derives; the scalable variant carries no width at all.
- **`TailPolicy` gains `Predicated`, `ScalarEpilogue`, and `IdentityPadded { .. }`.** Each derives a *different* target requirement, and the verifier must not treat them as interchangeable. `ScalableVectorLane` admits `Predicated` alone; `Exact` and `ScalarEpilogue` are refused for it because neither `N mod W` nor an epilogue trip count is a compile-time quantity.
- **A lane-partition reduction topology** carrying a `ContributorPartition`, a layout (contiguous or strided), and an accumulation `ArithmeticType`. Contiguous consumes `permits_reassociation`; strided consumes `permits_permutation` as well. Both are *required* rather than recorded, at the same place and by the same shape of check `verify_cooperative_semantics` and the multi-pass gate already use: topology flags must agree with the region's numerical realization, and order-sensitive families additionally require `family.consumes_reassociation && !*permits_reassociation` (the extrema family spends nothing and is the stated exception). It is refused outright for `ScalableVectorLane`, because `ContributorPartition::covers` is a product over a symbolic width.
- **The padding identity is a stated field and is never derived from `empty_identity_bits`.** The verifier currently requires `empty_identity_bits == 0.0_f32.to_bits()`, that is `+0.0`, and `+0.0` is not a two-sided identity of `f32` addition — `(-0.0) + (+0.0) = +0.0`. `-0.0` is. A padded fold reusing the empty identity is wrong on exactly the rows whose true sum is `-0.0`.
- **Identity encoding.** Every new variant takes an appended tag byte and every existing tag and field position stays put, so no previously encodable region's bytes move and the schedule identity domain does not step. `push_schedule` currently destructures `ExecutionBinding` and `TailPolicy` with irrefutable `let` bindings and pushes a constant `0x01` for each; both become matches, which is the build error that proves the widening reached the encoder.
- **`ResourceRequirements`** gains whatever the target-facing derivation needs; see the sibling profile ticket, which owns the subject type.

## Required failure-path evidence

Each of these must be run against a case that must fail and observed failing, against an accepted neighbour: a lane binding whose predicate leaves a coordinate uncovered; two lanes owning one output; a `Predicated` tail on a region whose bounds proof does not admit the overrun address; a lane partition under a reassociation-forbidding contract; a strided lane partition under a permutation-forbidding contract; a lane partition whose `covers` fails; a lane partition on a scalable binding; an `IdentityPadded` policy whose padding value is `+0.0` under a signed-zero-forbidding contract.

## Non-goals

Kernel-IR constructs (its own ticket). Target profile declarations (its own ticket). Emission of any kind. Any threading construct. Any performance claim.

## Decision packet — 2026-08-09

ADR 0093 accepted the vector-tier model, not the exact public Rust spellings listed here. Tom must accept the `ExecutionBinding`, `TailPolicy`, lane-partition topology, and padding-identity surface as one boundary. Recommendation: accept the record-derived shape, including distinct fixed/scalable bindings and the stated padding identity; merging or defaulting those concepts would erase obligations the verifier must name.

## Closes when

The vocabulary is admitted; every obligation in Required failure-path evidence is checked by a check observed failing; the identity encoding is exhaustive at every site (encoder matches fail to compile if a new variant is omitted); and the design record's *schedule-owned* intrinsic verdicts are constructible as tests: map under a strict contract consumes no permission; A3 double refusal (reassociation + covers); B2 padding identity and strided permutation refuse; B3 scalable partition refuse. Target-feasibility forks in the worked examples (A1 Proven on AVX / Rejected on NEON; A2 admissible on NEON) belong to the profile ticket's Realized/Unrealizable composition and are not required here — jointly closable only after both land.

**Correction — 2026-08-10.** Reassociation admission wording and Closes when were over-narrow / over-broad relative to live `verify_cooperative_semantics` / multi-pass gates and Non-goals; Implementation keys and Closes when above carry the repaired shape.

**Decision correction — 2026-08-11.** `TailPolicy::IdentityPadded` conflates two independent axes and is withdrawn from the contributor-padding design. `TailPolicy` governs iteration-domain launch tails; padding a lane-partition reduction extends its contributor sequence while every executing lane remains active. `Predicated` and `ScalarEpilogue` remain iteration-tail candidates. The lane-partition topology instead consumes the shared `ContributorCoverage` and typed `ReductionPaddingIdentity` owned by `admit-shared-contributor-coverage-and-reduction-padding-identity`. Existing `ContributorPartition::covers` remains exact, and no missing identity defaults to `empty_identity_bits`.
