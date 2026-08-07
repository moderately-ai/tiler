---
id: bound-the-tree-cap-s-unmeasured-downward-direction
title: Bound the tree cap's unmeasured downward direction
status: todo
priority: p2
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256, activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
---
## The defect in the rule, not in the constant

**Fact — the calibration measured only power-of-two contributor counts, and on those the cap's rule is degenerate.** The retained spike's seven shapes have contributor counts 16, 32, 2,048, 4,096, 8,192, 8,192, 16,384; [its README](../spikes/program-planning/reduction-partition-calibration/README.md) says so itself — "no shape here has a contributor count that is not a power of two". On a power of two, "the largest admissible participant count not exceeding 256" is identically `min(256, contributors / 2)`, which is the **widest** count the cap admits. Every measured cell therefore exercised the rule only where the divisor lattice is dense.

**Fact — where the lattice is sparse the same rule selects near-minimum widths, and this was verified rather than argued.** Re-implementing both rules and exhausting the range gives:

| contributors | `capped_tree_partition` | `governed_partition` |
| --- | --- | --- |
| 514 (`2 × 257`) | **2** partitions of 257 | 257 partitions of 2 |
| 8,192 | 256 of 32 | 128 of 64 |

Below 20,000 there are **8,452** counts where the capped rule chooses a *narrower* tree than the balanced rule it replaced, and **1,176** where it chooses exactly **2** participants. At 514 both alternatives are feasible on the qualified Apple9 profile (257 threads is within the entry's 1,024; 1,028 staged bytes within 32,768), so this is a real change of emitted width, not a decline.

**Inference — the retained calibration is direct evidence against that direction at the closest shape it measured.** At 4 rows × 8,192 the spike timed the tree at **9.53 µs with 256 participants against 48.15 µs with two**, a 5.05× span and its measured worst admissible value. The rule as written selects the analogue of that worst value at 514, 526, 538, and 8,449 further counts below 20,000.

**Fact — nothing in the code states the downward direction.** `capped_tree_partition`'s doc calls the rule "the calibrated one" and its only monotonicity paragraph bounds the width from *above*. A reader would not expect a 2-wide tree at 514 contributors.

**No live wrong answer.** Selection prunes the tree before assembly on every shape this profile admits, so no emitted plan carries any of these widths today. This becomes observable under [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md), which is why it is filed now rather than after.

## The candidate answer, and why it is not being applied unilaterally

**Take the wider of the two rules: `max(capped_tree_partition, governed_partition)` by participant count.** Verified: this is **identical to the cap on all seven measured shapes**, so it preserves the leave-one-out selection and its 1.008 held-out regret exactly, while never choosing a narrower tree than the balanced rule the calibration compared against. At 514 it gives 257; at 8,192 it gives 256; at 12 it gives 6.

That is a change to a rule the calibration selected and a ticket closed on, so it needs its own evidence rather than a coordinator's edit: the claim "identical on every measured shape" must be pinned as a test over the spike's own seven counts, and the docstring's "calibrated" must be re-scoped to what the seven shapes support.

## What this ticket owes

- The direction stated wherever the rule is described, with the verified numbers, so no reader takes "calibrated" to cover the whole domain.
- The rule refined or the current one defended on evidence — if refined, a test pinning agreement with the cap on the calibration's seven counts, and the divergent counts named.
- Whatever lands, the claim's extent matched to its evidence: seven power-of-two shapes, one profile, one contract, one family, `f32`.

## Explicit non-goals

No change to `MEASURED_TREE_PARTICIPANT_CAP`'s value, which the leave-one-out selection supports. No selection change. No new measurement is *required* — the candidate above is decidable from the retained data — though a sweep over non-power-of-two counts would settle it more strongly than argument.

## Graph maintenance

Filed 2026-08-07 by the coordinator from the post-landing multi-lens audit of [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md); every number above was re-verified by exhausting the range rather than taken from the audit report.
