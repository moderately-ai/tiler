---
id: calibrate-a-shape-aware-tree-width-cost-row
title: Calibrate a shape-aware tree-width cost row
status: in-progress
priority: p3
dependencies: []
related: [measure-the-tree-width-excursion-past-the-cap, activate-measured-reduction-selection-from-a-target-cost-row, test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix]
scopes: [research/program-planning]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [reductions, scheduling, measurement]
claimed_from: todo
assignee: sol-tree-cost-row
lease_expires_at: 1786445695
---
## The question

What target cost row should choose the single-workgroup reduction tree's participant count from the arithmetically admissible exact divisors of a contributor count that the qualified target admits, when neither a constant cap nor nearest-to-cap distance explains the measured optimum?

This is a wider selection study, not authority to change `capped_tree_partition`. It exists because [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) measured two different failures of the current nearest-256 rule on the qualified Apple9 row:

- **Measurement — dense lattice:** at 780 contributors, production selects 260, while 39 is the primary run's raw minimum at both 4 and 16,384 rows. Production costs 1.841x and 1.035x the primary minimum respectively. The repeat preserves both boundary verdicts and keeps 39 in the indistinguishable-from-best plateau at both row counts, but its 16,384-row raw minimum is 52 rather than 39. The curve improves from 195 to 260 and then reverses sharply by 390, so neither "never cross the cap" nor "take the nearest exact divisor" describes it.
- **Measurement — sparse cutoff:** at 1,042 contributors the only arithmetically admissible exact widths are 2 and 521, and the qualified target admitted both in this retained population. Production selects 2, but 521 is faster at both row counts and in both retained runs: 1.277x production regret at 4 rows and 1.012x at 16,384 in the primary run. The eight predeclared boundary verdicts across 514, 780, and 1,042 contributors repeat eight for eight.

Those are six host-specific cells, not a shape distribution. They refute a universal reading of the current heuristic but are too small a population to replace it.

## Experiment boundary

Design a finite matrix before timing that varies row count across and around the retained reduction-strategy saturation contour, and varies contributor counts across multiple divisor-lattice shapes:

- sparse lattices with only two arithmetically admissible exact widths on both sides of the current cutoff, while retaining any target decline rather than calling arithmetic alone legal;
- dense lattices whose best width could lie below, near, or above 256;
- counts where the current rule chooses below and above the cap; and
- neighbouring counts whose divisor sets change discontinuously, so a smooth formula cannot look supported merely because the population was smooth.

Reuse the retained partition-calibration region construction, compiler source/ABI anchor, oracle, preparation, 64-encode difference quotient, interleaving, repetition count, and qualified-host controls. Assert the exact shape and participant populations, retain a same-matrix repeat, and perturb each validator subject without changing its assertion. Analyze candidate rows with held-out shapes; report regret against every measured target-admitted width and not only against the current production choice.

The study should compare at least these candidate information sets separately: contributor count alone; row count plus contributor count; divisor-lattice summaries available before lowering; and the target saturation quantity already used by the reduction strategy cost row. The saturation quantity is available to offline scoring and to the later complete-plan selector, but the current `capped_tree_partition` construction site does not receive it; evidence supporting that candidate therefore does not by itself authorize or demonstrate a production consumer. A candidate that needs a new public request field, changes artifact identity, moves a feasibility decision into cost, or requires a consequential new construction-time authority stops for its own architecture ticket.

## Frozen matrix and scoring protocol — 2026-08-11, before timing

**Proposal — finite matrix.** Measure rows `{4, 1,024, 2,048, 16,384}` at twelve contributor counts, 48 cells and exactly 616 arithmetically admissible widths per run. The already-seen anchors `{780, 1,042}` are excluded from fitting and held-out aggregates; they only answer whether the dense plateau and sparse reversal recur. Fit-only contributors are `{756, 779, 840, 1,018, 1,020}`. Sealed held-out contributors are `{768, 781, 960, 1,022, 1,046}`. This makes 8 anchor, 20 fit, and 20 held-out cells. The exact participant populations and their 154-width-per-row census live beside the harness README and are validator assertions rather than prose alone.

The split is contributor-grouped: every row of one contributor count stays on the same side. The adjacent `779 / 780 / 781` lattices have `2 / 22 / 2` widths; neighbouring even counts `1,018 / 1,020 / 1,022` have `2 / 22 / 6`; and sparse `1,018 = 2 × 509` and held-out `1,046 = 2 × 523` sit on opposite sides of production's cutoff. Current production chooses below, at, and above 256 in the population. Rows 1,024 and 2,048 straddle the existing saturated-fold-step row's 1,056, while 4 and 16,384 retain the separated extremes.

