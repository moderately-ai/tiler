---
id: multi-device-and-sharding-scope-gate
title: Multi-device and sharding scope gate
status: deferred
priority: p2
dependencies: [transfer-synchronization-and-resource-lifetime-contract]
related: [spike-cuda-multi-device-transfers, spike-metal-multi-device-transfers, device-placement-and-memory-domain-contract, numerical-policy-contract]
scopes: [research/distributed]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, decision, distributed, sharding]
---
Decide whether multi-device placement and distributed tensor planning enter an
early product milestone or remain a deliberately reserved later execution
layer. Do not treat multiple devices as another intra-kernel schedule axis.

The gate must decide:

- whether frontends may submit semantic collectives, sharding constraints, or
  only unsharded global tensor programs;
- global logical tensor values versus explicit local shard resources;
- symbolic meshes/affinities versus concrete runtime device ordinals;
- required, preferred, and forbidden placement constraints;
- physical sharding propagation, reshard enforcers, collective insertion, and
  local-shard derivation;
- numerical order contracts for collective reductions;
- multi-artifact programs, topology guards, queues/timepoints, cancellation,
  partial submission, and fallback safety; and
- which extension points must exist now even if execution remains deferred.

Use StableHLO collectives, OpenXLA Shardy, XLA Send/Recv, IREE
Flow/Stream/HAL, CUDA peer topology, and Metal multi-device restrictions as
primary precedent. Produce a scope ADR and follow-up tickets only for the
admitted milestone.

## Activation gate

Keep this ticket deferred until Tiler proposes either semantic collectives,
compiler-selected sharding, or execution across more than one live device.
Activate it before any of those concepts enter a public IR or runtime contract.
Hardware-specific transfer spikes are evidence for this decision, not
prerequisites for the current single-device value proof.

## Exit criteria

Mark the gate done only with an accepted ADR that fixes whether collectives and
sharding are semantic inputs, compiler-selected physical properties, or both;
defines the program/resource/execution ownership boundary; and identifies the
minimum topology, numerical, lifetime, failure, and cost evidence required
before implementation. If the triggering workload does not justify the
surface, record explicit deferral without adding placeholder IR nodes.

## Initial research synthesis

The first pass on 2026-07-19 found that explicit semantic collectives and
compiler-inserted collectives occupy different layers. StableHLO collectives
define global tensor results, whereas OpenXLA Shardy represents distribution
over symbolic meshes and physical planning may insert resharding or
communication. IREE then lowers target affinities into explicit asynchronous
resources and execution dependencies.

The current single-device `KernelProgram` remains a sound initial boundary.
Multi-device support is a program-level extension requiring topology,
multi-resource lifetime, queues/timepoints, communication costs, collective
numerics, partial-failure semantics, and likely multiple target artifacts. It
must not be smuggled into `KernelSchedule` as another coordinate dimension.

Primary starting points:

- https://openxla.org/stablehlo/spec#collectives
- https://openxla.org/shardy/sharding_representation
- https://openxla.org/shardy/propagation
- https://openxla.org/xla/operation_semantics#send
- https://iree.dev/reference/mlir-dialects/Stream/
