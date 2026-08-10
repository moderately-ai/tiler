Ticket: pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase/51506fe858ca_c99ac54950f2.md
Pre-edit content hash (from ledger): 51506fe858ca602ad7dcc8b5d2c447d9077c0a4c8fe681ff9c820a5dfab3f97f
Post-edit content hash: f041a94b29f84a7dfb4527bba06a726a8f88948d55b26ef6fa550914a2ba1f79

Changes applied:
  - frontmatter `related: []` → `related: [pin-the-sdk-provenance-on-the-compile-profile-ledger-reproduce-block]` (remainder already names this ticket)
  - Outcome land hash `58cfe3d9` → `806d421b` (merge that parents worker `a050b0b5`); kept worker commit; dated correction notes false `58cfe3d9` was next mainline (sourced-shape) and outcome prose commit `12825292`
  - dated correction on opening "Five records… two… three…" as filing-time census; live `25F70` census is three in `docs/status.md` and four in `backend-scoped-route-requirement-answers.md`

Optional items skipped (with reason):
  - none (optional related edge applied as accurate graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none (no docs/crates edits; remainder already exists and is done)

Verification:
  - files read:
    - tickets/pin-the-sdk-provenance-the-xcrun-reproduce-forms-silently-rebase.md
    - audit report 51506fe858ca_c99ac54950f2.md
    - git show --stat 58cfe3d9 / 806d421b / a050b0b5; parents of 806d421b; 12825292 subject
    - rg 25F70 on docs/status.md and backend-scoped-route-requirement-answers.md
    - remainder ticket path present
  - checks:
    - 58cfe3d9 is sourced-shape only; 806d421b parents a050b0b5 and matches this ticket's tree
    - 25F70: 3 status.md, 4 backend-scoped (imprecise opening partition)
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
