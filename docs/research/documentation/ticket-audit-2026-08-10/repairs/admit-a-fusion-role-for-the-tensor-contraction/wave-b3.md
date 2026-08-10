Ticket: admit-a-fusion-role-for-the-tensor-contraction
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-fusion-role-for-the-tensor-contraction/98884d9f2077_c99ac54950f2.md
Pre-edit content hash (from ledger): 98884d9f20774a35a3bde8204c91da52ce95df94f9c80d9a7a800a2b28d1e76a
Post-edit content hash: 6fcc50b6beaeb93c77d777fa6064b434b69229d55b0a740307993b89669b6347

Changes applied:
  - Rewrote Outcome Fact exclusivity: replaced "policy's `TENSOR_CONTRACTION` row is the only admitted operation declaring `NumericalDimension::Contraction`" with several-families wording (multiply/add via `ARITHMETIC`, rms-norm via `NORMALIZATION`, contraction via `TENSOR_CONTRACTION`); distinctive point is the per-contributor multiply-plus-add adjacency that disqualifies the closed same-family-pointwise proof.
  - Added **Correction — 2026-08-10, on the explain pin absolute string** after the landing Measurement that cited `request=6dd42be71c6745fe` at `explain.rs:4149` — live pin at audit base is `request=7ba3d77a66f04638`; non-movement claim for this landing and provider-identity Inference preserved.
  - Metadata left unchanged (status done; dependencies/related/scopes already coherent per report).

Optional items skipped (with reason):
  - none (optional pin dated note applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-compiler/src/fusion_legality.rs — same false exclusivity phrase in registration/closed-proof comments (optional product prose; wave B ticket-only).
  - docs/research/program-planning/flash-class-capability-set.md — axis 2 residual stale `todo`/seam maturity lines after 2026-08-06 delivery correction (outside declared scopes; research/program-planning owner if board wants consistency).
  - No new remainder ticket required for this ticket's delivery.

Verification:
  - files read:
    - full audit report at assigned path
    - full ticket admit-a-fusion-role-for-the-tensor-contraction.md
    - policy.rs ARITHMETIC / NORMALIZATION / TENSOR_CONTRACTION dimension rows (all include Contraction)
    - explain.rs live pin `tiler-explain-v7 request=7ba3d77a66f04638`
  - checks:
    - sha256 of ticket after edit: 6fcc50b6beaeb93c77d777fa6064b434b69229d55b0a740307993b89669b6347
    - live present-tense "only admitted operation declaring" exclusivity no longer in Outcome Fact
    - pin Measurement retained as historical; Correction points at live explain golden

Recommended next ledger state:
  integrated
