---
schema: "tiler-doc/v1"
id: "tiler.contract.cpu-backend"
kind: "contract"
title: "Proposed CPU/SIMD target profile"
topics: ["backends", "cpu", "simd", "target-profiles"]
contract_status: "proposed"
implementation_status: "not-started"
governed_by: ["ADR-0043", "ADR-0047"]
evidence: ["tiler.research.placement.device-memory-domains", "tiler.research.transfers.synchronization-lifetime"]
---

# Proposed CPU/SIMD target profile

**Status:** future contract sketch; not an implementation commitment

A CPU backend uses ADR 0043's target-neutral feasibility interface without
pretending CPU workers are GPU threadgroups. Its declared profile identifies
target triple, CPU/features, ABI/data layout, address widths, fixed and scalable
vector models, threading runtime contract, and governed memory/execution
scopes.

Vector legality is contextual on operation, dtype, fixed or scalable shape,
mask/tail support, address space, width, and alignment. LLVM-style legality and
cost providers are useful implementation precedent, but their provider/version
and target-machine configuration must be explicit. Preferred vector width,
cache fit, register pressure, spills, task granularity, and oversubscription are
cost facts, not correctness guarantees.

Live feature detection or scalable-vector length may defer a specialized
variant to device/process preflight. A scalar or conservative generic variant
remains packaged. CPU caches are transparent cost-model levels; stack,
thread-local storage, heap buffers, and explicit scratch are addressable
resource contracts. Thread/task barriers state participants, memory ordering,
and runtime ownership rather than borrowing GPU barrier semantics.

## Traceability

This proposed contract owns the CPU/SIMD target-profile sketch, not tensor
semantics or runtime implementations. Placement and transfer research supplies
its physical-resource boundary; implementation evidence does not yet exist.
