---
schema: "tiler-doc/v1"
id: "tiler.spike.embedding"
kind: "experiment"
title: "Embedded-artifact cost and self-containment probes"
topics: ["embedding", "rustc", "binary-size", "proc-macros", "cargo", "rust-analyzer"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.embedding.artifact-costs", "tiler.research.embedding.self-contained"]
entrypoints: ["spikes/embedding/measure.py", "spikes/embedding/self_contained.py"]
last_verified: "2026-07-31"
ticket: "embedded-artifact-costs"
---

# Embedded-artifact cost and self-containment probes

Two harnesses answer two different questions about one representation. `measure.py`
asks what a byte-string embedding *costs* — build time, peak RSS, binary and
constant-section size, retained copies. `self_contained.py` asks whether it is
*self-contained*: whether the expanded code carries the artifact rather than
referring to it, and how Cargo and rust-analyzer behave around that.

# Embedded-artifact cost probe

This harness generates deterministic Rust fixtures whose dependency-free stable
proc macro emits literal artifact tokens, builds a bounded decision matrix, and
records build time, command-tree peak RSS, source/intermediate/final sizes,
Mach-O constant sections, and exact payload occurrences in the linked binary.

Run the full matrix on macOS:

```sh
uv run python spikes/embedding/measure.py \
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
uv run python spikes/embedding/measure.py \
  --verify-retained \
  docs/research/embedding/measurements/2026-07-20-macos-arm64
```

That verification does not reconstruct missing raw evidence or prove exact
reproducibility on a later toolchain. Reported linker folding remains an
observation of the recorded host and flags, not a Rust, LLVM, Mach-O, or linker
guarantee.

# Self-contained embedding probe

`self_contained.py` drives the Cargo workspace at `self-contained/`, whose proc
macro resolves a real artifact envelope through the real public
`tiler_cache::expansion::ExpansionCache` and emits it as one
`Literal::byte_string` token. Unlike the cost probe above, the payload is not
synthetic: the driver runs `prototypes/serial-sum-compile`, which drives the
offline Metal toolchain and writes envelopes carrying compiled `metallib`
objects, so every cache hit is validated by the real `decode_artifact` over
production-shaped bytes.

```sh
python3 spikes/embedding/self_contained.py --record macos-27.0-2026-07-31
uv run --with pytest pytest spikes/embedding/test_self_contained.py
```

`--skip-analyzer` runs the Cargo half alone. The pin declares
`profile = "minimal"` and ships no `rust-analyzer`, so the driver asks every
installed toolchain for one rather than trusting the rustup shim, which resolves
this directory's toolchain and fails; `--analyzer <path>` states one directly.
The expanding process is the pin's `libexec/rust-analyzer-proc-macro-srv` in
either case. The driver never installs, selects, or mutates a toolchain: a
missing `nightly-2026-07-20` or a missing analyzer stops the run with what is
needed, so an unreached axis is recorded as a boundary rather than assumed.

The run reconstructs everything it needs under a fresh run root — artifacts,
cache, target directories, and the generated standalone crate — and removes it
unless `--keep-root` is given. The fixture's `Cargo.lock` is generated once and
gitignored; every build after that is `--offline`.

It needs roughly 1 GiB free under `$TMPDIR`, or under `--root` when one is
given. Three target directories are unavoidable — the cross-crate axes must
race in their own, and the source-edit axis wipes its own to reach the cold
state it names — and the toolchain axis compiles the dependency graph a second
time under the other compiler. An earlier version used six and exhausted a
nearly full disk mid-run; the failure was loud (`No space left on device`) and
the run stopped rather than recording a scenario it had not completed, but the
cost was a wasted twenty minutes. `--keep-root` leaves all of it behind, so
clean up after using it.

What it records, and where. Fifteen scenario rows go to
`results/self-contained-embedding-<label>.tsv`: cold and warm embedding, the
three deletion cases, the standalone rebuild, and the four axes under both
drivers. The verbatim rendered text of seven failure classes goes to
`results/self-contained-diagnostics-<label>.txt`; each was reached by a build
that had to fail, and a class whose build succeeded fails the run.

Every deletion is proved in both directions: the same path must hold at least
one file before removal and none afterwards, so a mistyped path fails before
anything is deleted rather than passing an after-check for free. Every scenario
declares its expansion count, over one event file per expansion. The consumer's
printed length and checksum are compared against the producer's file as computed
independently by the driver, not only against the numbers the expansion recorded
beside the payload. `test_self_contained.py` runs each of those predicates
against inputs that must be rejected, including a per-byte representation, an
empty directory, and a literal containing escaped quotes.

See the [research report](../../docs/research/embedding/self-contained-embedding.md)
for the recorded results, the size and diagnostic gates as numbers, and the
complete list of what was not reached.
