---
schema: "tiler-doc/v1"
id: "tiler.spike.embedding"
kind: "experiment"
title: "Embedded-artifact cost probe"
topics: ["embedding", "rustc", "binary-size"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.embedding.artifact-costs"]
entrypoints: ["spikes/embedding/measure.py"]
last_verified: "2026-07-21"
ticket: "embedded-artifact-costs"
---

# Embedded-artifact cost probe

This harness generates deterministic Rust fixtures whose dependency-free stable
proc macro emits literal artifact tokens, builds a bounded decision matrix, and
records build time, command-tree peak RSS, source/intermediate/final sizes,
Mach-O constant sections, and exact payload occurrences in the linked binary.

Run the full matrix on macOS:

```sh
uv run --locked python spikes/embedding/measure.py \
  --preset decision \
  --output /tmp/tiler-embedding-measurement
```

Use `--preset smoke` for a three-case harness check. Add `--keep-work` to retain
generated Cargo workspaces. The harness invokes Cargo with `--offline` and does
not install, update, or otherwise mutate Rust or Apple toolchains.
The decision preset performs three independent fresh builds per matrix cell by
default; `--repetitions N` changes that bound. Each Cargo or inspection command
has a hard 600-second deadline by default; `--timeout-seconds N` may select a
value from 1 through 3,600 seconds. The complete run also has a hard one-hour
deadline; `--overall-timeout-seconds N` may select 1 through 21,600 seconds.
Measurement execution is macOS-only because
the metrics require `/usr/bin/time -l` and Mach-O `size -m` output. The output
directory must be absent or empty so stale files cannot be mistaken for the
current run.

`byte-string` emits one `Literal::byte_string` token per artifact, matching the
accepted proc-macro representation. `per-byte` emits one `Literal::u8_unsuffixed`
token per byte as a deliberately adverse control. `same` expands all artifacts
in the binary crate; `cross` expands one artifact in each dependency crate. Every byte is
read through `read_volatile` at runtime so the payload remains live, without
asserting that its address must be unique.

Successful schema-v2 runs retain raw `size -m` and build stdout/stderr under
`raw/`. They also record the harness revision and digest, generated source and
payload identities, executable identities, inherited Cargo/Rust environment,
deadlines, exact commands, and all required metrics. Missing or malformed time,
RSS, Mach-O, binary, or identity data makes the run fail rather than publishing
an apparently successful result. `complete.json` is the success predicate: it
is atomically published after required cleanup and identifies every retained
evidence file outside the optional `--keep-work` debugging tree. That tree is
reproducible scratch state, not published evidence. An absent marker means the
output is incomplete even if partial raw files remain.
Every inherited environment value is represented by its name, byte count, and
SHA-256 digest; values are not published, so output-affecting inputs remain
identifiable without leaking ambient credentials.

The checked-in 2026-07-20 result predates those controls. It contains complete
derived JSON and CSV rows but no raw logs or generated source workspaces. Its
freshness labels therefore do not independently prove package rebuilds or
proc-macro expansion counts, and its differing debug hashes do not retain the
binary evidence needed to attribute the cause. Verify
its retained structure and exact file digests without rerunning Cargo:

```sh
uv run --locked python spikes/embedding/measure.py \
  --verify-retained \
  docs/research/embedding/measurements/2026-07-20-macos-arm64
```

That verification does not reconstruct missing raw evidence or prove exact
reproducibility on a later toolchain. Reported linker folding remains an
observation of the recorded host and flags, not a Rust, LLVM, Mach-O, or linker
guarantee.
