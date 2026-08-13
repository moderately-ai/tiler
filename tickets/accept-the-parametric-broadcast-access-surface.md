---
id: accept-the-parametric-broadcast-access-surface
title: Accept the parametric broadcast access surface
status: done
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

**Excluded.** Binding a live extent, selecting `BroadcastReplication` or `ReindexBijection` as a fallback, and any runtime fallback. Self-acceptance. Kernel lowering of a body: the carrier is matched and refused as `BodyRefinement` rather than bound into `AxisDecode`.

**Repaired 2026-08-13, this session.** The earlier Excluded clause that `encode_access_relation` still writes `0x00` for this variant is false at the current base. [`admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary`](admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary.md) landed crate-internal request-subject tag `0x05` under `tiler.compiler.request-subject.v6` at `c056affb`. Existing `0x01`/`0x02`/`0x03` maps keep their bytes. `0x00` remains the refusal for unprojected maps. That projection is crate-internal, not part of this public IR surface.

## Recommendation

Accept as drafted. The carrier is a third relation, not a parameterization of the two concrete maps, which is what keeps the bijective-at-one binding from being classified as replication. **Strongest counterpoint:** the variant is publicly constructible while kernel lowering still only refuses `BodyRefinement`, so accepting freezes a schedule vocabulary that cannot yet emit a body.

## Accepted — 2026-08-13

**Tom accepted the exact surface as drafted**, with no named exclusion, in the live coordination session. The included set is `LogicalAccess::ParametricBroadcast { operand_shape, mapping, environment }`, schedule identity tag `0x08` under `tiler.schedule.v5`, `ParametricBroadcastRule` with the six named identifiers, `classify_broadcast_transform` / `replication_only_transform_is_admitted`, and kernel lowering that matches the carrier and refuses `BodyRefinement`. Live-extent binding and a `BroadcastReplication` / `ReindexBijection` fallback remain excluded.

The request-subject projection at crate-internal tag `0x05` is already landed and is not this public IR surface. In-code labels flip from labelled draft to accepted public surface.

## Closes when

Tom accepts, accepts with named exclusions, or revises. Do not merge the parent as an accepted surface on this packet alone.
