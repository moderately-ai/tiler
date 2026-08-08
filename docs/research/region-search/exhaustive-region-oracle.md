---
schema: "tiler-doc/v1"
id: "tiler.research.region-search.exhaustive-region-oracle"
kind: "research"
title: "Exhaustive fusion-region oracle"
topics: ["fusion", "search", "optimizer"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "partial"
evidence_classes: ["exhaustive-finite", "executable-model"]
informs: ["tiler.contract.optimizer", "tiler.contract.fusion-and-scheduling"]
ticket: "region-search-oracle"
---

# Exhaustive fusion-region oracle

## Purpose

Tiler needs a small executable definition of legal region formation before it
chooses a production search algorithm. The oracle exhaustively enumerates tiny
DAGs so heuristics can be tested against known alternatives and rejection
reasons. It is not a proposal to exhaustively optimize real programs.

## Precedents

Burn's `OperationFuser` at inspected revision
[`e5467f02`](https://github.com/tracel-ai/burn/blob/e5467f02c3cf88eb5d709f190c170005ce26038d/crates/burn-fusion/src/backend.rs)
(workspace version 0.22.0-pre.1) admits operations
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

**Correction — 2026-08-04, by the general-pipeline audit; this record's status moved from `spike-only` to `partial`.** The spike above is no longer the only executable witness, and it is no longer the strongest one. Independent exhaustive oracles now live beside the production search in `crates/tiler-compiler`: `region.rs` carries one over every nonempty operation subset that the enumerator must agree with set-for-set without budget pressure, and `cover.rs` carries two — an exact-partition oracle and a duplicating-cover oracle — over programs of four and five operations with up to seventeen candidate regions, together with a test that the comparison itself rejects a perturbed admitted set. That last one is what makes the agreement worth citing: a comparison nothing has shown can say no is not evidence. The record's decided behaviour is therefore production code and not only a model, which is what the status change records; it is `partial` rather than `implemented` because two of this record's own items are not discharged — duplication is a stated legality contract that the compile path does not enumerate under, and the cost comparison in step 6 of the protocol below is not run.

**One correction the oracles produced, recorded because it generalizes past this record.** An anchored partition search that admits a candidate only when it covers the branch's minimum uncovered operation can never choose a region every one of whose operations is already covered — and such a region is not idle, because it is one of the two ways to spell a partial duplication. The exhaustive oracle named the missing covers, and the repair is to enumerate the anchored base plus every augmentation by such a region, which is complete because running the anchor rule over any legal cover selects a base and leaves exactly that remainder. This is the class of defect the oracle exists to find, found by it.

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

**Correction — 2026-08-04: the frontier bound above never became real, and the contract that carried it forward has since withdrawn it.** Every other bound in this list is a `DeterministicBudgets` field, but "maximum 8 nondominated physical implementations per logical region" is not: the implementation frontier retains its non-dominated set as a pure Pareto filter with no count bound, and no corresponding field exists. [The optimizer contract](../../compiler/optimizer.md#bounded-hierarchical-search) states that as a fact and records that whether the frontier owes a retention budget at all is [an open decision](../../../tickets/decide-whether-the-implementation-frontier-owes-a-retention-budget.md). The line is kept here because this record is the source the contract's list was derived from, and deleting it would hide that the proposal was made; read it as a proposal this record made and the implementation declined, not as a budget a reader should look for.

**Correction — 2026-08-08: the three region-shape bounds in that list *did* become real, and all three have since been re-sized — one of them narrower. That is the opposite relationship from the frontier bound's, and the two notes must not be read together.** "maximum 32 semantic occurrences per candidate" and "maximum 8 boundary outputs and 64 live boundary/internal values" name `region_members`, `region_boundary_outputs`, and `region_live_values`, and `DeterministicBudgets::governed` carried exactly `32`, `8`, and `64` for them from `bc371d6d` until `4eb78100`. **Fact —** it now carries `region_members: 62`, `region_boundary_outputs: 3`, and `region_live_values: 80`, read from `pub(crate) const fn governed` in `crates/tiler-compiler/src/request.rs`. `region_boundary_outputs` moved *narrower*, which a reader assuming a budget only widens will misread. The two lines are retained above rather than rewritten because this record is the source the [optimizer contract](../../compiler/optimizer.md#bounded-hierarchical-search)'s list was derived from and deleting them would hide that the proposal was made — so a grep finding "32 semantic occurrences" in this file has found a retired proposal, not a live bound.

**What replaced them, stated at its own evidence rung.** [`derive-the-region-shape-budgets-from-the-declaration`](../../../tickets/derive-the-region-shape-budgets-from-the-declaration.md) replaced the three constants on Tom's 2026-08-07 decision, on the ground that a region is a subset of the program it covers: `region_members` sized against `semantic_operations`, `region_live_values` against `semantic_values`, and `region_boundary_outputs` against the declared output count. **That derivation is authoring-side, not run-time**, and the distinction is the whole maturity claim here: `governed` is still a nullary `const fn` returning integer literals, nothing is computed from a request's declaration while it compiles, and the three literals track the governed profile's declaration only for as long as somebody re-derives them when that declaration moves. The sizing is against the decoder layer's C1 decode row — sixty-two occurrences over eighty values, three ordered named outputs — which that function's own prose records.

**The list's other two budget numbers did not move, and separating them is the point.** `region_candidates_per_seed` is still `32` and `region_expansions` still `10,000`. So of the six numbers above, three were re-sized, two stand unchanged, and one — the frontier bound — never became a field at all: the list is neither uniformly stale nor uniformly current, and no single verdict covers it.

**Measurement, bounded to one family.** While `region_members` was `32`, the refusal it caused was observed rather than inferred: [the identity-growth ladder](../../../spikes/program-planning/identity-growth/README.md) recorded 33..=62 occurrences of a unary `f32` multiply chain refusing `BudgetExhausted` on `region_members` — that family's recognized partition is its whole program, so nothing smaller was implementable — and the same ladder now compiles all sixty-one points. That bounds one program family under one contract and one target profile; it is not a statement that no program is refused by a region-shape bound, which is exactly what these three bounds still exist to do.

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
