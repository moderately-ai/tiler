---
id: admit-shared-contributor-coverage-and-reduction-padding-identity
title: Admit shared contributor coverage and typed reduction padding identity
status: in-progress
priority: p1
dependencies: [accept-adr-0093-cpu-vector-lane-tier, accept-adr-0094-subgroup-execution-tier, accept-adr-0100-multi-round-reduction-composition]
related: [admit-subgroup-bindings-into-the-schedule-vocabulary, admit-vector-lane-bindings-into-the-schedule-vocabulary]
scopes: [implementation/ir, implementation/compiler, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, reductions, numerics, padding, identity, public-boundary, fail-closed]
claimed_from: todo
assignee: worker-shared-contributor-coverage
lease_expires_at: 1786650560
---
## User-visible outcome

A reduction schedule states whether its contributor sequence is covered exactly or extended by proved identity values. Missing, irrelevant, ill-typed, or non-neutral padding is refused; the empty-domain identity is never substituted.

## Accepted boundary — 2026-08-11

Tom accepted this boundary in the live decision review.

- `ContributorPartition::covers` retains its exact meaning for `MultiPass`, `CooperativeWorkgroup`, and every existing consumer.
- A new required tagged `ContributorCoverage` distinguishes `Exact(ContributorPartition)` from `IdentityPadded { partition, identity: ReductionPaddingIdentity }`. It may be `#[non_exhaustive]` for cross-crate additive growth, while same-crate verification and encoding remain exhaustive.
- The padding identity is an opaque or width-discriminated exact arithmetic value whose format and bit width cannot disagree. It is a statement, not a trusted proof: intrinsic verification derives two-sided neutrality against the actual scalar family, arithmetic type, rounding, signed-zero contract, NaN behavior, and any family-specific canonicalization.
- Exact coverage carries no identity. Padded coverage cannot omit one. No `Option`, `Default`, unknown mode, inferred constant, or fallback to `empty_identity_bits` exists.
- The verifier derives padding count by checked subtraction from the partition capacity and real contributor count, requires canonical suffix-only padding, and names exact-coverage and padded-coverage failures separately. A declared padding count would duplicate derivable authority and is excluded.
- The coverage value belongs to the reduction topology. `KernelSchedule::tail` remains iteration-domain launch coverage and cannot carry contributor padding.

## Source-first evidence

`ContributorPartition::covers` currently requires `partitions * contributors_per_partition == contributors`. The accepted subgroup example has `32 * 4 == 128` physical leaf positions for 101 real contributors. Strict f32 addition requires `-0.0` padding when signed zero is observable even though its empty-domain result is `+0.0`; the current maximum family uses `-inf`. These are different facts and make inference from the scalar program unsound.

## Required delivery

- Read ADRs 0022, 0025, 0093, 0094, and 0100 plus every current partition verifier, witness, identity encoder, lowering consumer, and numerical family before editing.
- Choose the smallest typed exact-bit carrier that supports the arithmetic types actually admitted by the owning reduction families without freezing a raw f32-only `u32` boundary.
- Encode coverage with an appended local tag and encode the identity only in the padded arm. Prove exact encodings remain byte-identical.
- Add typed rule/error vocabulary for overflow, capacity below the real count, noncanonical padding placement, arithmetic-type mismatch, and failed two-sided neutrality.
- Perturb each independent subject: coverage tag, partition capacity, arithmetic type, identity bits, signed-zero permission, and family. Quote each failure and restore the subject.
- Do not implement vector/subgroup execution, target facts, KIR instructions, or emission here.

## Performance boundary

Coverage validation is checked integer arithmetic plus a bounded family-specific identity proof at schedule verification. It allocates no step list and is outside kernel execution. A tighter host optimization needs measurement.

## Closes when

Both exact and identity-padded contributor coverage are canonically representable, malformed combinations are unrepresentable or typed refusals, old exact topologies retain their meaning and bytes, and the subgroup and vector schedule tickets can consume one shared concept.
