Ticket: reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section/42e7399691bb_c99ac54950f2.md
Pre-edit content hash (from ledger): 42e7399691bbda5d6ed3cfffd004d6f793a21c5db065f376383f5feebbe1e28c
Post-edit content hash: 823e65bca09bbf966988bde2fa22958ea9f97f0fed2a5619afe2ead151178e1e

Changes applied:
  - Fixed 2026-08-08 ticket repair paragraph: `identity_digest` is the fourth of five digest arguments (fifth envelope domain overall after the manifest framing tag), not the fifth digest argument.
  - Added **Correction — 2026-08-10.** one-line dated note recording the rank fix; status and outcome unchanged.
  - Optional graph hygiene: added `reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names` to `related` for reverse symmetry with the sibling that delivered site 1.

Optional items skipped (with reason):
  - none

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required no docs/crates edits and no new remainder ticket

Verification:
  - files read:
    - tickets/reconcile-the-artifact-abis-hashing-site-count-with-its-own-governed-digest-section.md (full)
    - report 42e7399691bb_c99ac54950f2.md (full)
    - docs/artifact-abi.md "The governed digest" (~405–432): five digest arguments ordered manifest, section, envelope, identity, payload_identity
  - checks:
    - shasum -a 256 of ticket after edit → 823e65bca09bbf966988bde2fa22958ea9f97f0fed2a5619afe2ead151178e1e
    - sibling ticket path exists: tickets/reconcile-the-artifact-abis-four-hashing-sites-with-the-fifth-it-names.md

Recommended next ledger state:
  integrated
