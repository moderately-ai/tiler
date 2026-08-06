---
id: state-the-oracle-boundary-for-sub-domain-write-roots
title: State the oracle boundary for sub-domain write roots
status: done
priority: p1
dependencies: [admit-sub-range-write-domains-for-unequal-partitions]
related: [lower-the-concatenate-occurrence-through-partitioned-writes, correct-the-reference-oracle-for-partitioned-output-writes]
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, oracle, indexing]
---
## User-visible outcome

The reference oracle states, rather than accidentally produces, what it does with a write root whose iteration domain is a strict subset of the region's parallel dimensions: such a region is refused under a named `UnsupportedRegionFeature` instead of failing somewhere downstream under a diagnostic about a different thing.

## Why this exists

**Fact — the oracle's admit-everything decision names the premise this ticket's dependency removed.** `output_plans` (`crates/tiler-reference/src/oracle.rs:2085`) carries a "Which partitioned regions this admits" doc whose argument is quoted here verbatim from `:2061-2065`: "`IndexRegionBuilder` refuses any write whose iteration domain is not exactly the region's parallel dimension set (`IndexBuildError::InvalidWriteDomain`), so every root of an output is visited at every parallel point this walk makes." The same premise is the first of the three facts the staged-evaluation span argument rests on (`:1458-1462`), and it is what licenses `:1481-1485`: "Since every root of an output iterates the whole parallel domain, splitting the points splits the pairs."

**Fact — [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md) removed that premise.** A write's domain is now any subset of the region's parallel dimensions. Both doc blocks are therefore false as written, and the decision they record was made under a constraint that no longer holds.

**Fact — the same doc already predicts the failure mode, and predicts it incompletely.** `:2073-2080` says a strict-subset root "could not name the missing dimensions in its coordinates … so walking the full parallel space would send it to one element repeatedly and `DuplicateWrite` would refuse it." That covers one shape of the relaxation. It does not cover the shape the concatenate lowering needs: a root over a **zero-extent** dimension makes the *whole* parallel product zero, so `ParallelWalk` visits no point at all, nothing is written, and `finish_output` (`:1987-2003`) reports `IncompleteWrite` naming `plan.roots.first()` — a root that is not the defective one, for a region that is not defective. The refusal is real but it is both accidental and misattributed.

**Inference — a refusal by accident is not a contract.** The deriving ticket's oracle-site note states the standard: the boundary is to be decided deliberately. Two `UnsupportedRegionFeature` variants exist for exactly this shape of decision already (`SymbolicDimensionExtent`, `SymbolicIndexDivisor`), so the vocabulary for stating it is present.

## What the work is

Add one `UnsupportedRegionFeature` variant — a write root whose domain is a strict subset of the region's parallel dimension set — and raise it from `output_plans`, before any buffer is planned and before any point is walked, so the refusal arrives at staging rather than mid-walk.

Rewrite the two doc blocks. `output_plans`'s "Which partitioned regions this admits" must state the new boundary and why it is a refusal rather than a fallthrough. `StagedIndexRegionEvaluation`'s first fact must stop asserting that a write's domain is the parallel dimension set and instead assert what the new refusal makes true: every root this evaluator *accepts* iterates the whole parallel domain, so the span argument's "splitting the points splits the pairs" step holds over the accepted set.

The `DuplicateWrite` and `IncompleteWrite` paths are not removed. They remain this evaluator's own joint obligation over the shared buffer, independent of the verifier's proof; what changes is that they stop being the only thing standing between a sub-domain root and a wrong answer.

## Explicit non-goals

- Evaluating sub-domain roots, which is [`evaluate-write-roots-over-their-own-domains-in-the-oracle`](evaluate-write-roots-over-their-own-domains-in-the-oracle.md) and supersedes this refusal.
- The IR-side relaxation, which is done and is this ticket's dependency.

## Closes when

A region with a strict-subset write root is refused at `stage` under the new named variant with no point walked; the zero-extent-root case is exercised specifically, because it is the one the existing doc's predicted `DuplicateWrite` does not reach; and both doc blocks state the current boundary.

## Graph maintenance

- `implementation/reference` alone: the variant, the refusal, and both doc blocks are in `crates/tiler-reference/src/oracle.rs`.
- Filed by [`admit-sub-range-write-domains-for-unequal-partitions`](admit-sub-range-write-domains-for-unequal-partitions.md), whose scopes are `implementation/ir` and cannot reach `crates/tiler-reference/`. The derivation above is that ticket's, recorded here rather than absorbed silently or left to the next reader of a doc that is now wrong.
- `project/tickets`: [`correct-the-reference-oracle-for-partitioned-output-writes`](correct-the-reference-oracle-for-partitioned-output-writes.md) carried two present-tense Facts asserting the builder refuses a strict-subset write domain. Both are now false. A **Superseded** paragraph was appended to its Outcome rather than the Facts rewritten, because what that ticket decided was correct under the constraint it decided it under, and rewriting the record would hide why the decision was reasonable.

## Outcome

