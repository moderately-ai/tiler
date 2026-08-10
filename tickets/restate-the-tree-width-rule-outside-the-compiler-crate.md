---
id: restate-the-tree-width-rule-outside-the-compiler-crate
title: Restate the tree width rule outside the compiler crate
status: done
priority: p2
dependencies: []
related: [bound-the-tree-cap-s-unmeasured-downward-direction, measure-the-tree-width-excursion-past-the-cap, correct-the-two-participant-residue-s-smallest-count, correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof]
scopes: [contracts/optimizer, contracts/numerics, implementation/conformance, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, documentation]
---
## Fact audit at `97282def`, before any edit

Every Fact below was re-read in full at this base rather than taken from the ticket text.

| Fact | Verdict | Evidence |
| --- | --- | --- |
| Two documents state the superseded rule verbatim | **verified** | Both anchors present word for word: `` the largest admissible participant count not exceeding 256 `` in `docs/compiler/fusion-and-scheduling.md`'s 2026-08-07 partition-calibration paragraph, and `` the largest admissible participant count not exceeding a measured 256 `` in `docs/correctness-and-testing.md`'s measured-boundary paragraph. |
| `3,530 - 2,561 = 969`, not 964 | **verified** | The sentence read `` 2,561 of the 3,530 admitting counts below 4,096 differ while the remaining 964 still agree ``. Independent enumeration of the *superseded* rule gives 3,530 admitting and 2,561 differing, so 969 agree. The 964 was an arithmetic slip and nothing else. |
| Under the landed rule the counts are 2,350 differing and 1,180 agreeing | **verified** | `cargo nextest run -p tiler-compiler -E 'test(/the_tree_(takes_the_capped\|widens_toward)/)'` passes at this base, asserting `admitted == 3_530` and `differing == 2_350`. 1,180 is the complement and is asserted nowhere; it was derived, not read. A transcription-independent Python enumeration of both rules reproduces 3,530 / 2,350 / 1,180 and, for the old rule, 3,530 / 2,561 / 969. |
| The neighbouring claims survive | **verified** | First differing count is 12 (tree `6 x 2`, split `4 x 3`); 4, 6, 8, 9, 10 agree and 5, 7, 11 are declined by both; at 8,192 the tree takes 256 of 32 against the split's 128 of 64; at four contributors both return `2 x 2`. |
| Three doc comments outside the compiler crate are "now incomplete rather than false" | **imprecise** | True of the two `tiler-conformance` anchors, which were repaired. The third, `crates/tiler-build/src/metal_plan.rs`'s `` `governed_partition` for the split and `capped_tree_partition` for the `` , is a *domain* claim that is simply still true — the ticket says so two lines later, so the heading over-counts its own list. It was verified and left alone; `implementation/build` went unused. |
| `prototypes/serial-sum-run/src/proof.rs` needs nothing | **imprecise** | Its four-contributor claims are all true. But `proof.rs`'s `declared_partition` doc says the two rules "diverge from twelve contributors upward", which reads as *every* count from twelve on and is false — 1,180 of the 3,530 admitting counts below 4,096 still agree, up from 969 under the old rule, so the landing made this phrasing worse rather than leaving it alone. It is `implementation/runtime` and was not edited here. |

**One site the ticket did not list, inside a scope it did hold.** `crates/tiler-conformance/src/serial_sum.rs`'s `declared_partition` doc carried the same "diverging from `SEPARATING_COLUMNS` upward" phrasing, for the same reason, and was tightened to "first diverge at" with the agreeing population named.

**One defect found in a scope this ticket does not hold.** `crates/tiler-compiler/src/physical.rs`'s `capped_tree_partition` doc says "Below 20,000 contributors, 1,133 counts still take two participants … The smallest is 1,042". The smallest of those 1,133 is **four**, where two is the only admissible count; 1,042 is the smallest at which the rule *declines* an admissible wider count. Filed as [`correct-the-two-participant-residue-s-smallest-count`](correct-the-two-participant-residue-s-smallest-count.md). The documents restated here state it the correct way.

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

## What landed

Five sites edited across four files, from base `97282def`.

- `docs/compiler/fusion-and-scheduling.md` — the rule restated as "the admissible participant count nearest 256, ties going to the narrower"; the population sentence corrected to 2,350 / 1,180 with the struck 2,561 / 964 pair retained under a dated correction naming both its defects; and three new paragraphs stating the two-branch omission, the four evidence rungs separately (empirical upward, arithmetic `s <= 509`, exhaustive finite over `0..4_096`, `Unknown` on cost), and the two-participant residue.
- `docs/correctness-and-testing.md` — the rule restated in the measured-boundary paragraph, plus two paragraphs: that the width rule moves the choice and never the domain, so no shape in that section loses an alternative; and that nothing in the section is evidence for the *cost* direction, because the cap is measured only at power-of-two counts where the two formulations coincide.
- `crates/tiler-conformance/src/serial_sum.rs` — `SEPARATING_COLUMNS`'s derivation completed (the upward search is empty below 514 contributors, so twelve is decided by the downward walk alone and the minimality argument survives untouched); `declared_partition`'s "diverging … upward" tightened to "first diverge at", with the 1,180 agreeing counts named.
- `crates/tiler-conformance/src/serial_sum/tests.rs` — `separating_tree_partition`'s derivation completed the same way.
- `crates/tiler-build/src/metal_plan.rs` — **read and deliberately unedited**; its domain claim verifies. `implementation/build` was claimed and not used.

**How the figures were derived.** Not transcribed. Both rules were re-implemented from `crates/tiler-compiler/src/physical.rs` at this base in a throwaway enumeration and run over `0..4_096` and `0..200_000`, then cross-checked against the two pinning tests. Independent agreement on all of: 3,530 admitting; 2,350 differing and 1,180 agreeing under the landed rule; 2,561 and 969 under the superseded one; 1,061 widened past the cap; widest width exactly 509; identical decline sets over `0..200_000`; 1,133 two-participant counts below 20,000 against 1,176; and, below 20,000, 1,065 counts where the wider-of-the-two candidate exceeds `MAX_COOPERATIVE_PARTICIPANTS` and this rule does not.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The closed Outcome left two live imprecise "diverge from twelve contributors upward" phrases unsplit. Both overclaim universal divergence past twelve; 1,180 of the 3,530 admitting counts below 4,096 still agree under the landed rule. Sites:

1. `crates/tiler-conformance/src/serial_sum/tests.rs` — four-contributor portfolio comment, anchor `` they diverge from twelve contributors upward `` (held `implementation/conformance` at close; same defect class repaired in `serial_sum.rs` `declared_partition` but missed here).
2. `prototypes/serial-sum-run/src/proof.rs` — `declared_partition` doc, anchor `` the two agree at four and diverge from twelve contributors upward `` (unheld `implementation/runtime`; already marked **imprecise** in the pre-edit Fact table above but never filed).

Prefer "first diverge at twelve" and/or name residual agreement, matching the repaired `serial_sum.rs` language. Filed as [`correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof`](correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof.md). The two-participant residue defect in `physical.rs` remains owned by [`correct-the-two-participant-residue-s-smallest-count`](correct-the-two-participant-residue-s-smallest-count.md) (now also listed under `related`).
