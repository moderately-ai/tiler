---
id: introduce-a-persistent-runtime-pipeline-cache-for-a-real-owner
title: Introduce a persistent runtime pipeline cache for a real owner
status: deferred
priority: p2
dependencies: []
related: [bind-prepared-pipeline-caches-to-loader-derived-route-identity, bind-runtime-library-and-pipeline-caches-to-exact-payload-bytes]
scopes: [implementation/artifact, implementation/runtime, contracts/artifacts, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, cache, identity, performance, deferred]
---
## User-visible outcome

An accepted runtime owner that executes more than one route attempt can reuse validated libraries and prepared pipelines without permitting two different objects, symbols, pipeline configurations, devices, or contexts to alias.

## Trigger

The first named and accepted runtime cache owner whose cache object survives one complete route attempt. A per-invocation adapter, a key-only unit test, a cache with no demonstrated hit, or a proposed future runtime does not fire the trigger.

## Required design

- Retain the governed `BackendPayloadCode` section digest already computed and validated by the artifact decoder as an opaque typed received identity, and carry it through `DecodedArtifact` and `RoutedEntry`. Do not rehash object bytes under a consumer-owned domain and do not substitute canonical artifact identity or the payload compilation-subject digest.
- Key a library by exact live device/context scope plus exact code-section digest.
- Key a pipeline by that library subject plus the resolved symbol and every present specialization, canonical pipeline descriptor, translation/archive, and runtime-mode input.
- Key a prepared observation by the exact pipeline subject plus the exact property query. Cache objects and observations, never satisfaction verdicts, fallback authority, or routing commitment.
- Decide concurrency, publication, eviction, in-flight retention, device/context loss, and stable-negative eligibility in the real owner's vocabulary.
- Demonstrate a real second-attempt hit and independent misses for perturbed code digest, symbol, pipeline configuration, device, and context. A population that cannot hit does not satisfy this ticket.

## Identity boundary

The typed code digest exposes an identity already present in and validated from the envelope; it does not change artifact bytes or canonical artifact identity. Runtime cache keys remain runtime-local.

## Trigger check log

- 2026-08-12 — **not fired.** `prototypes/candle-metal-adapter` constructs a fresh adapter and fresh cache for every route, and the accepted initial scalar CPU runtime declares no library or prepared-pipeline cache. The non-reusable Candle cache is being physically deleted under `bind-prepared-pipeline-caches-to-loader-derived-route-identity`.
- 2026-08-13 — **not fired.** Deletion landed: the prototype retains only per-attempt libraries and prepared pipelines on `CandleMetalAdapter`; no reusable cache object outlives a route attempt. The accepted initial scalar CPU runtime still declares no library or prepared-pipeline cache.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. The trigger is a cache object that outlives one route attempt, so the narrowest mechanical necessary condition is that such an object is named anywhere: the command returns **nothing** across `crates/` and `prototypes/`. Empty is a real negative rather than a structural one — both directories exist and are traversed, and the same pattern reports matches the moment any such field or type is introduced. A non-empty result is the changed answer, and is the point at which the *accepted owner* and *survives a complete route attempt* halves must be read rather than grepped. Command: `rg -n 'pipeline_cache|PipelineCache|library_cache' crates/ prototypes/ --glob '*.rs'`. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
