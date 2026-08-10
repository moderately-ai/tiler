Ticket: define-the-widening-relation-over-a-symbolic-broadcast-extent
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/define-the-widening-relation-over-a-symbolic-broadcast-extent/ab63a7b363c6_c99ac54950f2.md
Pre-edit content hash (from ledger): ab63a7b363c6e8857080c71caf8f95462ae465fd5308d2380663c248e0045950
Post-edit content hash: a480ebe951ce1fb9b3d6acec57a0f9ad637607036a5e21ad048f043deb9b8b93

Changes applied:
  - Repaired the second Fact in "Why this exists": replaced stale mechanical greps (`broadcast`/`mapping` each 0; control `extent` 55) with re-verified counts at current main (`broadcast` 2, `mapping` 0, `extent` 65), kept the true structural claim that `result_extents` remains `Vec<Extent>`, and noted that the two `broadcast` hits are incidental post-2026-08-08 family-list/check-path tokens rather than a design of sourced mapping extents or a symbolic widening predicate.
  - Left status, dependencies, related, scopes, tags, decision packet, Do-nots, close condition, graph maintenance, and trigger log unchanged (report: metadata correct; board `awaiting-decision` still right).

Optional items skipped (with reason):
  - none (dated incidental-token note folded into the same Fact rewrite rather than a separate zero-grep retention).

Residuals not applied (docs/crates/new tickets/authority):
  - L6 D-19 prose still saying the carrier "holds it at `deferred`" while this ticket is `awaiting-decision` lives in `docs/research/program-planning/complete-model-ingestion-and-execution.md` (out of ticket-only wave B scope).
  - Product decision itself (Tom accept define-or-refuse path; mapping encoding / `broadcast-f32@1` version; lowering for degenerate widen; decoder count collapse or durable two-graph counts) remains open — not this repair.

Verification:
  - files read: audit report; full ticket; re-ran greps on `docs/research/shapes/symbolic-semantic-extents.md`; confirmed `result_extents: Vec<Extent>` still at `crates/tiler-ir/src/semantic/broadcast.rs`.
  - checks: pre-edit sha256 matched ledger `ab63a7b363c6…`; post-edit sha256 `a480ebe951ce…`; greps 2 / 0 / 65.

Recommended next ledger state:
  integrated
