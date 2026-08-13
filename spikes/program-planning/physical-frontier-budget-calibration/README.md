---
schema: "tiler-doc/v1"
id: "tiler.spike.program-planning.physical-frontier-budget-calibration"
kind: "experiment"
title: "Physical-frontier provider-count and raw-outcome budget calibration"
topics: ["program-planning", "budgets", "measurement", "host-performance"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "exhaustive-finite"]
supports: ["tiler.research.program-planning.physical-frontier-budget-calibration"]
entrypoints: ["spikes/program-planning/physical-frontier-budget-calibration/src/main.rs"]
last_verified: "2026-08-13"
ticket: "calibrate-the-physical-frontier-provider-and-outcome-budgets"
---

# Physical-frontier provider-count and raw-outcome budget calibration

This harness censuses the compiler-owned physical-provider population and measures Tiler host runtime and process memory as installed provider count and raw proposal/decline emissions grow. It does not time kernel execution.

From this directory. `rust-toolchain.toml` is resolved by directory ancestry from the repository root.

```sh
cd spikes/program-planning/physical-frontier-budget-calibration
CARGO_TARGET_DIR=./target cargo build --release
./target/release/physical-frontier-budget-calibration census
./target/release/physical-frontier-budget-calibration perturb extra-production-provider
./target/release/physical-frontier-budget-calibration record results/2026-08-13-macos-27.0-m3-pro.json
```

`--quick` shortens warmup, repeats, and sweep points and skips `/usr/bin/time -l`. Nothing recorded here was produced with `--quick`. Spikes gate nothing.

## Recommended first limits

| Limit | Value | Headroom | Why not eight |
| --- | --- | --- | --- |
| Provider count, including the governed provider | **32** | **32** (measured-good empty and decline populations through 64 extras; untyped `InvalidCompilerOutput` at 128 all-decline extras) | Empty extras through 64 stay inside 1.11× the governed floor. Eight is below every measured binding constraint. |
| Raw proposal+decline outcomes, request-scoped | **256** | **256** (512 still below the 1088-decline working point and the 1920-decline untyped wall) | The governed five-op program already emits 17 extra-provider outcomes per installed observer, and the governed provider itself answers every one of those 17 subjects. A request-scoped eight would refuse the ordinary governed compile. |

One raw-outcome *count* remains the accepted cardinality bound. It is sized from the expensive side: an admitted proposal is about 34× a named decline on incremental host time (39 µs vs 1.1 µs). Using decline cost to pick the ceiling would admit a proposal-heavy population the host-time envelope did not measure.

## What these limits do not bound

They do not bound arbitrary native provider computation or allocation before an emission. A provider that loops, allocates a huge `ScheduledRegion`, or builds an oversized body still does that work before the host can charge the outcome. They also do not replace `physical_plan_combinations` (4096), and they do not sandbox explain-record growth except by cutting the outcomes that feed it.

## Census

Workload: `hot_path.rs`'s five-operation scale-then-reduce at 4×3, `NumericalContract::STRICT_F32`, `TargetProfile::governed()` except the infeasible rows, which use a declared workgroup-capacity profile so overrun is a hard rejection rather than a deferred predicate.

| Population | Governed only | +1 empty observer | +1 retained external specialist | +2 equal-cost specialists | +1 all-decline | +1 infeasible |
| --- | --- | --- | --- | --- | --- | --- |
| Distinct region subjects | 17 (compiler memo: one `propose` per subject) | 17 | 17 | 17 | 17 | 17 |
| Provider invocations | 17 (governed) | 17 extra | 17 extra | 34 extra | 17 extra | 17 extra |
| Extra proposals | 0 | 0 | 3 | 6 | 0 | 3 |
| Extra declines | 0 | 0 | 14 | 28 | 17 | 14 |
| Admitted extra implementations | 0 | 0 | 3 baseline subjects | 3 baseline subjects × 2 | 0 | 0 (rejected) |
| Complete-plan alternatives | 1 | 1 | 2 | 3 | 1 | 2 (profile, not admission) |
| Compiler-owned production `PhysicalImplementationProvider` impls | 1 (`GovernedPhysicalProvider`) | | | | | |

