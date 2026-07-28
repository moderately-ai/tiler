---
id: route-the-compile-path-through-the-rewrite-engine
title: Route the compile path through the rewrite engine
status: todo
priority: p1
dependencies: [generalize-the-normalize-transaction-to-alternatives]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites]
---
Split from `generalize-the-normalize-transaction-to-alternatives`, which delivered the engine. The engine exists, satisfies its pin, and **nothing calls it**: `normalize_semantics` is still the compile path's rewrite stage.

## Why this is a separate ticket

Everything in the parent was additive or self-contained — a new module, a provider over existing functions, a round-trip, and an engine, none of which changed what the compiler does. This is a **behaviour change**: the pipeline moves from one canonical program to a set of alternatives. Different risk, and a commit worth having on either side of.

## What exists

- `normalize::run_rewrite_engine(registry, program, budgets)` — collects, revalidates, budgets all-or-nothing, returns alternatives.
- `normalize::CommonSubexpressionRule` — the one rule, verifying its own postconditions before proposing.
- `normalize::revalidate_structurally` — the engine's rule-agnostic half.
- `rewrite::{RuleRegistry, collect_proposals, ProviderDefect, RewriteProposal, RewriteRuleIdentity}`.
- `normalize::EngineFailure` — provider defect versus revalidation failure, the latter naming the rule.

## Expect these, rather than discovering them

**The explain census will move.** `pipeline/tests.rs::every_wired_authority_emits_its_typed_explain_records` counts records per rule. Routing through the engine changes what `normalize.semantics.v1` emits and may add records under the rule identities. Update it in the same change; that test is what catches an unreported stage.

**Plan enumeration takes more than one semantic program.** Today the pipeline normalizes to one program and plans from it. A set of alternatives means either planning from each and comparing, or choosing one before planning. **Choosing one before planning is the cheaper option and is likely wrong**: it makes the rewrite decision without the cost model that would justify it, which is the same error as letting an analytical cost component enter dominance. Prefer planning from each; if that proves too expensive, measure it rather than assuming, and record the measurement.

**`normalize_semantics` and the engine must not both run.** Two rewrite authorities over one program is the second-authority failure `AGENTS.md` names. Either the engine replaces the stage or the stage is expressed through it — not both, and not one wrapping the other while the wrapped one keeps its own budget.

## Closes when

- The compile path's rewrite stage is the engine, with the common-subexpression rule registered.
- `normalize_semantics` is either removed or is a thin expression of the engine, with no second budget or second adoption path.
- The explain census is updated and every rewrite emits a typed record naming its rule identity.
- The serial-sum artifact identity and the producer's two-process determinism test either do not move, or move exactly once with the change stated at the site — a rewrite stage that changes what is compiled changes artifact identity, and that must be a decision rather than a surprise.
- A test drives a plan built from an alternative the engine produced, not only from the unrewritten program.

## Readmission is the hard part, not plan enumeration (2026-07-28)

Reading the call site (`pipeline.rs:420`) changes what this ticket is mostly about.

**Every committed rewrite already re-enters the request boundary.** After normalization the pipeline calls `verify_request` on the rewritten program, and the comment there states why in terms that matter more with alternatives than with one: *"A committed rewrite is a new program, so it must independently re-enter the request boundary rather than inheriting the input's verification... The caller's stated preference is what re-enters, not the contract this run resolved: readmission must repeat the resolution rather than inherit its answer, so a rewrite that changed what the program requires cannot keep a resolution it invalidated."*

Three consequences follow, and none is about plan enumeration:

**An alternative is a (program, verified request) pair, not a program.** Each must be readmitted independently, and readmission *re-resolves* the numerical contract from the caller's stated preference. Two alternatives can therefore resolve to **different** contracts, which means they are not comparable on cost alone — a cheaper alternative under a weaker contract is not better, and the existing rule that estimates from different cost models are incomparable is the precedent for how to treat that.

**Readmission failure is a fault for *every* alternative — I claimed otherwise above and was wrong.** My first pass said one failing readmission is ordinary and only all-failing is a fault. That reasoning does not survive the existing code's own justification: `"Rejection here is invalid compiler output, not an unsupported user program: the input was already admitted."` Every alternative is a semantics-preserving rewrite of a program that was already admitted, so an alternative the boundary rejects means the rewrite changed something it should not have. That is a compiler defect whether it happens to one alternative or all of them, and having other alternatives survive does not make it less of one.

