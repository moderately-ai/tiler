---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.physical-frontier-budget-calibration"
kind: "research"
title: "First physical-frontier provider-count and raw-outcome limits"
topics: ["program-planning", "budgets", "measurement", "host-performance"]
catalog_group: "physical-planning-lowering"
research_status: "in-progress"
disposition: "pending"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "exhaustive-finite"]
informs: ["tiler.contract.optimizer"]
ticket: "calibrate-the-physical-frontier-provider-and-outcome-budgets"
---

# First physical-frontier provider-count and raw-outcome limits

- **Status:** request census and valid idle-M3 rows are complete; the exact raw value remains held on an active-specialist support-policy choice and a newly exposed independent explain-capacity decision. The first experiment measured one target per request. The request-wide run reaches governed, one-specialist, two-specialist, and four-contract populations, while the attempted 31-specialist row refuses on the first target. Neither budget is yet a `DeterministicBudgets` field.
- **Ticket:** [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](../../../tickets/calibrate-the-physical-frontier-provider-and-outcome-budgets.md).
- **Experiment:** [Physical-frontier provider-count and raw-outcome budget calibration](../../../spikes/program-planning/physical-frontier-budget-calibration/README.md).
- **Evidence date:** 2026-08-14. The historical single-target timing and the custodial request-wide timing are Apple M3 Pro. Compiler behavior is exact base `4fb0427319b1504e1549e03ba023ac486343a743`; the request measurement ran from exact executable evidence commit `d086fe9953a09a1a8a64dbd2353e9ded78ef18e6`. The measured `program.rs`, `profile.rs`, and `providers.rs` blobs and all 45 subject/control rows equal `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`; the separate boundary observer has no call from a record or RSS-child path.

## Request-wide census

**Measurement.** The public installed specialist answers one raw outcome per reached subject. Against the five-operation strict program:

| Target slots | Invocations / raw outcomes | Proposals | Declines |
| ---: | ---: | ---: | ---: |
| 1 | 17 | 3 | 14 |
| 2 | 34 | 6 | 28 |
| 8 | 136 | 24 | 112 |
| 16 | 272 | 48 | 224 |

**Measurement.** The crate-private structural census includes the governed provider and separates the stages that a raw outcome can enter:

| Sixteen-target population | Emitted proposals / assessments started | Declines | Raw | Verified | Admitted / retained implementations | Proposal / total frontier rejections | Sort items (admitted / rejected) | Plan combinations / retained plans |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Five-op, governed | 48 / 48 | 256 | **304** | 48 | 48 / 48 | 0 / 256 | 48 / 256 | 32 / 32 |
| Five-op, governed + one feasible specialist | 96 / 96 | 480 | **576** | 96 | 96 / 96 | 0 / 480 | 96 / 480 | 96 / 96 |
| Five-op, governed + one infeasible specialist | 96 / 96 | 480 | **576** | 96 | 48 / 48 | 48 / 528 | 48 / 528 | 32 / 32 |
| Add chain, four contract groups, governed | 24 / 24 | 224 | **248** | 24 | 24 / 24 | 0 / 224 | 24 / 224 | 24 / 24 |

The installed-specialist request retains 103,137, 206,274, 825,096, and 1,651,952 summed rendered explanation bytes at 1, 2, 8, and 16 strict targets. The four-contract add-chain rows retain 42,545, 85,170, 514,744, and 1,030,032 rendered bytes. These `Compilation::explain().render()` lengths are recorded separately from work counts and process RSS. The add-chain request cycles strict, flush-only, reassociate-only, and flush-plus-reassociate profiles. Eight targets evaluate one semantic candidate and eight evaluate two. Reversing all sixteen target slots preserves the `248` work total and returns results in the reversed caller order.

`proposal_assessments_started` is an independent loop-entry count, not a restatement of emitted proposals. A retained fatal-first provider emits two proposals but starts one assessment because malformed cost provenance aborts before the later proposal; moving the valid proposal first fails the unchanged assertion at left 2 / right 1. A second retained negative spans an inapplicable scheduled proposal, an applicable reserved `View`, and an applicable scheduled kernel; removing the reserved-body path fails the three-start assertion at left 2 / right 3. The ordinary census happens to have equal emitted/start counts because none of its proposals is fatal.

