---
id: classify-machine-compilation-and-execution-outcomes-by-stage
title: Classify machine compilation and execution outcomes by stage
status: todo
priority: p1
dependencies: [define-the-conformance-obligation-and-evidence-requirement-algebra, decide-how-owner-private-conformance-inventories-cross-crate-boundaries]
related: [spike-a-red-yellow-first-full-conformance-suite]
scopes: []
shared_scopes: [project/tickets, research/verification]
paths: []
tags: [research, design, conformance-progress, verification]
---
# Classify machine compilation and execution outcomes by stage

## Goal

A typed design for machine-dependent evidence that keeps compilation, eligibility, preparation, execution, terminal completion, comparison, and unavailability distinct, including the fifteen current tests whose absent Apple toolchain returns early.

## Work

1. Re-audit the ten Metal golden and five Metal-AOT driver early-return tests and read their complete toolchain-resolution, compile/link, diagnostic, and consumption paths.
2. Read conformance's `Measured::{Ran, Unavailable, Failed}`, host policy, measurement boundary, device preflight, dispatch, runtime route, and terminal completion owners.
3. Define raw stage observations such as `Succeeded`, `Unavailable { reason, stage }`, and `Failed { stage, code, detail }` without conflating absence with a defect or a submitted command with completed execution.
4. State which authority supplies toolchain, eligibility, preparation, execution, completion, and comparison identity/context.
5. Model report-lane versus require-lane policy outside the observation itself.
6. Compare local test-support wrappers, a shared owner vocabulary, receipt events, and deferral. Account for non-Apple compilation and avoid moving machine tests into the conformance crate merely for reuse.
7. Specify subject perturbations for missing toolchain, invalid source, ineligible environment, preparation failure, submission failure, incomplete command buffer, and wrong output.
8. Produce a migration census and one ticket per owner boundary; do not migrate tests here.

## Non-goals

- Do not treat unavailable as pass, XFAIL, ignore, or platform omission.
- Do not infer execution from compilation, preparation, or submission.
- Do not add cross-crate API before its owner/boundary decision.

## Stop conditions

Stop when a stage lacks a singular owner or when sharing the vocabulary would create a public boundary not decided by its dependency.

## Acceptance

- All fifteen conditional tests have exact proposed stage/outcome mappings.
- Every machine stage and unavailable reason is independently representable and perturbable.
- Caller policy cannot rewrite a raw observation.
- Migration work is split by owner and no green evidence is manufactured.

## Refs

- [`give-the-private-conformance-gate-a-typed-host-unavailability-outcome`](give-the-private-conformance-gate-a-typed-host-unavailability-outcome.md)
- [`spike-a-red-yellow-first-full-conformance-suite`](spike-a-red-yellow-first-full-conformance-suite.md)
