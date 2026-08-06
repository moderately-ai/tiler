---
id: correct-the-reference-oracle-for-partitioned-output-writes
title: Correct the reference oracle for partitioned output writes
status: done
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

## Outcome

**Fact — `output_plans` now plans one buffer per output *boundary*, carrying every root that writes it.** `OutputPlan` holds a `Vec<OutputRoot>` (access plus value) beside one `elements` buffer sized to the boundary; `evaluate_point` fills one element per root at every parallel point; `finish_output` collects the shared buffer once. Boundaries are ordered by each one's first root, so a region no partition touches produces exactly the plans, in exactly the order, that one plan per root produced — every pre-existing oracle test is unchanged and green.

**Fact — the admitting boundary is total, and that is the decision recorded at `output_plans`.** No new `UnsupportedRegionFeature` was added. `IndexRegionBuilder::prepare_access` refuses any write whose domain is not exactly the region's parallel dimension set (`IndexBuildError::InvalidWriteDomain`, `crates/tiler-ir/src/index/builder.rs:1309`), so every root of an output is visited at every parallel point this walk makes; grouping the roots onto one buffer therefore reproduces every partition the IR can hand over. A refusal variant here would be one nothing could trigger, which reads as a guarantee while checking nothing.

**Inference — the premise fails closed rather than silently if the sibling ticket relaxes it.** A root whose domain were a strict subset of the parallel dimensions could not name the missing dimensions in its coordinates (`IndexBuildError::CoordinateOutsideAccessDomain`), so the full-space walk would send it to one element repeatedly and `DuplicateWrite` would refuse it. Disjointness and totality remain the oracle's own per-element checks over the shared buffer, independent of the verifier's joint proof — the same buffer the verifier's shared bitset covered.

**Superseded — the two Facts above hold only up to [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md).** That ticket relaxed `prepare_access` to admit any subset of the parallel dimensions, so the builder no longer refuses a strict-subset write domain and the admitting boundary is no longer total. The predicted fail-closed behaviour was also incomplete: a zero-extent omitted dimension empties the whole parallel product, so nothing is walked and `IncompleteWrite` names an innocent root rather than `DuplicateWrite` naming the defective one. [`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md) replaces both accidents with an explicit `UnsupportedRegionFeature::SubDomainWriteRoot` refusal at staging. Recorded here rather than rewritten above, because what this ticket decided was correct under the constraint it decided it under.

**Fact — the span-partition argument is restated over (root, parallel point) pairs and names all three proof forms.** `CoordinatePermutation`, `Exhaustive`, and `PartitionMember` with its `JointPartitionProofView`. Under the first two an output has one root and the pair collapses to the point, which is why the pair was invisible before partitions; under the third each root is injective by its own proof and the joint obligation makes the images pairwise disjoint and exactly covering, so pairs biject onto elements and the conclusion — a partition of the parallel points is a partition of each output's elements — survives unchanged.

**Measurement — the ticket's stated failure mode is not what the unfixed code does.** With `crates/tiler-reference/src/oracle.rs` reverted to `bdb6c1c4` and the new tests in place (`cargo nextest run -p tiler-reference --test index_region_oracle`), the concatenate case did not produce two half-filled tensors: `finish_output` refused first with `IncompleteWrite { access: VerifiedTensorAccessId { owner: VerifiedRegionOwner(1), index: 2 } }`. Per-root planning was therefore a **false refusal** of a region the IR admits for every partition with at least one element. The wrong *result* survives only where every root's rectangle is empty: `an_empty_partitioned_boundary_is_one_output_tensor` failed `left: 2, right: 1` — two output tensors reported for one declared boundary. Both tests pass after the change; `a_partition_missing_a_root_is_refused_before_evaluation` pins that a six-element boundary with one three-element root is refused at verification (`WriteOwnershipNotProven`).

**Fact — `IndexRegionEvaluation::outputs()` is now one tensor per output boundary.** No signature moved; the documented meaning did, and `IncompleteWrite`'s `access` field now documents that a jointly-owned output names its first root in region order rather than whichever root the fill reached last. Every in-tree consumer (`tiler-compiler`'s `governed.rs` and `legality.rs`) indexes `outputs()[0]` on single-output regions and is unaffected.
