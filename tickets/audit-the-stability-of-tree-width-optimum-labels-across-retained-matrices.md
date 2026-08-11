---
id: audit-the-stability-of-tree-width-optimum-labels-across-retained-matrices
title: Audit the stability of tree-width optimum labels across retained matrices
status: in-progress
priority: p3
dependencies: [compare-a-target-private-tree-width-table-with-a-prime-factor-signature]
related: []
scopes: [research/program-planning, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, research]
claimed_from: todo
assignee: sol-tree-label-stability
lease_expires_at: 1786458718
---
## The question

**Proposal.** Are raw-minimum width labels and noise-qualified plateau memberships repeatable enough across the primary/repeat pairs of the three opened 2026-08-11 shape-aware, interaction, and table/signature matrices to be meaningful supervised targets?

## Boundary

This is a device-free audit of already-retained rows. The three opened matrices are evidence about label and plateau stability only. They must not fit, tune, choose, or validate a selector, feature, table, threshold, or production rule, and no result may be presented as held-out policy evidence. Do not run Metal, add a fourth matrix, change `capped_tree_partition`, or change any production, public/API, profile, identity, feasibility, cost-row, or unsafe-code authority.

Before reading cross-run outcomes, freeze in this ticket the exact row populations and each record's own retained noise rule; exact raw-minimum tie order; primary-to-repeat and repeat-to-primary width agreement; log2-width movement; plateau-set overlap; reciprocal regret and plateau-membership checks; aggregation and missing-row behavior; and numeric thresholds that distinguish stable named subsets from unstable labels. Validate every input digest, schema, population, and environment record before computing a stability metric. Perturb independent result, population, tie, noise-rule, and digest subjects with assertions unchanged.

The stability audit is a different observable from selector regret: it asks whether the measured label and admissible plateau themselves recur under repetition. It supplies no pre-lowering selector input. If stable named subsets emerge, a later ticket may propose a fresh predeclared candidate only after naming a genuinely different request-known pre-lowering observable and proving its construction-time availability, owning target authority, feasibility separation, and identity consequence before any timing. None of the three opened matrices may validate that later candidate.

## Frozen device-free protocol — 2026-08-11, before cross-record analysis

**Proposal — source state and restraint.** The predecessor summaries and their within-record repeat diagnostics were read during the source-first audit. No cross-record label-stability metric was computed before this protocol was frozen. The analyzer may not fit a selector, feature, table, threshold, or production rule, and no output-derived contributor, row, divisor-count, or other slice may be named stable. The thresholds below are descriptive decision bars, not confidence intervals, and may not move after output.

**Proposal — exact inputs and fail-closed validation.** The immutable inputs are the three retained primary/repeat pairs: shape-aware has four rows by twelve contributors, 48 cells and 616 widths per run; interactions has five rows by fourteen contributors, 70 cells and 625 widths per run; and table/signature has five rows by fourteen contributors, 70 cells and 1,265 widths per run. The combined population is exactly 188 cells and 2,506 widths per run, or 5,012 measured rows across the pairs. Existing roles remain exactly shape-aware `8 anchor / 20 fit / 20 held`, interactions `10 / 30 / 30`, and table/signature `10 / 30 / 30`.

Before one stability metric is computed, validate each environment file by exact SHA-256, exact two-column schema and key population, and its claim-bearing matrix, sample, and environment values. Recompute each result SHA-256 against that environment's `hash.sweep` or `hash.repeat`; then validate the exact result schema and metadata, 30 repetitions, positive finite p50, nonnegative finite standard deviation, complete expected arithmetic population, zero declines, exactly one current-production mark per cell, the prepared-allocation relationship, primary/repeat key equality, and every census above. Historical source-digest rows remain retained observations; this audit does not claim the predecessor analyzer sources rebuild or replay byte-identically at this later base.

No row is skipped. A missing, unexpected, or duplicate width; missing cell; mismatched primary/repeat key; empty plateau; non-finite derived value; digest move; or environment mismatch aborts the whole audit before aggregation. There is no available-case denominator, imputation, or repaired input.

