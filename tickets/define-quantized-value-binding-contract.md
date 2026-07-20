---
id: define-quantized-value-binding-contract
title: Define quantized value and parameter binding contract
status: done
priority: p0
dependencies: []
related: [numerical-policy-contract]
scopes: [research/numerics, contracts/core, contracts/artifacts]
shared_scopes: []
paths: []
tags: [tiler-research, numerics, foundation]
---
Resolve the carrier and ownership model for first-class quantized value interpretations, including static and runtime scale/zero-point/codebook operands, per-axis and per-block mapping through views, canonical identity, reference evaluation, and ABI binding. Use docs/research/numerics/quantization-ir-precedents.md as the evidence baseline. Produce an ADR only after the ownership model and invariants are accepted.

## Outcome

Resolved by ADR 0030 and `docs/research/numerics/quantized-value-and-transform-contract.md`: static scheme structure lives in the semantic type contract; concrete components remain graph operands to a dedicated assembly/conversion operation; index transformations preserve quantization only by proving composed parameter-selection maps; logical values expand to stable component roles during ABI lowering.
