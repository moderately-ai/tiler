---
id: correct-the-ir-contract-cooperative-synchronization-claims
title: Correct the IR contract's expired cooperative-synchronization claims
status: done
priority: p2
dependencies: []
related: [implement-the-single-workgroup-synchronized-reduction-strategy, admit-loop-carried-cooperative-staging]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## What is wrong

`docs/ir.md` still describes the schedule as owning no synchronization point at all, which three landings have falsified. The claims are load-bearing: a reader takes them as the current contract, and they make landed work look unreachable.

**Fact — the exact sentences, each reproducible in one `grep`.**

- "**No such point exists.** The normalized `KernelSchedule` still has no identity-bearing synchronization point, placement, ordering, or convergence proof, so a tile's edges are *recorded and never discharged*, and the structured-kernel verifier refuses any kernel whose region carries one." `SynchronizationPoint` carries all four (`crates/tiler-ir/src/schedule/synchronization.rs`), `verify_synchronization` requires exactly one discharging point per edge (`crates/tiler-ir/src/schedule/builder.rs`), and a cooperative kernel verifies, lowers, and has a checked-in Metal golden.
- "**Fact — the implemented schedule profile admits only the absence of synchronization…**" — the heading of the same paragraph, false for the same reason.
- "A region whose cooperative tile carries any visibility edge is rejected as `UndischargedVisibility`, before any body is derived. No canonical lowering exists for such a region" — `emit_cooperative` in `crates/tiler-ir/src/kernel/lower.rs` is that lowering, and `UndischargedVisibility` now names an edge *no point* discharges rather than the presence of one.

## Why it is filed rather than fixed in place

Found by `admit-loop-carried-cooperative-staging`, which corrected only the one sentence its own change falsified (the barrier-convergence rule). These predate it, come from `admit-the-first-typed-synchronization-point-and-atomic-target-authority` and `implement-the-single-workgroup-synchronized-reduction-strategy`, and correcting them means rewriting several paragraphs of a shared contract document — which is a different change from the one that found them.

## Closes when

Every sentence above states what the source does now, the surrounding paragraphs agree with it, and the second derived evidence class (`AntiDependencyEdge`), the round vocabulary, and the round-boundary placement are described where the visibility edges are. `docs/ir.md`'s tile-invariant list is the natural place for the per-round reading of the one-writer rule.
