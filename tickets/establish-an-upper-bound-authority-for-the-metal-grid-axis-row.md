---
id: establish-an-upper-bound-authority-for-the-metal-grid-axis-row
title: Establish an upper-bound authority for the Metal grid-axis row
status: todo
priority: p1
dependencies: []
related: [calibrate-and-activate-parallel-reduction-selection]
scopes: [research/target-profiles, implementation/build, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [research, target-profiles, measurement]
---
## User-visible outcome

The authoritative macOS Metal profile declares a grid-axis bound established by a real upper-bound authority rather than by a conservative representability floor, so programs wider than four elements can compile and the reduction strategies become comparable on more than one shape.

## Why this exists

**Measurement, 2026-08-02 — the current row is a floor, and it collapses the parallel-reduction measurable domain to a single point.** [The retained sweep](../spikes/program-planning/reduction-crossover/README.md) compiled a reduction program family across 36 shapes against `tiler.metal.macos-apple9.msl4-0.f32.v1` under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, on a host matching the ledger's execution-environment row in every field. Exactly one shape retains all three reduction strategies at once: one row of four contributors. Every wider shape is refused by hard feasibility with `event=feasibility:grid-axis:rejected:target-infeasible:threads=<required>:4` at the pointwise prologue.

**Fact — the row says of itself that it is not a maximum.** `crates/tiler-build/src/metal_declaration.rs:185-188` declares `grid_axis_threads: 4` with the comment: the macOS 26.5 SDK's `dispatchThreads:` contract "proves extent 4 is representable and establishes no upper bound at all, so 4 is a deliberately conservative compile guarantee rather than a maximum." The compiler-side governed profile carries the same four with the same reasoning (`crates/tiler-compiler/src/target.rs`, `TargetProfileBuilder::governed`), citing `MTLComputeCommandEncoder.h` and `MTLTypes.h` as proving representability and explicitly not proving 65,535, an Apple-family maximum, or any prepared pipeline's capacity.

**Inference — this is an absent authority, not a hardware limit.** No measurement would confirm four; the row is what the primary sources happen to prove, and they prove a lower bound. So raising it is a question of finding the authority that states a real maximum, at the right phase, rather than of running a probe against the current one.

## What blocks on it

- [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md) cannot establish a crossover: a crossover needs at least two shapes on which the strategies coexist, and the domain is one point. The single point is forced by arithmetic — `governed_partition` withholds both parallel strategies below four contributors and the grid axis caps `rows * contributors`, so `4 <= contributors <= rows * contributors <= bound`.
- `target::tests::only_one_shape_admits_all_three_reduction_strategies` fails when this row widens, which is the designed signal that calibration has become possible.

## Implementation keys

Establish the authority first and the number second. Candidate authorities, each to be accepted or eliminated with the ground stated: an Apple feature-table row stating a maximum grid size per dimension; an SDK header or specification sentence bounding `MTLSize` dimensions for `dispatchThreads:`; or a retained device measurement, which qualifies a bounded profile rather than a portable guarantee and must be declared through the measured source with its execution environment attached.

Do not raise the number without moving the authority with it. A widened bound carrying the existing representability citation would be the citation saying something it does not say, which is exactly what the authority ledger exists to prevent.

Identity moves when the row moves: the profile's canonical descriptor is encoded into artifact identity and the cache subject, so every pinned identity must be recomputed on the tree the change lands into, with each moved pin enumerated.

## Closes when

The grid-axis row cites an authority that states an upper bound, the ledger records which class that authority is and what it does not cover, the descriptor and every pinned identity move together in one commit, and the reduction-domain trigger test reports the new domain. Reruning the retained sweep records what became measurable.
