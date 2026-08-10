Ticket: measure-apple-numerics-on-physical-ios-device
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-apple-numerics-on-physical-ios-device/38d1dce928ae_c99ac54950f2.md
Pre-edit content hash (from ledger): 38d1dce928ae763c4c3b5339a7bcce9b055c1183890f4e45451d759761fe4942
Post-edit content hash: 45f00d8f5fbcbca7aa2acfb228aaddeb3150945c4954b588c5fe5fdb852afcbf

Changes applied:
  - Replaced stale line citations `:461`, `:342`, and `:388` (Scope paragraph) and the Who-does-what citation of `:388` with searchable anchors into `docs/research/apple-targets/numerical-behaviour.md` (`leaves open for all three dtypes`; finding 26 Measurement clause `only dispatched device row, and … Unknown for both iOS families`; `two GPUs wide on the device side`).
  - Softened opening "all three `MetalPlatform` families" to "the three artifact families the numerical probe measures (`MacOs`, `IOsDevice`, `IOsSimulator`)" (enum is wider at current tree).
  - Left status `deferred`, empty dependencies, related list, and scopes unchanged (report: metadata sound).
  - No new trigger-log row (live USB not measured in this repair).

Optional items skipped (with reason):
  - Optional dated correction block noting line-citation drift — not required once prose uses anchors (report: optional if rewritten).

Residuals not applied (docs/crates/new tickets/authority):
  - Residual open expectation that honourability documentation name this ticket as reopen trigger lives on research/contract prose relative to already-`done` `declare-metal-numerical-honourability` — out of band for this ticket (report: no new dependency edge).
  - Activation product work (device runner, retained results, numerical-behaviour findings 11/13/24/26, measurement-boundary update) remains deferred until hardware attaches; not wave-B ticket repair.

Verification:
  - files read: audit report; ticket (full); grep anchors in `docs/research/apple-targets/numerical-behaviour.md`; `MetalPlatform` variants in `crates/tiler-metal/src/target.rs`
  - checks: post-edit ticket has no `:NNN` line citations and no bare `MetalPlatform` equate-to-three claim; anchors `leaves open for all three dtypes`, `two GPUs wide on the device side`, and finding 26 `Unknown` / both iOS families wording present in numerical-behaviour.md; sha256 recomputed after edit

Recommended next ledger state:
  integrated
