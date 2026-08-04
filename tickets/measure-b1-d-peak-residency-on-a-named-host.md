---
id: measure-b1-d-peak-residency-on-a-named-host
title: Measure B1-d peak residency on a named host
status: todo
priority: p3
dependencies: [build-the-model-level-measurement-harness, establish-a-dynamic-kv-physical-layout-authority]
related: [define-first-metal-lm-workload, design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, project-only-the-final-position-logits, scope-a-windowed-kv-append-into-retained-capacity]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, measurement, memory, performance, language-model, metal]
---
## User-visible outcome

The condition [`define-first-metal-lm-workload`](define-first-metal-lm-workload.md) attached to extending the benchmark matrix above 8,320 positions is met — a peak-residency measurement on a named host — and the survivor-specific model-level residency arithmetic is either confirmed or falsified.

## Why this exists

`establish-a-dynamic-kv-physical-layout-authority` is a direct dependency
because this measurement needs its selected resource population and residency
formula as the oracle. The edge replaces reliance on the rejected dense
candidate's totals; it does not broaden the measurement outcome.

**Fact.** The workload profile's exclusion table reads "Contexts beyond 8,320 tokens … A residency measurement on a named host, under L8", and its benchmark-row section says the same: extending the matrix upward "needs a residency measurement on a named host first, and it belongs to L8."

**Fact.** [`design-model-ingestion-and-complete-execution`](design-model-ingestion-and-complete-execution.md) states the model-level peak figures and labels every one an **Inference** over quantities L1, L4, and L5 already state. Nothing has measured any of them.

**Inference — the measurement's primary value is not performance.** The cited totals contain KV terms from a rejected compact-allocation candidate. [Dynamic KV physical-layout authority](../docs/research/runtime/dynamic-kv-physical-layout.md) instead selects 56 logical members with two capacity-sized pool banks each: `2 × capacity × 229,376` reserved bytes, or 3,816,816,640 at B1-d capacity 8,320, while final exact-live bytes touched are 3,816,587,264. This measurement can falsify the reservation arithmetic and distinguish it from resident pages; D-16 uses the measured token-transaction cost while the prefill-decomposition and final-position-projection terms remain separately attributable.

## Required work

- Measure peak resident bytes for the C1 prefill row, the C1 final decode step, the B1-d final decode step, and B1-d prefill, on a named host through the L8 harness, with the host, OS build, toolchain builds, and procedure recorded.
- After `establish-a-dynamic-kv-physical-layout-authority` supplies the selected representation, recompute each row from its exact storage population and compare the measurement against that survivor-specific arithmetic. Retain the prior totals only as historical rejected-candidate controls. **A measured peak above the survivor's stated sum means a plan allocated something the design did not account for**, which is a design defect rather than a performance result and is reported as one. A measured peak materially below it means the bound is loose in a way worth recording.
- Only after the B1-d row is confirmed, state what a longer row would cost and whether it is admissible on the measured host. The workload profile's exclusion stays in force until then, and a row above 8,320 positions is added by the change that measures it rather than by this one.

## Explicit non-goals

No new benchmark row, no D-16 decision — its trigger requires a per-layer recovery contract as well, and this measurement supplies at most the residency half. No prefill-decomposition selection: this measures what each costs and selects neither.

## Closes when

The four rows are measured on a named host, each is compared against its stated arithmetic with any disagreement reported as a design defect rather than a number, and the workload profile's exclusion row for contexts beyond 8,320 either fires with the measurement cited or is confirmed to stand.
