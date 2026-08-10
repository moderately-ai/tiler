Ticket: record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims/a2283be0ad1d_c99ac54950f2.md
Pre-edit content hash (from ledger): a2283be0ad1dffca66d3d1e87d2fec3250d3964bc7aab60c74e0ab2db4708577
Post-edit content hash: a210f11e53e478bbc59beabd638a4cfa0fc25adb8eb6e803ad43b7710c9e17c5

Changes applied:
  - related: added test-the-cooperative-lowering-shape-refusal for graph symmetry with its reverse edge
  - ## Fact audit — 2026-08-10 / **Correction — 2026-08-10.**: Outcome close conditions still hold; live false board-status labels in status.md and roadmap.md (second tile ticket is awaiting-decision, not deferred; realize remains deferred); ADR 0097 Implementation boundary "returns six files" drifted to seven; residual sites listed; status remains done

Optional items skipped (with reason):
  - none (optional related graph hygiene applied)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/status.md staging fact: replace "both `deferred`" with accurate board statuses (admit-a-cooperative-tile-over-shared-operands is awaiting-decision; realize-the-tiled-contraction-schedule-and-its-metal-emission is deferred) — Class C wave: ticket-only, no docs/ edit
  - docs/roadmap.md contraction row: replace "still `deferred` under admit-a-cooperative-tile-over-shared-operands" with awaiting-decision (or current status at edit time) — Class C wave: ticket-only
  - docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md Implementation boundary: replace "returns six files" with seven (or non-numeric claim) — Class C wave: ticket-only
  - no new remainder ticket required; lowering-refusal test already owned by test-the-cooperative-lowering-shape-refusal

Verification:
  - files read:
    - tickets/record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/record-adr-0097-as-implemented-and-correct-the-navigation-staging-claims/a2283be0ad1d_c99ac54950f2.md
    - tickets/admit-a-cooperative-tile-over-shared-operands.md (status: awaiting-decision)
    - tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md (status: deferred)
    - tickets/test-the-cooperative-lowering-shape-refusal.md (status: todo)
    - docs/status.md (both `deferred` claim present)
    - docs/roadmap.md (still `deferred` under second-tile claim present)
    - docs/decisions/0097-admit-a-two-dimensional-cooperative-staging-relation.md ("returns six files" present)
  - checks:
    - rg -m1 '^status:' on second-tile / realize / gap tickets
    - rg for both `deferred` / still `deferred` / returns six files in status, roadmap, ADR 0097
    - rg -l 'ParticipantSpace|MAX_COOPERATIVE_PARTICIPANT_RANK|SpanRank|LocalWorkgroupPosition' crates/ → 7 files
    - sha256 post-edit ticket file

Recommended next ledger state:
  integrated
