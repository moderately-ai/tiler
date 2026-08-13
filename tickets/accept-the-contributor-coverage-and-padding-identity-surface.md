---
id: accept-the-contributor-coverage-and-padding-identity-surface
title: Accept the contributor-coverage and padding-identity surface
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-shared-contributor-coverage-and-reduction-padding-identity]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft Rust spelling of the contributor-coverage model he accepted on 2026-08-11.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public types and a field rename on existing public variants to Tom. The 2026-08-11 packet on [`admit-shared-contributor-coverage-and-reduction-padding-identity`](admit-shared-contributor-coverage-and-reduction-padding-identity.md) accepted the *model*. This node is the spelling landed at `d8a09031139c603429169ab47972193c89c3478e`. Only Tom closes it.

## The surface, as landed at `d8a09031`

**Included.**

- `ContributorCoverage::{Exact(ContributorPartition), IdentityPadded { partition, identity }}` (`#[non_exhaustive]`).
- `ReductionPaddingIdentity::{F16(u16), Bf16(u16), F32(u32), F64(u64)}` (exhaustive; encoder convention 5b).
- `ContributorCoverageRule` and `ScheduledRegionDiagnostic::ContributorCoverage`.
- `KernelDiagnostic::PaddedContributorCoverage`.
- `ReductionTopology::{MultiPass, CooperativeWorkgroup}` field `partition` renamed to `coverage: ContributorCoverage`.
- Accessors `ContributorCoverage::partition`, `ReductionPaddingIdentity::arithmetic_type`, `RealizationWitness::contributor_coverage`.
- Exact coverage writes no schedule-identity suffix under `tiler.schedule.v5`. Padded coverage appends local tag `0x01` then `ArithmeticType::tag` plus exact-width bits.

**Excluded.**

- A declared pad count. `Option`/`Default`/unknown coverage. Padding on `KernelSchedule::tail`. Vector-lane or subgroup topologies. KIR emission of padding. Family-level algebraic-identity capability declarations. Weakening `ContributorPartition::covers`. A domain step.

## Recommendation

Accept as drafted. The spelling follows the accepted model: required tagged coverage, identity only on the padded arm, verifier-derived pad count, suffix-only placement, no empty-identity fallback. Exact encodings keep prior bytes. **Strongest counterpoint:** renaming `partition` to `coverage` on two existing public variants is a breaking field change even for callers who only construct `Exact`.

## Closes when

Tom accepts, accepts with named exclusions, or revises.
