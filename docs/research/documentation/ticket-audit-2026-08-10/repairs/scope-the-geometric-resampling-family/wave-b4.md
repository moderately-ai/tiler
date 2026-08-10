Ticket: scope-the-geometric-resampling-family
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-geometric-resampling-family/44a9212784dd_c99ac54950f2.md
Pre-edit content hash (from ledger): 44a9212784ddacd162d0fceb3d0c7f31d21af6e7ee6a70c0bdfb86257210c1de
Post-edit content hash: 19e8e6a3a2fdce663fae68168523dcc20b6052efb3c9cc128a29d3716b5b761c

Changes applied:
  - Replaced false present-tense “That gather is live work: admit-an-indirect-gather-family-for-tied-embedding-lookup owns …” with a **Correction — 2026-08-10** recording delivery of `tiler::gather-f32@1` under ADR 0107 (admit ticket `done`), open index-layer / physical successors, and F-41 still inheriting O-08’s uncovered physical half; kept the done admit ticket as historical frontmatter dependency.
  - Tightened the following Inference so “no lowering” is scoped to physical/index-layer route, not the admitted semantic gather family.
  - Updated “What the work would be” and non-goals to point at remaining gather-route successors rather than the closed admit ticket alone.
  - Added optional `related` entries: `revise-adr-0108-with-a-complete-data-dependent-index-vertical`, `emit-the-indirect-gather-on-metal`.
  - Appended 2026-08-10 trigger-check log line **not fired** (workload grounds; no restatement of the 2026-08-05 key census as current).
  - Status left `deferred`; scopes/dependencies metadata otherwise unchanged.

Optional items skipped (with reason):
  - none (recommended related + trigger log applied as cheap graph hygiene on this ticket)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/research/semantic-graph/operation-family-delivery-graph.md O-34 “physical route is a gather that does not yet exist” rationale cell (research-doc rot outside ticket-only wave; separate docs-scoped repair if authorized)

Verification:
  - files read:
    - tickets/scope-the-geometric-resampling-family.md (full, pre and post)
    - audit report 44a9212784dd_c99ac54950f2.md (full)
    - tickets/admit-an-indirect-gather-family-for-tied-embedding-lookup.md (status: done)
    - tickets/revise-adr-0108-with-a-complete-data-dependent-index-vertical.md (status: awaiting-decision)
    - tickets/emit-the-indirect-gather-on-metal.md (status: blocked)
    - crates/tiler-ir/src/semantic/gather.rs (OpKey gather-f32@1, register_standard_gather)
    - crates/tiler-ir/src/semantic/registry.rs (register_standard_gather call site)
  - checks:
    - admit gather ticket done; successors present at expected statuses
    - semantic gather registration live; no resampling OpKey claimed in ticket

Recommended next ledger state:
  integrated
