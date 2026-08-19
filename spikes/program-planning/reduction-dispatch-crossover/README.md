---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.reduction-dispatch-crossover"
kind: "experiment"
title: "Where the parallel-reduction crossover is, measured on the device"
topics: ["program-planning", "scheduling", "reductions", "cost-model", "metal"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger"]
entrypoints: ["spikes/program-planning/reduction-dispatch-crossover/src/main.rs", "spikes/program-planning/reduction-dispatch-crossover/src/fit.rs", "spikes/program-planning/reduction-dispatch-crossover/activated_selector_check.py"]
last_verified: "2026-08-07"
ticket: "calibrate-and-activate-parallel-reduction-selection"
---

# Where the parallel-reduction crossover is, measured on the device

[`reduction-crossover`](../reduction-crossover/README.md) beside this spike answers *where a crossover could be measured* — over which shapes the authoritative Apple profile retains all three reduction alternatives at once. Against the measured grid-axis row that domain is wide, so this spike answers the question `calibrate-and-activate-parallel-reduction-selection` actually owns: **at which shapes does the fastest strategy change, and can one analytical model predict where?**

It dispatches. For every cell of a predeclared shape matrix it compiles the same program family, emits MSL for each retained alternative, links it with `xcrun`, builds the pipelines, allocates the program's own buffers, and submits the whole plan repeatedly on the qualified Metal host.

## The result in one paragraph

**Measurement, 2026-08-07 — a crossover exists, it is large, and it is a contour rather than a point.** Over a 92-cell matrix the serial fold is up to **50.7x slower** than the best parallel strategy (4 rows of 8,192 contributors) and up to **1.78x faster** than it (16,384 rows of 4). The transition runs diagonally across the matrix: parallel strategies win wherever the row count alone cannot saturate the device, the serial fold wins wherever it can. **The two parallel strategies are inside each other's noise almost everywhere**, so the decision selection has to make on this program family is *whether to parallelize*, not *which parallel plan to use*. A three-parameter analytical model fitted to the fit set reproduces that verdict on **24 of the 26 held-out cells whose verdict is separated from the noise**, and the worst measured penalty for following it on a held-out cell is **1.81x**.

## What it drives

For each `(rows, contributors)` cell the sweep compiles the reduction program family — an elementwise multiply-add prologue feeding a sum over the trailing axis — under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` against `BoundMetalCompileDeclaration::first_macos_apple9`, and requires all three alternatives. Every dispatch parameter is read from the compiler's own record: the argument-table index of each buffer from the emitter's binding table, the byte window from the program's own view, both launch extents from the ABI arena. Nothing about the topology is assumed, which is why one code path prepares the fold, the split, and the tree unchanged.

The three strategies are told apart by structure rather than by name — the split is the alternative with three stages, the tree is the one declaring an entry wider than one thread per workgroup, the fold declares neither. That is deliberately the same rule the compile-only sweep, `tiler_build::metal_plan`'s parallel-portfolio fixture, and `prototypes/serial-sum-run` all use, so a divergence in what "the tree" means cannot make two of those claims about different things.

### The predeclared matrix

Contributor counts `{4, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384}` against row counts `{1, 4, 16, 64, 256, 1024, 4096, 16384, 65536, 262144}`, keeping every pair whose element count is at most `2^24`. That is 92 cells and 276 dispatched alternatives. The element cap is a 64 MiB allocation budget, not a capability edge: the profile's grid-axis row admits 268,435,456, sixteen times more.

Every contributor count admits `governed_partition`'s balanced exact split, because a count that does not retains only the serial fold and contributes no comparison. **They are deliberately not all perfect squares.** On a square count the partition count and the contributors-per-partition are equal, so a model fitted only to squares cannot be distinguished from one that memorized the square root; the four non-square counts split into `(8, 4)`, `(16, 8)`, `(64, 32)` and `(128, 64)` and are held out of the fit.

## What is measured, and what that number is not

One sample is the wall clock across `commit()` and `wait_until_completed()` for one submission. **`metal` 0.33.0 exposes no accessor for `MTLCommandBuffer`'s `GPUStartTime` or `GPUEndTime`**; reading those would need an `unsafe` `msg_send!`, and a new unsafe site is a decision under [ADR 0079](../../../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) rather than a convenience a spike may take. So the recorded time is end to end and **this is not a GPU-busy measurement**. Nothing here should be quoted as one.

**That round trip costs about 200 microseconds on this host before any kernel runs**, which is more than most cells' arithmetic put together, so a difference between strategies would have been invisible under it. The sweep therefore measures each alternative at two encode counts — the plan once, and the plan sixteen times in one command buffer — and reports the per-plan cost as `(batched - single) / 15`. The two submissions differ by exactly fifteen extra encodes of the same plan and by nothing else, so the fixed cost divides out. Metal orders compute encoders within a command buffer unconditionally, so the repeats run one after another rather than overlapping, and each repeat is idempotent for every strategy here.

Both the raw submission columns and the amortized ones are retained. The amortized ones are what the analysis uses, and the reason is not only resolution: the round trip is *identical* for all three strategies, so including it cannot produce a crossover and can only bury one.

## Noise controls

- Every alternative is fully prepared — emitted, linked, pipelined, allocated, input written — before any timing starts. No compilation, allocation, or host copy happens inside a timed region, and the command queue is built once for the whole sweep rather than per submission.
- Eight untimed submissions per alternative at each encode count precede the timed ones.
- The timed submissions are **interleaved**: each of thirty rounds submits every alternative once, and the round's starting alternative rotates, so a thermal or scheduling drift lands on all three strategies alike instead of on whichever ran last.
- Minimum, median, p90 and sample standard deviation are reported at both encode counts rather than a single number.
- The host's load averages are recorded before and after the sweep. This run had the machine to itself: every other agent lane was drained for it and nothing was dispatched during it. Recorded load was `2.93 3.09 4.46` before and `2.32 2.92 4.33` after, which is this machine's idle desktop session and no build.

## The oracle

Every operand is `1.0`, so the declared sum of a row is exactly the contributor count, representable in `f32` for every count this matrix reaches. **Every grouping of that row therefore produces the same bits**, which is what makes one expected value valid for three strategies under a contract that *permits* regrouping, and a dropped, double-counted, or unsynchronized contributor changes the sum and is caught. Every output element of every alternative is checked before that alternative is timed.

That closed form is checked against `tiler-reference`'s independent evaluation of the same semantic program on every cell of at most 4,096 elements, so the constant is tied to the oracle rather than asserted beside it. **Regrouped rounding is not observed and is not claimed**: unit operands cannot expose it, and `drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies` owns that evidence.

## Running it

```sh
cd spikes/program-planning/reduction-dispatch-crossover
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-dispatch-sweep > results/<date>-<host>/sweep.tsv
cargo run --release --bin reduction-cost-fit -- results/<date>-<host>/sweep.tsv
```

`DEVELOPER_DIR` selects the offline toolchain the [authority ledger](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s compilation-environment row names. Without it a host whose default selection is a newer Xcode links through a compiler the profile was not measured under, which is a different environment and makes the run unqualified. On the host that produced the retained result the default selection *is* a newer Xcode, so the variable is load-bearing rather than defensive.

The fit binary takes no device and can be rerun, audited, and perturbed anywhere. No `make` target reaches here, per [`spikes/README.md`](../../README.md).

**The timed sweep is load-sensitive and runs only on an otherwise idle host** — every other agent lane drained and nothing dispatched during it, per the repository performance protocol; the retained run's `environment.tsv` records exactly that occupancy. `--smoke` runs the load-insensitive half alone: it compiles every cell of the predeclared matrix through the production `CompileRequest`, prepares and oracle-verifies all three alternatives in one untimed submission each, checks the stage model against every published launch, and times nothing, so the whole pipeline can be proven working without waiting for the quiet window the sweep needs.

## The 2026-08-18 session state — pipeline reworked and proven, timed sweep pending its quiet window

Under the accepted (R, R) disposition of `resolve-the-retained-metal-profile-measurement-invocation-authority`, the harness was reworked on repository base `ea5c615d` so its records carry request-derived selection identity: the sweep header now prints `selection.compile_flags` and `selection.link_flag_count` from the same request constructor every cell compiles through, plus the resolved offline compiler, linker, and SDK identities from the driver that runs them. The shared stage model now models the single-workgroup tree at the compiler's capped participant width (`capped_tree_partition`, nearest-admissible-to-256) rather than the balanced split the 2026-08-07 sweep dispatched — the boundary that record already named — and the launch-geometry check refuses any cell where that copy and the published extents disagree.

**Measurement, 2026-08-18** — a full-matrix `--smoke` validation on macOS build `26A5406e` passed: all 92 cells compiled through the production `CompileRequest`, retained all three alternatives, matched the capped-tree stage model's launch extents, and verified against the oracle, untimed. Retained at [`results/2026-08-18-apple-m4-max-macos27.0-26A5406e/smoke.txt`](results/2026-08-18-apple-m4-max-macos27.0-26A5406e/smoke.txt); the timed sweep's exact command, preconditions, and custody steps are in the same directory's [`RUN.md`](results/2026-08-18-apple-m4-max-macos27.0-26A5406e/RUN.md). Until that sweep runs, this spike's only timed evidence remains the 2026-08-07 record below, whose execution build no longer exists on any qualified host.

## The retained result

**Measurement, 2026-08-07**, at [`results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/): `sweep.tsv` (276 measured alternatives), `environment.tsv`, `calibration.txt`, `perturbations.txt`. The environment matches the authority ledger's rows in every field — offline `Apple metal version 32023.883`, `AIR-LLD 32023.883`, Xcode 26.6 (17F113), SDK macosx 26.5 (25F70); execution macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max — under toolchain `nightly-2026-07-19`.

