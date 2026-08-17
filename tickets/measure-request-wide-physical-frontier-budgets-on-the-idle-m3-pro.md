---
id: measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro
title: Measure request-wide physical-frontier budgets on the idle M3 Pro
status: todo
priority: p1
dependencies: []
related: [calibrate-the-physical-frontier-provider-and-outcome-budgets]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [optimizer, budgets, measurement, host-performance]
---
## Outcome

Measure the retained physical-frontier harness on the idle Apple M3 Pro under the repository performance protocol. Compiler behavior under test is exact base `4fb0427319b1504e1549e03ba023ac486343a743`; the retained workload and corrected independent proposal-assessment counter are exact commit `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`, and exact executable evidence commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` adds raw timing/RSS custody while restoring the timed `program.rs` blob exactly and isolating the later explain-capacity observer outside every record and RSS-child path. Record identical release-profile warm-up 8, repeats 50, minimum/median/p90/maximum/mean, child-process peak RSS, load/noise controls, toolchain, OS, and both commits for the 1, 2, 8, and 16-target five-operation governed and installed-specialist requests plus the four-contract add-chain population. Include the 1,024 narrow-population candidate and the 16,384 full-32-provider-activity candidate in the host-runtime and memory comparison. Do not change host or toolchain components.

## Why this is separate

The available host on 2026-08-13 was an active Apple M4 Max, not the required idle M3 Pro: load was { 4.66 3.87 3.49 }, with iTerm at 24.5 percent and WindowServer at 14.1 percent. Publishing those numbers would contaminate the decision. The durable census and timing harness live in the calibration spike; this ticket supplies only the externally held environment evidence.

## Closing conditions

- Run the documented detached-worktree retained-spike request measurement at executable evidence commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` without changing the `bef9a39…` subjects or controls, and record behavior base `4fb0427319b1504e1549e03ba023ac486343a743` separately from the executable commit.
- Record all raw output, runtime, RSS, host, noise, and failure rows needed to compare 1,024 with 16,384.
- Perturb target count, target order, candidate-contract population, governed-outcome inclusion, and both candidate calculations with assertions unchanged; quote each failure.
- Update the calibration research note and ticket with the exact record and measurement boundary.
- Run the spike checks, tkt lint, make citations, git diff --check, and tkt guard.

## Exact-base Fact audit at `b2ab50f278616a1ad8f171184a16d60ae7e608ff`

- **Verified.** The checkout, live claim, and scopes match this ticket. Commands: `git rev-parse HEAD`, `git status --porcelain=v1`, `tkt show measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro --format json`, and `tkt claims --format json`.
- **Verified.** The behavior-under-test base is `4fb0427319b1504e1549e03ba023ac486343a743`; changes through the executable evidence revision are spike code and `#[cfg(test)]` compiler census instrumentation rather than a production behavior change.
- **Verified at the exact base; superseded for live custody.** `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a` was the historical request-wide executable revision. At this ticket's base, `main.rs`, `measure.rs`, `profile.rs`, `program.rs`, and `providers.rs` were byte-identical to it; `census.rs` differed only by three comment lines changing “valid” to “syntactically valid”. The 2026-08-13 request-wide measurement ran from a clean detached exact `bef9a39…` worktree, but review later withdrew it as non-custodial. The live replacement is separately identified below.
- **Verified.** [`2026-08-13-macos-27.0-m3-pro.json`](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-macos-27.0-m3-pro.json) is the historical singleton-request record introduced by `a5a9487e60b5eb385ae9abc684b01a3f894a3f5e`. The spike's `Request-wide timing hold` still said the new record was absent at this base.
- **Verified as retained historical evidence.** The earlier hold records an active Apple M4 Max at load `{ 4.66 3.87 3.49 }`, iTerm 24.5 percent, and WindowServer 14.1 percent. It is a past observation, not a live host state.
- **Verified.** `record`, `request_counts`, and `(8, 50, true)` in spike `main.rs` cover 1/2/8/16 governed, installed-specialist, and four-contract add-chain requests plus the 16-target two- and 31-specialist rows.
- **Verified.** `summarize` reports minimum, median, p90, maximum, and mean. `child_request_peak_rss` uses macOS `/usr/bin/time -l`; the child warms twice and compiles once.
- **Imprecise, made explicit in the record.** “Same warm-up 8 … and child-RSS protocol” does not mean eight child warm-ups. Runtime uses eight discarded warm-ups and fifty samples; RSS uses the distinct two-warm-up/one-compile child protocol.
- **Verified with an external recording obligation.** `host_record` captures commit, toolchain, OS, CPU, memory, load, and uptime. It does not capture pre/post process noise, thermal/power state, or post-run load, so separate retained environment snapshots discharge those conditions.
- **Verified.** The compiler census accepts unchanged-assertion perturbations for target count, target order, candidate-contract population, governed inclusion, fatal proposal order, and proposal body/applicability. Spike perturbations cover both candidate calculations and the source scanner.
- **Verified.** No production budget value or public seam is authorized by this ticket.

