---
id: calibrate-the-physical-frontier-provider-and-outcome-budgets
title: Calibrate the physical-frontier provider and raw-outcome budgets
status: todo
priority: p1
dependencies: [decide-whether-the-implementation-frontier-owes-a-retention-budget, measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro, decide-how-explain-capacity-bounds-active-physical-provider-populations, decide-the-truthful-public-class-for-complete-explain-capacity-refusals, implement-the-truthful-explain-capacity-budget-refusal, prove-the-truthful-explain-capacity-budget-refusal-boundary]
related: [replace-provider-offer-with-a-host-bounded-frontier-sink]
scopes: [research/program-planning, implementation/compiler]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [optimizer, budgets, measurement, host-performance]
---
## User-visible outcome

The first physical-provider-count and raw proposal/decline limits are measured calibration inputs rather than inherited folklore. The record names which ordinary and adversarial provider populations fit, what Tiler host work and memory each population causes, and where atomic exhaustion should fire.

## Required experiment

- Census distinct region subjects, total provider invocations, proposals, declines, admitted implementations, proposal rejections, retained bytes, verification work, sorting work, and complete-plan combinations separately.
- Include the governed provider, the retained external-provider vertical, at least two additive providers answering one region, equal-cost incomparable proposals, many declines, infeasible proposals, and empty providers. Count the full compiler-owned population; do not time kernel execution.
- Measure Tiler host runtime on the idle M3 Pro under the repository performance protocol. Record workload, target profile, warm-up, repetitions, variance, exact toolchain, source commit, and process/load controls.
- Recommend a provider-count limit and a raw-output limit with explicit headroom. Do not reuse the historical value eight without evidence.
- State what the proposed limits do not bound: arbitrary native provider computation and allocation before an emission.

## Stop conditions

Stop and split the work if proposal construction itself needs a bounded public builder, if provider count belongs to a different preflight authority than the complete budget policy, or if one raw-outcome count cannot represent proposal and decline cost without misleading calibration.

## Fact audit at exact reopened base `4fb0427319b1504e1549e03ba023ac486343a743`

- **Verified.** The historical value eight exists only as retired folklore. Search `8 nondominated implementations per region` in `docs/compiler/optimizer.md`; `ImplementationFrontier::non_dominated` remains an unbounded borrowed Pareto view.
- **Verified by source reading.** Production `PhysicalImplementationProvider` impls are exactly one: `GovernedPhysicalProvider`. The retained `source-declared-production-provider-impls` check is narrower than that Fact: it textually scans the exact ordinary impl spelling after excluding `tests.rs` and inline `#[cfg(test)] mod` tails, not the Rust type system. Its negative sends a syntactically valid second impl fragment through the same scanner instead of appending a fake result.
- **Verified.** `one_compile_enumerates_each_distinct_region_subject_once` and the independent installed observer both report 17 distinct subjects for `hot_path.rs`'s five-operation program. The specialist emits 3 proposals and 14 declines.
- **Verified.** Search `physical_plan_combinations: 4_096` in `crates/tiler-compiler/src/request.rs`; `DeterministicBudgets` still has no provider-count or raw-outcome field.
- **False, repaired before measurement.** The former outcome and spike comment said 256 was above 272. `256 < 272`; the old sweep was singleton-target and cannot calibrate a request-scoped counter.
- **Verified.** Search `pub const MAX_TARGET_PROFILES_PER_REQUEST` in `crates/tiler-compiler/src/target.rs`; the public request admits 16 unique profiles. Search `MAX_NUMERICAL_CONTRACT_PREFERENCES` in `crates/tiler-compiler/src/session.rs`; it admits four stated preferences.
- **Verified on preserved draft `54e272ba`.** `PhysicalAuthorities::outcomes` is constructed once in `compile_with_physical_providers` and threaded through target groups, target slots, candidates, and frontier enumeration. The counter is request-scoped.
- **Imprecise, corrected by the compiler census.** `17 × 16 = 272` was only a lower bound on governed outcomes because a governed reduction subject can emit several raw outcomes. The exact governed five-op population is 304: 48 proposals plus 256 declines.
- **False as a type bound.** `InstalledPhysicalProviders::installed` collects the complete iterator and has no provider-count refusal. The durable finite witness installs 129 identities. The proposed 32-provider budget and the raw-outcome budget therefore govern different dimensions.

## Superseded single-target outcome

**Correction, 2026-08-13 at exact base `4fb0427319b1504e1549e03ba023ac486343a743`.** The outcome below is retained as the first experiment's result, not as a live recommendation. It measured one target per request and therefore did not calibrate the accepted request-scoped raw-outcome authority over the public sixteen-target population. The earlier spike comment that called `256` "above" 272 outcomes was false: `256 < 272`. The reopened work below owns the replacement value and full-request evidence.

