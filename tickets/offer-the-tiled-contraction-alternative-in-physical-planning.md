---
id: offer-the-tiled-contraction-alternative-in-physical-planning
title: Offer the tiled contraction alternative in physical planning
status: todo
priority: p1
dependencies: [decide-the-contraction-tile-width-authority]
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, scheduling, cost-model, contraction]
---
## User-visible outcome

A caller whose request reaches a strict `f32` tensor contraction is offered the tiled cooperative alternative alongside the direct fold, and the cost model can actually choose it — so the realization the schedule, lowering, and Metal emission already support is reachable through planning rather than only through a hand-built region.

## Why this exists

Filed 2026-08-22 by the coordinator as the enumerated remainder of `realize-the-tiled-contraction-schedule-and-its-metal-emission`, which landed the IR and Metal halves and stopped at the compiler boundary. Splitting it here lets that ticket close so its dependents proceed, per AGENTS.md.

**Fact — the cost model cannot score the topology, so a tiled plan would be offered and never chosen.** `crates/tiler-compiler/src/measured_cost.rs` declares `fn work_span` and its match ends in a bare `_ => None` arm. `ReductionTopology::CooperativeWorkgroup` has an explicit arm above it; `CooperativeContraction` does not, so it falls to the wildcard and scores `None`. Verified by the coordinator at `b3c07259`.

**Fact — the wildcard is itself the hazard.** Because the final arm is `_ => None` rather than an exhaustive match, a topology added to the vocabulary silently scores `None` instead of failing the build. AGENTS.md prefers splitting genuine complexity over wildcard matches that weaken exhaustiveness. Repairing the arm and removing the wildcard are the same work and should land together.

**Fact — output binding verification is pinned to one region.** `crates/tiler-compiler/src/physical.rs` uses `RegionId::new(0)` at several sites; the tiled lane reports `verify_region_output_binding` must widen past it. Re-derive this rather than trusting the count.

**Blocked on a decision, not on effort.** The tile width has no authority — see [`decide-the-contraction-tile-width-authority`](decide-the-contraction-tile-width-authority.md). Do not hard-code the measured 16 to make progress; the same precedent that refuses a defaulted tree width refuses that.

## Required work

- Re-audit all three Facts at your own base and report a per-Fact verdict; re-derive each population rather than trusting the counts, and say which unit you report.
- Add the `CooperativeContraction` arm to `work_span` and **remove the wildcard**, so a future topology is a build error at this site rather than a silent `None`.
- Widen `verify_region_output_binding` past the single region, with the widened case tested.
- Offer the tiled alternative from the accepted width authority, never from a literal.
- Perturb each new behaviour separately, subject not assertion, with quoted failure text. Include one negative control that the direct fold is still offered and still chosen where it should be.

## Non-goals

Dispatching on a device — [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) owns that; changing the landed schedule, lowering, or Metal emission; and choosing the tile-width authority.

## Closes when

A strict contraction request is offered both alternatives, the cost model scores both, the wildcard arm is gone, the widened output-binding check is tested, each new behaviour has been watched failing on its own subject, and the workspace gate is green.