The machine-readable record is [`2026-08-13-request-population-census.json`](../../../spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-population-census.json). The compiler test `request_wide_physical_planning_population_is_pinned` is the governed-outcome authority; the public spike is the installed-provider, numerical-group, target-count, and target-order authority.

## Provider count and raw outcomes are different dimensions

**Fact from source reading.** The compiler-owned production provider population is one: `GovernedPhysicalProvider`. The retained mechanical check is only a textual source-declaration census over the exact ordinary impl spelling after named test-source exclusions; it is not a Rust type-system enumeration. Its negative passes a syntactically valid second impl fragment through those same scanner functions rather than appending a fake result. Separately, `InstalledPhysicalProviders::installed` collects the caller's complete iterator and has no count branch. The harness installs 129 distinct providers successfully; this is a finite witness for the source reading, not a claim of a portable maximum. The historical `InvalidCompilerOutput` at 128 all-decline providers is a later explain-capacity wall, not an installation limit.

**Proposal.** The earlier provider-count candidate remains 32, governed included. It bounds invocation/provenance overhead. It does not promise that all 31 installed providers may each emit an outcome for every subject: raw outcomes are an independent bound precisely so many active providers can be refused before downstream work becomes unbounded.

**Measurement.** Holding one five-operation strict target fixed, installed outcomes grow as `17n`, retained alternatives through the successful rows grow as `(n + 1)(n + 2)`, and rendered record lines grow as `39n² + 116n + 191` for `n = 1..6` specialists.

**Fact from source reading.** `record_frontier`, anchor `for rejection in frontier.rejections()`, retains each named decline, while complete-plan explanation retains the Cartesian plan population. **Inference.** One raw outcome is therefore a cardinality unit, not a uniform explain-work or memory unit.

## Raw-outcome decision frontier

**Fact.** `256` is eliminated. It refuses the governed sixteen-target ordinary request at 257 after only thirteen target successes on preserved draft `54e272ba`; the durable reproduction reports targets 0–12 compiled and 13–15 `BudgetExhausted { PhysicalFrontierOutcomes, limit: 256, reported: 257 }`. Returning that prefix is a separate draft propagation defect. It does not make 256 a viable whole-request calibration.

**Fact.** The arithmetic for the two material support readings is:

- governed only: `304`;
- governed plus one installed specialist: `304 + 272 = 576`;
- governed plus two: `304 + 2 × 272 = 848`;
- governed plus three: `304 + 3 × 272 = 1,120`;
- all 32 proposed provider slots active: `304 + 31 × 272 = 8,736`.

**Proposal — narrow active-specialist population.** `1,024` covers the retained experiment's governed-plus-two-additive population with 176 outcomes of headroom and intentionally refuses three or more active specialists. Its strongest counterargument is authority: no accepted policy currently says two is the ordinary maximum. Evidence reversing it would be a named consumer requiring three active specialists, or idle-M3 measurements showing the wider population is affordable.

**Eliminated under current authorities — complete proposed-provider population.** `16,384` arithmetically covers 8,736 outcomes with 7,648 headroom, but the raw limit alone cannot make every slot usable. The exact public run with 31 installed specialists refuses on its first target after 527 installed outcomes with `InvalidCompilerOutput`. The narrower control identifies the retained terminal cause as `explain-detail-capacity`: six specialists succeed, while seven fail at record ordinal 2,257. `ExplainWriter::push` bounds non-terminal canonical detail bytes at 1 MiB independently of its 4,096-record bound. A raw value of 16,384 is therefore eliminated as a standalone option for its named support population. Full 32-provider activity is unevaluable only as a composite option behind an explicit decision to widen or compact complete explain evidence.

The intermediate powers 2,048, 4,096, and 8,192 admit at most 6, 13, and 29 installed specialists respectively (`304 + n × 272`). Six happens to be the last successful active-specialist count for the exact one-target subject under the current explain byte ceiling, but that implementation accident is not an accepted consumer support boundary. Selecting any intermediate power would still invent policy. If a consumer names one of those populations, that value re-enters the frontier.

