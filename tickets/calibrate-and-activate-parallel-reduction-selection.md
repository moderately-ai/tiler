---
id: calibrate-and-activate-parallel-reduction-selection
title: Calibrate and activate parallel reduction selection
status: review
priority: p1
dependencies: [realize-parallel-reduction-strategies-on-metal, establish-an-upper-bound-authority-for-the-metal-grid-axis-row]
related: [implement-parallel-reduction-strategies, activate-measured-reduction-selection-from-a-target-cost-row, calibrate-the-reduction-partition-against-measured-alternatives]
scopes: [implementation/compiler, research/program-planning, contracts/optimizer]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: []
claimed_from: todo
assignee: agent-calibrate
lease_expires_at: 1786081232
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

## Outcome — 2026-08-07: the crossover is measured and large; the activation is a public boundary and is parked

**Read this first, because the shape of the close matters.** The 2026-08-02 outcome below closed on *"no crossover was established"* and named a target row as the reason. That row moved, and this run took the measurement it blocked. **A crossover exists, it is a contour rather than a point, and both sides of it are two orders of magnitude apart.** A three-parameter analytical calibration reproduces the measured verdict on held-out shapes within a stated bound, and every check is mutation-proved. What did **not** happen is activation, and the reason is a boundary rather than an evidence gap: consulting the calibration requires a target profile to declare a quantity no profile carries, which is a `pub` surface and an identity move, both reserved for Tom. That surface is designed and filed as [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md), `awaiting-decision`.

### The environment matched the ledger in every field

