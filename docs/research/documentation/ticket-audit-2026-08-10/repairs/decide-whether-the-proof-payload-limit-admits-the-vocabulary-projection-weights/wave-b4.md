Ticket: decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-proof-payload-limit-admits-the-vocabulary-projection-weights/ae197eeac78e_c99ac54950f2.md
Pre-edit content hash (from ledger): ae197eeac78ef42be8deb2d2babfe9af16c3e0d5d31881e26622103f1c08a7df
Post-edit content hash: 85c9c512bcf314386b2a2e5a7a5ff62d58f411554ab3565c7f21a84de01a3343

Changes applied:
  - Hard-stop reading-1 bullet: "the only L3 cell with none" → "the only L3 cell without a routed conformance member", with pointer to Findings Imprecise reclassification.
  - Findings population item 5: removed false `cases_for` operand-table gap; stated that `L3CorrectnessCell` already synthesizes for every `L3_CORRECTNESS_CELLS` extent including `w_vocab_slice`.
  - Census sentence: "six crates" → "five crates".
  - Closing condition and recommendation: value-change authority is "Tom's public-contract decision" rather than ADR 0075 alone (ADR 0075 still named for promotion/signature categories).
  - Added `## Fact audit — 2026-08-10` recording the four repairs. Board metadata left unchanged (`awaiting-decision`, scopes, related).

Optional items skipped (with reason):
  - none listed as optional in Repair required beyond "Do not file follow-through until a reading is chosen" (obeyed).

Residuals not applied (docs/crates/new tickets/authority):
  - Product decision still open: Tom must accept a reading for `MAX_PROOF_PAYLOAD_BYTES` (recommended: derive from container). Not applied — ticket outcome is decision, not implementation.
  - On accept, Exact files remain residual product debt: `crates/tiler-artifact/src/proof/mod.rs`, `docs/artifact-abi.md`, conformance envelope/publication pins + tests, owning-crate PayloadBytes negative test, optional schema-minor companion — wave B forbids crates/docs edits.

Verification:
  - files read:
    - full audit report at assigned path
    - full ticket before and after edit
    - `crates/tiler-conformance/src/publication/proof.rs` (`ProofFamily::L3CorrectnessCell` arm of `cases_for`)
    - `rg '16 \* 1024 \* 1024' crates/` (20 hits in five crates)
    - ADR 0075 Decision always-ask categories
  - checks:
    - post-edit `shasum -a 256` of ticket = `85c9c512bcf314386b2a2e5a7a5ff62d58f411554ab3565c7f21a84de01a3343`
    - repaired phrases present; false operand-table and "six crates" claims gone from live Findings text

Recommended next ledger state:
  integrated
