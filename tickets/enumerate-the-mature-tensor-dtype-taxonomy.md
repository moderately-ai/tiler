---
id: enumerate-the-mature-tensor-dtype-taxonomy
title: Enumerate the mature tensor dtype taxonomy
status: done
priority: p0
dependencies: []
related: [numerical-policy-contract]
scopes: [research/numerics]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research]
---

Enumerate the complete set of scalar value types, computation formats, storage
encodings, quantized types, and packed/block-scaled formats that a mature tensor
compiler should recognize as of 2026. This is a taxonomy and precedent pass,
not a commitment to implementation support.

The research must:

- use primary specifications and authoritative project documentation;
- distinguish logical element dtype, physical storage encoding, computation or
  accumulator format, quantization schema, and target capability;
- record canonical names, aliases, bit layouts/parameters, standards owners,
  ecosystem adoption, and maturity;
- include booleans, integers, binary/decimal floating point, bfloat, reduced
  precision floating formats, complex, quantized/fixed-point, packed sub-byte,
  and block-scaled families;
- identify types commonly called dtypes that should instead be resource, token,
  string, opaque, or metadata kinds;
- avoid equating the target-independent type universe with one backend's native
  arithmetic set; and
- preserve disputed, vendor-specific, or emerging formats as such rather than
  flattening them into false equivalence.

Deliver a reviewed taxonomy and capability-level model in `docs/research/`.
Only after that review should a separate decision select Tiler's representable,
reference-evaluable, optimizable, backend-supported, and initial-profile sets.

## Outcome

The [mature tensor dtype taxonomy](../docs/research/numerics/mature-dtype-taxonomy.md)
separates logical types, numeric interpretations, storage encodings, compute
formats, accumulators, results, and non-tensor graph values. Later ADRs adopt
selected recognized identities, but the inventory deliberately does not claim
reference-evaluator, optimizer, or backend support.
