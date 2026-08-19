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

If a real caller needs seeded fused contraction semantics and sufficient evidence can define them portably, specify a distinct operation identity rather than weakening the standard contraction key. The operation must state its seed, contributor order or result set, intermediate precision, rounding, NaN boundaries, validation, and reference behavior before any target realization is considered.

**Correction — 2026-08-19 (key retired; the outcome's substance is unchanged).** This Outcome named `tiler::strict-tensor-contraction-f32@1` as the key not to weaken. That key is **retired from the standard vertical** under [ADR 0112](../docs/decisions/0112-replace-the-strict-contraction-key-with-a-permission-indexed-successor.md), replaced by `tiler::tensor-contraction-f32@1` (`crates/tiler-ir/src/semantic/contraction.rs`, anchor `is the documented successor to the`), with `crates/tiler-compiler/tests/retired_contraction_key_never_compiles.rs` pinning that the retired spelling never compiles. The successor is `tiler::tensor-contraction-f32@1`, and the do-not-weaken instruction transfers to it unchanged — indeed ADR 0112 is itself an instance of the rule this ticket states, since it minted a **new key** rather than mutating the old one's declared meaning, on the measured ground that a same-key semantic mutation leaves an old/new artifact hybrid join possible.

**Consequence for trigger condition 1, which a later worker must not misread.** That condition speaks of "the current strict operation" and its intentional refusal of `+0.0`-seeded fused semantics. The successor is *reassociation-permission-indexed*, not unconditionally strict, so "strict" there now names the **forbidden-reassociation** branch of `tiler::tensor-contraction-f32@1` rather than a separate strict key. The trigger's substance survives: neither branch means seeded fused contraction, and the successor's registered signature still declares an unseeded fold whose accumulator starts at the first product. **Do not read the permission-indexed successor as having already granted anything this ticket's trigger requires** — a reassociation permission is not a seed, a fusion, or a rounding relaxation.

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
