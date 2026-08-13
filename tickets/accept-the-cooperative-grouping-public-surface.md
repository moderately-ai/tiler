---
id: accept-the-cooperative-grouping-public-surface
title: Accept the cooperative grouping public surface
status: awaiting-decision
priority: p1
dependencies: []
related: [execute-the-loop-carried-cooperative-kernel-on-a-real-backend]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft cooperative grouping oracle so dependents can treat one three-axis reference grouping as accepted vocabulary.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public types and functions to Tom. [`execute-the-loop-carried-cooperative-kernel-on-a-real-backend`](execute-the-loop-carried-cooperative-kernel-on-a-real-backend.md) landed the draft at `33e8bb442b92f6ba3bfedc18b2777b1af0adc39f`. This node is not implementation work. Only Tom closes it.

## The surface, as landed at `33e8bb44`

**Included.** `tiler_reference::CooperativeCellLayout::{RoundMajor, ParticipantMajor}`; `CooperativeGrouping { participants, contributors_per_partition, rounds, layout }` with `declared`, `participant_major`, `covered_contributors`, and `cell_index`; `cooperative_grouped_sum` and `cooperative_grouped_sum_under`. A one-round `RoundMajor` grouping is bit-identical to `strict_partitioned_sum` at the same participant count.

**Excluded.** A second local `cooperative_reference` helper as a production oracle (the IR-test helper stays layer-local). Inferring a threaded CPU realization from the Metal measurement. Letting a caller ask the compiler for `rounds > 1`. Binding a live extent.

## Recommendation

Accept as drafted. The three-axis form is what a loop-carried tile needs and what a two-level `ContributorPartition` cannot state. **Strongest counterpoint:** public fields on `CooperativeGrouping` let a caller construct a zero-axis grouping that only the evaluators refuse.

## Closes when

Tom accepts, accepts with named exclusions, or revises.