Treating it as "that alternative drops out" would silently discard the evidence of a compiler bug, and it would do so most often exactly when the bug is rarest — a rewrite that misbehaves on one program in a hundred. **Preserve the existing fault semantics per alternative.**

There is a real question underneath, and it should be settled by evidence rather than by picking the convenient answer: the call-site comment anticipates *"a rewrite that changed what the program requires"*, which implies a semantics-preserving rewrite can legitimately alter the resolved numerical contract. If that is true, some readmission failures are legitimate and the fault treatment is too strict. **Do not weaken it on that possibility alone.** Find a rewrite that provably changes what a program requires, or leave the fault in place.

**The unrewritten program must stay in the candidate set.** It is already verified — it entered as the caller's own program — so it needs no readmission and cannot fail one. If every alternative drops out, it is what remains, and that is the path that makes "all failed" recoverable rather than fatal.

*The check, reproducible in one line:* `grep -rn 'normalize_semantics(' crates/tiler-compiler/src/pipeline*` returns exactly one call site; read the 25 lines after it.

**Revised sizing.** The engine call itself is a few lines. The work is readmission-per-alternative, the contract-divergence consequence for comparison, and the ordinary-versus-fault distinction on readmission failure. Budget accordingly, and do the failure distinction first — it is the one with a wrong answer that still compiles.

## Readmission landed (2026-07-28)

`normalize::readmit_alternatives(alternatives, readmit)` pairs each alternative with its own verification and preserves the fault semantics settled above.

It takes the readmission as a closure rather than calling `verify_request` directly. That is not only for testability: the request context — shape environment, stated numerical preferences, budgets, target profiles, capabilities — lives in `pipeline.rs`, and threading it into `normalize` would move the request boundary into the rewrite stage. The caller supplies the readmission; this owns only the *policy* of what a refusal means.

**Two tests, and each exists because the other cannot catch its failure:**

- *Every alternative carries its own readmission.* The stub returns a distinct value per call, so a readmission that verified once and reused the answer fails here. That is the whole reason each is readmitted separately — two alternatives can resolve to different numerical contracts.
- *A refused readmission is a fault, not a dropped alternative.* Replacing the `?` with a `continue` — the exact regression this guards, and the one that looks like a tidy improvement — makes it fail with "a refused readmission was filtered instead of reported". Verified by making that change and watching it fire.

## Contract divergence guarded (2026-07-28)

`normalize::group_by_resolved_contract(alternatives, contract_key)` keeps alternatives that resolved to different numerical contracts out of one another's comparison.

**Why grouping rather than a rule to remember.** A cheaper alternative under a weaker contract is not a better alternative — it is a different answer to a different question, and ranking the two together lets a rewrite buy speed by quietly relaxing what the caller asked for. `PlanStructuralCost::dominates` already applies exactly this rule by returning `false` across differing cost-model keys: incomparable things are kept apart structurally rather than by remembering not to compare them.

**Groups are in first-appearance order over canonically-ordered input**, so the result is deterministic *without* imposing an order on contract keys — which have none, being an open vocabulary. A test pins that: a grouping that sorted by key would be inventing an order, and it fails.

More than one group means the caller chooses *within* a group on cost and *between* groups on the contract. A single group is the ordinary case and means nothing special.

## What remains for this ticket

Call the engine at `pipeline.rs:420` in place of `normalize_semantics`, readmit through `readmit_alternatives`, group through `group_by_resolved_contract`, and carry the result into planning. Every piece that step needs now exists and is tested; what is left is the wiring and the behaviour change it causes — the explain census, and whether the serial-sum artifact identity moves.

## The wiring is not a drop-in, and here is exactly why (2026-07-28)

I expected to swap the call at `pipeline.rs:420` for a behaviour-preserving first step — only the common-subexpression rule is registered, so the engine yields at most one alternative and the pin guarantees it is the same program. That swap does not work, and the reason is worth having before someone attempts it.

**`NormalizationOutcome` is not a local variable; it flows three levels down and emits the explain record itself.** Tracing every mention in `pipeline.rs`:

- `420` — produced by `normalize_semantics`.
- `422`, `445` — passed to `compile_verified` on both the rewrote and did-not-rewrite paths.
- `451`, `459` — threaded through to `compile_target`.
- `468`, `471` — threaded through to `compile_target_with_explain`.
- `797–798` — **`normalization.record(explain, ...)`** produces the normalization explain record, and `815`, `820`, `825` make that record the *cause* of the region-candidate enumeration records that follow.

So the outcome is not a result the pipeline consumes and discards. It is the head of the explain causal chain for everything downstream of normalization.

