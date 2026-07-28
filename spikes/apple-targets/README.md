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
entrypoints: ["spikes/apple-targets/compatibility_probe.sh", "spikes/apple-targets/runtime_failure_probe.swift", "spikes/apple-targets/validate_compatibility_record.py", "spikes/apple-targets/test_probes.py", "spikes/apple-targets/numerical_probe.py", "spikes/apple-targets/numerical_probe_host.m", "spikes/apple-targets/test_numerical_probe.py", "spikes/apple-targets/bfloat_dispatch_probe.py"]
last_verified: "2026-07-25"
ticket: "apple-artifact-compatibility"
---

# Apple Metal target compatibility and numerical spikes

Two independent probes share this directory because they share a host row. The
compatibility probe answers which artifact families and deployment minima
produce which bytes. The numerical probe answers what Apple GPU scalar
arithmetic actually does to subnormals, signed zero, and contraction — and, since
the dtype axis was added, that the answer is not the same for `f32` and `f16`.
Neither downloads or installs a toolchain component.

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
uv run python spikes/apple-targets/test_probes.py
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
cover multiply, add — including one bare add whose subnormal operand comes
straight from the buffer rather than out of a preceding multiply — division in
both the power-of-two form the driver rewrites into a multiply and the form it
keeps, a source-level `fma`, and a two-add chain whose value says where the
parentheses went. The swept axes are the three math
modes, the three contraction settings, both `-fmetal-math-fp32-functions`
values, and all five offline optimization levels. That costs more than the gate
should pay on every run, so `cases` assembles a `covering` set — at least one
case of every kernel, mode, level, contraction setting, and fp32-functions
value, plus every case a finding cites — and an `exhaustive` cross product
selected by `TILER_APPLE_NUMERICS_EXHAUSTIVE`. `probe.matrix` names which one
produced a record and `matrix_mismatch` refuses to compare one against a run of
the other. A portable guard test holds the covering set to its coverage claim.

**The dtype is not an axis of that matrix.** `DTYPES` names `f32`, `f16`, and
`bf16`, and each one owns its operand vector, its result width, its MSL constant
spelling and canonicalization helper, its exact evaluation, and its element width
and sentinel in the dispatch host. A kernel names its dtype in its own name only
when that dtype is not `f32`, so every case key recorded while the harness
measured one dtype keeps its exact meaning and the `_f16` and `_bf16` kernels are
the only new ones. Both narrow dtypes cover multiply in both flush directions, a
bare add, a surviving `fdiv` in both directions, materialization, the identity
multiply, and the trap — a portable test holds the two kernel sets to being the
same list at two widths, so neither can gain coverage the other silently lacks.
Contraction and reassociation are `f32`-only and are named as a boundary in the
research record. A source-level `fma` is `f32`-only for a stronger reason at
`bf16`: this front end has no `bfloat` overload, so `fma(bfloat, …)` returns
`float` and emits `fpext`/`air.fma.f32`/`fptrunc`, which would measure `f32`
arithmetic wearing `bfloat` operands.

**`bf16` needs its own float conversions, because `struct` has none.** `BrainFloat`
subclasses `Dtype` and overrides `as_float` and `as_bits` alone: `struct` offers
`<e` for `f16` and nothing for `bfloat16`, so the conversion passes through the
`f32` carrier this format is the high half of, with one round-to-nearest-even on
the discarded 16 bits and an explicit refusal for a value that rounds past the
largest finite `bfloat16`. Its `struct_format` field therefore names the carrier
rather than a packing of its own width.

**A recorded pattern is rendered at its own dtype's width** — four hex digits for
`f16` and `bf16`, eight for `f32`, in `case.*.results`, in a divergent
`comparison.*` row, in the dispatch host's `result=` lines, and in the operand
groups the host is given. The manifest states each entry's dtype rather than
letting the host infer it, because the width decides the buffer size, the operand
vector, the sentinel, and the print width, and a host that guessed would read a
correctly dispatched `f16` kernel back as half as many `f32` values. **Two dtypes
now share a width**, so the digit count identifies neither format and only the
kernel name in the case key does.