**Fact — the refusal is stated at staging and names the offending root.** `UnsupportedRegionFeature::SubDomainWriteRoot { access }` is the one new variant, and `RegionEvaluation::admit_write_roots` raises it from the first line of `output_plans`. It compares each output root's `TensorAccessRef::domain()` against the region's parallel dimension set as whole sets in both directions, so a root naming a dimension outside that set refuses here too rather than reaching a walk that has no coordinate for it. Extents are deliberately not read: a strict-subset root is outside this profile whether or not every dimension is static, and a root refused for its domain should not be reported as a symbolic one.

**Inference — that site is the earliest sound one.** The check needs only the region, so it *could* sit anywhere after revalidation; what bounds it from below is soundness, and both obligations — no parallel point walked, no output buffer allocated — are already satisfied at `output_plans`, which is the last thing `stage` does and which walks nothing. What argues against moving it later is attribution rather than waste: sharing the planning loop would let an earlier boundary's retained-element budget answer first for a region no budget could make evaluable, so the refusal is a whole pass before the first buffer. Keeping it in `output_plans` also keeps the check beside the doc making the claim it narrows; a check whose reason lives in another function's doc is one the next reader deletes. The mode check moved into the same pass rather than being duplicated.

**Measurement — both failure shapes were observed refusing accidentally before the fix, on this tree, by disabling the refusal call.** With `self.admit_write_roots()?;` replaced by a comment (`cargo nextest run -p tiler-reference --test index_region_oracle`), 22 of 24 cases stayed green — every previously-evaluable region still evaluates — and exactly the two new cases went red:

- Strict subset, no zero extent (roots of extent 3 and 5 into an 8-element boundary): staging succeeded, reporting `StagedIndexRegionEvaluation { parallel_point_count: Some(15), evaluated_points: 0, … }` — the fifteen-point product of two dimensions no root iterates together. Reaching the walk needed a second perturbation, since the test's staging assertion fires first: with that assertion also removed, the whole-region path returned `DuplicateWrite { access: VerifiedTensorAccessId { index: 0 } }` — a claim that the region writes one element twice, of a region the verifier admits *because* its roots are disjoint. The stated refusal names that same access, with the true reason.
- Zero extent (`out` of `[2, 1]`; root 0 over both parallel dimensions writing `out[d0, d1]` with `d1` of extent 0; root 1 over `d0` alone writing `out[d0, 0]`): the parallel product is zero, so nothing was walked and the result was `IncompleteWrite { access: … index: 1 }`, which is `region.outputs()` position 0 — root 0, the sibling that iterates the *whole* domain and owns no element at all. The stated refusal names position 1, the short root. This is the misattribution the filing derived, reproduced and then removed.

**Measurement — the attribution assertion was also shown able to fail.** Naming one root is only a claim about attribution if the two roots are distinct accesses, so the zero-extent case asserts that first. Perturbing `output_accesses` to return position 0 twice made it fire (`left != right` on `index: 1` against itself), which is what stops the following equality from holding whichever root the refusal meant. A first draft of that case instead asserted the error was *not* `IncompleteWrite`, which could never fail once the equality above it passed; it was replaced rather than kept as documentation.

**Fact — the control holds the shape fixed and varies only the property under test.** The same builder with one root covering the whole boundary (`unequal_partition_region(…, 8, &[(8, 0)])`) has a sole domain that *is* the parallel dimension set, and evaluates to eight elements. `partitioned_output_roots_fill_one_joined_tensor` and `an_empty_partitioned_boundary_is_one_output_tensor` — full-domain partitions over one boundary — are untouched and green.

**Fact — no pinned identity moved.** `UnsupportedRegionFeature` enters no canonical encoding: `crates/tiler-reference/src/identity.rs` encodes providers and signatures only, and the crate has no `trybuild` golden or fixture. Every `.stderr` golden in the tree belongs to `crates/tiler` or `crates/tiler-ir`, neither of which this branch touches. Checked with `grep -rn "UnsupportedRegionFeature" --include='*.rs' --include='*.md' .`: the only non-ticket sites are `oracle.rs`, its integration test, and the `lib.rs` re-export.

**Fact — the public surface delta is one additive variant, labelled a draft.** `UnsupportedRegionFeature` is `#[non_exhaustive]`, so `SubDomainWriteRoot { access: VerifiedTensorAccessId }` is additive and no in-tree consumer matches the enum. Carrying the access departs from the convention of the enum's fieldless siblings; it is what makes the refusal attributable, which is the whole point of preferring it to the accidental one. Pending Tom's boundary review with the rest of this crate's draft surface.

**Fact — the support is a separate ticket and this refusal is explicitly temporary.** Both the variant's doc and `output_plans`'s admit-everything block now cite `evaluate-write-roots-over-their-own-domains-in-the-oracle` as what supersedes them. `StagedIndexRegionEvaluation`'s first span-argument fact no longer claims the builder guarantees the set equality; it states that this oracle's own refusal does, and the "splitting the points splits the pairs" step is restated over the roots that reach the loop.
