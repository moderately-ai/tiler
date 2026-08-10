Ticket: scope-a-value-domain-precondition-for-squaring-overflow
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-a-value-domain-precondition-for-squaring-overflow/e26868559496_c99ac54950f2.md
Pre-edit content hash (from ledger): e268685594964911b8cfaf6e48c9962b2526746b7898880ee4e2c80726b75f55
Post-edit content hash: 9c94db6e1bd892c1dbb4f14e4a1e6719dfa4d829ea927c7e6b787d016f84e0bb

Changes applied:
  - Corrected the live Fact that equated *reaching* `0x5f7fffff` / `RMS_NORM_F32_SQUARING_OVERFLOW_BITS` with squaring overflow: overflow begins for magnitudes strictly above that constant (largest binary32 with a finite square); at the constant itself the square is finite and the row normalizes nonzero; corpus already covers threshold, successor, and above-threshold row.
  - Added `**Correction — 2026-08-10.**` under that Fact citing `the_squaring_overflow_threshold_is_the_last_argument_whose_square_is_finite` and the reference threshold/successor asserts.
  - Metadata left unchanged (status deferred, empty deps, related, scopes) per audit.

Optional items skipped (with reason):
  - Optional dual-wording note on `RMS_NORM_F32_SQUARING_OVERFLOW` / constant docs in `crates/tiler-ir/src/semantic/rms_norm.rs` ("largest … finite" vs "reach it produces +inf"): report marks it out of ticket scope for this wave; residual product/docs debt only.

Residuals not applied (docs/crates/new tickets/authority):
  - `crates/tiler-ir/src/semantic/rms_norm.rs` constant and `RMS_NORM_F32_FACT_SQUARING_OVERFLOW` docs still carry the same dual wording tension ("reach" the bits constant vs last finite-square threshold). Wave B forbids crates/docs edits; no new remainder ticket filed (report: none required).

Verification:
  - files read:
    - tickets/scope-a-value-domain-precondition-for-squaring-overflow.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-a-value-domain-precondition-for-squaring-overflow/e26868559496_c99ac54950f2.md (full)
    - crates/tiler-ir/src/semantic/rms_norm.rs (threshold constant + fact-field doc around RMS_NORM_F32_SQUARING_OVERFLOW_BITS)
    - crates/tiler-ir/src/semantic/rms_norm/tests.rs (`the_squaring_overflow_threshold_is_the_last_argument_whose_square_is_finite`)
    - crates/tiler-reference/src/rms_norm/tests.rs (`a_row_above_the_squaring_overflow_threshold_normalizes_to_signed_zeros` threshold/successor asserts)
  - checks:
    - `RMS_NORM_F32_SQUARING_OVERFLOW_BITS = 0x5f7f_ffff`; IR test asserts finite square at constant, non-finite at +1
    - reference test: normalize at threshold nonzero; at successor and at 1e20 signed zeros
    - post-edit: `shasum -a 256 tickets/scope-a-value-domain-precondition-for-squaring-overflow.md` → 9c94db6e1bd892c1dbb4f14e4a1e6719dfa4d829ea927c7e6b787d016f84e0bb

Recommended next ledger state:
  integrated
