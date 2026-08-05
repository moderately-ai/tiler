---
schema: "tiler-doc/v1"
id: "tiler.spike.cache"
kind: "experiment"
title: "Expansion cache crash and race spike"
topics: ["cache", "concurrency", "durability"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.cache.crash-race-protocol", "tiler.research.cache.bounded-collection", "tiler.research.cache.supported-filesystems", "tiler.research.cache.build-tool-exercise"]
entrypoints: ["spikes/cache/cache_harness.rs", "spikes/cache/filesystem_probe.rs", "spikes/cache/build_tool_exercise.py", "crates/tiler-cache/src/expansion/harness.rs"]
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
writer leaves at a content path.

That substitution is lifted by the build-tool exercise below, which is an
orchestrator holding both crates and drives the public `get_or_publish`. The
narrower gap it stated — its payload is *declared* rather than *carried* — was
closed on 2026-07-31 by the self-contained embedding spike (`spikes/embedding/self-contained/`),
which put envelopes carrying compiled `metallib` objects through the same
`get_or_publish` with every hit validated by the real `decode_artifact`; the
exercise's own rows still carry no object bytes, and that remains its boundary.

## Under Cargo and rust-analyzer

Everything above drives the cache from a harness. `build-tool-exercise/` is a
Cargo workspace whose proc macro resolves a real artifact through the real public
[`ExpansionCache`], so the processes driving the protocol are the ones ADR 0050's
context sentence names — `cargo` and a `rust-analyzer` proc-macro server — rather
than workers a harness spawned.

```sh
python3 spikes/cache/build_tool_exercise.py --skip-analyzer --concurrency 3
python3 spikes/cache/build_tool_exercise.py --concurrency 3 \
  --analyzer "$(rustup which --toolchain nightly rust-analyzer)" \
  --record macos-27.0-2026-07-25
```

The build closure is a genuine `tiler-compiler` session encoded through
`tiler-artifact`, deliberately not memoized in-process: a `OnceLock` would make
repeat expansions cheap and would hide the exact quantity being measured.

Two properties keep a pass from being vacuous, and both are checks that have
failed during development rather than decorations.

**The population is counted.** Each scenario declares how many expansion events
must exist and fails when the count differs, so a scenario that expanded nothing
cannot report success. Events are one file per expansion rather than appended
lines, because several uncoordinated processes write at once and an interleaved
append can lose a record with no reader noticing.

**Concurrency is observed, not assumed.** Each event carries its wall-clock
window, and the concurrent scenarios fail unless expansions in *different*
processes genuinely intersect. Three builds that happened to serialize and three
that raced produce identical outcome counts, so without this the scenario would
claim a workload it never reached — and at first it did not reach it, because the
compile was a few milliseconds wide.

**`negative-control-x3` is the reason the other rows mean anything.** It runs the
same race with the cache root pointed at a file, so no namespace can be created:
every resolution falls open to `uncached` and every process compiles every key.
It is what proves the driver can *see* duplicate compilation, which is what makes
"one compile per key" in `cargo-concurrent-x3` evidence rather than an artifact of
a counter that never moves.

Ordering, where a scenario needs it, is established by observed state: an
expansion writes a marker file before it waits and removes it after, and a driver
that wants to kill a lock holder waits for that file. No scenario rests on a
wall-clock margin.

The tracked
[2026-07-25 result](results/build-tool-exercise-macos-27.0-2026-07-25.tsv) is the
recorded command's output. See
[the build-tool exercise](../../docs/research/cache/build-tool-exercise.md).

**Measurement — re-run at `63f9259` on 2026-08-01, reproduced.** `envelope/` had stopped compiling: `compile_governed` now returns one `Compilation` rather than a collection, and `bind-the-artifact-variant-abi-to-the-program-abi` moved the variant ABI replay inside `push_variant`, deleting the `accessible_bytes`, `grid_threads`, `threads_per_workgroup`, and `applicability_guard` fields this spike used to declare by hand. The exercise now assembles the way `crates/tiler-build/src/metal_plan.rs` does, and the [2026-08-01 result](results/build-tool-exercise-macos-27.0-2026-08-01.tsv) is the same recorded command's output at that commit. Every counted quantity — events, builds, published, hit, uncached, processes, and drivers, in all eight scenarios — is identical to the 2026-07-25 row beside it, including the `negative-control-x3` row that makes the others mean anything. `overlaps` and `seconds` differ run to run and are not claims: `overlaps` is the observed-concurrency check, which requires only that expansions in different processes genuinely intersect, and three runs at this commit produced 21, 18, and 5 for `cargo-concurrent-x3` while the counted columns did not move.

**Measurement — re-run at `9ec8028c` on 2026-08-05, reproduced.** `envelope/` had stopped compiling again, on two drifts and exactly two: `NumericalContract` is now a composed record whose contracts are associated constants, so `NumericalContract::FlushSubnormalsToZeroF32` is `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`; and `BackendEntryRef` carries `payloads: Vec<PayloadId>`, one per delivery position and never empty, rather than a single `payload`. Both are spelled the current way in `hot-path-efficiency/harness/src/envelope.rs`, which never stopped compiling, and the [2026-08-05 result](results/build-tool-exercise-macos-27.0-2026-08-05.tsv) is the same recorded command's output with those two repairs applied. Seventy-two counted cells were compared — events, builds, published, hit, uncached, processes, cwds, drivers, and the note, across all eight scenarios — and every one is identical to the 2026-08-01 row beside it, `negative-control-x3`'s twelve compilations against `cargo-concurrent-x3`'s four included. `overlaps` and `seconds` moved again and again carry no claim: `cargo-concurrent-x3` observed 19 overlapping pairs where that row observed 5, and the two analyzer scenarios took roughly twice as long, both being quantities the driver records rather than asserts on.

**Measurement — re-run at `d5960e81` on 2026-08-05, reproduced, and no new fixture was minted.** `restore-the-spikes-against-the-composed-numerical-contract` re-checked this exercise because 146 commits had landed since the restoration above, 37 of them touching `crates/` across 91 files — including `tiler-cache/src/expansion/`, `tiler-artifact/src/program/model.rs`, and `tiler-compiler/src/session.rs`. `envelope/` needed **no** repair this time: `cargo check --workspace --all-targets` was clean, so the two drifts recorded above are still the current spellings. The recorded command was then run in full, analyzer scenarios included, and all **72** counted cells — events, builds, published, hit, uncached, processes, cwds, drivers, and the note, across all eight scenarios — are identical to the 2026-08-05 row beside it, `negative-control-x3`'s twelve compilations against `cargo-concurrent-x3`'s four included. The comparison was proved able to say no by perturbing one cell (`negative-control-x3.builds` 12 → 4) and watching it report exactly that difference; it also counts the cells it compared and fails when the population is not 72, so "no differences" cannot be confused with "the comparison did not run". `overlaps` and `seconds` moved as always and carry no claim: `negative-control-x3` observed 7 overlapping pairs where that row observed 21, and the `analyzer` scenario ran in 10.3 seconds against 18.1.

No fixture was recorded because every counted cell reproduced and the label would have been the same `macos-27.0-2026-08-05` the row above already holds — a second file for a different commit under one date, or an overwrite of evidence taken at `9ec8028c`. The retained [2026-08-05 result](results/build-tool-exercise-macos-27.0-2026-08-05.tsv) *is* the row this run reproduced, and the command that reproduces it is the one printed above.

**Measurement — re-run for `wire-the-delivered-realization-record-into-the-artifact`, based at `55d1d09f`, reproduced; again no fixture minted.** That ticket made every executable artifact carry a delivered-realization record, so `envelope/` stopped *producing* an artifact while continuing to compile: `ArtifactProgramBuilder::build` refuses a draft that never called `declare_realization`, which no `cargo check` detects. The repair is one `declare_realization` call and one `realization_record` helper beside it, deriving the eleven governed resolutions from the packaged program's own scheduled realization rather than restating them. The recorded command was then re-run in its `--skip-analyzer --concurrency 3` form, and every counted cell of all **five** cargo scenarios — events, builds, published, hit, uncached, processes, cwds, drivers, and the note — is identical to the 2026-08-05 row beside it, `negative-control-x3`'s twelve compilations against `cargo-concurrent-x3`'s four included. The three analyzer scenarios were not re-run and are therefore unverified at this base, which is the boundary of this note rather than a claim about them. `overlaps` and `seconds` moved as always and carry no claim: `cargo-concurrent-x3` observed 21 overlapping pairs where that row observed 19.

Only this exercise was re-run on 2026-08-01, and only this exercise again on 2026-08-05, twice — once at `9ec8028c` and once at `d5960e81`. The harness, production bundle, collection ladder, and filesystem probe above still carry their 2026-07-25 verification, which is why the frontmatter's `last_verified` is unchanged.

[`ExpansionCache`]: ../../crates/tiler-cache/src/expansion/store.rs

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

## What the protocol costs

Everything above asks whether the protocol is *correct* under crashes, races, build tools, and filesystems. What it costs is a separate question with its own harness and its own experiment record: [`hot-path-efficiency/`](hot-path-efficiency/README.md) measures hit latency decomposed, publication latency, scan and eviction cost, and the population scaling of each, through the public `ExpansionCache` with every hit validated by the real `decode_artifact`. It is a sibling of this record rather than part of it — a different question, a different boundary, and a result nobody should read as evidence about correctness. See [the hot-path efficiency note](../../docs/research/cache/hot-path-efficiency.md).
