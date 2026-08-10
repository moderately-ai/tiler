Ticket: declare-metal-subgroup-realization-facts-in-the-target-profile
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/declare-metal-subgroup-realization-facts-in-the-target-profile/18e7c4b4230e_c99ac54950f2.md
Pre-edit content hash (from ledger): 18e7c4b4230e4c10df502b9226b01728251fa6cef3e549ed7189fcec25f4c315
Post-edit content hash: 4897dff006e701b00f9c67407fe6dbf9ebbae939b7ecc97780706a8f08f00121

Changes applied:
  - Decision packet: replaced "separate atomic declared facts" with whole-subject language (one `SubgroupRealization` + `declare_subgroup_realization`, no per-dimension setters, two-valued support, `Unknown` when silent, width as dimension inside subject + landed route-dimension equality); exclusions kept including item 6 preflight.
  - Implementation keys first bullet: restated as one equality-matched subject; dimensions not independently declarable; refusal names unmatched/Unrealizable subject; anti-boolean clause preserved.
  - Required failure paths: mirrored CPU ticket shape (one-dimension-differing → Unknown; Unrealizable → Rejected; silent → Unknown; duplicate Realized+Unrealizable → construction error; family-cannot-support path).
  - Why now SubgroupThreads defect citation: dropped bare `design-the-subgroup-execution-tier.md:65`; cite design ticket link + anchors `A live defect was found in landed public vocabulary` / `threads one subgroup must execute in lockstep`.
  - Non-goals + Closes-when: stated residual ownership of public-boundary item 6 (`PreparedKernelPreflight` / `threadExecutionWidth`) and research measurement deferral — do not silently absorb or drop.
  - Dated **Correction — 2026-08-10.** under Decision packet recording the whole-subject restatement.

Optional items skipped (with reason):
  - related list: schedule sibling (`admit-subgroup-bindings-into-the-schedule-vocabulary`) not added — report marks as optional hygiene only; Non-goals already name schedule bindings; not load-bearing for graph truth.

Residuals not applied (docs/crates/new tickets/authority):
  - Product implementation of `SubgroupRealization` / `declare_subgroup_realization` / feasibility resolve (crates/tiler-compiler target + feasibility; Metal profile population) — out of wave B scope; ticket remains `awaiting-decision`.
  - Public-boundary item 6 preflight stage and `threadExecutionWidth` measurement experiment — ownership stated on ticket; no new ticket id invented; Tom decides split vs keep.
  - Exact dimension field set of `SubgroupRealization` remains Tom's at acceptance.

Verification:
  - files read:
    - tickets/declare-metal-subgroup-realization-facts-in-the-target-profile.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/declare-metal-subgroup-realization-facts-in-the-target-profile/18e7c4b4230e_c99ac54950f2.md
    - tickets/declare-cpu-vector-realization-facts-in-the-target-profile.md (failure-path / Implementation keys template)
    - tickets/design-the-subgroup-execution-tier.md (defect paragraph anchors via rg)
    - docs/research/scheduling/subgroup-execution-tier.md (atomic subject §3; public-boundary items 5–6; threadExecutionWidth deferral)
  - checks:
    - rg: `threads one subgroup must execute in lockstep` hits design ticket defect paragraph
    - rg: `declare_subgroup_realization` / atomic-subject heading on research record
    - shasum -a 256 post-edit ticket → 4897dff006e701b00f9c67407fe6dbf9ebbae939b7ecc97780706a8f08f00121
    - metadata (status, deps, related, scopes) left unchanged per report

Recommended next ledger state:
  integrated
