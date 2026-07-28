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
