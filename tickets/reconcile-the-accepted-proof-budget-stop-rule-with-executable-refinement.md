---
id: reconcile-the-accepted-proof-budget-stop-rule-with-executable-refinement
title: Reconcile the accepted proof-budget-stop rule with executable refinement
status: done
priority: p1
dependencies: []
related: [repair-adr-0078s-budget-stop-and-unknown-gap-evidence]
scopes: [contracts/decisions, contracts/foundation, contracts/navigation, contracts/optimizer, implementation/ir, implementation/compiler, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [decision, documentation, correctness]
---
## User-visible outcome

The accepted proof-budget-stop behaviour and the executable refinement path agree under [ADR 0109](../docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md): an exhausted index-domain proof remains Unknown, neither disproof nor admission, and compilation fails closed before executable planning and coverage. No prose record is silently adapted to implementation drift.

## Why this is P1

**Fact — accepted authority.** [ADR 0078](../docs/decisions/0078-name-the-intended-public-extension-seams.md) was accepted unchanged at `a17884b0`. Its item 3 anchor, "An exhausted analysis budget is an `Unknown` gap", required a typed budget stop, no disproof, an Ok lowering path, and a plan that stands. [The operation-extension contract](../docs/operation-extensions.md) uses the same Unknown-gap anchor but does not say that a plan stands.

**Fact — historical realization.** At `a17884b0`, `OccurrenceEvidence::BudgetStopped` in `crates/tiler-compiler/src/lowering.rs` carried the gap, and `refine` returned `Ok(OccurrenceEvidence::BudgetStopped(stop))`. The historical `fn proof_budget_stop` accepted a nonempty diagnostic slice only when every row was `IndexRegionDiagnostic::ProofResourceLimit`, returned the first through the source anchor `stop.get_or_insert(RefinementBudgetStop {`, and returned `None` on any other diagnostic; it did not require the slice to contain exactly one row. The historical pipeline then recorded `OccurrenceEvidence::BudgetStopped` instead of refusing lowering.

**Fact — current drift.** The `pub fn complete` method on `ResolvedIndexRealization` in `crates/tiler-ir/src/index/refinement.rs` has one `struct IndexDomainProofLedger` for its atomic assessment pass. Its `fn debit` path records exhaustion and `fn fill_unassessed` assigns the resulting `ResourceLimit` Unknown to every obligation left unassessed. The pass retains every produced assessment and chooses `IndexDomainProofRefusalKind::Disproved` before `IndexDomainProofRefusalKind::Unknown`, so an assessed disproof wins while a later unassessed obligation whose domain might contain a counterexample remains Unknown rather than being called disproved. `discharge_pending_index_refinement` projects an overall Unknown refusal through `IndexDomainDischargeRefusalKind::Unknown` and `LoweringError::SemanticDischarge`; `lowering_failure` then returns `UnsupportedCapability` before cover enumeration. `pending_and_refused_proofs_have_no_executable_coverage_spelling` pins the no-receipt boundary.

**Fact — the named whole-call test did not establish its claimed boundary at this base.** `whole_call_ledger_preserves_earlier_group_and_stops_later_groups_atomically` uses `fn two_domain_residual_region`, whose first domain already exceeds the integer-byte budget, and its source anchor `claims.iter().all` requires every claim to be the same `ResourceLimit` Unknown. It therefore establishes atomic filling after first-group exhaustion, but does not establish that a completed earlier-group assessment survives later exhaustion or that an earlier `Disproved` assessment takes precedence over a later `ResourceLimit` Unknown. Those are implementation properties of `pub fn complete` that need direct correctness-bearing tests rather than support borrowed from this test's name.

**Fact — the records split the contradiction at this ticket's base.** [The optimizer contract](../docs/compiler/optimizer.md) described the current `semantic-discharge` refusal path and said Unknown is unsupported capability before an executable frontier. [The runtime-execution contract](../docs/research/runtime/runtime-execution-contract.md) likewise said unsupported or over-budget Unknown refuses before an executable plan exists. Those records conflicted with ADR 0078's plan-standing rule. The operation-extension contract's `Missing optional knowledge is conservative` and `compile refusal for index/access lowering` anchors could agree with the fail-closed outcome, but its base-tree `A budget stop is reported only when it is the sole diagnostic` sentence conflicted with atomic assessments that retain ResourceLimit Unknown beside an assessed Disproved and return Disproved overall. The accepted ADR 0109 sweep replaces that sentence with the exact mixed-assessment and precedence rule.

**Fact — adjacent budget findings are not interchangeable.** In `crates/tiler-compiler/src/region.rs`, `RegionBudgetResource::CandidatesPerSeed` and `RegionBudgetResource::Expansions` stop search growth after `fn retain_singleton_coverage`, preserving singleton coverage while reporting lost alternatives. `RegionBudgetResource::Members`, `RegionBudgetResource::BoundaryOutputs`, and `RegionBudgetResource::LiveValues` are shape-admission bounds applied by `fn classify` and `fn classify_shape`; they can reject a fused or whole-program candidate. The `pub(crate) fn budget_stops` accessor retains both region classes, neither of which is an index-domain proof result.

## Required derivation

1. Re-read ADR 0078 at acceptance commit `a17884b0`, including the historical `OccurrenceEvidence::BudgetStopped`, `proof_budget_stop`, pipeline construction and consumption sites, and the correctness-bearing tests that established the accepted behaviour.
2. Re-read the current IR `pub fn complete` and `struct IndexDomainProofLedger` path, compiler `discharge_pending_index_refinement`, `LoweringError::SemanticDischarge`, `lowering_failure`, explain flow, all current construction and consumption sites, and `whole_call_ledger_preserves_earlier_group_and_stops_later_groups_atomically` plus `pending_and_refused_proofs_have_no_executable_coverage_spelling`; repair the former's evidence so its name and assertions establish the same population.
3. Decide with evidence whether to restore ADR 0078's plan-standing behaviour or to ask Tom to supersede that accepted rule with the current before-planning refusal.
4. Independently align the operation-extension contract's sole-diagnostic wording with retained mixed assessments and Disproved precedence, preserving the rule that ResourceLimit never overrides an assessed disproof.
5. Record Tom's accepted narrow supersession with provenance, execute its contract and catalog sweep, and leave public analysis-only results, executable pending coverage, artifact/cache/runtime fallback, and numerical behavior unchanged.

## Non-goals and stop conditions

Do not change crate code, ADR 0078, or the operation-extension contract merely to make a grep pass. Do not describe every `RegionBudgetStop` as search loss. Stop and split work if reconciliation reaches a public boundary, artifact identity, runtime fallback, or numerical-contract decision not already owned here.

## Relationship

This ticket produced the accepted authority and implementation evidence for [`repair-adr-0078s-budget-stop-and-unknown-gap-evidence`](repair-adr-0078s-budget-stop-and-unknown-gap-evidence.md). That repair now depends on the `done` acceptance node, while this ticket records the exact evidence, affected contracts, tests, and intentionally unsupported alternatives.

## Decided — accepted

Tom accepted the narrow supersession on 2026-08-08 in the current Codex session, relayed by the coordinator from Tom's message, “yes you may make the correct decision and accpet the change”. [`accept-adr-0109-fail-closed-on-unknown-index-domain-proof`](accept-adr-0109-fail-closed-on-unknown-index-domain-proof.md) records the acceptance act as `done`; the dependent ADR 0078 evidence repair now depends on that node rather than on this implementation/evidence owner.

The accepted outcome retains ResourceLimit as Unknown, refuses before cover enumeration and executable coverage, preserves assessed Disproved precedence, and retains every produced assessment for explanation. It adds no public non-executable compilation result, executable pending coverage, artifact/cache/runtime fallback, or numerical change.

## Outcome (2026-08-08)

**Facts repaired before source or contract edits.** The historical `proof_budget_stop` population is nonempty and all-`ProofResourceLimit`, not cardinality one. An unassessed obligation may contain a counterexample but remains honestly Unknown until assessed. The former whole-call test's two groups both exhausted at the first group and therefore established atomic filling only; it did not establish retained earlier work or mixed precedence.

**Accepted sweep.** [ADR 0109](../docs/decisions/0109-fail-closed-before-executable-planning-when-index-domain-proof-is-unknown.md) lands accepted with exact provenance and supersedes only ADR 0078's historical `Ok` and “the plan stands” clause. ADR 0078 carries the item-scoped note; the operation-extension contract states mixed assessment retention and Disproved precedence; the optimizer and runtime contracts cite the accepted before-cover authority; and both hand-maintained catalog views carry ADR 0109. The separate acceptance node is `done`, and the ADR 0078 evidence repair depends on it.

**Correctness evidence.** `whole_call_ledger_fills_every_obligation_when_the_first_group_exhausts` now names what the old fixture proved. `whole_call_ledger_preserves_an_earlier_group_before_later_exhaustion` uses one valid completed group and a later verified residual group against the same ledger. `disproof_precedes_later_resource_limit_and_retains_both_assessments` exercises the classification function used by `ResolvedIndexRealization::complete` and checks both retained claims. Shrinking the later group's subject to one point failed with `the later group must exhaust the shared ledger`; reversing the production classifier's precedence failed with `left: Unknown` and `right: Disproved`. Both subjects were restored before the green run.

**Verification.** `cargo fmt --all -- --check`; `cargo check -p tiler-ir`; `cargo nextest run -p tiler-ir` (991 passed); `cargo test -p tiler-ir --doc` (17 passed, one ignored); `cargo clippy -p tiler-ir --all-targets -- -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc -p tiler-ir --no-deps`; the targeted ledger tests; `make citations`; `tkt lint`; and `git diff --check` all pass. The change deliberately adds no compiler cover counter: the production order is already direct in `enumerate_complete_plans` — `resolve_lowering` returns its semantic-discharge failure before the sole `enumerate_covers` call — and adding test-only global instrumentation would not strengthen the end-to-end UnsupportedCapability assertion enough to justify another mutable test seam.

**Unsupported alternatives.** Restoring the historical behavior now requires either an unsound executable coverage mint or a new public analysis-only result. Carrying the gap farther requires artifact/cache identity and a runtime validation/fallback contract. Both are outside this accepted decision and remain unimplemented rather than partially spelled.
