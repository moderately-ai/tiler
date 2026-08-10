Ticket: pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate/12cd7c3dc7c1_c99ac54950f2.md
Pre-edit content hash (from ledger): 12cd7c3dc7c1152fdb9cdc1f00334cdd064acc463a98bbad1aff9f1baa489e22
Post-edit content hash: 85cec44451009ade64c13aeb0222d2c8d718a2023f38c109aa85aa96d7385b47

Changes applied:
  - Worker record intro: "Six independent properties, six separate perturbations of the subject" → "Nine separate perturbations covering six assertion properties" so the intro matches the nine-row perturbation table and the later "exactly the nine rows above" summary.

Optional items skipped (with reason):
  - Freeze-label clarity on cross-crate `tiler.` spelling counts at `c0829b41`: already labeled at that commit; report said no change required if readers honor the date.

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required no docs/crates edits, no new remainders; artifact exact-byte pin already on its own todo ticket).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate/12cd7c3dc7c1_c99ac54950f2.md (full)
    - tickets/pin-the-identity-domain-strings-so-a-reverted-domain-reddens-the-gate.md (full, pre- and post-edit)
  - checks:
    - shasum -a 256 on ticket post-edit → 85cec44451009ade64c13aeb0222d2c8d718a2023f38c109aa85aa96d7385b47
    - nine-row table and "exactly the nine rows above" already consistent; only the six/nine intro mismatch was wrong

Recommended next ledger state:
  integrated
