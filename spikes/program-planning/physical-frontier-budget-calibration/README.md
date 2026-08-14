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
./target/release/physical-frontier-budget-calibration request-census
./target/release/physical-frontier-budget-calibration perturb extra-production-provider
./target/release/physical-frontier-budget-calibration record results/2026-08-13-macos-27.0-m3-pro.json
```

The last command is reserved for the idle M3 Pro. It now measures request-wide 1, 2, 8, and 16-target governed/specialist rows, the four-contract add chain, the governed-plus-two-specialist population, and the full 31-installed-specialist population with the same warm-up 8, repeats 50, and child-RSS protocol. `--quick` shortens warmup, repeats, and sweep points and skips `/usr/bin/time -l`. Spikes gate nothing.

The compiler behavior under test is exact base `4fb0427319b1504e1549e03ba023ac486343a743`. The request harness and corrected independent proposal-assessment census are executable at `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`; the evidence record lands in a descendant that leaves that executable code unchanged. To rerun the exact evidence revision without depending on the current checkout:

```sh
evidence_worktree=$(mktemp -d /tmp/tiler-frontier-evidence.XXXXXX)
git worktree add --detach "$evidence_worktree" bef9a39afaeb929eef99d7d43232bdc61c9b5e2a
CARGO_TARGET_DIR="$evidence_worktree/target" cargo test --manifest-path "$evidence_worktree/Cargo.toml" -p tiler-compiler --lib request_wide_physical_planning_population_is_pinned -- --nocapture
CARGO_TARGET_DIR="$evidence_worktree/spike-target" cargo run --quiet --manifest-path "$evidence_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml" -- census
git worktree remove --force "$evidence_worktree"
```

The compiler-owned governed census is a targeted crate test because the old `ProviderOffer` public surface cannot expose raw governed emissions:

```sh
CARGO_TARGET_DIR=./target cargo test -p tiler-compiler --lib request_wide_physical_planning_population_is_pinned -- --nocapture
```

## Corrected request-wide result

The old raw-outcome recommendation is withdrawn. It compiled one target per request, while the accepted authority is request-scoped and the public request admits sixteen distinct targets. The old harness comment also called 256 "above" a 272-outcome specialist population; that was false (`256 < 272`).

| Candidate | Population | Raw outcomes | Headroom | Status |
| --- | --- | ---: | ---: | --- |
| 256 | governed sixteen-target five-op | 304 | −48 | eliminated |
| 1,024 | governed + two active specialists | 848 | 176 | nondominated if 3+ active specialists are intentionally unsupported |
| 16,384 | governed + all 31 installed slots active | 8,736 | 7,648 | nondominated; idle-M3 time/RSS held |

No exact value is recommended yet. `InstalledPhysicalProviders::installed` has no count branch, and the harness successfully installs 129 identities. The separate provider-count proposal remains 32 governed-included; that bounds invocation/provenance overhead and need not promise that every provider is active on every subject. Choosing 1,024 requires an explicit two-active-specialist support boundary. Choosing 16,384 covers the complete proposed provider cardinality but requires the held idle-M3 measurement.

Intermediate powers 2,048, 4,096, and 8,192 cover 6, 13, and 29 installed specialists. No ticket or consumer names any of those populations, so they are not material choices until such a support requirement exists.

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

**Fact from source reading.** The compiler-owned production population is one impl, `GovernedPhysicalProvider`. The retained mechanical guard is narrower: it is a textual census of the exact ordinary impl spelling after excluding `tests.rs` and inline `#[cfg(test)] mod` tails, not a Rust type-system enumeration. Its perturbation feeds a syntactically valid second impl fragment through the same source scanner; it no longer appends a fabricated result. Integration-test and `#[cfg(test)]` fixtures are outside the stated production population. The retained external-provider vertical is the public-surface specialist that clones `ImplementationContext::baseline` and perturbs workgroup width, the same shape as `crates/tiler-compiler/tests/external_physical_provider.rs`.

**Fact.** Equal-cost incomparable proposals grow retained alternatives linearly here (`alternatives = extra + 1`) because only the baseline subjects receive an extra body and the selected covers have one varying region. That is a fact about this program, not a bound on Cartesian growth in general.

### Full request

