---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.metal-thread-execution-width.protocol-2026-08-22-standard-profile"
kind: "experiment"
title: "Frozen protocol: threadExecutionWidth for the standard macOS Apple9 profile, on the 26A5416b row"
topics: ["target-profiles", "metal", "apple-targets", "subgroup", "feasibility", "provenance"]
experiment_status: "frozen"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
ticket: "measure-thread-execution-width-on-the-standard-metal-profiles-own-host"
---

# Frozen protocol — `threadExecutionWidth` for the standard macOS Apple9 profile, on the `26A5416b` row

**This document is written and committed before a single width is read.** It is the second frozen protocol over the harness in this directory. The first ([`README.md`](README.md)) pre-scoped its record to a *new M3 Pro Apple9 claim over the frozen population only* and is therefore closed to every other beneficiary, permanently, under [ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md) component 3. That record is untouched by this one, and nothing here rescopes it.

## Pre-registered beneficiary

The profile this measurement may inform is named here, before the run:

> `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`

That key, and no other. ADR 0113 component 3 admits a measured row into a family-keyed profile only when the producing measurement's frozen protocol named that exact key as beneficiary **before the run**, the declaring module transcribes only what the record states and refuses every other value by name, and the row's validity is `MeasuredEnvironment`. This document discharges the first condition and nothing else. The second and third are the declaring module's, and declaring is not this ticket's work — see "What this protocol does not authorize".

The candidate row, if the evidence supports it, is a `SubgroupRealizationSubject { width, arithmetic, transfer: InRangeXorShuffle }` marked `Realized` only for arithmetic the standard profile already marks `Dispatchable`: F32 and BF16. Every unobserved subject stays silent.

## Pre-named execution environment — the only row this measurement is valid for

Re-derived on the measuring host immediately before the run, not carried from the ticket, which records the superseded `26A5406e`:

| Component | Exact value | Command |
| --- | --- | --- |
| OS | macOS 27.0, build `26A5416b` | `sw_vers -productVersion`, `sw_vers -buildVersion` |
| Architecture | `arm64` | `uname -m` |
| Device | `Apple M4 Max` (`Mac16,6`) | `sysctl -n machdep.cpu.brand_string`, `hw.model` |
| Apple GPU family | `apple9`, asserted by the harness before any pipeline is built | `MTLDevice.supportsFamily` |

`26A5416b` is a **third** execution environment for this profile, beside the ledger's environment **A** (`26A5406e`) and **B** (`26A5388g`). It is not either of them and may not be folded into either. Under ADR 0113 component 2 it carries its own population source; under component 4 a host is inside this row's scope only on byte-exact equality of every environment field, and even then admission stays subject to [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md).

## Pre-named offline compilation environment

The run pins `DEVELOPER_DIR=/Applications/Xcode.app`, exactly as the first protocol does. Under that selector this host resolves the authority ledger's offline table **field for field**:

| Component | Exact value |
| --- | --- |
| Offline compiler | `Apple metal version 32023.883 (metalfe-32023.883)` |
| Offline linker | `AIR-LLD 32023.883 (metalfe-32023.883) (compatible with legacy metallib linker)` |
| Xcode | `Xcode 26.6 Build version 17F113` |
| macOS SDK | `macosx` 26.5, build `25F70` |
| Language standard | `-std=metal4.0`, requested target `air64-apple-macos26.0` |

**The `32023.921` toolchain is deliberately not the one measured under.** Bare `xcrun --sdk macosx --find metal` on this host resolves a downloaded Metal toolchain reporting `Apple metal version 32023.921 (metalfe-32023.921)`, because `xcode-select -p` points at `/Applications/Xcode-beta.app/Contents/Developer` (Xcode 27.0, build `27A5228h`, SDK 27.0 `26A5388f`). `DEVELOPER_DIR` overrides that selection for the invocation and mutates no host state. Selecting the toolchain the profile's other rows were compiled under is the point: it removes the offline row as a difference, so the only axis this measurement varies from environments A and B is the execution build. Whichever toolchain answers, the harness records it rather than trusting this table, and the retained record is authoritative over this paragraph.

## Workload and population — unchanged, and provably so

The frozen population is the existing `PIPELINES` array: **34 identities**, `PIPELINE_COUNT == 34`, over the kernel, ABI/descriptor, arithmetic, control-flow, threadgroup-shape, and compiler-selection subjects the first protocol enumerates. It is not edited for this run, and no kernel is added or removed.

