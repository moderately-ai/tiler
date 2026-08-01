---
id: name-the-execution-ordinal-in-model-level-failures
title: Name the execution ordinal in model-level failures
status: todo
priority: p2
dependencies: [drive-the-complete-forward-pass-over-three-artifacts]
related: [design-model-ingestion-and-complete-execution, scope-tiler-numerical-claims-across-the-candle-kernel-boundary, retain-the-c1-model-attribution-fixture]
scopes: [implementation/runtime, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, runtime, explain, language-model]
---
## User-visible outcome

A failed forward pass says which of its thirty executions failed, in which phase, with which token in flight — so a consumer changes something rather than re-running.

## Required content

The five classes [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) enumerates, each naming what a consumer can act on.

| Failure | Names |
| --- | --- |
| Bind refusal | The interface key, the axis, the declared value, the supplied value, and the execution ordinal — one weight-set mistake fires at exactly one of the thirty |
| Route refusal | The loader's own typed reason, the ordinal, and the phase (prefill, or decode step *n*) |
| Pre-commit adapter refusal | The adapter's own reason, the ordinal, and that the routing commit was not crossed |
| Post-commit failure | The ordinal, the token in flight, that no fallback was taken and none was available, and that the model state is poisoned rather than stale |
| Conformance failure | The position, which observable disagreed, the tie state at that position — and deliberately **no** ordinal, because the model boundary has none, which is why the attribution fixture exists |

**Every reported numerical realization carries the operations it covers.** [Candle integration](../docs/integration/candle.md)'s Diagnostics section already makes that pairing obligatory, and a model-level claim covers 30 executions and four host computations, so the scope is part of the statement rather than formatting.

## Closes when

Each of the five classes is produced by a case deliberately made to fail, the ordinal is correct in the four that carry one, and the conformance class is watched *not* inventing one.
