---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.reduction-partition-calibration"
kind: "experiment"
title: "Whether the balanced exact split is the partition to use, measured on the device"
topics: ["program-planning", "scheduling", "reductions", "cost-model", "metal"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger"]
entrypoints: ["spikes/program-planning/reduction-partition-calibration/src/main.rs", "spikes/program-planning/reduction-partition-calibration/src/regret.rs", "spikes/program-planning/reduction-partition-calibration/src/excursion.rs"]
last_verified: "2026-08-11"
ticket: "calibrate-the-reduction-partition-against-measured-alternatives"
---

# Whether the balanced exact split is the partition to use, measured on the device

[`reduction-dispatch-crossover`](../reduction-dispatch-crossover/README.md) beside this spike answers *which strategy* to use: it varied the shape across 92 cells and timed three reduction strategies at each. Every one of those cells used whatever partition `crates/tiler-compiler/src/physical.rs`'s `governed_partition` returned — the divisor of the contributor count nearest its integer square root from below, which that function's own doc calls "deliberately *a* choice and not a calibrated one". The partition was a **constant of that experiment rather than a variable in it**, so no value of that function is confirmed or refuted by it.

This spike is the inverse. It holds the shape fixed and sweeps the partition, on shapes drawn from that sweep's separated cells so the two records compose, and it sweeps both parallel strategies separately because they consume the number differently.

## The result in one paragraph

**Measurement, 2026-08-07 — the balanced exact split is not the best available choice for either strategy, and only one of the two has a constant that replaces it.** Over 130 predeclared variants at seven shapes the governed partition is outside the statistically indistinguishable-from-best plateau in **10 of 14 shape-and-strategy cells**, costing up to **1.413x** for the multi-pass split and **1.216x** for the single-workgroup tree. For the **tree**, capping the participant count at **256** replaces it outright: chosen leave-one-out on six shapes and paid for on the seventh, its worst regret is **1.008** against the governed choice's 1.216. For the **split**, no constant survives that test — its best partition moves from 256 at four rows to 2 at 65,536 rows, so a constant chosen on six shapes costs **2.131x** on the seventh, worse than the governed choice it would replace. **The partition is therefore not separable from the same device-saturation quantity the strategy contour turns on**, and 256 named as one calibrated value for both strategies is a compromise that is free for the tree and up to 1.38x off the split's own best at high row counts.

## How the partition is varied, and what is not changed to vary it

`governed_partition` is `pub(crate)` and its result is a total function of the contributor count, so **no shape and no request reaches a second value through the public `compile` entry point.** Measuring alternatives needs plans the compiler would build if it chose differently.

[`src/regions.rs`](src/regions.rs) builds them. `tiler-ir` publishes `ScheduledRegion`, `ContributorPartition`, and `lower_scheduled_region`, so its three constructors are transcriptions of `physical.rs`'s `partial_reduction_region`, `final_reduction_region`, and `single_workgroup_tree_region` with the partition supplied as a parameter rather than chosen. Everything else is read from the compilation rather than restated: the numerical realization is taken off the compiler's *own* reduction kernel, and the elementwise prologue kernel is taken from the compiler's plan unmodified and re-emitted beside the rebuilt reduction stages.

**Nothing shipped changes.** The mechanism is a spike-local module that `crates/` cannot reach. The 2026-08-07 record was produced while both strategies called `governed_partition`. Production has since separated them: the split still calls `governed_partition`, while the tree calls `capped_tree_partition`. The current harness therefore derives the tree's production participant count from the compiler-published ABI instead of pretending the historical balanced choice is still current.

### The anchor, which is what makes the off-governed rows evidence

A transcription is a claim, so it is checked before any timing. At each strategy's current production partition — balanced for the split, compiler-published nearest-cap for the tree — the sweep requires two independent equalities and refuses the shape if either fails:

- the rebuilt plan must emit the **byte-identical translation unit** the compiler emits for the same alternative, which covers the kernel bodies, both fold bounds, the declared workgroup width, the workgroup staging declaration, and every numerical realization decision the emitter makes; and
- the launch extents the rebuilt regions declare must equal the ones the compiler's **ABI publishes**, which covers the dispatch the source cannot state.

Both held on all seven shapes for both strategies. Without them the sweep would be comparing the compiler's plan against a lookalike, and any measured difference could be the transcription rather than the partition.

## The predeclared matrix

**Seven shapes, both strategies, every admissible partition: 130 variants**, of which 122 were measured and 8 declined.

The shapes are all cells of the retained crossover sweep and all separated there, drawn across its measured contour: `(4, 8192)` and `(64, 8192)` sit deep on the side where parallelizing pays by more than an order of magnitude, `(256, 16384)` and `(1024, 4096)` where it still pays by a small factor, `(4096, 2048)` astride the contour, and `(16384, 32)` and `(65536, 16)` on the side where the serial fold wins. A partition effect appearing only where parallelism already dominates would be a different finding from one holding across the contour, and this matrix tells those apart.

A partition is admissible when it splits the contributor sequence exactly into at least two parts of at least two — the same rule `governed_partition` searches within, since an inexact split leaves a ragged final partition this profile does not lower and a partition holding one contributor folds nothing. **Contributor count four is deliberately absent** even though the retained sweep separates several of its cells: four admits exactly one exact split and so contributes no comparison.

### The declines, which are rows rather than omissions

All eight are tree-side, and each is a real bound rather than a harness fault:

| shape | participants | why |
| --- | --- | --- |
| 4 x 8,192 | 2,048; 4,096 | the prepared entry admits 1,024 threads per workgroup |
| 64 x 8,192 | 2,048; 4,096 | the prepared entry admits 1,024 threads per workgroup |
| 256 x 16,384 | 2,048; 4,096 | the prepared entry admits 1,024 threads per workgroup |
| 256 x 16,384 | 8,192 | `tiler_ir::schedule::workgroup_tree_tile` has no representation |
| 1,024 x 4,096 | 2,048 | the prepared entry admits 1,024 threads per workgroup |

The workgroup bound is not a constant this spike asserts: the authoritative Apple9 declaration fills its max-threads-per-workgroup row with a **prepared-entry query** rather than a literal, so the pipeline's own `maxTotalThreadsPerThreadgroup` *is* the profile's declared bound, and checking it here is checking the profile. The split has no declines at any partition, which is itself part of the finding: the split's partition count is only a launch extent, and the tree's is a workgroup width that runs into a hardware limit the split never touches.

## The predeclared tree-width excursion extension

[`measure-the-tree-width-excursion-past-the-cap`](../../../tickets/measure-the-tree-width-excursion-past-the-cap.md) asks the question the 2026-08-07 matrix cannot: what happens on a sparse non-power-of-two divisor lattice when the production rule selects a width above 256, and what happens at the first count where it instead stays at two while 521 is available? The extension uses the same region constructors, compiler source/ABI anchors, input, closed-form-plus-reference oracle, preparation, warm-up, interleaving, sample count, 64-encode difference quotient, and summary fields. It measures only the tree, because the split's width rule is not the question.

**The matrix was frozen before any timed submission:** the Cartesian product of rows `{4, 16,384}` and contributors `{514, 780, 1,042}`, six shapes and 52 tree variants.

| contributors | every admissible participant count | production | why this row exists |
| --- | --- | --- | --- |
| 514 | `{2, 257}` | 257 | The minimal excursion: the only width above the cap is one participant past it. |
| 780 | `{2, 3, 4, 5, 6, 10, 12, 13, 15, 20, 26, 30, 39, 52, 60, 65, 78, 130, 156, 195, 260, 390}` | 260 | A dense sub-cap lattice followed by a sparse upper lattice; the selected width is four past the cap and 390 tests the far side. |
| 1,042 | `{2, 521}` | 2 | The first contributor count at which production keeps two while declining another admissible width. |

The three 4-row cells are deep on the retained contour's side where parallelism pays, while the three 16,384-row cells are on the side where the serial fold wins. They are separated row-count values of the retained contour, not a claim that the crossover location transfers exactly to these new contributor counts. All 52 widths are at most 521, inside the live prepared entry's observed 1,024-thread capacity and the declared 32,768-byte local-memory row; the harness still asks the prepared entry and records any decline rather than inferring feasibility from either statement.

Before timing, `--verify-tree-width-excursion` ran the identical anchors and per-element oracle without warm-up or samples. It asserted the exact live device name and `supportsFamily(Apple9)` answer, and reported six shapes, 52 verified variants, and zero declines. Four independent subject perturbations were watched fail with assertions unchanged and restored: the tree partition failed the byte-identical source anchor; the rebuilt launch failed the ABI anchor; unit operands changed to two failed the oracle; and redirecting the result binding left a zero that failed the same oracle. Exact failure text is retained in the ticket Outcome.

### The excursion result

**Measurement, 2026-08-11**, retained at [`results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-excursion/`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-excursion/): primary `sweep.tsv`, same-matrix `repeat.tsv`, pinned `environment.tsv`, and device-free validation and scoring at `analysis.txt`. Both runs measured all 52 predeclared widths and declined zero. A gap counts only when it exceeds twice the two medians' combined standard errors, the same conservative rule as the retained calibration.

| shape | production | cost | best | cost | production / best | verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 4 × 514 | 257 | 9.8717 µs | 257 | 9.8717 µs | 1.000× | within noise of best; 2 and 257 share the plateau |
| 16,384 × 514 | 257 | 444.3644 µs | 257 | 444.3644 µs | 1.000× | best; 257 alone is in the plateau |
| 4 × 780 | 260 | 6.1243 µs | 39 | 3.3274 µs | **1.841×** | production beaten |
| 16,384 × 780 | 260 | 653.7328 µs | 39 | 631.3783 µs | **1.035×** | production beaten |
| 4 × 1,042 | 2 | 9.1574 µs | 521 | 7.1700 µs | **1.277×** | production beaten |
| 16,384 × 1,042 | 2 | 906.9511 µs | 521 | 896.1025 µs | **1.012×** | production beaten |

The predeclared boundary comparisons agree in the primary and repeat, **eight verdicts out of eight**. At 514, 2 versus 257 is inside noise at four rows and 257 is faster at 16,384. At 780, widening 195 → 260 is faster and widening 260 → 390 is sharply slower at both row counts: the dense lattice's optimum is below the cap, not at the nearest divisor. At 1,042, 521 beats production's two at both row counts. The repeat's same eight verdicts are identical. Across all 52 widths, the median primary-to-repeat relative `p50` difference is **0.24%** and the maximum is **12.71%**, the latter on the small-cost side where host round-trip noise is largest.

**This is not a measured excursion boundary and does not retune production.** It establishes that the cost is not flat at the sparse cutoff and that nearest-to-256 distance is not a sufficient width model on the dense lattice. Six cells on one host do not support a replacement selection rule; [`calibrate-a-shape-aware-tree-width-cost-row`](../../../tickets/calibrate-a-shape-aware-tree-width-cost-row.md) owns the wider held-out study. `capped_tree_partition` therefore stays unchanged.

The exact environment matches the authority ledger: macOS 27.0 build `26A5388g`, arm64, Apple M4 Max reporting Apple9, Xcode 26.6 build `17F113`, SDK 26.5 build `25F70`, offline Metal/AIR-LLD 32023.883, and nightly-2026-07-19. The coordinator reserved a quiet window and both timed submissions ran sequentially with no concurrent Cargo or full gate. Primary load was `2.97 4.33 4.66` before and `3.03 4.15 4.57` after; repeat was `2.95 4.12 4.56` before and `2.86 3.96 4.48` after.

**Custody distinguishes the timed executable from the checked-in replay source.** Both timed runs used the same 7,807,056-byte release executable, whose filesystem mtime is `2026-08-11T06:23:37Z` and whose observed SHA-256 is `c9c3e5718a3a7aa3531179d735783f01c254b4907dda7f1a345ae96e670b571d`. The build product under `target/release` is not checked in; those are retained observations, not a source digest pretending byte identity. After timing, the harness renamed the result selection field from the misleading `governed` to `production` and received documentation and reasoned-Clippy repairs. The two TSVs received the same label-only repair. Kernel construction, source/ABI anchors, input, oracle, preparation, warm-up, timing, and every numeric result field are unchanged. `environment.tsv`'s `hash.main` therefore pins the **replay harness**, which reproduces the experiment, and does not claim to be the exact source bytes that built the timed executable.

`threadgroup_bytes` is the prepared Metal entry's reported static allocation, not a restatement of the source request. The tree source stages exactly `4 × participants` bytes; the prepared entries in both runs report that request rounded up to 16 bytes — for example 1,028 source bytes at 257 participants become 1,040, and 2,084 at 521 become 2,096. The validator checks both relationships separately. This is an observed prepared-pipeline allocation on the named row, not a portable Metal alignment guarantee.

## The predeclared shape-aware held-out extension

[`calibrate-a-shape-aware-tree-width-cost-row`](../../../tickets/calibrate-a-shape-aware-tree-width-cost-row.md) owns the wider study. **Proposal, frozen 2026-08-11 before timing:** rows `{4, 1,024, 2,048, 16,384}` crossed with the twelve contributor counts below, 48 cells and 616 arithmetically admissible variants per run. Primary and same-matrix repeat will use the excursion's region construction, compiler source/ABI anchors, oracle, preparation, 8 warm-ups, 30 rotating rounds, 64-encode difference quotient, and qualified Apple9 controls. Arithmetic enumerates candidates; preparation remains the authority that admits or declines them.

| role | contributors | every arithmetically admissible exact width | production |
| --- | ---: | --- | ---: |
| anchor | 780 | `{2, 3, 4, 5, 6, 10, 12, 13, 15, 20, 26, 30, 39, 52, 60, 65, 78, 130, 156, 195, 260, 390}` | 260 |
| anchor | 1,042 | `{2, 521}` | 2 |
| fit | 756 | `{2, 3, 4, 6, 7, 9, 12, 14, 18, 21, 27, 28, 36, 42, 54, 63, 84, 108, 126, 189, 252, 378}` | 252 |
| fit | 779 | `{19, 41}` | 41 |
| fit | 840 | `{2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 15, 20, 21, 24, 28, 30, 35, 40, 42, 56, 60, 70, 84, 105, 120, 140, 168, 210, 280, 420}` | 280 |
| fit | 1,018 | `{2, 509}` | 509 |
| fit | 1,020 | `{2, 3, 4, 5, 6, 10, 12, 15, 17, 20, 30, 34, 51, 60, 68, 85, 102, 170, 204, 255, 340, 510}` | 255 |
| held out | 768 | `{2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384}` | 256 |
| held out | 781 | `{11, 71}` | 71 |
| held out | 960 | `{2, 3, 4, 5, 6, 8, 10, 12, 15, 16, 20, 24, 30, 32, 40, 48, 60, 64, 80, 96, 120, 160, 192, 240, 320, 480}` | 240 |
| held out | 1,022 | `{2, 7, 14, 73, 146, 511}` | 146 |
| held out | 1,046 | `{2, 523}` | 2 |

The two anchors were observed at the extreme rows in the earlier extension, so they are excluded from fit and held-out aggregates and answer recurrence only. Fit and held-out roles are contributor-grouped across all four rows. The exact adjacent `779 / 780 / 781` divisor counts are `2 / 22 / 2`; the neighbouring even `1,018 / 1,020 / 1,022` counts are `2 / 22 / 6`; `1,018 = 2 × 509` and held-out `1,046 = 2 × 523` straddle the sparse production cutoff. Production widths cover below, at, and above 256. Rows 1,024 and 2,048 lie on opposite sides of the declared 1,056 saturated-fold-step quantity; 4 and 16,384 preserve the separated extremes.

### Frozen held-out scoring

The complete deterministic protocol is in the owning ticket under **Frozen matrix and scoring protocol**; this is its operational summary. A fit observation's response is `ln(p50 / cell_raw_min_p50)`. Raw minima and predicted minima use `f64::total_cmp`, exact ties going to the narrower width. Regret is the raw ratio, median means the zero-index `sorted[len / 2]` upper median, and the existing plateau band remains `2 × (left_stddev / sqrt(30) + right_stddev / sqrt(30))`.

Three ridge families are nested. With `w = log2(width)`, `q = log2(contributors / width)`, and `r = log2(rows)`, contributor-only is an unpenalized intercept plus `[w, q, w², q², wq]`; rows-plus-contributors appends `[rw, rq, rw², rq², rwq]`; lattice appends `[rank, log2(width / previous), log2(next / width), rank × log2(width_count), (width - 256) / 256]`. At the first and last divisor the missing neighbour equals the width, making that endpoint gap zero. A singleton rank is zero.

Non-intercept features use their fit fold's population mean and population standard deviation; exact zero variance maps to zero and fixes that coefficient at zero. Ridge penalizes non-intercept coefficients only and tries `{0, 0.000001, 0.0001, 0.01, 1}`. Deterministic Gauss-Jordan normal-equation solving chooses the greatest absolute remaining pivot, lower row on an exact tie, and rejects a non-finite or at-most-`1e-12` pivot; singular zero-ridge fits therefore decline instead of gaining a post-hoc pseudoinverse.

Contributor-group leave-one-out chooses each lambda by `(worst raw regret, upper-median raw regret, outside-plateau count, lambda)`. The existing-saturation candidate has no fit and ranks widths by `max(rows × (contributors + width), 1,056 × (contributors / width + width))`, the exact width-varying cooperative-tree term. The same fit-only tuple chooses one primary family, ties preferring contributor-only, rows-plus-contributors, lattice, then existing-saturation. Production is a baseline, not eligible. All families are reported, but only the fit-selected family may support a row.

The primary-fit policy is applied unchanged to both sealed runs. Each run independently requires worst regret at most 1.10, upper median at most 1.02, at least 18/20 choices in the noise plateau, lower worst regret than production, and median no greater than production. Repeat refitting is diagnostic and changes no verdict. Anchor recurrence cannot rescue held-out failure. The analyzer also reports relative `p50` variation across all 616 rows and the selected rows, plateau agreement, and anchor boundary verdicts.

Primary and repeat must have the same measured-width population. Relative differences use `abs(primary - repeat) / primary`. Selected plateau agreement counts the 20 held-out cells where the unchanged primary policy's selected width has the same inside/outside-plateau verdict in both runs.

Those controls were satisfied before timing: the device-free mode reached all 48 cells and 616 widths with no decline, and independent source, ABI, oracle-input, owning-result-binding, and matrix perturbations failed unchanged assertions before being restored.

### The retained shape-aware result

**Measurement, 2026-08-11**, at [`results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/): [`sweep.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/sweep.tsv), [`repeat.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/repeat.tsv), exact [`environment.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/environment.tsv), and validator/scorer output [`analysis.txt`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/analysis.txt). Both sequential runs measured all 616 widths with no decline, zero Cargo/rustc/make occupancy before and after, and the same before/after executable SHA-256. Primary load was `3.62 6.52 7.58` before and `2.97 4.31 6.03` after; repeat was `3.10 4.22 5.95` before and `4.09 4.06 5.15` after. The environment otherwise equals the qualified authority row named above. The timed binary itself is not retained; its digest, 7,807,232-byte size, and UTC mtime are observations rather than a byte-identical rebuild claim.

**Measurement — no candidate clears the held-out support bar.** Contributor-only wins the predeclared fit-only selection in both primary and diagnostic repeat refitting. Its fit LOO tuple is worst `1.041761`, upper median `1.000048`, and four plateau misses. The held-out result is materially worse:

| policy | primary worst | primary median | misses | repeat worst | repeat median | misses |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| contributor-only | 1.404845 | 1.011037 | 7 | 1.416495 | 1.011997 | 8 |
| rows plus contributors | 1.404845 | 1.012906 | 8 | 1.416495 | 1.013211 | 8 |
| divisor lattice | 1.718855 | 1.014938 | 10 | 1.612238 | 1.014052 | 10 |
| existing saturation 1,056 | 2.547430 | 1.237156 | 15 | 2.406007 | 1.269829 | 15 |
| production nearest 256 | 4.603724 | 1.069190 | 15 | 5.009975 | 1.077103 | 15 |

The selected contributor-only row misses the required worst-at-most-1.10 and at-most-two-plateau-misses bounds in both runs, so its lower worst and median than production cannot rescue it. The exact finite verdict is **insufficient evidence for a replacement row**; `capped_tree_partition` remains unchanged. Across all 616 rows, primary-to-repeat relative `p50` difference has upper median `0.007233` and maximum `0.235500`; across the 20 selected held-out rows it has upper median `0.012259` and maximum `0.113438`, with plateau membership agreeing on 19/20. These tails are an additional reason not to promote a narrow fit.

**Measurement — the motivating discontinuities recur, but do not yield a general selector.** For 780 contributors, all eight run/row cells have a below-cap raw minimum and a plateau excluding production 260. Primary raw minima over rows `{4, 1,024, 2,048, 16,384}` are `{39, 15, 15, 78}` and repeat minima are `{39, 15, 15, 65}`; every run also improves from 195 to 260 and reverses by 390. For sparse 1,042 contributors, 521 is faster in six of eight run/row cells, while both 1,024-row cells are indistinguishable from width two; it is never slower. Thus the dense plateau recurs throughout this record and the sparse reversal recurs except for one row regime becoming unresolved, but the sealed neighbouring contributors show that the frozen polynomial/lattice families do not generalize with acceptable regret.

This remains one Apple M4 Max/Apple9 row, unit operands, exact-divisor candidates, and a wall-clock submission difference quotient rather than GPU-busy time. Post-hoc enrichment of the opened feature families cannot turn these held-out cells into new validation evidence. [`test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix`](../../../tickets/test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix.md) owns whether interactions between row regime and divisor-neighbour discontinuities generalize on a newly frozen contributor-grouped population; that needs a separate record (and a second qualified profile before any portable claim), not production selection here.

## The frozen fresh row-regime interaction study

[`test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix`](../../../tickets/test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix.md) owns a genuinely fresh test of that hypothesis. **Proposal, frozen 2026-08-11 before timing:** rows `{8,528,1,056,2,112,8,192}` cross anchors `{780,1,042}`, fit groups `{774,783,900,1,006,1,082,1,280}`, and sealed held-out groups `{775,785,899,1,008,1,094,1,282}`. The scoring groups appear in no predecessor calibration TSV. Exact-divisor arithmetic with quotient at least two gives 61 fit + 40 held-out + 24 anchor widths per row, 125 per row and 625 per run across 70 cells; preparation must admit all 625 or the validator rejects the record. Anchors only report recurrence and cannot fit, select, or support a model.

For `w=log2(p)`, `q=log2(k/p)`, `l=log2(rows)`, divisor rank `rank`, adjacent log gaps `g-`/`g+`, and `s=clamp(log2(rows/1,056),-1,1)`, the fitted families are A `[w,q,w²,q²,wq]`; B adding `[lw,lq,lw²,lq²,lwq]`; C adding `[rank,g-,g+,rank*log2(|P|),(p-256)/256]`; and D adding only `[s*rank,s*g-,s*g+]`. A missing predecessor or successor equals `p`, making its endpoint gap zero. Each family has an unpenalized intercept. The existing `max(rows*(k+p), 1,056*(k/p+p))` quantity, narrower on an exact tie, is the zero-fit simpler baseline. No feature may be added after opening cells.

Fit responses are log regret from the raw p50 minimum, with `f64::total_cmp` then narrower-width ties. Each fold standardizes by its population mean and standard deviation; a zero-variance column becomes zero with coefficient constrained to zero. Ridge lambdas are `{0,0.000001,0.0001,0.01,1}`. The deterministic Gauss-Jordan solver uses largest-absolute pivots, lower-row exact ties, and refuses non-finite or absolute pivots at most `1e-12`; lambda zero is unpenalized ordinary least squares and is discarded only if a contributor-group fold cannot solve.

Each fitted family selects lambda by aggregate six-fold contributor-group LOO `(worst regret, upper-median regret, plateau misses)`, lexicographically by `f64::total_cmp`, integer misses, then smaller lambda. A prediction tie chooses narrower `p`; the raw-minimum tie does too. The upper median is sorted zero-based index 15 of 30. A miss lies beyond twice the sum of candidate and raw-minimum median standard errors (`stddev/sqrt(30)`). Simpler selection minimizes that tuple in exact order A, B, C, existing saturation. D compares to C in each fold at the two families' already-selected aggregate-LOO lambdas: D must be componentwise no worse in all six folds, and in at least three folds at least one of worst, upper median, or misses must be exactly strictly smaller with no tolerance. Its aggregate tuple must also strictly beat the selected simpler family.

Only primary fit cells select and finally fit; repeat refitting is diagnostic. The primary-fit selection is scored on primary and repeat held-out cells independently. Both must have worst regret `<=1.10`, upper median `<=1.02`, at most three misses, worst strictly below unchanged production, and median no greater than production. Report-only anchor comparisons are `780:195->260`, `780:260->390`, and `1,042:2->521` at each row. Measurement otherwise preserves the retained source/ABI/oracle controls, eight warm-ups, 30 rotating rounds, batch-64 difference quotient, and sequential primary/repeat custody. This remains a one-host Apple9 observation, not GPU-busy time or a portable row.

The separate `tree-width-interactions-analysis` binary validates exact schema, population, prepared resource bounds, production marks, environment/custody fields, source and result digests before scoring. The timed binary is not retained: its before/after digest, size, and mtime identify the observed executable, while retained source/lock digests identify build inputs without asserting a byte-identical rebuild.

### The retained fresh-interaction result

**Measurement, 2026-08-11**, at [`results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/): [`sweep.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/sweep.tsv), [`repeat.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/repeat.tsv), exact [`environment.tsv`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/environment.tsv), and reproducible validator/scorer output [`analysis.txt`](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/analysis.txt). Both runs measured all 625 widths and declined zero. The widest prepared workgroup is 641; the 2,564-byte source request becomes an observed 2,576-byte prepared allocation.

Primary ran `2026-08-11T10:02:50Z`–`10:06:56Z` at load `4.33 3.79 3.75`–`3.35 3.64 3.70`; repeat ran strictly afterward `10:07:04Z`–`10:11:13Z` at `3.14 3.58 3.68`–`2.60 3.21 3.50`. A broader external check found zero Cargo, rustc, rustdoc, nextest, clippy-driver, or make processes before and after each timed run, and the harness reported zero build-process occupancy too. Both began and ended with executable SHA-256 `f9bac263c0a140f5843667550a0b5373cc7a21c784dbefb79f837bf4fc6c7b29`. The unretained binary was observed at 7,823,936 bytes with UTC mtime `2026-08-11T10:00:57Z`; retained source/lock digests identify build inputs and do not claim a byte-identical rebuild.

The predeclared richer family is rejected. D is strictly better than C in only 2/6 fit folds, so it is ineligible and primary fit selects rows-plus-contributors. That selected family's sealed held-out worst regret is `1.484044` / `1.477213`, upper median `1.000000` in both, with five plateau misses in each run: both runs fail the frozen support bar. D's held-out diagnostic has worst `1.084829` with two misses primary and `1.105080` with two misses repeat, but its fit gate failed and its repeat worst also crosses `1.10`. At report-only anchors the dense below-cap plateau recurs at all ten 780 cells; 521 beats two at seven of the ten 1,042 cells and is within noise in three.

This is evidence that the tested compact row-regime × divisor-neighbour vocabulary does not generalize on this one qualified host, not evidence for a replacement. `capped_tree_partition`, the target cost row, feasibility, request/profile surface, and identity remain unchanged. [`compare-a-target-private-tree-width-table-with-a-prime-factor-signature`](../../../tickets/compare-a-target-private-tree-width-table-with-a-prime-factor-signature.md) owns the next useful test: a capacity/interpolation/default-frozen target-private table against an explicitly encoded pre-lowering prime-factor signature on another fresh contributor-grouped matrix. Neither opened matrix may validate it, and a second profile is premature until some policy survives this one.

## The frozen target-private table versus prime-signature study

[`compare-a-target-private-tree-width-table-with-a-prime-factor-signature`](../../../tickets/compare-a-target-private-tree-width-table-with-a-prime-factor-signature.md) owns the third fresh matrix. **Proposal, frozen 2026-08-11 before timing:** rows `{16,384,1,536,6,144,12,288}` cross fit contributors `{1,080,1,215,1,320,1,512,1,638,1,890}`, sealed held-out contributors `{1,050,1,155,1,274,1,430,1,575,1,925}`, and diagnostic-only anchors `{1,024,1,729}`. Every row and contributor is absent from every earlier retained TSV. Proper-divisor arithmetic gives 152 fit, 86 held-out, and 15 anchor widths per row: 1,265 variants in 70 cells per run. Preparation must admit and verify all 1,265 and decline zero. The widest candidate is 945 participants, with a 3,780-byte source request and expected 3,792-byte prepared allocation; the maximum element count is 23,654,400.

The finite table has exactly 30 occupied entries: the primary raw-minimum width at every exact `(row, fit contributor)` key. Exact keys read their entry. In-range contributors linearly interpolate in contributor count between the adjacent entries' log2 widths and choose the admitted width at minimum absolute log-distance, narrower on a tie. Contributors below 1,080 or above 1,890, and unlisted rows, default to current production; there is no extrapolation or cross-row interpolation. Held scoring therefore exercises ten defaults and twenty interpolations, while the anchors add five of each. Contributor-group leave-one-out rebuilds the table without one group's five entries. Analysis reports every occupied, interpolated, and defaulted lookup and its complete derivation.

The competing signature is the exact 30-byte little-endian device-free record `(contributors:u64, v2:u8, v3:u8, v5:u8, v7:u8, Omega:u8, omega:u8, smallest:u64, largest:u64)`. Missing named factors have valuation zero; zero contributors are refused; one uses zero counts and endpoint sentinel one; a prime uses counts one and identical endpoints but has no proper-divisor candidate. Trial division, primality, and checked recomposition are asserted. These bytes are spike evidence and do not enter artifact or cache identity.

The signature model predicts the raw-minimum log2 width from `[log2(rows), log2(rows)^2, log2(contributors), v2, v3, v5, v7, Omega, omega, log2(smallest), log2(largest)]` plus an intercept. Ridge, standardization, zero-variance handling, deterministic solver, pivot floor, contributor-group LOO, lambda grid, objective, and narrower ties are exactly the predecessor's. At the signature's selected aggregate-LOO lambda, the 30-entry table is eligible only when its aggregate objective is componentwise no worse and it is componentwise no worse with at least one strict improvement in four of six folds. Otherwise the twelve-coefficient signature wins the complexity order.

Only primary fit cells select and finally fit. The primary-fitted policies are applied unchanged to primary and repeat held-out data; a repeat refit is diagnostic. Standard error is `stddev / sqrt(30)`, and the plateau separation rule remains exactly **twice the sum** of the candidate and raw-minimum standard errors. Both runs independently require worst regret at most 1.10, upper median at most 1.02, at most three misses, worst strictly below production, and median no worse than production; the table must also pass its complexity gate. Anchors never select or support.

The separate mode and device-free analyzer retain exact matrix/schema, source, ABI, oracle, result, environment, and executable-custody controls. The timed binary remains unretained: source and lock digests identify build inputs, while before/after executable digest, size, and mtime identify the observed binary without a byte-identical rebuild claim. No result changes production, public surface, target schema, feasibility, or identity. A survivor stops for a separate authority/threading decision because current production receives only contributors and owns no target-private table.

**Measurement, 2026-08-11 on the qualified Apple M4 Max/Apple9 host:** the retained [primary](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-table/sweep.tsv) and [repeat](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-table/repeat.tsv) each measured all 1,265 predeclared widths in 70 cells, declined zero, reached P945 and 3,792 prepared bytes, and preserved the same unretained executable digest before and after. Primary ran `2026-08-11T11:22:31Z`–`11:40:38Z` at load `3.73 4.07 4.37`–`2.75 2.72 3.18`; repeat ran `11:40:50Z`–`11:58:38Z` at `2.94 2.77 3.19`–`3.42 3.30 3.19`. The [environment and custody record](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-table/environment.tsv) equality-pins those observations, the exact toolchain, every source/result digest, and the zero-process reservation.

The frozen [analysis](results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-table/analysis.txt) rejects both candidates. At lambda 1 the signature's fit LOO objective is worst `1.676963`, upper median `1.025229`, and 19 misses. The table is strictly better in only 2/6 folds, is ineligible, and primary selection therefore chooses the signature. On sealed held cells the signature records `1.318962 / 1.005762 / 7` primary and `1.439519 / 1.006505 / 10` repeat; the table records `2.794883 / 1.031449 / 16` and `2.542376 / 1.025240 / 15`. Both fail the predeclared support bar. Production itself is worse on worst regret and misses (`3.150855 / 1.072583 / 27` and `3.165281 / 1.070154 / 28`), but that comparison does not make either failed candidate supported.

This is finite one-host evidence that neither the frozen 30-entry interpolation/default table nor the frozen prime-factor-signature model generalizes across this matrix and repeat. It is not evidence for a replacement or a portable profile claim. Production selection, cost rows, feasibility, request/profile surface, and identity remain unchanged. [`audit-the-stability-of-tree-width-optimum-labels-across-retained-matrices`](../../../tickets/audit-the-stability-of-tree-width-optimum-labels-across-retained-matrices.md) owns the device-free residual: determine whether raw-minimum and plateau labels are repeatable enough to justify any later supervised selector study, without treating any opened matrix as validation.

## Both strategies, because they consume the number differently

The split's partition count is a launch extent and its contributors-per-partition is a fold length. The tree's partition count is *also* its declared workgroup width and, through the tile's staging, its threadgroup reservation — visible in the retained `threadgroup_bytes` column, which rises from 16 bytes at two participants to 4,096 at 1,024. That column records the prepared entry's aligned allocation as distinguished above, while the scheduled tile's source requirement remains one `f32` slot per participant. A partition best for one need not be best for the other, and the retained result shows a shape where their plateaus are **disjoint**.

## What is measured, and what that number is not

Identical in kind to the retained crossover sweep. One sample is the wall clock across `commit()` and `wait_until_completed()` for one submission of one whole plan, prologue included, because that is what a consumer pays. **`metal` 0.33.0 exposes no accessor for `MTLCommandBuffer`'s `GPUStartTime` or `GPUEndTime`**; reading them would need a new `unsafe` site, which is a decision under [ADR 0079](../../../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) rather than a convenience a spike may take. **This is not a GPU-busy measurement and nothing here should be quoted as one.**

The submission round trip costs about 200 microseconds on this host before any kernel runs. Each variant is therefore measured at two encode counts — the plan once, and the plan **64** times in one command buffer — and the per-plan cost is `(batched - single) / 63`. The two submissions differ by exactly 63 extra encodes of the same plan and by nothing else, so the fixed cost divides out; it is identical across every partition, so including it could only bury a difference and never manufacture one.

**Sixty-four rather than that sweep's sixteen, and the reason is the quantity being compared.** That sweep resolved differences between *strategies*, which span two orders of magnitude; this one resolves differences between *partitions of one strategy*, which a pilot run at sixteen encodes put at a few percent on several shapes — inside the band sixteen encodes leaves. The amortized spread is `(sigma_batched + sigma_single) / (BATCH - 1)` and the single-submission spread is a floor that does not shrink, so the encode count is the only lever: sixty-four moves the band from roughly four microseconds to roughly one. The per-plan quantity is the same difference quotient either way, so rows here remain comparable to that sweep's.

**The pilot at sixteen encodes is retained beside the result**, at [`pilot-batch-16-sweep.tsv`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/pilot-batch-16-sweep.tsv), because it is corroboration and it should be checkable rather than described. It agrees on **12 of the 14 verdicts**; the two that differ are the tree at four and at 64 rows of 8,192, where the pilot could not separate the governed partition from the best and the sharper run can — the band tightening rather than a number moving.

## Noise controls

- Every variant of a shape is fully prepared — emitted, linked, pipelined, allocated, input written — before any timing starts, and the command queue is built once for the whole sweep.
- Eight untimed submissions per variant at each encode count precede the timed ones.
- The timed submissions are **interleaved across every partition and both strategies of the shape at once**: each of thirty rounds submits every variant once and the round's starting variant rotates. This is the control the comparison actually needs — the compared rows differ only in the partition, so a drift that tracked partition order would look exactly like a partition effect.
- Minimum, median, p90 and sample standard deviation are reported at both encode counts.
- Load averages are recorded before and after. This run had the machine to itself: `2.88 3.13 3.40` before and `3.03 3.18 3.40` after, which is this machine's idle desktop session and no build.

## The oracle

Every operand is `1.0`, so a row's declared sum is exactly the contributor count, representable in `f32` for every count reached here. **Every grouping of that row produces the same bits**, which is what makes one expected value valid for every partition of every strategy under a contract that *permits* regrouping, and a dropped, double-counted, or unsynchronized contributor changes the sum and is caught. Every output element of every variant is checked before that variant is timed.

**Each variant owns its output buffer**, deliberately. A shared output would let a variant that failed to write some position read the previous variant's correct answer and pass. The prologue's materialized tensor *is* shared, because every submission rewrites all of it with one invocation per element and duplicating it per variant would cost gigabytes at these shapes for nothing.

That closed form is tied to `tiler-reference`'s independent evaluation of the same semantic program once per run, at four rows of sixteen, before any shape is measured. **Regrouped rounding is not observed and is not claimed**: unit operands cannot expose it, and `drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies` owns that evidence.

## Running it

```sh
cd spikes/program-planning/reduction-partition-calibration
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-partition-sweep > results/<date>-<host>/sweep.tsv
cargo run --release --bin partition-regret -- results/<date>-<host>/sweep.tsv

# Current production tree, non-power-of-two excursion matrix.
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-partition-sweep -- \
  --tree-width-excursion > results/<date>-<host>-tree-width-excursion/sweep.tsv
cargo run --release --bin tree-width-excursion-analysis -- \
  results/<date>-<host>-tree-width-excursion/sweep.tsv \
  results/<date>-<host>-tree-width-excursion/repeat.tsv \
  results/<date>-<host>-tree-width-excursion/environment.tsv

# Frozen shape-aware held-out study. Build before the reserved window, then run
# the binary directly so the timed occupancy assertion does not see Cargo.
cargo build --release --bin reduction-partition-sweep
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  target/release/reduction-partition-sweep --shape-aware-tree-width \
  > results/<date>-<host>-shape-aware-tree-width/sweep.tsv
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  target/release/reduction-partition-sweep --shape-aware-tree-width \
  > results/<date>-<host>-shape-aware-tree-width/repeat.tsv
cargo run --release --bin shape-aware-tree-width-analysis -- \
  results/<date>-<host>-shape-aware-tree-width/sweep.tsv \
  results/<date>-<host>-shape-aware-tree-width/repeat.tsv \
  results/<date>-<host>-shape-aware-tree-width/environment.tsv

# Frozen fresh row-regime interaction study. Verify and build before the
# reserved window; invoke the already-built binary directly for timing.
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  cargo run --release --bin reduction-partition-sweep -- \
  --verify-tree-width-interactions
cargo build --release --bin reduction-partition-sweep
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  target/release/reduction-partition-sweep --tree-width-interactions \
  > results/<date>-<host>-tree-width-interactions/sweep.tsv
DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
  target/release/reduction-partition-sweep --tree-width-interactions \
  > results/<date>-<host>-tree-width-interactions/repeat.tsv
cargo run --release --bin tree-width-interactions-analysis -- \
  results/<date>-<host>-tree-width-interactions/sweep.tsv \
  results/<date>-<host>-tree-width-interactions/repeat.tsv \
  results/<date>-<host>-tree-width-interactions/environment.tsv
```

`DEVELOPER_DIR` selects the offline toolchain the [authority ledger](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s compilation-environment row names. On the host that produced the retained result the default selection *is* a newer Xcode, so the variable is load-bearing rather than defensive.

The scoring binary takes no device and can be rerun and audited anywhere. No `make` target reaches here, per [`spikes/README.md`](../../README.md).

## The retained result

**Measurement, 2026-08-07**, at [`results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/): `sweep.tsv` (122 measured variants, 8 declined) with its scoring at `regret.txt`, `environment.tsv`, and two corroborating runs — `repeat-batch-64-sweep.tsv` and `pilot-batch-16-sweep.tsv`. The environment matches the authority ledger's rows in every field — offline `Apple metal version 32023.883`, `AIR-LLD 32023.883`, Xcode 26.6 (17F113), SDK macosx 26.5 (25F70); execution macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max — under toolchain `nightly-2026-07-19`.

### Repeatability, measured rather than assumed

The whole matrix was run three times: twice at 64 encodes and once at 16. **The two 64-encode runs agree on all 14 verdicts**, and the governed partition's median cost differs between them by at most **3.27%**, with a median difference of **0.36%**. The 16-encode pilot agrees on 12 of 14, both differences being a cell it could not resolve rather than a number that moved.

The second 64-encode run is retained at [`repeat-batch-64-sweep.tsv`](results/2026-08-07-apple-m4-max-macos27.0-26A5388g/repeat-batch-64-sweep.tsv). It was taken with the same harness before one function extraction that moved no computation, which is why it is reported as a repeat rather than as the result. Every conclusion below is stated from `sweep.tsv` alone; the repeat is what bounds how much any of it depends on one draw.

**One cell is visibly noisier in the retained run than in the repeat**, and the conservative spread is what caught it: at 1,024 rows of 4,096 contributors two single-submission rounds recorded outliers near a millisecond against a typical 250 microseconds, which widens that cell's band from 1.6 to 9.5 microseconds. Its median is unchanged — 204.24 against the repeat's 204.25 — and its verdict is the same in all three runs, so nothing rests on the outlier; it is reported because a band that grew tenfold should be explained rather than quietly used.

### The partition is worth up to 5.05x, and the governed choice is a reasonable middle

Per-plan cost in microseconds. Between the best and worst *admissible* partition of one shape the span reaches **5.05x** (tree at four rows of 8,192: 9.53 at 256 participants against 48.15 at two). So the partition is not a negligible term, and a plan that chose badly could pay several times over.

The governed choice never pays anything like that: its regret reaches 1.413x for the split and 1.216x for the tree. **Nothing below refutes it as a defensible default — it refutes it as the best available one**, which is the question the ticket asks.

### The verdict, per strategy

A cell is decided by the retained crossover sweep's rule: a gap counts only when it exceeds two combined standard errors of the two medians. The `band` column is that threshold.

| shape | strategy | governed | cost | best | cost | ratio | band | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 4 x 8,192 | split | 128 | 16.24 | 256 | 11.49 | **1.413x** | 0.97 | beaten |
| 4 x 8,192 | tree | 128 | 11.60 | 256 | 9.53 | **1.216x** | 0.72 | beaten |
| 64 x 8,192 | split | 128 | 40.28 | 32 | 34.01 | 1.184x | 1.18 | beaten |
| 64 x 8,192 | tree | 128 | 34.68 | 256 | 32.13 | 1.079x | 0.92 | beaten |
| 256 x 16,384 | split | 128 | 217.24 | 64 | 205.66 | 1.056x | 1.18 | beaten |
| 256 x 16,384 | tree | 128 | 212.18 | 64 | 202.96 | 1.045x | 1.20 | beaten |
| 1,024 x 4,096 | split | 64 | 204.24 | 64 | 204.24 | 1.000x | 9.49 | **best** |
| 1,024 x 4,096 | tree | 64 | 202.39 | 64 | 202.39 | 1.000x | 4.20 | **best** |
| 4,096 x 2,048 | split | 64 | 447.29 | 2 | 424.57 | 1.054x | 1.48 | beaten |
| 4,096 x 2,048 | tree | 64 | 444.50 | 256 | 427.86 | 1.039x | 1.21 | beaten |
| 16,384 x 32 | split | 8 | 33.34 | 2 | 28.82 | 1.157x | 1.33 | beaten |
| 16,384 x 32 | tree | 8 | 51.26 | 16 | 50.81 | 1.009x | 1.72 | within noise |
| 65,536 x 16 | split | 4 | 64.00 | 2 | 58.13 | 1.101x | 1.38 | beaten |
| 65,536 x 16 | tree | 4 | 147.45 | 2 | 147.09 | 1.002x | 1.34 | within noise |

**The split's governed partition is beaten on six of seven shapes; the tree's on four of seven.** The same verdicts fall out of the independent plateau rule — the set of partitions a shape cannot separate from its own best — which puts the governed choice outside the plateau in exactly those ten cells and inside it in the other four. The two rules agreeing on all fourteen is asserted in the scoring binary rather than observed by eye, so a disagreement would fail the analysis rather than be reported as one.

### Where the best partition goes, and why one constant cannot follow it

| shape | split's best | tree's best |
| --- | --- | --- |
| 4 x 8,192 | 256 | 256 |
| 64 x 8,192 | 32 | 256 |
| 256 x 16,384 | 64 | 64 |
| 1,024 x 4,096 | 64 | 64 |
| 4,096 x 2,048 | **2** | **256** |
| 16,384 x 32 | **2** | 16 |
| 65,536 x 16 | **2** | 2 |

**The split's optimum collapses to the minimum split as the row count rises**, and this is the retained crossover sweep's own physics rather than a new effect: once the row count alone saturates the device, extra partitions add total work and stage more partials without buying parallelism, so the cheapest split is the one that splits least. Where the row count cannot saturate, more partitions shorten the critical path and win. **The tree does not collapse the same way** — it is nearly flat in the participant count at high row counts, spanning 1.023x across every admissible value at 16,384 x 32 and 1.002x at 65,536 x 16 — because its extra participants cost a wider workgroup rather than a wider launch and a second pass.

### What replaces it

A candidate rule is a cap: take the largest admissible partition not exceeding it. Scored on the full seven shapes, and then leave-one-out — the cap chosen on six shapes, paid for on the seventh, ties broken toward the smaller cap.

| strategy | rule | worst regret, full population | worst regret, leave-one-out |
| --- | --- | --- | --- |
| tree | governed | 1.216 | — |
| tree | **cap 256** | **1.008** | **1.008** |
| tree | cap 32 | 1.037 | — |
| split | governed | 1.413 | — |
| split | cap 32 | 1.379 | 2.131 |
| split | cap 256 | 1.379 | 2.131 |

**For the tree the answer is a replacement.** Leave-one-out selects cap 256 on every one of the seven folds and its held-out worst regret is 1.008, against the governed choice's 1.216. Its median regret is 1.000, and the repeat run puts the same figures at 1.012 and 1.211. That is a rule supported by evidence rather than fitted to the population it is reported on.

**For the split the answer is that no constant replaces it.** Two caps beat the governed choice on the full population — 1.379 worst and 1.018 median against 1.413 and 1.184 — but the fold that holds out four rows of 8,192 selects a cap from the other six that pays **2.131x** on it, worse than the choice being replaced. **Which cap that fold selects is not even stable**: it is 8 in this run and 16 in the repeat, because the six remaining shapes score a wide band of caps almost equally and the tie falls differently on a re-measure. A selection that unstable is not a rule.

The reason is structural rather than statistical. The split's optimum moves across two orders of magnitude of row count, so a rule that improves it has to *read* the row count, and reading the row count against a saturation threshold is exactly the target cost row `activate-measured-reduction-selection-from-a-target-cost-row` owns. **The split's partition and the split's strategy selection turn on the same machine quantity, and calibrating one without the other is not available.**

### The single value, named as the compromise it is

If one number has to serve both strategies it is **256**. It is free for the tree, at a held-out worst regret of 1.008. For the split it is one of the two best constants on this population, at 1.379 worst and 1.018 median against the governed choice's 1.413 and 1.184 — so it is an improvement, and a larger one in the median than in the worst case — but it is 1.31x to 1.38x off the split's own best at the two highest row counts, where the split wants the minimum split.

**Four thousand and ninety-six rows of 2,048 contributors is where the compromise is sharpest, because there the two strategies' plateaus are disjoint.** The split cannot separate 2 from 4 and nothing else; the tree's plateau is 256 alone. Choosing 256 costs the split 1.023x there; choosing 2 costs the tree 1.100x. Both runs at 64 encodes agree on those two disjoint plateaus. One number cannot be best for both at that shape, and that is a fact about the shape rather than about the measurement.

### The curve is not monotone, and the reason is not established here

At 256 x 16,384 and at 1,024 x 4,096, **128 partitions is a local maximum for both strategies**, worse than both its neighbours by 7.7 to 14.5 microseconds against bands of 1.18 to 6.32. The effect appears in all three runs, so it is not a sampling artifact, and it is not confined to one strategy, so it is not a property of the split's second pass or the tree's workgroup width alone.

**What causes it is not isolated by this experiment.** The partial pass's invocation `(row, p)` reads a contiguous run of `contributors_per_partition` floats, so the stride between adjacent invocations is set by the partition and a memory-access explanation is available — but this sweep varied the partition and not the access pattern, so that is a hypothesis and not a measurement. Anyone acting on the partition should read the measured curve rather than assume a single interior optimum.

## Boundary

- **One profile** (`tiler.metal.macos-apple9.msl4-0.f32.v1`), **one contract** (`FLUSH_AND_REASSOCIATE_F32`), **one program family** (multiply-add prologue into a trailing-axis sum), **`f32` only**, **one host row**, **seven shapes**. It does not generalize to another Apple family, OS row, dtype, or device.
- **Wall clock end to end, never GPU-busy time.** The binding exposes no command-buffer timestamps and none were read.
- **Sixty-four back-to-back encodes hold the device busier than one cold submission would**, so every number here is a steady-state per-plan cost and not a first-call latency. That limitation is the retained crossover sweep's too, one factor deeper.
- **The cap-256 tree rule is held out over seven shapes, not over a shape distribution.** Leave-one-out on seven cells bounds how much the rule depends on any one of them; it does not establish the rule on shapes outside this matrix, and in particular no shape here has a contributor count that is not a power of two.
- **The non-monotonicity is measured and unexplained.** No mechanism is claimed.
- **No numerical claim.** The oracle is exact by construction and cannot observe regrouped rounding.
- **Nothing here is activated.** Acting on it needs a target profile to declare the machine quantity the split's optimum turns on, which is a public boundary and an identity move.
