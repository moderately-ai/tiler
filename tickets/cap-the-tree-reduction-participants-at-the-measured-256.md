---
id: cap-the-tree-reduction-participants-at-the-measured-256
title: Cap the tree reduction participants at the measured 256
status: done
priority: p2
dependencies: []
related: [calibrate-the-reduction-partition-against-measured-alternatives, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, reductions, measurement]
---
## User-visible outcome

The single-workgroup tree's participant count follows the measured rule — the largest admissible partition not exceeding 256 — instead of the balanced exact split, so the tree stops paying up to 1.216× on shapes the retained calibration measured.

**Correction — 2026-08-10.** The sentence above is the landing formulation. Live rule since [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md): the admissible participant count nearest 256, ties going to the narrower. On the measured power-of-two shapes the two formulations agree, so the 1.008 held-out regret and cap selection are unchanged.

## The evidence

**Measurement, 2026-08-07** — [`spikes/program-planning/reduction-partition-calibration`](../spikes/program-planning/reduction-partition-calibration/README.md), on the qualified Apple9 host matching the ledger in every field. The tree's governed (balanced) partition is beaten on 4 of 7 separated shapes, worst 1.216×; a cap at **256** is selected by leave-one-out on all seven folds with held-out worst regret **1.008** and median 1.000, agreeing across both 64-encode runs. One shape's plateau is `{256}` alone.

**The split keeps `governed_partition` unchanged.** The same sweep refuted the balanced choice for the split too, but no constant replaces it — its optimum moves from 256 at four rows to 2 at ≥4,096 rows with the same saturation quantity the strategy contour turns on, so its calibration is not separable from [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md). This ticket must not touch the split's partition.

## The work

Give the tree its own participant rule in `crates/tiler-compiler/src/physical.rs` — the largest admissible partition (exact split, ≥2 partitions of ≥2 contributors each) not exceeding 256 — leaving `governed_partition` as the split's choice. The 256 is a measured property of one host row; carry it with a comment citing the retained spike and its bound (one profile, one family, seven shapes), not as a portable constant.

**Correction — 2026-08-10.** Landing coded and described the truncate-from-below form above. Live `capped_tree_partition` is the admissible count nearest `MEASURED_TREE_PARTICIPANT_CAP` (256), ties narrower — restated by [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md). Measured shapes unchanged.

**Identity discipline.** Changing the tree's participant count changes every emitted tree plan: kernel-program identities, schedule identities, and any pinned digest or golden carrying a tree alternative move. Enumerate what moves before editing (survey the pinned population as the concatenate landing did), execute the step completely with each pin recomputed on your tree and ledgered, or — if the ripple reaches identity domains the evidence does not support moving — stop and park with the enumeration.

