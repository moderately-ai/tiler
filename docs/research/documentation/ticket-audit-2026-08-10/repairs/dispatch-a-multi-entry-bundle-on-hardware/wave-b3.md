Ticket: dispatch-a-multi-entry-bundle-on-hardware
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/dispatch-a-multi-entry-bundle-on-hardware/018ba6dfc9e1_c99ac54950f2.md
Pre-edit content hash (from ledger): 018ba6dfc9e1aafd21a63b05a36cc59de9825ef5119eefd4fe63316ff03e5ebb
Post-edit content hash: efae9369a2a11f4465fbb94de9634410f90a4c0a12c54de27c0d59f50623c0df

Changes applied:
  - Implementation keys window bullet: replaced present-tense refusals of `[1,8]`, `[2,4]`, `[1,5]` with delivery-time past tense; noted grid-axis 268,435,456 and closed declined-strategy defect; kept `[1,4]` as lower edge and sole hardware-dispatched window; assigned wider split selection to calibrate.
  - Evidence reordering citation: replaced `main.rs:1102` with function name `dispatching_the_two_entries_out_of_order_returns_a_wrong_answer_rather_than_a_refusal` (path without line).

Optional items skipped (with reason):
  - Dated correction block under Implementation keys / Evidence — report says unnecessary when prose is rewritten in place; rewrote in place.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by audit (Exact files: ticket only; no remainder tickets; Follow-up scheduling-doc line cites correctly left out of scope).

Verification:
  - files read:
    - tickets/dispatch-a-multi-entry-bundle-on-hardware.md
    - reports/…/018ba6dfc9e1_c99ac54950f2.md
    - crates/tiler-macros/src/aot/tests.rs (`split_region` upper-edge comment)
    - crates/tiler-runtime/tests/adapter_route/main.rs (fn name at reordering test)
    - spikes/runtime/inline-dispatch/src/multi_entry.rs (window past-tense language)
    - spikes/runtime/inline-dispatch/README.md (window Measurement paragraph)
  - checks:
    - reordering fn name present in adapter_route/main.rs
    - split_region comment anchors grid-axis 268,435,456 and closed declined-strategy defect
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
