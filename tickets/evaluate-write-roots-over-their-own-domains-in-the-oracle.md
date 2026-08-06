---
id: evaluate-write-roots-over-their-own-domains-in-the-oracle
title: Evaluate write roots over their own domains in the oracle
status: review
priority: p1
dependencies: [state-the-oracle-boundary-for-sub-domain-write-roots]
related: [lower-the-concatenate-occurrence-through-partitioned-writes, admit-sub-range-write-domains-for-unequal-partitions, decide-the-index-region-oracle-route-past-its-step-budget]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, oracle, indexing]
claimed_from: todo
assignee: agent-root-domains
lease_expires_at: 1785997200
---
## User-visible outcome

The reference oracle evaluates a partitioned output whose roots iterate different sub-domains, so a concatenation of unequally sized operands — including one that is empty — has an independent correctness oracle rather than only the verifier's own proof.

## Why this exists

**Fact — the evaluator has one walk and every root fires at every point of it.** `stage` builds one `ParallelWalk` from `parallel_domain()` (`crates/tiler-reference/src/oracle.rs:1424`, `:2006-2015`), which is the region's whole parallel dimension set read from `dimensions()` and never from an access. `evaluate_point` (`:2145-2183`) then seeds the frame with that point and evaluates **every** root of **every** plan at it.

**Fact — a sub-domain root has no correct behaviour under that walk.** Its coordinates cannot name the dimensions its domain omits (`IndexBuildError::CoordinateOutsideAccessDomain`), so at every full parallel point that agrees on the root's own dimensions it computes the same element — a duplicate — and a root over a zero-extent dimension zeroes the whole parallel product so that no root fires at all.

**Fact — the IR admits exactly those regions now.** [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) admits a write domain that is any subset of the parallel dimensions, and the construct it admits gives each root its own iteration space with the region's parallel set as their union.

**Inference — the refusal this ticket's dependency states is the honest interim, not the destination.** [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) emits precisely a sub-domain-rooted region at its pinned `[8, 0, 128]`-with-`[8, T, 128]` occurrence; while the oracle refuses that shape, the lowering has the verifier's proof and no independent check of it.

## What the work is

Walk each root over **its own** domain instead of walking one shared parallel space, and restate the span argument over the unit that then makes sense.

The unit question is the substance and must be decided rather than assumed. The current public surface counts and spans *parallel points* (`parallel_point_count`, `:1548-1558`; `evaluate_points`; `evaluated_points`), and the span-safety argument at `:1453-1486` says a partition of the parallel points is a partition of each output's elements — which is true only while every root iterates all of them. Under per-root domains the entity in bijection with an output's elements is the (root, root-domain point) pair, which the existing doc already names at `:1475-1477` as the general case. Decide whether the public unit becomes that pair, and say what a caller's existing division by `parallel_point_count` then means.

Keep the per-element `DuplicateWrite` and `IncompleteWrite` checks exactly where they are, over one buffer per output boundary. They are the oracle's own joint obligation, independent of the verifier's ownership proof, and they are what makes this an oracle rather than a second reading of the proof. `IncompleteWrite`'s attribution to `plan.roots.first()` (`:1987-1993`) needs revisiting under unequal roots, because the first root is no longer a meaningful blame target for a gap.

## Explicit non-goals

- The refusal this supersedes, which is the dependency and lands first so no window exists in which the oracle silently mis-evaluates.
- Symbolic extents, which this evaluator already refuses under `SymbolicDimensionExtent` and which are a separate question on the IR side.

## Closes when

A region whose roots partition one output into unequally sized contiguous pieces — including a zero-extent member — evaluates to the correct tensor; the superseded `UnsupportedRegionFeature` refusal is removed rather than left unreachable; a deliberate perturbation of one root's offset is shown refusing under `DuplicateWrite` or `IncompleteWrite`; and the staged-span argument is restated over the unit actually walked, with a test that spans of that unit compose to the same result as one span.

## Graph maintenance

- `implementation/reference` alone.
- Filed by [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md), which could not reach `crates/tiler-reference/`.
- Related rather than a declared dependency of the concatenate lowering: the lowering plainly wants an oracle for the region it emits, but whether its own Closes-when routes through `IndexRegionEvaluator` was not verified when this was filed, and a hard edge asserted on an unread path would block that ticket on a claim nobody checked.
- `project/tickets`: three prior records carried statements this work made false, and each got a **Superseded** paragraph rather than a rewrite. [`decide-the-index-region-oracle-route-past-its-step-budget`](decide-the-index-region-oracle-route-past-its-step-budget.md) listed the staged public surface under its old names and rested its span argument on the write-domain equality rule; [`correct-the-reference-oracle-for-partitioned-output-writes`](correct-the-reference-oracle-for-partitioned-output-writes.md) already carried one supersession and now records that the interim ended; [`state-the-oracle-boundary-for-sub-domain-write-roots`](state-the-oracle-boundary-for-sub-domain-write-roots.md) records that the refusal it landed is removed and that its two measurements were reproduced on this tree.

