---
id: carry-the-parametric-broadcast-relation-through-index-and-schedule-ir
title: Carry the parametric broadcast relation through index and schedule IR
status: done
priority: p1
dependencies: [replace-broadcast-f32-v1-with-sourced-broadcast-f32-v2-semantics]
related: [accept-the-parametric-broadcast-access-surface, project-parametric-broadcast-into-the-compiler-request-subject]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, scheduling, broadcast, identity]
---
# Carry the parametric broadcast relation through index and schedule IR

## User-visible outcome

Index realization and schedule verification carry one broadcast relation over its whole symbolic domain, including its bijective binding at one, without lying that it is always replication or always reindexing.

## Work

- Add one explicitly tagged parametric broadcast access relation carrying the sourced operand/result relation and exact environment identity needed to interpret it.
- Keep `BroadcastReplication` and `ReindexBijection` unchanged. The new carrier is neither concrete variant; consumers must match it explicitly.
- Extend the governed index law/lowering, canonical schedule encoder, builders, verifier, realization witnesses, request-subject projection, fusion classification, costing, and exhaustive tag/population tests.
- Prove bounds and coordinate equality for every admitted binding. Permit replication-only transformations only when the environment proves actual widening; otherwise conservatively decline them.
- Preserve all existing access/schedule bytes by adding fresh discriminants. Step a domain only if an old payload must be reinterpreted.
- Keep the relation symbolic. Do not bind an extent, select a concrete access variant, or introduce a runtime fallback in this layer.

## Acceptance

- The same carrier verifies at bindings one, two, ten, and the admitted upper bound.
- Forged zero-capable, foreign-environment, wrong-equality, and concrete-variant substitutions fail under distinct typed rules.
- A replication-only fusion/cost path declines when actual widening is unproved and admits the proved-widening neighbour.
- Existing concrete reindex and broadcast canonical bytes remain unchanged; new tag injectivity is perturbed and observed failing.

## Stop conditions

Stop if the carrier would require runtime-bound values in semantic identity, if any consumer treats it as concrete replication through a wildcard, or if one-artifact lowering needs a different coordinate language than the accepted sourced relation.

## Source-first Fact audit — 2026-08-13, exact base `1dc1c9d78c3a35b9c61993f970774f6afdd991bd`

- **Verified:** `LogicalAccess` already has concrete `BroadcastReplication` and `ReindexBijection` with operand/result `Shape` and `Vec<AxisDecode>`. Anchor: `pub enum LogicalAccess` in `crates/tiler-ir/src/schedule/model.rs`.
- **Verified:** `BroadcastReplication` is documented as the access relation of registered `tiler::broadcast-f32@2` when it actually widens, and is deliberately not `ScalarBroadcast`. Anchor: `The access relation of the registered \`tiler::broadcast-f32@2\` family`.
- **Verified:** `LogicalAccess` is ADR 0074 convention **5a** (`#[non_exhaustive]`), not 5b. A new variant is additive. Out-of-crate consumers already have fail-closed wildcards (`encode_access_relation` writes `0x00`; `access_domain_shape` answers `None`). In-crate matches are exhaustive and must name the new carrier.
- **Verified:** Index-law `Broadcast` is tag 6 and realizes coordinate expressions from the mapping. It used `mapping.result_shape` (static path) and `IndexRegionBuilder::new` (no environment).
- **Verified:** Schedule identity domain is `tiler.schedule.v5`. Access tags `0x01`–`0x07` are append-only; a fresh `0x08` does not reinterpret an old payload, so the domain does not step.
- **Verified:** Fusion classification in `tiler-compiler` is keyed by semantic `OpKey`, not by `LogicalAccess`. Costing has no broadcast-specific `LogicalAccess` path. IR now owns `classify_broadcast_transform` / `replication_only_transform_is_admitted` so a replication-only consumer must match the carrier explicitly.
- **Verified:** `IndexRefinementSubject::derive` previously refused symbolic boundaries under `SymbolicSemanticBoundary`. That refusal would have blocked carrying the sourced relation through index IR. Derive now retains the authored `SourcedShape`; static subject identity bytes still use the previous `encode_shape` path.
- **Decision:** one `LogicalAccess::ParametricBroadcast` carrier, not a sibling type. `Access.map` is `LogicalAccess`; `ReindexBijection` / `BroadcastReplication` already live there; a sibling would be a second public map on `Access`. Not two materially different carriers.

## Implementation record — 2026-08-13

`LogicalAccess::ParametricBroadcast { operand_shape, mapping, environment }` is tag `0x08`. `BroadcastReplication` and `ReindexBijection` are unchanged. Index-law `Broadcast` realizes a sourced mapping through `mapping.apply` and symbolic dimensions when the mapping names a symbol, and keeps the environment-free path for literal mappings. Kernel lowering matches the carrier and refuses (`BodyRefinement`) rather than binding `AxisDecode`. Replication-only fusion/cost is `replication_only_transform_is_admitted`. Tom accepted this exact spelling on 2026-08-13 in [`accept-the-parametric-broadcast-access-surface`](accept-the-parametric-broadcast-access-surface.md).
