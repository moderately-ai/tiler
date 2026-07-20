---
id: spike-cuda-multi-device-transfers
title: Spike CUDA multi-device transfers
status: deferred
priority: p2
dependencies: [transfer-synchronization-and-resource-lifetime-contract]
related: []
scopes: [research/cuda-transfers]
shared_scopes: []
paths: []
tags: [tiler-research, spike, cuda, transfer, measurement]
---
Measure the concrete CUDA realizations and failure points behind the abstract
placement/transfer contract.

Cover directional peer capability, peer enablement and its scope, peer
load/store, synchronous and asynchronous peer copies, source/destination event
ordering, host-staged fallback, pinned host memory, managed-memory behavior,
allocation lifetime, and topologies with as many available devices as
possible.

Record GPUs, driver/runtime versions, topology, byte counts, streams/events,
observed compatibility failures, latency/bandwidth/overlap, allocator effects,
and whether preflight catches the failure before device work. Treat P2P
feasibility as a hard constraint and measured route costs separately.