Verified rather than assumed, and retained at [`environment.tsv`](../spikes/program-planning/reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/environment.tsv). Offline compilation: `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`, Xcode 26.6 (17F113), SDK `macosx` 26.5 (25F70). Execution: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max. Toolchain `nightly-2026-07-19`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`.

**The offline half needed `DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer` and that is load-bearing, not defensive.** This host's default selection is Xcode 27.0 beta (`27A5228h`) with SDK 27.0 and offline compiler `32023.921` — a different environment from the one the profile was measured under. A run taking the default would have been unqualified and its numbers would not have belonged to this profile.

**Host occupancy, because the metric is wall clock.** The coordinator drained every other worker lane for this run and dispatched nothing during it. Load averages recorded by the harness itself: `2.93 3.09 4.46` before, `2.32 2.92 4.33` after — this machine's idle desktop session with no build running, confirmed by inspecting the running processes.

### The measurable domain, re-derived before measuring

The compile-only sweep was rerun unchanged and **agreed exactly with the 2026-08-05 read**: of its 36 shapes, **24 retain all three strategies and none is refused on the grid axis**. The twelve that retain one are the contributor counts admitting no balanced exact partition. So the domain is wide and the predeclared timing matrix could be chosen freely inside it.

The timing matrix is **92 cells and 276 dispatched alternatives**: contributor counts `{4, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384}` against row counts `{1, 4, 16, 64, 256, 1024, 4096, 16384, 65536, 262144}`, keeping every pair of at most `2^24` elements. The element cap is a 64 MiB allocation budget, not a capability edge. **The contributor counts are deliberately not all perfect squares**: on a square count the partition count and contributors-per-partition are equal, so a model fitted only to squares cannot be distinguished from one that memorized the square root. The four non-square counts split into `(8, 4)`, `(16, 8)`, `(64, 32)`, `(128, 64)` and are the held-out set.

### The measurement

[The retained dispatch sweep](../spikes/program-planning/reduction-dispatch-crossover/README.md), raw rows at `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/sweep.tsv`.

**The metric is wall clock across `commit()` and `wait_until_completed()`, and it is not GPU-busy time.** `metal` 0.33.0 exposes no accessor for `MTLCommandBuffer`'s `GPUStartTime`/`GPUEndTime`; reading them needs an `unsafe` `msg_send!`, which is an ADR 0079 decision rather than a spike's convenience. **The submission round trip costs about 200 microseconds on this host before any kernel runs**, more than most cells' arithmetic put together, so each alternative is measured at two encode counts — the plan once, and the plan sixteen times in one command buffer — and the per-plan cost is `(batched - single) / 15`. The two differ by exactly fifteen extra encodes and nothing else, so the fixed cost divides out. Both the raw and amortized columns are retained; the analysis uses the amortized ones because the round trip is identical for all three strategies and could only bury a crossover, never create one.

Noise controls: every alternative fully emitted, linked, pipelined and allocated before timing; the command queue built once for the sweep rather than per submission; eight untimed warm-ups per alternative at each encode count; thirty **interleaved** rounds with a rotating start so drift lands on all three strategies alike; minimum, median, p90 and standard deviation reported at both encode counts.

Correctness precedes timing at every cell. Operands are all `1.0`, so a row's declared sum is exactly its contributor count, exact in `f32` under **every** grouping — which is what makes one expected value valid for three strategies under a regrouping-permitting contract, and what catches a dropped, double-counted, or unsynchronized contributor. That closed form is checked against `tiler-reference`'s independent evaluation on every cell of at most 4,096 elements. **Regrouped rounding is not observed and is not claimed.**

#### The crossover

Per-plan microseconds, and the ratio of the serial fold to the best parallel plan. Above one, parallelizing pays. Every row below is separated from the noise.

| shape | serial fold | best parallel | ratio |
| --- | --- | --- | --- |
| 4 x 8,192 | 575.3 | 11.3 | **50.7x** |
| 16 x 8,192 | 592.8 | 17.2 | **34.5x** |
| 64 x 8,192 | 660.3 | 35.5 | **18.6x** |
| 16 x 16,384 | 515.0 | 34.4 | **15.0x** |
| 256 x 8,192 | 768.4 | 103.1 | **7.45x** |
| 256 x 16,384 | 850.6 | 213.5 | **3.98x** |
| 1,024 x 4,096 | 250.3 | 203.2 | 1.23x |
| 4,096 x 2,048 | 421.9 | 441.1 | 0.96x |
| 16,384 x 32 | 27.6 | 31.9 | 0.86x |
| 65,536 x 16 | 52.5 | 66.6 | 0.79x |
| 16,384 x 4 | 4.7 | 8.3 | **0.56x** |

The contour runs diagonally. At one row the serial fold's reduction stage launches a **single invocation** and the machine idles; at 262,144 rows it launches more than the device holds and a parallel plan's extra launches and staged partials are pure overhead.

#### Two of the three strategies are a near-tie, and that reshapes the decision

Across all 92 cells the single-workgroup tree and the multi-pass split are separated on a handful. Where they do separate it is at large row counts and the split wins — at 262,144 rows of 4 contributors the split costs 81.7 microseconds against the tree's 458.7. **So the consequential decision on this program family is binary: parallelize or not.** Picking the wrong parallel strategy costs percent; parallelizing on the wrong side of the contour costs a factor. Both accuracies are reported below for that reason.

### The calibration, and its held-out bound

Three parameters, each a quantity of the machine and none of a strategy, under the classical work-span cost

```text
cost = sum over stages of ( encoder_seconds + max(work / parallel_threads, depth) * step_seconds )
```

with `work` the stage's fold steps summed over every invocation and `depth` its longest sequential path. **The `max` produces the crossover instead of asserting it**: when the row count saturates the device, `work / parallel_threads` dominates and the cheapest plan is whichever does least total work — the serial fold, which stages nothing; when it does not, `depth` dominates and the fold's path is the whole contributor run against a tree's roughly square root of it.

| parameter | fitted |
| --- | --- |
| `encoder_seconds` | 1.166e-6 (1.17 us per dispatch) |
| `parallel_threads` | 1.056e3 fold steps retired at once |
| `step_seconds` | 2.909e-8 (29.1 ns per critical-path step) |

Fitted on the perfect-square contributor counts only, minimizing mean squared log **decision regret** — the measured time of the strategy the model names, over the measured time of the fastest — with magnitude error as a tie break that can never outvote a decision. A cell is **separated** when the gap exceeds two combined standard errors of the two medians; the fit is taken over the cells whose serial-or-parallel verdict is separated, because fitting to a cell whose measured ordering is noise means fitting to noise. Every cell is still scored.

| question | set | cells | agreed | worst measured penalty |
| --- | --- | --- | --- | --- |
| three-way winner | fit | 60 | 45 | 4.27x |
| three-way winner | held-out | 32 | 29 | 1.81x |
| three-way winner, separated only | fit | 16 | 15 | 1.04x |
| three-way winner, separated only | held-out | 9 | **9** | **1.00x** |
| serial or parallel | fit | 60 | 46 | 4.27x |
| serial or parallel | held-out | 32 | 30 | 1.81x |
| serial or parallel, separated only | fit | 34 | 32 | 1.04x |
| serial or parallel, separated only | held-out | 26 | **24** | **1.81x** |

**The stated error bound is the last row: on the 26 held-out cells whose verdict is resolvable the calibration agrees on 24, and following it costs at most 1.81x** — at 1,024 rows of 128 contributors, where it says serial and the tree is 81% faster. Median regret is 1.0000 on both sets. Magnitude accuracy is much weaker — median relative error 0.17 fit and 0.16 held out, p90 near 0.76 — so **this is a selector, not a latency estimate**, and it must not be quoted as one.

### The mutation proof, and the negative it produced

[`perturbations.txt`](../spikes/program-planning/reduction-dispatch-crossover/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/perturbations.txt), reproducible with `--perturb <encoder|parallel|step> <factor>`.

| perturbation | held-out separated agreement | held-out worst penalty | magnitude median |
| --- | --- | --- | --- |
| none | 24 / 26 | 1.81x | 0.16 |
| `parallel` x 0.25 | **20 / 26** | **3.04x** | 1.27 |
| `parallel` x 4 | 24 / 26 | **1.20x** | 0.52 |
| `encoder` x 20 | 24 / 26 | 1.81x | 3.23 |
| `step` x 0.1 | 24 / 26 | 1.81x | 0.74 |

**Only `parallel_threads` carries selection evidence.** Multiplying the per-encoder cost by twenty and dividing the per-step cost by ten leave *every* predicted winner unchanged while wrecking the magnitude fit — the first because the split's extra dispatch never decides a separated cell, the second because scaling every stage by one factor cannot reorder anything. The "smallest analytical calibration" the Implementation keys ask for is therefore really **one** number, and that is what the activation surface has to declare.

**And the fitted value is on the low side of what held-out data prefers, which is stated rather than smoothed.** Quadrupling it leaves fit-set agreement where it was and improves the held-out worst penalty from 1.81x to 1.20x: the fit set's regret is flat in that parameter over a wide band and the magnitude tie break chose within it, so the contour's position is determined to roughly a factor of four. That is a limit of this measurement and it is part of why the calibration is retained as evidence rather than pushed into selection unreviewed.

### Why nothing was activated, stated as a boundary rather than as a gap

**The measurement supports activation. The repository's own rules put the surface it needs outside this branch.**

- The term has to be **declared by a target profile** to be consulted at all — declaring it inside `tiler-compiler` for the target-neutral baseline would be the unsourced number the authority ledger exists to refuse, since a macOS Apple9 device measurement is evidence about one target.
- Declaring it needs a `pub` `TargetProfileBuilder::declare_*` / `declare_measured_*` pair and a new `TargetProfileBuildError` variant. A consequential public boundary is Tom's, and he is offline.
- Moving the row moves the canonical descriptor, and therefore every pinned artifact identity and cache subject derived from it. That is an identity-domain step, also Tom's.
- The declaration site is `crates/tiler-build`, whose scope (`implementation/build`) this ticket does not hold.
- And there is a **real design problem to settle before the row exists**, already recorded elsewhere in the repository and not invented here: [`docs/research/program-planning/flash-class-capability-set.md`](../docs/research/program-planning/flash-class-capability-set.md) eliminated putting a cost number on a target profile because every `CapabilityAxis` variant is a hard bound and silence about one is `Unknown`, so a cost row declared the same way would render a profile **unexecutable for a quantity no feasibility predicate reads**. Beside it, `crates/tiler-compiler/src/component_cost.rs` records that a second cost-model key cannot simply join the first, because `dominates` returns `false` across keys and Pareto pruning would silently go dark.

Encoding the preference some other way was considered and refused on the ticket's own terms: widening `PlanStructuralCost` with a fitted dimension would mix measured and counted quantities in one Pareto relation, and biasing a structural constant until the parallel plan won is exactly *"altering a constant until the desired plan wins"*. **A measured no-activation is the correct close here.** The complete design, including all three problems above, is [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md).

### Required evidence, clause by clause

| Clause | Status |
| --- | --- |
| Retained raw measurements identify stable crossover regions or explicitly report that none was established | **Satisfied on the first branch.** 276 measured alternatives retained; the crossover is a contour with both sides separated from the noise by up to 50.7x and 1.78x. |
| Calibration predicts held-out rows within a stated error bound | **Satisfied.** Fitted on perfect-square contributor counts, scored on the rest: 24 of 26 separated held-out cells, worst measured penalty **1.81x**, median regret 1.0000. The bound is stated with the population it is over. |
| Serial remains selected below its measured crossover | **True, and now for a weaker reason than it sounds.** Selection is unchanged and still takes the serial fold everywhere, so it is trivially selected below the crossover — and also above it, which is the finding this ticket hands to its successor. |
| An unavailable environment makes no performance claim | **Vacuous, and checked rather than waived.** The environment was available and matched the ledger in every field, so the unavailable path was never exercised. Every claim here is scoped to that exact row. |
| Perturbing the calibrated term or environment identity changes or refuses the selection evidence | **Satisfied, with a negative that is part of the result.** Perturbing `parallel_threads` changes the held-out verdict and the worst penalty in both directions. Perturbing `encoder` or `step` changes **no** predicted winner, which is evidence that those two are inert in the decision rather than evidence of a check that did not run — the magnitude fit moves by an order of magnitude under the same perturbations, so the checks demonstrably ran. |
| No infeasible plan is represented as expensive | **Satisfied, unchanged.** Nothing in this branch touches feasibility. The compile-only rerun observed no grid-axis refusal at any shape and every refusal that does occur is a typed predicate. |
| Explain output names why the winning strategy won | **Not satisfied, and not claimed.** Explain still reports `event=selection:tiler.selection.structural-pareto.v1:selected` with structural cost terms. Naming a measured term requires the term to exist, which is the parked surface. |

### What landed

- [`spikes/program-planning/reduction-dispatch-crossover`](../spikes/program-planning/reduction-dispatch-crossover/README.md) — the dispatching sweep, the device-free fit binary sharing one stage model with it, and the retained 2026-08-07 result directory (`sweep.tsv`, `environment.tsv`, `calibration.txt`, `perturbations.txt`). Catalogued in `spikes/README.md` and on the ledger row it supports in `docs/research/README.md`.
- The sibling compile-only spike's README records the rerun agreeing and points at the timing result it authorized. Its 2026-08-02 inference that `1x4` had no discriminating power is now confirmed by measurement rather than left as reasoning about magnitudes.
- Four sites in `crates/tiler-compiler` that said the measurement was obtainable and not taken now say what it found and why selection is unchanged anyway; two more that named this ticket as the owner of an unassigned preference now name its successor. `docs/compiler/fusion-and-scheduling.md`, `docs/open-questions.md`, and `docs/research/program-planning/flash-class-capability-set.md` carry the same correction.
- Two successor tickets: the activation surface above, and [`calibrate-the-reduction-partition-against-measured-alternatives`](calibrate-the-reduction-partition-against-measured-alternatives.md) for the choice this sweep held constant.

**`governed_partition` is unchanged and uncalibrated, and this run is not evidence about it.** Every cell used whatever partition it returned, so the sweep varied the shape and never the split. The four doc sites that named this ticket as the owner of "replacing it with measured evidence" now name that second ticket, because that is the experiment which would.

### Measurement boundary

One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract (`FLUSH_AND_REASSOCIATE_F32`), one program family (multiply-add prologue into a trailing-axis sum), `f32` only, one host row. **Wall clock end to end, never GPU-busy time.** The batched encode count amortizes the submission round trip and also leaves the input hotter in cache than a cold first call, so these are not first-call latencies. `parallel_threads` is determined to about a factor of four. No numerical claim: the oracle is exact by construction and cannot observe regrouped rounding. The matrix stops at `2^24` elements for allocation budget, so nothing here is evidence about the profile's widest admissible launch.

### Commands

```sh
# the compile-phase domain, rerun and unchanged
cd spikes/program-planning/reduction-crossover && cargo run --release

# the dispatch sweep, on the qualified environment
cd spikes/program-planning/reduction-dispatch-crossover
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-dispatch-sweep > results/<date>-<host>/sweep.tsv

# the calibration and its held-out score, no device needed
cargo run --release --bin reduction-cost-fit -- results/<date>-<host>/sweep.tsv
cargo run --release --bin reduction-cost-fit -- results/<date>-<host>/sweep.tsv --perturb parallel 0.25
```
