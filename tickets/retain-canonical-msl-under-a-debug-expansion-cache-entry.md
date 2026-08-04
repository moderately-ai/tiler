---
id: retain-canonical-msl-under-a-debug-expansion-cache-entry
title: Retain canonical MSL and tool diagnostics under a debug expansion-cache entry
status: todo
priority: p3
dependencies: []
related: [retain-and-attribute-a-real-msl-failure-through-an-expansion]
scopes: [implementation/cache, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, cache, diagnostics]
---
## User-visible outcome

Under a debug configuration, the canonical MSL an expansion emitted and the diagnostics the offline tools produced are readable from the expansion-cache entry the compilation resolved to, without recompiling and without weakening what a validated hit proves.

## Why this exists

**Fact — split out of `retain-and-attribute-a-real-msl-failure-through-an-expansion` on 2026-08-04, not absorbed.** That ticket's own implementation keys say the permission "is separable and may split into its own ticket; it changes what an entry stores, which is cache-identity-adjacent and needs its own reasoning about whether retained diagnostics participate in validation". The split is also a scope fact: the retention ticket holds `implementation/frontend`, and every file this work touches is under `crates/tiler-cache/**` and `crates/tiler-build/**`.

**Fact — the permission exists and nothing delivers it.** `docs/integration/frontends.md` states "Debug configuration may retain canonical MSL and tool diagnostics under the cache entry". `crates/tiler-cache/src/expansion.rs` mentions neither MSL nor tool output; a bundle carries the composed subject and the artifact envelope and nothing else.

**Inference — the hard part is validation, not storage.** `ExpansionCache::lookup` has no fast path: every read decodes the whole frame, re-derives the key from the carried subject, and re-proves the envelope. A new section has to answer three questions before any bytes are written — whether retained text participates in the key (it must not: the same compilation must resolve to one entry whether or not a debug configuration asked for text), whether it participates in the frame's digest set (it must, or an entry could be edited in place and stay valid), and what a hit does when the section is absent because the entry was published by a build that did not retain (a hit, with nothing to show, rather than a miss).

**Inference — a failing compilation has no entry to attach to.** The diagnostics most worth retaining are the ones from a compilation that produced no artifact, and nothing is published for a miss that failed. So "tool diagnostics under the cache entry" covers the *succeeding* compilation's warnings, and a failed compilation's diagnostics reach a consumer only through the family-scoped `compile_error!` the frontend already emits. Saying which of the two this ticket delivers is part of its first deliverable.

## Implementation keys

- Decide participation before storage: key, digest set, and absent-section behaviour, each with the reason, and none of them decided by what is convenient to encode.
- A debug configuration is an input a caller states, never a host or environment sniff — `tiler_cache` must not read the process environment to decide what an entry carries.
- Retained text is bounded the way `ToolOutput` already is, and truncation is recorded rather than hidden.
- Say explicitly whether a failed compilation is in scope; if it is not, record why and leave the frontend's retained `compile_error!` as the only route.

## Closes when

An expansion cache entry carries the canonical MSL and the retained tool diagnostics under a stated debug configuration; a hit validates the same properties it validates today with the section present, absent, and damaged, each exercised; the identity decision is recorded where cache identity is specified; and `docs/integration/frontends.md`'s remaining-checks list names what is delivered.
