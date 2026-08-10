Ticket: derive-the-conformance-evidence-ledger-cells-from-executed-runs
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-conformance-evidence-ledger-cells-from-executed-runs/5e1abed1fdd2_c99ac54950f2.md
Pre-edit content hash (from ledger): 5e1abed1fdd27372cb07576d52d1dff128cf17901673740240b0e8183890ced3
Post-edit content hash: 893f8e4c556d64bab049998562232e232281ab19863ca8f9b3c514fa80c9ef49

Changes applied:
  - Replaced false "corrected to current evidence on 2026-08-09" with ledger restatement date 2026-08-07 vs trigger re-check date 2026-08-09.
  - Removed false "later vertical fired that ticket's own reconsideration trigger"; restated as vertical Closes when + 2026-08-07 ledger correction (decide ticket has no reconsideration trigger).
  - Softened "documentation has no automated validator" to AGENTS wording: manual maintenance; only `make citations`; maturity cells not auto-validated.
  - Restated L3 six-field comparison for present tense from `retained_record` (device, gpu-family, architecture, os, offline-compiler, sdk; xcode deliberately excluded); marked publish-ticket Xcode list as 2026-08-05 historical Outcome vocabulary.
  - Dropped public `tag()` claim on `ConformanceEvidenceClass`; noted private `const fn tag` as internal encoding only; kept public `ALL`, `spelling()`, `discharges_hard_requirement()`.
  - Status/deps/scopes/trigger left unchanged (report: metadata sound).

Optional items skipped (with reason):
  - Dated strike/history block for the false 2026-08-09 / reconsideration wording — report allows direct prose fix on still-open todo; false present-tense claims were rewritten rather than left searchable as struck history.
  - Optional related-graph hygiene — none required; related population already verified terminal and correct.

Residuals not applied (docs/crates/new tickets/authority):
  - Product work remains open on this ticket (compare-beside-ledger harness identity key, compare-vs-generate, evidence-enum reconciliation under ADR 0075) — not wave-B prose repair.
  - No new remainder ticket filed (report: none; open work already stated on this ticket).
  - No docs/ or crates/ edits (wave B ticket-only).

Verification:
  - files read:
    - tickets/derive-the-conformance-evidence-ledger-cells-from-executed-runs.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/derive-the-conformance-evidence-ledger-cells-from-executed-runs/5e1abed1fdd2_c99ac54950f2.md (full)
    - docs/dtype-support.md (anchors: Corrected 2026-08-07 restatement; crossing-neither cell; no 2026-08-09 ledger string)
    - tickets/decide-whether-the-bf16-conformance-evidence-cell-overstates.md (no Trigger/reconsideration)
    - crates/tiler-conformance/src/retained_record.rs (six-field compare; xcode deliberately not compared)
    - crates/tiler-ir/src/semantic/accuracy/evidence.rs (`pub` ALL/spelling/discharges; private tag)
    - AGENTS.md (manual docs + make citations only)
  - checks:
    - rg Confirm ledger correction date 2026-08-07 and cell bound text
    - rg Trigger|reconsideration on decide ticket → no matches
    - rg retained_record xcode/deliberate compare field names
    - rg evidence.rs public vs private methods
    - shasum -a 256 post-edit ticket → 893f8e4c556d64bab049998562232e232281ab19863ca8f9b3c514fa80c9ef49

Recommended next ledger state:
  integrated
