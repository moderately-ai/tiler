---
id: expand-a-delivering-region-with-the-cache-disabled
title: Expand a delivering region when the expansion cache is disabled
status: in-progress
priority: p2
dependencies: [prototype-inline-aot-integration-proof]
related: []
scopes: [implementation/cache, implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, cache, inline-dx]
claimed_from: todo
assignee: worker-expand-a-del
lease_expires_at: 1785566409
---
## Why this exists

ADR 0089 accepted `TILER_EXPANSION_CACHE_DIR=off` as meaning "expand with no cache at all", and `CacheRootDecision::Disabled`'s own documentation spells it "expand, compile, embed, and cache nothing". `prototype-inline-aot-integration-proof` narrowed that: a region whose `deliver` statement selects an artifact family refuses under `off`, with `AotRefusal::CacheDisabled` naming the remedy. A region stating `fallback-only`, or stating nothing, is unaffected — it opens no cache at all.

**Fact.** `tiler_cache::expansion::ExpansionCache` has no store-nothing mode. Its constructor is `open(root)`, and every unusable-root path reaches `Resolution::Uncached` through an *attempted* publication rather than through a stated one. `tiler_build::accept_or_publish_metal_plan` requires an `&ExpansionCache`, so there is no cache-free route through the sequencing it owns.

**Inference.** The honest options are a `disabled` constructor on `ExpansionCache` whose every resolution is `Uncached`, or a cache-free path through `tiler-build`. The first is smaller and matches an outcome variant that already exists; both are public-boundary additions.

## Closes when

`off` compiles, embeds, and stores nothing for a region stating a selected family, and a test proves it publishes no file — with the deliberate-failure check that the same expansion *does* publish when a root is stated. `AotRefusal::CacheDisabled` and its diagnostic are removed in the same change, and the narrowing note in the proof's boundary packet is retired.
