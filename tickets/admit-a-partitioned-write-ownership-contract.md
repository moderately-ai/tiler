---
id: admit-a-partitioned-write-ownership-contract
title: Admit a partitioned write-ownership contract for one output
status: done
priority: p1
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, scope-an-in-place-append-into-a-caller-retained-allocation, scope-the-scatter-and-indexed-update-family]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership, public-boundary]
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

## Outcome

**The contract exists because the joint obligation is checked rather than assumed, and the two mechanisms that check it are recorded rather than inferred.** A third `WriteOwnershipProof` form carries partition-relative totality; what makes the *output* owned is a separate obligation over the whole root set, discharged by interval reasoning where every root is a contiguous static rectangle and by one shared enumeration bitset otherwise. Which of the two answered is carried in the proof, so a consumer re-deriving the obligation knows what population was decided.

**Fact — the correctness argument for joint coverage, and why disjointness is not optional.** Each root's rectangle is admitted only when each axis consumes at most one domain dimension with unit coefficient and the consumed set is the whole domain; two distinct domain points then differ in some dimension appearing in exactly one axis, so the point-to-coordinate map is injective and the rectangle's volume *is* the count of distinct elements that root writes. Pairwise disjointness is then decided exactly — two axis-aligned rectangles intersect exactly when their ranges overlap on every axis, so one separating axis refutes and no separating axis establishes — and coverage follows from disjointness: rectangles pairwise disjoint and contained in the boundary have a union of exactly the summed volume, so a sum equal to the element count means the union *is* the boundary. The order is load-bearing. Applied without the disjointness premise the same sum admits a set that double-covers one element and leaves another bare, which is why `decide_partition_by_interval` refutes on overlap before it computes a volume, and why containment is what makes an inequality provably a shortfall rather than an unexplained disagreement.

**Fact — the four refusal-site decisions, each with its reason.**

1. `DuplicateOutputTensor` (`builder.rs:1899-1901`) — **relaxed, and the variant removed.** It is the site the contract exists to open. Whether two roots over one boundary partition it is not decidable at `output()`: it depends on coordinates not yet authored and extents the environment resolves later. The check also refused for the wrong reason, reporting "this tensor already has a root" where the real question is joint ownership. The obligation moved to verification, where every coordinate exists, under three diagnostics that name what is actually wrong. A root repeated verbatim is the degenerate partition of two members in one rectangle and refuses as an overlap, so nothing the old check caught is now admitted.
2. `InvalidWriteDomain` (`builder.rs:1308-1310`) — **preserved**, on this ticket's own instruction and for the reason it gives: it is a rule about the write's *domain*, not about coverage. This contract partitions one shared domain *by coordinate*. **Inference — and this bounds what is expressible, which the ticket did not state and a reader must not assume away.** Every write iterates the complete parallel dimension set and every admitted root is injective over it, so every root owns the same element count and only *equal-share* partitions are representable. Two operands of extent 3 and 5 joined into an extent-8 output has no spelling. Filed as [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) and added as a dependency of the concatenate lowering, whose pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence is maximally unequal.
3. The exhaustive ownership walk (`proof.rs:702-712`) and `write_is_permutation` (`:806-823`) — **relaxed additively; neither predicate changed.** Whole-boundary coverage still holds for a root that owns its output alone, so every pre-existing region takes the identical path and retains the identical evidence. A root whose output is partitioned is excluded from the per-access coverage bitset by an `owns_alone` condition read once and consumed in four places; its ownership is decided once for the whole set.
4. `MultipleWriters` (`program/verify.rs:197-199`) — **preserved.** It is a kernel-program rule about two *stages* writing one materialized value, a different layer and a different construct from two write roots inside one index region, which reaches that layer as one stage. Relaxing it is the physical contract the sequence-extending record names — windowed binding, `ExternalValueWritten`, untouched-byte validity — and is not this ticket's.

