Ticket: admit-vector-lane-bindings-into-the-schedule-vocabulary
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-vector-lane-bindings-into-the-schedule-vocabulary/db224d52f07b_c99ac54950f2.md
Pre-edit content hash (from ledger): db224d52f07bc15bb34db6dc3a053ebfa21eddb59723c260dcd8305fd43c8f0e
Post-edit content hash: 5bbb278038b27fe04ab4a123ba26129ee1bd9725abb52e0e0661d9e09553d800

Changes applied:
  - Implementation keys third bullet: replaced false "exact" `!*permits_reassociation` precedent with live shape — realization agreement plus `family.consumes_reassociation && !*permits_reassociation`, extrema exception, matching `verify_cooperative_semantics` / multi-pass (re-verified at builder.rs permission blocks).
  - Closes when: split schedule-owned intrinsic verdicts (map strict; A3; B2; B3; failure paths; encoder exhaustiveness) from A1/A2 target Proven/Rejected forks left to profile sibling / joint close.
  - related: added `declare-cpu-vector-realization-facts-in-the-target-profile` and `admit-lane-typed-values-and-masked-memory-into-the-kernel-ir` (optional graph hygiene from report).
  - **Correction — 2026-08-10.** one-liner noting the two prose repairs.

Optional items skipped (with reason):
  - none (optional related edges and dated correction applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation after Tom's ADR 0075 public-surface decision (model.rs / builder.rs / tests) — out of wave B scope.
  - Profile sibling owns A1/A2 target-feasibility composition; no new remainder filed (report: do not duplicate).
  - Open public-spelling residual (width type, IdentityPadded bits vs witness, layout field vs two variants) stays on Decision packet for Tom — no authority change.

Verification:
  - files read:
    - full audit report db224d52f07b_c99ac54950f2.md
    - full ticket admit-vector-lane-bindings-into-the-schedule-vocabulary.md (pre and post)
    - crates/tiler-ir/src/schedule/builder.rs (consumes_reassociation && !*permits_reassociation arms for multi-pass and cooperative)
  - checks:
    - rg `consumes_reassociation && !` in builder.rs → two live arms (multi-pass + cooperative) matching repaired wording
    - status left `awaiting-decision` (report: board-correct; no status change required)
    - shasum -a 256 post-edit ticket → 5bbb278038b27fe04ab4a123ba26129ee1bd9725abb52e0e0661d9e09553d800

Recommended next ledger state:
  integrated
