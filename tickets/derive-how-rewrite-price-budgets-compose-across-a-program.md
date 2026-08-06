---
id: derive-how-rewrite-price-budgets-compose-across-a-program
title: Derive how rewrite price budgets compose across a program
status: deferred
priority: p3
dependencies: []
related: [reconcile-which-quantity-the-admission-rule-compares-against-a-caller-s-tolerance, accept-the-rewrite-price-tolerance-vocabulary, connect-certified-rounding-error-bounds-to-rewrite-permissions]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy]
---
## User-visible outcome

A caller who applies more than one priced rewrite to a program knows what their stated number bounds, instead of holding a per-rewrite threshold that names no program-level quantity.

## Why this exists

**Fact.** [The certified-bounds record's](../docs/research/numerics/certified-bounds-as-rewrite-permissions.md) Part 3 derives that the admission rule compares a rewrite's derived price `P` against a caller-stated price budget, where `P` is defined by `1 + B_1 = (1 + B_2)(1 + P)` over the rewrite's and the matched baseline's derived bounds. Its worked example has exactly one rewrite.

**Inference.** Relative error factors compose multiplicatively along a dependency chain, so `n` rewrites each admitted at a stated `tol` bound the program's degradation by `(1 + tol)^n - 1` and not by `tol`. A per-rewrite threshold is therefore not a program-level guarantee, and the two available vocabularies — "a threshold each rewrite passes independently" and "a budget the program spends" — give different verdicts on the same program at the same stated number. Part 4 item 6 of that record carries the spelling question to Tom; this ticket owns the derivation the spelling needs.

**Inference — the naive multiplicative law is an upper bound and is probably not the answer.** Two rewrites on independent subgraphs whose results never meet do not compound at all; two on the same reduction chain do. What decides it is the dependency structure between the priced sites and whether the shared real reference each price is stated against is the same one, which is a question about the program rather than about either rule. A composition law that ignored that would refuse correct programs at a rate that grows with the rewrite count, which is the direction that makes a vocabulary unusable rather than unsound.

## What this must produce

The derivation of how prices compose across a program: the law, the structural condition under which it is tight, the condition under which prices do not compound at all, and what the admission rule must be handed to evaluate it — since a per-candidate rule sees one candidate and a program-level budget is not a per-candidate quantity. If the answer is that a budget cannot be evaluated per candidate, that is a finding about where the check sits and belongs in the outcome rather than as a caveat.

## Non-goals

Deriving any new bound; changing `P` at either fold shape; spelling the caller-facing vocabulary, which is [`accept-the-rewrite-price-tolerance-vocabulary`](accept-the-rewrite-price-tolerance-vocabulary.md)'s and Tom's.

## Closes when

The composition law is derived with its tightness condition, the per-candidate evaluability question is answered, and the certified-bounds record's open axis naming this ticket carries the result.

## Trigger check log

**Deferred behind the first program carrying two priced rewrites.** Today the count is zero: [the flash-class record's](../docs/research/program-planning/flash-class-capability-set.md) five-rule table is a Proposal, [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md) derives R2 without registering it, and the two numerical dimensions any such rule consumes are declined and reserved. A composition law for a vocabulary no contract states, over rewrites no permission admits, would be derived against an assumed program shape.

- **2026-08-06 — not fired.** `grep -rniE "rewrite_price|derived_price|parametric bound" crates/tiler-compiler/src/` returns nothing, so no registered rule carries a price of any kind; `grep -m1 '^decision_status:' docs/decisions/0095-decline-a-distributivity-permission.md` returns `"accepted"` on a decline, so the worked rewrite remains illegal. No program in the tree carries one priced rewrite, so none carries two.
