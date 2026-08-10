Ticket: resolve-the-generated-facade-path-under-crate-renaming
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/resolve-the-generated-facade-path-under-crate-renaming/454672db3abc_c99ac54950f2.md
Pre-edit content hash (from ledger): 454672db3abc98a39d2260051397cb52c4ac48fd7c96691cb2de1a17d1e5d46e
Post-edit content hash: 45ae53a10f222585310e4164da0e8dd96ccbc919ff11ca9761cce731477f95e7

Changes applied:
  - Dropped rotten line numbers from the 2026-08-04 "stale citation corrected" Trigger check log entry (`:105`, `:113`, `:116`, `:119`, `lib.rs:103`, `aot.rs:270`); rewrote that entry with searchable anchors only (`FACADE_ENTRY_PATH` / siblings, ticket-id comment above `FACADE_ENTRY_PATH`, `RouteFacts::source` absolute-path emission in `aot.rs`).
  - Added 2026-08-10 Trigger check log entry recording that those line citations were themselves stale at c99ac549… and that anchors remain the authority.
  - Tightened Graph maintenance (optional, cheap scope truth): named `binding.rs`, `aot.rs`, and `delivery.rs` as additional absolute-path emission sites beside the four `FACADE_*` constants in `lib.rs` and the facade fixtures; cache-key separation into `implementation/cache` unchanged.
  - Left status/deps/scopes/related untouched (report: board metadata correct; trigger 1 unmet; trigger 2 retired).

Optional items skipped (with reason):
  - none (optional Graph maintenance scope tighten applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none for this repair pass; product mechanism choice remains deferred under trigger 1; no crates/docs edits authorized in wave B; no new remainder ticket required by the report

Verification:
  - files read:
    - tickets/resolve-the-generated-facade-path-under-crate-renaming.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/resolve-the-generated-facade-path-under-crate-renaming/454672db3abc_c99ac54950f2.md (full)
    - crates/tiler-macros/src/lib.rs (FACADE_* constants + ticket id comment via rg)
    - crates/tiler-macros/src/aot.rs, binding.rs, delivery.rs (absolute `::tiler::__private` emission sites via rg)
  - checks:
    - `rg -n 'const FACADE_ENTRY_PATH|resolve-the-generated-facade-path-under-crate-renaming' crates/tiler-macros/src/lib.rs` → ticket id at comment above constant; constants present
    - `rg -n '::tiler::__private' crates/tiler-macros/src` → emission also in binding.rs, aot.rs, delivery.rs
    - `shasum -a 256 tickets/resolve-the-generated-facade-path-under-crate-renaming.md` → 45ae53a10f222585310e4164da0e8dd96ccbc919ff11ca9761cce731477f95e7

Recommended next ledger state:
  integrated
