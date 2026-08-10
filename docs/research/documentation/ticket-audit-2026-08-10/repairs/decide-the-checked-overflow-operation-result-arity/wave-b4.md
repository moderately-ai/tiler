Ticket: decide-the-checked-overflow-operation-result-arity
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-the-checked-overflow-operation-result-arity/b7ec363b8f37_c99ac54950f2.md
Pre-edit content hash (from ledger): b7ec363b8f378966b1d0121a0c9a404de31c17d552df6d21a26dc1614eaa3d7a
Post-edit content hash: a85b7d0d21508b50d65e45971b3b3153304e8b2dc9014a376d0487548ed9d911

Changes applied:
  - Reframed User-visible outcome, work description, and Closes when: ADR 0039 already selects multi-result for checked (wrapped low bits + overflow predicate); residual is worked consumer program under that shape plus research-record reconciliation, not a free binary "choose a shape."
  - Added **Correction — 2026-08-10** under Why this is deferred: `RQ-OP-01`'s one-result + precondition shape is required-no-overflow, not a second checked arity; supersession of ADR 0039 is Tom-owned.
  - Explicit non-goals now name checked multi-result (and the other three families' result counts) as decided by ADR 0039; added non-goal forbidding unilateral supersession.
  - Graph: moved parent from `dependencies:` to `related:` (activation coupling, not completion edge); `dependencies: []`; Graph maintenance dated correction explains the deadlock-shaped completion risk.
  - Added `## Fact audit — 2026-08-10` with per-claim verdicts from the audit.
  - Trigger check log: 2026-08-10 **not fired** (parent deferred; no checked integer registry family; gather index identity only; ADR vs RQ-OP-01 prose defect independent of activation).
  - Status left `deferred` (activation not fired; report requires keep deferred).

Optional items skipped (with reason):
  - none — optional dependency/related frontmatter fix applied while editing.

Residuals not applied (docs/crates/new tickets/authority):
  - Taxonomy `RQ-OP-01` / F-08 D2 still frames checked arity as open (`docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md`) — docs residual outside ticket-only wave.
  - Dtype-track join table `RQ-OP-01` row (`docs/research/numerics/dtype-family-research-tracks.md`) if still open-framed — same residual.
  - Worked multi-result consumer program under ADR 0039 when a checked workload appears — product residual, activation-gated.
  - Closing solely on "already decided by ADR 0039" deferred until taxonomy/join-table language is repaired (ticket Closes when now requires that reconciliation).

Verification:
  - files read: audit report; full ticket (pre/post); ADR 0039 (checked + Required-no-overflow Decision bullets); parent ticket status/non-goal; gather.rs index-identity anchor; taxonomy RQ-OP-01 row.
````text
  - checks: parent `status: deferred`; anchors `plus an overflow predicate as explicit results`, `Returns the one admitted index-operand identity`, `Does a checked-overflow integer operation return one result`, `RQ-OP-01`'s arity question for a checked-overflow operation`; content hash via `shasum -a 256`.
````

Recommended next ledger state:
  integrated
