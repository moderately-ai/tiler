---
id: offer-the-tiled-contraction-alternative-in-physical-planning
title: Offer the tiled contraction alternative in physical planning
status: todo
priority: p1
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, scheduling, cost-model, contraction]
---
## User-visible outcome

The cost model can score a cooperative contraction at all, and the output-binding check reaches past one region — the two things that must be true before any tiled alternative can be compared against the direct fold.

## Narrowed 2026-08-22, and the dependency I gave it was backwards

I filed this ticket depending on [`decide-the-contraction-tile-width-authority`](decide-the-contraction-tile-width-authority.md). **That edge was wrong and has been removed.** The tile-width packet showed the ordering runs the other way: `work_span` has no `CooperativeContraction` arm, so *no tile width can be compared against another until the cost model can score the topology at all*. A width granted today would select a plan that still could not be chosen.

So this ticket now owns only the three **width-independent** repairs below and is dispatchable immediately. The width-dependent offer moved to [`offer-the-tiled-contraction-alternative-once-a-width-authority-exists`](offer-the-tiled-contraction-alternative-once-a-width-authority-exists.md), which depends on the authority. Do **not** offer a tiled alternative here.

## Why this exists

Filed 2026-08-22 by the coordinator as the enumerated remainder of `realize-the-tiled-contraction-schedule-and-its-metal-emission`, which landed the IR and Metal halves and stopped at the compiler boundary. Splitting it here lets that ticket close so its dependents proceed, per AGENTS.md.

**Fact — the cost model cannot score the topology, so a tiled plan would be offered and never chosen.** `crates/tiler-compiler/src/measured_cost.rs` declares `fn work_span` and its match ends in a bare `_ => None` arm. `ReductionTopology::CooperativeWorkgroup` has an explicit arm above it; `CooperativeContraction` does not, so it falls to the wildcard and scores `None`. Verified by the coordinator at `b3c07259`.

**Fact — the wildcard is itself the hazard.** Because the final arm is `_ => None` rather than an exhaustive match, a topology added to the vocabulary silently scores `None` instead of failing the build. AGENTS.md prefers splitting genuine complexity over wildcard matches that weaken exhaustiveness. Repairing the arm and removing the wildcard are the same work and should land together.

**Fact — output binding verification is pinned to one region.** `crates/tiler-compiler/src/physical.rs` uses `RegionId::new(0)` at several sites; the tiled lane reports `verify_region_output_binding` must widen past it. Re-derive this rather than trusting the count.

**Not blocked.** Nothing below needs a tile width. If you find yourself wanting one, you have strayed into the successor ticket — stop and say so.

## Required work

- Re-audit all three Facts at your own base and report a per-Fact verdict; re-derive each population rather than trusting the counts, and say which unit you report.
- Add the `CooperativeContraction` arm to `work_span` and **remove the wildcard**, so a future topology is a build error at this site rather than a silent `None`.
- Widen `verify_region_output_binding` past the single region, with the widened case tested.
- Perturb each new behaviour separately, subject not assertion, with quoted failure text. Include one negative control that the direct fold is still scored and still chosen where it should be.

## Non-goals

Dispatching on a device — [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns that; changing the landed schedule, lowering, or Metal emission; and choosing the tile-width authority.

## Closes when

The cost model scores a cooperative contraction, the wildcard arm is gone, the widened output-binding check is tested, each new behaviour has been watched failing on its own subject, and the workspace gate is green.