**Measurement, 2026-08-13, Apple M3 Pro, macOS 27.0 `26A5388g`, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, warm-up 8, repeats 50, min estimator, load `{ 6.82 3.66 2.70 }` → `{ 6.32 3.66 2.71 }`.** Record: [the spike](../spikes/program-planning/physical-frontier-budget-calibration/README.md) and [the research note](../docs/research/program-planning/physical-frontier-budget-calibration.md).

| Limit | First value | Headroom |
| --- | --- | --- |
| Provider count, governed included | 32 | 32 |
| Raw proposal+decline outcomes, request-scoped | ~~256~~ | withdrawn pending full-request calibration |

Eight is not a calibration of either axis. A request-scoped eight would refuse the governed five-op program (17 subjects). Empty extras through 64 stay inside 1.11× the 3205 µs governed floor. 128 all-decline extras refuse as untyped `InvalidCompilerOutput`. Admitted proposals cost about 34× named declines on incremental host time; the raw-outcome count is a cardinality bound sized from the expensive side.

Proposal construction did not need a new public builder (`ImplementationProposal::scheduled_kernel` plus `baseline()` is enough). Provider count and raw outcomes stay in the same complete budget policy. One count remains valid as a cardinality ceiling; it is not a uniform work unit.

The limits do not bound arbitrary native provider computation or allocation before an emission.

Each load-bearing population check was perturbed with the assertion unchanged. Quoted failures are in the spike README. `census` is green after restore.

## Reopened — request-population omission found 2026-08-13

The `256` raw-outcome value is not yet a valid request-scoped calibration input. The experiment measured one target profile per compile, while `MAX_TARGET_PROFILES_PER_REQUEST` admits 16 and the proposed implementation shares one raw-outcome counter across the whole request.

- **Verified at `f3e1efd3b3b4f896976b326e6a3d993147206cd3`.** `MAX_TARGET_PROFILES_PER_REQUEST` is 16. The retained experiment's `Measurement boundary` names one program and separate governed/declared-capacity profiles; it does not census a multi-target request.
- **Verified on draft `54e272baa525027a6f6f9d982bd3bd7c387597fb`.** `PhysicalAuthorities::outcomes` is one request-scoped counter reused while the pipeline compiles target profiles and semantic candidates.
- **Inference requiring the rerun.** The governed five-operation census has 17 nonempty region subjects per target and `GovernedPhysicalProvider` answers every placed region with at least one proposal or decline. Sixteen equivalent-capability target slots would therefore reach at least 272 governed outcomes before any external provider. The harness must construct the admitted population and measure it; this arithmetic is a stop signal, not a substitute for the run.

**Measurement on preserved draft `54e272ba`, 2026-08-13.** The retained public-only fixture [`draft_request_budget.rs`](../spikes/program-planning/physical-frontier-budget-calibration/fixtures/draft_request_budget.rs) runs in a disposable detached worktree and leaves the preserved branch untouched. The ordinary five-operation program, governed providers only, and 16 distinct strict profiles return targets 0 through 12 compiled; targets 13 through 15 report `BudgetExhausted { PhysicalFrontierOutcomes, limit: 256, reported: 257 }`. This confirms both the omitted population and the draft's non-atomic propagation. The compiled-prefix defect is separate from calibration.

Before this ticket can return to `done`:

- census raw outcomes across 1, 2, 8, and 16 admitted target profiles, across alternative numerical-contract candidates that the request may evaluate, and across the widest ordinary request population;
- separate a deliberate request-wide refusal from a limit that accidentally makes an admitted ordinary request impossible;
- compare request-wide, per-target, and any nested per-candidate authority that survives the decision gate. Eliminate any scope that permits retry or a partial product after exhaustion;
- rerun identical host-time/RSS sweeps for the nondominated candidate limit(s), update the spike and research record, and perturb the multi-target subject with the assertion unchanged; and
- state the full unsupported population and identity consequence before an exact value is encoded in `DeterministicBudgets`.

Until then, `256` remains a superseded single-target measurement and [`replace-provider-offer-with-a-host-bounded-frontier-sink`](replace-provider-offer-with-a-host-bounded-frontier-sink.md) is held. The release trigger is a reviewed, reproducible full-request calibration with one surviving authority/value pair.

## Request-wide census of behavior at `4fb0427319b1504e1549e03ba023ac486343a743`

