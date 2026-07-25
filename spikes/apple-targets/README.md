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
last_verified: "2026-07-25"
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
has, for **each of the three artifact families** `MetalPlatform` declares —
`MacOs`, `IOsDevice`, and `IOsSimulator` — each with its own `--sdk` and
`-target`. Offline: to LLVM IR, to AIR, and to a linked metallib under a matrix
of math modes, optimization levels, and contraction settings. In process: the
byte-identical source through `[device newLibraryWithSource:options:error:]`
with an explicit `MTLCompileOptions`, across the math modes and both
`MTLLibraryOptimizationLevel` values. Both then take the same path to the GPU
through `numerical_probe_host.m`, so a difference between them is a difference
between the two compilers rather than between two dispatch procedures.
`path_comparisons` pairs them case by case *within a family* and a divergence
fails the gate.

**The compile side and the device side do not reach equally far.** The compile
side needs no GPU and runs for all three families, so a per-family difference in
`air.compile.denorms_disable`, in the fast-math licence spellings, or in the
surviving operation count is a first-class result. The device side runs a family
only in *its own* execution environment: macOS on the host GPU, `IOsSimulator`
through `simctl spawn` on a booted runtime, and `IOsDevice` nowhere, because this
host has no iPhone or iPad attached. On the measured row the macOS GPU will load
and run an iOS-device metallib without complaint — `hazard.cross_family_load.*`
records that — but the GPU executing it is the Mac's, so the harness refuses to
treat it as a device measurement: a case for a family with no attached device
carries `Observation.results = None`, gets the `no-device-observation` verdict,
and has no `results` row in the record.

The compilers are recorded per family. The offline driver is one binary shared
by every SDK; the runtime compiler belongs to the *execution environment*, so on
the measured host it differs between families — macOS resolves
`GPUCompiler.framework` build `metalfe-32023.921` (the framework carrying
`MTLCompiler`; the offline driver is `metalfe-32023.883`), while the iOS
Simulator runtime loads its own `metalfe-32023.830.1`. `report_compiler_images`
in the dispatch host reports the image dyld actually loaded and
`environment.family.<name>.runtime_compiler_build` records its build, so no
family's runtime compiler is inherited from another's row.

**The operation vocabulary, and the two matrices it is measured in.** The kernels
cover multiply, add, division in both the power-of-two form the driver rewrites
into a multiply and the form it keeps, a source-level `fma`, and a two-add chain
whose value says where the parentheses went. The swept axes are the three math
modes, the three contraction settings, both `-fmetal-math-fp32-functions`
values, and all five offline optimization levels. That costs more than the gate
should pay on every run, so `cases` assembles a `covering` set — at least one
case of every kernel, mode, level, contraction setting, and fp32-functions
value, plus every case a finding cites — and an `exhaustive` cross product
selected by `TILER_APPLE_NUMERICS_EXHAUSTIVE`. `probe.matrix` names which one
produced a record and `matrix_mismatch` refuses to compare one against a run of
the other. A portable guard test holds the covering set to its coverage claim.

**A kernel added here must be checked against what the module emitted.** Widening
the vocabulary found the harness reporting a kernel whose whole body is one
`fma` as containing no floating-point arithmetic at all: this front end lowers
`fma(x, a, b)` to `@air.fma.f32` and `FUSED_INTRINSIC` named only the `llvm.`
spellings. The verdict still failed closed, but the count was wrong in the
direction a reader acts on, because a surviving operation reported as zero looks
exactly like a deleted one. Both spellings are matched now. Every hand-written
`SubnormalProbe`, `OrderProbe`, and `Witness` is also checked against `evaluate`,
which derives each candidate result from the kernel under exact arithmetic and
under the sign-preserving flush, so a mis-stated literal is a portable test
failure rather than a silently wrong classification.

Print a run and rewrite the retained record with:

