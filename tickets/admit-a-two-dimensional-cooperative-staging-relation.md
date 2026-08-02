---
id: admit-a-two-dimensional-cooperative-staging-relation
title: Admit a cooperative staging relation a two-dimensional tile can state
status: in-progress
priority: p1
dependencies: []
related: [realize-the-strict-contraction-on-metal, admit-a-cooperative-tile-over-shared-operands, realize-the-tiled-contraction-schedule-and-its-metal-emission, implement-the-single-workgroup-synchronized-reduction-strategy, admit-loop-carried-cooperative-staging]
scopes: [implementation/ir, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, identity, deferred]
claimed_from: todo
assignee: agent-stagingv5-r2
lease_expires_at: 1785686685
---
## User-visible outcome

A cooperative tile can state a staged access whose slot set depends on a participant's position in **two** dimensions — the relation every blocked GPU kernel's shared-memory read has, and the one thing the current vocabulary cannot express.

## The gap, with the exact refutation

**Fact.** `StagedSpan` (`crates/tiler-ir/src/schedule/cooperative.rs`) addresses `count` contiguous slots at `stride * l + offset`, and `CooperativeTile::addressed_slots` enumerates it over the linear participant coordinate. `LocalCoordinateSource` has one variant, `LocalLinearInvocation`, and the module's own documentation states that multi-dimensional local coordinates are "absent rather than reserved".

**Fact — the refutation, from [`realize-the-strict-contraction-on-metal`](realize-the-strict-contraction-on-metal.md).** A 16×16 tile's two staged reads need `base_a(l) = 16 * (l / 16)` and `base_b(l) = 16 * (l % 16)`. `base_a(0) = base_a(1) = 0` forces `stride = 0` while `base_a(16) = 16`; `base_b(0) = 0` and `base_b(1) = 16` force `stride = 16` while `base_b(16) = 0`. And no participant relabelling helps: over a 256-element domain `stride * l + offset` is constant or injective, while both profiles take 16 distinct values with multiplicity 16.

**Fact — two consumers are blocked by exactly this, from opposite directions.** The `tiled` contraction needs it for its operand broadcast. `workgroup_tree_tile` is depth two rather than log-depth for two reasons its own documentation records, and the first — "a span whose stride and count are functions of the round ordinal" — is the same missing dependence stated over the round instead of over a second participant dimension. Whether one relation covers both, or they are two, is part of what this ticket decides.

## What this owns

- The relation's shape. The narrowest form covering the 16×16 reads is the kernel lowering's own `OffsetTerm` — `stride * ((l / divisor) % modulus) + offset` — and the most faithful is a genuinely two-dimensional participant space with a per-dimension stride. They are not the same decision: the first keeps `ParticipantRange` one-dimensional and encodes the tile's shape in a divisor, the second makes the shape a first-class fact a verifier can check against the launch geometry. State what each enables and prevents on a concrete tile before proposing one.
- **Whether the round ordinal enters the relation**, which is the log-depth tree's half of the gap and is deliberately not assumed to be the same axis.
- The identity step. `push_staged_span` (`crates/tiler-ir/src/schedule/model.rs`) writes three unframed big-endian `u64`s with no tag, and `push_participant_range` two. Every candidate — an added field, or a tag byte in front of the existing form — moves every cooperative region's bytes. `tiler.schedule.v4` therefore steps to `v5`, and with it every pinned schedule, kernel, and artifact identity that folds one. Execute it completely or not at all: the version moves at its owning layer, the ledger documents move in the same commit, and every pinned identity is recomputed on the merged tree with each moved pin enumerated.
- Keeping the enumeration decidable. `MAX_COOPERATIVE_PARTICIPANTS` and `MAX_COOPERATIVE_STAGING_SLOTS` exist so disjointness and coverage are decided by enumerating addressed slots rather than by a modular argument. A widened relation must stay enumerable under the same bounds, and the disjointness rule must still refuse two writers reaching one slot inside one round.

## What this does not own

The second tile relation for participants that each commit their own output ([`admit-a-cooperative-tile-over-shared-operands`](admit-a-cooperative-tile-over-shared-operands.md)), the contraction's schedule and Metal body ([`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md)), and the log-depth tree's active-participant subset, which is a separate absent capability.

## Activation triggers

Deferred rather than dispatchable, because its central act is an identity-domain step at a public boundary and [AGENTS.md](../AGENTS.md) reserves that for Tom. It becomes work when **either** trigger fires:

1. Tom accepts the `tiler.schedule.v4` → `v5` step and a widened `StagedSpan`/`LocalCoordinates` boundary; **or**
2. a second consumer independently needs it — the log-depth tree under [`implement-the-single-workgroup-synchronized-reduction-strategy`](implement-the-single-workgroup-synchronized-reduction-strategy.md) reaching its depth limit is the concrete one — which changes the cost/benefit the first trigger is a judgement about.

## Closes when

A two-dimensional staged access is statable, its disjointness and coverage are still decided by enumeration under the governed bounds, every new rule has been watched refusing its own defect, and the identity step is complete: version moved at its owning layer, ledger updated in the same commit, every moved pin recomputed on the merged tree and enumerated in the report.

## Activated 2026-08-01 — trigger 1 fired, with a co-derivation direction

**Tom accepted the `tiler.schedule.v4` → `v5` step and the widened boundary at the live session, witnessed and executed by the coordinator.** The direction he chose sharpens what this ticket owns: the relation is **co-derived with ADR 0096's two-component coordinate** — one concept serving the tiled contraction's operand broadcast, the two-level composition's staged access by named coordinate component, and the log-depth tree's round-dependent span, rather than three widenings. ADR 0096 decision 4's two-component coordinate constrains the shape fork toward the first-class participant space over the divisor/modulus encoding; the elimination is still this ticket's to run and state, and the exact widened `StagedSpan`/`LocalCoordinates` boundary comes back to Tom as a draft under ADR 0075. The identity step lands once, completely, under the full ledger discipline the body above states.

