---
id: calibrate-a-shape-aware-tree-width-cost-row
title: Calibrate a shape-aware tree-width cost row
status: todo
priority: p3
dependencies: []
related: [measure-the-tree-width-excursion-past-the-cap, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
---
## The question

What target cost row should choose the single-workgroup reduction tree's participant count from the legal exact divisors of a contributor count, when neither a constant cap nor nearest-to-cap distance explains the measured optimum?

This is a wider selection study, not authority to change `capped_tree_partition`. It exists because [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) measured two different failures of the current nearest-256 rule on the qualified Apple9 row:

- **Measurement — dense lattice:** at 780 contributors, production selects 260, while 39 is the best retained width at both 4 and 16,384 rows. Production costs 1.841x and 1.035x the best respectively. The curve improves from 195 to 260 and then reverses sharply by 390, so neither "never cross the cap" nor "take the nearest exact divisor" describes it.
- **Measurement — sparse cutoff:** at 1,042 contributors the only legal widths are 2 and 521. Production selects 2, but 521 is faster at both row counts and in both retained runs: 1.277x production regret at 4 rows and 1.012x at 16,384 in the primary run. The eight predeclared boundary verdicts across 514, 780, and 1,042 contributors repeat eight for eight.

Those are six host-specific cells, not a shape distribution. They refute a universal reading of the current heuristic but are too small a population to replace it.

## Experiment boundary

Design a finite matrix before timing that varies row count across and around the retained reduction-strategy saturation contour, and varies contributor counts across multiple divisor-lattice shapes:

- sparse lattices with only two legal widths on both sides of the current cutoff;
- dense lattices whose best width could lie below, near, or above 256;
- counts where the current rule chooses below and above the cap; and
- neighbouring counts whose divisor sets change discontinuously, so a smooth formula cannot look supported merely because the population was smooth.

Reuse the retained partition-calibration region construction, compiler source/ABI anchor, oracle, preparation, 64-encode difference quotient, interleaving, repetition count, and qualified-host controls. Assert the exact shape and participant populations, retain a same-matrix repeat, and perturb each validator subject without changing its assertion. Analyze candidate rows with held-out shapes; report regret against every measured legal width and not only against the current production choice.

The study should compare at least these candidate information sets separately: contributor count alone; row count plus contributor count; divisor-lattice summaries available before lowering; and the target saturation quantity already used by the reduction strategy cost row. A candidate that needs a new public request field, changes artifact identity, or moves a feasibility decision into cost stops for its own architecture ticket.

## Closes when

A retained qualified-host record either supports one explicit shape-aware participant-selection row under held-out scoring, or shows that this finite population still cannot support one and names the next discriminating unknown. The outcome must state whether the dense-lattice below-cap optimum and sparse-cutoff reversal recur, the worst and median regret of every candidate, repeat variance, exact environment, and unsupported profiles. No production selection changes in this ticket.
