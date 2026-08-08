---
id: name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set
title: Name the fold-prologue chain boundary instead of reporting operation-set
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

`sum(matmul(a,b) * 2.0)` refuses under a rule naming what was actually declined — a fold's prologue may not itself be a chain — instead of `operation-set`, which claims the vocabulary cannot spell an operation it spells fine standalone.

## Why this exists (audited 2026-08-06, coordinator-verified: the From impl at request.rs:4259 flattens Folded to operation-set; only recognize_reduction's contributor walk consumes the flattening wrapper)

`recognize_elementwise_output` builds an epilogue chain from `ElementwiseRefusal::Folded`; `recognize_elementwise` — sole caller `recognize_reduction` — discards the finding. The refusal is correct (`NormalizedSerialSum` carries no producer); the rule name and the "the same general walk" doc at request.rs:5276 are the defects.

## The work (no admission change)

Distinct rule `reduction-prologue-chain`; the doc stops claiming one walk (shared planner, different findings); `select_supported_strategy`'s boundary paragraph gains the row. Perturbations: the chain-prologue refuses under the new rule, the standalone chain still compiles, and a genuinely unspellable prologue (e.g. `sum(rms_norm(x))`) still reports `operation-set` — else the split renamed rather than separated. File the admission itself as a `deferred` follow-on with the worked-example trigger.

## Closes when

The rule is separated with all three perturbations observed and the deferral filed.
