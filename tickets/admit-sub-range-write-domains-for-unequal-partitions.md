---
id: admit-sub-range-write-domains-for-unequal-partitions
title: Admit sub-range write domains for partitions of unequal extent
status: done
priority: p1
dependencies: []
related: [admit-a-partitioned-write-ownership-contract, lower-the-concatenate-occurrence-through-partitioned-writes, scope-the-concatenate-fusion-role-and-lowering, state-the-oracle-boundary-for-sub-domain-write-roots, evaluate-write-roots-over-their-own-domains-in-the-oracle, accept-the-sub-domain-write-domain-surface, prove-partition-coverage-for-symbolic-extents, correct-the-write-domain-rule-in-the-indexing-corpus]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, indexing, write-ownership]
---
## User-visible outcome

Two write roots over one output may iterate different sub-ranges of the region's parallel domain, so a partition whose members have *unequal* extents — which is every concatenation of unequally sized operands — becomes expressible rather than unstatable.

## Why this exists

**Fact — the partition contract landed with one shared domain, deliberately.** [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) admitted partition-relative totality with joint disjointness and coverage across roots, and preserved `IndexBuildError::InvalidWriteDomain` on its own ticket's instruction. At that landing the error enforced equality of write domain to the full parallel set; after this ticket the live refuse is the `prepare_access` write path that returns `InvalidWriteDomain` when a write domain names a non-parallel dimension (`role != DomainRole::Parallel` in `crates/tiler-ir/src/index/builder.rs`). That site remains a rule about a write's *domain*, not about coverage, and a region whose writes iterate different sub-domains is a different construct from one whose single domain is partitioned by coordinate.

**Inference — that choice bounds what a partition can express, and the bound bites.** Every write in a region iterates the complete parallel dimension set, so every root's domain-point count is the same. A root is admitted only when its point-to-coordinate map is injective, so each root owns exactly that many elements. A partition of `n` roots therefore covers `n * points` elements in equal shares, and any partition whose members differ in size is unrepresentable. Concretely: two operands of extent 3 and 5 joined into an output of extent 8 has no spelling — a shared domain of 5 makes the extent-3 root non-injective, and a shared domain of 3 leaves the extent-5 root unable to reach its own elements.

**Fact — the equal-share case is genuinely admitted, so this is a gap rather than a total block.** `contiguous_partitions_are_admitted_by_interval_reasoning` and `strided_partitions_fall_back_to_the_recorded_joint_walk` (`crates/tiler-ir/tests/index_region.rs`) build regions whose roots tile and interleave a boundary respectively.

**Fact — the dependent lowering needs the unequal case at its pinned occurrence.** [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) requires the zero-extent operand case, `[8, 0, 128]` joined with `[8, T, 128]`, which is maximally unequal.

## What the work is

Decide whether a write may declare a domain that is a subset of the region's parallel dimensions, or a sub-*range* of a dimension's extent, and record which construct is admitted — they are not the same relaxation and the difference is what `InvalidWriteDomain` currently forecloses.

Whichever is admitted, re-derive the two obligations the current contract rests on rather than assuming they carry: per-root injectivity, which today follows from "each coordinate consumes one whole shared dimension exactly once"; and the interval mechanism's volume identity, which today reads a root's rectangle volume as its element count. Both arguments are written out in `write_partition_box` and `decide_partition_by_interval` (`crates/tiler-ir/src/index/builder/proof.rs`) precisely so a later relaxation can check them rather than re-invent them.

Confirm what a sub-range domain does to every consumer that reads a write's domain as *the* region parallel domain. At filing time the oracle documented write-domain equality to the region parallel domain as a span-partition premise at that cite; after follow-up work the equality premise is gone — root-point docs and `output_plans` state that a write's iteration domain is any subset of the region's parallel dimensions, and each root walks `DomainWalk::new(self.domain(access.domain())?)` (`crates/tiler-reference/src/oracle.rs`).

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

## Outcome

**The subset-of-parallel-dimensions construct is admitted; the sub-range annotation is eliminated.** A write's domain may be any subset of the region's parallel dimensions, so each root carries its own iteration space and the region's parallel set is their union. This is the answer [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) is told not to invent a second one to: **several iteration domains, one per root**, not one domain partitioned by coordinate.

