Ticket: remove-the-loop-carried-redundant-staged-fold
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-loop-carried-redundant-staged-fold/d8fca12500ef_c99ac54950f2.md
Pre-edit content hash (from ledger): d8fca12500efcde587760c57722cb42ad10f0cab4e721f3b69ab2fe1fdfbd8dd
Post-edit content hash: bd4446a5887c6276f38dd25f3b9024818d2fd23587f372b1af0d5e3976b345fe

Changes applied:
  - related: added `realize-the-tiled-contraction-schedule-and-its-metal-emission`; kept closed `realize-the-strict-contraction-on-metal` as historical related only.
  - "What would remove it…": replaced false "OperationKind, OperationView, and the kernel identity grammar are all public" with accurate surface — OperationView public/`#[non_exhaustive]`/re-exported; OperationKind `pub(super)`; Predicated identity tag `0x18` load-bearing; Tom-decision conclusion retained.
  - Inference count: tightened from `(participants - 1) * participants * rounds` to `(participants - 1)² * rounds` combiner applications (emit_staged_fold seeds slot 0, loops `1..participants`).
  - Trigger log 2026-08-04: dropped stale `:755`/`:1289`; anchors `OperationKind::Predicated { predicate, body }` / `/// Executes a nested block when a predicate holds.`; noted line numbers stale at audit base `c99ac54950f2`.
  - Trigger log 2026-08-09: named live deferred owner `realize-the-tiled-contraction-schedule-and-its-metal-emission`; strict-contraction edge labeled historical closed/superseded.
  - Added `## Fact audit — 2026-08-10` dated summary of repairs.

Optional items skipped (with reason):
  - none (recommended Inference precision applied as cheap graph/prose hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none; Exact files listed only this ticket; no remainder filing required while deferred; product path (value-yielding Predicated / device measurement) remains activation work, not Phase B.

Verification:
  - files read:
    - tickets/remove-the-loop-carried-redundant-staged-fold.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-loop-carried-redundant-staged-fold/d8fca12500ef_c99ac54950f2.md
    - crates/tiler-ir/src/kernel/mod.rs (re-exports: OperationView yes, OperationKind no)
    - crates/tiler-ir/src/kernel/model.rs (`pub(super) enum OperationKind`, OperationView Predicated doc, identity tag 0x18)
    - crates/tiler-ir/src/kernel/lower.rs (`emit_staged_fold` start:1 end:participants)
    - tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md (status: deferred)
    - tickets/realize-the-strict-contraction-on-metal.md (status: closed, closed_reason: superseded)
  - checks:
    - shasum -a 256 tickets/remove-the-loop-carried-redundant-staged-fold.md → bd4446a5887c6276f38dd25f3b9024818d2fd23587f372b1af0d5e3976b345fe
    - status remains deferred (neither activation trigger fired; no status change required)

Recommended next ledger state:
  integrated
