---
id: accept-the-proof-bound-stage-coverage-public-boundary
title: Accept the proof-bound stage-coverage public boundary
status: done
priority: p1
dependencies: []
related: [bind-stage-coverage-to-index-refinement-identity, reclassify-the-covered-occurrence-public-boundary-acceptance-labels]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

Tom accepts or amends the public surface `bind-stage-coverage-to-index-refinement-identity` landed as a reviewed draft on 2026-08-05 (implementation `67765e0068ef5048b9698b4df02ec7b30519f827`, independently reviewed at `cd3119f50964af7086f9c93f6e5f4af4181050c4` with a safe-to-merge-as-draft verdict and two live perturbation re-runs):

- `tiler_ir::program::CoveredOccurrence` — private fields; `from_receipt(&IndexRefinementReceipt) -> Self` as the sole constructor; borrowed readers `occurrence()` and `refinement()`; no `Default`, no `Ord`, no serde.
- `KernelProgramBuilder::push_stage(.., coverage: &[CoveredOccurrence], ..)` — was `&[SemanticOccurrence]`.
- `StageRef::coverage() -> &[CoveredOccurrence]`.
- `KernelProgramBuildError::ForeignCoverageGraph { occurrence }` — new variant on the non-exhaustive enum.
- The artifact identity cross-referencing the stepped stage key (`tiler.kernel-program.v9`, IR stage key `v2`, `tiler.artifact-program.stage.v3`).

The construction boundary (verifier-receipt-only minting), the ADR 0072 unused-authority exclusion, the foreign-graph guard, and proof-gap unrepresentability were each verified by the independent review with the checks watched failing. The known cost is recorded in `measure-executable-coverage-identity-growth-against-the-program-identity-bound`.

Filed at `awaiting-decision` per this board's convention: only Tom closes an acceptance ticket. Amendments go back through a correction dispatch on a new branch; acceptance closes this ticket and releases the drafted-boundary labelling.

## Decided — accepted

Accepted by Tom on 2026-08-05 at the live decision review in the coordination session, witnessed first-hand by the coordinator. The surface carries no draft-labelling language in the code (`grep -rn "awaiting\|reviewed draft" crates/tiler-ir/src/program/model.rs crates/tiler-ir/src/program/mod.rs` returns nothing), so the sweep is this record and the catalog of decisions is untouched — the boundary was tracked here rather than in an ADR. The known identity-growth cost was measured before acceptance: exact fit 134n²+3650n+710, refusal at n=695 by labelled extrapolation, ×125 margin at the roadmap's per-layer partition.

**Correction — 2026-08-10.** The claim that "the sweep is this record and the catalog of decisions is untouched" understated live contract sites. Code draft labels under the ticket's named grep paths remain absent (verified at the audit base). As of audit base `c99ac54950f2`, three documents still carry present-tense "not yet accepted" language that acceptance was supposed to release: the `docs/ir.md` Proposal paragraph for CoveredOccurrence (`the public boundary that lands with them is not yet accepted`), ADR 0071 (`Built, and not yet accepted` / `public boundary is not yet accepted` / partial-status rationale still waiting on this acceptance), and `docs/research/documentation/production-crate-codebase-audit.md` (`the public boundary is not yet accepted`). Leaving the ADR catalog row untouched may still be intentional (no dedicated ADR for this surface), but present-tense denials in those three files are false live maturity claims after Tom's acceptance. Reclassification is owned by [`reclassify-the-covered-occurrence-public-boundary-acceptance-labels`](reclassify-the-covered-occurrence-public-boundary-acceptance-labels.md). Implementation and review short hashes above expand to `67765e0068ef5048b9698b4df02ec7b30519f827` and `cd3119f50964af7086f9c93f6e5f4af4181050c4`.
