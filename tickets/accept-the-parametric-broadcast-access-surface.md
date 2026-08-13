---
id: accept-the-parametric-broadcast-access-surface
title: Accept the parametric broadcast access surface
status: awaiting-decision
priority: p1
dependencies: []
related: [carry-the-parametric-broadcast-relation-through-index-and-schedule-ir]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the labelled-draft parametric broadcast access surface so dependents can treat one sourced relation as accepted vocabulary rather than a draft.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes a new public enum variant to Tom. [`carry-the-parametric-broadcast-relation-through-index-and-schedule-ir`](carry-the-parametric-broadcast-relation-through-index-and-schedule-ir.md) produced a tested labelled draft at `cefa74394ca81468409cdfc123e766227a78f178`. This node is not implementation work. Only Tom closes it.

## The surface, as drafted at `cefa7439`

**Included.** `LogicalAccess::ParametricBroadcast { operand_shape: SourcedShape, mapping: BroadcastAxisMapping, environment: ShapeEnvIdentity }` on the existing `#[non_exhaustive]` `LogicalAccess`. Schedule identity tag `0x08` under `tiler.schedule.v5`; tags `0x01`–`0x07` and their field layouts are unchanged. `ParametricBroadcastRule` (`#[non_exhaustive]`) with `rule()` identifiers `parametric-broadcast.zero-capable`, `foreign-environment`, `extents-not-proved-equal`, `concrete-variant`, `stretch-source-not-proved-unit`, and `mapping`. `classify_broadcast_transform` / `replication_only_transform_is_admitted`. Kernel lowering matches the carrier and refuses `BodyRefinement` rather than binding a concrete `AxisDecode`.

**Excluded.** Compiler request-subject projection (`encode_access_relation` still writes `0x00` for this variant; remainder is [`project-parametric-broadcast-into-the-compiler-request-subject`](project-parametric-broadcast-into-the-compiler-request-subject.md)). Binding a live extent, selecting `BroadcastReplication` or `ReindexBijection` as a fallback, and any runtime fallback. Self-acceptance.

## Recommendation

Accept as drafted. The carrier is a third relation, not a parameterization of the two concrete maps, which is what keeps the bijective-at-one binding from being classified as replication. **Strongest counterpoint:** accepting before the compiler can project the variant into the request subject leaves a public schedule value the compile path can only refuse-to-encode as `0x00`.

## Closes when

Tom accepts, accepts with named exclusions, or revises. Do not merge the parent as an accepted surface on this packet alone.
