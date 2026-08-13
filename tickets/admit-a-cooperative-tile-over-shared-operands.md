---
id: admit-a-cooperative-tile-over-shared-operands
title: Admit a cooperative tile whose participants share operands rather than one output
status: done
priority: p1
dependencies: [admit-a-two-dimensional-cooperative-staging-relation]
related: [realize-the-strict-contraction-on-metal, realize-the-tiled-contraction-schedule-and-its-metal-emission, implement-the-single-workgroup-synchronized-reduction-strategy, admit-guarded-output-tails-for-cooperative-contraction, accept-the-blocked-workgroup-and-cooperative-contraction-surface]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, public-boundary]
---
## User-visible outcome

On an exactly tiled output domain, a workgroup whose invocations each own **their own** output position, and cooperate only by staging shared *operand* data, is statable and verifiable — the exact-divisible relation a blocked GEMM tile has, and the one the existing cooperative tile is the inverse of.

## Why this is a second relation and not a widening

**Fact — the existing contract states the opposite relation, in three places.** `verify_cooperative_tile` (`crates/tiler-ir/src/schedule/builder.rs`) refuses `tile.commit.count != 1` under `CooperativeTileRule::CommitOwnership`, and its comment says why: exactly one committing participant "is what makes `OneGlobalInvocationPerOutput` true of a workgroup that runs several invocations over one output position". `owned_output_positions` divides `work_items` by the participant count for any region carrying a tile. `reduction_output_shape` requires the iteration shape to end in a trailing axis of exactly `partition.partitions`. `a_tile_committing_from_every_participant_is_rejected` and `a_cooperative_region_owns_one_position_per_workgroup` hold the first two.

**Inference.** A 16×16 operand tile has `commit.count == 256`, `owned_output_positions == work_items`, and no trailing participant axis. Relaxing those three rules would not narrow the existing contract — it would delete the fact the contract exists to state, and the tree strategy rests on that fact. The two relations must therefore be separate and separately verified, which is the same reason `ReductionTopology` keeps `MultiPass` and `CooperativeWorkgroup` apart rather than parameterizing one.

## Accepted boundary — 2026-08-11

**Accepted by Tom in the Codex coordination session after a fresh exact-base audit, with the ticket's original map/proof proposal corrected before implementation.** The accepted first pass is deliberately exact and narrow:

- Add a sibling cooperative-contraction topology. It may reuse the existing `CooperativeTile` dataflow record, but it has its own semantic, commit, coverage, and shape verifier. `ReductionTopology::CooperativeWorkgroup` and its one-committer theorem remain unchanged; no helper may infer reduction ownership merely from the presence of a tile.
- Add an explicit blocked-workgroup [`ExecutionBinding`](../crates/tiler-ir/src/schedule/model.rs). The binding maps hardware workgroup/local coordinates to the contraction's logical output coordinates once, for both operand reads and the owning write. This is the layer [ADR 0007](../docs/decisions/0007-first-class-kernel-schedules.md) assigns hardware-to-logical mapping to. It is required rather than defaulted.
- Keep the contraction's owning write as `LogicalAccess::LinearIdentity` *after* that binding, and keep `OwnershipProofKind::OneGlobalInvocationPerOutput`. The proof kind states the theorem; the binding verifier supplies the new bijectivity argument. A new logical-access map would restate one execution mapping on three accesses, and a new ownership-proof kind would duplicate the theorem while forcing a tag into the currently untagged ownership encoding.
- Admit only exact output blocks and exact contracted tiles. Preflight must prove every output extent is divisible by its block extent and `K` is divisible by the contracted tile width before constructing the schedule. A caller selecting this tiled approach receives a typed refusal when any equality is absent or false. It never silently substitutes the direct contraction.
- Preserve old identity bytes. The new execution-binding and topology alternatives receive fresh appended tags on the tree they land into; existing binding/topology encodings keep their bytes. No `tiler.schedule.v6` step is authorized by this decision.