| Program / providers | Targets | Raw | Emitted proposals / assessments started | Declines | Verified | Admitted / retained | Proposal / total rejections | Sort items admitted / rejected | Plan combinations / retained plans |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| five-op / governed | 16 | **304** | 48 / 48 | 256 | 48 | 48 / 48 | 0 / 256 | 48 / 256 | 32 / 32 |
| five-op / governed + feasible specialist | 16 | **576** | 96 / 96 | 480 | 96 | 96 / 96 | 0 / 480 | 96 / 480 | 96 / 96 |
| five-op / governed + infeasible specialist | 16 | **576** | 96 / 96 | 480 | 96 | 48 / 48 | 48 / 528 | 48 / 528 | 32 / 32 |
| add chain / four contract groups / governed | 16 | **248** | 24 / 24 | 224 | 24 | 24 / 24 | 0 / 224 | 24 / 224 | 24 / 24 |

The public installed specialist alone emits 17, 34, 136, and 272 outcomes at 1, 2, 8, and 16 strict targets. Its summed rendered explanation retention is 103,137, 206,274, 825,096, and 1,651,952 bytes. The four-contract add-chain rows are 10, 20, 124, and 248 outcomes and 42,545, 85,170, 514,744, and 1,030,032 rendered bytes. These are `Compilation::explain().render()` lengths, separate from compiler work counts and process RSS. In the 16-target add-chain row, eight targets evaluate one semantic candidate and eight evaluate two. Reversed target order preserves the work totals and is returned unchanged.

The retained population result is [`results/2026-08-13-request-population-census.json`](results/2026-08-13-request-population-census.json).

### Preserved draft reproduction

[`fixtures/draft_request_budget.rs`](fixtures/draft_request_budget.rs) is a public-only integration fixture for preserved draft `54e272baa525027a6f6f9d982bd3bd7c387597fb`. It does not modify the preserved branch:

```sh
draft_worktree=$(mktemp -d /tmp/tiler-frontier-draft.XXXXXX)
git worktree add --detach "$draft_worktree" 54e272baa525027a6f6f9d982bd3bd7c387597fb
cp fixtures/draft_request_budget.rs "$draft_worktree/crates/tiler-compiler/tests/"
CARGO_TARGET_DIR="$draft_worktree/target" cargo test --manifest-path "$draft_worktree/Cargo.toml" -p tiler-compiler --test draft_request_budget -- --nocapture
git worktree remove --force "$draft_worktree"
```

It reports target indexes 0–12 compiled and 13–15 refused with `BudgetExhausted { resource: PhysicalFrontierOutcomes, limit: 256, reported: 257 }`. That compiled prefix is a separate propagation defect in the draft; the calibration remains request-scoped.

## Historical single-target host-runtime record

