---
id: correct-the-reference-oracle-for-partitioned-output-writes
title: Correct the reference oracle for partitioned output writes
status: todo
priority: p1
dependencies: [admit-a-partitioned-write-ownership-contract]
related: [admit-sub-range-write-domains-for-unequal-partitions, lower-the-concatenate-occurrence-through-partitioned-writes]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, indexing, write-ownership]
---
## User-visible outcome

The reference oracle evaluates an index region whose output several roots partition into one joined tensor, and its span-partition argument states the fact that actually holds, so the oracle stops being wrong about regions the IR now admits.

## Why this exists

**Fact — the oracle allocates one output tensor per write *root*.** `output_plans` (`crates/tiler-reference/src/oracle.rs:1984-2023`) iterates `region.outputs()` and pushes one `OutputPlan` per root, each with `elements: vec![None; count]` sized to the whole tensor. A region with two roots over one output therefore produces two full-size buffers, each filled only where its own root wrote, instead of one buffer both roots jointly filled. That is a wrong result rather than a refusal.

**Fact — the doc comment enumerates two proof forms where three now exist.** `oracle.rs:1428-1433` reads "Every write access carries a `WriteOwnershipProofView`: `CoordinatePermutation` … or `Exhaustive`", and concludes "a partition of the parallel points is a partition of each output's elements". [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) added `WriteOwnershipProofView::PartitionMember`, for which that sentence is false: one parallel point now writes one element *per root* of a partitioned output, so the point-to-element map is no longer a bijection per output.

**Inference — the conclusion the argument reaches still holds, and only its premise moved.** The joint obligation proves the roots' images are pairwise disjoint and cover the boundary exactly, so the map from (root, parallel point) pairs to elements is still a bijection and no span can land on an element another span produced. The argument needs restating over pairs rather than over points; it does not need replacing.

**Fact — the first of the three facts the argument rests on is also under revision elsewhere.** It states that a write's iteration domain is exactly the region's parallel dimension set, citing `InvalidWriteDomain`. [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) may relax exactly that.

## What the work is

Group output roots by tensor when building `OutputPlan`s, so one plan is filled by every root that writes it, and decide what the evaluator does with a region the IR admits but whose partition the oracle cannot reproduce — a refusal with a named `UnsupportedRegionFeature` is the fail-closed option and is preferable to a partially filled tensor.

Restate the span-partition argument over (root, point) pairs, and enumerate the third proof form where the comment enumerates the other two. The comment is load-bearing: it is the recorded justification for evaluating spans concurrently.

## Explicit non-goals

- The partition contract, which exists in `crates/tiler-ir/`.
- Sub-range write domains, which is the ticket named above and would revise a different premise of the same argument.

## Closes when

A partitioned region either evaluates to one correctly joined tensor per output, or is refused under a named unsupported-feature reason; the span argument names all three proof forms and is stated over the entity that is actually in bijection; and a deliberate perturbation dropping one root's contribution is shown to fail.

## Graph maintenance

- `implementation/reference` alone: `oracle.rs` is in `crates/tiler-reference/`.
- Filed by the partition-contract ticket, whose exclusive scope was `implementation/ir` and which therefore could not correct a consumer it had made wrong. The defect is a wrong evaluation, not only a stale comment, which is why it is p1.
