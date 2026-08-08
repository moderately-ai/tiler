---
id: restate-the-tree-width-rule-outside-the-compiler-crate
title: Restate the tree width rule outside the compiler crate
status: todo
priority: p2
dependencies: []
related: [bound-the-tree-cap-s-unmeasured-downward-direction, measure-the-tree-width-excursion-past-the-cap]
scopes: [contracts/optimizer, contracts/numerics, implementation/conformance, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, documentation]
---
## What is stale, and why it could not be fixed in place

[`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md) changed `capped_tree_partition` from "the largest admissible participant count not exceeding 256" to "the admissible participant count nearest 256". It held `implementation/compiler` only, so every site outside `crates/tiler-compiler/**` that restates the rule or quotes a population derived from it is untouched and is now wrong or imprecise.

**Fact — two documents state the superseded rule verbatim.**

- `docs/compiler/fusion-and-scheduling.md`, in the 2026-08-07 partition-calibration paragraph: anchor `` `single_workgroup_tree_region` reads `capped_tree_partition` — the largest admissible participant count not exceeding 256 ``.
- `docs/correctness-and-testing.md`, in the measured-boundary paragraph: anchor `` the tree now reads `capped_tree_partition` — the largest admissible participant count not exceeding a measured 256 ``.

**Fact — one population in those documents moved, and one was already wrong before it moved.** The same `fusion-and-scheduling.md` sentence reads "2,561 of the 3,530 admitting counts below 4,096 differ while the remaining 964 still agree". Two defects, independently:

1. `3,530 - 2,561 = 969`, not 964. That arithmetic was wrong at commit `6eabf97e`, before any rule change.
2. Under the landed rule the counts are **2,350 differing and 1,180 agreeing** out of the same 3,530 admitting counts. `tiler_compiler::pipeline::tests::the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs` asserts 3,530 and 2,350 and is the recomputation authority.

**Fact — the neighbouring claims in those documents survive and must not be "corrected".** "Twelve is the smallest count at which they differ" is still true: the tree takes 6 partitions of 2 at twelve contributors under both the old rule and the new one, and the split takes 4 of 3. So is "at 8,192 the tree takes 256 of 32 against the split's 128 of 64", and so is the four-contributor agreement the numerical record rests on.

**Fact — three doc comments outside the compiler crate describe the rule's mechanism and are now incomplete rather than false.**

- `crates/tiler-conformance/src/serial_sum.rs`, anchor `` **Twelve, and it is the smallest such count.** `capped_tree_partition` walks `` — it describes only the downward walk to a divisor at or below the cap. Its conclusion at twelve is correct; its description of how the rule reaches it no longer covers the rule.
- `crates/tiler-conformance/src/serial_sum/tests.rs`, anchor `Six participants folding two contributors each`.
- `crates/tiler-build/src/metal_plan.rs`, anchor `` `governed_partition` for the split and `capped_tree_partition` for the `` — a domain claim, verified unchanged, listed so a sweep confirms it rather than rewrites it.

`prototypes/serial-sum-run/src/proof.rs` was read and needs nothing: its three mentions are the four-contributor agreement and "the two agree at four and diverge from twelve", both still true.

## What this ticket owes

- The rule restated in both documents, with the *direction* it now bounds in each sense and the evidence rung each direction reaches. `crates/tiler-compiler/src/physical.rs`'s `capped_tree_partition` carries the language to align to; copy the separation, not just the sentence.
- The two populations recomputed from the test rather than from this ticket, and the 964 arithmetic corrected as a separate fact so its cause is visible.
- The conformance and build doc comments made complete, with their surviving conclusions left alone.
- No new claim about *cost* at a non-power-of-two contributor count. The downward direction is an inference from the calibration's 5.05x span, not a measurement, and [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) owns the measurement.

## Graph maintenance

Filed 2026-08-08 by the worker on [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md), which held neither `contracts/optimizer`, `contracts/numerics`, `implementation/conformance`, nor `implementation/build`. Every anchor above was read in full at commit `6eabf97e`; the recomputed populations come from the landed test, not from a summary.
