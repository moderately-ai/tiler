---
id: name-the-execution-ordinal-in-model-level-failures
title: Name the execution ordinal in model-level failures
status: todo
priority: p2
dependencies: [drive-the-complete-forward-pass-over-three-artifacts, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, scope-tiler-numerical-claims-across-the-candle-kernel-boundary, retain-the-c1-model-attribution-fixture]
scopes: [contracts/integrations, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, runtime, explain, language-model, class-conformance-fixture]
---
## User-visible outcome

A failed model-level pass reports enough that a consumer changes something rather than re-running. For the four execution failure classes (bind, route, pre-commit adapter, post-commit), the consumer driver pairs the ordinal, phase, and token-in-flight it already holds with Tiler's typed stage reason. A conformance disagreement reports the position, which observable disagreed, and the tie state at that position, without inventing an ordinal — the attribution fixture owns "where" at the model boundary. The Required content table and Closes when below are the normative authority for which fields each class names.

## The ordinal is the driver's, and no Tiler type acquires one

**Classified as consumer conformance work on 2026-08-04 under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md).** This ticket reads as a generic runtime-diagnostics capability and is not one, and the distinction decides where its code lands.

**Fact.** `crates/tiler-runtime/src/adapter.rs`, anchor `pub fn route_with_adapter`, takes one program and one adapter and returns `Result<A::Completion, AdapterRouteFailure<A::Refusal, A::Failure>>` synchronously to its caller. Every stage that can refuse or fail returns through that one value; nothing is reported out of band and nothing is retained across calls.

**Inference.** The consumer therefore already knows which of its thirty invocations failed, because it is the one whose call returned the error — the ordinal is determined by the call site rather than carried by a value. Adding an execution ordinal, a phase, or a token-in-flight to a `tiler-runtime` type would put a caller's loop position into a consumer-agnostic runtime's public surface, which is the same class of workload vocabulary [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md) removed when it withdrew the cursor and the generation. What Tiler owes generically is what `AdapterRouteFailure` already does: carry each stage's own typed reason whole rather than flattening it, and say on which side of the routing commit the failure fell.

**So the work splits.** The five classes below are a *consumer driver's* obligation — the driver pairs each returned refusal with the ordinal, phase, and token it already holds and reports the pair. Any residue that is genuinely Tiler's is a named gap in a stage's own typed reason (for example, whether a bind refusal names the interface key and the axis), and each such residue is stated as its own generic requirement in this ticket's delivery rather than satisfied by attaching an ordinal.

**Scope correction — 2026-08-09.** Because the delivered ordinal/report composition belongs to the consumer driver, this ticket now claims `implementation/candle`, not `implementation/runtime`. If implementation discovers that a generic stage reason itself lacks required structured data, file that runtime boundary as a separate ticket rather than smuggling a driver ordinal into the generic adapter surface.

## Required content

The five classes [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) enumerates, each naming what a consumer can act on. Read the "Names" column as *what the consumer's report states*, with the ordinal, phase, and token supplied by the driver and the typed reason supplied by Tiler.

| Failure | Names |
| --- | --- |
| Bind refusal | The interface key, the axis, the declared value, the supplied value, and the execution ordinal — one weight-set mistake fires at exactly one of the thirty |
| Route refusal | The loader's own typed reason, the ordinal, and the phase (prefill, or decode step *n*) |
| Pre-commit adapter refusal | The adapter's own reason, the ordinal, and that the routing commit was not crossed |
| Post-commit failure | The ordinal, the token in flight, that no fallback was taken and none was available, and that the failed execution's outputs are not observable. *Corrected 2026-08-04 under [`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md): this read "and that the model state is poisoned rather than stale". Tiler holds no model state to poison; the report says which invocation failed and withholds its outputs, and the consumer's decision not to continue from its pre-failure tensors is the driver's under [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md).* |
| Conformance failure | The position, which observable disagreed, the tie state at that position — and deliberately **no** ordinal, because the model boundary has none, which is why the attribution fixture exists |

**Every reported numerical realization carries the operations it covers.** [Candle integration](../docs/integration/candle.md)'s Diagnostics section already makes that pairing obligatory, and a model-level claim covers 30 executions and four host computations, so the scope is part of the statement rather than formatting.

## Closes when

Each of the five classes is produced by a case deliberately made to fail, the ordinal is correct in the four that carry one, and the conformance class is watched *not* inventing one.