The census evidence revision is `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`; the durable result is [`2026-08-13-request-population-census.json`](../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-population-census.json) in its descendant. Custodial measurement executable `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6` restores the measured program/profile/provider blobs exactly and changes only custody on the record path; a separate boundary observer has no record or RSS-child caller. The public spike reports installed-provider emissions across 1, 2, 8, and 16 targets; the crate-private test `request_wide_physical_planning_population_is_pinned` counts governed emissions and compiler-owned downstream stages without adding a public seam. Exact detached-worktree rerun commands are in the spike README.

| Sixteen-target subject | Emitted proposals / assessments started | Declines | Raw | Verified | Admitted / retained | Proposal / total rejections | Sort items | Plan combinations / retained plans |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| five-op, governed | 48 / 48 | 256 | **304** | 48 | 48 / 48 | 0 / 256 | 48 admitted / 256 rejected | 32 / 32 |
| five-op, governed + feasible specialist | 96 / 96 | 480 | **576** | 96 | 96 / 96 | 0 / 480 | 96 admitted / 480 rejected | 96 / 96 |
| five-op, governed + infeasible specialist | 96 / 96 | 480 | **576** | 96 | 48 / 48 | 48 / 528 | 48 admitted / 528 rejected | 32 / 32 |
| add chain, four contract groups, governed | 24 / 24 | 224 | **248** | 24 | 24 / 24 | 0 / 224 | 24 admitted / 224 rejected | 24 / 24 |

The installed specialist alone emits 17, 34, 136, and 272 outcomes at 1, 2, 8, and 16 strict targets, with 103,137, 206,274, 825,096, and 1,651,952 summed rendered explanation bytes. Four-contract add-chain rows emit 10, 20, 124, and 248 outcomes with 42,545, 85,170, 514,744, and 1,030,032 rendered bytes. These `Compilation::explain().render()` lengths are separate from work counts and process RSS. Eight of the sixteen grouped targets evaluate one semantic candidate and eight evaluate two. Reversing all sixteen targets preserves work totals and output order.

The assessment column is independently observed at proposal-loop entry. A provider emitting a fatal malformed-cost proposal followed by a valid proposal reports two emissions and one assessment start; reversing that subject fails left 2 / right 1. The applicability/body negative covers an inapplicable scheduled proposal, an applicable reserved `View`, and an applicable scheduled kernel; removing the reserved-body subject fails left 2 / right 3. Equal emitted/start counts in the table mean none of those measured proposals aborted enumeration, not that one number was copied from the other.

## Decision gate

**Eliminated — request-scoped 256.** It fails the governed ordinary request at 257, below the exact 304 population. Calling that intentional would contradict this ticket's requirement to include the governed provider across the full admitted target set.

**Eliminated — request-scoped 512.** It admits governed-only 304 but refuses the named retained external-provider vertical at 576. It therefore omits a required experiment/support population.

**Nondominated — request-scoped 1,024.** It admits governed plus two active installed specialists: `304 + 2 × 272 = 848`, headroom 176. It bounds host work more strictly and intentionally refuses three specialists at 1,120. Strongest counterargument: no accepted policy says two is the maximum ordinary active population. Reversing evidence is a named three-specialist consumer or an accepted policy separating many installed identities from at most two active answerers.

**Eliminated under current authorities — request-scoped 16,384.** Under the separate 32-provider candidate, 31 installed specialists would arithmetically produce `304 + 31 × 272 = 8,736`, headroom 7,648. The exact idle-M3 public run does not reach that population: it refuses on the first target after 527 installed-provider outcomes. A fixed one-target control succeeds through six specialists and first fails at seven with `compiler-failure:explain-detail-capacity` at record ordinal 2,257. `ExplainWriter::push` has an independent one-MiB canonical detail-byte ceiling, so raw 16,384 alone cannot support its named population. Full-provider activity is unevaluable only as a composite option after [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](decide-how-explain-capacity-bounds-active-physical-provider-populations.md) deliberately retains, widens, or compacts that complete-explain authority.

**Not material yet — 2,048, 4,096, 8,192.** They cover at most 6, 13, and 29 installed specialists. Six is also the last successful active-specialist count for this exact one-target subject under the current explain byte ceiling, but an incidental implementation boundary is not an accepted support population. No contract, ticket outcome, or consumer names 6, 13, or 29; choosing one would still invent policy. A named consumer population makes the corresponding point material and reopens this enumeration.

**Eliminated — per-target or per-candidate authority.** Either lets one request spend the ceiling repeatedly and changes retry and partial-product behaviour, reopening the accepted accounting authority rather than merely changing a value. A nested request total plus sublimits adds a fairness authority no evidence requires. Request scope is the surviving authority; exact value is held.

