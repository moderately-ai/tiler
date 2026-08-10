Ticket: carry-semantic-enforcement-plans-through-program-and-artifact
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-semantic-enforcement-plans-through-program-and-artifact/573fd0fe81f0_c99ac54950f2.md
Pre-edit content hash (from ledger): 573fd0fe81f0a5fc4bc6191e56665930fc5aa9b0de23a71b42fd228c12c7125e
Post-edit content hash: 6281232a5e05a74097be673f0fb529110a6dbecfc8b1c4ffc3cd0e448b48896c

Changes applied:
  - In `## Corrected subject and dependencies — 2026-08-08`, rewrote the reconcile Fact so present-tense "currently disagree" is past ("had disagreed") and ownership is of the completed correction across artifact-abi, numerical-semantics, runtime-execution-contract, and first-quantized-lm-profile (dependency on reconcile kept).
  - Added `**Correction — 2026-08-10.**` noting at base `c99ac54950f2` the four records already agree under reconcile's named source-byte anchors; withdrawn present-tense disagreement; ownership and hard dependency retained.
  - Metadata left unchanged (status, deps, related, scopes correct per report).

Optional items skipped (with reason):
  - none listed as optional in Repair required.

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket repair (report: no remainder ticket; implementation product debt is out of wave B scope and already gated by fuse/parameter-map chain).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/carry-semantic-enforcement-plans-through-program-and-artifact/573fd0fe81f0_c99ac54950f2.md
    - tickets/carry-semantic-enforcement-plans-through-program-and-artifact.md
    - tickets/reconcile-direct-input-conformance-order-with-adr-0033.md (Reconciliation result anchors)
  - checks:
    - pre-edit sha256 matched ledger `573fd0fe81f0a5fc4bc6191e56665930fc5aa9b0de23a71b42fd228c12c7125e`
    - reconcile ticket: `status: done`; anchors live under `## Reconciliation result — 2026-08-08`

Recommended next ledger state:
  integrated
