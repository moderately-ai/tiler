---
id: define-initial-affine-quantization-semantics
title: Define initial affine quantization semantics
status: todo
priority: p0
dependencies: []
related: [numerical-policy-contract, define-quantized-value-binding-contract]
scopes: [research/numerics, contracts/core, contracts/compiler]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, foundation]
---
Resolve the first built-in affine Quantize/Dequantize/Requantize numerical contract, including NaN behavior, endpoint ordering, compute/intermediate dtypes, subnormal policy, and portable conformance. Use docs/research/numerics/affine-quantization-semantics.md as the evidence baseline. Keep logical Requantize distinct from specialized integer Rescale.
