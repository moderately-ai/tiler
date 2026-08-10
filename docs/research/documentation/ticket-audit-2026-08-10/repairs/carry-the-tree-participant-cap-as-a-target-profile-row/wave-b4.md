Ticket: carry-the-tree-participant-cap-as-a-target-profile-row
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-the-tree-participant-cap-as-a-target-profile-row/e755859ccbf6_c99ac54950f2.md
Pre-edit content hash (from ledger): e755859ccbf6b90926a6653e21070f53f1e94265538f3dd9d0f6a926de275077
Post-edit content hash: e06c51d975dc002502ab977b12168264af6c3ca42152f66c44f8335230942a53

Changes applied:
  - related: added `correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim` (done carrier of the retired workgroup-declares-1,024 claim)
  - left `status: awaiting-decision` (Reserved questions still open; no status downgrade/close)
  - dated **Correction — 2026-08-10** after the 2026-08-08 per-Fact table: third inventory claim retired in source by the correct- sibling; not this ticket's live doc deliverable
  - per-Fact table rows for inventory claims and "true today" updated to present-tense (two live local-memory claims; third gone)
  - inventory body paragraph rewritten: no present-tense assert that `capped_tree_partition` currently claims "widest workgroup … declares … 1,024"; residual leakage is the local-memory pair sequenced with `pin-the-local-memory-refusal-band-the-tree-cap-opened`
  - silence fork: replaced *unfired* characterization of `separate-the-tree-and-split-groupings-…` with re-collapse / already-fired (`status: done`) wording
  - Scope and sequencing repair section: residual local-memory sequencing + retired workgroup claim ownership

Optional items skipped (with reason):
  - optional related `activate-measured-reduction-selection-from-a-target-cost-row` — not in Repair required exact metadata; mechanism already narrated as `done` in body

Residuals not applied (docs/crates/new tickets/authority):
  - Reserved Tom decisions (1) may a profile-declared width preference withdraw alternatives / which guard, (2) silence selection — product gate; status correctly remains `awaiting-decision`
  - crates implementation if ever authorized (CostRow variant, declare pair, parameterized partition, Apple9 declare, identity pins)
  - residual pipeline-test comment `workgroup width a profile here declares` in `crates/tiler-compiler/src/pipeline/tests.rs` — out-of-scope doc noise per report; not reopened as this ticket's path

Verification:
  - files read:
    - tickets/carry-the-tree-participant-cap-as-a-target-profile-row.md (pre/post)
    - audit report e755859ccbf6_c99ac54950f2.md
    - crates/tiler-compiler/src/physical.rs (inventory anchors)
    - tickets/correct-the-capped-tree-partition-s-false-declared-workgroup-width-claim.md (status: done)
    - tickets/separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md (status: done; FIRED log)
  - checks:
    - `rg -F 'widest workgroup any profile' crates/tiler-compiler/src/physical.rs` → empty
    - `rg -F 'declares 32,768 bytes' crates/tiler-compiler/src/physical.rs` → present
    - `rg -F 'no profile in this repository' crates/tiler-compiler/src/physical.rs` → present
    - `rg -F 'workgroup width a profile here declares' crates/tiler-compiler/src/pipeline/tests.rs` → residual (not edited)
    - ticket related includes correct- sibling; no present-tense *unfired* claim for separate- sibling

Recommended next ledger state:
  integrated
