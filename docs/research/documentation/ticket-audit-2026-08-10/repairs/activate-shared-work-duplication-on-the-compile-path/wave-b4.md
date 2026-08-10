Ticket: activate-shared-work-duplication-on-the-compile-path
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/activate-shared-work-duplication-on-the-compile-path/0c27ad281307_c99ac54950f2.md
Pre-edit content hash (from ledger): 0c27ad281307fdfc8b2812f91156df99fc94e94f8b592cd0801c977a11e2d55d
Post-edit content hash: 81107e21595e651f25ae63a4fcb32f26a8ead9fa0d61065434c3bfb63a0a3445

Changes applied:
  - related: added `derive-physical-proposals-from-the-cover-region-subject` and `assemble-a-kernel-program-from-an-arbitrary-cover` (both done; already cited in body)
  - Replaced false "one-line change at the single call site in `enumerate_complete_plans`" with both production `CoverPolicy::governed` sites (planning `enumerate_complete_plans` + verify `verify_portfolio`) and note that selection already receives the planning policy
  - Corrected Graph maintenance enforcers claim: no differing region guarantee/requirement pairs under bounded profile; not the enforcer restart case; keep re-read of live restart (production refused `NotSatisfied` / opaque registration)
  - Added 2026-08-10 trigger check log: **not fired** at `c99ac54950f2` with recheck anchors

Optional items skipped (with reason):
  - none (optional related list applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - stale "exactly three plan shapes" comments in `crates/tiler-compiler/src/cover.rs` (`CoverPolicy::governed`) and `crates/tiler-compiler/src/component_cost.rs` (RedundantWork note) — out-of-ticket crate prose drift; wave B does not edit crates
  - product activation itself remains deferred (vocabulary wall / measured recompute win) — not wave B work

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/activate-shared-work-duplication-on-the-compile-path/0c27ad281307_c99ac54950f2.md
    - tickets/activate-shared-work-duplication-on-the-compile-path.md
    - tickets/implement-boundary-property-enforcers.md (2026-08-09 live trigger anchor)
  - checks:
    - `rg -n 'CoverPolicy::governed' crates/tiler-compiler/src/pipeline/` → planning.rs + verify.rs
    - `rg -n 'exactly three plan shapes|one-line change|A duplicating plan makes' tickets/activate-shared-work-duplication-on-the-compile-path.md` pre-edit confirmed false living claims
    - enforcer ticket 2026-08-09 log: live trigger is production refused `NotSatisfied` handoff
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
