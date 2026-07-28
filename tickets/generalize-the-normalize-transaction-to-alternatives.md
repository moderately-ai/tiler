---
id: generalize-the-normalize-transaction-to-alternatives
title: Generalize the normalize transaction to drive providers and yield alternatives
status: done
priority: p1
dependencies: [implement-transactional-rewrite-engine]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites]
---
Split from `implement-transactional-rewrite-engine`, which delivered the external rule-provider machinery. This ticket is the other half: the transaction itself.

## Why this is a separate ticket and not the tail of the last one

Everything landed so far was **additive** — a new `crate::rewrite` module beside the existing stage, with nothing in `normalize.rs` touched. This piece is the first that must change the 969-line normalize stage from the inside, converting a loop that produces one canonical program into one that produces a set of alternatives. Different risk, different review surface, and a boundary worth having a commit on either side of.

## What already exists, and should not be rebuilt

In `crates/tiler-compiler/src/rewrite.rs`:

- `RewriteRuleIdentity` — provider, rule key, output-affecting revision; length-prefixed canonical encoding so a dotted provider cannot collide with a dotted rule.
- `RewriteProposal<Program>` — a rule identity paired with a whole candidate program. Deliberately not an edit script; see that type's own derivation.
- `RewriteRuleProvider<Program>` — `identity()` and `propose(&Program) -> Vec<RewriteProposal<Program>>`. One provider owns exactly one rule.
- `misattributed` and `ProviderDefect` — the attribution contract. A defect is deliberately a different type from a rejected rewrite.
- `RuleRegistry<Program>` — refuses duplicate rule identities, iterates in canonical identity order rather than registration order.
- `collect_proposals` — drives the registry, enforces attribution, fails the whole batch on one misattributed proposal.

All six are tested with cases that can fail, not only cases that pass.

## What normalize already provides, and should also not be rebuilt

The stage's header documents six properties this transaction needs and already has, for one hard-coded rule: termination (single forward pass, no fixpoint, so termination does not rest on a decreasing measure), budgets (`DeterministicBudgets::normalization_rewrites`, exhaustion abandons the whole rewrite and keeps the verified input), rollback (input `SemanticProgram` immutable, candidate adopted only after every postcondition), semantic revalidation (rebuilt through the checked `SemanticProgramBuilder`, never trusting its own output structurally), deterministic traversal (verified topological order by ascending graph-local ordinal), and typed explain (`RuleRef`, governed rule constants).

The work is to make those six serve *registered* rules and *multiple* outcomes, not to reimplement them.

## Closes when

- The transaction consumes `collect_proposals` output and revalidates each candidate through `SemanticProgramBuilder` before adoption, so no provider's output is adopted on the provider's assertion.
- It yields a set of alternatives; the existing single-canonical-output contract is either expressed through it or left in place, but is not silently widened into alternative production.
- Termination, budget exhaustion, and rollback keep the all-or-nothing contract: no partially rewritten graph is ever observable, and a budget stop abandons rather than half-applies.
- The alternative set is reproducible across runs.
- A `ProviderDefect` is reported as a typed, explainable failure distinct from an ordinary rewrite rejection, asserted by a test that watches it fire.
- **The pin: with only the common-subexpression rule registered, the result has the same `SemanticIdentity` as today's `normalize.rs` output.** Compared on the identity, not a summary. Do not start the engine on a new rule — a new rule and a new transaction failing together are indistinguishable.

## The provider trait needs a fallible `propose` — found 2026-07-28

Sketching the CSE provider against the real functions surfaced a defect in the seam that landed with `implement-transactional-rewrite-engine`. Recording it here because this ticket is where it gets fixed, and because the fix changes a signature its tests depend on.

**The shape is clean.** `normalize.rs` separates cleanly at exactly the right line:

- `detect_shared_values(program) -> Result<Congruence, NormalizeError>` (line 329)
- `rebuild(program, &congruence) -> Result<SemanticProgram, NormalizeError>` (line 421)
- `normalize_semantics` (line 253) is the transaction: budget, verify, adopt.

