Ticket: stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace/63e5585d359c_c99ac54950f2.md
Pre-edit content hash (from ledger): 63e5585d359cd219b40a19302a9b11da75f8e20bfaa3f31fe31c0021b692373e
Post-edit content hash: c86289bbe2c342e42efc0c54d14502cc2b54a492d0734b61e7417aebb76b4c0a

Changes applied:
  - Replaced false drop-commit pin `` `43f685f` `` with `` `a56bff8c` `` in the 2026-08-07 correction paragraph (prototype inheritance drop, 2026-07-25).
  - Same pin fix in Outcome "False Fact repaired" (`diverged at a56bff8c`).
  - Added **Correction — 2026-08-10.** recording the hash repair and that `43f685f` is ADR 0079's same-day unsafe-extent measurement pin, not the Cargo.toml drop.
  - Optional graph hygiene: added `pin-lint-inheritance-across-the-workspace-member-set` to frontmatter `related` for symmetry with that ticket and the Outcome body.

Optional items skipped (with reason):
  - none (optional related metadata and optional ADR 0079 note were both applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none required; report listed no docs/crates edits or new remainder tickets for this close condition

Verification:
  - files read:
    - tickets/stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/stop-the-conformance-crate-s-lint-table-drifting-from-the-workspace/63e5585d359c_c99ac54950f2.md
  - checks:
    - `git log -S 'unsafe_code = "deny"' -- prototypes/serial-sum-run/Cargo.toml` → `a56bff8c`
    - `git show 43f685f --stat` → ticket md only (seam-evidence record)
    - `git show a56bff8c -- prototypes/serial-sum-run/Cargo.toml` → removes `[lints] workspace = true`, adds restated `deny` table
    - post-edit: live drop pins are `a56bff8c`; `43f685f` remains only inside the 2026-08-10 correction prose that names the mis-pin

Recommended next ledger state:
  integrated
