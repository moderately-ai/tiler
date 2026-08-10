Ticket: scope-search-state-caching-across-shape-families
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-search-state-caching-across-shape-families/00f9161a94a9_c99ac54950f2.md
Pre-edit content hash (from ledger): 00f9161a94a9c926221b1c81c9fa026ce56670eeb1bbeaa0dcf3897962e15607
Post-edit content hash: 7c96a8835d0ff9a3a44c38462506fdbd8c2fccf3bc098f10cba6a6f288014d30

Changes applied:
  - Why: replaced false present-tense deferral ("no search exists… formalism ticket decides") with current grounds (formalism selected and partially implemented; held by missing measured cold-search cost; memo vs e-graph still open).
  - Why: replaced "exact compilation identity" with complete expansion-cache / composed-subject identity (`ComposedSubject`).
  - Why: replaced "fall-open discipline, where the artifact cache falls closed" with ADR 0050 vocabulary (never serve wrong/unvalidated bytes; I/O fall open; search wrong-hit fall open to cold search).
  - related: added survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature, decide-whether-stage-one-semantic-exploration-adopts-an-e-graph, and design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature (cheap graph hygiene).

Optional items skipped (with reason):
  - 2026-08-10 trigger-check log line: not required for board correctness; 2026-08-09 still accurate.

Residuals not applied (docs/crates/new tickets/authority):
  - none (Exact files was ticket-only; no remainder filing while deferred).

Verification:
  - files read: full audit report; full ticket; status of survey (done), stage-one e-graph (deferred), feedback-tuning (done); grep anchors for fall open / ComposedSubject / full artifact identity across ADR 0050, expansion docs, key/store paths.
  - checks: shasum -a 256 on ticket post-edit; status: deferred unchanged; trigger still not fired.

Recommended next ledger state:
  integrated
