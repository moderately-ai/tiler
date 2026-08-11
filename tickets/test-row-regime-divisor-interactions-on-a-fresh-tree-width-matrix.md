---
id: test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix
title: Test row-regime and divisor-neighbour interactions on a fresh tree-width matrix
status: in-progress
priority: p3
dependencies: [calibrate-a-shape-aware-tree-width-cost-row]
related: [measure-the-tree-width-excursion-past-the-cap]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
claimed_from: todo
assignee: sol-tree-width-interactions
lease_expires_at: 1786447587
---
## The question

Can pre-lowering interactions between row regime and divisor-neighbour structure predict a single-workgroup reduction tree width on a newly frozen contributor-grouped population, or is this target row too discontinuous for the tested compact model class?

## Why this is separate

[`calibrate-a-shape-aware-tree-width-cost-row`](calibrate-a-shape-aware-tree-width-cost-row.md) opened its sealed cells on 2026-08-11. Its retained [`analysis.txt`](../spikes/program-planning/reduction-partition-calibration/results/2026-08-11-apple-m4-max-macos27.0-26A5388g-shape-aware-tree-width/analysis.txt) selects contributor-only on fit but records held-out worst regret `1.404845` / `1.416495` and seven / eight plateau misses, failing the frozen support bar in both runs. Rows-plus-contributors and the additive divisor-lattice family also fail. Those cells may motivate a hypothesis but cannot validate a post-hoc interaction family.

The same record shows why the unknown remains discriminating: at 780 contributors every row/run has a below-cap optimum plateau excluding production 260, while sparse 1,042 prefers 521 in six cells and is unresolved in two. Neighbouring contributor groups nevertheless move the raw optimum enough that all frozen compact candidates miss the held-out bar.

## Experiment boundary

Before any new device timing, predeclare:

- a fresh contributor-grouped fit/held-out split disjoint from the 2026-08-11 fit and held-out contributors;
- exact row regimes, arithmetic width populations, target-admission handling, primary/repeat policy, and noise/support bars;
- a minimal finite interaction vocabulary derived only from information available before lowering, such as row-regime × neighbour-gap or row-regime × width-rank terms; and
- a complexity penalty or nested comparison that can reject the richer family rather than selecting it merely because it has more terms.

Reuse the retained source/ABI/oracle/custody controls. Treat the Apple M4 Max/Apple9 result as host-specific and require a second qualified profile before any portable row. If a useful policy needs a public request field, artifact identity change, a feasibility move, device counters requiring a new unsafe site, or new construction-time authority, stop and file that architecture remainder separately.

## Non-goals

Do not change `capped_tree_partition`, reuse the opened 2026-08-11 held-out cells as validation, add a public profile field, or move target admission into cost.

## Closes when

A fresh sealed population either supports one explicitly encoded interaction family under primary and repeat scoring or rejects the family and states whether the next useful evidence is a second profile, a different observable, or a non-parametric target-private table.