**The engine returns `Vec<RewriteProposal<SemanticProgram>>` and has no such record.** Wiring it in therefore requires the outcome type to be generalized to carry rule identities and, eventually, more than one alternative — which is what will move the explain census, and now with the mechanism named rather than predicted.

*The check, reproducible in one line:* `grep -n 'normalization' crates/tiler-compiler/src/pipeline.rs` returns the eleven sites above; read line 797 and the causal wiring at 815–825.

**And generalizing the outcome is itself not what it sounds like.** Reading `NormalizationOutcome::record`, its records are **rule-specific**: a `normalize.shared-value-identity` assessment per merge, carrying `canonical-operation` and `merged-operation` facts drawn straight from `SharedValueMerge`. An outcome holding alternatives from arbitrary rules cannot emit those — a rule that does not merge has no canonical or merged operation to report.

So explain emission belongs to the **rule**, exactly as postcondition verification turned out to. `RewriteRuleProvider` needs a way to emit its own records, or a proposal must carry a rule-specific explain payload the engine threads without interpreting.

## The shape of the remaining design, stated once

Three attempts to generalize this stage have each found the same division, and naming it should stop the fourth from rediscovering it:

| Per **rule** | Per **engine** |
| --- | --- |
| postcondition verification (`verify_normalized` needs the `Congruence`) | the transaction and its all-or-nothing contract |
| explain emission and the facts reported (`canonical-operation`, `merged-operation`) | the budget, counted in proposals |
| what counts as an applicable program | structural revalidation (`revalidate_structurally`) |
| | attribution and provider-defect reporting |
| | readmission policy and contract grouping |

The engine's column is built and tested. The rule's column is what `normalize.rs` currently does inline for its one rule, and generalizing the stage means moving those three things behind the provider trait — not widening a struct.

**Revised order:** give `RewriteRuleProvider` an explain-emitting method and move `NormalizationOutcome`'s per-merge records behind it, keeping the stage-level records (budget stop, summary) with the engine. Then the outcome type generalizes to a thin carrier, and the swap at `420` is small. The census moves once, in that change.

## How the rule emits explain, derived (2026-07-28)

The revised order above says "give `RewriteRuleProvider` an explain-emitting method". Working out what that method looks like eliminates the obvious form.

**Rejected: emit during `propose`.** Pass `&mut ExplainWriter` and a cause into `propose`, and let the rule record as it works. It is the smallest change and it is wrong: a proposal may be **abandoned by the budget** or **rejected by structural revalidation** after `propose` returns. Records emitted during proposal would claim a rewrite that never happened, and the abandoned-run case is exactly where the explain output matters most. Today `normalize_semantics` records *after* adopting, and that ordering is load-bearing rather than incidental.

**Rejected: reconstruct the facts from the candidate.** The engine holds only the candidate program, and a `SharedValueMerge` names graph-local ordinals of the *original*. Recovering them means re-running detection, which is both wasteful and a second authority over what the rule already decided.

**What survives: the proposal carries a rule-supplied explain payload the engine emits only for survivors.** A `RewriteProposal` gains something like a `Box<dyn RuleExplain>`, where `RuleExplain::record(&self, explain, cause) -> Result<ExplainRecordId, ExplainError>` is implemented by the rule over facts it captured while proposing. The engine threads it opaquely — it never interprets the payload, which is what keeps rule-specific facts out of a generic type — and calls it only for alternatives that survived revalidation and the budget.

That preserves the record-after-adopting order, keeps `canonical-operation`/`merged-operation` in CSE's hands, and gives the engine exactly one new obligation: do not emit for a proposal you discarded.

**Blast radius, so it is budgeted rather than discovered:** `RewriteProposal` gains a field, so its constructor and the four test providers in `rewrite.rs` change; `collect_proposals` and `run_rewrite_engine` thread the payload; `CommonSubexpressionRule` captures its merges and implements `RuleExplain`; `NormalizationOutcome` sheds its per-merge records and keeps the stage-level ones. The pin and the readmission tests are unaffected — they compare programs, not records.

## The explain seam landed (2026-07-28)

`rewrite::RuleExplain` and `RewriteProposal::with_explain` / `explain()`, plus `rewrite::record_adopted_alternatives`.

The payload is `Option<Arc<dyn RuleExplain>>`. `Arc` because `RewriteProposal` derives `Clone` and a `Box` would not; `Option` because a rule with nothing rule-specific to report is legitimate — `None` means "this rule adds no records of its own", never "this rewrite went unrecorded", and the stage-level records are emitted regardless.