**Proposal — observations and ties.** A cell enumerates every exact `p` with `2 <= p <= contributors / 2`; preparation may still decline one, and the analyzer scores only target-admitted measured rows. No measured width or a declined production width aborts validation. The response for a fit observation is natural-log regret `ln(p50 / cell_raw_min_p50)`. A raw minimum is the least `p50` under `f64::total_cmp`, with narrower `p` breaking an exact tie. Regret is the chosen raw `p50` divided by that raw minimum. The reported median is the upper median, zero-index `sorted[len / 2]`. A chosen width is outside the noise plateau only when its median exceeds the raw minimum by more than `2 × (chosen_stddev / sqrt(30) + minimum_stddev / sqrt(30))`.

**Proposal — the three fitted information sets.** The action `p` and exact quotient `contributors / p` are candidate properties, not extra shape inputs. Write `w = log2(p)`, `q = log2(contributors / p)`, and `r = log2(rows)`. Every fitted family has an unpenalized intercept of exactly one.

1. Contributor-only uses the non-intercept vector `[w, q, w², q², wq]`.
2. Rows plus contributors appends `[rw, rq, rw², rq², rwq]`.
3. Divisor-lattice appends to family 2 `[rank, previous_gap, next_gap, rank × log2(width_count), (p - 256) / 256]`. `rank` is the zero-based index divided by `width_count - 1`, or zero for a singleton population. `previous_gap = log2(p / previous)` and `next_gap = log2(next / p)`; at the first width `previous = p`, and at the last `next = p`, so the missing endpoint gap is exactly zero.

For each fold, every non-intercept feature is standardized from that fold's fit observations using population mean and `sqrt(sum((x - mean)²) / count)`. An exactly zero scale maps that feature to zero for both training and inference and fixes its coefficient at zero. The response is not standardized. Ridge minimizes `sum((y - Xβ)²) + lambda × sum(β[j]²)` for non-intercept coefficients only, over `lambda` in `{0, 0.000001, 0.0001, 0.01, 1}`.

Normal equations are solved by deterministic Gauss-Jordan elimination in column order. Each pivot is the remaining row with greatest absolute pivot value, an exact tie choosing the lower row index. A non-finite pivot or absolute pivot at most `1e-12` makes that fit invalid; this is how a singular `lambda = 0` behaves rather than receiving a pseudoinverse chosen after measurement. An invalid lambda has infinite objective and cannot win; every lambda invalid is a validator failure.

**Proposal — fit-only model and family choice.** Contributor-group leave-one-out holds out all four cells of each fit contributor in turn. For a lambda, the fitted model chooses the target-admitted width with the least predicted response under `f64::total_cmp`, exact ties going narrower. Its objective across the 20 left-out cells is the lexicographic tuple `(worst raw regret, upper-median raw regret, outside-plateau count, lambda)`; this picks the family's lambda. The fourth information set is the existing saturation quantity with no fit or intercept: it chooses the narrower exact tie minimizing `max(rows × (contributors + p), 1,056 × (contributors / p + p))`, exactly the width-varying cooperative-tree term consumed later by the complete-plan selector. The production nearest-256 rule is a reported baseline, not an eligible fitted candidate.

Before opening the sealed cells, the fit-only objectives choose one primary family from contributor-only, rows-plus-contributors, lattice, and existing-saturation. The same lexicographic tuple applies, omitting lambda after each fitted family has selected its own; an exact tie chooses the least-information family in that written order. Every fitted family is then refit on all 20 primary fit cells at its selected lambda. The chosen primary family, coefficients, feature means/scales, and every prediction are retained. Repeat refitting is a stability diagnostic only and cannot change the selected family, lambda, coefficients, or support verdict.

**Proposal — held-out support bar and repeat.** Every family and both baselines report held-out worst and upper-median regret, plateau misses, and per-cell choices. Only the fit-selected primary family is eligible to support a row. Its unchanged primary-fit policy must independently satisfy all of these on both the primary held-out run and the repeat held-out run: worst regret at most 1.10, upper median at most 1.02, at least 18 of 20 choices inside the run's plateau, strictly lower worst regret than production, and median regret no greater than production. The comparative clauses apply separately to each run. Failure on either run means this finite population does not support the row. Anchors report recurrence separately and cannot rescue held-out failure.

Require primary and repeat to have the same measured-width population. Across that population — all 616 rows when this qualified target admits every arithmetic candidate — retain primary-to-repeat relative `p50` upper median and maximum, using `abs(primary - repeat) / primary`, and the same differences for the selected rows. Plateau-membership agreement counts the 20 held-out cells where the unchanged primary policy's selected width has the same inside/outside-plateau verdict in primary and repeat. Also retain anchor boundary verdicts. Any exact population, schema, digest, source/ABI/oracle, environment, occupancy, or load-row mismatch aborts before scoring. The timed executable's before/after digest identifies bytes that ran but does not claim those unretained binary bytes can be reconstructed byte-identically from retained source.

## Pre-timing control evidence — 2026-08-11