**Measurement**, 2026-08-13, Apple M3 Pro, macOS 27.0 (26A5388g / Darwin 27.0.0), 11 logical CPUs, 18 GiB, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` — the `rust-toolchain.toml` pin `nightly-2026-07-19`. Release profile. Warm-up 8, repeats 50, estimator is the minimum. Load averages at start `{ 6.82 3.66 2.70 }` (the cargo release build had just finished on the same host); at end `{ 6.32 3.66 2.71 }`. Min/p90 on the governed row are 3205 / 3251 µs, so the floor is readable even under that load. Peak RSS is `/usr/bin/time -l` maximum resident set size of a child that warms twice and compiles once.

| Row | Extra providers | Min µs | × governed | Peak RSS | Notes |
| --- | --- | --- | --- | --- | --- |
| governed-only | 0 | 3205 | 1.00 | 25.9 MB | one alternative |
| empty 8 / 32 / 64 | 8 / 32 / 64 | 3230 / 3376 / 3561 | 1.01 / 1.05 / 1.11 | 25.8–26.0 MB | invocations only |
| decline 16 / 64 / 128 | 16 / 64 / 128 | 3684 / 5607 / 5358 | 1.15 / 1.75 / — | 27.1 / 29.6 / 27.6 MB | 128 extras → `InvalidCompilerOutput` |
| propose 1 / 8 / 16 | 1 / 8 / 16 | 3869 / 8871 / 14813 | 1.21 / 2.77 / 4.62 | 30.4 / 54.4 / 82.3 MB | alternatives 2 / 9 / 17 |
| infeasible 16 / 32 | 16 / 32 | 5549 / 7349 | 1.73 / 2.29 | 30.3 / 31.1 MB | verification without selection growth |

The retained result is [`results/2026-08-13-macos-27.0-m3-pro.json`](results/2026-08-13-macos-27.0-m3-pro.json).

### Request-wide timing hold

The request harness was not recorded on 2026-08-13 because the available machine was an active Apple M4 Max, not the required idle M3 Pro: 36 GiB, 14 cores, load `{ 4.66 3.87 3.49 }`, iTerm 24.5%, WindowServer 14.1%. [`measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro`](../../../tickets/measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro.md) owns the external prerequisite. No M4 runtime or RSS is retained as decision evidence.

## Perturbations

Each load-bearing check keeps its assertion and fails when its subject is perturbed. Restored afterwards; `census` is green.

| Perturb | Failure quoted |
| --- | --- |
| `extra-production-provider` | `FAIL source-declared-production-provider-impls expected=1 observed=2` |
| `tiny-program` | `FAIL distinct-region-subjects expected=17 observed=3` |
| `missing-observer` | `FAIL distinct-region-subjects expected=17 observed=0` |
| `silent-decline` | `FAIL many-declines-count expected=17 observed=0` |
| `decline-instead-of-propose` | `FAIL external-vertical-proposals expected=3 observed=0` |
| `one-additive-specialist` | `FAIL two-additive-provider-count expected=2 observed=1` |
| `feasible-instead-of-infeasible` | `FAIL infeasible-not-selected expected=rejected observed=selected` |
| `limit-recommendation-population` | `FAIL request-narrow-limit-calculation expected=1024 observed=2048` |
| `full-limit-population` | `FAIL request-full-provider-limit-calculation expected=16384 observed=8192` |

The crate-private assessment counter has its own retained subject perturbations:

| Perturb | Failure quoted |
| --- | --- |
| `TILER_FRONTIER_CENSUS_PERTURB=fatal-proposal-order` | `the fatal first proposal must prevent the later proposal entering assessment` — left 2, right 1 |
| `TILER_FRONTIER_CENSUS_PERTURB=proposal-body-path` | `each emitted proposal enters assessment before applicability and body dispatch` — left 2, right 3 |

Run the scanner and assessment negatives at the exact executable evidence revision with their assertions unchanged. Each perturbation command intentionally exits nonzero, so run the lines individually before cleanup:

```sh
negative_worktree=$(mktemp -d /tmp/tiler-frontier-negatives.XXXXXX)
git worktree add --detach "$negative_worktree" bef9a39afaeb929eef99d7d43232bdc61c9b5e2a
CARGO_TARGET_DIR="$negative_worktree/spike-target" cargo run --quiet --manifest-path "$negative_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml" -- perturb extra-production-provider
TILER_FRONTIER_CENSUS_PERTURB=fatal-proposal-order CARGO_TARGET_DIR="$negative_worktree/target" cargo test --manifest-path "$negative_worktree/Cargo.toml" -p tiler-compiler --lib a_fatal_proposal_leaves_later_emitted_proposals_unassessed -- --nocapture
TILER_FRONTIER_CENSUS_PERTURB=proposal-body-path CARGO_TARGET_DIR="$negative_worktree/target" cargo test --manifest-path "$negative_worktree/Cargo.toml" -p tiler-compiler --lib proposal_assessment_precedes_applicability_and_body_dispatch -- --nocapture
git worktree remove --force "$negative_worktree"
```

The request-wide compiler test also accepts a subject perturbation through `TILER_FRONTIER_CENSUS_PERTURB`:

| Perturb | Failure quoted |
| --- | --- |
| `target-count` | `the request-wide census must reach all sixteen admitted target slots` — left 15, right 16 |
| `target-order` | `the compiler must preserve caller target order in the population under test` — reversed keys against forward keys |
| `candidate-contract-population` | `the four-contract semantic-candidate population changed` — left `224/40/216` invocation/proposal/decline population against `248/24/224` |
| `governed-outcome-inclusion` | `the raw-outcome authority must include governed and installed emissions` — left 272 raw outcomes against 576 |

## Measurement boundary

The finite census covers the five-operation strict program and tensor add chain, 1/2/8/16 distinct profiles, four numerical-contract groups, target order, the governed provider, and synthetic installed specialists. It does not claim a universal program, candidate, provider, or plan population. The historical timing covers one M3 Pro and singleton requests only; request-wide timing/RSS is held. Nothing here is a portable guarantee or a kernel time. Synthetic providers exercise the public seam, not a third-party crate's native compute.

An accepted budget value directly changes the compiler-internal canonical request/evidence subject and explain request qualifier. Budget bytes do not directly enter plan, artifact, or cache identity; those identities move only indirectly if the changed bound changes selected packaged content.
