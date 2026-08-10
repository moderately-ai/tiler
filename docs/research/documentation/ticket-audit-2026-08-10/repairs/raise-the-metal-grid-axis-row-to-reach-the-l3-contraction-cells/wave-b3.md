Ticket: raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells/9b974a69dabf_c99ac54950f2.md
Pre-edit content hash (from ledger): 9b974a69dabfafc6446ab0062f5b319a48f40ee571a8725a4662b2190bb402f5
Post-edit content hash: 0883fd1d746bfa372fdbbb93f0a9fd150496cd788dd5aef12bd634e46ebec3ae

Changes applied:
  - Replaced Outcome line citation `crates/tiler-build/src/metal_declaration.rs:225` with line-free anchor `FIRST_MACOS_APPLE9` in `crates/tiler-build/src/metal_declaration.rs` + `grid_axis_threads: 268_435_456` (value verified at current line 254).
  - Optional related hygiene: added `publish-an-l3-contraction-cell-through-the-accepted-route` so the remainder edge is visible from both ends.
  - Optional pin-tense clarity: dated **Correction — 2026-08-10** that the Outcome's 1,999-byte descriptor and grid-row-only identity triple are historical (base `561dfe0b`), not live pins; measured row value remains.
  - Optional four-thread runner prose: dated **Correction — 2026-08-10** that the `proof.rs` closing line was present at Outcome time and has since been corrected by `correct-the-four-thread-grid-rationales-the-measured-row-falsified` / `publish-an-l3-contraction-cell-through-the-accepted-route` (struck false present-tense "prints").

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report for this ticket; report listed no docs/crates product edits. Residual uncertainty from the audit (device re-run, historical bind-stage empty-diff, live pin vs ledger drift outside this ticket) left as audit residual, not ticket prose debt.

Verification:
  - files read:
    - audit report `.../9b974a69dabf_c99ac54950f2.md`
    - ticket `tickets/raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md`
    - `crates/tiler-build/src/metal_declaration.rs` (confirmed `grid_axis_threads: 268_435_456` under `FIRST_MACOS_APPLE9`)
  - checks:
    - `rg 'grid_axis_threads:\s*268_435_456' crates/tiler-build/src/metal_declaration.rs` → field present
    - `shasum -a 256 tickets/raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md` → 0883fd1d746bfa372fdbbb93f0a9fd150496cd788dd5aef12bd634e46ebec3ae

Recommended next ledger state:
  integrated