**Supersede the two stale doc paragraphs at the sites this edits** (owed by the calibration's Outcome): `governed_partition`'s and `single_workgroup_tree_region`'s docs both still say the calibration ticket owns replacing the choice and that the split was never measured — both are superseded by the delivered measurement; rewrite to current truth citing the spike.

Watched-failing: a test pinning the tree's participant count at a shape where the cap and the balanced choice differ (e.g. 8,192 contributors: balanced 128 vs cap 256), observed failing under the balanced rule before the change.

## Closes when

The tree reads the capped rule, the split is untouched, every moved pin is enumerated and recomputed with ledgers, the two doc paragraphs state the current truth, and `make full` is green.

## Outcome — delivered 2026-08-07 at `9415b450`

**The tree reads the capped rule and the split is untouched.** `single_workgroup_tree_region` calls `capped_tree_partition(contributors)` in `crates/tiler-compiler/src/physical.rs`; `split_reduction_regions` still calls `governed_partition(contributors)` in the same file. Verified by the coordinator on the merged tree, not relayed. Branch commits `39702d21` (the work) and `7d5b3f62` (branch coverage added at review), merged as `9415b450`.

**The cap is the retained calibration's own selection**, independently re-read from [`spikes/program-planning/reduction-partition-calibration`](../spikes/program-planning/reduction-partition-calibration/README.md) rather than from this ticket's prose: leave-one-out selects cap 256 on all seven folds, held-out worst regret **1.008** against the balanced choice's **1.216**, median 1.000, with the repeat 64-encode run at 1.012 and 1.211.

**The decline set does not move, and that is the load-bearing property.** `capped_tree_partition` returns `Some` on exactly the counts `governed_partition` does — at least four and composite — so `WorkgroupTreeUnavailable::NoAdmissibleParticipantCount` fires on exactly the same extents and the cap chooses *within* the strategy's domain rather than narrowing it. Pinned over `0..4_096` (3,530 admitting counts, 2,561 at which the two rules choose differently) and, at review, at `257 * 257` — the smallest count reaching the above-cap branch, which the ladder cannot reach and which was otherwise untested. That branch was watched failing under a perturbation that disabled its search.

**Correction — 2026-08-10.** Domain agreement (3,530 admitting below 4,096) still holds. The **2,561** differing figure is the superseded truncate-from-below rule's population; under the live nearest/ties-narrower rule the two choose differently at **2,350** of those admitting counts (pipeline test `the_tree_takes_the_capped_participant_count_where_the_balanced_split_differs`; same correction in `docs/compiler/fusion-and-scheduling.md` dated 2026-08-08).

**No identity pin moved, and no pin was recomputed.** Every site that could carry a tree plan was enumerated and read: the standard Metal identity fixture is `[2, 2]` under a flush-only contract and composes no tree at all; the three-strategy shapes assert structural observables that survive the width change; the remaining tree-reaching counts are 4, 6, 8, and 10, where the two rules agree.

> **Correction, 2026-08-07, from the post-landing audit — that enumeration was not exhaustive, though its conclusion holds.** `crates/tiler-build/src/metal_plan.rs` also compiles reduction programs at candidate shapes `(4, 16)` and `(64, 64)` under `FLUSH_AND_REASSOCIATE_F32`, and contributor counts **16** and **64** are both divergent (16: capped 8 partitions against balanced 4; 64: capped 32 against balanced 8). They were omitted above. The no-pin-moved conclusion survives because that test observes only that the width exceeds one, never a particular width, and because the workspace suite is green with no pin recomputed on the merged tree. What was not established by reading alone is whether those two shapes *retain* a tree in the portfolio at all; settling it means printing the widths from `portfolio_shape` for each candidate. Recorded rather than quietly amended, because an enumeration that claimed to be complete and was not is the kind of thing a later reader would rely on. `cargo nextest run --workspace` reports 2,936 passing on the merged tree.

**A consequence the ticket did not name, recorded rather than smoothed over.** The tree stages one `f32` slot per participant, so the wider capped width raises the `local-memory-bytes` requirement — 1,024 bytes at 8,192 contributors where the balanced 128 needed 512. A profile whose row falls between the two now refuses a tree it would previously have admitted. That refusal was left to the feasibility authority rather than narrowed away in the partition rule, which is the correct separation: a cost preference must not decide legality. The qualified profile declares `local_memory_bytes: 32_768` in `crates/tiler-build/src/metal_declaration.rs` and no profile in the repository sits in the affected band, so the band is argued from the feasibility path and not observed — filed as [`pin-the-local-memory-refusal-band-the-tree-cap-opened`](pin-the-local-memory-refusal-band-the-tree-cap-opened.md).

**Three stale doc claims superseded**, two owed by the calibration and one found during the work: `governed_partition`'s and `single_workgroup_tree_region`'s doc paragraphs; `docs/correctness-and-testing.md`'s assertion that both strategies take their partition from one function and so declare identical groupings at every count; `docs/compiler/fusion-and-scheduling.md`'s two paragraphs; and `target.rs`'s `three_strategy_domain`, which asked one rule as a predicate for both strategies and now asks each.

**Selection is unchanged.** The tree is still pruned before assembly on every shape this profile admits, so no emitted plan moves today. This calibrates the tree's *width*, not its odds of being chosen — that is [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md).

**Graph effect.** Firing the divergence closed the blocker on [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md), moved `deferred` → `todo` with its trigger-check entry recorded there. What fired is the count, not the case: no hardware run separates the two groupings.

**Correction — 2026-08-10.** That `deferred` → `todo` move is historical fire-record board state. [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) later closed (`status: done`).

**Boundary.** 256 is a property of one host row — one profile, one contract, one program family, `f32`, powers of two only. A second target profile carries its own row rather than inheriting this one.
