Ticket: plan-the-materialized-attention-decomposition
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/plan-the-materialized-attention-decomposition/1aa3d1187784_c99ac54950f2.md
Pre-edit content hash (from ledger): 1aa3d11877846e5e1eaf6b01a19d7c3ae24ecfcb2427282b583eacdf608a4514
Post-edit content hash: 04a7bcbae18c61c2573f48baa1f6f308d54d2c706618206d79b6d5d56c8d5911

Changes applied:
  - Replaced unqualified "Neither exists" on the L1/4.00 GiB Inference sentence with wording that separates undelivered attention plan rungs (epilogue-fused n=2, handoff n=1) from already-landed governed fusion roles and `StorageHandoff` / `push_storage_handoff` program-dependency vocabulary.
  - Shortened the fusion-table citation anchor from the multi-line full sentence to the single-line raw fragment `complete set of families the governed provider`.

Optional items skipped (with reason):
  - none (report required no optional items; metadata none; dated correction not needed for in-place prose fixes).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave (Exact files product delivery under compiler/optimizer out of audit scope; no new remainder tickets required).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/plan-the-materialized-attention-decomposition/1aa3d1187784_c99ac54950f2.md
    - tickets/plan-the-materialized-attention-decomposition.md
    - crates/tiler-compiler/src/fusion_legality.rs (anchor fragment)
    - crates/tiler-ir/src/program (StorageHandoff / push_storage_handoff presence)
  - checks:
    - `rg -n 'complete set of families the governed provider' crates/tiler-compiler/src/fusion_legality.rs` → 1 match
    - `rg -n 'push_storage_handoff|StorageHandoff' crates/tiler-ir/src/program` → present
    - ticket no longer contains `Neither exists`
    - `shasum -a 256 tickets/plan-the-materialized-attention-decomposition.md` → 04a7bcbae18c61c2573f48baa1f6f308d54d2c706618206d79b6d5d56c8d5911

Recommended next ledger state:
  integrated
