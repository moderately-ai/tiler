Ticket: restate-the-single-region-realization-docs-after-the-sequence-widening
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/restate-the-single-region-realization-docs-after-the-sequence-widening/281dd2c31370_c99ac54950f2.md
Pre-edit content hash (from ledger): 281dd2c3137036b9c649388a30380820034210f05f27378a7b2ca9599c19af7b
Post-edit content hash: 281dd2c3137036b9c649388a30380820034210f05f27378a7b2ca9599c19af7b

Changes applied:
  - none (report required none for ticket metadata, Outcome prose, and dated correction; in-scope IR restatement already landed; status `done` remains correct)

Optional items skipped (with reason):
  - `related:` edges for remainder tickets: skipped because those tickets are not filed in this wave (needs concrete id decision); report marks adding them optional only when filed
  - Optional oracle scalar-capability census ownership: not established as still false; out of ticket scope
  - Why-section line numbers left historical: Per-Fact audit already supersedes them (report optional hygiene: leave as-is)

Residuals not applied (docs/crates/new tickets/authority):
  - blocked residual — new ticket needed (suggested theme `restate-region-formation-registered-law-count-after-sequence-widening`): `crates/tiler-compiler/src/region.rs` still carries `// The cheap filter first: nine of the ten registered laws are` while standard registry registers fifteen laws (two staged among them). Scope `implementation/compiler`. Wave B forbids creating new ticket ids.
  - blocked residual — new ticket needed (suggested theme `restate-legality-module-header-for-sequence-realizations`): `crates/tiler-compiler/src/legality.rs` module header still says `this authority proves refinement of one occurrence to one index region` while `refine_index_region` drives `verify_sequence` and `single_region` refuses chains. Scope `implementation/compiler`. Wave B forbids creating new ticket ids.
  - optional residual: dedicated `implementation/reference` audit of oracle scalar-capability census vs installed rows (Fact 14 unowned; not re-derived this wave)
  - product residual (below reopen threshold): minor `verify_sequence` `# Errors` enumeration gap vs full `check_lowering_authority` set (`SubjectRealizationLawMismatch`, helper-emitted `ScalarSnapshotMismatch`) — report left unfixed deliberately
  - no crates/docs edits in this wave (region.rs / legality.rs product debt above)

Verification:
  - files read:
    - tickets/restate-the-single-region-realization-docs-after-the-sequence-widening.md (full)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/restate-the-single-region-realization-docs-after-the-sequence-widening/281dd2c31370_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/region.rs (stale nine-of-ten comment still present)
    - crates/tiler-compiler/src/legality.rs (module header one-occurrence-to-one-region still present)
    - crates/tiler-ir/src/index/law.rs (enum + `realizes_region_sequence` three staged arms)
    - crates/tiler-ir/src/semantic/registry.rs (standard law registration list: 15 rows including both staged constructors)
  - checks:
    - pre/post ticket sha256 unchanged: 281dd2c3137036b9c649388a30380820034210f05f27378a7b2ca9599c19af7b
    - grep `nine of the ten registered laws are` → region.rs hit
    - grep `one occurrence to one` → legality.rs module header hit
    - no ticket under `tickets/` owns the region-formation or legality-header remainder themes

Recommended next ledger state:
  integrated
