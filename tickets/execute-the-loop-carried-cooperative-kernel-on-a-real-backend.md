---
id: execute-the-loop-carried-cooperative-kernel-on-a-real-backend
title: Execute the loop-carried cooperative kernel on a real backend
status: todo
priority: p1
dependencies: [lower-a-loop-carried-cooperative-body]
related: [implement-the-single-workgroup-synchronized-reduction-strategy, share-one-structured-kernel-interpreter, promote-the-bounded-scalar-cpu-vertical-into-a-production-backend]
scopes: [implementation/ir, implementation/reference, implementation/metal, implementation/conformance, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [kernel-ir, synchronization, conformance, metal, cpu]
---
## User-visible outcome

The first multi-round cooperative kernel is compiled and executed by an actual eligible backend and agrees bit-for-bit with an independent reference at its declared grouping. The former host-simulator result is retained only as historical bounded evidence, not the implementation guarantee.

## Facts and boundary

- `lower-a-loop-carried-cooperative-body`, anchor `Measurement boundary`, states that its `[2, 6] -> [2]`, three-participant, two-round result was executed only by a host interpreter; nothing was compiled or dispatched on a device.
- Its structural tests already prove the canonical peeled round, barrier placement, synchronization census, anti-dependency, and verifier refusals. Those remain layer-local and are not duplicated here.
- The production scalar CPU profile intentionally refuses barriers and concurrency. The first execution may therefore use Metal. A later threaded CPU realization must run the same property matrix before claiming support; it is not inferred from the Metal result.

## Work

- Build the exact verified multi-round schedule/KIR through the real backend emitter and artifact/runtime path available at implementation time.
- Supply launch geometry from the scheduled program, not staging allocation or fixture constants.
- Derive the expected value through an independent reference-owned representation of the declared participant/round/contributor grouping. Do not add another local `cooperative_reference` helper.
- Use both a contributor-set-sensitive input and a grouping-sensitive input; pin the populations each can and cannot distinguish.
- Execute an accepted single-round neighbour and the multi-round subject. Perturb round contribution arithmetic, barrier placement, launch width, and grouping independently and retain each named refusal or wrong-result comparison.
- Report unavailable on an ineligible host. Never count source emission or successful compilation as execution.

## Acceptance

The real backend returns the exact independently derived bits, every property perturbation fails for its own reason, the structural tests remain intact, and the measurement claim names the host/profile/backend and does not generalize to a threaded CPU realization that has not run.
