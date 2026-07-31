---
schema: "tiler-doc/v1"
id: "tiler.contract.cpu-backend"
kind: "contract"
title: "Proposed CPU/SIMD target profile"
topics: ["backends", "cpu", "simd", "target-profiles"]
contract_status: "proposed"
implementation_status: "not-started"
evidence: ["tiler.research.placement.device-memory-domains", "tiler.research.transfers.synchronization-lifetime", "tiler.spike.target-profiles.scalar-cpu-vertical"]
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
its physical-resource boundary.

The [bounded scalar CPU backend vertical](../../spikes/target-profiles/scalar-cpu-vertical/README.md)
is the only implementation evidence, and it deliberately covers the *scalar*
half of this sketch alone. It declares a target triple, ABI/data layout, address
width, and scalar execution model, carries one `f32` payload from that profile to
a bitwise agreement with the reference evaluator, and leaves every vector,
scalable-vector, mask/tail, threading, and cache claim above undeclared and
therefore `Unknown`. Its README records which parts of the neutral target-profile
and artifact vocabularies had no CPU referent. This contract stays proposed and
Q-PLAN-011 stays open: one executed scalar vertical is not an implementation
plan, and nothing here has been accepted.
