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
