---
id: measure-metal-thread-execution-width-across-prepared-pipelines
title: Measure Metal thread execution width across prepared pipelines
status: in-progress
priority: p1
dependencies: []
related: [declare-metal-subgroup-realization-facts-in-the-target-profile, decide-the-prepared-subgroup-width-equality-gate]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, metal, subgroup, target-profiles, evidence]
claimed_from: todo
assignee: worker-measure-metal-width
lease_expires_at: 1786659450
---
## User-visible outcome

The first standard Metal subgroup row is based on retained observations of the exact prepared-pipeline property it claims, with variation across pipeline shapes made visible rather than assumed away.

## Measurement question

On the qualified Apple9/Xcode/SDK profile, does `MTLComputePipelineState.threadExecutionWidth` remain equal across a predeclared set of pipelines that vary operation family, arithmetic type, control flow, threadgroup shape, and relevant compiler selection?

## Required protocol before any submission

- Freeze the pipeline population, exact compilation-selection identities, source/ABI/oracle subjects, device/profile, metric, repetitions where applicable, environment, custody, and stop conditions in the ticket and retained README.
- Include the exact subgroup candidate families the profile would authorize and negative/control pipelines that could expose width variation. Do not select the matrix after reading widths.
- Build and verify first. Run only in an explicitly granted quiet device window on the authorized host; do not change Xcode, SDK, OS, Rust, or device state.
- Retain every width observation and pipeline identity. Equality or variation is the result; no modal value, first value, or fallback is substituted.
- Perturb pipeline identity, result population, environment, and executable custody independently with unchanged assertions.

## Outcome boundary

This measurement may license only the observed profile/pipeline population. Even perfect equality does not remove ADR 0094's prepared-pipeline confirmation without a separate accepted decision. Variation requires per-pipeline evidence and categorically forbids a single compile-profile width row.

## Closes when

The predeclared population has a retained, reproducible result and the exact target-profile claim it supports—or fails to support—is recorded.
