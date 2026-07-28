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
