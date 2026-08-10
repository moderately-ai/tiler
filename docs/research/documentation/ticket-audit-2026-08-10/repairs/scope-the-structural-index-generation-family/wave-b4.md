Ticket: scope-the-structural-index-generation-family
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-structural-index-generation-family/1cb2e3cc6e40_c99ac54950f2.md
Pre-edit content hash (from ledger): 1cb2e3cc6e4037eeea719ee43a8e424cb8628d9486b5b1d1de63cfa71df5c892
Post-edit content hash: b6e707e509395cf3e1d52158bd4df0349c588035105d543e902ec668746439d2

Changes applied:
  - Replaced join-table count "twenty-three" with "twenty-five" on the live Fact (F-02 remains one of the no-row set).
  - Re-ran the ticket's `rg … | sort -u` census at current main: 47 distinct source literals (not 46); restated recheck with that total and kept F-02/iota/arange absence.
  - Dropped "eighteen registered operation keys"; restated from `StandardSemantics::register` walk as nineteen operations (including gather and three strict-affine ops), noting the regex is not a registry walk (`format!` normative strings, underscore MX names).
  - Added **Correction — 2026-08-10** for the twenty-three→twenty-five join-table drift and the retired 46/eighteen census composition.
  - Applied the same census repair on the 2026-08-05 trigger-log line (struck 46/eighteen; pointed at live recheck).
  - Added 2026-08-10 trigger-check log line: **not fired** after census repair.
  - Metadata left unchanged (status deferred, dependencies, related, scopes).

Optional items skipped (with reason):
  - Optional F-34 join-table no-row vs Indirect gather matrix-row remainder: out of this ticket's Outcome; report said optional separate remainder only.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by Repair required (Exact files: ticket only).

Verification:
  - files read: tickets/scope-the-structural-index-generation-family.md; audit report 1cb2e3cc6e40_c99ac54950f2.md; docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md (join table cell + "Twenty-five of forty-seven"); crates/tiler-ir/src/semantic/registry.rs (StandardSemantics::register); crates/tiler-ir/src/semantic/quantization.rs (three strict-affine ops via format!); semantic/ greps for iota/arange/index-generation (no hits).
  - checks: counted 25 F-nn tokens in *(no matrix row today)* cell; `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` → 47 lines; no F-02 key in semantic/; status remains deferred.

Recommended next ledger state:
  integrated
