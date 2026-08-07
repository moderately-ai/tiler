---
id: correct-the-recognizer-era-sentences-in-the-optimizer-contract
title: Correct the recognizer era sentences in the optimizer contract
status: in-progress
priority: p3
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [docs, doc-drift]
claimed_from: todo
assignee: agent-optimizer-doc
lease_expires_at: 1786132480
---
## What is stale

`widen-the-strategy-recognizer-past-the-f32-wall` removed the `dtype-f32` gate on 2026-08-07: recognition now derives the program's arithmetic type from its values, a non-`f32` program reaches a selected `PlanAlternative`, and the refusal moved to the contract and the profile as their own typed causes.

Two sentences in `docs/compiler/optimizer.md` were falsified by it:

- **:197** — "two program-wide properties — at least one declared input, `f32` throughout".
- **:199** — "the program-wide `dtype-f32` check refuses it first".

Verify both at your own base before editing; line numbers move.

## Why it is a separate ticket

Found by the worker on [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md), **in its own exclusive scope**, which could have edited them. It deliberately did not, on the ground that they are another ticket's debt and silently absorbing them would hide that the recognizer landing left a documentation obligation. That is the right call, and this ticket is the obligation made visible rather than a chore.

## What to state instead

Not merely deleting the clauses — say what the program-wide properties **now are**, and where the refusal now lives, so a reader learns the current shape rather than losing a sentence. The refusal did not disappear: a BF16 program under an `f32` contract is refused by the **contract**, program-scoped and before any target, under `compile.request.numerics.inapplicable`; a dtype the profile cannot dispatch is refused by the **profile**. The recognizer keeps two rules of its own — a width this build spells no body for, and two widths in one program.

## Check the rest of the document

Read `docs/compiler/optimizer.md` in full rather than patching two lines. The recognizer widening was a structural change and this document describes the stage it changed; two sentences being reported does not mean two are wrong. Report anything else you find, whether or not you fix it.

## Closes when

Both sentences state current truth, the document carries no other recognizer-era claim, and the refusal's new authorities are named rather than the old check merely removed.
