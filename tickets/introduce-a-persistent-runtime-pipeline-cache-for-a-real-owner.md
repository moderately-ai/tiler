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
