---
id: define-the-model-level-regression-policy
title: Define the model-level regression policy
status: todo
priority: p3
dependencies: [define-the-model-level-conformance-corpus, build-the-model-level-measurement-harness]
related: [design-model-level-qualification-and-optimization, qualify-the-model-level-claims-per-apple-device-and-toolchain-row, reclassify-language-model-work-as-a-conformance-track]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, testing, performance, regression, policy, language-model, class-performance-study]
---
## User-visible outcome

A correctness regression fails; a performance regression is reported with the evidence needed to attribute it; and neither borrows the other's threshold. An environment change announces itself and declines to compare rather than silently rebaselining.

## Evidence prerequisite

The L8 qualification record's *Regression policy* section, and the two harnesses whose conventions it generalizes: the conformance fixture's counted, demonstrably-failing checks, and the Apple numerical probe's rule that a differing environment row is announced and not compared.

L8 already publishes a six-bullet **Proposal** under `## Regression policy` in [model-level-qualification.md](../docs/research/program-planning/model-level-qualification.md). This ticket finalizes that Proposal rather than restating a pre-correction pin list: (a) pick N with rationale; (b) land the policy as closed authority refining L8; (c) record failing perturbations for every check the policy introduces, including a deliberately altered environment row that must refuse; (d) keep no latency number as a threshold.

## Required work

- **Fix what is pinned**, so that a change to it is a failure rather than a discussion: the retained C1 record's checked-in files; the 18-token sequence; the per-position greedy token and runner-up gap; the top-32 entries; the envelope (L8 pin-list spelling; the measured joint comparison band is the quantity that envelope names in the C1 fixture); 30 executions per pass and 270 for the row; the peak-residency arithmetic; and the exact refusal each corpus row must produce. **Conditional on L6 D-19** (carrier [`define-the-widening-relation-over-a-symbolic-broadcast-extent`](define-the-widening-relation-over-a-symbolic-broadcast-extent.md)): the three artifact identities and the cold pipeline-creation count of three are pinned only after D-19 closes; until then a run producing four is *reported with attribution* (specialization on `S`, or the `T = 1` graph divergence), not failed. Do not reassert an unconditional three-identity gate ahead of that decision.
- **Fix what is reported rather than gated**: every latency, achieved bandwidth, measured peak, artifact size, and expansion time, each with its `EXPLAIN` diff retained so the change is attributable to a plan, codegen, toolchain, or hardware-profile change. Require the report to name which of the four it attributed to, or to say it could not.
- **Set no latency threshold before a baseline exists.** State the rule that makes one possible later: a performance row becomes a gate only after N recorded baselines on one host establish its spread, and the gate is then a multiple of that measured spread rather than a round percentage. Pick N and say why.
- **The environment-change rule.** When host, OS build, toolchain build, checkpoint revision, or reference revision differs from the retained record's, the comparison is refused and the difference announced. This is the one policy that stops a toolchain bump from rebaselining a conformance record, and it is the reason a retained record is evidence rather than a snapshot.
- **Intermittency is a defect.** A conformance check that sometimes passes is root-caused and fixed; it is never re-run until green, never loosened, and never labelled flaky.
- **Every check demonstrated able to fail**, with the perturbation recorded beside it — including the checks the policy itself adds, such as the environment-change refusal, which must be watched refusing a deliberately altered environment row.

## Explicit non-goals

No threshold values, because no baseline exists on any host for any row of this workload. No CI integration — this repository runs none. No policy for the quantized path's acceptability criterion, which is a profile qualification and never a build gate.

## Graph repair — 2026-08-10

[`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md) demoted from `dependencies` to `related` to match the design filing table (order-6 waits on corpus and harness only). Reclassify remains inventory context for the class tag; it is not load-bearing for writing the policy.

## Closes when

The pinned set, the reported set, the baseline-before-threshold rule with its chosen N, the environment-change refusal, and the intermittency rule are written down; every check the policy introduces has a recorded failing perturbation; and no latency number appears as a threshold.
