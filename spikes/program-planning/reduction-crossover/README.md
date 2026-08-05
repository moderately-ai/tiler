---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.reduction-crossover"
kind: "experiment"
title: "Where a parallel-reduction crossover could be measured"
topics: ["program-planning", "scheduling", "reductions", "feasibility", "metal"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger"]
entrypoints: ["spikes/program-planning/reduction-crossover/src/main.rs"]
last_verified: "2026-08-02"
ticket: "calibrate-and-activate-parallel-reduction-selection"
---

# Where a parallel-reduction crossover could be measured

This spike answers the question `calibrate-and-activate-parallel-reduction-selection` has to settle before a timing harness is worth building: **over which shapes does the authoritative Apple profile retain all three reduction alternatives at once?** A crossover is a shape at which the winner changes, so it needs at least two shapes on which every alternative exists and can be timed.

The answer is one shape. There is no crossover to measure, and the reason is a target-profile row rather than a property of the strategies.

## What it drives

For each `(rows, contributors)` in a 3x12 matrix it builds the same program shape the parallel-portfolio test uses — an elementwise multiply-add prologue feeding a sum over the trailing axis — compiles it under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` against `BoundMetalCompileDeclaration::first_macos_apple9`, and records one row per shape: whether a portfolio was produced, how many alternatives it retained, which strategies those were, which one selection chose, and the refusing predicate when there was none.

It is compile-only. It emits no MSL, links nothing, and dispatches nothing, because the domain question is decided entirely at compile-phase feasibility and never reaches the device. That is not a shortcut — it is the finding: the shapes a timing harness would need are refused before any kernel exists.

The three strategies are told apart by the same device-independent structure the realization work used, and by structure rather than by name: the multi-pass split is the alternative with three kernels, the single-workgroup tree is the one whose widest declared workgroup exceeds one thread, and the serial fold is the one with neither.

## Running it

```sh
cd spikes/program-planning/reduction-crossover
cargo run --release
```

`TILER_SWEEP_FULL_EXPLAIN=1` replaces the refusal column with the whole rendered explain report, which is what to do when a refusal lands on a predicate the filter does not name.

No `make` target reaches here, per `spikes/README.md`.

## Result

**Measurement, 2026-08-02**, retained at [`results/2026-08-02-apple-m4-max-macos27.0-26A5388g/sweep.tsv`](results/2026-08-02-apple-m4-max-macos27.0-26A5388g/sweep.tsv). Host: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max — matching the ledger's execution-environment row in every field. Toolchain: `nightly-2026-07-19`, Xcode 26.6 (17F113), SDK 26.5 (25F70), offline compiler `Apple metal version 32023.883`. Only the compile phase runs, so the toolchain row is recorded for provenance rather than because a kernel was built.

Of 36 shapes, **exactly one retains all three alternatives: `rows=1, contributors=4`.** On it, selection chooses the serial fold. Every other shape falls into one of two classes:

- **Refused by hard feasibility on the grid axis.** `event=feasibility:grid-axis:rejected:target-infeasible:threads=<required>:4`, at subject `region:region:pointwise`. The bound is 4 and the requirement is the prologue's work items, so every shape with more than four elements is refused. Wide shapes report two distinct rejections, the prologue's and the split's partial pass's.
- **Refused by a known defect before feasibility is reached.** Contributor counts admitting no balanced exact partition — below four, and primes, observed at 5 — fail the whole batch with `InvalidCompilerOutput`. That is [`correct-the-declined-strategy-record-for-an-unsplittable-reduction`](../../../tickets/correct-the-declined-strategy-record-for-an-unsplittable-reduction.md), not a statement about the measurable domain. Two things this sweep adds to that ticket's record are noted on it.

## The single point is forced by arithmetic, not by the sample

The sweep confirms a result that does not depend on which shapes were sampled. Two constraints bound the domain from opposite sides:

- `governed_partition` returns `None` below four contributors, so **both** parallel strategies require `contributors >= 4`. Below that the portfolio holds the serial fold alone and there is nothing to compare it against.
- The profile's `GridAxisThreads` row is 4, and the prologue launches one invocation per element, so a plan exists only where `rows * contributors <= 4`.

Since `rows >= 1`, the two give `4 <= contributors <= rows * contributors <= 4`, which forces `rows = 1` and `contributors = 4`. **The domain is a single point by derivation.** Sampling more shapes cannot enlarge it.

The workgroup axis does not vary independently either: the tree's participant count is `governed_partition(4)`, which is a balanced exact split into 2 partitions of 2. One shape, one workgroup width, one point.

## Why no timing run followed

A crossover, a calibration, and a held-out prediction each need at least two points; a fit through one point is not a model and a prediction from it is not evidence. So no amount of timing precision at `1x4` could close any of the owning ticket's requirements, and the performance loop correctly stops before measuring rather than producing a number that would have to be caveated into uselessness.

**Inference, stated so it can be refuted.** Even the one available point has no discriminating power: at four contributors the arithmetic is a handful of operations, so any wall-clock difference between the three alternatives would be dominated by dispatch and submission overhead rather than by the reduction strategy. This is reasoning about magnitudes, not a measurement — nothing here timed anything — but it is why widening the domain, not instrumenting the point, is the work that unblocks calibration.

## What would reopen this

The blocking row is `grid_axis_threads: 4` in `crates/tiler-build/src/metal_declaration.rs`, and its own comment records that it is **a deliberately conservative compile guarantee rather than a maximum**: the macOS 26.5 SDK's `dispatchThreads:` contract proves extent 4 is representable and establishes no upper bound at all. So the row is not a hardware limit that measurement would confirm — it is a floor awaiting an authority that states a real one.

Raising it needs a new normative source or a retained measurement, which is target-profile authority work in `research/target-profiles` and `implementation/build`. It is filed as [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](../../../tickets/establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md). When that row admits a wider grid, rerun this sweep first: it reports the new domain, and only a domain with at least two points makes a timing harness worth writing.

## Boundary

- One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract (`FLUSH_AND_REASSOCIATE_F32`), one program family (multiply-add prologue into a trailing-axis sum), `f32` only.
- The result is about **which plans exist**, not about how fast any of them runs. Nothing here was dispatched, so this spike makes no performance claim of any kind.
- The `InvalidCompilerOutput` rows are a defect's signature, not a domain boundary. If that defect is fixed, those shapes will report their real feasibility outcome, and the derivation above says the ones with more than four work items will still be refused on the grid axis.