## Outcome

**Fact — each root now walks its own domain, and the unit the whole staged surface is stated in changed with it.** `OutputRoot` carries a `DomainWalk` built from `TensorAccessRef::domain()` — that root's own dimensions and extents — and `RegionEvaluation::evaluate_root_point` seeds a fresh `Frame` from that root's point alone before filling the boundary's shared buffer. `RootCursor` walks the flattened (plan, root, domain-point) sequence and is kept settled, so an exhausted root is skipped rather than stalling the walk. `UnsupportedRegionFeature::SubDomainWriteRoot`, `RegionEvaluation::admit_write_roots`, and `parallel_domain` are gone; the `AccessMode::Write` floor moved back into `output_plans`'s loop.

**Inference — the joint checks' populations stay honest, and the argument is the bijection rather than the loop.** `DuplicateWrite` and `IncompleteWrite` are unmoved: one buffer per output *boundary*, one slot per element, checked per write and at `finish`. What makes them mean the right thing under unequal roots is that the entity in bijection with an output's elements is the (root, point-of-that-root's-own-domain) pair. Each of the three `WriteOwnershipProofView` forms quantifies over the root's **own** domain and never over the region's parallel set — which is why the write-domain relaxation cost the argument nothing — and the `PartitionMember` joint obligation makes the roots' images pairwise disjoint and exactly covering. So a strict-subset root fires once per point of its own domain and writes its rectangle exactly once, leaving `DuplicateWrite` to report only a genuine collision between two roots' images; a zero-extent root contributes no pair and owns no element, leaving `IncompleteWrite` to report only a genuine joint gap. Neither can now fire for a region the verifier admits, which is what a floor beneath a proof should look like.

**Fact — the per-root frame is well defined, and it is verification that makes it so.** Seeding a root's frame from its own domain alone would be unsound if the stored value could vary along a dimension the write omits. It cannot: `IndexBuildError::CoordinateOutsideAccessDomain` bounds the coordinates, and `IndexRegionDiagnostic::ValueDimensionOutsideWriteDomain` (`crates/tiler-ir/src/index/builder/proof.rs:79`) refuses a root whose stored value's `free_dimensions` leave its write domain. `free_dimensions` is transitive — a read's is its access domain (`builder.rs:1258`), an apply's is the union of its operands' plus its evaluation dimensions (`:1447-1458`), a reduce's is that union minus the bound dimensions (`:1706-1713`) — so no expression a root reaches can ask the frame for a dimension it lacks.

**Inference — dropping `parallel_domain` loses no refusal.** A parallel dimension a verified region declares is in some write root's domain, so its extent is still read — and still refused when symbolic — where that root's walk is built. Derivation: `proof.rs:101-124` refuses an unused parallel dimension, where "used" means appearing in a reachable access's domain or a reachable value's free dimensions; a reachable value feeds some output root transitively, and parallel dimensions are never bound by a reduction, so they survive into that root's stored value's free dimensions and `ValueDimensionOutsideWriteDomain` puts them inside its write domain. The one-line check: `grep -n "used_parallel" crates/tiler-ir/src/index/builder/proof.rs`.

**Fact — the staged path supports sub-domain roots; there is no split.** The unstaged path *is* the staged path walked in one span, so keeping a refusal on one side was never available without keeping it on both. The three facts the span argument rests on all survive, and the third does extra work: because no value can read a boundary the region writes (`IndexBuildError::ReadFromOutput`), the walk may visit the pairs root-major instead of point-major. A span may therefore cross from one root to the next and from one boundary to the next, and `spans_of_root_points_compose_across_a_root_boundary` watches a span of five ending two points inside the second root.

**Fact — the public unit is now the root point, and the old name would have been a lie rather than a rename.** `parallel_point_count` returned the product of all parallel extents. That number is fifteen for a boundary of eight elements, and **zero** for a legal region with a zero-extent member whose siblings have every point still to walk, so it cannot size a span. `root_point_count() -> Option<u64>` sums each root's own product — one per retained element — and `evaluate_points`/`evaluated_points` became `evaluate_root_points`/`evaluated_root_points` so the count and the loop name one unit. A caller's old division by `parallel_point_count` has no meaning to preserve; it must divide into `root_point_count`. `EmptyStagedSpan`, `IncompleteStagedWalk { evaluated }`, `is_exhausted`, and `finish` keep their names and shapes with their unit restated.

**Fact — the public surface delta, labelled a draft, not self-accepted.** Removed: `UnsupportedRegionFeature::SubDomainWriteRoot { access }`, one variant of a `#[non_exhaustive]` enum landed an hour earlier by this ticket's dependency and explicitly marked as superseded by this one. Renamed on `StagedIndexRegionEvaluation`: `parallel_point_count` → `root_point_count`, `evaluated_points` → `evaluated_root_points`, `evaluate_points` → `evaluate_root_points`. The only in-tree consumer is `crates/tiler-reference/tests/contraction_profile_cells.rs`, updated in the same change; `grep -rn "parallel_point_count\|evaluate_points\|evaluated_points\|SubDomainWriteRoot" crates docs spikes prototypes` returns nothing outside this branch's own edits. Pending Tom's boundary review with the rest of this crate's draft surface.

**Measurement — the unequal partition evaluates to hand-derived elements.** `unequal_partition_region` now copies each member's own `[extent]`-shaped input rather than storing a constant, so the expected tensor is a derivation and not a repetition. Roots of extent 3 at offset 0 and extent 5 at offset 3 into an 8-element boundary, with inputs `[1, 2, 3]` and `[10, 20, 30, 40, 50]`, evaluate to one tensor of shape `[8]` with values `[1, 2, 3, 10, 20, 30, 40, 50]`, and `root_point_count()` is `Some(8)`. The control — one root of extent 8, whose sole domain *is* the parallel dimension set — is unchanged.

**Measurement — the zero-extent member contributes nothing and misattributes nothing.** `out` of `[2, 1]`; root 0 over both parallel dimensions writing `out[d0, d1]` with `d1` of extent 0 and storing `7.0`; root 1 over `d0` alone writing `out[d0, 0]` and storing `1.0`. `root_point_count()` is `Some(2)` — the empty root contributes none, the full one both — and the result is `[1.0, 1.0]`, which only root 1's own-domain walk can produce. The two constants are asserted distinct and the two write accesses are asserted distinct, so the value assertion is a claim about *which* root ran rather than one that would hold either way.

**Measurement — every new path was watched failing, by the mirrored perturbation.** With `DomainWalk::new(self.domain(access.domain())?)` replaced by the region's whole parallel dimension set (`cargo nextest run -p tiler-reference --test index_region_oracle --no-fail-fast`), **22 of 25 cases stayed green and exactly the three new ones went red** — so every previously-evaluable region is unaffected by the property under test:

- `unequally_partitioned_roots_each_walk_their_own_domain`: `root_point_count` reported `Some(30)` against `Some(8)` — two roots each billed the fifteen-point product of dimensions neither iterates together. With that assertion also removed the walk returned `DuplicateWrite { access: VerifiedTensorAccessId { owner: VerifiedRegionOwner(1), index: 2 } }`, which is the accidental refusal `state-the-oracle-boundary-for-sub-domain-write-roots` documented.
- `a_zero_extent_write_root_contributes_nothing_and_empties_no_sibling`: `root_point_count` reported `Some(0)` against `Some(2)`; with that assertion removed, `IncompleteWrite { access: … index: 1 }`, which is `region.outputs()` position 0 — the sibling that iterates the whole domain and owns no element. Both shapes reproduced exactly as that ticket recorded them.
- `spans_of_root_points_compose_across_a_root_boundary`: `DuplicateWrite { … index: 2 }` at the whole-region baseline, before any span was taken.

Both perturbations were reverted; the final tree carries neither (`grep -n "DomainDimensionRef" crates/tiler-reference/src/oracle.rs` → no matches).

**Measurement — previously-evaluable regions are bit-identical, including at scale.** `cargo nextest run -p tiler-reference` → 285 passed, 2 skipped, and the whole workspace gate below. The digest tests are the bit-level half: `cargo nextest run -p tiler-reference --run-ignored only --no-capture -E 'binary(contraction_profile_cells)'` reproduces all six retained `direct` SHA-256 digests, and `the_staged_index_region_oracle_reaches_the_vocabulary_cell` walks `w_vocab_slice`'s region as **8,192 root points in 16 spans of 512** to the digest an Apple M4 Max produced — one span over the whole walk still refused at `EvaluationSteps { limit: 16_777_216, actual: 16_777_217 }`. Apple M4 Max, 2026-08-06, dev profile, `--no-capture`: refusal 8,512 ms (507 ns/step) and the staged walk 40,610 ms, against the 8,660 ms / 516 ns and 55.6 s recorded for the same test on 2026-08-01 — a loaded coordination host, so these are the same result and not a performance claim.

**Fact — no pinned identity moved.** No IR change, so no region canonical identity moves. `UnsupportedRegionFeature` enters no canonical encoding: `crates/tiler-reference/src/identity.rs` encodes providers and signatures only (`grep -n "UnsupportedRegionFeature\|oracle" crates/tiler-reference/src/identity.rs` → no matches), and the crate has no `trybuild` golden or fixture — every `.stderr` in the tree belongs to `crates/tiler` or `crates/tiler-ir` (`find . -name '*.stderr' -not -path './target/*'`), neither of which this branch touches.

**Fact — one stale citation found and left, because it is out of scope and predates this work.** `docs/research/reference/permitted-divergence-oracle.md:58` cites `IndexRegionEvaluator::under` at `oracle.rs:1316` and `from_realization` at `:1304`; on this ticket's base commit they were already at `:1359` and `:1347`, and this change moves them to `:1353` and `:1341`. That file is `research/reference`, which this ticket does not hold. Recorded rather than absorbed or silently corrected.
