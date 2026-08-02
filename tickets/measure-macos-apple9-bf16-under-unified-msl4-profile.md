---
id: measure-macos-apple9-bf16-under-unified-msl4-profile
title: Measure macOS Apple9 BF16 under the unified MSL 4 profile
status: todo
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

## Closes when

A clean replay produces the retained MSL 4.0 / macOS 26 BF16 covering and exhaustive rows, the validator demonstrably rejects a mutated producer input, the Apple numerical behaviour record cites the new row beside findings 24 and 26 without overwriting them, and `declare-the-bf16-rows-on-the-authoritative-metal-profile` has a record it can transcribe under the profile's own compilation.

## Graph maintenance

- Blocks `declare-the-bf16-rows-on-the-authoritative-metal-profile`, whose macOS half is unstatable without it.
- Related to `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes`, which owns the MSL 3.1 BF16 measurement and stays `done` — this ticket does not supersede it, it re-measures the same dimensions on the profile's own compilation row.
- Related to `measure-macos-apple9-f32-under-unified-msl4-profile`, whose shape this follows and whose retained records must not be disturbed.
- This ticket does **not** cover either iOS family. The iOS Simulator's BF16 refusal and the iOS device's absent row are gated on `first-authoritative-ios-metal-compile-declaration`, which is `deferred`.
