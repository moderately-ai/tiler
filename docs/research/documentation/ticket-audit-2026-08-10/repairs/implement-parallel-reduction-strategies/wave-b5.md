Ticket: implement-parallel-reduction-strategies
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/implement-parallel-reduction-strategies/8c25854c6448_c99ac54950f2.md
Pre-edit content hash (from ledger): 8c25854c64489e3ad2cd20334d9792728405b3123496f2bd2e34c2556033abe7
Post-edit content hash: 2d96ae44dd689c281f52f7b2d88205add7d6aae1342e925fa5e44e4c5a7575c4

Changes applied:
  - Outcome "No census exists to update": left original false "tiler-ir does not enable feature(variant_count)" wording; **Correction — 2026-08-10** records test-gated `#![cfg_attr(test, feature(variant_count))]` on tiler-ir and that the load-bearing claim is still "no inventory over ScheduledRegionDiagnostic".
  - Outcome unsupported-cases / "general accumulation contract is still owned here": left original wording; **Correction — 2026-08-10** moves residual D-5/D-6 general widening policy off this `done` rollup onto a new remainder (criteria 1–7 stay closed).
  - Class D required remainder filed: `tickets/decide-the-general-accumulation-width-contract-for-reductions-and-contractions.md` (`status: todo`, research/contracts scopes, no product code); parent `related` wired to that id.
  - Metadata: status remains `done`; dependencies unchanged.

Optional items skipped (with reason):
  - Dependency-notes live `:3681` line-cite hygiene before the 2026-08-08 `:4262` section — leave-original-with-correction policy already has the anchor-grep repair in the 2026-08-08 Fact audit; optional only.
  - Optional remainder for `ScheduledRegionDiagnostic` variant/rule census sized from the type — Outcome already marks unowned; not required by Repair required.

Residuals not applied (docs/crates/new tickets/authority):
  - Research docs still assign open D-5/D-6 closure solely to this done id (`docs/research/numerics/transformer-nonlinear-normalization-and-reductions.md`, `docs/research/scheduling/first-metal-contraction-realizations.md`, L4 vertical as needed) — Exact files list; wave B forbids docs edits; remainder Closes-when / Graph maintenance own retarget after decision.
  - Enforcers restart-condition restatement — owned by `implement-boundary-property-enforcers` (report: no action on this ticket beyond existing log).
  - Full suite / `make full` green counts at audit base — not re-run (report residual).

Verification:
  - files read: full audit report; full parent ticket pre-edit; `crates/tiler-ir/src/lib.rs` variant_count attr (via grep across crates); L3′ D-5 / L3 D-6 ownership anchors in research docs; sample narrow todo remainder for frontmatter shape.
  - checks: `grep -n 'feature(variant_count)' crates/tiler-ir/src/lib.rs` matches test-gated enable; remainder `status: todo`; parent related includes new id; two 2026-08-10 Correction blocks present; post-edit `shasum -a 256` on parent ticket.

Recommended next ledger state:
  integrated
