Ticket: correct-the-dangling-digest-parts-reference-in-the-artifact-program-module
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-dangling-digest-parts-reference-in-the-artifact-program-module/9203285489c9_c99ac54950f2.md
Pre-edit content hash (from ledger): 9203285489c900678552a857ab55496e9ffa2a31ccc08150df1c796fa07c1499
Post-edit content hash: da1c98c74a95f47213abad129f1fbb0dd21ca53aac26a24d4d86ef02b5df68df

Changes applied:
  - Added `## Outcome — 2026-08-10` recording delivered comment fix (parts-digest gone rather than promoted; `envelope_digest` crate-private; no tiler-digest edit; 8-claim neighbouring digest-block census; reproduce commands).
  - Added **Correction — 2026-08-10.** under Facts marking the present-tense dangling-`digest_parts` / crates-disagree claims as historical filing-time facts.
  - Extended `related` with sibling `correct-the-reachable-only-under-test-claim-the-delivered-realization-domain-falsifies` (cheap discoverability).

Optional items skipped (with reason):
  - none (optional Facts historical note and optional related sibling both applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required ticket Outcome only; no crate/docs edits; no remainder tickets; no commit hash invented).

Verification:
  - files read:
    - full audit report `9203285489c9_c99ac54950f2.md`
    - full ticket (pre- and post-edit)
    - `crates/tiler-artifact/src/program/mod.rs` digest re-export block (`parts-digest this crate carried gone rather than promoted`)
    - `crates/tiler-digest/src/lib.rs` public surface + "gone rather than promoted" header
    - sibling ticket Outcome for census cross-check
  - checks:
    - `rg 'digest_parts' crates/` → empty
    - `rg 'parts-digest this crate carried gone rather than promoted' crates/tiler-artifact/src/program/mod.rs` → 1
    - `rg 'gone rather than promoted|parts-digest|general form' crates/tiler-digest` → matches line-broken header
    - `shasum -a 256` post-edit ticket → da1c98c74a95f47213abad129f1fbb0dd21ca53aac26a24d4d86ef02b5df68df

Recommended next ledger state:
  integrated