**Hold.** The exact raw value is not chosen here. `1,024` remains viable only if Tom explicitly makes two active specialists the supported population; the measurement does not supply that authority. Full-provider activity now depends on [`decide-how-explain-capacity-bounds-active-physical-provider-populations`](../../../tickets/decide-how-explain-capacity-bounds-active-physical-provider-populations.md). The implementation sink remains transitively held through the calibration graph.

## Authority and identity

**Inference from accepted architecture.** One request-scoped raw-outcome counter remains the only survivor. Per-target or per-candidate counters permit the same caller request to spend the limit repeatedly and change retry/partial-product semantics, reopening the accepted accounting authority rather than merely changing a value. A nested request total plus sublimits adds authority and schema without evidence of a distinct fairness requirement. These scopes are not implementation defaults.

Whichever exact value Tom accepts belongs in the deterministic request budget subject. Changing it directly changes the compiler-internal canonical request/evidence subject and the explain request qualifier for every compilation. Consistent with the accepted optimizer contract's `Budget bytes bind the compiler-internal request/evidence subject` anchor, budget bytes do not directly enter plan, artifact, or cache identity; those identities move only indirectly when the changed bound changes selected packaged content. The census adds no public surface and no schema row; the preserved sink remains a labelled draft.

## Idle-M3 request-wide measurement

**Measurement.** Apple M3 Pro, macOS 27.0 build `26A5388g`, 11 logical CPUs, 18 GiB, pinned `rustc 1.99.0-nightly`, release profile, eight warm-ups and fifty timed repetitions. Load moved from `{ 2.18 2.23 2.24 }` to `{ 1.61 2.07 2.17 }`; free memory stayed 72 percent, and the machine stayed on AC power with no thermal/performance warning or swap I/O. No competing build or measurement process appears in the process snapshots; the retained Chrome and renderer rows are idle at 0.0 percent CPU. Exact min/median/p90/max/mean and child `/usr/bin/time -l` RSS rows are in the [spike record](../../../spikes/program-planning/physical-frontier-budget-calibration/README.md#request-wide-m3-pro-timing-and-rss).

The sixteen-target governed request measured 63,414 / 63,483 / 63,532 / 63,557 / 63,487 µs and 165,855,232 peak RSS bytes. One specialist measured 108,880 / 109,014 / 109,138 / 110,925 / 109,061 µs and 310,018,048 bytes. Two specialists — the 848-raw-outcome population used by the 1,024 candidate — measured 177,123 / 177,303 / 177,396 / 177,545 / 177,305 µs and 531,693,568 bytes. The 31-specialist row measured the early refusal only: 49,508 / 49,610 / 49,728 / 49,803 / 49,619 µs and 788,283,392 bytes, not the full 8,736-outcome request.

**Measurement custody.** The record retains 2,250 ordered integer-nanosecond observations and complete `/usr/bin/time -l` stderr plus child status for all 45 RSS rows. A deterministic verifier independently recomputes every published microsecond summary, refuses absent or duplicate RSS lines and unsuccessful children, compares exported raw artifacts byte-for-byte, proves annotated measurement fields equal the generated record, and verifies SHA-256. Duration, RSS output, RSS subject, duplicate-RSS, raw-artifact, and annotated-field perturbations all fail with unchanged assertions. The 2026-08-13 request-wide record remains only withdrawn non-custodial history.

The unchanged generated-record helper reports `propose_per_outcome_ns=0` because it mixes request-add rows with the singleton governed floor and saturates a negative difference. That derived field is rejected as evidence; the five distribution statistics and RSS above are direct observations.

## Unsupported guarantee

The candidates bound compiler-owned accepted outcomes and the verification that follows an emission. They do not bound arbitrary native provider computation or allocation before an emission, do not replace `physical_plan_combinations` or complete-explain capacity, and do not claim a universal provider/subject/candidate population beyond the measured programs and profiles.
