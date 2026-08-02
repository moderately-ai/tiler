---
id: measure-macos-apple9-bf16-under-unified-msl4-profile
title: Measure macOS Apple9 BF16 under the unified MSL 4 profile
status: done
priority: p1
dependencies: []
related: [measure-macos-apple9-f32-under-unified-msl4-profile, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, declare-the-bf16-rows-on-the-authoritative-metal-profile, first-authoritative-ios-metal-compile-declaration]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [metal, numerics, bf16, target-profiles, evidence, measurement]
---
## User-visible outcome

The authoritative Metal compile profile's BF16 rows have a measurement produced under the exact unified MSL 4.0 / macOS 26 target the profile names, so `declare-the-bf16-rows-on-the-authoritative-metal-profile` can transcribe rather than re-attribute. Until this lands, the macOS BF16 flush and dispatchability facts exist only on a compilation the profile refuses by name.

## Why this ticket exists — the gap that blocked the profile ticket

**Fact.** The only retained record carrying BF16 is `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv`, whose `probe.fixed_flags` is `-std=metal3.1` and whose `environment.family.macos.requested_target` is `air64-apple-macos13.0` (emitted `air64_v26-apple-macosx14.0.0`). The four `*-apple9-f32-unified-msl4-macos26-*` records are the MSL 4.0 row and carry `probe.dtypes f32` only.

```sh
for f in spikes/apple-targets/results/*/record.tsv; do
  printf '%s\t%s\t%s\n' "$(grep -m1 '^probe.fixed_flags' "$f" | cut -f2)" \
    "$(grep -m1 '^probe.dtypes' "$f" | cut -f2)" "$f"
done
```

**Fact — three independent authorities refuse the transcription.** `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` line 227 ("Reusing the older MSL 3.1 / macOS 14.0 record for this profile would attribute measurements to a compilation that did not produce them") and line 247 ("BF16 additionally has a *macOS-only* measurement on the older MSL 3.1 row ... neither reaches this profile"); `tickets/measure-macos-apple9-f32-under-unified-msl4-profile.md` line 19 ("It is excellent evidence for that exact row and is not evidence for a new MSL 4.0/macOS 26 profile merely because the same host and compiler accept both") and line 23, whose measurement boundary names BF16 among the things not to generalize to; and `crates/tiler-build/src/metal_declaration.rs`, whose live test `the_declaration_does_not_carry_the_superseded_msl_3_1_record` asserts the superseded record absent.

**Fact — the profile vocabulary cannot record the difference, which is why care is not a substitute.** `TargetCompileProfileMeasurementSource` carries `TargetCompilerBuild` (role, implementation, version, build) and `TargetExecutionEnvironment` (platform, platform version, platform build, architecture, hardware) and nothing else. Every one of those fields is byte-identical between the two records — same Metal 32023.883, same AIR-LLD 32023.883, same Xcode 26.6/17F113, same macOS SDK 26.5/25F70, same macOS 27.0/26A5388g/arm64/Apple M4 Max. The language standard and requested target are exactly the two components that differ and exactly the two the source cannot hold, so an MSL 3.1-sourced BF16 row would be indistinguishable in the profile descriptor from an MSL 4.0-sourced one. `record-the-compilation-selection-in-target-measurement-provenance` owns that gap; this ticket removes the need to rely on it.

## Experiment keys

Add one named indivisible profile to `spikes/apple-targets/numerical_probe.py` beside `APPLE9_F32_UNIFIED_MSL4_MACOS26` (`spikes/apple-targets/numerical_probe.py:936`) covering `macos` at `-std=metal4.0` / `air64-apple-macos26.0` with the BF16 dtype the harness already defines (`spikes/apple-targets/numerical_probe.py:754`). Do **not** widen the existing `apple9-f32-unified-msl4-macos26` profile in place: its retained records are cited by the authority ledger and by `construct-and-bind-the-first-authoritative-metal-compile-profile`, and changing its case set moves `probe.harness_sha256` on evidence four documents already pin.

Run both the covering and exhaustive matrices, retain the result directories, and re-run the record validator. The dispatchability half needs `environment.family.macos.device_bfloat_support` under the MSL 4.0 profile, not only the arithmetic cases.

## Required evidence

- The BF16 flush dimensions at MSL 4.0: the operand pairs the MSL 3.1 row measured (`0040` → `0000`, `0080` → `0000`, and the sign rows `8040` → `8000`), each carrying an execution witness reporting `executed`, plus `materialize_bf16` returning all eight operands unchanged so the zeros are attributable to arithmetic rather than to a buffer round trip.
- `device_bfloat_support` for `macos` under the MSL 4.0 profile.
- A negative validator mutation that changes a producer-defining input or manifest byte and must fail, watched failing.
- The research memo states which BF16 claims the MSL 4.0 row supports and which stay `Unknown`.

## Outcome — measured 2026-08-02

