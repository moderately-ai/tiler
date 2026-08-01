---
id: correct-the-frontends-doc-named-call-claim
title: Correct the frontends doc's claim that the named-call form is unfilled
status: done
priority: p3
dependencies: []
related: [denote-a-reduction-region-in-the-inline-macro-grammar]
scopes: [contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, frontend]
---
The sentence is now false in one clause.

**Fact.** `docs/integration/frontends.md:396` states that the `tensor!` grammar "admits neither a `let` binding, nor a named operation call such as `einops(…)` or `gelu(…)`, nor any operator beyond `*` and `+`", with "the named-call form reserved rather than filled, because the governed semantic profile registers no operation without an operator spelling."

**Fact.** `denote-a-reduction-region-in-the-inline-macro-grammar` filled one named call: `strict_serial_sum(<expression>, [<axis>, …])`, resolving to `tiler::strict-serial-sum-f32@1`. It also admitted a scalar real literal in the body and an optional axis name in an operand's declared shape (`f32[cols: 8]`). `einops(…)` and `gelu(…)` are still refused at the name, and no operator beyond `*` and `+` is admitted.

**Why this is a separate ticket.** `docs/integration/**` maps to `contracts/integrations`, which the grammar ticket did not hold. The correction is one paragraph and must not be swept into a frontend-scoped branch.

## User-visible outcome

A reader of `docs/integration/frontends.md` learns which named call is registered and which are not, rather than that none is.

## Closes when

The paragraph names `strict_serial_sum` as the one filled call, keeps the `einops`/`gelu` refusals as they are, and mentions the scalar literal and the axis-name form; a sweep confirms no other sentence in `docs/integration/` asserts the grammar admits only `*` and `+`.
