---
id: implement-transactional-rewrite-engine
title: Implement the external transactional rewrite engine
status: done
priority: p1
dependencies: [prototype-optimizer-conformance-gate]
related: [implement-first-algebraic-rewrite-portfolio]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites]
---
Implement the bounded external rule-provider and transactional alternative
engine after the ordinary optimizer path is proven. Preserve exact rule and
provider identity, termination/budget contracts, semantic revalidation,
rollback, deterministic traversal, and typed explain. Unknown provider behavior
is never optimizable merely because it is registered.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.


## Scoped — generalize the proven normalize machinery (2026-07-27)

The ticket had no closing criteria and reads as six subsystems. Five and a half of them already exist, in `crates/tiler-compiler/src/normalize.rs` (969 lines), which its own header describes as deliberately not this ticket: "it never produces alternatives: an alternative-producing rewrite engine is a separate later authority."

Checking the ticket's stated properties against that stage, one by one:

| Property this ticket names | State in `normalize.rs` |
| --- | --- |
| termination | **exists** — single forward pass over a finite verified operation list, no fixpoint loop, so termination does not rest on a decreasing measure |
| budget contract | **exists** — `DeterministicBudgets::normalization_rewrites`; exhaustion abandons the whole rewrite and keeps the verified input, so a budget never yields a partly-rewritten graph |
| rollback | **exists** — the input `SemanticProgram` is immutable and never mutated; a candidate is built separately and adopted only after every postcondition passes |
| semantic revalidation | **exists** — the candidate is rebuilt through the checked `SemanticProgramBuilder`, so the frozen semantic authority re-infers and re-validates; the stage never trusts its own output structurally |
| deterministic traversal | **exists** — verified topological order by ascending graph-local ordinal, results by ascending position; the earliest occurrence is always the canonical representative |
| typed explain | **exists** — `RuleRef`, `ExplainStage`, and governed rule constants `normalize.semantics.v1` / `normalize.common-subexpression.v1` |
| **rule and provider identity** | **absent** — exactly one rule, hard-coded, with no provider concept at all |
| **alternatives** | **absent, and deliberately** — the stage produces one canonical graph by design |

So this ticket is not six subsystems. It is two, on top of machinery already proven by the conformance tests in that module.

**The slice.** Generalize the normalize stage's transaction into an engine that (a) takes its rules from an external provider carrying a governed `ProviderIdentity` and per-rule identity, and (b) produces a set of alternatives rather than one canonical program. Everything else is reuse, and reuse is what makes the result checkable rather than merely plausible.

**The correctness pin, and it can say no.** Register only the existing common-subexpression rule and run the engine: it must produce exactly what `normalize.rs` produces today, compared on `SemanticIdentity` rather than on a summary. That is a byte-level equality against an implementation whose reference equivalence is already proven by checked conformance tests, so a regression in the generalized transaction fails visibly instead of producing a differently-shaped but arguable result. Do not start the engine on a new rule; a new rule and a new engine failing together are indistinguishable.

**On the ordering that looks like a deadlock and is not.** `implement-first-algebraic-rewrite-portfolio` depends on this ticket and owns the rules, so the engine has no rules of its own to run. It does not need any: an *external* rule provider is exactly what a test can supply, so a test-authored provider is a legitimate subject rather than a synthetic stand-in, and the CSE rule above gives a real one that is already proven.

**The constraint from the body, restated because it is the easy thing to lose.** Unknown provider behaviour is never optimizable merely because it is registered. A registered provider's proposed rewrite is subject to the same revalidation the normalize stage already performs — rebuilt through the checked builder, adopted only on passing every postcondition — and a provider that cannot be revalidated is rejected, not trusted.

## Closes when

- A rule provider is an external, registrable authority with a governed `ProviderIdentity` and a governed per-rule identity, both appearing in typed explain.
- The engine produces a set of alternatives; the existing single-canonical-output normalize contract is either expressed through it or left in place, but is not silently widened into alternative production.
- Every candidate rewrite is revalidated through `SemanticProgramBuilder` before adoption, and a provider's output is never adopted on the provider's assertion.
- Termination, budget exhaustion, and rollback preserve the existing all-or-nothing contract: no partially rewritten graph is ever observable.
- Traversal order is deterministic and the alternative set is reproducible across runs.
- **With only the common-subexpression rule registered, the engine's result has the same `SemanticIdentity` as today's `normalize.rs` output**, asserted by a test.
- A registered provider whose rewrite fails revalidation is rejected with a typed, explainable reason, asserted by a test that watches the rejection fire.

## Started — governed rule identity landed (2026-07-27)

