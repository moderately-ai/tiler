---
id: represent-cooperative-workgroup-reduction-dataflow
title: Represent cooperative workgroup reduction dataflow
status: todo
priority: p1
dependencies: []
related: [admit-the-first-typed-synchronization-point-and-atomic-target-authority]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/foundation, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A target-neutral schedule and KIR can describe the meaningful cross-invocation dataflow a bounded workgroup reduction needs before any synchronization point is admitted: local invocation coordinates, workgroup-shared staging storage, phased writes and reads, explicit lifetimes, and uniform participant convergence.

## Correctness boundary

The current schedule has only a global-linear one-output mapping. KIR exposes boundary reads and one write, has no usable workgroup allocation or local-invocation coordinate, and rejects synchronization. Adding a barrier to that program is either semantically redundant or divergent under predication; it cannot prove cooperative execution.

Represent one bounded reduction tile whose participating invocations write disjoint partials to explicit workgroup storage and later consume the complete staged set. Define the participant set, local coordinates, storage shape/alignment/lifetime, phases, uniform reachability, and the exact dependency that requires visibility. Do not add a barrier or claim backend support here; this ticket constructs the dataflow the synchronization authority will govern.

## Required evidence

The verifier accepts one cooperative tile and rejects overlapping writes, out-of-lifetime reads, missing writers, nonuniform phase reachability, invalid local coordinates, insufficient storage, and a staged read with no producing phase. Zero-extent input retains the reducer's explicit identity without entering a barrier. Every check is perturbed once and observed failing.

## Closes when

The cooperative dataflow is explicit and verifier-owned across schedule and KIR, no synchronization or Metal support is overclaimed, exact public drafts are presented to Tom before acceptance, targeted `tiler-ir`/compiler/reference nextest and Clippy pass, and `admit-the-first-typed-synchronization-point-and-atomic-target-authority` can bind a real point to this dataflow.
