---
id: multi-device-and-sharding-scope-gate
title: Multi-device and sharding scope gate
status: todo
priority: p2
dependencies: [device-placement-and-memory-domain-contract, transfer-synchronization-and-resource-lifetime-contract, numerical-policy-contract]
related: []
scopes: [research/distributed]
shared_scopes: []
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
