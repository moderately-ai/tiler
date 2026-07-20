---
schema: "tiler-doc/v1"
id: "tiler.research.program-planning.kernel-program-buffer-plan"
kind: "research"
title: "KernelProgram and conservative buffer planning"
topics: ["program-planning", "buffers", "scheduling"]
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["primary-source-synthesis", "executable-model"]
informs: ["tiler.contract.architecture", "tiler.contract.fusion-and-scheduling"]
reproduced_by: ["tiler.spike.program-planning"]
ticket: "kernel-program-buffer-plan"
---

# KernelProgram and conservative buffer planning

**Status:** completed research adopted into the compiler contracts
**Ticket:** `kernel-program-buffer-plan`

## Outcome

A `KernelProgram` should be a target-specific, consumer-agnostic executable DAG
over scheduled kernels, explicit materializations, validation/enforcement work,
and opaque calls. It binds logical materialized values to storage, carries a
separate host-preflight expression graph, and exposes an ordered list of named
outputs. It does not collapse semantic values, views, materializations, and
allocations into one identity.

The conservative first execution profile is one physical device and one
ordered command stream. A canonical topological order determines encoding
order. Edges still carry the reason for ordering; in particular, storage reuse
must add a `StorageHandoff` edge from the old value's final user to the new
value's writer. An accidental order between otherwise independent nodes is not
a reusable correctness proof and would break as soon as concurrent execution
is admitted.

The baseline buffer plan uses one distinct allocation per output and
cross-stage temporary. A later, optional conservative pass may reuse storage
only between internal temporaries with proven non-overlapping lifetimes,
compatible storage requirements, an explicit happens-before handoff, and no
live alias. Inputs and outputs do not participate in reuse in the first
profile. In-place updates, suballocation, multiple devices, multiple queues,
device-derived shapes, and conditional device control flow remain representable
future extensions rather than implicit behavior.

## Facts from primary systems

These facts inform the proposal; they are not Tiler requirements by themselves.

