---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets"
kind: "experiment"
title: "Apple Metal target compatibility and numerical spikes"
topics: ["apple-targets", "metal", "compatibility", "numerics", "subnormals", "reassociation", "runtime-compilation", "quantization"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.apple-targets.compatibility", "tiler.research.apple-targets.numerical-behaviour", "tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger", "tiler.research.reference.permitted-divergence-oracle"]
entrypoints: ["spikes/apple-targets/compatibility_probe.sh", "spikes/apple-targets/runtime_failure_probe.swift", "spikes/apple-targets/validate_compatibility_record.py", "spikes/apple-targets/replay_retained_compatibility_record.sh", "spikes/apple-targets/validate_numerical_record.py", "spikes/apple-targets/test_probes.py", "spikes/apple-targets/numerical_probe.py", "spikes/apple-targets/numerical_probe_host.m", "spikes/apple-targets/test_numerical_probe.py", "spikes/apple-targets/bfloat_dispatch_probe.py", "spikes/apple-targets/aot-runtime-compiler-observer/run.sh", "spikes/apple-targets/code-domain-integer-decode/decode_probe.py", "spikes/apple-targets/code-domain-integer-decode/decode_probe_host.m", "spikes/apple-targets/code-domain-integer-decode/validate_decode_record.py", "spikes/apple-targets/code-domain-integer-decode/test_decode_probe.py", "spikes/apple-targets/contraction-pragma-runtime-probe/pragma_probe.py", "spikes/apple-targets/evaluation-order-probe/order_probe.py", "spikes/apple-targets/exp-at-zero-runtime-probe/probe.py"]
last_verified: "2026-08-11"
ticket: "apple-artifact-compatibility"
---

# Apple Metal target compatibility and numerical spikes

Seven independent probes share this directory. The
compatibility probe answers which artifact families and deployment minima
produce which bytes. The numerical probe answers what Apple GPU scalar
arithmetic actually does to subnormals, signed zero, and contraction — and, since
the dtype axis was added, that the answer is not the same for `f32` and `f16`.
The AOT runtime-compiler observer asks whether native metallib and pipeline
preparation exposes an attributable compiler build without source JIT. The
code-domain integer decode probe asks whether the emitted MSL for the registered
strict-affine `u8` decode computes the contract's value over its complete finite
code domain. The contraction-pragma runtime probe asks whether a source-level
contraction pragma survives to the runtime compiler — the one question the
numerical probe's byte-identical pairing structurally forbids it from asking.
The evaluation-order probe asks whether an emitted floating-point evaluation
*order* survives either compiler, which is the property a plan's pinned reduction
grouping rests on. None downloads or installs a toolchain component.

The runtime exponential-at-zero probe asks the narrow exact-bit question the tree-fold online-softmax bound left open: what `precise::exp` returns for buffer-supplied positive and negative zero on the current Apple9 execution row.

**Five of the seven share one build-tool row; the evaluation-order and runtime exponential-at-zero probes use its successor.** They
were measured after this host's `xcode-select` moved to Xcode 27.0 and an offline
`metalfe-32023.921`, where every other retained record here names Xcode 26.6 and
an offline `metalfe-32023.883`. Its rows and theirs are not rows of one table,
and a difference between them is not evidence of drift until one is re-run on the
other's toolchain.

## Runtime exponential at signed zero

The [sibling harness](exp-at-zero-runtime-probe/README.md) reuses `numerical_probe_host.m` to runtime-compile one two-lane F32 kernel with the production emitter's `precise::exp` spelling and explicit `math=safe,fpfun=precise,lang=4.0,opt=default`. Its 2026-08-11 retained record binds the one authoritative `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` hardware row, the current host build toolchain, and the separately identified OS runtime compiler. On an Apple M4 Max reporting Apple9, both `00000000` and `80000000` returned `3f800000`, exactly binary32 `1.0`.

This is not an offline-compiler replay. The production AOT profile still names Xcode 26.6 and `metalfe-32023.883`; the runtime source route measured here identifies `metalfe-32023.921` from compiler text recovered after the producer scanned a serialized binary archive and before atomic publication. The raw archive is not retained, so neither archive replay nor transfer of the bits across compilers is claimed. Kernel, input, result, compiler-source-label, and Xcode-version perturbations were each watched failing against the retained producer evidence.

## Evaluation-order probe

The [sibling harness](evaluation-order-probe/README.md) dispatches a
four-contributor add chain whose written order and a legal alternative order
differ by one ULP — the seed [the permitted-divergence
oracle](../../docs/research/reference/permitted-divergence-oracle.md)'s Part 6
works through — at every `-fmetal-math-mode` × `-ffp-contract` × optimization
level the offline driver accepts and every `mathMode` × `MTLLibraryOptimizationLevel`
pair `MTLCompileOptions` exposes, 72 cases in one host invocation. Its 2026-08-06
retained record finds a written two-by-two split `(a+b)+(c+d)` **re-emitted as a
left-deep chain** and returning the serial value under `relaxed` and `fast` — two
offline cells and four runtime ones — while every `safe` cell on both paths
returns the order its own source names. The offline half reads the rewritten add
tree out of the emitted LLVM IR, so the change is attributable to the front end
rather than to the AIR-to-ISA stage below it.

**Its separation is what makes it evidence.** The fold kernels contain adds and
nothing else, measured rather than argued: their emitted operation list is
`fadd;fadd;fadd` in all 36 offline cases, so contraction has no pair to act on.
An `a*b+c` control is dispatched in the same run and the same matrix and fuses in
10 of its 24 cases, so the contraction axis is demonstrably live beside them; if
it fused nowhere the producer publishes nothing, and that refusal was watched
firing. An opcode count could not have found the reordering at all — three adds
rearranged are still three adds — which is why the emitted tree is a separate
reading.

## Contraction-pragma runtime probe

The [sibling harness](contraction-pragma-runtime-probe/README.md) takes the very
`contraction_pair`, `contraction_pair_f16`, and `contraction_pair_bf16` sources
this directory's numerical probe generates, inserts one 31-byte
`#pragma METAL fp contract(off)` line at file scope and changes nothing else, and
puts both sources through `newLibraryWithSource:options:` in one host invocation
per cell. Its 2026-08-02 retained record finds the pragma unfusing the pair in
**all twelve** cells of three widths × `{Relaxed, Fast}` × `{Default, Size}` on
runtime compiler `metalfe-32023.921`, with the unperturbed neighbour still
returning finding 30's fused value in every one and every case carrying an
executed witness. So an unfused contract is honourable on this row by *emitting
different source* — ADR 0076's `SupportedWithExactEmulation` shape — and not by
any `MTLCompileOptions` setting, which still has none.

**It is a sibling rather than a variant kernel here for a structural reason, not
a cost one.** The contraction comparison above *is* byte-identical source through
two compilers; a pragma variant is by construction the one perturbation that
comparison forbids. It reuses this directory's kernel definitions, candidate
derivation, and dispatch host **unmodified** — modifying the host would move
`probe.host_source_sha256` in all four retained numerical records — and shares
nothing else.

**Its control is its failure proof.** A run in which the unperturbed neighbour
does not fuse establishes nothing about the pragma, so the harness refuses the
whole run and publishes nothing unless all twelve controls fuse; `--perturb-control`
applies the pragma to the control too and was watched producing exactly that
refusal.

## Code-domain integer decode probe

The [sibling harness](code-domain-integer-decode/README.md) measures the strict-affine `u8` decode's integer machinery — a `uchar` read, a widening to `int`, an `int` subtraction, an `int`-to-`float` conversion, and a multiply by an `f32` scale — over the complete 256 × 256 code and zero-point grid against an exact rational reference rounded once to `binary32`. Its 2026-07-31 retained record found every cell of every normal-scale case bit-identical to that reference on both compilation paths, the subnormal scales flushing on the *input* rather than the result, and `+0.0` for the whole code-equals-zero-point diagonal.

**It is a sibling rather than a dtype on the numerical probe, and the trade was priced rather than guessed.** The kernel table above is shared by every profile, so a new kernel family moves `probe.harness_sha256` in all four retained records — the 2026-07-31 permutation landing measured that cost — for a question none of them asks; and a 65,536-cell population cannot be a `case.*.results` row. The numerical probe's verdict vocabulary classifies a *subnormal observation*, where this one classifies agreement with a computed reference over a population. The two harnesses share the host row, the profile identity, the atomic-publication and retained-manifest conventions, and nothing else.

**Its guard is the kernel rather than a witness.** Every operand of the arithmetic under test arrives in a buffer, so no stage of either compiler can fold it — which is precisely what the two-layer guard above exists to catch on kernels whose operands are immediates. Its emitted-operation recognizer nonetheless matches *every* named call rather than an expected intrinsic set, because this front end lowers `float(int)` to `air.convert.f.f32.s.i32` and not `sitofp`: naming the LLVM conversion opcodes would have reported the conversion stage absent from every module, which is the `air.fma.f32` retraction met again in a new spelling.

## Native-AOT runtime-compiler observer

The [preserved observer](aot-runtime-compiler-observer/README.md) takes only an offline-produced metallib through native library, function, and compute-pipeline preparation. It records compiler-related dyld membership before and after every stage while keeping disk presence, loaded-image membership, scan availability, embedded strings, and attributable compiler identity separate. Its 2026-07-31 retained result found two plausible GPUCompiler library images already loaded before the route and no population change; both direct dyld-cache file scans were unavailable. A separately compiled binary carrying a real source-JIT call proves the selector check can reject it, and an unrelated synthetic GPUCompiler image proves plausible membership and a build string do not satisfy attribution. Exact native translator/compiler identity is therefore unavailable on this bounded AOT route and remains `Unknown`; [ADR 0086](../../docs/decisions/0086-require-attributable-or-attested-native-translation.md) is the accepted applicability authority and requires attributable identity or exact host attestation before any positive receipt.

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
Schema v2 binds the repository base and the exact harness, validator, kernel,
and manifest digests. AIR and metallib files are regenerable and ignored in the
checked-in result area; their digests remain in the record.

**The v2 producer set is not one fixed list, and a replay must select the one its record names.** A record produced now binds three inputs — harness, kernel, validator — because `e197176` deleted the root `pyproject.toml` and `uv.lock` with the rest of the Python tooling, so the harness stopped hashing them and the current validator expects a three-row manifest. The retained 2026-07-21 record binds five, correctly: it was measured while those two files existed. Running the current validator against it fails with `retained input manifest does not match producer fields`, which is a shape difference between two producer generations and not a defect in either.

The retained 2026-07-21 local run is
[`results/2026-07-21-xcode26.6-metal32023.883/record.tsv`](results/2026-07-21-xcode26.6-metal32023.883/record.tsv).
Its SDK extracts and command logs are checked in beside it, and so is the exact producer set it names, under `producers/` laid out by the repository-relative paths its own `input-manifest.tsv` uses. Those five files were recovered from commit `b0fdba7` and each was verified against the record's digest before being retained. `b0fdba7` is the only commit in either file's complete history carrying the recorded harness `b37ba8…` and validator `63f579…` bytes; the exhaustive check is `git log --format=%H -- <path> | while read -r c; do git show "${c}:<path>" | shasum -a 256; done` over all six commits that touched the harness and all four that touched the validator. The record's own `probe.repository_base_revision` is `f3004a1`, whose committed harness and validator digests are `1425b6…` and `05f699…` — different bytes. So the run was taken from a working tree based on `f3004a1` with the producer edits still uncommitted, and those edits landed one commit later; the recorded base revision names where the tree came from and is not by itself enough to recover what ran. Retaining the bytes is what closes that gap.

Replay the retained record from its own producer set, with no repository Python environment and no current-tree validator:

```sh
spikes/apple-targets/replay_retained_compatibility_record.sh \
  spikes/apple-targets/results/2026-07-21-xcode26.6-metal32023.883
```

The replay verifies the manifest digest, then every retained producer byte against both the manifest row and the record's own producer field, and only then executes the retained validator on the record — so an edited retained validator is rejected before it can certify its own changed identity. It also counts the files under `producers/` against the manifest population, because an unlisted retained file would be bytes no digest covers. Retained producers are kept as data rather than as executables: their SHA-256 is the identity, the retained harness is not a live entrypoint, and the replay runs the retained validator through an explicit interpreter (`python3`, overridable with `TILER_REPLAY_PYTHON`). Both retained validators are standard-library-only, which is what makes this self-contained.

Every rejection named here was observed by perturbing the retained tree, watching the failure, and restoring, and each reports exactly one reason — the right one. Appending a comment to the retained validator, and separately to the retained harness, fails at the producer-digest step with the recorded and found digests named, before the validator runs. Zeroing the record's `probe.validator_sha256` fails the record-to-bytes cross-check. Replacing a record digest with `short` fails the SHA-256 shape check, duplicating a record key fails `record does not carry exactly one probe.source_sha256 row`, deleting `producers/uv.lock` fails as a missing producer, and an extra empty file under `producers/` fails the population count with `producers/ holds 6 files but the manifest names 5`.

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

**`device_registry_id` correlates a GPU inside one run and identifies nothing between runs.** macOS SDK 26.5's `MTLDevice.h` documents `registryID` as "the IORegistry ID for the Metal device", "global to all tasks", and usable "to identify the GPU across task boundaries" — a correlation handle in a live environment, and nothing more. The retained records measure the limit directly: the same named `Apple M4 Max`, on the same OS build, reports `4294968621` in the 2026-07-24 and 2026-07-25 records and `4294968452` in the 2026-07-27 and 2026-07-30 ones, with no recorded cause. So the invariant is macOS/simulator equality *within* one measurement — the evidence that the simulator dispatches on the host GPU — and never persistence of the number. `test_the_registry_id_agrees_within_a_measurement_and_is_free_between_them` enumerates and counts the thirteen retained records carrying a registry-ID row, holds the seven paired ones to that equality at their own value, holds the six macOS-only named-profile records to having no simulator row, and asserts positively that the values disagree between measurements, so no raw row can be rewritten into agreement to quiet it. (The counts are read from the enumeration rather than remembered; they last drifted because the figures here were not updated when records joined.) A record added to `results/` without joining that enumeration fails the check rather than escaping it. Nothing in this repository may key artifact identity, profile identity, or host applicability on the value; [ADR 0086](../../docs/decisions/0086-require-attributable-or-attested-native-translation.md) eliminates it by name.

**The operation vocabulary, and the two matrices it is measured in.** The kernels
cover multiply, add — including one bare add whose subnormal operand comes
straight from the buffer rather than out of a preceding multiply — division in
both the power-of-two form the driver rewrites into a multiply and the form it
keeps, a source-level `fma`, a two-add chain whose value says where the
parentheses went, and a three-add pair whose value says whether the contributors
were reordered. The swept axes are the three math
modes, the three contraction settings, both `-fmetal-math-fp32-functions`
values, and all five offline optimization levels. That costs more than the gate
should pay on every run, so `cases` assembles a `covering` set — at least one
case of every kernel, mode, level, contraction setting, and fp32-functions
value, plus every case a finding cites — and an `exhaustive` cross product
selected by `TILER_APPLE_NUMERICS_EXHAUSTIVE`. `probe.matrix` names which one
produced a record and `matrix_mismatch` refuses to compare one against a run of
the other. A portable guard test holds the covering set to its coverage claim.

**The last two kernels measure two different licences, which is why there are
two.** ADR 0014 keeps reassociation and contributor permutation as independent
permissions: the first moves the parentheses over a fixed leaf order, the second
moves the leaves. `reassociation_chain` measures the first. `permutation_chain`
and `permutation_chain_reordered` measure the second by carrying the *same three*
contributors — `2**30`, `2.0`, `-2**30` — in two orders and differing in nothing
else, so what separates their results is leaf order alone. The permuted value
`40000000` is what makes the pair a measurement of permutation rather than a
second reading of reassociation, and that is a finite property of the chosen
constants rather than a claim: four leaves admit exactly five full binary trees,
and `test_the_permutation_probe_is_unreachable_by_reassociating_the_canonical_order`
enumerates all five for every operand and holds the permuted value to being
absent from each. Both kernels witness on negative zero, the one non-subnormal
operand whose result survives the relaxed licence folding the cancelling pair
away; a witness anywhere else would report `disagrees` under `relaxed`, which
fails validation rather than publishing a witness that measures the licence under
test.

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
`SubnormalProbe`, `OrderProbe`, `PermutationProbe`, and `Witness` is also checked against `evaluate`,
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

The production-profile experiment is a separate named selection rather than a set of freely composable flags. From this directory, record the covering and exhaustive Apple9/F32 rows with:

```sh
TILER_REQUIRE_METAL_TOOLCHAIN=1 uv run python numerical_probe.py \
  --profile apple9-f32-unified-msl4-macos26 \
  --result-dir results/<yyyy-mm-dd>-numerics-covering-apple9-f32-unified-msl4-macos26-<toolchain>

TILER_REQUIRE_METAL_TOOLCHAIN=1 TILER_APPLE_NUMERICS_EXHAUSTIVE=1 \
  uv run python numerical_probe.py \
  --profile apple9-f32-unified-msl4-macos26 \
  --result-dir results/<yyyy-mm-dd>-numerics-exhaustive-apple9-f32-unified-msl4-macos26-<toolchain>
```

The `bf16` row is a **second named profile** rather than a widening of that one, recorded the same way:

```sh
TILER_REQUIRE_METAL_TOOLCHAIN=1 uv run python numerical_probe.py \
  --profile apple9-f32-bf16-unified-msl4-macos26 \
  --result-dir results/<yyyy-mm-dd>-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-<toolchain>

TILER_REQUIRE_METAL_TOOLCHAIN=1 TILER_APPLE_NUMERICS_EXHAUSTIVE=1 \
  uv run python numerical_probe.py \
  --profile apple9-f32-bf16-unified-msl4-macos26 \
  --result-dir results/<yyyy-mm-dd>-numerics-exhaustive-apple9-f32-bf16-unified-msl4-macos26-<toolchain>
```

**Why a neighbour and not a wider first profile.** `probe.harness_sha256` covers this whole harness file, so widening the F32 profile's case set would oblige a re-run whose record no longer carries the digest the target-profile authority ledger and its dependants pin. A neighbouring profile leaves all four retained F32 records byte-identical. **It carries F32 beside BF16 for two reasons, one structural and one evidential.** `archive_support` dispatches the F32 kernel `multiply_two` and the dispatch host rejects a manifest entry whose dtype has no operand group, so a BF16-only profile would have had to change what that probe measures for *every* profile; and the duplicated F32 rows are a control — measured through the identical target, language, device, and both compilers, their agreement with the neighbouring profile is what attributes the BF16 rows to the compilation the profile names. On the 2026-08-02 pair all 864 covering and 996 exhaustive F32 `case.*`/`comparison.*` rows are identical to the 2026-07-31 F32 record.

Either profile always selects `-target air64-apple-macos26.0` and `-std=metal4.0` offline, explicit `MTLLanguageVersion4_0` at runtime, the macOS artifact family, and an Apple9 device; they differ only in dtype coverage, F32 alone against F32 and BF16. The retained BF16 rows are [`results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883`](results/2026-08-02-numerics-covering-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv) and its `-exhaustive-` neighbour, and they are what the authoritative Metal compile profile may transcribe a macOS BF16 row from: `environment.family.macos.device_bfloat_support` is `supported` under this compilation, and the flush dimensions carry `executed` witnesses with `materialize_bf16` returning all eight operands unchanged. The [numerical-behaviour report](../../docs/research/apple-targets/numerical-behaviour.md) states which BF16 claims that row supports and which stay `Unknown`. `MTLCompileOptions` has no deployment-target property, so the runtime half is qualified by the exact host OS and device rather than falsely claiming it received the offline target flag. A missing macOS toolchain, rejected MSL version, non-Apple9 device, failed compile/link/pipeline/command buffer, missing execution witness, path divergence, or invalid retained input is a nonzero refusal and publishes no result directory.

Each successful result directory contains `record.tsv`, `input-manifest.tsv`, and one canonical `sources/*.metal` file per unique kernel the selected profile measures — seventeen for the F32 profile, twenty-eight for the F32+BF16 one. The producer hashes itself, the dispatch host, the validator, the manifest, and every retained source, validates the staged directory, and publishes it atomically. AIR, LLVM IR, metallibs, binary archives, and the host executable remain regenerable scratch products and are not retained. Validate a published row with `uv run python validate_numerical_record.py results/<result>/record.tsv`.

**A record is validated by the producers it pins, and the current tree is only one generation of those.** The validator digests the working tree's harness, dispatch host, and itself against the record's `probe.harness_sha256`, `probe.host_source_sha256`, and `probe.validator_sha256`, so any later producer edit makes the current tree refuse an earlier record with `validator digest mismatch` or `numerical harness digest mismatch` — a producer-generation difference, not decay, and the same shape the compatibility probe's v2 producer set documents above. The four `apple9-f32-unified-msl4-macos26` records are in that position since the BF16 profile landed. Recovering the two blobs into a scratch directory does not work — the validator digests `numerical_probe_host.m` beside itself, resolves the repository from its own location, and the retained results are committed after the producer commit. Use a detached worktree at the record's own revision, which differs per record:

```sh
REC=results/2026-07-31-numerics-covering-apple9-f32-unified-msl4-macos26-xcode26.6-metal32023.883/record.tsv
REV=$(awk -F'\t' '$1=="probe.repository_base_revision"{printf "%s", $2; exit}' "$REC")
git worktree add --detach /tmp/tiler-revalidate "$REV"
python3 /tmp/tiler-revalidate/spikes/apple-targets/validate_numerical_record.py "$PWD/$REC"
git worktree remove /tmp/tiler-revalidate
```

Both 2026-07-31 records exit 0 through it. `validate_revision_identity` is what makes the recovery sound rather than circular: it holds the recorded revision to resolving producer blobs whose digests are the ones the record already carries.

The validator derives the complete case, source, execution-witness, runtime-option, and comparison populations from the committed producer definitions. It rejects missing or extra rows, noncanonical source bytes, malformed or falsely reported witness values, path divergence, and a producer revision that cannot resolve to the recorded harness, host, and validator digests. A derived `not-executed` or `none` witness is retained as the reason that case is inadmissible rather than rejected or turned into evidence.

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
like instead of reading as a divergence. The contraction gap is the one of the
three that has a measured source-level substitute, and measuring it is what the
sibling probe above exists for; `-target` and `-O0` have none and none was
attempted.

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
Counted from the retained records rather than from memory, because the previous
figures in this paragraph had drifted below the matrix they described. The
covering matrix the gate runs is **579 offline cases** across three families
(193 per family) and **484 runtime cases** across the two families that dispatch
(242 per dispatching family); the exhaustive matrix is **903 offline cases**
(301 per family). The third dtype accounts for 153 of those offline cases (51 per
family) and 132 of the runtime ones (66 per dispatching family), and 261 of the
exhaustive offline cases (87 per family). Of the 484 runtime cases, **418 are
actually dispatched**: the iOS Simulator's 66 `bf16` runtime cases are refused
before dispatch and recorded as such, and so are its 51 offline `bf16` cases on
the device side, whose compile side still runs.

Reproduce each figure against the retained covering record:

```sh
cd spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883
grep -c '\.compile_options' record.tsv                      # 579 offline cases
grep -c '\.applied_options' record.tsv                      # 418 runtime cases dispatched
grep '\.refusal' record.tsv | grep -c '\.runtime\.'         # 66 runtime refusals
grep '\.refusal' record.tsv | grep -vc '\.runtime\.'        # 51 offline refusals
grep '\.compile_options' record.tsv | grep -c bf16          # 153 bf16 offline cases
```

418 dispatched plus 66 refused is the 484 above; the exhaustive figures come from
the same two commands against the `-exhaustive-` record beside it.

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

The retained runs are
[`results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv`](results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv)
and
[`results/2026-07-31-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv`](results/2026-07-31-numerics-exhaustive-xcode26.6-metal32023.883/record.tsv),
schema `tiler.apple-numerical-behaviour/v6`. They replace the 2026-07-27 pair,
which is retained beside them, and the reason for the new run is that the
permutation pair widened the kernel table and so moved the harness digest every
record carries. The same check applies as when the third dtype landed, and is the
evidence that widening the question did not change any answer: against the
2026-07-27 records, **0 `case.*`, `comparison.*`, or `hazard.*` rows disappeared
and 0 changed** across 3,215 and 4,079 pre-existing rows, and every one of the 144
rows added to each names a `permutation_chain` kernel. The only other rows that
moved are the timestamp, the repository revision, and two producer digests — and
`probe.host_source_sha256` moved because the 2026-07-27 records still named a
dispatch host that commit `7559e87` had already replaced, so regenerating restored
an agreement that had lapsed rather than breaking one. The schema `v3` record
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
