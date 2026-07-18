---
id: cache-crash-race-harness
title: Spike the content-addressed cache crash and race protocol
status: todo
priority: p1
dependencies: [artifact-envelope-model]
related: []
scopes: [research/cache]
shared_scopes: []
paths: []
tags: [tiler-research, spike, cache, correctness]
---
Specify and exercise the one-file immutable bundle protocol: complete key, per-key advisory lock, recheck, same-filesystem temporary publication, validation, atomic rename, corruption recovery, durability policy, and garbage collection coordination.

Build a process-level harness for concurrent identical and distinct keys, killed writers at publication phases, truncation, unwritable roots, deletion, and eviction racing readers. Record guarantees separately from observed filesystem behavior and leave production cache implementation out of scope.
