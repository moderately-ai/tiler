Ticket: record-the-landed-physical-provider-seam-in-adrs-0078-and-0090
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/record-the-landed-physical-provider-seam-in-adrs-0078-and-0090/aaf7ced4528e_c99ac54950f2.md
Pre-edit content hash (from ledger): aaf7ced4528e6649f89a529f67367068b5cc61cdf84a8d36e382a3c6863de3bb
Post-edit content hash: c66f2cd1b1fb2e0608dcd70bff6ad5854205e9f4b6cc8fe6bb0371ffd24a75b0

Changes applied:
  - Kept `status: done` (metadata unchanged per report).
  - Wired `related` to new remainder `retire-adr-0078s-stale-physical-provider-standing-clauses`.
  - Appended `## Fact audit — 2026-08-10` / **Correction — 2026-08-10.** documenting the two post-close ADR 0078 standing false clauses (offered disclosure half; forkless re-run), the seven→9 test under-count, and ownership of residual repair by the remainder ticket without reopening this carrier.
  - Created Class D remainder `tickets/retire-adr-0078s-stale-physical-provider-standing-clauses.md` (`todo`, `contracts/decisions`) owning the two required ADR 0078 dated retirements and optional seven-tests repin; related to parent, disclose, refresh, and accept surface.

Optional items skipped (with reason):
  - Optional "seven tests" repin on ADR 0078 itself deferred into the remainder ticket Implementation keys (docs residual, not applied on the ADR in this wave).

Residuals not applied (docs/crates/new tickets/authority):
  - Required ADR 0078 dated corrections (retire `item 5's *offered* disclosure half is still lowering-only` and `has not been re-run against the landed seam`; optional seven-tests census) — docs-only; owned by remainder ticket, not edited in this wave.
  - No crate or acceptance work.

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/record-the-landed-physical-provider-seam-in-adrs-0078-and-0090/aaf7ced4528e_c99ac54950f2.md (full)
    - tickets/record-the-landed-physical-provider-seam-in-adrs-0078-and-0090.md (full, pre and post)
    - docs/decisions/0078-name-the-intended-public-extension-seams.md (anchors for standing false clauses / seven tests)
    - tickets/repair-adr-0078s-budget-stop-and-unknown-gap-evidence.md, tickets/disclose-offered-and-selected-physical-provider-sets-separately.md, tickets/refresh-the-forkless-physical-provider-spike-against-the-landed-seam.md (frontmatter / style)
  - checks:
    - `grep -n "item 5's \*offered\* disclosure half is still lowering-only\|has not been re-run against the landed seam" docs/decisions/0078-name-the-intended-public-extension-seams.md` → both present as standing text
    - `grep -c '#\[test\]' crates/tiler-compiler/tests/external_physical_provider.rs` → 9
    - `pub fn offered_physical_providers` present in session.rs; spike results `2026-08-08-macos-arm64.json` present
    - `shasum -a 256 tickets/record-the-landed-physical-provider-seam-in-adrs-0078-and-0090.md` → c66f2cd1b1fb2e0608dcd70bff6ad5854205e9f4b6cc8fe6bb0371ffd24a75b0

Recommended next ledger state:
  integrated
