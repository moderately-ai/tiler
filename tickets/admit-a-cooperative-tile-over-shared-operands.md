---
id: admit-a-cooperative-tile-over-shared-operands
title: Admit a cooperative tile whose participants share operands rather than one output
status: todo
priority: p1
dependencies: [admit-a-two-dimensional-cooperative-staging-relation]
related: [realize-the-strict-contraction-on-metal, realize-the-tiled-contraction-schedule-and-its-metal-emission, implement-the-single-workgroup-synchronized-reduction-strategy, admit-guarded-output-tails-for-cooperative-contraction]
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
