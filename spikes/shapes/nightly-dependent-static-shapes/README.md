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
last_verified: "2026-07-21"
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

## Why this spike retains no diagnostics record

Its sibling [`shape-evidence`](../shape-evidence/README.md) checks its `.stderr` files against [a record](../shape-evidence/results/2026-07-24-macos-arm64.json) because they cannot be recompiled here: they were captured on stable 1.89.0, so nothing else would notice them decaying. This workspace is the opposite posture. Its diagnostics were captured on the nightly `rust-toolchain.toml` pins, so [`check.sh`](check.sh) recompiles them directly, and a fixture edited until it no longer fails for its recorded reason fails that run. Reproduction is the total check, and a record would restate on the side what the compiler restates on every run — including first lines and fragments that would then have to be re-recorded by hand at each pin migration.

Reproduction is no longer automatic: nothing runs `check.sh` for you, so the goldens sit unverified until someone working on this spike runs it.

The one thing reproduction genuinely cannot see is a case deleted from both the tree and the expectations, since the glob would simply resolve to fewer fixtures and still pass. `retained_fixture_inventory_is_complete` in [`conformance/tests/ui.rs`](conformance/tests/ui.rs) closes that directly: it names every compile-fail and compile-pass case and requires one retained diagnostic per compile-fail case, so losing evidence a governed decision cites has to be a deliberate edit to a named list.

What remains open is narrower, and `record-gated-shape-spike-diagnostic-claims` tracks it. Compilation proves that a fixture and its diagnostic agree; it does not prove the agreed diagnostic is still the claim ADR 0067 relies on, so a fixture weakened in step with its `.stderr` would pass. `spikes/extensions/non-exhaustive-visibility` carries both halves for exactly that reason. Closing it here belongs with lifting that probe's gated-record verifier into a shared form rather than growing a second copy under `spikes/shapes`.

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
uv run python spikes/shapes/nightly-dependent-static-shapes/measure.py
```

Raw compiler output and generated workloads are ignored. The retained summary
is derived directly by that entrypoint and records its actual UTC run date,
exact compiler commits, host provenance, wall time, peak RSS, release binary
size, and global symbol counts. Every compiler/tool subprocess runs in its own
process group with a 300-second overall deadline; a timeout kills the group and
fails without publishing a replacement summary. The harness requires POSIX
process groups, `nm`, and `/usr/bin/time`: BSD `time -lp` on macOS or GNU
`time -v` on Linux. Binary size uses Python's portable `Path.stat()` rather
than platform-specific `stat` flags. These measurements reject catastrophic
behavior on the tested host; they are not portable performance guarantees.

Both compilers pass the same correctness, diagnostics, Clippy, and rustdoc
suite. On the retained arm64 macOS 27 host and governed compiler, the
1,000-shape case completed a clean check in
0.166 seconds at 85.9 MiB peak RSS and a release build in 0.323 seconds. Its
binary was 16 bytes larger than the one-shape case and retained the same global
symbol count. See [`measurements/summary.json`](measurements/summary.json) for
the complete matrix and exact host boundary.
