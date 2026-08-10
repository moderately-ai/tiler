Ticket: decide-whether-the-implementation-frontier-owes-a-retention-budget
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-the-implementation-frontier-owes-a-retention-budget/ce4cfc435c49_c99ac54950f2.md
Pre-edit content hash (from ledger): ce4cfc435c497693ee64fc8e21d4fc43bee6ad845335d207510c863d5eae6274
Post-edit content hash: 50bb00ff5b6d285a4c1e399681e0518de48437559a91fc49756873ace18acc6c

Changes applied:
  - Measurement triple: replaced bare "38 coverage gaps" with "14 `selection.region-coverage.v1` records whose blocked-cover counts sum to 38 (cover, region) pairs", aligned with live explain census pin (`selection.region-coverage.v1`, 14).
  - Dated correction note (2026-08-10) on the Measurement so a later reader does not re-inflate the pair count into a record count.
  - New Fact: plan selection binds full `frontier.admitted()` (`bind_region_frontiers`), not `non_dominated()`; `physical_plan_combinations` (governed 4096) already bounds complete-plan combinations; retention-budget decision must name which population it bounds.

Optional items skipped (with reason):
  - none (optional dated note for the 38 rewrite was applied as cheap hygiene on this ticket)

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/compiler/optimizer.md` stage-8 explain paragraph still phrases the parallel triple with "38 coverage gaps" — contract edit deferred (wave B ticket-only; report: align if this ticket's carrier edits contracts).
  - Declare-budget / unbounded close paths (request.rs field, frontier budget-stop, identity pins, optimizer ninth-budget + "bounded Pareto frontier") remain product work for Tom's decision.
  - Multi-provider admitted-population measurement remains outside this repair (ticket already marks measurement boundary).

Verification:
  - files read:
    - full audit report `ce4cfc435c49_c99ac54950f2.md`
    - full ticket pre-edit
    - `selection.rs` bind site (`admitted: entry.frontier.admitted()`)
    - `request.rs` `physical_plan_combinations` field + governed 4096
    - `pipeline/tests.rs` census pin `("selection.region-coverage.v1", 14)`
    - `hot_path.rs` `DISTINCT_SUBJECTS = 17`
  - checks:
    - rg admitted bind / physical_plan_combinations / region-coverage.v1 / DISTINCT_SUBJECTS against crates/tiler-compiler/src
    - sha256 post-edit ticket content

Recommended next ledger state:
  integrated
