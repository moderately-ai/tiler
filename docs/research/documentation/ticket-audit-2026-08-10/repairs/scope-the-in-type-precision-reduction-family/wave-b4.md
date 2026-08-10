Ticket: scope-the-in-type-precision-reduction-family
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-in-type-precision-reduction-family/ead97fe3bb02_c99ac54950f2.md
Pre-edit content hash (from ledger): ead97fe3bb02d7351441d0fa42eeb0ca6dc7c493685bc83d97317a0e93b0f324
Post-edit content hash: 54d953a2da44d532e2ba33ba73fcdd944f12e76d534fe418c113c3d95a8dc3bd

Changes applied:
  - Replaced live Fact "twenty-three families the join table lists under *(no matrix row today)*" with "twenty-five families" (taxonomy join cell + 2026-08-05 correction).
  - Refreshed live Fact governed-key census: 47 unique `tiler::…@N` keys under `crates/tiler-ir/src/semantic/`, nineteen registered operation keys including `tiler::gather-f32@1`; family's key still absent.
  - Added 2026-08-10 trigger-check log entry: **not fired**, updated census, same-type kernel transforms are CanonicalizeF32Nan and CanonicalizeBf16Nan (neither F-21); marks prior "only CanonicalizeF32Nan" / "46 / eighteen" / "twenty-three" wording as superseded.

Optional items skipped (with reason):
  - none (report listed no optional repair bullets; metadata already correct).

Residuals not applied (docs/crates/new tickets/authority):
  - none (Exact files expected only this ticket; no remainder ticket or docs/crates edits required).

Verification:
  - files read:
    - tickets/scope-the-in-type-precision-reduction-family.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-in-type-precision-reduction-family/ead97fe3bb02_c99ac54950f2.md
    - docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md (join cell, F-21, twenty-five correction)
    - crates/tiler-ir/src/kernel/model.rs (ConvertOp CanonicalizeF32Nan / CanonicalizeBf16Nan)
  - checks:
    - `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` → 47 keys; no reduce_precision key
    - `pub fn *_op()` under semantic → 19 ops including gather_f32_op
    - join-table `*(no matrix row today)*` F-nn token count → 25; F-21 present

Recommended next ledger state:
  integrated