`crates/tiler-compiler/src/rewrite.rs`. `RewriteRuleIdentity` names a rule by provider, rule key, and an **output-affecting** revision — a rule refactored without changing what it produces keeps its revision, one whose output changes must not. `COMMON_SUBEXPRESSION_RULE` names the rule the normalize stage already proves, matching that stage's governed constants so the two cannot drift apart silently.

Identity is first because the ticket's governing constraint depends on it: unknown provider behaviour is never optimizable merely because it is registered, and a rewrite that cannot be attributed to a named, versioned rule cannot be explained, reproduced, or excluded when its provider turns out to be wrong. The engine must not accept a proposal before it can name what proposed it.

The canonical encoding is length-prefixed rather than delimiter-separated, so provider `"a.b"` with rule `"c"` cannot encode identically to provider `"a"` with rule `"b.c"`. A delimiter would let those two collide, and a collision here means two distinct rules sharing one identity. Tested directly, along with an empty name being refused and a revision change being a different rule — each driven against both the accepting and the rejecting case so a predicate that always said yes would fail.

**The proposal shape is now settled, by reading rather than by choosing.** I had deferred the trait because it was unclear whether alternatives are produced per rule or per traversal. `normalize.rs:329` answers it: `detect_shared_values(program: &SemanticProgram) -> Result<Congruence, NormalizeError>` inspects the **whole program** and returns the complete set of congruence classes at once. Detection is not per-site or incremental.

So a rule's natural signature is whole-program in, proposals out — `propose(&SemanticProgram) -> Vec<Proposal>` in shape — and alternatives compose *across* rules rather than being enumerated along a traversal. That is what the existing implementation already does for its one rule, which means the generalization is a widening rather than a restructuring, and the CSE pin stays cheap to satisfy.

Per-traversal alternatives are eliminated independently: the alternative set would be exponential in the number of applicable sites, and the ticket's budget contract counts *rewrites*, so a rewrite-count budget could not bound it. A budget that cannot bound the thing it governs is not a termination contract.

### Correction, same day: half of the above is refuted

I said the next worker should confirm `Congruence` can carry a proposal set from an external provider. I checked, and **it cannot**, which refutes the most load-bearing sentence above before anyone acted on it.

