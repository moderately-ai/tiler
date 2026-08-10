Ticket: realize-the-tiled-contraction-schedule-and-its-metal-emission
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/realize-the-tiled-contraction-schedule-and-its-metal-emission/135361571a98_c99ac54950f2.md
Pre-edit content hash (from ledger): 135361571a98c0c11ec972d839db560956dc6ef35d8d594dd831958506094172
Post-edit content hash: 131e9e998a18226325a97474d4ca078529870e23dacc1f5084de7bdeb364f696

Changes applied:
  - Replaced Numerical legality section: live text now states L3 `tiled` preserves ascending left fold / memory-schedule-only, attributes to `strict_fold+ftz`, consumes no numerical permission; `direct` and `tiled` both strict-admissible; drops FLUSH_AND_REASSOCIATE_F32 as warrant and re-scopes `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio` as a parallel-sum fixture not a warrant for this schedule.
  - Added **Correction — 2026-08-10.** withdrawing the reassociation-consumption claim with anchors to L3 "consuming no permission" and kernels.metal "changes the memory schedule and nothing about the reduction".
  - Multi-round Metal emission bullet: acknowledges KIR `emit_loop_carried_cooperative` multi-round reduction emission; landed goldens single-round; two-allocation contraction multi-round body remains this ticket's.
  - Metadata unchanged (status deferred, dependencies, related, scopes, trigger log).

Optional items skipped (with reason):
  - none (report listed no optional repairs)

Residuals not applied (docs/crates/new tickets/authority):
  - Product work remains blocked on `admit-a-cooperative-tile-over-shared-operands` (awaiting-decision); no remainder ticket required by this audit.
  - Proposed `ReductionTopology` append at `0x36` and Metal emission implementation are out of wave B scope (crates/docs).

Verification:
  - files read: ticket; audit report; L3 first-metal-contraction-realizations.md (legality rows / strict_fold+ftz attribution); spikes/.../kernels.metal (memory-schedule comment); lower.rs (`emit_loop_carried_cooperative`); tiler-metal/goldens listing
  - checks: L3 row `tiled` | Yes, consuming no permission; kernels.metal "changes the memory schedule and nothing about the reduction"; shasum -a 256 post-edit

Recommended next ledger state:
  integrated
