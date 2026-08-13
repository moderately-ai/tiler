---
id: measure-metal-thread-execution-width-across-prepared-pipelines
title: Measure Metal thread execution width across prepared pipelines
status: done
priority: p1
dependencies: []
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, decide-the-prepared-subgroup-width-equality-gate]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: [spikes/README.md]
tags: [measurement, metal, subgroup, target-profiles, evidence]
---
## User-visible outcome

The first standard Metal subgroup row is based on retained observations of the exact prepared-pipeline property it claims, with variation across pipeline shapes made visible rather than assumed away.

## Measurement question

On the authorized Apple M3 Pro Apple9 host, compiled with the first macOS Metal compile profile's offline flag set, does `MTLComputePipelineState.threadExecutionWidth` remain equal across a predeclared set of pipelines that vary operation family, arithmetic type, control flow, threadgroup shape, and relevant compiler selection?

## Fact audit at `4ef52cfee9b96e047b084deca09a239d3b606e68`

- **False as a host claim — "On the qualified Apple9/Xcode/SDK profile".** The first macOS Metal compile profile's execution environment is `Apple M4 Max` ([the authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md), table row `Device | Apple M4 Max`). This session authorizes observations on `Apple M3 Pro` via `ssh m3`. Both devices report Apple9; they are not one row. The measurement question is repaired above. The result does not source the M4 Max qualified numerical, grid-axis, or dispatchability rows.
- **Verified — the offline Xcode/SDK/metal builds on `m3` match the ledger's compilation table, observed rather than assumed.** `xcodebuild -version` → `Xcode 26.6` / `Build version 17F113`; `xcrun --sdk macosx metal --version` → `Apple metal version 32023.883 (metalfe-32023.883)`; `xcrun --sdk macosx metallib -version` → `AIR-LLD 32023.883 (metalfe-32023.883)`; `xcrun --sdk macosx --show-sdk-version` / `--show-sdk-build-version` → `26.5` / `25F70`. Host OS is macOS 27.0 build `26A5388g`, same spelling as the ledger's execution OS row, on a different device.
- **Verified — the metric lives on the prepared pipeline.** `crates/tiler-metal/src/target.rs` states `Prepared-pipeline facts such as maxTotalThreadsPerThreadgroup, threadExecutionWidth, and staticThreadgroupMemoryLength are deliberately absent`. ADR 0094 decision 7 requires `a confirmation against the prepared pipeline before routing commit`.
- **Verified — MSL states no numeric SIMD-group width.** [The subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md), anchor `The Metal Shading Language Specification 4.1 states no numeric SIMD-group width anywhere`. The only width that specification fixes is the quad-group's 4.
- **Verified — the accepted subject families this profile could later authorize.** `SubgroupRealizationSubject` is width × `ArithmeticType` × `SubgroupTransfer::InRangeXorShuffle` (`crates/tiler-ir/src/schedule/subgroup.rs`). The Apple9 compile profile declares F32 and BF16 `Dispatchable` and is silent on F16 and F64 ([the authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md), headings `F32 — Dispatchable` and `BF16 — Dispatchable`). `simd_shuffle`'s MSL type list excludes `bfloat` and `long` ([the subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md), anchor `simd_shuffle's own type list in MSL Table 6.14 excludes bfloat and long`), so a BF16 XOR-shuffle pipeline is an authorized-family candidate that may fail to compile; that failure is a retained row, not a reason to drop the candidate after the fact.
- **Verified — equality across this population would not remove ADR 0094's preflight.** Decision 7 keeps the prepared-pipeline confirmation even if a compile-profile subject is later declared. [The prepared-width gate](decide-the-prepared-subgroup-width-equality-gate.md), anchor `No compile-profile row alone can discharge this gate`.
- **False as written, repaired before any width was read — `[[threads_per_threadgroup(8,8,1)]]` is not a function attribute.** On this toolchain `xcrun metal` rejects that spelling on a kernel with `attribute cannot be applied to types` / `only applies to parameters and global builtin variables`. The source-side 8×8 control is `[[max_total_threads_per_threadgroup(64)]]` after the function name, which compiled. No `threadExecutionWidth` had been retained when this was corrected.

## Required protocol before any submission

- Freeze the pipeline population, exact compilation-selection identities, source/ABI/oracle subjects, device/profile, metric, repetitions where applicable, environment, custody, and stop conditions in the ticket and retained README.
- Include the exact subgroup candidate families the profile would authorize and negative/control pipelines that could expose width variation. Do not select the matrix after reading widths.
- Build and verify first. Run only in an explicitly granted quiet device window on the authorized host; do not change Xcode, SDK, OS, Rust, or device state.
- Retain every width observation and pipeline identity. Equality or variation is the result; no modal value, first value, or fallback is substituted.
- Perturb pipeline identity, result population, environment, and executable custody independently with unchanged assertions.

## Frozen protocol — 2026-08-13, before any pipeline submission

Recorded in this ticket and in [`spikes/target-profiles/metal-thread-execution-width/README.md`](../spikes/target-profiles/metal-thread-execution-width/README.md). The matrix is the `PIPELINES` array in `spikes/target-profiles/metal-thread-execution-width/src/population.rs`. It is not edited after the first `threadExecutionWidth` read.

### Device / profile

- **Execution host:** `ssh m3`, Apple M3 Pro, macOS 27.0 build `26A5388g`, `arm64`, Apple9. Not the M4 Max qualified numerical row.
- **Offline compilation:** `-std=metal4.0 -target air64-apple-macos26.0` plus the profile-strict numerical flags below, through `xcrun --sdk macosx metal` / `metallib`. ADR 0086 excludes `newLibraryWithSource:`.
- **Profile this may later inform:** a *new* M3 Pro Apple9 compile-profile width claim over the frozen population only. It does not edit `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`.

### Metric, repetitions, oracle

- **Metric:** `MTLComputePipelineState.threadExecutionWidth` on the exact prepared pipeline. `maxTotalThreadsPerThreadgroup` and `staticThreadgroupMemoryLength` are retained as corroborating prepared facts, never as a substitute for the metric.
- **Repetitions:** three independent `newComputePipelineState` constructions per frozen pipeline identity. Every repetition is retained.
- **Oracle:** none. The property is read, not compared to an expected numeric width. Equality or variation is computed from the retained set after the last read.

### Source / ABI / compilation-selection identities

- **Source subject:** one retained `.metal` file per kernel, hashed, compiled in isolation so a failing control cannot hide a required kernel.
- **ABI subject:** entry point name plus the pipeline-descriptor shape below.
- **Compilation-selection identity:** the exact `xcrun metal` flag vector. Seven identities:

| id | flags beyond `-c -o` |
| --- | --- |
| `profile_strict` | `-std=metal4.0 -target air64-apple-macos26.0 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` |
| `math_fast` | same as `profile_strict` with `-fmetal-math-mode=fast` |
| `math_relaxed` | same as `profile_strict` with `-fmetal-math-mode=relaxed` |
| `contract_fast` | same as `profile_strict` with `-ffp-contract=fast` |
| `opt_O0` | `profile_strict` plus `-O0` |
| `opt_Os` | `profile_strict` plus `-Os` |
| `std_metal3.1` | `profile_strict` with `-std=metal3.1` instead of `-std=metal4.0` |

- **Descriptor shapes:** `default` (unset max, `threadGroupSizeIsMultipleOfThreadExecutionWidth` left at Metal's default false); `max_1` / `max_32` / `max_256` / `max_1024`; `multiple_of_width`; `max_1024_multiple`.

### Authorized subgroup candidate families

These are the subjects `FIRST_MACOS_APPLE9` could later declare `Realized` if a width were licensed: `InRangeXorShuffle` at F32 (Dispatchable, shuffle in MSL Table 6.14) and at BF16 (Dispatchable, shuffle excluded from that table). F16 and F64 are silence on that profile and are controls, not candidate families.

### Frozen pipeline identities

Required pipelines must compile and prepare or the run aborts with no equality claim. Optional pipelines retain a compile or prepare failure as a row.

| id | role | required |
| --- | --- | --- |
| `xor_shuffle_f32/profile_strict/default` | authorized F32 XOR-shuffle | yes |
| `xor_shuffle_bf16/profile_strict/default` | authorized-family BF16 candidate | no |
| `xor_shuffle_f32/profile_strict/max_1` | threadgroup-shape control | yes |
| `xor_shuffle_f32/profile_strict/max_32` | threadgroup-shape control | yes |
| `xor_shuffle_f32/profile_strict/max_256` | threadgroup-shape control | yes |
| `xor_shuffle_f32/profile_strict/max_1024` | threadgroup-shape control | yes |
| `xor_shuffle_f32/profile_strict/multiple_of_width` | descriptor control | yes |
| `xor_shuffle_f32/profile_strict/max_1024_multiple` | descriptor control | yes |
| `xor_shuffle_f32/math_fast/default` | compiler-selection control | yes |
| `xor_shuffle_f32/math_relaxed/default` | compiler-selection control | yes |
| `xor_shuffle_f32/contract_fast/default` | compiler-selection control | yes |
| `xor_shuffle_f32/opt_O0/default` | compiler-selection control | yes |
| `xor_shuffle_f32/opt_Os/default` | compiler-selection control | yes |
| `xor_shuffle_f32/std_metal3.1/default` | compiler-selection control | yes |
| `store_u32/profile_strict/default` | independent-store control | yes |
| `add_f32/profile_strict/default` | elementwise F32 | yes |
| `add_f16/profile_strict/default` | elementwise F16 (profile-silent) | yes |
| `add_bf16/profile_strict/default` | elementwise BF16 | yes |
| `add_i32/profile_strict/default` | integer arithmetic control | yes |
| `add_f64/profile_strict/default` | elementwise F64 (profile-silent) | no |
| `xor_shuffle_f16/profile_strict/default` | F16 XOR-shuffle control | yes |
| `xor_shuffle_f64/profile_strict/default` | F64 XOR-shuffle control | no |
| `simd_sum_f32/profile_strict/default` | negative: refused collective | yes |
| `shuffle_down_f32/profile_strict/default` | negative: refused narrowing tree | yes |
| `quad_shuffle_f32/profile_strict/default` | control: spec-fixed quad width 4 | yes |
| `divergent_cf_f32/profile_strict/default` | control-flow control | yes |
| `loop_f32/profile_strict/default` | loop control | yes |
| `threadgroup_mem_f32/profile_strict/default` | threadgroup-memory control | yes |
| `threadgroup_mem_f32/profile_strict/max_1024` | memory × descriptor | yes |
| `high_reg_f32/profile_strict/default` | register-pressure control | yes |
| `constrained_tg_8x8/profile_strict/default` | source-side `[[max_total_threads_per_threadgroup(64)]]` (8×8 product) | yes |
| `source_max_tg_32/profile_strict/default` | source-side `[[max_total_threads_per_threadgroup(32)]]` | yes |
| `add_f32/math_fast/default` | compiler × non-shuffle | yes |
| `add_f32/opt_O0/default` | compiler × non-shuffle | yes |

**34 pipelines.** Counted from this table and from `PIPELINES.len()`.

### Environment, custody, stop conditions

- Record offline metal/linker/Xcode/SDK, rustc verbose, device name, `registryID`, `supportsFamily(Apple9)`, `maxBufferLength`, load averages, `sw_vers`. Do not change Xcode, SDK, OS, Rust, or device state.
- Custody: SHA-256 of every kernel file, of concatenated `src/*.rs` in name order, of `Cargo.lock`, and of the running executable. Source and lock digests are recomputed at validation; the executable digest is a recorded fact, not rebuilt on another host.
- **Stop:** no Apple9 default device; any required pipeline fails to compile or prepare; any attempt to read a width before the freeze above is committed in this ticket and the spike README. Optional compile/prepare failures are rows. Display-asleep is recorded, not gated.

### Perturbations

`cargo test` in the spike directory perturbs pipeline identity, result population, environment, and executable custody independently against unchanged assertions and quotes the failure text.

## Outcome boundary

This measurement may license only the observed profile/pipeline population. Even perfect equality does not remove ADR 0094's prepared-pipeline confirmation without a separate accepted decision. Variation requires per-pipeline evidence and categorically forbids a single compile-profile width row.

## Closes when

The predeclared population has a retained, reproducible result and the exact target-profile claim it supports—or fails to support—is recorded.

## Outcome

**Measurement, 2026-08-13, Apple M3 Pro.** Retained at [`spikes/target-profiles/metal-thread-execution-width/results/2026-08-13-apple-m3-pro-macos27.0-26A5388g/widths.json`](../spikes/target-profiles/metal-thread-execution-width/results/2026-08-13-apple-m3-pro-macos27.0-26A5388g/widths.json). Host `ssh m3`, device name `Apple M3 Pro`, `registryID 0x1000004e5`, `supportsFamily(Apple9)` true, macOS 27.0 build `26A5388g`, `arm64`. Offline: `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883`, Xcode 26.6 build 17F113, SDK 26.5 build 25F70. rustc `1.99.0-nightly (eff8269f7 2026-07-18)`. Load `{ 3.66 3.52 2.77 }`. Xcode, SDK, OS, Rust, and device state were not changed.

**Frozen population:** 34 identities in `PIPELINES` / the table above. 31 compiled and prepared three times each (93 width observations). Three optional identities failed to compile and have no width: `xor_shuffle_bf16/profile_strict/default` (`no matching function for call to 'simd_shuffle_xor'`), `add_f64/profile_strict/default` and `xor_shuffle_f64/profile_strict/default` (`'double' is not supported in Metal`).

**Every retained width is 32.** The 31 prepared identities, listed with their three repetitions:

`xor_shuffle_f32/profile_strict/default` 32,32,32; `xor_shuffle_f32/profile_strict/max_1` 32,32,32; `xor_shuffle_f32/profile_strict/max_32` 32,32,32; `xor_shuffle_f32/profile_strict/max_256` 32,32,32; `xor_shuffle_f32/profile_strict/max_1024` 32,32,32; `xor_shuffle_f32/profile_strict/multiple_of_width` 32,32,32; `xor_shuffle_f32/profile_strict/max_1024_multiple` 32,32,32; `xor_shuffle_f32/math_fast/default` 32,32,32; `xor_shuffle_f32/math_relaxed/default` 32,32,32; `xor_shuffle_f32/contract_fast/default` 32,32,32; `xor_shuffle_f32/opt_O0/default` 32,32,32; `xor_shuffle_f32/opt_Os/default` 32,32,32; `xor_shuffle_f32/std_metal3.1/default` 32,32,32; `store_u32/profile_strict/default` 32,32,32; `add_f32/profile_strict/default` 32,32,32; `add_f16/profile_strict/default` 32,32,32; `add_bf16/profile_strict/default` 32,32,32; `add_i32/profile_strict/default` 32,32,32; `xor_shuffle_f16/profile_strict/default` 32,32,32; `simd_sum_f32/profile_strict/default` 32,32,32; `shuffle_down_f32/profile_strict/default` 32,32,32; `quad_shuffle_f32/profile_strict/default` 32,32,32; `divergent_cf_f32/profile_strict/default` 32,32,32; `loop_f32/profile_strict/default` 32,32,32; `threadgroup_mem_f32/profile_strict/default` 32,32,32; `threadgroup_mem_f32/profile_strict/max_1024` 32,32,32; `high_reg_f32/profile_strict/default` 32,32,32; `constrained_tg_8x8/profile_strict/default` 32,32,32; `source_max_tg_32/profile_strict/default` 32,32,32; `add_f32/math_fast/default` 32,32,32; `add_f32/opt_O0/default` 32,32,32.

`verdict.widths_observed` is `[32]`. `verdict.all_prepared_widths_equal` is true.

**Claim this supports.** On this M3 Pro Apple9 host, under this offline toolchain and this frozen population, `MTLComputePipelineState.threadExecutionWidth` did not vary: every pipeline that prepared reported 32. That is evidence a later M3 Pro compile-profile subject could name width 32 for `InRangeXorShuffle` at F32 (and at F16 as a control that prepared, not as a dispatchability row).

**Claim this fails to support.** It does not source `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` or any M4 Max qualified row. It does not license a BF16 `InRangeXorShuffle` realization — that candidate did not compile. It does not remove ADR 0094's prepared-pipeline confirmation. It is not an Apple9-family guarantee.

Perturbations, assertions unchanged, quoted:

- pipeline identity: `pipeline identity not-a-frozen-identity is not in the frozen population`
- result population: `result population for xor_shuffle_f32/profile_strict/default has 2 preparations, expected 3`
- environment: `environment digest does not match the recorded environment subject`
- executable custody: `ending executable digest does not match retained custody`

Carry: this delta is `tickets/` + `spikes/` + no `crates/`, `Cargo.*`, or `Makefile`. `4ef52cfe` (ticket-only successor of `eecc4002`) therefore carries; `tkt lint` and `make citations` are the required re-runs.
