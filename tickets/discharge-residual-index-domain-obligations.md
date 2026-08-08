---
id: discharge-residual-index-domain-obligations
title: Discharge residual index-domain obligations before program work
status: done
priority: p1
dependencies: [carry-unknown-index-domain-obligations]
related: [implement-exact-compiler-index-domain-discharge]
scopes: [implementation/ir, implementation/compiler, contracts/foundation, contracts/optimizer, contracts/numerics, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, proof]
---

## User-visible outcome

Every residual index-domain obligation is either discharged by named semantic evidence or refused at an explicit pre-program-work stage, so no execution path can begin with an unproved coordinate bound.

## Implementation keys

- Define one named semantic-discharge stage over retained `IndexDomainPredicate` values and the accepted three-way proof outcome.
- `Proved` produces durable evidence; `Disproved` produces an explainable semantic refusal; `Unknown` may select an explicitly supported semantic host check before routing commitment, otherwise it refuses.
- Keep semantic host validation distinct from physical variant guards. A failed semantic validation is invalid input, never a plan miss or fallback trigger.
- Ensure no output/scratch allocation, encoding, submission, or other program work occurs before all required semantic discharge completes.
- Emit typed explain records for the predicate, subject, outcome, evidence or unknown reason, and refusal stage.
- Exercise at least one opaque semi-affine obligation once symbolic-divisor representation exists; until then, keep that fixture dependency explicit rather than synthesizing a substitute language.

## Closes when

Proved, disproved, and unknown obligations traverse the named stage with distinct typed outcomes; supported host validation occurs before program work; unsupported unknowns refuse explainably; post-commit fallback remains impossible; targeted `tiler-ir` and `tiler-compiler` nextest/Clippy pass; every new check has demonstrated its failure path; and `make full` passes.

## Graph maintenance

- On completion, update the IR, optimizer, correctness, and execution-order contracts that describe semantic obligation discharge.
- Link or file the concrete semantic-host-check implementation owner if this ticket establishes only the protocol.
- Close the parent chain only when no residual obligation can reach program work undisposed.

## Implementation outcome

- `PendingIndexRefinement` is consumed by one compiler-owned semantic-discharge stage before cover enumeration. The stage borrows each exact region-owned obligation once and preserves `Proved`, `Disproved`, and `Unknown` as distinct typed claims.
- An all-`Proved` result seals content-identity-bearing receipts over the region, obligation, rule authority and revision, and sound or exhaustive proof payload. Any other aggregate result refuses atomically; the immutable verified region is never rebuilt or mutated.
- This ticket initially installed a conservative production authority that preserved the verifier's exact `Unknown`, emitted one typed `semantic-discharge` record per obligation, and refused before cover enumeration. `implement-exact-compiler-index-domain-discharge` subsequently advances that authority with exact finite compiler evaluation while preserving the same protocol and refusal boundary.
- Runtime tensor-value conformance is distinct from index-coordinate discharge. `implement-first-runtime-semantic-value-precondition-enforcement` owns the first selected direct encoded-input vertical after the one-way routing commit; packed, complex, unsupported quantized maps, sparse/ragged, and extension-type views cannot be approximated by a dense-`f32` callback.
- The opaque semi-affine fixture remains dependent on the unimplemented symbolic-divisor representation; this ticket does not synthesize a substitute predicate language.
