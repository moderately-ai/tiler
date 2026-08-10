Ticket: size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them/8788ee083700_c99ac54950f2.md
Pre-edit content hash (from ledger): 8788ee083700cc25252bdcb1dda0607bbf6ebef8fdffaffb06356aec70cc2760
Post-edit content hash: 8b7055fedc5556604c0eb7c5e6982bfc5a37cec4c5f90bc8607b3e1c82a48435

Changes applied:
  - Under **Draft public surface (ADR 0075 — reported, not decided)**, added **Correction — 2026-08-10.** moving `CompileStage::ALL` from Excluded to Included: cites `crates/tiler-build/src/metal_cache.rs` `for stage in CompileStage::ALL` inside `stage_retention`, and that the reader landed in `7bd91ec9` (2026-08-05) before sizing repair `b3cd69c5` (2026-08-08). Leaves `AppleSdk::ALL` / `AppleSdk::COUNT` excluded. Does not re-decide the ADR 0075 boundary.
  - Metadata unchanged (status done; empty deps; related and scopes stay).

Optional items skipped (with reason):
  - none (report listed none optional)

Residuals not applied (docs/crates/new tickets/authority):
  - none — report required only ticket prose correction; no docs/crates edits, no new remainder ticket, no ADR decision.

Verification:
  - files read:
    - audit report 8788ee083700_c99ac54950f2.md
    - tickets/size-the-metal-aot-all-arrays-whose-count-coupling-does-not-rescue-them.md (full, pre- and post-edit)
    - crates/tiler-build/src/metal_cache.rs (stage_retention loop at `for stage in CompileStage::ALL`)
    - crates/tiler-metal-aot (rg: four ALL use variant_count; feature gate present)
  - checks:
    - `rg CompileStage::ALL crates/` → metal_cache.rs out-of-crate reader confirmed
    - `rg 'variant_count' crates/tiler-metal-aot` → four ALL sites + lib feature gate present
    - shasum -a 256 of ticket after edit → 8b7055fedc5556604c0eb7c5e6982bfc5a37cec4c5f90bc8607b3e1c82a48435

Recommended next ledger state:
  integrated