### The crossover

Per-plan cost in microseconds, and the ratio of the serial fold to the best parallel strategy. A ratio above one means parallelizing pays.

| shape | serial fold | best parallel | ratio | separated |
| --- | --- | --- | --- | --- |
| 4 x 8,192 | 575.3 | 11.3 | **50.7x** | yes |
| 16 x 8,192 | 592.7 | 17.2 | **34.5x** | yes |
| 16 x 16,384 | 514.9 | 34.4 | **15.0x** | yes |
| 64 x 8,192 | 660.2 | 35.5 | **18.6x** | yes |
| 256 x 8,192 | 768.4 | 103.1 | **7.45x** | yes |
| 256 x 16,384 | 850.6 | 213.5 | **3.98x** | yes |
| 1,024 x 4,096 | 250.4 | 203.2 | 1.23x | yes |
| 4,096 x 2,048 | 421.9 | 441.0 | 0.96x | yes |
| 16,384 x 32 | 27.6 | 31.9 | 0.86x | yes |
| 65,536 x 16 | 52.5 | 66.6 | 0.79x | yes |
| 16,384 x 4 | 4.7 | 8.3 | **0.56x** | yes |

The contour runs diagonally. At one row the serial fold's reduction stage launches a **single invocation** and the machine sits idle; at 262,144 rows it launches more invocations than the device holds and the parallel strategies' extra launches and staged partials are pure overhead. Both extremes are two orders of magnitude apart in the ratio, which is why the crossover is not a marginal effect that better measurement could erase.