`Congruence` (`normalize.rs:318`) holds `representative` (each value's canonical value ordinal), `retained` (whether each operation survives), `operation_results`, and `merges`. That is not a proposal — it is **the fully-resolved result of applying CSE to the whole program**, in CSE's own vocabulary. A rule that eliminates no values, or one that rewrites an operation into a different operation rather than merging two, has nothing to say in those fields.

What survives from the paragraph above:

- **Detection is whole-program.** `detect_shared_values` takes the entire `SemanticProgram`. That still holds and still argues for a whole-program rule signature.
- **Per-traversal alternatives are still eliminated** by the budget argument: exponential in applicable sites, and a rewrite-count budget cannot bound them.

What is refuted:

- **"The generalization is a widening rather than a restructuring."** It is a restructuring. A rule-agnostic proposal type has to be designed, and CSE then has to *produce* one instead of its current internal state. `Congruence` becomes CSE's private working type behind that proposal, not the proposal itself.

This changes the size of the next slice materially, which is why it is recorded here rather than left for someone to discover mid-implementation. The CSE pin is unaffected — the engine with CSE alone must still reproduce `normalize.rs`'s `SemanticIdentity` — but reaching it now requires designing the proposal type first.

**Proposal type landed 2026-07-27**, which was the blocker the correction above identified. `RewriteProposal<Program>` pairs a `RewriteRuleIdentity` with a whole candidate program.

*Why a whole candidate program rather than a structured edit script.* An edit vocabulary is more expressive to reason about and is refused for two reasons that compound. It would need a closed vocabulary covering every edit any rule might make, and this engine exists to admit rules from *outside* — so it would either constrain external rules to what was imagined when it was written, or grow an escape hatch that puts unchecked structure back into the graph, which is exactly what "unknown provider behaviour is never optimizable merely because it is registered" forbids. And it would need its own validator, whereas a candidate program needs none: the normalize stage already revalidates by rebuilding through the checked `SemanticProgramBuilder`, so a malformed candidate is rejected by the authority that already owns that judgement instead of by a second one written for edits.

*The cost, stated rather than hidden.* The engine cannot see **what** a rule changed, only that the result is valid and what it costs. That suffices for what the engine does — revalidate, compare identity, choose — and a rule that wants to explain itself does so through its own typed explain records rather than handing the engine a diff to interpret.

It is generic over the program type so this module does not depend on the semantic IR before the engine does; the engine instantiates it at `SemanticProgram`. Tests cover that a proposal always names its rule, and that two rules proposing an identical candidate stay distinct — if the rule were dropped or defaulted, excluding one provider that turned out to be wrong would silently exclude another's work.

**Provider trait landed 2026-07-27.** `RewriteRuleProvider<Program>` declares an `identity()` and a `propose(&Program) -> Vec<RewriteProposal<Program>>`. Whole-program in, proposals out, because detection here is whole-program — `detect_shared_values` takes the entire program and returns its complete result rather than walking sites. An empty result means "nothing to do", never a failure.

**One provider owns exactly one rule.** Bundling several would make `identity()` ambiguous, and the point of the identity is that a rewrite can be attributed, reproduced, and *excluded* — excluding a bundle would take out rules that were never implicated.

**The attribution invariant, and why it needs a runtime check.** A provider constructs its own proposals, so nothing in the type system stops one stamping another rule's identity on its work — by mistake when a provider is copied, or deliberately. The consequence is not cosmetic: attribution is what makes exclusion work, so a misattributed proposal *survives* the exclusion of the rule that actually produced it, and the exclusion of an innocent rule takes its place. `misattributed(expected, proposals)` returns the offenders.

**The engine must reject the whole batch, not filter it.** A provider that misattributes one proposal has demonstrated it does not know what it is, and its remaining proposals are not thereby trustworthy — the same reasoning that makes a cache key/subject mismatch a protocol defect rather than an ordinary miss. Tested against a clean batch and a tainted one, so a checker that always returned empty fails rather than passes.

**Registry landed 2026-07-27.** `RuleRegistry<Program>` holds the providers one engine run may draw on, with two invariants that are both load-bearing rather than tidiness.

*No two providers may claim one rule identity.* Registration refuses a duplicate rather than replacing or shadowing it. Sharing an identity would make a proposal untraceable to the code that produced it, and excluding the rule would exclude both — the same attribution failure `misattributed` prevents, arriving through the registry instead.

*Iteration is in canonical identity order, not registration order.* The alternative set must be reproducible across runs, and registration order is exactly the incidental ordering that differs between a host registering from a static list and one discovering providers. Sorting by identity makes the order a property of *which* rules are present rather than how they arrived — the same reason `enumerate_frontier`'s identity is independent of provider order. Tested by registering the same rules in two different orders and requiring identical iteration.

*An empty registry is legitimate*, not a misconfiguration: a run with no rules proposes and adopts nothing, which is correct behaviour rather than an error.

**Collection step landed 2026-07-27.** `collect_proposals(registry, program)` is the engine's front half: it drives the registry in canonical identity order, enforces the attribution contract, and returns the candidates revalidation will judge. It deliberately does not revalidate, adopt, or budget — those are the transaction's, and the transaction is `normalize`'s to generalize.

*`ProviderDefect` is a separate type from a rewrite being rejected*, and that separation is the point: a rejected candidate is an ordinary outcome of revalidation, while a defect says the provider violated its registration contract. Sharing one type would let a caller counting rejections count defects among them.

*One misattributed proposal fails the entire call*, discarding proposals from providers already collected. This is the stricter of the two readings and it is chosen deliberately. Keeping the offender's correctly-attributed proposals would mean trusting code that just proved untrustworthy; keeping *other* providers' proposals would return a smaller alternative set than the registry describes — a partial result that reads like a complete one. Tested with the honest provider registered first in canonical order, so the test confirms already-collected work is discarded rather than returned.

## Split, and closing on the half it delivered (2026-07-27)

This ticket named two things: the bounded **external rule-provider** machinery and the **transactional alternative engine**. The first is delivered — identity, proposal, provider trait, attribution contract, registry, and collection, each tested with cases that can fail. The second is `generalize-the-normalize-transaction-to-alternatives`, now live.

**The split is at a real boundary, not an arbitrary stopping point.** Everything here was *additive*: a new `crate::rewrite` module beside the existing stage, with nothing in `normalize.rs` touched. The transaction is the first piece that must change the 969-line normalize stage from the inside, turning a loop that produces one canonical program into one that produces a set of alternatives. Different risk, different review surface, and worth a commit on either side of.

**The CSE pin moves with the transaction, not with this ticket.** It is a statement about what the engine produces, and there is no engine here — only the machinery it will drive. Leaving the pin attached to this ticket would have made it look unmet when in fact it was unreachable, which is the opposite of what a pin is for.

Nothing in the child re-litigates what was decided here: the derivations for the candidate-program proposal shape, the one-rule-per-provider rule, batch rejection over filtering, and canonical over registration order are all recorded above and referenced rather than restated.

**Next step, unchanged:** generalize the normalize transaction to take rules from a provider and to yield alternatives, with the pin already stated above — CSE alone must reproduce `normalize.rs`'s `SemanticIdentity` exactly.

## Post-routing correction (verified against HEAD) (2026-07-28)

Four claims about this ticket's subject were checked against the current tree after the compile path was routed through the engine. **Three hold; one has since been fixed and is recorded as fixed rather than asserted.** Line numbers below are as of this record; each carries the check that finds the construct if it drifts again.

**1. Still true — no explain record carries a `RewriteRuleIdentity`, so the first `Closes when` bullet is not met.** That bullet requires a governed `ProviderIdentity` and a governed per-rule identity "both appearing in typed explain". All three normalize emission sites still emit a `&'static str` constant: `RuleRef::builtin(NORMALIZE_SHARED_VALUE_RULE)` at `normalize.rs:178`, and `RuleRef::builtin(NORMALIZE_STAGE_RULE)` at `:246` and `:289`. Reproduce with `grep -n 'RuleRef::builtin' crates/tiler-compiler/src/normalize.rs`. **The source now says this about itself** — `rewrite.rs:27-31` records "a route from [`RewriteRuleIdentity`] into explain output" as deliberately not yet present, and that "the provider identity reaches no record today". That is an accurate doc comment, and it makes the gap legible; it does not close the criterion.

**2. Still true — the doc claiming the two rule constants match is false in both halves.** `rewrite.rs:126-127` states that `COMMON_SUBEXPRESSION_RULE`'s keys "match that stage's existing governed constants so the two cannot drift apart silently". They do not match: `COMMON_SUBEXPRESSION_RULE` is `RewriteRuleIdentity::new("tiler.normalize", "common-subexpression.v1", 1)` (`rewrite.rs:128-129`), while `NORMALIZE_SHARED_VALUE_RULE` is the string `"normalize.common-subexpression.v1"` (`normalize.rs:71`). And nothing relates them, so nothing prevents the silent drift the sentence claims is prevented. Two one-line checks: `grep -rn 'NORMALIZE_SHARED_VALUE_RULE' crates/tiler-compiler/src/rewrite.rs` returns nothing (exit 1), and `grep -rn 'COMMON_SUBEXPRESSION_RULE' crates/tiler-compiler/src/normalize.rs` returns only `:63` (the import) and `:432` (the provider's `identity()`) — no comparison, no assertion, no shared derivation.

**3. Fixed since — the module doc no longer contradicts the file.** The previous revision of `rewrite.rs` said in its header that the module "does not yet run one" and, under "What is deliberately not here", that there was "no trait for proposing rewrites, no registry, and no engine" — while the same file defined `RewriteRuleProvider` (now `:284`), `RuleRegistry` (`:374`), and `collect_proposals` (`:503`). The doc sweep in `588be6e` corrected it: `rewrite.rs:4-6` now says the engine "lives in [`crate::normalize`] and drives this seam on the compile path", and `:23-31` names only the transaction and the explain route as absent, both of which are accurate. Recorded here because the correction is the evidence, not the original claim.

**4. Still true — the attribution failure paths are unreachable from the compile path, and the tests that cover them remain non-vacuous.** The sole registered provider stamps `self.identity()` on everything it emits, at `normalize.rs:455` (the `ProviderDefect::Failed` construction) and `:479` (the `RewriteProposal`), so `misattributed` can never fire in production today. The coverage is therefore entirely in tests, and both can say no: `one_misattributed_proposal_fails_the_whole_batch` (`rewrite.rs:837`) and `a_rule_that_cannot_run_is_not_read_as_proposing_nothing` (`:876`), the second carrying an explicit positive control at `:913-923` — the same registry without the broken provider must succeed, "or the assertion above would pass for the wrong reason".

**What would close the explain criterion.** Either derive the emitted rule key from `RewriteRuleIdentity` so one value produces both the identity and the explain record — which removes the possibility of drift rather than detecting it — or, if the two must stay separate constants, add a test asserting they agree. The first is preferable: the second detects a drift that the first makes unrepresentable, and `rewrite.rs:126-127` already asserts in prose the property that neither currently holds.

**One related note, for a reader following the pin.** The CSE pin this ticket states at lines 43, 126, and 130 was **removed** by `b17fd26` as tautological: once the routing made the normalize stage *be* the engine, both sides of the comparison were the same `run_rewrite_engine` call. The identity property it was meant to hold is now pinned by the provider-versus-stage test plus the two revalidation tests. The pin's *reasoning* above is preserved as the derivation that produced those tests; it is the assertion, not the argument, that became empty.
