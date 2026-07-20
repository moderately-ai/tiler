---
id: transfer-synchronization-and-resource-lifetime-contract
title: Transfer synchronization and resource-lifetime contract
status: done
priority: p1
dependencies: [device-placement-and-memory-domain-contract, runtime-execution-contract, index-access-model]
related: []
scopes: [research/transfers]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, transfer, runtime, correctness]
---
Define explicit physical transfer/enforcer and execution-resource contracts.
A transfer means making a logical value accessible at a destination affinity;
it must not assume that every realization is a byte copy.

Cover at minimum:

- source and destination placement/allocation identities;
- logical-view versus backing-allocation byte regions;
- bit-preserving movement versus separately represented dtype conversion;
- copy, alias/import, peer access, migration, direct copy, and host-staged
  mechanisms;
- source-producer, transfer-completion, and destination-consumer dependencies;
- completion tokens, queues, events/fences, cancellation, and failure stages;
- retention of source, destination, staging, view, command, and synchronization
  resources until completion;
- alias/hazard rules and proof requirements for no-copy elimination;
- preflight and fallback-before-partial-work boundaries; and
- stable explain and artifact identities without embedding live device handles.

Propose verifier invariants and small examples for CPU-to-accelerator,
accelerator-to-CPU, same-device materialization, peer transfer, shared backing,
and staged transfer. Use IREE Stream/HAL, MLIR GPU async copies, CUDA runtime,
Metal shared events/resources, PyTorch, and Candle as evidence.

## Initial research synthesis

CUDA peer accessibility is directional and pair-specific; peer access, peer
copy, managed migration, and host staging have different feasibility and
ordering. Metal buffers and queues are device-owned; portable cross-device
movement generally requires distinct resources and explicit shared backing or
staging, while deprecated peer-buffer views are narrowly constrained. Neither
backend treats reachability as synchronization.

The initial transfer abstraction therefore needs source/destination ownership,
a chosen mechanism, copy-versus-alias semantics, two-sided dependency edges,
and lifetime obligations. Same-backend identity is not proof of accessibility,
and host reference counts are not proof that asynchronous GPU use has ended.

Primary starting points:

- https://docs.nvidia.com/cuda/cuda-programming-guide/03-advanced/multi-gpu-systems.html
- https://docs.nvidia.com/cuda/cuda-runtime-api/group__CUDART__MEMORY.html
- https://developer.apple.com/documentation/metal/selecting-device-objects-for-compute-processing
- https://developer.apple.com/documentation/metal/synchronizing-events-across-multiple-devices-or-processes
- https://iree.dev/reference/mlir-dialects/Stream/
- https://mlir.llvm.org/docs/Dialects/GPU/

## Outcome

Delivered the [transfer and lifetime contract](../docs/research/transfers/transfer-synchronization-and-resource-lifetime.md)
and [bounded verifier](../spikes/transfers/README.md). Backend-specific
multi-device transfer measurements and calibrated costs remain deferred.
