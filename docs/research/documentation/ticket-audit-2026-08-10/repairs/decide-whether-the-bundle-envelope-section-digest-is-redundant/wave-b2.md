Ticket: decide-whether-the-bundle-envelope-section-digest-is-redundant
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-bundle-envelope-section-digest-is-redundant/c063445bfe35_c99ac54950f2.md
Pre-edit content hash (from ledger): c063445bfe35f67c53043ea31acf16d17175cc5c710d18c19aab645a9cd096bd
Post-edit content hash: a66c2e62f3632b9a6d035746e40bdb2aef2e2b1d59e59e3f4c78ed5a05778e60

Changes applied:
  - Graph maintenance: `Section 9's third outcome` → `Section 10's third outcome` (hot-path-efficiency.md Outcomes).
  - Dated correction block after Graph maintenance: `**Correction, 2026-08-10 — Section number.**` stating the answer lives in Section 10 item 3, not Section 9 (Section 9 is the re-run at the re-derived band).

Optional items skipped (with reason):
  - Clarify Outcome "Three results retained" vs later fourth `…-delivered-realization.tsv`: report marks not required for correctness of the decision; Outcome wording remains accurate as this ticket's retained close set.

Residuals not applied (docs/crates/new tickets/authority):
  - none (report listed only ticket prose; metadata already sound; no new remainder).

Verification:
  - files read:
    - tickets/decide-whether-the-bundle-envelope-section-digest-is-redundant.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-bundle-envelope-section-digest-is-redundant/c063445bfe35_c99ac54950f2.md
    - docs/research/cache/hot-path-efficiency.md (section headings + headline citation of Section 10's third outcome)
  - checks:
    - `rg -n '^## 9\.|^## 10\.' docs/research/cache/hot-path-efficiency.md` → §9 re-derived band, §10 Outcomes
    - note headline: `see Section 10's third outcome`
    - ticket Graph maintenance now says Section 10; dated correction present
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
