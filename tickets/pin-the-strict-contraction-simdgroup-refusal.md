---
id: pin-the-strict-contraction-simdgroup-refusal
title: Pin the strict-contraction simdgroup realization refusal
status: todo
priority: p1
dependencies: [qualify-the-simdgroup-matrix-contraction-realization]
related: [research-an-explicit-seeded-fused-contraction-operation, realize-the-contraction-through-the-appendable-direct-path]
scopes: [implementation/metal, implementation/compiler, implementation/ir, contracts/artifacts, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics, contraction, correctness, refusal, strict]
---
## Outcome

The owning realization boundary gives `simdgroup_multiply_accumulate` a named, typed incompatibility with `tiler::strict-tensor-contraction-f32@1`. The existing separately rounded, first-product-seeded lowering remains the only admitted Metal spelling. No target fact, fallback, default, or empirical runtime probe can turn the refused construct into that operation.

## Facts to re-read at the claimed base

- The registered `@1` operation requires separate multiplication/addition rounding, a first-product accumulator seed, and canonical NaN after every combine and at the result boundary.
- The retained `contraction_pair` observation distinguishes fused from separately rounded arithmetic by one result bit, and `negative_zero_seed` distinguishes a `+0.0` seed from the first-product seed.
- The current production Metal regression forbids `simdgroup`, `multiply_accumulate`, `fma`, and `mad` on the strict accumulation path.
- The retained finite attribution does not prove the instruction's contributor order, intermediate precision, or internal NaN behavior.

These are stale until the worker reads the operation schema, reference evaluator, construction and validation paths, realization/feasibility ownership, Metal lowerer, retained record, and correctness-bearing tests in full and reports a per-Fact verdict.

## Required delivery

Choose the narrowest owning typed refusal that remains visible in feasibility/explain output without adding a fake executable candidate merely to reject it. Reproduce the exact `contraction_pair` and `negative_zero_seed` observations in the owning check, retain the no-simdgroup emitted-source regression, and perturb fusion, seed, and NaN/order subjects independently so each load-bearing refusal fails with its own message. Preserve the distinction between “not a realization of `@1`” and any claim about a future distinct operation.

Do not revise `@1`, add a permissive target fact, infer semantics from the finite corpus, or widen artifact fixed records without a separately justified delivered fact. If the source census shows the current structural prohibition already provides the only honest owning classification, record and test that fact rather than creating duplicate error vocabulary.

## Closes when

The current strict operation has a reproducible, typed, explainable simdgroup incompatibility; both retained distinguishing observations and the emitted-source prohibition are guarded by subject perturbations; and every current identity/schema claim is re-derived at the implementation base.
