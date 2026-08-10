Ticket: honour-require-metal-toolchain-in-the-aot-driver-tests
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/honour-require-metal-toolchain-in-the-aot-driver-tests/db1f2e8573d0_c99ac54950f2.md
Pre-edit content hash (from ledger): db1f2e8573d0574e16a1578125a5794b7763b73420365294db16ca06c2c0d287
Post-edit content hash: 7be7b952e863ddffb9e011de5fc6a286725f7b0092dc2d265a0ecccd7f570b7d

Changes applied:
  - Under `## Why this exists`: added `**Correction — 2026-08-10.**` stating the two Fact sentences and resolve-site line citations are filing-time only; defect closed by Outcome via `resolved_system_toolchain`; `TILER_REQUIRE_METAL_TOOLCHAIN` now hits both metal packages; only two live resolve call sites remain.
  - Past-tensed the second Fact (`**Fact — at filing, before this ticket landed.**`) and the defect-framing paragraph (had/skipped/ignored) so they are not live claims about this base; marked the six resolve line numbers as historical.

Optional items skipped (with reason):
  - none (report’s optional one-liner is subsumed by the dated correction + past-tense rewrite).

Residuals not applied (docs/crates/new tickets/authority):
  - none (metadata already sound; no remainder ticket; no docs/crates edits required).

Verification:
  - files read:
    - tickets/honour-require-metal-toolchain-in-the-aot-driver-tests.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/honour-require-metal-toolchain-in-the-aot-driver-tests/db1f2e8573d0_c99ac54950f2.md
    - crates/tiler-metal-aot/src/driver.rs (REQUIRE_TOOLCHAIN, resolved_system_toolchain, five call sites, two resolve sites)
    - crates/tiler-metal/src/golden_compilation.rs (REQUIRE_TOOLCHAIN still present)
  - checks:
    - `rg -n 'TILER_REQUIRE_METAL_TOOLCHAIN|resolved_system_toolchain|toolchain\.resolve\(AppleSdk::MacOs\)' crates/tiler-metal-aot/src/driver.rs` → helper + five sites + two resolve calls
    - `rg -c 'TILER_REQUIRE_METAL_TOOLCHAIN' crates/` → hits both golden_compilation.rs and driver.rs
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
