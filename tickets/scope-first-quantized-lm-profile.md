---
id: scope-first-quantized-lm-profile
title: Scope the first workload-backed quantized language-model profile
status: todo
priority: p2
dependencies: [define-first-metal-lm-workload, spike-first-metal-contraction-vertical, prototype-quantized-value-vertical]
related: [implement-first-quantized-backend-profile, define-initial-affine-quantization-semantics, define-quantized-value-binding-contract]
scopes: [research/numerics, research/scheduling, research/apple-targets, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, quantization, language-model, matmul, metal]
---
Use the selected workload and measured contraction evidence to choose the first
quantized language-model profile. This ticket must not select a format before
the model, target, numerical behavior, and performance evidence make that
choice meaningful.

## Required analysis

- Compare candidate weight and value representations against the workload's
  memory, accuracy, packing, and Metal execution requirements.
- Define code, scale, zero-point, grouping, axis, layout, and conversion
  identity for every surviving candidate.
- Determine whether contraction consumes packed values directly or through an
  explicit dequantization boundary.
- Define the normative reference, accumulation behavior, output dtype, error
  criteria, artifact identity, weight validation, and runtime binding.
- Measure memory and performance against the non-quantized baseline on the
  selected target where feasible.

Eliminate any profile that cannot be validated or whose numerical realization
is unknown. A smaller artifact is not by itself evidence of a correct or faster
model.

## Ticket-producing outcome

Activate and refine `implement-first-quantized-backend-profile` for the selected
profile, or supersede it with narrower delivery tickets. File any additional
work for weight ingestion, packed contraction, conversion, conformance, and
model-level comparison with exact dependencies and scopes.

## Closes when

One bounded profile is selected from reproducible evidence or every candidate
is rejected with explicit reasons; the generic quantized-value reservation is
connected to a model-visible execution path; and all surviving work has
dependency-ordered tickets.

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L7** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L1 and L3 deliver **and** milestone 2Q supplies the quantized-value vertical proof.

**Rests on:** L1, L3, and milestone 2Q.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.
