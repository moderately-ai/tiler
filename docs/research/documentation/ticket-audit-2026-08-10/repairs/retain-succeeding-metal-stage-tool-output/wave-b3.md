Ticket: retain-succeeding-metal-stage-tool-output
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/retain-succeeding-metal-stage-tool-output/558ddc1fd711_c99ac54950f2.md
Pre-edit content hash (from ledger): 558ddc1fd71149c316211ccd183dc4b31ce5013d9bd8a8c375f30582f427fd7c
Post-edit content hash: adfff44b82b66cf6035223073c53e6b769f9c6c02098aa02f2727b7ce0c2a673

Changes applied:
  - Why Fact 1 / Fact 2 / Inference: dated Correction — 2026-08-10 plus historical rephrase so present-tense "keeps" / "states DebugRetention::none() today" cannot read as live; live anchors (success-path ToolOutput capture; stage_retention) noted.
  - Outcome lead "Held at review… what is not settled": struck; Correction points at accept-the-debug-retention-and-stage-outputs-public-surface (done; Tom accepted 2026-08-05).
  - Outcome "One gap filed": kept split fact; Correction notes carry-a-producer-stated-total-into-a-retained-run is done and stage_retention uses retaining_with_stated_total.
  - Single ## Fact audit — 2026-08-10 covering required items 1–3.
  - Optional related: added accept-the-debug-retention-and-stage-outputs-public-surface for graph symmetry.

Optional items skipped (with reason):
  - none (optional related applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none for ticket-only repair; report Exact files listed only this ticket. Unexercised real-toolchain path through publication remains an evidence gap named in Outcome, not a delivery remainder.

Verification:
  - files read: audit report; ticket (full); driver.rs run_stage Ok(ToolOutput::capture) via rg; metal_cache.rs stage_retention / retaining_with_stated_total via rg; accept-the-debug-retention-and-stage-outputs-public-surface status+Decided; carry-a-producer-stated-total-into-a-retained-run status: done
  - checks: shasum -a 256 of ticket after edit

Recommended next ledger state:
  integrated
