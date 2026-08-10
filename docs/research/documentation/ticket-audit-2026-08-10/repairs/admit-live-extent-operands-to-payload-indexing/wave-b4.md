Ticket: admit-live-extent-operands-to-payload-indexing
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-live-extent-operands-to-payload-indexing/37a205b089cd_c99ac54950f2.md
Pre-edit content hash (from ledger): 37a205b089cdd95053d1c4b4cb2caf8a0abe708ddc9b035d77ddd8e9cc72cb1e
Post-edit content hash: 3c08515b93dbffd808f4fbd83fc943a83b667c6c7e9caab06462367787d379b6

Changes applied:
  - Removed over-declared `contracts/integrations` from `scopes` (no frontend-integration contract ownership on this ticket).
  - Documented transitive kernel-identity-only path for payload compilation subject; left `implementation/metal-aot` off scopes unless a direct `tiler-metal-aot` edit is required.
  - Corrected Fact prose: `place_bindings` → `RoutedBinding::{accessible_offset, accessible_bytes}`; launch attributed to `evaluate_launch` / `RoutedLaunch` on `RoutedEntry::launch`.
  - Re-pinned Fact with re-verified base `c99ac54950f242d88d8dfe8335332bef0cf75f2d` (kept original pin `b4e3478d…`).
  - Added dated **Correction — 2026-08-10** and **Scopes note — 2026-08-10** blocks with reproduction command.
  - Status left `todo` (capability still unimplemented; close condition unmet).

Optional items skipped (with reason):
  - none (optional one-line dated place_bindings correction applied as house-style dated block).

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation of live extent operand, Metal emit/binding, artifact row, runtime transport, identity step (Exact files list in audit) — out of wave B scope.
  - Residual uncertainty on whether a future worker must edit `tiler-metal-aot` directly remains a worker-time scope decision recorded in the Scopes note, not a board repair blocker.
  - No remainder ticket filing required by the report.

Verification:
  - files read: audit report; ticket; `crates/tiler-runtime/src/load.rs` (`place_bindings`, `evaluate_launch`); `crates/tiler-runtime/src/load/route.rs` (`RoutedBinding`, `RoutedLaunch`, `RoutedEntry`); `docs/research/runtime/dynamic-kv-physical-layout.md` place_bindings wording; `ticketsplease.toml` scope names; house-style dated corrections on sibling tickets.
  - checks: re-verified place_bindings builds only `RoutedBinding` offset/bytes; launch is separate `evaluate_launch` → `RoutedLaunch`; sha256 post-edit of ticket file.

Recommended next ledger state:
  integrated
