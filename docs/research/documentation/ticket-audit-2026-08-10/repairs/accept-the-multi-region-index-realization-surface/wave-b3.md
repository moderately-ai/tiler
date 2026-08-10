Ticket: accept-the-multi-region-index-realization-surface
Wave: B3
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-multi-region-index-realization-surface/a084803afb6a_c99ac54950f2.md
Pre-edit content hash (from ledger): a084803afb6a1622b1fedbba08db6ac925603b4393edf9ced459d5b0b80c1d82
Post-edit content hash: f8393f50c0ab81df8dad9b8094b2acfa7688d5810aa0899b74c9bd82b5e040f9

Changes applied:
  - Appended `## Correction — 2026-08-10` noting that Exact-surface `StagedIntermediate` omits live `retained_through` (sibling multi-reader acceptance; field still on the named IR surface).
  - Same section marks the Choice claim that normalization's reciprocal square root "does not exist" as historical: `rsqrt_f32_scalar_op` exists, RMS uses `StagedRootMeanSquareScaleF32` with registry rows for rms-norm/softmax; original `StagedStrictSerialSumThenPointwiseF32` still has no standard registry row.
  - Metadata left unchanged (status, deps, related already accurate per report).
  - No further rename prose (2026-08-09 follow-on already covers `final_stage` / `final_scalar_authority`).

Optional items skipped (with reason):
  - none (report prose correction was labeled optional/recommended; applied as cheap same-ticket truthfulness)

Residuals not applied (docs/crates/new tickets/authority):
  - none (report Exact files: ticket-only; no docs/crates remainder; multi-value handoff already deferred elsewhere)

Verification:
  - files read:
    - full audit report a084803afb6a_c99ac54950f2.md
    - full ticket accept-the-multi-region-index-realization-surface.md (pre-edit)
    - crates/tiler-ir/src/index/sequence.rs (`StagedIntermediate` + `retained_through`)
    - grep: `rsqrt_f32_scalar_op`, `StagedRootMeanSquareScaleF32`, `StagedStrictSerialSumThenPointwiseF32` under crates/ and related docs
    - tickets/accept-the-multi-reader-index-realization-retention.md (path exists)
  - checks:
    - `retained_through: usize` on `StagedIntermediate` and `retained_through()` accessor present
    - `rsqrt_f32_scalar_op` re-exported from tiler-ir index and used by compiler governed path
    - sibling multi-reader ticket path resolves

Recommended next ledger state:
  integrated