**Fact — no identity-domain step, and the check is one line.** `encode_region` (`builder/identity.rs:413-422`) encodes an access's mode, tensor, domain, and coordinates; it encodes neither `bounds_proof` nor `ownership_proof`, so a proof form is outside canonical identity by construction. Every pre-existing region also takes an unchanged path, so no discharged or unknown index-domain predicate moves either. **Measurement — `cargo nextest run --workspace` on this branch: 2683 passed, 0 failed, 7 skipped**, which exercises the 20 pinned explain digests (`tiler-compiler/src/explain.rs:3807-4072`) and the six `tiler-metal/goldens/*.metal` shader identity pairs. `git diff --stat` touches seven files, all under `crates/tiler-ir/`; no golden, digest, or ledger file is in the diff. The pin population was surveyed before editing: 44 sixteen-hex and 33 sixty-four-hex literals across `crates/`, none inside `tiler-ir` except two unrelated normative-definition references.

**Measurement — every new check was observed refusing.** Four sites were deliberately perturbed to never refute (`if false &&` on the disjointness test, the interval volume identity, the walk's double-write bit, and the walk's coverage scan). Under the perturbation five refusal tests failed and the two admitting tests still passed: `overlapping_ranges_refuse_before_coverage_is_considered` and `a_repeated_output_root_is_admitted_then_refused_as_an_overlap` (both then reached `Ok`), `an_uncovered_element_refuses_under_interval_reasoning`, `an_uncovered_element_refuses_under_the_joint_walk`, and `a_double_written_element_refuses_under_the_joint_walk`. The perturbations were reverted and the full suite re-run green. The three named diagnostics are `OutputPartitionUncovered` (raised by either mechanism), `OutputPartitionRangesOverlap` (interval), and `OutputPartitionDoubleWritten` (enumeration); overlap and double-write are named apart because they are decided over different populations and the diagnostic tells the caller which check spoke.

**Inference — one consequence recorded rather than fixed.** A partitioned output consumes several ordered output-root positions, so root declaration order enters canonical identity even though it is not semantic within one partition. Left as is deliberately: `outputs` is declaration-ordered today and that ordering is semantic across distinct output tensors, so canonicalizing it would move the identity of every existing region — an identity-domain step for a conservative distinction that never conflates two regions, only fails to identify two spellings of one.

## Public boundary — draft, for Tom

Not self-accepted. Three public items changed in `crates/tiler-ir/src/index/`, tested but not approved:

- **Added** `WriteOwnershipProofView::PartitionMember { joint: JointPartitionProofView }` on the `#[non_exhaustive]` view (`model.rs`). Additive; the repository's own `trybuild` pass case already carries a wildcard arm, so no external match breaks.
- **Added** `pub enum JointPartitionProofView { Interval, Exhaustive { points: u64 } }`, `#[non_exhaustive]`, re-exported from `index::mod`. A separate type rather than fields on the variant so the two mechanisms are named and a third is a build error at every exhaustive site.
- **Removed** `IndexBuildError::DuplicateOutputTensor`. A removal from a `#[non_exhaustive]` public enum, and the only one of the three that is not additive. Kept as a removal rather than a repurposing because the name states a rule the contract no longer holds, and a variant whose name lies is worse than one that is gone.

Three diagnostics were added to the `#[non_exhaustive]` `IndexRegionDiagnostic`; additive, and flagged for completeness rather than as a decision.

## Follow-up work filed

| Ticket | Why it is separate |
| --- | --- |
| [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) | Relaxing `InvalidWriteDomain` is the separate decision this ticket's body names; added as a dependency of the concatenate lowering, which needs it. |
| [`bind-a-partitioned-output-through-index-refinement`](bind-a-partitioned-output-through-index-refinement.md) | `bind_results` (`index/refinement.rs:2744-2790`) requires one root per semantic result and refuses a partition as `ResultArity`. Relaxing it redesigns the public `ResultBinding`. |
| [`correct-the-reference-oracle-for-partitioned-output-writes`](correct-the-reference-oracle-for-partitioned-output-writes.md) | `output_plans` (`tiler-reference/src/oracle.rs:1984-2023`) allocates one full-size tensor per *root*, so a partitioned region evaluates to two half-filled tensors — a wrong result, not only the stale two-form enumeration in its span argument. `crates/tiler-reference/` is outside this ticket's exclusive scope. |
