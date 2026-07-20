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
last_verified: "2026-07-20"
ticket: "embedded-artifact-costs"
---

# Embedded-artifact cost probe

This harness generates deterministic Rust fixtures whose dependency-free stable
proc macro emits literal artifact tokens, builds a bounded decision matrix, and records build time, command-tree
peak RSS, source/intermediate/final sizes, Mach-O constant sections, and exact
payload occurrences in the linked binary.

Run the full matrix on macOS:

```sh
python3 spikes/embedding/measure.py \
  --preset decision \
  --output /tmp/tiler-embedding-measurement
```

Use `--preset smoke` for a three-case harness check. Add `--keep-work` to retain
generated Cargo workspaces. The harness invokes Cargo with `--offline` and does
not install, update, or otherwise mutate Rust or Apple toolchains.
The decision preset performs three independent fresh builds per matrix cell by
default; `--repetitions N` changes that bound.

`byte-string` emits one `Literal::byte_string` token per artifact, matching the
accepted proc-macro representation. `per-byte` emits one `Literal::u8_unsuffixed`
token per byte as a deliberately adverse control. `same` expands all artifacts
in the binary crate; `cross` expands one artifact in each dependency crate. Every byte is
read through `read_volatile` at runtime so the payload remains live, without
asserting that its address must be unique.

The raw `size -m` and build stdout/stderr are retained under `raw/`. JSON holds
the exact commands and full results; CSV is a compact analysis view. Reported
linker folding is an observation of the recorded host and flags, not a Rust,
LLVM, Mach-O, or linker guarantee.