**Correction to the original decision packet.** The assertion that a new `LogicalAccess` and a new `OwnershipProofKind` were required was false. The blocked coordinate governs both reads and the write, so placing it on one write would split one authority across layers; `OwnershipProofKind::OneGlobalInvocationPerOutput` already states the exact theorem the blocked bijection must establish. The assertion that the measured tile has every participant commit was also only true on exact-divisible output blocks: the retained `M = 1` and `M = 10` kernels keep every participant convergent but mask out-of-range operand loads and owning writes. Those guarded tails are a separate public and verification boundary owned by [`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md), not a silent widening of this first pass.

## What this owns

- The exact-divisible sibling cooperative-contraction topology and its relation-specific verifier.
- The explicit blocked-workgroup execution binding and the algebraic proof that it is a bijection from launched invocations onto the declared output domain.
- Typed preflight refusals for output-block or contracted-tile divisibility, with no automatic direct fallback.
- The additive identity encoding and regression evidence that every existing one-committer schedule retains its bytes and verifier rules.

## What this does not own

The staged relation the tile's reads need ([`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md), a hard dependency: without it the tile's reads are unstatable and this relation would be verified over an access it cannot express), guarded output tails ([`admit-guarded-output-tails-for-cooperative-contraction`](admit-guarded-output-tails-for-cooperative-contraction.md)), the Metal body ([`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)), and any cost model that would make the tile win.

## Activation history

**Historical state.** This was deferred behind its dependency and Tom's acceptance of two proposed public boundaries — a second cooperative tile relation and a new `OwnershipProofKind`. [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) reached `done`; the original acceptance packet then remained. The 2026-08-11 audit corrected that packet and Tom accepted the replacement boundary above, so neither the old proof-kind proposal nor this historical activation condition remains live.

## Superseded decision boundary — 2026-08-09

The implementation dependency is `done`; the only remaining activation condition is Tom's answer. This ticket therefore belongs in `awaiting-decision`, not `deferred`.

Tom decides whether the next cooperative-tile vertical may add both of these public schedule concepts:

1. a second cooperative relation in which every participant owns one output while sharing staged operands; and
2. an ownership-proof kind that proves the blocked invocation-to-output map is a bijection over the declared output block.

**Superseded 2026-08-11.** The recommendation to accept a new access map and ownership-proof kind is replaced by the accepted execution-binding design above. The observation that the one-committer relation must remain separate still stands.

This ticket is now `todo` for the exact-divisible IR vertical. Acceptance does not authorize guarded tails, the Metal body, or a cost model.

## Closes when

An exact-divisible cooperative contraction verifies under its explicit blocked execution binding; perturbing the block mapping, output extent, block extent, or launch relation independently makes the bijection proof refuse; the existing one-committer tile verifies unchanged and keeps identical canonical bytes; unsupported output or contracted tails fail in preflight with typed reasons; and every new identity tag and downstream pin consequence is recorded.

## Trigger check log

- 2026-08-04 — **half fired; stays `deferred`, and the half that has not fired is Tom's.** The activation trigger is a conjunction. Its first conjunct **has** fired: [`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) is `done`, so the staged relation this tile's reads need is expressible. Its second conjunct has not: Tom has accepted neither of the two public boundaries this ticket requires. Checked rather than assumed — `OwnershipProofKind`, source anchor `pub enum OwnershipProofKind`, still declares exactly one variant, `OneGlobalInvocationPerOutput`, so the new proof kind does not exist, and `verify_cooperative_tile` still refuses `tile.commit.count != 1` under `CooperativeTileRule::CommitOwnership`, so the second tile relation does not either. **Reactivating on the landed dependency alone would assert an acceptance nobody relayed**, which is a product boundary this sweep does not cross; it is recorded here instead so the next reader sees one conjunct outstanding rather than two. Recheck: `rg -n 'enum OwnershipProofKind' crates/tiler-ir/src/schedule/model.rs`.

