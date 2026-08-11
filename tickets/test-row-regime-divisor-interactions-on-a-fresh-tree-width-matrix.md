---
id: test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix
title: Test row-regime and divisor-neighbour interactions on a fresh tree-width matrix
status: done
priority: p3
dependencies: [calibrate-a-shape-aware-tree-width-cost-row]
related: [measure-the-tree-width-excursion-past-the-cap]
scopes: [research/program-planning]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [reductions, scheduling, measurement]
---
## The question

Can pre-lowering interactions between row regime and divisor-neighbour structure predict a single-workgroup reduction tree width on a newly frozen contributor-grouped population, or is this target row too discontinuous for the tested compact model class?

## Why this is separate

[`calibrate-a-shape-aware-tree-width-cost-row`](calibrate-a-shape-aware-tree-width-cost-row.md) opened its sealed cells on 2026-08-11. Its retained [`analysis.txt`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/analysis.txt) selects contributor-only on fit but records held-out worst regret `1.404845` / `1.416495` and seven / eight plateau misses, failing the frozen support bar in both runs. Rows-plus-contributors and the additive divisor-lattice family also fail. Those cells may motivate a hypothesis but cannot validate a post-hoc interaction family.

The same record shows why the unknown remains discriminating. **Measurement:** at 780 contributors every row/run has a below-cap optimum plateau excluding production 260, while sparse 1,042 prefers 521 in six cells and is unresolved in two. **Measurement:** raw minima move across the neighbouring contributor groups, and, separately, every frozen compact candidate misses the held-out bar; the record does not establish that the first observation caused the second.

The predecessor remains historical exact-source evidence and does not replay green at this ticket's base. Running its documented analyzer at `b0c41639f9ad266879ef52dfaee8de5e35eb47f9` stops with `../../../crates/tiler-compiler/src/physical.rs digest moved`: current `40a2f48a4a10383e1e1775c73a1fe95aacb2e1f2674cbbc8022533ff4ef1adac` versus retained `82139b1f5e5885b9d861f05a23628016033677ff0f2c30357969b752059bcd3d`. The sole intervening `physical.rs` change is `2de8c61b12330a8450b05ce4016af7478bb7f380`, which removed two dead-code `#[allow]` attributes without changing the partition construction or consumer. This study must pin the current source digest in its own record and must not claim that the predecessor analysis replayed on this base.

## Experiment boundary

Before any new device timing, predeclare:

- a fresh contributor-grouped fit/held-out split disjoint from the 2026-08-11 fit and held-out contributors;
- exact row regimes, arithmetic width populations, target-admission handling, primary/repeat policy, and noise/support bars;
- a minimal finite interaction vocabulary derived only from information available before lowering, such as row-regime × neighbour-gap or row-regime × width-rank terms; and
- a complexity penalty or nested comparison that can reject the richer family rather than selecting it merely because it has more terms.

Reuse the retained source/ABI/oracle/custody controls. Treat the Apple M4 Max/Apple9 result as host-specific and require a second qualified profile before any portable row. If a useful policy needs a public request field, artifact identity change, a feasibility move, device counters requiring a new unsafe site, or new construction-time authority, stop and file that architecture remainder separately.

## Frozen protocol

**Proposal, frozen 2026-08-11 before timing:** cross rows `{8, 528, 1,056, 2,112, 8,192}` with historical recurrence anchors `{780, 1,042}`, fresh fit contributors `{774, 783, 900, 1,006, 1,082, 1,280}`, and sealed held-out contributors `{775, 785, 899, 1,008, 1,094, 1,282}`. The twelve scoring groups are absent from every predecessor calibration TSV; the two anchors are reported only and never enter fitting, family selection, or the support verdict. Exact-divisor arithmetic with quotient at least two gives 61 fit, 40 held-out, and 24 anchor widths per row: 125 per row, 625 per run, 70 cells (30 fit, 30 held-out, 10 anchor). Preparation remains the target-admission authority. This matrix expects 625 admitted and zero declined rows but will retain and reject any different result population rather than silently rescore it.

The exact width populations are:

- fit: `774 -> {2,3,6,9,18,43,86,129,258,387}`, `783 -> {3,9,27,29,87,261}`, `900 -> {2,3,4,5,6,9,10,12,15,18,20,25,30,36,45,50,60,75,90,100,150,180,225,300,450}`, `1,006 -> {2,503}`, `1,082 -> {2,541}`, `1,280 -> {2,4,5,8,10,16,20,32,40,64,80,128,160,256,320,640}`;
- held out: `775 -> {5,25,31,155}`, `785 -> {5,157}`, `899 -> {29,31}`, `1,008 -> {2,3,4,6,7,8,9,12,14,16,18,21,24,28,36,42,48,56,63,72,84,112,126,144,168,252,336,504}`, `1,094 -> {2,547}`, `1,282 -> {2,641}`; and
- report-only anchors: the 22 exact widths of 780 and `{2,521}` for 1,042.