### The two parallel strategies are a near-tie, and that is a finding

Across all 92 cells, the single-workgroup tree and the multi-pass split differ by more than the noise on a handful. Where they do separate it is at large row counts, where the split wins — at 262,144 rows of 4 contributors the split costs 81.7 microseconds against the tree's 458.7, because the tree launches `partitions` invocations per output position where the split's second pass launches one. The tree's advantage at the other end is not over the split but over the fold.

**So the consequential decision on this program family is binary.** A model that picks the wrong parallel strategy costs a few percent; one that parallelizes on the wrong side of the contour costs a factor. The fit reports both accuracies for that reason.

### The calibration

Three parameters, each a quantity of the machine and none of a strategy, under the work-span cost

```text
cost = sum over stages of ( encoder_seconds + max(work / parallel_threads, depth) * step_seconds )
```

where `work` is the stage's fold steps summed over every invocation and `depth` is its longest sequential path. Fitted values:

| parameter | fitted | reading |
| --- | --- | --- |
| `encoder_seconds` | 1.166e-6 | 1.17 microseconds per dispatch |
| `parallel_threads` | 1.056e3 | fold steps retired at once when saturated |
| `step_seconds` | 2.909e-8 | 29.1 nanoseconds per critical-path step |

**The `max` is the whole of the model's physics and it produces the crossover instead of asserting it.** When the row count already saturates the device, `work / parallel_threads` dominates every strategy and the cheapest is whichever does least total work — the serial fold, which stages nothing. When it does not, `depth` dominates, and the fold's path is the whole contributor run against a tree's roughly square root of it. No parameter names a strategy and no strategy has a term of its own.

