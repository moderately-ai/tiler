Ticket: admit-a-reduction-over-a-declared-input-tensor
Wave: B1
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-reduction-over-a-declared-input-tensor/4ff7f64cab02_c99ac54950f2.md
Pre-edit content hash (from ledger): 4ff7f64cab02e1c05c2de2b3793cc3b1d11b9d5d7a7d2e07e28b326bf817a791
Post-edit content hash: 80de6a560caa4081f6455187ed44a08856da03241dd6db2f862b6ea4f8132838

Changes applied:
  - Outcome verifier-arm inventory: reworded serial write / pointwise "Deliberately left" as landing-time snapshot; corrected helper name to `reads_bind_boundary_tensors_in_order`; added **Correction — 2026-08-10** for `CommittedTensor::CoverAssigned` (Intermediate|Output) and later Intermediate-read/write widening.
  - Outcome compiler-side sentence: `contributor_tensor` derivation from `NormalizedSerialSum::contributor_input` (not `prologue`), noting complementary presence with prologue.
  - Measurement explain pin / hex census: framed as historical landing evidence; **Correction — 2026-08-10** for live `request=7ba3d77a66f04638`; nextest 2793 marked as landing totals.
  - Scopes neighbour inventory: **Correction — 2026-08-10** — `admit-elementwise-epilogues-over-a-materialized-intermediate` is done; lane-typed and bf16-ios remain blocked.
  - Added `## Fact audit — 2026-08-10` summarizing all five repaired claims; status/deps/related unchanged (done stays done).

Optional items skipped (with reason):
  - none (report optional path was "related list" hygiene; related already sound; no optional prose left unapplied).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket's close condition. Non-first contributor ordinal remainder remains on already-filed `admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary` (no new ticket). Report Exact files listed only this ticket for wave B1.

Verification:
  - files read:
    - tickets/admit-a-reduction-over-a-declared-input-tensor.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-reduction-over-a-declared-input-tensor/4ff7f64cab02_c99ac54950f2.md
    - crates/tiler-ir/src/schedule/builder.rs (CoverAssigned, DeclaredDomain, reads_bind_boundary_tensors_in_order)
    - crates/tiler-compiler/src/physical.rs (contributor_tensor / contributor_input)
    - crates/tiler-compiler/src/explain.rs (live request= pin)
    - tickets/admit-elementwise-epilogues-over-a-materialized-intermediate.md (status done)
    - tickets/admit-lane-typed-values-and-masked-memory-into-the-kernel-ir.md (status blocked)
    - tickets/declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles.md (status blocked)
  - checks:
    - grep CoverAssigned / DeclaredDomain / contributor_input / reads_bind_boundary_tensors_in_order / request= under crates
    - status greps on neighbour tickets
    - shasum -a 256 of ticket post-edit

Recommended next ledger state:
  integrated