A CSE provider is `detect_shared_values` then `rebuild`, returning the candidate. The transaction stays where it is. That is the whole of the first slice, and it is smaller than the ticket implied.

**The defect.** `RewriteRuleProvider::propose` returns `Vec<RewriteProposal<Program>>` with no error channel. Both functions above return `Result`, and their errors are *compiler faults* — `NormalizeError::Rebuild { rule: "builder-create" }` is a builder that would not construct, not a program with nothing to optimize. With the current signature the provider must swallow those into an empty vector, making **"detection failed" indistinguishable from "nothing to propose"**.

That is the same class of bug as `Unknown` reported as zero, and it is worse here: an empty proposal set is the *normal* result for most rules on most programs, so the failure is invisible by construction and no counter would show it.

**The fix, and its blast radius.** `propose` should return `Result<Vec<RewriteProposal<Program>>, ProviderDefect>` — reusing the existing defect type, since a rule that cannot run is a contract violation of the same kind as one that misattributes. `collect_proposals` already returns `Result<_, ProviderDefect>` and already fails the whole batch, so its body absorbs this with one `?`; what changes is the trait signature and the four test providers in `rewrite.rs` that implement it.

**Fixed 2026-07-28, before writing the provider.** `propose` now returns `Result<Vec<RewriteProposal<Program>>, ProviderDefect>`, with a new `ProviderDefect::Failed { rule, reason }` carrying a stable reason code from the rule's own error vocabulary. `Ok(vec![])` means "nothing to do here"; `Err` means the rule could not run.

`collect_proposals` absorbed it with one `?` and keeps its all-or-nothing behaviour, so a rule that cannot run discards the batch exactly as a misattributed proposal does. The test registers the failing provider *second* in canonical order, so it also confirms the first provider's proposals are discarded rather than returned as a partial result, and it asserts the same registry *without* the broken provider succeeds — otherwise the failure assertion would pass for the wrong reason.

The four test providers in `rewrite.rs` were updated with it. This ticket's first slice is now unobstructed: write the CSE provider over `detect_shared_values` and `rebuild`, returning `Err(ProviderDefect::Failed { .. })` on a `NormalizeError` rather than an empty vector.

## The CSE provider landed, and the pin is live (2026-07-28)

`normalize::CommonSubexpressionRule` implements `RewriteRuleProvider<SemanticProgram>` over this stage's own `detect_shared_values` and `rebuild`. It performs **no** part of the transaction — no budget, no revalidation, no adoption — so a proposal it returns is a candidate nothing has yet accepted.

Three outcomes kept distinct, which is what the fallible signature bought:

- no merges → `Ok(vec![])`, the ordinary case;
- detection or rebuild failed → `Err(ProviderDefect::Failed)` carrying the `NormalizeError`'s own stable reason;
- merges found → one candidate.

**The empty case is checked before rebuilding, not after.** Rebuilding a program with no merges yields a copy semantically identical to its input, so proposing it would have the engine revalidate and compare a program that cannot differ — and worse, a rule that always has a proposal available makes "this rule applies" meaningless. A second test pins that a program with nothing to merge proposes nothing, and asserts the fixture genuinely has no merge first, so it cannot pass against a broken fixture.

**The pin is live and verified to fail.** `the_provider_proposes_exactly_what_this_stage_produces` compares the **canonical bytes** of the candidate's `SemanticIdentity` against `normalize_semantics`'s normalized program — the bytes rather than a digest, so a collision cannot make two different programs compare equal, and the identity rather than a merge count, since two different programs can share one. Replacing the rebuild with a clone of the input makes it fail with "the provider's candidate differs from this stage's normalized program", so it is a check that can say no.

## Where revalidation splits, found by trying to write the engine (2026-07-28)

Attempting the transaction surfaced the division the earlier notes had not made, and it changes the design rather than just the schedule.

**`verify_normalized(original, normalized, congruence)` cannot move into the engine.** Its postconditions are stated in terms of the `Congruence` — an operation count of exactly `original - merges` — and a generic engine holds no congruence for an arbitrary rule. A rule that rewrites without merging anything has no meaningful value to put there. So this check is the **rule's**, not the engine's.

