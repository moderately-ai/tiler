---
id: calibrate-the-reduction-partition-against-measured-alternatives
title: Calibrate the reduction partition against measured alternatives
status: in-progress
priority: p3
dependencies: [calibrate-and-activate-parallel-reduction-selection]
related: [activate-measured-reduction-selection-from-a-target-cost-row]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, measurement, scheduling, reductions]
claimed_from: todo
assignee: agent-partition-cal
lease_expires_at: 1786086448
---
## User-visible outcome

The contributors-per-partition both parallel reduction strategies use is either confirmed as the best available choice on the qualified profile, or replaced by one that measurement supports.

## Why this exists, and what it is not

**Fact — `governed_partition` is a choice nothing has measured.** `crates/tiler-compiler/src/physical.rs` returns the divisor of the contributor count nearest its integer square root from below, and its own doc says so: "deliberately *a* choice and not a calibrated one". Both parallel strategies read it — the multi-pass split's partition count and the single-workgroup tree's participant count are the same number — so it fixes the tree's workgroup width as well as the split's partial extent.

**Fact — the 2026-08-07 crossover sweep did not measure it.** [`spikes/program-planning/reduction-dispatch-crossover`](../spikes/program-planning/reduction-dispatch-crossover/README.md) varied the shape across 92 cells and timed three strategies at each, but every cell used whatever partition this function returned. It is evidence about *which strategy*, and the partition was a constant of the experiment rather than a variable in it. No value of this function is confirmed or refuted by it.

**This is p3 and separated on purpose.** The measured strategy contour spans two orders of magnitude; a partition choice plausibly moves a plan by a much smaller factor, and choosing the wrong strategy is what the priority belongs to. Splitting it also keeps the activation ticket's surface to one term.

## Implementation keys

Sweep the partition at fixed shapes rather than the shape at a fixed partition — the inverse of the retained sweep, reusing its harness discipline. The partition is not freely settable through the public compiler entry point, so state how it is varied and keep that mechanism out of the shipped path.

Cover both strategies, because they consume the number differently: the split's partition count is a launch extent and its contributors-per-partition is a fold length, while the tree's partition count is *also* its workgroup width and its threadgroup reservation. A partition that is best for one need not be best for the other, and a single calibrated value would then be a compromise that should be named as one.

Pick shapes from the retained sweep's separated cells so the result composes with it rather than describing a different regime.

## Required evidence

Retained raw measurements on the qualified environment with its exact rows, or an explicit statement that the environment was unavailable. The verdict per strategy, with the noise band that makes it a verdict. If the balanced exact split is confirmed, say by how much it beats the alternatives and over which shapes; if it is not, name the shapes where it loses and by how much.

## Closes when

The record states, with retained measurements, whether the balanced exact split is the best available choice for each strategy on this profile, or which choice replaces it and where.
