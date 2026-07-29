---
id: implement-representation-aware-semantic-host-enforcement
title: Implement representation-aware semantic host enforcement
status: todo
priority: p1
dependencies: [discharge-residual-index-domain-obligations]
related: []
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/artifact, implementation/runtime, contracts/foundation, contracts/optimizer, contracts/numerics, contracts/artifacts, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, runtime, dtype]
---

## User-visible outcome

A residual logical index-domain obligation may be discharged by an explicitly supported semantic host check before routing commitment, across every admitted logical value representation, without treating validation failure as a plan miss or beginning program work first.

## Implementation keys

- Define the runtime/compiler ownership boundary for semantic host enforcement, including the target/runtime capability that admits a checker and the exact pre-routing phase that executes it.
- Evaluate the logical tensor view rather than assuming dense scalar storage: support or explicitly reject bit-packed booleans and sub-byte integers, complex planar/interleaved encodings, quantized compound values with stable component roles, per-tensor/per-axis/per-block parameter coordinate maps, codebooks, sparse or ragged components, and future nominal extension types.
- Bind checker identity and revision, the complete resolved logical type, numerical contract, component schema, coordinate map, physical encoding requirements, target capability, and validation witness into plan, artifact, cache, and explain identity at their proper layers.
- Keep semantic validation distinct from physical applicability guards. A disproved predicate is invalid input; Unknown is unsupported; neither is target infeasibility or a fallback trigger.
- Run every host check after input facts are bound but before routing commitment, output or scratch allocation, encoding, submission, cache publication, or other program work. No post-commit fallback is permitted.
- Specify deterministic typed errors, bounded witness/evidence formats, coherence and version checks, and resource limits. Never substitute a dense-f32 callback for a representation-aware evaluator.
- Exercise Proved, Disproved, and Unknown through packed bool/u4, complex, and multi-component quantized fixtures, including a quantization-parameter coordinate obligation.

## Closes when

A complete supported host-check vertical is represented, identified, enforced, and explained from compiler plan through artifact and runtime; every admitted representation has a logical-view evaluator or a named refusal; validation precedes routing commitment and all program work; invalid input cannot trigger fallback; targeted per-package tests and Clippy pass; every new check has demonstrated its failure path; and make full passes.

## Graph maintenance

- Update the IR, optimizer, numerical-correctness, artifact ABI, and runtime execution-order contracts.
- Revisit whether a public discharge-provider registry is justified only when a second independently installable authority and its resolution contract exist.
- Close only after the bounded runtime vertical is executable; protocol-only scaffolding does not satisfy this outcome.
