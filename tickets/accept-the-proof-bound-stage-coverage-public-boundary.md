---
id: accept-the-proof-bound-stage-coverage-public-boundary
title: Accept the proof-bound stage-coverage public boundary
status: awaiting-decision
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

Tom accepts or amends the public surface `bind-stage-coverage-to-index-refinement-identity` landed as a reviewed draft on 2026-08-05 (implementation `67765e00`, independently reviewed at `cd3119f5` with a safe-to-merge-as-draft verdict and two live perturbation re-runs):

- `tiler_ir::program::CoveredOccurrence` — private fields; `from_receipt(&IndexRefinementReceipt) -> Self` as the sole constructor; borrowed readers `occurrence()` and `refinement()`; no `Default`, no `Ord`, no serde.
- `KernelProgramBuilder::push_stage(.., coverage: &[CoveredOccurrence], ..)` — was `&[SemanticOccurrence]`.
- `StageRef::coverage() -> &[CoveredOccurrence]`.
- `KernelProgramBuildError::ForeignCoverageGraph { occurrence }` — new variant on the non-exhaustive enum.
- The artifact identity cross-referencing the stepped stage key (`tiler.kernel-program.v9`, IR stage key `v2`, `tiler.artifact-program.stage.v3`).

The construction boundary (verifier-receipt-only minting), the ADR 0072 unused-authority exclusion, the foreign-graph guard, and proof-gap unrepresentability were each verified by the independent review with the checks watched failing. The known cost is recorded in `measure-executable-coverage-identity-growth-against-the-program-identity-bound`.

Filed at `awaiting-decision` per this board's convention: only Tom closes an acceptance ticket. Amendments go back through a correction dispatch on a new branch; acceptance closes this ticket and releases the drafted-boundary labelling.
