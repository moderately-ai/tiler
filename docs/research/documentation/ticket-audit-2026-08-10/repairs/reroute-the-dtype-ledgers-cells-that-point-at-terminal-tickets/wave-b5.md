Ticket: reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets/e525575bad67_c99ac54950f2.md
Pre-edit content hash (from ledger): e525575bad67f4b1193888e7161350fc81339ad252832f95d5858466c4c21713
Post-edit content hash: a89a94b5092a2817410e5dc3dac2bd2844e38d42f124aea9e62fc85773a93b5c

Changes applied:
  - Struck present-tense universal "Every one of the thirty-five tickets linked … that concerns BF16 is likewise `done`" under Defect 1; **Correction — 2026-08-10.** scopes it to the pre-repair census and notes the post-landing blocked iOS partial owner link.
  - Added `## Residual audit — 2026-08-10`: D-4 physical/execution summary row still says `Owed; a live ticket carries it` / `lowering and execution are owed`; Trigger strike rationale `and so is every BF16 ticket this document links` overclaims while linking the blocked iOS owner; status stays `done`; residual is docs prose only.
  - No metadata changes (status `done`, empty dependencies/related, scopes unchanged per report).

Optional items skipped (with reason):
  - optional `related` edges: report said not required; historical host-dtype and rung tickets remain consumption sites, not live related edges.
  - optional short dated note if residual fixed on live docs: residual not fixed on docs this wave; residual section on ticket is the honesty form instead.
  - narrow docs remainder ticket ("align D-4 physical/execution summary row … and fix Trigger overclaim"): Class C ticket-only default; residual listed below rather than minting a new id this wave.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/research/numerics/dtype-family-research-tracks.md` `### Physical and execution obligations` D-4 row: replace three `Owed; a live ticket carries it` cells (Artifact ABI, Scalar/KIR, Conformance) and Backend `lowering and execution are owed` with language consistent with `#### D-4` (delivered-with-bounds for those rungs; remainders only where still true — conversion/accumulator/composed-vertical authorities, not anonymous live ticket).
  - `docs/dtype-support.md` BF16 Trigger strike rationale: rephrase `and so is every BF16 ticket this document links` so it does not assert a present-tense universal over this document's BF16 links; scope to the eight rung owners / pre-repair owner set.
  - No new remainder ticket filed this wave (Class C; product docs residual recorded for a later docs pass or narrow remainder).

Verification:
  - files read:
    - full audit report e525575bad67_c99ac54950f2.md
    - full ticket (pre/post edit)
    - docs/research/numerics/dtype-family-research-tracks.md physical table D-4 row + #### D-4 owner/rung/partial-owner anchors
    - docs/dtype-support.md BF16 Trigger (strike rationale + iOS partial owner link)
    - tickets/declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles.md frontmatter status
  - checks:
    - `rg -n 'live ticket carries' docs/research/numerics/dtype-family-research-tracks.md` → line 109 physical-table D-4 row only
    - `rg -n 'so is every BF16 ticket this document links' docs/dtype-support.md` → Trigger residual present
    - `grep -m1 '^status:' tickets/declare-the-bf16-ios-family-answers-on-authoritative-ios-profiles.md` → `status: blocked`
    - `shasum -a 256 tickets/reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets.md` → a89a94b5092a2817410e5dc3dac2bd2844e38d42f124aea9e62fc85773a93b5c

Recommended next ledger state:
  integrated
