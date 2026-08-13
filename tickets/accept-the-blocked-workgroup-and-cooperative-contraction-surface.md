---
id: accept-the-blocked-workgroup-and-cooperative-contraction-surface
title: Accept the blocked-workgroup and cooperative-contraction surface
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-a-cooperative-tile-over-shared-operands]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom accepts or revises the exact Rust spelling of the blocked-workgroup binding and operand-sharing cooperative-contraction topology he accepted as a model on 2026-08-11.

## Decision boundary

[ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) routes new public variants to Tom. The 2026-08-11 packet on [`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md) accepted the *model*. This node is the spelling landed at `feb25c21f0ca1e68dbd22bba4e77a450a595f5b3`. Only Tom closes it.

## The surface, as landed at `feb25c21`

**Included.** `ExecutionBinding::BlockedWorkgroup { block: Shape, workgroups: Shape }` (tag `0x02`; `GlobalLinearInvocation` keeps `0x01`). `ExecutionBinding` is no longer `Copy` because the map carries shapes. `ReductionTopology::CooperativeContraction { tile, contracted_shape, contracted_tile, order, accumulation, permits_reassociation, permits_permutation }` (tag `0x37`; `0x36` remains reserved for the accepted `CooperativeContractionSplit` spelling). `admit_exact_cooperative_contraction` and `prove_blocked_bijection`. Owning write stays `LogicalAccess::LinearIdentity`. Proof kind stays `OneGlobalInvocationPerOutput`. Domain stays `tiler.schedule.v5`.

**Excluded.** Weakening `CooperativeWorkgroup`'s one-committer theorem. A new `LogicalAccess` or `OwnershipProofKind`. Automatic fallback to a direct `Contraction` schedule. Consuming tag `0x36`. Metal lowering of the new topology (`CooperativeLoweringShape` refusal). Guarded output tails.

## Recommendation

Accept as drafted. The spelling follows the accepted model: sibling topology, required blocked binding, no new access map, no new proof kind, append-only tags. **Strongest counterpoint:** dropping `Copy` on `ExecutionBinding` is a public trait- impl change even for callers who only construct `GlobalLinearInvocation`.

## Closes when

Tom accepts, accepts with named exclusions, or revises.
