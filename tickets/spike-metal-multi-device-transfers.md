---
id: spike-metal-multi-device-transfers
title: Spike Metal multi-device transfers
status: todo
priority: p2
dependencies: [transfer-synchronization-and-resource-lifetime-contract]
related: []
scopes: [research/metal-transfers]
shared_scopes: []
paths: []
tags: [tiler-research, spike, metal, transfer, measurement]
---
Measure the concrete Metal realizations and failure points behind the abstract
placement/transfer contract.

Cover exact-device resource ownership, shared-system-memory staging,
shared-event ordering across devices, managed-resource synchronization,
peer-group availability, deprecated remote buffer views, and Apple Silicon
versus any available discrete/external GPU configuration. Distinguish alias,
direct blit, staged copy, and unsupported routes.

Record devices, OS/SDK versions, storage modes, byte counts, synchronization,
observed compatibility failures, latency/bandwidth, allocation/resource
lifetime, and whether preflight catches the failure before device work. Do not
generalize beyond measured configurations.
