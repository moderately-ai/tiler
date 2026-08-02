---
id: calibrate-and-activate-parallel-reduction-selection
title: Calibrate and activate parallel reduction selection
status: blocked
priority: p1
dependencies: [realize-parallel-reduction-strategies-on-metal, establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/compiler, research/program-planning, contracts/optimizer]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: []
claimed_from: todo
assignee: agent-calibrate
lease_expires_at: 1785697181
---
## User-visible outcome

Target-aware selection chooses serial, single-workgroup, or multi-pass reduction from measured cost evidence and hard feasibility, so larger reductions stop serializing by default only where the qualified profile demonstrates that choice is faster.

## The measured pair, corrected

**Fact — the stated measurement target named a pair that could not both hold, and now can.** "The three retained alternatives on the exact qualified Metal environment" was unreachable: a split and a single-workgroup tree each consume ordered regrouping, the measured Apple `f32` row flushes subnormals in every math mode, and none of the four registered contracts both flushed and permitted regrouping — so on that environment only the serial fold was ever retained and there were no three alternatives to measure. `compose-the-numerical-contract-from-its-decided-dimensions` closed that: the contract is composed from its dimensions, and `NumericalContract::FLUSH_AND_REASSOCIATE_F32` is the named point that resolves both.

**The measurement target is therefore the pair, stated together:** the three retained alternatives under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, against `BoundMetalCompileDeclaration::first_macos_apple9`, on the qualified Metal host. Any other contract on that declaration retains fewer than three alternatives and cannot supply a crossover; `crates/tiler-build/src/metal_plan.rs`'s `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` is what pins that the pair reaches a portfolio at all, and it also records the shape constraint the fixture ran into — this profile's grid axis admits four threads, so the widest stage of the measured program has to fit it.

## Implementation keys

Measure the three retained alternatives over a predeclared shape/workgroup matrix on the exact qualified Metal environment, under the contract named above. Fit or select the smallest analytical calibration that predicts the measured crossover without folding infeasibility into cost. Preserve all close alternatives in the portfolio and explain the measured assumptions and winning terms.

Do not encode an arbitrary preference for parallel plans. Current structural dominance favors fewer dispatches and analytical costs do not participate in dominance; activation must deliberately connect reviewed cost evidence to selection rather than altering a constant until the desired plan wins.

## Required evidence

Retained raw measurements identify stable crossover regions or explicitly report that none was established. Calibration predicts held-out rows within a stated error bound, serial remains selected below its measured crossover, and an unavailable environment makes no performance claim. Perturbing the calibrated term or environment identity changes or refuses the selection evidence.

## Closes when

Selection uses measured target-specific evidence, explain output names why the winning strategy won, no infeasible plan is represented as expensive, every check is mutation-proved, and the performance record plus targeted gates pass.

## Graph maintenance

- Keep this ticket after Metal realization so calibration measures executable strategies rather than synthetic cost constants.
- Close `implement-parallel-reduction-strategies` only after this ticket connects retained measurements to selection and the three-strategy rollup is true on one merged tree.
- File a bounded environment-specific measurement follow-up instead of asserting a crossover when the qualified host or stable region is unavailable.

## Outcome — 2026-08-02: no crossover was established, and the reason is a target row

**This is the "explicitly report that none was established" branch of Required evidence, not a failed attempt at the other one.** Selection is unchanged: all three strategies are enumerated and retained, the serial fold is selected, and no cost-based preference was activated. Activating one would have meant altering a constant until the desired plan won, which the Implementation keys forbid.

### The qualified environment was available and matched

The host matched the ledger's execution-environment row in every field — macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max — under Xcode 26.6 (17F113), SDK 26.5 (25F70), offline compiler `Apple metal version 32023.883`, toolchain `nightly-2026-07-19`. **So the blocker is not an unavailable environment**, and no unavailable-environment predicate is claimed. Nothing here rests on a host being missing.

### Measurement — the measurable domain is one shape

