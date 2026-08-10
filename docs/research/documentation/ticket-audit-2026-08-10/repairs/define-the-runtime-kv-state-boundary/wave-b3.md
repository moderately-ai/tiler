Ticket: define-the-runtime-kv-state-boundary
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/define-the-runtime-kv-state-boundary/47f3aa6a20db_c99ac54950f2.md
Pre-edit content hash (from ledger): 47f3aa6a20db175078b10d5bda089a1cf12163e6ed8fd86230a1f9c7a1cba6c7
Post-edit content hash: da02793efb0d75d62b7158c98cc45b575ecd0c4452310057a20d7c46232925c2

Changes applied:
  - In "### The preserved draft", replaced "three independent API reviews at `fc242fd1`, `59b0e4d8`, and `dca26e5a`" with one linear draft sequence ending at `dca26e5a` (exact-live authority → model/artifact split → dependent alignment), the 2026-08-04 architecture stop that freezes that tip, and the named branch as a related but non-identical lineage (shared base `0c54d5e8`, remote tip `488daa97`) that also carries draft `docs/integration/runtime-state.md`.
  - Added **Correction — 2026-08-10** that `git for-each-ref --contains dca26e5a` is empty at audit base `c99ac54950f2`, so durable preservation of the Tom-named tip needs an explicit ref if GC-safety matters.

Optional items skipped (with reason):
  - none (optional dated for-each-ref note applied as same-ticket evidence hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - Ops ref pinning `dca26e5a` (outside ticket prose; report names this as optional evidence hygiene, not a ticket file change).
  - No docs/ or crates/ edits required by the report.
  - No remainder tickets; do not reopen; do not re-file runtime KV ownership.

Verification:
  - files read: audit report; tickets/define-the-runtime-kv-state-boundary.md; git object graph for the draft SHAs and branch tip.
  - checks: `git rev-parse origin/tkt/define-the-runtime-kv-state-boundary` → `488daa97…`; merge-base with HEAD not ancestor; `git log --oneline 59b0e4d8^..dca26e5a` is the three-commit linear chain; `git for-each-ref --contains dca26e5a` empty; merge-base of branch tip and `dca26e5a` is `0c54d5e8…`; post-edit `shasum -a 256 tickets/define-the-runtime-kv-state-boundary.md`.

Recommended next ledger state:
  integrated
