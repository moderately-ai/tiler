---
id: close-the-eps-exp-open-axis-in-the-rule-object-record
title: Close the eps_exp open axis in the rule-object record
status: in-progress
priority: p3
dependencies: [expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
related: [derive-the-rewrite-rule-declaration-and-admission-shape-for-the-online-softmax-fold, connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, accuracy, docs]
claimed_from: todo
assignee: agent-eps-axis
lease_expires_at: 1786039981
---
## User-visible outcome

A reader of [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md) and [the certified-bounds record](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) learns that the numeric `eps_exp` gap is closed and what the number is, instead of reading two open axes whose owning ticket has landed.

## Why this exists

**Fact.** [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md) landed `elementary_relative_accuracy` in `crates/tiler-compiler/src/target/accuracy.rs`. Both records name the gap it closed, and neither could be edited by that ticket: its scopes were `implementation/compiler` and shared `project/tickets`, and `research/numerics` was outside them.

**Fact — three sentences are now false and one is a status claim rather than a derivation.** The rule-object record's open axes state "The numeric `eps_exp` gap is narrower than filed … → a dated note on [that ticket], which stays `todo`"; its Part 4 obligation 2 is headed "**refuses**, and the gap is narrower than it was filed as"; its Part 6 states "**Fact — prerequisite (2) is not satisfied.** `grep -m1 '^status:' tickets/expose-…` returns `status: todo` at this base." The certified-bounds record's open axes list "The target accuracy authority cannot yield the numeric `eps_exp` a bound needs" with no resolution note, where its sibling axes carry dated ones.

**Inference — the correction is not merely a status flip, and that is why it is a ticket rather than a sweep.** Part 6's readiness statement is a derivation over three prerequisites, and the third element of it moving changes the conclusion: the condition's stated prerequisites are now all satisfied. The record's own reconsideration-trigger section says "**The condition has not fired**", which stops being true. Restating it needs the same care the original had — in particular, that readiness existing and the rewrite becoming reachable are different claims, because obligation 1 still refuses on `SOFTMAX_F32_FACT_SUBNORMALS` independently of both permissions and obligation 3 still wants a merge topology no schedule type carries.

## What this ticket must produce

- Part 4 obligation 2 restated at its new verdict, with the number the query returns for each registered family read from source rather than from this ticket: `24u` for both exponentials, `2u` for the faithful reciprocal square root, `u` only for a contract that states correct rounding.
- The metric-conversion boundary recorded as what it became — a field of the returned answer naming the reference magnitude the bound holds at or above — and its dependence on obligation 1 restated as the *same* obligation rather than a second one.
- Part 6's readiness statement re-derived, and the reconsideration-trigger sentence corrected, keeping the two claims separate.
- Dated resolution notes on both records' open axes, in the shape their sibling axes already use.

## Non-goals

Deciding either permission; re-deriving any bound; editing the ADR, whose reopening condition is unchanged by this — what changes is whether its prerequisites are met, which is a fact the records report rather than a decision they make.
