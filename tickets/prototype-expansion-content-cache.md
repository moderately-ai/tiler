---
id: prototype-expansion-content-cache
title: Implement the expansion content cache
status: todo
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver, repair-cache-experiment-harness-integrity]
related: []
scopes: [implementation/cache, implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, cache, proc-macro, inline-dx]
---
Implement complete content identity, one immutable bounded bundle per key, validation on every hit, stable per-key advisory locking, post-lock recheck, unique same-filesystem temporary publication, atomic rename, corruption recovery, limits/diagnostics, and race/crash/unwritable tests. Generated code never depends on the cache.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.
