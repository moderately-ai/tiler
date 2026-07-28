---
id: discharge-residual-index-domain-obligations
title: Discharge residual index-domain obligations before program work
status: todo
priority: p1
dependencies: [carry-unknown-index-domain-obligations]
related: []
scopes: [implementation/ir, implementation/compiler]
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