No false premise changed this ticket's purpose, so measurement proceeded.

## Custody-repair Fact impact at `cec724e3`

- **False for independent recomputation.** The retained 2026-08-13 request-wide summaries and parsed RSS values were observations, but they were not sufficient evidence for their claimed distributions: the artifacts omitted all ordered duration samples, complete time(1) stderr, and child status. This changed the evidence-custody obligation, not the workload, candidate arithmetic, or ticket purpose.
- **Verified.** The 45-row manifest, per-row warm-up 8/repeats 50, child two-warm-up/one-compile protocol, and compiler behavior base remained the intended subjects. The repair had to retain those subjects unchanged while adding custody outside the timed workload.
- **Verified.** Exact reconstruction requires integer nanoseconds in sample execution order, upper median index `n/2`, p90 index `(9n−1)/10`, floor-truncated microseconds, complete child stderr/status, and refusal of absent or duplicate maximum-RSS lines.
- **Verified.** The annotation must identify behavior base and executable evidence commit separately, retain SHA-256 for generated and raw artifacts, and compare annotated measurement fields semantically with generated output.
- **Verified.** The active explain-capacity dependency, the absence of authority for a two-specialist policy, and the prohibition on choosing a production budget or public seam were unchanged. The ticket therefore remains unfinished and returns to `todo` behind that dependency after the fresh custodial measurement.

## Custody repair and 2026-08-14 idle precheck

**Fact.** Review found that the 2026-08-13 request-wide artifacts retain only five timing summaries per row and one parsed RSS integer: they omit every ordered duration, complete `/usr/bin/time -l` stderr, and the child exit status. Those distributions and RSS values therefore remain useful history but are withdrawn as independently recomputable live evidence. The custody implementation retains all fifty integer-nanosecond durations in execution order, complete stderr and status for every RSS child, refuses absent or duplicate maximum-RSS lines and unsuccessful children, keys every series by ordinal/name/targets/provider count/kind/program kind, exports raw timing/RSS artifacts, and verifies summaries, raw artifacts, annotation equality, and SHA-256 custody.

**Measurement hold, 2026-08-14 00:50 EDT.** The M3 Pro was on AC power at 100 percent with no thermal/performance warning and no swap I/O, but the formal pre-run load was `{ 3.08 2.45 2.26 }`; an immediate process control observed one Chrome renderer at 82.0 percent CPU and its Chrome parent at 24.3 percent, with load `{ 2.84 2.42 2.25 }`. The `record` command was not started, no timing or RSS samples were collected, and no host, toolchain, OS, Xcode, or SDK component changed. The clean detached custody executable and precheck snapshot remain retained on the measurement host for the next idle check.