- MLIR's One-Shot Bufferize analyzes immutable tensor SSA use-def chains before
  rewriting them to mutable buffers. It builds alias/equivalence information,
  detects read-after-write conflicts, and may always-copy when analysis cannot
  justify in-place bufferization. Bufferization deliberately does not also
  solve deallocation. This supports keeping value semantics, buffer assignment,
  and ownership/lifetime as separate concerns. See MLIR's
  [Bufferization documentation](https://mlir.llvm.org/docs/Bufferization/).
- MLIR's ownership-based deallocation models responsibility to deallocate
  separately and retains aliases that remain live. When static information is
  insufficient it may require runtime alias tests. Tiler's stricter first
  profile can reject such uncertainty instead of importing runtime ownership
  machinery. See
  [Ownership-based Buffer Deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/).
- IREE Stream lowers tensors into size-carrying resources with explicit
  lifetimes and sequences asynchronous availability with timepoints. It states
  that using a resource before its corresponding timepoint is undefined, and
  that explicit timepoints permit later aliasing. Its allocation-reuse pass
  reuses compatible transient allocations based on timeline ordering; its
  simple transient-emplacement experiment explicitly notes that interval-only
  liveness is a limitation. See the
  [Stream dialect](https://iree.dev/reference/mlir-dialects/Stream/) and
  [Stream passes](https://iree.dev/reference/mlir-passes/Stream/).
- IREE makes copy-on-write explicit before later copy elision and emplacement.
  That separation is useful precedent for making materialization and aliasing
  decisions inspectable instead of silently changing immutable value meaning.
- XLA describes buffer analysis and runtime-memory allocation as a distinct
  target-independent optimization concern after graph optimization. Its buffer
  assignment implementation distinguishes logical buffers from allocation
  slices, and its heap simulation/live-range machinery supplies reuse choices.
  See the official [XLA architecture](https://github.com/openxla/xla/blob/main/docs/architecture.md)
  and [`BufferAssignment`](https://github.com/openxla/xla/blob/main/xla/service/buffer_assignment.h).
- TVM's unified static memory planner was motivated by the loss from planning
  inter-operator and intra-operator storage independently. Its public pass is
  explicitly whole-program across primitive functions. This supports planning
  cross-dispatch temporaries at `KernelProgram` scope, while keeping
  threadgroup/private storage inside a scheduled kernel. See TVM's
  [`UnifiedStaticMemoryPlanner`](https://tvm.apache.org/docs/reference/api/doxygen/namespacetvm_1_1tir_1_1usmp.html).

## Inferences for Tiler

1. Immutable tensor/value semantics should survive until program formation;
   allocation aliasing is an explicit lowering choice.
2. Program dependencies, value liveness, and storage ownership are related but
   not interchangeable. A later asynchronous executor will need happens-before,
   not positions in one chosen list.
3. Cross-dispatch memory planning requires visibility across every stage in the
   program. Kernel-local shared/private storage remains part of its schedule.
4. Unknown aliasing can be handled conservatively. The first profile need not
   reproduce mature compilers' runtime alias checks or in-place heuristics.
5. Allocation count and peak bytes are costs. Initialization, range safety,
   ordering, alignment, and alias legality are hard feasibility constraints.

## Proposed normalized model

All IDs below are stable newtypes. Collections and expression arenas have a
canonical order, and there are no identity-affecting implicit defaults.

```text
KernelProgram {
  schema_version,
  execution_profile,
  inputs,
  host_preflight,
  stages,
  dependencies,
  materialized_values,
  views,
  buffer_plan,
  outputs,
  routing_commit_contract,
}

ExecutionProfile {
  target_domain,
  device_count = 1,
  command_streams = 1,
  ordered_stream = true,
}

ProgramStage {
  id,
  body: ScheduledKernel
      | MaterializeCopy
      | ValidationEnforcement
      | OpaqueCall,
  reads,
  writes,
  host_operands,
}

Dependency {
  predecessor,
  successor,
  reason: Data(MaterializedValueId)
        | Validation(ValidationWitnessId)
        | Effect(EffectDomainId)
        | StorageHandoff(AllocationId),
}
```

A `KernelSubprogram` selected for a region is flattened into ordinary program
stages and dependencies. Its grouping is retained as origin/explain metadata,
not as a second nested execution semantics. A materialization does not
necessarily create a copy node: a scheduled producer may write its result
directly to the materialized value's binding. Layout conversion or a necessary
copy is an explicit `MaterializeCopy` stage.

`OpaqueCall` declares read/write access, completion semantics, storage
requirements, applicable numerical contract, and effect domain. An undeclared
side effect or scratch buffer is invalid. Effect dependencies serialize calls
whose contracts do not establish independence.

### Host preflight is a separate expression DAG

Host expressions are pure checked expressions, not device stages. Their roots
are admitted host-available facts: input shapes/layout metadata, interface
bindings, constants, and target/prepared-pipeline facts available in the stated
preflight phase. They compute:

- output and temporary shapes;
- checked byte sizes and alignments;
- applicability guards;
- launch dimensions and scalar ABI values.

Expressions have explicit integer widths, signedness, overflow behavior, and
bounded evaluation. Every referenced fact carries an availability phase. A
device-produced value cannot feed allocation size or launch preflight in the
first profile. Such a program would require a later staged host/device control
model rather than a disguised ordinary dependency.

### Values, materializations, views, and allocations

```text
MaterializedValue {
  id,
  origin: SemanticValueId | InternalScratch | OpaqueResult,
  role: Input | Output | Temporary,
  tensor_type,
  logical_shape,
  storage_encoding,
  required_bytes,
  required_alignment,
  memory_space,
  definition: ExternalBinding | StageId,
}

ViewBinding {
  id,
  base: MaterializedValueId,
  byte_offset,
  reachable_byte_range,
  shape,
  strides,
  access: Read | Write | ReadWrite,
}

Allocation {
  id,
  capacity_bytes,
  alignment,
  memory_space,
  usage_capabilities,
  ownership: External | Program,
}

ValueAllocationBinding {
  value,
  allocation,
  offset,
  extent_bytes,
}
```

These identities answer different questions:

- a semantic value says what tensor result means;
- a materialized value says that the result crosses a program-stage boundary;
- a view says how a stage addresses a range of that stored value;
- an allocation says which storage object owns the bytes.

An input is externally bound and has no defining stage. An output or temporary
has exactly one defining stage in the first profile. Views do not independently
own or extend storage and may not escape their base allocation's range or
lifetime.

### Named outputs

Program outputs are an ordered list, not inferred graph leaves:

```text
ProgramOutput {
  key: ProgramOutputKey,
  value: MaterializedValueId,
  abi_role,
  display_label?,
}
```

`ProgramOutputKey`, ordering, and ABI role participate in program/artifact
identity. A display label is diagnostic only. Every selected semantic root has
exactly the declared output coverage; dead graph leaves need not become
outputs. The first profile requires each returned output to have a fresh,
program-owned allocation distinct from inputs, other outputs, and temporaries.

## Dependency and execution contract

The stage graph must be acyclic. Each read of a non-input value has a `Data`
dependency reachable from its unique writer. Validation witnesses, opaque
effects, and storage handoffs similarly require typed reachable dependencies.
The verifier derives one deterministic topological order by stable `StageId`
tie-breaking.

The first runtime contract encodes that order on one ordered stream. This
linearization is an execution-profile guarantee, not extra dependency
information. Therefore:

- incomparable stages are not promised to run concurrently in this profile;
- a cost model may still value the DAG's critical path for a future profile;
- changing the order must preserve all typed dependencies;
- reuse-induced order is represented by `StorageHandoff`, even if the current
  stream would happen to execute the stages in the same order.

This is analogous to a database physical plan whose operators carry required
properties: the total execution order is a chosen implementation, while a
dependency is a correctness property that every legal implementation must
preserve. The analogy stops at GPU completion: a host call that encodes a stage
does not imply its buffers are no longer in use. Device completion or an
equivalent retained dependency governs resource release and output visibility.

## Initialization and liveness

For each materialized value, define:

- `definition`: the external binding or its unique full writer;
- `first_use` and `last_use`: stage events under the selected execution profile;
- `logical_lifetime`: definition completion through final consumer completion;
- `resource_retention`: until the runtime can prove device use has completed.

Every read must happen after initialization. The initial profile requires the
defining stage to initialize the value's complete logical extent on every
executed path before any read or publication. Partial/tiled writes are legal
inside one defining stage only when that stage's verified ownership/coverage
proves the complete result. Multiple program-stage writers require a future
explicit versioned/region-write model.

A zero-element tensor has no logical element writes, but still has a defined
shape and follows the program's zero-size allocation/ABI policy. It is not an
uninitialized ordinary tensor.

For fan-out, one materialized temporary remains live through the last of all
consumers. If the physical planner instead chooses legal recomputation, each
recomputed value receives its own materialized identity and no shared live
temporary exists. Duplication must already be legal under semantic and
numerical contracts; buffer planning does not authorize it.

## Scratch storage

The word "scratch" does not define a special hidden lifetime:

- per-thread, subgroup, and workgroup storage belongs to a single
  `KernelSchedule` and cannot cross a dispatch;
- a multi-dispatch reduction's partials are ordinary cross-stage temporary
  materialized values;
- opaque provider scratch is an explicitly declared program temporary with
  checked size, alignment, memory space, and access/lifetime;
- host scratch, if later supported, belongs to a distinct memory space and
  execution profile.

For a two-pass reduction, stage 0 fully defines a partials temporary, stage 1
depends on and reads it to define the output, and the temporary remains retained
until stage 1 completes. The numerical semantics of the partial reduction are
owned by the selected reduction realization; calling the allocation scratch
does not erase dtype or rounding boundaries.

## Conservative buffer assignment

### Baseline

The mandatory baseline is always legal when resource sizes are admitted:

- inputs are external allocations and may alias one another;
- every output has one distinct program-owned allocation;
- every cross-stage temporary has one distinct program-owned allocation;
- no suballocation, input/output alias, output/output alias, in-place update,
  or temporary reuse occurs.

Because inputs may alias, a kernel may not assume input `noalias` unless an
independent proof or runtime guard supplies it.

### Optional temporary reuse

Two internal temporaries may share an allocation only if the verifier proves
all of the following:

1. both are program-owned internal temporaries, never inputs or outputs;
2. their memory space, storage encoding/addressability, and usage capabilities
   are compatible;
3. the allocation's checked capacity is at least the value's required bytes
   for every applicable runtime binding, and its alignment satisfies the
   value's requirement;
4. no executions of their logical lifetimes overlap;
5. an explicit `StorageHandoff` dependency orders the old value's final users
   before the new value's defining writer;
6. no view or alias of the old value remains live across the handoff;
7. the new writer fully initializes every byte logically reachable by its
   later readers before those reads.

In the one-stream profile, canonical topological positions provide a simple
conservative interval calculation. They are not the fundamental proof: the
handoff's reachability is. A future multi-stream planner can replace interval
comparison with event/timepoint interference without changing the value,
allocation, or dependency concepts.

Unknown symbolic capacity ordering rejects reuse, not the program. The planner
falls back to distinct allocations. Choosing which legal temporaries share an
allocation is a cost problem; proving a proposed assignment legal is verifier
work.

## Failure, fallback, and publication

`RoutingCommit` is a program-level state transition:

```text
artifact validation
  -> host expressions and guards
  -> target/pipeline/opaque-call preflight
  -> select one complete program
  -> RoutingCommit
  -> acquire output and temporary storage
  -> encode/submit stages
  -> publish completed or dependency-carrying outputs
```

Before `RoutingCommit`, an applicability or capability rejection may choose a
different complete program or an unfused/reference route. No output or scratch
allocation and no device work may have begun. After commit, allocation,
encoding, submission, asynchronous device, and opaque-call failures return an
error; they never replay fallback. Outputs remain private until their complete
initialization and required validation are ordered.

The contract does not require a synchronous host wait before return. A consumer
runtime may publish a dependency-carrying output handle if it guarantees
visibility ordering and retains all referenced resources until device
completion. This is deliberately a runtime adapter obligation, not a Candle,
CUDA, or Metal assumption in `KernelProgram`.

## Whole-program verifier

Verification should be deterministic and separated into phases so diagnostics
name the rejected invariant:

1. **Structural:** canonical IDs and ordering, valid references, bounded host
   expressions, unique output keys, and an acyclic stage DAG.
2. **Coverage:** selected semantic roots map to declared outputs; every
   non-input value has one writer; deliberate recomputation is accounted for.
3. **Dependency:** every read, witness, effect, and storage handoff has the
   required reachable predecessor.
4. **Initialization:** writer coverage is complete and dominates every read and
   output publication.
5. **Storage:** sizes and offsets are checked, views are in range, memory spaces
   and alignments match, forbidden aliases are absent, and allocation ownership
   is coherent.
6. **Lifetime/reuse:** all uses fit the logical lifetime, runtime retention is
   representable, reused values do not interfere, and no alias crosses a
   handoff.
7. **Stage contracts:** kernel schedules, materialization conversions,
   validations, and opaque calls agree with bound value types, numerical
   realizations, access modes, and target requirements.
8. **Execution/failure:** the execution profile realizes every dependency;
   all route-sensitive work precedes `RoutingCommit`; no fallback edge exists
   after commit.

Region-local or schedule-local verification is necessary but not sufficient:
it cannot see fan-out lifetimes, output completeness, inter-stage scratch,
opaque effects, or allocation reuse across stages.

### Stable rejection and explain vocabulary

At minimum, explain output distinguishes:

- `dependency-cycle`, `missing-data-dependency`,
  `missing-validation-dependency`, `unordered-effect`;
- `missing-writer`, `multiple-writers`, `uninitialized-read`,
  `incomplete-output`;
- `view-out-of-range`, `view-lifetime-escape`, `forbidden-alias`;
- `reuse-lifetime-overlap`, `reuse-missing-handoff`,
  `reuse-live-alias`, `reuse-capacity-unproven`,
  `reuse-alignment-mismatch`, `reuse-memory-space-mismatch`;
- `host-fact-unavailable`, `host-expression-overflow`;
- `fallback-after-routing-commit`.

Legal but rejected-for-cost alternatives use a separate vocabulary, such as
`higher-peak-live-bytes`, `extra-materialization-traffic`, `more-dispatches`,
or `higher-fragmentation`. An explain record should never report a hard
correctness failure as a cost-model preference.

## Worked examples

### Fan-out and multiple outputs

```text
input x
  -> K0 writes temporary t
       |-> K1 reads t, writes output "scores"
       `-> K2 reads t, writes output "summary"
```

`t` remains live until both `K1` and `K2` complete. The outputs are separately
allocated and ordered by their public output keys. A planner may choose either
consumer first on the one stream, but cannot reuse `t` until the later consumer
has completed.

### Multi-dispatch reduction

```text
input x
  -> ReducePartials writes scratch p
  -> ReduceFinal reads p, writes output "sum"
```

`p` is a typed temporary, not anonymous workspace. Its allocation may be reused
only after `ReduceFinal` and an explicit storage handoff. Both stages carry the
one selected reduction realization's numerical evidence.

### Rejected aliases

```text
input x allocation A
output y allocation A       // reject: output/input alias

output y allocation B
output z allocation B       // reject: output/output alias

temporary t allocation C
view v of t remains live
temporary u allocation C    // reject until v's final use precedes u's writer
```

## Hard constraints versus costs

| Hard feasibility | Estimated or measured cost |
| --- | --- |
| Acyclic typed dependencies | Dispatch count and overhead |
| Initialization before read | Peak live bytes |
| Complete named outputs | Allocation count and fragmentation |
| View bounds and lifetime | Materialization traffic |
| Alias and handoff legality | Benefit of reuse or recomputation |
| Capacity, alignment, usage, memory space | Critical path and concurrency opportunity |
| Host-expression availability and overflow | Allocation/encoding overhead |
| No fallback after commit | Runtime latency variance |

## Bounded spike

`spikes/program-planning/kernel_program_model.rs` implements a dependency-free
compile-checking model for:

- deterministic topological ordering and reachability;
- unique writers and initialization-before-read;
- fan-out lifetime calculation and ordered named outputs;
- explicit view ranges whose uses extend their base value's lifetime;
- distinct output/input allocation rules;
- conservative internal-temporary reuse with explicit handoff;
- cross-dispatch reduction scratch;
- rejection of fallback after `RoutingCommit`.

It intentionally does not implement a best-fit allocator, symbolic theorem
prover, asynchronous timeline, suballocation, control flow, or target API.
Those would obscure the contract being tested.

Run it with:

```sh
rustc --edition 2021 --test \
  spikes/program-planning/kernel_program_model.rs \
  -o /tmp/tiler-program-plan-spike
/tmp/tiler-program-plan-spike
```

## Measurements and later decisions

The following remain experiments or later architecture decisions rather than
gaps in the first verifier:

1. Compare no-reuse, greedy reuse, and exact/ILP assignment on representative
   programs using peak bytes, allocation count, planning time, and execution
   latency. A legal reuse need not be profitable.
2. Measure whether adapter allocation pools make compile-time reuse redundant
   for common workloads; identity and explainability still differ.
3. Define a multi-stream/timepoint execution profile before admitting
   concurrency. It must replace interval-only interference with explicit event
   reachability and incorporate queue/device placement in program identity.
4. Design partial writes, in-place updates, suballocation, and external-output
   aliasing only with explicit range/version and ownership semantics.
5. Define device-derived shape and conditional execution as staged program
   control, not by allowing device results into today's host expression DAG.
6. Specify cross-device transfer and storage-tier stages in the separate
   placement research; the initial single-device profile leaves explicit
   extension points and does not claim those operations are free annotations.
