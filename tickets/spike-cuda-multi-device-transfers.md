---
id: spike-cuda-multi-device-transfers
title: Spike CUDA multi-device transfers
status: deferred
priority: p2
dependencies: [transfer-synchronization-and-resource-lifetime-contract]
related: []
scopes: [research/cuda-transfers]
shared_scopes: [project/tickets]
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

**Precondition the hardware gate does not state, and it comes first.** `AGENTS.md` records the platform policy as "Tiler develops on macOS only; other platforms are unsupported rather than maintained as untested branches", and `deps.sh` provisions the Apple Metal toolchain and nothing else. CUDA is therefore outside the supported platform set today, and no arrangement of hardware changes that: two CUDA devices on a Linux host would satisfy the gate above and still leave this ticket unrunnable under the policy. **Widening the platform policy is Tom's decision and it is not a condition that arrives by waiting** — nothing in the work graph produces it, so this ticket does not become ready on its own. Treat it as gated on that decision first and on hardware second; if the decision is made, record it and the ticket's own gate applies unchanged from there.

A second reading is worth stating so it is not mistaken for the first: this ticket can also serve as *design* evidence for the placement/transfer contract without CUDA ever being supported, by naming the realizations a non-Metal backend would have to express. That use needs no policy change and no hardware, but it also cannot mark the ticket done — the exit criteria below require measurements from two real devices.

## Exit criteria

Produce a reproducible experiment and versioned report that separates hard
route feasibility from measured transfer costs and records every unmeasured
topology as `Unknown`. Marking the ticket done requires results from at least two real CUDA
devices plus complete device, driver, runtime, allocation, stream/event, and
failure-boundary provenance; otherwise retain the ticket as deferred evidence
work rather than generalizing from a partial host.