**False source-equivalence statement repaired.** Commit `981ddf7f10dcecb109962fbd39ba56cd80d10c78` inherited a pre-existing diagnostic that counted rendered explain-record lines inside `summarize_ok` on every timed compile. Although its 45-row subject manifest, warm-up, and repeat controls equalled the prior record, its timed summarization path was not equal to `bef9a39…`. One full run at `981ddf7f…` was rejected immediately after the exact diff exposed that extra work; none of its samples or outputs is evidence and no result was retained in the repository. Commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` restores `program.rs` byte-for-byte to `bef9a39…`; the boundary observer now lives in `boundary.rs`, has one call site under `request_boundary`, and a retained test refuses any diagnostic import from `measure.rs` or call from the record measurement functions.

**Non-evidence orchestration attempt.** The first `d086fe99…` run eventually completed, but its controlling SSH session reported completion eight seconds early; the nominal post snapshot therefore overlapped the running measurement. Its outputs were rejected. The final run used a detached child, separate stdout/stderr, read-only marker polling, and an atomic numeric exit-status file written only after the real post snapshot.

## Measurement record

**Measurement, 2026-08-14.** Exact executable commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`; behavior base `4fb0427319b1504e1549e03ba023ac486343a743`; Apple M3 Pro; macOS 27.0 build `26A5388g`; Darwin 27.0.0; 11 logical CPUs; 18 GiB; `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`; release profile. Three immediately preceding checks were clean. The machine was on AC power at 100 percent battery with no thermal/performance warning or swap I/O. Load moved from `{ 2.18 2.23 2.24 }` to `{ 1.61 2.07 2.17 }`; free-memory percentage stayed 72. Apart from the observing SSH process, the highest pre/post processes were 5.0 and 4.5 percent CPU.

The exact distributions and RSS are in [`2026-08-14-request-wide-macos-27.0-m3-pro.json`](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.json); the byte-for-byte harness output is [`2026-08-14-request-wide-macos-27.0-m3-pro.generated.json`](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.generated.json). The verifier recomputes all 2,250 ordered timings and reparses complete stderr/status for all 45 RSS children. The annotation's SHA-256 values match the copied generated JSON, timing TSV, and RSS JSONL, and removing the annotation produces a semantic value equal to the generated record. The 2026-08-13 record remains linked history but is marked withdrawn and non-custodial.

| Sixteen-target row | Total raw population | Min / median / p90 / max / mean µs | Peak RSS bytes | Outcome |
| --- | ---: | --- | ---: | --- |
| governed | 304 | 63,414 / 63,483 / 63,532 / 63,557 / 63,487 | 165,855,232 | success |
| governed + one specialist | 576 | 108,880 / 109,014 / 109,138 / 110,925 / 109,061 | 310,018,048 | success |
| governed + two specialists | 848 | 177,123 / 177,303 / 177,396 / 177,545 / 177,305 | 531,693,568 | success |
| governed + 31 specialists | 8,736 named, 527 installed outcomes reached on target 1 | 49,508 / 49,610 / 49,728 / 49,803 / 49,619 | 788,283,392 | `InvalidCompilerOutput`; refusal timing only |
| add chain, four groups, one specialist | 248 installed-provider outcomes | 70,540 / 70,614 / 70,716 / 71,045 / 70,645 | 156,467,200 | success |

The unchanged generated-record helper reports `propose_per_outcome_ns=0` because it mixes request-add rows with the singleton governed floor and saturates a negative difference. That field is not measurement evidence and no decision relies on it.

## Decision-changing explain boundary

**Fact from source and direct control.** `ExplainWriter::push`, anchor `let exceeds = if terminal`, independently limits non-terminal explain detail to 4,096 records and 1 MiB of canonical bytes. `record_frontier`, anchor `for rejection in frontier.rejections()`, retains each named decline; complete-plan explanation also follows the Cartesian plan population.

**Fact.** `DeterministicBudgets` has no physical-provider raw-outcome field at this base. `16,384` is a calibration candidate rather than an installed authority, so it cannot fire on this compile path; the preserved 256-outcome draft is read-only evidence on a different commit.

Holding the one-target five-operation strict subject fixed, specialists 1 through 6 succeed with alternatives 6, 12, 20, 30, 42, and 56 and rendered record lines 346, 579, 890, 1,279, 1,746, and 2,291. Thus successful alternatives are `(n + 1)(n + 2)` and rendered record lines are `39n² + 116n + 191`, while installed outcomes grow only as `17n`. Seven specialists emit 119 installed outcomes and fail at the exact retained terminal line:

```text
2257 target-feasibility compiler-failure rule=compile.failure@1 provider=compiler:tiler.compiler@1 subject=region:program-alternative:b489b9770d000255/region:0 event=compiler-failure:explain-detail-capacity causes=2256
```

**Inference from the measured ordinal and source disjunction.** Explain record IDs are zero-based (`local` is minted from `self.records.len()`), so terminal ordinal 2,257 and 2,258 rendered record lines mean 2,257 detail records had been retained. The 4,096-record arm therefore could not have fired. `explain-detail-capacity` identifies the non-terminal disjunction; eliminating its record-count arm leaves the one-MiB canonical-byte arm as the first governing authority. The public failure class erases that inner cause to `InvalidCompilerOutput`, while the complete failure trace retains it.

Therefore raw 16,384 is eliminated under current authorities as a standalone way to support all 31 active specialists. Full-provider activity is unevaluable only behind a deliberate explain-capacity widening or complete-record compaction decision, now owned by [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](decide-how-explain-capacity-bounds-active-physical-provider-populations.md). The successful 1,024 population measurement does not accept that value; it still requires Tom to choose two active specialists as the supported population.

## Required subject perturbations

All six closing-condition perturbations ran at exact executable evidence commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`, with their assertions unchanged. The retained [compiler log](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.compiler-negatives.txt) records exit 101 for each compiler perturbation; the retained [spike log](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.spike-negatives.txt) records exit 1 for each calculation perturbation. The [custody log](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.custody-negatives.txt) records unchanged-verifier failures for a changed duration, RSS output, RSS subject, duplicate RSS line, raw timing artifact, and annotated measurement field.

- `target-count`: `the request-wide census must reach all sixteen admitted target slots` — left 15, right 16.
- `target-order`: `the compiler must preserve caller target order in the population under test` — reversed target keys against forward target keys.
- `candidate-contract-population`: `the four-contract semantic-candidate population changed` — left 224/40/216 invocations/proposals/declines, right 248/24/224.
- `governed-outcome-inclusion`: `the raw-outcome authority must include governed and installed emissions` — left 272 raw outcomes, right 576.
- `limit-recommendation-population`: `FAIL request-narrow-limit-calculation expected=1024 observed=2048`.
- `full-limit-population`: `FAIL request-full-provider-limit-calculation expected=16384 observed=8192`.

The exact [green baseline log](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-14-request-wide-macos-27.0-m3-pro.baselines.txt) records eight spike custody/boundary tests, the focused compiler census, all 42 public spike census checks, and the retained evidence verifier passing after restoration.

## Current-base correction and terminal readiness — 2026-08-16

The measurement above remains dated evidence from executable `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`; its retained 31-specialist row truthfully records the historical class and subject that executable produced. It is not the current public classification authority. At exact current base `91e6bb23dac594c88e0cda237fb3833283e8082d`, `ExplainWriter::push` uses anchor `let capacity = if terminal`, and the same seven-specialist public subject reports `BudgetExhausted { resource: ExplainDetailCanonicalBytes, limit: 1048576, reported: 1048698 }` with terminal subject `f10d1b8bfd323115`. The accepted implementation preserves the same refusal point, complete-or-refused trace, and request-wide atomicity; it changes no retained timing/RSS observation.

The current checkout's custody verifier again returned `PASS custody evidence`, reparsing all 2,250 ordered durations and all 45 RSS child records. The three hashes remain `ec3abc4ef90acb0d0e3e8a53f355f86a172ac2c2fce5a442310172b80b376c41`, `ebfb9015623fef7da7e9cfc7c6420cf3f5cd8faa245761e2e28d7f500d2b44ce`, and `8d8146bed7f0fa6e3d6a1feaed1cd2b4e5e9fea16721bd7ef50a44c26eb9cf78`. No new M3 timing was run.

Every closing condition is satisfied and no policy choice can change the historical observations. The dependency on the explain-capacity policy is removed so the graph does not imply a rerun; terminal status is left to the coordinator because this worker is not authorized to close tickets.
