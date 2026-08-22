---
id: offer-the-tiled-contraction-alternative-in-physical-planning
title: Offer the tiled contraction alternative in physical planning
status: in-progress
priority: p1
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, integrate-the-contraction-vertical-into-the-runtime]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, scheduling, cost-model, contraction]
claimed_from: todo
assignee: worker-costarm
lease_expires_at: 1787429727
---
## User-visible outcome

The cost model can score a cooperative contraction at all, and the output-binding check reaches past one region — the two things that must be true before any tiled alternative can be compared against the direct fold.

## Narrowed 2026-08-22, and the dependency I gave it was backwards

I filed this ticket depending on [`decide-the-contraction-tile-width-authority`](decide-the-contraction-tile-width-authority.md). **That edge was wrong and has been removed.** The tile-width packet showed the ordering runs the other way: `work_span` has no `CooperativeContraction` arm, so *no tile width can be compared against another until the cost model can score the topology at all*. A width granted today would select a plan that still could not be chosen.

So this ticket now owns only the three **width-independent** repairs below and is dispatchable immediately. The width-dependent offer moved to [`offer-the-tiled-contraction-alternative-once-a-width-authority-exists`](offer-the-tiled-contraction-alternative-once-a-width-authority-exists.md), which depends on the authority. Do **not** offer a tiled alternative here.

## Why this exists

Filed 2026-08-22 by the coordinator as the enumerated remainder of `realize-the-tiled-contraction-schedule-and-its-metal-emission`, which landed the IR and Metal halves and stopped at the compiler boundary. Splitting it here lets that ticket close so its dependents proceed, per AGENTS.md.

**Fact — the cost model cannot score the topology.** `crates/tiler-compiler/src/measured_cost.rs` declares `fn work_span` and its match ends in a bare `_ => None` arm. `ReductionTopology::CooperativeWorkgroup` has an explicit arm above it; `CooperativeContraction` does not, so it falls to the wildcard and scores `None`. Verified by the coordinator at `b3c07259` and re-verified by `worker-costarm` at `e1ada851`; the perturbation that deletes the repaired arm reproduces it as `left: None`.

**Repaired 2026-08-22 — the consequence stated here was wrong twice, and the source documented it correctly all along.** This Fact first read "so a tiled plan would be offered and never chosen"; a coordinator correction mid-ticket replaced that with "never offered". *Both are false.* Traced at `e1ada851`: `assess_fold_steps` propagates the decline with `?`, and `pipeline::planning::measured_scores` then collects `Option<Vec<_>>` over **every** retained alternative, so one declining alternative collapses the measured comparison for *all* of them and `select_non_dominated` falls back to the structural Pareto view for the whole target. The declining plan is neither withheld nor outranked — it stays offered and selectable through `portfolio.non_dominated()`. `stage_work_span`'s own doc comment already said this: `a declined stage declines the whole plan, a declined plan declines the whole comparison`. The defect is therefore *worse* than either earlier wording: the retained sweep measured the structurally dominant serial fold up to 50.7x slower than the best parallel plan, and that fallback silently re-enters exactly that regime for neighbouring plans that scored fine.

**Fact — the wildcard is itself the hazard.** Because the final arm is `_ => None` rather than an exhaustive match, a topology added to the vocabulary silently scores `None` instead of failing the build.

**Repaired 2026-08-22 — the wildcard cannot be removed, and this Fact's prescription was impossible.** `ReductionTopology` and `ReductionPass` are both `#[non_exhaustive]` (`crates/tiler-ir/src/schedule/model.rs`, under `ADR 0074 convention 5a`), and `measured_cost.rs` is outside their defining crate, so rustc *requires* a wildcard. `work_span`'s own doc comment said so before this ticket was filed: `so the wildcard arms are required rather than chosen`. Deleting the arm and building gives `error[E0004]: non-exhaustive patterns` with the note `ReductionTopology is marked as non-exhaustive, so a wildcard _ is necessary to match exhaustively`. The hazard the Fact names is real, so it is discharged the way AGENTS.md prescribes for an untypable population — `variant_count`-sized censuses (`every_reduction_topology_states_a_verdict`, `every_reduction_pass_states_a_verdict`), one per independently-widening vocabulary, which make a widened enum a build error at the census rather than a silent `None`.

**Fact — output binding verification is pinned to one region.** `crates/tiler-compiler/src/physical.rs` uses `RegionId::new(0)` at several sites; the tiled lane reports `verify_region_output_binding` must widen past it.

**Re-derived 2026-08-22 at `e1ada851` — true in substance, imprecise in count.** Unit: occurrences of the literal `RegionId::new(0)`. The file carries **17** (one per line, so `grep -c` and `grep -o | wc -l` agree here). Only **4** are inside `verify_region_output_binding`, and only **one** — the arm pairing `NormalizedOutputSubject::Contraction` with `ScalarProgram::StrictTensorContraction` — is a pin a tiled contraction could ever reach; the other three belong to the pointwise and serial-sum arms. The remaining 13 are construction sites and test fixtures. The region identifiers are a **role vocabulary**, not sequential numbering: 0 whole-program/prologue/fused, 1 materialized fold, 2 split partial, 3 split final, 4 workgroup tree, 5 epilogue, 6 publishing copy, 7 staged fold, 8 staged pass. Widening therefore means giving the tiled realization an identifier and a binding rule of its own, which is what `CooperativeWorkgroup` already has relative to the materialized fold.

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
