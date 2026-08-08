---
id: implement-exact-compiler-index-domain-discharge
title: Implement exact compiler index-domain discharge
status: done
priority: p1
dependencies: [discharge-residual-index-domain-obligations]
related: []
scopes: [implementation/compiler, contracts/optimizer, contracts/foundation, contracts/numerics, research/runtime, implementation/cargo-lock]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, correctness]
---

## User-visible outcome

A finite residual logical index-domain obligation that exceeded the structural verifier's proof budget is discharged by exact compiler-host evaluation before executable planning, while an unsafe access is reported as invalid compiler output and an over-budget or unsupported case remains a named refusal.

## Implementation keys

- Implement one private production discharge authority that evaluates each exact residual atom over its subject access's complete finite logical domain with exact integer arithmetic and a separately governed resource budget.
- Return `Proved(ExhaustiveFinite)` only after checking every point, `Disproved` with a bounded canonical coordinate counterexample at the first canonical failing point, and `Unknown(ResourceLimit)` or `Unknown(UnsupportedFragment)` without execution permission.
- Preserve the existing failure distinction: disproving a lowering-produced access bound is invalid compiler/provider output; it is not invalid user input, target infeasibility, a plan miss, or a fallback trigger.
- Keep the checker representation-neutral by construction. `IndexDomainPredicate` contains only logical coordinate and extent atoms, so it must not read scalar payloads, storage encodings, component buffers, packing, complex layout, or quantization metadata.
- Exercise the same coordinate claim over recognized nominal, parameterized-complex, and encoded-numeric boundary types and require identical outcomes. These fixtures prove dtype independence; they do not claim executable bool, sub-byte, complex, or quantized support.
- Bind the authority, rule revision, exact region and obligation, proof basis, point count or counterexample, and governed budget result into receipts and explanation without adding a public provider registry.

## Closes when

The production compiler proves the existing beyond-verifier-budget valid fixture, rejects a deterministic unsafe fixture as invalid compiler output, preserves an over-discharge-budget fixture as `Unknown`, demonstrates identical assessment across nominal, parameterized, and encoded-numeric logical types without inspecting payload representation, emits attributed receipts and explain records, refuses before cover enumeration on every non-proof outcome, passes targeted per-package tests and Clippy, proves every new check can fail, and passes `make full`.

## Graph maintenance

- Correct the optimizer and IR contracts to state that this discharge is compiler-time logical coordinate evaluation, not runtime tensor-value validation.
- Keep the production `tiler-compiler` dependency closure free of `tiler-reference`; the independent oracle remains development-only evidence.
- Revisit a public discharge-provider registry only when a second independently installable authority and its resolution contract exist.
- Do not claim a quantization-parameter-coordinate fixture before ADR 0029's deferred parameter-map IR has a real producer. `prototype-quantized-value-vertical` owns that semantic representation.
- Track tensor-value semantic precondition enforcement separately, behind its first real operation producer, rather than reserving artifact/runtime fields no producer can fill.

## Scope correction

The original ticket conflated three contracts. Residual index-domain predicates are only `NonNegative` and `LessThanExtent` atoms over verified logical index expressions and extents; their truth is independent of dtype and physical representation. Runtime tensor-value preconditions such as strict affine quantization's NaN rejection are ADR 0033 enforcement work, while packed booleans, sub-byte integers, planar/interleaved complex values, quantized component buffers, and sparse or ragged layouts are logical-to-physical representation work. Combining them would make an irrelevant payload scan appear to prove coordinate safety and would misclassify an unsafe lowering as invalid user input. This ticket implements only the exact compiler-owned coordinate authority that the current residual protocol actually requires.

## Implementation outcome

- The private production authority evaluates the complete current `IndexExprView` vocabulary with arbitrary-precision integers over a canonical finite logical domain. It reuses its per-point maps, proves only after complete enumeration, and stops independently at sixteen million expression cells.
- A false atom retains the first canonical point ordinal and exact encoded coordinate/value witness, is explained with its provider and rule revision, and is classified as invalid compiler output before cover enumeration. Unsupported and over-budget claims remain typed `Unknown` refusals with the verifier's original resource stop kept distinct from the discharge authority's current stop.
- Nominal boolean, nominal sub-byte integer, parameterized complex, and encoded-numeric fixtures produce identical coordinate claims without payload or representation inspection. This is a guarantee about logical coordinate proof only; it does not claim executable support for those representations.
- The conflated runtime remainder is now `implement-first-runtime-semantic-value-precondition-enforcement`. Its first selected subject is a direct encoded interface input, using the delivered resolved-value conformance contract/evidence/scan after `RoutingCommit`; it does not require an internal compound-value producer.
- Every new proof, disproof, budget, dtype-neutrality, attribution, resource-separation, and canonical-point check was fault-injected and observed failing. `cargo nextest run -p tiler-compiler`, per-package Clippy, compiler doc-tests, dependency-closure inspection, `tkt lint`, `git diff --check`, and `make full` pass.
