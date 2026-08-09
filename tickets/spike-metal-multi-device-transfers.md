---
id: spike-metal-multi-device-transfers
title: Spike Metal multi-device transfers
status: deferred
priority: p2
dependencies: [transfer-synchronization-and-resource-lifetime-contract]
related: []
scopes: [research/metal-transfers]
shared_scopes: [project/tickets]
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

## Activation gate

Run only when the multi-device scope gate needs Metal evidence and a host with
more than one independently addressable Metal device is available. Apple
Silicon unified memory on one device does not satisfy that prerequisite; record
partial platform coverage explicitly.

## Exit criteria

Produce a reproducible experiment and versioned report that distinguishes
same-device aliasing, blits, host staging, cross-device synchronization, and
unsupported routes. Marking the ticket done requires results from at least two real,
independently addressable Metal devices with complete OS/SDK/device/storage-mode
provenance; all unavailable platform families remain `Unknown` rather than
being inferred from one configuration.

## Trigger check log

- 2026-08-04 — **not fired.** No host in this project has more than one independently addressable Metal device — every retained measurement is a single Apple-silicon unified-memory GPU — and the multi-device scope gate has not activated. Recheck: the gate's own two conditions.
- 2026-08-09 — **not fired.** The retained measurements still name one independently addressable Apple-silicon GPU per host, and [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md) remains deferred. Unified memory and multiple command queues on one device do not satisfy this ticket's two-device prerequisite.
