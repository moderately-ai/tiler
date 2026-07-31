---
id: repair-the-broken-proof-budget-anchor
title: Repair the broken proof-budget anchor in the fusion contract
status: todo
priority: p3
dependencies: []
related: []
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

The `proof budget` cross-reference in the fusion-and-scheduling contract resolves to a real heading instead of silently landing at the top of the optimizer contract.

## Why this slice exists

`docs/compiler/fusion-and-scheduling.md:35` links to `optimizer.md#refinement-is-exhaustive-finite-evidence-with-an-explicit-gap`. No such heading exists. The nearest candidates are `## Lowering capability resolution and index-region refinement` (line 160) and `### Refinement requires discharged index-domain evidence` (line 179). Reproduce with `grep -n '^#' docs/compiler/optimizer.md | grep -i refinement`.

Found while defining backend/device vocabulary under `define-backend-device-and-execution-context-vocabulary`, by a link resolver run over the whole corpus: it was the only broken local link among 2,495 checked, so this is a point defect rather than a class of drift.

## Implementation keys

- Decide whether the link should point at an existing heading or whether the optimizer contract is missing the section the citing sentence promises. That sentence attributes to the target the claim that a proof budget "leaves its subject's predicate open while the plan containing it stands", and the target must actually state it.
- Do not repoint the link at a heading that does not make that claim merely to resolve the anchor.

## Closes when

The link resolves and its target states what the citing sentence attributes to it.
