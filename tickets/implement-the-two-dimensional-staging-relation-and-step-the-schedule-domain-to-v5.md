---
id: implement-the-two-dimensional-staging-relation-and-step-the-schedule-domain-to-v5
title: Implement the two-dimensional staging relation and step the schedule domain to v5
status: todo
priority: p1
dependencies: [land-the-two-dimensional-staging-relation-adr]
related: [admit-a-two-dimensional-cooperative-staging-relation, admit-a-round-dependent-cooperative-staging-span, realize-the-tiled-contraction-schedule-and-its-metal-emission, realize-the-strict-contraction-on-metal]
scopes: [implementation/ir, contracts/artifacts, implementation/build, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, physical-planning, identity, schedule-version]
---
## User-visible outcome

A cooperative tile states a staged access over a two-dimensional participant space, and every identity that folds a scheduled region is recomputed on the tree the step lands into — so no pinned value in the repository is a `v4` answer wearing a `v5` label.

## Why this is a separate ticket

[`admit-a-two-dimensional-cooperative-staging-relation`](admit-a-two-dimensional-cooperative-staging-relation.md) ran the elimination, derived the relation, and drafted the public boundary; that is its whole delivered outcome and it is closed on it. What it deliberately did **not** do is touch `crates/`: its central act is an identity-domain step at a public boundary, and [AGENTS.md](../AGENTS.md) reserves that boundary for Tom. This ticket is that remainder, and it does not start until the boundary is accepted.

**Do not re-derive the design.** [The two-dimensional cooperative staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md) is the authority for the shape, the round-ordinal separation, the ADR 0096 interaction, the decidability argument, and the blast radius. Read it in full first.

## The identity step is executed whole or not at all

This is the failure mode the whole ticket is shaped around: a stepped version over unmoved pins is a lie the next reader builds on. AGENTS.md's discipline, applied to the enumeration the record already produced:

- The version moves at its owning layer — `crates/tiler-ir/src/schedule/model.rs:1878`, `b"tiler.schedule.v4\0"` → `v5`. No domain above it steps; the kernel identity frames the scheduled-region bytes whole at `crates/tiler-ir/src/kernel/model.rs:1757`, so the reach is a fold rather than a second version.
- The ledger documents move **in the same commit**. `docs/artifact-abi.md:207`, `:213`, and `:215` are under `contracts/artifacts`, which is why this ticket holds that scope alongside `implementation/ir` — the record notes that splitting them is how the step lands in halves.
- Every pin in the record's **class A** is recomputed on the merged tree and **enumerated by name in the report**: `schedule/builder.rs:1683` and `:1691`, `tiler-build/src/metal_plan.rs:840, 842, 858, 860`, and all six `crates/tiler-metal/goldens/*.metal` at lines 35/36/37/41-42. Five of the six carry no cooperative tile and move anyway, for the eighteen separator bytes alone.
- Class **C** must **not** move — those are dated measurements, and rebaselining one falsifies a record rather than maintaining a pin. Class **D** requires nothing; it is emitters and closed records. Treating the grep output as a task list would edit an emitter and three closed records.
- Recompute each value from an observed run, never by picking a side or by copying a sibling branch's answer.

## The one line that needs a decision, not a recomputation

`STRICT_F32_REGION_IDENTITY_HEX_V3` (`crates/tiler-ir/src/schedule/builder.rs:1691`) exists to make a step's blast radius a measured fact. Carried forward unchanged, `the_round_step_moves_only_the_domain_separator` would compare `v5` against `v3` — a two-step claim, which is strictly weaker, because two separator changes agreeing past the tag says nothing about whether the payload moved at either step individually. The record's proposal is to rebaseline the constant to the `v4` value and rename it, moving the test's name and doc with it, so the retained comparison keeps proving exactly one step. That discards the `v3` datum deliberately: its whole content was the `v3 → v4` claim, which the commit that made it already carries. Adopt or refute it explicitly — do not carry it forward silently.

## Required evidence

- The four accesses of the measured 16×16 kernel (`spikes/scheduling/metal_contraction_vertical/kernels.metal:116-133`) are all statable, with contiguous counts, under the strides the record's table gives.
- Disjointness and coverage are still decided by enumerating addressed slots under `MAX_COOPERATIVE_PARTICIPANTS` and `MAX_COOPERATIVE_STAGING_SLOTS`. The occupancy map must still refuse two writers reaching one slot in one round — the record's own case is `strides = [16, 16]`, which sends `(0,1)` and `(1,0)` both to slot 16. Watch it refuse.
- `verify_cooperative_tile`'s workgroup-width equality (`builder.rs:1147`) generalizes from `participants.count == threads_per_workgroup` to `∏ extents == threads_per_workgroup` and still fires. Perturb the extents and watch it.
- A read span whose stride vector disagrees with the launch geometry is refused. Note that today a wrong divisor on a *read* is admitted — `builder.rs:1258` calls `addressed_slots` and discards its result — and the record cites that as a defect class the widening must not inherit.
- The full gate is run after the version bump and **the failure set is compared against class A**. That comparison is the check that closes the enumeration's stated residual risk: a digest pinned in a form none of the seven grep patterns reach (a base64 or byte-array literal, or a golden symbol spelled without the `tiler_kernel_` prefix). A failure outside class A is a pin the enumeration missed and must be reported, not quietly fixed.

## Explicit non-goals

The round-dependent span and the per-access active-participant subset — [`admit-a-round-dependent-cooperative-staging-span`](admit-a-round-dependent-cooperative-staging-span.md) owns them at `deferred`, and the record derives why they are a second relation rather than a wider parameter of this one. ADR 0096's items 1, 4, 5, 6, and 7 stay where they are; item 6's `0x36` appends-only argument must be re-made at the encoding site on the `v5` tree if it lands after this step.

## Boundary

Do not start before [`land-the-two-dimensional-staging-relation-adr`](land-the-two-dimensional-staging-relation-adr.md) is done — that is Tom's acceptance of the exact spelling, and the drafted boundary includes a breaking signature change (`CooperativeTile::addressed_slots` going by-reference) and the loss of `Copy` on four public types. Tom's 2026-08-01 acceptance of the step *in principle* is a relayed fact recorded in the producing ticket, and it explicitly does not accept the spelling.

## Closes when

A two-dimensional staged access is statable, every new rule has been watched refusing its own defect, the identity step is complete — version moved at its owning layer, ledger moved in the same commit, every class A pin recomputed on the merged tree and enumerated in the report — and the post-bump gate's failure set has been compared against class A with any residual reported.

## Graph maintenance

- Filed 2026-08-02 at integration of the producing ticket, so that ticket could close on its delivered outcome rather than deadlock the graph in `review`.
- If the post-bump gate reveals a pin outside class A, file the enumeration gap as its own defect against the record rather than absorbing it silently.
