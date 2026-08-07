---
id: calibrate-the-reduction-partition-against-measured-alternatives
title: Calibrate the reduction partition against measured alternatives
status: review
priority: p3
dependencies: [calibrate-and-activate-parallel-reduction-selection]
related: [activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, measurement, scheduling, reductions]
claimed_from: todo
assignee: agent-partition-cal
lease_expires_at: 1786086448
---
## User-visible outcome

The contributors-per-partition both parallel reduction strategies use is either confirmed as the best available choice on the qualified profile, or replaced by one that measurement supports.

## Why this exists, and what it is not

**Fact — `governed_partition` is a choice nothing has measured.** `crates/tiler-compiler/src/physical.rs` returns the divisor of the contributor count nearest its integer square root from below, and its own doc says so: "deliberately *a* choice and not a calibrated one". Both parallel strategies read it — the multi-pass split's partition count and the single-workgroup tree's participant count are the same number — so it fixes the tree's workgroup width as well as the split's partial extent.

**Fact — the 2026-08-07 crossover sweep did not measure it.** [`spikes/program-planning/reduction-dispatch-crossover`](../spikes/program-planning/reduction-dispatch-crossover/README.md) varied the shape across 92 cells and timed three strategies at each, but every cell used whatever partition this function returned. It is evidence about *which strategy*, and the partition was a constant of the experiment rather than a variable in it. No value of this function is confirmed or refuted by it.

**This is p3 and separated on purpose.** The measured strategy contour spans two orders of magnitude; a partition choice plausibly moves a plan by a much smaller factor, and choosing the wrong strategy is what the priority belongs to. Splitting it also keeps the activation ticket's surface to one term.

## Implementation keys

Sweep the partition at fixed shapes rather than the shape at a fixed partition — the inverse of the retained sweep, reusing its harness discipline. The partition is not freely settable through the public compiler entry point, so state how it is varied and keep that mechanism out of the shipped path.

Cover both strategies, because they consume the number differently: the split's partition count is a launch extent and its contributors-per-partition is a fold length, while the tree's partition count is *also* its workgroup width and its threadgroup reservation. A partition that is best for one need not be best for the other, and a single calibrated value would then be a compromise that should be named as one.

Pick shapes from the retained sweep's separated cells so the result composes with it rather than describing a different regime.

## Required evidence

Retained raw measurements on the qualified environment with its exact rows, or an explicit statement that the environment was unavailable. The verdict per strategy, with the noise band that makes it a verdict. If the balanced exact split is confirmed, say by how much it beats the alternatives and over which shapes; if it is not, name the shapes where it loses and by how much.

## Closes when

The record states, with retained measurements, whether the balanced exact split is the best available choice for each strategy on this profile, or which choice replaces it and where.

## Outcome — 2026-08-07: the balanced exact split is refuted for both strategies, and only the tree gets a replacement

**Read the shape of the close first.** Required evidence asked whether the balanced exact split is the best available choice for each strategy, and if not which choice replaces it and where. The answer is **not for either strategy**, and the two strategies part company on what follows: the tree gets a constant replacement that survives held-out scoring, and the split gets a refutation with **no constant to put in its place**, because its optimum moves with the row count in the same way the strategy contour does. Nothing is activated — the compiler still calls `governed_partition`, no compiler boundary was widened, and `crates/` was not touched.

### The environment matched the ledger in every field