The harness source is **not modified**. This is a hard constraint rather than a preference: `validate` recomputes `harness_source_sha256` from the tree and holds the retained record against it, so editing any `src/*.rs`, any `kernels/*.metal`, or `Cargo.lock` would make the retained 2026-08-13 M3 Pro record fail validation. The digest pinned in that record, `a918c8e423ccb85f89334ed2f397efc926d89f0622d4ea676cdb44d48bb8ba38`, is the tree's value at this base, confirmed by `validate` reporting `validation passed` before this protocol was written.

Both records therefore share one harness, one population, and one metric, and differ only in execution environment — which is what makes them comparable at all.

## Metric, baseline, warm-up, repetitions, oracle

- **Metric:** `MTLComputePipelineState.threadExecutionWidth`, read from a prepared pipeline. Corroborating prepared facts recorded but never substituted for it: `maxTotalThreadsPerThreadgroup`, `staticThreadgroupMemoryLength`.
- **Baseline:** the retained 2026-08-13 M3 Pro record's whole-population `32`. It is a **comparison**, not an expectation, and it is not evidence for this row: a differing result here is a finding about this row, not a regression against that one.
- **Warm-up:** none, and none is applicable. The metric is a property of a prepared pipeline object, not a timing; there is no steady state to reach. No dispatch is executed.
- **Repetitions:** three independent pipeline constructions per identity, each retained. Variance is reported as the observed set per identity, not as a summary statistic.
- **Oracle:** none. The property is read; equality or variation is computed from the retained set. No modal, first, or fallback value may be substituted — `derive_verdict` recomputes the verdict and `validate` refuses a record whose verdict does not match its own observations.

## Noise controls, and the one that is deliberately absent

Load averages are **recorded, not gated**, which is the harness's existing design (`record.rs`: *"`uptime` load averages, recorded rather than gated"*). This is correct for this metric and would be wrong for a timing: `threadExecutionWidth` is a static property of a compiled pipeline for a device, and a busy machine does not change it. This measurement therefore does **not** require the idle-host discipline AGENTS.md imposes on CPU timing and profiling, and it is not evidence about performance under any load. The load at the run is retained so a reader can see the condition rather than infer it.

Custody is unchanged: SHA-256 of every kernel, of concatenated `src/*.rs` in name order, of `Cargo.lock`, and of the running executable at start and end.

## Stop conditions, frozen before the run

The run aborts with **no equality claim and no retained row** if:

- there is no default Metal device, or the default device does not report `supportsFamily(Apple9)`;
- any pipeline marked `required` fails to compile, or fails to prepare, or prepares without a width;
- the record fails its own validation immediately after measurement.

Optional compile failures are **retained as rows**, not dropped. Three are expected from the first record and are properties of the language, not the host: `xor_shuffle_bf16` (MSL Table 6.14 excludes `bfloat` from `simd_shuffle_xor`), `add_f64` and `xor_shuffle_f64` (Metal has no `double`).

If the observed widths are **not** all equal, the outcome is that no single width row may be declared for this population on this environment, and that is a complete and reportable result rather than a failed run.

## What this protocol does not authorize

- **It does not declare anything.** Landing a subgroup row on `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` is a `crates/tiler-build` change, and it moves the flagship public profile's stated content, its descriptor, and every dependent pin. ADR 0113 keeps that a Tom-facing decision packet. This protocol produces the evidence such a packet would need and stops there.
- **It sources no other row.** Not the grid-axis extent, not the cost row, not the tree-width policy, not any dispatchability or numerical row. Those rest on their own populations and environments.
- **It makes no family claim.** A width observed on one Apple9 device under one build is not an Apple9-family guarantee, and ADR 0094 decision 7's prepared-pipeline `ObservedEqualsRequired` confirmation stays in force regardless of the outcome.
- **It makes no portability, performance, or cross-build claim.** Nothing here establishes that a row measured on `26A5416b` holds on `26A5406e` or `26A5388g`, and nothing asks it to.

## Commands, frozen

```sh
cd spikes/target-profiles/metal-thread-execution-width
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release -- measure \
  > results/2026-08-22-apple-m4-max-macos27.0-26A5416b/widths.json
DEVELOPER_DIR=/Applications/Xcode.app cargo test
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release -- validate \
  results/2026-08-22-apple-m4-max-macos27.0-26A5416b/widths.json
```

## Result

Recorded after the run, below this line, and nowhere above it.
