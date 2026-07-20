# 0047: Model placement as physical properties over capability graphs

**Status:** proposed

## Context

Tensor semantics do not normally depend on which CPU or accelerator executes
them, but an executable program must place work and storage where every access
is legal and visible. Device APIs do not expose one portable memory hierarchy.
Shared, managed, private, pinned, peer, workgroup, and external resources differ
in access, coherence, lifetime, topology, and ownership; their physical backing
also changes across integrated and discrete systems.

Putting runtime device ordinals or storage modes on semantic nodes would make
portable meaning depend on one process and backend. Conversely, treating
placement as an informal kernel annotation would hide required transfers and
synchronization from whole-program correctness and cost.

## Proposed decision

Semantic tensor graphs remain device-neutral except for an explicitly authored
and independently bound semantic `TargetProperty` root under ADR 0008.
Ordinary device choice never creates or changes such a root.

Selected physical programs express placement as required and delivered
properties. A requirement names a symbolic execution affinity, admitted memory
domain, access mode, value version, storage encoding, byte range, alignment,
and availability dependency. A delivered placement names a materialized value,
allocation/domain, authoritative version, visibility state, ownership, and the
dependency after which it is usable.

Target/runtime profiles describe memory as a directed capability multigraph:

- memory-domain nodes declare allocation/import, scope, capacity, alignment,
  and resource-kind contracts;
- affinity-to-domain edges declare read/write/atomic accessibility,
  addressability, visibility, coherence, and completion requirements; and
- directed movement/import edges declare mechanisms, hard predicates,
  encoding preservation, resulting coherence, and cost inputs.

No portable total order or implicit meaning is inferred from domain names.
Transparent caches are costs, execution-local scratch has scoped lifetime, and
external storage is an explicit I/O resource connected by import/read stages.

The optimizer satisfies required placement directly or through explicit
enforcers: transfer/import, materialization/repacking, or legal recomputation.
Transfer does not silently convert encoding. Enforcers, new materialized
versions, dependencies, and costs are represented at `KernelProgram` scope.
Hard accessibility/coherence/capacity/alignment checks remain separate from
topology-dependent cost.

Portable plans use symbolic affinities and relational constraints. Runtime
preflight binds them to runtime-scoped live-device identities and concrete
domain, allocator, pool, and execution capabilities. Bare runtime ordinals are
not portable identities. Concrete device handles, queues, events, and storage
objects stay below the compiler/runtime boundary.

The initial execution profile remains one symbolic affinity, one live device,
and one ordered command stream. Inputs must already be accessible, and all
stages, temporaries, and outputs use that affinity. It rejects executable
cross-device transfers, placement search, sharding, collectives, and external
I/O while retaining explicit extension points for later profiles.

## Consequences

- Existing single-device `KernelProgram`s remain valid and simple.
- CPU, integrated/discrete GPU, peer-device, and shared-memory systems need not
  pretend to share a linear storage hierarchy.
- Whole-program verification can prove every read's accessibility, version,
  coherence, and availability rather than relying on runtime convention.
- Placement alternatives and transfer/recomputation tradeoffs enter physical
  search and explain output without changing tensor semantics.
- Multi-device execution still requires a separate transfer/synchronization
  contract before it becomes executable.
- External storage and sharding remain addable as explicit layers instead of
  overloading memory domains or ordinary value placement.

## Alternatives considered

A `Disk < RAM < VRAM < Scratch` enum cannot represent unified physical memory,
pairwise peer access, explicit coherence, or execution-scoped scratch. Concrete
device IDs and storage modes make artifacts process- and backend-specific.
Implicit copies hide failure, synchronization, memory use, and cost. Putting
placement directly on semantic nodes prevents target-independent equivalence
reasoning. Modeling every future queue and distributed collective now would
overcomplicate the sound single-device slice before its execution contracts
exist.
