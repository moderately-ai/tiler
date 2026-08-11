---
id: enumerate-tree-width-alternatives-before-target-cost-ranking
title: Enumerate tree-width alternatives before target cost ranking
status: deferred
priority: p2
dependencies: [gate-the-workgroup-tree-on-an-explicit-qualified-width-policy]
related: [carry-the-tree-participant-cap-as-a-target-profile-row, calibrate-a-shape-aware-tree-width-cost-row, test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix, compare-a-target-private-tree-width-table-with-a-prime-factor-signature]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, optimizer, deferred]
---
## Question

How can Tiler offer more than one legal single-workgroup-tree width, filter those alternatives through the correct feasibility phase, and rank only the survivors without a cost preference creating or withdrawing legality?

## Why this is separate

**Fact — a numeric cap is not merely a cost coefficient.** `capped_tree_partition` emits one width. Changing that width can make `workgroup_tree_tile` or prepared-kernel workgroup limits refuse the strategy even where another width would be valid.

**Fact — the decisive target fact arrives late.** The authoritative Apple9 workgroup width is a `PreparedKernelPreflight` query, not a compile-profile quantitative fact. Compile-time clamping cannot reproduce that authority honestly.

**Measurement — compact replacement policies are unsupported.** Three fresh held-out studies rejected shape-aware regression, row-regime interactions, and target-private table/prime-signature candidates. Their opened cells may motivate this architecture but may not validate a new ranking rule.

**Inference — legality must precede preference.** A future target cost row may rank already admitted candidates. It must not select the only candidate before the target can decide whether it is legal.

## Required research and design

1. Define a bounded, target-independent candidate population from contributor arithmetic and hard IR limits. State what legal widths are intentionally outside it; no cost input may alter this population.
2. Measure Tiler planning overhead—the number of candidate regions, verification work, allocations, and identity material—not kernel runtime alone. Compare exhaustive widths, a proven finite basis, and delayed/lazy representations.
3. Decide how prepared-kernel facts filter candidates before final selection without retrying a failed chosen plan. Automatic clamp, lower-width retry, backend switch, and balanced substitution are forbidden.
4. Require the target/profile author to state the approach explicitly. Any future vocabulary must distinguish an explicit `NoPreference` from a measured ranking policy; omission is not a default.
5. Preserve typed explanation for every unavailable candidate and for the final choice. Separate arithmetic unrepresentability, compile-profile feasibility, prepared-entry feasibility, and cost ranking.
6. Derive schedule, kernel, request, profile, artifact, and cache identity consequences for a population of alternatives and for any delayed choice.
7. Use a new unopened measurement matrix for any ranking claim. The three 2026-08-11 matrices are evidence about failed candidates, not validation data.

## Stop conditions

Stop for a public-boundary decision if the design needs a new request-level user policy, runtime retry, or set-valued execution environment. Stop for identity review if two different candidate populations or late choices could encode the same request/artifact subject. Stop if planning overhead cannot be bounded independently of tensor extent.

## Non-goals

No immediate production change, no retuning `256`, no second-profile performance claim, no silent fallback, and no use of a cost row as a feasibility predicate.

## Trigger check log

- **2026-08-11 — not fired.** Only the evidence-qualified fixed policy is currently needed. Fire when a second target requires the tree strategy, the fixed policy produces an evidenced avoidable refusal, or a new unopened study is authorized to evaluate a general ranking mechanism. Reproduce current state with `rg -n 'MEASURED_TREE_PARTICIPANT_CAP|fn capped_tree_partition' crates/tiler-compiler/src/physical.rs` and `tkt show gate-the-workgroup-tree-on-an-explicit-qualified-width-policy --format json`.