## Outcome — 2026-08-02, the derivation and the boundary draft

**Design phase complete; nothing was implemented and no encoding, version string, field, or pinned value moved.** [A two-dimensional cooperative staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) is the record: the elimination, the round-ordinal derivation, the ADR 0096 reconciliation, the decidability argument, the pin enumeration, and a verbatim-landable ADR 0075 boundary draft. **Measurement boundary: nothing was executed, emitted, compiled, dispatched, or timed** — the arithmetic below is derived by substitution over a stated domain, not observed.

**The shape fork converged, so it is not a question for Tom.** The `OffsetTerm` form `stride * ((l / divisor) % modulus) + offset` covers the tiled contraction's two staged reads and **cannot express its two staged writes**. The `b_tile` transpose write is `w(l) = 16 * (l % 16) + (l / 16)`, with `w(0) = 0`, `w(1) = 16`, `w(16) = 1`; from `w(0) = 0` the offset is zero, `d = 1` forces `stride = 16` and then requires `16 * x = 1`, and every `d >= 2` sends `w(1)` to zero. The transpose cannot be moved to the read side either, because the repaired read addresses a *strided* slot set and `StagedSpan.count` states contiguous slots. The narrower candidate therefore fails on the exact kernel it was proposed to admit, additionally carries no round variable, and embeds a tile width no verifier rule can relate to the launch. A two-dimensional participant space with a per-dimension stride states all four accesses with contiguous counts, is what `spikes/scheduling/metal_contraction_vertical/kernels.metal:103` already reads as `uint2 tid [[thread_position_in_threadgroup]]`, and is the direction accepted ADR 0096 decision 4 constrains toward.

**The round ordinal does not enter this relation, and the two consumers are two relations.** The contraction's staged slot indices are round-*invariant* — `kernels.metal:128-133` uses the same indices inside the `k0` loop as `:116-119` does before it, and only the loaded device address varies. The log-depth tree's are round-dependent, and admitting that here would land half a capability twice: a round-dependent span makes per-round coverage a shrinking subset rather than the bijection `crates/tiler-ir/src/schedule/builder.rs:1205-1210` decides, and the tree separately needs the per-access active-participant subset `crates/tiler-ir/src/schedule/cooperative.rs:70-75` records as absent. Filed as [`admit-a-round-dependent-cooperative-staging-span`](admit-a-round-dependent-cooperative-staging-span.md) at `deferred` with two triggers.

**Decidability survives, which is the result that would have stopped the dispatch had it gone the other way.** The widened enumeration ranges over the Cartesian product of the participant extents, which is *the same participant set* the linear enumeration walks, re-indexed — so `MAX_COOPERATIVE_PARTICIPANTS` and `MAX_COOPERATIVE_STAGING_SLOTS` bound it exactly as before, and no third enumeration bound is needed. One new bound, `MAX_COOPERATIVE_PARTICIPANT_RANK`, bounds the address sum and the encoded frame rather than the enumeration, and is not implied by the participant bound because a space of unit extents has a product of one at any rank. The occupancy map is keyed on slots, so the one-writer-per-slot-per-round rule is untouched and still fires: a span perturbed to `strides = [16, 16]` sends participants `(0,1)` and `(1,0)` both to slot 16.

**The `v5` blast radius is thirty-one lines across nine files, and six of them are outside `crates/tiler-ir`.** All six `crates/tiler-metal/goldens/*.metal` carry an entry symbol, a kernel identity digest, and a scheduled-region identity digest at lines 35, 36, 37, and 41 or 42 — **including the four with no cooperative tile**, because `crates/tiler-ir/src/kernel/model.rs:1757` folds the scheduled-region identity bytes whole and the separator is the leading eighteen bytes of every one. Plus `crates/tiler-ir/src/schedule/model.rs:1878`, `builder.rs:1683` and `:1691`, and `crates/tiler-build/src/metal_plan.rs:840, 842, 858, 860`. Eight prose lines move in the same commit, three of them in `docs/artifact-abi.md` — which is `contracts/artifacts`, **so the implementation wave's ticket must hold that scope or the step lands in halves**. The research record carries the reproducing command and the full table, including sixteen dated-measurement lines that must **not** move.

**One finding rather than a classification.** `spikes/runtime/inline-dispatch/README.md:90-91` tracks an entry symbol *across* commits by hand, so it will silently go stale at `v5` and no gate checks it. It is not an unrecomputable pin — the symbol needs the compile path and no device — so the brief's third stop condition did not fire. The treatment is one dated line recording that it moved and to what, with the transcript left intact.

**Public boundaries: seven items, none self-accepted**, enumerated in the record and drafted verbatim-landable there. The sharpest are `LocalCoordinates` carrying a participant *space* rather than a *range*, `StagedSpan` carrying a stride vector, the resulting loss of `Copy` on four public types, and `CooperativeTile::addressed_slots` changing to by-reference parameters — a breaking signature change and therefore always-ask under ADR 0075 however mechanical it is.

**Carrier filed.** [`land-the-two-dimensional-staging-relation-adr`](land-the-two-dimensional-staging-relation-adr.md) takes `contracts/decisions`, `contracts/navigation`, and `research/scheduling`, which this ticket's scopes cannot reach, and carries two catalog rows — the research record has never had one.

**What remains on this ticket** is the implementation and identity wave, which is a separate serialized dispatch after Tom rules on the drafted boundary, and which cannot share a wave with any other pinned-identity work.
