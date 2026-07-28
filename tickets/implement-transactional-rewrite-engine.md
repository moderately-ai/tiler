---
id: implement-transactional-rewrite-engine
title: Implement the external transactional rewrite engine
status: todo
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
