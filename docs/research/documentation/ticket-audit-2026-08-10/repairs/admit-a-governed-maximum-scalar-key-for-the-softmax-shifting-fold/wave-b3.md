Ticket: admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-governed-maximum-scalar-key-for-the-softmax-shifting-fold/1baa1c370e47_c99ac54950f2.md
Pre-edit content hash (from ledger): 1baa1c370e474ed64a3649b8a2da71875e4d90d60a35aa9a6a87812b8ae95ff8
Post-edit content hash: 0396ded09c79ba578c4491c306e1533b55a464a15aff6e223e3e8b85ae6c71ef

Changes applied:
  - Outcome 2026-08-06: struck "the twelfth governed key"; added **Correction — 2026-08-10** stating the key is one of twelve in `ScalarRegistryBuilder::standard`, eighth by registration order (after `canonicalize-nan-f32`), with reproduce command; note that draft/parked acceptance wording is landing snapshot and accept ticket is `done`.
  - No metadata changes (status, deps, related, scopes left as correct per report).

Optional items skipped (with reason):
  - None required beyond the ordinal; pre-Outcome line citations left as historical problem statement (not re-cited as live). Acceptance-ticket surface still saying "twelfth" is out of this ticket's ownership (report residual).

Residuals not applied (docs/crates/new tickets/authority):
  - None on product paths; Exact files listed ticket only.
  - Acceptance ticket ordinal prose residual remains on `accept-the-governed-maximum-scalar-key` (other ownership).

Verification:
  - files read: audit report; full ticket; `crates/tiler-ir/src/index/scalar.rs` `standard()` `builder.register` list (twelve calls; `maximum_f32_scalar_op` eighth).
  - checks: registration-order count matches audit Fact 10; no other required Repair bullets.

Recommended next ledger state:
  integrated
