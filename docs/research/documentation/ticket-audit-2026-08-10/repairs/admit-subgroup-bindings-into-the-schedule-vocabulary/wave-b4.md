Ticket: admit-subgroup-bindings-into-the-schedule-vocabulary
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-subgroup-bindings-into-the-schedule-vocabulary/ca8d11ea913b_c99ac54950f2.md
Pre-edit content hash (from ledger): ca8d11ea913b19fe3a7c9d0688eb75e3ce1056ff8817082d1c15ab46a89cd8a6
Post-edit content hash: 4683957bc500944eea8889102b126230bd371517db360bca571ebe2ba3dead59

Changes applied:
  - Decision packet: removed `ExecutionBinding` from Tom-acceptance surface; restated schedule-side public items as `ReductionTopology::SubgroupTree`, `CombineTree`, `lane_identity_bits` + proof, subgroup-lane `LocalCoordinateSource` (items 1–4). Dated 2026-08-10 correction for the false binding list.
  - Implementation keys first bullet: topology-first language aligned to research §1 sketch and §5 "no new binding"; kept stated combine order and vendor collective opacity; aligned vendor wording to "Metal nor WebGPU" with MSL/WGSL parenthetical.
  - Identity-encoding key: dropped load-bearing irrefutable-`let`→match claim for this ticket; stated existing exhaustive matches on `ReductionTopology` / `LocalCoordinateSource` and append-after-`0x35` proof.
  - Required failure paths: replaced "two lanes owning one output" with `result_lane` / multi-writer commit ownership; added descending-stride under permutation-forbidding contract; `lane_identity_bits` of `+0.0` under signed-zero-forbidding contract; threadgroup size not multiple of width (decision 3).

Optional items skipped (with reason):
  - related-list hygiene for kernel-IR and declare-metal siblings: report marks non-load-bearing; Non-goals already name both ids.
  - Public item 8 (Collective refusal-reason doc widening): optional ownership unclear per residual uncertainty; not claimed as this ticket's required surface.

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation remains open under this ticket after Tom accepts the corrected surface (no remainder ticket).
  - Shared CPU/subgroup lane-identity spelling stays coordination via related `admit-vector-lane-bindings-into-the-schedule-vocabulary`, not a new edge.
  - No crates/ or docs/ product edits in wave B4 (schedule model still lacks `SubgroupTree`).

Verification:
  - files read: audit report; full ticket; research §1 SubgroupTree sketch and §5 "no new binding"; ADR 0094 decisions 1/3/4/8/9 and public-boundary paragraph; `ExecutionBinding` / `ReductionTopology` / `push_schedule` / TAG `0x35` in `crates/tiler-ir/src/schedule/model.rs`; `LocalCoordinateSource` in `cooperative.rs`
  - checks: `shasum -a 256` post-edit hash; status/deps/related left unchanged per report (none required)

Recommended next ledger state:
  integrated