**Measurement.** `apple9-f32-bf16-unified-msl4-macos26` was added beside the F32 profile (producer commit `0fcc952ac8f548f462eff6b204386253e65d2522`) and both matrices retained: `spikes/apple-targets/results/2026-08-02-numerics-{covering,exhaustive}-apple9-f32-bf16-unified-msl4-macos26-xcode26.6-metal32023.883`. Environment: arm64 macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, Apple M4 Max reporting Apple9 `supported`, registry ID 4294968452; offline Metal/AIR-LLD 32023.883, runtime `GPUCompiler.framework` `metalfe-32023.921`; `air64-apple-macos26.0` emitted `air64_v28-apple-macosx26.0.0`.

**The BF16 flush dimensions at MSL 4.0**, identical under `safe`, `relaxed`, and `fast` and on both compilation paths — all seven of finding 24's dimensions, not only the three named above, because the harness sweeps every `bf16` kernel in all three modes:

| dimension | kernel | operand → result | execution witness |
| --- | --- | --- | --- |
| input flush, multiply | `multiply_two_bf16` | `0040` → `0000` | `operand=3f80,expected=4000,observed=4000,status=executed` |
| input flush, sign | `multiply_two_bf16` | `8040` → `8000` | `operand=3f80,expected=4000,observed=4000,status=executed` |
| result flush, multiply | `multiply_half_bf16` | `0080` → `0000` | `operand=3f80,expected=3f00,observed=3f00,status=executed` |
| input flush, additive path | `add_smallest_normal_bf16` | `8040` → `0080` | `operand=0080,expected=0100,observed=0100,status=executed` |
| input flush, division | `divide_by_three_eighths_bf16` | `0040` → `0000` | `operand=3f80,expected=402b,observed=402b,status=executed` |
| input flush, division, sign | `divide_by_three_eighths_bf16` | `8040` → `8000` | `operand=3f80,expected=402b,observed=402b,status=executed` |
| result flush, division | `divide_by_three_bf16` | `0080` → `0000` | `operand=3f80,expected=3eab,observed=3eab,status=executed` |

`materialize_bf16` returns `0001 0040 007f 0080 8040 8000 3eab 3f80` — all eight operands unchanged, `float_operations` `none` — so the zeros are attributable to arithmetic and not to a buffer round trip. `environment.family.macos.device_bfloat_support` is `supported`. Covering BF16 witness census: 91 `executed`, 8 `not-executed` (every `scale_one_bias_zero_bf16` case under `relaxed`/`fast` — the finding 7 trap the guard refuses), 18 `none` (`materialize_bf16` and `multiply_one_bf16`, declared witnessless). No `disagrees`.

**The control that makes the attribution stick.** Excluding `bf16` rows, all 864 covering and 996 exhaustive `case.*`/`comparison.*` rows are byte-identical to the 2026-07-31 `apple9-f32-unified-msl4-macos26` record, so this profile's compilation is demonstrably the one the authoritative profile names. The F32 profile was not widened: its four retained records are byte-identical to what they were, and `probe.harness_sha256` on them is unchanged at `e7b831d61024efcad712bce1495c0f2d078ef9ac766308e20d4a424e2d547d04`.

**What the MSL 4.0 row supports:** the macOS `bf16` subnormal flush across all seven isolated dimensions, its sign preservation, arithmetic-free materialization, and `bfloat` dispatchability — each on this exact Apple9/macOS/toolchain row, under this exact compilation.

**What stays `Unknown`:** `f16` under MSL 4.0 (this profile does not carry it — `device_bfloat_support` aside, no `f16` case was measured); both iOS families entirely, so finding 26's simulator refusal and the iOS device's never-asked row remain MSL 3.1-only and are gated on `first-authoritative-ios-metal-compile-declaration`; any other Apple GPU family, OS or SDK build, compiler build, or deployment minimum; and the finding 24 hypothesis-A/native-`bfloat` distinction, which no single-operation probe can separate at this width.

**Collateral this ticket caused, deliberately and reversibly.** Editing the shared harness moves `probe.harness_sha256`, so the current tree's validator now refuses the four retained F32 records with `validator digest mismatch`. Nothing measured moved. Revalidation runs from a detached worktree at each record's own `probe.repository_base_revision` (`0cd85ce5…` for the 2026-07-30 pair, `93ddc4a3…` for the 2026-07-31 pair); both 2026-07-31 records were re-validated that way at exit 0, and the procedure is recorded in the spike README and the research record. This was unavoidable: any harness edit does it, and the ticket requires one.

## Closes when

A clean replay produces the retained MSL 4.0 / macOS 26 BF16 covering and exhaustive rows, the validator demonstrably rejects a mutated producer input, the Apple numerical behaviour record cites the new row beside findings 24 and 26 without overwriting them, and `declare-the-bf16-rows-on-the-authoritative-metal-profile` has a record it can transcribe under the profile's own compilation.

## Graph maintenance

- Blocks `declare-the-bf16-rows-on-the-authoritative-metal-profile`, whose macOS half is unstatable without it.
- Related to `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes`, which owns the MSL 3.1 BF16 measurement and stays `done` — this ticket does not supersede it, it re-measures the same dimensions on the profile's own compilation row.
- Related to `measure-macos-apple9-f32-under-unified-msl4-profile`, whose shape this follows and whose retained records must not be disturbed.
- This ticket does **not** cover either iOS family. The iOS Simulator's BF16 refusal and the iOS device's absent row are gated on `first-authoritative-ios-metal-compile-declaration`, which is `deferred`.
