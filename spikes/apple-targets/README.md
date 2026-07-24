---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets"
kind: "experiment"
title: "Apple Metal target compatibility and numerical spikes"
topics: ["apple-targets", "metal", "compatibility", "numerics", "subnormals"]
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
shape, compiles each one to LLVM IR, to AIR, and to a linked metallib under a
matrix of math modes, optimization levels, and contraction settings, then
dispatches the library on the local GPU through `numerical_probe_host.m` and
classifies what came back. Print a run and rewrite the retained record with:

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

**Self-skip.** Every measurement resolves the toolchain, SDK, and GPU first and
skips when any is absent, following `DriverError::{ToolchainUnavailable,
SdkUnavailable}` and adding the device axis the offline driver has no name for
because it never dispatches. The skip reason is printed on standard error and
appears in `pytest -ra`; setting `TILER_REQUIRE_METAL_TOOLCHAIN` — the same
variable `crates/tiler-metal/src/golden_compilation.rs` reads — turns it into a
failure. The guard tests over synthetic observations carry no such condition
and run everywhere.

The retained 2026-07-24 run is
[`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv).
The gate compares a live run against it whenever the environment row matches
and announces a skip when it does not, because a different toolchain build
legitimately produces different values. See the
[numerical-behaviour report](../../docs/research/apple-targets/numerical-behaviour.md)
for the findings, the disagreements with the values it reproduces, and the
measurement boundaries.
