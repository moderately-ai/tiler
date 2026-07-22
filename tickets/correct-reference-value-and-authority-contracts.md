---
id: correct-reference-value-and-authority-contracts
title: Correct reference value and authority contracts
status: in-progress
priority: p0
dependencies: [harden-semantic-registry-and-program-construction]
related: []
scopes: [implementation/reference, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, reference]
claimed_from: todo
assignee: codex-root
lease_expires_at: 1784728370
---

Correct the reference capability boundary exposed by the fixed-point code
audit at `ad6e9f463de6eabad44af47eaddad9317e0935fd`. The reference crate is a
normative consumer-neutral capability, not an `f32` test helper with a generic
signature painted over it.

## Required outcome

- Replace or narrow the mismatch where `ReferenceSignature` admits arbitrary
  resolved types but `Tensor` and `ReferenceOperation` carry only `Vec<f32>`.
  Prefer an exact resolved-type-bearing reference value seam that can host
  compound and quantized values; if a profile remains `f32`-only, registration
  must reject every other type explicitly.
- Bind each reference capability to the complete reached semantic definition,
  admission provenance, provider revision, and registry snapshot subjects it
  actually implements. Reusing a key and signature must not make a changed
  normative/schema/conformance definition look equivalent.
- Make provider registration transactional and sticky-failing, including when
  a provider catches or ignores a returned error.
- Preserve provider identity and typed cause when result arity, shape, or
  evaluation fails. Establish the documented native-callback panic and
  determinism trust boundary, with recoverable attribution where promised.
- Remove float-derived `PartialEq` semantics from normative tensors or replace
  them with exact bitwise equality so NaNs and signed zero follow the numerical
  contract.
- Make empty shapes and empty reductions zero-absorbing before overflow-prone
  stride/product work, and iterate large Cartesian domains lazily rather than
  materializing `Vec<Vec<usize>>` coordinates.
- Bound signatures, registry size, identities, provider diagnostics, and
  produced values before retaining them.
- Make the byte order of canonical float-bit construction explicit and checked
  at the public boundary.

## Acceptance

Tests must cover a non-`f32` nominal value, changed semantic authority under an
unchanged operation key, ignored duplicate registration, NaN and signed-zero
equality, late-zero shapes such as `[0, u64::MAX, 2]`, empty contributors,
large lazy iteration, provider-specific failures, and canonical float byte
order. Unsupported types must fail closed with typed explanations.
