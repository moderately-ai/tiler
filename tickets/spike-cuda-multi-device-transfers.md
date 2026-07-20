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

## Activation gate

Run only when the multi-device scope gate needs CUDA evidence and at least two
identified CUDA devices are available. A single-device host or simulated result
cannot mark the ticket done; record partial topology coverage explicitly.

## Exit criteria

Produce a reproducible experiment and versioned report that separates hard
route feasibility from measured transfer costs and records every unmeasured
topology as `Unknown`. Marking the ticket done requires results from at least two real CUDA
devices plus complete device, driver, runtime, allocation, stream/event, and
failure-boundary provenance; otherwise retain the ticket as deferred evidence
work rather than generalizing from a partial host.
