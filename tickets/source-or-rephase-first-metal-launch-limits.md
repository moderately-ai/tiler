---
id: source-or-rephase-first-metal-launch-limits
title: Source or rephase the first Metal launch limits
status: todo
priority: p0
dependencies: []
related: [restore-replayable-apple-compatibility-evidence, prototype-metal-runtime-proof]
scopes: [research/apple-targets, implementation/compiler, implementation/artifact, implementation/runtime, contracts/foundation, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, launch, feasibility, target-profiles]
---
## User-visible outcome

The first Metal profile proves the bounded serial-sum grid extent 4 and workgroup size 1 from authority available at the phase that consumes each value, or correctly defers the predicate to live-device and prepared-kernel preflight without inventing a compile-time limit.

## Facts and measurement boundary

**Fact:** `GridAxisThreads` means dispatched thread extent along one grid axis and `WorkgroupThreads` means threads per workgroup. The governed placeholder offers 65,535 and 1; those are internally usable constants, not sourced Metal family facts.

**Fact:** Apple feature tables report 1,024 as the theoretical maximum threads per threadgroup and explicitly direct readers to `MTLComputePipelineState.maxTotalThreadsPerThreadgroup` for the actual compiled-function maximum. The table does not state a general compute-grid maximum. `thread_position_in_grid` delivered as `uint` bounds an invocation index; it does not prove a grid-capacity extent.

**Inference:** promoting 1,024 into `CompileProfile`, deriving 65,535 from a `uint`, or treating a prepared-pipeline observation as a portable profile fact can admit an infeasible launch. The bounded program needs extent 4 and workgroup 1, not either convenient maximum.

**Measurement boundary:** a conservative normative minimum may be a compile-profile guarantee only when the primary source says it applies to the exact profile. Device-reported maxima are `LiveDevicePreflight`; `maxTotalThreadsPerThreadgroup` is `PreparedKernelPreflight`; concrete launch validation is later still.

## Implementation and experiment keys

Search primary Metal API and feature-family documentation for guarantees sufficient for extent 4 and workgroup 1 on the exact Apple9/macOS profile. If both survive as compile guarantees, cite and encode only those conservative bounds. Otherwise represent the unresolved requirement at its truthful later phase, carry a typed deferred predicate into the artifact, bind the live device and prepared pipeline before routing commit, and keep compile search from calling the later fact proven.

## Required evidence

Provide reproducible source checks, one positive bounded serial-sum route, and negative cases for grid extent 5 or another value outside the declared conservative guarantee and for a workgroup one past the prepared pipeline capacity. Phase tests must reject `DeviceRuntime` or `PreparedKernel` evidence inserted at `CompileProfile`. Mutation tests must prove that replacing extent 4 with 65,535 or the prepared value with theoretical 1,024 is detected.

## Closes when

Both launch quantities have exact typed authorities at their real phases, the profile and artifact contain no 65,535/1,024 conflation, later facts are checked before routing commit, and the bounded compile/run proof passes or reports the unavailable required environment explicitly.

## Graph maintenance

This ticket blocks `construct-and-bind-the-first-authoritative-metal-compile-profile`. Keep `prototype-metal-runtime-proof` related because it owns the existing pre-commit pipeline check, and keep `restore-replayable-apple-compatibility-evidence` related because compiler acceptance is not a device or pipeline capacity guarantee.
