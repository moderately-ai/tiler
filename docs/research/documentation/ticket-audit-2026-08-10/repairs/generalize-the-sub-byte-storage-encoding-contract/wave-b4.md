Ticket: generalize-the-sub-byte-storage-encoding-contract
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/generalize-the-sub-byte-storage-encoding-contract/e278576b3cfc_c99ac54950f2.md
Pre-edit content hash (from ledger): e278576b3cfc08b7b7824fa9a9d50ee3498c49370de3f553564cc3913f47b6c8
Post-edit content hash: 05b1e98574d2034574974ed2e8129c74e4d5c0f6be9198a75edd67accce178a3

Changes applied:
  - Replaced the false live-Fact clause claiming U4 extract is checked by a test whose name ends in `_is_refused_on_the_measured_apple_profile` with the actual polarity: checked at the string level by `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile` (source contains `& 0x0fu`), absent from compiled goldens, never device-dispatched.
  - Added **Correction — 2026-08-10.** recording that the refused suffix was false; the measured-profile U4 test honours the normal-scale decode; historical `SubnormalFlushInArithmetic` refusal applied when a subnormal scale was still admitted (metal test comments).
  - Appended Trigger check log line 2026-08-10 restating not-fired after the prose fix, without reusing the false refused suffix.

Optional items skipped (with reason):
  - related[] hygiene for `scope-the-bit-reinterpretation-family-against-its-storage-carrier` — report marks optional graph hygiene only; deferred status and related list already coherent without it.

Residuals not applied (docs/crates/new tickets/authority):
  - Identical false `_is_refused_on_the_measured_apple_profile` phrasing in `measure-code-domain-integer-arithmetic-on-the-qualified-apple-row.md` — out of this ticket body; report leaves to coordinators.
  - Possible drift in `docs/dtype-support.md` Strict-affine U4/F32 honourability wording vs honoured normal-scale Metal test — adjacent evidence drift noted in audit residual uncertainty; not this ticket body.
  - Product work (generalized packing contract) remains the ticket itself once triggered; no remainder ticket filed.

Verification:
  - files read:
    - tickets/generalize-the-sub-byte-storage-encoding-contract.md (full, pre and post)
    - audit report e278576b3cfc_c99ac54950f2.md (full)
    - crates/tiler-metal/src/tests.rs around `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile` (honour path, string assert, historical refusal comments)
    - crates/tiler-metal/src/emit.rs `PackedExtractOp::U4LsbZeroTail` / `& 0x0fu` emission
  - checks:
    - rg `is_refused_on_the_measured_apple_profile` crates/ → empty
    - rg `strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile` crates/tiler-metal → 1
    - rg `0x0fu` / U4LsbZeroTail in tiler-metal confirms emit + string assert
    - sha256 post-edit: 05b1e98574d2034574974ed2e8129c74e4d5c0f6be9198a75edd67accce178a3

Recommended next ledger state:
  integrated