**Fact.** The compiler-owned production population is one impl. Integration-test and `#[cfg(test)]` fixtures are not production providers. The retained external-provider vertical is the public-surface specialist that clones `ImplementationContext::baseline` and perturbs workgroup width, the same shape as `crates/tiler-compiler/tests/external_physical_provider.rs`.

**Fact.** Equal-cost incomparable proposals grow retained alternatives linearly here (`alternatives = extra + 1`) because only the baseline subjects receive an extra body and the selected covers have one varying region. That is a fact about this program, not a bound on Cartesian growth in general.

## Host-runtime record

**Measurement**, 2026-08-13, Apple M3 Pro, macOS 27.0 (26A5388g / Darwin 27.0.0), 11 logical CPUs, 18 GiB, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` — the `rust-toolchain.toml` pin `nightly-2026-07-19`. Release profile. Warm-up 8, repeats 50, estimator is the minimum. Load averages at start `{ 6.82 3.66 2.70 }` (the cargo release build had just finished on the same host); at end `{ 6.32 3.66 2.71 }`. Min/p90 on the governed row are 3205 / 3251 µs, so the floor is readable even under that load. Peak RSS is `/usr/bin/time -l` maximum resident set size of a child that warms twice and compiles once.

| Row | Extra providers | Min µs | × governed | Peak RSS | Notes |
| --- | --- | --- | --- | --- | --- |
| governed-only | 0 | 3205 | 1.00 | 25.9 MB | one alternative |
| empty 8 / 32 / 64 | 8 / 32 / 64 | 3230 / 3376 / 3561 | 1.01 / 1.05 / 1.11 | 25.8–26.0 MB | invocations only |
| decline 16 / 64 / 128 | 16 / 64 / 128 | 3684 / 5607 / 5358 | 1.15 / 1.75 / — | 27.1 / 29.6 / 27.6 MB | 128 extras → `InvalidCompilerOutput` |
| propose 1 / 8 / 16 | 1 / 8 / 16 | 3869 / 8871 / 14813 | 1.21 / 2.77 / 4.62 | 30.4 / 54.4 / 82.3 MB | alternatives 2 / 9 / 17 |
| infeasible 16 / 32 | 16 / 32 | 5549 / 7349 | 1.73 / 2.29 | 30.3 / 31.1 MB | verification without selection growth |

The retained result is [`results/2026-08-13-macos-27.0-m3-pro.json`](results/2026-08-13-macos-27.0-m3-pro.json).

## Perturbations

Each load-bearing check keeps its assertion and fails when its subject is perturbed. Restored afterwards; `census` is green.

| Perturb | Failure quoted |
| --- | --- |
| `extra-production-provider` | `FAIL compiler-owned-production-providers expected=1 observed=2` |
| `tiny-program` | `FAIL distinct-region-subjects expected=17 observed=3` |
| `missing-observer` | `FAIL distinct-region-subjects expected=17 observed=0` |
| `silent-decline` | `FAIL many-declines-count expected=17 observed=0` |
| `decline-instead-of-propose` | `FAIL external-vertical-proposals expected=3 observed=0` |
| `one-additive-specialist` | `FAIL two-additive-provider-count expected=2 observed=1` |
| `feasible-instead-of-infeasible` | `FAIL infeasible-not-selected expected=rejected observed=selected` |

## Measurement boundary

One host, one toolchain, one five-operation program, one numerical contract, two target profiles (governed prototype and a declared-capacity profile for infeasible rows). Nothing here is a portable guarantee or a kernel time. Provider identities are synthetic `acme::*@1` specialists; they exercise the public seam, not a third-party crate's own compute.
