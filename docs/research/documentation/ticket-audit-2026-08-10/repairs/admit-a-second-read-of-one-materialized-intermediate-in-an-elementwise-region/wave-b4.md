Ticket: admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region/4b41cd2a40e7_c99ac54950f2.md
Pre-edit content hash (from ledger): 4b41cd2a40e7204a9124c0d9f785361ee766eb10d8f5e5431b9d012430b46da4
Post-edit content hash: 3e0f143c94e968b46849f1cef57c416d77f98856be4d077d5042e63d719e09d0

Changes applied:
  - related: set to [admit-two-reads-of-one-declared-input-in-an-elementwise-region, admit-a-scheduled-region-that-reads-two-materialization-edges]
  - Boundaries identity sentence: name parallel kernel encoder under tiler.kernel.v7 (and any other TensorRole-encoding domain) beside tiler.schedule.v5
  - Option A: "which cover edge it reads" → "which cover edge the access binds (producer or consumer)"; field meaning string unchanged

Optional items skipped (with reason):
  - none (report required items only; no optional prose beyond the related hygiene listed under Repair required)

Residuals not applied (docs/crates/new tickets/authority):
  - post-acceptance product work (crates/tiler-ir schedule+kernel model/builder, compiler request/program/5b maps, pins, acceptance node) left unstarted — wave B ticket-only
  - exact edge_ordinal numbering scheme deferred to implementation after Tom accepts (report residual uncertainty)

Verification:
  - files read: full audit report; full ticket; grep confirmed TensorRole::Intermediate => bytes.push(0x02) in schedule/model.rs and kernel/model.rs with tiler.schedule.v5 and tiler.kernel.v7
  - checks: shasum -a 256 of ticket after edit

Recommended next ledger state:
  integrated
