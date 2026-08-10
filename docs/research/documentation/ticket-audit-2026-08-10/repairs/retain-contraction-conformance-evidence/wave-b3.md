Ticket: retain-contraction-conformance-evidence
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/retain-contraction-conformance-evidence/f1d56481e48d_c99ac54950f2.md
Pre-edit content hash (from ledger): f1d56481e48de15c08a44efd2595fb7cda9dc78468bd039e6745faa5ef6704b7
Post-edit content hash: 9c49366e0dc1014cbbc0c9611bb25ee3559d6b4d00eed9969a7f5a9cef0c6bd8

Changes applied:
  - Amended `## Closes when` so the coverage-statement sentence is no longer a close condition of this ticket (implementation halves + decline discipline only).
  - Added **Correction — 2026-08-10.** under Closes when stating that the coverage ledger clause was rehomed on 2026-08-09 to `state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract` and is not a residual of this ticket.

Optional items skipped (with reason):
  - none (the report's recommended close-authority hygiene was applied rather than leaving Closes when intact and only dating under Outcome).

Residuals not applied (docs/crates/new tickets/authority):
  - none — metadata (status/deps/related/scopes) required no change; remainder ticket already connected; no docs/crates edits.

Verification:
  - files read:
    - tickets/retain-contraction-conformance-evidence.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/retain-contraction-conformance-evidence/f1d56481e48d_c99ac54950f2.md (full)
    - tickets/state-the-contraction-conformance-corpus-coverage-against-the-reduction-contract.md (frontmatter + body through Closes when; remainder id and depends-on this ticket confirmed)
  - checks:
    - remainder ticket id resolves; status `todo`; `dependencies: [retain-contraction-conformance-evidence]`
    - post-edit sha256 via `shasum -a 256 tickets/retain-contraction-conformance-evidence.md`

Recommended next ledger state:
  integrated