That splits revalidation in two, and both halves are needed:

- **The rule's postconditions**, checked inside `propose` before a candidate is returned. This is the concrete form of "unknown provider behaviour is never optimizable merely because it is registered" — a rule that proposes a candidate it has not checked is asking the engine to trust it.
- **The engine's structural revalidation**, rebuilding through the checked `SemanticProgramBuilder` so the frozen semantic authority re-infers and re-validates every operation. Rule-agnostic, and a different question from whether the rule did what it claims.

**Applied immediately:** `CommonSubexpressionRule::propose` now calls `verify_normalized` before returning, which it did not when it landed earlier today. Without it the provider returned an unverified candidate where `normalize_semantics` verifies before adopting — a real weakening of the existing contract, introduced by moving the rule out of the stage and not visible in any test, since the pin compares against a program that happens to be correct.

## The engine's half needs a function that does not exist (2026-07-28)

The remaining work is smaller than "write the engine" and has one precise prerequisite.

**Structural revalidation has no generic entry point.** `SemanticProgramBuilder` offers `try_new(registry)` (`semantic/program.rs:421`) and `build()` (line 684), with operations added in between. There is no `SemanticProgram` → builder → `SemanticProgram` round-trip: nothing walks an arbitrary program's operations and re-adds them through the checked builder.

*The check, reproducible in one line:* `grep -rn 'fn revalidate\|fn rebuild_program\|fn from_program\|fn reconstruct' crates/tiler-ir/src/semantic/` returns nothing.

`normalize::rebuild` does exactly this walk, but it is CSE-shaped — it consults the `Congruence` to decide which operations survive and how values are remapped. The engine needs the same walk with no congruence: re-add everything, remap nothing, and let the frozen semantic authority re-infer and re-validate.

**So the order is:** write the generic round-trip first, then the engine is short — `collect_proposals`, round-trip each candidate, apply the budget all-or-nothing, return the survivors as alternatives.

**Where the round-trip belongs is worth a moment's thought rather than a default.** It is a `tiler-ir` capability (it uses only that crate's builder and program), but adding it there is a public API change on the semantic authority, which ADR 0075 reserves. Writing it privately in `tiler-compiler` avoids that and duplicates a walk `tiler-ir` is better placed to keep correct as the operation vocabulary grows. Prefer the private version for this ticket, and record the duplication rather than pre-emptively promoting an interface — the second consumer is what should justify the promotion.

**Round-trip landed 2026-07-28.** `normalize::revalidate_structurally(program) -> Result<SemanticProgram, NormalizeError>` re-applies every operation through the frozen authority, so result types and shapes are **re-inferred rather than copied**, and a candidate whose structure does not survive inference is rejected rather than adopted.

*It is not `rebuild` with an identity congruence, deliberately.* An identity `Congruence` is a value only this call site would construct and which the congruence's own invariants do not describe; passing one would make the generic path's correctness depend on a CSE-shaped structure being filled the way CSE never fills it. The duplication is one loop, and it is the honest cost of two paths answering different questions — `rebuild` asks what the rewrite produces, this asks whether an arbitrary program still validates.

*Two tests, and the second exists because the first is not enough.* Round-tripping preserves canonical identity bytes on two fixtures; but a `revalidate_structurally` that returned its input would pass that. The second drives it against the program the **rewrite** produced — built by a different path than the fixture — and checks operation and value counts survive re-inference, and asserts the rewritten program is genuinely smaller than its input so the test cannot be silently running on the unrewritten fixture.

## The engine landed, and the pin holds against it (2026-07-28)

`normalize::run_rewrite_engine(registry, program, budgets) -> Result<Option<Vec<RewriteProposal<SemanticProgram>>>, EngineFailure>`.

**`Option` rather than an empty vector for an abandoned run.** `Ok(None)` says the run was abandoned; `Ok(Some(vec![]))` says nothing applied. A caller receiving an empty vector for a budget stop would record "no rewrite available" for a program that had one. Both cases are tested, and each would pass an engine that always returned the other — which is why both exist.

