Ticket: measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order/ea023238d221_c99ac54950f2.md
Pre-edit content hash (from ledger): ea023238d22125e60b445215edb279cf604e70ed53b167b7c4f8c9f2a27f3516
Post-edit content hash: e2d7c4574ee6aaf3aa85eb6f171809f8b1f61fb5e111c0f2e2fde90a9d9b6d16

Changes applied:
  - Replaced line-number pins in "Why this exists" with searchable anchors: `MetalNumericalRequirement::flag` in `record.rs`; tests-module phrase for the ffp-contract defence (dropped `:1311`); `MetalTargetFacts` / `CapabilityAxis` by symbol without `:755` / `:211`.
  - Replaced wrong `golden_compilation.rs:584` pin in "The bounded experiment" with `realization_honours` and related selection checks in that file.
  - Rewrote Outcome "Three edits follow…" into an **At landing (2026-08-06)** historical note plus **Correction — 2026-08-10.**: catalogs present; Part 7 item 5 measured; roll-up still stale filed/`todo`; MetalTargetFacts/CapabilityAxis still fieldless while compiler `declare_evaluation_order_preservation` vocabulary exists with profile row still `Unknown` on the profile toolchain.

Optional items skipped (with reason):
  - none — optional dated correction applied as the same Correction block (cheap hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/reference/permitted-divergence-oracle.md` four-outcome roll-up row still **Bounded experiment, filed** / ticket `todo` — outside this ticket's scopes (`research/reference`).
  - Profile measured row still `Unknown` by design until same-toolchain remeasure (Tom-authorized); not a ticket-prose fix.

Verification:
  - files read:
    - tickets/measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order/ea023238d221_c99ac54950f2.md (full)
    - crates/tiler-metal/src/record.rs (flag arms for NoFloatingPointContraction / SafeMathMode)
    - crates/tiler-metal/src/target.rs (`MetalTargetFacts` five fields)
    - crates/tiler-metal/src/tests.rs (defence phrase)
    - crates/tiler-metal/src/golden_compilation.rs (`realization_honours` / NoFloatingPointContraction selection)
    - crates/tiler-compiler/src/target/feasibility.rs (`CapabilityAxis` / CANONICAL_AXES)
    - docs/research/reference/permitted-divergence-oracle.md (Part 7 item 5 measured; roll-up still filed/todo)
    - spikes/README.md, docs/research/README.md (evaluation-order probe catalog rows present)
  - checks:
    - no remaining `:NNNN` line pins to record.rs / tests.rs / target.rs / feasibility.rs / golden_compilation.rs in the ticket
    - Correction block states measured Part 7 item 5, present catalogs, stale roll-up residual
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
