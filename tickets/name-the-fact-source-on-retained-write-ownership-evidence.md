---
id: name-the-fact-source-on-retained-write-ownership-evidence
title: Name the fact source on retained write-ownership evidence
status: awaiting-decision
priority: p2
dependencies: []
related: [bound-a-symbolic-index-coefficient-interval-from-its-declared-extent]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [indexing, proofs]
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

## Awaiting Tom — exact public variant shapes

**Independent-review stop — 2026-08-11, candidate `849f0bcdfbca0bae7790014833866f32c3d488fc`, base `099c6e2dfd236af59eedbb01d7e3bd67badca767`.** The candidate correctly propagates the fact source, keeps proof data outside canonical identity, and passes its subject perturbations and full gate. It cannot merge under the authority currently recorded. `accept-the-partitioned-write-ownership-proof-boundary` says Tom accepted the exact `WriteOwnershipProofView::PartitionMember { joint }` and `JointPartitionProofView::{Interval, Exhaustive { points }}` surface on 2026-08-06. The candidate changes all three `WriteOwnershipProofView` variant signatures and both `JointPartitionProofView` variant signatures by adding fields — most visibly `CoordinatePermutation` becomes `CoordinatePermutation { facts }` and `Interval` becomes `Interval { facts }`. The required compiler and public-trybuild edits demonstrate that this is a breaking change to existing public signatures, which ADR 0075 always routes to Tom. Calling it additive `#[non_exhaustive]` growth would be false: no variant is added; accepted variant shapes change.

**Option A — accept the field-bearing proof variants (recommended).** Accept the five revised variant shapes and the two total `facts()` accessors exactly as candidate `849f0bcd` spells them. This keeps each proof mechanism and its premise source in one value, makes every future variant decide its source explicitly, and avoids an optional sibling whose complementarity exists only by convention. The strongest counterpoint is that it revises a surface accepted only five days earlier and makes exact in-workspace patterns move.

**Option B — preserve the accepted variants and design an additive evidence wrapper.** Keep every accepted variant signature byte-for-byte and introduce a separate total wrapper/accessor carrying `{ proof, facts }`. This avoids changing the accepted vocabulary. The strongest counterpoint is that retaining the old source-less accessor or value gives callers two competing views, while replacing its return type is itself a breaking signature change; a satisfactory wrapper therefore needs an explicit compatibility and deprecation shape rather than the optional sibling this ticket rejects.

**Recommendation.** Choose Option A. The proof rule is already settled, the candidate demonstrates exact propagation over standalone and joint ownership, and no identity bytes move; the remaining choice is solely whether those accepted public variants may acquire the field-bearing shapes. Until Tom accepts one exact surface, the implementation stays on its branch and this ticket remains `awaiting-decision`.
