---
id: scope-the-fused-multiply-add-semantic-family
title: Scope the fused multiply-add semantic family
status: deferred
priority: p2
dependencies: []
related: [admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, fma, deferred]
---
## User-visible outcome

A caller who needs one rounding across a multiply and an add can ask for it as an operation, and a target that cannot honour single rounding refuses the plan as **infeasible** rather than silently supplying two roundings.

## Why this is its own track rather than part of pointwise float algebra

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-07 is "atomic, and specifically not a composition": three operands of one identical resolved float type, one result, and **one** rounding, "which is the entire reason the family exists". [ADR 0015](../docs/decisions/0015-fma-vs-contraction.md) forbids lowering it to separate roundings. Its reference "must be exact-rational, because a host `f32` route double-rounds and is wrong here", and "a target that cannot honour single rounding makes the operation infeasible, never approximately feasible".

**Inference — that is three departures from the algebraic families in one row.** A different oracle obligation (an exact-rational route is *required*, not merely correct), a different physical precondition (a target property, not a scalar kernel operation), and a different failure class (hard feasibility rather than cost). [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) records the same split from the physical side: F-05, F-06, F-08, and F-09 sit in its *covered — direct scalar or map route* class while F-07 sits in *covered only under a stated precondition*, "a target that offers single rounding, which ADR 0015 makes non-negotiable". A row holding all four at one rung asserts a shared route they do not share.

**Fact — the corpus already has the adjacent work and it is not this.** [`admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract`](admit-the-fused-multiply-add-pointwise-body-under-a-contracting-contract.md) is `todo` and owns the *contract permission* path: whether a written multiply-then-add pointwise body may be contracted under a permitting numerical contract. That is ADR 0015's permission applied to a body the caller did not ask to fuse. This ticket is the opposite direction — an operation whose identity *is* the single rounding, which no contract may withdraw. The two must not be merged, and the measurement that ticket records is directly relevant evidence: the measured Apple row fuses a written multiply/add pair under `-ffp-contract=fast`, so a target's honourability answer here cannot be inferred from a flag.

## Activation trigger

A named workload requires a single-rounding multiply-add as an operation — not as a permitted fusion of two — **and** a target profile can be asked whether it honours single rounding. The second half is not rhetorical: [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids narrowing or substituting a caller's stated contract to make a target feasible, so admitting the family without the target question would leave no honest answer for a target that cannot supply it.

## What the work would be, when it starts

The key and its three-operand schema; the exact-rational oracle, with the argument for why a host `f32` route is wrong stated rather than cited; the target-honourability question and the `Unknown`/`Rejected` verdict shape a target that cannot answer must produce; the structured-kernel construct and the emission that must not decompose it; and the conformance corpus, which has to include at least one input where the single-rounded result and the two-rounded result differ, because a corpus that cannot tell them apart tests nothing about this family.

## Explicit non-goals

- The contraction *permission* over an unfused body, which the ticket named above owns.
- ADR 0015's other sense. A tensor contraction's per-contributor step is where the two senses meet, and the permission there is a separate field resolved separately.
- Any decomposition capability. This family has none, by ADR 0015.

## Closes when

A single-rounding key exists with an exact-rational oracle, a target can be asked and can refuse, the emission is proved not to decompose, and the corpus contains a discriminating input — or the family is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-18** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-07 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named workload requires a single-rounding multiply-add as an operation, and the one live multiply-add adjacency in the corpus is the contraction's per-contributor step, which declares ADR 0015's permission **forbidden** and whose emitted kernel is asserted to contain no fused multiply-add. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