**Fact — device-free population validation.** A direct `--verify-shape-aware-tree-width` run on the qualified device reached all 48 cells, verified all 616 arithmetic widths, declined zero, and observed the same executable SHA-256 before and after. This mode compiles, anchors, prepares, submits once, and checks every output but performs no warm-up or timed sample.

**Fact — independent subject perturbations, assertions unchanged.** Each subject was changed alone, rebuilt, watched failing, and restored before the next:

- translation-unit composition: `4x780: the rebuilt single-workgroup tree at the production participant count (260) does not emit the source the compiler emits`;
- published ABI launch grid: `the compiler publishes tree launch extents Launch { grid_threads: 1040, threads_per_workgroup: 260 } and this spike derives Launch { grid_threads: 1041, threads_per_workgroup: 260 }`;
- oracle input: `output[0] is 1560 (44c30000), expected 780 (44430000)`;
- owning result binding: `output[0] is 0 (00000000), expected 780 (44430000)`; and
- held-out contributor population, changing 1,046 to 1,048: `the frozen shape-aware contributor population moved`, with the full unequal arrays printed.

The retained-result, environment, and executable-custody subjects do not exist until the quiet-window runs and remain required after retention. No timed submission had occurred when these controls were recorded.

## Outcome — qualified-host measurement complete, 2026-08-11

**Fact — retained custody.** The primary [`sweep.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/sweep.tsv), [`repeat.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/repeat.tsv), exact [`environment.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/environment.tsv), and reproducible [`analysis.txt`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/analysis.txt) are retained. Each run measured all 616 widths with no decline. Both began and ended with zero Cargo/rustc/make occupancy and executable SHA-256 `56fa8152ff5f5ff53c225398082e9f20e808df8208fc7a617023bb9912a34a59`. The unretained binary's corrected UTC mtime is `2026-08-11T08:16:16Z`; an earlier live terminal note appended `Z` to a local-time `stat` display and was discarded before retention.

**Fact — post-retention perturbations, validator unchanged.** A changed result with its old digest failed `sweep.tsv digest moved`. After coherently re-digesting separate disposable copies, a P2 allocation changed from 16 to 32 failed `P2 prepared threadgroup allocation moved`; deleting P523 at `16384x1046` and changing the reported measured count failed `exact outcome population moved: missing [(16384, 1046, 523)]; unexpected []`; changing primary load to zero failed `primary: load-before row does not match the retained environment`; and changing the starting executable digest failed `primary: starting executable digest does not match retained custody`. Changing only the environment toolchain to `nightly-1970-01-01-deadbeef` failed ``environment key `environment.toolchain` moved``. The positive record validates before scoring.

**Measurement — support bar failed.** Contributor-only wins primary fit-only selection and the diagnostic repeat refit. Its primary fit LOO tuple is worst `1.041761`, upper median `1.000048`, four plateau misses, with `lambda = 1`. Held out, the unchanged primary policy reaches `1.404845 / 1.011037 / 7` (worst / upper median / misses) in primary and `1.416495 / 1.011997 / 8` in repeat, failing the predeclared `1.10 / 1.02 / 2` bar in both. Rows-plus-contributors reaches `1.404845 / 1.012906 / 8` and `1.416495 / 1.013211 / 8`; divisor-lattice `1.718855 / 1.014938 / 10` and `1.612238 / 1.014052 / 10`; existing saturation `2.547430 / 1.237156 / 15` and `2.406007 / 1.269829 / 15`; production nearest-256 `4.603724 / 1.069190 / 15` and `5.009975 / 1.077103 / 15`. No candidate supports a row.

**Measurement — recurrence and repeat limits.** All eight 780-contributor run/row cells have below-cap raw minima and plateaus excluding production 260; primary minima are `{39, 15, 15, 78}` and repeat minima `{39, 15, 15, 65}` in row order `{4, 1,024, 2,048, 16,384}`. Width 521 is faster than two in six of eight 1,042-contributor cells and indistinguishable in the two 1,024-row cells, never slower. Across all 616 rows, primary-to-repeat relative `p50` difference has upper median `0.007233` and maximum `0.235500`; across selected held-out rows it is `0.012259` and `0.113438`, with plateau membership agreeing 19/20.

**Conclusion.** This exact Apple M4 Max/Apple9 population confirms the dense discontinuity and most of the sparse reversal, but it does not support any frozen candidate on neighbouring held-out contributors. Keep `capped_tree_partition` unchanged. [`test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix`](test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix.md) owns the next discriminating unknown on a new sealed population; this opened matrix cannot validate a post-hoc family, and a second qualified target profile is required before any portable claim. The ticket remains `in-progress` for independent review and coordinator closure.

## Closes when

A retained qualified-host record either supports one explicit shape-aware participant-selection row under held-out scoring, or shows that this finite population still cannot support one and names the next discriminating unknown. The outcome must state whether the dense-lattice below-cap optimum and sparse-cutoff reversal recur, the worst and median regret of every candidate, repeat variance, exact environment, and unsupported profiles. No production selection changes in this ticket.
