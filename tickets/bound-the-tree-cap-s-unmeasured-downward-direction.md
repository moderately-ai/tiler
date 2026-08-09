---
id: bound-the-tree-cap-s-unmeasured-downward-direction
title: Bound the tree cap's unmeasured downward direction
status: done
priority: p2
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256, activate-measured-reduction-selection-from-a-target-cost-row, correct-the-two-participant-residue-s-smallest-count]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
---
## Per-Fact audit — 2026-08-08, at base `6eabf97e`

Every Fact below was re-read at the dispatched base before any edit. Verdicts, in the order the claims appear.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The calibration measured only power-of-two contributor counts | **verified** | The spike README's closing bound, anchor "no shape here has a contributor count that is not a power of two". The seven contributor counts 16, 32, 2,048, 4,096, 8,192, 8,192, 16,384 match its shape table. |
| On a power of two the rule is `min(256, contributors / 2)` | **verified** | Read `capped_tree_partition` in full: `ceiling = MEASURED_TREE_PARTICIPANT_CAP.min(contributors / 2)`, and every value at or below a power of two's half divides it. |
| 514 gives the cap 2 partitions of 257 and the balanced rule 257 of 2; 8,192 gives 256 of 32 against 128 of 64 | **verified** | Both rules re-implemented and the range exhausted. |
| 8,452 counts below 20,000 where the cap chooses a narrower tree; 1,176 where it chooses exactly 2 | **verified** | Exhausted 0..20,000. |
| Both alternatives are feasible at 514 on the qualified Apple9 profile | **verified** | 257 threads inside the entry's 1,024; 257 `f32` slots is 1,028 bytes inside the declared 32,768. |
| The tree at four rows of 8,192 measured 9.53 µs at 256 participants against 48.15 µs at two, 5.05x | **verified** | The spike README's cost paragraph, anchor "Between the best and worst *admissible* partition of one shape the span reaches". |
| "The rule as written selects the analogue of that worst value at 514, 526, 538, and 8,449 further counts" | **imprecise, repaired below** | 514, 526 and 538 are the first three counts choosing *exactly two*, a population of 1,176; the 8,449 remainder belongs to the *narrower-than-balanced* population of 8,452. The sentence reads as one population and is two, differing by 7,276. |
| Nothing in the code states the downward direction | **verified** | At this base the rule sentence was "the largest admissible participant count not exceeding [`MEASURED_TREE_PARTICIPANT_CAP`]" and the only monotonicity paragraph was "**The chosen width exceeds the cap only where the balanced choice was already wider**", which bounds from above alone. |
| "This becomes observable under `activate-measured-reduction-selection-from-a-target-cost-row`, which is why it is filed now rather than after" | **stale** | That ticket is `status: done` with an Outcome dated 2026-08-07. The trigger has already fired; this was filed *after*, not before. The substance is unaffected — the tree is still pruned before assembly on every shape this profile admits — but the deferral it justified no longer exists. |
| The candidate `max(capped_tree_partition, governed_partition)` is identical to the cap on all seven measured shapes, and gives 257 at 514, 256 at 8,192, 6 at 12 | **verified** | Exhausted; all seven agree. |
| The candidate "never chooses a narrower tree than the balanced rule" — as a safe change | **false as a recommendation, and not applied** | It chooses widths `tiler_ir::schedule::workgroup_tree_tile` cannot represent. See the section below. |

### Repair of the imprecise sentence

Retired text: "The rule as written selects the analogue of that worst value at 514, 526, 538, and 8,449 further counts below 20,000."

Replacement: the rule as written chooses a **narrower** tree than the balanced rule at **8,452** counts below 20,000, and chooses the measured worst value of **two** participants at **1,176** of them, the smallest three being 514, 526 and 538.

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

## Outcome — 2026-08-08

### The proposed candidate was rejected on evidence, and a different rule landed

**`max(capped_tree_partition, governed_partition)` decides feasibility.** The balanced rule's participant count is `contributors / c` for `c` at most the integer square root, so it is unbounded above; taking the wider of the two therefore reaches widths `tiler_ir::schedule::workgroup_tree_tile` refuses. At 8,198 contributors (`2 * 4,099`) it chooses 4,099 against `MAX_COOPERATIVE_PARTICIPANTS` of 4,096, and `single_workgroup_tree_region` would report `WorkgroupTreeUnavailable::Unrepresentable` where it offers a two-participant tree today; at `2 * 65,537` it chooses 65,537. Exhausting 0..20,000 finds **1,065** counts where the candidate exceeds 4,096 and the current rule does not — the tree strategy would be *withdrawn* at every one of them. That is a cost preference narrowing the strategy's domain, which is precisely what `capped_tree_partition`'s own above-cap branch exists to avoid and what `WorkgroupTreeUnavailable` exists to keep separate. The candidate's other property — identical on all seven measured shapes — is verified and true; it is not sufficient.

