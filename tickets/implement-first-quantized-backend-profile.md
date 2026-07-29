---
id: implement-first-quantized-backend-profile
title: Implement the first selected quantized backend profile
status: deferred
priority: p2
dependencies: [prototype-quantized-value-vertical, group-internal-compound-materializations-by-logical-value]
related: [implement-workload-selected-quantized-parameter-maps]
scopes: [implementation/compiler, implementation/artifact, implementation/reference, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, quantization, backend, deferred]
---
Activate only after a concrete quantized format, operation set, target backend,
storage layout, numerical contract, and conformance corpus are selected. Then
implement lowering, schedule feasibility, code generation, ABI/runtime binding,
and device comparison without generalizing beyond that measured profile.

Before activation, revise the scopes and dependencies to name the selected
backend, runtime adapter, reference/conformance owner, and measured corpus. For
that selected profile, supported programs compile, execute, and match the
normative reference; every program outside it receives a typed refusal.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Required completeness after activation

- Consume one complete `ResolvedValueType::encoded_numeric` contract and its ordered component declarations. Do not introduce a backend `Q4`, `Q8`, or `quantized integer` dtype that loses the scheme, parameter map, conversion, or metadata association.
- Carry every selected logical component role through lowering, physical storage, artifact identity, role-addressed ABI binding, runtime placement, and resource lifetime. Same-shaped components remain distinguishable by role and can never be inferred from slot order.
- Keep logical scalar type, numerical interpretation, parameter map, physical encoding, and target-native arithmetic as separate capability decisions. An unpacked `u4`, packed `u4`, affine value with `u4` codes, MXFP4 value, and native FP4 operation are five different claims.
- State conversion, physical layout/access, and observable materialization contracts separately. Native instructions, helper code, unpack/dequantize paths, and fused paths must reproduce the same selected semantics or reject.
- Prove both an ordinary byte-addressed path and an actually packed sub-byte path if the selected profile claims both. State bit order, storage-unit order, padding, alignment, partial and unaligned access, ownership of shared boundary bits, and canonical tail behavior.
- Record exact target-family dispatchability and numerical honourability for every dtype the profile consumes. An unmeasured `(target family, dtype)` pair is `Unknown` and cannot produce an executable artifact.
- Compare exact codes and exact reference bits where the contract is exact. Any tolerance or model-level error criterion must be derived from the selected scheme and domain, with saturation and exceptional cases tested separately.
- Refuse every unselected scheme, code dtype, map, storage encoding, operation signature, accumulator, output dtype, and target realization with a typed diagnostic naming the missing capability.
- Keep future boolean, integer, complex, decimal, codebook, hierarchical-scale, MX, sparse, and ragged support outside this profile unless the selected workload makes one a real producer and consumer. Generic seams are not support claims.

## Graph maintenance

- Consume the selection and elimination record from `scope-first-quantized-lm-profile`; do not repeat format selection here.
- Add a dependency on `implement-workload-selected-quantized-parameter-maps` only if the chosen profile uses a non-per-tensor map.
- Add a dependency on `implement-first-runtime-semantic-value-precondition-enforcement` for any selected scheme whose valid execution domain requires runtime tensor-value validation.
- Split weight ingestion, packing/repacking, native contraction, runtime binding, and model-level comparison when their scopes or evidence can move independently. Each split ticket must name its exact scheme, target, operation, and corpus.
- Advance versioned identities only for fields the selected producers actually fill, then recompute pins on the merged tree.
