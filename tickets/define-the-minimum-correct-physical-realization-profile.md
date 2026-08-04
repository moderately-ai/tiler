---
id: define-the-minimum-correct-physical-realization-profile
title: Define the minimum correct physical realization profile
status: todo
priority: p1
dependencies: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
related: [implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, prototype-complete-physical-plan-selection]
scopes: [research/program-planning, research/scheduling, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [research, physical-planning, correctness, baseline]
---
## User-visible outcome

Any supported semantic program has a deliberately simple, valid physical route even
when no fusion, tiling, parallel reduction, or calibrated cost model applies. Advanced
physical optimization improves that baseline; it is not a prerequisite for general
correct execution.

Define the minimum profile for arbitrary acyclic MIMO programs over the explicitly
supported operation/signature set. Cover deterministic topological partitioning,
ordered multi-output preservation, conservative materialization, explicit buffers,
placement/transfers, serial/direct kernels where legal, reference/host fallback only
if the architecture explicitly permits it, and fail-closed refusal where no legal
realization exists. Separate hard feasibility from cost and separate semantic
correctness from target availability.

Audit the current Pointwise/SerialSum/Contraction strategy selector and advanced
physical-plan research against this baseline. Identify which existing tickets close
general DAG partitioning, complete covers, output identity, buffer/lifetime planning,
and multi-entry assembly; file the missing bounded work. Do not design a sophisticated
optimizer here and do not claim a fallback for an operation without a defined
reference/numerical contract.

## Closes when

There is a complete stage-by-stage correctness argument for the minimum supported
profile, every required component has a live ticket and dependency, unsupported
cases produce typed explanations, and advanced scheduling work is ordered after—not
substituted for—the generic executable baseline.