**The elimination.** Both candidates express the unequal partition, so neither is eliminated on capability; three other grounds decide it.

1. *Identity.* The subset construct adds nothing to `AccessData` — `domain` is already a per-access dimension vector that `encode_region` already encodes — so no encoding changes and no identity moves. A sub-*range* annotation is a new per-access, per-dimension field; it enters `encode_region`, steps `INDEX_REGION_DOMAIN` from `v9`, and forces recomputation of every pinned identity downstream. That is a real cost bought for no expressive gain.
2. *Duplication.* A sub-range `[lo, hi)` of a dimension `j` is exactly a fresh dimension of extent `hi - lo` with the coordinate `lo + j'`, and `coordinate_offset_dimension` already reads precisely that displaced-dimension form. Admitting the annotation would give one meaning two spellings, so two regions that mean the same thing could canonicalize to different bytes — the alpha-equivalence hazard `compact` exists to prevent.
3. *Symbolic reach.* The pinned occurrence's sub-range endpoints are `0` and `T`. A range annotation would need symbolic endpoints and symbolic endpoint arithmetic to say anything about that case; a dimension already carries a `SourcedExtent`, so the construct needs no new vocabulary to state it (proving it is a separate gap, recorded below).

One candidate survives on all three, so per `AGENTS.md` there was no question to bring to Tom — only a surface to label as a draft, which is filed as [`accept-the-sub-domain-write-domain-surface`](accept-the-sub-domain-write-domain-surface.md).

**Inference — per-root injectivity carries, because it never used the premise that changed.** `write_partition_box`'s argument quantifies only over `access.domain`: each axis consumes at most one domain dimension (`consumed.insert`), every consumed dimension is in the domain (`access.domain.contains`), and equal cardinality makes `consumed` exactly `access.domain`. Two distinct points of *that* domain differ in some `d ∈ consumed`; `d` appears in exactly one axis; the unit coefficient carries the difference into that coordinate. Nothing in the chain mentions the region's parallel set. What the old equality rule bought was the *global* corollary that all roots share one point count and therefore own equal shares — dropping it drops only that corollary. The argument is now written out at the site.

**Inference — the volume identity carries for the same reason, including at zero.** A root's rectangle volume is the product over axes of each span, which is `extent(d)` for a consumed `d` and `1` for a constant axis, hence the product of the extents of `access.domain` — that domain's point count, which injectivity makes the count of distinct elements written. A zero-extent domain dimension gives a zero span, an empty rectangle, zero volume, and a root that writes nothing: the degenerate case of the same identity rather than an exception to it.

**Fact — one thing did *not* carry, and it was closed by accident before.** `decide_partition_by_interval`'s per-axis separation test decides `b > c && d > a`, which is `max(a, c) < min(b, d)` only when both ranges are nonempty. An empty rectangle is disjoint from everything, but the axis test alone reports it as overlapping whenever its offset lies strictly inside a sibling's range. Before this change a zero-width range could only arise at offset `0`, where the test happens to separate; with per-root domains a zero-extent member can sit anywhere. Emptiness is now checked before the axes, which restores the exactness the site's doc claims. Watched failing: perturbation 2 below.

**Fact — two new obligations replace what the equality rule supplied for free.**

- `IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain` — an output root may not store a value that varies along a parallel dimension its write does not iterate. Previously unreachable, because every write iterated every parallel dimension. Both readings of such a region are wrong (evaluate once per omitted point and several values reach one element; pick one point and the value is one nothing selected), so it fails closed. It sits beside `FreeReductionDimension` in one loop, and the arm order is load-bearing: testing domain membership first would rename every unreduced value, which perturbation 6 showed.
- `UnusedDomainDimension` now also fires for a parallel dimension nothing iterates and no reachable value varies along. Compaction retains every declared dimension, so an unmentioned one would sit in the canonical identity of a region whose meaning does not include it. This refuses one previously-buildable shape — a parallel dimension declared *after* the write that would have had to name it — which was the narrow accident by which the hole was already reachable.

