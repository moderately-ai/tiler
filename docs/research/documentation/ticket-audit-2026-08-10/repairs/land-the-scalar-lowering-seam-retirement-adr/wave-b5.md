Ticket: land-the-scalar-lowering-seam-retirement-adr
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/land-the-scalar-lowering-seam-retirement-adr/1ab71dc3aeed_c99ac54950f2.md
Pre-edit content hash (from ledger): 1ab71dc3aeedc7061df413c9804459453b26325c98b6e44108b941abf9ed7a5b
Post-edit content hash: be56238f18f739be73372812bd306dc27734e43c93bc77c724ace78e139a871e

Changes applied:
  - Added `## Fact audit — 2026-08-10` on the ticket: UV outcome / What-to-do `proposed` wording marked as pre-Amendment plan text (board status remains done; Amendment + Outcome remain authoritative).
  - Same section: Outcome byte-identity `sed` line recipe corrected to heading-bounded durable check (ADR 22–84 / source 127–189 at audit-time verification; 13,202 bytes; empty diff); landing-time 105 ADR/catalog population counts marked historical.
  - Same section: documents ADR 0078's three acceptance-time present-tense residues after remove-ticket completion as unsplit docs residual (not a false claim about the 2026-08-06 sweep; ticket stays done).

Optional items skipped (with reason):
  - none — both optional ticket-hygiene items from the report (UV-outcome note; byte-identity recipe correction) applied in the dated Fact audit block.

Residuals not applied (docs/crates/new tickets/authority):
  - docs/decisions/0078-name-the-intended-public-extension-seams.md — three required present-tense end-state rewrites (Superseded note "still in the tree"; item-4 "`ScalarLoweringProvider` installation is reachable…"; implementation boundary "scheduled removal"). Class C wave B5 is ticket-only; verified all three strings still present at current tree.
  - Remainder ticket (e.g. retire-adr-0078-acceptance-time-scalar-family-present-tense) under contracts/decisions, related to remove-the-scalar-lowering-family-from-the-compiler and this carrier — not filed in Class C; file when a decisions-scope batch owns the ADR 0078 prose fix if it is not folded into another live carrier.
  - No crates/ or authority/public-boundary changes; compiler identity already matches ADR 0105 complete.

Verification:
  - files read:
    - tickets/land-the-scalar-lowering-seam-retirement-adr.md (full, pre- and post-edit)
    - report 1ab71dc3aeed_c99ac54950f2.md (full)
    - docs/decisions/0078-name-the-intended-public-extension-seams.md (anchors for three residual sentences; still live)
    - docs/decisions/0105-retire-the-scalar-lowering-provider-seam.md (heading locations)
    - tickets/resolve-or-retire-the-scalar-lowering-provider-seam.md (heading locations)
  - checks:
    - python heading-bounded / fixed-range equality: ADR 22–84 == source 127–189, 13202 bytes each, equal=True
    - grep residual phrases on ADR 0078: still in the tree; installation is reachable; scheduled removal — all three present
    - shasum -a 256 of ticket after edit

Recommended next ledger state:
  integrated
