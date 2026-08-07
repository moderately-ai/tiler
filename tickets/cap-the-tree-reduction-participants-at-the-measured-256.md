---
id: cap-the-tree-reduction-participants-at-the-measured-256
title: Cap the tree reduction participants at the measured 256
status: in-progress
priority: p2
dependencies: []
related: [calibrate-the-reduction-partition-against-measured-alternatives, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, reductions, measurement]
claimed_from: todo
assignee: agent-tree-cap
lease_expires_at: 1786088941
---
## User-visible outcome

The single-workgroup tree's participant count follows the measured rule — the largest admissible partition not exceeding 256 — instead of the balanced exact split, so the tree stops paying up to 1.216× on shapes the retained calibration measured.

## The evidence

**Measurement, 2026-08-07** — [`spikes/program-planning/reduction-partition-calibration`](../spikes/program-planning/reduction-partition-calibration/README.md), on the qualified Apple9 host matching the ledger in every field. The tree's governed (balanced) partition is beaten on 4 of 7 separated shapes, worst 1.216×; a cap at **256** is selected by leave-one-out on all seven folds with held-out worst regret **1.008** and median 1.000, agreeing across both 64-encode runs. One shape's plateau is `{256}` alone.

**The split keeps `governed_partition` unchanged.** The same sweep refuted the balanced choice for the split too, but no constant replaces it — its optimum moves from 256 at four rows to 2 at ≥4,096 rows with the same saturation quantity the strategy contour turns on, so its calibration is not separable from [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md). This ticket must not touch the split's partition.

## The work

Give the tree its own participant rule in `crates/tiler-compiler/src/physical.rs` — the largest admissible partition (exact split, ≥2 partitions of ≥2 contributors each) not exceeding 256 — leaving `governed_partition` as the split's choice. The 256 is a measured property of one host row; carry it with a comment citing the retained spike and its bound (one profile, one family, seven shapes), not as a portable constant.

**Identity discipline.** Changing the tree's participant count changes every emitted tree plan: kernel-program identities, schedule identities, and any pinned digest or golden carrying a tree alternative move. Enumerate what moves before editing (survey the pinned population as the concatenate landing did), execute the step completely with each pin recomputed on your tree and ledgered, or — if the ripple reaches identity domains the evidence does not support moving — stop and park with the enumeration.

**Supersede the two stale doc paragraphs at the sites this edits** (owed by the calibration's Outcome): `governed_partition`'s and `single_workgroup_tree_region`'s docs both still say the calibration ticket owns replacing the choice and that the split was never measured — both are superseded by the delivered measurement; rewrite to current truth citing the spike.

Watched-failing: a test pinning the tree's participant count at a shape where the cap and the balanced choice differ (e.g. 8,192 contributors: balanced 128 vs cap 256), observed failing under the balanced rule before the change.

## Closes when

The tree reads the capped rule, the split is untouched, every moved pin is enumerated and recomputed with ledgers, the two doc paragraphs state the current truth, and `make full` is green.
