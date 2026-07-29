---
id: implement-first-runtime-semantic-value-precondition-enforcement
title: Implement first runtime semantic value-precondition enforcement
status: todo
priority: p2
dependencies: [prototype-quantized-value-vertical]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/runtime, contracts/foundation, contracts/numerics, contracts/artifacts, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, validation, runtime, quantization]
---

## User-visible outcome

The first real tensor-value semantic precondition is enforced over its exact logical value view with a deterministic typed result, complete witness and identity binding, and no possibility that invalid input selects another plan or fallback.

## Implementation keys

- Start from a governed operation that actually produces a residual `SemanticPrecondition`; strict affine `Quantize` rejecting NaN is the expected first consumer after `prototype-quantized-value-vertical`. Do not reserve artifact or runtime fields before that producer exists.
- Keep semantic operation preconditions distinct from index-region coordinate safety and physical representation applicability. A failed semantic precondition is invalid input; a disproved lowering access bound is invalid compiler output; an unsupported physical encoding is capability or feasibility.
- Define the typed residual obligation and witness identity over the exact predicate, logical subject and view, complete resolved value type, component roles and coordinate maps, value version or immutability provenance, producer completion, and coherence dependencies. Raw pointer identity is insufficient.
- Select an explicit host `EnforcementPlan` only when the runtime capability can reconstruct and inspect the authoritative logical view. Bind the checker provider and output-affecting revision, resource limits, deterministic error schema, observability requirements, and cost inputs into the selected plan and explain identity.
- Carry the complete derived logical and physical component contract through artifact identity and ABI. Never reintroduce caller-declared ABI facts, infer component roles from slot position, or treat integer codes alone as the quantized logical value.
- Validate packed booleans and sub-byte integers without reading unused packed bits; complex values independent of planar or interleaved storage; quantized values through stable code, scale, zero-point, codebook, mask, or hierarchical roles and their per-tensor/per-axis/per-block maps; and nominal extensions only through an admitted versioned evaluator. Sparse and ragged values remain named unsupported until their logical cardinality and component contracts are admitted.
- Use canonical logical row-major index, stable error code, and obligation ordinal for deterministic diagnostics, never physical byte offset or worker order.
- Resolve checker capability, observability, cost, and every preparation obligation before `RoutingCommit`, but begin the actual host tensor-value validation only afterward at `EnforcementCommit`, as ADR 0033 requires. Starting the scan while fallback authority still exists would make semantic work coexist with an alternate route and weaken the accepted ownership state machine.
- Represent the one-way runtime state explicitly as preflight, `RoutingCommit`, committed-needs-enforcement, successful enforcement, and executable dispatch. The committed-needs-enforcement state must not expose an executable entry point, and an enforcement witness must not be cached independently of the value version, coherence dependencies, checker revision, and exact obligation identity that make it valid.
- A successful witness may authorize the one committed route. After `EnforcementCommit`, a semantic failure, malformed witness, coherence failure, or enforcement execution failure cannot trigger ordinary fallback.

## Closes when

One governed semantic value precondition is represented from semantic verification through enforcement planning, plan and artifact identity, ABI binding, runtime capability resolution, host evaluation, deterministic witness or typed failure, explanation, and execution ordering; valid and invalid compound quantized fixtures exercise constant and runtime parameter roles; packed/sub-byte, complex, extension, sparse, and ragged representations are either correctly reconstructed or refused by name; validation begins only after one-way routing commitment and before result work, failure cannot publish results or fall back, all new checks have demonstrated failure paths, targeted per-package tests and Clippy pass, and `make full` passes.

## Graph maintenance

- Update numerical semantics, IR, optimizer, artifact ABI, runtime execution order, correctness testing, and the accepted ADR 0033 application status together.
- Advance artifact and cache identity domains exactly once after the merged tree contains the complete enforcement record; recompute every pin on that tree.
- Revisit a public provider registry only when a second independently installable enforcement authority exists. The first runtime integration should expose the smallest reviewed adapter-facing boundary.
- Keep physical representation work with `prototype-quantized-value-vertical` and boundary enforcers. This ticket consumes their complete derived contracts; it does not redefine them.
