---
id: prototype-expansion-content-cache
title: Implement the expansion content cache
status: todo
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver]
related: []
scopes: [implementation/cache, implementation/metal-aot]
shared_scopes: []
paths: []
tags: [implementation, cache, proc-macro, inline-dx]
---
Implement complete content identity, one immutable bounded bundle per key, validation on every hit, stable per-key advisory locking, post-lock recheck, unique same-filesystem temporary publication, atomic rename, corruption recovery, limits/diagnostics, and race/crash/unwritable tests. Generated code never depends on the cache.
