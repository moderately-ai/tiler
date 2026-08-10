Ticket: execute-the-stateful-prefill-path
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/execute-the-stateful-prefill-path/fb5f165c7096_c99ac54950f2.md
Pre-edit content hash (from ledger): fb5f165c70964f335bad7c700fd4191dd7de9bf931a715253592f48222e73489
Post-edit content hash: 047034d00f8711d4cff541ce866445e9fd84d6ef19e1401e4be0346ba422e8c3

Changes applied:
  - Required behaviour first bullet: replaced pre-correction absolute ("twenty-two steps are identical and only the bound extents differ") with cache-axis same-program claim at `C = 0` (`a_nonempty_cache_changes_no_occurrence`), retained cache-axis P1 elimination unaffected by 2026-08-04 supersession, and explicit non-claim of identity across `T = 10` prefill and `T = 1` decode (L5/L6 D-19 / `define-the-widening-relation-over-a-symbolic-broadcast-extent`)
  - Added **Correction — 2026-08-10** citing `decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode` and L5 anchors (`narrowed, not withdrawn`; `a_nonempty_cache_changes_no_occurrence`); close still `T = S = 10` only
  - Adjacent Required behaviour / Closes when bullets left in substance; unwrapped mid-paragraph hard wraps only on lines rewritten for the same edit

Optional items skipped (with reason):
  - none (report required metadata none; optional remainder none)

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave (report Exact files on repair: ticket only; delivery under `implementation/candle` remains product work outside wave B)

Verification:
  - files read: full audit report; full ticket; L5 `narrowed, not withdrawn` / 2026-08-05 correction; `decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode` measurement; `a_nonempty_cache_changes_no_occurrence` / `a_single_new_position_changes_six_widenings` present in `crates/tiler-reference/tests/decoder_layer.rs`
  - checks: `shasum -a 256 tickets/execute-the-stateful-prefill-path.md`; status/deps/scopes left unchanged per report

Recommended next ledger state:
  integrated
