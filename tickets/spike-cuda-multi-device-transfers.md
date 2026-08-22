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

**The evidence environment still needs explicit authorization, but the old platform-policy claim is retired.** This ticket previously quoted `AGENTS.md` as saying that Tiler develops on macOS *only* and that other platforms are unsupported. The current guide says instead that Tiler develops on macOS and has no CI; it does not state the quoted categorical exclusion. It does require Tom's authorization before changing Rust, Xcode, SDK, simulator, GPU, or other host components for a measurement. Therefore the live gate is: the multi-device scope needs CUDA evidence, at least two identified CUDA devices are available, and Tom authorizes the exact host/toolchain evidence environment. `deps.sh` still provisions the Apple Metal toolchain rather than a CUDA environment, so that environment must be recorded rather than inferred from the repository bootstrap.

A second reading is worth stating so it is not mistaken for the first: this ticket can also serve as *design* evidence for the placement/transfer contract without CUDA ever being supported, by naming the realizations a non-Metal backend would have to express. That use needs no policy change and no hardware, but it also cannot mark the ticket done — the exit criteria below require measurements from two real devices.

## Exit criteria

Produce a reproducible experiment and versioned report that separates hard
route feasibility from measured transfer costs and records every unmeasured
topology as `Unknown`. Marking the ticket done requires results from at least two real CUDA
devices plus complete device, driver, runtime, allocation, stream/event, and
failure-boundary provenance; otherwise retain the ticket as deferred evidence
work rather than generalizing from a partial host.

## Trigger check log

- 2026-08-04 — **superseded wording.** This entry said `AGENTS.md` categorically excluded non-macOS platforms. That is not the current repository rule; preserve this line only as the dated trigger reading that has now been corrected.
- 2026-08-09 — **not fired.** No repository evidence identifies a two-device CUDA host or says that the multi-device portability gate now needs CUDA measurements. Tom has not authorized a CUDA host/toolchain measurement boundary in this ticket. The current blocker is that concrete evidence gate, not a categorical macOS-only policy.
- **Recheck supplied — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, and no earlier entry in this log names one either, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — has never been met on this ticket. **Two blockers, one mechanical and one not.** The mechanical half is hardware, and on this host it is decisive: `ls /usr/local/cuda /opt/cuda 2>&1` and `command -v nvidia-smi` both report absence on a macOS host, so no CUDA device — let alone the two independently identified devices the gate requires — can be enumerated here. That establishes *not runnable on this host* rather than *not fired*. **This condition is not mechanically checkable, and saying so is the repair.** The remaining blockers are that the multi-device scope gate must *need* CUDA evidence, and that Tom must authorize a CUDA host and toolchain as an evidence environment — the second being an authorization, which no command can observe. A human must read [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md) for a stated need and the authorization record for Tom's decision. The retired platform-policy claim is not the blocker; the evidence gate is. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).