**Proposal — each record's own noise and tie rule.** Keep three named noise-rule subjects even though their arithmetic is the same. Shape-aware is sourced by its frozen ticket and analyzer, interactions by its frozen ticket and analyzer, and table/signature by its retained `analysis.noise_band = 2 * (SE_a + SE_b)` environment row. Within each record, `SE = stddev / sqrt(30)`, and a candidate belongs to that run's plateau inclusively when `candidate.p50 - raw_minimum.p50 <= 2 * (candidate.SE + raw_minimum.SE)`. No noise is pooled and no confidence interpretation is added.

A run's raw minimum is the least p50 under `f64::total_cmp`, with the narrower participant count breaking an exact p50 tie. Plateau sets contain participant counts rather than positions, and the raw minimum must belong to its own nonempty plateau.

**Proposal — symmetric cell metrics.** For each cell retain all of:

1. exact-width agreement, `primary_best_P == repeat_best_P`;
2. symmetric movement in octaves, `abs(log2(primary_best_P) - log2(repeat_best_P))`, equivalently `abs(log2(primary_best_P / repeat_best_P))`;
3. plateau Jaccard `|A intersection B| / |A union B|`, both directional containments `|A intersection B| / |A|` and `|A intersection B| / |B|`, and symmetric containment as the smaller directional value;
4. primary-best cost in repeat divided by repeat's raw-minimum cost, repeat-best cost in primary divided by primary's raw-minimum cost, and symmetric reciprocal regret as the larger ratio; and
5. primary best in the repeat plateau, repeat best in the primary plateau, and the conjunction called reciprocal plateau membership.

All ratios read validated positive p50 values. Both cross-evaluated widths must exist because population equality was already proved. Exact ordering uses `f64::total_cmp`; finite threshold comparisons use the inclusive `<=` and `>=` spellings below.

**Proposal — frozen aggregations.** The only reportable subsets are `overall` with 188 cells; each study, `shape-aware` 48, `interactions` 70, and `table` 70; pooled roles, `anchor` 28, `fit` 80, and `held` 80; and the nine study-by-role subsets: `shape-aware/anchor` 8, `shape-aware/fit` 20, `shape-aware/held` 20, `interactions/anchor` 10, `interactions/fit` 30, `interactions/held` 30, `table/anchor` 10, `table/fit` 30, and `table/held` 30.

For every subset report its cell count; exact-agreement count and rate; movement upper median, p90, and maximum; Jaccard lower median and minimum; symmetric-containment lower median and minimum; both directional plateau-membership counts plus reciprocal count and rate; and reciprocal-regret upper median, p90, and maximum. Sort floating values with `f64::total_cmp`. The upper median is `sorted[N / 2]`, the lower median is `sorted[(N - 1) / 2]`, and p90 is `sorted[ceil(0.9 * N) - 1]`. A rate bar becomes the exact integer count `ceil(rate * N)`.

**Proposal — frozen stability bar.** The same bar applies independently to `overall` and every named subset. A subset is stable if and only if all six clauses hold:

- exact raw-minimum agreement is at least `ceil(0.80 * N)`;
- p90 symmetric log2 movement is at most `1.0` octave;
- lower-median plateau Jaccard is at least `0.50`;
- lower-median symmetric plateau containment is at least `0.75`;
- reciprocal plateau membership is at least `ceil(0.90 * N)`; and
- symmetric reciprocal regret has upper median at most `1.02`, p90 at most `1.10`, and maximum at most `1.25`.

The first clause demands four-in-five exact labels. The second prevents the nonexact tail from moving by more than a factor of two in nine-in-ten cells. Jaccard and containment together distinguish genuine overlap from one plateau merely swallowing another. Reciprocal membership requires both run minima to remain mutually admissible in nine-in-ten cells. The regret bar reuses the predecessors' `1.02` median and `1.10` bounded-regret scale and adds the predeclared `1.25` worst-outlier ceiling.

**Proposal — verdict precedence.** Report `stable-across-records` only if `overall` and all three study subsets pass. Otherwise, if at least one of the nine predeclared study-by-role subsets passes, report `stable-named-subsets-only` and list exactly those passing subsets; pooled-role and study summaries cannot manufacture an unnamed hypothesis. If none of the nine passes, report `unstable-labels`, pause same-host supervised tree-width-selector studies, and file only a separately authorized different-measurement-quantity research or decision remainder. Any verdict remains bounded to these opened one-host records and cannot validate a policy.

