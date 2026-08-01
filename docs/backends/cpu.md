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

It is a *backend* in the [glossary's sense](../glossary.md#backend-device-and-execution-context-vocabulary) — the responsibility of translating verified physical work into one target representation and declaring the target facts that translation depends on — and nothing here requires it to reproduce the Metal package layout. It states its own governed backend family and executable representation, and whether AOT invocation, artifact assembly, and live execution are separate crates as they are for Metal is a packaging question this sketch does not decide. The bounded scalar vertical below declares both keys from a spike rather than from a crate at all, which is the evidence that the role and the topology are separable.

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

**Fact — that same vertical is evidence for a second record, accepted 2026-07-31.** [ADR 0090](../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) proposes how a backend set composes across the compiler, build, artifact, and runtime boundaries, and takes from this vertical that a second backend needed no production edit and no provider interface at all — it wanted target-profile, emission, payload, provenance, plan, adapter, and execution-context responsibilities and reused the rest. It also names four vocabulary gaps this sketch's own axes expose: no availability phase names a bound host *process*, `ArtifactExecutionPolicy` has no spelling for an interpreted image, `PayloadProvenance` requires Apple-shaped fields, and no capability axis names a target triple, ABI, or data layout — so the CPU vertical's missing vector, mask, tail, scalable-length, cache, and thread axes are inexpressible rather than merely undeclared. This sketch itself stays proposed — an accepted composition model does not accept a CPU backend.

**The third of those four gaps closed on 2026-08-01**, by [`generalize-payload-provenance-beyond-the-apple-shape`](../../tickets/generalize-payload-provenance-beyond-the-apple-shape.md): `PayloadProvenance` carries a `PayloadPlatform`, so a CPU payload states `Unversioned` instead of minting an SDK identity and a deployment minimum, and which fields it owes follows the shape it declares rather than a platform it does not have. The [artifact ABI](../artifact-abi.md) is the normative record of the obligation and of why the widening moved no already-encodable payload's bytes. The other three gaps remain open until their owners close them, and nothing about this sketch's own status changed.
