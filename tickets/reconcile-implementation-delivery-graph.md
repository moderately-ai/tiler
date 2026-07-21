---
id: reconcile-implementation-delivery-graph
title: Reconcile implementation delivery graph
status: done
priority: p0
dependencies: [prototype-target-neutral-fusion-slice]
related: []
scopes: [project/tickets, contracts/navigation]
shared_scopes: []
paths: []
tags: [planning, implementation, milestone-0b]
---
Correct the implementation work graph after the target-neutral fusion audit.
Create dependency-ordered tickets for the missing compiler foundations,
neutral artifact path, Metal AOT proof, runtime proof, inline AOT developer
experience, and first consumer adapter; repair premature milestone
dependencies; and update navigation/status documents so Metal is gated on a
backend-consumable neutral compiler contract.

## Exit criteria

- Every required implementation stage has a bounded outcome, explicit
  dependencies, and conflict scopes.
- Existing Metal and runtime umbrella tickets no longer hide missing
  prerequisites.
- The ready frontier reflects the actual next compiler-foundation task.
- Status, roadmap, and open-question navigation agree with the ticket graph.
- Ticket lint, documentation validation, and branch guard pass.
