Ticket: scope-the-ieee-decimal-vertical
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-ieee-decimal-vertical/2e662f5954bc_c99ac54950f2.md
Pre-edit content hash (from ledger): 2e662f5954bc5125cde780d46fba302062a3024a57db36b7c23991c629965259
Post-edit content hash: ae507f5aac8b46d8a62806932e5d1cf9b132821fbc3f61612196f964a473321b

Changes applied:
  - related: added `state-the-non-enumerable-float-conformance-profile` (optional graph hygiene for the prose consumption claim; not elevated to depends-on)
  - Why-this-exists third Fact: rephrased from "consumes … answer rather than deriving its own" to state D-7 reuses D-3's bounded-measurement evidence-class methodology once landed (D-3 is binary f16/f64/f128 only), and must consume a landed profile or derive decimal128's bounded universe explicitly
  - 2026-08-04 Trigger check log: dropped stale `:187` line citation; replaced with searchable fragment `#### D-7 — IEEE decimal32`; not-fired substance and 2026-08-09 entry unchanged
  - status left deferred; dependencies left []; no remainder ticket filed

Optional items skipped (with reason):
  - none (optional related edge applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave; activation still requires naming DPD-versus-BID for program bytes and either consuming a landed D-3 profile framework or deriving decimal128's bounded universe (product work, not ticket prose)

Verification:
  - files read: tickets/scope-the-ieee-decimal-vertical.md (full pre/post); audit report 2e662f5954bc_c99ac54950f2.md (full)
  - checks: shasum -a 256 tickets/scope-the-ieee-decimal-vertical.md → ae507f5aac8b46d8a62806932e5d1cf9b132821fbc3f61612196f964a473321b; frontmatter related list includes D-3 id; no depends-on added; status remains deferred

Recommended next ledger state:
  integrated
