---
id: generalize-the-normalize-transaction-to-alternatives
title: Generalize the normalize transaction to drive providers and yield alternatives
status: todo
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