**What the second dtype found.** `f16` arithmetic **preserves** the subnormals
`f32` arithmetic flushes, on this row, under every math mode, on both compilation
paths, with an execution witness reporting `executed` — from modules that declare
`air.compile.denorms_disable` exactly as the `f32` ones do. So the module-level
declaration does not summarize what the hardware does per dtype, and a subnormal
flush declared once without a dtype is false for one of the two. The guard is
what keeps that result honest: in `f16` the admissible verdict is `preserved`,
which is the same word the trap kernel produces when nothing ran, so the gate
holds the `f16` trap to the refusal and the `f16` flush kernels to their
witnesses in the same run.

**What the third dtype found, and why it was the one worth measuring.** `bf16`
arithmetic **flushes**, matching `f32` and not `f16`, with an execution witness
on every verdict. That is not a third data point; it is the one that separates
the two mechanisms the `f16` result left open. `f16` subnormals are all `f32`
**normals**, so "narrow arithmetic is evaluated at `f32` precision" predicts `f16`
preserving for free; `bf16` carries `f32`'s exponent field, so every `bf16`
subnormal *is* an `f32` subnormal and the same hypothesis predicts it flushing.
It does. The competing hypothesis — that the hardware honours subnormals natively
in narrow formats — predicted preservation and is refuted. In `bf16` the
admissible verdict and the trap's unguarded reading are different words again, as
in `f32`, so the risk runs the other way and the gate additionally holds the
`bf16` trap kernel under `safe` to *admission*: a guard that had started refusing
everything would make the result unreachable rather than wrong.

**A device can compile a module and then refuse to run it.** The iOS Simulator
compiles and links every `bf16` module on this row and then fails pipeline
creation with `XPC_ERROR_CONNECTION_INTERRUPTED`, on both compilation paths.
`bfloat_support` asks each family once,
before any measured case is dispatched, and a family that refuses has its `bf16`
cases left out of that family's manifest and recorded with the exact diagnostic
in `case.*.refusal` and `environment.family.*.device_bfloat_support`. This is a
capability probe and not a fallback: a family that *accepts* the probe and then
fails a real case is still a hard `ProbeFailure`. `Verdict.DEVICE_REFUSED_DTYPE`
and `Verdict.NO_DEVICE_OBSERVATION` stay separate members, because "a device that
answered no" and "no device to ask" are different measurements and both would
otherwise be a missing `results` row.

**Whether that refusal is about the type or the arithmetic is a separate probe.**
`bfloat_support` asks with `multiply_two_bf16`, which cannot distinguish them, so
`bfloat_dispatch_probe.py` asks the arithmetic-free `materialize_bf16` directly —
it is refused too, so the refusal is about the format. That probe is deliberately
outside the gate: the refusal path costs minutes of `XPC` retries per case, and
the gate already carries what it needs on every run. Its control is what makes it
evidence rather than a guess, and it runs on **both** sides of the `bfloat`
attempts, because a refusal seen after earlier faults could be the simulator's
compiler service degrading instead:

```sh
uv run python spikes/apple-targets/bfloat_dispatch_probe.py
```

The measured order and outcome: `materialize_f16` dispatched, `materialize_bf16`
refused, `multiply_one_bf16` refused, `materialize_f16` dispatched. The probe
exits nonzero when the trailing control fails, because such a run establishes
nothing about `bfloat` and must be discarded rather than reported.

**A kernel added here must be checked against what the module emitted.** Widening
the vocabulary found the harness reporting a kernel whose whole body is one
`fma` as containing no floating-point arithmetic at all: this front end lowers
`fma(x, a, b)` to `@air.fma.f32` and `FUSED_INTRINSIC` named only the `llvm.`
spellings. The verdict still failed closed, but the count was wrong in the
direction a reader acts on, because a surviving operation reported as zero looks
exactly like a deleted one. Both spellings are matched now, and a portable test
pins the parse over a module fragment carrying `fmul half`, `fdiv half`,
`@air.fma.f16`, `@air.fma.f32`, `fmul`/`fdiv`/`fadd bfloat`, the `fpext` and
`fptrunc` around a `bfloat` fused call that must not count, an `fcmp` that must
not count, and the `call` to the generated canonicalization helper that appears
at `-O0` and is not arithmetic. Widening the dtype puts the same question again
in a new spelling, so check it against real emitted modules at both `-O0` and
`-O2` before trusting any count a new dtype gives you. **Every line in that
fragment is now copied from a module this toolchain actually emitted**, which the
`bf16` pass had to correct: the helper `call` had been pinned in an *unmangled*
spelling the front end never produces. It named no fused intrinsic either way so
no count was ever wrong, but a reader checking the recognizer against a real
module would have found the pinned line absent from it. The helper is a
file-local C++ function mangled with its parameter type, so the three dtypes
spell it `_ZL35tiler_canonicalize_nan_f32_7fc00000f`,
`_ZL31tiler_canonicalize_nan_f16_7e00Dh`, and
`_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b`. Every hand-written
`SubnormalProbe`, `OrderProbe`, and `Witness` is also checked against `evaluate`,
which derives each candidate result from the kernel under exact arithmetic and
under the sign-preserving flush at that kernel's own dtype, so a mis-stated
literal is a portable test failure rather than a silently wrong classification.

