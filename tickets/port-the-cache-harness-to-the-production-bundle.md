---
id: port-the-cache-harness-to-the-production-bundle
title: Port the cache harness to the production bundle
status: todo
priority: p1
dependencies: []
related: [implement-the-expansion-cache-protocol, cache-crash-race-harness]
scopes: [research/cache, implementation/cache]
shared_scopes: []
paths: []
tags: [cache, concurrency, durability, testing]
---
`spikes/cache/cache_harness.rs` kills real processes at nine publication phases and is the only evidence Tiler has for the cross-process crash and race behaviour ADR 0050 decides. It exercises **its own miniature frame**, not the bundle `tiler-cache` publishes.

`tiler-cache`'s in-crate suite is threaded, and it says so rather than implying otherwise: a thread that returns unwinds, closes its own descriptors, and never leaves a half-written file with no owner, so it is not evidence for a killed-process property and is not offered as one. **The production bundle's crash and race behaviour is currently unmeasured.**

## What this ticket owes

- Re-point the harness at `tiler_cache::expansion`, so the nine kill points exercise the real namespace, the real lock adapter, the real `create_new` temporary, the real separate-descriptor validation, and the real rename.
- Keep every case the spike already covers: concurrent identical keys producing one compilation, concurrent distinct keys, recovery at each kill point, truncated and digest-corrupt finals, entry and whole-cache deletion, active recursive deletion, an unusable root, and a reader holding an open descriptor across eviction.
- Record the exact host, toolchain, and repetition counts as a measurement, in the form `spikes/cache/results/` already uses. It is an observation about a host, not a portable guarantee.
- The harness needs a real artifact envelope to publish, which needs a semantic program; decide whether it builds one or whether this waits on a fixture the orchestrator can supply.

This is the process half of the research note's second follow-up gate. The envelope-integration half landed with `implement-the-expansion-cache-protocol`: the bundle carries a real artifact envelope and validates it through `decode_artifact` on every hit.
