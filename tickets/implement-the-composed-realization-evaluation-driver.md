---
id: implement-the-composed-realization-evaluation-driver
title: Implement the composed realization evaluation driver
status: todo
priority: p2
dependencies: [retain-each-plan-alternative-s-verified-semantic-candidate, define-the-composed-realization-driver-subject-bridge]
related: [accept-the-composed-realization-evaluation-surface, compose-a-declared-reduction-topology-into-a-semantic-program-evaluation, decide-the-safe-cross-crate-composed-reference-boundary]
scopes: [implementation/compiler, implementation/conformance, implementation/ir, implementation/reference, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, public-boundary, conformance, reference, numerics, correctness]
---
## User-visible outcome

One public conformance entry computes expected bits from the exact semantic candidate and the exact ordered realization the selected plan declared, including plans that spend reassociation in both a semantic rewrite and a physical reduction split.

## Authority and prerequisites

Tom accepted the driver as the sole public composition entry through [`accept-the-composed-realization-evaluation-surface`](accept-the-composed-realization-evaluation-surface.md), retained the `ValueId` pin/observe primitive crate-private, and on 2026-08-12 fixed `tiler-conformance` as the driver's home. Implement only after the mandatory candidate retention and exact subject bridge are complete.

**Accepted correction, 2026-08-12.** The sentence above records the original intent but not the implementable boundary Tom subsequently accepted on [`decide-the-safe-cross-crate-composed-reference-boundary`](decide-the-safe-cross-crate-composed-reference-boundary.md): raw tensor-taking pin/observe remains private; a separate safe cross-crate reference session owns every internal tensor, receives explicit reference registry/work authority, and discharges only a completely witnessed freedom; and the first plan-binding conformance wrapper remains `pub(crate)` and test-only. The compiler-subject decision replaces the remaining provisional bridge spelling before implementation.

## Required delivery

- Implement the accepted test-only `tiler-conformance` entry over the complete `PlanAlternative`, explicit frozen reference registry/work authority, and declared input bindings. It must obtain every candidate/stage/value association from the compiler-minted bridge, never from caller-provided parallel arrays or keys.
- Implement the reference evaluator's crate-private raw pin/observe primitive plus the safe language-public cross-crate composed-evaluation session. The session accepts no caller-provided internal tensor and owns the observation/fold/pin chain. Typed refusals cover invalid or unreachable values, type/shape disagreement, incomplete witness discharge, unsupported freedoms/topologies, and registry/subject mismatch.
- Drive the retained `P'` through the plan's ordered stage/materialization sequence and the existing declared-order fold evaluators. Refuse every unsupported population from the composed-realization record by name; never silently run the strict baseline interpretation.
- Repair `the_assembled_split_program_matches_the_partitioned_sum_oracle` so its expected prologue comes from `P'`, not `kernels[0]`. Replace or extend the fixture so `P' != P` and both semantic and physical reassociation change the exact bits.
- Keep `tiler-conformance`'s no-public-surface and test-only contract. [`activate-a-public-composed-realization-oracle-for-a-named-consumer`](activate-a-public-composed-realization-oracle-for-a-named-consumer.md) owns any later reusable public entry.
- Keep artifact, cache, request, schedule, KIR, and semantic identity bytes unchanged.

## Watched failures

Feed a baseline program in place of `P'`; swap two alternatives' stage subjects; reverse or omit one materialization; pin one implementation-produced tensor; remove one reference observation; and restore the existing `kernels[0]` provenance. Each independent perturbation must fail with the expected typed rule and quoted output.

## Non-goals

Artifact-only replay, a public pinning primitive, a plan type in `tiler-reference`, tolerance comparisons, device-produced oracle inputs, or a new schedule evaluator.

## Closes when

The complete accepted population evaluates through the one public driver, every named unsupported case refuses, the provenance regression is discriminating, no other public composition/pinning entry exists, and targeted compiler/reference/conformance checks plus exact-base guard are green.
