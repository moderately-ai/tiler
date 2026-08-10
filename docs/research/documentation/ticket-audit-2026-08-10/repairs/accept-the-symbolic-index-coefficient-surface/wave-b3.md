Ticket: accept-the-symbolic-index-coefficient-surface
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-symbolic-index-coefficient-surface/45ef570e4de5_c99ac54950f2.md
Pre-edit content hash (from ledger): 45ef570e4de5e41b24007b04ba971e662a38e05ff11eae979e61a392bac370e7
Post-edit content hash: 0281a6e6508a54e72e3be51a21e7035b1df029ef07836263e889e2c60d03f173

Changes applied:
  - frontmatter `related`: added `bound-a-symbolic-index-coefficient-interval-from-its-declared-extent` (optional graph discoverability)
  - dated section `## Residual post-acceptance hygiene — 2026-08-10` recording that draft labels and `docs/ir.md` interval/draft claims were not flipped at close; lists the three residual sites from the audit (rustdoc draft labels + false constant pending; assemble/interval-decline comments; `docs/ir.md` coefficient paragraph / v11 lag)
  - status / dependencies / scopes left unchanged (report: none required)
  - acceptance Outcome and 2026-08-07 interval-ground correction left intact (report: read true)

Optional items skipped (with reason):
  - none — both optional ticket items (related link; dated residual note) applied as cheap same-ticket hygiene

Residuals not applied (docs/crates/new tickets/authority):
  - Flip "Draft surface, not yet accepted" on `SourcedIndexInteger`, `sourced_linear_combination`, `LinearTermRef::coefficient` (and stop listing `IndexExprView::LinearCombination` constant as pending) — crates paths out of wave scope
  - Rewrite interval-decline claims in those rustdocs and assemble comment to match `interval_linear` + bound ticket — crates paths out of wave scope
  - Update `docs/ir.md` coefficient paragraph (interval no longer declines-by-policy; half accepted 2026-08-07; domain currently v11) — docs path out of wave scope
  - New remainder ticket: report requires a narrow remainder or attach to existing hygiene ticket but lists no concrete id; blocked residual (no id decision in this wave). Verified residual sites still present: `Draft surface, not yet accepted` in `crates/tiler-ir/src/index/{sourced,builder,model}.rs`; assemble comment "interval propagation declines on the same terms" in `builder.rs`; `docs/ir.md` still states interval decline. Bound ticket status is `done`. `correct-the-symbolic-coefficient-era-index-vocabulary-claims` is a different (done) population (divisor-only coordinate claims), not this draft-label/interval residual.

Verification:
  - files read:
    - full audit report `.../45ef570e4de5_c99ac54950f2.md`
    - full ticket `tickets/accept-the-symbolic-index-coefficient-surface.md` (pre/post)
    - `tickets/bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md` frontmatter (`status: done`)
    - grep `Draft surface, not yet accepted` under `crates/tiler-ir/src/index` (hits sourced.rs, builder.rs, model.rs; predicate-module labels left as audit residual uncertainty / bound-v11 chain)
    - grep interval/draft phrasing in `docs/ir.md` (coefficient half still declines)
    - assemble comment anchors in `crates/tiler-ir/src/index/builder.rs`
    - `crates/tiler-ir/src/index/sourced.rs` draft rustdoc span (constant field still listed pending)
    - skim of `correct-the-symbolic-coefficient-era-index-vocabulary-claims` (done; different remainder population)
  - checks:
    - pre-edit sha256 matched ledger pin `45ef570e4de5e41b24007b04ba971e662a38e05ff11eae979e61a392bac370e7`
    - post-edit sha256 `0281a6e6508a54e72e3be51a21e7035b1df029ef07836263e889e2c60d03f173` via `shasum -a 256`
    - acceptance and 2026-08-07 correction prose preserved
    - no crates/docs edits; no new ticket filed

Recommended next ledger state:
  integrated
