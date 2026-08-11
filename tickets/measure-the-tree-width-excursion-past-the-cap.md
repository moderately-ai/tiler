---
id: measure-the-tree-width-excursion-past-the-cap
title: Measure the tree width excursion past the cap
status: in-progress
priority: p3
dependencies: []
related: [bound-the-tree-cap-s-unmeasured-downward-direction, cap-the-tree-reduction-participants-at-the-measured-256]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [reductions, scheduling, measurement]
claimed_from: todo
assignee: sol-tree-width
lease_expires_at: 1786431075
---
## The question

`capped_tree_partition` now takes the admissible participant count **nearest** `MEASURED_TREE_PARTICIPANT_CAP` rather than truncating at it, which fixed the direction but not its extent. The rule will widen past the cap by at most 253 participants, and that ceiling is arithmetic — a count `s` above the cap beats an admissible `l` at or below it only when `s - 256 < 256 - l`, and `l >= 2` forces `s <= 509` — rather than measured. **Nothing measured says the excursion should stop there, or anywhere.**

**Fact — corrected 2026-08-09.** Below 20,000 contributors, 1,133 counts still take **two** participants. Four is the smallest member, where two is the only admissible count. The first member at which the rule takes two while declining a wider admissible choice is 1,042 (`2 * 521`), whose only two admissible counts are 2 and 521: 521 is representable (`MAX_COOPERATIVE_PARTICIPANTS` is 4,096), inside the qualified Apple9 entry's 1,024 threads per workgroup, and stages 2,084 `f32` bytes against a declared 32,768. The rule declines it only because 521 is 265 above the cap while 2 is 254 below.

**Fact — the direction is inferred, not measured, at every count this concerns.** [The retained partition calibration](../spikes/program-planning/reduction-partition-calibration/README.md) states its own bound: "no shape here has a contributor count that is not a power of two". On a power of two the largest admissible count at or below the cap *is* the widest the cap admits, so no measured cell separates "nearest the cap" from "largest not exceeding the cap", and none bears on a width above the cap at all. The steepest measured span — the tree at four rows of 8,192, 9.53 µs at 256 participants against 48.15 µs at two, 5.05x — is what the inference rests on.

## The experiment

Sweep the tree's admissible participant counts at **non-power-of-two** contributor counts on the qualified Apple9 macOS host, reusing `spikes/program-planning/reduction-partition-calibration`'s harness, variant declaration, and per-element verification unchanged so the results are comparable cell by cell.

Shapes should include at least: 514 (`2 * 257`, admissible {2, 257}), 1,042 (`2 * 521`, {2, 521}), and one count with a dense sub-cap lattice and a sparse one just above it, so a cost curve either falls off past the cap or does not. Hold the row count at two of the crossover contour's separated values so a finding is not confined to one side of it.

## What would change

- A measured excursion boundary replaces the arithmetic 509 in `capped_tree_partition`, or confirms it.
- A measurement that the cost curve is flat between 2 and 521 at 1,042 contributors would close this as *no rule change needed* and would be worth recording, because the whole downward-direction argument rests on the curve being steep.
- A measurement disagreeing with the 5.05x direction at a non-power-of-two count would reopen `MEASURED_TREE_PARTICIPANT_CAP` itself.

## Non-goals

No change to `MEASURED_TREE_PARTICIPANT_CAP`'s value without evidence at its own power-of-two shapes. No selection change. No widening that reaches past `tiler_ir::schedule::MAX_COOPERATIVE_PARTICIPANTS` or past a declared workgroup width — a width preference that withdraws a legal alternative has decided feasibility, which is what ruled out `max(capped_tree_partition, governed_partition)` in [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md).

## Graph maintenance

Filed 2026-08-08 from [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md), which landed the rule and bounded the direction arithmetically rather than by measurement. Every count above was verified by exhausting the range at that ticket's base rather than taken from a report.
