---
id: measure-b1-d-peak-residency-on-a-named-host
title: Measure B1-d peak residency on a named host
status: todo
priority: p3
dependencies: [build-the-model-level-measurement-harness]
related: [define-first-metal-lm-workload, design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, project-only-the-final-position-logits, scope-a-windowed-kv-append-into-retained-capacity]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, measurement, memory, performance, language-model, metal]
---
## User-visible outcome

The condition [`define-first-metal-lm-workload`](define-first-metal-lm-workload.md) attached to extending the benchmark matrix above 8,320 positions is met — a peak-residency measurement on a named host — and the model-level residency arithmetic three design decisions rest on is either confirmed or falsified.

## Why this exists

**Fact.** The workload profile's exclusion table reads "Contexts beyond 8,320 tokens … A residency measurement on a named host, under L8", and its benchmark-row section says the same: extending the matrix upward "needs a residency measurement on a named host first, and it belongs to L8."

**Fact.** [`design-model-ingestion-and-complete-execution`](design-model-ingestion-and-complete-execution.md) states the model-level peak figures and labels every one an **Inference** over quantities L1, L4, and L5 already state. Nothing has measured any of them.

**Inference — the measurement's primary value is not performance.** It is the only check that can falsify that arithmetic, and three separate design positions rest on it: D-16's 1.714 GiB of declined peak KV residency, the choice between the two prefill decompositions, and the final-position projection's 4,978,027,008-byte saving.

## Required work

- Measure peak resident bytes for the C1 prefill row, the C1 final decode step, the B1-d final decode step, and B1-d prefill, on a named host through the L8 harness, with the host, OS build, toolchain builds, and procedure recorded.
- Compare each against the stated arithmetic — 2,394,286,488 B at C1 prefill; ≤ 2,393,069,056 B at C1 decode 8; ≤ 6,203,791,360 B at B1-d's final decode; and the three B1-d prefill figures for the unfused decomposition, the alternative decomposition, and the alternative with final-position logits. **A measured peak above the row's stated sum means a plan allocated something the design did not account for**, which is a design defect rather than a performance result and is reported as one. A measured peak materially below it means the transient column's bound is loose in a way worth recording.
- Only after the B1-d row is confirmed, state what a longer row would cost and whether it is admissible on the measured host. The workload profile's exclusion stays in force until then, and a row above 8,320 positions is added by the change that measures it rather than by this one.

## Explicit non-goals

No new benchmark row, no D-16 decision — its trigger requires a per-layer recovery contract as well, and this measurement supplies at most the residency half. No prefill-decomposition selection: this measures what each costs and selects neither.

## Closes when

The four rows are measured on a named host, each is compared against its stated arithmetic with any disagreement reported as a design defect rather than a number, and the workload profile's exclusion row for contexts beyond 8,320 either fires with the measurement cited or is confirmed to stand.
