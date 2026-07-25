---
id: prototype-expansion-content-cache
title: Implement the expansion content cache
status: in-progress
priority: p1
dependencies: [prototype-neutral-artifact-codec, prototype-apple-aot-driver, repair-cache-experiment-harness-integrity]
related: []
scopes: [implementation/cache, implementation/metal-aot, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, cache, proc-macro, inline-dx]
claimed_from: todo
assignee: agent-cache
lease_expires_at: 1785002816
---
Implement complete content identity, one immutable bounded bundle per key, validation on every hit, stable per-key advisory locking, post-lock recheck, unique same-filesystem temporary publication, atomic rename, corruption recovery, limits/diagnostics, and race/crash/unwritable tests. Generated code never depends on the cache.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

Complete content identity includes the strengthening `prototype-apple-aot-driver` explicitly deferred here: `tiler_metal_aot::ArtifactProvenance` currently records the portable compiler fingerprint as the `metal`/`metallib` **version strings** plus the resolved local tool paths, and documents that two component builds can report the same front-end version. A content digest of the tool binaries and SDK (and of the produced `metallib`) is what makes artifact identity sound **across hosts**; the driver crate deliberately stayed dependency-free and left it to this cache. Decide and record whether cross-host identity requires that digest, and if so add it here rather than leaving version strings as the only cross-host discriminator.
