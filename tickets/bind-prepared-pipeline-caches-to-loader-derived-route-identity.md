---
id: bind-prepared-pipeline-caches-to-loader-derived-route-identity
title: Bind prepared-pipeline caches to loader-derived route identity
status: todo
priority: p1
dependencies: []
related: [decide-the-prepared-subgroup-width-equality-gate, carry-subgroup-width-through-exact-prepared-entry-equality]
scopes: [implementation/runtime, implementation/candle, contracts/artifacts, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, cache, identity, public-boundary, correctness]
---
## User-visible outcome

A prepared pipeline and every property observed from it are structurally tied to the decoded route that supplied its payload, rather than to identity bytes a wrapper could accidentally or dishonestly reuse.

## Fact — 2026-08-11

The normal Candle wrapper passes the decoded artifact identity correctly, and its cache key includes device scope, artifact identity, execution-order entry, and symbol. The public adapter constructor nevertheless accepts caller-vouched identity bytes; the runtime trait does not structurally supply loader-derived identity to payload validation or preparation. Reusing the wrong bytes can select another artifact's cached library/pipeline before the current payload is loaded.

## Required delivery

- Supply loader-authenticated route/artifact identity structurally to payload validation and preparation, or mint an equivalent non-forgeable per-route preparation token.
- Key prepared state by the exact device, artifact, entry, payload, and symbol subjects the cache claims. Do not key a property observation by entry ordinal alone outside that route.
- Cache prepared objects and observations, never satisfaction verdicts or routing authority. Every attempt performs its own comparison against its own requirement.
- Perturb caller identity, payload bytes, entry order, symbol, device scope, and cache hit/miss independently. Quote the refusal or miss proving cross-route reuse is impossible.

## Closes when

No public construction path can make a prepared-entry observation originate from a pipeline other than the exact loader-selected entry.
