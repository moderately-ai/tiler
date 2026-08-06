---
id: compose-a-declared-reduction-topology-into-a-semantic-program-evaluation
title: Compose a declared reduction topology into a semantic program evaluation
status: in-progress
priority: p2
dependencies: []
related: [decide-how-a-pinned-pointwise-grouping-becomes-evaluable, derive-the-oracle-for-a-permitted-divergence-candidate, enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, accept-the-realization-witness-surface]
scopes: [research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, reference, conformance]
claimed_from: todo
assignee: agent-topology-oracle
lease_expires_at: 1786050116
---
## User-visible outcome

A derivation of how one reference evaluation answers for a program that spends reassociation at *both* the semantic rewrite and a physical reduction split — so that a plan carrying a reassociated pointwise chain feeding a partitioned fold has one expected value rather than two half-answers.

## Why this exists

**Fact — the two witnesses are answered by two evaluators that do not compose today.** [The permitted-divergence oracle](../docs/research/reference/permitted-divergence-oracle.md) establishes O5: evaluate the program under the plan's own realization witness, compared bitwise. [The freedom-sites enumeration](../docs/research/reference/plan-freedom-sites.md) Part 2 then splits the witness across sites answered by different objects — sites 4.1 through 4.4 (the reduction topologies) by the declared-order evaluators `strict_partial_sums_under` (`crates/tiler-reference/src/evaluate.rs:603`) and `strict_partitioned_sum_under` (`:766`), and site 4.5 (the pointwise chain) by the semantic evaluator over the selected candidate's program, which [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) derived as the surviving design.

**Fact — the semantic evaluator cannot be told a topology.** `ReferenceEvaluator::evaluate` (`crates/tiler-reference/src/evaluate.rs:178`) dispatches each operation through the frozen reference registry, and a `tiler::strict-serial-sum-f32@1` occurrence therefore resolves to the registered strict left fold. Its signature takes a program and input bindings; nothing in it names a `ReductionTopology`, a `ContributorPartition`, or an accumulation width. Exact check: `grep -n "ReductionTopology\|ContributorPartition" crates/tiler-reference/src/evaluate.rs` returns nothing.

**Fact — the population is non-empty and reachable.** A prologue expression feeding a fold is the ordinary shape of a reduced-elementwise program: `pointwise_region` builds a prologue region from `NormalizedSerialSum::prologue` (`crates/tiler-compiler/src/physical.rs:829-832`), and the multi-pass and cooperative topologies are constructed at `physical.rs:2060` and `:1896`. Under `REASSOCIATE_F32` or `RELAXED_F32` the same contract that admits the semantic rewrite admits the reduction split, so one plan can spend both freedoms.

**Inference — so an oracle assembled from the two objects separately answers for neither.** Evaluating the selected semantic program alone gets the fold's grouping wrong whenever the plan split it; evaluating the declared partition alone has no prologue. The composition is the question, and the shapes it could take — a topology-parameterized evaluation request through the registry, an evaluation staged at the materialization boundary the cover already names, or a witness-driven evaluation of the whole plan — are candidates to eliminate rather than one obvious answer.

## What this ticket must produce

The elimination, run against correctness first: which object answers for a program spending reassociation at both layers, stated so a reader can refute it, with the evaluated population named and every unsupported case an explicit refusal rather than a silent strict reading. If it resolves to a public surface, that surface is drafted and parked for Tom under ADR 0075, never self-accepted.

## Explicit non-goals

Implementing an evaluator or changing `crates/`; re-deciding the pointwise fork, which is settled; accepting the realization witness surface, which is [`accept-the-realization-witness-surface`](accept-the-realization-witness-surface.md).

## Closes when

The composition question has a derived answer with its evaluated population and its refusals named, or it is deferred with the evidence that would close it and a trigger stated.

## Graph maintenance

Filed by [`decide-how-a-pinned-pointwise-grouping-becomes-evaluable`](decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md) as the bounded residue its surviving design does not reach. Not a blocker for that design: a pure pointwise region carries `ReductionTopology::None`, so site 4.5 is answered without this.
