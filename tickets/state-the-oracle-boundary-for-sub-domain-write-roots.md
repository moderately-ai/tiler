---
id: state-the-oracle-boundary-for-sub-domain-write-roots
title: State the oracle boundary for sub-domain write roots
status: todo
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
