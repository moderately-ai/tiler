---
id: name-the-fact-source-on-retained-write-ownership-evidence
title: Name the fact source on retained write-ownership evidence
status: in-progress
priority: p2
dependencies: []
related: [bound-a-symbolic-index-coefficient-interval-from-its-declared-extent]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [indexing, proofs]
claimed_from: todo
assignee: sol-fact-source
lease_expires_at: 1786430540
---
## User-visible outcome

A caller reading a write access's retained ownership evidence can tell whether the proof rested on this region's shape environment or on the program's own literals, exactly as it already can for the access's bounds evidence.

## Why this exists

**Fact — the bounds half landed and the ownership half did not.** [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md) settled the environment-in-proofs question: a proof may read the region's shape environment, because the region's canonical identity folds that environment's identity, so a fact read from it is a fact about *this* region; a rewrite may not, because the node's bytes must keep naming the symbol. It then made the answer legible by retaining an `IndexDomainFactSource` on every discharged index-domain assessment and on every `BoundsProofView`.

**The ownership obligation rests on the same environment reads and says nothing about it.** `write_is_permutation` decides its per-axis equality through `extents_proved_equal`, which is `ShapeEnv::proves_equal` for a symbolic extent — that is the whole mechanism a dynamically shaped output rests on. `decide_partition_by_interval` places every rectangle through `determined_extent` and `boundary_extents`, and `partition_walk_elements` gates the joint walk on the same queries. So `WriteOwnershipProofView::CoordinatePermutation` on a `[m] -> [m]` copy and on a `[4] -> [4]` one are indistinguishable, and `JointPartitionProofView::Interval` over symbolic extents is indistinguishable from one over literals.

**Why it was left.** `WriteOwnershipProofView` and `JointPartitionProofView` are pattern-matched in `crates/tiler-compiler/src/governed.rs` as `PartitionMember { joint: JointPartitionProofView::Interval }`. Adding a field to either breaks that match, and `implementation/compiler` was held by a parallel worker when the bounds half ran, so the producing ticket could not reach it. This is a scope boundary, not a design disagreement: the rule is already decided and this is applying it to the remaining half.

## What this ticket owes

- `WriteOwnershipProofView` and `JointPartitionProofView` carry an `IndexDomainFactSource`, reached through one accessor rather than an optional sibling whose complementarity only a test would hold — the shape `BoundsProofView` already uses.
- The `crates/tiler-compiler` match sites are updated with it.
- A test drives both sides: a dynamically shaped `[m] -> [m]` write records `ShapeEnvironment` and a static `[4] -> [4]` one records `Program`, and the same for a partition decided by interval.

## Also in scope, because it is the same rule

`coordinate_offset_dimension` declines a symbolic coefficient outright, on the ground that it "is not the unit this vocabulary requires and is not known to be anything else". Under the settled rule a coefficient the environment pins to one *is* known to be one, so `u * d` with `u == 1` could be placed as a rectangle exactly as `d` is, instead of falling to the joint enumeration. Decide it explicitly — take it or carve it out with a reason — rather than leaving a third site on the old line.

## Explicit non-goals

Not a change to normalization, and not a widening of any coefficient or extent vocabulary. Not a re-litigation of the environment-in-proofs decision; that is settled and this applies it.

## Closes when

No retained ownership or joint-partition evidence leaves a caller unable to tell whether the environment was read, and `coordinate_offset_dimension`'s treatment of a determined symbolic coefficient is decided in the record rather than inherited.

## Graph maintenance

Filed 2026-08-07 by the worker of the bounds half, from a remainder that a live parallel claim on `implementation/compiler` put out of reach rather than one it chose to omit. Identity note: the ownership proofs are **not** part of the canonical region encoding — `encode_region` writes an access's mode, tensor, domain, and coordinates and no proof — so this should move no pin. Confirm that on the branch rather than assuming it.
