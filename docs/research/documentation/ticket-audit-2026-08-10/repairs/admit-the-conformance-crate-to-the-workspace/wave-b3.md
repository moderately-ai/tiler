Ticket: admit-the-conformance-crate-to-the-workspace
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-the-conformance-crate-to-the-workspace/390ed5231e42_c99ac54950f2.md
Pre-edit content hash (from ledger): 390ed5231e4247e7079faa251ca68b77abe4721afee595d12ba8373ce366fcfc
Post-edit content hash: a557470f49291c3daeffe1cd807d24005365db66cce372a3d6acf10d8c7d7c9e

Changes applied:
  - Four authorities intro: reattributed atomic crate-admission mapping obligation from `AGENTS.md` to `ticketsplease.toml` (above `[scope_crates]`).
  - Added **Correction — 2026-08-10.** documenting the mis-citation as never true at any commit (substitute-style), with reproduce anchors; left Authority 4's true AGENTS.md lint-inheritance note alone.

Optional items skipped (with reason):
  - Soft-date "hosts nothing yet" / inherited-lints bullets as admission-time requirements: report marks not strictly required for a done Outcome already dated at `5d31fd03`; no live-board ambiguity warrants prose churn.

Residuals not applied (docs/crates/new tickets/authority):
  - Architecture packaging-block omission of live `tiler-cache` dep vs admission-time edge list: report assigns to architecture/ADR maintenance, not this ticket's remainder.
  - No crate/docs/graph metadata changes required.

Verification:
  - files read: audit report; full ticket; `ticketsplease.toml` atomic-admission comment; AGENTS.md (no atomic phrase; lint inheritance note present at inheritance is not enforced).
  - checks: `rg 'must atomically add' AGENTS.md` empty (exit 1); same fragment hits ticketsplease.toml; false `AGENTS.md states that a crate-admission` gone from ticket; `shasum -a 256` post-edit.

Recommended next ledger state:
  integrated
