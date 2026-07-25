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
entrypoints: ["spikes/cache/cache_harness.rs", "crates/tiler-cache/src/expansion/harness.rs"]
last_verified: "2026-07-25"
ticket: "port-the-cache-harness-to-the-production-bundle"
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

## The production bundle, measured separately

Everything above exercises this spike's **own miniature frame**. The same nine
kill points now also run against the frame `tiler-cache` actually publishes, and
that harness lives in the crate rather than here: reaching a phase inside
`ExpansionCache::publish` needs a seam no external process can name, and the seam
is `cfg(test)` so it exists in that crate's test binary and nowhere else. A
Cargo feature would have been externally reachable and is deliberately not used,
because Cargo unifies features across a build graph and one unrelated crate
enabling it would arm mid-publication aborts inside a production cache.

Children are real processes: the harness re-executes the test binary, and the
armed child calls `abort` inside the real publication path. Run it with:

```sh
cargo nextest run -p tiler-cache -E 'test(expansion::harness)'
```

That is the cheap form the repository gate runs — one repetition, four
concurrent children. A measurement uses more and records one row per case:

```sh
TILER_CACHE_HARNESS_REPETITIONS=10 \
TILER_CACHE_HARNESS_CONCURRENCY=32 \
TILER_CACHE_HARNESS_EVIDENCE=/tmp/tiler-cache-production-evidence.tsv \
cargo nextest run -p tiler-cache -E 'test(expansion::harness)' \
  --test-threads 1 --success-output never
```

The tracked
[2026-07-25 result](results/production-bundle-macos-27.0-rustc-1.99.0-nightly-2026-07-19.tsv)
is that command's output, with the host and toolchain header added. It is an
observation about one host, not a portable guarantee.

**One substitution is stated in the evidence header rather than hidden.** The
children drive the crate-private `resolve` with a payload validator accepting any
non-empty bytes, not the public `get_or_publish`, whose validator is
`decode_artifact`. Building a real artifact envelope needs a `SemanticProgram`,
which needs `tiler-ir`, which ADR 0082 item 2 decides `tiler-cache` does not
depend on. Every byte of the bundle frame and every filesystem operation is real;
the substituted validator sits strictly inside an envelope the frame has already
delimited, so it changes how long the pre-rename window is and not what a killed
writer leaves at a content path. A positive end-to-end hit carrying a real
compiled artifact is therefore still unmeasured and belongs to the orchestrator
holding both crates.
