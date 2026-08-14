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

- **Status:** request census complete; exact raw value held on one support-policy choice and one external host measurement. The first experiment measured one target per request. The retained harness now reaches all sixteen target slots and the compiler-owned work stages, but the available host was an active Apple M4 Max rather than the required idle M3 Pro. Neither budget is yet a `DeterministicBudgets` field.
- **Ticket:** [`calibrate-the-physical-frontier-provider-and-outcome-budgets`](../../../tickets/calibrate-the-physical-frontier-provider-and-outcome-budgets.md).
- **Experiment:** [Physical-frontier provider-count and raw-outcome budget calibration](../../../spikes/program-planning/physical-frontier-budget-calibration/README.md).
- **Evidence date:** 2026-08-13. The historical single-target timing is Apple M3 Pro. The request-wide result is a finite census of compiler behavior at exact base `4fb0427319b1504e1549e03ba023ac486343a743`, executed with the retained harness and corrected independent proposal-assessment counter at `bef9a39afaeb929eef99d7d43232bdc61c9b5e2a`; this record lands in a descendant that leaves that executable evidence unchanged. It is not a new timing record.

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

## Raw-outcome decision frontier

**Fact.** `256` is eliminated. It refuses the governed sixteen-target ordinary request at 257 after only thirteen target successes on preserved draft `54e272ba`; the durable reproduction reports targets 0–12 compiled and 13–15 `BudgetExhausted { PhysicalFrontierOutcomes, limit: 256, reported: 257 }`. Returning that prefix is a separate draft propagation defect. It does not make 256 a viable whole-request calibration.

**Fact.** The arithmetic for the two material support readings is:

- governed only: `304`;
- governed plus one installed specialist: `304 + 272 = 576`;
- governed plus two: `304 + 2 × 272 = 848`;
- governed plus three: `304 + 3 × 272 = 1,120`;
- all 32 proposed provider slots active: `304 + 31 × 272 = 8,736`.

**Proposal — narrow active-specialist population.** `1,024` covers the retained experiment's governed-plus-two-additive population with 176 outcomes of headroom and intentionally refuses three or more active specialists. Its strongest counterargument is authority: no accepted policy currently says two is the ordinary maximum. Evidence reversing it would be a named consumer requiring three active specialists, or idle-M3 measurements showing the wider population is affordable.

**Proposal — complete proposed-provider population.** `16,384` covers 8,736 outcomes with 7,648 headroom, so every slot under the separate 32-provider candidate can answer every measured subject. Its strongest counterargument is host cost: it admits sixteen times the raw cardinality of 1,024 and has no valid idle-M3 request-wide runtime/RSS record. Evidence reversing that concern is the held measurement showing acceptable host time and memory.

The intermediate powers 2,048, 4,096, and 8,192 admit at most 6, 13, and 29 installed specialists respectively (`304 + n × 272`). They are not material options yet: the repository names no support boundary at 6, 13, or 29, so selecting one would invent the same missing policy as selecting two without gaining complete-cardinality coverage. If a consumer names one of those populations, that value re-enters the frontier.

**Hold.** The exact raw value is not chosen here. Tom must decide whether three or more active specialists are intentionally unsupported; the wider survivor additionally waits on [`measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro`](../../../tickets/measure-request-wide-physical-frontier-budgets-on-the-idle-m3-pro.md). The implementation sink already depends transitively on that prerequisite through this calibration ticket.

## Authority and identity

**Inference from accepted architecture.** One request-scoped raw-outcome counter remains the only survivor. Per-target or per-candidate counters permit the same caller request to spend the limit repeatedly and change retry/partial-product semantics, reopening the accepted accounting authority rather than merely changing a value. A nested request total plus sublimits adds authority and schema without evidence of a distinct fairness requirement. These scopes are not implementation defaults.

Whichever exact value Tom accepts belongs in the deterministic request budget subject. Changing it directly changes the compiler-internal canonical request/evidence subject and the explain request qualifier for every compilation. Consistent with the accepted optimizer contract's `Budget bytes bind the compiler-internal request/evidence subject` anchor, budget bytes do not directly enter plan, artifact, or cache identity; those identities move only indirectly when the changed bound changes selected packaged content. The census adds no public surface and no schema row; the preserved sink remains a labelled draft.

## Host measurement hold

The request-wide timing/RSS protocol is implemented for 1, 2, 8, and 16 targets, governed and installed populations, four contract groups, two specialists, and the full 31-installed-specialist population. It was not run as a record: the available machine identified itself as Apple M4 Max, 36 GiB, 14 cores, under load `{ 4.66 3.87 3.49 }`, with iTerm at 24.5%, WindowServer at 14.1%, and the coordinating agent active. The repository protocol requires the idle M3 Pro. No M4 number is published as decision evidence.

## Unsupported guarantee

The candidates bound compiler-owned accepted outcomes and the verification that follows an emission. They do not bound arbitrary native provider computation or allocation before an emission, do not replace `physical_plan_combinations`, and do not claim a universal provider/subject/candidate population beyond the measured programs and profiles.