**The budget counts proposals, not accepted alternatives**, and counts them *before* revalidation. Otherwise a rule could buy extra budget by proposing candidates that fail. Same resource `normalize_semantics` bounds.

**A budget stop or a revalidation failure abandons the whole run.** Returning the alternatives collected so far is a partial result that reads like a complete one — a caller cannot tell a run that found two from one that found five and lost three.

**`EngineFailure` separates a provider defect from a revalidation failure**, and the revalidation case carries the rule. Without it a caller would know a rewrite was invalid and not which rule to exclude.

**The engine adopts the *revalidated* program, not the provider's.** Verifying a rebuild while keeping the original would retain whatever the provider actually constructed; the rebuild is the version the frozen authority produced, and that is the one that should survive.

**The pin holds against the engine.** With only the common-subexpression rule registered, `run_rewrite_engine`'s single alternative has the same canonical `SemanticIdentity` bytes as `normalize_semantics`'s normalized program. The budget test's failure path was verified by returning `Ok(Some(vec![]))` on exhaustion and watching it fail.

## Closing criteria, checked one by one (2026-07-28)

- *Consumes `collect_proposals` and revalidates each candidate through `SemanticProgramBuilder` before adoption* — **met**, via `revalidate_structurally`, and the engine adopts the revalidated program rather than the provider's.
- *Yields a set of alternatives; the single-canonical contract left in place rather than silently widened* — **met**. `normalize_semantics` is untouched and still produces one program; the engine is a separate entry point.
- *Termination, budget exhaustion, and rollback keep the all-or-nothing contract* — **met**. A budget stop returns `Ok(None)`, distinct from `Ok(Some(vec![]))`, and both are tested because each would pass an engine that always returned the other.
- *The alternative set is reproducible across runs* — **met**, through the registry's canonical identity ordering, which its own test pins against two registration orders.
- *A `ProviderDefect` is reported as a typed, explainable failure distinct from an ordinary rejection, asserted by a test that watches the rejection fire* — **met, and it was the last one outstanding.** Added `a_provider_defect_abandons_the_engine_run` and `a_misattributing_provider_fails_the_engine_run`; the second confirms the engine inherits `collect_proposals`' attribution contract rather than assuming the `?` carries it.
- *With only the common-subexpression rule registered, the result has the same `SemanticIdentity` as today's `normalize.rs` output* — **met at the time, compared on canonical bytes — and the audit later found the pin degenerated**: when `route-the-compile-path…` made the stage *be* the engine, `the_engine_with_only_cse_reproduces_this_stage` became a self-comparison. The property is still pinned by `the_provider_proposes_exactly_what_this_stage_produces` plus the two revalidation tests (adversarially verified), and the tautological test was removed. Recorded here because "The pin holds against the engine" below reads as a live guarantee and is now historical.

## Split: routing is a behaviour change

The engine satisfies every criterion above and **nothing calls it**. Routing the compile path through it is `route-the-compile-path-through-the-rewrite-engine`, now live.

That split is not a technicality. Everything here was additive or self-contained — a provider over existing functions, a round-trip, an engine — and none of it changed what the compiler does. Routing moves the pipeline from one canonical program to a set of alternatives, which touches plan enumeration, the explain census, and potentially artifact identity. The child records all three as expectations rather than leaving them to be found as mysterious failures, and states the trap: choosing one alternative before planning is cheaper and likely wrong, because it makes the rewrite decision without the cost model that would justify it.

## Superseding outcome (2026-07-28)

The paragraph above records the split-time state. The engine now serves two distinct transactions on the live compile path: deterministic CSE remains canonical normalization, while the add and multiply ordered-reassociation rules produce a baseline-preserving algebraic portfolio afterward. `implement-first-algebraic-rewrite-portfolio` consumed the previously idle alternative seams by independently readmitting every candidate, grouping resolved contracts before any cost comparison, emitting survivor-only rule payloads, planning every evaluated semantic candidate, and selecting globally only after verified physical alternatives exist. The earlier warning against choosing an algebraic alternative before planning is therefore discharged rather than relaxed.
