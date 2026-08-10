Ticket: repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows/9b0edc7a3971_c99ac54950f2.md
Pre-edit content hash (from ledger): 9b0edc7a39711df6888b7e38af1c4246642a657aa4392aff8f1f82982e5e782d
Post-edit content hash: c925cc1fa70813ea4d8d909107473ed8a11ab6daed7de6c236de18ec13633975

Changes applied:
  - Outcome — done, 2026-08-08: replaced false merge identity `cb56bf8e` with true land `f1760a42` (worker `23b09e62` unchanged).
  - Added `## Fact audit — 2026-08-10` dated correction recording that `cb56bf8e` was the next linear commit (citation-links filing) and that the true merge is `f1760a42`.
  - Optional graph hygiene: `related: []` → `related: [resolve-the-markdown-links-the-citation-check-cannot-see]` (ticket exists, status done; Outcome already links it).

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket. Report explicitly says do not file 0108 off-alpha topic placement here; no docs/crates edits in wave B.

Verification:
  - files read:
    - full audit report 9b0edc7a3971_c99ac54950f2.md
    - full ticket repair-adr-0107-s-duplicated-and-mislabelled-decision-catalog-rows.md (pre- and post-edit)
    - frontmatter of tickets/resolve-the-markdown-links-the-citation-check-cannot-see.md
  - checks:
    - `git log -1 --format='%H %P %s' f1760a42` → merge parents 209013bd + 23b09e62, subject Repair ADR 0107's duplicated and mislabelled catalog rows
    - `git log -1 --format='%H %P %s' cb56bf8e` → parent f1760a42, subject File the markdown links the citation check cannot see
    - both 23b09e62 and f1760a42 are ancestors of HEAD
    - post-edit sha256 via shasum -a 256

Recommended next ledger state:
  integrated
