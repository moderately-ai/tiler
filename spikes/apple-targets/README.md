---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets"
kind: "experiment"
title: "Apple Metal target compatibility and numerical spikes"
topics: ["apple-targets", "metal", "compatibility", "numerics", "subnormals", "runtime-compilation"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.apple-targets.compatibility", "tiler.research.apple-targets.numerical-behaviour"]
entrypoints: ["spikes/apple-targets/compatibility_probe.sh", "spikes/apple-targets/runtime_failure_probe.swift", "spikes/apple-targets/validate_compatibility_record.py", "spikes/apple-targets/test_probes.py", "spikes/apple-targets/numerical_probe.py", "spikes/apple-targets/numerical_probe_host.m", "spikes/apple-targets/test_numerical_probe.py"]
last_verified: "2026-07-24"
ticket: "apple-artifact-compatibility"
---

# Apple Metal target compatibility and numerical spikes

Two independent probes share this directory because they share a host row. The
compatibility probe answers which artifact families and deployment minima
produce which bytes. The numerical probe answers what Apple GPU `f32`
arithmetic actually does to subnormals, signed zero, and contraction. Neither
downloads or installs a toolchain component.

## Artifact-family and reproducibility probe

The compile probe records exact SDK/tool versions, commands, logs, artifact
digests, and byte comparisons for explicit macOS, iOS-device, and iOS-simulator
artifacts. It requires an installed Apple Metal toolchain. Its optional
argument is the result directory; omitting it preserves the run in a newly
created operating-system temporary directory.

```sh
spikes/apple-targets/compatibility_probe.sh \
  spikes/apple-targets/results/<yyyy-mm-dd>-<toolchain>
```

Success means the complete line-oriented `record.tsv` passed
`validate_compatibility_record.py`; compile-matrix success without valid host,
SDK, compiler, and linker provenance fails closed. Preserve `record.tsv`, SDK
settings, `input-manifest.tsv`, and command logs for any published measurement.
Schema v2 binds the repository base and exact harness, validator, kernel,
project, lockfile, and manifest digests. AIR and metallib
files are regenerable and ignored in the checked-in result area; their digests
remain in the record.

The retained 2026-07-21 local run is
[`results/2026-07-21-xcode26.6-metal32023.883/record.tsv`](results/2026-07-21-xcode26.6-metal32023.883/record.tsv).
Its SDK extracts and command logs are checked in beside it.

On a macOS Metal host, the Swift control distinguishes library, function, and
pipeline failure stages:

```sh
xcrun --sdk macosx swiftc spikes/apple-targets/runtime_failure_probe.swift -framework Metal -o /tmp/tiler-apple-runtime-probe
/tmp/tiler-apple-runtime-probe
```

The control exits nonzero for every unexpected library, function, or pipeline
outcome. Run its portable record-mutation tests and, on macOS, its compiled
runtime-stage injections with:

```sh
uv run --locked python spikes/apple-targets/test_probes.py
```

Old OS/GPU devices and cross-machine reproducibility remain unmeasured. See the
[compatibility report](../../docs/research/apple-targets/artifact-compatibility.md).

## Numerical-behaviour probe

`numerical_probe.py` generates probe kernels in the Metal emitter's output
shape and puts each one through **both** compilation stages Tiler's Metal story
has. Offline: to LLVM IR, to AIR, and to a linked metallib under a matrix of
math modes, optimization levels, and contraction settings. In process: the
byte-identical source through `[device newLibraryWithSource:options:error:]`
with an explicit `MTLCompileOptions`, across the math modes and both
`MTLLibraryOptimizationLevel` values. Both then take the same path to the GPU
through `numerical_probe_host.m`, so a difference between them is a difference
between the two compilers rather than between two dispatch procedures.
`path_comparisons` pairs them case by case and a divergence fails the gate.

On the measured host those are two different compiler builds — offline
`metalfe-32023.883` from the Xcode toolchain asset, runtime `metalfe-32023.921`
from `/System/Library/PrivateFrameworks/MTLCompiler.framework` — which is why
`environment.runtime_compiler` is recorded separately and is part of what makes
two runs comparable.

Print a run and rewrite the retained record with:

```sh
uv run --locked python spikes/apple-targets/numerical_probe.py \
  --record spikes/apple-targets/results/<yyyy-mm-dd>-numerics-<toolchain>/record.tsv
```

Pass `--work-dir spikes/apple-targets/local-work` to keep the generated
sources, IR, AIR, and libraries for inspection; that directory is ignored.

The assertions live in `test_numerical_probe.py`, which the repository gate
collects through `pytest`'s `spikes/apple-targets` test path, so every finding
is re-established on every gate run rather than trusted from a document:

```sh
uv run --locked python -m pytest -c pyproject.toml spikes/apple-targets
```

**The guard, which matters more than the numbers.** A relaxed math mode can
appear to honour a strict contract by deleting the arithmetic that would have
violated it, and it does so on this row. The harness therefore never reads a
subnormal claim out of a returned bit pattern alone. `subnormal_verdict`
admits an observation only when the emitted module retains a floating-point
operation *and* the same kernel returns its declared execution witness, a
non-subnormal operand whose result differs from the operand exactly when the
arithmetic ran. A kernel that is an identity on every operand has no possible
witness, declares `witness = None`, and can never support a claim. Both layers
are needed: at `-O0` under `relaxed` the front end still emits both operations
and a stage below it removes them anyway. Extend the kernel table only with
that contract intact.

**The runtime path has no readable module, and the harness says so.**
`newLibraryWithSource:options:` returns an opaque `MTLLibrary`, so the first
layer is unavailable there. `Observation.operations` is `None` for a runtime
case and never `()` — `()` is a measured absence of arithmetic, `None` records a
question that could not be asked — the record omits the `float_operations` row
for those cases rather than writing an empty one, and portable guard tests pin
both. The execution witness carries the decision alone, which is the layer that
caught the trap at `-O0` where counting emitted operations did not; and because
a guard that never refuses anything is not a guard, every run must show that
layer refusing the trap kernel under `relaxed` and `fast` and admitting it under
`safe`. Serializing an `MTLBinaryArchive` recovers the runtime compiler's
version string and the presence of individual `air.compile.*` names, but the
container has no published layout and stores its strings concatenated, so that
scan is corroboration and feeds nothing in the guard.

**Where `MTLCompileOptions` has no counterpart.** `-target`, `-ffp-contract`,
and `-O0` have no property to set. None is approximated: the gaps are recorded
in `OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART` and in the record, and each
runtime case is compared against *every* offline contraction setting for its
kernel and mode rather than against one chosen row, so a kernel on which
contraction is observable reports which offline setting the runtime path behaves
like instead of reading as a divergence.

**Self-skip.** Every measurement resolves the toolchain, SDK, and GPU first and
skips when any is absent, following `DriverError::{ToolchainUnavailable,
SdkUnavailable}` and adding the device axis the offline driver has no name for
because it never dispatches. The skip reason is printed on standard error and
appears in `pytest -ra`; setting `TILER_REQUIRE_METAL_TOOLCHAIN` — the same
variable `crates/tiler-metal/src/golden_compilation.rs` reads — turns it into a
failure. The guard tests over synthetic observations carry no such condition
and run everywhere, including on a host with neither an Apple toolchain nor
`git`.

**What it costs the gate.** `uv run --locked python -m pytest -c pyproject.toml
spikes/apple-targets` takes about 11.6 s on the measured host, against 8.2 s
before the runtime path was added; the numerical probe itself is about 10.5 s of
one gate run, covering 44 offline cases and 40 runtime ones. On a host with no
Apple toolchain the whole thing skips in well under a second.

The retained 2026-07-24 run is
[`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv),
schema `tiler.apple-numerical-behaviour/v2`. The directory name identifies the
offline toolchain; `environment.runtime_compiler` names the second one. The gate
compares a live run's `case.*` and `comparison.*` rows against it whenever the
environment row matches and announces a skip when it does not, because a
different toolchain build legitimately produces different values. See the
[numerical-behaviour report](../../docs/research/apple-targets/numerical-behaviour.md)
for the findings, the disagreements with the values it reproduces, and the
measurement boundaries.