**Measurement — no pinned identity moved; the gate is green on the whole workspace.** `encode_region` encodes an access as `(mode, tensor, domain, coordinates)` and encodes neither `bounds_proof` nor `ownership_proof`, so the partition proof forms remain outside canonical region identity — re-verified on this tree, as the ticket's identity-care instruction required. Every region that was buildable before encodes byte-identically, because the relaxation only *admits* domains that could not previously exist. The surveyed pins that could have moved and did not: the explain request qualifier `6dd42be71c6745fe` (`crates/tiler-compiler/src/explain.rs`) — its subject folds the index-realization *law registry* identity, never region bytes, and `IndexRegionDiagnostic` is never encoded into any identity; `STRICT_F32_REGION_IDENTITY_HEX` and its `v4` sibling; `FAMILY_ORDER_IDENTITY_FIXTURE`; the governed target-profile descriptor; the `tiler-build` artifact and cache-subject digests; and all seven `tiler-metal` goldens with their kernel and scheduled-region digests. `cargo nextest run --workspace` → 2750 passed, 0 failed, 7 skipped.

**Inference — no consumer read a write's domain, which is why the blast radius is prose rather than code.** Exactly one `.domain()` call site exists outside `crates/tiler-ir/src/index/`, and it is a test over *read* accesses (`crates/tiler-ir/tests/index_region.rs`); reproduce with `grep -rn --include='*.rs' '\.domain()' crates prototypes`. Every other consumer derives its iteration space from the region's parallel dimension set or from a semantic output shape and relies on the equality silently. The compiler's write-emission sites all pass the complete parallel vector, so subset-includes-equality leaves them unchanged. The schedule and kernel layers take `iteration_shape` from the semantic output shape rather than from an access, so a sub-domain region simply cannot be scheduled through the current verifiers — fail-closed, and their business when a producer exists.

