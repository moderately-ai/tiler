---
id: admit-sub-range-write-domains-for-unequal-partitions
title: Admit sub-range write domains for partitions of unequal extent
status: in-progress
priority: p1
dependencies: []
related: [admit-a-partitioned-write-ownership-contract, lower-the-concatenate-occurrence-through-partitioned-writes, scope-the-concatenate-fusion-role-and-lowering]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership]
claimed_from: todo
assignee: agent-subrange
lease_expires_at: 1785994006
---
## User-visible outcome

Two write roots over one output may iterate different sub-ranges of the region's parallel domain, so a partition whose members have *unequal* extents — which is every concatenation of unequally sized operands — becomes expressible rather than unstatable.

## Why this exists

**Fact — the partition contract landed with one shared domain, deliberately.** [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) admitted partition-relative totality with joint disjointness and coverage across roots, and preserved `IndexBuildError::InvalidWriteDomain` (`crates/tiler-ir/src/index/builder.rs:1308-1310`) on its own ticket's instruction: that site is a rule about a write's *domain*, not about coverage, and a region whose writes iterate different sub-domains is a different construct from one whose single domain is partitioned by coordinate.

**Inference — that choice bounds what a partition can express, and the bound bites.** Every write in a region iterates the complete parallel dimension set, so every root's domain-point count is the same. A root is admitted only when its point-to-coordinate map is injective, so each root owns exactly that many elements. A partition of `n` roots therefore covers `n * points` elements in equal shares, and any partition whose members differ in size is unrepresentable. Concretely: two operands of extent 3 and 5 joined into an output of extent 8 has no spelling — a shared domain of 5 makes the extent-3 root non-injective, and a shared domain of 3 leaves the extent-5 root unable to reach its own elements.

**Fact — the equal-share case is genuinely admitted, so this is a gap rather than a total block.** `contiguous_partitions_are_admitted_by_interval_reasoning` and `strided_partitions_fall_back_to_the_recorded_joint_walk` (`crates/tiler-ir/tests/index_region.rs`) build regions whose roots tile and interleave a boundary respectively.

**Fact — the dependent lowering needs the unequal case at its pinned occurrence.** [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) requires the zero-extent operand case, `[8, 0, 128]` joined with `[8, T, 128]`, which is maximally unequal.

## What the work is

Decide whether a write may declare a domain that is a subset of the region's parallel dimensions, or a sub-*range* of a dimension's extent, and record which construct is admitted — they are not the same relaxation and the difference is what `InvalidWriteDomain` currently forecloses.

Whichever is admitted, re-derive the two obligations the current contract rests on rather than assuming they carry: per-root injectivity, which today follows from "each coordinate consumes one whole shared dimension exactly once"; and the interval mechanism's volume identity, which today reads a root's rectangle volume as its element count. Both arguments are written out in `write_partition_box` and `decide_partition_by_interval` (`crates/tiler-ir/src/index/builder/proof.rs`) precisely so a later relaxation can check them rather than re-invent them.

Confirm what a sub-range domain does to every consumer that reads a write's domain as *the* region parallel domain. `crates/tiler-reference/src/oracle.rs:1420-1427` states that equivalence explicitly as the first of three facts its span-partition argument rests on.

## Explicit non-goals

- The joint coverage and disjointness contract itself, which exists and is not reopened here.
- The concatenate lowering, which is its own ticket and consumes this.

## Closes when

A region whose roots partition one output into unequally sized contiguous pieces builds, verifies, and canonicalizes; the injectivity and volume arguments are restated for the admitted construct; and a deliberate perturbation that makes two unequal partitions overlap or leave a gap is shown refusing under its existing diagnostic.

## Graph maintenance

- `implementation/ir` alone: the refusal site, the proof code, and the two arguments to re-derive are all in `crates/tiler-ir/`.
- Filed by the partition-contract ticket on discovering that preserving `InvalidWriteDomain` — which its own body instructed — leaves the unequal case unstatable. Recorded there rather than absorbed silently.

## Oracle site note — 2026-08-06

The oracle correction (`correct-the-reference-oracle-for-partitioned-output-writes`, done) rests its admit-everything decision on `InvalidWriteDomain` holding: every root iterates the whole parallel domain, so grouped filling reproduces every statable partition, and no unsupported-feature refusal exists because none is reachable. This ticket relaxes exactly that premise. The failure mode is closed rather than silent — a sub-domain root's coordinates cannot name the missing dimensions, so the full-space walk revisits an element and `DuplicateWrite` refuses — but a refusal by accident is not a contract: re-read `output_plans` (`crates/tiler-reference/src/oracle.rs`) and its "Which partitioned regions this admits" doc when relaxing, and decide the oracle's admitting boundary for sub-range roots deliberately.
