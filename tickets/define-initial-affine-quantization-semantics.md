---
id: define-initial-affine-quantization-semantics
title: Define initial affine quantization semantics
status: done
priority: p0
dependencies: []
related: [numerical-policy-contract, define-quantized-value-binding-contract]
scopes: [research/numerics, contracts/core, contracts/compiler]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, foundation]
---
Resolve the first built-in affine Quantize/Dequantize/Requantize numerical contract, including NaN behavior, endpoint ordering, compute/intermediate dtypes, subnormal policy, and portable conformance. Use docs/research/numerics/affine-quantization-semantics.md as the evidence baseline. Keep logical Requantize distinct from specialized integer Rescale.

## Outcome

[ADR 0031](../docs/decisions/0031-strict-affine-quantization-rejects-nan.md)
resolves strict NaN handling as semantic rejection while preserving explicitly
named alternative mappings.

[ADR 0032](../docs/decisions/0032-strict-affine-quantization-evaluation.md)
resolves strict evaluation order, widened subtraction, explicit
computation dtype, subnormal preservation, endpoint handling, and the separation
of logical `Requantize` from integer `Rescale`. The complete evidence is in
[Affine quantization numerical semantics](../docs/research/numerics/affine-quantization-semantics.md).

The initial semantic/reference profile is now `i4/u4/i8/u8` codes with `f32`
expressed, scale, computation, and requantization-intermediate values across
per-tensor, per-axis, and per-block maps. The first physical proof requires one
8-bit path and one packed 4-bit block path; individual backend cells remain
capability-gated.