For rows `r`, contributors `k`, candidate width `p`, ordered exact-width set `P`, zero-based position `i`, predecessor `p-`, and successor `p+`, define `w=log2(p)`, `q=log2(k/p)`, `l=log2(r)`, `rank=i/(|P|-1)` (or zero for a singleton), `g-=log2(p/p-)`, `g+=log2(p+/p)`, and `s=clamp(log2(r/1,056),-1,1)`. At the first endpoint `p-=p`; at the last `p+=p`, so the absent neighbour's gap is exactly zero. Each fitted family has an unpenalized intercept:

- A, contributor-only: `[w,q,w²,q²,wq]`;
- B, rows-plus-contributors: A plus `[lw,lq,lw²,lq²,lwq]`;
- C, divisor-lattice: B plus `[rank,g-,g+,rank*log2(|P|),(p-256)/256]`; and
- D, interaction: C plus only `[s*rank,s*g-,s*g+]`.

The existing saturated-fold-step quantity `max(r*(k+p), 1,056*(k/p+p))`, with narrower `p` on an exact integer tie, is the fourth simpler family and is not fitted. No other interaction or basis term may be added after cells open.

Fit responses are `ln(candidate p50 / cell raw-minimum p50)`. A raw minimum uses `f64::total_cmp` and narrower `p` on an exact tie. Features use population mean and population standard deviation over the current fold's training observations. A zero-variance column standardizes to zero and is constrained to coefficient zero. Ridge candidates are exactly `{0, 0.000001, 0.0001, 0.01, 1}`; the intercept is not penalized. Deterministic Gauss-Jordan elimination uses the largest absolute pivot, the lower row on an exact pivot tie, and refuses a non-finite or absolute pivot at most `1e-12`. Lambda zero is ordinary least squares and remains a candidate only if every contributor-group leave-one-out fold solves; any lambda with one singular fold is discarded.

For each fitted family independently, lambda minimizes the aggregate six-fold contributor-group LOO tuple `(worst regret, upper-median regret, plateau misses)` lexicographically using `f64::total_cmp`, integer comparison, then smaller lambda. Each fold trains on five whole contributor groups and scores all five row cells of the sixth. Regrets are sorted with `f64::total_cmp`; the upper median is zero-based index 15 of 30. A predicted-width tie uses `f64::total_cmp` then narrower `p`. A plateau miss means the chosen p50 exceeds the raw minimum by more than twice the sum of their median standard errors, with each standard error `sample stddev / sqrt(30)`.

The exact simpler-family selection order is lexicographic objective followed, on an exact objective tie, by A, then B, then C, then existing saturation. D's eligibility compares C and D in every fit-group fold at each family's already-selected aggregate-LOO lambda. D is componentwise no worse in a fold only when its worst and upper median are no greater under `f64::total_cmp` and its misses are no greater by integer comparison. It is strictly better only when it is componentwise no worse and at least one component is strictly smaller, with no tolerance. D must be no worse in all six folds, strictly better in at least three, and have an aggregate LOO tuple strictly better than the already-selected simpler winner; otherwise the simpler winner remains selected.

Only the primary fit population selects and finally fits a family. A repeat refit and D-eligibility result are diagnostics; primary-fit coefficients score both sealed primary and repeat held-out rows. The selected family is supported only if, independently in both runs, held-out worst regret is at most `1.10`, upper-median regret at most `1.02`, plateau misses are at most three, worst regret is strictly below unchanged production's, and upper-median regret is no greater than production's. Primary/repeat use eight untimed warm-ups, 30 rotating timed rounds, batch 64, the commit-to-completed difference quotient, and the predecessor's twice-combined-standard-error noise rule.

The report-only anchors predeclare comparisons `780: 195 -> 260`, `780: 260 -> 390`, and `1,042: 2 -> 521` at every row, plus raw minima and plateaus. They test recurrence of the prior dense below-cap plateau and sparse cutoff reversal without validating a family.

The retained TSV schema names the unchanged selector column `production`, records the exact 625-row population and actual prepared resource bounds, and records before/after executable digest, load, and build-process occupancy. A two-column environment record equality-pins every claim-bearing host/tool/date/load/occupancy/matrix/custody field and SHA-256-pins the measuring source, ABI/source authorities, analyzer wrapper/shared source, manifests, and both TSVs. The timed binary is not retained: its digest, size, and mtime identify the observed unretained executable, while retained source and lock digests identify build inputs and do not claim a byte-identical rebuild.

## Outcome

**Measurement, 2026-08-11 — complete finite population.** The retained [`sweep.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/sweep.tsv) and [`repeat.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/repeat.tsv) each measured all 625 predeclared arithmetic widths across 70 cells and declined zero. The widest prepared workgroup was 641 and the maximum prepared allocation was 2,576 bytes, matching the 2,564-byte source request after Metal's observed 16-byte rounding. The direct non-timed verifier had independently reached 625/625 with no decline before timing.

