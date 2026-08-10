Ticket: build-the-model-level-measurement-harness
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/build-the-model-level-measurement-harness/e56b9615f51f_c99ac54950f2.md
Pre-edit content hash (from ledger): e56b9615f51f9aff64208fa85f63fff694c84bff2cbc7e7e98010678b4547160
Post-edit content hash: 105a5ed419a8ea4fa041b99a4fe4955202f315a817aa8b8ee6e1bbf787941249

Changes applied:
  - Required work **Assert the exact invariants**: left original bullet quotable; added **Correction — 2026-08-10.** marking identity count and cold pipeline-creation count as pinned **conditionally** on L6's D-19 (citing L8 2026-08-05 regression-policy correction and L6); until D-19 closes, a fourth is reported with attribution (specialization on `S` vs `T = 1` graph divergence) rather than failed as a build defect; 30/270 remain unconditional; parallel counted-populations and TTFT three-identity/pipeline wording inherits the same pin.
  - Same correction: tiled value-contraction at exactly one of nine executions at `S = 16` kept as design invariant the harness records; cannot fire on device until `realize-the-tiled-contraction-schedule-and-its-metal-emission` (deferred) supplies the second packaged variant, so harness refuses a pass claim on a population that never had two variants.
  - Bench-host discipline bullet: free `N` replaced in place with inherited L3/L8 procedure — five interleaved A/B rounds; settled minimum over rounds 1–4; round 0 separate; spread beside every figure (reiterated in the dated correction).
  - Metadata unchanged (status todo; deps/related/scopes/tags already graph-true per report).

Optional items skipped (with reason):
  - none (report listed no optional graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required (Exact files: this ticket only; no new remainder for D-19 — owned by decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode / L6).
  - Product residual (out of wave B): no model-level measurement harness exists yet; close still depends on drive-the-complete-forward-pass-over-three-artifacts and retained C1 records on both hosts.
  - Report residual uncertainty left as-is: schema-only harness before drive closes; unaccepted draft public spelling of record schema and on-disk location.

Verification:
  - files read: audit report full; ticket full (pre/post); L8 model-level-qualification.md regression-policy pin and bench-host five-round Proposal; L3 first-metal-contraction-realizations.md five interleaved A/B rounds Measurement; realize-the-tiled-contraction-schedule-and-its-metal-emission status deferred.
  - checks: shasum -a 256 post-edit; rg anchors for conditionally / five interleaved / Correction — 2026-08-10 / 1–4 / realize-the-tiled on the ticket.

Recommended next ledger state:
  integrated