**Proposal — controls.** With assertions and retained inputs unchanged, independently perturb and restore a result value, one population row, the comparator subject exercised by an equal-p50 synthetic tie, one record's noise-rule subject, and one input or environment digest. Each must fail with its own reason before the positive retained output is trusted.

## Outcomes

- **Unstable labels:** pause same-host supervised tree-width-selector studies until a separately authorized ticket defines a different measurement quantity.
- **Stable named subsets:** record only the bounded subset and stability metric; any selector hypothesis moves to a later fresh, predeclared matrix and the authority proof above.

Retain a replayable analyzer, input digests, exact population census, thresholds, and result. This ticket never authorizes production selection.

## Outcome — 2026-08-11

**Measurement.** The separate device-free analyzer validated all three exact environment SHA-256 values, each environment's exact claim-bearing key population, all six result digests, schemas, metadata/custody rows, zero declines, current-production marks, and arithmetic populations before scoring. The accepted census is 188 cells, 2,506 widths per run, and 5,012 measured rows. The retained input-manifest digest is `24d9c60f8a116a2ed4245d3709b1303127d78c8800c1b6a7e7df5609511ada12`; the retained [`analysis.txt`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-tree-width-label-stability/analysis.txt) digest is `6df96e8c907891b03bdcd6484a6dc9ccd45c89228ea99ac18ce919b4e2928e88`. Exact replay prints `# exact_replay passed`.

**Measurement.** The frozen verdict is `stable-named-subsets-only`. Overall exact raw-minimum agreement is 139/188 (`0.739361702`), so `overall` misses the predeclared `ceil(0.80*N)` clause. Every cell has reciprocal plateau membership; overall movement p90 is `0.862496476` octaves, reciprocal-regret p90 is `1.011071220`, and maximum reciprocal regret is `1.159887165`. Only `interactions` passes at whole-study level. The exact predeclared study-by-role subsets that pass are `shape-aware/held`, `interactions/fit`, `interactions/held`, and `table/anchor`; no output-derived subset was added.

**Inference.** The named subsets are bounded repeatability observations, not a learned width-selector hypothesis and not policy evidence. The overall and two-of-three study failures rule out `stable-across-records`; the four named passes do not authorize fitting on any opened cell. Production selection, cost rows, feasibility, request/profile surface, and identity remain unchanged. Any later supervised hypothesis still needs a genuinely different request-known observable with construction/authority/identity analysis and a fresh predeclared matrix.

**Controls.** Assertions and positive retained inputs stayed unchanged while each subject was independently broken and restored. All probes exited 101:

- digest: changing the manifest's shape-aware environment digest failed `shape-aware: environment digest moved` and printed the exact differing digests;
- population: deleting only `(rows=4, contributors=780, P=12)` while coherently updating its result/environment digest chain failed `shape-aware: exact outcome population moved: missing [(4, 780, 12)]; unexpected []`;
- result: replacing only shape-aware primary `4x780 P39` amortized p50 with a positive finite value while coherently updating its digest chain passed input validation, then exact replay failed `retained stability analysis moved: expected 6df96e8c907891b03bdcd6484a6dc9ccd45c89228ea99ac18ce919b4e2928e88, observed 59f4d5cad992ece0d0982d2f4916d3298f36a29686f1b334ec31cdca49e5e712`;
- tie: reversing only the narrower-width comparator subject failed the unchanged equal-p50 fixture with `raw-minimum narrow tie moved`, `left: 4`, `right: 2`; and
- noise: changing only shape-aware's multiplier from `2.0` to `1.0` failed the unchanged source-specific self-check with `shape-aware retained noise multiplier moved`, `left: 4607182418800017408`, `right: 4611686018427387904`.

The result and population probes recomputed the disposable result and environment digests so a stale custody row could not mask the semantic assertion under test. The repository subjects were restored, the analyzer was rebuilt, `--self-check` passed at 188/2,506/5,012, and the exact retained replay passed again. No Metal command, device timing, measuring-harness mode, or production path ran in this audit.
