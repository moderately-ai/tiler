---
schema: "tiler-doc/v1"
id: "tiler.spike.cache"
kind: "experiment"
title: "Expansion cache crash and race spike"
topics: ["cache", "concurrency", "durability"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.cache.crash-race-protocol"]
entrypoints: ["spikes/cache/cache_harness.rs"]
last_verified: "2026-07-21"
ticket: "repair-cache-experiment-harness-integrity"
---

# Expansion cache crash and race spike

This process-level harness exercises immutable publication, advisory locking,
writer death, corruption, deletion, eviction, and uncached recovery.

```sh
rustc --edition 2021 spikes/cache/cache_harness.rs -o /tmp/tiler-cache-harness
/tmp/tiler-cache-harness selftest
/tmp/tiler-cache-harness selftest --stress 32
/tmp/tiler-cache-harness selftest --stress 32 --repetitions 10 \
  --evidence /tmp/tiler-cache-evidence.tsv
```

Every spawned child has an overall deadline. A timeout kills and reaps the
child and identifies its case deterministically. Each suite repetition also
injects a permanently blocked child and verifies that it fails within the
bounded deadline instead of hanging the harness.

`--repetitions` executes the complete suite independently for every repetition.
When `--evidence` is present, the harness synchronizes one compact tab-separated
row after each successful run. The tracked
[2026-07-21 result](results/macos-27.0-rustc-1.99.0-nightly-2026-07-21.tsv)
is the direct output of the documented ten-repetition command at stress 32.

It models local-filesystem process crashes, not power loss or every supported
filesystem. See the [research result](../../docs/research/cache/crash-and-race-protocol.md).
