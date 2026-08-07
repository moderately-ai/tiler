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
last_verified: "2026-08-05"
ticket: "calibrate-and-activate-parallel-reduction-selection"
---

# Where a parallel-reduction crossover could be measured

This spike answers the question `calibrate-and-activate-parallel-reduction-selection` has to settle before a timing harness is worth building: **over which shapes does the authoritative Apple profile retain all three reduction alternatives at once?** A crossover is a shape at which the winner changes, so it needs at least two shapes on which every alternative exists and can be timed.

**The retained answer, measured 2026-08-02, is one shape** — no crossover was measurable, and the reason was a target-profile row rather than a property of the strategies. **That row has since moved, and the domain opened**: see [What reopened this](#what-reopened-this-the-row-is-a-measurement-now) below. Everything between here and that section describes the profile as it stood on 2026-08-02 and is kept as the record of it; nothing in it is a claim about the profile today.

## What it drives

For each `(rows, contributors)` in a 3x12 matrix it builds the same program shape the parallel-portfolio test uses — an elementwise multiply-add prologue feeding a sum over the trailing axis — compiles it under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` against `BoundMetalCompileDeclaration::first_macos_apple9`, and records one row per shape: whether a portfolio was produced, how many alternatives it retained, which strategies those were, which one selection chose, and the refusing predicate when there was none.

It is compile-only. It emits no MSL, links nothing, and dispatches nothing, because the domain question is decided entirely at compile-phase feasibility and never reaches the device. In 2026-08-02 that was not a shortcut but the finding itself: the shapes a timing harness would have needed were refused before any kernel existed. Against the measured row it is a shortcut again, and a correct one — the harness those shapes now permit is a separate, dispatching thing that this sweep sizes rather than replaces.

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

## The single point was forced by arithmetic, not by the sample

Under the profile as it stood on 2026-08-02, the sweep confirmed a result that did not depend on which shapes were sampled. Two constraints bounded the domain from opposite sides:

- `governed_partition` returns `None` below four contributors, so **both** parallel strategies require `contributors >= 4`. Below that the portfolio holds the serial fold alone and there is nothing to compare it against. **This half still holds.**
- The profile's `GridAxisThreads` row was 4, and the prologue launches one invocation per element, so a plan existed only where `rows * contributors <= 4`. **This half is the one that moved.**

Since `rows >= 1`, the two gave `4 <= contributors <= rows * contributors <= 4`, which forced `rows = 1` and `contributors = 4`. **The domain was a single point by derivation**, and sampling more shapes could not have enlarged it. The derivation is not wrong; its second premise is simply no longer the profile's row.

The workgroup axis did not vary independently either: the tree's participant count is `governed_partition(4)`, a balanced exact split into 2 partitions of 2. One shape, one workgroup width, one point.

## Why no timing run followed in 2026-08-02

A crossover, a calibration, and a held-out prediction each need at least two points; a fit through one point is not a model and a prediction from it is not evidence. So no amount of timing precision at `1x4` could have closed any of the owning ticket's requirements, and the performance loop correctly stopped before measuring rather than producing a number that would have to be caveated into uselessness.

**Inference, stated so it can be refuted, and since confirmed by measurement.** Even the one available point had no discriminating power: at four contributors the arithmetic is a handful of operations, so any wall-clock difference between the three alternatives would be dominated by dispatch and submission overhead rather than by the reduction strategy. This is reasoning about magnitudes, not a measurement — nothing here timed anything — but it is why widening the domain, rather than instrumenting the point, was named as the work that unblocks calibration.

**Measurement, 2026-08-07 — the timing harness now exists, and it confirms the inference at exactly this shape.** [`reduction-dispatch-crossover`](../reduction-dispatch-crossover/README.md) dispatches all three alternatives across a 92-cell matrix on the qualified host. At `1x4` the fold and the tree land within 1% of one another and no pair of the three is separated from the noise, so no crossover could have been read there whatever the profile admitted. Wide shapes are a different matter: the serial fold is 50.7 times slower than the best parallel strategy at four rows of 8,192 contributors, and 1.78 times faster at 16,384 rows of four.

## What reopened this: the row is a measurement now

**Fact, 2026-08-04.** The blocking row was `grid_axis_threads: 4` in `crates/tiler-build/src/metal_declaration.rs`, which its own comment described as a deliberately conservative compile guarantee rather than a maximum — a floor awaiting an authority that stated a real one. [`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](../../../tickets/establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md) found that **no normative source can fill that row at all**: the row is consumed as a *guarantee*, so its authority has to state a floor on capability, while every available Metal source states a ceiling on the space. It is now a bounded measurement at **268,435,456**, carried by [the retained extent ladder](../../target-profiles/metal-grid-axis-extent/README.md) and recorded in the [authority ledger](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md).

