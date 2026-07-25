---
schema: "tiler-doc/v1"
id: "tiler.spike.cache"
kind: "experiment"
title: "Expansion cache crash and race spike"
topics: ["cache", "concurrency", "durability"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.cache.crash-race-protocol", "tiler.research.cache.bounded-collection", "tiler.research.cache.supported-filesystems"]
entrypoints: ["spikes/cache/cache_harness.rs", "spikes/cache/filesystem_probe.rs", "crates/tiler-cache/src/expansion/harness.rs"]
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

## Collection under load

The same harness runs the collection ladder the research note's sixth gate asks
for, at 1, 8, and 32 real writer processes against a real collecting process:

```sh
cargo nextest run -p tiler-cache \
  -E 'test(collection_races_active_processes_at_one_eight_and_thirty_two)'
```

The scales are fixed in the case rather than taken from
`TILER_CACHE_HARNESS_CONCURRENCY`, because the ladder *is* the deliverable — a
run that silently used four would report having stressed 32 having stressed
nothing. `TILER_CACHE_HARNESS_EVIDENCE` still records one row per scale, and the
row carries the entries removed and the candidates the collector deliberately
left alone. How often the contended and superseded dispositions are reached is a
property of the host's scheduling, so the ladder records those totals rather than
asserting on them; each disposition has deterministic coverage in
`expansion::tests`, which holds the lock and replaces the entry itself. See
[the collection design](../../docs/research/cache/bounded-collection.md).

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

## Deciding whether a directory can hold a cache

Everything above assumes the filesystem provides what the protocol asks of it.
`filesystem_probe.rs` is what asks:

```sh
rustc --edition 2021 spikes/cache/filesystem_probe.rs -o /tmp/tiler-fs-probe
/tmp/tiler-fs-probe ~/Library/Caches \
  --across /Volumes/OtherFilesystem/scratch \
  --evidence /tmp/tiler-fs-evidence.tsv
```

It measures the six properties the cache rests on — one filesystem under the
root, `rename` replacing without ever exposing a missing name, `create_new`
refusing an existing path, a descriptor still readable after its file is
unlinked, an exclusive advisory lock excluding a *separate process* and released
when that process is killed, and a reportable modification time — plus how the
host maintains access time. One tab-separated row per property, and a non-zero
exit when a required one is refuted.

Two details keep a pass from being vacuous. The lock checks re-execute the probe
so the contenders are real processes that handshake over a pipe rather than
sleeping, and `--across` reports `skipped` rather than passing when the directory
it names turns out to share the root's device.

The tracked
[2026-07-25 result](results/filesystem-probe-macos-27.0-2026-07-25.tsv) covers
local APFS and a formatted exFAT RAM disk on one macOS host. No Linux filesystem
has been measured. See
[the supported-filesystem contract](../../docs/research/cache/supported-filesystems.md).
