---
id: admit-parametric-symbolic-broadcast-at-the-compiler-request-boundary
title: Admit parametric symbolic broadcast at the compiler request boundary
status: todo
priority: p1
dependencies: [carry-the-parametric-broadcast-relation-through-index-and-schedule-ir, admit-symbolic-extents-at-the-compiler-request-boundary]
related: []
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler, shapes, broadcast]
---
# Admit parametric symbolic broadcast at the compiler request boundary

## User-visible outcome

A semantic program containing the accepted parametric broadcast reaches physical selection without its symbolic extent being folded, split into row-specific graphs, or refused under an unrelated static-shape rule.

## Work

- Bind the compiler request to the semantic program's exact shape environment; accept no second caller-supplied environment.
- Recognize and retain the parametric broadcast access relation through normalization, region construction, physical subject binding, selection, explanation, and candidate verification.
- Specialize nothing at request admission. A provider that cannot implement the symbolic carrier declines under its own typed rule.
- Ensure any later guarded specialization is an explicitly identified physical alternative rather than a mutation of graph meaning or the baseline route.
- Update governed physical-provider revision/provenance only if the previously admitted context-to-offer function changes; document the comparison.

## Acceptance

- One symbolic program reaches selection with its environment and mapping unchanged.
- Perturbing a bound value does not change semantic, normalized-program, or request identity.
- A provider lacking parametric support declines by the named capability rule; no static-signature or generic unsupported error masks it.
- No selected plan contains a silently substituted concrete reindex/broadcast access unless it is a separately guarded and identified physical alternative.

## Stop conditions

Stop if compiler admission needs the bound extent value, if a second environment can disagree with the program, or if a provider can silently reinterpret the relation through an existing concrete access variant.
