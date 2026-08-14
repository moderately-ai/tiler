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
./target/release/physical-frontier-budget-calibration request-boundary 31
./target/release/physical-frontier-budget-calibration perturb extra-production-provider
./target/release/physical-frontier-budget-calibration record /tmp/physical-frontier-request-wide-rerun.json
```

The last command is reserved for the idle M3 Pro. It now measures request-wide 1, 2, 8, and 16-target governed/specialist rows, the four-contract add chain, the governed-plus-two-specialist population, and the full 31-installed-specialist population with the same warm-up 8, repeats 50, and child-RSS protocol. `--quick` shortens warmup, repeats, and sweep points and skips `/usr/bin/time -l`. Spikes gate nothing.

The compiler behavior under test is exact base `4fb0427319b1504e1549e03ba023ac486343a743`. The request measurement harness and corrected independent proposal-assessment census are executable at `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`. The request-wide record below was run from that exact detached commit. This descendant adds only the public-spike `request-boundary` diagnostic and rendered-record observation used after measurement to locate an independent explain-capacity refusal; it is not the executable measurement revision. To rerun the exact evidence revision without depending on the current checkout:

```sh
evidence_worktree=$(mktemp -d /tmp/tiler-frontier-evidence.XXXXXX)
git worktree add --detach "$evidence_worktree" bef9a39afaeb929eef99d7d43232bdc61c9b5e2a
CARGO_TARGET_DIR="$evidence_worktree/target" cargo test --manifest-path "$evidence_worktree/Cargo.toml" -p tiler-compiler --lib request_wide_physical_planning_population_is_pinned -- --nocapture
CARGO_TARGET_DIR="$evidence_worktree/spike-target" cargo run --quiet --manifest-path "$evidence_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml" -- census
CARGO_TARGET_DIR="$evidence_worktree/spike-target" cargo build --release --manifest-path "$evidence_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml"
"$evidence_worktree/spike-target/release/physical-frontier-budget-calibration" record /tmp/physical-frontier-request-wide-rerun.json
git worktree remove --force "$evidence_worktree"
```

Run the release `record` line only after the idle/noise precheck. The retained snapshots used `sw_vers`, `uname -a`, `sysctl -n machdep.cpu.brand_string`, `sysctl -n hw.ncpu`, `sysctl -n hw.memsize`, `sysctl -n vm.loadavg`, `uptime`, `pmset -g batt`, `pmset -g therm`, `memory_pressure`, `df -h /`, and `ps -Ao pid,ppid,%cpu,%mem,state,etime,comm -r`; the same load, power, thermal, memory, filesystem, and process checks ran immediately after the record. The [pre-run](results/2026-08-13-request-wide-macos-27.0-m3-pro.environment-before.txt) and [post-run](results/2026-08-13-request-wide-macos-27.0-m3-pro.environment-after.txt) artifacts retain those outputs with only `memory_pressure`'s trailing spaces normalized.

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
| 16,384 | governed + all 31 installed slots active | 8,736 | 7,648 | eliminated under current explain authority; first target refuses at seven specialists |

No exact value is recommended yet. `InstalledPhysicalProviders::installed` has no count branch, and the harness successfully installs 129 identities. The separate provider-count proposal remains 32 governed-included; that bounds invocation/provenance overhead and need not promise that every provider is active on every subject. Choosing 1,024 still requires an explicit two-active-specialist support boundary. A raw limit of 16,384 alone cannot cover the named full-provider population: the independent complete-explain authority refuses one target at seven specialists, long before 31 specialists can emit 8,736 request-wide outcomes. Full-provider activity is now a composite option behind [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](../../../tickets/decide-how-explain-capacity-bounds-active-physical-provider-populations.md), not a surviving raw-budget value.

Intermediate powers 2,048, 4,096, and 8,192 cover 6, 13, and 29 installed specialists arithmetically. Six is also the largest active-specialist count this exact one-target subject carries under the current explain byte ceiling, but that incidental implementation boundary is not an accepted consumer population. No ticket or consumer names 6, 13, or 29 as the intended support boundary, so those powers remain non-material until such a requirement exists.

## What these limits do not bound

They do not bound arbitrary native provider computation or allocation before an emission. A provider that loops, allocates a huge `ScheduledRegion`, or builds an oversized body still does that work before the host can charge the outcome. They also do not replace `physical_plan_combinations` (4096). Raw outcomes and complete-explain capacity are different dimensions: this subject's installed emissions grow as `17n`, while retained alternatives grow as `(n + 1)(n + 2)` and rendered record lines through the last successful row grow as `39n² + 116n + 191`.

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

### Request-wide M3 Pro timing and RSS

**Measurement**, 2026-08-13 local / 2026-08-14 UTC, exact executable commit `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`, behavior base `4fb0427319b1504e1549e03ba023ac486343a743`. Apple M3 Pro, macOS 27.0 build `26A5388g`, Darwin 27.0.0, 11 logical CPUs, 18 GiB, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, release profile. Runtime rows discard eight warm-ups and summarize fifty in-process compiles. RSS is macOS `/usr/bin/time -l` maximum resident set size for a child that warms twice and compiles once.

The machine was on AC power at 100 percent battery, reported no thermal or performance warning, and had no swap I/O. Load moved from `{ 2.17 2.39 2.26 }` before the run to `{ 2.12 2.32 2.24 }` after it; the harness's intervening embedded snapshot was `{ 2.22 2.39 2.26 }`. Free-memory percentage was 73 then 72. No Cargo, rustc, make, nextest, or measurement binary appears in the process-name snapshot; the retained `pgrep` lines are the snapshot shell and a long-lived Node process whose argv or environment contains a search token, not competing builds. Apart from the observing SSH session, the highest pre-run process was `opendirectoryd` at 4.9 percent CPU; the highest post-run processes were Tailscale and `opendirectoryd` at 3.5 percent. The earlier build was allowed to settle before these snapshots.

| Row | Targets | Installed outcomes | Alternatives / failure | Min / median / p90 / max / mean µs | Peak RSS bytes |
| --- | ---: | ---: | --- | --- | ---: |
| five-op, governed | 1 | 0 | 2 | 3,857 / 3,869 / 3,878 / 3,901 / 3,870 | 29,687,808 |
| five-op, one specialist | 1 | 17 | 6 | 6,577 / 6,597 / 6,617 / 6,627 / 6,599 | 40,779,776 |
| add chain, four groups | 1 | 10 | 2 | 3,041 / 3,050 / 3,057 / 3,060 / 3,050 | 25,690,112 |
| five-op, governed | 2 | 0 | 4 | 7,747 / 7,763 / 7,787 / 7,808 / 7,766 | 39,813,120 |
| five-op, one specialist | 2 | 34 | 12 | 13,244 / 13,276 / 13,298 / 13,309 / 13,278 | 59,080,704 |
| add chain, four groups | 2 | 20 | 4 | 6,103 / 6,120 / 6,134 / 6,142 / 6,121 | 30,425,088 |
| five-op, governed | 8 | 0 | 16 | 31,486 / 31,537 / 31,581 / 32,266 / 31,555 | 93,716,480 |
| five-op, one specialist | 8 | 136 | 48 | 54,048 / 54,100 / 54,172 / 54,215 / 54,112 | 166,674,432 |
| add chain, four groups | 8 | 124 | 24 | 35,017 / 35,056 / 35,085 / 35,185 / 35,065 | 88,195,072 |
| five-op, governed | 16 | 0 | 32 | 63,418 / 63,483 / 63,555 / 64,080 / 63,502 | 165,855,232 |
| five-op, one specialist | 16 | 272 | 96 | 108,601 / 108,698 / 108,853 / 110,835 / 108,767 | 309,837,824 |
| five-op, two specialists | 16 | 544 | 192 | 177,748 / 178,847 / 179,403 / 185,961 / 179,058 | 531,202,048 |
| five-op, 31 specialists | 16 requested | 527 reached | `InvalidCompilerOutput` on target 1 | 49,507 / 49,621 / 49,761 / 51,084 / 49,665 | 788,185,088 |
| add chain, four groups | 16 | 248 | 48 | 70,420 / 70,496 / 70,558 / 70,688 / 70,505 | 156,270,592 |

“Installed outcomes” excludes governed emissions because the public provider tally observes only caller-installed providers; the independent compiler census gives 304 governed outcomes and therefore 848 total raw outcomes for the successful two-specialist request. The 31-specialist row is refusal timing and RSS, not a measurement of the named 8,736-outcome population. It reached one target's 527 installed outcomes (93 proposals and 434 declines) before complete explain construction refused.

The annotated record is [`results/2026-08-13-request-wide-macos-27.0-m3-pro.json`](results/2026-08-13-request-wide-macos-27.0-m3-pro.json); the harness-generated JSON is retained byte-for-byte in [`results/2026-08-13-request-wide-macos-27.0-m3-pro.generated.json`](results/2026-08-13-request-wide-macos-27.0-m3-pro.generated.json). Retained raw files are [stdout](results/2026-08-13-request-wide-macos-27.0-m3-pro.stdout.txt), [pre-run environment](results/2026-08-13-request-wide-macos-27.0-m3-pro.environment-before.txt), [post-run environment](results/2026-08-13-request-wide-macos-27.0-m3-pro.environment-after.txt), [green baselines](results/2026-08-13-request-wide-macos-27.0-m3-pro.baselines.txt), [compiler perturbations](results/2026-08-13-request-wide-macos-27.0-m3-pro.compiler-negatives.txt), and [candidate-calculation perturbations](results/2026-08-13-request-wide-macos-27.0-m3-pro.spike-negatives.txt). The generated rationale's `propose_per_outcome_ns=0` is not evidence: the unchanged helper selects a request-add minimum below the singleton governed floor and saturates that mixed-population subtraction to zero.

### Explain-capacity boundary control

`request-boundary 31` holds the one-target five-operation strict subject fixed and varies only the number of installed specialists. Six specialists succeed with 102 installed outcomes, 56 alternatives, 2,291 rendered record lines, and 650,099 rendered bytes. Seven emit 119 installed outcomes and fail with the exact final retained line:

```text
2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:b489b9770d000255/region:0 event=compiler-failure:explain-detail-capacity causes=2256
```

**Fact.** Source anchor `let exceeds = if terminal` in `ExplainWriter::push` gives two non-terminal bounds: 4,096 detail records and 1 MiB of canonical detail bytes. Each declined strategy contributes a detail record under `for rejection in frontier.rejections()`, while complete-plan explanation grows with the Cartesian plan population.

**Fact.** `DeterministicBudgets` has no physical-provider raw-outcome field at the behavior base. `16,384` is a calibration candidate rather than an installed authority, so it cannot fire on this compile path; the preserved 256-outcome draft is read-only evidence on a different commit.

**Inference.** Explain record IDs are zero-based (`local` is minted from `self.records.len()`), so terminal ordinal 2,257 and 2,258 rendered record lines mean 2,257 detail records had been retained. The record-count arm therefore did not fire. `explain-detail-capacity` identifies the non-terminal disjunction; eliminating its record-count arm leaves the canonical byte ceiling as the first governing authority. [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](../../../tickets/decide-how-explain-capacity-bounds-active-physical-provider-populations.md) now owns the decision to retain, widen, or compact that independent authority.

The full [1-through-31 control](results/2026-08-13-request-wide-macos-27.0-m3-pro.boundary-full.txt) and a separate [six/seven first-failure control](results/2026-08-13-request-wide-macos-27.0-m3-pro.boundary-first-failure.txt) retain the exact public outputs.

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

The idle-M3 rerun retained the exact green compiler/spike baselines and all six closing-condition negatives in the linked raw files above. The four compiler perturbations exit 101 at their unchanged assertions; the two calculation perturbations exit 1 after the other 41 census checks remain green.

## Measurement boundary

The finite census covers the five-operation strict program and tensor add chain, 1/2/8/16 distinct profiles, four numerical-contract groups, target order, the governed provider, and synthetic installed specialists. The valid request-wide timing/RSS rows cover governed, one-specialist, two-specialist, and four-contract add-chain populations on one M3 Pro. The attempted 31-specialist row measures only the existing explain-capacity refusal and does not reach all targets or 8,736 raw outcomes. Nothing here claims a universal program, candidate, provider, or plan population, a portable guarantee, or kernel time. Synthetic providers exercise the public seam, not a third-party crate's native compute.

An accepted budget value directly changes the compiler-internal canonical request/evidence subject and explain request qualifier. Budget bytes do not directly enter plan, artifact, or cache identity; those identities move only indirectly if the changed bound changes selected packaged content.