```sh
uv run --locked python spikes/apple-targets/numerical_probe.py \
  --record spikes/apple-targets/results/<yyyy-mm-dd>-numerics-covering-<toolchain>/record.tsv

TILER_APPLE_NUMERICS_EXHAUSTIVE=1 \
  uv run --locked python spikes/apple-targets/numerical_probe.py \
  --record spikes/apple-targets/results/<yyyy-mm-dd>-numerics-exhaustive-<toolchain>/record.tsv
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

**A guard layer can be missing on either side, and neither may be defaulted.**
The runtime path returns an opaque `MTLLibrary`, so layer 1 is unavailable there:
`Observation.operations` is `None`, never `()` — `()` is a measured absence of
arithmetic, `None` records a question that could not be asked — and the record
omits the `float_operations` row rather than writing an empty one. A family with
no attached device loses layer 2 instead: nothing was dispatched, so
`Observation.results` is `None`, never `()`, and the record omits the `results`
row. Losing layer 1 is cheap because layer 2 is sufficient where layer 1 is only
necessary; losing layer 2 is the expensive direction, so a compile-side-only
observation can never be admissible evidence and gets its own
`no-device-observation` verdict rather than being classified by the layer it
still has. Portable guard tests pin both `None`s on a host with no Apple
toolchain. Where only layer 2 remains, every run must still show it refusing the
trap kernel under `relaxed` and `fast` and admitting it under `safe`.
Serializing an `MTLBinaryArchive` recovers the runtime compiler's version string
and the presence of individual `air.compile.*` names, but the container has no
published layout and stores its strings concatenated, so that scan is
corroboration and feeds nothing in the guard — and in the iOS Simulator it
aborts the process, so `archive_support` probes for it in a one-entry batch of
its own before any manifest that carries measurements asks for one.

**Where `MTLCompileOptions` has no counterpart.** `-target`, `-ffp-contract`,
and `-O0` have no property to set. None is approximated: the gaps are recorded
in `OFFLINE_FLAGS_WITHOUT_RUNTIME_COUNTERPART` and in the record, and each
runtime case is compared against *every* offline contraction setting for its
kernel and mode rather than against one chosen row, so a kernel on which
contraction is observable reports which offline setting the runtime path behaves
like instead of reading as a divergence.

**Self-skip, and per-family self-skip.** The whole probe resolves the toolchain
and every family's SDK first and skips when any is absent, following
`DriverError::{ToolchainUnavailable, SdkUnavailable}` and adding the device axis
the offline driver has no name for because it never dispatches. The skip reason
is printed on standard error and appears in `pytest -ra`; setting
`TILER_REQUIRE_METAL_TOOLCHAIN` — the same variable
`crates/tiler-metal/src/golden_compilation.rs` reads — turns it into a failure.
A family whose *own* execution environment is absent is a different thing again
and is neither a skip nor a failure: its compile side still runs and its
device-side assertions announce the family and skip that family alone, so the
record never silently loses a family and never gains a device-side claim it did
not measure. The guard tests over synthetic observations carry no condition at
all and run everywhere, including on a host with neither an Apple toolchain nor
`git`.

**What it costs the gate.** `uv run --locked python -m pytest -c pyproject.toml
spikes/apple-targets` takes about 47 s on the measured host once a simulator is
booted, covering the 204 offline cases of the covering matrix across three
families and 164 runtime ones across the two families that dispatch; the same
command took about 20 s over 126 and 80 before the matrix was widened, on the
same host. The exhaustive matrix adds 99 offline cases and about 5 s to a probe
run. All of these were measured while several other worktrees were running their
own gates, so treat them as an upper bound with a loaded host rather than as a
clean figure. The one-time cost of booting a cold simulator adds roughly 8 s to
the first gate run that needs it; the harness leaves the device booted so
subsequent runs pay only one `simctl spawn` per family. On a host with no Apple
toolchain the whole thing skips in well under a second.

The retained 2026-07-25 runs are
[`results/2026-07-25-numerics-covering-xcode26.6-metal32023.883/record.tsv`](results/2026-07-25-numerics-covering-xcode26.6-metal32023.883/record.tsv)
and
[`results/2026-07-25-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv`](results/2026-07-25-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv),
schema `tiler.apple-numerical-behaviour/v4`. The schema `v3` record
[`results/2026-07-24-numerics-families-xcode26.6-metal32023.883/record.tsv`](results/2026-07-24-numerics-families-xcode26.6-metal32023.883/record.tsv)
is retained as the previous row and is no longer compared against. The directory
name identifies the offline toolchain and the matrix; the `environment.family.*`
rows identify each family's SDK, emitted triple, execution environment, and both
compilers. The gate compares a live run's `case.*`, `comparison.*`, and
`hazard.*` rows against the record for the matrix it measured, whenever the
environment row matches, and announces a skip when it does not, because a
different toolchain build or simulator runtime legitimately produces different
values. See the
[numerical-behaviour report](../../docs/research/apple-targets/numerical-behaviour.md)
for the findings, whether the subnormal flush is Apple-wide or per-family, the
disagreements with the values it reproduces, and the measurement boundaries.
