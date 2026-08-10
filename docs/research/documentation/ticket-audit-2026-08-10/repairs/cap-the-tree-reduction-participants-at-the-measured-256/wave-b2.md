Ticket: cap-the-tree-reduction-participants-at-the-measured-256
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/cap-the-tree-reduction-participants-at-the-measured-256/bbb85e4b9716_c99ac54950f2.md
Pre-edit content hash (from ledger): bbb85e4b9716a9bbc8fd8d92fd3711be00afc30a05f30d0e052223468862a20a
Post-edit content hash: 933bfa90b02caadaed743f557873e1093e74ffb6c35fd6699a89ad25f4085e45

Changes applied:
  - User-visible outcome: dated 2026-08-10 note that landing formulation was "largest admissible not exceeding 256"; live rule is nearest 256, ties narrower (bound-the-tree-cap); measured shapes unchanged.
  - Work: same dated note on truncate-from-below vs live `capped_tree_partition` / `MEASURED_TREE_PARTICIPANT_CAP`.
  - Outcome decline-set sentence: dated correction marking 2,561 as superseded truncate-from-below population; live nearest rule differs at 2,350 of 3,530 admitting counts (domain agreement retained).
  - Outcome call-site citations: retired `physical.rs:2547` / `:2733` and `metal_declaration.rs:255` line numbers; symbol anchors `capped_tree_partition(contributors)`, `governed_partition(contributors)`, `local_memory_bytes: 32_768`.
  - Graph effect: dated note that separate-the-tree later closed (done); deferred→todo remains historical fire record.

Optional items skipped (with reason):
  - none (optional graph note applied as cheap hygiene on this ticket).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report; Exact files listed ticket only. pin-the-local-memory-refusal-band-the-tree-cap-opened remains open product remainder (already filed).

Verification:
  - files read: assigned audit report; entire ticket; physical.rs (capped/governed sites via search); pipeline/tests.rs (assert_eq differing 2_350, admitted 3_530; nearest-rule docs); metal_declaration.rs (`local_memory_bytes: 32_768`); fusion-and-scheduling.md (nearest 256 wording); separate-the-tree ticket status done.
  - checks: shasum -a 256 on ticket post-edit.

Recommended next ledger state:
  integrated
