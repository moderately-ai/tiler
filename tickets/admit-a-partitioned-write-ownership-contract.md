---
id: admit-a-partitioned-write-ownership-contract
title: Admit a partitioned write-ownership contract for one output
status: in-progress
priority: p1
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, scope-an-in-place-append-into-a-caller-retained-allocation, scope-the-scatter-and-indexed-update-family]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership, public-boundary]
claimed_from: todo
assignee: agent-partition-proof
lease_expires_at: 1785982711
---
## User-visible outcome

An index region proves that several write roots jointly own one output — each total over its own declared partition, the partitions disjoint and covering — so that a lowering which writes an output in pieces is refused for a stated reason or admitted with a proof, rather than being unstatable.

## Why this exists

**Fact — the contract does not exist and four sites refuse it, each read in full at `d5960e81`.** `crates/tiler-ir/src/index/builder.rs:1899-1901` returns `DuplicateOutputTensor` for a second output root over one tensor. `crates/tiler-ir/src/index/builder.rs:1308-1310` returns `InvalidWriteDomain` unless a write's domain equals the region's complete parallel dimension set, so a write cannot iterate a sub-range. `crates/tiler-ir/src/index/builder/proof.rs:702-712` requires the exhaustive ownership walk to cover every element of the whole output tensor, and `write_is_permutation` (`:806-823`) sends any offset write down that path by demanding bare `Dimension` coordinates. `crates/tiler-ir/src/program/verify.rs:197-199` returns `MultipleWriters` if the partitions are separate regions instead.

**Fact — `WriteOwnershipProof` has two forms and neither expresses a partition.** `CoordinatePermutation` and `Exhaustive { points }` (`crates/tiler-ir/src/index/model.rs:140-144`) each prove one access total and injective over its own whole boundary. [Sequence-extending tensor family](../docs/research/shapes/sequence-extending-tensor-family.md) named the gap in the same words: neither expresses "total over a partition and disjoint from a sibling partition", so the partitioned form owes a third proof kind and a joint-coverage obligation across roots.

**Inference — this is a shared prerequisite, not a concatenate detail.** The concatenate lowering needs it ([Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md)), the in-place append into a caller-retained allocation needs it ([`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md), which owes it in common with the mechanism it eliminated), and the scatter family's duplicate-write and determinism rules sit on the same subject.

## What the work is

Admit a third `WriteOwnershipProof` form carrying partition-relative totality, and the joint obligation across the roots of one output: the partitions are pairwise disjoint and their union covers the output exactly. For contiguous coordinate ranges fixed by static extents the joint question is decidable by interval reasoning without enumeration, and the exhaustive walk stays the fallback rather than the mechanism — but which of the two a given root uses must be recorded in the proof, not chosen silently.

Decide each of the four refusal sites explicitly, and preserve the refusal where the relaxation is not what the contract admits. `InvalidWriteDomain` in particular is a rule about the write's *domain*, not about coverage, and relaxing it to admit a sub-range domain is a separate decision from admitting a partition — a region whose writes iterate different sub-domains is a different construct from one whose single domain is partitioned by coordinate.

Prove the new check can say no. A partition set that leaves one element uncovered, one that double-covers an element, and one whose ranges overlap must each be refused under their own named diagnostic, and each refusal exercised before the admitting case is trusted.

## Explicit non-goals

- The concatenate's index-access capability, which is [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) and depends on this.
- In-place writing into a caller-bound input. `ExternalValueWritten` stays; relaxing it is a named contract with its own identity under Q-PLAN-015 and is not this ticket's.
- Proving anything about bytes a region does not write. A partition that covers the output leaves no untouched remainder; the partially-written-value question is a different subject and stays where the sequence-extending record left it.

## Closes when

A region with several write roots over one output builds, verifies, and canonicalizes with a proof naming its partition form; the three deliberate failure cases each refuse under their own diagnostic; and each of the four refusal sites is either relaxed with its reason recorded or preserved with its reason recorded.

## Stop conditions

**The public boundary is Tom's.** `WriteOwnershipProofView` (`crates/tiler-ir/src/index/model.rs:970-979`) is `pub` and `#[non_exhaustive]`; a variant on it is a public boundary under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) and must not be self-accepted. A tested implementation is a concrete draft, not approval of its spelling.

**An identity-domain step ends the dispatch.** Adding a proof form changes an index region's canonical encoding. If any pinned identity, golden, or explain digest moves, that is a complete step — version at its owning layer, ledger documents in the same commit, every pin recomputed on the merged tree and enumerated — or it is not started.

## Graph maintenance

- Only `implementation/ir` is declared: all four refusal sites and the proof form live in `crates/tiler-ir/`. The compiler-side consumer is the dependent ticket's and declares its own scopes.
