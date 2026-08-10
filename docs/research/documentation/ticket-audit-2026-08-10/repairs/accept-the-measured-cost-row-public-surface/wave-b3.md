Ticket: accept-the-measured-cost-row-public-surface
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-measured-cost-row-public-surface/4fb2a670a2dd_c99ac54950f2.md
Pre-edit content hash (from ledger): 4fb2a670a2dda1c86fa88d15b00e8bc3e582eb775ae19bca0841802d2fc3e8c7
Post-edit content hash: 70c24fe4363d14555ec75396ffa26f79caeb0ef1a33e083d39bd96f278043196

Changes applied:
  - Removed obsolete frontmatter tag `needs-tom` (Tom accepted spelling 2026-08-07; status already `done`).
  - Added `retire-the-draft-label-on-the-accepted-cost-row-surface` to `related` for bidirectional graph completeness with the released draft-label carrier.

Optional items skipped (with reason):
  - Evidence one-clause rename `select_global_non_dominated` → measured path inside `select_non_dominated`: optional and not load-bearing; not graph hygiene; left as historical Evidence wording (IMPRECISE name, substance holds per audit Fact 14).
  - Dated live-pin note on Identity: report says none required when Identity is read as landing history; retire ticket and `metal_plan.rs` already carry the post-landing ladder.

Residuals not applied (docs/crates/new tickets/authority):
  - Draft-label + optimizer/cost-model/ledger draft sentences remain owned by open `retire-the-draft-label-on-the-accepted-cost-row-surface` (todo); not re-opened on this acceptance ticket.
  - Product paths named for that remainder only: `crates/tiler-compiler/src/target.rs`, `docs/compiler/optimizer.md`, `docs/compiler/cost-model.md`, authority ledger draft sentence, embedding ladder — out of scope for wave B ticket-only edit.

Verification:
  - files read:
    - audit report `…/reports/accept-the-measured-cost-row-public-surface/4fb2a670a2dd_c99ac54950f2.md` (full)
    - ticket `tickets/accept-the-measured-cost-row-public-surface.md` (full, pre- and post-edit)
  - checks:
    - frontmatter: `tags` no longer contains `needs-tom`; `related` includes both activate and retire ids
    - `shasum -a 256` post-edit: `70c24fe4363d14555ec75396ffa26f79caeb0ef1a33e083d39bd96f278043196`

Recommended next ledger state:
  integrated
