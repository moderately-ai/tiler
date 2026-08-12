---
id: research-an-explicit-seeded-fused-contraction-operation
title: Research an explicit seeded fused contraction operation
status: deferred
priority: p3
dependencies: [qualify-the-simdgroup-matrix-contraction-realization]
related: [pin-the-strict-contraction-simdgroup-refusal]
scopes: [research/apple-targets, contracts/numerics, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, numerics, contraction, semantic-operation, deferred]
---
## Outcome

If a real caller needs seeded fused contraction semantics and sufficient evidence can define them portably, specify a distinct operation identity rather than weakening `tiler::strict-tensor-contraction-f32@1`. The operation must state its seed, contributor order or result set, intermediate precision, rounding, NaN boundaries, validation, and reference behavior before any target realization is considered.

## Boundary

The retained Apple9 record is empirical evidence over eight cases and twenty-two named candidate topologies. It is not a normative guarantee of `simdgroup_multiply_accumulate`'s unpublished contributor order, intermediate precision, or per-combine NaN behavior. A target-specific opaque “whatever Apple does” operation has no portable reference contract and is not an acceptable substitute.

If this trigger fires, begin with a source-first census of actual caller semantics and primary vendor/specification evidence. Decide the semantic `OpKey`, reference and validation contract before target facts, lowering, artifact fields, or performance qualification. Reuse existing reached semantic/schedule identity where it is sufficient; widen fixed artifact records only for a separately justified delivered physical fact.

## Trigger

Reconsider only when both conditions hold:

1. a concrete consumer requests explicit `+0.0`-seeded fused contraction semantics that the current strict operation intentionally does not mean; and
2. normative documentation, a sound proof, or an exhaustive finite domain establishes enough order, precision, rounding, and NaN behavior to implement a portable reference contract.

More observations of an unpublished instruction alone do not satisfy the second condition.

## Trigger check log

- 2026-08-12 — **not fired**. No registered caller requests this distinct meaning. The retained Apple evidence is finite empirical attribution, while the vendor description `d = a * b + c` leaves the correctness-bearing internal order, precision, and NaN boundaries unspecified.
