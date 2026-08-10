Ticket: admit-subgroup-typed-values-and-collectives-into-the-kernel-ir
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-subgroup-typed-values-and-collectives-into-the-kernel-ir/0ed51d4d00ec_c99ac54950f2.md
Pre-edit content hash (from ledger): 0ed51d4d00ec0a30f17e50fbb95020924df69ce7daa0b1a35c66934157a61e95
Post-edit content hash: 4db3e44c6d10b41e8d874a85223ef17a34f80a514b798e7f49346a4c57cd95a2

Changes applied:
  - Non-goals: removed false claim that two-level composition fires `add-subgroup-memory-scope-when-collectives-land`; restated ADR 0096 decision 7 (workgroup visibility for inter-subgroup handoff) and deferred trigger (subgroup-private scratch tile); kept shuffle-tree / MemoryScope::Subgroup caution (ADR 0094 decision 2 / MSL §6.10.2).
  - Implementation keys tripwire: dropped "land that tripwire first"; now requires updating existing `barrier_scope_vocabulary_is_closed` / `the_barrier_scope_vocabularies_are_still_closed` if either scope enum widens.
  - Replaced research-record line citation `:396` with searchable anchor `becomes the second construct in the vocabulary needing a proved reduction identity`.
  - Combine-tree ownership: schedule sibling owns stated combine order / unstated-order failure path; this ticket owns subgroup-typed values + explicit shuffle + ordinary arithmetic and refuses reduction collectives by name; removed kernel-owned "combine tree" key and the unstated-order failure path.
  - Dated **Correction — 2026-08-10.** under Non-goals summarizing the above for readers of prior wording.

Optional items skipped (with reason):
  - none (optional dated correction applied together with the required rewrites).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave — report listed no docs/crates edits and no remainder ticket; future implementation (kernel model, public shapes to Tom, schedule dep first) is product work outside Phase B.

Verification:
  - files read:
    - full audit report at reports/.../0ed51d4d00ec_c99ac54950f2.md
    - full ticket pre-edit
    - rg anchors: lane-identity phrase in research record + ADR 0094; barrier tripwire fns in crates/tiler-ir/src/kernel/tests.rs; ADR 0096 decision 7 / deferred trigger text; schedule sibling unstated-order failure path
  - checks:
    - metadata left unchanged (status todo, deps, related, scopes) per report
    - post-edit sha256: 4db3e44c6d10b41e8d874a85223ef17a34f80a514b798e7f49346a4c57cd95a2

Recommended next ledger state:
  integrated
