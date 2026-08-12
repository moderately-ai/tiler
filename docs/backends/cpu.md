---
schema: "tiler-doc/v1"
id: "tiler.contract.cpu-backend"
kind: "contract"
title: "CPU backend and SIMD target profile"
topics: ["backends", "cpu", "simd", "target-profiles"]
contract_status: "mixed"
implementation_status: "spike-only"
evidence: ["tiler.research.extensions.backend-provider-composition", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.placement.device-memory-domains", "tiler.research.transfers.synchronization-lifetime"]
---

# CPU backend and SIMD target profile

**Status:** mixed — the bounded scalar production boundary is accepted by [ADR 0110](../decisions/0110-split-the-bounded-scalar-cpu-backend-at-the-production-process-boundaries.md) and remains spike-only; SIMD, scalable-vector, and threaded target detail remains proposed

A CPU backend uses ADR 0043's target-neutral feasibility interface without pretending CPU workers are GPU threadgroups. Its declared profile identifies target triple, CPU/features, ABI/data layout, address widths, execution model, and the exact operation, dtype, numerical, memory, and resource facts the selected representation consumes.

It is a *backend* in the [glossary's sense](../glossary.md#backend-device-and-execution-context-vocabulary) — the responsibility of translating verified physical work into one target representation and declaring the target facts that translation depends on — and it does not reproduce the Metal package layout. ADR 0110 accepts three concrete packages split at production process boundaries: `tiler-cpu-image` owns the governed scalar-image grammar, checked codec, and pure decoded-image executor; `tiler-cpu` owns CPU target declaration, verified-KIR translation, and payload production; and `tiler-cpu-runtime` owns live host qualification, the exact runtime adapter, allocation, execution, and completion. None exists as a production crate yet, and the bounded scalar vertical remains the executable evidence rather than the implementation.

## Accepted bounded scalar production profile

The first production profile is explicitly selected scalar, single-threaded F32 execution. The image is a versioned backend representation executable without compiler objects. The producer reaches it only from verified structured KIR; the runtime reaches it only by decoding carried artifact bytes and validating the exact backend family, representation, target profile, launch, buffers, and numerical realization against the live host. `tiler-reference` remains an independent oracle and is never called by the backend.

Every unsupported value type, operation, address space, vector or packed value, barrier, cooperative scope, thread model, and numerical realization is a typed refusal. The adapter takes an explicit resource policy with no default. A caller may deliberately select an unbounded alpha policy, or a bounded policy that is checked against the decoded image's worst-case scalar operations and allocation bytes before the one-way routing commit. No policy truncates work or silently falls back.

The scalar image remains a supported correctness/debug representation if a native CPU compiler is later admitted. A native tier has its own versioned representation and explicit selection; it does not replace or shadow the scalar image. Metal, native CPU, and scalar CPU attempts never substitute for one another implicitly.

The production package split is an accepted ownership boundary, not a `BackendProvider` aggregate or registry. `tiler-cpu-image` depends on neither compiler/build nor runtime routing; `tiler-cpu` does not execute artifacts or implement `RuntimeAdapter`; and `tiler-cpu-runtime` never depends on compiler, build, or KIR. Producer and adapter join through artifact identity and received bytes.

The accepted scalar profile does not add a host-process availability phase merely to rename adapter-local work. It declares no governed availability row. A later CPU fact that genuinely differs between a live device and a bound host process must first trigger and decide that phase vocabulary.

## Proposed vector, scalable-vector, and threaded profile

Vector legality is contextual on exact operation, dtype, fixed or scalable shape, mask/tail support, memory domain, width, alignment, and numerical execution path. These are operation-specific realization requirements rather than one flat conjunction of independent dimensions. LLVM-style legality and cost providers are useful implementation precedent, but their provider/version and target-machine configuration must be explicit. Preferred vector width, cache fit, register pressure, spills, task granularity, and oversubscription are cost facts, not correctness guarantees.

Live feature detection or scalable-vector length may defer a specialized variant to process preflight only after a runtime authority earns those facts from the host. CPU caches are transparent cost-model levels; stack, thread-local storage, heap buffers, and explicit scratch are addressable resource contracts. Thread/task barriers state participants, memory ordering, and runtime ownership rather than borrowing GPU barrier semantics.

No vector, scalable-vector, mask/tail, threading, cache, or native-code field-level public surface in this section is accepted merely because the scalar package boundary is. Q-PLAN-011 owns the remaining decisions and evidence.

## Traceability

This mixed contract owns the accepted bounded scalar backend boundary and the proposed CPU/SIMD target-profile extensions, not tensor semantics or consumer workload state. Placement and transfer research supplies its physical-resource boundary.

The [bounded scalar CPU backend vertical](../../spikes/target-profiles/scalar-cpu-vertical/README.md) is the only execution evidence. It declares a target triple, ABI/data layout, address width, and scalar execution model, carries one F32 payload from that profile to bitwise agreement with the reference evaluator, and leaves every vector, scalable-vector, mask/tail, threading, and cache claim undeclared and therefore `Unknown`. Its multi-entry route, shared-allocation pairing, serial loops, and operation-level refusals are implemented but were not exercised by the retained run, so production acceptance does not promote them to tested guarantees.

[ADR 0090](../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) accepts responsibility-based backend composition and records why the scalar vertical needed no monolithic provider interface. [ADR 0110](../decisions/0110-split-the-bounded-scalar-cpu-backend-at-the-production-process-boundaries.md) accepts the scalar production ownership, refusal, resource-policy, and retirement boundaries. The scalar implementation remains spike-only and Q-PLAN-011 stays open for vector and threaded execution.

One historical vocabulary gap is already closed: `PayloadProvenance` carries a `PayloadPlatform`, so a CPU payload states `Unversioned` instead of inventing an Apple SDK and deployment minimum. The remaining gaps stay with their owners; this contract does not add generic axes or phases ahead of a consumer.