Retained at [`environment.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-07-apple-m4-max-macos27.0-26A5388g/environment.tsv). Offline: `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`, Xcode 26.6 (17F113), SDK `macosx` 26.5 (25F70). Execution: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max. Toolchain `nightly-2026-07-19`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`.

`DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer` was load-bearing: this host's default selection is `/Applications/Xcode-beta.app/Contents/Developer`, which links through a compiler the profile was not measured under. **Host occupancy**, because the metric is wall clock: sole occupant, no other lane running and nothing dispatched. Harness-recorded load averages `2.88 3.13 3.40` before and `3.03 3.18 3.40` after.

### How the partition was varied, and why the rows are evidence

**`governed_partition` is `pub(crate)` and its result is a total function of the contributor count**, so no shape and no request reaches a second value through the public `compile` entry point. [`spikes/program-planning/reduction-partition-calibration/src/regions.rs`](../spikes/program-planning/reduction-partition-calibration/src/regions.rs) rebuilds the two reduction regions from `tiler-ir`'s published `ScheduledRegion` / `ContributorPartition` / `lower_scheduled_region` vocabulary with the partition as a parameter — transcriptions of `physical.rs`'s `partial_reduction_region`, `final_reduction_region`, and `single_workgroup_tree_region`. The numerical realization is read off the compiler's own reduction kernel and the elementwise prologue kernel is taken from the compiler's plan unmodified. **The mechanism is spike-local and nothing shipped changed.**

**The transcription is checked, not trusted.** At the governed partition of every shape the sweep requires the rebuilt plan to emit the **byte-identical translation unit** the compiler emits for the same alternative *and* to declare the launch extents the compiler's ABI publishes, refusing the shape otherwise. Both held on all seven shapes for both strategies. That anchor is the whole licence for reading off-governed rows as statements about the compiler's plans.

### Population, predeclared before measuring

**Seven shapes x two strategies x every admissible partition = 130 variants; 122 measured, 8 declined.** Shapes: `(4, 8192)`, `(64, 8192)`, `(256, 16384)`, `(1024, 4096)`, `(4096, 2048)`, `(16384, 32)`, `(65536, 16)` — every one a separated cell of the retained crossover sweep, drawn across its contour from where parallelizing pays 50x to where the serial fold wins. Admissible means an exact split into at least two parts of at least two, the same rule `governed_partition` searches within. Contributor count four is excluded because it admits exactly one split and contributes no comparison.

All 8 declines are tree-side and each is a real bound: 7 because the declared workgroup width exceeds the prepared entry's 1,024 — which *is* the profile's declared bound, since the Apple9 declaration fills that row with a prepared-entry query rather than a literal — and 1 because `tiler_ir::schedule::workgroup_tree_tile(8192)` has no representation. The split declined nothing at any partition.

### Verdicts, with the noise band that makes them verdicts

A gap counts only when it exceeds two combined standard errors of the two medians, the retained crossover sweep's own rule. Per-plan microseconds, submission round trip cancelled by encode-count differencing at 64 encodes.

| shape | strategy | governed | best | ratio | band | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 4 x 8,192 | split | 128 @ 16.24 | 256 @ 11.49 | **1.413x** | 0.97 | beaten |
| 4 x 8,192 | tree | 128 @ 11.60 | 256 @ 9.53 | **1.216x** | 0.72 | beaten |
| 64 x 8,192 | split | 128 @ 40.28 | 32 @ 34.01 | 1.184x | 1.18 | beaten |
| 64 x 8,192 | tree | 128 @ 34.68 | 256 @ 32.13 | 1.079x | 0.92 | beaten |
| 256 x 16,384 | split | 128 @ 217.24 | 64 @ 205.66 | 1.056x | 1.18 | beaten |
| 256 x 16,384 | tree | 128 @ 212.18 | 64 @ 202.96 | 1.045x | 1.20 | beaten |
| 1,024 x 4,096 | split | 64 @ 204.24 | 64 @ 204.24 | 1.000x | 9.49 | **best** |
| 1,024 x 4,096 | tree | 64 @ 202.39 | 64 @ 202.39 | 1.000x | 4.20 | **best** |
| 4,096 x 2,048 | split | 64 @ 447.29 | 2 @ 424.57 | 1.054x | 1.48 | beaten |
| 4,096 x 2,048 | tree | 64 @ 444.50 | 256 @ 427.86 | 1.039x | 1.21 | beaten |
| 16,384 x 32 | split | 8 @ 33.34 | 2 @ 28.82 | 1.157x | 1.33 | beaten |
| 16,384 x 32 | tree | 8 @ 51.26 | 16 @ 50.81 | 1.009x | 1.72 | within noise |
| 65,536 x 16 | split | 4 @ 64.00 | 2 @ 58.13 | 1.101x | 1.38 | beaten |
| 65,536 x 16 | tree | 4 @ 147.45 | 2 @ 147.09 | 1.002x | 1.34 | within noise |

**The split's governed partition is beaten on 6 of 7 shapes, the tree's on 4 of 7 — 10 of 14 cells.** An independent plateau rule (the set of partitions a cell cannot separate from its own best) puts the governed choice outside the plateau in exactly those ten and inside it in the other four; the scoring binary *asserts* that the two rules agree, so a disagreement would fail the analysis rather than be reported as one.

**The partition is worth up to 5.05x** between the best and worst admissible value of one shape (tree at 4 x 8,192: 9.53 at 256 participants against 48.15 at two), so it is not a negligible term — but the governed choice is a defensible middle and never pays more than 1.413x. It is refuted as *optimal*, not as *reasonable*.

### What replaces it

Candidate rule: a cap, selecting the largest admissible partition not exceeding it. Scored on the full population and then leave-one-out — chosen on six shapes, paid for on the seventh.

| strategy | rule | worst regret, full population | worst regret, leave-one-out |
| --- | --- | --- | --- |
| tree | governed | 1.216 | — |
| tree | **cap 256** | **1.008** | **1.008** |
| split | governed | 1.413 | — |
| split | cap 32 or cap 256 | 1.379 | 2.131 |

**Tree — replaced.** Cap the participant count at 256. Leave-one-out selects it on all seven folds, held-out worst regret **1.008** and median regret **1.000**, against the governed choice's 1.216. The repeat run puts the same pair at 1.012 and 1.211.

**Split — refuted, not replaced.** Two caps beat the governed choice on the full population (1.379 worst, 1.018 median against 1.413 and 1.184), but the fold holding out `(4, 8192)` pays **2.131x**, worse than the choice it would replace — and *which* cap that fold selects is not even stable, being 8 in the retained run and 16 in the repeat. The reason is structural: **the split's best partition moves from 256 at four rows to 2 at 4,096 and above**, because once the row count alone saturates the device the extra partitions add total work and stage more partials without buying parallelism. A rule that improves the split has to read the row count against a saturation threshold, which is exactly the target cost row [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md) owns. **The split's partition and the split's strategy selection turn on the same machine quantity; calibrating one without the other is not available.**

### The single value, named as the compromise the Implementation keys asked for

**It is 256.** Free for the tree at a held-out worst regret of 1.008; for the split an improvement on the governed choice (1.379 worst and 1.018 median against 1.413 and 1.184) but **1.31x to 1.38x off the split's own best at the two highest row counts**, where the split wants the minimum split.

`(4096, 2048)` is where the compromise is sharpest, because the two strategies' plateaus there are **disjoint**: the split's is `{2, 4}` and the tree's is `{256}` alone. Choosing 256 costs the split 1.023x; choosing 2 costs the tree 1.100x. Both 64-encode runs agree on both plateaus.

### Two things measured that are not explained

- **The curve is not monotone.** At `(256, 16384)` and `(1024, 4096)`, 128 partitions is a local maximum for *both* strategies, worse than both neighbours by 7.7 to 14.5 microseconds against bands of 1.18 to 6.32, in all three runs. A memory-stride explanation is available — the partial pass reads a contiguous run of `contributors_per_partition` floats — but this sweep varied the partition and not the access pattern, so no mechanism is claimed.
- **`(1024, 4096)` is noisier in the retained run than in the repeat**, with two single-submission outliers near a millisecond widening its band from 1.6 to 9.5 microseconds. The conservative spread absorbed it; its median is 204.24 against the repeat's 204.25 and its verdict is unchanged in all three runs.

### Repeatability

The matrix was run three times, twice at 64 encodes and once at 16. **The two 64-encode runs agree on all 14 verdicts**; the governed partition's median differs between them by at most 3.27%, median 0.36%. The 16-encode pilot agrees on 12 of 14, both differences being cells its coarser band could not resolve rather than numbers that moved. All three are retained.

The encode count is 64 rather than the crossover sweep's 16 deliberately: that sweep resolved differences between *strategies* spanning two orders of magnitude, this one resolves differences between *partitions of one strategy* at a few percent, and the amortized band is `(sigma_batched + sigma_single) / (BATCH - 1)` with a single-submission floor that does not shrink. Sixty-four moves the band from roughly four microseconds to roughly one. The per-plan quantity is the same difference quotient, so rows remain comparable.

### Commands

```sh
cd spikes/program-planning/reduction-partition-calibration
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-partition-sweep > results/<date>-<host>/sweep.tsv
cargo run --release --bin partition-regret -- results/<date>-<host>/sweep.tsv
```

The scoring half needs no device and recomputes every verdict, plateau, and regret figure above from the retained TSV.

### Boundary

One profile (`tiler.metal.macos-apple9.msl4-0.f32.v1`), one contract (`FLUSH_AND_REASSOCIATE_F32`), one program family, `f32` only, one host row, seven shapes, every contributor count a power of two. Wall clock end to end and never GPU-busy time — `metal` 0.33.0 exposes no command-buffer timestamps and reading them would be a new unsafe site under ADR 0079. Sixty-four back-to-back encodes hold the device busier than one cold submission, so these are steady-state per-plan costs and not first-call latencies. The cap-256 tree rule is held out over seven shapes, not over a shape distribution. No numerical claim: the oracle is exact by construction and cannot observe regrouped rounding.

### Owed rows, outside this ticket's scopes

Stated verbatim rather than written, because each belongs to a scope this ticket does not hold. None is required for this ticket's own evidence.

1. **`spikes/README.md`** (`contracts/navigation`) — a catalog row beside the crossover sweep's:
   `- [Whether the balanced exact split is the partition to use, measured on the device](program-planning/reduction-partition-calibration/README.md) — reproducible; bounded-measurement; supports: [Authority ledger for the first macOS Metal compile profile](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)`
2. **`docs/research/README.md`** (`contracts/navigation`) — append to the authority ledger row's `experiments:` list:
   `, [Whether the balanced exact split is the partition to use, measured on the device](../../spikes/program-planning/reduction-partition-calibration/README.md)`
3. **`crates/tiler-compiler/src/physical.rs`** (`implementation/compiler`) — `governed_partition`'s doc currently says the choice is uncalibrated and that this ticket owns replacing it with measured evidence, and `single_workgroup_tree_region`'s doc says the retained sweep "varied the shape, never the split of a given contributor run". Both are now superseded: the split is measured, the governed choice is beaten on 10 of 14 cells, the tree has a held-out replacement (cap participants at 256) and the split does not.
4. **`docs/compiler/fusion-and-scheduling.md`** (`contracts/optimizer`) — optional. It states the tree's and split's topology without making a calibration claim about the partition value, so nothing there is now false; a reader-facing pointer to this measurement would fit beside the 2026-08-07 crossover paragraph.
