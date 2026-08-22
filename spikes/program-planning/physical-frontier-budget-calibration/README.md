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
last_verified: "2026-08-16"
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
python3 ../../scheduling/metal_contraction_tile_width/tile_width_sweep.py --mode gate \
  && ./target/release/physical-frontier-budget-calibration record /tmp/physical-frontier-request-wide-rerun.json
./target/release/physical-frontier-budget-calibration export-raw /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
./target/release/physical-frontier-budget-calibration annotate-record /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.annotated.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
./target/release/physical-frontier-budget-calibration verify-evidence /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.annotated.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
```

The `record` command is reserved for the idle M3 Pro. It measures request-wide 1, 2, 8, and 16-target governed/specialist rows, the four-contract add chain, the governed-plus-two-specialist population, and the full 31-installed-specialist population with the same warm-up 8, repeats 50, and child-RSS protocol. `--quick` shortens warmup, repeats, and sweep points and skips `/usr/bin/time -l`. Spikes gate nothing.

The compiler behavior under test is exact base `4fb0427319b1504e1549e03ba023ac486343a743`. The retained workload and corrected independent proposal-assessment census are exact at `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`; exact executable evidence commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` adds raw custody while restoring `program.rs`, `profile.rs`, and `providers.rs` to the exact prior blobs. Its separate `boundary.rs` observer has one production call under `request_boundary`; the retained unit check refuses a diagnostic call from record or RSS-child paths. To rerun the exact evidence revision without depending on the current checkout:

```sh
evidence_worktree=$(mktemp -d /tmp/tiler-frontier-evidence.XXXXXX)
git worktree add --detach "$evidence_worktree" d086fe9953a09a1a8a64dbd2353e9ded78ef18e6
CARGO_TARGET_DIR="$evidence_worktree/target" cargo test --manifest-path "$evidence_worktree/Cargo.toml" -p tiler-compiler --lib request_wide_physical_planning_population_is_pinned -- --nocapture
CARGO_TARGET_DIR="$evidence_worktree/spike-target" cargo run --quiet --manifest-path "$evidence_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml" -- census
CARGO_TARGET_DIR="$evidence_worktree/spike-target" cargo build --release --manifest-path "$evidence_worktree/spikes/program-planning/physical-frontier-budget-calibration/Cargo.toml"
# The probe comes from the current checkout: it postdates d086fe99.
python3 ../../scheduling/metal_contraction_tile_width/tile_width_sweep.py --mode gate \
  && "$evidence_worktree/spike-target/release/physical-frontier-budget-calibration" record /tmp/physical-frontier-request-wide-rerun.json
"$evidence_worktree/spike-target/release/physical-frontier-budget-calibration" export-raw /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
"$evidence_worktree/spike-target/release/physical-frontier-budget-calibration" annotate-record /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.annotated.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
"$evidence_worktree/spike-target/release/physical-frontier-budget-calibration" verify-evidence /tmp/physical-frontier-request-wide-rerun.json /tmp/physical-frontier-request-wide-rerun.annotated.json /tmp/physical-frontier-request-wide-rerun.timings.tsv /tmp/physical-frontier-request-wide-rerun.rss.jsonl
git worktree remove --force "$evidence_worktree"
```

