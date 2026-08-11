---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets.exp-at-zero-runtime"
kind: "experiment"
title: "Metal runtime precise exponential at signed zero"
topics: ["apple-targets", "metal", "numerics", "transcendentals", "runtime-compilation", "softmax"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.apple-targets.numerical-behaviour", "tiler.research.numerics.tree-fold-online-softmax-bound"]
entrypoints: ["spikes/apple-targets/exp-at-zero-runtime-probe/probe.py", "spikes/apple-targets/exp-at-zero-runtime-probe/exp_at_zero.metal"]
last_verified: "2026-08-11"
ticket: "measure-whether-a-targets-exponential-is-exact-at-zero"
---

# Metal runtime precise exponential at signed zero

This bounded experiment answers one exact-bit question for the current authoritative macOS Apple9 target row: what does a runtime-compiled device kernel return for `precise::exp(+0.0f)` and `precise::exp(-0.0f)`?

The kernel reads both signed-zero bit patterns from a buffer, applies the exact `precise::exp` spelling the production Metal emitter uses, and writes the results to another buffer. The shared [`numerical_probe_host.m`](../numerical_probe_host.m) compiles that source with `math=safe,fpfun=precise,lang=4.0,opt=default`, creates a pipeline, dispatches both lanes, waits for the command buffer, requires terminal `Completed` status with no error, and only then reads the result bits. Reading the inputs from a buffer makes the kernel/input/result path observable instead of asking the compiler to fold a literal.

## Authority boundary

The one hardware row is `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, constructed by `BoundMetalCompileDeclaration::first_macos_apple9`. Its numerical vocabulary includes F32 and BF16, but the elementary function measured here is the governed F32 `precise::exp`; the target-neutral governed compiler profile is not a second hardware row.

This measurement is deliberately adjacent to, not a replacement for, that profile's offline compiler row. The production route is AOT and the profile names Xcode 26.6 / offline `metalfe-32023.883`; this experiment uses `MTLDevice.newLibraryWithSource` and records the current host build toolchain and the OS runtime compiler separately. Agreement at the measured inputs says what that runtime route delivered on this execution environment. It does not attribute the result to the production offline compiler and does not establish any unmeasured input or target.

## Run and validate

From the repository root:

```sh
python3 spikes/apple-targets/exp-at-zero-runtime-probe/probe.py \
  --result-dir spikes/apple-targets/exp-at-zero-runtime-probe/results/<dated-row>

python3 spikes/apple-targets/exp-at-zero-runtime-probe/probe.py \
  --validate spikes/apple-targets/exp-at-zero-runtime-probe/results/<dated-row>/record.tsv
```

The producer publishes atomically only after validating the record, manifest, invocation, raw host output, compiler text recovered from the producer-scanned runtime archive, producer digests, exact two-input population, applied compile options, current execution row, retained Xcode/SDK/clang/Metal build row, exact `serialized-MTLBinaryArchive` source label, Apple9 support, and the result link back to the host output. Tool paths are retained as nonempty provenance rather than fixed build-row authority. The raw archive is not retained, so this record supports recovery of the compiler text and does not support archive replay. It refuses a result directory that already exists. Nothing in `make` runs this device probe.

## Failure demonstration

The validator can say no at each load-bearing link without modifying the retained result:

```sh
python3 spikes/apple-targets/exp-at-zero-runtime-probe/probe.py \
  --demonstrate-failures spikes/apple-targets/exp-at-zero-runtime-probe/results/<dated-row>/record.tsv
```

This perturbs the kernel bytes, one recorded input, one recorded result, the runtime-compiler source label, and the Xcode version while leaving the corresponding retained producer evidence unchanged. Every perturbation must be rejected; changing an assertion would prove only that the assertion runs and is not the test performed here.

## Measurement boundary

One F32 function, two inputs, one source spelling, one compile-option selection, one OS runtime compiler, one macOS build, one Apple M4 Max reporting Apple9, and one dispatch shape. No CPU path or timing is measured. `exp` away from signed zero, the offline AOT compiler's delivered bits, iOS devices and simulators, other Apple GPU families, other dtypes, other math modes, and other compiler or OS builds remain unmeasured.

## Retained result

**Measurement — 2026-08-11.** [`record.tsv`](results/2026-08-11-macos-apple9-runtime-metal32023.921/record.tsv) reports `00000000 -> 3f800000` and `80000000 -> 3f800000`: both signed zeros return exactly binary32 `1.0`. The device is an Apple M4 Max reporting Apple9 on arm64 macOS 27.0 build 26A5388g. The host binary was built through Xcode 27.0 build 27A5228h, macOS SDK 27.0 build 26A5388f, and Apple clang 21.0.0; the compiled source went to the OS runtime compiler `Apple metal version 32023.921 (metalfe-32023.921)`, recovered from the producer-scanned serialized `MTLBinaryArchive` before atomic publication. The raw archive is not retained, so this is not an archive-replay claim. The record also carries the production profile's separate offline Xcode 26.6 / `metalfe-32023.883` row so the two are not conflated.

**Fact — all five subject perturbations were rejected.** Against that retained record the command above reports `kernel perturbation rejected: probe.kernel_sha256 mismatch`, `input perturbation rejected: measurement.input.1 mismatch`, `result perturbation rejected: measurement.result.0 mismatch`, `version-source perturbation rejected: environment.runtime_compiler.version_source mismatch`, and `xcode-version perturbation rejected: environment.xcode_version mismatch`, including the conflicting digest, exact bits, or row in each full diagnostic. The unperturbed validation returned zero before the perturbations.
