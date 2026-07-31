---
id: realize-parallel-reduction-strategies-on-metal
title: Realize parallel reduction strategies on Metal
status: todo
priority: p1
dependencies: [implement-the-target-neutral-multi-pass-reduction-strategy, implement-the-single-workgroup-synchronized-reduction-strategy, declare-a-required-gpu-family-in-the-artifact, construct-and-bind-the-first-authoritative-metal-compile-profile]
related: [implement-parallel-reduction-strategies]
scopes: [implementation/metal, implementation/build, implementation/runtime, implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The accepted multi-pass and synchronized single-workgroup reduction programs lower, package, preflight, and execute on a qualified Metal host without inventing backend support from source syntax or successful compilation.

## Implementation keys

Map every target-neutral synchronization/storage/dispatch requirement through typed Metal facts and exact live-device or prepared-pipeline authorities at their real phases. Emit explicit workgroup memory and barriers only for verified points. Preserve multi-pass temporary lifetimes and command ordering through final device use. Reuse artifact route requirements and the one-way preparation/commit boundary; do not add Apple vocabulary to the neutral artifact.

Primary Metal documentation and retained host measurements must establish the supported realization. Compilation success is not a capability fact. An unavailable qualified host reports the missing environment rather than converting an unrun path into a guarantee.

## Required evidence

Both strategies execute against the reference on a qualified host, or the exact unavailable predicate is retained. Negative fixtures refuse missing family/feature authority, insufficient prepared capacity, insufficient local memory, and invalid synchronization realization before routing commit. Command-buffer terminal success precedes readback and asynchronous resources survive final use.

## Closes when

Metal lowering, artifact, build, and runtime paths agree with the target-neutral contracts; public backend/runtime boundaries are reviewed by Tom; every check is mutation-proved; and targeted tests/Clippy plus `make full` pass.

## Graph maintenance

- Follow both target-neutral strategies, backend-neutral route requirements, and the authoritative Metal compile profile explicitly; scope collision is not prerequisite evidence.
- Keep measured crossover and winner activation in `calibrate-and-activate-parallel-reduction-selection` after executable Metal evidence exists.
- Split a named hardware measurement when the qualified host is unavailable; do not convert compilation success into feature or performance evidence.
