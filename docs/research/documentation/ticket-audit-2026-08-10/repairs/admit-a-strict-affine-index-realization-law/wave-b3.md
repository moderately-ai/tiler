Ticket: admit-a-strict-affine-index-realization-law
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-strict-affine-index-realization-law/9f3c65c1ee57_c99ac54950f2.md
Pre-edit content hash (from ledger): 9f3c65c1ee578c509757a09cc2540f6176fbcbac683293fd8f32a25c04aa52ef
Post-edit content hash: ad16a17c27194dc0d7c4beec72ac24e558f221a639238821699e4812cc975043

Changes applied:
  - Reframed opening `## Fact` present-tense census and "finds no strict-affine realization law" as pre-delivery problem statements (struck as live claims), with a 2026-08-10 Correction pointing at `## Accepted public boundary` / acceptance and a reproduce command for the live law and governed capability.
  - Marked the filing-time `## Inference` as design argument only (row no longer missing).
  - Optional Identity analysis dated note: delivery-time explain pin `fb0b64dd69649785` and residual ceiling `6,144` correct at acceptance; current sealed pin `7ba3d77a66f04638` and residual ceiling 12,288 (`6 * MAX_TENSOR_RANK * 2`).

Optional items skipped (with reason):
  - none (optional Identity pin/ceiling note applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report for this ticket (metadata sound; no remainder; no docs/crates edits listed as required).

Verification:
  - files read:
    - tickets/admit-a-strict-affine-index-realization-law.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-strict-affine-index-realization-law/9f3c65c1ee57_c99ac54950f2.md (full)
    - house-style samples: tickets/admit-the-structural-families-into-the-scheduled-region-vocabulary.md
  - checks:
    - `StrictAffineU4Dequantize` / `strict_affine_u4_dequantize` present under crates/tiler-ir/src/index (law.rs, scalar.rs, mod.rs)
    - sealed explain pin `tiler-explain-v7 request=7ba3d77a66f04638` in crates/tiler-compiler/src/explain.rs
    - `MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS = 6 * super::MAX_TENSOR_RANK * 2` in crates/tiler-ir/src/index/refinement.rs; `MAX_TENSOR_RANK = 1_024` in crates/tiler-ir/src/index/mod.rs
    - post-edit: `shasum -a 256 tickets/admit-a-strict-affine-index-realization-law.md` → ad16a17c27194dc0d7c4beec72ac24e558f221a639238821699e4812cc975043

Recommended next ledger state:
  integrated
