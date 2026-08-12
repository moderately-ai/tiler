---
id: implement-the-composed-realization-evaluation-driver
title: Implement the composed realization evaluation driver
status: todo
priority: p2
dependencies: [retain-each-plan-alternative-s-verified-semantic-candidate, define-the-composed-realization-driver-subject-bridge]
related: [accept-the-composed-realization-evaluation-surface, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation]
scopes: [implementation/compiler, implementation/conformance, implementation/ir, implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, conformance, reference, numerics, correctness]
---
## User-visible outcome

One public conformance entry computes expected bits from the exact semantic candidate and the exact ordered realization the selected plan declared, including plans that spend reassociation in both a semantic rewrite and a physical reduction split.

## Authority and prerequisites

Tom accepted the driver as the sole public composition entry through [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md), retained the `ValueId` pin/observe primitive crate-private, and on 2026-08-12 fixed `tiler-conformance` as the driver's home. Implement only after the mandatory candidate retention and exact subject bridge are complete.

## Required delivery

- Implement the accepted `tiler-conformance` entry over the complete `PlanAlternative` plus declared input bindings. It must obtain every candidate/stage/value association from the compiler-minted bridge, never from caller-provided parallel arrays or keys.
- Implement the reference evaluator's crate-private pin/observe primitive with typed refusals for invalid pins, unreachable observations, and type/shape disagreement. No public re-export or `#[doc(hidden)]` escape.
- Drive the retained `P'` through the plan's ordered stage/materialization sequence and the existing declared-order fold evaluators. Refuse every unsupported population from the composed-realization record by name; never silently run the strict baseline interpretation.
- Repair `the_assembled_split_program_matches_the_partitioned_sum_oracle` so its expected prologue comes from `P'`, not `kernels[0]`. Replace or extend the fixture so `P' != P` and both semantic and physical reassociation change the exact bits.
- Update `tiler-conformance`'s crate header: this accepted driver is its first public non-test surface; retain the rule that nothing in the workspace depends on this top evidence crate.
- Keep artifact, cache, request, schedule, KIR, and semantic identity bytes unchanged.

## Watched failures

Feed a baseline program in place of `P'`; swap two alternatives' stage subjects; reverse or omit one materialization; pin one implementation-produced tensor; remove one reference observation; and restore the existing `kernels[0]` provenance. Each independent perturbation must fail with the expected typed rule and quoted output.

## Non-goals

Artifact-only replay, a public pinning primitive, a plan type in `tiler-reference`, tolerance comparisons, device-produced oracle inputs, or a new schedule evaluator.

## Closes when

The complete accepted population evaluates through the one public driver, every named unsupported case refuses, the provenance regression is discriminating, no other public composition/pinning entry exists, and targeted compiler/reference/conformance checks plus exact-base guard are green.
