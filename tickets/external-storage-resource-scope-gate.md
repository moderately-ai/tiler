---
id: external-storage-resource-scope-gate
title: External storage resource scope gate
status: todo
priority: p2
dependencies: [device-placement-and-memory-domain-contract, transfer-synchronization-and-resource-lifetime-contract]
related: []
scopes: [research/external-storage]
shared_scopes: []
paths: []
tags: [tiler-research, decision, storage, deferred]
---
Determine the proper boundary for file-backed parameters, memory mapping,
asynchronous disk I/O, host caches, eviction, persistence, and out-of-core
execution. Treat external storage as a resource/orchestration concern rather
than another GPU address space.

Decide:

- stable external-resource identity without coupling semantic tensors to file
  paths or handles;
- typed tensor metadata versus opaque aligned byte ranges;
- import/map/read/stage operations and their asynchronous completion contract;
- ownership, persistence, mutability, caching, eviction, and failure behavior;
- whether the compiler merely expresses residency requirements or also plans
  streaming and spills; and
- what is explicitly deferred beyond macro-local kernel/program compilation.

Use IREE parameter resources/archives, IREE Stream/HAL, MLIR bufferization and
ownership, TVM Relax, and JAX host offloading as primary precedents. Produce a
scope ADR; default to deferral unless measurements demonstrate an initial
whole-program requirement.

## Initial research synthesis

The first pass on 2026-07-19 supports deferring general out-of-core planning.
IREE models parameter archives as external, asynchronously accessible,
device-aware resources and may realize them through mapping, caches, device
I/O, or staged reads. The file representation uses opaque aligned byte ranges
rather than making file paths part of tensor semantics. JAX host offloading
similarly combines placement and rematerialization policy above kernel code.

Tiler should reserve external resource identity, asynchronous availability,
and staging interfaces, but should not initially own file handles, persistence,
eviction, or disk scheduling. Those require a broader orchestration lifetime
than a macro-local compiled tensor program.

Primary starting points:

- https://iree.dev/reference/mlir-dialects/IOParameters/
- https://iree.dev/guides/parameters/
- https://iree.dev/reference/mlir-dialects/Stream/
- https://docs.jax.dev/en/latest/notebooks/host-offloading.html