The raw-budget frontier now has one conditional survivor: `1,024`, only if Tom explicitly chooses two active specialists as the supported population. The measurement supplies no such policy, so it is not accepted by this ticket. A future full-provider candidate is a composite decision, not `16,384` alone. Whichever value is accepted directly changes the compiler-internal canonical request/evidence subject and explain request qualifier. Budget bytes do not directly enter plan, artifact, or cache identity; those move only indirectly if the changed bound changes selected packaged content. No public surface or schema is added by this census; the sink stays a labelled draft.

**Graph safety.** The truthful-classification decision is not sufficient evidence by itself. Before that decision closes, it must create the exact accepted implementation and regression/evidence tickets and add this calibration as a hard dependent of both. This ticket must remain blocked until the false public `InvalidCompilerOutput` path is removed or made unreachable and the required public/evidence regression is complete.

## Idle-M3 request-wide measurement

**Measurement, 2026-08-14.** Exact executable commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`, behavior base `4fb0427319b1504e1549e03ba023ac486343a743`; Apple M3 Pro, macOS 27.0 `26A5388g`, 11 logical CPUs, 18 GiB, pinned nightly, release warm-up 8 / repeats 50. Load moved `{ 2.18 2.23 2.24 }` → `{ 1.61 2.07 2.17 }`; free memory stayed 72 percent; AC power, no thermal/performance warning or swap I/O. Full rows and raw files are in the [spike](../spikes/program-planning/physical-frontier-budget-calibration/README.md#request-wide-m3-pro-timing-and-rss).

The sixteen-target governed request measured min/median/p90/max/mean 63,414/63,483/63,532/63,557/63,487 µs and 165,855,232 peak RSS bytes. One specialist measured 108,880/109,014/109,138/110,925/109,061 µs and 310,018,048 bytes. Two specialists — 848 total raw outcomes, the population 1,024 would cover — measured 177,123/177,303/177,396/177,545/177,305 µs and 531,693,568 bytes. The 31-specialist row is early-refusal timing, 49,508/49,610/49,728/49,803/49,619 µs and 788,283,392 bytes; it is not 8,736-outcome evidence.

The live record retains all 2,250 ordered durations and complete stderr/status for all 45 RSS children. Its verifier recomputes every summary, reparses RSS, checks raw artifact bytes, annotation equality, and SHA-256. The 2026-08-13 request-wide record is retained only as withdrawn non-custodial history. A precheck hold, a `981ddf7f…` source-equivalence rejection, and an overlapping post-snapshot attempt supply no measurements; the accepted record used a detached child and an atomic completion marker written after the post snapshot.

The unchanged generated-record helper's `propose_per_outcome_ns=0` is invalid derived evidence because it mixes request-add and singleton rows. No value decision relies on it.

## Negative controls

Assertions stayed unchanged while each subject was perturbed:

- target count: `the request-wide census must reach all sixteen admitted target slots`, left 15 / right 16;
- target order: `the compiler must preserve caller target order in the population under test`, reversed keys / forward keys;
- candidate/contract population: `the four-contract semantic-candidate population changed`, left 224 invocations, 40 proposals, 216 declines / right 248, 24, 224;
- governed inclusion: `the raw-outcome authority must include governed and installed emissions`, left 272 raw outcomes / right 576;
- narrow candidate calculation: `FAIL request-narrow-limit-calculation expected=1024 observed=2048` after changing the population from two to three specialists.
- full candidate calculation: `FAIL request-full-provider-limit-calculation expected=16384 observed=8192` after changing the population from 31 to 29 specialists.
- fatal proposal order: `the fatal first proposal must prevent the later proposal entering assessment`, left 2 / right 1 after moving the valid proposal before the fatal one;
- proposal body/applicability population: `each emitted proposal enters assessment before applicability and body dispatch`, left 2 / right 3 after removing the reserved-body proposal.
- retained duration: `FAIL custody governed-only.min_us expected=0 observed=3197` after setting the first ordered duration to zero;
- RSS output: `FAIL custody governed-only.rss.parsed_peak_rss_bytes expected=1 observed=25886720` after changing the retained maximum;
- RSS subject: `FAIL custody governed-only RSS command subject mismatch` after changing `child-measure` to `child-request-measure`;
- duplicate RSS output: `FAIL custody governed-only retained time stderr has 2 maximum RSS lines`;
- raw/annotated custody: `raw timing artifact does not match generated record` and `annotated measurement fields differ from generated record`.

The check reaches the request-wide subject: the target-count perturbation returns 15 target slots and fails before any arithmetic assertion can mask the omission.