### Accuracy, on the fit set and the held-out set

| question | set | cells | agreed | worst measured penalty |
| --- | --- | --- | --- | --- |
| three-way winner | fit | 60 | 45 | 4.27x |
| three-way winner | held-out | 32 | 29 | 1.81x |
| three-way winner, separated cells only | fit | 16 | 15 | 1.04x |
| three-way winner, separated cells only | held-out | 9 | **9** | **1.00x** |
| serial or parallel | fit | 60 | 46 | 4.27x |
| serial or parallel | held-out | 32 | 30 | 1.81x |
| serial or parallel, separated cells only | fit | 34 | 32 | 1.04x |
| serial or parallel, separated cells only | held-out | 26 | **24** | **1.81x** |

**The stated error bound is the last row: on the 26 held-out cells whose serial-or-parallel verdict is resolvable, the calibration agrees on 24, and following it costs at most 1.81x** — at 1,024 rows of 128 contributors, where it says serial and the tree is 81% faster. The other held-out miss costs 1.20x. Median regret is 1.0000 on both sets.

A cell is **separated** when the gap exceeds two combined standard errors of the two medians. The unseparated cells are not excluded from the tables above, only reported apart: at a few hundred elements the whole plan costs less than one dispatch and the recorded order of three indistinguishable strategies is a coin toss. The fit set's 4.27x worst case is exactly such a cell — four rows of 64 contributors, whose fastest alternative is recorded at 0.37 microseconds, below the fitted cost of the two dispatches it performs.

Magnitude accuracy is much weaker than decision accuracy: median relative error 0.17 on the fit set and 0.16 held out, with a p90 near 0.76 and a worst case in the noise cells. **This model is a selector, not a latency estimate**, and it should not be quoted as one.

### The mutation proof, and what it refutes

[`perturbations.txt`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/perturbations.txt) rescales one fitted parameter at a time and rescores. The result is sharper than "the checks are able to fail":

| perturbation | held-out separated agreement | held-out worst penalty | magnitude median |
| --- | --- | --- | --- |
| none | 24 / 26 | 1.81x | 0.16 |
| `parallel` x 0.25 | **20 / 26** | **3.04x** | 1.27 |
| `parallel` x 4 | 24 / 26 | **1.20x** | 0.52 |
| `encoder` x 20 | 24 / 26 | 1.81x | 3.23 |
| `step` x 0.1 | 24 / 26 | 1.81x | 0.74 |