`record_adopted_alternatives` carries the obligation the type system cannot: **pass only survivors.** A proposal abandoned by the budget or rejected by revalidation must never reach it, because its payload describes a rewrite that did not happen. That is stated at the function rather than left to the caller's memory, and it is why alternatives are threaded through revalidation before they arrive.

**One honest gap, recorded rather than implied.** `record_adopted_alternatives` itself is **untested**: constructing an `ExplainWriter` needs a `VerifiedTargetRequest`, which `rewrite.rs` has no fixture for and should not grow one for. The emission loop belongs to the routing change and is covered there, against a real writer. What is pinned here is what routing depends on — that a rule's payload reaches its own proposal and does not leak to a sibling.

## The rule's explain moved behind the trait (2026-07-28)

`normalize::SharedValueExplain` holds the committed merges and implements `RuleExplain`. `CommonSubexpressionRule::propose` attaches it to its proposal, so the records travel with the rewrite and are emitted only if it is adopted.

**One implementation, two callers.** `NormalizationOutcome::record` now calls it rather than carrying its own copy of the loop. Extracted rather than duplicated because two code paths writing the same governed records under the same rule key would drift, and an explain reader cannot tell which path produced a record — the drift would be invisible.

**The extraction is byte-identical, and the existing census proves it.** `pipeline/tests.rs::every_wired_authority_emits_its_typed_explain_records` counts records per rule and is unchanged; a moved emission that altered a count, a key, or a fact would have failed it. That is a stronger check than any test written for the extraction would have been, because it was written by someone who did not know the extraction was coming.

**Remaining, and now only the pipeline edit:** call the engine at `pipeline.rs:420`, readmit through `readmit_alternatives`, group through `group_by_resolved_contract`, emit through `record_adopted_alternatives`, and have `NormalizationOutcome` shed the per-merge call once the engine owns it. The census moves in that change — and only then, since everything up to here left it untouched.

## The last structural step: `NormalizationOutcome` must shed `merges` (2026-07-28)

Attempting the pipeline edit surfaces the final entanglement, and it is worth having stated because it determines the shape of the edit rather than being a detail inside it.

`compile_verified` takes `&NormalizationOutcome`, so routing must still produce one. But the outcome currently *holds* `Vec<SharedValueMerge>` — rule-specific data the engine deliberately does not have, since the merges now travel opaquely in the proposal's `RuleExplain` payload. An engine-produced outcome cannot fill that field.

**So the outcome must stop holding merges.** Two consequences:

- Its `record` no longer constructs a `SharedValueExplain`; the per-merge records come from the adopted proposal's payload via `record_adopted_alternatives`. The outcome keeps only stage-level facts — budget stop, operations before and after, contract key, canonical digest.
- Its `rewrite-count` fact, currently `merges.len()`, becomes the count of adopted alternatives. That is the same number today, with one rule producing one proposal, which is why the census will not move on this field alone — but it is a different quantity, and the difference becomes visible the first time a second rule is registered.

`NormalizationOutcome::merges()` is `#[cfg(test)]`-only, so the test surface shrinks with it.

**Done 2026-07-28, and the ordering worry turned out to be avoidable.** I expected the shed and the swap to be inseparable — the shed alone leaving nothing to emit per-merge records. A third option avoids that: the outcome sheds `merges` and gains `rule_explains: Vec<Arc<dyn RuleExplain>>` plus a `rewrite_count`. `normalize_semantics` still builds it, supplying a `SharedValueExplain`, so **nothing about the pipeline changed and the census did not move**.

That is the structural step that was actually blocking. The outcome now holds what any rule can supply and reads none of it, so an engine driving arbitrary rules can produce one — which is what `compile_verified` requires and what the merge field made impossible.

Two consequences worth having recorded:

- **`rewrite-count` is now the adopted-rewrite count, not the merge count.** The same number today with one rule; a different quantity the moment a second is registered.
- **The merge-contents assertion moved to the rule.** `NormalizationOutcome` could assert exactly which operations merged; it no longer can, because the payloads are opaque to it *by design*. That assertion now runs against `detect_shared_values` directly, which is the level that still owns the vocabulary. This is a real relocation rather than a deletion, and the coverage is the same.

**What remains is now genuinely small:** at `pipeline.rs:420`, build the outcome from `run_rewrite_engine`'s adopted alternatives instead of from the congruence — readmitting through `readmit_alternatives`, grouping through `group_by_resolved_contract`, and taking each alternative's `explain()` as the outcome's `rule_explains`. The census moves in that change, once, when more than one alternative can survive.