**Measurement — exact host and custody boundary.** Both runs used Apple M4 Max reporting Apple9, macOS 27.0 build `26A5388g`, Xcode 26.6 build `17F113`, SDK 26.5 build `25F70`, `Apple metal version 32023.883`, `AIR-LLD 32023.883`, and `nightly-2026-07-19` / rustc `eff8269f7`. Primary ran `2026-08-11T10:02:50Z`–`10:06:56Z` with load `4.33 3.79 3.75`–`3.35 3.64 3.70`; repeat ran strictly afterward `10:07:04Z`–`10:11:13Z` with load `3.14 3.58 3.68`–`2.60 3.21 3.50`. The broader process check found zero Cargo, rustc, rustdoc, nextest, clippy-driver, or make processes before and after each run. Both TSVs report zero harness build-process occupancy and the same before/after executable SHA-256 `f9bac263c0a140f5843667550a0b5373cc7a21c784dbefb79f837bf4fc6c7b29`. The binary is unretained; [`environment.tsv`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/environment.tsv) records its observed 7,823,936-byte size and correct UTC mtime `2026-08-11T10:00:57Z` without claiming a byte-identical rebuild.

**Measurement — the predeclared interaction is rejected.** Reproducible [`analysis.txt`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-tree-width-interactions/analysis.txt) passes exact population, schema, source/result digest, environment, and custody validation. D is componentwise no worse than C and strictly better in only 2/6 fit folds, below the frozen three-fold eligibility threshold; `rows-plus-contributors` therefore remains the primary fit selection. Its sealed held-out worst regret is `1.484044` / `1.477213`, upper median `1.000000` / `1.000000`, and five plateau misses in each run. It fails the `1.10` worst-regret and at-most-three-miss clauses independently in both runs. D's diagnostic held-out rows are better (`1.084829`, two misses primary; `1.105080`, two misses repeat) but do not alter the fit-only rejection and the repeat also exceeds the frozen `1.10` bar.

**Measurement — the motivating discontinuities recur but are not one compact rule.** For 780 contributors, every fresh row/run again puts the raw-minimum plateau below production 260, and the predeclared 195→260 comparison prefers 260 while 260→390 prefers 260 in all ten cells. For 1,042 contributors, 521 is faster in seven cells and within noise in three. These report-only anchors did not select the family.

**Fact — subject controls failed with assertions unchanged and were restored.** Before timing, changing contributor 1,282 to 1,284 failed `the frozen interaction contributor population moved`; changing the tree's coherent ownership witness from 4 to 40 failed `8x780: the rebuilt single-workgroup tree at the production participant count (260) does not emit the source the compiler emits`; shifting emitted buffer-table lookup from `slot` to `slot + 1` produced zero admissions and failed `the frozen interaction admitted/verified census moved` with left 0 / right 625; replacing unit inputs with two failed `output[0] is 1560 (44c30000), expected 780 (44430000)`; and redirecting the tree's owning output binding failed `output[0] is 0 (00000000), expected 780 (44430000)`.

After retention, changing one p50 digit failed `<sweep.tsv> digest moved`, showing the retained result is bound before scoring. Updating that copied digest but changing one status to `declined` then failed `<sweep.tsv>: every frozen width must measure`. Independently changing copied environment subjects failed `environment key environment.toolchain moved`, `environment key environment.date_utc.primary_start moved`, `environment key host.load_before.primary moved`, and `environment key host.occupancy moved`. Finally, changing the copied TSV's starting executable digest and updating only its result digest failed `primary: starting executable digest does not match retained custody`. Every probe used a disposable copy where needed; the retained files and sources were restored before the positive replay.

**Conclusion.** This qualified Apple9 population rejects the tested row-regime × neighbour-gap interaction vocabulary; it does not authorize retuning `capped_tree_partition`, adding a public request/profile field, or changing identity, feasibility, or the accepted cost row. A second target profile is premature while no compact family survives this host. [`compare-a-target-private-tree-width-table-with-a-prime-factor-signature`](compare-a-target-private-tree-width-table-with-a-prime-factor-signature.md) owns the next discriminating evidence: a capacity/interpolation/default-frozen target-private table versus an explicitly encoded pre-lowering prime-factor signature on another fresh contributor-grouped validation matrix. Neither opened 2026-08-11 matrix may validate it. No architecture remainder is filed because the rejected family needs no new authority or threading surface. The ticket remains `in-progress` for independent review and coordinator closure.

## Non-goals

Do not change `capped_tree_partition`, reuse the opened 2026-08-11 held-out cells as validation, add a public profile field, or move target admission into cost.

## Closes when

A fresh sealed population either supports one explicitly encoded interaction family under primary and repeat scoring or rejects the family and states whether the next useful evidence is a second profile, a different observable, or a non-parametric target-private table.

## Integration

- 2026-08-11 — Candidate `15f5f61fe0efb2a22452943510aa594602d6d887` received an independent exact-hash review with no findings and was integrated into `main` by merge commit `a0a4fe96777c8e8ddf1290eac2832241b4192024`. The review independently reproduced all 1,250 measured rows, six C/D fold comparisons, the 56-key environment record, and every reported objective. This spike/docs/tickets-only delta touches none of the repository full-gate invalidation paths and carries exact-main full gate `477634a1d251afd8cdb116947df3c33fd8b7e7ae`; citations and ticket lint are rerun on the closing commit.