**The oracle boundary, derived and filed.** `crates/tiler-reference` is `implementation/reference`, which this ticket does not hold, so the derivation is recorded here and the edit filed. `output_plans` walks one `ParallelWalk` over the whole parallel set and fires every root at every point of it. Under a subset domain there are two failures, and the existing doc predicts only the first: a strict-subset root recomputes one element at every point that agrees on its own dimensions, and `DuplicateWrite` refuses; but a **zero-extent** root zeroes the entire parallel product, so no point is walked, nothing is written, and `IncompleteWrite` blames `plan.roots.first()` — a root that is not the defective one, for a region that is not defective. Both are refusals by accident, and the second is also misattributed. The boundary decided: refuse explicitly now under a named `UnsupportedRegionFeature` ([`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md)), then support it by walking each root over its own domain ([`evaluate-write-roots-over-their-own-domains-in-the-oracle`](evaluate-write-roots-over-their-own-domains-in-the-oracle.md)), which the concatenate lowering needs if it is to have an independent oracle. Refusal first so no window exists in which the oracle silently mis-evaluates.

**Bounded limitation — symbolic partitions are unproved, and unchanged by this ticket.** Both partition mechanisms resolve extents through `determined`, so a symbolic boundary or member extent yields `PartitionVerdict::Enumerate` and then a `None` from `partition_walk_elements`, reporting every root `WriteOwnershipNotProven`. Offsets are literal-only for the same reason (`coordinate_offset_dimension` goes through `to_u64`). This predates the change and is fail-closed in both directions. It matters because the lowering's pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence lands here *if* `T` reaches the region as a `ShapeEnv` symbol; filed with its trigger log as [`prove-partition-coverage-for-symbolic-extents`](prove-partition-coverage-for-symbolic-extents.md). The literal analogue of that occurrence is exercised and passes.

**Watched failing — six perturbations, each run and each observed.** A check nobody watched fail is a check nobody has evidence for.

| # | Perturbation | Observed |
|---|---|---|
| 1 | Restore `domain_set != parallel_dimensions()` | 6 of the 8 new tests fail with `InvalidWriteDomain` at `write` — the relaxation is what admits them |
| 2 | Drop the empty-rectangle guard from the disjointness test | exactly `an_empty_partition_member_inside_a_sibling_range_is_still_disjoint` fails with `OutputPartitionRangesOverlap`; the edge-placed zero-extent test still passes, so the guard targets exactly the strictly-interior placement |
| 3 | Disable the `ValueDimensionOutsideWriteDomain` arm | `a_value_may_not_vary_along_a_dimension_its_write_omits` builds a region whose value has `free_dimensions: {1}` against a write with `domain: [0]` |
| 4 | Force parallel dimensions "used" | `a_parallel_dimension_no_root_iterates_is_refused_as_unused` builds, retaining `Parallel Extent(7)` in the identity with nothing referencing it |
| 5 | Drop the role gate in `prepare_access` | `a_write_may_not_iterate_a_reduction_dimension` admits the reduction dimension |
| 6 | Swap the two arms of the output-value loop | `unused_and_free_reduction_dimensions_fail_closed` fails — reversing the order renames every unreduced value, exactly as the site's comment claims |

**Commands run.** `cargo fmt --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-ir --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir --no-deps`; `cargo nextest run --workspace` (2750 passed); `cargo test --workspace --doc`; `tkt lint`; `git diff --check`; `tkt guard`; `make full`.

**Scope.** Every edit is under `crates/tiler-ir/**` (`implementation/ir`) or `tickets/**` (`project/tickets`); `tkt guard` against the named base reports no under-declaration. Nothing under `crates/tiler-reference/`, `docs/`, or any other crate was touched — the three corrections that land there are filed, alongside the acceptance node and the deferred symbolic gap, in the table below.

## Follow-up work filed

| Ticket | Why |
|---|---|
| [`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md) | `implementation/reference`. The oracle's admit-everything decision cites the premise this ticket removed, and its two failure paths are accidental — one of them misattributed to an innocent root. |
| [`evaluate-write-roots-over-their-own-domains-in-the-oracle`](evaluate-write-roots-over-their-own-domains-in-the-oracle.md) | `implementation/reference`. Supersedes that refusal with support, which is what gives the concatenate lowering an independent oracle. |
| [`accept-the-sub-domain-write-domain-surface`](accept-the-sub-domain-write-domain-surface.md) | The public surface is a draft: a widened `write` contract, a narrowed `InvalidWriteDomain`, and one new diagnostic variant. Parked at `awaiting-decision`; nothing releases on it. |
| [`prove-partition-coverage-for-symbolic-extents`](prove-partition-coverage-for-symbolic-extents.md) | `deferred` with a trigger log. The symbolic gap is pre-existing and fail-closed, and fires only when a consumer emits a symbolic region. |
| [`correct-the-write-domain-rule-in-the-indexing-corpus`](correct-the-write-domain-rule-in-the-indexing-corpus.md) | `research/indexing` and `contracts/navigation`. Two documents state the equality rule as a current fact in the passages their conclusions rest on. |

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** Outcome's claim that exactly one `.domain()` call site exists outside `crates/tiler-ir/src/index/`, and that it is a read-access test, was true of this ticket's blast radius and landing tree (`implementation/ir` only; no reference edits on this node). It is false on the live tree: `crates/tiler-reference/src/oracle.rs` `output_plans` does `DomainWalk::new(self.domain(access.domain())?)` — a production consumer of write-root domains, landed by [`evaluate-write-roots-over-their-own-domains-in-the-oracle`](evaluate-write-roots-over-their-own-domains-in-the-oracle.md) (`done`). Other `.domain()` sites also exist under `tiler-compiler` accuracy tests and `tiler-ir` refinement; the load-bearing live consumer for this ticket's construct is the oracle per-root walk.

**Correction — 2026-08-10.** The interim oracle refuse-then-support path under "The oracle boundary, derived and filed" completed. Both [`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md) and [`evaluate-write-roots-over-their-own-domains-in-the-oracle`](evaluate-write-roots-over-their-own-domains-in-the-oracle.md) are `done`. Current `output_plans` admits all partitioned shapes with per-root `DomainWalk`s and documents under "Which partitioned regions this admits" that there is no `UnsupportedRegionFeature` for this shape. Read that Outcome section as landing-time history, not as the live oracle boundary.
