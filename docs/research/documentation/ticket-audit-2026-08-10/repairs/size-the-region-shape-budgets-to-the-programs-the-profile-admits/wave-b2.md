Ticket: size-the-region-shape-budgets-to-the-programs-the-profile-admits
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/size-the-region-shape-budgets-to-the-programs-the-profile-admits/4ff25656e138_c99ac54950f2.md
Pre-edit content hash (from ledger): 4ff25656e138700170c238920b7bf65ecbe9bb85a37a0e5b87ccbd3a1e0a881c
Post-edit content hash: 1279ed2a941bfa5ef0c2c98f5d3826c2dbad0ddac0214f16e620f282942f8617

Changes applied:
  - Reframed "## The decision" opening: present-tense bare constants 32/8/64 and "region_members is 32" marked as historical open-decision (pre-derive) framing; semantic_operations 62 sizing sentence kept as filing-time fact.
  - Corrected request-subject injectivity claim: budget value changes still do not step encoding; live domain tag is `tiler.compiler.request-subject.v6` (unrelated SemanticIdentity shape-environment step), not live `v5`.
  - Added **Correction — 2026-08-10.** after Released work: derive ticket landed; live region-shape budgets are derived 62/3/80; v6 domain note; Decided section left intact as decision-time history.
  - Optional graph hygiene: added the three Released work ids to `related` frontmatter.

Optional items skipped (with reason):
  - none (optional related-list hygiene applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this node; open follow-ons already live as `state-the-rule-that-a-deterministic-budget-is-a-derivation` and `decide-whether-a-derived-budget-belongs-in-the-request-subject`. No docs/crates edits (wave B ticket-only). Source comments in `request.rs` still mention v5 in places — residual product prose debt outside ticket scope.

Verification:
  - files read:
    - audit report `…/4ff25656e138_c99ac54950f2.md`
    - ticket `tickets/size-the-region-shape-budgets-to-the-programs-the-profile-admits.md` (pre/post)
    - `crates/tiler-compiler/src/request.rs` (governed body + domain tag via grep)
  - checks:
    - `region_members: 62`, `region_boundary_outputs: 3`, `region_live_values: 80` present in `governed()`
    - encoder writes `tiler.compiler.request-subject.v6\0`
    - post-edit `shasum -a 256` of ticket file

Recommended next ledger state:
  integrated
