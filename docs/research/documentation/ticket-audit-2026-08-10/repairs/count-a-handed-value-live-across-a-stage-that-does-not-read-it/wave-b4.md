Ticket: count-a-handed-value-live-across-a-stage-that-does-not-read-it
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/count-a-handed-value-live-across-a-stage-that-does-not-read-it/64f27eb631cc_c99ac54950f2.md
Pre-edit content hash (from ledger): 64f27eb631cc9f4acc8b7ff6c5ce745322dc02df8d2c102024515e6251399489
Post-edit content hash: 24b112ef64628189c9d858f2595868bea46d0a26e79e937efb129c475415cc98

Changes applied:
  - Rewrote Why this exists third Fact: production census is three sequence-realizing arms (`StagedStrictSerialSumThenPointwiseF32`, `StagedRootMeanSquareScaleF32`, `StagedSoftmaxF32`) via `realizes_region_sequence`; preserved no-gap claims for all three and the expressible-gap example.
  - Added **Correction — 2026-08-10.** noting the stale "exactly two" census relative to the 2026-08-09 log.
  - Appended Trigger check log **2026-08-10 — not fired** with three-builder recheck anchors and cover/physical absence of a separate live-set demand.

Optional items skipped (with reason):
  - None material; dated correction and 2026-08-10 log line applied as recommended hygiene on the same ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - None required by Repair required. Product decision (count spanning live values vs record boundary-only bound) remains parked on this deferred ticket until trigger fires; no crate/docs edits in wave B.

Verification:
  - files read:
    - tickets/count-a-handed-value-live-across-a-stage-that-does-not-read-it.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/count-a-handed-value-live-across-a-stage-that-does-not-read-it/64f27eb631cc_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/law.rs (anchors: realizes_region_sequence three-arm match; builder docs)
    - crates/tiler-compiler/src/region.rs (produced_here/consumed_here skip; live_values formula)
    - crates/tiler-compiler/src/cover.rs and physical.rs (no live_values|region_shape|retained_through matches)
  - checks:
    - `rg -n 'realizes_region_sequence|StagedSoftmaxF32|StagedStrictSerialSumThenPointwiseF32|StagedRootMeanSquareScaleF32|Builds the' crates/tiler-ir/src/index/law.rs` — three sequence arms present
    - `rg -n 'produced_here|consumed_here|fn region_shape' crates/tiler-compiler/src/region.rs` — skip still present
    - cover/physical live-set consumer absence confirmed
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
