---
id: derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause
title: Derive the value precondition the online-softmax bound needs for its subnormal clause
status: deferred
priority: p3
dependencies: []
related: [derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, connect-certified-rounding-error-bounds-to-rewrite-permissions, reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, decide-whether-to-admit-an-elementary-identity-permission]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, optimizer]
---
## User-visible outcome

The exact value-domain predicate that discharges the online-softmax rescaling bound's "no subnormal intermediate" side condition, with its ADR 0021 provenance route derived rather than assumed — so a reader knows what an admitting decision would and would not deliver.

## Why this exists

**Fact.** [The rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s Part 4 obligation 1 finds that the bound's subnormal clause **cannot be discharged** for `tiler::softmax-f32@1` on an ordinary attention row, and that the refusal is independent of both permissions. The evidence is the operation's own registered fact: `SOFTMAX_F32_FACT_SUBNORMALS` (`crates/tiler-ir/src/semantic/softmax.rs:337`) records that "a contributor about 87 below the row maximum has an exponential of `0x00b33687`, a subnormal".

**Fact.** [The certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md)'s Part 2 states the clause as an assumption "the admission rule must discharge rather than inherit", because (2.5a)'s relative-error model does not hold in the subnormal band; [the tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md)'s Part 2 restates it as a refusal its probe performs, and its Step 5 notes that a tree roughly doubles the number of exposed sites while leaving the bound unchanged.

**Inference — the precondition is not simply a bound on the logit spread, and that is why this is a derivation rather than a constant.** The rule-object record derives that the contributor terms are safe when the spread is below `-126 ln 2 = 87.3365…`, and that the partial sums are never exposed because the running maximum's own term is `exp(0) = 1` so `d >= 1` throughout. It also identifies a second exposed family it does not close: the rescaled product `d_{j-1} · exp(m_{j-1} - m_j)`, whose factor can be arbitrarily small when the maximum jumps.

## What this ticket must produce

- The complete predicate over the operation's operands that discharges the clause at every exposed site, for the sequential fold and for a merge tree, with the tree's roughly doubled site count carried rather than assumed away.
- Its [ADR 0021](../docs/decisions/0021-validated-value-assumptions.md) provenance route, derived: caller-declared is ineligible; compiler-proven is unavailable while the logits are unbounded external inputs; so what remains is runtime validation before routing commit, and this ticket states its cost and its refusal rather than asserting it is affordable.
- What the price collapses to when the precondition holds, since the certified-bounds measurement shows an observed price of exactly zero once the running maximum stops moving — the sharpening is a separate claim from the discharge and must not be conflated with it.
- Any public boundary reached stops at a draft with an acceptance node; nothing is self-accepted.

## Non-goals

Admitting either permission; re-deriving `P(D, h2)`; implementing a validation mechanism; proposing an ADR.

## Trigger

A precondition for a rewrite no contract can perform discharges nothing, so this waits on the permissions the fold consumes. It fires when [`reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller`](reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller.md) **and** [`decide-whether-to-admit-an-elementary-identity-permission`](decide-whether-to-admit-an-elementary-identity-permission.md) both resolve in the admitting direction, **or** a second rule appears whose bound carries the same subnormal clause, at which point the derivation has more than one caller.

**One thing this deferral deliberately does not gate.** The *finding* that the clause refuses is already recorded in the rule-object record and is available to the joint decision now. What waits is the full derivation of the discharging predicate, not the fact that one is needed.

## Trigger check log

- 2026-08-06 — not fired. ADR 0095 was reaffirmed on 2026-08-06 and admits no distributivity permission; ADR 0101 decision 5 reserves the elementary-identity permission and admits none. No second rule carries the clause. Reproduce with `grep -n 'decision_status' docs/decisions/0095-decline-a-distributivity-permission.md docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md`.
