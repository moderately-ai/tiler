---
id: reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement
title: Reconcile the accepted proof-budget-stop rule with executable refinement
status: in-progress
priority: p1
dependencies: []
related: [repair-adr-0078s-budget-stop-and-unknown-gap-evidence]
scopes: [contracts/decisions, contracts/foundation, contracts/optimizer, implementation/ir, implementation/compiler, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, correctness]
claimed_from: todo
assignee: w-sol-adr0078-reconcile
lease_expires_at: 1786213690
---
## User-visible outcome

The accepted proof-budget-stop behaviour and the executable refinement path agree under one deliberate authority: either an exhausted index-domain proof remains a non-executable Unknown analysis state while other planning continues, or Tom accepts a superseding decision that makes the current refusal-before-planning behaviour normative. No prose record is silently adapted to implementation drift.

## Why this is P1

**Fact — accepted authority.** [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) was accepted unchanged at `a17884b0`. Its item 3 anchor, "An exhausted analysis budget is an `Unknown` gap", required a typed budget stop, no disproof, an Ok lowering path, and a plan that stands. [The operation-extension contract](../docs/operation-extensions.md) uses the same Unknown-gap anchor but does not say that a plan stands.

**Fact — historical realization.** At `a17884b0`, `OccurrenceEvidence::BudgetStopped` in `crates/tiler-compiler/src/lowering.rs` carried the gap, `proof_budget_stop` accepted only a sole proof-resource diagnostic, and `refine` returned `Ok(OccurrenceEvidence::BudgetStopped(stop))`. The historical pipeline then recorded `OccurrenceEvidence::BudgetStopped` instead of refusing lowering.

**Fact — current drift.** The `pub fn complete` method on `ResolvedIndexRealization` in `crates/tiler-ir/src/index/refinement.rs` has one `struct IndexDomainProofLedger` for its atomic assessment pass. Its `fn debit` path records exhaustion and `fn fill_unassessed` assigns the resulting `ResourceLimit` Unknown to every obligation left unassessed. The pass retains every produced assessment and chooses `IndexDomainProofRefusalKind::Disproved` before `IndexDomainProofRefusalKind::Unknown`, so an assessed disproof wins while an unassessed counterexample remains Unknown. `discharge_pending_index_refinement` projects that refusal through `IndexDomainDischargeRefusalKind::Unknown` and `LoweringError::SemanticDischarge`; `lowering_failure` then returns `UnsupportedCapability` before cover enumeration. `pending_and_refused_proofs_have_no_executable_coverage_spelling` pins the no-receipt boundary.

**Fact — current records split the contradiction.** [The optimizer contract](../docs/compiler/optimizer.md) describes the current `semantic-discharge` refusal path and says Unknown is unsupported capability before an executable frontier. [The runtime-execution contract](../docs/research/runtime/runtime-execution-contract.md) likewise says unsupported or over-budget Unknown refuses before an executable plan exists. Those records conflict with ADR 0078's plan-standing rule. The operation-extension contract's `Missing optional knowledge is conservative` and `compile refusal for index/access lowering` anchors can agree with the current fail-closed outcome, but its `A budget stop is reported only when it is the sole diagnostic` sentence must be reconciled with current atomic assessments that retain ResourceLimit Unknown beside an assessed Disproved and return Disproved overall.

**Fact — adjacent budget findings are not interchangeable.** In `crates/tiler-compiler/src/region.rs`, `RegionBudgetResource::CandidatesPerSeed` and `RegionBudgetResource::Expansions` stop search growth after `fn retain_singleton_coverage`, preserving singleton coverage while reporting lost alternatives. `RegionBudgetResource::Members`, `RegionBudgetResource::BoundaryOutputs`, and `RegionBudgetResource::LiveValues` are shape-admission bounds applied by `fn classify` and `fn classify_shape`; they can reject a fused or whole-program candidate. The `pub(crate) fn budget_stops` accessor retains both region classes, neither of which is an index-domain proof result.

## Required derivation

1. Re-read ADR 0078 at acceptance commit `a17884b0`, including the historical `OccurrenceEvidence::BudgetStopped`, `proof_budget_stop`, pipeline construction and consumption sites, and the correctness-bearing tests that established the accepted behaviour.
2. Re-read the current IR `pub fn complete` and `struct IndexDomainProofLedger` path, compiler `discharge_pending_index_refinement`, `LoweringError::SemanticDischarge`, `lowering_failure`, explain flow, all current construction and consumption sites, and `whole_call_ledger_preserves_earlier_group_and_stops_later_groups_atomically` plus `pending_and_refused_proofs_have_no_executable_coverage_spelling`.
3. Decide with evidence whether to restore ADR 0078's plan-standing behaviour or to ask Tom to supersede that accepted rule with the current before-planning refusal.
4. Independently align the operation-extension contract's sole-diagnostic wording with retained mixed assessments and Disproved precedence, preserving the rule that ResourceLimit never overrides an assessed disproof.
5. If restoration is chosen, define the smallest coherent implementation, explain, coverage, and test changes; if supersession is chosen, prepare the proposed ADR/contract sweep for Tom rather than editing accepted authority as if it had changed.

## Non-goals and stop conditions

Do not change crate code, ADR 0078, or the operation-extension contract merely to make a grep pass. Do not describe every `RegionBudgetStop` as search loss. Stop and split work if reconciliation reaches a public boundary, artifact identity, runtime fallback, or numerical-contract decision not already owned here.

## Relationship

This ticket blocks [`repair-adr-0078s-budget-stop-and-unknown-gap-evidence`](repair-adr-0078s-budget-stop-and-unknown-gap-evidence.md). Its outcome must record the chosen authority, exact evidence, affected contracts, tests, and any follow-on implementation or acceptance tickets before the blocked repair ticket resumes.