Run the release `record` line only behind the quiet-host gate stated in [Host precondition for a timed rerun](#host-precondition-for-a-timed-rerun) below, which is what the phrase *idle/noise precheck* used to gesture at without defining. The retained snapshots used `sw_vers`, `uname -a`, `sysctl -n machdep.cpu.brand_string`, `sysctl -n hw.ncpu`, `sysctl -n hw.memsize`, `sysctl -n vm.loadavg`, `uptime`, `pmset -g batt`, `pmset -g therm`, `memory_pressure`, `df -h /`, and `ps -Ao pid,ppid,%cpu,%mem,state,etime,comm -r`; the same load, power, thermal, memory, filesystem, and process checks ran immediately after the record. The [pre-run](results/2026-08-14-request-wide-macos-27.0-m3-pro.environment-before.txt) and [post-run](results/2026-08-14-request-wide-macos-27.0-m3-pro.environment-after.txt) artifacts retain those outputs with only `memory_pressure`'s trailing spaces normalized.

## Host precondition for a timed rerun

The only command here that reads a wall clock is `record`, and it is gated on the quiet-host probe below. Until 2026-08-22 that precondition existed but was never written down: it was applied by judgement, run by run, under a phrase — *the idle/noise precheck* — that named no threshold, no probe, and nothing a rerun could apply. The retained numbers are sound. What was missing is the rule deciding which host they may be taken on, and without it a rerun cannot reproduce the admission decision, so the numbers are not reproducible in the sense `AGENTS.md` requires of a measurement. This section is that rule. It changes no measured value and re-runs nothing.

**The rule is adopted, not written.** It is the gate the contraction tile-width lane derived and froze, unchanged in every component and every threshold: [its pre-run amendment](../../scheduling/metal_contraction_tile_width/PROTOCOL-2026-08-22-contraction-tile-width.md) states it and `spikes/scheduling/metal_contraction_tile_width/tile_width_sweep.py "def quiet_host_gate"` implements it. Writing a second rule here was the alternative and is rejected. Two quiet-host rules in one repository drift, and the drift is silent because `spikes/` sits outside every repository gate, so a copied threshold that falls out of date keeps reading green while admitting a contaminated host. Invoking the other lane's file instead fails **loudly**: a moved file, a renamed mode, or a Python that will not parse is a nonzero exit, and this gate reads a nonzero exit as a refusal. That asymmetry — silent divergence against loud absence — is the whole argument for adoption, and it is why the probe is invoked rather than vendored.

**No component was added for this subject, and that was decided on evidence rather than economy.** The four are mean CPU idle at or above 95% over ten one-second samples, GPU device utilization at or below 5%, a one-minute load average at or below a baseline-relative 3.5, and an exclusive advisory lock; each fails closed, so a probe whose input cannot be read refuses instead of passing. This spike measures Tiler host runtime and process RSS rather than kernel time, so the contended device is the CPU, not the GPU — which invites both dropping the GPU component and adding the dimensions this spike's own snapshots already record: AC power, thermal state, memory pressure, and a concurrent-build-process count. Both moves are refused. The single refusal this spike ever actually made turned on the load average and on a Chrome renderer's CPU, and the CPU-idle component discriminates both directly and earlier; power, thermal, and swap were all clean at that same refusal, so promoting them to gate components would write down a rule *stricter* than the one that was applied, which is a different repair than the one this record needs. A concurrent-build-process count is worse than redundant: an active `cargo` or `rustc` already drives CPU idle below the floor, and a name-matched count is the shape the sibling lane rejected by name, because it admits whatever the pattern misses and reads green forever once the pattern rots. The narrower in-repository precedent at `spikes/program-planning/reduction-partition-calibration/src/main.rs "fn concurrent_build_processes"` stays correct for its own study and is deliberately not generalized here. The GPU component is retained unchanged despite not being this subject's contended device: on a unified-memory part a busy GPU still competes for the memory system an RSS-sensitive compile measurement runs in, it costs nothing, it fails closed, and dropping one component is precisely how the second rule would begin.

**The command, and the fail-closed contract.** From this directory, in a checkout new enough to contain the probe:

```sh
python3 ../../scheduling/metal_contraction_tile_width/tile_width_sweep.py --mode gate
```

Exit 0 admits, exit 1 refuses and prints which component refused and why; the probe dispatches nothing and reads no wall clock. Chain it with `&&` rather than running it as a separate step, so a refusal *stops* the measurement instead of being read and ignored — the chaining is what makes this a gate rather than a note, and both command blocks above are written that way. The historical-evidence block chains the probe from the **current** checkout, because commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` predates it. That is also why the precondition is an external probe rather than a check inside the binary: this record has two rerun paths, one through the current executable and one through a detached worktree at that commit, and a gate compiled into either binary would not reach the other path at all. A precondition that cannot reach half its subject is the failure class this repository has already been bitten by.

**Two consequences of adopting another lane's file, stated rather than discovered.** The probe's `main()` refuses outright when `xcrun` is absent, because its other modes compile Metal. That check is irrelevant to this spike, which needs only `rustc`, so on a host without an Xcode command-line toolchain the gate refuses for a reason that has nothing to do with host quiet. It fails in the safe direction and the bench host has the toolchain, but a reader who sees that refusal should not go looking for competing work. Second, the probe was exercised here under the bench host's `Python 3.9.6`, not under a newer interpreter, so that is the version this precondition is known to run on.


**What it would take for this gate to say no, and confirmation that case is reachable.** Any one of the four components refusing is enough, and all four were driven to refuse on this bench host by the lane that derived them, with each failure quoted in [the tile-width record](../../scheduling/metal_contraction_tile_width/README.md#the-quiet-host-gate-and-both-directions-of-it-watched). Those perturbations are evidence about *this* gate and not merely about a similar one: the probe file was verified byte-identical between the two hosts before the reads below, at SHA-256 `1815079e7495df2f16d9ecb6d8af4fbc9bc66be5ec5f1f6f081e8f8514db9ff0`. What is new here, and what is specific to this record, is whether a refusal actually stops `record` in the chained form written above. Both directions of that chain were watched on 2026-08-22 on the bench host, with `echo` standing in for `record` so that no wall clock was read and no calibration timing was re-run.

**It admits a quiet host, and the chained command proceeds.** Quoted unedited:

```text
== quiet-host gate ==
   host Thomass-MacBook-Pro, 11 cores, Apple M3 Pro
  CPU idle, mean of 10 x 1 s    98.64%  (floor 95.0%, min sample 97.81%)  pass
  GPU device utilization              0%  (ceiling 5%)  pass
  load average, one minute         1.88   (ceiling 3.5, recorded idle baseline one-minute 1.86-2.47 over 20+ observations 2026-08-22; retained in-tree at spikes/program-planning/physical-frontier-budget-calibration/results/ as { 2.22 2.39 2.26 } on 2026-08-13 and { 2.18 2.23 2.24 } on 2026-08-14, same host)  pass
  exclusive measurement lock     /tmp/tiler-contraction-tile-width-sweep.lock  held

verdict: ADMIT -- a timing run may start now.
NO WALL CLOCK WAS READ. This mode makes no timing claim of any kind.
RECORD WOULD START (record stand-in; no timing was run)
chain-exit=0
```

**It refuses under real competing work, and the chained command does not start.** Four bounded busy loops were started on the bench host and the same chain read twenty-five seconds later. The `RECORD WOULD START` line is absent, which is the property being demonstrated:

```text
  CPU idle, mean of 10 x 1 s    53.03%  (floor 95.0%, min sample 48.13%)  REFUSE
  GPU device utilization              0%  (ceiling 5%)  pass
  load average, one minute         3.71   (ceiling 3.5, ...)  REFUSE
  exclusive measurement lock     /tmp/tiler-contraction-tile-width-sweep.lock  held

REFUSED: CPU idle averaged 53.03% over 10 s, below the 95.0% floor. Work is competing for CPU, and on a unified-memory device that competes for bandwidth with the dispatch being timed.
REFUSED: One-minute load average is 3.71, above the 3.5 ceiling. This ceiling is relative to this host's recorded idle baseline of ..., not an absolute quiet figure.
chain-exit=1
```

The subject was perturbed in both cases and no assertion was edited. The busy loops self-terminated after 150 seconds and were confirmed gone; where a long constant is elided above it is marked `...`, and no line is combined from a different read.

**An independent confirmation of the load average's demotion, from the opposite side.** The tile-width lane demoted the one-minute load average to a lagging cross-check on evidence that it lags on the way *up*: twenty-five seconds into competing work, CPU idle had fallen to 61.17% while load still read 2.89. The recovery read here shows the same lag on the way *down*. Immediately after those busy loops ended, CPU idle had already returned to `98.29%` while the one-minute load average still read `4.06` and refused the run. The instrument lags in both directions, which is a stronger reason to keep it as a cross-check and not as the discriminator than either observation alone.

**One consequence of writing the rule down, stated rather than left for a reader to hit.** This gate admits the run the current result rests on and refuses one it does not. The 2026-08-14 request-wide record was taken at load `{ 2.18 2.23 2.24 }` falling to `{ 1.61 2.07 2.17 }`, with the highest non-observing process at 5.0 percent CPU: inside every component. The [historical single-target record](#historical-single-target-host-runtime-record) below was taken at load `{ 6.82 3.66 2.70 }` and `{ 6.32 3.66 2.71 }`, which this gate refuses at start and at end. That row is already labelled historical and is superseded by the corrected request-wide result, and nothing here re-runs it, withdraws a number, or changes a value — but a rerun of it under this precondition would refuse, and a reader is entitled to know that before treating it as reproducible. An unwritten gate can never disagree with a retained run, which is exactly why it could not be relied on.

**Before a rerun, quiesce the interactive session.** The bench host carries a permanent console login whose foreground applications produce episodic bursts of roughly one core, and the gate refuses during them, correctly. Across ten arbitrary reads on 2026-08-22 while that session was live, six admitted and four refused; every read taken is counted here, including the ones that refused, and the perturbed read and the burner-recovery read are excluded because their load was induced. Quit the foreground applications before the run rather than re-reading the gate until it passes. Re-reading until green selects for the quiet phase of a host that is not quiet, which is the contamination this gate exists to exclude; where several reads are genuinely needed, fix the count in advance and report every verdict, which is how the six-and-four above were obtained. Quiescing a user session is an operator action. Nothing here authorizes disabling the system extensions that carry this host's load-average floor: they are the machine's networking, and that is Tom's decision.

**Work record:** [`write-the-frontier-calibration-s-unwritten-quiet-host-gate`](../../../tickets/write-the-frontier-calibration-s-unwritten-quiet-host-gate.md).

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

No exact value is recommended. `InstalledPhysicalProviders::installed` has no count branch, and the harness successfully installs 129 identities. The separate provider-count proposal remains 32 governed-included; that bounds invocation/provenance overhead and does not promise that every provider is active on every subject. Choosing 1,024 still requires an explicit two-active-specialist support boundary. A raw limit of 16,384 alone cannot cover the named full-provider population: the independent complete-explain authority refuses one target at seven specialists, long before 31 specialists can emit 8,736 request-wide outcomes. [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](../../../tickets/decide-how-explain-capacity-bounds-active-physical-provider-populations.md) now retains both explain ceilings with a truthful atomic capacity refusal and no numeric active-provider guarantee. Full-provider activity is therefore a deferred expansion triggered by a named consumer or accepted contract, not a surviving raw-budget value or a current Tom question.

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

**Measurement**, 2026-08-14, exact executable commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`, behavior base `4fb0427319b1504e1549e03ba023ac486343a743`. Apple M3 Pro, macOS 27.0 build `26A5388g`, Darwin 27.0.0, 11 logical CPUs, 18 GiB, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, release profile. Every runtime row retains all fifty integer-nanosecond observations after eight discarded warm-ups; published microseconds are independently recomputed with upper median `n/2`, p90 `(9n−1)/10`, and floor truncation. Every RSS row retains the complete stderr and exit status from one macOS `/usr/bin/time -l` child that warms twice and compiles once.

The machine was on AC power at 100 percent battery, reported no thermal or performance warning, and had no swap I/O. Three immediately preceding controls were clean. Formal load moved from `{ 2.18 2.23 2.24 }` to `{ 1.61 2.07 2.17 }`; free-memory percentage stayed 72. Apart from the observing SSH session, the highest pre-run process was `opendirectoryd` at 5.0 percent CPU and the highest post-run process was `launchd` at 4.5 percent. No Cargo, rustc, make, nextest, or competing measurement process appears in either snapshot; the retained Chrome renderer/helper rows are at 0.0 percent CPU and the Chrome application row peaks at 0.1 percent.

| Row | Targets | Installed outcomes | Alternatives / failure | Min / median / p90 / max / mean µs | Peak RSS bytes |
| --- | ---: | ---: | --- | --- | ---: |
| five-op, governed | 1 | 0 | 2 | 3,853 / 3,866 / 3,876 / 3,883 / 3,866 | 29,671,424 |
| five-op, one specialist | 1 | 17 | 6 | 6,579 / 6,599 / 6,609 / 6,620 / 6,599 | 41,009,152 |
| add chain, four groups | 1 | 10 | 2 | 3,042 / 3,052 / 3,058 / 3,077 / 3,052 | 25,755,648 |
| five-op, governed | 2 | 0 | 4 | 7,750 / 7,766 / 7,779 / 7,798 / 7,766 | 39,862,272 |
| five-op, one specialist | 2 | 34 | 12 | 13,274 / 13,303 / 13,323 / 13,364 / 13,304 | 59,277,312 |
| add chain, four groups | 2 | 20 | 4 | 6,101 / 6,117 / 6,127 / 6,137 / 6,117 | 30,490,624 |
| five-op, governed | 8 | 0 | 16 | 31,508 / 31,547 / 31,597 / 31,720 / 31,557 | 93,847,552 |
| five-op, one specialist | 8 | 136 | 48 | 54,158 / 54,375 / 55,783 / 61,352 / 54,737 | 166,821,888 |
| add chain, four groups | 8 | 124 | 24 | 34,998 / 35,046 / 35,074 / 35,077 / 35,045 | 88,326,144 |
| five-op, governed | 16 | 0 | 32 | 63,414 / 63,483 / 63,532 / 63,557 / 63,487 | 165,855,232 |
| five-op, one specialist | 16 | 272 | 96 | 108,880 / 109,014 / 109,138 / 110,925 / 109,061 | 310,018,048 |
| five-op, two specialists | 16 | 544 | 192 | 177,123 / 177,303 / 177,396 / 177,545 / 177,305 | 531,693,568 |
| five-op, 31 specialists | 16 requested | 527 reached | `InvalidCompilerOutput` on target 1 | 49,508 / 49,610 / 49,728 / 49,803 / 49,619 | 788,283,392 |
| add chain, four groups | 16 | 248 | 48 | 70,540 / 70,614 / 70,716 / 71,045 / 70,645 | 156,467,200 |

“Installed outcomes” excludes governed emissions because the public provider tally observes only caller-installed providers; the independent compiler census gives 304 governed outcomes and therefore 848 total raw outcomes for the successful two-specialist request. The 31-specialist row remains refusal timing and RSS, not a measurement of the named 8,736-outcome population: it reached one target's 527 installed outcomes (93 proposals and 434 declines) before complete explain construction refused. The 8-target specialist row's 61,352 µs maximum is retained rather than discarded; its median is 54,375 µs, so readers can see the isolated upper-tail disturbance without mistaking the minimum envelope for a complete distribution.

The live [annotated record](results/2026-08-14-request-wide-macos-27.0-m3-pro.json) is an exact semantic copy of the [generated JSON](results/2026-08-14-request-wide-macos-27.0-m3-pro.generated.json) plus its evidence annotation. Its SHA-256 values are `ec3abc4e…76c41` for generated JSON, `ebfb9015…44ce` for the ordered [timing TSV](results/2026-08-14-request-wide-macos-27.0-m3-pro.timings.tsv), and `8d8146be…cf78` for the complete [RSS JSONL](results/2026-08-14-request-wide-macos-27.0-m3-pro.rss.jsonl). The deterministic verifier recomputes all 2250 timings and reparses all 45 RSS rows. Retained supporting artifacts are [stdout](results/2026-08-14-request-wide-macos-27.0-m3-pro.stdout.txt), [stderr](results/2026-08-14-request-wide-macos-27.0-m3-pro.stderr.txt), [numeric exit status](results/2026-08-14-request-wide-macos-27.0-m3-pro.exit-status.txt), [pre-run environment](results/2026-08-14-request-wide-macos-27.0-m3-pro.environment-before.txt), [post-run environment](results/2026-08-14-request-wide-macos-27.0-m3-pro.environment-after.txt), [subject equivalence](results/2026-08-14-request-wide-macos-27.0-m3-pro.subject-equivalence.txt), [green baselines](results/2026-08-14-request-wide-macos-27.0-m3-pro.baselines.txt), [compiler perturbations](results/2026-08-14-request-wide-macos-27.0-m3-pro.compiler-negatives.txt), [spike perturbations](results/2026-08-14-request-wide-macos-27.0-m3-pro.spike-negatives.txt), and [custody perturbations](results/2026-08-14-request-wide-macos-27.0-m3-pro.custody-negatives.txt).

The [2026-08-13 annotated record](results/2026-08-13-request-wide-macos-27.0-m3-pro.json) and byte-for-byte generated JSON remain as withdrawn, non-custodial history: they omit the ordered timings and full RSS child records. A 00:50 idle precheck was held before launching because Chrome used 82 percent CPU. A later `981ddf7f…` run was rejected because exact diffing found a pre-existing explain-line observer inside its timed summarization path. The first `d086fe99…` launch completed, but its controlling SSH session reported completion eight seconds early and caused the nominal post snapshot to overlap the measurement. None of those attempts supplies live numbers. The final record used a detached process, an atomic exit-status marker written after its post snapshot, and read-only polling.

The generated rationale's `propose_per_outcome_ns=0` is not evidence: the unchanged helper selects a request-add minimum below the singleton governed floor and saturates that mixed-population subtraction to zero.

### Explain-capacity boundary control

`request-boundary 31` holds the one-target five-operation strict subject fixed and varies only the number of installed specialists. Six specialists succeed with 102 installed outcomes, 56 alternatives, 2,291 rendered record lines, and 650,099 rendered bytes. A current-toolchain reproduction on 2026-08-16 at audited base `b757936b6620620a321c3c9ba43ec75ca4376599` (where implementation commit `b474fd01e339396ab7779c6ae0ae9e58631a7856` is ancestral) used the repository's `nightly-2026-07-19` pin. Seven specialists emit 119 installed outcomes and fail with the exact current final retained line:

```text
2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:f10d1b8bfd323115/region:0 event=compiler-failure:explain-detail-capacity causes=2256
```

The linked 2026-08-13 raw boundary artifacts were produced by the historical `d086fe99…` executable and retain their earlier `b489b9770d000255` digest unchanged. They remain rerunnable provenance for that dated toolchain rather than the current digest authority.

**Fact.** Source anchor `let capacity = if terminal` in `ExplainWriter::push` gives two non-terminal bounds: 4,096 detail records and 1 MiB of canonical detail bytes. Each declined strategy contributes a detail record under `for rejection in frontier.rejections()`, while complete-plan explanation grows with the Cartesian plan population.

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
git worktree add --detach "$negative_worktree" d086fe9953a09a1a8a64dbd2353e9ded78ef18e6
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

Custody checks perturb the retained subjects while leaving the verifier unchanged:

| Perturb | Failure quoted |
| --- | --- |
| first ordered duration set to zero | `FAIL custody governed-only.min_us expected=0 observed=3197` |
| complete RSS stderr maximum changed | `FAIL custody governed-only.rss.parsed_peak_rss_bytes expected=1 observed=25886720` |
| RSS child subcommand changed | `FAIL custody governed-only RSS command subject mismatch: expected=["child-measure", "governed-only", "0", "empty"] observed=["child-request-measure", "governed-only", "0", "empty"]` |
| duplicate maximum-RSS line | `FAIL custody governed-only retained time stderr has 2 maximum RSS lines` |
| one raw TSV duration changed | `FAIL custody evidence raw timing artifact does not match generated record` |
| one annotated measurement field changed | `FAIL custody evidence annotated measurement fields differ from generated record` |

## Measurement boundary

The finite census covers the five-operation strict program and tensor add chain, 1/2/8/16 distinct profiles, four numerical-contract groups, target order, the governed provider, and synthetic installed specialists. The valid request-wide timing/RSS rows cover governed, one-specialist, two-specialist, and four-contract add-chain populations on one M3 Pro. The attempted 31-specialist row measures only the existing explain-capacity refusal and does not reach all targets or 8,736 raw outcomes. Nothing here claims a universal program, candidate, provider, or plan population, a portable guarantee, or kernel time. Synthetic providers exercise the public seam, not a third-party crate's native compute.

An accepted budget value directly changes the compiler-internal canonical request/evidence subject and explain request qualifier. Budget bytes do not directly enter plan, artifact, or cache identity; those identities move only indirectly if the changed bound changes selected packaged content.
