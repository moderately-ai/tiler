---
schema: "tiler-doc/v1"
id: "tiler.research.region-search.exhaustive-region-oracle"
kind: "research"
title: "Exhaustive fusion-region oracle"
topics: ["fusion", "search", "optimizer"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["exhaustive-finite", "executable-model"]
informs: ["tiler.contract.optimizer", "tiler.contract.fusion-and-scheduling"]
reproduced_by: ["tiler.spike.region-search"]
ticket: "region-search-oracle"
---

# Exhaustive fusion-region oracle

## Purpose

Tiler needs a small executable definition of legal region formation before it
chooses a production search algorithm. The oracle exhaustively enumerates tiny
DAGs so heuristics can be tested against known alternatives and rejection
reasons. It is not a proposal to exhaustively optimize real programs.

## Precedents

Burn's `OperationFuser` in `crates/burn-fusion/src/backend.rs` admits operations
incrementally, exposes open/closed state, reports whether a candidate is ready,
and scores ready implementations. Its search tests retain open alternatives
instead of committing to the first ready prefix. This is strong precedent for
candidate-local feasibility and continued unfused coverage. Burn operates on a
runtime stream and mostly prefix-shaped regions, so it is not an oracle for an
arbitrary compile-time DAG.

DataFusion's physical optimizer separates executable-plan validity from rules
that choose implementations or enforce properties. The analogy supports
separating Tiler region legality from physical alternatives and cost, but SQL
trees do not expose the same shared-producer, recomputation, multi-output, or
kernel-convexity choices.

XLA's documented GPU scheduling cost model mixes analytical fusion estimates
with measured performance tables. This supports retaining multiple legal
implementations for later costing; a cost estimate must not establish semantic
or physical legality.

## `RegionCandidate`

A logical region candidate is a canonical, nonempty set of semantic operations
plus derived boundary inputs and ordered boundary outputs. It does not yet
contain a schedule or claim that one kernel is feasible.

Initial candidate legality requires:

1. Every operation is in the verified pure semantic graph.
2. Every internal data edge permits fusion under the resolved numerical and
   materialization contracts.
3. The induced dataflow is connected.
4. The region is convex: a path between two included operations cannot leave
   and later re-enter the region. Otherwise contracting the region would hide
   a required interleaving or create a cyclic region graph.
5. Ordered graph outputs and values used outside the region become explicit
   boundary outputs; external operands become boundary inputs.
6. Region-formation budgets may stop growth but never make an illegal region
   legal or remove singleton/unfused coverage.

Convexity applies to one occurrence of each operation. Explicit duplication
creates distinct logical occurrences in a plan alternative; it does not waive
convexity by silently pretending an outside producer is inside.

## `ImplementationFrontier`

For one semantic region, the implementation frontier is the nondominated set
of verified implementation alternatives after:

```text
semantic/access legality
-> schedule construction and intrinsic verification
-> target feasibility: proven or safely deferred
-> applicability guards and boundary guarantees
-> cost/resource dominance within comparable applicability domains
```

An entry identifies its covered semantic occurrences, scheduled region or
typed opaque implementation, numerical contract, boundary requirements and
guarantees, target predicates, deferred queries, exact/proven resources,
estimates, and explanation provenance.

Dominance is only valid when semantic coverage, result contract, and
applicability domain are comparable. A slower unguarded implementation does not
dominate—or become dominated by—a faster alignment-specialized variant merely
because their scalar cost estimates differ. At least one complete unfused or
external-fallback route must remain for every admitted input domain.

## Program alternatives

The oracle distinguishes:

- **partition:** every semantic operation occurs exactly once across regions;
- **materialization:** an internal value becomes a producer-region output and
  consumer-region input;
- **multi-output fusion:** one region exports several ordered values;
- **duplication:** an explicitly duplicable pure producer occurs in more than
  one region, with cost and numerical obligations charged per occurrence;
- **overlapping candidates:** alternatives may share operations, but one chosen
  partition cannot overlap except through explicit duplication.

Duplication is never implied by a fan-out edge. It requires operation
capability, numerical equivalence, bounded expansion, and a cost comparison
against materialization. Random, effectful, expensive opaque, or identity-
sensitive operations are not duplicable.

## Executable witnesses

[`exhaustive_oracle.py`](../../../spikes/region-search/exhaustive_oracle.py)
enumerates all nonempty subsets and all exact covers for tiny graphs. It checks:

- a three-operation chain and its four partitions;
- rejection of a non-convex diamond subset;
- shared-producer materialization, multi-output fusion, and explicit producer
  duplication;
- a numerically incompatible edge retaining exactly one unfused partition;
- implementation-frontier dominance without discarding a guarded variant.

Run:

```sh
python3 spikes/region-search/exhaustive_oracle.py
```

## First heuristic bounds

The initial production search should be bounded and deterministic:

- seed from every operation in stable topological order;
- grow only across legal producer/consumer edges while preserving convexity;
- retain singleton coverage unconditionally;
- maximum 32 semantic occurrences per candidate;
- maximum 8 boundary outputs and 64 live boundary/internal values;
- maximum 32 logical candidates per seed after structural deduplication;
- maximum 8 nondominated physical implementations per logical region;
- maximum 10,000 candidate-expansion attempts per compilation request;
- duplication disabled initially except in oracle tests;
- deterministic tie-breaking by canonical candidate/implementation identity.

These numbers are provisional safety budgets, not performance conclusions.
Hitting one produces an explain event and conservatively stops that growth
path. Calibration may change defaults without changing correctness or IR.

## Required comparison protocol

For every curated graph with at most eight operations:

1. enumerate the complete oracle candidates and partitions;
2. run the production heuristic under a fixed budget;
3. verify every heuristic candidate is oracle-legal;
4. verify singleton/unfused coverage remains complete;
5. report legal oracle alternatives missed because of bounds;
6. compare selected cost against the best oracle plan under the same cost
   model, without treating agreement as proof that the cost model is accurate.

This separates three failures: illegal enumeration, search-quality loss, and
cost-model error.
