---
id: admit-a-cooperative-tile-over-shared-operands
title: Admit a cooperative tile whose participants share operands rather than one output
status: awaiting-decision
priority: p1
dependencies: [admit-a-two-dimensional-cooperative-staging-relation]
related: [realize-the-strict-contraction-on-metal, realize-the-tiled-contraction-schedule-and-its-metal-emission, implement-the-single-workgroup-synchronized-reduction-strategy]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, public-boundary, decision, needs-tom]
---
## User-visible outcome

A workgroup whose invocations each own **their own** output position, and cooperate only by staging shared *operand* data, is statable and verifiable — the relation a blocked GEMM tile has, and the one the existing cooperative tile is the inverse of.

## Why this is a second relation and not a widening

**Fact — the existing contract states the opposite relation, in three places.** `verify_cooperative_tile` (`crates/tiler-ir/src/schedule/builder.rs`) refuses `tile.commit.count != 1` under `CooperativeTileRule::CommitOwnership`, and its comment says why: exactly one committing participant "is what makes `OneGlobalInvocationPerOutput` true of a workgroup that runs several invocations over one output position". `owned_output_positions` divides `work_items` by the participant count for any region carrying a tile. `reduction_output_shape` requires the iteration shape to end in a trailing axis of exactly `partition.partitions`. `a_tile_committing_from_every_participant_is_rejected` and `a_cooperative_region_owns_one_position_per_workgroup` hold the first two.

**Inference.** A 16×16 operand tile has `commit.count == 256`, `owned_output_positions == work_items`, and no trailing participant axis. Relaxing those three rules would not narrow the existing contract — it would delete the fact the contract exists to state, and the tree strategy rests on that fact. The two relations must therefore be separate and separately verified, which is the same reason `ReductionTopology` keeps `MultiPass` and `CooperativeWorkgroup` apart rather than parameterizing one.

## What this owns

- **The second tile relation**: participants sharing staged operand data, each committing one output, with its own commit and coverage rules. What replaces "exactly one committer" is a statement that the participants' owning writes are a bijection onto the workgroup's output block — and that statement has to be *checkable*, not asserted.
- **The invocation-to-output map, and its ownership evidence.** `verify_contraction` requires `write.map == LogicalAccess::LinearIdentity` and the kernel lowering stores at the invocation value itself. Under a 16×16 tiling the owning output of global invocation `gid` is `(gid / 256 / (N/16) * 16 + gid % 256 / 16) * N + (gid / 256 % (N/16)) * 16 + gid % 256 % 16`, which equals `gid` only at `N == 16`. So this needs a new `LogicalAccess` variant for the tile-blocked map **and** a new `OwnershipProofKind`: `OneGlobalInvocationPerOutput` is discharged today by the identity map, and discharging it for a blocked map is a bijectivity argument nothing in the model currently makes. That is a new validation authority, and it is the part to be most sceptical about — an ownership proof that cannot fail is not evidence.
- Whether the tail is refused or handled. The existing tile has no tail by contract, and a blocked map over extents that are not tile multiples has one. Refuse it explicitly rather than masking committers, and state which of the two this relation admits.

## What this does not own

The staged relation the tile's reads need ([`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md), a hard dependency: without it the tile's reads are unstatable and this relation would be verified over an access it cannot express), the contraction topology and its Metal body ([`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)), and any cost model that would make the tile win.

## Activation history

This was deferred behind its dependency and Tom's acceptance of two public boundaries — a second cooperative tile relation and a new `OwnershipProofKind`. [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) is now `done`; only the acceptance condition remains.

## Decision boundary — 2026-08-09

The implementation dependency is `done`; the only remaining activation condition is Tom's answer. This ticket therefore belongs in `awaiting-decision`, not `deferred`.

Tom decides whether the next cooperative-tile vertical may add both of these public schedule concepts:

1. a second cooperative relation in which every participant owns one output while sharing staged operands; and
2. an ownership-proof kind that proves the blocked invocation-to-output map is a bijection over the declared output block.

**Recommendation: accept the pair as one bounded vertical.** The existing one-committer relation proves a different ownership theorem and cannot be weakened without invalidating the workgroup-reduction contract; a separate relation and proof keep both statements checkable. **Strongest counterpoint:** the blocked map, tail policy, and exact proof payload have not yet been implemented, so Tom may prefer a smaller research spike that fixes their spelling before accepting public enum variants.

If accepted, return this ticket to `todo` and implement the exact accepted spelling. If revised, record the replacement boundary here before dispatch. Acceptance does not authorize the Metal body or cost model owned by the related tickets.

## Closes when

A tile whose participants each commit their own output verifies, its ownership evidence has been watched refusing a map that is not a bijection onto the declared block, the existing one-committer tile still verifies unchanged with its own rules intact, and the identity consequence of whatever the relation required is recorded.

## Trigger check log

- 2026-08-04 — **half fired; stays `deferred`, and the half that has not fired is Tom's.** The activation trigger is a conjunction. Its first conjunct **has** fired: [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) is `done`, so the staged relation this tile's reads need is expressible. Its second conjunct has not: Tom has accepted neither of the two public boundaries this ticket requires. Checked rather than assumed — `OwnershipProofKind` still declares exactly one variant, `OneGlobalInvocationPerOutput` (`crates/tiler-ir/src/schedule/model.rs:253-259`), so the new proof kind does not exist, and `verify_cooperative_tile` still refuses `tile.commit.count != 1` under `CooperativeTileRule::CommitOwnership`, so the second tile relation does not either. **Reactivating on the landed dependency alone would assert an acceptance nobody relayed**, which is a product boundary this sweep does not cross; it is recorded here instead so the next reader sees one conjunct outstanding rather than two. Recheck: `grep -n 'enum OwnershipProofKind' -A 8 crates/tiler-ir/src/schedule/model.rs`.
