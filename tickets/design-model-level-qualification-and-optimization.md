---
id: design-model-level-qualification-and-optimization
title: Design model-level correctness and performance qualification
status: todo
priority: p2
dependencies: [define-first-metal-lm-workload, design-model-ingestion-and-complete-execution]
related: [implement-analytical-component-cost-model, calibrate-device-cost-models, scope-first-quantized-lm-profile]
scopes: [research/cost-model, research/apple-targets, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [design, testing, performance, conformance, language-model, metal]
---
Define how Tiler will establish that a supported language model is both correct
and optimized on its declared Metal target. Correctness, feasibility, estimated
cost, and measured performance remain separate claims.

## Required design

- Select model-level reference outputs and adversarial inputs for prompt,
  prefill, decode, exceptional values, sequence bounds, and persistent state.
- Define exact or tolerance-based comparison from the effective numerical
  contract rather than choosing thresholds after observing results.
- Define the Apple device-family and toolchain matrix for each claim.
- Specify cold and warm time to first token, decode latency, tokens per second,
  peak and persistent memory, artifact preparation, dispatch count,
  materialization count, and cache behavior.
- Distinguish correctness gates, performance measurements, cost-model
  calibration data, and regression thresholds.
- Define how failures remain attributable to frontend, compiler, backend,
  artifact, runtime, or consumer boundaries.

## Ticket-producing outcome

File separate tickets for the conformance corpus, measurement harness, device
qualification, cost-model calibration, kernel or schedule improvements exposed
by evidence, and regression policy. Do not turn an unmeasured performance goal
into a normative guarantee or file optimization work without a measured
bottleneck.

## Closes when

The complete-model vertical has a reproducible correctness and performance
qualification plan; every metric names an environment and procedure; baseline
and quantized paths can be compared without conflating claims; and justified
follow-up work is represented by scoped tickets.
