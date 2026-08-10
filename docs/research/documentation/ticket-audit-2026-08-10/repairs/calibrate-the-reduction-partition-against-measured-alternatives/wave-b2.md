Ticket: calibrate-the-reduction-partition-against-measured-alternatives
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/calibrate-the-reduction-partition-against-measured-alternatives/43b66274c7c3_c99ac54950f2.md
Pre-edit content hash (from ledger): 43b66274c7c36410bdeb9b18604635ac267ff418bcb0315e60cb2efaec6dbf14
Post-edit content hash: a50a93c181b8209e7788592ea89497e6680dc0595d44371ea7f48440e725f197

Changes applied:
  - Opening Fact (`governed_partition` unmeasured / doc quote / both strategies read it): added **Correction — 2026-08-10.** marking it historical open-ticket prose; measurement landed in this Outcome; quoted doc language gone from physical.rs; only split still reads `governed_partition`, tree reads `capped_tree_partition` after cap-the-tree-reduction-participants-at-the-measured-256; reproduction anchors included.
  - Outcome lead-in "Nothing is activated / crates not touched": added **Correction — 2026-08-10.** clarifying *this ticket's* delivery boundary vs later tree-cap activation at this base.
  - Owed rows section: added **Correction — 2026-08-10.** that rows 1–4 were delivered by carry-the-partition-calibration-rows-into-the-two-catalogs-and-the-optimizer-contract and cap-the-tree-reduction-participants-at-the-measured-256 (both done); historical handoff text retained, marked delivered.
  - Metadata: no change (status done, deps/related fine per report).

Optional items skipped (with reason):
  - none (optional Outcome activation clarifying sentence applied as cheap hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - Spike `regions.rs` footer drift ("shipped compiler still calls governed_partition" for both strategies) — out of ticket file; product/spike debt, not wave-B ticket prose.
  - No new remainder tickets (report: none; existing remainders already ticketed elsewhere).

Verification:
  - files read: ticket full; audit report full; physical.rs anchors via rg (governed_partition / capped_tree_partition / MEASURED_TREE_PARTICIPANT_CAP / no "deliberately *a* choice"); successor ticket status:done for cap-the-tree-... and carry-the-partition-calibration-rows-...
  - checks: shasum -a 256 of ticket post-edit; successor frontmatter status greps.

Recommended next ledger state:
  integrated
