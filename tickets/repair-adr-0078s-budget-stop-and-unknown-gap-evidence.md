---
id: repair-adr-0078s-budget-stop-and-unknown-gap-evidence
title: Repair ADR 0078's budget-stop and Unknown-gap evidence
status: in-progress
priority: p3
dependencies: [accept-adr-0109-fail-closed-on-unknown-index-domain-proof]
related: [record-the-landed-physical-provider-seam-in-adrs-0078-and-0090]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, citations]
claimed_from: todo
assignee: w-adr0078-sol
lease_expires_at: 1786223949
---
## User-visible outcome

ADR 0078's stale source evidence is repaired against the accepted narrow supersession: ResourceLimit remains Unknown, while ADR 0109 makes refusal before executable planning and coverage normative. The repair does not revive historical symbols or widen the supersession beyond the retired `Ok` and “the plan stands” requirement.

## Why this exists

`a17884b0` accepted ADR 0078 unchanged. At that commit, `OccurrenceEvidence::BudgetStopped` in `crates/tiler-compiler/src/lowering.rs` represented the `Unknown` gap. `proof_budget_stop` accepted a nonempty diagnostic population only when every row was `ProofResourceLimit`, returned `None` if any diagnostic belonged to another class, and retained the first stop when several resource-limit rows were present. `refine` returned `Ok(OccurrenceEvidence::BudgetStopped(stop))`, and the pipeline recorded that state while retaining the compile path. ADR 0078 alone retains the historical `and the plan stands` sentence, now followed by its item-scoped supersession note. [The operation-extension contract](../docs/operation-extensions.md) combines its "An exhausted analysis budget is an `Unknown` gap" anchor with the accepted fail-closed and mixed-assessment rule.

**Fact — the current implementation does not realize ADR 0078's plan-standing rule.** The `pub fn complete` method on `ResolvedIndexRealization` in `crates/tiler-ir/src/index/refinement.rs` discharges residual predicates with one `struct IndexDomainProofLedger`. Its `fn debit` path records exhaustion, `fn fill_unassessed` assigns the resulting `ResourceLimit` claim to obligations not reached, and `IndexDomainProofRefusalKind::Disproved` is selected before `IndexDomainProofRefusalKind::Unknown`. Thus every produced assessment is retained and an assessed disproof wins, while a later unassessed obligation whose domain might contain a counterexample remains Unknown rather than being called disproved. The compiler maps that outcome through `IndexDomainDischargeRefusalKind::Unknown` and `LoweringError::SemanticDischarge`; `const fn semantic_discharge_is_invalid` returns false for Unknown, yet `lowering_failure` produces `UnsupportedCapability` before cover enumeration. `pending_and_refused_proofs_have_no_executable_coverage_spelling` confirms that no executable receipt is minted. This is source-true current behaviour, but it contradicts ADR 0078's accepted plan-standing behaviour rather than repairing its evidence.

**Fact — the operation-extension reconciliation is complete.** Its `Missing optional knowledge is conservative` and `compile refusal for index/access lowering` anchors agree with the fail-closed `UnsupportedCapability` outcome without admitting executable coverage or calling an Unknown a semantic disproof. ADR 0109's sweep replaced the base-tree sole-diagnostic sentence with the current atomic rule: every produced assessment is retained, an assessed `Disproved` claim takes precedence over a later ResourceLimit Unknown, and otherwise any Unknown refuses before executable planning and coverage.

**Fact — `RegionBudgetStop` is separate and has two classes.** `RegionBudgetResource::CandidatesPerSeed` and `RegionBudgetResource::Expansions` bound candidate growth; `fn retain_singleton_coverage` runs before `fn grow`, so either stop leaves singleton coverage intact and reports lost search alternatives. `RegionBudgetResource::Members`, `RegionBudgetResource::BoundaryOutputs`, and `RegionBudgetResource::LiveValues` are region-shape admission bounds applied by `fn classify` and `fn classify_shape`; they may refuse a fused candidate, including the whole-program candidate, rather than merely truncate search. The `pub(crate) fn budget_stops` accessor retains both classes, but neither is an index-domain proof claim or an executable-refinement receipt.

## Stop condition and graph repair

The ticket's original purpose changed: truthful prose cannot name current code as realizing ADR 0078's retired plan-standing rule. [ADR 0109](../docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md) now owns the accepted replacement, and the operation-extension contract no longer makes the false sole-diagnostic claim. This ticket repairs only ADR 0078's current-source evidence after that accepted authority.

## Resume condition

Resume after the `done` acceptance node satisfies this dependency. Re-read ADR 0078, ADR 0109, the operation-extension contract, current lowering/discharge/planning construction and consumption sites, and the exact diff before repairing the pending evidence note.

## Verification boundary

This graph repair changes only ticket files, so the latest full gate carries under `AGENTS.md`; `tkt lint`, `make citations`, the base diff check, and the ticket guard remain mandatory.
