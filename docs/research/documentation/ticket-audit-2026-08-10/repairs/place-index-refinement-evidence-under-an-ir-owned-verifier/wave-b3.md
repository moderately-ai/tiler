Ticket: place-index-refinement-evidence-under-an-ir-owned-verifier
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/place-index-refinement-evidence-under-an-ir-owned-verifier/c7ccc92cdcc2_c99ac54950f2.md
Pre-edit content hash (from ledger): c7ccc92cdcc2c9d9063f84f9e0176498af1174c03ac8cca79d04a42c51eab8a6
Post-edit content hash: 9f354a2f8bf6d7a72180924725a1f0b982ca22df595537ac6a4a1d545fdc0007

Changes applied:
  - Rewrote present-tense **Unsupported case** from f32-only templates to closed multi-template law vocabulary (f32/bf16, staged, concatenate, strict affine u4, etc.); widening remains a reviewed law/template boundary.
  - Marked fourth fixed-point residual ceiling `3 * 1024 * 2 = 6,144` as historical-at-landing; stated live `MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS = 6 * MAX_TENSOR_RANK * 2` (12,288 at rank 1024) with staged five-access + margin-six rationale.
  - Annotated every live-looking pin `3a2bda87fc26f899` as historical 2026-08-03 measurement; named current sealed pin `7ba3d77a66f04638`.
  - Soft-corrected acceptance prose “closed F32 law vocabulary” → multi-template (not f32-only).
  - Added `## Fact audit — 2026-08-10` covering (1)–(3) with reproduce anchors. Metadata unchanged (status/deps/related/scopes already coherent).

Optional items skipped (with reason):
  - none (report listed no optional ticket prose beyond the three required items).

Residuals not applied (docs/crates/new tickets/authority):
  - none; Exact files listed only this ticket; no remainder ticket; no docs/crates product edits.

Verification:
  - files read:
    - tickets/place-index-refinement-evidence-under-an-ir-owned-verifier.md (full, pre- and post-edit)
    - reports/.../c7ccc92cdcc2_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/refinement.rs (MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS + comment)
    - crates/tiler-ir/src/index/mod.rs (MAX_TENSOR_RANK)
    - crates/tiler-ir/src/index/law.rs (IndexRealizationLaw variants head)
    - crates/tiler-compiler/src/explain.rs (sealed pin 7ba3d77a66f04638 via grep)
  - checks:
    - `MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS = 6 * super::MAX_TENSOR_RANK * 2`; `MAX_TENSOR_RANK = 1_024`
    - explain pin `"tiler-explain-v7 request=7ba3d77a66f04638\n"`
    - `shasum -a 256 tickets/place-index-refinement-evidence-under-an-ir-owned-verifier.md` → 9f354a2f8bf6d7a72180924725a1f0b982ca22df595537ac6a4a1d545fdc0007

Recommended next ledger state:
  integrated