## Fact audit — 2026-08-13, exact base `4333df31`

1. **Verified.** `verify_cooperative_tile` still refuses `tile.commit.count != 1` under `CooperativeTileRule::CommitOwnership`. Source anchor: `if tile.commit.count != 1 || !participants.contains_range(tile.commit)`. The one-committer path still uses that rule; the sibling uses a new `OperandTileCommit` rule instead of weakening it.
2. **Verified.** `ReductionTopology::CooperativeWorkgroup` and `OwnershipProofKind::OneGlobalInvocationPerOutput` exist and were not weakened. The proof kind still has that one variant.
3. **Coordinator-unverified, now read.** `owned_output_positions` previously divided `work_items` by the participant count whenever `cooperative_tile` returned a tile. That helper inferred one-committer ownership from the mere presence of a tile. It now matches `CooperativeWorkgroup` only. `reduction_output_shape` still appends a trailing participant axis only for `CooperativeWorkgroup` (and multi-pass partial); `CooperativeContraction` keeps the iteration shape as the output. `ExecutionBinding` had only `GlobalLinearInvocation`, encoded as `0x01`. Schedule identity is `tiler.schedule.v5`. The two-dimensional staging relation is `done` and is what the tile's four staged accesses use.

## Outcome — 2026-08-13

Exact-divisible first pass as accepted 2026-08-11.

- `ReductionTopology::CooperativeContraction` (accepted 2026-08-13) reuses `CooperativeTile`, with its own semantic / commit / coverage / shape verifier.
- `ExecutionBinding::BlockedWorkgroup { block, workgroups }` (accepted 2026-08-13) is required, never defaulted. The verifier proves `workgroups[d] * block[d] == output[d]` per axis: greater is overlap, lesser is gap.
- Owning write stays `LogicalAccess::LinearIdentity`. Proof kind stays `OneGlobalInvocationPerOutput`. No new access map, no new proof kind, no `tiler.schedule.v6`.
- `admit_exact_cooperative_contraction` is the typed preflight. It never returns a direct `Contraction` schedule.
- Kernel lowering refuses the new topology as `CooperativeLoweringShape` (Metal body is out of scope).

### New identity tags

| Site | Tag | Meaning |
| --- | --- | --- |
| `ExecutionBinding::BlockedWorkgroup` | `0x02` | appended; `GlobalLinearInvocation` keeps `0x01` |
| `ReductionTopology::CooperativeContraction` | `0x37` | appended. `0x36` is reserved for the accepted `CooperativeContractionSplit` spelling and is not consumed |

### Identity blast radius

Existing one-committer `[2, 6] -> [2]` cooperative fixture bytes are identical to the encoding at `4333df31`, pinned as `ONE_COMMITTER_COOPERATIVE_IDENTITY_HEX`. `STRICT_F32_REGION_IDENTITY_HEX` is unchanged. Domain remains `tiler.schedule.v5`.

### Quoted perturbations

- Overlap (`workgroups = [3, 2]` over `[32, 32]` / `[16, 16]`): `blocked-workgroup-mapping-overlap`
- Gap (`workgroups = [1, 2]`): `blocked-workgroup-mapping-gap`
- Non-divisible output (`33` / `16`): `cooperative-contraction-output-block-not-divisible` (`OutputBlockNotDivisible { axis: 0, output: 33, block: 16 }`)
- Non-divisible K (`17` / `16`): `cooperative-contraction-contracted-tile-not-divisible` (`ContractedTileNotDivisible { axis: 0, contracted: 17, tile: 16 }`)
- Helper inferring ownership from a tile (`output_count: 4` on a 1024-position tile): `proof-reference`
- One-committer still refuses every participant committing: `cooperative-commit-ownership`
- Operand-sharing tile refuses a single committer: `cooperative-operand-tile-commit`

Status `review`. Not merged. Not `done`.
