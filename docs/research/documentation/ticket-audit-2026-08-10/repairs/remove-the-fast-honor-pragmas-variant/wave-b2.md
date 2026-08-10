Ticket: remove-the-fast-honor-pragmas-variant
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-fast-honor-pragmas-variant/045dfe506655_c99ac54950f2.md
Pre-edit content hash (from ledger): 045dfe5066559b26f653e200fdea425b0c18ea56a6e5fa83c3c0c9ad1c698bce
Post-edit content hash: c2242785e1cc3bbde64a08c68d1308b6e4da64af2920f8664e45ef7678182024

Changes applied:
  - Outcome "Out-of-crate consumers": corrected the live rg command from `rg -n 'FpContract'` (eight lines at current base: three imports + five uses) to `rg -n 'FpContract::'` so the stated five-site count matches the five named construction/compare anchors.

Optional items skipped (with reason):
  - Optional dated correction one-liner: not applied; report says optional when the command pattern is fixed in place, which it was.

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required no docs/crates/new-ticket work)

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/remove-the-fast-honor-pragmas-variant/045dfe506655_c99ac54950f2.md
    - tickets/remove-the-fast-honor-pragmas-variant.md
  - checks:
    - `rg -n 'FpContract' crates/ prototypes/ -g '!**/tiler-metal-aot/**'` → 8 lines
    - `rg -n 'FpContract::' crates/ prototypes/ -g '!**/tiler-metal-aot/**'` → 5 lines (golden_compilation Off compare + Off construction; metal_assembly Off compare + two Fast constructions)

Recommended next ledger state:
  integrated
