---
id: device-placement-and-memory-domain-contract
title: Device placement and memory-domain contract
status: done
priority: p1
dependencies: [target-profile-feasibility-model, kernel-program-buffer-plan]
related: []
scopes: [research/placement, contracts/core]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, placement, memory]
---
Define the target-independent physical property model for where a tensor value
is accessible and how that differs from layout, storage identity, lifetime, and
ownership. Preserve the initial device-neutral semantic graph and single-device
`KernelProgram` while reserving interfaces that do not prevent later
multi-device planning.

Deliver a research memo and proposed contract covering:

- symbolic placement/affinity rather than concrete runtime ordinals above
  execution lowering;
- memory domains as a capability graph, not a linear disk/RAM/VRAM hierarchy;
- accessibility, visibility, coherence, capacity, alignment, allocation,
  import, and legal movement/aliasing edges;
- the separation of semantic value, placement, memory domain, layout/encoding,
  transfer, lifetime, and ownership;
- placement as a required physical property with transfer, import,
  materialization, packing, or recomputation as possible enforcers;
- hard feasibility versus topology-dependent cost; and
- the boundary between compiler requirements and runtime-owned device handles,
  allocators, pools, and concrete storage modes.

Use IREE Flow/Stream/HAL, MLIR tensor/bufferization/memref/GPU memory spaces,
OpenXLA sharding, and Metal/CUDA memory topology as primary precedents. Record
an ADR for the durable layer boundary.

## Initial research synthesis

The first pass on 2026-07-19 supports three layers: device-neutral tensor
semantics; physical placement, sharding, and enforcers; then execution
resources, queues, synchronization, and ownership. IREE's documented
Flow-to-Stream-to-HAL lowering is the closest integrated precedent. MLIR keeps
layout and target-specific memory space independent and delays allocation,
copying, aliasing, and ownership decisions until bufferization. OpenXLA Shardy
models distribution over symbolic logical device meshes.

A linear `Disk < RAM < VRAM < Scratch` enum is rejected as the starting model.
Pinned memory is an accessibility property, unified memory is a topology, GPU
workgroup/register storage has execution-scoped lifetime, and file-backed data
has external I/O semantics. Model target-dependent domains and legal
access/movement edges instead.

Primary starting points:

- https://iree.dev/reference/mlir-dialects/Stream/
- https://iree.dev/reference/mlir-dialects/HAL/
- https://mlir.llvm.org/docs/Bufferization/
- https://mlir.llvm.org/docs/Dialects/Builtin/#memref-type
- https://openxla.org/shardy/sharding_representation
- https://developer.apple.com/documentation/metal/choosing-a-resource-storage-mode-for-apple-gpus

## Outcome

Delivered the [placement and memory-domain report](../docs/research/placement/device-placement-and-memory-domains.md),
[executable model](../spikes/placement/README.md), and accepted
[ADR 0047](../docs/decisions/0047-model-placement-as-physical-properties.md).
Distributed scheduling and external-storage semantics remain deferred.
