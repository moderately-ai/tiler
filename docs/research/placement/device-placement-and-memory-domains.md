---
schema: "tiler-doc/v1"
id: "tiler.research.placement.device-memory-domains"
kind: "research"
title: "Device placement and memory-domain contract"
topics: ["placement", "memory-domains", "devices", "physical-properties"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.cpu-backend", "tiler.contract.metal-backend"]
adopted_by: ["ADR-0047"]
reproduced_by: ["tiler.spike.placement"]
ticket: "device-placement-and-memory-domain-contract"
---

# Device placement and memory-domain contract

**Status:** completed research adopted by ADR 0047; distributed planning remains deferred

## Question

How can `KernelProgram` require that values are usable where work executes
without making device choice semantic, embedding runtime device ordinals in
portable plans, or assuming memory is a linear disk/RAM/VRAM hierarchy?

The database-optimizer analogy is a required physical property plus enforcers:
a consumer requires a value accessible at an execution affinity, and the
planner may retain an existing placement or insert transfer, import,
materialization, packing, or recomputation. The analogy stops at visibility:
GPU accessibility can depend on runtime topology and a synchronization event,
so the delivered property is not just a timeless label on a node.

## Facts from primary precedents

### Compiler IR

- [IREE Stream](https://iree.dev/reference/mlir-dialects/Stream/) is the closest
  integrated precedent. It lowers tensor data flow into explicitly scheduled
  asynchronous work, attaches symbolic affinities, represents transfer between
  affinities, classifies resource lifetimes, carries resource sizes, and uses
  timepoints to say when results become available. Devices and allocations are
  still unresolved at that level.
- [IREE HAL](https://iree.dev/reference/mlir-dialects/HAL/) subsequently models
  runtime devices, queue affinity, allocation, copies, file reads/writes, and
  wait/signal fences. This supports a boundary between symbolic physical
  planning and runtime execution resources.
- MLIR's [`memref`](https://mlir.llvm.org/docs/Dialects/Builtin/#memreftype)
  keeps layout maps and target-specific memory-space attributes as independent
  parameters. [Bufferization](https://mlir.llvm.org/docs/Bufferization/)
  separately decides allocation, aliasing, and copying; the
  [GPU dialect](https://mlir.llvm.org/docs/Dialects/GPU/#gpualloc-gpuallocop)
  makes asynchronous allocation and host-shared accessibility explicit.
- [OpenXLA Shardy](https://openxla.org/shardy/sharding_representation) binds
  tensor sharding to a named logical mesh, then separately maps logical device
  IDs. This is evidence for symbolic device structure above concrete runtime
  handles, but sharding is a later Tiler layer rather than ordinary placement
  of one unsharded value.

### Device APIs

- Metal resources belong to the `MTLDevice` that created them. Even GPUs in one
  peer group need a destination-device resource and an explicit copy; remote
  views are copy-only. See [Metal multi-GPU transfers](https://developer.apple.com/documentation/metal/transferring-data-between-connected-gpus).
- Metal storage mode combines location and access policy, and its realization
  varies by hardware. On Apple GPUs, shared and private storage can both use
  system memory while only shared is CPU-accessible. Managed resources on
  discrete systems require explicit synchronization. See
  [`MTLStorageMode`](https://developer.apple.com/documentation/metal/mtlstoragemode),
  [Apple GPU storage modes](https://developer.apple.com/documentation/metal/choosing-a-resource-storage-mode-for-apple-gpus),
  and [managed synchronization](https://developer.apple.com/documentation/metal/synchronizing-a-managed-resource-in-macos).
- CUDA unified virtual addressing does not imply universal accessibility,
  residency, or coherence. Managed memory can migrate; mapped pinned host
  memory is directly device-addressable; peer access depends on a device pair
  and runtime enablement; virtual-memory mappings have per-device access
  control. See [CUDA unified and system memory](https://docs.nvidia.com/cuda/cuda-programming-guide/02-basics/understanding-memory.html)
  and [CUDA peer access](https://docs.nvidia.com/cuda/cuda-programming-guide/#peer-to-peer-memory-access).
- CUDA streams have explicit context-scoped ordering rules, while Metal queues
  and resources are created from a particular device. Queue identity therefore
  belongs to execution lowering, not portable placement identity. See
  [CUDA stream synchronization](https://docs.nvidia.com/cuda/cuda-runtime-api/stream-sync-behavior.html)
  and [`MTLDevice`](https://developer.apple.com/documentation/metal/mtldevice).

## Inferences

1. A scalar storage-class enum is unsound. `Private` may mean unified physical
   memory with GPU-only API access or discrete VRAM; pinned host memory can be
   accessed by CPU and GPU; peer accessibility is pairwise and conditional;
   workgroup/register storage has execution-scoped lifetime; external storage
   requires I/O rather than loads by a tensor kernel.
2. Device, memory domain, allocation, and value placement are distinct. One
   device exposes several domains; one domain may be accessible by several
   executors; one logical value may have several versioned materializations.
3. Accessibility is directional and mode-specific. Read permission does not
   imply write or atomic permission, and addressability does not imply coherent
   visibility of the latest version.
4. Symbolic affinities belong in selected physical programs. Runtime handles,
   contexts, ordinals, allocators, pools, queues, and storage-mode objects are
   adapter-owned bindings with runtime-scoped identity.
5. External storage is not merely a slow memory domain. Files and parameter
   archives have I/O, persistence, failure, partial-transfer, and ownership
   semantics. They connect to addressable domains through explicit import/read
   stages.

## Proposed normalized model

All identities are stable newtypes. The spelling is illustrative rather than a
committed Rust API.

```text
PlacementModel {
  symbolic_affinities,
  memory_domain_classes,
  accessibility_edges,
  movement_and_import_edges,
  capability_schema,
}

SymbolicAffinity {
  id,
  device_class,              // CPU, Metal, CUDA, extension class
  selector,                  // required predicates, not runtime ordinal
  relation_constraints,      // same-as/different-from/peer-capable when needed
}

MemoryDomainClass {
  id,
  scope,                     // system, device, dispatch, workgroup, thread...
  allocation_capability,
  import_capability,
  capacity_contract,
  alignment_contract,
  supported_resource_kinds,
}

AccessibilityEdge {
  affinity,
  domain,
  modes,                     // read/write/read-write/atomic as governed modes
  addressability,
  visibility_scope,
  coherence_protocol,
  required_completion,
}

MovementEdge {
  source_domain,
  destination_domain,
  mechanism,                 // copy, peer copy, migration, import, extension
  capability_predicates,
  preserved_encoding,
  source/destination access,
  completion_and_coherence_result,
  hard_size/alignment/range limits,
  cost_parameters,
}
```

Domains and edges form a directed capability multigraph. Several edges may
connect the same domains with different mechanisms, queues, constraints, or
costs. Domain names are governed provider keys; no portable ordering is
inferred from them.

### Required and delivered properties

```text
PlacementRequirement {
  consumer_affinity,
  admitted_domain_set,
  access_mode,
  logical_value_version,
  storage_encoding,
  byte_range,
  alignment,
  availability_dependency,
}

DeliveredPlacement {
  materialized_value,
  allocation_binding,
  domain,
  storage_encoding,
  authoritative_version,
  visible_to,
  available_after,
  ownership,
}
```

`visible_to` is justified by an accessibility edge plus the required coherence
and completion state; it is not copied blindly from the domain declaration.
Every stage read consumes a delivered placement satisfying its requirement.

The planner may enforce a requirement with:

- direct binding when the current allocation and version already satisfy it;
- an explicit movement/import stage producing a destination materialization;
- materialization or repacking when layout/encoding differs;
- recomputation at the destination when the producer is pure and the numerical
  contract permits the same result;
- rejection when no legal path exists.

Transfer preserves the declared bytes/encoding unless its contract explicitly
includes conversion. Repacking is not silently called a transfer. Transfer
normally creates a new allocation and materialized version; aliasing remote
storage requires an explicit shared-backing/import contract.

### Separation of concerns

| Concept | Meaning | Identity owner |
| --- | --- | --- |
| Semantic value | Tensor result and mathematical contract | Semantic graph |
| Semantic `TargetProperty` | Explicit target-dependent program input affecting shapes/values | Semantic root binding |
| Symbolic affinity | Required class/relationship of an execution location | Physical program |
| Live device | Runtime device/context instance satisfying an affinity | Runtime adapter/cache scope |
| Memory domain | Capability class for allocation and access | Target/runtime profile |
| Allocation | Concrete byte storage in one domain | Runtime allocator/pool |
| View/layout/encoding | Interpretation and addressing of allocation bytes | Buffer/view plan |
| Materialized placement | Value version stored in an allocation and visible after dependencies | Physical program/runtime state |
| Transfer/import/repack | Enforcer producing a required physical property | Explicit program stage |
| Lifetime/ownership | When storage remains valid and who releases it | Buffer plan/runtime adapter |
| Queue/timepoint | Submission and completion resource | Execution lowering/runtime |

A semantic `TargetProperty` root cannot be inferred from the chosen physical
affinity. It is bound once under ADR 0008 before semantic evaluation. Placement
may use that already-bound semantic environment, but choosing another device
cannot overwrite the value or change tensor meaning.

### Device identity and runtime binding

Portable plans refer to `SymbolicAffinityId`. At preflight the runtime binds it
to a `LiveDeviceKey` containing at least provider/runtime-instance identity and
a provider-defined stable device token. A bare ordinal is insufficient: CUDA
ordinal 0 must be scoped to its runtime instance, and a Metal resource remains
tied to the device instance that created it. Object addresses alone are not
stable portable identities.

Runtime binding verifies all relational constraints and instantiates domain,
accessibility, movement, allocator, and execution capabilities. Concrete
storage modes are provider choices realizing a declared domain contract. They
may enter plan-routing and runtime-cache provenance, but handles and volatile
device topology do not enter portable semantic identity.

### `KernelProgram` integration and initial slice

The future-capable `KernelProgram` boundary gains:

```text
KernelProgram {
  placement_profile,
  symbolic_affinities,
  ...
}

MaterializedValue {
  placement_requirement,
  ...
}

ProgramStage {
  execution_affinity,
  reads/writes with placement requirements,
  body: ScheduledKernel | MaterializeCopy | PlacementEnforcer
      | ValidationEnforcement | OpaqueCall,
}
```

The first executable profile remains intentionally smaller:

- exactly one symbolic device affinity bound to one live device;
- one ordered command stream;
- all scheduled/opaque stages and temporaries use that affinity;
- inputs must be preflight-proven accessible in an admitted domain;
- outputs are allocated in an admitted runtime-provided domain;
- no cross-device movement stage, sharding, collective, external I/O, or
  placement search is executable.

This profile is a verifier restriction, not an IR impossibility. Later profiles
can enable explicit placement-enforcer stages, multiple affinities, and
timepoints without changing semantic tensor nodes or reinterpreting existing
single-device programs. The follow-on transfer contract must define the full
stage dependency, failure, cancellation, and resource-retention rules before
those profiles become executable.

### Hard feasibility and cost

Hard checks include:

- successful symbolic-affinity binding and device/domain compatibility;
- access mode, visibility/coherence protocol, and completion dependency;
- allocation/import/resource-kind support;
- capacity policy, checked byte range, maximum size, and alignment;
- a legal directed enforcer path and source/destination ownership;
- encoding preservation or an explicit conversion stage;
- program-profile limits on device and queue count.

Allocation failure after `RoutingCommit` is an execution failure, not evidence
that another plan is semantically applicable. A reserving allocator contract
could establish capacity earlier, but volatile free memory is otherwise cost
or policy evidence rather than a portable guarantee.

Cost inputs include bytes, fixed launch/setup latency, effective directional
bandwidth, topology, residency/migration probability, page faults, contention,
NUMA distance, copy-engine/queue availability, synchronization, lost overlap,
temporary peak bytes, repack work, and recomputation. A legal peer-access load
can still be slower than a copy; unified physical memory can still make a
private or packed representation preferable.

## Worked example

```text
logical: x -> normalize -> matmul -> y

physical requirement:
  K0/K1 execute at accelerator affinity A
  x readable at A in encoding E
  temporary t writable/readable at A in encoding T
  y returned through consumer boundary B
```

Possible plans include direct access to imported/shared `x`, upload to a
private domain, or pure recomputation of a cheap producer. `t` can stay private.
Returning `y` may be an allocation already consumable by the caller, a copy to
a host-accessible domain, or an exported dependency-carrying handle. Those are
different physical programs with the same tensor semantics. The initial profile
admits only the case where all inputs are already accessible at A and the
consumer accepts the resulting device allocation.

## Bounded model spike

[`spikes/placement/placement_domain_model.rs`](../../../spikes/placement/placement_domain_model.rs)
implements symbolic/live device identities, domain/access and directed movement
graphs, required versus delivered properties, hard capacity/alignment checks,
a least-cost legal movement path, and the initial single-device verifier.

Eight tests establish:

- equal runtime ordinals in different sessions are not equal device identity;
- direct access is an affinity-domain relation, not a domain label;
- live topology chooses one peer edge or two staged edges;
- intermediate staging domains satisfy capacity and alignment constraints;
- capacity/alignment failures are not cost penalties;
- a transfer cannot silently change storage encoding;
- the initial profile rejects multiple affinities and movement stages; and
- semantic target-property and physical-affinity IDs have disjoint types.

For a 1 MiB model value, the configured peer path selects one edge with
1,048,586 abstract cost units; removing peer capability selects two staged
edges. These units only demonstrate deterministic graph selection. They are not
device measurements and must never seed a production cost model.

## Remaining experiments and decisions

1. The transfer/lifetime ticket should refine `MovementEdge` into executable
   stages with queues, timepoints, coherence, partial failure, cancellation,
   and resource retention.
2. Metal and CUDA spikes should enumerate live multi-device topology and
   measure direct access, peer copy, and host staging independently.
3. The multi-device/sharding gate should decide whether device meshes and
   communication become a Tiler optimization layer or remain frontend-provided.
4. External-storage research should define import/read/prefetch/persistence
   semantics rather than adding `Disk` to `MemoryDomainClass`.
5. Runtime adapters need conformance tests proving that their concrete storage
   modes, allocators, and synchronization implement the advertised domain and
   edge contracts.

## Traceability

ADR 0047 adopts this physical-property boundary. The
[placement spike](../../../spikes/placement/README.md) exercises the model;
distributed scheduling and external-storage semantics remain deliberately deferred.
