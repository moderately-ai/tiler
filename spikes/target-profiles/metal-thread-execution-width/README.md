---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.metal-thread-execution-width"
kind: "experiment"
title: "Whether Metal threadExecutionWidth stays equal across a predeclared pipeline population"
topics: ["target-profiles", "metal", "apple-targets", "subgroup", "feasibility"]
experiment_status: "planned"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger", "tiler.research.scheduling.subgroup-execution-tier"]
entrypoints: ["spikes/target-profiles/metal-thread-execution-width/src/main.rs"]
last_verified: "2026-08-13"
ticket: "measure-metal-thread-execution-width-across-prepared-pipelines"
---

# Whether Metal threadExecutionWidth stays equal across a predeclared pipeline population

ADR 0094 decision 7 keeps subgroup width as a literal in the schedule, an equality against an atomic compile-profile subject, and a confirmation against the prepared pipeline. [The subgroup execution tier](../../../docs/research/scheduling/subgroup-execution-tier.md) deferred the question of whether `threadExecutionWidth` is knowable earlier than `PreparedKernelPreflight` to a bounded experiment that reads the property across several pipelines on one device. This spike is that experiment.

It answers the question only for the frozen population below, on the authorized Apple M3 Pro Apple9 host, under the first macOS Metal compile profile's offline flag set. It does not source the M4 Max qualified numerical, grid-axis, or dispatchability rows, and even perfect equality does not remove the prepared-pipeline confirmation.

## Frozen before any submission

The matrix, compilation-selection identities, descriptor shapes, source/ABI/oracle subjects, metric, repetitions, environment, custody, and stop conditions are written here and in the ticket, and encoded as `PIPELINES` in [`src/population.rs`](src/population.rs), before the first `newComputePipelineState`. The array is not edited after a width is read.

### Device / profile

- **Execution host:** `ssh m3`. Apple M3 Pro, macOS 27.0 build `26A5388g`, `arm64`, `supportsFamily(Apple9)`. Not `Apple M4 Max`.
- **Offline compilation:** `xcrun --sdk macosx metal` then `metallib`. ADR 0086 excludes `newLibraryWithSource:`.
- **Profile-strict flags:** `-std=metal4.0 -target air64-apple-macos26.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`.

Observed on `m3` before the run, and recorded again by the harness rather than trusted from this paragraph: `Xcode 26.6` build `17F113`, `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`, macOS SDK `26.5` build `25F70`. Those match the authority ledger's offline compilation table. The device name does not.

### Metric, repetitions, oracle

- **Metric:** `MTLComputePipelineState.threadExecutionWidth`.
- **Corroborating prepared facts, not substitutes:** `maxTotalThreadsPerThreadgroup`, `staticThreadgroupMemoryLength`.
- **Repetitions:** three independent pipeline constructions per identity. Every repetition is retained.
- **Oracle:** none. The property is read. Equality or variation is computed from the retained set.

### Source / ABI / compilation-selection subjects

- **Source:** one file under [`kernels/`](kernels/) per kernel, compiled in isolation.
- **ABI:** the entry point plus one of `default`, `max_1`, `max_32`, `max_256`, `max_1024`, `multiple_of_width`, `max_1024_multiple`.
- **Compilation selections:** `profile_strict`, `math_fast`, `math_relaxed`, `contract_fast`, `opt_O0`, `opt_Os`, `std_metal3.1`, each a named flag vector in `src/population.rs`.

### Authorized families and controls

The Apple9 compile profile could later declare `SubgroupRealizationSubject { width, arithmetic, transfer: InRangeXorShuffle }` as `Realized` only for arithmetic types it already marks `Dispatchable`: F32 and BF16. F32 shuffle is in MSL Table 6.14. BF16 shuffle is not. F16 and F64 are profile-silent.

The frozen population therefore includes the F32 XOR-shuffle butterfly at the profile-strict selection (required), the BF16 XOR-shuffle candidate (optional compile), and negative or isolating controls that vary operation family (`store`, elementwise add, `simd_sum`, `simd_shuffle_down`, `quad_shuffle_xor`, threadgroup memory, high live-register count), arithmetic type (`f16`, `bf16`, `f32`, `f64`, `i32`, `u32`), control flow (uniform, divergent, loop), threadgroup shape (descriptor max 1/32/256/1024, `threadGroupSizeIsMultipleOfThreadExecutionWidth`, source `[[threads_per_threadgroup(8,8,1)]]`, source `[[max_total_threads_per_threadgroup(32)]]`), and compiler selection.

The 34 identities are the table on the ticket and the `PIPELINES` array. `PIPELINES.len()` is the count; a test refuses a kernel file that is not named there and a named kernel that has no file.

### Environment, custody, stops

The harness records the answering toolchain, rustc, device name, `registryID`, Apple9 support, `maxBufferLength`, load averages, and `sw_vers`. It does not change Xcode, SDK, OS, Rust, or device state.

Custody is SHA-256 of every kernel, of concatenated `src/*.rs` in name order, of `Cargo.lock`, and of the running executable. Validation recomputes the source and lock digests from this tree. The executable digest is a recorded fact from the measuring host.

The run aborts, with no equality claim, if there is no Apple9 default device or if a required pipeline fails to compile or prepare. Optional failures are retained rows.

## Running it

From this directory, on `m3` only for the width observations:

```sh
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release -- measure > results/<date>-<host>/widths.json
DEVELOPER_DIR=/Applications/Xcode.app cargo test
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release -- validate results/<date>-<host>/widths.json
```

`DEVELOPER_DIR` selects the offline toolchain for the invocation and mutates no host state. The harness records whatever toolchain answered.

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

The coordination host is an M4 Max on a different OS build (`26A5406e`). `cargo test` may run there because it does not read `threadExecutionWidth`. `cargo run -- measure` must not.

## Result

Not yet run. This section is filled after the m3 submission. The freeze above does not change when the numbers arrive.

## Boundary

- One host, one offline toolchain row, one frozen population.
- No portable Apple9 claim, no M4 Max claim, no family-table width.
- No performance claim.
- Equality, if observed, still leaves ADR 0094's prepared-pipeline confirmation in place.
- Variation, if observed, forbids a single compile-profile width row for this population.
