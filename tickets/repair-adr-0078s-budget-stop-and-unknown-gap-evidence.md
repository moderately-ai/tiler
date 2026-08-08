---
id: repair-adr-0078s-budget-stop-and-unknown-gap-evidence
title: Repair ADR 0078's budget-stop and Unknown-gap evidence
status: todo
priority: p3
dependencies: [reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement]
related: [record-the-landed-physical-provider-seam-in-adrs-0078-and-0090]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, citations]
---
## User-visible outcome

This record stays unchanged because its accepted proof-budget-stop rule does not match the current executable-refinement path. Its P1 dependency must decide whether to restore the accepted behaviour or supersede the decision; this ticket must not adapt ADR 0078 to the drift.

## Why this exists

`a17884b0` accepted ADR 0078 unchanged. At that commit, `OccurrenceEvidence::BudgetStopped` in `crates/tiler-compiler/src/lowering.rs` represented the `Unknown` gap, `proof_budget_stop` returned a stop only when no other diagnostic was present, and `refine` returned `Ok(OccurrenceEvidence::BudgetStopped(stop))`; the pipeline recorded that state while retaining the compile path. ADR 0078 alone says `and the plan stands`; [the operation-extension contract](../docs/operation-extensions.md) instead combines its "An exhausted analysis budget is an `Unknown` gap" anchor with fail-closed index/access lowering language and a separate sole-diagnostic sentence.

**Fact — the current implementation does not realize ADR 0078's plan-standing rule.** The `pub fn complete` method on `ResolvedIndexRealization` in `crates/tiler-ir/src/index/refinement.rs` discharges residual predicates with one `struct IndexDomainProofLedger`. Its `fn debit` path records exhaustion, `fn fill_unassessed` assigns the resulting `ResourceLimit` claim to obligations not reached, and `IndexDomainProofRefusalKind::Disproved` is selected before `IndexDomainProofRefusalKind::Unknown`. Thus every produced assessment is retained and an assessed disproof wins, but a counterexample in a later unassessed obligation remains honestly unknown. The compiler maps that outcome through `IndexDomainDischargeRefusalKind::Unknown` and `LoweringError::SemanticDischarge`; `const fn semantic_discharge_is_invalid` returns false for Unknown, yet `lowering_failure` produces `UnsupportedCapability` before cover enumeration. `pending_and_refused_proofs_have_no_executable_coverage_spelling` confirms that no executable receipt is minted. This is source-true current behaviour, but it contradicts ADR 0078's accepted plan-standing behaviour rather than repairing its evidence.

**Fact — the operation-extension contract has a narrower reconciliation question.** Its `Missing optional knowledge is conservative` and `compile refusal for index/access lowering` anchors can agree with the current fail-closed `UnsupportedCapability` outcome without admitting executable coverage or calling an Unknown a semantic disproof. Its literal `A budget stop is reported only when it is the sole diagnostic` sentence needs separate reconciliation: the current atomic assessment retains an assessed `Disproved` with a later ResourceLimit Unknown, then returns Disproved overall. The implementation preserves the intended safety property that ResourceLimit never overrides an assessed disproof, but the ticket must determine whether that precedence satisfies the contract's sole-diagnostic wording.

**Fact — `RegionBudgetStop` is separate and has two classes.** `RegionBudgetResource::CandidatesPerSeed` and `RegionBudgetResource::Expansions` bound candidate growth; `fn retain_singleton_coverage` runs before `fn grow`, so either stop leaves singleton coverage intact and reports lost search alternatives. `RegionBudgetResource::Members`, `RegionBudgetResource::BoundaryOutputs`, and `RegionBudgetResource::LiveValues` are region-shape admission bounds applied by `fn classify` and `fn classify_shape`; they may refuse a fused candidate, including the whole-program candidate, rather than merely truncate search. The `pub(crate) fn budget_stops` accessor retains both classes, but neither is an index-domain proof claim or an executable-refinement receipt.

## Stop condition and graph repair

The ticket's original purpose changed: truthful prose cannot name current code as realizing ADR 0078's plan-standing rule. The dependency `reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement` owns both the ADR restore-versus-supersede question and the operation-extension contract's sole-diagnostic reconciliation. This ticket remains in progress only to preserve its live claim; after the graph-repair commit lands, the coordinator must release it to the dependency-blocked queue rather than close it or edit the ADR.

## Resume condition

Resume only after the P1 ticket has either restored ADR 0078's accepted plan-standing behaviour with current tests, or Tom has accepted a superseding decision, and after it has aligned the operation-extension contract's sole-diagnostic wording with the implementation. Then re-read ADR 0078, the operation-extension contract, current lowering/discharge/planning construction and consumption sites, and the exact diff before proposing any evidence repair.

## Verification boundary

This graph repair changes only ticket files, so the latest full gate carries under `AGENTS.md`; `tkt lint`, `make citations`, the base diff check, and the ticket guard remain mandatory.