**Measurement, 2026-08-05 — this sweep was rerun against the moved row, and the result is not retained here.** Rerun unchanged, it reports **24 of 36 shapes retaining all three strategies (was 1), with zero grid-axis refusals (was 23)**, and every one of the 23 previously grid-axis-refused shapes is now in the domain. The remaining twelve are the contributor counts admitting no balanced exact partition; those previously failed the whole batch with `InvalidCompilerOutput` and now return a portfolio, which is [`correct-the-declined-strategy-record-for-an-unsplittable-reduction`](../../../tickets/correct-the-declined-strategy-record-for-an-unsplittable-reduction.md) (done) rather than anything the row changed. The rerun is recorded in full on the grid-axis ticket's outcome.

**Why no `results/` directory accompanies that paragraph, stated so the absence is not read as an oversight.** A retained results directory is a positive measurement claim carrying its own environment, provenance, and boundary, and it belongs to the ticket that owns the measurement. The grid-axis ticket ran this rerun read-only and recorded exactly that — a new result under `spikes/program-planning/` is the calibration ticket's to retain — and this spike's own `ticket:` field already names [`calibrate-and-activate-parallel-reduction-selection`](../../../tickets/calibrate-and-activate-parallel-reduction-selection.md), whose next act was to rerun this sweep as the first step of the timing work the widened domain unblocks. Retaining a directory here under the documentation-correction ticket that wrote this paragraph would attach a measurement to a ticket that ran no harness and recorded no execution environment. **So the 2026-08-02 directory is the only retained result, and it remains the record of the superseded row rather than of the current one.**

**Measurement, 2026-08-07 — that rerun happened, agreed exactly, and the result it unblocked is retained next door.** Rerun unchanged on a host matching the ledger row in every field, this sweep again reports **24 of 36 shapes retaining all three strategies, with no grid-axis refusal at any shape**, and the same twelve contributor counts retaining the serial fold alone. The rerun is still not retained here — a compile-phase domain count is what this document already states, and a second directory holding the same answer would be a second claim about the same thing — while the timing evidence it authorized is retained at [`reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`](../reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/).

## Boundary

- One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract (`FLUSH_AND_REASSOCIATE_F32`), one program family (multiply-add prologue into a trailing-axis sum), `f32` only.
- The result is about **which plans exist**, not about how fast any of them runs. Nothing here was dispatched, so this spike makes no performance claim of any kind.
- The `InvalidCompilerOutput` rows are a defect's signature, not a domain boundary. That defect is now fixed and those rows report their real outcome: a portfolio holding the serial fold alone, because the contributor counts that produced them admit no balanced exact partition. The rest of that bullet's prediction — that shapes above four work items would still refuse on the grid axis — was made under the superseded row and is falsified by the 2026-08-05 rerun, which observed no grid-axis refusal at any shape.
- Both retained result classes describe the profile of 2026-08-02. The 2026-08-05 rerun above is stated as a count, not retained, and is therefore weaker evidence than the `sweep.tsv` beside it; a reader who needs the per-shape rows against the measured row has to rerun.
