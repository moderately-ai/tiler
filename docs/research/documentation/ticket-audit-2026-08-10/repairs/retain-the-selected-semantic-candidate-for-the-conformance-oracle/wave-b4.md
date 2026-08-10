Ticket: retain-the-selected-semantic-candidate-for-the-conformance-oracle
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/retain-the-selected-semantic-candidate-for-the-conformance-oracle/bc48d07e2473_c99ac54950f2.md
Pre-edit content hash (from ledger): bc48d07e247380cb4d43accbdd6240a144a529995108dd30093acb01f941613c
Post-edit content hash: e2d374cd96a002cb463d3ca896571aac9fc3f9f616ef54d1864f622526d90030

Changes applied:
  - In-scope first bullet: removed the false implication that the design Outcome already sites the accessor; now states design 1 selected retention only, siting is Tom's via this ticket's Option A/B (or custom), and a new public surface stays a labelled draft under ADR 0075.
  - Why-this-exists: refreshed the SemanticProgram-accessor absence pin from `5cec07d0` to `07226d1e` (current tree) and noted the same empty result at audit base `c99ac549`; command form aligned to `rg`.

Optional items skipped (with reason):
  - none (optional pin refresh applied; no dated correction block — in-place rewrite was the report's preferred path)

Residuals not applied (docs/crates/new tickets/authority):
  - Product work remains: Tom picks A/B (or custom siting), implement retention, repair composition-site provenance, add close-condition tests; no docs/crates edits in this wave.
  - Future Option A surface still needs ADR 0075 labelled-draft + acceptance node when implemented (not this wave).

Verification:
  - files read:
    - tickets/retain-the-selected-semantic-candidate-for-the-conformance-oracle.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/retain-the-selected-semantic-candidate-for-the-conformance-oracle/bc48d07e2473_c99ac54950f2.md
    - tickets/decide-how-a-pinned-pointwise-grouping-becomes-evaluable.md (Outcome anchor: public boundary still Tom's)
  - checks:
    - `rg 'pub fn .*SemanticProgram' crates/tiler-compiler/src` → zero matches at 07226d1e
    - pre-edit shasum matched ledger `bc48d07e247380cb4d43accbdd6240a144a529995108dd30093acb01f941613c`
    - post-edit shasum `e2d374cd96a002cb463d3ca896571aac9fc3f9f616ef54d1864f622526d90030`
    - metadata left unchanged (status awaiting-decision, scopes, related, empty dependencies)

Recommended next ledger state:
  integrated
