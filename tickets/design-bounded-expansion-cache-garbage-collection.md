---
id: design-bounded-expansion-cache-garbage-collection
title: Design bounded expansion cache garbage collection
status: in-progress
priority: p2
dependencies: []
related: [implement-the-expansion-cache-protocol]
scopes: [research/cache, implementation/cache]
shared_scopes: []
paths: []
tags: [cache, durability, concurrency]
claimed_from: todo
assignee: agent-design-bounded-expansion-cache-garbage-collection
lease_expires_at: 1785038575
---
The research note's sixth follow-up gate: design bounded garbage collection and accounting separately, and stress eviction with active writers and readers at 1, 8, and 32 processes.

`tiler-cache` implements the two pieces that are *not* policy: `evict` takes the per-key lock before removing an entry and retains the lock file, and `sweep_temporaries` removes abandoned temporaries for one key under the same lock and outside a grace period. It deliberately implements **no whole-cache policy at all**, and `Limits` carries no maximum entry count — bounding the entry count means choosing which entry to evict, which is exactly the policy this gate holds open. A field recording a bound nothing enforced would read as a guarantee.

## What this ticket owes

- Decide the accounting: total bytes, entry count, recency, and the best-effort work budget per invocation, kept as disposable metadata that never mutates bundle bytes and is never trusted for hit correctness.
- Decide who runs it and when. An expansion-time collection competes with the compilation it is meant to accelerate.
- Stress it against active writers and readers at 1, 8, and 32 processes, which the existing harness structure already supports.
- A whole-cache purge must either require quiescence or rename the version root out of service; the harness already shows that an arbitrary external recursive deletion loses compile-once suppression while preserving correctness, and a Tiler-provided purge must not promise what `rm -r` cannot.
