Ticket: fold-the-attribution-stage-into-region-and-request-subject-identity
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/fold-the-attribution-stage-into-region-and-request-subject-identity/12370b46e3ed_c99ac54950f2.md
Pre-edit content hash (from ledger): 12370b46e3edd41271af7587e3c6f68449391585651c75e0b2ec7afc69595cbf
Post-edit content hash: 61edc6dbc1d4166e29efc4d8931bf66f3a612bfb599753686404b84b2a897686

Changes applied:
  - tags: removed `deferred` (status done); kept `[implementation, identity-domain]`
  - related: added carrier `enumerate-region-candidates-over-realization-stages` and decision edge `resolve-which-authority-mints-a-multi-stage-region-candidate` (optional graph hygiene applied)
  - Added `## Outcome` stating live encoding: content-side conditional trailer, occurrence via embedded content only, request-subject unchanged for stage, rebuild verify, zero region pin moves
  - Renamed/reframed `## What is missing` → `## What was missing (pre-delivery)` as historical, not live present tense
  - Corrected discharge (final trigger-log bullet) and "must then do" trailer wording: append_stage_trailer only from encode_content; no occurrence-side trailer
  - Added `## Fact audit — 2026-08-10` summarizing metadata and trailer-site corrections

Optional items skipped (with reason):
  - none — optional related edge for resolve-which-authority-mints applied as cheap graph hygiene

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket's identity obligations; report listed no remainder filing
  - sibling ticket implement-stage-level-cover-atoms retired-wall prose (out of scope per report residual uncertainty)

Verification:
  - files read:
    - tickets/fold-the-attribution-stage-into-region-and-request-subject-identity.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/fold-the-attribution-stage-into-region-and-request-subject-identity/12370b46e3ed_c99ac54950f2.md
    - crates/tiler-compiler/src/region.rs (append_stage_trailer call sites; encode_occurrence embed docs)
    - crates/tiler-compiler/src/domains.rs (region-content.v1 / region-occurrence.v1 pins)
  - checks:
    - `rg -n 'append_stage_trailer' crates/tiler-compiler/src/region.rs` → definition + single call inside encode_content
    - domain pins still v1 for region-content and region-occurrence
    - ticket frontmatter: no deferred tag; related includes carrier + resolve edge
    - Outcome present; What was missing framed historical; discharge trailer wording corrected
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
