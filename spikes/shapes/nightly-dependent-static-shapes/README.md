---
schema: "tiler-doc/v1"
id: "tiler.spike.shapes.nightly-dependent-static-shapes"
kind: "experiment"
title: "Nightly dependent-array static-shape conformance"
topics: ["shapes", "rust", "const-generics", "diagnostics"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.shapes.nightly-const-shape-parameters"]
entrypoints: ["spikes/shapes/nightly-dependent-static-shapes/check.sh", "spikes/shapes/nightly-dependent-static-shapes/measure.py"]
last_verified: "2026-07-20"
ticket: "spike-nightly-arbitrary-rank-shape-evidence"
---

# Nightly dependent-array static-shape conformance

This isolated workspace tests ADR 0067's exact dependent-array evidence form on
the governed `nightly-2026-07-19` compiler. It keeps the graph authoritative:
Rust evidence is sealed, privately attached only after checked refinement,
explicitly weakenable, and absent from the model's semantic identity.

The fixture covers ranks 0 through a rank-64 probe, equivalent aliases across
independent crates, private and public constants, reexports, stable proc-macro
token generation, compile-fail diagnostics, evidence forgery, foreign-graph
rejection, exact feature-gate requirements, and an isolated borrowed-slice
comparison.

The retained feature-boundary probes also distinguish evidence preservation
from generic evidence derivation. The governed features admit scalar-broadcast
preservation and caller-selected checked output evidence. Adding
`generic_const_exprs` admits `Rank<{ RANK - 1 }>` but still rejects a generic
exact-extent array with one axis removed. The emerging `generic_const_args`
path is not usable on either tested compiler without additional solver state
and remains outside the governed profile.

The repository root `rust-toolchain.toml` is the sole governed compiler pin.
The check entrypoint deliberately has no fallback: callers pass that canonical
pin explicitly, while adjacent-nightly migration probes may pass another exact
dated nightly without adding a second toolchain file:

```sh
spikes/shapes/nightly-dependent-static-shapes/check.sh "$(command -v rustup)" nightly-2026-07-19
spikes/shapes/nightly-dependent-static-shapes/check.sh "$(command -v rustup)" nightly-2026-07-20
```

Regenerate the ignored 1/10/100/1,000-shape sources and the compact checked-in
measurement summary through the locked repository Python environment:

```sh
uv run --locked python spikes/shapes/nightly-dependent-static-shapes/measure.py
```

Raw compiler output and generated workloads are ignored. The retained summary
records exact compiler commits, host provenance, wall time, peak RSS, release
binary size, and global symbol counts. These measurements reject catastrophic
behavior on the tested host; they are not portable performance guarantees.

Both compilers pass the same correctness, diagnostics, Clippy, and rustdoc
suite. On the governed compiler, the 1,000-shape case completed a clean check in
0.132 seconds at 86.2 MiB peak RSS and a release build in 0.240 seconds. Its
binary was 16 bytes larger than the one-shape case and retained the same global
symbol count. See [`measurements/summary.json`](measurements/summary.json) for
the complete matrix and exact host boundary.
