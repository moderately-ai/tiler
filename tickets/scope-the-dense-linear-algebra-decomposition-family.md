---
id: scope-the-dense-linear-algebra-decomposition-family
title: Scope the dense linear-algebra decomposition family
status: deferred
priority: p3
dependencies: []
related: [scope-the-complex-arithmetic-vertical, scope-the-data-dependent-extent-representation, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, linear-algebra, deferred]
---
## User-visible outcome

`RQ-OP-12` is answered: dense decompositions are either semantic operations with an admissible numerical contract, or consumer-owned by derivation — and the answer is recorded so that a vendor library call is never admitted as a fallback for something the optimizer's own rule forbids.

## Why this is deferred rather than open

**Fact — the family has no exact reference in the sense every other family has one.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-42 records "**No exact reference exists in the F-05 sense**: these are iterative algorithms whose results differ across implementations and whose failure modes are singular or non-convergent inputs", with determinism "per-algorithm and generally not portable", and result extents of a rank-revealing decomposition that "depend on values".

**Fact — the admissibility problem is an already-stated rule rather than a reluctance.** [Optimizer](../docs/compiler/optimizer.md) requires every implementation candidate to advertise "a machine-checkable numerical guarantee, realization/provider identity, and scoped evidence", admitted "only when that guarantee refines every effective operation contract", and checks numerical conformance before dominance "because accuracy is a hard semantic dimension, not a Pareto cost". A decomposition delegated to a vendor library publishes no accumulation order, so its evidence is `Unknown` and it is inadmissible rather than merely expensive.

**Fact — the physical profile reaches the same place from its own side and says why it matters.** [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-42 in *not covered, and the refusal is the guarantee*, and names it "the sharpest case... because it is the one where a 'fallback' is most tempting: a vendor decomposition would run, and would produce numbers, and the profile still must not use it... Admitting it as a baseline would make the baseline the one plan whose correctness nothing checks."

**Inference — the honest default is already derived.** `RQ-OP-12`'s own closure test requires a workload *and* a realization publishing a guarantee that refines an expressible contract; absent the second half the family is "consumer-owned by derivation rather than by choice". This ticket exists so that derivation is a recorded position with a reopening condition rather than an absence.

## Activation trigger

A named workload requires a decomposition **and** a realization publishes a numerical guarantee that refines a contract Tiler can express. Either half alone leaves the derived answer standing: a workload with no admissible realization gets a typed refusal, and an admissible realization with no workload admits nothing.

## What the work would be, when it starts

Answer `RQ-OP-12` against the named realization's published guarantee: state which expressible contract it refines and how, or state that it refines none and record the refusal as the family's answer. If it does refine one, the family then owes a named-algorithm high-precision oracle, the structural attributes, the value-dependent result extents (which reach [`scope-the-data-dependent-extent-representation`](scope-the-data-dependent-extent-representation.md) for the rank-revealing cases), and the singular and non-convergent failure modes as typed outcomes rather than as `NaN`.

## Explicit non-goals

- Admitting a vendor routine whose accumulation order is unpublished. The optimizer's rule already forbids it and this ticket must not carve an exception.
- Complex-operand decompositions ahead of the complex vertical.
- An ordering comparison over complex, which is on the taxonomy's intentionally-invalid list and must never become work here.

## Closes when

`RQ-OP-12` is answered with the realization's guarantee named and its refinement shown or refuted, and the taxonomy's `RQ-OP-12` row carries the answer — or the consumer-owned derivation is restated with the exact condition that would reopen it.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-35** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-42 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired, on either half.** No workload names a decomposition, and the corpus's one measured vendor-library candidate went the other way: `MPSMatrixMultiplication` was refuted against all twenty-two named topologies, which is a weaker operation than a decomposition and still published nothing admissible. Recheck: `rg -n 'MPSMatrixMultiplication' docs/research/scheduling/first-metal-contraction-realizations.md`.
- 2026-08-09 — **not fired, on either half.** The selected workloads name contractions and normalizations, not a factorization or decomposition, and no vendor or governed realization publishes a numerical guarantee refining a Tiler decomposition contract. A callable library routine without that guarantee remains an inadmissible fallback rather than evidence for activation.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -n 'MPSMatrixMultiplication' docs/research/scheduling/first-metal-contraction-realizations.md`, and run at this base it returns **1** line. A result other than the 1 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
