Ticket: test-the-autoregressive-state-failure-cases
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/test-the-autoregressive-state-failure-cases/7f4e6d3361a5_c99ac54950f2.md
Pre-edit content hash (from ledger): 7f4e6d3361a5e31a5167076514ab1f2eff5aed30024feb5ae1fbeb1259b0b7ec
Post-edit content hash: 138a4742cbc5d52f7b89471613132406784d90e79b835fdbf0eb659043860c74

Changes applied:
  - related: added evaluate-retained-shape-relations-before-routing-commit, admit-a-position-selecting-slice-for-the-rotary-table, and bind-repeated-invocations-over-caller-retained-tensors (optional packaging hygiene for cases 5/6). Did not promote evaluate or slice to hard dependencies.
  - Case 2 retitled Stale bindings → Inconsistent extent bindings; body states mutual-inconsistency refusal of S,C,T before routing commit and explicitly excludes content-stale allocations with consistent extents (consumer obligation). Interim *not refusable* until evaluate-retained lands; full watched-failing refusal once it has (expected before this ticket is ready).
  - Case 6 retitled One identity across the run → One identity across decode steps; pin is eight decode steps at C1 (T=1, S=11…18) share one artifact identity; prefill T=10 sharing conditional on symbolic packaging / D-19; record measured count rather than absolute nine-as-one.
  - Case 3: "names the step" clarified as driver step index composed with Tiler stage-named failure (not a public execution-ordinal field).
  - Closes when: split case 2 (full refusal once evaluate-retained lands) from case 7 (recorded uncaught / differential); withdrawn capacity case absent-from-harness clause unchanged.
  - Appended ## Fact audit — 2026-08-10 dated correction summarizing case 2 dual, case 6 C/T narrowing, Closes when split, and case 3 step-naming.

Optional items skipped (with reason):
  - none; optional related bind-repeated and optional case 3 one-liner and dated correction block were all cheap graph/prose hygiene and were applied.

Residuals not applied (docs/crates/new tickets/authority):
  - No harness under crates/ or prototypes/ — product implementation of cases 2–7 remains this ticket's own todo outcome, not wave-B remainder filing.
  - Content-staleness consumer obligation stays on integrate-loop / prove paths; not re-filed as a Tiler refusal.
  - Case 5 specialization-assembly surface (latent under crates/) and case 6 D-19 prefill identity condition are product/research work owned elsewhere.
  - No docs/ or crates/ edits (wave B ticket-only).

Verification:
  - files read: audit report 7f4e6d3361a5_c99ac54950f2.md; tickets/test-the-autoregressive-state-failure-cases.md (HEAD + working tree); git show HEAD and git diff for pre/post comparison
  - checks: HEAD content hash matches Phase A pin 7f4e6d3361a5e31a5167076514ab1f2eff5aed30024feb5ae1fbeb1259b0b7ec; post-edit shasum -a 256 = 138a4742cbc5d52f7b89471613132406784d90e79b835fdbf0eb659043860c74; grep confirms no remaining "Stale bindings" / unconditional "nine invocations produce one" / paired "current limitation" Closes-when wording; all Repair required bullets present in working tree

Recommended next ledger state:
  integrated
