Ticket: correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate/8d0915697d6d_c99ac54950f2.md
Pre-edit content hash (from ledger): 8d0915697d6d8b542497c4dcc8ee0ce3b9f152c670218b332d747bedec93cfdc
Post-edit content hash: 3f5a3d240fce6766c9d4eeed7ff2ebd309a451f9881bca981f0426664280e5cc

Changes applied:
  - Outcome: added **Correction — 2026-08-10.** striking the 2026-08-07 clause that names `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` as the `unproven-reassociation` site; replaces with `fusion_legality::tests::a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` (f32 reduction + permitting reassociation; asserts `ReductionReassociation` / `unproven-reassociation`). Notes the BF16 wall asserts `unrealized-contraction` instead.
  - Outcome: same correction strikes "Six surviving `dtype-f32` mentions" and records the classified census at this base (four hits, all dated 2026-08-07 / retired-gate framing; counting rule stated).
  - Frontmatter `related`: added optional discoverability edge to `correct-the-reassociation-unknown-claim-a-repair-block-introduced-in-the-bf16-vertical` (already-done sibling that repaired the source header).
  - Status left `done` (code-side close conditions hold; residual was ticket prose only).

Optional items skipped (with reason):
  - none (optional related edge applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required. Report listed no crates/docs product debt and no remainder filing for residual repair.

Verification:
  - files read:
    - audit report 8d0915697d6d_c99ac54950f2.md (full)
    - tickets/correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate.md (full, pre/post)
    - crates/tiler-conformance (rg dtype-f32)
    - crates/tiler-compiler/src/fusion_legality.rs (reassociating-contract test body at `a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction`)
    - crates/tiler-compiler/tests/bf16_numerical_contract.rs (wall asserts unrealized-contraction)
    - crates/tiler-conformance/src/bf16_vertical.rs (header cites correct reassociation Unknown site)
  - checks:
    - `rg 'dtype-f32' crates/tiler-conformance` → four hits, all correction/history prose
    - wall test asserts `unrealized-contraction`; fusion_legality reassociating test asserts `unproven-reassociation`
    - post-edit `shasum -a 256` ticket → 3f5a3d240fce6766c9d4eeed7ff2ebd309a451f9881bca981f0426664280e5cc

Recommended next ledger state:
  integrated