[The retained sweep](../spikes/program-planning/reduction-crossover/README.md) compiled the reduction program family across a 3x12 shape matrix (rows in {1, 2, 4}, contributors in {1, 2, 3, 4, 5, 6, 8, 9, 12, 16, 64, 1024}) against `BoundMetalCompileDeclaration::first_macos_apple9` under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`. Raw rows at `spikes/program-planning/reduction-crossover/results/2026-08-02-apple-m4-max-macos27.0-26A5388g/sweep.tsv`; reproduce with `cargo run --release` from the spike directory.

**Of 36 shapes, exactly one retains all three strategies: `rows=1, contributors=4`,** and on it selection chooses the serial fold. The rest split into two classes:

- **Refused by hard feasibility on the grid axis** — `event=feasibility:grid-axis:rejected:target-infeasible:threads=<required>:4`, at subject `region:region:pointwise`. This settles one Closes-when clause directly: no infeasible plan is represented as expensive. The refusal is a typed predicate naming its axis and both quantities, never a cost.
- **Refused earlier by a known defect** — contributor counts admitting no balanced exact partition fail the batch with `InvalidCompilerOutput`. That is [`correct-the-declined-strategy-record-for-an-unsplittable-reduction`](correct-the-declined-strategy-record-for-an-unsplittable-reduction.md), and two additions to its evidence are recorded as a comment on it rather than absorbed here.

### Why one shape closes the question rather than inviting a longer sweep

The single point is **forced by arithmetic, not found by sampling.** `governed_partition` withholds both parallel strategies below four contributors, and the grid-axis row caps the prologue's one-invocation-per-element launch, so `4 <= contributors <= rows * contributors <= bound`. At a bound of four that chain closes on `(1, 4)`. The workgroup axis does not vary either: the tree's participant count at four contributors is a fixed two-by-two split. So the predeclared shape/workgroup matrix the Implementation keys call for does not exist on this profile, and no larger sweep can create it.

A crossover, a calibration, and a held-out prediction each need at least two points. **No error bound is stated and no held-out row is predicted, because a fit through one point is not a model and any bound quoted from it would be fabricated.** The performance loop therefore stops at its own validity gate rather than producing a number needing a caveat that would negate it.

**Inference, offered to be refuted rather than relied on.** Even the one available point could not discriminate the strategies: at four contributors the arithmetic is a handful of operations, so any wall-clock difference would be dominated by dispatch and submission overhead. Nothing was dispatched and this is reasoning about magnitudes, not a measurement — it is why widening the domain, rather than instrumenting the point, is the unblocking work.

### What landed

- The retained sweep, its results, and its README (`spikes/program-planning/reduction-crossover/`), catalogued in `spikes/README.md`.
- `target::tests::only_one_shape_admits_all_three_reduction_strategies` — the reconsideration trigger made executable. It reads the grid-axis bound from the governed profile rather than hardcoding it, so it fails when the row widens and says calibration is unblocked. Mutation-proved: raising the declared bound to 8 made it fail with the observed domain `[(1, 4), (1, 6), (1, 8), (2, 4)]`, which is what the arithmetic predicts.
- Four doc comments in `crates/tiler-compiler` that named this ticket as owner of "replacing it with measured evidence" now record that the evidence is not obtainable on this profile and name the blocking row — they were making unreachable work look reachable.
- `docs/compiler/fusion-and-scheduling.md` gains the same correction where it defers preference to measured calibration.

### Required evidence, clause by clause

Stated separately because three of the five are **unreachable rather than satisfied**, and a summary that did not separate them would read as a pass.

| Clause | Status |
| --- | --- |
| Retained raw measurements identify stable crossover regions **or explicitly report that none was established** | **Satisfied, on the second branch.** Raw rows retained; none established, with the reason. |
| Calibration predicts held-out rows within a stated error bound | **Unreachable, and deliberately not faked.** No calibration exists, so no bound is stated and no held-out row is predicted. A bound quoted from a single point would be fabricated. |
| Serial remains selected below its measured crossover | **Vacuously true, and observed at the one shape.** There is no measured crossover to be below. At `1x4` the sweep records `selected=serial-fold`, and structural dominance is what selects it — unchanged by this work. |
| An unavailable environment makes no performance claim | **Vacuous, and not claimed as demonstrated.** The environment was available and matched the ledger row in every field, so the unavailable path was never exercised. What holds instead is stronger and was checked: *no* performance claim is made at all, from any environment. |
| Perturbing the calibrated term or environment identity changes or refuses the selection evidence | **Unreachable.** There is no calibrated term, and no selection evidence is derived from environment identity. The analogous perturbation was done on the term that actually bounds the result: the declared grid-axis row was moved from 4 to 8 and the trigger test failed with the widened domain `[(1, 4), (1, 6), (1, 8), (2, 4)]`. |

### Measurement boundary

One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract, one program family (multiply-add prologue into a trailing-axis sum), `f32` only. The result is about **which plans exist**, not how fast any of them runs; nothing was dispatched, so no performance claim of any kind is made. It does not generalize to another Apple family, OS row, dtype, or to any profile with a different grid-axis bound.

### What unblocks this

[`establish-an-upper-bound-authority-for-the-metal-grid-axis-row`](establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md). The blocking row is a **deliberately conservative compile guarantee rather than a hardware maximum** — its own comment records that the SDK contract proves extent four representable and establishes no upper bound at all — so it is an absent authority rather than a limit that measurement would confirm. This ticket's Closes-when is unmet and was not restated to fit what was achievable; the coordinator decides whether it parks behind the new ticket or is superseded by it.
