Ticket: replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/replace-the-stale-artifact-abi-byte-figures-with-the-properties-tests-pin/ae9957fe5c7d_c99ac54950f2.md
Pre-edit content hash (from ledger): ae9957fe5c7dd66c5f5e9fc5805545f321af9e283fab2fdfcba5a6695dae4741
Post-edit content hash: 5586b497512de2cf66a8132d7cded914cb3828acbb4cdfc4ea997095f8299405

Changes applied:
  - Amended `## Later follow-through — 2026-08-09` so it no longer claims zero live remainder; scopes the 2026-08-09 close to implementation remainders only.
  - Added `**Correction — 2026-08-10.**` recording that `DIFFERING_IDENTITY_POSITIONS` / length equality landed in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` while live `docs/artifact-abi.md` paragraph `What is left unpinned, stated rather than left for a reader to assume.` still denies the pin; residual is contracts rewrite without reopening absolute lengths; status stays `done`.
  - Optional graph hygiene: added `pin-the-differing-identity-positions-beside-the-carrier-positions-constant` and `date-or-regenerate-the-six-kernel-identity-lengths-in-the-artifact-abi` to `related` (symmetry with those tickets listing this one).

Optional items skipped (with reason):
  - none (related edges applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/artifact-abi.md`: rewrite or date-correct the "What is left unpinned" paragraph so it cites `DIFFERING_IDENTITY_POSITIONS` and the length-equality assert in `a_bf16_artifact_round_trips_and_its_carrier_enters_identity` (or states the unpinned claim was true only until the pin ticket). Wave B forbids docs edits.
  - Optional forged-pair Measurement clause alignment (present-tense "exactly four byte positions" framed as pinned the way **68** is) — same contracts residual.
  - No new remainder ticket filed (wave B does not mint ticket ids); residual is pure contracts prose debt under `contracts/artifacts`.

Verification:
  - files read: audit report; full ticket; greps for `DIFFERING_IDENTITY_POSITIONS` / `What is left unpinned` confirming pin in `crates/tiler-artifact/src/program/codec/tests.rs` (`const DIFFERING_IDENTITY_POSITIONS: usize = 4`) and live false prose at `docs/artifact-abi.md` line with that anchor.
  - checks: `shasum -a 256` of ticket after edit → `5586b497512de2cf66a8132d7cded914cb3828acbb4cdfc4ea997095f8299405`

Recommended next ledger state:
  integrated