**What landed instead: the admissible participant count *nearest* `MEASURED_TREE_PARTICIPANT_CAP`, ties to the narrower.** This is not a new rule so much as the completion of the one already documented. "The largest admissible count not exceeding the cap" is nearest-from-below and "when every admissible count exceeds the cap, the smallest admissible one" is nearest-from-above; both branches were already present, and neither ever compared against the other. That omission is the whole defect: at 514 the only admissible count at or below the cap is 2 while 257 sits one step above it.

### Why this one is safe where the candidate is not

The excursion bound is arithmetic, not fitted. A count `s` above the cap is taken over an admissible `l` at or below it only when `s - 256 < 256 - l`, and `l >= 2` forces **`s <= 509`**. So the rule widens past the calibrated 256 by at most 253 participants, never at all in the above-cap fallback branch, and 509 sits inside both authorities that refuse a width — `MAX_COOPERATIVE_PARTICIPANTS` of 4,096 and the widest workgroup any profile here declares, the qualified entry's 1,024. **No contributor count offered a tree before this rule loses one because of it**, and the decline set is unchanged: the domain is still exactly `governed_partition`'s.

### The evidence rung, by direction

- **Upward (never exceed 256 without cause):** empirical evidence, one host. Seven shapes, one profile, one contract, one program family, `f32`. Unchanged by this ticket, and `MEASURED_TREE_PARTICIPANT_CAP`'s value is untouched as the non-goals required.
- **Downward, structural claim:** *arithmetic* for the `s <= 509` bound, plus **exhaustive finite evidence** over a named finite range — `pipeline::tests::the_tree_widens_toward_the_cap_rather_than_truncating_at_it` enumerates every count below 4,096, reports 3,530 admitting ones, and pins the 1,061 this rule widens and the widest width reached.
- **Downward, cost claim ("nearer the cap is cheaper"):** `Unknown` at every count the rule moves. No measured shape has a non-power-of-two contributor count, and on a power of two the two formulations coincide, so **no measured cell exercised this at all**. The direction is an **Inference** from the calibration's steepest measured span (5.05x at four rows of 8,192, 9.53 µs at 256 participants against 48.15 µs at two) and is labelled as one in the code. The measurement boundary is the qualified Apple9 macOS host, one profile, one contract, one family, `f32`, power-of-two counts only.

### The residue, named rather than closed over

Below 20,000 contributors, **1,133** counts still take two participants, against 1,176 before. The smallest is 1,042 (`2 * 521`): 521 is admissible, representable, and inside the qualified entry's workgroup width, and the rule still declines it because 521 is 265 above the cap while 2 is 254 below. Nothing measured says which costs less, so the excursion width is filed as [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md) rather than fitted to no data.

### Populations, and what moved

| Population, contributor counts below 4,096 | Before | After |
| --- | --- | --- |
| admitting a participant count | 3,530 | 3,530 |
| where the tree's choice differs from the split's | 2,561 | 2,350 |
| where the tree's width exceeds the cap | 0 | 1,061 |
| widest width the rule reaches | 256 | 509 |

Smallest count at which the two strategies' groupings differ is still **twelve**, so the numerical record's twelve-contributor claim survives.

### Out of scope, and filed

This ticket held `implementation/compiler` only. Sites outside `crates/tiler-compiler/**` that restate the superseded rule are collected in [`restate-the-tree-width-rule-outside-the-compiler-crate`](restate-the-tree-width-rule-outside-the-compiler-crate.md), which also carries a defect that predates this work: `docs/compiler/fusion-and-scheduling.md` says "2,561 of the 3,530 admitting counts below 4,096 differ while the remaining 964 still agree", and `3,530 - 2,561` is 969.

## Later residue correction — 2026-08-09

The Outcome's sentence `The smallest is 1,042` conflates two populations. Four
is the smallest of the 1,133 contributor counts that take two participants,
and two is its only admissible participant count. The first count at which the
rule takes two while *declining a wider admissible count* is 1,042 (`2 * 521`).
The counts and distance arithmetic remain correct. The live compiler comment is
owned by
[`correct-the-two-participant-residue-s-smallest-count`](correct-the-two-participant-residue-s-smallest-count.md);
this dated note corrects the completed record without rewriting its historical
Outcome.
