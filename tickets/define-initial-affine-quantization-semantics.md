---
id: define-initial-affine-quantization-semantics
title: Define initial affine quantization semantics
status: in-progress
priority: p0
dependencies: []
related: [numerical-policy-contract, define-quantized-value-binding-contract]
scopes: [research/numerics, contracts/core, contracts/compiler]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, foundation]
claimed_from: todo
assignee: codex
lease_expires_at: 1784505945
---
Resolve the first built-in affine Quantize/Dequantize/Requantize numerical contract, including NaN behavior, endpoint ordering, compute/intermediate dtypes, subnormal policy, and portable conformance. Use docs/research/numerics/affine-quantization-semantics.md as the evidence baseline. Keep logical Requantize distinct from specialized integer Rescale.

## Progress

ADR 0031 resolves strict NaN handling as semantic rejection while preserving explicitly named alternative mappings. Evaluation dtype/order, subnormal behavior, and the initial portable dtype profile remain in progress.

ADR 0032 resolves strict evaluation order, widened subtraction, explicit
computation dtype, subnormal preservation, endpoint handling, and the separation
of logical `Requantize` from integer `Rescale`. The initial portable dtype and
backend profile remains to be selected.