Print a run and rewrite the retained record with:

```sh
uv run python spikes/apple-targets/numerical_probe.py \
  --record spikes/apple-targets/results/<yyyy-mm-dd>-numerics-covering-<toolchain>/record.tsv

TILER_APPLE_NUMERICS_EXHAUSTIVE=1 \
  uv run python spikes/apple-targets/numerical_probe.py \
  --record spikes/apple-targets/results/<yyyy-mm-dd>-numerics-exhaustive-<toolchain>/record.tsv
```

Pass `--work-dir spikes/apple-targets/local-work` to keep the generated
sources, IR, AIR, and libraries for inspection; that directory is ignored.

The assertions live in `test_numerical_probe.py`. Nothing runs them for you —
re-establish a finding by running them yourself before citing it:

```sh
uv run --with pytest pytest spikes/apple-targets
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

**What it costs the gate, in case counts, which are the only exact figures here.**
The covering matrix the gate runs is **402 offline cases** across three families
and **368 runtime cases** across the two families that dispatch; the exhaustive
matrix is **663 offline cases**. The third dtype accounts for 90 of those offline
cases (30 per family) and 96 of the runtime ones (48 per dispatching family), and
171 of the exhaustive offline cases. Of the 368 runtime cases, **320 are actually
dispatched**: the iOS Simulator's 48 `bf16` runtime cases are refused before
dispatch and recorded as such, and so are its 30 offline `bf16` cases on the
device side, whose compile side still runs. The figures the second dtype left
were 312 offline, 272 runtime, and 492 exhaustive offline.

**Wall-clock figures are deliberately not carried forward.** Every measurement of
this suite has been taken with a different number of other worktrees running
their own gates concurrently, and that load dominates the difference — the last
widening measured *faster* than the narrower matrix it replaced, which is
information about the host and not about the change. A misleading speedup is
worse than no number, so the case counts above are the claim. For orientation
only, and not as a comparison against any earlier figure: the full suite
completed in roughly 33 s on the measured host with a simulator already booted.
The one-time cost of booting a cold simulator adds roughly 8 s to the first gate
run that needs it; the harness leaves the device booted so subsequent runs pay
only one `simctl spawn` per family. On a host with no Apple toolchain the whole
thing skips in well under a second.

The retained 2026-07-25 runs are
[`results/2026-07-27-numerics-covering-xcode26.6-metal32023.883/record.tsv`](results/2026-07-27-numerics-covering-xcode26.6-metal32023.883/record.tsv)
and
[`results/2026-07-27-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv`](results/2026-07-27-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv),
schema `tiler.apple-numerical-behaviour/v6`. Both were rewritten in place when the
third dtype landed, on the identical host, toolchain, and date row they already
named, and every `case.*`, `comparison.*`, and `hazard.*` row they carried before
reproduced unchanged. The exact check, which is the evidence that widening the
dtype did not change what the harness asks about the dtypes already measured:
of the 1921 such rows in the covering record and the 2401 in the exhaustive one,
**0 disappeared and 0 changed**, and every one of the 480 and 696 rows added
names a `bf16` kernel; independently, the generated MSL of all 23 pre-existing
kernels is **byte-identical** to what the base commit's harness produced. The
only non-`case`/`comparison`/`hazard` rows that moved are the schema, the two
digests, the repository revision, the timestamp, `probe.dtypes`, and the new
`probe.operands.bf16` and `environment.family.*.device_bfloat_support`. The schema `v3` record
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