**Only `parallel_threads` carries selection evidence on this matrix.** Multiplying the per-encoder cost by twenty and dividing the per-step cost by ten leave *every* predicted winner unchanged while wrecking the magnitude fit — the first because the split's extra dispatch never decides a separated cell, the second because scaling every stage by one factor cannot reorder anything. Those two parameters are pinned by magnitude alone and are inert in the decision.

**And the fitted `parallel_threads` is on the low side of what held-out data prefers.** Quadrupling it leaves fit-set agreement where it was and improves the held-out worst penalty from 1.81x to 1.20x. The fit set's regret is flat in that parameter over a wide band and the magnitude tie break is what chose within it, so the contour's position is determined to roughly a factor of four rather than tightly. That is a limit of this measurement and it is the reason the calibration is retained as evidence rather than activated.

## What the compiler activated, and the one thing that had to be measured for it

**Measurement, 2026-08-07 — the activated selector drops two of the three fitted parameters and reproduces this sweep's penalties exactly.** `activate-measured-reduction-selection-from-a-target-cost-row` declares `parallel_threads` on the qualified target profile as a measured cost row and activates

```text
fold_steps = sum over stages of max( work, depth * P )
```

which is `P` times the model above at `encoder = 0, step = 1`. Dropping `step` is *provably* order-preserving: it is one positive factor over the whole sum. Dropping `encoder` is not provable that way — it is a per-stage constant, so removing it can in principle reorder — and the argument for it is that `encoder` prices *dispatch count*, which the compiler's structural cost model already carries as one of its four exact dimensions and prunes on, so pricing it here would put one quantity under two authorities.

**Provably order-preserving and measured-inert are different claims, and the second needed measuring.** [`activated_selector_check.py`](activated_selector_check.py) scores the reduced selector on this directory's recorded `threads:work:depth` triples, and its held-out worst measured penalties are **1.81x fitted, 3.04x at a quarter, 1.20x at four times** — the same three numbers [`perturbations.txt`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/perturbations.txt) reports for the complete three-parameter model. The worst-cell coordinates agree too: 1,024 rows of 128 contributors at the fitted value, which is the cell this record already names.

The agreed/total counts differ from that file's and the difference is a separation rule rather than a disagreement: the script resolves a cell when the fold and the *best parallel* strategy are separated, which is the binary decision the compiler makes, while `fit.rs` additionally reports the three-way winner. It takes no device and can be rerun anywhere:

```sh
python3 spikes/program-planning/reduction-dispatch-crossover/activated_selector_check.py
```

**One boundary the consumer inherits.** This sweep dispatched the single-workgroup tree at `governed_partition`'s balanced split, because `MEASURED_TREE_PARTICIPANT_CAP` landed after it. The compiler now emits the capped width, so at some shapes the tree it dispatches is not the tree timed here. That moves *which parallel plan* the selector prefers and not *whether it parallelizes* — the distinction this record already found consequential.

## Boundary

- **One profile** (`tiler.metal.macos-apple9.msl4-0.f32.v1`), **one contract** (`FLUSH_AND_REASSOCIATE_F32`), **one program family** (multiply-add prologue into a trailing-axis sum), **`f32` only**, **one host row**. It does not generalize to another Apple family, OS row, dtype, or device, and the fitted parameters are quantities of *this* machine.
- **Wall clock end to end, never GPU-busy time.** The binding exposes no command-buffer timestamps and none were read.
- The batched encode count amortizes the submission round trip; it also leaves the input hotter in cache than a single cold run would, so a first-call latency is not what these numbers describe.
- **`parallel_threads` is determined to about a factor of four**, by the mutation table above.
- **No numerical claim.** The oracle is exact by construction and cannot observe regrouped rounding.
- The matrix stops at `2^24` elements for allocation budget, so nothing here is evidence about the profile's widest admissible launch.
